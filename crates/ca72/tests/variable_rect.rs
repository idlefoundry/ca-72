//! VARIABLE's rectangle and its PWM (decisions.md R-VARIABLE-RECT), which the instrument does
//! not have: at the switch's own three widths VARIABLE is the switch, to the bit, in every
//! quality; its travel gives those three exactly; PWM's share follows oscillator 3 as deep as
//! the wheel is forward, square to floor at the wheel fully forward, in both voices alike; the
//! pulse still comes at the floor across the keyboard; and a width that moves past the ramp
//! switches the comparator at once.

use ca72::light::Light;
use ca72::tuning::Range;
use ca72::vco::{CoreParams, Vco};
use ca72::voice::{
    Panel, Quality, RECT_FLOOR, RectWidth, Voice, Waveform, rect_bias_travel, rect_duty,
    rect_travel_bias,
};

const RATE: f64 = 48_000.0;

/// A panel with oscillator 1 alone on `w`, its rectangle `r`, at `quality`.
fn panel(quality: Quality, w: Waveform, r: RectWidth) -> Panel {
    let mut p = Panel {
        quality,
        ..Panel::default()
    };
    p.osc[0].waveform = w;
    p.rect[0] = r;
    p
}

/// The full voice's samples over `secs`, a note held from the start, its panel set by `set`
/// at each sample's start (from the sample's index).
fn full(p: Panel, secs: f64, set: &dyn Fn(&mut Panel, usize)) -> Vec<f64> {
    let mut v = Voice::new(RATE, p);
    v.note(57, true);
    (0..(secs * RATE) as usize)
        .map(|i| {
            set(&mut v.panel, i);
            v.tick()
        })
        .collect()
}

/// The light voice's, likewise.
fn light(p: Panel, secs: f64) -> Vec<f64> {
    let mut l = Light::new(RATE);
    l.set_panel(&p);
    l.note(57, true);
    (0..(secs * RATE) as usize).map(|_| l.tick(0.0)).collect()
}

#[test]
fn the_travel_gives_the_switchs_three_widths_exactly() {
    for (bias, steps) in [(0.0, 0.0), (-1.5, 1.0), (-2.5, 2.0)] {
        let t = rect_bias_travel(bias);
        assert_eq!(rect_travel_bias(t), bias, "{bias} V");
        assert!((t * ca72::voice::RECT_STEPS - steps).abs() < 1e-12);
        // (Within a ten-thousandth of a step, as a host's or a preset's rounding leaves it.)
        let nudged = t + 0.5e-4 / ca72::voice::RECT_STEPS;
        assert_eq!(rect_travel_bias(nudged), bias, "{bias} V nudged");
        assert_eq!(rect_travel_bias(f64::from(t as f32)), bias, "{bias} V as f32");
    }
    assert_eq!(rect_travel_bias(1.0), RECT_FLOOR);
    assert_eq!(rect_travel_bias(2.0), RECT_FLOOR);
    assert_eq!(rect_travel_bias(-1.0), 0.0);
    // Straight between, and back.
    for k in 0..=100 {
        let t = f64::from(k) / 100.0;
        let b = rect_travel_bias(t);
        assert!((rect_bias_travel(b) - t).abs() < 1e-3, "{t}");
    }
    // The duty falls along it: the square's about half, the floor's about a sixteenth.
    let d: Vec<f64> = (0..=20)
        .map(|k| rect_duty(rect_travel_bias(f64::from(k) / 20.0)))
        .collect();
    assert!(d.windows(2).all(|w| w[1] < w[0]), "{d:?}");
    assert!((d[0] - 0.484).abs() < 1e-6 && d[20] > 0.05 && d[20] < 0.08, "{d:?}");
}

