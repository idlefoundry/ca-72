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

## Flute: first repeating-modulation study (superseded trigger behavior)

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
| 2 (later rejected by listening; see correction below) | 4.805 dB | 4.424 dB | -2.8 / +1.0 / -2.0 cents |

These aggregate chord-band improvements are modest, not exact acoustic matches. Individual
reed balance/tuning, bellows motion, stereo microphone perspective and room still differ.

## Delivery, replay and checks for the register/phrase-wide study

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

That revision changed settings, artifacts and guidance, not the previously tested Rust/DSP
source. A read-only critic verified sample-exact source crops, distinct preset files/hashes,
render hashes, declared listening gains, held-note ratios, 44 local links and byte-identical
served assets. All 15 browser audio elements load with expected durations and no errors.

## Follow-up: individual LFO RETRIG, legato main contours

The owner clarified that the flute should restart each LFO's RISE on new notes through its
own RETRIG mode, independently of the main envelope's RETRIGGER. The integrated 0.3.0 engine
had intentionally coupled them in the earlier R-LFO design. R-LFO-NOTE-RETRIG supersedes
that coupling for RETRIG sources: every note-on resets their phase and RISE. Shared ENV and
FREE's RISE retain their existing phrase/main-switch behavior, and newly assigned polyphonic
voices still start their own generators. The circuit contours and voice placement retain
the main switch's behavior. No analogue model was retuned.

The current delivery is `Flute Note Rise Study.toml`: individual LFO 1 RETRIG, 4.2 Hz,
1.2 s RISE, main RETRIGGER **off**. The original 7.7-second phrase ENV and legato contours
remain intact. A second exported file changes only RISE to 0.9 s. Global-RETRIGGER auditions
were discarded, not published as the correction. These preset files need the corrected
development engine; earlier 0.3 builds with coupled triggering behave differently.

The revised original/prior/new comparison uses the actual archived prior-engine replay for
B, and exact exported-file replay from the new engine for C. Whole-phrase slow-envelope RMS
error is slightly higher, 2.580 to 2.691 dB, while harmonic-band error is 3.914 to 3.919 dB.
The requested note articulation takes precedence over the aggregate objective. RISE is a
ramp, not a hard vibrato delay: brief notes still receive shallow movement.

A new audition uses A4 0.25 s, G4 0.55 s, F4 0.8 s and E4 2.4 s with 10 ms key overlaps.
Individual note resets therefore cannot be explained by silence between keys. On the long
note, the actual pitch contribution relative to the same preset at LFO 1 LEVEL zero grows
from approximately 1.72 cents early to 5.00 cents late; short-window periodic descriptors
remain approximate. Both RISE versions are rendered from their exported files and gain
matched together. Main-file export/reload spectral MAE is 0.189/0.125/0.083 dB and envelope
MAE 0.0058 dB. Source fingerprint:
`c182626605628a35e3c8d3844961d92b2d42c0be1bfaf960dcb22f5483913365`.

Both accordion presets regenerate with identical WAV hashes under the new engine; their
settings, published audio and report sections remain unchanged. The previous report is
archived. The personal library and installed application were not changed.

Validation: 572 plug-in library tests pass (28 existing ignored tests); all 5 production
stdio audio tests pass; default/all-feature workspace/all-target Clippy with warnings
denied; formatting and matching-skill validation. Tests cover independent phase/RISE,
mixed FREE/ENV/RETRIG sources, shared/voice-owned and mono/unison/poly paths, exact LO/HI
main-contour preservation, cross-modulation loop resets, and real offline RISE attenuation
then audible rebuilding after export/reload with overlapping keys and main RETRIGGER off.

## Second-register correction: reject the invented high tones

The owner approved the first accordion register and rejected the second for an added high
pitch absent from the original. The second preset's EMPHASIS 9 produced strong narrow tones
around 8.9, 11.0 and 13.0 kHz. The earlier H1–H8 objective missed most of that range; its
lower harmonic error did not establish an acceptable sound. The old second-register file
is superseded, and the accepted first register and current flute are unchanged.

