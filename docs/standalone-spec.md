# CA-72 standalone and playable keyboard

Written 2026-10-13. Baseline: CA-72 `e27681b` (0.2.0) and plugin-kit `adf8419`.
This is the implementation contract. Record discoveries and departures in
`docs/standalone-progress.md`; do not quietly weaken an acceptance criterion.

## Goal and authorized design

Ship a desktop application that plays the same CA-72 as the plug-in, using the same
engine, parameters, presets, MIDI Learn and editor. Users launch it normally, choose
audio and MIDI hardware in the editor, play, and return to their last working session.
The host services belong in plugin-kit so another instrument can adopt them.

The owner's keyboard design replaces VOICES, STEREO and OUTPUT with an onscreen
keyboard. A button beside SAVE and settings on the preview/preset bar swaps between
the keyboard and the extra controls. The instrument's main panel, left-hand wheels,
GLIDE and DECAY retain their positions. Switching views never changes a parameter,
the window's dimensions, a physical MIDI note, or the saved sound. Controls view is
the hosted plug-in's initial default; standalone initially shows the keyboard.

## Research and architectural decision

Research was completed before implementation. The existing NIH launcher is a GUI
test host: its CLI exposes CPAL 0.15, midir 0.9 and JACK, but has no device dialog,
live reconfiguration or session save. Its CPAL path supplies silence to auxiliary
inputs. The existing CLAP wrapper has useful native settings precedents, but has
stereo-only processing assumptions, a spinning audio lock, incomplete platform
settings, numeric device persistence, and unsafe multi-producer MIDI queues.

Use a reusable Rust host core (`plugin-kit-standalone`) and a thin NIH adapter.
Preserve the existing circuit and plug-in processing path. The reusable core owns
device identity/capabilities, negotiated stream configuration, bounded MIDI queues,
recovery, status, versioned settings and atomic session files. The adapter owns
NIH parameters, state and editor context. Prepare/reset/deactivate and file operations
run on control threads. Audio callbacks do only bounded processing and atomic/queue
operations. An editor can exist and modify its sound while the audio engine is stopped.
GUI settings submit commands and read snapshots; callbacks never invoke a dialog.

Primary sources checked:

