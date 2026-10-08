# Calibration against a hardware reference

**Owner decision, 2026-10-07.** The CA-72 is matched to the owner's Behringer Model D, a
modern recreation of the instrument: "as long as we have the right sound to calibrate to, we
can assume that this matches the Model D almost exactly in terms of the sound
characteristics ... if we match this, that this will be the right thing to do." In
side-by-side recordings, tuned and levelled the same, the owner could not tell it from a
Model D. This replaces the 2026-09-28 answer in [history.md](../history.md) that comparisons
with hardware would be qualitative.

**How a difference becomes a change** (agent decisions, 2026-10-07): the circuit changes,
never the output. A difference is traced to a component, a trim or a device model; the
netlist and the real-time model change together and stay within their budgets against
ngspice; the change is recorded below as a deviation from the drawing, with the
measurement behind it. Calibration (where a unit's trims sit) is copied; the reference's own
features (its LFO, patch points, high-pass mode, a second DECAY switch) are not targets.

## The rig

| Part | What |
|---|---|
| Interface | MOTU 828ES on the Mac, its own driver, 48 kHz. On Linux over class-compliant USB, playback started alongside a capture shifted the captured channels by 16 or scrambled them; capture alone was sound |
| Control | Two Expert Sleepers ES-3 on the interface's optical outputs: DC-coupled, sample-locked with the inputs. ES-3 #1 outputs 1 to 5 (computer channels 9 to 13) drive LC GATE, FC GATE, OSC 1 V/OCT, CUT CV and EXT; output 6 (14) is looped back into analog input 5 (computer input 7) |
| Recorded | The reference's main output (analog input 1, computer input 3; the rear HIGH or the front 3.5 mm MAIN, to be confirmed) and its MIX jack (analog input 3, computer input 5) |
| Stream | `sounddevice.playrec`: stimuli out and responses in on one clock. The driver's latency (12,387 samples) is measured on every take from a timing mark on the loopback and removed |
| Interface's input | A first-order high-pass of 0.94 Hz (169.5 ms, from a DC step on the loopback): renders are passed through it before a comparison of slow events (the VCA's thump) |
| Scale | The ES-3's full scale is about 10 V (an octave per 0.1 of full scale measured 1.042 octaves on the reference's V/OCT input, so 10.42 V if that input is exact). Comparisons that need volts carry this ±4 % |

