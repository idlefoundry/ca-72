//! The light model's stages used alone, through their public API only (decisions.md
//! R-LIGHT-STAGES), as another crate (Olifant) uses them: an oscillator in tune at its law's
//! increment and CONTINUOUS's blend the switch's places to the bit; the filter ringing where
//! CUTOFF's law and the light voice put it and falling 24 dB an octave above; the contours the
//! light voice's own, to the bit, run beside it with its trigger's times; the noise's colours;
//! the VCA's gain and level.

use ca72::light::{
    self, Contour, ContourLaw, HISS, LadderFilter, Light, Noise, NoiseLaw, Osc, Vca, laws,
};
use ca72::vcf::FilterMode;
use ca72::voice::{ContourKnobs, Panel, Waveform, shape_place};

const RATE: f64 = 48_000.0;

/// The frequency of `x` at `rate` from its rising zero crossings, Hz.
fn hz(x: &[f64], rate: f64) -> f64 {
    let ups: Vec<f64> = (1..x.len())
        .filter(|&i| x[i - 1] <= 0.0 && x[i] > 0.0)
        .map(|i| i as f64 - x[i] / (x[i] - x[i - 1]))
        .collect();
    assert!(ups.len() > 2, "{} crossings", ups.len());
    (ups.len() - 1) as f64 * rate / (ups[ups.len() - 1] - ups[0])
}

