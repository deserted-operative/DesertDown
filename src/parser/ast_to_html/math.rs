#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Whether math (block and inline) should be rendered.
/// If so, uses Temml.
/// If not, written out as LaTeX source code
pub enum MathRendering {
    #[default]
    Disabled,

    Enabled,
}

impl MathRendering {
    #[inline]
    /// Returns whether math rendering is enabled. Required for determining whether Temml scripts should be included in HTML output
    pub const fn is_enabled(self) -> bool {
        matches!(self, MathRendering::Enabled)
    }
}

// include Temml script
pub const TEMML_SCRIPT: &str = include_str!("temml/temml.min.js");

// Base64 encoding of SHA-256 of Temml's script, for CSP
// WARNING: scripts/update_hashes.sh finds this constant by its name and declaration
// changing this will require changes to the script
pub const TEMML_SCRIPT_HASH: &str = "VrQzgXbTkXBpJQO4t+xjDT7fjhCGGJVljet0rvtmr50=";

// script for rendering the maths in HTML output with Temml (must come after Temml's script)
// applies to maths within elements with '.desert-down' in HTML output
pub const MATH_SCRIPT: &str = include_str!("math.js");

// Base64 encoding of SHA-256 of maths scripting, for CSP
// WARNING: scripts/update_hashes.sh finds this constant by its name and declaration
// changing this will require changes to the script
pub const MATH_SCRIPT_HASH: &str = "sbQfjtA3TebdL/KrcxV6edZjg4v2VsLj5JmY9POxGV4=";

// Temml's Latin Modern stylesheet, with its fonts embedded as data: URIs, so the document needs no font files (generated - see temml/README.md)
pub const MATH_STYLING: &str = include_str!("math.css");

// Base64 encoding of SHA-256 of maths styling, for CSP
// WARNING: scripts/update_hashes.sh finds this constant by its name and declaration
// changing this will require changes to the script
pub const MATH_STYLING_HASH: &str = "hiRiiw6njW888IfBTSNGvD0r7wahAjttbJYgHbyMqQU=";
