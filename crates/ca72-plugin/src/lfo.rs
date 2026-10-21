//! The LFO page's engine (decisions.md R-LFO; docs/lfo.md): four LFOs, each a plain
//! mathematical generator, not a circuit (the owner, 2026-10-20: "if there is a cheaper way to
//! do these oscillators so that we don't have to simulate an entire circuit for each of them,
//! this is fine because this is no longer on the audio path"), each modulating up to
//! [`MOST`] of the panel's continuous knobs by a depth a share of the knob's travel.
//!
//! - **The generators** ([`wave`]): a phase accumulator through the WAVEFORM switch's six
//!   legends' exact shapes (TRIANGLE, the shark tooth, SAWTOOTH, SQUARE and the two
//!   rectangles), bipolar about zero from -1 to 1, STEPPED or CONTINUOUS (R-CONTINUOUS-WAVE's
//!   travel: each shape blended into the next, past SQUARE the duty narrowed to the floor).
//!   PHASE is added to the phase.
//! - **Timing**: RATE, 0.01 to 50 Hz, or timed, its cycle a DIVISION on the arpeggiator's
//!   clock as the timed oscillators' (R-OSC-TIMED): FREE timed takes its phase from the host's
//!   place every block, never drifting.
//! - **MODE**: FREE runs on; RETRIG starts again at PHASE at a new note; ENV runs one cycle
//!   from PHASE and holds its last value (timed, one note value long). RETRIG starts phase and
//!   RISE on every note-on, independently of the panel's main RETRIGGER. Shared ENV and FREE's
//!   RISE follow that main switch: off, a key from silence; on, every key.
//! - **RISE** fades the output in from a new note over 0 to 10 s.
//! - **MONO** on: one LFO every voice shares. Off: one a voice for the targets each voice has
//!   of its own (its panel); a target the voices share ([`Target::per_voice`] false: VOLUME,
//!   the strip's) always takes the shared one (the owner's option C).
//! - **EDGES**: SOFT (the default) slews the output, a jump taking about [`SOFT_EDGE`] (10 to
//!   90 %); SHARP does not.
//! - **The modulation**: each modulated knob's travel (after its glide, R-KNOB-GLIDE) plus the
//!   sum of depth × LFO, clamped to the travel, through the knob's own law
//!   ([`Target::write`]), every [`STEP`] samples where the engine gives the voices their panel,
//!   a voice's own where MONO is off.
//! - **Nothing runs** without an assignment of a depth other than 0: the engine is as before
//!   to the bit, and the sleeping instrument (R68) is not woken by an LFO.
//!
//! The editor's side (the page, the rings, the drag): [`Patch`] holds the assignments and
//! EDGES (patch state, saved with presets and sessions, not host parameters); [`Scope`]
//! carries each LFO's present value and phase from the audio thread; [`target_of`] says
//! which panel controls can take an LFO now; [`Phosphor`] is each LFO's colour, a setting of
//! the computer's (`settings::Settings::lfo_colors`).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};

use ca72::freq_dial::{DIAL_END, Law, Pot};
use ca72::voice::{Panel, SHAPE_PLACES, shape_bias, shape_snapped};
use nih_plug::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::{Controls, DRIVE_TOP, POLY_VOICES};
use crate::params::{Ca72Params, DOUBLED_AT_NONE, Division, Wave, WaveMode};
use crate::timed::{Length, Position};

/// The LFOs.
pub const LFOS: usize = 4;
/// The most knobs one LFO modulates (the owner: "Sure").
pub const MOST: usize = 8;
/// The depth an assignment dropped on a knob starts at, a share of its travel (Pigments'
/// convention: the owner, 2026-10-20).
pub const DEFAULT_DEPTH: f32 = 0.25;
/// RATE's range, Hz (the owner: "RATE 0.01–50 Hz"), and a new patch's.
pub const RATE_HZ: (f32, f32) = (0.01, 50.0);
pub const DEFAULT_RATE: f32 = 2.0;
/// RISE's longest, s.
pub const RISE_MOST: f32 = 10.0;
/// EDGES SOFT: how long a jump of the output takes from 10 to 90 % of its way, s (a one-pole
/// slew, its time constant this over ln 9).
pub const SOFT_EDGE: f64 = 0.002;
/// How often the modulation reaches the voices, samples: the knobs' glide's step.
pub const STEP: usize = crate::smooth::STEP;
/// The voices that can each have their own LFOs (POLY's most).
pub const VOICES: usize = POLY_VOICES.2;

// ---- What an LFO can modulate.

/// A knob an LFO can modulate: the panel's continuous knobs and the strip's (27), each
/// oscillator's WAVEFORM while it is CONTINUOUS (its shape's travel), and each LFO's own RATE,
/// PHASE, RISE, CONTINUOUS shape and LEVEL (R-LFO's addendum). Its parameter's ID
/// ([`Target::id`]) is what presets and sessions save.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Tune,
    Glide,
    ModMix,
    Osc2Frequency,
    Osc3Frequency,
    Osc1Volume,
    ExtVolume,
    Osc2Volume,
    NoiseVolume,
    Osc3Volume,
    Cutoff,
    Emphasis,
    ContourAmount,
    FilterAttack,
    FilterDecay,
    FilterSustain,
    LoudnessAttack,
    LoudnessDecay,
    LoudnessSustain,
    Volume,
    Feedback,
    Entropy,
    Spread,
    Double,
    Drive,
    Inner,
    Osc1Shape,
    Osc2Shape,
    Osc3Shape,
    /// The strip's OUTPUT LEVEL (R-LFO's corrections: the owner, 2026-10-20, "and entropy,
    /// drive, and output"), last of the panel's, so that no target's index moves.
    Level,
    /// LFO `n`'s (0 to 3) own knobs, moved by the macros and by the LFOs, itself among them
    /// (decisions.md R-LFO's addendum; the owner, 2026-10-20: "it might be interesting to allow
    /// macros to control LFO parameters. And LFOs to control each other's parameters"): its
    /// RATE (not while it is timed, as a FREQUENCY), its PHASE, its RISE, its WAVEFORM's shape
    /// while CONTINUOUS, and its LEVEL (`lfoN_level`, the OUT jack's: the share of every one of
    /// its assignments it gives). After the panel's, so that none of theirs moves.
    LfoRate(usize),
    LfoPhase(usize),
    LfoRise(usize),
    LfoShape(usize),
    LfoLevel(usize),
    /// The standalone's TEMPO (decisions.md R-LFO; the owner, 2026-10-20: "in standalone mode,
    /// lfo or macro should also be able to control the BPM, I think"): its travel 30 to 300 BPM
    /// straight along it (`params::TEMPO_RANGE`), moved on the LFOs' ticks ([`TICK`]) as an
    /// LFO's own knobs are, and everything the tempo times following the beats it adds up. The
    /// plug-ins follow their host's tempo: there it is never applied ([`controls`] leaves its
    /// assignments out), though a patch keeps them. After the LFOs' own, so that no target's
    /// index moves.
    Tempo,
    /// A pinned parameter of the standalone's effects, `n` 0 to [`FX_PINS`] less 1 (decisions.md
    /// R-FX-PARAMS; the owner, 2026-10-20: "expose plugin parameters so that those can also be
    /// modulated by our LFOs and Macros"): a fixed target the effects' chain pins a parameter
    /// to, which stays the pin's while its effect moves. Last, so that no other's index moves.
    /// Its assignments are the chain's (saved with it, `effects.json`), not the sound's: a
    /// preset or session's assignments leave them as they are ([`Patch::load`]).
    Fx(usize),
}

/// An LFO's own knobs another source can move ([`Target::LfoRate`] and the rest), in the order
/// their targets come.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LfoKnob {
    Rate,
    Phase,
    Rise,
    Shape,
    Level,
}

impl LfoKnob {
    pub const ALL: [LfoKnob; LFO_KNOBS] = [
        LfoKnob::Rate,
        LfoKnob::Phase,
        LfoKnob::Rise,
        LfoKnob::Shape,
        LfoKnob::Level,
    ];

    /// Its parameter's ID's last part (`lfo2_rate`): `shape` is its WAVEFORM's CONTINUOUS
    /// shape, `level` its LEVEL.
    pub fn part(self) -> &'static str {
        match self {
            LfoKnob::Rate => "rate",
            LfoKnob::Phase => "phase",
            LfoKnob::Rise => "rise",
            LfoKnob::Shape => "shape",
            LfoKnob::Level => "level",
        }
    }
}

/// The panel's and the strip's targets (theirs first in [`Target::ALL`]).
pub const KNOB_TARGETS: usize = 30;
/// Each LFO's own targets ([`LfoKnob`]).
pub const LFO_KNOBS: usize = 5;
/// The effects' pinned parameters' targets ([`Target::Fx`]): four for each of the chain's four
/// effects (`ca72_fx::MAX_PINS`).
pub const FX_PINS: usize = 16;
/// TEMPO's place in [`Target::ALL`], after the LFOs' own; the effects' pins' first, after it.
const TEMPO_AT: usize = KNOB_TARGETS + LFOS * LFO_KNOBS;
const FX_AT: usize = TEMPO_AT + 1;
/// How many targets there are: the panel's and the strip's, each LFO's own, TEMPO, the
/// effects' pins.
pub const TARGETS: usize = FX_AT + FX_PINS;

/// The pin targets that can take a source now, a bit each ([`target_of`]): those pinned to
/// an effect that runs, set by the standalone's effects (none in the plug-ins).
pub static FX_PINNED: AtomicU32 = AtomicU32::new(0);

/// The panel's and the strip's targets, [`Target::ALL`]'s first.
const PANEL_TARGETS: [Target; KNOB_TARGETS] = [
    Target::Tune,
    Target::Glide,
    Target::ModMix,
    Target::Osc2Frequency,
    Target::Osc3Frequency,
    Target::Osc1Volume,
    Target::ExtVolume,
    Target::Osc2Volume,
    Target::NoiseVolume,
    Target::Osc3Volume,
    Target::Cutoff,
    Target::Emphasis,
    Target::ContourAmount,
    Target::FilterAttack,
    Target::FilterDecay,
    Target::FilterSustain,
    Target::LoudnessAttack,
    Target::LoudnessDecay,
    Target::LoudnessSustain,
    Target::Volume,
    Target::Feedback,
    Target::Entropy,
    Target::Spread,
    Target::Double,
    Target::Drive,
    Target::Inner,
    Target::Osc1Shape,
    Target::Osc2Shape,
    Target::Osc3Shape,
    Target::Level,
];

/// Every target: the panel's and the strip's, then each LFO's own, LFO by LFO, then the
/// effects' pins.
const fn all_targets() -> [Target; TARGETS] {
    let mut all = [Target::Tune; TARGETS];
    let mut i = 0;
    while i < KNOB_TARGETS {
        all[i] = PANEL_TARGETS[i];
        i += 1;
    }
    let mut n = 0;
    while n < LFOS {
        let at = KNOB_TARGETS + LFO_KNOBS * n;
        all[at] = Target::LfoRate(n);
        all[at + 1] = Target::LfoPhase(n);
        all[at + 2] = Target::LfoRise(n);
        all[at + 3] = Target::LfoShape(n);
        all[at + 4] = Target::LfoLevel(n);
        n += 1;
    }
    all[TEMPO_AT] = Target::Tempo;
    let mut k = 0;
    while k < FX_PINS {
        all[FX_AT + k] = Target::Fx(k);
        k += 1;
    }
    all
}

impl Target {
    pub const ALL: [Target; TARGETS] = all_targets();

    /// LFO `n`'s knob `k` as a target.
    pub fn of_lfo(n: usize, k: LfoKnob) -> Target {
        match k {
            LfoKnob::Rate => Target::LfoRate(n),
            LfoKnob::Phase => Target::LfoPhase(n),
            LfoKnob::Rise => Target::LfoRise(n),
            LfoKnob::Shape => Target::LfoShape(n),
            LfoKnob::Level => Target::LfoLevel(n),
        }
    }

    /// The LFO and its knob it is, if it is one of an LFO's own.
    pub fn lfo_knob(self) -> Option<(usize, LfoKnob)> {
        match self {
            Target::LfoRate(n) => Some((n, LfoKnob::Rate)),
            Target::LfoPhase(n) => Some((n, LfoKnob::Phase)),
            Target::LfoRise(n) => Some((n, LfoKnob::Rise)),
            Target::LfoShape(n) => Some((n, LfoKnob::Shape)),
            Target::LfoLevel(n) => Some((n, LfoKnob::Level)),
            _ => None,
        }
    }

    /// Whether it is one of an LFO's own (moving the LFO, not the sound's controls).
    pub fn is_lfo(self) -> bool {
        self.lfo_knob().is_some()
    }

    /// The effects' pin it is, if it is one (R-FX-PARAMS).
    pub fn fx(self) -> Option<usize> {
        match self {
            Target::Fx(k) => Some(k),
            _ => None,
        }
    }

    /// Whether it is an effects' pin (moving an effect, not the synth).
    pub fn is_fx(self) -> bool {
        self.fx().is_some()
    }

    /// Its place in [`Target::ALL`].
    pub fn index(self) -> usize {
        match self {
            Target::Tune => 0,
            Target::Glide => 1,
            Target::ModMix => 2,
            Target::Osc2Frequency => 3,
            Target::Osc3Frequency => 4,
            Target::Osc1Volume => 5,
            Target::ExtVolume => 6,
            Target::Osc2Volume => 7,
            Target::NoiseVolume => 8,
            Target::Osc3Volume => 9,
            Target::Cutoff => 10,
            Target::Emphasis => 11,
            Target::ContourAmount => 12,
            Target::FilterAttack => 13,
            Target::FilterDecay => 14,
            Target::FilterSustain => 15,
            Target::LoudnessAttack => 16,
            Target::LoudnessDecay => 17,
            Target::LoudnessSustain => 18,
            Target::Volume => 19,
            Target::Feedback => 20,
            Target::Entropy => 21,
            Target::Spread => 22,
            Target::Double => 23,
            Target::Drive => 24,
            Target::Inner => 25,
            Target::Osc1Shape => 26,
            Target::Osc2Shape => 27,
            Target::Osc3Shape => 28,
            Target::Level => 29,
            Target::Tempo => TEMPO_AT,
            Target::Fx(k) => FX_AT + k.min(FX_PINS - 1),
            t => {
                let (n, k) = t.lfo_knob().unwrap_or((0, LfoKnob::Rate));
                KNOB_TARGETS + LFO_KNOBS * n.min(LFOS - 1) + k as usize
            }
        }
    }

