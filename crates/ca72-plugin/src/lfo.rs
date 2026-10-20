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
//!   from PHASE and holds its last value (timed, one note value long). A new note follows the
//!   panel's RETRIGGER switch: off, only a key pressed with none held; on, every key.
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
/// How often while a source moves a pitch or CUTOFF ([`Target::fast`]) at QUALITY HI or ULTRA,
/// samples (decisions.md R-LFO-STRESS): at [`STEP`] a fast LFO there is heard as a fizz 1.5 kHz
/// from the sound, its steps' rate; at 8 that is gone, moved to 6 kHz and a quarter as strong.
pub const FAST_STEP: usize = 8;
/// The voices that can each have their own LFOs (POLY's most).
pub const VOICES: usize = POLY_VOICES.2;

// ---- What an LFO can modulate.

/// A knob an LFO can modulate: the panel's continuous knobs and the strip's (26), and each
/// oscillator's WAVEFORM while it is CONTINUOUS (its shape's travel). Its parameter's ID
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
}

/// How many targets there are.
pub const TARGETS: usize = 29;

impl Target {
    pub const ALL: [Target; TARGETS] = [
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
    ];

    /// Its place in [`Target::ALL`].
    pub fn index(self) -> usize {
        self as usize
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
        }
    }

    /// Whether its steps are heard at [`STEP`] (decisions.md R-LFO-STRESS): a pitch (TUNE, the
    /// FREQUENCYs) or CUTOFF, moved by [`FAST_STEP`]s instead at HI and ULTRA.
    pub fn fast(self) -> bool {
        matches!(
            self,
            Target::Tune | Target::Osc2Frequency | Target::Osc3Frequency | Target::Cutoff
        )
    }

    /// Whether each voice has its own (it is on the voice's panel, or FEEDBACK, each voice's
    /// own loop): an LFO with MONO off moves it a voice at a time. The others the voices share
    /// (MAIN OUTPUT's VOLUME, the strip's ENTROPY, WIDTH, INNER, DETUNE and DRIVE), which take
    /// the LFO's shared copy whatever MONO is (option C).
    pub fn per_voice(self) -> bool {
        !matches!(
            self,
            Target::Volume
                | Target::Entropy
                | Target::Spread
                | Target::Double
                | Target::Drive
                | Target::Inner
        )
    }

    /// Its parameter.
    pub fn param(self, p: &Ca72Params) -> &FloatParam {
        match self {
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
        }
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
        let n = self.param(p).preview_normalized(plain);
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
            None => self.param(p).preview_plain(t),
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
            Target::Feedback => c.feedback = ca72::voice::feedback_law(t),
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
            | Target::Feedback
            | Target::Entropy
            | Target::Spread
            | Target::Double
            | Target::Drive
            | Target::Inner => {}
        }
    }
}

/// The target a panel control can take an LFO on now, by the control's parameter's ID: a
/// continuous knob of the panel's or the strip's; a WAVEFORM switch only while CONTINUOUS
/// (`oscN_waveform` gives its shape); a FREQUENCY knob not while its oscillator is timed (it is
/// a DIVISION dial then, stepped). None for anything else: switches, stepped selectors,
/// VOICES, the timed dials, the wheels, the drawers' knobs and the LFOs' own. An assignment
/// already made stays while its knob cannot take one (it does nothing then).
pub fn target_of(p: &Ca72Params, param_id: &str) -> Option<Target> {
    let wave = ["osc1_waveform", "osc2_waveform", "osc3_waveform"];
    if let Some(n) = wave.iter().position(|w| *w == param_id) {
        return p
            .continuous(n)
            .then_some([Target::Osc1Shape, Target::Osc2Shape, Target::Osc3Shape][n]);
    }
    let t = Target::of_id(param_id)?;
    match t {
        Target::Osc1Shape | Target::Osc2Shape | Target::Osc3Shape => None,
        Target::Osc2Frequency if p.timed_now(1) => None,
        Target::Osc3Frequency if p.timed_now(2) => None,
        _ => Some(t),
    }
}

/// [`target_of`] as a yes or no.
pub fn is_lfo_target(p: &Ca72Params, param_id: &str) -> bool {
    target_of(p, param_id).is_some()
}

// ---- Modulation sources.

/// The macros (knobs of the patch's own, 0 to 100 %, sources like the LFOs; not built yet: the
/// owner's mock-ups are in progress). [`Source::Macro`] and the sources' arrays take them when
/// this is more than 0, without reshaping the patch, its saved forms or the engine.
pub const MACROS: usize = 0;
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
    /// Macro `n` (0 to [`MACROS`] less 1; none yet).
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
#[derive(Debug)]
pub struct Patch {
    slots: [[AtomicU64; MOST]; SOURCES],
    sharp: [AtomicBool; LFOS],
    changes: AtomicU32,
}

