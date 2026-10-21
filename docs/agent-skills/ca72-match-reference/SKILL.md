---
name: ca72-match-reference
description: Fit a CA-72 synthesizer preset to a user-supplied audio reference using its MCP controls, repeatable offline renders, spectrograms, pitch, harmonics, envelopes and modulation sidebands. Deliver a saved, reloaded preset with level-matched audio and measured remaining differences.
---

# Fit a CA-72 preset to a reference

Use the CA-72 MCP tools. The deliverable is a usable preset, a level-matched listening
comparison, and a candid account of what still differs. Change supported sound controls;
do not alter DSP code to fit the recording.

## Reference and processing

Keep the original intact. Processing in these tools is local: mono/stereo PCM or float WAV,
8 to 192 kHz, excerpts up to 60 s and files up to 128 MiB. An existing local converter may
make a WAV copy of another format. Source separation is optional and outside these tools:
agree any external processing with the user before uploading audio or downloading models.
When the user supplied a recording, that recording is the target. A render of an existing
preset is a synthetic regression fixture and does not establish reference-to-preset fitting.

Choose an isolated part or the clearest passage. For a full mix, establish which part the
user means and avoid fitting other instruments. A separated stem can help with note timing
and envelope; verify harmonics against the original mix because separation can move upper
harmonics into another stem. White Lotus exposed exactly this problem.

Call `ca72_analyze` with the reference's absolute `path`, optional `start` and `seconds`,
and a fresh absolute `output_dir` whose parent exists. Keep the source hash and crop. Inspect
the spectrogram, pitch track and onsets; establish the notes and their gates before timbre.
Pitch tracking assumes a predominant monophonic part. For polyphonic references, identify
simultaneous fundamentals independently from spectral spacing; do not interpret monophonic
YIN or every reed register/harmonic as a played note. Use overlapping MIDI gates with POLY on,
retain the chord's relative balance in measurements, and validate another chord/take with
the patch frozen only when its timbre/register setting is the same. Two repetitions of the
same chord may use different reed/register settings: split and fit separate presets when
they do. Matching pitches alone do not establish a shared sound. Overlapping harmonic bands
describe aggregate chord energy. Check uncertain octaves and transients
by ear and the harmonic spacing; a full mix does not become a trustworthy transcription.
If using an existing transcription, identify its provenance and recheck pitches and timing
against the recording. Reserve some notes or passages for validation after fitting.

## Repeatable candidates

For live work, select the intended instance, read `ca72_describe`, and capture `ca72_snapshot`.
The snapshot preserves unsaved values, assignments, AUTO GAIN, the frequency dial law, tempo
and seed. It describes settled settings, with fresh notes, oscillator phases and controllers.
The bridge and app must have the same engine source fingerprint.

`ca72_render` takes that `snapshot`, or a library `preset`, plus:

```json
{
  "preset": "White Lotus",
  "effects": "dry",
  "notes": [{"note": 38, "start": 0, "duration": 1.5}],
  "seconds": 2,
  "sample_rate": 48000,
  "seed": 72,
  "output_dir": "/absolute/existing-parent/candidate-01"
}
```

With neither preset nor snapshot, it captures the selected live instance. `values` supplies
offline-only candidate overrides in `ca72_set`'s panel units; the result contains the complete
resulting snapshot. Keep the best snapshot and a record of its plain-value overrides and
assignment changes so they can be applied live. Use the reference's rate, matching notes and
timing, enough release tail, and the same seed, quality and tempo across candidates. Preset-only
renders use canonical 120 BPM and semitone frequency-dial law; live captures retain theirs.

This version renders the dry synth, explicitly `effects: "dry"`, with silent external input.
It includes the arpeggiator, LFOs, macros, stereo, DRIVE, LEVEL and AUTO GAIN. It neither
renders hosted effects nor captures live MIDI pressure, pedals, wheel offsets or held notes.
Explain when one of those is material to a reference. Never claim a dry match reproduces
reverb or another effect. Do not turn ENTROPY or other sound controls off to hide randomness;
the seed makes it repeatable.

## Fit by measurements and listening

Call `ca72_compare` with absolute `reference` and `candidate` WAV paths, explicit crop starts
where needed, equal `seconds`, and a fresh `output_dir`. It refuses rate/duration differences.
It level matches, returns spectral errors at three time resolutions, envelope error, pitch
difference, and paired spectrograms on one scale. Full metrics include the envelope traces;
the output also contains two level-matched A/B WAVs. Do not compare raw waveforms or use
correlation as a timbre score: freely running oscillators have arbitrary phase. The tools
do not silently align time or pitch, and their errors are not a similarity percentage.

Fit the audible structure, using a small number of changes each iteration:

- Pitch and gates first; check glide and retrigger at note transitions.
- Oscillator shape/count, relative tuning and filter tracking from harmonics across notes.
  For a held-note crop pass its known `fundamental_hz`. Inspect H1 to H8 relative to the
  fundamental and the strongest harmonic, especially a resonant third. A whole-phrase
  average can hide the missing harmonic that matters.
