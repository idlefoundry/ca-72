# Skin packs

*This document, and the starter pack in [skin-packs/starter](skin-packs/starter), are dedicated to
the public domain under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/): follow
them, copy them and build on them under any terms you like. The CA-72 itself, its code and its
official artwork are under the GNU General Public License, version 3 or later.*

**Status: a prototype (decisions.md R-SKIN-PACKS).** The format, the checks and the CA-72's
support for packs exist on the `skin-packs` branch, with tests. Nothing has been released:
there is no store, no seller agreement and no review process yet. The format may still change
before its first release.

A skin pack changes how the CA-72's editor looks: where each control is on the panel, how each
looks, every legend, rule and title printed on it, the pictures its realistic look is made of,
a drawing over the panel, and its typeface. A pack holds **artwork and declarative data only,
never code**. The plug-in's own open-source renderer draws the controls and decides what each
one does. A pack cannot change the sound, the presets, MIDI, automation or what a session saves.

The CA-72's own looks, CLASSIC and REALISTIC, are packs in this format. They are built into the
plug-in under the GPL; `cargo run -p ca72-panel --example export_pack -- realistic <folder>`
writes one out as a folder to study. Its id (`com.idlefoundry.*`) is reserved for the built-in
packs, so a copy must be given another id before it will load.

## Licences: the pack's are its own

- **A pack's licence is its creator's choice**, named in its manifest: an open licence (CC0,
  CC BY, CC BY-SA, the GPL, MIT, ...), one with restrictions (CC's NonCommercial or
  NoDerivatives), or the creator's own terms (`LicenseRef-...`, the text in the pack).
- **Price and licence are independent.** An open pack may be sold, and a proprietary one given
  away. A pack never states a price: that belongs to the store. A manifest with `price`,
  `currency`, `sku`, `license_key` or `drm` is rejected.
- **The plug-in's licence does not apply to a pack, and a pack's does not apply to the
  plug-in.** Every pack keeps its licence, creators, each asset's provenance and its
  third-party notices in its own manifest and folder.
- **Don't copy the CA-72's artwork into a pack under other terms.** Its pictures, its drawn
  panel and its official layout (`crates/ca72-panel/packs/official/layout.toml`) are under the
  GPL. A pack made from them is under the GPL too, unless Idle Foundry gives additional
  permission. If a pack under another licence contains a byte-for-byte copy of one of the
  plug-in's pictures, the check flags it for review. Start from the [CC0
  starter](skin-packs/starter) instead.
- **Fonts, stock pictures and other makers' branding** go in a pack only when their licences
  permit that distribution, including sale if the pack is sold. A pack never uses a trademark
  it has no right to: no instrument maker's names or logos.
- **Declarations are not clearance.** A seller declares the right to distribute every asset
  under the pack's licence, and authorizes the marketplace to license the pack to its
  customers. The checks record these as declared, never as cleared: a review decides.

## Installing

A pack is a folder. Put it in the `Skins` folder of Idle Foundry's shared folder, one folder
per pack:

| System | Folder |
|---|---|
| macOS | `~/Library/Application Support/Idle Foundry/Skins/` |
| Windows | `%APPDATA%\Idle Foundry\Skins\` |
| Linux | `~/.local/share/Idle Foundry/Skins/` (or `$XDG_DATA_HOME/Idle Foundry/Skins/`) |

The settings key on the editor's rail opens its menu, which lists the installed packs under
LOOK after REALISTIC and CLASSIC. A pack that cannot be used is shown with "(CANNOT BE USED)";
the reasons are in the plug-in's log. No store login or network check is needed to use an
installed pack, and none is made.

The look chosen is a setting for the computer (`settings.toml`), not something the session
saves. If the chosen pack is missing or refused, the editor draws the pack's base look
(REALISTIC or CLASSIC) and keeps the setting, so the pack is used again once it is back. An
older CA-72 reads only the base look.

## Checking a pack

```bash
cargo run --release -p ca72-panel --example check_pack -- path/to/pack preview.png
```

This prints what the checks found and draws the panel in the pack's look, in both lettering
sizes. Each finding has a stable code (`svg.refused`, `layout.overlap`, ...) and one of three
severities:

| Severity | What it means |
|---|---|
| **Rejected** | The plug-in will not load the pack. |
| **Review** | The pack loads for its owner, but a marketplace's review must look at this before listing it. |
| **Note** | For the record: its licence's terms, its declarations as declared, a font's vendor. |

## The folder

```
night-panel/
  skin.toml            the manifest (required)
  LICENSE.txt          the pack's licence text
  layout.toml          where everything is (optional: the official layout otherwise)
  art/panel.svg        a drawing over the panel (optional)
  pictures/knob.png    pictures replacing the realistic look's (optional)
  fonts/Face-Bold.otf  a typeface (optional)
  NOTICES.txt          third-party notices (as their licences ask)
  README.md            anything you like