Ablations isolate resonance: EMPHASIS 9 still produces the tones with oscillator 2 off;
lowering emphasis removes them. Twenty-three same-gate candidates check lower resonance,
cutoff and reed layers using one-twelfth-octave power bands (about 146 Hz–18.7 kHz) plus a
separate 6–20 kHz measure. All diagnostic spectra normalize once to 150–1000 Hz power,
so LUFS matching cannot hide an added whistle. These are diagnostics, not perceptual scores.

The replacement `Accordion Register 2 Corrected` uses CUTOFF +0.5, EMPHASIS 2.5, two saw layers
in the same octave with +0.06-semitone detuning, second-layer level 3, and the existing
per-voice bellows ENV. It removes the former upper-octave layer as part of the refit. No
audio EQ/notch, source processing, Rust change or analogue-model retuning is used.

Delivered-file steady-window power relative to the same low-band reference:

| Band | Original | Rejected preset | Corrected preset |
|---|---|---|---|
| 6–20 kHz total | -21.12 dB | +15.16 dB | -23.81 dB |
| 8.9 kHz ±60 Hz | -44.76 dB | +9.95 dB | -44.43 dB |
| 11.0 kHz ±60 Hz | -46.25 dB | +9.67 dB | -45.64 dB |
| 13.0 kHz ±60 Hz | -42.40 dB | +10.18 dB | -49.56 dB |

The corrected H1–H8 error is about 4.57 dB, versus 4.42 for the rejected fit. This small
metric advantage did not justify the unsupported tones. Actual replay diagnostics also
check attacks, late hold, release and a four-chord demonstration; the old dominant high
peaks are absent there too. Remaining upper partial/reed and room differences still need
the owner's listening review. The corrected example is not claimed an exact reproduction.

The normal preset export/reload has spectral MAE 0.262/0.156/0.086 dB and envelope MAE
0.0053 dB; the actual reloaded file supplies all revised listening/new-chord audio. The
original/corrected pair uses MCP LUFS matching followed by one common gain, and the new
chord sequence is normalized separately. The rejected version remains archived rather than
appearing as the current preset. First-register and flute preset/audio files and report
sections are preserved. The engine fingerprint remains the one recorded above.

## Flute harmonic refit with the approved expression retained

The owner approved the flute's envelopes and asked for a closer harmonic match. The same
original 7.7-second recording, E4–G#4–G4–A4–F4 gates, pitch, main contours, LFO 1's 4.2 Hz
triangle/1.2 s RISE/individual RETRIG, all existing volume/tune routes, phrase ENV settings
and noise input are retained. No Rust or analogue DSP is changed.

The selected `Flute Harmonic Study` uses a continuous oscillator shape of 0.37, CUTOFF +1,
EMPHASIS 3, keyboard tracking off and oscillator mixer level 9.6. Two additional routes
from the existing phrase LFO 3 move WAVEFORM by -3% and its mixer level by +4%. The gentle
shape movement toward 0.34 follows the phrase; mixer compensation limits the accompanying
change in loudness. No additional oscillator is used. Stronger co-pitched layers and larger
waveform movements were rejected for expression or other-note regressions.

The local search contains 199 rendered candidates. E4/A4 were the fitting windows. Other
notes were inspected during acceptance and must not be called untouched validation. The
final compromise improves both held notes while keeping the other errors close to the
approved replay. Measurements below use the actual exported-file replay:

| Harmonic-band RMS error | Approved tone | Harmonic refit |
|---|---|---|
| E4 | 4.848 dB | 4.064 dB |
| G#4 ornament | 7.328 dB | 7.284 dB |
| G4 ornament | 10.870 dB | 10.926 dB |
| A4 | 2.687 dB | 2.030 dB |
| F4 | 3.501 dB | 3.594 dB |
| E4/A4 fitting windows | 3.919 dB | 3.212 dB |
| All five, gate-duration weighted | 4.775 dB | 4.458 dB |
| E4/A4 full-spectrum band RMS error | 9.120 dB | 7.818 dB |

