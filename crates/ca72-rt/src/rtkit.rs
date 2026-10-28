//! RTKit, for the standalone application's audio threads where Linux refuses them `SCHED_FIFO`
//! (decisions.md R-RTKIT). Many systems (Ubuntu's, Fedora's and others') give a desktop user's
//! programs real-time priority only through RTKit, a system service on D-Bus
//! (`org.freedesktop.RealtimeKit1` at `/org/freedesktop/RealtimeKit1` on the system bus), not
//! through an rtprio allowance (`RLIMIT_RTPRIO`).
//!
//! - **Never on the audio threads.** A thread refused [`ask`]s: its kernel id goes into a table
//!   of atomics and this module's thread (`ca72-rt-rtkit`, ordinary) is woken, which makes the
//!   D-Bus calls (they block) and records the answer in the thread's [`Group`].
//! - **The calls.** D-Bus's wire protocol over the system bus's socket, spoken here: the
//!   `EXTERNAL` authentication, `Hello`, `Properties.Get` of `MaxRealtimePriority`,
//!   `MinNiceLevel` and `RTTimeUSecMax`, then `MakeThreadRealtimeWithPID` (this process, the
//!   thread, the priority) for each thread, `MakeThreadRealtime` where RTKit is older than that
//!   method. Little-endian messages out; either order in. No crate and no C library: `zbus`
//!   brings an async executor and a tree of crates to the standalone for four calls, and the
//!   `dbus` crate links libdbus, a library and headers more for the build, CI and the Linux
//!   package to carry.
//! - **The priority** asked is the workers' 70 ([`PRIORITY`]), at most RTKit's
//!   `MaxRealtimePriority` (20 where RTKit is as it comes). RTKit gives `SCHED_RR`.
//! - **RLIMIT_RTTIME.** RTKit refuses a process whose hard `RLIMIT_RTTIME` (the processor time
//!   a real-time thread may take without blocking) is above its `RTTimeUSecMax` (200 ms as it
//!   comes): a thread past the soft limit is sent SIGXCPU (which ends the process unless it is
//!   handled), and at the hard limit SIGKILL. So before the first request the hard limit is set
//!   to `RTTimeUSecMax` but at most [`HARD_US`] (200 ms), and the soft limit 50 ms below it
//!   ([`MARGIN_US`]; half the hard limit if that is less): 150 ms. A block's work, even 8192
//!   frames at 44.1 kHz (186 ms) loaded to half its period, stays under it, as each block ends
//!   blocking on the device and a worker parks between blocks. SIGXCPU is handled here: every
//!   thread promoted by this crate is made ordinary at once ([`super::watchdog::demote_all`]),
//!   the soft limit is put back (the kernel moves it a second on), and the watchdog says so.
//!   50 ms is many scheduler ticks for the handler to run before SIGKILL could come. Without
//!   an rtprio allowance the watchdog cannot run above RTKit's threads; this stands in for it,
//!   as RTKit's own canary does for the system.
//! - **What is not done:** RTKit's `MakeThreadHighPriority` (a nice level, `MinNiceLevel` read
//!   and said only), and the effects' helper processes (decisions.md R-RTKIT).

use super::{Group, Why, watchdog};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, AtomicU64, Ordering};
use std::thread::Thread;
use std::time::Duration;

/// The priority asked for: the workers' (`sys::promote`), at most RTKit's most.
pub const PRIORITY: u32 = 70;
/// The hard `RLIMIT_RTTIME` set, at most (microseconds): RTKit's own default most.
pub const HARD_US: u64 = 200_000;
/// The soft limit's distance below the hard one (microseconds), at most half of it.
pub const MARGIN_US: u64 = 50_000;

const RTKIT: &str = "org.freedesktop.RealtimeKit1";
const RTKIT_PATH: &str = "/org/freedesktop/RealtimeKit1";
const SYSTEM_BUS: &str = "unix:path=/var/run/dbus/system_bus_socket";
/// How long a call may take (RTKit may first be started by the bus).
const TIMEOUT: Duration = Duration::from_secs(5);

static ENABLED: AtomicBool = AtomicBool::new(false);
static BROKER: OnceLock<Thread> = OnceLock::new();

/// A thread waiting for its answer: its kernel id (0: a free place; -1: being filled), its
/// group and the set of the group's threads it is of.
struct Slot {
    tid: AtomicI32,
    group: AtomicPtr<Group>,
    set: AtomicU32,
}

static QUEUE: [Slot; 64] = [const {
    Slot {
        tid: AtomicI32::new(0),
        group: AtomicPtr::new(std::ptr::null_mut()),
        set: AtomicU32::new(0),
    }
}; 64];

