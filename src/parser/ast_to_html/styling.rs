#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,

    Dark,
}

impl Theme {
    #[inline]
    /// Used to parse CL arg for theme.
    /// Is case-insensitive
    pub fn parse_potential_theme_string(potential_theme: &str) -> Option<Theme> {
        if potential_theme.eq_ignore_ascii_case("light") {
            return Some(Theme::Light);
        }

        if potential_theme.eq_ignore_ascii_case("dark") {
            return Some(Theme::Dark);
        }

        None
    }

    #[inline]
    /// Takes a theme and returns the string for its name.
    /// Used for writing out the theme name to HTML document
    pub const fn get_theme_name_as_string(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    #[inline]
    /// Takes a theme, and returns the styling for that colour scheme
    pub const fn get_theme_styling(self) -> &'static str {
        match self {
            Theme::Light => LIGHT_STYLING,
            Theme::Dark => DARK_STYLING,
        }
    }

    #[inline]
    /// The whole stylesheet for this theme at this width - the two `:root` blocks followed by the shared rules.
    /// Three slices rather than one string, because the rules are the same bytes for every combination and are only stored once
    pub const fn get_html_styling(self, output_width: OutputWidth) -> [&'static str; 3] {
        [
            self.get_theme_styling(),
            output_width.get_output_width_styling(),
            STYLING_RULES,
        ]
    }

    /*
        styling is hashed so that the CSP can avoid using unsafe-inline on style-src

        hashes must be updated whenever any styling is

        note that the hashing is done externally to the project at the moment, to avoid adding another dependency to calculate it at build time, helping to maintain strong supply-chaing security
    */
    #[inline]
    /// Returns base64 encoded SHA-256 of the styling, accounting for theme and output width parameters
    pub const fn stylesheet_hash(self, output_width: OutputWidth) -> &'static str {
        match (self, output_width) {
            (Theme::Light, OutputWidth::Fixed) => "+op+z50sCKyZ+dVKEL1DPSfJR0oRuCiYfWSSWvzIGD4=",
            (Theme::Light, OutputWidth::Full) => "J8i56134E1I2BLEpcTxd5UGXY/4A6YhlrPWr1FsCXiE=",
            (Theme::Dark, OutputWidth::Fixed) => "5onbELHMHuwBOgtdWL6FkEf3VUeYoMhmHgNdVaQGYxU=",
            (Theme::Dark, OutputWidth::Full) => "P4PH6dH6gIL93G6wBiCzH/RclTCd8AMLlx/yDRdBRC4=",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Indicates whether HTML may consume a fixed width, or the full available width
pub enum OutputWidth {
    #[default]
    Fixed,

    Full,
}

impl OutputWidth {
    #[inline]
    /// Takes an output width, and returns the name as a string
    pub const fn get_output_width_name_as_string(self) -> &'static str {
        match self {
            OutputWidth::Fixed => "fixed",
            OutputWidth::Full => "full",
        }
    }

    #[inline]
    /// Used to parse CL arg for output width.
    /// Is case-insensitive
    pub fn parse_potential_output_width_string(
        potential_output_width: &str,
    ) -> Option<OutputWidth> {
        if potential_output_width.eq_ignore_ascii_case("fixed") {
            return Some(OutputWidth::Fixed);
        }

        if potential_output_width.eq_ignore_ascii_case("full") {
            return Some(OutputWidth::Full);
        }

        None
    }

    #[inline]
    /// Takes an output width, and returns the corresponding styling
    pub const fn get_output_width_styling(self) -> &'static str {
        match self {
            OutputWidth::Fixed => FIXED_WIDTH_STYLING,
            OutputWidth::Full => FULL_WIDTH_STYLING,
        }
    }
}

/*
    CSS definitions

    these are the parts relevant to hashing - start from the line below the r#", finish before the "#;
*/

/* width variable */
const FIXED_WIDTH_STYLING: &str = r#"
:root{--dd-max-width: 50rem}
"#;

