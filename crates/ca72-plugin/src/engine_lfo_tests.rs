//! The LFOs in the engine (decisions.md R-LFO): nothing assigned is the engine as before to the
//! bit; an assigned knob moved by depth × LFO after its glide, clamped to its travel, through
//! its law; RETRIG independent of the main RETRIGGER, ENV and FREE's FADE against it;
//! each voice its own with MONO off, the
//! shared targets the shared copy; the timed lock; the sleep.

use super::*;
use crate::lfo::{self, Assignment, Edges, Mode, Settings, Target};

const RATE: f64 = 48_000.0;

/// LO's light voices (quick in debug), no ENTROPY (its tolerances leave CUTOFF alone anyway).
fn controls() -> Controls {
    Controls {
        entropy: 0.0,
        potato: true,
        ..Controls::default()
    }
}

/// `c` with LFO `lfo` set to `s` and modulating `t` by `depth` from travel `base`.
fn with(mut c: Controls, lfo: usize, s: Settings, t: Target, depth: f32, base: f32) -> Controls {
    c.lfo.lfo[lfo] = s;
    let slot = c.lfo.to[lfo]
        .iter_mut()
        .find(|a| a.is_none())
        .expect("room");
    *slot = Some(Assignment { target: t, depth });
    c.lfo.base[t.index()] = base;
    c
}

fn sharp(mode: Mode, at: f64, hz: f64) -> Settings {
    Settings {
        at,
        hz,
        mode,
        edges: Edges::Sharp,
        ..Settings::default()
    }
}

fn engine(c: &Controls) -> Engine {
    let mut e = Engine::new();
    e.set(c);
    e.prepare(RATE, 3);
    e
}

/// `n` samples in blocks of up to 256: the left output.
fn run(e: &mut Engine, n: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(n);
    let (mut l, mut r) = ([0.0f32; 256], [0.0f32; 256]);
    let mut left = n;
    while left > 0 {
        let k = left.min(256);
        e.render(&[], &mut l[..k], &mut r[..k]);
        e.end_block(k);
        out.extend_from_slice(&l[..k]);
        left -= k;
    }
    out
}

fn secs(s: f64) -> usize {
    (s * RATE) as usize
}

fn key(e: &mut Engine, key: u8, on: bool) {
    e.event(Event::Note { key, on });
}

/// Voice `k`'s panel as it last played.
fn panel(e: &Engine, k: usize) -> Panel {
    e.slots[k].play.try_lock().expect("free").voice.panel
}

fn slot_of(e: &Engine, key: u8) -> usize {
    e.slots
        .iter()
        .position(|s| s.gate && s.note == key)
        .expect("a voice")
}

/// With no assignment, or every depth 0, the LFOs' settings (MONO off, RETRIG, FADE, timed)
/// change nothing: the samples are the same to the bit, the one instrument and POLY.
#[test]
fn nothing_assigned_plays_as_before_to_the_bit() {
    for poly in [false, true] {
        let base = Controls {
            poly,
            voices: 3,
            ..controls()
        };
        let mut set = base;
        for k in 0..lfo::LFOS {
            set.lfo.lfo[k] = Settings {
                at: 2.4,
                hz: 7.0,
                mode: Mode::Retrig,
                mono: false,
                fade: 0.3,
                // (And POLARITY UNI, REVERSE and ON off: R-LFO's corrections.)
                uni: k % 2 == 0,
                reverse: k < 2,
                on: k != 3,
                ..Settings::default()
            };
        }
        let zero = with(set, 1, set.lfo.lfo[1], Target::Cutoff, 0.0, 0.5);
        let play = |c: &Controls| {
            let mut e = engine(c);
            let mut out = Vec::new();
            for (k, n) in [60u8, 64, 67].into_iter().enumerate() {
                key(&mut e, n, true);
                out.extend(run(&mut e, secs(0.1) + 37 * k));
            }
            key(&mut e, 64, false);
            out.extend(run(&mut e, secs(0.1)));
            out.into_iter().map(f32::to_bits).collect::<Vec<_>>()
        };
        let before = play(&base);
        assert!(before.iter().any(|b| f32::from_bits(*b).abs() > 1e-3));
        assert_eq!(before, play(&set), "settings alone, poly {poly}");
        assert_eq!(before, play(&zero), "a depth of 0, poly {poly}");
    }
}

/// A shared LFO moves its knob by depth × its value from the knob's travel, every step,
/// clamped to the travel: CUTOFF at 0.5 by 0.4 swings 0.1 to 0.9; at 0.9 by 0.5 it stops at 1.
#[test]
fn a_shared_lfo_moves_its_knob_within_its_travel() {
    for (base, depth, lo, hi) in [(0.5f32, 0.4f32, 0.1, 0.9), (0.9, 0.5, 0.4, 1.0)] {
        let c = with(
            controls(),
            0,
            sharp(Mode::Free, 0.0, 5.0),
            Target::Cutoff,
            depth,
            base,
        );
        let mut e = engine(&c);
        key(&mut e, 60, true);
        let (mut min, mut max) = (f64::MAX, f64::MIN);
        for _ in 0..(secs(0.25) / lfo::STEP) {
            run(&mut e, lfo::STEP);
            let v = e.bank.shared_values[0];
            let want = (f64::from(base) + f64::from(depth) * v).clamp(0.0, 1.0);
            let got = panel(&e, 0).cutoff;
            assert!((got - want).abs() < 1e-6, "{got} {want}");
            (min, max) = (min.min(got), max.max(got));
        }
        assert!(
            (min - lo).abs() < 0.01 && (max - hi).abs() < 0.01,
            "{min} {max}"
        );
        assert!(max <= 1.0);
    }
}

