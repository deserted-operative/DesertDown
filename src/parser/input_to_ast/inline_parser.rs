use super::{
    ACTION_LOOKUP_WIDTH, INLINE_STATE_COUNT, INLINE_TRANSITIONS, SYMBOL_COUNT,
    action_utils::{
        ACTIONS_OF_INLINE_FAST_TRACK_STATES, INLINE_ACTION_TO_INLINE_TYPE, evaluate_action,
    },
    ast_utils::{AST, NO_NODE, NodeId},
    char_buffer_utils::ParserStructures,
    inline_actions::{INLINE_ACTION_LOOKUP, INLINE_ACTIONS},
    inline_parser_utils::{
        InlineParserWorkspace, OpenInline, get_info_for_non_matching_emphasis_closure,
        get_node_type_from_emph_levels,
    },
    input_string_utils::{InputCharsForParsing, InputStringRange, InputStringView},
    node_utils::NodeType,
    transition_utils::get_char_column,
};

// index of the start state in the inline transition array
// should double check the inline_transitions.rs to ensure that this holds true
pub const INLINE_INITIAL_STATE: usize = 0;

// italic = 1, bold = 2, bold and italic = 3
// cannot have more than 3 levels of emphasis open at any one point
pub const MAX_EMPHASIS_LEVEL: u8 = 3;

