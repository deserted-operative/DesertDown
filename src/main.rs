use desert_down::parser::{self, LinkPermissions, OutputWidth, Theme};
use std::{env, fs, path::Path, process};

const USAGE_INFO: &str = "Usage: cargo run -- <input-file> [--html] [--theme light|dark] [--width fixed|full] [--allow-links] [--fragment]";

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

    // read CL args
    let mut arguments = env::args().skip(1);

    // process CL args
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--html" => should_output_html = true,

            "--fragment" => should_output_fragment = true,

            "--allow-links" => links_allowed = LinkPermissions::Allowed,

            // --theme dark, with --light and --dark below as allowed shorthands
            "--theme" => {
                let Some(name) = arguments.next() else {
                    eprintln!("--theme requires a theme: light or dark");
                    eprintln!("{USAGE_INFO}");
                    process::exit(1);
                };

                let Some(chosen) = Theme::parse_potential_theme_string(&name) else {
                    eprintln!("Unknown theme {name:?}, expected light or dark");
                    eprintln!("{USAGE_INFO}");
                    process::exit(1);
                };

                theme = chosen;
            }

            "--light" => theme = Theme::Light,
            "--dark" => theme = Theme::Dark,

            // --width full, with --fixed-width and --full-width below as allowed shorthands
            "--width" => {
                let Some(name) = arguments.next() else {
                    eprintln!("--width needs a width: fixed or full");
                    eprintln!("{USAGE_INFO}");
                    process::exit(1);
                };

                let Some(chosen) = OutputWidth::parse_potential_output_width_string(&name) else {
                    eprintln!("Unknown width {name:?}, expected fixed or full");
                    eprintln!("{USAGE_INFO}");
                    process::exit(1);
                };

                width = chosen;
            }

            "--fixed-width" => width = OutputWidth::Fixed,
            "--full-width" => width = OutputWidth::Full,

            _ if argument.starts_with('-') => {
                eprintln!("Unknown option {argument:?}");
                eprintln!("{USAGE_INFO}");
                process::exit(1);
            }

            _ if input_file_path.is_none() => input_file_path = Some(argument),

            // a second file, which there is nothing sensible to do with
            _ => {}
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

    ast.ast_to_html_document_to_stdout(&input, theme, width, links_allowed, title);
}
