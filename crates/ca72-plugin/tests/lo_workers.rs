//! QUALITY's LO with its voices shared with the workers or not (decisions.md R-LO-WORKERS):
//! the host's thread's load, in real time.
//!
//! - `paced`: material played in real time at 48 kHz, each block at its time with the
//!   plug-in's deadline, with `CA72_WORKERS_LIST` workers (1,2,3,4,9; fewer than this
//!   machine's standing for a smaller computer) in blocks of `CA72_BLOCKS` frames
//!   (64,128,256,512), each way of `CA72_MODES`: `host` (every voice on the host's thread, as
//!   before R-LO-WORKERS), `always` (shared by `lo_threads` whenever it can be), `load` (shared
//!   while the host's thread is busy), `rule` (this system's way, `lo_sharing`), or a number,
//!   at most that many threads (the host's among them) for every run of two voices or more.
//!   Prints, per case, the host's thread's time in the plug-in as a share of the block's
//!   period (the mean, the 99th percentile and the most: what a host's meter shows), the
//!   blocks late, the voices' runs not heard, the processor time of the host's thread and of
//!   the workers as shares of a core, the share of the blocks LO shared, and the runs of
//!   samples that differ from the same chords on the host's thread alone (none unless a run
//!   went unheard).
//!
//!   The material (`CA72_MATERIAL`, a part of a name): `idle` (POLY's ten voices, no note),
//!   `chords-N` (the measurements' changing chords on N voices of Brass Choir's sound
//!   without DOUBLE, N from `CA72_VOICES_LIST`, 1 to 10), `chords-N-double` (with DOUBLE at
//!   50 %), and the factory presets that play the most voices, each as set (Horn Bloom, Brass
//!   Choir, Warp Field, Cycle of Fifths, Swarm Riser).
//!
//!   `CA72_RT`, set: the workers made audio threads (Linux: `SCHED_FIFO`, as the plug-in's
//!   are) and the measuring thread too, as a host's audio thread is; else ordinary threads.
//!   `CA72_SECONDS` (4) each. `CA72_EXTRA`, a share of the period (0): the measuring thread
//!   kept busy that long at each block's start, counted in the plug-in's block (a slower
//!   computer standing in, for the load-responsive sharing).
//! - `same`: the renders at LO with 3, 5 and 9 workers sharing the voices by the rule, on 4
//!   and 10 threads whatever the rule, and with the sharing switched on and off every 37
//!   blocks (as the load would switch it), the same to the bit as on the host's thread alone in
//!   blocks of the same size (64 to 512 frames), every material played whole (no deadline);
//!   the workers' processor time shown, so that they are seen to play.
//!
//! By hand: `cargo test --release -p ca72-plugin --test lo_workers -- --ignored --nocapture`.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::engine::{
    CHUNK, Controls, DEADLINE, Engine, Event, LoSharing, lo_sharing, set_budget_for_measurement,
    set_lo_sharing_for_measurement, set_lo_threads_for_measurement,
};
use common::{Chords, controls_of, env, play_block, played_as, preset};
use std::time::{Duration, Instant};

const RATE: f64 = 48_000.0;

/// A list of numbers from the environment, else `default`.
fn list(name: &str, default: &[usize]) -> Vec<usize> {
    std::env::var(name).ok().map_or_else(
        || default.to_vec(),
        |v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect(),
    )
}

