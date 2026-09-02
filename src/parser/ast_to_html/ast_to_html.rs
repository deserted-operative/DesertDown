use super::ast_utils::{AST, NO_NODE, Node, NodeId};
use super::html_escape_utils::write_out_escaped_bytes;
use super::links::LinkPermissions;
use super::conditional_styling::{get_range_for_callout_type_chars, get_range_for_num_in_num_list_item};
use super::tables::{MAX_ALIGNED_TABLE_COLUMNS, TableColumnAlignment};
use super::node_utils::NodeType;

#[inline]
/// Takes a node type, and returns whether a node of that type would be a block continuation
pub(super) const fn is_continuation_node(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::IndentContinuation
            | NodeType::CalloutContinuation
            | NodeType::DashListItemContinuation
            | NodeType::PlusListItemContinuation
            | NodeType::AsteriskListItemContinuation
            | NodeType::NumListItemContinuation
    )
}

#[inline]
/// Takes a node type, and returns whether a node of that type would be a content entry within a block
pub(super) const fn is_content_node(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::IndentContent
            | NodeType::CalloutContent
            | NodeType::DashListItemContent
            | NodeType::PlusListItemContent
            | NodeType::AsteriskListItemContent
            | NodeType::NumListItemContent
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Containers for blocks that may appear in the AST.
/// Required as the AST provides no indication on how to handle list items or table rows, deferring that decision to the renderer
enum BlockContainers {
    None,
    UnorderedList,
    OrderedList,
    Table,
}

impl BlockContainers {
    #[inline]
    /// Takes a node type (in use, should be a block node type), and returns what container that block should be put in for the HTML
    const fn get_container_type_for_block(block_node_type: NodeType) -> BlockContainers {
        match block_node_type {
            NodeType::DashListItem | NodeType::PlusListItem | NodeType::AsteriskListItem => {
                BlockContainers::UnorderedList
            }

            NodeType::NumListItem => BlockContainers::OrderedList,

            NodeType::TableRowBlock => BlockContainers::Table,

            // default - the type of the block is enough to go to HTML, and doesn't need wrapping
            _ => BlockContainers::None,
        }
    }
}

/// Frames for pushing to a stack when traversing a node's child node list.
/// Should be popped from stack when said child node list has been fully traversed
pub(super) struct TreeFrame {
    // next sibling to process at this level in the AST
    // NO_NODE when level has been fully traversed
    next_sibling: NodeId,

    // HTML closing tag to be written out once the node's child nodes have been traversed
    // for a "silent" frame (see below), this is empty
    closing_tag: &'static [u8],

    // type of container which block current node belongs to (or delimits) should be wrapped in, in order to go from AST to HTML
    block_container: BlockContainers,

    /*
        a "silent" frame does not write out any tags to the HTML
        it's used for content nodes (eg. DashListItemContent, H1Content, etc...) in the AST
        required as content nodes can close a block, so need to propagate block_container down
    */
    is_silent: bool,
}

/// Traverses AST, and builds HTML byte array for HTML
pub(super) struct HtmlRenderer<'input_string> {
    // AST of input which HTML is requested for
    pub(super) ast: &'input_string AST,

    // input string, as byte array
    // AST lifetime (intuitively) bounded by lifetime of input string
    pub(super) input_string_bytes: &'input_string [u8],

    // output array (of bytes)
    pub(super) html_output_array: &'input_string mut Vec<u8>,

    // working stack for AST to HTML logic
    // top of stack is stored at end of array
    pub(super) stack: Vec<TreeFrame>,

    /*
        vars for tracking table state
        works as tables only allow inlines in cells, so at most one table can be open at any given time, thus can keep here, rather than on TreeFrames
    */
    // true when writing out into the table header row
    // NOTE that the automaton refers to the separator row as the header row. Here, the header row is the row above the separator row
    pub(super) in_table_header_row: bool,

    // column index of the next cell in the current row
    pub(super) table_column: usize,

    // number of entries in table_alignments the separator row specified
    pub(super) aligned_column_count: usize,

    // as many entries as is the maximum number of aligned columns, each of which takes an alignment
    pub(super) table_alignments: [TableColumnAlignment; MAX_ALIGNED_TABLE_COLUMNS],

    // whether links and embedded links are allowed or blocked in HTML output
    pub(super) link_permission_status: LinkPermissions,
}

// to-HTML specific methods for AST struct
impl AST {
    /// Takes an AST, the input string, and an array, and appends the HTML for that AST to the array
    pub fn ast_to_html_to_array(&self, input_string: &str, output_array: &mut Vec<u8>, link_permission_status: LinkPermissions) {
        // reserve space for output bytes of HTML
        // guess based on input string length, not AST size, as output bytes need to include content chars that AST nodes fold into content
        output_array.reserve(input_string.len() + input_string.len() / 2 + 256);

        // init the HTML renderer
        let mut renderer = HtmlRenderer {
            ast: self,
            input_string_bytes: input_string.as_bytes(),
            html_output_array: output_array,
            stack: Vec::with_capacity(32), // stack depth grows with nesting depth in the AST
            in_table_header_row: false,
            table_column: 0,
            aligned_column_count: 0,
            table_alignments: [TableColumnAlignment::None; MAX_ALIGNED_TABLE_COLUMNS],
            link_permission_status,
        };

        // process the AST for the HTML
        renderer.process_ast(self.root);
    }

