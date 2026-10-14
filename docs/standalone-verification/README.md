# Standalone acceptance evidence

Research began 2026-10-13; integration and verification continued into 2026-10-14.
The contract is [standalone-spec.md](../standalone-spec.md).

## Design and compatibility

The owner explicitly delegated visual judgment: “Do the best you can with the
keyboard, but don't get stuck on it or wait for me to approve it. Just keep moving
forward with the rest of the items on the list.” The [final proof and interactive
mock](../standalone-keyboard/README.md) use 44 keys, true L/T white-key cutouts,
narrow dark clearances and aligned tops/fronts. Held notes never displace a key.
Photographs were inspected, never supplied to the generator or copied into the
product. Original built-in-generated surfaces, their GPL provenance and exact
prompts are in the [asset record](../../crates/ca72-panel/assets/keyboard/README.md).

R70's existing PANEL hide/show remains beside the keyboard button. Hiding the
panel releases local notes; keyboard/controls swaps retain its height. View and
octave use the existing per-computer settings.toml, shared with hosted editors,
outside presets and DAW state. This is an explicit departure from putting those
preferences in session.json. TYPE always starts disabled. --state-dir separates
instrument/I/O session state; the library and computer preferences remain shared.

Physical channels, pointer and typing retain separate ownership. Focus/UI takeover
releases local notes only. CA-72 retains its existing global CC120/123 semantics.
Physical key identity fixes release after modifier changes; unidentified host keys
normalize shifted punctuation. Paired UI/core panic is consumed once at the audio
boundary, preserving fresh local notes across either callback ordering.

Typed-field preflight follows migrations and precedes every state setter. Wrong
types cannot partially restore a session. Existing documented MIDI table and
calibration repairs remain compatible with hosted loading. Audio tasks use
preallocated SPSC queues with Copy payloads; callback queue exhaustion cannot run
an allocating task destructor. The sole task executor is shared on control threads.

## Native backend evidence

The reproducible [private Linux fixture](https://github.com/idlefoundry/plugin-kit/blob/e93e7e6bdadedcc61b4bf7a358fc0a0c23ce8457/scripts/standalone-virtual/verify.py)
creates only its own socket, dummy driver, null sink/source and streams. No session
manager, udev, physical nodes or owner's configuration is involved. It verifies
stereo playback/capture at 48 kHz/128 requested frames, 44.1 kHz/256 and 48 kHz/256,
automatic recovery after terminating/restarting that server, preservation of one
processor and exactly one Drop at close (1 ms). All 13,404 nonzero captured samples
match left 0.125/right -0.25; none are invalid. Observed periods are 64 frames at
48 kHz and 117/118 at 44.1 kHz, so the UI reports measured callbacks.

The [CoreAudio probe](coreaudio_probe.rs) opens only an explicitly named, verified
virtual output, produces silence and uses its existing rate/default period. It
never opens input or falls back to a physical/default device. Twelve cycles on a
duplex virtual endpoint at 48 kHz and twelve on an output-only endpoint at 44.1 kHz
each delivered 48 callbacks. No callback owners survived Drop and no callbacks
occurred afterward. Worst stream Drop: 32.55/36.46 ms. These runs exercise the
upstream CoreAudio lifetime and output-probing backports. They do not prove
microphone permission, physical hotplug or concurrent disconnection-listener teardown.

All 29 host tests plus 2 queue tests pass natively on macOS and Windows. Windows
testing reproduced competing atomic replacements failing with AccessDenied;
control-thread serialization fixes it without removing the old file before
replacement. Actual WASAPI hardware callbacks remain unverified.

The shared kit has 102 passing workspace tests, plus 17 NIH derive tests. Final
application checks include 220 library tests and 7 adapter tests; 42 SPICE tests
pass. Direct and adapter processing are sample-identical with active stereo input,
alternating 128/256 frames and note/controller events. Allocation guards observe
zero callback allocations or frees; idle behavior retains the circuit's settling.
Strict all-feature Clippy and formatting pass. Native package CI follows below.

## Independent review and packaging

Review corrected source/channel reset ownership, first-note source identity,
final-save error propagation, moved/replaced bad-file protection, buffer-range
format selection, stale capture startup, preempted queue publication, CoreAudio
callback retention/output-only probing and unchanged-identity server recovery.
Final app review additionally reproduced typed-field partial restoration,
shifted-semicolon stuck notes and double-panic ordering. Regression tests cover
all corrected behavior.

Four Linux installer tests include real GIO launches of a harmless private fixture
in ordinary, space, percent, quote, dollar, backtick, backslash and equals-containing
prefixes. Two escaping layers and a static shell program with a positional path
avoid parser rejection and interpreting paths as code. Install/upgrade/uninstall
retain presets/session data and validate all selected payloads before changes.

CPAL 0.15 provides names/capabilities, not durable OS identifiers; ambiguous matches
fail closed. Independent capture clocks use a preallocated adaptive reservoir,
bounded to ±1 percent correction. Simulated 60-second ±500 ppm runs at 48 kHz and
128/256 frames have bounded fill and no under/overflow after priming. Equal clocks
copy exactly. Common nominal rates are required; input buffering adds latency and
output buffer duration is not total playing or input/output latency.

Native driver calls cannot safely be forcibly interrupted. Caller shutdown is
bounded; a broken OS call can leave a detached control thread until it returns.
Ordinary virtual-device close returned the sole processor and stopped callbacks.
Physical audio/MIDI latency, permission and hotplug remain UNVERIFIED. No physical
device was opened, no owner's installed plug-in was replaced, and no release was published.
