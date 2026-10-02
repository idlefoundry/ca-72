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
bypass: the output fades over 10 ms, and the voice rests once silent. A third fix to
nih-plug came from Steinberg's validator: its VST3 wrapper matched the host's bus
arrangements against the wrong buses, reading past the end of the host's array, and so
refused even the mono layout it offers.

**Evidence (the Mac, 2026-10-01):** `scripts/validate.sh` on the release bundles:
clap-validator 0.4.1, 37 tests passed, 0 failed, 7 skipped; pluginval 1.0.4 at strictness
10, SUCCESS, its editor tests among them; Steinberg's validator (SDK 3.8.1), 47 passed, 0
failed. That validator's bypass test prints two errors and still passes: nih-plug ignores
a value the host sets on the controller while the plug-in is processing, and takes it from
the next block's parameter changes instead. Hosts send both, so it is left as it is.

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

## R6. The editor opens at 80 % of the screen
**Owner decision, 2026-10-01.** The editor opened at 1720 logical pixels, the whole width
of a 16-inch MacBook Pro's screen; the owner asked for about 80 %.

**Agent decisions, 2026-10-01.**
- Until the grip resizes it, the editor opens at 80 % of the screen's usable width (less
  the menu bar, dock or taskbar), its height also kept within 80 % of the usable height.
  The screen is macOS's main screen, Windows' primary monitor, or the X11 screen. Where
  the size is unknown, it opens at 1380 pixels.
- The saved width is 0 until the grip sets one, so a session opens to fit whichever
  screen it is on. A session saved before this change keeps the 1720 it stored.

## R7. POLY, the full MIDI range, ANALOG and SPREAD
**Owner decisions, 2026-10-01** (the request made for DAW-v3's Minimoog and for this
plug-in alike): POLY switches between the one instrument and one per note, 1 to 10 voices,
easily; every MIDI note plays, 41 to 84 exactly as before; ANALOG and SPREAD, each 0 to
100 % and 0 by default, ANALOG giving each voice its own tolerances on the oscillators'
tuning, the cutoff and EMPHASIS, the contours' times and GLIDE, with the oscillators' and
cutoff's drift; SPREAD placing POLY's voices by their number, the first in the centre; the
voice count never asking more of the machine without saying so. With them all off a
session sounds as before. Ported from DAW-v3 (DMM, branch `minimoog-poly`), where each
part was measured first.

**Agent decisions, 2026-10-01** (not separately approved):

### POLY
- POLY is a parameter (a switch), VOICES another (2 to 10, 4 by default), both under the
  panel on the plug-in's own strip, not on the instrument's face. Ten voices are built when
  the plug-in is activated (off the audio thread: the first at a new rate takes up to a
  second, the rest are copies), so switching POLY or VOICES while playing allocates
  nothing; switching POLY lifts every key, and the notes sounding stop.
- Each voice is the whole instrument with its own keyboard circuit, which its note reaches
  as one key held while the note is. GLIDE slides from the last key that voice played.
- A new note takes a free voice (one let go whose output has stayed under 1e-6 for a whole
  block) by its number, else the one let go longest ago, else the one held longest.
- A note that takes a voice whose key was still down lifts the key and presses the new one
  once the voice's trigger contact has been open 13 ms: the modelled circuit re-arms its
  trigger only after about 12 ms (Q20 holds the reset line until C7 has drained), so its
  contours start again, as the instrument's S-trigger does. The note's pitch and attack
  come 13 ms after it. `crates/ca72/tests/retrigger.rs` finds the threshold between 11.5
  and 12 ms at 44.1, 48 and 96 kHz.
- The MIDI wheels and the panel reach every voice.
- POLY off is the plug-in as it was, to the sample (`tests/sound.rs`).
- A voice that has fallen silent is not run until its next note: it stops in time, and its
  contours resume from where they were (below the VCA's cutoff), not from rest. The same
  fuzz found that such a voice resumed with DECAY moved to its end could send the decay's
  Newton steps below the -10 V rail, and the follower's bracket check then panicked on the
  audio thread. Now a step that runs off below the rail is taken again from where the
  capacitor was, both nodes' steps held to 0.2 V, else the capacitor holds for that
  sample; every other step is as before. Test: `crates/ca72-plugin/tests/fuzz_bounds.rs`
  (every control at its ends, notes over all 128: a panic and NaN before, six seeds of 200
  rounds clean after).

### The full MIDI range
- Beyond the instrument's 44 keys, the circuit plays its end key (F or C) and the keys
  beyond are added at the keyboard's output, at the output's own volts a key, gliding with
  GLIDE while the trigger is closed: the plug-in's extension, not the circuit's. Moving the
  key string's foot in the circuit instead saturates the keyboard's amplifier (-2.86 and
  +6.92 V), so MIDI 0 to 7 and 122 to 127 each played one pitch.
- Every note's keyboard voltage is within 0.12 mV of the scale, and on 8' every note is
  within 5 cents (`crates/ca72/tests/midi_range.rs`). The oscillators' own limits, at the
  ends: on 2' the top notes run sharp (2.6 cents at MIDI 96, 8.4 kHz, to 53 at MIDI 127,
  50 kHz), on LO the lowest flat (3.7 to 10.5 cents at MIDI 0 to 36), as the circuit does
  there. Lowest-note priority covers all 128.
