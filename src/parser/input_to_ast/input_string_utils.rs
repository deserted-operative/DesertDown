use super::transition_utils::WILDCARD_COLUMN_INDEX;

#[derive(Debug, Clone, Copy)]
/// Represents a contiguous section of the input string.
/// Uses byte offsets in the input string rather than character indices, to account for UTF-8 chars being able to span multiple bytes.
/// Each byte offset points to the first byte of a char.
pub struct InputStringRange {
    pub start: u32,
    pub end: u32, // exclusive, not inclusive - therefore is the index of the next byte after the last char captured. Note that if there is no next byte, then the index will equal input_string.len()
}

#[derive(Debug, Clone, Copy)]
/// Collates multiple disjoint and potentially separated ranges over the original input string as one logically sequential input.
/// Necessary for parsing blocks that reside within blocks, where some sections of the input string may belong to an outer block, and need hiding from an inner block
pub struct InputStringView<'input_string, 'ranges> {
    /*
        InputStringView takes two lifetime parameters: 'input_string and 'ranges
        The compiler will ensure that the InputStringView cannot outlive the borrowed input string or borrowed input string ranges
        This is useful, as the struct doesn't make any sense if it outlives either of these things
    */
    input_string: &'input_string str,
    ranges: &'ranges [InputStringRange],
}

/*
    implemented methods for InputStringView
    on impl<...> the two lifetimes are declared
    they are used after the InputStringView type name
*/
impl<'input_string, 'ranges> InputStringView<'input_string, 'ranges> {
    /// Constructor for an InputStringView, takes a borrowed input string, and borrowed array of input string ranges within that input string
    pub fn new(input_string: &'input_string str, ranges: &'ranges [InputStringRange]) -> Self {
        // this constructor is the only way to build an InputView from outside the module, since both fields of the InputView struct are private

        // debug check: ensures that every range is inside the input string
        debug_assert!(ranges.iter().all(|range| {
            range.start <= range.end
                && range.end as usize <= input_string.len()
                && input_string.is_char_boundary(range.start as usize)
                && input_string.is_char_boundary(range.end as usize)
        }));

        /*
            input string ranges should occur in input order, and must not overlap

            windows(2) evaluates the ranges in the array pairwise - eg. [a,b,c,d].windows(2).all iterates over [a,b], [b,c], [c,d]
            the pair[0] and pair[1] comparison ensures that the ranges are sorted and non-overlapping
        */
        debug_assert!(ranges.windows(2).all(|pair| pair[0].end <= pair[1].start));

        // return InputStringView, with ownership, to the caller
        Self {
            input_string,
            ranges,
        }
    }

    #[inline]
    /// Takes an InputStringView, by value, and returns the byte indices of the start of the first range, and the end of the last range in the view.
    /// This may span over ranges in the input string that are not captured by the ranges in the view.
    /// ie. Exclusion zones for characters belonging to an outer block, whilst the view is concerned with an inner block.
    pub fn get_view_bounds(self) -> (u32, u32) {
        match (self.ranges.first(), self.ranges.last()) {
            (Some(first), Some(last)) => (first.start, last.end),
            _ => (0, 0), // not reached in practice, but keep the compiler happy
        }
    }

