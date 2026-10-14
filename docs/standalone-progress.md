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
- No owner's hardware opened during research. Physical-interface test selection awaits
  a concrete inventory and authorization; simulated/virtual tests remain available.
