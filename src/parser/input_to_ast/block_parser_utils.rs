use super::{
    ast_utils::{AST, NodeId},
    input_string_utils::{InputStringRange, merge_contiguous_ranges},
    node_utils::NodeType,
    parser_utils::Job,
};
use std::collections::VecDeque;

/// Structures and variables for the block parser to work out of/in to
pub(crate) struct BlockParserWorkspace<'l> {
    // 'l is a lifeteime parameter, providing a compile time guarantee that a BlockParserWorkspace cannot outlive the borrow it holds

    // mut borrows for the ast and job deque
    // because both use the same lifetime, the compiler unifies them to the shorter of the two at the call site, which is fine here, as both live for the entire duration of block_parser
    ast: &'l mut AST,
    job_queue: &'l mut VecDeque<Job>,

    // full input string, as bytes
    input_string: &'l [u8],

    /*
        root node for parse run
        for the initial, full-input block parse, this is ast.root
        for parsing an already identified block, this is the staging node, or a table cell
    */
    root: NodeId,

    /*
        type of the root node of the block
        used when a table row, thematic break, or folding break block closes itself, as opposed to relying on closing the block when opening a new one
        in these situations, the node type is used to return the current node to a safe state before opening a new block
    */
    root_type: NodeType,

    // two cursors - current_block is the block being filled, current_node is the working node in that block
    current_block: NodeId,
    current_node: NodeId,

    /*
        cached versions of the node type and node end for the current block node and working node
        this prevents unnecessary, repeated reads/writes over the values in the nodes in the array, offering performance improvements

        these values are valid while the nodes are open, whilst the versions in the node objects are not
        they are written out to the nodes in the array when they are going to be closed
    */
    current_block_type: NodeType,
    current_node_type: NodeType,
    current_block_end: u32,
    current_node_end: u32,

    // stores whether current char is at the start of a line (ie. follows a newline char)
    // used to prevent escape sequences at the start of a line (used to use block opening/continuation syntax without opening/continuing a block) getting folded in to an already open block
    at_line_start: bool,
}

impl<'a> BlockParserWorkspace<'a> {
    // constructor
    // note that no blocks are open on init, so the root stands in for both the current block and current node until the first block is opened
    pub(crate) fn new(
        ast: &'a mut AST,
        job_queue: &'a mut VecDeque<Job>,
        input_string: &'a [u8],
        root: NodeId,
    ) -> Self {
        // read the type of the node this subtree should be rooted at
        let root_type = ast.get_node(root).node_type;

        BlockParserWorkspace {
            ast,
            job_queue,
            input_string,
            root,
            root_type,
            current_block: root,
            current_node: root,
            current_block_type: root_type,
            current_node_type: root_type,
            current_block_end: 0,
            current_node_end: 0,
            at_line_start: true, // initially, at start of line before consuming any input
        }
    }

    #[inline(always)]
    /// Writes cached versions of the node ends for the current block node and working node back into the array of nodes
    fn commit_ends_of_open_nodes(&mut self) {
        // if the current block type is equal to the root type, then no block is open (via initialisation condition)
        // therefore, nothing to do
        if self.current_block == self.root {
            return;
        }

        // write the values into the nodes in the array
        self.ast.nodes[self.current_node as usize].end = self.current_node_end;
        self.ast.nodes[self.current_block as usize].end = self.current_block_end;
    }

    #[inline(always)]
    /// Closes the open block by writing out pending information to the node object, and creates a parse job for the now closed block
    pub(crate) fn close_open_block(&mut self) {
        // if the current block type is equal to the root type, then no block is open (via initialisation condition)
        // therefore, nothing to do
        if self.current_block == self.root {
            return;
        }

        // the end indices of the current block node and working node need writing out from the cached versions, into the nodes in the array
        // this makes the node objects valid before anything else is done with them
        self.commit_ends_of_open_nodes();

        // a block has just been closed - therefore a job needs creating for it in the job queue
        self.job_queue.push_back(Job {
            block: self.current_block,
            node_type: self.current_block_type,
        });
    }