/// SIGXCPUs handled (the watchdog says each), and the limits put back after one.
pub static XCPU: AtomicU64 = AtomicU64::new(0);
static SOFT: AtomicU64 = AtomicU64::new(0);
static HARD: AtomicU64 = AtomicU64::new(0);

/// The soft `RLIMIT_RTTIME` set, in milliseconds (0: none set).
pub fn soft_ms() -> u64 {
    SOFT.load(Ordering::Relaxed) / 1000
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

/// From now on a refused thread is asked of RTKit; its thread started (once).
pub fn enable() {
    BROKER.get_or_init(|| {
        let h = std::thread::Builder::new()
            .name("ca72-rt-rtkit".into())
            .spawn(serve)
            .expect("RTKit's thread");
        h.thread().clone()
    });
    ENABLED.store(true, Ordering::Release);
}

/// Thread `tid` (the calling one, refused) asked of RTKit through this module's thread: Ok,
/// or why it was not (RTKit not in use, or too many waiting). Atomics and a wake only: on the
/// audio thread.
pub fn ask(tid: i32, group: &'static Group) -> Result<(), Why> {
    if !enabled() {
        return Err(Why::Refused);
    }
    let Some(broker) = BROKER.get() else {
        return Err(Why::Refused);
    };
    for s in &QUEUE {
        if s.tid
            .compare_exchange(0, -1, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            s.group
                .store(std::ptr::from_ref(group).cast_mut(), Ordering::Relaxed);
            s.set.store(group.set(), Ordering::Relaxed);
            s.tid.store(tid, Ordering::Release);
            broker.unpark();
            return Ok(());
        }
    }
    Err(Why::Full)
}

/// RTKit's thread: parked until a thread is waiting, then each asked for, one at a time.
fn serve() {
    let mut kit: Option<Kit> = None;
    // (What has been said on standard error, once a process: by group and answer.)
    let mut said: Vec<(usize, bool)> = Vec::new();
    loop {
        std::thread::park();
        for s in &QUEUE {
            let tid = s.tid.load(Ordering::Acquire);
            if tid <= 0 {
                continue;
            }
            // SAFETY: stored by `ask` from a `&'static Group` before the id was.
            let group: &'static Group = unsafe { &*s.group.load(Ordering::Relaxed) };
            let set = s.set.load(Ordering::Relaxed);
            // (A thread that has ended meanwhile, or ends as RTKit answers, is let go: its id may
            // be another's soon, and RTKit's refusal of it says nothing. So is one of a set of
            // the group's threads that a newer has replaced: a stream or a pool begun again.)
            let answer = if watchdog::holds(tid) && group.set() == set {
                Some(ask_now(&mut kit, tid))
            } else {
                None
            };
            let answer = answer.filter(|_| watchdog::holds(tid));
            let first = |said: &mut Vec<(usize, bool)>, ok| {
                let key = (std::ptr::from_ref(group) as usize, ok);
                let new = !said.contains(&key);
                if new {
                    said.push(key);
                }
                new
            };
            match answer {
                Some(Ok(priority)) => {
                    watchdog::wake();
                    if group.answer(set, Some(Ok(()))) && first(&mut said, true) {
                        eprintln!(
                            "CA-72: {} real-time through RTKit (SCHED_RR {priority}; RLIMIT_RTTIME {} ms, then SIGXCPU and ordinary priority)",
                            group.name(),
                            soft_ms()
                        );
                    }
                }
                Some(Err((why, detail))) => {
                    if group.answer(set, Some(Err(why))) && first(&mut said, false) {
                        eprintln!(
                            "CA-72: {} at ordinary priority: SCHED_FIFO refused (no rtprio allowance) and {detail}. With other work on the computer the sound may drop out (decisions.md R-DROPOUTS, R-RTKIT).",
                            group.name()
                        );
                    }
                }
                None => {
                    group.answer(set, None);
                }
            }
            s.tid.store(0, Ordering::Release);
        }
    }
}

/// RTKit as found: the connection and its limits.
struct Kit {
    bus: Bus,
    max_priority: i64,
    rttime_max: u64,
}

/// Thread `tid` asked of RTKit (connected first if need be): the priority given, or why not
/// and what to say.
fn ask_now(kit: &mut Option<Kit>, tid: i32) -> Result<u32, (Why, String)> {
    if kit.is_none() {
        *kit = Some(Kit::find()?);
    }
    let Some(k) = kit.as_mut() else {
        return Err((Why::RtkitFailed, "RTKit was not found".into()));
    };
    let r = k.make_realtime(tid);
    // (A connection that failed is made again at the next ask.)
    if matches!(r, Err((Why::RtkitFailed, _))) {
        *kit = None;
    }
    r
}

impl Kit {
    /// RTKit on the system bus, its limits read, this process's RLIMIT_RTTIME set for it and
    /// SIGXCPU handled.
    fn find() -> Result<Kit, (Why, String)> {
        let address =
            std::env::var("DBUS_SYSTEM_BUS_ADDRESS").unwrap_or_else(|_| SYSTEM_BUS.into());
        let mut bus = Bus::connect(&address)
            .map_err(|e| (Why::NoBus, format!("there is no system bus ({e})")))?;
        let mut property = |name: &str| -> Result<i64, (Why, String)> {
            match bus.call(
                RTKIT,
                RTKIT_PATH,
                "org.freedesktop.DBus.Properties",
                "Get",
                &[Arg::Str(RTKIT), Arg::Str(name)],
            ) {
                Ok(Reply::Return { body, big }) => variant_int(&body, big)
                    .ok_or_else(|| (Why::RtkitFailed, format!("RTKit's {name} unreadable"))),
                Ok(Reply::Error { name: e, message }) => Err(refusal(&e, &message)),
                Err(e) => Err((Why::RtkitFailed, format!("RTKit did not answer ({e})"))),
            }
        };
        let max_priority = property("MaxRealtimePriority")?;
        let min_nice = property("MinNiceLevel")?;
        let rttime_max = property("RTTimeUSecMax")?;
        let rttime_max = u64::try_from(rttime_max).unwrap_or(0);
        let (soft, hard) = limit_rttime(rttime_max).map_err(|e| {
            (
                Why::RtkitFailed,
                format!("RLIMIT_RTTIME could not be set for RTKit ({e})"),
            )
        })?;
        handle_xcpu();
        eprintln!(
            "CA-72: RTKit: MaxRealtimePriority {max_priority}, MinNiceLevel {min_nice}, RTTimeUSecMax {rttime_max} us; RLIMIT_RTTIME set to {soft} us (SIGXCPU), {hard} us (SIGKILL)"
        );
        Ok(Kit {
            bus,
            max_priority,
            rttime_max,
        })
    }

    fn make_realtime(&mut self, tid: i32) -> Result<u32, (Why, String)> {
        let priority = u32::try_from(self.max_priority.min(i64::from(PRIORITY))).unwrap_or(0);
        if priority == 0 {
            return Err((
                Why::RtkitRefused,
                "RTKit allows no real-time priority (MaxRealtimePriority 0)".into(),
            ));
        }
        // (Set again before each: PipeWire's ALSA client, in this process, sets RLIMIT_RTTIME
        // itself as it opens a stream, soft and hard alike, which would leave SIGKILL first.)
        limit_rttime(self.rttime_max).map_err(|e| {
            (
                Why::RtkitFailed,
                format!("RLIMIT_RTTIME could not be set for RTKit ({e})"),
            )
        })?;
        let (pid, tid64) = (u64::from(std::process::id()), tid as u64);
        let mut r = self.bus.call(
            RTKIT,
            RTKIT_PATH,
            RTKIT,
            "MakeThreadRealtimeWithPID",
            &[Arg::U64(pid), Arg::U64(tid64), Arg::U32(priority)],
        );
        if matches!(&r, Ok(Reply::Error { name, .. }) if name == "org.freedesktop.DBus.Error.UnknownMethod")
        {
            // (RTKit before 0.11.)
            r = self.bus.call(
                RTKIT,
                RTKIT_PATH,
                RTKIT,
                "MakeThreadRealtime",
                &[Arg::U64(tid64), Arg::U32(priority)],
            );
        }
        match r {
            Ok(Reply::Return { .. }) if watchdog_realtime(tid) => Ok(priority),
            Ok(Reply::Return { .. }) => Err((
                Why::RtkitRefused,
                "RTKit answered, but the thread is still ordinary".into(),
            )),
            Ok(Reply::Error { name, message }) => Err(refusal(&name, &message)),
            Err(e) => Err((Why::RtkitFailed, format!("RTKit did not answer ({e})"))),
        }
    }
}

fn watchdog_realtime(tid: i32) -> bool {
    // SAFETY: a thread id; an error (the thread has gone) is -1.
    let p = unsafe { libc::sched_getscheduler(tid) };
    super::realtime_policy(p)
}

/// Why, from a D-Bus error's name, and what to say.
fn refusal(name: &str, message: &str) -> (Why, String) {
    let absent = [
        "org.freedesktop.DBus.Error.ServiceUnknown",
        "org.freedesktop.DBus.Error.NameHasNoOwner",
    ];
    if absent.contains(&name) || name.starts_with("org.freedesktop.DBus.Error.Spawn.") {
        (Why::NoRtkit, format!("RTKit is not running ({name})"))
    } else if message.is_empty() {
        (Why::RtkitRefused, format!("RTKit refused ({name})"))
    } else {
        (
            Why::RtkitRefused,
            format!("RTKit refused ({name}: {message})"),
        )
    }
}

/// This process's RLIMIT_RTTIME made acceptable to RTKit (module documentation): the hard
/// limit at most `max` (RTKit's `RTTimeUSecMax`) and [`HARD_US`], the soft [`MARGIN_US`] below
/// it (at most half of it); lower limits already set kept. The soft and hard limits set.
fn limit_rttime(max: u64) -> Result<(u64, u64), String> {
    let mut r = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: a valid, writable rlimit.
    if unsafe { libc::getrlimit(libc::RLIMIT_RTTIME, &mut r) } != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let (soft, hard) = rttime_limits(r.rlim_cur, r.rlim_max, max)?;
    r.rlim_cur = soft;
    r.rlim_max = hard;
    // SAFETY: a valid rlimit; only lowered.
    if unsafe { libc::setrlimit(libc::RLIMIT_RTTIME, &r) } != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    SOFT.store(soft, Ordering::Relaxed);
    HARD.store(hard, Ordering::Relaxed);
    Ok((soft, hard))
}

/// The soft RLIMIT_RTTIME kept at what was set for RTKit (the watchdog's, at each look): PipeWire's
/// ALSA client, in this process, sets soft and hard alike as it opens a stream (the device list
/// opens them), which would leave a runaway thread SIGKILL and no SIGXCPU first. Only lowered.
pub fn keep_rttime() {
    let soft = SOFT.load(Ordering::Relaxed);
    if soft == 0 {
        return;
    }
    let mut r = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: a valid, writable rlimit.
    if unsafe { libc::getrlimit(libc::RLIMIT_RTTIME, &mut r) } == 0 && r.rlim_cur > soft {
        let lowered = libc::rlimit {
            rlim_cur: soft.min(r.rlim_max),
            rlim_max: r.rlim_max,
        };
        // SAFETY: a valid rlimit; only the soft limit lowered.
        unsafe { libc::setrlimit(libc::RLIMIT_RTTIME, &lowered) };
    }
}

/// The soft and hard RLIMIT_RTTIME for limits now `soft` and `hard` and RTKit's most `max`.
fn rttime_limits(soft: u64, hard: u64, max: u64) -> Result<(u64, u64), String> {
    if hard < 2 {
        return Err(format!("its hard limit is already {hard} us"));
    }
    let hard = hard.min(max).min(HARD_US);
    if hard < 2 {
        return Err(format!("RTKit allows {max} us"));
    }
    Ok((soft.min(hard - MARGIN_US.min(hard / 2)), hard))
}

/// SIGXCPU (the soft RLIMIT_RTTIME passed) handled: unless something else handles it already.
fn handle_xcpu() {
    // SAFETY: sigaction structures zeroed and filled; the handler only stores atomics and makes
    // system calls.
    unsafe {
        let mut old: libc::sigaction = std::mem::zeroed();
        if libc::sigaction(libc::SIGXCPU, std::ptr::null(), &mut old) != 0
            || old.sa_sigaction != libc::SIG_DFL
        {
            return;
        }
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = on_xcpu as extern "C" fn(libc::c_int) as libc::sighandler_t;
        sa.sa_flags = libc::SA_RESTART;
        libc::sigemptyset(&mut sa.sa_mask);
        libc::sigaction(libc::SIGXCPU, &sa, std::ptr::null_mut());
    }
}

/// A thread of this process ran past the soft RLIMIT_RTTIME without blocking: every thread
/// promoted here made ordinary, the soft limit put back. Async-signal-safe.
extern "C" fn on_xcpu(_: libc::c_int) {
    watchdog::demote_all();
    let r = libc::rlimit {
        rlim_cur: SOFT.load(Ordering::Relaxed),
        rlim_max: HARD.load(Ordering::Relaxed),
    };
    // SAFETY: a valid rlimit, the limits set before (the kernel raised the soft one).
    unsafe { libc::setrlimit(libc::RLIMIT_RTTIME, &r) };
    XCPU.fetch_add(1, Ordering::Release);
}

/// An argument of a call.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Arg<'a> {
    Str(&'a str),
    U32(u32),
    U64(u64),
}

impl Arg<'_> {
    fn code(&self) -> char {
        match self {
            Arg::Str(_) => 's',
            Arg::U32(_) => 'u',
            Arg::U64(_) => 't',
        }
    }
}