const FULL_WIDTH_STYLING: &str = r#"
:root{--dd-max-width: none}
"#;

/* theme variables */
const LIGHT_STYLING: &str = r#"
:root{
    color-scheme: light;

    --dd-bg: #ffffff;
    --dd-bg-secondary: #f6f6f6;
    --dd-thead_bg: #eaeaea;
    
    --dd-lines: #d3d3d3;    

    --dd-text: #000000;
    --dd-italic: #41337A;
    --dd-bold: #246EB9;
    --dd-link: #3066be;
    --dd-quiet: #565656;

    --dd-highlight-colour-1: #f9c22e;
    --dd-highlight-colour-1-text: #000000;
    --dd-highlight-colour-2: #a5be00;
    --dd-highlight-colour-2-text: #000000;

    --dd-amber: #f18701;
    --dd-blue: #4070c9;
    --dd-cyan: #31929b;
    --dd-green: #498a42;
    --dd-grey: #595959;
    --dd-pink: #d91c5b;
    --dd-purple: #7546af;
    --dd-red: #b11b27;
    --dd-teal: #024F4a;

    --dd-colour-bg-opacity: 5%;
    --dd-colour-border-opacity: 50%;
}
"#;

const DARK_STYLING: &str = r#"
:root{
    color-scheme: dark;

    --dd-bg: #111111;
    --dd-bg-secondary: #161616;
    --dd-thead_bg: #202731;

    --dd-lines: #444450;
    
    --dd-text: #eeeeee;
    --dd-italic: #f4562a;
    --dd-bold: #ef438b;
    --dd-link: #5cadff;
    --dd-quiet: #aaaaaa;
    
    --dd-highlight-colour-1: #443300;
    --dd-highlight-colour-1-text: #ffda5d;
    --dd-highlight-colour-2: #094733;
    --dd-highlight-colour-2-text: #8bffcf;

    --dd-amber:#d98324;
    --dd-blue:#329bec;
    --dd-cyan:#1adbd5;
    --dd-green:#5af924;
    --dd-grey:#576a75;
    --dd-pink:#ed317f;
    --dd-purple:#977be0;
    --dd-red:#fb3741;
    --dd-teal:#6bffdc;

    --dd-colour-bg-opacity:5%;
    --dd-colour-border-opacity:50%;
}
"#;

/* style rules */
const STYLING_RULES: &str = r#"
/*
    DOCUMENT STYLING
*/
/* apply border-box to every element and pseudo-element, so that width and height include border and padding */
*,*::before,*::after{box-sizing: border-box}

html{-webkit-text-size-adjust: 100%}

body{
    background: var(--dd-bg);
    color: var(--dd-text);
    font-size: 16px;
    font-family: sans-serif;
    line-height: 1.65;
    margin: 0;
}

.desert-down{
    max-width: var(--dd-max-width);
    padding: 2.5rem 1.25rem 2.5rem; /* top, left/right, bottom */
    margin: 0 auto;
    overflow-wrap: break-word;
}

/*
    HEADINGS
*/
.desert-down h1,.desert-down h2,.desert-down h3,
.desert-down h4,.desert-down h5,.desert-down h6{
    font-weight: 600;
    line-height: 1.25;
    margin: 0;
}

.desert-down h1{font-size: 2.75rem}
.desert-down h2{font-size: 2.25rem}
.desert-down h3{font-size: 1.75rem; color: var(--dd-blue)}
.desert-down h4{font-size: 1.5rem; color: var(--dd-cyan)}
.desert-down h5{font-size: 1.25rem; color: var(--dd-teal)}
.desert-down h6{font-size: 1rem; color: var(--dd-green)}


/*
    TEXT
*/
.desert-down p{margin: 0; white-space: pre-wrap} /* pre-wrap to retain newlines in input */

.desert-down strong{color :var(--dd-bold)}
.desert-down em{color: var(--dd-italic)}

