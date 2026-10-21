//! The chain (docs/fx-host.md): up to [`MAX_SLOTS`] effects after the synth's output, in
//! order, each in a helper process of its own, each bypassed or not; saved with the
//! standalone's session.
//!
//! [`Chain`] is the control side (any thread but the audio thread): it loads and unloads
//! effects, orders them, opens their editors, saves and restores them. [`Runner`] is the
//! audio side, owned by the synth's processor: it takes the chain's changes from a queue,
//! without locks or allocation, and runs the effects' nodes. Nodes it lets go of go back on
//! another queue, to be dropped off the audio thread.
//!
//! Nothing waits for an effect to load but the drawer's own thread when the player adds one
//! (decisions.md R-FX-SAFETY): a chain restored at start is put in place at once, its effects
//! dry and loading, each loaded on a thread of its own and joining the sound when it is ready;
//! a new sample rate loads them again the same way. An effect that crashes or hangs as it
//! loads, or is given up on after crashing again and again, is turned off and saved so
//! ([`Disabled`]), and is not loaded at the next start until the player tries it again; one
//! whose saved settings crash it can be loaded without them, the settings kept
//! ([`SavedSlot::set_aside`]). A safe start puts the chain in place without loading any of it.
//!
//! **Pinned parameters** (decisions.md R-FX-PARAMS): up to [`PINS_PER_SLOT`] of an effect's
//! parameters are pinned, each to one of [`MAX_PINS`] fixed targets (the synth's `fx1` to
//! `fx16`) that stays its own while its effect moves, so that what moves it is saved with it.
//! A pin's base is the plug-in's own value (its window's edits, the drawer's knob, a learned
//! controller), saved with the slot ([`SavedPin`]) and given back after every load and restore.
//! The synth's modulation reaches a pin as timed points ([`Runner::process_modulated`]):
//! CLAP's own modulation where the parameter takes it, else base plus offset (VST3: to its
//! processor only). A pin of an effect missing, turned off or restarting does nothing, kept.

use crate::node::Node;
use crate::sandbox::{self, Fault, Instance, LoadError, Sandbox, SandboxOptions, lock};
use crate::{Mode, catalog};
use daw_ipc::MAX_BLOCK;
use daw_ipc::layout::{EV_PARAM, EV_PARAM_MOD};
use daw_ipc::protocol::{self, Op};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// The chain's length (the owner, 2026-10-18).
pub const MAX_SLOTS: usize = 4;

/// The parameters an effect pins at most (the owner, 2026-10-20: B1, "up to 4 pinned
/// parameters per effect"), and the pins' fixed targets: four an effect.
pub const PINS_PER_SLOT: usize = 4;
pub const MAX_PINS: usize = MAX_SLOTS * PINS_PER_SLOT;

/// A pin's modulation points a block sends at most: one every 32 samples, thinned past
/// 1024-sample blocks (R-FX-PARAMS).
pub const PIN_POINTS: usize = 32;

/// The saved file's format.
pub const FORMAT: u32 = 1;

/// How many changes may wait for the audio thread (while no audio device plays, they wait
/// for the next one).
const QUEUE: usize = 1024;

/// Why an effect is off and stays off from one start to the next, until the player tries it
/// again (decisions.md R-FX-SAFETY).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disabled {
    /// `crashed` (it crashed as it loaded, or was given up on after crashing again and
    /// again), `hung` (it stopped responding as it loaded, or kept doing so as it played) or
    /// `settings` (its saved settings crash it, or are thought to).
    pub why: String,
    /// How many times in a row it has been turned off (tried again and turned off again).
    #[serde(default = "one")]
    pub times: u32,
    /// When it was last turned off, in seconds since 1970 (UTC).
    #[serde(default)]
    pub at: u64,
    /// What happened, in words.
    #[serde(default)]
    pub detail: String,
}

fn one() -> u32 {
    1
}

/// A pinned parameter as the session keeps it, beside its effect's state (decisions.md
/// R-FX-PARAMS): what a preset that carries the chain carries of it too.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedPin {
    /// The plug-in's parameter ID.
    pub id: u32,
    /// Its name as the plug-in last gave it (shown while the effect is missing).
    pub name: String,
    /// Its base: its value as the player set it, in the plug-in's units (a VST3 parameter's
    /// normalized, a CLAP parameter's plain); given to the plug-in after its state.
    #[serde(default)]
    pub base: Option<f64>,
    /// Its target, 0 to [`MAX_PINS`] less 1 (the synth's `fx1` to `fx16`).
    #[serde(default)]
    pub target: Option<usize>,
    /// What moves it: each source's ID and depth (the synth's assignments, -1..1).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<(String, f32)>,
}

/// The synth's side of the pins' assignments (decisions.md R-FX-PARAMS): the chain knows its
/// pins, the synth what moves them. Read when the chain is saved, given when it is restored.
pub trait PinMods: Send + Sync {
    /// The assignments to target `target`: each source's ID and depth.
    fn get(&self, target: usize) -> Vec<(String, f32)>;
    /// These in place of target `target`'s (none: it is moved by nothing).
    fn set(&self, target: usize, mods: &[(String, f32)]);
    /// Moves whenever any pin's assignments change (the chain is saved again).
    fn revision(&self) -> u64;
}

/// Whether a player may pin a parameter: one a player could automate, not hidden, read-only,
/// a program change or the plug-in's bypass (its slot's ON tab is the bypass).
pub fn pinnable(p: &protocol::ParamInfo) -> bool {
    p.automatable && !p.hidden && !p.read_only && !p.bypass && !p.program
}

/// A pinned parameter as the drawer, the editor and the agents see it.
#[derive(Debug, Clone, PartialEq)]
pub struct PinStatus {
    /// Its target, 0 to [`MAX_PINS`] less 1.
    pub target: usize,
    /// Its effect's place now, and name.
    pub slot: usize,
    pub effect: String,
    pub id: u32,
    /// The plug-in's name for it (its last listing's, else the saved one), and short name.
    pub name: String,
    pub short: String,
    /// Its base, in the plug-in's units, and as a share of its travel (0..1).
    pub base: Option<f64>,
    pub value: f64,
    /// Its steps past its first value (0: continuous).
    pub steps: u32,
    /// Its effect runs and has it: it takes modulation now. Not: dimmed and inert, its
    /// assignments kept.
    pub live: bool,
    /// Its effect runs but no longer has it (a plug-in updated): MISSING.
    pub missing: bool,
    /// The plug-in's description of it, if its effect was listed here.
    pub info: Option<protocol::ParamInfo>,
}

/// An effect as the session keeps it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Turned off, and why: not loaded at start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<Disabled>,
    /// The saved state that crashed it, set aside when it was loaded without its settings:
    /// kept, never given to it unless the player asks, so that a later version of the plug-in
    /// may take it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_aside: Option<String>,
    /// Its pinned parameters, in the order pinned (decisions.md R-FX-PARAMS).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pins: Vec<SavedPin>,
}

/// The chain as the session keeps it: `effects.json` in the standalone's session folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// A saved chain from the bytes of its file.
    pub fn load_bytes(bytes: &[u8]) -> Result<SavedChain, String> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }

    /// Whether any of its effects would be loaded at start (one not turned off).
    pub fn has_effects(&self) -> bool {
        self.slots.iter().any(|s| s.disabled.is_none())
    }
}

/// How a saved chain is put back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restore {
    /// Each effect loaded on a thread of its own, the chain in place and dry meanwhile: the
    /// application's start, which never waits for an effect.
    Background,
    /// As `Background`, then waited for until every effect has loaded or failed (the tests,
    /// the console).
    Wait,
    /// A safe start: the chain in place, none of it loaded (each [`Health::Off`]) until the
    /// player turns the effects back on; nothing about them is saved differently.
    Off,
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
    /// Being loaded (restored at start, tried again, at a new sample rate): dry meanwhile.
    Loading,
    /// Its helper died or hung; it is being started again. Dry meanwhile.
    Restarting,
    /// Turned off after it crashed or hung ([`SlotStatus::disabled`] says why), and kept off
    /// at the next start until tried again: dry, its state kept.
    Disabled,
    /// Its plug-in could not be loaded (not installed here, say): dry, its saved state kept.
    Missing,
    /// Not loaded this time: a safe start (the effects off until the player turns them on).
    Off,
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
    /// Why it is turned off ([`Health::Disabled`]).
    pub disabled: Option<Disabled>,
    /// It has saved settings (it can be loaded without them).
    pub has_state: bool,
    /// It runs without its saved settings, which are kept ([`SavedSlot::set_aside`]).
    pub set_aside: bool,
    /// When the player's last try (its settings set aside given back) failed, if it did.
    pub try_failed: Option<Instant>,
    /// What went wrong, for a missing or failed effect.
    pub error: Option<String>,
    /// Its latency in frames, as it is in the chain now (0 bypassed).
    pub latency: u32,
    pub crashes: u32,
    pub misses: u64,
    pub helper_pid: Option<u32>,
    /// The frame of the block it was given last (the chain's own count, from 0).
    pub position: i64,
}

