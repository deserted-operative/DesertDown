/// Stores the index of the 129th slot in the automata transition arrays. The 129th slot is for wildcard characters, for which every concrete state must have a transition defined for
pub const WILDCARD_COLUMN_INDEX: usize = 128;

#[inline(always)]
/// Takes the first byte of a character, and returns the column index which should be used on the relevant row of the applicable automaton transition table to get the state reached by consuming the character
pub(crate) fn get_char_column(first_byte_of_char: u8) -> usize {
    if first_byte_of_char < 0x80 {
        /*
            0x80 is 128
            thus, if first byte is less less than 128, then it is the only byte, and the char is ASCII
            hence return the index, as the first 128 slots of transition lookup table rows are for the ASCII char codes
        */
        first_byte_of_char as usize
    } else {
        // not ASCII char, return wildcard index (ie. 129th slot)
        WILDCARD_COLUMN_INDEX
    }
}