    /// Takes an AST, the input string, and returns the HTML for that AST as a string
    pub fn ast_to_html_to_string(&self, input_string: &str, link_permission_status: LinkPermissions) -> String {
        // create output array
        let mut output_array = Vec::new();

        // populate output array
        self.ast_to_html_to_array(input_string, &mut output_array, link_permission_status);

        // convert to string, and return
        // panics if not a valid UTF-8 string, but the to-AST logic ensures it is
        String::from_utf8(output_array).expect("Rendered HTML was not valid UTF-8")
    }

    /// Takes an AST, the input string, and writes the HTML for that AST to standard output
    pub fn ast_to_html_to_stdout(&self, input_string: &str, link_permission_status: LinkPermissions) {
        /*
            note that for portability with windows console, output must be UTF-8
            this is checked in the to-AST implementation, as String and &str are guaranteed to refer to valid UTF-8
            the to-HTML functionality also ensures that the output is UTF-8
        */

        // create output array
        let mut output_array = Vec::new();

        // populate output array
        self.ast_to_html_to_array(input_string, &mut output_array, link_permission_status);

        // get access to process' standard output, and lock it so that the write can proceed in one go
        // lock is released as function goes out of scope
        let mut stdout = std::io::stdout().lock();

        // so .write_all() can be used below
        use std::io::Write;

        // if error is broken pipe (eg. downstream program exited early), don't want to error
        // otherwise, write error to standard error, not standard output (as standard output may be what failed)
        if let Err(error) = stdout.write_all(&output_array)
            && error.kind() != std::io::ErrorKind::BrokenPipe
        {
            eprintln!("Failed to write HTML: {error}");
        }
    }
}

impl<'render> HtmlRenderer<'render> {
    /// Traverses AST nodes below the supplied root node, to produce the output byte array
    fn process_ast(&mut self, root_node: NodeId) {
        // nothing to do for an empty AST
        if root_node == NO_NODE {
            return;
        }

        /*
            note that the root node itself doesn't contribute any html tags to the output
            this allows the HTML to have the document structure built up around it, independent of the content
        */

        // get first child node of the root
        let first_child_of_root = self.ast.get_node(root_node).first_child;

        // push a frame to the working stack, with no closing HTML tag for the node
        self.stack.push(TreeFrame {
            next_sibling: first_child_of_root,
            closing_tag: b"",
            block_container: BlockContainers::None, // root does not need wrapping
            is_silent: false,
        });

        // walking frame is always on top of stack (ie. end of the array)
        // terminate when stack is empty
        while let Some(tree_frame) = self.stack.last_mut() {
            // get ID of next node in the current node's child list for processing
            let node_id = tree_frame.next_sibling;

            // if NO_NODE the list of child nodes is empty, so frame can be unwound
            if node_id == NO_NODE {
                // pop frame off of working stack
                let Some(processed_frame) = self.stack.pop() else {
                    // shouldn't be None, so if it is, just break, rather than panic
                    break; // break not continue, so there's no risk of infinite loop
                };

                // does frame NOT contribute any HTML tags to the output?
                if processed_frame.is_silent {
                    /*
                        frame that has finished processing doesn't contribute any HTML tags to output

                        with frame being closed, push the container for the block down to the next frame on the stack
                        this handles the case of several list items in a callout, where each list item in its own callout content entry is now grouped in one container list, rather than one per list item
                    */
                    if let Some(parent) = self.stack.last_mut() {
                        parent.block_container = processed_frame.block_container;
                    }
                } else {
                    // frame that has finished processing does contribute HTML tags to output

                    // therefore, close the container this frame opened (if it opened one)
                    self.close_container(processed_frame.block_container);

                    // push the closing tag of the node associated with the frame (may be nothing: b"")
                    self.html_output_array
                        .extend_from_slice(processed_frame.closing_tag);
                }

                continue;
            }

            /*
                here, list of child nodes was not empty, therefore there's at least one node to process
            */

            // deref the node
            let node = *self.ast.get_node(node_id);

            // advance the cursor to the next child in the list of child nodes
            tree_frame.next_sibling = node.next_sibling;

            // process the node (recurses into the node's child list where required)
            self.process_node(node_id, &node);
        }
    }