/* 50-50 blend the two colours when both simultaneously apply */
.desert-down strong em,
.desert-down em strong{
    color: var(--dd-bold);
    color: color-mix(in srgb, var(--dd-bold) 50%, var(--dd-italic) 50%);
}

/*
    LINKS
*/
a.link{
    text-decoration: underline;
    text-decoration-style: dotted;
    text-underline-offset: .2em;
}

.desert-down a{color: var(--dd-link); text-decoration: none}
.desert-down a:hover{text-decoration: underline}

.dead-link,.embed-image{
    color:var(--dd-quiet);
    text-decoration:line-through;
    text-decoration-thickness:1px; /* thin so content still readable */
}

.dead-link{cursor:not-allowed}


/*
    TAGS
*/
.tag{
    align-items: center;
    display: inline-flex;
    padding: .2em .5em;
    border-radius: calc(infinity * 1px);
    font-size: .85em;
    line-height: 1em;
    white-space: nowrap;
    color: var(--dd-link);
    background: var(--dd-bg-secondary);
    background: color-mix(in srgb, var(--dd-link) 10%, transparent);
}

.desert-down del{color: var(--dd-quiet)}
.desert-down u{text-underline-offset: .15em}


/*
    HIGHLIGHTING
*/
mark.hl-1{
background:var(--dd-highlight-colour-1);
color:var(--dd-highlight-colour-1-text);
padding: 0 0.1em;
}

mark.hl-2{
background:var(--dd-highlight-colour-2);
color:var(--dd-highlight-colour-2-text);
padding: 0 0.1em;
}

mark.hl-1.hl-2,
mark.hl-1 mark.hl-2,
mark.hl-2 mark.hl-1{
background:var(--dd-highlight-colour-2);
background:color-mix(in srgb,var(--dd-highlight-colour-1) 50%,var(--dd-highlight-colour-2) 50%);

color:var(--dd-highlight-colour-2-text);
color:color-mix(in srgb,var(--dd-highlight-colour-1-text) 50%,var(--dd-highlight-colour-2-text) 50%);
}

/*
    A nested mark represents an overlap inside an already padded highlight.
    It must not introduce another gap or rounded internal boundary.
*/
mark.hl-1 mark.hl-2,
mark.hl-2 mark.hl-1{
padding-inline:0;
border-radius:0;
}

/*
    When an overlapping highlight reaches the end of its outer highlight, the
    inner mark already supplies the ending. Remove the outer mark's additional
    strip of background.
*/
mark.hl-1:has(> mark.hl-2:last-child),
mark.hl-2:has(> mark.hl-1:last-child){
padding-right:0;
}


/*
    LISTS
*/
.desert-down ul,.desert-down ol{margin: 0; padding-left: 1.5rem}
.desert-down li{margin: 0}

/* dash, asterisk, plus list items - use bullet marker */
.desert-down ul{list-style-type: disc}

/* numbered list marker can vary in width (with the quantity of numbers in the list item opening), thus take the width of the widest list item */
.desert-down ol{
    list-style: none;
    display: grid;
    grid-template-columns: max-content 1fr;
    column-gap: .5rem;
    padding-left: 0;
}

/* child combinator (>) means list items will be selected only if they are children of an ordered list */
.desert-down ol > li{
    display: grid;
    grid-template-columns: subgrid;
    grid-column: 1/-1; /* list item span all columns in the grid */
}

/* selects the pseudo-element before a list item's contents, if that list item is a child of an ordered list, and sets the content of this element to the list item's number and appends a dot */
/* ie. the list item in an ordered list is styled with no marker, and one is added, so that the numbers need not be sequential */
.desert-down ol > li::before{
    content: attr(value) ".";
    text-align: right;
    grid-column: 1;
}

/* sets everything else (ie. the content) of ordered list items to the seond column (marker, as set above, in first) */
.desert-down ol > li > * {grid-column: 2}


/*
    LIST INDENTATION LINES
*/

