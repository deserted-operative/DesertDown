#[path = "input_to_ast/ast_utils.rs"]
mod ast_utils;
#[path = "input_to_ast/block_parser.rs"]
mod block_parser;
#[path = "input_to_ast/block_parser_utils.rs"]
mod block_parser_utils;
#[path = "input_to_ast/char_buffer_utils.rs"]
mod char_buffer_utils;
#[path = "input_to_ast/inline_parser.rs"]
mod inline_parser;
#[path = "input_to_ast/inline_parser_utils.rs"]
mod inline_parser_utils;
#[path = "input_to_ast/input_string_utils.rs"]
mod input_string_utils;
#[path = "input_to_ast/placement_utils.rs"]
mod placement_utils;

#[path = "input_to_ast/automata_utils/block_transitions.rs"]
mod block_transitions;
#[path = "input_to_ast/automata_utils/inline_transitions.rs"]
mod inline_transitions;
#[path = "input_to_ast/automata_utils/transition_utils.rs"]
mod transition_utils;

#[path = "ast_to_html/ast_to_html.rs"]
mod ast_to_html;
#[path = "ast_to_html/conditional_styling.rs"]
mod conditional_styling;
#[path = "ast_to_html/html_document_utils.rs"]
mod html_document_utils;
#[path = "ast_to_html/html_escape_utils.rs"]
mod html_escape_utils;
#[path = "ast_to_html/links.rs"]
mod links;
#[path = "ast_to_html/styling.rs"]
mod styling;
#[path = "ast_to_html/tables.rs"]
mod tables;

#[path = "input_to_ast/automata_utils/action_utils.rs"]
pub mod action_utils;
#[path = "input_to_ast/automata_utils/block_actions.rs"]
pub mod block_actions;
#[path = "input_to_ast/automata_utils/inline_actions.rs"]
pub mod inline_actions;
#[path = "input_to_ast/automata_utils/node_utils.rs"]
pub mod node_utils;
#[path = "input_to_ast/parser_utils.rs"]
pub mod parser_utils;

use node_utils::NodeType;
use placement_utils::{ContentNodeBehaviour, place_parser_output_in_block};
use std::collections::VecDeque;

// re-exported to facilitate parser::X
pub use ast_utils::{AST, MAX_SOURCE_LEN, NO_NODE, Node, NodeId};
pub use block_parser::{BLOCK_INITIAL_STATE, block_parser};
pub use block_transitions::BLOCK_TRANSITIONS;
pub use char_buffer_utils::ParserStructures;
pub use inline_parser::{INLINE_INITIAL_STATE, MAX_EMPHASIS_LEVEL, inline_parser};
pub use inline_transitions::INLINE_TRANSITIONS;
pub use input_string_utils::{InputChar, InputCharsForParsing, InputStringRange, InputStringView};
pub use links::LinkPermissions;
pub use parser_utils::Job;
pub use styling::{OutputWidth, Theme};
pub use transition_utils::WILDCARD_COLUMN_INDEX;

// constants
pub const BLOCK_STATE_COUNT: usize = 727;
pub const INLINE_STATE_COUNT: usize = 804;
pub const SYMBOL_COUNT: usize = 129;

pub const ACTION_LOOKUP_WIDTH: usize = 2;

pub type StateId = u16; // type choice influenced by number of states in BLOCK_STATE_COUNT and INLINE_STATE_COUNT

// declare the two types of parser with readable names
#[derive(Eq, PartialEq)]
enum ParserType {
    BlockParser,
    InlineParser,
}

#[inline]
/// Takes a heading node type, and returns the content node for that heading level.
/// Used to implement creating heading jobs with less verbosity
const fn heading_content_kind(node_type: NodeType) -> NodeType {
    match node_type {
        NodeType::H1 => NodeType::H1Content,
        NodeType::H2 => NodeType::H2Content,
        NodeType::H3 => NodeType::H3Content,
        NodeType::H4 => NodeType::H4Content,
        NodeType::H5 => NodeType::H5Content,
        _ => NodeType::H6Content,
    }
}

