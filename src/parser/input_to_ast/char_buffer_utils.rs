use super::node_utils::NodeType;
use super::{inline_parser_utils::OpenInline, input_string_utils::InputStringRange};

/// The buffer is the space in which input characters are accumulated, and then acted upon by states performing their action(s).
/// Note that for fast-track processing, the buffer is avoided entirely
pub(crate) struct CharBuffer {
    // for the normal buffer process, one range captures exactly one input characters
    ranges: Vec<InputStringRange>,

    // index of the character in the ranges which is the first char in the buffer
    head: usize,

    /*
        when adding exactly one character to the buffer, if the buffer is empty, it is stored here
        this avoids unnecessary overhead from using the ranges array above

        when this is being used, ranges will be empty

        note that if the buffer started at a length above one, but has been worked down to length 1, the range will stay in the array, rather than be moved
    */
    single: InputStringRange,

    /*
        number of chars in the buffer
        stored explicitly to abstract whether the range is stored as a single, or in the ranges array
    */
    buffer_len: usize,

    // set to false when some of the buffered characters are non-contiguous
    // make public (fields private by default) to permit expressions: if buffer.contiguous
    pub(crate) contiguous: bool,
}

/*
    sets a limit of 4096 dead entries in the array over which the head can increment before they are removed from the buffer
    these can accumulate when the buffer isn't fully flushed
    however, with the implementation strategy used for both the block and inline automata, to encounter anything close to 4096 characters without a full buffer flush is extremely unlikely
*/
const CHAR_BUFFER_SIZE_LIMIT: usize = 4096;

// implemented methods for CharBuffer struct:
impl CharBuffer {
    // constructor
    fn new() -> Self {
        CharBuffer {
            ranges: Vec::new(),
            head: 0,
            single: InputStringRange { start: 0, end: 0 },
            buffer_len: 0,
            contiguous: true,
        }
    }

    #[inline(always)]
    /// Takes a CharBuffer, and returns whether there is one character in the [`single`] attribute rather than in the ranges array
    fn is_single(&self) -> bool {
        // recall that when `single` is used, the ranges array is empty
        self.buffer_len == 1 && self.ranges.is_empty()
    }

    #[inline(always)]
    /// Takes a CharBuffer, and returns how many characters have been accumulated within it
    pub(crate) fn get_len(&self) -> usize {
        self.buffer_len
    }

    #[inline(always)]
    /// Takes a CharBuffer, returns the current state of accumulated characters as a slice
    pub(crate) fn get_buffer_as_slice(&self) -> &[InputStringRange] {
        if self.is_single() {
            // a slice of exactly one
            // note that it's over a field of the struct rather than over the heap
            std::slice::from_ref(&self.single)
        } else {
            // debug check that the number of characters (from, and) after the head match the number in the struct member
            debug_assert_eq!(self.ranges.len() - self.head, self.buffer_len);

            &self.ranges[self.head..]
        }
    }

    #[inline(always)]
    /// Takes an [`InputStringRange`] over one character, and appends it to the back of the buffer.
    /// For clarity, the buffer is a queue
    pub(crate) fn push_char_to_buffer(&mut self, char_range: InputStringRange) {
        // if the buffer is empty, and one character needs appending => simple case: use the `single` struct member, and set the length
        if self.buffer_len == 0 {
            self.single = char_range;
            self.buffer_len = 1;

            return;
        }

        // buffer not empty, appending one character

        // get the byte offset of the next byte after the last char in the buffer
        // note that if there is no next byte, then the index will equal input_string.len()
        let previous_end = if self.is_single() {
            self.single.end
        } else {
            self.ranges[self.ranges.len() - 1].end
        };

        // if the end found doesn't point to the byte of the start of the range being pushed to the buffer, then the last range and this range are non-contiguous
        // ie. an excluded zone in the input string sits between them
        if previous_end != char_range.start {
            self.contiguous = false;
        }

        // if there is one character in single, and another is being pushed, first put the char in single into the array, before putting the char requested to be pushed there, so that the char in single isn't lost
        if self.is_single() {
            self.ranges.push(self.single);
        }

        // push the char to the array
        self.ranges.push(char_range);
        self.buffer_len += 1;
    }

    #[inline(always)]
    /// Takes a CharBuffer, and a number of characters.
    /// Removes that many characters from the front of the buffer.
    /// Used when an action is executed on the buffer
    pub(crate) fn dequeue_chars_from_buffer(&mut self, count: usize) {
        /*
            debug check that no more chars than the buffer length have been requested to be written out
            note though that below, the implementation simply resets the buffer if more than are present are requested to be written out
            this is to avoid a panic in prod
        */
        debug_assert!(count <= self.buffer_len);

        // reset the buffer if nothing will remain after the write out
        if count >= self.buffer_len {
            self.reset();
            return;
        }

        // some chars will remain in the buffer after dequeuing

        // increment pointers over now dead ranges
        self.head += count;
        self.buffer_len -= count;

        // if the buffer had non-contiguous ranges, now some have been written out, contiguity may be restored, so need to check
        if !self.contiguous {
            self.contiguous = self
                .get_buffer_as_slice()
                .windows(2)
                .all(|pair| pair[0].end == pair[1].start);
        }

        // dequeuing chars means there are now some dead ranges in the ranges array of the buffer
        // check that the number of dead chars hasn't exceeded the limit, where if it has, remove the dead elements from the ranges array, and reset the head to zero
        if self.head >= CHAR_BUFFER_SIZE_LIMIT {
            self.ranges.drain(..self.head);
            self.head = 0;
        }
    }

    #[inline]
    /// Empty the character buffer whilst keeping it allocated
    pub(crate) fn reset(&mut self) {
        // clear() has cost, so only clear if not already empty
        if !self.ranges.is_empty() {
            self.ranges.clear();
        }

        self.head = 0;
        self.buffer_len = 0;
        self.contiguous = true;
    }
}

/// The structures needed for a parser to be run on some input
pub struct ParserStructures {
    pub(crate) buffer: CharBuffer,

    // stack for open inline structures
    pub(crate) stack: Vec<OpenInline>,

    // array of inline structures which an overlapping closer has cut short, and need to continue being worked in the next parser iteration
    pub(crate) requires_reopening: Vec<(NodeType, u8)>,
}

impl ParserStructures {
    // constructor
    pub fn new() -> Self {
        ParserStructures {
            buffer: CharBuffer::new(),
            stack: Vec::new(),
            requires_reopening: Vec::new(),
        }
    }
}

impl Default for ParserStructures {
    fn default() -> Self {
        Self::new()
    }
}
