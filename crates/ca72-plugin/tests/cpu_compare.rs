//! The plug-in's processor cost as a host pays it (issue #62): the plug-in played through its own
//! `process`, paced in real time as a host's audio thread calls it, its workers started by its
//! own `initialize` as in a host. The file uses only what 0.2.8 has too, so the same file
//! measures one version against another.
//!
//! - `paced`: one case, from the environment. `CA72_SOUND`:
//!   - `one`: Felt Bass as set (one voice), a note every half second;
//!   - `ten`: Brass Choir's sound, POLY's ten voices without DOUBLE, ten-note chords;
//!   - `brass`, `horn`: Brass Choir (8 voices) and Horn Bloom (10) as set, with DOUBLE, chords
//!     of as many notes;
//!   - `unison`: Swarm Riser as set (UNISON, 4 voices, DOUBLE), a note every half second;
//!   - `idle`: `ten`'s panel with no note;
//!   - any of them with `-knob`: CUTOFF automated at each block's start the whole time (a sine
//!     across most of its travel, one cycle in two seconds), as a host's automation is.
//!
//!   `CA72_QUALITY` (`lo`, `hi`, `ultra`), `CA72_BLOCK` frames (256) at `CA72_RATE` Hz (48000),
//!   for `CA72_SECONDS` (20) after a second unmeasured. The host's thread is made an audio
//!   thread as a host's is (Linux `SCHED_FIFO`, Windows MMCSS, macOS time-constraint in a work
//!   interval opened and closed each block), unless `CA72_PLAIN` is set, and flushes denormals
//!   to zero as nih-plug's wrapper has it.
//!
//!   Prints one `RESULT` line: the host's thread's time in `process` as a share of the block's
//!   period (mean, 99th percentile, most), the blocks that took longer than their period, the
//!   voices' runs not heard (the whole run's, its first second's too), the processor time of the
//!   host's thread and of the whole process as shares of a core, the other processes' processor
//!   time meanwhile (cores: a run beside other work is to be discarded), the watchdog's
//!   demotions (Linux) and the output's peak (that it played).
//!
//! By hand: `cargo test --release -p ca72-plugin --test cpu_compare -- --ignored --nocapture`.

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::Ca72;
use ca72_plugin::engine::Event;
use ca72_plugin::library::Sound;
use ca72_plugin::params::{Ca72Params, QualityMode};
use ca72_plugin::presets::sound_params;
use common::{Chords, env, preset};
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The flush-to-zero bit nih-plug sets on the host's thread for each block (`ScopedFtz`).
#[cfg(target_arch = "x86_64")]
const FTZ: u32 = 1 << 15;
#[cfg(target_arch = "aarch64")]
const FTZ: u32 = 1 << 24;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const FTZ: u32 = 0;

struct Init;
impl InitContext<Ca72> for Init {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn execute(&self, _: ()) {}
    fn set_latency_samples(&self, _: u32) {}
    fn set_current_voice_capacity(&self, _: u32) {}
}

/// The sound's values set as the host sets parameters.
fn set_sound(params: &Ca72Params, c: &TestProcessContext<Ca72>, s: &Sound) {
    let map = params.param_map();
    let sound = sound_params(params);
    for (id, v) in &s.values {
        let (Some((_, q)), Some((_, ptr, _))) = (
            sound.iter().find(|(i, _)| i == id),
            map.iter().find(|(i, ..)| i == id),
        ) else {
            continue;
        };
        assert!(c.automate(*ptr, q.normalized_of(*v)));
    }
}

/// The case's sound set on the plug-in's parameters: how many notes its chords hold.
fn sound(name: &str, quality: QualityMode, p: &Ca72Params, c: &TestProcessContext<Ca72>) -> usize {
    let bool_ = |q: &BoolParam, v: bool| assert!(c.automate(q.as_ptr(), if v { 1.0 } else { 0.0 }));
    let ten = |s: &str| {
        set_sound(p, c, &preset(s));
        bool_(&p.poly, true);
        bool_(&p.unison, false);
        bool_(&p.doubled, false);
        assert!(c.automate(p.voices.as_ptr(), p.voices.preview_normalized(10)));
    };
    let notes = match name {
        "one" => {
            set_sound(p, c, &preset("Felt Bass"));
            1
        }
        "ten" => {
            ten("Brass Choir");
            10
        }
        "idle" => {
            ten("Brass Choir");
            0
        }
        "brass" => {
            set_sound(p, c, &preset("Brass Choir"));
            p.voices.value() as usize
        }
        "horn" => {
            set_sound(p, c, &preset("Horn Bloom"));
            p.voices.value() as usize
        }
        "unison" => {
            set_sound(p, c, &preset("Swarm Riser"));
            1
        }
        other => panic!("no sound {other}"),
    };
    assert!(c.automate(p.quality.as_ptr(), p.quality.preview_normalized(quality)));
    notes
}

