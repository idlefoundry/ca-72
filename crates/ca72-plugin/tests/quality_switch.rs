//! QUALITY switched while the instrument plays (issue #56: a loud burst on choosing ULTRA in the
//! macOS standalone, then lasting distortion). Each preset's notes are played while QUALITY moves
//! between LO, HI and ULTRA, mid-note and at rest, at the start of a host's block as the
//! plug-in's parameters arrive; each render is held against the same notes played at the
//! quality it ends in from the start.
//!
//! - `offline`: no deadline (as a host renders offline). Every sample must be finite and no
//!   voice put back clean ([`Engine::take_mending`]); the peak near each switch is held against
//!   the reference's there (a burst), and the loudness and the high band (the third
//!   difference's energy: distortion's edges) of each stretch after the switches settle against
//!   the reference's (a state left broken). `CA72_RATES` (44100,48000,96000), `CA72_BLOCKS`
//!   (64,100,256,1024), `CA72_ONLY` (a part of a preset's name), `CA72_WORKERS` (0 and the
//!   plug-in's own count by default, or one count).
//! - `paced`: the same in real time at the plug-in's deadline (`CA72_BLOCK` frames, 256, at 48
//!   kHz; `CA72_WORKERS`, the plug-in's own count by default; the workers and the measuring
//!   thread audio threads as a host's are, unless `CA72_PLAIN` is set), on a computer that cannot play
//!   ULTRA live: the voices' runs not heard while it plays, the peak against the offline render's,
//!   and whether the sound after QUALITY is back at HI is HI's again. `CA72_SPLIT`: ULTRA's
//!   split voices (this machine's by default).
//!
//! Each render's thread flushes denormals to zero as nih-plug's process call has it (the
//! workers follow it), unless `CA72_NO_FTZ` is set.
//!
//! By hand: `cargo test --release -p ca72-plugin --test quality_switch -- --ignored --nocapture`.
//! `CA72_SWITCH_OUT=<dir>` writes the renders as WAVs there.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::engine::{AUTO, Controls, DEADLINE, Engine, Event};
use common::{controls_of, env, played_as, preset};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The flush-to-zero bit nih-plug sets on the host's thread for each block (`ScopedFtz`).
#[cfg(target_arch = "x86_64")]
const FTZ: u32 = 1 << 15;
#[cfg(target_arch = "aarch64")]
const FTZ: u32 = 1 << 24;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const FTZ: u32 = 0;

/// The render's seconds.
const SECONDS: f64 = 5.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Q {
    Lo,
    Hi,
    Ultra,
}

fn at_quality(c: &Controls, q: Q) -> Controls {
    let mut c = *c;
    (c.potato, c.ultra) = match q {
        Q::Lo => (true, false),
        Q::Hi => (false, false),
        Q::Ultra => (false, true),
    };
    c
}

/// The sounds: their names and controls (POLY's voices held to four, so that ULTRA's renders
/// stay short).
fn sounds() -> Vec<(String, Controls)> {
    let mut out = Vec::new();
    for name in [
        "Felt Bass",
        "Feedback Funk",
        "Driven Kick",
        "Brass Choir",
        "Swarm Riser",
        "Warp Field",
    ] {
        let s = preset(name);
        let mut c = played_as(&s, controls_of(&s));
        c.voices = c.voices.min(4);
        out.push((name.to_string(), c));
    }
    // The panel as a new instance has it, in POLY.
    let mut init = ca72_plugin::params::Ca72Params::default().controls();
    init.voices = 4;
    init.poly = true;
    out.push(("init POLY".to_string(), init));
    let only = std::env::var("CA72_ONLY").ok();
    out.retain(|(n, _)| only.as_deref().is_none_or(|o| n.contains(o)));
    out
}

/// The notes, (seconds, event): a chord (a held key with POLY off) from 0.1 s to 2.4 s, and
/// another from 3.0 s to 4.2 s, after the first has rung out.
fn notes(poly: bool) -> Vec<(f64, Event)> {
    let first: &[u8] = if poly { &[48, 55, 60, 64] } else { &[48] };
    let second: &[u8] = if poly { &[50, 57, 62, 65] } else { &[55] };
    let mut e = Vec::new();
    for &key in first {
        e.push((0.1, Event::Note { key, on: true }));
        e.push((2.4, Event::Note { key, on: false }));
    }
    for &key in second {
        e.push((3.0, Event::Note { key, on: true }));
        e.push((4.2, Event::Note { key, on: false }));
    }
    e
}

