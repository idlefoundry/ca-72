# Original-recording reference fitting, 2026-10-21

The owner's acceptance example is the original studio excerpt used to create White Lotus
(R43), rather than a render of that existing preset. This run used that recording and the
shipped `ca72-mcp` stdio interface to produce a new candidate from a neutral single saw,
export a normal user-preset file and reload/render that exact file. It is a measured
candidate for listening review, not an assertion of perceptual identity.

## Source and scope

- Original user-supplied file: SHA-256
  `e12b8449c0b8333b72e73969764bcccf4bf42df77bbe64699950c8f5c653fcce`.
  This matches the `e12b8449c0b8333b` prefix in the October 9 fitting session's record.
- Locally decoded first 52 seconds, preserving 48 kHz stereo: WAV SHA-256
  `deb30712f33f29b5b8809c7fe6a7dec051a9e7f7052ba14d59f3f4b19394832a`.
  Neither the source audio nor the private listening package is committed to this repository.
- The earlier agent's 30-note MIDI transcription supplies timing. The opening onset was
  rechecked/corrected from the recording's bass-band rise, and the four sustained pitches
  were measured again. This is not a fresh automatic-transcription test.
- D2 and C2 guided the fit; G2 and B2 were reserved for validation. No separation model,
  upload, model download, audio device or live editor was used.

## Method and results

The starting file sets a single saw at 8', CUTOFF 0, EMPHASIS 0, no filter contour or
modulation. It does not load White Lotus. The agent selected staged candidate grids for
oscillator shape, cutoff/resonance, keyboard tracking and modulation: 112 tonal/modulation
candidates, then 9 attack/held-level candidates and a measured TUNE adjustment. Each render
and held-note analysis went through MCP. Candidate state, settings, source fingerprints,
WAV hashes and the complete request/result trace are retained in the private evidence.

H1–H8 are relative to each note's strongest harmonic, floored at -50 dB, with weights
`[4,4,4,2,1,1,0.5,0.5]`. D2 has weight 3, C2 weight 1. This note-conditioned objective avoids
trying to fit the recording's percussion and other instruments with the dry synth.
The weighted harmonic RMS error falls from 11.412 dB at the neutral start to 2.424 dB in
the short fitting render; the final, tuned/enveloped phrase measures 2.510 dB on those notes.
The opening recording's third harmonic is approximately 5.8 dB above its fundamental.
Its modulation sideband spacing is 11.25 Hz, and the selected candidate's is 11.25 Hz.

| Note | Role | Final harmonic RMS error | Candidate pitch minus reference |
|---|---|---|---|
| D2 | Fitting | 1.887 dB | +0.093 cents |
| C2 | Fitting | 3.810 dB | +7.324 cents |
| G2 | Held out | 3.300 dB | +4.466 cents |
| B2 | Held out | 4.740 dB | -0.181 cents |

Selected sound, in MCP panel units (the rest retained from the neutral start):

```json
{
  "osc1_waveform": 2, "cutoff": -2.25, "emphasis": 8,
  "osc3_control": false, "osc3_range": 0, "osc3_waveform": 0,
  "osc3_frequency": 8.15, "mod_mix": 0,
  "filter_mod": true, "osc_mod": false, "mod_wheel": 0.18,
  "tune": -0.026505,
  "loudness_attack": 0.5, "loudness_decay": 3.5, "loudness_sustain": 10
}
```

The preset export uses the normal serializer, which converts oscillator frequency to its
stored physical pot position and excludes player conditions. The 52-second selected render
was regenerated after review fixes from these plain overrides, without rewriting a snapshot
fingerprint; its WAV hash remained identical. Re-export/reload gives spectral MAE
0.06360/0.03358/0.06395 dB at 512/2048/8192 samples, and a gain difference of 0.00000169 dB.
The small nonzero differences follow normal four-decimal preset quantization.
Conditions are QUALITY HI, 48 kHz, seed 72, 120 BPM and semitone frequency dials. The preset
was exported and reloaded from file, not installed into the player's personal library.

The listening package includes the unfiltered original, the dry candidate, a minimally
accompanied seven-second opening D2 pair, the full phrase, same-scale MCP spectrograms,
the preset and manifests. Listening gain is measured from the opening D2's stereo energy
at 1.9–7.4 seconds through an identical fourth-order 40–400 Hz Butterworth band-pass:
-1.16249 dB on the candidate, then common attenuation to prevent clipping. An optional
identically filtered pair is labelled as a listening aid, not an isolated stem. Full-mix
LUFS and spectral error remain diagnostic, not the bass fitting objective.

## Gaps found by this test

- Without an open editor, the original tools could retain a snapshot but not deliver a
  saved preset. `ca72_preset_export` now writes only to a fresh directory, using the existing
  library writer. `ca72_render.preset_file` reads the exact exported file through the
  browser's parser, recording its hash, without importing it or resolving a factory overlay.
- Named/file preset rendering must select AUTO GAIN's curve for the requested quality
  before rendering; default LO's curve must not leak into a HI/ULTRA request.
- Offline construction must clear computer MIDI preset-note shortcuts. The old production
  bridge rendered a bound D2 silently and changed its WAV hash; the corrected bridge renders
  the same audible, identical waveform with or without that setting. A production stdio
  regression covers this, because unit-test parameter defaults deliberately omit settings.

The fitting skill now calls out original-versus-synthetic provenance, validation notes,
full-mix gain pitfalls, and export/file-reload delivery. These are agent workflow guidance,
not an automatic optimizer or a similarity percentage. A broader recording corpus,
macOS/Windows runtime checks, and the owner's listening judgement remain separate work.
