//! The plug-in's character (decisions.md, "ANALOG and SPREAD"; the same as that of the DAW's
//! instrument built on the same model):
//! ENTROPY, each voice its own parts and its oscillators' and cutoff's slow movements, and
//! SPREAD, where a voice sits in the stereo field. Neither is the instrument's; at 0 neither
//! does anything (the circuit as drawn).

/// ENTROPY's figures at 100 %: the standard deviations of a voice's own tolerances, and of
/// its oscillators' movements.
pub mod entropy {
    /// Each oscillator's tuning off its calibration, cents.
    pub const CENTS: f64 = 4.0;
    /// Each oscillator's slow drift, cents, wandering with this time constant, s.
    pub const DRIFT_CENTS: f64 = 3.0;
    pub const DRIFT_TAU: f64 = 4.0;
    /// Each oscillator's waver: noise under this frequency (Hz), cents.
    pub const WAVER_CENTS: f64 = 0.4;
    pub const WAVER_HZ: f64 = 30.0;
    /// The cutoff off its calibration, and its drift, octaves.
    pub const CUTOFF: f64 = 0.04;
    pub const CUTOFF_DRIFT: f64 = 0.015;
    /// A pot's (or the parts' behind it) offset, of its rotation (about 5 % of a time on
    /// an audio-taper pot).
    pub const KNOB: f64 = 0.012;
    /// Samples between drift steps (the drift moves over seconds).
    pub const STEP: u32 = 64;
}

/// The most oscillators and knobs a voice's character covers.
const OSCILLATORS: usize = 3;
const KNOBS: usize = 10;

/// Where SCATTER puts the voices POLY plays at full SPREAD, the player's choice of three (the
/// CA-74's, its decisions.md R30; here decisions.md R-STEREO); each voice as loud wherever it
/// sits ([`pan_gains`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Placement {
    /// Evenly spaced from edge to edge, however many voices play: taken left and right in
    /// turn, each side's from its edge, then halfway in, then the rest from the outside
    /// inwards; with an odd number the last in the centre. The first two voices (a pair, a
    /// chord's first notes) are at the edges, and the field is filled without a hole.
    #[default]
    Even,
    /// From the edges inwards, out to either side in turn, every voice from 60 % out and none
    /// in the centre: the widest, its voices together at the sides (the CA-74's R27's).
    Edges,
    /// From the centre outwards, by the golden ratio: the first voice in the centre and the
    /// next spread evenly about it, however many play: the narrowest.
    Centre,
}

/// [`Placement::Edges`]'s places, voice by voice.
const EDGES: [f64; 10] = [-1.0, 1.0, -0.9, 0.9, -0.8, 0.8, -0.7, 0.7, -0.6, 0.6];

impl Placement {
    /// Where voice `k` of `n` (VOICES) sits at full SPREAD, -1 (left) to 1 (right). A voice
    /// past `n` (one letting go after VOICES was lowered) takes the place of `k` less `n`.
    pub fn place(self, k: usize, n: usize) -> f64 {
        let n = n.clamp(2, EDGES.len());
        let k = k % n;
        match self {
            Placement::Edges => EDGES[k],
            Placement::Centre => 2.0 * (0.5 + k as f64 * 0.618_033_988_749_894_8).fract() - 1.0,
            Placement::Even => {
                let side = n / 2;
                if k >= 2 * side {
                    return 0.0;
                }
                // The `i`th place taken on its side, `j` places in from its edge.
                let i = k / 2;
                let middle = side / 2;
                let j = match i {
                    0 => 0,
                    1 => middle,
                    _ if i <= middle => i - 1,
                    _ => i,
                };
                let at = 1.0 - 2.0 * j as f64 / (n - 1) as f64;
                if k.is_multiple_of(2) { -at } else { at }
            }
        }
    }