/// What a depth means (the owner, 2026-10-20, ATTACK TIME: "why am I seeing +25% on the
/// selector, but then the actual automation goes +-12.5%?"): an LFO at +25 % swings its knob's
/// travel (its pointer, not seconds or hertz) exactly 0.25 each way about its setting, the
/// contours' ATTACK TIMEs as CUTOFF, clamped at the travel's ends; a macro at +25 % and 100 %
/// adds exactly 0.25, at −25 % takes it away.
#[test]
fn a_depth_is_so_much_of_the_knobs_travel_each_way() {
    type Of = fn(&Panel) -> f64;
    let knobs: [(Target, Of); 3] = [
        (Target::FilterAttack, |p| p.filter_contour.attack),
        (Target::LoudnessAttack, |p| p.loudness_contour.attack),
        (Target::Cutoff, |p| p.cutoff),
    ];
    for (t, of) in knobs {
        for (base, lo, hi) in [(0.5f32, 0.25, 0.75), (0.1, 0.0, 0.35), (0.9, 0.65, 1.0)] {
            let c = with(controls(), 0, sharp(Mode::Free, 0.0, 5.0), t, 0.25, base);
            let mut e = engine(&c);
            key(&mut e, 60, true);
            let (mut min, mut max) = (f64::MAX, f64::MIN);
            for _ in 0..(secs(0.25) / lfo::STEP) {
                run(&mut e, lfo::STEP);
                let v = e.bank.shared_values[0];
                let want = (f64::from(base) + 0.25 * v).clamp(0.0, 1.0);
                let got = of(&panel(&e, 0));
                assert!((got - want).abs() < 1e-6, "{t:?} at {base}: {got} {want}");
                (min, max) = (min.min(got), max.max(got));
            }
            // (The triangle's peaks within a step of 32 samples at 5 Hz.)
            assert!(
                (min - lo).abs() < 0.002 && (max - hi).abs() < 0.002,
                "{t:?} at {base}: {min}..{max}, not {lo}..{hi}"
            );
        }
        for depth in [0.25f32, -0.25] {
            let mut c = controls();
            c.lfo.to[lfo::Source::Macro(0).index()][0] = Some(Assignment { target: t, depth });
            c.lfo.base[t.index()] = 0.5;
            c.lfo.macros[0] = 1.0;
            let mut e = engine(&c);
            key(&mut e, 60, true);
            run(&mut e, lfo::STEP * 4);
            let got = of(&panel(&e, 0));
            assert!(
                (got - (0.5 + f64::from(depth))).abs() < 1e-6,
                "{t:?}: a macro at 100 % by {depth}: {got}"
            );
        }
    }
}

/// The stereo and output knobs (the owner, 2026-10-20: "make sure the stereo controls can be
/// controlled by macros and lfos", "and entropy, drive, and output"): WIDTH, INNER and DETUNE
/// (DOUBLE on), ENTROPY, DRIVE, LEVEL, MAIN OUTPUT VOLUME and FEEDBACK each moved by an LFO at
/// +25 % exactly as their law takes setting + 0.25 × its output every step, and by a macro at
/// 100 % and +25 % to setting + 0.25; the shared ones take the LFO's shared copy.
#[test]
fn the_stereo_and_output_knobs_take_an_lfo_and_a_macro() {
    type Of = fn(&Controls) -> f64;
    let knobs: [(Target, Of); 8] = [
        (Target::Spread, |c| c.spread),
        (Target::Inner, |c| c.inner),
        (Target::Double, |c| c.double),
        (Target::Entropy, |c| c.entropy),
        (Target::Drive, |c| c.drive),
        (Target::Level, |c| c.level),
        (Target::Volume, |c| c.volume),
        (Target::Feedback, |c| c.feedback),
    ];
    let base = Controls {
        double: 0.5,
        ..controls()
    };
    let law = base.lfo.law.into();
    let want = |t: Target, travel: f64, of: Of| {
        let mut c = base;
        t.write(&mut c, travel, law);
        of(&c)
    };
    for (t, of) in knobs {
        assert_eq!(t.per_voice(), t == Target::Feedback, "{t:?}");
        let c = with(base, 0, sharp(Mode::Free, 0.0, 5.0), t, 0.25, 0.5);
        let mut e = engine(&c);
        key(&mut e, 60, true);
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for _ in 0..(secs(0.25) / lfo::STEP) {
            run(&mut e, lfo::STEP);
            let v = e.bank.shared_values[0];
            let got = of(&e.controls);
            let w = want(t, 0.5 + 0.25 * v, of);
            assert!(
                (got - w).abs() < 1e-9 * w.abs().max(1.0),
                "{t:?}: {got} {w}"
            );
            (lo, hi) = (lo.min(got), hi.max(got));
        }
        let (a, b) = (want(t, 0.25, of), want(t, 0.75, of));
        let span = (b - a).abs();
        assert!(
            (lo - a.min(b)).abs() < 0.01 * span && (hi - a.max(b)).abs() < 0.01 * span,
            "{t:?}: {lo}..{hi}, not {a}..{b}"
        );
        let mut c = base;
        c.lfo.to[lfo::Source::Macro(2).index()][0] = Some(Assignment {
            target: t,
            depth: 0.25,
        });
        c.lfo.base[t.index()] = 0.5;
        c.lfo.macros[2] = 1.0;
        let mut e = engine(&c);
        key(&mut e, 60, true);
        run(&mut e, lfo::STEP * 4);
        let got = of(&e.controls);
        assert!(
            (got - b).abs() < 1e-9 * b.abs().max(1.0),
            "{t:?}: a macro: {got} {b}"
        );
    }
}

/// DRIVE moved by an LFO under AUTO GAIN (R-AUTO-GAIN-DRIVE; agreed with the owner,
/// 2026-10-20): the correction is the curve in force read at the modulated DRIVE every step, so
/// the level stays as steady as the curve keeps it at DRIVE's settings, and the curve is not
/// changed (nothing measures it: only the editor's own DRIVE asks, `editor.rs`). The sound's
/// curve measured as the plug-in measures it; a held note's level at DRIVE 0 to 24 dB in 3 dB
/// steps with AUTO GAIN on sets what the curve keeps; an LFO sweeping DRIVE over 0 to 24 dB
/// keeps every 50 ms within that and half a decibel (the correction's 10 ms glide behind a
/// moving DRIVE); without AUTO GAIN the same sweep moves the level by several decibels.
#[test]
fn an_lfo_sweeping_drive_keeps_auto_gains_level() {
    let mut sound = controls();
    sound.lock = true;
    sound.panel.osc[1].on = false;
    sound.panel.osc[2].on = false;
    sound.panel.loudness_contour.sustain = 1.0;
    let cal = crate::drive::Calibration::default();
    cal.set_rate(RATE);
    cal.ask(&Controls {
        drive: 12.0,
        ..sound
    });
    cal.measure_now();
    let curve = crate::drive::Curve(cal.saved().db);
    assert_ne!(curve.0, crate::drive::AVERAGE.0, "its own curve");
    let db = |y: &[f32]| {
        let e = y.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>() / y.len() as f64;
        10.0 * e.log10()
    };
    let held = |c: &Controls| {
        let mut e = engine(c);
        e.set_curve(curve);
        key(&mut e, 45, true);
        run(&mut e, secs(0.3));
        e
    };
    let at = |drive: f64| {
        let mut e = held(&Controls { drive, ..sound });
        db(&run(&mut e, secs(0.4)))
    };
    let off = at(0.0);
    let kept = (1..=8)
        .map(|k| (at(3.0 * f64::from(k)) - off).abs())
        .fold(0.0f64, f64::max);
    let sweep = |auto_gain: bool| {
        let c = Controls {
            drive: 12.0,
            auto_gain,
            ..sound
        };
        // (A triangle from DRIVE 0 to 24 dB and back each 2 s.)
        let c = with(c, 0, sharp(Mode::Free, 0.0, 0.5), Target::Drive, 0.5, 0.5);
        let mut e = held(&c);
        let windows: Vec<f64> = (0..40).map(|_| db(&run(&mut e, secs(0.05)))).collect();
        assert_eq!(e.curve().0, curve.0, "the curve in force, unchanged");
        windows
    };
    let swept = sweep(true);
    let worst = swept.iter().map(|w| (w - off).abs()).fold(0.0f64, f64::max);
    assert!(
        worst <= kept + 0.5,
        "swept {worst:.2} dB from DRIVE off, the curve keeps {kept:.2} dB"
    );
    let plain = sweep(false);
    let moved = plain.iter().fold(f64::MIN, |m, w| m.max(*w))
        - plain.iter().fold(f64::MAX, |m, w| m.min(*w));
    assert!(
        moved > 3.0 && moved > 2.0 * worst,
        "without AUTO GAIN: {moved:.2} dB"
    );
    eprintln!(
        "DRIVE swept 0..24 dB: within {worst:.2} dB of DRIVE off (the curve keeps {kept:.2} dB \
         at its settings); without AUTO GAIN {moved:.2} dB"
    );
}

