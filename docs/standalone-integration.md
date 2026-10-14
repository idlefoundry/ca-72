# Standalone feature integration

Owner-authorized integration, 2026-10-14. Start from current main
`db681b8ccbfbaaa928d71e82aba641681fff7921`, the corrected plug-in-only 0.2.2.
Integrate the standalone preview as a new feature PR to main. Preserve the released
audio/resize fixes, package version and corrected release records. Do not publish
a release or tag. The withdrawn 0.2.1 tag, draft and assets remain untouched.

Source inventory: preview `8877dde`, settings/grid `c16e188`, font `95ba2e9`, and
the already incorporated keyboard-feedback branch `cf90cef`. Shared production
hosting uses plugin-kit `f7aaa2b0`, including cold-start readiness, tracked Apply receipts and exact PANIC request
identity. An older callback cannot acknowledge a newer UI request. The product contract remains
`standalone-spec.md`; this document defines integration acceptance.

## Definition of success

- One reviewable feature PR against current main, with focused source, settings,
  keyboard/UI, packaging and documentation commits. No old release tree restoration.
- Current main's DSP source, audio correction tests, host resize/refusal behavior,
  version and release records are preserved. New local notes coexist with physical
  MIDI without one source releasing another's note.
- macOS universal, Windows x64 and Linux x64 apps build and package. Each app's
  `--help`, invalid argument, no-device startup, isolated session persistence and
  clean shutdown are checked. Tests never open an unnamed hardware interface.
- Audio/MIDI configuration supports initial defaults, no devices, discovery errors,
  hotplug/reconnect, explicit disable, invalid configurations and tracked Apply
  success/failure. Unchanged/reverted drafts show Done and close without a restart;
  later edits survive an older completion. Remembered missing MIDI selections retain
  intent and retry. A new disconnected-entry presentation remains a design proposal.
- Pointer keys work in standalone and plug-ins. Standalone-only TYPING maps ASDFG to
  CDEFG, uses Z/X for its independent octave, survives controls/compact views, and
  yields to text entry. Hosted instances expose no TYPING/PANIC targets or typing
  note capture. Focus loss and panic release the intended sources.
- Both skins preserve the selected assets, straight wood frame meeting silver trim,
  three-natural/two-accidental icon, full-surface pressed keys, preset-style octave
  display, shared control grid, equal adjacent action buttons and active PANEL tint.
  Grey-button labels use URW Gothic 20 bold, in unchanged 62-unit caps. Native renders
  and partial/full repaint equality pass at 0.44 and 0.87 scales.
- Installer payloads, dependencies, architecture/minimum OS, application entry points,
  optional component selection and uninstall preservation are verified. Mac signing
  and notarization checks apply to a newly delivered signed candidate, never inferred
  from the earlier 0.2.1 preview package.
- Formatting, strict workspace Clippy with/without all features, touched-crate debug
  tests, typed persistence derives and installer/notices tests pass. Final exact-head
  CI passes on macOS/Windows/Linux, including unchanged required validators. Hosted
  plug-in regressions and independent final source/package reviews have no blockers.
  Native checks, unperformed physical MIDI/audio checks and any unavailable host
  checks are recorded separately and never represented as passing.

## Validation ledger

Pending: integrated-source checks, all three platform jobs, current candidate native
app/installer/host checks and independent final review. Earlier preview evidence is
precedent, not acceptance of the integrated source.

Known unresolved check: preview `c16e188` twice timed out in pluginval's nested AU
stress test. An exact-artifact probe passed direct strict stress-20, but the nested
30-second inactivity watchdog fired while samples showed active DSP work in its
child and the parent waiting for child stdout. The workload/state difference remains
unproven. No validator limit or strictness is relaxed. This integration does not
claim the issue fixed; its required validator result must be obtained independently.
No additional blind diagnostic retry is authorized by this integration plan.
