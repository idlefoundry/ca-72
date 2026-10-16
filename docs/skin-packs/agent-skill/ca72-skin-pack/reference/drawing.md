# The drawing, pictures and fonts

*CC0 1.0. Checked against the CA-72's `skin-packs` prototype, October 2026; `docs/skin-packs.md`
is the authority.*

## The drawing, `art/panel.svg`

Give it `viewBox="0 0 3108 925"`. It covers the wooden strip (y 0 to 130) and the face (y 130
to 925), and is stretched over them. **Its y is the layout's y + 130**: a control at layout
`[777, 360]` is at `777, 490` in the drawing. Its x is the layout's x.

It is drawn over the panel's surfaces (the wood, the face, the realistic look's trim and
shadows) and under the print, controls, screws and jacks. So:

- A face colour is a `rect` from y 130 to 925. Leave y 0 to 130 alone (or keep it
  translucent) to keep the wood showing; fill it to replace the wood.
- Bands, panels and frames behind groups of controls are the usual way to give a look its
  character. Line them up with the layout's rows (add 130).
- Keep enough contrast for the print. The plug-in prints legends, rules and dial numbers in
  light ink on a dark face in the classic look: a light drawing makes them hard to read.
  Check both previews.

### Allowed

`svg`, `g`, `defs`, `title`, `desc`, `metadata`, `path`, `rect`, `circle`, `ellipse`, `line`,
`polyline`, `polygon`, `linearGradient`, `radialGradient`, `stop`, `clipPath`, `mask`,
`pattern`, `symbol`, `use`, `image`, `filter`, `feGaussianBlur`, `feOffset`, `feFlood`,
`feComposite`, `feMerge`, `feMergeNode`, `feColorMatrix`, `feBlend`, `feDropShadow`, in SVG's
namespace. Style with presentation attributes (`fill`, `stroke`, `opacity`, ...).

### Refused (`svg.refused`)

A DTD or entities, processing instructions, `<script>`, `<foreignObject>`, animation,
**`<text>` and `<tspan>`**, `<style>` (by that name in any namespace, anywhere), markers,
interactive elements, `on...` attributes, anything of SVG's inside `<metadata>` (which may
hold only other namespaces' data, such as a licence's RDF), a pattern filled with another
pattern, an `id` given to two elements (or to anything not SVG's, or inside `<metadata>`),
and any reference outside the file: only `href="#name"`, `url(#name)` and `<image>`
with `data:image/png;base64,...`. Drawing programs often add `<style>`, editor namespaces and
fonts: write the SVG yourself, or strip those out.

### Limits

20,000 elements drawn (each reference, a `use`, `href` or `url(#name)`, counts as all it
repeats; one that repeats itself is refused), 200,000 path segments, 32 filter primitives, a
blur's deviation of at most 64, 4096 pattern tiles over all that patterns fill (and 20,000
elements drawn in those tiles), and embedded pictures within the picture limits.

### What it costs to draw

Counts don't bound the cost: a few long lines across the panel cost more than thousands of
small shapes. The check estimates the work at the size the drawing is used (the panel's
drawing 3000 pixels across, a picture at its slot's size) in pixel-equivalents, and refuses
past **60 million**:

| Work | Counts |
|---|---|
| a pixel painted (a fill's box; a stroke's length by its width) | 1 |
| a pixel row a path's edges cross | 100 |
| a pixel of a layer (a group's `opacity`, a clip, a mask) | 6 |
| a pixel of a filter's region | 10 for each primitive |

Edge rows are counted for every edge, by how far it climbs: the sum, over all edges, of the
pixel rows each spans (a stroke's twice over). So tall or diagonal lines are what cost:
hundreds of stripes or hatching running down or across the panel add up fast, while
horizontal edges cost almost nothing and short shapes little. Keep tall and diagonal edges
few; put `opacity` on shapes rather than on big groups; keep filters to small regions;
prefer gradients to blurs and to hatching. Run `svg_cost` (SKILL.md, step 4) on every drawing you
make: it prints the cost part by part, with each costly kind left out in turn.

A timed test drawing (250 ms, `svg.slow`) is a backstop. If it is skipped because another
drawing's test is running, `svg.untested` notes it; the counted cost still bounds the drawing.

Write the SVG by hand or with a script, and keep it readable: name gradients and groups, and
put a `<title>` saying what it is.

## Pictures (`base = "realistic"` only)

```toml
[look]
base = "realistic"

[look.pictures]
knob = "pictures/knob.svg"
face = "pictures/face.png"
```

Each slot takes a PNG or an SVG; an SVG is drawn at the built-in picture's size, so give it
that `viewBox`. A picture is drawn at the built-in one's proportions (another shape gets a
`look.picture` note and is stretched). Frame the part as the built-in one is framed, centred
on a transparent ground; round parts that turn should be lit evenly all round, because the
panel lays its own lamp over them. The slots and their sizes are in `docs/skin-packs.md`,
"Pictures"; the framing of each is described in
`crates/ca72-panel/assets/worn/README.md`. Read that description, but **don't open, copy,
trace or edit the built-in pictures themselves**: they are under the GPL, and a byte-for-byte
copy is flagged (`asset.official-copy`).

A coding agent draws slot pictures best as SVG (a knob from above: concentric circles,
radial gradients, a dot at twelve o'clock). A picture made with an image generator is
`origin = "generated"` with that generator named as the `tool`.

Picture limits: 4096 pixels a side, **16 million pixels in all**, pictures embedded in SVGs
counted too (`pack.pixels`), 8 MiB a file. A PNG or SVG picture at other proportions than its
slot is stretched and noted (`look.picture`).

## Fonts

```toml
[look]
fonts = { regular = "fonts/Face-Regular.ttf", bold = "fonts/Face-Bold.ttf" }
```

Prefer none: the built-in typeface (URW Gothic) suits the panel. Use a font only when the
user supplies it or names one whose licence lets a pack carry and sell it (OFL-1.1,
Apache-2.0, CC0-1.0, MIT, Unlicense, Bitstream-Vera), from its maker's own source. Then:

- give it an `[[assets]]` entry with `origin = "third-party"`, `source` (where it came from)
  and `license` (the font's licence, e.g. `"OFL-1.1"`);
- put its licence text in the pack (`OFL.txt`), and a `[[notices]]` entry pointing at it;
- don't rename or change a font whose licence reserves its name;
- one face per file, `.otf` or `.ttf`, at most 4 fonts, 4 MiB each.

The font must have a glyph for every legend and title, "CA-72" (the plate's) and everything
the plug-in prints itself: printable ASCII, "–" and "…". Otherwise the built-in typeface is
used and a `look.fonts` review finding says why. A legend that runs off the panel in the font
is drawn cut off (`look.fit`, a review): the font is still used, and the layout is what to
fix. See what a font says of itself and what it lacks before using it:

```bash
cd "$CA72" && cargo run -p skin-pack --example font_info -- fonts/Face-Regular.ttf "EVERY LEGEND"
```

It prints its names (family, copyright, licence and link, trademark), its maker, its
embedding permission, the printable ASCII it lacks and whether it has each character of the
text given.