/// The material: its name, controls and how many notes each chord holds.
fn material() -> Vec<(String, Controls, usize)> {
    let mut plain = controls_of(&preset("Brass Choir"));
    (plain.poly, plain.potato, plain.ultra, plain.double) = (true, true, false, 0.0);
    let mut out = Vec::new();
    let mut idle = plain;
    idle.voices = 10;
    out.push(("idle".to_string(), idle, 0));
    for v in list("CA72_VOICES_LIST", &[1, 2, 3, 4, 5, 6, 8, 10]) {
        let mut c = plain;
        c.voices = v;
        out.push((format!("chords-{v}"), c, v));
        c.double = 0.5;
        out.push((format!("chords-{v}-double"), c, v));
    }
    for name in [
        "Horn Bloom",
        "Brass Choir",
        "Warp Field",
        "Cycle of Fifths",
        "Swarm Riser",
    ] {
        let s = preset(name);
        let mut c = played_as(&s, controls_of(&s));
        (c.potato, c.ultra) = (true, false);
        out.push((name.to_string(), c, c.voices));
    }
    let only = std::env::var("CA72_MATERIAL").ok();
    out.retain(|(n, ..)| {
        only.as_deref()
            .is_none_or(|o| o.split('|').any(|o| n.contains(o)))
    });
    out
}

/// A thread's processor time so far, ns: the measuring thread's (`None`), or the workers'
/// together.
#[cfg(target_os = "linux")]
fn cpu(workers: bool) -> u64 {
    let read = |p: std::path::PathBuf| {
        std::fs::read_to_string(p)
            .ok()
            .and_then(|s| s.split_whitespace().next()?.parse::<u64>().ok())
            .unwrap_or(0)
    };
    if !workers {
        return read("/proc/thread-self/schedstat".into());
    }
    let mut ns = 0;
    if let Ok(tasks) = std::fs::read_dir("/proc/self/task") {
        for t in tasks.flatten() {
            let comm = std::fs::read_to_string(t.path().join("comm")).unwrap_or_default();
            if comm.starts_with("ca72-voices") {
                ns += read(t.path().join("schedstat"));
            }
        }
    }
    ns
}

/// As on Linux: the measuring thread's time, or the process's less it (the workers', and the
/// little else the process does meanwhile), from their cycle counts (`QueryThreadCycleTime`,
/// the invariant time-stamp counter's: `GetThreadTimes` charges whole clock ticks), turned
/// into time by the counter's rate against the clock, measured once.
#[cfg(all(windows, target_arch = "x86_64"))]
#[allow(unsafe_code)]
fn cpu(workers: bool) -> u64 {
    use winapi::um::processthreadsapi::{GetCurrentProcess, GetCurrentThread};
    use winapi::um::realtimeapiset::{QueryProcessCycleTime, QueryThreadCycleTime};
    static RATE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    let rate = *RATE.get_or_init(|| {
        // SAFETY: reading the time-stamp counter has no preconditions.
        let (c0, t0) = (unsafe { core::arch::x86_64::_rdtsc() }, Instant::now());
        std::thread::sleep(Duration::from_millis(200));
        // SAFETY: as above.
        let c1 = unsafe { core::arch::x86_64::_rdtsc() };
        (c1 - c0) as f64 / t0.elapsed().as_secs_f64() / 1e9
    });
    let (mut thread, mut process) = (0u64, 0u64);
    // SAFETY: the pseudo-handles of the current thread and process, and a u64 of this frame.
    unsafe {
        QueryThreadCycleTime(GetCurrentThread(), &mut thread);
        if workers {
            QueryProcessCycleTime(GetCurrentProcess(), &mut process);
        }
    }
    let cycles = if workers {
        process.saturating_sub(thread)
    } else {
        thread
    };
    (cycles as f64 / rate) as u64
}

/// As on Linux: the measuring thread's time (`CLOCK_THREAD_CPUTIME_ID`), or the process's less
/// it (`getrusage`: the workers', and the little else the process does meanwhile).
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn cpu(workers: bool) -> u64 {
    let mut t = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: a timespec of this frame.
    unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut t) };
    let thread = t.tv_sec as u64 * 1_000_000_000 + t.tv_nsec as u64;
    if !workers {
        return thread;
    }
    // SAFETY: a zeroed rusage is a valid one, written by the call.
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    // SAFETY: as above.
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let tv = |v: libc::timeval| v.tv_sec as u64 * 1_000_000_000 + v.tv_usec as u64 * 1000;
    (tv(u.ru_utime) + tv(u.ru_stime)).saturating_sub(thread)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    all(windows, target_arch = "x86_64")
)))]
fn cpu(_workers: bool) -> u64 {
    0
}

