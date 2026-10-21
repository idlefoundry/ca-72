//! The supervisor (docs/fx-host.md): starts one helper process for each effect, loads the
//! effect into it, notices at once when the helper dies, starts another and restores the
//! effect from its last state snapshot and the values the host gave it. A monitor thread
//! takes the snapshots (after changes, debounced, and periodically) and kills a helper whose
//! blocks stop coming back (the watchdog).
//!
//! DAW-v3's `daw-sandbox/src/supervisor.rs` (commit cb15558, its D19 and D25), reduced to
//! effects: no notes, names for keys or modulation; one helper for each instance; and two
//! additions, an instance unloaded (an effect taken out of the chain) and an instance
//! loaded again at another sample rate in the same helper.
//!
//! Since decisions.md R-FX-SAFETY, a load says what went wrong ([`LoadError`]: refused,
//! crashed or hung, and while doing what), its saved state given in a step of its own so that
//! a state that crashes the plug-in is told from a plug-in that crashes; a state that crashes
//! it while it is restored after a crash is not tried again (given up on at once), nor is an
//! effect restored again and again that crashes each time within a second of it.

use crate::helper::{Failure, HelperProc};
use daw_ipc::Transport;
use daw_ipc::protocol::{self, Activated, Loaded, Op};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::{Duration, Instant, SystemTime};

pub(crate) fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Instance status: running.
pub const RUNNING: u8 = 0;
/// The helper died; another is being started.
pub const DOWN: u8 = 1;
/// Recovery gave up (too many crashes, or the plug-in no longer loads): dry from now on.
pub const FAILED: u8 = 2;
/// Taken out of the chain, or replaced by an instance at another sample rate.
pub const RETIRED: u8 = 3;

/// No value set yet, in [`Instance::values`].
const UNSET: u64 = u64::MAX;

/// A crash this soon after the effect was (re)started counts as quick: an effect given up on
/// after nothing but quick crashes is thought crashed by its settings (R-FX-SAFETY).
const QUICK: Duration = Duration::from_secs(1);

/// What went wrong when an effect was loaded (or listed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fault {
    /// The plug-in, or the helper, said no (not found, not an effect, an error of its own):
    /// nothing crashed, and trying again next time costs nothing.
    Refused,
    /// Its helper died (the plug-in crashed it).
    Crashed,
    /// It did not answer within the load timeout; its helper was killed.
    Hung,
}

/// What was being done when it went wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum During {
    /// Its helper started, the plug-in made.
    Loading,
    /// Its saved state given to it.
    Settings,
    /// It was activated and its state read back.
    Starting,
}

/// An effect that would not load: why, when, and in words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    pub fault: Fault,
    pub during: During,
    pub message: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl LoadError {
    fn refused(message: String) -> LoadError {
        LoadError {
            fault: Fault::Refused,
            during: During::Loading,
            message,
        }
    }

    fn from_failure(f: Failure, during: During, what: &str) -> LoadError {
        let fault = match f {
            Failure::Refused(_) => Fault::Refused,
            Failure::Exited | Failure::NotRunning => Fault::Crashed,
            Failure::TimedOut(_) => Fault::Hung,
        };
        let message = match &f {
            Failure::Refused(e) => e.clone(),
            Failure::Exited | Failure::NotRunning => format!("it crashed while {what}"),
            Failure::TimedOut(t) => format!("it stopped responding while {what} ({t:?})"),
        };
        LoadError {
            fault,
            during,
            message,
        }
    }
}

/// Why an effect was given up on: it kept crashing, hung, or crashed (it is thought) because
/// of its saved state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GaveUp {
    pub hung: bool,
    pub settings: bool,
}

/// How the supervisor treats its helpers.
#[derive(Debug, Clone)]
pub struct SandboxOptions {
    /// The `ca72-fx-helper` executable.
    pub helper: PathBuf,
    /// Driven by the audio device's clock (the monitor thread is then the watchdog), or
    /// offline (the tests: each block is waited for, and a lost one done again).
    pub realtime: bool,
    /// A helper whose block has not come back for this long is killed and treated as
    /// crashed.
    pub hang_timeout: Duration,
    /// Snapshots are taken this long after the last change...
    pub snapshot_debounce: Duration,
    /// ...and at least this often.
    pub snapshot_interval: Duration,
    /// Helper restarts before an effect is given up on (it then passes its input).
    pub max_restarts: u32,
    /// How long loading or activating a plug-in may take.
    pub load_timeout: Duration,
}

impl SandboxOptions {
    pub fn new(helper: PathBuf) -> SandboxOptions {
        SandboxOptions {
            helper,
            realtime: true,
            hang_timeout: Duration::from_secs(2),
            snapshot_debounce: Duration::from_millis(500),
            snapshot_interval: Duration::from_secs(60),
            max_restarts: 5,
            load_timeout: Duration::from_secs(30),
        }
    }
}