- [CPAL API and supported configurations](https://docs.rs/cpal/0.15.3/cpal/): query
  device capabilities and still handle failure when opening the selected stream.
- [CPAL current platform baseline](https://github.com/RustAudio/cpal/blob/master/README.md):
  upgrading to current CoreAudio code raises macOS minimum to 14.2; keep 0.15 initially.
- [midir connections](https://docs.rs/midir/0.9.1/midir/struct.MidiInputConnection.html):
  connections own callbacks; enumeration and disconnect are control-thread work.
- [RtAudio device probing](https://caml.music.mcgill.ca/~gary/rtaudio/probe.html):
  names are not unique and enumeration IDs are not durable identities.
- [RtAudio duplex](https://caml.music.mcgill.ca/~gary/rtaudio/duplex.html): independent
  audio clocks drift; reliable arbitrary-device duplex needs clock adaptation.
- [Microsoft low-latency audio](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/low-latency-audio):
  requested period alone does not prove end-to-end latency.
- [JACK callbacks](https://jackaudio.org/api/group__ClientCallbacks.html) and
  [PipeWire JACK configuration](https://pipewire.pages.freedesktop.org/pipewire/page_man_pipewire-jack_conf_5.html):
  respect server rate/period, never force a global graph quantum.
- [Official ASIO GPL licence](https://github.com/audiosdk/asio/blob/main/LICENSE.txt):
  do not assume newer licence terms cover historical bundled SDK copies.
- [Apple microphone purpose string](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmicrophoneusagedescription),
  [audio-input entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.device.audio-input),
  [notarization](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution),
  [desktop entry specification](https://specifications.freedesktop.org/desktop-entry/latest-single/).

## Scope and platform contract

| Platform | Required first-release playback/MIDI | Packaging |
|---|---|---|
| macOS 11+, Intel and Apple silicon | CoreAudio and CoreMIDI | Universal `.app`, existing installer choice, hardened signing and input purpose/entitlement |
| Windows 10/11, x64 | WASAPI and Windows MIDI | Standalone `.exe`, existing installer choice and portable ZIP |
| Linux x64, glibc 2.35+, X11/XWayland | ALSA and ALSA MIDI, including session-server compatibility devices | Binary, icon, desktop entry, existing tarball installer |

The existing JACK example remains available. A production JACK backend with graph
port selection and server-controlled configuration is a separate support tier: claim
it only after the same acceptance matrix passes. Direct ASIO and native PipeWire are
extensions, not required for the baseline release; their licensing and OS-baseline
changes require deliberate integration. There is no sequencer, recorder or new DSP.

External audio is optional, disabled initially. Required reliable duplex is one
interface/OS aggregate or server-managed device at a common sample rate. Separate
hardware clocks are rejected with an explanation until drift correction is implemented.
Input permission denial must leave output-only playback available.

## User-visible behavior

**Audio & MIDI settings:** open from the existing settings bar/menu in standalone.
List available backend, output device, mono channel or stereo pair, optional external
input/channel selection, sample rate, requested buffer frames, selected MIDI inputs,
and channel filter (ALL or 1–16). Refresh is available. Unsupported choices cannot be
applied. Show actual rate, callback period and `buffer duration = 1000 × frames / rate`
in ms. Do not label buffer duration as total latency. Show stopped/starting/running/
recovering/error status and a concise actionable error. Apply and Cancel have explicit
behavior; Cancel keeps the running configuration.

Apply releases notes and suspends audio before preparing a candidate. Keep sound,
MIDI Learn, editor and view. On success activate and save the negotiated configuration.
On failure restore the last working configuration or show an editable stopped state.
The window stays responsive and open. Stream changes and initialization may briefly
interrupt sound; never reset the patch to defaults.

**Keyboard:** 49 keys, default MIDI 36–84 (C2–C6), ivory whites and charcoal blacks,
subtle amber held-note indication, C labels and an octave/range header. At the drawing's
scale replace x=370..3080 below the 116-unit preset rail; keep x<370 unchanged. Black
keys take precedence in hit testing. The bar button indicates which view is active.
Octave down/up clamps to MIDI 0–127 without wrapping. A TYPE switch opts into computer
keys; in hosted mode it defaults off and gives host shortcuts back when off. Map
`A W S E D F T G Y H U J K O L P ;` chromatically from the selected root; repeat keydown
does not retrigger. No velocity control: the circuit's keyboard is not velocity-sensitive.
Mouse press/drag/release plays through the same engine as external MIDI. One-pointer
touch through OS mouse emulation is supported; native multitouch is not claimed.

Keyboard visibility is a view preference, not a preset parameter. Existing knobs must
be completely hidden and unclickable under the keybed. MIDI Learn, preset search/name
typing, value editing and menus take precedence over note typing. Piano highlights
reflect audio-side held notes, including physical MIDI. PANIC clears all notes and
transient performance state without changing the sound's settings.

**Session:** restore unsaved parameter edits, chosen preset/name, MIDI Learn, editor
width, view/octave preference and last successful device configuration. Never restore
held notes, sustain, live bend/modulation or active gestures. Use a versioned app file,
separate from factory/user presets and hosted plug-in project state. Save atomically
off the callback; periodic saves are coalesced and close saves are flushed. A malformed
or unsupported file is preserved and a visible warning accompanies safe defaults.

**Hardware loss:** check device inventory at most every two seconds, plus callback
error notification. A missing selected device clears its notes, reports status and
leaves the editor usable. Reconnect only an unambiguous matching device; never silently
switch to a similarly named interface. Close completes independently of future callbacks.

## Definitions of success and tasks

Every task has an evidence entry in `docs/standalone-progress.md`. Mark it PASS only
when its acceptance criteria are demonstrated; BLOCKED/UNVERIFIED retain the exact gap.

| ID | Task | Required acceptance evidence |
|---|---|---|
| S1 | Research and frozen specification | Current code inspected; primary sources cited; known wrapper defects, platform contract and acceptance matrix recorded before implementation |
| S2 | Shared host core | Capability enumeration, unambiguous identity resolution, supported negotiation, selected channel routing, finite-input sanitization, mono `(L+R)/2`, zero unused channels, bounded queues, overflow recovery, atomic versioned persistence and deterministic fake-device tests |
| S3 | Production NIH adapter and app entry | Same plug-in/editor/params; open with no hardware; configuration prepared off callback; apply/rollback/recovery preserves sound; parameter edits work stopped and running; no callback locks, allocations, file/network or GUI calls; all threads terminate on close |
| S4 | Playable keyboard and view switch | Approved visual proof; fixed window and left-hand controls; both skins; hidden control hit-test isolation; mouse/glissando/outside release; typing chords/repeat; octave, view, focus, close and text takeover release local notes; source overlap tests; queue overflow cannot strand notes; audio-owned highlights and panic |
| S5 | Audio/MIDI settings UI | Every listed option reflects capabilities; input disabled default; ALL/1–16 filter; refresh; apply/cancel; actual rate/period/duration; errors/recovery shown; inaccessible device remains editable; settings available only to standalone host |
| S6 | Session and resilience | Restart restores unsaved edits, MIDI map/width/view/working I/O; corrupt/unsupported files retained; failed config not saved; missing/duplicate identities; hotplug notes released; close after lost callback bounded to two seconds; background operations have bounded shutdown |
| S7 | Packaging and documentation | Buildable standalone target; app/icon/metadata; standalone optional installer choice on all platforms; portable Windows ZIP; Linux install/uninstall and desktop validation; macOS universal signing/input configuration; notices and corresponding sources; README instructions and truthful support limitations |
| S8 | Independent review and verification | Reviewer findings resolved; focused tests and fmt/clippy; same-event/input plug-in-versus-app sample parity; no-active-note idle behavior; clean builds/package smoke on all OSes; actual playback, MIDI, external-input and disconnect evidence per OS; 128/256-frame timing observations |

### Exact behavioral test matrix

1. Notes: press/release, overlapping physical channels, physical+mouse+typing on the
   same pitch, velocity-zero note-on, sustain, CC120/121/123, focus loss, hidden view,
   octave shift, mouse release outside, repeated keydown, text/menu takeover and panic.
   Releasing one source never releases a pitch held by another. Host MIDI event offsets
   remain sample accurate. UI notes enter at the next block boundary (at most one
   callback period after enqueue); do not claim sample-accurate UI timestamps.
2. Routing: output 1/2/4/8-channel fake streams; every valid/invalid mono/pair mapping;
   external mono/stereo; silence when disabled/missing; NaN/Inf input becomes zero;
   every unselected channel remains zero. Input reaches the existing auxiliary bus.
3. Lifecycle: no devices, device busy, unsupported period/rate, disconnect before first
   callback, disconnect running, reconnect, failed apply with rollback, close while
   starting, close while recovering, failed state write and denied input permission.
4. Persistence: missing, valid, truncated, wrong version, unknown fields, unwriteable
   directory, competing writes and interrupted replacement. Never destroy a valid last
   session or the malformed original; never persist transient held-note state.
5. Realtime: allocation guard around adapter processing, queue exhaustion with emergency
   release, bounded processing under large MIDI bursts, same engine samples as direct
   plug-in processing. No busy real-time spin loops; no machine-wide audio settings changed.
6. Visual: controls/keyboard at normal 1380-pixel width and close-up, both skins, resizing,
   keyboard/controls view reversibility, drawer overlay, menus and damage repaint correctness.

## Implementation order and evidence gates

Research/spec first; shared core and keyboard work can proceed independently after
their interfaces are fixed. Show the keyboard layout for review while host work
continues. Integrate app/settings/session; then packaging; then focused and cross-OS
checks and independent review. After a visual change, show the rendered result and
allow the owner to review before milestone CI or merging, as circuit-plugin requires.
Packaging a release candidate is authorized; publishing a release remains a final
reviewable action, not an automatic consequence of completing code tasks.

Tests may use deterministic fake devices and private virtual endpoints. Before using
the owner's physical devices ask which interface is available, as circuit-plugin's
shared-machine rule requires. Hardware/driver tests without available devices stay
UNVERIFIED. Do not mark S8 or a public release complete merely because unit tests pass.
