# High-note aliasing, October 2026

Measurement for [issue #67](https://github.com/idlefoundry/ca-72/issues/67), made on
18 October 2026 (Toronto; the logs' UTC date is 19 October). No sound-producing code was
changed. This is a controlled reproduction, not a reconstruction of the reviewer's patch:
the report cited in the issue supplied no recording, settings, notes, rate or version.

**The multicore release does not fix the measured aliasing:** every engine sample in this
matrix is bit-identical between v0.2.5 and main/0.2.6. Fidelity substantially reduces the
oscillators' foldbacks; the complete voice retains residual aliasing, so "removes aliasing"
would overstate the result. Audible-band components are established; subjective audibility
still needs a listening comparison.

## Revisions and scope

| Build | Commit | Meaning |
|---|---|---|
| v0.2.5 | `212967e3efe0945c60c44fa4272f51a854d5f3eb` | Published tag, peeled to its commit |
| main | `1a1dd6cd2c241f3f2fa94e79ab93cdd489e4a219` | GitHub main, 0.2.6, including the multicore work |
| fidelity | `9b0f9e8e3bbe3f4a51567475311423f61ece5b55` | [PR #87](https://github.com/idlefoundry/ca-72/pull/87), R-BLEP and related fidelity work |

These are frozen revisions, rather than whichever tips the branches have when this is read.
The tag object for v0.2.5 is `1f3d9cc`; the table gives the actual source commit.

The main measurement uses `ca72_plugin::engine::Engine`, as `preset_render` does, in ordinary
offline 256-frame calls, with no workers or deadlines. No host, audio device, installed bundle
or real settings folder was used. Each revision was built with the same measurement example
in its own detached worktree and target directory, with four Cargo build jobs, at nice 19.
The hardware was an x86-64 Ryzen 7 7800X3D on Linux; Rust 1.97.1.

There are 162 engine cases per build: three rates (44.1, 48 and 96 kHz), three modes
(LO, HI, ULTRA), three notes (C6/C7/C8, MIDI 84/96/108; nominally 1046.50/2093.00/4186.01 Hz),
and six oscillator-1 waveforms (triangle, shark tooth, sawtooth, square, wide rectangle and
narrow rectangle). The rectangles use the actual switch positions, not assumed ideal duty
cycles. Oscillator 3's reverse saw is outside this matrix.

The patch is oscillator 1 at 8', VOLUME 8, the other oscillators off; CUTOFF +5, EMPHASIS 0,
AMOUNT OF CONTOUR 0, both KEYBOARD CONTROL switches off, low-pass filter; LOUDNESS ATTACK 0,
DECAY 0, SUSTAIN 10; GLIDE off, both modulation switches off, wheels centred/down, noise,
external input, A-440 and FEEDBACK off. LOCK is on, ENTROPY, DRIVE, SPREAD and DOUBLE are zero,
POLY and UNISON off, MAIN OUTPUT on at 10, LEVEL 0, AUTO GAIN off, POWER on. Other controls
are `Controls::default()`; the committed example is the exact patch specification. The seed
is 1. LO uses the light engine; HI uses the circuit engine with 1x oscillators and a 4x ladder;
ULTRA uses 4x oscillators and ladder. The baseline's VCA is at 1x in every mode; fidelity
changes ULTRA's VCA to 4x as well as changing oscillator band-limiting.

A further 108 cases per build render the circuit oscillator alone in HI's 1x and ULTRA's 4x
configurations. These use the ideal keyboard input and exact exponential converter, tuned by
`folkman_1973`, rather than the engine's calibrated keyboard bus and HI converter table.
They diagnose the oscillator separately; they are **not taps of the engine's oscillator**.
The shark tooth uses the switch's unloaded resistor mixture. An isolated LO oscillator API
is absent in v0.2.5/main; LO is measured at the complete engine output instead.

## Spectral method

Each case holds its note for one second, then retains 131,072 steady samples: 2.972 s at
44.1 kHz, 2.731 s at 48 kHz, 1.365 s at 96 kHz. The engine's f32 stereo samples are averaged
to mono and stored as little-endian f64; oscillator diagnostics are f64 throughout. The
storage conversion adds no precision to the engine output.

The analyser removes the weighted DC mean and applies the seven-term Blackman-Harris window
used by fidelity's `ca72-lab/src/alias.rs`. FFT bin widths are 0.3365/0.3662/0.7324 Hz.
It estimates the actual fundamental with an 8x zero-padded FFT and logarithmic peak
interpolation within 3% of the nominal note. Oscillator diagnostics instead use frequency
from reset timing, cross-checked against that estimate. A difference greater than a quarter
of a native FFT bin fails the run. The fundamental estimated in each half of the segment
must also agree within a quarter bin; drift figures are saved per case.

All integer multiples of the **measured** fundamental are intended harmonics, including even
harmonics of the circuit's imperfect triangle and square. Bins within 8.5 native FFT bins of
each harmonic are assigned to that harmonic. This includes the window's main lobe and a
margin. Everything else from 20 Hz through 20 kHz is inharmonic power. The main ratio is:

`10 log10(inharmonic power / summed intended-harmonic power in 20 Hz–20 kHz)`.

This is an aliasing estimate **with a noise floor**, not a claim that every non-harmonic bin
is an alias. Model hiss, numerical noise, any unsettled gain and unrecognised modulation
would also enter it. LOCK and the explicit unmodulated patch remove intended pitch drift
and modulation as explanations here; the half-segment check is a stability guard, not a
general FM/AM detector. The TSV also gives the ratio to the fundamental alone, allowing
comparison with R-BLEP's earlier figures, and a separate 20 Hz–10 kHz residual ratio.

For attribution, inspect local peaks outside the harmonic masks and compare their frequencies
with `abs(((m*f0 + fs/2) mod fs) - fs/2)` for integer source partials above Nyquist through
partial 256. A match must be within 1.5 native bins and the line must stand at least 15 dB
above its neighbouring spectral floor. The sum of matched lobes among the strongest 100
peaks is saved as `identified_alias_db_signal`: **identified foldback candidates**, not an
exhaustive total. It misses weaker, broad or overlapping aliases; frequency coincidence is
supporting evidence, not a proof by itself. Rate-dependent movement and the isolated
oscillator comparison provide additional evidence. Each case's eight strongest peaks,
levels and candidate source partials are in `results.json`.

Aliases that fall inside a harmonic mask, including ones exactly coincident with a harmonic,
are inseparable by this method and excluded. No waveform subtraction or frequency-independent
"difference from the old build" is labelled aliasing. DC, sub-20-Hz and above-20-kHz energy
are not scored. Being in 20 Hz–20 kHz does not establish that a particular listener hears it.

The synthetic self-test rejects silent, non-finite and short input, puts clean harmonic
leakage below -145 dB, and recovers a known -40 dB folded partial within 0.02 dB. Complete,
unique render matrices and settings/source hashes are checked before analysis.

The candidate detector additionally requires integrated lobe power above -140 dB relative to
summed harmonic signal, clear of the synthetic leakage floor. A `null` candidate level or
`false` audible-band candidate flag means no line passed these detection rules; it does not
mean zero aliasing. The band flag describes frequency location, not subjective audibility.

## Results

### The multicore comparison

All 162 engine renders and all 108 oscillator diagnostics in main are identical to
v0.2.5 to the byte. Their spectra and aliasing estimates are therefore identical. In this
matrix, 0.2.6's multicore work has **zero effect on aliasing**. The existing split/unsplit
voice equivalence test also passes. This does not measure host dropouts or worker deadlines.

### Complete voice: worst cases

Levels below are total 20 Hz–20 kHz inharmonic power relative to summed intended harmonic
power, dB; more negative is lower. Each baseline worst is selected from all 18 notes/waves
for that rate and mode. The next column compares **that same case** in fidelity; the final
column gives fidelity's own worst, which can be a different waveform or note.

| Rate, kHz | Mode | Baseline worst case | v0.2.5 / main | Fidelity, same case | Fidelity's worst in the group |
|---|---|---|---:|---:|---|
| 44.1 | LO | C8 saw | -30.5 | -62.9 | -55.4, C8 wide |
| 44.1 | HI | C8 saw | -28.2 | -44.6 | -32.2, C8 narrow |
| 44.1 | ULTRA | C8 narrow | -32.1 | -40.0 | -39.5, C8 wide |
| 48 | LO | C8 narrow | -31.8 | -71.2 | -65.2, C7 wide |
| 48 | HI | C8 narrow | -30.8 | -36.6 | -36.6, C8 narrow |
| 48 | ULTRA | C8 narrow | -36.6 | -45.4 | -44.5, C8 wide |
| 96 | LO | C8 narrow | -48.2 | -85.3 | -68.5, C8 wide |
| 96 | HI | C8 shark | -50.1 | -68.9 | -54.2, C8 narrow |
| 96 | ULTRA | C8 narrow | -53.8 | -53.9 | -52.7, C8 wide |

The residual is dominated by discrete foldback candidates in the strong baseline cases.
For example, main's HI C8 saw at 44.1 kHz has `f0 = 4184.3881 Hz`; its strongest lines are
18,993.63 Hz at -29.9 dB relative to harmonic signal (folded partial 6), 14,809.12 Hz at
-34.6 dB (partial 7), 10,624.95 Hz at -40.6 dB (partial 8), and 6,440.45 Hz at -48.5 dB
(partial 9). The measured frequencies follow the folding formula; they are not integer
multiples of f0. They lie in the scored audible band, including components well below its
upper edge. The total and identified-candidate estimates are both -28.23 dB for this case
to two decimal places.

ULTRA's baseline C8 narrow at 44.1 kHz also has lines at 2,256.28 Hz (-49.9 dB) and
6,440.45 Hz (-48.0 dB), alongside its stronger 19.0/14.8/10.6 kHz foldbacks. Its 20 Hz–10 kHz
residual is -45.2 dB. Fidelity's C8 wide ULTRA case still has lines at 2,256.28 Hz (-51.5 dB)
and 1,928.23 Hz (-53.2 dB); its below-10-kHz residual is -47.7 dB. Thus an upper-hearing-limit
argument alone does not dispose of the measured pulse residuals.

### Every waveform at 48 kHz

Each cell is **main → fidelity**, dB, for the same complete-voice case. The full three-rate
matrix, actual fundamentals, drift checks and component locations are in the linked data.

**LO**

| Waveform | C6 | C7 | C8 |
|---|---:|---:|---:|
| Triangle | -53.8 → -111.6 | -45.0 → -103.8 | -35.0 → -102.5 |
| Shark tooth | -49.6 → -100.3 | -45.1 → -98.3 | -35.2 → -92.1 |
| Saw | -41.4 → -72.7 | -38.4 → -69.7 | -33.7 → -69.6 |
| Square | -43.4 → -75.0 | -40.4 → -69.7 | -34.7 → -76.9 |
| Wide rectangle | -42.2 → -71.9 | -38.7 → -65.2 | -35.7 → -66.5 |
| Narrow rectangle | -41.0 → -73.9 | -36.9 → -79.2 | -31.8 → -71.2 |

**HI**

| Waveform | C6 | C7 | C8 |
|---|---:|---:|---:|
| Triangle | -55.4 → -69.2 | -45.5 → -67.3 | -34.6 → -60.0 |
| Shark tooth | -50.7 → -67.9 | -43.2 → -66.3 | -34.1 → -63.7 |
| Saw | -39.9 → -50.6 | -36.9 → -48.4 | -31.9 → -45.5 |
| Square | -42.0 → -51.1 | -39.5 → -48.1 | -33.2 → -42.7 |
| Wide rectangle | -40.9 → -49.0 | -37.5 → -46.1 | -34.2 → -40.6 |
| Narrow rectangle | -40.2 → -46.6 | -36.8 → -44.5 | -30.8 → -36.6 |

**ULTRA**

| Waveform | C6 | C7 | C8 |
|---|---:|---:|---:|
| Triangle | -79.7 → -81.4 | -74.3 → -80.4 | -61.3 → -74.5 |
| Shark tooth | -75.1 → -76.3 | -71.8 → -75.9 | -64.0 → -72.4 |
| Saw | -50.5 → -72.9 | -48.3 → -71.3 | -45.5 → -73.1 |
| Square | -50.9 → -53.5 | -47.9 → -50.2 | -42.8 → -47.0 |
| Wide rectangle | -48.8 → -52.2 | -45.8 → -49.1 | -40.5 → -44.5 |
| Narrow rectangle | -46.3 → -50.7 | -44.2 → -49.5 | -36.6 → -45.4 |

Fidelity lowers this ratio in 155 of 162 engine cases, by as much as 67.5 dB. Seven ULTRA
pulse cases at 96 kHz have a small increase instead: C6/C7/C8 square (+0.47/+0.59/+1.27 dB),
C6/C7/C8 wide (+0.29/+0.07/+1.14 dB), and C6 narrow (+0.10 dB). These ratios describe changed
harmonic spectra as well as residual power; they are not listening verdicts. LO and HI use
different models and nonlinear paths, which matters when comparing their aliasing scores.

### Separate oscillator diagnostics

All 108 oscillator cases improve with fidelity. The worst remaining waveform is the C8
triangle at every rate/mode; it still has measurable residuals. The saw improvements are
much greater: at C8/44.1 kHz, HI goes from -25.6 to -101.4 dB and ULTRA from -64.5 to
-118.3 dB. At C8/48 kHz, those pairs are -28.8 → -106.2 and -66.8 → -121.2 dB. R-BLEP's
oscillator band-limiting therefore addresses a demonstrated source of the problem.

| Rate, kHz | Configuration | Worst v0.2.5 / main diagnostic | Worst fidelity diagnostic |
|---|---|---|---|
| 44.1 | HI, 1x | -25.6, C8 saw | -60.4, C8 triangle |
| 48 | HI, 1x | -26.9, C8 narrow | -62.1, C8 triangle |
| 96 | HI, 1x | -49.4, C8 shark | -68.7, C8 triangle |
| 44.1 | ULTRA, 4x | -62.0, C8 shark | -72.8, C8 triangle |
| 48 | ULTRA, 4x | -64.2, C8 shark | -73.5, C8 triangle |
| 96 | ULTRA, 4x | -78.7, C8 saw | -88.2, C8 triangle |

These diagnostics and the complete voice are different paths, not a controlled subtraction
of two stage taps. In particular, #87 also changes ULTRA's VCA rate and timing. This run does
not apportion every output improvement to BLEP alone. A useful next measurement is at the
mixer bus, filter output and VCA output for the remaining rectangles. The source has
signal-dependent rectangle loading in `waveform_source` before the 4x ladder, a plausible
contributor that this experiment has **not isolated**.

### Listening examples and plots

The scripts select the worst baseline engine case in each mode, the worst 96 kHz engine
case, and the worst isolated HI oscillator case. Each has all three builds rendered at the
same settings. These are steady excerpts, without note attacks or releases.

| Case | Spectrum | v0.2.5 WAV | Main WAV | Fidelity WAV |
|---|---|---|---|---|
| LO, C8 saw, 44.1 kHz | [PNG](aliasing-2026-10/engine-LO-44100-108-saw.png) | [WAV](aliasing-2026-10/engine-LO-44100-108-saw-v0.2.5.wav) | [WAV](aliasing-2026-10/engine-LO-44100-108-saw-main.wav) | [WAV](aliasing-2026-10/engine-LO-44100-108-saw-fidelity.wav) |
| HI, C8 saw, 44.1 kHz | [PNG](aliasing-2026-10/engine-HI-44100-108-saw.png) | [WAV](aliasing-2026-10/engine-HI-44100-108-saw-v0.2.5.wav) | [WAV](aliasing-2026-10/engine-HI-44100-108-saw-main.wav) | [WAV](aliasing-2026-10/engine-HI-44100-108-saw-fidelity.wav) |
| ULTRA, C8 narrow, 44.1 kHz | [PNG](aliasing-2026-10/engine-ULTRA-44100-108-narrow.png) | [WAV](aliasing-2026-10/engine-ULTRA-44100-108-narrow-v0.2.5.wav) | [WAV](aliasing-2026-10/engine-ULTRA-44100-108-narrow-main.wav) | [WAV](aliasing-2026-10/engine-ULTRA-44100-108-narrow-fidelity.wav) |
| LO, C8 narrow, 96 kHz | [PNG](aliasing-2026-10/engine-LO-96000-108-narrow.png) | [WAV](aliasing-2026-10/engine-LO-96000-108-narrow-v0.2.5.wav) | [WAV](aliasing-2026-10/engine-LO-96000-108-narrow-main.wav) | [WAV](aliasing-2026-10/engine-LO-96000-108-narrow-fidelity.wav) |
| Isolated HI oscillator, C8 saw, 44.1 kHz | [PNG](aliasing-2026-10/oscillator-HI-44100-108-saw.png) | [WAV](aliasing-2026-10/oscillator-HI-44100-108-saw-v0.2.5.wav) | [WAV](aliasing-2026-10/oscillator-HI-44100-108-saw-main.wav) | [WAV](aliasing-2026-10/oscillator-HI-44100-108-saw-fidelity.wav) |

The 15 WAVs are mono IEEE float32 at their original rates, RMS-matched to -18 dBFS before
20 ms end fades. They are finite and unclipped; matching after the fades is within 0.001 dB.
The gain code would reduce all three together if any exceeded -1 dBFS peak; no such reduction
was needed here. No extra quantisation floor from 16-bit PCM was added. The plots analyse
the **unfaded raw** samples, with per-bin power relative to the **integrated fundamental**;
their peak heights consequently differ from the integrated component/summed-harmonic ratios
in the tables. Red marks every bin outside harmonic bands, including noise, not just aliases.
All five PNGs were visually checked.

### Recommendation and audibility

Treat issue #67 as a reproducible aliasing concern in the documented high-note/open-filter
patch. The 44.1/48 kHz LO and HI baselines have the strongest oscillator-related contamination;
ULTRA lowers much of it, particularly on the saw, while its pulse outputs still show substantial
residuals. At 96 kHz the worst baseline engine ratios are about -48 to -54 dB, lower than at
44.1/48 kHz. The multicore release did not change these offline spectra.

**Audible-band aliasing is confirmed; perceptual audibility is not established by this run.**
The -28 to -35 dB total residuals, and discrete 2–10 kHz lines tens of dB below the tone, make
audibility plausible in isolated high notes. This is an inference from spectra, not a claim
to have heard the examples. Hearing range, playback level and masking matter; much of the
largest energy is above 10 kHz. Use the supplied matched examples for an owner listening
comparison, including the residual pulse cases, before describing severity to users.

Keep #87's band-limiting as a demonstrated remedy for oscillator foldbacks. Describe the
complete candidate as **reducing** aliasing, not eliminating it. Confirm the pulse residuals
and the seven 96 kHz ULTRA ratio increases before making a broader claim. Choosing whether
to ship its sound-changing work, and in which release, remains the owner's decision. This
measurement branch changes no synthesis behaviour and makes no release decision.

## Re-running and data

From the repository root, on Linux with Git, Rust and Python available:

```sh
git fetch github main fidelity --tags
export CARGO_TARGET_DIR=$PWD/target CARGO_BUILD_JOBS=4
python3 -m venv target/aliasing-venv
target/aliasing-venv/bin/pip install -r scripts/measurements/aliasing-requirements.txt
target/aliasing-venv/bin/python scripts/measurements/aliasing.py self-test
target/aliasing-venv/bin/python scripts/measurements/aliasing.py render --work target/aliasing
OPENBLAS_NUM_THREADS=1 target/aliasing-venv/bin/python scripts/measurements/aliasing.py analyse \
  --work target/aliasing --out docs/measurements/aliasing-2026-10
```

The renderer is [aliasing_render.rs](../../crates/ca72-plugin/examples/aliasing_render.rs),
the runner/analyser [aliasing.py](../../scripts/measurements/aliasing.py). The runner pins the
three commits, refuses tracked changes in comparison worktrees, copies only this example,
uses each worktree's own target directory and an isolated `IDLE_FOUNDRY_SHARED`, and builds
with `--locked --release`. It never installs a bundle. A completed run can be resumed; an
interrupted raw directory must be moved aside or a fresh `--work` directory chosen. A
manifest is written only after all 270 renders for a build complete. The raw files and build
logs remain in `target/aliasing/`; they are regenerated rather than added to Git.

Committed data:

- [results.tsv](aliasing-2026-10/results.tsv): all 810 cases, including both inharmonic ratios,
  identified-candidate ratio, fundamental, half-segment drift, strongest peak and its
  frequency, audible-band candidate flag, and SHA-256 of the raw samples.
- [results.json](aliasing-2026-10/results.json): the same rows plus the eight strongest
  inharmonic components, their integrated levels and candidate folded partial numbers.
- [builds.json](aliasing-2026-10/builds.json): commits, renderer/analyser, executable and
  Cargo.lock hashes, tool versions, sample count, block length, seed and platform.
- [comparisons.json](aliasing-2026-10/comparisons.json): counts of exact old/main matches.
- [examples.json](aliasing-2026-10/examples.json): listening files and their gains/RMS levels.

## Validation and limits

- The three release builds and all 810 renders completed. Both the complete engine and
  separate oscillator diagnostics use unchanged production sources at their pinned commits.
- `cargo test -p ca72-plugin --no-fail-fast`: 291 passed, zero failed, 31 ignored. This includes
  `the_split_voice_with_drive_is_the_voice`, the existing regression that compares a split
  ULTRA voice with its unsplit counterpart. The spectrum matrix itself uses no workers;
  it is not a new exhaustive multicore matrix or a real-time dropout test.
- `cargo clippy -p ca72-plugin --all-targets -- -D warnings` and `cargo fmt --all -- --check`
  passed. The synthetic FFT self-test passed, including the detection floor guard.
- One full `cargo test --workspace --no-fail-fast` run, with `RUST_TEST_THREADS=4` and
  ngspice 47 available, passed: 494 passed, zero failed, 56 ignored (default features).
- An independent source review checked the quality/pitch mappings, harmonic masks,
  candidate attribution, matrix integrity, and example matching. Its corrections are in
  the committed scripts: float WAVs, accurate plot annotations, complete matrix checks,
  tracked-worktree checks and explicit attribution limits.

Not tried: human listening or blind ABX, the reviewer's actual patch, live host playback,
macOS/Windows renders, multiple notes together, unlocked drift, modulation, high resonance,
feedback or other drive levels. The results describe this steady single-oscillator patch;
they do not rate every factory preset or prove perceptual transparency. No installed plug-in,
real settings or audio device was changed.