    /// Its parameter's ID (what presets and sessions save; the editor's control's parameter,
    /// but WAVEFORM's, whose target is its CONTINUOUS shape, `oscN_shape`).
    pub fn id(self) -> &'static str {
        match self {
            Target::Tune => "tune",
            Target::Glide => "glide",
            Target::ModMix => "mod_mix",
            Target::Osc2Frequency => "osc2_frequency",
            Target::Osc3Frequency => "osc3_frequency",
            Target::Osc1Volume => "osc1_volume",
            Target::ExtVolume => "ext_volume",
            Target::Osc2Volume => "osc2_volume",
            Target::NoiseVolume => "noise_volume",
            Target::Osc3Volume => "osc3_volume",
            Target::Cutoff => "cutoff",
            Target::Emphasis => "emphasis",
            Target::ContourAmount => "contour_amount",
            Target::FilterAttack => "filter_attack",
            Target::FilterDecay => "filter_decay",
            Target::FilterSustain => "filter_sustain",
            Target::LoudnessAttack => "loudness_attack",
            Target::LoudnessDecay => "loudness_decay",
            Target::LoudnessSustain => "loudness_sustain",
            Target::Volume => "volume",
            Target::Feedback => "feedback",
            Target::Entropy => "entropy",
            Target::Spread => "spread",
            Target::Double => "double",
            Target::Drive => "drive",
            Target::Inner => "inner",
            Target::Osc1Shape => "osc1_shape",
            Target::Osc2Shape => "osc2_shape",
            Target::Osc3Shape => "osc3_shape",
            Target::Level => "level",
            Target::Tempo => "tempo",
            Target::Fx(k) => FX_TARGET_IDS[k.min(FX_PINS - 1)],
            t => {
                let (n, k) = t.lfo_knob().unwrap_or((0, LfoKnob::Rate));
                LFO_TARGET_IDS[n.min(LFOS - 1)][k as usize]
            }
        }
    }

    /// The target whose parameter's ID is `id`.
    pub fn of_id(id: &str) -> Option<Target> {
        Target::ALL.into_iter().find(|t| t.id() == id)
    }

    /// A short name for MODULATES' screen (at most ten characters, the panel's words).
    pub fn short(self) -> &'static str {
        match self {
            Target::Tune => "TUNE",
            Target::Glide => "GLIDE",
            Target::ModMix => "MOD MIX",
            Target::Osc2Frequency => "OSC 2 FREQ",
            Target::Osc3Frequency => "OSC 3 FREQ",
            Target::Osc1Volume => "OSC 1 VOL",
            Target::ExtVolume => "EXT VOL",
            Target::Osc2Volume => "OSC 2 VOL",
            Target::NoiseVolume => "NOISE VOL",
            Target::Osc3Volume => "OSC 3 VOL",
            Target::Cutoff => "CUTOFF",
            Target::Emphasis => "EMPHASIS",
            Target::ContourAmount => "CONTOUR",
            Target::FilterAttack => "F ATTACK",
            Target::FilterDecay => "F DECAY",
            Target::FilterSustain => "F SUSTAIN",
            Target::LoudnessAttack => "L ATTACK",
            Target::LoudnessDecay => "L DECAY",
            Target::LoudnessSustain => "L SUSTAIN",
            Target::Volume => "VOLUME",
            Target::Feedback => "FEEDBACK",
            Target::Entropy => "ENTROPY",
            Target::Spread => "WIDTH",
            Target::Double => "DETUNE",
            Target::Drive => "DRIVE",
            Target::Inner => "INNER",
            Target::Osc1Shape => "OSC 1 WAVE",
            Target::Osc2Shape => "OSC 2 WAVE",
            Target::Osc3Shape => "OSC 3 WAVE",
            Target::Level => "LEVEL",
            Target::Tempo => "TEMPO",
            // (The pin's own name is the effects' to give: its parameter's.)
            Target::Fx(k) => FX_TARGET_SHORT[k.min(FX_PINS - 1)],
            // (Ten characters: "LFO2 LEVEL", apart from the strip's LEVEL.)
            t => {
                let (n, k) = t.lfo_knob().unwrap_or((0, LfoKnob::Rate));
                LFO_TARGET_SHORT[n.min(LFOS - 1)][k as usize]
            }
        }
    }

    /// Its name as the agents and the menus give it: an LFO's own with its LFO's number
    /// ("LFO 2 LEVEL", "LFO 1 RATE"), apart from the strip's LEVEL; the others their short
    /// names.
    pub fn name(self) -> String {
        match self.lfo_knob() {
            Some((n, k)) => format!("LFO {} {}", n + 1, k.part().to_uppercase()),
            None => self.short().to_owned(),
        }
    }

    /// Whether each voice has its own (it is on the voice's panel, or FEEDBACK, each voice's
    /// own loop): an LFO with MONO off moves it a voice at a time. The others the voices share
    /// (MAIN OUTPUT's VOLUME, the strip's ENTROPY, WIDTH, INNER, DETUNE, DRIVE and LEVEL), which
    /// take the LFO's shared copy whatever MONO is (option C). An LFO's own knobs are on no
    /// voice's panel: each copy of that LFO has its own (per voice where its MONO is off:
    /// [`takes_shared`]).
    pub fn per_voice(self) -> bool {
        !matches!(
            self,
            Target::Volume
                | Target::Level
                | Target::Entropy
                | Target::Spread
                | Target::Double
                | Target::Drive
                | Target::Inner
                | Target::LfoRate(_)
                | Target::LfoPhase(_)
                | Target::LfoRise(_)
                | Target::LfoShape(_)
                | Target::LfoLevel(_)
                | Target::Tempo
                | Target::Fx(_)
        )
    }

    /// Whether it runs on the LFOs' ticks ([`TICK`]): an LFO's own knob, or TEMPO (the clock
    /// the timed LFOs keep).
    pub fn ticks(self) -> bool {
        self.is_lfo() || self == Target::Tempo
    }

    /// Its parameter (none for an effects' pin, which is its effect's: R-FX-PARAMS).
    pub fn param(self, p: &Ca72Params) -> Option<&FloatParam> {
        Some(match self {
            Target::Tune => &p.tune,
            Target::Glide => &p.glide,
            Target::ModMix => &p.mod_mix,
            Target::Osc2Frequency => &p.osc2_frequency,
            Target::Osc3Frequency => &p.osc3_frequency,
            Target::Osc1Volume => &p.osc1_volume,
            Target::ExtVolume => &p.ext_volume,
            Target::Osc2Volume => &p.osc2_volume,
            Target::NoiseVolume => &p.noise_volume,
            Target::Osc3Volume => &p.osc3_volume,
            Target::Cutoff => &p.cutoff,
            Target::Emphasis => &p.emphasis,
            Target::ContourAmount => &p.contour_amount,
            Target::FilterAttack => &p.filter_attack,
            Target::FilterDecay => &p.filter_decay,
            Target::FilterSustain => &p.filter_sustain,
            Target::LoudnessAttack => &p.loudness_attack,
            Target::LoudnessDecay => &p.loudness_decay,
            Target::LoudnessSustain => &p.loudness_sustain,
            Target::Volume => &p.volume,
            Target::Feedback => &p.feedback,
            Target::Entropy => &p.entropy,
            Target::Spread => &p.spread,
            Target::Double => &p.double,
            Target::Drive => &p.drive,
            Target::Inner => &p.inner,
            Target::Osc1Shape => &p.osc1_shape,
            Target::Osc2Shape => &p.osc2_shape,
            Target::Osc3Shape => &p.osc3_shape,
            Target::Level => &p.level,
            Target::LfoRate(n) => &p.lfo(n).rate,
            Target::LfoPhase(n) => &p.lfo(n).phase,
            Target::LfoRise(n) => &p.lfo(n).rise,
            Target::LfoShape(n) => &p.lfo(n).shape,
            Target::LfoLevel(n) => &p.lfo_level(n).level,
            Target::Tempo => &p.tempo_knob,
            Target::Fx(_) => return None,
        })
    }

    /// The FREQUENCY pot it is, if it is one (its knob's travel read through the computer's
    /// law, R54).
    fn pot(self) -> Option<Pot> {
        match self {
            Target::Osc2Frequency => Some(Pot::Two),
            Target::Osc3Frequency => Some(Pot::Three),
            _ => None,
        }
    }

    /// The knob's travel (0..1, as its pointer turns) with its parameter at `plain` (its
    /// dial's units): the parameter's normalized value, a FREQUENCY knob's through `law`.
    pub fn travel(self, p: &Ca72Params, plain: f32, law: Law) -> f32 {
        let Some(param) = self.param(p) else {
            return plain.clamp(0.0, 1.0);
        };
        let n = param.preview_normalized(plain);
        match self.pot() {
            Some(pot) => pot.knob_at(law, crate::params::wiper_of(plain)) as f32,
            None => n,
        }
    }

    /// The parameter's plain value (its dial's units) with the knob at `travel` (0..1): the
    /// inverse of [`Target::travel`].
    pub fn plain_at(self, p: &Ca72Params, travel: f32, law: Law) -> f32 {
        let t = travel.clamp(0.0, 1.0);
        match self.pot() {
            Some(pot) => ((2.0 * pot.wiper_at(law, f64::from(t)) - 1.0) * DIAL_END) as f32,
            None => self.param(p).map_or(t, |q| q.preview_plain(t)),
        }
    }

    /// The engine's controls with this knob at `travel` (0..1), through its law as
    /// `Ca72Params::controls_with` takes its parameter (the same to within an f32's rounding:
    /// `every_targets_law_is_the_parameters`).
    pub fn write(self, c: &mut Controls, travel: f64, law: Law) {
        let t = travel.clamp(0.0, 1.0);
        match self {
            Target::Volume => c.volume = t,
            Target::Entropy => c.entropy = t,
            Target::Spread => c.spread = t,
            Target::Inner => c.inner = t,
            // (DETUNE does nothing with DOUBLE off, SCATTER: then it stays off.)
            Target::Double => {
                if c.double > 0.0 {
                    c.double = t.max(DOUBLED_AT_NONE);
                }
            }
            Target::Drive => c.drive = t * DRIVE_TOP,
            // (Its dB straight along its travel, as its parameter's range is.)
            Target::Level => {
                c.level = crate::engine::LEVEL_RANGE.0
                    + t * (crate::engine::LEVEL_RANGE.1 - crate::engine::LEVEL_RANGE.0);
            }
            Target::Feedback => c.feedback = ca72::voice::feedback_law(t),
            // (An LFO's own: the LFOs' generators take them, `Bank`'s ticks, not the controls;
            // an effects' pin, the effects' chain.)
            Target::LfoRate(_)
            | Target::LfoPhase(_)
            | Target::LfoRise(_)
            | Target::LfoShape(_)
            | Target::LfoLevel(_)
            | Target::Tempo
            | Target::Fx(_) => {}
            _ => self.write_panel(&mut c.panel, t, law),
        }
    }

    /// A voice's panel with this knob at `travel`, for the targets on the panel
    /// ([`Target::write`]'s; FEEDBACK, which is not, is [`Target::write`]'s alone).
    fn write_panel(self, p: &mut Panel, t: f64, law: Law) {
        match self {
            Target::Tune => p.tune = 2.0 * t - 1.0,
            Target::Glide => p.glide = t,
            Target::ModMix => p.mod_mix = t,
            Target::Osc2Frequency => p.osc[1].freq = Pot::Two.wiper_at(law, t),
            Target::Osc3Frequency => p.osc[2].freq = Pot::Three.wiper_at(law, t),
            Target::Osc1Volume => p.osc[0].volume = t,
            Target::ExtVolume => p.ext_volume = t,
            Target::Osc2Volume => p.osc[1].volume = t,
            Target::NoiseVolume => p.noise_volume = t,
            Target::Osc3Volume => p.osc[2].volume = t,
            Target::Cutoff => p.cutoff = t,
            Target::Emphasis => p.emphasis = t,
            Target::ContourAmount => p.contour_amount = t,
            Target::FilterAttack => p.filter_contour.attack = t,
            Target::FilterDecay => p.filter_contour.decay = t,
            Target::FilterSustain => p.filter_contour.sustain = t,
            Target::LoudnessAttack => p.loudness_contour.attack = t,
            Target::LoudnessDecay => p.loudness_contour.decay = t,
            Target::LoudnessSustain => p.loudness_contour.sustain = t,
            Target::Osc1Shape => p.shape[0].at = shape_snapped(t * SHAPE_PLACES),
            Target::Osc2Shape => p.shape[1].at = shape_snapped(t * SHAPE_PLACES),
            Target::Osc3Shape => p.shape[2].at = shape_snapped(t * SHAPE_PLACES),
            Target::Volume
            | Target::Level
            | Target::Feedback
            | Target::Entropy
            | Target::Spread
            | Target::Double
            | Target::Drive
            | Target::Inner
            | Target::LfoRate(_)
            | Target::LfoPhase(_)
            | Target::LfoRise(_)
            | Target::LfoShape(_)
            | Target::LfoLevel(_)
            | Target::Tempo
            | Target::Fx(_) => {}
        }
    }
}

/// TEMPO's travel (0..1, straight along its range) at `bpm`.
pub fn tempo_travel(bpm: f64) -> f64 {
    let (lo, hi) = crate::params::TEMPO_RANGE;
    ((bpm - lo) / (hi - lo)).clamp(0.0, 1.0)
}

/// The tempo (BPM) with TEMPO's travel at `travel`, clamped to its range.
pub fn tempo_at(travel: f64) -> f64 {
    let (lo, hi) = crate::params::TEMPO_RANGE;
    lo + (hi - lo) * travel.clamp(0.0, 1.0)
}
/// The effects' pins' targets' IDs (R-FX-PARAMS).
const FX_TARGET_IDS: [&str; FX_PINS] = [
    "fx1", "fx2", "fx3", "fx4", "fx5", "fx6", "fx7", "fx8", "fx9", "fx10", "fx11", "fx12", "fx13",
    "fx14", "fx15", "fx16",
];

/// Their names on a MODULATES screen where the effects give none.
const FX_TARGET_SHORT: [&str; FX_PINS] = [
    "FX PIN 1",
    "FX PIN 2",
    "FX PIN 3",
    "FX PIN 4",
    "FX PIN 5",
    "FX PIN 6",
    "FX PIN 7",
    "FX PIN 8",
    "FX PIN 9",
    "FX PIN 10",
    "FX PIN 11",
    "FX PIN 12",
    "FX PIN 13",
    "FX PIN 14",
    "FX PIN 15",
    "FX PIN 16",
];

/// Each LFO's own targets' IDs, [`LfoKnob`]'s order: its parameters' (its WAVEFORM's
/// CONTINUOUS shape `lfoN_shape`, its LEVEL `lfoN_level`).
const LFO_TARGET_IDS: [[&str; LFO_KNOBS]; LFOS] = [
    [
        "lfo1_rate",
        "lfo1_phase",
        "lfo1_rise",
        "lfo1_shape",
        "lfo1_level",
    ],
    [
        "lfo2_rate",
        "lfo2_phase",
        "lfo2_rise",
        "lfo2_shape",
        "lfo2_level",
    ],
    [
        "lfo3_rate",
        "lfo3_phase",
        "lfo3_rise",
        "lfo3_shape",
        "lfo3_level",
    ],
    [
        "lfo4_rate",
        "lfo4_phase",
        "lfo4_rise",
        "lfo4_shape",
        "lfo4_level",
    ],
];

/// Their names on a MODULATES screen (ten characters at most).
const LFO_TARGET_SHORT: [[&str; LFO_KNOBS]; LFOS] = [
    [
        "LFO1 RATE",
        "LFO1 PHASE",
        "LFO1 RISE",
        "LFO1 WAVE",
        "LFO1 LEVEL",
    ],
    [
        "LFO2 RATE",
        "LFO2 PHASE",
        "LFO2 RISE",
        "LFO2 WAVE",
        "LFO2 LEVEL",
    ],
    [
        "LFO3 RATE",
        "LFO3 PHASE",
        "LFO3 RISE",
        "LFO3 WAVE",
        "LFO3 LEVEL",
    ],
    [
        "LFO4 RATE",
        "LFO4 PHASE",
        "LFO4 RISE",
        "LFO4 WAVE",
        "LFO4 LEVEL",
    ],
];

/// Whether source `s` moves target `t` by its shared copy where it has one of each voice's own
/// (MONO off): `t` a control the voices share, or an LFO's own knob of an LFO with MONO on (one
/// copy every voice shares: option C). False for a macro, or an LFO with MONO on.
pub fn takes_shared(p: &Ca72Params, s: Source, t: Target) -> bool {
    let own = s.lfo().is_some_and(|n| !p.lfo(n).mono.value());
    own && match t.lfo_knob() {
        Some((n, _)) => p.lfo(n).mono.value(),
        None => !t.per_voice(),
    }
}

/// The target a panel control can take an LFO on now, by the control's parameter's ID: a
/// continuous knob of the panel's or the strip's; a WAVEFORM switch only while CONTINUOUS
/// (`oscN_waveform` gives its shape); a FREQUENCY knob not while its oscillator is timed (it is
/// a DIVISION dial then, stepped); an LFO's RATE, PHASE, RISE and LEVEL (its OUT jack's), its
/// RATE not while it is timed, its WAVEFORM only while CONTINUOUS (`lfoN_waveform` gives its
/// shape: R-LFO's addendum). None for anything else: switches, stepped selectors, VOICES, the
/// timed dials, the wheels and the drawers' knobs. An assignment already made stays while its
/// knob cannot take one (it does nothing then).
pub fn target_of(p: &Ca72Params, param_id: &str) -> Option<Target> {
    let wave = ["osc1_waveform", "osc2_waveform", "osc3_waveform"];
    if let Some(n) = wave.iter().position(|w| *w == param_id) {
        return p
            .continuous(n)
            .then_some([Target::Osc1Shape, Target::Osc2Shape, Target::Osc3Shape][n]);
    }
    // (An LFO's WAVEFORM likewise, its CONTINUOUS shape: R-LFO's addendum.)
    let lfo_wave = [
        "lfo1_waveform",
        "lfo2_waveform",
        "lfo3_waveform",
        "lfo4_waveform",
    ];
    if let Some(n) = lfo_wave.iter().position(|w| *w == param_id) {
        return (p.lfo(n).shape_mode.value() == WaveMode::Continuous)
            .then_some(Target::LfoShape(n));
    }
    let t = Target::of_id(param_id)?;
    match t {
        Target::Osc1Shape | Target::Osc2Shape | Target::Osc3Shape | Target::LfoShape(_) => None,
        // (An effects' pin while it is pinned to an effect that runs: R-FX-PARAMS.)
        Target::Fx(k) if FX_PINNED.load(Ordering::Acquire) & (1 << k) == 0 => None,
        Target::Osc2Frequency if p.timed_now(1) => None,
        Target::Osc3Frequency if p.timed_now(2) => None,
        // (Timed, its knob is a DIVISION dial, stepped: as a FREQUENCY.)
        Target::LfoRate(n) if p.lfo(n).timed.value() => None,
        // (The standalone's alone: a plug-in follows its host's tempo.)
        Target::Tempo if !p.is_standalone() => None,
        _ => Some(t),
    }
}

/// [`target_of`] as a yes or no.
pub fn is_lfo_target(p: &Ca72Params, param_id: &str) -> bool {
    target_of(p, param_id).is_some()
}

// ---- Modulation sources.

/// The macros (decisions.md R-MACROS; the owner, "Yes, 2x3 as recommended"): six knobs of the
/// patch's own, host parameters `macro1` to `macro6` (0 to 100 %, gliding as the panel's knobs),
/// each a source as an LFO is, unipolar (its value 0..1, so an assignment moves its knob one way,
/// by depth × value), with a typed name of [`NAME_MOST`] characters saved with the patch.
pub const MACROS: usize = 6;
/// The most characters a macro's name has (its dot screen's seven).
pub const NAME_MOST: usize = 7;
/// Every modulation source: the LFOs, then the macros.
pub const SOURCES: usize = LFOS + MACROS;

/// A modulation source: something with a value that moves the knobs assigned to it, each by
/// its own depth, with rings on them (decisions.md R-LFO). Its index ([`Source::index`]) is
/// its place in the sources' arrays; its ID ([`Source::id`], `lfo1`, `macro1`) what presets
/// and sessions save.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Source {
    /// LFO `n` (0 to 3).
    Lfo(usize),
    /// Macro `n` (0 to [`MACROS`] less 1).
    Macro(usize),
}

impl Source {
    /// Every source, the LFOs first.
    pub fn all() -> impl Iterator<Item = Source> {
        (0..SOURCES).filter_map(Source::of_index)
    }

    /// Its place in the sources' arrays (past [`SOURCES`] for one that is not there).
    pub fn index(self) -> usize {
        match self {
            Source::Lfo(n) if n < LFOS => n,
            Source::Macro(n) => Some(LFOS + n)
                .filter(|i| *i < SOURCES)
                .unwrap_or(usize::MAX),
            Source::Lfo(_) => usize::MAX,
        }
    }

    pub fn of_index(i: usize) -> Option<Source> {
        if i < LFOS {
            Some(Source::Lfo(i))
        } else if i < SOURCES {
            Some(Source::Macro(i - LFOS))
        } else {
            None
        }
    }

    /// As presets and sessions save it: `lfo1` to `lfo4`, `macro1` on.
    pub fn id(self) -> String {
        match self {
            Source::Lfo(n) => format!("lfo{}", n + 1),
            Source::Macro(n) => format!("macro{}", n + 1),
        }
    }

    pub fn of_id(id: &str) -> Option<Source> {
        Source::all().find(|s| s.id() == id)
    }

    /// The LFO it is, if it is one.
    pub fn lfo(self) -> Option<usize> {
        match self {
            Source::Lfo(n) if n < LFOS => Some(n),
            _ => None,
        }
    }

    /// The macro it is, if it is one.
    pub fn macro_n(self) -> Option<usize> {
        match self {
            Source::Macro(n) if n < MACROS => Some(n),
            _ => None,
        }
    }
}