/// A way of moving QUALITY: its name, the quality it starts at and its switches (seconds,
/// quality), and the stretches (seconds) compared with the reference at the quality it ends in.
struct Moves {
    name: &'static str,
    start: Q,
    switches: Vec<(f64, Q)>,
    compared: Vec<(f64, f64)>,
}

fn moves() -> Vec<Moves> {
    let mut flap = Vec::new();
    for k in 0..16 {
        let q = if k % 2 == 0 { Q::Ultra } else { Q::Hi };
        flap.push((0.6 + 0.05 * f64::from(k), q));
    }
    flap.push((1.5, Q::Ultra));
    vec![
        Moves {
            name: "HI to ULTRA mid-note",
            start: Q::Hi,
            switches: vec![(0.8, Q::Ultra)],
            compared: vec![(1.2, 2.4), (3.2, 4.2)],
        },
        Moves {
            name: "HI to ULTRA at rest",
            start: Q::Hi,
            switches: vec![(2.75, Q::Ultra)],
            compared: vec![(3.2, 4.2)],
        },
        Moves {
            name: "LO to ULTRA mid-note",
            start: Q::Lo,
            switches: vec![(0.8, Q::Ultra)],
            compared: vec![(3.2, 4.2)],
        },
        Moves {
            name: "LO, HI, ULTRA, HI, ULTRA",
            start: Q::Lo,
            switches: vec![(0.5, Q::Hi), (0.9, Q::Ultra), (1.3, Q::Hi), (1.7, Q::Ultra)],
            compared: vec![(3.2, 4.2)],
        },
        Moves {
            name: "ULTRA and back to HI, then LO",
            start: Q::Hi,
            switches: vec![(0.5, Q::Ultra), (1.4, Q::Hi), (2.7, Q::Lo)],
            compared: vec![(3.2, 4.2)],
        },
        // (The same return to LO without ULTRA: what LO's light voices, resting while HI plays,
        // differ by alone.)
        Moves {
            name: "HI, then LO",
            start: Q::Hi,
            switches: vec![(2.7, Q::Lo)],
            compared: vec![(3.2, 4.2)],
        },
        Moves {
            name: "ULTRA and back to HI",
            start: Q::Ultra,
            switches: vec![(0.9, Q::Hi)],
            compared: vec![(1.3, 2.4), (3.2, 4.2)],
        },
        Moves {
            name: "HI and ULTRA every 50 ms",
            start: Q::Hi,
            switches: flap,
            compared: vec![(1.9, 2.4), (3.2, 4.2)],
        },
    ]
}

impl Moves {
    fn end(&self) -> Q {
        self.switches.last().map_or(self.start, |s| s.1)
    }
}

/// What a render gives: left, right, samples not finite, voices put back clean, the voices'
/// runs not heard, and blocks late.
struct Render {
    l: Vec<f32>,
    r: Vec<f32>,
    non_finite: usize,
    mended: usize,
    silent: u64,
    late: usize,
    /// The runs not heard and the blocks late in each phase: before the first switch, between
    /// each switch and the next, and after the last.
    phases: Vec<(u64, usize)>,
}