/// Something that happened to an effect: `crashed`, `hung`, `restored`, `failed`,
/// `warning`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The effect, as its slot names it.
    pub effect: String,
    pub kind: String,
    pub detail: String,
    pub at: SystemTime,
}

/// Everything about one effect instance that outlives its helper's restarts, shared by its
/// node on the audio thread (the atomics) and the supervisor.
#[derive(Debug)]
pub struct Instance {
    /// The effect's name, for messages.
    pub label: String,
    /// Its number within its helper.
    pub(crate) number: u32,
    pub path: PathBuf,
    pub plugin: String,
    pub sample_rate: f64,
    pub max_block: u32,
    pub transport: Transport,
    pub info: Loaded,
    /// Latency and tail, in frames, from the latest activation.
    pub latency: AtomicU32,
    pub tail: AtomicU32,
    /// [`RUNNING`], [`DOWN`], [`FAILED`] or [`RETIRED`].
    pub status: AtomicU8,
    /// Moves after every restore.
    pub generation: AtomicU32,
    /// The plain value of each parameter the host set (by place in `info.params`), as `f64`
    /// bits: what a restore gives it again.
    pub values: Box<[AtomicU64]>,
    /// When something changed that its state may hold, in ms since the sandbox started (0:
    /// nothing since the last snapshot).
    pub(crate) dirty_since: AtomicU64,
    /// The frame of the block submitted last.
    pub position: AtomicI64,
    /// When the outstanding block was submitted, in µs since the sandbox started (0: none):
    /// the watchdog's.
    pub submitted_at: AtomicU64,
    /// Blocks it did not return in time (each passed its input instead).
    pub misses: AtomicU64,
    pub crashes: AtomicU32,
    /// Set by an offline node whose watchdog killed the helper.
    pub hung: AtomicBool,
    /// The first frame processed after the latest restore (-1: none yet).
    pub resumed_at: AtomicI64,
    pub(crate) snapshot: Mutex<Option<Vec<u8>>>,
    pub(crate) snapshot_at: Mutex<Instant>,
    /// Moves whenever the snapshot's bytes change.
    pub snapshot_rev: AtomicU64,
    /// When it was last (re)started, in ms since the sandbox started; how many times in a row
    /// it crashed soon after.
    pub(crate) started_ms: AtomicU64,
    pub(crate) quick: AtomicU32,
    /// Why it was given up on (set before its status is [`FAILED`]).
    pub(crate) gave_up: Mutex<Option<GaveUp>>,
    /// Signalled when the status or generation changes.
    pub(crate) changed: (Mutex<()>, Condvar),
    pub(crate) group: Weak<Group>,
    pub(crate) epoch: Instant,
    pub(crate) state_rev: Arc<AtomicU64>,
    /// The values its editor gave its parameters (ID, value in its units), the latest of
    /// each, until the chain takes them: a pinned parameter's base (decisions.md R-FX-PARAMS).
    pub(crate) edits: Mutex<Vec<(u32, f64)>>,
    /// Its parameters' names or list changed: the chain lists them again (R-FX-PARAMS).
    pub(crate) relist: AtomicBool,
}

/// How long a parameter request (a listing, value texts) may take.
const ASK: Duration = Duration::from_secs(5);

impl Instance {
    pub fn param_index(&self, id: u32) -> Option<usize> {
        self.info.params.iter().position(|p| p.id == id)
    }

    /// Records a parameter's new plain value (by its node, once delivered).
    pub fn set_value(&self, i: usize, plain: f64) {
        if let Some(v) = self.values.get(i) {
            v.store(plain.to_bits(), Ordering::Relaxed);
            self.mark_dirty();
        }
    }

    /// Records the value a parameter has (its editor's), for a restore to give it again,
    /// without marking its state changed: the edit that gave it did (R-FX-PARAMS).
    pub fn record_value(&self, id: u32, plain: f64) {
        if let Some(v) = self.param_index(id).and_then(|i| self.values.get(i)) {
            v.store(plain.to_bits(), Ordering::Relaxed);
        }
    }

    /// Its parameters as it lists them now, with their values (`Op::Params`, R-FX-PARAMS).
    pub fn list_params(&self) -> Result<Vec<protocol::ParamInfo>, String> {
        if self.status() != RUNNING {
            return Err("the effect is not running".into());
        }
        let p = self.helper().ok_or("the effect's helper is not running")?;
        let v = p.request(
            Op::Params {
                instance: self.number,
            },
            ASK,
        )?;
        serde_json::from_value(v).map_err(|e| e.to_string())
    }

