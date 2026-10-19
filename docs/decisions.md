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
  **Fixed 2026-10-16** (R-MAC-FIXES): the bench's hold switch closed at 1 ohm, not 1 milliohm.
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

**Release scope correction, 2026-10-14:** the owner confirmed that this release is
limited to the audio fix and the panel hide/show feature. The standalone application,
playable keyboard and unrelated changes are excluded. The earlier 0.2.1 release was
returned to draft and 0.2.0 restored as latest, with its tag and assets preserved.
This candidate starts from the 0.2.0 tag and selects the two features and their
resize, regression-test and allocation-test support from before the standalone merge.
The owner subsequently authorised building and publishing this corrected scope as latest.
Version 0.2.2 is the next unused patch version; the withdrawn 0.2.1 tag and assets
remain preserved. Acceptance of the withdrawn build does not validate this
candidate's binaries; the corrected candidate is checked separately before publication.


## R73. 0.2.2

The release starts at the 0.2.0 tag and adds only R70's panel hide/show, R71's
HI/LO filter correction and their resize and regression-test support. Production
standalone hosting, the playable keyboard, their dependencies and packaging are
excluded. The Mac CoreGraphics import ordering is adjusted only because the
repository's existing formatter requires it; its behaviour does not change.

The package scripts and release workflow match the plug-in-only 0.2.0 baseline.
The release tag is `v0.2.2`, commit `f440a30b88e0e9a5ca0d9c745f576bcae71c0ced`,
from PR #35. Its tree, `e6711cd82a731bdd1b9d3d434f3f2a244dfcf8c0`, is exactly the
accepted candidate's (`76acd686`); the integration merge retains current main's
history while restoring the selected production source. The optional development-only
launcher from 0.2.0 remains unchanged in source and is not packaged.

