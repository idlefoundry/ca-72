# Revised acoustic reference studies, 2026-10-21

The owner selected a real flute excerpt and requested a different instrument playing
simultaneous notes. The revised delivery has three main presets: a flute with rising
vibrato/tremolo and two independent C-major accordion register studies. Supported MCP
controls, the normal renderer and preset writer are used throughout. No DSP was modified
and nothing was installed into the personal library. Sound remains for listening review;
these numeric errors are diagnostics, not perceptual similarity percentages.

## Sources and the register correction

- Flute: [Flute notes.aiff by trumpetplayer](https://freesound.org/people/trumpetplayer/sounds/648201/),
  an improvisation under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
  The reference remains the 16.8–24.5 s excerpt from the first acoustic study.
- Accordion: [Accordion C major chord.wav by juskiddink](https://freesound.org/people/juskiddink/sounds/120689/),
  7.662 s, [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
  The creator describes a real Paolo Soprani 80-bass accordion, Rode NT4 into Zoom H4,
  unprocessed. Recording credit, source/licence links and processing notices accompany
  the listening files. No endorsement is implied.

**Owner correction:** the two chords within that C-major file have different reed/register
settings. They are both C major. The earlier report wrongly treated its second chord as
validation of one patch. That assumption is withdrawn. Register 1 uses 0–3.34 s and register
2 uses 3.37–7.661979 s, with independent targets, searches and presets. Both are fitting data.
The original is intact; the separate C-minor recording from the first study is not used here.

Inputs are public HQ MP3 previews decoded locally to 48 kHz WAV, not original lossless
masters. Original bytes, page snapshots, hashes, crops, scripts and RPC traces are retained
with the private listening package outside the checkout. Public permission allows the
stated reuse subject to its conditions, without guaranteeing against mistaken automated
claims. Audio, plots and private paths are not committed.

## Method

The starting point was a neutral single saw. Flute gates describe E4–G#4–G4–A4–F4, with
small overlaps for continuous breath. Accordion uses three simultaneous MIDI notes,
C4/E4/G4; independent fundamentals were inferred from spectral spacing, not monophonic YIN.
Reed octaves remain timbre. Gates and source timing stay fixed during control searches.
No source separation, audio upload or model download was used.

H1–H8 integrated stereo power uses at least 7 Hz / 1.5% halfwidth, weights
`[3,3,2,2,1,0.8,0.5,0.3]` and a -45 dB floor. Flute notes normalize to their fundamental;
chords normalize once to total fundamental-band power, retaining relative note balance.
Overlapping chord bands describe aggregate energy. Candidate bands follow measured pitch;
cents error is reported independently. A 50 ms-smoothed RMS envelope is sampled every
10 ms, with one common fitting gain and identical fixed windows for prior/new comparisons.
Flute E4/A4 select controls; ornaments and final F4 remain excluded. New phrases from the
exported files demonstrate playability, without an acoustic reference for those new notes.

## Flute: repeating modulation with RISE

The first pass only tried strong slow one-shot amplitude/brightness envelopes and retained
a static patch. The owner's feedback exposed that this missed the flute's growing vibrato
and amplitude movement. The revision evaluates 60 additional full-phrase candidates:
shallow AM on VOLUME, loudness sustain and mixer; AM plus FM; rate, phase, RISE
(0.35/0.9/1.6 s); shared/per-voice sources; retrigger behaviour; gentle phrase envelopes.
The previous tone and breath-noise controls are retained.

Selected `Flute Rise Study`:

- LFO 1: bipolar triangle, RETRIG, 4.2 Hz, 90-degree phase, 0.9 s RISE, shared. Assignments
  to VOLUME at 3% and TUNE at 1.2% of full knob travel. RISE grows modulation continuously
  from zero; it is not a fixed delay.
- RETRIGGER off: overlapping legato gates keep RISE running from the breath phrase's first key.
- LFO 3: UNI triangle ENV, one 7.7-second cycle, VOLUME depth 20%, for a gentle phrase swell.
  Base VOLUME 7.5. A triangle holds zero after its one cycle; ordinary release remains active.
- AM-only and loudness-sustain alternatives are exported too. The latter uses sustain depth
  15%, 4.2 Hz and 0.9 s RISE.

Selection now includes periodic AM depth in dB and FM depth in cents across early/late E4/A4
subwindows, alongside slow-envelope error and any harmonic-tone degradation. Analytic
fundamental-band traces are measurements only; audition audio is unfiltered. Short-window
sinusoid descriptors are approximate: cubic detrending does not fully reject slow expression.
A phase-independent objective does not establish matching AM/FM phase coupling. The real
player's rate/depth vary; a fixed 4.2 Hz source cannot reproduce every gesture.

Delivered-file slow-envelope RMS error falls from about 3.47 to 2.58 dB. Harmonic-band error
stays nearly unchanged, 3.923 to 3.914 dB. The old nearly static held notes gain periodic AM/FM
and a growing onset. Short G4, changing breath tone, per-note intonation and room remain weak
points. No listening approval is claimed.

## Accordion: two independently fitted presets

Each register has its own tuning, oscillator/filter/reed-layer search, full-duration contour
search and keyboard-tracking trials. Selected differences, in plain panel units:

| Setting | Register 1 | Register 2 |
|---|---|---|
| TUNE | 0.043327 | 0.071754 |
| CUTOFF / EMPHASIS | +0.5 / 3.5 | +1.0 / 9.0 |
| Keyboard tracking | switch 1 off, switch 2 on | both on |
| Oscillator 1 | continuous shape 0.34, near saw | stepped saw |
| Oscillator 2 | upper-octave saw, level 7 | upper-octave saw, level 3 |
| LFO mixer attack | 0.200 s | 0.333 s |
| Loudness attack / decay | 0 / 0.5 | 1 / 0.5 |

Both use oscillator-2 frequency +0.06 semitones and three independent polyphonic voices.
The second reed layer is timbre per note, not an oscillator tuned to a whole chord. LFO 1
is a UNI rising ENV saw, MONO off, with base mixer levels zero and routes reaching their
chosen final levels. It holds its last value; the normal loudness contour handles release.
Twelve-second held-chord AC RMS late/early ratios are 1.013 and 0.999: neither cuts off.
Per-voice mixer/filter targets can carry envelopes; main VOLUME and LEVEL are shared.

The prior `Accordion Study` is replayed with each crop's identical MIDI/gates for B in A/B/C:

| Reference register | Prior shared patch | Independent revision | Pitch errors C / E / G |
|---|---|---|---|
| 1 | 5.895 dB | 5.413 dB | -4.4 / +5.2 / -3.4 cents |
| 2 | 4.805 dB | 4.424 dB | -2.8 / +1.0 / -2.0 cents |

These aggregate chord-band improvements are modest, not exact acoustic matches. Individual
reed balance/tuning, bellows motion, stereo microphone perspective and room still differ.

## Delivery, replay and checks

`Flute Rise Study.toml`, `Accordion Register 1.toml` and `Accordion Register 2.toml`, plus both
flute alternatives, were exported normally and reloaded by exact file path. All revised
listening and new-phrase audio uses those delivered files. QUALITY HI, 48 kHz, seed 72,
120 BPM, semitone frequency dials.

| Preset | Reload spectral MAE, 512 / 2048 / 8192 | Envelope MAE |
|---|---|---|
| Flute Rise Study | 0.190 / 0.127 / 0.083 dB | 0.0058 dB |
| Accordion Register 1 | 0.372 / 0.173 / 0.104 dB | 0.0070 dB |
| Accordion Register 2 | 1.717 / 1.624 / 1.546 dB | 0.0294 dB |

Normal four-decimal quantization can shift phase-sensitive spectra; no sample identity is
claimed. The higher second-register reload difference is disclosed; its fit measurements
above use the actual delivered replay, not the pre-export candidate.

Each candidate is matched to its reference by MCP LUFS. One common boost/headroom limit
then applies to original, prior and all revised alternatives within that audition group.
No EQ, denoising or room effects. The complete source and new phrases are normalized
separately and labelled. The old report is archived with its validation assumption withdrawn.
No merge, release, library installation or video upload is performed.

This revision changes settings, artifacts and guidance, not the previously tested Rust/DSP
source. A read-only critic verified sample-exact source crops, distinct preset files/hashes,
render hashes, declared listening gains, held-note ratios, 44 local links and byte-identical
served assets. All 15 browser audio elements load with expected durations and no errors.
