//! The plugin helper process (DESIGN 10.1): hosts plugins on behalf of the DAW, which
//! talks to it through [`daw_ipc`]. Control requests arrive as lines on stdin and replies
//! leave on stdout; audio moves through shared memory, processed by one thread per plugin
//! instance. The DAW process itself never loads plugin code.
//!
//! `daw-helper --scan <file>` prints the plugins in a file (CLAP or VST3) as JSON and exits.

pub mod clap;
pub mod plugin;
pub mod vst3;
pub mod watch;
pub mod window;

use crate::plugin::{Instance, Library, Processor};
use daw_ipc::layout::{QUIT, SEQ_MASK};
use daw_ipc::protocol::{self, Activated, Loaded, Op, Reply, Request};
use daw_ipc::{Event, MAX_BLOCK, MAX_EVENTS, SLOTS, Transport};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::thread::JoinHandle;
use std::time::Duration;

/// Entry point of the `daw-helper` binary; returns the exit code.
pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--scan") {
        let Some(path) = args.get(2) else {
            eprintln!("usage: daw-helper --scan <plugin file>");
            return 2;
        };
        return match Library::load(Path::new(path)) {
            Ok(lib) => {
                println!(
                    "{}",
                    serde_json::to_string(&lib.descriptors()).unwrap_or_default()
                );
                0
            }
            Err(e) => {
                eprintln!("{e}");
                1
            }
        };
    }
    serve()
}

/// The protocol's output: the original stdout, with the process's stdout redirected to
/// stderr so that plugins printing to stdout cannot interleave with replies.
fn protocol_output() -> Box<dyn Write> {
    #[cfg(unix)]
    {
        use std::os::fd::FromRawFd;
        #[allow(unsafe_code)]
        // SAFETY: duplicating and redirecting the standard descriptors of this process
        // before any other thread writes to them.
        unsafe {
            let fd = libc::dup(1);
            if fd >= 0 && libc::dup2(2, 1) >= 0 {
                return Box::new(std::fs::File::from_raw_fd(fd));
            }
        }
    }
    Box::new(std::io::stdout())
}

