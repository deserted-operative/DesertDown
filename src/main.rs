use desert_down::parser::{self, Folding, LinkPermissions, MathRendering, OutputWidth, Theme};
use std::{env, fs, path::Path, process};

const USAGE_INFO: &str = "Usage: cargo run -- <input-file> [--html] [--theme=light|dark|reactive] [--width=fixed|full] [--allow-links] [--foldable[=headings,list-items,callouts]] [--math] [--fragment]";

fn main() {
    // for CL args
    let mut input_file_path = None; // path of input to parse
    let mut should_output_html = false;
    let mut should_output_fragment = false; // whether to write out full document with header, or just fragment. Defaults to full document

    // default is light
    let mut theme = Theme::Light;

    // default is fixed, not full, width
    let mut width = OutputWidth::Fixed;

    // reject links and embeds by default, for security
    let mut links_allowed = LinkPermissions::Blocked;

    // no folding by default
    let mut folding = Folding::DISABLED;

    // math is written out as LaTeX source code by default
    let mut math = MathRendering::Disabled;

    // read CL args
    let arguments = env::args().skip(1);

    // process CL args
    for argument in arguments {
        let (option, value) = match argument.split_once('=') {
            Some((option, value)) if option.starts_with("--") => (option, Some(value)),
            _ => (argument.as_str(), None),
        };

        // WARNING: scripts/update_hashes.sh writes out documents with --html, --theme=, --width=, --foldable, --math, and --allow-links
        // renaming these will require changes to the script
        match (option, value) {
            ("--html", None) => should_output_html = true,

            ("--fragment", None) => should_output_fragment = true,

            ("--allow-links", None) => links_allowed = LinkPermissions::Allowed,

            ("--math", None) => math = MathRendering::Enabled,

            // --foldable applies to headings, callouts, and list items
            // --foldable=headings,callouts,list-items applies to only those listed
            ("--foldable", None) => folding = Folding::ALL,

            ("--foldable", Some(folding_targets)) => {
                let Some(chosen) = Folding::parse_potential_folding_targets(folding_targets) else {
                    eprintln!(
                        "Unknown folding {folding_targets:?}, expected a comma separated list of headings, and/or list-items, and/or callouts"
                    );
                    eprintln!("{USAGE_INFO}");
                    process::exit(1);
                };

                folding = chosen;
            }

            // --theme=dark, with --light, --dark, and --reactive below as allowed shorthands
            ("--theme", None) => {
                eprintln!(
                    "--theme requires a theme: --theme=light, --theme=dark, or --theme=reactive"
                );
                eprintln!("{USAGE_INFO}");
                process::exit(1);
            }

            ("--theme", Some(name)) => {
                let Some(chosen) = Theme::parse_potential_theme_string(name) else {
                    eprintln!("Unknown theme {name:?}, expected light, dark, or reactive");
                    eprintln!("{USAGE_INFO}");
                    process::exit(1);
                };

                theme = chosen;
            }

            ("--light", None) => theme = Theme::Light,
            ("--dark", None) => theme = Theme::Dark,
            ("--reactive", None) => theme = Theme::Reactive,

            // --width=full, with --fixed-width and --full-width below as allowed shorthands
            ("--width", None) => {
                eprintln!("--width needs a width: --width=fixed or --width=full");
                eprintln!("{USAGE_INFO}");
                process::exit(1);
            }

            ("--width", Some(name)) => {
                let Some(chosen) = OutputWidth::parse_potential_output_width_string(name) else {
                    eprintln!("Unknown width {name:?}, expected fixed or full");
                    eprintln!("{USAGE_INFO}");
                    process::exit(1);
                };

                width = chosen;
            }

            ("--fixed-width", None) => width = OutputWidth::Fixed,
            ("--full-width", None) => width = OutputWidth::Full,

            // includes options given a value they don't take, eg. --html=yes
            _ if argument.starts_with('-') => {
                eprintln!("Unknown option {argument:?}");
                eprintln!("{USAGE_INFO}");
                process::exit(1);
            }

            _ if input_file_path.is_none() => input_file_path = Some(argument),

            // a second file, which there is nothing sensible to do with
            _ => {
                eprintln!("Unexpected argument {argument:?}, only one input file can be given");
                eprintln!("{USAGE_INFO}");
                process::exit(1);
            }
        }
    }

    let Some(path) = input_file_path else {
        eprintln!("{USAGE_INFO}");
        process::exit(1);
    };

    // read input from file
    let input = match fs::read_to_string(&path) {
        Ok(input) => input,

        Err(error) => {
            eprintln!("Failed to read {path:?}: {error}");
            process::exit(1);
        }
    };

    // prepare AST for input
    let ast = parser::parse_input(&input);

    // if not HTML, write out AST to standard output
    if !should_output_html {
        ast.ast_to_stdout(&input);
        return;
    }

    // if a fragment, write out the HTML without the document header
    if should_output_fragment {
        ast.ast_to_html_to_stdout(&input, links_allowed);
        return;
    }

    // not a fragment, so write out full HTML document
    // use file name as HTML document title
    let title = Path::new(&path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");

    ast.ast_to_html_document_to_stdout(&input, theme, width, links_allowed, folding, math, title);
}
