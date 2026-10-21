//! The LFOs moving their own knobs and each other's (decisions.md R-LFO's addendum; the owner,
//! 2026-10-20: "Let's try B. It's unconventional, but it's interesting"): each LFO knob a
//! target; loops bounded and finite for ten minutes; the same trajectory at 44.1, 48 and
//! 96 kHz and any block size, live and offline; RETRIG starting a loop again exactly; and
//! their cost.

use super::*;
use crate::lfo::{
    self, Assignment, Edges, LEVEL_RANGE, LfoKnob, Mode, PHASE_RANGE, RATE_HZ, RATE_RANGE,
    RISE_RANGE, Settings, Source, TICKS_A_SECOND, Target,
};
use ca72::voice::{SHAPE_PLACES, shape_snapped};

/// LO's light voices (quick in debug), no ENTROPY.
fn controls() -> Controls {
    Controls {
        entropy: 0.0,
        potato: true,
        ..Controls::default()
    }
}

/// LFO `n`'s knob `k`'s travel (0..1) with its settings `s`, as its parameter's range has it.
fn travel(s: &Settings, k: LfoKnob) -> f32 {
    match k {
        LfoKnob::Rate => RATE_RANGE.normalize(s.hz as f32),
        LfoKnob::Phase => PHASE_RANGE.normalize((s.phase * 360.0) as f32),
        LfoKnob::Rise => RISE_RANGE.normalize(s.rise as f32),
        LfoKnob::Shape => (s.at / SHAPE_PLACES) as f32,
        LfoKnob::Level => LEVEL_RANGE.normalize((s.level * 100.0) as f32),
    }
}