The listening page of 2026-10-08 is `listen/2026-10-08/index.html` there. Captures and
renders are kept outside the repository (they are large; a capture's JSON has
its WAV's SHA-256): `lab/artifacts/ca-72/calibration/` on the lab's share, with the panel's
photographs and the interface's settings.

## Method

`scripts/calibration/` (Python 3 with numpy, scipy, soundfile, sounddevice):

- `plans.py`: the takes, each a set of stimuli in fractions of the ES-3's full scale and a
  record of its segments. `session_a` is what the gates and voltages reach with the panel
  left at its home settings (oscillators and noise off, EXTERNAL INPUT on at VOLUME 5,
  CUTOFF fully clockwise, EMPHASIS and AMOUNT OF CONTOUR at 0, KEYBOARD CONTROL off, both
  contours at ATTACK 0, the DECAY switches off); `session_b` its follow-ups.
- `capture.py PLAN OUTDIR`: plays and records each take; fails a take that clips, whose
  timing mark is missing, or whose idle inputs are not quiet (a shifted channel).
- `ca72-lab stim TAKE.wav PATCH.json OUT.wav`: plays the same stimuli into the CA-72's voice
  (EXT at the EXTERNAL INPUT jack, CUT at the FILTER CONTROL jack through R51, LC GATE as
  EXT. S-TRIG).
- `analyze.py`: the same measures of both, each of the main output or of the main output
  against the MIX (for the CA-72, its mixer bus current), so that the two compare whatever
  their absolute levels: harmonics against level, the filter's transfer, its corner and
  slope, the loudness contour's attack, held level and release, the thump.
- `figures.py` and `listen.py`: the plots and a listening page (one HTML file, its clips
  levelled by K-weighting, the renders passed through the interface's high-pass). For a
  comparison by ear, `ca72-lab stim --gate-delay 2.3,5.9` delays S-TRIG as the reference's
  gate input delays LC GATE.

The panel is fitted where its knobs cannot be read exactly: the CA-72's loudness SUSTAIN
0.60 and DECAY 0.07 hold and decay as the reference did (-4.94 against -4.93 dB at 100 ms,
-6.56 against -6.44 dB at 300 ms, 44.9 against 44.8 ms), and its EXTERNAL INPUT VOLUME 0.375
drives the filter as the reference's 5 does (the third harmonic against level within 0.5 dB)
(`patches/session_a_fitted.json`).

## Results, 2026-10-08 (sessions A and B)

| Behaviour | Reference | CA-72 0.1.3 | Verdict |
|---|---|---|---|
| Filter, EMPHASIS 0: response against its corner | -0.6, -1.2, -3.0, -6.7, -13.1, -21.0, -30.8 dB at 0.5, 0.7, 1, 1.4, 2, 2.8, 4 times the corner | -0.4, -1.1, -3.0, -6.8, -13.4, -21.5, -31.5 | The same within 0.7 dB at every corner from 600 Hz to 3.6 kHz; slope 2 to 4 times the corner -17.8 against -18.1 dB an octave |
| Filter driven hard (drive matched) | passband -0.43 to -0.54 dB; corner 0.92 to 0.95 of its clean value | -0.42 to -0.47 dB; 0.89 | Close; the corner's fall with drive to be checked with the oscillators |
| Corner against CUT CV, CUTOFF fully clockwise | 19.0 kHz at 0 V; a factor of 1.43 each 0.5 V; 606 Hz at -5 V | 15.8 kHz at 0 V through R51; 546 Hz at -5 V | 0.15 to 0.26 octave lower. Which of CUTOFF's span, R39, R49 and the jack's resistor accounts for it needs CUTOFF moved and the keyboard's tracking: the knob session |
| Loudness contour, ATTACK 0 | 10 to 90 % in 1.0 ms, 2.3 ms after LC GATE | 0.8 ms, at once (S-TRIG) | The same; the reference's delay is its gate input's (it ignores gates under 3 ms) |
| Release with DECAY off, one contour charged | -6 to -40 dB in 9.9 ms, starting about 6 ms after LC GATE falls | 9.1 ms, at once | The same shape (R1401 1.5K as drawn); the delay is the reference's gate input's |
| Release with DECAY off, both contours charged | as with one: 21.33 against 21.29 ms to -40 dB | twice as slow: 32.3 against 14.3 ms to -40 dB | **Different: changed (below)** |
| The VCA's thump, its shape (through the interface's high-pass) | peak at 27 ms, a dip to -0.23 of it at 251 ms; the release's 0.37 of the attack's | peak at 44 ms, dip -0.28 at 295 ms; 0.77 | Alike, the CA-72's slower and its release thump larger. Its size against a note's needs the oscillators: the knob session |
| Interface | | | A first-order 0.94 Hz high-pass at its inputs |

## Changes

1. **Each contour dumps through an R1401 of its own** (`ContourCircuit::dump_each`,
   2026-10-08). The left hand controller's one DECAY switch dumps both timing capacitors
   through CR7, CR4 and one R1401 1.5K into V-trig (Figure 9-12; S-ERR's correction does not
   change it), so with DECAY off a contour held higher slows the other's release: in the
   CA-72, both contours held, the loudness release took 32.3 ms to fall 40 dB against 14.3
   ms with the filter's empty. The reference, which has a DECAY switch for each contour,
   releases the loudness contour as fast whatever the filter contour holds (FC GATE pulsed
   with LC GATE or not, gates of 5 ms to 300 ms: within 0.3 ms). With an R1401 for each, the
   CA-72 falls -6 to -40 dB in 9.5 ms with both held (reference 9.9 ms); with the reference's
   gate delays applied, it crosses -6, -12, -20, -30 and -40 dB within 0.2 ms of the
   reference (0.1.3: 9.7 to 16.9 ms later). One DECAY switch still serves both. The drawing's arrangement is kept (`dump_each: false`) and tested
   against ngspice beside the new one. It changes only presets with DECAY off (Pulse Strut,
   Cruising Whistle, Slow Bow, Pink Riser); the others render the same to the bit.

## Not changed, and why

- **R1401's value.** With one contour charged the release already has the reference's
  shape; the factor of two was the shared resistor.
- **The trigger delay** (board2.md, B2-3: 8 ms from a key, 12 ms after its release). The
  reference's gate input adds 2.3 ms at a gate's rise and about 6 ms at its fall, and needs 3
  ms of gate: its own processing, not its contour's. What a MIDI note takes on the reference
  is measured in the knob session, with MIDI sent through the interface.
- **The thump** (B4-6): its size against a note's level, which the oscillators will give.
- **The corner's mapping:** see the table.

## Next: the knob session

Each needs the panel moved, so the owner at the instrument; the plans run each take, and the
analysis and the fits are ready for them:

1. The oscillators: each waveform's shape and level at 8', the ranges' octaves, OSC 3's
   reverse sawtooth, FREQUENCY's span, the mixer's VOLUME taper; MIX recorded beside MAIN.
2. The mixer's overdrive: one, two and three oscillators at VOLUME 2 to 10 into the filter.
3. EMPHASIS at 0, 2.5, 5, 7.5 and 10, and self-oscillation, against CUT CV (the regeneration
   and range trims); CUTOFF at its printed marks; KEYBOARD CONTROL 1, 2 and both.
4. The contours' ATTACK and DECAY at their printed marks; SUSTAIN at 0, 5 and 10; the
   contour outputs into the modular's DC-coupled inputs, which return on computer inputs 11
   to 26.
5. MIDI from the interface's MIDI output (a cable to the reference's MIDI IN): a note's delay
   to the attack and to the release, against the CA-72's 8 and 12 ms.
6. EXTERNAL INPUT VOLUME at 2, 4, 6, 8 and 10 (two points so far: 10 to 5 is -30.7 dB on the
   reference, -20 dB on the CA-72's assumed taper).

## Reproducing

```
# On the Mac, in a terminal with the microphone permission:
cd scripts/calibration && python capture.py session_a ~/lab/artifacts/ca-72/calibration/captures/<date>-A
# Anywhere:
cargo run -p ca72-lab --release -- stim <take.wav> scripts/calibration/patches/home_ext5.json <render.wav>
python scripts/calibration/analyze.py <take.json> [--render <render.wav>]
```
