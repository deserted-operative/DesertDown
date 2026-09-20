use afl::fuzz;
use desert_down::parser::{self, LinkPermissions, OutputWidth, Theme};

fn main() {
    fuzz!(|data: &[u8]| {
        let Ok(input) = std::str::from_utf8(data) else {
            return;
        };

        let ast = parser::parse_input(input);

        // fragment, links not allowed
        let mut fragment_blocked = Vec::new();
        ast.ast_to_html_to_array(input, &mut fragment_blocked, LinkPermissions::Blocked);

        // fragment, links allowed
        let mut fragment_allowed = Vec::new();
        ast.ast_to_html_to_array(input, &mut fragment_allowed, LinkPermissions::Allowed);

        // document, links blocked
        let mut document_blocked = Vec::new();
        ast.ast_to_html_document_to_array(
            input,
            &mut document_blocked,
            Theme::Dark,
            OutputWidth::Fixed,
            LinkPermissions::Blocked,
            "fuzz",
        );

        // document, links allowed
        let mut document_allowed = Vec::new();
        ast.ast_to_html_document_to_array(
            input,
            &mut document_allowed,
            Theme::Dark,
            OutputWidth::Fixed,
            LinkPermissions::Allowed,
            "fuzz",
        );
    })
}