**Corrected candidate validation, 2026-10-14:** formatting, both full workspace clippy
configurations, cutoff-fit regeneration and the full workspace tests passed locally:
431 passed, none failed, 50 ignored. Universal VST3/CLAP and AU builds passed local
clap-validator, Steinberg's validator, pluginval strictness 10 and strict auval.
PR CI [38584514287](https://github.com/idlefoundry/ca-72/actions/runs/38584514287)
passed all three platforms. The Mac job's unchanged retry cleared a nested auval
timeout; its direct arm64 and Intel AU checks had already passed. The release build
is [38608650903](https://github.com/idlefoundry/ca-72/actions/runs/38608650903).
Two independent scope and packaging/licensing reviews found no blocker.

**Native acceptance used these corrected installer payloads:**

| System and host | Result |
|---|---|
| macOS, REAPER 7.82 VST3 | Eight full/compact cycles, presets reveal/restore, compact grip, settings, saved project and three MIDI notes restored; exact canvas equality after reload. |
| macOS, Bitwig 5.2.7 CLAP | Eight full/compact cycles, presets reveal/restore, compact grip, saved dimensions restored; exact canvas equality after editor reopen. |
| Windows, REAPER VST3 | 23 confirmed native frames, compact grip, presets and settings; saved project and three MIDI notes restored; exact visible repaint equality. |
| Windows, Sandyne 2.5 VST3 | 23 confirmed native frames, compact grip, presets and settings; exact visible repaint equality. |
| Linux, REAPER 7.79 VST3 | 23 confirmed native frames at 2x desktop scale, compact grip, saved project and three MIDI notes restored; exact full repaint equality. |
| Linux, Bitwig 5.2.7 CLAP | 23 confirmed native frames, compact grip and presets; loaded the saved Mac project state; exact full repaint equality. |

The loaded module path and hash were checked. An older system CLAP initially selected
by Mac Bitwig was excluded, then its scan location was removed temporarily and a fresh
process loaded the corrected user-installed CLAP. Linux Bitwig used its existing
PulseAudio driver because its configured EVO4 hardware was disconnected. These editor
checks do not claim a new hardware recording or factory-preset re-leveling.

The Mac installer is universal, Developer ID signed, Apple-notarized, stapled and
Gatekeeper accepted. Its SHA-256 is
`c685ee7dc14e0efe1aaf725e4d891548a8a4ce5cec472cbbd67057ea54531970`.
The package's signed executable hashes match the native test payloads. CI's unsigned
Mac package is excluded from the distribution. The release assets retain the required
Windows manual-install ZIP, checksum sidecar, 0.1 presets, git dependency sources and
licence notices. The release verification checks fresh downloads against GitHub asset
digests and checksum manifests, audits all package payloads, checks the downloaded
Mac package's signature/staple/Gatekeeper result, and checks public Latest routing.

The evidence and verification records accompany the release in the 0.2.2 artifact
folder. The withdrawn 0.2.1 remains draft with its tag and all nine original assets
preserved. Its acceptance does not stand in for these corrected candidate checks.

## R-STANDALONE. A desktop application and playable lower keyboard

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

## R-TYPE. Computer keys follow a piano layout with their own octave

**Owner decisions, 2026-10-14.** ASDFG in TYPE should play CDEFG. Computer-keyboard
notes should have their own octave, with Z lowering and X raising it, as in Live.
TYPE should remain playable after switching from the keyboard to the extra controls.

**Agent decisions, 2026-10-14** (not separately approved):
- Keep the chromatic key order but anchor it to a separate C root. A defaults to
  C3 (MIDI 48 in this instrument's labels). Z/X move 12 semitones once a press;
  roots clamp to C notes 0–120 and keys beyond MIDI 127 stay silent. The visible
  F-to-C range and oscillator RANGE controls do not change TYPE's MIDI pitches.
- TYPE's octave is a per-computer preference, outside presets and host state.
  Existing preferences retain their visible range and get the new C3 TYPE default.
  The header shows A's note and the Z/X shortcut while TYPE is enabled.
- An octave change releases only that input's notes. Typed keys remain recorded
  until keyup, including silent out-of-range keys, so host-forwarded repeats cannot
  restart them after transposition. Pointer and physical MIDI ownership remain
  independent; text entry, menus, modifiers and focus retain precedence.
- TYPE stays active across keyboard/controls swaps and compact view, releasing
  only pointer ownership when hiding the keys. Held typed notes and their releases
  still work, including after a host refuses the lower panel's expansion.
- The [official keyboard diagram](https://www.ableton.com/en/manual/live-keyboard-shortcuts/#keymidi-map-mode-and-the-computer-midi-keyboard)
  confirms the white/black key layout and Z/X shortcuts. Live calls MIDI 60 C3;
  this instrument calls it C4, so the chosen default is not a claim of identical
  starting pitch. L/O/P/semicolon continue the existing second-octave extension.

**Evidence (the Linux reference machine, 2026-10-14):** restoring the old shared-root
calculation makes `computer_keys_start_on_c_and_follow_the_piano_layout` fail on A,
with MIDI 41 instead of 48. Regression checks cover every mapped pitch, octave
limits, held-note source overlap, known/unknown key codes, repeated octave presses,
unflagged repeats after transposing, silent out-of-range keys, text takeover,
separate preference roundtrips and header invalidation. Independent review found
the two repeat cases and confirmed their fixes. The plug-in's production-feature
library tests passed: 236 passed, 13 ignored; all three keyboard-panel checks passed.
Formatting, diff checks and strict clippy for both touched crates, all targets,
with all features and without optional features passed. Physical devices were not opened.

## R-MIDI-LIST. Open device selectors follow discovery and hotplug

**Owner report, 2026-10-14.** The standalone did not appear to see the Mac's MIDI
keyboard, while other hosts did.

**Agent decisions, 2026-10-14** (not separately approved):
- Rebuild an open device selector when its inventory changes, retaining the
  highlighted device where it still exists and preserving the user's draft.
  Discovery finishing after opening MIDI INPUTS can no longer leave it showing
  only the disable option. Removed devices disappear without silently changing
  the requested input configuration.
- When hardware MIDI is unselected, show the detected-input count and an explicit
  prompt to choose inputs. Selecting a MIDI port and applying it remains deliberate.

**Evidence (the Mac and Linux reference machine, 2026-10-14):** CoreMIDI reports
the keyboard's two sources online; the pinned midir 0.9.1 lists both from main
and control threads. The running application's saved configuration has no MIDI
inputs selected. Its actual open selector was not inspected, so the stale menu
is a confirmed code path rather than a claim about that particular window.
`open_device_menu_follows_inventory_without_changing_the_draft` covers discovery
arriving into an empty open MIDI selector, selection, reordering, removal,
highlight bounds and unchanged running configuration until Apply. Disabling the
menu rebuild reproduces its regression failure (one option instead of three);
restoring it passes. The production-feature library suite passed: 237 passed,
13 ignored. Strict clippy with all features and without optional features,
formatting and diff checks pass; independent review confirmed the selector logic.
No audio device
or MIDI connection was opened, and the running application's preferences were not changed.

## R-MIDI-RECOVERY. Automatic input selection and honest discovery failures

**Owner report and request, 2026-10-14.** The updated Mac standalone initially
reports two MIDI inputs, then the count becomes zero after opening MIDI INPUTS.
Select a default MIDI input at startup and fix the disappearing inventory.

**Agent decisions, 2026-10-14** (not separately approved):
- New sessions use Automatic: select the first available uniquely named input,
  avoiding duplicate notes from opening every port. Remember that name across
  reorder, unplug and restart. Wait for an input when none exists; connect it
  without restarting running audio. Explicit input selections and Disable persist.
  Older sessions retain their previous choices, including disabled MIDI; selecting
  Automatic and applying once enables the new policy for those sessions.
- Retain the successful MIDI discovery client for the control thread's lifetime.
  Initialization, port-name and inconsistent-count failures preserve the last good
  MIDI inventory and active bindings, while audio inventory continues to update.
  Report discovery errors in settings and clear only their own diagnostic on recovery.
- A successful stable empty inventory remains empty and releases missing inputs.
  Report a transition from available inputs to none, with Refresh/reconnect guidance.
  Successful reconnection clears owned MIDI warnings; unrelated session/audio issues
  keep priority. The code never fabricates ports to conceal a successful zero count.
- Automatic's resolved port and normalized Apply results update an untouched dialog.
  Track edits explicitly: changing back to a previous value is still an edit, and
  any edit after Apply prevents its result from overwriting the new draft.

**Research and limits:** the [pinned midir CoreMIDI implementation](https://github.com/Boddlnagg/midir/blob/v0.9.1/src/backend/coremidi/mod.rs)
queries global sources. Apple's [source-count API](https://developer.apple.com/documentation/coremidi/midigetnumberofsources())
returns a count without an error status. A stable zero cannot be classified as a
failed query by this API alone. CoreMIDI reports the NI S88 MK2's two ports online;
250 MIDI-only inventories and 50 inventories surrounding the exact CPAL property
query ordering kept both sources, with no discovery errors. No audio streams or
MIDI connections were opened. The user's actual count loss is confirmed by their
report; client churn or CPAL ordering is not a proven cause. The new build repairs
confirmed error-handling defects and exposes successful-zero transitions; physical
playback and disappearance/recovery on the owner's running app remain unverified.

**Software evidence (2026-10-14):** 37 shared-host tests pass, including retained
client reuse, discovery failure/recovery, true empty inventory, MIDI warning ownership
and default selection/persistence. Five settings tests pass independently. Returning
the dialog's old value-equality sync condition reproduces the post-Apply ALL→2→ALL
failure; explicit edit tracking fixes it. The final pinned host revision is
`4bf3fa665c5e7b47c6be4bff9201ce02b10f60c9`. The production-feature plug-in library
suite passes: 238 passed, 13 ignored. Independent reviews found no remaining confirmed
issues after the corrections.

## R-KEY-FEEDBACK. Keyboard icon and pressed keys

**Owner decision, 2026-10-14.** Remove the last piano key on the keyboard button's right
and darken a playable key when pressed.

The remaining three natural keys and two accidentals are centred in the button. Held keys
use darker versions of their surfaces in both looks; the realistic material's alpha remains
unchanged, so a black key cannot brighten by revealing its background. Local note ownership
joins the audio snapshot for immediate visual feedback while a device is stopped.

Key masks use fixed key coordinates and integer compositing. Press/release repaint checks
found a fractional-scale seam when the old mask and float sampler followed the damage patch.
The regression now compares incremental frames with a fresh keyboard, including black keys.
The [rendered previews and focused checks](keyboard-feedback/README.md) record the candidate;
visual review precedes full CI or merging.

**Header follow-up, 2026-10-14.** The owner also asked for the keyboard header's buttons and
text to be vertically centred on the wood. Their shared centre is now halfway between the
rail and the key bed's outer border. Labels use their visible glyph bounds for an alphabetic
baseline; this aligns OCTAVE, the range, plus/minus, TYPE, PANIC and the mapping legend despite
their different sizes. Button drawing, materials and hit-testing share the same centre.
The updated preview records the alignment; panel and focused editor checks still pass.

**Display follow-up, 2026-10-14.** The owner asked to reuse the preset screen's pixel lettering
and bezel for the octave range. It now calls the shared bezel renderer and uses the same glass
asset, dot font, grid spacing and light colours. Its readout remains centred on the wood;
the preset display keeps its existing appearance. Updated renderer previews and panel/lint
checks pass; visual review still precedes full CI or merging.

**Button follow-up, 2026-10-14.** The owner asked to place TYPE and PANIC side by side and
give them the same size. TYPE now sits just left of PANIC, with both caps 230 × 48 units and
a 16-unit gap. Header drawing, realistic materials and hit-testing share a single button layout.

**Host follow-up, 2026-10-14.** The owner asked for TYPE and PANIC only in the standalone
application, leaving those functions to the DAW. Runtime host API selects their visibility and
hit targets, independent of build features. Plug-ins keep the pointer keyboard and octave
controls, with note typing and its legend disabled. The repaint regression includes switching
between both layouts; editor checks cover actual CLAP, VST3 and standalone contexts.

**Label follow-up, 2026-10-14.** The owner chose TYPING as the compact label for enabling
computer-keyboard note input. It replaces TYPE on the standalone button.

**Integration follow-up, 2026-10-14.** The owner asked to include these UI fixes in
the MIDI recovery build. Merge the keyboard feedback candidate `cf90cef` into
the standalone development branch, preserving its icon, materials, pixel display,
header alignment, grouped buttons, host-specific visibility and TYPING label.
Retain R-TYPE's separate C-root octave and Z/X legend; TYPING continues through
controls/compact views in standalone. Plug-ins leave note typing to their host.
The combined source passes 240 production-feature library tests (13 ignored),
45 panel unit tests (7 ignored), the panel approval check and strict Clippy for
both feature configurations. The native combined render was shown to the owner;
their earlier delegation permits proceeding without another approval wait.

**Frame follow-up, 2026-10-14.** The owner requested straight wood at both sides
and wood meeting the bottom silver border. Replace the metal substrate and separate
wood rails with one continuous walnut field around the recessed key bed. Its right
and bottom limits match the trim's inner edges (`W - 28`, `H - 6`). The drawn skin
uses a continuous wood-coloured surface too. Include fractional boundary coverage
and composite it over the existing trim; inward pixel clipping left a thin dark gap.
Both skins' incremental/fresh-frame checks pass at 0.44 and 0.87 scale, as do all
45 panel tests and the approval check. The updated native render was shown to the owner.

## R-SETTINGS-DONE. Done when settings have no unapplied changes

**Owner decision, 2026-10-14.** The Audio & MIDI primary button says Done when
settings are unchanged and Apply only after changes. Compare the draft with the
host's applied configuration, retaining a pending submission until acknowledged.
Done closes without submitting a configuration or restarting audio, even without
available hardware. No-op choices and edits reverted before application restore
Done. A failed application keeps Apply; successful normalisation restores Done.
Keep later edits when an older application completes, including edits back to the
previously running value. Pointer and Enter use the same primary action.

The host's additive `apply_tracked` receipt identifies completion of the exact
request; configuration equality or a Running phase can still describe an older
snapshot. Permit one queued application per dialog. Its receipt returns the
accepted normalised configuration or failure, including repeated identical
failures; failure unlocks retry. Re-selecting an unchanged output, input or
Automatic preserves its subordinate channels and remembered MIDI choice.

The existing missing-MIDI policy is unchanged pending the owner's discussion:
remember selected inputs, release missing live bindings and retry reconnection;
do not silently switch to a different controller. The UI currently does not label
remembered missing ports distinctly or retain them in the selector for deselection.

## R-KEYBOARD-GRID. Keyboard header shares the lower-panel control grid

2026-10-14 font follow-up: the owner approved matching the keyboard's grey-button
labels to the preset rail. They now use the same built-in URW Gothic, size 20,
bold (Demi), including the octave buttons. Their visible glyphs remain centred
using the bold face's measurements. Button geometry remains 62 units high.

**Owner decision, 2026-10-14.** Align the OCTAVE group and TYPING/PANIC to the
same grid as the other controls. Use the strip's existing column anchors: OCTAVE
on CV, the range display on CW, the typing legend on CI and the paired action
buttons centred on CO. Keep TYPING/PANIC side by side with equal 230-unit caps
and a 14-unit gap. Arrow widths and all cap heights reuse the rail's KEY_W/KEY_H.
Caps and the range bezel share their top, bottom and vertical centre; measured
glyph centring remains in use. Drawing, material placement and pointer targets
derive from the same coordinates. Wood, the recessed key bed and key shapes keep
their existing geometry.

**Active-state follow-up, 2026-10-14.** The owner requested the same coloured
active treatment for PANEL as for the keyboard and TYPING. Publish the editor's
expanded state to the rail scene and apply the existing amber cap tint. Collapsed
or refused expansion stays neutral; refused collapse retains its active finish.
Both skins and incremental repaint use this same state and shared key geometry.

## R-STANDALONE-INTEGRATION. Add the standalone feature to current main

**Owner authorization, 2026-10-14, via Add expandable second panel.** The
standalone agent leads one feature PR against current main, preserving the corrected
0.2.2 audio/resize fixes and release history. Integrate the already combined UI work,
current settings completion behavior, MIDI recovery and 20-bold button labels.
This authorizes integration and merge on completed validation; it does not authorize
a new public release or tag. The withdrawn 0.2.1 assets remain preserved.

The baseline is `db681b8`. The integration contract and pending acceptance matrix are
in `standalone-integration.md`. Existing preview and release evidence remains scoped
to its original binaries. In particular, the unresolved c16 nested AU validator
inactivity timeout is not waived or called fixed by this integration.

**PANIC integration review, 2026-10-14.** Two clicks can span an already captured
callback. A regression first failed on the boolean handshake: the first hardware
boundary admitted a fresh note belonging to the second UI request. Use the shared
host's tagged request, captured atomically with its panic flag, and acknowledge
only an exact local token. Older or generic boundaries reset physical sources
without clearing a newer pending UI identity. CC120/123 and stream resets do not
acknowledge a pending UI request. Adapter metadata is consumed once, including on
invalid blocks. Tokens wrap within 63 bits and skip zero. Regressions cover split
and coalesced clicks, generic resets, CC120/123, invalid blocks and stale queued
input. The shared implementation is plugin-kit `fc8448dd`; full integrated
validation remains required before merge.


## R-STARTUP-READY. Restore the standalone editor before cold DSP preparation

2026-10-14: exact-head native app checks passed two launch/restore/close cycles on
Linux and Windows. On macOS a fresh session passed, but reopening its saved state
reported the two-second startup timeout. The control thread later saved the restored
value successfully. Its readiness handshake included cold circuit factory preparation,
which can take seconds; the editor only needs restored fields and configuration.

Pin plugin-kit `f7aaa2b0` (PR9): publish restored state/configuration readiness before
cold processor warmup; keep preparation/deactivation on the control thread before
commands or audio devices. A channel-gated regression failed before the fix and now
passes; independent review accepted it. No timeout, validator or DSP code changes.
Native two-launch checks and full final-head CI must pass on the updated candidate.
The earlier `e43d39c` proof remains scoped to its original artifacts, including the
macOS native failure; it is not acceptance of this candidate.


## R-AU-VALIDATION-BUDGET. Bound the AU stress run by its actual workload

2026-10-14: the owner requested the AU test pass before further installer work.
Final integration 589e5cc failed pluginval 1.0.4's default 30-second inactivity
watchdog inside nested auval. Run 38653356837 wrapped its exact CLAP bytes with
bounded tracing: 1,513 completed calls consumed 31.315 seconds; the expensive
ULTRA/unison state had A-440 on; offline replay showed retained doubled voice
tails. Requests and notes progressed normally. Direct and nested parameter randomization differed, so a
fast direct pass was not proof of a hang or a controlled performance comparison.
An offline replay reproduced the workload cost; forwarding missing AU Reset did
not materially reduce it and is not represented as the repair.

Agent decision following independent review: the AU-only pluginval invocation
uses a finite 180000 ms inactivity timeout. Its nested `-strict -stress 20`
simulates 20 seconds of audio, which need not render in 30 wall-clock seconds.
Strictness 10, all editor/conformance/stress tests, error propagation, direct
native/Rosetta auval and other validators stay enabled. DSP, quality, product
binaries and installers are unchanged by this correction. The default-30-second
failure remains recorded; this is a harness budget change, not a performance fix.

Run 38654571401 uses the original, uninstrumented 589 AU and embedded CLAP from
CI artifact 11932543293, with its verified archive hash and original pluginval
seed. Direct strict stress and nested auval exited zero; all pluginval tests
including editors completed successfully. This fresh auval draw was faster than
the captured failure, so it does not reproduce the exact failing randomized state.
`au-validation.md` records the evidence and limits. Full exact-head CI on all
three OSes remains required before integration merge. No new release or tag.

## R-RELEASE-0.2.3. 0.2.3

**Owner authorisation, 2026-10-14.** Publish the validated standalone/keyboard
integration from main, excluding PR #40's Linux repaint experiment. The owner
confirmed that the experiment did not remove the visible stretch and deferred it.
The publication request authorises release signing, tagging, upload and publication.
The existing installer choices are retained; no installer redesign is included.

The base is `edf44abeab34e4ae5a4c01839026641c99b97d23`, whose post-merge CI run
38657626743 passed Linux, macOS and Windows, including the AU checks. This release
preparation changes the workspace/package version and public documentation only.
The engine, editor, host adapter, dependencies and packaging sources match that
validated base. The already published 0.2.2 assets and withdrawn 0.2.1 draft remain
preserved. Release notes are in `releases/0.2.3.md`.

Before publication, the 0.2.3 commit must pass all three CI jobs, independent
documentation/provenance review, and package verification. The macOS installer
must be Developer ID signed, notarised, stapled and accepted by Gatekeeper.
Every uploaded asset must be downloaded again and checked against the manifest.
Exact release artifacts and verification evidence are recorded separately when
those checks complete; earlier checks retain their original source/binary scope.


**Publication evidence, 2026-10-14.** [CA-72 0.2.3](https://github.com/idlefoundry/ca-72/releases/tag/v0.2.3)
is public and Latest. PR #41 was fast-forwarded to main at the exact tested commit
`a28ba0a0c9a7f6b748d1789d8eedfb637f4faba5`, which the annotated `v0.2.3` tag
also names. PR #40 remains open and unmerged; its repaint experiment is excluded.

- [Release PR CI 38668501280](https://github.com/idlefoundry/ca-72/actions/runs/38668501280)
  and [tag CI 38670736059](https://github.com/idlefoundry/ca-72/actions/runs/38670736059)
  passed Linux, Windows and macOS. Raw logs were independently audited, including
  strict Clippy, workspace and derive tests, CLAP/VST3/Steinberg validators and
  editor tests. Direct native and Intel/Rosetta strict AU validation passed;
  nested AU pluginval passed strictness 10 with the finite 180000 ms inactivity
  budget. This remains a validation budget correction, not a DSP performance claim.
- The universal Mac package was signed from release PR artifact 11942903792 at
  that exact source commit. Executable and data sections match its verified CI
  originals. All four components are universal, require macOS 11 and carry
  Developer ID Application signatures; the standalone has the audio-input
  entitlement. The installer has the matching Developer ID Installer signature,
  Apple's Accepted notarisation result and a validated staple. A fresh downloaded,
  quarantine-marked copy passed signature, staple and Gatekeeper assessment.
  Its SHA-256 is `861f313b500853a61a142f331da79be7f136ce9bd97cd35dbe3f3484b095ff4f`.
- Final tag Windows and Linux installers match their digest-verified CI inputs.
  The actual Windows compressed installer was extracted without executing it;
  all eight payload files match the final binaries, icon, licence and notices.
  Windows binaries use system libraries without a Visual C++ Redistributable
  dependency. Linux payloads match the raw artifacts and retain the glibc 2.35
  floor. Installer choices and metadata are unchanged from the reviewed sources.
- All nine uploaded assets were downloaded afresh. Their bytes match GitHub's
  asset digests; `SHA256SUMS.txt` covers the other eight assets, including the
  notices and Windows ZIP checksum sidecar. The Windows ZIP's internal manifest
  and recorded tag artifact provenance, 24 retired presets, and git-pinned
  dependency sources were independently checked. Anonymous Latest API/page
  requests resolve to 0.2.3, and all nine public asset URLs return HTTP 200.
- Published 0.2.2 and withdrawn draft 0.2.1 retain their original asset identities,
  sizes and digests. No earlier download was replaced.

The earlier no-device native launch/save/restore/close checks remain scoped to
`589e5cc` and their recorded binaries. There is no new claim of a complete physical
audio/MIDI, hotplug, latency, minimum-OS runtime or external-host matrix for this
release. Immediate close during cold preparation remains untested. The known brief
Linux host/compositor panel stretch is deferred and listed in the release notes.

## R-LETTERING. LETTERING: a larger print, a setting

**Owner decisions, 2026-10-15.** The owner reported complaints about readability from people
using the plug-in, and later that "users were complaining about the size of the menu
lettering as well". At the size the editor opens on a 1920 by 1080 screen (1380 logical pixels
wide, 0.444 of the drawing), the panel's legends were 8.4 pixels and its dials' numbers 6.7.
Shown the editor rendered at that size with the lettering as it was, at legends 25 and numbers
20, and at a middle step, the owner asked "what about slightly smaller text?" and chose the
middle step: "Go with the middle size: fix the four tight spots (RETRIGGER, KEYBOARD CONTROL,
OSC. 3 CONTROL, ON/ON), update the approval test, and show me before/after renders before
anything is merged." Shown those, the owner asked to "make the increased font size a menu
option": "this is just SOME people, so I want it as an option, not as a general change."

**Agent decisions, 2026-10-15** (not separately approved):
- **The setting.** The settings' menu has LETTERING between LOOK and FREQUENCY DIALS:
  STANDARD (the default, the panel as it was, to the pixel) and LARGE. It is this computer's,
  kept in `settings.toml` as `lettering`, "standard" or "large" (anything else standard), in
  both looks; a look chosen keeps it (`art::Lettering`, `Renderer::with_look`,
  `Editing::set_lettering`).
- **LARGE's sizes.** The panel's legends 19 to 22 units and its numbers (dials' numbers, ON,
  LO and HI, the left hand's small print) 15 to 17; the strip's legends 21 to 23 and its small
  print (dB, L and R) 16 to 18. On screen at 0.444: legends 9.8 pixels, numbers 7.5, the
  strip's 10.2 and 8.0. The section titles, the tabs' caps, SAVE, the displays and OSC. 3
  CONTROL (printed smaller than its neighbours, 16) are as they were.
- **LARGE's menus.** The tip, the notes and every menu (the settings', each control's) a
  quarter larger: menus 13.8 to 17.25 logical pixels, the tip 12 to 15. On a window the size
  of a 1920 by 1080 screen's the settings' menu is then too tall to drop below its key and is
  moved up within the window, over the rail's right end, as any menu that does not fit is.
- **Rejected: legends 25, numbers 20.** About a dozen collisions with the controls where they
  are: CUTOFF FREQUENCY into EMPHASIS, FILTER MODULATION over the divider, OSCILLATOR MODULATION
  and MODULATION MIX across theirs, RANGE on its 16' and 8'. That size needs the panel laid out
  again, not its print enlarged.
- **The four tight spots, in LARGE only, each control where it was.** RETRIGGER's centre 104
  to 96, its right end clear of its switch. KEYBOARD and CONTROL 431 and 448 to 426 and 446:
  20 apart as the other two-line legends are (17 had them touching at 22), centred in the gap
  between their switches. OSC. 3 and CONTROL 10 higher (480, 499), clear of RANGE's 32' beside
  them, which grows with the numbers. The column's right-hand legends (HI and the three ONs)
  1954 to 1944, their right ends at the switches' (1961), the second ON clear of ATTACK TIME's
  200.
- **The approval tests.** `approved.png` is unchanged and the standard lettering is still
  compared with it. LARGE is compared with `approved-large.png`: the same mock-up, but where
  the large lettering changes the drawing (100,676 pixels of the panel, the drawn skin at one
  unit a pixel, every control at half its travel) the renderer's pixels, as the switch column
  has the renderer's (R46); the limits are the same. Over the large lettering the test guards
  the drawing as reviewed, not the mock-up.

**Evidence (the Mac, 2026-10-15):** The standard lettering's renders (both looks, 2760 pixels
wide) are the same to the pixel as `main`'s before the change; the large lettering's the same
to the pixel as the renders the owner reviewed. `the_panel_is_drawn_as_approved`: mean
difference 1.444, 30,745 pixels 32 levels or more apart (limits 1.5 and 32,851), as before;
`the_panel_in_the_large_lettering_is_drawn_as_approved`: 0.965 and 17,557.
`cargo test -p ca72-panel`: 47 passed, 7 ignored; `cargo test -p ca72-plugin --lib --tests`:
the library's 236 passed, 14 ignored, `the_settings_key_chooses_the_lettering` among them, and
every integration test passed; clippy with `-D warnings` on both crates. `the_lettering_pngs` (ignored, `$CA72_LETTERING_PNG`) writes the
window in each look and lettering and with the menus open. Not tried: a host; the presets'
drawer and the keyboard view, whose lettering LARGE leaves as it was; spacing the other
two-line legends wider in LARGE (OSCILLATOR MODULATION, FILTER MODE, FILTER MODULATION,
EXTERNAL INPUT VOLUME sit tighter at 20 apart, not touching).

## R-SIZE. WINDOW › SIZE…, and a grip that can be seen

**Owner decisions, 2026-10-15.** The owner reported that "a few people saying they can't or
don't know how to resize the window". Offered a SIZE group in the settings' menu, a grip that
can be seen (a resize pointer and a tip) and, later and on its own, resizing by the host's
window edge, the owner chose the first two, to be released with the large lettering
(R-LETTERING): "add the menu size group + grip then release that along side the new lettering
option."

**Agent decisions, 2026-10-15** (not separately approved):
- **SIZE as a menu of its own.** The settings' menu has WINDOW › SIZE…, which opens the SIZE
  menu where the settings' was, as STANDALONE › AUDIO & MIDI… opens its dialog. Six more items
  in the settings' menu would have made it taller than the compact view of a small window.
- **The sizes.** 75 %, 100 %, 125 %, 150 % and 200 % of 1380 logical pixels wide (the size the
  editor opens at on a 1920 by 1080 screen, `DEFAULT_WIDTH`), each within the grip's limits
  (860 to 3108), then FIT TO SCREEN, the size it opens at on this screen (R6). A size whose
  window, the panel shown or hidden as it is, would be wider or taller than the usable screen
  cannot be chosen; the size the window is, marked. A size chosen is asked of the host at the
  next frame, as the grip's are, and saved with the session as theirs are.
- **The grip had not been seen.** Drawn at the drawing's corner (10 units in), it lay under
  the strip's aluminium trim (28 units at its right end, 6 along its foot), drawn after it: in
  CLASSIC it did not show at all, in REALISTIC only its ends. It is now drawn last, its corner
  inside the trim (36 and 14 units in), four lines instead of three, at 65 % instead of 30 %
  and 4 units wide instead of 3; the compact view's grip likewise (four lines, 65 %), its
  square 44 units instead of 30. The grip's area is 90 by 60 units.
- **Its tip,** DRAG TO RESIZE, over it, its right end at the window's. **Its pointer**, the
  resize arrows (`MouseCursor::NwseResize`) over the grip and while it is dragged, on Windows
  and Linux. Not on macOS: baseview's macOS `set_mouse_cursor` is `todo!()` (it would panic).
- **A menu too tall for the view is made smaller** (`Menu::placed`), its lettering with it:
  the large lettering's settings' menu ran off the compact view's foot in a window at its
  smallest (860 wide).
- **Not done: the host's window edge.** nih-plug's VST3 and CLAP wrappers answer that the
  editor cannot be resized (`can_resize`), so a host's edge does nothing. Doing it means the
  wrappers' size constraints and the hosts' resizes within resizes that have aborted hosts
  before (R26), in plugin-kit's nih-plug; it is left for its own change.
- **PR #40 kept out.** Its Linux repaint presents a frame within the input event only for a
  height-only resize (PANEL); a SIZE choice changes the width, so it would not take that path.
  The owner had found on 2026-10-14 that it did not remove the visible stretch in Bitwig
  (R-RELEASE-0.2.3), and it conflicts with `main`.

**Evidence (the Mac, 2026-10-15):** `the_settings_key_s_size_menu_resizes_the_window`,
`the_grip_says_what_it_does`, `a_menu_too_tall_for_the_view_is_made_smaller` and
`a_menu_too_tall_is_made_smaller`; with R-LETTERING's, `cargo test -p ca72-panel -p
ca72-plugin --lib --tests`: 325 passed, none failed; clippy with `-D warnings`.
`the_lettering_pngs` writes the SIZE menu and the grip's tip. Not tried: a host.

## R-RELEASE-0.2.4. 0.2.4

**Owner authorisation, 2026-10-15.** Release R-SIZE alongside R-LETTERING ("add the menu size
group + grip then release that along side the new lettering option"), PR #40 reviewed and kept
out (R-SIZE). The request authorises merging, signing, tagging, upload and publication.

The sound engine, parameter identifiers, preset format and installers are unchanged from 0.2.3;
the editor changes, and the computer's settings file gains one key, `lettering` (SIZE's width
is the session's, as the grip's is). Release notes are in `releases/0.2.4.md`.
Before publication the release commit must pass all three CI jobs; the macOS installer must be
Developer ID signed, notarised, stapled and accepted by Gatekeeper; every uploaded asset
downloaded again and checked against the manifest. The evidence is recorded when those checks
complete.

**Owner check, 2026-10-16.** The owner installed the signed macOS installer before
publication: "I tried the 0.2.4 build on my Mac and it looks good. Finish the draft and publish
0.2.4."

**Publication evidence, 2026-10-16.** [CA-72 0.2.4](https://github.com/idlefoundry/ca-72/releases/tag/v0.2.4)
is public and Latest (also to an anonymous request, which the update check makes). PR #44 was
fast-forwarded to `main` at the exact tested commit `ec5d2aee04ce3b7320f64eb105281ccd5cc62d2c`,
which the annotated `v0.2.4` tag names.
- [PR CI 38843349685](https://github.com/idlefoundry/ca-72/actions/runs/38843349685) and
  [tag CI 38845351212](https://github.com/idlefoundry/ca-72/actions/runs/38845351212) passed on
  Linux, Windows and macOS; the tag's drafted the release.
- The macOS installer was made from the PR run's unsigned universal bundles (artifact digest
  `sha256:afa8fa2d…`, checked), signed with Developer ID, notarised (submission
  `45123f3e-359c-4db0-87be-32bdaa6c16fb`, Accepted), stapled and accepted by Gatekeeper, on the
  Mac from the agent's own session (no detour through a desktop shell was needed). Each
  bundle's executable and data sections are the CI build's, byte for byte; the Audio Unit's
  embedded CLAP is the signed CLAP.
- `CA-72-0.2.4-macOS.pkg` sha256 `ef6b1052b913ba7ada7826ed12e654f5c784d747acece1e6cffa55ed5688fa65`,
  90,797,995 bytes. All nine assets were downloaded again and checked against
  `SHA256SUMS.txt` and GitHub's digests; the published installer, downloaded once more with
  the quarantine flag set, passed `pkgutil`, `stapler validate` and Gatekeeper ("Notarized
  Developer ID").
- On the lab share macOS writes `._` files beside each download; the downloads are checked in
  a local folder (`~/.cache`). The scripts and logs are in `~/lab/artifacts/ca-72/0.2.4/`.

## R-AU-PITCH-BEND. Preserve the AU MIDI pitch wheel (issue 43)

The AU adapter converted channel pitch bend into CLAP per-note tuning whenever
an instrument preferred typed CLAP notes, including ports that also support MIDI.
nih-plug maps that expression to `PolyTuning`, which CA-72 does not follow; its
`MidiPitchBend` handler never received the wheel. The conversion also imposed a
fixed two-semitone range. Preserve the original MIDI bytes when the port supports
MIDI or MPE, so the instrument applies its own MIDI BEND RANGE. CLAP-only ports
retain the existing tuning fallback. Other event translations, the DSP and the
production component identity are unchanged.

The AU build now runs five adapter regression groups and nine in-memory public-AU
render cases. These exercise MIDI bytes and MIDI 1/2 event-list entry points,
ranges 0/2/7, both bend directions, centering, CC121 reset, note delivery/release,
CC120/123 and MIDI 2's distinct zero-velocity semantics. The tests use disposable
state without physical audio or MIDI output; event-list checks require macOS 12.
They do not establish native end-to-end MIDI 2 hardware support.

On macOS 27.0/M4 Pro, the fixed 0.2.4 review build passes all five adapter groups,
all nine render cases and strict auval under a separate arm64 test identity.
The embedded engine's 28 compiled code/data sections match the installed 0.2.4
engine. The unchanged release fails the six nonzero-range render cases. The
previous 0.2.3 GarageBand reproduction and Apple DLS control establish the original
host symptom, but fixed-build GarageBand playback remains blocked by the unsaved
`ca74-au-test` project; the project switch was canceled. Live/physical controller
input and Logic have not been tested. These results do not claim resolution in
GarageBand or Logic. No test-identity packaging is part of the production change.

## R-MAC-FIXES. The Mac's own: the lab's VCO bench, and the pointer over the grip

**Owner decision, 2026-10-16.** After 0.2.4: "please identify and fix those Mac specific
issues": the three circuit-lab tests that fail on the Mac (R60's notes), and the pointer
that does not change over the grip on macOS (R-SIZE).

**Agent decisions, 2026-10-16** (not separately approved):
- **The lab's VCO bench.** `vco_electrical` (twice) and `vco_osc23` stopped on the Mac with
  Homebrew's ngspice 47, built with KLU: "Timestep too small ... trouble with node \"hold\"",
  0.8 ms into the run. The bench holds the ramp at -2 V through a switch that opens 1 us in;
  closed it was 1 milliohm, open 1e14 ohm, a ratio of 1e17 that KLU's factorisation did not
  survive. Either 1 ohm closed or 1e12 open passed; 1 ohm closed is chosen, so that the open
  switch's leak (what the measurement sees) stays as it was, at 2 pA at most.
- **The pointer on macOS.** baseview's macOS `set_mouse_cursor` was `todo!()`; it is now
  baseview's change 7 (`third_party/baseview/PATCHES.md`): each look mapped to an `NSCursor`,
  the diagonal resize arrows to the ones AppKit's own window corners use, shown again as the
  pointer moves and as AppKit updates it. The editor sets it on every system (R-SIZE's).
  plugin-kit's baseview has the same `todo!()`; it is to take the change too.

**Evidence (the Mac, macOS 27.0, 2026-10-16):** `cargo test -p ca72-lab --test vco_electrical
--test vco_osc23`: 5 passed (3 had failed), with either change alone as well. `NSCursor`
answers to `_windowResizeNorthWestSouthEastCursor` and `…NorthEastSouthWest…` on this Mac.
Not tried: the pointer seen over the grip in a host (to be checked in the next build).

## R-LO-DEFAULT. QUALITY at LO for a new instance

**Owner decision, 2026-10-16.** During 0.2.4's release: "Can we make the plugin default to “LO”
quality setting for new users?", and "You can push this update first if changing this single
setting requires you to rerun CI, but if you can make that change without, please do so now."
A parameter's default is in the binaries, so it needed a new build: 0.2.4 was published as
tested, and this follows it.

**Agent decisions, 2026-10-16** (not separately approved):
- **What changes.** QUALITY's default is LO (Potato mode, R47), not HI, so a new instance
  (and the standalone's first session) plays the light voices: 2.1 to 3.8 % of a core for ten
  POLY voices against 78 to 107 % at HI (R47's measurement, three presets), so a first try
  works on a slower computer. "New users" is taken as new instances: a parameter's default cannot tell a
  new user from an existing one, and the choice per computer that could (like LOOK) would hide
  the default from a host's own reset.
- **What does not.** A project keeps the QUALITY it was saved with (every 0.2 session saves
  it); presets never carried it (`library::KEPT`). A session saved before QUALITY existed
  (0.1's) opens at HI, as it did from 0.2.0 to 0.2.4, not at the new default
  (`Ca72::filter_state`), so no old project changes its sound.
- **Tests.** `a_new_instance_is_at_lo_and_an_old_session_at_hi`; the two editor tests that
  began from a new instance's HI (`quality_goes_where_its_toggle_is_clicked`,
  `quality_s_opening_follows_quality_and_ultra_s_note_is_read_once`) begin at HI explicitly.

## R-RELEASE-0.2.5. 0.2.5

**Owner authorisation, 2026-10-16.** "Once PR #46's CI is green, bump to 0.2.5 and release it
the same way as 0.2.4": R-MAC-FIXES and R-LO-DEFAULT. The request authorises merging, signing,
tagging, upload and publication. On 2026-10-17 the owner added PR #47, R-AU-PITCH-BEND (issue
43): "release both of those right now". #47 was merged to `main` first and merged into 0.2.5's
branch.

The sound engine, parameter identifiers, preset format and installers are unchanged from
0.2.4; QUALITY's default, baseview's macOS pointer and the AU wrapper's pitch bend change, and the
circuit lab's VCO bench (not part of the plug-in). Release notes are in `releases/0.2.5.md`. Before publication the
release commit must pass all three CI jobs; the macOS installer must be Developer ID signed,
notarised, stapled and accepted by Gatekeeper; every uploaded asset downloaded again and
checked against the manifest.

**Publication evidence, 2026-10-17.** [CA-72 0.2.5](https://github.com/idlefoundry/ca-72/releases/tag/v0.2.5)
is public and Latest (also to an anonymous request, which the update check makes). PR #47 was
merged to `main`, then merged into PR #46's branch; PR #46 was fast-forwarded to `main` at the
exact tested commit `212967e3efe0945c60c44fa4272f51a854d5f3eb`, which the annotated `v0.2.5` tag
names. No host test of the installed 0.2.5 was made before publication; R-AU-PITCH-BEND's
GarageBand and Logic checks remain open.
- [PR CI 39066363897](https://github.com/idlefoundry/ca-72/actions/runs/39066363897) and
  [tag CI 39068050615](https://github.com/idlefoundry/ca-72/actions/runs/39068050615) passed on
  Linux, Windows and macOS; the tag's drafted the release.
- The macOS installer was made from the PR run's unsigned universal bundles (artifact digest
  `sha256:fb77cc38…`, checked), signed with Developer ID, notarised (submission
  `b9a0ee91-5eed-4f94-88f0-4384f4a72b64`, Accepted), stapled and accepted by Gatekeeper, on the
  Mac from the agent's own session.
- `CA-72-0.2.5-macOS.pkg` sha256 `8b8745fa04c1f5f269a9ba8f1ae7150f63efe2e8cf77d399f7c5cd00f59be2f5`,
  90,804,304 bytes. All nine assets were downloaded again and checked against
  `SHA256SUMS.txt` and GitHub's digests; the published installer, downloaded once more with
  the quarantine flag set, passed `pkgutil`, `stapler validate` and Gatekeeper ("Notarized
  Developer ID").
- The scripts and logs are in `~/lab/artifacts/ca-72/0.2.5/`.

## R-THREADS. Threads by the machine: measurements, and a proposal for approval

**The owner's request, 2026-10-16** (relayed by the session that started this work). Assign
the plug-in's threads by the machine's processor, so that stronger machines play more
voices and more instances, and ULTRA live. Real-time safety comes first (history.md: on
2026-09-30, more spinning real-time threads than free CPUs froze a machine). The owner's
conditions:
- today's count stays the minimum;
- the maximum rises with the processor wherever more workers measurably help;
- on 4 cores or fewer ULTRA stays as it is;
- with more cores, ULTRA's voices are spread over them (the six-thread voice);
- no lookahead and no added latency.

**Status: approved and built on the branch (2026-10-16), not released.** Decisions 1 to 3 were
approved as proposed, 4 deferred (AUTO only) and 5 tried (below); what was built, and how it
measured, is at the end of this record. The owner decides how it is released.

### How it was measured
- **`tests/scaling_bench.rs`** runs 1 to 8 instances in one process. Each runs on a host
  audio thread of its own (promoted, in a work interval of its own on macOS, as a CoreAudio
  IO thread is), all paced to the same period from the same start, as a host's tracks are.
  Each plays `poly_bench`'s load, a changing chord every half second with the modulation
  moving: 10 voices at HI, 4 at ULTRA, the instances' chords on different roots changing at
  the same blocks. The budget of workers for the process is set from 0 to 12
  (`engine::set_budget_for_measurement`) and shared evenly among the instances, or, as the
  plug-in does today, the first instance takes all 4 (R18, R21). Each run is 2 s of warm-up
  and then 10 s measured, at 48 kHz, 256 and 128 frames.
- Each cell gives the host threads' blocks: the mean and worst share of the period, the late
  blocks, and the voices' runs not heard. A run is not heard when a worker was not done by
  the deadline, 0.75 of the period (R11), which is a gap in one voice: an audible fault even
  when no block is late.
- **`tests/ultra_split.rs`**, the prototype of ULTRA's split voices, plays 1 to 4 ULTRA voices
  through the plug-in's own pool (`pool::Pool`). Some or all of them are each on six threads
  (`ca72::threaded::Threaded`, now with DRIVE), the rest whole, all from one budget of 4 to
  14 threads: each split voice's five helpers first, the pool's workers what is left. The
  load is `poly_bench`'s panel at ULTRA, DRIVE at 6 dB, no EXTERNAL INPUT, one host thread.
- **The grids:** `scripts/scaling-sweep.sh` (and `.ps1` for Windows) and
  `scripts/ultra-split-sweep.sh`; `scripts/scaling-table.py` makes their tables. The raw
  lines are in `docs/threads/`.
- **Machines.**
  - **The Mac:** Apple M4 Pro, 14 cores, of them 10 performance (`hw.perflevel0.physicalcpu`)
    and 4 efficiency. Ableton Live 12 was open throughout (about half a core). During the
    first sweep's HI 256-frame runs with 4 and 8 instances, other sessions' builds and
    benchmarks ran too (load average up to 24). Those 4-instance points were run again, twice,
    on a quieter machine (`scaling-mac-repeats.txt`; the repeats are what the tables show). The
    8-instance points were not run again and are left out of the conclusions. From about
    15:05 to 16:30 local time, the whole of the first sweep, another session's work also ran:
    builds, clippy and renders at nice 10 to 19, and from 15:39 to 16:21 `preset_cost` on one
    thread at normal priority. So the Mac's first sweep, ULTRA's grid among it, may be noisy.
    The repeats (from 16:45) and the split grid (from 16:31) ran after it. The repeated 1- and
    2-instance points agree with the first sweep's within 2 points of the period (27 / 40
    against 25 / 40, 46 / 73 against 46 / 73).
  - **The Linux reference machine:** Ryzen 7 7800X3D, 8 cores, 16 hardware threads, one kind of
    core, real-time throttling off as the owner set it. Its load average was 1.5 to 3 before the
    runs (other users' applications).
  - **The Windows machine:** i5-13600K, 6 performance cores with two hardware threads each and
    8 efficiency cores, 20 hardware threads in all.
  - The three machines ran their grids at the same time, each only its own (they share no
    processor). Each machine's runs were one at a time.
  - **The cores as each system reports them** (2026-10-16):
    - the Mac: `hw.nperflevels` 2, `hw.perflevel0.physicalcpu` 10 (`logicalcpu` 10),
      `hw.perflevel1.physicalcpu` 4, 14 logical;
    - Windows: `GetLogicalProcessorInformationEx` (cores), `EfficiencyClass` 1: 6 cores, 12
      logical processors; class 0: 8 cores, 8 logical; 20 in all;
    - Linux: no `/sys/devices/cpu_core` (not Intel hybrid); `cpu_capacity` 1024 on all 16, and
      one `cpuinfo_max_freq` (5.05 GHz); 8 physical cores (`core_cpus_list`), 16 logical.
- **Safety in the runs.** On Linux and Windows, no scaling run had more host threads and
  workers together than the hardware threads less two. On Linux no split run had more threads,
  the host's among them, than the hardware threads less two. On Windows one split run had 19
  on 20 (three voices split). On the Mac, where the system limits a time-constraint thread's
  computation, the grids ran in full (up to 20 threads on 14 cores). No run on any machine had
  a watchdog demotion (the watchdog is Linux's).

### What the measurements show

**1. Today's first-come budget starves every instance after the first.** HI, 10 voices each,
256 frames: the mean / the worst block, % of the period, · late blocks · runs not heard.
Instances share the number of workers in the column evenly:

| Machine | Instances | Today (4, first come) | 4 shared | 6 | 8 | 10 | 12 |
|---|---|---|---|---|---|---|---|
| Mac | 1 | 27 / 40 · 0 | 27 / 40 · 0 | 24 / 137 · 1 | 24 / 43 · 0 | 20 / 38 · 0 | 19 / 35 · 0 |
| Mac | 2 | 67 / 177 · 1,820 | 46 / 73 · 0 | 35 / 56 · 0 | 27 / 41 · 0 | 27 / 49 · 0 | 30 / 53 · 0 |
| Mac | 4 | 159 / 347 · 5,625 | 58 / 95 · 0 · 134 | 54 / 114 · 3 · 118 | 52 / 102 · 1 · 66 | 52 / 121 · 1 · 76 | 51 / 103 · 1 · 59 |
| Linux | 1 | 38 / 66 · 0 | 38 / 66 · 0 | 37 / 64 · 0 | 31 / 51 · 0 | 24 / 39 · 0 | 23 / 34 · 0 |
| Linux | 2 | 97 / 239 · 1,875 | 63 / 106 · 3 · 224 | 50 / 81 · 0 · 17 | 46 / 70 · 0 | 42 / 65 · 0 | 41 / 65 · 0 |
| Linux | 4 | 148 / 314 · 5,625 | 88 / 237 · 1,110 · 3,069 | 77 / 236 · 490 · 2,249 | 71 / 137 · 58 · 1,784 | 66 / 123 · 15 · 904 | (not run) |
| Windows | 1 | 28 / 54 · 0 | 28 / 54 · 0 | 29 / 61 · 0 | 38 / 75 · 0 · 10 | 45 / 78 · 0 · 65 | 46 / 94 · 0 · 134 |
| Windows | 2 | 74 / 220 · 1,875 | 49 / 83 · 0 · 12 | 50 / 107 · 1 · 87 | 49 / 92 · 0 · 189 | 48 / 96 · 0 · 237 | 48 / 170 · 30 · 577 |
| Windows | 4 | 100 / 199 · 5,625 | 70 / 146 · 91 · 942 | 63 / 120 · 21 · 635 | 56 / 108 · 1 · 355 | 52 / 97 · 0 · 111 | 52 / 97 · 0 · 83 |

The blocks are 1,875 per instance. Under today's rule the second instance and every one
after it are late in every block, on all three machines. The same 4 workers shared evenly
leave two instances with no block late. With four instances the Mac has no block late (134
runs not heard), and the others have far fewer late blocks. With 8 instances (80 HI voices) no count played cleanly anywhere. More workers
still reduced the late blocks, on Windows from 13,125 (today) to 1,185 at 10. At 128 frames
the pattern is the same (`docs/threads/`).

**2. More workers help where there is work for them, and on the hybrid processor only on its
performance cores.**
- **One instance:**
  - The Mac's mean falls from 27 % at 4 workers to 19 % at 9 (one voice a thread), its worst
    block from 40 to 36 %.
  - Linux's falls from 38 to 23 % at 12 (worst 66 to 34 %): its second hardware threads help.
  - Windows' is best at 4 to 6 workers (28 %). From 7 workers on it rises (46 % at 12) and
    voices start to go unheard (134 runs at 12): the 7th thread and those after it land on the
    efficiency cores, a voice there is slower, and the host waits for the slowest.
- **Several instances:** the total that keeps helping rises. With four instances Windows has
  no late block from 9 workers on, and Linux improves through 10, its limit here (4 + 10 =
  14 threads). On the Mac, four instances have at most 8 late blocks of 7,500 at any count
  from 4 to 12, and more than 8 workers gives little.
- **No cliff at the performance-core count.** On the Mac, 4 instances with 12 workers (16
  audio threads on 10 performance cores) still had at most 1 late block in 7,500 in both
  repeats. The collapse the first sweep showed from 7 workers on came from the other sessions'
  load: it did not repeat.

**Where the curves flatten** (HI, 256 frames):

| Machine | One instance | Four instances |
|---|---|---|
| Mac | 9 workers, one voice a thread (mean 19 %); flat to 12 | 8 (52 %); flat to 12 |
| Linux | about 10 (24 %), 12 a little better (23 %) | still falling at 10, the most run |
| Windows | 4 to 6 (28 %), worse from 7 | 10 to 12 (52 %) |

None of these machines has more than 14 cores (20 hardware threads), so how the curves go
further is not known.

**3. ULTRA's whole voices are limited by one voice's worst block, not by the number of
workers.** With one instance and 4 ULTRA voices, the mean stops falling at 3 workers (one
voice a thread): 69 % of the period on the Mac, 72 % on Windows, 85 % on Linux. The worst
block stays at 2.6 to 4.8 times the period, and 0.4 % (the Mac at 10 workers) to 35 % (Linux
at 8) of blocks are late, at any count from 3 to 12. More whole-voice workers cannot play ULTRA live. With 2 or more instances at
ULTRA, nothing is close.

**4. Splitting an ULTRA voice works when every sounding voice is split.** The split prototype
(DRIVE 6 dB, one host thread): the mean / the worst block, % of the period, · late blocks ·
runs not heard. At 256 frames 1,875 blocks are measured, at 128 frames 3,750:

| Voices | Threads (budget) | How | Mac 256 | Mac 128 | Linux 256 | Linux 128 | Windows 256 | Windows 128 |
|---|---|---|---|---|---|---|---|---|
| 1 | 4 | whole | 53 / 166 · 72 | 61 / 274 · 187 | 65 / 198 · 130 | 63 / 205 · 193 | 55 / 171 · 81 | 55 / 289 · 140 |
| 1 | 6 | split | 31 / 149 · 7 | 47 / 212 · 42 | 34 / 111 · 5 | 31 / 156 · 27 | 28 / 75 · 0 | 27 / 112 · 1 |
| 1 | 12 | split | 25 / 70 · 0 | 45 / 211 · 49 | 33 / 109 · 3 | 35 / 148 · 36 | 28 / 81 · 0 | 27 / 116 · 1 |
| 2 | 12 | whole | 51 / 167 · 3 · 489 | 50 / 76 · 0 · 798 | 67 / 247 · 28 · 948 | 64 / 280 · 8 · 1,955 | 56 / 212 · 53 · 318 | 56 / 176 · 88 · 431 |
| 2 | 12 | both split | 41 / 135 · 4 · 23 | 34 / 119 · 3 · 19 | 37 / 155 · 14 · 23 | 37 / 168 · 35 · 43 | 54 / 186 · 43 · 79 | 42 / 169 · 34 · 71 |
| 2 | 12 | one split | 52 / 169 · 38 · 218 | 48 / 186 · 17 · 376 | 66 / 205 · 69 · 399 | 63 / 262 · 77 · 649 | – | – |
| 3 | 12 | whole | 51 / 276 · 11 · 678 | 50 / 192 · 8 · 1,171 | 74 / 305 · 77 · 2,926 | 68 / 294 · 11 · 3,537 | 60 / 277 · 64 · 437 (14) | 60 / 248 · 86 · 896 (14) |
| 3 | 12 | two split | 49 / 187 · 27 · 228 | 51 / 185 · 35 · 576 | 70 / 226 · 77 · 593 | 64 / 183 · 44 · 763 | – | – |

- **One voice** (the one instrument, POLY off, is this case). Split, it takes about half the
  time. Its late blocks fall from 4 to 7 % to 0 to 0.4 % at 256 frames (none on Windows), and
  to about 1 % at 128 frames (Windows 0.03 %). It needs 6 threads: the host's, which plays the
  front, and five helpers. More threads change nothing.
- **Two voices:** splitting both (12 threads) cuts the runs not heard from 300 to 2,000 to
  about 20 to 80, and the means fall on the Mac and Linux. It is not clean: 4 to 43 late
  blocks of 1,875. Splitting only one is no better than splitting none: the whole voice's
  worst block is still there.
- **Three or four voices** cannot all be split within 14 threads. Splitting some takes
  workers from the pool and is no better, or worse. With no pool worker left (four voices, two
  of them split, 10 threads) every block is late.
- **The keyboard part remains the limit** (history.md: a helper for it was slower). The Mac's
  worst split blocks are worse than Windows'. The Mac's helpers are not in a work interval as
  the pool's workers are (R11), and the system may clock them down, as R11 found for the
  workers. That is the first thing to try there.
- The prototype's samples are the whole voice's to the bit, with DRIVE
  (`the_split_voice_with_drive_is_the_voice`, which also checks that DRIVE changes them).

### The proposal (for the owner's approval)

**A. What the machine is.** At start, off the audio thread, two numbers:
- **N**, the hardware threads the process may use (`available_parallelism`, as today);
- **F**, the fast ones: the performance cores, counting their second hardware threads only on
  a processor with one kind of core.

| Machine | macOS | Windows | Linux | Measured here |
|---|---|---|---|---|
| How F is read | `hw.perflevel0.physicalcpu` (all cores where `hw.nperflevels` is 1) | `GetLogicalProcessorInformationEx`: the cores of the highest `EfficiencyClass`; their logical processors only when every core has the same class | the physical cores among `/sys/devices/cpu_core/cpus` (Intel hybrid), else those whose `cpu_capacity` is the largest (ARM), else every hardware thread of the affinity mask | |
| M4 Pro | F = 10, N = 14 | | | yes |
| i5-13600K | | F = 6, N = 20 | F = 6 | Windows only |
| 7800X3D | | F = 16 | F = 16, N = 16 | Linux only |

Beyond 14 cores (20 hardware threads) the formulas below are not verified: their values
there follow from the rule, not from a measurement.

Measured: on Windows the 7th thread of one instance and those after it are slower (item 2),
so the second hardware threads of a hybrid processor's performance cores are not counted.
On the 7800X3D the second threads help, so they are.

**B. How many workers.**
- **The process's budget:** B = max(today's count, N − 2 − A), where A is the number of the
  plug-in's instances with POLY on (each has a host thread of its own). Today's count is
  `workers_for(N)`, the floor the owner asked to keep. Two hardware threads are left for the
  system and the host.
- **An instance's cap:** C = max(today's count, F − 2), and never more than its VOICES less
  one (a worker without a voice of its own does nothing).
- **Shares:** each instance with POLY on holds min(C, its even share of B), the remainder of
  the division going to the first ones. Its host thread plays the rest, as today.

What this gives on the machines measured, against the table in item 1:

| Machine | 1 instance | 2 instances | 4 instances | 8 instances |
|---|---|---|---|---|
| Mac (F 10, N 14; today 4) | 8 (B 11): 24 / 43 · 0 | 5 + 5 (B 10): 27 / 49 · 0 | 2 each (B 8): 52 / 102 · 1 | 1 each, 4 of them (B 4, today's floor) |
| Linux (F 16, N 16) | 9 (VOICES 10; B 13): 26 / 40 · 0 | 6 + 6 (B 12): 41 / 65 · 0 | 3 + 3 + 2 + 2 (B 10): 66 / 123 · 15 · 904 | 1 each, 6 of them (B 6) |
| Windows (F 6, N 20) | 4 (C 4): 28 / 54 · 0 | 4 + 4 (B 16, C 4): 49 / 92 · 0 · 189 | 4 + 4 + 3 + 3 (B 14): about 52 / 97 · 0 (12 measured) | 10 shared (B 10): 82 / 227 · 1,185 |

Each choice against the measurements:
- **N − 2 − A** keeps the plug-in's audio threads and the instances' host threads together
  within the machine less two. That is the limit the Linux and Windows runs kept to safely,
  and below the more-than-CPUs case that froze the machine in September. Item 2 shows the
  total keeps helping up to it.
- **F − 2** keeps a lone instance on the fast cores: on the hybrid processor, workers past them
  made one instance worse (item 2). It is today's 4 on the i5-13600K.
- **Even shares** remove today's starvation (item 1). Shares are not fitted to each instance's
  load: an idle instance with POLY on still holds its share, as today.

**C. Shares that follow the instances.** The budget counts the instances with POLY on and
publishes a generation when the count changes or an instance's share does. At a block's start
each instance's audio thread compares the generation (one atomic load). If it changed, the
audio thread asks its helper thread (R23) to start its pool again at its new share, as when
POLY is switched on (R21). An instance's voices play on its host thread until the new pool is
handed over: no wait, no allocation and no thread started on the audio thread, the same samples
(R11). A shrinking instance gives its workers back before a growing one claims them; a claim
that finds too few takes what there is and asks again at the next generation.
- **Considered, not proposed: one pool for the whole process.** All instances' voices would go
  into one queue for all workers. That uses idle shares, but couples the instances: one
  instance's late voice holds a worker another is waiting for, and every host thread would then
  wait on the others' work. Not measured; shares are simpler and their effect is measured.

**D. When ULTRA splits a voice, and how many.**
- **How many split voices, S:** none where F < 6 (the owner's rule for 4 cores or fewer; 5 is
  not measured, so none there either), one where F is 6 to 9, two where F ≥ 10. Three or more
  are not proposed until measured: 18 threads were not run on the Mac or Linux, and on Windows
  3 split were not clean (52 late blocks, 184 runs not heard). By A: the Mac 2, Linux 2,
  Windows 1.
- **When:** only at ULTRA, without FEEDBACK (the six-thread voice plays its parts in turn
  with FEEDBACK on, a one-sample loop: no gain), and only when every sounding voice of the
  instance can be split: POLY's or UNISON's sounding voices, with DOUBLE's twins, no more than S.
  Otherwise every voice plays whole, as today (item 4: splitting some is no gain).
- **Threads:** a split voice takes 6 threads: its front on the thread that plays it (the host's
  or a pool worker's) and 5 helpers. They come out of the same budget B and count against the
  instance's share. So the one instrument at ULTRA (POLY off) holds 5 helpers from B. This
  changes R21's "workers only while POLY is on": the helpers are held only while ULTRA is
  chosen and S ≥ 1.
- **How, in the engine** (the prototype did not do this):
  - Each instance keeps up to S crews of five helpers.
  - A voice that is to split lends its parts to a crew: the keyboard, the contours, the VCA's
    bias, the preamplifier and the back move into the crew's job slots between runs, and back
    when the voice stops sounding or ULTRA is left. That needs a `Voice` that can be taken
    apart and put together again (`into_parts`, a new `from_parts`) without allocating.
  - Notes stay at their samples: the six-thread voice steps its keys itself, as the whole
    voice does.
  - ENTROPY's character and SPREAD's place are worked out per sample before the run (they do
    not depend on the voice's output).
  - DRIVE is set before each run (`Threaded::set_drive`).
  - Crews are started and stopped by the helper thread, never on the audio thread.
- **Expected:** on the Mac, the one instrument at ULTRA at 256 frames 0 to 7 late blocks in
  1,875, against 72 whole; two POLY voices at ULTRA about 4 late and 23 runs not heard, against
  3 and 489. ULTRA's note (R48) can say what this machine plays.

**E. The override (Advanced).** *Deferred by the owner (below): AUTO only for now; none of
this is to be built yet.* The computer's settings file (`settings.toml`, R48) gains
`threads`: `auto` (A to C) or a number from 0 to N − 2, an instance's workers (0: every voice on
the host's thread). It also gains `ultra_split`: `auto` (D), `off`, or a number from 0 to 2.
In the settings menu: ADVANCED › THREADS… with AUTO and the numbers, showing what AUTO
chose and this machine's F and N, and ULTRA SPLIT: AUTO, OFF. The override is the computer's,
not the session's, as LETTERING is (R-LETTERING), and is clamped by the same safety limit
(B never above N − 2 − A).

**F. What stays as it is.** Every wait is bounded (`Backoff`); the Linux watchdog and its
demotions; the deadline and its silent runs; one voice played whole by one thread outside
ULTRA's split; the samples (the same to the bit with any number of threads, R11, and split,
item 4); no lookahead, no added latency.

### For the owner to decide
1. The count: B = max(today's, N − 2 − A), C = max(today's, F − 2), shared evenly (B, C).
   **Approved.**
2. Shares that follow the instances (C), rather than one pool for the process. **Approved.**
3. ULTRA's split: S = 0, 1 or 2 by F, only when every sounding voice can be split, the one
   instrument at ULTRA included (so it holds threads with POLY off) (D). **Approved.**
4. The override's form (E). **Answered:** AUTO only for now.
5. Before building D on the Mac: whether to try the helpers in a work interval first (item 4).
   **Answered:** yes. It was tried; below.

### The owner's answers on 4 and 5, and the work interval tried

**The owner, 2026-10-16** (relayed): "Why don't we just try everything on auto right now with
regards to question 4? And then, yes, try the cheaper option on Mac to see if it helps
anything." So there is no override for now: no `threads` or `ultra_split` setting and no
ADVANCED › THREADS… menu. Decisions 1 to 3 are still open.

**What was tried (the prototype only).** In `tests/ultra_split.rs`, `CA72_HELPER_WG=1` puts
the split voices' helpers in a work interval of their own (`ca72_rt::WorkInterval`, as the
pool's workers are in theirs, R11). The host's thread cannot join one (R11), and only a member
may open or close it. So a driver thread, a member too, is woken at each block's start and
opens the interval with the period as its deadline, then closes it when woken at the block's
end. That mirrors the pool, whose first worker opens its interval at a block's first run. The
interval opened once per block in every run (2,250 at 256 frames, 4,500 at 128, warm-up
included). `ca72::threaded` and the plug-in are unchanged.

**How it was measured.** The Mac, from 17:14 local time:
- Three configurations, each at 256 and 128 frames, with and without the interval, run in
  turn and repeated three times, 10 s each.
- The machine was not quiet, but each with/without pair ran under the same conditions:
  - repeat 1: another project's test (`voices_inert`, one core);
  - repeat 2: `npm` processes (about three cores);
  - repeat 3: Ableton Live 12 open (about half a core), as throughout.
- Load average 5.7 to 6.5. No watchdog demotions. Raw lines: `docs/threads/split-helper-wg-mac.txt`.

**Results:** the three repeats' values, without → with the interval: the mean and the worst
block, % of the period; the late blocks (of 1,875 at 256 frames, 3,750 at 128); runs not heard.

| Frames | Voices, threads | Mean | Worst block | Late blocks | Not heard |
|---|---|---|---|---|---|
| 256 | 1 split, 6 | 24, 34, 35 → 25, 30, 30 | 69, 116, 100 → 70, 86, 86 | 0, 4, 0 → 0, 0, 0 | 0 → 0 |
| 256 | 1 split, 12 | 29, 36, 34 → 29, 31, 30 | 129, 123, 148 → 92, 94, 92 | 1, 6, 5 → 0, 0, 0 | 0 → 0 |
| 256 | 2 split, 12 | 37, 41, 40 → 38, 37, 38 | 134, 135, 132 → 123, 125, 138 | 4, 4, 4 → 2, 4, 5 | 7, 18, 20 → 6, 4, 6 |
| 128 | 1 split, 6 | 42, 35, 34 → 31, 27, 30 | 199, 181, 171 → 91, 124, 124 | 36, 14, 17 → 0, 1, 3 | 0 → 0 |
| 128 | 1 split, 12 | 34, 34, 40 → 30, 30, 32 | 206, 173, 167 → 106, 120, 124 | 17, 7, 14 → 4, 2, 8 | 0 → 0 |
| 128 | 2 split, 12 | 42, 44, 42 → 37, 36, 38 | 184, 182, 191 → 117, 121, 122 | 20, 35, 23 → 4, 6, 5 | 150, 188, 143 → 24, 18, 16 |

**What it shows:**
- **It helps, most at 128 frames.**
  - One split voice: the worst block falls by about a third at 128 frames and its late blocks
    from 14–36 to 0–3 of 3,750, close to Windows' (item 4). At 256 frames no block is late in
    any repeat.
  - Two split voices at 128 frames: late blocks fall from 20–35 to 4–6, the runs not heard from
    143–188 to 16–24, the worst block from about 185 % to 120 %.
  - At 256 frames two split voices change little: the late blocks stay at 2 to 5.
- So, as R11 found for the pool's workers, the system ran the helpers slower without an
  interval.
- **This run's baseline is better than the split grid's** (one split voice, 6 threads, 256
  frames: 24–35 % mean and 0–4 late here, against 31 % and 7 earlier): the machine's state
  differs from run to run. The pairs above are what compare.
- **Proposed for D, and built (below):** on macOS each crew's helpers join a work interval of
  their own. One of them opens it at a block's first pass and closes it when the block's passes
  are done, as the pool's first worker does, rather than a driver thread (that thread was the
  prototype's shortcut).

### Not done
- Nothing beyond 14 cores or 20 hardware threads: the formulas' values there (a 32-core
  machine's budget, say) are unverified.
- A 5-core machine, a 6-core machine without hybrid cores, Linux on the hybrid i5, Intel macs
  (one kind of core) and AMD's mixed-core processors were not measured.
- Real hosts were not used (the bench stands in for them). Other plug-ins' threads were not
  present.
- The 8-instance points on the Mac were not run again.
- ULTRA split in the engine itself, the crews, the rebalancing and the override are not built.

### The code added for the measurements (before the build, none of it changed the plug-in)
- `engine::set_budget_for_measurement` and `engine::budget_workers`: the process's budget is
  `default_workers()` unless a measurement sets its own. The plug-in never sets it, and the
  pool's budget reads `budget_workers` where it read `default_workers`.
- `Engine::silent_runs`: a count of voices' runs not heard. The plug-in does not read it.
- `Threaded::set_drive` and `BackPart::set_drive`: DRIVE for the six-thread voice (it lacked
  it). Only the prototype calls them.
- `tests/scaling_bench.rs`, `tests/ultra_split.rs` (measurements by hand, `--ignored`; the
  exactness test runs with the others), `scripts/scaling-sweep.sh`, `scaling-sweep.ps1`,
  `ultra-split-sweep.sh` and `scaling-table.py`, and the raw lines in `docs/threads/`.

### Approved, and built (2026-10-16)

**The owner, 2026-10-16** (relayed): "Yes, I accept decisions 1 through 3 as as proposed."
With 4 (AUTO only, no override) and 5 (the helpers' interval, tried above), the work: the
count (B and C), even shares that follow the instances, ULTRA's split with the one instrument
holding helpers while ULTRA is chosen, and on macOS each split voice's helpers in a work
interval opened by one of its own helpers. Not to be pushed or released yet.

**Agent decisions, 2026-10-16** (not separately approved):

**What was built:**
- **The machine** (`ca72_rt::cores`, read once): N, the hardware threads the process may use;
  F, the fast ones, read as in A. On the three machines it reads 10 of 14 (the Mac), 6 of 20
  (Windows), 16 of 16 (Linux).
- **The count** (`engine::budget_workers`, `instance_cap`, `split_voices`; `budget_for`, `cap_for`
  and `split_voices_for` the rules alone):
  - B = max(`workers_for(N)`, N − 2 − A) for A instances holding threads;
  - C = max(`workers_for(N)`, F − 2);
  - S = 0 below 6 fast cores, 1 from 6 to 9, 2 from 10.

  The plug-in asks for `engine::AUTO` at activation, where it asked for `default_workers()`.
  `default_workers()` is still today's count, now the floor.
- **The budget** (`pool.rs`):
  - An instance that needs threads takes a **seat** (`Seat`) when its first pool is started,
    and gives it up when it needs none (its pool let go with nothing asked for, a new plan, or
    the plug-in dropped).
  - The seats share B evenly in the order they came, the remainder to the first. Claims are
    counted as before.
  - At a block's start the audio thread lets its pool go and asks again (one atomic load a
    block) when either:
    - its seat's share is no longer the share it was started at; or
    - it got less than its share and some of the budget is free again.

    Its voices play on its host thread until the new pool comes (the helper thread starts it,
    R23).
  - Pools started unpaced (the tests' and measurements') take no seat and are not bounded by
    a share.
- **ULTRA's split** (`ca72::threaded::Helpers`, `pool::Split`, `Playing::play_split_from`):
  - `Threaded`'s five workers became `Parts`, shared with the new `Helpers`.
  - `Helpers` are five threads any voice lends its keyboard circuit, contours, VCA's bias,
    preamplifier and back to for a run, swapped in before it and back after it (between
    passes, under the workers' own locks; nothing is allocated). Between runs the voice is
    whole again: everything else the engine reads of it is as before. The voice's front and
    keys run on the thread that plays it.
  - A pool started for ULTRA holds the instance's crews (`Arc<Mutex<Helpers>>`). Each run, if
    every sounding voice can have its own crew, each is lent one and plays split; otherwise
    every voice plays whole.
  - The keys stepped, the notes pressed at their samples, ENTROPY's character and SPREAD's
    places are worked out before the voices play. SCATTER's turn and the places' glides follow
    them, sample by sample. The samples are the whole voice's to the bit.
  - A crew is never dropped on the audio thread. A pool takes its crews back from its voices
    before it is let go (`Play::recall`), and a voice holding the last reference to a crew
    whose pool has gone keeps it, unused, until a later pool takes it back.
- **The work interval** (macOS): each crew's helpers join an interval of their own
  (`ca72::threaded::Interval`, the plug-in's `SplitInterval` over `ca72_rt::WorkInterval`).
  The keyboard's helper opens it at its first pass after the host's block count moves on
  (`Pool::block`) and closes it when it parks; each helper leaves it before it ends.
- **R21, changed:** an instance holds threads while it needs them: workers while POLY or
  UNISON is on, crews while ULTRA is chosen on a machine that splits voices. So at ULTRA the
  one instrument (POLY and UNISON off) holds one crew of five helpers with POLY off, from the
  budget, and gives it back when ULTRA is left (its pool let go and its seat given up, as when
  POLY was switched off).
- No override (decision 4): the counts are the machine's. The measurement hooks remain
  (`set_budget_for_measurement`, now with the cap lifted while set, and
  `set_split_for_measurement`).

**Where it differs from the proposal, and why:**
1. **C caps the pool's workers, not the split voices' threads, which the share alone bounds.**
   With C (8 on the Mac) bounding them too, the Mac could not split two voices (11 threads),
   which the measurements showed it should.
2. **The cap is POLY's ten voices less one, not VOICES less one.** VOICES is dragged by
   steps (R51), and each step would start the pool again.
3. **A doubled note's twin plays on its voice's crew, after the voice**, not on a crew of its
   own. The voice's thread waits for its crew anyway, so a second crew would only be idle.
   The doubled note then takes two split runs, still about half of what it takes whole. One
   crew per sounding note.
4. **The one instrument holds one crew**, for the same reason.
5. **A pool now wakes at most its voices less one workers a run** (before, all of them).
   Idle workers spin a moment after waking, beside a split voice's helpers. With ten HI
   voices and fewer than ten workers, as many wake as before.
6. **ULTRA's need does not depend on FEEDBACK.** A voice with FEEDBACK on plays whole and
   the crew waits, so moving FEEDBACK does not start the threads again.
7. **A restart costs a moment.** Until the new pool comes (some milliseconds), the
   instance's voices play on its host thread, and blocks may be late then. This happens when
   instances are activated, switch POLY, UNISON or ULTRA, or come and go.

**Tests** (all new ones pass on the Mac; the exactness tests also pass on Linux and Windows):
- `ca72/tests/helpers.rs`:
  - one crew lent in turn to two voices gives each its own samples to the bit, notes inside
    runs, with the trigger contact, the OVERLOAD lamp and the probes;
  - the helpers open their interval once a block, close it every time and leave it when
    they end.
- `ca72-plugin/tests/ultra_split_engine.rs`: the engine's split voices are its whole voices to
  the bit, for POLY, POLY with DOUBLE, UNISON, the one instrument (scattering) and the one
  instrument with DOUBLE; with fewer crews than voices, every voice whole and the same samples.
- `realtime.rs`, `ultra_s_split_voices_neither_allocate_nor_free_while_they_play`: POLY, DOUBLE
  and FEEDBACK switched, the crews let go and asked for again; no allocation and no free on the
  audio thread.
- `pool.rs`: `seats_share_the_budget_evenly` and `the_shares_follow_the_instances` (a pool
  restarted at its new share when a second instance sits down, the short one asked again once
  some is free, both settled, and the share all back when the first leaves).
- `engine.rs`, `the_count_follows_the_machine`: B, C and S on the three machines measured and
  on a quad-core.
- `ca72-rt`: the fast cores are among all of them; a kernel CPU list read.
- **The workspace's tests** (as CI runs them, `--features ca72-plugin/standalone-production`,
  the Mac, debug): 489 passed, 3 failed, 53 ignored. The 3 are `ca72-lab`'s ngspice tests
  (`vco_electrical`'s two, `vco_osc23`'s one: "Timestep too small"), and they fail the same way
  on the base commit (`bcb6c0b`) on this Mac; nothing here touches them.
- **Lint and format:** clippy as CI runs it (`--workspace --all-targets --all-features -D
  warnings`) is clean. rustfmt is clean on every crate file. (`cargo fmt --all` cannot run in
  this worktree, which sits inside the main checkout; rustfmt was run on the files directly.)

**Measured with the built engine** (2026-10-16):
- **How.** `tests/scaling_bench.rs` with the plug-in's own counts (`CA72_AUTO=1`):
  - each instance has the plug-in's helper thread;
  - no instance is let go until every instance has played its blocks;
  - `scripts/threads-measure.sh` (`.ps1`) runs the grid, 10 s a run, one machine at a time.
- **The load is not the prototype's.** It is the scaling bench's chords with ENTROPY and
  SPREAD and without DRIVE. So the built engine's whole voices cost more than the prototype's,
  and split is compared with whole within each run (alternated, twice).
- **Conditions:**
  - the Mac: Ableton Live open (about 0.4 core), load average 4 to 6.7, no other session's work;
  - Linux: load average 2.7 to 5.6 (other users);
  - Windows: about 14 % busy before.
- **Raw lines:** `docs/threads/built-*.txt`.

ULTRA (one instance): the mean / the worst block, % of the period · late blocks · runs not heard,
for each of the two repeats:

| Machine, frames | One instrument, split | One instrument, whole | Two POLY voices, split | Two POLY voices, whole |
|---|---|---|---|---|
| Mac, 256 | 38–39 / 72–90 · 0, 0 | 56–57 / 130–132 · 40, 40 | 50–51 / 103–122 · 2, 3 · 14, 10 | 60 / 185 · 42, 41 · 61, 79 |
| Mac, 128 | 31 / 109–114 · 1, 1 | 58 / 157–167 · 86, 81 | 45 / 116–119 · 8, 6 · 25, 16 | 61 / 163–166 · 90, 90 · 130, 100 |
| Linux, 256 | 37–38 / 112–115 · 4, 2 | 80 / 186 · 157, 154 | 41–42 / 160–170 · 17, 18 · 21, 25 | 88–94 / 304 · 352, 540 · 792, 1,015 |
| Linux, 128 | 36–37 / 148–177 · 33, 29 | 79–80 / 210–213 · 317, 337 | 42 / 187–192 · 40, 37 · 31, 19 | 80 / 211–216 · 347, 303 · 1,530, 1,414 |
| Windows, 256 | 31 / 84–92 · 0, 0 | 69 / 189–217 · 108, 107 | (one crew: whole) 69 / 223 · 75, 73 | 69 / 222 · 85, 73 |
| Windows, 128 | 30–31 / 94–95 · 0, 0 | 70 / 265–321 · 202, 198 | (one crew: whole) 68 / 181–185 · 121, 122 | 68–69 / 182–185 · 123, 120 |

- **The one instrument at ULTRA:**
  - **Mac and Windows:** no late block in 1,875 at 256 frames. At 128 frames, Windows none in
    3,750 and the Mac 1.
  - **Linux:** 2 to 4 late at 256, about 30 at 128, against 154 to 337 whole.
  - **Against the prototype** (its split 6-thread rows):
    - Windows matches (0 and 0 to 1 late);
    - Linux matches at 256 (5) and 128 (27);
    - the Mac is better, as the work interval made the prototype: with it in the A/B, 0 late at
      256 and 0 to 8 at 128; in the split grid without it, 7 and 42.
- **Two POLY voices at ULTRA**, split (the Mac and Linux, two crews): late blocks an eighth to
  a thirtieth of whole, and the runs not heard a fifth to an eightieth. This is as the
  prototype found (Mac 4 late and 23 not heard, Linux 14 and 23, at 256 frames). Windows has one
  crew, so two voices play whole: the same as whole.
- **Four ULTRA voices** play whole (more than S); the counts are in the raw lines.

HI (ten voices an instance), the plug-in's own counts against the prototype's runs at the same
counts (the mean / the worst block · late · not heard; at 256 frames):

| Machine | 1 instance | 2 instances | 4 instances | 8 instances |
|---|---|---|---|---|
| Mac, built | 8: 24 / 37 · 0 | 5 + 5: 27 / 44 · 0 | 2 each: 52 / 94 · 0 · 73 | 1 each of 4: 95 / 293 · 7,483 · 805 |
| Mac, prototype | 8: 24 / 43 · 0 | 10: 27 / 49 · 0 | 8 (repeats): 52 / 94–102 · 0–1 · 64–66 | (not repeated) |
| Linux, built | 9: 26 / 39 · 0 | 6 + 6: 42 / 66 · 0 | 3, 3, 2, 2: 66 / 137 · 28 · 1,037 | 6 of 8: 116 / 320 · 7,655 |
| Linux, prototype | 9: 26 / 40 · 0 | 12: 41 / 65 · 0 | 10: 66 / 123 · 15 · 904 | 6: 113 / 307 · 6,838 |
| Windows, built | 4: 28 / 86 · 0 · 4 | 4 + 4: 52 / 98 · 0 · 252 | 4, 4, 3, 3: 49 / 78 · 0 · 78 | 10: 82 / 216 · 1,162 |
| Windows, prototype | 4: 28 / 54 · 0 | 8: 49 / 92 · 0 · 189 | 12: 52 / 97 · 0 · 83 | 10: 82 / 227 · 1,185 |

- **The shares settle where the formula puts them**, on all three machines, the instances
  started in turn as a host activates them.
- **The figures are the prototype's at the same counts, within the machines' noise.**
  - Windows' one instance had a higher worst block in this run (86 against 54 % of the
    period) with the same mean and 99th percentile (28 and 40 %): single blocks, none late.
  - At 128 frames the pattern is the same (the raw lines).
- **Against today's first come** (item 1), the 2- and 4-instance rows have no instance late in
  every block. With 8 instances the Mac's budget is its floor (4) and half its blocks are late
  (7,483 of 15,000, against 13,125 with today's rule); its first sweep suggested 8 or more
  workers would help there (about 2,000 late), but that sweep was noisy and the rule is as
  approved.

**Not done:**
- ULTRA's note (R48) still says one voice takes about a whole core.
- More than two split voices.
- Linux on a hybrid processor.
- Anything beyond 14 cores (unverified, as above).
- Real hosts.
- The restart's cost in a host while instances come and go (measured here only as each run's
  first blocks, in the warm-up).

No benchmark or real-time thread was left running on any of the three machines.

### Six and eight instances on the Mac, and a macOS rule proposed (2026-10-16)

**The owner, reviewing the eight-instance result** (relayed):
- re-measure 6 and 8 HI instances on the Mac, on the built branch;
- report whether more than N − 2 − A measurably helps on macOS;
- check for demoted time-constraint threads or a stalled system;
- propose a macOS rule if it helps and is safe, but do not build it.

**How it was measured** (`scripts/mac-budget-sweep.sh`, raw lines in
`docs/threads/mac-budget.txt`; from 18:52 local time):
- The setup:
  - the built engine at `d82e9b2`;
  - 6 and 8 instances of ten HI voices (the scaling bench's chords), each with the plug-in's
    helper thread;
  - process budgets of 4, 6, 8, 10 and 12 workers shared evenly
    (`set_budget_for_measurement`);
  - 256 and 128 frames, 10 s each.
- Three repeats, the budgets in a different order each time (4 6 8 10 12; 12 8 4 10 6;
  6 10 12 4 8), so that drift spreads across them.
- Before each run the load and the busiest processes were logged:
  - Live open throughout (0.2 to 0.4 core);
  - in some runs another application (Codex, CodexBar) took up to 0.9 core;
  - load average 3 to 12 (the bench's own threads among it);
  - no other session's builds or benchmarks.
- **A stall canary:** an ordinary thread sleeping 10 ms at a time, its worst oversleep kept
  (`canary_ms`). It stays a few milliseconds unless the audio threads keep ordinary work off
  the processors.
- **Demotion:** `log show` returns nothing from this session's shell, so the kernel's log
  could not be read. Instead, `ca72_rt::scheduled_realtime` reads the thread's scheduling from
  `thread_info` (a time-constraint thread demoted by the system's fail-safe reports
  time-sharing).
  - Checked first: a promoted thread reports scheduled as audio
    (`a_promoted_thread_is_reported_scheduled_as_audio`).
  - A promoted thread spinning without blocking is reported demoted after 1.0 s, and is still
    so 0.5 s after it blocks
    (`a_time_constraint_thread_spinning_past_its_computation_is_demoted`, by hand).
  - The bench checks each instance's host thread at every block's end. The workers were not
    checked: they park between runs.
  - Six runs at the heaviest settings were made again with this check
    (`docs/threads/mac-demotion.txt`).

**Results** (the three repeats' values: the mean and the worst block, % of the period; late
blocks and voices' runs not heard). B is what the built rule gives (N − 2 − A, at least 4): 6
for 6 instances, 4 for 8.

| Frames, instances | Workers | Mean | Worst | Late (of the blocks) | Not heard |
|---|---|---|---|---|---|
| 256, 6 (11,250 blocks) | 4 | 77–78 | 238–252 | 3,074, 3,739, 3,753 | 125–193 |
| | **6 (B)** | 69–71 | 196–269 | 138, 128, 104 | 1,938–2,005 |
| | 8 | 69–71 | 252–265 | 114, 98, 127 | 1,863–2,194 |
| | 10 | 70–72 | 280–357 | 137, 111, 112 | 2,061–2,606 |
| | 12 | 72–73 | 372–376 | 130, 126, 93 | 2,305–2,522 |
| 128, 6 (22,500 blocks) | 4 | 77–79 | 237–275 | 7,480, 7,489, 7,475 | 278–293 |
| | **6 (B)** | 67–70 | 183–270 | 427, 323, 325 | 5,957–6,179 |
| | 8 | 71 | 198–275 | 316, 260, 356 | 4,439–4,732 |
| | 10 | 72–73 | 262–284 | 418, 335, 402 | 4,467–4,895 |
| | 12 | 73–74 | 292–360 | 434, 417, 436 | 4,669–4,997 |
| 256, 8 (15,000 blocks) | **4 (B)** | 93–95 | 277–280 | 7,550, 7,637, 7,596 | 949–1,284 |
| | 6 | 92–93 | 355–387 | 4,337, 4,220, 4,319 | 3,008–4,247 |
| | 8 | 89–91 | 380–390 | 2,235, 1,772, 2,000 | 5,212–5,397 |
| | 10 | 89–93 | 397–573 | 1,956, 1,678, 2,227 | 4,533–4,999 |
| | 12 | 89–92 | 384–495 | 1,780, 1,930, 1,922 | 4,370–4,618 |
| 128, 8 (30,000 blocks) | **4 (B)** | 98–99 | 356–368 | 15,254, 15,154, 15,151 | 1,581–1,704 |
| | 6 | 93–94 | 382–471 | 9,699, 9,299, 9,410 | 6,830–8,969 |
| | 8 | 92 | 371–381 | 9,703, 7,773, 9,217 | 13,434–14,324 |
| | 10 | 90–91 | 465–495 | 6,506, 6,223, 7,162 | 9,591–11,485 |
| | 12 | 92 | 496–554 | 7,063, 7,144, 7,177 | 9,218–9,518 |

**What it shows:**
- **With 6 instances, more than B (6) does not measurably help at 256 frames.**
  - The late blocks are within the repeats' spread from 6 to 12 (93 to 138).
  - The runs not heard rise a little.
  - The worst block grows: about 250 % of the period at 6, 372 % at 12.
- **At 128 frames, 8 helps a little:** late blocks 260 to 356 against 323 to 427, and runs not
  heard about 4,600 against 6,100. 10 and 12 are no better than 6.
- **With 8 instances, B (4) leaves four instances with no worker, and each is late in every
  block**: 7,600 of 15,000 late at 256 frames, 15,200 of 30,000 at 128.
  - One worker for every instance (8) cuts the late blocks to 1,800 to 2,200 at 256 frames, a
    quarter. At 128 it cuts them to 7,800 to 9,700, a half to three fifths.
  - 10 and 12 give a little more at 128 (6,200 to 7,200 late), and nothing measurable at 256.
  - The runs not heard rise as workers are added: with workers, a slow voice is silenced for a
    run instead of the whole block being late. 5,300 runs not heard of about 300,000 voices'
    runs (1.8 %) at 8 workers, against half of all blocks late at 4.
- **The worst block is 2 to 6 times the period in every setting.** Eight instances
  of ten HI voices (80 voices) do not play cleanly on this machine at any budget. This is the
  work (item 1), not the count.
- **No demotion and no stall:**
  - every host thread was promoted, and none was demoted in any of the six checked runs:
    8 instances with 12 workers (20 audio threads on 14 cores) at 128 and 256 frames, 8 with 8
    at 256, 6 with 12 at 256 and 128, and 8 with 4 at 128;
  - the canary's worst oversleep was 5.3 to 16.8 ms in the sweep's 60 runs, and 5.5 to 7.4 ms in
    the checked six.

**On the safety margin.** macOS bounds a time-constraint thread: one that runs far past its
computation without blocking is demoted (here after 1.0 s), so a runaway cannot hold a CPU for
good, as one at `SCHED_FIFO` with throttling off could on the Linux machine (2026-09-30). Up to
20 audio threads on the Mac's 14 cores, nothing was demoted and ordinary work kept running.
So on macOS the margin can be relaxed to give every instance at least one worker, which is
where the gain is. Past that, more workers gained little, and the worst blocks grew.

**Proposed for macOS (not built; the owner decides):**
- **The rule:** on macOS only, B = max(today's count, N − 2 − A, min(A + 2, N − 2)): every
  instance holding threads at least one worker and two over, never more than N − 2 in all.
  Elsewhere B stays as approved.
- **On the Mac** (N 14) it gives 11, 10, 8, 8 and 10 for 1, 2, 4, 6 and 8 instances, against
  11, 10, 8, 6 and 4 now.
- **The measurements behind it:**
  - at 6 instances, 8 is as good as 6 at 256 frames and better at 128;
  - at 8 instances, 10 is as good as 8 at 256 and better at 128, against 4: late blocks a
    quarter at 256 and under half at 128.
- **Not measured:** more than 8 instances; this rule's numbers on another Mac.

**Not done:**
- The workers' own scheduling was not checked for demotion (only the host threads').
- The kernel's log could not be read from here.

### The macOS rule, approved and built (2026-10-16)

**The owner, 2026-10-16** (relayed): "Yes, build the Mac OS rule."

**Built:**
- `engine::budget_for_macos`: B = max(today's count, N − 2 − A, min(A + 2, N − 2)) on macOS,
  where `budget_workers` uses it (`cfg!(target_os = "macos")`).
- Linux and Windows keep `budget_for`, as approved.
- On the Mac it gives 11, 10, 8, 8, 10 and 12 for 1, 2, 4, 6, 8 and 10 or more instances.
- `the_count_follows_the_machine` checks both rules. A functional run of 8 instances on the
  Mac (the scaling bench, `CA72_AUTO=1`, 2 s) settled at 2, 2 and six of 1: 10 in all.
- The plug-in's library, `realtime.rs` and `workers.rs` tests pass, clippy as CI runs it is
  clean, and rustfmt is clean.

### Hosts that run each plug-in in a process of its own (2026-10-16)

**The owner, 2026-10-16** (relayed): "Yes, you can run the [Bitwig] test." The concern: the
budget (`pool::BUDGET`) is the process's. Bitwig can host each plug-in in a process of its own
(REAPER has a like option), and then every instance counts itself alone (A = 1) and takes a
lone instance's share.

**What each process claims, from the code.** A lone instance's budget is B(1); its pool holds
min(B(1), C, 9) workers at HI with POLY on. At ULTRA with POLY it also holds S crews of five
helpers, claiming max(workers, 6S − 1) threads (S = 2 here).

| Machine (N, F) | B(1) | C | Workers at HI | Audio threads with its host thread | 8 processes |
|---|---|---|---|---|---|
| 7800X3D (16, 16) | 13 | 14 | 9 | 10 (20 at ULTRA with POLY, at most 12 busy) | 80 on 16 CPUs |
| M4 Pro (14, 10) | 11 | 8 | 8 | 9 | 72 on 14 cores |
| i5-13600K (20, 6) | 17 | 4 | 4 | 5 | 40 on 20 |

Released 0.2.4 claims today's 4 a process (first come within it): 40 on any of them.

**Emulated on the Linux machine** (`scripts/separate-processes.sh`; raw lines in
`docs/threads/separate-processes-linux.txt`):
- **The setup:** N processes of the scaling bench, one instance each (HI, ten voices, the
  plug-in's own counts), started together and escalated 1, 2, 4, 8, each round only after the
  last was clean.
- **The guards:**
  - an ordinary-priority canary process sleeping 10 ms at a time, its worst oversleep logged;
  - a monitor that kills every bench process if the canary oversleeps more than 100 ms;
  - a second guard doing the same at `SCHED_FIFO` 98, above the audio threads, in case they
    starved the first;
  - each process's own watchdog (demotion after 500 ms);
  - a hard `timeout` on every process and on the round.
- **Conditions:** load average 0.4 before the first round. The owner's Bitwig was open, its
  plug-in host's threads at real-time priority 50 and PipeWire's at 83, the whole time.

| Processes | Audio threads | Each process | Late | Canary's worst | Watchdog |
|---|---|---|---|---|---|
| 1 | 10 | 9 workers: 25 % mean, worst 39–40 % | 0 | 0.7 ms | none |
| 2 | 20 | 9 workers each: worst 34–38 % | 0 | 1.1 ms | none |
| 4 | 40 | 9 workers each: means 22–29 %, worst 41–57 % | 0 | 4.3 ms | none |
| 8 | 80 | **aborted** about 3 s in, before any block was measured | – | **126 ms** | none (it acts at 500 ms) |

- **Oversubscription is confirmed.** At 8 processes the ordinary-priority canary was kept off
  the processors for 126 ms within seconds of the start, and the real-time guard stopped the
  round. The machine answered at once after, and nothing of the bench was left running. The
  escalation stopped there, as agreed.
- At 4 processes (40 threads) every instance was clean: better than four instances in one
  process (28 late blocks), since each had 9 workers. Too many threads is not, up to there, a
  loss in itself. What 8 shows is the risk the budget exists to bound: on a machine with
  real-time throttling off, nothing but the plug-in's own watchdog stands between 80 such
  threads and the September stall.
- **Released 0.2.4 is exposed too, less:** 4 a process, 40 for eight, the case that ran clean
  here.

**Proposed (not built; the owner decides): one budget for every CA-72 process of the user.**
- **Seats as locked files.** In a folder of the user's temporary files (`$TMPDIR` on macOS,
  `$XDG_RUNTIME_DIR` or `/tmp/ca72-$UID` on Linux, `%TEMP%` on Windows), 64 seat files. An
  instance that needs threads takes an exclusive, non-blocking advisory lock on a free one
  (`flock` on macOS and Linux, `LockFileEx` on Windows) instead of only a seat in its process's
  list, and keeps it while it holds threads.
- **Crash-safe by the kernel.** A process's locks are released when it exits or is killed, so
  no count is left behind (a named semaphore or a counter in shared memory would be, by a
  crash).
- **The count.** Each process's helper thread probes the seats (a non-blocking lock tried on
  each and let go at once if it was free) when its own instances change and twice a second.
  A is the seats held, and each instance's share is B(A) shared evenly in the seats' order.
  When A changes, the shares move and the pools restart as they do now within a process. No
  audio thread touches the files.
- **What it costs, measured** (`scripts/seats-probe.py`, Python, so an upper bound): probing 64
  seats, 8 of them held by another process, takes 33 µs on the Mac and 57 µs on the Linux
  machine, the helper thread's work twice a second. A holder killed with its locks held left
  none held.
- **Limits:**
  - Shares only, not claims, across processes: while instances restart at new shares, the
    total may pass B for some milliseconds (within a process the shared claim counter prevents
    that).
  - A sandboxed host on macOS has a temporary folder of its own: its processes share seats
    with each other, not with other hosts.
  - A host that runs plug-ins as another user does not share them.
  - Not measured in a host.

**The Bitwig check (for the owner, at the Linux machine; superseded by the steps for the build with the shared budget, below):**
1. **Mind the risk.** This branch's build, eight instances each in its own process, is the case
   the emulation stopped at 126 ms. Real-time throttling is off on this machine (the owner's
   setting); for the test it can be turned on as a safety net and back after, if the owner
   chooses:

   ```
   sudo sysctl -w kernel.sched_rt_runtime_us=950000
   sudo sysctl -w kernel.sched_rt_runtime_us=1000000
   ```

   The first command turns throttling on, the second sets it back to the owner's setting.
2. **Get the branch's build.** The branch is local to the Mac, so copy it over:

   ```
   rsync -az --exclude target --exclude .git --exclude .claude ~/Code/ca-72/.claude/worktrees/goofy-ellis-af3f5d/ omarchy-ts:Code/ca72-threads/
   ```

   Then on the Linux machine, back up `~/.clap/CA-72.clap` and copy in the new one:

   ```
   cd ~/Code/ca72-threads && cargo xtask bundle ca72-plugin --profile bundle
   ```

   The new `target/bundled/CA-72.clap` replaces it (the VST3 likewise).
3. **Watch ordinary work in a terminal.** This prints the oversleep, in milliseconds, whenever
   it passes 20:

   ```
   python3 -c "import time
   while True:
       t=time.monotonic(); time.sleep(0.01); d=(time.monotonic()-t-0.01)*1e3
       d>20 and print(round(d,1),flush=True)"
   ```

   In a second terminal, count CA-72's audio threads:

   ```
   watch -n1 'ps -eLo rtprio,comm | grep -c ca72-'
   ```
4. **Set up Bitwig.** Settings › Plug-ins › plug-in hosting mode: **Individually** (each plug-in
   in its own process).
   - Eight instrument tracks, each with CA-72: POLY on, VOICES 10, QUALITY HI.
   - One clip of ten-note chords copied to all eight, looping.
   - Play for a minute, with the DSP meter and the dropouts in view.
5. **If the desktop stops answering for a second or more, stop the transport.** The plug-in's
   watchdog demotes its threads after 0.5 s and says so in Bitwig's log (lines that begin
   `ca72:`).
6. **Compare** with the hosting mode at **Together** (one process), then with released 0.2.4
   back in `~/.clap` in both modes.
7. **Report:** the thread counts, the canary's worst, the dropouts and any `ca72:` lines.

### ULTRA's note, as this computer plays it (2026-10-16)

**The owner, 2026-10-16** (relayed): "approved", the new text for ULTRA's first-time note (R48),
with their wording for the line about dropouts: first "with many voices or too small of a
buffer, it may drop out", then "multiple" in place of "many".

**Why:** ULTRA was measured at 3.7 to 6.4 times HI's cost over the 19 factory presets (8.5 with
FEEDBACK). That was the coordinating session's measurement on the Linux machine today,
`preset_cost` at HI and with `CA72_ULTRA`; it was not repeated here. So the note's "SEVEN
TIMES" was out of date, and the note did not say that a voice may now be split.

**Updated, 2026-10-18 (the owner: "Sure, change that line"):** "FIVE" became "SIX". R-HI-PAIRS
made HI lighter after the measurement above, so ULTRA's cost against HI's rose: on the Mac (M4
Pro), `preset_cost` over the 19 factory presets, ten POLY voices, two interleaved rounds, gave
ULTRA a median of 6.3 times HI (3.9 to 9.8 across the presets) on `cpu-release` at `2c0a584`.

**Built** (`editor.rs`, `ultra_note_lines`): the note follows what the engine decides for this
computer, `engine::split_voices() > 0`. Each version's accent line, `ULTRA · NO COMPROMISES`, and
its last two dim lines (`RIGHT-CLICK QUALITY'S SWITCH TO HIDE ITS INDICATOR, OR SHOW IT.`, `CLICK
TO CLOSE`) are as before.
- **Where it splits a voice:**
  - THE CIRCUIT'S MODEL EXACTLY, AT ABOUT SIX TIMES THE WORK OF HI.
  - ON THIS COMPUTER ONE VOICE IS SPREAD OVER SIX THREADS AND
  - CAN BE PLAYED LIVE. WITH MULTIPLE VOICES OR TOO SMALL OF A BUFFER,
  - IT MAY DROP OUT.
  - IT IS MEANT FOR OFFLINE RENDERS (EXPORT, BOUNCE, FREEZE) AND SMALL
  - PROJECTS ON POWERFUL COMPUTERS.
- **Where it does not:**
  - THE CIRCUIT'S MODEL EXACTLY, AT ABOUT SIX TIMES THE WORK OF HI.
  - ONE VOICE TAKES UP TO A WHOLE CORE OF A FAST COMPUTER.
  - IT IS MEANT FOR OFFLINE RENDERS (EXPORT, BOUNCE, FREEZE) AND SMALL
  - PROJECTS ON POWERFUL COMPUTERS. PLAYED LIVE, IT MAY DROP OUT.
- **Rewrapped, as the owner allowed:** "ON THIS COMPUTER ONE VOICE IS SPREAD OVER SIX THREADS
  AND CAN BE" was 776 units wide at the note's size, wider than the old note's widest line (763,
  "IT IS MEANT FOR OFFLINE RENDERS…"). So "CAN BE" moved to the next line (760), and "IT MAY
  DROP OUT." to the last.

**Checks:**
- `ultra_s_note_says_what_this_computer_plays`:
  - the note shown is the one for this computer's split voices;
  - both versions say what they should, with the accent and dim lines as before;
  - no line is wider than the old note's widest;
  - the box is within the panel and left of the lamp's opening, at the window's opening size.
- The window at its opening size, with each version drawn (`the_ultra_pngs`, `note-split.png`
  and `note-whole.png`), was looked at: each note sits as the old one did, its box the same
  width, right of the FILTER section and left of the lamp.

### One budget for every CA-72 process of the user, built (2026-10-16)

**The owner, 2026-10-16** (relayed): "ya, build it": the seat files proposed above.

**Built** (`crates/ca72-plugin/src/seats.rs`; `pool.rs`; the helper):
- **Seat files.** 64 files in a folder of the user's own:
  - macOS: `$TMPDIR/CA-72 seats`;
  - Windows: `%TEMP%\CA-72 seats`;
  - Linux: `$XDG_RUNTIME_DIR/ca72-seats`, else `/tmp/ca72-seats-<uid>`, made private to the user
    and not used if another user owns it;
  - `CA72_SEATS_DIR` puts it elsewhere, for the tests and measurements.

  An instance that takes a seat in its process's budget also takes an exclusive, non-blocking
  lock on a free seat file (`std::fs::File::try_lock`: `flock` on macOS and Linux, `LockFileEx`
  on Windows). Giving up the seat lets the lock go, and a process's ending, however it ends,
  lets all of its locks go (the kernel's doing).
- **The count.** A is the seat files held by every process (each tried and let go at once if
  free; the look made twice, 0.5 ms apart, and only seats held both times counted), plus any
  of this process's instances that found no seat file. A seat that cannot be tried counts as
  held, so an error can only make a share smaller. Each instance's share is B(A) shared
  evenly in the seat files' order.
- **When it is counted.** When an instance of this process takes or gives up a seat (where
  pools are started and stopped), and by the helper threads at most every 0.5 s (`LOOK`; a
  helper now waits at most that long). When the count changes the shares move, and pools
  restart as they do within a process.
- **The audio thread never touches the files.** It reads its pool's share, an atomic, as
  before.
- If the folder or its files cannot be had, the budget is the process's alone, as before.

**Tests** (all pass on the Mac; the seat and cross-process tests also on Linux and Windows):
- `seats.rs`:
  - a seat taken is held to another set of the files (another process's, as the kernel counts
    locks) and free once given back or its files closed;
  - a killed holder's seats are free again (a child process holding three, killed);
  - the seats' file operations are counted on the thread that makes them.
- `pool.rs`, `the_shares_follow_other_processes_instances`: a child process holding two seats
  moves this process's shares (11 alone, 3 for three, 2 each for four), and killed, gives
  them back (5 each for two, 11 alone).
- `realtime.rs`, `the_seats_neither_touch_files_nor_allocate_on_the_audio_thread`: POLY's paced
  workers taking a seat and its file, the plug-in's helper looking at the seats, POLY switched
  as it plays. No file operation (counted per thread), no allocation and no free on the audio
  thread, and a seat file held while POLY was on.
- **The workspace's tests** (the Mac, as CI runs them): 496 passed, 3 failed: `ca72-lab`'s three
  ngspice tests, which fail the same way on the base commit. Clippy as CI runs it (and the
  plug-in's library for Windows) and rustfmt are clean.

**Separate processes, measured again** (`scripts/separate-processes.sh`, now with each process's
seats in a folder of the run's own, and every process's blocks started at one moment,
`CA72_START_AT`, after all have started and taken their seats; raw lines in
`docs/threads/separate-processes-seats-*.txt`):
- **The Linux machine,** the same guards and escalation as before (the ordinary canary, its
  limit of 100 ms, the monitor and the `SCHED_FIFO` 98 guard, the watchdog, timeouts). Load
  average about 1.4 before each round; the owner's Bitwig open throughout.
- Each process's workers and the instances it counted were sampled halfway through its blocks.

| Processes | Workers (each, halfway) | In all | B for that many | Instances each counted | Late blocks (each, of 1,875) | Canary's worst | Watchdog |
|---|---|---|---|---|---|---|---|
| 1 | 9 | 9 | 13 (cap 9) | 1 | 0 | 0.8 ms | none |
| 2 | 6, 6 | 12 | 12 | 2 | 0, 0 | 0.9 ms | none |
| 4 | 2, 3, 2, 3 | 10 | 10 | 4 | 11, 0, 10, 0 | 2.4 ms | none |
| 8 | 1 for six, 0 for two | 6 | 6 | 8 | 577–654 (one worker), 834–854 (none) | 2.9 ms | none |

(The rows for 1 and 2 are from the first round, before the common start, as each process
held them at its end; for 4 and 8, the common start's, halfway.)

- **Eight processes now stay within the budget.** They hold 6 audio-thread workers between
  them, not 72, and the canary never came near its limit: 2.9 ms, against 126 ms and the
  round stopped before the seats.
- Eight instances of ten HI voices are still too much work for this machine (nearly half the
  blocks of the two without a worker late, a third of the others'). The rule gives eight what it gives
  eight in one process (item 1), but no longer more than the machine.
- **The earlier rounds of this run, without the common start** (in the raw lines), showed what
  happens as a host starts its processes one by one.
  - Each instance, alone at first, takes a lone instance's share until the others' seats are
    counted (at most 0.5 s after they sit).
  - On this machine the first ones' workers slowed the later ones' start by seconds.
  - The counts each process kept were right for the processes alive when it counted. Then the
    last to start, at their halfway point, counted the ones that had already finished.
- **The Mac and Windows,** started together, each process one instance of ten HI voices:

| Machine | Processes | Workers (each) | In all | B | Counted | Late (each) |
|---|---|---|---|---|---|---|
| Mac | 2 | 5, 5 | 10 | 10 | 2 | 0, 0 |
| Mac | 4 | 2 each | 8 | 8 | 4 | 0, 1, 0, 0 |
| Windows | 2 | 4, 4 (the cap) | 8 | 16 | 2 | 0, 0 |
| Windows | 4 | 4, 4, 3, 3 | 14 | 14 | 4 | 0, 2, 0, 0 |

  On the Mac other work was running meanwhile (another session's `preset_render`, another
  project's tests, Spotlight; load average 17 to 23), which does not bear on the counts.
- **The lab copies** were cleaned up afterwards, and checked:
  - on the Linux machine, moved to the user's Trash with `gio trash` and seen in
    `~/.local/share/Trash/files`;
  - on Windows, the copy moved to `C:\Users\idlewile\Code\tmp\holding`, not the Recycle Bin.
  - The copied archive on Windows was deleted after it was unpacked, and each run's own
    temporary folder on the Linux machine was removed by the script.

**The Bitwig check for this build** (for the owner, at the Linux machine):
1. **Copy the branch from the Mac and build it.**

   ```
   rsync -az --exclude target --exclude .git --exclude .claude ~/Code/ca-72/.claude/worktrees/goofy-ellis-af3f5d/ omarchy-ts:Code/ca72-threads/
   ```

   Then on the Linux machine:

   ```
   cd ~/Code/ca72-threads && cargo xtask bundle ca72-plugin --profile bundle
   ```

   Back up `~/.clap/CA-72.clap` (and `~/.vst3/CA-72.vst3`), then copy in the new ones from
   `target/bundled`.
2. **Watch ordinary work** in a terminal. This prints the oversleep, in milliseconds, whenever
   it passes 20:

   ```
   python3 -c "import time
   while True:
       t=time.monotonic(); time.sleep(0.01); d=(time.monotonic()-t-0.01)*1e3
       d>20 and print(round(d,1),flush=True)"
   ```

   In a second terminal, count CA-72's worker threads:

   ```
   watch -n1 'ps -eLo comm | grep -c ca72-voices'
   ```

   And list the seats taken (the instances counted; the folder always has 64 files, the
   locks are what count):

   ```
   lslocks | grep ca72-seats
   ```
3. **Set up Bitwig.** Settings › Plug-ins › plug-in hosting mode **Individually** (each plug-in in
   its own process).
   - Eight instrument tracks, each with CA-72: POLY on, VOICES 10, QUALITY HI.
   - One clip of ten-note chords copied to all eight, looping.
   - Play for a minute. Expect about 6 worker threads in all once the eight are up (the budget
     for eight), not 72; a moment more while they start.
4. **If the desktop stops answering for a second or more, stop the transport.** The
   plug-in's watchdog demotes its threads after 0.5 s and says so in Bitwig's log (lines that
   begin `ca72:`).
5. **Compare** with **Together** (one process): the same count. Eight instances of ten voices
   will drop out either way on this machine; what the check shows is that separate processes
   no longer multiply the threads.
6. **Afterwards,** put the backed-up released plug-ins back. Real-time throttling need not be
   turned on for this build (the owner's setting is left alone).

## R-CPU-EXACT. CPU without changing a sample

**Owner decisions, 2026-10-17.** CPU use being the users' first complaint, the owner decided that
HI and ULTRA stay bit-identical to 0.2.5, "only do the changes that keep the sound the same", and
that the worker budget (R-THREADS) goes out without the band-limiting, latency and ULTRA VCA
changes; then "Okay, proceed!" to the exact shortcuts a three-agent council and the profiles named.

**Agent decisions, 2026-10-17** (not separately approved):
- **OVERLOAD's lamp driver a step every 8 samples.** The driver is a peak detector (Q25 into
  0.47 uF, let down through 780 kilohms, a third of a second) that only lights the lamp; the
  audio reads none of it. It was a nodal solve every sample beside the preamplifier's four
  substeps (ULTRA, and HI's circuit inside FEEDBACK's loop). It now steps every 8 samples, its
  step 8 samples long, on the highest the amplifier reached in them, so a peak between steps
  still lights it.
- **The filter's ladder bias keeps its temperature's part.** `VcfCircuit::bias` is worked out
  again whenever the tail current moves; the transistor at the temperature (`Bjt::at`) and C_mu
  at the fixed V_bc are kept for the ladder, V_bc and temperature they were worked out for. The
  same arithmetic, so the same bias.
- **`preset_render` takes `CA72_POTATO` and `CA72_ULTRA`,** as `preset_cost` does, to compare
  each mode to the bit.

**Evidence (the Mac, 2026-10-17, binaries built from 0d478a5 plus the switches and from this
change, run alternately):** every factory preset the same to the bit at HI (ten voices) and ULTRA
(four voices), 4 s each (`preset_render`). `preset_cost` at ULTRA, four voices, the mean of two
runs each: Feedback Funk 395 to 380 % of a core (-3.9 %); the other eighteen presets, without the
preamplifier, within ±1 % (no change beyond noise: the bias's temperature part is a small share of
it, the tail current's iterations the rest).

**Correction, later the same day, withdrawn on 2026-10-18:** a note here said ULTRA's renders
were not repeatable over time in 0.2.5. They are. The runs that differed were the comparing
script's fault: `env $e` in zsh, with `e="CA72_ULTRA=1 CA72_VOICES=4"`, set one variable (zsh does
not split an unquoted parameter), so those renders played ten voices against references of four.
The ULTRA comparisons above set both variables themselves and stand. (Found by the session that
looked into it; its branch makes `preset_render` refuse references of another length.)

## R-HI-PAIRS. Two HI voices' filters solved side by side

**Owner decisions, 2026-10-17.** HI and ULTRA stay bit-identical to 0.2.5 (R-CPU-EXACT). Of the
avenues that keep the sound, two voices per SIMD register was measured as the one worth building
(a prototype of the filter's elimination for two voices: 20,000 real HI eliminations the same to
the bit, 91 ns against 34 ns a voice); asked whether to build it, the owner: "do the HI fix
first."

**Agent decisions, 2026-10-17** (not separately approved):
- **The filter's loop elimination for two voices at once** (`vcf::solve_loop_pair`), a lane a
  voice, each lane the scalar's operations in the scalar's order (Rust does not fuse a multiply
  and an add unasked): each lane picks its own pivots, and a step the scalar skips (a zero pivot,
  a zero factor) is kept as it was in that lane, not computed (a computed `a - 0 * p` can turn
  -0.0 into +0.0). The divisions and sums of the two lanes run in one register; a single voice's
  elimination is a chain each step of which waits on the last, so two fill each other's waits.
- **`Vcf::tick_pair`**: two filters at HI (Potato, reciprocals, the same oversampling); each its
  own resampling, bias, prewarping and Newton iterations (one, or two if the first did not
  converge), their eliminations paired while both iterate and the single filter's where only one
  does. `Vcf::tick` and `Vcf::step` are untouched: ULTRA and any filter alone take them as before.
- **`Voice::before_filter`, `Voice::filter_pair`, `Voice::after_filter`**: `Voice::tick_jacks`
  cut at the filter, each part copied from it as it was.
- **Each voice and its DOUBLE twin** (`Playing::play_from`, the one instrument included): with
  the twin sounding and both at HI, both voices' keys and jacks first (neither hangs on the other's
  output), then each to its filter, the pair, and each on from it. `turned` (the one instrument
  scattering) changes the voice's own place only.
- **POLY's voices two at a time** (`Playing::play_pair`): two voices' runs a sample of each in
  turn, where both play HI on their own (no twin sounding, not split); else one after the other.
  Without workers the engine plays the list two at a time. With workers, voices are taken two at
  a time (`pool::take_some`) when the run has more voices than threads to play them (more voices
  than workers and the audio thread), so that pairing never leaves a thread idle that could have
  played a voice; each voice keeps its own lock, buffer and done mark, and the outputs are summed
  in order as before.
- **Not paired:** LO (the light voices), ULTRA (its elementary functions are the system's,
  called a voice at a time), and a lone voice without DOUBLE.
- **The loop's evaluation paired too: tried, measured, taken out** (the owner asked for it,
  2026-10-17). Two filters of one circuit evaluated a lane each (`eval_sparse` at Potato: the
  arithmetic in lanes in the scalar's order, the tables and `fast::tanh` the scalar's a lane at a
  time) gave every factory preset the same to the bit, but cost more than the elimination paired
  alone: ten HI voices, one thread, 1479 against 1541 % of a core (+4.1 %; a first version, its
  Jacobian gathered and then scattered, +6 %). The elimination is one chain of dependent steps
  that two voices fill; the evaluation is mostly independent operations that one voice already
  overlaps, so packing two costs more than it saves. Pairing `fast::tanh` and the tables
  themselves (gathers, per-lane branches) was not tried.

**Evidence (the Mac, 2026-10-17, binaries built from `cpu-release` with `preset_render`'s
DOUBLE switch and from this change):**
- `tests/pairs.rs`: two voices set apart (notes, cutoffs, emphasis, FEEDBACK, the jacks, a
  waveform and the filter's mode changed), alone and paired: 120,000 samples at HI the same to the
  bit; ULTRA and mixed qualities, each alone.
- `preset_render`, every factory preset, ten voices, 4 s, the same to the bit at HI: one thread,
  with DOUBLE at 50 %, and with 3 workers (pairs taken) and 12 (not).
- `preset_cost`, ten voices, one thread, the mean of two runs each, alternated: at HI 1785 to
  1509 % of a core in all (-15.5 %; -11 to -22 % a preset, the hats the most); with DOUBLE at 50 %,
  3491 to 2895 (-17.1 %; -13 to -24 %).
- **On x86 (2026-10-18, the same `preset_cost`, built on each machine from 5870fdd and from
  the release head, alternated, the mean of two runs):** Linux, Ryzen 7 7800X3D (under the timing
  lock): HI 2286 to 2175 (-4.9 %), with DOUBLE 4652 to 4403 (-5.4 %); Windows, i5-13600K: HI
  2457 to 2312 (-5.9 %), with DOUBLE 4908 to 4672 (-4.8 %), the presets spread more widely
  there. A third of the Mac's saving at first; why, and the SSE2 elimination that brings x86 to
  -7 to -9 %, are below ("Why x86 saved a third as much").
- The core's, real-time library's and plug-in's tests; clippy with `-D warnings`.

### Why x86 saved a third as much, and the elimination in SSE2 (2026-10-18)

**Agent decisions, 2026-10-18** (not separately approved; asked for by the session releasing
0.2.6; merging waits for the owner):
- **The cause, two parts.** (1) On x86-64 the paired elimination compiled to scalar code. Its
  lanes form (`pair::solve5_lanes`: each lane's pivot searched and its rows swapped by reading
  and writing `.0[lane]`, each lane's step kept or skipped by a select on a `[bool; 2]`) gives
  LLVM a branch per lane: on x86-64 (`x86_64-unknown-linux-gnu`, SSE2) it updated the rows one
  lane at a time, a `movsd` load, `mulsd`, `subsd` and `movsd` store and a `testb`/`je` per lane
  and entry, the matrix in a 1224-byte frame indexed by a loop over the columns (48 `mulsd`, 35
  `subsd`, 47 branches, 217 stack accesses in the elimination alone). AArch64 has the same
  structure but 32 vector registers, `fcsel` and cheap lane moves, and lost less to it. Alone,
  the loop elimination (`vcf::pair_tests::pair_timing`, 20,000 of a HI filter's own Jacobians):
  M4 Pro 101 ns a voice scalar, 35 paired (-66 %); Ryzen 7 7800X3D 88 and 54 (-39 %); i5-13600K
  76 to 83 and 49 (-36 to -41 %). (2) The elimination is a smaller part of an x86 HI voice: its
  scalar form is faster there than on the M4 (76 to 88 ns against 101) while the rest of the voice
  is about 1.45 times slower (the Ryzen's 0.2.5, 2279 % of a core in all, against the M4's 1785).
  The voice's saving over the elimination's gives its share: M4 15.5 / 66, about 23 %; Ryzen
  4.7 / 39 and 8.1 / 65, about 12.5 %; i5 5.1 / 39 and 7.4 / 62, about 12.5 %. The same share
  from both forms on both machines: the model holds. So x86 can't reach the M4's 15 % by the
  elimination alone; what it can reach, the elimination's full saving, it now does.
- **`pair::sse2::solve5`**, used on x86-64 (elsewhere the lanes form as before), in SSE2
  intrinsics (x86-64's baseline; no SSE4.1, AVX or FMA assumed): each lane's pivot by comparing
  both lanes at once (`MAXPD` returns its first operand only where it is the greater, a NaN
  included, the scalar's `>`), the rows swapped by a masked exchange only if some lane's pivot
  is not on the diagonal, a row's step done unmasked where both lanes take it (`MOVMSKPD`),
  masked where one does, skipped where neither does; a column at a time (`column::<COL>`, a
  constant) so that the loops unroll. Each lane does the scalar's multiplies and subtracts in the
  scalar's order (no FMA, no reassociation); entries no later step reads (below a pivot in its
  column, and left of it when rows are swapped) are left as they were rather than computed.
  `unsafe` in the core crate for the first time, allowed for this module alone: the intrinsics
  touch registers only, no memory.
- **A test that keeps it exact**: `vcf::pair_tests::the_paired_solve5_is_the_scalars_to_the_bit`
  (not ignored), 200,000 random pairs of 5 by 5 systems with zeros of both signs, values below
  the pivot floor, ties in magnitude, repeated rows and zero columns, each lane of both paired
  forms against `solve::<5, true>`, to the bit.

**Evidence (2026-10-18; `ref` = 5870fdd, 0.2.5's engine, with `preset_render`'s DOUBLE switch
added; `before` = c3ada88, `cpu-release`; `after` = this branch; each built on its machine):**
- `preset_render`, every factory preset, ten voices, 4 s, at HI one thread, DOUBLE at 50 %, 3
  workers and 12: `before` and `after` the same to the bit as `ref` on the Mac, the Ryzen
  (Linux) and the i5 (Windows), in all four. (The first check on x86 of the pairs against 0.2.5's
  own samples.)
- `cargo test --release -p ca72` (`tests/pairs.rs` among them) on all three; clippy
  `-D warnings` for aarch64 and x86-64.
- `pair_timing` with the SSE2 form: Ryzen 31 ns a voice (-65 %, against the lanes form's -39 %),
  i5 30 to 32 (-60 to -63 %). A loop over the columns first gave 39 and 38: the unrolling is a
  quarter of the gain.
- `preset_cost`, ten voices, one thread, alternated: Ryzen (timing lock, two runs each) at HI
  `ref` 2279, `before` 2171 (-4.7 %), `after` 2094 (-8.1 %; -6 to -12 % a preset); with DOUBLE
  at 50 % 4589, 4340 (-5.4 %), 4190 (-8.7 %). i5 (pinned to one P-core, three runs each) at HI
  1990, 1888 (-5.1 %), 1843 (-7.4 %; -5 to -11 %); with DOUBLE 4026, 3800 (-5.6 %), 3661
  (-9.1 %). The Mac (aarch64 code unchanged; the machine busy with other sessions): `before`
  1628, `after` 1620; with DOUBLE 3197 and 3200, no change.
- **The i5's earlier spread**: unpinned, Windows ran `preset_cost` partly on the efficiency
  cores (0.2.5 at 2457 unpinned on 2026-10-18, 1990 pinned, runs up to 6 % apart unpinned and
  within 1 % pinned). Measure on that machine pinned (`ProcessorAffinity`), or say which.
- **Not done**: the lanes form unrolled a column at a time for AArch64 (the Mac's own
  elimination might gain; NEON intrinsics would be a second copy of this one). The rest of the
  x86 voice (the loop's evaluation, `fast::tanh`, the tables, the resamplers) is now the larger
  part of the gap; pairing the evaluation cost more than it saved on the Mac (above).

## R-RELEASE-0.2.6. 0.2.6

**Owner authorisation, 2026-10-17.** "Merge hi-pairs into cpu-release, run the full checks, push
and open a release PR against main": R-THREADS (the worker budget, without the band-limiting,
latency and ULTRA VCA work, held by the owner the same day), R-CPU-EXACT and R-HI-PAIRS; R-SPIN (the owner, 2026-10-18); and, the
owner asked on 2026-10-18 ("I want editor-perf to be included in this release. So test this in
any way that you need to."), R-EDITOR-CPU. The request authorises the pull request; merging, signing, tagging, upload and publication wait for
the owner's go.

LO's and HI's samples are the same to the bit as 0.2.5's (`preset_render`, every factory
preset); ULTRA's sound is unchanged (the OVERLOAD lamp's driver is not read by the audio; the
split voice gives the serial voice's samples); parameter identifiers, the preset format and the
installers are unchanged. Release notes are in `releases/0.2.6.md`. Before publication the
release commit must pass all three CI jobs; the macOS installer must be Developer ID signed,
notarised, stapled and accepted by Gatekeeper; every uploaded asset downloaded again and checked
against the manifest.
**Also in 0.2.6 (the owner, 2026-10-18):** #84 (R-RENDER-SETTINGS) and #85
(R-PRESET-READABILITY), "Include #84 (test tooling) and #85 (preset-browser readability) in 0.2.6";
#88 (R-AU-REMOTE-VISIBILITY), "Yes, I meant #88. Merge it into cpu-release for 0.2.6", with the
owner's hands-on checks; and the AU render test's fixed seed state. Each was merged into
`cpu-release`, not `main`. Publication: "okay, push to production then please!", and for `main`,
"Yes, push c0e325e to main yourself (fast-forward), then upload the Mac installer, verify every
asset, publish, and record it."

**Publication evidence, 2026-10-18.** [CA-72 0.2.6](https://github.com/idlefoundry/ca-72/releases/tag/v0.2.6)
is public and Latest (also to an anonymous request, which the update check makes). `main` was
fast-forwarded to PR #50's exact tested commit `c0e325e8c1f88b2f335cc9161a7b6d41d2fc95ed`, which
the annotated `v0.2.6` tag names; GitHub marked #50, #84 and #85 merged with it (#88 already was,
into `cpu-release`). Besides the owner's checks of #88, no host test of the installed 0.2.6 was
made before publication.
- [PR CI 39177243882](https://github.com/idlefoundry/ca-72/actions/runs/39177243882) and
  [tag CI 39179031325](https://github.com/idlefoundry/ca-72/actions/runs/39179031325) passed on
  Linux, Windows and macOS; the tag's drafted the release.
- The macOS installer was made from the PR run's unsigned universal bundles (artifact digest
  `sha256:8e3b4b31…`, checked), signed with Developer ID, notarised (submission
  `5eef65a2-5cdd-4a59-b908-77c31844094b`, Accepted), stapled and accepted by Gatekeeper, on the
  Mac from the agent's own session.
- `CA-72-0.2.6-macOS.pkg` sha256 `e9028155d57e209bc1ece50c0a25116228f05da1becfbc34faff5b6a1cfb5b38`,
  91,307,471 bytes. All nine assets, the Windows ZIP and its checksum among them, were downloaded
  again and checked against `SHA256SUMS.txt` and GitHub's digests; the published installer,
  downloaded once more with the quarantine flag set, passed `pkgutil`, `stapler validate` and
  Gatekeeper ("Notarized Developer ID").
- Issues #63 and #69 were closed with the release. The scripts and logs are in
  `~/lab/artifacts/ca-72/0.2.6/`.

## R-STILL-LONGER. The Windows window test waits out the hamster's pause

**Agent decisions, 2026-10-18** (not separately approved; asked for by the session releasing
0.2.6, whose PR #50 fails on Windows in the same test):
- **What failed.** From 0.2.5, `windows_window_tests::the_window_shown_in_places_is_the_window_shown_whole`
  failed on Windows at times: `main`'s run 39068098309 ("52 pixels shown otherwise than in the
  whole frame") on commit `212967e`, whose PR run 39066363897 and tag run 39068050615 passed on
  Windows; #50's runs with 58 and 35. 0.2.4 and every earlier run passed.
- **Why.** Not a place missed by the partial present (R58, R59): the test's idea of still. It
  called the window still once two frames 400 ms apart were alike. A new instance is at LO
  since R-LO-DEFAULT, so the test's QUALITY opening shows the hamster, not HI's lamp: once the
  synth is quiet his wheel slows for about 1.9 s (`opening::PACE`, its turn read in quarter
  degrees, so it creeps in steps), stands `DOZE_AFTER` (1 s), and he curls up asleep over
  `DOZE_S` (0.7 s). Two looks inside that slowing or that pause were alike; he moved after, and
  where Windows' repaint came after he had started to doze, it showed the pixels he had
  changed. HI's lamp only dims smoothly, with no pause; the test was written against it.
- **The change.** Still is now the same frame at every look, 400 ms apart, for `DOZE_AFTER`
  plus a second (2 s), within 20 s. `DOZE_AFTER` is `pub(crate)` so the test names the pause
  it waits out. The test stays at LO, a new instance's QUALITY, so it covers the hamster's run
  and doze shown in places. Nothing in the editor or the panel changes.

**Evidence (Windows 11, the lab's `DESKTOP-KHN6H8K`, in the desktop session by a scheduled
task; a debug build of `main` at `0d478a5`, 0.2.5's code, with a diagnostic, 2026-10-18).**
With the old rule the test alone passed every time, but called the window still about 1.52 s
after the synth went quiet (10 runs of 11 timed; once 3.1 s); in each of the 9 runs that
looked, 63 to 69 pixels had changed 4 s later, unrepainted, all inside a 10 by 6 pixel box at
the hamster's feet (x 831 to 841, y 66 to 72 of an 860-pixel-wide window; the opening's middle
at 836, 61). A full repaint then matched that later frame to the pixel each time, so the pixels
shown in places were right. Calling it still after 1.7 to 4.5 s of quiet: 0 pixels changed
after. With the new rule: 0 pixels apart, and 0 changed 4 s after, in 3 runs with the
diagnostic and 5 without. Then `cargo test --workspace --features
ca72-plugin/standalone-production` as CI runs it; and the editor's tests and clippy on the Mac
(see the PR).

## R-EDITOR-CPU. The editor's cost while sound plays

**Owner decisions, 2026-10-17.** CPU use is the users' first complaint. Told where it goes (a
three-agent council and measurements: LO about 1.5 % of a core a voice on the M4 Pro, HI 7 to
12 %, ULTRA 33 to 108 %), the owner decided: LO stays the default in every version; HI and ULTRA
stay bit-identical to 0.2.5, only changes that keep the sound the same; the band-limiting, latency
and ULTRA VCA work waits, the worker budget (R-THREADS) going out alone; and "spend time focusing
on the editor too. that's something that adds cost to all 3 performance modes." Asked to start
on the editor's changes that keep its look: "Okay, start."

**Measured first (the Mac, 2026-10-17, `the_window_played`, a window 3440 pixels across):** 11.3 %
of one core on the editor's thread with only the changed places turned (macOS since R59, Windows,
X11), 16.6 % with every frame turned whole (two runs each; 9.5 and 13.7 % earlier in the day, the
machine less busy). Sampled (`/usr/bin/sample`): about a third the panel's renderer (QUALITY's
shutter's paths, `ultra::Drawn`; pictures scaled bicubic; layers whose SVG changes rasterized
again), a seventh the strip (most of it the voices' drops, `Liquid::draw`), a tenth composing.
What the editor draws while sound plays mostly moves: the shutter follows the output thirty times
a second, the drops the voices, the layers' bodies change (a cache of rasterized bodies hit 9 times
in 183 and was taken out). The SVG drawn over a layer repeats (4 different in 467 draws), but drawn
once and put it differs by up to 2 in 255 in 8 % of the channels it covers, for 0.3 % of a core:
not done, the look being kept to the bit.

**Agent decisions, 2026-10-17** (not separately approved):
- **Unseen, drawn every 250 ms and not shown.** baseview's frame timer runs whether or not the
  window can be seen, so the shutter and the drops were drawn thirty times a second behind other
  windows, minimised, on another Space or with the host hidden. `window::seen` asks macOS for the
  occlusion state of the window holding the view (`occlusionState`, through `objc`, the version
  baseview takes) and Windows whether its top window is minimised (not `IsWindowVisible`: in a
  session without a desktop on the screen, as an agent's over SSH on the lab PC, every window
  answers that it is not visible though it is drawn and shown, and the editor showed nothing:
  five of the real-window tests failed so); X11 says nothing reliable without a compositor and
  is left as it was. A window covered by others is drawn as before on Windows. Unseen, the keys, gestures
  and the host are attended to every frame as before and the scene drawn every 250 ms (a resize,
  the drawer, MIDI Learn kept up), nothing shown; seen again, the frame is shown whole at once.
  Audio Units are excepted by R-AU-REMOTE-VISIBILITY below: their native window need not be
  the surface the host presents.
- **The drops' columns' dx squared once for all their rows,** the same sums in the same order:
  200 frames at two scales hashed whole, the same before and after; 0.71 to 0.66 ms a frame for
  ten voices' drops at 3440 pixels.
- **Thirty frames a second stay** (the owner, 2026-10-17, shown a trial at twenty: 11.45 to
  10.1 % of a core, 834 to 662 frames presented in the measure's 24 s): "I don't approve
  dropping the FPS".
- **`CA72_NO_FRAMES`** keeps `the_window_played` to its times: its frames are 2 GB in memory and
  on disk, and on a nearly full disk the run failed before printing them.

**Evidence (the Mac, 2026-10-17):** the panel's and plug-in's tests in debug; clippy with
`-D warnings` on macOS, `cargo check` of the plug-in for `x86_64-pc-windows-msvc`. Not yet: the
unseen window in hosts (Live, Logic, Reaper; Activity Monitor with the window covered, minimised
and the host in the background), and on Windows.

## R-SPIN. Workers spin 50 us, not 200, before they park

**Owner decision, 2026-10-18.** The measurement pass (`docs/work/cpu-measurements-2026-10.md`
on the measuring session's branch, item 4) found `pool::SPIN` at 50 us took 12 to 28 points of
a core off the process for one to ten HI voices on the Mac (one voice 63 to 68 % down to 43 to
48 %), late blocks unchanged. Asked whether to test it on the other systems and put it in 0.2.6:
"Yes, do the tests and add it to 0.2.6".

**Agent decision, 2026-10-18:** `SPIN` 50 us. A block's runs follow each other within
microseconds (its second `CHUNK`), so 50 us still catches them; the next block is 2.7 ms away at
128 frames and 48 kHz, so a worker parks and wakes for every block either way, and the longer
spin was mostly spent after a block's last voice.

**Evidence (2026-10-18; `poly_bench` paced, Feedback Funk, four workers made audio threads, 48
kHz, 20 s, the 200 and 50 us builds alternated in two rounds of opposite order, built from this
branch on each machine; process CPU from the shell's `time` on Linux, the process's own
processor time on Windows):**
- **Linux, Ryzen 7 7800X3D** (real-time granted; the timing lock): process CPU the same within
  half a point at one, four and ten voices, 128 and 256 frames (9.5, 14 and 27 %); no late block
  in any run (0 of 7,500 and 3,750); the worst blocks alike.
- **Windows, i5-13600K** (unpinned, as a host runs it): process CPU within about two points
  either way (20 to 22 %, 26 to 29 %, 30 to 32 %); late blocks only at ten voices, as many at 200
  us as at 50 (128 frames: 4 and 3, then 3 and 1; 256 frames: 1 and 9, then 6 and 0); the worst
  blocks alike.
- So the saving is the Mac's: parking a thread costs little on Linux and Windows, and the spin
  with it. On none of the three is the 50 us spin later.

## R-RENDER-SETTINGS. ULTRA is repeatable; a comparison's settings are checked

**Agent finding, 2026-10-18** (placeholder id; not yet reviewed by the owner). R-CPU-EXACT's
correction (branch `cpu-release`, 5870fdd) says ULTRA's renders are not repeatable over time on
the Mac in 0.2.5 itself. They are. The comparisons that differed rendered ten voices against
references of four.
- **ULTRA is repeatable.** The binary that made the ULTRA references at 19:12 on 2026-10-17
  (main with `CA72_ULTRA`, saved at 19:10) rendered every factory preset the same to the bit
  against them at 10:17 on 2026-10-18 (`CA72_ULTRA=1 CA72_VOICES=4`, workers 0). A new build of
  main did the same in four runs at load averages from 4.5 to 22, and once on the efficiency cores
  alone (`taskpolicy -c background`). So did `cpu-release` at 5870fdd, whose ULTRA plays two
  voices split over helper threads on the M4 Pro (R-THREADS), and the saved `rel-preset_render`
  (19:56). Nothing in ULTRA's serial engine depends on time or another thread: there is no clock,
  no pool or deadline without workers, the factory voice and shared tables are made once per
  process from the rate alone, and `ulp::FastScope` is off at ULTRA on the one thread.
- **What differed.** Both failing comparisons (19:47 and 20:23) ran
  `env $e CA72_RENDER_REF=… preset_render` with `e="CA72_ULTRA=1 CA72_VOICES=4"` under zsh, which
  does not split an unquoted parameter. `env` got one assignment, `CA72_ULTRA="1 CA72_VOICES=4"`:
  ULTRA on (any value counted), `CA72_VOICES` unset, so 10 voices. HI's `e` was empty and LO's
  a single assignment, so they matched. The first differing sample is the first one (frame 0,
  left): -2.85e-6 in the reference against -7.12e-6, the idle voices' hiss, 10/4 = 2.5 times as
  much. Felt Bass's line (2.2 dB) was reproduced exactly both by that command and by
  `CA72_VOICES=10`. The 19:47 run's "three presets" were its `tail -3`; every preset differed.
- **Change (tests only).** `preset_render` writes the run's settings (QUALITY, voices, seconds,
  presets, 0.1's translation) to `settings.txt` beside `CA72_RENDER_OUT`'s renders, and stops if a
  reference's are not the run's. A presets file is named there by a digest of its contents, not
  its path. The file lists the renders the run wrote. It is removed when a run begins and written
  once every render is, so an output left half made, or holding an earlier run's renders, lists
  none of them; a render compared must be on the reference's list. A reference without the file
  is compared with a note. A switch
  (`CA72_ULTRA`, `CA72_POTATO`, `CA72_FROM_0_1`) takes `1` or nothing; any other value stops the run,
  and so does setting both qualities. The shared `env` stops on a value that does not parse
  (it fell back to the default). The engine is unchanged: HI, LO and ULTRA render every preset the
  same to the bit as main's.
- **Also learned.** Cargo names a test's binary by its metadata, not its code: main's and
  `cpu-release`'s `preset_render` are both `preset_render-d3ac7cd2e0e0b1e9`, so a binary run later
  by its target path can be another version's. Copy a binary aside to compare against it later.
- **Merged into `cpu-release` for 0.2.6 (2026-10-18):** R-CPU-EXACT's correction is withdrawn
  there; its ULTRA comparison minutes apart is as good evidence as HI's. `CA72_DOUBLE` (R-HI-PAIRS)
  reads through the same checked `env`, and `settings.txt` names DOUBLE where it is not 0.

## R-PRESET-READABILITY. The preset drawer's labels and selected sound are easier to read

**Owner decision, 2026-10-18.** Improve the preset browser's text contrast and small action
labels (#69). After reviewing matched renders at native size and an enlarged list crop,
the owner approved the final visibility treatment and requested a separate pull request.

**Implementation:** action names sit on larger caps in clear print, with their existing
centres and separate hit regions. Quieter glass and unlit dots leave more contrast for
amber lettering. The current preset alone uses gold and a filled triangle; hover uses
brighter amber and an outline arrow. Selected tags and tabs have distinct markers, while
unavailable actions are shaded. Unselected text remains darker than selected, hovered or
focused controls, including the quieter labels and placeholders within those controls.
Moving and clearing selection restores the old row, including its colour and marker.

**Evidence (the Mac, Rust 1.97.1, 2026-10-18):** 52 panel library tests and 2 approved-image
tests pass, as do 7 affected editor drawer tests. Eight panel image tests and one optional
editor image test are ignored in those suites. The separate 32-image matrix renders both
looks and lettering settings at widths 860, 1035, 1380 and 3108; browsing, editing, empty
and MIDI states were checked at small widths. The selected/hovered/ordinary and active
placeholder/MIDI tests retain separate assertions; redraw tests compare partial updates
with fresh whole frames. The preset list, FIND, PRESET and existing action-cap regions
match the approved review render to the pixel after excluding the separate Init action.
Formatting, whitespace and workspace/all-target clippy pass with default and all features.
Fresh independent source review found no issues.

This change concerns the panel renderer and its regression coverage. Init and the Settings
version display remain separate. Native Logic/GarageBand rendering validation is still
under investigation separately; offscreen checks do not establish release acceptance in
those hosts. No installed plug-in, host window or audio device was used for this work.

## R-AU-REMOTE-VISIBILITY. A remote AU view keeps presenting

**Decision, 2026-10-18.** Keep R-EDITOR-CPU's native-window occlusion optimization for
ordinary CLAP, VST3 and both standalone adapters. Except AUv2: a remote view may be visible
while its local NSWindow's Visible occlusion bit is clear. The original gate suppresses
all presentations in that case, including the first image on reopening. The local window's
visibility is not authoritative for the host's remotely presented surface.

**Implementation.** The AUv2 wrapper advertises the private, versioned host capability
`com.idlefoundry.native-window-occlusion-unreliable/1`. `IHost` defaults to no marker;
only `WrapAsAUV2` opts out, per instance. The CLAP backend queries presence once during
initialization and caches it in that instance's GUI context. `PluginApi` remains `Clap`.
The editor bypasses macOS's native occlusion gate only for that capability. Missing markers,
unknown versions, VST3 and standalone retain the original policy. Rust neither dereferences
nor retains the static marker pointer, and the frame timer makes no host extension calls.
The AU wrapper and embedded CLAP must be rebuilt together. No hidden-window CPU saving is
claimed for AU; changed-frame suppression and the other performance work remain.

**Evidence (GarageBand 10.4.14, macOS 27.0, Apple silicon, 2026-10-18).** A distinct
diagnostic build based on `2c0a584` ran in a fresh AUHostingService, separately from the
production instrument, with transport stopped. Its visibly rendered proxy had a backing
`NSKVONotifying_NSServiceViewControllerWindow` with raw occlusion state 8192, Visible bit
clear and `isVisible=true`. The logged rectangle alone did not prove that it was wholly
off-screen; the unreliable visibility signal is the relevant observation.

- The diagnostic's original-policy switch, on reopen: a blank visible editor, 929 frame
  callbacks, 55 draws and zero presentation calls across a contiguous 13.996-second sample.
- With the capability policy: the full panel returned on the next frame, about 14 ms after
  the test switch. Two more editor reopenings each made an initial presentation call.
- Editor close: no callbacks, draws or presentation calls in a verified 3.173-second interval.
- Native host preset loading exercised a visible redraw. The exact resulting parameter
  value and direct knob-drag interaction were not independently asserted.
- The host's Hide command was issued; the retained view continued drawing an unchanged
  scene. Actual disappearance from screen and restore still need hands-on confirmation.

These are diagnostic call counts, not compositor acknowledgements or production FPS/CPU
measurements. The full/blank panel outcomes were also observed in native screenshots.
Diagnostic logging, mode switches, identities and test-data paths are not in product code.
The experiment predates the later worker-spin change; integration preserves that work.

**Regression coverage.** The C++ proxy test checks local/default dispatch, the remote marker,
unknown versions, instance isolation and the standard GUI extension. The Rust tests exercise
actual CLAP construction, initialization and GUI context propagation, including no early
query and one query per instance. CI runs the vendored crate's tests explicitly because
workspace tests do not execute dependency unit tests. A separate extracted production-frame
harness passed the remote/hidden/restore cases with the correction and failed the original
remote case; that mock is control-flow evidence, not native-host validation.

**Remaining acceptance.** Direct TUNE drag and independently observed GarageBand Hide/restore;
Logic (not installed during the experiment); ordinary direct CLAP/VST3 and standalone
visible/hidden/reopen smoke tests; normal release validators and production CPU measurements.
Do not treat approval to include the correction as evidence that these checks passed.

**Owner's acceptance, 2026-10-18.** Told these checks remained, the owner did them by hand:
"i did hands on checks. we're good".

## R-INIT. Start a patch from the instrument's fresh-instance sound

**Owner decision, 2026-10-18.** Implement issue 55 as a clearly visible INIT action in the
presets' menu. Recall the existing fresh-instance sound, not zeroed knobs, and preserve player
and computer preferences. The owner will try a separate test build before production release.

**Agent decisions, 2026-10-18** (not separately approved):
- INIT is a dedicated key beside PRESET, available without choosing a preset first. It uses
  `Ca72Params`' parameter defaults directly, through normal host parameter gestures; it does
  not create or replace a library entry, including a user's preset named Init. The current
  preset is cleared, as on a fresh instance, and SAVE AS can keep the new patch.
- The starting patch on main `5e7abe9` is oscillator 1 alone, sawtooth at 8', its mixer at 8;
  oscillators 2 and 3 off, also sawtooth at 8', FREQUENCY centred, mixer levels 8; noise off,
  white, level 8; external input off, level 5. TUNE and GLIDE are 0, modulation mix and wheel
  0, oscillator modulation off, oscillator 3 keyboard control on. Filter mode LO, modulation
  off, first keyboard-control switch on and second off; CUTOFF centred, EMPHASIS 0, CONTOUR
  AMOUNT 5. The filter's ATTACK/DECAY/SUSTAIN are 0/4/5; loudness 0/4/8. MAIN OUTPUT is on at
  10; A-440 off; GLIDE switch off; DECAY switch on; NOTE PRIORITY LOW; RETRIGGER off.
  POLY and UNISON are off, VOICES 4, ENTROPY/WIDTH/INNER/DETUNE/FEEDBACK/DRIVE/LEVEL 0,
  placement EVEN, DOUBLE and LOCK off, AUTO GAIN on.
- The existing sound-preset exclusions stay: PITCH wheel, bypass, QUALITY and MIDI BEND RANGE.
  MIDI assignments, the instance's random seed, editor dimensions and visibility, look,
  lettering, frequency-dial law, keyboard and audio-device preferences are not reset.
  The engine's running oscillators and envelopes are not restarted; audio equivalence is
  checked with equivalent seed, phase and initial engine state, as for a new instance.
- No factory preset, parameter ID, preset format or saved-project migration changes.

**Evidence (the Mac, Rust 1.97.1, 2026-10-18):**
`init_restores_every_sound_default_and_preserves_preferences` changes every hosted parameter,
including one only 0.0000002 from its normalised default, then checks all sound defaults and
the exclusions; a second INIT adds no gestures and no file is written.
`init_saved_and_reloaded_matches_a_fresh_instances_state_and_audio` starts from Brass Choir,
uses the drawer action, saves and reloads the result, checks a user's preset named Init is
untouched, and compares three notes' stereo renders at LO, HI and ULTRA with seed 12345:
15,360 stereo samples at each quality match to the bit and are finite and non-silent.
Both tests and plug-in library/test clippy pass. No host or audio device was opened for them.

**Integration, 2026-10-18.** Rebased onto main `1a1dd6c` (0.2.6), retaining #85's
caption sizes, selection colours, hit-region and redraw tests. The INIT cap uses those same
caption measurements and remains separate from the existing keys. Its hover is checked in
partial redraws and shown in the optional editing-state review render.

The audio reference now comes from a separate fresh parameter instance and engine, with the
same quality and seed, before the other instance loads a factory sound and INIT. After
saving both INIT and a patch edited from it, a new browser reads the files back; their states
and audio must match. The test also compares every factory library entry afterwards. The
parameter test explicitly keeps the editor's transient visibility flags as well as its
persisted preferences. Engine history is still not restarted by INIT.

## R-SETTINGS-VERSION. Settings names the installed version

**Owner authorisation, 2026-10-18.** Implement only issue 75's installed-version display in
Settings. The changelog and What's New modal are outside this work.

**Integration decision, 2026-10-18.** Keep `33cec21`'s Settings heading, derived from the
existing `update::VERSION` (`CARGO_PKG_VERSION`), matching the plug-in's reported version.
It remains visible independently of update status, including offline failure; opening the
menu does not start an update check. The drawer still distinguishes a newer online version
from the installed version.

Compared with fidelity's `414eaa0` (R-VERSION), this keeps the change within issue 75's
version-display scope and avoids its unrelated worker-seat test change. That alternative
adds a build script, commit suffixes in the drawer and a VERSION menu group. Local builds
with the same package version remain indistinguishable under the chosen approach. Whether
to add commit identity separately remains for the owner; neither a new version policy nor
approval of that alternative is inferred here. No version number is changed.

**Integration evidence (Linux, Rust 1.97.1, 2026-10-18):**
`cargo test -p ca72-panel -p ca72-plugin` passes, including both approved panel images,
the Init parameter/state/audio checks and the Settings tests. Clippy for both crates, all
targets, passes with default and all features and `-D warnings`; formatting and whitespace
checks pass. A separate read-only source review found no product bugs or unrelated changes.
All checks use this worktree's target, shared-data and worker-seat folders. No native host,
installed plug-in or audio device was opened; host acceptance of the new action remains
untested here.

## R-ENTROPY-FLOOR. ENTROPY reaches steady oscillator pitch at 0 %

**Owner decision, 2026-10-18 (#68).** "we had decided to make our current 0% to be 5%,
and then scale our current entropy to 5->100%. This means all current patches will have
at least 5% entropy, which I think is good." The new reading is `5 + 0.95 × old`, linear
from the old minimum at 5 % to the old maximum at 100 %. Below 5 %, the oscillators'
random tuning offsets, drift and waver fall linearly to none at 0 %. New instances begin
at 5 %, preserving the previous default sound. LOCK retains its existing pitch suppression
and gain trim; ENTROPY's OFF action now reaches steady pitch, and ON recalls its amount.

**Compatibility.** Old presets and saved projects move to the new reading on load. The
owner accepted the exception for a DAW's automation lanes: they hold only normalised knob
positions, and old automation plays the new mapping. A lane at 0 now means steady pitch;
an old lane is not shifted to 5 %.

**Investigation (Linux, 2026-10-18).** At 0.2.6's minimum with LOCK off, each oscillator
has three independent character contributions: a fixed standard-normal tuning offset
(nominal SD 3 cents), mean-reverting drift (SD 2.25 cents, time constant 4 s, stepped
every 64 samples), and one-pole-filtered noise (SD 0.3 cent, cutoff 30 Hz). The fixed
offset differs between oscillators and voices but does not move during a held note.
`the_legacy_pitch_floor_is_tuning_drift_and_waver`, before changing the mapping, sampled
64 seeds, all three oscillators, at 48 kHz for 2 s, discarding the first second: measured
SDs 2.9664092073, 2.3263836871 and 0.3020300236 cents respectively. LOCK sets all three
pitch contributions to zero, at every ENTROPY amount. At the old minimum there are no
random cutoff or panel tolerances; these begin above the new 5 %.

The circuit has no other autonomous random oscillator-pitch source. Its noise generator
can deliberately modulate pitch through OSCILLATOR MODULATION, MODULATION MIX and the
MODULATION wheel, including with LOCK. This is the instrument's musical modulation,
and remains available at 0 %. Oscillator 3 modulation, pitch bend, GLIDE, fixed circuit
calibration, DOUBLE's deliberate detuning and random initial voice phases also remain;
none is random movement added to a held oscillator's pitch. Mixer hiss and noise at
the audio mixer do not modulate oscillator pitch. Steady-pitch acceptance has these
intentional modulation controls at rest.

**Agent decisions, 2026-10-18** (not separately approved): the first version with this
mapping is provisionally 0.2.7. Preset format 1 gains a writer `version`, as nih-plug's
project state already has; absent versions denote the old mapping. Factory and retired
files store translated amounts and the new version. Reading migrates in memory, without
rewriting the user's library; subsequent writes include the current writer version.

**Numerical limit.** Project parameters are saved as `f32`. Compressing the old 0..100
interval into 5..100 cannot retain every representable old value without another stored
representation. The translation is mathematically equivalent; its forward rounding can
change the recovered old reading by at most about 0.000004016 percentage point (the
largest half-ULP of the new reading, divided by 0.95), or about 0.00000008031 in the
character's depth. For example, an old saved 51.00910186767578125 % becomes
53.458648681640625 %, which recovers 51.00910387541118496 %: a difference of
0.00000200773540371 percentage point. `fractional_entropy_state_migration_has_a_bounded_rounding_error`
checks representative fractional states against a 0.0000041 bound and asserts that this
example is not exact. Bit identity for every fractional host state is not claimed.

**Render evidence (Linux x86, Rust 1.97.1, 2026-10-18).** The unmodified `preset_render`
harness was built from 0.2.6's `1a1dd6c` before the mapping changed, its binary copied
aside, and compared with the new build. Both use 48 kHz, seed 1, four voices, 256-frame
blocks and the same chord sequence. All 19 factory presets at 0.5 s in LO, HI and ULTRA
are identical to the bit: 57 comparisons, 23,808 stereo frames each. Felt Bass (old 0 %),
Brass Choir (40 %) and Swarm Riser (100 %) also match over 4 s in all three qualities:
nine comparisons, 192,000 stereo frames each. This harness supplies the files' plain
values as `f64`; it does not establish bit identity after a host's `f32` parameter
quantisation. The files and SHA-256 manifests are in this worktree's
`target/entropy-evidence`, alongside the commands' logs. The permanent regression test
`translated_entropy_renders_identically_to_the_legacy_mix` compares the actual engine
with the old pitch/cutoff mix and panel tolerances, independently calculated at old
0, 20, 40 and 100 %, LOCK off and on, HI and LO, with the same initial state.

**Steady pitch and integration.** `zero_entropy_has_steady_pitch_in_every_oscillator`
measures all three HI oscillators' reset-to-reset frequencies across eight quarter-second
windows, after half a second's settling, at two seeds: less than 0.01 cent movement.
`zero_entropy_has_steady_audio_pitch_in_every_quality` measures each oscillator alone
from its audio in LO, HI and ULTRA, with intentional modulation at rest: voiced-frame
medians across four quarter-second windows move at most 0.004038, 0.015446 and 0.004610
cent respectively (a 0.05-cent gate; the audio estimate includes waveform and sampling
variation). The offsets test asserts zero detune at 0 % and under LOCK, and zero cutoff
movement throughout 0..5 %. State tests include the real CLAP save/load interface,
resaving without a second migration and host automation using the new position. Readout
and OFF/recall tests cover the new default, 2 % and 43 %.

Independent source review found two writer-version edge cases, both corrected: the
historical-preset converter now emits a complete file with its writer version, and the
version gate understands semver build metadata. Tests cover reading the converter's
output without migrating twice and `0.2.7+build` retaining the new scale.

The full workspace run passed 503 tests, 0 failed, 56 ignored with ngspice 47 available,
before the final converter/version edge-case fixes and additional regression tests.
After those fixes, `cargo test -p ca72-plugin --no-fail-fast` passed 302 tests,
0 failed, 31 ignored; the converter example's regression passed separately. Formatting,
`cargo clippy -p ca72-plugin --all-targets -- -D warnings` and the same command with
`--all-features` passed. The reviewer rechecked both fixes and found no further defects. No native DAW, macOS
or Windows build, installed plug-in, or audio device was used for this job. The numerical
rounding limit, preserving deliberate panel modulation at 0 %, and 0.2.7 as the first
writer of the new scale are recorded here for the owner's confirmation at review.


## R-CLAP-ENVELOPES. Zero-velocity typed notes are attacks

**Agent finding, 2026-10-18** (issue 57; not separately reviewed by the owner).
`event_of` treated every zero-velocity `NoteOn` as a release. CLAP expressly permits a
native note-on at velocity 0. nih-plug's MIDI decoder already turns MIDI 1's zero-velocity
note-on bytes into `NoteOff`; VST3 preserves its native note-on velocity. Accept typed
`NoteOn` as an attack, leaving MIDI 1 releases in the decoder. No panel setting, saved id
or contour model changes. The existing keyboard-source test now constructs its MIDI
zero-velocity release through that decoder.

**Evidence (Linux, 2026-10-18):** the native CLAP regression failed before the correction,
held RMS 0. The headless CLAP matrix covers 126 mode/priority/retrigger/buffer combinations
in both note dialects, with direct filter and loudness contour checks in 54 configurations
across LO, HI and ULTRA. REAPER 7.79 rendered the isolated MIDI item in CLAP and VST3 in
all 18 mode/priority/retrigger combinations: 36 renders, 504 sounding windows, all above
RMS 0.102, final residual RMS 0.0046 to 0.0065. Bitwig 5.2.7 loaded the renamed CLAP, but
MIDI playback and rendering there remain unverified. The disposable profile's graphics
and file-chooser failures and Demo Mode's export restriction are recorded in
`docs/work/linux-clap-envelopes-2026-10.md`, with the reproduction, bounds and draft reply.

This establishes a missing-note defect, not the cause of the reporter's intermittent
failure. Their host, patch and MIDI are still unknown; both linked forum pages returned
HTTP 403. Do not treat ordinary mono legato, the circuit's approximately 12 ms trigger
re-arming interval, or issue 15 as an explanation for separated-note failures. CC64
currently has no built-in pedal function; changing that is a separate feature decision.


## R-TWO-SCREENS. The editor keeps its size on a screen of another scale (issue #48)

**Agent decisions, 2026-10-18** (not separately approved; the issue was given to fix):
- **The report.** Issue #48: on a MacBook Pro with an ultrawide monitor, the editor drew fine on
  the laptop's screen, but on the ultrawide at half its size in the top left corner of its
  window, the rest empty; in Cubase, Ableton Live and Kushview Element.
- **Why.** Not the editor's size asked of the host in the wrong units: the window keeps its size,
  the frame shrinks. The laptop's screen is Retina (2 pixels a point) and the monitor 1x. As the
  window moves between them, AppKit tells baseview's view (its `viewDidChangeBackingProperties:`
  handler sends `Resized` at the new scale, and the editor draws its frame for it, 1382 pixels
  wide for a window of 1382 points) but leaves the view's layer at the scale it was made with, 2.
  softbuffer's layer follows the view's layer's `contentsScale` and lays its tiles out at it
  (R59), so the 1x frame filled 691 points. It never mended itself: a later frame (an octave
  changed) still showed at half size, since nothing changed the layer's scale; only closing the
  editor did. A trace of the standalone (the window's scale, the view's layer's, softbuffer's
  tiles) showed, on the move to the 1x monitor, the window at 1, `Resized` at 1, the frame 1382
  by 789, and both layers still at 2. Calling NSView's own `viewDidChangeBackingProperties:`
  first (baseview's handler replaces it without calling it) changed nothing. The window opened
  straight on the 1x monitor showed the same.
- **The change.** baseview's change 8 (`third_party/baseview/PATCHES.md`): the handler sets the
  view's layer's `contentsScale` to the window's `backingScaleFactor`, without animation, before
  it sends `Resized`, and whether or not the size in pixels changed. softbuffer's observer
  carries it to its layer, and the editor's next frame (shown whole after a resize) is tiled at
  the new scale. Nothing in the editor, nor softbuffer, changes. And `becomeFirstResponder` no
  longer messages a nil window (a debug build stopped there as the new test opened the window).
- **Test.** `crates/ca72-plugin/tests/backing_scale.rs` (on the main thread, `harness = false`,
  macOS): baseview's view opened in a window, its layer left at the other scale as AppKit leaves
  it, the view told; the view's layer and softbuffer's must take the window's scale, and the
  frame shown then must fill the view, its tiles at that scale. Without the change it fails
  ("the view's layer left at another scale"). A test cannot move a window to another screen, so
  it reproduces what AppKit leaves behind rather than the move itself.

**Evidence (the Mac, macOS 27, M4 Pro; its Retina screen at 1728 by 1117 points and a 4K
monitor, switched for the test from "looks like 1920 by 1080" HiDPI to 1920 by 1080 at 1x and
restored after, 2026-10-18).** The standalone of `main` at `1a1dd6c`: whole on the Retina
screen and on the 4K at HiDPI; at half size on the 4K at 1x, moved there or opened there, and
still after a click that changed the panel. The released CA-72 in Live 12.2.7 (VST3): the same
picture as the issue's. With the change, the standalone and a renamed VST3 build ("CA-72 I48",
removed after) in Live: whole on the 1x monitor, and on the Retina screen before and after.
Screenshots are kept with the night's records, not in the repository.

**Seen as the change was tried (changed since, below).** The standalone opened on the 1x monitor (the screen in use)
drew its editor wider than its window, cut off on the right, before and after the change: its
window was sized for the Retina screen (1382 points, 80 % of 1728) and the editor, measuring the
screen again as it opened (R19's `opening`), took the monitor's (about 1472, read from the
pictures). The editor's opening size comes from AppKit's "main" screen, the one in use, which may
differ between the host's question and the window's opening. Not the issue's picture; not tried in
a host.

**The window and its editor the same size wherever it opens. Agent decisions, 2026-10-18,
later the same night** (not separately approved; the fix was asked for):
- **Why, traced.** The standalone asks the editor's size before baseview makes its window
  (nih-plug's wrapper), and the editor measured AppKit's main screen: with no window of the
  application's yet, the Mac's main display, its Retina screen (1728 by 1084 points usable, so
  1382 wide). macOS put the new window on the screen in use, the 1x monitor, and made it the key
  window; the editor, opening (R19's `opening`), measured the main screen again, now the
  monitor's, the key window's (1920 by 1050: 1471 wide), and drew a frame of 1471 by 840 points
  in a window of 1382 by 789. A trace of each measurement and of the size opened showed the two,
  in that order.
- **The change.** The editor opens at the size the host was last told, the size of its window:
  `opening` no longer measures the screen. The width is measured at the host's first question
  (or as the editor opens, if it asked none) and kept until the editor closes, as R19 kept it
  while open. A scale the host gives after asking (Windows, Linux, where the screen is measured
  in its pixels) has it measured again at the next question. The SIZE menu's FIT TO SCREEN still
  measures the screen in use.
- **What it costs.** An editor opening on another screen than the one measured opens fitted to
  the one measured: the standalone on the 1x monitor opens 1382 wide, not 1471. To fit it to the
  screen it lands on, the window would be resized once open (a frame at one size, then the
  next); not done.
- **In a host.** Live 12.2.7 (VST3) asks the size first and opens the editor in its plug-in
  window, already made that size; it puts that window on the screen of its own main window (a
  window reopened after being moved to the monitor came back on the Retina screen). The trace
  there showed one measurement, the Retina screen's, and the editor opened at it in a window that
  size; moved to the monitor, it stayed whole. Live's main window could not be moved to the
  monitor (neither by the accessibility interface nor by dragging), so an editor opened on the
  monitor in Live was not seen.
- **Test.** `the_editor_opens_at_the_size_the_host_was_told`: an editor measuring a screen of the
  test's is told its size on a Retina screen's measure, and opens as the screen in use becomes the
  monitor's; it must open at the size told, and measure the monitor once it has closed. Without
  the change it fails (opened 1478 by 844, told 1382 by 789).
  `the_screen_is_measured_where_the_host_has_not_asked_in_this_scale` covers a host that asks
  nothing first and one that gives its scale after asking.

**Evidence (the Mac, the 4K monitor at 1x again and restored after, 2026-10-18).** The
standalone opened on the monitor (from a terminal there, the window placed by macOS): with
`opening` measuring again as before, its window 1382 points wide and the editor 1471, cut off on
the right and at the bottom; with the change, whole, and whole on the Retina screen. In Live, a
renamed VST3 build ("CA-72 OPN", removed after): whole on the Retina screen and moved to the
monitor.


## R-WIN-STANDALONE. The Windows standalone's window is its panel's, at any display scale

**The report (issue 64).** A Windows 11 user of the standalone: "opens up with a large
background window with extra black space in it", the VST3 fine (KVR forum, 2026-10-16).

**Agent decisions, 2026-10-18** (not separately approved; the owner is asked to confirm the
second):
- **The cause, reproduced.** With 0.2.6's released `CA-72.exe` and with `main` (`1a1dd6c`) on
  Windows 11 at 125 %, 150 % and 175 % (the lab's 1920 by 1080 display; Windows offers no more
  there), the window's client was the panel's size times the display's scale, and the panel
  itself was drawn at 1: 1429 by 816 pixels around a panel of 1143 by 653 at 125 %, 1412 by 806
  around 941 by 537 at 150 %, 1505 by 859 around 860 by 491 at 175 %, black to its right and
  below. nih-plug's standalone opened its window at the system's scale (baseview's
  `SystemScaleFactor`), which baseview measures only after the window is open, having made the
  process aware of each monitor's scale only then; the editor, told no scale (as in Ableton
  Live, R18), measured the screen first and drew at 1. At 100 % the two agreed, which is why it
  went unseen. And at any scale the window's frame could be dragged or the window maximised,
  which the editor does not follow: black beyond the panel, or the panel cut off (the grip with
  it). A window open while the display's scale changes was scaled by Windows, the panel left as
  it was.
- **The window and the panel share one scale.** On Windows the standalone makes the process
  aware of each monitor's scale before anything is measured, takes the primary monitor's scale
  (where the window opens), tells the editor that scale before asking its size, and opens its
  window at the same scale (nih-plug's `production::app_window`, `third_party/nih-plug/PATCHES.md`).
  The panel opens at the same share of the screen as before (80 %, R6), sharp, and the SIZE
  menu's sizes (R-SIZE) are logical pixels as in a host that gives a scale. The saved width is
  logical, so a session restored at another scale opens at the same share of the panel's size.
- **No frame to drag and no maximise box** (for the owner to confirm). The standalone's
  window is sized by the panel alone, with the grip and WINDOW › SIZE, as on macOS, whose
  window was never resizable: baseview's change 9 (`third_party/baseview/PATCHES.md`). The
  alternative, the panel following a dragged frame, needs the editor to take a size from
  outside (with its proportions, a maximised window would still show bars), which the hosts'
  edge resize (R-SIZE, "Not done") would need too; left for that change.
- **A resize from outside is undone** (Windows): a window scaled by Windows as its display's
  scale changes, or maximised by another program, is fitted to the panel again at its next
  frame (`app_window::refits`). The panel keeps the scale it opened at until the window is
  opened again, as an editor in a host does (R19). Not on Linux or macOS, where a window
  manager may hold a window at a size of its own.
- **One baseview.** nih-plug's standalone took upstream's baseview at `579130e` from git, a
  second copy beside the editor's patched one (`third_party/baseview`); it now opens its window
  with the patched copy, without the `opengl` feature neither window uses (the release's
  `CA-72.exe` linked `OPENGL32.dll` for nothing; this build does not). Its X11 backend is the same code; the macOS
  backend's differences are upstream's later fixes and the CA-72's changes 4 and 7, which the
  editor already runs on.

**Evidence (Windows 11 `DESKTOP-KHN6H8K`, 1920 by 1080, in the desktop session; 2026-10-18).**
The display's scale was changed live (`SystemParametersInfo`'s logical DPI override) to 125,
150 and 175 % and put back to 100 %, its registry key as before. The fixed build
(`ca72-standalone`, `--release`, `--no-devices`, its own `IDLE_FOUNDRY_SHARED`): the window's
client equal to the panel's window at each scale (1446 by 826 at 100 %, 1429 by 816 at 125 %,
1412 by 806 at 150 %, 1505 by 859 at 175 %); PANEL off and on (1505 by 504 and back); SIZE
75 % at 125 % (1294 by 739), closed and opened again at that size; the frame's corner dragged
by the pointer, a double click on the title and Win+Up: no change; 125 % to 150 % and to 100 %
while open: the window kept to the panel; minimised and restored. The new test
`the_standalone_s_window_is_its_editor_s_at_every_scale` (`crates/ca72-plugin/src/editor.rs`,
Windows, `standalone-production`) opens the window as the app does at 1, 1.25, 1.5, 1.75 and 2
whatever the display's scale, clicks PANEL off and on, and scales the window from outside:
the window's client is the panel's window, pixel for pixel, each time, and its style has no
thick frame and no maximise box. It fails without each part: the scale not told the editor
("at 1.25, opened: the editor's window", 860 by 491 for 1075 by 614), the old styles ("at 1: a
frame to drag") and no refit ("at 1, scaled by Windows", 1016 by 550 for 860 by 491). All 14
`windows_window_tests` pass (MSVC, 1.97.1). The VST3 of this tree in REAPER 7.82: presets'
drawer opened, shut and reopened, a row chosen, the arrows, PANEL off and the drawer over the
compact view, no fault. Linux: the standalone built and opened (Hyprland 0.56 under XWayland).

**Not tried:** macOS (the app window now opens through the patched baseview; CI builds it),
200 % and more (not offered on this display), two displays at different scales (the window
opens on the primary display, at its scale). Linux under a tiling window manager shows the
same black space when the manager tiles the window larger than the panel (Hyprland tiled it
2536 by 1390); the X11 code is the same as before, so this change leaves it as it was.


## R-LINUX-ICON. The Linux standalone identifies itself to the dock

**Owner request, 2026-10-18.** Follow up issue #49, a source build showing a gear in Ubuntu
26.10: rebase the existing fix, check X11 and Wayland identity, "document in the README how a
source build gets its desktop file and icon", and verify on Hyprland and Ubuntu GNOME.

**Agent decisions, 2026-10-18** (not separately approved):
- Both Linux standalone adapters use the local patched baseview. A top-level X11 window's
  initial title becomes both strings of `WM_CLASS`, set before mapping; parented editors keep
  their host's identity. The adapters use `P::NAME`, `CA-72`, which matches `StartupWMClass`.
  Linux retains the editor's non-OpenGL features; the other platforms' dependency is unchanged.
- This is X11, through XWayland on Wayland desktops. `AppOptions.app_id` is a saved-session and
  MIDI identifier, not a Wayland surface id. It stays `com.idlefoundry.ca-72` so existing
  sessions load. A future native Wayland surface would need `com.idlefoundry.ca72`, the desktop
  file's stem. The installed desktop id, icon name and `StartupWMClass` stay as they are.
- The README gives a standalone-only source installation, since a bare executable does not
  register desktop files. Protocol, product-name, dependency and installation checks enforce
  the identities and feature selection. The metadata checks may fetch their locked target
  dependencies, including ones not built on Linux.

**Evidence (the Linux reference machine, 2026-10-18):** Hyprland 0.56.2 reports class and initial
class `CA-72`, `xwayland: true`; `xprop` confirms both class strings and the title. Ubuntu 26.10's
GNOME Shell 51.0, in a new VM's live Wayland desktop: the bare binary shows a gear; the same
binary after `install.sh --standalone-only` shows the CA-72 icon. The installed desktop entry
launches it, and its pinned icon groups with the running window. Screenshots retained with the
job's summary. Protocol tests: 4 passed; plug-in tests with production standalone: 308 passed,
31 ignored; nih-plug library: 68 passed; Python identity: 3 passed; installer: 4 passed. Product
and legacy release builds, plug-in clippy with default/all features, baseview's default-feature
clippy, formatting and notices pass. Direct vendored clippy has existing upstream warnings;
checks with allowances limited to those categories pass. Details and remaining limits are in
`docs/issue-49-linux-icon.md`. No native Xorg session, other-platform builds or playback test;
no full workspace repeat beside the other agents' runs. Independent source review found no
remaining defects.

**Review correction, 2026-10-19** (agent decision, not separately approved). A review found the
fatal-error window, titled `CA-72 — error`, would take that title as its X11 class and so miss
`StartupWMClass`. The class now comes from the application: the standalone names itself first
(`identify_standalone`, `Ca72::NAME`, through baseview's new `set_application_class`), and every
top-level window takes that class whatever its title; without one, the title stays the class.
A protocol test for a window titled `CA-72 — error` failed before the change (its class was the
title) and passes after; a plug-in test checks the class against the launcher and that `main`
names the application before anything else.


## R-INSTALL-DESTINATION. Choose the standalone application's folder

**Owner request, 2026-10-18 (issue #54).** Support a chosen installation folder, including an
external SSD: "Plug-ins must stay where hosts look for them (VST3 and CLAP's standard folders);
a custom location applies to what can live anywhere: the standalone application and its files."

**Agent decisions, 2026-10-18** (not separately approved):

- **Windows shows the destination page on every install and upgrade.** Program Files remains
  the default; Inno Setup remembers the previous folder with the same AppId. The page names
  the fixed VST3 and CLAP folders and explains what moves: the optional app, licences and
  uninstaller. It also explains that presets and settings stay, and an external drive must
  be connected to run, update or uninstall the app.
- **An upgrade stays in the previous folder.** Changing it requires uninstalling first, then
  installing again; otherwise the old app and uninstall log would be stranded. A missing drive
  stops before any installed plug-in is replaced, including on a silent update. Reconnect it
  with the same drive letter. No installer entry removes the user's presets or settings.
- **Linux's `--prefix` applies to the app and its licence files.** VST3 and CLAP stay in the
  usual user folders, or `/usr/lib` with `--system`; the desktop entry and icon also stay in
  their usual folders, so the menu can find them. A prefix must be absolute and already exist,
  to avoid silently creating a replacement folder when a drive is absent. The same prefix and
  component/system flags are needed to update or uninstall. Check that the drive is mounted
  at the same path; an empty mount point alone does not prove a drive is connected. Earlier
  scripts moved plug-ins with `--prefix` too; their copies need uninstalling with the earlier
  script first. The README says so.
- **The Mac app can move in Finder after installation.** Keep its bundle whole; reconnect its
  drive to run it. The package still installs and updates `/Applications/CA-72.app`; replace
  the moved copy after an update. Its choice description and the README explain this. The
  package's destinations, bundle relocation settings, signing and notarisation are unchanged.
  A package chooser for arbitrary app folders is not added.

**Evidence (2026-10-18):**

- Linux: `python3 scripts/test-standalone-install.py`, nine tests passed, including GIO
  launching paths with spaces, percent signs, quotes, dollar signs, backticks and backslashes;
  fixed plug-in and desktop folders with a custom app prefix; an update; missing-drive
  rejection before replacing a plug-in; default installation; and uninstall keeping presets,
  settings, a session and an unrelated file. Shell syntax checks passed for `install.sh` and
  `package.sh`; the macOS distribution XML parses with the new description.
- Windows 11 (build 26200), Inno Setup 6.7.3: the production script compiles. The production
  destination page was opened and its rendering checked. For installation, a copy changed
  only the AppId and the system folder constants to isolated fixture folders: the owner's
  installed CA-72 must not be replaced. The payload was the public 0.2.6 Windows ZIP, checked
  against its file manifest. A `subst S:` drive stood in for an external disk; no second
  writable physical drive was available.
- The isolated installer installed all components with the app in
  `S:\Apps with spaces\CA-72`, and the installed app opened a native window with
  `IDLE_FOUNDRY_SHARED` in the test folder and `--no-devices`, then closed normally and saved
  its session. An upgrade from fixture version 0.2.6 to 0.2.7 (version metadata only; the same
  released binaries) remembered the app folder without `/DIR`. An upgrade from the original
  installer layout also retained its folder. A requested folder change and a disconnected
  drive were rejected. Uninstall removed the installed binaries, shortcut and test registry
  entry, keeping user data and an unrelated file. The real installation's registry entry,
  presets, settings and installed binaries were unchanged by hash.

- `cargo test --workspace`, ngspice 47 required, four build jobs and four test threads,
  isolated shared data: 494 passed, 0 failed, 56 ignored, exit 0.
  `cargo fmt --all -- --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` also passed.
  An independent source review found no actionable regressions.

Not tried: a physical external SSD; a host rescan from real system plug-in folders (those
folders were deliberately untouched); Mac app relocation or a rebuilt, signed and notarised
Mac package. No Rust source or crate manifest changes are included.

**Review correction, 2026-10-19** (agent decision, not separately approved). A review found a
prefix that exists but cannot be written, such as a drive mounted read-only, passed the folder
check; the plug-ins were then replaced before the app's first write failed, leaving new
plug-ins with the old app. `install.sh` now writes and removes a probe file in each chosen
application destination (or its nearest existing folder) before touching any plug-in, and
stops with "not writable; nothing was changed". A new installer test, with an installed prefix
and an empty one made read-only, failed before the change (the VST3 was replaced) and passes
after; all ten installer tests pass. The same script under a real read-only bind mount, in a
user namespace, left the plug-ins as they were.


## R-WHATS-NEW. Offline release highlights after an update

**Owner instruction, 2026-10-18 (#75, overnight job).** Build the machinery with a
"deliberately plain, clearly temporary presentation, kept separate from the logic so the
chosen look can replace it". The owner is choosing between a dialog in the AUDIO & MIDI
style and a drawer on the panel's face. Neither look is decided here. SETTINGS already
shows the installed build's version (R-VERSION).

**Agent decisions, 2026-10-19** (not separately approved):

- **Bundled highlights.** `docs/releases/notes.toml` condenses the maintained Markdown
  notes, with Features, Performance improvements, Bug fixes. This checkout has sources
  for 0.2.3 to 0.2.7; missing historical changes are not invented. The current entry follows
  0.2.7's draft, which must be revised along with the draft before publication. A release
  test checks the workspace/build version, current entry and maintained releases. CI runs
  it before bundling on each platform. `docs/whats-new.md` records the release steps.
- **Ranges.** A safe previous semantic version selects every available entry after it
  through the installed version. The view lists exactly the releases included, and each
  combined item names its release. First installs and unsafe history show current
  highlights. Semantic-version precedence ignores build metadata; a downgrade and the
  same version show nothing automatically.
- **One claim on this computer.** The existing shared `CA-72/settings.toml` stores
  `last_version_shown`, under its existing file lock. Reading, comparing and replacing
  happen under one lock, preserving other keys. The highest version shown is retained
  across downgrades. The claim is kept before displaying, so a crash immediately after
  it can consume that automatic showing; Settings remains available. Unreadable or
  unwritable settings suppress the automatic display without overwriting a bad file.
- **Only a visible editor.** The claim is made in the editor window's frame, after a
  visibility check, never in construction, activation or processing. The claim's check is
  stricter than the repaint policy: X11 requires a mapped, viewable window, Windows a
  visible window with its top window not minimised, macOS an attached, unhidden view with
  its window visible. A remote AU uses its attached view rather than unreliable backing
  window occlusion (R-AU-REMOTE-VISIBILITY). No audio or project state participates.
- **Manual reopening.** SETTINGS > WHAT'S NEW always opens the current release's
  highlights, without claiming or rewriting history or starting an update check.
- **Temporary view.** A separate panel module accepts prepared content, wraps and scrolls
  it inside the current viewport in either look and lettering size, including compact
  views. Page buttons, the wheel and keyboard navigation reach all entries; CLOSE,
  Escape and Enter dismiss it. It does not resize the saved window or become project
  state. Replaced by WN-A below.

**Verification (Linux x86, Rust 1.97.1, 2026-10-19).** The touched crates' tests pass,
including release ranges, unknown history, downgrades, restart persistence, twelve
simultaneous threads and eight separate host processes claiming one version. A real CLAP
entry-point test constructs instances, processes live and offline, and saves state with no
editor: shared history is untouched on both a fresh install and an existing settings file.
Editor tests show, dismiss and reopen the notes without sound gestures, preset changes or
saved-field changes, with audio identical to the bit. A held typing note continues and its
release passes through the notes. The view's tests wrap, scroll to the last entry and close
at multiple widths, both lettering sizes and compact/full heights; eight editor renders
cover both looks. Both approved panel images still pass. Clippy for the touched crates,
all targets, passes with default and all features. `target/whats-new-evidence/` holds the
logs and PNGs. No native host, installed plug-in or audio device was opened; macOS and
Windows execution and the remote AU's visibility still need host acceptance. The
current-only manual view and claim-before-display policy await the owner's confirmation at
review; the appearance is WN-A, below.

The full workspace run, including standalone support with ngspice 47 required, passed
537 tests, 0 failed, 57 ignored at `56b9a1e`. Independent source review found one macOS
ABI mismatch: AppKit's `BOOL` is a signed char on Intel and a Rust bool only on Apple
silicon. The visibility check now uses `objc::runtime::BOOL` for both native Boolean
results. It also requires `NSWindow.isVisible` for remote AU views, bypassing only the
occlusion bit. The existing remote-view evidence above had `isVisible=true`, so this
preserves that observed case while excluding an ordered-out backing window. A hidden
host proxy whose backing window remains visible still needs native host acceptance.
This final correction is macOS-only; Linux does not compile or execute those calls.

**Owner's choice, 2026-10-19: WN-A.** Of the two mock-ups on the review page, the owner
chose WN-A: the notes in a dialog in the style of the standalone's AUDIO & MIDI dialog,
drawn over the dimmed editor, in every build (the plug-ins and the standalone). Its
colours and sizes are that dialog's: the box #1f2228, edged #4a4640, corners of 20, over
#08090b at .72; WHAT'S NEW at 52 in #f0a030; under it, at 31 in #a6a5a1, the version
installed and the releases covered ("CA-72 0.2.7 is installed · Changes since 0.2.5,
including 0.2.6"); FEATURES, PERFORMANCE IMPROVEMENTS and BUG FIXES at 30, letter-spaced;
the notes at 33 in #ebe6d8 after a "·"; at 27 the footnote "Shown once after an update ·
SETTINGS › WHAT'S NEW shows it again"; one button, GOT IT, at DONE's place and in its
style. Told of the Mac test build's stand-in, which filled the window: "on standalone, i
don't like that this What's New screen wasn't a modal." So the dialog is a bounded box,
inset from the window's edges at every size, the editor visible behind it; nothing behind
takes a press, wheel or key while it shows; GOT IT, Escape or Enter closes it. The stand-in's
paging row (PREVIOUS, NEXT, the line count, CLOSE · ESC) is gone: scrolling is enough.

**Agent decisions, building WN-A (2026-10-19):**

- **Sizes.** The dialog keeps the AUDIO & MIDI dialog's place and width in the drawing, so
  it shrinks with the window. Its height follows the view: the mock-up's margins (155 and
  140 of 1775) in the full view; in the compact view, 60 and 55, with the footnote beside
  GOT IT, to leave the notes room. LARGE lettering sets all of its text and spacing a
  quarter larger, as the menus and tips (`Lettering::text`).
- **The notes.** They wrap under their bullet (the following lines start where the text
  does) and scroll a line at a time: the wheel or trackpad, the arrow keys, Page Up and
  Down, Home and End. A slim scroll bar in the dialog's colours (#15171b, thumb #82715c)
  shows only when they are longer than the dialog; a press on its track pages. A section's
  title is never left alone at the foot. Combined releases keep naming each note's release
  (above), set before it in the subtitle's grey rather than as "0.2.6:", which the mock-up,
  showing no labels, did not settle. The owner should confirm the labels.
- **Coverage wording.** "Highlights of this release" when no earlier shown version is
  known; "Changes since 0.2.6" for one release; "Changes since 0.2.3, including 0.2.4,
  0.2.5 and 0.2.6" for several, naming only releases with bundled notes.
- **Speed.** The dialog's picture is made once for a scale and layout; the notes are drawn
  into a strip a line at a time as each first shows; each frame dims the editor with a
  table, copies the dialog's opaque rows and blends only its edges, then copies the lines
  that show. Measured in a Retina window (3440 pixels across, full view, notes since 0.2.3,
  release build, `release_notes_frame_timings`): opening 33 to 41 ms, the first the
  slowest (the stand-in's 28 to 29 ms, measured the same way at 68b30ba); a frame with the
  panel changing behind it 6.5 ms (stand-in 25 ms); a line scrolled 6.6 ms (stand-in
  25 ms); the panel alone 1.1 ms. Opening still makes its pictures on the UI thread.
- **Review fix (PR #106).** Trackpad travel left over from one direction is cleared when the
  direction reverses, as in the drawer's list; the line pitch, not a fixed 60 units, sets
  the travel. A regression test fails without the fix.

**Verification of WN-A (Linux x86, 2026-10-19).** The panel module's tests wrap every line
clear of the scroll bar, scroll to the last note and back, page from the track, close with
GOT IT, keep the dialog inside the compact and full views at both lettering sizes and three
widths, and show lines drawn as they come into view to be identical, to the bit, to lines
drawn at once. Editor tests close the notes with GOT IT in both looks, both lettering sizes,
compact and full; in the standalone at its default and smallest widths, compact and full,
they find the window's edges dimmed outside the box and no control, menu, drawer, note or
host gesture reached by presses, wheel, right clicks or keys behind it. Renders for the
owner, with a comparison with the mock-up, are in `target/wna-evidence/` of the job's
worktree. No macOS or Windows host has shown it yet.

## R-ARP. An arpeggiator, in a drawer dropped from the rail's ARP key

**Owner decisions, 2026-10-18** (issue #90, its comment of that day, and the night's brief). In
every build (VST3, CLAP, AU and the standalone), for 0.3.0:
- **Modes and range:** "up, down, up/down and random, with an octave range": DIRECTION is UP |
  DOWN | UP/DOWN | RANDOM, OCTAVES 1 to 4 (ORDER and OCTAVES 5 added 2026-10-19, below).
- **Rate:** "tempo-synced divisions only": 1/2, 1/4, 1/8, 1/16 and 1/32 with the triplet and
  dotted values between them; the dial numbered 2, 4, 8, 16 and 32 at the straight values, the
  others unnumbered ticks; the readout the value's number (16). It follows the host's tempo; "the
  standalone needs its own tempo": a simple TEMPO setting where the standalone's other settings
  are, 120 by default (its final look the owner's to design).
- **Latch, gate, swing:** LATCH (hold; HOLD on the panel since 2026-10-19, below); GATE, the
  note's length, 0 to 100 %; SWING, 0 to 100 %, 0 straight.
- **Transport stopped:** "keeps running at the host tempo while keys are held, and locks to the bar
  when the transport plays".
- **Voicing:** "arpeggiated notes feed the existing engine exactly as played notes do, through
  mono/poly/unison, NOTE PRIORITY, RETRIGGER and glide. The arpeggiator does not bypass or
  duplicate voice allocation."
- **Presets:** "arpeggiator settings are saved with the patch. Existing presets and saved projects
  load with the arpeggiator off and sound unchanged."
- **Automation:** "every arpeggiator setting is a host parameter (ON, LATCH, DIRECTION, RATE,
  OCTAVES, GATE, SWING)". **MIDI:** no MIDI out.
- **Placement, C1** (mock-ups drawn by the panel's renderer on a throwaway copy; the owner chose
  the drawer, then "make it even slimmer", then C1; rejected: a page of its own, the rail's dot
  screen, a two-row drawer, a drawer of readouts alone): an ARP key on the rail after the
  keyboard's, the name's screen a key shorter (star, previous, name, next, SAVE, PANEL, keyboard,
  ARP, settings); it drops a one-row drawer, 265 units deep, inside the window over the top of
  whichever lower page shows, as the presets' drawer drops; ARP again puts it away; the key lit
  while the arpeggiator runs. The row: the ARP bank (ON | LATCH), the DIRECTION bank, a rule, then
  RATE, OCTAVES, GATE and SWING as knob-and-readout units of the strip's own parts. "The mock's
  drawer lip and drop shadow were a quick composite": a real edge, as the presets' drawer has.

**Agent decisions, 2026-10-18** (not separately approved):
- **One path for every key.** Every key the note sources pass (the host's MIDI on any channel, the
  standalone's keyboard and computer keys) goes through the arpeggiator (`arp.rs`) to the
  engine's `Event::Note`; off, it hands each on at once, unchanged, so the sound without it is
  the code's before to the bit. Its notes are the engine's events at their samples, the run
  split there as at a host's event, after the host's events at the same sample. It plays one
  key at a time (a monophonic arpeggiator): in POLY each step takes a voice, in UNISON all.
- **The order.** UP and DOWN by pitch, the next above (below) the last played, from the other end
  past the last; UP/DOWN bounces, its ends once (60 64 67 64 60 64…); a chord changed while it
  plays carries on from the pitch it last played, and from silence it starts again from the
  pattern's beginning. OCTAVES repeats the keys an octave up each time, none past 127; a pitch
  two keys reach (C4 and C5 with two octaves) is played once. **RANDOM's rule:** a key chosen
  evenly among the pattern's pitches but the last played, while there are two or more (one
  alone repeats); its numbers from the session's noise seed, so a render repeats.
- **RATE's thirteen values**, slow to fast as the dial turns, each straight value then its dotted
  and its triplet (the dotted the longer): 2, 4., 2t, 4, 8., 4t, 8, 16., 8t, 16, 32., 16t, 32;
  the dial a tick every 25 degrees, the straight ones numbered; the readout `16.` for a dotted
  sixteenth (the segments' point) and `16t` for a triplet (a seven-segment t). Its host
  parameter names them "1/16 dotted", "1/8 triplet". The default a sixteenth.
- **The grid.** While the host plays and says where it is, the steps are counted from the start of
  each bar (its bar start and time signature), the last of a bar cut short at the bar's end (a
  dotted quarter in 4/4: 0, 1.5 and 3 beats, then the next bar's downbeat); without a bar,
  from the song's start. A key pressed while silent waits for the next step, but one pressed
  up to a quarter of a step after a step plays at once as that step's (a key a little late
  is not held back a whole step). A jump of the host's position (a loop, a locate) finds the
  grid again there, the step on a loop's start played at its first sample; the note sounding
  keeps what was left of it. Stopped (or a host that gives no position), it runs at the host's
  tempo (else TEMPO) from the moment a key is pressed; the transport stopping while it plays,
  it goes on in time; starting, it finds the host's grid. A tempo change moves the steps on
  at the new tempo from where they were (times are kept in beats).
- **GATE** (superseded 2026-10-19, below: 5 ms was silent): each note let go at GATE's share of
  its own step (a swung step's length), but never shorter than 5 ms (`arp::SHORTEST`), so 0 %
  sounds a blip (the contours, DECAY on, then
  release as from a quick tap); at 100 % tied: the next key pressed before the last is let go
  (single triggering and GLIDE take it as legato), but for the same key again, which is let go
  first. The default 50 %.
- **SWING:** every second step of each bar late by SWING × half a step: 100 % puts it three
  quarters of the way through the pair (a drum machine's 75 %), 67 % is the triplet shuffle.
- **LATCH:** a key pressed with none held (none down, none held by the pedal) starts a new chord;
  one pressed while others are held adds to it; the chord plays on after its keys are let go,
  until a new one or LATCH off, which lets it go at once. ON switched off with keys held: its
  note let go and the keys pressed again, as played; on: the engine's keys let go and played in
  turn. A preset or session that switches it off while it plays does the same at the block's
  first sample: nothing is left down.
- **The sustain pedal** (CC 64, ≥ 64 down), which the instrument has never followed: the
  arpeggiator's while it is on, the keys let go while it is down held as if they were still,
  until it comes up (Reset All Controllers lifts it). Played without the arpeggiator the pedal
  does nothing, as before (a project with pedal data keeps its sound). A controller MIDI Learn has
  on CC 64 is that parameter's alone, as before. **All Notes Off, All Sound Off, a reset and
  PANIC** forget the held, pedalled and latched keys (LATCH stays on).
- **The parameters**, after every other (no id moves): `arp_on` ("Arp"), `arp_latch`,
  `arp_direction` (`up`, `down`, `up_down`, `random`), `arp_rate` (`1/2`, `1/4d`, `1/2t`, … `1/32`),
  `arp_octaves` (1 to 4), `arp_gate` (%), `arp_swing` (%). Sound parameters: saved with presets
  (`presets::sound_params`; a preset that does not name them plays without it, every factory
  preset so) and sessions; a session saved before them opens with them at their defaults, off,
  whatever the instance had (`Ca72::filter_state`). Turning it on marks the preset changed. MIDI
  Learn takes them all (65 learnable: ON and LATCH switches, DIRECTION, RATE and OCTAVES
  selectors, GATE and SWING knobs); the knobs' right-click menus learn as the panel's.
- **TEMPO** (the standalone, which has no host to give one; and any host that gives none; its
  screen on the wood since 2026-10-19, below): the AUDIO & MIDI dialog's last row, "120 BPM (the arpeggiator's; applied at once)", a list every
  5 from 40 to 240 and the arrow keys a beat, 30 to 300; applied at once, outside the devices'
  APPLY (nothing to restart), and kept for the computer in `settings.toml` as
  `standalone_tempo` (not in the session: the owner will design its final look and may move it).
- **The drawer** (`ca72_panel::strip::arp`): x 372 (the left hand's rule stays seen) to the strip's
  right trim, from under the rail 265 deep; the banks at x 565 and 1091, the rule at 1440, the
  units at 1644, 2046, 2448 and 2850 (repacked for ORDER 2026-10-19, below), the row 142 under its top, the legends and the banks'
  titles 109 above it, as the mock-up. Its face the strip's (the worn skin's picture laid from
  another place, so its grain does not run on from the face under it, the lamp's light over
  it; the drawn skin's black); the rail's shadow across its top as the presets' drawer has, its
  left edge catching the lamp, its foot shaded to a dark line, and its shadow cast 16 units on
  the page below (laid over the page, not a copy). It slides as the presets' drawer (140 ms).
  Open, it is all the pointer finds there (the strip's tabs and knobs and the keyboard's header
  under it do nothing); sliding, nothing. The presets' drawer opens over it; ARP while they show
  puts them away and shows it; PANEL hiding the lower panel puts it away; ARP with the panel
  hidden shows it. The key is lit while ON is on (taken as "while the arpeggiator is running":
  the owner to confirm, whether it should light only while notes play).
- **Its knobs are the panel's controls** (`controls::CONTROLS`, four appended, `Place::Arp`,
  found by `interact::hit_arp` while the drawer is open): dragged, scrolled, double-clicked to
  their defaults, tipped and learned as every knob; drawn by the drawer, through the panel's
  renderer (`Renderer::paint_control`) in the skin it draws. RATE and OCTAVES go from position
  to position while dragged (`Dial::steps`, as VOICES). Its readouts take a value typed, double
  clicked as the strip's (R52): RATE `16`, `16.` or `16t` (one of the thirteen; any other
  nothing), OCTAVES a digit, GATE and SWING percent.
- **Allocation-free:** the keys bit sets, the times beats; `tests/realtime.rs`
  `the_arpeggiator_neither_allocates_nor_frees` plays it through the plug-in's process with every
  setting automated, the transport playing, looping and stopping, the pedal and All Notes Off.
- **nih-plug change 11** gains `TestProcessContext::set_transport` (`PATCHES.md`), for the tests.

**Evidence (omarchy, Linux, Rust 1.97.1, 2026-10-18; under other agents' builds, load 8 to 16):**
- `arp.rs`'s 17 tests: the pattern orders, RANDOM's rule and its repeat, steps on their samples
  whatever the block (1, 64, 1024, 4096) within a sample of the grid at 120/48 kHz, 97.3/44.1 kHz,
  174/96 kHz and 60/22.05 kHz, the host's grid and a late key, a bar's cut, SWING, GATE's blip and
  tie, latch, the pedal, ON handed over, clearing, the transport stopping, starting and looping,
  a tempo change. The plug-in's: `arpeggiated_notes_play_through_the_engine_as_played_ones` (a
  chord arpeggiated with the transport playing renders to the bit what the same keys played at
  its steps' samples render without it: MONO with RETRIGGER and POLY, blocks of 256 realtime and
  1000 offline), `nothing_is_left_stuck_by_resets_latch_or_presets`,
  `the_sustain_pedal_reaches_the_arpeggiator_unless_learned`,
  `an_old_session_opens_with_the_arpeggiator_off`,
  `the_arpeggiator_is_part_of_a_preset_and_off_in_one_that_does_not_name_it`, the settings' and
  the dialog's TEMPO; the editor's `the_arp_key_drops_its_drawer_over_the_lower_page` (the frame
  put together whole, nothing under it answering, the presets' drawer over it, PANEL) and
  `the_arp_drawer_operates_its_parameters`; the drawer's `drawn_in_parts_it_is_drawn_whole`
  (both skins, two scales). `cargo test -p ca72-plugin` (default and `standalone-production`)
  and `-p ca72-panel` pass; clippy `-D warnings` with all targets.
- The standalone (release build, `IDLE_FOUNDRY_SHARED` in the worktree) on omarchy: the drawer
  dropped over the keyboard and over the strip, ON lit with the ARP key, TEMPO chosen from its
  list and kept (`standalone_tempo = 100`). The notes were not listened to.
- Frame times (`cargo run --release -p ca72-panel --example frames`, the editor's drawing and
  composing, the worn skin; the code before and after run in turn, the quieter pair): the
  panel's and the strip's cases as before within the machine's noise (0.87 of the drawing:
  CUTOFF 2.7 and 2.7 ms median, DRIVE 6.6 and 6.7, a tab 1.8 and 1.8; at 1.0: CUTOFF 3.7 and
  4.6, DRIVE 5.6 and 7.6, a tab 2.4 and 2.4). The drawer's renderer made as the editor opens:
  0.03 to 0.06 ms on its thread (its plate made on a thread of its own, about 65, 145 and 125 ms
  at 0.44, 0.87 and 1.0); its first drop once that is done: 6, 18 to 25 and 16 to 24 ms (its four
  knobs drawn the first time); at once after opening, waiting for the plate: 64 to 155 ms. A knob
  turned in it: 1.2, 4.0 to 5.7 and 2.5 ms median; a tab pressed 0.1 to 0.5 ms; open and still,
  nothing. While it slides the whole frame is put together, as for the presets' drawer.

**Not done:** listening to it in a host or the standalone; the installed plug-ins in a DAW (the
transport's bar start and loops as each host reports them, Logic's and Ableton's block splits);
the Mac and Windows builds; the drawer's knobs dragged in the standalone (the editor's tests drag
them; in the standalone its tabs were clicked by synthetic X events).

### R-ARP, 2026-10-19: HOLD's drone, HOLD, ORDER, five OCTAVES, the TEMPO screen, GATE

**Owner decisions, 2026-10-19** (issue #90's comments of that day, 6269120007 among them, after
playing PR #96; and the owner's requests relayed with the night's brief):
- **A drone is ARP off with LATCH on.** "LATCH with ON off holds the notes played. They keep
  sounding after the keys are released, as played (no arpeggiation), until LATCH is turned off or
  a new chord is played. 'A new chord' means the same rule as the arpeggiator's latch, so the two
  behave alike." "Through the engine as played: held notes go through mono/poly/unison, NOTE
  PRIORITY, RETRIGGER and glide." "With LATCH off, every sound stays bit-identical." "No stuck
  notes after LATCH off, All Notes Off, PANIC, a preset change or a reset. Test the sustain
  pedal with a held drone, and turning ON on and off while notes are held." A rate of zero was
  considered and turned down ("most synths use a hold or latch that also works with the
  arpeggiator off").
- **LATCH becomes HOLD:** "Rename LATCH to HOLD on the panel, in the host parameter's display
  name and in the docs. Keep the parameter ID so saved patches and projects still load."
- **ORDER:** "notes play in the order they were pressed (Moog's ORDER, Ableton's Play Order,
  Logic's As Played). It's a fifth button after RANDOM", "a key added while held goes on the end;
  a key let go leaves the order"; the buttons 140 units, the banks at 555 and 1114, the rule at
  1512, the units at 1716, 2099, 2482 and 2865, as rendered for the owner.
- **OCTAVES 1 to 5:** "let's add 5 octaves as an option in the arp. I think it will make the
  spacing around the knob look better." Past the top: "For high notes, I would repeat the highest
  octave available of the selected note rather than go silent."
- **The standalone's TEMPO, option D:** "a dot-matrix screen on the top wood band, mirroring the
  CA-72 badge: the same size, height and distance from the edge. It's always in view, and the
  rail and drawer are unchanged. Standalone only: in a plug-in the tempo comes from the host and
  the wood stays bare. Setting it should work like the other readouts (drag, scroll,
  double-click to type); the AUDIO & MIDI row can stay or go."
- **GATE was silent at short settings:** "At 1/32 and 120 BPM with GATE at 12 %, nothing sounds",
  the note let go before `TRIGGER_ON`'s 7.8 ms; GATE 0 % silent at every tempo; at GATE 90 % the
  6 ms gap under `RETRIGGER_GAP` loses retriggers. The fix: "GATE stays a percentage, but of the
  window the circuit can play. Each note is held at least about 20 ms and let go at least 13 ms
  before the next, except at 100 %, which ties notes. Where a step is too short for both (1/32
  above about 225 BPM), the note sounds and the gap shortens."

**Agent decisions, 2026-10-19** (not separately approved):
- **The drone** (`Arp::drone`): with ON off and HOLD on each key is handed on as pressed; a key let
  go is held on; a key pressed with none held (none down, none held by the pedal) lets the held
  chord go first; one pressed while it holds is let go and pressed again (struck again, never
  down twice). HOLD switched on while ON is off holds the keys down at that moment; off, it lets
  go of what it holds but not of keys still down. **ON and the drone hand over:** ON switched on,
  the drone's keys are let go and arpeggiated, HOLD's chord the arpeggiator's; off with HOLD on,
  the chord (held and latched keys, and those the pedal holds) is pressed again as a drone. With
  HOLD off the path is the old one, untouched: keys pass straight through.
- **The sustain pedal under a drone** does not sound anything itself (the instrument still
  ignores CC 64): it keeps the keys let go under it part of the chord, so that a key pressed
  then adds to the drone rather than starting a new one, as under the arpeggiator's HOLD.
- **ORDER** keeps the keys in the order they were last pressed (`Arp::order`, a list of 128, each
  key once: pressed again, it moves to the end), passing over those no longer played. With
  OCTAVES it plays the order, then the order an octave up, and so on (Ableton's and Logic's way);
  a pitch two keys reach is played by each (UP, DOWN, UP/DOWN and RANDOM still play it once). A
  key let go: the one now in its place plays next. ORDER is the enum's last value
  (`arp_direction` `order`), so saved UP to RANDOM keep their values.
- **Past 127** (`arp::audible`): the pattern is the one it would be with room above (pitches kept
  to 255 in `Wide`), each note brought down by whole octaves into range as it is played: G7 (103)
  with OCTAVES 5 plays G7 G8 G9 G9 G9 under UP, and its UP/DOWN turns at the pattern's top (G7 G8
  G9 G9 G9 G9 G9 G8 G7), so every step sounds and the pattern keeps its length. RANDOM never
  plays the note just heard while another may sound (judged by the note heard, so collapsed
  repeats are passed over). At GATE 100 % a repeat of the same note is let go and pressed again.
- **OCTAVES 5:** `arp_octaves` 1 to 5. Presets and sessions keep it plain (an `i32`; the
  session's `arp_direction` an id), so a saved 1 to 4 loads as 1 to 4 and needs no translation.
  A host's automation lane or a MIDI-learned controller holds normalized values and maps onto the
  new range (4 positions became 5, DIRECTION's 4 became 5); no release has had the arpeggiator,
  so only test builds' projects could hold such a lane. The dial: five numbered positions a
  quarter of its travel apart (75 degrees).
- **GATE** (`arp::SHORTEST` 20 ms, `arp::REARM` 13 ms): a note is held `SHORTEST + GATE × (step −
  SHORTEST − REARM)`; a step shorter than both (1/32 above 225 BPM) holds the note `SHORTEST` (or
  the whole step, past 750 BPM) and the gap is what is left. 100 % ties as before. GATE 50 % at
  1/16 and 120 is now 66 ms, not 62.5.
- **The TEMPO screen** (`art::TEMPO_SCREEN`, `art::tempo_screen`): the rail's screen's bezel and
  glass and dot lettering (`strip::pixel_screen`), its surround the plate's 261 by 82 units at
  x 269 to 530, y 24 to 106; a layer of the panel's renderer always (empty in a plug-in, so that
  the layers' count stays), its bezel and glass made once a scale and kept, its dots drawn as it
  changes. It reads "120 BPM". A drag sets a beat a minute every 2 logical pixels up or down
  (SHIFT: every 8), a wheel's notch one, a trackpad's 8 pixels one; a double click types it (up
  to three digits, `_` marking the next; Enter sets it within 30 to 300, Escape leaves it); kept
  for this computer as before (written as a drag ends, not at each pixel). **The AUDIO & MIDI row
  stays,** a second way in: it already reads and writes the same setting, and taking it away
  would change the dialog the owner approved for nothing.
- **The drawer's percent signs** sit 18 units from their readouts (26 before), so that SWING's,
  moved right by ORDER's repacking, stays inside the trim (it was cut in half there, the mock-up's
  too).
- **The 0.3.0 draft notes** moved to `docs/releases/drafts/0.3.0.md`: R-WHATS-NEW's test wants a
  bundled entry for every release file up to the workspace version, and 0.3.0 is not that yet.

**Evidence, 2026-10-19 (omarchy, Linux):**
- Unit tests in `arp.rs` (22): ORDER's order, a key added at the end, one let go, one pressed
  again, ORDER with OCTAVES and HOLD; OCTAVES 5 past the top in all five directions (every step
  sounding, none past 127, RANDOM's rule kept, G9 alone repeated, HOLD); the drone held, a new
  chord, a key struck again, HOLD off with a key still down; the drone and the arpeggiator
  handing over, the pedal, clearing; GATE's floors at every RATE, at 30 to 300 BPM and GATE 0 to
  99 % (the note at least 20 ms, the gap at least 13 ms where the step allows, the note kept where
  it does not), and the owner's case (1/32, 120, 12 %: 23.5 ms). The plug-in's:
  `a_held_drone_plays_through_the_engine_as_held_keys` (a drone, a new chord, renders to the bit
  what the keys held by hand render: MONO with RETRIGGER, MONO with GLIDE and HIGH priority, POLY,
  UNISON; blocks of 256 realtime and 1000 offline), `a_drone_leaves_nothing_stuck` (HOLD off, All
  Notes Off, PANIC, a preset, a reset, ON on and off, the pedal, POLY: no key down in the engine
  after each), the presets' plain values for OCTAVES 4 and RANDOM, the editor's
  `the_standalone_s_tempo_screen_sets_the_tempo` (drag, wheel, typing, the limits, Escape; none in
  a plug-in) and `the_arp_drawer_operates_its_parameters` (ORDER, OCTAVES 5 typed), the panel's
  `the_tempo_screen_mirrors_the_name_plate` and its part-and-whole test with the screen shown,
  set and gone; `the_arpeggiator_neither_allocates_nor_frees` with ORDER, OCTAVES 5, a key at 124
  and the drone among its automation.
- **Renders, measured, not listened to** (`tests/arp_renders.rs`, ignored, run by hand through
  the plug-in's process as the standalone runs it; the WAVs in the night's `arp2-out/wav` and
  `wav-before`): 1/32 at 120 with GATE 12 %: 0 of 32 steps sounding before (-240 dBFS), 32 of 32
  after. GATE 0 %: 0 steps sounding at any of the 13 RATEs before, every step at every RATE after,
  and 40 of 40 at 1/32 and 300 BPM. GATE 90 % at 1/32 and 120, with a plucked contour so that
  each attack is its own: MONO 1 attack in 32 before (7 steps sounding, the rest decayed away),
  32 of 32 after, for a chord and for one key; MONO with RETRIGGER and POLY 32 of 32 before and
  after. (Calibrated by hand-played notes: a 6 ms gap gives 1 attack in 16, 13 ms 16 in 16.) The
  drone: RMS -18 dBFS (MONO) 1 to 2.5 s after the keys are let go, the same after a new chord,
  -170 dBFS or less 2 s after HOLD off; POLY and UNISON alike; ON on and off under it, held,
  arpeggiated, held again. ORDER: G C E pressed in that order, A added: G C E, G C E an octave up,
  G C E A …, measured from each step's zero crossings. OCTAVES 5 from G7: 103 115 127 127 127,
  every step sounding. TEMPO 96 set as the screen sets it, stopped: 16 of 16 attacks, onsets
  155.7 ms apart on average (156.25). With HOLD off and the arpeggiator off, three phrases (MONO,
  POLY, MONO with GLIDE and RETRIGGER) render the same samples on the build before and after
  (the WAVs compared byte for byte).
- **The Linux standalone** (release, `--no-devices`, `IDLE_FOUNDRY_SHARED` in the worktree, on a
  hidden Hyprland workspace, events sent by XSendEvent to baseview's child window, pictures by
  XGetImage): the TEMPO screen scrolled (123), dragged 40 pixels (143), typed (96, its `_` shown)
  and each kept in `settings.toml`; the drawer's ON, HOLD and ORDER pressed, OCTAVES typed 5 and
  dragged to 4, the session saving `arp_octaves` 4 (plain) and `arp_direction` `order`.
- **Frame times** (`frames`, the build before and after run in turn twice, the worn skin): every
  case as before (0.44 and 0.87: CUTOFF 0.7 and 2.8 ms median, DRIVE 1.8 and 6.3, a tab 0.5 and
  1.9, a drawer knob 1.1 and 3.9, the drawer's first drop 5.3 to 5.8 and 16.2 to 17.9 ms). The
  TEMPO screen set: 1.1 and 3.0 ms median (3.4 and 5.7 before its bezel was kept); its first frame
  9 to 12 ms, inside the editor's first. It is a fixed layer, so nothing floats and only its
  place is put together again (counting it as floating had made every frame whole; the editor's
  part-and-whole test caught it).

**Owner rule, 2026-10-19 (later):** "the gate in the arp should NEVER go below the minimum gate
length to trigger a sound in the device." `SHORTEST` (20 ms) was a floor only where the step
allowed it (a step shorter than 20 ms held the note the step's length); it is now absolute:
- **Agent decisions:** every note is held at least `SHORTEST`, whatever GATE, RATE, SWING or
  tempo. A step shorter than that (SWING's short steps at the fastest RATEs, a host's tempo past
  375 at 1/32, a bar's last step cut short) presses the next note while the last is still held,
  and lets the last go at its 20 ms: up to eight notes may sound at once (`arp::OVERLAP`; a ninth
  lets the oldest go early, which takes steps under 2.5 ms). Overlapping notes are the engine's
  as any held keys: POLY gives each a voice, MONO and UNISON play them by NOTE PRIORITY (the
  later sounds legato, or once the earlier is let go), RETRIGGER waits its gap. **The same key
  again** while it is still under its `SHORTEST` cannot be pressed twice: the step is tied into
  the note, which is held on to that step's release (never re-struck short); once it has had
  its `SHORTEST`, the same key is let go and pressed again, as before. The gap floor (`REARM`)
  stays wherever the step has room for both. Letting the keys go, and HOLD off, let a note go at
  once only when it has had its `SHORTEST` (else at it); switching ON off, All Notes Off, PANIC
  and a reset let everything go at once, as the engine does.
- **Evidence:** `no_note_is_shorter_than_the_shortest` (`arp.rs`): all 13 RATEs, at 30, 60, 97,
  120, 180, 225, 240 and 300 BPM and a host's 1000, GATE 0, 12, 50, 90, 99 and 100 %, SWING 0
  and 100 %, for one key, a chord, ORDER with C4 and C5 (a pitch two keys reach) and notes folded
  down from past 127 (UP/DOWN from G8 over four octaves): every note, pressed to let go, held at
  least 960 samples at 48 kHz (the keys let go at the end included), a key down at every step,
  nothing stuck, no key down twice; over 45 000 notes. Renders (`arp_renders` `shortest_floor`,
  `wav/floor-*`): 1/32 at 300 BPM, SWING 0 and 100 % (25 and 12.5 ms steps), GATE 0 and 100 %,
  in MONO with LOW and HIGH priority, MONO with RETRIGGER, POLY and UNISON: every step's window
  sounding (-17.8 to -21.0 dBFS at the quietest) with the default contour. With a contour of
  sustain 0, MONO (single triggering) and UNISON fade at 1/32 and 300 BPM without SWING (18 and
  21 of 48 steps sounding): the 5 ms gap is under the circuit's re-arm time (11.5 to 12 ms), so
  the contours cannot start again; RETRIGGER and POLY sound every step. No gate can change that
  short of a longer step, a slower tempo, RETRIGGER or POLY.

**Not done:** listening, in the standalone or a host (renders measured instead); the installed
plug-ins in a DAW; the Mac and Windows builds; the standalone with an audio device (it ran with
`--no-devices`; its process is the one the renders ran).

## R-FX-HOST. Effects in the standalone, hosted out of process with DAW-v3's sandbox (issue 82)

**Owner decisions, 2026-10-18** (relayed by the coordinating agent): the standalone
application only, for 0.3.0, never the DAW plug-ins; effects only, not instruments; VST3 and
CLAP both ("the code we can reuse hosts both"), AU later; a chain of four with add, remove,
reorder and bypass, each effect in its own editor window, the chain and each effect's state
saved with the standalone's session. The view: "FX-A", an FX key on the walnut rail after
ARP dropping a one-row drawer of four name screens with ON tabs (docs/fx-host.md); not
built yet, to share the arpeggiator's drawer.

**Agent decisions, 2026-10-18 (for the owner's review; docs/fx-host.md has the reasons and
nine questions):**
- DAW-v3's `daw-ipc`, `daw-helper` and test plug-ins vendored in `third_party/daw-v3`
  (two patches: a run loop for VST3 editors on Linux; macOS's wait functions looked up
  at run time); its `daw-sandbox` adapted into
  `crates/ca72-fx`; the plug-ins loaded only in `ca72-fx-helper`, one process per effect.
- Synchronous by default (no latency added), each effect's block waited for until 0.9 of the
  period, then dry; pipelined as an option (`--fx-mode`).
- A crash, a hang (2 s) or a kill: the effect dry meanwhile, its helper started again, the
  effect restored from its last snapshot; given up on after 5 restarts.
- The chain saved in `effects.json` beside `session.json`, not inside it (size, and the
  plug-ins' state and presets kept free of it); a missing plug-in kept with its state.
- The scan's listings cached in `effects-cache.json`, by file size and time.
- Until the drawer exists, `--fx-console` drives the chain from standard input.
- Notices: the helper's crates and Steinberg's MIT notice for the VST 3 interfaces.

**Evidence (Linux, 2026-10-18):** `crates/ca72-fx-helper/tests/chain.rs` with the test
plug-ins in both formats (exact output, bypass identical to the bit, latency, order, save
and restore, a sample-rate change, crash, hang and kill in real time); the standalone with
the default device; Dragonfly Reverb 3.2.10's eight effects loaded, played and restored, their
editors drawn (the VST3 ones' knobs at zero, unsolved). The chain's tests pass on the Mac and on Windows too
(the standalone not built there). A macOS blocker found and fixed: the transport linked
`os_sync_wait_on_address` (macOS 14.4) into the application (minimum 11); now looked up at
run time (third_party/daw-v3/PATCHES.md).

## R-FX-DRAWER. The effects' drawer: an FX key on the standalone's rail (issue 82)

**Owner decisions, 2026-10-18** (FX-A, chosen from the coordinator's mock-ups over an AUDIO &
MIDI-style dialog and slots in the left column; relayed with the night's brief): in the
standalone application only, the DAW plug-ins keeping their rail as it is, an **FX** key on the
walnut rail after **ARP**, the preset's name's screen a key shorter again for it. Pressed, it drops
a one-row drawer inside the window, about 170 units deep, over the top of whichever lower page
shows, "the same mechanics and edge as the arpeggiator's drawer (share code; don't duplicate it;
only one drawer is open at a time)". Four slots side by side, each with a SLOT legend, a dot screen
(the rail's, about 400 units wide, twelve characters) and an ON tab beside it (bypass), lit while
its effect plays. A click on a name opens the effect's own window; a slot dragged onto another
moves it there; a right click offers replace and remove "in the editor's existing menu style"; an
empty slot reads "+ ADD EFFECT" and opens the effects' list, "a dropdown list in the presets
list's style, listing found effects by name and maker", its scan off the editor's thread with a
visible SCANNING. The FX key lit while any effect is active. A failed or given-up effect shown
plainly on its slot, "its name dimmed and a short word ... keep it subtle". The mock's lip and
shadow were a composite: the arpeggiator's drawer's real edge instead.

**Agent decisions, 2026-10-18** (not separately approved):
- **One drawer from the rail, its plate shared.** `strip/lower.rs` is what the arpeggiator's
  drawer and the effects' share: the plate (the strip's face from another place, the rail's
  shadow across its top, its left edge lit, its foot in shade), the print worn into it, the lamp,
  the shadow it casts, the lit tabs and their words. `strip/arp.rs` draws its plate with it now
  (the same look; its edges drawn before its recesses rather than in one pass with them). In the
  editor one `Dropped` holds which drawer is down: ARP with the effects' drawer open puts it away
  and drops its own from the rail anew, and FX the same; PANEL hiding the lower page puts either
  away at once. `compose` lays either the same way, with what hangs under it (its shadow; the
  effects' list).
- **The rail's keys by build.** `strip::Rail`: the plug-ins' (as before, to the pixel) or the
  standalone's, FX at 2912 and the keys left of it a key's pitch (100 units) to the left, the
  name 2000 units wide; chosen as the editor opens, the standalone's rail only when the
  application has a chain (its helper found). The name's surround is the strip's renderer's
  alone now (the drawn skin had it twice, once in the drawing: it would have shown past the
  shorter name).
- **The slots.** Equal quarters of the drawer, each a screen 400 by 50 units in the rail's bezel
  and glass, twelve places of the rail's dots, the name centred and cut with an ellipsis; a 20-unit
  gap, then the 150-unit tab of the strip's amber caps reading ON; the legend over the pair. Every
  tab has room (tested: none touches its screen or the next slot's light). An effect's whole name
  and maker are the tip over its screen. Only the first empty slot reads "+ ADD EFFECT" (an
  effect is always added at the chain's end); the others are dark. A bypassed effect keeps its name
  lit, its tab dark. The small print under a screen, in the legends' colour at 72 %: LOADING while
  it loads, RESTARTING while its helper is started again, CRASHED once given up on (dry), MISSING
  when its plug-in is not on this computer, FAILED for five seconds where a chosen effect would not
  load; its name dimmed with it.
- **Clicks and drags.** A press on a name and a move of more than 12 units carries the slot: its
  name dimmed, the slot under the pointer ringed in MIDI Learn's accent; let go there, the chain's
  `move_to` (the others keep their order). A press let go without moving opens its window. The
  tab bypasses (shown at once). Right click: the effect's name as the menu's title, REPLACE… and
  REMOVE. While something loads, the slots wait (no drag, no menu).
- **The list.** It drops from under the drawer's foot below its slot (clamped within the
  drawer's ends), a plate of the drawer's face with its own edges and shadow, and a display of the
  presets' list's dots and smoothed glass (the rail's glass drawn out over it showed bands; R44):
  a title (ADD TO SLOT n or REPLACE SLOT n) with SCANNING… or the number found at its right, then
  eleven rows a view of name, maker and format (CLAP, VST3: the owner's question 7 in
  docs/fx-host.md is still open; small and dimmer), and SCAN AGAIN last. Sorted by name, maker,
  format. Always eleven rows deep, as the presets' list is, so that its plate is made once, on
  the drawer's thread as the editor opens (first made each drop it cost 105 to 215 ms; now 3 to 10).
  A press elsewhere puts it away and does nothing else; Escape too; the wheel scrolls it.
- **Never waiting on the chain.** The drawer reads the chain with `try_status` and `try_active`
  (new in `ca72-fx`: none while the chain is busy, the last reading kept), every tenth of a second
  or as soon as something done there finishes. What loads or opens a window runs on a thread of
  the drawer's, in order; a bypass on another, so a tab answers while an effect loads. And
  `Chain::insert` no longer holds the chain while the plug-in loads (seconds for some): it checks
  the place, loads, then checks again (and loads again if the device changed the rate meanwhile).
  The scan runs on a thread of its own the first time the list opens, its results kept for the
  application's life; again only from SCAN AGAIN. The `fx_view::Effects` trait is the drawer's
  reach into the chain (the standalone's `fx::Host`; the tests' own), so the editor's tests run
  without helpers.
- **Two fixes found in the real standalone.** With the keyboard page showing, a right click or
  the wheel on a drawer from the rail (either) went to the keyboard under it; and a menu over the
  keyboard did not light the item under the pointer. Both answer the drawer and the menu now.
- `Chain::params` (the effects' parameter lists) is not used: FX-A shows no effect's parameters.

**Evidence (Linux, omarchy, 2026-10-18):**
- Panel: `strip::fx`'s `the_drawer_finds_each_part_where_it_is_drawn` (screens, tabs, list rows,
  room for each tab, within the drawer and the window), `drawn_in_parts_it_is_drawn_whole` (a tab,
  a crash, the pointer, a carry, the list opened, scrolled, pointed at and put away; both skins,
  0.44 and 0.87) and the arpeggiator's tests as before. Editor:
  `the_fx_key_drops_its_drawer_one_drawer_at_a_time` (the rails, FX and ARP each putting the
  other away, the frame put together whole, the key's light, PANEL) and
  `the_fx_drawer_operates_the_chain` (window, bypass, a drag that moves, the list and a row
  added, REPLACE… and REMOVE, the wheel, a press elsewhere, nothing under the drawer; the keyboard
  shown); `fx_view`'s tests of the slots' states and the list. `cargo test -p ca72-panel`,
  `-p ca72-plugin --features standalone-production --lib`, `-p ca72-fx`, workspace clippy
  `-D warnings` with all targets.
- The real standalone (release, `--no-devices`, `IDLE_FOUNDRY_SHARED` and `--state-dir` in the
  worktree, `CA72_FX_PATH` the DAW-v3 test plug-ins and Dragonfly Reverb 3.2.10's CLAP and VST3,
  `CA72_FX_STANDARD_FOLDERS=0`), driven by synthetic X events: the key, SCANNING then ten
  effects, three added, a bypass, a carry and a move, two effects' windows opened (a CLAP and a
  VST3), REMOVE, REPLACE…, a helper killed six times (CRASHED), closed and opened again with the
  chain as it was (bypass, order, states) and with a plug-in moved away (MISSING); ARP and FX
  putting each other away.
- Frame times (`cargo run --release -p ca72-panel --example frames`, before (the merge) and after
  in turn, each from its own target; full log on the branch's summary): the panel's, the strip's
  and the arpeggiator's cases as before within the noise, with either rail (0.87 of the drawing:
  CUTOFF 2.7 and 2.7 ms median, DRIVE 6.3 and 6.2, a tab 1.8 and 1.9; the frames example's opening
  372, 915 and 769 ms before, 372, 904 and 782 after, at 0.44, 0.87 and 1.0). The effects' drawer
  at 0.44, 0.87 and 1.0: its renderer made as the editor opens 0.02 ms (its still parts, the
  list's plate among them, on its thread: 83, 128 and 144 ms); its first drop once they are made
  2.2, 5.1 and 6.2 ms; a tab 0.1, 0.3 and 0.5 ms; a carry 0.3, 0.7 and 0.8; the list dropped 2.8,
  7.9 and 10.3 ms, pointed at 2.1, 5.2 and 6.6, scrolled 2.2, 5.3 and 6.7; open and still,
  nothing.

**Not done:** an audio device (the standalone ran with `--no-devices`, so no effect was heard
and the chain's audio path was not run from the drawer; the engine's own tests and runs cover
it, R-FX-HOST); LOADING and RESTARTING seen on screen (the test effects loaded and restarted
within the screenshots' tenth of a second; both in the tests); the Mac and Windows; a scan of
the standard folders on a computer with many plug-ins; Escape with the list open in the real
window (the keyboard is not asked for while the list shows).

## R-FX-MAC. Effects heard, and their windows opened again, on the Mac (issue 82)

**The owner's report, 2026-10-19** (the CA-72 TEST standalone on the Mac, 47 effects found):
"the plugins themselves will load, which is good. I can click add an effect. I can scan, I can
search through my plugins on my computer, which is fantastic. And I can load up the plugin. The
plugin window pops up and I can change things. However, there is no audible effect. So something
is wrong, even when it's clearly on... The sound still goes through, but it's like it's in bypass
mode. And then the other issue is once I've closed the plugin window, clicking on the slot should
open it up again, and it doesn't..."

**Causes and fixes:**
- **No effect heard.** The synth runs the chain only if `Runner::is_empty` says it has effects,
  and the runner learnt of the chain's changes (an effect added, removed, moved) only inside
  `Runner::process`. An effect added to an empty chain therefore waited on its queue for a block
  that never came: it loaded, its window opened and it read as running, with no misses, while
  every block went out as the synth made it. A chain restored at start was heard (`prepare`
  takes the changes), which is how the Linux runs played. Not the deadline nor macOS's waits:
  the third-party effects played paced at 48 kHz in 256-frame blocks with a deadline of 0.9 of
  the period, none missed. `is_empty` now takes the changes first (no allocation, no locks).
- **A window not opened again.** The click reached the drawer and the helper each time, and the
  helper opened a new window, but behind the CA-72's: the click had just made the CA-72 the
  active application, and the helper, an accessory application that was not active, ordered its
  window front with `makeKeyAndOrderFront:`, which leaves an inactive application's window behind
  the active one's. The same for an open window asked for again. The helper now also sends
  `orderFrontRegardless` (third_party/daw-v3/PATCHES.md, change 3). The window comes up in front
  without taking the keyboard: the player's first click on it gives it the keyboard.

**Evidence (2026-10-19):**
- omarchy: `ca72-fx-helper`'s new `an_effect_added_while_the_synth_plays_is_heard_at_once` (the
  chain run as the synth runs it, real time at 48 kHz and 256 frames, each block due by 0.9 of
  its period, an effect added while it plays; both formats): fails without the fix (never wet),
  passes with it (wet from the block it was added in, none dry or missed after). The editor's
  `a_slots_name_clicked_again_opens_its_window_again`; the helper's GUI test opens, closes and
  opens again, and asks again for an open window (`CA72_FX_GUI_TEST=1`).
- Mac (macOS 27, the owner's ValhallaRoom, kHs Distortion and kHs Gain VST3): the new
  `third_party_effects_are_heard_in_real_time` (`CA72_FX_THIRD_PARTY`) played each paced at
  48 kHz, 256 frames: none missed, worst block 0.7 to 1.9 ms (kHs Gain at 0 dB comes out as it
  went in). The real standalone, a build of this branch with a diagnostic measure of its output
  in the audio callback, on a virtual output device at 44.1 kHz and 256 frames (the Mac's own
  device and rate untouched), a note from a virtual MIDI source, ValhallaRoom added from the
  console while it played: before the fix the output was the synth's to the bit (the effect at
  frame 0, never given a block); after it, the output differed from the synth by -26 to -35 dBFS
  (the synth at -30.4 dBFS), every block through the chain, none missed, and equal to it again
  with the effect bypassed. The window, with clicks posted to the standalone and the screen's
  window order read: opened, closed with its close button, opened again (behind the CA-72's
  before the fix, in front after it); covered by the CA-72's and asked for again (in front).

**Not done:** Windows and Linux windows (unchanged; a helper's window on Windows may meet the same
rule, untried); a keyboard for the effect's window without a click.

## R-MCP. Agents operate the standalone through MCP (#76)

**Owner decisions, 2026-10-18.** Target 0.3.0. The first version controls the standalone
application. Plug-in instances in a host come once the design has settled, so instance
discovery and selection are designed in now. It covers "all user-facing controls and
functions": presets (list, load, save, rename, favourites), the panel and strip, the settings
menu, the drawers, notes (for #77), MIDI Learn and QUALITY. Reuse the DAW-v3 MCP server's
patterns.

**Agent proposal, 2026-10-18** (placeholder id; not yet reviewed by the owner). docs/mcp.md has
the inventory, the design, the tools and what comes next.

- **Two halves.** The bridge `ca72-mcp` speaks MCP on stdio and is what an agent's client
  starts. The standalone listens on a Unix socket in the user's runtime folder while agents
  are allowed. The bridge finds the CA-72s that are open from their records and passes each
  call to the chosen one. The MCP layer takes DAW-v3's shape (`daw-api/src/mcp.rs`, DESIGN
  7.6): revisions; `tools/list` with schemas generated from the API's methods (D165) and, for
  `ca72_set`, from the parameters; a failed call returned with `isError`. The bridge connecting
  to a long-running process copies `daw rpc`'s proxy to `daw serve`.
- **The editor's own paths.** The socket's threads hand each request to an inbox. The editor
  takes from it at each frame, on its thread, and carries the request out as its controls
  would:
  - parameters through `ParamSetter`, a gesture each;
  - DETUNE and VOICES refused where they're dimmed;
  - MODE as the strip's tabs;
  - presets through `Browser` (SAVE AS and RENAME now share `save_sound` and `rename_preset`)
    and the drawer's `Library`, with the drawer's hint;
  - settings as the menu changes them;
  - notes through `LocalNotes`, owned as the on-screen keyboard's (`Playing::agent`).

  AUTO GAIN's curve is asked for after an agent's change, as after the player's. Nothing new
  touches the audio thread.
- **Values in the panel's units, range checked.** Knobs take their dial's numbers. FREQUENCY
  takes semitones. The strip takes %, cents and dB. A selector takes its position by name, id
  or index; a switch takes on or off. A value outside its range is refused, not clamped, with
  the range given, and nothing in that call is set. The panel's typed values clamp; an agent
  learns that it asked for too much.
- **Access (owner's call).**
  - Off by default: SETTINGS has a new AGENTS group, ALLOW and OFF (`agents` in
    `settings.toml`), shown only in the standalone. `--agents` allows agents for one session.
  - Until then no socket exists. The folder (700) and the socket (600) are the user's alone,
    and there is no network port.
  - An agent can turn agents off, never on.
  - No per-connection prompt or token. Recommendation: the setting and the user-only socket
    are enough for a local instrument.
- **Lifetime.**
  - A request the window hasn't taken within 10 s is answered as timed out and never carried
    out. One whose client has gone before the window takes it is dropped. One already taken is
    carried out, and its answer is discarded.
  - At most eight clients connect at once, and the editor carries out at most 32 requests a
    frame.
  - With no instance chosen, the bridge reconnects to the one open after a restart.
- **Left out.**
  - Visual-only things and indicators.
  - TYPING: `ca72_note_on` covers it.
  - Opening the browser for a download.
  - Changing AUDIO & MIDI's devices: read only for now, since a change can stop the sound
    (**owner's call**: offer it after a confirmation design).
  - Rendering and listening, which are #77's to design.
- **Next, for plug-ins in a host** (docs/mcp.md):
  - a server that works while the editor is closed, setting parameters through the
    process context as MIDI Learn does;
  - each instance identified by its host and track;
  - a named pipe on Windows;
  - `ca72-mcp` in the installers.

**Evidence (omarchy, Linux x86-64, Rust 1.97, 2026-10-18).**

- **The agent tests.** `cargo test -p ca72-plugin --lib agent` runs 23 tests, with and
  without the standalone feature, and all pass.
  - 17 test the protocol's parts: the MCP dispatcher, the tools' schemas, the values,
    discovery, the server (order, errors, a client gone mid-request, eight clients, stopping)
    and the bridge (none open, one, two, choosing, reconnecting).
  - 5 test the editor's side. Four of them drive a real `Editing` behind a real socket,
    through the bridge's code in MCP's text. The host applies the changes as the standalone's
    does. They cover parameters, presets, notes, settings, the view, MIDI Learn, agents turned
    off, and a client gone mid-request.
  - 1 tests the settings menu.
- **The built bridge.** `tests/mcp_bridge.rs` runs the built `ca72-mcp`, including its process
  killed mid-request: the request was dropped, not carried out, and the next bridge was served.
- **The whole crate.** The lib suite passes with the standalone feature (279). Without it, 269
  pass, but `the_workers_follow_poly_through_the_plug_ins_life` timed out once at a load
  average of 17 and passed alone.
- **Clippy.** Clean with `-D warnings` for Linux, with and without the standalone feature, and
  for `x86_64-pc-windows-gnu`.
- **End to end.** The real `ca72-standalone --no-devices --agents`, its window on a hidden
  Hyprland workspace, was driven through the built `ca72-mcp` by a JSON-RPC client. 21 checks
  passed:
  - the handshake, 28 tools, the instance found and `describe`'s 63 parameters;
  - setting and an out-of-range value refused;
  - a preset saved, marked changed, loaded back and deleted;
  - notes played and let go, the look changed, the drawer opened, AUDIO & MIDI read;
  - 20 round trips in 406 ms.

  Ten raw clients that sent a request and closed at once, and a bridge killed mid-request,
  left the application serving. Agents turned off removed the socket and its record, and the
  next call was told that none listen.
- **A real MCP client.** Claude Code (`claude -p --mcp-config <scratch file>
  --strict-mcp-config`, no global configuration changed) read the version and MODE, set
  CUTOFF and EMPHASIS, reported the out-of-range refusal word for word, played two notes and
  read the values back.

**Not tried.**

- macOS (it should work, since it also uses Unix sockets), and Windows (not built).
- A real audio device with notes heard.
- The AGENTS menu in the running application. The menu itself was drawn in a test
  (`CA72_AGENTS_PNG`); in the application it sits below STANDALONE's AUDIO & MIDI….
- Hosts and plug-ins.
