# Calibration against a hardware reference

**Owner decision, 2026-10-07.** The CA-72 is matched to the owner's hardware reference, a
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
| Contours | From session F the reference's FILT CONT and LOUD CONT jacks go to two DC-coupled inputs of an ES-6 on the interface's optical inputs (computer inputs 11 and 12), read at 10 V full scale. Calibrated in session L against the ES-3's CUT CV output looped into each (0 to 4.68 V and -0.52 V): FILT CONT's input reads 0.9411 of the voltage +0.019 V, LOUD CONT's 0.9432 -0.049 V, both straight within 0.1 mV. The contours' voltages below are corrected by them (before session L they were not: a contour read 5.9 % low) |
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
   (`expo::R_EXT`, 2026-10-08). The keyboard reaches each oscillator through R27 51.1K and
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
3. **The filter's RANGE trim, R39: 0.632 to 0.559** (`filter_cal::CALIBRATED`, 2026-10-08).
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
4. **EXTERNAL INPUT VOLUME's law** (R9, `voice::ext_taper`, 2026-10-08). The voice's
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
   2026-10-08). With the control off oscillator 3 loses its keyboard, tune, bend, modulation
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
6. **EMPHASIS's law** (R14, `voice::emphasis_r14`, 2026-10-08). The reference's passband
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
7. **ATTACK's and DECAY's laws** (the four 1M rheostats, `voice::time_pot`, 2026-10-08).
   Session F set each contour's ATTACK and DECAY to its dial's marks (10 and 200 ms, 1 and 10
   s at -120, -90, 30 and 105 degrees on the reference's dial, its manual's drawing and the
   owner's photographs agreeing; the CA-72's panel, after the original's, prints 10 s at
   108 and 5 s at 67 where the reference has 60), its output
   on a DC-coupled input; the CA-72 renders the takes at the same marks. Measures: the
   attack's rise from 10 to 90 %; the final decay's fall (DECAY on, SUSTAIN 10) from 90 to 50
   % (the takes end before the slowest falls further). At 1 ms (fully anticlockwise) both
   attacks match (0.6 and 0.9 ms). At the 10 ms mark and the tick past it the times scatter
   about the drawing's taper by where the knob was set (attack 0.65 to 1.08 of the
   reference's, decay 0.84 to 1.08), so the generic taper stays to there (0.15 of the travel).
   Above it the CA-72's contours were too slow at the 1 s mark (2.8 to 3.3 times the
   reference's time), too fast at the 10 s mark (0.68 to 0.84) and, fully clockwise, the
   attacks 0.91 to 0.94 and the decays 1.16 to 1.18. The pot's resistance that gives the
   reference's time through the CA-72's contour (the attack exactly linear in it, the decay
   found between renders at bracketing resistances):

   | | 1 s mark (0.6) | 10 s mark (0.85) | fully clockwise |
   |---|---|---|---|
   | Generic audio taper | 162K | 535K | 1M |
   | Filter ATTACK | 52.5K | 749K | 1.06M |
   | Filter DECAY | 58.5K | 660K | 873K |
   | Loudness ATTACK | 53.0K | 788K | 1.10M |
   | Loudness DECAY | 49.9K | 630K | 853K |

   Each law runs straight in the resistance's logarithm from the generic taper's 11.7K at
   0.15 through its three points. Every mark now matches within 0.2 % (loudness attack at 1
   s 457 ms, 10 s 6.78 s, fully clockwise 9.47 s; its decay 293 ms, 4.02 s, 5.53 s; filter
   attack 294 ms, 4.18 s, 5.92 s; decay 343 ms, 4.33 s, 5.82 s). The reference's laws are
   steeper than an audio taper between the 1 s and 10 s marks (13 to 15 times the
   resistance), and its pots' ends are within a 1M part's 20 %, the ATTACK pots above 1M and
   the DECAY pots below. The contour circuits and their ngspice benches are unchanged (the
   benches take resistances); `tests/panel_laws.rs` holds the laws to their points.
8. **The mixer's VOLUME law** (`voice::volume_track`, 2026-10-08). Oscillator 1's sawtooth
   alone at 8', the reference's MIX against VOLUME 10 fell -2.10, -5.19, -9.42 and -16.51 dB
   at 8, 6, 4 and 2 (session C, takes 9 to 12); the CA-72's 25K linear pot, its wiper
   loaded by the 33K into the bus, fell -2.71, -5.53, -8.96 and -14.47 dB. The fraction of
   the track at which the CA-72's channel gives the reference's level is 0.155, 0.378, 0.623
   and 0.845, and the law runs straight between those, 0 and 1. The MIX now matches at every
   mark within 0.01 dB, the main output within 0.11 dB. The S is what a linear track gives
   when the knob's first and last few degrees turn on its dead ends. One part serves the
   four channels (oscillators 1 to 3 and the noise), so all four take the law; only
   oscillator 1's was measured. At 0 and 10 nothing changes. Presets with a VOLUME between
   rise by up to 0.6 dB (Bass, Lead and Slow Horn Swell, at about 8; not re-levelled); the
   others stay within -136 dB.