/// One oscillator alone at [`light::oscillator_increment`]'s step is within 0.2 cent of equal
/// temperament from A4 (`laws::A4`), from two octaves under it to more than an octave and a
/// half over, at 44.1 and 48 kHz; its sawtooth swings from 1 to -1, its band-limited edges
/// ringing past them by under a fifth (1.156 at A4).
#[test]
fn the_oscillator_alone_is_in_tune() {
    for rate in [44_100.0, RATE] {
        for semis in [-24.0, -5.0, 0.0, 7.0, 19.0] {
            let (dt, inv) = light::oscillator_increment(semis, rate);
            let mut o = Osc::new(Waveform::Triangle);
            let x: Vec<f64> = (0..rate as usize)
                .map(|_| f64::from(o.tick(dt, inv)))
                .collect();
            let want = laws::A4 * 2f64.powf(semis / 12.0);
            let cents = 1200.0 * (hz(&x, rate) / want).log2();
            assert!(
                cents.abs() < 0.2,
                "{semis} semitones at {rate}: {cents:+.3} cents"
            );
        }
    }
    let (dt, inv) = light::oscillator_increment(0.0, RATE);
    let mut o = Osc::new(Waveform::Sawtooth);
    let x: Vec<f32> = (0..4800).map(|_| o.tick(dt, inv)).collect();
    let (lo, hi) = x
        .iter()
        .fold((0.0f32, 0.0f32), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(
        (1.0..1.2).contains(&hi) && (-1.2..-1.0).contains(&lo),
        "{lo} to {hi}"
    );
    assert_eq!(o.peak(), laws::SAWTOOTH);
}

/// CONTINUOUS through the public stage: at a switch's place it is that place's waveform, to
/// the bit; between two it is their blend at their levels through the mixer
/// ([`light::continuous_ratios`]), to the bit; past SQUARE, still, the rectangle.
#[test]
fn the_continuous_waveform_blends_the_switchs_places() {
    assert_eq!(
        light::continuous_waveform(0, 2.0, false),
        (Waveform::Sawtooth, None)
    );
    assert_eq!(
        light::continuous_waveform(0, 1.25, false),
        (Waveform::SharkTooth, Some(1.25))
    );
    assert_eq!(
        light::continuous_waveform(2, 2.0, true),
        (Waveform::ReverseSawtooth, Some(2.0))
    );
    assert_eq!(
        light::continuous_waveform(1, 3.4, false),
        (Waveform::Square, None)
    );
    let (dt, inv) = light::oscillator_increment(-3.0, RATE);
    let ratios = light::continuous_ratios(0.7);
    let mut blend = Osc::new(Waveform::Sawtooth);
    let (wave, shape) = light::continuous_waveform(0, 1.5, false);
    blend.set_continuous(wave, shape, false, ratios, 1.5);
    let mut at_place = Osc::new(Waveform::Sawtooth);
    at_place.set_continuous(Waveform::Sawtooth, Some(2.0), false, [1.0; 4], 2.0);
    let mut shark = Osc::new(shape_place(0, 1).unwrap());
    let mut saw = Osc::new(shape_place(0, 2).unwrap());
    for _ in 0..4800 {
        let (a, b) = (shark.tick(dt, inv), saw.tick(dt, inv));
        assert_eq!(at_place.tick(dt, inv).to_bits(), b.to_bits());
        let want = 0.5 * ratios[1] * a + 0.5 * ratios[2] * b;
        assert_eq!(blend.tick(dt, inv).to_bits(), want.to_bits());
    }
}

/// The light voice's own panel for measuring the filter (as its unit tests' `measuring`):
/// oscillator 1 at VOLUME 3, CUTOFF `cutoff`, EMPHASIS 10, no contour or keyboard control.
fn ringing_panel(cutoff: f64) -> Panel {
    let mut p = Panel {
        cutoff,
        emphasis: 1.0,
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

/// The public filter at CUTOFF `cutoff` and EMPHASIS 10, kicked by a sawtooth for 20 ms and
/// then fed the mixer's hiss alone, as the voice's is: its last half second.
fn ring_alone(cutoff: f64) -> Vec<f64> {
    let (emphasis, corner) = light::emphasis_law(1.0);
    let log2 = light::node_log2(&light::cutoff_points(), light::knob_current(cutoff));
    let (g, k) = light::ladder_coefficients(
        log2,
        &emphasis,
        corner,
        RATE * light::FILTER_OVERSAMPLE as f64,
    );
    let mut f = LadderFilter::new(RATE);
    f.set_passband(light::passband(log2));
    f.set_drive(1.0, light::ring_lift(log2, corner));
    let (dt, inv) = light::oscillator_increment(60.0 - 69.0, RATE);
    let mut o = Osc::new(Waveform::Sawtooth);
    let gain = light::channel(ca72::voice::volume_track(0.3), laws::SOURCE_OHMS[1], 33e3);
    let mut seed = light::HISS_SEED;
    let scale = HISS as f32;
    let mut hiss = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed as i32 as f32) * (scale / 2_147_483_648.0)
    };
    let mut out = Vec::new();
    for n in 0..72_960 {
        let osc = if n < 960 { gain * o.tick(dt, inv) } else { 0.0 };
        let y = f.tick(osc + hiss(), g, k);
        if n >= 48_960 {
            out.push(f64::from(y));
        }
    }
    out
}

/// The light voice at the same panel, kicked by its oscillator 1 for 20 ms: its last half
/// second.
fn ring_in_voice(cutoff: f64) -> Vec<f64> {
    let mut l = Light::new(RATE);
    let mut p = ringing_panel(cutoff);
    l.set_panel(&p);
    l.note(60, true);
    for _ in 0..960 {
        l.tick(0.0);
    }
    p.osc[0].on = false;
    l.set_panel(&p);
    for _ in 0..48_000 {
        l.tick(0.0);
    }
    (0..24_000).map(|_| l.tick(0.0)).collect()
}

/// The public filter at EMPHASIS 10 rings at CUTOFF's law (`laws::CUTOFF_LOG2`, the circuit's
/// self-oscillating pitch, within 1 %) and where the light voice's filter rings (within 0.3 %),
/// at CUTOFF 3, 5 and 7.
#[test]
fn the_filter_alone_rings_where_the_law_puts_it() {
    for (cutoff, point) in [(0.3, 6), (0.5, 10), (0.7, 14)] {
        let alone = hz(&ring_alone(cutoff), RATE);
        let law = 2f64.powf(f64::from(laws::CUTOFF_LOG2[point]));
        let voice = hz(&ring_in_voice(cutoff), RATE);
        assert!(
            (alone / law - 1.0).abs() < 0.01,
            "CUTOFF {cutoff}: {alone:.1} Hz alone, the law's {law:.1}"
        );
        assert!(
            (alone / voice - 1.0).abs() < 0.003,
            "CUTOFF {cutoff}: {alone:.1} Hz alone, {voice:.1} in the voice"
        );
    }
}

/// A sine's level through the public filter at EMPHASIS 0 and CUTOFF `cutoff`, FILTER POLES
/// `poles`, dB.
fn through(cutoff: f64, poles: u8, sine: f64) -> f64 {
    let (emphasis, corner) = light::emphasis_law(0.0);
    let log2 = light::node_log2(&light::cutoff_points(), light::knob_current(cutoff));
    let (g, k) = light::ladder_coefficients(
        log2,
        &emphasis,
        corner,
        RATE * light::FILTER_OVERSAMPLE as f64,
    );
    let mut f = LadderFilter::new(RATE);
    f.set_shape(FilterMode::Lo, poles);
    f.set_passband(light::passband(log2));
    f.set_drive(1.0, light::ring_lift(log2, corner));
    let w = std::f64::consts::TAU * sine / RATE;
    let mut peak = 0.0f64;
    for n in 0..48_000 {
        let y = f.tick((0.01 * (w * n as f64).sin()) as f32, g, k);
        if n >= 24_000 {
            peak = peak.max(f64::from(y).abs());
        }
    }
    20.0 * peak.log10()
}

/// Above its cutoff the public filter falls 6 dB an octave a pole (FILTER POLES 4 and 2,
/// within 2 dB an octave, between four and eight times CUTOFF 4's pitch).
#[test]
fn the_filter_alone_falls_six_db_an_octave_a_pole() {
    let fc = 2f64.powf(f64::from(laws::CUTOFF_LOG2[8]));
    for poles in [4u8, 2] {
        let fall = through(0.4, poles, 8.0 * fc) - through(0.4, poles, 4.0 * fc);
        let want = -6.0 * f64::from(poles);
        assert!(
            (fall - want).abs() < 2.0,
            "{poles} poles: {fall:+.1} dB an octave"
        );
    }
}

/// The public contours run beside the light voice's, gated by its trigger's times
/// (`laws::TRIGGER_ON` after the key, `laws::TRIGGER_OFF` after its release), are its own to
/// the bit, both of them, with the DECAY switch on and off.
#[test]
fn the_contours_alone_are_the_voices() {
    for decay_switch in [true, false] {
        let mut p = Panel {
            decay: decay_switch,
            ..Panel::default()
        };
        p.filter_contour = ContourKnobs {
            attack: 0.35,
            decay: 0.6,
            sustain: 0.4,
        };
        p.loudness_contour = ContourKnobs {
            attack: 0.2,
            decay: 0.45,
            sustain: 0.7,
        };
        let mut l = Light::new(RATE);
        l.set_panel(&p);
        let laws_ = [
            ContourLaw::new(p.filter_contour, decay_switch, 0, RATE),
            ContourLaw::new(p.loudness_contour, decay_switch, 1, RATE),
        ];
        let mut alone = laws_.map(|law| Contour::at_rest(&law));
        let on = (laws::TRIGGER_ON * RATE).round() as usize;
        let off = (laws::TRIGGER_OFF * RATE).round() as usize;
        let release = 60_000;
        l.note(48, true);
        for n in 1..=120_000 {
            if n == release + 1 {
                l.note(48, false);
            }
            l.tick(0.0);
            let gate = n >= on && n < release + off;
            if n == on {
                for c in &mut alone {
                    c.start(true);
                }
            }
            let v = [
                alone[0].tick(gate, &laws_[0]),
                alone[1].tick(gate, &laws_[1]),
            ];
            let (filter, loud) = l.contours();
            assert_eq!(
                (v[0].to_bits(), v[1].to_bits()),
                (filter.to_bits(), loud.to_bits()),
                "sample {n}, DECAY switch {decay_switch}: {v:?} against {filter}, {loud}"
            );
            assert_eq!(alone[1].volts().to_bits(), loud.to_bits());
        }
    }
}

/// A band's power in `x` about `f` Hz at 48 kHz (an octave's third wide, every 10 Hz).
fn band(x: &[f32], f: f64) -> f64 {
    let mut sum = 0.0;
    let (lo, hi) = (f * 2f64.powf(-1.0 / 6.0), f * 2f64.powf(1.0 / 6.0));
    let mut at = lo;
    while at < hi {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, &v) in x.iter().enumerate() {
            let a = std::f64::consts::TAU * at * n as f64 / RATE;
            re += f64::from(v) * a.cos();
            im += f64::from(v) * a.sin();
        }
        sum += re * re + im * im;
        at += 10.0;
    }
    sum / ((hi - lo) / 10.0)
}