/// POLARITY UNI (R-LFO's corrections; the owner, 2026-10-20: "yes, bipolar is exactly the right
/// idea here ... probably a switch would be sufficient"): the LFO runs 0..1, so at +25 % its knob
/// moves from its setting up to 0.25 above it, at −25 % down to 0.25 below, never the other way;
/// BI swings both ways.
#[test]
fn a_unipolar_lfo_moves_its_knob_one_way() {
    for (depth, lo, hi) in [(0.25f32, 0.5, 0.75), (-0.25, 0.25, 0.5)] {
        let s = Settings {
            uni: true,
            ..sharp(Mode::Free, 0.0, 5.0)
        };
        let c = with(controls(), 0, s, Target::Cutoff, depth, 0.5);
        let mut e = engine(&c);
        key(&mut e, 60, true);
        let (mut min, mut max) = (f64::MAX, f64::MIN);
        for _ in 0..(secs(0.25) / lfo::STEP) {
            run(&mut e, lfo::STEP);
            let v = e.bank.shared_values[0];
            assert!((0.0..=1.0).contains(&v), "UNI's output 0..1: {v}");
            let got = panel(&e, 0).cutoff;
            assert!((got - (0.5 + f64::from(depth) * v)).abs() < 1e-6);
            (min, max) = (min.min(got), max.max(got));
        }
        assert!(
            (min - lo).abs() < 0.002 && (max - hi).abs() < 0.002,
            "UNI at {depth}: {min}..{max}, not {lo}..{hi}"
        );
    }
}

/// REVERSE (R-LFO's corrections): the waveform read backwards, its place in its cycle p at
/// 1 − p, so that a reversed SAWTOOTH is a falling sawtooth, 1 − 2p, sample for sample; every
/// shape, STEPPED and CONTINUOUS, is its own mirror; and a timed LFO reversed is the mirror of
/// it forwards, step for step.
#[test]
fn a_reversed_lfo_reads_its_waveform_backwards() {
    let s = Settings {
        reverse: true,
        ..sharp(Mode::Free, 2.0, 3.7)
    };
    let mut g = lfo::Gen::default();
    let per = s.hz / RATE;
    let mut cycles = 0.0f64;
    for _ in 0..4000 {
        let v = g.value(&s, None, lfo::STEP as f64 / RATE);
        let falling = 1.0 - 2.0 * cycles;
        assert!(
            (v - falling).abs() < 1e-12,
            "at {cycles}: {v}, not {falling}"
        );
        g.advance(&s, per, lfo::STEP, 1.0 / RATE);
        let c = cycles + per * lfo::STEP as f64;
        cycles = c - c.floor();
    }
    for at in [0.0, 1.0, 1.5, 2.0, 2.6, 3.0, 4.0, 5.0, 5.95] {
        // (From just after a cycle's start: at 0 a reversed read is the cycle's end, where a
        // forward read at 1 is the next cycle's start.)
        for k in 1..=64 {
            let p = f64::from(k) / 64.0 * 0.999;
            assert_eq!(
                lfo::read(at, p, true),
                lfo::read(at, 1.0 - p, false),
                "{at} {p}"
            );
        }
    }
    let timed = |reverse: bool| {
        let s = Settings {
            reverse,
            timed: Some(Length::Beats(0.25)),
            ..sharp(Mode::Free, 2.0, 1.0)
        };
        let c = with(controls(), 0, s, Target::Cutoff, 0.25, 0.5);
        let mut e = engine(&c);
        e.set_position(Position::start(123.0, RATE));
        key(&mut e, 60, true);
        (0..200)
            .map(|_| {
                run(&mut e, lfo::STEP);
                e.bank.shared_values[0]
            })
            .collect::<Vec<_>>()
    };
    let (fwd, rev) = (timed(false), timed(true));
    for (k, (f, r)) in fwd.iter().zip(&rev).enumerate() {
        // (A sawtooth's mirror is its negative, but at its cycle's start, where both are at 1
        // or -1 a sample apart.)
        assert!(
            (f + r).abs() < 1e-9 || (f.abs() > 0.99 && r.abs() > 0.99),
            "{k}: {f} {r}"
        );
    }
    assert!(rev.iter().any(|v| *v > 0.5) && rev.iter().any(|v| *v < -0.5));
}

