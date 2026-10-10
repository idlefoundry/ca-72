//! Potato mode's voice (decisions.md R31): the instrument's panel played by a model far
//! cheaper than the circuit's, for computers the circuit is too heavy for. Not derived from
//! the circuit but fitted to the circuit model's measured behaviour ([`laws`]; the plug-in's
//! `tests/potato_reference.rs` measures both): band-limited oscillators (polyBLEP) at the
//! circuit's levels and shapes, the mixer's channels through their loaded pots, the input
//! pair's compression, one ladder of four one-pole stages solved without delay around the
//! loop (its gain the emphasis, its own limiter setting how loud it rings), contours of RC
//! segments with the circuit's time law and trigger, and the VCA's measured law. The audio
//! path runs in single precision at the voice's rate; what the panel and the contours set
//! (the oscillators' increments, the ladder's coefficient) is worked out every
//! [`CONTROL_EVERY`] samples and followed between.
//!
//! The keys are the instrument's: lowest-note priority, single triggering (a key pressed while
//! another is held does not start the contours again), the contours re-armed once the trigger
//! contact has been open 12 ms; GLIDE moves the pitch as the circuit's hold does. Nothing here
//! allocates once made.

use crate::tuning::FREQ_CENTRE;
#[cfg(test)]
use crate::voice::OscPanel;
use crate::voice::{ContourKnobs, Panel, Waveform, audio_taper, glide_r};

/// Samples between the control laws' updates.
pub const CONTROL_EVERY: usize = 16;

/// The laws, fitted to the circuit model's measurements at 48 kHz (`tests/potato_reference.rs`;
/// decisions.md R31).
pub mod laws {
    /// Key 69 on 8' at TUNE's centre, Hz (the circuit's: 439.78, within 1.4 cents of equal
    /// temperament from key 24 to 108).
    pub const A4: f64 = 440.0;
    /// FREQUENCY (oscillators 2 and 3) from its centre, cents: a cubic in the knob (0..1),
    /// within 0.07 cents of the measured.
    pub const OSC2_CENTS: [f64; 4] = [-855.506, 1753.088, -94.505, 19.578];
    pub const OSC3_CENTS: [f64; 4] = [-862.370, 1806.991, -206.463, 83.185];
    /// TUNE at either end, cents.
    pub const TUNE_CENTS: f64 = 267.44;
    /// Oscillator 3 with OSC. 3 CONTROL off: as this key held on 8' (232.7 Hz) at FREQUENCY's
    /// centre; FREQUENCY then, from its centre, cents, a quintic in the knob (its summing node
    /// without the keys' input: 69 semitones end to end, within 0.2 cents of the measured);
    /// TUNE moves it not at all.
    pub const OSC3_FREE_KEY: f64 = 57.97;
    pub const OSC3_FREE_CENTS: [f64; 6] = [-3797.431, 9228.8, -4699.331, 3541.6, -1559.71, 402.07];

    /// Each waveform's level against the sawtooth's (the bus's unit: a sawtooth at VOLUME 10
    /// is 1 at its peak), its channel's source resistance (ohm, for the pot's loading) and the
    /// rectangles' duty cycles.
    pub const TRIANGLE: f32 = 0.847;
    pub const SAWTOOTH: f32 = 1.0;
    pub const REVERSE_SAWTOOTH: f32 = 1.0;
    pub const SQUARE: f32 = 1.032;
    pub const WIDE: f32 = 1.046;
    pub const NARROW: f32 = 1.059;
    pub const DUTY: [f32; 3] = [0.484, 0.30, 0.179];
    /// The shark tooth: the triangle inverted (its peak at the sawtooth's reset) and the
    /// falling sawtooth, as R030 and R031 mix them.
    pub const SHARK_TRIANGLE: f32 = 0.677;
    pub const SHARK_SAWTOOTH: f32 = 0.182;
    pub const SOURCE_OHMS: [f64; 4] = [687.0, 2246.0, 3500.0, 8790.0];
    /// Oscillator 3's channel against the others' (its sawtooth through the reverse
    /// sawtooth's stage).
    pub const OSC3_LEVEL: f32 = 0.894;
    /// The high-pass the output shows (the waveforms' droop), Hz.
    pub const DROOP_HZ: f64 = 12.0;
    /// The VCA's control feeding through to the output, its thump as the loudness contour
    /// moves: the VCA's gain through a high-pass and a low-pass (time constants, s), times
    /// this (fitted to the circuit's output with nothing sounding, four contours at once,
    /// within 14 dB of the thump; most of it under 5 Hz).
    pub const THUMP: f32 = 0.0385;
    pub const THUMP_TIMES: (f64, f64) = (0.11, 0.035);

    /// The noise: white's and pink's level (on their generators' units), their channels'
    /// source resistances (pink's through R50), and each one's band (a high-pass and a
    /// low-pass, Hz).
    pub const WHITE: f32 = 0.576;
    pub const PINK: f32 = 0.437;
    pub const NOISE_OHMS: (f64, f64) = (1e3, 26e3);
    pub const WHITE_BAND: (f64, f64) = (60.0, 9_500.0);
    pub const PINK_BAND: (f64, f64) = (40.0, 6_500.0);
    /// The white's low-pass at each of `NOISE_TOP_AT`'s rates, Hz: the fit at 48 kHz takes in
    /// the circuit's own top near Nyquist; at 88.2 kHz and over its noise reaches higher.
    pub const NOISE_TOP_AT: [f64; 2] = [48_000.0, 88_200.0];
    pub const WHITE_TOP: [f64; 2] = [9_500.0, 19_000.0];
    pub const PINK_TOP: [f64; 2] = [6_500.0, 9_000.0];

    /// The ladder's input pair, a tanh on the input less the loop's feedback: the bus in its
    /// units (fitted to the compression of one to three sawtooths at full VOLUME, -0.3 to
    /// -1.1 dB), off its centre by `BIAS` (its second harmonic: a triangle's from -37 dB at
    /// VOLUME 1 to -26 dB at 10).
    pub const DRIVE: f32 = 0.60;
    pub const BIAS: f32 = 0.28;
    /// The output from the ladder's (at the VCA's gain 1: the loudness contour at 4.455 V).
    pub const OUT: f32 = 0.724;

    /// CUTOFF: log2 of the self-oscillating ladder's pitch, Hz, at the knob's 0, 0.05, ...,
    /// 1 (measured from 0.15; below, continued at the slope there).
    pub const CUTOFF_LOG2: [f32; 21] = [
        5.155, 5.583, 6.011, 6.436, 6.864, 7.315, 7.783, 8.262, 8.750, 9.247, 9.745, 10.247,
        10.749, 11.252, 11.752, 12.252, 12.747, 13.234, 13.711, 14.103, 14.271,
    ];
    /// KEYBOARD CONTROL 1 and 2: octaves of the cutoff an octave of the keys, from F (41).
    pub const KEYBOARD_CONTROL: [f64; 2] = [0.339, 0.672];
    pub const TRACKING_FROM: f64 = 41.0;
    /// AMOUNT OF CONTOUR: octaves a volt of the filter contour above its rest at 10, and the
    /// share at 0, 2.5, 5, 7.5 and 10.
    pub const CONTOUR_OCTAVES_PER_VOLT: f64 = 2.18;
    pub const AMOUNT: [f64; 5] = [0.0, 0.23, 0.47, 0.72, 1.0];
    /// EMPHASIS: the loop's gain (4 rings) at these knob positions, for cutoffs of 60, 87,
    /// 228, 912 and 3731 Hz (log2 below; between them interpolated, beyond them held): fitted to
    /// the circuit's responses to noise at EMPHASIS up to 7 (within 0.2 to 0.6 dB) and its
    /// ringing thresholds (7.9 near 220 Hz, 7.44 near 900, 7.35 near 3.7 kHz; at 10 it rings
    /// down to about 87 Hz and not below).
    /// Past the threshold the gain follows R14's law, 11.84 / (R14 + 1.58K) (R14 in K), to
    /// 7.5 at 10, scaled by `RING_GAIN`.
    pub const EMPHASIS_AT: [f64; 11] = [
        0.0, 0.25, 0.5, 0.7, 0.72, 0.735, 0.744, 0.79, 0.85, 0.9, 1.0,
    ];
    pub const LOOP_AT_LOG2: [f64; 10] = [
        5.9, 6.44, 6.864, 7.783, 8.75, 9.745, 10.749, 11.752, 12.747, 13.711,
    ];
    pub const LOOP: [[f64; 11]; 10] = [
        [
            0.146, 0.502, 1.395, 2.879, 3.025, 3.134, 3.2, 3.4, 3.5, 3.6, 3.8,
        ],
        [
            0.154, 0.528, 1.468, 3.031, 3.184, 3.299, 3.368, 3.72, 3.95, 4.05, 4.212,
        ],
        [
            0.158, 0.54, 1.502, 2.79, 2.955, 3.079, 3.445, 3.805, 4.1, 4.229, 4.429,
        ],
        [
            0.166, 0.567, 1.575, 3.306, 3.509, 3.678, 3.693, 4.13, 4.407, 4.652, 5.229,
        ],
        [
            0.201, 0.616, 1.675, 3.595, 3.815, 3.962, 3.964, 4.313, 5.043, 5.669, 6.713,
        ],
        [
            0.238, 0.668, 1.779, 3.749, 3.986, 4.079, 4.127, 4.651, 5.436, 6.112, 7.695,
        ],
        [
            0.244, 0.676, 1.798, 3.82, 4.033, 4.121, 4.127, 4.707, 5.502, 6.186, 8.055,
        ],
        [
            0.248, 0.68, 1.81, 3.849, 4.06, 4.123, 4.127, 4.737, 5.536, 6.225, 8.29,
        ],
        [
            0.248, 0.68, 1.811, 3.845, 4.061, 4.125, 4.127, 4.74, 5.54, 6.23, 8.267,
        ],
        [
            0.248, 0.68, 1.811, 4.05, 4.123, 4.125, 4.127, 4.74, 5.54, 6.23, 8.267,
        ],
    ];
    pub const RING_GAIN: f64 = 2.5;
    /// The ladder's corner above the ring's pitch (the table above), octaves, below the ringing
    /// threshold: fitted to the circuit's responses at CUTOFF 0.3, 0.5 and 0.7 (220, 858 and
    /// 3454 Hz rings); it closes to nothing at the threshold, as the ring's own saturation
    /// pulls the circuit's corner down to its pitch.
    pub const CORNER: [f64; 3] = [0.048, 0.088, 0.111];
    pub const CORNER_AT_LOG2: [f64; 3] = [7.78, 9.74, 11.75];
    /// Past the threshold, where `CORNER` has closed, the ring's own pitch raised this much,
    /// octaves (the input pair's knee in the loop pulls the ring flat by about 1.4 % from 300
    /// Hz to 4 kHz; the circuit's rings measured).
    pub const RING_RAISE: [f64; 5] = [0.001, 0.011, 0.021, 0.019, 0.01];
    pub const RING_RAISE_AT_LOG2: [f64; 5] = [6.44, 7.78, 8.75, 11.75, 12.75];

