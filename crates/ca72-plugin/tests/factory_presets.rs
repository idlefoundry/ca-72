//! The factory presets played through the plug-in's own `process` (`common::host`), as a host
//! renders offline: every value a preset names is the plug-in's, every assignment a real source
//! and target; each preset that uses 0.3's features (the arpeggiator, the LFOs, the macros,
//! velocity, the timed and CONTINUOUS oscillators, pressure and timbre) sounds, finite, not
//! silent, not clipped; and those that keep time (the arpeggiator, a timed oscillator or LFO)
//! play at the host's tempo, and in the standalone at its TEMPO (120 BPM, a new instance's) as
//! a host at 120 does (decisions.md R-PRESETS-0.3.0, which folded the candidates' test in).
//!
//! By hand, the first held note's pitch against its key, in cents, where the phrase has one:
//! `cargo test --release -p ca72-plugin --test factory_presets -- --ignored --nocapture`
//! (`CA72_ONLY`, a part of a name; `CA72_ONE_NOTE` set: middle C alone for two seconds, the
//! arpeggiator off, for those whose phrase has none). Their loudness is `preset_levels.rs`'.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::library::{Sound, factory};
use common::host::{Play, Quality, RATE, check, peak, render, rms, timed, uses_0_3};
use common::{LEAD, env, phrase};

fn tag(s: &Sound, t: &str) -> bool {
    s.tags.iter().any(|x| x == t)
}

fn value(s: &Sound, id: &str) -> Option<f64> {
    s.values.iter().find(|(k, _)| k == id).map(|(_, v)| *v)
}

/// Every factory preset sets only the plug-in's parameters, sources and targets.
#[test]
fn every_factory_preset_reads() {
    for s in factory() {
        check(s);
    }
    assert!(
        factory().iter().filter(|s| uses_0_3(s)).count() >= 21,
        "the presets that show 0.3 off"
    );
}

/// Each preset that uses 0.3's features sounds at LO through `process`: finite, not silent,
/// not clipped (its first four seconds, for the debug build's time).
#[test]
fn the_0_3_presets_sound() {
    for s in factory().iter().filter(|s| uses_0_3(s)) {
        let mut play = Play::phrase(s, Quality::Lo);
        play.seconds = play.seconds.min(4.0);
        let (l, r) = render(s, &play);
        assert!(l.iter().chain(&r).all(|x| x.is_finite()), "{}", s.name);
        let (p, m) = (peak(&l, &r), rms(&l, &r));
        assert!(m > 1e-3 && p < 1.0, "{}: peak {p}, RMS {m}", s.name);
    }
}

/// How far apart two renders sound: the mean difference of their 10 ms loudness (RMS, dB),
/// over the windows where either is within 40 dB of the louder's loudest. (A timed LFO's edge
/// can land a few samples apart with the host's transport and without it, the host's place
/// taken each block, the standalone's counted on: inaudible, and no drift.)
fn apart(a: &(Vec<f32>, Vec<f32>), b: &(Vec<f32>, Vec<f32>)) -> f64 {
    let win = (0.01 * RATE) as usize;
    let env = |x: &(Vec<f32>, Vec<f32>)| -> Vec<f64> {
        x.0.chunks(win)
            .zip(x.1.chunks(win))
            .map(|(l, r)| 20.0 * rms(l, r).max(1e-9).log10())
            .collect()
    };
    let (ea, eb) = (env(a), env(b));
    let top = ea.iter().chain(&eb).fold(f64::MIN, |m, x| m.max(*x));
    let d: Vec<f64> = ea
        .iter()
        .zip(&eb)
        .filter(|(x, y)| x.max(**y) > top - 40.0)
        .map(|(x, y)| (x - y).abs())
        .collect();
    d.iter().sum::<f64>() / d.len().max(1) as f64
}