/// ON (R-LFO's corrections; the owner, 2026-10-20: "we need to be able to mute the LFO"): off,
/// the LFO fades out in a straight line over 10 ms, its knob never stepping more than the fade
/// and its own movement take it, then contributes exactly nothing (its knob at its setting);
/// on again, it fades back in over 10 ms and is then exactly the output of the same LFO never
/// turned off. Its assignments and depths stay.
#[test]
fn a_muted_lfo_fades_to_nothing_and_back() {
    let s = sharp(Mode::Free, 0.0, 2.0);
    let c = with(controls(), 0, s, Target::Cutoff, 0.25, 0.5);
    let (mut a, mut b) = (engine(&c), engine(&c));
    for e in [&mut a, &mut b] {
        key(e, 60, true);
        run(e, secs(0.1));
    }
    let both = |a: &mut Engine, b: &mut Engine| {
        run(a, lfo::STEP);
        run(b, lfo::STEP);
    };
    let mut off = c;
    off.lfo.lfo[0].on = false;
    a.set(&off);
    let fade = secs(lfo::MUTE_FADE) / lfo::STEP + 1;
    let step = 0.25 * lfo::STEP as f64 / RATE / lfo::MUTE_FADE;
    let mut was = panel(&a, 0).cutoff;
    for _ in 0..fade {
        both(&mut a, &mut b);
        let now = panel(&a, 0).cutoff;
        assert!((now - was).abs() <= step + 0.002, "no jump: {was} to {now}");
        was = now;
    }
    for _ in 0..50 {
        both(&mut a, &mut b);
        assert_eq!(a.bank.shared_values[0], 0.0, "exactly nothing");
        assert!((panel(&a, 0).cutoff - 0.5).abs() < 1e-12, "at its setting");
        assert!(!a.lfo_readings()[0].moving, "nor the LFO key's glow");
    }
    assert_eq!(a.controls.lfo.to[0][0].map(|x| x.depth), Some(0.25), "kept");
    a.set(&c);
    for _ in 0..fade {
        both(&mut a, &mut b);
        let now = panel(&a, 0).cutoff;
        assert!(
            (now - was).abs() <= step + 0.002,
            "no jump back: {was} to {now}"
        );
        was = now;
    }
    for _ in 0..50 {
        both(&mut a, &mut b);
        assert_eq!(
            a.bank.shared_values[0], b.bank.shared_values[0],
            "the same output"
        );
        assert_eq!(panel(&a, 0).cutoff, panel(&b, 0).cutoff);
    }
}

/// A target's law: OSC. 2 FREQUENCY's knob travel through the computer's law, as its knob
/// reads (the pot's wiper the law's), and FEEDBACK's taper.
#[test]
fn a_knob_is_moved_through_its_own_law() {
    let mut c = with(
        controls(),
        0,
        sharp(Mode::Free, 3.0, 2.0),
        Target::Osc2Frequency,
        0.3,
        0.5,
    );
    c = with(c, 0, c.lfo.lfo[0], Target::Feedback, 0.2, 0.0);
    let mut e = engine(&c);
    key(&mut e, 60, true);
    run(&mut e, lfo::STEP * 3);
    let v = e.bank.shared_values[0];
    let p = panel(&e, 0);
    let want = ca72::freq_dial::Pot::Two.wiper_at(ca72::freq_dial::Law::Semitones, 0.5 + 0.3 * v);
    assert!(
        (p.osc[1].freq - want).abs() < 1e-6,
        "{} {want} {v}",
        p.osc[1].freq
    );
    assert!((e.controls.feedback - ca72::voice::feedback_law((0.2 * v).max(0.0))).abs() < 1e-6);
}

/// Each LFO's RETRIG starts phase and FADE again on a legato key independently of the main
/// contours' RETRIGGER. The one instrument and POLY's shared LFO.
#[test]
fn retrig_and_fade_start_on_every_note_independently_of_main_retrigger() {
    for poly in [false, true] {
        for retrigger in [false, true] {
            let mut c = controls();
            c.poly = poly;
            c.panel.retrigger = retrigger;
            let s = Settings {
                fade: 1.0,
                ..sharp(Mode::Retrig, 2.0, 1.0)
            };
            let c = with(c, 0, s, Target::Cutoff, 0.5, 0.5);
            let mut e = engine(&c);
            assert_eq!(
                e.lfo_readings()[0].fade,
                0.0,
                "no note yet: a FADE in at nothing"
            );
            key(&mut e, 60, true);
            run(&mut e, secs(0.3));
            let r = e.lfo_readings()[0];
            assert!((r.since - 0.3).abs() < 0.002, "{r:?}");
            assert!((r.fade - 0.3).abs() < 0.002 && (r.phase - 0.3).abs() < 0.002);
            key(&mut e, 64, true);
            run(&mut e, secs(0.1));
            let r = e.lfo_readings()[0];
            let since = 0.1;
            assert!((r.since - since).abs() < 0.002, "{poly} {retrigger} {r:?}");
            assert!((r.phase - since).abs() < 0.002);
            key(&mut e, 60, false);
            key(&mut e, 64, false);
            run(&mut e, secs(0.1));
            key(&mut e, 62, true);
            run(&mut e, secs(0.05));
            assert!(
                (e.lfo_readings()[0].since - 0.05).abs() < 0.002,
                "from silence"
            );
        }
    }
}

/// A mixed patch can restart its vibrato while its phrase ENV and FREE source carry on.
/// Both shared and voice-owned generators, including UNISON, follow their own mode.
#[test]
fn only_retrig_sources_restart_on_a_legato_key() {
    for unison in [false, true] {
        for mono in [false, true] {
            for retrigger in [false, true] {
                let mut c = controls();
                c.unison = unison;
                c.voices = 2;
                c.panel.retrigger = retrigger;
                for (j, mode) in [Mode::Retrig, Mode::Env, Mode::Free, Mode::Retrig]
                    .into_iter()
                    .enumerate()
                {
                    let s = Settings {
                        mono,
                        fade: 1.0,
                        phase: 0.15,
                        ..sharp(mode, 2.0, 1.0)
                    };
                    c = with(c, j, s, Target::Tune, 0.001, 0.5);
                }
                let mut e = engine(&c);
                key(&mut e, 60, true);
                run(&mut e, secs(0.3));
                key(&mut e, 64, true);
                run(&mut e, secs(0.1));
                for (j, r) in e.lfo_readings().iter().enumerate() {
                    let reset = retrigger || matches!(j, 0 | 3);
                    let since = if reset { 0.1 } else { 0.4 };
                    let phase = if reset && j != 2 { 0.25 } else { 0.55 };
                    assert!(
                        (r.since - since).abs() < 0.002
                            && (r.fade - since).abs() < 0.002
                            && (r.phase - phase).abs() < 0.002,
                        "unison {unison}, mono {mono}, main retrigger {retrigger}, source {j}: {r:?}"
                    );
                }
            }
        }
    }
}

