# Vendored: TeX Gyre Adventor

The panel's font, downloaded with the owner's approval on 2026-09-30 for DAW-v3's panel
(its D144), so the panel is drawn with the same letters on every platform. Not modified.

| File | Licence | Copied from |
|---|---|---|
| `texgyreadventor-regular.otf`, `texgyreadventor-bold.otf` | GUST Font License (the LaTeX Project Public License 1.3c or later) | CTAN, `fonts/tex-gyre/opentype/` |
| `GUST-FONT-LICENSE.txt`, `README-TeX-Gyre-Adventor.txt` | (the licence and the fonts' readme) | CTAN, `fonts/tex-gyre/doc/` |

TeX Gyre Adventor 2.501 (8 May 2018), by the GUST e-foundry, is based on URW Gothic L,
whose bold is URW Gothic's Demi. The italics are left out: the panel does not use them.
`crates/ca72-panel` builds both files into the plug-in (`src/fonts.rs`).

SHA-256 of the tree (every file but this one, sorted by path, as
`find . -type f ! -name VENDORED.md -print0 | sort -z | xargs -0 sha256sum | sha256sum`
from this directory):
`dcbd1622e6bd544259360ff19ca1e520b6fc2490bb92068e9c7bb9d291a2c8c6`.

To update: download the same files from CTAN, run `cargo test -p ca72-panel` (the panel as
drawn against the approved design, `tests/approved.rs`), and record the version and hash.
