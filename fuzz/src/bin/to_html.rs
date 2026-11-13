use afl::fuzz;
use desert_down::parser::{self, Folding, LinkPermissions, MathRendering, OutputWidth, Theme};

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

        // document, links blocked, math rendering disabled
        let mut document_blocked = Vec::new();
        ast.ast_to_html_document_to_array(
            input,
            &mut document_blocked,
            Theme::Dark,
            OutputWidth::Fixed,
            LinkPermissions::Blocked,
            Folding::DISABLED,
            MathRendering::Disabled,
            "fuzz",
        );

        // document, links allowed, math rendering disabled
        let mut document_allowed = Vec::new();
        ast.ast_to_html_document_to_array(
            input,
            &mut document_allowed,
            Theme::Dark,
            OutputWidth::Fixed,
            LinkPermissions::Allowed,
            Folding::DISABLED,
            MathRendering::Disabled,
            "fuzz",
        );

        // document, links allowed, folding and math rendering enabled
        let mut document_foldable = Vec::new();
        ast.ast_to_html_document_to_array(
            input,
            &mut document_foldable,
            Theme::Reactive,
            OutputWidth::Full,
            LinkPermissions::Allowed,
            Folding::ALL,
            MathRendering::Enabled,
            "fuzz",
        );

        // document, links blocked, only some folding enabled, math rendering enabled
        let mut document_partly_foldable = Vec::new();
        ast.ast_to_html_document_to_array(
            input,
            &mut document_partly_foldable,
            Theme::Light,
            OutputWidth::Fixed,
            LinkPermissions::Blocked,
            Folding {
                headings: true,
                list_items: false,
                callouts: true,
            },
            MathRendering::Enabled,
            "fuzz",
        );
    })
}