/// The public noise: seeded, it repeats; the white about as strong at 300 Hz as at 2.4 kHz
/// (within 1.5 dB), the pink 9 dB weaker three octaves up (within 2 dB); its channel at full
/// VOLUME the generator's level (`laws::WHITE`, `laws::PINK`).
#[test]
fn the_noise_alone_has_its_colours() {
    let law = NoiseLaw::new(RATE);
    let mut a = Noise::new();
    a.set_seed(7);
    let mut b = Noise::default();
    b.set_seed(7);
    let mut white = Vec::new();
    let mut pink = Vec::new();
    for _ in 0..48_000 {
        let x = a.tick(&law);
        assert_eq!(x, b.tick(&law));
        white.push(x.0);
        pink.push(x.1);
    }
    let db = |x: &[f32], f: f64| 10.0 * band(&x[24_000..], f).log10();
    let w = db(&white, 2400.0) - db(&white, 300.0);
    assert!(w.abs() < 1.5, "white {w:+.1} dB over three octaves");
    let p = db(&pink, 2400.0) - db(&pink, 300.0);
    assert!((p + 9.0).abs() < 2.0, "pink {p:+.1} dB over three octaves");
    assert!((light::noise_gain(1.0, false) - laws::WHITE).abs() < 1e-6);
    assert!((light::noise_gain(1.0, true) - laws::PINK).abs() < 1e-6);
    assert!(light::noise_gain(0.5, false) < laws::WHITE);
}

/// The public VCA: no gain at 0 V, 1 within 0.1 % at the loudness contour's 4.455 V; a 1 kHz
/// sine through it at that gain `laws::OUT` of its level (within 1 %); nothing in, its thump
/// gone again a second after the contour stops.
#[test]
fn the_vca_alone_has_its_gain() {
    assert_eq!(light::vca_gain(0.0), 0.0);
    assert!((light::vca_gain(4.455) - 1.0).abs() < 1e-3);
    let mut v = Vca::new(RATE);
    let w = std::f64::consts::TAU * 1000.0 / RATE;
    let mut peak = 0.0f32;
    for n in 0..48_000 {
        let y = v.tick((w * f64::from(n)).sin() as f32, 4.455);
        if n >= 24_000 {
            peak = peak.max(y.abs());
        }
    }
    assert!((peak / laws::OUT - 1.0).abs() < 0.01, "{peak}");
    v.reset();
    let thump: Vec<f32> = (0..96_000)
        .map(|n| v.tick(0.0, if n < 4800 { 4.455 } else { 0.0 }))
        .collect();
    assert!(thump[..9600].iter().any(|y| y.abs() > 1e-3));
    assert!(thump[96_000 - 4800..].iter().all(|y| y.abs() < 1e-4));
}