/// A macro's name as it is kept: printable ASCII in capitals (the panel's lettering; anything
/// else left out), the ends' spaces trimmed, at most [`NAME_MOST`] characters. Empty: none (the
/// panel shows MACRO n).
pub fn name_within(text: &str) -> String {
    let kept: String = text
        .chars()
        .map(|c| c.to_ascii_uppercase())
        .filter(|c| (' '..='~').contains(c))
        .collect();
    kept.trim()
        .chars()
        .take(NAME_MOST)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

/// A name as one word: its characters' bytes, the first lowest (none: 0).
fn name_word(name: &str) -> u64 {
    name_within(name)
        .bytes()
        .enumerate()
        .fold(0, |w, (i, b)| w | (u64::from(b) << (8 * i)))
}

fn of_name_word(w: u64) -> String {
    (0..NAME_MOST)
        .map(|i| ((w >> (8 * i)) & 0xff) as u8)
        .take_while(|b| *b != 0)
        .map(char::from)
        .collect()
}

// ---- The patch's part: assignments and EDGES.

/// One knob a source modulates, by `depth` (-1..1, a share of the knob's travel either way:
/// -100 to +100 %; negative inverts the source exactly).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Assignment {
    pub target: Target,
    pub depth: f32,
}

/// EDGES: SOFT (a jump slewed over [`SOFT_EDGE`]) or SHARP (none). An LFO's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Edges {
    #[default]
    Soft,
    Sharp,
}

/// Why an assignment was not made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The source already modulates [`MOST`] knobs.
    Full,
    /// No such source.
    NoSource,
}

/// Every source's assignments, and the LFOs' EDGES: the patch's (saved with presets and
/// sessions, not host parameters: the owner, "Patch state only"). Set from the editor and the
/// agents, read by the audio thread without locks: each slot one atomic word (its target and
/// its depth), and [`Patch::changes`] counting every change.
///
/// The assignments to the effects' pins ([`Target::Fx`]) are kept here too, for the engine and
/// the editor, but are the effects' chain's (decisions.md R-FX-PARAMS: "Pins and their
/// assignments are saved with the chain"): never in [`Patch::saved`] or [`Patch::values`], left
/// as they are by [`Patch::load`], [`Patch::clear`] and [`Patch::copy_from`], and counted apart
/// ([`Patch::fx_changes`]).
#[derive(Debug)]
pub struct Patch {
    slots: [[AtomicU64; MOST]; SOURCES],
    sharp: [AtomicBool; LFOS],
    /// Each macro's typed name, its characters in one word (decisions.md R-MACROS).
    names: [AtomicU64; MACROS],
    changes: AtomicU32,
    fx_changes: AtomicU32,
}

impl Default for Patch {
    fn default() -> Self {
        Patch {
            slots: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            sharp: std::array::from_fn(|_| AtomicBool::new(false)),
            names: std::array::from_fn(|_| AtomicU64::new(0)),
            changes: AtomicU32::new(0),
            fx_changes: AtomicU32::new(0),
        }
    }
}

fn word(a: Option<Assignment>) -> u64 {
    match a {
        None => 0,
        Some(a) => ((a.target.index() as u64 + 1) << 32) | u64::from(a.depth.to_bits()),
    }
}

fn of_word(w: u64) -> Option<Assignment> {
    let k = (w >> 32) as usize;
    let target = *Target::ALL.get(k.checked_sub(1)?)?;
    Some(Assignment {
        target,
        depth: f32::from_bits(w as u32),
    })
}

/// A depth as kept: within -1..1 (not finite: 0).
fn depth_within(d: f32) -> f32 {
    if d.is_finite() {
        d.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

impl Patch {
    /// `source`'s assignments, in the order they were made.
    pub fn assignments(&self, source: Source) -> [Option<Assignment>; MOST] {
        let mut out = [None; MOST];
        if let Some(slots) = self.slots.get(source.index()) {
            let mut n = 0;
            for s in slots {
                if let Some(a) = of_word(s.load(Ordering::Acquire)) {
                    out[n] = Some(a);
                    n += 1;
                }
            }
        }
        out
    }

    /// How many knobs `source` modulates.
    pub fn count(&self, source: Source) -> usize {
        self.assignments(source).iter().flatten().count()
    }

    /// `source`'s depth on `target`, if it modulates it.
    pub fn depth(&self, source: Source, target: Target) -> Option<f32> {
        self.assignments(source)
            .into_iter()
            .flatten()
            .find(|a| a.target == target)
            .map(|a| a.depth)
    }

    /// Each source's depth on `target`, [`Source::index`]'s order (the knob's rings, its
    /// hover dials, its tip and menu).
    pub fn on_knob(&self, target: Target) -> [Option<f32>; SOURCES] {
        std::array::from_fn(|i| Source::of_index(i).and_then(|s| self.depth(s, target)))
    }

    /// `source` set to modulate `target` by `depth` (-1..1): its depth changed if it already
    /// does, else added after the others, if it has room.
    pub fn assign(&self, source: Source, target: Target, depth: f32) -> Result<(), Refused> {
        let slots = self.slots.get(source.index()).ok_or(Refused::NoSource)?;
        let depth = depth_within(depth);
        let new = word(Some(Assignment { target, depth }));
        let same = slots
            .iter()
            .find(|s| of_word(s.load(Ordering::Acquire)).is_some_and(|a| a.target == target));
        let slot = same.or_else(|| slots.iter().find(|s| s.load(Ordering::Acquire) == 0));
        let slot = slot.ok_or(Refused::Full)?;
        slot.store(new, Ordering::Release);
        self.changed_on(target);
        Ok(())
    }

    /// `source`'s depth on `target` set, if it modulates it (false if not).
    pub fn set_depth(&self, source: Source, target: Target, depth: f32) -> bool {
        if self.depth(source, target).is_none() {
            return false;
        }
        self.assign(source, target, depth).is_ok()
    }

    /// `source` no longer modulating `target` (false if it did not).
    pub fn remove(&self, source: Source, target: Target) -> bool {
        let Some(slots) = self.slots.get(source.index()) else {
            return false;
        };
        let mut kept = [None; MOST];
        let mut n = 0;
        let mut gone = false;
        for a in self.assignments(source).into_iter().flatten() {
            if a.target == target {
                gone = true;
            } else {
                kept[n] = Some(a);
                n += 1;
            }
        }
        if gone {
            for (s, a) in slots.iter().zip(kept) {
                s.store(word(a), Ordering::Release);
            }
            self.changed_on(target);
        }
        gone
    }

    /// Source `source`'s words with its assignments to the effects' pins first, in order, then
    /// `rest` (none past [`MOST`]): what a load or a clear leaves it.
    fn keeping_fx(&self, source: Source, rest: &[u64]) -> [u64; MOST] {
        let mut out = [0u64; MOST];
        let mut n = 0;
        for a in self.assignments(source).into_iter().flatten() {
            if a.target.is_fx() {
                out[n] = word(Some(a));
                n += 1;
            }
        }
        for w in rest {
            if n == MOST {
                break;
            }
            if *w != 0 {
                out[n] = *w;
                n += 1;
            }
        }
        out
    }

    /// Every source's every assignment taken away, every EDGES SOFT again, every macro's name
    /// none (Init; a preset or session saved before LFOs).
    pub fn clear(&self) {
        // (The effects' pins' assignments are the chain's: kept.)
        for source in Source::all() {
            let kept = self.keeping_fx(source, &[]);
            for (s, w) in self.slots[source.index()].iter().zip(kept) {
                s.store(w, Ordering::Release);
            }
        }
        for e in &self.sharp {
            e.store(false, Ordering::Release);
        }
        for n in &self.names {
            n.store(0, Ordering::Release);
        }
        self.changed();
    }

    /// Macro `n`'s (0 to 5) typed name; empty where it has none (the panel shows MACRO n).
    pub fn name(&self, n: usize) -> String {
        self.names
            .get(n)
            .map_or_else(String::new, |w| of_name_word(w.load(Ordering::Acquire)))
    }

    /// Macro `n`'s name set ([`name_within`]: an empty one, none).
    pub fn set_name(&self, n: usize, name: &str) {
        if let Some(w) = self.names.get(n) {
            w.store(name_word(name), Ordering::Release);
            self.changed();
        }
    }

    /// Every macro's name, as a preset keeps them ([`Patch::name`]'s, empty for none).
    pub fn names(&self) -> [String; MACROS] {
        std::array::from_fn(|n| self.name(n))
    }

    /// A preset's names in place of these (one that names none: none; past six left out).
    pub fn load_names(&self, names: &[String]) {
        for (k, w) in self.names.iter().enumerate() {
            w.store(names.get(k).map_or(0, |n| name_word(n)), Ordering::Release);
        }
        self.changed();
    }

    /// Whether these names are a preset's (`names`, as [`Patch::load_names`] reads them).
    pub fn same_names(&self, names: &[String]) -> bool {
        (0..MACROS)
            .all(|k| self.name(k) == names.get(k).map_or_else(String::new, |n| name_within(n)))
    }

    /// LFO `lfo`'s (0 to 3) EDGES.
    pub fn edges(&self, lfo: usize) -> Edges {
        match self.sharp.get(lfo) {
            Some(s) if s.load(Ordering::Acquire) => Edges::Sharp,
            _ => Edges::Soft,
        }
    }

    pub fn set_edges(&self, lfo: usize, edges: Edges) {
        if let Some(s) = self.sharp.get(lfo) {
            s.store(edges == Edges::Sharp, Ordering::Release);
            self.changed();
        }
    }

    /// How many changes have been made (the audio thread reads the slots again when it moves).
    pub fn changes(&self) -> u32 {
        self.changes.load(Ordering::Acquire)
    }

    fn changed(&self) {
        self.changes.fetch_add(1, Ordering::AcqRel);
    }

    fn changed_on(&self, target: Target) {
        if target.is_fx() {
            self.fx_changes.fetch_add(1, Ordering::AcqRel);
        }
        self.changed();
    }

    /// How many changes have been made to the effects' pins' assignments (the chain saves
    /// them: R-FX-PARAMS).
    pub fn fx_changes(&self) -> u32 {
        self.fx_changes.load(Ordering::Acquire)
    }

    /// As saved with the session: each source that has an assignment or SHARP EDGES.
    pub fn saved(&self) -> Saved {
        Saved {
            sources: Source::all()
                .map(|s| SavedSource {
                    source: s.id(),
                    to: self
                        .assignments(s)
                        .into_iter()
                        .flatten()
                        .filter(|a| !a.target.is_fx())
                        .map(|a| (a.target.id().to_owned(), a.depth))
                        .collect(),
                    sharp: s.lfo().is_some_and(|l| self.edges(l) == Edges::Sharp),
                })
                .filter(|s| !s.to.is_empty() || s.sharp)
                .collect(),
            names: {
                let names = self.names();
                if names.iter().all(String::is_empty) {
                    Vec::new()
                } else {
                    names.to_vec()
                }
            },
        }
    }

    /// A session's (or a preset's) in place of what it had: what is not understood left out
    /// (a source or target not known, a depth not finite, past [`MOST`] a source).
    pub fn load(&self, saved: &Saved) {
        let mut words = [[0u64; MOST]; SOURCES];
        let mut counts = [0usize; SOURCES];
        let mut sharp = [false; LFOS];
        for s in &saved.sources {
            let Some(source) = Source::of_id(&s.source) else {
                continue;
            };
            let i = source.index();
            if let Some(l) = source.lfo() {
                sharp[l] |= s.sharp;
            }
            for (id, depth) in &s.to {
                // (The effects' pins' are the chain's: R-FX-PARAMS.)
                let Some(target) = Target::of_id(id).filter(|t| !t.is_fx()) else {
                    continue;
                };
                let n = counts[i];
                let taken = words[i][..n]
                    .iter()
                    .any(|w| of_word(*w).is_some_and(|a| a.target == target));
                if n == MOST || !depth.is_finite() || taken {
                    continue;
                }
                words[i][n] = word(Some(Assignment {
                    target,
                    depth: depth_within(*depth),
                }));
                counts[i] += 1;
            }
        }
        for (source, w) in Source::all().zip(words) {
            let kept = self.keeping_fx(source, &w);
            for (slot, w) in self.slots[source.index()].iter().zip(kept) {
                slot.store(w, Ordering::Release);
            }
        }
        for (s, v) in self.sharp.iter().zip(sharp) {
            s.store(v, Ordering::Release);
        }
        for (k, w) in self.names.iter().enumerate() {
            w.store(
                saved.names.get(k).map_or(0, |n| name_word(n)),
                Ordering::Release,
            );
        }
        self.changed();
    }

    /// As a preset's values (with the parameters' own): `<source>_to_<target>` (`lfo1_to_cutoff`),
    /// the depth in percent, for each assignment, and `lfoN_edges` (1, SHARP) where it is SHARP.
    pub fn values(&self) -> Vec<(String, f64)> {
        let mut out = Vec::new();
        for s in Source::all() {
            for a in self.assignments(s).into_iter().flatten() {
                if a.target.is_fx() {
                    continue;
                }
                let pct = (f64::from(a.depth) * 100.0 * 1e4).round() / 1e4;
                out.push((format!("{}_to_{}", s.id(), a.target.id()), pct));
            }
            if s.lfo().is_some_and(|l| self.edges(l) == Edges::Sharp) {
                out.push((format!("{}_edges", s.id()), 1.0));
            }
        }
        out
    }

    /// A preset's assignments and EDGES (its [`Patch::values`]) in place of what it had: a
    /// preset that names none, nothing assigned and every EDGES SOFT. In the targets' order. Its
    /// macros' names are not values: [`Patch::load_names`] (none until then).
    pub fn load_values(&self, values: &[(String, f64)]) {
        let mut saved = Saved::default();
        for s in Source::all() {
            let prefix = format!("{}_to_", s.id());
            let edges = format!("{}_edges", s.id());
            let mut to: Vec<(Target, f32)> = values
                .iter()
                .filter_map(|(k, v)| {
                    let t = Target::of_id(k.strip_prefix(&prefix)?)?;
                    Some((t, (*v / 100.0) as f32))
                })
                .collect();
            to.sort_by_key(|(t, _)| *t);
            saved.sources.push(SavedSource {
                source: s.id(),
                to: to
                    .into_iter()
                    .map(|(t, d)| (t.id().to_owned(), d))
                    .collect(),
                sharp: values.iter().any(|(k, v)| *k == edges && *v >= 0.5),
            });
        }
        self.load(&saved);
    }

    /// Whether these are what a preset's values (`values`, its [`Patch::values`]) give: the
    /// same assignments, in any order, their depths within a hundredth of a percent, and the same
    /// EDGES (the presets' bar marks a preset changed where not).
    pub fn same_as_values(&self, values: &[(String, f64)]) -> bool {
        let theirs = Patch::default();
        theirs.load_values(values);
        let sorted = |p: &Patch, s: Source| {
            let mut a: Vec<Assignment> = p
                .assignments(s)
                .into_iter()
                .flatten()
                .filter(|a| !a.target.is_fx())
                .collect();
            a.sort_by_key(|a| a.target);
            a
        };
        Source::all().all(|s| {
            let (a, b) = (sorted(self, s), sorted(&theirs, s));
            a.len() == b.len()
                && a.iter()
                    .zip(&b)
                    .all(|(x, y)| x.target == y.target && (x.depth - y.depth).abs() < 1e-4)
        }) && (0..LFOS).all(|l| self.edges(l) == theirs.edges(l))
    }

    /// `other`'s assignments and EDGES in place of these, without allocating (a preset a MIDI
    /// pad chose, prepared off the audio thread and applied on it).
    pub fn copy_from(&self, other: &Patch) {
        // (This patch's effects' pins' assignments kept, the other's not taken: R-FX-PARAMS.)
        for source in Source::all() {
            let i = source.index();
            let theirs: [u64; MOST] = std::array::from_fn(|k| {
                let w = other.slots[i][k].load(Ordering::Acquire);
                if of_word(w).is_some_and(|a| a.target.is_fx()) {
                    0
                } else {
                    w
                }
            });
            let kept = self.keeping_fx(source, &theirs);
            for (a, w) in self.slots[i].iter().zip(kept) {
                a.store(w, Ordering::Release);
            }
        }
        for (a, b) in self.sharp.iter().zip(&other.sharp) {
            a.store(b.load(Ordering::Acquire), Ordering::Release);
        }
        for (a, b) in self.names.iter().zip(&other.names) {
            a.store(b.load(Ordering::Acquire), Ordering::Release);
        }
        self.changed();
    }

    /// Whether a preset's value's ID is one of these (not a parameter's).
    pub fn is_value_id(id: &str) -> bool {
        Source::all().any(|s| {
            id.strip_prefix(&s.id())
                .and_then(|rest| rest.strip_prefix('_'))
                .is_some_and(|rest| {
                    (rest == "edges" && s.lfo().is_some()) || rest.starts_with("to_")
                })
        })
    }
}

/// The sources' assignments as a session keeps them (JSON under the state's `lfo` key), and the
/// macros' names (none in a session saved before them, or with none typed).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Saved {
    pub sources: Vec<SavedSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<String>,
}

/// A source's, by its ID (`lfo1`): each knob it modulates by its parameter's ID and its depth
/// (-1..1), and whether its EDGES are SHARP (an LFO's).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SavedSource {
    pub source: String,
    pub to: Vec<(String, f32)>,
    #[serde(default)]
    pub sharp: bool,
}

/// The state's key the assignments are kept under.
pub const STATE_KEY: &str = "lfo";

/// Saved and loaded with the session: a session's replaces the instance's (one saved before
/// LFOs has none, and is given none by `Ca72::filter_state`).
impl<'a> nih_plug::params::persist::PersistentField<'a, Saved> for Arc<Patch> {
    fn set(&self, new_value: Saved) {
        self.load(&new_value);
    }

    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&Saved) -> R,
    {
        f(&self.saved())
    }
}

// ---- What the editor reads back: each LFO's present value.

/// Each LFO's present state from the audio thread, for its scope's orb, the knobs' ring points
/// and the rail key's glow: published after each block while an editor is open.
#[derive(Debug, Default)]
pub struct Scope {
    value: [AtomicU32; LFOS],
    phase: [AtomicU32; LFOS],
    since: [AtomicU32; LFOS],
    rise: [AtomicU32; LFOS],
    moving: AtomicU8,
    shape: [AtomicU32; LFOS],
    offset: [AtomicU32; LFOS],
    level: [AtomicU32; LFOS],
    moved: AtomicU8,
}

