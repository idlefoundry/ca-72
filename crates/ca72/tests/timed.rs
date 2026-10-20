//! Oscillator 3 timed (decisions.md R-OSC-TIMED), the oscillators themselves: the circuit's
//! (HI and ULTRA) and the light voice's (LO) reset where their clock's cycles start, to a
//! small part of a sample, at any tempo, and follow a tempo changed or a jump of the clock.

use ca72::expo::ExpoCircuit;
use ca72::light::Light;
use ca72::tuning::Range;
use ca72::vco::{TIMED_EARLY, Timed, Vco};
use ca72::voice::{Panel, Waveform};

/// The timed ramp's bend is the converter's own at LO's currents: the timing current at -4 V
/// on the ramp under that at 0 V, as the circuit's oscillator solves it, from LO's slowest
/// to past its fastest.
#[test]
fn the_timed_ramp_bends_as_the_converter_does() {
    let e = ExpoCircuit::default();
    for i_in in [1e-10, 1e-8, 1e-6, 1e-5] {
        let top = e.solve_current(i_in, 0.0, 25.0, None).ic_exp;
        let bottom = e.solve_current(i_in, -4.0, 25.0, None).ic_exp;
        let slope = (top - bottom) / 4.0 / top;
        assert!(
            (slope / TIMED_EARLY - 1.0).abs() < 1e-3,
            "{i_in}: {slope} against {TIMED_EARLY}"
        );
    }
}

/// The clock's phase at host sample `s`: `c0` cycles at 0, `per` a sample.
fn clock(c0: f64, per: f64, s: u64) -> Timed {
    let at = c0 + s as f64 * per;
    Timed {
        phase: at - at.floor(),
        per,
    }
}

/// The circuit's oscillator at `rate` (four times oversampled, as HI and ULTRA run it) on a
/// clock, `phase(s)` at sample `s`, for `samples`: each reset's time (samples).
fn vco_resets(rate: f64, samples: u64, phase: impl Fn(u64) -> Timed) -> Vec<f64> {
    let mut v = Vco::new(rate, 4);
    v.start_at(0.3);
    let mut out = Vec::new();
    let mut seen = 0;
    for s in 0..samples {
        let o = v.tick_timed(phase(s), 0.0);
        assert!(o.saw.is_finite() && o.tri.is_finite() && o.rect.is_finite());
        let (ago, count) = v.reset_ago();
        if count > seen {
            seen = count;
            out.push(s as f64 - ago);
        }
    }
    out
}

/// The light voice's (LO) oscillator `n` at `rate`, a sawtooth at RANGE's LO, on a clock.
fn light_osc_resets(rate: f64, samples: u64, n: usize, phase: impl Fn(u64) -> Timed) -> Vec<f64> {
    let mut l = Light::new(rate);
    let mut p = Panel::default();
    p.osc[n].on = true;
    p.osc[n].range = Range::Lo;
    p.osc[n].waveform = Waveform::Sawtooth;
    l.set_panel(&p);
    l.start_oscillators_at(0.3);
    let mut out = Vec::new();
    let mut seen = 0;
    for s in 0..samples {
        let mut timed = [None; 3];
        timed[n] = Some(phase(s));
        l.set_timed(timed);
        assert!(l.tick(0.0).is_finite());
        let (ago, count) = l.probe_timed_reset(n);
        if count > seen {
            seen = count;
            out.push(s as f64 - ago);
        }
    }
    out
}

/// The light voice's oscillator 3.
fn light_resets(rate: f64, samples: u64, phase: impl Fn(u64) -> Timed) -> Vec<f64> {
    light_osc_resets(rate, samples, 2, phase)
}

/// The light voice's oscillators 1 and 2 timed as its 3 is: on the clock.
#[test]
fn the_light_voices_oscillators_1_and_2_reset_on_their_clocks() {
    let (rate, per) = (48_000.0, 123.7 / 60.0 / 48_000.0 / 0.25);
    for n in [0, 1] {
        let resets = light_osc_resets(rate, 60_000, n, |s| clock(0.37, per, s));
        assert_eq!(resets.len(), ((0.37 + 59_999.0 * per) as usize), "{n}");
        for &r in &resets {
            let m = (0.37 + r * per).round();
            assert!((r - (m - 0.37) / per).abs() < 0.1, "{n}: {r}");
        }
    }
}

