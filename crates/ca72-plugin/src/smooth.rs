//! The voices follow the panel's continuous knobs (and its two wheels) by a glide, whatever
//! moved them: the editor (a drag, a scroll, a double click, a typed value), the host's
//! automation, offline as in real time, and a learned MIDI controller (issue #107;
//! decisions.md R-KNOB-GLIDE, R34).
//!
//! - **What glides**: the knobs of [`KNOBS`], each from where the voices have it to its
//!   parameter's new value, in a straight line, the voices' controls set again every [`STEP`]
//!   samples while one moves. A new value while it glides starts again from where it is.
//!   Selectors, switches, VOICES and everything else stay stepped.
//! - **How long**: [`GLIDE`] (30 ms), or the time since the knob's previous value when that is
//!   longer but no longer than the host's largest block (its automation reaching the plug-in
//!   once a block of more than 30 ms), never past [`LONGEST`]; a value after a longer rest
//!   glides over [`GLIDE`]. A learned controller's value glides over `learn::DEZIP` (10 ms)
//!   as before.
//! - **What lands**: a preset, Init, a session or a project's state, and the host's reset set
//!   the voices at once. A preset and Init mark the values they set ([`Lands`]); a value the
//!   audio thread finds marked lands instead of gliding. A session's state comes through
//!   `initialize` and `reset`, which land every knob ([`Smooth::land`]).
//! - **At rest**: with no knob moving, the voices have each knob's parameter exactly, and a
//!   block costs a comparison a knob; the renders are the same to the bit as without glides.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use nih_plug::prelude::*;

use crate::params::Ca72Params;

/// How long the voices take to follow a knob's new value, s, unless the time since its
/// previous one is longer (issue #107: a mouse sends a value about every 17 ms, and a host's
/// automation can arrive once a block of 1024 frames).
pub const GLIDE: f64 = 0.030;

/// The longest glide, s, however large the host's blocks.
pub const LONGEST: f64 = 0.2;

/// While a knob glides, the voices' controls are set again every this many samples.
pub const STEP: usize = 32;

/// How long a preset's marks wait for its values, s of audio processed after the audio thread
/// first sees the preset chosen (a host takes the editor's values at its next block).
pub const LAND_WAIT: f64 = 0.5;

/// The knobs that glide, by their parameters' IDs: every continuous control of the panel and
/// the strip (the learnable knobs, `learn::knob_param`'s), and the panel's PITCH and
/// MODULATION wheels, the per-note expression's four reaches (decisions.md R-MPE), and each
/// oscillator's CONTINUOUS shape (R-CONTINUOUS-WAVE), and the six macros (R-MACROS). Not the
/// arpeggiator's GATE and SWING ([`TIMING`]).
pub const KNOBS: [&str; 42] = [
    "tune",
    "glide",
    "mod_mix",
    "osc2_frequency",
    "osc3_frequency",
    "osc1_volume",
    "ext_volume",
    "osc2_volume",
    "noise_volume",
    "osc3_volume",
    "cutoff",
    "emphasis",
    "contour_amount",
    "filter_attack",
    "filter_decay",
    "filter_sustain",
    "loudness_attack",
    "loudness_decay",
    "loudness_sustain",
    "volume",
    "feedback",
    "entropy",
    "spread",
    "inner",
    "double",
    "drive",
    "level",
    "pitch_wheel",
    "mod_wheel",
    "pressure_mod",
    "pressure_filter",
    "timbre_mod",
    "timbre_filter",
    "osc1_shape",
    "osc2_shape",
    "osc3_shape",
    "macro1",
    "macro2",
    "macro3",
    "macro4",
    "macro5",
    "macro6",
];

/// The continuous knobs that do not glide: the arpeggiator's GATE and SWING, which set when its
/// notes start and stop, each note taking them as it is played (`crate::arp`), so that a change
/// has no level to step and a glide would only make it late (decisions.md R-KNOB-GLIDE); and the
/// velocity's CURVE and FLOOR, which shape the value each note takes as it is struck
/// (R-VELOCITY).
pub const TIMING: [&str; 4] = ["arp_gate", "arp_swing", "velocity_curve", "velocity_floor"];