/// Each preset that keeps time plays in the standalone at its TEMPO, 120 BPM, as at a host's
/// 120 BPM; at a host's 90 BPM it plays otherwise (it follows the host), as the standalone does
/// with its TEMPO at 90 (their 10 ms loudness within a decibel, and 90 against 120 more than a
/// decibel and a half apart). Three seconds of its phrase at LO.
#[test]
fn the_timed_presets_keep_the_hosts_tempo_and_the_standalones() {
    let timed: Vec<&Sound> = factory().iter().filter(|s| timed(s)).collect();
    assert!(timed.len() >= 6, "{} timed presets", timed.len());
    for s in timed {
        let play = |host: Option<f64>, standalone: f64| {
            let mut p = Play::phrase(s, Quality::Lo);
            (p.seconds, p.host_tempo, p.standalone_tempo) = (3.0, host, standalone);
            render(s, &p)
        };
        let host = play(Some(120.0), 0.0);
        let standalone = play(None, 120.0);
        let slower = play(Some(90.0), 120.0);
        let standalone_slower = play(None, 90.0);
        let (same, same_slower) = (
            apart(&host, &standalone),
            apart(&slower, &standalone_slower),
        );
        let other = apart(&host, &slower);
        println!(
            "{:20} standalone {same:.3} dB at 120, {same_slower:.3} dB at 90; 90 against 120 \
             {other:.2} dB",
            s.name
        );
        assert!(
            same < 1.0,
            "{}: the standalone at 120 is {same} dB off",
            s.name
        );
        assert!(
            same_slower < 1.0,
            "{}: the standalone at 90 is {same_slower} dB off",
            s.name
        );
        assert!(other > 1.5, "{}: 90 BPM plays as 120 ({other} dB)", s.name);
    }
}

/// How far the first note's pitch is from its key, cents, where it has one: the strongest
/// normalized autocorrelation over the 0.4 s ending 0.25 s before its release, where the
/// contours have settled, its lag looked for within 60 cents of the key's octaves from four
/// below to one above.
fn cents_off(notes: &[(f64, f64, Vec<u8>)], l: &[f32], r: &[f32]) -> Option<f64> {
    let (t0, len, keys) = notes.first()?;
    if keys.len() != 1 || *len < 0.8 {
        return None;
    }
    let from = ((t0 + len - 0.65) * RATE) as usize;
    let n = (0.2 * RATE) as usize;
    let x: Vec<f64> = (from..from + 2 * n)
        .map(|i| f64::from(l[i] + r[i]))
        .collect();
    let corr = |lag: usize| {
        let (mut xy, mut xx, mut yy) = (0.0, 0.0, 0.0);
        for i in 0..n {
            xy += x[i] * x[i + lag];
            xx += x[i] * x[i];
            yy += x[i + lag] * x[i + lag];
        }
        xy / (xx * yy).sqrt().max(1e-30)
    };
    let key_hz = 440.0 * 2f64.powf((f64::from(keys[0]) - 69.0) / 12.0);
    let mut best = (f64::NEG_INFINITY, 0.0);
    for octave in -4..=1 {
        let hz = key_hz * 2f64.powi(octave);
        let lo = (RATE / (hz * 2f64.powf(60.0 / 1200.0))).floor() as usize;
        let hi = (RATE / (hz * 2f64.powf(-60.0 / 1200.0))).ceil() as usize;
        for lag in lo.max(2)..=hi.min(n) {
            let c = corr(lag);
            if c > best.0 {
                // The sub-sample peak, by the parabola through the lag and its neighbours.
                let (a, b) = (corr(lag - 1), corr(lag + 1));
                let shift = 0.5 * (a - b) / (a - 2.0 * c + b);
                let f = RATE / (lag as f64 + shift.clamp(-0.5, 0.5));
                best = (c, 1200.0 * (f / hz).log2());
            }
        }
    }
    (best.0 > 0.8).then_some(best.1)
}

#[test]
#[ignore = "a measurement, run by hand"]
fn each_presets_pitch() {
    let only: String = env("CA72_ONLY", String::new());
    let one = std::env::var_os("CA72_ONE_NOTE").is_some();
    for s in factory().iter().filter(|s| s.name.contains(&only)) {
        let mut s = s.clone();
        let notes = if one {
            s.values.retain(|(k, _)| k != "arp_on");
            vec![(LEAD, 2.0, vec![60])]
        } else if tag(&s, "arp") && value(&s, "arp_on") == Some(1.0) {
            Vec::new()
        } else {
            phrase(&s)
        };
        let cents = if notes.is_empty() {
            None
        } else {
            let (l, r) = render(&s, &Play::new(notes.clone(), Quality::Lo));
            cents_off(&notes, &l, &r)
        };
        println!(
            "{:25} {}",
            s.name,
            cents.map_or_else(|| "-".to_owned(), |c| format!("{c:+.0}"))
        );
    }
}
