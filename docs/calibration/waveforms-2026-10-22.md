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

## Experimental source candidate

The combined candidate uses Q2-only BR 0.35 and the standard 9.53K front-panel R031,
retaining the drawn transistor model and resistor values as separate references.
Its triangle table and finite-knee LO approximation are derived from ngspice, including
the reset-area change. The physical pulse divider and transistor comparator replace the
old ideal width source; a separate provisional profile uses standard R1 41.2K and R44 374K.
These equivalent model properties are inferred from recordings, not measured installed
parts. Mechanisms, primary scans, selection exclusions and remaining source approximations:
[triangle and hybrid](waveform-fold-reference.md), [pulse source](pulse-source-20261022.md),
and [production alias coverage](pulse-alias-validation.md).

An early BR-only prototype improved triangle but regressed the hybrid. It remains preserved
with that failure. The combined model includes the subsequent physical hybrid-network
correction. Existing knob-only fits are also preserved: a triangle fit that compensated for
the old source can become too dark after calibration and must not be reused as proof.

The local site has six identified hardware / prepared / combined comparisons at the original
panel controls. All parameters, fields, notes, seed, tempo, frequency law and STEPPED states
are checked against the saved baselines. Each audition has a fixed crop/alignment and one
constant source gain, followed by PCM24 quantization; all 18 clips reconstruct exactly.
Full raw recordings, renders, snapshots, source hashes, gains and noise flags are available.
No output filtering or other correction is applied. Unequal background noise prevents using
these diagnostics as accepted blind trials.

The first frozen combined build is source commit `c8b07cb`, source fingerprint
`533243c203a4d3646cc88cb022777f2037bf7afcd51d751ea90cf9426d7b526d`.
Subsequent validation commits and test fixture changes must be recorded separately; textual
changes inside `cfg(test)` still change the source fingerprint on a rebuild. Never relabel
the frozen executable or rewrite its snapshots to a later source identity.

| Waveform | Prepared harmonic RMS | First combined RMS |
| --- | ---: | ---: |
| Triangle | 4.008 dB | 1.488 dB |
| Triangle / saw hybrid | 0.277 dB | 0.293 dB |
| Sawtooth | 0.197 dB | 0.197 dB |
| Square | 8.594 dB | 2.048 dB |
| Wide rectangle | 9.666 dB | 1.914 dB |
| Narrow rectangle | 8.970 dB | 0.987 dB |

These feature errors use harmonic groups through 20 kHz at each source's own fundamental,
with the same Hann analysis and a 10 dB quiet-proxy support rule. They are not perceptual
similarity scores. A stricter source-ON adjacent-band proxy leaves five waveforms unchanged;
Square H27 becomes unresolved in hardware. Its shared-group score changes to 7.797 / 1.320 dB,
while that candidate-only excess remains explicit, at least 6.32 dB above the working hardware
support threshold. Both scoring methods and all groups are retained. Three fixed adjacent
guard variants agree on the masks. No favorable mask change is silently substituted.

## Preset effects and open checks

The first candidate's 19 factory presets were rendered before and after in HI, LO and ULTRA,
with unchanged settings (4 seconds, 10 voices, 48 kHz, 256-frame blocks, workers off).
Maximum absolute level changes were 0.1394 / 0.1899 / 0.1394 dB respectively. Nine LO presets
are byte-identical; no HI or ULTRA render is byte-identical, including tiny numerical changes.
This is not a claim that all presets are audibly unchanged. LO Smoke Flute's 3.175 kHz band
falls 5.43 dB, from about 30.24 to 35.68 dB below its strongest band. Presets are not re-levelled
during diagnosis. The session's `factory-impact.json` retains every render hash and change.

The first full debug run exposed a stale hybrid harmonic fixture, a stale pulse display
fixture and a real LO shape-modulation normalization issue after the asymmetric triangle
source changed. The fixture corrections retain the original tolerances and current circuit
reference. The normalization must follow both source extrema, as the circuit voice does.
Independent reviews and final validation results are retained with the session artifacts.
Until the normalization fix and complete checks are recorded, the first candidate is blocked.

The original build and 289 core capture/selected-fit assets are independently verified on
the NAS checkpoint without mismatches. The installed app and capture application remain
unchanged. Hardware source validation currently covers OSC1; other physical oscillators,
additional levels and notes remain outside this calibration's demonstrated coverage.
No listening or blind equivalence has been accepted.