/// Knob `k`'s parameter ([`KNOBS`]' order).
pub fn param(p: &Ca72Params, k: usize) -> Option<&FloatParam> {
    Some(match k {
        0 => &p.tune,
        1 => &p.glide,
        2 => &p.mod_mix,
        3 => &p.osc2_frequency,
        4 => &p.osc3_frequency,
        5 => &p.osc1_volume,
        6 => &p.ext_volume,
        7 => &p.osc2_volume,
        8 => &p.noise_volume,
        9 => &p.osc3_volume,
        10 => &p.cutoff,
        11 => &p.emphasis,
        12 => &p.contour_amount,
        13 => &p.filter_attack,
        14 => &p.filter_decay,
        15 => &p.filter_sustain,
        16 => &p.loudness_attack,
        17 => &p.loudness_decay,
        18 => &p.loudness_sustain,
        19 => &p.volume,
        20 => &p.feedback,
        21 => &p.entropy,
        22 => &p.spread,
        23 => &p.inner,
        24 => &p.double,
        25 => &p.drive,
        26 => &p.level,
        27 => &p.pitch_wheel,
        28 => &p.mod_wheel,
        29 => &p.pressure_mod,
        30 => &p.pressure_filter,
        31 => &p.timbre_mod,
        32 => &p.timbre_filter,
        33 => &p.osc1_shape,
        34 => &p.osc2_shape,
        35 => &p.osc3_shape,
        36..=41 => p.macro_param(k - 36),
        _ => return None,
    })
}

/// The knob of parameter `id`, if it glides.
pub fn index(id: &str) -> Option<usize> {
    KNOBS.iter().position(|k| *k == id)
}

/// The values a preset or Init is setting, marked so the voices take them at once (issue #107:
/// choosing a preset does not glide). The editor marks each value just before it sets it; the
/// audio thread lands a knob whose new value is marked, and takes the mark. A mark is the
/// preset's generation (high 32 bits) and the plain value's bits; 0 is none.
#[derive(Debug)]
pub struct Lands {
    generation: AtomicU32,
    marks: [AtomicU64; KNOBS.len()],
}

impl Default for Lands {
    fn default() -> Self {
        Lands {
            generation: AtomicU32::new(0),
            marks: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}

impl Lands {
    /// A preset (or Init) about to be set: its generation, for [`Lands::mark`].
    pub fn begin(&self) -> u32 {
        let g = self
            .generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        if g == 0 { self.begin() } else { g }
    }

    /// Parameter `id` about to be set to `normalized` by generation `g`'s preset: marked, if it
    /// is a knob that glides.
    pub fn mark(&self, g: u32, p: &Ca72Params, id: &str, normalized: f32) {
        if let Some(k) = index(id)
            && let Some(q) = param(p, k)
        {
            let plain = q.preview_plain(normalized);
            self.marks[k].store(
                u64::from(g) << 32 | u64::from(plain.to_bits()),
                Ordering::Release,
            );
        }
    }

    fn generation(&self) -> u32 {
        self.generation.load(Ordering::Acquire)
    }

    /// Whether knob `k`'s value `v` is generation `g`'s mark; if so the mark is taken.
    fn take(&self, k: usize, g: u32, v: f32) -> bool {
        let m = self.marks[k].load(Ordering::Acquire);
        let marked = m != 0 && (m >> 32) as u32 == g && m as u32 == v.to_bits();
        if marked {
            // (A newer mark written meanwhile stays.)
            let _ = self.marks[k].compare_exchange(m, 0, Ordering::AcqRel, Ordering::Relaxed);
        }
        marked
    }
}

/// A knob in the voices: gliding from `from` to `to` (plain values), `done` of `len` samples
/// gone; at `to` once `done` reaches `len`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Glide {
    from: f32,
    to: f32,
    done: u32,
    len: u32,
}

impl Glide {
    fn at(v: f32) -> Glide {
        Glide {
            from: v,
            to: v,
            done: 0,
            len: 0,
        }
    }

    fn moving(&self) -> bool {
        self.done < self.len
    }