/// CUTOFF's panel value (-5..5) at second `t`: a sine across most of its travel, a cycle in
/// two seconds.
fn cutoff_at(t: f64) -> f32 {
    (4.0 * (std::f64::consts::TAU * t / 2.0).sin()) as f32
}

fn note(timing: u32, key: u8, on: bool) -> NoteEvent<()> {
    if on {
        NoteEvent::NoteOn {
            timing,
            voice_id: None,
            channel: 0,
            note: key,
            velocity: 1.0,
        }
    } else {
        NoteEvent::NoteOff {
            timing,
            voice_id: None,
            channel: 0,
            note: key,
            velocity: 0.0,
        }
    }
}

/// The calling thread's processor time so far, ns.
#[cfg(target_os = "linux")]
fn thread_cpu() -> u64 {
    std::fs::read_to_string("/proc/thread-self/schedstat")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(0)
}

/// The process's processor time so far (every thread's, ended ones too), ns, and the whole
/// system's busy time and all its time (the processors', in the same units) for the other
/// processes' share.
#[cfg(target_os = "linux")]
fn process_and_system() -> (u64, (u64, u64)) {
    let tick = 10_000_000u64; // (USER_HZ, 100.)
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    // (The fields after the command's closing parenthesis: utime and stime are the 12th and
    // 13th of them.)
    let rest = stat.rsplit_once(')').map_or("", |(_, r)| r);
    let f: Vec<u64> = rest
        .split_whitespace()
        .map(|x| x.parse().unwrap_or(0))
        .collect();
    let own = (f.get(11).copied().unwrap_or(0) + f.get(12).copied().unwrap_or(0)) * tick;
    let sys = std::fs::read_to_string("/proc/stat").unwrap_or_default();
    let cpu: Vec<u64> = sys
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse().unwrap_or(0))
        .collect();
    // user nice system idle iowait irq softirq steal
    let at = |i: usize| cpu.get(i).copied().unwrap_or(0) * tick;
    let busy = at(0) + at(1) + at(2) + at(5) + at(6) + at(7);
    let all = busy + at(3) + at(4);
    (own, (busy, all))
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn thread_cpu() -> u64 {
    let mut t = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: a timespec of this frame.
    unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut t) };
    t.tv_sec as u64 * 1_000_000_000 + t.tv_nsec as u64
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn process_and_system() -> (u64, (u64, u64)) {
    unsafe extern "C" {
        fn mach_host_self() -> u32;
        fn host_statistics(host: u32, flavor: i32, info: *mut u32, count: *mut u32) -> i32;
    }
    // SAFETY: a zeroed rusage is a valid one, written by the call.
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    // SAFETY: as above.
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let tv = |v: libc::timeval| v.tv_sec as u64 * 1_000_000_000 + v.tv_usec as u64 * 1000;
    let own = tv(u.ru_utime) + tv(u.ru_stime);
    // (HOST_CPU_LOAD_INFO: the ticks in user, system, idle and nice, summed over the
    // processors; only their proportions are used.)
    let mut ticks = [0u32; 4];
    let mut count = 4u32;
    // SAFETY: a buffer of `count` 32-bit words for the host's load.
    let kr = unsafe { host_statistics(mach_host_self(), 3, ticks.as_mut_ptr(), &mut count) };
    let (busy, all) = if kr == 0 {
        let t = ticks.map(u64::from);
        (t[0] + t[1] + t[3], t[0] + t[1] + t[2] + t[3])
    } else {
        (0, 0)
    };
    (own, (busy, all))
}

/// The time-stamp counter's rate against the clock, ticks a nanosecond, measured once.
#[cfg(all(windows, target_arch = "x86_64"))]
#[allow(unsafe_code)]
fn tsc_rate() -> f64 {
    static RATE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *RATE.get_or_init(|| {
        // SAFETY: reading the time-stamp counter has no preconditions.
        let (c0, t0) = (unsafe { core::arch::x86_64::_rdtsc() }, Instant::now());
        std::thread::sleep(Duration::from_millis(200));
        // SAFETY: as above.
        let c1 = unsafe { core::arch::x86_64::_rdtsc() };
        (c1 - c0) as f64 / t0.elapsed().as_secs_f64() / 1e9
    })
}