    /// Takes a node, and appends the HTML it contributes to the output array.
    /// Recurses into the node's child list where required
    fn process_node(&mut self, node_id: NodeId, node: &Node) {
        // how the node is processed depends on its type:
        match node.node_type {
            /*
                structural nodes such as openers, continuations, closers, and delimiters, are not relevant for HTML output
                therefore, nothing to do for them
            */
            NodeType::Document
            | NodeType::NoOp
            | NodeType::Emphasis
            | NodeType::Highlighting
            | NodeType::H1Opener
            | NodeType::H2Opener
            | NodeType::H3Opener
            | NodeType::H4Opener
            | NodeType::H5Opener
            | NodeType::H6Opener
            | NodeType::H1Closer
            | NodeType::H2Closer
            | NodeType::H3Closer
            | NodeType::H4Closer
            | NodeType::H5Closer
            | NodeType::H6Closer
            | NodeType::IndentOpener
            | NodeType::IndentContinuation
            | NodeType::CalloutOpener
            | NodeType::CalloutType // as it's handled with CSS
            | NodeType::CalloutHeaderTerminator
            | NodeType::CalloutContinuation
            | NodeType::DashListItemOpener // check list styling handled with CSS
            | NodeType::DashListItemContinuation
            | NodeType::PlusListItemOpener // check list styling handled with CSS
            | NodeType::PlusListItemContinuation
            | NodeType::AsteriskListItemOpener // check list styling handled with CSS
            | NodeType::AsteriskListItemContinuation
            | NodeType::NumListItemOpener
            | NodeType::NumListItemContinuation
            | NodeType::CodeBlockOpener
            | NodeType::CodeBlockOpenerTerminator
            | NodeType::CodeBlockOpenerContent
            | NodeType::CodeBlockLanguage
            | NodeType::CodeBlockCloser
            | NodeType::TableWall
            | NodeType::TableHeaderRow
            | NodeType::LeftAlignedTableHeaderRow
            | NodeType::CentredTableHeaderRow
            | NodeType::RightAlignedTableHeaderRow
            | NodeType::EmphDelimiter
            | NodeType::DoubleEmphDelimiter
            | NodeType::TripleEmphDelimeter
            | NodeType::HighlightC1Delimiter
            | NodeType::HighlightC2Delimiter
            | NodeType::UnderlineDelimiter
            | NodeType::StrikethroughDelimiter
            | NodeType::InlineCodeOpener
            | NodeType::InlineCodeCloser
            | NodeType::TagOpener
            | NodeType::TagContent
            | NodeType::LinkOpener
            | NodeType::LinkContent
            | NodeType::LinkCloser
            | NodeType::EmbeddedLinkOpener
            | NodeType::EmbeddedLinkContent
            | NodeType::EmbeddedLinkCloser => {}

            /*
                escape backslashes are not to be shown in HTML - only the literals that follow them are
                therefore, nothing to do for them
            */
            NodeType::EscapeBackslash => {}

            /*
                comments - both block and inline - are not to be shown in HTML
                therefore, nothing to do for them
            */
            NodeType::CommentBlock
            | NodeType::InlineComment
            | NodeType::CommentBlockOpener
            | NodeType::CommentBlockContent
            | NodeType::CommentBlockCloser
            | NodeType::InlineCommentOpener
            | NodeType::InlineCommentContent
            | NodeType::InlineCommentCloser => {}

            /*
                literals and math delimiters (both block and inline) should be pushed to the output
                need to escape chars that don't work so well in HTML outupt - ie. &, >, <, "
            */
            NodeType::Literal
            | NodeType::MathBlockOpener
            | NodeType::MathBlockContent
            | NodeType::MathBlockCloser
            | NodeType::CodeBlockContent
            | NodeType::InlineCodeContent
            | NodeType::InlineMathOpener
            | NodeType::InlineMathContent
            | NodeType::InlineMathCloser => {
                self.escape_and_push_to_html(node.start, node.end);
            }

            /*
                BLOCKS
            */
            
            NodeType::Paragraph => {
                /*
                    note that vertical spacing in the output is exclusively specified by blank lines in the input string
                    no block reserves vertical spacing around it

                    every blank line is displayed, relaying on CSS white-space:pre-wrap on <p>
                */

                // if the paragraph consists only of newlines, then write them out
                // this prevents issues with blocks that are not terminated by a newline (ie. a code block, math block, comment block, or (sometimes) table row blocks)
                if let Some(num_of_blank_lines) = self.get_number_of_blank_lines_in_blank_paragraph(node) {
                    if num_of_blank_lines > 0 {
                        // open a paragraph block, which also writes out the previously open block's closing tag
                        self.open_block(b"<p>", node_id, node.node_type);

                        // write out the blank lines
                        for _ in 0..num_of_blank_lines {
                            self.html_output_array.push(b'\n');
                        }

                        // close the paragraph
                        self.html_output_array.extend_from_slice(b"</p>\n");
                    }

                    return;
                }

                /*
                    here, the paragraph is not only newlines
                */
                
                // open a paragraph block, pushing the opening tag
                self.open_block(b"<p>", node_id, node.node_type);

                // get the node for which should continue processing from - will be the node's first child if there's no newline to skip, or the next sibling of the node (which will have been written out, minus the leading newline) if there's a newline to skip
                let next_node = self.paragraph_skip_newline_that_terminates_predecessor(node);

                // process the next node
                self.process_child_nodes(next_node, b"</p>\n");
            }

            NodeType::H1 => self.process_heading(b"<h1>", b"</h1>\n", node_id, node),
            NodeType::H2 => self.process_heading(b"<h2>", b"</h2>\n", node_id, node),
            NodeType::H3 => self.process_heading(b"<h3>", b"</h3>\n", node_id, node),
            NodeType::H4 => self.process_heading(b"<h4>", b"</h4>\n", node_id, node),
            NodeType::H5 => self.process_heading(b"<h5>", b"</h5>\n", node_id, node),
            NodeType::H6 => self.process_heading(b"<h6>", b"</h6>\n", node_id, node),

            NodeType::ThematicBreak => {
                self.open_block(b"<hr>\n", node_id, node.node_type);
            }

            NodeType::FoldingBreak => {
                /*
                    folding breaks enable one block to follow another where it wouldn't otherwise be able to

                    eg.
                        - list item
                        ///
                            indented, not a continuation of the list
                    
                    folding breaks aren't shown, but they must close the open block to achieve their goal
                    hence, open a block that pushes no tags to the HTML
                */
                self.open_block(b"", node_id, node.node_type);
            }

            NodeType::IndentBlock => {
                // open the indent block
                self.open_block(b"<div class=\"indent\">\n", node_id, node.node_type);

                // process the nodes in the indent
                self.process_child_nodes(node.first_child, b"</div>\n");
            }

            NodeType::Callout => {
                // open a callout block
                // note that the opening tag hasn't yet been closed - defer this until after the callout type (if specified) has been found and handled
                self.open_block(b"<div class=\"callout\"", node_id, node.node_type);

                // find the node for the type of the callout
                if let Some(callout_type_node_id) = self.find_child_of_type(node.first_child, NodeType::CalloutType, Some(2)) {
                    // deref the node
                    let callout_type_node = *self.ast.get_node(callout_type_node_id);

                    // push bytes to declare that the callout has a type
                    self.html_output_array
                        .extend_from_slice(b" data-callout=\"");

                    // node for the callout type captures the square brackets and explanation mark - eg. [!warning] - in the type, but only want the text
                    let (callout_type_start, callout_type_end) =
                        get_range_for_callout_type_chars(self.input_string_bytes, callout_type_node.start, callout_type_node.end);

                    // push bytes for the callout type, so CSS can react if it's a recognised type
                    self.escape_and_push_to_html(callout_type_start, callout_type_end);

                    // close the type
                    self.html_output_array.push(b'"');
                }

                // close the opener
                self.html_output_array.extend_from_slice(b">\n");

                // process the next node
                self.process_child_nodes(node.first_child, b"</div>\n");
            }

            NodeType::CalloutTitle => {
                // open the callout title
                self.html_output_array
                    .extend_from_slice(b"<div class=\"callout-title\">");

                // process the next node
                self.process_child_nodes(node.first_child, b"</div>\n");
            }

            NodeType::CodeBlock => {
                // if there's a language for the code block, get the node for it
                let language = self
                    .find_child_of_type(node.first_child, NodeType::CodeBlockLanguage, Some(2))
                    .map(|language| *self.ast.get_node(language));

                match language {
                    Some(language) => {
                        // if a language has been defined, open a node for the language
                        self.open_block(b"<pre data-language=\"", node_id, node.node_type);

                        // push the bytes for the language title
                        self.escape_and_push_to_html(language.start, language.end);

                        // close the language tag, and open the code tag
                        self.html_output_array
                            .extend_from_slice(b"\"><code class=\"language-");

                        // push the bytes for the language on the code
                        self.escape_and_push_to_html(language.start, language.end);

                        // push the closer to the opener
                        self.html_output_array.extend_from_slice(b"\">");
                    }

                    // no language found, so open a code block without a language
                    None => self.open_block(b"<pre><code>", node_id, node.node_type),
                }

                // process the next node
                self.process_child_nodes(node.first_child, b"</code></pre>\n");
            }

            NodeType::MathBlock => {
                // open the math block
                self.open_block(b"<div class=\"math math-block\">", node_id, node.node_type);

                // process the next node
                self.process_child_nodes(node.first_child, b"</div>\n");
            }

            NodeType::DashListItem | NodeType::PlusListItem | NodeType::AsteriskListItem => {
                // handle if the list item is a check list item
                match self.get_check_list_char(node) {
                    // if it's a check list item, open a check list item with the char in the check list box
                    Some(check_list_char_range) => self.open_check_list_item(check_list_char_range, node_id, node.node_type),
                    
                    // if it's a regular list item, open that
                    None => self.open_block(b"<li>", node_id, node.node_type),
                }

                // process the next node
                self.process_child_nodes(node.first_child, b"</li>\n");
            }

            NodeType::NumListItem => {
                // open the list item
                // note that the opening tag isn't closed yet
                self.open_block(b"<li", node_id, node.node_type);

                // handle the number used to open the list item
                if let Some(opener_node_id) = self.find_child_of_type(node.first_child, NodeType::NumListItemOpener, Some(1))
                {
                    // deref the node
                    let opener_node = *self.ast.get_node(opener_node_id);

                    // get the range of the digits in the opener for the num list item
                    if let Some((start, end)) =
                        get_range_for_num_in_num_list_item(self.input_string_bytes, opener_node.start, opener_node.end)
                    {
                        // push the number to the byte array
                        // done this way, rather than using auto-numbering, so that num list items can be defined using arbitrary numbers, not necessarily in sequence (eg. a num list item numbered 1, then 2, then 95 would be ok)
                        self.html_output_array.extend_from_slice(b" value=\"");
                        self.escape_and_push_to_html(start, end);
                        self.html_output_array.push(b'"');
                    }
                }

                // close the opening tag
                self.html_output_array.push(b'>');

                // process the next node
                self.process_child_nodes(node.first_child, b"</li>\n");
            }

            NodeType::TableRowBlock => {
                // if it's a separator row (defined by containing at least one alignment node, at any column in row)
                // separator rows are not shown as a row, but change the table formatting
                if self.is_separator_row(node.first_child) {
                    // if already in a table header row, now can close it and open the body of the table
                    if self.in_table_header_row {
                        self.html_output_array
                            .extend_from_slice(b"</thead>\n<tbody>\n");
                        self.in_table_header_row = false; // no longer in header row
                    }

                    return;
                }

                /*
                    here, either not a separator row OR is a seaprator row, but no header row was open, so it is ignored
                */

                // open table row
                self.open_block(b"<tr>\n", node_id, node.node_type);

                // init the current column
                self.table_column = 0;

                // process the next node
                self.process_child_nodes(node.first_child, b"</tr>\n");
            }

            NodeType::TableCellContent => {
                // get the alignment for the column (if it's within the range which can and do have alignments specified)
                let alignment = if self.table_column < self.aligned_column_count {
                    self.table_alignments[self.table_column]
                } else {
                    TableColumnAlignment::None
                };

                // if a content cell is encountered, the column should be incremented for the next content cell (if present) in the row
                self.table_column += 1;

                // table header cells: th, table body cells: td
                let (open_tag, close_tag): (&[u8], &'static [u8]) = if self.in_table_header_row {
                    (b"<th", b"</th>\n")
                } else {
                    (b"<td", b"</td>\n")
                };

                // push the opening tag - minus the closing '>' for the table cell
                self.html_output_array.extend_from_slice(open_tag);

                // push the styling for the cell based on the alignment
                self.html_output_array.extend_from_slice(alignment.get_styling());
                
                // close the opener
                self.html_output_array.push(b'>');

                // process the next node
                self.process_child_nodes(node.first_child, close_tag);
            }

            /*
                content nodes require a silent frame to be pushed, as they don't require explicit nodes in the HTML
            */
            NodeType::H1Content
            | NodeType::H2Content
            | NodeType::H3Content
            | NodeType::H4Content
            | NodeType::H5Content
            | NodeType::H6Content
            | NodeType::IndentContent
            | NodeType::CalloutContent
            | NodeType::DashListItemContent
            | NodeType::PlusListItemContent
            | NodeType::AsteriskListItemContent
            | NodeType::NumListItemContent => {
                self.process_child_nodes_silently(node.first_child);
            }

            /*
                INLINES
            */

            NodeType::Italic => self.process_inline(b"<em>", b"</em>", node),
            NodeType::Bold => self.process_inline(b"<strong>", b"</strong>", node),
            NodeType::BoldItalic => self.process_inline(b"<strong><em>", b"</em></strong>", node),
            NodeType::Underline => self.process_inline(b"<u>", b"</u>", node),
            NodeType::Strikethrough => self.process_inline(b"<del>", b"</del>", node),
            NodeType::HighlightC1 => self.process_inline(b"<mark class=\"hl-1\">", b"</mark>", node),
            NodeType::HighlightC2 => self.process_inline(b"<mark class=\"hl-2\">", b"</mark>", node),
            NodeType::InlineCode => self.process_inline(b"<code>", b"</code>", node),
            NodeType::InlineMath => {
                self.process_inline(b"<span class=\"math math-inline\">", b"</span>", node)
            }

            NodeType::Tag => {
                // find the ID of the node for the tag content
                let Some(content_node_id) = self.find_child_of_type(node.first_child, NodeType::TagContent, Some(2)) else {
                    return;
                };

                // deref the node
                let content_node = *self.ast.get_node(content_node_id);

                // open the tag
                self.html_output_array
                    .extend_from_slice(b"<span class=\"tag\">#");

                // push bytes for the tag content
                self.escape_and_push_to_html(content_node.start, content_node.end);

                // close the tag
                self.html_output_array.extend_from_slice(b"</span>");
            }

            NodeType::Link => {
                // links can have their content broken over several nodes due to escape `]` chars, so need to extract and concatenate bytes of all content nodes in Link node
                let Some((first_content_node_start, first_content_node_end, link_content_bytes)) =
                    self.get_joined_content_node_ranges(node.first_child, NodeType::LinkContent)
                else {
                    // no content nodes found, nothing to do
                    return;
                };

                // get bytes captured by first content node, in case there was only one content node, in which case these are all the content bytes
                let input_string_bytes = self.input_string_bytes;
                let bytes_captured_by_first_content_node = &input_string_bytes[first_content_node_start as usize..first_content_node_end as usize];

                // links are are less straightforward to deal with, so hand processing off
                // if the array of concatenated link content bytes is Some, then use that for the link content, else use the bytes extracted from the first (and only) content node
                self.handle_link(link_content_bytes.as_deref().unwrap_or(bytes_captured_by_first_content_node));
            }

            NodeType::EmbeddedLink => {
                // embedded links can have their content broken over several nodes due to escape `]` chars, so need to extract and concatenate bytes of all content nodes in EmbeddedLink node
                let Some((first_content_node_start, first_content_node_end, embedded_link_content_bytes)) =
                    self.get_joined_content_node_ranges(node.first_child, NodeType::EmbeddedLinkContent)
                else {
                    // no content nodes found, nothing to do
                    return;
                };

                // get bytes captured by first content node, in case there was only one content node, in which case these are all the content bytes
                let input_string_bytes = self.input_string_bytes;
                let bytes_captured_by_first_content_node = &input_string_bytes[first_content_node_start as usize..first_content_node_end as usize];

                // embedded links are are less straightforward to deal with, so hand processing off
                // if the array of concatenated embedded link content bytes is Some, then use that for the embedded link content, else use the bytes extracted from the first (and only) content node
                self.handle_embedded_link(embedded_link_content_bytes.as_deref().unwrap_or(bytes_captured_by_first_content_node));
            }
        }
    }