/* alignment and spacing for dash, asterisk, plus, numbered list items */
/* numbered list items align to where the centre of an unordered list bullet would be, as their varying width makes it aligning varying their alignment less feasible */
.desert-down ul > li,
.desert-down ol > li{
    position:relative;
    --dd-marker-axis-x: -.7em; /* x position for centring the lines from/to bullet and checkbox markers */
    --dd-trace-from: .5em; /* vertical spacing below the marker being drawn from */
    --dd-trace-to: .6em; /* vertical spacing above the marker being drawn to */
}

/* spacing for a checkbox marker - different as checkbox is taller than bullet */
.desert-down ul > li.check-list-item{
    --dd-trace-from: .75em;
    --dd-trace-to: .75em;
}

/* spacing for a number marker - different as height different to bullets and checkboxes */
.desert-down ol > li{
    --dd-trace-from: .75em;
    --dd-trace-to: .8em;
}

/* bullet marker to checkbox marker */
.desert-down ul > li:not(.check-list-item):has(+ li.check-list-item){
    --dd-trace-to: .8em;
}

/* checkbox marker to bullet marker */
.desert-down ul > li.check-list-item:has(+ li:not(.check-list-item)){
    --dd-trace-to: .6em;
}

/* bullet, checkbox, number to number */
.desert-down :is(ul,ol):has(+ ol) > li:last-child{
    --dd-trace-to: .85em;
}

/* bullet, checkbox, number to bullet */
.desert-down :is(ul,ol):has(+ ul > li:first-child:not(.check-list-item)) > li:last-child{
    --dd-trace-to: .6em;
}

/* bullet, checkbox, number to checkbox */
.desert-down :is(ul,ol):has(+ ul > li:first-child.check-list-item) > li:last-child{
    --dd-trace-to: .75em;
}

.desert-down ul > li:not(:last-child)::after,
.desert-down ol > li:not(:last-child)::after,
.desert-down :is(ul,ol):has(+ :is(ul,ol)) > li:last-child::after{
    top: max(
        calc(.825em + var(--dd-trace-from)),
        calc(4.125em - var(--dd-trace-to) - 100%)
    );
    top: max(
        calc(.5lh + var(--dd-trace-from)),
        calc(2.5lh - var(--dd-trace-to) - 100%)
    );

    bottom: calc(-.825em + var(--dd-trace-to));
    bottom: calc(-.5lh + var(--dd-trace-to));
}

.desert-down :is(ul,ol):not(:has(+ :is(ul,ol))) > li:last-child::after{
    top: max(
        calc(.825em + var(--dd-trace-from)),
        calc(3.3em - var(--dd-trace-to) - 100%)
    );
    top: max(
        calc(.5lh + var(--dd-trace-from)),
        calc(2lh - var(--dd-trace-to) - 100%)
    );

    bottom: var(--dd-trace-to);
}

.desert-down ul > li::after{
    left: var(--dd-marker-axis-x);
}

.desert-down ol > li::after{
    left: calc(1.5rem + var(--dd-marker-axis-x));
}

.desert-down ul > li::after,
.desert-down ol > li::after{
    content: "";
    position: absolute;
    transform: translateX(-50%);
    background: var(--dd-lines);
    width: 1px;
    pointer-events: none;
}


/*
    CHECKLIST ITEMS
*/
.check-list-item{list-style: none; position: relative}

.checkbox{
    appearance: none;
    -webkit-appearance: none;
    font: inherit;

    position: absolute;
    left: calc(var(--dd-marker-axis-x) - .4em); /* subtract half the checkbox width */
    top: .375em; /* fallback based on 1.65 line height */
    top: calc((1lh - .8em) / 2 - .05em);
    width: .8em;
    height: .8em;
    margin: 0;
    padding: 0;
    border: 1px solid var(--dd-lines);
    border-radius: 3px;
    background: transparent;
    opacity: 1;
    cursor: default;
}

.checkbox:checked{
    background: var(--dd-green);
    border-color: var(--dd-green);
}