/// One case's figures.
#[derive(Debug, Clone, Copy)]
struct Figures {
    mean: f64,
    p99: f64,
    most: f64,
    late: usize,
    silent: u64,
    host_cpu: f64,
    workers_cpu: f64,
    workers: usize,
    /// The share of the measured blocks at whose end LO shared, and the runs of [`CHUNK`]
    /// samples that differ from the render on the host's thread alone (none unless a voice's
    /// run went unheard).
    shared: f64,
    differ: usize,
}

/// `c`'s chords of `notes` notes played in real time for `seconds`, `block` frames at a time,
/// with `workers` workers (made audio threads where `rt`).
fn paced_run(
    c: &Controls,
    notes: usize,
    block: usize,
    workers: usize,
    rt: bool,
    seconds: f64,
) -> Figures {
    let period = Duration::from_secs_f64(block as f64 / RATE);
    let extra: f64 = env("CA72_EXTRA", 0.0);
    let mut e = Engine::new();
    e.set(c);
    e.prepare(RATE, 1);
    let started = e.start_workers(workers, rt.then_some(period));
    // (Made an audio thread for this period, and on macOS in a work interval of its own, opened
    // at each block's start and closed at its end, as a host's audio thread is in its device's
    // workgroup: without one, macOS ran a thread busy part of each period about half as fast,
    // decisions.md R11.)
    let host = if rt {
        if let Err(e) = ca72_rt::promote(period, period / 2) {
            eprintln!("(the measuring thread not promoted: {e})");
        }
        ca72_rt::WorkInterval::new("CA-72 measurement host")
    } else {
        None
    };
    let token = host.as_ref().and_then(ca72_rt::WorkInterval::join);
    let mut chords = Chords::new(notes, RATE);
    let blocks = (seconds * RATE / block as f64) as usize;
    // (A second first, unmeasured: the workers asked for and started, the caches warm.)
    let warm = (RATE / block as f64) as usize;
    let len = (warm + blocks) * block;
    let (mut l, mut r) = (vec![0.0f32; len], vec![0.0f32; len]);
    let mut events = Vec::with_capacity(64);
    let mut took = Vec::with_capacity(blocks);
    let (mut late, mut silent0, mut host0, mut work0, mut shared) = (0, 0, 0, 0, 0);
    let start = Instant::now();
    for i in 0..warm + blocks {
        let due = start + period * i as u32;
        if let Some(w) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(w);
        }
        if i == warm {
            (silent0, host0, work0) = (e.silent_runs(), cpu(false), cpu(true));
        }
        if token.is_some()
            && let Some(h) = &host
        {
            h.start(period);
        }
        let t = Instant::now();
        e.set_deadline(Some(t + period.mul_f64(DEADLINE)));
        if e.take_serving() {
            // (The plug-in's helper thread's work, here between blocks, untimed.)
            e.crew().serve();
            e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
        }
        let t = Instant::now();
        // (A slower computer's time, standing in: the host's thread kept busy for `extra` of
        // the period within the plug-in's block, which the plug-in counts as its own.)
        while t.elapsed() < period.mul_f64(extra) {
            std::hint::spin_loop();
        }
        let at = i * block;
        let (bl, br) = (&mut l[at..at + block], &mut r[at..at + block]);
        play_block(&mut e, &mut chords, at, bl, br, &mut events);
        let d = t.elapsed();
        if token.is_some()
            && let Some(h) = &host
        {
            h.finish();
        }
        if e.take_mending() {
            e.spares().mend();
        }
        if i >= warm {
            took.push(d.as_secs_f64() / period.as_secs_f64());
            late += usize::from(d > period);
            shared += usize::from(e.lo_shares());
        }
    }
    let wall = (blocks as f64) * period.as_secs_f64() * 1e9;
    let host_cpu = (cpu(false) - host0) as f64 / wall;
    let workers_cpu = cpu(true).saturating_sub(work0) as f64 / wall;
    let silent = e.silent_runs() - silent0;
    let running = e.workers().max(started);
    drop(e);
    if let (Some(h), Some(t)) = (&host, token) {
        h.leave(t);
    }
    // (The same chords on the host's thread alone, every voice waited for.)
    let sharing = lo_sharing();
    set_lo_sharing_for_measurement(Some(LoSharing::Never));
    let (l0, r0, ..) = whole(c, notes, block, 0, len, None);
    set_lo_sharing_for_measurement(Some(sharing));
    let differ = l
        .chunks(CHUNK)
        .zip(r.chunks(CHUNK))
        .zip(l0.chunks(CHUNK).zip(r0.chunks(CHUNK)))
        .filter(|((a, b), (a0, b0))| {
            a.iter().zip(*a0).any(|(x, y)| x.to_bits() != y.to_bits())
                || b.iter().zip(*b0).any(|(x, y)| x.to_bits() != y.to_bits())
        })
        .count();
    took.sort_by(f64::total_cmp);
    let mean = took.iter().sum::<f64>() / took.len() as f64;
    let p99 = took[(took.len() * 99 / 100).min(took.len() - 1)];
    let most = *took.last().unwrap();
    Figures {
        mean,
        p99,
        most,
        late,
        silent,
        host_cpu,
        workers_cpu,
        workers: running,
        shared: shared as f64 / blocks as f64,
        differ,
    }
}