    /// How far out to either side DOUBLE's pair of voice `k` of `n` sits at full SPREAD, 0 to 1
    /// (the CA-74's R41; decisions.md R-STEREO): with EDGES every pair at the edges; with EVEN
    /// and CENTER as far out as the voice's own place, its flat voice that far to the right and
    /// its sharp twin as far to the left, so each pair stays about the centre (a voice placed
    /// in the centre, its pair there too).
    pub fn pair(self, k: usize, n: usize) -> f64 {
        match self {
            Placement::Edges => 1.0,
            _ => self.place(k, n).abs(),
        }
    }

    /// Its index (0..3), as the voices' mix carries it and a preset saves it, and back.
    pub fn index(self) -> usize {
        match self {
            Placement::Even => 0,
            Placement::Edges => 1,
            Placement::Centre => 2,
        }
    }

    pub fn from_index(i: usize) -> Placement {
        match i {
            1 => Placement::Edges,
            2 => Placement::Centre,
            _ => Placement::Even,
        }
    }
}

/// A voice's character: its own tolerances, drawn once from its seed, its oscillators' and
/// cutoff's movements, and where it sits in the stereo field.
#[derive(Debug, Clone)]
pub struct Character {
    /// How many oscillators and knobs the instrument has.
    oscillators: usize,
    knobs: usize,
    /// Per unit of ENTROPY (standard normal draws): each oscillator's tuning, the cutoff,
    /// and the knobs (each instrument's own list).
    tune: [f64; OSCILLATORS],
    cutoff: f64,
    knob: [f64; KNOBS],
    /// The drifts (standard normal, moving: the oscillators', then the cutoff's), the
    /// wavers (in cents at full ENTROPY), the noise generator, samples to the next drift
    /// step.
    drift: [f64; OSCILLATORS + 1],
    waver: [f64; OSCILLATORS],
    rng: u64,
    count: u32,
    /// Which voice it is, for its place among the voices played (SPREAD's).
    voice: usize,
    /// The rate the movements' constants below are for, and they: the drift's step and its
    /// noise's share, the waver's pole and its noise's scale (worked out once a rate, not
    /// each sample: the same values).
    rate: f64,
    drift_ab: (f64, f64),
    waver_ws: (f64, f64),
}

impl Character {
    /// Voice `voice`'s character from `seed`, for an instrument of `oscillators` (at most
    /// 3) and `knobs` (at most 10) tolerances. The draws are made in a fixed order (the
    /// tunings, the cutoff, the knobs, the drifts' starts), so an instrument's voices are
    /// the same whenever they are made from the same seeds.
    pub fn new(seed: u64, voice: usize, oscillators: usize, knobs: usize) -> Character {
        let oscillators = oscillators.min(OSCILLATORS);
        let knobs = knobs.min(KNOBS);
        let mut c = Character {
            oscillators,
            knobs,
            tune: [0.0; OSCILLATORS],
            cutoff: 0.0,
            knob: [0.0; KNOBS],
            drift: [0.0; OSCILLATORS + 1],
            waver: [0.0; OSCILLATORS],
            // (Never zero, as xorshift needs.)
            rng: (seed ^ 0x5EED_0FA1_A106_0000) | 1,
            count: 0,
            voice,
            rate: f64::NAN,
            drift_ab: (0.0, 0.0),
            waver_ws: (0.0, 0.0),
        };
        for k in 0..oscillators {
            c.tune[k] = c.normal();
        }
        c.cutoff = c.normal();
        for k in 0..knobs {
            c.knob[k] = c.normal();
        }
        for k in 0..=oscillators {
            c.drift[k] = c.normal();
        }
        c
    }