    /// Takes the first child node of a node, and extracts the ranges of content nodes from that list.
    /// If no content entries are found, None is returned.
    /// If more than one content entry is found and joined, two offsets and Some(array of bytes) will be returned - use the array.
    /// If only one content entry is found, that array will be None, and the start/end offsets from the return should be used instead
    fn get_joined_content_node_ranges(
        &self,
        first_child: NodeId,
        content_node_type: NodeType,
    ) -> Option<(u32, u32, Option<Vec<u8>>)> {
        // stores the range offsets for the first content entry, and will be the only ones of interest if there's only one content node in the child list
        let mut first_content_entry_range: Option<(u32, u32)> = None;

        // the array of bytes, joined together from each content node found
        let mut joined_content_bytes: Option<Vec<u8>> = None;

        // traversal node
        let mut child_node = first_child;

        // traverse the list of child nodes for content nodes
        while child_node != NO_NODE {
            // deref the node
            let node = self.ast.get_node(child_node);

            // only interested in content nodes
            if node.node_type == content_node_type {
                // get bytes in range covered by content node
                let bytes_in_range = &self.input_string_bytes[node.start as usize..node.end as usize];

                // where to put the range depends on how many content nodes have been found already
                match (&mut joined_content_bytes, first_content_entry_range) {
                    // third piece onwards

                    // used for the third range and beyond
                    // push the bytes to the array
                    (Some(buffer), _) => buffer.extend_from_slice(bytes_in_range),

                    // used for the first range found
                    // write the range offstes into the var
                    (None, None) => first_content_entry_range = Some((node.start, node.end)),

                    // used for the second range found
                    // use the joined_content_bytes array
                    (None, Some((start, end))) => {
                        // get the bytes of the first content node's range
                        let bytes_of_first_content_entry = &self.input_string_bytes[start as usize..end as usize];

                        // init the array for joined bytes with capacity for the first range's bytes and the bytes in the range by the second content node
                        let mut buffer = Vec::with_capacity(bytes_of_first_content_entry.len() + bytes_in_range.len());

                        // push the bytes from the first and second ranges to the array
                        buffer.extend_from_slice(bytes_of_first_content_entry);
                        buffer.extend_from_slice(bytes_in_range);

                        // write the array
                        joined_content_bytes = Some(buffer);
                    }
                }
            }

            // increment traversal node
            child_node = node.next_sibling;
        }

        // returns None here if no content node was found
        let (start, end) = first_content_entry_range?;

        // returns the offsets for the first found content entry, and Some(array of bytes) if more than one was found, or None for that array if only one content entry found
        Some((start, end, joined_content_bytes))
    }