#[test]
#[ignore = "a measurement, run by hand"]
fn paced() {
    let rt = std::env::var_os("CA72_RT").is_some();
    let seconds: f64 = env("CA72_SECONDS", 4.0);
    let modes: Vec<String> = std::env::var("CA72_MODES")
        .unwrap_or_else(|_| "host,rule".to_string())
        .split(',')
        .map(str::to_string)
        .collect();
    if rt {
        let p = Duration::from_secs_f64(256.0 / RATE);
        if let Err(e) = ca72_rt::promote(p, p / 2) {
            eprintln!("(the measuring thread not promoted: {e})");
        }
    }
    eprintln!(
        "LO, 48 kHz, {} threads; per case: the host's thread's time in the plug-in as a share of the period (mean, 99th percentile, most), blocks late, voices' runs not heard, processor time of the host's thread and of the workers (shares of a core)",
        if rt { "audio" } else { "ordinary" }
    );
    for (name, c, notes) in material() {
        for &workers in &list("CA72_WORKERS_LIST", &[1, 2, 3, 4, 9]) {
            set_budget_for_measurement(workers.max(1));
            for &block in &list("CA72_BLOCKS", &[64, 128, 256, 512]) {
                for mode in &modes {
                    let (sharing, threads) = match mode.as_str() {
                        "host" => (Some(LoSharing::Never), None),
                        "always" => (Some(LoSharing::Always), None),
                        "load" => (Some(LoSharing::Load), None),
                        "rule" => (None, None),
                        k => (Some(LoSharing::Always), Some(k.parse().unwrap())),
                    };
                    set_lo_sharing_for_measurement(sharing);
                    set_lo_threads_for_measurement(threads);
                    let f = paced_run(&c, notes, block, workers, rt, seconds);
                    eprintln!(
                        "{name:>18} w{workers} ({} running) b{block:>3} {mode:>6}: mean {:5.1} % p99 {:5.1} % most {:6.1} % late {:3} unheard {:4} cpu host {:5.1} % workers {:5.1} % shared {:3.0} % differ {}",
                        f.workers,
                        100.0 * f.mean,
                        100.0 * f.p99,
                        100.0 * f.most,
                        f.late,
                        f.silent,
                        100.0 * f.host_cpu,
                        100.0 * f.workers_cpu,
                        100.0 * f.shared,
                        f.differ,
                    );
                }
            }
        }
    }
    set_lo_threads_for_measurement(None);
    set_lo_sharing_for_measurement(None);
    set_budget_for_measurement(0);
    if rt {
        ca72_rt::demote();
    }
}