// entry point
pub fn parse_input(input_string: &str) -> AST {
    // init object
    let mut parser = Parser::new(input_string);

    // run the block parser over the full input string to extract the level-zero blocks
    parser.extract_level_zero_blocks();

    // process the job queue until it's empty
    parser.process_job_queue();

    // take ownership of the AST out of the parser object, and return it
    parser.ast
}

/// Struct for setting up the parser and its structures for processing an input string
struct Parser<'input_string> {
    input_string: &'input_string str,

    ast: AST,

    // extracted blocks require parsing either by the block/inline parser, and are queued up for processing
    job_queue: VecDeque<Job>,

    // init parser structures here so can be reused between block/inline parser instancesc
    parser_structures: ParserStructures,

    // stores ranges supplied to each block/inline parser instance
    // reused by each job
    input_string_ranges: Vec<InputStringRange>,

    // a node to place the nodes extracted by the block/inline parser onto, before reconciling that content with the nodes of the block it belongs to
    staging_node: NodeId,
}

impl<'input_string> Parser<'input_string> {
    // constructor
    fn new(input_string: &'input_string str) -> Self {
        // pre-alloc some nodes for the AST
        let mut ast = AST::init_with_capacity(input_string.len());

        let input_string_len = input_string.len();
        assert!(
            input_string_len < MAX_SOURCE_LEN,
            "Input string length longer than max number representable by u32."
        );

        // create root node as Document
        let root = ast.new_detatched_node(NodeType::Document, 0, input_string_len as u32);

        // set the root
        ast.root = root;

        // add the staging node into the AST
        // will be placed at index 1, is never removed, and it's start/end are never used
        let staging_node = ast.new_detatched_node(NodeType::Document, 0, 0);

        // create and return Parse object, giving ownership to caller
        Parser {
            input_string,
            ast,
            job_queue: VecDeque::new(),
            parser_structures: ParserStructures::new(),
            input_string_ranges: Vec::new(),
            staging_node,
        }
    }

    /// Takes the full input string, runs the block parser, and extracts all level zero blocks from the input.
    /// Adds jobs for each block to the job queue
    fn extract_level_zero_blocks(&mut self) {
        // for the level-zero block extraction, the ranges should capture the full inpput string
        let ranges = [InputStringRange {
            start: 0,
            end: self.input_string.len() as u32,
        }];

        // take a local copy of the root for the function call, to avoid reading value that is mutably borrowed
        let root = self.ast.root;

        // after block parser returns, job queue holds every level zero block found, in order found
        block_parser(
            InputStringView::new(self.input_string, &ranges),
            &mut self.ast,
            root,
            &mut self.job_queue,
            &mut self.parser_structures,
        );
    }