    /// The plug-in's own text for each (parameter, value in its units) pair (`Op::ParamText`,
    /// R-FX-PARAMS): `None` where it gave none.
    pub fn param_text(&self, values: &[(u32, f64)]) -> Result<Vec<Option<String>>, String> {
        if self.status() != RUNNING {
            return Err("the effect is not running".into());
        }
        let p = self.helper().ok_or("the effect's helper is not running")?;
        let v = p.request(
            Op::ParamText {
                instance: self.number,
                values: values.to_vec(),
            },
            ASK,
        )?;
        serde_json::from_value(v).map_err(|e| e.to_string())
    }

    /// The values its editor gave its parameters since the last call, the latest of each.
    pub fn take_edits(&self) -> Vec<(u32, f64)> {
        std::mem::take(&mut *lock(&self.edits))
    }

    /// Whether its parameters' names or list changed since the last call.
    pub fn take_relist(&self) -> bool {
        self.relist.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn mark_dirty(&self) {
        let now = (self.epoch.elapsed().as_millis() as u64).max(1);
        self.dirty_since.store(now, Ordering::Relaxed);
    }

    /// The parameter values the host set so far, for a restore.
    pub fn current_values(&self) -> Vec<(u32, f64)> {
        self.info
            .params
            .iter()
            .zip(self.values.iter())
            .filter_map(|(p, v)| {
                let bits = v.load(Ordering::Relaxed);
                (bits != UNSET).then(|| (p.id, f64::from_bits(bits)))
            })
            .collect()
    }

    pub fn status(&self) -> u8 {
        self.status.load(Ordering::Acquire)
    }

    /// Why it was given up on, once it has been.
    pub fn gave_up(&self) -> Option<GaveUp> {
        *lock(&self.gave_up)
    }

    fn now_ms(&self) -> u64 {
        (self.epoch.elapsed().as_millis() as u64).max(1)
    }

    /// Given up on for `why` (its event already recorded): dry from now on.
    fn give_up(&self, why: GaveUp) {
        *lock(&self.gave_up) = Some(why);
        self.status.store(FAILED, Ordering::Release);
        self.transport.mark_dead();
        self.notify();
    }

    /// Microseconds since the sandbox started (for `submitted_at`).
    pub fn now_us(&self) -> u64 {
        (self.epoch.elapsed().as_micros() as u64).max(1)
    }

    /// Blocks until the instance runs with a generation other than `seen`, or has failed;
    /// the new generation, or `None`. Offline only.
    pub fn wait_recovered(&self, seen: u32) -> Option<u32> {
        let (m, cv) = &self.changed;
        let mut g = lock(m);
        loop {
            match self.status() {
                FAILED | RETIRED => return None,
                RUNNING => {
                    let now = self.generation.load(Ordering::Acquire);
                    if now != seen {
                        return Some(now);
                    }
                }
                _ => {}
            }
            g = cv
                .wait_timeout(g, Duration::from_millis(50))
                .map(|(g, _)| g)
                .unwrap_or_else(|e| e.into_inner().0);
        }
    }

    /// Blocks until the instance no longer runs with generation `seen`, or `deadline`.
    pub fn wait_down(&self, seen: u32, deadline: Instant) {
        let (m, cv) = &self.changed;
        let mut g = lock(m);
        while self.status() == RUNNING && self.generation.load(Ordering::Acquire) == seen {
            let now = Instant::now();
            if now >= deadline {
                return;
            }
            g = cv
                .wait_timeout(g, (deadline - now).min(Duration::from_millis(50)))
                .map(|(g, _)| g)
                .unwrap_or_else(|e| e.into_inner().0);
        }
    }

    fn notify(&self) {
        let (m, cv) = &self.changed;
        let _g = lock(m);
        cv.notify_all();
    }

    /// Kills its helper (the watchdog, or a test).
    pub fn kill_helper(&self) {
        if let Some(p) = self.helper() {
            p.kill();
        }
    }

    /// The latest state snapshot.
    pub fn snapshot(&self) -> Option<Vec<u8>> {
        lock(&self.snapshot).clone()
    }

    /// The helper hosting it now, if any.
    pub fn helper(&self) -> Option<Arc<HelperProc>> {
        let g = self.group.upgrade()?;
        lock(&g.proc).clone()
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    /// The process ID of its helper now, if any.
    pub fn helper_pid(&self) -> Option<u32> {
        self.helper().map(|p| p.pid())
    }

    /// Saves its state now into its snapshot, if it runs.
    pub fn take_snapshot(&self) -> Result<(), String> {
        if self.status() != RUNNING {
            return Err("the effect is not running".into());
        }
        let p = self.helper().ok_or("the effect's helper is not running")?;
        let s = save_state(self, &p)?;
        self.keep_snapshot(s);
        Ok(())
    }

    fn keep_snapshot(&self, s: Vec<u8>) {
        let mut snap = lock(&self.snapshot);
        if snap.as_deref() != Some(&s[..]) {
            *snap = Some(s);
            self.snapshot_rev.fetch_add(1, Ordering::AcqRel);
            self.state_rev.fetch_add(1, Ordering::AcqRel);
        }
        *lock(&self.snapshot_at) = Instant::now();
    }
}

/// A helper process and its one instance (more after a sample-rate change, briefly): it
/// survives the process's restarts.
#[derive(Debug)]
pub(crate) struct Group {
    proc: Mutex<Option<Arc<HelperProc>>>,
    instances: Mutex<Vec<Arc<Instance>>>,
    restarts: AtomicU32,
    next_instance: AtomicU32,
    /// Its instances are all unloaded: its helper is to quit, not to be started again.
    retired: AtomicBool,
    /// Held while its helper's death is dealt with: the helper started in its place may die
    /// too, and its own death waits for this one's to be done.
    recovering: Mutex<()>,
}

#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) opts: SandboxOptions,
    groups: Mutex<Vec<Arc<Group>>>,
    events: Mutex<Vec<Event>>,
    closing: AtomicBool,
    epoch: Instant,
    /// Moves whenever any instance's snapshot changes: the chain's saved file is behind.
    pub(crate) state_rev: Arc<AtomicU64>,
}

/// Every effect instance and its helpers.
#[derive(Debug)]
pub struct Sandbox {
    inner: Arc<Inner>,
    monitor: Option<std::thread::JoinHandle<()>>,
}

impl Sandbox {
    pub fn new(opts: SandboxOptions) -> Sandbox {
        let inner = Arc::new(Inner {
            opts,
            groups: Mutex::new(Vec::new()),
            events: Mutex::new(Vec::new()),
            closing: AtomicBool::new(false),
            epoch: Instant::now(),
            state_rev: Arc::new(AtomicU64::new(0)),
        });
        let weak = Arc::downgrade(&inner);
        let monitor = std::thread::Builder::new()
            .name("ca72-fx-monitor".into())
            .spawn(move || monitor(&weak))
            .ok();
        Sandbox { inner, monitor }
    }