/// From the cycle counts (`QueryThreadCycleTime`: `GetThreadTimes` charges whole clock ticks).
#[cfg(all(windows, target_arch = "x86_64"))]
#[allow(unsafe_code)]
fn thread_cpu() -> u64 {
    use winapi::um::processthreadsapi::GetCurrentThread;
    use winapi::um::realtimeapiset::QueryThreadCycleTime;
    let mut cycles = 0u64;
    // SAFETY: the current thread's pseudo-handle and a u64 of this frame.
    unsafe { QueryThreadCycleTime(GetCurrentThread(), &mut cycles) };
    (cycles as f64 / tsc_rate()) as u64
}

/// The process's from its cycle counts; the system's from `GetSystemTimes` (whose kernel time
/// includes the idle time).
#[cfg(all(windows, target_arch = "x86_64"))]
#[allow(unsafe_code)]
fn process_and_system() -> (u64, (u64, u64)) {
    use winapi::shared::minwindef::FILETIME;
    use winapi::um::processthreadsapi::{GetCurrentProcess, GetSystemTimes};
    use winapi::um::realtimeapiset::QueryProcessCycleTime;
    let mut cycles = 0u64;
    // SAFETY: the current process's pseudo-handle and a u64 of this frame.
    unsafe { QueryProcessCycleTime(GetCurrentProcess(), &mut cycles) };
    let own = (cycles as f64 / tsc_rate()) as u64;
    let zero = || FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut idle, mut kernel, mut user) = (zero(), zero(), zero());
    // SAFETY: three FILETIMEs of this frame.
    unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) };
    let v = |f: FILETIME| (u64::from(f.dwHighDateTime) << 32 | u64::from(f.dwLowDateTime)) * 100;
    let all = v(kernel) + v(user);
    (own, (all.saturating_sub(v(idle)), all))
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    all(windows, target_arch = "x86_64")
)))]
fn thread_cpu() -> u64 {
    0
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    all(windows, target_arch = "x86_64")
)))]
fn process_and_system() -> (u64, (u64, u64)) {
    (0, (0, 0))
}

/// The voices' runs not heard so far, read from the engine's own account (`Engine`'s
/// `silent_runs`, in the plug-in's debug view: the plug-in shows no more of it).
fn silent_runs(p: &Ca72) -> u64 {
    let s = format!("{p:?}");
    s.split("silent_runs: ")
        .nth(1)
        .and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok())
        .unwrap_or(u64::MAX)
}

