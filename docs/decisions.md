# The release's decisions

How the circuit model became the CA-72 plug-in. The model's own decisions, made while it
was a device of the DAW it was developed in, are in [history.md](history.md). "The owner"
is the person who commissioned the work; an agent decision is one the owner has not
separately approved.

## R1. A plug-in of its own, the CA-72 by Idle Foundry
**Owner decisions, 2026-10-01.** The circuit-derived model is released as an open-source
plug-in, in this repository, separate from that DAW: the synth core, the SPICE runner, the
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
(history.md has the measurements; the DAW's own panel was changed to match).

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
- **Fixed 2026-10-02** (the owner found that on the Mac a resize shrank the window while
  the panel kept its size, until the editor was opened again). baseview on macOS resizes
  its view when asked but reports no resize of its own making (only a change of scale), so
  the frames stayed at the old size. The editor now applies the new size itself when the
  grip resizes it; on Linux and Windows the report that follows finds it already taken.
  Test: the grip's, every frame at the new size.
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
**Owner decisions, 2026-10-01** (the request made for the DAW's instrument and for this
plug-in alike): POLY switches between the one instrument and one per note, 1 to 10 voices,
easily; every MIDI note plays, 41 to 84 exactly as before; ANALOG and SPREAD, each 0 to
100 % and 0 by default, ANALOG giving each voice its own tolerances on the oscillators'
tuning, the cutoff and EMPHASIS, the contours' times and GLIDE, with the oscillators' and
cutoff's drift; SPREAD placing POLY's voices by their number, the first in the centre; the
voice count never asking more of the machine without saying so. With them all off a
session sounds as before. Ported from the DAW, where each part was measured first.

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
- The DAW's character, the same draws and arithmetic
  (`crates/ca72-plugin/src/character.rs`): each voice's parts drawn once from its seed (the
  first voice's is the session's seed, as with one voice). At 100 % (twice the character's
  figures, `ANALOG_DEPTH`: the owner found them too subtle, 2026-10-02; 50 % is what 100 %
  was): each oscillator's tuning spread 8 cents (an offset at its range tap), the cutoff
  0.08 octave (at R51, 0.98 V an octave), EMPHASIS, the contours' ATTACK and DECAY and
  GLIDE 0.024 of their rotation; each oscillator drifts (a mean-reverting walk, 6 cents, 4
  s) and wavers (0.8 cent under 30 Hz), the cutoff drifts (0.03 octave). Sustains, levels
  and waveshapes are left alone.
- SPREAD places voice n at 0, -1, +1, -0.5, +0.5, -0.75, +0.75, -0.25, +0.25, -0.125 times
  the amount, a voice in the centre whole on both sides. The output became stereo for it;
  on a mono output it is the two channels together, so a session with SPREAD at 0 sounds
  as before on either.
- SPREAD is drawn at half opacity while POLY is off (the owner, 2026-10-02, approved
  greying it out for a patch of a single voice): the one voice is in the centre, so it does
  nothing. It stays operable (agent's decision), to be set before POLY goes on. The strip
  has no tips, so it is dimmed only. Test: `strip::tests::spread_dims_while_poly_is_off`.
  (A stereo spread of the three oscillators in one voice was weighed and dropped: they are
  summed before the one filter, so it would need a second mixer, filter and VCA.)

### POLY in real time
- **Since R11** (the owner's decision, 2026-10-02): POLY's voices are shared between the
  host's audio thread and up to four threads of the plug-in's own, and the strip's count
  measures the engine with them. What follows is how it stood before, with its figures.
- POLY's voices run in turn on the host's audio thread. The plug-in starts no threads of
  its own: the host sizes its own threads to the machine, and a plug-in's busy threads
  beside them would be more than the machine has.
- When it is activated, the plug-in measures a voice's cost in a block where its key
  changes, at the host's rate and largest block, and counts how many such fit in 60 % of
  the block's period (the DAW's figure). The strip shows that number beside VOICES and
  marks VOICES when it asks for more ("expect dropouts"). **Since R12** the strip shows no
  count and the plug-in measures none when activated.
- Measured (`tests/poly_bench.rs`: the engine as a host calls it, on one promoted thread,
  48 kHz, changing chords every half second with the modulation moving, ANALOG 50 %,
  SPREAD 70 %, 60 s, a block a period and never back to back; two clean runs a count):
  - **The Linux reference machine** (Ryzen 7 7800X3D), 256 frames: **4 voices** with no
    late block in two runs (mean 2.5 ms of 5.33, worst 4.7); 5 voices 1 late block in a
    minute, 6 voices 87, 8 voices 1,821, 10 every block. At 512 frames 8 voices were still
    late 842 times. The strip's own count there: 4 at 256 frames.
  - **The Mac** (M4 Pro): not measured cleanly. Ableton Live was running and playing there,
    and the block's time hardly changed from 3 to 6 voices, so the bench thread was held
    back rather than paying for the voices. The strip's own count there: 7 at 256 frames.
    To be measured again on a quiet Mac.
  - No run was demoted by the watchdog.
- **Evidence (the Mac, 2026-10-02, the release bundles of this version):** clap-validator
  0.4.1, 37 passed, 0 failed, 7 skipped (before the two fixes `param-fuzz-bounds` failed
  every time); pluginval 1.0.4 at strictness 10, SUCCESS, its editor tests skipped this
  time (`--skip-gui-tests`: the Mac was in use, and they open windows on its screen);
  Steinberg's validator (SDK 3.8.1), 47 passed, 0 failed. The workspace's tests and clippy
  pass on the Mac and on Linux. Linux's validators were not run locally (the Linux
  reference machine has no cmake or xvfb-run); CI runs them.
- So VOICES defaults to 4, what the Linux reference machine plays at 256 frames; the strip
  shows each machine's own count (a voice costs about the same share of any block, so a
  larger block barely changes it: 4 at 1024 frames on that machine too).

## R8. FEEDBACK: the phones' VOLUME knob
**Owner decisions, 2026-10-02.** The instrument has no overdrive control; players cable its
PHONES output into EXTERNAL INPUT and turn the phones' VOLUME up, overdriving the external
input's preamplifier and the mixer. The owner asked for a feedback control, and then for it
to take the place of the unused volume knob: the phones' VOLUME, which did nothing (R3), is
now FEEDBACK, its legend reading FEEDBACK. The same was done in the DAW's instrument.

**Agent decisions, 2026-10-02** (not separately approved):
- FEEDBACK (`feedback`, 0 to 10 on the knob's dial, 0 by default: no cable) sends each
  voice's output, before MAIN OUTPUT's VOLUME, into its EXTERNAL INPUT one sample later (a
  loop cannot be instantaneous), added to the side chain there. The level sent follows the
  knob's taper (the owner, 2026-10-02, found 3 on the dial already very intense; that
  should come at about 6): `feedback_law` = travel^2.357, so 6 sends 30 % of the output, 3
  about 6 %, 5 20 %, 10 all of it. The parameter (what automation sees) is the knob's
  position.
  As with the cable it is heard through the MIXER's EXTERNAL INPUT switch and VOLUME, and
  OVERLOAD lights.
- While FEEDBACK is on the preamplifier is solved as High Fidelity solves it: Potato's
  (its paths fitted to the circuit's gain) sent the loop into a low, dark oscillation the
  circuit does not make. With it Potato sounds as High Fidelity does at 30 % (level within
  1 %, brightness 30.5 % against 28.7 % above 1 kHz; `crates/ca72/tests/feedback.rs`).
  **Since R11** (the owner's decision, 2026-10-02): inside the loop Potato's preamplifier is
  its own model at twice the rate, its output as late as the circuit's
  (`preamp::Delayed`), within the circuit's own spread in the loop.
- Cost: a voice about 9.6 us a sample with FEEDBACK on, against 1.8 without (the Linux
  reference machine): about half a core at 48 kHz. The strip's real-time count is measured
  both ways at activation and shows the one in force: on that machine at 256 frames, 4 POLY
  voices without FEEDBACK, none with it. One voice (POLY off) with FEEDBACK costs about
  half the period. **Since R11:** FEEDBACK costs a voice about a fifth more.
- The approved panel (R2, `crates/ca72-panel/tests/approved.png`) differs only at that knob
  and its legend; the comparison still passes within its limits.
- The knob is dimmed while EXTERNAL INPUT is closed (the owner, 2026-10-02, approved: in a
  test the knob seemed to do nothing, as EXTERNAL INPUT was off, and the owner asked for it
  to be greyed out unless the external input is switched on and its volume above zero).
  While the mixer's EXTERNAL INPUT switch is off or its VOLUME is at 0 the knob is drawn at
  half opacity, as the parts with no parameter are, and its tip reads "FEEDBACK (silent:
  switch EXTERNAL INPUT on in the MIXER and turn its VOLUME up)". It stays operable
  (agent's decision), so the amount can be set first. The sound is unchanged. Tests:
  `controls::feedback_silent`'s cases and `render::tests` (the knob's pixels dimmed,
  nothing else on the panel changed).
- Tests: `crates/ca72/tests/feedback.rs` (not heard with EXTERNAL INPUT off; OVERLOAD lit and
  the sound brighter in every mode; Potato as High Fidelity; finite and within the rails
  with every control feeding the loop at its end), the threaded test (the same samples to
  the bit), the engine's bounds fuzz (`tests/fuzz_bounds.rs`, FEEDBACK at its ends too).

## R9. A floor of mismatch, and LOCK
**Owner decisions, 2026-10-02** (with the DAW's instrument). With ANALOG at 0 the three
oscillators were identical: at one pitch they locked in phase and sounded up to 3.5 dB
louder than with ANALOG on, which invites judging by loudness. The owner chose a small
built-in mismatch, with a switch that makes the oscillators perfectly identical, turned
down to match so it cannot win by loudness, and a tooltip.

**Agent decisions, 2026-10-02** (not separately approved):
- With LOCK off each oscillator keeps a floor of its own (`ANALOG_FLOOR`, 0.75 of the
  character's figures: a tuning spread of 3 cents, 2.25 cents of drift, 0.3 cent of waver);
  ANALOG adds to it. In the DAW the level then moves 0.4 dB across ANALOG's range.
- LOCK (`lock`, off by default) makes the oscillators the circuit as drawn and turns the
  output down by `lock_trim`: the switched-on oscillators that share a pitch and their
  volumes give the ideal gain of locking, of which 0.68 shows at the output (`LOCK_SHARE`,
  fitted in the DAW). Measured: three in unison +0.4 dB with LOCK, one oscillator 0.0
  (`tests/sound.rs`); octaves lock only in part and are left, about 0.7 dB.
- The plug-in has no rear panel, so LOCK is a parameter without a control on the panel: a
  host shows it in its list of parameters ("Lock (oscillators identical)").
- A session saved before this version, with ANALOG at 0, now has the floor (the plug-in is
  not yet released; the DAW keeps its older devices exact).

## R10. Presets: a library shared with the DAW, a bar and a drawer
**Owner decisions, 2026-10-02** (with the DAW's instrument). The owner asked for a few
factory presets, from well-regarded published patches under new names; for every preset to
be created, edited and deleted; and for favourites, search and tags. The plug-in gets the
same full browser as the DAW's (built there first); the two share one library; the presets
drop down from the bottom, in a drawer (agreed: a bar under the strip, a drawer sliding up
over the panel, not growing the window, which hosts may refuse).

**Agent decisions, 2026-10-02** (not separately approved):
- **A preset is named panel settings**: the dials' plain values by parameter ID (a choice
  by its index, a switch 0 or 1), POLY and VOICES too, never the PITCH wheel or the bypass;
  the MOD wheel is part of one. The plug-in's IDs and ranges are the DAW's instrument's,
  so one file serves both. Choosing one sets every other parameter to its default, POLY and
  VOICES only when it names them; each parameter changed is a gesture of its own. **Since
  R12** POLY, VOICES, ANALOG and SPREAD are set like the rest, and every factory preset
  names them.
- **The factory's 17** are `crates/ca72-plugin/sounds/minimoog.toml`, the DAW's file byte
  for byte (the DAW's record gives each one's source, their levelling and the adjustments
  made on the model). Built in, never changed.
- **The user's are files** in `<data>/Idle Foundry/CA-72/Presets` (`$IDLE_FOUNDRY_SHARED`
  overrides the `Idle Foundry` folder), `<name>.toml`: `format`, `name`, `description`,
  `tags`, `favorite`, `replaces`, `values`; written whole and moved into place. The factory
  presets' tags, favourites and deletions are the folder's `factory.toml`; saving over or
  renaming a factory preset writes the user's version standing in for it (REVERT takes it
  back); deleting one hides it (RESTORE FACTORY). Names: 1 to 64 characters, none of
  `/ \ : * ? " < > |`, unique ignoring case. The DAW reads and writes the same files the
  same way (checked both ways with each program's own writer: each lists the other's
  presets, tags and favourites, and sets the same normalized values).
- **The plug-in remembers its preset** (`preset`, a persisted field saved with the
  session); the bar marks it • once a parameter leaves the preset's value.
- **The window grows by the bar** (80 panel units): 617 pixels tall at 1720 wide, was 577.
  **Since R12** the bar is the strip's left, and the window is 577 again.
- **The drawer takes the keyboard** while open (baseview's focus on opening and on a
  field): typing goes to the field with the caret (the search when none), the arrows step
  through the list setting each preset, Enter commits, Escape takes back an edit or
  closes, Tab moves between the search and the save fields. Keys with Command or Control
  stay the host's, and with the drawer shut every key is.
- **Drawn** with the strip's colours and the panel's typeface; the star and the arrows are
  shapes (the typeface has no such glyphs).
- **Tests:** `library::tests` (saved, found, tagged, made favourite, renamed, removed; a
  factory preset edited, renamed, reverted, deleted, restored; a file of the DAW's read, an
  unreadable one reported), `ca72-panel` `presets::tests` (fields edit at the caret, every
  control found where drawn, the caret placed by a click, the frames drawn), and the
  editor's (the bar opens the drawer over the panel, a row sets the parameters and is
  remembered, typing searches, the arrows and the bar's step, a chip filters, the wheel
  scrolls the list, SAVE… saves with tags, RENAME and TAGS in place, DELETE after asking,
  RESTORE, the host's shortcuts left to it).
- Checked on the Mac, 2026-10-02: clap-validator, pluginval at strictness 10 (its GUI tests
  skipped while Live was open) and Steinberg's validator pass; in Live the drawer takes the
  keyboard and the search works (the owner).

**Owner decisions, 2026-10-02, later** (trying it in Live): the drawer opens downward,
extending the plug-in's window; the strip's "NOT THE INSTRUMENT'S" read oddly: it says
CA-72; a double click on a preset chooses it and closes the drawer.

**Agent decisions:**
- The drawer opens below the bar: the editor asks the host to resize (as the grip does),
  its size counting the drawer while it is open (`drawer_height`: 845 panel units, 423
  pixels at 1720 wide), and the window follows at once (baseview on macOS reports no
  resize of its own making: R5). It slides down from under the bar; it shuts at once, the
  window shrinking back. A host that will not resize keeps the window and the drawer
  slides up over the panel's lower part as before; the bar's arrow points where the
  drawer goes. Tests: the window's height with the drawer and without, a double click, a
  host refusing to resize.

## R11. Ten POLY voices in real time
**Owner decisions, 2026-10-02.** Every preset should play ten POLY voices on the owner's
Mac without trouble, as a VST3 in Ableton Live; on Windows in Live, POLY had dropped out
constantly. The Mac (Apple M4 Pro) is the acceptance machine, Live's buffer there 256
samples: in Ableton Live 12 at 48 kHz, the VST3 with POLY on and VOICES at 10 plays each of
the 17 factory presets with ten-note chords (held, and changing every half second) for 60 s
with no dropout and no CPU overload, at 256 samples. Headless, each preset at ten voices
takes at most 60 % of the audio thread's period, worst block included. Shown both
measurements, the owner approved:
- **Worker threads of the plug-in's own, on by default**, reversing R7's "the plug-in starts
  no threads".
- **FEEDBACK's new preamplifier model** inside the loop (below): not the same samples as
  R8's, but within the circuit's own spread.

**Agent decisions, 2026-10-02** (not separately approved):

### Measuring
- `tests/preset_cost.rs` (the engine as a host calls it, as fast as it goes on one thread)
  and `tests/poly_bench.rs` (paced, a block a period on a promoted thread) as before, now
  playing their blocks through `Engine::render` between events, as the plug-in does.
  `poly_bench` also plays a factory preset (`CA72_PRESET`), shares the voices with workers
  (`CA72_WORKERS`, each block given the plug-in's deadline) and can put its own thread in a
  work interval as a CoreAudio IO thread is (`CA72_HOST_WG`). Both share the presets'
  controls and chords (`tests/common`).
- `tests/preset_render.rs` renders every factory preset at ten voices and compares it with
  a reference render: the same to the bit, or the difference's RMS and peak, the level and
  the largest third-octave band's change. Every change below was checked against the code
  before it this way.
- The Mac with Live closed, but not idle: other applications kept the load average at 4 to
  6 throughout (the window server among them). The same conditions before and after.

### Before (the Mac, 2026-10-02, the code before this record)
Each preset at ten voices, 256-frame blocks at 48 kHz (`preset_cost`, one thread, as fast
as it goes; the share of one core real time takes, and the worst block's share of the
period), and `poly_bench`'s own panel at ten voices (paced, 60 s):

| | Mean, of the period | p99.9 | Worst | Late blocks |
|---|---:|---:|---:|---:|
| 256 frames (5.33 ms) | 88.0 % | 135 % | 138 % | 1,706 of 11,250 |
| 512 frames (10.67 ms) | 88.1 % | 130 % | 135 % | 764 of 5,625 |

The strip's count: 6 voices at 256 frames, none with FEEDBACK on. `preset_cost` per preset
is in the first columns of the table under "After".

### FEEDBACK nearly free
- Inside the loop the circuit's preamplifier (R8) cost a voice four to eight times over:
  Undertow Growl took 694 % of a core at ten voices, Pulse Strut 349 %.
- Why Potato's plain model misbehaved in the loop (R8): mostly time. The circuit's four
  substeps a sample run between halfband resamplers that delay its output 44.5 samples; the
  plain model has none, so its loop is some 45 samples shorter and falls into another
  oscillation. Recorded inside the loop (the circuit's input, sample by sample) and replayed
  open-loop, the plain model's output differs from the circuit's by -2 to +5 dB (the
  error's power against the signal's); its equations run at four times the rate inside the
  circuit's own resamplers, by -19 to -36 dB (`crates/ca72/src/voice/loop_lab.rs`).
- **The model** (`preamp::Delayed`, `InLoop::Delayed`, the default in Potato with FEEDBACK
  on): the plain model's equations at twice the rate between sparse halfband resamplers,
  its input delayed so that its output comes 44.5 samples late, as the circuit's does; the
  input's coupling, C20 and the lamp at the sample rate. Built with the preamplifier, so
  switching FEEDBACK on allocates nothing. A load-dependent low clip (R57 against R46 through
  C20, which sets the circuit's lower limit) was tried and left out: no closer.
- **Proof.** Open-loop on the recorded inputs, FEEDBACK 3, 6 and 10, EXTERNAL INPUT VOLUME
  5 and 10, both FEEDBACK presets: the error -21 to -40 dB (plain: -2 to +5). In the loop
  the sound is chaotic (the circuit's own renders differ from one ANALOG seed to the next by
  up to 8 dB in a band), so the models were compared by their statistics over six seeds
  (`tests/feedback_models.rs`):

  | Preset, FEEDBACK, VOLUME | Level, dB: circuit | new | plain | Above 1 kHz, %: circuit | new | plain | Under 150 Hz, %: circuit | new | plain | Worst band, dB: new | plain |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | Pulse Strut, 3, 5 | -25.20 ±0.12 | -25.21 | -25.30 | 5.2 ±0.1 | 5.2 | 5.5 | 66.0 ±1.3 | 66.0 | 67.5 | -0.2 | -3.6 |
  | Pulse Strut, 3, 10 | -17.21 ±0.16 | -17.21 | -16.30 | 7.9 ±0.4 | 7.9 | 5.6 | 57.5 ±1.6 | 57.2 | 76.1 | -0.7 | -18.7 |
  | Pulse Strut, 6, 5 | -21.37 ±0.12 | -21.37 | -21.75 | 11.5 ±0.3 | 11.5 | 12.1 | 48.0 ±1.7 | 47.8 | 59.3 | -0.5 | +15.3 |
  | Pulse Strut, 6, 10 | -16.04 ±0.11 | -16.14 | -15.77 | 5.0 ±0.5 | 5.4 | 3.2 | 67.6 ±2.0 | 66.2 | 82.6 | -1.4 | -16.3 |
  | Pulse Strut, 10, 5 | -17.13 ±0.14 | -17.18 | -16.35 | 7.6 ±0.3 | 7.7 | 5.4 | 60.2 ±1.5 | 59.6 | 77.8 | -0.6 | -18.8 |
  | Pulse Strut, 10, 10 | -15.99 ±0.09 | -16.10 | -15.78 | 4.0 ±0.3 | 4.4 | 2.3 | 71.9 ±1.2 | 69.8 | 85.1 | -1.5 | -15.5 |
  | Undertow Growl, 3, 5 | -24.12 ±0.44 | -24.12 | -23.88 | 10.8 ±1.1 | 10.8 | 13.7 | 72.0 ±2.0 | 71.9 | 68.3 | +0.2 | -3.6 |
  | Undertow Growl, 3, 10 | -10.33 ±0.06 | -10.30 | -10.86 | 34.8 ±0.6 | 34.2 | 46.4 | 13.7 ±1.6 | 14.1 | 17.9 | +0.9 | -22.7 |
  | Undertow Growl, 6, 5 | -16.94 ±0.18 | -16.90 | -15.53 | 48.6 ±2.3 | 48.7 | 53.4 | 24.8 ±2.8 | 24.3 | 16.3 | +0.3 | +13.4 |
  | Undertow Growl, 6, 10 | -9.50 ±0.02 | -9.51 | -9.57 | 23.4 ±2.7 | 21.6 | 6.1 | 18.8 ±9.8 | 24.0 | 79.6 | +2.8 | -22.7 |
  | Undertow Growl, 10, 5 | -10.52 ±0.08 | -10.49 | -10.93 | 34.6 ±2.0 | 34.2 | 45.0 | 17.3 ±3.5 | 17.6 | 20.6 | +1.0 | -22.3 |
  | Undertow Growl, 10, 10 | -9.46 ±0.03 | -9.49 | -9.60 | 20.3 ±3.9 | 21.2 | 7.1 | 27.3 ±12.6 | 25.2 | 78.1 | -1.0 | -22.7 |

  (The circuit's mean over the six seeds and its spread, one standard deviation; the new
  model's and the plain one's means over the same seeds; the largest change of a
  third-octave band, the power over the seeds, among the bands within 40 dB of the
  strongest.) The OVERLOAD lamp's mean is the same for all three in every case (dark at
  FEEDBACK 3 and VOLUME 5, lit otherwise). Two of the circuit's own renders, seeds 1 and 2,
  differ by up to 8.2 dB in a band (Undertow Growl, FEEDBACK 6, VOLUME 10).
  At VOLUME 0 nothing reaches the preamplifier and the three are the same. R8's test passes
  as before: Potato against High Fidelity at 30 %, level 0.540 and 0.540, brightness 30.6 %
  and 28.7 % (R8 had 30.5 %).
- **Cost:** FEEDBACK now adds what the overdriven voice itself costs (its VCA's pairs, the
  filter, harder driven), about a fifth: Undertow Growl 100 % of a core at ten voices on one
  thread (84 % with FEEDBACK at 0, measured with this change), Pulse Strut 88 %.

### A voice cheaper, the same samples
- **A voice at a time.** `Engine::render` plays the samples between one event and the next:
  each POLY voice 128 samples in turn rather than every voice a sample at a time, its state
  (some 30 KB) then staying in the nearest cache, the voices summed in the same order. About
  3 % on the Mac; it is also the shape a block takes to share the voices among threads.
- **Less work, to the bit:** the noise generator not run while nothing hears it (its mixer
  channel off and the modulation mix's gain from it exactly zero, or neither modulation
  switch on; it resumes where it stopped, as an unheard oscillator does); the filter's
  Jacobian and output gradient kept from step to step (the entries always zero stay zero)
  and its first-order correction taken over the entries that are not; the bus
  conductance's resampler skipped while the conductance holds; ANALOG's per-sample
  constants worked out once a rate; the contours' transistor parameters looked up once a
  tick. 2 to 10 % (Wooden Mallet the most).
- **Measured and left out:** link-time optimisation (`lto = "fat"`, one codegen unit): 3 to
  4 % faster, but every test binary of the workspace would link that way too (R13: the
  bundles alone are, now); caching the
  filter's per-bias divisions and reading its two output tables with shared weights, and an
  oscillator's repeated exponential: no gain the measurement could see.
- **Where a voice's time goes** (Three Saw Slab): the filter about a third, the
  oscillators a fifth (the reverse sawtooth among them), the VCA a sixth, the contours a
  tenth; the noise, the keyboard, the VCA's bias and the control node a few percent each.
  Running the contours eight times less often saved 9 to 15 %. A key change costs a voice
  about a third more in its block: on a note the keyboard circuit's Newton solve after its
  contacts jump, on a release the contours' trigger section. No further change found here
  halves a voice without changing its sound; the threads below do the rest.

### Worker threads
- **How** (`crates/ca72-plugin/src/pool.rs`; safe Rust, as `ca72::threaded`): the host's
  audio thread and up to four workers share each run of samples between events. Each voice
  sits behind a mutex that the audio thread only ever tries; the engine keeps each voice's
  note, gate and age itself, so events never wait on a voice. A run is offered through
  atomics: one word holds its number, its voices and the next to take, so a voice is taken
  by a compare-and-swap on it and a worker late from one run can take nothing from the
  next. The audio thread takes voices too, until none is left (any voice no worker has
  started, it plays itself), then waits for the workers' voices, spinning, yielding, then
  napping, until the block's deadline, 0.75 of its period, and no longer: a voice not done
  by then is silent for that run and sits out until its worker lets it go; a change meant
  for it (its panel, a lifted key) waits for it. Each voice's samples go to its own buffer
  and the audio thread sums them in the voices' order, so the output is the same to the bit
  with any number of workers (`tests/workers.rs`; `preset_render.rs`, all 17). Nothing
  allocates on the audio thread (`tests/realtime.rs`).
- **When:** with POLY on, two voices or more sounding and a run of at least 8 samples
  (waking the workers takes microseconds). POLY off, the one voice stays on the host's
  thread.
- **Threads:** started when the plug-in is activated (not on the audio thread), a third of
  the processors less two, at most four (four on the Mac's 14 and on the i5-13600K's 20,
  two on 8, none below 5; **since R39** one on 3 and 4), made audio threads for the host's
  largest block
  (`ca72_rt::promote`: on macOS the time-constraint policy, on Linux `SCHED_FIFO` with the
  watchdog, on Windows time-critical priority). They spin 200 us after a run (the block's
  next run follows at once), then park; the audio thread wakes them for each run.
- **A work interval of their own (macOS).** Paced as audio is, a thread busy a third of each
  period ran about half as fast as one busy for most of it: the system's performance
  controller does not raise the clock for threads it does not know work to a deadline (on
  one paced thread a voice's 256 samples took 708 us with 3 voices sounding, 433 with 10).
  The workers therefore join an `os_workgroup` interval of their own
  (`AudioWorkIntervalCreate`; `ca72_rt::WorkInterval`), which the first of them opens at a
  block's first run with the block's period as its deadline and closes when the block's runs
  are done; a worker leaves it before it ends. A plug-in cannot join its host's audio
  workgroup: VST3 has no way to hand it over and nih-plug's CLAP wrapper none either; nor
  can it put the host's thread into one safely (a thread that ends while joined stops the
  process). How fast the host's own thread runs is the host's: in the bench, with its thread
  in an interval of its own (as a CoreAudio IO thread is), the worst blocks fall by about a
  third.
- **The strip's count** (**since R12** not shown, nor measured when the plug-in is
  activated; `engine::realtime_voices` stays for `poly_bench`) measures the engine itself
  with its workers, paced at the block's period on an audio thread of its own for a moment,
  as a host plays it (chords of as many notes as voices tried, a new one every few blocks,
  the voices still sounding taken): the most voices whose key-change blocks take, at their
  median, no more than 60 % of the period; ten tried first, then down from the estimate,
  about a fifth of a second each, once a process for each rate, block, FEEDBACK and number
  of workers. That thread is not in a workgroup, so the count does not count on the host's
  being in one. On the Mac: 10 at 256 and 1024 frames, FEEDBACK on too; 4 with the host's
  thread alone (R7's paced figure on the Linux reference machine was 4).
- **Tests:** a voice still held by a late worker sits out and the block is not held up
  for it; a worker past the deadline is not waited for; the same samples with and without
  workers; no allocation with workers, FEEDBACK switched, VOICES changed and POLY switched.

### After (the Mac, 2026-10-02)
**`preset_cost`** (each preset at ten voices, 256 frames, as fast as it goes; the share of
one core real time takes and the worst block's share of the period; before: the code before
this record, the circuit's preamplifier in FEEDBACK's loop; after: this record, one thread
and with four workers, which here are ordinary threads since the measurement is not paced):

| Preset | Before: one thread | worst block | After: one thread | worst block | Four workers | worst block |
|---|---:|---:|---:|---:|---:|---:|
| Bass | 67 % | 112 % | 59 % | 105 % | 14 % | 27 % |
| Lead | 75 % | 123 % | 68 % | 117 % | 16 % | 26 % |
| Three Saw Slab | 83 % | 131 % | 78 % | 123 % | 18 % | 30 % |
| Elastic Octaves | 72 % | 239 % | 67 % | 114 % | 15 % | 26 % |
| Pulse Strut | 349 % | 519 % | 88 % | 141 % | 21 % | 96 % |
| Undertow Growl | 694 % | 1040 % | 100 % | 208 % | 22 % | 34 % |
| Upright Pluck | 63 % | 110 % | 59 % | 213 % | 13 % | 25 % |
| Hollow Glider | 89 % | 140 % | 80 % | 127 % | 19 % | 30 % |
| Cruising Whistle | 80 % | 130 % | 74 % | 151 % | 17 % | 31 % |
| Stacked Fifths | 93 % | 141 % | 84 % | 200 % | 20 % | 78 % |
| Ringing Saw Line | 70 % | 119 % | 61 % | 123 % | 14 % | 25 % |
| Slow Horn Swell | 69 % | 279 % | 64 % | 112 % | 15 % | 87 % |
| Brass Tutti | 85 % | 134 % | 77 % | 154 % | 17 % | 29 % |
| Breath Flute | 67 % | 113 % | 62 % | 108 % | 14 % | 25 % |
| Wooden Mallet | 63 % | 102 % | 53 % | 92 % | 12 % | 21 % |
| Shoreline Wash | 73 % | 110 % | 68 % | 207 % | 15 % | 24 % |
| Slow Bow | 67 % | 125 % | 62 % | 114 % | 14 % | 26 % |

(Run as fast as it goes, its worst blocks include the other applications' turns: the
workers here are ordinary threads, so 78 to 96 % in three presets is that; the paced runs
below, on audio threads, are the measure.)

**`poly_bench`, its own panel** (ten voices, paced on a promoted thread, 60 s):

| | Mean | p99.9 | Worst | Late blocks |
|---|---:|---:|---:|---:|
| 256 frames, before | 88.0 % | 135 % | 138 % | 1,706 of 11,250 |
| 256 frames, one thread | 82.7 % | 132 % | 138 % | 781 of 11,250 |
| 256 frames, four workers | 33.2 % | 65 % | 75 % | 0 of 11,250 |
| 512 frames, before | 88.1 % | 130 % | 135 % | 764 of 5,625 |
| 512 frames, one thread | 81.6 % | 123 % | 127 % | 361 of 5,625 |
| 512 frames, four workers | 31.4 % | 57 % | 61 % | 0 of 5,625 |

**`poly_bench`, every preset** (ten voices, four workers, paced on a promoted thread: 30 s
at 256 and 512 frames with the bench's own thread outside any workgroup, and 20 s at 256
with it in a work interval of its own, as a CoreAudio IO thread is; shares of the period):

| Preset | 256: mean | p99.9 | worst | late | 512: mean | worst | late | 256, host in an interval: mean | worst |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Bass | 32 % | 68 % | 74 % | 0 of 5,625 | 31 % | 62 % | 0 of 2,812 | 25 % | 47 % |
| Lead | 38 % | 68 % | 75 % | 0 of 5,625 | 32 % | 60 % | 0 of 2,812 | 28 % | 49 % |
| Three Saw Slab | 42 % | 71 % | 93 % | 0 of 5,625 | 31 % | 59 % | 0 of 2,812 | 29 % | 47 % |
| Elastic Octaves | 37 % | 72 % | 82 % | 0 of 5,625 | 33 % | 62 % | 0 of 2,812 | 27 % | 51 % |
| Pulse Strut | 39 % | 75 % | 81 % | 0 of 5,625 | 31 % | 66 % | 0 of 2,812 | 31 % | 51 % |
| Undertow Growl | 41 % | 71 % | 79 % | 0 of 5,625 | 31 % | 58 % | 0 of 2,812 | 31 % | 50 % |
| Upright Pluck | 33 % | 66 % | 73 % | 0 of 5,625 | 30 % | 75 % | 0 of 2,812 | 22 % | 49 % |
| Hollow Glider | 42 % | 73 % | 75 % | 0 of 5,625 | 31 % | 59 % | 0 of 2,812 | 29 % | 49 % |
| Cruising Whistle | 40 % | 74 % | 82 % | 0 of 5,625 | 31 % | 56 % | 0 of 2,812 | 29 % | 48 % |
| Stacked Fifths | 43 % | 73 % | 76 % | 0 of 5,625 | 31 % | 56 % | 0 of 2,812 | 31 % | 49 % |
| Ringing Saw Line | 34 % | 66 % | 78 % | 0 of 5,625 | 30 % | 56 % | 0 of 2,812 | 25 % | 45 % |
| Slow Horn Swell | 37 % | 69 % | 84 % | 0 of 5,625 | 30 % | 58 % | 0 of 2,812 | 25 % | 49 % |
| Brass Tutti | 41 % | 73 % | 84 % | 0 of 5,625 | 31 % | 58 % | 0 of 2,812 | 29 % | 47 % |
| Breath Flute | 36 % | 68 % | 76 % | 0 of 5,625 | 29 % | 58 % | 0 of 2,812 | 26 % | 47 % |
| Wooden Mallet | 31 % | 59 % | 65 % | 0 of 5,625 | 27 % | 59 % | 0 of 2,812 | 21 % | 41 % |
| Shoreline Wash | 39 % | 68 % | 73 % | 0 of 5,625 | 30 % | 57 % | 0 of 2,812 | 25 % | 45 % |
| Slow Bow | 35 % | 68 % | 75 % | 0 of 5,625 | 31 % | 58 % | 0 of 2,812 | 24 % | 51 % |

No preset had a late block. With the host's thread in an interval every preset's worst
block is under 60 % of the period (41 to 51 %); with it outside one, the means stay under
half the period but the worst blocks, at the chord changes, reach 65 to 93 % at 256 frames
and 56 to 75 % at 512.

**The strip's count:** 10 voices at 256 and 1024 frames, FEEDBACK on too (6 and none
before); 4 with the host's thread alone.

**Old against new** (`preset_render`, every factory preset at ten voices, 4 s, against a
render of the code before): 15 the same to the bit; Pulse Strut and Undertow Growl, whose
FEEDBACK loop now runs the new preamplifier model: level +0.10 and -0.04 dB, the largest
third-octave band's change -0.91 dB at 1.26 kHz and +1.31 dB at 20 kHz; their difference
signals are about as loud as the renders (+0.4 and +2.5 dB), as two renders of a chaotic
loop are once they part (above).

**Evidence (the Mac, 2026-10-02, the release bundles of this version, Live closed):**
clap-validator 0.4.1, 37 passed, 0 failed, 7 skipped; pluginval 1.0.4 at strictness 10,
SUCCESS, its editor tests run this time; Steinberg's validator (SDK 3.8.1), 47 passed, 0
failed. The workspace's tests (155 passed, 0 failed, 20 by hand) and clippy pass on the
Mac. The bundles are installed for Live.

### Limitations
- The host's own thread: in a host whose audio threads are not in a workgroup, its share of
  the voices runs slower (paced and lightly loaded, the system clocks it down), and the
  worst blocks at a chord change rise to 65 to 93 % of the period at 256 frames (no block
  late in these runs); with it in one, 41 to 51 %. Whether Live 12's threads are is for the
  check in Live to show.
- Windows and Linux were not measured for this record (the Linux reference machine and the
  Windows one, an i5-13600K): the
  workers start there too (four on the i5's 20 processors), without a work interval.
- A worker held up past the deadline (a machine overloaded by other work) costs that voice
  its samples for the run: a gap in one voice rather than a dropout of all.
- VOICES still defaults to 4.

### For the DAW's instrument (its own copy of the model; porting left for later)
- The FEEDBACK finding and model, if its loop runs the circuit's preamplifier in Potato
  (`preamp::Delayed`, `InLoop`; the open-loop lab and the in-loop comparison).
- The bit-exact savings: the noise while unheard (`ControlPart`), the filter's sparse
  Jacobian and held conductance (`vcf.rs`), the character's constants (the DAW's character
  module), the contours' devices once a tick.
- The engine's runs between events and its threads belong to the DAW's own engine and pool,
  not to the model; the macOS work interval (`ca72_rt::WorkInterval`) is a candidate for
  the DAW's real-time threads.

## R12. One row under the panel; a preset holds POLY, VOICES, ANALOG and SPREAD
**Owner decisions, 2026-10-02**, trying R11's build in Live: the strip's real-time count
("10 in real time here ..."), which the owner found unhelpful and a waste of space, is
gone, and CA-72's label with it, the presets' selector brought up into the strip's row; the
presets' list scrolls with the mouse wheel and touch gestures (it did with neither); POLY,
VOICES, ANALOG and SPREAD are saved with a preset.

**Agent decisions, 2026-10-02** (not separately approved):
- **One row.** The strip's left (to 1110 panel units) is the presets' selector: its star,
  the previous preset, the name (680 units, the drawer's arrow inside), the next and SAVE…;
  then POLY and VOICES where the count was; ANALOG and SPREAD where they were. The bar's
  PRESET label went: the star, the arrows and SAVE… say what it is. The window is 80 panel
  units shorter (577 pixels tall at 1720 wide, as before R10's bar); the drawer opens below
  the strip. `ca72-panel`: `presets::bar_svg` draws the selector into the strip's frame
  (one renderer instead of two); `strip::hit` leaves the selector to `presets::bar_hit`.
  Tests: the row's parts do not overlap (each label measured in the strip's face), the
  selector's controls found where drawn, the window's heights.
- **No real-time count**: the strip shows none and VOICES is no longer marked; the plug-in
  no longer measures one when it is activated (half a second saved). R11's measurement stays
  as `engine::realtime_voices`, for `poly_bench`.
- **The list scrolls a row for each row's travel**: a wheel's lines as they come (on macOS a
  slow turn gives fractions of one), a trackpad's or a Magic Mouse's points as rows of the
  list at the window's size (a row is 62 panel units), the rest kept for the next event; a
  turn the other way starts again. Each such event is a few points, and the half-notch
  threshold before dropped every one. Test: `small_scrolls_add_up_to_rows`.
- **A preset sets POLY, VOICES, ANALOG and SPREAD** like every other parameter, to its value
  or the default (R10 left POLY and VOICES alone when a preset did not name them, and the
  factory presets named none of the four, so choosing one put ANALOG and SPREAD to 0). A
  preset saved already held all four. Every factory preset now names them: Brass Tutti
  POLY on, 8 voices, ANALOG 40 %, SPREAD 50 %; Wooden Mallet POLY on, 8, 20 %, 60 %; the
  other fifteen, documented mono patches, POLY off, 4 voices, ANALOG and SPREAD 0 (as they
  were). Test: `presets::tests`.
- **The DAW** carries the factory file byte for byte (R10); its copy now differs by these
  four keys in each preset, and its instrument's choosing of a preset by the same rule:
  both to port.

## R13. How much lighter ten voices can be
**Owner's question, 2026-10-02**, trying R11's build in Live: Live's CPU meter reaches
about 40 % with FEEDBACK and ten POLY voices. The owner asked whether more speed could be
had without losing audio quality, or large gains from a small concession, concerned that it
was too heavy for many people's computers. Two changes that keep the sound are built; the
concessions were measured and left for the owner to decide (any audible change is theirs).

**Agent decisions, 2026-10-02** (not separately approved):

### Measuring
- `preset_cost` (ten voices on one thread, as fast as it goes), the least of three 5 s
  runs, the builds interleaved. Live was open with the owner's Set, so a share of a core
  here is a few percent above R11's (Live closed): compare within a row.
- The sound against a render of the code before (`preset_render`, every factory preset at
  ten voices, 4 s), as in R11.

### The same sound, a little less work
- **The pairs' tanh, fast inside a tick.** A transistor pair's tanh anchor (`PairWarm`: the
  VCA's three pairs, the filter's in High Fidelity) takes `ulp::tanh_in_tick`: inside a High
  Fidelity or Potato part's tick `fast::tanh` (within 2e-14 of libm's, relatively), as
  `ulp::exp` already was; No Compromises and the factory calibration (outside a tick) keep
  libm's. A voice in Potato takes some 35,000 of them a second (Bass). Against the code
  before: 14 presets the same to the bit, Upright Pluck and Pulse Strut differ at -263 and
  -166 dB; Undertow Growl's FEEDBACK loop, chaotic, parts from the old render as any change
  of rounding makes it do (its difference as loud as the render, the level +0.04 dB, the
  largest third-octave band's change +1.3 dB at 20 Hz, inside R11's spread from one ANALOG
  seed to the next of up to 8 dB). 1 to 2 % faster.
- **The bundles link-time optimised:** `[profile.bundle]` (release, `lto = "fat"`, one
  codegen unit), built by `cargo xtask bundle ca72-plugin --profile bundle` in CI, the
  README and `scripts/validate.sh`; the tests stay on `release`, which links far faster.
  Every preset the same to the bit as `release`. 2 to 4 % faster. Its build output takes
  about half a gigabyte more under `target/bundle`.
- Together, a share of one core at ten voices:

  | Preset | Before | Fast tanh | And link-time optimised | Change |
  |---|---:|---:|---:|---:|
  | Undertow Growl | 100.1 % | 98.1 % | 94.5 % | -5.6 % |
  | Pulse Strut | 89.8 % | 88.4 % | 84.9 % | -5.5 % |
  | Three Saw Slab | 79.2 % | 78.6 % | 75.7 % | -4.4 % |
  | Brass Tutti | 76.6 % | 75.4 % | 72.5 % | -5.4 % |
  | Wooden Mallet | 53.5 % | 52.9 % | 51.2 % | -4.3 % |

- **Measured and not built:** the workers spinning 50 us rather than 200 after a run
  before they park. Four workers spin up to 0.8 ms a block, some 15 % of a core at 256
  frames that Live's meter does not show but other tracks' threads could use; whether the
  shorter spin makes a block late wants a paced run with Live closed.
- **Nothing more to the bit** that the measurement can see (R11 tried the filter's
  divisions and tables and the oscillators' exponentials). A large gain with the same
  samples would need the voices computed side by side, two to a NEON register (four to
  AVX2's), each operation the same in every lane, the solvers' iterations masked lane by
  lane: a rewrite of the voice's data and its solvers, its gain not measured.

### Concessions, measured (none built)
Each switched on alone in Potato for the measurement (and removed): its cost against the
same build without it (Undertow Growl, Three Saw Slab, Wooden Mallet, Brass Tutti), and
its sound against the reference render (17 presets):

| Concession | Cost | Sound |
|---|---:|---|
| The filter at the host's rate, not twice it | -12 to -23 % | the top octave 2 to 8 dB down in most presets; Cruising Whistle +3.5 dB at 12.7 kHz; FEEDBACK's loop another (Pulse Strut +15 dB at 630 Hz, Undertow Growl +24 dB at 20 Hz) |
| The filter's Newton solve one iteration a step, not two | -2 to -3 % (Undertow Growl -18 %) | the ordinary presets within 0.5 dB in every band; FEEDBACK's loop another (Undertow Growl +19 dB at 20 Hz) |
| The contours at a sixteenth of the rate, not an eighth | -2 to -8 % | attacks click: Bass +10 dB at 10 kHz, Upright Pluck +4 dB at 6.3 kHz |
| The keyboard circuit at a sixteenth of the rate | about -1 % | not compared |
| All of them | -22 to -38 % | all of the above |

So no small concession gives a large gain: the circuit's cheapest voice is 1.3 to 1.6
times cheaper and audibly another. A voice's cost is the circuit itself (R11: the filter a
third, the oscillators a fifth, the VCA a sixth). Five times cheaper or more would take
another model of the voice for POLY (the filter as a ladder of one-pole stages around one
nonlinearity, the oscillators as band-limited waveforms): another instrument than the
circuit's, and a second quality in the plug-in against R1's one; the owner's to decide.

### What ten voices ask of a machine
- On the M4 Pro a voice takes 5 to 10 % of a core (Wooden Mallet the least, Undertow
  Growl the most): ten voices about half a core's work to a core's, shared by the host's
  thread and the workers (R11: a third of the processors less two, at most four: none
  below 5 processors, one at 5 to 7, two at 8 to 10, three at 11 to 13; **since R39** one
  at 3 and 4 too). Live's meter shows
  the host's thread's block against its period, waiting for the workers' voices included:
  about a fifth of the work at ten voices with four workers, and its worst blocks.
- A machine of 8 processors (two workers) shares the voices three ways; one of 4 (none)
  plays them all on the host's thread, where ten voices of the heavier presets would need
  a core faster than the M4 Pro's (one of its cores holds five or six of them within 60 %
  of the period; a slower core fewer). Windows and Linux are still not measured.
- VOICES defaults to 4; Brass Tutti and Wooden Mallet set 8 (R12).

**Evidence (the Mac, 2026-10-02, the bundles of this version built with `--profile
bundle`, Live open):** clap-validator 0.4.1, 37 passed, 0 failed, 7 skipped; pluginval
1.0.4 at strictness 10, SUCCESS (its editor tests skipped while Live was open); Steinberg's
validator (SDK 3.8.1), 47 passed, 0 failed. The workspace's tests (157 passed, 0 failed, 20
by hand) and clippy pass. The bundles, with R12's changes, are installed for Live.

### For the DAW's instrument
- The pairs' fast tanh inside a tick (`ulp::tanh_in_tick`, `PairWarm`), if its copy of the
  model has the anchors.

## R14. ANALOG is ENTROPY
**Owner decisions, 2026-10-03:** ANALOG is renamed ENTROPY; its parameter id may change
too, as long as it is the same everywhere; and no compatibility with what was saved before
(below).

**Agent decisions, 2026-10-03** (not separately approved):
- **ENTROPY everywhere:** the strip's label, the parameter's name in a host ("Entropy") and
  its id (`entropy`), the presets' key (`entropy`, the factory file's seventeen included),
  the README, and the code's names (the engine's control and `ENTROPY_DEPTH` with its kin,
  the character's `entropy` figures, `crates/ca72/tests/entropy.rs`). What it does is
  unchanged. The strip's label fits before its slider (the row's parts measured as in
  R12).
- **No compatibility kept** (the owner, 2026-10-03, saw no need for it, since no one had
  used the plug-in outside testing): a session or a preset saved with ANALOG's `analog`
  opens with ENTROPY at its default, and a host's automation of ANALOG no longer reaches
  it.
- The records before this one keep the name ANALOG, as written then.
- **The DAW** reads the plug-in's presets by the same keys (R10): until its instrument
  takes the name too (its control, its key and its copy of the factory file), each
  program's presets hold the amount under a key the other does not know. To port, with
  R12's keys.

**Evidence (the Mac, 2026-10-03, the bundles of this version, Live open):** clap-validator
0.4.1, 37 passed, 0 failed, 7 skipped; pluginval 1.0.4 at strictness 10, SUCCESS (its editor
tests skipped while Live was open); Steinberg's validator, 47 passed, 0 failed. The
workspace's tests (157 passed, 0 failed, 20 by hand) and clippy pass. The bundles are
installed for Live.

## R15. Six factory presets: the launch film's drums and riser
**Owner decisions, 2026-10-03:** the agent made the CA-72's launch film with every sound on
the model, the drums included. The owner then asked for these sounds to be added to the
factory presets, in the DAW and in the VST3 and CLAP plug-ins, and for the film to make
clear that every sound is the CA-72.

**Agent decisions, 2026-10-03** (not separately approved):
- **Six presets after the seventeen, 23 in all:** Ladder Kick, Noise Snare, Closed Hat,
  Open Hat, Noise Crash and Pink Riser, tagged `drums` (with `kick`, `snare`, `hi-hat`,
  `cymbal`, `noise`, `percussive`) or `fx` and `riser`. Every control is named, as in the
  documented patches; POLY off, VOICES 4, ENTROPY and SPREAD 0 (R12). A preset is one
  monophonic instrument, so a kit takes one instance a drum.
- **Each works from the panel alone.** A preset holds neither the PITCH wheel nor
  automation, so the film's first kick (a triangle with the PITCH wheel swept 16 semitones
  down on each hit) and riser (CUTOFF automated over 8 s) were redesigned to do it
  themselves (measured on the model):
  - **Ladder Kick:** a triangle at 32' into a filter whose EMPHASIS (9.6) sits just under
    oscillation. A 0.2 FILTER DECAY sweeps the ring from about 540 Hz into the low end
    within 30 ms, over the triangle's 55 Hz on MIDI note 57. Longer, wider sweeps (EMPHASIS
    8 to 10, AMOUNT OF CONTOUR 6 to 8) lingered in the mids: their energy below 120 Hz was 8
    to 19 dB under the whole, against 1.7 dB here.
  - **Pink Riser:** FILTER ATTACK at 10 opens a resonant pink noise (EMPHASIS 7, CUTOFF
    -3.5, AMOUNT OF CONTOUR 10) over about 4.5 s, while the loudness swells (ATTACK 6).
  - The snare, the hats and the crash are the film's patches: noise with the filter open
    (a triangle at 7 for the snare's body, MIDI 54 about 185 Hz), told apart by DECAY.
- **The film was rendered again with these presets as they are** (no value changed, no
  automation), so its drums are these.
- **Levelled as the first seventeen:** MAIN OUTPUT VOLUME for about -18 LUFS momentary
  maximum on a phrase in each one's register. Noise Crash 9.7 (-17.9). The other five are
  under it at 10, quiet by nature: Ladder Kick -22.9 (LUFS weighs the lows lightly; a
  second triangle added 0.6 dB), Noise Snare -20.6, Closed Hat -20.7, Open Hat -20.8, Pink
  Riser -25.6.
- **The editor's test of the drawer over the panel** clicked "Lead" by name. With Closed
  Hat and Ladder Kick sorting before it, Lead left the list's first page and the test failed.
  It now clicks the first page's second row, whichever preset that is.
- **The DAW** carries the same six in its copy of the file (on a branch of its own),
  without R12's four keys, which it has not ported. Its golden renders record each.

**Evidence (the Mac, 2026-10-03, the bundles of this version, Live open):** clap-validator
0.4.1, 37 passed, 0 failed, 7 skipped; pluginval 1.0.4 at strictness 10, SUCCESS;
Steinberg's validator, 47 passed, 0 failed. The workspace's tests (157 passed, 0 failed, 20
by hand), clippy and rustfmt pass. Both installed bundles carry the 23 presets. They are
installed for Live, the ones they replaced kept aside.

## R16. Warped Pad, the IDM film's pad, as a factory preset
**Owner decision, 2026-10-03:** after the launch film's IDM version, the owner asked for
its pad to become a preset if it was not one already.

**Agent decisions, 2026-10-03** (not separately approved):
- **Warped Pad, the 24th preset:** two sawtooths a hair apart (oscillator 2 at +0.1), the
  filter opened slowly (CUTOFF -1.5, EMPHASIS 2, AMOUNT OF CONTOUR 2, ATTACK 5) and a slow
  vibrato from oscillator 3 at LO (MOD wheel 0.03). POLY is on with eight voices, ENTROPY 85
  and SPREAD 80; the drift is the point of it. The film automated CUTOFF and moved ENTROPY
  between 60 and 100; the preset holds the values of the film's last section.
- **Levelled as the others:** MAIN OUTPUT VOLUME 7.8, -18.0 LUFS momentary maximum on three
  held six-note chords.
- The DAW's copy names POLY, VOICES, ENTROPY and SPREAD for this preset, since the pad's
  sound depends on them. It is the first preset in that copy to name all four.

**Evidence (the Mac, 2026-10-03, the bundles of this version, Live open):** clap-validator
0.4.1, 37 passed, 0 failed, 7 skipped; pluginval 1.0.4 at strictness 10, SUCCESS;
Steinberg's validator, 47 passed, 0 failed. The workspace's tests (157 passed, 0 failed, 20
by hand) pass. Both installed bundles carry the 24 presets.

## R17. Installers, and a README that starts with them
**Owner's request, 2026-10-03**, preparing the open-source launch: installation and setup
instructions that are clear and concise, as easy as possible for people, perhaps with an
installer.

**Agent decisions, 2026-10-03** (not separately approved):
- **An installer for each platform**, made from the bundles by `scripts/package.sh` on the
  platform it runs on, and by CI on every run (artifacts `CA-72-<platform>-installer`):
  - **macOS:** `CA-72-<version>-macOS.pkg` (pkgbuild, productbuild), universal, macOS 11 or
    later. It installs `CA-72.vst3` and `CA-72.clap` for every user into
    `/Library/Audio/Plug-Ins/VST3` and `.../CLAP`, each format a choice; it asks for an
    administrator's password, as plug-in installers do. The bundles are not relocatable
    (Installer would otherwise put them wherever it finds a bundle of the same identifier,
    a build folder say) and not version-checked (any version installs over any other).
  - **Windows:** `CA-72-<version>-Windows-setup.exe` (Inno Setup 6,
    `scripts/installer/CA-72.iss`), 64-bit, Windows 10 or later. It installs into
    `Common Files\VST3` and `Common Files\CLAP`, each format a component, with an
    uninstaller in Settings' Apps and the licence and notices in `Program Files\Idle
    Foundry\CA-72`. Its AppId must never change.
  - **Linux:** `CA-72-<version>-Linux-x86_64.tar.gz` with `install.sh`: for the user into
    `~/.vst3` and `~/.clap`, `--system` into `/usr/lib/vst3` and `/usr/lib/clap`,
    `--uninstall`. It warns when X11's client libraries are missing.
- **Unsigned for now**, as R4 waits for the owner's Developer ID. The README says how to
  open them past Gatekeeper and SmartScreen once. Files an installer writes carry no
  quarantine flag, so the README's `xattr` step is gone. `package.sh` signs and notarises
  when given identities and a notarytool profile (`CA72_MACOS_SIGN_APP`,
  `CA72_MACOS_SIGN_INSTALLER`, `CA72_NOTARY_PROFILE`); CI does not yet, which needs the
  certificates as secrets.
- **The bundles' Info.plist on macOS:** nih-plug's bundler writes the identifier
  `com.nih-plug.ca72-plugin` and version 1.0.0. The installer's copies carry
  `com.idlefoundry.ca-72.vst3` and `.clap`, the crate's version and macOS 11 as their
  minimum, and are signed again (ad hoc).
- **macOS 11 is the minimum.** The audio workgroups (R11) are macOS 11's and are linked as
  required, while the toolchain's default for the Intel slice is 10.12: it would have
  failed to load on 10.12 to 10.15. `.cargo/config.toml` sets `MACOSX_DEPLOYMENT_TARGET`
  to 11.0 (checked: both slices say 11.0).
- **Linux's bundles are built on Ubuntu 22.04** in CI (24.04 before), so that they load
  where the C library is glibc 2.35 or later.
- **Third-party notices.** The plug-ins' binaries include some 190 crates (MIT,
  Apache-2.0, BSD, ISC, Zlib and others) whose licences ask for their notices to go with
  every copy. `scripts/notices.py` writes `THIRD-PARTY-NOTICES.txt` from Cargo.lock for
  every platform released: each crate's own licence files, MIT's where a crate offers it
  among others, the standard MIT text with its authors where a crate ships none, and the
  font's licence. CI checks that it is current. Every installer carries it and LICENSE
  beside the plug-ins and inside each bundle that has room.
- **A release:** a tag `v<version>` runs CI and drafts a GitHub release of the three
  installers, their SHA-256 sums and the notices, for the owner to publish.
- **The README** opens with downloading and running the installer, the warnings while
  unsigned, the folders, and uninstalling. Building from source lists each platform's
  prerequisites (rustup; Xcode's command line tools, Visual Studio's Build Tools, or
  Linux's packages for Debian and Ubuntu, Fedora and Arch), cloning, how long the build
  and tests take, and `validate.sh`'s own needs.

**Evidence (2026-10-03).** The README's Building and Testing followed on the three
reference machines, from a fresh clone from GitHub with an empty cargo cache:

| | Mac (M4 Pro, macOS 27) | Linux (Ryzen 7 7800X3D) | Windows 11 (i5-13600K) |
|---|---|---|---|
| Bundles (`--profile bundle`) | 93 s | 35 s | 69 s |
| `cargo test --workspace` | 157 passed, 0 failed (164 s) | 158 passed, 0 failed (470 s) | 157 passed, 0 failed (218 s) |
| Standalone built | yes | yes | yes |
| clap-validator 0.4.1 | 37 passed, 7 skipped | 37 passed, 7 skipped | 36 passed, 8 skipped (one runs on Unix only) |
| pluginval 1.0.4, strictness 10 | SUCCESS, editor tests skipped (Live open) | SUCCESS, editor tests skipped (no display) | SUCCESS, editor tests included |
| Steinberg's validator, SDK 3.8.1 | 47 passed | 47 passed | not built: the SDK needs Visual Studio, and this machine builds with MinGW |
| Installed copies validated | yes | yes | CLAP yes |

On Windows, pluginval's editor test crashed when run over SSH (no desktop) and passed in
the desktop session. The installers, made by `package.sh` from this branch:
- **macOS:** the `.pkg`'s payload is the staged bundles to the byte, each bundle's
  signature valid within it, both slices macOS 11.0; its bundles pass clap-validator (37
  passed), pluginval at strictness 10 (SUCCESS, editor tests skipped while Live was open)
  and Steinberg's validator (47 passed). Not installed on the Mac (that needs an
  administrator, and the owner's own build in `~/Library` was in use by Live), and the
  Intel slice not run (no Rosetta).
- **Linux:** unpacked and `./install.sh` run, the installed copies pass clap-validator and
  Steinberg's validator; installing again, `--uninstall` and `--help` behave.
- **Windows** (Inno Setup 6.7.3 installed for it, with the owner's approval): the setup
  program installed silently (exit 0) into `Common Files\VST3` and `\CLAP` with the licence,
  the notices and an uninstaller in `Program Files\Idle Foundry\CA-72`, and an entry
  "CA-72 0.1.0, Idle Foundry" in Settings' Apps; the installed copies pass clap-validator
  and pluginval at strictness 10; it installs again over itself (VST3 only); its
  uninstaller removes everything it wrote, the folders it made included.

## R18. Before the release: a review's findings fixed, the name plate fixed, a new home
**Owner decisions, 2026-10-03.** Preparing the open-source launch, the owner asked for a
review of the code and an install test on the three reference machines, and for every
issue the review found to be fixed; and for R1's name plate to stop turning over. The
CA-72's public home is the Idle Foundry organisation, `github.com/idlefoundry/ca-72`. R17's
installers were approved, and the 24 presets (R15, R16), the installers and these fixes go
to `main` together.

**Agent decisions, 2026-10-03** (not separately approved), by part:

### Three factory presets, as the owner changed them
**Owner decision, 2026-10-03:** the owner saved three factory presets anew from the panel
and asked for the changes to be made permanent. In `sounds/presets.toml`, to the values the
controls show:
- **Slow Horn Swell:** POLY on, VOICES 10, ENTROPY 25 % (saved as 24.96).
- **Undertow Growl:** FEEDBACK 3.8 (saved as 3.8022) and MAIN OUTPUT VOLUME 9.17 (9.167),
  so no longer levelled as R15's presets were.
- **Wooden Mallet:** POLY off (R12 had it on with eight voices; VOICES 8, ENTROPY 20 and
  SPREAD 60 stay for when POLY is switched on).
- **Upright Pluck** (asked later the same day): the three oscillators' VOLUME at 10 (8
  before). Only oscillator 1 sounds in it, now louder into the mixer.

### The name plate
- It no longer turns over (R1's "Model DEEZ" is gone, with its click and its animation). It
  always shows CA-72 and its maker, at the width it had (261 units), so the panel is the
  same to the pixel.

### The voices on the audio thread
- **Exports and freezes wait for every voice.** The deadline (R11) applies only in real time:
  the processing mode is read each block (nih-plug's `ProcessContext::process_mode`, below),
  since CLAP may switch to offline without initialising the plug-in again. A voice that
  misses a run keeps its level for the silence test, so it is not freed and its tail is not
  cut.
- **After the deadline the block's later runs play on the host's thread**, rather than being
  handed to workers and lost: the block is late rather than voices silent.
- **A sample that is not finite** on the side chain is read as 0. A voice whose output goes
  non-finite is silent until a clean voice takes its place: a spare for each voice, made off
  the audio thread when the plug-in is prepared and made again on nih-plug's background
  thread after use. The output is never NaN. (Ten spare voices an instance: memory, no
  work.)
- **All Sound Off (CC 120) silences at once:** a 5 ms fade, every sounding voice replaced by a
  clean one, then POWER's 10 ms fade in. All notes off (CC 123) still releases the notes.
- **A state loaded into a running instance** (CLAP and VST3 initialise the plug-in again
  for it) keeps its workers when their count and period are unchanged, so a host's preset
  change no longer drops out; a new seed draws ENTROPY's characters again, so a session
  plays as it did when saved. A session saved before R14 opens with ENTROPY at its default.
- A retrigger due for a voice that did not play a run is kept for the next.

### The plug-in's threads
- **Every instance in a process shares one budget of workers** (R11's count, at most four):
  an instance that finds none plays its voices on the host's thread. An instance with POLY
  off still takes its share when activated (starting workers when POLY turns on would mean
  starting them off the audio thread on demand; not done).
- **A late worker cannot touch a later run:** a worker that finds its run closed leaves the
  voice alone (that voice does not advance for the run), and each voice's mark is its own.
- **Workers take the host thread's flush-to-zero bits** for each run, so a voice's samples
  are the same on any thread. DAZ is not forced where the host has not set it, for the same
  reason.
- **Windows:** workers join MMCSS's "Pro Audio" task at critical priority (time-critical
  priority if that fails), and the host's thread spins rather than sleeping while it waits.
- **Linux:** the watchdog (R11) demotes only the threads it promoted, never the host's or
  other plug-ins'; its messages begin `ca72:`, and the real-time limit's message prints at
  most once. Its threads park while no promoted thread lives.
- Nits: no Mach port leaked a worker; a worker that fails to start stops the others; the
  lab's threads are `ca72-*`.

### The presets' library (R10)
- **Names:** `factory` (the overlay's file) and Windows' device names (CON, NUL, COM1 and
  so on) are refused on every system, in any case; names are one preset regardless of case
  on every file system.
- **Writes go to the preset's own file**, which keeps its name unless no other file has the
  new one; a name read from a file is checked as one typed (`../x` or an absolute path is
  reported, not listed).
- **An overlay that cannot be read is left as it is,** and marking a factory preset reports
  why; the overlay is edited in place, keeping keys this version does not know.
- **An edit stays with its preset** (by name and origin) while the list filters, refreshes or
  loses it; a preset gone meanwhile is reported.
- **The library is read again only when its files change**, and search and tags work on what
  was read, off the disk.
- **MIDI BEND RANGE is the player's,** not the sound's: choosing a preset that does not name
  it leaves it as it is, and the plug-in's own saves no longer write it (a file that names
  it still sets it).
- **The factory file is `sounds/presets.toml`,** its comments without trademarks or the
  DAW's name (R1: the file is inside every bundle); Ladder Kick is "a booming electronic
  kick". R10's "byte for byte" with the DAW's copy no longer holds for the comments.

### The editor (R5, R6, R10)
- **Linux:** the drawer no longer takes the keyboard by asking for focus (baseview's X11
  window does not implement it and panicked, closing the editor). X11 gives the editor the
  keys while the pointer is over it.
- **Windows:** when the drawer shuts it gives the keyboard back to the window that had it.
- **Scale:** on Windows and Linux the editor uses the host's scale, or exactly 1 when the
  host gives none (Live), so the drawing, the window and the pointer agree (at 150 % the
  pointer had landed at two thirds of its place).
- **Linux's first size** is the monitor the pointer is on, else the primary (RandR, through
  x11rb), less the desktop's panels; not the whole X screen.
- **The drawer opens over the panel** when the window with it below would not fit on the
  screen, or when the host refuses to resize (VST3 now reports a refusal on macOS and
  Windows; below).
- **Gestures never nest:** a wheel's gesture, a drag or a slider is ended before any other
  begins, when the window loses focus, and when the editor closes; on Windows a drag whose
  mouse capture was lost ends, and the capture is released.
- The editor reopens at its own size (not the drawer's); the grip asks the host for a size
  at most once a frame; names with characters XML forbids no longer blank the drawer;
  without a parent window or a display no window opens, and softbuffer's failures stop the
  drawing rather than the host. The approved-image test prints its margin (mean 1.245,
  99th percentile 26, against 1.5 and 32, the same on Apple silicon and x86-64).
- `docs/panel.png` is drawn again, with the strip under the panel.

### nih-plug (`third_party/nih-plug/PATCHES.md`, changes 4 to 8)
- A VST3 host's refusal to resize reaches the editor; auxiliary buses are bounded by the
  host's own counts (two read one past the end of the host's array); a side chain's missing
  channels are as long as the block; VST3's buffer configuration carries the processing mode
  just set; and `ProcessContext::process_mode()`.

### Building, testing, the repository
- `scripts/validate.sh` fetches the validators with curl (no GitHub CLI), checked against
  their SHA-256, and `CA72_SKIP_GUI_TESTS=1` leaves out pluginval's editor tests.
- `scripts/fetch-sources.sh` finds its manifest again and fails when a source cannot be
  fetched or does not match.
- The circuit tests ask for ngspice 47, the reference measurements' version, and skip (or,
  with `CA72_REQUIRE_NGSPICE`, fail) with another.
- Timing tests: the two measurements of what a machine plays are run by hand; the engine's
  deadline tests no longer depend on a worker being scheduled within milliseconds.
- CI builds the standalone. LICENSE is the GPL's canonical text. CONTRIBUTING.md and
  SECURITY.md. The repository's URL, the plug-in's (shown by hosts) and the installers'
  are `github.com/idlefoundry/ca-72`.

**Evidence (2026-10-03).**
- **The three reference machines,** from this release's code before it was published:
  the Mac (Apple M4 Pro, macOS 27), 201 tests passed, 0 failed (22 run by hand), clippy,
  rustfmt and the notices' check, the universal bundles and the `.pkg`, clap-validator 37
  passed (7 skipped), pluginval at strictness 10 SUCCESS (editor tests skipped while Live
  was open), Steinberg's validator 47 passed; the Linux reference machine (Ryzen 7 7800X3D),
  202 passed, 0 failed, the tarball, the same validators (editor tests skipped: no
  display); the Windows machine (i5-13600K, Windows 11, the GNU toolchain), 200 passed, 0
  failed, the setup program and the standalone.
- **CI** (macOS 15, Windows Server 2025, Ubuntu 22.04): every job green, with ngspice 47
  required on Linux and the three validators on each system.
- **The installers CI made, each installed on a machine without the CA-72:**
  - **Linux:** `install.sh` into `~/.vst3` and `~/.clap`; the plug-in needs glibc 2.35 at
    most (Ubuntu 22.04's); the installed copies pass clap-validator (37 passed), pluginval
    at strictness 10 (SUCCESS, editor tests skipped) and Steinberg's validator (47 passed);
    `--uninstall` leaves nothing. The owner then played it in Bitwig Studio, CLAP and VST3.
  - **Windows:** the downloaded setup program, unsigned, met SmartScreen's warning, as the
    README says; allowed, it installed (exit 0), "CA-72 0.1.0, Idle Foundry" in Settings'
    Apps; the installed copies pass clap-validator (36 passed, 8 skipped: one test runs on
    Unix only) and pluginval at strictness 10 with its editor tests (SUCCESS, in the
    desktop session); its uninstaller leaves nothing, the folders it made included.
  - **macOS:** Gatekeeper would not open the downloaded package until Open Anyway in
    Privacy & Security, as the README says; the owner installed it. In
    `/Library/Audio/Plug-Ins`: universal, 0.1.0, macOS 11, `com.idlefoundry.ca-72.vst3` and
    `.clap`, signatures valid, no quarantine flag, the licence and notices inside; they pass
    clap-validator (37 passed), pluginval at strictness 10 with its editor tests (SUCCESS)
    and Steinberg's validator (47 passed).

## R19. Two editor fixes after the installers were tried
**The owner's reports, 2026-10-03,** trying the installed plug-in: in Bitwig Studio on the
Linux reference machine, CLAP, the editor opened black until the pointer moved over it; on
macOS in Live, the presets' drawer, opened below the strip, did not shut the way it opened:
it jumped up over the panel and slid down off it, and when that was fixed, it flickered at
the moment it had shut.

**Agent decisions, 2026-10-03** (the owner confirmed the first three fixes in the hosts they were found in):
- **Linux: the frame shown again every quarter second.** The editor drew only when its frame
  changed (R5), the first frame once. X11 drops what is drawn to a window not yet on screen,
  and Bitwig maps the editor's window after that first frame; without a compositor it also
  drops what a covering window hid; baseview reports neither. On Linux the last frame is now
  shown again, unchanged, every 250 ms (a copy, no rendering). macOS and Windows keep a
  window's pixels and are unchanged.
- **The drawer shuts as it opened.** Shutting below the strip had shrunk the window at once,
  so the drawer's slide (still running) was drawn as the over-the-panel one (R10, R18). Now
  the window keeps its height while the drawer slides back up under the strip (140 ms), and
  shrinks when it has; the over-the-panel drawer is unchanged.
- **The window shrinks in one frame.** The host was asked for the shorter window while a
  frame was drawn, and the editor's view followed only in the next: on macOS, for that
  frame, the tall view sat in the short window with its top cut off. The view now follows in
  the same frame, before anything is shown.
- **The width is measured once an opening.** Until the grip sets a width, the editor's was
  fitted to the screen afresh at each of the host's questions, and the drawer asks again as
  it opens and shuts; on Linux the screen is the monitor under the pointer (R18), so the
  width, and the panel's scale, could change with the drawer. It is now fitted as the editor
  opens and kept until it closes.
- **A worker lets a voice go before marking it played.** R18's fencing of late workers had
  the mark stored while the voice's lock was still held: the caller's thread could see the
  mark, close the run, take the voice in the next and fail its `try_lock`, the voice then
  silent for that run. CI on macOS caught it as a FEEDBACK preset's samples differing between
  one thread and four (`tests/workers.rs`); the lock is now released first.

**Evidence (2026-10-03):** the plug-in's library tests (65 passed, among them
`the_drawer_shuts_the_way_it_opened`, `a_frame_is_shown_again_on_linux_only` and
`the_width_stays_while_the_drawer_opens_and_shuts`) and clippy on
macOS and Linux, clippy for Windows; the owner, in Bitwig on Linux (CLAP) and in Live on
macOS, with the builds of these changes.

## R20. Developer ID signing and notarization
**Owner's request, 2026-10-03:** set up Apple Developer ID for CA-72 so people installing
it on a Mac are recognized as receiving software from a trusted developer.

- Developer ID Application and Developer ID Installer certificates were issued for
  Idle Foundry Ltd., team `3JA8JUZ36W`, with the owner's approval. Both identities and
  private keys are installed in this Mac's login Keychain; encrypted backups and the
  local release configuration are outside the repository. Notarization credentials
  were entered by the owner and stored as the `CA72_NOTARY` Keychain profile.
- A universal 0.1.0 installer from main was signed, accepted by Apple's notary service,
  stapled and validated. Both payload bundles pass strict signature verification and
  contain x86_64 and arm64; their signatures include hardened runtime and a trusted
  timestamp. Gatekeeper accepts the installer as `Notarized Developer ID`, including a
  copy carrying the download quarantine attribute. It also opens normally in Installer.
- `CA72_REQUIRE_NOTARIZATION=1` makes public Mac packaging require both Developer ID
  identities and a notarization profile. Partial signing configuration is rejected.
  Apple must return `Accepted`; the ticket, installer signature and Gatekeeper policy
  are checked before the final distributable is replaced. A missing-profile failure
  was verified to preserve the previous installer.
- `docs/macos-release.md` records the reproducible local release procedure. Hosted CI
  signing secrets are not configured by this change; its unsigned macOS artifacts must
  not be published as the trusted installer. No signing private key or password is
  committed to the repository.

## R21. POLY's workers only while POLY is on; the copyright holder
**Owner decisions, 2026-10-03.** Of R18's note that an instance with POLY off still took its
share of the workers: it "should be fixed, absolutely". The copyright holder is Idle Foundry
Ltd., the company that holds the Developer ID (R20); the brand in hosts stays Idle Foundry.

**Agent decisions, 2026-10-03** (not separately approved):
- **An instance holds workers only while its POLY is on.** At activation it starts them only
  if POLY is on. When POLY is switched on, the audio thread asks for them and plays every
  voice itself until they come; nih-plug's background thread (as for the spare voices, R18)
  starts the pool and hands it over through a slot the audio thread only tries to lock.
  When POLY is switched off, or the plug-in is deactivated, the pool is given back and
  stopped on the background thread, its workers returned to the budget. Nothing is
  allocated, waited for or started on the audio thread; the sound is the same either way
  (R11). A state load at the same rate and period keeps running workers (R19).
- Still first come, first served among instances with POLY on: one that found none plays on
  the host's thread and asks again only when its POLY is next switched on (no polling).
- **The copyright line** "Copyright © 2026 Idle Foundry Ltd." is in the README, the notices,
  the macOS bundles' Info.plist and the Windows installer (whose publisher is now Idle
  Foundry Ltd.).

**Evidence (2026-10-03, the Mac):** the plug-in's tests (`ca72-plugin`: the library's 72, among
them `an_instance_with_poly_off_leaves_its_workers_to_one_with_it_on` and
`the_workers_follow_poly_through_the_plug_ins_life`; the real-time test with POLY switched,
nothing allocated on the audio thread), `ca72`'s `threaded` and `rt_alloc`, rustfmt and
clippy (and clippy for Windows).

## R22. URW Gothic for the lettering
**Owner decision, 2026-10-03:** shown the panel lettered in TeX Gyre Adventor, URW Gothic and
Nimbus Sans side by side, the owner chose URW Gothic.

**Agent decisions, 2026-10-03** (not separately approved):
- **Why change.** TeX Gyre Adventor's GUST Font License is the LaTeX Project Public
  License 1.3c, which the FSF counts as incompatible with the GNU GPL (some modified
  versions must carry, or point to, the unmodified original), and the plug-in builds its
  font into its binaries. URW Gothic, the font TeX Gyre Adventor was made from, is under the
  GNU AGPL 3.0 with a font exception; the GPL 3.0 (section 13) lets a work be combined with
  one under the AGPL 3.0, so the binaries are one work under compatible licences.
- **The same letters.** The drawn panel differs in 0.3 % of its pixels, at the letters'
  edges; the approved-image test gives a mean difference of 1.304 and a 99th percentile of
  28 (TeX Gyre Adventor's: 1.245 and 26; the limits 1.5 and 32). Book is the regular, Demi
  the bold.
- `third_party/urw-gothic/`: the two fonts unmodified, from
  `ArtifexSoftware/urw-base35-fonts` at tag `20200910`, with its licence, exception and
  readme, the hashes in `VENDORED.md`; the notices carry the licence and the exception.
  R2's "TeX Gyre Adventor" stands as the record of that time; `docs/panel.png` is drawn
  again.

## R23. The plug-in's own helper thread, so that a host may unload it at once
**Found 2026-10-03,** in the first CI run of the repository as published: on Windows,
clap-validator's `param-fuzz-modulation` test crashed (`0xc0000005`). On the Windows
reference machine it crashed in 167 of 200 runs with the code before R21 (workers started at
activation), and about once in 300 with R21.

**The cause (gdb).** The validator's main thread was unloading the plug-in's library while
nih-plug's shared background thread still ran a task of ours inside it (rebuilding the spare
voices, R18; starting or stopping POLY's workers, R21), the voice workers alive beside it.
nih-plug's background thread keeps the plug-in alive while it runs a task for it, so a host's
destroy during one returned at once and left the plug-in, its pools and their workers
running; Windows unmaps a library as soon as it is unloaded. macOS and Linux keep a library
mapped while threads of it live (on Linux, glibc's record of their thread-local
destructors), so only Windows crashed.

**Agent decisions, 2026-10-03** (not separately approved):
- **The plug-in gives nih-plug's background thread no work.** A helper thread of its own,
  `ca72-helper`, started at the first activation, rebuilds the spares and starts and stops
  POLY's workers; the audio thread asks it with an atomic and an unpark, allocating nothing
  and waiting for nothing. Dropping the plug-in stops and joins the helper, then the pools
  and their workers: when the host's destroy returns, no thread of the plug-in runs. A
  rebuild stops before its next voice when told to stop, so a destroy waits at most for one
  voice to be built.
- Linux's watchdog threads (R11, R18) still live for the process once started. glibc keeps
  the library loaded while they live, so a host's unload there is deferred, not a crash; left
  as it is.

**Evidence (2026-10-03):** on the Windows reference machine (the GNU toolchain), with the
helper: `param-fuzz-modulation` 0 crashes in 300 runs, and 0 in 100 runs of a build whose
helper was made to stall 1.5 s a round regardless of a stop (the worst case for a destroy;
those runs took about 1.5 s longer each, the destroy waiting); the full clap-validator, 36
passed, 0 failed. On the Linux reference machine: 0 failures in 100 runs, the full
clap-validator 37 passed, clippy, and the plug-in's tests on 16 processors and on 3 (as CI's).
On the Mac: the plug-in's tests (among them
`a_plug_in_dropped_while_its_helper_works_leaves_no_thread_running`), rustfmt and clippy, and
clippy for Windows. GitHub's CI could not run: the organisation's Actions minutes for private
repositories were spent.

## R24. 0.1.0's Windows and Linux installers built on the reference machines
**2026-10-03.** The tag's CI could not run (R23: the organisation's Actions minutes for
private repositories were spent), so the draft release had only the Mac's installer. The
owner chose to build the other two on the reference machines from the tagged commit rather
than make the repository public early for CI's free minutes.

**Agent decisions, 2026-10-03** (the route approved by the owner, the means not separately):
- **Windows** with the GNU toolchain (Rust 1.97.1, `x86_64-pc-windows-gnu`) and Inno Setup
  6.7.3: `cargo xtask bundle ca72-plugin --profile bundle`, then `scripts/package.sh`. The
  plug-in imports only Windows's own libraries (the universal C runtime among them, as on any
  Windows 10 or 11), none of MinGW's.
- **Linux** linked by Zig 0.17.0 through cargo-zigbuild 0.23.4 against glibc 2.35, the floor
  CI's Ubuntu 22.04 gives (the machine's own glibc, 2.43, would have raised it): the wrapper
  `cargo zigbuild --target x86_64-unknown-linux-gnu.2.35` makes is given to the bundler as
  `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER`, with `RUSTFLAGS="-L native=<dir>"` where
  `<dir>` holds links to the system's libX11, libX11-xcb and libxcb only, then
  `cargo xtask bundle ca72-plugin --profile bundle --target x86_64-unknown-linux-gnu` and
  `scripts/package.sh`. The plug-in's newest glibc symbol is `GLIBC_2.35`; it needs libX11,
  libX11-xcb, libxcb, libm, libc and nothing else.
- CI stays the way releases are built; this was for 0.1.0 only.
- **The release job's checksums** now take CI's line for a file over the draft's earlier one,
  so a re-run that replaces an installer replaces its checksum too (before, `sort -u` kept
  the earlier line).

**Evidence.** Each installer was validated and installed on its system before it went into
the release, built from the release's tag: on Windows clap-validator and pluginval at
strictness 10 (editor tests skipped: no desktop over SSH; Steinberg's validator needs Visual
Studio), a silent install (the plug-ins in Common Files, the Apps entry "CA-72 0.1.0, Idle
Foundry Ltd."), clap-validator on the installed CLAP and a silent uninstall leaving nothing;
on Linux clap-validator, pluginval at strictness 10 (editor tests skipped) and Steinberg's
validator, and `install.sh` into an empty home. Their checksums are the release's
`SHA256SUMS.txt`.

## R25. The review before going public
**2026-10-04.** A last review of the repository and the release before it was made public
(the owner: "if not, give it one last review"), by two agents, one on the documents and one
on licensing and provenance, checked against the code. Nothing blocked; these were fixed.

**Agent decisions, 2026-10-04** (not separately approved):
- **The Rust standard library's notice.** std, core, alloc and what the standard library
  bundles are linked into every Rust binary, under MIT or Apache-2.0; the notices now carry
  the Rust project's MIT notice, for the Rust that `rust-toolchain.toml` names.
- **The sources of the crates taken from git.** Cargo.lock takes seven sources from git
  repositories of others (vst3-sys, GPL-3.0, among them, from a branch), which could vanish;
  the rest are on crates.io, which keeps every version. `scripts/git-sources.sh` packs the
  seven at their pinned commits into `CA-72-<version>-git-sources.tar.gz`, which the release
  job puts into each release (0.1.0's was made on the Mac the same way); the notices say so
  and give each git crate's repository and commit.
- **Credits:** the fonts' own copyright line and the licence they are built in under (the
  AGPL 3.0, R22) in the notices; `circuits/models/ua741.lib` credits TI's PSpice uA741
  macromodel, whose layout it follows (its values are computed from the data sheet); the
  trademark notice names Moog as well.
- **Documents:** the README's install steps for Linux, uninstalling on each system, building
  universal bundles (the two targets to add), the voice POLY takes ("the oldest of those let
  go", as the code does, not the one released longest ago); a sentence about the DAW the
  model came from removed; CONTRIBUTING's commands as CI runs them, and the preset
  comparison's procedure; SECURITY's link; `docs/macos-release.md` written as a maintainer's
  procedure rather than one machine's notes; two crates' descriptions; the ngspice version
  Homebrew must have. R17's "some 190 crates" was too many: the notices list 159.
- **0.1.0 rebuilt.** The notices are inside every installer, so 0.1.0's three installers were
  built again from the corrected tree, and the tag v0.1.0, not yet public, moved to it.

## R26. Windows: the host no longer aborts as the presets' drawer opens
**The report, 2026-10-05.** A developer tried the VST3 on Windows 11 in Sonar and in a DAW of
their own: it loaded, but opening the presets' drawer ended the host. Their recording, in Sonar:
the window grows for the drawer, the new part stays white, and some 7 s later Sonar is gone,
with no dialog.

**The cause,** found on the Windows reference machine with 0.1.0's released VST3 (its
installer checked against `SHA256SUMS.txt`) in a test host written for it, which answers
`IPlugFrame::resizeView` in several ways. baseview borrows the editor's window handler (a
`RefCell`) for each event; a `WM_SIZE` that reached the window while the handler was still
handling the press borrowed it again: a panic inside the window procedure, which cannot
unwind, so the process aborted (exception 0xC0000409, fail fast; Windows Error Reporting
names `CA-72.vst3`). Two ways led there:
- **The drawer taking the keyboard.** The press asked the host to grow the window and baseview
  to follow (a deferred task); `take_keys` then called baseview's `focus()`, `SetFocus`, whose
  focus messages reached the window from within the handler, and baseview ran its deferred
  tasks for them: the resize, and so the `WM_SIZE`. Only where the editor's window did not
  already have the keyboard: REAPER 7.82 gives a plug-in's window the keyboard as it is
  clicked, and its drawer opened and shut normally (with real clicks, three launches).
- **A host resizing the editor's window from within `resizeView`.** Then the `WM_SIZE` came
  from the host, whatever had the keyboard. The grip, which asks from the editor's frame, did
  the same in such a host.

The test host aborted 3 times in 3 for each host behaviour that agrees to the resize (five
behaviours, with and without the processing active, posted and real clicks), and never where
the host refused it (the drawer then opens over the panel). Which way Sonar takes was not
seen (it needs an account the owner does not have); the symptoms and the second DAW fit
either. The same panic was fixed upstream for `focus()` alone
([baseview#252](https://github.com/RustAudio/baseview/pull/252), 2026-05-25), after the
revision the editor took.

**Agent decisions, 2026-10-05** (the owner approved making the fix and asked that both ways
work; not separately approved otherwise):
- **baseview vendored and patched** in `third_party/baseview`, at the commit the editor took
  from git, as nih-plug is (`third_party/baseview/PATCHES.md`): an event that arrives while
  the handler is busy waits and is given to it once its call returns, the deferred tasks run
  only then, and `focus()` is deferred (#252). Not upstream's master: its Windows backend and
  handler interface were rewritten, a port of the editor. Not #252 alone: it leaves the second
  way (the tests below abort with it alone). A review of the patch (a second agent) found that
  closing the window freed its state while baseview's window procedure still read it (as
  upstream did); each call of it now keeps the state until it returns.
- **Tests in a real window** (Windows only, `editor.rs`, `windows_window_tests`): the editor's
  window in a window of the test's, off the screen, in a host that grows its own window, or
  resizes the editor's as well from within the request: the drawer opens below the strip and
  shuts, and the grip resizes. They aborted the test process before the patch. The test's
  window has no taskbar button and is not brought to the front, and the tests count in pixels
  whatever the screen's scale (each thread per-monitor aware). The editor's window now takes its
  presets' library from its caller, so that these tests read an empty folder of theirs rather
  than the user's.
- **The git sources.** baseview at 9a0b42c is no longer taken from git, so
  `CA-72-<version>-git-sources.tar.gz` leaves it out (it is in this repository);
  nih-plug's standalone still takes baseview at 579130e from git. The notices name its
  repository.

**Evidence (2026-10-05, the Windows reference machine, the GNU toolchain as R24, a screen
at 200 %):** the bundle as it ships (`--profile bundle`) in the test host: the drawer opened
under nine conditions, and opened, took the keyboard, shut and reopened under seven host
behaviours; the grip resized under three (0.1.0 aborted under two of them); the editor
opened, closed and opened again; no abort, panic or error report in any. Before the review's
changes the patch had passed the same, the drawer runs 3 times in 3 each. In REAPER 7.82 (the
bundle loaded from its own folder) the drawer opened below the strip with real clicks, three
launches. `windows_window_tests` 3 passed; each aborted with baseview's Windows files as
upstream had them, and the two where the host resizes the editor's window aborted with #252's
change alone. `cargo test --workspace`: 217 passed, 0 failed (22 ignored, run by hand).
Clippy with `-D warnings` on the workspace, all targets, with
`--features ca72-plugin/standalone` (`assert_process_allocs` does not build for the GNU
target; CI's `--all-features` on MSVC not run here); `cargo fmt --all -- --check`;
`scripts/notices.py --check`. Not tried: Sonar, the reporter's DAW, the CLAP in a host (the
same editor), macOS and Linux (whose code is unchanged: `src/macos/window.rs` only as rustfmt
orders it).

## R27. An update check, when asked
**Owner decisions, 2026-10-06.** The owner asked whether the plug-in could look for new
releases on GitHub, or let its user look. Of three ways offered (a link to the releases'
page; a check made when the user clicks; a check made by itself, at most daily, that could
be turned off), the owner chose the second, with a button that downloads the installer for
the user's system. Installing it from within the plug-in was weighed and not made: the
plug-in would be replacing its own files while the host has them loaded (locked, on
Windows), both installers ask for an administrator anyway, and a plug-in that ran what it
downloaded would let whoever controlled a release run code on every user's machine (the
Windows installer is unsigned, and the release's checksums come from the same release).

**Agent decisions, 2026-10-06** (not separately approved):
- **Where:** the presets' drawer's tools, before ×: the version (`CA-72 0.1.0`) and CHECK FOR
  UPDATES. The strip's row is full (R12), and the panel is the instrument's.
- **Through the system's curl,** not a network library linked in: Windows' own
  `System32\curl.exe` (Windows 10 1803 on), macOS's `/usr/bin/curl`, elsewhere the one on the
  path. It asks `api.github.com/repos/idlefoundry/ca-72/releases/latest`, which names only
  published releases (no drafts or pre-releases): HTTPS only (redirects too), 20 s and 1 MB at
  most, as `CA-72/<version>`, nothing else sent; on Windows without a console window.
  serde_json, which nih-plug already links, reads the answer: the notices are unchanged.
- **No thread waits for it** (R23): curl writes into a file the editor makes afresh in the
  temporary folder (never one already there, nor through a link), and the editor looks each
  frame whether curl has finished. Closing the editor ends curl and removes the file; past
  30 s the check fails. A curl gone with its exit unknown (a host that reaps its children
  itself, ignoring SIGCHLD) has its answer read all the same and judged by itself; curl
  writes none on an HTTP error.
- **What it shows:** CHECKING…; `<version> IS UP TO DATE` (this release or a later one) and
  CHECK FOR UPDATES again; `<new> IS OUT (THIS IS <this>)` in the accent, and DOWNLOAD; after
  DOWNLOAD, CLOSE THE DAW, THEN INSTALL IT; COULD NOT CHECK (no curl, no answer, an HTTP
  error, an answer without a version) and RELEASES PAGE; NO BROWSER WOULD OPEN. A version is
  three numbers, compared in order.
- **DOWNLOAD opens the installer in the browser,** which downloads it: the release's
  `CA-72-<version>-Windows-setup.exe`, `-macOS.pkg` or `-Linux-x86_64.tar.gz`, as the README
  lists them; the release's page where it has none for the system, or the system has none
  (Linux on ARM). Windows' shell (`ShellExecuteW`) opens it, macOS's `open`, Linux's
  `xdg-open`, and only an address under `https://github.com/idlefoundry/ca-72/releases`,
  whatever the answer says. An `open` or `xdg-open` that exits in failure (with no display,
  `xdg-open` exits 3 and opens nothing) turns the message to NO BROWSER WOULD OPEN.
- **SECURITY.md** no longer says the plug-in opens no network connections: it says when it
  connects and what it sends. The README says how to update, and the check's limits (curl,
  no Windows proxy settings).
- **Tests:** `update::tests` (versions; each system's installer found; addresses elsewhere
  refused; answers without a version; every scene fits the drawer whole; a newer release's
  DOWNLOAD and what follows, and this release up to date and checked again, each through a
  stand-in for curl printing a canned answer; curl missing or failing; an opener failing
  once started; the editor closed mid-check ending curl at once and leaving no file; and,
  run by hand, GitHub asked, the releases' page opened in the browser, and the latest
  release's installer for the system downloaded by it), `ca72-panel`'s `presets::tests`
  (the button found where drawn, and not where there is none), and the editor's
  `the_drawer_checks_for_updates`.

**Evidence (2026-10-06):** on Windows 11 (the GNU toolchain as R24), macOS (the Mac, Apple
silicon) and Linux (an x86-64 desktop running Hyprland), the tests of `ca72-plugin` and
`ca72-panel` all passed, R26's real-window tests among them on Windows, and clippy with
`-D warnings` passed on both crates and all targets (on Windows the whole workspace with
`--features ca72-plugin/standalone`); `cargo fmt --all -- --check` and
`scripts/notices.py --check` on Windows. On each system, by hand, the system's own curl
found 0.1.0 up to date, and the browser (Chrome on Windows and macOS, Chromium on Linux)
opened the releases' page and downloaded 0.1.0's installer for the system, its SHA-256 the
release's `SHA256SUMS.txt` gives (the `.pkg` signed and notarised). A build calling itself
0.0.9, so that 0.1.0 was newer:
- **REAPER 7.79 on Linux** (a portable copy scanning only the build's folder; the clicks
  sent to the editor's X11 window): the drawer opened below the strip, the window growing;
  CHECK FOR UPDATES, then CHECKING…, then `0.1.0 IS OUT (THIS IS 0.0.9)` and DOWNLOAD;
  DOWNLOAD had Chromium download the Linux installer (its SHA-256 the release's) and said
  CLOSE THE DAW, THEN INSTALL IT; no process of the check's was left, nor its file.
- **The standalone on Windows** (started without a console, as a host is; the clicks posted
  to the editor's window; a screen at 200 %): the same, `System32\curl.exe` run by it with
  the arguments above, no console window appearing, its file removed; DOWNLOAD had Chrome
  download the Windows installer (15,419,949 bytes, the release's); closed, it exited.

Later the same day, the build calling itself 0.0.9 renamed `CA-72 TEST` (its own CLAP and VST3
IDs, so that a host never takes it for the installed CA-72), in portable copies of REAPER 7.82
that kept their settings to themselves:
- **On macOS** (the VST3; the clicks made with `cliclick`): the same, from CHECK FOR UPDATES to
  `CLOSE THE DAW, THEN INSTALL IT`; Chrome downloaded the `.pkg` (its SHA-256 the release's).
- **On Windows** (the CLAP, then the VST3; the clicks posted to the editor's window): the same;
  `System32\curl.exe` run by REAPER, no console window, the file removed; Chrome downloaded the
  Windows installer (the release's size).

Seen meanwhile, and not this record's: on Windows the editor does not draw again when Windows
repaints its window, so whatever invalidates it (a capture made with `PrintWindow` here) leaves
it blank until something in it changes; 0.1.0 did the same. Left for a change of its own (R28).

Not tried: Live or Bitwig, an HTTP proxy, a release whose installer is missing.

## R28. Windows: the editor shown again when Windows repaints its window
**Seen, 2026-10-06,** while the update check was tried (R27) in REAPER 7.82 on Windows 11 (a
screen at 200 %): with 0.1.0's released VST3 in a floating FX window, the editor's area on the
screen had 872 distinct colours (64 by 64 samples); right after `PrintWindow(FX window,
PW_RENDERFULLCONTENT)` it had 2, and still 2 three seconds later, the panel gone for RGB 240,
240, 240. A test build of the update check's branch did the same.

**The cause.** The editor shows a frame only when it has changed (R5), and on Linux again
every 250 ms (R19, which took macOS and Windows to keep a window's pixels). Windows does not
always: whatever invalidates a window (a host repainting its own, a capture with
`PrintWindow`, some remote-desktop and capture tools) has Windows ask for it to be painted
(`WM_PAINT`), and a parent window without `WS_CLIPCHILDREN`, as a dialog may be, paints its
background over its children first. baseview left `WM_PAINT` to `DefWindowProcW`, which
validates the window without drawing, and told the editor nothing; the editor's window
class has no background of its own, so what showed was the parent's. 240, 240, 240 is a
dialog's (`COLOR_BTNFACE`). In a test host whose window is such a dialog, the editor's
window was left that one colour when the host's window was repainted with its children, and
when the host's window was captured with `PrintWindow`, 1.1 s later still; repainting the
editor's window alone changed nothing (no background to paint). In REAPER the editor's
window is in REAPER's `reaperPluginHostWrapProc`, in a dialog in the FX window's dialog
(`#32770`): none of the three has `WS_CLIPCHILDREN`.

**Agent decisions, 2026-10-06** (the owner asked for the fix and its test; not separately
approved otherwise):
- **baseview tells the handler,** a second patch in `third_party/baseview` (PATCHES.md):
  `WM_PAINT` gives the handler a new event, `WindowEvent::Damaged`, as R26 gives any event
  (queued while the handler is busy, given once its call returns, never a second borrow from
  within the window procedure), and `DefWindowProcW` then validates what the handler has not
  drawn, or Windows would ask again ahead of every frame's timer. Upstream's rewritten master
  does the same since [baseview#344](https://github.com/RustAudio/baseview/pull/344)
  (2026-10-04): it tells its handler of the damage and draws within `WM_PAINT`; its
  interface is not this revision's (R26).
- **The editor shows its last frame again at once** on that event: a copy, nothing rendered
  (as R19's on Linux), only when Windows asks; no frame more otherwise. Not before a first
  frame since the window opened or resized (that frame is due anyway). softbuffer validates
  the window as it presents, so Windows does not ask again.
- **Not at the next frame instead** (the frame marked unshown, for `on_frame` to show): the
  parent's background would show for up to a frame's 15 ms at each repaint.
- **Linux and macOS unchanged.** baseview's X11 backend ignores `Expose`, so R19's repaint
  stays; macOS keeps a window's pixels.
- **The test,** in `windows_window_tests`: `the_panel_is_shown_again_as_windows_repaints_its_window`.
  R26's host window could not show this: nothing is kept of what is drawn off the screen (a
  window's pixels there are clipped away, so there is nothing to read), and it clips its
  children from its painting. This test's host window is a dialog's (its class's background
  `COLOR_BTNFACE`, no `WS_CLIPCHILDREN`), on the screen at its top left, but at an alpha of 1
  in 255, at the bottom of the windows, letting the pointer through to whatever is under it
  (`WS_EX_LAYERED`, `WS_EX_TRANSPARENT`), with no taskbar button and never activated, as
  R26's. The editor's pixels are read with `GetPixel`, 64 by 64 (the window's own, whatever
  covers it). The test checks that the host's window repainted alone covers the editor's
  window here (one colour), that the panel's pixels are back, the same, within 100 ms of the
  host's window repainted with its children (`RedrawWindow`, `RDW_INVALIDATE | RDW_ERASE |
  RDW_UPDATENOW | RDW_ALLCHILDREN`) and of `PrintWindow(host's window,
  PW_RENDERFULLCONTENT)`; and, with the drawer opened below the strip in a host that repaints
  its windows from within the editor's request (a `WM_PAINT` reaching the editor while its
  handler is busy), that the panel and the drawer are back after a repaint.

**Evidence (2026-10-06, the Windows reference machine, the GNU toolchain as R24, a screen at
200 %):** with the `WM_PAINT` event never given (baseview as before this record), the test
failed 3 times in 3, the editor's window one colour after the host's was repainted, and
`PrintWindow` on the host's window alone (a check of its own, run by hand) left 0xF0F0F0
there, still 1.1 s later. With the patch, the panel's 632 colours (of 64 by 64 samples) came
back each time, `PrintWindow` alone left them as they were (3 runs in 3), and a probe saw a
`WM_PAINT` reach the editor while its handler was busy, as the drawer opened, with no abort.
`windows_window_tests` 4 passed in each of 9 runs; the plug-in's tests
(`cargo test -p ca72-plugin`) 102 passed, 0 failed (8 ignored, run by hand). Clippy with
`-D warnings` on the workspace, all targets, with `--features ca72-plugin/standalone`;
`cargo fmt --all -- --check` (on a copy of the tree outside the main checkout, which this one
was nested in); `scripts/notices.py --check`.

In REAPER 7.82, a copy of it with a resource folder of its own each run (`-cfgfile`), its
first-run questions answered No (importing the license key it found on the clipboard,
choosing an audio device), and a script putting the VST3 on a track, its FX window floating;
then, as in the report, the editor sampled 64 by 64 (on the screen and from its own window),
`PrintWindow(FX window, PW_RENDERFULLCONTENT)`, the editor sampled at once and 3 s later, then
the same for `RedrawWindow(FX window, RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW |
RDW_ALLCHILDREN)`:
- **0.1.0** (the installed VST3, as REAPER loaded it): 331 colours before; one, 240, 240, 240,
  at once after the capture and 3 s later, and after the repaint (2 runs).
- **This change,** built as the bundle is (`--profile bundle`) under a name and IDs of its own
  (`CA-72 R28`, as R27's `CA-72 TEST`), the file it loaded checked: REAPER scans the standard
  VST3 folders whatever its settings say, and took the installed 0.1.0, of the same name and
  IDs, over a build of this change named as it is. 331 colours before and after both, 3 runs
  in 3; in two, the screen where the editor is was the same image, byte for byte, before,
  after the capture and after the repaint (in the third, REAPER's About window had come over
  part of it on the screen; the editor's own pixels kept their 331 colours). `cargo xtask
  bundle`, run in a worktree inside another checkout, builds the outer one (nih-plug's xtask
  goes to the outermost folder with a `Cargo.lock`), so the build was `cargo build --profile
  bundle` here.

Not tried: another host, a remote-desktop session, CI's Windows runner (the new test needs
the editor drawn on a desktop), macOS and Linux (whose code is unchanged).

## R29. 0.1.1
**Owner decision, 2026-10-06.** A 0.1.1 release once R28 is merged ("once #4 merges,
prepare a 0.1.1 release").

**What it brings since 0.1.0** (the tag `v0.1.0`):
- **Windows: the host no longer aborts** as the presets' drawer opens, nor as the grip
  resizes in a host that resizes the editor's window from within the editor's request (R26;
  Sonar, and a developer's own DAW).
- **Windows: the editor is shown again when Windows repaints its window** (R28; it stayed
  the host's grey after a capture or a host's repaint, in REAPER among others).
- **CHECK FOR UPDATES** in the presets' drawer (R27). 0.1.0 has none, so its users learn of
  0.1.1 from the releases page.
- The README's video (documents only).

**Agent decisions, 2026-10-06** (not separately approved):
- **A patch release, 0.1.1.** Nothing a project or a preset holds has changed since 0.1.0:
  the parameters, the plug-in's IDs, the voice (`crates/ca72`, the plug-in's engine) and the
  presets' format are 0.1.0's, so what was saved with 0.1.0 opens unchanged. Its installers
  install over 0.1.0's, as they are made to (the Windows installer's AppId is 0.1.0's, the
  macOS packages are not version-checked, `install.sh` replaces the bundles).
- **The version** is the workspace's (`Cargo.toml`): every crate's, so the installers'
  names, the macOS packages' versions, the Windows installer's Apps entry and the update
  check's own version (`CARGO_PKG_VERSION`) follow; `Cargo.lock` changed only in the
  workspace's eight crates. The README's status names 0.1.1.
- **The update check's test of versions** (`a_version_is_three_numbers`, R27) took 0.1.1 to
  be newer than this build, true only of 0.1.0, and failed with the bump: it now takes the
  next patch, minor and major of this build's version to be newer, and neither this one nor
  0.1.0, so that a release's version leaves it passing.
- **Released the way R17 and R20 set out** (R24 was 0.1.0's exception): once this is
  merged, the tag `v0.1.1` on main's commit; CI's release job drafts the release with the
  Windows and Linux installers, the notices, the git sources and `SHA256SUMS.txt`; the macOS
  installer is built, signed and notarised on the Mac (`docs/macos-release.md`) and put into
  the draft; the owner publishes it, with notes in 0.1.0's form (drafted in this change's
  pull request).

**Evidence (2026-10-06, the Windows reference machine, the GNU toolchain as R24),** on this
change's tree (R28's, main merged into it, and the bump): `cargo test --workspace` 229
passed, 0 failed (25 ignored, run by hand); clippy with `-D warnings` on the workspace, all
targets, with `--features ca72-plugin/standalone`; `cargo fmt --all -- --check`;
`scripts/notices.py --check`. `cargo xtask bundle ca72-plugin --profile bundle` and
`scripts/package.sh` made `CA-72-0.1.1-Windows-setup.exe`, its version information 0.1.1, the
plug-in in it built from this tree (not installed: the machine's 0.1.0 is left as it is). The
macOS and Linux installers are CI's, on the pull request.

**The release (2026-10-06, the release Mac: an M4 Pro, macOS 27.0, Xcode 27.0, notarytool
1.1.3),** from a fresh clone of the tag `v0.1.1` (`425af615`, main's commit; clean, version
0.1.1). The tag's CI passed and drafted the release with its five assets. `cargo xtask
bundle-universal ca72-plugin --profile bundle` (Rust 1.97.1) made the bundles, then
`scripts/package.sh` with the two Developer ID identities, the notary profile and
`CA72_REQUIRE_NOTARIZATION=1` made `CA-72-0.1.1-macOS.pkg`:
- **Apple's notary service** accepted it: submission `d2746af0-c27b-4fee-9a99-5705a755bd6a`,
  status `Accepted`. The ticket is stapled (`stapler validate` passes); the installer is
  signed `Developer ID Installer: Idle Foundry Ltd. (3JA8JUZ36W)`, trusted timestamp
  2026-10-06 16:18:47 UTC; Gatekeeper (`spctl --assess --type install`): `accepted`,
  `source=Notarized Developer ID`.
- **The VST3 and the CLAP in its payload** (expanded with `pkgutil --expand-full`): each
  signed `Developer ID Application: Idle Foundry Ltd. (3JA8JUZ36W)` with the hardened
  runtime and a secure timestamp, `codesign --verify --strict` passing; version 0.1.1
  (nih-plug's bundler writes 1.0.0, `package.sh` the workspace's); `x86_64 arm64`; LICENSE
  and the notices inside; installed to `/Library/Audio/Plug-Ins/VST3` and `…/CLAP`.
- **Its SHA-256** is `f0077b6b74ab10d0c66271b921de7390f5b6f282c963c3a388ada181b38a5624`
  (9,189,503 bytes). It went into the draft with its line added to CI's `SHA256SUMS.txt`;
  the four files it lists, downloaded again from the draft, passed `shasum -a 256 -c`.
- **Published** on the owner's word at 2026-10-06 16:22:13 UTC as the latest release
  (https://github.com/idlefoundry/ca-72/releases/tag/v0.1.1, six assets). The API's
  `releases/latest`, which CHECK FOR UPDATES reads, gives `v0.1.1`; the `.pkg` downloaded
  from the public release matches its checksum and Gatekeeper accepts it.

`notarization.json` is kept with the release's evidence outside the repository.

## R30. An Audio Unit for macOS
**Owner decision, 2026-10-06.** An Audio Unit, as asked for in issue #6 ("an AU-Plugin
version"): "start the AU work on a branch, you can use GarageBand for testing on this
computer. When we've determined that everything works, please merge into main and clean up
the branch." Made alongside another machine's work on Potato's speed: the Audio Unit wraps
the CLAP plug-in as it is, so the two meet only in the plug-in's parameters, its state and
its latency, which the wrapper passes on unchanged, and in this file.

**Agent decisions, 2026-10-06** (not separately approved):
- **Version 2, made from the CLAP plug-in by clap-wrapper.** nih-plug makes no Audio Units.
  [clap-wrapper](https://github.com/free-audio/clap-wrapper)'s AUv2 wrapper (MIT; v0.16.0,
  its latest) presents the CLAP plug-in to the host as an Audio Unit: the plug-in's own code
  is unchanged, and the Audio Unit plays what the CLAP plays. Version 2 rather than 3: Logic
  Pro and GarageBand, and every macOS host that takes Audio Units, load version 2 in their
  own process; version 3 is an app extension inside an app of its own, and what iPad hosts
  need, which the CA-72 is not built for.
- **The CLAP inside the component** (`Contents/PlugIns/CA-72.clap`). The wrapper looks
  there before the CLAP folders, so the component plays the CLAP it was built with, whether
  or not a CLAP is installed, and never another version that happens to be. The installer's
  component carries the same signed CLAP as its CLAP choice (`scripts/package.sh` signs the
  CLAP, then copies it in, then signs the component). It costs the component the CLAP's
  10 MB.
- **Its identity, which a host keeps in its songs and so must never change:** type `aumu`
  (an instrument), subtype `CA72`, manufacturer `IdlF` (Idle Foundry; Apple keeps the
  all-lowercase codes for itself); the name "Idle Foundry: CA-72" (Logic and GarageBand list
  it as CA-72 under Idle Foundry); the bundle identifier `com.idlefoundry.ca-72.component`.
- **Vendored:** clap-wrapper, the CLAP headers (1.2.6) and Apple's AudioUnitSDK (1.1.0, the
  Apache License 2.0), at the versions clap-wrapper fetches for itself, in `third_party/`
  with their sources' commits and hashes (`PATCHES.md`, `VENDORED.md`). The build needs no
  network, and the source of everything built into the component is in the repository, as
  the GPL asks of the whole. `THIRD-PARTY-NOTICES.txt` carries their licences, and {fmt}'s,
  which clap-wrapper compiles in (`scripts/notices.py`).
- **clap-wrapper patched** (`third_party/clap-wrapper/PATCHES.md`): pluginval aborted the
  host in about one run in four with the heap corrupted, because the host's parameter
  changes went straight into the audio thread's event list from whatever thread the host
  called on; and a state saved before the host rendered a block lost the changes since the
  last one, and the host read the values from before a state it had just restored. Four
  changes: the host's changes wait under a lock for the audio thread; a save hands any
  still waiting to the plug-in first (`clap_plugin_params.flush()`, with no block
  processed meanwhile); a restore refreshes the values the host reads at once; and the
  host lists the parameters in the plug-in's order, the panel's, rather than by id.
  Nothing upstream had reported or fixed them; offering the fixes upstream is the owner's
  call.
- **Built by `scripts/auv2.sh`** (CMake 3.21 or later; `scripts/auv2/CMakeLists.txt`), from
  `target/bundled/CA-72.clap`, after the bundles, for the CLAP's architectures (universal
  for a release) and for macOS 11 as they are. Afresh each time, a few seconds: the wrapper
  reads the CLAP while it builds, which CMake does not track.
- **In the macOS installer** as a third choice, Audio Unit, into
  `/Library/Audio/Plug-Ins/Components`, like the others not relocatable and not version
  checked.
- **Validated** by `scripts/validate.sh` on macOS: Apple's `auval -strict` (and the Intel
  slice under Rosetta, where it is installed) and pluginval at strictness 10. Hosts and these
  validators find an Audio Unit only where it is installed, so the script first copies it
  into `~/Library/Audio/Plug-Ins/Components`, over any copy there. CI builds it on macOS,
  validates it and keeps it with the bundles.
- **Not released here.** The README says that the Audio Unit comes with the macOS installer
  from the release after 0.1.1; until then it is built from source.

**Evidence (2026-10-06, the release Mac: an M4 Pro, macOS 27.0, Xcode 27.0, CMake 4.4.4,
Rust 1.97.1),** on this change's tree. `scripts/auv2.sh` on the universal bundles made
`CA-72.component`, `x86_64 arm64`.
- **`auval -strict`:** AU VALIDATION SUCCEEDED: 51 parameters (in the panel's order with
  change 4), the Cocoa view, renders from 11.025 to 192 kHz and 64 to 4096 frames, MIDI.
  One warning: the preset's name is not kept in the class data. The Intel slice was not
  run: this Mac has no Rosetta.
- **pluginval 1.0.4 at strictness 10** (editor tests skipped, the screen in use). With
  clap-wrapper as released it aborted the host in 2 runs of 10 ("BUG IN CLIENT OF
  LIBMALLOC: memory corruption of free block": the message thread in `SetParameter()` while
  another thread initialized the unit), where the VST3 passed 5 of 5. With change 1, no abort
  in 12 runs, and with changes 1 and 2 none in 20, but the state restoration test failed in
  1 and 3 of them; with all four, 24 of 24 passed, and the seeds that had failed pass.
- **A host of AVAudioEngine's,** the way GarageBand and Logic load units: five notes sound
  (RMS 0.13), the output bounded; the full state, saved after a block, restored into a
  second instance; parameters set and saved with no block between are kept (clap-wrapper as
  released loses them). The editor through `kAudioUnitProperty_CocoaUI`, opened and closed
  four times and the unit disposed of with it open; and through Apple's out-of-process
  hosting (`AUHostingService`), drawn whole at 1382 by 463.
- **GarageBand 10.4.14,** which on this macOS loads the component in Apple's hosting
  service: listed under AU Instruments › Idle Foundry › CA-72 (Stereo); its Smart Controls
  took the first parameters in the panel's order; the editor shown whole in its plug-in
  window. Four notes recorded with Musical Typing and exported as WAVE: 0.15 RMS, the C at
  260 Hz (C4, 261.6 Hz), released to silence. POLY on and VOICES 5, set on the panel; the
  project saved, closed and reopened with both as set; played back with POLY on, no crash.
- **`scripts/package.sh`** (signed ad hoc here) made a three-choice installer, every package
  not relocatable, the component's CLAP identical to the CLAP choice's;
  `scripts/notices.py --check` passes. CI's runs are on the pull request.

## R31. Hosts that switch the side chain off: the plug-in no longer reads address 0
**The report, 2026-10-06.** Users told the owner that the CA-72 did not work in Sandyne, a
free DAW, as a VST on Windows. The owner asked for it to be tried there.

**The cause,** found on the Windows reference machine with Sandyne 2.5.0.0 (its installer,
`Sandyne_v2.5.0.0_win64.exe`, 538,577,168 bytes, from the link on sandyne.com, installed by
the owner; it takes VST3 and CLAP) and 0.1.1's released VST3 (its installer checked against
`SHA256SUMS.txt`; the module Sandyne loaded was that file, 4,636,160 bytes, version 0.1.1).
Sandyne scanned the plug-in without trouble. Put on a track, it was set up for blocks of up
to 1920 samples at 48 kHz, its editor opened, and in its first block, of 480 samples, Sandyne
caught an access violation inside the plug-in's processing and disabled it for the session
("Plugin Crashed: CA-72"): no sound. Sandyne is built with JUCE, and sets an instrument up
with outputs only (`setPlayConfigDetails (maxBs=1920 outCh=2)` in its log), which switches
off the CA-72's one input bus, its side chain (EXTERNAL INPUT). For a bus switched off on its
side, JUCE's VST3 hosting gives the bus's full channel count, two, with every channel's
pointer null (`HostBufferMapper::associateBufferTo`, `juce_VST3Common.h`). nih-plug checked
only that the array of pointers was not null, then copied each channel through its pointer:
a read from address 0. 0.1.0 does the same; the code is unchanged since. REAPER, where the
plug-in has been tried most, gives the bus its buffers. Hosts built with JUCE that set an
instrument up without inputs likely all meet it. That Sandyne switches the bus off is
inferred from its log and JUCE's source (Sandyne's is not public); the fix below, which
handles nothing else, ended the crash.

**Agent decisions, 2026-10-06** (the owner asked for the fix and for it to be tried in
Sandyne again; not separately approved otherwise):
- **nih-plug's ninth change** (`third_party/nih-plug/PATCHES.md`): a channel the host gives as
  a null pointer is read as silence if it is an input, and backed by scratch storage,
  discarded, if it is an output, so that every channel the plug-in sees is as long as the
  block. In the buffers both of nih-plug's wrappers share, so the CLAP takes it too, and with
  it the Audio Unit, which wraps the CLAP (R30). Not by the bus's activation
  (`IComponent::activateBus`), which nih-plug accepts and ignores: the pointers are what is
  read, and a host may give a null one for a bus it left active.
- **Tests.** `a_vst3_host_that_deactivates_the_side_chain_has_its_blocks_played`
  (`crates/ca72-plugin/src/lib.rs`) drives the plug-in's VST3 wrapper as Sandyne did: set up
  for 1920 samples, the side chain switched off, three blocks of 480 with its two channels
  null; every output sample written and finite. nih-plug's own `null_channel_pointers`
  covers each kind of channel, main and auxiliary, input and output; nih-plug's tests are
  not the workspace's, so it is run by hand, from a copy of `third_party/nih-plug` outside
  the repository (which keeps a `Cargo.lock` of its own out of it).
- **Tried in Sandyne under a name and IDs of its own,** as R28 was: `CA-72 R30` (so named
  before the Audio Unit took R30), built as the bundle is (`--profile bundle`, with MSVC as CI
  builds it: R32), in the user's VST3 and CLAP folders (`%LOCALAPPDATA%\Programs\Common`).
  R27's `CA-72 TEST` builds, still there, went to the Recycle Bin, so that Sandyne listed no
  stale build.

**Evidence (2026-10-06, the Windows reference machine: i5-13600K, Windows 11, a screen at
200 %; Rust 1.97.1, GNU as R24 and MSVC as R32):**
- **Sandyne 2.5.0.0** (the owner's clicks, its log and the modules it loaded read here):
  0.1.1's VST3 crashed in its first block as above. This change's: scanned and put on a
  track, set up and set up again after its editor opened, as before; the editor at 3072 by
  1030, then 1536 by 515; the owner played notes on its track (four in Sandyne's log) and
  heard it play; no crash, no error in the log. Not tried: the CLAP in Sandyne, the drawer
  and the grip there, a project saved and opened again, other hosts built with JUCE.
- **The new test** aborted before the change, at the side chain's copy (a debug build checks
  the null pointer: `slice::from_raw_parts_mut requires the pointer to be aligned and
  non-null`; a release build reads address 0), and passes after it. nih-plug's buffer tests,
  `buffer_io` and `null_channel_pointers`, pass.
- **On this change's tree** (main with the Audio Unit, R30): `cargo test --workspace` 230
  passed, 0 failed (25 ignored, run by hand) with GNU, and the same with MSVC; clippy with
  `-D warnings` on the workspace and all targets, with `--features ca72-plugin/standalone`
  (GNU) and with `--all-features` (MSVC, as CI); `cargo fmt --all -- --check`;
  `scripts/notices.py --check`. macOS and Linux run the same code (CI's tests on the pull
  request); the Audio Unit was not tried.

## R32. Windows: the C runtime linked into the plug-in
**Seen, 2026-10-06,** while R31 was looked into: 0.1.1's Windows plug-in, built by CI with
MSVC, imports Visual C++'s runtime, `VCRUNTIME140.dll`, and four of the Universal C
Runtime's `api-ms-win-crt-*` libraries. The installer does not bring Visual C++'s runtime,
so where no other program has installed it and the host has no copy of its own, Windows
cannot load the plug-in, and the host shows it failing or not at all. Not seen on a
computer: read from the imports. Not R31's cause: Sandyne has its own copy, beside its
executable. The reference machine's builds, with GNU, never needed it.

**Agent decisions, 2026-10-06** (the owner asked for what was found on the way to be fixed,
and approved Visual Studio's Build Tools on the reference machine; not separately approved
otherwise):
- **The C runtime linked statically with MSVC** (`.cargo/config.toml`: `+crt-static` for
  `cfg(all(windows, target_env = "msvc"))`, which CI's Windows build is): the plug-in
  imports Windows' own libraries only. Not Visual C++'s redistributable in the installer
  (some 25 MB beside its 3.6, and a second program installed), nor the runtime's DLL beside
  the plug-in, where Windows does not look (it looks beside the host's executable). A
  `RUSTFLAGS` in the environment would replace the configuration's; CI sets none.
- **MSVC on the reference machine,** beside the GNU toolchain, which stays its default:
  Visual Studio Build Tools 2022 (17.14, the C++ workload) and Rust 1.97.1 for
  `x86_64-pc-windows-msvc` (`cargo +1.97.1-x86_64-pc-windows-msvc`). The bundles as CI
  builds them, and its clippy with `--all-features` (R26 could not run it), run here now.

**Evidence (2026-10-06, the Windows reference machine):** `cargo +1.97.1-x86_64-pc-windows-msvc
xtask bundle ca72-plugin --profile bundle`: the VST3 and the CLAP import
`api-ms-win-core-synch-l1-2-0`, `avrt`, `bcryptprimitives`, `gdi32`, `kernel32`, `ntdll`,
`ole32`, `oleaut32`, `shell32` and `user32`, all Windows'; 0.1.1's VST3 imports these and
`VCRUNTIME140` and `api-ms-win-crt-heap`, `-math`, `-runtime` and `-string`. The exports are
0.1.1's (`GetPluginFactory`, `InitDll`, `ExitDll`, `clap_entry`); the VST3 is 4,802,048
bytes, 0.1.1's 4,636,160. R31's test build, linked the same way, played in Sandyne; the
workspace's tests pass with MSVC (R31). Not tried: a computer without Visual C++'s
redistributable (this one has had it since 2026-09-18), so the failure itself was not seen.

## R33. 0.1.2
**Owner decision, 2026-10-06.** A release as soon as may be, with R31 and the Audio Unit
(R30): "I would like to get this out as a release as soon as possible along with the AU pr,
as I think people might be running into issues."

**What it brings since 0.1.1** (the tag `v0.1.1`):
- **The plug-in plays in hosts that switch its side chain off** (R31): in Sandyne, and
  likely in other hosts built with JUCE, 0.1.0 and 0.1.1 crashed in their first block and
  the host disabled them. On every system, the VST3 and the CLAP.
- **An Audio Unit for macOS** (R30), a third choice in the macOS installer.
- **Windows: no Visual C++ runtime needed** (R32).

**Agent decisions, 2026-10-06** (not separately approved):
- **A patch release, 0.1.2,** as R29: nothing a project or a preset holds has changed since
  0.1.1 (the parameters, the plug-in's IDs, the voice and the presets' format are its), so
  what was saved with 0.1.0 or 0.1.1 opens unchanged, and the installers install over
  theirs. The Audio Unit is new; its identity is R30's.
- **In R31's pull request,** a commit of its own, so that CI runs once before the tag (R29
  was a pull request of its own).
- **The version** as R29: the workspace's (`Cargo.lock` changed only in its eight crates);
  the README's status names 0.1.2, and its line on the Audio Unit names 0.1.2 rather than
  "the release after 0.1.1".
- **Released the way R29 was:** once merged, the tag `v0.1.2` on main's commit; CI's release
  job drafts the release with the Windows and Linux installers, the notices, the git sources
  and `SHA256SUMS.txt`; the macOS installer, now with the Audio Unit, is built, signed and
  notarised on the release Mac (`docs/macos-release.md`) and put into the draft; the owner
  publishes it, with notes in 0.1.1's form (drafted in the pull request).

**Evidence (2026-10-06, the Windows reference machine, GNU as R24),** on this change's tree
(R31 and R32, and the bump): `cargo test --workspace` 230 passed, 0 failed (25 ignored, run
by hand); `Cargo.lock` changed only in the workspace's eight crates. The installers are
CI's, on the pull request.

**The release (2026-10-06, the release Mac: an M4 Pro, macOS 27.0, Xcode 27.0, notarytool
1.1.3),** from a fresh clone of the tag `v0.1.2` (`b1e6d67`, main's commit, whose tree is the
pull request's head's, `75b1f2f`; clean, version 0.1.2), in a folder of its own beside
another agent's checkout, which was left untouched. The tag's CI passed and drafted the
release with its five assets. Over SSH from the Windows reference machine, at low priority
(`nice -n 19`, 8 jobs), `cargo xtask bundle-universal ca72-plugin --profile bundle` (Rust
1.97.1, through rustup's proxy: the login shell's `cargo` is stable's) and `scripts/auv2.sh`
made the bundles and the Audio Unit. Signing could not be done over SSH, where the keychain
is locked (`codesign`: `errSecInternalComponent`; `notarytool`: `keychainLocked`), so the
owner ran `scripts/package.sh` with the two Developer ID identities, the notary profile and
`CA72_REQUIRE_NOTARIZATION=1` in a terminal on the Mac:
- **Apple's notary service** accepted it: submission `1c5d7685-2a84-4260-8058-e104c4fa5e6c`,
  status `Accepted`. The ticket is stapled (`stapler validate` passes); the installer is
  signed `Developer ID Installer: Idle Foundry Ltd. (3JA8JUZ36W)`, trusted timestamp
  2026-10-06 19:26:06 UTC; Gatekeeper (`spctl --assess --type install`): `accepted`,
  `source=Notarized Developer ID`. `pkgbuild` printed "write: Permission denied" four times
  for each bundle, as `package.sh` notes it does on macOS 15 and later; the payload is whole.
- **The VST3, the CLAP and the Audio Unit in its payload** (expanded with `pkgutil
  --expand-full`; the first release with the Audio Unit): each signed `Developer ID
  Application: Idle Foundry Ltd. (3JA8JUZ36W)` with the hardened runtime and a secure
  timestamp, `codesign --verify --strict` passing; version 0.1.2; `x86_64 arm64`; LICENSE
  and the notices inside; installed to `/Library/Audio/Plug-Ins/VST3`, `…/CLAP` and
  `…/Components`. The component's CLAP is the CLAP choice's, byte for byte, its signature
  valid.
- **Its SHA-256** is `8c9ed3bba0ba17bc415136cde0cc98a94441fe7a0bf1bcaf044fb3fba9f57a7d`
  (14,059,118 bytes). It went into the draft with its line added to CI's `SHA256SUMS.txt`;
  the four files it lists, downloaded again from the draft, passed `sha256sum -c`. CI's
  Windows plug-in, from the pull request's run, imports Windows' own libraries only (R32).
- **Published** on the owner's word at 2026-10-06 19:28:40 UTC as the latest release
  (https://github.com/idlefoundry/ca-72/releases/tag/v0.1.2, six assets), with the notes
  drafted in #9. The API's `releases/latest`, which CHECK FOR UPDATES reads, gives `v0.1.2`;
  the public downloads have the draft's sizes, and the public `SHA256SUMS.txt` is the one
  uploaded.

`notarization.json`, the build's and the packaging's logs, the commands (`build.sh`) and
`SHA256SUMS.txt` are kept with the release's evidence outside the repository, as 0.1.1's are.
The release's clone then went to the Mac's Trash, and R31's test CLAP to the reference
machine's Recycle Bin; its VST3, which Sandyne had loaded, was gone from the folder once
Sandyne closed, another change's test build (`CA-72 R34`) in its place.

## R34. MIDI Learn
**The owner's request, 2026-10-06:** a focused first MIDI Learn. Each sound control (knobs,
selectors, switches) learns a hardware controller's absolute 7-bit control change on its exact
channel from its context menu, which shows the assignment and a clear waiting state, with a
keyboard-accessible list besides; Escape, Cancel and closing the editor cancel; one controller
a parameter and one parameter a controller, a controller reused moving and saying from where;
the capture message changes nothing. The wheels keep pitch bend and CC 1; the wheels, POWER,
MIDI BEND RANGE and the editor's own actions are not learned. Bank select, the modulation
wheel, data entry and RPN/NRPN, and the channel mode messages are reserved, and explained while
learning. Each parameter's own normalization; switches off at 0–63, on at 64–127; selectors
deterministic; "jump" takeover; only the dezippering learned knobs need. Saved per instance in
the host's project, versioned, by stable id, apart from the CA-72's presets (which leave it
alone); an older project opens with none, even into an instance with some; malformed,
duplicate and unknown entries handled safely; no learning saved; no global defaults. Through
nih-plug's existing MIDI path (VST3's MIDI CCs arrive as hidden parameters), the parameter set
from the audio thread so that its value, the voices, the editor and the host agree with the
editor open or closed; event timing honoured and automation's order with a controller defined;
no duplicate recording (CLAP's `DONT_RECORD`); nothing allocated, locked or waited for on the
audio thread. Deferred: relative encoders, 14-bit pairs, NRPN/RPN, MIDI 2.0, profiles, feedback
to controllers, Omni.

**Agent decisions, 2026-10-06** (not separately approved):

### What is learned, and what is not
- **The list is explicit** (`crates/ca72-plugin/src/learn.rs`, `LEARNABLE`, 47): every panel
  knob (19), RANGE and WAVEFORM (6), every panel and left hand controller switch (15, NOISE's
  WHITE/PINK among them), the strip's POLY, VOICES, ENTROPY and SPREAD, and LOCK, which has no
  control and is learned from the list. That is every parameter a preset sets but the
  MODULATION wheel and MIDI BEND RANGE (a test holds the two lists together). Not learned: the
  PITCH and MODULATION wheels (pitch bend and CC 1 move them already), POWER (the host's
  bypass), MIDI BEND RANGE (the player's, R18), anything else of the editor's.
- **Reserved** (`learn::reserved`): CC 0 and 32, 1, 6 and 38, 96–101, 120–127. They go where
  they went: CC 1 the MODULATION wheel, 120, 121 and 123 as R18 has them, the others nowhere. A
  reserved controller moved while a control waits is named in the note ("CC 1 IS THE
  MODULATION WHEEL: NOT LEARNED") and the control waits on.

### Values
- A 7-bit value `v` (nih-plug gives `v / 127`; `round(x × 127)` takes it back) sets a knob to
  `v / 127` of its travel through the parameter's own normalization; a selector or VOICES to the
  position `⌊v × n / 128⌋` of its `n` (each an equal share of 0–127: RANGE's six change at 22, 43,
  64, 86 and 107), snapped by the parameter's own steps; a switch (two positions) off at 0–63, on
  at 64–127. 64 is not a knob's exact centre (64/127): TUNE sits 0.02 semitone above it, an
  oscillator's FREQUENCY 0.06 (said in the README's limitations).
- **"Jump" takeover**: the first message sets the control there. A value the parameter already
  has (a selector's other values within one position, a controller sending the same value)
  changes nothing and tells the host nothing.

### Learning
- **The audio thread only reads the assignments** (`learn::MidiMap`): a table of 16 × 128
  atomics, each the learnable parameter's index plus one. Every change is made off it under a
  mutex it never takes, which keeps at most one controller a parameter: learning a parameter
  lets its old controller go; a controller another had moves, and `Assigned` says which
  (`displaced`) and what it replaced. Removing one leaves the sound as it is.
- **The capture**: the editor arms a parameter (an atomic word, its index and the arming's
  number). The audio thread takes the next learnable control change as the one, with a
  compare-and-swap that disarms it, and posts the parameter, channel and controller in another
  atomic word; that message sets nothing. The editor assigns it at its next frame
  (`MidiMap::poll`), only if the arming is still the one caught for: armed again elsewhere or
  cancelled meanwhile, it is dropped. Until then that controller's further moves do nothing (a
  frame at most). Arming another parameter moves the learning; cancelling keeps every
  assignment. Closing the editor assigns a controller already caught (its message came) and then
  disarms, so nothing is left armed with no editor to show it; nothing of learning is saved.

### In the editor (`crates/ca72-plugin/src/learning.rs`; `ca72-panel`'s `learn.rs`)
- **A right click** on a control opens its menu, drawn over the panel in the tips' colours: its
  name and controller (`CUTOFF FREQUENCY · CH 1 · CC 74`, or `NO MIDI CONTROLLER`), **MIDI
  LEARN** (**CANCEL MIDI LEARN** while it waits), **REMOVE MIDI ASSIGNMENT** (dim with none),
  **MIDI ASSIGNMENTS…**. A strip control's opens over the panel's foot. The wheels and POWER
  open one that says why they are not learned. A press outside it only closes it; a right click
  first ends any gesture (R18); losing the window's focus closes it.
- **Waiting**: the control is ringed in the drawer's accent (dashed), on the panel or the strip,
  and a note over it says what it waits for, how to stop, and which controllers are not learned
  (or the reserved one just moved). The hover tip gives way to it. When it is learned the note
  names the controller, and the control it was taken from, for four seconds.
- **The list**: the drawer's tools gain a **MIDI** button (the search field 220 units shorter
  for it) that shows, in the presets' place, every learnable control with its controller (or
  WAITING), LEARN, REMOVE and CANCEL, the keys and the reserved controllers in the chips' row,
  and what was last done in the save row. The keyboard works it while the drawer has the keys:
  Up, Down, Page Up and Down, Home, End choose; a letter finds the next control beginning with
  it; Enter learns or cancels; Delete or Backspace removes; Escape cancels, then closes. The
  drawer opens on the presets again the next time.
- **The keyboard**: a menu opened or learning begun takes the keyboard as the drawer does (R10,
  R18, R26: baseview's focus on macOS and Windows; X11 gives the editor the keys under the
  pointer), so that Escape reaches it, and gives it back when neither is left, checked each
  frame (a controller ends learning without an event in the window).
- **Every key while it holds them** (Windows; baseview's third change,
  `third_party/baseview/PATCHES.md`): while the drawer is open or MIDI Learn holds the keyboard,
  the editor's window answers a dialog's `WM_GETDLGCODE` with `DLGC_WANTALLKEYS`; otherwise
  as before. REAPER's FX window is a dialog: there an arrow moved REAPER's focus to one of its
  own buttons, so the list never had it (found on Windows, below). The drawer's own keys
  (R10's arrows, Enter, Escape and Tab) had the same fate there, untried until now.
- `ca72-panel`'s approved image is unchanged: nothing of this is drawn at rest.

### On the audio thread
- `Ca72::process` sends a learnable control change to the table and, for an assigned one, sets
  the parameter through the host at its event's sample (`ProcessContext::set_parameter_normalized`,
  below): the parameter is at the new value at once, for the host, the editor (which reads the
  parameters each frame, R5) and the state; with the editor closed the same.
- **Dezippering, learned knobs only** (`learn::Dezip`). A 7-bit step is coarse: CUTOFF's wiper
  spans 20 V through 200 kΩ into the control node, where R51's 100 kΩ takes 0.98 V an octave,
  so the knob spans about ten octaves and a step about 0.08 (a semitone); a step of MAIN OUTPUT
  VOLUME is 0.8 % of its travel; of TUNE, 3.9 cents. **Measured** (`a_controllers_steps`, by
  hand: a held A2 sawtooth, each knob turned in a quarter of a second by the host's automation
  every 32 samples, by the controller's steps taken at once, and as built; the energy above
  8 kHz, where the note has little, against the automation's): MAIN OUTPUT VOLUME 10 to 3,
  steps **+6.64 dB** (clicks: the output's gain steps after the circuit), gliding +0.88;
  EMPHASIS 2 to 9, steps **+1.68 dB**, gliding −0.07; CUTOFF −1 to 2 at EMPHASIS 7 and 9.5,
  steps +0.02 and +0.13, gliding −0.06 and −0.16; OSCILLATOR-1 VOLUME 8 to 2 −0.01 and +0.15;
  TUNE −1 to 1 +0.03 and −0.03 (the filter and the pitch take a step without a click). The
  whole's level within 0.1 dB stepped; gliding, MAIN OUTPUT VOLUME's 1.0 dB above (its glide
  lags the turn by half of 10 ms), the others within 0.3 dB. So a glide is needed for some
  knobs and costs the others nothing measurable: every knob a learned controller moves glides
  alike, simpler to know than a list. The voices take it from where they had it to the new
  value over 10 ms (`DEZIP`), linearly, their controls set every 32 samples (`DEZIP_STEP`),
  runs no longer than that while one moves; a second message mid-glide glides on from where it
  is. Only the voices glide: the parameter jumped. Switches, selectors and VOICES do not. The
  host's automation, the editor and presets reach the voices as they always have: anything
  setting a gliding knob's parameter ends its glide there. nih-plug's smoothers are not used:
  none of the parameters has one, and giving them one would change automation's sound too.
- Nothing allocated or freed (`tests/realtime.rs`, the plug-in's own `process` with every kind
  of learned control, a capture, reserved controllers and automation ending a glide).

### Through the host (`third_party/nih-plug/PATCHES.md`, changes 10 and 11)
- **The MIDI path is nih-plug's as it was**: CLAP's MIDI events and VST3's hidden MIDI CC
  parameters (`IMidiMapping` maps every controller on every channel to one, which the wrapper
  turns into `NoteEvent::MidiCC`) reach `process` as before. Nothing is asked of a host's own
  MIDI mapping.
- **`ProcessContext::set_parameter_normalized`** (change 10) sets the parameter as each wrapper
  already sets the host's automation during a process call, and tells the host: CLAP an output
  `CLAP_EVENT_PARAM_VALUE` at the change's time flagged `CLAP_EVENT_DONT_RECORD` and no gesture
  (`clap/ext/params.h`: "Turning a knob via plugin's internal MIDI mapping"), so the host shows
  the value without recording automation over the MIDI it records (nothing more: no rescan of
  the parameters, below); VST3 a point in the call's
  `outputParameterChanges` at its sample offset, VST3's way for a processor to tell its host
  and controller (Steinberg's Communication FAQ). Room for 1024 changes a call is reserved when
  the wrapper is made. No feedback loop: the plug-in tells the host only of its own changes, never
  of the host's automation, and nih-plug ignores a controller value the host sets while
  processing (R4).
- **The order at one sample** (change 10): a host's parameter change is set before the events
  the plug-in reads at its sample, in both formats, whatever the host's order (VST3 sorts by
  time, parameter changes first; CLAP splits a run before an event a parameter change follows at
  its sample). So the host's automation and a learned controller of one parameter take turns in
  time: the later holds, and at one sample the controller (set after the automation). A host
  playing automation moves the parameter back at its next point. Without the CLAP change the
  test's "listed first" case gave the automation (checked).
- **`TestProcessContext`** (change 11): a context with no host, for the plug-in's tests of
  `process`, since nih-plug keeps its parameter setters to itself.

### Saved with the project
- **`midi_map`, a persisted field** (with `preset` and the editor's width): JSON
  `{"version":1,"assignments":[{"param":"cutoff","channel":1,"cc":74}, …]}`, by the
  parameters' stable ids, the channel 1 to 16 as shown, in the list's order. A host's project
  and its presets of the plug-in hold it; the CA-72's own presets never do (R10: they hold
  parameters' values), and choosing, saving, replacing or reverting one leaves it alone.
- **Read leniently** (`learn::Saved`): anything but an object of version 1 is an empty table;
  an entry of an unknown or unlearnable parameter, a channel outside 1–16, a controller outside
  0–127 or reserved, or a parameter or controller already assigned above it in the file, is left
  out (the first kept). `Plugin::filter_state` writes a state's table back checked before it
  loads, and gives a state without one (saved before this) an empty one: nih-plug sets only the
  fields a state names, so an older project loaded into an instance with assignments would
  otherwise have kept them.
- The parameters, their ids, ranges and defaults, the voice and the presets' format are 0.1.1's:
  a project saved before opens with the same sound and no assignments.

### Tests
`learn::tests` (13: the list against the presets' parameters, every kind's conversion, the
reserved controllers, catching and assigning, cancelling, arming another, relearning and
moving, channels, the JSON both ways, malformed, duplicate and unknown entries, a state made
good, the glide); the plug-in's (`lib.rs`: a controller set at its sample through the host and
told once, the capture changing nothing, reserved controllers keeping their paths and channels
told apart, switches, selectors and VOICES, the glide and automation ending it, automation and
a controller taking turns, instances apart, a session's table and an old one's, the editor
never open in any); `tests/clap_host.rs` (a CLAP host in the test process, through the
plug-in's own entry point: the change told at its time flagged `DONT_RECORD` and no rescan
asked, the order at one sample both ways, the table out and back with the state and gone with
an older one or a broken one, instances apart); `tests/realtime.rs` (nothing allocated); the
editor's (the menu learns, cancels with Escape, removes; arming another and a controller moved;
closing cancels and keeps what was caught; the strip's controls and the wheels' and POWER's
menus; the MIDI list from the keyboard and the pointer; a right click ending a drag; on
Windows, in a real window, the window asking a dialog for every key only while the drawer or a
control's menu holds the keyboard); `presets::tests` (presets leave the assignments alone);
`ca72-panel` (a menu's items found where drawn, a menu and a note kept on the panel).

### In hosts
On the Mac each host loaded one build of the CA-72 only, named "CA-72 MIDI TEST" so as not to replace the
installed CA-72, and a virtual MIDI port sent the controllers. Kept apart:
- **Automated, no host:** everything under Tests; `tests/clap_host.rs` is the plug-in's CLAP
  entry point driven by a host in the test process (the event, its flag and time, no rescan
  asked, the order at one sample, the state).
- **By hand, Ableton Live 12.2.7 Beta, VST3 (macOS):** a control's menu, MIDI LEARN's ring and
  waiting note, CC 1 while waiting explained and the control still waiting, CC 74 learned with
  the capture changing nothing and the next value setting CUTOFF FREQUENCY, and the controller
  working with the editor closed. Recorded with Automation Arm on: the clip got the controller
  (Live's "MIDI Ctrl 74" envelope) and the track got Cutoff Frequency automation from the
  plug-in's reported values: the VST3 limit above. Played back, the clip moved CUTOFF along its
  sweep (and moving to its start sent the controller's value there), and Live marked the track's
  automation overridden. A controller on channel 2 arrived as channel 1. Saved into a set and the
  set reopened: the set's plug-in state held `"midi_map": {"version":1,"assignments":
  [{"param":"cutoff","channel":1,"cc":74}]}` beside the parameters, the reopened menu named
  `CH 1 · CC 74`, and CC 74 at 64 set CUTOFF to its centre. The drawer's MIDI list took the
  arrow keys.
- **By hand, Bitwig Studio 5.2.7, CLAP (macOS):** the controller reached the plug-in and set
  the parameter (a diagnostic build's counters: two controllers, one learned, one told), but
  Bitwig's display kept the old value. A diagnostic build sending the same event with other
  flags: with `DONT_RECORD` (flags 2 or 3) the display stayed; with none (0) it followed, and
  each value was an undo step. A values rescan after each block with a change was added for
  it (and passed the CLAP test host), never tried in Bitwig, and taken out again when the
  Audio Unit (R30) came in from main: clap-wrapper's AUv2 takes any rescan, values only too,
  for a new parameter list (`setupParameters()`, then `ParameterList`, `ParameterInfo` and
  `ClassInfo` changed), which a turning knob would have asked for many times a second. The
  Audio Unit is told each value by the event itself (`onPerformEdit`, then a
  `kAudioUnitEvent_ParameterValueChange`). So Bitwig's display does not follow a learned
  controller, a limit of Bitwig's, said in the README.
- **By hand, REAPER 7.82 on Windows 11 (the reference machine, a screen at 200 %), VST3 and
  CLAP,** the build named `CA-72 R34` (IDs of its own), bundled with MSVC as the release is, in
  a REAPER with a settings folder of its own (`-cfgfile`: the owner's audio settings, the
  plug-in folders pointed at the build). REAPER's scripts sent the MIDI
  (`StuffMIDIMessage`), the pointer's messages were posted to the editor's window, and keys
  were real (`SendInput`), sent only while REAPER's window was in front:
  - The menu (`CUTOFF FREQUENCY · NO MIDI CONTROLLER`), MIDI LEARN's ring and note, CC 1
    explained, the capture changing nothing (0.5), `CH 2 · CC 74` learned and its 127 setting
    1.0, and CC 74 on channel 1 then changing nothing: REAPER keeps the channels apart.
  - The keyboard: learning took REAPER's focus from its FX window's preset box to the editor's
    window. Before baseview's third change a real Up arrow moved the focus to a button of
    REAPER's and the list stayed; after it, Up moved the list's choice, and Down, Enter (EMPHASIS
    waiting, then `CH 1 · CC 71` learned) and Delete (removed) worked. Escape closed the FX
    window, before the change and after it, in VST3 and in CLAP (which stops learning: closing
    the editor does); with the plug-in's "Send all keyboard input to plug-in" on (`WAK 1`), it
    cancelled learning, the window stayed, and the focus went back to the preset box.
  - Recording a sweep (50 CCs of 74, 0 to 127) with the track's automation in Trim/Read: the
    item got the CCs, no envelope, and the undo history "Recorded media". In Write mode REAPER
    wrote a Cutoff Frequency envelope besides, from the VST3's reported values (46 points) and
    from the CLAP's (48), though the CLAP's are flagged `DONT_RECORD`: REAPER does not honour
    the flag. A learned move left "Edit FX parameter" as the last undo step (the capture did
    not).
  - Playback: the item's CCs moved CUTOFF (1.0 by 1.2 s); as playback stopped REAPER sent CC 74
    with 0 (a MIDI logger before the plug-in saw it), and CUTOFF followed.
  - The CLAP's project saved and REAPER started again: the menu named `CH 1 · CC 74`, and CC 74
    set CUTOFF (live MIDI reached the reopened track only once its effects changed, a MIDI logger
    added before the plug-in: REAPER's, seen once).
- **By hand, Sandyne 2.5.0.0 on Windows 11 (the reference machine), VST3 and CLAP,** the same
  `CA-72 R34` build in the user's plug-in folders (`%LOCALAPPDATA%\Programs\Common`, R31's
  `CA-72 R30` builds to the Recycle Bin), the clicks real (`SendInput`, Sandyne in front;
  Sandyne, built with JUCE, ignores posted ones). The owner's Moog Sub 37 sent nothing over USB
  (a MIDI monitor on its port heard no note or controller, twice), so, with the owner's
  approval, loopMIDI 1.0.16.27 (Tobias Erichsen's, its installer's signature checked) was
  installed and a port of its, `CA72 Test`, enabled as Sandyne's MIDI input; a script sent the
  controllers into it, on channel 4 as the Sub 37 is set to send:
  - Sandyne found both builds when asked to scan, put the VST3 on a track of its own (armed)
    and opened its editor in a window of its own, drawn a pixel a point (no display scale
    given: MIDI Learn's menu and note are small, as the hover tips are).
  - MIDI LEARN on CUTOFF FREQUENCY: CC 1 explained, then Sandyne's own CC 123 explained (it
    sends one at times), CC 74 on channel 4 learned (`CH 4 · CC 74`), the capture changing
    nothing; a sweep to 127 turned CUTOFF fully up and 0 fully down.
  - A real Escape cancelled learning on EMPHASIS and the window stayed: Sandyne gives the
    plug-in its keys.
  - The project saved (`.sand`, the plug-in's state in JUCE's VST3 form: its component state
    held `{"param":"cutoff","channel":4,"cc":74}`), Sandyne started again and the project
    opened: CC 74 at 127, sent with the plug-in's window shut, turned CUTOFF fully up, and the
    menu named `CH 4 · CC 74`.
  - The CLAP on a track of its own: `CH 4 · CC 75` learned and its sweep moved CUTOFF.
  - Not tried in Sandyne: recording controllers or automation, playback.
- **Not checked in a host:** a CLAP host's recording, playback and reopening on macOS; the
  Audio Unit (R30, merged while this was made) in Logic or GarageBand: its MIDI, the editor's
  learning, and what those hosts record (an AU has no "don't record" either); Escape in a host
  on macOS (the tool driving the Mac's hosts sent Escape to no application, not even to close
  Live's own menu; the editor's tests cover it); REAPER on macOS, Cubase and other hosts;
  REAPER's Touch and Latch modes; Linux. The owner stopped the checks in Bitwig after the clash
  below, kept them to Live on the Mac, and then asked for Windows.
- **Seen besides, not MIDI Learn's:** in Bitwig, which loads every plug-in into one process,
  a second, differently built CA-72 opened a blank editor. softbuffer 0.4.8's macOS backend
  defines an Objective-C class under a fixed name, `SoftbufferObserver` (objc2's
  `define_class!`), and a second copy of the library in the process cannot define it again,
  so any two plug-ins carrying their own softbuffer 0.4 in one process would meet it. Left
  for its own change.

### Not done, and why
- **VST3 has no "don't record"**: a VST3 host that writes automation for a plug-in's own
  changes may record a learned move as automation as well as the MIDI it records (Steinberg:
  "some hosts support writing of automation for parameters sent back (outputParameterChanges)").
  The alternative, `restartComponent(kParamValuesChanged)` from the GUI thread, would keep such
  hosts from recording but re-read every parameter (and nih-plug's 2,080 hidden MIDI ones) for
  each move, and the host's view would no longer follow at the move's sample; the other,
  `IMidiMapping` mapping a learned controller to its parameter in the host, would take the
  controller from nih-plug's MIDI path into each host's own, which hosts implement unevenly (and
  the request kept nih-plug's path). Left as the format's documented way, and said in the README.
  Live 12.2.7 is such a host (below).
- **Bitwig shows nothing of a value flagged not to be recorded** (above): its display of a
  learned control stays as it was until something else sets it; the sound, the plug-in's editor
  and the saved state have the value. Without the flag each move would be an undo step, and a
  rescan costs the Audio Unit its parameter list; left to Bitwig.
- **Live gave the VST3 a channel-2 controller as channel 1** (below): there a controller is
  learned as `CH 1` whatever its channel, and one CC number on two channels is one controller.
  The channel is lost before the plug-in (nih-plug's hidden parameters keep all 16 apart, and the
  plug-in's tests tell channels apart); nothing the plug-in can change.
- **CLAP, upstream and unchanged:** `handle_in_events_until` never splits a run at the first event
  of a call's queue, so a parameter change first in a block at a later sample is set from the
  block's start (seen reading it; not this change's to alter: it moves automation in every CLAP
  host).
- Deferred, as asked: relative encoders, 14-bit pairs, NRPN/RPN, MIDI 2.0, profiles, feedback to
  controllers, Omni, soft takeover, global defaults.

**Evidence (2026-10-06, the Mac: Apple M4 Pro, macOS 27.0, Rust 1.97.1; a host open, so
pluginval's editor tests skipped), on main with R30–R33 merged and the rescan taken out:**
`cargo test --workspace` 265 passed, 0 failed (26 ignored, run by hand); clippy with
`-D warnings` on the workspace, all targets, all features; `cargo fmt --all -- --check`;
`scripts/notices.py --check` (the notices unchanged: serde was linked already, `clap-sys` is for
the tests). Before the merge, `preset_render` against main's code: all 24 factory presets the
same to the bit; with nothing learned, `process` the same to the bit as the code before
(`with_nothing_learned_the_sound_is_as_before`, run again after it). `cargo xtask bundle
ca72-plugin --profile bundle` and `scripts/auv2.sh`, then `scripts/validate.sh`: clap-validator
0.4.1, 37 passed, 0 failed, 7 skipped; pluginval 1.0.4 at strictness 10, SUCCESS; Steinberg's
validator (SDK 3.8.1), 47 passed, 0 failed (its bypass test prints two messages it does not
count as failures: nih-plug's controller ignores `setParamNormalized` while processing,
upstream's and untouched); auval `-strict`, AU VALIDATION SUCCEEDED; pluginval on the Audio
Unit, SUCCESS. (The Audio Unit validate.sh leaves in `~/Library/Audio/Plug-Ins/Components` was
put back to the one there before.)

**Evidence on Windows (2026-10-06, the reference machine: i5-13600K, Windows 11, a screen at
200 %; Rust 1.97.1, GNU and MSVC),** this branch with baseview's third change: `cargo test
--workspace` in the desktop session 269 passed, 0 failed (26 ignored) with each toolchain,
`the_editor_asks_a_dialog_for_every_key_only_while_it_holds_them` and R28's real-window tests
among them (over SSH, with no desktop, R28's
`the_panel_is_shown_again_as_windows_repaints_its_window` sees one colour and fails: it needs
a desktop, R28); clippy with `-D warnings` on the workspace and all targets (MSVC with
`--all-features`, as CI; GNU with `--features ca72-plugin/standalone`); `cargo fmt --all --
--check`. CI's Windows job (MSVC) on the pull request, before the third change: 268 passed, 0
failed; clap-validator 36 passed, 0 failed, 8 skipped; pluginval at strictness 10, its editor
tests included, SUCCESS; Steinberg's validator 47 passed, 0 failed.

## R35. 0.1.3
**Owner decision, 2026-10-06.** "merge it and release 0.1.3 if you think it's ready to go":
MIDI Learn (R34) as 0.1.3.

**What it brings since 0.1.2** (the tag `v0.1.2`):
- **MIDI Learn** (R34): a hardware controller's control change for each sound control, from
  the control's menu or the MIDI list in the presets' drawer, kept with the project.
- **Windows: the presets' drawer and the MIDI list take the arrow keys, Enter and Tab** in a
  host whose plug-in window is a dialog, REAPER's (baseview's third change, R34).

**Agent decisions, 2026-10-06** (not separately approved):
- **Ready, the agent judged:** the workspace's tests, clippy, the validators and CI pass on
  every system; MIDI Learn was tried by hand in Live 12 (VST3, macOS), REAPER 7.82 (VST3 and
  CLAP, Windows) and Sandyne 2.5 (VST3 and CLAP, Windows), and partly in Bitwig 5.2 (CLAP,
  macOS). Not tried: the Audio Unit in a host. It tells the host of a learned value as it does
  of an edit in the editor (both reach clap-wrapper as the CLAP's output values), which
  GarageBand took (R30).
- **0.1.3, as asked,** though MIDI Learn is a feature: the parameters, the plug-in's IDs, the
  voice and the presets' format are 0.1.2's, so what was saved with 0.1.0 to 0.1.2 opens
  unchanged, and the installers install over theirs. A project saved with 0.1.3 opens in an
  earlier version without its assignments (nih-plug reads only the fields it knows).
- **The version** as R29: the workspace's (`Cargo.lock` changed only in its eight crates);
  the README's status names 0.1.3, its MIDI Learn section says it is there from 0.1.3, and its
  line on the Audio Unit no longer waits for 0.1.2.
- **Released the way R29 and R33 were:** a commit of its own in R34's pull request, so that CI
  runs once before the tag; once merged, the tag `v0.1.3` on main's commit; CI's release job
  drafts the release with the Windows and Linux installers, the notices, the git sources and
  `SHA256SUMS.txt`; the macOS installer is built, signed and notarised on the release Mac
  (`docs/macos-release.md`) and put into the draft; then published, with notes in 0.1.2's
  form.

**The release (2026-10-06, the release Mac: an M4 Pro, macOS 27.0, Xcode 27.0, notarytool
1.1.3).** The pull request's CI passed on the bump (`cb21704`) on every system; it was merged
(`07a7dfb`, main's commit, its tree `cb21704`'s) and tagged `v0.1.3`, whose CI passed and
drafted the release with its five assets. The macOS installer was built from a fresh clone at
`cb21704` (nothing in the build records the commit), made while the tag's CI ran: `cargo xtask
bundle-universal ca72-plugin --profile bundle` (Rust 1.97.1) and `scripts/auv2.sh`, the bundles
then passing `scripts/validate.sh` (clap-validator 37 passed, 0 failed, 7 skipped; pluginval at
strictness 10, SUCCESS; Steinberg's validator 47 passed; auval `-strict`, AU VALIDATION
SUCCEEDED; pluginval on the Audio Unit, SUCCESS; editor tests skipped), and `scripts/package.sh`
with the two Developer ID identities, the notary profile and `CA72_REQUIRE_NOTARIZATION=1`
making `CA-72-0.1.3-macOS.pkg`:
- **Apple's notary service** accepted it: submission `9892bffb-b042-4427-8e06-2a3155bba08f`,
  status `Accepted` (some 50 minutes in Apple's queue). The ticket is stapled (`stapler
  validate` passes); the installer is signed `Developer ID Installer: Idle Foundry Ltd.
  (3JA8JUZ36W)`, trusted timestamp 2026-10-06 21:14:48 UTC; Gatekeeper: `accepted`,
  `source=Notarized Developer ID`.
- **The VST3, the CLAP and the Audio Unit in its payload**: each signed `Developer ID
  Application: Idle Foundry Ltd. (3JA8JUZ36W)` with the hardened runtime, `codesign --verify
  --strict --deep` passing; version 0.1.3; `x86_64 arm64`.
- **Its SHA-256** is `19911160f8398cab3a3f9b103ac4dd5d90970dc2720c3bf3bf3a44c339d3f421`
  (14,293,865 bytes). It went into the draft with its line added to CI's `SHA256SUMS.txt`; the
  four files it lists, downloaded again from the draft, passed `shasum -a 256 -c`.
- **Published** on the owner's word ("are we able to make this the latest release?") at
  2026-10-06 22:15:21 UTC as the latest release
  (https://github.com/idlefoundry/ca-72/releases/tag/v0.1.3, six assets), with notes in
  0.1.2's form. The API's `releases/latest`, which CHECK FOR UPDATES reads, gives `v0.1.3`;
  the `.pkg` downloaded from the public release matches its checksum and Gatekeeper accepts it.

`notarization.json`, the build's, the validators' and the packaging's logs, the commands
(`build.sh`) and `SHA256SUMS.txt` are kept with the release's evidence outside the repository,
as 0.1.1's and 0.1.2's are. The release's clone then went to the Mac's Trash.

## R36. The editor's switches and buttons in Cubase
**Owner decision, 2026-10-07.** Users reported that the plug-in's buttons did nothing in
Cubase 15 on Windows 11. The owner had a Cubase trial installed on the Windows machine for the
agent to test with and, on its report, said to "move forward with your recommendations": fix
it in the vendored nih-plug, release the fix as 0.1.4 from the plug-in's own nih-plug rather
than wait for the move to `plugin-kit` (which becomes 0.1.5), and make the same change in
`plugin-kit` for the CA-74 and the MC-79.

**What was wrong.** While the host processes audio, nih-plug's VST3 wrapper did not set a value
the editor changed: it told the host (`performEdit()`) and left the value for the host to send
back to the processor at the next process call, so that a value never changes in the middle of
one; until then the controller reported the value from before the edit. Within `endEdit()`,
Cubase 15 reads the controller's value (`getParamNormalized()`) and sends that to the
processor, after the edit. A click is a whole gesture between two process calls, so Cubase sent
back the old value and the control did not move. Every control set by a click was affected:
the rockers and switches, POWER, POLY, the − and + beside VOICES, the buttons beside ENTROPY and
SPREAD, a selector stepped by a click on its legend, and a double click setting a knob to its
default. A knob being dragged moved, since each move came back before the next, but a quick
drag let go between two process calls could end one move short. Presets were not affected: a
preset's state is set at the end of a process call by another route.

**Agent decisions, 2026-10-07** (not separately approved):
- **Held, not set at once** (nih-plug's `PATCHES.md`, change 12). Setting the value at once
  from the editor's thread would have been simpler, but a value could then change in the middle
  of a process call, which the plug-in's code does not expect (VOICES and POLY are read more
  than once in a call). The edit is held instead, the latest for each parameter: the controller
  reports it, the next process call sets it at its start, before the host's own changes at its
  first sample, and stopping processing sets it if no call will. A host that never sends an
  edit back now has it set as well.
- **A test of the wrapper, with a probe rather than the CA-72.**
  `crates/ca72-plugin/tests/vst3_host.rs` drives the wrapper through its own VST3 factory, with
  a probe plug-in whose editor opens no window but keeps the wrapper's context, and a host that
  does what Cubase 15 does. Its six tests run on every system. Four fail without change 12: the
  click read back as 1.0, a quick drag ending at 0.25 rather than 0.75, an edit lost by a host
  that sends nothing back, and an edit lost as processing stops; the other two guard what did
  not change. With nih-plug's allocation check on (`assert_process_allocs`, built with MSVC),
  all six pass: the change allocates nothing in a process call.
- **The CLAP is untouched:** its wrapper sets an edit itself as it sends it to the host. The
  Audio Unit wraps the CLAP.
- **R36 and R37 numbered at once:** no branch on GitHub had a record after R35 (2026-10-07).
  The unmerged lines elsewhere (the softbuffer fix on the Mac, `perf` on the Linux reference
  machine) take later numbers when they merge.

**Evidence (the Windows machine, 2026-10-07):** Cubase Pro 15.0.30, a trial, with the Steinberg
built-in ASIO Driver at 48 kHz and 480 samples, ASIO-Guard on, Windows 11 at 200 % scaling, and
"Suspend VST 3 plug-in processing when no audio signals are received" off.
- **Before:** the installed 0.1.1 (4,636,160 bytes). A click on OSCILLATOR-1's switch in the
  mixer changed neither the panel nor Cubase's generic editor; the same switch set from the
  generic editor moved on the panel; with the instrument deactivated in Cubase, the click
  worked. The editor's window held the pointer's capture from press to release. A renamed build
  of 0.1.3 that logged the wrapper's calls showed, for one click on that switch, `performEdit`
  with 0.0, then within `endEdit` `getParamNormalized` returning 1.0, then 1.0 at the next
  process call; for a knob's drag, each move came back unchanged.
- **After:** a renamed build of this change, `CA-72 R36`. Clicks on OSCILLATOR-1's switch,
  OSCILLATOR MODULATION and A-440 moved them (about 1,000 of 1,500 sampled pixels around each
  changed; none around two switches not clicked); POLY went from OFF to ON and VOICES from 4 to
  5; a quick drag of CUTOFF FREQUENCY stayed where it was let go, and a double click set it back
  to 0. Cubase's generic editor then showed OSCILLATOR MODULATION on, OSCILLATOR-1 off and
  CUTOFF 0.00.
- `cargo test -p ca72-plugin --test vst3_host`: 6 passed; with the wrapper restored, 2 passed
  and 4 failed.

Not tried: Cubase on macOS (the same wrapper; the test runs on every system in CI), Cubase
before version 15, and the Elements and Artist editions.

**Since R37 (the Windows machine, 2026-10-07):** the owner tried `CA-72 R36` in Cubase 15
themselves: "that one works!". (`CA-72 R34`, an older test build left in the user VST3 folder,
showed them the fault: knobs moving, switches not.) The 0.1.4 that CI built on the pull request
(run 37642706976, the tree of the tag), installed over 0.1.1 with its installer
(`CA-72.vst3` 4,900,352 bytes, SHA-256 `82dff939…`), then did the same in two hosts while they
processed audio: in Cubase 15.0.30, OSCILLATOR-1's switch, OSCILLATOR MODULATION and A-440
moved, POLY went from OFF to ON, VOICES from 4 to 5, and a quick drag of CUTOFF FREQUENCY
stayed; in REAPER 7.82 (a copy run with its own `-cfgfile`, the Dummy Audio driver, playing),
the three switches, POLY and VOICES the same. The installer kept this machine's earlier choice
of the VST3 alone (a custom install), so the CLAP was not installed. Not tried: the release's
own Windows installer in a host (a second CI build of the same tree), the CLAP in a host, and
Linux and macOS hosts.

## R37. 0.1.4
**Owner decision, 2026-10-07.** R36 released at once as 0.1.4, from the plug-in's own
nih-plug; the move to `plugin-kit`, planned as 0.1.4, becomes 0.1.5 (R36).

**What it brings since 0.1.3** (the tag `v0.1.3`):
- **Cubase: the editor's switches and buttons work** (R36). In Cubase 15, and in any VST3 host
  that reads a value back as an edit ends, a click on a switch, a button or a selector's legend
  did nothing, and a quick drag could end one move short. The VST3 on every system; the CLAP
  and the Audio Unit were not affected.

**Agent decisions, 2026-10-07** (not separately approved):
- **A patch release, 0.1.4,** as R29, R33 and R35: nothing a project or a preset holds has
  changed since 0.1.3, so what was saved with 0.1.0 to 0.1.3 opens unchanged, and the
  installers install over theirs.
- **The version** as R29: the workspace's (`Cargo.lock` changed only in its eight crates); the
  README's status names 0.1.4.
- **Released the way R35 was:** a commit of its own in R36's pull request, so that CI runs once
  before the tag; once merged, the tag `v0.1.4` on main's commit; CI's release job drafts the
  release with the Windows and Linux installers, the notices, the git sources and
  `SHA256SUMS.txt`; the macOS installer is built, signed and notarised on the release Mac
  (`docs/macos-release.md`) and put into the draft; then published, with notes in 0.1.3's form.

**The release (2026-10-07, the release Mac: an M4 Pro, macOS 27.0, Xcode 27.0, notarytool
1.1.3).** The pull request's CI passed on the bump (`c81b661`) on every system; it was merged
(`2ee178d`, main's commit, its tree `c81b661`'s) and tagged `v0.1.4`, whose CI passed and
drafted the release with its five assets. The macOS installer was built over SSH from a fresh
clone of the tag (`2ee178d`) while the tag's CI ran: `cargo xtask bundle-universal ca72-plugin
--profile bundle` (Rust 1.97.1) and `scripts/auv2.sh`, the bundles then passing
`scripts/validate.sh` as far as auval (clap-validator 37 passed, 0 failed, 7 skipped; pluginval
at strictness 10, SUCCESS; Steinberg's validator 47 passed; editor tests skipped). Over SSH
auval finds no Audio Unit at all, not even the MC-79's installed one, so the owner ran it in
their desktop session, before `scripts/package.sh` there: `auval -strict`, CA-72 0.1.4 (0x104),
AU VALIDATION SUCCEEDED. pluginval on the Audio Unit was not run on the Mac; CI's macOS job ran
it on the same tree. The owner's `~/Library` Audio Unit (0.1.1) was set aside for each run and
put back. Packaging, with the two Developer ID identities, the notary profile and
`CA72_REQUIRE_NOTARIZATION=1`, made `CA-72-0.1.4-macOS.pkg`:
- **Apple's notary service** accepted it: submission `c5074124-c2b5-4af0-9894-1054764ed4c5`,
  status `Accepted` (about a minute). The ticket is stapled (`stapler validate` passes); the
  installer is signed `Developer ID Installer: Idle Foundry Ltd. (3JA8JUZ36W)`, trusted
  timestamp 2026-10-07 16:20:01 UTC; Gatekeeper: `accepted`, `source=Notarized Developer ID`.
- **The VST3, the CLAP and the Audio Unit in its payload** (with the CLAP inside the Audio
  Unit): each signed `Developer ID Application: Idle Foundry Ltd. (3JA8JUZ36W)` with the
  hardened runtime, `codesign --verify --strict --deep` passing; version 0.1.4; `x86_64 arm64`.
- **Its SHA-256** is `29cfa22841009ad5db843ea742f47822dfb7c50c819aab6e81fef4005291b692`
  (14,300,983 bytes). It went into the draft with its line added to CI's `SHA256SUMS.txt`; the
  four files it lists, downloaded again from the draft, passed `sha256sum -c`.
- **Published** on the owner's go for 0.1.4 (R36) at 2026-10-07 16:33:11 UTC as the latest
  release (https://github.com/idlefoundry/ca-72/releases/tag/v0.1.4, six assets), with notes
  in 0.1.3's form. The API's `releases/latest`, which CHECK FOR UPDATES reads, gives `v0.1.4`;
  the `.pkg` downloaded from the public release, marked as downloaded, matches its checksum and
  Gatekeeper accepts it.

`notarization.json`, the build's, the validators', auval's and the packaging's logs, the
commands (`build.sh`, `auval.sh`, `package.sh`) and `SHA256SUMS.txt` are kept with the
release's evidence outside the repository, as 0.1.3's are.

## R38. The editor's window opaque on Linux desktops that composite
**Owner decision, 2026-10-08.** The CA-74 found that on a Linux desktop that composites, its
standalone's panel looked washed out, far too bright, and fixed it in its editor. The owner
asked for the same fix in the CA-72 and the MC-79, merged only on their go. Shown it tried
(below), asked whether to merge it once CI passed and release it with the next version rather
than on its own: "yes".

**What was wrong.** baseview makes its X11 window with a 32-bit visual, an alpha channel,
wherever the screen has one (`third_party/baseview`, `find_best_visual_config`), and the editor
gave softbuffer each pixel with its highest byte 0, as softbuffer's documentation asks. On that
window 0 is transparent: the compositor adds whatever is behind the window to the panel's
colours. The CA-74's standalone on Hyprland, through XWayland, showed the wallpaper through its
panel, its cream face near white. Inside a Linux host the host's own window under the editor's
would show through the same way.

**Agent decisions, 2026-10-08** (not separately approved):
- **Every pixel the window is given is opaque:** its highest byte 0xff (`window::shown`, and
  `window::EMPTY` for the window beyond the frames, 0xff3b2213). softbuffer 0.4.8's other
  backends ignore that byte: Core Graphics draws with `NoneSkipFirst`, GDI copies a 32-bit
  bitmap with `BitBlt`, KMS's buffers are XRGB8888, and an X11 window of 24 bits has no alpha.
- **Tried off the owner's screen:** the standalone's window was sent to a workspace not shown
  and captured from the compositor by itself (`grim -T`), so the capture shows the alpha the
  compositor was given rather than the colours it would have made of it on screen.

**Evidence (the Linux reference machine, 2026-10-08):** Hyprland 0.56.2 with XWayland; the
standalone (`--example standalone`, nih-plug's dummy audio backend), debug, before the change
(main, `7cdf904`) and after, each captured 8 s after its window appeared.
- **Before:** every pixel of the panel and the strip (2045 by 685) had alpha 0, transparent.
- **After:** every pixel opaque. Against the panel's drawing (`cargo run -p ca72-panel
  --example png` at the editor's scale, 2045 / 3438 pixels a panel unit), the face is 29, 27,
  26 and a legend's cream 235, 230, 216 in both; 72.5 % of the panel's pixels are identical,
  the rest where the knobs and switches stood at other values than the example draws them.
- `the_window_is_given_opaque_pixels` fails without the change (0x123456 given where 0xff123456
  is wanted) and passes with it; the editor's tests (39 passed, 2 that write pictures for a look
  ignored); rustfmt; clippy with and without `--all-features` (rustc 1.97.1).

Not tried: the panel on screen (the CA-74 saw it there), a Linux host, macOS and Windows (the
byte ignored there, by softbuffer's code; CI runs the tests on all three).

## R39. A worker on machines of 3 and 4 processors: the POLY presets in Waveform
**Owner decision, 2026-10-07.** Two users reported the CPU overloaded in Tracktion Waveform: on
KVR, Waveform 14 on Linux Mint 22.3 with "4 cores at 3.6 GHz", where choosing Slow Horn Swell,
Brass Tutti or Warped Pad took the CPU to 100 % "after a moment" and the audio engine had to be
reset, while REAPER showed 8 to 30 %; on Bedroom Producers Blog, Waveform 13 on Windows 10 on
an AMD A8 notebook, 70 to 80 % "on every preset". Shown the cause below, the owner asked whether
it was the host's fault or the plug-in's, and on the answer (mostly the plug-in's) said to give
machines of fewer than 5 processors a worker, the change that keeps the sound and the voices.
On 2026-10-08 the owner said to release it at once: "let's just push the performance fix into
production immediately" (R40).

**What was wrong.** The three presets are the only factory presets with POLY on, and they ask
for 10, 8 and 8 voices. A machine of fewer than 5 processors started no worker (R11), so every
voice played on the host's audio thread, and one thread holds about four or five of these
voices even on the Linux reference machine (R7). Waveform's engine (Tracktion Engine, whose
source is public) mutes its output and renders nothing once its measure of the audio callback's
load, smoothed, passes 0.98 (`cpuLimitBeforeMuting`, `tracktion_DeviceManager`): a plug-in that
keeps the callback near its period makes it fall silent by turns, and its CPU meter sits near
100 %. REAPER renders a track's plug-ins up to 200 ms ahead and shows the whole machine's use, so
the same work there read as one core of four. Waveform adds next to nothing to the plug-in's
cost: the bench without a host and Waveform agree (below). The voice, the engine and the
presets are unchanged from 0.1.0 to 0.1.4, so the version the users had makes no difference.

**Agent decisions, 2026-10-07** (not separately approved):
- **One worker from 3 processors up** (`engine::workers_for`): none on 1 or 2, where the host's
  thread and a worker would be all the machine has; one on 3 and 4, as on 5 to 7; from 5 up as
  before. One, not two, at 4: two would give a machine of 4 more workers than one of 5 to 7,
  and leave the host one processor for everything else. How many play in real time still
  depends on the processor: by the figures below, with one worker a core about 1.5 times
  slower than the Linux reference machine's still plays Brass Tutti under Tracktion Engine's
  limit (before, about 1.15 times), an estimate, not a measurement on such a machine.
- **The samples do not change**: a voice plays the same samples on any thread (R11,
  `tests/workers.rs`), and every factory preset rendered with one worker on 4 processors is the
  same to the bit as 0.1.4's render. Under overload the worker's late voices fall silent for a
  run (R11) rather than the host muting its whole output.
- **CI now exercises the workers**: its runners have 3 or 4 processors, so for the first time
  the pool runs in CI on all three systems.
- **Not done here:** the presets' VOICES (the owner's), the idle sleep and POTATO on the
  unreleased `perf` branch, a published minimum specification.

**Evidence (the Linux reference machine, 2026-10-07 and 2026-10-08; Ryzen 7 7800X3D, 4 of its 8
cores given to the host with `taskset`, so that the plug-in counts 4 processors):**
- **Waveform 14.0.50** (Tracktion's Ubuntu package, unpacked into a folder of its own, a profile
  of its own whose only VST3 was the build under test, its output on a silent PipeWire sink, 44.1
  kHz, quantum 512), the owner playing a clip of one note every 0.25 s. Released 0.1.4 (SHA-256
  a8b1ed8b…): Brass Tutti took Waveform's audio thread to 79 to 86 % of a core, the CPU meter to 80
  to 90 %. This change (211e0124…): Brass Tutti, the audio thread 51 % at its median (57 % at the
  90th percentile) and the worker 52 %, the meter at most about 60 %; Slow Horn Swell 48 % (55 %)
  and 49 %, the meter 55 to 60 %. The threads' times from `/proc` while the owner played.
- **The released 0.1.4 in a host bench** (the MC-79's `hostbench`, paced on a real-time thread,
  CLAP, 48 kHz, 256 frames, 30 s, two runs each), mean of the period, late blocks of 5,625: Bass
  10 to 11 %, 0; Slow Horn Swell 79 to 80 %, 54 to 89; Brass Tutti 71 to 72 %, 3; Warped Pad 65
  to 67 %, 0 to 1 (one note every 0.25 s). With 4-key chords every second, 70 to 83 % and up to
  467 late. 128 and 512 frames and 44.1 kHz much the same. Replayed through Tracktion Engine's
  limit with each block 1.5 times as long (a slower processor), 7 to 23 % of the callbacks muted
  with these presets; none with Bass even at three times.
- `preset_render` against 0.1.4's renders: all 24 presets the same to the bit, with no worker
  and with one on 4 processors. `cargo test -p ca72-plugin` (debug): 152 passed on 16, on 4 and
  on 3 processors; clippy with and without `--all-features`; rustfmt.

Not tried: a machine as slow as the users' (none here), Waveform 13, Windows, Maniac Audio's
Dark Studio, REAPER on this machine, the editor closed against open, and the CLAP in Waveform.

## R40. 0.1.5
**Owner decision, 2026-10-08.** R39 released at once: "let's just push the performance fix into
production immediately", with R38 (its owner's go: with the next version). The softbuffer fix
(R34, "Seen besides"), made on the Mac and not yet merged, comes with the next patch release.

**What it brings since 0.1.4** (the tag `v0.1.4`):
- **Machines of 3 and 4 processors play POLY's voices with a worker** (R39). With the POLY
  presets (Slow Horn Swell, Brass Tutti, Warped Pad) such a machine played every voice on the
  host's audio thread: in Tracktion Waveform the CPU reached 100 % and the audio engine had to
  be reset. The samples are the same to the bit.
- **The editor's window opaque on Linux desktops that composite** (R38): the desktop, or the
  host's own window, no longer shows through the panel.

**Agent decisions, 2026-10-08** (not separately approved):
- **A patch release, 0.1.5,** as R29, R33, R35 and R37: nothing a project or a preset holds has
  changed since 0.1.4, so what was saved with 0.1.0 to 0.1.4 opens unchanged, and the
  installers install over theirs.
- **The version** as R37: the workspace's (`Cargo.lock` changed only in its eight crates); the
  README's status names 0.1.5.
- **The move to `plugin-kit`,** planned as 0.1.5 (R37), takes a later version.
- **Released the way R37 was:** a commit of its own in R39's pull request, so that CI runs once
  before the tag; once merged, the tag `v0.1.5` on main's commit; CI's release job drafts the
  release with the Windows and Linux installers, the notices, the git sources and
  `SHA256SUMS.txt`; the macOS installer is built, signed and notarised on the release Mac
  (`docs/macos-release.md`) and put into the draft; then published, with notes in 0.1.4's form.
- **The Windows ZIP, for installing by hand, made for each release** (the owner, 2026-10-08:
  "don't forget the windows zip file... it should be" part of the procedure). 0.1.4's,
  `CA-72-0.1.4-Windows-x86_64.zip` with its `.zip.sha256`, was made by hand from that tag's CI
  artifact and added after the release was published, outside `SHA256SUMS.txt`.
  `scripts/windows-zip.sh` now makes it the same way: CI's Windows VST3 and CLAP unchanged,
  `README.txt` (installing, updating and uninstalling by hand, the presets' folder, the build's
  provenance), `LICENSE.txt` and `THIRD-PARTY-NOTICES.txt` beside them and in the VST3's
  Resources, and `FILE-SHA256SUMS.txt`. Run on 0.1.4's artifact, every file but the README's
  wording is the same as in 0.1.4's ZIP. CI's release job runs it on the tag's `CA-72-Windows`
  artifact (the job may now read the run's artifacts), and `SHA256SUMS.txt` lists the ZIP;
  its own `.zip.sha256` stays beside it, as 0.1.4's. 0.1.5's was made by the script by hand
  (the release job's step first runs at the next tag), and the release notes give it a row.

**The release (2026-10-08, the release Mac: an M4 Pro, macOS 27.0, Xcode 27.0, notarytool
1.1.3).** The pull request's CI passed on the bump (`0eda08f`) on every system; it was merged
(`38ed3cf`, main's commit, its tree `0eda08f`'s) and tagged `v0.1.5`, whose CI passed and
drafted the release with its five assets. The macOS installer was built over SSH from a fresh
clone of the pull request's head (`0eda08f`), started while its CI ran, and signed only after
the tag's tree was checked to be the tree built (`c70b81e`): `cargo xtask bundle-universal
ca72-plugin --profile bundle` (Rust 1.97.1) and `scripts/auv2.sh`, the bundles then passing
`scripts/validate.sh` as far as auval (clap-validator 37 passed, 0 failed, 7 skipped; pluginval
at strictness 10, SUCCESS; Steinberg's validator 47 passed; editor tests skipped). auval and the
packaging ran in the Mac's desktop session without the owner, in a shell opened by LaunchServices
in the background (`open -g -j -na Ghostty.app --args -e ...`): over SSH, and in a tmux session
of the desktop's tmux server, auval sees only Apple's Audio Units, while codesign and notarytool
reach the login Keychain from either. `auval -strict`: CA-72 0.1.5 (0x105), AU VALIDATION
SUCCEEDED. The owner's `~/Library` Audio Unit (0.1.1) was set aside for each run and put back.
Packaging, with the two Developer ID identities, the notary profile and
`CA72_REQUIRE_NOTARIZATION=1`, made `CA-72-0.1.5-macOS.pkg`:
- **Apple's notary service** accepted it: submission `9f468420-e0f4-47ce-8f51-ee7c21677413`,
  status `Accepted` (about a minute). The ticket is stapled (`stapler validate` passes); the
  installer is signed `Developer ID Installer: Idle Foundry Ltd. (3JA8JUZ36W)`, trusted
  timestamp 2026-10-08 20:13:34 UTC; Gatekeeper: `accepted`, `source=Notarized Developer ID`.
- **The VST3, the CLAP and the Audio Unit in its payload** (with the CLAP inside the Audio
  Unit): each signed `Developer ID Application: Idle Foundry Ltd. (3JA8JUZ36W)` with the
  hardened runtime, `codesign --verify --strict --deep` passing; version 0.1.5; `x86_64 arm64`.
- **Its SHA-256** is `df16b9f6e995d563d2adfbcddd2ef7da42853ec1682e3745e4deb7ca9f13aa13`
  (14,300,825 bytes).
- **The Windows ZIP** from the tag's run (37837077300, artifact 11576455731), its SHA-256
  `064fc66dc456073314496eebef0ece263d1002f08c804edf6be5d2bba9fe8ab7` (4,494,839 bytes), the
  plug-in's binary `f92720c7…` in both formats.
- Both went into the draft with their lines added to CI's `SHA256SUMS.txt`, the ZIP with its
  `.zip.sha256`; the five files it lists, downloaded again from the draft, passed
  `sha256sum -c`, and the notices are the tag's.
- **Published** on the owner's go ("release 0.1.5") at 2026-10-08 20:20:30 UTC as the latest
  release (https://github.com/idlefoundry/ca-72/releases/tag/v0.1.5, eight assets), with notes
  in 0.1.4's form. The API's `releases/latest`, which CHECK FOR UPDATES reads, gives `v0.1.5`;
  the `.pkg` downloaded from the public release, marked as downloaded, matches its checksum and
  Gatekeeper accepts it.

`notarization.json`, the build's, the validators', auval's and the packaging's logs, the
commands (`build.sh`, `auval.sh`, `package.sh`) and `SHA256SUMS.txt` are kept with the
release's evidence outside the repository, as 0.1.4's are.

## R41. macOS: the editor drawn beside other copies of softbuffer in one process
**Seen, 2026-10-06,** in Bitwig Studio 5.2.7 on macOS (CLAP), while MIDI Learn was tried there
(R34, "Seen besides"): Bitwig loads every plug-in into one
process, and there a second, differently built CA-72 opened a blank editor.

**The cause,** confirmed on the Mac. softbuffer 0.4.8, which shows the editor's frames, keeps its
layer in step with the editor's view on macOS through an Objective-C class of its own, defined
with objc2's `define_class!` under a fixed name, `SoftbufferObserver`, and registered as the
first surface is made. A process holds one class of a name, and `define_class!` panics when its
name is taken: a second copy of softbuffer, in a second plug-in's library, panicked as its
editor opened ("could not create new class "SoftbufferObserver", perhaps a class with that name
already exists?"), and the editor, which catches softbuffer's panics (R18), drew nothing. Any
two copies meet it: two builds of the CA-72; its CLAP and its VST3, one binary at two paths;
another plug-in carrying softbuffer 0.4 (0.4.6 and 0.4.7 used the same name). Every release so
far, 0.1.0 to 0.1.5, carries 0.4.8. No release fixes it: 0.4.8 is the latest, and upstream's
master defines the class the same way. Two reproductions, kept out of the repository, each a
host that loads the libraries it is given into its one process and works on its main thread:
- a library that makes a softbuffer surface on an NSView of its own, presents a frame and
  resizes the view, as the editor does: with softbuffer 0.4.8, a second library (another build
  of it, or the same one copied to another path) panicked as above;
- a CLAP host that opens each library's editor in a view of its own, then lets the main run
  loop run for 1.5 s: with 0.1.2's plug-in (built from `v0.1.2`) at two paths, the second
  editor's layers held no frame, and nih-plug logged the panic, from the editor's `Surface::new`
  in `PanelWindow::new`, from `clap_plugin_gui.set_parent`.

**Agent decisions, 2026-10-06** (the owner asked for the fix; not separately approved
otherwise):
- **softbuffer vendored and patched,** as baseview and nih-plug are
  (`third_party/softbuffer/PATCHES.md`): the 0.4.8 release, its macOS backend changed so that
  each copy registers a class of its own at run time, under the first of `SoftbufferObserver1`,
  `SoftbufferObserver2`, ... that is free, never upstream's name (which an unpatched copy loaded
  later still needs). Each copy runs only its own code. The class has no state: the layer an
  observer updates comes as the observation's context. `ca72-plugin` takes softbuffer by path,
  with the same features; in `Cargo.lock` the crate loses its registry source and checksum, and
  the plug-in gains the test's two dependencies; no crate is added or changes version.
- **Not objc2's own way out,** leaving the name to `define_class!` (a patch of one line): a copy
  that finds that name taken uses the class it names, and so runs the code of whichever copy came
  first, which objc2 itself calls unsound across libraries. The name,
  `softbuffer::backends::cg::Observer0.4.8`, does not tell code apart: upstream's master, changed
  since the release, still calls itself 0.4.8.
- **The test,** `crates/ca72-plugin/tests/softbuffer_copies.rs`: in a process that already holds
  the classes an unpatched copy and a patched one register, a surface on a view of the test's
  own is made and its frame reaches the view's layer; the layer follows the view as it is
  resized (the observer); a second surface, after the first is gone, registers no second class.
  softbuffer's macOS backend works on the main thread only, where the test harness runs no test,
  so this test runs without it (`harness = false`); on other systems it does nothing. Its view
  and classes take two dependencies on macOS, objc2 and objc2-foundation, at softbuffer's
  versions.

**Evidence (2026-10-06, the Mac: Apple silicon, macOS 27.0; Rust 1.97.1):**
- **The reproductions,** with this copy of softbuffer. The surfaces' library: two builds, and
  one build at two paths, each drew and resized with a class of its own (`SoftbufferObserver1`,
  `SoftbufferObserver2`), each class's method in its own library (`dladdr`); an unpatched build
  loaded before it and after it, both drew; four libraries in one process (patched, unpatched,
  the patched one's copy, a second patched build) took `SoftbufferObserver`, then
  `SoftbufferObserver1` to `3`. The CLAP host, with this tree's plug-in: one build at two paths,
  a debug and a release build, and 0.1.2 loaded before it and after it: every editor's
  softbuffer layer held a frame of the editor's size, 1382 by 463. The plug-in's library no
  longer exports upstream's `__CLASS_SoftbufferObserver` symbols.
- **The new test** panicked as above with upstream's `cg.rs` put back, and passes with the
  change.
- `cargo test --workspace`: 227 passed, 0 failed (25 ignored, run by hand), and the new test;
  clippy with `-D warnings` on the workspace, all targets and features; `cargo check` of the
  plug-in for Windows (MSVC); `scripts/notices.py --check` (the notices are unchanged: the copy
  carries the release's licences and repository); `cargo fmt --all -- --check`, on a copy of the
  tree outside the main checkout (in a worktree inside it, cargo takes the vendored crates for
  the main checkout's workspace and stops, baseview's as well).
- Not tried: Bitwig itself, with two builds of the CA-72 in one project; other hosts on macOS;
  the installer's bundles; the Audio Unit, built around the CLAP (R30), so with the change.
  Linux and Windows compile softbuffer's macOS backend out (CI builds and tests them).

## R42. A hardware reference, and the first change it settled
**Owner decision, 2026-10-07.** The CA-72 is matched to the owner's hardware reference, a
modern recreation of the instrument which the owner could not tell from a Model D in
level- and pitch-matched recordings: "if we match this, that this will be the right thing to
do." The comparisons with hardware become measurements ([calibration](calibration/README.md)).

**Agent decisions, 2026-10-07 and 08** (not separately approved):
- **Change the circuit, never the output:** a difference is traced to a component, trim or
  device model, the netlist and the real-time model change together within their budgets
  against ngspice, and the change is recorded as a deviation from the drawing. Calibration
  is copied; the reference's own features are not targets.
- **The rig:** the MOTU 828ES on the Mac (on Linux its class-compliant USB shifted the
  captured channels once playback started beside a capture), two ES-3 for gates and
  control voltages on the inputs' clock, one stream for stimuli and responses; captures and
  renders on the lab's share, measured by the same code (`scripts/calibration/`,
  `ca72-lab stim`).
- **What the panel at its home settings showed** (oscillators off, EMPHASIS 0): the filter's
  response against its corner matches within 0.7 dB from 600 Hz to 3.6 kHz; the loudness
  contour's attack, held level and decay match with its knobs fitted; with DECAY off one
  contour's release has the reference's shape. Differences that need the knobs moved (the
  corner's mapping, the thump's size, EXTERNAL INPUT VOLUME's taper, the trigger delay from
  MIDI) wait for a session at the instrument.
- **One change: an R1401 for each contour** (`ContourCircuit::dump_each`, board2.md B2-6).
  Through the drawing's one R1401 a contour held higher halved the other's release with
  DECAY off; the reference's is the same whatever the other contour holds. It changes the
  four presets with DECAY off; every other preset renders the same to the bit.
- **The knob session, 2026-10-08** (sessions C to H; the owner's instructions: trace each
  difference to a component, a trim or a device model, netlist and model together, one
  commit each). Changed: the oscillators' external control input to 50.5K (a volt an
  octave); RANGE (R39) to the reference's 0.559; oscillator 3's R162 to 2.96K (its pitch with
  OSC. 3 CONTROL off); and, measured on the reference in place of the generic tapers (A9),
  the laws of EXTERNAL INPUT VOLUME (R9), EMPHASIS (R14), the four ATTACK and DECAY pots and
  the mixer's VOLUME; and, at the owner's go, the external preamplifier's R61 to 232K (its
  gain 0.97 dB up). The 8 and 12 ms keyboard delays stay.
  What still differs is listed in the calibration README.
- **Owner decisions, 2026-10-08, after the knob session's listening page:** level the
  presets, retune the two that sound the filter's own pitch, and settle the external input's
  drive (R61 above). **Cruising Whistle and Ladder Kick retuned:** R39 put the filter's pitch
  3.7 semitones up in both; CUTOFF lowered by 0.305 (Cruising Whistle -1.33 to -1.635, its
  whistle within 5 cents of 0.1.3's at C4, G4 and E4; Ladder Kick -3.6 to -3.9, its ring's
  sweep over the first 11 ms within the measure's resolution of 0.1.3's).
- **The presets re-levelled** to where R15 and R16 put them: each one's momentary maximum
  on a phrase in its register (`tests/preset_levels.rs`), on 0.1.3 and now, and MAIN OUTPUT
  VOLUME moved by the difference through its own law. Twelve within 0.03 dB of 0.1.3 (Three
  Saw Slab 7.5 to 8.29, Undertow Growl 9.17 to 8.77, Warped Pad 7.8 to 7.56, the rest by
  0.03 to 0.22). As R15 has it, a preset at VOLUME 10 under its level stays at 10: nine
  are now quieter than they were (Upright Pluck -3.6 dB, Breath Flute -2.2, Stacked Fifths
  -1.9, Ladder Kick -1.6, Open Hat -1.6, Ringing Saw Line -1.4, Noise Snare -1.0, Shoreline
  Wash -0.5, Wooden Mallet -0.4), and Pink Riser, quiet by nature, 1.3 dB louder and still
  under -18 LUFS. Hollow Glider and Closed Hat moved under 0.1 dB and were left.
- **Again after CUTOFF's law** (change 10) and the contours' 10 s mark: Cruising Whistle
  -1.635 to -1.403 and Ladder Kick -3.9 to -3.504, the same places on CUTOFF's track (the
  law's inverse; the whistle within 4 cents of 0.1.3's, the ring's first 5 ms within 4);
  Pulse Strut 9.05 to 9.36, Undertow Growl 8.77 to 8.73, Hollow Glider 6.2 to 6.28, Brass
  Tutti 4.08 to 4.18, Warped Pad 7.56 to 7.61: every preset VOLUME can reach within 0.1 dB
  of 0.1.3. At 10 and quieter: Upright Pluck -3.65 dB, Breath Flute -1.91, Stacked Fifths
  -1.83, Ladder Kick -1.64, Open Hat -1.60, Ringing Saw Line -1.19, Shoreline Wash -1.06,
  Wooden Mallet -1.04, Noise Snare -0.89; Pink Riser +1.44, still under -18 LUFS.
- **Owner decision, 2026-10-08:** the keyboard follows the reference's MIDI: 0 V on C2, not
  the original's lowest F (board2.md B2-8; calibration change 11).
- **Again after session J's fits** (KEYBOARD CONTROL's R53 and R54, the contours' laws,
  AMOUNT OF CONTOUR, EMPHASIS, GLIDE, the MODULATION wheel and the contours' first decay
  sample): Cruising Whistle -1.765 to -1.693 (its whistle at C4, G4 and E4 +18, -23 and +5
  cents from 0.1.3's, their mean 0), Ladder Kick -3.637 to -3.615 (its boom, 50 to 120 ms,
  within 0.2 cents of 0.1.3's on average; its sweep starts about 2 semitones lower, from
  AMOUNT OF CONTOUR's law, and its ring decays faster, from EMPHASIS's: positions kept, as
  for every other preset); Bass 9.68, Lead 8.23, Three Saw Slab 8.23, Pulse Strut 8.88,
  Undertow Growl 8.44, Slow Horn Swell 8.1, Wooden Mallet 9.91, Noise Crash 9.87, Warped
  Pad 7.62. And after the contours' peak (calibration change 19; the two pitched presets
  unmoved: Ladder Kick's boom -0.7 cents from 0.1.3's): Bass 9.7, Three Saw Slab 8.29,
  Pulse Strut 8.99, Undertow Growl 8.48, Noise Crash 9.89: every preset VOLUME can reach
  within 0.09 dB of 0.1.3. At 10 and quieter: Upright Pluck -3.31 dB, Breath Flute -2.15,
  Stacked Fifths -2.09, Ladder Kick -2.06, Ringing Saw Line -1.55, Open Hat -1.50,
  Shoreline Wash -0.90, Noise Snare -0.87, Closed Hat -0.11; Pink Riser +1.84, still under
  -18 LUFS.
- **Again after session L's fits** (AMOUNT OF CONTOUR in true volts, the contours' peak
  diodes, LO, MOD DEPTH's law, the mixer's noise): Ladder Kick -3.615 to -3.604 (its boom
  +0.3 cents from 0.1.3's), Cruising Whistle unmoved (its three notes +1.8 cents on
  average); Three Saw Slab 8.24, Pulse Strut 9.04, Cruising Whistle 7.12, Brass Tutti 4.2,
  Wooden Mallet 9.88, Noise Crash 9.84: every preset VOLUME can reach within 0.07 dB of
  0.1.3. At 10 and quieter: Upright Pluck -3.00 dB, Breath Flute -1.85, Ladder Kick -1.82,
  Stacked Fifths -1.76, Ringing Saw Line -1.24, Open Hat -1.12, Shoreline Wash -0.58, Noise
  Snare -0.52; quiet by nature and left at 10, Closed Hat +0.30 and Pink Riser +1.76.
- **Again after the contours' capacitors as electrolytics** (the owner's choice of the full
  model, 2026-10-09; calibration change 25): Cruising Whistle and Ladder Kick unmoved (+1.8
  and -0.9 cents); Three Saw Slab 8.32, Pulse Strut 9.07, Undertow Growl 8.55, Cruising
  Whistle 7.16, Wooden Mallet 9.91, Noise Crash 9.89: every preset VOLUME can reach within
  0.09 dB of 0.1.3. At 10 and quieter: Upright Pluck -3.49 dB, Ladder Kick -2.37, Stacked
  Fifths -2.15, Breath Flute -1.85, Open Hat -1.63, Ringing Saw Line -1.54, Noise Snare
  -1.08, Shoreline Wash -0.60, Closed Hat -0.23; Pink Riser +1.83.
- **Again after SUSTAIN's law** (2026-10-09; calibration change 26): twelve presets moved
  0.01 to 0.23 dB; Lead 8.2, Pulse Strut 9.1, Cruising Whistle 7.11 (its whistle unmoved:
  AMOUNT OF CONTOUR 0): every preset VOLUME can reach within 0.02 dB of 0.1.3. At 10 and
  quieter: Upright Pluck -3.50 dB, Ladder Kick -2.37, Stacked Fifths -2.33, Breath Flute
  -1.75, Open Hat -1.63, Ringing Saw Line -1.54, Noise Snare -1.08, Shoreline Wash -0.60,
  Closed Hat -0.23; Pink Riser +1.83.
- **Again after the contours' release at DECAY 0 and Q12's gain** (2026-10-09; calibration
  changes 27 and 28): the first changes no preset; after the second every level within 0.01
  dB but Undertow Growl's chaotic loop (-0.12): Undertow Growl 8.58. Every preset VOLUME can
  reach within 0.02 dB of 0.1.3; at 10 as above.
- **Again after EMPHASIS at 2, 3 and 4** (2026-10-09; calibration change 30): the presets
  with EMPHASIS from 2 to 4 more resonant, their passbands lower: Bass 9.94, Lead 8.66,
  Warped Pad 7.52, Wooden Mallet 10 (0.09 dB short). At 10 and quieter than 0.1.3: Upright
  Pluck -5.59 dB, Breath Flute -3.77, Ladder Kick -2.37, Stacked Fifths -2.34, Open Hat
  -1.63, Ringing Saw Line -1.54, Shoreline Wash -0.60, Noise Snare -0.60, Closed Hat -0.23;
  Pink Riser +1.81.
- **Again after the VCA's R43** (2026-10-09; calibration change 31): every preset 0.2 to
  0.6 dB louder. Bass 9.88, Lead 8.57, Three Saw Slab 8.23, Elastic Octaves 5.43, Pulse
  Strut 9.02, Undertow Growl 8.45, Hollow Glider 6.07, Cruising Whistle 7.02, Slow Horn Swell
  8, Brass Tutti 4.12, Wooden Mallet 9.96, Slow Bow 8.29, Closed Hat 9.96, Noise Crash 9.82,
  Warped Pad 7.44: each within 0.02 dB of 0.1.3. At 10 and quieter than 0.1.3: Upright
  Pluck -5.16 dB, Breath Flute -3.41, Ladder Kick -1.95, Stacked Fifths -1.88, Open Hat
  -1.16, Ringing Saw Line -1.12, Shoreline Wash -0.23, Noise Snare -0.16; Pink Riser +2.13
  (left at 10, as before).
- **Again after the VCA's balance trims** (2026-10-09; calibration change 32): Elastic
  Octaves 5.5, Pulse Strut 9, Hollow Glider 6.04, Cruising Whistle 6.99, Wooden Mallet 9.94,
  Closed Hat 9.94, Noise Crash 9.8; the rest moved under 0.1 dB. At 10 and quieter than
  0.1.3: Upright Pluck -5.05 dB, Breath Flute -3.30, Ladder Kick -1.84, Stacked Fifths
  -1.81, Ringing Saw Line -1.08, Open Hat -1.05, Shoreline Wash -0.11; Pink Riser +2.26.
- **Owner decision, 2026-10-09: Ringing Saw Line's oscillator 2 at +3.4 cents** (FREQUENCY
  0.026). At FREQUENCY 0 the CA-72's two oscillators sit exactly an octave apart and their
  phase stays where the octave adds up: 3.6 dB over the 32' fundamental on every note, the
  line heard an octave up ("it sounds like we're missing an entire bottom octave"). On
  session I's take the reference's oscillator 2 sat 3.4 cents sharp (from the pair's phase
  rolling within each note), its octave swinging from 4 dB over the fundamental to 6 dB
  under; with the same detune the CA-72's swings over the same range. The owner: "Much
  better!", "save that as the new ringing saw preset". It plays 0.45 dB quieter (at 10:
  -1.53 dB against 0.1.3). The other presets whose oscillators stand at unison or an octave
  with FREQUENCY at 0 lock the same way (Three Saw Slab, Pulse Strut, Undertow Growl, Hollow
  Glider, Brass Tutti, Slow Bow): detuned as a panel set by ear, each on the owner's
  hearing.
- **Owner decision, 2026-10-09: those six detuned** ("slightly detuned makes everything
  better", after a page of each as it was and detuned). Within the reference's own range
  (its knobs at 0, set by eye, landed 7 and 26 cents flat in session N, 3.4 sharp on
  Ringing Saw Line's take): Three Saw Slab, Pulse Strut, Undertow Growl and Brass Tutti with
  oscillator 2 at +3.4 cents (FREQUENCY 0.026) and oscillator 3 at -7 (-0.053), Slow Bow's
  oscillator 2 at +3.4, Hollow Glider's oscillator 3 at -16 (-0.1215, as on its take 31).
  Re-levelled: Three Saw Slab 8.54 (1.2 dB quieter detuned), Brass Tutti 4.22, Pulse Strut
  8.96, Undertow Growl 8.4, Slow Bow 8.23; Hollow Glider within 0.02 dB.
- **The presets' oscillators 2 and 3 FREQUENCY values rewritten after their laws**
  (2026-10-09; calibration change 29): every value but 0 moved to where the new law puts
  the pot where the old value did (Stacked Fifths 5 and 7 to 4.1667 and 6.4465, Breath
  Flute's oscillator 3 4 to 3.1678, the detunes 0.1 to 0.0833, and so on): their intervals
  and oscillator 3's rates as they were, within 0.005 cent; FREQUENCY 0, the unison, is
  unmoved. A pitch is a preset's design, as the two retuned filter pitches were.
- **MIDI's modulation wheel follows the reference's curve** (agent decision, 2026-10-08,
  under the owner's decision to match the reference): control change 1 puts the
  MODULATION wheel where the reference's resistance is for it (`modulation::midi_wheel`);
  the wheel's own law stays the drawing's, to the reference's 685 ohm fully forward
  (calibration change 18). The reference's manual makes the curve a MIDI setting, so the
  measured curve is its MIDI handling's, not its MOD DEPTH pot's; presets keep their wheel
  positions.
- **Again after the keyboard's 0 V on C2:** Cruising Whistle -1.403 to -1.765 (its whistle
  within 4 cents of 0.1.3's), Ladder Kick -3.504 to -3.637 (its ring as 0.1.3's); Bass 9.71,
  Elastic Octaves 5.48, Pulse Strut 9.16, Undertow Growl 8.71, Hollow Glider 6.16, Slow Horn
  Swell 8.13, Brass Tutti 4.14, Warped Pad 7.58: every preset VOLUME can reach within 0.05
  dB of 0.1.3. At 10 and quieter: Upright Pluck -3.64 dB, Breath Flute -2.31, Stacked Fifths
  -2.01, Ladder Kick -1.63, Open Hat -1.59, Ringing Saw Line -1.52, Wooden Mallet -0.82,
  Noise Snare -0.81, Shoreline Wash -0.44; Pink Riser +1.44, still under -18 LUFS.

## R43. White Lotus, a held bass fitted to a recording, as a factory preset
**Owner decision, 2026-10-09:** after hearing a preset fitted to the synthesizer bass of a
studio recording, the owner asked for it to join the factory presets as White Lotus, and
chose it without feedback.

**Agent decisions, 2026-10-09** (not separately approved):
- **White Lotus, the 25th preset:** one narrow rectangle at 8' (mixer 10, the other
  oscillators off), CUTOFF -2.14 and EMPHASIS 7 with both KEYBOARD CONTROL switches on, so
  the filter rings on each note's third harmonic (on D2 the A3 stands 5.9 dB over the
  fundamental; on the recording's first note, 5.5 dB); oscillator 3 at LO (FREQUENCY 7.45,
  11.3 Hz; OSC. 3 CONTROL off, out of the mixer) into FILTER MODULATION at MOD wheel 0.17;
  LOUDNESS ATTACK 0.5, DECAY 3.5 and SUSTAIN 8.5 with DECAY on; no glide.
- **Fitted by measurement** to the recording's first 52 s, where the part is long held notes
  (D2, C2, G2 and B2, each after an F2 on the fourth beat) under little else. The bass was
  separated with a source-separation model, but each note's harmonics were read from the
  full mix in windows locked to them, since the separation put some of the synthesizer's
  upper harmonics in the other stem. The fit weighs H1 to H8, each note's relative to its
  strongest, toward the first five and toward D2 (the first note and the commonest): an
  average over many harmonics had hidden the ringing third the owner heard missing. The
  wobble is fitted to its sidebands (+-11.3 Hz: -21 dB around H2 and H3, -15.5 around H4,
  -29 around H1, none at half that rate), the note shapes to the stem's level around each
  start and end (a 1 dB lift settling over half a second, no dip between notes), and glide
  to the pitch track (without glide within half a semitone 99 % of the time; worse with
  any). One oscillator: the recording's harmonics show no beating, and a pair exactly in
  tune fitted only at one locked phase (the score went from 1.7 to 3.6 dB with 2 cents of
  drift).
- **On this version's engine, after R42's laws:** the panel fitted first on 0.1.3 (CUTOFF
  -1.6, OSC. 3 FREQUENCY 7.5, MOD wheel 0.025, SUSTAIN 9) rings on the fourth harmonic here.
  Fitted again: CUTOFF -2.14; OSC. 3 FREQUENCY 7.45 (LO now reaches 11.36 Hz); MOD wheel
  0.17, the sidebands the owner heard on 0.1.3's render (-15 dB around H3, -14 around H4);
  SUSTAIN 8.5 (9 now lifts the start 0.4 dB).
- **What differs from the recording:** H2 4 to 6 dB strong on D2 and G2 (the narrow
  rectangle's own); with full tracking every note rings on its third harmonic, where the
  recording's C2 and B2 ring less; the wobble deeper around H3 and H4 and shallower around
  H1 and H2. FEEDBACK through EXTERNAL INPUT made the fit worse at every amount tried (EXT 3
  to 5, FEEDBACK 2 to 3).
- **Levelled as the others:** MAIN OUTPUT VOLUME 9.78, -18.03 LUFS momentary maximum on
  `preset_levels.rs`'s bass phrase.
- The README counts twenty-five and names it. The DAW's copy of the file is not changed here.

**Evidence (omarchy, 2026-10-09):** the plug-in crate's tests in debug, 153 passed, 0 failed
(the factory file's count now 25; its text still names no trademark); `preset_levels.rs` by
hand for White Lotus, -18.03 LUFS momentary maximum at VOLUME 9.78.


## R44. The realistic look: the panel in the Model D's materials, and the CA-74's strip

**The owner's request, 2026-10-09:** "Give the CA-72's editor a realistic look and feel, as
the CA-74 (Highstead) got on 2026-10-08, but Moog-oriented: think of the Model D family the
CA-72 is calibrated against, its materials and conventions, not its trade dress (no trademarks
or logos anywhere)." The panel: "Keep every control's place, size and legend; only the surfaces
become pictures." The plug-in's own controls: "a strip that is always there below the panel,
fixed in position (not a drawer that opens and closes) ... push buttons that light for the
choices, slide or rotary controls with readouts, and the presets in a rail with a display and a
list that drops down over the strip inside the window."

**Through the mock-ups, 2026-10-09:**
- "you need to research a bit more what the classic moog aesthetic/knobs were like. Especially
  for the Model D". The look follows the original's materials as published descriptions and
  parts listings give them and as Wikimedia Commons' "Minimoog panel.jpg" (a 1970s Minimoog,
  CC BY 2.0, the photograph this panel was measured from) shows them, only looked at. "the
  photographs taken were of my behrigner, not a model d. and as such they shouldn't be used for
  reference unless i specifically request them to be used": none were.
- "Also the new panel needs the low/high filter switch we just added": FILTER MODE (R46, on
  `cal/hp-mode`) is on the panel in the new look as in the drawn one.
- Of three directions (A, the panel carried down; B, a programmer after Moog's of 1982; C,
  aluminium modules), with A's buttons orange or blue where they switch something on: "blue and
  orange. Let's go option a.", then "sorry i meant only orange buttons. not blue/oprange": A,
  its buttons orange.
- "we need to add all the same stereo controls that we added to the CA74", then "in fact, i
  think all controls on that panel should probably be added here": the strip carries the
  CA-74's (its R25 and R27 to R31, R34, R36, R41): VOICES (MONO | POLY | UNISON, VOICES,
  ENTROPY), STEREO (SCATTER | DOUBLE, EVEN | EDGES | CENTER, WIDTH, DETUNE and the display of
  the voices), OUTPUT (DRIVE, AUTO GAIN, LEVEL).

**Agent decisions, 2026-10-09** (not separately approved):
- **The materials** (A): solid walnut, oiled; the face aluminium with a textured black finish,
  printed in white; Moog's modular knobs (a smooth flared black skirt with a white dot, a grip
  of broad flutes, a spun aluminium cap), OSC 2 and 3's the same knob larger; wedge pointer
  knobs with a cream line for RANGE and WAVEFORM; blue and orange rockers; filament lamps
  behind jewels (POWER red in a chrome bezel, OVERLOAD dark); chrome jacks; white ridged
  wheels. The strip is the panel's face carried down under the name board, with its white
  rules between sections and its titles along the foot; the push buttons are translucent
  caps of the orange rockers' plastic, lit from inside; the displays orange
  gas-discharge digits and dots, the instrument having no LEDs.
- **The pictures** were made by an image generator (Codex CLI 0.160.1) from words alone and
  cut out of a grey ground as the CA-74's were; their prompts and account go with the assets.
  The generated originals, the logs and the mock-up are kept on the lab's share
  (`ca-72/look`).
- **Where:** a branch of its own from `main` (8ff1455). FILTER MODE comes from the drawing
  (the controls and the print are the drawing's), so the look carries it once `cal/hp-mode`
  is merged; each step is tried against a local merge of the two.
- **The panel, built (`skin.rs`, `art::control_worn`, `Renderer::with_skin`):** the editor
  draws it worn; the drawn panel stays as the drawing the worn one is printed and placed from,
  and its approval test (`approved.png`) still guards every place, size and legend. The print is
  the drawing's own (`art::printed`), laid over the face's picture; each knob's picture turns
  about its cap's axis and its cap is drawn over it unturned, so its sheen stays where the lamp
  is; a rocker's picture is mirrored to the end pressed; the knobs' and switches' shadows are
  the background's, soft, down and to the right; FEEDBACK, dimmed in the drawing while EXTERNAL
  INPUT is off, is shaded instead (a picture is not dimmed). The pictures, about 3.8 MB in the
  plug-in, and how they were made: `crates/ca72-panel/assets/worn/README.md`.
- **"You can do better. The 74 did better."** (the owner, 2026-10-09, of the first pictures).
  Side by side at one size the CA-74's parts stood off its face and ours lay flat on it. Now: a
  tall knob casts a long soft shadow down and to the right and a dark one where it stands; the
  lamp lights its black skirt's near side and darkens its far side (a colour dodge to 1 / (1 -
  0.45) and a multiply to 0.35 at its edges, the CA-74's), not its cap, whose spun sheen is
  turned to lie along the line to the lamp; the face takes the lamp's light, up to 1.26 of
  itself up and to the left and 0.84 at the far corner, so its texture catches a sheen; the print
  is worn into the face, its texture showing through it; each rocker's paddle is lit by its
  shape (the pressed half low, the raised half rising in a hump that rounds over at its end, its
  sides rounded) and casts its own shadow, longer from the raised end; and, asked "do you think
  we should make the pitch and mod wheel more realistic too?", the wheels are white ridged
  cylinders lit by the lamp, their ridges and PITCH's line or MOD.'s dot rolling as they turn.
- **"The switches need to be a touch more obvious as to which side is up and which is down."**
  (the owner, 2026-10-09). Tried and dropped: a rocker's pressed half a shade darker (0.86 of
  its light), a shadowed crease where its raised half begins (0.7 at the pivot, gone 3 units
  up), the raised half's hump higher (0.38 of the paddle's width, was 0.32). The owner: "No, the
  new thing you just did is worse. Go back.", and of the pictures compared: "Top left was still
  the best" (the rockers before it, close up). The rockers are as they were before it.
- **Cost (release build, the Linux reference machine, another agent's benchmark on four of its
  cores):** the background once a scale, 182 ms at 0.4 of the drawing (the pictures decoded the
  first time) and 508 ms at 1; a knob turned, 1.5 ms at 0.4 and 4.8 ms at 1.
- **The strip chosen: A6** (the browser mock-up, 2026-10-09, its version 14; the sources on the
  lab's share, `ca-72/look/mock`). Of the strip in three rows (A5): "this screen is much too
  large. Needs to be the same height as the buttons flanking it"; the presets' screen is now as
  tall as the keys beside it, its surround included (62 units; its dots 6 apart, were 9). Then:
  "can you do one more mock up? This one will move the keyboard mod/pitch controls back down to
  the lower area to take advantage of using more height and less width for the plugin to allow
  the top panel to be a little larger compared to the bottom", the plug-in's left edge "the red
  line" at the column's right; and, shown A6: "yup, this is the one. A6 looks good. Also, no need
  to put a back plate on top of another backplate here" (the left hand's controls now straight
  on the strip's face). So:
  - the panel's controller column (GLIDE and DECAY and their jacks, the PITCH and MOD. wheels)
    leaves the panel for the strip's left, laid out as the column had them; the window starts
    at the panel's face, 3108 units of the drawing wide instead of 3438, as tall as A5's;
  - the strip in three rows on A5's grid: banks of lit tabs on the first (MODE: MONO, POLY,
    UNISON; VOICES PLAYED AS: SCATTER, DOUBLE; PLACEMENT: EVEN, EDGES, CENTER; AUTO GAIN's ON),
    then knobs with readouts (VOICES and ENTROPY; WIDTH and DETUNE over the voices' display,
    WHERE THE VOICES SOUND; DRIVE and LEVEL, AUTO GAIN's correction in dB), sections VOICES,
    STEREO and OUTPUT; the presets' rail in walnut above it (favourite, previous, the screen,
    next, SAVE), the list dropping down over the strip;
  - what it gains (told the owner): where the window's width decides, the panel is 11 % larger
    at the same width; on a 16:9 screen its height decides already, and it opens the same size
    as before in a window a tenth narrower (on 1920 by 1080, about 1356 pixels wide, was 1500).
- **A6 built (2026-10-09 and 10).** The drawing is one: the panel, 3108 by 1057 units, and the
  strip below it, 850 units, 3108 by 1907 in all (`art::PANEL_H`, `art::STRIP_H`). What turns
  is the drawing's: the left hand's controls and the strip's six knobs (VOICES, ENTROPY, WIDTH,
  DETUNE, DRIVE, LEVEL) are among its controls (now 50), drawn, turned, learned and found as
  the panel's are. The strip's own parts (the tabs, the readouts, the voices' display, the
  rail's keys and the preset's name) are drawn over the drawing's strip by
  `strip::StripRenderer`, in the CA-74's materials, which moved into the shared kit
  (`plugin-kit-materials`, its K5; the CA-74 switched to it on its `kit-materials` branch):
  the tabs translucent amber caps lit from inside, their light on the face round them; the
  readouts orange seven-segment digits behind smoked glass; the drops of the voices; the name
  in orange dots. Two pictures came with them from the CA-74, the keys' cap and the displays'
  glass (`crates/ca72-panel/assets/worn/README.md`).
  - The window opens at the drawing's proportions and never grows: the presets' list drops
    down from under the rail over the strip, seven rows tall (was eight; the panel stays in
    sight).
  - The approval test (`tests/approved.rs`) compares the panel with the approved mock-up right
    of the mock-up's column. The wood's grain is laid out as it was with the column there, so
    the panel's wood is unchanged; the mean difference is 1.38, as before A6 (1.38).
  - **Cost** (`cargo run --release -p ca72-panel --example frames`: a frame, the panel's
    renderer, the strip's, the editor's putting them together and the conversion for the
    window, for each kind of change). As first built, every change anywhere drew the strip's
    parts again whole, and so did each frame while the voices' drops moved: on the Linux
    reference machine (another agent's benchmark busy on four of its cores) 5 ms a frame at 0.44
    of the drawing (the window on a 1920 by 1080 screen) and 19 ms at 0.87 (the same on a Retina
    screen), all the time notes were played. Most of it was tiny-skia laying two pictures the
    strip's size, mostly clear, over every pixel of it. Now:
    - those two are laid only where they have pixels (`Sparse`);
    - the panel's renderer puts its frame together again only inside the rectangle its layers
      changed (`Renderer::damage`), each pixel as when it is put together whole;
    - the strip keeps the drawing's strip with what never changes over it, brings it up to the
      drawing only where that changed, and draws again only the rectangle that changed (where
      the drawing changed under it, a tab, a readout, the rail, the pointer, MIDI Learn's ring,
      the drops' window while they move), grown until every part reaching into it lies inside
      it, each part being drawn whole;
    - the editor puts its frame together again only in the rows that changed (all of them
      while the drawer slides or something floats over everything).

    On the owner's Mac (Apple silicon), the median of 40 frames, ms: with the first of these
    changes (the commit that built A6: the two pictures laid where they have pixels, the drops'
    window drawn alone, the strip left alone for a knob above it), then with all of them:

    | Change | first: 0.44 | 0.87 | 1.0 | now: 0.44 | 0.87 | 1.0 |
    |---|---|---|---|---|---|---|
    | a panel knob (CUTOFF) | 1.0 | 3.8 | 5.0 | 0.5 | 1.6 | 2.1 |
    | a strip knob (DRIVE, its readout) | 2.5 | 8.3 | 8.6 | 1.2 | 4.1 | 3.1 |
    | the PITCH wheel | 1.8 | 6.1 | 8.1 | 0.5 | 1.9 | 2.6 |
    | a tab | 0.8 | 2.4 | 3.2 | 0.2 | 0.7 | 0.9 |
    | notes played (the drops moving) | 0.4 | 1.7 | 2.2 | 0.4 | 1.7 | 2.2 |
    | nothing | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |

    What is left is mostly a turned knob's own picture, a readout's digits (an SVG with a blur)
    and the drops. Each frame drawn in part is the frame drawn whole, to the bit: the panel's
    (`a_frame_put_together_in_part_is_the_frame_put_together_whole`), the strip's, through
    every kind of change and the drawing changing under its parts
    (`a_frame_drawn_in_parts_is_the_frame_drawn_whole`), and the editor's
    (`the_frame_put_together_in_rows_is_the_frame_put_together_whole`).
  - **In hosts (2026-10-10).** A renamed build, "CA-72 A6" (IDs of its own, so no host takes it
    for the installed CA-72), of this branch merged locally with `cal/hp-mode` (FILTER MODE,
    which the owner asked to have on the new panel) and `main`; built as the release is
    (`--profile bundle`).
    - **macOS 27.0 (the owner's Mac):** clap-validator (37 passed, 7 skipped), Steinberg's VST3
      validator (47 passed) and pluginval at strictness 10 with its editor tests (the editor
      opened, opened while processing, automated) all passed. Installed beside the released
      CA-72 in `~/Library/Audio/Plug-Ins` (VST3 and CLAP) for the owner to play in Ableton Live
      12.4 beta; not opened there by the agent (Live was open with the owner's sets).
    - **Windows 11, REAPER 7.82** (the CA-74's harness, `ca72-it\a6`): the editor opened at
      the drawing's proportions (2692 by 1652). Clicks on every tab, ENTROPY's readout, GLIDE,
      and the wheel over DRIVE, LEVEL, VOICES and the PITCH wheel set their parameters, each
      read back through REAPER's API; the presets' list dropped over the strip; notes lit the
      drops; and real pointer clicks (the cursor moved and pressed) on POLY, EDGES, AUTO GAIN
      and WIDTH's readout did the same. Two faults found and fixed: the wheel moved VOICES as a
      smooth knob, two notches short of the next voice (now a voice a notch); and with the list
      open while notes played, its rows over the voices' display came out a few pixels lower
      than the rest (the slide's last frame stops short of its end, and the frames since were
      put together in rows only; now a drawer whose place changed is put together whole,
      `the_drawer_ends_its_slide_in_the_frame_shown`).
    - **Windows 11, Cubase Pro 15.0.30** (the trial R36 used): an instrument track with CA-72
      A6; real clicks on MONO, POLY, DOUBLE, EDGES, AUTO GAIN, ENTROPY's readout, GLIDE and
      FILTER MODE, posted ones on POLY and UNISON, and a real double click on DETUNE (back to
      off, SCATTER lit), each shown by the editor (its tabs, readouts and tips). The first real click after the harness
      brought the window forward was lost: it does so with an Alt key, which leaves Cubase's
      window in menu mode; after a first click elsewhere every click took. Cubase closed
      without saving; the test builds recycled.

**The owner, playing the A6 build, 2026-10-10:**
- "the preset text needs to be vertically centered. Also, we need some sort of border or edge
  around that screen to help it feel more natural": the name's dots a sixth smaller (5 units
  apart), a capital in the window's middle (2 units above it) with the descenders inside it;
  and a bezel the worn skin never drew (only the drawn one had a flat surround): a dark frame
  standing proud of the rail's wood, its top edge catching the lamp, its shadow below, a black
  lip into the glass, as tall as the keys beside it (`strip::name_bezel`, both skins).
- "these orange buttons get too bright on the edges. looks unnatural": a lit tab's glow
  pressed down to a quarter over its knee (0.8), its outermost band (from 0.86 of the way out)
  up to 0.4 darker, and its light on the face round it 0.6 of what it was, one orange (the rim
  had been a hot yellow-white round the cap, brighter than its face).
- "all the knobs look like they have white halos. and why do the knobs always have 4 shiny
  places that have nothing to do with the light?": the knobs' pictures quieted
  (`assets/worn/quiet.py`, the README): the ring light they were made under had left broad
  reflections round the skirts and a bright line round each outline. Each skirt's light is
  the same all round now, each grip's to below its flutes, the outline held dark; the panel's
  lamp alone lights them. The evenness test (`the_knobs_pictures_are_lit_evenly_all_round`)
  holds: the edge between grip and skirt, off the cap's axis, is left as it was.
- "If we moved the CA-72 badge to the top, we could save a lot of space in that center wooden
  area.": the name plate is in the top strip now, its left end on the MODIFIERS|OUTPUT line as
  before, between the strip's screws; the name board under the face is gone, the presets'
  rail right under the face. The drawing is 3108 by 1775 (was 1907): the window 7 % shorter at
  a width, and on a wide screen it opens wider (a 2560 by 1440 monitor: 2017 wide, was 1878).
  The approved test compares the top strip and the face (the plate's place left out), and
  allows as many strongly different pixels (32 levels or more) as a hundredth of the approved
  panel's area with its board: over the smaller area the 99th percentile, which the board's
  closely matching wood had kept under 32, is 36 on the same face (the commit before, measured
  over the same rows: 1.457 and 36). On `potato` the last column's head (QUALITY, approved
  apart, R47) is left out too: 1.445 and 30745 of 32851.
- "I feel like the badge isn't quite in the right place though." / "is it on the same grid as
  everything else?" It was not: its left end 4 units right of the MODIFIERS|OUTPUT line (its
  place on the old board) and a touch above the strip's middle. Then: "maybe it's also partially
  cause it's not up to the same realistic quality as everything else. Let's fix that.": the
  plate a generated picture of a blank plate, brushed black anodised aluminium with a bright
  chamfer, proud of the wood with its shadow, its lettering paint filled into engraving
  (`art::plate_worn`, `assets/worn/README.md`); "dial back the lighting effect of the badge a
  bit": its picture at 80 %. Of three places, each rendered whole on the mock-up's page (as
  built; centred on OUTPUT, recommended; its left end on the divider line): "I think we can go
  with option C." Its left end on the line (2578), in the strip's middle.
- **The strip's knobs' legends and the lit tabs' corners** (the owner, 2026-10-10, from the
  build in Live: "these labels are too high above their knobs. also, is it just me, or should
  the corners of these buttons be darker, not lighter", "i'm just thinking that's where the
  plastic would be densist from the user's perspective"): the legends 109 units above their
  knobs, not 122, as far over the numeral at twelve o'clock as TUNE's over its own (27 pixels
  at a pixel a unit; they were 40); a lit tab's corners darker as well as its edge, half its
  light at the corner itself, the rim line round them too, easing in from 0.35 of the way out
  both ways (the light round the tab on the face, the kit's, is left as it is: it is not
  brighter at the corners). The window as it opens, for a look: `presets.rs`'s `window_png`.
  Then "go even darker in the corners": a quarter of its light at the corner itself, easing in
  from 0.3 of the way out. And "why does the overdrive light have a circle in the middle of
  it?": the jewel's picture (POWER's, OVERLOAD's made from it) had a ring light's reflection
  in its dome, a bright ring halfway out, which on OVERLOAD's dark glass read as a pink circle
  (as the knobs' ring-light reflections did); painted out of the picture
  (`assets/worn/README.md`), for both lamps.
- **The rail's star and the display's reds** (the owner, 2026-10-10, sending the mock-up's dot
  star and its voices' display: "turn this into a pixel star. like the one you have here. And
  go back to that red too. Same thing with the stereo bar. I want it to go back to the red
  pictured here."): the favourite's star before the name drawn in the name's dots (the
  mock-up's five by seven), as faint as the unlit dots when the preset is not a favourite; the
  name's dots in the mock-up's neon (255, 112, 40) with its deeper glow (255, 72, 10), not the
  hotter amber they had been given; and the voices' display the mock-up's drops again: flat
  discs where their fields sum past one, 25 units across for a sounding voice and 15 for an
  idle one, no white-hot middle, their light (172, 80, 30) added in so that under this
  display's glass a sounding drop measures (233, 130, 63) and an idle one (155, 89, 44) where
  the owner's picture has (225, 134, 63) and (159, 80, 38) (an idle one's light half a
  sounding one's share, the mock-up's a third, for the same).
  Then "make sure the arrow on the right side of the preset dropdown is also dots like this":
  the list's arrow in the name's dots too, the mock-up's glyphs (down while the list is shut,
  up while it is down), 0.7 of the name's light but under the pointer.
- **The presets' drawer, the mock-up's** (the owner, 2026-10-10: "i just realized we didn't
  update the preset search page to the new one that you had sketched out! let's fix that"; the
  drawn drawer, flat panels and buttons, had been left as it was): A6's dropdown, a tenth smaller
  as the mock-up drew it, on the strip's face under the rail (its picture placed as the strip's,
  its trim and the rail's shadow), in three sections in the panel's print, their displays dots
  behind glass as the rail's name (the mock-up's neon, every dot faintly there, the lit ones
  with their glow; `ca72-panel/src/presets/drawer.rs`):
  - FIND: the search on its own line (SEARCH, faintly, while empty), and the tags in use two to
    a line, each picked as a filter bright (sixteen lines show; the wheel scrolls them).
  - PRESETS: ALL, FAVORITES and MINE as tabs over the list (one at a time: the drawn drawer's
    two switches could both be on), then seventeen rows, each the current preset's mark, its
    star (outlined if not a favourite; a click on it makes it one), its name, EDITED or YOURS,
    its tags; arrows at the right where there are more above or below.
  - PRESET: eight keys, the rail's charcoal caps with their legends printed over them (RENAME,
    TAGS, DELETE, REVERT; SAVE AS, RESTORE, MIDI LEARN, CLOSE), over a display of the preset
    the plug-in is set to (its name; where it is from, starred if a favourite; its tags; what
    was last done; how many presets the list shows of how many) and the update check (R27) on
    its last lines.
  Agent decisions, not separately approved: the drawn drawer's per-row RENAME, TAGS, REVERT
  and DELETE became the PRESET keys, acting on the preset the plug-in is set to (the mock-up's
  keys have no row); what they begin stays with that preset as the list changes and ends when it
  is gone or another is set (R18's rule, which had followed a row). RENAME, TAGS and SAVE AS
  are typed into the PRESET display (SAVE AS's name and tags on two lines, NAME and TAGS before
  them), DELETE asks there (DELETE again, or Enter, deletes; Escape keeps); a key held down while
  what it began goes on, and the update check not shown meanwhile. The caret is a character
  display's: a line of dots under the character it is before, steady. MIDI LEARN shows MIDI
  Learn's list (R34) in the list's place (its keys and the controllers it will not learn where
  the tags are), LEARN, REMOVE and CANCEL as words at a row's end; anything of the presets'
  pressed shows them again. The glass is the rail's picture with its rows and columns evened out
  and its mesh softened: drawn out over displays many times its height, its faint lines had been
  hard bands across the list (the owner, 2026-10-10: "there's several harsh horizontal lines,
  please get rid of them").
  Its cost (measured at a Retina screen's scale, two pixels a unit, a release build, on the
  Linux machine): the drawn drawer drew whole for every change, 13 ms to open and about 11 ms
  for each change after (a row under the pointer, a letter typed); this one draws again only the
  lines of a display that changed, 2 ms for a row under the pointer or a letter typed, 4 ms for a
  key's prompt, 20 ms for the MIDI list in the list's place. What does not change (the face,
  print, bezels, glass and every unlit dot) takes about 0.1 s to make, on a thread of its own
  begun as the editor opens, so the drawer's first opening draws only its lit dots: 28 ms. It is
  shared by the editors open at the same scale.

## R45. The CA-74's stereo in the CA-72: SPREAD's law and places, the placement, UNISON, DOUBLE

**The owner, 2026-10-09:** "we need to add all the same stereo controls that we added to the
CA74", then "in fact, i think all controls on that panel should probably be added here", and,
asked whether SPREAD should change to the CA-74's (a constant-power law and the places from the
edges, wider and up to about 1.5 dB louder, changing projects saved with SPREAD) or keep the
CA-72's for projects saved before: "I want you to copy the same stereo algorithms we in ca-74".
So the CA-74's R27 (SPREAD's law and places, POLY going round its voices, a key played again
keeping its voice), R30 (the placement: EVEN, EDGES or CENTER), R28 and R41 (DOUBLE, its pairs
mirrored as far out as the placement puts each voice), as they are there.

**Agent decisions, 2026-10-09** (the first step: SPREAD's law and places, the placement, POLY's
choice of voice; not separately approved):
- **The pan law** is the CA-74's: constant power, √2 cos and √2 sin of the place's quarter turn,
  whole on both sides at the centre (`character::pan_gains`). The CA-72's before it, the DAW's
  instrument's, kept both sides whole until the far one faded, a voice in the centre twice the
  power of one at an edge.
- **The places** are SCATTER's placement's (`Placement`, the host's Scatter Placement, the last
  parameter, `placement`: `even`, `edges` or `centre`, shown EVEN, EDGES and CENTER; a session
  or preset saved before reads EVEN), among VOICES's voices: EVEN, evenly from edge to edge from
  the edges in; EDGES, from 100 to 60 % out; CENTER, the first in the centre and the rest by the
  golden ratio. The CA-72's places before them were the first voice in the centre, then the
  quarters and an eighth. The voices' mix carries the placement and VOICES to the workers
  (`Mix`, `MIX_LEN` 6).
- **One voice alone sits in the centre:** with POLY off the instrument's voice is not placed (it
  was in the centre before too, by its place).
- **POLY's choice of voice:** the voice that last played the key if it is free or letting go,
  else the free voice whose last note began longest ago, else the one let go longest ago, else
  the one held longest (was: the first free voice by its number, then the oldest let go, then the
  oldest held).
- **MIDI Learn:** SCATTER PLACEMENT is learnable as a selector (48 learnable controls: 23 knobs,
  8 selectors, 17 switches); it has no place on the panel until the strip is built.
- **The presets** that play POLY moved and are levelled again through MAIN OUTPUT VOLUME's law,
  as R42 levels them: Slow Horn Swell 8 to 7.94 (+0.22 dB from the choice of voice alone, its
  SPREAD 0), Brass Tutti 4.22 to 3.92 (+1.34 dB, SPREAD 50), Warped Pad 7.44 to 6.9 (+2.05 dB,
  SPREAD 80): each one's momentary maximum on its phrase within 0.01 dB of before. The DAW's copy
  of the factory file now differs in these three volumes, and its instrument keeps its own law
  and places.

**Evidence (the Linux reference machine, 2026-10-09):** the CA-74's tests ported:
`every_place_is_as_loud_as_the_centre`, `even_fills_the_field_evenly_from_its_edges`,
`edges_fills_the_field_from_its_edges`, `centre_starts_in_the_centre_and_spreads_by_the_golden_ratio`,
`a_placement_goes_by_its_index_and_back`, `a_chord_fills_the_field_and_one_voice_stays_in_the_centre`,
`poly_notes_go_round_the_voices`, `a_key_played_again_keeps_its_voice`; with the old law and
choice of voice the first, the chord's and the round's fail, and without the key kept the last
fails (the chord struck again on voices 3, 4 and 5); the learn list's (48). `preset_levels` (each
preset as set) against the commit before: 21 presets the same to the hundredth of a dB, the
three above levelled back within 0.01 dB. `preset_render` (every preset with POLY's ten voices)
against the commit before: Pink Riser the same to the bit, the rest moved by the new choice of
voice (levels within 0.4 dB) but those with SPREAD (Warped Pad +1.38 dB, Wooden Mallet +1.18,
before levelling). The plug-in's unit tests (127 passed, 7 ignored); rustfmt; clippy.

**Agent decisions, 2026-10-09** (the second step: UNISON; not separately approved):
- **UNISON** (the host's Unison, `unison`, off by default; a session or preset saved before
  reads off) is the CA-74's R25 on this instrument: VOICES whole instruments, each taking every
  key through its own keyboard circuit as POLY off's one instrument does (lowest-note priority,
  single triggering), each with its own parts (ENTROPY's tolerances) and its place in the field
  by SPREAD and the placement, the output turned down by the square root of VOICES. It
  outranks POLY. Switching it, or VOICES while it is on, lets every key go.
- **Out of step:** every voice but the first, when it is made or put back to rest, starts its
  oscillators where its seed puts them on their ramps (`Vco::start_at`); the first starts at
  the reference's initial condition, so one voice, and every preset with POLY and UNISON off,
  renders as before to the bit. Without it UNISON's voices at LOCK would sound as one voice
  9 dB louder.
- **A voice's three oscillators start together**, at one point, as the reference starts them
  together at the top. The CA-74 starts each of its two apart, but here LOCK's trim (R9) is for
  three started together: started apart, the four-voice chord at LOCK came out 3.4 dB under the
  drifting one, and each voice would have had its own level at LOCK.
- **A seed loaded into a running instance makes its voices again**, as an instance opened with
  it makes them (the CA-74 does the same), so their oscillators start where the new seed says
  (R18's test caught it).
- **MIDI Learn:** UNISON is learnable as a switch (49 learnable controls: 23 knobs,
  8 selectors, 18 switches).
- **The presets that play POLY:** their voices past the first now start out of step, so a
  chord's onset no longer adds up in phase. They are levelled back through MAIN OUTPUT VOLUME's
  law by their momentary maximum, as before: Slow Horn Swell 7.94 to 8.011 (+0.27 dB), Brass
  Tutti 3.92 to 3.894 (-0.12 dB), Warped Pad 6.9 to 7.141 (+0.91 dB). Their integrated
  loudness on the phrase moves +0.26, -0.19 and +0.89 dB.

**Evidence (the Linux reference machine, 2026-10-09):** the CA-74's tests ported:
`unison_voices_come_in_out_of_step` (eight voices at LOCK within 2 dB of one; with every voice
started in step it fails at +9.0 dB), `unison_is_about_as_loud_as_one_voice`,
`unison_voices_take_the_keys_as_the_instrument_does` (every voice's keyboard voltage the one
instrument's after each of eight key events, every voice sounding),
`switching_unison_lets_every_key_go`. `tests/sound.rs` passes: LOCK +0.19 dB against
drifting (`lock_is_the_circuit_at_the_drifting_level`), and a reseeded instance plays as one
opened with the seed. `preset_levels` against the first step: 21 presets the same to the
hundredth of a dB, the three above levelled back within 0.005 dB. `preset_render` (every
preset with POLY's ten voices) against the first step: the noise presets the same to the bit
or within -145 dB, the rest moved by the voices' starts (levels within 0.71 dB; Warped Pad
+0.99 dB after levelling). The plug-in's unit tests (131 passed, 7 ignored); rustfmt; clippy.

**Agent decisions, 2026-10-09** (the third step: DOUBLE; not separately approved):
- **DOUBLE** (the host's Double, `double`, 0 to 100 %, off at 0, the last parameter; a session
  or preset saved before reads 0; saved in presets, as SPREAD is) is the CA-74's R28 and R41:
  each note two voices, the voice half the detune flat and a twin half sharp, up to 20 cents
  apart at 100 % (`DOUBLE_CENTS`); the pair about the centre, the flat voice to the right and
  its sharp twin as far to the left, SPREAD's way out as far as the placement puts the voice
  (EDGES: every pair to the edges); with POLY and UNISON off the one voice's pair as far out
  as SPREAD whatever the placement; the output turned down by two's square root. Turning
  DOUBLE on or off lets every note go; turned off, a twin still sounding plays its tail out on
  its side until silent.
- **Where the CA-72's differs:** an amount without a switch, as ENTROPY and SPREAD are here
  (the CA-74's has one). The detune is the oscillators': half of it added to each of the three
  oscillators' ENTROPY offsets (`Jacks::detune`), the filter's keyboard tracking left as it is,
  so oscillator 3 as a low-frequency source moves by the same few cents. The twin is a
  place's past the voices' (its seed, noise, tolerances and oscillators' start) with a
  keyboard of its own: in POLY its own key; with POLY off and in UNISON taking the keys
  straight, but those pressed only while DOUBLE is on, so a twin let go takes no new key (the
  CA-74's, let go, would sound with a new note on its voice's place).
- **A voice's place glides** (10 ms, the CA-74's `GLIDE`, R27): moved by SPREAD, the
  placement, VOICES or DOUBLE, it glides instead of stepping (switching DOUBLE off moves each
  releasing voice from its pair's place to its own). Constant, it is the same to the bit.
- **MIDI Learn:** DOUBLE is learnable as a knob (50 learnable controls: 24 knobs, 8 selectors,
  18 switches).
- **Memory:** each voice's place holds its twin, and each place's spare a clean twin, boxed:
  a voice is 34 KB, and unboxed the ten spares overflowed a test thread's 2 MB stack in a
  debug build.

**Cost (`preset_cost`, now with `CA72_DOUBLE`; the Linux reference machine, a Ryzen 7 7800X3D
shared with other work, load average 5 to 8, so single figures varied by up to three times
between runs):** about twice the voices' work, as twice the voices. Ten notes of POLY,
256-frame blocks at 48 kHz, Bass (the steadiest): one thread 80 % of a core off, 160 to 199 %
with DOUBLE at 35 %; with three workers 24 % off, 49 % on. The README's limitations say so.

**Evidence (the Linux reference machine, 2026-10-09):** the CA-74's tests ported:
`double_puts_the_sharp_twin_left_and_the_flat_voice_right` (+7.0 cents at 35 %; with the
voice sharp too, 0.0, and fails), `double_is_about_as_loud_and_stays_in_the_centre_without_spread`
(without the trim +3.3 dB, and fails), `double_takes_the_placement_its_pairs_mirrored` (with
every pair at the edges, CENTER's first fails), `doubled_voices_on_the_workers_are_the_same_to_the_bit`;
the CA-72's own `a_twin_let_go_plays_its_tail_out_and_takes_no_new_key` (with the twin taking
every key, it fails holding the new key). `preset_render` (every preset with POLY's ten
voices) against UNISON's commit: every preset the same to the bit; `preset_levels` the same.
The learn list's (50). The plug-in's unit tests (136 passed, 7 ignored), the plug-in's and the
model's integration tests; rustfmt; clippy.

**Agent decisions, 2026-10-09** (the fourth step: DRIVE, LEVEL and the host's names; not
separately approved):
- **DRIVE** (the host's Drive, `drive`, 0 to 24 dB, off at 0; after DOUBLE, saved in presets,
  learnable as a knob) is the CA-74's R29 DRIVE on this circuit: the mixer's signal raised into
  the filter's input pair (Q29 and Q30) after C27. The pair sees DRIVE's gain times the drop
  the bus's current makes across R54 (its own base currents' drop with it); C27's current, the
  bus's load and the bias chain (b5) stay the circuit's (`ca72::vcf::Drive::gain`). At 0 it is
  the circuit to the bit. A voice and its twin take it at each run. The model's threaded voice
  (the lab's) does not.
- **What it does here:** a held note grows louder, by less the higher DRIVE goes, as the input
  pair saturates (oscillator 1's sawtooth alone: +2.7, +5.1, +8.5 and +10.7 dB at 3, 6, 12 and
  24 dB); and darker, not brighter: the ladder's stages, slew-limited by their currents, take
  the highs the clipping adds (a triangle's energy over 1 kHz against under it -1.0, -3.3 and
  -10.2 dB at 3, 6 and 12 dB; +15.5 at 24, where it is nearly a square).
- **No thump promised:** raising DRIVE quickly on an asymmetric wave (the narrow rectangle)
  steps the output under 20 Hz to about twelve times its held level there, whether the gain is
  after C27 or before it (0.41 against 0.44): the saturating pair rectifies, its DC moving with
  DRIVE, as turning a mixer VOLUME up quickly does. The CA-74's reason for after the capacitors
  does not carry over; after C27 is kept so that DRIVE leaves C27's charge the circuit's. A test
  of no thump was written, failed both ways, and was dropped.
- **LEVEL** (the host's Level, `level`, -30 to +12 dB, 0 by default; after DRIVE, saved in
  presets, learnable as a knob) is the CA-74's: the output's gain after MAIN OUTPUT's, the
  plug-in's own; it steps as MAIN OUTPUT VOLUME's gain does (a learned knob glides, R34).
- **The host's names, the CA-74's (R31):** SPREAD is shown as Width, and DOUBLE as Double
  Detune, read in cents ("12 cents", Off at 0) and taken back so; their ids are as they were.
  MIDI Learn's list says WIDTH and DETUNE (DOUBLE) (52 learnable controls: 26 knobs,
  8 selectors, 18 switches); the strip as drawn says SPREAD until it is drawn again.
- **AUTO GAIN is the next step:** until then DRIVE makes the sound louder.

**Cost:** DRIVE's settings measured alike within the machine's noise (Bass, ten POLY voices,
one thread: 85 to 124 % of a core at 0, 12 and 24 dB, three runs each, interleaved, load
average 10 to 16): Potato's filter takes at most two Newton iterations a step whatever it is
driven by.

**Evidence (the Linux reference machine, 2026-10-09):** tests: `drive_drives_the_filter_harder`
(with the gain kept from the pair it reads +0.0 dB and fails), `level_moves_the_output_by_its_decibels`,
the CA-74's `double_reads_in_cents_and_spread_is_width`, and `filter_jacobian_matches_finite_differences`
at DRIVE's 24 dB too. `preset_render` against UNISON's commit: every preset the same to the
bit; `preset_levels` the same. The learn list's (52). The plug-in's unit tests (139 passed,
7 ignored), the plug-in's and the model's integration tests; rustfmt; clippy.

**Agent decisions, 2026-10-09** (the fifth step: AUTO GAIN; not separately approved):
- **AUTO GAIN** (the host's Auto Gain, `auto_gain`, on by default, the last parameter: a
  session saved before reads it on; saved in presets; learnable as a switch, 53 learnable
  controls, 19 switches) is the CA-74's R29 (`drive.rs`, ported): the output turned down by as
  much as DRIVE made the sound louder, from a curve measured for the sound. The plug-in plays
  three notes together (C2, G3 and C5, held 0.3 s, heard for 0.45 s) on three voices made as
  an engine's first three are (a seed of their own), without DRIVE and at 6, 12, 18 and 24 dB,
  and compares their K-weighted energy: four corrections, drawn straight between. The curve is
  saved with the session (`drive_curve`); a session saved before, or one not understood,
  opens with the factory presets' average (`Curve::AVERAGE`, their mean: -4.74, -8.43, -10.95
  and -12.42 dB). It is measured only when the sound is changed in the plug-in's own window
  (after a press, release, wheel or key there, once nothing is held, and as the window opens),
  never on the host's automation or a learned controller, so that a render never depends on
  when a measurement finished. The audio thread reads the curve without a lock, once a block.
- **Where the CA-72's differs:** the sound's key is its panel, LOCK, ENTROPY and FEEDBACK (the
  voice's output into its own external input changes how DRIVE loads it); not POLY, UNISON,
  VOICES, DOUBLE or SPREAD, as on the CA-74. The measurement runs on the plug-in's own helper
  (R23), a render at a time, the audio thread's asks (mending the spares, the workers) seen to
  between renders, so one of those may wait a render (about a tenth of a second); dropping the
  plug-in waits out a render under way. Only AUTO GAIN's correction glides (10 ms, at the
  host's rate): MAIN OUTPUT's and the other gains step as before, so that every preset plays as
  before to the bit.
- **The CA-72's DRIVE needs measuring per sound more than the CA-74's did:** the presets'
  curves at 24 dB run from 0 (Cruising Whistle: its whistle is the filter's own, with no mixer
  signal for DRIVE to raise) to -21.5 dB (Breath Flute), and Undertow Growl's turns back
  (-5.2, -5.3, -4.9 and -3.9 dB).

**Accuracy (`tests/drive_auto.rs`, by hand; the 24 presets, DRIVE every 3 dB from 3 to 24,
AUTO GAIN on, dB left over against DRIVE off):**
- With each preset's own curve, on its levelling phrase (momentary maximum): 0.50 dB RMS from
  3 to 12 dB (5 of 96 over 1 dB, the worst 2.7), 1.05 from 15 to 24 (18 of 96; the worst 3.5,
  Pink Riser, left quieter); on four held chords (integrated): 0.44 and 0.94 dB RMS (the worst
  4.4, Undertow Growl, quieter). The CA-74's own were 0.17 and 1.34 dB RMS on its phrases.
- With the average's curve standing in: 2.05 and 2.07 dB RMS from 3 to 12 dB, 3.44 and 3.72 over
  all of DRIVE (the worst 12.4, Cruising Whistle, taken down for nothing); with the CA-74's
  average before the CA-72's was measured, 5.3 dB RMS.

**Cost (the Linux reference machine, a Ryzen 7 7800X3D shared with other work):** a
measurement on one thread, a median 668 to 904 ms over two runs, the longest 1.7 s (Bass), on
the helper thread, which is not a real-time one (the CA-74's, 355 ms). Beside the voices
(`a_measurement_beside_the_voices`: Brass Tutti, ten POLY voices at DRIVE 12 dB, three
workers, 256-frame blocks paced in real time, none of the threads promoted), with the machine's
load average at 12 to 14: alone a median 50 % of the period and its 99.9th percentile 142 %;
with measurements made one after another beside them, 43 % and 168 % (twelve measurements);
the worst 210 % both ways. The machine's noise decides more than the measurements; to be
measured again on a quiet machine.

**Evidence (the Linux reference machine, 2026-10-09):** the CA-74's tests ported: the curve
drawn straight between its steps; a reader seeing each write once and never half of one; a
sound asked for once (and FEEDBACK's in its key); a measurement for a sound no longer wanted
let go; `autos_correction_glides_and_without_drive_there_is_none` (with the correction stepped,
2.6 times at once, it fails); `the_helper_measures_a_sound_asked_for` (and lets the curve go
when dropped); `a_session_holds_its_curve_and_an_old_one_the_average`;
`auto_brings_a_preset_back_and_off_it_is_louder` (Lead at 16 kHz). `preset_render` against
UNISON's commit: every preset the same to the bit. The plug-in's unit tests (146 passed,
7 ignored), the plug-in's and the model's integration tests; rustfmt; clippy.

### Into plugin-kit (2026-10-10)

**The owner, 2026-10-09:** "are we going to move the stereo effects to the plugin kit as well?
seems like a common thing we would use"; told the general parts would, after the materials,
and on 2026-10-10: "Do 1 & 2" (the kit's materials published and the plug-ins pinned to them;
and this).

**Agent decisions, 2026-10-10** (not separately approved):
- The general parts are plugin-kit's now (`plugin-kit-stereo`, its K6), taken from
  github.com/idlefoundry/plugin-kit at one commit (b301c97, its `stereo` branch until it is
  merged there), with the panel's materials (K5) at the same commit: the pan law and the
  placement (`character.rs` re-exports them), DOUBLE's pair, detune and trim, UNISON's trim,
  the places' glide (`engine.rs`), and AUTO GAIN's curve, its sharing among the threads and the
  host, its measurement a render at a time and K-weighting (`drive.rs`).
- The CA-72's own stays here: `Probe` (its voices, which measure a sound, as the kit's
  `Measure`), a sound's key (`drive::sound`), the four steps up to DRIVE's 24 dB, the presets'
  average curve (`drive::AVERAGE`), the notes played, and the session's field (`drive_curve`,
  on the plug-in's own `drive::Calibration`, since the kit takes no nih-plug).

**Evidence (the Linux reference machine, 2026-10-10):** against the commit before (414b4d9):
`preset_render`, every preset the same to the bit, from the kit's worktree and again from its
published commit; every factory preset's AUTO GAIN curve measured at 48 kHz the same to the
bit; three presets each with DOUBLE, UNISON and each placement forced on (eighteen renders,
SPREAD 80 %, DRIVE 7 dB) the same to the bit; the session's field the same text
(`a_session_holds_its_curve_and_an_old_one_the_average`). The plug-in's unit tests (140
passed, 7 ignored: the placement's and the curve's tests moved to the kit, where they pass),
its integration tests; rustfmt; clippy. Not yet: the CA-74 on the kit's K6 (and K5 from its
published commit).

- **The number** of this record is a placeholder until the merge.

## R46. FILTER MODE: the hardware reference's high-pass
**Owner decisions, 2026-10-09.** The CA-72 gets the hardware reference's FILTER MODE switch
(LO or HI): "I'm just looking to offer the same functionality." Measured on the reference
first (session HP), then built. On the panel where the reference has it: at the head of the
switch column between MIXER and MODIFIERS, the original's three switches below it (the
second of two mock-ups drawn by the panel's renderer; the first kept them in place and put
FILTER MODE at the column's foot).

**Agent decisions, 2026-10-09** (not separately approved):
- **What HI is** (session HP: EMPHASIS 0, 2.5, 5 and 7.5, each in LO and HI back to back at
  six control voltages; `scripts/calibration/hp_mode.py`). The filter's input less its
  low-pass output at equal gain, through the same output: HI = V - b LO against the
  reference's MIX output, b = 1.002, V flat within 0.25 dB, explains all 24 sweeps to -29 to
  -42 dB, and rejects a high-pass not made from LO (the analysis's self-test). So about 6 dB
  an octave below the corner, not the 24 its maker gives; a bump of +3 dB above it; the deep
  bass back to about -8 dB at 30 Hz, where only the low-pass branch has the output's
  coupling (C5/C1); and as EMPHASIS lowers LO's passband, a shelf (-12, -3 and -2 dB below
  the corner at 2.5, 5 and 7.5) with the resonance's peak on it. The cancellation holds as
  the input overdrives (150 Hz at CUT CV -4 V, within 0.85 dB over 34 dB of level), and HI
  distorts less than LO: part of LO's distortion is after the mixing point. EMPHASIS 10
  sings at LO's pitch and level in HI.
- **The circuit** (`circuits/boards/filter-mode.lib`, behavioural: the reference's circuit
  is not known, only its transfer). HI is the bus's Norton current (the current its
  channels would push into a virtual ground) through `vcf::MODE_RT`, a coupling of
  `vcf::MODE_HZ`, less the filter's output. The Norton current, not the bus's voltage:
  against it, the CA-72's LO falls at low frequencies as the reference's does against MIX
  (-0.82 and -0.88 dB at 30 Hz), while against the bus's voltage C27 would add its own.
  MODE_RT, 23.6K, is the filter's passband at EMPHASIS 0 as the voice has it, from ngspice:
  the voice's trims (REGEN CAL at 0.78, which leaves a little feedback at EMPHASIS 0) and
  every mixer channel's resistor on the bus (on or off, they share the current: one channel
  alone gives 26.9K; NOISE on changes it by 0.15 dB). At 26.9K, HI's shelf sat 1.1 dB too
  high against the reference's. MODE_HZ, 3 Hz, matches the phase between the two branches,
  which is all HI hears below 100 Hz: within 0.3 degrees and 0.1 dB from 15 to 80 Hz.
  (The reference's direct branch leads its MIX as 6.8 Hz would, but MIX is not the
  CA-72's Norton current; at 6.8 Hz HI's lows came back 3 dB short at 30 Hz.)
- **In real time** HI is taken inside the filter's oversampled step (`Vcf::high_pass`), so
  that the two branches line up through the resamplers; the coupling's state runs in LO
  too, as its capacitor would. A few operations a step: `preset_cost` (ten voices, one
  thread, d07d1d0 and this branch alternated three times under the timing lock, the least
  of each) moved by -1.9 to +1.7 %, median +0.3 %, inside its run-to-run spread (the
  machine's load 4 to 7). LO is the output as before: every factory preset renders the
  same to the bit (`preset_render`, against d07d1d0).
- **Against ngspice** (`vcf_realtime::filter_mode_matches_the_circuit`, the bench with the
  voice's trims and the rest of the mixer on its bus, `MIXER_REST`): HI's error measured
  against the larger branch (HI's error is LO's, the direct branch is exact), budget -26 dB
  to 10 kHz (LO's own 0.3 dB with a few degrees), -18 dB above; worst -27.2 dB, at 8.9 kHz
  on a resonant peak.
- **Against the reference** (`render_hp.sh`, `compare_hp.py`: session HP's takes through
  `ca72-lab stim`, each normalised to its own LO passband): HI within a median of 0.1 to 1.0
  dB and a 90th percentile of 1.7 dB in every pair (No Compromises; Potato the same within
  0.4 dB); the lows at 30 Hz within 0.4 dB; the shelf within 0.8 dB at EMPHASIS 2.5 to 7.5.
  What remains is LO's own (EMPHASIS's peak, the control voltage's scale: the calibration
  README's "Still differs"). At EMPHASIS 0 the notch below the corner is deeper in the CA-72
  (-41 to -60 dB against -29 to -46): its depth rests on a fraction of a degree.
- **The plug-in:** `filter_mode` (LO by default), in presets (a preset without it is LO),
  MIDI-learnable as a two-way switch, in `ca72-lab`'s patches as `"filter_mode": "lo"` or
  `"hi"`. The panel test's approved drawing takes the new column from the renderer (its
  region named in `tests/approved.rs`); everything else is still compared with the approved
  mock-up.
- **Not done:** no factory preset uses HI (the owner's call); the calibration README's list
  of sessions gains session HP when this branch meets `cal/behringer`'s.
- **The number** of this record, and board4.md's B4-HP, are placeholders until the merge.

## R47. QUALITY: Potato mode, the light voices, at LO

**The owner, 2026-10-10:** "with this release I think we should officially launch potato mode.
We'll just have a HI/LOW selector switch on the UI, I think. Perhaps we can put it above the
power switch?" Told that Potato mode (the development line's R31, on its unreleased `perf`
branch, 2026-10-06) is fitted to the circuit as it was before the hardware reference
recalibrated it (R42 on), and knows neither DRIVE nor FILTER MODE: "just do quality HI / LO for
now." Shown QUALITY above POWER in the mock-up, as POWER's black rocker or as FILTER MODE's
orange one: "black, for sure". The owner's decisions of 2026-10-05 on the development line
stand: a lighter model, allowed to sound worse than the circuit's, costing no more than a
comparable patch of a common free synthesizer ("Surge XT's usage is your ceiling. Make it as
good as you can under that."); saved with the session, not with presets.

**Agent decisions, 2026-10-10** (not separately approved):
- **Where:** the branch `potato`, from `realistic-look` with `cal/hp-mode` merged (the release
  carries both, and the light voices must play FILTER MODE too). The development line is
  another history (its base, 6126b71, is not this one's): its light model
  (`crates/ca72/src/light.rs`) is brought over whole, its engine's integration written again
  for this engine.
- **The switch:** QUALITY, in the panel's last column above POWER's lamp, where the column was
  empty: POWER's black ribbed rocker, upright, HI its upper end pressed, LO its lower (the
  worn skin draws it from POWER's picture). The host's parameter `potato`, named Quality, HI or
  LO, HI by default; not a preset's (`library::KEPT`); not learned by MIDI Learn (its menu: "set
  for the computer, not the sound"). (Since R48, a chrome toggle of three positions, and the
  parameter `quality`.)
- **The engine at LO:** a light voice beside each circuit voice and beside DOUBLE's twin, made
  with it, played in its place with the same keys, character (its offsets read each sample, as
  the circuit's voice reads them, so that a voice's character is the same whichever plays it),
  places, DRIVE and FEEDBACK; UNISON's voices started out of step as the circuit's are; a
  broken voice put to rest in place (nothing to make); no worker woken for a light voice (it
  costs less than the waking); the one instrument played a run of 128 samples at a time, its
  lock taken once (the same samples, to the bit, as a sample at a time). Switching QUALITY
  lets go of the keys held, as switching POLY does. AUTO GAIN measures a sound at LO with the
  light voices, its key LO's own (HI's keys as before, so sessions' curves stand).
- **What a voice costs** (`CA72_POTATO=1` with `tests/preset_cost.rs`, the Linux reference
  machine, ten POLY voices, one thread, unpaced): 2.1 to 3.8 % of a core at LO against 78 to
  107 % at HI for Bass, Cruising Whistle and Brass Tutti (30 to 45 times lighter).

**Evidence so far (the Linux reference machine, 2026-10-10):** HI plays every factory preset
the same to the bit as before the change (`preset_render` against 1535b5f); the engine's tests
of LO (the light voices play, switching lets go of the keys, POLY's voices sound and are freed,
the one instrument's runs the same to the bit as its samples, DOUBLE placed and UNISON
levelled as at HI, AUTO GAIN measuring with the light voices); `quality_lo_neither_allocates_nor_frees_while_it_plays`;
`tests/fuzz_bounds.rs` with QUALITY among the controls at their ends; the editor's QUALITY
clicked as a rocker and its MIDI Learn menu; rustfmt; clippy.

### The light voices fitted again to this circuit (2026-10-10)

**Agent decisions** (not separately approved; a helper agent on the branch `potato-fit`, merged
here at cf2e1e0):
- **The harness** from the development line (`tests/measure/mod.rs`, `potato_reference.rs` and
  `potato_match.rs`: every law from a panel control to pitch, level, cutoff, resonance and
  time, measured alike on the circuit's voice and the light one), with DRIVE on six patches,
  FILTER MODE's HI against LO (noise curves, sawtooth levels, the ring), FEEDBACK on four
  patches at four of its settings, oscillator 3's free FREQUENCY and the release with DECAY
  off; `potato_cost.rs` a timed loop.
- **Each knob read through the circuit's own law** where the hardware reference recalibrated
  it (`volume_track`, the FREQUENCY tracks, `sustain_track`, `time_pot`, `emphasis_r14`,
  `glide_r`, `ext_taper`, `mod_wheel_r`), the laws behind them fitted again, and DECAY's shape
  (faster at first, slower at its end), the input pair's compression, the lowest cutoffs, the
  rings' level, the shark tooth and EXTERNAL INPUT's couplings fitted as the circuit's. Against
  the circuit as it is now (the light voice before, where it moved):

  | Law | Circuit | Light | Before |
  |---|---|---|---|
  | Oscillator 3 free, 8' | 201.49 Hz | 201.50 | 232.68 |
  | Level against VOLUME 2 / 6 / 10 | .0291 / .1061 / .1863 | .0290 / .1054 / .1854 | .0353 / .0980 / .1811 |
  | Compression, three oscillators at 10 | -1.19 dB | -1.06 | -0.70 |
  | Ring at EMPHASIS 10, CUTOFF 0.5 | 1056 Hz, 0.170 | 1056 Hz, 0.169 | 857 Hz, 0.155 |
  | Threshold at CUTOFF .3 / .5 / .7 | .827 / .725 / .700 | .825 / .729 / .700 | .789 / .742 / .733 |
  | Passband at EMPHASIS 5 / 7, CUTOFF .5 | -9.35 / -11.80 dB | -9.32 / -11.60 | -6.59 / -11.15 |
  | KEYBOARD CONTROL 1 / 2 / both | .328 / .649 / .956 | .328 / .646 / .952 | .339 / .673 / .977 |
  | AMOUNT 2.5 / 5 / 7.5 at 3.83 V | +1.52 / +3.87 / +6.15 oct | +1.51 / +3.84 / +6.18 | +1.88 / +3.83 / +5.88 |
  | ATTACK 4 to the plateau, filter / loudness | .250 / .438 s | .252 / .440 | .433 / .685 |
  | DECAY 2 / 6, loudness to half | .091 / .313 s | .091 / .313 | .123 / 1.135 |
  | SUSTAIN 2.5 / 5 / 7.5, loudness | .500 / 1.675 / 3.070 V | .504 / 1.671 / 3.074 | .655 / 1.704 / 2.897 |
  | Release, DECAY on at 6, to 90 % | 1.165 s | 1.163 | 3.784 |
  | VCA at 1.0 / 1.9 / 3.1 / 4.3 V | .193 / .423 / .732 / .999 | .194 / .421 / .732 / .999 | .170 / .404 / .720 / .995 |
  | GLIDE 6, 12 semitones up / down | 70.5 / 126.4 ms | 69.7 / 125.5 | 65.2 / 141.0 |
  | Oscillator modulation, wheel 1 | +752 / -592 cents | +745 / -585 | +991 / -779 |
  | EXTERNAL INPUT's gain, VOLUME 2.5 / 5 / 7.5 / 10 | 2.07 / 4.38 / 14.75 / 159.0 | 2.08 / 4.41 / 14.87 / 158.0 | 2.79 / 7.28 / 14.02 / 137.3 |

  (The rest of the helper's table: the pitch, both FREQUENCY tracks, the triangle's second
  harmonic, the shark tooth's harmonics, the rings and peaks at low cutoffs, the passband near
  CUTOFF 0, ATTACK 8, the release with DECAY off, the retrigger, the modulations, the noises,
  A-440 and the filter contour's rest, each within a percent or two of the circuit's.)
- **DRIVE** (`Light::set_drive`, the circuit voice's argument): the bus into the input pair
  raised, the pair's limit falling as the gain to the -0.045 (0.88 at 24 dB). The level's rise
  at 6, 12, 18 and 24 dB within 0.75 dB of the circuit's on six patches (a sawtooth open, 5.67 /
  10.54 / 13.60 / 14.95 dB against 5.74 / 10.75 / 13.91 / 15.02); open patches darken as the
  circuit's do. Driven hard at EMPHASIS 7 the light voice loses its resonance sooner (three
  sawtooths at 24 dB: highs -14.5 dB against -9.7).
- **FILTER MODE's HI:** the bus at the open ladder's EMPHASIS-0 passband through a 3 Hz
  coupling, less the filter's coupled output (13 Hz, apart from the 4.4 Hz one after the VCA).
  Noise curves at CUTOFF .3, .5 and .7 and EMPHASIS 0 to 5 within 0.2 to 0.45 dB RMS from 40 Hz
  to 10 kHz (EMPHASIS 0 at .5 and .7, 1.6 to 1.8 dB: the notch deeper; EMPHASIS 7.5, at its
  threshold, 1.7 to 2.9 dB); the sawtooth's levels within -0.57 to +0.07 dB; at EMPHASIS 10 the
  same ring as LO's and the circuit's.
- **What still differs:**
  - FEEDBACK's loop has the circuit's latency (about 3.5 samples more) and the preamplifier's
    coupling, and its tones are within 1.5 % of the circuit's; but which oscillation the loop
    falls into differs (alone at CUTOFF .5 the circuit drops into a 26 to 38 Hz relaxation from
    3.8 on the knob, the light voice holds 440 Hz; a sawtooth at CUTOFF .6, the knob at 5 to 7,
    the other way about). Undertow Growl is 2.9 dB quiet at LO and 8 to 16 dB short at 0.5 to
    2 kHz in its first moments. Stopped after three experiments without a cause, as the stuck
    rule has it.
  - Ringing Saw Line darker (0.72; 2 dB down from 4 to 8 kHz: the light ladder's top octaves,
    R31's); Closed Hat a little brighter (1.16); Ladder Kick 0.8 dB louder.
  - The DECAY switch off slows the circuit's attack a little (0.537 against 0.570 at 0.2 s into
    ATTACK 6); not modelled. The release with DECAY off without the circuit's slow creep after
    90 %.

**Every factory preset at HI and LO** (`tests/potato_ab.rs`; LO's level less HI's, and the
ratio of their spectral centroids): within 1 dB in 22 of the 24 and 0.87 to 1.13 in brightness in
22, R31's mark; outside it Pulse Strut -1.2 dB and Undertow Growl -2.9 dB, Ringing Saw Line
0.72 and Closed Hat 1.16. Their renders, HI then LO, are on the owner's Mac for listening
(`~/Downloads/ca72-potato`).

**What a light voice costs** (`potato_cost.rs`, ns a sample, the Linux reference machine): 34.3
with three oscillators into the ladder (about 168 cycles; 30.6 before the fit), 83.0 with
everything on (79.9). It allocates nothing.

**Evidence (the Linux reference machine, 2026-10-10):** after the merge, HI plays every factory
preset the same to the bit as before (`preset_render` against 1535b5f); the light voice's
tests (13, each new law seen failing with its law broken), the model's, the plug-in's and the
panel's (276 passed); rustfmt; clippy.

**To follow:** the cost on the Windows reference machine against the same free synthesizer's
patch, as R31 measured it; the owner's listening; the README.
- **The number** of this record is a placeholder until the merge.

## R48. QUALITY's ULTRA: the circuit's model with no compromises, and its lamp

**The owner, 2026-10-10:** "I think we should also bring back our highest quality mode. Is
that still in the code? I have a fun UI idea for it." Then: "I think we should have a no
compromises mode, don't you? Maybe the first time the user launches that mode, we can give
them a little warning that says that it's only intended for offline renders, or small projects
on powerful PCs." The idea: "a small panel above the quality selector (make it a slightly
rounded square) opens up, moves aside, and a big round light appears (the kind you see on old
tube amps). On the light, the word "ULTRA" is written. This change should ideally feel a bit
like a transformer taking shape. It should feel old school and mechanical, and make the user
feel like they can see a little bit inside the machine when the transition happens. Little
mechanical arms and parts, the green of a PCB dimly appearing in the background, etc. please
mock this up for me."

Mocked up in the browser (the lab's `ca-72/look/mock`, study "ULTRA"), the owner steered it in
turn: "Panel drops off to the right. Light slides in from the left and moves up once it's in
the center position."; of six ideas for the inside generated from words, "That's the right
direction" (jointed chrome arms, a lead screw, a filament in the lamp); "It needs to be obvious
how it's sliding in. Something needs to look like it's pushing it or pulling it, or both." (a
carriage and a chain); "the light looks too small when it's being pulled in. It's not a mile
away" (an inch down). Then a second way, asked for to compare: "just like a circle with 8
sliding blades, and those blades just pull apart to reveal the light underneath, and the light
lifts up ... the light is already underneath. It just needs to move upwards."; "I mean more
like a camera shutter."; "I like the little twist that the mechanical option does right at the
end when the light turns on. Can you do the same twist for the camera shutter version?"; the
blades "fit in just a little more with the surrounding look ... I still want it to be like
plain sort of gunmetal, like you had before. But I just want the color to match closely.";
"the move upward that the light takes and the rotation motion that turns on the bulb, all of
that should happen at the same time." And: "Looks great! I think that's the one we should go
with. We should also offer a no animation option that leaves the bulb out and turns it on
instantly." then "Actually, let's still animate the light coming on, but no rotation or
anything." The switch: "Sure, add a third position, let's see how that looks."; "That's only 2
positions graphically. The switch has no image for a middle position"; "No. Imagine it being a
3 positions graphically switch. Not a rocker."; of a rotary selector, a toggle and a slide, the
toggle's middle "should look like there's no angle to the toggle lever", then "Maybe you were
right. Give it the slightest bit of angle"; "Cool. Looks good. Let's run with the silver
toggle."

**Agent decisions, 2026-10-10** (not separately approved):
- **Where:** the branch `ultra`, from `potato` (QUALITY is R47's switch, given its third
  position).
- **The parameter:** QUALITY becomes the host's enum parameter `quality` (LO, HI, ULTRA, in that
  order; HI by default), in place of R47's boolean `potato`, which was never released. It
  stays out of presets (`library::KEPT`) and out of MIDI Learn ("set for the computer, not the
  sound").
- **The engine at ULTRA:** the circuit's voices with the model's No Compromises setting in place
  of its real-time one (the engine's HI), switched while a key is held, without letting it go
  (the model switches between its settings sample by sample). AUTO GAIN keeps HI's curve: the
  circuit is measured at its real-time setting, which No Compromises differs from far below what
  the correction can tell, and measuring at No Compromises would take the helper many times as
  long.
- **The switch:** the chrome toggle's three pictures (`assets/worn/README.md`), 50 units wide at
  (3023, 266), its lever up for ULTRA, out for HI, down for LO; "ULTRA" printed above it, "HI" to
  its left, "LO" below, in 15-unit letters, QUALITY above them at 168. A click above its nut
  chooses ULTRA, below it LO, on it (or on HI's legend) HI; the mouse wheel steps it; no drag;
  one gesture, at the release, as a rocker's. In the drawn skin, a drawn nut and lever.
- **The lamp:** at (3023, 92): the opening 50 units in radius, the ring 3.6 beyond it, the lamp
  46 (the mock-up's sizes); the lamp waits at 0.9 of its size, its bezel turned back 40 degrees.
  Opening, 2.5 s: the blades over its first 0.04 to 0.44, then the lamp up, turning and lighting
  over the rest (the light 0.12 of that behind the turn); closing, 1.8 s; the light only, 1.1 s
  on and 0.5 s off. The editor moves it on by the time between its frames (at most 50 ms of it,
  so that a stalled frame does not jump it), and opens with it where QUALITY is, still. The
  lens does not turn (its highlight stays where the room's light is): the lamp's pictures are
  split into a lens and a bezel, the bezel turned over it. The lamp's light on the panel round
  it is a very faint amber ring just past the steel one (0.06 at most, out to 1.3 of the lamp's
  radius; the owner, after the first build: "No need for that much yellow glow around the lamp.
  Keep it very subtle"). In the drawn skin: nothing while it is shut (the
  approved mock-up has no lamp), the shutter and a plain lamp as it opens.
- **The note:** once ULTRA's lamp is lit, until it is read (a press anywhere closes it), and
  then never again on that computer: "ULTRA · NO COMPROMISES / THE CIRCUIT'S MODEL EXACTLY, AT
  ABOUT SEVEN TIMES THE WORK OF HI: / ONE VOICE TAKES ABOUT A WHOLE CORE OF A FAST COMPUTER. / IT
  IS MEANT FOR OFFLINE RENDERS (EXPORT, BOUNCE, FREEZE) AND SMALL / PROJECTS ON POWERFUL
  COMPUTERS. PLAYED LIVE, IT MAY DROP OUT. / RIGHT-CLICK THE LAMP FOR ITS SHUTTER, OR ALWAYS
  OPEN. / CLICK TO CLOSE", left of the lamp's column.
- **Shutter or always open** (the light only): first a button in the presets drawer's top row;
  then, the owner: "Shutter versus always open mode should be selectable via a right click on
  the ultra lamp". A right click on the lamp (its ring and what is in it) opens its menu, as a
  control's MIDI Learn menu opens: "ULTRA'S LAMP · SHUTTER" (or "· ALWAYS OPEN"), its items
  SHUTTER and ALWAYS OPEN, the one it is in dim. The button is gone from the drawer, and the
  search has its room again. The computer's, not the session's; the note tells of it ("RIGHT-CLICK
  THE LAMP FOR ITS SHUTTER, OR ALWAYS OPEN.").
- **The computer's settings:** a file of its own, `CA-72/settings.toml` in the folder the
  presets' library keeps (`settings.rs`): `ultra_shutter`, `ultra_note_read`; keys it does not
  know kept, a file it cannot read never written over.
- **The approved test** (`tests/approved.rs`) leaves out the last column's head, above POWER's
  lamp: QUALITY, its toggle and ULTRA's lamp came after the approved mock-up and were approved
  in their own. With it out, the panel against the mock-up: mean 1.359, 99th percentile 30 (on
  `potato`, with the column in, 1.431 and 31).

**What ULTRA costs** (`tests/preset_cost.rs`, one POLY voice, 256-frame blocks at 48 kHz, one
thread, unpaced, the Linux reference machine, an AMD Ryzen 7 7800X3D):

| Preset | HI, of a core (worst block) | ULTRA, of a core (worst block) | ULTRA / HI |
|---|---|---|---|
| Bass | 12.9 % (22 %) | 98.3 % (187 %) | 7.6 |
| Cruising Whistle | 14.6 % (27 %) | 104.2 % (216 %) | 7.1 |
| Brass Tutti | 17.8 % (34 %) | 118.4 % (212 %) | 6.7 |

So one voice at ULTRA takes about a whole core of a fast computer, and its worst blocks twice
that: played live on this machine, even one voice drops out. A voice cannot be shared between
threads, so POLY's workers do not help a voice; they help many voices.

**Evidence (the Linux reference machine, 2026-10-10):** the engine's test of ULTRA (the voices
at No Compromises, the key kept when switched to HI); the real-time test with ULTRA switched on
and off while it plays (no allocation, no free); AUTO GAIN's key at ULTRA is HI's; the toggle's
gestures (`interact.rs`, the editor's `quality_goes_where_its_toggle_is_clicked`); ULTRA's lamp
following QUALITY by the time between frames, its note read once, the light only's times
(`ultra_s_lamp_follows_quality_and_its_note_is_read_once`); the settings file kept and a bad one
left alone; the parts' pictures cut out; the lamp's menu choosing its shutter or always open
(`ultra_s_lamp_s_menu_chooses_its_shutter_or_always_open`). The
plug-in's own frames at points of the opening (`the_ultra_pngs`) match the mock-up's. A frame
of the opening costs 1.45 ms to draw on average (3.4 at most) at the window's opening size,
3.2 ms (6.6) at twice it (`ultra_timings`, release); the editor's frames come every 15 ms. The
Mac's build ("CA-72 ULTRA", renamed, beside the owner's open Live) passes clap-validator and
pluginval (strictness 5); it has not yet been opened in a host.

**The owner, told what ULTRA costs and asked whether it should play HI live and the exact model
only when the host renders offline:** "No, I think Ultra should remain Ultra." ULTRA plays the
circuit's model with no compromises always, live or rendering; the note says what it costs.

**Open (asked, not answered):** the lamp's colour (amber, as built; green and red were offered);
whether the note shows once a computer.
- **The number** of this record is a placeholder until the merge.

### QUALITY's opening: a hamster at LO, the lamp at HI, a Tesla lamp at ULTRA (2026-10-10, later)

**The owner, 2026-10-10**, in the mock-up: "How hard would it be to put an animation of a
hamster runnign in a wheel for "LO" mode?"; "Let's have the same animation. Shutter opens,
hamster wheel appears and is brought forward as the hamster starts running."; "i would make him
more like an animated creature than a real one. And make his wheel fitting the mechanical
1970's aesthetic"; "don't make the hamster too cute and disney-like. I liked your first hamster,
he was just too real."; of three, "The "rise" hamster is the best". The wheel: "You gotta make
that hamster wheel look more like a rickety brass hamster wheel. Old school looking."; "don't
make the cage warped like that though. Otherwise it's better. Also, these hamster wheels
normally have a back to them don't they? More brass spokes."; "is there an easy way to make the
back of the cage feel more inset? Like making it a little darker or something?"; "Show me an
example with the number of spokes reduced by 15%", then "the hamster wheel has the right amount
of spokes now". The lamps: "the current ULTRA mode lamp should be what we see in High mode. Make
another ULTRA mode, which should be a Tesla coil, complete with animated lightning bolts coming
out of it."; "It should look a lot like the amber lamp, but blue, with lightning, and a tesla
coil in the center"; "HIGH is now the only mode without any animation. Can you just add a warm
glow to it? Perhaps something that can match the intensity of the synth itself?"; "make sure the
Tesla coil has the same glow intensity spread that high has now."; and, after HI, HIGH and
ULTRA had been printed on the lenses, "Get rid of "ULTRA" and "HI" writing on the glass."
("HIGH I mean"). Then: "fantastic work. build this out. I want to test it on my Mac.", and
"make sure nothing that we did with the UI here today decreases performance in any way. And one
other thing can we make the dots in the stereo spread a little more animated according to the
volume of their respective sounds in the synth? and perhaps someway to animate the detune effect
being placed on them as well? Show me a before/after video once you are done."

**As built** (the mock-up's study, `ultra.js`, `hamster.js`, `tesla.js`, is the spec):
- **The opening** (`ca72_panel::ultra::Opening`) holds one thing at a time: LO's hamster in his
  wheel, HI's amber lamp, ULTRA's Tesla lamp; changing QUALITY, what is up goes back down (1.8 s)
  before the next comes up (2.5 s) through the same shutter; then, from the build, "that's too
  long of an animation time. the transition from one mode to the next should take like 3
  seconds of animation": 1.25 s down and 1.75 s up, three seconds in all.
- **None in place of always open** (the owner, 2026-10-10: "i forgot about the "always open"
  mode. Let's remove that. Now the alternative is no indicator animation area at all. Just the
  switch and the panel will remain blank above it."): the opening's menu (a right click above
  QUALITY, where it is or would be) is SHUTTER or NONE; with NONE nothing of it is drawn, its
  ring's shadow included (it moved from the face's drawing into the opening's own layer), and
  the editor keeps what the setting shows up, still, so that shown again it is there. The
  computer's setting is the same (`ultra_shutter`, false now NONE). Then: "default mode is to
  show the quality indicators (tesla coil, hamster, etc.), but they can be hidden as an option.
  This can be selectable by right clicking the toggle switch.": shown by default (as it was);
  the choice in QUALITY's own menu, its toggle (or a legend) right-clicked: its title why MIDI
  Learn does not learn it, as before, then SHOW ITS INDICATOR and HIDE ITS INDICATOR (the one it
  is, dim), then MIDI Learn's two; a right click where the indicator is is the panel's, as
  anywhere off a control. The note: "RIGHT-CLICK QUALITY'S SWITCH TO HIDE ITS INDICATOR, OR SHOW
  IT." Its title, "QUALITY: SET FOR THE COMPUTER, NOT THE SOUND", read
  as if QUALITY were the computer's, all its instances' (the owner: "I want quality to be set for
  independently by each instance. not shared across the computer."): it is each instance's, as
  it always was, and the title says so, "QUALITY: SET PER PLUGIN INSTANCE". (Whether its
  indicator is shown is the computer's, for every instance, as a way of showing the panel.) The lenses are plain. Its menu (a
  right click) is "QUALITY'S OPENING · SHUTTER" or "· ALWAYS OPEN"; always open, each setting's
  thing stands in the ring and fades in over the first 0.35 of its coming (1.1 s; out in
  0.5 s), a lamp lighting as before.
- **The Tesla lamp:** the amber lamp's twin in cobalt glass with a miniature Tesla coil where the
  bulb was (generated as an edit of the amber lamp's picture, so the two match; its lit picture
  an edit of that), cut into a lens and a bezel as the amber one. Its lightning is drawn as it
  plays (the mock-up's, ported): six streaks at most, each living 0.1 to 0.25 s from the
  toroid's rim to the inside of the glass (0.74 of the radius), its jagged shape struck again
  every 45 ms, a wide faint blue glow, a paler stroke, a white core, a spot where it meets the
  glass; seeded, so a frame drawn twice is the same.
- **The lamps follow the synth:** the plug-in's output's level (its peak over each block, -42 dB
  dark to 0 dB full) through a filament (0.05 s up, 0.3 s down): at rest 0.28 of their light, at
  the loudest all of it, the filament flaring (0.65 at most) and a glow on the panel from the
  ring out (0.16 at most; at first it washed over the bezels, and was kept off them). The Tesla
  coil's streaks come only as it sounds, more, longer and brighter the louder it is.
- **The hamster** (the drawn, naturalistic one: an eight-frame run and a sleeping picture,
  generated) in the old brass wire wheel (generated round and true, its back of 31 spokes made
  26 by moving its own spokes, with their joints and heads, to even places; its back, inside
  the inner rim, darkened to 0.55 of its light and 0.38 just inside the rim, set back). The
  wheel comes forward from 0.72 of its size as he picks up his run; he runs at the wheel's
  foot, a little up the rising side (12 degrees at a full run), the wheel turning 137.5 degrees
  a second under him. Then "don't wait for the hamster to come forward for him to start
  running. He should already be running": running at full pace as the shutter opens on him.
- **Agent decision, for "nothing ... decreases performance":** at rest nothing moves. The
  hamster runs while the synth sounds (its output over -40 dB), slows and stands when it falls
  quiet, and dozes off a second later (0.7 s; the mock-up offered this, its default ran
  always); the coil sparks only while it sounds (the mock-up let an odd spark fly at rest). A
  window at rest costs what it did.
- **The drops** (the strip's voices' display): each swells and brightens with its voice's level
  (0.72 to 1.22 of a drop's size; rising quickly, falling over a fifth of a second); with
  DOUBLE a note's two drops beat against each other as two notes DETUNE apart do at 220 Hz (20
  cents, 2.5 a second), one swelling as the other ebbs and the two swaying in turn (24 % and 4
  units at 10 cents or more; then "a bit intense. dial it back a little": 13 % and 2; then
  "dial back the detune wobble a bit more": 7 % and 1), a pair coming out of its voice as
  DOUBLE comes on.
- **DOUBLE a switch of its own** (the owner, 2026-10-10: "whenever I put detune to 0 it
  instantly switches to scatter, and vice versa if I'm in scatter and try and detune it throws
  me into double ... i think it makes more sense to just make that knob unresponsive unless the
  user is in double mode"): DOUBLE had been DETUNE above 0. Now the parameter `doubled`
  (SCATTER | DOUBLE's tabs, a preset's, learnable from the tabs as POLY is); DETUNE only an
  amount, its knob dimmed and not answering the pointer in SCATTER (MIDI may still move it).
  DOUBLE at DETUNE's 0 is a pair at one pitch (the engine told a ten-thousandth of a cent:
  `params::DOUBLED_AT_NONE`). A session or a preset saved before reads DOUBLE on wherever its
  DETUNE is above 0 (`Ca72::filter_state`, `presets::targets`); no factory preset had DOUBLE.
- **Cycle of Fifths, a factory preset** (the owner, 2026-10-10: "I added a new preset "cycle of
  fifths" let's make that a factory default"): the owner's preset as saved (ten UNISON voices,
  two sawtooths at 8' about six semitones apart, DOUBLE at 100 cents, INNER 21, gliding), last
  in `sounds/presets.toml`, DOUBLE named on; its MAIN OUTPUT VOLUME 8.6 (it was saved at 10)
  for a momentary maximum of -17.97 LUFS, where the others are levelled (R15; `tests/
  preset_levels.rs`: at 10 it was -11.5). Its tags, ours: fat, glide. (The file kept VOLUME at
  10 until 2026-10-10, R55's branch: the commit named 8.6 but left the line as the owner's;
  measured again at 8.6, -17.96 LUFS.)
- **The levels:** each voice's output's peak over a block was measured already (for knowing when
  a voice falls silent: `Engine::levels` keeps it); the output's peak is measured (two compares
  a sample) only while an editor is open, else a flag read a block. The audio thread keeps the
  highest of each (an atomic max) until the editor takes them.
- **Cost:** what follows the synth (the opening, the drops) is drawn at most 30 times a second;
  something coming up or going down, every frame as before. The opening is two layers, the
  well, its floor and the ring drawn again only as something comes or goes, and what moves and
  its light; the lightning is drawn without blend modes or a clip (a fifth cheaper). The drops
  are drawn only near each (six radii): 2.3 ms a frame for twenty moving at twice the opening
  size, where they took 5.2 ms (`drops_timings`); with the mock-up's flat drops (R44), 0.77.
  Then, measured whole (below), the opening and the drops were put on one clock, thirty times
  a second, something coming up or going down too (two clocks out of step had presented sixty
  frames a second between them), and the window's frame turned into the screen's pixels a row
  at a time (`window::fill`): 3.1 ms a frame at twice the opening size, where a pixel at a
  time, a frame found for each, took 7.3 (`the_window_s_pixels_timings`), on every frame the
  window presents.
- **Measured whole** (`the_window_played`: 24 s in real time at sixty frames a second, a Retina
  window, four POLY voices with DOUBLE playing chords at HI, ULTRA, then LO, falling quiet, then
  HI with DETUNE swept; each frame drawn and turned into the screen's pixels as each build's
  `present` does, timed; the build before these changes, `ultra` at c23a0a1, played the same,
  each run twice, in either order): before, 57.8 % of one core on the editor's thread, 1,061
  frames presented, 9.6 ms a frame; now, 32.4 %, 745 frames, 5.4 ms. The audio thread does no
  more per sample in the voices; with an editor open it compares the output's samples for
  their peak and keeps eleven numbers a block, and with none, reads a flag.
- **DETUNE to 100 cents** (the owner, 2026-10-10: "let's make our maximum detune 100"): the
  CA-72's own `engine::DOUBLE_CENTS`, 100 (the kit's, the CA-74's, stays 20), its dial 0, 25, 50,
  75, 100. A session saved with DETUNE up plays five times as far apart. The drops' beat follows
  the pair's to 4 a second and stays there (two notes a semitone apart beat 13 times a second at
  220 Hz, which the drops would only flicker at).


## R49. INNER: the inner edge of each side's band, and WIDTH, INNER and DETUNE three across

**The owner, 2026-10-10:** "right now, we have a way to control the outer edges, which is
width ... Imagine we're just dealing with the left-hand side of the stereo spectrum. You have
your far left-hand edge ... and then you have your right-hand edge, which is currently our
center. Width would bring things in from the left-hand edge, but we would have no control to
bring things in from the right-hand edge, from the center." Told the two choices (a voice
placed in the centre; INNER a place of its own or a share of WIDTH) with a recommendation for
each: "Go with 'inner' and yes, this should affect the CA-74 as well". Shown two layouts for
each plug-in, drawn by their renderers (INNER under WIDTH, the display beside it under DETUNE;
or WIDTH, INNER and DETUNE three across, the VOICES section narrower), with three across
recommended here: "3 across wins for both".

**Agent decisions, 2026-10-10**, as recommended to the owner and taken with the name:
- **INNER is plugin-kit's** (its K7, at ac04479, the kit's branch `stereo-inner` on `stereo`;
  the panel's materials from the same commit): INNER, 0 to 100 %, is a share of SPREAD's way
  out, not a place of its own, so the two never cross and SPREAD still scales the whole
  picture; each side's voices sit in a band from there to SPREAD's edge, each as far across it
  as SCATTER puts it at full SPREAD. A voice SCATTER puts in the centre (CENTER's first, EVEN's
  last of an odd number) takes the side its turn falls on, the left first, so INNER clears the
  centre; DOUBLE's pairs keep to the band, a pair in the centre opening to INNER's share either
  side. The one voice's DOUBLE pair with POLY and UNISON off, all of SPREAD's way out (R45),
  stays there.
- **The parameter:** `inner`, the host's Inner, 0 to 100 %, off by default, after AUTO GAIN, so
  a session or preset saved before reads it 0; a preset holds it (no factory preset names it);
  MIDI Learn's INNER (54 learnable controls: 27 knobs, 8 selectors, 19 switches). It glides as
  SPREAD does (the voices' targets move, 10 ms), it is not part of a sound's key for AUTO GAIN,
  and it does nothing at SPREAD 0 or with POLY and UNISON off and no DOUBLE.
- **The display** draws each voice where the engine's gains put it, INNER's band and all, by
  the kit's `voice_at` and `pair_at`, which the gains use too.
- **The strip, three across** (R44's A6 redrawn in its STEREO section): VOICES (370, 976) and
  STEREO (976, 2370), OUTPUT as it was; WIDTH, INNER and DETUNE 430 apart about STEREO's middle
  (1243, 1673, 2103), each a knob and its readout; the two banks of tabs centred over the
  section as a pair, 180 apart, in their order; the display from WIDTH's unit to DETUNE's, wider
  than before. INNER is a strip knob (51 controls), appended after the others so that no
  control's index moves, with its readout, a switch as WIDTH's (off, and back on at its amount),
  lit where it does something; its tip reads "INNER: 40 %".
- `docs/panel.png` is left as it was (the 0.1.0 picture, as R44 leaves it): drawn again, it
  would be the worn editor's first picture there, for the look's merge to choose.

**Evidence (the Linux reference machine, 2026-10-10):** `preset_render` against renders of
dd215c4: every factory preset (24) the same to the bit; three presets each with POLY, UNISON and
DOUBLE in each placement, SPREAD 80 % (27 renders), the same to the bit. The kit's tests (19,
its K7); here `inner_clears_the_centre_and_one_voice_stays_there`,
`the_strips_switches_turn_their_amounts_off_and_back_on_at_their_last`,
`the_strips_units_stand_in_their_sections_clear_of_each_other`, and the strip's MIDI Learn,
gesture, tip and switch tests with INNER. The panel's and plug-in's tests, 202 passed, 16
ignored; `approved.png` (the panel) unchanged; rustfmt; clippy with `-D warnings`. Not yet: the
installed build in a host.

### EDGES dropped (2026-10-10)

**The owner, 2026-10-10**, once INNER was built: "I don't really see any difference between edge
and even, because edge is now just even with the inner controls on maximum, is it not?" Told it
is close but not the same at eight to ten voices: "if there is cases where it yields different
behavior, then maybe we can just leave it. Just make sure that that's true, that it's not
exactly replicable from other settings." Told where it differs, then: "Do you think those
differences will be practically audible? I'm changing my mind again…"; "Drop it."; "Actually
render first"; and having listened to the renders: "k, I think we should remove edges".

**Agent decisions, 2026-10-10**, as recommended to the owner:
- **What EDGES had of its own**, measured with the kit's `voice_at`: with VOICES 2 to 7, EVEN
  with INNER (90 % at three voices down to 70 % at seven) puts every voice where EDGES did;
  with DOUBLE, every pair at WIDTH's edge is INNER at 100 % in either placement, to the bit;
  only with eight to ten voices and no DOUBLE does EDGES's strictly outside-in order (100, 100,
  90, 90, 80, 80 % ...) differ from anything EVEN or CENTER can do, the nearest EVEN at most
  about 8 % of the way out from it. Renders for the owner (`mac:~/Downloads/ca72-edges-ab`, the
  CA-72's, ten voices, WIDTH 100 %): POLY chords and UNISON with EDGES and with EVEN at INNER
  55 % measured the same width (side to mid within 0.05 dB, left-right correlation within
  0.006); single notes one at a time, the worst case, about 2 dB of balance on notes already 12
  to 17 dB to one side; plain EVEN, the reference, 6 to 8 dB.
- **SCATTER's placement is EVEN or CENTER** (`Scatter`, ids `even` and `centre` as they were;
  the host's Scatter Placement two positions). A preset's `placement` is read by the plug-in's
  index, 0 EVEN and 1 CENTER, an older 2 (CENTER) as CENTER (no factory preset names one); a
  session saved with `edges`, or a placement not understood, opens with EVEN (`filter_state`),
  not at what the instance had, as R18 has it for ENTROPY. MIDI Learn takes the placement as a
  switch, EVEN below 64 and CENTER from it (54 learnable controls: 27 knobs, 7 selectors, 20
  switches).
- **The strip:** PLACEMENT is two lit tabs, EVEN and CENTER, its recess a tab narrower; the two
  banks stay centred over STEREO as a pair, 180 apart. The lit tab and a click go by the
  plug-in's choice, not the kit's index.
- **The kit at eb374f8** (its K7's "EDGES dropped", made by a session the owner opened in the
  kit, since a change to the shared kit from here was refused): no `Edges`, EVEN and CENTER
  its placements (indices 0 and 1), and `ALONE` for one voice alone, whose DOUBLE pair goes all
  of SPREAD's way out (R45). The voices' mix carries no placement for a lone voice
  (`Option<Placement>`), and its pair takes `ALONE`, as it took EDGES's pair, 1, before; the
  panel's materials from the same commit.

**Evidence (the Linux reference machine, 2026-10-10):** `preset_render` against renders of
bd63519: every factory preset (24) the same to the bit. Tests: the placement's two choices by
their ids, the strip's two tabs where they stand and the lit tab, DOUBLE's pairs at INNER 100 %
the same to the bit in EVEN, CENTER and as the engine's lone pair, a value learned for each kind,
`a_session_saved_with_edges_opens_with_even`; the panel's and plug-in's tests, 206 passed, 18
ignored; `approved.png` (the panel) unchanged; rustfmt; clippy with `-D warnings`. With the
kit at eb374f8, against renders of d159f42: three presets each in MONO, POLY and UNISON, with
and without DOUBLE, EVEN and CENTER, INNER 0 and 60 % (72 renders), and every factory preset
(24), the same to the bit; tests, 206 passed; rustfmt; clippy with `-D warnings`.

**At the merge to main, 2026-10-10** (R44 to R49 together, as the owner asked: "k, can we merge
this into main now?", INNER with them: "yes"): the kit at 4c0f0f3, its main, where K5 to K7
were merged (its pull requests 2 and 3); its tree is eb374f8's, the commit pinned before, so
nothing built changes. The third-party notices leave the kit's crates out, as the CA-74's do:
they are Idle Foundry's own code under the CA-72's licence, which the notices now say
(`scripts/notices.py`). The README's picture of the panel (`docs/panel.png`) is the editor's
window as it opens in this look.

## R50. NOTE PRIORITY (LOW, LAST or HIGH) and RETRIGGER, over GLIDE and DECAY

**The owner, 2026-10-10:** asked what the usual choices of priority are ("lowest note, last
note, I would guess. Um, are there others?"), was told of the three (LOW, the Model D's own
keyboard; HIGH; LAST, most modern instruments' and the hardware reference's three, with LOW for
its poly chain) and the rarer FIRST and duophonic split, and how much adding them would take:
"I say we do lowest, highest, and last. I'm thinking for UI, it makes sense to put that control
in this section [the left hand controller: GLIDE, DECAY and the wheels] (rotated 90 degrees),
and we can use the same 3 way toggle that we use on the quality selector. This item should be
saved per patch, I think." Built on `look-quality` (the owner: "i feel like there are better,
more recent branches" than `potato`).

**Agent decisions, 2026-10-10** (not separately approved):
- **What each does**, with POLY off and in each of UNISON's voices (POLY's voices each take one
  key, where it changes nothing): LOW, the default, is the keyboard circuit as drawn, its
  contacts at the lowest and highest keys held, and every sound stays the same to the bit. HIGH
  and LAST hand the circuit the key they choose alone, as if it were the only key down: HIGH the
  highest held, LAST the one pressed last and, once it is let go, the last of those still held.
  A key let go keeps its pitch contact for the release lead (2 ms, A18) in every mode, so a key
  LAST or HIGH has chosen sounds that much longer unless a newer (LAST) or higher (HIGH) one
  takes over. Beyond the 44 keys the chosen key continues the string as a key alone does there
  ("The full MIDI range"). `ca72::voice::Priority`, the panel's `priority` (not the
  instrument's, as `quality` is not), chosen in `Keys::step`; the light voice (R47) the
  same, its keys' presses counted for LAST.
- **RETRIGGER** (the owner, told single triggering was kept and a retrigger choice not added:
  "ah, yes we should add retrigger for sure"): on, each key pressed starts the contours again,
  other keys held or not. As the circuit has it, the contours start again only once the
  trigger contact has been open about 12 ms (Q20 holding the reset line until C7 drains:
  `RETRIGGER_GAP`, 13 ms, `tests/retrigger.rs`), so a key pressed opens the trigger contact
  for that long first, the keys held staying down, and only then comes down, as if every key
  were let go and pressed again (POLY waits the same gap for a voice whose key is down). The
  new note's attack so starts 13 ms late, the pitch held where it was until then; a key pressed
  within 13 ms of the last let go waits out the rest. A key let go, the pitch falling back to
  one still held, does not start them again. Off (the default), single triggering, the
  instrument's: a key pressed while another is held moves the pitch without starting the
  contours again. `Panel::retrigger`; `Keys::note` and the light voice's `note` hold the key
  back (`waiting`) until the gap is out.
- **LOW keeps the string's own effect:** a higher key held with the lowest shorts the string
  between them, and the lowest sounds a little sharp of itself alone (key 57 with 64 held:
  0.98 mV, 1.2 cents). HIGH and LAST play the key chosen exactly as alone (within 0.05 mV), as
  the hardware reference's, whose keys are read digitally, plays all three.
- **The parameters:** `note_priority`, the host's Note Priority, LOW, LAST, HIGH in that order
  (the toggle's positions left to right, its value 0, 0.5, 1), and `retrigger`, the host's
  Retrigger, off by default, after it, the last parameters. A preset's, as any panel control:
  saved with the sound, LOW and off for a preset that does not name them (every factory
  preset). A session saved before them opens with LOW and off, not at what the instance had
  (`filter_state`, as R18 has it for ENTROPY). MIDI Learn takes NOTE PRIORITY as a selector, a
  third of 0-127 a position, and RETRIGGER as a switch (58 learnable controls: 27 knobs, 8
  selectors, 23 switches).
- **Where they stand** (the owner): first built as a row of NOTE PRIORITY alone between DECAY
  and the wheels, then: "note priority and retrigger should be added in the top row of the.
  keyboard controls section. not in the middle where you have it"; shown both on a top row
  over GLIDE and DECAY, their titles under them: "maybe we should put the names above, since
  that's what we do everywhere else", and "show me another example that has them all in a
  single column. Starting with priority, then retrigger, etc."; of the two (A, the top row,
  titles above; B, one column), "build Option B for sure". So the left hand's column is NOTE
  PRIORITY (y 95), RETRIGGER (185), GLIDE (265, was 150) and DECAY (345, was 230), each at the
  switches' place (x 214), the wheels 25 lower than A6 had them (y 540), every row's legend to
  its left as GLIDE's and DECAY's are (PRIORITY centred at 108, RETRIGGER at 104, moved in from
  their 118, clear of the lever at LOW and of the switch).
- **The toggle:** QUALITY's chrome toggle (R48), its three pictures mirrored about their
  diagonal (flipped top to bottom, then turned a quarter clockwise), so that the lever moves
  left and right and the light still comes from the upper left (a plain quarter turn would
  light it from the upper right): LOW the lever left (the upright picture's up), LAST out, HIGH
  right; 81 by 50 units as seen; LAST lettered above it and LOW and HIGH under its ends, in the
  dials' lettering, as QUALITY's are round its lever. A click goes to the position clicked
  (left of the nut LOW, right of it HIGH, on it or LAST's legend LAST), the mouse wheel a
  position a notch; its tip reads "NOTE PRIORITY (LOW, LAST or HIGH)"; its menu is MIDI
  Learn's. RETRIGGER is an ivory switch as GLIDE's and DECAY's, its ON under its right end, its
  mountings' screws as theirs. Both controls are appended after INNER so that no other
  control's index moves (55 controls). The drawn skin draws them the same way.

**Evidence (the Linux reference machine, 2026-10-10):** `preset_render` against renders of
80b8e1c, every factory preset (26) the same to the bit with POLY's ten voices, in MONO with its
chords held (where LOW chooses), and in MONO at LO (the light voice), RETRIGGER off. Tests:
`tests/priority.rs` (each mode's key after a run of keys pressed and let go, beyond the 44 keys
too, the keyboard's output against the chosen key's alone; RETRIGGER's contours started again
with a key held, the pitch held through the gap, a key pressed soon after a release waiting it
out), the light voice's `the_priority_chooses_the_key_held_that_sounds` and
`retrigger_starts_the_contours_again_with_keys_held`,
`note_priority_reaches_the_one_instrument_and_unisons_voices`,
`an_old_session_opens_with_note_priority_low_and_no_retrigger`,
`note_priority_and_retrigger_are_part_of_a_preset`, the toggle's clicks on its side; the core's,
panel's and plug-in's tests, 316 passed, 42 ignored; rustfmt; clippy with `-D warnings`. Not
yet: the installed build in a host.

## R51. The settings key and the editor's look; menus' heads; VOICES dragged by steps

**The owner, 2026-10-10**, shown a picture of the drawn skin: "why are you drawing it in that old
skin?", then: "i guess we're maintaining it, that's fine. Maybe we should add a settings button
on the right hand side of the preview preset row. We can let people change their skin there. But
note, all future versions default to the new skin." Shown the menu: "just say settings: don't say
'settings for this computer'", "also, we need to better distinguish menu headers from menu
options", and "do you think we should make look a submenu header and put the two options in the
submenu? I don't like 'LOOK: ITEM' as a format" (told a group with its head in the one menu was
recommended over a menu opening from it, two choices not being worth a second step). Then:
"make sure knobs or sliders with discrete options (like number of voices) shouldn't move smoothly
between discrete options when the user is clicking and dragging them."

**Agent decisions, 2026-10-10** (not separately approved):
- **The settings key** at the rail's right end, a key as the star's and the arrows' (86 units),
  its mark a gear of eight teeth drawn in the keys' print (the typeface has none); the name's
  display 100 units shorter (2400) and › and SAVE moved left by as much, the gaps between the
  keys as they were (`strip::KEYS`, `BarTarget::Settings`).
- **Its menu**, MIDI Learn's kind, dropping down from under the key to its left, within the
  window: SETTINGS, then LOOK heading REALISTIC and CLASSIC, the look the editor is in marked
  with a dot and not to be chosen. REALISTIC is the worn skin, CLASSIC the drawn one (the panel
  as drawn; the strip has one look and keeps it). Choosing one makes the panel's renderer again
  in it at once (its pictures decoded as it first draws: a moment's pause, once).
- **For the computer, not the session or the preset:** kept in `settings.toml` as `skin`,
  "realistic" or "classic", beside QUALITY's indicator choice. Anything but "classic", or no
  file, is the realistic look, so every version opens in it unless CLASSIC was chosen there.
- **Menus' heads** (every menu: MIDI Learn's, QUALITY's, SETTINGS): the title on a band darker
  than the menu, ruled off from the items, in smaller (0.82), bold, spaced capitals in the
  accent's amber; it had been dim, like an item that cannot be chosen. A group's head (LOOK) is
  dim, smaller still (0.72), bold and spaced, ruled off from what is above it, its items set in
  a letter's width under it; the current choice is marked with an amber dot and lettered as an
  item, not dimmed (`Menu::heads`, `Menu::marked`).
- **VOICES dragged by steps:** a drag goes from voice to voice (a step every 30 pixels), the knob
  never between two (`interact::drag`), as the mouse wheel and the arrow keys already did. It is
  the CA-72's only knob of steps; RANGE and WAVEFORM already stepped.

**Evidence (the Linux reference machine, 2026-10-10):** `the_settings_key_chooses_the_look` (the
menu, its marked choice, the head that does nothing, the look changed and changed back, the menu
within the window), `the_settings_are_kept_and_a_bad_file_left_alone` (the look kept; anything
but "classic" read as realistic), the drag test (VOICES on a step at every pixel of a drag);
renders of the rail and of the menus (the realistic look). The core's, panel's and plug-in's
tests, 317 passed, 42 ignored; rustfmt; clippy with `-D warnings`.

### In hosts (2026-10-10), R50 and R51

**The owner:** "do the windows testing right now", having asked whether the tests cover "the
issues we had with Cubase and other DAWs before" (told: R36's Cubase behaviour and R31's side
chain have tests that act as those hosts did; R26, R28 and R32 are Windows-only and R41 macOS-only,
none of which had been tried on this build; nothing had been tried in a host).

**Windows 11 at 200 %** (the CA-74's harness again, `ca72-it\prio`: main's tree at 73ba311 built
as CI builds the release, MSVC with `--profile bundle`, renamed CA-72 PRIORITY with IDs of its
own; REAPER with a settings folder of its own; a desktop agent in the signed-in session):
- **Validators:** clap-validator 36 passed, 8 skipped, none failed; pluginval at strictness 10
  (its editor tests skipped: no desktop over SSH) SUCCESS. Steinberg's validator is not built on
  this machine (47 passed on the Mac, below).
- **REAPER 7.82, the VST3 and the CLAP:** the editor opened at the drawing's proportions (2892 by
  1652). NOTE PRIORITY went where clicked (LOW, HIGH, LAST, LOW) and RETRIGGER on and off, each
  read back through REAPER's API, by posted clicks and by real ones (the cursor moved and
  pressed); the settings key's menu (SETTINGS, LOOK, REALISTIC marked), CLASSIC drawing the panel
  as drawn at once and REALISTIC bringing it back, `settings.toml` following ("classic", then
  "realistic"); VOICES dragged part way between voices, the picture taken with the button still
  down, its knob on 5, then on 7, never between; ‹ and › stepping the presets from where the keys
  are now; the presets' list dropped over the strip and closed (R26); the editor whole after
  minimising and restoring (R28); a project saved with HIGH and RETRIGGER on, REAPER closed and the
  project opened again: HIGH and on; notes heard (the track's peak 0.12). The VST3 and the CLAP
  alike.
- **A harness fault, not the editor's:** a posted press within about half a second of the last
  release went where the real cursor was, not where the posted move put it (Windows sends the
  window the cursor's own place as the pointer's capture ends); with real input, a click and a
  drag 0.3 s apart both took (NOTE PRIORITY LOW then VOICES 8 to 9, HIGH then 9 to 8).
- **Cubase Pro 15.0.30** (the trial R36 used; the build in the user's VST3 folder for the test,
  recycled after): an instrument track with CA-72 PRIORITY, real clicks while it processed audio:
  NOTE PRIORITY to HIGH, staying there; RETRIGGER on (its tip "RETRIGGER: On"); POLY lit; VOICES
  dragged from 4 to 5, its readout 5; the settings' menu, CLASSIC and back to REALISTIC; OSCILLATOR-1's
  switch flipped; CUTOFF FREQUENCY dragged quickly and let go, staying there, and a double click
  setting it back to 0.00, each shown by the editor as Cubase sent it back (R36's fault showed as
  controls that did not stay). Cubase closed without saving.

**macOS 27 (the owner's Mac, Apple silicon only; VST3 and CLAP, no Audio Unit):** clap-validator
37 passed, 7 skipped; Steinberg's VST3 validator 47 passed; pluginval at strictness 5 without its
editor tests SUCCESS. Installed as CA-72 PRIORITY for the owner; not opened in Live by the agent.

Not tried: the Audio Unit and an Intel Mac (a universal build), R41's test (macOS-only, not run on
this build), Linux hosts, Cubase on macOS.

## R52. A value typed into a readout

**The owner, 2026-10-10:** "i should be able to type into any of the number displays and enter a
value i want the knob to turn to."

**Agent decisions, 2026-10-10** (told to the owner as they were begun, not separately approved):
- **A double click** on a readout begins: VOICES', ENTROPY's, WIDTH's, INNER's, DETUNE's, DRIVE's
  and LEVEL's. A single click does what it did: ENTROPY's, WIDTH's and INNER's readouts switch
  their amounts off and on (R45), so a double click there undoes its first press's switch
  before the typing begins; the others did nothing and are now the strip's targets
  (`StripTarget::Readout`). AUTO GAIN's reads what is measured and takes no value; DETUNE's takes
  none in SCATTER, where its knob does nothing either.
- **Typing:** the readout lit with what is typed, an underscore (its seven segments' lowest)
  where the next digit goes while there is room for one: digits, a point, and a minus sign for
  LEVEL, three cells as the readout has. Backspace takes the last back. Enter sets the knob, as
  the readout reads it (VOICES a count, ENTROPY, WIDTH and INNER percent, DETUNE cents, DRIVE and
  LEVEL dB), within its travel (VOICES 2 to 10), a gesture of its own; what is not a number sets
  nothing. Escape, or a press anywhere but the readout, leaves it as it was; the editor takes
  every key meanwhile and gives them back after (as the drawer does: R34).

**Evidence (the Linux reference machine, 2026-10-10):** `a_value_typed_into_a_readout_sets_its_knob`
(DRIVE 8.4 dB; LEVEL beyond its travel either way, Backspace, -9 dB; Escape and a press elsewhere;
ENTROPY's switch double-clicked, then 25 %; VOICES 12 as 10; DETUNE in SCATTER none); a render of
DRIVE's and LEVEL's readouts being typed into. The core's, panel's and plug-in's tests, 318
passed, 42 ignored; rustfmt; clippy with `-D warnings`.

**In hosts (Windows 11, 2026-10-10, the `prio` harness, f6da784):** REAPER 7.82: a real double
click on DRIVE's readout lit it with the underscore, "8.4" typed with real keys showed "8.4_", and
Enter set DRIVE to 8.4 dB (read back through REAPER's API: 0.35). Cubase Pro 15.0.30: the double
click began the typing (the underscore lit), but no key reached the editor: "8.4" and Enter did
nothing, and Escape went to Cubase, which closed the plug-in's window. Cubase gives a VST3
plug-in's view its keys through `IPlugView::onKeyDown`, which nih-plug's wrapper leaves
unimplemented (`kNotImplemented`), and keeps them for its own key commands; the drawer's fields
(FIND, a preset's name and tags) most likely fare the same there (not tried). Not yet fixed. A
first try in REAPER found the presets' drawer open before the double click, which three fresh
tries did not show again.

## R53. The keys Cubase gives a VST3 plug-in's view reach the editor

**The owner, 2026-10-10**, told that a value typed into a readout (R52) never reached the
editor in Cubase, why, and the fix recommended: "absolutely, you should do that."

**What was wrong.** Cubase keeps the keyboard from a VST3 plug-in's own window and gives the
plug-in its keys through `IPlugView::onKeyDown()` and `onKeyUp()`, using those the plug-in does
not take as its own key commands (Escape closes the plug-in's window). nih-plug's wrapper
answered `kNotImplemented`, so no key reached the editor there: not a value typed into a
readout, and not the drawer's fields (FIND, a preset's name and tags), which had never worked in
Cubase. REAPER gives the editor's window its keys itself, which is why they worked there.

**Agent decisions, 2026-10-10** (as recommended to the owner):
- **In the wrapper** (`third_party/nih-plug/PATCHES.md`, change 13): `onKeyDown()` and `onKeyUp()`
  offer the key to the editor (`Editor::on_host_key()`, `HostKey`: VST3's character, its
  `VirtualKeyCodes` code and `KeyModifier` flags, down or up), and tell the host `kResultTrue` if
  the editor took it, else `kResultFalse`, so a key the editor does not take stays the host's.
- **In the editor:** the host's keys are taken only while the editor holds the keyboard (a value
  typed, the drawer open, MIDI Learn's menu or learning), and then every one, so that none is
  Cubase's key command meanwhile; otherwise all stay Cubase's. Those taken are read as the
  window's own (VST3's codes named: Backspace, Enter, Escape, the arrows, Home, End, Delete, Tab;
  else the character, the keypad's digits, point and minus sign too when the host gives none;
  the modifiers) and acted on at the window's next frame, as its own keys are (`HostKeys`,
  `host_key_event`). A host that gives a key both ways has it taken once: each kept 150 ms for
  its twin from the other way (`KeyTwins`).

**Evidence (2026-10-10, 46a9ec1):** the wrapper driven as Cubase drives it,
`a_key_the_host_gives_the_view_reaches_the_editor` (not taken: `kResultFalse`; taken, down and
up, with character, code and modifiers); `a_hosts_key_is_read_as_the_windows_own`,
`the_hosts_keys_are_taken_only_while_the_editor_holds_the_keyboard`,
`a_key_given_both_ways_is_taken_once`, `a_value_typed_with_the_hosts_keys`; the core's, panel's
and plug-in's tests, 323 passed, 42 ignored; rustfmt; clippy with `-D warnings`. **Windows 11
(the `prio` harness, real pointer and keys):** Cubase Pro 15.0.30: DRIVE's readout double-clicked
lit the underscore, "8.4" showed "8.4_", Enter set DRIVE to 8.4 (the readout reading the value
back through Cubase); LEVEL's "-6_" and Escape: back to 0.0, the plug-in's window still open; the
drawer's FIND took "lead", the list narrowed to the six presets tagged so. REAPER 7.82: "11" into
DRIVE's readout set 11 dB exactly (read back through its API), FIND took "lead": nothing doubled.
Two runs in REAPER began with the drawer open: the harness moved the pointer and pressed at once,
and the first press was taken where the real cursor had been (on the name, left there by a click
in Cubase): not the editor's. Not tried: Cubase on macOS, other hosts' keyboards.

## R54. Oscillators 2 and 3's FREQUENCY: a semitone a mark, or the hardware's dial

**The owner, 2026-10-10:** "For the frequency dials on the octaves, we have two choices. Either
we follow the [hardware reference's] curves exactly (almost what we're doing today, except I
believe we have them pinned at 0), or we create accurate curves that are exact to the dial
position that they claim to be measuring. I think we do both, use our perfect ones by default,
but allow the user to use the exact curves if they should so choose." Told that the 0 mark is
the reference's measured knob's too (its 7 and 26 cents flat were where its knob was set by eye,
change 29), and asked three things, the owner chose: the law a preference of the computer's in
the settings key, each knob saved as its pitch, so a session or preset sounds the same through
either ("a"); Stacked Fifths "a true 4th and 5th"; and for older sessions "one conversion".
Then: "Build it, and when you're done, test it on existing patches from the old versions of the
plugin. If successful, merge it into main. Don't forget to make this a selectable option in the
settings menu."

**Agent decisions, 2026-10-10** (not separately approved):
- **The knob saves its pot's position, as 0.1.x did** (its plain value -7.5 to +7.5, the wiper at
  (value + 7.5) / 15 of the track), and the voice reads the pot as it is: change 29's law left the
  voice and the light voice (`voice.rs`, `light.rs`) for the knob (`ca72::freq_dial`). A pot's
  position is its pitch, so a sound is the same through either law; and since every released
  version saved the pot so, their sessions, their hosts' automation and their presets play as
  they did without a conversion (the "one conversion" the owner chose is not needed for them),
  the shared library's files keep format 1 (the DAW the model came from, and older versions,
  read them at their pitch), and the host's lane is the pot's. The host shows and takes the
  value in semitones from the oscillator's centre: "+7.00 semitones", cents to a tenth under a
  semitone ("+3.5 cents"); "7", "7 st", "35 cents" or "35c" typed (`params::frequency`).
- **The curve:** oscillator 2's and 3's semitones from the centre measured on the voice at 61
  pot positions, A3 held, from the oscillator's resets (`tests/freq_dial.rs`), a quintic in the
  wiper's distance from the centre: within 0.003 cent of the measured on A3, 0.08 cent on A2 and
  A5; its inverse by Newton's steps (`Pot::semitones`, `Pot::wiper`). The pot reaches -8.56 and
  +8.24 semitones (oscillator 2), -8.63 and +8.22 (3). 0.1.5's circuit stood within 1 cent of
  this everywhere.
- **SEMITONES** (`Law::Semitones`, the default): the knob at a mark puts the pot where the
  oscillator stands that many semitones from its centre, -7.5 to +7.5 at the stops (the service
  manual's 14 to 17 semitones of travel). **HARDWARE**: change 29's law (`Law::Hardware`, its
  tracks moved to `freq_dial`), the reference's -5 and +5 marks at -6.48 and +6.61 semitones
  (oscillator 2) from its 0, its stops the pot's ends. Both keep the 0 mark at the centre, where
  the factory tuning puts the oscillator in unison with oscillator 1. A pot beyond a law's stops
  (a sound made through the other, or set by the host) shows the knob at its stop and plays its
  own pitch; a press that does not move the knob leaves it so.
- **OSC. 3 CONTROL off**: the knob sets the pot as with the control on (the pot then spans 69
  semitones and the dial gives no interval); the readout reads the dial, as with it on.
- **The settings key's menu:** under LOOK, FREQUENCY DIALS heading SEMITONES and HARDWARE (the
  names the agent's), each group's choice marked (`Menu::marked` now one a group); kept in
  `settings.toml` as `frequency_dials`, "semitones" or "hardware" (anything else, or no file,
  semitones). Choosing one turns the two knobs at once; the parameters are not touched.
- **A learned controller** turns the knob, value / 127 of its travel, through the computer's law
  (`learn::normalized_for`): the plug-in reads the law as it is initialized and the editor as it
  opens or the law is chosen (`Ca72Params::freq_law`, not saved with the session).
- **The factory presets** keep their sound: their FREQUENCY values change-29's law had rewritten
  put back at their pots' positions, to six places, so every preset but Stacked Fifths renders
  as main's did (those existing in 0.1.5 at its values, within 0.00005; the feedback patches
  Pulse Strut and Undertow Growl had amplified a 0.005-cent difference left by four places into
  a different waveform). **Stacked Fifths** at +5.00 and +7.00 semitones, a fourth and a fifth to
  the cent (they had stood at +5.52 and +7.67 since 0.1.0, set as "5 and 7"): its level -19.98
  LUFS momentary maximum (-19.95 before), left at VOLUME as it was. **Cycle of Fifths**' second
  oscillator stands at +7.16 semitones as its owner set it (a fifth 16 cents wide); its
  description said "about six semitones apart" (change 29's mark), now "about a fifth apart".
- **Saved by the test builds** (CA-72 CAL, A6, ULTRA, PRIORITY and the rest, between change 29
  and this) a FREQUENCY value was a mark of change 29's law. The plug-in does not guess which a
  file holds (a mark key would not survive the DAW that shares the library, which writes a
  preset's file again without the keys it does not know): the owner's one such preset, Cycle of
  Fifths in the Mac's library (format 1 with parameters 0.1.5 did not have), is converted in
  place, its old file kept beside the folder. Their sessions are the renamed builds', which this
  plug-in's ids do not open.

**Found in the testing, not this change's:** since calibration change 5 (R162 2.96K, the
reference's oscillator 3 at 198.82 Hz on 8' with OSC. 3 CONTROL off), oscillator 3 with the
control off sounds 2.48 semitones lower than in 0.1.5 at every FREQUENCY (201.3 Hz at the centre,
0.1.5 232.4 Hz), as change 5 meant: a 0.1.x session or preset that uses it as a drone or a
modulation source plays it so. Main and this change agree to the hundredth of a hertz there.

**Evidence (the Linux reference machine, 2026-10-10):** `every_mark_is_its_number_of_semitones`
(the voice through SEMITONES at -7.5, -7, -5, -3, -1, +1, +3, +5, +7 and +7.5, each oscillator:
within 0.002 cent on A3, 0.08 on A2, 0.21 on A5), `the_laws_and_their_inverses`,
`frequency_follows_the_reference_at_its_marks` (HARDWARE's points),
`frequency_reads_in_semitones_and_keeps_the_pot`,
`a_controller_turns_a_frequency_knob_through_the_dials_law`,
`the_settings_key_chooses_how_the_frequency_dials_read` (the menu and its marks, the knobs
turned and the parameters untouched, the tip, a drag through each law, a knob past its stop
pressed), `stacked_fifths_is_a_fourth_and_a_fifth`, the settings' file and the light voice's
pitch law; pictures of the menu and of the knob at +7 through each law
(`the_window_settings_png`). The factory presets rendered (`preset_render.rs`, ten voices, 3 s)
against main's at 5dc7b81: 18 the same to the bit, 7 within -119 dB, Stacked Fifths changed. The
core's, panel's and plug-in's tests, 330 passed, 43 ignored, in release and in debug; rustfmt;
clippy with `-D warnings`.

**Old patches** (the owner: "test it on existing patches from the old versions"): every
FREQUENCY value of the factory presets the six releases shipped (one file, the same in each), of
the presets in the owner's library on the Mac (twelve saved by 0.1.x-era builds, one by a test
build) and the dial's -5 and +5 marks and stops. No session of a released version was on the
owner's computers (the one Live set using the CA-72 uses CA-72 CAL), so 0.1.5 made them:
- **On the voice**, each oscillator alone, A3, its semitones from its centre: this change within
  1.05 cents of 0.1.5's at every value (the circuit's own change since, main the same); main
  before it had every value but 0 off, up to 5.4 semitones (Elastic Octaves' oscillator 3 with
  the control off, +538 cents; the owner's edited Slow Horn Swell's, +431).
- **As sessions:** 33 states saved through 0.1.5's own CLAP entry point (the oscillator alone,
  FREQUENCY and OSC. 3 CONTROL set by the host's automation, then saved as a host saves), each
  opened through this change's: every FREQUENCY value as saved, to the last bit; played (A3, the
  second half of a second), the oscillator within 1.2 cents of 0.1.5's wherever it follows the
  keys, and oscillator 3 with the control off 2.47 to 2.50 semitones lower at every value (change
  5, above: at the centre main and this change both 201.2672 Hz, 0.1.5 232.3974 Hz).

**On the Mac (2026-10-10, main at 230c41e):** built as the bundles are released (`--profile
bundle`), renamed CA-72 SEMITONES with ids of its own (`com.idlefoundry.ca-72-semitones`,
`IdleFoundryCA72S`), in a folder of its own; clap-validator 37 passed, 7 skipped; Steinberg's
VST3 validator 47 passed; pluginval at strictness 5 without its editor tests SUCCESS. Installed
beside the other test builds for the owner, nothing replaced, and not opened in Live by the
agent. The owner's Cycle of Fifths converted (oscillator 2's FREQUENCY 5.8494 to 6.50964, +7.16
semitones as before), its old file in `CA-72/Presets-before-semitones`: the test builds before
this one read it 0.43 semitone sharp now. Not tried: any host, Windows, Linux.

## R55. Factory presets retired from the plug-in, and the pack that keeps them

**The owner, 2026-10-10:** "for the 0.2.0 release, I want to have a bunch of new presets, and I
want to remove a lot of the old ones. Will people who downloaded older versions still have
access to the original presets (which is what I want)?" Told that the factory presets are built
into the plug-in, so an upgrade drops those removed from the drawer: "can't we make it part of our
installer to have it convert someone's presets if they've already got an earlier version of the
plugin installed?" Told why not (below) and offered the retired presets kept built in, shown to a
player an earlier version had served (their presets folder or a session of 0.1): "If the person
never saved any presets or projects, then I don't really worry about saving their presets." Then:
"what do you think I should do. I'm just loath to think of the bland patches being shipped as
contributing to people thinking the plugin doesn't sound great." Recommended a clean cut with the
retired presets as a download beside each release: "Okay, let's just do it that way. As long as no
one's projects are affected, I'm happy." Which presets go, and the new ones, are the owner's.

**Agent decisions, 2026-10-10** (not separately approved):
- **A project keeps its sound.** A session holds every parameter's value and its preset's name
  (`preset`), never a reference to a preset's values, so it plays as saved whether or not its
  preset is still there. Only the bar reads the name: shown as not found, never marked changed,
  and nothing is set from it (`Browser::tick` has no setter; `presets::tests::
  a_session_whose_preset_was_retired_keeps_its_sound`). The plug-in takes no program changes and
  offers hosts no preset list, so neither can point at a preset that has gone. A player's edit of
  a retired preset is their file, listed as their own preset (without REVERT, nothing left to go
  back to). A retired name is never a factory preset's again (`library::tests::
  the_retired_presets_are_well_formed`), so a session's name cannot come to mean another sound.
- **Not the installers.** The Windows ZIP, the Linux tarball and a build from source install with
  no installer; the macOS package and the Windows installer run as an administrator and install
  for the computer, where the presets are each user's (`<shared>/CA-72/Presets`); and the work
  would be three programs in three languages, two of them testable only on their own system.
- **Not built in and shown to some.** It would keep the bland patches in front of the earliest
  and most vocal players, and every built-in preset is measured and levelled again with each
  change to the sound.
- **`sounds/retired.toml`:** a retired preset moves there whole from `presets.toml`, its values
  as levelled for the model then, in the same format. No build of the plug-in reads it.
  `tests/preset_levels.rs` measures its presets with `CA72_RETIRED` set, for levelling them again
  after a change to the sound, as the factory's.
- **The pack, `CA-72-<version>-0.1-presets.zip`,** in every release with a retired preset:
  `scripts/package.sh` makes it on Linux with the tarball (`examples/preset_pack.rs`), so CI's
  Linux build carries it into the release's draft and `SHA256SUMS.txt` with the installers; none
  while nothing is retired. Inside, `Presets/` and a `README.txt` (where the folder is on each
  system). Each preset is the user's (`library::write_pack`), tagged `0.1`, in
  `<name>_0.1.toml`: copying never writes over a player's file (theirs is `<name>.toml`), and
  where a player kept their own edit of the same preset, theirs, first by file name, is the one
  listed; a pack's preset saved again moves to `<name>.toml` when that is free (R18).
  `library::tests::the_pack_reads_back_as_the_users_presets_and_writes_over_none`. The update
  check finds its installers by their exact names, so it never offers the pack. The names (the
  asset's, the tag `0.1`) are the agent's, for the owner to change before 0.2.0.
- **Cycle of Fifths levelled as its record has it** (MAIN OUTPUT VOLUME 8.6, -17.96 LUFS momentary
  maximum): found at 10 while measuring every preset for the owner's choice, 6.5 dB over the
  others.

**Left for the cut:** the owner's choice; then the chosen presets move to `retired.toml` in one
commit, with `library::tests::the_factory_file_is_read_whole`'s count and the tests that name a
retired preset (`Bass` and `Lead` are named in several), the factory file's opening comment, the
README's presets section and 0.2.0's notes saying where they went (the pack, with a row in the
install table). The DAW's copy of the factory file follows when its owner chooses.

**Evidence (omarchy, 2026-10-10):** the library and presets tests pass; each new check failed
once with the code broken (the pack named `<name>.toml`, the bar's name cleared, a retired preset
named `bass`). Upright Pluck moved to `retired.toml` for a try: the pack held
`Presets/Upright Pluck_0.1.toml`, read back as the factory's values tagged `bass, plucked, 0.1`;
with nothing retired, `package.sh` made no pack. Every preset's level measured on this branch and
on 0.1.3's build (`~/.cache/ca72-baseline-07a7dfb`): each as R42's re-levelling left it, but Cycle
of Fifths.

## R56. R50 to R53 on macOS, in Bitwig Studio (2026-10-11)

**The owner, 2026-10-10:** "you should test that on a mac this evening", run at midnight by a
timer. The Mac (macOS 27.0, Apple silicon, Retina) has no Cubase; Bitwig Studio 5.2.7 was the host,
the build CA-72 PRIORITY2 (main at 5dc7b81, VST3 and CLAP), driven by real pointer and key events
from a terminal window (screen capture and accessibility are allowed there, not over SSH). Live,
open with the owner's sets, was not touched.

- **Bitwig, the VST3 and the CLAP:** the editor drawn whole at the drawing's proportions;
  NOTE PRIORITY to HIGH (its tip "NOTE PRIORITY (LOW, LAST or HIGH): HIGH") and to LAST; RETRIGGER
  on ("RETRIGGER: On"); POLY lit; VOICES dragged from 4, the picture taken with the button down,
  on 5 exactly; DRIVE's readout double-clicked, "8.4" typed, Enter: 8.4 (Bitwig's device panel
  then read "Drive 8.4 dB"); LEVEL "-6_" and Escape: back to 0.0, the window open; FIND took
  "lead" and the list narrowed to the six presets tagged so; the settings key's menu, CLASSIC drawing
  the panel as drawn and REALISTIC bringing it back, `settings.toml` following.
- **pluginval at strictness 10 with its editor tests** (Editor, Open editor whilst processing,
  Editor Automation): SUCCESS. **R41's test** (`softbuffer_copies`, macOS only): passed.
- **Found: a first click into the editor's window is lost while Bitwig's own window is in front.**
  NOTE PRIORITY's and RETRIGGER's first click after a click in Bitwig's arrangement did nothing;
  the next click on the same control took (pictures before, after the first and after the second).
  baseview's view answers `acceptsFirstMouse:` with YES, so the plug-in's window takes the click
  to come forward rather than passing it on; how Bitwig's floating plug-in window asks for that is
  not yet known. Every click inside the editor's window once it is in front takes. Not yet fixed.
- **A harness fault, not the editor's:** cliclick's `kp:return` and `kp:esc` were never delivered
  to the window (a throwaway build that logged every key, its own and the host's, saw the digits
  and nothing for them); the same keys pressed by their key codes (System Events) arrived as Enter
  and Escape and did what they should. Bitwig gives a plug-in's window its keys itself: none came
  through `onKeyDown()`. Bitwig only finds a plug-in it has not seen at its start (it was relaunched
  once for the throwaway build).

**In Ableton Live 12.4.15 beta (2026-10-11, 08:26), the owner:** "You can close my Mac Ableton
sets. I don't care", then, offered a test in Live and a tidy of the test builds, "i want both".
Live was quit (a TERM signal: the screen was locked, no dialog could be answered). Its log's start
named only its default template, but on relaunch its crash recovery named the owner's set open
then (on disk unchanged since the day before): its recovery files were copied to a backup folder
beside the owner's sets with a README on restoring them, and the recovery declined. The build, **CA-72 TEST** (main at f60833a,
the one test build from now on, replaced in place; A6, ULTRA to ULTRA3, PRIORITY and PRIORITY2
moved to the Trash; clap-validator, Steinberg's validator and pluginval at strictness 5 passed),
VST3 (Live takes no CLAP), on a new set's MIDI track, the MOTU at 44.1 kHz as Live had it:
- **The first click takes in Live:** after a click in Live's own window, one click on NOTE
  PRIORITY moved it (LOW to HIGH). Bitwig's lost first click is Bitwig's way with its floating
  plug-in windows, not every host's.
- NOTE PRIORITY to LAST ("NOTE PRIORITY (LOW, LAST or HIGH): LAST"); RETRIGGER on; POLY lit;
  VOICES dragged from 4, on 5 with the button down; DRIVE's readout "8.4" and Enter: 8.4; LEVEL
  "-6_" and Escape: 0.0, the window open; FIND "lead": the six presets tagged so; the settings
  key's menu (now LOOK and R54's FREQUENCY DIALS, each a group under its head), CLASSIC
  drawing the panel as drawn and REALISTIC bringing it back, `settings.toml` following.
- Live quit without saving the test set.

Not tried: Cubase on macOS (not installed), Logic and the Audio Unit (this build has none), an
Intel Mac.

**Bitwig's lost first click fixed, and the Audio Unit in GarageBand (2026-10-11, morning).** The
owner: "I don't have logic. What about garage band? And ya, fix bitwig."
- **The lost first click** (above): a throwaway build that logged every press, release and
  focus change, in Bitwig 5.2.7, showed both clicks reaching the editor whole; the first was
  taken where the pointer had last been seen. Bitwig sends its floating plug-in window no
  `mouseMoved:` as the pointer comes onto it while its own window is in front, and baseview gave
  the press without its place (an instant move-and-click after a click in Bitwig's window: no
  control found; the same with a move first: taken; Live's plug-in window gets the movement and
  never lost one). Fixed in baseview (`third_party/baseview/PATCHES.md`, change 4: each press and
  release tells the handler where the pointer is first). Tried: three instant move-and-clicks
  after a click in Bitwig's window, each switched RETRIGGER; a drag of VOICES begun the same way
  moved it to 5.
- **A release-style build:** CA-72 TEST (the tree above, renamed), `cargo xtask bundle-universal`
  and `scripts/auv2.sh` for both Apple silicon and Intel (`x86_64 arm64`), the Audio Unit with codes
  of its own (`aumu C72T IdlF`, "Idle Foundry: CA-72 TEST") beside the released one;
  clap-validator (37 passed, 7 skipped), Steinberg's validator (47 passed) and pluginval at
  strictness 5 passed. Apple's `auval` lists no Audio Unit but Apple's on this Mac, from SSH or a
  terminal in the desktop session (not even ones installed long before, such as RC-20), so it
  was not run; GarageBand finds them.
- **GarageBand 10.4.14:** listed under AU Instruments › Idle Foundry › CA-72 TEST (Stereo); its
  Smart Controls showed the panel's first parameters; the editor drawn whole (GarageBand opens its
  window partly off the screen's right and bottom: moved by hand). **Found and fixed:** with the
  transport stopped, the editor's changes did not take (NOTE PRIORITY, RETRIGGER, POLY, VOICES and
  DRIVE each shown and then gone) until playback began, when they all arrived: GarageBand was not
  rendering the instrument, and clap-wrapper ignored nih-plug's request to flush them
  (`third_party/clap-wrapper/PATCHES.md`, change 5). With the fix, stopped: NOTE PRIORITY to HIGH,
  RETRIGGER on, POLY lit, VOICES dragged to 5, DRIVE "8.4" and Enter, LEVEL "-6" and Escape back to
  0.0, FIND "lead" (the six presets tagged so), CLASSIC and REALISTIC (`settings.toml` following),
  each at once. The same gap affects Logic's tracks that are not selected while stopped (not tried:
  no Logic here). GarageBand's plug-in window is an ordinary window, behind its own once that is
  clicked, so the floating-window first click does not arise there.
- GarageBand's own project left by another session (ca74-au-test, which wants a CA-74 Audio Unit
  not installed) was opened by GarageBand at launch and closed without saving.

## R57. Four faults the CA-74's review of its ports found in the CA-72's code

**The owner, 2026-10-11,** through the CA-74's lead (its brief, `~/Code/ca-74/.lead/for-ca72.md`):
four faults that independent reviews of the CA-74's ports of the CA-72's opening, settings file and
frames drawn in part traced to the CA-72's code, each confirmed in its main at 08c6657; then, told
of them: "yes" (fix them).

**Agent decisions, 2026-10-11** (as the brief proposed, adapted where noted):
- **QUALITY's opening ran at about 22 frames a second, not 30.** `move_opening` moved on only when
  a thirtieth of a second had passed since it last had; baseview's frames come every 15 ms on all
  three systems, so it moved every third frame, 45 ms apart. It now keeps a due time, a thirtieth
  on from the last due time and never before the present (the brief's "never more than a
  thirtieth behind" let a stall bring a burst of ticks 15 ms apart; this moves once after a stall
  and once more at the next frame at most), and moves on by the time since it last moved, at most
  50 ms (`motion_due`). The strip's drops share its clock and had themselves waited a thirtieth
  between steps, so they would have missed the 30 ms ticks: on the editor's clock they now move
  on whenever it has (`StripRenderer::render_at`); on the wall's (`render`), at most thirty times a
  second as before. The opening's drawing now runs as often as it was meant to: its cost is that
  much higher (the CA-74 measured about 35 %; not measured here).
- **The Tesla lamp's glow was painted twice and flickered.** The streaks' flash wrote a second
  gradient with the steady spill's id into the same layer, and one element of an id is kept, so
  with a streak drawn the glow was the steady one twice, switching with the streaks. The flash
  has its own id (`u-spill-flash`).
- **One editor's settings write undid another's.** Each wrote the whole of what it had read as it
  opened, so an editor marking ULTRA's note read wrote back an indicator another had hidden, and
  could show the note again after another had read it; and every write used one temporary name,
  which two processes could rename into place torn. Now each change writes only its key, under a
  lock every instance takes (`settings.toml.lock`, the standard library's file lock), the file read
  again first (`Settings::keep`, `Change`); the note is checked in the file before it is shown; and
  every write in the library's folder has a temporary name of its own (the process's id and a
  count), removed if the rename fails.
- **The panel's damage was one rectangle round every change,** so QUALITY's opening at the top
  right and a lamp, a wheel, a tip or an automated knob in the same frame put most of the panel
  together again. It is now a short list, places that overlap or come within 8 pixels merged, at
  most eight (beyond, one round all), each put together again on its own (`Renderer::damages`);
  `damage()` still gives one round all, which the strip and the editor's rows take.

**Evidence (the Linux reference machine, 2026-10-11):**
`the_opening_moves_thirty_times_a_second_on_baseviews_frames` (90 moves in 3 s of 15 ms frames, it
had been 66; one move after a half-second stall and at most one more within 30 ms),
`the_drops_move_on_each_tick_of_the_clock_they_are_given`, `no_layer_of_the_opening_repeats_an_id`
(every setting, lit and sparking or not: it fails with the flash's old id),
`one_editors_change_does_not_undo_anothers` (two editors' stale copies; four threads keeping a key
each 25 times at once: all land, never torn, no temporary file left),
`ultra_s_note_read_elsewhere_is_not_shown_again`,
`places_changed_apart_are_put_together_again_apart` (the opening and TUNE in one frame: two places,
under an eighth of the area round both, the frame as put together whole). The core's, panel's and
plug-in's tests, 339 passed, 43 ignored; rustfmt; clippy with `-D warnings`. Not measured: the
editor's cost at baseview's own frame rate.

## R58. The window shown where it changed; the lamps' light kept as pictures

**The owner, 2026-10-11,** told of the CA-74's note (`~/Code/ca-74/.lead/for-ca72.md`, "Also worth
knowing": converting and presenting only the changed rows and keeping the lamps' light as pictures
took the CA-74's played editor from 21 % to 13 % of a core): "Yeah, let's do that."

**Agent decisions, 2026-10-11:**
- **Measured first.** `the_window_played` (24 s of chords at HI, ULTRA, LO and HI with DETUNE swept,
  a 3440-pixel window) now times each frame twice, drawn and turned whole (macOS's way) and drawn
  and turned in the places changed (Windows' and X11's), and checks at every frame that the screen
  turned in places is the screen turned whole. On Linux it counts the thread's own time
  (`/proc/thread-self/schedstat`): with other builds running, the wall's clock varied a third from
  run to run. Probes in `draw` (not kept) found the costs, and the two largest were not the note's.
- **The strip was drawn almost whole at every tick of QUALITY's opening.** The panel's change,
  the opening at its top right, reached the strip in the strip's rows (above its top, so negative)
  and was put in one rectangle with the display's window before it was cut to the strip: from the
  strip's top to the window's foot and nearly its width, 3389 by 813 pixels at every tick, 10 to
  18 ms a frame. The change is now cut to the strip first (`StripRenderer::render_at`), and the
  editor gives the strip only the panel's places that reach it (`draw`).
- **The frame was put together in rows, not places.** The rows between the opening at the top and
  the drops at the foot were most of the frame. It is now put together in the panel's places and
  the strip's alone (`compose`, `paste`), and those places are the frame's damage
  (`Editing::damage`; none, the whole frame, while the drawer moves, something floats over
  everything, or the frame is new or another size).
- **Shown where it changed** (the note's first half). `Surface::present` takes the damage: where
  softbuffer's buffer still holds the frame shown last (its age 1, Windows' GDI and X11's buffers,
  kept), only the places are turned (`fill_places`) and sent (`present_with_damage`); else the
  whole frame, as before: always on macOS, where softbuffer's Core Graphics backend makes a new
  buffer each frame and ignores damage; after the surface is made or resized or a frame fails
  (`whole`); on Windows' repaint (`WindowEvent::Damaged`); and on X11's repaint every 250 ms (R19),
  which may have lost any of it.
- **The lamps' light as pictures** (the note's second half). The filament's flare, the halo past
  the ring and the spill on the panel were SVG gradients drawn afresh whenever the synth's level
  moved. Each is now a picture made once a scale at its strongest in the opening's layer's own
  pixels (`ultra::Glow`, `Renderer::lights`) and drawn with its strength as its opacity
  (`art::Light`, a layer's `lights`, over its pictures and under its SVG), the flare scaled about
  the middle while its lamp comes up. The Tesla lamp's flash is a second spill over the steady
  one, not a gradient in the streaks' SVG. The amber lamp lit and playing has no SVG left to parse.
  Each light is within three levels of 255 of the light drawn afresh (each rounded once more; the
  flash over the steady spill, twice).

**Evidence (the Linux reference machine, 2026-10-11; load 11 to 19 from other sessions' builds):**
`the_window_played`, its thread's own time, main at 66f8ef7 and this, run alternately twice: before
40.7 % and 51.0 % of one core (6.8 and 8.5 ms a frame on average, 21 ms at the 95th percentile);
after, turned in places (Windows, X11) 21.7 % and 22.4 % (3.6 and 3.7 ms, 14 ms), turned whole
(macOS) 37.3 % and 38.3 % (6.2 and 6.4 ms, 16 ms), every frame's places-turned screen the whole one.
The same on the owner's Mac (idle; its wall's time), before and after each in its own target folder
(in one, Cargo took one tree's build as the other's: path packages are named from the workspace's
root, and the second tree's files were older than the first's build): before 36.9 % and 36.3 % (6.2
and 6.0 ms a frame, 16 ms at the 95th percentile), after, turned whole as macOS does 29.4 % and
30.1 % (4.9 and 5.0 ms, 12 ms); turned in places 21.6 % and 21.9 %, which macOS cannot yet do.
`ultra_timings`, the opening alone lit and playing (wall's time), at the opening size and a Retina
window's: the amber lamp 0.53 and 1.36 ms a frame before, 0.33 and 1.07 after; the Tesla lamp 1.43
and 2.14 before, 0.95 and 1.88 after. Tests: `a_change_above_the_strip_draws_none_of_it` (it fails
without the cut: the strip drawn from its top), `a_light_from_its_picture_is_the_light_drawn` (both
lamps, quiet, playing, sparking and coming up, at both scales),
`the_frame_put_together_in_places_is_the_frame_put_together_whole` (now with the opening ticking,
the strip and OVERLOAD at once, and the window turned in those places the window turned whole: it
fails with the strip's place left out). The core's, panel's and plug-in's tests, 341 passed, 43
ignored; rustfmt; clippy with `-D warnings`.

**Not done, measured, for later:** macOS still turns and shows the whole window at every frame
that changes (about 2.5 ms a frame here, 1.9 on the Mac, and Core Animation takes the whole image
again); a patched Core Graphics backend that kept its buffer, or showed the window in bands,
would be the next step there. The shutter's blades are SVG, about 6 ms a frame while QUALITY
changes; the Tesla lamp's streaks about 3 ms; a strip knob (DETUNE swept) about 2 ms a render; the
lamps' lens and bezel pictures are resampled at each tick.

## R59. The costs R58 left: macOS shown in tiles, the shutter, the lightning, knobs

**The owner, 2026-10-11,** told what R58 left ("Still costly, for later": macOS turning and
showing the whole window each frame, the shutter's blades, the Tesla lamp's lightning, a knob
redrawn, the lamps' pictures resampled each tick): "okay, let's improve those costly items".

**Agent decisions, 2026-10-11** (each measured first, at a Retina window's scale):
- **macOS shows the window in tiles, its buffer kept** (softbuffer's second change,
  `third_party/softbuffer/PATCHES.md`). softbuffer's Core Graphics backend made a new buffer each
  frame and gave Core Animation one image of all of it, damage or not; so the editor turned every
  pixel each frame and Core Animation took the whole image again. Now the buffer is kept (`age`
  1, as on Windows and X11), the layer shows it through tiles of 256 pixels (whole points at a
  scale factor of 1 or 2: no seam), and a present gives new images (copies of their pixels) to
  the tiles its damage touches alone. On macOS the editor so takes R58's way: the places
  changed turned and sent.
- **Pictures placed in layers are kept** (`Renderer::placed`): a picture turned, mirrored,
  scaled and moved by fractions of a pixel is resampled once onto a pixmap just round it and put
  from there; placed so again (the lamps' lens and bezel at rest, a knob's cap, the hamster's
  frames), it is put without being resampled. Always put from its own pixmap, kept or not, so a
  frame is the same whatever was kept; the least lately used given up past 32 MB.
- **What does not change over a layer is drawn once** (`art::Overlay`): a knob's and a
  selector's lights (the lamp's, dodged lighter on the near side and multiplied darker on the
  far, and the cap's) are drawn once in the layer's place and put over its turning picture
  with their blend, as SVG's `mix-blend-mode` put them; no SVG is parsed or painted as a knob
  turns.
- **QUALITY's opening is drawn by code** (`ultra::Drawn`): the shade over the hamster deep in
  the well, the shutter and the Tesla lamp's lightning, no SVG. The shutter's eight blades are
  put together a pixel at a time from the last blade that covers a pixel wholly (as SVG, eight
  sheens over eight blades were filled one over another, each mostly hidden by the next
  blade's metal), the whole cut to the opening once as SVG's clip did; its circles are made as
  usvg makes SVG's, four quarter turns, so that their edges cover pixels alike. The streaks are
  stroked straight from their corners (`Streaks`). The SVG versions stay as the tests'
  reference (and the shutter's for the drawn skin).
- **Pixmaps neither scaled nor turned are put by a loop of our own** (`render::put`), not
  tiny-skia's `draw_pixmap`, which samples them as a pattern in floats (highp): the kept
  pictures, the lights at their size, the overlays put normally, the renderer's recompose and
  the shutter. The same to the bit at full opacity, within a level otherwise.
- **The drops cost less** (not on the list; the Mac's sampling found them the largest left):
  their fields summed in single precision, apart for the sounding and the idle (a loop the
  compiler makes vector code of), and only the part of the display they reach looked at again.

**Evidence (2026-10-11):** `costs_timings` (Linux, best of five runs of 100, as drawn before
and now): the shutter half closed 3.78 and 1.19 ms; the Tesla lamp sparking 1.56 and 0.97; the
amber lamp lit 0.45 and 0.25 (1.07 after R58); a knob turned 1.52 and 0.30.
`drops_timings`, twenty drops moving: 1.96 and 1.17 ms a frame (0.46 and 0.29 at the opening
size). `put` against `draw_pixmap`, a 196-pixel square: on the Mac 11 against 104 µs, on Linux
91 against 148. On the Mac (`softbuffer_copies`, `CA72_TIMINGS=1`), a present of the whole
3441 by 1965 window 1.28 ms, of a frame's usual damage (the opening and the display) 0.08 ms.
`the_window_played`, main at bd4e866 and this, alternately twice: on Linux (its thread's time,
load 3 to 6) turned in places 13.6 % and 15.2 % of one core before, 11.0 % and 10.9 % after;
turned whole 22.7 % and 25.3 % before, 19.8 % and 19.6 % after. So on macOS, which turned the
whole window before and now turns the places, about 23 % becomes about 11 % here, less its
presents. On the Mac (wall time, at the lowest priority while the owner used it: no more than a
direction) main turned whole 29.7 % and 30.5 %, this branch in places 22.0 % and 18.8 %
(measured before the drops' change). Tests: `the_opening_drawn_by_code_is_the_opening_drawn`
(closing, opening, nearly open, over the lamp and the hamster, sparking soft and loud, both
scales: within a quarter of a level on the whole, under half a percent past three levels, none
past 16), `a_light_from_its_picture_is_the_light_drawn` (the lights alone now), the knob's
light test as before, and on macOS `softbuffer_copies`: a frame shown in tiles drawn by Core
Animation (offscreen, `renderInContext:`) the same as the frame at every pixel, a change of one
pixel giving its tile alone a new image and drawn so, the buffer kept and made again on a
resize (it fails with tiles placed upside down, or their images). The core's, panel's and
plug-in's tests, 342 passed, 44 ignored; rustfmt; clippy with `-D warnings`; clippy on macOS.

**Not done:** the editor in a host on macOS with the tiles, by eye (the Mac was in use);
Windows not tried again (nothing of its own changed but the drawing). Left: pictures that turn
each frame (the hamster's wheel, a knob dragged) are resampled each frame, as they must be.

## R60. The checks before a release, on the three computers (2026-10-11)

**The owner, 2026-10-11:** "are there any more tests you need to run before release? If so, run
them now and fix any issues. You have access to all 3 computers."

**What was run.** CI's job by hand on each computer, from a clean export of main (9c01641) in a
target folder of its own (main has not been on GitHub since 0.1.0, so CI had not run): the
bundles (universal on macOS, with the Audio Unit), the installer (`scripts/package.sh`; macOS
unsigned), the notices' check (Linux), clippy over the workspace with every feature, the
standalone, every test (Linux with ngspice 47 required), and `scripts/validate.sh` (its editor
tests included: Linux on XWayland, macOS in the desktop session; Windows' pluginval run there
separately). Then the editor in hosts, R58's and R59's ways of drawing in each:
REAPER 7 (VST3 and CLAP) and Cubase 15 (VST3) on Windows; GarageBand 10.4 (Audio Unit) and
Bitwig 5.2 (CLAP) on macOS; REAPER 7 (VST3) on Linux. Each host with a renamed build (CA-72
TEST, or CA-72 PRIORITY in the Windows harness), the owner's installed CA-72 not touched; Live,
open with the owner's set, was left alone.

**Found and fixed:**
- **macOS: the window drawn in rows bottom to top** (GarageBand: the strip and the rail above
  the panel). R59's tiles sit in softbuffer's layer, which upstream always flips; in a
  flipped view's layer (baseview's, every plug-in editor's) the two flips undid each other.
  The layer is now flipped only where the layers over it leave the coordinates running up
  (softbuffer's PATCHES.md, change 2). R59's test drew softbuffer's layer alone, in a view
  of AppKit's own, which hid it: it now draws the view as AppKit does, a flipped one too (and
  fails without the fix). Seen right in GarageBand and Bitwig afterwards.
- **Windows: a press with no move of the pointer before it was taken where the pointer had
  last been** (Cubase: the harness's clicks on POLY, QUALITY and CUTOFF did nothing, the next
  did). Change 4 had told the handler the press's place on macOS alone, taking Windows to send
  a move first; it does not always. baseview's PATCHES.md change 5 does the same on Windows;
  a test presses POLY with no move before it (and fails without it). In Cubase afterwards each
  first click took.
- **The Windows editor's own window tests had not built since the A6 drawer** (they asked for
  the drawer's growing the window, gone; clippy and the tests failed on Windows). Brought to
  the drawer dropping over the strip; and one more: the synth played, a knob turned, the drawer
  opened and shut, then every pixel of the window as when Windows has it all repainted
  (it fails when a place is left out of a present).
- **The notices' check failed** (plugin-kit-materials, GPL, ships no licence file): the look's
  branch had left the kit's crates out of the notices, Idle Foundry's own code under the same
  licence, as the CA-74's do (8da3aea), but that commit had not reached main. Its change taken
  over, the notices made again (and the branch merged in as the records went to GitHub: R44 to
  R49's numbers, R43 before them, the kit at its main).

**Passed (the final commit's runs are in the commit's message):** the Linux tests (398 then),
the Windows tests in the desktop session (402, and the window tests at the final commit),
clippy on all three, the standalone, the installers, the three validators on Linux and the
Mac's VST3 (pluginval at strictness 10 with its editor tests, clap-validator, Steinberg's
validator). In each host: the editor drawn whole and right, the synth played (REAPER; the
drops and the lamp), QUALITY switched (the shutter, the lamps, the hamster), a knob dragged, the
drawer, tips, ULTRA's note, typing into DRIVE's readout in Cubase; and once still, the window as
drawn in places the same as the window drawn whole (REAPER both formats and Cubase: moved off
the screen and back, Windows repainting it; GarageBand: closed and opened again).

**Not done, or not passing, and why:**
- auval and pluginval on the Audio Unit on this Mac: its registry of Audio Units lists none but
  Apple's (not the owner's Youlean meter either), so neither finds the CA-72; restarting the
  registrar did not help. GarageBand finds it and passed. CI runs auval on a clean Mac.
- Three of the circuit lab's tests (`vco_electrical` twice, `vco_osc23`) fail on this Mac with
  Homebrew's ngspice 47, built with KLU: "Timestep too small" at the VCO's hold node. They pass
  with Linux's ngspice 47 (and on Windows); CI's Mac has no ngspice and skips them. The lab is
  not part of the plug-in; `.options sparse` did not help.
- The synth played in the Mac's hosts: GarageBand took no MIDI from a script (its virtual input
  or a new one), so the opening's animation there was QUALITY's, driven by the editor's clock.
- Live (the owner's set open), Logic (not owned), an Intel Mac, a second screen of another
  scale (the tiles are made again when the scale changes; not seen).

**At publication** (the owner: "Yes", to main going to GitHub): main merged with the look's
branch (pull request 25: R44 to R49's numbers, R43 before them, the kit at its main 4c0f0f3,
the same tree), the records since numbered R50 to R60 the same way, a private detail of one
record (the name and folders of a set of the owner's) edited out, and the README's picture of
the panel (`docs/panel.png`) drawn again as the window opens (`the_window_png`), now with NOTE
PRIORITY, RETRIGGER and the settings key.

**Afterwards (2026-10-11), from porting change 5 into plugin-kit (the kit's K9):** an independent
review there found baseview's count of held buttons read before the press's new `CursorMoved`
call and written after it, so a press or release let through by a modal loop run from the
handler would be lost. The count is now read after the call (baseview's `PATCHES.md`, change 5),
in both copies alike. With it, the seven Windows window tests passed in the desktop session (an
export of this main with the kit's baseview over its own), the press test failed without change
5, and clippy over the workspace passed on Windows.

## R61. Escape reaches the editor in REAPER's CLAP on Windows

**The owner, 2026-10-11:** "merge anything into main that should be merged. Just don't push
anything public", while the CA-74's lead, porting its own fix into plugin-kit (the kit's K10),
found by reading this repository that the CA-72 has the same fault.

**Agent decisions, 2026-10-11:**
- **The fault:** in REAPER 7.82 on Windows, the CLAP plug-in's window lost Escape while the
  editor held the keyboard: typing a value into a readout or into the drawer's FIND, Escape
  closed REAPER's FX window instead of cancelling. REAPER's CLAP window handles Escape before it
  asks the window what keys it wants (baseview's change 3, R34) and before dispatching it, so
  that answer could not help.
- **Taken from plugin-kit as the kit has it** (baseview's `PATCHES.md`, change 6, the CA-74's):
  a `WH_GETMESSAGE` hook for each library's GUI thread (`src/win/escape.rs`) hands an Escape
  just removed from the queue to the focused editor while it holds the keys, after putting a
  `WM_NULL` in its place. Idle Escape, other keys, peeks and other windows go as before. The
  CA-72's baseview is the kit's, file for file.

**Evidence (2026-10-11, Windows, MSVC 1.97.1):** baseview's three tests of it pass in the
desktop session, and two fail with the hook's registration taken out; the CA-72's seven window
tests pass; clippy over the workspace with every feature passes. In REAPER 7.82, CLAP, a renamed
build of this tree (CA-72 PRIORITY, its own REAPER settings): LEVEL's readout double-clicked,
"-6" typed, Escape: back to 0.0, the FX window open and in front; the drawer opened, "lead"
typed into FIND (the list narrowed), Escape: the drawer shut, the window open; Escape with
nothing held: the FX window closed, as REAPER does. The same build of the commit before
(without change 6): "-6" typed, Escape closed the FX window. Not tried: the VST3 in REAPER, and
other hosts on Windows (the kit's K10 tried REAPER's VST3 and Cubase 15 with the CA-74).

## R62. SCATTER in MONO: the one voice at the next of VOICES's places at each attack

**The owner, 2026-10-11:** "what if me made "scatter" an option in mono? So I would imagine
something like a plucked voice, which we would want to be monophonic, um, would have scat- when
scatter gets applied to it, it would jump around uh, seemingly at random um, to the listener,
but there would actually be it would, it would actually be uh, a pattern according to how it was
played, I assume. Uh, so the order that the notes were played, perhaps, uh, that they they move
around um, up to as many voices as, as there are." Told that with POLY and UNISON off, WIDTH,
INNER, VOICES and the placement moved nothing (the voice in the centre) though the switch reads
SCATTER, so that no new control was needed; that it costs nothing, the one voice only placed
anew; that CENTER's places, by the golden ratio, sound random and repeat, and EVEN's go from side
to side; that the voice should move only at a new attack, a legato phrase staying where it began
and RETRIGGER moving every key; that one voice moved, not a voice a note, keeps MONO (a new key
takes the one voice's pitch at once, so nothing of the note before is left behind); that Wooden
Mallet (MONO, WIDTH 60, which did nothing) and sessions saved with MONO and WIDTH up would start
to scatter; and that moving over the places' 10 ms at an attack might smear it: "this sounds
great. make it".

**Agent decisions, 2026-10-11** (not separately approved):
- **When the voice moves:** when its trigger contact closes, the start of an attack, which the
  contours follow (`Voice::trigger_closed` and `Light::trigger_closed`: the keys' gate each model
  already kept): a key pressed with none held, or with RETRIGGER a key coming down after the
  contact's 13 ms open (R50). A key pressed while another is held (single triggering) moves
  nothing, nor does a key let go. A key pressed again within the re-arming time after the last
  was let go (about 12 ms) closes the contact and moves the voice, though the circuit may not
  start its contours again.
- **Where:** VOICES's places as POLY's voices 0, 1, 2 ... have them by the placement, WIDTH and
  INNER (the kit's `voice_gains`), the next at each closing, round and round (`Playing::turn`);
  the first attack at the first place (EVEN's left edge; CENTER's centre or, with INNER, the side
  its turn falls on). VOICES lowered meanwhile takes the turn round its own count.
- **Starting again:** All Notes Off and the host's reset (`Engine::release_all`) start the
  pattern again at the first place, so that a part played from its start after either places its
  notes alike each time.
- **How fast:** the one voice's place glides with a time constant of a fifth of `TRIGGER_ON`
  (`TURN_GLIDE`, 1.56 ms), its knobs' moves too, not the kit's 10 ms (`GLIDE`): its contours
  start 7.8 ms after the contact closes (the light model's `TRIGGER_ON`, fitted to the
  circuit's), by when the voice is within 0.7 % of its new place; with 10 ms it would be 46 %
  short, and the attack heard sliding from the place before. What sounds in those 7.8 ms is the
  note before's tail, already at the new key's pitch, and it moves with it.
- **DOUBLE in MONO as before:** the one pair all of WIDTH's way out, whatever VOICES and the
  placement; a twin let go as DOUBLE is switched off plays its tail out where it was, and the
  voice glides to its first place.
- **The strip:** in MONO without DOUBLE, VOICES's readout is lit, WIDTH's and INNER's where they
  are up, and the placement's tab; WHERE THE VOICES SOUND shows VOICES's places with the one
  voice lit at its last attack's (the engine's bit for it, `Engine::sounding_mask`). With DOUBLE
  as before: VOICES and the placement unlit. The picture example's MONO strip likewise.
- **No new parameter, and nothing kept for what was saved before:** a session or preset with
  POLY, UNISON and DOUBLE off and WIDTH above 0 now scatters. Of the factory presets only Wooden
  Mallet (MONO, WIDTH 60, VOICES 8, EVEN) does, and it is kept so, scattering, its loudness the
  same (below).

**The owner, 2026-10-11**, asked whether sessions saved before should open as they sounded
(recommended: leave them, WIDTH having done nothing there), whether Wooden Mallet should keep
scattering, and whether the CA-74 should have it too: "okay. what presets would that be?", "ok,
sounds good" and "yes". Of the presets in the Mac's library only Wooden Mallet (the factory's and
the owner's copy) scatters; Hollow Glider, Noise Crash, Noise Snare and Slow Bow there are MONO
with WIDTH up but DOUBLE on, and play as before; the retired presets' pack (R55) holds none yet.

**Evidence (the Linux reference machine, 2026-10-11):** tests:
`the_one_instrument_takes_the_next_place_at_each_attack` (seven detached notes, each one's left
and right power at the gains of place `i` mod VOICES within 1e-6: EVEN with four voices, CENTER
with five and INNER, and EVEN at LO; and the editor's bit for each),
`a_legato_note_stays_and_retrigger_moves_it`, `all_notes_off_starts_the_pattern_again`,
`the_one_instrument_is_at_its_new_place_before_its_contours_start` (0.7 % short), and the
strip's `mono_shows_its_places_and_the_one_it_sounds_at` and
`the_strip_lights_the_placements_tab`; each seen to fail with the code broken (the kit's glide,
no turn, no restart, every key moving, the light voice not turning, the strip's old lights).
Tests that held the one voice in the centre with WIDTH up now hold it there at WIDTH 0
(`a_chord_fills_the_field_and_one_voice_without_spread_stays_in_the_centre`,
`inner_clears_the_centre_and_one_voice_stays_there`) or at its first place
(`a_twin_let_go_plays_its_tail_out_and_takes_no_new_key`; `tests/sound.rs`'s
`poly_plays_a_chord_and_spread_parts_it`: the same sound at EVEN's left edge, its left side √2
times the centre's within 1e-5 of the peak, its right silent). `preset_levels` (each factory
preset as it is set, on its phrase, written out) against main (bc5de77): 25 of the 26 the same to
the bit; Wooden Mallet scattering, its momentary maximum and integrated loudness the same to the
hundredth of a dB (-22.80 and -24.11 LUFS). `preset_render` (every factory preset on POLY's
ten voices) against main: all 26 the same to the bit. The plug-in's and the panel's tests, 266
passed, 32 ignored (the new ones in a debug build too); rustfmt; clippy with `-D warnings`. On
the Mac, CA-72 TEST built from this branch: clap-validator (37 passed), the VST3 SDK's validator
(47 passed) and pluginval at strictness 5 pass. Not yet: the installed build in a host.

## R63. 0.2.0's factory presets: the owner's favourites, 0.1's renamed

**The owner, 2026-10-11:** "for 0.2.0 I only want the presets I currently have favorited. Any
preset that i have favorited that existed in 0.1.5 needs to be renamed. Others can keep their
current name."

**Agent decisions, 2026-10-11** (not separately approved):
- **The favourites as the drawer shows them**, read from the owner's library on the Mac at
  19:46 (EDT; Wooden Mallet starred at 19:42 and saved again at 19:44, after a first reading
  at 19:29): the stars in
  `factory.toml` and in their own files, a factory preset with an edit of theirs counted by the
  edit's. Nineteen; not kept: Lead, Elastic Octaves, Undertow Growl, Cruising Whistle, Stacked
  Fifths, Ringing Saw Line, Shoreline Wash.
- **Each as the owner has it.** Thirteen are their edits, their values whole (every control
  named, but MIDI BEND RANGE, the player's, which Wooden Mallet's file of 2026-10-03 held):
  Wooden Mallet (RETRIGGER on, SPREAD 100, VOLUME 10), and, saved that day, Brass Tutti, Breath Flute, Hollow Glider, Ladder Kick, Noise Crash,
  Noise Snare, Pink Riser, Slow Bow, Slow Horn Swell, Upright Pluck and Warped Pad (DOUBLE on
  most, DRIVE, ENTROPY and SPREAD, UNISON, an octave, NOTE PRIORITY LAST, RETRIGGER), and Cycle
  of Fifths (four voices, DOUBLE 30 cents, oscillator 3 and FILTER MODULATION off). Six are the
  factory's as they were: Bass, Three Saw Slab, Pulse Strut, Closed Hat, Open Hat, White Lotus.
  Their descriptions say what the edits changed; their tags are the owner's.
- **The seventeen from 0.1.5 renamed** (names proposed by the agent, the owner's to change):
  Bass Felt Bass, Three Saw Slab Saw Monolith, Pulse Strut Feedback Funk, Upright Pluck Tenor
  Pluck, Hollow Glider Singing Drift, Slow Horn Swell Horn Bloom, Brass Tutti Brass Choir,
  Breath Flute Smoke Flute, Wooden Mallet Bright Xylo, Slow Bow Wide Bow, Ladder Kick Driven Kick, Noise Snare Split
  Snare, Closed Hat Tight Hat, Open Hat Loose Hat, Noise Crash Wide Crash, Pink Riser Swarm
  Riser, Warped Pad Warp Field. White Lotus and Cycle of Fifths keep theirs.
- **All twenty-four of 0.1.5's presets moved whole to `retired.toml`** (R55), the seventeen kept
  under new names among them, so the pack is 0.1's whole set at its own names, and no retired
  name is a factory preset's.
- **Levelled at -18 LUFS momentary maximum** on each one's phrase (`tests/preset_levels.rs`), as
  R15: MAIN OUTPUT VOLUME first (Feedback Funk 8.96 to 8.711, Singing Drift 6.04 to 9.105, Horn
  Bloom 7.0174 to 7.248, Brass Choir 3.894 to 6.585, Wide Bow 8.23 to 9.456, Wide Crash 8.0716
  to 9.809, Cycle of Fifths 7.2399 to 8.044; Warp Field and White Lotus left). **The owner,
  2026-10-11**, asked whether LEVEL should bring up the ten still under at VOLUME 10 (R15 had left
  a preset at 10 under its level): "do what you have to to level the volumes. i am done starring
  things." So LEVEL, the plug-in's output gain after MAIN OUTPUT, by each one's shortfall: Felt
  Bass +2.57 dB, Saw Monolith +2.41, Smoke Flute +6.85, Bright Xylo +4.46, Driven Kick +11.32,
  Split Snare +1.24, Tight Hat +8.19, Loose Hat +3.97, Swarm Riser +4.65; every one at -18.00,
  the highest peak Tight Hat's, -2.7 dBFS. **Tenor Pluck** reached -21.18 with its oscillator,
  VOLUME and LEVEL at their tops (LEVEL's +12 dB); offered DRIVE with AUTO GAIN off for the rest,
  the owner: "add 3db of drive with autogain off." DRIVE 3 dB, AUTO GAIN off: -18.20, its peak
  -4.8 dBFS.
- **Sessions keep their sound** (R55). `drive::AVERAGE`, the factory presets' average DRIVE
  curve that a session without a measurement of its own takes, is left as it was, so no
  session's AUTO GAIN changes.
- **The owner's own library** is left as it is: in 0.2.0 their thirteen edits list as their own
  presets beside the factory's of the same sound, and the stars keyed by 0.1's names mark
  nothing; theirs to tidy.
- **The tests** that named 0.1's presets name 0.2.0's, or take a retired one by name through
  `tests/common`'s `preset` (the factory's, else `retired.toml`'s): the measurements made on
  Lead, Pulse Strut, Undertow Growl, Stacked Fifths and Brass Tutti keep them. The editor's
  scrolling tests add twelve presets of the user's, so that the list runs below the rows
  shown whatever the factory's count. `tests/common`'s `controls_of` takes NOTE PRIORITY and
  RETRIGGER, which no factory preset had named.

**Evidence (the Linux reference machine, 2026-10-11, on R62):** the owner's library read again at 19:54, unchanged
since 19:46. `cargo test -p ca72-plugin`: the unit tests and every integration test pass;
clippy clean. Every preset's level on its phrase after levelling (momentary maximum, LUFS; peak,
dBFS): Felt Bass -18.00 (-5.0), Saw Monolith -18.00 (-4.2), Feedback Funk -18.05 (-11.8), Tenor
Pluck -18.20 (-4.8; DRIVE 3 dB, AUTO GAIN off), Singing Drift -17.97 (-14.1), Horn Bloom -18.06 (-10.4), Brass Choir
-18.02 (-9.3), Smoke Flute -18.00 (-11.0), Bright Xylo -18.00 (-6.0), Wide Bow -18.09 (-12.5),
Driven Kick -18.00 (-7.2), Split Snare -18.00 (-6.0), Tight Hat -18.00 (-2.7), Loose Hat -18.00
(-6.3), Wide Crash -17.88 (-10.5), Swarm Riser -18.00 (-12.0), Warp Field -18.09 (-9.4), White
Lotus -18.03 (-12.4), Cycle of Fifths -18.15 (-9.7). R62 (SCATTER in MONO) moved none of them.

**The owner's last changes, 2026-10-12:** "i made a few updates to the presets. They are now as I
want them for release. Please make sure to grab my latest changes." Read from the owner's library
on the Mac (08:29 EDT; five edits, nothing renamed, deleted or starred) and taken whole: Bright Xylo
an octave lower (oscillator 1 at 4', 2 and 3 at 16') with NOTE PRIORITY LAST; Split Snare, Tight
Hat, Loose Hat and Wide Crash with NOTE PRIORITY LAST and RETRIGGER on. Their descriptions say so
(the agent's). Bright Xylo levelled again by LEVEL, 4.46 to 5.71 dB: -18.00 LUFS momentary
maximum, its peak -6.3 dBFS; the other four measured where they were (-18.00, -18.00, -18.00,
-17.88). Bright Xylo costs 7.7 % of a core.

## R64. VOICES dimmed where it does nothing

**The owner, 2026-10-11:** "something like the driven kick says it has 4 voices, but i don't
think those 4 voics do anything do they? if so, find all such examples and reduce their voice to
1", then "when something is polyphonic, this doesn't apply. i only mean with mono patches
specifically". Told that VOICES goes from 2 to 10 (R7) and offered three ways (VOICES down to
1; the presets at 2; VOICES dimmed wherever it does nothing, every sound's and not only the
presets'), with dimming recommended: "ya, i think you're right. do it".

**Agent decisions, 2026-10-11** (not separately approved):
- **Where VOICES does nothing** (`editor::voices_inert`): MONO, neither POLY nor UNISON, with
  DOUBLE on (the one pair has no place among others: R62) or with WIDTH at 0 (every one of
  VOICES's places the centre). Found by the engine's code (`mix_of`, `places`) and by
  rendering each factory preset's phrase at its VOICES and at 2: to the bit the same in all
  twelve MONO presets without SCATTER's spread (Felt Bass, Saw Monolith, Feedback Funk, Tenor
  Pluck, Singing Drift, Wide Bow, Driven Kick, Split Snare, Tight Hat, Loose Hat, Wide Crash,
  White Lotus); different in Bright Xylo (MONO, SCATTER, WIDTH 100: its 8 places), and in
  Smoke Flute and Swarm Riser (UNISON).
- **Shown as DETUNE is in SCATTER:** its knob dimmed (the realistic look shades it, the drawn
  one draws it at half opacity: `Scene::voices_off`), not operable (no press, drag, wheel or
  step), its readout unlit (R62 had unlit it with DOUBLE only) and not typed into. Its value
  is kept, and the host, a learned controller and a preset still set it: it counts again as
  POLY, UNISON or WIDTH make it. The presets keep their VOICES (4 in most MONO ones), so
  switching POLY on gives the voices it gave before.
- The editor's tests that work VOICES at the defaults (MONO, SCATTER, WIDTH 0) do so with POLY
  on (`editing_poly`); `voices_is_dimmed_where_it_does_nothing` holds the rule in each mode and
  the editor's refusals. `the_window_voices_pngs` (by hand, `$CA72_VOICES_PNG`) draws it dimmed
  and lit in both looks.

## R65. 0.2.0

**The owner, 2026-10-11:** "I think we can delete the CA-72 TEST and install 0.2.0 locally for
pre-release. I'm going to test it with the assumption it's our release candidate."

**Agent decisions, 2026-10-11** (not separately approved):
- **0.2.0, a minor version**: the factory presets change (R63, 0.1's in the pack: R55) and a
  MONO session with WIDTH up now scatters (R62), where a patch release changes nothing a
  project or preset holds.
- **The release candidate** is main at this version built as the release is, on the Mac (in
  the agents' build tree, the Mac having 5.9 GB free): the universal bundles, the Audio Unit, the validators, and
  `scripts/package.sh` with the Developer ID identities and notarisation required
  (`docs/macos-release.md`). It is installed by its installer over 0.1.x, as a user installs it;
  the CA-72 TEST bundles and the 0.1.x Audio Unit in the user's own Components folder go to the
  Trash first, so that no other copy stands in for it in a host.
- **Left for the release:** CI on the three systems (main has not run there since PR #25), the
  README's Status line and the release notes, the tag, the Windows and Linux installers and the
  0.1 presets' pack (CI's jobs), publishing.

**Evidence (the Mac, 2026-10-11):** main 5ec29af (tree b6d7b7146f233970e6d3a788e5fc0a4116d15de5),
exported into `~/Code/ca72-a6-build/ca-72` and built with the 1.97.1 toolchain: VST3 and CLAP
`x86_64 arm64`, the Audio Unit 0.2.0; clap-validator 0.4.1 37 passed, 0 failed, 7 skipped;
Steinberg's VST3 validator 47 passed; pluginval 1.0.4 at strictness 5 (GUI tests skipped)
SUCCESS. Signed and notarised in a session of the desktop's tmux server (the hidden Ghostty
shell 0.1.5 used started no command this time): Developer ID Application and Installer,
Accepted, stapled, `spctl` "Notarized Developer ID". auval not run: the tmux session sees no
third-party Audio Unit. `CA-72-0.2.0-macOS.pkg`, SHA-256
90eed1dd78670853114f4964c72fb426ed0ff69053d5dde689d879f2d7c7e22b, with the scripts and logs in
`~/lab/artifacts/ca-72/0.2.0-rc/`. Live quit (asked to, the owner's set offered for saving);
CA-72 TEST and the 0.1 Audio Unit in `~/Library/Audio/Plug-Ins/Components` to the Trash; the
installer run by the owner over 0.1.x: the three bundles in `/Library/Audio/Plug-Ins` 0.2.0,
signed Developer ID Application.

## R66. Sessions saved by 0.1 translated into 0.2.0's calibration

**The owner, 2026-10-11**, told that 0.1.5's sounds played through 0.2.0 came out up to 7 dB
from where they were (R42's calibration, R67 below): "For 0.1 projects, I thought we were able to
translate into the new calibration." Told what could be (the knobs whose laws changed) and what
could not (the circuit's parts): "Okay, let's make sure we can translate, and not worry about the
things we can't."

**Agent decisions, 2026-10-11** (not separately approved):
- **The knobs whose laws changed since 0.1.5** (`compat::CHANGED`): the mixer's four VOLUMEs
  (a straight track, now `volume_track`), EXTERNAL INPUT VOLUME (the drawing's audio taper, now
  `ext_taper`), CUTOFF, AMOUNT OF CONTOUR and the two SUSTAINs (straight, now their measured
  tracks), EMPHASIS (`emphasis_r14_drawn`, now `emphasis_r14`), the four ATTACK and DECAY pots
  and GLIDE (the drawing's tapers, now `time_pot` and `glide_pot` on the reference's points),
  and the MODULATION wheel (`mod_wheel_r_drawn`, now `mod_wheel_r`). Each moved to the travel at
  which 0.2.0's law sets what 0.1's set at the travel it held (a bisection on the law, exact to
  a part in a million of its range); where 0.2.0's law does not reach (the time pots' and the
  wheel's tops, lower now), the knob's end.
- **When:** in `Plugin::filter_state`, for a state whose `version` (nih-plug's, the plug-in's
  version that saved it) is 0.1.x, or empty (the oldest). Test builds saved 0.1.5 with the new
  calibration, but under names and IDs of their own, which a CA-72 never loads.
- **What 0.1 did not have** is set to its default, as 0.1 sounded, rather than kept from the
  instance: DRIVE, AUTO GAIN, INNER, UNISON, DETUNE, FILTER MODE (LO) and the placement (EVEN);
  QUALITY, the computer's, is left. **LEVEL -0.5 dB** (`compat::LEVEL_FROM_0_1`): R42's VCA trims
  made the voice 0.2 to 0.6 dB louder, and 0.1.5's presets, translated, played 0.53 dB louder
  at the median.
- **Not translated**, being the circuit's parts or the keyboard's: each contour's R1401, the
  contours' capacitors, peak diodes and release, KEYBOARD CONTROL's R53 and R54, RANGE's R39,
  oscillator 3's R162, the external input's R61 and R_EXT, the VCA's trims, Q12's gain, the
  filter's calibrated trims and the keyboard's 0 V on C2. Nor the presets the user saved with 0.1:
  their files carry no version (format 1 is 0.2.0's too).
- **The 0.1 presets' pack** (R55) made the same way from 0.1.5's own factory file
  (`examples/translate_0_1.rs`), in place of the presets as 0.2.0 had levelled them, so that it
  holds 0.1's sounds; Cruising Whistle's and Ladder Kick's CUTOFF kept as R42 retuned them (their
  filter's pitch, which a part moved), and Stacked Fifths a true fourth and fifth (the owner's,
  R54).

**Evidence (the Linux reference machine, 2026-10-11):** `tests/preset_render.rs` (ten POLY voices,
4 s) with 0.1.5's factory file (`CA72_PRESETS_FILE`) against 0.1.5's own renders (a worktree of
v0.1.5, `CA72_RENDER_OUT`). Untranslated, the level against 0.1.5's from -3.95 dB (Three Saw
Slab) to +6.92 (Pink Riser), nine of twenty-four within 1 dB. Translated (`CA72_FROM_0_1`) with
LEVEL -0.5 dB: seventeen within 1 dB, the median +0.03; the rest Undertow Growl +3.67 (its
FEEDBACK loop chaotic), Wooden Mallet +2.28, Slow Bow -1.84, Warped Pad +1.43, Ladder Kick +1.42,
Three Saw Slab -1.24, Brass Tutti +1.20. The pack as made: nineteen within 1 dB, Cruising
Whistle -0.02 dB (its whistle's band -8 dB from 0.1.5's, -35 untranslated) and Ladder Kick
+0.09. Tests: `compat`'s five (each knob sets what it set, ends and order kept, versions, the
additions' defaults), and `a_session_of_0_1_opens_translated_and_one_of_0_2_as_saved`.

## R67. The checks before 0.2.0's release

**The owner, 2026-10-11:** "have we completed all tests for release with this build?" Told not
(R60's checks had run on main before R61 to R65), with a list: "Yes, please do that."

**Found:**
- **The POLY presets with DOUBLE cost twice 0.1.5's** (`tests/preset_cost.rs`, new
  `CA72_AS_SET`: each preset as it sets POLY, UNISON, VOICES, DOUBLE and DRIVE, and
  `CA72_RETIRED`): of a Ryzen 7 7800X3D core, Horn Bloom 181 % (Slow Horn Swell 87), Brass
  Choir 172 % (Brass Tutti 84), Warp Field 149 % (Warped Pad 73); Cycle of Fifths 73 %, the rest
  under 53 %. Paced on four processors with one worker (R39's machine): Slow Horn Swell and
  Warped Pad no block late, Horn Bloom 26 %, Brass Choir every one, Warp Field 7 %. At QUALITY LO
  4 to 5 %. Offered halving their VOICES (recommended), DOUBLE off, or leaving them with a note:
  **the owner:** "No. Keep them as is." The README's limitations say so.
- **0.1's sessions sounded different**: translated (R66).
- **Linux, CI by hand on 40b6a33** (a clean export, a target of its own): bundles, notices,
  `package.sh` (the tarball and the 0.1 presets' pack: 24 files, tagged 0.1, the retired file's
  names and values; read by the plug-in's own `Library` into a fresh library beside the 19 factory
  presets as 43), clippy with every feature (no warning), the standalone, `cargo test --workspace`
  with ngspice required (404 passed, none failed), `validate.sh` (clap-validator 37 passed and 7
  skipped, pluginval at strictness 10 with its editor tests SUCCESS, Steinberg's 47 passed).
  REAPER 7.79 (VST3): the editor whole, notes and a chord played, QUALITY's three, CUTOFF
  dragged, the drawer with the 19, Felt Bass chosen and played, VOICES dimmed in MONO and lit in
  POLY (R64), the window closed and opened again the same to the pixel.
- **Windows, the same on 40b6a33** (MSVC 1.97.1): bundles, the installer, clippy, 409 tests
  passed with the seven window tests in the desktop session, `validate.sh` (clap-validator 36
  passed and 8 skipped, pluginval at strictness 10 SUCCESS, Steinberg's 47) once run with
  `CMAKE_GENERATOR="Visual Studio 17 2022"` (MinGW's cmake first on that PATH chose GNU, and
  Steinberg's validator would not link). REAPER 7.82 (VST3 and CLAP) and Cubase Pro 15 (VST3):
  each check as on Linux, and DRIVE's readout typed in Cubase, the window moved off the screen
  and back, minimised and restored, the same to the pixel.
- **The Mac:** the 0.2.0 release candidate signed, notarised and installed (R65); `target/debug`
  of the main checkout cleaned (10.4 GB) and the owner's Trash emptied, so that the tests have
  room.
- **Branches** (the owner: "Yes, delete them"): the fourteen merged into main deleted from the
  Mac's repository, and the CAL build's worktree removed; GitHub's three when main reaches it.

**The final round, on main 026dfea (R66 in), 2026-10-11:**
- **Linux:** the CI by hand as above, every step passed: 410 tests, none failed (ngspice required,
  none skipped); clippy with every feature, no warning; the validators as before; the pack, its
  24 files with LEVEL -0.5 and the 16 knobs written out, the retired file's names and values,
  read into a fresh library as 43 presets. REAPER 7.79 (VST3), started with the shared folder
  in the test's own place: the editor whole, a note, the drawer's 19.
- **Windows:** the CI by hand, every step passed: 415 tests with the window tests in the desktop
  session, clippy, the installer, the validators (`CMAKE_GENERATOR` as above). REAPER 7.82 (VST3)
  smoke as on Linux. **0.1's session translated end to end:** 0.1.5 and 026dfea built as one
  plug-in (the same names and IDs, a test build's); in 0.1.5 EMPHASIS 5, CUTOFF 2, FILTER DECAY
  6, OSC 1 VOLUME 6 set and the project saved (its state `"version":"0.1.5"`, no LEVEL or
  DRIVE); opened with 026dfea: 3.67, 1.74, 6.75, 5.82, LEVEL -0.5 dB, DRIVE 0, each
  `compat::translated`'s value; saved from 026dfea (`"version":"0.2.0"`) and opened again, in
  the same REAPER and a fresh one, unchanged.
- **The Mac:** a clean export, universal bundles, the Audio Unit, the unsigned installer, clippy,
  the standalone; `cargo test --workspace`: 406 passed, 3 failed, R60's three circuit-lab tests
  with Homebrew's ngspice; `validate.sh` in the desktop session: clap-validator 37 passed,
  pluginval at strictness 10 SUCCESS, Steinberg's 47, auval not finding the component (R60's
  registry). GarageBand 10.4.14 (AU) and Bitwig 5.2.7 (CLAP), the owner away: the editor whole,
  QUALITY's three, CUTOFF dragged, the drawer's 19 and 0.2.0, Horn Bloom chosen, VOICES dimmed
  and lit, the window reopened the same (Bitwig: the window's corners within 5 of 255).
- **The release candidate again** (026dfea, R66 in): that export's bundles signed and notarised
  in a session of the desktop's tmux server, accepted, stapled, `spctl` "Notarized Developer
  ID"; `CA-72-0.2.0-macOS.pkg` SHA-256
  f3f1670d82e61f042f781b9040dd21a4f8d43fcfbc926de8174992d160eef0a9, with its logs and the Mac
  checks' in `~/lab/artifacts/ca-72/0.2.0-rc2/`.
- **The release candidate with the owner's last presets** (main 721b5bd, R63's last changes; a
  data change, checked by the unit tests and the presets' levels): universal bundles and the
  Audio Unit built in the agents' tree (their presets read in the binaries), clap-validator 37
  passed, Steinberg's 47, pluginval at strictness 5 SUCCESS; signed, notarised, stapled, `spctl`
  "Notarized Developer ID". `CA-72-0.2.0-macOS.pkg` SHA-256
  a9751ed47422d1ee20541757f27920a0858bfcca4ddddb39545c50982afde17c, in
  `~/lab/artifacts/ca-72/0.2.0-rc3/` with its logs. The leftovers of the Mac's checks (an unsigned
  Audio Unit in `~/Library`, the library folder GarageBand's run made) and the checks' build
  folders went to the Trash, at the owner's word.
- **Not done:** auval and pluginval on the Audio Unit on this Mac (its registry), MIDI played in
  the Mac's hosts (GarageBand takes none from a script), an Intel Mac, Live (the owner's).

## R68. The one instrument sleeps while nothing plays

**The owner, 2026-10-12:** "why does our plugin cost cpu usage even when it's not playing
(according to ableton)", then, told that the development line had fixed it (its R30, the `perf`
branch, never brought over): "oh, we need to include that fix please!"

**Found:** with POLY and UNISON off the one instrument ran every sample whether or not a note
played, `process` asking the host to keep calling it (`KeepAlive`). Idle, after 10 s of 256-sample
blocks (the Linux reference machine, one thread): QUALITY HI 5.25 % of a core (a note held
6.45 %), ULTRA 30.6 % (31.2), DOUBLE 10.4 % (12.5), LO 0.25 % (0.26). POLY's and UNISON's voices
are freed as each falls silent: 0.02 % idle.

**Agent decisions, 2026-10-12** (not separately approved), the development line's R30 brought
over to the engine as it is now:
- **The one instrument sleeps** once no key is down, its output has stayed under -120 dB
  (POLY's rule for freeing a voice) and its slow state has moved less than 1 uV over 50 ms:
  `Voice::slow_state` (the keyboard's hold capacitor, the keys beyond the string's ends, both
  contours; brought back to the circuit's voice, which had lost it, the light voice having kept
  its own), its twin's too while DOUBLE plays it, of whichever model QUALITY plays. The check
  runs at each block's end (`Engine::end_block`); asleep, `render` and `tick_stereo` give
  silence without playing. A note or any other event, a control changed (`Engine::set`, only
  when something changed), All Notes Off, the engine prepared again, or a sound at EXTERNAL
  INPUT (over -120 dBFS) wakes it, from the state it slept in. `KeepAlive` stays.
- **The same sound.** Awake, nothing is computed differently. Asleep, only the free-running
  oscillators' and noise's phases stop. Every factory preset's phrase (`tests/preset_levels.rs`,
  each preset as it sets POLY, new `CA72_NEVER_SLEEP`) renders the same to the bit with the
  sleep and without it, through every note; two differ only in their last 0.14 to 0.18 s,
  where the sleeping one gives exact silence and the other a residue at -124 dBFS.

**Evidence (the Linux reference machine, 2026-10-12):** idle now 0.001 % of a core at HI, LO,
ULTRA and with DOUBLE, asleep within 10 s (UNISON and POLY 0.02 %, as before). Tests:
`a_silent_settled_instrument_sleeps_and_wakes` (asleep with nothing played and silent so;
awake while a key is down; woken by a note, a control, the wheel, All Notes Off and a sound at
EXTERNAL INPUT, the same controls again not waking it; never with `never_sleep`; asleep at LO
and with DOUBLE; never the one instrument's sleep with POLY or UNISON, nor while A-440 sounds)
and `a_woken_instrument_plays_its_next_note_as_one_that_never_slept` (at rest the contours within
20 uV, through the next note the keyboard the same and the contours within 0.5 mV, the level
within 0.1 dB). Each fails with what it guards broken: without the wake on events, and sleeping
as soon as the output is silent (the contours then 24 mV apart). Clippy with every target; the
plug-in's 233 tests.

**On main 866f6dc, 2026-10-12:** Windows (a clean export, MSVC 1.97.1): clippy with every
feature and target, no warning; `cargo test --workspace` 417 passed, none failed, the seven window
tests in the desktop session; the bundles and the installer. The Mac: the plug-in's 233 tests
passed and clippy; the universal bundles and the Audio Unit, clap-validator 37 passed, Steinberg's
47, pluginval at strictness 5 SUCCESS; signed, notarised, stapled, `spctl` "Notarized Developer
ID": `CA-72-0.2.0-macOS.pkg` SHA-256
9b0dd7003f3fc09dc1ac46370299e59c7997f480779cb76f973aadbf03ce8eeb, the release candidate, with its
logs in `~/lab/artifacts/ca-72/0.2.0-rc4/`.

## R69. 0.2.0, released (2026-10-12)

**The owner, 2026-10-12:** "verify that there's no dependencies or anything that this new
release is lacking for any of the platforms. we need to make sure everyone can install it
without issues. If that's the case, then we're good to release!", and, told the agent would
merge and publish on green unless something failed: "k, let me know when it's done".

**What each platform needs** (the release's own builds): the Mac's VST3, CLAP and Audio Unit load
only the system's frameworks and `/usr/lib` (AppKit, AudioToolbox, CoreFoundation,
CoreGraphics, CoreMIDI, Foundation, QuartzCore; libSystem, libiconv, libobjc, libc++), each with
x86_64 and arm64 and macOS 11.0 as its least; Windows' VST3 and CLAP (CI's, from the ZIP) import
only the system's DLLs (kernel32, user32, gdi32, shell32, ole32, oleaut32, avrt, ntdll,
bcryptprimitives, an api-ms-win-core set of Windows 8 on), the C runtime linked in, x64;
Linux's (CI's, built on Ubuntu 22.04) need libX11, libxcb and libX11-xcb, as `install.sh` checks,
and glibc 2.35 at most. Since 0.1.5 the plug-in takes two crates more, `plugin-kit-materials` and
`plugin-kit-stereo`, Idle Foundry's own, from the public plugin-kit at 4c0f0f3 (in the git
sources' archive); no system library is new, and nothing asks for more than each processor's
baseline.

**The release:** main on the Mac (9fbb1f8, the README's status and DOUBLE brought up to date) as
pull request 26, CI passed on the three systems; merged as ef009fa, whose tree
(dbde6ea988ec7a89db16053f08d019a76542bf00) is the one the Mac's installer was built and signed
from; tagged v0.2.0 (annotated). The tag's CI built the Windows installer and ZIP, the Linux
tarball and the 0.1 presets' pack and drafted the release; the Mac's installer (universal
bundles, the Audio Unit, clap-validator 37 passed, Steinberg's 47, pluginval at strictness 5,
signed with the Developer ID identities, notarised, stapled; each bundle inside 0.2.0,
x86_64 and arm64, `codesign --verify --deep --strict`) uploaded and its line merged into
`SHA256SUMS.txt`. Every asset downloaded again and checked against it, and the ZIP against its
own `.sha256`. Published 2026-10-12 13:46 UTC with notes in 0.1.5's form (a row for the 0.1
presets; what a 0.1 project gets, R66). `releases/latest`, asked anonymously, gives v0.2.0 and
the three installers the update check looks for; the installer as downloaded from GitHub:
`spctl` accepted, "Notarized Developer ID", the ticket stapled. GitHub's merged branches deleted
(calibration, r33- and r35-release-evidence, release-0.2.0); pull requests 11 and 12 closed as
merged with main's arrival. The installer, its logs and notarisation are in
`~/lab/artifacts/ca-72/0.2.0/`.

| Asset | SHA-256 |
|---|---|
| `CA-72-0.2.0-macOS.pkg` | c8d47b3d587ee79056ee76789ad8ff7ebd2b2f46f7a77679e98dd1e609aecc94 |
| `CA-72-0.2.0-Windows-setup.exe` | 3e8fe542f342f14add36d2cd1aa30ff707ba3d3e1c6ec3e12db2e4a1fb0df656 |
| `CA-72-0.2.0-Windows-x86_64.zip` | c6f5cddc9a1c942ed536359bb8ffe680878fab648b8e27dc33222191d7265a28 |
| `CA-72-0.2.0-Linux-x86_64.tar.gz` | f3126bab1213a25fb0257ee2ab254ced2148d07d53817c3be21cf04b4a85365c |
| `CA-72-0.2.0-0.1-presets.zip` | 499eccec93e4063645e68d59a512d23e3b4077e0b44863aace21770221a46413 |
| `CA-72-0.2.0-git-sources.tar.gz` | 22d5c0733c307d510369fe345d5682973cdd33c841a168d49500913299772545 |

**Arch Linux** (the owner, after publication: "what about arch linux systems?"): the released
tarball installed by its `install.sh` on the Linux reference machine (Omarchy, on Arch; glibc
2.44), no library reported missing, the installed bundles the release's to the byte, each library
found in Arch's packages (libx11, libxcb, gcc-libs, glibc); clap-validator 37 passed and 7 skipped,
Steinberg's validator 47 passed, pluginval at strictness 5 SUCCESS. Arch is named beside the
other systems in the README and the release's notes.

**Not done:** an Intel Mac, auval on this Mac's registry (CI's macOS job runs it), and Live (the
owner's).

## R70. The lower panel can be shown or hidden

**Owner decision, 2026-10-13.** The whole second panel should show or hide, with "no animation
of growing". Asked whether hiding it should shorten the window while leaving the synth and
preset rail visible: "Yes, snap between the short and full window."

The owner reviewed the rendered preview and approved CI and merging the change.

**Agent decisions, 2026-10-13** (not separately approved):
- PANEL beside the rail's settings key toggles the lower controls. The window's width and
  controls' scale stay fixed; its height changes in one request. The rail's name display is
  100 drawing units shorter to fit the key, with NEXT and SAVE moved left by the same amount.
- The controls are shown by default, preserving existing projects' view. The choice is saved
  with the project as `editor_panel`, outside the sound's parameters and presets. Presets
  temporarily reveal a hidden panel; closing them restores the compact view. The preset
  browser's existing slide stays inside the full panel; the window itself never animates.
- A refused resize keeps the height accepted by the host. The compact view has a grip on the
  rail's bottom right corner, and hidden controls do not take pointer events.

**Evidence (the Linux reference machine, 2026-10-13):** the editor's 64 tests passed, 7 visual
or timing tests ignored. New checks cover the immediate height change, temporary preset
view, unchanged sound controls, reopening compact, refused resizing, the compact grip and
compact frames drawn in parts matching frames drawn whole. Review found and fixed compact
menus extending below the viewport and panel visibility recalled into an already open
editor; both have regression checks. A height-only resize preserves the frame composed
just before the window follows it; without that guard, its regression check fails with
"keep the rail and compact grip". Actual editor frames and an
interactive preview are saved in this chat's visualization folder. Real-host resize checks
and release CI have not been run. Linux VST3's queued resize can report acceptance before the
host responds, as with the existing width grip; later refusal is not handled here.

The touched crates' library tests: 243 passed, none failed, 20 ignored; the panel's approved
image comparison passed. Formatting for both crates and clippy for all their targets, with
and without optional features, passed. The independent reviewer rechecked the fixes.

## R71. HI's filter at 4x under strong regeneration
**Owner report, 2026-10-13:** high EMPHASIS and AMOUNT OF CONTOUR sound harsh and crackly,
like tuning a shortwave radio. The owner requested a spectrogram.

**Agent decision:** HI keeps its filter at 4x instead of 2x. The cheap oscillator,
contour and device models remain HI's. A whole C4 note with EMPHASIS and AMOUNT OF
CONTOUR at 10 reproduces broadband noise while its contour opens the filter above the
audio band. More Newton iterations at 2x remove much of that noise but leave strong
aliased tones; 4x removes both. At 44.1 kHz the old voice has 21.77 dB more power from
8 to 20 kHz than ULTRA; with the fix, +0.16 dB. At 48 and 96 kHz, at CUTOFF 0 and
+2.5, the fixed HI is within 1.66 dB of ULTRA in that band.

The rate is fixed while the controls move. Selecting 4x only above EMPHASIS 7.5 reset
the resamplers on crossing that mark and introduced a click; the regression checks
both crossings on a held note. HI's CPU cost rises, and its renders change. ULTRA's
rate and model do not change. No circuit values, trims or knob laws are changed.

**Evidence:** `crates/ca72-lab/tests/high_contour.rs` checks whole notes at 44.1, 48
and 96 kHz and control movement. Its ngspice 47 bench regenerates at 36.75 kHz; the
4x filter keeps 8 to 20 kHz output power 121 dB below that oscillation's power and
continues oscillating internally. Existing hardware takes were found on the lab share
(sessions E and J, EMPHASIS 10, J with AMOUNT OF CONTOUR 10; session I's Ringing Saw
Line). They do not capture the exact reported patch. Diagnostic spectrograms
are retained in [the filter-contour diagnostic record](filter-contour/README.md).

**Checks on the Linux reference machine:** 17 focused debug tests passed, including
the whole voice, resamplers, allocations and new regressions; clippy on the core and
lab, all targets, with and without all features; the changed Rust files formatted.
`high_contour_cost`, paced under the exclusive timing lock, retriggering notes at
48 kHz, passed twice: at EMPHASIS 10, 128 frames mean 0.49 to 0.50 ms, worst 0.95 ms
of 2.67; 256 frames mean 0.96 to 0.97 ms, worst 1.71 ms of 5.33. No block exceeded
75 % of its period. These are single-voice measurements, not a polyphony budget.

**LO follow-up, requested before installation:** the separate fitted voice ran its
nonlinear ladder at the output rate. At high contour it produced aliased resonance
tones. Its filter now runs at 8x through the existing sparse resamplers; the direct
FILTER MODE HI branch passes through the same resamplers, and the external feedback
delay is shortened by their delay to retain the fitted total loop delay. The nonlinear
input pair is solved with a bracketed Newton iteration, its caps counted. Unbounded
exponential cutoff above the fitted dial is replaced with relative saturation measured
from the calibrated converter and a high-rate circuit ladder. The ignored
`lo_cutoff_saturation_fit` test reproduces the table; it preserves the dial's fitted
pitch at the join and does not claim new hardware calibration. White noise's fitted
top at 48 kHz moves from 9.5 to 10 kHz after the filter rate change.

LO's whole-note check passes at 44.1, 48 and 96 kHz, with no solver caps. Relative
8–20 kHz power versus ULTRA is -15.02/-21.04, -1.92/-2.59 and -6.68/-6.65 dB at
CUTOFF 0/+2.5 respectively. Its 16 focused unit tests, including pitch, contours,
resonance, filter mode, noise and the new nonlinear residual check, pass. This
comparison tests unwanted high-band energy; it does not establish that LO matches
every detail of ULTRA's spectrum.

**Final checks and personal Mac installation, 2026-10-13:** independent DSP review
signed off source commit `2e1520d`. All 315 non-ignored core and plug-in tests passed
in debug; the four high-contour regressions passed with ngspice required, and the
cutoff-fit regeneration passed explicitly. Clippy passed for the core, lab and plug-in,
all targets, with and without all features. The old LO failed the high-band reproduction
at 48 kHz (+22.09 dB against ULTRA); the final LO measured -1.92 dB.

Two final paced runs under the exclusive timing lock passed at 48 kHz. HI's worst
block was 0.95 ms at 128 frames and 1.44 ms at 256; LO's 0.112 and 0.181 ms. LO's
mean was 0.070 to 0.075 ms at 128 and 0.130 to 0.152 ms at 256. No block exceeded
75 % of its period. These are one-voice measurements on the Linux reference machine;
the oversampling increases LO's cost and this is not a new comparison with Surge XT.

The Mac's own worktree built universal arm64/x86_64 VST3, CLAP and AU, deployment
target 11.0. The high-contour, control-movement, LO unit and callback-allocation tests
passed on Apple silicon. Signed bundles passed clap-validator (37 passed, 7 skipped),
pluginval strictness 10 (editor tests skipped while the owner's Live was open), and
Steinberg's validator (47 passed). The installer was Developer ID signed, accepted
by Apple's notary service, stapled and accepted by Gatekeeper. It installed all three
system bundles; their executable hashes match the validated signed staging bundles,
and the AU embeds the same signed CLAP. The previous bundles were renamed to a
protected backup before installation, preserving the executable inodes mapped by the
owner's open Live session. Live was left open; the owner must save and restart it to
load the fix. The attempted hidden desktop auval launch produced no validation result;
AU validation remains unverified (its build, signature and embedded CLAP were checked).
Personal installation remains version 0.2.0. No public release, Windows
host check, new hardware recording, or factory-preset re-leveling was done.

## R72. 0.2.1

**Owner decision, 2026-10-13:** the combined personal build is approved. Run the full test
suite and try the editor in hosts on macOS, Linux and Windows, checking the window's height
changes in particular, then submit and merge the pull request and ship to production.

The release combines R70's instant lower-panel toggle with R71's HI and LO filter fixes.
Parameter ids, plug-in ids, project migration and the preset format do not change. HI and
LO renders change, and their filter oversampling increases CPU cost; ULTRA is unchanged.
The native Windows tests cover repeated toggles with both parent and child resizing,
presets revealing and restoring a compact view, its width grip, and repainting at both
heights. Release validation is being recorded separately before publication.

**Resize review and repairs:** a queued VST3 resize had returned success before the host
answered, so a later refusal could leave the parent and child at different sizes. The
wrapper now reports that answer with a request id. The editor restores the last confirmed
view, including the saved width, compact preference and presets, and discards stale replies
from earlier requests or openings. A run-loop regression exercises Linux's actual queued
socket path. The native Windows matrix includes synchronous and delayed refusal as well
as acceptance. At fractional DPI scales, the composed frame now uses the physical viewport
size, including its last row and the compact grip, rather than the artwork's rounded size.

A second review found that presets opened without a height change had not updated the
rollback snapshot, and that a pending snapshot could change during state recall. Stable
non-size state is now refreshed between actions, and each issued action's completed snapshot
is frozen. Both editor regressions fail on the earlier candidate and pass after the repair.

R73's standalone and keyboard arrived on main before publication and are included in the
combined candidate. Merge review found that a delayed expansion refusal could restore the
compact view without releasing a keyboard note started while expansion was pending. Rollback
now releases local notes, clears their hover and restores keyboard capture to match the view.
Its regression failed before the repair and passes afterward. The allocation-test harness
also clears its thread marker before releasing its serial lock, including on unwind, and
records only the first failure backtrace while still counting every allocation and free.

**Release acceptance, 2026-10-14:** the combined source `717c83bf` passed
[CI on macOS, Windows and Linux](https://github.com/idlefoundry/ca-72/actions/runs/38514437842),
including the complete workspace tests, production adapters, installers and native Windows
window tests. Local workspace checks and clippy, with and without all features, also passed
on each system. Linux and Mac ngspice checks explicitly required version 47. The 74 focused
editor checks passed, with seven visual/timing checks ignored; the cutoff-saturation fit
regeneration was run explicitly. Two paced CPU runs on the identical DSP source passed;
these remain single-voice measurements, not a polyphony budget.

Installed modules were checked by path and executable hash before accepting host results.
The installed Linux VST3/CLAP passed all required validators. The signed Mac VST3/CLAP/AU
passed all five validator checks, including strictness-10 pluginval with editors and strict
auval. Windows installed CLAP and Steinberg validation passed, as did ten fresh strictness-10
pluginval runs with editors. Windows CLAP also passed 200 fuzz cases across 50 fresh validator
runs, four case types per run, using the CI artifact unchanged.

Native acceptance used isolated projects with tracks disarmed and outputs muted:

| Platform and host | Window evidence |
|---|---|
| Linux standalone | 23 size checks, compact grip and exact full-repaint pixels in a floating window; tiling constrains its outer window |
| Linux REAPER VST3 and CLAP | 23 size checks per format, MIDI with a dummy device, save/reload and exact repaint pixels |
| Linux Bitwig CLAP | 23 size checks, compact grip and exact full-repaint pixels; demo mode prevents save/reload |
| Windows standalone | 22 visible desktop parent/child size checks, compact grip and exact full-repaint pixels |
| Windows REAPER VST3 and CLAP | 23 size checks per format, compact grip, MIDI in muted projects, save/reload and exact full-repaint pixels |
| Windows Sandyne VST3 | 23 size checks, compact grip and exact full-repaint pixels; no MIDI/save-reload claim |
| macOS standalone installer payload | 17 height/preset checks, compact grip, width restoration and exact instrument pixels after reopen; devices disabled |
| macOS REAPER VST3 | 23 frame checks, compact grip and its following click, MIDI in a muted project, save/reload at 1495 × 907 and exact instrument pixels after reopen |
| macOS Bitwig CLAP | Eight short/full cycles and temporary preset reveal/restore; no numerical frame, grip, MIDI or save/reload claim |

Each 23-check DAW run includes eight short/full cycles and presets temporarily revealing
then restoring the compact view. The Mac REAPER drag harness initially hit the host's border;
the corrected run uses the instrument's grip and waits for the resize to settle before
releasing. It confirms the next panel click and the resized project's restoration. Minimum-OS
runtime, physical audio/MIDI latency, hardware hotplug and other installed DAWs remain
unverified. This acceptance concerns the tested configurations, not every host or display.

PR [#31](https://github.com/idlefoundry/ca-72/pull/31) merged as `1f5a9f4c`, tagged `v0.2.1`.
The merge/tag tree `96f1a1c411ca9a210e011eb18f78879a43681ee2` equals the tested source tree
and the Mac package's build tree. The [tag workflow](https://github.com/idlefoundry/ca-72/actions/runs/38576346799)
produces the Windows/Linux installers, Windows ZIP and pinned dependency sources. The Mac
package was Developer ID signed, notarized, stapled and accepted by Gatekeeper; its SHA-256
is `c24ebaccfd5759e0b25153851afc71c2c4d6f540970f77123e90c0a46b3d6032`.
The [release](https://github.com/idlefoundry/ca-72/releases/tag/v0.2.1) includes the asset
checksums. Download verification and publication are required before merging this evidence.

## R73. A desktop application and playable lower keyboard

**Owner decisions, 2026-10-13.** Add a standalone application, research and write a
detailed specification with clear success criteria, then implement its tasks. Put
the keyboard in the VOICES/STEREO/OUTPUT area, with a button beside settings to
switch views. Keep the realistic materials and original Model D proportions,
uniform key alignment and white-key cutouts around the black keys. The owner later
delegated visual judgement and directed work to continue without an approval wait.

**Agent decisions, 2026-10-14** (not separately approved):
- Native CoreAudio, WASAPI and ALSA hosting lives in the shared plugin-kit. A thin
  NIH adapter runs the existing engine, parameters, presets and editor; the legacy
  JACK development launcher remains available. macOS 11 remains the minimum.
- Device selection, real external capture, channel routing, sample rate, requested
  buffer, MIDI inputs/filter, recovery and atomic sessions are available from the
  editor. Requested buffer duration is labelled separately from playing latency.
  Independently clocked input uses preallocated, bounded clock correction.
- The 44-key F-to-C keyboard uses original generated surface assets over true L/T
  key shapes. Physical MIDI, pointer and typing own notes separately. TYPE starts
  disabled; focus loss and hidden controls release local notes. Paired PANIC
  requests are consumed once at the audio boundary. R70's PANEL remains available.
- Keyboard view and octave use the existing per-computer preferences, outside
  sound presets and project state. The standalone's instrument and I/O session
  uses its own versioned state directory. Invalid sessions are preserved and
  validated completely before any parameter or persistent-field setter runs.
- Installers add an optional application; Linux also gets a desktop entry and
  Windows' portable ZIP contains the executable. Existing component locations and
  session data remain compatible. No release is published by this change.

**Evidence, 2026-10-14:** the [contract](standalone-spec.md),
[task status](standalone-progress.md), [visual proof](standalone-keyboard/README.md)
and [acceptance evidence](standalone-verification/README.md) record the researched
sources, regression checks and native virtual-backend tests. Independent final
review reproduced the original malformed-field and shifted-key failures and
confirmed their fixes, the seven adapter checks, paired-panic ordering and
malformed-session close preservation. Native CI passed all three systems on
`acaa03c`; merged as `c34cb3c` with the same tree. Actual native package/CLI checks,
Windows no-device save/reopen/close and a private Developer ID signed/notarised
standalone Mac candidate pass. Physical audio/MIDI latency, microphone permission,
hardware hotplug, Mac GUI and minimum-OS runtime remain unverified. R71 and R72 are reserved for the already open filter fix and 0.2.1 release
records; this record follows them numerically.
