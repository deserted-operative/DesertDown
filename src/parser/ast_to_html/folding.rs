#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Which (if any) of headings, list items, and callouts should be foldable in HTML (document, not fragment) output
pub struct Folding {
    pub headings: bool,
    pub list_items: bool,
    pub callouts: bool,
}

impl Folding {
    // nothing foldable, no scripts added to HTML output
    pub const DISABLED: Folding = Folding {
        headings: false,
        list_items: false,
        callouts: false,
    };

    // all foldable
    pub const ALL: Folding = Folding {
        headings: true,
        list_items: true,
        callouts: true,
    };

    #[inline]
    /// Whether anything is foldable - useful for determining whether the folding script and styling need to be added to the HTML document
    pub const fn is_anything_foldable(self) -> bool {
        self.headings || self.list_items || self.callouts
    }

    /// Takes comma separated list of targets for folding.
    /// Looks for which (if any) of headings, list items, and callouts should be foldable in HTML (document, not fragment) output.
    /// Not case nor order sensitive.
    /// Returns None if list is empty or if anything in list is not recognised.
    pub fn parse_potential_folding_targets(potential_folding_targets: &str) -> Option<Folding> {
        let mut folding = Folding::DISABLED;

        for kind in potential_folding_targets.split(',') {
            if kind.eq_ignore_ascii_case("headings") {
                folding.headings = true;
            } else if kind.eq_ignore_ascii_case("list-items") {
                folding.list_items = true;
            } else if kind.eq_ignore_ascii_case("callouts") {
                folding.callouts = true;
            } else if kind.eq_ignore_ascii_case("all") {
                folding = Folding::ALL;
            } else {
                return None;
            }
        }

        Some(folding)
    }

    /// Write out value of the data-dd-fold attribute to inform folding script what is foldable.
    /// Space separated - eg. "headings callouts".
    /// Everything folded if attribute is absent.
    pub(crate) fn write_out_folding_data_attribute_value(self, output_array: &mut Vec<u8>) {
        let mut separator: &[u8] = b"";

        for (is_foldable, kind) in [
            (self.headings, b"headings".as_slice()),
            (self.list_items, b"list-items".as_slice()),
            (self.callouts, b"callouts".as_slice()),
        ] {
            if is_foldable {
                output_array.extend_from_slice(separator);
                output_array.extend_from_slice(kind);
                separator = b" ";
            }
        }
    }
}

// script for adding folding to HTML
// applies to elements with '.desert-down' in HTML output
pub const FOLDING_SCRIPT: &str = include_str!("folding.js");

// Base64 encododing of SHA-256 of folding scripting, for CSP
// WARNING: scripts/update_hashes.sh finds this constant by its name and declaration
// renaming these will require changes to the script
pub const FOLDING_SCRIPT_HASH: &str = "JiJADJzWzfQbbq+cCD1jzpBhIld3ruqPnsOdtgwbgvo=";

// styling for folding support in HTML
pub const FOLDING_STYLING: &str = include_str!("folding.css");

// Base64 encododing of SHA-256 of folding-specific styling, for CSP
// WARNING: scripts/update_hashes.sh finds this constant by its name and declaration
// renaming these will require changes to the script
pub const FOLDING_STYLING_HASH: &str = "9mATNWP91y5peXslc2XgJrNCdmFZoVLfieLE8V9VZqI=";