- A session made before this version may hold notes outside 41 to 84 that were silent;
  they now sound.
- **The converter's ceiling.** clap-validator's `param-fuzz-bounds` found that a key far
  beyond the 44 with an oscillator's RANGE at 2', FREQUENCY and the PITCH wheel at their
  tops (about 100 kHz) drove the exponential converter past where its model holds: above
  about 1e-2 A R42's feedback takes the tail node past any rail, the solve went to NaN and
  the voice stayed silent for good. The timing current is held at 5 mA
  (`vco::TIMING_MAX`, twice MIDI 127's at the panel's centre; up to it the solve converges),
  its Early slope as the converter gives it there. Below it nothing changes. Test:
  `crates/ca72/tests/midi_range.rs`, the case in every mode (NaN before, held after).

### ANALOG and SPREAD
- DAW-v3's character, the same draws and arithmetic (`crates/ca72-plugin/src/character.rs`):
  each voice's parts drawn once from its seed (the first voice's is the session's seed, as
  with one voice). At 100 % (twice the character's figures, `ANALOG_DEPTH`: the owner
  found them "extremely subtle", 2026-10-02; 50 % is what 100 % was): each oscillator's
  tuning spread 8 cents (an offset at its range tap), the cutoff 0.08 octave (at R51, 0.98 V
  an octave), EMPHASIS, the contours' ATTACK and DECAY and GLIDE 0.024 of their rotation;
  each oscillator drifts (a mean-reverting walk, 6 cents, 4 s) and wavers (0.8 cent under
  30 Hz), the cutoff drifts (0.03 octave). Sustains, levels and waveshapes are left alone.
- SPREAD places voice n at 0, -1, +1, -0.5, +0.5, -0.75, +0.75, -0.25, +0.25, -0.125 times
  the amount, a voice in the centre whole on both sides. The output became stereo for it;
  on a mono output it is the two channels together, so a session with SPREAD at 0 sounds
  as before on either.
- SPREAD is drawn at half opacity while POLY is off (the owner, 2026-10-02, approved: "in
  the case of a single voice only patch, then the spread should be grayed out"): the one
  voice is in the centre, so it does nothing. It stays operable (agent's decision), to be
  set before POLY goes on. The strip has no tips, so it is dimmed only. Test:
  `strip::tests::spread_dims_while_poly_is_off`. (A stereo spread of the three oscillators
  in one voice was weighed and dropped: they are summed before the one filter, so it would
  need a second mixer, filter and VCA.)

### POLY in real time
- POLY's voices run in turn on the host's audio thread. The plug-in starts no threads of
  its own: the host sizes its own threads to the machine, and a plug-in's busy threads
  beside them would be more than the machine has.
- When it is activated, the plug-in measures a voice's cost in a block where its key
  changes, at the host's rate and largest block, and counts how many such fit in 60 % of
  the block's period (DAW-v3's figure). The strip shows that number beside VOICES and marks
  VOICES when it asks for more ("expect dropouts").
- Measured (`tests/poly_bench.rs`: the engine as a host calls it, on one promoted thread,
  48 kHz, changing chords every half second with the modulation moving, ANALOG 50 %,
  SPREAD 70 %, 60 s, a block a period and never back to back; two clean runs a count;
  `evidence/poly-<machine>.log`):
  - **omarchy** (Ryzen 7 7800X3D), 256 frames: **4 voices** with no late block in two runs
    (mean 2.5 ms of 5.33, worst 4.7); 5 voices 1 late block in a minute, 6 voices 87, 8
    voices 1,821, 10 every block. At 512 frames 8 voices were still late 842 times. The
    strip's own count there: 4 at 256 frames.
  - **The Mac** (M4 Pro): not measured cleanly. Ableton Live was running and playing there,
    and the block's time hardly changed from 3 to 6 voices, so the bench thread was held
    back rather than paying for the voices (`evidence/poly-mac.log`). The strip's own count
    there: 7 at 256 frames. To be measured again on a quiet Mac.
  - No run was demoted by the watchdog.
- **Evidence (the Mac, 2026-10-02, the release bundles of this version):** clap-validator
  0.4.1, 37 passed, 0 failed, 7 skipped (before the two fixes `param-fuzz-bounds` failed
  every time); pluginval 1.0.4 at strictness 10, SUCCESS, its editor tests skipped this
  time (`--skip-gui-tests`: the Mac was in use, and they open windows on its screen);
  Steinberg's validator (SDK 3.8.1), 47 passed, 0 failed. The workspace's tests and clippy
  pass on the Mac and on Linux. Linux's validators were not run locally (omarchy has no
  cmake or xvfb-run); CI runs them.