    /// The contours: their rest and peak, V (filter, loudness), the level an attack stops at
    /// and where its RC curve aims, as shares of the peak's rise from rest, and the
    /// comparator's lag past it, s.
    pub const REST: [f64; 2] = [0.088, -0.520];
    pub const PEAK: [f64; 2] = [4.970, 5.418];
    pub const ATTACK_STOP: [f64; 2] = [0.9017, 0.977];
    pub const ATTACK_AIM: [f64; 2] = [1.779, 1.451];
    pub const ATTACK_LAG: f64 = 0.000_12;
    /// ATTACK's and DECAY's time constant: (the 1M audio pot + this, ohm: ATTACK's, DECAY's)
    /// times C, F.
    pub const TIME_OHMS: (f64, f64) = (100.0, 5.0);
    pub const TIME_FARADS: f64 = 10.1e-6;
    /// SUSTAIN's level, V: a cubic in the knob (filter, loudness).
    pub const SUSTAIN: [[f64; 4]; 2] = [
        [0.0436, 3.7616, -2.1722, 2.1998],
        [-0.4724, 4.9607, -2.3957, 2.3616],
    ];
    /// Where a release falls to, V (filter, loudness), with the DECAY switch on (at DECAY's
    /// time).
    pub const RELEASED: [f64; 2] = [0.144, -0.495];
    /// With the switch off, by DECAY at each of `RELEASE_OFF_AT`: where it falls to, V, and
    /// its time constant, s (the capacitor through DECAY's pot and the switch's resistor at
    /// once: quick and to `RELEASED`'s level at 0, lower and slower as the pot turns up).
    pub const RELEASE_OFF_AT: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    pub const RELEASED_OFF: [[f64; 11]; 2] = [
        [
            0.1423, -0.0016, -0.0591, -0.0964, -0.1254, -0.1503, -0.1728, -0.1936, -0.2122,
            -0.2279, -0.2401,
        ],
        [
            -0.4966, -0.6418, -0.6997, -0.7373, -0.7667, -0.7918, -0.8145, -0.8355, -0.8543,
            -0.8701, -0.8825,
        ],
    ];
    pub const RELEASE_OFF: [[f64; 11]; 2] = [
        [
            0.0085, 0.0312, 0.0382, 0.041, 0.0421, 0.0427, 0.0431, 0.0434, 0.0437, 0.044, 0.0441,
        ],
        [
            0.0059, 0.0214, 0.0263, 0.0289, 0.0304, 0.0315, 0.0323, 0.0327, 0.0332, 0.0335, 0.0338,
        ],
    ];
    /// The trigger: the contours start this long after a key goes down and fall this long
    /// after the last is let go, s; a key pressed within `REARM` of the last let go does not
    /// start them again.
    pub const TRIGGER_ON: f64 = 0.0078;
    pub const TRIGGER_OFF: f64 = 0.015;
    pub const REARM: f64 = 0.012;

    /// The VCA: its gain (1 at the loudness contour's 4.455 V) at these volts; past the last,
    /// as there.
    pub const VCA_VOLTS: [f64; 18] = [
        0.3, 0.4, 0.7, 1.0, 1.3, 1.6, 1.9, 2.2, 2.5, 2.8, 3.1, 3.4, 3.7, 4.0, 4.3, 4.6, 4.9, 5.2,
    ];
    pub const VCA_GAIN: [f64; 18] = [
        0.0, 0.0227, 0.0923, 0.1716, 0.2461, 0.3244, 0.4027, 0.4804, 0.56, 0.6407, 0.7169, 0.7958,
        0.8764, 0.9554, 0.9984, 0.9993, 0.996, 0.9914,
    ];

    /// GLIDE: the pitch slides at a constant rate, semitones a second times its pot's ohms,
    /// rising; falling, this share of it. It holds while no key is down (the keyboard's
    /// track-and-hold).
    pub const GLIDE_RATE: f64 = 1.02e8;
    pub const GLIDE_FALLING: f64 = 0.46;
    /// The keys the keyboard circuit plays (F to C, MIDI); beyond them the plug-in adds the
    /// rest of the scale after it, moving through GLIDE's resistance and the hold's 1 uF alone
    /// (an RC, not the slide), while a key is down (voice.rs's `beyond`).
    pub const KEYBOARD_KEYS: (f64, f64) = (41.0, 84.0);
    pub const GLIDE_FARADS: f64 = 1e-6;
    /// Where a fresh voice's pitch rests, as a key: the keyboard's output before any key.
    pub const PITCH_AT_REST: f64 = 53.75;

    /// The modulation at the MODULATION wheel fully forward, from oscillator 3: semitones of
    /// the oscillators at its waveform's swing (each from -1 to 1) and its centre against the
    /// triangle's, DC-coupled as the circuit's line is: the triangle, reverse sawtooth and
    /// sawtooth about their centres, the rectangles one-sided and inverted against their
    /// sound (up while the rectangle is low: 0.28 to 18.9 semitones with the line's offset);
    /// the cutoff's octaves `FILTER_PER_SEMITONE` of the oscillators' semitones; the line's
    /// offset; from the noise (MODULATION MIX at 10): their RMS and offset.
    pub const MOD_SWING: [f64; 4] = [-8.93, -8.85, -8.81, -9.32];
    pub const MOD_CENTRE: [f64; 4] = [0.0, -1.47, -0.69, 8.54];
    pub const FILTER_PER_SEMITONE: f64 = 0.2632;
    pub const MOD_OFFSET: (f64, f64) = (1.06, 0.29);
    pub const NOISE_SEMITONES_RMS: f64 = 2.80;
    pub const NOISE_OCTAVES_RMS: f64 = 0.679;
    pub const NOISE_OFFSET: (f64, f64) = (0.25, 0.062);
    /// The swing's share against the wheel's position (a cubic), and the offsets' (the wheel
    /// to this power; the line's loading by its rheostat).
    pub const WHEEL: [f64; 3] = [0.828, 0.584, -0.412];
    pub const OFFSET_POWER: f64 = 0.6;

    /// EXTERNAL INPUT: its gain to the output, `EXT_GAIN t / (1 + EXT_LOAD t (1 - t))` for
    /// VOLUME's audio law t (the 1M pot loaded by the preamplifier; within 0.3 %), the level
    /// its preamplifier clips at there, and the OVERLOAD lamp's onset and full brightness
    /// there.
    pub const EXT_GAIN: f64 = 148.1;
    pub const EXT_LOAD: f64 = 10.16;
    pub const EXT_CLIP: f64 = 0.81;
    pub const OVERLOAD: (f64, f64) = (0.24, 0.36);
    /// EXTERNAL INPUT's coupling at the jack, a high-pass, Hz (the circuit's input falls 9 dB
    /// more than the output's droop at 5 Hz, 4.5 at 8).
    pub const EXT_COUPLING_HZ: f64 = 10.5;