/// An LFO as the audio thread last had it: its output (-1..1, after RISE and EDGES, as it
/// moves its knobs: the latest voice's where MONO is off), where it is in its cycle (0..1,
/// PHASE included: the scope's trace at `phase` is the waveform there), how long since the
/// note that last started it (s; the scope's time base starts there), RISE's gain then (0..1),
/// and whether it moves (assigned, running, not an ENV whose cycle is over); and the settings
/// its trace is drawn from as they are in force (its WAVEFORM's place along the switch's
/// travel, PHASE in cycles, LEVEL 0..1), whether a source moves them (R-LFO's corrections: the
/// owner, 2026-10-20, "the oscilloscope wave isn't changing to match the actual pattern that's
/// happening at the moment").
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    pub value: f32,
    pub phase: f32,
    pub since: f32,
    pub rise: f32,
    pub moving: bool,
    pub shape: f32,
    pub offset: f32,
    pub level: f32,
    pub moved: bool,
}

impl Scope {
    pub fn read(&self, lfo: usize) -> Reading {
        let f =
            |a: &[AtomicU32; LFOS]| f32::from_bits(a[lfo.min(LFOS - 1)].load(Ordering::Relaxed));
        Reading {
            value: f(&self.value),
            phase: f(&self.phase),
            since: f(&self.since),
            rise: f(&self.rise),
            moving: self.moving.load(Ordering::Relaxed) & (1 << lfo.min(7)) != 0,
            shape: f(&self.shape),
            offset: f(&self.offset),
            level: f(&self.level),
            moved: self.moved.load(Ordering::Relaxed) & (1 << lfo.min(7)) != 0,
        }
    }

    /// Whether any LFO moves a knob now (the rail key's faint glow).
    pub fn any_moving(&self) -> bool {
        self.moving.load(Ordering::Relaxed) != 0
    }

    pub fn publish(&self, r: &[Reading; LFOS]) {
        let (mut moving, mut moved) = (0u8, 0u8);
        for (k, r) in r.iter().enumerate() {
            self.value[k].store(r.value.to_bits(), Ordering::Relaxed);
            self.phase[k].store(r.phase.to_bits(), Ordering::Relaxed);
            self.since[k].store(r.since.to_bits(), Ordering::Relaxed);
            self.rise[k].store(r.rise.to_bits(), Ordering::Relaxed);
            self.shape[k].store(r.shape.to_bits(), Ordering::Relaxed);
            self.offset[k].store(r.offset.to_bits(), Ordering::Relaxed);
            self.level[k].store(r.level.to_bits(), Ordering::Relaxed);
            moving |= u8::from(r.moving) << k;
            moved |= u8::from(r.moved) << k;
        }
        self.moving.store(moving, Ordering::Relaxed);
        self.moved.store(moved, Ordering::Relaxed);
    }
}

// ---- Each LFO's colour: the computer's.

/// An LFO's phosphor colour, for its scope's trace, its orb, its rings and its pop-up dial;
/// changed by right-clicking its scope; a setting of the computer's, not the patch's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phosphor {
    Amber,
    Green,
    Blue,
    Red,
    Violet,
    White,
}

/// The LFOs' colours where the computer has none set: amber, green, blue, red (the owner).
pub const COLORS: [Phosphor; LFOS] = [
    Phosphor::Amber,
    Phosphor::Green,
    Phosphor::Blue,
    Phosphor::Red,
];

impl Phosphor {
    pub const ALL: [Phosphor; 6] = [
        Phosphor::Amber,
        Phosphor::Green,
        Phosphor::Blue,
        Phosphor::Red,
        Phosphor::Violet,
        Phosphor::White,
    ];

    /// Its glow, sRGB. The four defaults stay apart under simulated protanopia, deuteranopia
    /// and tritanopia (Machado et al., 2009): no pair nearer than 19 in CIEDE2000 under any of
    /// them (the green bright and leaning to teal, the blue deep, the red a true red;
    /// decisions.md R-LFO-PAGE has the table).
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Phosphor::Amber => [0xff, 0xb0, 0x20],
            Phosphor::Green => [0x26, 0xff, 0xb7],
            Phosphor::Blue => [0x4c, 0x8c, 0xff],
            Phosphor::Red => [0xe6, 0x39, 0x48],
            Phosphor::Violet => [0xc0, 0x80, 0xff],
            Phosphor::White => [0xe4, 0xec, 0xf4],
        }
    }

    /// As the settings' file keeps it.
    pub fn word(self) -> &'static str {
        match self {
            Phosphor::Amber => "amber",
            Phosphor::Green => "green",
            Phosphor::Blue => "blue",
            Phosphor::Red => "red",
            Phosphor::Violet => "violet",
            Phosphor::White => "white",
        }
    }

    pub fn of_word(w: &str) -> Option<Phosphor> {
        Phosphor::ALL.into_iter().find(|p| p.word() == w)
    }

    /// As a menu names it.
    pub fn name(self) -> &'static str {
        match self {
            Phosphor::Amber => "AMBER",
            Phosphor::Green => "GREEN",
            Phosphor::Blue => "BLUE",
            Phosphor::Red => "RED",
            Phosphor::Violet => "VIOLET",
            Phosphor::White => "WHITE",
        }
    }
}

// ---- Each macro's colour: the computer's.

/// A macro's colour, for its name's dots, its jack's collar, its rings, its cord and its pop-up
/// dial; changed by right-clicking its name; a setting of the computer's, not the patch's
/// (decisions.md R-MACROS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroColor {
    Teal,
    Rose,
    Lemon,
    Sage,
    Lavender,
    White,
    Purple,
    Pink,
}

/// The macros' colours where the computer has none set. Round 2's six (purple, pink, lemon, sage,
/// lavender, white) moved apart from the LFOs' final four under simulated colour blindness:
/// round 2's purple was all but the LFOs' blue for a deuteranope (CIEDE2000 1.5), so it is a teal
/// and the pink a rose (decisions.md R-MACROS has the check). Round 2's purple and pink stay in
/// the palette.
pub const MACRO_COLORS: [MacroColor; MACROS] = [
    MacroColor::Teal,
    MacroColor::Rose,
    MacroColor::Lemon,
    MacroColor::Sage,
    MacroColor::Lavender,
    MacroColor::White,
];

impl MacroColor {
    pub const ALL: [MacroColor; 8] = [
        MacroColor::Teal,
        MacroColor::Rose,
        MacroColor::Lemon,
        MacroColor::Sage,
        MacroColor::Lavender,
        MacroColor::White,
        MacroColor::Purple,
        MacroColor::Pink,
    ];

    /// Its glow, sRGB (`tools/colours.py`'s search, lit on the black face).
    pub fn rgb(self) -> [u8; 3] {
        match self {
            MacroColor::Teal => [0x14, 0xbb, 0xc2],
            MacroColor::Rose => [0xb8, 0x7e, 0x99],
            MacroColor::Lemon => [0xf9, 0xfb, 0x70],
            MacroColor::Sage => [0x8a, 0xb1, 0x81],
            MacroColor::Lavender => [0xc9, 0xc2, 0xfc],
            MacroColor::White => [0xdc, 0xf8, 0xfa],
            MacroColor::Purple => [0xbf, 0x58, 0xfd],
            MacroColor::Pink => [0xff, 0x8c, 0xc8],
        }
    }

    /// As the settings' file keeps it.
    pub fn word(self) -> &'static str {
        match self {
            MacroColor::Teal => "teal",
            MacroColor::Rose => "rose",
            MacroColor::Lemon => "lemon",
            MacroColor::Sage => "sage",
            MacroColor::Lavender => "lavender",
            MacroColor::White => "white",
            MacroColor::Purple => "purple",
            MacroColor::Pink => "pink",
        }
    }

    pub fn of_word(w: &str) -> Option<MacroColor> {
        MacroColor::ALL.into_iter().find(|p| p.word() == w)
    }

    /// As a menu names it.
    pub fn name(self) -> &'static str {
        match self {
            MacroColor::Teal => "TEAL",
            MacroColor::Rose => "ROSE",
            MacroColor::Lemon => "LEMON",
            MacroColor::Sage => "SAGE",
            MacroColor::Lavender => "LAVENDER",
            MacroColor::White => "WHITE",
            MacroColor::Purple => "PURPLE",
            MacroColor::Pink => "PINK",
        }
    }
}

// ---- The macros' parameters.

/// The macros' parameters' IDs (host names "MACRO 1" to "MACRO 6", fixed: the owner's 5c).
pub const MACRO_IDS: [&str; MACROS] = ["macro1", "macro2", "macro3", "macro4", "macro5", "macro6"];
/// Their labels, as the host, the agents and MIDI Learn name them.
pub const MACRO_LABELS: [&str; MACROS] = [
    "MACRO 1", "MACRO 2", "MACRO 3", "MACRO 4", "MACRO 5", "MACRO 6",
];

/// Macro `n`'s (1 to 6) parameter: 0 to 100 %, 0 in a new patch.
pub fn macro_param(n: usize) -> FloatParam {
    FloatParam::new(
        format!("MACRO {n}"),
        0.0,
        FloatRange::Linear {
            min: 0.0,
            max: 100.0,
        },
    )
    .with_unit(" %")
    .with_value_to_string(formatters::v2s_f32_rounded(0))
}

/// The macro of parameter `id`, if it is one (0 to 5).
pub fn macro_of_id(id: &str) -> Option<usize> {
    MACRO_IDS.iter().position(|m| *m == id)
}

// ---- The generators.

/// MODE.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    /// Runs on, on its own clock (or, timed, the host's).
    #[default]
    Free,
    /// Starts again at PHASE at each new note.
    Retrig,
    /// Starts at PHASE at each new note, runs one cycle and holds.
    Env,
}

/// The shark tooth legend's top, a share of its cycle (its picture's, 14 of 22 units).
pub const SHARK_PEAK: f64 = 14.0 / 22.0;

/// The rectangle's duty cycle along CONTINUOUS's travel past SQUARE (`at`, the switch's
/// steps): exactly a half at SQUARE (an LFO's square, agent decision: the circuit's 48.4 % is
/// its comparator's), the circuit's 30 % and 17.9 % at WIDE and NARROW, straight in the width
/// bias between them and on past NARROW to the floor (about 6.4 %).
pub fn duty(at: f64) -> f64 {
    let b = shape_bias(at);
    if b >= -1.5 {
        0.5 + (b / -1.5) * (0.30 - 0.5)
    } else {
        0.30 + (-1.5 - b) * (0.179 - 0.30)
    }
}

/// The switch's place `k` (0 TRIANGLE, 1 the shark tooth, 2 SAWTOOTH, 3 SQUARE) at `p`
/// (0..1 of its cycle), as its legend draws it, -1 to 1.
fn place(k: usize, p: f64) -> f64 {
    match k {
        0 => {
            if p < 0.5 {
                -1.0 + 4.0 * p
            } else {
                3.0 - 4.0 * p
            }
        }
        1 => {
            if p < SHARK_PEAK {
                -1.0 + 2.0 * p / SHARK_PEAK
            } else {
                -(p - SHARK_PEAK) / (1.0 - SHARK_PEAK)
            }
        }
        2 => 2.0 * p - 1.0,
        _ => {
            if p < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

/// The waveform at `at` along the switch's travel (0 to [`SHAPE_PLACES`]: a place of the
/// six exactly, or between two blended as R-CONTINUOUS-WAVE's oscillators are, past SQUARE the
/// rectangle at its [`duty`]) at `phase` (cycles; its fraction is where in the cycle): -1 to 1.
pub fn wave(at: f64, phase: f64) -> f64 {
    shape(at, phase - phase.floor())
}

/// The waveform as an LFO reads it at `phase` (cycles): forwards, or with REVERSE backwards,
/// its place in its cycle `p` read at 1 − p (decisions.md R-LFO's corrections), so that a
/// sawtooth falls (+1 at its cycle's start).
pub fn read(at: f64, phase: f64, reverse: bool) -> f64 {
    let p = phase - phase.floor();
    shape(at, if reverse { 1.0 - p } else { p })
}

/// The waveform at `at` at `p` (0..1 of its cycle; 1 its end, as REVERSE reads 0).
fn shape(at: f64, p: f64) -> f64 {
    let at = at.clamp(0.0, SHAPE_PLACES);
    if at >= 3.0 {
        return if p < duty(at) { 1.0 } else { -1.0 };
    }
    let k = at as usize;
    let f = at - k as f64;
    if f == 0.0 {
        place(k, p)
    } else {
        (1.0 - f) * place(k, p) + f * place(k + 1, p)
    }
}

/// An LFO's settings, as the engine takes them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Where its waveform is along the switch's travel ([`wave`]'s `at`).
    pub at: f64,
    /// RATE, Hz.
    pub hz: f64,
    /// Timed: its cycle's length on the clock.
    pub timed: Option<Length>,
    /// PHASE, cycles (0..1).
    pub phase: f64,
    /// RISE, s.
    pub rise: f64,
    pub mode: Mode,
    pub mono: bool,
    pub edges: Edges,
    /// POLARITY UNI: its output 0..1 rather than -1..1, so that its knobs move one way.
    pub uni: bool,
    /// REVERSE: its waveform read backwards.
    pub reverse: bool,
    /// ON (off: muted, faded to nothing and back over [`MUTE_FADE`]).
    pub on: bool,
    /// LEVEL (0..1): the share of every one of its assignments it gives (R-LFO's addendum).
    pub level: f64,
    /// WAVEFORM CONTINUOUS (its shape can be moved: [`Target::LfoShape`]).
    pub continuous: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            at: 0.0,
            hz: f64::from(DEFAULT_RATE),
            timed: None,
            phase: 0.0,
            rise: 0.0,
            mode: Mode::Free,
            mono: true,
            edges: Edges::Soft,
            uni: false,
            reverse: false,
            on: true,
            level: 1.0,
            continuous: false,
        }
    }
}

impl Settings {
    /// Its cycles a second at `tempo` (BPM, 4/4 for bars): how far the scope's ENV orb is through
    /// its one cycle.
    pub fn hz_at(&self, tempo: f64) -> f64 {
        match self.timed {
            None => self.hz,
            Some(Length::Beats(q)) => tempo / 60.0 / q,
            Some(Length::Bars(n)) => tempo / 60.0 / (4.0 * n),
        }
    }

    /// Its waveform `x` of the way (0..1) through one cycle from PHASE, before RISE and EDGES:
    /// the scope's trace, one cycle across its width whatever the rate (timed, one note value).
    /// At 1 it is the cycle's last value, just short of the next one's start, as ENV holds.
    pub fn cycle_at(&self, x: f64) -> f64 {
        self.shaped(self.phase + x.clamp(0.0, 1.0) * ENV_END)
    }

    /// Its waveform at `phase` (cycles, PHASE included), REVERSE and POLARITY as set: -1..1,
    /// or 0..1 UNI.
    pub fn shaped(&self, phase: f64) -> f64 {
        let x = read(self.at, phase, self.reverse);
        if self.uni { 0.5 * (x + 1.0) } else { x }
    }

    fn rise_gain(&self, since: f64) -> f64 {
        if self.rise > 0.0 {
            (since / self.rise).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

/// Where ENV's one cycle holds: its last value, just short of the next cycle's start.
const ENV_END: f64 = 1.0 - 1e-9;

/// How long ON's fade takes (s): an LFO turned off fades to nothing, and back on from nothing,
/// in a straight line over 10 ms, so that its knobs do not jump (the owner, 2026-10-20: "we
/// need to be able to mute the LFO").
pub const MUTE_FADE: f64 = 0.010;

/// One generator: its own cycles since it last started (RETRIG and ENV) or running on (FREE),
/// the time since the note that last started it, whether a note has, and its output.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gen {
    cycles: f64,
    since: f64,
    started: bool,
    out: f64,
    /// Where it was in its cycle at its last value (PHASE included), for the scope.
    at: f64,
    /// How far ON's fade has it muted: 0 sounding (its output as it is, to the bit), 1 off.
    muted: f64,
    /// Its output after ON's fade.
    heard: f64,
}

impl Gen {
    /// A new note starts it: RETRIG and ENV from PHASE, RISE from nothing.
    pub fn start(&mut self, s: &Settings) {
        if s.mode != Mode::Free {
            self.cycles = 0.0;
        }
        self.since = 0.0;
        self.started = true;
    }

    /// Its value now, on the clock `clock` (cycles, if timed and FREE: its place on the host's
    /// clock), moved on by `dt` s of EDGES' slew; -1..1.
    pub fn value(&mut self, s: &Settings, clock: Option<f64>, dt: f64) -> f64 {
        let c = match (s.mode, clock) {
            (Mode::Free, Some(c)) => c,
            (Mode::Env, _) if !self.started => ENV_END,
            (Mode::Env, _) => self.cycles.min(ENV_END),
            _ => self.cycles,
        };
        self.at = (s.phase + c).rem_euclid(1.0);
        let gain = if s.rise > 0.0 && !self.started {
            0.0
        } else {
            s.rise_gain(self.since)
        };
        let x = s.shaped(self.at) * gain;
        self.out = match s.edges {
            Edges::Sharp => x,
            Edges::Soft => {
                let tau = SOFT_EDGE / 9f64.ln();
                self.out + (x - self.out) * (1.0 - (-dt / tau).exp())
            }
        };
        // (ON's fade: straight to 1 off, to 0 on; while on and faded in, its output untouched.)
        let step = dt / MUTE_FADE;
        self.muted = if s.on {
            (self.muted - step).max(0.0)
        } else {
            (self.muted + step).min(1.0)
        };
        self.heard = if self.muted == 0.0 {
            self.out
        } else {
            self.out * (1.0 - self.muted)
        };
        self.heard
    }

    /// Moved on by `n` samples of `per` cycles each and `dt` s each.
    pub fn advance(&mut self, s: &Settings, per: f64, n: usize, dt: f64) {
        let c = self.cycles + per * n as f64;
        self.cycles = if s.mode == Mode::Env {
            c.min(1.0)
        } else {
            c - c.floor()
        };
        self.since += dt * n as f64;
    }

    /// Whether it moves: not an ENV whose cycle is over (nor one never started), nor off and
    /// faded out.
    pub fn moving(&self, s: &Settings) -> bool {
        (s.on || self.muted < 1.0) && (s.mode != Mode::Env || (self.started && self.cycles < 1.0))
    }

    /// The scope's reading of it.
    pub fn reading(&self, s: &Settings, moving: bool) -> Reading {
        Reading {
            value: self.heard as f32,
            phase: self.at as f32,
            since: self.since as f32,
            rise: if s.rise > 0.0 && !self.started {
                0.0
            } else {
                s.rise_gain(self.since) as f32
            },
            moving,
            shape: s.at as f32,
            offset: s.phase as f32,
            level: s.level as f32,
            moved: false,
        }
    }
}

/// The modulation as the engine is given it: each LFO's settings, every source's assignments,
/// each assigned knob's travel after its glide, and the computer's FREQUENCY law (the knobs'
/// travel through it). Part of the engine's `Controls`; none assigned, nothing more runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lfos {
    pub lfo: [Settings; LFOS],
    /// Each source's assignments, [`Source::index`]'s order.
    pub to: [[Option<Assignment>; MOST]; SOURCES],
    /// Each target's travel (0..1), [`Target::ALL`]'s order: the assigned ones'; 0 the rest.
    pub base: [f32; TARGETS],
    pub law: FreqLaw,
    /// Each macro's value (0..1), where its knob's glide has it.
    pub macros: [f32; MACROS],
}

impl Default for Lfos {
    fn default() -> Self {
        Lfos {
            lfo: Default::default(),
            to: Default::default(),
            base: [0.0; TARGETS],
            law: FreqLaw::default(),
            macros: [0.0; MACROS],
        }
    }
}

/// The computer's FREQUENCY law, as a value the controls can compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FreqLaw {
    #[default]
    Semitones,
    Hardware,
}