    pub fn options(&self) -> &SandboxOptions {
        &self.inner.opts
    }

    /// Events from the `from`th on (crashes, restores, failures), and how many there are.
    pub fn events_from(&self, from: usize) -> (Vec<Event>, usize) {
        let e = lock(&self.inner.events);
        (
            e.get(from..).map(<[Event]>::to_vec).unwrap_or_default(),
            e.len(),
        )
    }

    pub(crate) fn event(&self, effect: &str, kind: &str, detail: String) {
        self.inner.event(effect, kind, detail);
    }

    /// Moves whenever an effect's saved state changes.
    pub fn state_rev(&self) -> u64 {
        self.inner.state_rev.load(Ordering::Acquire)
    }

    /// Starts a helper and loads and activates an effect in it. `params` are (parameter ID,
    /// plain value) pairs; `state` a state to restore first. A helper that crashed or hung is
    /// gone when this returns.
    #[allow(clippy::too_many_arguments)]
    pub fn load(
        &self,
        label: &str,
        path: &Path,
        plugin: &str,
        state: Option<&[u8]>,
        params: &[(u32, f64)],
        sample_rate: f64,
        max_block: u32,
    ) -> Result<Arc<Instance>, LoadError> {
        if self.inner.closing.load(Ordering::Acquire) {
            return Err(LoadError::refused("the application is closing".into()));
        }
        let group = Arc::new(Group {
            proc: Mutex::new(None),
            instances: Mutex::new(Vec::new()),
            restarts: AtomicU32::new(0),
            next_instance: AtomicU32::new(0),
            retired: AtomicBool::new(false),
            recovering: Mutex::new(()),
        });
        let proc = spawn(&self.inner, &group).map_err(LoadError::refused)?;
        *lock(&group.proc) = Some(proc.clone());
        lock(&self.inner.groups).push(group.clone());
        // (Closed meanwhile: `close` may have missed the group.)
        if self.inner.closing.load(Ordering::Acquire) {
            retire(&self.inner, &group);
            return Err(LoadError::refused("the application is closing".into()));
        }
        let loaded = self.load_in(
            &group,
            &proc,
            label,
            path,
            plugin,
            state,
            params,
            sample_rate,
            max_block,
        );
        if let Err(e) = &loaded {
            if e.fault != Fault::Refused {
                proc.kill();
            }
            retire(&self.inner, &group);
        }
        loaded
    }