/// A slot on the audio thread: its slot's own number, which its pins name.
#[derive(Debug)]
pub struct Stage {
    id: u64,
    node: Option<Box<Node>>,
    bypass: Arc<AtomicBool>,
}

/// How a pin's modulation reaches its plug-in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum How {
    /// Not pinned, or its effect not listed.
    #[default]
    None,
    /// VST3: an `EV_PARAM_MOD` amount on the normalized travel, which the helper adds to
    /// the base for the processor alone.
    Normalized,
    /// CLAP, a modulatable parameter: its own modulation, an amount in its units.
    Plain,
    /// CLAP, not modulatable: base plus offset as its value (`EV_PARAM`).
    Value,
}

/// A pin as the audio thread has it.
#[derive(Debug, Clone, Copy, Default)]
struct PinRt {
    /// Its stage's ([`Stage::id`]; 0: none).
    stage: u64,
    param: u32,
    how: How,
    min: f64,
    max: f64,
    stepped: bool,
    /// Its base, a share of its travel.
    base: f64,
}

impl PinRt {
    /// Its plain value at `travel` (0..1).
    fn plain(&self, travel: f64) -> f64 {
        let t = travel.clamp(0.0, 1.0);
        match self.how {
            How::Normalized => t,
            _ => {
                let v = self.min + (self.max - self.min) * t;
                if self.stepped { v.round() } else { v }
            }
        }
    }
}

enum Msg {
    Insert {
        at: usize,
        stage: Stage,
    },
    Remove {
        at: usize,
    },
    Move {
        from: usize,
        to: usize,
    },
    Param {
        at: usize,
        id: u32,
        value: f64,
    },
    /// The node of the stage at `at` exchanged for this one's (the old goes back in it).
    Node {
        at: usize,
        stage: Stage,
    },
    /// Pin target `target` as this (R-FX-PARAMS).
    Pin {
        target: usize,
        pin: PinRt,
    },
}

/// What a slot is doing, besides what its instance says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Loaded (its instance says how it is), missing, or turned off ([`SavedSlot::disabled`]).
    Ready,
    /// A thread of its own is loading it.
    Loading,
    /// A safe start: not loaded.
    Off,
}

/// Which saved state a load gives the effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Attempt {
    /// Its saved state.
    Saved,
    /// None: its saved state set aside, kept (LOAD WITHOUT ITS SETTINGS).
    Fresh,
    /// The state set aside, the effect as it runs now kept playing meanwhile and kept if they
    /// still crash it.
    SetAside,
}

struct Slot {
    /// The slot's own number, for the threads loading it (its place may change meanwhile).
    id: u64,
    saved: SavedSlot,
    inst: Option<Arc<Instance>>,
    bypass: Arc<AtomicBool>,
    error: Option<String>,
    phase: Phase,
    try_failed: Option<Instant>,
    /// Its parameters as it last listed them (none until it is loaded here).
    params: Vec<protocol::ParamInfo>,
}

/// Which slot and parameter a pin target is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PinRef {
    slot: u64,
    id: u32,
}

struct Core {
    slots: Vec<Slot>,
    /// Each pin target's slot and parameter (R-FX-PARAMS).
    pins: [Option<PinRef>; MAX_PINS],
    /// The sample rate and largest block effects run at now, and whether an audio device
    /// has given them.
    rate: f64,
    block: usize,
}

impl Core {
    fn find(&self, id: u64) -> Option<usize> {
        self.slots.iter().position(|s| s.id == id)
    }

    /// The target slot `slot`'s parameter `id` is pinned to.
    fn target_of(&self, slot: u64, id: u32) -> Option<usize> {
        self.pins
            .iter()
            .position(|p| *p == Some(PinRef { slot, id }))
    }

    /// Pin target `target`'s slot's place, and its saved pin there.
    fn pin(&self, target: usize) -> Option<(usize, &SavedPin)> {
        let r = (*self.pins.get(target)?)?;
        let at = self.find(r.slot)?;
        let pin = self.slots[at].saved.pins.iter().find(|p| p.id == r.id)?;
        Some((at, pin))
    }

    fn pin_mut(&mut self, target: usize) -> Option<(usize, &mut SavedPin)> {
        let r = (*self.pins.get(target)?)?;
        let at = self.find(r.slot)?;
        let pin = self.slots[at]
            .saved
            .pins
            .iter_mut()
            .find(|p| p.id == r.id)?;
        Some((at, pin))
    }

    /// Pin target `target` as the audio thread is to have it.
    fn pin_rt(&self, target: usize) -> PinRt {
        let Some((at, pin)) = self.pin(target) else {
            return PinRt::default();
        };
        let slot = &self.slots[at];
        let Some(info) = slot.params.iter().find(|p| p.id == pin.id) else {
            return PinRt::default();
        };
        let how = match slot.saved.format.as_str() {
            "vst3" => How::Normalized,
            _ if info.modulatable => How::Plain,
            _ => How::Value,
        };
        PinRt {
            stage: slot.id,
            param: pin.id,
            how,
            min: info.min,
            max: info.max,
            stepped: info.stepped,
            base: pin.base.map_or(0.0, |b| info.normalized(b)),
        }
    }
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
    /// The application is closing: nothing is loaded from now on.
    closing: AtomicBool,
    next_id: AtomicU64,
    /// The synth's side of the pins' assignments, once it has given it.
    mods: Mutex<Option<Arc<dyn PinMods>>>,
    /// Moves when a pin's base, name or list changes; the revisions last saved, of it and of
    /// the assignments.
    pins_rev: AtomicU64,
    saved_pins: AtomicU64,
    saved_mods: AtomicU64,
    /// The bases a learned controller gave the pins on the audio thread (plain bits;
    /// `u64::MAX`: none), until the housekeeping takes them.
    learnt: Arc<[AtomicU64; MAX_PINS]>,
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
    /// The pins (R-FX-PARAMS), and those that were moved in the last block.
    pins: [PinRt; MAX_PINS],
    moved: u32,
    learnt: Arc<[AtomicU64; MAX_PINS]>,
    /// The tempo and place the next block begins at, for the effects (decisions.md R-LFO).
    timing: daw_ipc::Timing,
}

impl std::fmt::Debug for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner")
            .field("stages", &self.stages.len())
            .finish_non_exhaustive()
    }
}

