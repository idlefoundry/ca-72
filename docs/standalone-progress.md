# Standalone implementation evidence

2026-10-13. Contract: [standalone-spec.md](standalone-spec.md).

| Task | Status | Evidence |
|---|---|---|
| S1 Research/spec | PASS | Current CA-72/kit inspected; two independent research audits; primary documentation and platform constraints in the spec |
| S2 Shared host core | IN PROGRESS | |
| S3 NIH adapter/app | NOT STARTED | |
| S4 Keyboard | IN PROGRESS | |
| S5 Device UI | NOT STARTED | |
| S6 Session/recovery | NOT STARTED | |
| S7 Packaging/docs | NOT STARTED | |
| S8 Verification/review | NOT STARTED | |

## Decisions and departures

- Native CoreAudio/WASAPI/ALSA is the required baseline. Existing JACK launcher is
  retained; production JACK, direct ASIO and native PipeWire require their own evidence.
- Existing stereo-only CLAP standalone host is not adopted unchanged. The new host
  core uses the Rust engine and preserves the macOS 11 minimum.
- The keyboard replaces only VOICES/STEREO/OUTPUT; the left-hand controls remain.
- After the first mock-up, the owner requested a more faithful original Model D
  appearance and the realistic feel of the existing editor (2026-10-13). The revised
  design uses the original 44-key F-to-C span, replacing the spec's proposed 49 keys,
  with original generated ivory/black key materials and existing walnut/black face
  textures. The source/provenance and exact generation prompts accompany the assets.
- CPAL 0.15 exposes names/capabilities rather than stable hardware IDs. Ambiguous
  matches are rejected. Separately named capture/playback endpoints cannot be proved
  to share a clock: this restricts Windows external-input devices and remains a
  support limitation, not a passed test of every same-interface endpoint pair.
- Linux installer component/upgrade/uninstall tests pass in an isolated prefix,
  including spaces/percent signs and session preservation (3 tests). Desktop entry
  validation and packaging shell syntax pass. Native packages are not yet verified.
- No owner's hardware opened during research. Physical-interface test selection awaits
  a concrete inventory and authorization; simulated/virtual tests remain available.
