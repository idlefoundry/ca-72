# Standalone keyboard proof

Integration into current main is tracked in [standalone-integration.md](../standalone-integration.md). Earlier preview renders and test results below apply
to their recorded source; they do not validate the integrated candidate.


The lower-right VOICES / STEREO / OUTPUT area swaps with a playable 44-key F2–C6
keyboard using the piano button in the preset/settings rail. The panel and window
size remain unchanged when swapping views. The separate PANEL button keeps the
existing whole-panel show/hide action; hiding it releases pointer notes while
standalone TYPING and physical MIDI continue. Main synthesis controls, pitch/modulation wheels and
left-hand controls retain their positions. Octave buttons reach all MIDI pitches.

- `native-full.png`: neutral Realistic skin, 1380-pixel full window.
- `native-closeup.png`: neutral Realistic keybed and rail at 87% panel scale.
- `mock.html`: original interactive layout study using the generated key sprites;
  it does not produce audio. Current native header, pressed-key feedback and render
  instructions are recorded in [keyboard-feedback](../keyboard-feedback/README.md).

Natural keys have L/T silhouettes with real cutouts around their neighbouring
black keys; the empty recess does not trigger a note. Tops and fronts stay aligned
in the neutral and held states. The final sprites' provenance, study references,
source dimensions and crops are in `crates/ca72-panel/assets/keyboard/README.md`.

Hosted editors initially show Controls; standalone initially shows Keyboard unless
a local view preference has been saved. View and octave are local settings, while
TYPING always starts off in standalone. TYPING and PANIC are absent and inactive
in hosted plug-ins, which leave note typing to the DAW. Visible range and TYPING octave are separate preferences,
absent from preset and DAW state. ASDFG plays CDEFG from TYPING's own C root
(A defaults to MIDI 48); Z/X lowers/raises it by an octave independently of the
visible range and oscillator RANGE controls.
Computer-note releases follow the recorded physical key even when Shift changes
its character; forwarded host releases normalize colon back to semicolon.
Pointer and computer-key owners are tracked separately for their shared local
audio source; physical MIDI has separate ownership by channel and pitch. TYPING
continues across controls and compact views. Hiding keys or changing the visible
range releases only pointer notes; Z/X releases only typed notes. Focus loss,
closing, preset or text takeover, MIDI Learn and settings entry release local
notes, preserving physical MIDI notes. Host reset and global CC120/123 clear the
source registry and invalidate queued local attacks. PANIC additionally clears
hardware-source ownership in production standalone. The UI and hardware panic
share a boundary reset; fresh local notes wait for that boundary and survive it
even when the two requests arrive in successive callbacks.

The shared packed-u64 queue holds 256 local messages. Its producer and consumer
make one attempt and never retry or wait. Overflow requests local release through
an independent generation counter. Audio drains at most 256 events at the next
block boundary; host MIDI sample offsets retain their existing timing. The editor
receives held-note highlights from audio-owned source state.

Focused verification on Linux:

- Panel geometry: every natural key shape, black-key precedence, recess hit tests,
  aligned fronts, all MIDI pitches reachable.
- Partial-render comparisons: both skins at 44% and 87%, held notes, hover, octave,
  TYPING and view changes, with pixels outside the keyboard area unchanged.
- Editor interaction: hidden-control isolation, pointer release outside the window,
  editor drop, preset/settings/text takeover and host key forwarding.
- Source/queue: channel and local ownership overlap, overflow, reset, panic,
  velocity-zero releases, global CC120/123, complete concurrent packets, queue wrap,
  and stale producer publication after release.
- Local audio matches host MIDI samples at equivalent block boundaries with the
  same saved noise seed. Focused plugin tests and strict Clippy passed; the
  production standalone library also compiled with its optional feature.

Native pointer events use the editor's existing mouse input backend. Touch input
that the OS maps to mouse input follows that path; independent multitouch voices
are outside this milestone. Windows no-device GUI save/reopen/close and macOS package/CLI checks now pass.
The macOS GUI, physical-device and minimum-OS runtime checks remain unverified.
