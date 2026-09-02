use super::ast_utils::{AST, NO_NODE, NodeId};
use super::node_utils::NodeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Used to describe whether content nodes in a block (eg. DashListItemContent for a dash list item) should be retained in the tree, or replaced by the parsed nodes that sit under them
pub(crate) enum ContentNodeBehaviour {
    // content nodes kept in the AST
    Retained,

    // content nodes not placed in AST, parsed nodes under the content node are placed directly in the block where the content node was
    Replaced,
}

/// Takes a block, and the parser output that was extracted from parsing the content nodes for that block (eg. DashListItemContent nodes for a dash list item block).
/// Arguments supply which node type corresponds to content entries for the block type, and whether content nodes should be retained in the AST, or whether the nodes parser output should be placed directly in the block (as opposed to under the content entry/entries)
pub(crate) fn place_parser_output_in_block(
    ast: &mut AST, // mutably borrow the AST so can modify to relink nodes
    block_to_rebuild: NodeId,
    content_node_type: NodeType, // eg. for a dash list item block, this will be DashListItemContent
    mut parser_output_to_place_in_block: NodeId, // note that the mut is put on the binding, not the type, so the list of nodes can be traversed through
    content_node_behaviour: ContentNodeBehaviour, // whether content nodes should be retained or replaced
) {
    // empty and get the block's child node list, so block can be rebuilt
    let mut block_child_node = ast.take_child_nodes(block_to_rebuild);

    /*
        a consuming block is a block within the paser output to be placed that started within a node of the block being rebuilt, but extends beyond it
        therefore, the consuming block may be required to consume nodes from the block being rebuilt

        eg. a paragraph spanning multiple lines within a single dash list item should consume the dash list item continuations that sit between the paragraph
    */
    let mut consuming_block = NO_NODE;
    let mut consuming_block_end = 0;

    /*
        where the last consumed node was placed,
        the last consumed node, which can be used to get where the last consumed node was placed in the consuming block
        this can be used to speed up looking for where the next node should be consumed
    */
    let mut last_consumed_node = NO_NODE;

    // traverse nodes in the block
    while block_child_node != NO_NODE {
        // deref to get the node
        let node = *ast.get_node(block_child_node);

        // read the next sibling before an append method may be called, which will clear the pointer
        let next_child = node.next_sibling;

        // is a block in the parser output to place consuming nodes at the moment?
        if consuming_block != NO_NODE {
            // a block that may consume nodes from the block being rebuilt is open

            // is the node within the range of the consuming block
            if node.end <= consuming_block_end {
                // the node should be consumed by the block
                // eg. a dash list continuation being consumed by a paragrpah within the dash list content

                // note that if the node is of content type, it is dropped from the block
                // this is because if a consuming block is open (which it is in this if-block), a content entry must have already been handled, thus if it's meant to be in the AST, it'll already be there
                if node.node_type != content_node_type {
                    last_consumed_node =
                        consume_node(ast, consuming_block, last_consumed_node, block_child_node);
                }

                // advance the node in the block for being rebuilt
                block_child_node = next_child;

                // move on to the next node
                continue;
            }

            // node is not within the range of the consuming block
            // therefore it should not be consumed, and the consuming block needs to be cleared
            consuming_block = NO_NODE;

            // continue processing now that the consuming block has been cleared
        }

        /*
            here, no block is consuming nodes
        */

        // if the node is not a content node, and a block is not consuming, the node should stay where it is in the block
        if node.node_type != content_node_type {
            // put the node back in the block, which will put it in the same place
            ast.append_child_to_parent(block_to_rebuild, block_child_node);

            // advance the node in the block being rebuilt
            block_child_node = next_child;

            continue;
        }

        // node is a content node, no block is consuming nodes
        // if content nodes should be retained, push it to the AST, and use that node to place the parser output under
        // else, ignore it, and use the root of the block being rebuilt for placing the parser output under
        let parent_node = match content_node_behaviour {
            ContentNodeBehaviour::Retained => {
                ast.append_child_to_parent(block_to_rebuild, block_child_node);

                block_child_node // use the content node as the root for placing the parser output under
            }

            ContentNodeBehaviour::Replaced => block_to_rebuild, // use the root node of block for placing the parser output under
        };

        // a content node has been found, place the parser output (position will be correct, as set up above)
        while parser_output_to_place_in_block != NO_NODE {
            // dereference the node to be placed
            let node_to_be_placed = *ast.get_node(parser_output_to_place_in_block);

            /*
                if the block begins at or after the end of the node to be placed
                if the node to be placed begins at or after the end of the current node in the block being rebuilt, let the next iteration on the block being rebuilt handle it
                ---

                this check also catches an important edge case

                consider input: **a*b**
                with no newline at the end

                this input will result in an empty italic node being re-opened after the bold is closed
                this will be caught by this if-statement below, preventing it from being placed in the tree
            */
            if node_to_be_placed.start >= node.end {
                break;
            }

            // check that the edge case has been caught correctly by the `break` above
            debug_assert_ne!(
                node_to_be_placed.start, node_to_be_placed.end,
                "zero-width produced node {parser_output_to_place_in_block} placed under {parent_node}",
            );

            // read the next sibling before the append, which will clear the pointer
            let next_node_to_be_placed = node_to_be_placed.next_sibling;

            // place the node
            ast.append_child_to_parent(parent_node, parser_output_to_place_in_block);

            // the node being placed extends beyond the current node in the block being rebuilt
            // eg. consider a paragraph that spans multiple lines of a list item
            // in such a case, the paragraph should consume the list item continuations
            if node_to_be_placed.end > node.end {
                // set the consuming block, and its end
                consuming_block = parser_output_to_place_in_block;
                consuming_block_end = node_to_be_placed.end;

                // set the last consumed node to none (as consuming block is being newly set)
                last_consumed_node = NO_NODE;

                // advance the node in the output needing to be placed
                parser_output_to_place_in_block = next_node_to_be_placed;

                // break so that the consuming block logic runs for the next `parser_output_to_place_in_block` node
                break;
            }

            // advance the node in the output needing to be placed
            parser_output_to_place_in_block = next_node_to_be_placed;
        }

        // advance the node in the block for being rebuilt
        block_child_node = next_child;
    }

    /*
        check that the only unplaced nodes are of zero length, so that nothing has been missed
    */
    #[cfg(debug_assertions)]
    {
        let mut unplaced = parser_output_to_place_in_block;

        while unplaced != NO_NODE {
            let node = ast.get_node(unplaced);

            debug_assert_eq!(
                node.start, node.end,
                "produced node {} covering [{}, {}) was never placed",
                unplaced, node.start, node.end,
            );

            unplaced = node.next_sibling;
        }
    }
}