/// `c`'s chords of `notes` notes for `n` samples, `block` frames at a time, with `workers`
/// workers, every voice waited for, LO's sharing switched on and off every `flip` blocks if
/// asked (from on): left and right, the runs not heard, and the workers' processor time, ns.
fn whole(
    c: &Controls,
    notes: usize,
    block: usize,
    workers: usize,
    n: usize,
    flip: Option<usize>,
) -> (Vec<f32>, Vec<f32>, u64, u64) {
    let mut e = Engine::new();
    e.set(c);
    e.prepare(RATE, 1);
    e.start_workers(workers, None);
    let mut chords = Chords::new(notes, RATE);
    let (mut l, mut r) = (vec![0.0f32; n], vec![0.0f32; n]);
    let mut events: Vec<(usize, Event)> = Vec::with_capacity(64);
    for b in (0..n).step_by(block) {
        if let Some(k) = flip {
            let on = (b / block / k).is_multiple_of(2);
            set_lo_sharing_for_measurement(Some(if on {
                LoSharing::Always
            } else {
                LoSharing::Never
            }));
        }
        e.set_deadline(None);
        let (bl, br) = (&mut l[b..b + block], &mut r[b..b + block]);
        play_block(&mut e, &mut chords, b, bl, br, &mut events);
    }
    let ns = cpu(true);
    (l, r, e.silent_runs(), ns)
}

#[test]
#[ignore = "a measurement, run by hand"]
fn same() {
    let seconds: f64 = env("CA72_SECONDS", 3.0);
    // (A whole number of the largest blocks, so that every render is as long.)
    let n = (seconds * RATE) as usize / 1024 * 1024;
    let same = |(l, r): (&[f32], &[f32]), (l0, r0): (&[f32], &[f32])| {
        l.len() == l0.len()
            && l.iter().zip(l0).all(|(a, b)| a.to_bits() == b.to_bits())
            && r.iter().zip(r0).all(|(a, b)| a.to_bits() == b.to_bits())
    };
    let mut differ = 0;
    for (name, c, notes) in material() {
        if name == "idle" {
            continue;
        }
        for block in [64, 128, 256, 512] {
            // (Every voice on the host's thread, as before R-LO-WORKERS: the reference.)
            set_lo_sharing_for_measurement(Some(LoSharing::Never));
            set_lo_threads_for_measurement(None);
            let (l0, r0, ..) = whole(&c, notes, block, 0, n, None);
            // (By the rule, on 4 and 10 threads whatever it, and switched on and off every 37
            // blocks, as by load.)
            for (workers, threads, flip) in [
                (3, None, None),
                (5, None, None),
                (9, None, None),
                (3, Some(4), None),
                (9, Some(10), None),
                (5, None, Some(37)),
            ] {
                set_lo_sharing_for_measurement(Some(LoSharing::Always));
                set_lo_threads_for_measurement(threads);
                let (l, r, silent, ns) = whole(&c, notes, block, workers, n, flip);
                let ok = same((&l, &r), (&l0, &r0));
                differ += usize::from(!ok || silent > 0);
                eprintln!(
                    "{name:>18} w{workers} {:>8} b{block:>3}: {} ({silent} runs not heard; the workers' time {:.1} ms)",
                    match (threads, flip) {
                        (_, Some(k)) => format!("flip{k}"),
                        (Some(t), _) => format!("t{t}"),
                        _ => "rule".to_string(),
                    },
                    if ok { "same" } else { "DIFFERS" },
                    ns as f64 / 1e6,
                );
            }
        }
    }
    set_lo_threads_for_measurement(None);
    set_lo_sharing_for_measurement(None);
    assert_eq!(
        differ, 0,
        "a render differed from the host's thread's alone"
    );
}
