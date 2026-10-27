# The 0.3 presets (were candidates)

**Shipped in 0.3.0** (decisions.md R-PRESETS-0.3.0): the seventeen presets proposed here, and the
four LFO LOOP presets, are factory presets, forty in all, at the end of
`crates/ca72-plugin/sounds/presets.toml`. Two aims, the owner's (2026-10-26):

1. **Sounds the 0.2.8 set leaves out.** Its nineteen are five basses, two horn leads, a brass
   section, a flute, a xylophone, bowed strings, a five-piece drum kit, a riser, a pad and a
   gliding unison. Nothing played as an organ or keys, a chord stab, a bell, an acid or sub bass,
   a tom, a zap or an ambient texture, and nothing was rhythmic.
2. **The 0.3 features in use.** No factory preset set the arpeggiator, an LFO, a macro, velocity,
   a CONTINUOUS waveform or its MOD, a timed oscillator, or pressure and timbre.

| Preset | Kind | What it shows |
|---|---|---|
| Acid Squelch | bass | the acid bass; NOTE PRIORITY LAST and glide; macros SQUELCH, DECAY |
| Tonewheel Drift | organ, POLY | three LFOs as a rotating speaker; macro ROTOR moving the LFOs' RATEs; PERC |
| Glass Bell | bell, POLY | OSCILLATOR MODULATION as FM from oscillator 3 at 2'; velocity on CUTOFF; macro CLANG |
| Night Surf | ambient FX | four slow LFOs on the filter, level, emphasis and width of pink noise |
| Ray Gun | zap FX | an LFO in ENV mode as a pitch envelope (the panel has none); macro SWEEP on its RATE |
| Pitch Tom | drum | the first tom: the same pitch envelope, gentler |
| Deep Boom | sub bass | pitch drop, DRIVE and glide; macro GROWL on DRIVE |
| Club Stab | chord stab, POLY | velocity on CUTOFF; macros OPEN and WIDE |
| Octave Runner | arp bass | ARP UP, two octaves, sixteenths; a timed LFO over two bars; macro BITE |
| Random Glass | arp, HOLD | ARP RANDOM, three octaves, triplets, HOLD; a timed LFO on a CONTINUOUS pulse's width |
| Swing Keys | arp keys, POLY | ARP UP/DOWN with SWING; tremolo LFO; macro TINE |
| Tempo Wobble | bass | two timed LFOs faded in by macros (WOB 8, WOB 16) through their LEVEL; GROWL |
| Pulse Strings | strings, POLY | CONTINUOUS waveforms with MOD: PWM from each voice's oscillator 3 at LO |
| Gated Chords | pad, POLY | oscillator 3 TIMED at LO through FILTER MODULATION: a tempo gate |
| Pressure Solo | lead | PRESSURE to the filter, TIMBRE to the mod wheel; delayed vibrato by RISE |
| Morph Bloom | pad, POLY | per-voice LFOs (MONO off) on CONTINUOUS shapes; four named macros |
| Bubble Computer | FX | four timed LFOs, one moving another's LEVEL; SHARP edges |
| LFO LOOP Coupled Rates | pad, POLY | two LFOs setting each other's RATE |
| LFO LOOP Phase Chase | lead | two LFOs pushing each other's PHASE |
| LFO LOOP Level Pump | pad, POLY | an LFO fading another's LEVEL, which dents its own |
| LFO LOOP Self Rate | lead | an LFO on its own RATE |

Every macro starts at 0 (Tempo Wobble's WOB 8 at 80), so each sounds as described before a
macro is touched. Changed on the way in: Club Stab and Glass Bell brighten with velocity
(R-VELOCITY arrived the same day); Gated Chords' filter sits lower (CUTOFF -5, EMPHASIS 3.5,
AMOUNT OF CONTOUR 1, the mod wheel at 0.8), as at -2.5 the gate did not close (1.4 dB between
its steps, the same as with oscillator 3 off; 24 dB now); "pluck" became the factory's
"plucked"; the LFO LOOP presets have tags of their own in place of "LFO Loops".

## The files

