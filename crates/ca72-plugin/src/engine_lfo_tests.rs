//! The LFOs in the engine (decisions.md R-LFO): nothing assigned is the engine as before to the
//! bit; an assigned knob moved by depth × LFO after its glide, clamped to its travel, through
//! its law; RETRIG, ENV and RISE against RETRIGGER; each voice its own with MONO off, the
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

/// With no assignment, or every depth 0, the LFOs' settings (MONO off, RETRIG, RISE, timed)
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
                rise: 0.3,
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

/// RETRIG starts again with a new note: with RETRIGGER off, a key from silence (a legato key
/// does not); with it on, every key. So for RISE. The one instrument and POLY's shared LFO.
#[test]
fn retrig_and_rise_start_again_with_a_new_note_as_retrigger_says() {
    for poly in [false, true] {
        for retrigger in [false, true] {
            let mut c = controls();
            c.poly = poly;
            c.panel.retrigger = retrigger;
            let s = Settings {
                rise: 1.0,
                ..sharp(Mode::Retrig, 2.0, 1.0)
            };
            let c = with(c, 0, s, Target::Cutoff, 0.5, 0.5);
            let mut e = engine(&c);
            assert_eq!(
                e.lfo_readings()[0].rise,
                0.0,
                "no note yet: RISE at nothing"
            );
            key(&mut e, 60, true);
            run(&mut e, secs(0.3));
            let r = e.lfo_readings()[0];
            assert!((r.since - 0.3).abs() < 0.002, "{r:?}");
            assert!((r.rise - 0.3).abs() < 0.002 && (r.phase - 0.3).abs() < 0.002);
            key(&mut e, 64, true);
            run(&mut e, secs(0.1));
            let r = e.lfo_readings()[0];
            let since = if retrigger { 0.1 } else { 0.4 };
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