impl From<FreqLaw> for Law {
    fn from(l: FreqLaw) -> Law {
        match l {
            FreqLaw::Semitones => Law::Semitones,
            FreqLaw::Hardware => Law::Hardware,
        }
    }
}

impl From<Law> for FreqLaw {
    fn from(l: Law) -> FreqLaw {
        match l {
            Law::Semitones => FreqLaw::Semitones,
            Law::Hardware => FreqLaw::Hardware,
        }
    }
}

impl Lfos {
    /// The sources' values with the LFOs' at `lfos`, the macros' their knobs' (0..1, unipolar).
    pub fn source_values(&self, lfos: &[f64; LFOS]) -> [f64; SOURCES] {
        std::array::from_fn(|i| match Source::of_index(i) {
            Some(Source::Lfo(n)) => lfos[n],
            Some(Source::Macro(n)) => f64::from(self.macros[n]),
            None => 0.0,
        })
    }

    /// Whether any source modulates anything (an assignment of a depth other than 0).
    pub fn active(&self) -> bool {
        self.to.iter().flatten().flatten().any(|a| a.depth != 0.0)
    }

    /// The effects' pins some source moves (a bit each, [`Target::Fx`]'s number): R-FX-PARAMS.
    pub fn fx_moved(&self) -> u32 {
        let mut bits = 0;
        for a in self.to.iter().flatten().flatten() {
            if let (Some(k), true) = (a.target.fx(), a.depth != 0.0) {
                bits |= 1 << k;
            }
        }
        bits
    }

    /// Each effects' pin's offset on its travel with the sources at `shared` (the sum of depth
    /// × value; the shared copies: an effect has no voices).
    pub fn fx_offsets(&self, shared: &[f64; SOURCES]) -> [f32; FX_PINS] {
        let mut out = [0.0f64; FX_PINS];
        for (i, to) in self.to.iter().enumerate() {
            for a in to.iter().flatten() {
                if let Some(k) = a.target.fx() {
                    out[k] += f64::from(a.depth) * shared[i];
                }
            }
        }
        out.map(|v| v as f32)
    }

    /// Whether source `i` ([`Source::index`]) has a value of each voice's own: an LFO with MONO
    /// off. The macros never.
    pub fn own(&self, i: usize) -> bool {
        matches!(Source::of_index(i), Some(Source::Lfo(n)) if !self.lfo[n].mono)
    }

    /// Whether any LFO is assigned (a depth other than 0): its generator runs.
    pub fn lfo_active(&self, lfo: usize) -> bool {
        self.to[lfo].iter().flatten().any(|a| a.depth != 0.0)
    }

    /// Whether any source moves an LFO's own knob (R-LFO's addendum) or TEMPO: the generators
    /// then run on [`TICK`]'s clock ([`Bank`]'s ticks).
    pub fn cross(&self) -> bool {
        self.to
            .iter()
            .flatten()
            .flatten()
            .any(|a| a.depth != 0.0 && a.target.ticks())
    }

    /// Whether a source moves LFO `k`'s knobs its trace is drawn from: PHASE, its CONTINUOUS
    /// shape, LEVEL.
    pub fn trace_moved(&self, k: usize) -> bool {
        self.to.iter().flatten().flatten().any(|a| {
            a.depth != 0.0
                && matches!(a.target.lfo_knob(), Some((n, LfoKnob::Phase | LfoKnob::Shape | LfoKnob::Level)) if n == k)
        })
    }

    /// Whether any source moves TEMPO.
    pub fn tempo_moved(&self) -> bool {
        self.to
            .iter()
            .flatten()
            .flatten()
            .any(|a| a.depth != 0.0 && a.target == Target::Tempo)
    }

    /// Whether a source with values of each voice's own modulates a knob each voice has.
    pub fn per_voice(&self) -> bool {
        self.to.iter().enumerate().any(|(i, to)| {
            self.own(i)
                && to
                    .iter()
                    .flatten()
                    .any(|a| a.depth != 0.0 && a.target.per_voice())
        })
    }

    /// The controls `c` with every assigned knob moved by the sources' shared values `shared`.
    pub fn modulated(&self, c: &Controls, shared: &[f64; SOURCES]) -> Controls {
        let mut m = *c;
        // (An LFO's own knobs are the generators', not the controls'.)
        let (sums, touched) = self.sums(|_, t| !t.is_lfo() && !t.is_fx(), |i| shared[i]);
        for t in Target::ALL {
            if touched & (1 << t.index()) != 0 {
                let travel = f64::from(self.base[t.index()]) + sums[t.index()];
                t.write(&mut m, travel, self.law.into());
            }
        }
        m
    }

    /// Voice's panel `p` and FEEDBACK `feedback` with its own values (`own`, of the sources
    /// [`Lfos::own`] says are) and the shared ones (`shared`) on the knobs it has of its own
    /// that a source of its own moves; the rest as `p` has them.
    pub fn voice(
        &self,
        p: &mut Panel,
        feedback: &mut f64,
        shared: &[f64; SOURCES],
        own: &[f64; SOURCES],
    ) {
        // (The knobs a source of the voice's own moves: only they differ from the shared panel.)
        let (_, own_touched) = self.sums(|i, t| self.own(i) && t.per_voice(), |_| 0.0);
        if own_touched == 0 {
            return;
        }
        let (sums, _) = self.sums(
            |_, t| own_touched & (1u128 << t.index()) != 0,
            |i| if self.own(i) { own[i] } else { shared[i] },
        );
        let law = self.law.into();
        for t in Target::ALL {
            if own_touched & (1u128 << t.index()) == 0 {
                continue;
            }
            let travel = (f64::from(self.base[t.index()]) + sums[t.index()]).clamp(0.0, 1.0);
            if t == Target::Feedback {
                *feedback = ca72::voice::feedback_law(travel);
            } else {
                t.write_panel(p, travel, law);
            }
        }
    }

    /// The sum of depth × value for each target `which` takes (by the source's index and the
    /// target), and which those are (a bit a target).
    fn sums(
        &self,
        which: impl Fn(usize, Target) -> bool,
        value: impl Fn(usize) -> f64,
    ) -> ([f64; TARGETS], u128) {
        let mut sums = [0.0f64; TARGETS];
        let mut touched = 0u128;
        for (i, to) in self.to.iter().enumerate() {
            for a in to.iter().flatten() {
                if a.depth == 0.0 || !which(i, a.target) {
                    continue;
                }
                sums[a.target.index()] += f64::from(a.depth) * value(i);
                touched |= 1 << a.target.index();
            }
        }
        (sums, touched)
    }
}

/// The LFOs' generators in the engine: each one's shared copy, and each voice's own (for the
/// LFOs with MONO off), with their values at the latest step; and, while a source moves an
/// LFO's own knob, the ticks they then run on ([`Cross`]).
#[derive(Clone, Debug, Default)]
pub struct Bank {
    pub shared: [Gen; LFOS],
    pub voices: [[Gen; LFOS]; VOICES],
    pub shared_values: [f64; LFOS],
    pub voice_values: [[f64; LFOS]; VOICES],
    /// The voice the scope shows where MONO is off (the latest started).
    pub latest: usize,
    pub cross: Cross,
}

impl Bank {
    /// Every key starts RETRIG's phase and RISE. `full` is the main contours' trigger:
    /// from silence or RETRIGGER on, FREE's RISE and ENV start too (R-LFO-NOTE-RETRIG).
    pub fn start_shared(&mut self, l: &Lfos, full: bool) {
        for (g, s) in self.shared.iter_mut().zip(&l.lfo) {
            if full || s.mode == Mode::Retrig {
                g.start(s);
            }
        }
        if self.cross.on {
            for (j, s) in l.lfo.iter().enumerate() {
                if s.mode != Mode::Free && (full || s.mode == Mode::Retrig) {
                    self.cross.shared[j] = *s;
                    self.cross.shared_out[j] = self.shared[j].fresh(s);
                }
            }
        }
    }

    /// Every key starts voice `k`'s RETRIG sources; `full` starts its other sources too.
    /// Newly assigned polyphonic voices always receive a full trigger.
    pub fn start_voice(&mut self, l: &Lfos, k: usize, full: bool) {
        if let Some(v) = self.voices.get_mut(k) {
            for (g, s) in v.iter_mut().zip(&l.lfo) {
                if full || s.mode == Mode::Retrig {
                    g.start(s);
                }
            }
            self.latest = k;
            if self.cross.on {
                for (j, s) in l.lfo.iter().enumerate() {
                    if s.mode != Mode::Free && (full || s.mode == Mode::Retrig) {
                        self.cross.voices[k][j] = *s;
                        self.cross.voice_out[k][j] = v[j].fresh(s);
                    }
                }
            }
        }
    }

    /// Every LFO's value now (`clock`, each timed FREE one's place on the host's clock, in
    /// cycles), its slew moved on by `dt` s; voices' own only for the LFOs with MONO off. Each
    /// is its output at its LEVEL.
    pub fn step(&mut self, l: &Lfos, clock: &[Option<f64>; LFOS], dt: f64, voices: usize) {
        let n = voices.min(VOICES);
        for (k, (s, &c)) in l.lfo.iter().zip(clock).enumerate() {
            // (At LEVEL 100 %, × 1: the output to the bit.)
            self.shared_values[k] = self.shared[k].value(s, c, dt) * s.level;
            if !s.mono {
                for (values, gens) in self.voice_values[..n].iter_mut().zip(&mut self.voices[..n]) {
                    values[k] = gens[k].value(s, c, dt) * s.level;
                }
            }
        }
    }

    /// Moved on by `n` samples at `rate` Hz, each LFO at `per[k]` cycles a sample.
    pub fn advance(&mut self, l: &Lfos, per: &[f64; LFOS], n: usize, rate: f64, voices: usize) {
        let dt = 1.0 / rate;
        let m = voices.min(VOICES);
        for (k, (s, &p)) in l.lfo.iter().zip(per).enumerate() {
            self.shared[k].advance(s, p, n, dt);
            if !s.mono {
                for gens in &mut self.voices[..m] {
                    gens[k].advance(s, p, n, dt);
                }
            }
        }
    }

    /// Each LFO's cycles a sample at `rate` Hz, and each timed FREE one's place on the clock,
    /// at `at` samples after `position`.
    pub fn clock(
        l: &Lfos,
        position: &Position,
        at: f64,
        rate: f64,
    ) -> ([f64; LFOS], [Option<f64>; LFOS]) {
        let mut per = [0.0; LFOS];
        let mut clock = [None; LFOS];
        for (k, s) in l.lfo.iter().enumerate() {
            match s.timed {
                Some(length) => {
                    let (phase, p) = position.cycle(length);
                    per[k] = p;
                    if s.mode == Mode::Free {
                        clock[k] = Some(phase + at * p);
                    }
                }
                None => per[k] = s.hz / rate,
            }
        }
        (per, clock)
    }

    /// The scope's readings: the shared copies', or the latest voice's where MONO is off;
    /// moving where assigned and running; at its LEVEL (its RISE's gain too), with the settings
    /// in force (on the ticks, as the sources have moved them).
    pub fn readings(&self, l: &Lfos) -> [Reading; LFOS] {
        std::array::from_fn(|k| {
            let latest = self.latest.min(VOICES - 1);
            let s = match (self.cross.on, l.lfo[k].mono) {
                (false, _) => &l.lfo[k],
                (true, true) => &self.cross.shared[k],
                (true, false) => &self.cross.voices[latest][k],
            };
            let g = if l.lfo[k].mono {
                &self.shared[k]
            } else {
                &self.voices[latest][k]
            };
            let assigned = l.lfo_active(k);
            let mut r = g.reading(s, assigned && g.moving(s));
            r.value *= s.level as f32;
            r.rise *= s.level as f32;
            // (What its trace is drawn from, as in force; moved where a source moves them.)
            (r.shape, r.offset, r.level) = (s.at as f32, s.phase as f32, s.level as f32);
            r.moved = self.cross.on && l.trace_moved(k);
            r
        })
    }
}

// ---- The LFOs moving each other (decisions.md R-LFO's addendum).

/// How often the LFOs' own knobs are moved by the sources, s: a tick fixed in time, not in
/// samples or blocks, so that a patch whose LFOs move each other (in a loop too: the owner,
/// 2026-10-20, "Let's try B. It's unconventional, but it's interesting") runs the same at
/// every sample rate and block size, live and offline. Half a millisecond: 40 ticks a cycle of
/// the fastest LFO (50 Hz), so that an LFO moving another keeps its shape; a loop's delay, one
/// tick, short beside the slowest cycle it can shape (20 ms); about the audio's own step (32
/// samples, 0.33 to 0.73 ms); and cheap (2000 a second, a few hundred operations each).
pub const TICK: f64 = 1.0 / TICKS_A_SECOND;
/// [`TICK`]s a second.
pub const TICKS_A_SECOND: f64 = 2000.0;

/// The latest tick by host sample `played` at `rate` Hz: the greatest whose time (its number
/// over [`TICKS_A_SECOND`]) is not past `played / rate`. Exact: `played × 2000` is an f64
/// exactly, the quotient is rounded once, and a quotient that is not whole is at least
/// `1 / rate` from one, far more than its rounding; so the same at every sample rate for the
/// same time.
pub fn tick_at(played: u64, rate: f64) -> u64 {
    (played as f64 * TICKS_A_SECOND / rate).floor() as u64
}

/// One source moving one LFO's knob: the source's index, the LFO, its knob, the depth.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Route {
    from: usize,
    lfo: usize,
    knob: LfoKnob,
    depth: f64,
}

/// Every source's assignments on the LFOs' own knobs (of a depth other than 0), gathered so
/// that a tick reads only them.
#[derive(Clone, Copy, Debug)]
struct Routes {
    n: usize,
    all: [Route; SOURCES * MOST],
    /// The sources moving TEMPO (each one's index and depth), and how many.
    tempo_n: usize,
    tempo: [(usize, f64); SOURCES],
}

impl Default for Routes {
    fn default() -> Self {
        Routes {
            n: 0,
            all: [Route {
                from: 0,
                lfo: 0,
                knob: LfoKnob::Rate,
                depth: 0.0,
            }; SOURCES * MOST],
            tempo_n: 0,
            tempo: [(0, 0.0); SOURCES],
        }
    }
}

impl Routes {
    fn of(l: &Lfos) -> Routes {
        let mut r = Routes::default();
        for (from, to) in l.to.iter().enumerate() {
            for a in to.iter().flatten() {
                if let Some((lfo, knob)) = a.target.lfo_knob()
                    && a.depth != 0.0
                    && lfo < LFOS
                    && r.n < r.all.len()
                {
                    r.all[r.n] = Route {
                        from,
                        lfo,
                        knob,
                        depth: f64::from(a.depth),
                    };
                    r.n += 1;
                }
                if a.target == Target::Tempo && a.depth != 0.0 && r.tempo_n < r.tempo.len() {
                    r.tempo[r.tempo_n] = (from, f64::from(a.depth));
                    r.tempo_n += 1;
                }
            }
        }
        r
    }

    fn iter(&self) -> impl Iterator<Item = &Route> {
        self.all[..self.n].iter()
    }

    /// TEMPO with each source's value `value(i)` (its index): its travel plus depth × value,
    /// clamped, in BPM; none where nothing moves it.
    fn tempo(&self, l: &Lfos, value: impl Fn(usize) -> f64) -> Option<f64> {
        if self.tempo_n == 0 {
            return None;
        }
        let sum: f64 = self.tempo[..self.tempo_n]
            .iter()
            .map(|&(i, d)| d * value(i))
            .sum();
        Some(tempo_at(f64::from(l.base[Target::Tempo.index()]) + sum))
    }
}

/// Each LFO's clock over a block: a timed one's cycles a second at the tempo and, FREE, its
/// place on the host's clock at the block's start (R-OSC-TIMED's, as [`Bank::clock`]'s), on
/// the ticks' time (s from the engine's first sample).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clocks {
    hz: [Option<f64>; LFOS],
    from: [Option<f64>; LFOS],
    start: f64,
}

impl Clocks {
    /// For a block from `position`, which began `start` s on the ticks' time, at `rate` Hz.
    pub fn of(l: &Lfos, position: &Position, start: f64, rate: f64) -> Clocks {
        let mut c = Clocks {
            start,
            ..Clocks::default()
        };
        for (k, s) in l.lfo.iter().enumerate() {
            if let Some(length) = s.timed {
                let (phase, per) = position.cycle(length);
                c.hz[k] = Some(per * rate);
                if s.mode == Mode::Free {
                    c.from[k] = Some(phase);
                }
            }
        }
        c
    }

    /// LFO `k`'s cycles a second with `s` in force: the tempo's timed, else its RATE.
    fn hz(&self, k: usize, s: &Settings) -> f64 {
        self.hz[k].unwrap_or(s.hz)
    }

