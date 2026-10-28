---
name: ca72-match-reference
description: Turn a supplied audio reference into a playable CA-72 preset using local analysis, repeatable offline experiments, level-matched listening and exact saved-file replay.
---

# Match a reference with CA-72

Deliver a usable preset, a fair listening comparison and a candid account of the remaining
mismatch. Use supported sound controls; do not change circuit DSP to fit a recording.
The original recording is the target. An existing preset can be a starting point or approved
baseline, but its render is not a replacement for that recording.

## Establish the target and performance

Keep the original intact and record its hash and chosen crop. Tools accept mono/stereo PCM
or float WAV at 8–192 kHz, up to 128 MiB, with selected excerpts up to 60 seconds. An existing
local converter may make a WAV copy. Source separation is optional and outside these tools;
agree external uploads or model downloads with the user first. A separated stem may help
with timing, but check harmonic balance against the original because separation can move
partials into another stem.

Identify which part, sound setting and articulation the user means. Split passages which
use different timbres even when their notes repeat. Separate the performance (notes, gates,
legato, expression) from the patch's tone/motion and from room or other effects. Establish
pitch and timing before refining tone. For chords, supply simultaneous fundamentals rather
than treating harmonics as extra played notes. Monophonic pitch tracking is not polyphonic
transcription. Recheck an existing transcription and identify its provenance.

Use `ca72_analyze` with named `regions`: separate notes/chords, attacks, holds and releases,
with `id`, clip-relative `start`, `seconds`, optional `fundamentals_hz` and `modulation_hz`.
Each window needs at least 1024 samples; supplied fundamentals need enough cycles and FFT
resolution and a credible local peak; weak leakage from an out-of-range tone is refused.
Region fundamentals are locally refined within ±4% independently for each file.
Comparable competing peaks can make a carrier ambiguous: pitch/AM/FM stay null while
harmonic bands retain explicitly labelled spectral anchors. Do not treat null as a match.
Chord normalization is shared across fundamentals; overlapping bands describe aggregate
energy, not separated voices. A frequency estimate is not proof of a transcription.
Reserve representative notes or passages for later validation when the reference permits.
Once inspected to select a candidate, a passage is acceptance data, not an untouched test.

## Choose a reproducible starting state

For live work select the intended instance, read `ca72_describe`, then `ca72_snapshot`.
For detached work `init:true` starts from factory parameter defaults without a library or
app. A library `preset`, exact `preset_file`, `snapshot` or `snapshot_file` also works offline.
Choose one source only. Snapshot files retain exact state and require the same engine source
fingerprint; ordinary preset files are portable but quantized by their normal serializer.

Keep quality, sample rate, tempo, seed and frequency-dial law fixed while fitting. Snapshots
preserve unsaved values, routing, macro names and AUTO GAIN. Renders use fresh voices,
oscillator phases and controllers; they do not capture a performance already sounding.
Seeded noise and component variation repeat. Do not disable sound controls to conceal
randomness. Change seeds during later robustness checks, not between fitting candidates.

Supply matching `notes [{note,start,duration}]` (C4=60), total `seconds` including release,
`sample_rate`, a fresh absolute `output_dir` with an existing parent, and `effects:"dry"`.
Dry output includes arpeggiator, LFOs/macros, stereo, DRIVE, LEVEL and AUTO GAIN, but excludes
hosted effects, external input and live pressure/pedal/wheel history. Explain those limits
when material to the reference. No hidden pitch/time alignment, padding or truncation occurs.

## Run purposeful, bounded experiments

State the suspected cause and what a trial should change. Vary a small related parameter
family at a time: oscillator shape/layers, filter/tracking, contours, then modulation as
appropriate. Use the shortest informative performance; run the complete phrase on finalists.
Keep a control candidate and retain a better prior result when a trial is worse.

Prefer `ca72_experiment` for a sweep instead of writing a custom render/FFT/report script.
It uses the same sources and render arguments, plus the actual `reference`, named `regions`
and up to eight independent `candidates`. Common values/routing apply first, then each
candidate's changes. All candidate settings validate before output is created. Reference
regions are decoded/analyzed once per batch. Rendering is sequential and bounded to 120
rendered seconds and 12 million frames total. Example fields, alongside a valid source and
performance:

```json
{
  "reference": "/work/reference.wav",
  "regions": [
    {"id":"held-note","start":0.4,"seconds":1.0,"fundamentals_hz":[220]},
    {"id":"release","start":1.7,"seconds":0.3}
  ],
  "candidates": [
    {"id":"control"},
    {"id":"darker","values":{"cutoff":-0.5}},
    {"id":"shape","values":{"osc1_shape_mode":"Continuous","osc1_shape":0.36}}
  ]
}
```

The returned candidate summaries keep harmonic, full-spectrum, pitch, peak and preservation
results separate. Full evidence, source hashes, exact snapshots and a common-level HTML
listening report are written to disk. There is no automatic winner. Continue a promising
candidate with its `snapshot_path` as `snapshot_file` in another fresh batch; large snapshots
need not pass through the agent context. `experiment.json` marks a completed batch. A runtime
failure can leave partial candidate files: inspect their manifests and continue in a new
folder, never treat an incomplete batch as completed evidence.

