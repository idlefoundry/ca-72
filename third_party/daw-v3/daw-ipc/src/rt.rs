//! Real-time scheduling for audio threads (DESIGN 9.2): the threads that process audio
//! (a driver's callback thread, the engine's workers, a plugin helper's processing
//! threads) wake every block and must not wait behind ordinary work.
//!
//! - macOS: the Mach time-constraint policy, with the block period;
//! - Linux: `SCHED_FIFO`, which needs an rtprio allowance (`/etc/security/limits.conf`,
//!   rtkit or the `audio` group); without it the thread stays as it was;
//! - Windows: time-critical thread priority.
//!
//! On Linux a thread at `SCHED_FIFO` that never stops (an overloaded engine, a bug)
//! keeps everything below its priority off its CPU, the kernel's interrupt threads
//! included; where the system's real-time throttling is off (`sched_rt_runtime_us` equal
//! to the period, as on the owner's machine) nothing stops it, and the machine stalls.
//! So the first thread promoted starts a watchdog ([`demotions`]): a thread on each CPU at
//! ordinary priority beats every 50 ms, and a thread above the audio threads looks every
//! 100 ms; when a CPU's beat is [`STARVED`] old while one of this process's real-time
//! threads runs there, every real-time thread of the process is made ordinary
//! (`SCHED_OTHER`), and it is counted and said on stderr. The audio then runs on, late
//! if it must, and the machine answers.

#![allow(unsafe_code)]

use std::time::Duration;

/// Promotes the calling thread for audio work that repeats every `period` and needs up
/// to `computation` of it. Returns an explanation when the system refuses.
pub fn promote(period: Duration, computation: Duration) -> Result<(), String> {
    let r = sys::promote(period, computation);
    #[cfg(target_os = "linux")]
    if r.is_ok() {
        watchdog::start();
    }
    r
}

/// Makes the calling thread ordinary again (Linux: `SCHED_OTHER`; elsewhere nothing).
pub fn demote() {
    #[cfg(target_os = "linux")]
    {
        let param = libc::sched_param { sched_priority: 0 };
        // SAFETY: pid 0 is the calling thread; `param` is valid.
        unsafe { libc::sched_setscheduler(0, libc::SCHED_OTHER, &param) };
    }
}

/// Starts the real-time watchdog now (Linux; once per process). [`promote`] starts it
/// too, but a thread may be promoted where starting threads should not happen (in an audio
/// thread's scope, which counts allocations): call this first from outside.
pub fn watch() {
    #[cfg(target_os = "linux")]
    watchdog::start();
}

/// How long a CPU's ordinary threads may go without running while one of this process's
/// real-time threads runs there before the watchdog makes them ordinary (Linux).
pub const STARVED: Duration = Duration::from_millis(500);

/// How many times the watchdog has made this process's real-time threads ordinary
/// (Linux; 0 elsewhere).
pub fn demotions() -> u64 {
    #[cfg(target_os = "linux")]
    return watchdog::DEMOTIONS.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(not(target_os = "linux"))]
    0
}

/// Whether the calling thread runs at real-time priority (Linux: `SCHED_FIFO` or
/// `SCHED_RR`; elsewhere false).
pub fn is_realtime() -> bool {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: pid 0 is the calling thread.
        let p = unsafe { libc::sched_getscheduler(0) };
        p == libc::SCHED_FIFO || p == libc::SCHED_RR
    }
    #[cfg(not(target_os = "linux"))]
    false
}

/// The calling thread's id (Linux; 0 elsewhere), for [`pin`].
#[cfg(target_os = "linux")]
pub fn tid() -> i32 {
    // SAFETY: no arguments, always succeeds.
    unsafe { libc::gettid() }
}

#[cfg(not(target_os = "linux"))]
pub fn tid() -> i32 {
    0
}

/// Pins thread `tid` (0: the calling thread) to one logical CPU (Linux; elsewhere it does
/// nothing). For measurements (`mm-lab worst`): the DAW does not pin its threads.
#[cfg(target_os = "linux")]
pub fn pin(tid: i32, cpu: usize) -> Result<(), String> {
    // SAFETY: a zeroed cpu_set_t is a valid empty set.
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(cpu, &mut set);
        if libc::sched_setaffinity(tid, std::mem::size_of::<libc::cpu_set_t>(), &set) == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error().to_string())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn pin(_tid: i32, _cpu: usize) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
mod sys {
    use std::time::Duration;

    // From <mach/mach_time.h> and <mach/thread_policy.h> (declared here: libc's copies are
    // deprecated in favour of another crate).
    #[repr(C)]
    struct MachTimebaseInfo {
        numer: u32,
        denom: u32,
    }

    #[repr(C)]
    struct TimeConstraintPolicy {
        period: u32,
        computation: u32,
        constraint: u32,
        preemptible: i32,
    }

    const THREAD_TIME_CONSTRAINT_POLICY: u32 = 2;
    const THREAD_TIME_CONSTRAINT_POLICY_COUNT: u32 = 4;