/// Seconds since 1970, now.
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl Chain {
    /// An empty chain and its audio side. A housekeeping thread saves the chain when an
    /// effect's state changes, turns off effects given up on, and logs what happens to the
    /// effects to stderr.
    pub fn new(opts: ChainOptions) -> (Chain, Runner) {
        let (to_audio, from_control) = rtrb::RingBuffer::new(QUEUE);
        let (to_control, from_audio) = rtrb::RingBuffer::new(QUEUE);
        let shared = Arc::new(Shared {
            sandbox: Sandbox::new(opts.sandbox.clone()),
            core: Mutex::new(Core {
                slots: Vec::new(),
                pins: [None; MAX_PINS],
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
            closing: AtomicBool::new(false),
            next_id: AtomicU64::new(1),
            mods: Mutex::new(None),
            pins_rev: AtomicU64::new(0),
            saved_pins: AtomicU64::new(0),
            saved_mods: AtomicU64::new(0),
            learnt: Arc::new(std::array::from_fn(|_| AtomicU64::new(u64::MAX))),
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
            pins: [PinRt::default(); MAX_PINS],
            moved: 0,
            learnt: shared.learnt.clone(),
            timing: daw_ipc::Timing::default(),
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

    /// Adds a found effect at the chain's end; its place. The caller waits while it loads (the
    /// drawer's own thread).
    pub fn add(&self, found: &catalog::Found) -> Result<usize, String> {
        let saved = new_slot(found)?;
        self.insert(saved, None)
    }

    /// Puts a found effect in place of the one at `at` (the drawer's "replace"): the old one
    /// goes only once the new one has loaded.
    pub fn replace(&self, at: usize, found: &catalog::Found) -> Result<(), String> {
        let saved = new_slot(found)?;
        self.insert(saved, Some(at)).map(|_| ())
    }

    /// Loads an effect into a slot of its own and puts it at the end, or in place of the one
    /// at `replace`; one that does not load is an error. The chain is not held while the
    /// plug-in loads (seconds, for some): the drawer reads it meanwhile, and its other effects
    /// can be bypassed (decisions.md R-FX-DRAWER).
    fn insert(&self, saved: SavedSlot, replace: Option<usize>) -> Result<usize, String> {
        let shared = &self.shared;
        shared.drain_retired();
        let fits = |core: &Core| match replace {
            Some(at) if at >= core.slots.len() => Err(format!("there is no effect {}", at + 1)),
            None if core.slots.len() >= MAX_SLOTS => {
                Err(format!("the chain holds {MAX_SLOTS} effects at most"))
            }
            _ => Ok(()),
        };
        let bypass = Arc::new(AtomicBool::new(saved.bypass));
        let state = match saved.state.as_deref().map(protocol::unhex).transpose() {
            Ok(s) => s,
            Err(e) => return Err(format!("{}'s saved state is damaged: {e}", saved.name)),
        };
        let label = saved.name.clone();
        // Loaded at the rate the chain runs at, again if the device changed it meanwhile.
        let (mut core, inst, listed) = loop {
            let rate = {
                let core = lock(&shared.core);
                fits(&core)?;
                core.rate
            };
            let inst = shared
                .sandbox
                .load(
                    &label,
                    &saved.path,
                    &saved.plugin,
                    state.as_deref(),
                    &[],
                    rate,
                    MAX_BLOCK as u32,
                )
                .map_err(|e| e.message)?;
            // (Listed again with its values as its state left them: R-FX-PARAMS.)
            let listed = inst.list_params().ok();
            let core = lock(&shared.core);
            if let Err(e) = fits(&core) {
                shared.sandbox.unload(&inst);
                return Err(e);
            }
            if core.rate != rate {
                drop(core);
                shared.sandbox.unload(&inst);
                continue;
            }
            break (core, inst, listed);
        };
        let at = replace.unwrap_or(core.slots.len());
        let id = shared.next_id.fetch_add(1, Ordering::Relaxed);
        let stage = Stage {
            id,
            node: Some(Box::new(shared.node(&inst, core.block))),
            bypass: bypass.clone(),
        };
        let slot = Slot {
            id,
            saved,
            params: listed.unwrap_or_else(|| inst.info.params.clone()),
            inst: Some(inst),
            bypass,
            error: None,
            phase: Phase::Ready,
            try_failed: None,
        };
        let sent = {
            let mut q = lock(&shared.to_audio);
            if q.slots() < 2 {
                Err(stage)
            } else {
                if replace.is_some() {
                    let _ = q.push(Msg::Remove { at });
                }
                q.push(Msg::Insert { at, stage }).map_err(|e| match e {
                    rtrb::PushError::Full(Msg::Insert { stage, .. }) => stage,
                    rtrb::PushError::Full(_) => unreachable!("an insert was pushed"),
                })
            }
        };
        if sent.is_err() {
            if let Some(i) = &slot.inst {
                shared.sandbox.unload(i);
            }
            return Err("the audio thread is not taking changes".into());
        }
        let old = match replace {
            Some(at) => Some(std::mem::replace(&mut core.slots[at], slot)),
            None => {
                core.slots.push(slot);
                None
            }
        };
        // (The effect replaced takes its pins with it.)
        if let Some(o) = &old {
            shared.free_pins(&mut core, o.id);
        }
        drop(core);
        if let Some(i) = old.as_ref().and_then(|o| o.inst.as_ref()) {
            shared.sandbox.unload(i);
        }
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
        shared.free_pins(&mut core, slot.id);
        drop(core);
        // Unloading marks it gone first: if the audio thread is in its block now, the block
        // ends at once, dry. (One still loading is let go of by its thread, which finds it
        // gone.)
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

    /// Tries the effect at `at` again (one turned off, missing, or off for a safe start), with
    /// its saved settings, on a thread of its own: it shows loading meanwhile, and stays off
    /// (or missing) if it fails again.
    pub fn retry(&self, at: usize) -> Result<(), String> {
        self.start(at, Attempt::Saved)
    }

    /// Loads the effect at `at` without its saved settings (it starts as the plug-in starts),
    /// which are kept, set aside ([`SavedSlot::set_aside`]), on a thread of its own.
    pub fn load_without_settings(&self, at: usize) -> Result<(), String> {
        self.start(at, Attempt::Fresh)
    }

    /// Gives the effect at `at` back the settings set aside, on a thread of its own: it keeps
    /// playing as it is meanwhile, and stays so (the settings still set aside) if they still
    /// crash it.
    pub fn try_set_aside(&self, at: usize) -> Result<(), String> {
        self.start(at, Attempt::SetAside)
    }

    /// The effects off for a safe start loaded, each on a thread of its own ("TURN EFFECTS
    /// BACK ON"); those turned off for crashing stay off.
    pub fn turn_on(&self) {
        let n = self.len();
        for at in 0..n {
            let off = lock(&self.shared.core)
                .slots
                .get(at)
                .is_some_and(|s| s.phase == Phase::Off);
            if off {
                let _ = self.start(at, Attempt::Saved);
            }
        }
    }

    /// Whether the chain was put back off (a safe start) and some of it is still off.
    pub fn is_off(&self) -> bool {
        lock(&self.shared.core)
            .slots
            .iter()
            .any(|s| s.phase == Phase::Off)
    }

    /// The effect at `at` loaded on a thread of its own as `attempt` says.
    fn start(&self, at: usize, attempt: Attempt) -> Result<(), String> {
        let shared = &self.shared;
        let mut core = lock(&shared.core);
        let slot = core
            .slots
            .get_mut(at)
            .ok_or_else(|| format!("there is no effect {}", at + 1))?;
        if slot.phase == Phase::Loading {
            return Err(format!("{} is loading", slot.saved.name));
        }
        let loaded = slot
            .inst
            .as_ref()
            .is_some_and(|i| matches!(i.status(), sandbox::RUNNING | sandbox::DOWN));
        match attempt {
            Attempt::Saved | Attempt::Fresh if loaded => {
                return Err(format!("{} is loaded", slot.saved.name));
            }
            Attempt::Fresh if slot.saved.state.is_none() => {
                return Err(format!("{} has no saved settings", slot.saved.name));
            }
            Attempt::SetAside if slot.saved.set_aside.is_none() => {
                return Err(format!("{} has no settings set aside", slot.saved.name));
            }
            _ => {}
        }
        slot.phase = Phase::Loading;
        slot.try_failed = None;
        let id = slot.id;
        drop(core);
        shared.spawn_load(id, attempt);
        Ok(())
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
        self.status_of(&lock(&self.shared.core))
    }

    /// Every slot, in order, unless the chain is busy (a device's sample rate being taken):
    /// for the drawer, which never waits on it.
    pub fn try_status(&self) -> Option<Vec<SlotStatus>> {
        try_lock(&self.shared.core).map(|core| self.status_of(&core))
    }

    fn status_of(&self, core: &Core) -> Vec<SlotStatus> {
        let pipelined = self.shared.opts.mode == Mode::Pipelined;
        core.slots
            .iter()
            .map(|s| {
                let bypass = s.bypass.load(Ordering::Acquire);
                let (mut health, latency, crashes, misses, pid) = match &s.inst {
                    None => (Health::Missing, 0, 0, 0, None),
                    Some(i) => (
                        match i.status() {
                            sandbox::RUNNING => Health::Running,
                            sandbox::DOWN => Health::Restarting,
                            sandbox::FAILED => Health::Disabled,
                            _ => Health::Missing,
                        },
                        i.latency.load(Ordering::Relaxed)
                            + if pipelined { core.block as u32 } else { 0 },
                        i.crashes.load(Ordering::Relaxed),
                        i.misses.load(Ordering::Relaxed),
                        i.helper_pid(),
                    ),
                };
                // (An instance given up on is turned off by the housekeeping within a tenth
                // of a second; meanwhile it is said so already.)
                let mut disabled = s.saved.disabled.clone();
                if disabled.is_none()
                    && let Some(g) = s.inst.as_ref().and_then(|i| i.gave_up())
                {
                    disabled = Some(disabled_for(g, crashes, None, String::new()));
                }
                match s.phase {
                    Phase::Loading => health = Health::Loading,
                    Phase::Off => health = Health::Off,
                    Phase::Ready if s.saved.disabled.is_some() => health = Health::Disabled,
                    Phase::Ready => {}
                }
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
                    disabled: disabled.filter(|_| health == Health::Disabled),
                    has_state: s.saved.state.is_some() || s.inst.is_some(),
                    set_aside: s.saved.set_aside.is_some(),
                    try_failed: s.try_failed,
                    error: s.error.clone(),
                    crashes,
                    misses,
                    helper_pid: pid,
                    position: s
                        .inst
                        .as_ref()
                        .map_or(0, |i| i.position.load(Ordering::Relaxed)),
                }
            })
            .collect()
    }

    /// Waits until no effect is loading, for at most `timeout`: whether none is.
    pub fn wait_loaded(&self, timeout: Duration) -> bool {
        let end = Instant::now() + timeout;
        loop {
            let loading = lock(&self.shared.core)
                .slots
                .iter()
                .any(|s| s.phase == Phase::Loading);
            if !loading {
                return true;
            }
            if Instant::now() >= end {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The parameters of the effect at `at`, as it listed them when loaded (none if it is
    /// missing).
    pub fn params(&self, at: usize) -> Vec<protocol::ParamInfo> {
        lock(&self.shared.core)
            .slots
            .get(at)
            .and_then(|s| s.inst.as_ref())
            .map(|i| i.info.params.clone())
            .unwrap_or_default()
    }

    /// The parameters of the effect at `at` as it lists them now, with their values, asked of
    /// its helper (decisions.md R-FX-PARAMS); the plug-in's own order. Its pins follow the
    /// listing's names. An effect not running has none to give.
    pub fn list_params(&self, at: usize) -> Result<Vec<protocol::ParamInfo>, String> {
        let (inst, slot) = {
            let core = lock(&self.shared.core);
            let s = core
                .slots
                .get(at)
                .ok_or_else(|| format!("there is no effect {}", at + 1))?;
            (s.inst.clone().ok_or("the effect is not loaded")?, s.id)
        };
        let params = inst.list_params()?;
        let mut core = lock(&self.shared.core);
        if let Some(i) = core.find(slot) {
            core.slots[i].params = params.clone();
            self.shared.refresh_pins(&mut core, slot);
        }
        Ok(params)
    }

    /// The effect at `at`'s own text for each (parameter, value in its units) pair: `None`
    /// where it gave none (R-FX-PARAMS).
    pub fn param_text(
        &self,
        at: usize,
        values: &[(u32, f64)],
    ) -> Result<Vec<Option<String>>, String> {
        let (inst, _) = self.running(at)?;
        inst.param_text(values)
    }

    /// Pins parameter `id` of the effect at `at` (R-FX-PARAMS): its target (0 to
    /// [`MAX_PINS`] less 1), on which the synth's sources can move it. Its base is its value
    /// now. One the effect does not list as a player's ([`pinnable`]), one pinned already, a
    /// fifth of an effect, or one of an effect not loaded, is refused.
    pub fn pin(&self, at: usize, id: u32) -> Result<usize, String> {
        let shared = &self.shared;
        let listed = self.list_params(at).ok();
        let mut core = lock(&shared.core);
        let slot = core
            .slots
            .get(at)
            .ok_or_else(|| format!("there is no effect {}", at + 1))?;
        let params = listed.as_deref().unwrap_or(&slot.params);
        let info = params
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| format!("{} has no parameter {id}", slot.saved.name))?;
        if !pinnable(&info) {
            return Err(format!(
                "{}'s {} cannot be pinned",
                slot.saved.name, info.name
            ));
        }
        if slot.saved.pins.iter().any(|p| p.id == id) {
            return Err(format!(
                "{}'s {} is pinned already",
                slot.saved.name, info.name
            ));
        }
        if slot.saved.pins.len() >= PINS_PER_SLOT {
            return Err(format!(
                "{} has {PINS_PER_SLOT} parameters pinned: unpin one first",
                slot.saved.name
            ));
        }
        let target = core
            .pins
            .iter()
            .position(Option::is_none)
            .ok_or("every pin is taken")?;
        let slot_id = slot.id;
        core.slots[at].saved.pins.push(SavedPin {
            id,
            name: info.name.clone(),
            base: info.value,
            target: Some(target),
            mods: Vec::new(),
        });
        core.pins[target] = Some(PinRef { slot: slot_id, id });
        if let (Some(inst), Some(v)) = (&core.slots[at].inst, info.value) {
            inst.record_value(id, v);
        }
        shared.send_pin(&core, target);
        if let Some(m) = lock(&shared.mods).clone() {
            m.set(target, &[]);
        }
        shared.pins_rev.fetch_add(1, Ordering::AcqRel);
        shared.save_in(&core);
        Ok(target)
    }

    /// Pin target `target` unpinned: what moved it let go of, its parameter left at its base.
    pub fn unpin(&self, target: usize) -> Result<(), String> {
        let shared = &self.shared;
        let mut core = lock(&shared.core);
        let r = core
            .pins
            .get(target)
            .copied()
            .flatten()
            .ok_or_else(|| format!("nothing is pinned to fx{}", target + 1))?;
        if let Some(at) = core.find(r.slot) {
            core.slots[at].saved.pins.retain(|p| p.id != r.id);
        }
        core.pins[target] = None;
        shared.send_pin(&core, target);
        if let Some(m) = lock(&shared.mods).clone() {
            m.set(target, &[]);
        }
        shared.pins_rev.fetch_add(1, Ordering::AcqRel);
        shared.save_in(&core);
        Ok(())
    }

    /// Every pin, by target (R-FX-PARAMS).
    pub fn pins(&self) -> Vec<PinStatus> {
        self.pins_of(&lock(&self.shared.core))
    }

    /// Every pin, unless the chain is busy: for the drawer, which never waits on it.
    pub fn try_pins(&self) -> Option<Vec<PinStatus>> {
        try_lock(&self.shared.core).map(|core| self.pins_of(&core))
    }

    fn pins_of(&self, core: &Core) -> Vec<PinStatus> {
        (0..MAX_PINS)
            .filter_map(|t| {
                let (at, pin) = core.pin(t)?;
                let s = &core.slots[at];
                let info = s.params.iter().find(|p| p.id == pin.id).cloned();
                let running = s.phase == Phase::Ready
                    && s.saved.disabled.is_none()
                    && s.inst
                        .as_ref()
                        .is_some_and(|i| i.status() == sandbox::RUNNING);
                let listed = !s.params.is_empty();
                Some(PinStatus {
                    target: t,
                    slot: at,
                    effect: s.saved.name.clone(),
                    id: pin.id,
                    name: info.as_ref().map_or(pin.name.clone(), |i| i.name.clone()),
                    short: info.as_ref().map_or(String::new(), |i| i.short.clone()),
                    base: pin.base,
                    value: match (&info, pin.base) {
                        (Some(i), Some(b)) => i.normalized(b),
                        _ => 0.0,
                    },
                    steps: info.as_ref().map_or(0, |i| i.steps),
                    live: running && info.is_some(),
                    missing: running && listed && info.is_none(),
                    info,
                })
            })
            .collect()
    }

    /// Pin target `target`'s base set to `travel` (0..1 of its travel: the drawer's knob):
    /// its plug-in's value from the next block, its window following (R-FX-PARAMS).
    pub fn set_pin_value(&self, target: usize, travel: f64) -> Result<(), String> {
        let shared = &self.shared;
        let mut core = lock(&shared.core);
        let rt = core.pin_rt(target);
        let (at, id) = match core.pin(target) {
            Some((at, pin)) if rt.how != How::None => (at, pin.id),
            Some(_) => return Err("the effect is not loaded".into()),
            None => return Err(format!("nothing is pinned to fx{}", target + 1)),
        };
        let plain = rt.plain(travel);
        if let Some((_, pin)) = core.pin_mut(target) {
            pin.base = Some(plain);
        }
        lock(&shared.to_audio)
            .push(Msg::Param {
                at,
                id,
                value: plain,
            })
            .map_err(|_| "the audio thread is not taking changes")?;
        shared.send_pin(&core, target);
        shared.pins_rev.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    /// The synth's side of the pins' assignments (R-FX-PARAMS): given what moves each pin
    /// now (a chain restored before the synth was made gives it then), read when saved.
    pub fn set_pin_mods(&self, mods: Arc<dyn PinMods>) {
        let core = lock(&self.shared.core);
        for t in 0..MAX_PINS {
            let saved = core.pin(t).map(|(_, p)| p.mods.clone()).unwrap_or_default();
            mods.set(t, &saved);
        }
        self.shared
            .saved_mods
            .store(mods.revision(), Ordering::Release);
        *lock(&self.shared.mods) = Some(mods);
    }

    /// Whether any effect is playing (loaded, not bypassed): the FX key's light.
    pub fn active(&self) -> bool {
        active(&lock(&self.shared.core))
    }

    /// Whether any effect is playing, unless the chain is busy: for the drawer, which never
    /// waits on it.
    pub fn try_active(&self) -> Option<bool> {
        try_lock(&self.shared.core).map(|core| active(&core))
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

    /// Replaces the chain with a saved one and waits for it to load ([`Restore::Wait`]); an
    /// error is said for each effect that would not load, and the rest load.
    pub fn restore(&self, chain: &SavedChain) -> Vec<String> {
        self.restore_as(chain, Restore::Wait)
    }

    /// Replaces the chain with a saved one, as `how` says. Effects turned off stay off (not
    /// loaded); the others are loaded on threads of their own, or (a safe start) not. What
    /// could not be put back is said (and, waiting, what would not load).
    pub fn restore_as(&self, chain: &SavedChain, how: Restore) -> Vec<String> {
        while !self.is_empty() {
            let _ = self.remove(self.len() - 1);
        }
        let mut errors = Vec::new();
        let mut started = Vec::new();
        {
            let shared = &self.shared;
            let mut core = lock(&shared.core);
            let mut q = lock(&shared.to_audio);
            for s in chain.slots.iter().take(MAX_SLOTS) {
                let bypass = Arc::new(AtomicBool::new(s.bypass));
                let id = shared.next_id.fetch_add(1, Ordering::Relaxed);
                let stage = Stage {
                    id,
                    node: None,
                    bypass: bypass.clone(),
                };
                let at = core.slots.len();
                if q.push(Msg::Insert { at, stage }).is_err() {
                    errors.push(format!(
                        "{}: the audio thread is not taking changes",
                        s.name
                    ));
                    continue;
                }
                let phase = match how {
                    _ if s.disabled.is_some() => Phase::Ready,
                    Restore::Off => Phase::Off,
                    Restore::Background | Restore::Wait => {
                        started.push(id);
                        Phase::Loading
                    }
                };
                core.slots.push(Slot {
                    id,
                    saved: s.clone(),
                    inst: None,
                    bypass,
                    error: s
                        .disabled
                        .as_ref()
                        .map(|d| format!("turned off: {}", d.detail)),
                    phase,
                    try_failed: None,
                    params: Vec::new(),
                });
            }
            drop(q);
            // Its pins on the targets they had, where they are free (R-FX-PARAMS).
            shared.bind_pins(&mut core);
        }
        for id in started {
            self.shared.spawn_load(id, Attempt::Saved);
        }
        if chain.slots.len() > MAX_SLOTS {
            errors.push(format!(
                "{} effects past the {MAX_SLOTS}th were left out",
                chain.slots.len() - MAX_SLOTS
            ));
        }
        if how == Restore::Wait {
            let wait = self.shared.opts.sandbox.load_timeout * 2 + Duration::from_secs(10);
            if !self.wait_loaded(wait) {
                errors.push("the effects were still loading".into());
            }
            for s in &self.status() {
                if let (Some(e), Health::Missing | Health::Disabled) = (&s.error, s.health) {
                    errors.push(format!("{}: {e}", s.name));
                }
            }
        }
        errors
    }

    /// Restores the chain from its file, if it has one, and waits for it to load; the errors,
    /// as [`Chain::restore`].
    pub fn restore_file(&self) -> Vec<String> {
        self.restore_file_as(Restore::Wait)
    }

    /// Restores the chain from its file, if it has one, as `how` says ([`Chain::restore_as`]).
    pub fn restore_file_as(&self, how: Restore) -> Vec<String> {
        let Some(path) = self.shared.opts.file.clone() else {
            return Vec::new();
        };
        match SavedChain::load(&path) {
            Ok(Some(c)) => {
                // Saving while restoring would write a shorter chain over the file.
                self.shared.quiet.store(true, Ordering::Release);
                let errors = self.restore_as(&c, how);
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
    /// every helper, those still loading an effect too (killed: one hung in a plug-in must
    /// not outlive the application). The audio thread must have stopped.
    pub fn shutdown(&self) -> Result<(), String> {
        let saved = self.save_now();
        self.shared.quiet.store(true, Ordering::Release);
        self.shared.closing.store(true, Ordering::Release);
        let slots: Vec<Slot> = lock(&self.shared.core).slots.drain(..).collect();
        for s in slots {
            if let Some(i) = &s.inst {
                self.shared.sandbox.unload(i);
            }
        }
        self.shared.sandbox.close();
        saved
    }
}

/// `m` locked, unless another holds it (a poisoned lock taken as it is, as [`lock`] takes it).
fn try_lock<T>(m: &Mutex<T>) -> Option<std::sync::MutexGuard<'_, T>> {
    match m.try_lock() {
        Ok(g) => Some(g),
        Err(std::sync::TryLockError::Poisoned(p)) => Some(p.into_inner()),
        Err(std::sync::TryLockError::WouldBlock) => None,
    }
}

/// Whether any effect in `core` is playing (loaded, not bypassed, not given up on).
fn active(core: &Core) -> bool {
    core.slots.iter().any(|s| {
        !s.bypass.load(Ordering::Acquire)
            && s.saved.disabled.is_none()
            && s.phase != Phase::Off
            && s.inst
                .as_ref()
                .is_some_and(|i| matches!(i.status(), sandbox::RUNNING | sandbox::DOWN))
    })
}

/// A found effect as a new slot: an instrument refused.
fn new_slot(found: &catalog::Found) -> Result<SavedSlot, String> {
    if found.kind == catalog::Kind::Instrument {
        return Err(format!(
            "{} is an instrument: only effects are hosted",
            found.desc.name
        ));
    }
    Ok(SavedSlot {
        format: found.desc.format.clone(),
        path: found.path.clone(),
        plugin: found.desc.id.clone(),
        name: found.desc.name.clone(),
        vendor: found.desc.vendor.clone(),
        bypass: false,
        state: None,
        disabled: None,
        set_aside: None,
        pins: Vec::new(),
    })
}

/// The record of an effect turned off after `g`, `crashes` crashes, following `before` (the
/// record it had, if it was tried again).
fn disabled_for(
    g: sandbox::GaveUp,
    crashes: u32,
    before: Option<&Disabled>,
    detail: String,
) -> Disabled {
    let why = if g.settings {
        "settings"
    } else if g.hung {
        "hung"
    } else {
        "crashed"
    };
    Disabled {
        why: why.into(),
        times: before.map_or(1, |d| d.times.saturating_add(1)),
        at: now_secs(),
        detail: if detail.is_empty() {
            format!("given up on after {crashes} crashes")
        } else {
            detail
        },
    }
}

/// The record of an effect turned off because it crashed or hung as it loaded (`e`).
fn disabled_by_load(e: &LoadError, before: Option<&Disabled>) -> Disabled {
    let why = match (e.fault, e.during) {
        (_, sandbox::During::Settings) => "settings",
        (Fault::Hung, _) => "hung",
        _ => "crashed",
    };
    Disabled {
        why: why.into(),
        times: before.map_or(1, |d| d.times.saturating_add(1)),
        at: now_secs(),
        detail: e.message.clone(),
    }
}

impl Shared {
    /// Pin target `target` sent to the audio thread as `core` has it.
    fn send_pin(&self, core: &Core, target: usize) {
        let pin = core.pin_rt(target);
        let _ = lock(&self.to_audio).push(Msg::Pin { target, pin });
    }

    /// Slot `slot`'s pins let go of (it left the chain): their targets free, moved by nothing.
    fn free_pins(&self, core: &mut Core, slot: u64) {
        let mods = lock(&self.mods).clone();
        for t in 0..MAX_PINS {
            if core.pins[t].is_some_and(|p| p.slot == slot) {
                core.pins[t] = None;
                self.send_pin(core, t);
                if let Some(m) = &mods {
                    m.set(t, &[]);
                }
            }
        }
        self.pins_rev.fetch_add(1, Ordering::AcqRel);
    }

    /// Every slot's saved pins put on targets: each on the one it had, if free, else the first
    /// free; what moved each given to the synth. Targets with no pin are moved by nothing.
    fn bind_pins(&self, core: &mut Core) {
        let mut wanted: Vec<(u64, u32, Option<usize>)> = Vec::new();
        for s in &core.slots {
            for p in s.saved.pins.iter().take(PINS_PER_SLOT) {
                wanted.push((s.id, p.id, p.target.filter(|t| *t < MAX_PINS)));
            }
        }
        core.pins = [None; MAX_PINS];
        let mut later = Vec::new();
        for (slot, id, target) in wanted {
            match target {
                Some(t) if core.pins[t].is_none() => core.pins[t] = Some(PinRef { slot, id }),
                _ => later.push(PinRef { slot, id }),
            }
        }
        for r in later {
            if let Some(t) = core.pins.iter().position(Option::is_none) {
                core.pins[t] = Some(r);
            }
        }
        let mods = lock(&self.mods).clone();
        for t in 0..MAX_PINS {
            self.send_pin(core, t);
            if let Some(m) = &mods {
                let saved = core.pin(t).map(|(_, p)| p.mods.clone()).unwrap_or_default();
                m.set(t, &saved);
            }
        }
        self.pins_rev.fetch_add(1, Ordering::AcqRel);
    }

    /// Slot `slot`'s pins after it was listed: names from its listing, a base where none was
    /// known (its value now), each sent to the audio thread.
    fn refresh_pins(&self, core: &mut Core, slot: u64) {
        let Some(at) = core.find(slot) else { return };
        let s = &mut core.slots[at];
        for pin in &mut s.saved.pins {
            if let Some(info) = s.params.iter().find(|p| p.id == pin.id) {
                if !info.name.is_empty() {
                    pin.name = info.name.clone();
                }
                if pin.base.is_none() {
                    pin.base = info.value;
                }
            }
        }
        for t in 0..MAX_PINS {
            if core.pins[t].is_some_and(|p| p.slot == slot) {
                self.send_pin(core, t);
            }
        }
        self.pins_rev.fetch_add(1, Ordering::AcqRel);
    }

    /// The pins followed (the housekeeping's): the values the effects' windows gave pinned
    /// parameters and the learned controllers' become their bases (given again after a
    /// restore), effects whose parameters changed are listed again, and the chain is saved
    /// when its pins or their assignments changed, at most once a second.
    fn follow_pins(&self, may_save: bool) -> bool {
        let mut relist = Vec::new();
        {
            let mut core = lock(&self.core);
            let mut changed = Vec::new();
            for t in 0..MAX_PINS {
                let bits = self.learnt[t].swap(u64::MAX, Ordering::AcqRel);
                if bits != u64::MAX
                    && let Some((_, pin)) = core.pin_mut(t)
                {
                    pin.base = Some(f64::from_bits(bits));
                    changed.push(t);
                }
            }
            for at in 0..core.slots.len() {
                let Some(inst) = core.slots[at].inst.clone() else {
                    continue;
                };
                if inst.take_relist() {
                    relist.push((core.slots[at].id, inst.clone()));
                }
                let slot = core.slots[at].id;
                for (id, v) in inst.take_edits() {
                    let Some(t) = core.target_of(slot, id) else {
                        continue;
                    };
                    if let Some(pin) = core.slots[at].saved.pins.iter_mut().find(|p| p.id == id) {
                        pin.base = Some(v);
                        inst.record_value(id, v);
                        changed.push(t);
                    }
                }
            }
            for t in changed {
                self.send_pin(&core, t);
                self.pins_rev.fetch_add(1, Ordering::AcqRel);
            }
        }
        for (slot, inst) in relist {
            if let Ok(params) = inst.list_params() {
                let mut core = lock(&self.core);
                if let Some(at) = core.find(slot) {
                    core.slots[at].params = params;
                    self.refresh_pins(&mut core, slot);
                }
            }
        }
        let pins = self.pins_rev.load(Ordering::Acquire);
        let mods = lock(&self.mods).as_ref().map_or(0, |m| m.revision());
        let due = pins != self.saved_pins.load(Ordering::Acquire)
            || mods != self.saved_mods.load(Ordering::Acquire);
        if due && may_save {
            self.saved_pins.store(pins, Ordering::Release);
            self.saved_mods.store(mods, Ordering::Release);
            self.save();
            return true;
        }
        false
    }

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

    /// Slot `id` loaded on a thread of its own as `attempt` says (its phase already
    /// [`Phase::Loading`]).
    fn spawn_load(self: &Arc<Self>, id: u64, attempt: Attempt) {
        let shared = self.clone();
        let spawned = std::thread::Builder::new()
            .name("ca72-fx-load".into())
            .spawn(move || shared.load_slot(id, attempt));
        if spawned.is_err() {
            let mut core = lock(&self.core);
            if let Some(at) = core.find(id) {
                core.slots[at].phase = Phase::Ready;
                core.slots[at].error = Some("no thread could be started to load it".into());
            }
        }
    }

    /// Loads slot `id` (on its own thread): at the rate the chain runs at (again, if that
    /// changed meanwhile), then puts its node in its stage, where it joins the sound. One that
    /// crashes or hangs as it loads is turned off and saved so; one that would not load for
    /// another reason is missing, as before.
    fn load_slot(self: &Arc<Self>, id: u64, attempt: Attempt) {
        loop {
            let (saved, rate) = {
                let core = lock(&self.core);
                let Some(at) = core.find(id) else { return };
                (core.slots[at].saved.clone(), core.rate)
            };
            if self.closing.load(Ordering::Acquire) {
                return;
            }
            let hex = match attempt {
                Attempt::Saved => saved.state.clone(),
                Attempt::Fresh => None,
                Attempt::SetAside => saved.set_aside.clone(),
            };
            // Its pins' bases given after its state (R-FX-PARAMS: the state may hold a
            // modulated instant).
            let bases: Vec<(u32, f64)> = saved
                .pins
                .iter()
                .filter_map(|p| p.base.map(|b| (p.id, b)))
                .collect();
            let result = match hex.as_deref().map(protocol::unhex).transpose() {
                Ok(state) => self.sandbox.load(
                    &saved.name,
                    &saved.path,
                    &saved.plugin,
                    state.as_deref(),
                    &bases,
                    rate,
                    MAX_BLOCK as u32,
                ),
                Err(e) => Err(LoadError {
                    fault: Fault::Refused,
                    during: sandbox::During::Settings,
                    message: format!("its saved state is damaged: {e}"),
                }),
            };
            let listed = result.as_ref().ok().and_then(|i| i.list_params().ok());
            let mut core = lock(&self.core);
            let at = core.find(id);
            let closing = self.closing.load(Ordering::Acquire);
            let (Some(at), false) = (at, closing) else {
                drop(core);
                if let Ok(i) = &result {
                    self.sandbox.unload(i);
                }
                return;
            };
            if result.is_ok() && core.rate != rate {
                drop(core);
                if let Ok(i) = &result {
                    self.sandbox.unload(i);
                }
                continue;
            }
            let (block, now_rate) = (core.block, core.rate);
            let slot = &mut core.slots[at];
            let mut retired = None;
            match result {
                Ok(inst) => {
                    let stage = Stage {
                        id: slot.id,
                        node: Some(Box::new(self.node(&inst, block))),
                        bypass: slot.bypass.clone(),
                    };
                    if lock(&self.to_audio).push(Msg::Node { at, stage }).is_err() {
                        slot.phase = Phase::Ready;
                        slot.error = Some("the audio thread is not taking changes".into());
                        drop(core);
                        self.sandbox.unload(&inst);
                        return;
                    }
                    match attempt {
                        Attempt::Saved => {}
                        Attempt::Fresh => {
                            // The settings that crashed it kept (the first set aside, if it
                            // was loaded without them before).
                            if slot.saved.set_aside.is_none() {
                                slot.saved.set_aside = slot.saved.state.take();
                            }
                            slot.saved.state = None;
                        }
                        Attempt::SetAside => slot.saved.state = slot.saved.set_aside.take(),
                    }
                    if slot.saved.disabled.take().is_some() {
                        self.sandbox.event(
                            &saved.name,
                            "restored",
                            "tried again: it loaded".into(),
                        );
                    }
                    slot.params = listed.unwrap_or_else(|| inst.info.params.clone());
                    retired = slot.inst.replace(inst);
                    slot.error = None;
                    slot.phase = Phase::Ready;
                }
                Err(e) if attempt == Attempt::SetAside => {
                    // Still crashed by them: it plays on as it was, the settings still aside.
                    self.sandbox.event(
                        &saved.name,
                        "warning",
                        format!("its settings set aside would not load: {e}"),
                    );
                    slot.try_failed = Some(Instant::now());
                    slot.phase = Phase::Ready;
                }
                Err(e) if e.fault == Fault::Refused => {
                    self.sandbox
                        .event(&saved.name, "warning", format!("not loaded: {e}"));
                    slot.error = Some(e.message);
                    slot.phase = Phase::Ready;
                }
                Err(e) => {
                    let d = disabled_by_load(&e, slot.saved.disabled.as_ref());
                    self.sandbox.event(
                        &saved.name,
                        "disabled",
                        format!("turned off ({}): {e}", d.why),
                    );
                    // (Loaded without its settings, it crashed all the same: they were not
                    // the cause, and stay its own.)
                    slot.saved.disabled = Some(d);
                    slot.error = Some(e.message);
                    slot.phase = Phase::Ready;
                }
            }
            // A try of its settings set aside that failed while the device changed its rate:
            // the effect it plays on with is loaded again at the new one.
            let again = slot
                .inst
                .clone()
                .filter(|i| slot.phase == Phase::Ready && i.sample_rate != now_rate);
            if again.is_some() {
                slot.phase = Phase::Loading;
            }
            self.refresh_pins(&mut core, id);
            self.save_in(&core);
            drop(core);
            if let Some(old) = retired {
                self.sandbox.unload(&old);
            }
            if let Some(old) = again {
                self.spawn_reload(id, old);
            }
            return;
        }
    }

    /// Slot `id`'s effect, `old`, loaded again at the chain's new sample rate on a thread of its
    /// own (`Runner::prepare`: its node is gone meanwhile, dry).
    fn spawn_reload(self: &Arc<Self>, id: u64, old: Arc<Instance>) {
        let shared = self.clone();
        let spawned = std::thread::Builder::new()
            .name("ca72-fx-reload".into())
            .spawn(move || shared.reload_slot(id, old));
        if spawned.is_err() {
            let mut core = lock(&self.core);
            if let Some(at) = core.find(id) {
                core.slots[at].phase = Phase::Ready;
            }
        }
    }

    fn reload_slot(&self, id: u64, mut old: Arc<Instance>) {
        loop {
            let rate = lock(&self.core).rate;
            if self.closing.load(Ordering::Acquire) {
                return;
            }
            let result = self.sandbox.reload(&old, rate, MAX_BLOCK as u32);
            let listed = result.as_ref().ok().and_then(|i| i.list_params().ok());
            let mut core = lock(&self.core);
            let Some(at) = core
                .find(id)
                .filter(|_| !self.closing.load(Ordering::Acquire))
            else {
                drop(core);
                if let Ok(i) = &result {
                    self.sandbox.unload(i);
                }
                return;
            };
            if let Ok(new) = &result
                && core.rate != rate
            {
                // Loaded again at the rate it is to run at now.
                core.slots[at].inst = Some(new.clone());
                old = new.clone();
                continue;
            }
            let block = core.block;
            let slot = &mut core.slots[at];
            slot.phase = Phase::Ready;
            let mut failed = false;
            match result {
                Ok(inst) => {
                    let stage = Stage {
                        id: slot.id,
                        node: Some(Box::new(self.node(&inst, block))),
                        bypass: slot.bypass.clone(),
                    };
                    if let Some(l) = listed {
                        slot.params = l;
                    }
                    slot.inst = Some(inst);
                    let _ = lock(&self.to_audio).push(Msg::Node { at, stage });
                }
                Err(e) => {
                    self.sandbox.event(
                        &slot.saved.name,
                        "failed",
                        format!("could not be loaded again at {rate} Hz: {e}"),
                    );
                    slot.saved.state = old.snapshot().map(|b| protocol::hex(&b));
                    if e.fault != Fault::Refused {
                        slot.saved.disabled = Some(disabled_by_load(&e, None));
                    }
                    slot.error = Some(e.message);
                    slot.inst = None;
                    failed = true;
                }
            }
            self.refresh_pins(&mut core, id);
            self.save_in(&core);
            drop(core);
            if failed {
                self.sandbox.unload(&old);
            }
            return;
        }
    }

    /// Effects given up on (crashed again and again, or in their restore) turned off: their
    /// latest state kept, the record saved, their nodes let go of.
    fn turn_off_given_up(&self) {
        let mut gone = Vec::new();
        {
            let mut core = lock(&self.core);
            let mut q = lock(&self.to_audio);
            for (at, slot) in core.slots.iter_mut().enumerate() {
                if slot.phase != Phase::Ready || slot.saved.disabled.is_some() {
                    continue;
                }
                let Some(inst) = &slot.inst else { continue };
                let Some(g) = inst.gave_up().filter(|_| inst.status() == sandbox::FAILED) else {
                    continue;
                };
                let crashes = inst.crashes.load(Ordering::Relaxed);
                let detail = self
                    .sandbox
                    .events_from(0)
                    .0
                    .iter()
                    .rev()
                    .find(|e| e.effect == slot.saved.name && e.kind == "failed")
                    .map(|e| e.detail.clone())
                    .unwrap_or_default();
                let d = disabled_for(g, crashes, None, detail);
                self.sandbox.event(
                    &slot.saved.name,
                    "disabled",
                    format!("turned off ({}) until it is tried again", d.why),
                );
                if let Some(s) = inst.snapshot() {
                    slot.saved.state = Some(protocol::hex(&s));
                }
                slot.saved.disabled = Some(d);
                slot.error = Some("turned off after it crashed".into());
                let _ = q.push(Msg::Node {
                    at,
                    stage: Stage {
                        id: slot.id,
                        node: None,
                        bypass: slot.bypass.clone(),
                    },
                });
                gone.extend(slot.inst.take());
            }
            drop(q);
            if !gone.is_empty() {
                self.save_in(&core);
            }
        }
        for i in &gone {
            self.sandbox.unload(i);
        }
    }

    fn saved(&self) -> SavedChain {
        self.saved_of(&lock(&self.core))
    }

    fn saved_of(&self, core: &Core) -> SavedChain {
        let mods = lock(&self.mods).clone();
        SavedChain {
            format: FORMAT,
            slots: core
                .slots
                .iter()
                .map(|s| SavedSlot {
                    // Each pin with its target and what moves it now (R-FX-PARAMS).
                    pins: s
                        .saved
                        .pins
                        .iter()
                        .map(|p| {
                            let target = core.target_of(s.id, p.id);
                            SavedPin {
                                target,
                                mods: match (&mods, target) {
                                    (Some(m), Some(t)) => m.get(t),
                                    _ => p.mods.clone(),
                                },
                                ..p.clone()
                            }
                        })
                        .collect(),
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
        self.save_in(&lock(&self.core));
    }

    /// Saves the chain `core` holds, as [`Shared::save`], with the lock still held: the file
    /// follows the chain in step. What anyone sees of the chain (an effect turned off, above
    /// all) is on disk before the lock is let go, and saves land in the order their chains
    /// were taken, an older one never written over a newer (on Windows, where a write and its
    /// flush take milliseconds, an effect was shown turned off before its record was
    /// written). Nothing on the audio thread takes the lock.
    fn save_in(&self, core: &Core) {
        if self.quiet.load(Ordering::Acquire) || self.keep_file.load(Ordering::Acquire) {
            return;
        }
        if let Err(e) = self.write_in(core) {
            eprintln!("CA-72 effects: {e}");
        }
    }

    fn write(&self) -> Result<(), String> {
        self.write_in(&lock(&self.core))
    }

    fn write_in(&self, core: &Core) -> Result<(), String> {
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
        let bytes = serde_json::to_vec_pretty(&self.saved_of(core)).map_err(|e| e.to_string())?;
        crate::write_atomically(path, &bytes)
            .map_err(|e| format!("the effects could not be saved in {}: {e}", path.display()))?;
        self.saved_rev.store(rev, Ordering::Release);
        Ok(())
    }
}

/// Saves the chain when an effect's state changed, turns off effects given up on, drops
/// what the audio thread let go of, and logs the effects' events.
fn housekeeping(shared: &Weak<Shared>) {
    let mut last_save = Instant::now();
    loop {
        std::thread::sleep(Duration::from_millis(100));
        let Some(s) = shared.upgrade() else { return };
        s.drain_retired();
        if !s.closing.load(Ordering::Acquire) {
            s.turn_off_given_up();
        }
        let from = *lock(&s.logged);
        let (events, n) = s.sandbox.events_from(from);
        for e in events {
            eprintln!("CA-72 effects: {} {}: {}", e.effect, e.kind, e.detail);
        }
        *lock(&s.logged) = n;
        // At most a save a second: a plug-in whose state changes at every change of a knob.
        let may_save = last_save.elapsed() >= Duration::from_secs(1);
        if s.sandbox.state_rev() != s.saved_rev.load(Ordering::Acquire) && may_save {
            s.save();
            last_save = Instant::now();
        } else if !s.closing.load(Ordering::Acquire) && s.follow_pins(may_save) {
            last_save = Instant::now();
        }
    }
}

impl Runner {
    /// Before the audio device plays (on a control thread, no audio thread running): the
    /// device's sample rate and largest block. Changes the chain made meanwhile are taken; at
    /// another sample rate every effect is loaded again in its helper with its latest state,
    /// each on a thread of its own, dry until it is (nothing here waits for a plug-in:
    /// decisions.md R-FX-SAFETY); every node is made again for the block size.
    pub fn prepare(&mut self, rate: f64, block: usize) {
        self.take_changes();
        let shared = self.shared.clone();
        let mut core = lock(&shared.core);
        let block = block.clamp(1, MAX_BLOCK);
        let reload = rate != core.rate;
        core.rate = rate;
        core.block = block;
        let mut reloads = Vec::new();
        for (slot, stage) in core.slots.iter_mut().zip(self.stages.iter_mut()) {
            if slot.phase != Phase::Ready {
                // (Loading: its thread sees the new rate and loads it again at it. One that
                // plays on while its settings set aside are tried keeps playing at the same
                // rate, and is loaded again after the try at another.)
                stage.node = slot
                    .inst
                    .as_ref()
                    .filter(|i| i.sample_rate == rate && i.status() == sandbox::RUNNING)
                    .map(|i| Box::new(shared.node(i, block)));
                continue;
            }
            if reload
                && let Some(old) = slot.inst.clone()
                && matches!(old.status(), sandbox::RUNNING | sandbox::DOWN)
            {
                slot.phase = Phase::Loading;
                stage.node = None;
                reloads.push((slot.id, old));
                continue;
            }
            stage.node = slot.inst.as_ref().map(|i| Box::new(shared.node(i, block)));
        }
        drop(core);
        for (id, old) in reloads {
            shared.spawn_reload(id, old);
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
                        self.retire(stage);
                    }
                }
                Msg::Remove { at } => {
                    if at < self.stages.len() {
                        let stage = self.stages.remove(at);
                        self.retire(stage);
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
                Msg::Node { at, mut stage } => {
                    if let Some(s) = self.stages.get_mut(at) {
                        std::mem::swap(&mut s.node, &mut stage.node);
                    }
                    // (It holds the old node now, or the new one that found no stage.)
                    self.retire(stage);
                }
                Msg::Pin { target, pin } => {
                    if let Some(p) = self.pins.get_mut(target) {
                        *p = pin;
                    }
                }
            }
        }
    }

    /// `stage` handed back to be dropped off the audio thread.
    fn retire(&mut self, stage: Stage) {
        if let Err(rtrb::PushError::Full(stage)) = self.to_control.push(stage) {
            // Never dropped here: kept to the end rather (the queue holds a thousand).
            std::mem::forget(stage);
        }
    }

    /// Where the music is as the next block begins (the standalone's tempo, as its clock has
    /// it: decisions.md R-LFO), given to each effect with its blocks as a host's stopped
    /// transport; none until told.
    pub fn set_timing(&mut self, timing: daw_ipc::Timing) {
        self.timing = timing;
    }

    /// The synth's output through the chain, in place (audio thread). In real time each
    /// effect's block is waited for until `deadline` at the latest; past it, and for any
    /// effect not running, its input goes on untouched.
    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], deadline: Option<Instant>) {
        self.process_modulated(left, right, deadline, 0, &[], &[]);
    }

    /// As [`Runner::process`], the pins moved by the synth's sources (decisions.md
    /// R-FX-PARAMS): `moved`, a bit for each pin target that some source moves; `at`, the
    /// block's frames where the sources were read (each step of 32 samples); `offsets`, each
    /// pin's offset there on its travel (the sum of depth × source). A pin moved goes to its
    /// effect as one point for each step, at most [`PIN_POINTS`] a block (every other step
    /// past 1024 samples, and so on), and a pin no longer moved goes back to its base once.
    /// Nothing is sent for a pin that nothing moves. Never allocates or waits.
    pub fn process_modulated(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        deadline: Option<Instant>,
        moved: u32,
        at: &[u32],
        offsets: &[[f32; MAX_PINS]],
    ) {
        self.take_changes();
        let n = left.len().min(right.len());
        let points = at.len().min(offsets.len());
        let stride = points.div_ceil(PIN_POINTS).max(1);
        // (Those no longer moved, owed a point at their base.)
        let mut owed = self.moved & !moved;
        let mut off = 0;
        while off < n {
            let end = (off + MAX_BLOCK).min(n);
            for stage in &mut self.stages {
                if stage.bypass.load(Ordering::Relaxed) {
                    continue;
                }
                if let Some(node) = &mut stage.node {
                    for (t, pin) in self.pins.iter().enumerate() {
                        if pin.stage != stage.id || pin.how == How::None {
                            continue;
                        }
                        let bit = 1u32 << t;
                        if moved & bit != 0 {
                            for k in (0..points).step_by(stride) {
                                let a = at[k] as usize;
                                if a < off || a >= end {
                                    continue;
                                }
                                let o = f64::from(offsets[k][t]);
                                let (kind, value) = match pin.how {
                                    How::Normalized => (EV_PARAM_MOD, o),
                                    How::Plain => (EV_PARAM_MOD, o * (pin.max - pin.min)),
                                    _ => (EV_PARAM, pin.plain(pin.base + o)),
                                };
                                node.push_mod(self.frame + a as i64, kind, pin.param, value);
                            }
                        } else if owed & bit != 0 {
                            let (kind, value) = match pin.how {
                                How::Value => (EV_PARAM, pin.plain(pin.base)),
                                _ => (EV_PARAM_MOD, 0.0),
                            };
                            node.push_mod(self.frame + off as i64, kind, pin.param, value);
                            owed &= !bit;
                        }
                    }
                    node.process_at(
                        self.frame + off as i64,
                        self.timing.at(off as f64),
                        &mut left[off..end],
                        &mut right[off..end],
                        deadline,
                    );
                }
            }
            off = end;
        }
        self.frame += n as i64;
        // (A pin whose effect was bypassed or down is still owed its base.)
        self.moved = moved | owed;
    }

    /// Pin target `target` set to `travel` (0..1) by a learned controller (audio thread: no
    /// allocation, no locks): its plug-in's value from the next block, its base as the chain
    /// keeps it within a tenth of a second (R-FX-PARAMS). Nothing for a target not pinned.
    pub fn set_pin_value(&mut self, target: usize, travel: f64) {
        let Some(pin) = self.pins.get_mut(target) else {
            return;
        };
        if pin.how == How::None {
            return;
        }
        pin.base = travel.clamp(0.0, 1.0);
        let (id, plain) = (pin.param, pin.plain(travel));
        if let Some(node) = self
            .stages
            .iter_mut()
            .find(|s| s.id == pin.stage)
            .and_then(|s| s.node.as_mut())
        {
            node.set_param(id, plain);
        }
        self.learnt[target].store(plain.to_bits(), Ordering::Release);
    }

    /// Whether pin target `target` is pinned to an effect listed here.
    pub fn is_pinned(&self, target: usize) -> bool {
        self.pins.get(target).is_some_and(|p| p.how != How::None)
    }

    /// Whether there are effects to run (a bypassed one counts), the chain's changes taken
    /// first (audio thread: no allocation, no locks). The synth asks it before each block and
    /// runs the chain only if not: the changes must be taken here, or an effect added to an
    /// empty chain would wait for a block that never comes (decisions.md R-FX-MAC).
    pub fn is_empty(&mut self) -> bool {
        self.take_changes();
        self.stages.is_empty()
    }
}