## Measure tone and preserve approved behavior

Use regional harmonic balance together with the full measured spectrum. A harmonic-only
objective can improve while adding an unrelated resonance. Inspect strong unexplained peak
flags, upper-band differences, attacks and tails. Flags can also reflect tuning or changed
harmonic balance; inspect them rather than treating every flag as an automatic rejection.
Do not conceal a new tone through whole-signal normalization, a notch or a restricted band.

Named-region harmonic RMS uses uniform H1–H8 weights and a -60 dB floor; spectral-band RMS
uses a -80 dB floor. Normalization uses total supplied fundamental-band power, or 50–1000 Hz
when no fundamentals are supplied. These are diagnostics, not universal objectives or a
perceptual similarity percentage. Full-mix LUFS and spectrum include other instruments:
choose clear passages for fitting the intended part. Preserve the unprocessed original if
providing an explicitly labelled, identically filtered listening aid for both signals.

When the user approves expression, retain its gates, pitch, main contours and expression
routes while fitting tone. Supply the actual approved render as `approved` to
`ca72_experiment` or region-aware `ca72_compare`. Optional `preservation_limits` are explicit
engineering budgets for envelope RMS/p95, AM/FM depth and pitch shift, not perceptual approval.
No default pass mark is invented. Check each short note and audible attack/release as well
as the whole clip; a whole-phrase percentile can miss a complete short ornament.

Preservation compares rendered audio after one common AC-energy gain. Its 50 ms AC-RMS frames
at 10 ms hops ignore DC shifts and include candidate-only audible tails. Unchanged knobs do
not guarantee unchanged expression: waveform/filter changes and oscillator beating can
alter it. Small mixer compensation may balance a tone change, but verify the result.
AM/FM limits need at least one region with `modulation_hz`; pitch limits need fundamentals.
Requested unresolved checks remain null and cannot become a pass. AM/FM depth is approximate,
needs one isolated fundamental and at least three resolved cycles, and returns null for an
unreliable analysis-window response (roughly above 11 Hz). Use sideband analysis for other
rates; unresolved sidebands are also null, not evidence of absent movement.

Use the four LFOs for repeating movement or secondary envelopes when the measured behavior
calls for it. `lfoN_mode:"Env"` plays one cycle and holds its endpoint; rate is inverse cycle
duration or a timed division. Each LFO's `Retrig` restarts phase and FADE on every note-on
independently of main `retrigger`. Keep main retrigger off when physical contours should stay
legato. Shared Env and Free's FADE retain phrase/main-trigger behavior. `lfoN_fade` is in
seconds, signed: positive fades the LFO in from each note (a ramp, not a hard delay), negative
fades it out from full to nothing, 0 is off. With MONO off, mixer/filter routes can be per voice; main volume/output remain
shared. Configure values before `modulations [{source,target,depth}]`; depth is signed percent
of target travel. Route continuous oscillator shape through `oscN_waveform`, not
`oscN_shape`. Existing routes remain unless edited or removed with `remove:true`.

Prefer balanced improvements across the phrase. Explain which audible aspect a tradeoff
serves; keep user listening judgement authoritative. Stop when targeted trials cease making
useful progress, evidence is insufficient, or remaining differences reflect synth limits.

## Generalize, save and replay

Check different pitches, durations, legato/staccato playing, relevant chords and several
seeds with settings frozen. A phrase-timed envelope may reproduce an excerpt's performance
without defining a generally useful instrument response: test it with different phrasing.
New synth-only performances demonstrate playability, not acoustic validation without a
corresponding recording of the same sound setting.

Export the selected snapshot using `ca72_preset_export` with its `snapshot_file` (or snapshot),
name, optional tags and a fresh output directory. Render the exact returned `preset_file`
with its `render_conditions`, same notes, duration and rate. Compare the replay with the
pre-save candidate and original. Normal four-decimal preset quantization may change sound;
judge and report the actual delivered replay, not only the pre-save candidate. Render
conditions include quality, pitch wheel, bend range, bypass, tempo, seed and frequency-dial
law, which the preset does not store. Do not manually write panel units into preset TOML.

Export/reload does not install into the player's library. When installation is requested,
apply the selected values/routing to the intended live instance, retain its render conditions,
then save/load/capture/replay through the live paths. For DRIVE/AUTO GAIN recalibration,
release agent notes and let the live measurement settle before capture; snapshots return
BUSY during measurement. Verify and disclose any resulting curve change.

Deliver the saved preset, replay conditions, level-matched audio and useful regional evidence,
with remaining differences and measurement limits. Experiment auditions share one common gain
and headroom cap across reference, approved baseline and all candidates. Final comparisons
must use the exact exported-file replay. Distinguish measured facts from listening judgement.