/* centre the tick for checked items */
.checkbox:checked::before,
.checkbox:checked::after{
    content: "";
    position: absolute;
    left: calc(50% - .075em);
    top: calc(50% + .1375em);
    transform-origin: .0375em 50%;
    height: .075em;
    border-radius: .0375em;
    background: var(--dd-bg);
}
.checkbox:checked::before{
    width: .25178em;
    transform: translate(-.0375em, -50%) rotate(-135deg);
}
.checkbox:checked::after{
    width: .46391em;
    transform: translate(-.0375em, -50%) rotate(-45deg);
}

/* if check list item contains forward slash, then it requires incomplete/half-checked styling */
.check-list-item[data-check-value="/"] > .checkbox{
    border-color: var(--dd-blue);
    background: linear-gradient(
        to right,
        var(--dd-blue) 50%,
        transparent 50%
    );
}

.check-list-item:has(> .checkbox:checked) > p{
    color: var(--dd-quiet);
}


/*
    THEMATIC BREAKS
*/
.desert-down hr{
    height: 0;
    margin: 0;
    border: 0;
    border-top: 1px solid var(--dd-lines);
}


/*
    INDENTS
*/
.indent{margin: 0; padding-left: 1.5rem}


/*
    CALLOUTS
*/
.callout{
    margin: 0;
    padding: .75rem 1rem;
    --dd-callout: var(--dd-grey); /* default to grey */
    border: 2px solid color-mix(in srgb, var(--dd-callout) var(--dd-colour-border-opacity), transparent);
    border-radius: 6px;
    background: var(--dd-bg-secondary);
    background: color-mix(in srgb, var(--dd-callout) var(--dd-colour-bg-opacity), var(--dd-bg));
}

.callout-title{
    margin: 0;
    color: var(--dd-callout);
    font-weight: 600;
    line-height: 1.5;
    padding-bottom: .5em;
}

/* bold and italic text in callout title shouldn't use their coloured styling, instead inheriting callout title colour */
.desert-down .callout-title :is(strong, em){
    color:inherit;
}

/* map callout titles to colours */
.callout[data-callout="attention" i],
.callout[data-callout="caution" i],
.callout[data-callout="warning" i]{--dd-callout:var(--dd-amber)}

.callout[data-callout="info" i],
.callout[data-callout="note" i],
.callout[data-callout="todo" i]{--dd-callout:var(--dd-blue)}

.callout[data-callout="abstract" i],
.callout[data-callout="summary" i],
.callout[data-callout="tldr" i]{--dd-callout:var(--dd-cyan)}

.callout[data-callout="check" i],
.callout[data-callout="done" i],
.callout[data-callout="good" i],
.callout[data-callout="pass" i],
.callout[data-callout="passed" i],
.callout[data-callout="success" i]{--dd-callout:var(--dd-green)}

.callout[data-callout="cite" i],
.callout[data-callout="quote" i]{--dd-callout:var(--dd-grey)}

.callout[data-callout="bug" i]{--dd-callout:var(--dd-pink)}

.callout[data-callout="example" i],
.callout[data-callout="faq" i],
.callout[data-callout="help" i],
.callout[data-callout="question" i]{--dd-callout:var(--dd-purple)}

.callout[data-callout="bad" i],
.callout[data-callout="danger" i],
.callout[data-callout="error" i],
.callout[data-callout="fail" i],
.callout[data-callout="failed" i],
.callout[data-callout="failure" i],
.callout[data-callout="issue" i],
.callout[data-callout="missing" i]{--dd-callout:var(--dd-red)}

.callout[data-callout="hint" i],
.callout[data-callout="important" i],
.callout[data-callout="tip" i]{--dd-callout:var(--dd-teal)}


/*
    CODE
*/
.desert-down code,
.desert-down pre {
    font-family: monospace;
}

.desert-down :not(pre) > code{
    background: var(--dd-bg-secondary);
    border :1px solid var(--dd-lines);
    border-radius: 3px;
    font-size: .875em;
    padding: .1em .35em;
}