9. **The external input's preamplifier: R61 200K to 232K** (`preamp::R61`, 2026-10-08).
   At the MIX, oscillator 1's sawtooth at VOLUME 10 against EXT's tone per volt at the jack
   (VOLUME 2 and 5, clean) was 0.92 dB weaker on the reference than on the CA-72, the same
   from 84 Hz to 1.5 kHz: a comparison of the two paths with the filter left out. Through
   the filter, the CA-72 needed 1.0 to 2.0 dB more input than the reference for the same
   compression of the main output (0.5, 1 and 2 dB, at EXT VOLUME 2 to 8; session H), while
   the oscillators drive it more nearly as the reference's do (session D: 0.5 dB short at
   170 Hz, about 2 at 84 Hz, where the takes beat slowly and the measure is coarse). So the
   external path was short, and of its parts the preamplifier's gain is the one
   that fits: in oscillator units the reference's path is 0.9 dB more sensitive but clips
   0.7 dB lower, where a smaller R46 would have raised both. R61 at 232K (1 %, E96) gives
   the circuit 0.97 dB more gain in ngspice (229K would give 0.89); the netlist and the
   three real-time preamplifiers change together. Now the paths at the MIX agree within
   0.04 dB, and the CA-72 needs 0.2 to 0.7 dB more input than the reference for the same
   compression: the rest is on the filter's side, shared with the oscillators. The 2.2 dB
   first estimated (change 4) came from session A's fit under the old taper. 5.26's test
   now finds the OVERLOAD lamp at VOLUME 9.2, searched in steps of 0.1 (4.4 % at the output,
   as before at 9.5). Only Pulse Strut (+0.4 dB) and Undertow Growl (+1.8 dB), whose
   FEEDBACK runs through the input, change.