    unsafe extern "C" {
        fn mach_timebase_info(info: *mut MachTimebaseInfo) -> i32;
        fn mach_thread_self() -> u32;
        fn thread_policy_set(thread: u32, flavor: u32, info: *mut i32, count: u32) -> i32;
    }

    pub fn promote(period: Duration, computation: Duration) -> Result<(), String> {
        let mut tb = MachTimebaseInfo { numer: 0, denom: 0 };
        // SAFETY: `tb` is a valid, writable timebase structure.
        if unsafe { mach_timebase_info(&mut tb) } != 0 || tb.numer == 0 {
            return Err("no Mach timebase".into());
        }
        // Nanoseconds to Mach absolute time units.
        let abs = |d: Duration| -> u32 {
            (d.as_nanos() * u128::from(tb.denom) / u128::from(tb.numer)).min(u128::from(u32::MAX))
                as u32
        };
        let mut policy = TimeConstraintPolicy {
            period: abs(period),
            computation: abs(computation.min(period)),
            constraint: abs(period),
            preemptible: 1,
        };
        // SAFETY: setting our own thread's policy with a correctly sized structure (four
        // 32-bit fields, the policy's count).
        let kr = unsafe {
            thread_policy_set(
                mach_thread_self(),
                THREAD_TIME_CONSTRAINT_POLICY,
                (&mut policy as *mut TimeConstraintPolicy).cast(),
                THREAD_TIME_CONSTRAINT_POLICY_COUNT,
            )
        };
        if kr == 0 {
            Ok(())
        } else {
            Err(format!("thread_policy_set failed ({kr})"))
        }
    }
}

/// The starvation watchdog (Linux; see the module's documentation).
#[cfg(target_os = "linux")]
mod watchdog {
    use super::STARVED;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    pub static DEMOTIONS: AtomicU64 = AtomicU64::new(0);

    /// Each CPU's canary's last beat: milliseconds since `epoch` (0 until its first, so a
    /// canary that never runs counts as starved from the start).
    struct Beats {
        epoch: Instant,
        cpus: Vec<(usize, AtomicU64)>,
    }

    static STARTED: OnceLock<()> = OnceLock::new();

    const BEAT: Duration = Duration::from_millis(50);
    const LOOK: Duration = Duration::from_millis(100);

    /// Starts the canaries and the watchdog, once per process.
    pub fn start() {
        STARTED.get_or_init(|| {
            let cpus = allowed_cpus();
            if cpus.is_empty() {
                return;
            }
            let beats = Box::leak(Box::new(Beats {
                epoch: Instant::now(),
                cpus: cpus.iter().map(|&c| (c, AtomicU64::new(0))).collect(),
            }));
            let beats: &'static Beats = beats;
            for k in 0..beats.cpus.len() {
                let _ = std::thread::Builder::new()
                    .name("daw-rt-canary".into())
                    .spawn(move || canary(beats, k));
            }
            let _ = std::thread::Builder::new()
                .name("daw-rt-watchdog".into())
                .spawn(move || watch(beats));
        });
    }