- Filter and loudness contours from attacks, held levels and releases. Include candidate-only
  tails. Preserve the musical timing while fitting the envelope.
- Modulation from sideband spacing and level: supply `modulation_hz` and a long enough held
  note to resolve it. Compare sidebands around several harmonics. Unresolved sidebands are
  null, not proof of absent modulation. Test whether detuned oscillators explain beating
  across several notes; avoid a pair which fits only at one locked phase.
- For vibrato which grows into a held note, test a repeating LFO with RISE before concluding
  that a static contour is the best match. Try shallow volume, loudness-sustain and mixer
  modulation as well as pitch; measure both AM and FM depth over early/late note windows.
  Average harmonic spectra alone miss missing movement. RISE fades modulation in over its
  duration; it is not a fixed delay. Each LFO's `Retrig` mode resets phase and RISE on every
  note-on independently of the main `retrigger` switch: keep that switch off for legato
  loudness/filter contours. Shared ENV and FREE's RISE still follow the main switch's phrase
  trigger, so a separate phrase envelope can continue while vibrato resets. Rate estimates
  from short windows can be imprecise;
  removing a polynomial trend does not fully reject slower expression. Do not claim matched
  AM/FM phase coupling from a phase-independent modulation objective.
- Use the four LFOs as secondary envelopes when the main contours are insufficient.
  Configure `lfoN_mode: "Env"`, waveform, rate (one cycle's inverse duration), phase, rise,
  polarity, reverse, mono and level through `values`; supply `ca72_render.modulations`
  routing edits (`source`, `target`, `depth`, optional `edges`). A triangular UNI envelope
  rises then falls; a reversed UNI saw falls then holds zero. Rates can also follow a timed
  division. Use `remove:true` to remove a route or source's assignments. Parameter overrides
  apply before routing validation. An existing snapshot/preset's routes stay unless edited.
  With MONO off, oscillator/noise mixer volumes and cutoff can have per-voice envelopes;
  main volume and output level always share one copy. Consider separate sources for breath,
  brightness and vibrato; measure the result rather than forcing every source into a fit.
- Check several notes and transitions after each promising local fit. Listen to the
  level-matched pair, using the user's ear to judge what the metrics prioritise poorly.

Use the best measured candidate as the next starting point. Keep it when a trial is worse.
Stop when remaining differences reflect the synth's capabilities, evidence is insufficient,
or several targeted trials make no useful progress; describe that limit instead of claiming
an exact match. Do not invent an optimisation algorithm or an arbitrary numeric pass mark.

For a mix containing other parts, whole-mix LUFS does not establish matched bass loudness,
and whole-mix spectral error must not drive a dry bass fit. Use the clearest note's harmonic
energy to set listening gain, or provide the same clearly labelled local band-pass listening
aid for both signals. Include the unprocessed recording too. Explain any filtering and gain.

## Deliver a preset, then verify it

For a file deliverable without an open app, call `ca72_preset_export` with the selected
`snapshot`, `name`, optional `tags` and a new `output_dir`. It uses the normal preset writer,
preserving assignments, macro names and AUTO GAIN; it does not install into the player's
library. Then call `ca72_render` with that exact returned `preset_file`, its
`render_conditions`, the same notes, duration and sample rate. Compare this reload against
the selected candidate and the recording. Keep the exported file and manifest. The conditions
include quality, pitch wheel, bypass, MIDI bend range, tempo, seed and frequency-dial law,
which presets do not store. Frequency overrides are agent semitones; the preset writer
converts them to the correct stored pot positions. Do not write panel overrides as TOML
values yourself. Named/file preset rendering selects the requested quality's AUTO GAIN curve;
snapshot overrides retain the captured curve.

To install into the player's live library:
If candidates began with a library preset, load that starting preset into the intended
instance first. Apply the best candidate's plain values and assignments through `ca72_set` and modulation
tools to the intended instance, setting its captured quality with ca72_set, and tempo and frequency_dials with
ca72_settings_set. Reuse the candidate seed as an explicit render argument.
Offline overrides retain the captured AUTO GAIN curve. For a new DRIVE/AUTO GAIN calibration,
make that change live, release the agent's notes and wait for measurement before capturing
the next candidate. `ca72_snapshot` returns BUSY while AUTO GAIN is measuring. Check the
captured curve is stable before saving, and report a changed curve if it alters the fit.
Capture and render again: this tests that the actual live settings reproduce the candidate.
Use `ca72_preset_save` with the user's name and suitable tags. Load that preset back, capture
and render the same phrase again, and compare it with the pre-save render and the reference.
Saved values are rounded to four decimal places, so report any material change; do not
require sample identity if quantisation changed a frequency. Presets exclude the player's
quality, tempo, seed and frequency-dial preference: keep and report those render conditions.

Deliver the preset name, replay conditions, level-matched A/B audio, and the most useful
spectrograms/measurements. Distinguish measured facts from listening judgements and unresolved
differences. Keep candidate WAVs and the complete snapshot. Distinguish a file export/reload
from installation into the player's live library.