10. **CUTOFF's law** (R11, `voice::cutoff_track`, 2026-10-08). The owner photographed the
    reference's dials at their stops (`rig/dial-photos-2026-10-08` on the lab's share): each
    knob turns 300 degrees and stops on its dial's end ticks; CUTOFF's eleven ticks are 30
    degrees apart, -5 and 5 at the stops, as on the CA-72's panel. (The first comparison of
    the marks took the stops for -4.5 and 4.5.) Through session E's sweeps, the same measure
    of both, the CA-72's corner was 0.36 to 0.43 octave under the reference's at +4, 0.28 to
    0.35 under at +2, 0.33 to 0.42 over at -2 and 0.45 to 0.53 over at -4, and within 0.08
    at 0 and at the anticlockwise stop. At CUT CV 0, the reference runs 10.0 octaves from
    stop to stop, as the CA-72 does (10.1), but steeper in the middle (1.17 octaves a mark)
    and flatter near the stops (0.5 to 0.6 in the last mark): the S of the mixer's VOLUME
    pots (change 8), a linear track whose ends turn slowly. The fraction of the track that
    puts the CA-72's filter where the reference's is at -4, -2, 2 and 4 is 0.0582, 0.2669,
    0.7304 and 0.9346 (a straight track's 0.1, 0.3, 0.7, 0.9); the law runs straight between
    those and the stops and centre, where R39 was fitted, so R39 stands. Now every mark is
    within -0.08 to +0.10 octave of the reference. What is left leans with CUT CV the same
    way at every mark: the CA-72's filter moves 2.3 to 3.1 % more per volt than the
    reference's (Still differs). Folkman's calibration and the service manual's checks keep
    the drawing's straight track (`filter_cal::inputs`). ENTROPY's cutoff offset test now
    allows 8 %: at +3 the knob is 0.83 of the track, where the converter's curve leaves the
    offset 6.6 to 7.5 % short. Presets with CUTOFF off its marks change in brightness; the
    retuned two move again (redone with the levels, below).

## Not changed, and why

- **CUTOFF's law at its odd marks** (session N, takes 23 to 30, 2026-10-09). The law was
  measured at -4, -2, 0, +2 and +4 (change 10) and interpolated between. With the filter
  singing alone (EMPHASIS 10, every source off, KEYBOARD CONTROL off), the reference's pitch
  at -3, -2, -1, 0, +1 and +2 was 110.8, 206.6, 472.2, 1073.8, 2344 and 5188 Hz, the
  CA-72's -0.05, +0.65, -0.06, -0.29, +0.15 and +0.30 semitone from it; at -1 with both
  KEYBOARD CONTROLs on and E3 held, 1118 Hz and +0.64 semitone (at -4 the reference hardly
  sang). The law stays. (Hollow Glider's first take had seemed to show the filter half an
  octave high: its oscillator 2 sat about 16 dB under the sheet's VOLUME 8, take 31 played
  from the sheet matches the CA-72 within 2 to 3 dB, and with the filter open, take 22, the
  mixer's balance within 0.5 dB.)
- **R1401's value.** With one contour charged the release already has the reference's
  shape; the factor of two was the shared resistor.
- **The trigger delay** (board2.md, B2-3: 8 ms from a key, 12 ms after its release). The
  reference's gate input adds 2.3 ms at a gate's rise and about 6 ms at its fall, and needs 3
  ms of gate: its own processing, not its contour's. The 8 and 12 ms stay (the owner,
  2026-10-08; session G holds a MIDI note's timing on the reference).
- **The thump** (B4-6): its size against a note's level, which the oscillators will give.
- **The corner's mapping:** see the table.

11. **The keyboard's 0 V on C2** (`keyboard::R_FLOOR`, board2.md B2-8, 2026-10-08). The
    reference's MIDI puts its keyboard's 0 V on C2 (MIDI NOTE ZERO VOLTS, 36 by default in its
    manual); the original keyboard, and the CA-72, on its lowest key, F, five keys higher.
    The oscillators are tuned to either, but the filter's KEYBOARD CONTROL hears the voltage
    itself: at the same CUTOFF the reference's filter sat 0.35 octave higher with both
    switches on (session E: 856 against 672 Hz at note 41), and in session I's Ringing Saw
    Line its ring 0.15 to 0.24 octave above the CA-72's once the contour had died away. The
    owner chose to follow the reference: the key string's bottom reaches GND through 50 ohm,
    five of its resistors, every key 0.42 V higher; the factory tuning absorbs it (A3 at
    220.000 Hz, the keys within 2.43 cents) and the bench's string matches it within 0.0001
    mV a key. Presets with KEYBOARD CONTROL on are brighter, as on the reference (by 0.14,
    0.28 and 0.42 octave with 1, 2 and both).

12. **KEYBOARD CONTROL's resistors: R53 300K to 312K, R54 150K to 156K** (`vcf::R53`,
    `vcf::R54`, 2026-10-08). The filter's corner over five MIDI notes (29 to 77, both
    switches, CUTOFF 0; session J) moves 0.970 octave an octave on the reference, the same
    within 0.023 from octave to octave; through the CA-72 (its keyboard's 0 V on C2 already)
    1.004, its corner 0.005 octave from the reference's at note 29 and 0.142 at 77. Session E
    found switch 1 and switch 2 alone 4.4 and 4.0 % steep, so both resistors. CUT CV, set
    beside the reference the same way (the self-oscillating filter stepped 1 V at CUTOFF -2:
    1.007 to 1.018 octave on the reference, 1.012 on the CA-72; session E's CUT CV sweep 0.996
    against 0.986 octave a volt), is right: the earlier "4 % steeper" set the CA-72 at 1 kHz
    beside the reference at 220 Hz. With R53 and R54 4 % higher (312K and 156K, 1 %) the
    CA-72 tracks 0.967 and its corner is within 0.016 to 0.035 octave of the reference's
    across the four octaves. The voice, the keyboard's load and the filter's ngspice bench
    change together; Folkman's procedure keeps the drawn values.

13. **ATTACK's and DECAY's laws at every printed mark** (`voice::FILTER_ATTACK` and the
    others, session J, 2026-10-08). Session J set both contours' ATTACK and DECAY at the
    marks session F had not taken: 200 ms (-90 degrees), 600 ms (-30), the top tick and 5 s
    (60), the angles of the reference's dial (its manual's drawing, measured at 1200 dpi, and
    the owner's photographs agree within a degree). The laws now run through 0.2, 0.4, 0.5,
    0.6, 0.7, 0.85 and 1.0 of the travel; their shape is the reference's own: gentle to the
    1 s mark (15 to 53K), then steep (230 to 300K at 5 s). Between the 1 s and 10 s marks the
    old laws, straight from 1 s to 10 s, were up to 2.5 times out. Every mark of sessions F
    and J now within 0.2 % (attack 10 to 90 %, final decay 90 to 50 %). Below the tick past
    10 ms the generic taper stays.

14. **AMOUNT OF CONTOUR: its knob's law, and R74 47K to 45.3K** (`voice::contour_input`,
    `vcf::R74`, board4.md B4-9, 2026-10-08). Session J held the filter contour at a SUSTAIN of
    5 (1.67 V at the FILT CONT jack against 0.29 V at rest) with the filter self-oscillating
    at CUTOFF -2, AMOUNT OF CONTOUR at 0, 2.5, 5, 7.5 and 10: the step moved the reference's
    filter 0, 0.606, 1.506, 2.389 and 3.081 octaves. `ca72-lab filterlaw` (now taking AMOUNT
    and the contour's volts through the voice's `contour_input`) gave the CA-72 0.701, 1.415,
    2.159 and 2.952: nearly straight where the reference's is an S, and 4 % short at full.
    R74 at 45.3K gives 3.06 at full; the knob's track at 0.2088, 0.5131 and 0.7976 for 2.5, 5
    and 7.5 (a linear track's 0.25, 0.5, 0.75: the S of the VOLUME and CUTOFF pots) gives the
    reference's octaves there exactly. In session I's Ringing Saw Line (AMOUNT 4) the CA-72's
    filter moved 0.89 to 0.95 octave a volt of contour against the reference's 0.81; with the
    law, 0.83 at 4. The contours' own voltages match (session I: their peaks within 3 %).
    (Change 20 corrects this: the contour was read 5.9 % low.)

15. **EMPHASIS at 6, 7 and 8.5** (`voice::emphasis_r14`, 2026-10-08). Session J measured the
    passband as session E did at three more marks: -10.43, -11.92 and -14.67 dB. At 8.5 the
    reference self-oscillates (its peak +47 to +54 dB over the passband), as the CA-72 does
    once R14 is below about 1.3K. Between 5 and 7.5 the law had run straight in R14's
    logarithm (2.14K and 1.57K at 6 and 7: -10.71 and -11.85 dB on session E's map of the
    CA-72) and past 7.5 on the generic taper's shape (0.63K at 8.5). On the same map the
    reference's losses are R14 2.30K, 1.548K and 834 ohm; the law now runs through them, and
    past 8.5 on the generic taper's shape scaled to meet 834 ohm (0 at 10). Rendering
    sessions E's and J's six takes: -2.16, -9.47, -10.43, -11.92, -12.41 and -14.63 dB
    against the reference's -2.25, -9.55, -10.43, -11.92, -12.48 and -14.67. Folkman's
    procedure keeps the drawing's law.

16. **A model fault: DECAY at its end** (`contour.rs`, board2.md B2-9, 2026-10-08). Played
    over MIDI with LOUDNESS DECAY fully anticlockwise (session J's GLIDE and mod-wheel
    panels), the reference sounds at once; the plug-in went silent 2 ms into the attack and
    came back over 30 to 50 ms. The drawn circuit in ngspice does not: the real-time
    contour's first decay sample took half a step from a current through a path that had
    been open, through DECAY's 0 ohm about an ampere. That sample is now backward Euler. No
    factory preset has a DECAY at 0: they render within -58 dB of before, Ladder Kick within
    -34 dB, and the two in FEEDBACK's loop (Pulse Strut, Undertow Growl), which any change
    sets on another course, with their levels within 0.17 dB.

17. **GLIDE's law** (`voice::glide_pot`, 2026-10-08). Session J played C3, C4, C3 over MIDI
    at GLIDE 2.5, 5, 7.5 and 10. From each note's start, the keyboard's voltage slid from 20
    to 80 % of the octave at 61 and 22 (up and down), 29 and 18, 3.8 and 2.1, and 1.6 and 0.9
    octaves a second: a straight slide, as the CA-72's circuit makes it (the hold amplifier
    saturates toward its rails, faster up than down). The CA-72 on the generic taper (125K,
    500K, 1.63M, 5M): 63 and 29, 16 and 8.3, 5.1 and 2.6, 1.65 and 0.86. The netlist takes
    GLIDE's resistance; rendering the phrase at chosen resistances, the reference's rates
    (both directions together) are 162K at 2.5, 257K at 5 (216K to 280K by the measure: the
    CA-72's downward slide starts fast and then slows where the reference's runs straight),
    2.1M at 7.5, and 5M at 10 as drawn: its pot has the two slopes of its contour pots. The
    voice's law runs through them (below 2.5 the generic taper's shape scaled to 162K). Now
    58 and 23, 29 and 13, 3.9 and 2.05, 1.65 and 0.86.