impl Default for Patch {
    fn default() -> Self {
        Patch {
            slots: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            sharp: std::array::from_fn(|_| AtomicBool::new(false)),
            changes: AtomicU32::new(0),
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
        self.changed();
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
            self.changed();
        }
        gone
    }

    /// Every source's every assignment taken away, every EDGES SOFT again (Init; a preset or
    /// session saved before LFOs).
    pub fn clear(&self) {
        for s in self.slots.iter().flatten() {
            s.store(0, Ordering::Release);
        }
        for e in &self.sharp {
            e.store(false, Ordering::Release);
        }
        self.changed();
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
                        .map(|a| (a.target.id().to_owned(), a.depth))
                        .collect(),
                    sharp: s.lfo().is_some_and(|l| self.edges(l) == Edges::Sharp),
                })
                .filter(|s| !s.to.is_empty() || s.sharp)
                .collect(),
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
                let Some(target) = Target::of_id(id) else {
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
        for (slots, w) in self.slots.iter().zip(words) {
            for (slot, w) in slots.iter().zip(w) {
                slot.store(w, Ordering::Release);
            }
        }
        for (s, v) in self.sharp.iter().zip(sharp) {
            s.store(v, Ordering::Release);
        }
        self.changed();
    }

    /// As a preset's values (with the parameters' own): `<source>_to_<target>` (`lfo1_to_cutoff`),
    /// the depth in percent, for each assignment, and `lfoN_edges` (1, SHARP) where it is SHARP.
    pub fn values(&self) -> Vec<(String, f64)> {
        let mut out = Vec::new();
        for s in Source::all() {
            for a in self.assignments(s).into_iter().flatten() {
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
    /// preset that names none, nothing assigned and every EDGES SOFT. In the targets' order.
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
            let mut a: Vec<Assignment> = p.assignments(s).into_iter().flatten().collect();
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
        for (a, b) in self
            .slots
            .iter()
            .flatten()
            .zip(other.slots.iter().flatten())
        {
            a.store(b.load(Ordering::Acquire), Ordering::Release);
        }
        for (a, b) in self.sharp.iter().zip(&other.sharp) {
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

/// The sources' assignments as a session keeps them (JSON under the state's `lfo` key).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Saved {
    pub sources: Vec<SavedSource>,
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
}

/// An LFO as the audio thread last had it: its output (-1..1, after RISE and EDGES, as it
/// moves its knobs: the latest voice's where MONO is off), where it is in its cycle (0..1,
/// PHASE included: the scope's trace at `phase` is the waveform there), how long since the
/// note that last started it (s; the scope's time base starts there), RISE's gain then (0..1),
/// and whether it moves (assigned, running, not an ENV whose cycle is over).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    pub value: f32,
    pub phase: f32,
    pub since: f32,
    pub rise: f32,
    pub moving: bool,
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
        }
    }

    /// Whether any LFO moves a knob now (the rail key's faint glow).
    pub fn any_moving(&self) -> bool {
        self.moving.load(Ordering::Relaxed) != 0
    }

    pub fn publish(&self, r: &[Reading; LFOS]) {
        let mut moving = 0u8;
        for (k, r) in r.iter().enumerate() {
            self.value[k].store(r.value.to_bits(), Ordering::Relaxed);
            self.phase[k].store(r.phase.to_bits(), Ordering::Relaxed);
            self.since[k].store(r.since.to_bits(), Ordering::Relaxed);
            self.rise[k].store(r.rise.to_bits(), Ordering::Relaxed);
            moving |= u8::from(r.moving) << k;
        }
        self.moving.store(moving, Ordering::Relaxed);
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
    let p = phase - phase.floor();
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
        }
    }
}

impl Settings {
    /// Its cycles a second at `tempo` (BPM, 4/4 for bars), as a scope's time base needs.
    pub fn hz_at(&self, tempo: f64) -> f64 {
        match self.timed {
            None => self.hz,
            Some(Length::Beats(q)) => tempo / 60.0 / q,
            Some(Length::Bars(n)) => tempo / 60.0 / (4.0 * n),
        }
    }