.desert-down pre{
    margin: 0;
    padding: .8rem 1rem;
    border: 1px solid var(--dd-lines);
    border-radius: 6px;
    background: var(--dd-bg-secondary);
    line-height: 1.5;
    overflow-x: auto;
}

.desert-down pre > code{
    background:none;
    border:0;
    font-size:.875em;
    padding:0;
}


/*
    CODE LANGUAGE DECLARATION
*/
.desert-down pre[data-language]{position: relative; padding-top: 1.75rem; padding-bottom: 1.75rem}

.desert-down pre[data-language]::before{
    content: attr(data-language);
    position: absolute;
    top: 0;
    right: 0;
    padding: .1rem .5rem;
    background: var(--dd-bg);
    border-left: 1px solid var(--dd-lines);
    border-bottom: 1px solid var(--dd-lines);
    border-radius: 0 5px 0 5px;
    color: var(--dd-quiet);
    font-size: .7rem;
    line-height: 1.5;
    letter-spacing: .025em;
    user-select: none;
}


/*
    TABLES
*/
.table-scroll{
    margin: 0;
    overflow-x: auto;
}

.desert-down table{
    width: max-content;
    min-width: 100%;
    margin: 0;
    border: 0;
    border-collapse: separate;
    border-spacing: 0;
    border-radius: 6px;
    overflow: hidden; /* ensures cell backgrounds are clipped by border */
    position: relative;
    font-size: .95em;
}

/* table border */
.desert-down table::after{
    content: "";
    position: absolute;
    inset: 0;
    border: 1px solid var(--dd-lines);
    border-radius: inherit;
    pointer-events: none;
}

.desert-down th,
.desert-down td{
    padding: .4rem .7rem;
    border-right: 1px solid var(--dd-lines);
    border-bottom: 1px solid var(--dd-lines);
    max-width: 20rem; /* limit column width
    white-space: normal;
    overflow-wrap: anywhere;
    text-align: left; /* default alignment */
}

.desert-down tbody tr:last-child td{
    border-bottom:0; /* remove bottom row border, as covered by table border */
}

.desert-down thead{
    box-shadow: inset 0 -2px 0 var(--dd-lines);
}

.desert-down thead th{
    background: var(--dd-thead_bg);
    font-weight: 600;
    border-bottom: 2px solid var(--dd-lines);
}

.desert-down .align-left{text-align: left}
.desert-down .align-centre{text-align: center}
.desert-down .align-right{text-align: right}

/* row alternating background styling */
.desert-down tbody tr:nth-child(even){background: var(--dd-bg-secondary)}
.desert-down tbody tr:nth-child(odd){background: var(--dd-bg)}


/*
    MATH
*/
.math-block{
    margin: 0;
    padding: .25rem 0;
    white-space: pre-wrap;
    overflow-x: auto;
    text-align: center;
}

.math-inline{overflow-wrap: break-word}


/*
    EMBEDS
*/
.desert-down img{
    max-width: 100%;
    height: auto;
    border-radius: 6px;
    vertical-align: middle;
}

.embed{max-width: 100%; vertical-align: middle}

.embed-image{height: auto; border-radius: 6px}

.embed-video{width: 100%; height: auto; border-radius: 6px; background: var(--dd-bg-secondary)}
.embed-audio{width: 100%; min-height: 2.5rem}

.embed-pdf{
    display: block;
    width: 100%;
    height: min(80vh,40rem);
    margin: 0;
    border: 1px solid var(--dd-lines);
    border-radius: 6px;
    background: var(--dd-bg-secondary);
}

/*
    SMALL SCREEN OVERRIDES
*/
/* added last to give it higher precedence */
/* when narrower than 40rem, change some of the spacing, and the largest heading sizes */
@media (max-width:40rem){
    .desert-down{padding:1.5rem 1rem 1.5rem}
    .desert-down h1{font-size:2.25rem}
    .desert-down h2{font-size:2rem}
}
"#;