    #[inline(always)]
    /// Write out a contiguous sequence of input bytes to the AST, under the supplied node type
    pub(crate) fn write_out_contiguous_chars(
        &mut self,
        output_label: NodeType,
        block_type_of_char: NodeType,
        range: InputStringRange,
    ) {
        /*
            edge case - if the range is empty (ie. end overlaps with start), then nothing should be written out to the AST
            this allows flush characters (a newline for the block parser) to be run through the machine, but not be presented in the AST (as they're not part of the original input string)
        */
        if range.start >= range.end {
            return;
        }

        /*
            updates at_line_start

            1) pulls the old value - which determines if the last write out left off at a newline
                note that it is this behaviour that enables this to handle blocks within blocks, and the outer block's exclusion zones
                    eg. consider:
                        > - a list in the callout
                        > \- not a list, but still in the callout
                    the write out for the dash list item in the first line leaves at_line_start as true, thus meaning that the write out on the next line will not group the escape sequence in with the open block (the dash list item), instead opening a paragraph

            2) then updates the value for after this write out

            as already illustrated, this ensures that escape sequences at the start of a line aren't folded into an open block (that isn't a paragraph block)
            they shouldn't be captured by the block because escape sequences at the start of the line only occur in the block parser when characters that would open/continue a block are being used without the desire to open/continue a non-paragraph block
        */
        let at_line_start = self.at_line_start;
        self.at_line_start = self.input_string[range.end as usize - 1] == b'\n'; // the -1 is fine because end > start, as validated by the if test above

        /*
            edge case - character(s) written out with no block of its/their own
            eg. escape backslash, the escape sequence - neither of these are alone attributable to a particular block

            how these chars are handled depends on the relative location to the block being parsed (which is the entire input on the initial block parser run)
            if chars are at the start of the line (relative to the block being parsed), then this indicates that a new block paragraph block should be opeend - as an escape sequence in such a location is being used to prevent opening/continuing a non-paragraph block
            if these chars are not at the start of the line, then they belong to the open block - they're not relevant to delimiting the block structure
        */
        let block_type_of_char = if block_type_of_char == NodeType::NoOp {
            if self.current_block != self.root && !at_line_start {
                // a block is open, the chars being written out are not at the start of the line (relative to the block being parsed), thus belong to open block
                self.write_out_to_open_block(output_label, range);

                return;
            }

            // otherwise, open a new paragraph block
            NodeType::Paragraph
        } else {
            // not an edge case, return the block type
            block_type_of_char
        };

        // does the output of this action belong to the same block or a different one?
        if block_type_of_char != self.current_block_type {
            // different block, so open a new block
            self.open_block(output_label, block_type_of_char, range);

            // a one line block is finished by the newline written out inside it,
            // rather than by the next block opening - see end_completed_block

            // some blocks span exactly one line - thematic and folding breaks, table row blocks
            // check if this situation is one of those, and close the block if it is
            self.check_if_block_completed_then_close(range);

            return;
        }

        // same block as the one that is open

        /*
            need to handle case where even though the chars are the same block type as the one that is open, this doesn't necessarily indicate that the chars should be written out to that same block
            eg. on two consecutive input lines, a dash list item block could be opened, and thus would need two dash list item blocks, rather than one

            therefore, need to check if the output label of the chars being written out indicates that a new block should be opened

            caveat on the check: if the output label is the same as the one of the current node (ie. still current, as these chars haven't been written out yet), and if the chars to be written out are contiguous to the last chars written out, then a new block should not be opened
            handling this case prevents openers with multiple characters from opening a new block per character
        */
        if output_label.is_block_opener()
            && !(output_label == self.current_node_type && self.current_node_end == range.start)
        {
            // opener to new block of the same type
            self.open_block(output_label, block_type_of_char, range);

            // some blocks span exactly one line - thematic and folding breaks, table row blocks
            // check if this situation is one of those, and close the block if it is
            self.check_if_block_completed_then_close(range);

            return;
        }

        // same block as the one already open, new block of the same type does not need to be opened
        // thus, write chars out to current block
        self.write_out_to_open_block(output_label, range);

        // some blocks span exactly one line - thematic and folding breaks, table row blocks
        // check if this situation is one of those, and close the block if it is
        self.check_if_block_completed_then_close(range);
    }