/// A FADE out (decisions.md R-LFO-FADE) starts where RISE started, and by the same rules: at
/// full before any note, however long; from full at each note-on for RETRIG, whatever the main
/// RETRIGGER; for FREE and ENV as main RETRIGGER says (off, a key from silence; on, every key);
/// each POLY voice's own (MONO off) at the note it takes. Once over, the LFO gives nothing: its
/// knob is where it is set.
#[test]
fn a_fade_out_falls_from_each_note_by_rises_rules() {
    let out = |mode, mono| Settings {
        mono,
        fade: -1.0,
        phase: 0.15,
        ..sharp(mode, 2.0, 1.0)
    };
    // Before any note: full, the waveform as it is.
    let c = with(
        controls(),
        0,
        out(Mode::Free, true),
        Target::Cutoff,
        0.5,
        0.5,
    );
    let mut e = engine(&c);
    run(&mut e, secs(1.5));
    let r = e.lfo_readings()[0];
    assert_eq!(r.fade, 1.0, "no note yet: a FADE out at full");
    assert!(r.value != 0.0 && r.moving, "{r:?}");
    // Each mode, each copy, against main RETRIGGER.
    for unison in [false, true] {
        for mono in [false, true] {
            for retrigger in [false, true] {
                let mut c = controls();
                c.unison = unison;
                c.voices = 2;
                c.panel.retrigger = retrigger;
                for (j, mode) in [Mode::Retrig, Mode::Env, Mode::Free, Mode::Retrig]
                    .into_iter()
                    .enumerate()
                {
                    c = with(c, j, out(mode, mono), Target::Tune, 0.001, 0.5);
                }
                let mut e = engine(&c);
                key(&mut e, 60, true);
                run(&mut e, secs(0.3));
                for r in e.lfo_readings() {
                    assert!((r.fade - 0.7).abs() < 0.002, "{r:?}");
                }
                key(&mut e, 64, true);
                run(&mut e, secs(0.1));
                for (j, r) in e.lfo_readings().iter().enumerate() {
                    let reset = retrigger || matches!(j, 0 | 3);
                    let since = if reset { 0.1 } else { 0.4 };
                    assert!(
                        (r.since - since).abs() < 0.002 && (r.fade - (1.0 - since)).abs() < 0.002,
                        "unison {unison}, mono {mono}, main retrigger {retrigger}, source {j}: {r:?}"
                    );
                }
                // From silence, every one again.
                key(&mut e, 60, false);
                key(&mut e, 64, false);
                run(&mut e, secs(0.1));
                key(&mut e, 62, true);
                run(&mut e, secs(0.05));
                for r in e.lfo_readings() {
                    assert!((r.fade - 0.95).abs() < 0.002, "from silence: {r:?}");
                }
                // Over: nothing.
                run(&mut e, secs(1.0));
                for (j, r) in e.lfo_readings().iter().enumerate() {
                    assert_eq!(r.fade, 0.0, "source {j}");
                    assert_eq!(r.value, 0.0, "source {j}");
                }
            }
        }
    }
}

/// A POLY voice's own LFO (MONO off) fading out starts from full at the note the voice takes,
/// whatever the other voices' fades have reached: each voice's knob moved by its own.
#[test]
fn each_poly_voice_fades_out_from_its_own_note() {
    let mut c = controls();
    c.poly = true;
    c.voices = 4;
    let s = Settings {
        mono: false,
        fade: -1.0,
        ..sharp(Mode::Free, 3.0, 0.01)
    };
    let c = with(c, 0, s, Target::Cutoff, 0.4, 0.5);
    let mut e = engine(&c);
    key(&mut e, 60, true);
    run(&mut e, secs(0.6));
    key(&mut e, 67, true);
    run(&mut e, secs(0.2));
    // SQUARE at its top (1): the first voice's fade at 0.2, the second's at 0.8.
    let (a, b) = (panel(&e, slot_of(&e, 60)), panel(&e, slot_of(&e, 67)));
    assert!((a.cutoff - (0.5 + 0.4 * 0.2)).abs() < 0.01, "{}", a.cutoff);
    assert!((b.cutoff - (0.5 + 0.4 * 0.8)).abs() < 0.01, "{}", b.cutoff);
    run(&mut e, secs(1.0));
    let (a, b) = (panel(&e, slot_of(&e, 60)), panel(&e, slot_of(&e, 67)));
    assert!(
        (a.cutoff - 0.5).abs() < 1e-9 && (b.cutoff - 0.5).abs() < 1e-9,
        "both over: as set, {} {}",
        a.cutoff,
        b.cutoff
    );
}

/// Restarting an individual LFO must not open the physical trigger contact or restart the
/// loudness/filter contours. Compare their actual states with the same legato performance
/// without an assignment, throughout the transition, at LO and HI.
#[test]
fn individual_retrig_keeps_the_main_contours_legato() {
    for potato in [false, true] {
        let mut c = Controls {
            potato,
            ..controls()
        };
        c.panel.retrigger = false;
        c.panel.priority = ca72::voice::Priority::Last;
        let set = with(
            c,
            0,
            Settings {
                fade: 0.9,
                ..sharp(Mode::Retrig, 0.0, 4.2)
            },
            Target::Tune,
            0.012,
            0.5,
        );
        let (mut plain, mut vibrato) = (engine(&c), engine(&set));
        for key_number in [60, 64, 67] {
            key(&mut plain, key_number, true);
            key(&mut vibrato, key_number, true);
            for _ in 0..100 {
                run(&mut plain, 48);
                run(&mut vibrato, 48);
                let envelopes = |e: &Engine| {
                    let p = e.slots[0].play.lock().expect("idle voice");
                    if potato {
                        p.light.contours()
                    } else {
                        let (f, l, _) = p.voice.envelopes();
                        (f, l)
                    }
                };
                assert_eq!(envelopes(&plain), envelopes(&vibrato));
            }
            assert!(vibrato.lfo_readings()[0].since < 0.102);
        }
    }
}

/// ENV runs one cycle from PHASE after a note, then holds its last value, no longer moving; it
/// does not move before a note. FREE runs on regardless.
#[test]
fn env_runs_one_cycle_and_holds() {
    let s = Settings {
        phase: 0.25,
        ..sharp(Mode::Env, 2.0, 4.0)
    };
    let c = with(controls(), 0, s, Target::Cutoff, 0.5, 0.5);
    let mut e = engine(&c);
    run(&mut e, secs(0.1));
    assert!(!e.lfo_readings()[0].moving, "before a note");
    key(&mut e, 60, true);
    run(&mut e, secs(0.1));
    let r = e.lfo_readings()[0];
    assert!(r.moving);
    assert!((r.phase - (0.25 + 0.4)).abs() < 0.003, "{r:?}");
    run(&mut e, secs(0.3));
    let held = e.lfo_readings()[0];
    assert!(!held.moving);
    // (The sawtooth's last value from 0.25: at 0.25 again, just short: -0.5.)
    assert!((held.value - lfo::wave(2.0, 0.25 - 1e-9) as f32).abs() < 1e-6);
    run(&mut e, secs(0.2));
    assert_eq!(e.lfo_readings()[0].value, held.value);
}