    /// The same effect in the same helper at `sample_rate` and `max_block`, with the old
    /// one's latest state and values; the old one is unloaded.
    pub fn reload(
        &self,
        old: &Arc<Instance>,
        sample_rate: f64,
        max_block: u32,
    ) -> Result<Arc<Instance>, LoadError> {
        let _ = old.take_snapshot();
        let group = old
            .group
            .upgrade()
            .ok_or_else(|| LoadError::refused("the effect's helper is gone".into()))?;
        let proc = lock(&group.proc)
            .clone()
            .ok_or_else(|| LoadError::refused("the effect's helper is not running".into()))?;
        let state = old.snapshot();
        let new = self.load_in(
            &group,
            &proc,
            &old.label,
            &old.path,
            &old.plugin,
            state.as_deref(),
            &old.current_values(),
            sample_rate,
            max_block,
        )?;
        new.crashes
            .store(old.crashes.load(Ordering::Relaxed), Ordering::Relaxed);
        self.unload(old);
        Ok(new)
    }

    #[allow(clippy::too_many_arguments)]
    fn load_in(
        &self,
        group: &Arc<Group>,
        proc: &Arc<HelperProc>,
        label: &str,
        path: &Path,
        plugin: &str,
        state: Option<&[u8]>,
        params: &[(u32, f64)],
        sample_rate: f64,
        max_block: u32,
    ) -> Result<Arc<Instance>, LoadError> {
        let number = group.next_instance.fetch_add(1, Ordering::Relaxed);
        let transport =
            Transport::create().map_err(|e| LoadError::refused(format!("shared memory: {e}")))?;
        // The plug-in made with no state, then given its saved one: a state that crashes or
        // hangs it is told from a plug-in that does (R-FX-SAFETY).
        let reply = proc
            .ask(
                Op::Load {
                    instance: number,
                    path: path.to_string_lossy().into_owned(),
                    plugin: plugin.to_owned(),
                    state: None,
                },
                self.inner.opts.load_timeout,
            )
            .map_err(|f| LoadError::from_failure(f, During::Loading, "loading"))?;
        let loaded: Loaded =
            serde_json::from_value(reply).map_err(|e| LoadError::refused(e.to_string()))?;
        let unload = |why: String| {
            let _ = proc.request(Op::Unload { instance: number }, Duration::from_secs(5));
            Err(LoadError::refused(why))
        };
        // An effect: audio in and out. An instrument is not hosted (the owner, 2026-10-18),
        // whatever the list said of it: one that says so, or has no audio input.
        if crate::catalog::kind(&loaded.desc) == crate::Kind::Instrument {
            return unload(format!(
                "{} is an instrument: only effects are hosted",
                loaded.desc.name
            ));
        }
        if loaded.inputs == 0 || loaded.outputs == 0 {
            return unload(format!(
                "{} has no audio {}: only effects are hosted",
                loaded.desc.name,
                if loaded.inputs == 0 {
                    "input"
                } else {
                    "output"
                }
            ));
        }
        let values: Box<[AtomicU64]> = loaded
            .params
            .iter()
            .map(|_| AtomicU64::new(UNSET))
            .collect();
        let inst = Arc::new(Instance {
            label: label.to_owned(),
            number,
            path: path.to_path_buf(),
            plugin: plugin.to_owned(),
            sample_rate,
            max_block,
            transport,
            info: loaded,
            latency: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            status: AtomicU8::new(DOWN),
            generation: AtomicU32::new(0),
            values,
            dirty_since: AtomicU64::new(0),
            position: AtomicI64::new(0),
            submitted_at: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            crashes: AtomicU32::new(0),
            hung: AtomicBool::new(false),
            resumed_at: AtomicI64::new(-1),
            snapshot: Mutex::new(state.map(<[u8]>::to_vec)),
            snapshot_at: Mutex::new(Instant::now()),
            snapshot_rev: AtomicU64::new(0),
            started_ms: AtomicU64::new(0),
            quick: AtomicU32::new(0),
            gave_up: Mutex::new(None),
            changed: (Mutex::new(()), Condvar::new()),
            group: Arc::downgrade(group),
            epoch: self.inner.epoch,
            state_rev: self.inner.state_rev.clone(),
            edits: Mutex::new(Vec::new()),
            relist: AtomicBool::new(false),
        });
        for (id, plain) in params {
            match inst.param_index(*id) {
                Some(i) => inst.set_value(i, *plain),
                None => self.inner.event(
                    label,
                    "warning",
                    format!("the plug-in has no parameter {id}; its value is left out"),
                ),
            }
        }
        if let Some(state) = state {
            match proc.ask(
                Op::LoadState {
                    instance: number,
                    state: protocol::hex(state),
                },
                self.inner.opts.load_timeout,
            ) {
                Ok(_) => {}
                Err(Failure::Refused(e)) => {
                    return unload(format!("its saved settings were refused: {e}")).map_err(
                        |mut e: LoadError| {
                            e.during = During::Settings;
                            e
                        },
                    );
                }
                Err(f) => {
                    return Err(LoadError::from_failure(
                        f,
                        During::Settings,
                        "its saved settings were restored",
                    ));
                }
            }
        }
        match activate_why(&inst, proc, &self.inner.opts) {
            Ok(()) => {}
            Err(Failure::Refused(e)) => return unload(e),
            Err(f) => return Err(LoadError::from_failure(f, During::Starting, "starting")),
        }
        if let Ok(s) = save_state(&inst, proc) {
            inst.keep_snapshot(s);
        }
        if !proc.is_alive() {
            return Err(LoadError::from_failure(
                Failure::Exited,
                During::Starting,
                "starting",
            ));
        }
        inst.dirty_since.store(0, Ordering::Relaxed);
        inst.started_ms.store(inst.now_ms(), Ordering::Relaxed);
        inst.status.store(RUNNING, Ordering::Release);
        lock(&group.instances).push(inst.clone());
        Ok(inst)
    }