```

The folder is read as untrusted input:

- **Allowed types:** `.toml`, `.txt` and `.md` (UTF-8 text), `.png`, `.svg`, `.otf` and `.ttf`,
  with lower-case extensions. Anything else is rejected, as is code or an archive whatever
  its name (programs, libraries, scripts, WebAssembly, ZIP files).
- **Names:** ASCII letters, digits, `-`, `_` and `.`. A name must not start with a dot, end with
  one, or be a Windows device name (`con`, `nul`, ...). No two names may differ only in case.
  At most three folders deep, and paths of at most 128 bytes. `.DS_Store`, `Thumbs.db`,
  `desktop.ini` and `__MACOSX` are ignored.
- **No links** of any kind, and no special files.
- **Limits:** 256 files; 8 MiB per file, 256 KiB per text file, 4 MiB per font; 64 MiB in all.
  Pictures at most 4096 pixels a side and 48 million pixels in all. At most 4 fonts.

## The manifest, `skin.toml`

```toml
format = 1
id = "org.example.night-panel"   # lower case, dotted, yours; com.idlefoundry.* is reserved
name = "NIGHT PANEL"             # as the settings menu lists it
version = "1.0.0"                # major.minor.patch
plugin = "ca-72"
summary = "A dark panel."        # optional
license = "CC-BY-4.0"            # an SPDX expression, or LicenseRef-<name>
license_files = ["LICENSE.txt"]  # required for a LicenseRef-, expected for the rest

[[creators]]
name = "A. Maker"
role = "artwork"                 # optional
url = "https://example.org"      # optional

[[assets]]                       # one for each file but the manifest, licences, notices and .md
path = "pictures/knob.png"
origin = "original"              # original | generated | derived | third-party
author = "A. Maker"              # optional
tool = "..."                     # generated: the tool and its version
source = "..."                   # derived or third-party: the work it came from
license = "CC0-1.0"              # if not the pack's own (derived or third-party: required)
sha256 = "..."                   # optional; checked if given
note = "..."                     # optional

[[notices]]                      # a third-party work's notice, as its licence asks
name = "Some Sans"
license = "OFL-1.1"
file = "NOTICES.txt"

[declarations]                   # a seller's; needed before a marketplace lists the pack
rights_to_distribute = true
authorizes_customer_licensing = true
agreement = "2027-01"            # the seller agreement's version, once there is one
declared_by = "A. Maker"
declared_on = "2026-10-16"

[look]                           # the CA-72's table: see below
base = "classic"
```

Unknown fields are rejected. **Licences known to the checks:** CC0-1.0; CC-BY-3.0, CC-BY-4.0,
CC-BY-SA-3.0, CC-BY-SA-4.0; CC-BY-NC-4.0, CC-BY-NC-SA-4.0, CC-BY-ND-4.0, CC-BY-NC-ND-4.0;
GPL-2.0-only, GPL-2.0-or-later, GPL-3.0-only, GPL-3.0-or-later; AGPL-3.0-only,
AGPL-3.0-or-later; LGPL-3.0-or-later; MPL-2.0; MIT; Apache-2.0; BSD-2-Clause, BSD-3-Clause;
ISC; Unlicense; OFL-1.1, OFL-1.1-RFN, OFL-1.1-no-RFN; Bitstream-Vera; with `AND`, `OR`,
parentheses, and `WITH Font-exception-2.0`. A creator's own terms are `LicenseRef-` followed by
a name, with their text in the pack.

**For review:** every file without provenance; every generated asset (whether a picture made
with a generator is protected, and so can be licensed, differs between countries); a derived
or third-party asset without its source or licence; a pack with no creators or no
declarations.

## The look, `[look]`

```toml
[look]
base = "realistic"           # how the controls are drawn: realistic (pictures) or classic (drawn)
layout = "layout.toml"       # optional: the official layout if absent
art = "art/panel.svg"        # optional: drawn over the panel's surfaces, under its print
fonts = { regular = "fonts/Face-Regular.otf", bold = "fonts/Face-Bold.otf" }  # optional
face_tint = [1.03, 1.0, 0.94] # optional: the face picture's channels scaled (0 to 2)

