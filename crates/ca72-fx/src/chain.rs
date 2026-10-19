//! The chain (docs/fx-host.md): up to [`MAX_SLOTS`] effects after the synth's output, in
//! order, each in a helper process of its own, each bypassed or not; saved with the
//! standalone's session.
//!
//! [`Chain`] is the control side (any thread but the audio thread): it loads and unloads
//! effects, orders them, opens their editors, saves and restores them. [`Runner`] is the
//! audio side, owned by the synth's processor: it takes the chain's changes from a queue,
//! without locks or allocation, and runs the effects' nodes. Nodes it lets go of go back on
//! another queue, to be dropped off the audio thread.

use crate::node::Node;
use crate::sandbox::{self, Instance, Sandbox, SandboxOptions, lock};
use crate::{Mode, catalog};
use daw_ipc::MAX_BLOCK;
use daw_ipc::protocol::{self, Op};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

/// The chain's length (the owner, 2026-10-18).
pub const MAX_SLOTS: usize = 4;

/// The saved file's format.
pub const FORMAT: u32 = 1;

/// How many changes may wait for the audio thread (while no audio device plays, they wait
/// for the next one).
const QUEUE: usize = 1024;

/// An effect as the session keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedSlot {
    /// `clap` or `vst3`.
    pub format: String,
    /// The plug-in's file or bundle.
    pub path: PathBuf,
    /// Its ID within the file (CLAP's ID; VST3's processor class, in hex).
    pub plugin: String,
    pub name: String,
    #[serde(default)]
    pub vendor: String,
    #[serde(default)]
    pub bypass: bool,
    /// Its state as it last gave it, in hex; none if it never ran here.
    #[serde(default)]
    pub state: Option<String>,
}

/// The chain as the session keeps it: `effects.json` in the standalone's session folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedChain {
    pub format: u32,
    pub slots: Vec<SavedSlot>,
}

impl SavedChain {
    /// The saved chain in `path`: `None` if there is none; an error for a file that cannot
    /// be read, which is left as it is.
    pub fn load(path: &Path) -> Result<Option<SavedChain>, String> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let chain: SavedChain = serde_json::from_slice(&bytes)
            .map_err(|e| format!("{} could not be read: {e}", path.display()))?;
        if chain.format != FORMAT {
            return Err(format!(
                "{} is format {}, not {FORMAT}: left as it is",
                path.display(),
                chain.format
            ));
        }
        Ok(Some(chain))
    }
}

/// How the chain runs.
#[derive(Debug, Clone)]
pub struct ChainOptions {
    pub sandbox: SandboxOptions,
    pub mode: Mode,
    /// Where the chain is saved after each change (none: not saved).
    pub file: Option<PathBuf>,
    /// The sample rate effects are loaded at before an audio device plays.
    pub provisional_rate: f64,
}

impl ChainOptions {
    pub fn new(helper: PathBuf) -> ChainOptions {
        ChainOptions {
            sandbox: SandboxOptions::new(helper),
            mode: Mode::Sync,
            file: None,
            provisional_rate: 48_000.0,
        }
    }
}

/// An effect's state, as [`Chain::status`] tells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Running,
    /// Its helper died or hung; it is being started again. Dry meanwhile.
    Restarting,
    /// Given up on: dry from now on.
    Failed,
    /// Its plug-in could not be loaded (not installed here, say): dry, its saved state kept.
    Missing,
}

/// One slot of the chain, for a view of it.
#[derive(Debug, Clone)]
pub struct SlotStatus {
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub path: PathBuf,
    pub plugin: String,
    pub bypass: bool,
    pub health: Health,
    /// What went wrong, for a missing or failed effect.
    pub error: Option<String>,
    /// Its latency in frames, as it is in the chain now (0 bypassed).
    pub latency: u32,
    pub crashes: u32,
    pub misses: u64,
    pub helper_pid: Option<u32>,
    /// Its parameters, for a view of them.
    pub params: Vec<protocol::ParamInfo>,
}

/// A slot on the audio thread.
#[derive(Debug)]
pub struct Stage {
    node: Option<Box<Node>>,
    bypass: Arc<AtomicBool>,
}