#[test]
fn at_the_switchs_widths_variable_is_the_switch_to_the_bit() {
    let at = |bias: f64, pwm: bool| RectWidth {
        variable: true,
        width: rect_travel_bias(rect_bias_travel(bias)),
        pwm,
    };
    for q in [
        Quality::NoCompromises,
        Quality::HighFidelity,
        Quality::Potato,
    ] {
        for (w, bias) in [
            (Waveform::Square, 0.0),
            (Waveform::WideRectangle, -1.5),
            (Waveform::NarrowRectangle, -2.5),
        ] {
            let stepped = full(panel(q, w, RectWidth::STEPPED), 0.15, &|_, _| {});
            // VARIABLE ignores which of the three the switch names; with PWM on and the
            // wheel back it is still the width as turned.
            for (sw, pwm) in [(Waveform::Square, false), (w, true)] {
                let variable = full(panel(q, sw, at(bias, pwm)), 0.15, &|_, _| {});
                assert!(
                    stepped == variable,
                    "{q:?} {w:?}: VARIABLE at {bias} V (PWM {pwm}) differs from the switch"
                );
            }
        }
    }
    for (w, bias) in [
        (Waveform::Square, 0.0),
        (Waveform::WideRectangle, -1.5),
        (Waveform::NarrowRectangle, -2.5),
    ] {
        let p = panel(Quality::Potato, w, RectWidth::STEPPED);
        let stepped = light(p, 0.15);
        for pwm in [false, true] {
            let variable = light(panel(Quality::Potato, w, at(bias, pwm)), 0.15);
            assert!(
                stepped == variable,
                "light {w:?}: VARIABLE at {bias} V (PWM {pwm}) differs from the switch"
            );
        }
    }
}

/// A panel for PWM: oscillator 1 VARIABLE at the square with PWM, oscillator 3 a triangle at
/// LO off the keyboard and out of the mixer (an LFO), MODULATION MIX at oscillator 3, the
/// wheel at `wheel`.
fn pwm_panel(quality: Quality, wheel: f64) -> Panel {
    let mut p = panel(
        quality,
        Waveform::Square,
        RectWidth {
            variable: true,
            width: 0.0,
            pwm: true,
        },
    );
    p.osc[2].waveform = Waveform::Triangle;
    p.osc[2].range = Range::Lo;
    p.osc[2].on = false;
    p.osc3_control = false;
    p.mod_mix = 0.0;
    p.mod_wheel = wheel;
    p
}

/// PWM's share's lowest and highest over `secs` (after a settling second's tenth), and its
/// samples.
fn pwm_span(quality: Quality, wheel: f64, light_voice: bool, secs: f64) -> (f64, f64, Vec<f64>) {
    let p = pwm_panel(quality, wheel);
    let n = (secs * RATE) as usize;
    let u: Vec<f64> = if light_voice {
        let mut l = Light::new(RATE);
        l.set_panel(&p);
        l.note(57, true);
        (0..n)
            .map(|_| {
                l.tick(0.0);
                l.probe_pwm()
            })
            .collect()
    } else {
        let mut v = Voice::new(RATE, p);
        v.note(57, true);
        (0..n)
            .map(|_| {
                v.tick();
                v.probe_pwm()
            })
            .collect()
    };
    let tail = &u[(0.1 * RATE) as usize..];
    let lo = tail.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = tail.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (lo, hi, u)
}

#[test]
fn pwm_sweeps_the_whole_travel_with_the_wheel_forward_and_none_with_it_back() {
    for (q, light_voice) in [(Quality::NoCompromises, false), (Quality::Potato, true)] {
        let (lo, hi, _) = pwm_span(q, 1.0, light_voice, 3.0);
        eprintln!("{q:?} light {light_voice}: wheel forward {lo:.3}..{hi:.3}");
        assert!(lo < 0.03 && hi > 0.97, "{q:?}: {lo}..{hi}");
        let (lo, hi, _) = pwm_span(q, 0.0, light_voice, 1.0);
        assert!(lo == 0.0 && hi == 0.0, "{q:?} wheel back: {lo}..{hi}");
        // Halfway, a smaller sweep about the travel's first half.
        let (lo, hi, _) = pwm_span(q, 0.5, light_voice, 3.0);
        eprintln!("{q:?} light {light_voice}: wheel half {lo:.3}..{hi:.3}");
        assert!(hi - lo > 0.05 && hi - lo < 0.7 && hi < 0.75, "{q:?} half: {lo}..{hi}");
    }
}

#[test]
fn both_voices_sweep_the_same_way() {
    // (Oscillator 3's own pitch differs a little between the voices: compare their first
    // quarter cycle's direction and their means.)
    let (_, _, full_u) = pwm_span(Quality::NoCompromises, 1.0, false, 2.0);
    let (_, _, light_u) = pwm_span(Quality::Potato, 1.0, true, 2.0);
    let mean = |u: &[f64]| u.iter().sum::<f64>() / u.len() as f64;
    let (a, b) = (mean(&full_u[4800..]), mean(&light_u[4800..]));
    eprintln!("means: full {a:.3}, light {b:.3}");
    assert!((a - b).abs() < 0.1, "means {a} and {b}");
    // The share's correlation over the run, centred.
    let corr = {
        let (fa, la) = (&full_u[4800..], &light_u[4800..]);
        let n = fa.len().min(la.len());
        let (ma, mb) = (mean(&fa[..n]), mean(&la[..n]));
        let cov: f64 = (0..n).map(|i| (fa[i] - ma) * (la[i] - mb)).sum();
        let va: f64 = (0..n).map(|i| (fa[i] - ma).powi(2)).sum();
        let vb: f64 = (0..n).map(|i| (la[i] - mb).powi(2)).sum();
        cov / (va * vb).sqrt()
    };
    eprintln!("correlation {corr:.3}");
    assert!(corr > 0.5, "the voices sweep apart: correlation {corr}");
}