/// At several tempos, divisions and sample rates, each reset is where its cycle starts: the
/// circuit's within a thousandth of a sample, the light voice's (its phase single precision)
/// within a tenth.
#[test]
fn the_oscillators_reset_where_their_clocks_cycles_start() {
    for (rate, bpm, beats) in [
        (48_000.0, 123.7, 0.25),
        (48_000.0, 300.0, 0.125),
        (48_000.0, 97.3, 1.0 / 6.0),
        (44_100.0, 30.0, 0.125),
        (96_000.0, 1000.0, 0.125),
        (48_000.0, 240.0, 3.0),
        (48_000.0, 180.0, 16.0),
    ] {
        let per = bpm / 60.0 / rate / beats;
        let c0 = 0.37;
        // Four cycles, or a few seconds of the slowest.
        let samples = ((4.2 / per) as u64).min((6.0 * rate) as u64);
        // (Cycle m starts at sample (m - c0) / per; the clock's count at r is c0 + r per.)
        let count = |r: f64| c0 + r * per;
        let at_m = |r: f64| {
            let m = count(r).round();
            (m - c0) / per
        };
        for (name, resets) in [
            ("circuit", vco_resets(rate, samples, |s| clock(c0, per, s))),
            ("light", light_resets(rate, samples, |s| clock(c0, per, s))),
        ] {
            let errors: Vec<f64> = resets.iter().map(|&r| (r - at_m(r)).abs()).collect();
            let worst = errors.iter().copied().fold(0.0, f64::max);
            let limit = if name == "circuit" { 1e-3 } else { 0.1 };
            assert!(
                worst < limit,
                "{name} {rate} Hz {bpm} BPM {beats}: {worst} samples ({errors:?})"
            );
            // Every cycle is there: none skipped, none doubled.
            let cycles = (count(samples as f64 - 1.0).floor() - count(0.0).floor()) as usize;
            assert_eq!(resets.len(), cycles, "{name} {rate} {bpm} {beats}");
        }
    }
}