    /// A-440: its level at the output (its table's units).
    pub const A440_LEVEL: f32 = 0.244;

    /// FEEDBACK's cable: how late the output reaches EXTERNAL INPUT, samples, at each of
    /// `FEEDBACK_AT`'s rates (the circuit's loop from EXTERNAL INPUT back is a count of
    /// samples, not a time: 88 at 44.1 and 48 kHz, 95 at 88.2 and 96, less Potato's own path's;
    /// the loop's time sets how it oscillates).
    pub const FEEDBACK_AT: [f64; 4] = [44_100.0, 48_000.0, 88_200.0, 96_000.0];
    pub const FEEDBACK_DELAY: [f64; 4] = [84.7, 83.5, 87.6, 87.2];
}

/// The noise generators' coefficients at a rate: the white's scale (its density a hertz the
/// same at every rate), the pink filter's poles and gains (Paul Kellet's, at 48 kHz, their
/// corners kept in hertz), the red's pole and gain, and the bands' (white's high-pass and
/// low-pass, pink's).
#[derive(Debug, Clone, Copy)]
struct NoiseLaw {
    scale: f32,
    poles: [f32; 3],
    gains: [f32; 3],
    red: (f32, f32),
    band: [f32; 4],
}

impl NoiseLaw {
    fn new(rate: f64) -> NoiseLaw {
        // (At 48 kHz each factor is 1 exactly: the coefficients as they were.)
        let r = (48_000.0 / rate) as f32;
        let pole = |p: f32| p.powf(r);
        let gain = |g: f32, p: f32| g * ((1.0 - pole(p)) / (1.0 - p));
        let one_pole = |hz: f64| 1.0 - (-std::f64::consts::TAU * hz / rate).exp();
        NoiseLaw {
            scale: (rate / 48_000.0).sqrt() as f32,
            poles: [pole(0.99765), pole(0.963), pole(0.57)],
            gains: [
                gain(0.099_046, 0.99765),
                gain(0.296_516_4, 0.963),
                gain(1.052_691_3, 0.57),
            ],
            red: (pole(0.995), gain(0.05, 0.995)),
            band: [
                (-std::f64::consts::TAU * laws::WHITE_BAND.0 / rate).exp() as f32,
                one_pole(interpolate(&laws::NOISE_TOP_AT, &laws::WHITE_TOP, rate).min(0.45 * rate))
                    as f32,
                (-std::f64::consts::TAU * laws::PINK_BAND.0 / rate).exp() as f32,
                one_pole(interpolate(&laws::NOISE_TOP_AT, &laws::PINK_TOP, rate).min(0.45 * rate))
                    as f32,
            ],
        }
    }
}

/// A-440's waveform, one period (the circuit model's, measured).
const A440_SHAPE: [f32; 64] = [
    -0.014, 0.045, 0.106, 0.170, 0.238, 0.304, 0.362, 0.415, 0.459, 0.491, 0.504, 0.496, 0.480,
    0.464, 0.448, 0.433, 0.418, 0.403, 0.389, 0.375, 0.361, 0.346, 0.329, 0.311, 0.289, 0.265,
    0.239, 0.210, 0.179, 0.146, 0.112, 0.076, 0.038, -0.001, -0.042, -0.085, -0.130, -0.176,
    -0.221, -0.264, -0.304, -0.340, -0.374, -0.404, -0.430, -0.452, -0.470, -0.483, -0.492, -0.496,
    -0.495, -0.490, -0.479, -0.464, -0.443, -0.418, -0.388, -0.354, -0.316, -0.273, -0.227, -0.178,
    -0.126, -0.071,
];

