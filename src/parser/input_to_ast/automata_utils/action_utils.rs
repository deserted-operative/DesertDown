use super::node_utils::NodeType;
use super::{
    ACTION_LOOKUP_WIDTH, BLOCK_STATE_COUNT, INLINE_STATE_COUNT, block_actions::BLOCK_ACTION_LOOKUP,
    block_actions::BLOCK_ACTIONS, inline_actions::INLINE_ACTION_LOOKUP,
    inline_actions::INLINE_ACTIONS,
};

/// Each range expression used in DesertDown can be expressed as one of these functions on n
#[derive(Clone, Copy)]
pub enum RangeExpression {
    Identity, // takes n, returns n
    Add(u8),  // takes n, adds something to it
    Sub(u8),  // takes n, subtracts something from it
    Set(u8), // takes n, returns whatever Set() is called with - eg. n = 15, Action::Set(10), returns 10
}

/// Each state can perform an action/chain of actions on the buffer once it is reached.
/// Each action consists of an expression in terms of n (the length of the buffer), for which its output label shoud be applied to
#[derive(Clone, Copy)]
pub struct Action {
    pub range_expression: RangeExpression,
    pub output_label: NodeType,
}

/// Takes an action, and a value for n (the length of the buffer), and returns a NodeType and the number of characters in the buffer it should apply to
#[inline(always)]
pub fn evaluate_action(action: Action, n: usize) -> (usize, NodeType) {
    let result = match action.range_expression {
        RangeExpression::Identity => n,
        RangeExpression::Add(value) => n + value as usize,
        RangeExpression::Sub(value) => n - value as usize,
        RangeExpression::Set(value) => value as usize,
    };

    (result, action.output_label)
}

/// Lookup to get the block associated with an action from the block parser automaton
pub static BLOCK_ACTION_TO_BLOCK_TYPE: [NodeType; BLOCK_ACTIONS.len()] = {
    // array precomputed by compiler (see: static)
    // this can be inferred at runtime, but a lookup based on the action is more efficient

    // builds an array of as many actions as there are, with the value of NoOp in the NodeType enum (so 1, as NoOp is the second entry in the enum)
    // inits to NoOp as the base case if there's no block associated with the action (eg. internal states output no chars, so have no block association)
    let mut types = [NodeType::NoOp; BLOCK_ACTIONS.len()];

    let mut index = 0;

    // for each action
    while index < BLOCK_ACTIONS.len() {
        // for this action, lookup the block type indicated by the output label of the action
        types[index] = BLOCK_ACTIONS[index].output_label.block_type();

        index += 1;
    }

    types
};

/// Lookup to get the inline structure associated with an action from the inline parser automaton
pub static INLINE_ACTION_TO_INLINE_TYPE: [NodeType; INLINE_ACTIONS.len()] = {
    // array precomputed by compiler (see: static)
    // this can be inferred at runtime, but a lookup based on the action is more efficient

    // builds an array of as many actions as there are, with the value of NoOp in the NodeType enum (so 1, as NoOp is the second entry in the enum)
    // inits to NoOp as the base case if there's no inline structure associated with the action (eg. internal states output no chars, so have no inline structure association)
    let mut types = [NodeType::NoOp; INLINE_ACTIONS.len()];

    let mut index = 0;

    // for each action
    while index < INLINE_ACTIONS.len() {
        // for this action, lookup the inline structure indicated by the output label of the action
        types[index] = INLINE_ACTIONS[index].output_label.inline_type();

        index += 1;
    }

    types
};

/// Lookup for identifying states which are eligible for fast-track processing (exactly one action, full buffer flush), and returning their action indices (plus one).
pub static ACTIONS_OF_BLOCK_FAST_TRACK_STATES: [u16; BLOCK_STATE_COUNT] = {
    // array precomputed by compiler (see: static)

    let mut fast_track_state_action_indices = [0; BLOCK_STATE_COUNT]; // array of zeros, for as many states there are
    let mut state = 0;

    while state < BLOCK_STATE_COUNT {
        let lookup = state * ACTION_LOOKUP_WIDTH;

        let first_action_index: u32 = BLOCK_ACTION_LOOKUP[lookup]; // the first entry in the block action lookup table is the index in the actions table of the first action performed by this state
        let action_count = BLOCK_ACTION_LOOKUP[lookup + 1]; // the second entry is the number of actions the state performs - one normally, several if its an action chain

        // check if the state performs exactly one action - ie. not an action chain
        if action_count == 1 {
            // state has exactly one action

            // evaluate the index of the first action as a usize so it can be used for array lookup
            let index = first_action_index as usize;

            // check if the state flushes the buffer - ie. writes out n chars from the buffer
            if matches!(
                BLOCK_ACTIONS[index].range_expression,
                RangeExpression::Identity
            ) {
                /*
                    state performs one action, that action fully flushes the buffer
                    eligible for fast-track processing

                    therefore change array value from 0 to the index of the first action (plus one)
                    the plus one is so that an action with index 0 can still be included, rather than getting caught up with the rest of the array of zeros
                */

                // `as u16` binds to index, so expression is: (index as u16) + 1
                fast_track_state_action_indices[state] = index as u16 + 1;
            }
        }

        state += 1;
    }

    fast_track_state_action_indices
};

/// Lookup for identifying states which are eligible for fast-track processing (exactly one action, full buffer flush), and returning their action indices (plus one).
pub static ACTIONS_OF_INLINE_FAST_TRACK_STATES: [u16; INLINE_STATE_COUNT] = {
    // array precomputed by compiler (see: static)

    let mut fast_track_state_action_indices = [0; INLINE_STATE_COUNT]; // array of zeros, for as many states there are
    let mut state = 0;

    while state < INLINE_STATE_COUNT {
        let lookup = state * ACTION_LOOKUP_WIDTH;

        let first_action_index: u32 = INLINE_ACTION_LOOKUP[lookup]; // the first entry in the inline action lookup table is the index in the actions table of the first action performed by this state
        let action_count = INLINE_ACTION_LOOKUP[lookup + 1]; // the second entry is the number of actions the state performs - one normally, several if its an action chain

        // check if the state performs exactly one action - ie. not an action chain
        if action_count == 1 {
            // state has exactly one action

            // evaluate the index of the first action as a usize so it can be used for array lookup
            let index = first_action_index as usize;

            // check if the state flushes the buffer - ie. writes out n chars from the buffer
            if matches!(
                INLINE_ACTIONS[index].range_expression,
                RangeExpression::Identity
            ) {
                /*
                    state performs one action, that action fully flushes the buffer
                    eligible for fast-track processing

                    therefore change array value from 0 to the index of the first action (plus one)
                    the plus one is so that an action with index 0 can still be included, rather than getting caught up with the rest of the array of zeros
                */

                // `as u16` binds to index, so expression is: (index as u16) + 1
                fast_track_state_action_indices[state] = index as u16 + 1;
            }
        }

        state += 1;
    }

    fast_track_state_action_indices
};
