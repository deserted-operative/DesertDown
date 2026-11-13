#!/bin/sh

# Regenerates HTML output code derived from other files
#
# 1. math.css - Temml's Latin Modern stylesheet, with its fonts embedded using `data:`
# 2. CSP hashes (Base64 of SHA-256) of the scripts and styling added to HTML documents:
#    - folding.js, folding.css                        -> folding.rs
#    - temml/temml.min.js, math.js, math.css          -> math.rs
#    - DesertdDown stylesheet of each theme and width -> styling.rs
#
# Run after changing any of those files, or the styling in styling.rs
#
# Requires cargo, openssl, and awk

set -eu

cd "$(dirname "$0")/.."

SOURCE_DIRECTORY=src/parser/ast_to_html
TEMML_DIRECTORY=$SOURCE_DIRECTORY/temml

WORKING_DIRECTORY=$(mktemp -d)
trap 'rm -rf "$WORKING_DIRECTORY"' EXIT

export LC_ALL=C

fail() {
    echo "update_hashes: $1" >&2
    exit 1
}

# prints the Base64 of the SHA-256 of a file
hash_of() {
    openssl dgst -sha256 -binary "$1" | openssl base64 -A
}

# replaces file content with content of another file, only if they differ (so that unchanged files are left untouched)
replace_if_changed() {
    if cmp -s "$1" "$2"; then
        return
    fi

    cat "$1" > "$2"
    echo "updated  $2${3:+ ($3)}"
}

# set_hash <file> <text> <hash>
# sets the string that comes straight after the text passed in (which must be in the file exactly once) to the hash
set_hash() {
    [ "$(grep -c -F -- "$2\"" "$1")" = 1 ] || fail "expected exactly one '$2\"' in $1"

    awk -v text="$2\"" -v hash="$3" '
        {
            start = index($0, text)

            if (start) {
                after = substr($0, start + length(text))
                $0 = substr($0, 1, start + length(text) - 1) hash substr(after, index(after, "\""))
            }

            print
        }
    ' "$1" > "$WORKING_DIRECTORY/with_hash"

    replace_if_changed "$WORKING_DIRECTORY/with_hash" "$1" "$2"
}

# block_of <document> <tag> <n>
# prints the contents of the n'th element with the tag passed in (as written out by the program: <tag>, no attributes), or fails if there isn't one
block_of() {
    awk -v opener="<$2>" -v closer="</$2>" -v n="$3" '
        { document = document $0 "\n" }

        END {
            for (i = 1; i <= n; i++) {
                start = index(document, opener)

                if (!start) {
                    exit 1
                }

                document = substr(document, start + length(opener))
                end = index(document, closer)
                block = substr(document, 1, end - 1)
                document = substr(document, end + length(closer))
            }

            printf "%s", block
        }
    ' "$1"
}

# writes out a document for the input with the program, with the options passed in
write_out_document() {
    cargo run --release --quiet -- "$WORKING_DIRECTORY/input.md" --html "$@" > "$WORKING_DIRECTORY/document.html"
}


# 1. math.css
# the stylesheet refers to each font once, as a url() - each is replaced with the font itself, as a `data:`` URI
awk -v directory="$TEMML_DIRECTORY" -v quote="'" '
    function data_uri(font,    command, base64) {
        command = "openssl base64 -A -in \"" directory "/" font "\""
        command | getline base64
        close(command)

        return "data:font/woff2;base64," base64
    }

    function embed(line, reference, font,    start) {
        start = index(line, reference)

        if (!start) {
            return line
        }

        embedded[font]++

        return substr(line, 1, start - 1) "url(" quote data_uri(font) quote ")" substr(line, start + length(reference))
    }

    BEGIN {
        print "/* Generated from temml/Temml-Latin-Modern.css, with its fonts (temml/Temml.woff2, temml/latinmodernmath.woff2) embedded - see temml/README.md */"
    }

    {
        $0 = embed($0, "url(" quote "Temml.woff2" quote ")", "Temml.woff2")
        $0 = embed($0, "url(" quote "./latinmodernmath.woff2" quote ")", "latinmodernmath.woff2")

        print
    }

    END {
        if (embedded["Temml.woff2"] != 1 || embedded["latinmodernmath.woff2"] != 1) {
            exit 1
        }
    }
' "$TEMML_DIRECTORY/Temml-Latin-Modern.css" > "$WORKING_DIRECTORY/math.css" \
    || fail "expected each font to be referred to exactly once in $TEMML_DIRECTORY/Temml-Latin-Modern.css"

replace_if_changed "$WORKING_DIRECTORY/math.css" "$SOURCE_DIRECTORY/math.css"


# 2. hashes of the files that are included as-is in documents
set_hash "$SOURCE_DIRECTORY/folding.rs" 'pub const FOLDING_SCRIPT_HASH: &str = ' "$(hash_of "$SOURCE_DIRECTORY/folding.js")"
set_hash "$SOURCE_DIRECTORY/folding.rs" 'pub const FOLDING_STYLING_HASH: &str = ' "$(hash_of "$SOURCE_DIRECTORY/folding.css")"

set_hash "$SOURCE_DIRECTORY/math.rs" 'pub const TEMML_SCRIPT_HASH: &str = ' "$(hash_of "$TEMML_DIRECTORY/temml.min.js")"
set_hash "$SOURCE_DIRECTORY/math.rs" 'pub const MATH_SCRIPT_HASH: &str = ' "$(hash_of "$SOURCE_DIRECTORY/math.js")"
set_hash "$SOURCE_DIRECTORY/math.rs" 'pub const MATH_STYLING_HASH: &str = ' "$(hash_of "$SOURCE_DIRECTORY/math.css")"


# 3. hashes of the stylesheets, each taken from a document written out with its theme and width (its first style element)
printf 'Some *text*, and some maths: $x^2$\n\n- a list item\n  with a continuation\n' > "$WORKING_DIRECTORY/input.md"

for theme in Light Dark Reactive; do
    for width in Fixed Full; do
        theme_option=$(echo "$theme" | tr '[:upper:]' '[:lower:]')
        width_option=$(echo "$width" | tr '[:upper:]' '[:lower:]')

        write_out_document "--theme=$theme_option" "--width=$width_option"

        block_of "$WORKING_DIRECTORY/document.html" style 1 > "$WORKING_DIRECTORY/stylesheet.css" \
            || fail "no stylesheet in the document written out with --theme=$theme_option --width=$width_option"

        set_hash "$SOURCE_DIRECTORY/styling.rs" "(Theme::$theme, OutputWidth::$width) => " "$(hash_of "$WORKING_DIRECTORY/stylesheet.css")"
    done
done


# 4. check - with the hashes updated, all scripts and styling in a document should be allowed by the CSP
check_document() {
    write_out_document "$@"

    for tag in style script; do
        n=1

        while block_of "$WORKING_DIRECTORY/document.html" "$tag" "$n" > "$WORKING_DIRECTORY/block"; do
            grep -q -F -- "'sha256-$(hash_of "$WORKING_DIRECTORY/block")'" "$WORKING_DIRECTORY/document.html" \
                || fail "$tag $n of the document written out with $* isn't allowed by its CSP"

            n=$((n + 1))
        done
    done
}

for theme_option in light dark reactive; do
    for width_option in fixed full; do
        check_document "--theme=$theme_option" "--width=$width_option"
    done
done

check_document --foldable --math
check_document --foldable --math --allow-links

echo "hashes are up to date"