These are engineering diagnostics, not perceptual similarity percentages. G4 remains a
substantial mismatch. Interharmonic-energy RMS error is slightly worse, 0.929 to 1.833 dB;
individual intonation, breath detail and the room still differ. A larger held-note gain
(about 2.04 dB aggregate error) was rejected because ornaments/F4 worsened by about 2 dB.

Preservation is checked against the approved rendered waveform, rather than inferred from
the unchanged envelope knobs. After one common gain, 50 ms AC-RMS frames at 10 ms hops
differ by 0.112 dB RMS, 0.218 dB at the absolute-error 95th percentile. The mask is set by
the approved signal down to -45 dB of its peak AC RMS, including audible onset and release.
Separate note/onset/release intervals prevent a short ornament from disappearing within a
phrase-wide percentile. The worst interval RMS difference is 0.247 dB. Maximum approximate
early/late periodic AM/FM-depth changes are 0.012 dB and 0.025 cents. Declared conservative
preservation budgets were 0.5 dB RMS/1 dB p95 per interval and whole phrase, 0.2 dB AM and
0.75 cents FM; passing those budgets is not a listening approval.

All five steady windows, first attack, final release and a separate overlapping-note
performance receive 150 Hz–20 kHz peak/upper-band checks. No strong added nonharmonic tone
is identified; both fitting windows' 6–20 kHz energy is below the source. Alternate seeds
71 and 73 retain the held-note improvement (about 3.933 to 3.224/3.236 dB), with envelope
differences of 0.111/0.106 dB RMS from their respective approved-tone controls.

The short/long-note demonstration retains the previous A4/G4/F4/E4 performance and 10 ms
key overlaps. Its actual old/new envelope difference is 0.147 dB RMS. It checks performance
beyond the fitting sequence; no second acoustic reference is claimed. Original/previous/new
audition files use MCP LUFS matching followed by one common boost/headroom cap; the probe
pair uses its own common gain. No EQ, denoising, separation or reverb is applied.

Normal preset export/reload spectral MAE is 0.152/0.110/0.080 dB (512/2048/8192) and envelope
MAE 0.0054 dB. Source fingerprint remains
`c182626605628a35e3c8d3844961d92b2d42c0be1bfaf960dcb22f5483913365`.
The current report preserves both accordion sections byte-for-byte and provides the real
source, approved flute replay, new replay, new-phrase pair, per-note harmonic plots and
saved preset. The prior report and its 0.9-second RISE alternative are archived. No library
installation, merge, release or video upload is performed.

An independent read-only audit verified source/preset/replay hashes, declared listening gains
to 32-bit rounding, correct WAV formats/durations, preservation claims, both unchanged
accordion sections/files, all 42 local links and identical served copies. All 13 current
browser audio players load with expected durations and no errors. Matching-skill validation
and whitespace checks pass. This settings/artifact/guidance follow-up does not change the
previously tested Rust implementation.

## Native experiment workflow, 2026-10-21

The new `ca72_experiment` tool repeats the actual-recording exercise without a custom FFT,
metric or report loop. One production MCP call evaluates six flute candidates across seven
regions in about 11 seconds; another evaluates four second-register accordion candidates
across three regions in about eight seconds on the Linux development machine. Each batch
decodes/analyzes its reference once. These are descriptive timings, not a controlled speedup
comparison with the earlier search. The candidates are known variations of previous fits,
not fresh blind fits or a new validation corpus.

The original flute recording, approved revision-4 expression replay, performance, seed,
quality and dry conditions are unchanged. The batch includes a control, static waveform,
fixed filter, accepted balanced refit, lower resonance and stronger phrase shape movement.
Native uniform-weight H1–H8 RMS diagnostics illustrate the tradeoff:

| Region | Approved control | Accepted balanced refit | Stronger shape movement |
|---|---|---|---|
| E4 | 4.973 dB | 4.356 dB | 3.025 dB |
| G#4 | 8.747 dB | 7.146 dB | 8.552 dB |
| G4 | 9.446 dB | 10.137 dB | 11.842 dB |
| A4 | 4.362 dB | 2.147 dB | 1.879 dB |
| F4 | 4.484 dB | 3.899 dB | 5.434 dB |