    /// LFO `k`'s place on the host's clock at `t` s, if it is timed and FREE.
    fn at(&self, k: usize, t: f64) -> Option<f64> {
        let hz = self.hz[k]?;
        self.from[k].map(|c| c + (t - self.start) * hz)
    }
}

/// The LFOs on their ticks: whether they run on them (a source moves an LFO's own knob), the
/// next tick's number, each copy's settings in force since its latest tick (its knobs where
/// the sources had them then), and each copy's value at its latest tick as the LFOs read it at
/// the next: its waveform, RISE, POLARITY, ON's fade and LEVEL, not EDGES' slew (which smooths
/// what reaches the sound), so that a note that starts every LFO of a loop again starts the
/// loop again exactly.
#[derive(Clone, Debug, Default)]
pub struct Cross {
    pub on: bool,
    next: u64,
    shared: [Settings; LFOS],
    voices: [[Settings; LFOS]; VOICES],
    shared_out: [f64; LFOS],
    voice_out: [[f64; LFOS]; VOICES],
    routes: Routes,
    /// TEMPO in force since the latest tick (BPM), moved by the sources' shared values at the
    /// tick before (a shared target: option C), if a source moves it.
    pub tempo: Option<f64>,
}

impl Cross {
    /// The latest tick's number.
    pub fn latest(&self) -> u64 {
        self.next.saturating_sub(1)
    }

    /// LFO `k`'s value at the latest tick, its shared copy's or voice `v`'s own (MONO off), as
    /// the LFOs read it (the tests' trajectory).
    pub fn value(&self, k: usize, v: Option<usize>) -> f64 {
        match v {
            Some(v) => self.voice_out[v.min(VOICES - 1)][k.min(LFOS - 1)],
            None => self.shared_out[k.min(LFOS - 1)],
        }
    }

    /// LFO `k`'s settings in force since the latest tick (its shared copy's).
    pub fn settings(&self, k: usize) -> Settings {
        self.shared[k.min(LFOS - 1)]
    }

    /// Voice `v`'s own copy's of LFO `k` (MONO off).
    pub fn voice_settings(&self, v: usize, k: usize) -> Settings {
        self.voices[v.min(VOICES - 1)][k.min(LFOS - 1)]
    }
}

/// LFO `j`'s settings with its knobs moved by the routes to them, each source's value
/// `value(i)` (its index): its knob's travel plus depth × value, clamped, through its
/// parameter's own range. RATE not while timed, the shape only while CONTINUOUS.
fn moved(l: &Lfos, routes: &Routes, j: usize, value: impl Fn(usize) -> f64) -> Settings {
    let mut s = l.lfo[j];
    let mut sums = [0.0f64; LFO_KNOBS];
    let mut touched = [false; LFO_KNOBS];
    for r in routes.iter().filter(|r| r.lfo == j) {
        sums[r.knob as usize] += r.depth * value(r.from);
        touched[r.knob as usize] = true;
    }
    for k in LfoKnob::ALL {
        if !touched[k as usize] {
            continue;
        }
        let base = f64::from(l.base[Target::of_lfo(j, k).index()]);
        let t = (base + sums[k as usize]).clamp(0.0, 1.0) as f32;
        match k {
            LfoKnob::Rate => {
                if s.timed.is_none() {
                    s.hz = f64::from(RATE_RANGE.unnormalize(t))
                        .clamp(f64::from(RATE_HZ.0), f64::from(RATE_HZ.1));
                }
            }
            LfoKnob::Phase => {
                s.phase = (f64::from(PHASE_RANGE.unnormalize(t)) / 360.0).rem_euclid(1.0);
            }
            LfoKnob::Rise => {
                s.rise = f64::from(RISE_RANGE.unnormalize(t)).clamp(0.0, f64::from(RISE_MOST));
            }
            LfoKnob::Shape => {
                if s.continuous {
                    s.at = shape_snapped(f64::from(t) * SHAPE_PLACES);
                }
            }
            LfoKnob::Level => s.level = f64::from(t),
        }
    }
    s
}

impl Gen {
    /// Its value at its start (a note's), with `s` its settings: as the LFOs read it.
    fn fresh(&self, s: &Settings) -> f64 {
        let c = if s.mode == Mode::Free {
            self.cycles
        } else {
            0.0
        };
        let gain = if s.rise > 0.0 { 0.0 } else { 1.0 };
        let x = s.shaped((s.phase + c).rem_euclid(1.0)) * gain;
        let x = if self.muted == 0.0 {
            x
        } else {
            x * (1.0 - self.muted)
        };
        x * s.level
    }

    /// Moved on by a tick at `hz` cycles a second with `s` in force: its cycle (an ENV's to
    /// its end), the time since its note, ON's fade.
    fn tick_on(&mut self, s: &Settings, hz: f64) {
        let c = self.cycles + hz * TICK;
        self.cycles = if s.mode == Mode::Env {
            c.min(1.0)
        } else {
            c - c.floor()
        };
        self.since += TICK;
        let step = TICK / MUTE_FADE;
        self.muted = if s.on {
            (self.muted - step).max(0.0)
        } else {
            (self.muted + step).min(1.0)
        };
    }

    /// Its value at its tick with `s` in force (`clock`, a timed FREE one's place on the
    /// host's clock): as the LFOs read it ([`Cross`]).
    fn tick_value(&self, s: &Settings, clock: Option<f64>) -> f64 {
        let c = self.place(s, clock, 0.0, 0.0);
        let gain = if s.rise > 0.0 && !self.started {
            0.0
        } else {
            s.rise_gain(self.since)
        };
        let x = s.shaped((s.phase + c).rem_euclid(1.0)) * gain;
        let x = if self.muted == 0.0 {
            x
        } else {
            x * (1.0 - self.muted)
        };
        x * s.level
    }

    /// Its cycle `ahead` s past its latest tick, at `hz` (`clock`: a timed FREE one's place).
    fn place(&self, s: &Settings, clock: Option<f64>, hz: f64, ahead: f64) -> f64 {
        match (s.mode, clock) {
            (Mode::Free, Some(c)) => c,
            (Mode::Env, _) if !self.started => ENV_END,
            (Mode::Env, _) => (self.cycles + hz * ahead).min(ENV_END),
            _ => self.cycles + hz * ahead,
        }
    }

    /// Its value for the sound `ahead` s past its latest tick, with `s` in force since at `hz`
    /// cycles a second (`clock`: a timed FREE one's place): its waveform there, RISE, EDGES'
    /// slew moved on by `dt` s, ON's fade, at its LEVEL.
    fn peek(&mut self, s: &Settings, clock: Option<f64>, hz: f64, ahead: f64, dt: f64) -> f64 {
        let c = self.place(s, clock, hz, ahead);
        self.at = (s.phase + c).rem_euclid(1.0);
        let gain = if s.rise > 0.0 && !self.started {
            0.0
        } else {
            s.rise_gain(self.since + ahead)
        };
        let x = s.shaped(self.at) * gain;
        self.out = match s.edges {
            Edges::Sharp => x,
            Edges::Soft => {
                let tau = SOFT_EDGE / 9f64.ln();
                self.out + (x - self.out) * (1.0 - (-dt / tau).exp())
            }
        };
        let step = ahead / MUTE_FADE;
        let muted = if s.on {
            (self.muted - step).max(0.0)
        } else {
            (self.muted + step).min(1.0)
        };
        self.heard = if muted == 0.0 {
            self.out
        } else {
            self.out * (1.0 - muted)
        };
        self.heard * s.level
    }
}

impl Bank {
    /// The ticks due by host sample `played` (counted from the engine's first at `rate` Hz)
    /// run, `clocks` the block's: each copy moved on to each tick at the settings in force
    /// since the one before, its knobs then moved by the sources' values at that one, its value
    /// there. The first (the patch's first source on an LFO's knob) starts them at the latest
    /// tick from where the generators are.
    pub fn cross_to(&mut self, l: &Lfos, clocks: &Clocks, played: u64, rate: f64, voices: usize) {
        let due = tick_at(played, rate);
        let n = voices.min(VOICES);
        // (Started, or behind by more than a second: from here.)
        if !self.cross.on || due > self.cross.next + TICKS_A_SECOND as u64 {
            self.cross.on = true;
            self.cross.next = due + 1;
            self.cross.shared = l.lfo;
            self.cross.voices = [l.lfo; VOICES];
            self.cross.shared_out = self.shared_values;
            self.cross.voice_out = self.voice_values;
            self.cross.routes = Routes::of(l);
            let shared = self.shared_values;
            self.cross.tempo = self.cross.routes.tempo(l, |i| match Source::of_index(i) {
                Some(Source::Lfo(m)) => shared[m],
                Some(Source::Macro(m)) => f64::from(l.macros[m.min(MACROS - 1)]),
                None => 0.0,
            });
            return;
        }
        self.cross.routes = Routes::of(l);
        if self.cross.routes.tempo_n == 0 {
            self.cross.tempo = None;
        }
        while self.cross.next <= due {
            let k = self.cross.next;
            self.tick(l, clocks, k, n);
            self.cross.next = k + 1;
        }
    }

    /// Tick `k` for the shared copies and voices `..n`' own.
    fn tick(&mut self, l: &Lfos, clocks: &Clocks, k: u64, n: usize) {
        let Bank {
            shared,
            voices,
            cross,
            ..
        } = self;
        let t = k as f64 / TICKS_A_SECOND;
        // Moved on from the latest tick, at the settings in force since.
        for j in 0..LFOS {
            shared[j].tick_on(&cross.shared[j], clocks.hz(j, &cross.shared[j]));
            if !l.lfo[j].mono {
                for (g, s) in voices[..n].iter_mut().zip(&cross.voices[..n]) {
                    g[j].tick_on(&s[j], clocks.hz(j, &s[j]));
                }
            }
        }
        // Their knobs moved by the sources' values at the latest tick (so that nothing feeds
        // back within one): a voice's own LFO by the sources' own for that voice, a shared LFO
        // by their shared copies (option C).
        let (was_shared, was_voices) = (cross.shared_out, &cross.voice_out);
        let macros = |m: usize| f64::from(l.macros[m.min(MACROS - 1)]);
        let routes = cross.routes;
        for j in 0..LFOS {
            cross.shared[j] = moved(l, &routes, j, |i| match Source::of_index(i) {
                Some(Source::Lfo(m)) => was_shared[m],
                Some(Source::Macro(m)) => macros(m),
                None => 0.0,
            });
            if !l.lfo[j].mono {
                for (s, was) in cross.voices[..n].iter_mut().zip(&was_voices[..n]) {
                    s[j] = moved(l, &routes, j, |i| match Source::of_index(i) {
                        Some(Source::Lfo(m)) if l.own(i) => was[m],
                        Some(Source::Lfo(m)) => was_shared[m],
                        Some(Source::Macro(m)) => macros(m),
                        None => 0.0,
                    });
                }
            }
        }
        // TEMPO likewise, by the shared copies (it is the instrument's: option C).
        cross.tempo = routes.tempo(l, |i| match Source::of_index(i) {
            Some(Source::Lfo(m)) => was_shared[m],
            Some(Source::Macro(m)) => macros(m),
            None => 0.0,
        });
        // Their values here.
        for j in 0..LFOS {
            let at = clocks.at(j, t);
            cross.shared_out[j] = shared[j].tick_value(&cross.shared[j], at);
            if !l.lfo[j].mono {
                let copies = voices[..n].iter().zip(&cross.voices[..n]);
                for ((g, s), out) in copies.zip(&mut cross.voice_out[..n]) {
                    out[j] = g[j].tick_value(&s[j], at);
                }
            }
        }
    }

    /// Every LFO's value for the sound at host sample `played` (on the ticks: from its latest,
    /// at the settings in force since), its slew moved on by `dt` s; voices' own only for the
    /// LFOs with MONO off.
    pub fn peek(
        &mut self,
        l: &Lfos,
        clocks: &Clocks,
        played: u64,
        rate: f64,
        dt: f64,
        voices: usize,
    ) {
        let n = voices.min(VOICES);
        let t = played as f64 / rate;
        let ahead = (t - self.cross.latest() as f64 / TICKS_A_SECOND).max(0.0);
        let Bank {
            shared,
            voices,
            shared_values,
            voice_values,
            cross,
            ..
        } = self;
        for j in 0..LFOS {
            let at = clocks.at(j, t);
            let s = &cross.shared[j];
            shared_values[j] = shared[j].peek(s, at, clocks.hz(j, s), ahead, dt);
            if !l.lfo[j].mono {
                let copies = voices[..n].iter_mut().zip(&cross.voices[..n]);
                for ((g, s), value) in copies.zip(&mut voice_values[..n]) {
                    let s = &s[j];
                    value[j] = g[j].peek(s, at, clocks.hz(j, s), ahead, dt);
                }
            }
        }
    }
}

// ---- From the parameters.

/// LFO `n`'s (0 to 3) settings from its parameters and the patch's EDGES.
pub fn settings(p: &Ca72Params, n: usize, edges: Edges) -> Settings {
    let q = p.lfo(n);
    let at = match q.shape_mode.value() {
        WaveMode::Stepped => q.waveform.value().to_index() as f64,
        WaveMode::Continuous => shape_snapped(f64::from(q.shape.value()) * SHAPE_PLACES),
    };
    Settings {
        at,
        hz: f64::from(q.rate.value()).clamp(f64::from(RATE_HZ.0), f64::from(RATE_HZ.1)),
        timed: q
            .timed
            .value()
            .then(|| crate::timed::DIVISIONS[q.division.value().to_index()].1),
        phase: (f64::from(q.phase.value()) / 360.0).rem_euclid(1.0),
        rise: f64::from(q.rise.value()).clamp(0.0, f64::from(RISE_MOST)),
        mode: q.mode.value().into(),
        mono: q.mono.value(),
        edges,
        uni: p.lfo_keys(n).polarity.value() == Polarity::Uni,
        reverse: p.lfo_keys(n).reverse.value(),
        on: p.lfo_keys(n).on.value(),
        level: (f64::from(p.lfo_level(n).level.value()) / 100.0).clamp(0.0, 1.0),
        continuous: q.shape_mode.value() == WaveMode::Continuous,
    }
}

/// The LFOs for the engine: each one's settings, the patch's assignments, and each assigned
/// knob's travel with its parameter where `knob` has it (its glide's place).
pub fn controls(p: &Ca72Params, knob: &dyn Fn(&FloatParam) -> f32) -> Lfos {
    let patch = &p.lfo_patch;
    let law = p.freq_law();
    let mut l = Lfos {
        law: law.into(),
        ..Lfos::default()
    };
    for n in 0..LFOS {
        l.lfo[n] = settings(p, n, patch.edges(n));
    }
    let standalone = p.is_standalone();
    for s in Source::all() {
        let i = s.index();
        l.to[i] = patch.assignments(s);
        for a in &mut l.to[i] {
            // (TEMPO in a plug-in: kept in the patch, never applied, so that nothing runs.)
            if a.is_some_and(|a| a.target == Target::Tempo && !standalone) {
                *a = None;
            }
        }
        for a in l.to[i].iter().flatten() {
            let t = a.target;
            l.base[t.index()] = if t == Target::Tempo {
                tempo_travel(p.tempo()) as f32
            } else if let Some(q) = t.param(p) {
                t.travel(p, knob(q), law)
            } else {
                0.0
            };
        }
    }
    // (A macro's value only where it is assigned, so that one turned with nothing to move
    // leaves the controls as they are.)
    for n in 0..MACROS {
        if l.to[Source::Macro(n).index()]
            .iter()
            .flatten()
            .any(|a| a.depth != 0.0)
        {
            l.macros[n] = (knob(p.macro_param(n)) / 100.0).clamp(0.0, 1.0);
        }
    }
    l
}

// ---- The parameters.

/// An LFO's MODE as its parameter offers it.
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoMode {
    #[id = "free"]
    #[name = "Free"]
    Free,
    #[id = "retrig"]
    #[name = "Retrig"]
    Retrig,
    #[id = "env"]
    #[name = "Env"]
    Env,
}

/// An LFO's POLARITY (decisions.md R-LFO's corrections; the owner, 2026-10-20: "yes, bipolar is
/// exactly the right idea here. either a switch or a polarity knob. probably a switch would be
/// sufficient"): BI swings its knobs both ways about their settings, UNI one way, by the sign
/// of their depths.
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    #[id = "bi"]
    #[name = "Bipolar"]
    Bi,
    #[id = "uni"]
    #[name = "Unipolar"]
    Uni,
}

impl From<LfoMode> for Mode {
    fn from(m: LfoMode) -> Mode {
        match m {
            LfoMode::Free => Mode::Free,
            LfoMode::Retrig => Mode::Retrig,
            LfoMode::Env => Mode::Env,
        }
    }
}

impl std::fmt::Debug for LfoParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LfoParams").finish_non_exhaustive()
    }
}

/// One LFO's host parameters (IDs `lfoN_waveform` and so on, N 1 to 4): WAVEFORM (the six
/// positions), its STEPPED or CONTINUOUS mode and CONTINUOUS's shape (0..1 of the travel),
/// RATE (Hz), TIMED and DIVISION, PHASE (degrees), RISE (s), MODE and MONO (its POLARITY,
/// REVERSE and ON are [`LfoKeys`]').
#[derive(Params)]
pub struct LfoParams {
    #[id = "waveform"]
    pub waveform: EnumParam<Wave>,
    #[id = "shape_mode"]
    pub shape_mode: EnumParam<WaveMode>,
    #[id = "shape"]
    pub shape: FloatParam,
    #[id = "rate"]
    pub rate: FloatParam,
    #[id = "timed"]
    pub timed: BoolParam,
    #[id = "division"]
    pub division: EnumParam<Division>,
    #[id = "phase"]
    pub phase: FloatParam,
    #[id = "rise"]
    pub rise: FloatParam,
    #[id = "mode"]
    pub mode: EnumParam<LfoMode>,
    #[id = "mono"]
    pub mono: BoolParam,
}

impl std::fmt::Debug for LfoKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LfoKeys").finish_non_exhaustive()
    }
}

/// One LFO's POLARITY, REVERSE and ON (decisions.md R-LFO's corrections; IDs `lfoN_polarity`,
/// `lfoN_reverse` and `lfoN_on`): a nest of their own after the macros, so that no parameter
/// before them moves. A preset or a session saved before reads BI, not reversed and ON.
#[derive(Params)]
pub struct LfoKeys {
    #[id = "polarity"]
    pub polarity: EnumParam<Polarity>,
    #[id = "reverse"]
    pub reverse: BoolParam,
    #[id = "on"]
    pub on: BoolParam,
}

