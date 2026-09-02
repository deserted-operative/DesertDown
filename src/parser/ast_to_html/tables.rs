use super::ast_to_html::{HtmlRenderer, is_content_node, is_continuation_node};
use super::ast_utils::{NO_NODE, NodeId};
use super::node_utils::NodeType;

/*
    alignment given to a table column by the separator row

    stored per column rather than resolved per cell, because the separator row
    is read once when the table opens and every later cell just indexes it
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableColumnAlignment {
    None,
    Left,
    Centre,
    Right,
}

impl TableColumnAlignment {
    #[inline]
    pub(super) const fn get_styling(self) -> &'static [u8] {
        match self {
            TableColumnAlignment::None => b"",
            TableColumnAlignment::Left => b" class=\"align-left\"",
            TableColumnAlignment::Centre => b" class=\"align-centre\"",
            TableColumnAlignment::Right => b" class=\"align-right\"",
        }
    }
}

/*
    constant for the number of columns for which alignment styling is maintined for
    more table columns may be specified, but alignment styling simply won't be applied
    ie. the columns will still be shown
*/
pub(super) const MAX_ALIGNED_TABLE_COLUMNS: usize = 64;

impl HtmlRenderer<'_> {
    /// Open a table block.
    /// Handles the separator row, if one is specified in the correct position
    pub(super) fn open_table_block(&mut self, first_row_node_id: NodeId) {
        // init states for building the table
        self.table_column = 0;
        self.aligned_column_count = 0;

        // if the first row is not a separator row, and the second row is a separator row, then the table has a header row
        let has_header = self.is_there_a_separator_row(first_row_node_id);
        self.in_table_header_row = has_header;

        // push the according tags for the table
        // note that each table is wrapped in a div
        if has_header {
            // thead for table with header
            self.html_output_array
                .extend_from_slice(b"<div class=\"table-scroll\">\n<table>\n<thead>\n");
        } else {
            // tbody for table without header
            self.html_output_array
                .extend_from_slice(b"<div class=\"table-scroll\">\n<table>\n<tbody>\n");
        }
    }

    /// Takes the ID of the node for the first table row block that opens a table, and determines whether there's a valid separator row, in the correct position
    fn is_there_a_separator_row(&mut self, first_row_node_id: NodeId) -> bool {
        // get the sibling node of the (first) table row block - this may or may not be another table row block
        // if it's not, that doesn't mean that the table shouldn't continue - the table may be within another block
        let mut sibling_id = self.ast.get_node(first_row_node_id).next_sibling;

        // get the frame on the top of the stack - this is the one being processed
        let mut frame = self.stack.len() - 1;

        // var to keep track of whether the search for a separator row has stepped into a content node
        let mut stepped_into_content_node = false;

        // begin search for separator row
        loop {
            // if sibling id is NO_NODE, then the end of the node list has been reached
            if sibling_id == NO_NODE {
                // node list end, therefore the next line is outside this frame, or there is no next line
                match self.continue_traversing_outside_frame(frame) {
                    // if no location to resume searching for was found, return false, as no separator row is specified
                    Some(_) if stepped_into_content_node => return false,

                    // if some location to resume searching for the separator row was found, update the vars, and use continue to trigger the logic to deal with the separator row on the next iteration
                    Some((resumed, outside)) => {
                        sibling_id = resumed;
                        frame = outside;

                        continue;
                    }

                    // if no location to resume searching for was found, return false, as no separator row is specified
                    None => return false,
                }
            }

            // deref the node
            let node = *self.ast.get_node(sibling_id);

            // action to take depends on the node type
            match node.node_type {
                // if table row block found, it's the one after the first
                // therefore, it's either the separator row, or the table has no header
                NodeType::TableRowBlock => {
                    // if not a separator row, then there's no separator row
                    if !self.is_separator_row(node.first_child) {
                        return false;
                    }

                    // is a separator row, set the alignment array for each column, up to the maximum number of aligned columns
                    let mut child = node.first_child;
                    let mut column = 0;

                    while child != NO_NODE && column < MAX_ALIGNED_TABLE_COLUMNS {
                        let child_node = *self.ast.get_node(child);

                        let alignment = match child_node.node_type {
                            NodeType::LeftAlignedTableHeaderRow => TableColumnAlignment::Left,
                            NodeType::CentredTableHeaderRow => TableColumnAlignment::Centre,
                            NodeType::RightAlignedTableHeaderRow => TableColumnAlignment::Right,
                            NodeType::TableHeaderRow => TableColumnAlignment::None,

                            // table cell content can appear in a separator row, so handle it
                            NodeType::TableCellContent => TableColumnAlignment::None,

                            // a table wall - ignore rather than error
                            _ => {
                                child = child_node.next_sibling;

                                continue;
                            }
                        };

                        // store the alignment
                        self.table_alignments[column] = alignment;

                        // increment column count for row
                        column += 1;

                        // increment to next node
                        child = child_node.next_sibling;
                    }

                    // set the number of aligned columns found
                    self.aligned_column_count = column;

                    // true as separator row found
                    return true;
                }

                // if a newline is found after a table row block and it only contains newlines, then it's just the termintor to that row
                // look for next sibling, which may be a table row block (which may be the separator row being searched for)
                NodeType::Paragraph
                    if self.get_number_of_blank_lines_in_blank_paragraph(&node) == Some(0) =>
                {
                    sibling_id = node.next_sibling;
                }

                // if a continuation node is found (ie. indent or callout continuation), need to look at next sibling, which may be a table row block (which may be the separator row being searched for)
                continuation_node if is_continuation_node(continuation_node) => {
                    sibling_id = node.next_sibling
                }

                // the next line of a callout or an indent, which is where the row it holds is
                // if a content node is found (ie. indent or callout content), then need to look inside the node for a table row block (which may be the separator row being searched for)
                content_node if is_content_node(content_node) => {
                    // if already stepped into a content node at an earlier point, don't want to go another level deep, so no separator row found
                    if stepped_into_content_node {
                        return false;
                    }

                    // stpe into the content node to look for the separator row
                    stepped_into_content_node = true;
                    sibling_id = node.first_child;
                }

                // anything else, end the table
                _ => return false,
            }
        }
    }

    #[inline]
    /// Takes a node ID for a table row block, and returns whether it's a separator row (true), or a normal content row (false).
    /// A row is taken as a separator row if it contains ANY separator row node types, even if it contains any content cells
    pub(super) fn is_separator_row(&self, first_child: NodeId) -> bool {
        /*
            separator row eg. | --- | :-- | --: |

            note that a row is taken as a separator row if it contains at least one alignment node
            if it ALSO contains any content cells, then it is still taken as a separator row, so that the AST to HTML can proceed in a best-effort fashion
        */

        // set up traversal node
        let mut child_node = first_child;

        // until child list of table row block has been fully traversed - need to traverse either fully or until a node that indicates a separator row is encountered
        while child_node != NO_NODE {
            // deref the node
            let node = self.ast.get_node(child_node);

            match node.node_type {
                // encountering one alignment node indicates that this is a separator row
                NodeType::TableHeaderRow
                | NodeType::LeftAlignedTableHeaderRow
                | NodeType::CentredTableHeaderRow
                | NodeType::RightAlignedTableHeaderRow => return true,

                // a content cell or table wall, keep looking
                _ => child_node = node.next_sibling,
            }
        }

        // no alignment node found, thus not a separator row
        false
    }
}