    #[inline]
    /// Get frame on top of stack (ie. the one currently being processed) without popping it off the stack
    fn peek_top_stack_frame(&mut self) -> &mut TreeFrame {
        // unwrap is fine as process_ast pushes a frame before its first iteration, and breaks after popping the last one
        self.stack.last_mut().unwrap()
    }

    #[inline]
    /// Takes a frame index, and a point to continue traversing from if the stack frame can be stepped out of, or None if it has a HTML tag that must be written out on close 
    pub(super) fn continue_traversing_outside_frame(&self, frame_index: usize) -> Option<(NodeId, usize)> {
        // if at the bottom of the stack, or the frame is not silent (meaning that a closing tag must be written out), then cannot resume traversal outside the frame
        if frame_index == 0 || !self.stack[frame_index].is_silent {
            return None;
        }

        // otherwise, look at the next frame down the stack, and continue traversing from the next sibling of the node it indicates
        Some((self.stack[frame_index - 1].next_sibling, frame_index - 1))
    }

    #[inline]
    /// Processes the children of a node in the AST, storing the HTML tag that must be written out when popping the frame off the stack, in order to close the block the children belong to.
    /// A node with no child nodes does not have a frame created, and its closing tag is written out immediately
    fn process_child_nodes(&mut self, first_child_node_id: NodeId, closing_tag: &'static [u8]) {
        // if there are no child nodes to process, write out the closing tag and move on
        if first_child_node_id == NO_NODE {
            self.html_output_array.extend_from_slice(closing_tag);

            return;
        }

        // push the stack frame, so the child nodes are processed, and the closing tag for the block is written out after that processing has completed
        self.stack.push(TreeFrame {
            next_sibling: first_child_node_id,
            closing_tag,
            block_container: BlockContainers::None,
            is_silent: false,
        });
    }