- `scripts/presets_0_3.py` makes all twenty-one, every panel value named, and writes them into
  `presets.toml` after its marker line (`# ---- 0.3.0's presets`); edit them there, not in the
  file. It keeps the AUTO GAIN curves already in the file.
- `scripts/presets_0_3_levels.json`: their MAIN OUTPUT VOLUME and LEVEL, written by
  `scripts/level_presets.py`.
- `crates/ca72-plugin/tests/factory_curves.rs` writes every preset's curves
  (`CA72_WRITE_CURVES=<presets.toml>`, its ignored test).

The candidates' own files (`candidates.toml`, `load/`, `levels.json`), the candidates' test and
`docs/lfo-loops/presets` are gone: the factory file holds them now.

## Checked, and how

The factory measurements play every preset as the plug-in loads it: `tests/common`'s
`controls_of` loads it into the plug-in's own parameters as the preset drawer does (values,
defaults, what the LFOs, macros and velocity move, the macros' names) and takes the plug-in's
own `Ca72Params::controls`; `tests/common/host.rs` plays it through `process` as a host renders
offline, the transport at 120 BPM, with the preset's AUTO GAIN curve in force.

- `tests/factory_presets.rs` (by `cargo test`): every value names a parameter, every assignment a
  source and a target; each preset using 0.3's features sounds, finite, not silent, not clipped;
  each that keeps time (the arpeggiator, a timed oscillator or LFO) plays in the standalone at
  its TEMPO, 120, as at a host's 120, and at a host's 90 otherwise, as the standalone at 90.
  By hand, each held note's pitch against its key.
- `tests/preset_levels.rs` (by hand): the loudness at LO, HI and ULTRA; `scripts/level_presets.py`
  levels from it.
- `tests/lfo_loops.rs`: the LFO LOOP presets move an LFO's own knob, and sound.

### The levels (2026-10-27, the Linux desktop, the calibrated sound of `release-0.3.0`)

Momentary maximum on each preset's phrase, LUFS. Each new preset is levelled to -18 midway
between LO and HI; every preset is within -18 ± 1.5 at LO, HI and ULTRA but Feedback Funk at
ULTRA (-20.3; its ULTRA reads 3.5 dB under its HI). One seed and the phrase from the first
sample, as the factory set was levelled: a render repeats to the bit, where a new instance's
own parts and phases moved a reading by up to a decibel.

| Preset | LO | HI | ULTRA |
|---|---|---|---|
| Acid Squelch | -18.28 | -17.72 | -17.72 |
| Tonewheel Drift | -17.85 | -18.13 | -18.10 |
| Glass Bell | -17.91 | -17.96 | -18.06 |
| Night Surf | -18.81 | -17.18 | -16.98 |
| Ray Gun | -18.30 | -17.70 | -17.69 |
| Pitch Tom | -18.32 | -17.83 | -17.84 |
| Deep Boom | -18.20 | -17.79 | -17.79 |
| Club Stab | -18.07 | -17.93 | -17.86 |
| Octave Runner | -18.10 | -17.90 | -17.92 |
| Random Glass | -18.19 | -17.75 | -17.76 |
| Swing Keys | -18.06 | -18.04 | -18.04 |
| Tempo Wobble | -18.16 | -17.84 | -17.85 |
| Pulse Strings | -18.10 | -17.90 | -17.89 |
| Gated Chords | -18.77 | -17.24 | -17.20 |
| Pressure Solo | -18.23 | -17.76 | -17.76 |
| Morph Bloom | -18.03 | -17.96 | -17.94 |
| Bubble Computer | -17.72 | -18.42 | -18.41 |
| LFO LOOP Coupled Rates | -17.87 | -18.13 | -18.13 |
| LFO LOOP Phase Chase | -18.14 | -17.86 | -17.85 |
| LFO LOOP Level Pump | -18.21 | -17.89 | -17.91 |
| LFO LOOP Self Rate | -18.17 | -17.84 | -17.84 |

Every preset's, before and after, and renders for listening:
`/mnt/lab/artifacts/ca-72/presets-0.3.0/` (`levels/`, `audition/`).

Nobody had listened to these when they shipped to the release branch: the audition pack is for
the owner's ears.