    fn allowed_cpus() -> Vec<usize> {
        // SAFETY: a zeroed cpu_set_t is valid; pid 0 is the calling thread.
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            if libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &mut set) != 0 {
                return Vec::new();
            }
            (0..libc::CPU_SETSIZE as usize)
                .filter(|&c| libc::CPU_ISSET(c, &set))
                .collect()
        }
    }

    fn millis(b: &Beats) -> u64 {
        b.epoch.elapsed().as_millis() as u64
    }

    /// Beats on CPU `k` at ordinary priority (a thread made by a real-time one inherits
    /// its policy).
    fn canary(b: &'static Beats, k: usize) {
        let param = libc::sched_param { sched_priority: 0 };
        // SAFETY: pid 0 is the calling thread; `param` is valid.
        unsafe { libc::sched_setscheduler(0, libc::SCHED_OTHER, &param) };
        let (cpu, beat) = (&b.cpus[k].0, &b.cpus[k].1);
        if super::pin(0, *cpu).is_err() {
            // Not ours to run on after all: never counted as starved.
            beat.store(u64::MAX, Ordering::Relaxed);
            return;
        }
        loop {
            beat.store(millis(b), Ordering::Relaxed);
            std::thread::sleep(BEAT);
        }
    }

    /// Looks at the beats from above the audio threads' priority.
    fn watch(b: &'static Beats) {
        // The highest priority allowed (RLIMIT_RTPRIO), at most 99.
        let top = {
            let mut r = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            // SAFETY: a valid, writable rlimit.
            if unsafe { libc::getrlimit(libc::RLIMIT_RTPRIO, &mut r) } == 0 {
                r.rlim_cur.min(99) as i32
            } else {
                0
            }
        };
        let param = libc::sched_param {
            sched_priority: top,
        };
        // SAFETY: pid 0 is the calling thread; `param` is valid.
        let ok = top > 0 && unsafe { libc::sched_setscheduler(0, libc::SCHED_FIFO, &param) } == 0;
        if !ok || top <= 70 {
            eprintln!(
                "daw: the real-time watchdog cannot run above the audio threads (real-time priority allowed: {top})"
            );
        }
        // SAFETY: no arguments, always succeeds.
        let me = unsafe { libc::gettid() };
        loop {
            std::thread::sleep(LOOK);
            let now = millis(b);
            for (cpu, beat) in &b.cpus {
                let last = beat.load(Ordering::Relaxed);
                // Not a CPU it may run on.
                if last == u64::MAX {
                    continue;
                }
                if now.saturating_sub(last) < STARVED.as_millis() as u64 {
                    continue;
                }
                let ours = realtime_threads(me);
                if ours.iter().any(|t| t.running_on == Some(*cpu)) {
                    for t in &ours {
                        let param = libc::sched_param { sched_priority: 0 };
                        // SAFETY: a thread of this process; `param` is valid.
                        unsafe { libc::sched_setscheduler(t.tid, libc::SCHED_OTHER, &param) };
                    }
                    DEMOTIONS.fetch_add(1, Ordering::Relaxed);
                    eprintln!(
                        "daw: CPU {cpu}'s ordinary threads had not run for {} ms under this process's real-time threads: its {} real-time threads now run at ordinary priority",
                        now.saturating_sub(last),
                        ours.len()
                    );
                    break;
                }
            }
        }
    }

    struct RtThread {
        tid: i32,
        /// The CPU it is running on, if it is running.
        running_on: Option<usize>,
    }

    /// This process's threads at real-time priority but `me`.
    fn realtime_threads(me: i32) -> Vec<RtThread> {
        let Ok(dir) = std::fs::read_dir("/proc/self/task") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for e in dir.flatten() {
            let Some(tid) = e.file_name().to_str().and_then(|n| n.parse::<i32>().ok()) else {
                continue;
            };
            if tid == me {
                continue;
            }
            // SAFETY: a thread id; an error (the thread has gone) is -1.
            let p = unsafe { libc::sched_getscheduler(tid) };
            if p != libc::SCHED_FIFO && p != libc::SCHED_RR {
                continue;
            }
            // `/proc/self/task/<tid>/stat`: after the name's closing parenthesis, the state
            // (field 3) and, 36 fields on, the CPU it last ran on (field 39).
            let running_on = std::fs::read_to_string(e.path().join("stat"))
                .ok()
                .and_then(|st| {
                    let rest = &st[st.rfind(')')? + 1..];
                    let f: Vec<&str> = rest.split_whitespace().collect();
                    (f.first() == Some(&"R"))
                        .then(|| f.get(36)?.parse().ok())
                        .flatten()
                });
            out.push(RtThread { tid, running_on });
        }
        out
    }
}

#[cfg(target_os = "linux")]
mod sys {
    use std::time::Duration;

    pub fn promote(_period: Duration, _computation: Duration) -> Result<(), String> {
        let param = libc::sched_param { sched_priority: 70 };
        // SAFETY: pid 0 is the calling thread; `param` is valid.
        if unsafe { libc::sched_setscheduler(0, libc::SCHED_FIFO, &param) } == 0 {
            Ok(())
        } else {
            Err(format!(
                "SCHED_FIFO refused ({}); allow real-time priority for this user to use it",
                std::io::Error::last_os_error()
            ))
        }
    }
}

#[cfg(windows)]
mod sys {
    use std::time::Duration;
    use windows_sys::Win32::System::Threading::{
        GetCurrentThread, SetThreadInformation, SetThreadPriority,
        THREAD_POWER_THROTTLING_CURRENT_VERSION, THREAD_POWER_THROTTLING_EXECUTION_SPEED,
        THREAD_POWER_THROTTLING_STATE, THREAD_PRIORITY_TIME_CRITICAL, ThreadPowerThrottling,
    };

    pub fn promote(_period: Duration, _computation: Duration) -> Result<(), String> {
        // Not throttled for power (EcoQoS, DSEM): a lightly loaded thread of a process
        // without a foreground window is otherwise run on efficiency cores and at low
        // clocks, and an audio thread's blocks then take two to four times as long. A hint
        // for this thread only; refused on Windows before 10 1709, which is no harm.
        let state = THREAD_POWER_THROTTLING_STATE {
            Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: THREAD_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: 0,
        };
        // SAFETY: the pseudo-handle of the calling thread; `state` is a valid structure and
        // its size is the one given.
        let _ = unsafe {
            SetThreadInformation(
                GetCurrentThread(),
                ThreadPowerThrottling,
                (&raw const state).cast(),
                std::mem::size_of::<THREAD_POWER_THROTTLING_STATE>() as u32,
            )
        };
        // SAFETY: the pseudo-handle of the calling thread.
        if unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL) } != 0 {
            Ok(())
        } else {
            Err(format!(
                "SetThreadPriority failed ({})",
                std::io::Error::last_os_error()
            ))
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
mod sys {
    use std::time::Duration;

    pub fn promote(_period: Duration, _computation: Duration) -> Result<(), String> {
        Err("real-time scheduling is not implemented on this platform".into())
    }
}
