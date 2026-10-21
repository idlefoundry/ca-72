# Acoustic and polyphonic reference studies, 2026-10-21

The owner selected a short real flute reference and requested a second instrument playing
simultaneous notes, then asked that the LFOs be available as secondary envelopes over MCP.
Two working presets were fitted using supported controls and the same offline bridge,
without modifying the DSP or installing anything in the personal library. These are synth
approximations for listening review, not claims of exact acoustic reproduction.

## Sources and permissions

- Flute: [Flute notes.aiff by trumpetplayer](https://freesound.org/people/trumpetplayer/sounds/648201/),
  described by its uploader as an improvisation, published under
  [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Crop: 16.8–24.5 s.
- Accordion: [Accordion C major chord.wav by juskiddink](https://freesound.org/people/juskiddink/sounds/120689/),
  two takes, 7.662 s; validation also uses
  [Accordion C minor chord.wav by juskiddink](https://freesound.org/people/juskiddink/sounds/120690/),
  4.484 s. Both [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
  Creator's description: real Paolo Soprani 80-bass accordion, Rode NT4 into Zoom H4,
  unprocessed. Recording credit, source/licence links and notice of conversion/listening-gain
  changes accompany the listening artifacts. No endorsement is implied.

The working inputs are Freesound's public HQ MP3 previews decoded locally to 48 kHz WAV,
not original lossless masters. Original downloaded bytes, creator-page snapshots, hashes,
crops, scripts and MCP traces are retained with the listening package outside the checkout.
The audio and generated plots are not committed. Published permission permits the described
commercial reuse subject to its conditions; it is not a guarantee against automated claims.

## Method and remaining differences

Start: a neutral single saw; POLY with three simultaneous MIDI pitches for accordion.
Pitch/gates were established from the original signals before timbre. Monophonic pitch
tracking was not used to transcribe chords: independent C4/E4/G4 fundamentals were inferred
from spectral spacing, while octave registers and upper harmonics remained timbre. C minor
uses C4/Eb4/G4. No source separation, audio upload or model download was used.

Fit flute E4/A4 and accordion take 1. Exclude the flute's G#4/G4 ornaments and final F4,
accordion take 2, and the separate C-minor recording from control-selection objectives.
Measure H1–H8 integrated stereo power in bands with 1.5%/at-least-7-Hz halfwidth; use weights
`[3,3,2,2,1,0.8,0.5,0.3]` and a -45 dB floor. Monophonic notes normalize to their fundamental;
the chord normalizes once to total fundamental-band power, retaining relative note balance.
Chord bands overlap, so their scores describe aggregate chord energy. Candidate bands follow
its measured fundamental: report cents errors independently. Breath noise was selected by
interharmonic energy, not rewarded for filling weak harmonic bands.

A 50 ms-smoothed RMS envelope, evaluated every 10 ms over fixed fitting windows, selects
contour/LFO candidates. The same gates, duration and windows are used for the final neutral
baseline and fitted comparisons. Values are measurements, not perceptual similarity scores.

| Study | Neutral/final band RMS error | Independent validation |
|---|---|---|
| Flute | 4.456 / 3.923 dB | G#4 7.320 dB, G4 10.871 dB, F4 3.488 dB; pitch -5.1/-15.7/+11.9 cents |
| Accordion | 6.648 / 5.895 dB | C-major take 2: 4.747 dB; C minor: 4.053 dB; chord-note errors within 6.5 cents |

The flute's brief G4 is a weak match. A static patch does not follow every change of breath,
articulation, intonation or vibrato. The accordion retains a difference in individual reed
balance/tuning, bellows motion and stereo room/microphone sound. Dry rendering and pitch/gate
events do not reproduce performance-expression curves or a room response.

## Secondary envelopes and offline MCP

Live MCP already exposed all four LFOs, including ENV, through parameters and modulation
tools. The missing workflow was editing routing on a detached candidate without an app.
`ca72_render.modulations` now applies ordered set/remove edits after `values`, sharing the
live target rules and preserving the resulting routing in snapshots and saved presets.
Rejected batches leave sound state unchanged and create no output. Effects pins are refused.

The accordion uses LFO 1 in ENV, rising UNI saw, 5 Hz (0.2 s attack), MONO off. It routes to
oscillator 1 mixer level at 100% and oscillator 2 at 60%, whose base levels are zero.
The source holds its final value; the usual loudness contour handles key release. A separate
12-second held-note render retains its level (late/early AC RMS ratio 1.020). This supersedes
a square-with-RISE audition whose eventual falling edge would stop a long held note.
Main VOLUME and strip LEVEL remain shared; mixer/cutoff targets can be per voice.
Flute LFO trials did not beat the overall static-contour selection and were not forced in.

Both prior static waveforms regenerated with identical hashes after the API-only change.
New regression coverage checks actual per-voice ENV sound, a second overlapping note while
the first stays held, one-cycle completion, exact-file export/reload, invalid routing batches
(including numeric source zero), removal and bridge liveness. The shared parser also rejects
macro EDGES before assigning anything, and numeric source zero no longer panics.

## Delivery and replay

Working files: `Flute Study.toml` and `Accordion Study.toml`, exported with the normal preset
writer then replayed by exact file path. QUALITY HI, 48 kHz, seed 72, 120 BPM, semitone dials.
Normal four-decimal preset quantization gives reload spectral MAE at 512/2048/8192 samples:
flute 0.191/0.122/0.080 dB; accordion 1.001/0.490/0.269 dB. Reload gain differences are
-0.00008/+0.00138 dB and RMS-envelope MAE 0.0054/0.0239 dB respectively. Short-window errors
are more sensitive to detuned phase; no sample-identity or perceptual-pass claim is made.
The listening comparisons and novel phrases use the delivered, reloaded preset files.

The original and fitted pairs are matched by MCP LUFS, followed by the same gain on both
for listening near -18 LUFS subject to headroom. Novel phrase loudness is normalized
separately and labelled. The owner has not yet approved the sound; no release or public
video upload has occurred.