    #[inline]
    /// Processes the children of a node in the AST, with no HTML tag for writing out when popping the frame off the stack.
    /// Used for containers for blocks (ie. unordered and ordered lists, and tables)
    fn process_child_nodes_silently(&mut self, first_child_node_id: NodeId) {
        // if there are no child nodes to process, move on
        if first_child_node_id == NO_NODE {
            return;
        }

        // take the container from the parent node, and put it on this frame, to ensure correct propagation
        // this is returned to the parent node when the frame is popped from the stack
        let inherited = std::mem::replace(&mut self.peek_top_stack_frame().block_container, BlockContainers::None);

        // push the stack frame, so the child nodes are processed, and the container is closed properly when that processing has completed
        self.stack.push(TreeFrame {
            next_sibling: first_child_node_id,
            closing_tag: b"",
            block_container: inherited,
            is_silent: true,
        });
    }

    #[inline]
    /// Writes out a block's opening tag, after opening/closing container blocks as required for the node type passed in
    pub(super) fn open_block(
        &mut self,
        opening_tag: &'static [u8],
        node_id: NodeId,
        node_type: NodeType,
    ) {
        // open the corresponding container (if required) for the block type passed in
        // eg. DashListItem node requries an unordered list to be opened
        self.set_open_container(
            BlockContainers::get_container_type_for_block(node_type),
            node_id,
        );

        // write out the opening tag for the block, now that the block container (if required) has been dealt with
        self.html_output_array.extend_from_slice(opening_tag);
    }