[look.pictures]              # optional: realistic only; pictures replacing the built-in ones
knob = "pictures/knob.png"
walnut = "pictures/walnut.svg"
```

- **The drawing** (`art`) is stretched over the panel: the wooden strip and the face, 3108 by
  925 units (give it `viewBox="0 0 3108 925"`). It is drawn over the panel's surfaces (the wood,
  the face, the realistic look's trim and shadows) and under its print, controls, screws and
  jacks. Nothing outside the panel is drawn. The strip below the panel, which holds the
  plug-in's own controls, is not part of a pack's drawing.
- **The typeface** prints the panel's legends, titles, tips and menus. It is used only if it
  has a glyph for every legend and keeps every legend on the panel in both lettering sizes.
  Otherwise the built-in typeface (URW Gothic) is used and the pack goes to review. Without a
  `bold`, the regular is used for both.

### Pictures

`base = "realistic"` draws its parts from pictures. A pack may replace any of them by slot,
with a PNG, or with an SVG drawn at the built-in picture's size. A picture is drawn at the
built-in one's proportions. Lay out your pictures as the built-in ones are laid out (each
described in `crates/ca72-panel/assets/worn/README.md`): the same framing, the part centred,
on a transparent ground. Round parts that turn (knobs, pointers, the bezels) should be lit
evenly all round, because the panel lays its own fixed lamp over them.

| Slot | What | Built-in size |
|---|---|---|
| `walnut` | The wood of the top strip and the rail, grain along it | 1400 × 788 |
| `face` | The face's surface, tiled 640 units across | 1024 × 1024 |
| `knob`, `knob-big` | A knob from above, its dot at twelve o'clock; its skirt 97.8 % (big: 98.9 %) of the picture, its cap 57.8 % (big: 65.3 %) | 512 × 512 |
| `pointer` | A selector's pointer knob, the pointer at twelve o'clock reaching the picture's edge; its body 78 %, its cap 43 % | 512 × 512 |
| `rocker-blue`, `rocker-orange` | A rocker in its opening, its right half raised (the ivory ones are made from the blue) | 512 × 194, 512 × 181 |
| `power` | POWER's rocker, its upper half raised | 112 × 320 |
| `jewel` | POWER's lamp in its bezel (OVERLOAD's darker one is made from it) | 256 × 252 |
| `jack`, `screw` | A jack's ring nut; a screw head | 256 × 251, 96 × 96 |
| `toggle-up`, `toggle-mid`, `toggle-down` | QUALITY's toggle, its nut in the same place in all three | 180 × 291 |
| `ultra-lens-off`, `ultra-lens-on`, `ultra-bezel-off`, `ultra-bezel-on`, `ultra-floor` | ULTRA's lamp: its lens, its bezel (the lens cut out), dark and lit; the well's floor | 300 × 300 |
| `tesla-lens-off`, `tesla-lens-on`, `tesla-bezel-off`, `tesla-bezel-on` | ULTRA's Tesla lamp, as the amber one | 300 × 300 |
| `hamster-wheel`, `hamster-asleep` | LO's hamster's wheel; the hamster asleep | 300 × 300, 300 × 289 |
| `hamster-run` | The hamster's run: eight frames side by side | 1840 × 120 |
| `plate` | The name plate, blank (its lettering is printed on it) | 1044 × 328 |

### Drawings: SVG's allowed subset

**Allowed:** `svg`, `g`, `defs`, `title`, `desc`, `metadata` (its contents are not read), `path`,
`rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`, `linearGradient`, `radialGradient`,
`stop`, `clipPath`, `mask`, `pattern`, `symbol`, `use`, `image`, `filter`, `feGaussianBlur`,
`feOffset`, `feFlood`, `feComposite`, `feMerge`, `feMergeNode`, `feColorMatrix`, `feBlend` and
`feDropShadow`, in SVG's namespace.

**Refused:**

- a DTD, and with it every entity, and processing instructions;
- scripts and foreign content;
- animation;
- text (the legends are the layout's, printed by the plug-in in the pack's typeface);
- style sheets (`<style>`, by that name in any namespace, anywhere), markers and interactive
  elements;
- anything of SVG's inside `<metadata>`, which may hold only other namespaces' data (a
  licence's RDF);
- event attributes (`on...`);
- any reference outside the drawing: an `href` other than `#name`, a `url(...)` other than
  `url(#name)`, or an `<image>` other than embedded PNG data (`data:image/png;base64,...`);
- a pattern filled with another pattern.

**Limits:** 20,000 elements drawn, each reference (a `use`, a `href`, a `url(#name)`) counted
as everything it repeats (a reference that repeats itself is refused); 200,000 path segments;
32 filter primitives; a blur's deviation of at most 64; 4096 pattern tiles over what patterns
fill, all their uses together; embedded pictures within the picture limits. The counting stops
as soon as a limit is passed. A drawing is also drawn as a test, 3000 pixels across, on a thread
of its own, and must take no longer than 250 ms.

### Fonts

OpenType or TrueType, one face per file (no collections, no WOFF). The check reads what the
font's own tables say about it and sorts it into one of three outcomes. **It does not clear
it:** anyone who edits a font can change what its tables say.