The stronger held-note result worsens G4/F4. All three pass the explicit expression budgets;
that does not make their tone equally acceptable. The balanced refit's worst regional
envelope RMS change is 0.247 dB, approximate maximum AM/FM changes 0.006 dB/0.003 cents
under this method. Native diagnostics use uniform harmonic weights and their own documented
windows/normalization; their absolute values must not be compared with the earlier bespoke
weighted diagnostic as if its definition were unchanged.

The control and balanced refit reproduce the previous actual preset-replay WAV hashes
exactly, including after the boxed-engine host-stack fix. Exporting the batch snapshot by
`snapshot_file`, rendering its normal returned preset file and comparing with the batch WAV
has zero measured spectral error at 512/2048/8192 samples and zero envelope/pitch difference.
This continuation begins from already serialized settings; it does not establish that normal
preset quantization is universally lossless.

The accordion batch keeps the corrected second register and its C4/E4/G4 performance as the
baseline. Changing only EMPHASIS to 9 recreates the previously rejected extra high tones.
Its harmonic RMS looks slightly better (6.242 versus 6.283 dB); full-spectrum RMS also looks
better (14.132 versus 16.579 dB). Yet normalized upper-band difference from the actual
recording jumps from −3.518 to +35.448 dB, with 15 unexplained peak flags including roughly
6.0, 7.5 and 8.9 kHz. Its envelope changes by up to 6.080 dB regional RMS and 13.949 dB p95,
failing preservation. The control reproduces the corrected preset replay hash exactly.
The tool displays separate upper-band/peak evidence and preservation verdicts, and chooses
no automatic winner. Both accepted accordion presets and the accepted flute remain intact.

Durable local evidence is under `acoustic-matches/native-experiments-v4`: production
requests/results, complete experiment manifests, exact snapshots, region metrics, common-level
HTML auditions and the flute's export/reload check. All candidate auditions share the batch's
recorded listening gain/headroom cap after level matching; measurement normalization is
separate. No hidden EQ, alignment, denoising or effects are applied.

Checks for the implementation: 21 analysis tests, 572 plug-in library tests (28 existing
ignored), six production stdio audio tests, 14 CLAP host tests (one existing manual stress
test ignored), default/all-feature workspace/all-target Clippy with warnings denied,
formatting, skill validation and all three Linux installer-identity checks pass locally.
Synthetic checks include calibrated shallow AM/FM, strong-FM ambiguity, unsupported pitch
hints, DC-heavy channel selection, short-note regressions, added tails and unresolved budgets.
An independent critic rechecked the confidence fixes and native API paths. The 6 dB competing-
peak check is a conservative heuristic, not a universal carrier estimator; stronger AM can
hide a sideband. Requested unresolved measurements cannot pass.

Windows MSVC 1.97.1 reproduces the original 256 KB CLAP lifecycle stack overflow and passes
the unchanged guard with only `Ca72.engine` moved to a box. All 14 CLAP host tests and all six production MCP audio tests pass
with `standalone-production`. This is a source-built test host without a GUI/audio device;
it is not a claim of installed-DAW qualification. The normal three-OS PR checks remain the
merge gate. No tag/release, personal-library installation or video upload is part of this work.


The previous CI run's macOS CLAP validator also exposed incomplete 0.3 parameter-text
conversion: continuous waveform blends/named rectangle duties could not be typed back,
LFO RATE printed its Hz unit twice, and rate precision changed after parsing near 1/10 Hz.
The oscillator/LFO text inverse now accepts its displayed labels while retaining oscillator
numeric duty-percent and LFO bare numeric travel units. Rounded labels select a representative
within their display precision. Rate units are appended once and precision is stable across
rounded tier boundaries. No parameter defaults, values, formatter waveform labels or audio
processing change. The new real-CLAP regression checks every parameter's display roundtrip,
with 10,001 samples per shape parameter, plus numeric LFO compatibility/nonfinite refusal.
The script-pinned official CLAP validator's `param-conversions` check passes against a fresh
source-built debug library; full platform validators remain in CI. An independent 100,001-
sample sweep per shape parameter covered all 352 canonical labels with stable roundtrips.