    fn now(&self) -> f32 {
        if self.moving() {
            self.from + (self.to - self.from) * (self.done as f32 / self.len as f32)
        } else {
            self.to
        }
    }
}

/// Where the voices have each knob of [`KNOBS`], and its glide (the audio thread's).
#[derive(Clone, Debug)]
pub struct Smooth {
    knobs: [Glide; KNOBS.len()],
    /// Each knob's parameter's address, to know it by in `Ca72Params::controls_with`.
    addresses: [usize; KNOBS.len()],
    /// The samples processed, and when each knob last took a new value.
    clock: u64,
    changed: [u64; KNOBS.len()],
    moving: usize,
    /// Lengths in samples: [`GLIDE`], a learned controller's, the longest, and [`LAND_WAIT`].
    glide: u32,
    learned: u32,
    longest: u32,
    wait: u64,
    /// The presets' generation last seen, and the samples since.
    generation: u32,
    since_generation: u64,
}

impl Default for Smooth {
    fn default() -> Self {
        Smooth {
            knobs: [Glide::at(0.0); KNOBS.len()],
            addresses: [0; KNOBS.len()],
            clock: 0,
            changed: [0; KNOBS.len()],
            moving: 0,
            glide: 1,
            learned: 1,
            longest: 1,
            wait: 0,
            generation: 0,
            since_generation: 0,
        }
    }
}

impl Smooth {
    /// The glides' lengths at `rate` Hz, the host's largest block `block` frames; every knob
    /// at its parameter's value.
    pub fn prepare(&mut self, rate: f64, block: usize, p: &Ca72Params) {
        let samples = |s: f64| (s * rate).round().max(1.0) as u32;
        self.glide = samples(GLIDE);
        self.learned = samples(crate::learn::DEZIP);
        self.longest = (block.min(u32::MAX as usize) as u32)
            .min(samples(LONGEST))
            .max(self.glide);
        self.wait = u64::from(samples(LAND_WAIT));
        self.land(p);
    }

    /// Every knob at its parameter's value at once, nothing gliding (a session's state, the
    /// host's reset).
    pub fn land(&mut self, p: &Ca72Params) {
        for k in 0..KNOBS.len() {
            if let Some(q) = param(p, k) {
                self.knobs[k] = Glide::at(q.value());
                self.addresses[k] = std::ptr::from_ref(q) as usize;
            }
            self.changed[k] = self.clock;
        }
        self.moving = 0;
        self.generation = p.lands.generation();
        self.since_generation = 0;
    }

    /// Whether a knob glides.
    pub fn moving(&self) -> bool {
        self.moving > 0
    }

    /// Each knob whose parameter has a new value glides there from where the voices have it, or
    /// lands there if a preset marked it: at a block's start, after a learned controller's
    /// event, and every [`STEP`] samples while one glides.
    pub fn follow(&mut self, p: &Ca72Params) {
        let g = p.lands.generation();
        if g != self.generation {
            self.generation = g;
            self.since_generation = 0;
        }
        let lands = self.since_generation <= self.wait;
        for k in 0..KNOBS.len() {
            let Some(q) = param(p, k) else { continue };
            let v = q.value();
            if lands && p.lands.take(k, g, v) {
                if self.knobs[k].moving() {
                    self.moving -= 1;
                }
                self.knobs[k] = Glide::at(v);
                self.changed[k] = self.clock;
            } else if v != self.knobs[k].to {
                // (The time since its previous value, if no longer than a host's block.)
                let gap = self.clock - self.changed[k];
                let len = if gap <= u64::from(self.longest) {
                    (gap as u32).max(self.glide)
                } else {
                    self.glide
                };
                self.start(k, v, len);
            }
        }
    }

    /// Knob `k` set by a learned controller (to its parameter's value now): it glides there
    /// over `learn::DEZIP` (decisions.md R34).
    pub fn learned(&mut self, k: usize, p: &Ca72Params) {
        if let Some(q) = param(p, k) {
            self.start(k, q.value(), self.learned);
        }
    }

    fn start(&mut self, k: usize, to: f32, len: u32) {
        let g = &mut self.knobs[k];
        let from = g.now();
        let was = g.moving();
        *g = if from == to {
            Glide::at(to)
        } else {
            Glide {
                from,
                to,
                done: 0,
                len: len.max(1),
            }
        };
        match (was, g.moving()) {
            (false, true) => self.moving += 1,
            (true, false) => self.moving -= 1,
            _ => {}
        }
        self.changed[k] = self.clock;
    }

    /// `n` samples on.
    pub fn advance(&mut self, n: usize) {
        self.clock += n as u64;
        self.since_generation = self.since_generation.saturating_add(n as u64);
        if self.moving == 0 {
            return;
        }
        for g in &mut self.knobs {
            if g.moving() {
                g.done = g.done.saturating_add(n as u32).min(g.len);
                if !g.moving() {
                    self.moving -= 1;
                }
            }
        }
    }