    /// Its output `t` s after a note started it, before EDGES (the scope's trace: from PHASE,
    /// one cycle then held in ENV, faded in over RISE; FREE drawn as if it were at PHASE at
    /// the note).
    pub fn output_after(&self, t: f64, tempo: f64) -> f64 {
        let cycles = t.max(0.0) * self.hz_at(tempo);
        let c = if self.mode == Mode::Env {
            cycles.min(ENV_END)
        } else {
            cycles
        };
        wave(self.at, self.phase + c) * self.rise_gain(t)
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
        let x = wave(s.at, self.at) * gain;
        self.out = match s.edges {
            Edges::Sharp => x,
            Edges::Soft => {
                let tau = SOFT_EDGE / 9f64.ln();
                self.out + (x - self.out) * (1.0 - (-dt / tau).exp())
            }
        };
        self.out
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

    /// Whether it moves: not an ENV whose cycle is over (nor one never started).
    pub fn moving(&self, s: &Settings) -> bool {
        s.mode != Mode::Env || (self.started && self.cycles < 1.0)
    }

    /// The scope's reading of it.
    pub fn reading(&self, s: &Settings, moving: bool) -> Reading {
        Reading {
            value: self.out as f32,
            phase: self.at as f32,
            since: self.since as f32,
            rise: if s.rise > 0.0 && !self.started {
                0.0
            } else {
                s.rise_gain(self.since) as f32
            },
            moving,
        }
    }
}

/// The modulation as the engine is given it: each LFO's settings, every source's assignments,
/// each assigned knob's travel after its glide, and the computer's FREQUENCY law (the knobs'
/// travel through it). Part of the engine's `Controls`; none assigned, nothing more runs.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Lfos {
    pub lfo: [Settings; LFOS],
    /// Each source's assignments, [`Source::index`]'s order.
    pub to: [[Option<Assignment>; MOST]; SOURCES],
    /// Each target's travel (0..1), [`Target::ALL`]'s order: the assigned ones'; 0 the rest.
    pub base: [f32; TARGETS],
    pub law: FreqLaw,
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

/// The sources' values with the LFOs' at `lfos` (the macros', when there are any, their knobs').
pub fn source_values(lfos: &[f64; LFOS]) -> [f64; SOURCES] {
    std::array::from_fn(|i| match Source::of_index(i) {
        Some(Source::Lfo(n)) => lfos[n],
        _ => 0.0,
    })
}

impl Lfos {
    /// Whether any source moves a knob whose steps are heard ([`Target::fast`]).
    pub fn fast(&self) -> bool {
        self.to
            .iter()
            .flatten()
            .flatten()
            .any(|a| a.depth != 0.0 && a.target.fast())
    }

    /// Whether every source moving `t` is an LFO with EDGES SHARP (none moving it: false).
    pub fn sharp_on(&self, t: Target) -> bool {
        let mut any = false;
        for (i, to) in self.to.iter().enumerate() {
            if to.iter().flatten().any(|a| a.depth != 0.0 && a.target == t) {
                match Source::of_index(i) {
                    Some(Source::Lfo(n)) if self.lfo[n].edges == Edges::Sharp => any = true,
                    _ => return false,
                }
            }
        }
        any
    }