    #[inline]
    /// Takes an InputStringView, and returns a borrowed slice of the bytes of the input string.
    /// Required for inspecting the input string to determine newline presence with respect to closing Table Row Blocks
    pub fn get_input_string_bytes(self) -> &'input_string [u8] {
        self.input_string.as_bytes()
    }

    #[inline]
    /// Takes an InputStringView, and returns an InputCharsForParsing, which has Iterator implemented on it.
    /// This allows the block/inline parsers to iterate over the characters in the view for parsing (skipping over excluded characters)
    pub fn chars(self) -> InputCharsForParsing<'input_string, 'ranges> {
        // the returned InputCharsForParsing also depends on the lifetime variables because the InputStringView argument does
        InputCharsForParsing {
            input_string_bytes: self.input_string.as_bytes(), // use as_bytes to give InputCharsForParsing the borrowed source string as bytes, as the iterator needs raw bytes
            ranges: self.ranges.iter(),                       // returns an iterator over the slice
            cursor: 0, // cursor and range_end are conceptually intertwined. On the first next() call (recall, using an iterator), cursor < range_end is false (because 0 < 0 is false), and the control flow determines that the current char range is depleted, and looks for the next one. This also handles edge cases of no ranges at all, or an empty range
            range_end: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// One character in an InputStringView
pub struct InputChar {
    // char location in input string
    pub start: u32,
    pub end: u32,

    // the character's first byte, which is the whole character when ASCII and otherwise only indicates that it is not ASCII
    pub first_byte: u8,
}

/// Iterator for characters in an InputStringView
pub struct InputCharsForParsing<'input_string, 'ranges> {
    input_string_bytes: &'input_string [u8],
    ranges: std::slice::Iter<'ranges, InputStringRange>, // slice::Iter is internaly a start pointer and an end pointer, next() is a compare and a pointer increment with no bounds check

    // where in the input string the next character begins, and where the range being consumed ends
    cursor: u32,    // ranges.next() gives the current range, and cursor is incremented
    range_end: u32, // ranges.next() gives the current range, and range_end is incremented
}

#[inline(always)]
/// Inspects the first byte of a character, and determines how many bytes the character occupies.
/// Relies on input string being guaranteed valid UTF-8, and that it's only ever called on character boundaries
const fn get_utf8_char_len(first_byte: u8) -> u32 {
    match first_byte {
        0x00..=0x7F => 1, // if the leading byte starts with a b0, it's a one byte char - yes, ASCII
        0xC0..=0xDF => 2, // if the leading byte starts with b110, it's a two byte char - not ASCII
        0xE0..=0xEF => 3, // if the leading byte starts with b1110, it's a three byte char - not ASCII
        _ => 4,           // not ASCII
    }
}

// implement the Iterator trait for InputCharsForParsing
// the '_ are anonymous lifetime parammeters - ie. the compiler picks names for them itself, which is fine, because they don't appear in this impl block or in Item
impl Iterator for InputCharsForParsing<'_, '_> {
    /*
        an ASSOCIATED TYPE
        ie. a type that the implementor must supply, but which the caller does not choose
        is used for the return type
    */
    type Item = InputChar;

    /*
        .next() method implementation
        takes a mutable borrow of an InputCharsForParsing
        returns None or Some Item
    */
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // try to take another character from the input range being consumed
            if self.cursor < self.range_end {
                let start = self.cursor;

                let byte = self.input_string_bytes[start as usize]; // need to cast start to usize for valid array lookup

                let end = start + get_utf8_char_len(byte); // work only on char boundaries wrt. bytes

                // a range ends on a character boundary, so a character which begins inside a range also ends inside it
                debug_assert!(end <= self.range_end);

                // update/move the cursor
                self.cursor = end;

                // return the values required to reconstruct the input character
                return Some(InputChar {
                    start,
                    end,
                    first_byte: byte,
                });
            }

            /*
                current range is exhausted, move onto the next one
                ? is postfix, and binds tighter than the * dereference
                so this reads as *(self.ranges.next()?)
                ? works to obtain the value if the expression it applies to has Some(value). If the expression is None, it returns out of this function (ie. `next`) with return value None
                * copies the range out, since InputStringRange derives the Copy trait
            */
            let range = *self.ranges.next()?; // works because next return type is Option. Conceptually, just a recursive call with a base case provided by ?

            // update the cursor and range end
            self.cursor = range.start;
            self.range_end = range.end;
        }
    }
}

impl InputCharsForParsing<'_, '_> {
    #[inline(always)]
    /// Takes a borrowed InputCharsForParsing, and returns the index of the last char the iterator provided.
    /// Required because fast-track processing may extend beyond the last character the loop handed
    pub(crate) fn get_cursor_end(&self) -> u32 {
        self.cursor
    }