#[allow(clippy::too_many_arguments)]
fn render(
    c: &Controls,
    start: Q,
    switches: &[(f64, Q)],
    rate: f64,
    block: usize,
    workers: usize,
    paced: bool,
) -> Render {
    let was = ca72_rt::flush_mode();
    if std::env::var_os("CA72_NO_FTZ").is_none() {
        ca72_rt::set_flush_mode(was | FTZ);
    }
    let mut e = Engine::new();
    e.set(&at_quality(c, start));
    e.prepare(rate, 1);
    let period = Duration::from_secs_f64(block as f64 / rate);
    // (Paced, the workers and this thread made audio threads as a host's is, this one in a work
    // interval on macOS, unless `CA72_PLAIN` is set: ordinary threads busy part of each period
    // run slowly on macOS, and HI's voices then miss their deadline whatever QUALITY did.)
    let rt = paced && std::env::var_os("CA72_PLAIN").is_none();
    e.start_workers(workers, rt.then_some(period));
    let host = if rt {
        if let Err(e) = ca72_rt::promote(period, period.mul_f64(0.5)) {
            eprintln!("(the measuring thread not promoted: {e})");
        }
        ca72_rt::WorkInterval::new("CA-72 measurement host")
    } else {
        None
    };
    let token = host.as_ref().and_then(ca72_rt::WorkInterval::join);
    let n = (SECONDS * rate) as usize;
    let mut events: Vec<(usize, Event)> = notes(c.poly || c.unison)
        .into_iter()
        .map(|(t, ev)| ((t * rate) as usize, ev))
        .collect();
    events.sort_by_key(|x| (x.0, matches!(x.1, Event::Note { on: true, .. })));
    let mut next = events.iter().peekable();
    let mut sw = switches
        .iter()
        .map(|&(t, q)| ((t * rate) as usize, q))
        .peekable();
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let (mut mended, mut late) = (0, 0);
    let mut phases = vec![(0u64, 0usize)];
    let mut since = 0u64;
    let t0 = Instant::now();
    let mut b = 0;
    let mut i = 0u32;
    while b < n {
        let len = block.min(n - b);
        // QUALITY as a host's block brings it: at the block's start.
        let mut q = None;
        while let Some((_, nq)) = sw.next_if(|(at, _)| *at < b + len) {
            q = Some(nq);
        }
        if let Some(q) = q {
            e.set(&at_quality(c, q));
            let now = e.silent_runs();
            phases.last_mut().unwrap().0 = now - since;
            since = now;
            phases.push((0, 0));
        }
        if paced {
            let due = t0 + period * i;
            if let Some(w) = due.checked_duration_since(Instant::now()) {
                std::thread::sleep(w);
            }
            if token.is_some()
                && let Some(h) = &host
            {
                h.start(period);
            }
            e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
            if e.take_serving() {
                e.crew().serve();
                e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
            }
        }
        let t = Instant::now();
        let mut k = b;
        while k < b + len {
            while let Some((_, ev)) = next.next_if(|(at, _)| *at <= k) {
                e.event(*ev);
            }
            let stop = next
                .peek()
                .map_or(b + len, |(at, _)| (*at).clamp(k + 1, b + len));
            e.render(&[], &mut l[k..stop], &mut r[k..stop]);
            k = stop;
        }
        e.end_block(len);
        if token.is_some()
            && let Some(h) = &host
        {
            h.finish();
        }
        if paced && t.elapsed() > period {
            late += 1;
            phases.last_mut().unwrap().1 += 1;
        }
        if e.take_mending() {
            mended += 1;
            e.spares().mend();
        }
        if !paced && e.take_serving() {
            e.crew().serve();
        }
        b += len;
        i += 1;
    }
    let silent = e.silent_runs();
    drop(e);
    if let (Some(h), Some(t)) = (&host, token) {
        h.leave(t);
    }
    if rt {
        ca72_rt::demote();
    }
    ca72_rt::set_flush_mode(was);
    let non_finite = l.iter().chain(&r).filter(|v| !v.is_finite()).count();
    Render {
        l,
        r,
        non_finite,
        mended,
        silent,
        phases: {
            phases.last_mut().unwrap().0 = silent - since;
            phases
        },
        late,
    }
}

/// The energy (mean square) of `x` and of its third difference (a high-pass rising 18 dB an
/// octave: distortion's edges), samples `a..b`.
fn energies(x: &[f32], a: usize, b: usize) -> (f64, f64) {
    let b = b.min(x.len());
    let a = a.max(3).min(b);
    let (mut e, mut h) = (0.0, 0.0);
    for i in a..b {
        let v = f64::from(x[i]);
        let d = v - 3.0 * f64::from(x[i - 1]) + 3.0 * f64::from(x[i - 2]) - f64::from(x[i - 3]);
        e += v * v;
        h += d * d;
    }
    let n = (b - a).max(1) as f64;
    (e / n, h / n)
}

fn db(x: f64) -> f64 {
    10.0 * x.max(1e-30).log10()
}

fn peak(x: &[f32], a: usize, b: usize) -> f32 {
    x[a.min(x.len())..b.min(x.len())]
        .iter()
        .fold(0.0f32, |m, v| m.max(v.abs()))
}