/// A tempo changed in the middle of a cycle: the resets follow the clock's cycles as they
/// now fall, slower or faster, from where it was; and a tempo moved block by block (a ramp, as
/// a host gives it: its place at each block's start, the block's samples at its start's
/// tempo), each reset within a tenth of a sample of the ramp's own cycle starts.
#[test]
fn the_oscillators_follow_the_tempo_changed_mid_cycle() {
    let rate = 48_000.0;
    let at = |bpm: f64| bpm / 60.0 / rate / 0.5;
    // 120 then, half way through a cycle, 77.7, then 241.
    let (p1, p2, p3) = (at(120.0), at(77.7), at(241.0));
    let (n1, n2) = (30_000u64, 70_000u64);
    let count = move |s: f64| {
        if s < n1 as f64 {
            s * p1
        } else if s < n2 as f64 {
            n1 as f64 * p1 + (s - n1 as f64) * p2
        } else {
            n1 as f64 * p1 + (n2 - n1) as f64 * p2 + (s - n2 as f64) * p3
        }
    };
    let phase = move |s: u64| Timed {
        phase: count(s as f64) - count(s as f64).floor(),
        per: if s < n1 {
            p1
        } else if s < n2 {
            p2
        } else {
            p3
        },
    };
    // Where cycle m starts.
    let start = move |m: f64| {
        let (c1, c2) = (count(n1 as f64), count(n2 as f64));
        if m <= c1 {
            m / p1
        } else if m <= c2 {
            n1 as f64 + (m - c1) / p2
        } else {
            n2 as f64 + (m - c2) / p3
        }
    };
    for (name, resets) in [
        ("circuit", vco_resets(rate, 120_000, phase)),
        ("light", light_resets(rate, 120_000, phase)),
    ] {
        let limit = if name == "circuit" { 1e-3 } else { 0.1 };
        for &r in &resets {
            let m = count(r).round();
            let e = r - start(m);
            assert!(e.abs() < limit, "{name}: reset at {r}, cycle {m}: {e}");
        }
        // (Cycle 0 starts at sample 0, where the oscillator is first put on its clock: the
        // light voice's wrap counts it, the circuit's ramp is put at its top.)
        let after = resets.iter().filter(|&&r| r > 0.5).count();
        assert_eq!(after as f64, count(119_999.0).floor(), "{name}");
    }
    // The ramp: 90 to 150 BPM over five seconds, a host's 256-sample blocks.
    const BLOCK: u64 = 256;
    let bpm = |s: f64| 90.0 + 60.0 * (s / (5.0 * rate)).min(1.0);
    // The host's place (quarter notes) at sample s: the ramp's integral.
    let beats = move |s: f64| {
        let t = s.min(5.0 * rate);
        let ramp = (90.0 * t + 30.0 * t * t / (5.0 * rate)) / 60.0 / rate;
        ramp + (s - t) * 150.0 / 60.0 / rate
    };
    let given = move |s: u64| {
        let b = s - s % BLOCK;
        let per = bpm(b as f64) / 60.0 / rate / 0.25;
        let at = beats(b as f64) / 0.25 + (s - b) as f64 * per;
        Timed {
            phase: at - at.floor(),
            per,
        }
    };
    for (name, resets) in [
        ("circuit", vco_resets(rate, 300_000, given)),
        ("light", light_resets(rate, 300_000, given)),
    ] {
        // (The host says its place and tempo as each block begins: within a block the clock
        // runs straight on at the start's tempo, the ramp's own curve up to 0.09 samples from
        // it by the block's end at this ramp, 60 BPM in five seconds. The next block's start
        // is the host's place again.)
        let limit = 0.1;
        for &r in &resets {
            // The exact cycle start by bisection of the ramp's count.
            let m = (beats(r) / 0.25).round();
            let (mut lo, mut hi) = (r - 50.0, r + 50.0);
            for _ in 0..60 {
                let mid = 0.5 * (lo + hi);
                if beats(mid) / 0.25 < m {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            assert!(
                (r - lo).abs() < limit,
                "{name}: reset at {r}, cycle {m}: {}",
                r - lo
            );
        }
        assert!(resets.len() > 40, "{name}: {}", resets.len());
    }
}

/// The clock jumps (a loop, a locate, the transport started elsewhere): the oscillator is
/// where the clock is at once, and resets on its cycles from there; its outputs stay within
/// the circuit's swing through the jump (stepped, band-limited, as a reset steps them).
#[test]
fn the_oscillators_take_a_jump_of_their_clock_at_once() {
    let rate = 48_000.0;
    let per = 123.7 / 60.0 / rate / 1.0;
    let jump = 20_000u64;
    // From 0.1 cycles, then at sample 20000 back by 0.62 of a cycle (a loop's end).
    let count = move |s: u64| {
        let c = 0.1 + s as f64 * per;
        if s < jump { c } else { c - 0.62 }
    };
    let phase = move |s: u64| Timed {
        phase: count(s) - count(s).floor(),
        per,
    };
    let mut v = Vco::new(rate, 4);
    let mut l = Light::new(rate);
    let mut p = Panel::default();
    p.osc[2].on = true;
    p.osc[2].range = Range::Lo;
    l.set_panel(&p);
    let (mut seen, mut lseen) = (0, 0);
    let (mut resets, mut lresets) = (Vec::new(), Vec::new());
    for s in 0..80_000u64 {
        let o = v.tick_timed(phase(s), 0.0);
        assert!(
            o.saw.abs() < 6.0 && o.tri.abs() < 6.0 && o.rect.abs() < 6.0,
            "{s}: {o:?}"
        );
        l.set_timed([None, None, Some(phase(s))]);
        assert!(l.tick(0.0).abs() < 4.0);
        let (ago, count) = v.reset_ago();
        if count > seen {
            seen = count;
            resets.push(s as f64 - ago);
        }
        let (ago, count) = l.probe_timed_reset(2);
        if count > lseen {
            lseen = count;
            lresets.push(s as f64 - ago);
        }
    }
    // After the jump, the cycle starts as the clock now has them.
    let start = |m: f64| (m + 0.62 - 0.1) / per;
    for (name, rs, limit) in [("circuit", &resets, 1e-3), ("light", &lresets, 0.1)] {
        let after: Vec<f64> = rs.iter().copied().filter(|&r| r > jump as f64).collect();
        assert!(after.len() >= 2, "{name}: {rs:?}");
        for r in after {
            let m = (0.1 - 0.62 + r * per).round();
            assert!(
                (r - start(m)).abs() < limit,
                "{name}: {r} against {}",
                start(m)
            );
        }
    }
}