18. **The MODULATION wheel: 685 ohm fully forward, and MIDI's curve** (`modulation::
    mod_wheel_r`, `modulation::midi_wheel`, 2026-10-08). Session J held A3 on oscillator 1
    under oscillator 3's triangle (LO, MODULATION MIX at oscillator 3, OSCILLATOR MODULATION
    on, MOD DEPTH 10) and sent the modulation wheel at 0, 32, 64, 96 and 127: the
    reference's vibrato swung 0, 1.29, 2.62, 6.05 and 11.9 semitones peak to peak (98th less
    2nd percentile over each 1.65 s). The CA-72, its MIDI wheel moving its MODULATION wheel
    on the drawing's law (A22, 1.2K fully forward): 4.1, 8.5, 12.9 and 15.8. Rendering at
    chosen resistances, the reference's swings are 46.3, 97.8, 261 and 685 ohm (the
    drawing's at those positions 161, 394, 731 and 1.2K). Two things: fully forward the
    reference's wheel is 685 ohm, a component (`MOD_WHEEL_FULL`; the wheel keeps the
    drawing's law, A22, to it, as MOD DEPTH's own law was not measured); and its MIDI
    wheel's curve, which its manual makes a MIDI setting ("soft" by default), goes where it
    has it: in the plug-in's handling of control change 1, which puts the wheel where the
    reference's resistance is (straight in its logarithm between the four). Over MIDI now
    1.28, 2.59, 6.01 and 12.3 (the last within the measure's spread between 1.65 s windows,
    1.16 to 1.23 held). Presets keep their wheel positions, at 0.57 of the drawing's
    resistance (Breath Flute 0.4: 287 to 164 ohm; Undertow Growl, fully forward, 685). The
    service manual's checks still pass (5.37: 13.5 semitones under oscillator 3's square,
    13 to 23; 5.19: the corner 16.5 times, at least 5.45).

19. **A model fault: the contours' peak at Potato's rate** (`contour.rs`, board2.md B2-10,
    2026-10-08). Session I recorded Ringing Saw Line's FILT CONT (the interface's input at
    10 V full scale): its filter contour peaked at 4.63 V and decayed from there. The
    plug-in's (its voices in Potato, the contours at 6 kHz) peaked at 4.95 to 5.06 V and ran
    15 to 20 % above the reference's through the decay: its attack ended a sample past the
    flip-flop's threshold, at ATTACK 0 half a volt over. With the capacitor stopped where
    the output meets the threshold, the peak is the circuit's 4.49 V and the decay within 3
    to 7 % of the reference's; per volt the ring moves alike (0.79 and 0.74 octave against
    0.75 and 0.76), and the ring's excess over the reference's fell from 0.85 to 0.44 octave
    50 ms into the long G2 and from 0.29 to 0.12 at 850 ms. Every preset with a fast attack
    changes (levels below).

20. **AMOUNT OF CONTOUR again, in true volts: R74 48.1K, and a point at 4** (`vcf::R74`,
    `voice::contour_input`, board4.md B4-9, 2026-10-08). Session L calibrated the inputs
    that record FILT CONT (the rig, above): session J's contour step was 1.465 V, not 1.379,
    and through R74 45.3K the CA-72's filter moved 3.25 octaves at AMOUNT 10 for it against
    the reference's 3.08. 48.1K (E192) gives 3.07; the knob's track at 2.5, 5 and 7.5 is then
    0.2080, 0.5111 and 0.7950 (from 0.2088, 0.5131, 0.7976). Session L held the contour at
    0.187 and 3.942 V (FILTER SUSTAIN 0 and 10) at AMOUNT 4, the filter self-oscillating at
    CUTOFF -1 with both KEYBOARD CONTROLs on: the reference moved 2.879, 2.876 and 2.848
    octaves on G2, C3 and C4, 0.767, 0.766 and 0.758 octave a volt; the straight law
    between 2.5 and 5 gave the CA-72 0.390 of the track, and rendering the takes (the voice,
    both KEYBOARD CONTROLs loading the node) puts the reference's at 0.3796, a point of the
    law now: 0.767, 0.765 and 0.755 octave a volt of the CA-72's own contour. At AMOUNT 0
    the two filters sit within -0.03 to -0.05 octave there, and with EMPHASIS 6.8 their
    resonances within 0.01 (session L).

21. **The contours' peak: CR3 and CR6 silicon** (`contour::DCR36`, `mm-devices.lib`,
    board2.md B2-11, 2026-10-08). With the contour jacks' inputs calibrated, the reference's
    contours peak at 4.90 to 4.94 V (filter) and 5.73 to 5.75 V (loudness; session J's
    takes, the slower attacks a little higher), rest at 0.29 and -0.31 V and hold at 3.94 and
    4.61 V at SUSTAIN 10 (session L); the circuit's peak at 4.49 and 5.28 V, rest at 0.15 and
    -0.48 and hold at 3.83 and 4.48. The jacks sit 0.14 and 0.17 V above the circuit at rest
    (a buffer's offset: with it, the held levels agree within 1 %, and at FILTER SUSTAIN 0
    the reference's filter at AMOUNT 4 rose only as far as 0.09 V of contour would take it);
    the difference between peak and held level, which no offset touches, is 0.306 and 0.315
    V larger on the reference. Through the two peak dividers that is the same 0.23 V at both
    flip-flops' thresholds: a silicon diode where the drawing has germanium. One saturation
    current puts both peaks there (4.788 and 5.597 V in ngspice, against 4.795 and 5.597).
    Every preset's contours peak 7 % higher: a filter sweep's first moments up to a third of
    an octave higher, a loudness attack's overshoot 0.5 dB louder.

22. **Oscillator 3's LO range: 4.74 octaves below 32'** (`tuning::LO_BELOW_2`, 2026-10-08).
    The drawing does not give LO's place on the range switch (B1-6); the model assumed five
    octaves below 32' (A3). Session L played oscillator 3 alone on LO at FREQUENCY 0 and
    recorded the MIX jack: 2.0097 Hz with OSC. 3 CONTROL on (A3 held) and 1.8699 Hz with it
    off. Off, the reference's oscillator 3 was 198.82 Hz at 8' (session C): LO 6.732 octaves
    below 8'; on, it sat 35 cents under A-440 at 8' (session J, F5): LO 6.745 below. The
    CA-72 had LO 7.000 below 8' (1.720 and 1.575 Hz, 270 and 297 cents under the reference);
    LO is now 6.74 below 8' in the model and the netlist's bench alike (2.06 and 1.89 Hz).
    The oscillators' circuit reference (`vco1-core.json`) is regenerated from ngspice tuned
    by Folkman's procedure again (R11 150.3 ohm since the external input's 50.5K and the
    keyboard's 0 V on C2, 183.65 before; every range but LO within 0.05 cent of 0.1.0's,
    LO 312 cents up); the real-time oscillator tracks it within 0.08 cent (LO 0.57). Every
    vibrato and slow sweep from oscillator 3 on LO runs 20 % faster; 5.36's slowest click
    with the control off is 4.78 s.
23. **MOD DEPTH's own law on the MODULATION wheel** (`modulation::mod_wheel_r`, 2026-10-08).
    Session L turned the reference's MOD DEPTH knob with no MIDI wheel (a MIDI wheel's message
    takes the depth from the knob until the knob moves: at MOD DEPTH 2.5 a wheel at 127 still
    gave the full 12.2 semitones): 0.84, 2.62, 7.80 and 12.26 semitones at 2.5, 5, 7.5 and
    10, 0.069, 0.214 and 0.636 of the full swing, where the MIDI wheel at 32, 64 and 96 gave
    0.108, 0.220 and 0.509 (change 18). The CA-72's wheel had the drawing's law to the full
    685 ohm; now the knob's: 29.5, 95.1 and 349 ohm at a quarter, half and three quarters
    (the drawing's 91, 222 and 412), rendering 0.069, 0.211 and 0.643 of its full swing.
    MIDI's curve is unchanged: control change 1 puts the wheel where the reference's
    resistance for it is, through the knob's law now. Presets keep their wheel positions:
    those between the ends modulate less (Breath Flute's 0.4 from 164 to 60 ohm, about 40 %
    of its vibrato).

24. **The mixer's noise floor** (`voice::MIXER_HISS`, 2026-10-08). With every source off,
    EMPHASIS at 10 and a key held, the reference sang at once (session L, take 00); the
    CA-72 stayed silent (-85 dB): its filter sat at an equilibrium nothing disturbs, where
    the reference's starts from its circuit's noise. The reference's MIX jack carries
    -103.5 dBFS with every source off, 12 to 15 dB over the interface's idle inputs and 82.2
    dB under a sawtooth at VOLUME 10 (-21.3 dBFS, session J). The CA-72's bus now carries the
    same: white noise as a Norton current, 2.48 nA RMS from 0 to 24 kHz against the
    sawtooth's 0.0319 mA, seeded with the voice (renders stay reproducible). The filter now
    sings from silence at the pitches it had when seeded (-0.03 to -0.05 octave from the
    reference's, at CUTOFF -1 with both KEYBOARD CONTROLs). The floor is the MIX jack's,
    which may include its own buffer: an upper bound for what reaches the filter, whose
    start needs only some.

25. **The timing capacitors as electrolytics** (`contour::C_ABSORPTION`, `C_ESR`, `R7`, the
    four time laws; board2.md B2-12; 2026-10-09). Session M (the owner's choice among three:
    the full model) took the contours' decay toward SUSTAIN 0 at three marks (they run as
    the laws have them) and the drop after the peak alone (DECAY fully clockwise) after the
    fastest attack and one of 40 ms. After the fastest the reference's filter contour falls
    0.08 V within 0.1 ms, 0.21 V within 1 ms, 0.33 V within 10 ms and 0.46 V by 100 ms
    (about 0.05 V of it the decay); after 40 ms, 0.007 V at 1 ms and 0.03 V at 10 ms. The
    loudness contour the same at about 85 %. The drawing's capacitors are ideal and the
    CA-72's contours held their peak. Absorption alone (branches across the 10 uF) fitted
    the drop but slowed the fastest attack 30 %; with a series resistance and R7/R42 free,
    fitted with the real-time contour model to both contours and both attacks: R7 86.6 ohm,
    1.53 ohm in series, 0.769 uF through 779 ohm, 0.202 uF through 28.3K and 0.223 uF
    through 238K. The drops within 4.1 and 4.0 mV (fastest) and 0.8 and 1.5 mV (40 ms) from
    0.1 to 400 ms; the fastest rises 0.60 and 0.91 ms against the reference's 0.62 and 0.96.
    Over a long time the capacitor holds 11.19 uF, so each point of the four ATTACK and DECAY
    laws (and the generic taper below 0.15, scaled) moved to the resistance at which the
    contour keeps the time the first fit gave (0.88 to 0.90). Session M's decays toward
    SUSTAIN 0 now reach half way at 296, 356 and 226 ms (filter; the reference's 285, 349 and
    217) and 257, 309 and 190 ms (loudness; 258, 319 and 192), where the ideal capacitor
    took 337, 404 and 257 (288, 346, 213); their time constants stay within 0.5 to 4.6 %.
    Every preset's contours change after their attacks (levels within 0.5 dB, below). In
    Potato, FEEDBACK's loop now runs away (a dark self-oscillation near 58 Hz) from 0.3 of its
    coefficient, High Fidelity's from 0.35; `tests/feedback.rs` compares the modes at 0.25.

26. **SUSTAIN's law** (`voice::sustain_track`, 2026-10-09). Session N held a C3 with both
    contours' SUSTAIN at 2, 4, 6 and 8 (ATTACK fully anticlockwise, DECAY at the 200 ms mark,
    the DECAY switches off); session F had taken 0, 5 and 10 with them on. The DECAY switch
    moves where a contour rests between notes (0.22 V on the reference, 0.24 V on the
    CA-72), not where it holds, so the held levels compare as they stand at the jacks, less
    the reference's jack offset (0.15 V on FILT CONT, 0.18 V on LOUD CONT: its contours at
    rest against the CA-72's). At 0, 5 and 10 the CA-72 held within 0.01, 0.07 and 0.04 V of
    the reference; at 2 and 4 it held 0.12 to 0.18 V higher, at 8 0.11 V (filter) and 0.22 V
    (loudness) lower: the reference's knob sets less of its track below the middle and more
    above, the S of its VOLUME, CUTOFF and AMOUNT OF CONTOUR knobs. The track at 0.1611,
    0.3735, 0.4924, 0.6185 and 0.8375 for 2, 4, 5, 6 and 8 (the two contours' fractions,
    within 0.015 of each other, averaged; a linear track's 0.2 to 0.8) gives the loudness
    contour every mark's held level within 0.03 V and the filter contour 0.04 to 0.06 V over
    it throughout (the jack offset's own uncertainty: with DECAY off the contours rest 0.17
    V apart, with it on 0.15). The ngspice bench takes the pot's position, as for the other
    knobs' laws: the netlist is unchanged. Breath Flute's filter SUSTAIN at 2 ended each note at
    0.63 V against the reference's 0.43 in its take, now 0.51; its loudness SUSTAIN between 6 and 7
    held 0.11 V under the reference's, now on it.

27. **A model fault: the release at DECAY 0** (`contour.rs`, board2.md B2-13, 2026-10-09).
    Session N released both contours together over MIDI with DECAY fully anticlockwise
    (takes 20 and 21, the DECAY switches on and off, alike): from 90 to 50 % of the way down
    in 1.06 ms (filter) and 1.35 ms (loudness), from 50 to 10 % in 0.79 and 0.98; session F's
    contours released one at a time from their gates fell the same. The plug-in's fell in
    4.44 and 5.62 ms, then 4.42 and 5.88; ngspice's circuit in 1.75 and 2.36, then 1.47 and
    2.03. The real-time model solved V-trig from the contours' last states and then stepped
    them from it; through CR2 [CR9] a falling V-trig takes their currents, which hold it up,
    and the lag grew with the step (Potato's contours run at 6 kHz). Solved together while
    V-trig falls with DECAY near its end, the falls are within 0.04 ms of ngspice's at every
    rate, the plug-in's voice 1.77 and 2.38 ms, then 1.48 and 2.04. No factory preset
    releases with DECAY that near its end: their renders are unchanged to the bit. The
    circuit's own fall stays 1.6 to 2.1 times the reference's (change 28 follows).

28. **Q12, V-trig's transistor, at the reference's gain** (`contour::Q12HG`,
    `mm-devices.lib`, board2.md B2-14, 2026-10-09). Released with DECAY at its end, both
    contours fall as fast as Q12 takes their current, which grows with its base drive as Q20
    turns off and the reset line rises: so the fall's speed is Q12's gain over R19, and the
    delay before it is C7 and R55's (it agrees). In ngspice the gain moved the fall and
    hardly the delay: at 212 (the 2N3392 bin's middle) 1.75 and 1.47 ms (filter, 90 to 50
    and 50 to 10 %) and 2.36 and 2.03 (loudness); at 650, 0.97 and 0.79, 1.33 and 1.09,
    against the reference's 1.06 and 0.79, 1.35 and 0.98: no gain fits all four better (the
    loudness's last half wants more, the filter's first half less), and 650 holds them all
    within 0.11 ms. A BC547C-class part's gain (its bin 420 to 800), where the 2N3392's is
    150 to 300: the likelier difference in a recreation (R19 halved moved the fall about
    half as far as the gain did).
    The netlist and the model take it together (`ContourCircuit::q12`). V-trig's edges in
    ngspice: 7.97 ms after a key's press as before, 12.14 ms after its release (12.48). The
    plug-in's voice now falls in 0.98 and 0.81 ms (filter), 1.33 and 1.13 (loudness). The
    factory presets move little (levels within 0.01 dB but Undertow Growl's loop, -0.12).

29. **Oscillators 2 and 3's FREQUENCY laws** (`voice::osc2_freq_track`, `osc3_freq_track`,
    2026-10-09). Session N held an A3 with each oscillator alone, FREQUENCY at both stops,
    -5, 0 and +5, between takes of oscillator 1 alone (its tuning drifted 0.15 cent). From
    oscillator 1 the reference's oscillator 2 stood at -8.05, -6.55, -0.07, +6.54 and +8.23
    semitones, its oscillator 3 (OSC. 3 CONTROL on) at -8.87, -6.99, -0.26, +6.66 and +8.29;
    the CA-72's at -8.57, -5.67, 0.00, +5.52 and +8.24, and -8.63, -5.67, 0.00, +5.48 and
    +8.22. Their stops nearly agree where their -5 and +5 marks are a semitone apart: the
    reference's knobs set their tracks with the S of its other linear knobs, much stronger
    (no change of span gives it). Its 0 marks sat 7 and 26 cents flat, but oscillator 3's sat
    8 cents off in the same session's Hollow Glider (from its 16' pair's beat): where a knob
    lands by eye (a tenth of a mark is 13 cents there), not a trim. So the 0 mark stays the
    centre, where the factory tuning puts it in unison, and the laws take the reference's
    marks from its 0: oscillator 2's track at 0.0334, 0.1195, 0.9000 and 1 for the stops and
    the -5 and +5 marks, oscillator 3's at 0.0008, 0.1060, 0.9209 and 1 (a linear track's 0,
    1/6, 5/6, 1). Every mark now within 0.2 cent of the reference's but the clockwise stops,
    0.06 and 0.33 semitone short (the CA-72's pots end there). The ngspice benches take the
    pot's position: the netlist is unchanged. The presets' FREQUENCY values (all but 0) are
    rewritten so that each keeps its pot's position (R42): their intervals and oscillator
    3's rates as they were.

30. **EMPHASIS at 2, 3 and 4** (`voice::emphasis_r14`, 2026-10-09). Upright Pluck played
    from its sheet twice (takes 01 and 34, alike) stood 5 to 6 dB under the reference from
    400 to 800 Hz. With the filter open (take 35) the shark tooth matched to 0.2 dB, and with
    the contour out of the filter (take 36) the filter at CUTOFF -3 nearly did: the
    difference came with AMOUNT OF CONTOUR. Held at a full contour (takes 37 to 40: AMOUNT 4,
    the filter near 800 Hz, the shark tooth at 32' laying its harmonics 16 Hz apart through
    it, each take against the open one), the reference's passband and peak at EMPHASIS 2, 3,
    4 and 5 stood at -0.8 and 0.0 dB, -4.0 and +2.6, -8.0 and +9.3, -8.7 and +11.0; the
    CA-72's where its own knob was at 1.25, 3.4, 4.7 and 5.0: at 5 (a point of change 6's
    law) the same, between 2.5 and 5 steeper than the law's straight line in R14's
    logarithm, below 2.5 gentler than the generic taper's shape. R14 at 30.68K, 9.47K and
    3.64K for 2, 3 and 4 (the generic taper's 20.4K, 12.9K and 8.1K) puts every mark within
    0.4 dB of the passband and 0.6 dB of the peak; 2.5 and 5 keep their points (2.5 lies
    where the reference's 2 and 3 put it). The peak stands 0.07 to 0.09 octave below the
    reference's. Folkman's procedure keeps the drawing's law. Upright Pluck now 1.4 and
    2.9 dB under the reference at 400 and 800 Hz (5.6 and 5.7 before).

31. **The VCA's R43: 270K to 180K** (`VcaCircuit::r43`, `board4-vca.lib`, board4.md B4-10,
    2026-10-09). Session O played sines from the ES-3 into EXT, stepped from 0.0003 to 0.1
    of full scale (oscillator 1's sawtooth at VOLUME 10 reaches the mixer as about 0.03
    does): C3 with the filter open, and 995 Hz at the resonance of EMPHASIS 7 with CUT CV at
    -0.5 (the reference's peak at 998 Hz, the CA-72's at 982), the VCA at LOUDNESS SUSTAIN
    10, 7, 5 and 2. Against SUSTAIN 10 the reference's VCA passed -3.25, -8.12 and -32.6 dB
    at 7, 5 and 2, the CA-72's -3.96, -9.27 and -49.9. Session F's slow releases (a 1 kHz
    tone held through LOUDNESS DECAY at 10 s and fully clockwise, LOUD CONT recorded) give
    the whole law: at 4.0, 2.0, 1.0, 0.6, 0.45 and 0.35 V at the jack the reference's output
    stood -0.4, -7.3, -15.9, -25.2, -34.0 and -48.3 dB under SUSTAIN 10's, the CA-72's (its
    contour the jack's less 0.18 V, change 26) -0.8, -8.3, -17.9, -30.3, -47.5 and -75. The
    contour itself 0.18 V higher at the VCA would have fitted 7 and 5 but not the bottom
    (-44 dB at 0.3 V against the reference's -61). Q18's emitter returns to -10 V through
    R43, so a smaller one lets it conduct from a lower contour; R43 and the jack's offset
    trade against each other (220K at 0.12 V fits as well), and at the offset change 26
    measured, 180K (E24) puts the law within 0.6 dB from SUSTAIN 10 to 48 dB under it
    (`tests/vca_gain_law.rs`: 0.8 dB to -20, 1.5 to -35, 3 below). Q18's tail at full
    SUSTAIN is 2 % higher, and the driven VCA's 3rd harmonic with it: with the filter open at
    0.1 of full scale -17.7 dBc against the reference's -16.2 (-18.2 before), at the
    resonance -51.8 against -45.3 (-53.3). The netlist and the model change together, and
    against ngspice the VCA keeps its budgets (`vca_realtime`; not modelled: Q21 run
    backwards by 1.8 uA with EXT. LOUDNESS overdriven to 9 V, the VCA off in both). The
    factory balance on the new circuit: R14 at 0.5602 (ngspice 0.5599), R12 at 0.6700
    (0.6722). The thump at SUSTAIN 10 stays as it was, 0.225 of the tone for 0.01 of full
    scale against the reference's 0.218. A departure from Moog's schematic, which draws 270K.
    Every preset comes out 0.2 to 0.6 dB louder (the tail at full SUSTAIN), its spectrum
    within 1.6 dB, and is re-levelled (R42).

## Still differs (2026-10-08, after session J's fits)

- **The filter's overdrive:** with the external path matched (change 9), the CA-72 still
  needs 0.2 to 0.7 dB more drive than the reference for the same compression, from EXT and
  about as much from the oscillators: the filter's input side (R54 470 and the input pair),
  for the mixer's overdrive (session D) to settle. With the VCA nearly closed (session O,
  SUSTAIN 2) its 2nd harmonic stands 1.5 to 4.7 dB over the reference's at 0.03 to 0.1 of
  full scale.
- **The VCA's law at its top** (change 31): from 1 to 4 V of contour the CA-72's gain stands
  0.4 to 0.6 dB further under SUSTAIN 10's than the reference's.
- **EMPHASIS's peak against its passband:** at the same passband loss the CA-72's peak
  stands 1 to 1.5 dB higher than the reference's (+14.7 to +15.3 dB at 6 against +13.9 to
  +14.3; +21.1 to +22.3 at 7 against +19.4 to +21.3), and at 7.5, near regeneration, up to 5
  dB higher at some cutoffs: the ladder's Q at a given loop gain.
- **GLIDE's two directions:** at GLIDE 7.5 and 10 the CA-72 slides down 1.9 times slower
  than up, the reference 1.76 and 1.81 times (the hold amplifier's downward drive, toward
  -4.4 V through R59/R54); at GLIDE 5 the CA-72's downward slide starts fast and then slows
  where the reference's runs straight.
- **Ringing Saw Line's ring** (session I) stands +0.06 to +0.16 octave over the reference's
  mid-note on G2 and 0 to +0.22 on C4 (0.3 before change 25): within where the patch's
  DECAY was set on the reference by eye, just right of the top tick.
- **The contours' release with DECAY at 0** (changes 27 and 28): from 90 to 50 % in 0.98 ms
  (filter) and 1.34 ms (loudness) against the reference's 1.06 and 1.35, from 50 to 10 % in
  0.80 and 1.10 against 0.79 and 0.98: the loudness contour's last half 0.1 ms long.
- **Oscillator tracking:** with the external control input at 50.5K, over four volts the
  CA-72 ends 22 cents under the reference (0.9986 against 1.0030 octave a volt at the ES-3's
  10.39 V), against 80 cents before.
- **VOLUME on oscillator 2 and the noise** (session J, measured at the MIX jack and on the
  CA-72's mixer bus; no change): at 8 and 4, oscillator 2 -2.10 and -9.42 dB against the
  reference's -2.29 and -9.55 (oscillator 3's as the reference's oscillator 2's within 0.2
  dB), and the noise, through its 11K, -3.60 and -11.93 against -3.94 and -11.98.
- **Oscillators 2 and 3's FREQUENCY at their clockwise stops** (change 29): 0.06 and 0.33
  semitone short of the reference's; the 0 mark at the factory tuning's unison, where the
  reference's sits wherever its knob is set by eye (7 to 26 cents flat in session N).
- **The presets** are re-levelled and the two that sound the filter's pitch retuned (R42),
  after sessions J's, L's, M's, N's and O's fits too. Eight at MAIN OUTPUT VOLUME 10 are
  quieter than in 0.1.3 and VOLUME cannot raise them: Upright Pluck -5.16 dB, Breath Flute
  -3.41, Ladder Kick -1.95, Stacked Fifths -1.88, Open Hat -1.16, Ringing Saw Line -1.12,
  Shoreline Wash -0.23, Noise Snare -0.16 (mostly EMPHASIS's and the contours' laws, and the
  contours' fall after a fast attack). Presets keep their
  knobs' positions where a law changed: those between a law's ends sound as the reference
  would at those marks (Ladder Kick's sweep, for one, starts about 2 semitones lower).

## Reproducing

```
# On the Mac, in a terminal with the microphone permission:
cd scripts/calibration && python capture.py session_a ~/lab/artifacts/ca-72/calibration/captures/<date>-A
# A preset's phrase on the reference (its panel set by hand), in the same terminal:
cd scripts/calibration && python midi_capture.py ~/lab/artifacts/ca-72/calibration/captures/<date>-I --phrase <phrase.json> --take <name> --index <n>
# Anywhere:
cargo run -p ca72-lab --release -- stim <take.wav> scripts/calibration/patches/home_ext5.json <render.wav>
# The same phrase through the plug-in with the preset (CA72_SET to match the reference's TUNE):
CA72_ONLY=<preset> CA72_PHRASE=<phrase.json> CA72_LEVELS_OUT=<dir> cargo test --release -p ca72-plugin --test preset_levels -- --ignored --nocapture
# The contour takes (session F): --fc-gate lets FC GATE trigger the CA-72's contours too;
# the render's fourth and sixth channels are the loudness and filter contours.
python scripts/calibration/analyze.py <take.json> [--render <render.wav>]
```
