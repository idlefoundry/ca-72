# Quick-start pictures

These are rendered illustrations of CA-72 0.3.0, not captures from a particular host.
They use the repository's own panel, strip, keyboard, preset, value-entry, MIDI Learn,
arpeggiator and effects renderers, with `Skin::Worn` (the default realistic look), standard
lettering and 0.6 pixels a panel unit before cropping and resizing.

The rendering follows `crates/ca72-panel/examples/png.rs` and `golden.rs`; the parameter
lookup in `crates/ca72-plugin/examples/preset_png.rs` is another reference. The main
control positions use this branch's `Ca72Params::default()` values, rather than the PNG
example's demonstration patch. Strip readouts follow `editor.rs`'s `strip_scene`.

- `01`: standalone keyboard, with TYPING off.
- `02`: Init's panel and lower controls, with the standalone rail.
- `03`: Init's upper panel, with numbered callouts.
- `04`: oscillator 3 at LO, triangle, FREQUENCY below centre, OSC. 3 CONTROL off, its mixer
  switch off, MODULATION MIX at OSC. 3, OSCILLATOR MODULATION on and MOD wheel at 0.2;
  the separate filter controls alongside it.
- `05`: four POLY voices, SCATTER, EVEN, WIDTH 70 %, INNER off.
- `06`: QUALITY LO, with a key to all three settings.
- `07`: Init's two OUTPUT sections.
- `08`: the preset drawer, populated from this branch's `sounds/presets.toml`.
- `09`: CUTOFF waiting for MIDI Learn, and a separate view typing 1.5 at that knob.
- `10`: the examples' ARP and FX drawers, shown separately; effect names are illustrative.

ImageMagick crops, combines and annotates the renders, strips metadata, and writes indexed
256-colour PNGs. All ten pictures together are below 3 MB. For an update, render the
controls and overlays again rather than painting a different setting onto a control.