/// How a render stands against the reference: the worst rise of the peak within 50 ms after a
/// switch, dB; and over the compared stretches the worst difference of loudness and of the high
/// band, dB.
fn against(m: &Moves, x: &Render, reference: &Render, rate: f64) -> (f64, f64, f64) {
    let s = |t: f64| (t * rate) as usize;
    let mut burst = f64::NEG_INFINITY;
    for &(t, _) in &m.switches {
        let (a, b) = (s(t), s(t + 0.05));
        let here = peak(&x.l, a, b).max(peak(&x.r, a, b));
        // (Against the reference's loudest in the whole render, as the voices at a switch
        // between qualities may be at a different point of their contours.)
        let there =
            peak(&reference.l, 0, reference.l.len()).max(peak(&reference.r, 0, reference.r.len()));
        burst = burst.max(20.0 * f64::from(here.max(1e-9) / there.max(1e-9)).log10());
    }
    let (mut loud, mut high) = (0.0f64, 0.0f64);
    for &(t0, t1) in &m.compared {
        for (y, z) in [(&x.l, &reference.l), (&x.r, &reference.r)] {
            let (e, h) = energies(y, s(t0), s(t1));
            let (e0, h0) = energies(z, s(t0), s(t1));
            if e0 < 1e-12 {
                continue;
            }
            let dl = db(e) - db(e0);
            let dh = db(h) - db(h0);
            if dl.abs() > loud.abs() {
                loud = dl;
            }
            if dh.abs() > high.abs() {
                high = dh;
            }
        }
    }
    (burst, loud, high)
}

/// A stereo WAV of 32-bit floats at `rate` Hz.
fn write_wav(path: &Path, rate: f64, l: &[f32], r: &[f32]) {
    let frames = l.len().min(r.len());
    let data = (frames * 8) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&(rate as u32).to_le_bytes());
    b.extend_from_slice(&(rate as u32 * 8).to_le_bytes());
    b.extend_from_slice(&8u16.to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for i in 0..frames {
        b.extend_from_slice(&l[i].to_le_bytes());
        b.extend_from_slice(&r[i].to_le_bytes());
    }
    std::fs::write(path, b).unwrap();
}

fn list<T: std::str::FromStr + Clone>(name: &str, default: &[T]) -> Vec<T> {
    match std::env::var(name) {
        Err(_) => default.to_vec(),
        Ok(s) => s
            .split(',')
            .map(|v| {
                v.trim()
                    .parse()
                    .unwrap_or_else(|_| panic!("{name}={s:?} does not read as a list"))
            })
            .collect(),
    }
}