/// Oscillator 1's rectangle alone at `hz` with width bias `bias`: its duty cycle (share of
/// the time at its high level, 0 V) and its falls per cycle, over `cycles` cycles.
fn vco_duty(hz: f64, bias: f64, cycles: f64) -> (f64, f64) {
    let rate = 96_000.0;
    let mut v = Vco::new(rate, 8);
    let core = CoreParams::default();
    // A timing current for `hz`: the capacitor through the ramp's travel each period.
    let i0 = (core.c1 + core.c_par) * (core.v_top0 - core.v_threshold) * hz;
    let n = (cycles / hz * rate) as usize;
    let warm = (rate * 0.02) as usize;
    let (mut high, mut falls, mut was_high) = (0usize, 0usize, true);
    for i in 0..warm + n {
        let r = v.tick_current(i0, i0 * 1e-3, bias).rect;
        let is_high = r > -2.15;
        if i >= warm {
            high += usize::from(is_high);
            falls += usize::from(was_high && !is_high);
        }
        was_high = is_high;
    }
    (high as f64 / n as f64, falls as f64 / cycles)
}

#[test]
fn the_pulse_still_comes_at_the_floor_across_the_keyboard() {
    for hz in [55.0f64, 220.0, 880.0, 2_000.0, 4_000.0] {
        let cycles = (hz * 0.25).max(20.0);
        for bias in [0.0, -1.5, -2.5, RECT_FLOOR] {
            let (duty, falls) = vco_duty(hz, bias, cycles);
            eprintln!(
                "{hz} Hz, {bias} V: duty {duty:.3} (law {:.3}), {falls:.2} falls a cycle",
                rect_duty(bias)
            );
            assert!((falls - 1.0).abs() < 0.1, "{hz} Hz {bias} V: {falls} falls a cycle");
            // (The edges' fixed delays add a little at the top of the keyboard.)
            let tol = if hz > 1_000.0 { 0.03 } else { 0.015 };
            assert!(
                (duty - rect_duty(bias)).abs() < tol,
                "{hz} Hz {bias} V: duty {duty} against the law's {}",
                rect_duty(bias)
            );
        }
    }
}

#[test]
fn a_width_that_moves_past_the_ramp_switches_the_comparator_at_once() {
    let rate = 96_000.0;
    let hz = 100.0;
    let core = CoreParams::default();
    let i0 = (core.c1 + core.c_par) * (core.v_top0 - core.v_threshold) * hz;
    for moves in [true, false] {
        let mut v = Vco::new(rate, 8);
        v.set_width_moves(moves);
        // Square for a few cycles, then just after a reset plus 30 % of a cycle (high: the
        // square falls about half way), narrow (it falls at about 18 %: already passed).
        let mut resets = v.last_reset().1;
        let mut since = None;
        let mut low_after = None;
        for i in 0..(rate * 0.2) as usize {
            let (_, r) = v.last_reset();
            if r != resets {
                resets = r;
                if r == 10 {
                    since = Some(i);
                }
            }
            let narrow = since.is_some_and(|s| i >= s + (0.3 * rate / hz) as usize);
            let out = v.tick_current(i0, i0 * 1e-3, if narrow { -2.5 } else { 0.0 });
            if narrow && low_after.is_none() && out.rect < -2.15 {
                low_after = Some(i - since.unwrap() - (0.3 * rate / hz) as usize);
            }
        }
        // (Less the outputs' delay through band-limiting and decimation.)
        let after = (low_after.expect("it falls at last") as f64 - v.latency()) / rate;
        eprintln!("width moves {moves}: falls {:.1} ms after", after * 1e3);
        if moves {
            assert!(after < 1e-4, "following, it fell {after} s after");
        } else {
            // (The instrument's switch never moves mid-cycle: the next cycle's threshold.)
            assert!(after > 0.7 / hz, "not following, it fell {after} s after");
        }
    }
}
