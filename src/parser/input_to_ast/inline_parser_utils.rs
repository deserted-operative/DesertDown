use super::{
    ast_utils::{AST, NO_NODE, NodeId},
    char_buffer_utils::CharBuffer,
    input_string_utils::{InputStringRange, merge_contiguous_ranges},
    node_utils::NodeType,
};

#[derive(Debug, Clone, Copy)]
/// Stores the currently open inline structure.
/// A stack of these is maintained to handle the nesting of inline structures
pub(crate) struct OpenInline {
    // node in the AST for the inline structure
    pub(crate) node: NodeId,

    // the node type is keppt here to avoid needing to reference back in the array for it
    pub(crate) node_type: NodeType,

    // the number of emphasis levels that are open at this inline - between 0 and 3 (both inclusive)
    pub(crate) emphasis_levels: u8,
}

#[inline]
/// Takes a number of emphasis levels (strictly greater than 0), and returns the corresponding [`NodeType`]
pub(crate) const fn get_node_type_from_emph_levels(level_count: u8) -> NodeType {
    match level_count {
        1 => NodeType::Italic,
        2 => NodeType::Bold,
        _ => NodeType::BoldItalic,
    }
}

#[inline]
// Takes a [`NodeType`] and returns whether the node is a self-contained node (ie. chars inside require no further processing) or not.
// ie. No nodes should be in any self-contained nodes
const fn is_self_contained_inline(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::InlineCode
            | NodeType::InlineComment
            | NodeType::InlineMath
            | NodeType::Tag
            | NodeType::Link
            | NodeType::EmbeddedLink
    )
}

#[inline]
/// Takes the stack of open inlines and the number of levels to close.
/// Returns the index of the inline on the stack that is closest to the bottom of the stack required to close the levels.
/// eg. Consider a triple emphasis delimiter, where three levels were opened with a double then a single: **a*b***. In this example, the index of the double delimiter is of interest, as outside that node is where the tree should continue to be built.
/// If any levels of emphasis remain after closing the desired number of levels, a residual is returned
pub(crate) fn get_info_for_non_matching_emphasis_closure(
    stack: &[OpenInline],
    levels_to_close: u8,
) -> (usize, Option<(NodeType, u8)>) {
    let mut remaining_levels_to_close = levels_to_close;
    let mut stack_index = stack.len();
    let mut residual = None; // inits to None

    while remaining_levels_to_close > 0 {
        // walk down the stack, from the top
        stack_index -= 1;

        let open_inline = stack[stack_index];

        // if the inline being considered doesn't have anything to do with emphasis, move on
        if open_inline.emphasis_levels == 0 {
            continue;
        }

        if open_inline.emphasis_levels > remaining_levels_to_close {
            // the inline being considered opened more levels of emphasis than remain to be closed
            // therefore, this inline should be closed, and the excess of how many need re-opening should be returned

            let remainder = open_inline.emphasis_levels - remaining_levels_to_close;

            residual = Some((get_node_type_from_emph_levels(remainder), remainder));

            remaining_levels_to_close = 0;
        } else {
            // the inline being considered either closes exactly as many or fewer levels of emphasis that remain to be closed
            // therefore, simply subtract from the count, which will either trigger another iteration, or the return case
            remaining_levels_to_close -= open_inline.emphasis_levels;
        }
    }

    (stack_index, residual)
}

/// Structures and variables for the inline parser to work out of/in to.
pub(crate) struct InlineParserWorkspace<'l> {
    // 'l is a lifeteime parameter, providing a compile time guarantee that a BlockParserWorkspace cannot outlive the borrow it holds

    // mut borrow for the ast
    pub(crate) ast: &'l mut AST,

    // every inline which is currently open
    // base of stack stored first, top of stack stored last
    pub(crate) stack: &'l mut Vec<OpenInline>,

    // array of inlines that have been cut short by closing an overlapping inline
    // ie. they need reopening after the overlapping inline has been closed
    pub(crate) requires_reopening: &'l mut Vec<(NodeType, u8)>,

    // tracks the offset in the input string where the most recently written out char(s) ends
    // ie. tracks progression through the input
    pub(crate) last_end: u32,

    // two cursors - pending_node is the last node that chars were written out to, pending_parent ensures that new nodes are opened if an inline closed (required as the output label of the chars to write out may be the same as pending_node, even though the inline that's open has changed)
    // pending_node is NO_NODE when there is no open node to continue writing to
    pub(crate) pending_node: NodeId,
    pub(crate) pending_parent: NodeId,

    /*
        cached versions of the node type and node end for the current node
        this prevents unnecessary, repeated reads/writes over the values in the nodes in the array, offering performance improvements

        these values are valid while the nodes are open, whilst the versions in the node objects are not
        they are written out to the nodes in the array when they are going to be closed
    */
    pub(crate) pending_label: NodeType,
    pub(crate) pending_end: u32,
}

