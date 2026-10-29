//! CONTINUOUS's waveform and its MOD (decisions.md R-CONTINUOUS-WAVE), which the instrument does
//! not have: at each of the switch's positions CONTINUOUS is the switch, to the bit, in every
//! quality; between them each shape blends into the next with no silence (oscillator 3's reverse
//! path too); past SQUARE the rectangle's width goes to the floor, where the pulse still comes;
//! MOD moves the shape on, back or either way about the switch, as deep as the wheel is forward,
//! in both voices alike; and a width that moves past the ramp switches the comparator at once.

use ca72::light::Light;
use ca72::tuning::Range;
use ca72::vco::{CoreParams, Vco};
use ca72::voice::{
    ModDirection, Panel, Quality, RECT_FLOOR, SHAPE_PLACES, Shape, Voice, Waveform, bias_at,
    rect_duty, shape_at, shape_bias,
};

const RATE: f64 = 48_000.0;

/// A panel with oscillator `n` alone on `w`, its shape `s`, at `quality`.
fn panel(quality: Quality, n: usize, w: Waveform, s: Shape) -> Panel {
    let mut p = Panel {
        quality,
        ..Panel::default()
    };
    for k in 0..3 {
        p.osc[k].on = k == n;
    }
    p.osc[n].waveform = w;
    p.shape[n] = s;
    p
}

fn continuous(at: f64, modulated: bool) -> Shape {
    Shape {
        continuous: true,
        at,
        modulated,
        direction: ModDirection::Onward,
    }
}

/// The full voice's samples over `secs`, a note held from the start.
fn full(p: Panel, secs: f64) -> Vec<f64> {
    let mut v = Voice::new(RATE, p);
    v.note(57, true);
    (0..(secs * RATE) as usize).map(|_| v.tick()).collect()
}

/// The light voice's, likewise.
fn light(p: Panel, secs: f64) -> Vec<f64> {
    let mut l = Light::new(RATE);
    l.set_panel(&p);
    l.note(57, true);
    (0..(secs * RATE) as usize).map(|_| l.tick(0.0)).collect()
}

fn rms(x: &[f64]) -> f64 {
    let tail = &x[x.len() / 3..];
    (tail.iter().map(|v| v * v).sum::<f64>() / tail.len() as f64).sqrt()
}

#[test]
fn the_travel_gives_the_switchs_widths_exactly() {
    assert_eq!(shape_bias(2.0), 0.0);
    assert_eq!(shape_bias(3.0), 0.0);
    assert_eq!(shape_bias(4.0), -1.5);
    assert_eq!(shape_bias(5.0), -2.5);
    assert_eq!(shape_bias(SHAPE_PLACES), RECT_FLOOR);
    for (bias, at) in [
        (0.0, 3.0),
        (-1.5, 4.0),
        (-2.5, 5.0),
        (RECT_FLOOR, SHAPE_PLACES),
    ] {
        assert!((bias_at(bias) - at).abs() < 1e-12, "{bias} V");
    }
    // A place a host's or a preset's rounding leaves near (four decimals of the travel: three
    // ten-thousandths of a step at most): the place exactly.
    for k in 0..6 {
        let k = f64::from(k);
        for off in [0.5e-3, -0.5e-3, 3e-4] {
            assert_eq!(shape_at(continuous(k + off, false), Default::default()), k);
        }
        let travel = (k / SHAPE_PLACES) as f32;
        let back = f64::from(travel) * SHAPE_PLACES;
        assert_eq!(shape_at(continuous(back, false), Default::default()), k);
    }
    // The duty falls along the rectangle's stretch: the square's about half, the floor's about
    // five to nine percent after retaining the physical width-divider load.
    let d: Vec<f64> = (0..=20)
        .map(|k| rect_duty(shape_bias(3.0 + f64::from(k) / 20.0 * (SHAPE_PLACES - 3.0))))
        .collect();
    assert!(d.windows(2).all(|w| w[1] < w[0]), "{d:?}");
    assert!(
        d[0] > 0.47 && d[0] < 0.49 && d[20] > 0.05 && d[20] < 0.095,
        "{d:?}"
    );
}

/// Each place of the switch on oscillator `n` (its steps) and the switch's own position there;
/// oscillator 3's second place (its reverse shark tooth) is the switch's nowhere.
fn places(n: usize) -> Vec<(f64, Waveform)> {
    let mut v = vec![(0.0, Waveform::Triangle)];
    if n == 2 {
        v.push((2.0, Waveform::ReverseSawtooth));
    } else {
        v.push((1.0, Waveform::SharkTooth));
        v.push((2.0, Waveform::Sawtooth));
    }
    v.extend([
        (3.0, Waveform::Square),
        (4.0, Waveform::WideRectangle),
        (5.0, Waveform::NarrowRectangle),
    ]);
    v
}

