use super::ast_to_html::HtmlRenderer;
use super::ast_utils::{NO_NODE, Node, NodeId};
use super::node_utils::NodeType;

#[inline]
/// Takes the input range captured by a callout type node (eg. '[!warning] '), and returns the range of the type string inside the '[!' and '] '
pub(super) fn get_range_for_callout_type_chars(source: &[u8], start: u32, end: u32) -> (u32, u32) {
    let mut start = start as usize;
    let mut end = end as usize;

    // "[!"
    if source[start..end].starts_with(b"[!") {
        start += 2;
    }

    // trailing space after the ']'
    while end > start && source[end - 1].is_ascii_whitespace() {
        end -= 1;
    }

    // "]"
    if end > start && source[end - 1] == b']' {
        end -= 1;
    }

    (start as u32, end as u32)
}

#[inline]
/// Takes the range of a num list item opener, and returns the range of just the digits of the number
pub(super) fn get_range_for_num_in_num_list_item(
    source: &[u8],
    start: u32,
    end: u32,
) -> Option<(u32, u32)> {
    let mut cursor = start as usize;
    let end = end as usize;

    // iterate over leading whitespace (num list items can be preceded by up to three spaces of indentation)
    while cursor < end && source[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    // mark the first digit found
    let first_digit = cursor;

    // increment the cursor over the digits whilst in the opener
    while cursor < end && source[cursor].is_ascii_digit() {
        cursor += 1;
    }

    // if the cursor hasn't been incremented, then no digits were found
    if cursor == first_digit {
        return None;
    }

    Some((first_digit as u32, cursor as u32))
}

impl HtmlRenderer<'_> {
    #[inline(never)]
    /// Write out opening tag for a check list item, handling if it's checked or not, based on the check list character supplied to the function
    pub(super) fn open_check_list_item(
        &mut self,
        check_list_char: (u32, u32), // the char between the square brackets
        node_id: NodeId,
        node_type: NodeType,
    ) {
        // destructure the range
        let (character_start, character_end) = check_list_char;

        // determine if the list item is checked (by determining that it's not unchecked or partially checked)
        let is_checked = !matches!(
            &self.input_string_bytes[character_start as usize..character_end as usize],
            b" " | b"\t" | b"/"
        );

        // open the list item
        self.open_block(
            b"<li class=\"check-list-item\" data-check-value=\"",
            node_id,
            node_type,
        );

        // push the char between the check list's square brackets to the output
        self.escape_and_push_to_html(character_start, character_end);

        // push bytes so list item is check list item
        // useful for screen readers
        self.html_output_array
            .extend_from_slice(b"\"><input class=\"checkbox\" type=\"checkbox\" disabled");

        // add styling for if it's checked or not - half checked or unchecked handled by css
        if is_checked {
            self.html_output_array.extend_from_slice(b" checked");
        }

        // close the opening tag
        self.html_output_array.push(b'>');
    }

    #[inline]
    /// Takes a node for a dash, plus or asterisk list item, and returns the range of the character inside the square brackets (if defined), else None for a normal dash/plus/asterisk list item
    pub(super) fn get_check_list_char(&self, list_item_node: &Node) -> Option<(u32, u32)> {
        /*
            eg. - [x] checked list item

            will be exactly one char in between the square brackets if it's a valid check list item
            the square brackets and char between them are in the opener of the list item
        */

        // get the first child of the list item - it will be the opener
        let first_child_node_id = list_item_node.first_child;

        // empty list check to be safe
        if first_child_node_id == NO_NODE {
            return None;
        }

        // deref the node
        let opener_node = self.ast.get_node(first_child_node_id);

        // get range bounds of opener
        let (mut start, mut end) = (opener_node.start as usize, opener_node.end as usize);

        // if the opener is less than six characters in length, then it cannot be a check list item
        // if it is six or more chars in length, and does not end with a square bracket, then it cannot be a check list item
        if end < start + 6 || self.input_string_bytes[end - 2] != b']' {
            return None;
        }

        // need to find char after '[' and before ']'

        // the ']' is easy to find - it's simply the second last char in the opener, so point the end at it - which is the byte after the character in the box
        end -= 2;

        // to find the '[', increment start until found - must be found before ']'
        while start <= end {
            if self.input_string_bytes[start] == b'[' {
                // the character runs from the byte after the '[' up to the ']'
                return Some((start as u32 + 1, end as u32));
            }
            start += 1; // works because chars between the start of a list item opener and the opening square bracket must be ASCII, thus will be exactly one byte
        }

        // not reached in practice - if while loop is met, then indices will be found
        None
    }
}