    /// Iterates over the job queue, parsing blocks, until the queue is empty
    fn process_job_queue(&mut self) {
        // queue is FIFO, thus traversal is breadth-first
        while let Some(job) = self.job_queue.pop_front() {
            // how to process the job depends on the node type of the block to be processed
            match job.node_type {
                // for a paragraph, use the inline parser, and place the parsed content directly below the block-level paragraph node
                NodeType::Paragraph => {
                    self.parse_content_in_block_and_replace_content_nodes(
                        job.block,
                        NodeType::Paragraph,
                        ParserType::InlineParser,
                    );
                }

                // for a heading, only need to run heading content through inline parser
                // content node for the heading (eg. H1Content) should be retained in the AST
                node_type @ (NodeType::H1
                | NodeType::H2
                | NodeType::H3
                | NodeType::H4
                | NodeType::H5
                | NodeType::H6) => {
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        heading_content_kind(node_type),
                        ParserType::InlineParser,
                    ); // job: parse the job's block, node to put it under: whatever level of heading - eg. H1Content, parser to use: inline parser 
                }

                // for an indent block, run indent content through the block parser
                // content node for the indent (ie. IndentContent) should be retained in the AST
                NodeType::IndentBlock => {
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        NodeType::IndentContent,
                        ParserType::BlockParser,
                    );
                }

                // for a callout, run the title through the inline parser, and the content through the block parser
                // retain both content nodes in the AST
                NodeType::Callout => {
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        NodeType::CalloutTitle,
                        ParserType::InlineParser,
                    );
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        NodeType::CalloutContent,
                        ParserType::BlockParser,
                    );
                }

                // for a dash list item, run the content through the block parser
                // retain the content nodes in the AST
                NodeType::DashListItem => {
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        NodeType::DashListItemContent,
                        ParserType::BlockParser,
                    );
                }

                // for a plus list item, run the content through the block parser
                // retain the content nodes in the AST
                NodeType::PlusListItem => {
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        NodeType::PlusListItemContent,
                        ParserType::BlockParser,
                    );
                }

                // for a asterisk list item, run the content through the block parser
                // retain the content nodes in the AST
                NodeType::AsteriskListItem => {
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        NodeType::AsteriskListItemContent,
                        ParserType::BlockParser,
                    );
                }

                // for a num list item, run the content through the block parser
                // retain the content nodes in the AST
                NodeType::NumListItem => {
                    self.parse_content_in_block_and_retain_content_nodes(
                        job.block,
                        NodeType::NumListItemContent,
                        ParserType::BlockParser,
                    );
                }

                // table row blocks may have several table cell content entries, each of which should be run through the inline parser
                // parser output for each table cell should be placed directly under the content node
                NodeType::TableRowBlock => {
                    self.parse_each_content_node_in_block(
                        job.block,
                        NodeType::TableCellContent,
                        ParserType::InlineParser,
                    );
                }

                // only the blocks that require further parsing have explpicit cases
                // therefore, the default case matches every block that doesn't need any further parsing, and is done as-is
                _ => {
                    // nothing required to be done
                }
            }
        }
    }

    /// Takes a block, and the node type of content nodes for that block (eg. for a table row block, TableCellContent), and runs the parser supplied over each node of that type.
    /// Principally used for table row blocks, which may have several separate content nodes, each of which requiring parsing through the inline parser
    fn parse_each_content_node_in_block(
        &mut self,
        block: NodeId,
        content_node_type: NodeType,
        parser_type: ParserType,
    ) {
        // get the first child node of the block
        let mut child_node_in_block = self.ast.get_node(block).first_child;

        // traverse the block's child list
        while child_node_in_block != NO_NODE {
            // dereference the node
            let node = *self.ast.get_node(child_node_in_block);

            // only need to process the node if it's of the content type (eg. TableCellContent, rather than TableWall), and only if it's not empty
            if node.node_type == content_node_type && node.start != node.end {
                // set up the ranges for the parser based on the range captured by the content node
                let ranges = [InputStringRange {
                    start: node.start,
                    end: node.end,
                }];

                // run the appropriate parser, placing the content from the parser directly onto the child list of the content node
                if parser_type == ParserType::BlockParser {
                    block_parser(
                        InputStringView::new(self.input_string, &ranges),
                        &mut self.ast,
                        child_node_in_block,
                        &mut self.job_queue,
                        &mut self.parser_structures,
                    );
                } else {
                    inline_parser(
                        InputStringView::new(self.input_string, &ranges),
                        &mut self.ast,
                        child_node_in_block,
                        &mut self.parser_structures,
                    );
                }
            }

            // advance to next node (will hit NO_NODE when there is no next_sibling, breaking the loop)
            child_node_in_block = node.next_sibling;
        }
    }

    /// Takes a block, and the node type of content nodes for that block type (eg. H1Content for a H1 block), and runs the parser supplied over each node of that type.
    /// Content nodes are retained in the AST where they are placed correctly (they may be removed or replaced in order to correctly construct the tree with the newly extracted content).
    /// ie. Principally, content nodes will remain in the block in the AST, with the nodes extracted by the parser placed under it
    fn parse_content_in_block_and_retain_content_nodes(
        &mut self,
        block: NodeId,
        content_node_type: NodeType,
        parser_type: ParserType,
    ) {
        self.run_parser_on_block(
            block,
            content_node_type,
            parser_type,
            ContentNodeBehaviour::Retained,
        );
    }

    /// Takes a block, and the node type of content nodes for that block type (eg. H1Content for a H1 block), and runs the parser supplied over each node of that type.
    /// Content nodes are not placed in the AST.
    /// ie. Principally, content nodes will be removed from the block in the AST, with the nodes extracted by the parser placed directly under the block
    fn parse_content_in_block_and_replace_content_nodes(
        &mut self,
        block: NodeId,
        content_node_type: NodeType,
        parser_type: ParserType,
    ) {
        self.run_parser_on_block(
            block,
            content_node_type,
            parser_type,
            ContentNodeBehaviour::Replaced,
        );
    }

    /// Runs parser logic over content nodes of a block, with arguments for block, content node type, which parser to use, and how the content nodes should be handled
    fn run_parser_on_block(
        &mut self,
        block: NodeId,
        content_node_type: NodeType,
        parser_type: ParserType,
        content_node_behaviour: ContentNodeBehaviour,
    ) {
        // for each content node (eg. DashListItemContent) in the block, build ranges so the input can be parsed
        self.collect_ranges_for_content_nodes(block, content_node_type);

        // a block with no content chars means nothing is required to be done
        if self.input_string_ranges.is_empty() {
            return;
        }

        // debug check that the staging node is empty before the parser tries to place anything on it
        // (it should be empty, as after a parser instance runs, anything built on the staging node should be taken and placed into the correct location in the AST)
        debug_assert_eq!(self.ast.get_node(self.staging_node).first_child, NO_NODE);

        // run the appropriate parser, placing the content from the parser onto the staging node
        if parser_type == ParserType::BlockParser {
            block_parser(
                InputStringView::new(self.input_string, &self.input_string_ranges),
                &mut self.ast,
                self.staging_node,
                &mut self.job_queue,
                &mut self.parser_structures,
            );
        } else {
            inline_parser(
                InputStringView::new(self.input_string, &self.input_string_ranges),
                &mut self.ast,
                self.staging_node,
                &mut self.parser_structures,
            );
        }

        // take the content extracted by the parser from the staging node
        let parser_output = self.ast.take_child_nodes(self.staging_node);

        // put the parsed output into the correct place in the AST
        place_parser_output_in_block(
            &mut self.ast,
            block,
            content_node_type,
            parser_output,
            content_node_behaviour,
        );
    }

    /// Takes a block, and sets the `input_string_ranges` of the Parser object to have one range per each content node (eg. DashListItemContent in a DashListItem) in the block, so the content of the block can be run through the appropriate parser
    fn collect_ranges_for_content_nodes(&mut self, block: NodeId, content_node_type: NodeType) {
        // empty any now stale ranges
        self.input_string_ranges.clear();

        // get the first child node of the block
        let mut child_node_in_block = self.ast.get_node(block).first_child;

        // traverse the block's child list
        while child_node_in_block != NO_NODE {
            // dereference the node
            let node = *self.ast.get_node(child_node_in_block);

            // if the node is of the content type for this block, construct a range, and push it to the array
            if node.node_type == content_node_type {
                self.input_string_ranges.push(InputStringRange {
                    start: node.start,
                    end: node.end,
                });
            }

            // advance to next node (will hit NO_NODE when there is no next_sibling, breaking the loop)
            child_node_in_block = node.next_sibling;
        }
    }
}

/*
            /"""""\
            | "   |
            |     | /"""\
      /"""\ |   " | |"  |
      |"  | |     | |  "|
      |  "| |     |_| " /
      \ " |_|"    |____/
       \____|     |
            |    "|
        ====|=====|====
              H.N
*/
