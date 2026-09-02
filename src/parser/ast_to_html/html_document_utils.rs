use super::ast_utils::AST;
use super::html_escape_utils::write_out_escaped_bytes;
use super::links::LinkPermissions;
use super::styling::{OutputWidth, Theme};

// HTMl header
const HEAD_BEFORE_THEME: &[u8] = b"<!DOCTYPE html>
<html lang=\"en\" data-theme=\"";

const HEAD_BEFORE_TITLE: &[u8] = b"\">
<head>
<meta charset=\"utf-8\">
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
<meta charset=\"utf-8\">
";

/*
    CSP for defence in depth
    uses hash for styling to avoid unsafe-inline

    note that there are two variants - one for if links and embedded links are allowed, one for if they're blocked
*/
const CSP_BEFORE_STYLING_HASH: &[u8] = b"<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src 'none'; style-src 'sha256-";

const CSP_AFTER_STYLING_HASH_LINKS_ALLOWED: &[u8] =
    b"'; img-src 'self' file:; connect-src 'none'; font-src 'none'; object-src 'none'; media-src 'self' file:; frame-src 'self' file:; form-action 'none'; base-uri 'none';\">\n<title>";

const CSP_AFTER_STYLING_HASH_LINKS_BLOCKED: &[u8] =
    b"'; img-src 'none'; connect-src 'none'; font-src 'none'; object-src 'none'; media-src 'none'; frame-src 'none'; form-action 'none'; base-uri 'none';\">\n<title>";

const HEAD_AFTER_TITLE: &[u8] = b"</title>\n<style>";
const HEAD_AFTER_STYLE: &[u8] = b"</style>\n</head>\n<body>\n<main class=\"desert-down\">\n";
const HTML_CLOSER: &[u8] = b"</main>\n</body>\n</html>\n";

// default value for <title> tag if title passed in is empty
const DEFAULT_TITLE: &[u8] = b"Untitled Document";

impl AST {
    /// Takes an AST, the input string, and an array, and appends a fully formed HTML document for that AST to the array.
    /// Additionally takes a title for the HTML document
    pub fn ast_to_html_document_to_array(
        &self,
        input_string: &str,
        output_array: &mut Vec<u8>,
        theme: Theme,
        output_width: OutputWidth,
        link_permission_status: LinkPermissions,
        doc_title: &str,
    ) {
        // get styling
        let [theme_styling, width_styling, styling_rules] = theme.get_html_styling(output_width);

        // reserve space for output bytes of HTML
        // guess based on input string length, not AST size, as output bytes need to include content chars that AST nodes fold into content
        output_array.reserve(
            input_string.len()
                + input_string.len() / 2
                + theme_styling.len()
                + width_styling.len()
                + styling_rules.len()
                + doc_title.len()
                + CSP_BEFORE_STYLING_HASH.len()
                + CSP_AFTER_STYLING_HASH_LINKS_ALLOWED.len()
                + 512,
        );

        // write bytes for the HTML doc header, and light/dark theme
        output_array.extend_from_slice(HEAD_BEFORE_THEME);
        output_array.extend_from_slice(theme.get_theme_name_as_string().as_bytes());
        output_array.extend_from_slice(HEAD_BEFORE_TITLE);

        // write out bytes for CSP
        output_array.extend_from_slice(CSP_BEFORE_STYLING_HASH);
        output_array.extend_from_slice(theme.stylesheet_hash(output_width).as_bytes());

        output_array.extend_from_slice(match link_permission_status {
            // if links/embedded links allowed, use more permissive CSP, otherwise, more restrictive one
            LinkPermissions::Allowed => CSP_AFTER_STYLING_HASH_LINKS_ALLOWED,
            LinkPermissions::Blocked => CSP_AFTER_STYLING_HASH_LINKS_BLOCKED,
        });

        // write out bytes for doc title
        if doc_title.is_empty() {
            output_array.extend_from_slice(DEFAULT_TITLE);
        } else {
            write_out_escaped_bytes(output_array, doc_title.as_bytes());
        }

        // write out rest of header - mainly styling
        output_array.extend_from_slice(HEAD_AFTER_TITLE);
        output_array.extend_from_slice(theme_styling.as_bytes());
        output_array.extend_from_slice(width_styling.as_bytes());
        output_array.extend_from_slice(styling_rules.as_bytes());
        output_array.extend_from_slice(HEAD_AFTER_STYLE);

        // write out the HTML content for the AST
        self.ast_to_html_to_array(input_string, output_array, link_permission_status);

        // close the HTML
        output_array.extend_from_slice(HTML_CLOSER);
    }

    /// Takes an AST, the input string, and returns a fully formed HTML document for that AST as a string
    pub fn ast_to_html_document_to_string(
        &self,
        input_string: &str,
        theme: Theme,
        output_width: OutputWidth,
        link_permission_status: LinkPermissions,
        doc_title: &str,
    ) -> String {
        // create output array
        let mut output_array = Vec::new();

        // populate output array
        self.ast_to_html_document_to_array(
            input_string,
            &mut output_array,
            theme,
            output_width,
            link_permission_status,
            doc_title,
        );

        // convert to string, and return
        // panics if not a valid UTF-8 string, but the to-AST logic ensures it is
        String::from_utf8(output_array).expect("Rendered HTML was not valid UTF-8")
    }

    /// Takes an AST, the input string, and writes a fully formed HTML document for that AST to standard output
    pub fn ast_to_html_document_to_stdout(
        &self,
        input_string: &str,
        theme: Theme,
        output_width: OutputWidth,
        link_permission_status: LinkPermissions,
        doc_title: &str,
    ) {
        /*
            note that for portability with windows console, output must be UTF-8
            this is checked in the to-AST implementation, as String and &str are guaranteed to refer to valid UTF-8
            the to-HTML functionality also ensures that the output is UTF-8
        */

        // create output array
        let mut output_array = Vec::new();

        // populate the output array
        self.ast_to_html_document_to_array(
            input_string,
            &mut output_array,
            theme,
            output_width,
            link_permission_status,
            doc_title,
        );

        // get access to process' standard output, and lock it so that the write can proceed in one go
        // lock is released as function goes out of scope
        let mut stdout = std::io::stdout().lock();

        // so .write_all() can be used below
        use std::io::Write;

        // otherwise, write error to standard error, not standard output (as standard output may be what failed)
        if let Err(error) = stdout.write_all(&output_array)
            && error.kind() != std::io::ErrorKind::BrokenPipe
        {
            eprintln!("Failed to write HTML document: {error}");
        }
    }
}