/// tanh, rational: within 2.4 % of it, exactly odd, ±1 beyond ±3.
#[inline(always)]
fn tanh(x: f32) -> f32 {
    let x = x.clamp(-3.0, 3.0);
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

/// 2^x for |x| < 64, within 2e-7 of it relatively: the nearest integer in the exponent's
/// bits (found by adding and taking away 1.5 * 2^23, not by a library call), and 2^f for the
/// rest (|f| <= 1/2) by its series to the sixth power.
#[inline(always)]
fn exp2(x: f32) -> f32 {
    const SHIFT: f32 = 12_582_912.0;
    let x = x.clamp(-60.0, 60.0);
    let i = (x + SHIFT) - SHIFT;
    let f = x - i;
    let p = 1.0
        + f * (std::f32::consts::LN_2
            + f * (0.240_226_5
                + f * (0.055_504_11
                    + f * (0.009_618_129 + f * (0.001_333_355 + f * 0.000_154_035)))));
    f32::from_bits((p.to_bits() as i32 + ((i as i32) << 23)) as u32)
}

/// The two-sample polyBLEP at phase `t` (0..1) for an increment `dt` and its reciprocal.
#[inline(always)]
fn blep(t: f32, dt: f32, inv: f32) -> f32 {
    if t < dt {
        let x = t * inv;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) * inv;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

/// A table's value at `x`, its points at `xs` (rising); continued flat beyond its ends.
fn interpolate(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if x <= xs[0] {
        return ys[0];
    }
    for i in 1..xs.len() {
        if x <= xs[i] {
            let a = (x - xs[i - 1]) / (xs[i] - xs[i - 1]);
            return ys[i - 1] + (ys[i] - ys[i - 1]) * a;
        }
    }
    ys[ys.len() - 1]
}

/// A mixer channel's gain for its VOLUME (25K linear, its wiper through `series` ohm to the
/// bus, the source `source` ohm behind it), against the same at 10 for the sawtooth's.
fn channel(volume: f64, source: f64, series: f64) -> f32 {
    let law = |p: f64, source: f64| {
        let p = p.clamp(0.0, 1.0);
        let (up, down) = (source + 25e3 * (1.0 - p), (25e3 * p).max(1e-3));
        (down / (up + down)) / (up * down / (up + down) + series)
    };
    (law(volume, source) / law(1.0, laws::SOURCE_OHMS[1])) as f32
}

/// The keys as the engine plays them: which are held (a bit a MIDI note), lowest first.
#[derive(Debug, Clone, Copy, Default)]
struct Held(u128);

impl Held {
    fn set(&mut self, key: i32, on: bool) {
        if (0..128).contains(&key) {
            let b = 1u128 << key;
            if on {
                self.0 |= b;
            } else {
                self.0 &= !b;
            }
        }
    }

    fn lowest(self) -> Option<i32> {
        (self.0 != 0).then(|| self.0.trailing_zeros() as i32)
    }
}

/// An oscillator: its phase (0..1) and waveform.
#[derive(Debug, Clone, Copy)]
struct Osc {
    phase: f32,
    wave: Waveform,
}

impl Osc {
    /// One sample at increment `dt` (cycles a sample, under a half) and its reciprocal.
    #[inline(always)]
    fn tick(&mut self, dt: f32, inv: f32) -> f32 {
        let mut p = self.phase + dt;
        if p >= 1.0 {
            p -= 1.0;
        }
        self.phase = p;
        // The sawtooth falls and jumps up at its reset; the reverse sawtooth rises.
        let fall = || 1.0 - 2.0 * p + blep(p, dt, inv);
        let rect = |d: f32| {
            let q = if p >= d { p - d } else { p - d + 1.0 };
            let naive = if p < d { 1.0 } else { -1.0 };
            // (Centred: the rectangle's mean taken out, as the coupling does.)
            naive + blep(p, dt, inv) - blep(q, dt, inv) - (2.0 * d - 1.0)
        };
        match self.wave {
            // (The triangle at its top as the sawtooth resets, as the circuit's is.)
            Waveform::Triangle => laws::TRIANGLE * (4.0 * (p - 0.5).abs() - 1.0),
            Waveform::SharkTooth => {
                laws::SHARK_TRIANGLE * (4.0 * (p - 0.5).abs() - 1.0) + laws::SHARK_SAWTOOTH * fall()
            }
            Waveform::Sawtooth => laws::SAWTOOTH * fall(),
            Waveform::ReverseSawtooth => -laws::REVERSE_SAWTOOTH * fall(),
            Waveform::Square => laws::SQUARE * rect(laws::DUTY[0]),
            Waveform::WideRectangle => laws::WIDE * rect(laws::DUTY[1]),
            Waveform::NarrowRectangle => laws::NARROW * rect(laws::DUTY[2]),
        }
    }

    /// Its waveform's level at its peak (oscillator 3's swing as a modulation source).
    fn peak(wave: Waveform) -> f32 {
        match wave {
            Waveform::Triangle => laws::TRIANGLE,
            Waveform::SharkTooth => laws::SHARK_TRIANGLE + laws::SHARK_SAWTOOTH,
            Waveform::Sawtooth => laws::SAWTOOTH,
            Waveform::ReverseSawtooth => laws::REVERSE_SAWTOOTH,
            Waveform::Square => laws::SQUARE,
            Waveform::WideRectangle => laws::WIDE,
            Waveform::NarrowRectangle => laws::NARROW,
        }
    }
}

/// White noise (xorshift), and pink and red from it (filtered), with the white's band.
#[derive(Debug, Clone, Copy)]
struct Noise {
    state: u32,
    pink: [f32; 3],
    red: f32,
    /// The white's high-pass (its last input and output) and low-pass, the pink's.
    hp: (f32, f32),
    lp: f32,
    pink_hp: (f32, f32),
    pink_lp: f32,
}

impl Noise {
    #[inline(always)]
    fn white(&mut self) -> f32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        (x as i32 as f32) * (1.0 / 2_147_483_648.0)
    }

    /// White (its band), pink (its) and red, each in its generator's units; `c` the
    /// generators' coefficients at the rate.
    #[inline(always)]
    fn tick(&mut self, c: &NoiseLaw) -> (f32, f32, f32) {
        let w = self.white() * c.scale;
        let b = &mut self.pink;
        b[0] = c.poles[0] * b[0] + w * c.gains[0];
        b[1] = c.poles[1] * b[1] + w * c.gains[1];
        b[2] = c.poles[2] * b[2] + w * c.gains[2];
        let pink = 0.25 * (b[0] + b[1] + b[2] + w * 0.1848);
        self.red = c.red.0 * self.red + c.red.1 * pink;
        let (x0, y0) = self.hp;
        let hp = c.band[0] * (y0 + w - x0);
        self.hp = (w, hp);
        self.lp += (hp - self.lp) * c.band[1];
        let (x0, y0) = self.pink_hp;
        let hp = c.band[2] * (y0 + pink - x0);
        self.pink_hp = (pink, hp);
        self.pink_lp += (hp - self.pink_lp) * c.band[3];
        (self.lp, self.pink_lp, self.red)
    }
}

/// The ladder: four one-pole stages, trapezoidal, the loop solved without delay for its
/// linear part; the loop's feedback through its own limit (how loud it rings).
#[derive(Debug, Clone, Copy, Default)]
struct Ladder {
    s: [f32; 4],
}

impl Ladder {
    /// One sample of input `x` (the pair's units) at stage gain `g` (G / (1 + G) of the
    /// prewarped coefficient G) and loop gain `k`: the fourth stage's output. The input pair
    /// takes the input less the loop's feedback through its knee ([`pair`], off its centre by
    /// `bias` whose own output is `rest_bias`), as the circuit's does: driven hard, it squashes
    /// the resonance and loses less bass.
    #[inline(always)]
    fn tick(&mut self, x: f32, g: f32, k: f32, bias: f32, rest_bias: f32) -> f32 {
        let s = &mut self.s;
        let h = 1.0 - g;
        let g2 = g * g;
        let g4 = g2 * g2;
        let rest = h * (g * (g * (g * s[0] + s[1]) + s[2]) + s[3]);
        let y4 = (g4 * x + rest) / (1.0 + k * g4);
        let mut y = tanh(x - k * y4 + bias) - rest_bias;
        for st in s.iter_mut() {
            let v = (y - *st) * g;
            let out = v + *st;
            *st = out + v;
            y = out;
        }
        y
    }
}

/// A contour generator: its level (V), segment, and the samples it has gone past the
/// attack's stop (its comparator's lag).
#[derive(Debug, Clone, Copy)]
struct Contour {
    volts: f64,
    attacking: bool,
    lag: u32,
}

/// A contour's coefficients a sample (the attack's, the decay's, the release's), the
/// levels it aims at, stops at, holds and rests at (V), and its comparator's lag (samples).
#[derive(Debug, Clone, Copy, Default)]
struct ContourLaw {
    attack: f64,
    decay: f64,
    release: f64,
    aim: f64,
    stop: f64,
    sustain: f64,
    released: f64,
    rest: f64,
    lag: u32,
}

impl ContourLaw {
    fn new(k: ContourKnobs, decay_switch: bool, which: usize, rate: f64) -> ContourLaw {
        let tau = |p: f64, ohms: f64| (1e6 * audio_taper(p) + ohms) * laws::TIME_FARADS;
        let (rest, peak) = (laws::REST[which], laws::PEAK[which]);
        let rise = peak - rest;
        let [c0, c1, c2, c3] = laws::SUSTAIN[which];
        let s = k.sustain.clamp(0.0, 1.0);
        let (tau_r, released) = if decay_switch {
            (tau(k.decay, laws::TIME_OHMS.1), laws::RELEASED[which])
        } else {
            let d = k.decay.clamp(0.0, 1.0);
            (
                interpolate(&laws::RELEASE_OFF_AT, &laws::RELEASE_OFF[which], d),
                interpolate(&laws::RELEASE_OFF_AT, &laws::RELEASED_OFF[which], d),
            )
        };
        let per = |tau: f64| 1.0 - (-1.0 / (tau * rate)).exp();
        ContourLaw {
            attack: per(tau(k.attack, laws::TIME_OHMS.0)),
            decay: per(tau(k.decay, laws::TIME_OHMS.1)),
            release: per(tau_r),
            aim: rest + laws::ATTACK_AIM[which] * rise,
            stop: rest + laws::ATTACK_STOP[which] * rise,
            sustain: c0 + s * (c1 + s * (c2 + s * c3)),
            released,
            rest,
            lag: (laws::ATTACK_LAG * rate).round() as u32,
        }
    }
}

impl Contour {
    /// At rest: where a release leaves it (the circuit's rest from power-on differs a little,
    /// and moves there after its first note: decisions.md R31).
    fn at_rest(law: &ContourLaw) -> Contour {
        Contour {
            volts: law.released,
            attacking: false,
            lag: 0,
        }
    }

    /// A sample on, its gate high or low.
    #[inline(always)]
    fn tick(&mut self, gate: bool, law: &ContourLaw) -> f64 {
        if gate {
            if self.attacking {
                self.volts += (law.aim - self.volts) * law.attack;
                if self.volts >= law.stop {
                    if self.lag >= law.lag {
                        self.attacking = false;
                    }
                    self.lag += 1;
                }
            } else {
                self.volts += (law.sustain - self.volts) * law.decay;
            }
        } else {
            self.attacking = false;
            self.volts += (law.released - self.volts) * law.release;
        }
        self.volts
    }
}

/// What the panel sets, worked out when it changes.
#[derive(Debug, Clone, Copy, Default)]
struct Setting {
    /// Each oscillator's pitch offset from its key, semitones, and whether it follows the
    /// keys (oscillator 3: OSC. 3 CONTROL).
    offset: [f64; 3],
    follows: [bool; 3],
    /// Each channel's gain into the bus: the oscillators, the noise, the external input.
    gain: [f32; 3],
    noise: f32,
    ext: f32,
    /// The cutoff's log2 Hz from CUTOFF, the keys' share, AMOUNT OF CONTOUR's octaves a
    /// volt, EMPHASIS's loop gain at each of [`laws::LOOP_AT_LOG2`]'s cutoffs.
    cutoff: f64,
    tracking: f64,
    contour: f64,
    emphasis: [f64; 10],
    /// How much of the corner's rise above the ring's pitch EMPHASIS leaves (1 below 7, 0 from
    /// the threshold).
    corner: f64,
    /// The modulation: its depth (the wheel), its mix, where it goes; oscillator 3 as its
    /// source: its sample to its waveform's -1..1 (times `swing`, plus `swing_bias`: the
    /// rectangles' levels before their coupling), and that to semitones.
    depth: f64,
    offset_depth: f64,
    mix: f32,
    osc_mod: bool,
    filter_mod: bool,
    swing: f32,
    swing_bias: f32,
    mod_swing: f32,
    mod_centre: f32,
    /// GLIDE's slide a control step, semitones (rising, falling), and its RC's share left a
    /// control step (beyond the keyboard's keys); the external input's gain to the output.
    glide: (f64, f64),
    beyond: f64,
    ext_gain: f64,
    contours: [ContourLaw; 2],
}

/// Potato mode's voice.
#[derive(Debug, Clone)]
pub struct Light {
    rate: f64,
    /// The panel, and whether `set` is worked out from it.
    panel: Panel,
    set_for_panel: bool,
    set: Setting,
    held: Held,
    /// The pitch the keys ask for and where GLIDE has it, semitones (MIDI): the keyboard's
    /// own (F to C), and the keys beyond it (asked for, and where they are).
    target: f64,
    pitch: f64,
    foot: f64,
    beyond: f64,
    /// The keys' gate; the contours' (the trigger's), the samples to its next change, the
    /// samples the keys' gate has been low, and the trigger's times in samples.
    gate: bool,
    trigger: bool,
    pending: u32,
    /// Whether the trigger, when it follows the keys up, starts an attack (re-armed).
    attack: bool,
    open: u32,
    rearm: u32,
    trigger_on: u32,
    trigger_off: u32,
    osc: [Osc; 3],
    /// Each oscillator's increment, its step a sample, and the reciprocal of where it goes.
    dt: [f32; 3],
    dt_step: [f32; 3],
    inv: [f32; 3],
    noise: Noise,
    noise_c: NoiseLaw,
    /// FEEDBACK's cable at the rate, samples.
    feedback_delay: f32,
    /// The output's high-pass (its droop): its coefficient, last input and output; the input
    /// pair's offset at rest.
    droop: (f32, f32, f32),
    /// The thump's band-pass: its coefficients (high-pass, low-pass) and its state (the
    /// VCA's last gain, the high-pass's output, the low-pass's).
    thump_c: (f32, f32),
    thump: (f32, f32, f32),
    bias: f32,
    ladder: Ladder,
    /// The ladder's stage gain (and its step a sample) and loop gain.
    g: f32,
    g_step: f32,
    k: f32,
    contours: [Contour; 2],
    count: usize,
    /// ENTROPY's offsets: each oscillator's cents and the cutoff's octaves.
    detune: [f64; 3],
    cutoff_offset: f64,
    /// Oscillator 3's last sample (the modulation's source).
    osc3: f32,
    /// The VCA's law on a grid of 0.25 V from 0 to 6 V.
    vca: [f32; 25],
    /// A-440's phase and its step a sample.
    a440: (f32, f32),
    /// FEEDBACK's share; the output's last samples (FEEDBACK's delay) and where the next
    /// goes; the OVERLOAD lamp's brightest.
    pub feedback: f64,
    fed_back: [f32; 128],
    fed_at: usize,
    lamp: f64,
    /// EXTERNAL INPUT's coupling: its coefficient, last input and output.
    ext_hp: (f64, f64, f64),
    overload_peak: f64,
}

impl Light {
    /// A voice at `rate` Hz, at rest, with the default panel.
    pub fn new(rate: f64) -> Light {
        let mut vca = [0.0f32; 25];
        for (i, g) in vca.iter_mut().enumerate() {
            *g = interpolate(&laws::VCA_VOLTS, &laws::VCA_GAIN, i as f64 * 0.25) as f32;
        }
        let at = |s: f64| (s * rate).round() as u32;
        let law = ContourLaw::new(Panel::default().filter_contour, true, 0, rate);
        let mut l = Light {
            rate,
            panel: Panel::default(),
            set_for_panel: false,
            set: Setting::default(),
            held: Held::default(),
            target: laws::PITCH_AT_REST,
            pitch: laws::PITCH_AT_REST,
            foot: 0.0,
            beyond: 0.0,
            gate: false,
            trigger: false,
            pending: 0,
            attack: false,
            open: u32::MAX,
            rearm: at(laws::REARM),
            trigger_on: at(laws::TRIGGER_ON),
            trigger_off: at(laws::TRIGGER_OFF),
            osc: [Osc {
                phase: 0.0,
                wave: Waveform::Sawtooth,
            }; 3],
            dt: [0.0; 3],
            dt_step: [0.0; 3],
            inv: [0.0; 3],
            noise: Noise {
                state: 0x4D6F_6F67,
                pink: [0.0; 3],
                red: 0.0,
                hp: (0.0, 0.0),
                lp: 0.0,
                pink_hp: (0.0, 0.0),
                pink_lp: 0.0,
            },
            noise_c: NoiseLaw::new(rate),
            feedback_delay: interpolate(&laws::FEEDBACK_AT, &laws::FEEDBACK_DELAY, rate) as f32,
            droop: (
                (-std::f64::consts::TAU * laws::DROOP_HZ / rate).exp() as f32,
                0.0,
                0.0,
            ),
            thump_c: (
                (-1.0 / (laws::THUMP_TIMES.0 * rate)).exp() as f32,
                (1.0 - (-1.0 / (laws::THUMP_TIMES.1 * rate)).exp()) as f32,
            ),
            thump: (0.0, 0.0, 0.0),
            bias: tanh(laws::BIAS),
            ladder: Ladder::default(),
            g: 0.0,
            g_step: 0.0,
            k: 0.0,
            contours: [Contour::at_rest(&law); 2],
            count: 0,
            detune: [0.0; 3],
            cutoff_offset: 0.0,
            osc3: 0.0,
            vca,
            a440: (0.0, (440.0 / rate) as f32),
            feedback: 0.0,
            fed_back: [0.0; 128],
            fed_at: 0,
            lamp: 0.0,
            ext_hp: (
                (-std::f64::consts::TAU * laws::EXT_COUPLING_HZ / rate).exp(),
                0.0,
                0.0,
            ),
            overload_peak: 0.0,
        };
        // (The oscillators start in phase, as the circuit's do when its voice is made; ENTROPY's
        // floor of mismatch then draws them apart: decisions.md R9, R31.)
        l.set_panel(&Panel::default());
        l.contours = [
            Contour::at_rest(&l.set.contours[0]),
            Contour::at_rest(&l.set.contours[1]),
        ];
        l
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Seeds the noise (each voice its own stream).
    pub fn set_seed(&mut self, seed: u64) {
        let s = (seed ^ (seed >> 32)) as u32;
        self.noise.state = if s == 0 { 0x4D6F_6F67 } else { s };
    }

    /// The panel: what it sets is worked out again if it changed (some microseconds: not for
    /// every sample).
    pub fn set_panel(&mut self, p: &Panel) {
        if self.set_for_panel && *p == self.panel {
            return;
        }
        self.set_for_panel = true;
        self.panel = *p;
        let cubic = |c: [f64; 4], x: f64| c[0] + x * (c[1] + x * (c[2] + x * c[3]));
        for o in 0..3 {
            let range = (2.0 - p.osc[o].range.octaves_below_2()) * 12.0;
            let x = p.osc[o].freq.clamp(0.0, 1.0);
            let free = o == 2 && !p.osc3_control;
            let freq = match o {
                0 => 0.0,
                1 => (cubic(laws::OSC2_CENTS, x) - cubic(laws::OSC2_CENTS, FREQ_CENTRE)) / 100.0,
                _ if free => {
                    let quintic = |x: f64| {
                        laws::OSC3_FREE_CENTS
                            .iter()
                            .rev()
                            .fold(0.0, |a, c| a * x + c)
                    };
                    (quintic(x) - quintic(FREQ_CENTRE)) / 100.0
                }
                _ => (cubic(laws::OSC3_CENTS, x) - cubic(laws::OSC3_CENTS, FREQ_CENTRE)) / 100.0,
            };
            self.osc[o].wave = p.osc[o].waveform;
            let tune = if free {
                0.0
            } else {
                p.tune * laws::TUNE_CENTS / 100.0
            };
            self.set.offset[o] = range + freq + tune;
            let source = match p.osc[o].waveform {
                Waveform::Triangle => laws::SOURCE_OHMS[0],
                Waveform::Sawtooth | Waveform::ReverseSawtooth => laws::SOURCE_OHMS[1],
                Waveform::SharkTooth => laws::SOURCE_OHMS[3],
                _ => laws::SOURCE_OHMS[2],
            };
            let level = if o == 2 { laws::OSC3_LEVEL } else { 1.0 };
            self.set.gain[o] = if p.osc[o].on {
                channel(p.osc[o].volume, source, 33e3) * level
            } else {
                0.0
            };
        }
        self.set.follows = [true, true, p.osc3_control];
        self.set.noise = if p.noise_on {
            let (ohms, level) = if p.noise_pink {
                (laws::NOISE_OHMS.1, laws::PINK)
            } else {
                (laws::NOISE_OHMS.0, laws::WHITE)
            };
            channel(p.noise_volume, ohms, 11e3) / channel(1.0, ohms, 11e3) * level
        } else {
            0.0
        };
        // (The external input works at the output's level; the bus's is that over OUT and
        // DRIVE through the ladder's passband at EMPHASIS 0.)
        // (The pair's slope at its offset.)
        let slope = 1.0 - self.bias * self.bias;
        let passband = f64::from(laws::OUT * laws::DRIVE * slope) / (1.0 + laws::LOOP[3][0]);
        self.set.ext = if p.ext_on {
            (1.0 / passband) as f32
        } else {
            0.0
        };
        let t = audio_taper(p.ext_volume);
        self.set.ext_gain = laws::EXT_GAIN * t / (1.0 + laws::EXT_LOAD * t * (1.0 - t));
        let knob = p.cutoff.clamp(0.0, 1.0) * 20.0;
        let i = (knob.floor() as usize).min(19);
        let a = knob - i as f64;
        let c = &laws::CUTOFF_LOG2;
        self.set.cutoff = f64::from(c[i]) + (f64::from(c[i + 1]) - f64::from(c[i])) * a;
        self.set.tracking = f64::from(u8::from(p.keyboard_control_1)) * laws::KEYBOARD_CONTROL[0]
            + f64::from(u8::from(p.keyboard_control_2)) * laws::KEYBOARD_CONTROL[1];
        self.set.contour = laws::CONTOUR_OCTAVES_PER_VOLT
            * interpolate(
                &[0.0, 0.25, 0.5, 0.75, 1.0],
                &laws::AMOUNT,
                p.contour_amount,
            );
        self.set.corner = ((0.744 - p.emphasis) / 0.044).clamp(0.0, 1.0);
        for (k, row) in self.set.emphasis.iter_mut().zip(&laws::LOOP) {
            let at = interpolate(&laws::EMPHASIS_AT, row, p.emphasis);
            // (Past the threshold, `RING_GAIN` of the rise beyond it.)
            *k = if at > 4.0 {
                4.0 + (at - 4.0) * laws::RING_GAIN
            } else {
                at
            };
        }
        let w = p.mod_wheel.clamp(0.0, 1.0);
        let [a, b, c] = laws::WHEEL;
        self.set.depth = w * (a + w * (b + w * c));
        self.set.offset_depth = w.powf(laws::OFFSET_POWER);
        self.set.mix = p.mod_mix.clamp(0.0, 1.0) as f32;
        self.set.osc_mod = p.osc_mod;
        self.set.filter_mod = p.filter_mod;
        let w3 = p.osc[2].waveform;
        self.set.swing = 1.0 / Osc::peak(w3);
        let (kind, duty) = match w3 {
            Waveform::Triangle => (0, None),
            Waveform::ReverseSawtooth | Waveform::SharkTooth => (1, None),
            Waveform::Sawtooth => (2, None),
            Waveform::Square => (3, Some(laws::DUTY[0])),
            Waveform::WideRectangle => (3, Some(laws::DUTY[1])),
            Waveform::NarrowRectangle => (3, Some(laws::DUTY[2])),
        };
        self.set.swing_bias = duty.map_or(0.0, |d| 2.0 * d - 1.0);
        self.set.mod_swing = laws::MOD_SWING[kind] as f32;
        self.set.mod_centre = laws::MOD_CENTRE[kind] as f32;
        // (GLIDE off, the hold's own 330 ohm: at once.)
        let ohms = glide_r(p.glide, p.glide_on) + 330.0;
        let step = laws::GLIDE_RATE / ohms * CONTROL_EVERY as f64 / self.rate;
        self.set.glide = (step, step * laws::GLIDE_FALLING);
        self.set.beyond = (-(CONTROL_EVERY as f64 / self.rate) / (ohms * laws::GLIDE_FARADS)).exp();
        self.set.contours = [
            ContourLaw::new(p.filter_contour, p.decay, 0, self.rate),
            ContourLaw::new(p.loudness_contour, p.decay, 1, self.rate),
        ];
    }

    /// ENTROPY's offsets for the samples that follow: each oscillator's cents, the cutoff's
    /// octaves.
    pub fn set_offsets(&mut self, detune: [f64; 3], cutoff: f64) {
        self.detune = detune;
        self.cutoff_offset = cutoff;
    }

    /// A MIDI note pressed or released.
    pub fn note(&mut self, midi: i32, on: bool) {
        self.held.set(midi, on);
    }

    /// Whether a key is held.
    pub fn held(&self) -> bool {
        self.held.0 != 0
    }

    /// As [`crate::voice::Voice::slow_state`]: the pitch (as volts, an octave a volt) and
    /// the contours, V, once no key is down and the trigger has followed.
    pub fn slow_state(&self) -> Option<[f64; 3]> {
        if self.held() || self.trigger || self.pending > 0 {
            return None;
        }
        Some([
            (self.pitch + self.beyond) / 12.0,
            self.contours[0].volts,
            self.contours[1].volts,
        ])
    }

    /// The contours (filter, loudness), V.
    pub fn contours(&self) -> (f64, f64) {
        (self.contours[0].volts, self.contours[1].volts)
    }

    /// The OVERLOAD lamp now (0 off, 1 fully lit).
    pub fn overload(&self) -> f64 {
        self.lamp
    }

    /// The OVERLOAD lamp's brightest since the last call.
    pub fn take_overload_peak(&mut self) -> f64 {
        std::mem::take(&mut self.overload_peak)
    }

    /// Back to rest: every key up, the contours and the ladder at rest (All Sound Off).
    pub fn rest(&mut self) {
        self.held = Held::default();
        (self.gate, self.trigger, self.pending, self.open) = (false, false, 0, u32::MAX);
        self.contours = [
            Contour::at_rest(&self.set.contours[0]),
            Contour::at_rest(&self.set.contours[1]),
        ];
        self.ladder = Ladder::default();
        self.droop = (self.droop.0, 0.0, 0.0);
        self.thump = (0.0, 0.0, 0.0);
        self.ext_hp = (self.ext_hp.0, 0.0, 0.0);
        self.fed_back = [0.0; 128];
        // (In phase again, as the circuit's clean voice is.)
        for o in &mut self.osc {
            o.phase = 0.0;
        }
        self.pitch = self.target;
        self.beyond = self.foot;
    }

    /// The control laws, every [`CONTROL_EVERY`] samples: the pitch (GLIDE), the
    /// oscillators' increments and the ladder's coefficients for the next samples.
    fn control(&mut self) {
        if let Some(key) = self.held.lowest() {
            let (lo, hi) = laws::KEYBOARD_KEYS;
            let key = f64::from(key);
            self.target = key.clamp(lo, hi);
            self.foot = key - self.target;
            // (The track-and-hold follows while a key is down.)
            let (up, down) = self.set.glide;
            self.pitch += (self.target - self.pitch).clamp(-down, up);
            self.beyond = self.foot + (self.beyond - self.foot) * self.set.beyond;
        }
        let pitch = self.pitch + self.beyond;
        let wheel = self.panel.pitch_wheel * crate::modulation::PITCH_WHEEL_SEMITONES;
        for o in 0..3 {
            let key = if self.set.follows[o] {
                pitch + wheel
            } else {
                laws::OSC3_FREE_KEY
            };
            let semis = key - 69.0 + self.set.offset[o] + self.detune[o] / 100.0;
            let hz = laws::A4 * f64::from(exp2((semis / 12.0) as f32));
            let to = ((hz / self.rate) as f32).min(0.45);
            self.dt_step[o] = (to - self.dt[o]) / CONTROL_EVERY as f32;
            self.inv[o] = 1.0 / to.max(1e-9);
        }
        let (g, k) = self.coefficients(0.0);
        self.g_step = (g - self.g) / CONTROL_EVERY as f32;
        self.k = k;
    }

    /// The ladder's stage gain and loop gain for the cutoff the panel, the keys and the
    /// filter contour set, `extra` octaves added.
    #[inline(always)]
    fn coefficients(&self, extra: f64) -> (f32, f32) {
        let s = &self.set;
        let rise = self.contours[0].volts - s.contours[0].rest;
        let log2 = s.cutoff
            + s.tracking * (self.pitch + self.beyond - laws::TRACKING_FROM) / 12.0
            + s.contour * rise
            + self.cutoff_offset
            + extra;
        let corner = interpolate(&laws::CORNER_AT_LOG2, &laws::CORNER, log2) * s.corner
            + interpolate(&laws::RING_RAISE_AT_LOG2, &laws::RING_RAISE, log2) * (1.0 - s.corner);
        let hz = f64::from(exp2((log2 + corner) as f32));
        let w = (std::f64::consts::PI * hz / self.rate).min(1.4);
        // tan by its Padé form, close enough below 1.4 for the ladder's tuning.
        let w2 = w * w;
        let t = w * (15.0 - w2) / (15.0 - 6.0 * w2);
        let t = if t.is_finite() && t > 0.0 { t } else { 1e3 };
        // The loop's gain at this cutoff, between the fitted ones.
        let k = interpolate(&laws::LOOP_AT_LOG2, &s.emphasis, log2);
        ((t / (1.0 + t)) as f32, k as f32)
    }

    /// One output sample (1.0 = 5 V), `ext` at EXTERNAL INPUT.
    #[inline]
    pub fn tick(&mut self, ext: f64) -> f64 {
        if self.count == 0 {
            self.count = CONTROL_EVERY;
            self.control();
        }
        self.count -= 1;
        // The trigger: the contours follow the keys' gate late, and a key pressed again
        // within the re-arming time does not start them again.
        let gate = self.held();
        if gate != self.gate {
            self.gate = gate;
            if gate {
                if self.trigger {
                    // (Let go and pressed again before the contours fell: they hold.)
                    self.pending = 0;
                } else {
                    self.attack = self.open >= self.rearm;
                    self.pending = if self.attack {
                        self.trigger_on.max(1)
                    } else {
                        1
                    };
                }
            } else {
                self.pending = self.trigger_off.max(1);
            }
            self.open = 0;
        }
        if !gate {
            self.open = self.open.saturating_add(1);
        }
        if self.pending > 0 {
            self.pending -= 1;
            if self.pending == 0 {
                self.trigger = gate;
                if gate {
                    for c in &mut self.contours {
                        c.attacking = self.attack;
                        c.lag = 0;
                    }
                }
            }
        }
        let trigger = self.trigger;
        self.contours[0].tick(trigger, &self.set.contours[0]);
        let loud = self.contours[1].tick(trigger, &self.set.contours[1]);

        // (What the panel sets, read as needed: not the whole of it copied each sample.)
        let s = &self.set;
        let (gain, noise_gain, ext_bus, ext_gain) = (s.gain, s.noise, s.ext, s.ext_gain);
        let (depth, offset_depth, mix_k) = (s.depth, s.offset_depth, s.mix);
        let (swing, swing_bias, mod_swing, mod_centre) =
            (s.swing, s.swing_bias, s.mod_swing, s.mod_centre);
        let (osc_mod, filter_mod, follows3) = (s.osc_mod, s.filter_mod, s.follows[2]);
        for o in 0..3 {
            self.dt[o] += self.dt_step[o];
        }
        let mut dt = self.dt;
        let mut inv = self.inv;
        let mut g = self.g + self.g_step;
        self.g = g;
        let k = self.k;
        let modulating = depth > 0.0 && (osc_mod || filter_mod);
        let needs_noise = noise_gain != 0.0 || (modulating && mix_k > 0.0);
        let (white, pink, red) = if needs_noise {
            self.noise.tick(&self.noise_c)
        } else {
            (0.0, 0.0, 0.0)
        };
        if modulating {
            // Oscillator 3 (its swing to ±1) and the noise (its RMS to 1) as MODULATION MIX
            // blends them, and the line's offset, through the wheel.
            let (n, rms) = if self.panel.noise_pink {
                (red, 2.163)
            } else {
                (pink, 0.425)
            };
            // (Oscillator 3's semitones at the wheel fully forward.)
            let o = (self.osc3 * swing + swing_bias) * mod_swing + mod_centre;
            let mix = f64::from(mix_k);
            let depth = depth as f32;
            let offset = offset_depth as f32;
            if osc_mod {
                let semis = depth
                    * ((1.0 - mix_k) * o + mix_k * n * (laws::NOISE_SEMITONES_RMS / rms) as f32)
                    + offset
                        * ((1.0 - mix) * laws::MOD_OFFSET.0 + mix * laws::NOISE_OFFSET.0) as f32;
                let r = exp2(semis * (1.0 / 12.0));
                let ri = 1.0 / r;
                for o in 0..if follows3 { 3 } else { 2 } {
                    dt[o] = (dt[o] * r).min(0.45);
                    inv[o] *= ri;
                }
            }
            if filter_mod {
                let octaves = depth
                    * ((1.0 - mix_k) * o * laws::FILTER_PER_SEMITONE as f32
                        + mix_k * n * (laws::NOISE_OCTAVES_RMS / rms) as f32)
                    + offset
                        * ((1.0 - mix) * laws::MOD_OFFSET.1 + mix * laws::NOISE_OFFSET.1) as f32;
                // (On the step's prewarped coefficient, tan scaling as the cutoff does below
                // Nyquist's neighbourhood; the loop's gain as the step has it.)
                let t = (g / (1.0 - g)) * exp2(octaves);
                g = t.min(5.8) / (1.0 + t.min(5.8));
            }
        }
        // The oscillators.
        // (Each oscillator's increment's reciprocal the control step's: close enough for its
        // polyBLEP's two samples.)
        let w0 = if gain[0] != 0.0 {
            self.osc[0].tick(dt[0], inv[0])
        } else {
            0.0
        };
        let w1 = if gain[1] != 0.0 {
            self.osc[1].tick(dt[1], inv[1])
        } else {
            0.0
        };
        // (Oscillator 3 also as the modulation's source; neither heard nor modulating, its
        // phase rests, as an unheard free-running oscillator's may.)
        let w2 = if gain[2] != 0.0 || modulating {
            self.osc[2].tick(dt[2], inv[2])
        } else {
            0.0
        };
        self.osc3 = w2;
        let noise = if self.panel.noise_pink { pink } else { white };
        // EXTERNAL INPUT through its preamplifier (FEEDBACK's cable added), at the output's
        // level.
        let ext_in = if self.feedback > 0.0 {
            let d = self.feedback_delay;
            let back = |n: usize| self.fed_back[(self.fed_at + 128 - n) & 127];
            let (n, f) = (d as usize, d.fract());
            ext + self.feedback * f64::from(back(n) + (back(n + 1) - back(n)) * f)
        } else {
            ext
        };
        let mut pre = 0.0f32;
        // (Through its coupling, while anything is in it.)
        let ext_in = if ext_in != 0.0 || self.ext_hp.2 != 0.0 {
            let (a, x0, y0) = self.ext_hp;
            let y = a * (y0 + ext_in - x0);
            let y = if y.abs() < 1e-15 && ext_in == 0.0 {
                0.0
            } else {
                y
            };
            self.ext_hp = (a, ext_in, y);
            y
        } else {
            0.0
        };
        if ext_in != 0.0 {
            let x = ext_in * ext_gain;
            let (on, full) = laws::OVERLOAD;
            let lamp = ((x.abs() - on) / (full - on)).clamp(0.0, 1.0);
            self.lamp = lamp;
            self.overload_peak = self.overload_peak.max(lamp);
            pre = tanh((x / laws::EXT_CLIP) as f32) * laws::EXT_CLIP as f32;
        }
        let bus = gain[0] * w0 + gain[1] * w1 + gain[2] * w2 + noise_gain * noise + ext_bus * pre;
        let y = self.ladder.tick(
            laws::DRIVE * bus,
            g.clamp(0.0, 0.999),
            k,
            laws::BIAS,
            self.bias,
        );
        // The VCA on the loudness contour.
        let v = (loud.max(0.0) * 4.0) as f32;
        let i = (v as usize).min(23);
        let f = (v - i as f32).min(1.0);
        let gain = self.vca[i] + (self.vca[i + 1] - self.vca[i]) * f;
        // The output's coupling (the waveforms' droop, and the input pair's offset kept out).
        let y = y * laws::OUT * gain;
        let (a, x0, y0) = self.droop;
        let coupled = a * (y0 + y - x0);
        self.droop = (a, y, coupled);
        // The VCA's thump: its gain through the band-pass.
        let ((ha, lb), (g0, h0, l0)) = (self.thump_c, self.thump);
        let h = ha * (h0 + gain - g0);
        let l = l0 + lb * (h - l0);
        self.thump = (gain, h, l);
        let mut out = f64::from(coupled + laws::THUMP * l);
        if self.panel.a440 {
            let (phase, step) = self.a440;
            let p = phase + step;
            let p = if p >= 1.0 { p - 1.0 } else { p };
            self.a440.0 = p;
            let at = p * 64.0;
            let j = (at as usize).min(63);
            let f = at - j as f32;
            let (a, b) = (A440_SHAPE[j], A440_SHAPE[(j + 1) % 64]);
            out += f64::from((a + (b - a) * f) * laws::A440_LEVEL);
        }
        self.fed_back[self.fed_at] = out as f32;
        self.fed_at = (self.fed_at + 1) & 127;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fast functions follow the true ones closely enough for the model.
    #[test]
    fn the_fast_functions_follow() {
        for i in -2000..2000 {
            let x = i as f32 * 0.01;
            let t = tanh(x);
            assert!((t - x.tanh()).abs() < 0.025, "tanh({x}) = {t}");
            assert_eq!(tanh(-x), -t);
            let e = exp2(x);
            assert!((e / x.exp2() - 1.0).abs() < 1e-6, "exp2({x}) = {e}");
        }
    }

    /// A key sounds at its pitch, and silence follows its release.
    #[test]
    fn a_note_sounds_and_ends() {
        let mut l = Light::new(48_000.0);
        l.set_panel(&Panel {
            cutoff: 1.0,
            decay: false,
            ..Panel::default()
        });
        for _ in 0..4800 {
            assert!(l.tick(0.0).abs() < 1e-9);
        }
        l.note(57, true);
        // (The contours start 7.8 ms after the key, as the circuit's do.)
        for _ in 0..4800 {
            l.tick(0.0);
        }
        let mut peak = 0.0f64;
        let mut crossings = 0;
        let mut last = 0.0;
        for _ in 0..48_000 {
            let y = l.tick(0.0);
            assert!(y.is_finite());
            peak = peak.max(y.abs());
            if last <= 0.0 && y > 0.0 {
                crossings += 1;
            }
            last = y;
        }
        assert!(peak > 0.05, "silent: {peak}");
        assert!(
            (219..=221).contains(&crossings),
            "{crossings} cycles in a second at 220 Hz"
        );
        l.note(57, false);
        // (Two seconds: the VCA's thump has gone too.)
        for _ in 0..96_000 {
            l.tick(0.0);
        }
        assert!(l.tick(0.0).abs() < 1e-6);
        assert!(l.slow_state().is_some());
    }

    /// The panel the laws are checked on: oscillator 1's sawtooth alone at VOLUME 3, the
    /// filter open, the loudness contour held at its top (decisions.md R31's measurement
    /// panel).
    fn measuring() -> Panel {
        let mut p = Panel {
            cutoff: 1.0,
            contour_amount: 0.0,
            keyboard_control_1: false,
            ..Panel::default()
        };
        p.osc[0].volume = 0.3;
        p.loudness_contour = ContourKnobs {
            attack: 0.0,
            decay: 0.5,
            sustain: 1.0,
        };
        p
    }

    /// `n` samples' output.
    fn run(l: &mut Light, n: usize) -> Vec<f64> {
        (0..n).map(|_| l.tick(0.0)).collect()
    }

    /// The frequency of `x` from its rising zero crossings, Hz at 48 kHz.
    fn hz(x: &[f64]) -> f64 {
        let ups: Vec<f64> = (1..x.len())
            .filter(|&i| x[i - 1] <= 0.0 && x[i] > 0.0)
            .map(|i| i as f64 - x[i] / (x[i] - x[i - 1]))
            .collect();
        (ups.len() - 1) as f64 * 48_000.0 / (ups[ups.len() - 1] - ups[0])
    }

    /// Key 69 on 8' is 440 Hz, a RANGE an octave, oscillator 2's FREQUENCY at 0 the circuit's
    /// -855.5 cents.
    #[test]
    fn the_pitch_laws() {
        let mut l = Light::new(48_000.0);
        l.set_panel(&measuring());
        l.note(69, true);
        run(&mut l, 4800);
        assert!((hz(&run(&mut l, 48_000)) / 440.0 - 1.0).abs() < 2e-4);
        let mut p = measuring();
        p.osc[0].range = crate::tuning::Range::R4;
        l.set_panel(&p);
        run(&mut l, 4800);
        assert!((hz(&run(&mut l, 48_000)) / 880.0 - 1.0).abs() < 2e-4);
        p.osc[0] = OscPanel {
            on: false,
            ..p.osc[0]
        };
        p.osc[1] = OscPanel {
            on: true,
            volume: 0.3,
            freq: 0.0,
            ..p.osc[1]
        };
        l.set_panel(&p);
        run(&mut l, 4800);
        let cents = 1200.0 * (hz(&run(&mut l, 48_000)) / 440.0).log2();
        assert!((cents + 855.5).abs() < 1.0, "{cents} cents");
    }

    /// The filter at EMPHASIS 10 rings at the circuit's pitch once something reaches it
    /// (CUTOFF 0.5: 858 Hz), not at all at CUTOFF 0.1; at CUTOFF 0.5 it rings from EMPHASIS
    /// 7.5, not at 7.3 (the circuit's threshold 7.44).
    #[test]
    fn the_filter_rings_where_the_circuits_does() {
        let ring = |cutoff: f64, emphasis: f64| {
            let mut l = Light::new(48_000.0);
            let mut p = measuring();
            p.cutoff = cutoff;
            p.emphasis = emphasis;
            l.set_panel(&p);
            l.note(60, true);
            run(&mut l, 960);
            // (A kick from oscillator 1, then nothing reaching the filter.)
            p.osc[0].on = false;
            l.set_panel(&p);
            run(&mut l, 48_000);
            run(&mut l, 24_000)
        };
        let x = ring(0.5, 1.0);
        let rms = (x.iter().map(|y| y * y).sum::<f64>() / x.len() as f64).sqrt();
        assert!(rms > 0.1, "rms {rms}");
        assert!((hz(&x) / 858.0 - 1.0).abs() < 0.01, "{} Hz", hz(&x));
        let quiet = |x: &[f64]| x.iter().all(|y| y.abs() < 1e-4);
        assert!(quiet(&ring(0.1, 1.0)), "rang at CUTOFF 0.1");
        assert!(!quiet(&ring(0.5, 0.75)), "silent at EMPHASIS 7.5");
        assert!(quiet(&ring(0.5, 0.73)), "rang at EMPHASIS 7.3");
    }

    /// EXTERNAL INPUT coupled at its jack as the circuit's: 5 Hz through the open filter 12
    /// to 20 dB under 100 Hz (the circuit's 17.5; the output's droop alone 8).
    #[test]
    fn the_external_input_is_coupled() {
        let level = |hz: f64| {
            let mut l = Light::new(48_000.0);
            let mut p = measuring();
            p.osc[0].on = false;
            p.ext_on = true;
            p.ext_volume = 0.5;
            l.set_panel(&p);
            l.note(60, true);
            let w = std::f64::consts::TAU * hz / 48_000.0;
            let mut peak = 0.0f64;
            for i in 0..96_000 {
                let y = l.tick(0.001 * (w * f64::from(i)).sin());
                if i >= 48_000 {
                    peak = peak.max(y.abs());
                }
            }
            20.0 * peak.log10()
        };
        let d = level(5.0) - level(100.0);
        assert!(
            (-20.0..-12.0).contains(&d),
            "5 Hz {d:+.1} dB against 100 Hz"
        );
    }

    /// The noise as loud at 44.1 and 96 kHz as at 48 (within 0.5 dB, white and pink at
    /// VOLUME 5 through the open filter): its density a hertz, not a sample, as the
    /// circuit's.
    #[test]
    fn the_noise_is_as_loud_at_every_rate() {
        let rms = |rate: f64, pink: bool| {
            let mut l = Light::new(rate);
            let mut p = measuring();
            p.osc[0].on = false;
            p.noise_on = true;
            p.noise_pink = pink;
            p.noise_volume = 0.5;
            l.set_panel(&p);
            l.note(60, true);
            let n = (rate * 0.5) as usize;
            run(&mut l, n);
            let x = run(&mut l, 4 * n);
            10.0 * (x.iter().map(|y| y * y).sum::<f64>() / x.len() as f64).log10()
        };
        for pink in [false, true] {
            let at48 = rms(48_000.0, pink);
            for rate in [44_100.0, 96_000.0] {
                let d = rms(rate, pink) - at48;
                assert!(
                    d.abs() < 0.5,
                    "{} at {rate} Hz: {d:+.2} dB",
                    if pink { "pink" } else { "white" }
                );
            }
        }
    }

    /// Beyond the keyboard's keys (F to C) the rest of a glide is GLIDE's RC alone, as the
    /// circuit's voice has it: at GLIDE 6, from key 36 held a second to 43, still 2.14
    /// semitones short after 0.4 s (the circuit's 2.14); and the slide holds once the key is
    /// let go (the keyboard's track-and-hold).
    #[test]
    fn glide_beyond_the_keys_and_held_once_let_go() {
        let mut l = Light::new(48_000.0);
        let mut p = measuring();
        p.glide = 0.6;
        p.glide_on = true;
        l.set_panel(&p);
        l.note(36, true);
        run(&mut l, 48_000);
        l.note(43, true);
        l.note(36, false);
        run(&mut l, 19_200);
        let short = 43.0 - (l.pitch + l.beyond);
        assert!((short - 2.14).abs() < 0.1, "{short} semitones short");
        // Let go mid-slide in the keyboard's own range: the pitch holds there.
        l.note(72, true);
        l.note(43, false);
        run(&mut l, 1440);
        l.note(72, false);
        run(&mut l, 32);
        let at = l.pitch + l.beyond;
        assert!(at < 70.0, "the slide over by then: {at}");
        run(&mut l, 24_000);
        assert!((l.pitch + l.beyond - at).abs() < 1e-9, "slid on from {at}");
    }

    /// The contours start 7.8 ms after the key; SUSTAIN at 10 holds the loudness contour at
    /// 4.45 V; with the DECAY switch off the release falls with a time constant near 26 ms
    /// after its 15 ms, towards -0.70 V (DECAY 2); GLIDE at 4 slides 12 semitones in about
    /// 35 ms (340 a second).
    #[test]
    fn the_contour_and_glide_laws() {
        let mut l = Light::new(48_000.0);
        let mut p = measuring();
        p.decay = false;
        p.loudness_contour.decay = 0.2;
        l.set_panel(&p);
        let rest = l.contours().1;
        l.note(60, true);
        let mut started = None;
        for i in 0..4800 {
            l.tick(0.0);
            if started.is_none() && l.contours().1 > rest + 0.01 {
                started = Some(i as f64 / 48.0);
            }
        }
        let ms = started.expect("no attack");
        assert!((ms - 7.8).abs() < 0.2, "the attack at {ms} ms");
        run(&mut l, 48_000);
        assert!((l.contours().1 - 4.4549).abs() < 0.01, "{}", l.contours().1);
        l.note(60, false);
        // 15 ms of the trigger, then a time constant (26.3 ms at DECAY 2): from 4.45 V
        // towards -0.70 V.
        run(&mut l, 720 + 1440);
        let fallen = (l.contours().1 + 0.6997) / (4.4549 + 0.6997);
        let tau = 0.030 / -fallen.ln();
        assert!((tau - 0.0263).abs() < 0.002, "tau {tau}");
        // GLIDE.
        p.glide = 0.4;
        p.glide_on = true;
        l.set_panel(&p);
        l.note(48, true);
        run(&mut l, 48_000);
        l.note(60, true);
        l.note(48, false);
        let mut at = None;
        for i in 0..9600 {
            l.tick(0.0);
            if at.is_none() && l.pitch > 59.5 {
                at = Some(i as f64 / 48.0);
            }
        }
        let ms = at.expect("no glide");
        assert!((ms - 33.8).abs() < 2.0, "11.5 semitones in {ms} ms");
    }
}