    #[inline]
    /// Writes out an inline structure's opening tag, and processes its child nodes, before writing out its closing tag
    fn process_inline(&mut self, open_tag: &'static [u8], close_tag: &'static [u8], node: &Node) {
        // write out the opening tag
        self.html_output_array.extend_from_slice(open_tag);

        // process the child nodes, before writing out the closing tag
        self.process_child_nodes(node.first_child, close_tag);
    }

    #[inline]
    /// Writes out a heading's opening tag, and processes its child nodes, before writing out its closing tag
    fn process_heading(
        &mut self,
        opening_tag: &'static [u8],
        closing_tag: &'static [u8],
        node_id: NodeId,
        node: &Node,
    ) {
        // open block for heading
        self.open_block(opening_tag, node_id, node.node_type);

        // process the child nodes, before writing out the closing tag
        self.process_child_nodes(node.first_child, closing_tag);
    }

    /// Sets the open container at the current frame, closing the old one and opening the new one
    fn set_open_container(&mut self, desired_container: BlockContainers, node_id: NodeId) {
        // peek the frame on the top of the stack
        let frame = self.peek_top_stack_frame();

        // if the container to be set to open is already open, don't need to do anything
        if frame.block_container == desired_container {
            return;
        }

        // replace the container on the frame
        let previous = std::mem::replace(&mut frame.block_container, desired_container);

        // close the container that was replaced by the new one
        self.close_container(previous);

        // write the opener/open the table block/do nothing if no container should be open
        match desired_container {
            BlockContainers::UnorderedList => self.html_output_array.extend_from_slice(b"<ul>\n"),
            BlockContainers::OrderedList => self.html_output_array.extend_from_slice(b"<ol>\n"),
            BlockContainers::Table => self.open_table_block(node_id),
            BlockContainers::None => {}
        }
    }

    /// Writes out the closing tag for a container, if one was open
    fn close_container(&mut self, container: BlockContainers) {
        match container {
            BlockContainers::UnorderedList => self.html_output_array.extend_from_slice(b"</ul>\n"),
            BlockContainers::OrderedList => self.html_output_array.extend_from_slice(b"</ol>\n"),

            BlockContainers::Table => {
                if self.in_table_header_row {
                    // close table, in the header row
                    // therefore, write out closing tag for thead, and for the table, and for the wrapping div
                    self.html_output_array
                        .extend_from_slice(b"</thead>\n</table>\n</div>\n");
                } else {
                    // close table, not in a header row
                    // therefore, write out closing tag for tbody, and for the table, and for the wrapping div
                    self.html_output_array
                        .extend_from_slice(b"</tbody>\n</table>\n</div>\n");
                }
            }

            BlockContainers::None => {}
        }
    }

