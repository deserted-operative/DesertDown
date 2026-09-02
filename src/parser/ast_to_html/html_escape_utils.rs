// array of escapes which chars requiring escaping should be replaced with
// first entry is empty string so that index 0 into this array can be used to indicate that a byte does not need escaping
const ESCAPES: [&[u8]; 5] = [b"", b"&amp;", b"&lt;", b"&gt;", b"&quot;"];

// constant array of 256 so that every possible byte has a lookup in the table
// byte goes in, index into the ESCAPES array yields what that byte should be replaced with (unless its the empty string, in which case it doesn't need replacing)
const BYTE_TO_ESCAPES_INDEX: [u8; 256] = {
    // init the lookup array to zeros - defaulting to byte doesn't need escaping
    let mut lookup_array = [0u8; 256];

    // set the entries for chars that require escaping with the index of the corresponding escape in the above ESCAPES array
    lookup_array[b'&' as usize] = 1;
    lookup_array[b'<' as usize] = 2;
    lookup_array[b'>' as usize] = 3;
    lookup_array[b'"' as usize] = 4;

    // return table
    lookup_array
};

// how many bytes are escaped in one go
const ESCAPE_BATCH_SIZE: usize = 16;

#[inline(always)]
/// Takes a byte, returns 1 if it requires escaping, 0 otherwise
const fn does_byte_require_escaping(byte: u8) -> u8 {
    // bool cast to u8 maps true to 1, 0 to false
    ((byte == b'&') as u8)
        | ((byte == b'<') as u8)
        | ((byte == b'>') as u8)
        | ((byte == b'"') as u8)
}

/// Takes an output array, and a slice of bytes, and writes those bytes to the array, escaping them where necessary for HTML output
pub(super) fn write_out_escaped_bytes(output_array: &mut Vec<u8>, bytes: &[u8]) {
    // keep track of where next write out needs to start in the array of bytes
    let mut start = 0;

    // track how many bytes have been checked
    let mut cursor = 0;

    // check 16 bytes at a time
    // can work without consideration for where the char boundaries are wrt. bytes as the chars to be escaped are ASCII, and thus bytes of a non-ASCII UTF-8 char won't collide with the ASCII bytes being looked for
    while let Some(batch_of_bytes) = bytes[cursor..].first_chunk::<ESCAPE_BATCH_SIZE>() {
        // if all bytes returned 0 in checking if they require escaping, then none require escaping
        if batch_of_bytes
            .iter()
            .fold(0, |found, &byte| found | does_byte_require_escaping(byte))
            == 0
        {
            // no bytes require escaping, increment cursor
            cursor += ESCAPE_BATCH_SIZE;

            continue;
        }

        /*
            here, at least one byte in the batch requires escaping
        */

        // process each byte in the batch in turn
        for index in 0..ESCAPE_BATCH_SIZE {
            // get the index of the replacement in the array of escapes for this byte
            // 0 if no escaping required
            let escape_array_index = BYTE_TO_ESCAPES_INDEX[batch_of_bytes[index] as usize];

            // non-zero means => escaping required
            if escape_array_index != 0 {
                // write out pending bytes before the one that requires escaping
                output_array.extend_from_slice(&bytes[start..cursor + index]);

                // write out the escape for the byte
                output_array.extend_from_slice(ESCAPES[escape_array_index as usize]);

                // increment start to point after the now written out byte
                start = cursor + index + 1;
            }
        }

        // increment cursor
        cursor += ESCAPE_BATCH_SIZE;
    }

    /*
        process bytes that couldn't fill up a batch of 16 (if any)
    */

    while cursor < bytes.len() {
        // get the index of the replacement in the array of escapes for this byte
        // 0 if no escaping required
        let escape_array_index = BYTE_TO_ESCAPES_INDEX[bytes[cursor] as usize];

        // non-zero means => escaping required
        if escape_array_index != 0 {
            // write out pending bytes before the one that requires escaping
            output_array.extend_from_slice(&bytes[start..cursor]);

            // write out the escape for the byte
            output_array.extend_from_slice(ESCAPES[escape_array_index as usize]);

            // increment start to point after the now written out byte
            start = cursor + 1;
        }

        // increment cursor
        cursor += 1;
    }

    // if start hasn't been incremented to the end, then there are still bytes to write out, so write them out if they exist, or will (without issue) call on an empty range if not
    output_array.extend_from_slice(&bytes[start..]);
}
