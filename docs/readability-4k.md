# CA-72 0.3.0 at 4K and 100 % scaling

Survey for [issue #70](https://github.com/idlefoundry/ca-72/issues/70), 2026-10-19. Base:
`17f1b5f`, release-0.3.0. Both built-in looks, STANDARD and LARGE, one physical pixel a
logical pixel. These are offscreen renders of the editor's components, not photographs of a
27 or 32 inch display or a test in a host.

The fresh editor on a 3840 by 2160 usable screen is already almost the largest window:
**3026 by 1728**, not the SIZE menu's **100 %**, which is 1380 by 788. At the fresh size most
legends, the keyboard's controls and the presets' drawer clear the threshold below. At the
100 % menu size many do not. Tooltips and menu headings stay small at every window size.

## Method and threshold

Flag an x-height of **9 physical pixels or less**. This is a conservative screen for small
print, not a guarantee of comfort above 9. [Legge and Bigelow (2011)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3428264/)
report a fluent-reading lower limit near 0.2 degrees of x-height. Calculated for a 16:9 4K
display, 9 pixels are 1.40 mm at 27 inches or 1.66 mm at 32 inches. At a 60 cm viewing
distance these subtend only 0.134 to 0.159 degrees; 0.2 degrees would take about 13.5 or
11.4 pixels respectively. The 9 px cutoff is therefore a useful lower warning line.

Font size is the em, not the visible height. The bundled URW Gothic outlines have:

| Typeface | Capital H / em | Lowercase x / em |
|---|---:|---:|
| Book | 0.739 | 0.547 |
| Demi | 0.739 | 0.554 |

For type, multiply the drawing's font size by the outline ratio and `window_width / 3108`.
For tooltips and ordinary menus the size is in logical pixels and does **not** grow with the
window: `Renderer::text_size` divides out the drawing scale. LARGE multiplies it by 1.25.
The settings' menu is reduced further if it cannot fit the viewport.

The hand-drawn dot font has five rows a lowercase x and seven a capital. Measure the centres'
span plus the lit dot's diameter, excluding glow: presets' drawer x/cap = 4.9/6.9 times
pitch; rail and FX x/cap = 4.72/6.72 times pitch. Seven-segment readouts have a 53.76-unit
capital envelope (`84 * 0.64`); they are above the threshold at every offered size.

The appendix gives **both x-height and cap height**. For capitals and numerals, x-height is a
reference for the same size of type, not an assertion that a lowercase x appears there.
A numeral with an 8 px x-reference can have an 11 px capital height. Such rows are cautionary,
less severe than mixed-case text whose actual lowercase letters are below 9 px. Stroke, glow,
contrast, grain, sight and viewing distance also matter; these measurements do not establish
that every flagged item is unreadable.

## Window sizes

`editor.rs`: `SIZES`, `fitted_width`, `height_for`, `MIN_WIDTH`, `MAX_WIDTH`. FIT TO SCREEN
uses 80 % of the usable screen, limited by height as well as width; saved sessions can retain
another width. Window decorations, docks and host chrome reduce the usable area. All five
menu percentages fit a full 3840 by 2160 screen. Minimum and maximum grip widths are included.

| Choice | Physical viewport | Drawing scale |
|---|---:|---:|
| Minimum grip | 860 × 491 | 0.27671 |
| 75 % | 1035 × 591 | 0.33301 |
| 100 % / screen size unknown fallback | 1380 × 788 | 0.44402 |
| 125 % | 1725 × 985 | 0.55502 |
| 150 % | 2070 × 1182 | 0.66602 |
| 200 % | 2760 × 1576 | 0.88803 |
| Fresh / FIT TO SCREEN, full 4K usable area | 3026 × 1728 | 0.97362 |
| Maximum grip | 3108 × 1775 | 1.00000 |

The renderer allocates the ceiling of the drawing's extent; each evidence image is clipped
by at most one row or column to the editor's rounded viewport. No evidence image is rescaled.
The 4K canvases centre the window at one source pixel a screen pixel. The annotated crop
sheets additionally show explicitly labelled nearest-neighbour enlargements.

## What remains small

Values here are x-height, STANDARD / LARGE, in physical pixels, before the narrow fix below.
The complete per-instance table includes capitals, positions and the 1380 px case.

| Element | At 1380 px wide | At fresh 3026 px wide | Interpretation |
|---|---:|---:|---|
| Panel legends, 19 / 22 | 4.67 / 5.41 | 10.25 / 11.87 | Fresh 4K clears the line; smaller saved windows do not |
| Dial numbers, 15 / 17 | 3.69 / 4.18 | 8.09 / 9.17 | LARGE helps at FIT TO SCREEN; actual capitals are taller |
| M-SEC., SEC., OSC. 3, NOISE dial captions, 14 / 16 | 3.44 / 3.94 | 7.55 / 8.63 | Still small in LARGE |
| OSC. 3 CONTROL, fixed 16 | 3.94 / 3.94 | 8.63 / 8.63 | Explicitly left fixed by R-LETTERING |
| QUALITY's ULTRA / HI / LO, fixed 15 | 3.69 / 3.69 | 8.09 / 8.09 | Omission corrected on this branch |
| IDLE FOUNDRY, fixed 12 | 2.95 / 2.95 | 6.47 / 6.47 | Decorative maker name; cap height only 8.63 px even fresh |
| Strip legends, 21 / 23 | 5.17 / 5.66 | 11.33 / 12.41 | LARGE works |
| Strip small print, 16 / 18 | 3.94 / 4.43 | 8.63 / 9.71 | LARGE works |
| Strip / arp / FX tab caps, fixed 17 | 4.18 / 4.18 | 9.17 / 9.17 | Explicit fixed-size family, still small in narrow windows |
| SAVE / PANEL / ARP / FX rail caps, fixed 20 | 4.92 / 4.92 | 10.79 / 10.79 | Unchanged by LARGE |
| Keyboard mapping, fixed 18 Book | 4.37 / 4.37 | 9.59 / 9.59 | Borderline at fresh size; SMALL at 100 % menu size |
| Keyboard TYPING / PANIC / − / +, fixed 20 | 4.92 / 4.92 | 10.79 / 10.79 | LARGE leaves the keyboard unchanged |
| Preset drawer list, tags, status and MIDI text, pitch 4 | 8.70 / 8.70 | 19.08 / 19.08 | Below the line at 100 %; easily clears it fresh |
| Preset drawer search, pitch 4.5 | 9.79 / 9.79 | 21.47 / 21.47 | Search already larger than the list |
| Preset drawer headings, 26 | 6.40 / 6.40 | 14.02 / 14.02 | Cap heights 8.53 / 18.71 px respectively |
| Preset action keys, 28 (MIDI LEARN fitted) | about 6.7–6.9 / same | about 14.6–15.1 / same | Actions already enlarged on this base; no further change here |
| FX list names and makers, pitch 4 | 8.38 / 8.38 | 18.38 / 18.38 | Effect chooser ignores LARGE; needs a coordinated layout change |
| Tooltip / MIDI Learn note, 12 / 15 Book | 6.56 / 8.21 | 6.56 / 8.21 | LARGE already works; both sizes below the line |
| Menu items, 13.8 / 17.25 Book | 7.55 / 9.44 | 7.55 / 9.44 | LARGE works at these sizes |
| Menu title, 82 % of items, Demi | 6.27 / 7.84 | 6.27 / 7.84 | Both below the line |
| Menu group headings, 72 % of items, Demi | 5.50 / 6.88 | 5.50 / 6.88 | Smallest functional text in the fresh 4K window |
| Typed knob value's unit, 70 % of the entry's size, Book | 5.28 / 6.61 | 5.28 / 6.61 | Entry follows LARGE; unit still small |

At 860 px the standalone settings fixture's 19 items plus title force both lettering sizes
down to about 12.16 px em: x-height 6.65 px for items and 4.85 px for group headings. At
1035 px LARGE is reduced to about 14.63 px em, only a little above STANDARD's 13.8. A
plug-in's shorter menu fits better; installed packs can make either menu taller. These
conditional shrinkages are in the measurements and overlay manifest.

The typical 15 / 17-unit numbers' x-reference, across the widths above, is respectively:
2.30 / 2.61, 2.77 / 3.14, 3.69 / 4.18, 4.61 / 5.23, 5.53 / 6.27,
7.38 / 8.36, 8.09 / 9.17, and 8.31 / 9.42 px. Increasing the window is much more effective
than LARGE alone for the panel. The already approved panel has tight legends; increasing
all its type again would require moving controls, outside this survey's scope.

## Changes and proposals

1. **Implemented: QUALITY's small captions follow LARGE, 15 to 17 units.** R-LETTERING
   names the panel's LO / HI small print as part of the 15 to 17 step, but the three QUALITY
   positions remained hard-coded at 15 even before skin packs. `large_size` is an optional
   layout property; the official layout uses it for these three captions only. STANDARD and
   older packs' fixed sizes are unchanged. Font-fit checks use the selected size too. At
   1380 px this remains tiny (cap 4.92 to 5.58 px); at 3026 px the x-reference goes from 8.09
   to 9.17 px, cap 10.79 to 12.23. The owner should inspect the crops before accepting this
   interpretation of R-LETTERING.
2. **Highest value, owner's eye: a larger minimum for tooltips and menu headings.** A Book
   font size of 17 logical pixels gives a 9.30 px x-height; 21.25 in LARGE gives 11.62.
   Headings also need a minimum of about 17 rather than a 0.72 multiplier. Keep the title's
   band, colour and spacing. A scrolling menu or shorter submenus would preserve that minimum
   in small windows; shrinking the entire menu defeats LARGE. This changes density and
   behaviour, so it is proposed, not implemented. The current tooltip and ordinary-menu paths
   already honour LARGE; increasing their baseline is a design choice, not an omitted setting.
3. **High value, coordinate with preset-browser work: a drawer minimum or responsive LARGE.**
   At 1380 px changing the list's pitch from 4 to 4.5 would raise its lowercase height from
   8.70 to 9.79 px. Rows, columns, scroll counts and hit targets must change together. A
   physical minimum near 9.5 px x-height needs still larger pitch in the minimum window.
   The search is already pitch 4.5. R-LETTERING explicitly left the drawer unchanged; do not
   treat a new drawer layout as a bug fix. The current action captions are already on caps
   at 28 units, rather than the old tiny legends above them.
4. **Medium value, owner's eye: FX chooser text.** It is a menu made from a fixed dot display,
   introduced after R-LETTERING. A 1.25 step makes its 4-unit pitch 5 (10.48 px x-height at
   1380 px), but nine rows of dots including descenders no longer fit 40-unit lines. Its
   existing plate is 540 units tall with only about 564 available below the drawer. A larger
   font requires fewer visible rows, altered columns and shared drawing/hit-test geometry.
   The FX context menu already follows LARGE. The chooser is left for a coordinated fix.
5. **Medium value, owner's eye: keyboard mapping and tab/rail caps in LARGE.** Extend LARGE
   deliberately with fitted labels, rather than scale caps blindly: PANEL's narrow cap and
   ARP's UP/DOWN need width checks. The keyboard, caps and displays were explicitly outside
   the original enlargement. Do not alter the approved realistic panel to accommodate them.
6. **Low cost, owner's eye: make the existing sizing choices clearer.** The fresh 4K opening
   is already 3026 px wide, beyond the 200 % menu entry. A larger default would gain at most
   2.7 % before the 3108 px limit. Prefer explaining FIT TO SCREEN and considering an explicit
   4K-friendly choice or access to LETTERING beside it. Do not silently override saved sizes.
   Leave decorative maker lettering and the realistic face's layout for the owner's judgement.

## Evidence and limits

The off-repository handoff contains 224 whole-window baseline PNGs: eight widths, two looks,
two lettering settings, seven states (strip, keyboard with typing, presets, rename, MIDI list,
arp, FX with chooser). A further 32 show the settings fixture and tooltip in every combination.
The fixture uses the panel's real menu renderer and the built-in standalone's 0.3.0 item list;
it is not a live host capture. Eight before/after comparison renders cover the narrow change
in both looks at 1380 and 3026 px. Crops retain native pixels; the manifest names their source
and rectangle. The image inventory includes dimensions and SHA-256 hashes.

`examples/png.rs` now accepts `CA72_TIP=1`, `CA72_MENU=settings` and
`CA72_PRINT_SVG=<file>` for reproduction. `examples/golden.rs` accepts
`CA72_GOLDEN_LAYOUT=<file>` to compare the frozen v1 layout with the official one. SVG text
was measured before rasterisation, font outlines with fontTools; action-caption advances
were shaped with HarfBuzz. Dynamic strings sharing one size are explicitly grouped in the
appendix; each fixed panel/strip text instance has its own row and drawing coordinates.
The CSV contains every width and both lettering modes, plus the fixed LARGE result.

`cargo test -p ca72-panel` passes, including both approved-panel tests and frozen-pack
compatibility. `cargo clippy -p ca72-panel --all-targets -- -D warnings` passes. The generated
32-frame golden comparison has 16 identical STANDARD frames; the 16 LARGE frames change
only QUALITY's three captions, at scales 0.44 and 1.0 in both looks and all four scenes.
Neither `tests/approved.png` nor `tests/approved-large.png` was changed. They exclude this
QUALITY region by design; the separate before/after comparison records its change.

Not tested: a real 4K monitor, a human comfort trial, hosts, OS/host scales other than 1x,
third-party font packs, a live installed-effects scan or menus with arbitrary installed-pack
counts. Geometry and existing hit targets are unchanged by the fix; panel tests cover both
looks and the minimum viewport. Proposed reflows need their own small-screen and pointer
checks before implementation. The original review does not establish which build it tested;
this survey makes no claim to reproduce that review's exact environment.

See [the complete small-text inventory](readability-4k-text.md).