    #[inline(always)]
    /// Takes a mutable borrowed InputCharsForParsing, a borrowed slice of a transition lookup table (ie. a whole row, which is every transition from one state), and the current state.
    /// Collects a sequence of characters forward from the current character that, when consumed, result in reaching the same state as the current state (ie. a self loop in the automaton).
    /// This is required to implement fast-track processing.
    /// Returns None if no characters are found. Returns Some InputStringRange over the input string if suitable characters are found
    pub(crate) fn fast_track_over_self_loops(
        &mut self,
        transition_row: &[super::StateId],
        state: super::StateId,
    ) -> Option<InputStringRange> {
        // load in current character as the starting point
        let start = self.cursor;

        // whether wildcard transitions from the current state loop back onto the same state can be tested once for the whole loop
        let does_wildcard_transition_loop = transition_row[WILDCARD_COLUMN_INDEX] == state;

        // while in the current range of the InputCharsForParsing:
        while self.cursor < self.range_end {
            // get the first byte of the character under the cursor
            let first_byte = self.input_string_bytes[self.cursor as usize];

            // calculate the number of bytes the current character takes
            // the assignment to char_width captures the result of the if/else block
            let char_width = if first_byte < 0x80 {
                // is ASCII character

                // if consuming the character doesn't lead back to the same state, can break out of the loop looking for a sequence of chars that loop back to the current state
                if transition_row[first_byte as usize] != state {
                    break;
                }

                // otherwise, consuming the ASCII character does indeed lead back to the same state, thus return char width, which is 1 byte
                1
            } else {
                // not an ASCII character

                // again, if consuming the character doesn't lead back to the same state, can break out of the loop looking for a sequence of chars that loop back to the current state
                if !does_wildcard_transition_loop {
                    break;
                }

                // otherwise, consuming the character does indeed lead back to the same state, thus return char width
                // call the method to inspect the first byte of the char to determine its width
                get_utf8_char_len(first_byte)
            };

            // debug check for that the found characters haven't extended beyond the current range
            debug_assert!(self.cursor + char_width <= self.range_end);

            // incremenet the cursor - using char width to ensure cursor always stays on byte boundaries that mark the start of characters
            self.cursor += char_width;
        }

        // then_some() - if self.cursor == start, then None is returned, else Some and then an InputStringRange is returned
        (self.cursor != start).then_some(InputStringRange {
            start,
            end: self.cursor,
        })
    }
}

#[inline]
/// Takes a borrowed slice of input ranges, and - conceptually - merges adjacent ranges into larger ranges.
/// eg. [0..1, 1..2, 4..5, 5..6] becomes [0..2, 4..6].
/// Implementation does this lazily, returning an iterator that evaluates input ranges when elements are requested
pub(crate) fn merge_contiguous_ranges(
    ranges: &[InputStringRange],
) -> impl Iterator<Item = InputStringRange> + '_ {
    /*
        `'_` is an anonymous lifetime bound inferred from the input borrow
        the returned iterator owns `rest`, but `rest` borrows the slice `ranges`
        therefore, the returned iterator cannot outlive `ranges`
    */

    // accumulator for characters - ie. "character run"
    // inits to None to signify "no run in progress"
    let mut run: Option<InputStringRange> = None;

    // cursor over the input string ranges
    // separate to the above run so that the closure can advance input without producing output
    let mut rest = ranges.iter();

    /*
        from_fn turns a FnMut() -> Option<T> into an iterator, where each call to next() invokes the closure once
        note that the closure is marked by the two pipes - where there's nothing between the pipes means its a closure with no arguments

        move transfers run and rest into the closure, where they are part of the returned iterator's state
        this allows them to persist between calls of next()
    */
    std::iter::from_fn(move || {
        // loop to let several contiguous input ranges collapse into one input range
        loop {
            // rest.next() gives Option<&InputStringRange>
            // the &next pattern in Some copies out the value, thus InputStringRange must implement Copy
            let Some(&next) = rest.next() else {
                // input is exhausted, so take() returns the final accumulated range - if any - and leaves `run` as None so subsequent calls return None
                return run.take();
            };

            match run {
                // the previous character ends exactly where this one begins, so the range continues
                Some(current) if next.start == current.end => {
                    run = Some(InputStringRange {
                        start: current.start,
                        end: next.end,
                    });
                }

                // an excluded region sits between the two characters, so this range ends, and the next starts at this character
                // (this is true because ranges are ordered and non-overlapping)
                Some(current) => {
                    run = Some(next);

                    return Some(current);
                }

                None => run = Some(next),
            }
        }
    })
}