    /// Whether any source modulates anything (an assignment of a depth other than 0).
    pub fn active(&self) -> bool {
        self.to.iter().flatten().flatten().any(|a| a.depth != 0.0)
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
        let (sums, touched) = self.sums(|_, _| true, |i| shared[i]);
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
            |_, t| own_touched & (1 << t.index()) != 0,
            |i| if self.own(i) { own[i] } else { shared[i] },
        );
        let law = self.law.into();
        for t in Target::ALL {
            if own_touched & (1 << t.index()) == 0 {
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
    ) -> ([f64; TARGETS], u32) {
        let mut sums = [0.0f64; TARGETS];
        let mut touched = 0u32;
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
/// LFOs with MONO off), with their values at the latest step.
#[derive(Clone, Debug, Default)]
pub struct Bank {
    pub shared: [Gen; LFOS],
    pub voices: [[Gen; LFOS]; VOICES],
    pub shared_values: [f64; LFOS],
    pub voice_values: [[f64; LFOS]; VOICES],
    /// The voice the scope shows where MONO is off (the latest started).
    pub latest: usize,
}

impl Bank {
    /// A new note for the shared LFOs (a key pressed with none held, or any with RETRIGGER on).
    pub fn start_shared(&mut self, l: &Lfos) {
        for (g, s) in self.shared.iter_mut().zip(&l.lfo) {
            g.start(s);
        }
    }

    /// A new note for voice `k`'s own LFOs.
    pub fn start_voice(&mut self, l: &Lfos, k: usize) {
        if let Some(v) = self.voices.get_mut(k) {
            for (g, s) in v.iter_mut().zip(&l.lfo) {
                g.start(s);
            }
            self.latest = k;
        }
    }

    /// Every LFO's value now (`clock`, each timed FREE one's place on the host's clock, in
    /// cycles), its slew moved on by `dt` s; voices' own only for the LFOs with MONO off.
    pub fn step(&mut self, l: &Lfos, clock: &[Option<f64>; LFOS], dt: f64, voices: usize) {
        let n = voices.min(VOICES);
        for (k, (s, &c)) in l.lfo.iter().zip(clock).enumerate() {
            self.shared_values[k] = self.shared[k].value(s, c, dt);
            if !s.mono {
                for (values, gens) in self.voice_values[..n].iter_mut().zip(&mut self.voices[..n]) {
                    values[k] = gens[k].value(s, c, dt);
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
    /// moving where assigned and running.
    pub fn readings(&self, l: &Lfos) -> [Reading; LFOS] {
        std::array::from_fn(|k| {
            let s = &l.lfo[k];
            let g = if s.mono {
                &self.shared[k]
            } else {
                &self.voices[self.latest.min(VOICES - 1)][k]
            };
            let assigned = l.lfo_active(k);
            g.reading(s, assigned && g.moving(s))
        })
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
    for s in Source::all() {
        let i = s.index();
        l.to[i] = patch.assignments(s);
        for a in l.to[i].iter().flatten() {
            let t = a.target;
            l.base[t.index()] = t.travel(p, knob(t.param(p)), law);
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
/// RATE (Hz), TIMED and DIVISION, PHASE (degrees), RISE (s), MODE and MONO.
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
            rate: FloatParam::new(
                name("Rate"),
                DEFAULT_RATE,
                FloatRange::Skewed {
                    min: RATE_HZ.0,
                    max: RATE_HZ.1,
                    factor: 0.2,
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(Arc::new(rate_text))
            .with_string_to_value(Arc::new(|s| {
                let s = s.trim().trim_end_matches("Hz").trim();
                let v: f32 = s.parse().ok()?;
                v.is_finite().then(|| v.clamp(RATE_HZ.0, RATE_HZ.1))
            })),
            timed: BoolParam::new(name("Timed"), false),
            division: EnumParam::new(name("Division"), Division::Quarter),
            phase: FloatParam::new(
                name("Phase"),
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 360.0,
                },
            )
            .with_unit("\u{b0}")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
            // (A cube-root travel, so that short fades have room: 1 s near its middle.)
            rise: FloatParam::new(
                name("Rise"),
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: RISE_MOST,
                    factor: 1.0 / 3.0,
                },
            )
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

/// Each LFO's parameters' IDs, in [`LfoParams::sound_params`]' order.
pub const PARAM_IDS: [[&str; 10]; LFOS] = [
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
    ],
];

/// Each LFO's parameters' labels for the agents (docs/mcp.md), [`PARAM_IDS`]' order, and
/// each LFO's section.
pub const PARAM_LABELS: [[&str; 10]; LFOS] = [
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
        _ => &q.mono,
    })
}

/// An LFO's knob of `id` (SHAPE, RATE, PHASE, RISE), if it is one (`learn::knob_param`). They
/// do not glide: none is on the audio path (R-LFO; EDGES smooths what they step).
pub fn knob<'a>(p: &'a Ca72Params, id: &str) -> Option<&'a FloatParam> {
    let (n, k) = param_place(id)?;
    let q = p.lfo(n);
    match k {
        2 => Some(&q.shape),
        3 => Some(&q.rate),
        6 => Some(&q.phase),
        7 => Some(&q.rise),
        _ => None,
    }
}

/// Whether `id` is one of the LFOs' knobs (continuous, not gliding).
pub fn is_knob_id(id: &str) -> bool {
    param_place(id).is_some_and(|(_, k)| matches!(k, 2 | 3 | 6 | 7))
}

/// The IDs of each LFO's parameters, `lfoN_<part>`, in the order the host lists them.
pub fn param_ids() -> impl Iterator<Item = String> {
    const PARTS: [&str; 10] = [
        "waveform",
        "shape_mode",
        "shape",
        "rate",
        "timed",
        "division",
        "phase",
        "rise",
        "mode",
        "mono",
    ];
    (1..=LFOS).flat_map(|n| PARTS.iter().map(move |p| format!("lfo{n}_{p}")))
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
            p.assign(Source::Macro(0), Target::Drive, 0.1),
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
                    let q = t.param(&p);
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
}
