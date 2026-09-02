use super::{
    ACTION_LOOKUP_WIDTH, BLOCK_STATE_COUNT, BLOCK_TRANSITIONS, SYMBOL_COUNT,
    action_utils::{
        ACTIONS_OF_BLOCK_FAST_TRACK_STATES, BLOCK_ACTION_TO_BLOCK_TYPE, evaluate_action,
    },
    ast_utils::{AST, NodeId},
    block_actions::{BLOCK_ACTION_LOOKUP, BLOCK_ACTIONS},
    block_parser_utils::BlockParserWorkspace,
    char_buffer_utils::ParserStructures,
    input_string_utils::{InputCharsForParsing, InputStringRange, InputStringView},
    parser_utils::Job,
    transition_utils::get_char_column,
};
use std::collections::VecDeque;

// index of the start state in the block transition array
// should double check the block_transitions.rs to ensure that this holds true
pub const BLOCK_INITIAL_STATE: usize = 0;

pub fn block_parser(
    input: InputStringView<'_, '_>,
    ast: &mut AST,
    root: NodeId,
    job_queue: &mut VecDeque<Job>,
    parser_structures: &mut ParserStructures,
) {
    let (input_start_index, _) = input.get_view_bounds(); // destructured tuple, _ discards the end offset, as not needed here

    let mut state = BLOCK_INITIAL_STATE;

    // contains the source range of every character currently held in the parser's buffer
    let buffer = &mut parser_structures.buffer; // borrow the buffer

    // reset the buffer, as it's shared between runs of the block parser
    // is ok because of single-threaded implementation
    buffer.reset();

    let mut workspace =
        BlockParserWorkspace::new(ast, job_queue, input.get_input_string_bytes(), root); // move the borrowed AST and job deque into the workspace, where now they cannot be used here again directly

    // track where last input character ends, so the flushed newline can be given an empty span at the right place
    // inits to input start, so an empty view yields start == end == input_start
    let mut last_input_char_processed_end = input_start_index;

    /*
        closure for running one character through the automaton, and performing the action(s) of the state reached by consuming the character

        note that a closure is a function that also carries some of the surrounding scope with it
        here: state, buffer, and workspace are captured by the closure, without explicitly passing them in as arguments
    */
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
        debug_assert!(state < BLOCK_STATE_COUNT);
        debug_assert!(column < SYMBOL_COUNT);

        // lookup what state is reached by consuming this character from the current state
        // then update the current state
        state = BLOCK_TRANSITIONS[state * SYMBOL_COUNT + column] as usize;

        /*
            check for, and use if possible, fast-track processing
            if the reached state performs exactly one action, and that action fully flushes the buffer, it is eligible for fast-track processing

            under such a strategy, the parser walks forward over contiguous characters that loop back onto this same automaton state, where they can be efficiently processed without using the full buffer procedure

            because this applies to the majority of characters in an input string (ie. most of the characters that aren't meta characters), the performance improvement gained by this strategy is significant
        */

        // get the index of the action in the actions array of the action performed by this state
        // the lookup table has precomputed some checks, so 0 is returned if the state is ineligible for fast-track processing, or the index of the action + 1 if it is eligible
        let fast_track_action = ACTIONS_OF_BLOCK_FAST_TRACK_STATES[state] as usize;

        // buffer must be empty to qualify for fast-track processing - can't fast-track if there's characters pending write-out
        if buffer.get_len() == 0 && fast_track_action != 0 {
            // fast-track processing feasible

            let action_index = fast_track_action - 1; // need to take the one added off to get the valid index back (see ACTIONS_OF_BLOCK_FAST_TRACK_STATES implementation)
            let mut range = range;
            let row_index = state * SYMBOL_COUNT;

            /*
                get the range of chars fast-tracked over, or None if nothing could be walked over

                the logic for fast-tracking over self loops advances the cursor property of the InputCharsForParsing, which is hooked into the custom Iterator implementation
                thus, don't need to worry about the next char for processing being wrong, as a result of dealing with more than one character in this closure
            */
            if let Some(range_fast_tracked_over) = chars.fast_track_over_self_loops(
                &BLOCK_TRANSITIONS[row_index..row_index + SYMBOL_COUNT],
                state as super::StateId,
            ) {
                // debug check that the fast track range starts after this step's character
                debug_assert_eq!(range.end, range_fast_tracked_over.start);

                // extend the current step's range to now include the fast track characters
                range.end = range_fast_tracked_over.end;
            }

            // don't need to use split runs because confining fast track processing to one span keeps chars contiguous
            workspace.write_out_contiguous_chars(
                BLOCK_ACTIONS[action_index].output_label,
                BLOCK_ACTION_TO_BLOCK_TYPE[action_index],
                range,
            );

            return;
        }

        /*
            if here, need to use the full buffer procedure
        */

        // push the char to the buffer (range captures one character)
        buffer.push_char_to_buffer(range);

        // lookup the first action associated with the reached state, and how many actions the state performs (will be at least one)
        let first_action_index = BLOCK_ACTION_LOOKUP[state * ACTION_LOOKUP_WIDTH] as usize;
        let action_count = BLOCK_ACTION_LOOKUP[state * ACTION_LOOKUP_WIDTH + 1] as usize;

        // perform the reached state's action(s)
        for offset in 0..action_count {
            // get the index in the actions array of the action that needs to be performed
            let action_index = first_action_index + offset;

            // evaluate action on the buffer
            let (mut evaluated_range, output_label) =
                evaluate_action(BLOCK_ACTIONS[action_index], buffer.get_len());

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

            // use precomputed lookup table based on action indices to efficiently determine which block type this output label belongs to
            let block_type_of_char = BLOCK_ACTION_TO_BLOCK_TYPE[action_index];

            // handle exclusion zones in the ranges if they are present
            if buffer.contiguous {
                // contiguous case is the simple case

                // get the chars in the buffer that are to be written out
                // returns a slice where each char is a range
                let chars_to_write_out = buffer.get_buffer_as_slice();

                // package as one range
                let range_to_write_out = InputStringRange {
                    start: chars_to_write_out[0].start,
                    end: chars_to_write_out[evaluated_range - 1].end,
                };

                // write them to the AST
                workspace.write_out_contiguous_chars(
                    output_label,
                    block_type_of_char,
                    range_to_write_out,
                );
            } else {
                // non-contiguous case

                // use the non-contiguous write out logic, passing in as many chars (each as its own range in the slice) as are to be written out from the buffer
                workspace.write_out_non_contiguous_chars(
                    output_label,
                    block_type_of_char,
                    &buffer.get_buffer_as_slice()[..evaluated_range],
                );
            }

            // remove the written out characters from the front of the buffer
            buffer.dequeue_chars_from_buffer(evaluated_range);
        }
    };
    // (end of closure)

    /*
        main loop for block parser
    */

    /*
        iterate through characters visible to this instance of the parser

        ie. there may be sequences of characters in the input string that are not being shown to this instance of the block parser
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
        flush a newline through the machine, to ensure that any accumulated input string characters are written out of the buffer

        (the block parser automaton was designed in such a way to facilitate this behaviour)

        pass in an empty range at the location of one byte beyond the last character of the input given to this instance of the block parser
        by passing an empty range, the newline won't be written out to the AST - a behaviour requried as this newline char isn't in the input string, so of course shouldn't be shown in the output
    */
    step(
        b'\n',
        InputStringRange {
            start: last_input_char_processed_end,
            end: last_input_char_processed_end,
        },
        &mut chars,
    );

    /*
        if a block is open, the character flush above won't close it
        therefore, close block if open
    */
    workspace.close_open_block();
}