#[test]
fn at_the_switchs_positions_continuous_is_the_switch_to_the_bit() {
    for q in [
        Quality::NoCompromises,
        Quality::HighFidelity,
        Quality::Potato,
    ] {
        for n in [0, 2] {
            for (at, w) in places(n) {
                let stepped = full(panel(q, n, w, Shape::STEPPED), 0.12);
                // CONTINUOUS ignores which position the switch names; with MOD on and the wheel
                // back, the shape is where it is turned.
                for (named, modulated) in [(Waveform::Triangle, false), (w, true)] {
                    let c = full(panel(q, n, named, continuous(at, modulated)), 0.12);
                    assert!(
                        stepped == c,
                        "{q:?} oscillator {}: CONTINUOUS at {at} (MOD {modulated}) differs",
                        n + 1
                    );
                }
            }
        }
    }
    for n in [0, 2] {
        for (at, w) in places(n) {
            let stepped = light(panel(Quality::Potato, n, w, Shape::STEPPED), 0.12);
            for modulated in [false, true] {
                let c = light(
                    panel(Quality::Potato, n, w, continuous(at, modulated)),
                    0.12,
                );
                assert!(
                    stepped == c,
                    "light oscillator {}: CONTINUOUS at {at} (MOD {modulated}) differs",
                    n + 1
                );
            }
        }
    }
}

#[test]
fn between_the_places_each_shape_blends_into_the_next_without_silence() {
    for n in [0, 2] {
        for (q, light_voice) in [(Quality::NoCompromises, false), (Quality::Potato, true)] {
            let level = |at: f64| {
                let p = panel(q, n, Waveform::Sawtooth, continuous(at, false));
                rms(&if light_voice {
                    light(p, 0.25)
                } else {
                    full(p, 0.25)
                })
            };
            let levels: Vec<(f64, f64)> = (0..=12)
                .map(|k| f64::from(k) * 0.25)
                .map(|at| (at, level(at)))
                .collect();
            eprintln!("oscillator {} light {light_voice}: {levels:.4?}", n + 1);
            let (lo, hi) = levels
                .iter()
                .fold((f64::INFINITY, 0.0f64), |(a, b), &(_, l)| {
                    (a.min(l), b.max(l))
                });
            assert!(lo > 0.0 && hi / lo < 4.0, "a dip: {levels:?}");
            // A blend is neither of its places.
            for k in 0..3 {
                let (a, m, b) = (levels[4 * k].1, levels[4 * k + 2].1, levels[4 * k + 4].1);
                assert!(m != a && m != b, "{k}: {a} {m} {b}");
            }
        }
    }
}

