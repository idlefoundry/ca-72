# Waveform calibration, 2026-10-22

The owner authorized a new circuit-calibration stage after the fixed-waveform, main-panel-only matching pass: preserve the current build and focus on matching the waveforms while following the project rules. This stage is separate from the earlier knob-only candidates. No new blind acceptance is implied.

## Preserved reference

- Source: `d9d12044ef2e40dacb16503f5f9cec7131234cf2`.
- Engine source fingerprint: `7a3a72a074bd2465fd34b9d43cc84a1d15d2c8915d801a706290539a921550fb`.
- Independent, read-only copies of both prepared executables, the comparison app, the exact source archive and continuation metadata were made before changes. The session's `preserved-builds/2026-10-22-d9d1204/manifest.json` records SHA-256 values. Original executables, installed app, raw recordings, panel photos, existing snapshots and blind assignments stay intact.
- Work is isolated on `codex/waveform-calibration-20261022`; it has its own target directory.
- Current raw captures use EVO4 input 1 at 48 kHz. Historical session C uses a different interface and has simultaneous MIX and MAIN captures. Do not transfer the older interface's measured input response to the new rig.
- Evidence stays outside the repository, under the session's `calibration-waveforms/` directory. Historical originals are under `lab/artifacts/ca-72/calibration/captures/2026-10-08-C/`; their SHA-256 values are checked before analysis.

## Method

Follow the circuit-plugin hardware-calibration rules: ask ngspice first, trims before parts before knob laws, physical mechanisms before fitted numbers, and standard component values where supported. Keep as-drawn component constants and factory procedure tests. A component change must propagate to the circuit reference and every affected real-time quality.

Keep the offered waveform switch positions fixed. No continuous waveform interpolation, output EQ, added filtering, harmonic corrections, phase fitting or seed search. Audition derivatives may have one documented constant gain and timing offset. Measurements can inspect harmonic ratios and separately account for downstream signal paths; those calculations do not alter the audio.

Current MAIN captures alone cannot uniquely identify the oscillator waveform: mixer, filter, VCA and input-path effects are included. Use the older simultaneous MIX/MAIN recordings to constrain source shape, then validate against the newer MAIN captures and controls. Treat an equivalent component mechanism inferred from audio as a model calibration, not a measurement of the physical unit's installed part values.

## Acceptance matrix, before implementation

1. Reproduce the difference in the immutable recordings, with common analysis, measured noise limits and repeat-window uncertainty.
2. Compare the drawn circuit and preserved real-time VCO at multiple pitches. Separate existing approximation error from hardware mismatch.
3. Test one physical mechanism at a time; if two components trade, fit them jointly and report the trade. For pulse calibration, select a common bias/width mechanism on Square and Narrow at the outer historical MIX pitches (about 84 and 359 Hz); hold Wide and every middle-pitch result (about 174 Hz) out of final parameter selection. Earlier exploratory measurements are disclosed. Current MAIN remains combined-path validation, not an unloaded-source fit. Quantify repeat-window variation and harmonic/noise support separately from harmonic-cutoff sensitivity.
4. Regenerate reference parameters/tables from the candidate netlist; do not hand-edit a generated waveform or fit output harmonics directly.
5. Check every fixed waveform, all three software oscillator instances, all relevant quality modes, sample rates, tracking and alias bounds. Current hardware waveform targets cover OSC1 only; software coverage is not evidence of physical OSC2/3 calibration. Keep service-document checks against the as-drawn circuit passing.
6. Render factory presets before and after with identical settings. Identify which change and report level/spectrum movement; do not re-level presets during diagnosis.
7. Run formatting, Clippy with and without all features, and touched-crate tests under the project's debug validation workflow. Independently review an immutable commit and recheck any fixes.
8. Save a new source fingerprint and exact exported preset/render proof for every audition candidate. Existing snapshots remain associated with their original build; do not relabel them as compatible with a different engine.
9. Put identified hardware/before/candidate audio, graphs, methodology and remaining differences on the local review site. The owner judges remaining audible differences. Background-noise parity remains a separate requirement for a later blind test.

## Initial findings

The fixed waveform path still matches the existing ngspice reference within its existing tests. The continuous-mode knot test also agrees with the stepped path. Historical MIX captures show pulse-duty and triangle-shape differences before the downstream filter/VCA. Detailed measurements, uncertainty and candidate decisions will be added with the individual calibration changes.

No calibration component or renderer change has been selected yet.
