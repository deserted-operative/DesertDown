use super::ast_to_html::HtmlRenderer;
use super::html_escape_utils::write_out_escaped_bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkPermissions {
    // if blocked, links and embedded links won't be interactive in the HTML output, and the CSP will become more strict
    #[default]
    Blocked,
    Allowed,
}

// only http and https allowed for web links, otherwise taken as a local file reference
const ACCEPTED_SCHEMES: [&[u8]; 2] = [b"http", b"https"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Options available for how the <content> in either `[[<content>]]` or `![[<content>]]` is handled
enum TargetType {
    // local file in same or descendant directory as output HTML
    File,

    // an http or https URL - a link, and never an embed
    // http or https URL, only shown as link, not embedded
    Web,

    // link/embedded link content that won't be linked to or embedded
    Rejected,
}

/// What a target's leading bytes turn out to be, when read the way a URL parser reads them
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scheme {
    // no scheme at all, so the target is a relative reference
    None,

    // a scheme, and one of LINKABLE_SCHEMES
    Linkable,

    // a scheme, and not one of those
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Supported types of file for embedding
enum EmbedTypes {
    Image,
    Pdf,
    Audio,
    Video,
    None, // if file extension doesn't suggest file is one of the above types, then it's shown as a link rather than an embedded link
}

const ACCEPTED_IMAGE_EXTENSIONS: [&[u8]; 9] = [
    b"png", b"jpg", b"jpeg", b"gif", b"webp", b"avif", b"bmp", b"ico", b"svg",
];
const ACCEPTED_VIDEO_EXTENSIONS: [&[u8]; 4] = [b"mp4", b"webm", b"ogv", b"m4v"];
const ACCEPTED_AUDIO_EXTENSIONS: [&[u8]; 8] = [
    b"mp3", b"wav", b"ogg", b"oga", b"m4a", b"aac", b"flac", b"opus",
];
const ACCEPTED_PDF_EXTENSIONS: [&[u8]; 1] = [b"pdf"];

const CURRENT_DIRECTORY_PATH_SEGMENT: [u8; 2] = *b"./";

/// Remove bytes from link content that are leading and trailing spaces and control characters
fn trim_link_content(target: &[u8]) -> &[u8] {
    /*
        0x20 is ASCII code for space
        all ASCII codes below 0x20 are control chars

        therefore, trim spaces and control chars from the input
    */

    // init start and end
    let mut start = 0;
    let mut end = target.len();

    // trim front
    while start < end && target[start] <= 0x20 {
        start += 1;
    }

    // trim end
    while end > start && target[end - 1] <= 0x20 {
        end -= 1;
    }

    // return (borrowed) slice
    &target[start..end]
}

/// Maps link content to whether it should be interpreted as a web link, file, or rejected
fn classify_link_content(link_content: &[u8]) -> TargetType {
    // trim leading and trailing spaces and control chars
    let trimmed_link_content = trim_link_content(link_content);

    // if trimmed after empty, not a valid link
    if trimmed_link_content.is_empty() {
        return TargetType::Rejected;
    }

    // identify if a scheme is specified in the trimmed link content, and map it to the corresponding link content target type
    match identify_scheme(trimmed_link_content) {
        Scheme::Linkable => TargetType::Web,

        // the trimmed link content contained a colon, meaning it has been identified as specifying a scheme
        // if the scheme found isn't in the valid list (ie. https:, http: are allowed, javascript: is not), it gets rejected
        Scheme::Other => TargetType::Rejected,

        // a file - check that it's in a valid directory (same or descendant directory to that HTML is in)
        Scheme::None => {
            if is_in_same_or_descendant_directory(trimmed_link_content) {
                TargetType::File
            } else {
                TargetType::Rejected
            }
        }
    }
}

/// Takes trimmed link content, and returns whether it is a linkable scheme (ie. http: or https:), or another scheme (eg. javascript:), or neither
fn identify_scheme(trimmed_link_content: &[u8]) -> Scheme {
    // get the bytes of the link content that indicate a scheme (denoted by <chars>:) - None if no scheme can be found
    let Some(extracted_scheme_bytes) = extract_scheme_bytes(trimmed_link_content) else {
        // no scheme found
        return Scheme::None;
    };

    // scheme found, is it in the list of allowed schemes?
    if ACCEPTED_SCHEMES
        .iter()
        .any(|allowed| does_input_match_scheme(extracted_scheme_bytes, allowed))
    {
        // linkable scheme - checks first

        /*
            backslashes, control characters, and DEL chars - anywhere in the string - all lead to the link being rejected
        */
        for &byte in trimmed_link_content {
            if byte == b'\\' || byte < 0x20 || byte == 0x7F {
                return Scheme::Other; // Other leads to the link being rejected
            }
        }

        // return that scheme is linkable
        Scheme::Linkable
    } else {
        // catches things like `javascript:`
        Scheme::Other
    }
}

/// Takes trimmed link content, and returns the bytes before a `:` that end a scheme, or None if no potential scheme is found
fn extract_scheme_bytes(trimmed_link_content: &[u8]) -> Option<&[u8]> {
    /*
        extract bytes before first colon
    */

    let mut cursor = 0;
    let start = cursor;

    while cursor < trimmed_link_content.len() {
        let byte = trimmed_link_content[cursor];

        if byte == b':' {
            return Some(&trimmed_link_content[start..cursor]);
        }

        cursor += 1;
    }

    // no colon found, so no scheme, thus return None
    None
}

/// Whether a scheme span is the given scheme, ignoring case and the three bytes a URL parser removes
fn does_input_match_scheme(potential_scheme: &[u8], scheme: &[u8]) -> bool {
    // take iterator on reference scheme (eg. https)
    let mut scheme = scheme.iter();

    // bytewise identity check between potential scheme and scheme
    for &byte in potential_scheme {
        match scheme.next() {
            Some(expected) if expected.eq_ignore_ascii_case(&byte) => {}
            _ => return false,
        }
    }

    // every byte must have been matched, else `potential_scheme` is prefix of `scheme`, and thus not a match
    scheme.next().is_none()
}

// takes a path segment (ie. part between forward slashes) and returns True if it is equivalent to the backward directory (..), or False otherwise
fn is_path_segment_backward_directory(path_segment: &[u8]) -> bool {
    // count consecutive dots (ignoring whitespace) - returns True if two or more are encountered
    let mut dots = 0;

    // if a percent char doesn't start a valid percent encoding, then that percent encoded char is unreadable, but need to keep looking for dots, to ensure that a % cannot be used to break `..` filtering
    let mut unreadable = false;

    // init cursor over path segment
    let mut cursor = 0;

    while cursor < path_segment.len() {
        let byte = if path_segment[cursor] == b'%' {
            // potentially percent encoded byte encountered

            // try to decode the char
            let Some(decoded) = decode_percent_escape(path_segment.get(cursor + 1..)) else {
                // it didn't decode properly, mark the char where the percent is as unreadable, so that the next char is checked
                unreadable = true;
                cursor += 1;

                continue;
            };

            // percent decoded fine, increment cursor
            cursor += 3;

            // write `decoded` into `byte`
            decoded
        } else {
            // byte encountered is not percent encoded

            // get the byte
            let byte = path_segment[cursor];

            // increment the cursor
            cursor += 1;

            // simply write the byte into `byte`
            byte
        };

        // if the byte - either literal or decoded from percent encoding - is a dot, then need to add the dots counter
        match byte {
            b'.' => dots += 1,
            b' ' => {}

            // not a dot and not a space, and byte is not unreadable, then the path segment doesn't give `..`, so can return false
            _ if !unreadable => return false,

            // not a dot, and not a space, but byte is unreadable, so keep searching
            _ => {}
        }
    }

    // if dots found >= 2, return True, else False
    dots >= 2
}

/// Takes a percent escaped char (eg. %20), and returns the decoded char byte
fn decode_percent_escape(bytes: Option<&[u8]>) -> Option<u8> {
    /*
        percent escaping takes the hex code of a char and prepends a percent sign for it
        eg. space is 0x20, thus becomes %20

        thus, to decode, throw away the %, take the first char as base 10 and mulitply by 16, then add the second char (in base 10)
    */

    let pair = bytes?.get(..2)?; // will return none if less than two chars are passed in

    let sixteens = (pair[0] as char).to_digit(16)?;
    let ones = (pair[1] as char).to_digit(16)?;

    Some((sixteens * 16 + ones) as u8)
}

/// Whether a relative reference resolves to a file the document could have been handed with, rather than one outside its directory
fn is_in_same_or_descendant_directory(trimmed_link_content: &[u8]) -> bool {
    /*
        backslashes, control characters, and DEL chars - anywhere in the string - all lead to the file being rejected
    */
    for &byte in trimmed_link_content {
        if byte == b'\\' || byte < 0x20 || byte == 0x7F {
            return false;
        }
    }

    // leading forward slash means resource won't necessarily be in the same directory as the HTML content, thus to avoid arbitrary path traversal, reject
    if trimmed_link_content.first() == Some(&b'/') {
        return false;
    }

    /*
        AN ESCAPE MAY NOT DECODE TO A SEPARATOR

        The walk below splits the path on `/` and calls what is between two of
        them a segment. Whoever resolves this URL splits it on `/` too - but
        they do it after percent decoding, and this does not, so `..%2fsecret`
        is one segment here and two everywhere else. It climbs, and the walk
        never sees a `..` to catch.

        Which decoder does that is not something this can know, and it is not
        theoretical: `python -m http.server` decodes and then normalises, so a
        document served by one and asking for `..%2fsecret.png` is handed the
        file from the directory above. Chromium opening the same document from
        disk declines to load it at all.

        So the rule is the one that holds whoever is reading: an escape may not
        decode to a byte the scan above would have refused unescaped, and may
        not decode to a separator. Nothing is lost - no file has a `/` in its
        name, so a percent encoded one can only ever have been a separator
        somebody spelled quietly.

        A doubly encoded `%252f` decodes once, to the text `%2f`, and is left
        alone: one decode is the contract, and a reader that runs two is broken
        in a way no renderer can anticipate
    */
    let mut cursor = 0;

    while cursor < trimmed_link_content.len() {
        if trimmed_link_content[cursor] != b'%' {
            // byte does not mark the start of a percent encoded character
            // simply increment cursor
            cursor += 1;
            continue;
        }

        // cursor points to a %
        // decode the percent encoded byte, and if it's a forward slash, backslash, space, or DEL, then reject path
        if let Some(decoded_byte) = decode_percent_escape(trimmed_link_content.get(cursor + 1..)) {
            if decoded_byte == b'/'
                || decoded_byte == b'\\'
                || decoded_byte < 0x20
                || decoded_byte == 0x7F
            {
                return false;
            }

            // +3 as percent encoded char: %__ (thus len 3)
            cursor += 3;
        } else {
            // not a percent encoded byte
            // simply increment cursor over it
            cursor += 1;
        }
    }

    // split the path into segments on forward slashes, and check that no backward directory traversals (..) are present
    for path_segment in trimmed_link_content.split(|&byte| byte == b'/') {
        if is_path_segment_backward_directory(path_segment) {
            return false;
        }
    }

    // if here, no segment with .. (or equivalent) was found, so resource is in same or descendant directory
    true
}

/// Takes trimmed link content, and returns whether it has a file extension that indicates that the file can be embedded or not.
/// Note that parity between the claimed and actual file types is not checked or enforced
fn get_embed_type(trimmed_link_content: &[u8]) -> EmbedTypes {
    // find the last dot in the file path (percent encoding is not respected, so not looking for %2e)
    // anything after this will be taken as the file extension
    let Some(last_dot) = trimmed_link_content.iter().rposition(|&byte| byte == b'.') else {
        return EmbedTypes::None;
    };

    // slice the extension
    let potential_extension = &trimmed_link_content[last_dot + 1..];

    // does potential extension indicate an allowed image extension?
    if is_potential_extension_in_array(potential_extension, &ACCEPTED_IMAGE_EXTENSIONS) {
        return EmbedTypes::Image;
    }

    // does potential extension indicate a PDF extension?
    if is_potential_extension_in_array(potential_extension, &ACCEPTED_PDF_EXTENSIONS) {
        return EmbedTypes::Pdf;
    }

    // does potential extension indicate an allowed audio extension?
    if is_potential_extension_in_array(potential_extension, &ACCEPTED_AUDIO_EXTENSIONS) {
        return EmbedTypes::Audio;
    }

    // does potential extension indicate an allowed video extension?
    if is_potential_extension_in_array(potential_extension, &ACCEPTED_VIDEO_EXTENSIONS) {
        return EmbedTypes::Video;
    }

    // not a candidate for embedding
    // indicate that it won't be embedded, so fallback to link behaviour can take place
    EmbedTypes::None
}

#[inline]
/// Takes a potential file extension, and checks - irrespective of upper/lower case - whether it's in the supplied array of valid extensions of a given category
fn is_potential_extension_in_array(
    potential_extension: &[u8],
    allowed_extension_array: &[&[u8]],
) -> bool {
    // note that there's no assurance that the potential_extension is actually a valid extension, let alone an allowed one

    allowed_extension_array
        .iter()
        .any(|known| potential_extension.eq_ignore_ascii_case(known))
}

/// Takes trimmed link content, and encodes characters that may prove problematic in path parsing.
/// Returns None if either encoding is not required, or no problematic characters are present
fn encode_problematic_chars(trimmed_link_content: &[u8]) -> Option<Vec<u8>> {
    // if the trimmed char is a URL, in the accepted format, don't need to sanitise the content as is required for file paths
    if identify_scheme(trimmed_link_content) != Scheme::None {
        return None;
    }

    // if the input doesn't contain any problematic chars, also return early
    // checked outside of loop below to avoid doing the alloc in the (common) case where these chars don't occur in the file path
    if !trimmed_link_content
        .iter()
        .any(|&byte| byte == b'#' || byte == b'?' || byte == b'%')
    {
        return None;
    }

    // init array for encoded link content - which will be minimally sized at the size of the link content
    let mut encoded_link_content = Vec::with_capacity(trimmed_link_content.len() + 2);

    /*
        the problematic chars are the % (for percent encoding), the # (for fragments), and the ? (for queries)
        all of these need to be replaced with their percent encoded variant so that they don't invoke any of those features
        this allows file paths to be WYSIWYG
    */
    for &byte in trimmed_link_content {
        match byte {
            b'%' => encoded_link_content.extend_from_slice(b"%25"),
            b'#' => encoded_link_content.extend_from_slice(b"%23"),
            b'?' => encoded_link_content.extend_from_slice(b"%3F"),
            _ => encoded_link_content.push(byte),
        }
    }

    // return encoded link content
    Some(encoded_link_content)
}

/// Takes the raw link content bytes, and returns a trimmed version, and a version with problematic chars encoded
fn get_link_content_bytes(raw_link_content: &[u8]) -> (&[u8], Option<Vec<u8>>) {
    // remove leading/trailing whitespace and control chars
    let trimmed_link_content = trim_link_content(raw_link_content);

    // percent encode problematic characters: ?, #, %
    let encoded_link_content = encode_problematic_chars(trimmed_link_content);

    (trimmed_link_content, encoded_link_content)
}

impl HtmlRenderer<'_> {
    /// Takes the bytes associated with a link's content, and appends the HTML it contributes to the output array
    pub(super) fn handle_link(&mut self, raw_link_content: &[u8]) {
        // perform initial processing on the input bytes
        let (trimmed_link_content, encoded_link_content) = get_link_content_bytes(raw_link_content);

        // links are denied by default, to provide some additional security
        // unless the feature flag has been passed, need to write out the link in a non-interactive manner, so the user can see that a link was there, but it's not been accepted for interactivity
        if self.link_permission_status == LinkPermissions::Blocked
            || classify_link_content(trimmed_link_content) == TargetType::Rejected
        {
            // write out opening tag
            self.html_output_array
                .extend_from_slice(b"<span class=\"dead-link\">");

            // write out the link content
            write_out_escaped_bytes(self.html_output_array, raw_link_content);

            // write out the closing tag
            self.html_output_array.extend_from_slice(b"</span>");

            return;
        }

        /*
            here, the link can be shown
        */

        // encode the link content - if None was returned, then either encoding is not required or no problematic chars exist, in which case the trimmed link content can be used
        let encoded_link_content = encoded_link_content
            .as_deref()
            .unwrap_or(trimmed_link_content);

        // write out the opening tag
        self.html_output_array
            .extend_from_slice(b"<a class=\"link\" href=\"");

        // if the target is being treated as a file, additional security measure
        if classify_link_content(trimmed_link_content) == TargetType::File {
            // write out the bytes `./` as another safeguard to keep the file path safe (same or descendant directory to where the HTML is)
            write_out_escaped_bytes(self.html_output_array, &CURRENT_DIRECTORY_PATH_SEGMENT);
        }

        // write out the bytes for the link used as the actual link - NOT what is displayed
        write_out_escaped_bytes(self.html_output_array, encoded_link_content);

        /*
            close the opening tag

            target=_blank opens the file in a new tab

            rel = noopener noreferrer prevents the target resource having access to the document that opened it, and makes the referrer unknown to the target resource
        */
        self.html_output_array
            .extend_from_slice(b"\" target=\"_blank\" rel=\"noopener noreferrer\">");

        // write out the display text - NOT what the actual link is
        write_out_escaped_bytes(self.html_output_array, raw_link_content);

        // write out the closing tag
        self.html_output_array.extend_from_slice(b"</a>");
    }

    /// Takes the bytes associated with an embedded link's content, and appends the HTML it contributes to the output array.
    /// Falls back to Link if embedding cannot proceed, but linking remains appropriate
    pub(super) fn handle_embedded_link(&mut self, raw_link_content: &[u8]) {
        // perform initial processing on the input bytes
        let (trimmed_link_content, encoded_link_content) = get_link_content_bytes(raw_link_content);

        // get the type of the embed - includes None, for if should NOT embed
        // note that similarly to links, embeds are denied by default, to provide some additional security
        let embed_type = if self.link_permission_status == LinkPermissions::Allowed
            && classify_link_content(trimmed_link_content) == TargetType::File
        {
            // only files are embedded

            // if being treated as a file (which it is here, then proceed to get the specific type - eg. image, pdf - of the embed)
            get_embed_type(trimmed_link_content)
        } else {
            // should not embed if links blocked, or is a web resource, or embedded link has been rejected
            EmbedTypes::None
        };

        /*
            here, MIGHT be able to embed the resource
        */

        match embed_type {
            EmbedTypes::Image => {
                // encode the link content - if None was returned, then either encoding is not required or no problematic chars exist, in which case the trimmed link content can be used
                let encoded_link_content = encoded_link_content
                    .as_deref()
                    .unwrap_or(trimmed_link_content);

                // write out the opening tag
                self.html_output_array
                    .extend_from_slice(b"<img class=\"embed embed-image\" src=\"");

                // write out the bytes `./` as another safeguard to keep the file path safe (same or descendant directory to where the HTML is)
                write_out_escaped_bytes(self.html_output_array, &CURRENT_DIRECTORY_PATH_SEGMENT);

                // write out the embed's source, using the encoded path - NOT raw input
                write_out_escaped_bytes(self.html_output_array, encoded_link_content);

                // write out bytes for alt text - use the trimmed link content for this
                self.html_output_array.extend_from_slice(b"\" alt=\"");
                write_out_escaped_bytes(self.html_output_array, trimmed_link_content);

                // write out the end of the opening tag (img doesn't require a closing tag)
                self.html_output_array
                    .extend_from_slice(b"\" loading=\"lazy\">");
            }

            EmbedTypes::Video => {
                // encode the link content - if None was returned, then either encoding is not required or no problematic chars exist, in which case the trimmed link content can be used
                let encoded_link_content = encoded_link_content
                    .as_deref()
                    .unwrap_or(trimmed_link_content);

                // write out the opening tag
                self.html_output_array
                    .extend_from_slice(b"<video class=\"embed embed-video\" src=\"");

                // write out the bytes `./` as another safeguard to keep the file path safe (same or descendant directory to where the HTML is)
                write_out_escaped_bytes(self.html_output_array, &CURRENT_DIRECTORY_PATH_SEGMENT);

                // write out the embed's source, using the encoded path - NOT raw input
                write_out_escaped_bytes(self.html_output_array, encoded_link_content);

                // write out closing tag
                // only preload metadata to avoid thrashing the disk if multiple videos are embedded
                self.html_output_array
                    .extend_from_slice(b"\" controls preload=\"metadata\"></video>");
            }

            EmbedTypes::Audio => {
                // encode the link content - if None was returned, then either encoding is not required or no problematic chars exist, in which case the trimmed link content can be used
                let encoded_link_content = encoded_link_content
                    .as_deref()
                    .unwrap_or(trimmed_link_content);

                // write out the opening tag
                self.html_output_array
                    .extend_from_slice(b"<audio class=\"embed embed-audio\" src=\"");

                // write out the bytes `./` as another safeguard to keep the file path safe (same or descendant directory to where the HTML is)
                write_out_escaped_bytes(self.html_output_array, &CURRENT_DIRECTORY_PATH_SEGMENT);

                // write out the embed's source, using the encoded path - NOT raw input
                write_out_escaped_bytes(self.html_output_array, encoded_link_content);

                // write out closing tag
                // only preload metadata to avoid thrashing the disk if multiple audio tracks are embedded
                self.html_output_array
                    .extend_from_slice(b"\" controls preload=\"metadata\"></audio>");
            }

            EmbedTypes::Pdf => {
                // encode the link content - if None was returned, then either encoding is not required or no problematic chars exist, in which case the trimmed link content can be used
                let encoded_link_content = encoded_link_content
                    .as_deref()
                    .unwrap_or(trimmed_link_content);

                // write out the opening tag
                // note that `sandbox` cannot be applied to the iframe, as it breaks the PDF embed
                self.html_output_array
                    .extend_from_slice(b"<iframe class=\"embed embed-pdf\" src=\"");

                // write out the bytes `./` as another safeguard to keep the file path safe (same or descendant directory to where the HTML is)
                write_out_escaped_bytes(self.html_output_array, &CURRENT_DIRECTORY_PATH_SEGMENT);

                // write out the embed's source, using the encoded path - NOT raw input
                write_out_escaped_bytes(self.html_output_array, encoded_link_content);

                // write out bytes for the title that should be used for the iframe
                // helps screen readers
                self.html_output_array.extend_from_slice(b"\" title=\"");
                write_out_escaped_bytes(self.html_output_array, trimmed_link_content);

                // write out closing tag
                self.html_output_array
                    .extend_from_slice(b"\" loading=\"lazy\"></iframe>");
            }

            // cannot embed, fallback to normal link behaviour
            // note that some unnecessary overhead is incurred here, due to repeated work on the handle_link call
            EmbedTypes::None => self.handle_link(raw_link_content),
        }
    }
}
