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

The reproducible [private Linux fixture](https://github.com/idlefoundry/plugin-kit/blob/f6bc24e16015e1536f571c8f8b2dcda6823b9309/scripts/standalone-virtual/verify.py)
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
Strict all-feature Clippy and formatting pass. The final native CI and candidate
checks are recorded below.

## Independent review and packaging

Review corrected source/channel reset ownership, first-note source identity,
final-save error propagation, moved/replaced bad-file protection, buffer-range
format selection, stale capture startup, preempted queue publication, CoreAudio
callback retention/output-only probing and unchanged-identity server recovery.
Final app review additionally reproduced typed-field partial restoration,
shifted-semicolon stuck notes and double-panic ordering. Regression tests cover
all corrected behavior.

Final independent re-review confirmed the original wrong-field and shifted-key
reproductions now pass, with a rejected session remaining byte-identical. Seven
adapter tests, same/split-boundary paired PANIC and malformed-session close
preservation pass. No remaining confirmed issue was found in those reviewed paths.

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

The final Windows installer was compiled and exercised with silent install and
uninstall using fake payloads, a unique application identity, lowest privilege
and all component paths under a private temporary directory. A standalone-only
installation preserves an existing unselected VST3 bundle on uninstall. A selected
VST3 installation replaces its stale bundle and removes the selected bundle on
uninstall. No product installation or owner's plug-in path was changed.

The Linux shipped-profile application builds, `--help` exits successfully, and
unknown/incomplete options exit 2 without accessing a display or device. Its native
library dependencies include ALSA and X11/OpenGL, with no JACK dependency.

CI run [38508250169](https://github.com/idlefoundry/ca-72/actions/runs/38508250169)
built the Linux application and installer from `e94218b`. The downloaded archive
was installed, launched with `--help` and uninstalled under a private prefix.
Its notices match the committed source, every native library resolves, no JACK
library is linked, and its maximum GLIBC symbol version is 2.35. Archive SHA-256:
`ef418002a6c96135890dd351fde624f0753fa9a640140566599e11bbb615fc4b`.

That run compiled both macOS application slices, then exposed a `lipo` argument
order/parser difference. Both build and package scripts now verify each slice
separately with the input first. Apple clang 21 built a private universal C
fixture; the native Apple tool verified arm64 and x86_64 with those exact calls.
The corrected universal application/package checks passed in the final CI run below.

The Windows artifact from that exact run is a GUI x64 executable. Its imports are
Windows system DLLs only, with no VCRUNTIME or MSVCP redistributable dependency.
Binary SHA-256: `8aebe8e122f6abfe0599b6f4669541a2035354522c9573e32e76094e76484265`.
Redirected help/invalid-option CLI checks pass. Two no-device application cycles
in a private SSH session close via their own WM_CLOSE in 142.73/143.83 ms with
exit 0, save session.json and restore unsaved cutoff, preset name and editor width.
The existing per-computer settings file is unchanged. No owner window receives
focus and no physical endpoint is opened. The test fixture is removed afterward.

## Final candidate acceptance

[CI 38510356796](https://github.com/idlefoundry/ca-72/actions/runs/38510356796)
passed macOS, Windows and Linux on `acaa03c`. Its merge `c34cb3c` has exactly the
same tree. Builds, all-feature Clippy, workspace and 17 derive tests, installers
and platform validators pass. The [candidate summary](native-candidate-summary.json)
records exact source/artifacts and hashes.

The final Windows executable again passes native no-device start/save/reopen/close,
with exits 0 in 108.11/151.86 ms and shared preferences unchanged. The real portable
ZIP contains the actual CI app/icon/plug-ins/licences; all file and archive hashes
pass. The final Mac app and installer payload are universal arm64/x86_64 with
Mach-O and plist deployment 11.0, correct icon/executable and microphone purpose.
The optional installer choice targets /Applications. Payload CLI exits 0/2/2
without GUI, state writes or hardware. No Homebrew or /usr/local library is linked.

The raw CI app bundle is unsigned; its executable has only the linker's ad hoc
signature. The CI full installer is unsigned, with an ad hoc sealed app carrying
the audio-input entitlement. Those artifacts are not public release installers.
A separate private standalone-only candidate from the exact verified app was signed
with Developer ID Application, hardened runtime, secure timestamp and audio-input
entitlement; its expanded installer payload verifies across both architectures.
Developer ID Installer signing and trusted timestamp pass. Apple notarization
returned Accepted; both app and package are stapled and validated, and Gatekeeper
accepts both as Notarized Developer ID. Its resources/notices match the original
CI payload. Existing credentials worked through a fresh owned desktop session;
no Keychain was unlocked or changed and the session is closed. Nothing was installed
or published. This verifies the new application's signing capability, not a signed
combined plug-in release.

S1–S7 implementation/candidate work is complete within the recorded software and
virtual/native evidence. S8 remains PARTIAL: physical playback/MIDI/external-input
latency, permission grant/denial, hardware hotplug, the macOS application's GUI
lifecycle, Intel/macOS 11 and Windows 10 runtime remain unverified. A broken native
driver call can still outlive the bounded caller shutdown. The [implementation contract](../standalone-spec.md#implementation-order-and-evidence-gates)
requires a named physical interface before opening it; none was supplied.