enum Msg {
    Insert { at: usize, stage: Stage },
    Remove { at: usize },
    Move { from: usize, to: usize },
    Param { at: usize, id: u32, value: f64 },
}

struct Slot {
    saved: SavedSlot,
    inst: Option<Arc<Instance>>,
    bypass: Arc<AtomicBool>,
    error: Option<String>,
}

struct Core {
    slots: Vec<Slot>,
    /// The sample rate and largest block effects run at now, and whether an audio device
    /// has given them.
    rate: f64,
    block: usize,
}

struct Shared {
    sandbox: Sandbox,
    opts: ChainOptions,
    core: Mutex<Core>,
    to_audio: Mutex<rtrb::Producer<Msg>>,
    from_audio: Mutex<rtrb::Consumer<Stage>>,
    /// The sandbox's state revision last saved.
    saved_rev: AtomicU64,
    /// Events already logged.
    logged: Mutex<usize>,
    /// Changes are not saved one by one (restoring, closing).
    quiet: AtomicBool,
    /// The file could not be read: it is never written over this session.
    keep_file: AtomicBool,
}

/// The chain's control side.
#[derive(Clone)]
pub struct Chain {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chain").finish_non_exhaustive()
    }
}

/// The chain's audio side.
pub struct Runner {
    shared: Arc<Shared>,
    from_control: rtrb::Consumer<Msg>,
    to_control: rtrb::Producer<Stage>,
    stages: Vec<Stage>,
    frame: i64,
}

impl std::fmt::Debug for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner")
            .field("stages", &self.stages.len())
            .finish_non_exhaustive()
    }
}

impl Chain {
    /// An empty chain and its audio side. A housekeeping thread saves the chain when an
    /// effect's state changes and logs what happens to the effects to stderr.
    pub fn new(opts: ChainOptions) -> (Chain, Runner) {
        let (to_audio, from_control) = rtrb::RingBuffer::new(QUEUE);
        let (to_control, from_audio) = rtrb::RingBuffer::new(QUEUE);
        let shared = Arc::new(Shared {
            sandbox: Sandbox::new(opts.sandbox.clone()),
            core: Mutex::new(Core {
                slots: Vec::new(),
                rate: opts.provisional_rate,
                block: MAX_BLOCK,
            }),
            opts,
            to_audio: Mutex::new(to_audio),
            from_audio: Mutex::new(from_audio),
            saved_rev: AtomicU64::new(0),
            logged: Mutex::new(0),
            quiet: AtomicBool::new(false),
            keep_file: AtomicBool::new(false),
        });
        let weak = Arc::downgrade(&shared);
        let _ = std::thread::Builder::new()
            .name("ca72-fx-housekeeping".into())
            .spawn(move || housekeeping(&weak));
        let runner = Runner {
            shared: shared.clone(),
            from_control,
            to_control,
            stages: Vec::with_capacity(MAX_SLOTS),
            frame: 0,
        };
        (Chain { shared }, runner)
    }

    pub fn sandbox(&self) -> &Sandbox {
        &self.shared.sandbox
    }