- So VOICES defaults to 4, what omarchy plays at 256 frames; the strip shows each
  machine's own count (a voice costs about the same share of any block, so a larger block
  barely changes it: 4 at 1024 frames on omarchy too).

## R8. FEEDBACK: the phones' VOLUME knob
**Owner decisions, 2026-10-02.** The instrument has no overdrive control; players cable its
PHONES output into EXTERNAL INPUT and turn the phones' VOLUME up, overdriving the external
input's preamplifier and the mixer. The owner asked for a feedback control, and then for it
to take the place of the unused volume knob: the phones' VOLUME, which did nothing (R3), is
now FEEDBACK, its legend reading FEEDBACK. The same was done in DAW-v3 (DMM).

**Agent decisions, 2026-10-02** (not separately approved):
- FEEDBACK (`feedback`, 0 to 10 on the knob's dial, 0 by default: no cable) sends each
  voice's output, before MAIN OUTPUT's VOLUME, into its EXTERNAL INPUT one sample later (a
  loop cannot be instantaneous), added to the side chain there. The level sent follows the
  knob's taper (the owner, 2026-10-02: 3 on the dial was "already super intense"; that
  should come at about 6): `feedback_law` = travel^2.357, so 6 sends 30 % of the output, 3
  about 6 %, 5 20 %, 10 all of it. The parameter (what automation sees) is the knob's
  position.
  As with the cable it is heard through the MIXER's EXTERNAL INPUT switch and VOLUME, and
  OVERLOAD lights.
- While FEEDBACK is on the preamplifier is solved as High Fidelity solves it: Potato's
  (its paths fitted to the circuit's gain) sent the loop into a low, dark oscillation the
  circuit does not make. With it Potato sounds as High Fidelity does at 30 % (level within
  1 %, brightness 30.5 % against 28.7 % above 1 kHz; `crates/ca72/tests/feedback.rs`).
- Cost: a voice about 9.6 us a sample with FEEDBACK on, against 1.8 without (omarchy): about
  half a core at 48 kHz. The strip's real-time count is measured both ways at activation
  and shows the one in force: on omarchy at 256 frames, 4 POLY voices without FEEDBACK,
  none with it. One voice (POLY off) with FEEDBACK costs about half the period.
- The approved panel (R2, `crates/ca72-panel/tests/approved.png`) differs only at that knob
  and its legend; the comparison still passes within its limits.
- The knob is dimmed while EXTERNAL INPUT is closed (the owner, 2026-10-02, approved: in a
  test the knob seemed to do nothing, as EXTERNAL INPUT was off; "grayed out ... unless the
  external volume input has been turned on and is above zero"). While the mixer's EXTERNAL
  INPUT switch is off or its VOLUME is at 0 the knob is drawn at half opacity, as the parts
  with no parameter are, and its tip reads "FEEDBACK (silent: switch EXTERNAL INPUT on in
  the MIXER and turn its VOLUME up)". It stays operable (agent's decision), so the amount
  can be set first. The sound is unchanged. Tests: `controls::feedback_silent`'s cases and
  `render::tests` (the knob's pixels dimmed, nothing else on the panel changed).
- Tests: `crates/ca72/tests/feedback.rs` (not heard with EXTERNAL INPUT off; OVERLOAD lit and
  the sound brighter in every mode; Potato as High Fidelity; finite and within the rails
  with every control feeding the loop at its end), the threaded test (the same samples to
  the bit), the engine's bounds fuzz (`tests/fuzz_bounds.rs`, FEEDBACK at its ends too).

## R9. A floor of mismatch, and LOCK
**Owner decisions, 2026-10-02** (with DAW-v3's Minimoog, DMM). With ANALOG at 0 the three
oscillators were identical: at one pitch they locked in phase and sounded up to 3.5 dB
louder than with ANALOG on, which invites judging by loudness. The owner chose a small
built-in mismatch, with a switch that makes the oscillators perfectly identical, turned
down to match so it cannot win by loudness, and a tooltip.

**Agent decisions, 2026-10-02** (not separately approved):
- With LOCK off each oscillator keeps a floor of its own (`ANALOG_FLOOR`, 0.75 of the
  character's figures: a tuning spread of 3 cents, 2.25 cents of drift, 0.3 cent of waver);
  ANALOG adds to it. In DAW-v3 the level then moves 0.4 dB across ANALOG's range.
- LOCK (`lock`, off by default) makes the oscillators the circuit as drawn and turns the
  output down by `lock_trim`: the switched-on oscillators that share a pitch and their
  volumes give the ideal gain of locking, of which 0.68 shows at the output (`LOCK_SHARE`,
  fitted in DAW-v3). Measured: three in unison +0.4 dB with LOCK, one oscillator 0.0
  (`tests/sound.rs`); octaves lock only in part and are left, about 0.7 dB.
- The plug-in has no rear panel, so LOCK is a parameter without a control on the panel: a
  host shows it in its list of parameters ("Lock (oscillators identical)").
- A session saved before this version, with ANALOG at 0, now has the floor (the plug-in is
  not yet released; DAW-v3 keeps its older devices exact).
