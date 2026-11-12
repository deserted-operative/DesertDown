# Temml

[Temml](https://temml.org) converts LaTeX to MathML, which browsers can render natively. DesertDown includes Temml in HTML when the `--math` flag is supplied.

The following files are from Temml **0.14.0**, as found at https://github.com/ronkok/Temml.

| File | Location in Temml repository |
|---|---|
| `temml.min.js` | `dist/` |
| `Temml-Latin-Modern.css` | `dist/` |
| `Temml.woff2` | `dist/` |
| `latinmodernmath.woff2` | `site/assets/` |
| `LICENSE-Temml.txt` | `LICENSE` |
| `GUST-FONT-LICENSE.txt`, `README-Latin-Modern-Math.txt`, `MANIFEST-Latin-Modern-Math.txt` | `site/assets/` |

## Updating Temml

1. Replace the files in the table above with those from the updated version of Temml.
2. Run `scripts/update_hashes.sh` (from anywhere in the repository). It regenerates `../math.css` (ie. `Temml-Latin-Modern.css`, with the fonts embedded using `data:`), and updates `TEMML_SCRIPT_HASH`, `MATH_SCRIPT_HASH`, and `MATH_STYLING_HASH` in `../math.rs`, along with the other CSP hashes. It also updates `MATH_SCRIPT_HASH` whenever `../math.js` changes.