/// A call's answer: its body (big-endian or not), or an error's name and message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Reply {
    Return { body: Vec<u8>, big: bool },
    Error { name: String, message: String },
}

/// A connection to a bus, authenticated and said hello.
pub(crate) struct Bus {
    s: UnixStream,
    serial: u32,
}

impl Bus {
    /// Connected to the first address of `address` (D-Bus's `unix:path=...;unix:abstract=...`)
    /// that answers.
    pub(crate) fn connect(address: &str) -> std::io::Result<Bus> {
        let mut last = std::io::Error::other(format!("no usable address in {address:?}"));
        for a in address.split(';').filter(|a| !a.is_empty()) {
            match socket(a) {
                Ok(s) => {
                    s.set_read_timeout(Some(TIMEOUT))?;
                    s.set_write_timeout(Some(TIMEOUT))?;
                    let mut bus = Bus { s, serial: 0 };
                    bus.authenticate()?;
                    match bus.call(
                        "org.freedesktop.DBus",
                        "/org/freedesktop/DBus",
                        "org.freedesktop.DBus",
                        "Hello",
                        &[],
                    )? {
                        Reply::Return { .. } => return Ok(bus),
                        Reply::Error { name, .. } => {
                            last = std::io::Error::other(format!("Hello refused ({name})"));
                        }
                    }
                }
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    /// SASL's EXTERNAL, as this process's user.
    fn authenticate(&mut self) -> std::io::Result<()> {
        // SAFETY: no arguments, always succeeds.
        let uid = unsafe { libc::geteuid() };
        let hex: String = uid
            .to_string()
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect();
        self.s
            .write_all(format!("\0AUTH EXTERNAL {hex}\r\n").as_bytes())?;
        let line = self.line()?;
        if !line.starts_with("OK ") {
            return Err(std::io::Error::other(format!(
                "the bus refused this user ({})",
                line.trim()
            )));
        }
        self.s.write_all(b"BEGIN\r\n")
    }

    /// A line of the authentication, its "\r\n" included.
    fn line(&mut self) -> std::io::Result<String> {
        let mut line = Vec::new();
        let mut b = [0u8; 1];
        while !line.ends_with(b"\r\n") {
            self.s.read_exact(&mut b)?;
            line.push(b[0]);
            if line.len() > 4096 {
                return Err(std::io::Error::other("an authentication line too long"));
            }
        }
        Ok(String::from_utf8_lossy(&line).into_owned())
    }

    /// Calls `member` of `interface` on `path` of `destination` with `args`, and waits for its
    /// answer (other messages that come meanwhile let go).
    pub(crate) fn call(
        &mut self,
        destination: &str,
        path: &str,
        interface: &str,
        member: &str,
        args: &[Arg<'_>],
    ) -> std::io::Result<Reply> {
        self.serial += 1;
        let serial = self.serial;
        self.s.write_all(&method_call(
            serial,
            destination,
            path,
            interface,
            member,
            args,
        ))?;
        for _ in 0..256 {
            let m = self.message()?;
            if m.reply_to != Some(serial) {
                continue;
            }
            return Ok(match m.kind {
                ERROR => Reply::Error {
                    name: m.error.unwrap_or_default(),
                    // (Its first argument, if a string, says what went wrong.)
                    message: if m.signature.as_deref().is_some_and(|s| s.starts_with('s')) {
                        Reader::new(&m.body, m.big).string().unwrap_or_default()
                    } else {
                        String::new()
                    },
                },
                _ => Reply::Return {
                    body: m.body,
                    big: m.big,
                },
            });
        }
        Err(std::io::Error::other("no answer among the bus's messages"))
    }

    /// The next message from the bus.
    fn message(&mut self) -> std::io::Result<Message> {
        let mut fixed = [0u8; 16];
        self.s.read_exact(&mut fixed)?;
        let big = match fixed[0] {
            b'l' => false,
            b'B' => true,
            _ => return Err(std::io::Error::other("not a D-Bus message")),
        };
        let word = |at: usize| {
            let b = [fixed[at], fixed[at + 1], fixed[at + 2], fixed[at + 3]];
            if big {
                u32::from_be_bytes(b)
            } else {
                u32::from_le_bytes(b)
            }
        };
        let (body_len, fields_len) = (word(4) as usize, word(12) as usize);
        if body_len + fields_len > 1 << 20 {
            return Err(std::io::Error::other("a D-Bus message too long"));
        }
        // (The fields, then padding to 8, then the body.)
        let head = (16 + fields_len).div_ceil(8) * 8;
        let mut all = fixed.to_vec();
        all.resize(head + body_len, 0);
        self.s.read_exact(&mut all[16..])?;
        parse(all, big).ok_or_else(|| std::io::Error::other("a D-Bus message unreadable"))
    }
}

/// A unix socket for one address (`unix:path=...` or `unix:abstract=...`).
fn socket(address: &str) -> std::io::Result<UnixStream> {
    let rest = address
        .strip_prefix("unix:")
        .ok_or_else(|| std::io::Error::other(format!("not a unix address: {address}")))?;
    for kv in rest.split(',') {
        if let Some(p) = kv.strip_prefix("path=") {
            return UnixStream::connect(unescape(p));
        }
        if let Some(n) = kv.strip_prefix("abstract=") {
            use std::os::linux::net::SocketAddrExt;
            let a = std::os::unix::net::SocketAddr::from_abstract_name(unescape(n).as_bytes())?;
            return UnixStream::connect_addr(&a);
        }
    }
    Err(std::io::Error::other(format!(
        "no path in the address {address}"
    )))
}

/// A D-Bus address's value, its `%xx` escapes undone.
fn unescape(v: &str) -> String {
    let b = v.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(x) = v
                .get(i + 1..i + 3)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(x);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

const METHOD_CALL: u8 = 1;
const ERROR: u8 = 3;

/// A method call's bytes (little-endian).
pub(crate) fn method_call(
    serial: u32,
    destination: &str,
    path: &str,
    interface: &str,
    member: &str,
    args: &[Arg<'_>],
) -> Vec<u8> {
    let mut w = Writer::default();
    w.byte(b'l');
    w.byte(METHOD_CALL);
    w.byte(0);
    w.byte(1);
    w.u32(0); // the body's length, filled in below
    w.u32(serial);
    w.u32(0); // the fields' length, filled in below
    let start = w.0.len();
    let signature: String = args.iter().map(Arg::code).collect();
    let field = |w: &mut Writer, code: u8, kind: &str, value: &str| {
        w.pad(8);
        w.byte(code);
        w.signature(kind);
        match kind {
            "g" => w.signature(value),
            _ => w.string(value),
        }
    };
    field(&mut w, 1, "o", path);
    field(&mut w, 2, "s", interface);
    field(&mut w, 3, "s", member);
    field(&mut w, 6, "s", destination);
    if !signature.is_empty() {
        field(&mut w, 8, "g", &signature);
    }
    let fields = (w.0.len() - start) as u32;
    w.0[12..16].copy_from_slice(&fields.to_le_bytes());
    w.pad(8);
    let body = w.0.len();
    for a in args {
        match *a {
            Arg::Str(s) => w.string(s),
            Arg::U32(v) => w.u32(v),
            Arg::U64(v) => w.u64(v),
        }
    }
    let len = (w.0.len() - body) as u32;
    w.0[4..8].copy_from_slice(&len.to_le_bytes());
    w.0
}

/// D-Bus's marshalling, little-endian (alignment from the message's start).
#[derive(Default)]
struct Writer(Vec<u8>);

impl Writer {
    fn pad(&mut self, to: usize) {
        while !self.0.len().is_multiple_of(to) {
            self.0.push(0);
        }
    }
    fn byte(&mut self, b: u8) {
        self.0.push(b);
    }
    fn u32(&mut self, v: u32) {
        self.pad(4);
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.pad(8);
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn string(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
        self.0.push(0);
    }
    fn signature(&mut self, s: &str) {
        self.0.push(s.len() as u8);
        self.0.extend_from_slice(s.as_bytes());
        self.0.push(0);
    }
}

/// A message read: its kind, the call it answers, an error's name, its body's signature and
/// its body.
#[derive(Debug)]
pub(crate) struct Message {
    kind: u8,
    reply_to: Option<u32>,
    error: Option<String>,
    signature: Option<String>,
    body: Vec<u8>,
    big: bool,
}

/// A whole message's bytes read (None: not one this understands).
pub(crate) fn parse(all: Vec<u8>, big: bool) -> Option<Message> {
    let mut r = Reader {
        b: &all,
        at: 12,
        big,
    };
    let fields = r.u32()? as usize;
    let end = 16 + fields;
    let (mut reply_to, mut error, mut signature) = (None, None, None);
    while r.at < end {
        r.align(8);
        let code = r.byte()?;
        let kind = r.signature()?;
        match (code, kind.as_str()) {
            (5, "u") => reply_to = Some(r.u32()?),
            (4, "s") => error = Some(r.string()?),
            (8, "g") => signature = Some(r.signature()?),
            (_, "s" | "o") => {
                r.string()?;
            }
            (_, "u") => {
                r.u32()?;
            }
            (_, "g") => {
                r.signature()?;
            }
            _ => return None,
        }
    }
    let head = end.div_ceil(8) * 8;
    Some(Message {
        kind: *all.get(1)?,
        reply_to,
        error,
        signature,
        body: all.get(head..)?.to_vec(),
        big,
    })
}

/// D-Bus's unmarshalling (alignment from the start of `b`).
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
    big: bool,
}

impl<'a> Reader<'a> {
    fn new(b: &'a [u8], big: bool) -> Self {
        Reader { b, at: 0, big }
    }
    fn align(&mut self, to: usize) {
        self.at = self.at.div_ceil(to) * to;
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.at..self.at + n)?;
        self.at += n;
        Some(s)
    }
    fn byte(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn u32(&mut self) -> Option<u32> {
        self.align(4);
        let b: [u8; 4] = self.take(4)?.try_into().ok()?;
        Some(if self.big {
            u32::from_be_bytes(b)
        } else {
            u32::from_le_bytes(b)
        })
    }
    fn u64(&mut self) -> Option<u64> {
        self.align(8);
        let b: [u8; 8] = self.take(8)?.try_into().ok()?;
        Some(if self.big {
            u64::from_be_bytes(b)
        } else {
            u64::from_le_bytes(b)
        })
    }
    fn string(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        let s = String::from_utf8(self.take(n)?.to_vec()).ok()?;
        self.take(1)?;
        Some(s)
    }
    fn signature(&mut self) -> Option<String> {
        let n = self.byte()? as usize;
        let s = String::from_utf8(self.take(n)?.to_vec()).ok()?;
        self.take(1)?;
        Some(s)
    }
}

/// A `Properties.Get` answer's value, a variant of an integer (`i`, `u`, `x` or `t`).
fn variant_int(body: &[u8], big: bool) -> Option<i64> {
    let mut r = Reader::new(body, big);
    Some(match r.signature()?.as_str() {
        "i" => i64::from(r.u32()? as i32),
        "u" => i64::from(r.u32()?),
        "x" => r.u64()? as i64,
        "t" => i64::try_from(r.u64()?).ok()?,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A call's bytes as D-Bus lays them out: the fixed header, the fields each at 8, the body
    /// at 8, its integers at their own alignment.
    #[test]
    fn a_call_is_marshalled_as_dbus_lays_it_out() {
        let m = method_call(
            7,
            RTKIT,
            RTKIT_PATH,
            RTKIT,
            "MakeThreadRealtimeWithPID",
            &[Arg::U64(1234), Arg::U64(5678), Arg::U32(20)],
        );
        assert_eq!(&m[..4], b"l\x01\x00\x01");
        // (Two u64s and a u32: no padding after the last.)
        assert_eq!(u32::from_le_bytes(m[4..8].try_into().unwrap()), 20);
        assert_eq!(u32::from_le_bytes(m[8..12].try_into().unwrap()), 7);
        let fields = u32::from_le_bytes(m[12..16].try_into().unwrap()) as usize;
        let body = (16 + fields).div_ceil(8) * 8;
        assert_eq!(m.len(), body + 20);
        assert_eq!(
            u64::from_le_bytes(m[body..body + 8].try_into().unwrap()),
            1234
        );
        assert_eq!(
            u64::from_le_bytes(m[body + 8..body + 16].try_into().unwrap()),
            5678
        );
        assert_eq!(
            u32::from_le_bytes(m[body + 16..body + 20].try_into().unwrap()),
            20
        );
        // Read back as a message: its fields understood, the signature among them.
        let back = parse(m, false).expect("its own call read");
        assert_eq!(back.kind, METHOD_CALL);
        assert_eq!(back.signature.as_deref(), Some("ttu"));
        assert_eq!(back.body.len(), 20);
    }

    /// An error answer and a property's variant, in both byte orders.
    #[test]
    fn answers_are_read_in_either_byte_order() {
        // A variant `x` of 200000 after its signature: "\x01x\0" then padding to 8.
        let mut le = vec![1, b'x', 0, 0, 0, 0, 0, 0];
        le.extend_from_slice(&200_000u64.to_le_bytes());
        assert_eq!(variant_int(&le, false), Some(200_000));
        let mut be = vec![1, b'i', 0, 0];
        be.extend_from_slice(&20u32.to_be_bytes());
        assert_eq!(variant_int(&be, true), Some(20));
        let mut neg = vec![1, b'i', 0, 0];
        neg.extend_from_slice(&(-15i32).to_le_bytes());
        assert_eq!(variant_int(&neg, false), Some(-15));
        assert_eq!(
            refusal("org.freedesktop.DBus.Error.ServiceUnknown", "").0,
            Why::NoRtkit
        );
        assert_eq!(
            refusal(
                "org.freedesktop.DBus.Error.AccessDenied",
                "Operation not permitted"
            )
            .0,
            Why::RtkitRefused
        );
    }

    /// RLIMIT_RTTIME as set for RTKit: 150 ms then 200 ms as RTKit comes; never above what
    /// RTKit allows or what was set before; half the hard limit where that is small.
    #[test]
    fn the_rttime_limits_are_rtkit_s_and_leave_the_handler_time() {
        let inf = libc::RLIM_INFINITY;
        assert_eq!(rttime_limits(inf, inf, 200_000), Ok((150_000, 200_000)));
        assert_eq!(rttime_limits(inf, inf, 1_000_000), Ok((150_000, 200_000)));
        assert_eq!(rttime_limits(inf, inf, 50_000), Ok((25_000, 50_000)));
        assert_eq!(
            rttime_limits(10_000, 100_000, 200_000),
            Ok((10_000, 100_000))
        );
        assert!(rttime_limits(inf, inf, 0).is_err());
        assert_eq!(unescape("/tmp/dbus%2dabc"), "/tmp/dbus-abc");
    }

    /// A real-time thread that runs past the soft RLIMIT_RTTIME set for RTKit is made ordinary by
    /// the SIGXCPU handler, and the process lives on (no SIGKILL at the hard limit). Ignored: it
    /// sets this process's RLIMIT_RTTIME and needs real-time priority (an rtprio allowance) to
    /// spin a thread at `SCHED_FIFO` for 400 ms; run it alone (`-- --ignored --exact
    /// rtkit::tests::a_runaway_thread_is_demoted_by_sigxcpu_not_killed`).
    #[test]
    #[ignore = "sets RLIMIT_RTTIME for the process and spins a real-time thread"]
    fn a_runaway_thread_is_demoted_by_sigxcpu_not_killed() {
        let (soft, hard) = limit_rttime(200_000).expect("RLIMIT_RTTIME set");
        assert_eq!((soft, hard), (150_000, 200_000));
        handle_xcpu();
        let before = XCPU.load(Ordering::Acquire);
        let r = std::thread::spawn(|| -> Result<Option<Duration>, String> {
            let period = Duration::from_millis(5);
            super::super::promote(period, period / 2)?;
            let start = std::time::Instant::now();
            let mut demoted_at = None;
            // (Never blocking: the CPU time counts toward RLIMIT_RTTIME.)
            while start.elapsed() < Duration::from_millis(400) {
                if demoted_at.is_none() && !super::super::is_realtime() {
                    demoted_at = Some(start.elapsed());
                }
                std::hint::spin_loop();
            }
            super::super::demote();
            Ok(demoted_at)
        })
        .join()
        .expect("the thread");
        let demoted_at = match r {
            Err(e) => {
                eprintln!("no real-time priority here ({e}): not checked");
                return;
            }
            Ok(d) => d.expect("demoted while it ran"),
        };
        assert!(XCPU.load(Ordering::Acquire) > before, "SIGXCPU handled");
        eprintln!("demoted after {demoted_at:?} of spinning");
        assert!(
            demoted_at >= Duration::from_millis(140) && demoted_at < Duration::from_millis(200),
            "{demoted_at:?}"
        );
    }

    /// A real bus (a private `dbus-daemon`, where one is installed): authenticated, said hello,
    /// a call answered, and RTKit's absence named as such.
    #[test]
    fn a_private_bus_answers_and_names_rtkit_absent() {
        let dir = std::env::temp_dir().join(format!("ca72-rt-bus-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let socket = dir.join("bus");
        let config = dir.join("bus.conf");
        std::fs::write(
            &config,
            format!(
                "<!DOCTYPE busconfig PUBLIC \"-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN\" \"http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd\"><busconfig><type>session</type><listen>unix:path={}</listen><auth>EXTERNAL</auth><policy context=\"default\"><allow send_destination=\"*\" eavesdrop=\"true\"/><allow eavesdrop=\"true\"/><allow own=\"*\"/></policy></busconfig>",
                socket.display()
            ),
        )
        .unwrap();
        let Ok(mut daemon) = std::process::Command::new("dbus-daemon")
            .arg(format!("--config-file={}", config.display()))
            .arg("--nofork")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        else {
            eprintln!("no dbus-daemon here: not checked");
            return;
        };
        let address = format!("unix:path={}", socket.display());
        let mut bus = None;
        for _ in 0..100 {
            if let Ok(b) = Bus::connect(&address) {
                bus = Some(b);
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let mut bus = bus.expect("the private bus answers");
        let r = bus
            .call(
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
                "GetNameOwner",
                &[Arg::Str("org.freedesktop.DBus")],
            )
            .expect("an answer");
        assert!(matches!(r, Reply::Return { .. }), "{r:?}");
        let r = bus
            .call(
                RTKIT,
                RTKIT_PATH,
                "org.freedesktop.DBus.Properties",
                "Get",
                &[Arg::Str(RTKIT), Arg::Str("MaxRealtimePriority")],
            )
            .expect("an answer");
        let Reply::Error { name, message } = r else {
            panic!("RTKit answered on a private bus");
        };
        assert_eq!(
            refusal(&name, &message).0,
            Why::NoRtkit,
            "{name}: {message}"
        );
        let _ = daemon.kill();
        let _ = daemon.wait();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