    pub fn len(&self) -> usize {
        lock(&self.shared.core).slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds a found effect at the chain's end; its place.
    pub fn add(&self, found: &catalog::Found) -> Result<usize, String> {
        if found.kind == catalog::Kind::Instrument {
            return Err(format!(
                "{} is an instrument: only effects are hosted",
                found.desc.name
            ));
        }
        self.insert(
            SavedSlot {
                format: found.desc.format.clone(),
                path: found.path.clone(),
                plugin: found.desc.id.clone(),
                name: found.desc.name.clone(),
                vendor: found.desc.vendor.clone(),
                bypass: false,
                state: None,
            },
            false,
        )
    }

    /// Adds an effect as saved (its state restored) at the chain's end; its place. One whose
    /// plug-in cannot be loaded is kept, missing: dry, its state saved again as it was.
    pub fn add_saved(&self, saved: SavedSlot) -> Result<usize, String> {
        self.insert(saved, true)
    }

    /// Adds an effect at the end; one that does not load is kept, missing, if `keep`, else
    /// an error.
    fn insert(&self, saved: SavedSlot, keep: bool) -> Result<usize, String> {
        let shared = &self.shared;
        shared.drain_retired();
        let mut core = lock(&shared.core);
        if core.slots.len() >= MAX_SLOTS {
            return Err(format!("the chain holds {MAX_SLOTS} effects at most"));
        }
        let at = core.slots.len();
        let bypass = Arc::new(AtomicBool::new(saved.bypass));
        let state = match saved.state.as_deref().map(protocol::unhex).transpose() {
            Ok(s) => s,
            Err(e) => return Err(format!("{}'s saved state is damaged: {e}", saved.name)),
        };
        let label = saved.name.clone();
        let loaded = shared.sandbox.load(
            &label,
            &saved.path,
            &saved.plugin,
            state.as_deref(),
            &[],
            core.rate,
            MAX_BLOCK as u32,
        );
        let (inst, error) = match loaded {
            Ok(i) => (Some(i), None),
            Err(e) if !keep => return Err(e),
            Err(e) => {
                shared
                    .sandbox
                    .event(&label, "warning", format!("not loaded: {e}"));
                (None, Some(e))
            }
        };
        let stage = Stage {
            node: inst.as_ref().map(|i| Box::new(shared.node(i, core.block))),
            bypass: bypass.clone(),
        };
        if lock(&shared.to_audio)
            .push(Msg::Insert { at, stage })
            .is_err()
        {
            if let Some(i) = &inst {
                shared.sandbox.unload(i);
            }
            return Err("the audio thread is not taking changes".into());
        }
        core.slots.push(Slot {
            saved,
            inst,
            bypass,
            error,
        });
        drop(core);
        shared.save();
        Ok(at)
    }

    /// Takes the effect at `at` out of the chain.
    pub fn remove(&self, at: usize) -> Result<(), String> {
        let shared = &self.shared;
        shared.drain_retired();
        let mut core = lock(&shared.core);
        if at >= core.slots.len() {
            return Err(format!("there is no effect {}", at + 1));
        }
        lock(&shared.to_audio)
            .push(Msg::Remove { at })
            .map_err(|_| "the audio thread is not taking changes")?;
        let slot = core.slots.remove(at);
        drop(core);
        // Unloading marks it gone first: if the audio thread is in its block now, the block
        // ends at once, dry.
        if let Some(i) = &slot.inst {
            shared.sandbox.unload(i);
        }
        shared.save();
        Ok(())
    }

    /// Moves the effect at `from` to `to`, the others keeping their order.
    pub fn move_to(&self, from: usize, to: usize) -> Result<(), String> {
        let shared = &self.shared;
        let mut core = lock(&shared.core);
        let n = core.slots.len();
        if from >= n || to >= n {
            return Err(format!("the chain has {n} effects"));
        }
        if from == to {
            return Ok(());
        }
        lock(&shared.to_audio)
            .push(Msg::Move { from, to })
            .map_err(|_| "the audio thread is not taking changes")?;
        let slot = core.slots.remove(from);
        core.slots.insert(to, slot);
        drop(core);
        shared.save();
        Ok(())
    }

    /// Bypasses the effect at `at` (its input goes out untouched, to the bit) or not.
    pub fn set_bypass(&self, at: usize, on: bool) -> Result<(), String> {
        let shared = &self.shared;
        let mut core = lock(&shared.core);
        let slot = core
            .slots
            .get_mut(at)
            .ok_or_else(|| format!("there is no effect {}", at + 1))?;
        slot.bypass.store(on, Ordering::Release);
        slot.saved.bypass = on;
        drop(core);
        shared.save();
        Ok(())
    }

    /// Sets a parameter of the effect at `at` (its ID, a plain value) from the next block.
    pub fn set_param(&self, at: usize, id: u32, value: f64) -> Result<(), String> {
        let shared = &self.shared;
        let core = lock(&shared.core);
        let slot = core
            .slots
            .get(at)
            .ok_or_else(|| format!("there is no effect {}", at + 1))?;
        let inst = slot.inst.as_ref().ok_or("the effect is not loaded")?;
        if inst.param_index(id).is_none() {
            return Err(format!("{} has no parameter {id}", slot.saved.name));
        }
        lock(&shared.to_audio)
            .push(Msg::Param { at, id, value })
            .map_err(|_| "the audio thread is not taking changes".into())
    }

    /// Opens the editor of the effect at `at` in a window of its own (its helper's), or
    /// shows it again; what the helper says of the window.
    pub fn open_editor(&self, at: usize) -> Result<serde_json::Value, String> {
        let (inst, title) = self.running(at)?;
        let h = inst.helper().ok_or("the effect's helper is not running")?;
        h.request(
            Op::GuiOpen {
                instance: inst.number(),
                title,
                x: None,
                y: None,
            },
            Duration::from_secs(10),
        )
    }

    /// Closes the editor of the effect at `at`.
    pub fn close_editor(&self, at: usize) -> Result<serde_json::Value, String> {
        let (inst, _) = self.running(at)?;
        let h = inst.helper().ok_or("the effect's helper is not running")?;
        h.request(
            Op::GuiClose {
                instance: inst.number(),
            },
            Duration::from_secs(10),
        )
    }

    /// Kills the helper of the effect at `at` (to try the recovery).
    pub fn kill(&self, at: usize) -> Result<(), String> {
        let (inst, _) = self.running(at)?;
        inst.kill_helper();
        Ok(())
    }

    fn running(&self, at: usize) -> Result<(Arc<Instance>, String), String> {
        let core = lock(&self.shared.core);
        let slot = core
            .slots
            .get(at)
            .ok_or_else(|| format!("there is no effect {}", at + 1))?;
        let inst = slot.inst.clone().ok_or("the effect is not loaded")?;
        Ok((
            inst,
            format!("CA-72 effect {}: {}", at + 1, slot.saved.name),
        ))
    }

    /// Every slot, in order.
    pub fn status(&self) -> Vec<SlotStatus> {
        let core = lock(&self.shared.core);
        let pipelined = self.shared.opts.mode == Mode::Pipelined;
        core.slots
            .iter()
            .map(|s| {
                let bypass = s.bypass.load(Ordering::Acquire);
                let (health, latency, crashes, misses, pid, params) = match &s.inst {
                    None => (Health::Missing, 0, 0, 0, None, Vec::new()),
                    Some(i) => (
                        match i.status() {
                            sandbox::RUNNING => Health::Running,
                            sandbox::DOWN => Health::Restarting,
                            _ => Health::Failed,
                        },
                        i.latency.load(Ordering::Relaxed)
                            + if pipelined { core.block as u32 } else { 0 },
                        i.crashes.load(Ordering::Relaxed),
                        i.misses.load(Ordering::Relaxed),
                        i.helper_pid(),
                        i.info.params.clone(),
                    ),
                };
                SlotStatus {
                    name: s.saved.name.clone(),
                    vendor: s.saved.vendor.clone(),
                    format: s.saved.format.clone(),
                    path: s.saved.path.clone(),
                    plugin: s.saved.plugin.clone(),
                    bypass,
                    latency: if bypass || health != Health::Running {
                        0
                    } else {
                        latency
                    },
                    health,
                    error: s.error.clone(),
                    crashes,
                    misses,
                    helper_pid: pid,
                    params,
                }
            })
            .collect()
    }

    /// The chain's latency now, in frames: the sum of its running effects' that are not
    /// bypassed. What the player hears late behind the keys, besides the device's own.
    pub fn latency(&self) -> u32 {
        self.status().iter().map(|s| s.latency).sum()
    }

    /// The sample rate the effects run at now.
    pub fn sample_rate(&self) -> f64 {
        lock(&self.shared.core).rate
    }

    /// The chain as the session keeps it, with each effect's latest state.
    pub fn saved(&self) -> SavedChain {
        self.shared.saved()
    }

    /// Takes every effect's state now and saves the chain (if it has a file).
    pub fn save_now(&self) -> Result<(), String> {
        self.shared.sandbox.snapshot_all();
        self.shared.write()
    }

    /// Replaces the chain with a saved one. An effect that cannot be loaded is kept,
    /// missing; an error is said for each, and the rest load.
    pub fn restore(&self, chain: &SavedChain) -> Vec<String> {
        while !self.is_empty() {
            let _ = self.remove(self.len() - 1);
        }
        let mut errors = Vec::new();
        for s in chain.slots.iter().take(MAX_SLOTS) {
            if let Err(e) = self.add_saved(s.clone()) {
                errors.push(format!("{}: {e}", s.name));
            }
        }
        for s in &self.status() {
            if let Some(e) = &s.error {
                errors.push(format!("{}: {e}", s.name));
            }
        }
        if chain.slots.len() > MAX_SLOTS {
            errors.push(format!(
                "{} effects past the {MAX_SLOTS}th were left out",
                chain.slots.len() - MAX_SLOTS
            ));
        }
        errors
    }

    /// Restores the chain from its file, if it has one; the errors, as [`Chain::restore`].
    pub fn restore_file(&self) -> Vec<String> {
        let Some(path) = self.shared.opts.file.clone() else {
            return Vec::new();
        };
        match SavedChain::load(&path) {
            Ok(Some(c)) => {
                // Saving while restoring would write a shorter chain over the file.
                self.shared.quiet.store(true, Ordering::Release);
                let errors = self.restore(&c);
                self.shared.quiet.store(false, Ordering::Release);
                self.shared.save();
                errors
            }
            Ok(None) => Vec::new(),
            Err(e) => {
                // Not written over: the chain starts empty and is not saved this session.
                self.shared.keep_file.store(true, Ordering::Release);
                vec![e]
            }
        }
    }

    /// What happened to the effects from the `from`th event on, and how many there are.
    pub fn events_from(&self, from: usize) -> (Vec<sandbox::Event>, usize) {
        self.shared.sandbox.events_from(from)
    }

    /// Saves the chain with every effect's state, then unloads every effect and stops
    /// every helper. The audio thread must have stopped.
    pub fn shutdown(&self) -> Result<(), String> {
        let saved = self.save_now();
        self.shared.quiet.store(true, Ordering::Release);
        let mut core = lock(&self.shared.core);
        for s in core.slots.drain(..) {
            if let Some(i) = &s.inst {
                self.shared.sandbox.unload(i);
            }
        }
        saved
    }
}

impl Shared {
    fn node(&self, inst: &Arc<Instance>, block: usize) -> Node {
        Node::new(
            inst.clone(),
            self.opts.mode,
            self.opts.sandbox.realtime,
            block,
            self.opts.sandbox.hang_timeout,
        )
    }

