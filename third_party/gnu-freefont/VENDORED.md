# Vendored: GNU FreeFont's notes (FreeSerif, a subset)

The note values printed beside oscillators 2's and 3's TIMED keys (decisions.md R-OSC-TIMED):
the panel's typeface has no musical signs but the eighth note. The plug-in reads these glyphs'
outlines from the font (`crates/ca72-panel/src/notes.rs`). GNU FreeFont is under the GNU
General Public License, version 3 or later, with a font exception (`README`), compatible with
the plug-in's GPL-3.0-or-later. The music fonts under the SIL Open Font License alone (Bravura,
Leland, Noto Music) were compared too, and are not used: the plug-in builds no OFL font into its
binaries (decisions.md R22, R-SKIN-PACKS).

| File | Licence | Copied from |
|---|---|---|
| `FreeSerif-notes.otf` | GNU General Public License 3.0 or later, with a font exception (`README`, `COPYING`) | [GNU FreeFont](https://www.gnu.org/software/freefont/) `freefont-otf-20120503.tar.gz` (ftp.gnu.org/gnu/freefont), `FreeSerif.otf`, subset (below) |
| `COPYING`, `README`, `AUTHORS` | (the licence, the licensing statement with its exception, and the authors) | the same archive |

**Modified: a subset.** Only the glyphs used, their outlines unchanged: the double whole note to
the thirty-second note and the augmentation dot (U+1D15C to U+1D162, U+1D16D), made with
fontTools 4.67.0:

    pyftsubset FreeSerif.otf --unicodes="U+1D15C-1D162,U+1D16D" --output-file=FreeSerif-notes.otf \
        --name-IDs='*' --name-languages='*' --notdef-outline --no-hinting --drop-tables+=GSUB,GPOS,GDEF

The font's name table (its copyright, "Copyright 2002, 2003, 2005, 2008, 2009, 2010, 2012 GNU
Freefont contributors", and its licence) is kept.

SHA-256: the archive `freefont-otf-20120503.tar.gz`
3a6c51868c71b006c33c4bcde63d90927e6fcca8f51c965b8ad62d021614a860; its `FreeSerif.otf`
aab33b5d0efd450d3ac3153b176f542315e171db79e494aa287cd940ae16231b (the same as Arch Linux's
gnu-free-fonts 20120503); `FreeSerif-notes.otf`
a319b8b94bac423b4001ccedef3db1260ce7475291c7de720a0c7344f6db8ff9. SHA-256 of the tree (every
file but this one, sorted by path, as
`find . -type f ! -name VENDORED.md -print0 | sort -z | xargs -0 sha256sum | sha256sum` from
this directory): `7d4cf907c58497060d421afec033de5f0c26641aa16610cc4a6ac847fe4f8023`.

To update: subset a newer release's `FreeSerif.otf` the same way, run `cargo test -p
ca72-panel` (the notes' proportions, the panel as approved), and record the release and hashes
here.