pub fn inline_parser(
    input: InputStringView<'_, '_>,
    ast: &mut AST,
    root: NodeId,
    parser_structures: &mut ParserStructures,
) {
    let (input_start_index, _) = input.get_view_bounds(); // destructured tuple, _ discards the end offset, as not needed here

    let mut state = INLINE_INITIAL_STATE;

    // destructure the structures out of the object
    let ParserStructures {
        buffer,
        stack,
        requires_reopening: reopen,
    } = parser_structures;

    // reset the buffer, as it's shared between runs of the inline parser
    // is ok because of single-threaded implementation
    buffer.reset();

    // reset the stack, as it's shared between runs of the inline parser
    // is ok because of single-threaded implementation
    stack.clear();

    /*
        push the root node to the bottom of the stack
        this node isn't closed by the inline parser

        given type of Document, as that isn't an inline, isn't a self contained node type, isn't a delimiter node type
        ie. is a safe choice
    */
    stack.push(OpenInline {
        node: root,
        node_type: NodeType::Document,
        emphasis_levels: 0,
    });

    let mut workspace = InlineParserWorkspace {
        ast,
        stack,
        requires_reopening: reopen,
        last_end: input_start_index,
        pending_node: NO_NODE,
        pending_parent: NO_NODE,
        pending_label: NodeType::NoOp,
        pending_end: input_start_index,
    };

    // the number of open emphasis levels, between 0 and 3 (both inclusive)
    let mut emphasis_levels: u8 = 0;

    // track where last input character ends, so the flushed null char can be given an empty span at the right place
    // inits to input start, so an empty view yields start == end == input_start
    let mut last_input_char_processed_end = input_start_index;

    // closure for running one character through the automaton, and performing the action(s) of the state reached by consuming the character
    let mut step = |first_byte_of_char: u8,
                    range: InputStringRange,
                    chars: &mut InputCharsForParsing<'_, '_>| {
        /*
            transitions are, conceptually, stored as a 2D array, where each state has a row
            each column (in a row) stores the row index of the state reached by consuming the current character

            thus, column represents the row index of the state reached after consuming this iteration's character
        */
        let column = get_char_column(first_byte_of_char);

        // debug check that current state and column indices are valid
        debug_assert!(state < INLINE_STATE_COUNT);
        debug_assert!(column < SYMBOL_COUNT);

        // lookup what state is reached by consuming this character from the current state
        // then update the current state
        state = INLINE_TRANSITIONS[state * SYMBOL_COUNT + column] as usize;

        /*
            check for, and use if possible, fast-track processing
            if the reached state performs exactly one action, and that action fully flushes the buffer, it is eligible for fast-track processing

            under such a strategy, the parser walks forward over contiguous characters that loop back onto this same automaton state, where they can be efficiently processed without using the full buffer procedure

            because this applies to the majority of characters in an input string (ie. most of the characters that aren't meta characters), the performance improvement gained by this strategy is significant
        */

        // get the index of the action in the actions array of the action performed by this state
        // the lookup table has precomputed some checks, so 0 is returned if the state is ineligible for fast-track processing, or the index of the action + 1 if it is eligible
        let fast_track_action = ACTIONS_OF_INLINE_FAST_TRACK_STATES[state] as usize;

        // buffer must be empty to qualify for fast-track processing - can't fast-track if there's characters pending write-out
        if buffer.get_len() == 0 && fast_track_action != 0 {
            // fast-track processing feasible

            /*
                edge case - if the range is empty (ie. end overlaps with start), then nothing should be written out to the AST
                this allows flush characters (a null char for the inline parser) to be run through the machine, but not be presented in the AST (as they're not part of the original input string)

                additionally to how this step behaves in the block parser, an inline must not be opened in processing the flush character
            */
            if range.start == range.end {
                return;
            }

            let action_index = fast_track_action - 1; // need to take the one added off to get the valid index back (see ACTIONS_OF_INLINE_FAST_TRACK_STATES implementation)
            let output_label = INLINE_ACTIONS[action_index].output_label;
            let inline_type_of_char = INLINE_ACTION_TO_INLINE_TYPE[action_index];
            let mut input_range = range;
            let row_index = state * SYMBOL_COUNT;

            /*
                get the range of chars fast-tracked over, or None if nothing could be walked over

                the logic for fast-tracking over self loops advances the cursor property of the InputCharsForParsing, which is hooked into the custom Iterator implementation
                thus, don't need to worry about the next char for processing being wrong, as a result of dealing with more than one character in this closure

                additional restrictions apply to this logic compared to in the block parser
                chars that form delimiters are excluded from the fast path, as they influence decisions on the input, rather than just being literals
            */
            if !matches!(
                inline_type_of_char,
                NodeType::Emphasis
                    | NodeType::Highlighting
                    | NodeType::Underline
                    | NodeType::Strikethrough
            ) && let Some(range_fast_tracked_over) = chars.fast_track_over_self_loops(
                &INLINE_TRANSITIONS[row_index..row_index + SYMBOL_COUNT],
                state as super::StateId,
            ) {
                // debug check that the fast track range starts after this step's character
                debug_assert_eq!(input_range.end, range_fast_tracked_over.start);

                // extend the current step's range to now include the fast track characters
                input_range.end = range_fast_tracked_over.end;
            }

            // address consequences of writing out chars on the stack of open inlines
            match inline_type_of_char {
                // an output label type that describes no inline of its own - ie. an escape backslash - is written out at the current node, and doesn't close it
                NodeType::NoOp => {
                    workspace.write_out_chars_in_range_to_newest_inline(output_label, input_range);

                    return;
                }

                // these types are specified with delimiters
                // the consequences of these are handled by the full buffer procedure - which the `{}`, with no return statement, causes a fall through to
                NodeType::Emphasis
                | NodeType::Highlighting
                | NodeType::Underline
                | NodeType::Strikethrough => {}

                /*
                    principally captures NodeType::Literal => NodeType::Literal
                    case must be below the delimiter cases above, otherwise it'd incorrectly capture those

                    in this case, the chars are written out to the node on the top of the stack
                */
                node_type if node_type == output_label => {
                    // after a self contained inline (eg. inline code) is closed, the node for it may still be on the stack
                    // therefore, need to close these self contained inlines (because a literal that gets captured in this case won't be part of one)
                    workspace.close_self_contained_inlines();

                    // now, after closing any open self-contained inlines, the inline on the top of the stack will be the correct owner for the literal char
                    workspace.write_out_chars_in_range_to_newest_inline(output_label, input_range);

                    return;
                }

                /*
                    anything not already captured by the previous cases is a self-contained inline
                    ie. nothing else can be within them, they take precedence in capturing characters

                    note that for a self-contained inline, the node types are reliably processed in order of: opener, content, closer
                */
                _ => {
                    // determine if a new self contained inline needs to be opened

                    /*
                        solves edge cases like: `a``b`, which before would be captured as one code inline, rather than two
                    */
                    if workspace.is_new_self_contained_inline_required(
                        inline_type_of_char,
                        output_label,
                        range.start,
                    ) {
                        // close any opened self contained inlines on the stack
                        workspace.close_self_contained_inlines();

                        // open a new one
                        workspace.open_inline(inline_type_of_char, 0, range.start);
                    }

                    // here, the correct inline for these chars is on the top of the stack

                    // get that inline
                    let node = workspace.get_newest_opened_inline_node();

                    // write them out
                    workspace.write_out_contiguous_chars(node, output_label, input_range);

                    return;
                }
            }
        }

        /*
            if here, need to use the full buffer procedure

            can get here if fast-track processing wasn't suitable, or if an emphasis delimiter was encountered
        */

        // push the char to the buffer (range captures one character - fast track processing doesn't modify `range`, so if falling through, will still be one char)
        buffer.push_char_to_buffer(range);

        // lookup the first action associated with the reached state, and how many actions the state performs (will be at least one)
        let first_action_index = INLINE_ACTION_LOOKUP[state * ACTION_LOOKUP_WIDTH] as usize;
        let action_count = INLINE_ACTION_LOOKUP[state * ACTION_LOOKUP_WIDTH + 1] as usize;

        // perform the reached state's action(s)
        for offset in 0..action_count {
            // get the index in the actions array of the action that needs to be performed
            let action_index = first_action_index + offset;

            // evaluate action on the buffer
            let (mut evaluated_range, output_label) =
                evaluate_action(INLINE_ACTIONS[action_index], buffer.get_len());

            // don't do any more in this inner loop if this action writes out no characters from the front of the buffer
            if evaluated_range == 0 {
                continue;
            }

            // debug check that the evaluated range is valid
            // however, note below that if the range is more than the buffer length, it simply takes the length of the buffer, to avoid a prod panic
            debug_assert!(evaluated_range <= buffer.get_len());
            if evaluated_range > buffer.get_len() {
                evaluated_range = buffer.get_len();
            }

            /*
                the parser flushes a null char through the automaton at the end
                it is sent through the closure as a range with no length, allowing it to be distinguished that it shouldn't be written out to the AST

                this flush character can only ever be the last in a sequence of written out characters

                therefore, if the first character in the buffer is the null char, then there's no characters of interest to deal with, and can skip the processing logic
            */
            let first_char_in_buffer = buffer.get_buffer_as_slice()[0];

            if first_char_in_buffer.start != first_char_in_buffer.end {
                // use precomputed lookup table based on action indices to efficiently determine which inline structure type this output label belongs to
                let inline_type_of_char = INLINE_ACTION_TO_INLINE_TYPE[action_index];

                // address consequences of writing out chars on the stack of open inlines
                match inline_type_of_char {
                    // an output label type that describes no inline of its own - ie. an escape backslash - is written out at the current node, and doesn't close it
                    NodeType::NoOp => {
                        workspace.write_out_chars_from_buffer_to_newest_inline(
                            output_label,
                            buffer,
                            evaluated_range,
                        );
                    }

                    // emphasis is specified with delimiters, and the number of levels of open emphasis need checking, bounding, and altering
                    // the complicating factor with emphasis - that doesn't burden highlighting/strikethrough/underlining - is that it's not binary open/close, instead having numbers of levels that can be opened/closed
                    NodeType::Emphasis => {
                        // take the delimiter - EmphDelimiter, DoubleEmphDelimiter, TripleEmphDelimiter - and convert it to the number of levels of emphasis it corresponds to opening/closing
                        let levels_implied_by_delimiter: u8 = match output_label {
                            NodeType::EmphDelimiter => 1,
                            NodeType::DoubleEmphDelimiter => 2,
                            _ => 3,
                        };

                        // get the node type associated with chars captured by that many levels of emphasis
                        // eg. 2 levels of emphasis => bold
                        let node_type = get_node_type_from_emph_levels(levels_implied_by_delimiter);

                        // if the automaton has returned any chars associated with emphasis, then anything open that is self-contained must be closed
                        workspace.close_self_contained_inlines();

                        // search down from the top of the stack for a node that exactly closes the number of levels of emphasis indicated by the delimiter
                        // rposition for reverse search, as want to look from top of stack, down
                        if let Some(stack_index_of_open_emph_node) = workspace
                            .stack
                            .iter()
                            .rposition(|open| open.node_type == node_type)
                        {
                            /*
                                an emphasis node of exactly the same number of levels has been found to be open
                                therefore, it can be closed

                                note that this does NOT mean that no levels of emphasis will remain open after the node has been closed
                                eg. **a*b**
                                where after the bold has been closed, the italic remains open
                            */

                            // proceed to close that emphasis node, re-opening everything above it on the stack after the emph node is closed
                            workspace.close_inline_with_delimiter(
                                stack_index_of_open_emph_node,
                                None,
                                false,
                                output_label,
                                buffer,
                                evaluated_range,
                            );

                            // subtract the closed levels of emphasis from the count
                            emphasis_levels -= levels_implied_by_delimiter;
                        } else if emphasis_levels + levels_implied_by_delimiter
                            <= MAX_EMPHASIS_LEVEL
                            || emphasis_levels < levels_implied_by_delimiter
                        {
                            /*
                                opening this many levels of emphasis would not exceed the limit
                                therefore, this emphasis can be opened

                                note that some subtleties apply:

                                the first clause: `emphasis_levels + levels_implied_by_delimiter <= MAX_EMPHASIS_LEVEL` is the simple case
                                the second clause: `emphasis_levels < levels_implied_by_delimiter` is there to encapsulate that there are levels of emphasis that can be opened

                                one might imagine that if one level of emphasis is open, and the delimiter implies three levels of emphasis, the sum would exceed the limit of three
                                the automaton, however, enforces the number of open levels of emphasis to be no more than three, and will not write out a delimiter unless it is valid
                                thus, the situation just outlined, is caught earlier in the process
                            */

                            // proceed to open that emphasis node
                            workspace.open_inline_from_delimiter(
                                node_type,
                                levels_implied_by_delimiter,
                                output_label,
                                buffer,
                                evaluated_range,
                            );

                            // add the closed levels of emphasis from the count
                            emphasis_levels += levels_implied_by_delimiter;
                        } else {
                            /*
                                here, emphasis cannot be opened, and the delimiter does not have a node that matches that many levels of emphasis to close
                                therefore, it must walk down from the top of the stack, looking to close

                                closing delimiter cases (delimiter on left is the closing one, the cases on the right will be delimiters encountered when walking down from the top of the stack) (recall that only valid delimiters are placed, and only in valid locations):
                                    triple delimiter *** : will find a 2 then a 1, or a 1 then a 2
                                    double delimiter ** : will find a 3
                                    single delimiter * : will find a 3
                            */

                            // get the index of the inline on the stack that is closest to the bottom of the stack required to close the levels
                            //      eg. consider a triple emphasis delimiter, where three levels were opened with a double then a single: **a*b***. In this example, the index of the double delimiter is of interest, as outside that node is where the tree should continue to be built
                            // may be returned a residual of emphasis to be re-opened if fewer levels were closed than open
                            let (index, residual) = get_info_for_non_matching_emphasis_closure(
                                workspace.stack.as_slice(),
                                levels_implied_by_delimiter,
                            );

                            // proceed to close that emphasis node, re-opening everything above it on the stack after the emph node is closed, AFTER re-opening any residual emphasis that wasn't closed
                            workspace.close_inline_with_delimiter(
                                index,
                                residual,
                                true,
                                output_label,
                                buffer,
                                evaluated_range,
                            );

                            emphasis_levels -= levels_implied_by_delimiter;
                        }
                    }

                    // inlines specified with paired delimiters - ie. one to open, one to close
                    // ie. only need to evaluate whether one is already open
                    NodeType::Highlighting | NodeType::Underline | NodeType::Strikethrough => {
                        let node_type = match output_label {
                            NodeType::HighlightC1Delimiter => NodeType::HighlightC1,
                            NodeType::HighlightC2Delimiter => NodeType::HighlightC2,

                            // underlining and strikethrough each have one delimiter, so don't need to split them out into variants as required for highlighting
                            _ => inline_type_of_char,
                        };

                        // if the automaton has returned any chars associated with highlighting/underlining/strikethrough, then anything open that is self-contained must be closed
                        workspace.close_self_contained_inlines();

                        // look for the inline closest to the top of the stack that matches
                        if let Some(index) = workspace
                            .stack
                            .iter()
                            .rposition(|open| open.node_type == node_type)
                        {
                            // open inline of same type found, close it

                            workspace.close_inline_with_delimiter(
                                index,
                                None,
                                false,
                                output_label,
                                buffer,
                                evaluated_range,
                            );
                        } else {
                            // no inline open of same type, so open one

                            workspace.open_inline_from_delimiter(
                                node_type,
                                0,
                                output_label,
                                buffer,
                                evaluated_range,
                            );
                        }
                    }

                    /*
                        principally captures NodeType::Literal => NodeType::Literal
                        case must be below the delimiter cases above, otherwise it'd incorrectly capture those

                        in this case, the chars are written out to the node on the top of the stack
                    */
                    node_type if node_type == output_label => {
                        // after a self contained inline (eg. inline code) is closed, the node for it may still be on the stack
                        // therefore, need to close these self contained inlines (because a literal that gets captured in this case won't be part of one)
                        workspace.close_self_contained_inlines();

                        // now, after closing any open self-contained inlines, the inline on the top of the stack will be the correct owner for the literal char
                        workspace.write_out_chars_from_buffer_to_newest_inline(
                            output_label,
                            buffer,
                            evaluated_range,
                        );
                    }

                    /*
                        anything not already captured by the previous cases is a self-contained inline
                        ie. nothing else can be within them, they take precedence in capturing characters

                        note that for a self-contained inline, the node types are reliably processed in order of: opener, content, closer
                    */
                    _ => {
                        // determine if a new self contained inline needs to be opened

                        let start = buffer.get_buffer_as_slice()[0].start;

                        /*
                            solves edge cases like: `a``b`, which before would be captured as one code inline, rather than two
                        */
                        if workspace.is_new_self_contained_inline_required(
                            inline_type_of_char,
                            output_label,
                            start,
                        ) {
                            // close any opened self contained inlines on the stack
                            workspace.close_self_contained_inlines();

                            // open a new one
                            workspace.open_inline(inline_type_of_char, 0, start);
                        }

                        // here, the correct inline for these chars is on the top of the stack

                        // get that inline
                        let node = workspace.get_newest_opened_inline_node();

                        // write them out
                        workspace.write_out_chars_from_buffer(
                            node,
                            output_label,
                            buffer,
                            evaluated_range,
                        );
                    }
                }
            }

            // remove now written out chars from front of buffer
            buffer.dequeue_chars_from_buffer(evaluated_range);
        }
    };
    // (end of closure)

    /*
        main loop for block parser
    */

    /*
        iterate through characters visible to this instance of the parser

        ie. there may be sequences of characters in the input string that are not being shown to this instance of the inline parser
        however, the characters that are being shown to this parser instance need to be processed as if they were part of one contiguous sequence
    */
    let mut chars = input.chars();

    while let Some(input_char) = chars.next() {
        // call the step closure
        step(
            input_char.first_byte,
            InputStringRange {
                start: input_char.start,
                end: input_char.end,
            },
            &mut chars,
        );

        /*
            if fast-track processing was used, the range may have extended beyond the range of the character given to the step
            therefore, retain the index of the last character processed
            used for the flush character logic below
        */
        last_input_char_processed_end = chars.get_cursor_end();
    }

    /*
        flush a null char through the machine, to ensure that any accumulated input string characters are written out of the buffer

        (the inline parser automaton was designed in such a way to facilitate this behaviour)

        pass in an empty range at the location of one byte beyond the last character of the input given to this instance of the inline parser
        by passing an empty range, the null char won't be written out to the AST - a behaviour requried as this null char isn't in the input string, so of course shouldn't be shown in the output
    */
    step(
        b'\0',
        InputStringRange {
            start: last_input_char_processed_end,
            end: last_input_char_processed_end,
        },
        &mut chars,
    );

    /*
        if any inlines are open, the character flush above won't close them
        therefore, close any and all open inlines
    */
    workspace.close_all_open_inlines();
}