/// A panel for MOD: oscillator 1 CONTINUOUS at `at` with MOD `direction`, oscillator 3 a
/// triangle at LO off the keyboard and out of the mixer (an LFO), MODULATION MIX at oscillator 3,
/// the wheel at `wheel`.
fn mod_panel(quality: Quality, at: f64, direction: ModDirection, wheel: f64) -> Panel {
    let mut p = panel(
        quality,
        0,
        Waveform::Square,
        Shape {
            direction,
            ..continuous(at, true)
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

/// MOD's onward share over `secs` (after a settling tenth of a second), and oscillator 1's shape
/// where the full voice had it each sample.
fn mod_run(p: Panel, light_voice: bool, secs: f64) -> (Vec<f64>, Vec<f64>) {
    let n = (secs * RATE) as usize;
    let skip = (0.1 * RATE) as usize;
    if light_voice {
        let mut l = Light::new(RATE);
        l.set_panel(&p);
        l.note(57, true);
        let u = (0..n)
            .map(|_| {
                l.tick(0.0);
                l.probe_shape_mod()
            })
            .collect::<Vec<_>>();
        (u[skip..].to_vec(), Vec::new())
    } else {
        let mut v = Voice::new(RATE, p);
        v.note(57, true);
        let (mut u, mut at) = (Vec::new(), Vec::new());
        for _ in 0..n {
            v.tick();
            u.push(v.probe_shape_mod());
            at.push(v.probe_shapes()[0].expect("CONTINUOUS"));
        }
        (u[skip..].to_vec(), at[skip..].to_vec())
    }
}

fn span(x: &[f64]) -> (f64, f64) {
    x.iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &v| {
            (a.min(v), b.max(v))
        })
}

#[test]
fn mod_moves_the_shape_its_way_as_deep_as_the_wheel_is_forward() {
    for (q, light_voice) in [(Quality::NoCompromises, false), (Quality::Potato, true)] {
        let run = |wheel: f64, secs: f64| {
            mod_run(
                mod_panel(q, 0.0, ModDirection::Onward, wheel),
                light_voice,
                secs,
            )
            .0
        };
        let (lo, hi) = span(&run(1.0, 3.0));
        eprintln!("{q:?} light {light_voice}: wheel forward {lo:.3}..{hi:.3}");
        assert!(lo < 0.03 && hi > 0.97, "{q:?}: {lo}..{hi}");
        assert_eq!(span(&run(0.0, 1.0)), (0.0, 0.0), "{q:?} wheel back");
        let (lo, hi) = span(&run(0.5, 3.0));
        eprintln!("{q:?} light {light_voice}: wheel half {lo:.3}..{hi:.3}");
        assert!(
            hi - lo > 0.05 && hi - lo < 0.7 && hi < 0.75,
            "{q:?} half: {lo}..{hi}"
        );
    }
    // Its way, from the switch at SAWTOOTH: on (clockwise) never below it, back never above it,
    // either way about it; held to the travel.
    for (direction, check) in [
        (ModDirection::Onward, 0),
        (ModDirection::Back, 1),
        (ModDirection::Both, 2),
    ] {
        let (_, at) = mod_run(
            mod_panel(Quality::NoCompromises, 2.0, direction, 1.0),
            false,
            3.0,
        );
        let (lo, hi) = span(&at);
        eprintln!("{direction:?}: {lo:.3}..{hi:.3}");
        assert!(lo >= 0.0 && hi <= SHAPE_PLACES, "{direction:?}");
        match check {
            0 => assert!(lo >= 2.0 - 1e-9 && hi > 5.0, "{direction:?}: {lo}..{hi}"),
            1 => assert!(hi <= 2.0 + 1e-9 && lo == 0.0, "{direction:?}: {lo}..{hi}"),
            _ => assert!(
                lo < 0.5 && hi > 3.5 && lo < 2.0 && hi > 2.0,
                "{direction:?}: {lo}..{hi}"
            ),
        }
    }
}

#[test]
fn both_voices_move_the_shape_the_same_way() {
    let (full_u, _) = mod_run(
        mod_panel(Quality::NoCompromises, 0.0, ModDirection::Onward, 1.0),
        false,
        2.0,
    );
    let (light_u, _) = mod_run(
        mod_panel(Quality::Potato, 0.0, ModDirection::Onward, 1.0),
        true,
        2.0,
    );
    let mean = |u: &[f64]| u.iter().sum::<f64>() / u.len() as f64;
    let n = full_u.len().min(light_u.len());
    let (fa, la) = (&full_u[..n], &light_u[..n]);
    let (ma, mb) = (mean(fa), mean(la));
    eprintln!("means: full {ma:.3}, light {mb:.3}");
    assert!((ma - mb).abs() < 0.1, "means {ma} and {mb}");
    let cov: f64 = (0..n).map(|i| (fa[i] - ma) * (la[i] - mb)).sum();
    let va: f64 = (0..n).map(|i| (fa[i] - ma).powi(2)).sum();
    let vb: f64 = (0..n).map(|i| (la[i] - mb).powi(2)).sum();
    let corr = cov / (va * vb).sqrt();
    eprintln!("correlation {corr:.3}");
    assert!(corr > 0.5, "the voices move apart: correlation {corr}");
}

/// Oscillator 1's rectangle alone at `hz` with width bias `bias`: its duty cycle (share of
/// the time at its high level, 0 V) and its falls per cycle, over `cycles` cycles.
fn vco_duty(hz: f64, bias: f64, cycles: f64) -> (f64, f64) {
    let rate = 96_000.0;
    let mut v = Vco::new(rate, 8);
    v.use_width_network(true);
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
            assert!(
                (falls - 1.0).abs() < 0.1,
                "{hz} Hz {bias} V: {falls} falls a cycle"
            );
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
    let hz = 100.0;
    let core = CoreParams::default();
    let i0 = (core.c1 + core.c_par) * (core.v_top0 - core.v_threshold) * hz;
    for physical_width_source in [false, true] {
        for rate in [44_100.0, 48_000.0, 96_000.0] {
            for oversample in [1, 4, 8] {
                for moves in [true, false] {
                    let mut v = Vco::new(rate, oversample);
                    v.use_width_network(physical_width_source);
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
                    eprintln!(
                        "physical source {physical_width_source}, rate {rate}, os{oversample}, moves {moves}: falls {:.3} ms after",
                        after * 1e3
                    );
                    if moves {
                        assert!(after < 1e-4, "following, it fell {after} s after");
                    } else {
                        // (The instrument's switch never moves mid-cycle: the next cycle's threshold.)
                        assert!(after > 0.7 / hz, "not following, it fell {after} s after");
                    }
                }
            }
        }
    }
}

#[test]
fn pulse_knots_are_continuous_just_outside_the_rounding_deadband() {
    for q in [
        Quality::NoCompromises,
        Quality::HighFidelity,
        Quality::Potato,
    ] {
        for (at, w) in [
            (3.0, Waveform::Square),
            (4.0, Waveform::WideRectangle),
            (5.0, Waveform::NarrowRectangle),
        ] {
            let reference = full(panel(q, 0, w, Shape::STEPPED), 0.12);
            for off in [-0.00101, 0.00101] {
                let moved = full(panel(q, 0, w, continuous(at + off, false)), 0.12);
                let delta: Vec<_> = reference.iter().zip(&moved).map(|(a, b)| a - b).collect();
                assert!(
                    rms(&delta) / rms(&reference) < 0.025,
                    "{q:?} knot{at} offset{off} produced a finite jump"
                );
            }
        }
    }
}