/// Timed, ENV's one cycle is one note value long at the tempo: an eighth at 120 BPM, 0.25 s.
#[test]
fn a_timed_env_runs_one_note_value() {
    let s = Settings {
        timed: Some(Length::Beats(0.5)),
        ..sharp(Mode::Env, 2.0, 1.0)
    };
    let c = with(controls(), 0, s, Target::Cutoff, 0.5, 0.5);
    let mut e = engine(&c);
    e.set_position(Position::start(120.0, RATE));
    key(&mut e, 60, true);
    run(&mut e, secs(0.2));
    let r = e.lfo_readings()[0];
    assert!(r.moving && (r.phase - 0.8).abs() < 0.01, "{r:?}");
    run(&mut e, secs(0.1));
    assert!(!e.lfo_readings()[0].moving);
}

/// MONO off in POLY: each voice's LFO starts with its own note, and moves its own CUTOFF; a
/// shared LFO on the same knob adds the same to every voice; the scope shows the latest voice's.
#[test]
fn each_voice_has_its_own_lfo_with_mono_off() {
    let mut c = controls();
    (c.poly, c.voices) = (true, 4);
    let own = Settings {
        mono: false,
        ..sharp(Mode::Retrig, 2.0, 1.0)
    };
    let c = with(c, 0, own, Target::Cutoff, 0.25, 0.5);
    let c = with(c, 1, sharp(Mode::Free, 0.0, 3.0), Target::Cutoff, 0.1, 0.5);
    let mut e = engine(&c);
    key(&mut e, 60, true);
    run(&mut e, secs(0.25));
    key(&mut e, 64, true);
    run(&mut e, secs(0.1));
    let (a, b) = (slot_of(&e, 60), slot_of(&e, 64));
    assert_ne!(a, b);
    let mut cutoffs = [0.0; 2];
    for (j, k) in [a, b].into_iter().enumerate() {
        let own = e.bank.voice_values[k][0];
        let shared = e.bank.shared_values[1];
        let want = (0.5 + 0.25 * own + 0.1 * shared).clamp(0.0, 1.0);
        let got = panel(&e, k).cutoff;
        assert!((got - want).abs() < 1e-6, "voice {k}: {got} {want}");
        cutoffs[j] = got;
    }
    // (A quarter of a cycle of the sawtooth apart: half its swing, a quarter of 0.25 × 2.)
    assert!(
        (cutoffs[0] - cutoffs[1] - 0.125).abs() < 0.002,
        "{cutoffs:?}"
    );
    assert_eq!(e.bank.latest, b);
    assert!((e.lfo_readings()[0].since - 0.1).abs() < 0.002);
}

/// Option C: an LFO with MONO off on a knob the voices share (VOLUME) takes its shared copy.
#[test]
fn a_shared_target_takes_the_shared_copy() {
    let mut c = controls();
    (c.poly, c.voices) = (true, 4);
    let own = Settings {
        mono: false,
        ..sharp(Mode::Retrig, 2.0, 1.0)
    };
    let c = with(c, 0, own, Target::Volume, -0.5, 1.0);
    let mut e = engine(&c);
    key(&mut e, 60, true);
    run(&mut e, secs(0.2));
    key(&mut e, 64, true);
    run(&mut e, secs(0.13));
    let want = (1.0 - 0.5 * e.bank.shared_values[0]).clamp(0.0, 1.0);
    assert!((e.controls.volume - want).abs() < 1e-9);
    assert!(!c.lfo.per_voice(), "nothing of each voice's own");
}

/// Timed and FREE, the LFO's phase is the host's place every block: ten minutes of blocks at
/// 123.7 BPM, an eighth triplet from a quarter cycle on, within 1e-6 of a cycle at the end.
#[test]
fn a_timed_lfo_keeps_the_hosts_place_for_ten_minutes() {
    let q = 1.0 / 3.0;
    let s = Settings {
        timed: Some(Length::Beats(q)),
        phase: 0.25,
        ..sharp(Mode::Free, 2.0, 1.0)
    };
    let c = with(controls(), 0, s, Target::Cutoff, 0.3, 0.5);
    let mut e = engine(&c);
    let tempo = 123.7;
    let per = crate::timed::per_sample(tempo, RATE);
    const BLOCK: usize = 512;
    let blocks = (600.0 * RATE) as usize / BLOCK;
    let mut worst = 0.0f64;
    for b in 0..blocks {
        let beats = (b * BLOCK) as f64 * per;
        e.set_position(Position {
            beats,
            bars: beats / 4.0,
            per,
            bar: 4.0,
        });
        let (mut l, mut r) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
        e.render(&[], &mut l, &mut r);
        e.end_block(BLOCK);
        if b % 97 == 0 || b == blocks - 1 {
            // (The last step began 32 samples before the block's end.)
            let at = beats + (BLOCK - lfo::STEP) as f64 * per;
            let want = (at / q + 0.25).rem_euclid(1.0);
            let got = f64::from(e.lfo_readings()[0].phase);
            let d = (got - want).abs();
            worst = worst.max(d.min(1.0 - d));
        }
    }
    assert!(worst < 1e-6, "{worst}");
}

/// An assigned LFO running never keeps the one instrument awake, nor wakes it (R68).
#[test]
fn an_lfo_neither_keeps_nor_wakes_the_sleeping_instrument() {
    let c = with(
        controls(),
        0,
        sharp(Mode::Free, 0.0, 9.0),
        Target::Cutoff,
        0.5,
        0.5,
    );
    let mut e = engine(&c);
    key(&mut e, 60, true);
    run(&mut e, secs(0.2));
    key(&mut e, 60, false);
    let mut t = 0.0;
    while !e.asleep() && t < 30.0 {
        run(&mut e, secs(0.1));
        t += 0.1;
    }
    assert!(e.asleep(), "kept awake");
    assert!(run(&mut e, secs(1.0)).iter().all(|y| *y == 0.0));
    assert!(e.asleep(), "woken");
    key(&mut e, 60, true);
    assert!(!e.asleep());
    run(&mut e, secs(0.1));
    let want = 0.5 + 0.5 * e.bank.shared_values[0];
    assert!(
        (panel(&e, 0).cutoff - want).abs() < 1e-6,
        "the knob where the LFO is, woken"
    );
}