fn serve() -> i32 {
    let mut out = protocol_output();
    let (tx, rx) = channel::<String>();
    std::thread::Builder::new()
        .name("daw-helper-stdin".into())
        .spawn(move || {
            for line in std::io::stdin().lock().lines() {
                match line {
                    Ok(l) => {
                        if tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        })
        .ok();
    let mut helper = Helper::default();
    loop {
        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(line) => {
                let Some(req) = protocol::decode::<Request>(&line) else {
                    continue;
                };
                let (reply, quit) = match req {
                    Ok(req) => {
                        let quit = req.op == Op::Quit;
                        (helper.handle(req), quit)
                    }
                    Err(e) => (err(0, format!("bad request: {e}")), false),
                };
                if out.write_all(protocol::encode(&reply).as_bytes()).is_err()
                    || out.flush().is_err()
                {
                    break;
                }
                if quit {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            // The DAW went away: stop.
            Err(RecvTimeoutError::Disconnected) => break,
        }
        for n in helper.service() {
            if out.write_all(protocol::encode(&n).as_bytes()).is_err() || out.flush().is_err() {
                break;
            }
        }
    }
    helper.shutdown();
    0
}

/// Locks `m` on the main thread without blocking on it: a lock an audio thread takes with
/// `try_lock` is then never one it has to wake a waiter on (the review's finding: its unlock
/// would make a system call). The audio thread holds it only to move a few values.
pub(crate) fn lock_soon<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    loop {
        match m.try_lock() {
            Ok(g) => return g,
            Err(std::sync::TryLockError::Poisoned(p)) => return p.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => std::thread::yield_now(),
        }
    }
}

fn ok(id: u64, result: serde_json::Value) -> Reply {
    Reply {
        id,
        ok: true,
        error: None,
        result,
    }
}

fn err(id: u64, e: impl Into<String>) -> Reply {
    Reply {
        id,
        ok: false,
        error: Some(e.into()),
        result: serde_json::Value::Null,
    }
}

struct Running {
    transport: Arc<Transport>,
    thread: JoinHandle<()>,
}

struct Hosted {
    inst: Instance,
    running: Option<Running>,
    /// The names for its keys the DAW was last given (D216).
    names: Vec<protocol::NoteName>,
    /// When its values are to be read for the DAW (D228): once it has been quiet
    /// [`SETTLE`] since it last said they changed.
    values_due: ValuesDue,
    /// Host values actually applied by the plugin, observed on the service thread, for
    /// clients waiting for acknowledgement rather than guessing how long a turn takes.
    host_values: BTreeMap<u32, f64>,
    value_reads: u64,
    /// What it changes without a word, and whether its state goes with its values (D234);
    /// from its activation.
    watch: Option<watch::Watch>,
    /// Its parameters whose values choose a program (D216), and until when what it says or
    /// changes is the host's doing (D234: a program the host gave it, a state it loaded),
    /// as D228's window, kept here where the time between them is the helper's own.
    programs: Vec<u32>,
    host_until: Option<std::time::Instant>,
    /// The most it said of its state since its last reading (D234).
    said: Option<watch::Said>,
    /// States the host gave it (D234), and whether its state was found too big to bring.
    loads: u64,
    too_big: bool,
}

/// A state bigger than this is not brought into the project (D234): its values are; its
/// state is the one the project has (a warning on the helper's output).
const STATE_MAX: usize = 64 << 20;

/// How long after the host gives a plugin a program or a state what it says or changes is
/// the host's doing (D228's second, D234).
const HOSTS_OWN: Duration = Duration::from_secs(1);

/// How long a plugin that said its values changed is let be before they are read (D228): a
/// plugin marks its state dirty at every edit of its own, and a preset loads a value at a
/// time; read once it is quiet, its edits are taken first.
const SETTLE: Duration = Duration::from_millis(100);

/// Whether a plugin's values are to be read now (D228): `said` it changed them (each word
/// puts the reading [`SETTLE`] after it), and it has been quiet since.
#[derive(Default)]
struct ValuesDue {
    at: Option<std::time::Instant>,
    /// Latched when the change is observed, before waiting for quiet or scheduling.
    host: bool,
}

impl ValuesDue {
    fn settled(&mut self, said: bool, host: bool, now: std::time::Instant) -> bool {
        if said {
            self.at = Some(now + SETTLE);
            self.host = host;
        }
        if self.at.is_some_and(|d| now >= d) {
            self.at = None;
            return true;
        }
        false
    }

    fn take_host(&mut self) -> bool {
        std::mem::take(&mut self.host)
    }
}

#[derive(Default)]
struct Helper {
    libraries: BTreeMap<PathBuf, Library>,
    instances: BTreeMap<u32, Hosted>,
}

impl Helper {
    fn library(&mut self, path: &Path) -> Result<Library, String> {
        if let Some(l) = self.libraries.get(path) {
            return Ok(l.clone());
        }
        let l = Library::load(path)?;
        self.libraries.insert(path.to_path_buf(), l.clone());
        Ok(l)
    }

    fn handle(&mut self, req: Request) -> Reply {
        let id = req.id;
        let result = match req.op {
            Op::Hello { version } => {
                if version == protocol::VERSION {
                    Ok(
                        serde_json::json!({ "version": protocol::VERSION, "pid": std::process::id() }),
                    )
                } else {
                    Err(format!(
                        "protocol version {version} is not supported (helper speaks {})",
                        protocol::VERSION
                    ))
                }
            }
            Op::Scan { path } => self
                .library(Path::new(&path))
                .map(|l| serde_json::to_value(l.descriptors()).unwrap_or_default()),
            Op::Load {
                instance,
                path,
                plugin,
                state,
            } => self.load(instance, &path, &plugin, state.as_deref()),
            Op::Activate {
                instance,
                sample_rate,
                max_block,
                params,
                transport,
                start,
                realtime,
            } => self.activate(
                instance,
                sample_rate,
                max_block,
                &params,
                &transport,
                start,
                realtime,
            ),
            Op::SaveState { instance } => match self.instances.get(&instance) {
                Some(h) => h
                    .inst
                    .save_state()
                    .map(|b| serde_json::json!({ "state": protocol::hex(&b) })),
                None => Err(format!("no instance {instance}")),
            },
            Op::LoadState { instance, state } => match self.instances.get_mut(&instance) {
                Some(h) => protocol::unhex(&state).and_then(|b| h.load_state(&b)),
                None => Err(format!("no instance {instance}")),
            },
            Op::Unload { instance } => {
                self.unload(instance);
                Ok(serde_json::Value::Null)
            }
            Op::GuiOpen {
                instance,
                title,
                x,
                y,
            } => match self.instances.get_mut(&instance) {
                Some(h) => h.inst.gui_open(&title, x.zip(y)).map(|v| h.with_watch(v)),
                None => Err(format!("no instance {instance}")),
            },
            Op::GuiMove { instance, x, y } => match self.instances.get_mut(&instance) {
                Some(h) => h.inst.gui_move(x, y).map(|v| h.with_watch(v)),
                None => Err(format!("no instance {instance}")),
            },
            Op::GuiClose { instance } => match self.instances.get_mut(&instance) {
                Some(h) => {
                    h.inst.gui_close();
                    Ok(h.with_watch(h.inst.gui_info()))
                }
                None => Err(format!("no instance {instance}")),
            },
            Op::GuiInfo { instance } => match self.instances.get(&instance) {
                Some(h) => Ok(h.with_watch(h.inst.gui_info())),
                None => Err(format!("no instance {instance}")),
            },
            Op::Quit => {
                self.shutdown();
                Ok(serde_json::Value::Null)
            }
        };
        match result {
            Ok(v) => ok(id, v),
            Err(e) => err(id, e),
        }
    }

    fn load(
        &mut self,
        instance: u32,
        path: &str,
        plugin: &str,
        state: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        if self.instances.contains_key(&instance) {
            return Err(format!("instance {instance} exists"));
        }
        let lib = self.library(Path::new(path))?;
        let inst = lib.create(plugin)?;
        let mut state_restored = false;
        if let Some(s) = state {
            let bytes = protocol::unhex(s)?;
            inst.load_state(&bytes)?;
            state_restored = true;
        }
        let (inputs, outputs) = inst.main_ports();
        let loaded = Loaded {
            desc: inst.descriptor(),
            params: inst.params(),
            inputs,
            outputs,
            notes: inst.takes_notes(),
            state_restored,
            note_names: inst.note_names(),
        };
        self.instances.insert(
            instance,
            Hosted {
                inst,
                running: None,
                names: Vec::new(),
                values_due: ValuesDue::default(),
                host_values: BTreeMap::new(),
                value_reads: 0,
                watch: None,
                programs: Vec::new(),
                host_until: None,
                said: None,
                loads: 0,
                too_big: false,
            },
        );
        serde_json::to_value(loaded).map_err(|e| e.to_string())
    }

    // The arguments are the fields of the `Activate` request.
    #[allow(clippy::too_many_arguments)]
    fn activate(
        &mut self,
        instance: u32,
        sample_rate: f64,
        max_block: u32,
        params: &[(u32, f64)],
        name: &daw_ipc::TransportName,
        start: u32,
        realtime: bool,
    ) -> Result<serde_json::Value, String> {
        let h = self
            .instances
            .get_mut(&instance)
            .ok_or(format!("no instance {instance}"))?;
        if h.running.is_some() {
            return Err(format!("instance {instance} is already active"));
        }
        let max_block = (max_block as usize).clamp(1, MAX_BLOCK);
        h.inst.flush_params(params);
        h.inst.activate(sample_rate, max_block as u32)?;
        let transport = Arc::new(Transport::open(name).map_err(|e| e.to_string())?);
        let processor = h.inst.processor(max_block);
        let t = transport.clone();
        let thread = std::thread::Builder::new()
            .name(format!("daw-helper-audio-{instance}"))
            .spawn(move || {
                if realtime {
                    let period =
                        std::time::Duration::from_secs_f64(max_block as f64 / sample_rate.max(1.0));
                    if let Err(e) = daw_ipc::rt::promote(period, period / 2) {
                        eprintln!("instance {instance}: no real-time priority: {e}");
                    }
                }
                audio_loop(processor, &t, start)
            })
            .map_err(|e| e.to_string())?;
        h.running = Some(Running { transport, thread });
        // Asked again with the project's values set (D211): a VST3 plugin's program
        // parameter chooses whose pitch names these are. From here on a change is sent on
        // its own (D216, `service`).
        h.inst.take_names_changed();
        h.names = h.inst.note_names();
        // Edits made while it was set up (its own values given back) are not the editor's,
        // nor a word that its values changed (its state loaded, its values given) a preset
        // of its own (D228).
        h.inst.take_edits();
        h.inst.take_clears();
        h.inst.take_values_changed();
        h.values_due = ValuesDue::default();
        // What it has now is what it changes from (D234).
        h.inst.take_host_given();
        h.host_values.clear();
        h.value_reads = 0;
        let now = std::time::Instant::now();
        let (state, cost) = h.timed_state();
        h.inst.take_state_said();
        h.said = None;
        h.programs = h
            .inst
            .params()
            .iter()
            .filter(|p| p.program)
            .map(|p| p.id)
            .collect();
        h.host_until = None;
        h.watch = Some(watch::Watch::new(
            now,
            &h.inst.param_values(),
            state.as_deref().map(|s| (s, cost)),
        ));
        let a = Activated {
            latency: h.inst.latency(),
            tail: h.inst.tail(),
            note_names: Some(h.names.clone()),
        };
        serde_json::to_value(a).map_err(|e| e.to_string())
    }

    fn unload(&mut self, instance: u32) {
        if let Some(mut h) = self.instances.remove(&instance) {
            if let Some(r) = h.running.take() {
                r.transport.quit();
                let _ = r.thread.join();
            }
            drop(h);
        }
    }

    fn shutdown(&mut self) {
        let ids: Vec<u32> = self.instances.keys().copied().collect();
        for i in ids {
            self.unload(i);
        }
    }

    /// Main-thread duties between requests; returns notices for the DAW (a GUI the user
    /// closed).
    fn service(&mut self) -> Vec<Reply> {
        window::pump();
        let mut notices = Vec::new();
        for (i, h) in &mut self.instances {
            if h.inst.gui_service() {
                notices.push(ok(
                    0,
                    serde_json::json!({"notice": "gui_closed", "instance": i}),
                ));
            }
        }
        for (i, h) in &mut self.instances {
            h.inst.service();
            // Its names for its keys, asked again when they may have changed, and sent
            // when they did (D216).
            if h.running.is_some() && h.inst.take_names_changed() {
                let names = h.inst.note_names();
                if names != h.names {
                    h.names = names;
                    notices.push(ok(
                        0,
                        serde_json::json!({
                            "notice": protocol::NOTICE_NOTE_NAMES,
                            "instance": i,
                            "names": h.names,
                        }),
                    ));
                }
            }
            // Its own edits (D222: its editor's) and what it asks to clear, for the DAW.
            if h.running.is_some() {
                let edits = h.inst.take_edits();
                let given = h.inst.take_host_given();
                let observed = std::time::Instant::now();
                if given.iter().any(|id| h.programs.contains(id)) {
                    h.host_until = Some(observed + HOSTS_OWN);
                }
                for id in &given {
                    if let Some(value) = h.inst.host_value(*id) {
                        h.host_values.insert(*id, value);
                    }
                }
                if let Some(w) = &mut h.watch {
                    w.note_edits(&edits);
                    w.note_host(&given, &|id| h.inst.host_value(id));
                }
                if !edits.is_empty() {
                    notices.push(ok(
                        0,
                        serde_json::json!({
                            "notice": protocol::NOTICE_PARAM_EDITS,
                            "instance": i,
                            "edits": edits,
                        }),
                    ));
                }
                // Its values, once it said they changed by its own doing (a preset loaded),
                // or a poll found them or its state changed without a word (D234), and it
                // has been quiet a moment: every one, for the DAW to compare (D228), with
                // its state when that goes with them.
                let now = std::time::Instant::now();
                let polled = h.poll(now);
                if let Some(said) = h.inst.take_state_said() {
                    h.said = h.said.max(Some(said));
                }
                if h.values_due.settled(
                    h.inst.take_values_changed() || polled,
                    h.host_until.is_some_and(|t| observed < t),
                    now,
                ) {
                    notices.push(ok(0, h.reading(*i, now)));
                }
                for (param, flags) in h.inst.take_clears() {
                    notices.push(ok(
                        0,
                        serde_json::json!({
                            "notice": protocol::NOTICE_PARAM_CLEAR,
                            "instance": i,
                            "param": param,
                            "flags": flags,
                        }),
                    ));
                }
            }
            if h.inst.restart_requested() {
                eprintln!(
                    "instance {i}: the plugin asked for a restart; the new configuration (for example a latency change) applies when the device is reloaded"
                );
            }
        }
        notices
    }
}

/// Whether a running plugin's values are polled (D234): while its editor is open, where a
/// person loads presets. `DAW_PLUGIN_POLL=always` polls without one (the tests, which open
/// no windows on the desktop), `off` never.
fn polling(editor_open: bool) -> bool {
    static WHEN: std::sync::OnceLock<Option<bool>> = std::sync::OnceLock::new();
    let when = WHEN.get_or_init(|| match std::env::var("DAW_PLUGIN_POLL").as_deref() {
        Ok("always") => Some(true),
        Ok("off") => Some(false),
        _ => None,
    });
    when.unwrap_or(editor_open)
}

impl Hosted {
    /// Its state saved now, and how long that took (D234).
    fn timed_state(&self) -> (Option<Vec<u8>>, Duration) {
        let t = std::time::Instant::now();
        let s = self.inst.save_state().ok();
        (s, t.elapsed())
    }

    /// Polls its values, and its state when that is cheap, if due (D234): whether it
    /// changed one itself without a word.
    fn poll(&mut self, now: std::time::Instant) -> bool {
        let Hosted { inst, watch, .. } = self;
        let Some(w) = watch else { return false };
        if !polling(inst.editor_open()) || !w.due(now) {
            return false;
        }
        let t = std::time::Instant::now();
        let values = inst.param_values();
        let cost = t.elapsed();
        let host = |id| inst.host_value(id);
        let mut said = w.poll(now, &values, cost, &host);
        if w.state_due(now) {
            let t = std::time::Instant::now();
            if let Ok(s) = inst.save_state() {
                said |= w.poll_state(now, &values, &s, t.elapsed(), &host);
            }
        }
        said
    }

    /// Its values now, for the DAW to compare (D228), with its state when values moved by
    /// its own doing or its state changed alone (D234).
    fn reading(&mut self, instance: u32, now: std::time::Instant) -> serde_json::Value {
        let host = self.values_due.take_host();
        self.value_reads += 1;
        let values = self.inst.param_values();
        let said = self.said.take().unwrap_or(watch::Said::Values);
        // A state is saved when it is cheap, or it may go with these values (the review's
        // finding: not a costly save at every edit's `mark_dirty`), and kept when not too big.
        let save = self.watch.as_ref().is_none_or(|w| {
            w.cheap() || w.may_bring_state(&values, said, &|id| self.inst.host_value(id))
        });
        let (state, cost) = if save {
            self.timed_state()
        } else {
            (None, Duration::ZERO)
        };
        let state = state.filter(|s| {
            let fits = s.len() <= STATE_MAX;
            if !fits && !std::mem::replace(&mut self.too_big, true) {
                eprintln!(
                    "instance {instance}: its state is {} MB, more than the {} MB a project takes; its values are read, not its state",
                    s.len() >> 20,
                    STATE_MAX >> 20
                );
            }
            fits
        });
        let mut n = serde_json::json!({
            "notice": protocol::NOTICE_PARAM_VALUES,
            "instance": instance,
            "values": values,
            "host": host,
        });
        let Hosted { inst, watch, .. } = self;
        if let Some(w) = watch {
            let host = |id| inst.host_value(id);
            // A reading without independent value changes must not forward an older
            // full automation snapshot for the DAW to mistake for an editor preset.
            n["values"] = serde_json::json!(w.preset_values(&values, &host));
            let r = w.read(
                now,
                &values,
                state.as_deref().map(|s| (s, cost)),
                said,
                &host,
            );
            if r.with_state()
                && let Some(s) = &state
            {
                n["state"] = serde_json::json!(protocol::hex(s));
                n["alone"] = serde_json::json!(r.alone);
            }
        }
        n
    }

    /// The host gives it a state (D234): loaded, running or not. The host's doing: what it
    /// says of it now is no preset of its own, nor are edits answering it; what it has now
    /// is what it changes from.
    fn load_state(&mut self, bytes: &[u8]) -> Result<serde_json::Value, String> {
        self.inst.load_state(bytes)?;
        self.host_until = Some(std::time::Instant::now() + HOSTS_OWN);
        self.loads += 1;
        self.inst.take_values_changed();
        self.inst.take_state_said();
        self.said = None;
        self.inst.take_edits();
        self.values_due = ValuesDue::default();
        if self.watch.is_some() {
            let now = std::time::Instant::now();
            let values = self.inst.param_values();
            let (state, cost) = self.timed_state();
            if let Some(w) = &mut self.watch {
                w.reset(now, &values, state.as_deref().map(|s| (s, cost)));
            }
        }
        Ok(serde_json::json!({}))
    }

    /// A window reply with how its values and state are watched (D234): `{polling, polls,
    /// every_ms, values_us, state_us, state_bytes, state_polled, loads, host_ms}` (`loads`:
    /// states the host gave it; `host_ms`: how long what it does is the host's doing still).
    fn with_watch(&self, mut v: serde_json::Value) -> serde_json::Value {
        if let (Some(w), Some(o)) = (&self.watch, v.as_object_mut()) {
            let c = w.costs();
            o.insert(
                "watch".into(),
                serde_json::json!({
                    "polling": self.running.is_some() && polling(self.inst.editor_open()),
                    "polls": c.polls,
                    "every_ms": c.every.as_millis() as u64,
                    "values_us": c.values.as_micros() as u64,
                    "state_us": c.state.map(|d| d.as_micros() as u64),
                    "state_bytes": c.state_bytes,
                    "state_polled": c.state_polled,
                    "loads": self.loads,
                    "host_values": self.host_values,
                    "value_reads": self.value_reads,
                    "values_pending": self.values_due.at.is_some(),
                    "host_ms": self.host_until.map_or(0, |t| {
                        t.saturating_duration_since(std::time::Instant::now()).as_millis() as u64
                    }),
                }),
            );
        }
        v
    }
}

/// An instance's processing thread: waits for blocks and processes them in place.
/// How long the audio thread waits for a block before it sees to what the plugin asked for
/// meanwhile (a CLAP flush, D222).
const IDLE: Duration = Duration::from_millis(50);

fn audio_loop(mut p: Processor, t: &Transport, start: u32) {
    let started = p.start();
    let mut last = start;
    let mut events = [Event::default(); MAX_EVENTS];
    loop {
        // Waits a while at most: between blocks the plugin may want a flush (D222).
        let deadline = std::time::Instant::now() + IDLE;
        let Ok(req) = t.wait_request(last, Some(deadline)) else {
            if started {
                p.idle();
            }
            continue;
        };
        if req & QUIT != 0 {
            break;
        }
        last = req;
        let seq = req & SEQ_MASK;
        #[allow(unsafe_code)]
        // SAFETY: the host wrote this slot and then made the request; it does not touch the
        // slot again until `finish`. Values read from it are clamped before use.
        let status = unsafe {
            let s = t.slot(seq as usize % SLOTS);
            let frames = ((&raw const (*s).frames).read_volatile() as usize).min(MAX_BLOCK);
            let n = ((&raw const (*s).n_events).read_volatile() as usize).min(MAX_EVENTS);
            let steady = (&raw const (*s).steady_time).read_volatile();
            let timing = (&raw const (*s).timing).read_volatile();
            std::ptr::copy_nonoverlapping(
                (&raw const (*s).events) as *const Event,
                events.as_mut_ptr(),
                n,
            );
            let input = [
                (&raw mut (*s).input[0]) as *mut f32,
                (&raw mut (*s).input[1]) as *mut f32,
            ];
            let output = [
                (&raw mut (*s).output[0]) as *mut f32,
                (&raw mut (*s).output[1]) as *mut f32,
            ];
            let status = if started {
                p.process(steady, timing, frames, &events[..n], input, output)
            } else {
                daw_ipc::layout::STATUS_ERROR
            };
            (&raw mut (*s).status).write_volatile(status);
            status
        };
        let _ = status;
        t.finish(seq);
    }
    if started {
        p.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D228: the values are read once the plugin has been quiet `SETTLE` since its last
    /// word, once.
    #[test]
    fn a_plugins_values_are_read_once_it_is_quiet() {
        let t0 = std::time::Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        let mut due = ValuesDue::default();
        assert!(!due.settled(false, false, ms(0)), "nothing said");
        assert!(!due.settled(true, false, ms(0)));
        assert!(!due.settled(false, false, ms(60)));
        // Said again: from then.
        assert!(!due.settled(true, false, ms(90)));
        assert!(!due.settled(false, false, ms(150)));
        assert!(due.settled(false, false, ms(190)));
        assert!(!due.take_host());
        assert!(!due.settled(false, false, ms(400)), "once");
    }

    #[test]
    fn a_delayed_read_retains_the_host_origin_of_the_observed_change() {
        let now = std::time::Instant::now();
        let mut due = ValuesDue::default();
        assert!(!due.settled(true, true, now));
        // The helper does not get another turn until the one-second window has expired.
        assert!(due.settled(false, false, now + Duration::from_secs(2)));
        assert!(
            due.take_host(),
            "the host's program cannot become an editor preset"
        );
        assert!(!due.settled(true, false, now + Duration::from_secs(3)));
        assert!(due.settled(false, false, now + Duration::from_secs(4)));
        assert!(!due.take_host(), "a later editor preset has its own origin");
    }

    #[test]
    fn a_new_editor_change_replaces_a_pending_host_changes_origin() {
        let now = std::time::Instant::now();
        let mut due = ValuesDue::default();
        assert!(!due.settled(true, true, now));
        // No read occurred before a new editor preset after the host's window expired.
        assert!(!due.settled(true, false, now + Duration::from_secs(2)));
        assert!(due.settled(false, false, now + Duration::from_secs(3)));
        assert!(
            !due.take_host(),
            "the latest editor change must reach the project"
        );
    }
}