    /// Where the voices have `knob` now: a gliding knob's place, a knob at rest as its
    /// parameter was when last followed; a parameter that does not glide, its value.
    pub fn value(&self, knob: &FloatParam) -> f32 {
        let at = std::ptr::from_ref(knob) as usize;
        match self.addresses.iter().position(|a| *a == at) {
            Some(k) => self.knobs[k].now(),
            None => knob.value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ca72;
    use nih_plug::context::process::TestProcessContext;
    use std::sync::Arc;

    fn ready(p: &Ca72Params) -> Smooth {
        let mut s = Smooth::default();
        s.prepare(48_000.0, 256, p);
        s
    }

    /// The parameters, and a host's way to set them as its automation does.
    fn hosted() -> (Arc<Ca72Params>, TestProcessContext<Ca72>) {
        let p = Arc::new(Ca72Params::default());
        let as_params: Arc<dyn Params> = p.clone();
        let c = TestProcessContext::new(as_params, 48_000.0, ProcessMode::Realtime);
        (p, c)
    }

    /// Set as the host's automation sets it.
    fn set(c: &TestProcessContext<Ca72>, q: &FloatParam, plain: f32) {
        assert!(c.automate(q.as_ptr(), q.preview_normalized(plain)));
    }

    /// Every knob glides is a continuous FloatParam, each once; the learnable knobs are all
    /// among them.
    #[test]
    fn the_knobs_are_the_panels_continuous_controls() {
        let p = Ca72Params::default();
        for (k, id) in KNOBS.iter().enumerate() {
            let q = param(&p, k).expect("a knob");
            let named = p
                .param_map()
                .into_iter()
                .find(|(i, ..)| i == id)
                .map(|(_, ptr, _)| ptr);
            assert_eq!(named, Some(q.as_ptr()), "{id}");
            assert_eq!(index(id), Some(k));
        }
        assert!(param(&p, KNOBS.len()).is_none());
        for i in 0..crate::learn::LEARNABLE.len() {
            let id = crate::learn::LEARNABLE[i].id;
            let unglided = TIMING.contains(&id) || crate::lfo::is_knob_id(id);
            if crate::learn::knob_param(&p, i).is_some() && !unglided {
                assert!(index(id).is_some(), "{id}");
            }
        }
        // Every continuous control the editor turns, but the arpeggiator's timing: a knob
        // added later is not left out unseen.
        for c in ca72_panel::controls::CONTROLS.iter() {
            // (Nor the LFOs' knobs: not on the audio path, decisions.md R-LFO.)
            let timing = TIMING.contains(&c.param) || crate::lfo::is_knob_id(c.param);
            assert_eq!(
                index(c.param).is_some(),
                c.kind.is_continuous() && !timing,
                "{}",
                c.param
            );
        }
        for id in [
            "voices",
            "osc1_range",
            "quality",
            "midi_bend_range",
            "poly",
            "note_priority",
            "arp_rate",
            "arp_octaves",
            "arp_gate",
            "arp_swing",
            "osc2_timed",
            "osc2_division",
            "osc3_timed",
            "glide_range",
            "osc3_division",
        ] {
            assert_eq!(index(id), None, "{id} does not glide");
        }
    }

    /// A new value glides from where the voices have it over 30 ms, in a straight line; a value
    /// mid-glide starts again from there; the voices end on the parameter's value exactly.
    #[test]
    fn a_new_value_glides_over_thirty_ms_and_lands_exactly() {
        let (p, c) = hosted();
        let mut s = ready(&p);
        let q = &p.cutoff;
        assert!(!s.moving());
        assert_eq!(s.value(q), q.value());
        s.advance(48_000);
        set(&c, q, 4.0);
        s.follow(&p);
        assert!(s.moving());
        assert_eq!(s.value(q), 0.0);
        s.advance(720);
        assert!((s.value(q) - 2.0).abs() < 1e-5, "{}", s.value(q));
        // (Another value: from here, 30 ms again.)
        set(&c, q, -2.0);
        s.follow(&p);
        assert!((s.value(q) - 2.0).abs() < 1e-5);
        s.advance(720);
        assert!((s.value(q) - 0.0).abs() < 1e-5, "{}", s.value(q));
        s.advance(720);
        assert!(!s.moving());
        assert_eq!(s.value(q).to_bits(), q.value().to_bits());
        // A parameter that does not glide reads as it is.
        assert_eq!(s.value(&p.midi_bend_range), p.midi_bend_range.value());
    }

    /// Values arriving further apart than 30 ms but no further than the host's largest block (its
    /// automation once a large block) glide over the time between them; one after a longer
    /// rest, over 30 ms.
    #[test]
    fn values_further_apart_glide_over_the_gap_up_to_the_largest_block() {
        let (p, c) = hosted();
        let mut s = Smooth::default();
        s.prepare(48_000.0, 2048, &p);
        let q = &p.emphasis;
        s.advance(48_000);
        set(&c, q, 1.0);
        s.follow(&p);
        // (After a rest: 30 ms.)
        s.advance(1440);
        assert!(!s.moving());
        // 2048 frames later: over 2048 frames.
        s.advance(2048 - 1440);
        set(&c, q, 3.0);
        s.follow(&p);
        s.advance(1024);
        assert!((s.value(q) - 2.0).abs() < 1e-5, "{}", s.value(q));
        s.advance(1024);
        assert!(!s.moving());
        // Further apart than the largest block: over 30 ms.
        s.advance(3000);
        set(&c, q, 5.0);
        s.follow(&p);
        s.advance(1439);
        assert!(s.moving());
        s.advance(1);
        assert!(!s.moving());
        assert_eq!(s.value(q), 5.0);
    }

    /// A value a preset marked lands at once, and the mark is taken; an unmarked value, or one
    /// of an earlier preset's generation, or one arriving after the marks have waited their
    /// time, glides.
    #[test]
    fn a_presets_marked_values_land_at_once() {
        let (p, c) = hosted();
        let mut s = ready(&p);
        let g = p.lands.begin();
        p.lands
            .mark(g, &p, "cutoff", p.cutoff.preview_normalized(3.0));
        p.lands.mark(g, &p, "osc1_range", 1.0);
        set(&c, &p.cutoff, 3.0);
        set(&c, &p.emphasis, 7.0);
        s.follow(&p);
        assert_eq!(s.value(&p.cutoff), 3.0, "landed");
        assert_eq!(s.value(&p.emphasis), 0.0, "glides");
        assert!(s.moving());
        // The mark is taken: the same value set again later glides.
        s.advance(4800);
        set(&c, &p.cutoff, 1.0);
        s.follow(&p);
        s.advance(1440);
        set(&c, &p.cutoff, 3.0);
        s.follow(&p);
        assert!(s.value(&p.cutoff) < 3.0);
        // An earlier generation's mark does not land a value.
        let old = p.lands.begin();
        p.lands
            .mark(old, &p, "volume", p.volume.preview_normalized(2.0));
        let _new = p.lands.begin();
        s.advance(4800);
        set(&c, &p.volume, 2.0);
        s.follow(&p);
        assert!(s.value(&p.volume) != 2.0);
        // A mark past its wait does not land.
        let g = p.lands.begin();
        p.lands.mark(g, &p, "tune", p.tune.preview_normalized(1.0));
        s.follow(&p);
        s.advance(48_000);
        set(&c, &p.tune, 1.0);
        s.follow(&p);
        assert!(s.value(&p.tune) != 1.0);
        // Seen before the audio thread notices the generation (a weakly ordered processor):
        // the glide that began lands at the next step.
        s.advance(48_000);
        set(&c, &p.inner, 50.0);
        s.follow(&p);
        assert!(s.value(&p.inner) < 50.0);
        let g = p.lands.begin();
        p.lands
            .mark(g, &p, "inner", p.inner.preview_normalized(50.0));
        s.follow(&p);
        assert_eq!(s.value(&p.inner), 50.0);
        // Landing every knob: nothing moves.
        s.land(&p);
        assert!(!s.moving());
    }

    /// A learned controller's value glides over 10 ms (decisions.md R34).
    #[test]
    fn a_learned_value_glides_over_ten_ms() {
        let (p, c) = hosted();
        let mut s = ready(&p);
        let k = index("cutoff").expect("a knob");
        set(&c, &p.cutoff, 4.0);
        s.learned(k, &p);
        s.follow(&p);
        s.advance(240);
        assert!((s.value(&p.cutoff) - 2.0).abs() < 1e-5);
        s.advance(240);
        assert!(!s.moving());
        assert_eq!(s.value(&p.cutoff), 4.0);
    }
}