/// What the LFOs cost: ten POLY voices held, every LFO MONO off on eight knobs each voice has,
/// against none (the best of three; release builds: `cargo test --release -- --ignored
/// lfo_cost --nocapture`).
#[test]
#[ignore = "a measurement"]
fn lfo_cost() {
    let targets = [
        Target::Cutoff,
        Target::Emphasis,
        Target::Osc2Frequency,
        Target::Osc1Volume,
        Target::Osc2Volume,
        Target::ContourAmount,
        Target::FilterSustain,
        Target::Tune,
    ];
    for (name, potato) in [("LO", true), ("HI", false)] {
        let mut base = Controls {
            potato,
            ..controls()
        };
        (base.poly, base.voices, base.entropy) = (true, 10, 0.05);
        let mut lfos = base;
        for k in 0..lfo::LFOS {
            let s = Settings {
                mono: false,
                ..sharp(Mode::Retrig, k as f64, 3.0 + k as f64)
            };
            for t in targets {
                lfos = with(lfos, k, s, t, 0.05, 0.5);
            }
        }
        let time = |c: &Controls| {
            (0..3)
                .map(|_| {
                    let mut e = engine(c);
                    for n in 0..10u8 {
                        key(&mut e, 48 + 3 * n, true);
                    }
                    run(&mut e, secs(0.2));
                    let t = Instant::now();
                    run(&mut e, secs(2.0));
                    t.elapsed().as_secs_f64() / 2.0 * 1e3
                })
                .fold(f64::MAX, f64::min)
        };
        let (off, on) = (time(&base), time(&lfos));
        println!(
            "{name}: {off:.1} ms a second of audio without LFOs, {on:.1} with (+{:.1} %)",
            (on / off - 1.0) * 100.0
        );
    }
}

/// The effects' pins (decisions.md R-FX-PARAMS): an LFO assigned to one gives the block a point
/// every 32 samples, at its frame, its offset depth × the LFO's shared value; the synth's
/// sound as without it, to the bit; and none is recorded where nothing moves a pin.
#[test]
fn an_effects_pin_gets_a_point_every_step() {
    let s = sharp(Mode::Free, 0.0, 5.0);
    let plain = with(controls(), 0, s, Target::Cutoff, 0.0, 0.5);
    let pinned = with(controls(), 0, s, Target::Fx(3), 0.5, 0.0);
    let (mut a, mut b) = (engine(&plain), engine(&pinned));
    let pos = crate::timed::Position::start(crate::timed::FALLBACK_TEMPO, RATE);
    let (mut l, mut r) = ([0.0f32; 256], [0.0f32; 256]);
    let (mut l2, mut r2) = ([0.0f32; 256], [0.0f32; 256]);
    key(&mut a, 60, true);
    key(&mut b, 60, true);
    for _ in 0..4 {
        a.set_position(pos);
        b.set_position(pos);
        a.render(&[], &mut l, &mut r);
        b.render(&[], &mut l2, &mut r2);
        a.end_block(256);
        b.end_block(256);
        assert_eq!(
            l.map(f32::to_bits),
            l2.map(f32::to_bits),
            "the synth as without it"
        );
        let p = b.fx_points();
        assert_eq!((p.n, p.moved), (8, 1 << 3));
        assert_eq!(&p.at[..8], &[0, 32, 64, 96, 128, 160, 192, 224]);
        assert!(
            p.offsets[..8]
                .iter()
                .all(|o| o[3].abs() <= 0.5 && o[0] == 0.0)
        );
        assert!(p.offsets[..8].iter().any(|o| o[3] != 0.0), "it moves");
        assert_eq!(a.fx_points().n, 0, "nothing moves a pin: no point");
    }
}

/// Every knob steps every `lfo::STEP` samples at every QUALITY (decisions.md R-LFO-STRESS):
/// after 32 samples from a SAWTOOTH's start, the knob is where the LFO was at the step's first
/// sample. A pitch or CUTOFF the voice glides across the step, in what the circuit (HI, ULTRA)
/// or the light voice (LO) reads after the knob's law, to where the LFO is at the next step's
/// first sample; any other knob it holds.
#[test]
fn a_pitch_or_cutoff_glides_across_each_step() {
    const HZ: f64 = 40.0;
    for (potato, ultra, target, glides) in [
        (false, false, Target::Cutoff, true),
        (false, true, Target::Cutoff, true),
        (false, false, Target::Tune, true),
        (false, false, Target::Osc2Frequency, true),
        (false, true, Target::Osc3Frequency, true),
        (true, false, Target::Cutoff, true),
        (true, false, Target::Tune, true),
        (false, false, Target::Volume, false),
        (false, false, Target::Emphasis, false),
    ] {
        let c = with(
            Controls {
                potato,
                ultra,
                ..controls()
            },
            0,
            sharp(Mode::Free, 2.0, HZ),
            target,
            0.4,
            0.5,
        );
        let mut e = engine(&c);
        key(&mut e, 60, true);
        let (mut l, mut r) = ([0.0f32; lfo::STEP], [0.0f32; lfo::STEP]);
        e.render(&[], &mut l, &mut r);
        let at = |i: usize| {
            let value = lfo::wave(2.0, i as f64 * HZ / RATE);
            c.lfo
                .modulated(&c, &c.lfo.source_values(&[value, 0.0, 0.0, 0.0], 1.0))
        };
        let (now, next) = (at(0), at(lfo::STEP));
        let what = format!("{target:?}, potato {potato}, ultra {ultra}");
        assert_eq!(
            (e.controls.panel, e.controls.volume),
            (now.panel, now.volume),
            "{what}"
        );
        let sweep = e.slots[0].play.try_lock().expect("free").sweep;
        assert_eq!(sweep.on, glides, "{what}");
        if !glides {
            continue;
        }
        assert_eq!(sweep.at, lfo::STEP as f64, "{what}");
        let end = |x: f64, step: f64| x + step * lfo::STEP as f64;
        let close = |a: f64, b: f64| assert!((a - b).abs() < 1e-12, "{what}: {a} against {b}");
        let (a, b) = (&now.panel, &next.panel);
        if potato {
            let ((pa, ca), (pb, cb)) = (ca72::light::swept_laws(a), ca72::light::swept_laws(b));
            for o in 0..3 {
                close(end(pa[o] * 100.0, sweep.cents[o]), pb[o] * 100.0);
            }
            close(end(ca, sweep.current), cb);
        } else {
            let s = sweep.circuit;
            close(
                end(cutoff_track(a.cutoff), s.cutoff),
                cutoff_track(b.cutoff),
            );
            close(end(a.tune, s.tune), b.tune);
            close(end(a.osc[1].freq, s.freq[0]), b.osc[1].freq);
            close(end(a.osc[2].freq, s.freq[1]), b.osc[2].freq);
        }
    }
}