    /// The next of the noise generator's numbers, uniform in -0.5..0.5 (xorshift64*).
    fn uniform(&mut self) -> f64 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        let x = self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (x >> 11) as f64 / (1u64 << 53) as f64 - 0.5
    }

    /// A standard normal draw (Box-Muller).
    fn normal(&mut self) -> f64 {
        let a = self.uniform() + 0.5;
        let b = self.uniform() + 0.5;
        libm::sqrt(-2.0 * libm::log(a.max(1e-300))) * libm::cos(std::f64::consts::TAU * b)
    }

    /// Knob `k`'s rotation `x` (0..1) with this voice's tolerance, by `amount` (0..1) of
    /// ENTROPY.
    pub fn knob(&self, k: usize, x: f64, amount: f64) -> f64 {
        (x + amount * entropy::KNOB * self.knob[k.min(self.knobs.max(1) - 1)]).clamp(0.0, 1.0)
    }

    /// One output sample's offsets, by `amount` (0..1) of ENTROPY, at `rate` Hz: each
    /// oscillator's off its calibration, cents (its tolerance, drift and waver), and the
    /// cutoff's, octaves (its tolerance and drift).
    pub fn offsets(&mut self, amount: f64, rate: f64) -> ([f64; OSCILLATORS], f64) {
        if rate.to_bits() != self.rate.to_bits() {
            self.rate = rate;
            let a = libm::exp(-f64::from(entropy::STEP) / (rate * entropy::DRIFT_TAU));
            self.drift_ab = (a, libm::sqrt(1.0 - a * a));
            let w = libm::exp(-std::f64::consts::TAU * entropy::WAVER_HZ / rate);
            self.waver_ws = (
                w,
                entropy::WAVER_CENTS * libm::sqrt(12.0 * (1.0 + w) / (1.0 - w)),
            );
        }
        if self.count == 0 {
            // The drifts' next step: each a mean-reverting random walk, its spread one.
            self.count = entropy::STEP;
            let (a, b) = self.drift_ab;
            for k in 0..=self.oscillators {
                // (Four uniforms: near enough normal, at a quarter of the cost.)
                let g = (self.uniform() + self.uniform() + self.uniform() + self.uniform())
                    * 3f64.sqrt();
                self.drift[k] = a * self.drift[k] + b * g;
            }
        }
        self.count -= 1;
        // The waver: white noise through a one-pole low-pass, scaled to its spread.
        let (w, scale) = self.waver_ws;
        for k in 0..self.oscillators {
            let x = self.uniform() * scale;
            self.waver[k] = w * self.waver[k] + (1.0 - w) * x;
        }
        let mut cents = [0.0; OSCILLATORS];
        for (k, c) in cents.iter_mut().enumerate().take(self.oscillators) {
            *c = amount
                * (entropy::CENTS * self.tune[k]
                    + entropy::DRIFT_CENTS * self.drift[k]
                    + self.waver[k]);
        }
        let d = self.drift[self.oscillators];
        (
            cents,
            amount * (entropy::CUTOFF * self.cutoff + entropy::CUTOFF_DRIFT * d),
        )
    }

    /// How far out DOUBLE's pair of this voice sits, as `placement` has it among `voices`
    /// ([`Placement::pair`]).
    pub fn pair(&self, placement: Placement, voices: usize) -> f64 {
        placement.pair(self.voice, voices)
    }

    /// The gains (left, right) that put the voice in its place, as `placement` has it among
    /// `voices`, by `spread` (0..1) of SPREAD ([`pan_gains`]; both whole at 0).
    pub fn gains(&self, spread: f64, placement: Placement, voices: usize) -> (f32, f32) {
        if spread > 0.0 {
            pan_gains(0.5 + 0.5 * spread * placement.place(self.voice, voices))
        } else {
            (1.0, 1.0)
        }
    }
}

