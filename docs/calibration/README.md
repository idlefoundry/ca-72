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
| Scale | The ES-3's full scale is 10.39 V. The reference's manual gives its oscillators' control input 1 V an octave; they moved 1.0407, 1.0386 to 1.0393 and 1.0381 octaves per 0.1 of full scale (session C), and its filter's self-oscillation 1.035 under CUT CV (session E): the ES-3 runs 3.9 % over 10 V. `ca72-lab stim --volts-fs 10.39` |

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
2. **The oscillators' external control input, R38, R63 and R144: 51.1K to 50.5K**
   (`expo::R_EXT`, 2026-10-09). The keyboard reaches each oscillator through R27 51.1K and
   the factory's scale trim makes its keys exact; the rear jack's resistor, the same 51.1K,
   then gives 0.987 octaves a volt (measured on the model, session C's take
   `03_osc1_sawtooth`). The reference's control input is 1 V an octave (its manual; its three
   oscillators 1.0016, 0.9996 to 1.0003 and 0.9991 with the ES-3 at 10.39 V). 50.5K (1 %,
   E192) gives 0.998: 1.0374 octaves per 0.1 of full scale against 1.0407 (oscillator 1)
   and 1.038 to 1.039 (2 and 3), where 51.1K gave 0.987. The keys stay exact: the factory
   tuning runs on the model as before. An empty jack's bus now rests at +0.067 V (R156 33K
   against the three 50.5K), not +0.107 V; the tuning absorbs it. The plug-in has no jack
   (R3), so this changes what a plugged jack does (the DAW's device, `ca72-lab stim`), and
   the presets only by the tuning's re-solve.
3. **The filter's RANGE trim, R39: 0.632 to 0.559** (`filter_cal::CALIBRATED`, 2026-10-09).
   With the trims of Folkman's procedure the CA-72's filter sat 0.32 to 0.34 octave below
   the reference's at the same control voltage, self-oscillating at EMPHASIS 10 from 22.9 kHz
   down to 950 Hz (session E, CUTOFF fully clockwise, CUT CV 0 to -5.2 V), and 0.28 octave
   below it at CUTOFF's centre mark (session E's corners, through the reference's own ratio of
   oscillation to corner, 1.578). The slopes agree: 10.35 and 10.41 octaves per full scale of
   CUT CV. RANGE is a trim, so the reference's setting is copied. With R39 at 0.559 the
   whole voice's corner is within -0.05 to +0.03 octave of the reference's along session A's
   CUT CV sweep (before: -0.26 to -0.35) and within 0 to +0.08 at the centre mark (before:
   -0.23 to -0.29). Folkman's trims stay the procedure's result (`FACTORY`) and its test.
   Every patch's filter opens about 0.31 octave higher than in 0.1.3: the presets' top
   bands rise 3 to 10 dB, their levels within 0.3 dB (Ladder Kick +1.0 dB). Two presets
   sound the filter's own pitch, which rises by the same 3.7 semitones: Cruising Whistle
   and Ladder Kick (not retuned: the owner's call).
4. **EXTERNAL INPUT VOLUME's law** (R9, `voice::ext_taper`, 2026-10-09). The voice's
   gain from the jack to the mixer, against VOLUME 10, fell -36.5, -28.4, -25.7, -23.4 and
   -18.6 dB at 2, 4, 5, 6 and 8 with the generic audio taper and the preamplifier's input
   (97K) loading the wiper; the reference's falls -39.5, -33.1, -31.3, -29.9 and -16.9 dB
   (session H, the mixer's MIX against the loopback, 1 kHz in the clean region). R9's track
   fraction at each mark is solved for the reference's gain through the same loading (0.0118,
   0.0285, 0.0371, 0.0465, 0.509; the generic taper's 0.0176, 0.0600, 0.100, 0.162, 0.408),
   and the law runs straight in the fraction's logarithm between them. The voice now matches
   at every mark within 0.01 dB. The marks are taken as the knob's tenths of travel. The
   reference's law is nearly flat to 6 and steep above it, as a gain control behaves rather
   than an attenuator; through the CA-72's circuit it is R9's law. At a given mark the
   CA-72's drive into the filter is about 2.2 dB under the reference's (session A's match: its
   5 drove as the CA-72's old 0.375): left to the mixer's comparison (session D), which
   tells whether the bus or the external path accounts for it. Only the two presets whose
   FEEDBACK runs through EXT change: Pulse Strut -1.1 dB, Undertow Growl +3.0 dB and brighter
   (not re-levelled); the others render the same to the bit. `feedback.rs` plays its loop at
   VOLUME 0.7, where R9 now sits as 0.6 did.
5. **Oscillator 3 with OSC. 3 CONTROL off: R162 3.01K to 2.96K** (`tuning::R162`,
   2026-10-09). With the control off oscillator 3 loses its keyboard, tune, bend, modulation
   and external inputs and its frequency trimmer, so at FREQUENCY's tuned centre its pitch
   comes from its range resistor and IC8's output, which R150, R155, R162 and R170 set. The
   reference sat at 198.82 Hz at 8' (session C, take 25; V/OCT moved it not at all), the
   CA-72 at 232.67 Hz, 0.27 octave higher. With the control on the same knob put the
   reference's oscillator 3 within 9 cents of oscillator 1, so the knob was at its tuned
   centre. R162 sets IC8's output at the centre; the factory tuning absorbs it with the
   control on (the control-on pitch unchanged to 0.001 Hz), so it moves only the control-off
   pitch: 2.95K gave 195.73 Hz, 2.955K fits, 2.96K (1 %, E192) gives 201.5 Hz in the voice
   (+23 cents; 202.7 Hz in ngspice's bench). The netlist (`board1-osc23.lib`) and the model
   change together, and IC8's input offset at its new operating point (0.1868 mV, measured in
   ngspice by `vco_osc23.rs`) with them. Preset levels move within 0.04 dB; the presets that
   use oscillator 3 with the control off change only in its pitch (vibrato rates). On LO with the
   control off, FREQUENCY at its minimum now clicks every 5.7 s: past the service manual's 2
   to 5 s (5.36), within the reference's manual (oscillators from 0.1 Hz); the voice test
   takes 10 s as its bound.
6. **EMPHASIS's law** (R14, `voice::emphasis_r14`, 2026-10-09). The reference's passband
   falls -2.25, -9.53 and -12.50 dB at EMPHASIS 2.5, 5 and 7.5 and its peak rises +0.4, +11.6
   and +21 to +33 dB across the cutoffs (session E). With Figure 9-17's 50K reverse audio on
   the generic taper the CA-72's fell -2.63, -7.19 and -12.85 dB, peaking +0.8, +6.4 and +39
   dB: too little resonance in the middle, a little too much near the top. Mapping the CA-72's
   response over EMPHASIS in steps of 0.05, R14 that gives the reference's loss (its peak as a
   check) is 18.82K, 2.91K and 1.35K (the generic taper's 16.25K, 5.0K and 1.25K); the
   voice's law runs straight in R14's logarithm from 0 (50K) through those, and from 7.5 to 10
   follows the generic taper's shape scaled to meet them (0 at 10). Now -2.18, -9.50 and
   -12.45 dB, peaking +0.5, +11.9 and +23 to +38 dB. Folkman's regeneration calibration and
   the service manual's checks keep the drawing's law (`emphasis_r14_drawn`), so R73 and
   `FACTORY` are unchanged. Presets with EMPHASIS between its ends change, their levels down
   by up to 2 dB (Three Saw Slab -1.96, Lead and Upright Pluck -1.02; not re-levelled); at 0
   or 10 they render the same to the bit.

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