/// A step a SHARP edge falls in is held, not glided through (decisions.md R-LFO-STRESS): a
/// SAWTOOTH on CUTOFF holds the steps its cycles restart in, and glides across every other.
#[test]
fn a_sharp_edge_holds_its_step() {
    // (Its edges never on a step's first sample, which rounding could put either side.)
    const HZ: f64 = 37.0;
    for potato in [false, true] {
        let c = with(
            Controls {
                potato,
                ..controls()
            },
            0,
            sharp(Mode::Free, 2.0, HZ),
            Target::Cutoff,
            0.4,
            0.5,
        );
        let mut e = engine(&c);
        key(&mut e, 60, true);
        let (mut l, mut r) = ([0.0f32; lfo::STEP], [0.0f32; lfo::STEP]);
        let mut held = 0;
        for step in 0..240 {
            e.render(&[], &mut l, &mut r);
            let at = |i: usize| lfo::wave(2.0, i as f64 * HZ / RATE);
            let edge = (at(step * lfo::STEP) - at((step + 1) * lfo::STEP)).abs() > 0.5;
            let on = e.slots[0].play.try_lock().expect("free").sweep.on;
            assert_eq!(on, !edge, "step {step}, potato {potato}");
            held += usize::from(edge);
        }
        assert!(held >= 5, "{held} edges");
    }
}

/// VOLUME moved by an LFO reaches the output a sample at a time across each step, in a straight
/// line from where the gain was to where the step puts it (decisions.md R-LFO-STRESS): EDGES
/// SOFT's slew is then smooth, not a few clicks. Moved by SHARP LFOs alone, or not moved (an LFO
/// on CUTOFF), the gain is the step's at once, as before.
#[test]
fn volume_moved_by_an_lfo_ramps_a_sample_at_a_time() {
    for (target, edges, ramps) in [
        (Target::Volume, Edges::Soft, true),
        (Target::Volume, Edges::Sharp, false),
        (Target::Cutoff, Edges::Soft, false),
    ] {
        let s = Settings {
            edges,
            ..sharp(Mode::Free, 3.0, 6.0)
        };
        let c = with(controls(), 0, s, target, -0.3, 0.4);
        let mut e = engine(&c);
        key(&mut e, 60, true);
        e.fade = 1.0;
        // (A step first: SOFT's slew has had no time at the first, SHARP's none to take.)
        e.modulate(lfo::STEP);
        for _ in 0..lfo::STEP {
            e.faded(1.0, 1.0);
        }
        let was = e.gain;
        e.modulate(lfo::STEP);
        let now = e.gain;
        let out: Vec<f64> = (0..lfo::STEP)
            .map(|_| f64::from(e.faded(1.0, 1.0).0))
            .collect();
        let name = format!("{target:?} {edges:?}");
        if !ramps {
            assert!(out.iter().all(|g| *g == f64::from(now as f32)), "{name}");
            continue;
        }
        assert!(now < was, "the SQUARE's first half turns VOLUME down");
        let per = (now - was) / lfo::STEP as f64;
        for (k, g) in out.iter().enumerate() {
            let want = was + per * (k + 1) as f64;
            assert!((g - want).abs() < 1e-6, "sample {k}: {g} against {want}");
        }
        assert_eq!(*out.last().unwrap(), f64::from(now as f32));
        // (The next step's samples: the gain where the step left it.)
        assert_eq!(f64::from(e.faded(1.0, 1.0).0), f64::from(now as f32));
    }
}

/// The glide's end is where the next step puts the knob, to the last bit or two
/// (decisions.md R-LFO-STRESS), on either of the LFOs' paths: where one LFO moves another's
/// RATE (the generators on their ticks, R-LFO's addendum), with a macro on the same knob, whose
/// part is in both ends, and with the note's velocity on it (each voice's own, R-VELOCITY). A
/// macro alone on CUTOFF holds still between steps: no glide.
#[test]
fn the_glide_ends_where_the_next_step_puts_the_knob() {
    use crate::lfo::{LfoKnob, Source};
    for potato in [false, true] {
        let base = Controls {
            potato,
            ..controls()
        };
        // LFO 1, a TRIANGLE at 3 Hz on CUTOFF; LFO 2 at 0.5 Hz moving LFO 1's RATE.
        let tri = |hz| Settings {
            edges: Edges::Soft,
            ..sharp(Mode::Free, 0.0, hz)
        };
        let mut cross = with(base, 0, tri(3.0), Target::Cutoff, 0.4, 0.5);
        cross = with(
            cross,
            1,
            tri(0.5),
            Target::of_lfo(0, LfoKnob::Rate),
            0.3,
            0.0,
        );
        cross.lfo.base[Target::of_lfo(0, LfoKnob::Rate).index()] = lfo::RATE_RANGE.normalize(3.0);
        let mut macro_too = cross;
        macro_too.lfo.to[Source::Macro(0).index()][0] = Some(Assignment {
            target: Target::Cutoff,
            depth: 0.2,
        });
        macro_too.lfo.macros[0] = 0.5;
        let mut macro_alone = base;
        macro_alone.lfo.to[Source::Macro(0).index()][0] =
            macro_too.lfo.to[Source::Macro(0).index()][0];
        macro_alone.lfo.base[Target::Cutoff.index()] = 0.5;
        macro_alone.lfo.macros[0] = 0.5;
        let mut velocity_too = cross;
        velocity_too.lfo.to[Source::Velocity.index()][0] = Some(Assignment {
            target: Target::Cutoff,
            depth: 0.3,
        });
        for (what, c, glides) in [
            ("an LFO moving an LFO", cross, true),
            ("and a macro", macro_too, true),
            ("and the velocity", velocity_too, true),
            ("a macro alone", macro_alone, false),
        ] {
            let what = format!("{what}, potato {potato}");
            let mut e = engine(&c);
            e.event(Event::NoteVelocity {
                key: 60,
                velocity: 0.3,
            });
            key(&mut e, 60, true);
            let (mut l, mut r) = ([0.0f32; lfo::STEP], [0.0f32; lfo::STEP]);
            e.render(&[], &mut l, &mut r);
            assert_eq!(e.lfo_cross().on, glides, "{what}: on the ticks");
            for step in 0..60 {
                let (now, sweep) = (
                    panel(&e, 0),
                    e.slots[0].play.try_lock().expect("free").sweep,
                );
                e.render(&[], &mut l, &mut r);
                assert_eq!(sweep.on, glides, "{what}, step {step}");
                if !glides {
                    continue;
                }
                let next = panel(&e, 0);
                assert_ne!(now.cutoff, next.cutoff, "{what}, step {step}: it moves");
                let end = |x: f64, d: f64| x + d * lfo::STEP as f64;
                let close = |a: f64, b: f64| {
                    assert!(
                        (a - b).abs() < 1e-12,
                        "{what}, step {step}: {a} against {b}"
                    )
                };
                if potato {
                    let (ca, cb) = (
                        ca72::light::swept_laws(&now).1,
                        ca72::light::swept_laws(&next).1,
                    );
                    close(end(ca, sweep.current), cb);
                } else {
                    close(
                        end(cutoff_track(now.cutoff), sweep.circuit.cutoff),
                        cutoff_track(next.cutoff),
                    );
                }
            }
        }
    }
}