impl InlineParserWorkspace<'_> {
    #[inline]
    /// Takes an [`InlineParserWorkspace`] and returns the [`NodeId`] of the most recently opened - and still open - inline structure.
    /// Required as chars are written out to the most recently opened inline structure
    pub(crate) fn get_newest_opened_inline_node(&self) -> NodeId {
        self.stack[self.stack.len() - 1].node
    }

    #[inline]
    /// Takes an [`InlineParserWorkspace`] and returns the [`NodeType`] of the most recently opened - and still open - inline structure
    fn get_newest_opened_inline_node_type(&self) -> NodeType {
        self.stack[self.stack.len() - 1].node_type
    }

    #[inline(always)]
    /// Writes cached version of the pending node's end back into the array of nodes.
    /// Safe to call twice, or when no node is pending write out to the AST
    fn commit_end_of_open_node(&mut self) {
        // NO_NODE if there is no open node to continue - if that's the case, nothing to do
        if self.pending_node != NO_NODE {
            self.ast.nodes[self.pending_node as usize].end = self.pending_end;
        }
    }

    #[inline(always)]
    /// Write out a contiguous sequence of input bytes to the AST, under the supplied node type
    pub(crate) fn write_out_contiguous_chars(
        &mut self,
        parent: NodeId,
        output_label: NodeType,
        range: InputStringRange,
    ) {
        /*
            edge case - if the range is empty (ie. end overlaps with start), then nothing should be written out to the AST
            this allows flush characters (a null character for the inline parser) to be run through the machine, but not be presented in the AST (as they're not part of the original input string)
        */
        if range.start == range.end {
            return;
        }

        // if a node is open, and the parent node hasn't changed, and the output label is the same as the one of the node last written to, and the chars are contiguous
        // then, the char captured by the range is just pushed to the existing pending node
        if self.pending_node != NO_NODE
            && self.pending_parent == parent
            && self.pending_label == output_label
            && self.pending_end == range.start
        {
            self.pending_end = range.end;
            self.last_end = range.end;
            return;
        }

        /*
            need to open a new node
        */

        // before opening a new node, write out the cached node end to the AST, so that it's version of the node is accurate
        self.commit_end_of_open_node();

        // create a new detached node for the char
        let node = self
            .ast
            .new_detatched_node(output_label, range.start, range.end);

        // link it up to it's parent in the AST
        self.ast.append_child_to_parent(parent, node);

        // update worksapce
        self.pending_node = node;
        self.pending_parent = parent;
        self.pending_label = output_label;
        self.pending_end = range.end;
        self.last_end = range.end;
    }

    #[inline(never)]
    /// Write out a non-contiguous sequence of input bytes to the AST, under the supplied node type.
    /// Used when chars that are interrupted by an exclusion zone are to be written out
    fn write_out_non_contiguous_chars(
        &mut self,
        parent: NodeId,
        output_label: NodeType,
        ranges: &[InputStringRange],
    ) {
        for range in merge_contiguous_ranges(ranges) {
            self.write_out_contiguous_chars(parent, output_label, range);
        }
    }

    #[inline]
    /// Takes a specific inline node to write out a number of the characters from the front of the buffer to, with a given output label
    pub(crate) fn write_out_chars_from_buffer(
        &mut self,
        parent: NodeId,
        output_label: NodeType,
        buffer: &CharBuffer,
        num_of_chars: usize,
    ) {
        // get the chars in the buffer that are to be written out
        // returns a slice where each char is a range
        let chars_to_write_out = buffer.get_buffer_as_slice();

        // handle exclusion zones in the ranges if they are present
        if buffer.contiguous {
            // contiguous case is the simple case

            // the ordinary case - the characters are one piece of the source

            // package as one range
            let range_to_write_out = InputStringRange {
                start: chars_to_write_out[0].start,
                end: chars_to_write_out[num_of_chars - 1].end,
            };

            // write them to the AST
            self.write_out_contiguous_chars(parent, output_label, range_to_write_out);

            return;
        }

        /*
            non-contiguous case
        */

        // use the non-contiguous write out logic, passing in as many chars (each as its own range in the slice) as are to be written out from the buffer
        self.write_out_non_contiguous_chars(
            parent,
            output_label,
            &chars_to_write_out[..num_of_chars],
        );
    }

    #[inline]
    /// Writes out a number of the characters from the front of the buffer to, with a given output label, to whichever inline node was most recently opened.
    pub(crate) fn write_out_chars_from_buffer_to_newest_inline(
        &mut self,
        output_label: NodeType,
        buffer: &CharBuffer,
        num_of_chars: usize,
    ) {
        let parent = self.get_newest_opened_inline_node();
        self.write_out_chars_from_buffer(parent, output_label, buffer, num_of_chars);
    }

    #[inline(always)]
    /// Writes out a number of the characters from an input string range, with a given output label, to whichever inline node was most recently opened.
    /// Chars to be written out from the range must be contiguous
    pub(crate) fn write_out_chars_in_range_to_newest_inline(
        &mut self,
        output_label: NodeType,
        range: InputStringRange,
    ) {
        let parent = self.get_newest_opened_inline_node();
        self.write_out_contiguous_chars(parent, output_label, range);
    }

    #[inline]
    /// Opens a new inline inside whichever inline was most recently opened.
    /// Takes a node type for the inline, and a number of emphasis levels between 0 and 3 (both inclusive)
    pub(crate) fn open_inline(&mut self, node_type: NodeType, emph_levels: u8, start: u32) {
        // the pending node is about to be cleared, as a new node will need to be opened in the newly opened inline, in order to process more input
        // therefore, write out pending node end to AST
        self.commit_end_of_open_node();

        // set that nothing is pending
        self.pending_node = NO_NODE;

        // create new detached node, setting the end temporarily as `start` - will be updated when inline closes
        let node = self.ast.new_detatched_node(node_type, start, start);

        let parent = self.get_newest_opened_inline_node();

        // link up node
        self.ast.append_child_to_parent(parent, node);

        // add inline to stack of open inlines
        self.stack.push(OpenInline {
            node,
            node_type,
            emphasis_levels: emph_levels,
        });
    }

    #[inline]
    /// Returns whether the chars about to be written out should open a new self-contained inline.
    /// True when the inline on the top of the stack is of a different node type, or if the inline on the top of the stack has just been closed
    pub(crate) fn is_new_self_contained_inline_required(
        &self,
        inline_type: NodeType,
        output_label: NodeType,
        start: u32,
    ) -> bool {
        // different inline on top of stack - new self-contained inline requried
        if self.get_newest_opened_inline_node_type() != inline_type {
            return true;
        }

        // same inline on top of stack
        // if an opener, procceed to open a new self-contained inline if there's no pending node, and the output label is the same, and the chars are contiguous
        output_label.is_inline_opener()
            && !(self.pending_node != NO_NODE
                && self.pending_label == output_label
                && self.pending_end == start)
    }

    #[inline]
    /// Closes every self contained inline open on the stack (see [`is_self_contained_inline`]).
    /// Used when input assigned a different (thus, non-compatible) output label arrives
    pub(crate) fn close_self_contained_inlines(&mut self) {
        while let Some(open_inline) = self.stack.last().copied() {
            // if this inline on the stack is not self contained, nothing to do
            if !is_self_contained_inline(open_inline.node_type) {
                break;
            }

            // set the end point on the node in the AST
            self.ast.get_mut_node(open_inline.node).end = self.last_end;

            // inline now closed, remove it from stack
            self.stack.pop();
        }
    }

    #[inline]
    /// Opens an inline based on encountering a delimiter (eg. for italic, or bold) in the input string.
    /// Writes out chars from buffer, as instructed by arguments
    pub(crate) fn open_inline_from_delimiter(
        &mut self,
        node_type: NodeType,
        emph_levels: u8,
        output_label: NodeType,
        buffer: &CharBuffer,
        num_of_chars: usize,
    ) {
        // the delimiter is part of the inline, so must capture it
        self.open_inline(
            node_type,
            emph_levels,
            buffer.get_buffer_as_slice()[0].start,
        );

        // get the node just opened above
        let node = self.get_newest_opened_inline_node();

        // write out the number of chars from the buffer, as requested by the argument
        self.write_out_chars_from_buffer(node, output_label, buffer, num_of_chars);
    }

    #[inline]
    /// Closes an inline based on encountering a delimiter (eg. for italic, or bold) in the input string.
    /// Writes out chars from buffer, as instructed by arguments.
    /// Re-opens inlines after the one closed where necessary
    pub(crate) fn close_inline_with_delimiter(
        &mut self,
        stack_index_of_delimited_inline: usize,
        remaining_emphasis: Option<(NodeType, u8)>,
        is_all_emphasis_closed: bool,
        output_label: NodeType,
        buffer: &CharBuffer,
        num_of_chars: usize,
    ) {
        // nothing should logically require re-opening at this point, so anything in the array is stale information
        // thus, clear for re-use
        self.requires_reopening.clear();

        // close each inline opened AFTER the one the delimiter closes, updating their ends to be just before the delimiter (wrt. the input string)
        for open_inline in &self.stack[stack_index_of_delimited_inline + 1..] {
            // write out the end
            self.ast.nodes[open_inline.node as usize].end = self.last_end;

            // determine if the inline requries re-opening after the inline the delimiter closes
            // ie. the only time it doesn't is if the inline currently being handled is for emphasis, and the flag for closing all emphasis is set to true
            if !(is_all_emphasis_closed && open_inline.emphasis_levels > 0) {
                self.requires_reopening
                    .push((open_inline.node_type, open_inline.emphasis_levels));
            }
        }

        // remove the inlines from the stack that have been closed or marked for re-opening
        self.stack.truncate(stack_index_of_delimited_inline + 1);

        // get the id of the node that the delimiter belongs to
        // (this will be the node that the delimiter closes)
        let node_id = self.stack[stack_index_of_delimited_inline].node;

        // write out the number of chars from the buffer, as requested by the argument
        self.write_out_chars_from_buffer(node_id, output_label, buffer, num_of_chars);

        // write out the end of the inline closed by the delimiter
        self.ast.get_mut_node(node_id).end = self.last_end;

        // remove the now closed inline from the stack
        self.stack.pop();

        /*
            now need to re-open all inlines that require re-opening

            they will begin from AFTER the delimiter, in the same order they were originally opened in
        */

        // set up the start index as immediately after the delimiter
        let start = self.last_end;

        // if any emphasis remains after the delimiter, that needs opening first
        if let Some((node_type, emph_levels)) = remaining_emphasis {
            self.open_inline(node_type, emph_levels, start);
        }

        // now re-open each inline that requires re-opening
        // indexed rather than iterated, because reopen and stack are both borrowed by the call inside
        for pos in 0..self.requires_reopening.len() {
            let (node_type, emph_levels) = self.requires_reopening[pos];
            self.open_inline(node_type, emph_levels, start);
        }
    }

    // an inline is only closed by its own delimiter, and once the input has run
    // out there is nothing left to provide one - so whatever is still open when
    // the run ends is closed here instead
    /// Closes all open inlines on the stack, except for the root node (ie. the bottom node on the stack)
    pub(crate) fn close_all_open_inlines(&mut self) {
        // the run is over, so the node being held open has to reach the arena
        //
        // this is the last thing the inline parser does, so nothing after this
        // point would write it back

        // write out the end of the pending node, to ensure it is reflected in the AST
        self.commit_end_of_open_node();

        // close all nodes, except for the root node - ie. the bottom one on the stack
        // root node doesn't belong to the inline parser, so shouldn't be closed by it
        while self.stack.len() > 1 {
            if let Some(open_inline) = self.stack.pop() {
                self.ast.nodes[open_inline.node as usize].end = self.last_end;
            }
        }
    }
}