    #[inline(always)]
    /// Check if a block can - and should - be closed.
    /// Closes block if this is the case
    fn check_if_block_completed_then_close(&mut self, range: InputStringRange) {
        // some blocks span exactly one line - thematic and folding breaks, table row blocks
        //    ie. a newline character closes these blocks
        // check if this situation is one of those, and close the block if it is

        // check if the block type may require proactive closure
        if !self.current_block_type.does_newline_terminate_block() {
            return;
        }

        // block type may require proactive closure

        // get the chars to be written out
        let written = &self.input_string[range.start as usize..range.end as usize];

        // if the chars in the range don't contain a newline, then the block isn't terminated, so don't proceed
        if !written.contains(&b'\n') {
            return;
        }

        // closes the open block by writing out pending information to the node object, and creates a parse job for the now closed block
        self.close_open_block();

        // reset the workspace
        self.current_block = self.root;
        self.current_node = self.root;
        self.current_block_type = self.root_type;
        self.current_node_type = self.root_type;
        self.current_block_end = 0;
        self.current_node_end = 0;
    }

    #[inline(always)]
    /// Writes out a contiguous sequence of input bytes to the currently open block
    fn write_out_to_open_block(&mut self, output_label: NodeType, range: InputStringRange) {
        // is it the same node?
        // note that to add the chars to the same node, it also needs to be contiguous - if there's an exclusion zone in-between, a new node will need to be opened
        if output_label == self.current_node_type && self.current_node_end == range.start {
            // same node, range contiguous with already written chars - extend the range to capture the written out chars

            /*
                note that these two updates here and the use of the type in the if test above motivates the use of these cached values for the node/block types and end indices
                these accesses and updates now don't have to invoke operations on objects in the AST's array
            */

            // update the cached end indices of the current node and block
            self.current_node_end = range.end;
            self.current_block_end = range.end;
        } else {
            // need to write out to a new node

            // first, write out the cached value for node end, so AST version of node is correct
            // the block is not written out, as it will be staying open
            self.ast.nodes[self.current_node as usize].end = self.current_node_end;

            // create the new node
            let new_node = self
                .ast
                .new_detatched_node(output_label, range.start, range.end);

            // append the node to the end of the current block's child node list
            self.ast
                .append_child_to_parent(self.current_block, new_node);

            // update the workspace
            self.current_node = new_node;
            self.current_node_type = output_label;
            self.current_node_end = range.end;

            // increment the block range over the chars being written out
            // note that don't need to worry about non-contiguity (ie. exclusion zones), as the block range bounds over everything between its start and end
            self.current_block_end = range.end;
        }
    }

    #[inline]
    /// Writes out a contiguous sequence of input bytes to a new block
    fn open_block(
        &mut self,
        output_label: NodeType,
        block_type_of_char: NodeType,
        range: InputStringRange,
    ) {
        // opening a block closes whichever block was open before it, so the outgoing block becomes a job
        self.close_open_block();

        // first create the block itself
        let new_block = self
            .ast
            .new_detatched_node(block_type_of_char, range.start, range.end);

        // blocks produced by this block parser instance are children of the run's root
        self.ast.append_child_to_parent(self.root, new_block);

        /*
            need to handle case that the block is self explanatory
            ie. the block requires no further children to correctly interpret the semantics of the input chars

            this applies to thematic and folding breaks

            ie. if a block is self explanatory, the chars should be placed directly under the block
        */
        let (new_node, new_node_type) = if block_type_of_char.is_self_explanatory_block() {
            (new_block, block_type_of_char)
        } else {
            // create the node, detached, in the AST
            let node = self
                .ast
                .new_detatched_node(output_label, range.start, range.end);

            // attach the node to its parent in the AST
            self.ast.append_child_to_parent(new_block, node);

            (node, output_label)
        };

        // update workspace
        self.current_block = new_block;
        self.current_node = new_node;
        self.current_block_type = block_type_of_char;
        self.current_node_type = new_node_type;
        self.current_block_end = range.end;
        self.current_node_end = range.end;
    }

    #[inline(never)]
    /// Write out a non-contiguous sequence of input bytes to the AST, under the supplied node type.
    /// Used when chars that are interrupted by an exclusion zone are to be written out
    pub(crate) fn write_out_non_contiguous_chars(
        &mut self,
        output_label: NodeType,
        block_type_of_char: NodeType,
        ranges: &[InputStringRange],
    ) {
        // extract each contiguous range within the non-contiguous ranges, and run the contiguous write-out logic on each
        for range in merge_contiguous_ranges(ranges) {
            self.write_out_contiguous_chars(output_label, block_type_of_char, range);
        }
    }
}