    /// Takes an effect out: its node passes its input from now on, the helper unloads it,
    /// and a helper left with none quits.
    pub fn unload(&self, inst: &Arc<Instance>) {
        inst.status.store(RETIRED, Ordering::Release);
        inst.transport.mark_dead();
        inst.notify();
        let Some(group) = inst.group.upgrade() else {
            return;
        };
        let empty = {
            let mut list = lock(&group.instances);
            list.retain(|i| !Arc::ptr_eq(i, inst));
            list.is_empty()
        };
        if empty {
            retire(&self.inner, &group);
        } else if let Some(p) = lock(&group.proc).clone() {
            let _ = p.request(
                Op::Unload {
                    instance: inst.number,
                },
                Duration::from_secs(5),
            );
        }
    }

    /// Takes a snapshot of every running instance now.
    pub fn snapshot_all(&self) {
        let groups: Vec<Arc<Group>> = lock(&self.inner.groups).clone();
        for g in groups {
            for inst in lock(&g.instances).clone() {
                let _ = inst.take_snapshot();
            }
        }
    }

    /// Closing: every helper still running is stopped (killed after a moment, so that one
    /// hung in a plug-in's load does not outlive the application), none started again, and no
    /// effect loaded from now on.
    pub fn close(&self) {
        self.inner.closing.store(true, Ordering::Release);
        let groups: Vec<Arc<Group>> = lock(&self.inner.groups).drain(..).collect();
        for g in groups {
            g.retired.store(true, Ordering::Release);
            for inst in lock(&g.instances).iter() {
                inst.status.store(RETIRED, Ordering::Release);
                inst.transport.mark_dead();
                inst.notify();
            }
            if let Some(p) = lock(&g.proc).take() {
                p.shutdown(Duration::from_millis(500));
            }
        }
    }