fn file_name(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

#[test]
#[ignore = "a measurement, run by hand"]
fn offline() {
    let rates: Vec<f64> = list("CA72_RATES", &[44_100.0, 48_000.0, 96_000.0]);
    let blocks: Vec<usize> = list("CA72_BLOCKS", &[64, 100, 256, 1024]);
    let workers: Vec<usize> = match std::env::var("CA72_WORKERS") {
        Ok(_) => vec![env("CA72_WORKERS", AUTO)],
        Err(_) => vec![0, AUTO],
    };
    let out = std::env::var_os("CA72_SWITCH_OUT").map(PathBuf::from);
    let mut faults = Vec::new();
    let (mut worst_burst, mut worst_loud, mut worst_high) = (f64::NEG_INFINITY, 0.0f64, 0.0f64);
    for (name, c) in sounds() {
        for &rate in &rates {
            for &block in &blocks {
                for &w in &workers {
                    let mut refs = std::collections::HashMap::new();
                    for m in moves() {
                        let end = m.end();
                        let reference = refs
                            .entry(format!("{end:?}"))
                            .or_insert_with(|| render(&c, end, &[], rate, block, w, false));
                        let x = render(&c, m.start, &m.switches, rate, block, w, false);
                        let (burst, loud, high) = against(&m, &x, reference, rate);
                        worst_burst = worst_burst.max(burst);
                        if loud.abs() > worst_loud.abs() {
                            worst_loud = loud;
                        }
                        if high.abs() > worst_high.abs() {
                            worst_high = high;
                        }
                        let line = format!(
                            "{name}, {}, {rate} Hz, {block} frames, {} workers: peak after a switch {burst:+.1} dB of the reference's, loudness {loud:+.2} dB, high band {high:+.2} dB; {} not finite, {} mended",
                            m.name,
                            if w == AUTO {
                                "auto".to_string()
                            } else {
                                w.to_string()
                            },
                            x.non_finite,
                            x.mended
                        );
                        eprintln!("{line}");
                        // (A move through LO: the voices the other model plays rest meanwhile
                        // and take up from where they rested, their oscillators at other phases
                        // than the reference's. HI then LO, without ULTRA, differs by as much: up
                        // to 2.2 dB and 5.4 dB in a detuned swarm such as Swarm Riser's.)
                        let through_lo =
                            m.start == Q::Lo || m.switches.iter().any(|&(_, q)| q == Q::Lo);
                        let (most_loud, most_high) =
                            if through_lo { (3.0, 6.0) } else { (1.5, 3.0) };
                        if x.non_finite > 0
                            || x.mended > 0
                            || burst > 1.0
                            || loud.abs() > most_loud
                            || high.abs() > most_high
                        {
                            faults.push(line);
                        }
                        if let Some(d) = &out
                            && rate == 48_000.0
                            && block == 256
                        {
                            std::fs::create_dir_all(d).unwrap();
                            write_wav(
                                &d.join(format!("{}-{}.wav", file_name(&name), file_name(m.name))),
                                rate,
                                &x.l,
                                &x.r,
                            );
                        }
                    }
                }
            }
        }
    }
    eprintln!(
        "worst: peak after a switch {worst_burst:+.1} dB, loudness {worst_loud:+.2} dB, high band {worst_high:+.2} dB"
    );
    eprintln!("outside the bounds: {}", faults.len());
    for f in &faults {
        eprintln!("  {f}");
    }
    assert!(
        faults.is_empty(),
        "{} renders outside the bounds",
        faults.len()
    );
}

#[test]
#[ignore = "a measurement, run by hand"]
fn paced() {
    let block: usize = env("CA72_BLOCK", 256);
    let workers: usize = env("CA72_WORKERS", AUTO);
    let rate: f64 = env("CA72_RATE", 48_000.0);
    // (ULTRA's split voices as another machine has them: two on a Mac of ten fast cores.)
    if let Ok(s) = std::env::var("CA72_SPLIT") {
        ca72_plugin::engine::set_split_for_measurement(Some(s.parse().unwrap()));
    }
    let out = std::env::var_os("CA72_SWITCH_OUT").map(PathBuf::from);
    for (name, c) in sounds() {
        // ULTRA chosen mid-note and held, then HI again (where a state left broken would last).
        let m = Moves {
            name: "ULTRA for 1.6 s, then HI",
            start: Q::Hi,
            switches: vec![(0.6, Q::Ultra), (2.2, Q::Hi)],
            compared: vec![(3.2, 4.2)],
        };
        let offline = render(&c, m.start, &m.switches, rate, block, workers, false);
        let live = render(&c, m.start, &m.switches, rate, block, workers, true);
        let (burst, loud, high) = against(&m, &live, &offline, rate);
        let s = |t: f64| (t * rate) as usize;
        let p_off = peak(&offline.l, 0, offline.l.len()).max(peak(&offline.r, 0, offline.r.len()));
        let p_ultra = peak(&live.l, s(0.6), s(2.2)).max(peak(&live.r, s(0.6), s(2.2)));
        eprintln!(
            "paced {name}, {rate} Hz, {block} frames: {} voices' runs not heard, {} blocks late; peak while ULTRA played {:+.1} dB of the offline render's loudest, after a switch {burst:+.1} dB; back at HI: loudness {loud:+.2} dB, high band {high:+.2} dB of offline; {} not finite, {} mended",
            live.silent,
            live.late,
            20.0 * f64::from(p_ultra.max(1e-9) / p_off.max(1e-9)).log10(),
            live.non_finite,
            live.mended
        );
        eprintln!(
            "  runs not heard and blocks late: before ULTRA {:?}, at ULTRA {:?}, back at HI {:?}",
            live.phases[0], live.phases[1], live.phases[2]
        );
        if let Some(d) = &out {
            std::fs::create_dir_all(d).unwrap();
            write_wav(
                &d.join(format!("{}-paced-{block}.wav", file_name(&name))),
                rate,
                &live.l,
                &live.r,
            );
            write_wav(
                &d.join(format!("{}-offline-{block}.wav", file_name(&name))),
                rate,
                &offline.l,
                &offline.r,
            );
        }
    }
}
