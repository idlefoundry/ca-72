# Linux CLAP envelope investigation, 18 October 2026

Issue [57](https://github.com/idlefoundry/ca-72/issues/57), based on `1a1dd6c` (0.2.6).
The issue describes the published 0.2.5 report without establishing host/version, patch,
MIDI or mode. `gh issue view 57 --repo idlefoundry/ca-72 --json title,body,comments,url`
was read. Both linked KVR pages were inaccessible through the web reader and returned
HTTP 403 to a direct fetch; searches for the post ids returned no text. No forum wording
beyond the issue's summary was recovered.

## Reproduced and corrected

A native `CLAP_EVENT_NOTE_ON` with velocity 0 was converted into a release in `event_of`.
CLAP explicitly permits this attack (`third_party/clap/include/clap/events.h:55`).
nih-plug's MIDI decoder already converts MIDI 1 note-on bytes with velocity 0 into
`NoteOff`, so the additional velocity test was wrong for typed notes. VST3 also passes
its native note-on velocity through unchanged. The production change accepts every
framework `NoteOn` as an attack. MIDI 1 velocity-zero releases keep their meaning.

The native CLAP regression failed before the correction with sustained RMS exactly 0.
It now checks an audible held note and its subsequent MIDI velocity-zero release for
all 18 mode/priority/retrigger combinations at 256 frames. The existing keyboard-source
regression now constructs its MIDI zero-velocity release with the actual MIDI decoder.
It had incorrectly constructed a typed zero-velocity `NoteOn` and expected a release.

This is a definite event-conversion defect, but there is no evidence that the reporter
sent zero-velocity native notes. Do not close issue 57 on this evidence alone.

## Headless coverage

`crates/ca72-plugin/tests/clap_host.rs` calls CA-72's real CLAP entry point, without an editor:

- MONO, POLY and UNISON; two voices; LOW, LAST and HIGH; RETRIGGER off/on.
- Buffers 32, 64, 128, 256, 512, 1024 and 2048 at 48 kHz, QUALITY LO, offline rendering.
- Separated notes; a repeated pitch after 5 ms; different overlapping pitches; releases
  in later blocks; note-off/on at one sample in both orders, for different and equal pitches.
- CC64 down/up around notes, transport start/stop while a note is held, CC123 and CC120,
  and a fresh note after the resets.
- 126 configurations, each rendered through MIDI bytes and native CLAP notes. The dialects
  must produce identical samples within each configuration. Fourteen sounding windows
  must exceed RMS 0.03. Release windows must fall below 30 % of their held levels. Across buffer sizes the sounding windows must stay within 3 dB.
  POLY voice retirement occurs at block boundaries, so oscillator phase need not be identical
  across buffer sizes. Equal-pitch on-then-off at one sample intentionally leaves no held note.

`engine::tests::separated_and_overlapping_keys_start_and_hold_both_contours` additionally
checks the filter and loudness contour voltages at LO, HI and ULTRA in all 18 combinations
(54 configurations). It checks every held instrument and both overlapping POLY keys,
that releasing the overlapping key preserves the first, and that ordinary releases and
All Notes Off lower both contours. The circuit's released voltages need not be zero.

The existing `ca72/tests/retrigger.rs` documents the circuit's approximately 12 ms re-arming
interval, with `RETRIGGER_GAP` 13 ms. A shorter gap without RETRIGGER can carry on a contour;
ordinary mono legato without RETRIGGER also carries it on. Neither is proof of issue 57.
CC64 currently has no built-in pedal function; it is available to MIDI Learn. The matrix
checks that its unassigned messages do not disrupt subsequent notes. Adding a pedal would
be a separate feature and compatibility decision.

## Real Linux hosts

REAPER 7.79/linux-x86_64 loaded a separate `CA-72 ENV57` 0.0.57 build with distinct CLAP and
VST3 ids. Both formats' module identities were checked. The libraries lived under
`~/.cache/ca72-release/linux-plugins`; the separate REAPER profile, shared settings,
projects, scripts, MIDI and renders lived in this worktree's `target/env-host`.
The owner's plug-in folders and settings were not changed.

A raw MIDI item preserves the two event orders, overlaps, rapid repeats, pedal messages,
CC120/123 and the final fresh note. REAPER rendered 18 combinations in each format,
48 kHz/24-bit stereo, QUALITY LO: 36 files, each 2.4 s. All 504 checked sounding windows
exceeded RMS 0.102; final residual RMS was 0.0046 to 0.0065. A diagnostic copy logged
REAPER's note events: both wrappers delivered matching typed MIDI notes, including their
releases. The diagnostic print is absent from the production change. REAPER quantised
MIDI to its 960-PPQ item grid; the 5 ms repeat gap became about 5.21 ms.

The first disposable item looped its short MIDI source into the intended release tail.
Those renders were discarded from the evidence: the final item has source looping disabled
and an explicit terminal CC123 at 2.4 s. The final projects and analysis are retained.

Bitwig Studio 5.2.7 loaded the renamed CLAP successfully in a filesystem namespace whose
home directory and plug-in folder were under `target/env-host/bitwig-home`. `/proc` module
maps and Bitwig's engine log confirmed the test instance. A fresh profile first crashed
in graphics initialisation (Vulkan/NVIDIA, before loading CA-72); copying the machine's
existing `skia-gl` setting into the disposable profile allowed startup. Its native file
chooser then failed in GTK's icon loader. The profile was in Demo Mode, which explicitly
disables saving and exporting. MIDI-item playback and an envelope render in Bitwig are
**not verified**. No account was signed in. All test host processes were closed.

## Comparison and remaining bounds

CLAP's native note path preserves velocity 0. Its MIDI path, both standalone backends,
and the production standalone adapter use `NoteEvent::from_midi`, which maps MIDI 1 zero
to `NoteOff`. VST3's native `kNoteOnEvent` preserves velocity. They all reach `event_of`.
Transport state is not itself a note release: the host must send notes off, CC123/120 or
reset the processor. No adapter-specific envelope state machine was found.

Not covered: the reporter's patch, live controller or host; all factory presets; loaded
VST3 native velocity-zero notes; a live standalone session; native CLAP note-id stacking
or wildcard note-off/choke addresses; live audio deadlines under CPU overload; macOS or
Windows. The source still ignores note ids and the framework does not represent wildcard
key/channel addresses explicitly. Capture those fields if the reporter's failure persists.

## Verification

With `CARGO_TARGET_DIR=$PWD/target CARGO_BUILD_JOBS=4`, plus
`RUST_TEST_THREADS=4`, `IDLE_FOUNDRY_SHARED=$PWD/target/env-evidence/shared` and
`CA72_SEATS_DIR=$PWD/target/env-evidence/seats` for the final runs:

- `cargo test -p ca72-plugin`: 294 passed, 31 ignored diagnostics, no failures.
- `cargo test -p ca72-plugin --test clap_host`: 6 passed after the final test-only
  strengthening, including the two envelope regressions.
- `cargo test -p ca72-plugin --features standalone-production standalone_`: 11 passed.
- `cargo clippy -p ca72-plugin --all-targets -- -D warnings`: passed.
- `cargo clippy -p ca72-plugin --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `python3 target/env-host/analyse.py`: all 36 REAPER renders pass the 14 sounding-window
  and final-release checks. `target/env-host/render-analysis.json` holds the measurements.

The full workspace suite was not run: other overnight jobs were already running it, and
this change touches only `ca72-plugin`. No Mac or Windows host or release validator was run.
Early validation attempts used a too-early release window or assumed a released contour
voltage was zero; the measured fall and offset corrected those test assumptions. An early
unisolated worker-count test failed while other jobs were running; private-seat runs pass. Reusing the main target for the renamed export caused a
crate-version conflict; cleaning those local package artifacts resolved it. These are
harness corrections, not further product defects. The renamed build's cached libraries
were moved to the Trash after the host processes closed. The remaining evidence is inside
the worktree's `target/env-evidence` and `target/env-host`.

## Draft reply for issue 57 (not posted)

I found and corrected one missing-note case: a native CLAP note-on at velocity zero was
being treated as a release. MIDI note-on bytes at velocity zero still release notes as
required. I have not established whether this is the case you encountered.

The headless tests cover separated and overlapping notes, rapid repeats, equal-time
note-on/off ordering, pedal messages, transport and all-notes-off, with MONO/POLY/UNISON,
every priority, RETRIGGER on/off and buffers from 32 to 2048. Linux REAPER renders passed
in CLAP and VST3. Bitwig loaded the test plug-in, but its playback/render check is incomplete.

Please attach a small MIDI clip and the CA-72 preset or host project that reproduces the
failure, and give the host/version, CA-72 version, sample rate, buffer size, QUALITY,
MONO/POLY/UNISON, VOICES, NOTE PRIORITY and RETRIGGER settings. Does it occur on a fresh
instance, after transport stop/start, or only with a sustain pedal? A short expected/actual
render and, if available, an event trace with timestamps, velocities, note ids and off/choke
addresses would distinguish the remaining cases. A separated-note failure is not explained
by ordinary mono legato alone, so issue 15 is not being treated as its resolution.