#[test]
#[ignore = "a measurement, run by hand"]
#[allow(unsafe_code)]
fn paced() {
    let name = std::env::var("CA72_SOUND").unwrap_or_else(|_| "ten".into());
    let (base, knob) = match name.strip_suffix("-knob") {
        Some(b) => (b.to_string(), true),
        None => (name.clone(), false),
    };
    let quality = match std::env::var("CA72_QUALITY").as_deref() {
        Ok("lo") => QualityMode::Lo,
        Ok("ultra") => QualityMode::Ultra,
        _ => QualityMode::Hi,
    };
    let block: usize = env("CA72_BLOCK", 256);
    let rate: f64 = env("CA72_RATE", 48_000.0);
    let seconds: f64 = env("CA72_SECONDS", 20.0);
    let rt = std::env::var_os("CA72_PLAIN").is_none();
    let period = Duration::from_secs_f64(block as f64 / rate);

    // The plug-in as a host makes it: its parameters set, then initialised for real time.
    let mut p = Ca72::default();
    let params = p.parameters().clone();
    let as_params: Arc<dyn Params> = params.clone();
    let mut c = TestProcessContext::<Ca72>::new(as_params, rate as f32, ProcessMode::Realtime);
    let notes = sound(&base, quality, &params, &c);
    if knob {
        assert!(c.automate(
            params.cutoff.as_ptr(),
            params.cutoff.preview_normalized(cutoff_at(0.0))
        ));
    }
    let config = BufferConfig {
        sample_rate: rate as f32,
        min_buffer_size: None,
        max_buffer_size: block as u32,
        process_mode: ProcessMode::Realtime,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    p.reset();

    // This thread made the host's audio thread.
    ca72_rt::watch();
    let demoted0 = ca72_rt::demotions();
    let was = ca72_rt::flush_mode();
    ca72_rt::set_flush_mode(was | FTZ);
    let host = if rt {
        if let Err(e) = ca72_rt::promote(period, period.mul_f64(0.5)) {
            eprintln!("(the measuring thread not promoted: {e})");
        }
        ca72_rt::WorkInterval::new("CA-72 measurement host")
    } else {
        None
    };
    let token = host.as_ref().and_then(ca72_rt::WorkInterval::join);

    let (mut l, mut r) = (vec![0.0f32; block], vec![0.0f32; block]);
    let mut buffer = Buffer::default();
    // SAFETY: both slices are `block` long and outlive the buffer, dropped first below.
    unsafe {
        buffer.set_slices(block, |s| {
            s.clear();
            s.push(&mut l);
            s.push(&mut r);
        });
    }
    let mut aux = AuxiliaryBuffers {
        inputs: &mut [],
        outputs: &mut [],
    };
    let mut chords = Chords::new(notes, rate);
    let mut events: Vec<(usize, Event)> = Vec::with_capacity(64);
    let warm = (rate / block as f64).ceil() as usize;
    let blocks = (seconds * rate / block as f64) as usize;
    let mut took = Vec::with_capacity(blocks);
    let (mut late, mut peak) = (0usize, 0.0f32);
    let (mut host0, mut own0, mut sys0, mut wall0) = (0, 0, (0, 0), Instant::now());
    let start = Instant::now();
    for i in 0..warm + blocks {
        // (The block's notes and CUTOFF's value given to the host's context, as a host has
        // them ready before it calls the plug-in.)
        let at = i * block;
        events.clear();
        if notes > 0 {
            for k in 0..block {
                chords.events(at + k, k, &mut events);
            }
        }
        for (k, ev) in &events {
            if let Event::Note { key, on } = ev {
                c.push_event(note(*k as u32, *key, *on));
            }
        }
        if knob {
            let v = params
                .cutoff
                .preview_normalized(cutoff_at(at as f64 / rate));
            assert!(c.automate(params.cutoff.as_ptr(), v));
        }
        let due = start + period * i as u32;
        if let Some(w) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(w);
        }
        if i == warm {
            host0 = thread_cpu();
            (own0, sys0) = process_and_system();
            wall0 = Instant::now();
        }
        if token.is_some()
            && let Some(h) = &host
        {
            h.start(period);
        }
        let t = Instant::now();
        p.process(&mut buffer, &mut aux, &mut c);
        let d = t.elapsed();
        if token.is_some()
            && let Some(h) = &host
        {
            h.finish();
        }
        if i >= warm {
            took.push(d.as_secs_f64() / period.as_secs_f64());
            late += usize::from(d > period);
            for ch in buffer.as_slice_immutable() {
                peak = ch.iter().fold(peak, |m, v| m.max(v.abs()));
            }
        }
    }
    let wall = wall0.elapsed().as_nanos() as f64;
    let host_cpu = (thread_cpu() - host0) as f64 / wall;
    let (own1, sys1) = process_and_system();
    let own = (own1 - own0) as f64 / wall;
    // (The other processes': the system's busy share of its processors' time, in cores, less
    // this one's.)
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get()) as f64;
    let (busy, all) = (sys1.0 - sys0.0, sys1.1 - sys0.1);
    let others = if all > 0 {
        (busy as f64 / all as f64 * cores - own).max(0.0)
    } else {
        f64::NAN
    };
    let demoted = ca72_rt::demotions() - demoted0;
    if let (Some(h), Some(t)) = (&host, token) {
        h.leave(t);
    }
    if rt {
        ca72_rt::demote();
    }
    ca72_rt::set_flush_mode(was);
    drop(buffer);
    let silent = silent_runs(&p);
    p.deactivate();
    drop(p);

    took.sort_by(f64::total_cmp);
    let mean = took.iter().sum::<f64>() / took.len() as f64;
    let p99 = took[(took.len() * 99 / 100).min(took.len() - 1)];
    let most = *took.last().unwrap();
    println!(
        "RESULT sound={name} quality={} block={block} rate={rate} seconds={seconds} rt={rt} mean={:.2} p99={:.2} most={:.1} late={late} blocks={blocks} unheard={silent} host_cpu={:.2} total_cpu={:.2} others={:.3} demoted={demoted} peak={peak:.3}",
        match quality {
            QualityMode::Lo => "lo",
            QualityMode::Hi => "hi",
            QualityMode::Ultra => "ultra",
        },
        100.0 * mean,
        100.0 * p99,
        100.0 * most,
        100.0 * host_cpu,
        100.0 * own,
        others,
    );
}