- **Passes** when its declared licence (its `[[assets]]` entry's `license`) is one that lets a
  pack carry and sell it (OFL-1.1, Apache-2.0, CC0-1.0, MIT, Unlicense, Bitstream-Vera) and its
  own tables name the same licence. The OFL's or Apache's text must be in the pack.
- **Rejected** when its embedding permission is "restricted", or its licence text says
  "personal use", "non-commercial", "demo", "trial version", "evaluation", "may not be
  redistributed" or "not be embedded".
- **Review** for everything else: no declared licence; a licence not on that list; a
  difference between the declared licence and the font's own; a "preview and print"
  embedding permission; a changed font whose licence reserves its name.

## The layout, `layout.toml`

The official layout is the reference for every field: it is
`crates/ca72-panel/packs/official/layout.toml` (under the GPL), and the [starter's](skin-packs/starter/layout.toml)
is a CC0 one to begin from.

Units are the drawing's. The face's top left corner is at 0, 0, x runs right (0 to 3108) and y
runs down: the face from 0 to 795, the wooden strip above it from -130 to 0.

```toml
format = 1

[fixtures]
power = [3023, 580]       # POWER's switch (centre)
lamp = [3023, 426]        # its lamp
overload = [1764, 274]    # OVERLOAD's lamp
ultra = [3023, 92]        # QUALITY's opening (ULTRA's lamp, LO's hamster)
plate = [2578, -106]      # the name plate's top left corner (261 by 82)
phones = [2855, 581, 34]  # optional: the PHONES jack, centre and radius
screws = [[155.4, -22, 7]] # optional: the wooden strip's screws

[[control]]               # one for each of the panel's 42 controls, no more
param = "osc_mod"
at = [444, 269]           # its centre
size = [108, 50]          # a switch's or toggle's width and height, 16 to 240
colour = "red"            # a rocker's: blue, red, black, ivory
on = "right"              # the end a rocker is on at: right, top, bottom
# big = true              # a knob's larger size
# across = true           # a toggle on its side

[[line]]                  # a rule
from = [445, 9]
to = [445, 198]
width = 4                 # 0.5 to 20

[[title]]                 # a section title, spaced out to its extent
text = "MIXER"
span = [1442, 1591]
baseline = 724
# size = 52

[[text]]                  # a legend, centred on `at` (the middle of its capitals)
text = "VOLUME"
at = [1221, 59]
# size = 16               # fixed; without it, the panel's legend size (19, large 22)
# large = [1221, 55]      # its place in the large lettering, if different
```

**What a layout can't change.** Each control's parameter, kind (knob, selector, switch,
toggle), dial and legend values, and how it responds to the mouse are the plug-in's. A layout
may move a control, make a knob large, size a switch or toggle, colour a rocker and choose
its "on" end. Fields that don't apply to a control's kind are rejected.
The dials' numbers and the selectors' legends are printed by the plug-in, because they state
real values. The left hand's controls (PITCH, MODULATION, GLIDE, DECAY, NOTE PRIORITY,
RETRIGGER) and the strip's (VOICES, STEREO, OUTPUT, the presets' rail) are the plug-in's own
and stay where they are.

**Rejected:**

- a control missing, unknown, placed twice, or one of the strip's;
- anything reaching outside the panel;
- two controls (or a control and POWER) whose click areas overlap, because a click would
  find both;
- more than 300 rules, 300 legends, 16 titles or 32 screws;
- a legend of more than 48 characters;
- a layout `format` this plug-in does not read.

The controls' parameters:

> `tune`, `osc_mod`, `glide`, `mod_mix`, `osc1_range`, `osc2_range`, `osc3_range`,
> `osc2_frequency`, `osc3_frequency`, `osc1_waveform`, `osc2_waveform`, `osc3_waveform`,
> `osc3_control`, `osc1_volume`, `osc1_on`, `ext_volume`, `ext_on`, `osc2_volume`, `osc2_on`,
> `noise_volume`, `noise_on`, `osc3_volume`, `osc3_on`, `noise_type`, `filter_mode`,
> `filter_mod`, `keyboard_control_1`, `keyboard_control_2`, `cutoff`, `emphasis`,
> `contour_amount`, `filter_attack`, `filter_decay`, `filter_sustain`, `loudness_attack`,
> `loudness_decay`, `loudness_sustain`, `volume`, `main_output`, `a440`, `feedback`, `quality`.

## Making a pack with an AI agent

Coding agents (Claude Code, Codex and others) can write a layout and SVG drawings well. Give
the agent this document and the starter, have it run `check_pack` after every change and
look at the preview, and keep what it reports. Record what the agent made in each asset's
provenance: `origin = "generated"` and the `tool`. Whether such work is protected, and so
can be licensed, differs between countries, so a marketplace's review will ask about it.

## Not in the first version

These are each flagged for separate review, and none is supported:

- scripts or custom widget code;
- animation;
- the strip's and the keyboard's pictures and layout;
- colours of the drawn (classic) parts other than through a drawing;
- packs as single archive files;
- packs shared between plug-ins.