/// `c` with source `from` moving target `t` by `depth`; `t`'s knob's travel where its settings
/// (an LFO's) or `base` (any other's) have it.
fn assign(mut c: Controls, from: Source, t: Target, depth: f32, base: f32) -> Controls {
    let slot = c.lfo.to[from.index()]
        .iter_mut()
        .find(|a| a.is_none())
        .expect("room");
    *slot = Some(Assignment { target: t, depth });
    c.lfo.base[t.index()] = match t.lfo_knob() {
        Some((n, k)) => travel(&c.lfo.lfo[n], k),
        None => base,
    };
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

fn engine(c: &Controls, rate: f64) -> Engine {
    let mut e = Engine::new();
    e.set(c);
    e.prepare(rate, 3);
    e
}

/// `n` samples in blocks of up to `block`.
fn run(e: &mut Engine, n: usize, block: usize) {
    let (mut l, mut r) = (vec![0.0f32; block], vec![0.0f32; block]);
    let mut left = n;
    while left > 0 {
        let k = left.min(block);
        e.render(&[], &mut l[..k], &mut r[..k]);
        e.end_block(k);
        left -= k;
    }
}

fn key(e: &mut Engine, key: u8, on: bool) {
    e.event(Event::Note { key, on });
}

/// Every LFO's value at the latest tick, as the LFOs read it (the shared copies'), and that
/// tick.
fn values(e: &Engine) -> (u64, [f64; lfo::LFOS]) {
    let x = e.lfo_cross();
    (x.latest(), std::array::from_fn(|k| x.value(k, None)))
}

/// A patch of loops: LFO 1's RATE moved by LFO 2 and LFO 2's by LFO 1 (a coupled pair), LFO
/// 3's PHASE by LFO 4 and LFO 4's by LFO 3, LFO 3 on its own RATE, LFO 4 on its own LEVEL,
/// macro 1 on LFO 2's RISE; each LFO on CUTOFF too. `mode` every LFO's.
fn looped(mode: Mode) -> Controls {
    let mut c = Controls {
        lfo: lfo::Lfos {
            lfo: [
                sharp(mode, 0.0, 3.0),
                sharp(mode, 2.0, 0.7),
                Settings {
                    edges: Edges::Soft,
                    ..sharp(mode, 1.0, 5.0)
                },
                sharp(mode, 3.0, 1.3),
            ],
            ..lfo::Lfos::default()
        },
        ..controls()
    };
    c.lfo.macros[0] = 0.6;
    for (from, t, d) in [
        (Source::Lfo(1), Target::LfoRate(0), 0.6),
        (Source::Lfo(0), Target::LfoRate(1), -0.5),
        (Source::Lfo(3), Target::LfoPhase(2), 0.7),
        (Source::Lfo(2), Target::LfoPhase(3), 0.4),
        (Source::Lfo(2), Target::LfoRate(2), 0.45),
        (Source::Lfo(3), Target::LfoLevel(3), -0.8),
        (Source::Macro(0), Target::LfoRise(1), 0.3),
    ] {
        c = assign(c, from, t, d, 0.0);
    }
    for k in 0..lfo::LFOS {
        c = assign(c, Source::Lfo(k), Target::Cutoff, 0.1, 0.5);
    }
    c
}

/// Each of an LFO's knobs takes a source, as its own knob would have it: RATE, PHASE, RISE,
/// the CONTINUOUS shape and LEVEL, through their parameters' ranges, clamped; RATE not while
/// timed, the shape not while STEPPED; LEVEL scaling every assignment of its LFO's; and with
/// none, the LFO as its knobs have it.
#[test]
fn each_lfo_knob_takes_a_source() {
    let base = Settings {
        level: 0.9,
        ..sharp(Mode::Free, 0.0, 2.0)
    };
    // (A note held, so that the instrument is awake and takes the knobs: R68.)
    let settled = |c: &Controls| {
        let mut e = engine(c, 48_000.0);
        key(&mut e, 60, true);
        run(&mut e, 4800, 256);
        e
    };
    let moved = |s: Settings, t: Target, depth: f32| {
        let mut c = controls();
        c.lfo.lfo[0] = s;
        c.lfo.macros[0] = 1.0;
        let c = assign(c, Source::Macro(0), t, depth, 0.0);
        settled(&c).lfo_cross().settings(0)
    };
    let s = moved(base, Target::LfoRate(0), 0.25);
    let want = RATE_RANGE.unnormalize(RATE_RANGE.normalize(2.0) + 0.25);
    assert!((s.hz - f64::from(want)).abs() < 1e-4, "RATE {}", s.hz);
    let s = moved(base, Target::LfoRate(0), 1.0);
    assert_eq!(s.hz, f64::from(RATE_HZ.1), "RATE clamped");
    let s = moved(base, Target::LfoPhase(0), 0.25);
    assert!((s.phase - 0.25).abs() < 1e-6, "PHASE {}", s.phase);
    let s = moved(base, Target::LfoRise(0), 0.5);
    assert!((s.rise - f64::from(RISE_RANGE.unnormalize(0.5))).abs() < 1e-6);
    let s = moved(base, Target::LfoLevel(0), -0.6);
    assert!((s.level - 0.3).abs() < 1e-6, "LEVEL {}", s.level);
    // The shape only while CONTINUOUS.
    let continuous = Settings {
        continuous: true,
        at: 0.0,
        ..base
    };
    let s = moved(continuous, Target::LfoShape(0), 0.5);
    assert_eq!(s.at, shape_snapped(0.5 * SHAPE_PLACES));
    assert_eq!(moved(base, Target::LfoShape(0), 0.5).at, 0.0, "STEPPED");
    // RATE not while timed.
    let timed = Settings {
        timed: Some(crate::timed::Length::Beats(1.0)),
        ..base
    };
    assert_eq!(moved(timed, Target::LfoRate(0), 0.5).hz, 2.0, "timed");
    // LEVEL scales its LFO's every assignment: CUTOFF at 0.5 by +0.5, at LEVEL 30 %.
    let mut c = controls();
    c.lfo.lfo[0] = base;
    c.lfo.macros[0] = 1.0;
    let c = assign(c, Source::Macro(0), Target::LfoLevel(0), -0.6, 0.0);
    let c = assign(c, Source::Lfo(0), Target::Cutoff, 0.5, 0.5);
    let e = settled(&c);
    let v = e.bank.shared_values[0];
    let x = e.lfo_cross().value(0, None);
    assert!(v.abs() > 0.01 && v.abs() <= 0.3 + 1e-9, "{v}");
    assert!(x.abs() <= 0.3 + 1e-9, "{x}");
    assert!((e.controls.panel.cutoff - (0.5 + 0.5 * v)).abs() < 1e-6);
    // With nothing on its knobs, the generator as before.
    let mut c = controls();
    c.lfo.lfo[0] = base;
    let c = assign(c, Source::Lfo(0), Target::Cutoff, 0.5, 0.5);
    assert!(!c.lfo.cross());
}

/// Two LFOs moving each other's RATE, and one its own (and its own LEVEL, and PHASE in a pair):
/// ten minutes on, every value within -1..1 and finite, every RATE within its range; and the
/// loop moves (its RATE does not settle).
#[test]
fn loops_stay_bounded_and_finite_for_ten_minutes() {
    let c = looped(Mode::Free);
    let mut e = engine(&c, 48_000.0);
    key(&mut e, 60, true);
    let block = 4096;
    let blocks = (600.0 * 48_000.0) as usize / block;
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for b in 0..blocks {
        if b == 20 {
            key(&mut e, 60, false);
        }
        run(&mut e, block, block);
        let x = e.lfo_cross();
        for k in 0..lfo::LFOS {
            let v = x.value(k, None);
            assert!(v.is_finite() && v.abs() <= 1.0, "LFO {k} at block {b}: {v}");
            let s = x.settings(k);
            assert!(
                s.hz.is_finite()
                    && (f64::from(RATE_HZ.0)..=f64::from(RATE_HZ.1)).contains(&s.hz)
                    && (0.0..1.0).contains(&s.phase)
                    && (0.0..=1.0).contains(&s.level),
                "LFO {k}: {s:?}"
            );
            let r = e.lfo_readings()[k];
            assert!(r.value.is_finite() && r.value.abs() <= 1.0, "{r:?}");
        }
        if b > blocks / 2 {
            let hz = x.settings(0).hz;
            (lo, hi) = (lo.min(hz), hi.max(hz));
        }
    }
    assert!(hi > lo * 1.2, "LFO 1's RATE still moving: {lo}..{hi}");
}

/// A looped patch's trajectory at 44.1, 48 and 96 kHz, in blocks of 64 and of 1024 samples: each
/// LFO's value, as the LFOs read it, the same to the bit at every 10 ms for three seconds, a
/// note starting and stopping between (its RETRIG LFOs starting again at the same tick).
#[test]
fn a_loop_runs_the_same_at_every_rate_and_block() {
    let mut c = looped(Mode::Free);
    c.lfo.lfo[1].mode = Mode::Retrig;
    c.lfo.lfo[3].mode = Mode::Env;
    let marks: Vec<f64> = (1..=300).map(|k| k as f64 * 0.01).collect();
    let notes = [(0.25, true), (1.0, false), (1.5, true), (2.2, false)];
    let trajectory = |rate: f64, block: usize| {
        let mut e = engine(&c, rate);
        let mut at = 0usize;
        let mut out = Vec::new();
        let mut events = notes.iter().peekable();
        for &t in &marks {
            let to = (t * rate).round() as usize;
            loop {
                let next = events.peek().map(|(s, _)| (s * rate).round() as usize);
                let stop = match next {
                    Some(n) if n <= to => n,
                    _ => to,
                };
                while at < stop {
                    let n = (stop - at).min(block - at % block);
                    run(&mut e, n, n);
                    at += n;
                }
                match next {
                    Some(n) if n <= to => {
                        let (_, on) = events.next().expect("an event");
                        key(&mut e, 60, *on);
                    }
                    _ => break,
                }
            }
            let (tick, v) = values(&e);
            assert_eq!(
                tick,
                (t * TICKS_A_SECOND).round() as u64,
                "{rate} {block} at {t}"
            );
            out.push(v.map(f64::to_bits));
        }
        out
    };
    let want = trajectory(48_000.0, 64);
    assert!(
        want.windows(2).filter(|w| w[0] != w[1]).count() > 250,
        "it moves"
    );
    for (rate, block) in [
        (48_000.0, 1024),
        (44_100.0, 64),
        (44_100.0, 1024),
        (96_000.0, 64),
        (96_000.0, 1024),
    ] {
        let got = trajectory(rate, block);
        let first = got.iter().zip(&want).position(|(a, b)| a != b);
        assert_eq!(
            first, None,
            "{rate} Hz in blocks of {block}: from mark {first:?}"
        );
    }
}

/// Live (each block with its deadline, paced at the rate) and offline (none): a looped patch's
/// LFOs the same to the bit, block by block, and so its sound.
#[test]
fn live_and_offline_play_a_loop_alike() {
    let c = looped(Mode::Free);
    let rate = 48_000.0;
    const BLOCK: usize = 256;
    let play = |live: bool| {
        let mut e = engine(&c, rate);
        let period = std::time::Duration::from_secs_f64(BLOCK as f64 / rate);
        let start = Instant::now();
        let (mut l, mut r) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
        let mut sound = Vec::new();
        let mut lfos = Vec::new();
        for b in 0..(2.0 * rate) as usize / BLOCK {
            if b % 40 == 0 {
                key(&mut e, 48 + (b / 40 % 12) as u8, true);
            }
            if live {
                let due = start + period * b as u32;
                if let Some(wait) = due.checked_duration_since(Instant::now()) {
                    std::thread::sleep(wait);
                }
                e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
            } else {
                e.set_deadline(None);
            }
            e.render(&[], &mut l, &mut r);
            e.end_block(BLOCK);
            sound.extend(l.iter().chain(&r).map(|x| x.to_bits()));
            lfos.push(values(&e).1.map(f64::to_bits));
        }
        (sound, lfos)
    };
    let (offline, live) = (play(false), play(true));
    assert_eq!(offline.1, live.1, "the LFOs");
    assert!(offline.0.iter().any(|x| f32::from_bits(*x).abs() > 1e-3));
    assert!(offline.0 == live.0, "the sound");
}

/// Every LFO of a loop RETRIG, either main RETRIGGER setting: each note starts the loop again, its every LFO's
/// value the same to the bit tick by tick after it, whatever came before and wherever in a tick
/// the note falls; a FREE LFO in the loop runs on (not the same).
#[test]
fn retrig_starts_a_loop_again_note_for_note() {
    // (Tick by tick, 24 samples at 48 kHz, each note where it falls; every tick recorded.)
    let record = |mode: Mode, retrigger: bool| {
        let mut c = looped(mode);
        c.panel.retrigger = retrigger;
        let mut e = engine(&c, 48_000.0);
        let notes = [14_462usize, 52_813, 98_407, 140_011];
        let end = 170_011usize;
        let mut by_tick = std::collections::HashMap::new();
        let mut at = 0;
        let mut next = notes.iter().peekable();
        while at < end {
            if next.peek().is_some_and(|n| **n == at) {
                let k = notes.len() - next.clone().count();
                key(&mut e, 50 + k as u8, true);
                next.next();
            }
            let to = next.peek().map_or(end, |n| (**n).min(end));
            let step = (to - at).min(24 - at % 24).max(1);
            run(&mut e, step, step);
            at += step;
            let (t, v) = values(&e);
            by_tick.insert(t, v.map(f64::to_bits));
        }
        let starts: Vec<u64> = notes
            .iter()
            .map(|&n| lfo::tick_at(n as u64, 48_000.0))
            .collect();
        (starts, by_tick)
    };
    let after = |starts: &[u64], by: &std::collections::HashMap<u64, [u64; 4]>, k: usize| {
        (1..=1200u64)
            .map(|m| {
                by.get(&(starts[k] + m))
                    .copied()
                    .expect("each tick recorded")
            })
            .collect::<Vec<_>>()
    };
    for retrigger in [false, true] {
        let (starts, by) = record(Mode::Retrig, retrigger);
        let first = after(&starts, &by, 0);
        assert!(
            first.windows(2).filter(|w| w[0] != w[1]).count() > 1000,
            "it moves"
        );
        for k in 1..starts.len() {
            assert!(
                after(&starts, &by, k) == first,
                "main retrigger {retrigger}, note {k} as note 0"
            );
        }
    }
    let (starts, by) = record(Mode::Free, false);
    assert!(
        after(&starts, &by, 1) != after(&starts, &by, 0),
        "FREE runs on"
    );
}

/// A voice's own LFO (MONO off) on another's knob acts a voice at a time: two POLY voices
/// started apart, LFO 2 (MONO off) moving LFO 1's RATE (MONO off), their LFO 1s apart; LFO 1
/// shared (MONO on), it takes LFO 2's shared copy, whatever the voices' (option C).
#[test]
fn a_voices_own_lfo_moves_its_own_lfos_knob() {
    let mut c = Controls {
        poly: true,
        voices: 2,
        ..controls()
    };
    c.lfo.lfo[0] = Settings {
        mono: false,
        ..sharp(Mode::Retrig, 0.0, 1.0)
    };
    c.lfo.lfo[1] = Settings {
        mono: false,
        ..sharp(Mode::Retrig, 2.0, 3.0)
    };
    let c = assign(c, Source::Lfo(1), Target::LfoRate(0), 0.5, 0.0);
    let c = assign(c, Source::Lfo(0), Target::Cutoff, 0.3, 0.5);
    let mut e = engine(&c, 48_000.0);
    key(&mut e, 60, true);
    run(&mut e, 4000, 256);
    key(&mut e, 67, true);
    run(&mut e, 3000, 256);
    let x = e.lfo_cross();
    let (a, b) = (x.value(1, Some(0)), x.value(1, Some(1)));
    assert!((a - b).abs() > 1e-3, "the voices' LFO 2s apart: {a} {b}");
    let rate = |v: usize| x.voice_settings(v, 0).hz;
    assert!((rate(0) - rate(1)).abs() > 1e-3, "{} {}", rate(0), rate(1));
    // Shared: LFO 1's one copy, by LFO 2's shared one.
    let mut s = c;
    s.lfo.lfo[0].mono = true;
    let mut e = engine(&s, 48_000.0);
    key(&mut e, 60, true);
    run(&mut e, 4000, 256);
    key(&mut e, 67, true);
    run(&mut e, 3000, 256);
    let x = e.lfo_cross();
    let want = RATE_RANGE.unnormalize(
        (f64::from(RATE_RANGE.normalize(1.0)) + 0.5 * x.value(1, None)).clamp(0.0, 1.0) as f32,
    );
    // (The value read is the tick before's: within a tick's movement of LFO 2.)
    assert!(
        (x.settings(0).hz - f64::from(want)).abs() < 0.1,
        "{}",
        x.settings(0).hz
    );
}

/// What the ticks cost: ten POLY voices held, every LFO MONO off moving its own RATE and the next
/// one's RATE, PHASE and LEVEL as well as four knobs each voice has, against the same four knobs
/// alone (the best of three; release builds: `cargo test --release -- --ignored lfo_cross_cost
/// --nocapture`).
#[test]
#[ignore = "a measurement"]
fn lfo_cross_cost() {
    let knobs = [
        Target::Cutoff,
        Target::Emphasis,
        Target::Osc2Frequency,
        Target::Osc1Volume,
    ];
    for (name, potato) in [("LO", true), ("HI", false)] {
        let mut base = Controls {
            potato,
            ..controls()
        };
        (base.poly, base.voices, base.entropy) = (true, 10, 0.05);
        for k in 0..lfo::LFOS {
            base.lfo.lfo[k] = Settings {
                mono: false,
                ..sharp(Mode::Retrig, k as f64, 3.0 + k as f64)
            };
        }
        let mut plain = base;
        for k in 0..lfo::LFOS {
            for t in knobs {
                plain = assign(plain, Source::Lfo(k), t, 0.05, 0.5);
            }
        }
        let mut cross = plain;
        for k in 0..lfo::LFOS {
            let next = (k + 1) % lfo::LFOS;
            for t in [
                Target::LfoRate(k),
                Target::LfoRate(next),
                Target::LfoPhase(next),
                Target::LfoLevel(next),
            ] {
                cross = assign(cross, Source::Lfo(k), t, 0.3, 0.0);
            }
        }
        let time = |c: &Controls| {
            (0..3)
                .map(|_| {
                    let mut e = engine(c, 48_000.0);
                    for n in 0..10u8 {
                        key(&mut e, 48 + 3 * n, true);
                    }
                    run(&mut e, 9600, 256);
                    let t = Instant::now();
                    run(&mut e, 96_000, 256);
                    t.elapsed().as_secs_f64() / 2.0 * 1e3
                })
                .fold(f64::MAX, f64::min)
        };
        let (off, on) = (time(&plain), time(&cross));
        println!(
            "{name}: {off:.1} ms a second of audio, four LFOs on four knobs a voice; {on:.1} with \
             each moving its own and the next one's knobs too (+{:.1} %)",
            (on / off - 1.0) * 100.0
        );
    }
}