/// The pan law: at `pan` (0 left, 0.5 centre, 1 right) each side's gain, of constant power
/// (a voice as loud wherever it sits, so the centre does not outweigh the sides), both
/// whole at the centre as with SPREAD at 0: √2 cos and √2 sin of a quarter turn's `pan` (the
/// CA-74's R27; the law before, the DAW's instrument's, both whole until the far side faded,
/// weighed the centre double).
pub fn pan_gains(pan: f64) -> (f32, f32) {
    let a = pan.clamp(0.0, 1.0) * std::f64::consts::FRAC_PI_2;
    (
        (std::f64::consts::SQRT_2 * a.cos()) as f32,
        (std::f64::consts::SQRT_2 * a.sin()) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every place carries a voice as loud as the centre does (constant power), and the
    /// centre is whole on both sides, as with SPREAD at 0.
    #[test]
    fn every_place_is_as_loud_as_the_centre() {
        for k in 0..=20 {
            let (l, r) = pan_gains(f64::from(k) / 20.0);
            let power = f64::from(l).powi(2) + f64::from(r).powi(2);
            assert!((power - 2.0).abs() < 1e-5, "at {k}/20: {power}");
        }
        let (l, r) = pan_gains(0.5);
        assert!((l - 1.0).abs() < 1e-6 && (r - 1.0).abs() < 1e-6, "{l}, {r}");
        assert_eq!(pan_gains(0.0).1, 0.0);
    }

    /// EVEN: however many voices play, their places are evenly spaced from edge to edge (each
    /// one once), the first two at the edges, then left and right in turn as mirror pairs, the
    /// second pair halfway in; with an odd number the last in the centre. Eight: 100, 43, 71
    /// and 14 % out.
    #[test]
    fn even_fills_the_field_evenly_from_its_edges() {
        let place = |k, n| Placement::Even.place(k, n);
        for n in 2..=10 {
            let places: Vec<f64> = (0..n).map(|k| place(k, n)).collect();
            let mut sorted = places.clone();
            sorted.sort_by(f64::total_cmp);
            for (i, p) in sorted.iter().enumerate() {
                let even = -1.0 + 2.0 * i as f64 / (n - 1) as f64;
                assert!((p - even).abs() < 1e-12, "{n} voices: {places:?}");
            }
            assert_eq!((places[0], places[1]), (-1.0, 1.0), "{n} voices");
            for k in 0..n / 2 {
                assert_eq!(places[2 * k], -places[2 * k + 1], "{n} voices: {places:?}");
                assert!(places[2 * k] < 0.0, "{n} voices: {places:?}");
            }
            if n % 2 == 1 {
                assert_eq!(places[n - 1], 0.0);
            }
            if n >= 6 {
                assert!((0.3..=0.7).contains(&places[3]), "{n} voices: {places:?}");
            }
            // A voice past the count takes an earlier one's place.
            assert_eq!(place(n + 1, n), place(1, n));
        }
        let eight: Vec<f64> = (0..8).map(|k| (place(k, 8) * 7.0).round()).collect();
        assert_eq!(eight, [-7.0, 7.0, -3.0, 3.0, -5.0, 5.0, -1.0, 1.0]);
    }

    /// EDGES: no voice in the centre, none nearer it than 60 %; voices taken in turn go to
    /// either side in turn, from the edges inwards (the CA-74's R27's).
    #[test]
    fn edges_fills_the_field_from_its_edges() {
        let places: Vec<f64> = (0..10).map(|k| Placement::Edges.place(k, 10)).collect();
        for (k, p) in places.iter().enumerate() {
            assert!(p.abs() >= 0.6, "voice {k} at {p}");
            assert_eq!(*p < 0.0, k % 2 == 0, "voice {k} on the wrong side");
        }
        assert!(places.windows(2).all(|w| w[0].abs() >= w[1].abs()));
    }

    /// CENTER: the first voice in the centre, the rest by the golden ratio about it, as the
    /// CA-74's are; never two in one place.
    #[test]
    fn centre_starts_in_the_centre_and_spreads_by_the_golden_ratio() {
        let places: Vec<f64> = (0..10).map(|k| Placement::Centre.place(k, 10)).collect();
        let heard = [
            0.0, -0.763932, 0.472136, -0.291796, 0.944272, 0.18034, -0.583592, 0.652476, -0.111456,
            -0.875388,
        ];
        for (p, h) in places.iter().zip(heard) {
            assert!((p - h).abs() < 1e-6, "{places:?}");
        }
        let mut sorted = places.clone();
        sorted.sort_by(f64::total_cmp);
        assert!(sorted.windows(2).all(|w| w[1] - w[0] > 0.05), "{sorted:?}");
    }

    #[test]
    fn a_placement_goes_by_its_index_and_back() {
        for p in [Placement::Even, Placement::Edges, Placement::Centre] {
            assert_eq!(Placement::from_index(p.index()), p);
        }
        assert_eq!(Placement::default(), Placement::Even);
    }
}