impl LfoKeys {
    /// LFO `n`'s (1 to 4): BI, not reversed, ON.
    pub fn new(n: usize) -> LfoKeys {
        let name = |what: &str| format!("LFO {n} {what}");
        LfoKeys {
            polarity: EnumParam::new(name("Polarity"), Polarity::Bi),
            reverse: BoolParam::new(name("Reverse"), false),
            on: BoolParam::new(name("On"), true),
        }
    }
}

impl LfoParams {
    /// LFO `n`'s (1 to 4): TRIANGLE, STEPPED, 2 Hz free, PHASE 0, no RISE, FREE, MONO.
    pub fn new(n: usize) -> LfoParams {
        let name = |what: &str| format!("LFO {n} {what}");
        LfoParams {
            waveform: EnumParam::new(name("Waveform"), Wave::Triangle),
            shape_mode: EnumParam::new(name("Waveform Mode"), WaveMode::Stepped),
            shape: FloatParam::new(
                name("Shape"),
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(Arc::new(|t| crate::params::shape_text(0, f64::from(t)))),
            // (Its travel nearly logarithmic: about 1.6 Hz at its middle.)
            rate: FloatParam::new(name("Rate"), DEFAULT_RATE, RATE_RANGE)
                .with_unit(" Hz")
                .with_value_to_string(Arc::new(rate_text))
                .with_string_to_value(Arc::new(|s| {
                    let s = s.trim().trim_end_matches("Hz").trim();
                    let v: f32 = s.parse().ok()?;
                    v.is_finite().then(|| v.clamp(RATE_HZ.0, RATE_HZ.1))
                })),
            timed: BoolParam::new(name("Timed"), false),
            division: EnumParam::new(name("Division"), Division::Quarter),
            phase: FloatParam::new(name("Phase"), 0.0, PHASE_RANGE)
                .with_unit("\u{b0}")
                .with_value_to_string(formatters::v2s_f32_rounded(0)),
            // (A cube-root travel, so that short fades have room: 1 s near its middle.)
            rise: FloatParam::new(name("Rise"), 0.0, RISE_RANGE)
                .with_unit(" s")
                .with_value_to_string(formatters::v2s_f32_rounded(2)),
            mode: EnumParam::new(name("Mode"), LfoMode::Free),
            mono: BoolParam::new(name("Mono"), true),
        }
    }

    /// Its parameters as a preset sets them, with their IDs' last parts.
    pub fn sound_params(&self) -> [(&'static str, &dyn crate::presets::SoundParam); 10] {
        [
            ("waveform", &self.waveform),
            ("shape_mode", &self.shape_mode),
            ("shape", &self.shape),
            ("rate", &self.rate),
            ("timed", &self.timed),
            ("division", &self.division),
            ("phase", &self.phase),
            ("rise", &self.rise),
            ("mode", &self.mode),
            ("mono", &self.mono),
        ]
    }
}

/// One LFO's LEVEL (decisions.md R-LFO's addendum; ID `lfoN_level`): 0 to 100 %, 100 in a new
/// patch, the share of every one of its assignments it gives, so that a macro or an LFO (itself
/// too) can fade a whole LFO in and out. Its ring is round the LFO's OUT jack, which has no
/// knob of its own: turned there by the wheel, typed there (Ctrl or Cmd and a click), by the
/// host, MIDI Learn and the agents. A nest of its own after the LFOs' keys, so that no
/// parameter before it moves; a preset or a session saved before reads 100 %.
#[derive(Params)]
pub struct LfoLevel {
    #[id = "level"]
    pub level: FloatParam,
}

impl std::fmt::Debug for LfoLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LfoLevel").finish_non_exhaustive()
    }
}

impl LfoLevel {
    /// LFO `n`'s (1 to 4): 100 %.
    pub fn new(n: usize) -> LfoLevel {
        LfoLevel {
            level: FloatParam::new(format!("LFO {n} Level"), 100.0, LEVEL_RANGE)
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_rounded(0)),
        }
    }
}

/// The LFOs' knobs' ranges, as their parameters have them and the ticks read a moved one's.
pub const RATE_RANGE: FloatRange = FloatRange::Skewed {
    min: RATE_HZ.0,
    max: RATE_HZ.1,
    factor: 0.2,
};
pub const PHASE_RANGE: FloatRange = FloatRange::Linear {
    min: 0.0,
    max: 360.0,
};
pub const RISE_RANGE: FloatRange = FloatRange::Skewed {
    min: 0.0,
    max: RISE_MOST,
    factor: 1.0 / 3.0,
};
pub const LEVEL_RANGE: FloatRange = FloatRange::Linear {
    min: 0.0,
    max: 100.0,
};

/// How many parameters an LFO has: [`LfoParams`]' ten, [`LfoKeys`]' three and its LEVEL.
pub const PARTS: usize = 14;

/// LFO `n`'s (0 to 3) parameters as a preset sets them, [`PARAM_IDS`]' order: its
/// [`LfoParams`]', then its POLARITY, REVERSE and ON, then its LEVEL.
pub fn sound_params(p: &Ca72Params, n: usize) -> [&dyn crate::presets::SoundParam; PARTS] {
    let (q, k) = (p.lfo(n), p.lfo_keys(n));
    let [a, b, c, d, e, f, g, h, i, j] = q.sound_params().map(|(_, v)| v);
    [
        a,
        b,
        c,
        d,
        e,
        f,
        g,
        h,
        i,
        j,
        &k.polarity,
        &k.reverse,
        &k.on,
        &p.lfo_level(n).level,
    ]
}

/// RATE as the host shows it: Hz to two figures under 1 Hz, else to a tenth or whole.
pub fn rate_text(hz: f32) -> String {
    if hz < 1.0 {
        format!("{hz:.2} Hz")
    } else if hz < 10.0 {
        format!("{hz:.1} Hz")
    } else {
        format!("{hz:.0} Hz")
    }
}

/// Each LFO's parameters' IDs, in [`sound_params`]' order.
pub const PARAM_IDS: [[&str; PARTS]; LFOS] = [
    [
        "lfo1_waveform",
        "lfo1_shape_mode",
        "lfo1_shape",
        "lfo1_rate",
        "lfo1_timed",
        "lfo1_division",
        "lfo1_phase",
        "lfo1_rise",
        "lfo1_mode",
        "lfo1_mono",
        "lfo1_polarity",
        "lfo1_reverse",
        "lfo1_on",
        "lfo1_level",
    ],
    [
        "lfo2_waveform",
        "lfo2_shape_mode",
        "lfo2_shape",
        "lfo2_rate",
        "lfo2_timed",
        "lfo2_division",
        "lfo2_phase",
        "lfo2_rise",
        "lfo2_mode",
        "lfo2_mono",
        "lfo2_polarity",
        "lfo2_reverse",
        "lfo2_on",
        "lfo2_level",
    ],
    [
        "lfo3_waveform",
        "lfo3_shape_mode",
        "lfo3_shape",
        "lfo3_rate",
        "lfo3_timed",
        "lfo3_division",
        "lfo3_phase",
        "lfo3_rise",
        "lfo3_mode",
        "lfo3_mono",
        "lfo3_polarity",
        "lfo3_reverse",
        "lfo3_on",
        "lfo3_level",
    ],
    [
        "lfo4_waveform",
        "lfo4_shape_mode",
        "lfo4_shape",
        "lfo4_rate",
        "lfo4_timed",
        "lfo4_division",
        "lfo4_phase",
        "lfo4_rise",
        "lfo4_mode",
        "lfo4_mono",
        "lfo4_polarity",
        "lfo4_reverse",
        "lfo4_on",
        "lfo4_level",
    ],
];

/// Each LFO's parameters' labels for the agents (docs/mcp.md), [`PARAM_IDS`]' order, and
/// each LFO's section.
pub const PARAM_LABELS: [[&str; PARTS]; LFOS] = [
    [
        "LFO 1 WAVEFORM",
        "LFO 1 WAVEFORM MODE",
        "LFO 1 SHAPE",
        "LFO 1 RATE",
        "LFO 1 TIMED",
        "LFO 1 DIVISION",
        "LFO 1 PHASE",
        "LFO 1 RISE",
        "LFO 1 MODE",
        "LFO 1 MONO",
        "LFO 1 POLARITY",
        "LFO 1 REVERSE",
        "LFO 1 ON",
        "LFO 1 LEVEL",
    ],
    [
        "LFO 2 WAVEFORM",
        "LFO 2 WAVEFORM MODE",
        "LFO 2 SHAPE",
        "LFO 2 RATE",
        "LFO 2 TIMED",
        "LFO 2 DIVISION",
        "LFO 2 PHASE",
        "LFO 2 RISE",
        "LFO 2 MODE",
        "LFO 2 MONO",
        "LFO 2 POLARITY",
        "LFO 2 REVERSE",
        "LFO 2 ON",
        "LFO 2 LEVEL",
    ],
    [
        "LFO 3 WAVEFORM",
        "LFO 3 WAVEFORM MODE",
        "LFO 3 SHAPE",
        "LFO 3 RATE",
        "LFO 3 TIMED",
        "LFO 3 DIVISION",
        "LFO 3 PHASE",
        "LFO 3 RISE",
        "LFO 3 MODE",
        "LFO 3 MONO",
        "LFO 3 POLARITY",
        "LFO 3 REVERSE",
        "LFO 3 ON",
        "LFO 3 LEVEL",
    ],
    [
        "LFO 4 WAVEFORM",
        "LFO 4 WAVEFORM MODE",
        "LFO 4 SHAPE",
        "LFO 4 RATE",
        "LFO 4 TIMED",
        "LFO 4 DIVISION",
        "LFO 4 PHASE",
        "LFO 4 RISE",
        "LFO 4 MODE",
        "LFO 4 MONO",
        "LFO 4 POLARITY",
        "LFO 4 REVERSE",
        "LFO 4 ON",
        "LFO 4 LEVEL",
    ],
];
pub const SECTIONS: [&str; LFOS] = ["LFO 1", "LFO 2", "LFO 3", "LFO 4"];

/// LFO parameter `id`'s LFO (0 to 3) and its place in [`PARAM_IDS`]' row.
fn param_place(id: &str) -> Option<(usize, usize)> {
    PARAM_IDS
        .iter()
        .enumerate()
        .find_map(|(n, row)| row.iter().position(|r| *r == id).map(|k| (n, k)))
}

/// An LFO's parameter of `id`, as MIDI Learn sets it (`learn::target`).
pub fn learn_target<'a>(p: &'a Ca72Params, id: &str) -> Option<&'a dyn crate::learn::Target> {
    let (n, k) = param_place(id)?;
    let q = p.lfo(n);
    let keys = p.lfo_keys(n);
    Some(match k {
        0 => &q.waveform,
        1 => &q.shape_mode,
        2 => &q.shape,
        3 => &q.rate,
        4 => &q.timed,
        5 => &q.division,
        6 => &q.phase,
        7 => &q.rise,
        8 => &q.mode,
        9 => &q.mono,
        10 => &keys.polarity,
        11 => &keys.reverse,
        12 => &keys.on,
        _ => &p.lfo_level(n).level,
    })
}

/// An LFO's knob of `id` (SHAPE, RATE, PHASE, RISE, LEVEL), if it is one (`learn::knob_param`).
/// They do not glide: none is on the audio path (R-LFO; EDGES smooths what they step).
pub fn knob<'a>(p: &'a Ca72Params, id: &str) -> Option<&'a FloatParam> {
    let (n, k) = param_place(id)?;
    let q = p.lfo(n);
    match k {
        2 => Some(&q.shape),
        3 => Some(&q.rate),
        6 => Some(&q.phase),
        7 => Some(&q.rise),
        13 => Some(&p.lfo_level(n).level),
        _ => None,
    }
}

/// Whether `id` is one of the LFOs' knobs (continuous, not gliding).
pub fn is_knob_id(id: &str) -> bool {
    param_place(id).is_some_and(|(_, k)| matches!(k, 2 | 3 | 6 | 7 | 13))
}