    /// The process IDs of the running helpers.
    pub fn helper_pids(&self) -> Vec<u32> {
        lock(&self.inner.groups)
            .iter()
            .filter_map(|g| lock(&g.proc).as_ref().map(|p| p.pid()))
            .collect()
    }
}

/// A helper with no instance left quits; it is not started again.
fn retire(inner: &Arc<Inner>, group: &Arc<Group>) {
    group.retired.store(true, Ordering::Release);
    lock(&inner.groups).retain(|g| !Arc::ptr_eq(g, group));
    if let Some(p) = lock(&group.proc).take() {
        p.shutdown(Duration::from_secs(2));
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        self.inner.closing.store(true, Ordering::Release);
        if let Some(m) = self.monitor.take() {
            let _ = m.join();
        }
        let groups: Vec<Arc<Group>> = lock(&self.inner.groups).drain(..).collect();
        for g in groups {
            g.retired.store(true, Ordering::Release);
            for inst in lock(&g.instances).iter() {
                inst.status.store(RETIRED, Ordering::Release);
                inst.transport.mark_dead();
                inst.notify();
            }
            if let Some(p) = lock(&g.proc).take() {
                p.shutdown(Duration::from_secs(2));
            }
        }
    }
}

impl Inner {
    pub(crate) fn event(&self, effect: &str, kind: &str, detail: String) {
        lock(&self.events).push(Event {
            effect: effect.to_owned(),
            kind: kind.to_owned(),
            detail,
            at: SystemTime::now(),
        });
    }
}

fn spawn(inner: &Arc<Inner>, group: &Arc<Group>) -> Result<Arc<HelperProc>, String> {
    let weak_inner = Arc::downgrade(inner);
    let weak_group = Arc::downgrade(group);
    HelperProc::spawn(
        &inner.opts.helper,
        Box::new(move |proc: &Arc<HelperProc>| {
            if let (Some(inner), Some(group)) = (weak_inner.upgrade(), weak_group.upgrade()) {
                on_exit(&inner, &group, proc);
            }
        }),
    )
}

fn activate_why(inst: &Instance, proc: &HelperProc, opts: &SandboxOptions) -> Result<(), Failure> {
    let start = inst.transport.request().word().load(Ordering::Acquire);
    // Nothing is outstanding for the new helper: `done` matches the last request, which
    // also clears the dead flag.
    inst.transport
        .done()
        .store(start & daw_ipc::layout::SEQ_MASK);
    let a: Activated = serde_json::from_value(proc.ask(
        Op::Activate {
            instance: inst.number,
            sample_rate: inst.sample_rate,
            max_block: inst.max_block,
            params: inst.current_values(),
            transport: inst.transport.name().clone(),
            start,
            realtime: opts.realtime,
        },
        opts.load_timeout,
    )?)
    .map_err(|e| Failure::Refused(e.to_string()))?;
    inst.latency.store(a.latency, Ordering::Relaxed);
    inst.tail.store(a.tail, Ordering::Relaxed);
    Ok(())
}

fn save_state(inst: &Instance, proc: &HelperProc) -> Result<Vec<u8>, String> {
    let v = proc.request(
        Op::SaveState {
            instance: inst.number,
        },
        Duration::from_secs(10),
    )?;
    let s = v
        .get("state")
        .and_then(serde_json::Value::as_str)
        .ok_or("no state in the reply")?;
    protocol::unhex(s)
}

/// A helper's stdout closed: it exited, crashed or was killed. Its instances are marked
/// down, another helper is started and they are restored: made, given their last state in a
/// step of its own, activated. One that crashes or hangs while that is done, or that has
/// crashed too often, is given up on, and a helper left with nothing alive is stopped
/// (decisions.md R-FX-SAFETY).
fn on_exit(inner: &Arc<Inner>, group: &Arc<Group>, proc: &Arc<HelperProc>) {
    // One death at a time: the helper started here may die while it restores, and its own
    // death is dealt with after this one (by then its instances are given up on, or running).
    let _one = lock(&group.recovering);
    let current = lock(&group.proc)
        .as_ref()
        .is_some_and(|p| Arc::ptr_eq(p, proc));
    let instances: Vec<Arc<Instance>> = lock(&group.instances)
        .iter()
        .filter(|i| matches!(i.status(), RUNNING | DOWN))
        .cloned()
        .collect();
    if inner.closing.load(Ordering::Acquire)
        || group.retired.load(Ordering::Acquire)
        || !current
        || instances.is_empty()
    {
        let _ = proc.reap();
        return;
    }
    // Down at once, before the process is reaped: a crashed process can take a while to
    // end (its core written), and meanwhile its node would wait out every block's deadline.
    let mut hung = false;
    for inst in &instances {
        inst.status.store(DOWN, Ordering::Release);
        inst.submitted_at.store(0, Ordering::Release);
        inst.transport.mark_dead();
        inst.crashes.fetch_add(1, Ordering::Relaxed);
        let ran = inst
            .now_ms()
            .saturating_sub(inst.started_ms.load(Ordering::Relaxed));
        if ran < QUICK.as_millis() as u64 {
            inst.quick.fetch_add(1, Ordering::Relaxed);
        } else {
            inst.quick.store(0, Ordering::Relaxed);
        }
        inst.notify();
    }
    let how = proc.reap();
    for inst in &instances {
        let mut detail = if inst.hung.swap(false, Ordering::AcqRel) {
            hung = true;
            format!(
                "its helper (pid {}) stopped returning blocks for {:?}; the watchdog killed it",
                proc.pid(),
                inner.opts.hang_timeout
            )
        } else {
            format!("its helper (pid {}) {how}", proc.pid())
        };
        if let Some(last) = proc.stderr_tail().last() {
            detail.push_str(&format!("; its last output: {last}"));
        }
        inner.event(&inst.label, "crashed", detail);
    }
    let restarts = group.restarts.fetch_add(1, Ordering::Relaxed) + 1;
    // The event first: whoever the status wakes finds it recorded.
    let fail = |inst: &Instance, why: String, gave: GaveUp| {
        inner.event(&inst.label, "failed", why);
        inst.give_up(gave);
    };
    if restarts > inner.opts.max_restarts {
        for inst in &instances {
            // Crashed every time within a second of being started with its state: its state
            // is thought the cause (R-FX-SAFETY).
            let settings = inst.quick.load(Ordering::Relaxed) >= restarts;
            fail(
                inst,
                format!(
                    "gave up after {} restarts of its helper{}; it passes the sound through untouched from now on",
                    inner.opts.max_restarts,
                    if settings {
                        ", each crash within a second of its settings being restored"
                    } else {
                        ""
                    }
                ),
                GaveUp { hung, settings },
            );
        }
        *lock(&group.proc) = None;
        return;
    }
    let new = match spawn(inner, group) {
        Ok(p) => p,
        Err(e) => {
            for inst in &instances {
                fail(
                    inst,
                    e.clone(),
                    GaveUp {
                        hung,
                        settings: false,
                    },
                );
            }
            *lock(&group.proc) = None;
            return;
        }
    };
    *lock(&group.proc) = Some(new.clone());
    let timeout = inner.opts.load_timeout;
    for inst in &instances {
        let state = inst.snapshot();
        let made = new
            .ask(
                Op::Load {
                    instance: inst.number,
                    path: inst.path.to_string_lossy().into_owned(),
                    plugin: inst.plugin.clone(),
                    state: None,
                },
                timeout,
            )
            .map_err(|f| LoadError::from_failure(f, During::Loading, "loading"));
        let restored = made.and_then(|_| match &state {
            None => Ok(()),
            Some(s) => new
                .ask(
                    Op::LoadState {
                        instance: inst.number,
                        state: protocol::hex(s),
                    },
                    timeout,
                )
                .map(|_| ())
                .map_err(|f| {
                    LoadError::from_failure(f, During::Settings, "its settings were restored")
                }),
        });
        let started = restored.and_then(|()| {
            activate_why(inst, &new, &inner.opts)
                .map_err(|f| LoadError::from_failure(f, During::Starting, "starting"))
        });
        match started {
            Ok(()) => {
                inner.event(
                    &inst.label,
                    "restored",
                    format!(
                        "started its helper again (pid {}) with its {} state",
                        new.pid(),
                        if state.is_some() {
                            "last saved"
                        } else {
                            "initial"
                        },
                    ),
                );
                inst.started_ms.store(inst.now_ms(), Ordering::Relaxed);
                inst.generation.fetch_add(1, Ordering::AcqRel);
                inst.status.store(RUNNING, Ordering::Release);
                inst.notify();
            }
            Err(e) => {
                // Not tried again: a state that crashes it while restored would crash it each
                // time, and one that hangs would hang each time.
                let settings = e.during == During::Settings && e.fault != Fault::Refused;
                fail(
                    inst,
                    format!("could not be restored: {e}"),
                    GaveUp {
                        hung: e.fault == Fault::Hung,
                        settings,
                    },
                );
            }
        }
    }
    // A helper with nothing alive in it (hung, say) is stopped.
    if !instances.iter().any(|i| i.status() == RUNNING) {
        *lock(&group.proc) = None;
        new.kill();
    }
}

/// The monitor thread: the watchdog, and the snapshots.
fn monitor(inner: &Weak<Inner>) {
    loop {
        std::thread::sleep(Duration::from_millis(20));
        let Some(inner) = inner.upgrade() else { return };
        if inner.closing.load(Ordering::Acquire) {
            return;
        }
        let groups: Vec<Arc<Group>> = lock(&inner.groups).clone();
        for g in groups {
            let Some(p) = lock(&g.proc).clone() else {
                continue;
            };
            let instances: Vec<Arc<Instance>> = lock(&g.instances).clone();
            // A plug-in that changed itself (its editor) has a state to save again.
            for number in p.take_edited() {
                if let Some(i) = instances.iter().find(|i| i.number == number) {
                    i.mark_dirty();
                }
            }
            // Its editor's values (a pinned parameter's base) and a word that its parameters
            // changed, for the chain (decisions.md R-FX-PARAMS).
            for (number, id, value) in p.take_param_edits() {
                if let Some(i) = instances.iter().find(|i| i.number == number) {
                    let mut e = lock(&i.edits);
                    match e.iter_mut().find(|x| x.0 == id) {
                        Some(x) => x.1 = value,
                        None => e.push((id, value)),
                    }
                }
            }
            for number in p.take_relist() {
                if let Some(i) = instances.iter().find(|i| i.number == number) {
                    i.relist.store(true, Ordering::Release);
                }
            }
            for inst in instances {
                if inst.status() != RUNNING {
                    continue;
                }
                let since = inst.submitted_at.load(Ordering::Acquire);
                if inner.opts.realtime && since != 0 {
                    let age = Duration::from_micros(inst.now_us().saturating_sub(since));
                    if age > inner.opts.hang_timeout {
                        inst.hung.store(true, Ordering::Release);
                        inner.event(
                            &inst.label,
                            "hung",
                            format!("no block back for {age:?}; killing its helper"),
                        );
                        p.kill();
                        break;
                    }
                }
                let dirty = inst.dirty_since.load(Ordering::Relaxed);
                let now_ms = inst.epoch.elapsed().as_millis() as u64;
                let debounced = dirty != 0
                    && now_ms.saturating_sub(dirty)
                        >= inner.opts.snapshot_debounce.as_millis() as u64;
                let periodic = lock(&inst.snapshot_at).elapsed() >= inner.opts.snapshot_interval;
                if (debounced || periodic)
                    && let Ok(s) = save_state(&inst, &p)
                {
                    inst.keep_snapshot(s);
                    if debounced {
                        let _ = inst.dirty_since.compare_exchange(
                            dirty,
                            0,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        );
                    }
                }
            }
        }
    }
}
