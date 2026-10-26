# Preset candidates for 0.3

Seventeen presets proposed for the factory set: none is in the plug-in yet. Two aims, the
owner's (2026-10-26):

1. **Sounds the 0.2.8 set leaves out.** Its nineteen are five basses, two horn leads, a brass
   section, a flute, a xylophone, bowed strings, a five-piece drum kit, a riser, a pad and a
   gliding unison. Nothing plays as an organ or keys, a chord stab, a bell, an acid or sub bass,
   a tom, a zap or an ambient texture, and nothing is rhythmic.
2. **The 0.3 features in use.** No factory preset sets the arpeggiator, an LFO, a macro, a
   CONTINUOUS waveform or its MOD, a timed oscillator, or pressure and timbre.

| Preset | Kind | What it shows |
|---|---|---|
| Acid Squelch | bass | the acid bass; NOTE PRIORITY LAST and glide; macros SQUELCH, DECAY |
| Tonewheel Drift | organ, POLY | three LFOs as a rotating speaker; macro ROTOR moving the LFOs' RATEs; PERC |
| Glass Bell | bell, POLY | OSCILLATOR MODULATION as FM from oscillator 3 at 2'; macro CLANG |
| Night Surf | ambient FX | four slow LFOs on the filter, level, emphasis and width of pink noise |
| Ray Gun | zap FX | an LFO in ENV mode as a pitch envelope (the panel has none); macro SWEEP on its RATE |
| Pitch Tom | drum | the first tom: the same pitch envelope, gentler |
| Deep Boom | sub bass | pitch drop, DRIVE and glide; macro GROWL on DRIVE |
| Club Stab | chord stab, POLY | macros OPEN and WIDE |
| Octave Runner | arp bass | ARP UP, two octaves, sixteenths; a timed LFO over two bars; macro BITE |
| Random Glass | arp, HOLD | ARP RANDOM, three octaves, triplets, HOLD; a timed LFO on a CONTINUOUS pulse's width |
| Swing Keys | arp keys, POLY | ARP UP/DOWN with SWING; tremolo LFO; macro TINE |
| Tempo Wobble | bass | two timed LFOs faded in by macros (WOB 8, WOB 16) through their LEVEL; GROWL |
| Pulse Strings | strings, POLY | CONTINUOUS waveforms with MOD: PWM from each voice's oscillator 3 at LO |
| Gated Chords | pad, POLY | oscillator 3 TIMED at LO through FILTER MODULATION: a tempo gate |
| Pressure Solo | lead | PRESSURE to the filter, TIMBRE to the mod wheel; delayed vibrato by RISE |
| Morph Bloom | pad, POLY | per-voice LFOs (MONO off) on CONTINUOUS shapes; four named macros |
| Bubble Computer | FX | four timed LFOs, one moving another's LEVEL; SHARP edges |

Every macro starts at 0 (Tempo Wobble's WOB 8 at 80), so each sounds as described before a
macro is touched.

## The files

- `candidates.toml`: all seventeen in the factory file's format, ready to move into
  `crates/ca72-plugin/sounds/presets.toml`.
- `load/`: each as a preset file of its own. To play them, copy them into the CA-72's presets
  folder (on a Mac `~/Library/Application Support/Idle Foundry/CA-72/presets`) and they appear
  under MINE.

Both are generated from one script; the levels in them come from the measurement below.

## Checked, and how

`crates/ca72-plugin/tests/preset_candidates.rs` plays each through the plug-in's own `process`
as a host renders offline, the transport at 120 BPM (the arpeggiator and the timed LFOs follow
it), the phrase chosen by its tags as `preset_levels.rs` chooses:

- by `cargo test`: every value names a parameter, every assignment a source and a target; each
  sounds, finite, not silent, not clipped;
- by hand (`-- --ignored --nocapture`): the loudness (momentary maximum, the factory set's
  -18 LUFS), the peak, and the first note's pitch against its key in cents where it has one;
  `CA72_CANDIDATES_OUT=<dir>` writes each render as a WAV at the level it plays, and Pressure
  Solo's again with a pressure swell.

What the measurement cannot say is how they sound: nobody has listened to them yet.

### The results (2026-10-26, the Mac)

The harness agrees with the factory set's own levelling: measured this way, the nineteen
factory presets read -18 ± 1.5 LUFS (Split Snare -20.9) and their held notes' pitch is within
12 cents of their keys. Each candidate is levelled to -18 LUFS between its LO and HI readings,
MAIN OUTPUT VOLUME bringing a loud one down and LEVEL a quiet one up (VOLUME at 10), the
factory set's rule:

| Preset | LO | HI | Pitch (cents) |
|---|---|---|---|
| Acid Squelch | -18.3 | -17.7 | -6 |
| Tonewheel Drift | -16.9 | -16.9 | 0 |
| Glass Bell | -17.8 | -18.5 | +6 (TUNE -0.1; +13 without the trim) |
| Night Surf | -19.5 | -18.3 | |
| Ray Gun | -18.3 | -17.7 | |
| Pitch Tom | -18.2 | -17.8 | +1 |
| Deep Boom | -19.1 | -16.8 | -3 |
| Club Stab | -18.0 | -18.3 | +2 |
| Octave Runner | -18.5 | -18.0 | |
| Random Glass | -18.4 | -17.7 | +1 |
| Swing Keys | -18.0 | -18.0 | +6 |
| Tempo Wobble | -16.5 | -17.0 | -5 (WOB 8 at 0) |
| Pulse Strings | -17.8 | -18.1 | -6 |
| Gated Chords | -18.1 | -18.2 | +7 |
| Pressure Solo | -18.2 | -18.3 | +8 |
| Morph Bloom | -16.1 | -17.6 | -1 |
| Bubble Computer | -17.7 | -18.3 | |

Loudness is the momentary maximum in LUFS. A run differs from the next by up to about 1 dB, each
new instance having its own parts (ENTROPY). Pitch is a held middle C (`CA72_ONE_NOTE`, the
arpeggiator off, the loudness held for the plucked ones), or the phrase's first held note for
the mono presets. A run's pitch differs from the next by up to about 5 cents. Two things the
numbers taught:

- Acid Squelch's EMPHASIS stops at 7: at 7.6 the filter rang a few cents flat of the note.
- Pressure Solo's pressure reaches the filter (a full press doubles the zero-crossing rate
  mid-note), but loudness barely moves with it: the momentary maximum is the attack's.

## Before any goes into the factory set

`preset_levels.rs`, `factory_curves.rs` and the other factory measurements build the engine's
controls from a preset by `tests/common/mod.rs`'s `controls_of`, which stops at any parameter
it does not know: every 0.3 parameter. A factory preset that uses the arpeggiator, an LFO or a
macro fails them until they play presets through `process`, as this test does. That is the
change to make with the first candidate kept.
