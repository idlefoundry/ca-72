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

/// A pitch or CUTOFF moved at HI or ULTRA steps every `lfo::FAST_STEP` samples; at LO, and any
/// other knob anywhere, every `lfo::STEP` (decisions.md R-LFO-STRESS): after 32 samples from a
/// SAWTOOTH's start, the knob is where the LFO was at the last step's first sample.
#[test]
fn a_pitch_or_cutoff_steps_finer_at_hi_and_ultra() {
    const HZ: f64 = 40.0;
    for (potato, ultra, target, last) in [
        (false, false, Target::Cutoff, 24),
        (false, true, Target::Cutoff, 24),
        (false, false, Target::Tune, 24),
        (false, false, Target::Osc2Frequency, 24),
        (true, false, Target::Cutoff, 0),
        (false, false, Target::Volume, 0),
        (false, false, Target::Emphasis, 0),
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
        let (mut l, mut r) = ([0.0f32; 32], [0.0f32; 32]);
        e.render(&[], &mut l, &mut r);
        let value = lfo::wave(2.0, f64::from(last) * HZ / RATE);
        let want = c
            .lfo
            .modulated(&c, &lfo::source_values(&[value, 0.0, 0.0, 0.0]));
        assert_eq!(
            (e.controls.panel, e.controls.volume),
            (want.panel, want.volume),
            "{target:?}, potato {potato}, ultra {ultra}"
        );
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