#[inline]
/// Takes a block, and a node for that block to consume.
/// The node to consume is placed in the correct position according to the input range it captures.
/// The last consumed node is also taken, to speed up finding the correct location for the node.
/// Returns the consumed node, which will have updated properties, so that the function can be re-called later with an updated [`last_consumed_node`]
fn consume_node(
    ast: &mut AST,
    consuming_block: NodeId,
    last_consumed_node: NodeId,
    node_to_consume: NodeId,
) -> NodeId {
    // get the start of the node to be consumed, so can look for an appropriate node to put it after in the block
    let start = ast.get_node(node_to_consume).start;

    // used to find a node which the node to be consumed should be put after
    // this will either be the last node that starts before the node to be consumed starts, or NO_NODE if no suitable predecessor can be found
    let mut precedes_node_to_consume = last_consumed_node;

    // traverse the block, starting from the last consumed node (if one has been previously consumed - this works as nodes to be consumed appear in order wrt. the input string)
    loop {
        let next = if precedes_node_to_consume == NO_NODE {
            // initial iteration behaviour: this is the first node to be consumed by the block, start looking from the first child node of the root of the block
            // note that this only runs on the initial iteration
            ast.get_node(consuming_block).first_child
        } else {
            // initial iteration behaviour: if a node has been consumed by the block previously, start looking from that previously consumed node
            // works because nodes to be consumed appear in input order

            // for subsequent iterations, this simply increments over sibling nodes

            ast.get_node(precedes_node_to_consume).next_sibling
        };

        // if next is NO_NODE, then the list has been traversed to its end (required to be first condition in expression, to prevent get_node being called with NO_NODE)
        if next == NO_NODE || ast.get_node(next).start >= start {
            // note that the `precedes_node_to_consume = next` update hasn't run yet, so precedes_node_to_consume still points to the predecessor node, whilst next points to a node that starts at or after the start of the node to be consumed
            // therefore, precedes_node_to_consume will precede next
            break;
        }

        // update the pointer - will update until this points to the last node that starts before the node to consume
        precedes_node_to_consume = next;
    }

    // place in the block based on the attempt to find a predecessor
    if precedes_node_to_consume == NO_NODE {
        // no appropriate predecessor
        // therefore, put at front of list of child nodes

        let first_node_in_block = ast.get_node(consuming_block).first_child;

        // set the sibling of the node to consume to the first child node of the block
        ast.get_mut_node(node_to_consume).next_sibling = first_node_in_block;

        // set the node to consume to be the first child node
        ast.get_mut_node(consuming_block).first_child = node_to_consume;

        // if the list of child nodes for the block was empty, also need to set the last_child property on the block
        if first_node_in_block == NO_NODE {
            ast.get_mut_node(consuming_block).last_child = node_to_consume;
        }
    } else {
        // an appropriate predecessor node has been found
        // therefore, put the node to consume after it

        // read the next node after the predecessor node, before changing the pointer
        let next_node = ast.get_node(precedes_node_to_consume).next_sibling;

        // set the node to consume to point to the sibling of the predecessor node
        ast.get_mut_node(node_to_consume).next_sibling = next_node;

        // set the predecessor to point to the node to consume
        ast.get_mut_node(precedes_node_to_consume).next_sibling = node_to_consume;

        // if the predecessor node was the last child node of the block, also need to update the last_child property on the block
        if next_node == NO_NODE {
            ast.get_mut_node(consuming_block).last_child = node_to_consume;
        }
    }

    // return the consumed node, which has now been updated, so that a subsequent consume_node() call can traverse forward from this successfully consumed node
    node_to_consume
}