    /// Drops what the audio thread let go of.
    fn drain_retired(&self) {
        let mut q = lock(&self.from_audio);
        while let Ok(stage) = q.pop() {
            drop(stage);
        }
    }

    fn saved(&self) -> SavedChain {
        let core = lock(&self.core);
        SavedChain {
            format: FORMAT,
            slots: core
                .slots
                .iter()
                .map(|s| SavedSlot {
                    bypass: s.bypass.load(Ordering::Acquire),
                    state: s
                        .inst
                        .as_ref()
                        .and_then(|i| i.snapshot())
                        .map(|b| protocol::hex(&b))
                        .or_else(|| s.saved.state.clone()),
                    ..s.saved.clone()
                })
                .collect(),
        }
    }

    /// Saves the chain, unless it is quiet, keeps an unread file or has none; a failure is
    /// logged.
    fn save(&self) {
        if self.quiet.load(Ordering::Acquire) || self.keep_file.load(Ordering::Acquire) {
            return;
        }
        if let Err(e) = self.write() {
            eprintln!("CA-72 effects: {e}");
        }
    }

    fn write(&self) -> Result<(), String> {
        let Some(path) = &self.opts.file else {
            return Ok(());
        };
        if self.keep_file.load(Ordering::Acquire) {
            return Err(format!(
                "{} could not be read when the application started, so it is not written over",
                path.display()
            ));
        }
        let rev = self.sandbox.state_rev();
        let bytes = serde_json::to_vec_pretty(&self.saved()).map_err(|e| e.to_string())?;
        crate::write_atomically(path, &bytes)
            .map_err(|e| format!("the effects could not be saved in {}: {e}", path.display()))?;
        self.saved_rev.store(rev, Ordering::Release);
        Ok(())
    }
}

/// Saves the chain when an effect's state changed, drops what the audio thread let go of,
/// and logs the effects' events.
fn housekeeping(shared: &Weak<Shared>) {
    let mut last_save = Instant::now();
    loop {
        std::thread::sleep(Duration::from_millis(100));
        let Some(s) = shared.upgrade() else { return };
        s.drain_retired();
        let from = *lock(&s.logged);
        let (events, n) = s.sandbox.events_from(from);
        for e in events {
            eprintln!("CA-72 effects: {} {}: {}", e.effect, e.kind, e.detail);
        }
        *lock(&s.logged) = n;
        // At most a save a second: a plug-in whose state changes at every change of a knob.
        if s.sandbox.state_rev() != s.saved_rev.load(Ordering::Acquire)
            && last_save.elapsed() >= Duration::from_secs(1)
        {
            s.save();
            last_save = Instant::now();
        }
    }
}

impl Runner {
    /// Before the audio device plays (on a control thread, no audio thread running): the
    /// device's sample rate and largest block. Changes the chain made meanwhile are taken;
    /// at another sample rate every effect is loaded again in its helper with its latest
    /// state; every node is made again for the block size.
    pub fn prepare(&mut self, rate: f64, block: usize) {
        self.take_changes();
        let shared = self.shared.clone();
        let mut core = lock(&shared.core);
        let block = block.clamp(1, MAX_BLOCK);
        let reload = rate != core.rate;
        core.rate = rate;
        core.block = block;
        for (slot, stage) in core.slots.iter_mut().zip(self.stages.iter_mut()) {
            if reload && let Some(old) = slot.inst.clone() {
                match shared.sandbox.reload(&old, rate, MAX_BLOCK as u32) {
                    Ok(new) => slot.inst = Some(new),
                    Err(e) => {
                        shared.sandbox.event(
                            &slot.saved.name,
                            "failed",
                            format!("could not be loaded again at {rate} Hz: {e}"),
                        );
                        slot.saved.state = old.snapshot().map(|b| protocol::hex(&b));
                        slot.error = Some(e);
                        slot.inst = None;
                        shared.sandbox.unload(&old);
                    }
                }
            }
            stage.node = slot.inst.as_ref().map(|i| Box::new(shared.node(i, block)));
        }
    }

