use super::ast_utils::AST;
use super::folding::{
    FOLDING_SCRIPT, FOLDING_SCRIPT_HASH, FOLDING_STYLING, FOLDING_STYLING_HASH, Folding,
};
use super::html_escape_utils::write_out_escaped_bytes;
use super::links::LinkPermissions;
use super::math::{
    MATH_SCRIPT, MATH_SCRIPT_HASH, MATH_STYLING, MATH_STYLING_HASH, MathRendering, TEMML_SCRIPT,
    TEMML_SCRIPT_HASH,
};
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
    
    Uses hashes for styling and scripts to avoid unsafe-inline

    Written out in pieces, as it varies with:
    - Whether links and embedded links are allowed or blocked
    - Whether folding is enabled (where its script and styling are hashed)
    - Whether math rendering is enabled (Temml's script, the math script, and the math styling are hashed, and the math styling's embedded fonts and images are `data:` URIs, so `data:`` is allowed for fonts and images)
    
    There are no scripts, and no fonts or data: images, unless folding or math rendering is enabled
*/
const CSP_BEFORE_SCRIPT_SOURCES: &[u8] =
    b"<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src";
const CSP_BEFORE_STYLE_SOURCES: &[u8] = b"; style-src";
const CSP_BEFORE_IMG_SOURCES: &[u8] = b"; img-src";
const CSP_BEFORE_FONT_SOURCES: &[u8] = b"; connect-src 'none'; font-src";

// a source list of only 'none'
const CSP_NO_SOURCES: &[u8] = b" 'none'";

// a hash in a source list
// WARNING: scripts/update_hashes.sh checks for each hash in this form ('sha256-...')
// changing these will require changes to the script
const CSP_HASH_OPENER: &[u8] = b" 'sha256-";
const CSP_HASH_CLOSER: &[u8] = b"'";

// image sources, by whether links are allowed (and whether math rendering is enabled)
const CSP_IMG_SOURCES_LINKS_ALLOWED: &[u8] = b" 'self' file:";
const CSP_IMG_SOURCES_LINKS_ALLOWED_WITH_MATH: &[u8] = b" 'self' file: data:";
const CSP_IMG_SOURCES_LINKS_BLOCKED_WITH_MATH: &[u8] = b" data:";

// font sources when maths rendering is enabled (otherwise none)
const CSP_FONT_SOURCES_WITH_MATH: &[u8] = b" data:";

const CSP_AFTER_FONT_SOURCES_LINKS_ALLOWED: &[u8] =
    b"; object-src 'none'; media-src 'self' file:; frame-src 'self' file:; form-action 'none'; base-uri 'none';\">\n<title>";

const CSP_AFTER_FONT_SOURCES_LINKS_BLOCKED: &[u8] =
    b"; object-src 'none'; media-src 'none'; frame-src 'none'; form-action 'none'; base-uri 'none';\">\n<title>";

// WARNING: scripts/update_hashes.sh finds style and script elements by their tags
// changing these will require changes to the script
const HEAD_AFTER_TITLE: &[u8] = b"</title>\n<style>";
const HEAD_AFTER_STYLE: &[u8] = b"</style>\n</head>\n<body>\n<main class=\"desert-down\"";
const MAIN_OPENER_CLOSER: &[u8] = b">\n";
const MAIN_CLOSER: &[u8] = b"</main>\n";
const BODY_AND_HTML_CLOSER: &[u8] = b"</body>\n</html>\n";

// folding and math styling receive separate style elements, so the hash of the main document styling is unaffected
// WARNING: as above, scripts/update_hashes.sh relies on this being <style> without attributes
// changing this will require changes to the script
const ADDITIONAL_STYLE_OPENER: &[u8] = b"</style>\n<style>";

const FOLDING_DATA_ATTRIBUTE_OPENER: &[u8] = b" data-dd-fold=\""; // informs folding script what (if anything) of headings, list items, and/or callouts is foldable
const FOLDING_DATA_ATTRIBUTE_CLOSER: &[u8] = b"\"";

// WARNING: as above, scripts/update_hashes.sh relies on this being <script> without attributes
// changing this will require changes to the script
const SCRIPT_OPENER: &[u8] = b"<script>";
const SCRIPT_CLOSER: &[u8] = b"</script>\n";

// default value for <title> tag if title passed in is empty
const DEFAULT_TITLE: &[u8] = b"Untitled Document";

impl AST {
    /// Takes an AST, the input string, and an array, and appends a fully formed HTML document for that AST to the array.
    /// Additionally takes a title for the HTML document
    #[allow(clippy::too_many_arguments)]
    pub fn ast_to_html_document_to_array(
        &self,
        input_string: &str,
        output_array: &mut Vec<u8>,
        theme: Theme,
        output_width: OutputWidth,
        link_permission_status: LinkPermissions,
        folding: Folding,
        math: MathRendering,
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
                + if folding.is_anything_foldable() {
                    FOLDING_SCRIPT.len() + FOLDING_STYLING.len() + 256
                } else {
                    0
                }
                + if math.is_enabled() {
                    TEMML_SCRIPT.len() + MATH_SCRIPT.len() + MATH_STYLING.len() + 256
                } else {
                    0
                }
                + 512,
        );

        // write bytes for the HTML doc header, and light/dark theme
        output_array.extend_from_slice(HEAD_BEFORE_THEME);
        output_array.extend_from_slice(theme.get_theme_name_as_string().as_bytes());
        output_array.extend_from_slice(HEAD_BEFORE_TITLE);

        // write out bytes for CSP
        output_array.extend_from_slice(CSP_BEFORE_SCRIPT_SOURCES);

        // scripts, in order (math is rendered before folding is handled, so folding respects rendered math)
        if !folding.is_anything_foldable() && !math.is_enabled() {
            output_array.extend_from_slice(CSP_NO_SOURCES);
        }

        if math.is_enabled() {
            write_out_csp_hash(output_array, TEMML_SCRIPT_HASH);
            write_out_csp_hash(output_array, MATH_SCRIPT_HASH);
        }

        if folding.is_anything_foldable() {
            write_out_csp_hash(output_array, FOLDING_SCRIPT_HASH);
        }

        // styling, in the order it's written out
        output_array.extend_from_slice(CSP_BEFORE_STYLE_SOURCES);

        if folding.is_anything_foldable() {
            write_out_csp_hash(output_array, FOLDING_STYLING_HASH);
        }

        if math.is_enabled() {
            write_out_csp_hash(output_array, MATH_STYLING_HASH);
        }

        write_out_csp_hash(output_array, theme.stylesheet_hash(output_width));

        // if links/embedded links allowed, use more permissive CSP
        output_array.extend_from_slice(CSP_BEFORE_IMG_SOURCES);

        output_array.extend_from_slice(match (link_permission_status, math) {
            (LinkPermissions::Allowed, MathRendering::Disabled) => CSP_IMG_SOURCES_LINKS_ALLOWED,
            (LinkPermissions::Allowed, MathRendering::Enabled) => CSP_IMG_SOURCES_LINKS_ALLOWED_WITH_MATH,
            (LinkPermissions::Blocked, MathRendering::Disabled) => CSP_NO_SOURCES,
            (LinkPermissions::Blocked, MathRendering::Enabled) => CSP_IMG_SOURCES_LINKS_BLOCKED_WITH_MATH,
        });

        output_array.extend_from_slice(CSP_BEFORE_FONT_SOURCES);

        output_array.extend_from_slice(match math {
            MathRendering::Disabled => CSP_NO_SOURCES,
            MathRendering::Enabled => CSP_FONT_SOURCES_WITH_MATH,
        });

        output_array.extend_from_slice(match link_permission_status {
            LinkPermissions::Allowed => CSP_AFTER_FONT_SOURCES_LINKS_ALLOWED,
            LinkPermissions::Blocked => CSP_AFTER_FONT_SOURCES_LINKS_BLOCKED,
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

        // folding and math styling in own sections, so hash of main document styling is unaffected
        if folding.is_anything_foldable() {
            output_array.extend_from_slice(ADDITIONAL_STYLE_OPENER);
            output_array.extend_from_slice(FOLDING_STYLING.as_bytes());
        }

        if math.is_enabled() {
            output_array.extend_from_slice(ADDITIONAL_STYLE_OPENER);
            output_array.extend_from_slice(MATH_STYLING.as_bytes());
        }

        output_array.extend_from_slice(HEAD_AFTER_STYLE);

        // inform folding script what to fold
        if folding.is_anything_foldable() {
            output_array.extend_from_slice(FOLDING_DATA_ATTRIBUTE_OPENER);
            folding.write_out_folding_data_attribute_value(output_array);
            output_array.extend_from_slice(FOLDING_DATA_ATTRIBUTE_CLOSER);
        }

        output_array.extend_from_slice(MAIN_OPENER_CLOSER);

        // write out the HTML content for the AST
        self.ast_to_html_to_array(input_string, output_array, link_permission_status);

        // close HTML, with any scripts after the content they apply to
        output_array.extend_from_slice(MAIN_CLOSER);

        // maths first, so that folding sees the rendered maths
        if math.is_enabled() {
            write_out_script(output_array, TEMML_SCRIPT);
            write_out_script(output_array, MATH_SCRIPT);
        }

        if folding.is_anything_foldable() {
            write_out_script(output_array, FOLDING_SCRIPT);
        }

        output_array.extend_from_slice(BODY_AND_HTML_CLOSER);
    }

    /// Takes an AST, the input string, and returns a fully formed HTML document for that AST as a string
    #[allow(clippy::too_many_arguments)]
    pub fn ast_to_html_document_to_string(
        &self,
        input_string: &str,
        theme: Theme,
        output_width: OutputWidth,
        link_permission_status: LinkPermissions,
        folding: Folding,
        math: MathRendering,
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
            folding,
            math,
            doc_title,
        );

        // convert to string, and return
        // panics if not a valid UTF-8 string, but the to-AST logic ensures it is
        String::from_utf8(output_array).expect("Rendered HTML was not valid UTF-8")
    }

    /// Takes an AST, the input string, and writes a fully formed HTML document for that AST to standard output
    #[allow(clippy::too_many_arguments)]
    pub fn ast_to_html_document_to_stdout(
        &self,
        input_string: &str,
        theme: Theme,
        output_width: OutputWidth,
        link_permission_status: LinkPermissions,
        folding: Folding,
        math: MathRendering,
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
            folding,
            math,
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

// writes out a hash as a CSP source
fn write_out_csp_hash(output_array: &mut Vec<u8>, hash: &str) {
    output_array.extend_from_slice(CSP_HASH_OPENER);
    output_array.extend_from_slice(hash.as_bytes());
    output_array.extend_from_slice(CSP_HASH_CLOSER);
}

// writes out a script element
fn write_out_script(output_array: &mut Vec<u8>, script: &str) {
    output_array.extend_from_slice(SCRIPT_OPENER);
    output_array.extend_from_slice(script.as_bytes());
    output_array.extend_from_slice(SCRIPT_CLOSER);
}