/// The IDs of each LFO's parameters, `lfoN_<part>`, LFO by LFO ([`PARAM_IDS`]' order; the host
/// lists each LFO's POLARITY, REVERSE and ON after the macros).
pub fn param_ids() -> impl Iterator<Item = String> {
    PARAM_IDS.iter().flatten().map(|id| (*id).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each place is its legend's picture: TRIANGLE from its foot to its top at half way;
    /// the shark tooth up to its top at 14/22, down to the middle and on to its foot; the
    /// sawtooth rising; the rectangles high first. Bipolar about zero, -1 to 1.
    #[test]
    fn the_shapes_are_the_legends() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(close(wave(0.0, 0.0), -1.0) && close(wave(0.0, 0.5), 1.0));
        assert!(close(wave(0.0, 0.25), 0.0) && close(wave(0.0, 0.75), 0.0));
        assert!(close(wave(1.0, 0.0), -1.0));
        assert!((wave(1.0, SHARK_PEAK - 1e-12) - 1.0).abs() < 1e-9);
        assert!(close(wave(1.0, SHARK_PEAK), 0.0));
        assert!(close(wave(2.0, 0.0), -1.0) && close(wave(2.0, 0.5), 0.0));
        assert!(wave(2.0, 1.0 - 1e-9) > 0.999_999);
        assert_eq!(wave(3.0, 0.49), 1.0);
        assert_eq!(wave(3.0, 0.51), -1.0);
        assert!(close(duty(3.0), 0.5) && close(duty(4.0), 0.30) && close(duty(5.0), 0.179));
        assert!((duty(SHAPE_PLACES) - 0.064).abs() < 0.001);
        for at in [0.0, 0.4, 1.0, 1.7, 2.0, 2.5, 3.0, 4.2, SHAPE_PLACES] {
            for k in 0..1000 {
                let x = wave(at, k as f64 / 1000.0);
                assert!((-1.0..=1.0).contains(&x), "{at} {k} {x}");
            }
        }
        // Between two places, a blend of the two.
        let p = 0.3;
        assert!(close(wave(2.4, p), 0.6 * wave(2.0, p) + 0.4 * wave(3.0, p)));
        // PHASE is added to the phase: a whole cycle on is the same.
        assert!(close(wave(1.0, 0.2 + 3.0), wave(1.0, 0.2)));
    }

    /// The patch: at most eight a source, a depth set again in place, removal keeping the
    /// others' order, a saved form and a preset's values read back the same.
    #[test]
    fn assignments_are_kept_set_and_saved() {
        let (l0, l2) = (Source::Lfo(0), Source::Lfo(2));
        let p = Patch::default();
        assert_eq!(p.assign(l0, Target::Cutoff, 0.4), Ok(()));
        assert_eq!(p.assign(l0, Target::Osc2Frequency, -0.1), Ok(()));
        assert_eq!(p.assign(l0, Target::Cutoff, 0.5), Ok(()));
        assert_eq!(p.count(l0), 2);
        assert_eq!(p.depth(l0, Target::Cutoff), Some(0.5));
        for t in &Target::ALL[..7] {
            assert_eq!(p.assign(l0, *t, 0.1), Ok(()));
        }
        assert_eq!(p.count(l0), MOST);
        assert_eq!(p.assign(l0, Target::Drive, 0.1), Err(Refused::Full));
        assert_eq!(
            p.assign(Source::Lfo(4), Target::Drive, 0.1),
            Err(Refused::NoSource)
        );
        assert_eq!(
            p.assign(Source::Macro(MACROS), Target::Drive, 0.1),
            Err(Refused::NoSource)
        );
        assert!(p.remove(l0, Target::Cutoff));
        assert!(!p.remove(l0, Target::Cutoff));
        assert_eq!(
            p.assignments(l0)[0].map(|a| a.target),
            Some(Target::Osc2Frequency)
        );
        p.assign(l2, Target::Volume, -2.0).expect("room");
        assert_eq!(p.depth(l2, Target::Volume), Some(-1.0));
        assert_eq!(p.on_knob(Target::Volume)[2], Some(-1.0));
        p.set_edges(3, Edges::Sharp);
        let q = Patch::default();
        q.load(&p.saved());
        assert_eq!(q.saved(), p.saved());
        let r = Patch::default();
        r.load_values(&p.values());
        for s in Source::all() {
            let mut a: Vec<_> = p
                .assignments(s)
                .into_iter()
                .flatten()
                .map(|a| a.target)
                .collect();
            a.sort();
            let b: Vec<_> = r
                .assignments(s)
                .into_iter()
                .flatten()
                .map(|a| a.target)
                .collect();
            assert_eq!(a, b);
        }
        assert_eq!(r.edges(3), Edges::Sharp);
        r.load_values(&[]);
        assert!(Source::all().all(|s| r.count(s) == 0));
        assert!((0..LFOS).all(|l| r.edges(l) == Edges::Soft));
        assert!(Patch::is_value_id("lfo2_to_cutoff") && Patch::is_value_id("lfo4_edges"));
        assert!(!Patch::is_value_id("lfo2_rate") && !Patch::is_value_id("lfo5_edges"));
        assert_eq!(Source::of_id("lfo3"), Some(l2));
        assert_eq!(Source::all().count(), SOURCES);
    }

    /// A macro's name: capitals, printable, seven at most, trimmed; kept with the patch, a
    /// session's and a preset's read back, one saved before macros with none (decisions.md
    /// R-MACROS).
    #[test]
    fn a_macros_name_is_kept_and_saved() {
        assert_eq!(name_within("  bright!  "), "BRIGHT!");
        assert_eq!(name_within("wobble wobble"), "WOBBLE");
        assert_eq!(name_within("é\u{7f}ab"), "AB");
        assert_eq!(name_within("   "), "");
        let p = Patch::default();
        assert_eq!(p.name(0), "");
        p.set_name(0, "Bright");
        p.set_name(4, "swell");
        p.set_name(MACROS, "none");
        assert_eq!(p.names()[..], ["BRIGHT", "", "", "", "SWELL", ""]);
        p.assign(Source::Macro(0), Target::Cutoff, 0.6)
            .expect("room");
        let q = Patch::default();
        q.load(&p.saved());
        assert_eq!(q.saved(), p.saved());
        assert_eq!(q.name(4), "SWELL");
        assert_eq!(q.depth(Source::Macro(0), Target::Cutoff), Some(0.6));
        assert!(Patch::is_value_id("macro1_to_cutoff"));
        assert!(!Patch::is_value_id("macro1_edges"));
        // A session saved before macros: no names; one with none typed keeps its old form.
        let old: Saved = serde_json::from_str(r#"{"sources":[]}"#).expect("old form");
        q.load(&old);
        assert!(q.names().iter().all(String::is_empty));
        assert_eq!(
            serde_json::to_string(&Patch::default().saved()).expect("json"),
            r#"{"sources":[]}"#
        );
        // A preset's.
        let r = Patch::default();
        r.load_values(&p.values());
        r.load_names(&p.names());
        assert!(r.same_as_values(&p.values()) && r.same_names(&p.names()));
        r.load_names(&[]);
        assert!(!r.same_names(&p.names()) && r.same_names(&[]));
        r.copy_from(&p);
        assert_eq!(r.name(0), "BRIGHT");
        r.clear();
        assert_eq!(r.name(0), "");
    }

    /// A macro moves its knobs one way: by depth × its value (0..1), from where the knob's glide
    /// has it, clamped to the travel; its value only where it is assigned, so that one turned
    /// with nothing to move leaves the controls as they are (and the instrument asleep).
    #[test]
    fn a_macro_moves_its_knobs_one_way() {
        let p = Ca72Params::default();
        let plain = |q: &FloatParam, v: f32| q.preview_plain(q.preview_normalized(v));
        let at = |k: &FloatParam| k.value();
        let unassigned = controls(&p, &|k: &FloatParam| {
            if std::ptr::eq(k, p.macro_param(2)) {
                plain(k, 80.0)
            } else {
                at(k)
            }
        });
        assert_eq!(
            unassigned,
            Lfos {
                law: p.freq_law().into(),
                ..Lfos::default()
            }
        );
        p.lfo_patch
            .assign(Source::Macro(2), Target::Cutoff, 0.6)
            .expect("room");
        p.lfo_patch
            .assign(Source::Macro(2), Target::Emphasis, -0.2)
            .expect("room");
        let turned = |v: f32| {
            controls(&p, &|k: &FloatParam| {
                if std::ptr::eq(k, p.macro_param(2)) {
                    v
                } else {
                    at(k)
                }
            })
        };
        let l = turned(50.0);
        assert!((l.macros[2] - 0.5).abs() < 1e-6);
        let c = p.controls();
        let values = l.source_values(&[0.0; LFOS]);
        assert_eq!(values[Source::Macro(2).index()], 0.5);
        let m = l.modulated(&c, &values);
        assert!(
            (m.panel.cutoff - (c.panel.cutoff + 0.3)).abs() < 1e-6,
            "{}",
            m.panel.cutoff
        );
        assert!((m.panel.emphasis - (c.panel.emphasis - 0.1).max(0.0)).abs() < 1e-6);
        // Clamped at the knob's end; at 0 the knob's own place.
        let l = turned(100.0);
        let m = l.modulated(&c, &l.source_values(&[0.0; LFOS]));
        assert!(
            m.panel.cutoff <= 1.0
                && (m.panel.cutoff - (c.panel.cutoff + 0.6).min(1.0)).abs() < 1e-6
        );
        let l = turned(0.0);
        let m = l.modulated(&c, &l.source_values(&[0.0; LFOS]));
        assert!((m.panel.cutoff - c.panel.cutoff).abs() < 1e-6);
        assert!(!l.own(Source::Macro(2).index()));
    }

    /// The standalone's TEMPO a target (decisions.md R-LFO): found by its ID, named TEMPO, a
    /// target the voices share, on the LFOs' ticks, its travel 30 to 300 BPM straight along it;
    /// one the standalone alone can take; saved with a session and a preset and read back in a
    /// plug-in too (kept, never applied: [`controls`] leaves it out there).
    #[test]
    fn tempo_is_a_target_of_the_standalones_alone() {
        let t = Target::Tempo;
        assert_eq!(
            (t.id(), t.short(), t.name()),
            ("tempo", "TEMPO", "TEMPO".to_owned())
        );
        assert_eq!(Target::of_id("tempo"), Some(t));
        assert_eq!(
            t.index(),
            TEMPO_AT,
            "after the LFOs' own, before the effects' pins"
        );
        assert!(!t.per_voice() && t.ticks() && !t.is_lfo());
        assert_eq!(
            (tempo_at(0.0), tempo_at(1.0), tempo_at(2.0)),
            (30.0, 300.0, 300.0)
        );
        assert!((tempo_travel(120.0) - 1.0 / 3.0).abs() < 1e-12);
        let params = Ca72Params::default();
        assert_eq!(target_of(&params, "tempo"), None, "a plug-in's");
        params.set_standalone(true);
        assert_eq!(target_of(&params, "tempo"), Some(t));
        // The parameter's law is TEMPO's (the depth a share of its travel).
        assert!((t.travel(&params, 165.0, Law::Semitones) - 0.5).abs() < 1e-6);
        assert!((t.plain_at(&params, 0.25, Law::Semitones) - 97.5).abs() < 1e-3);
        params.lfo_patch.assign(Source::Lfo(1), t, -0.4).unwrap();
        params.lfo_patch.assign(Source::Macro(5), t, 0.7).unwrap();
        let values = params.lfo_patch.values();
        let saved = params.lfo_patch.saved();
        params.set_standalone(false);
        for (q, how) in [(Patch::default(), "session"), (Patch::default(), "preset")] {
            if how == "session" {
                q.load(&saved);
                assert_eq!(q.saved(), saved);
            } else {
                q.load_values(&values);
                assert!(q.same_as_values(&values));
            }
            assert_eq!(q.depth(Source::Lfo(1), t), Some(-0.4), "{how}");
            assert_eq!(q.depth(Source::Macro(5), t), Some(0.7), "{how}");
        }
        // In a plug-in the controls leave it out: nothing runs.
        let l = controls(&params, &|q: &FloatParam| q.value());
        assert!(!l.active() && !l.cross() && !l.tempo_moved());
        params.set_standalone(true);
        let l = controls(&params, &|q: &FloatParam| q.value());
        assert!(l.active() && l.cross() && l.tempo_moved());
        assert!((l.base[t.index()] - (1.0 / 3.0)).abs() < 1e-6);
    }

    /// An LFO's own knobs as targets (R-LFO's addendum): every target at its index, found by
    /// its ID, its screen name ten characters at most; LFO 2's LEVEL apart from the strip's
    /// (`lfo2_level`, "LFO2 LEVEL", "LFO 2 LEVEL"; `level`, "LEVEL"); saved with a session and a
    /// preset (`lfo1_to_lfo2_rate`) and read back; RATE none while timed, WAVEFORM its shape only
    /// while CONTINUOUS, the shape's own ID none; a MONO off LFO's copies on a MONO on LFO's knob
    /// the shared one's (option C).
    #[test]
    fn an_lfos_own_knobs_are_targets() {
        use nih_plug::context::process::TestProcessContext;
        for (i, t) in Target::ALL.iter().enumerate() {
            assert_eq!(t.index(), i, "{t:?}");
            assert_eq!(Target::of_id(t.id()), Some(*t));
            assert!(t.short().len() <= 10, "{}", t.short());
        }
        let level = Target::LfoLevel(1);
        assert_eq!(
            (level.id(), level.short(), level.name()),
            ("lfo2_level", "LFO2 LEVEL", "LFO 2 LEVEL".to_owned())
        );
        assert_eq!(
            (Target::Level.id(), Target::Level.short()),
            ("level", "LEVEL")
        );
        assert!(level.is_lfo() && !Target::Level.is_lfo() && !level.per_voice());
        let p = Patch::default();
        p.assign(Source::Lfo(0), Target::LfoRate(1), 0.4)
            .expect("room");
        p.assign(Source::Lfo(2), Target::LfoPhase(2), -0.2)
            .expect("room");
        p.assign(Source::Macro(2), Target::LfoLevel(3), -0.5)
            .expect("room");
        assert!(
            p.values()
                .iter()
                .any(|(k, v)| k == "lfo1_to_lfo2_rate" && (*v - 40.0).abs() < 1e-3)
        );
        assert!(Patch::is_value_id("macro3_to_lfo4_level"));
        let q = Patch::default();
        q.load_values(&p.values());
        assert!(q.same_as_values(&p.values()));
        assert_eq!(q.depth(Source::Macro(2), Target::LfoLevel(3)), Some(-0.5));
        let r = Patch::default();
        r.load(&p.saved());
        assert_eq!(r.saved(), p.saved());
        // Which can take one now.
        let params = Arc::new(Ca72Params::default());
        assert_eq!(target_of(&params, "lfo1_rate"), Some(Target::LfoRate(0)));
        assert_eq!(target_of(&params, "lfo2_level"), Some(Target::LfoLevel(1)));
        assert_eq!(target_of(&params, "lfo3_rise"), Some(Target::LfoRise(2)));
        assert_eq!(target_of(&params, "lfo4_phase"), Some(Target::LfoPhase(3)));
        assert_eq!(target_of(&params, "lfo1_waveform"), None, "STEPPED");
        assert_eq!(target_of(&params, "lfo1_shape"), None);
        assert_eq!(target_of(&params, "lfo1_timed"), None);
        let c =
            TestProcessContext::<crate::Ca72>::new(params.clone(), 48_000.0, ProcessMode::Realtime);
        assert!(c.automate(params.lfo(0).shape_mode.as_ptr(), 1.0));
        assert!(c.automate(params.lfo(0).timed.as_ptr(), 1.0));
        assert_eq!(
            target_of(&params, "lfo1_waveform"),
            Some(Target::LfoShape(0))
        );
        assert_eq!(target_of(&params, "lfo1_rate"), None, "timed");
        // The shared copy's: LFO 1 MONO off on LFO 2's (MONO on), not on its own.
        assert!(c.automate(params.lfo(0).mono.as_ptr(), 0.0));
        assert!(takes_shared(&params, Source::Lfo(0), Target::LfoRate(1)));
        assert!(!takes_shared(&params, Source::Lfo(0), Target::LfoRate(0)));
        assert!(takes_shared(&params, Source::Lfo(0), Target::Volume));
        assert!(!takes_shared(&params, Source::Lfo(0), Target::Cutoff));
        assert!(
            !takes_shared(&params, Source::Lfo(1), Target::Volume),
            "MONO on"
        );
        assert!(!takes_shared(&params, Source::Macro(0), Target::LfoRate(0)));
        // A LEVEL a share of its LFO's every assignment: its settings', from its parameter.
        assert_eq!(settings(&params, 0, Edges::Soft).level, 1.0);
    }

    /// The control a target sets, as the engine has it.
    fn field(t: Target, c: &Controls) -> f64 {
        let p = &c.panel;
        match t {
            Target::Tune => p.tune,
            Target::Glide => p.glide,
            Target::ModMix => p.mod_mix,
            Target::Osc2Frequency => p.osc[1].freq,
            Target::Osc3Frequency => p.osc[2].freq,
            Target::Osc1Volume => p.osc[0].volume,
            Target::ExtVolume => p.ext_volume,
            Target::Osc2Volume => p.osc[1].volume,
            Target::NoiseVolume => p.noise_volume,
            Target::Osc3Volume => p.osc[2].volume,
            Target::Cutoff => p.cutoff,
            Target::Emphasis => p.emphasis,
            Target::ContourAmount => p.contour_amount,
            Target::FilterAttack => p.filter_contour.attack,
            Target::FilterDecay => p.filter_contour.decay,
            Target::FilterSustain => p.filter_contour.sustain,
            Target::LoudnessAttack => p.loudness_contour.attack,
            Target::LoudnessDecay => p.loudness_contour.decay,
            Target::LoudnessSustain => p.loudness_contour.sustain,
            Target::Volume => c.volume,
            Target::Feedback => c.feedback,
            Target::Entropy => c.entropy,
            Target::Spread => c.spread,
            Target::Double => c.double,
            Target::Drive => c.drive,
            Target::Inner => c.inner,
            Target::Osc1Shape => p.shape[0].at,
            Target::Osc2Shape => p.shape[1].at,
            Target::Osc3Shape => p.shape[2].at,
            Target::Level => c.level,
            // (An LFO's own and TEMPO: not the controls', the ticks' (engine_lfo_tests,
            // tempo_tests); an effects' pin, the chain's.)
            Target::LfoRate(_)
            | Target::LfoPhase(_)
            | Target::LfoRise(_)
            | Target::LfoShape(_)
            | Target::LfoLevel(_)
            | Target::Tempo
            | Target::Fx(_) => 0.0,
        }
    }

    /// Every target's law is its parameter's as the engine's controls take it: the knob at a
    /// travel, written by the target, is the controls with the parameter at that travel's
    /// value (within an f32's rounding), under both FREQUENCY laws; and travel and value are
    /// each other's inverses.
    #[test]
    fn every_targets_law_is_the_parameters() {
        let p = Ca72Params::default();
        for law in [Law::Semitones, Law::Hardware] {
            p.set_freq_law(law);
            for t in Target::ALL {
                for x in [0.0f32, 0.07, 0.25, 0.5, 0.61, 0.9, 1.0] {
                    let plain = t.plain_at(&p, x, law);
                    let back = t.travel(&p, plain, law);
                    assert!((back - x).abs() < 2e-4, "{t:?} {x} {back}");
                    // (An effects' pin has no parameter here: its travel is its value.)
                    let Some(q) = t.param(&p) else {
                        assert_eq!(plain, x);
                        continue;
                    };
                    let want = p.controls_with(&|k| {
                        if std::ptr::eq(k, q) { plain } else { k.value() }
                    });
                    let mut got = p.controls();
                    t.write(&mut got, f64::from(back), law);
                    let (a, b) = (field(t, &got), field(t, &want));
                    let tol = if t == Target::Drive { 1e-4 } else { 1e-5 };
                    assert!((a - b).abs() < tol, "{t:?} {law:?} at {x}: {a} {b}");
                }
            }
        }
    }

    /// The modulation starts from where the knob's glide has it, not its parameter's value:
    /// the base travel is the glided value's (R-KNOB-GLIDE).
    #[test]
    fn the_base_is_the_knobs_glided_place() {
        let p = Ca72Params::default();
        p.lfo_patch
            .assign(Source::Lfo(1), Target::Cutoff, 0.3)
            .expect("room");
        let gliding = |k: &FloatParam| {
            if std::ptr::eq(k, &p.cutoff) {
                2.5
            } else {
                k.value()
            }
        };
        let l = controls(&p, &gliding);
        assert!((l.base[Target::Cutoff.index()] - 0.75).abs() < 1e-6);
        assert_eq!(l.to[1][0].map(|a| a.depth), Some(0.3));
        assert!(l.active() && !l.per_voice());
        assert_eq!(p.controls_with(&gliding).lfo, l);
    }

    /// The effects' pins' assignments are the chain's (decisions.md R-FX-PARAMS): kept by the
    /// patch for the engine and the editor, never saved with the sound, left as they are by a
    /// preset's or session's assignments, Init and a pad's preset, and counted apart; a pin
    /// takes a source only while it is pinned to an effect that runs.
    #[test]
    fn the_effects_pins_assignments_are_the_chains() {
        let p = Patch::default();
        let before = p.fx_changes();
        p.assign(Source::Lfo(0), Target::Cutoff, 0.5).unwrap();
        assert_eq!(p.fx_changes(), before, "a panel knob's is the sound's");
        p.assign(Source::Lfo(0), Target::Fx(2), 0.3).unwrap();
        p.assign(Source::Macro(1), Target::Fx(15), -0.4).unwrap();
        assert_eq!(p.fx_changes(), before + 2);
        assert_eq!(Target::Fx(2).id(), "fx3");
        assert_eq!(Target::of_id("fx16"), Some(Target::Fx(15)));
        assert_eq!(Target::ALL[TARGETS - 1], Target::Fx(15));
        // Not the sound's.
        let saved = p.saved();
        assert!(
            saved
                .sources
                .iter()
                .all(|s| s.to.iter().all(|(t, _)| !t.starts_with("fx")))
        );
        assert!(p.values().iter().all(|(k, _)| !k.contains("_to_fx")));
        // A session's assignments, Init, and a pad's preset leave them.
        let other = Patch::default();
        other.assign(Source::Lfo(2), Target::Emphasis, 0.2).unwrap();
        other.assign(Source::Lfo(0), Target::Fx(0), 0.9).unwrap();
        p.load(&other.saved());
        assert_eq!(p.depth(Source::Lfo(0), Target::Fx(2)), Some(0.3));
        assert_eq!(p.depth(Source::Lfo(0), Target::Cutoff), None);
        assert_eq!(p.depth(Source::Lfo(2), Target::Emphasis), Some(0.2));
        p.copy_from(&other);
        assert_eq!(p.depth(Source::Lfo(0), Target::Fx(2)), Some(0.3));
        assert_eq!(
            p.depth(Source::Lfo(0), Target::Fx(0)),
            None,
            "not the other's"
        );
        p.clear();
        assert_eq!(p.depth(Source::Macro(1), Target::Fx(15)), Some(-0.4));
        assert_eq!(p.count(Source::Lfo(2)), 0);
        assert!(
            p.same_as_values(&[]),
            "a preset with nothing: the same sound"
        );
        // Taken where pinned to an effect that runs.
        let params = Ca72Params::default();
        let was = FX_PINNED.swap(0, Ordering::AcqRel);
        assert_eq!(target_of(&params, "fx3"), None);
        FX_PINNED.store(1 << 2, Ordering::Release);
        assert_eq!(target_of(&params, "fx3"), Some(Target::Fx(2)));
        FX_PINNED.store(was, Ordering::Release);
        // The engine's controls: their offsets, the shared values, and the pins moved.
        let l = Lfos {
            to: std::array::from_fn(|i| p.assignments(Source::of_index(i).unwrap())),
            ..Lfos::default()
        };
        assert_eq!(l.fx_moved(), (1 << 15) | (1 << 2));
        let mut v = [0.0; SOURCES];
        v[Source::Macro(1).index()] = 0.5;
        assert!((l.fx_offsets(&v)[15] + 0.2).abs() < 1e-6);
        // They move no control of the synth's.
        let c = Controls::default();
        assert_eq!(l.modulated(&c, &v), c);
    }
}