    /// Takes the chain's changes (audio thread: no allocation, no locks).
    fn take_changes(&mut self) {
        while let Ok(msg) = self.from_control.pop() {
            match msg {
                Msg::Insert { at, stage } => {
                    if self.stages.len() < MAX_SLOTS && at <= self.stages.len() {
                        self.stages.insert(at, stage);
                    } else {
                        let _ = self.to_control.push(stage);
                    }
                }
                Msg::Remove { at } => {
                    if at < self.stages.len() {
                        let stage = self.stages.remove(at);
                        if let Err(rtrb::PushError::Full(stage)) = self.to_control.push(stage) {
                            // Never dropped here: kept to the end rather (the queue holds
                            // a thousand).
                            std::mem::forget(stage);
                        }
                    }
                }
                Msg::Move { from, to } => {
                    if from < self.stages.len() && to < self.stages.len() {
                        let s = self.stages.remove(from);
                        self.stages.insert(to, s);
                    }
                }
                Msg::Param { at, id, value } => {
                    if let Some(node) = self.stages.get_mut(at).and_then(|s| s.node.as_mut()) {
                        node.set_param(id, value);
                    }
                }
            }
        }
    }

    /// The synth's output through the chain, in place (audio thread). In real time each
    /// effect's block is waited for until `deadline` at the latest; past it, and for any
    /// effect not running, its input goes on untouched.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], deadline: Option<Instant>) {
        self.take_changes();
        let n = left.len().min(right.len());
        let mut off = 0;
        while off < n {
            let end = (off + MAX_BLOCK).min(n);
            for stage in &mut self.stages {
                if stage.bypass.load(Ordering::Relaxed) {
                    continue;
                }
                if let Some(node) = &mut stage.node {
                    node.process(
                        self.frame + off as i64,
                        &mut left[off..end],
                        &mut right[off..end],
                        deadline,
                    );
                }
            }
            off = end;
        }
        self.frame += n as i64;
    }

    /// Whether there are effects to run (a bypassed one counts).
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }
}
