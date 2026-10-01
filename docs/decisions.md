# The release's decisions

How the circuit model became the CA-72 plug-in. The model's own decisions, made while it was
a device of DAW-v3, are in [history.md](history.md). "The owner" is the person who
commissioned the work; an agent decision is one the owner has not separately approved.

## R1. A plug-in of its own, the CA-72 by Idle Foundry
**Owner decisions, 2026-10-01.** The circuit-derived model is released as an open-source
plug-in, in this repository, separate from DAW-v3: the synth core, the SPICE runner, the
analysis, the lab, the netlists and the derivation documents are all here.

- **Nothing trademarked on the product or its interface.** The instrument it models is
  named only as a fact, in the documents (the README's notice). The plug-in is the CA-72;
  its maker, Idle Foundry.
- **The name plate turns over.** A click on it turns it about its long axis, in 0.65 s,
  to a second name, "Model DEEZ", and back. The plate is as wide as that name needs, and
  stays that width for both.
- **Only the real-time quality, Potato.** The model's No Compromises and High Fidelity
  qualities do not run in real time; they stay in the library and the lab, and the
  plug-in has no QUALITY control.

## R2. The panel: the controller in a column, the ticks as printed
**Owner decisions, 2026-10-01.** Of the agent's mock-ups the owner chose the one with the
left hand controller in a full-height column at the panel's left (GLIDE and DECAY at its
top, the wheels below). The ticks were measured again against a photograph, the knobs'
reflections made less pronounced, and the wood runs the whole width with no cheeks
(DAW-v3's D151 has the measurements; the DAW's own panel was changed to match).

**Agent decision, 2026-10-01.** The plate is lettered in TeX Gyre Adventor, the panel's
face, 261 units wide. The approved mock-up drew it in Helvetica Bold, 270 wide, only
because its page named a bold font file that did not exist. Not separately approved.

## R3. The rear panel: the side chain, MIDI bend, no CV
**Owner decisions, 2026-10-01.** EXTERNAL INPUT is the host's side chain. The rear
panel's control voltage and trigger jacks are left out. How far a MIDI keyboard's bend
moves the pitch is a parameter, MIDI BEND RANGE (2 semitones by default).

**Agent decision, 2026-10-01.** PHONES and the column's jacks are drawn, as the
instrument has them, but do nothing: the plug-in's output is its only output.

## R4. VST3 and CLAP, GPL-3.0-or-later
**Owner decisions, 2026-10-01.** The formats are VST3 and CLAP, and the licence is the GNU
General Public License, version 3 or later. macOS signing and notarisation wait: the owner
has a Developer ID, and the builds are unsigned until it is set up.

**Agent decisions, 2026-10-01.** The plug-in is built with nih-plug, vendored with two
fixes that clap-validator needs (`third_party/nih-plug/PATCHES.md`). The output is mono,
the same on every output channel, in a stereo or a mono layout. POWER is the host's
bypass: the output fades over 10 ms, and the voice rests once silent.

## R5. The editor, drawn natively
**Owner decision, 2026-10-01.** The editor is drawn in Rust, not in a web view.

**Agent decisions, 2026-10-01.**
- The panel's art is SVG, written by `ca72-panel` and rendered with resvg. What never
  moves is rendered once for a size; each control, lamp and the plate is a layer of its
  own, rendered again only when it changes. The window is baseview's, its pixels shown
  with softbuffer.
- The editor reads the parameters each frame rather than waiting for the host's
  notifications, so automation, presets and undo all show the same way.
- The window resizes from a grip at its bottom right corner, in the panel's proportions,
  and the width is saved with the session. A host cannot resize it (nih-plug does not
  support that).
- `crates/ca72-panel/tests/approved.png` is the approved mock-up, every control at half
  its travel. The panel as drawn must match it, the plate left out.