    #[inline]
    /// Appends input bytes captured by range supplied, escaping chars as required for HTML output
    pub(super) fn escape_and_push_to_html(&mut self, start: u32, end: u32) {
        // range validity check
        if start >= end {
            return;
        }

        // write out the bytes
        write_out_escaped_bytes(
            self.html_output_array,
            &self.input_string_bytes[start as usize..end as usize],
        );
    }

    /// Takes a node, and returns the ID of the node where traversal should continue, after determining whether a leading newline belongs to the paragraph, or the preceding block (and skipping over it if it's the latter)
    fn paragraph_skip_newline_that_terminates_predecessor(&mut self, node: &Node) -> NodeId {
        /*
            if the input range captured by the node starts at the first character, then the "line above" (base case) ended on its own
            if the character before the input range captured by the node is a newline, then that block closed on that line with nothing after it on that line
            in either of these cases, return the first child of the node, as nothing needs skipping
        */
        if node.start == 0 || self.input_string_bytes[node.start as usize - 1] == b'\n' {
            return node.first_child;
        }

        // if the first child node is NO_NODE, then the paragraph is empty, thus there's nothing to skip over
        if node.first_child == NO_NODE {
            return NO_NODE;
        }

        // deref the first child of the node
        let first_child_node = *self.ast.get_node(node.first_child);

        // if the first child isn't a literal, or its first byte isn't a newline char, then there's no leading newline to skip
        if first_child_node.node_type != NodeType::Literal
            || self.input_string_bytes[first_child_node.start as usize] != b'\n'
        {
            return node.first_child;
        }

        /*
            if here, then the first char (a newline) doesn't belong to this paragraph, and should be skipped over
        */

        // write out the node's content (which must be a literal, due to the if-test above) minus the first newline char
        self.escape_and_push_to_html(first_child_node.start + 1, first_child_node.end);

        // point to resume from is the next sibling of this node
        first_child_node.next_sibling
    }

    /// Takes a Paragraph node, and returns the number of blank lines it covers if it entirely consists of newlines, or None if it's not only newlines
    pub(super) fn get_number_of_blank_lines_in_blank_paragraph(&self, node: &Node) -> Option<usize> {
        // a document that opens with a blank line has no line above it for the newline to have ended
        let ended_above =
            node.start == 0 || self.input_string_bytes[node.start as usize - 1] == b'\n';

        // if ended_above is true, then newlines inits to 1
        // if false, inits to 0
        let mut newline_count = usize::from(ended_above);

        // init the traversal node
        let mut child_node_id = node.first_child;

        // traverse the list of child nodes of the node passed in
        while child_node_id != NO_NODE {
            // deref the node
            let child_node = self.ast.get_node(child_node_id);

            // increment the traversal node
            child_node_id = child_node.next_sibling;

            // skip over continuations and comments, as the former belongs to enclosing blocks (not current block of concern), and the latter isn't rendered
            if is_continuation_node(child_node.node_type)
                || child_node.node_type == NodeType::InlineComment
            {
                continue;
            }

            // a newline char of interest will be captured as a literal - if a non-literal found, then not a paragraph of only blank lines
            if child_node.node_type != NodeType::Literal {
                return None;
            }

            // iterate over the bytes in the range
            for &byte in
                &self.input_string_bytes[child_node.start as usize..child_node.end as usize]
            {
                // if non-whitespace found, then not a paragraph of only blank lines
                if !byte.is_ascii_whitespace() {
                    return None;
                }

                // if whitespace is a newline, add one to count
                newline_count += usize::from(byte == b'\n');
            }
        }

        // take one off (prevented from going below 0) the count as it counts the number of line terminators
        // hence the number of blank lines is the number of terminators minus one
        Some(newline_count.saturating_sub(1))
    }

    #[inline]
    /// Takes the first child in a list of child nodes, and a type of node to look for.
    /// Takes Some search breadth (as only looks at siblings of the nodes, not children) to only look a limited number of times, or None to look until all children have been checked.
    /// Returns the ID of the node if found, or None otherwise
    pub(super) fn find_child_of_type(&self, first_child: NodeId, desired_type: NodeType, search_breadth: Option<NodeId>) -> Option<NodeId> {
        // take a mutable copy of the ID
        let mut child_node = first_child;

        // counter for how many siblings checked so far
        let mut breadth_searched = 0;

        // while the node list isn't empty and the search breadth (if specified) has not been exceeded
        // note it checks one more than max_depth (provides a moderate human stress reduction...)
        while child_node != NO_NODE && search_breadth.is_none_or(|max_depth| breadth_searched <= max_depth) {
            // deref the node
            let node = self.ast.get_node(child_node);

            // return the node if it's of the desired type
            if node.node_type == desired_type {
                return Some(child_node);
            }

            // if node is not of desired type, increment node ID and breadth counter
            child_node = node.next_sibling;
            breadth_searched += 1;
        }

        // if no node of desired type was found within search breadth, return None
        None
    }
}
