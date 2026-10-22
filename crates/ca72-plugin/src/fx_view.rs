//! The effects' drawer in the editor (decisions.md R-FX-DRAWER; the owner's FX-A, 2026-10-18):
//! what it shows of the standalone's chain and what a pointer does on it. The chain is reached
//! through [`Effects`] (the standalone's, `fx.rs`; the tests' own), never waited on: what loads
//! or opens a window (seconds, for some plug-ins) is done on a thread of the drawer's, in order,
//! and a bypass on another, so that a slot switched on or off answers while another loads; the
//! chain is read only when it is free, every tenth of a second, or as soon as something done
//! there is finished.
//!
//! A click on a slot's name opens its effect's window, a drag onto another slot moves it there,
//! its tab bypasses it; a right click offers PARAMETERS, REPLACE and REMOVE (the editor's menus, MIDI
//! Learn's); the first empty slot reads "+ ADD EFFECT" and drops the effects' list, the effects
//! on this computer, found by a scan on a thread of its own the first time it is asked for.
//!
//! An effect turned off after it crashed or stopped responding (decisions.md R-FX-SAFETY) shows
//! its name dimmed over CRASHED or NOT RESPONDING: a click tries it again, and its menu offers
//! TRY AGAIN and LOAD WITHOUT ITS SETTINGS (first, when its settings are thought the cause). After
//! a safe start each effect reads OFF: a click turns the effects back on, and the menu offers
//! TURN EFFECTS BACK ON and TURN THIS ONE ON. One running without its settings offers TRY ITS OLD
//! SETTINGS. The tip over a slot says why, in a line.

mod search;
use search::{Choice, rows};
pub use search::{EffectId, Preferences};

use ca72_panel::presets::Field;
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ca72_panel::strip::fx::{
    self as fx_ui, FxList, FxOrder, FxRow, FxScene, FxSlot, FxTarget, FxWord, LIST_ROWS, SLOTS,
};

mod parameters;
use parameters::{ParameterReply, Parameters};

/// How an effect in the chain is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Health {
    #[default]
    Running,
    /// Being loaded (at start, tried again): dry meanwhile.
    Loading,
    /// Its helper died or hung; it is being started again (dry meanwhile).
    Restarting,
    /// Turned off after it crashed, and off at the next start until tried again: dry.
    Crashed,
    /// Turned off after it stopped responding, and off at the next start until tried again.
    NotResponding,
    /// Its plug-in could not be loaded here: dry, its state kept.
    Missing,
    /// Not loaded this time: a safe start.
    Off,
}

impl Health {
    /// Whether it is turned off or not loaded, so that a click tries it (or, off, turns the
    /// effects back on) rather than opening its window.
    fn idle(self) -> bool {
        matches!(
            self,
            Health::Crashed | Health::NotResponding | Health::Missing | Health::Off
        )
    }
}

/// An effect in the chain, as the drawer shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlotInfo {
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub bypass: bool,
    pub health: Health,
    /// Turned off because its saved settings crash it (or are thought to).
    pub by_settings: bool,
    /// It has saved settings (it can be loaded without them).
    pub has_state: bool,
    /// It runs without its saved settings, which are kept aside.
    pub set_aside: bool,
    /// Its old settings, given back a moment ago, still crashed it.
    pub try_failed: bool,
}

/// Why the effects are off this time (a safe start), for the slots' tips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeStart {
    /// The CA-72 did not close properly last time.
    DidNotClose,
    /// The player held Shift (or Option) as it opened.
    Asked,
}

/// A slot's menu's items: what each does, and its words.
pub type MenuItems = Vec<(FxAction, &'static str)>;

/// Something a slot's menu does (the editor's menus hand it back: `learning::Pressed::Fx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FxAction {
    /// The list dropped to put another effect in its place.
    Parameters(usize),
    Unpin(usize),
    Replace(usize),
    Remove(usize),
    /// Tried again (turned off, missing), or turned on (off for a safe start).
    Retry(usize),
    /// Loaded without its saved settings, which are kept aside.
    WithoutSettings(usize),
    /// Given back its settings kept aside.
    OldSettings(usize),
    /// Every effect off for a safe start turned back on.
    AllOn,
}

/// An effect found on this computer.
#[derive(Clone, Debug, PartialEq)]
pub struct Listing {
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub features: Vec<String>,
    pub key: EffectId,
}

/// The effects found: whether a scan is under way, whether one has finished, which scan this
/// list is (each scan's own number), and the effects.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalogue {
    pub scanning: bool,
    pub scanned: bool,
    /// Plug-in files the last scan could not list (crashed or hung their helper, or are not
    /// plug-ins): remembered, listed again by SCAN AGAIN or when they change (R-FX-SAFETY).
    pub unlisted: usize,
    pub generation: u64,
    pub effects: Vec<Listing>,
}

/// One of an effect's parameters as its PARAMETERS list shows it (decisions.md R-FX-PARAMS).
#[derive(Clone, Debug, PartialEq)]
pub struct ParamRow {
    /// The plug-in's ID for it.
    pub id: u32,
    /// The plug-in's names for it, and its units (VST3 alone gives a short name and units).
    pub name: String,
    pub short: String,
    pub units: String,
    /// Its steps past its first value (0: continuous); it steps when modulated.
    pub steps: u32,
    /// Its value now, a share of its travel (0..1), and in the plug-in's units (for its text,
    /// [`Effects::param_text`]).
    pub value: f64,
    pub plain: f64,
    /// The pin target it is pinned to (0 to 15: the synth's `fx1` to `fx16`), if it is.
    pub pinned: Option<usize>,
}

/// A pinned parameter, for its knob under its effect (B1), its rings, tip and menu
/// (decisions.md R-FX-PARAMS).
#[derive(Clone, Debug, PartialEq)]
pub struct PinInfo {
    /// Its target (0 to 15: the synth's `fx1` to `fx16`, `lfo::Target::Fx`), which stays its
    /// own while its effect moves.
    pub target: usize,
    /// Its effect's place now, and name.
    pub slot: usize,
    pub effect: String,
    pub id: u32,
    pub name: String,
    pub short: String,
    /// Its base, a share of its travel (0..1), and in the plug-in's units.
    pub value: f64,
    pub plain: Option<f64>,
    pub steps: u32,
    /// Its effect runs and has it. Not: dimmed and inert, its assignments kept.
    pub live: bool,
    /// Its effect runs but no longer has it: MISSING (UNPIN in its menu).
    pub missing: bool,
}

/// The rows of `rows` a PARAMETERS list shows, by index (decisions.md R-FX-PARAMS): those
/// whose name, short name or units hold every typed word (case aside), and, with `pinned`, only
/// the pinned ones (ALL · PINNED); in name order (the owner, 2026-10-20), then by ID.
pub fn search_params(rows: &[ParamRow], typed: &str, pinned: bool) -> Vec<usize> {
    let words: Vec<String> = typed.split_whitespace().map(str::to_lowercase).collect();
    let mut out: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            let text = format!("{} {} {}", r.name, r.short, r.units).to_lowercase();
            words.iter().all(|w| text.contains(w.as_str())) && (!pinned || r.pinned.is_some())
        })
        .map(|(i, _)| i)
        .collect();
    out.sort_by(|a, b| {
        let (x, y) = (&rows[*a], &rows[*b]);
        (x.name.to_lowercase(), x.id).cmp(&(y.name.to_lowercase(), y.id))
    });
    out
}

/// The synth's target for pin target `target` (`lfo::Target::Fx`), and whether it can take a
/// cord now: pinned to an effect that runs (`lfo::target_of` of its ID, `fx1` to `fx16`).
pub fn pin_target(p: &crate::params::Ca72Params, target: usize) -> Option<crate::lfo::Target> {
    let id = crate::lfo::Target::Fx(target.min(crate::lfo::FX_PINS - 1)).id();
    crate::lfo::target_of(p, id)
}

/// The preset’s chain compared with the chain heard (R-FX-PRESETS).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PresetStatus {
    pub locked: bool,
    pub differs: bool,
    pub held: Option<[String; SLOTS]>,
}

/// The chain as the drawer reaches it. What may take a while (loading, a window) is called on
/// the drawer's threads, never the editor's; the rest returns at once.
pub trait Effects: Send + Sync {
    fn load_preset(&self, _section: Option<String>) {}
    fn observe_preset(&self, _section: Option<String>) {}
    fn save_preset(&self) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn prepare_save_preset(&self) -> mpsc::Receiver<Result<Option<String>, String>> {
        let (tx, rx) = mpsc::channel();
        let _ = tx.send(self.save_preset());
        rx
    }
    fn preset_status(&self) -> PresetStatus {
        PresetStatus::default()
    }
    fn set_locked(&self, _on: bool) {}
    /// The chain's effects in order, or none if it is busy now.
    fn slots(&self) -> Option<Vec<SlotInfo>>;
    /// Whether any effect plays (the FX key's light), or none if the chain is busy now.
    fn active(&self) -> Option<bool>;
    /// The effects found so far.
    fn catalogue(&self) -> Catalogue;
    fn preferences(&self) -> Preferences;
    fn set_order(&self, order: FxOrder);
    /// A scan begun on a thread of its own, unless one is under way, or (unless `again`) one
    /// has finished.
    fn scan(&self, again: bool);
    /// Effect `effect` of scan `generation` added at the chain's end.
    fn add(&self, generation: u64, effect: usize) -> Result<(), String>;
    /// Effect `effect` of scan `generation` put in place of the chain's `slot`th.
    fn replace(&self, slot: usize, generation: u64, effect: usize) -> Result<(), String>;
    fn remove(&self, slot: usize) -> Result<(), String>;
    fn move_to(&self, from: usize, to: usize) -> Result<(), String>;
    fn set_bypass(&self, slot: usize, on: bool) -> Result<(), String>;
    /// The `slot`th effect's own window opened (or brought forward).
    fn open_editor(&self, slot: usize) -> Result<(), String>;
    /// The `slot`th effect tried again (turned off, missing; off for a safe start, turned on),
    /// on a thread of the chain's: it returns at once, the slot loading.
    fn retry(&self, slot: usize) -> Result<(), String>;
    /// The `slot`th effect loaded without its saved settings (kept aside), as `retry`.
    fn without_settings(&self, slot: usize) -> Result<(), String>;
    /// The `slot`th effect given back the settings kept aside, as `retry`.
    fn old_settings(&self, slot: usize) -> Result<(), String>;
    /// Every effect off for a safe start turned back on, as `retry`.
    fn turn_on(&self);
    /// Why the effects are off this time, if they are (a safe start).
    fn safe_start(&self) -> Option<SafeStart>;

    // The effects' parameters and pins (decisions.md R-FX-PARAMS). The listing and the texts
    // ask the effect's helper (milliseconds): the drawer's threads', or a frame's while its
    // list is open.

    /// The `slot`th effect's parameters a player may pin, with their values now, in the
    /// plug-in's order ([`search_params`] gives the list's).
    fn params(&self, _slot: usize) -> Result<Vec<ParamRow>, String> {
        Err("no effects here".into())
    }
    /// The `slot`th effect's own text for each (parameter, value in its units) pair: "2.40 s".
    fn param_text(&self, _slot: usize, values: &[(u32, f64)]) -> Vec<Option<String>> {
        vec![None; values.len()]
    }
    /// Every pin, by target; none if the chain is busy now.
    fn pins(&self) -> Option<Vec<PinInfo>> {
        Some(Vec::new())
    }
    /// Parameter `id` of the `slot`th effect pinned: its target. A fifth of an effect is refused.
    fn pin(&self, _slot: usize, _id: u32) -> Result<usize, String> {
        Err("no effects here".into())
    }
    /// Pin target `target` unpinned, what moved it let go of.
    /// Recent editor touches, newest first, or none while the chain is busy.
    fn touched(&self, _slot: usize) -> Option<Vec<u32>> {
        Some(Vec::new())
    }

    fn repin(&self, _target: usize, _id: u32) -> Result<(), String> {
        Err("parameters are not available".into())
    }

    fn unpin(&self, _target: usize) -> Result<(), String> {
        Err("no effects here".into())
    }
    /// Pin target `target`'s value set to `travel` (0..1): the drawer's knob turned.
    fn set_pin_value(&self, _target: usize, _travel: f64) -> Result<(), String> {
        Err("no effects here".into())
    }
}

/// The standalone's chain, if the application has one (the plug-ins: none).
pub fn installed() -> Option<Arc<dyn Effects>> {
    #[cfg(feature = "fx-host")]
    {
        crate::fx::host().map(|h| h as Arc<dyn Effects>)
    }
    #[cfg(not(feature = "fx-host"))]
    {
        None
    }
}

/// Something to do to the chain, on one of the drawer's threads.
#[derive(Clone, Debug, PartialEq)]
enum Job {
    Add(u64, usize),
    Replace(usize, u64, usize),
    Remove(usize),
    Move(usize, usize),
    Bypass(usize, bool),
    Window(usize),
    Act(FxAction),
    Parameters(usize, u64),
    Pin(usize, u32),
    Unpin(usize),
    Repin(usize, u32),
    Recent(usize, Vec<u32>),
    Texts(Vec<PinInfo>),
    Order(FxOrder),
}

/// What the drawer's threads are doing and have done: an effect being loaded (its slot and
/// name), one that would not load (its slot, name and when), and how many jobs are finished.
#[derive(Debug, Default)]
struct Pending {
    loading: Option<(usize, String)>,
    failed: Option<(usize, String, Instant)>,
    done: u64,
    parameters: Option<ParameterReply>,
    pin_error: Option<String>,
    pin_texts: Vec<(PinInfo, String)>,
    recent: Vec<(usize, Vec<u32>, Vec<ParamRow>)>,
}

/// How long an effect that would not load is said so on its slot.
const FAILED_SHOWN: Duration = Duration::from_secs(5);
/// How often the chain is read.
const POLL: Duration = Duration::from_millis(100);
/// How far a slot's name is dragged (drawing units) before it is carried.
const CARRY: f64 = 12.0;

/// The list, open: for which slot, to add or to replace, its first row in view, the row under
/// the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Open {
    slot: usize,
    replace: bool,
    first: usize,
    hover: Option<usize>,
    chosen: Option<usize>,
}

/// A slot's name pressed: which, where (drawing units), whether it is being carried, and the slot
/// it is over.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Press {
    slot: usize,
    from: (f64, f64),
    carried: bool,
    over: Option<usize>,
}

/// The editor's effects' drawer.
pub struct FxView {
    effects: Option<Arc<dyn Effects>>,
    loads: Option<mpsc::Sender<Job>>,
    quick: Option<mpsc::Sender<Job>>,
    pending: Arc<Mutex<Pending>>,
    slots: Vec<SlotInfo>,
    active: bool,
    polled: Option<Instant>,
    seen: u64,
    catalogue: Catalogue,
    open: Option<Open>,
    query: Field,
    preferences: Preferences,
    rows: Vec<(FxRow, Choice)>,
    press: Option<Press>,
    pub hover: Option<FxTarget>,
    parameters: Option<Parameters>,
    pins: Vec<PinInfo>,
    pin_texts: std::collections::BTreeMap<usize, String>,
    request: u64,
    touched: [Vec<u32>; SLOTS],
    recent: [Vec<ParamRow>; SLOTS],
}

impl std::fmt::Debug for FxView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FxView")
            .field("slots", &self.slots)
            .field("open", &self.open)
            .finish_non_exhaustive()
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl FxView {
    /// The drawer for `effects` (none: no drawer, the plug-ins' rail), its threads begun.
    pub fn new(effects: Option<Arc<dyn Effects>>) -> FxView {
        let pending = Arc::new(Mutex::new(Pending::default()));
        let worker = |name: &str| {
            let effects = effects.clone()?;
            let pending = pending.clone();
            let (tx, rx) = mpsc::channel::<Job>();
            std::thread::Builder::new()
                .name(name.into())
                .spawn(move || {
                    for job in rx {
                        run(effects.as_ref(), &pending, job);
                    }
                })
                .ok()?;
            Some(tx)
        };
        let (loads, quick) = (worker("ca72-fx-loads"), worker("ca72-fx-quick"));
        let preferences = effects
            .as_ref()
            .map(|e| e.preferences())
            .unwrap_or_default();
        FxView {
            effects,
            loads,
            quick,
            pending,
            slots: Vec::new(),
            active: false,
            polled: None,
            seen: 0,
            catalogue: Catalogue::default(),
            open: None,
            query: Field::default(),
            preferences,
            rows: Vec::new(),
            press: None,
            hover: None,
            parameters: None,
            pins: Vec::new(),
            pin_texts: Default::default(),
            request: 0,
            touched: Default::default(),
            recent: Default::default(),
        }
    }

    /// Whether there is a chain (the standalone's): the rail has its FX key.
    pub(crate) fn effects(&self) -> Option<Arc<dyn Effects>> {
        self.effects.clone()
    }

    pub fn available(&self) -> bool {
        self.effects.is_some()
    }

    /// The chain read again if it is time (or something done there has finished), and the
    /// effects found while the list is open.
    pub fn poll(&mut self, now: Instant) {
        let Some(e) = &self.effects else { return };
        let done = lock(&self.pending).done;
        if done == self.seen && self.polled.is_some_and(|t| now.duration_since(t) < POLL) {
            return;
        }
        self.polled = Some(now);
        if let Some(slots) = e.slots() {
            self.seen = done;
            self.slots = slots;
        }
        if let Some(a) = e.active() {
            self.active = a;
        }
        if self.open.is_some() {
            let catalogue = e.catalogue();
            if self.catalogue != catalogue {
                self.catalogue = catalogue;
                self.refilter();
            }
        }
        self.poll_parameters();
    }

    /// Whether any effect plays: the FX key lit.
    pub fn active(&self) -> bool {
        self.active
    }

    /// The effect in slot `i`, if there is one.
    fn effect(&self, i: usize) -> Option<&SlotInfo> {
        self.slots.get(i)
    }

    /// The slot reading "+ ADD EFFECT": the first empty one, none while an effect loads into it.
    fn add_slot(&self) -> Option<usize> {
        let n = self.slots.len();
        let loading = lock(&self.pending)
            .loading
            .as_ref()
            .is_some_and(|(s, _)| *s >= n);
        (n < SLOTS && !loading).then_some(n)
    }

    /// Whether something is being done to the chain that the slots must wait for.
    fn busy(&self) -> bool {
        lock(&self.pending).loading.is_some()
    }

    /// What the drawer shows.
    pub fn scene(&self) -> FxScene {
        let pending = lock(&self.pending);
        let failed = pending
            .failed
            .as_ref()
            .filter(|(.., at)| at.elapsed() < FAILED_SHOWN);
        let slots = std::array::from_fn(|i| {
            if let Some((_, name)) = pending.loading.as_ref().filter(|(s, _)| *s == i) {
                return FxSlot {
                    text: name.clone(),
                    dim: true,
                    lit: false,
                    word: Some(FxWord::Loading),
                };
            }
            if let Some(e) = self.effect(i) {
                let (dim, word) = match e.health {
                    Health::Running if e.try_failed => (false, Some(FxWord::Failed)),
                    Health::Running => (false, None),
                    Health::Loading => (true, Some(FxWord::Loading)),
                    Health::Restarting => (true, Some(FxWord::Restarting)),
                    Health::Crashed => (true, Some(FxWord::Crashed)),
                    Health::NotResponding => (true, Some(FxWord::NotResponding)),
                    Health::Missing => (true, Some(FxWord::Missing)),
                    Health::Off => (true, Some(FxWord::Off)),
                };
                return FxSlot {
                    text: e.name.clone(),
                    dim,
                    lit: e.health == Health::Running && !e.bypass,
                    word,
                };
            }
            if let Some((_, name, _)) = failed.filter(|(s, ..)| *s == i) {
                return FxSlot {
                    text: name.clone(),
                    dim: true,
                    lit: false,
                    word: Some(FxWord::Failed),
                };
            }
            if pending.loading.is_none() && i == self.slots.len() {
                return FxSlot {
                    text: "+ ADD EFFECT".into(),
                    ..FxSlot::default()
                };
            }
            FxSlot::default()
        });
        drop(pending);
        let preset = self
            .effects
            .as_ref()
            .map(|e| e.preset_status())
            .unwrap_or_default();
        FxScene {
            locked: preset.locked,
            held: preset.held,
            slots,
            hover: self.hover,
            drag: self.press.filter(|p| p.carried).map(|p| (p.slot, p.over)),
            list: self
                .parameters_list()
                .or_else(|| self.open.map(|o| self.list(o))),
            pins: self.drawn_pins(),
            occupied: self.slots.len(),
            offers: std::array::from_fn(|slot| {
                self.offer(slot).map(|r| {
                    format!(
                        "+ {}",
                        if r.short.is_empty() {
                            &r.name
                        } else {
                            &r.short
                        }
                    )
                    .to_uppercase()
                })
            }),
        }
    }

    /// The list as it shows for `o`.
    fn list(&self, o: Open) -> FxList {
        let c = &self.catalogue;
        let n = self
            .rows
            .iter()
            .filter(|(_, c)| matches!(c, Choice::Effect(_)))
            .count();
        FxList {
            slot: o.slot,
            title: if o.replace {
                format!("REPLACE SLOT {}", o.slot + 1)
            } else {
                format!("ADD TO SLOT {}", o.slot + 1)
            },
            note: if c.scanning {
                "SCANNING\u{2026}".into()
            } else if !c.scanned {
                String::new()
            } else {
                let found = match n {
                    0 => "NONE FOUND".to_owned(),
                    1 => "1 EFFECT".to_owned(),
                    _ => format!("{n} EFFECTS"),
                };
                let found = if !self.query.text.is_empty() {
                    format!("{n} OF {} EFFECTS", c.effects.len())
                } else if self.preferences.order == FxOrder::Recent {
                    format!("{n} RECENT")
                } else {
                    found
                };
                match c.unlisted {
                    0 => found,
                    u => format!("{found}, {u} NOT LISTED"),
                }
            },
            rows: self.rows.iter().map(|(r, _)| r.clone()).collect(),
            first: o.first,
            hover: o.hover,
            find: None,
            chosen: o.chosen,
            query: self.query.clone(),
            order: self.preferences.order,
        }
    }

    /// Whether the effects' list is open.
    pub fn list_open(&self) -> bool {
        self.open.is_some() || self.parameters.is_some()
    }

    /// The effects' list dropped for slot `slot`, to add an effect there or to replace its own;
    /// the effects found the first time it is asked for.
    pub fn open_list(&mut self, slot: usize, replace: bool) {
        self.close_list();
        self.press = None;
        self.open = Some(Open {
            slot: slot.min(SLOTS - 1),
            replace,
            first: 0,
            hover: None,
            chosen: None,
        });
        self.query = Field::default();
        if let Some(e) = &self.effects {
            e.scan(false);
            self.catalogue = e.catalogue();
            self.preferences = e.preferences();
        }
        self.refilter();
    }

    pub fn close_list(&mut self) {
        self.open = None;
        self.parameters = None;
    }

    /// The list scrolled by `rows` (+ down).
    pub fn scroll(&mut self, rows_by: i32) {
        if let Some(p) = &mut self.parameters {
            let last = p.found().len().saturating_sub(LIST_ROWS);
            p.first = p.first.saturating_add_signed(rows_by as isize).min(last);
            p.chosen = p.chosen.clamp(p.first, p.first + LIST_ROWS - 1);
            return;
        }
        let last = self.rows.len().saturating_sub(LIST_ROWS);
        if let Some(o) = &mut self.open {
            o.first = o.first.saturating_add_signed(rows_by as isize).min(last);
        }
    }

    fn set_order(&mut self, order: FxOrder) {
        self.preferences.order = order;
        self.submit(Job::Order(order));
        self.refilter();
    }

    fn refilter(&mut self) {
        self.rows = rows(
            &self.catalogue.effects,
            &self.query.text,
            &self.preferences,
            self.catalogue.scanning,
        );
        let chosen = self
            .rows
            .iter()
            .position(|(_, c)| !matches!(c, Choice::Head | Choice::Blank));
        if let Some(o) = &mut self.open {
            o.first = 0;
            o.hover = None;
            o.chosen = chosen;
        }
    }

    fn navigate(&mut self, by: i32) {
        let choices: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(i, (_, c))| (!matches!(c, Choice::Head | Choice::Blank)).then_some(i))
            .collect();
        let Some(o) = &mut self.open else { return };
        if choices.is_empty() {
            o.chosen = None;
            return;
        }
        let at = choices
            .iter()
            .position(|i| Some(*i) == o.chosen)
            .unwrap_or(0);
        let at = at.saturating_add_signed(by as isize).min(choices.len() - 1);
        let row = choices[at];
        o.chosen = Some(row);
        o.hover = None;
        if row < o.first {
            o.first = row;
        }
        if row >= o.first + LIST_ROWS {
            o.first = row + 1 - LIST_ROWS;
        }
        o.first = o.first.min(self.rows.len().saturating_sub(LIST_ROWS));
    }

    /// The list takes typing and navigation while open, including releases, so notes and
    /// preset stepping cannot start beneath it. FIND uses the presets’ own field editor.
    pub fn key(&mut self, e: &KeyboardEvent) -> bool {
        if self.open.is_none() {
            return false;
        }
        if e.state != KeyState::Down {
            return true;
        }
        if e.modifiers.intersects(Modifiers::META | Modifiers::CONTROL) || e.is_composing {
            return true;
        }
        let was = self.query.text.clone();
        match &e.key {
            Key::Escape if self.query.text.is_empty() => self.close_list(),
            Key::Escape => self.query = Field::default(),
            Key::ArrowUp => self.navigate(-1),
            Key::ArrowDown => self.navigate(1),
            Key::PageUp => self.navigate(-(LIST_ROWS as i32)),
            Key::PageDown => self.navigate(LIST_ROWS as i32),
            Key::Enter => {
                if let Some(row) = self.open.and_then(|o| o.chosen) {
                    self.choose(row);
                }
            }
            Key::Character(c) => self.query.insert(c),
            Key::Backspace => self.query.backspace(),
            Key::Delete => self.query.delete(),
            Key::ArrowLeft => self.query.left(),
            Key::ArrowRight => self.query.right(),
            Key::Home => self.query.home(),
            Key::End => self.query.end(),
            _ => {}
        }
        if was != self.query.text {
            self.refilter();
        }
        true
    }

    /// The list as it shows (for the hit test).
    fn shown_list(&self) -> Option<FxList> {
        self.parameters_list()
            .or_else(|| self.open.map(|o| self.list(o)))
    }

    /// A press at (`x`, `y`) (drawing units) on the drawer or its list.
    pub fn press(&mut self, x: f64, y: f64) {
        let list = self.shown_list();
        if let Some(l) = &list {
            match self.scene().hit(x, y) {
                Some(FxTarget::Row(r)) if l.find.is_some() => self.choose_parameter(r),
                Some(FxTarget::Filter(pinned)) => {
                    if let Some(p) = &mut self.parameters {
                        p.pinned = pinned;
                        p.chosen = 0;
                        p.first = 0;
                        p.refilter();
                    }
                }
                Some(FxTarget::Find(col)) if l.find.is_some() => {
                    if let Some(p) = &mut self.parameters {
                        p.field.caret = (p.field.caret.saturating_sub(33) + col)
                            .min(p.field.text.chars().count());
                    }
                }
                Some(FxTarget::Row(r)) => self.choose(r),
                Some(FxTarget::Order(order)) => self.set_order(order),
                Some(FxTarget::Find(col)) => {
                    let first = self.query.caret.saturating_sub(27);
                    self.query.caret = (first + col).min(self.query.text.chars().count());
                }
                Some(FxTarget::List) => {}
                _ => self.close_list(),
            }
            return;
        }
        match self.scene().hit(x, y) {
            Some(FxTarget::Field(slot, place)) => {
                if let Some(pin) = self.scene().pin(slot, place) {
                    self.open_repin(pin.target);
                } else if let Some(row) = self.offer(slot) {
                    self.submit(Job::Pin(slot, row.id));
                } else {
                    self.open_parameters(slot);
                }
            }
            Some(FxTarget::Arrow(slot, place)) => {
                if let Some(pin) = self.scene().pin(slot, place) {
                    self.open_repin(pin.target);
                } else {
                    self.open_parameters(slot);
                }
            }
            Some(FxTarget::Lock) => {
                if let Some(e) = &self.effects {
                    e.set_locked(!e.preset_status().locked);
                }
            }
            Some(FxTarget::Name(i)) if self.effect(i).is_some() && !self.busy() => {
                self.press = Some(Press {
                    slot: i,
                    from: (x, y),
                    carried: false,
                    over: None,
                });
            }
            Some(FxTarget::Name(i)) if self.add_slot() == Some(i) => self.open_list(i, false),
            Some(FxTarget::Tab(i)) => {
                if let Some(e) = self.effect(i) {
                    let on = !e.bypass;
                    self.submit(Job::Bypass(i, on));
                    // (Shown at once; the chain read again confirms it.)
                    if let Some(e) = self.slots.get_mut(i) {
                        e.bypass = on;
                    }
                }
            }
            _ => {}
        }
    }

    /// The pointer at (`x`, `y`): a slot carried over another, the part under it lit.
    pub fn moved(&mut self, x: f64, y: f64) {
        let list = self.shown_list();
        let found = self.scene().hit(x, y);
        if let Some(p) = &mut self.parameters
            && let Some(FxTarget::Row(row)) = found
        {
            p.chosen = row;
        }
        if let Some(o) = &mut self.open {
            o.hover = match found {
                Some(FxTarget::Row(r)) => Some(r),
                _ => None,
            };
        }
        let n = self.slots.len();
        if let Some(p) = &mut self.press {
            if (x - p.from.0).hypot(y - p.from.1) > CARRY {
                p.carried = true;
            }
            p.over = match found {
                Some(FxTarget::Name(j) | FxTarget::Tab(j)) if j < n && j != p.slot => Some(j),
                _ => None,
            };
        }
        self.hover = found.filter(|_| list.is_none());
    }

    /// The pointer let go: a slot carried onto another moved there; one pressed and not moved,
    /// its effect's window opened, or (turned off, missing) it tried again, or (off for a safe
    /// start) the effects turned back on.
    pub fn released(&mut self) {
        let Some(p) = self.press.take() else { return };
        match (p.carried, p.over) {
            (true, Some(to)) => self.submit(Job::Move(p.slot, to)),
            (true, None) => {}
            (false, _) => match self.effect(p.slot).map(|e| e.health) {
                Some(Health::Off) => self.act(FxAction::AllOn),
                Some(h) if h.idle() => self.act(FxAction::Retry(p.slot)),
                Some(Health::Loading) | None => {}
                Some(_) => self.submit(Job::Window(p.slot)),
            },
        }
    }

    /// Whether a slot's name is held.
    pub fn pressing(&self) -> bool {
        self.press.is_some()
    }

    /// A press let go of without its release (the window lost the pointer).
    pub fn cancel_press(&mut self) {
        self.press = None;
    }

    /// The menu a right click at (`x`, `y`) opens: its effect's name as its title, and its
    /// items. What the effect's state offers comes first (TRY AGAIN, LOAD WITHOUT ITS SETTINGS,
    /// TURN EFFECTS BACK ON...), then REPLACE… and REMOVE.
    pub fn menu_for(&self, x: f64, y: f64) -> Option<(String, MenuItems)> {
        let i = match fx_ui::hit(x, y, None) {
            Some(FxTarget::Name(i) | FxTarget::Tab(i)) if !self.busy() => i,
            _ => return None,
        };
        let e = self.effect(i)?;
        let mut items = Vec::new();
        let without = (FxAction::WithoutSettings(i), "LOAD WITHOUT ITS SETTINGS");
        let again = (FxAction::Retry(i), "TRY AGAIN");
        match e.health {
            Health::Off => {
                items.push((FxAction::AllOn, "TURN EFFECTS BACK ON"));
                items.push((FxAction::Retry(i), "TURN THIS ONE ON"));
            }
            Health::Crashed | Health::NotResponding if e.by_settings => {
                items.push(without);
                items.push(again);
            }
            Health::Crashed | Health::NotResponding => {
                items.push(again);
                if e.has_state {
                    items.push(without);
                }
            }
            Health::Missing => items.push(again),
            Health::Running if e.set_aside => {
                items.push((FxAction::OldSettings(i), "TRY ITS OLD SETTINGS"));
            }
            Health::Running | Health::Loading | Health::Restarting => {}
        }
        items.push((FxAction::Parameters(i), "PARAMETERS…"));
        items.push((FxAction::Replace(i), "REPLACE\u{2026}"));
        items.push((FxAction::Remove(i), "REMOVE"));
        Some((e.name.clone(), items))
    }

    /// A slot's menu's item chosen.
    pub fn act(&mut self, a: FxAction) {
        match a {
            FxAction::Parameters(slot) => self.open_parameters(slot),
            FxAction::Unpin(target) => self.unpin(target),
            FxAction::Replace(slot) => {
                if self.effect(slot).is_some() {
                    self.open_list(slot, true);
                }
            }
            FxAction::Remove(slot) => {
                if self.effect(slot).is_some() {
                    self.submit(Job::Remove(slot));
                }
            }
            FxAction::AllOn => {
                self.submit(Job::Act(a));
                // (Shown at once; the chain read again confirms it.)
                for e in &mut self.slots {
                    if e.health == Health::Off {
                        e.health = Health::Loading;
                    }
                }
            }
            FxAction::Retry(slot)
            | FxAction::WithoutSettings(slot)
            | FxAction::OldSettings(slot) => {
                if let Some(e) = self.slots.get_mut(slot) {
                    e.health = Health::Loading;
                    e.try_failed = false;
                    self.submit(Job::Act(a));
                }
            }
        }
    }

    /// The tip over slot `i`'s name: its effect's whole name and maker, or why it is off and what
    /// a click does (decisions.md R-FX-SAFETY).
    pub fn tip(&self) -> Option<(String, f64, f64)> {
        let Some(FxTarget::Name(i)) = self.hover else {
            return None;
        };
        if self.press.is_some() {
            return None;
        }
        let e = self.effect(i)?;
        let (x, y, w, _) = fx_ui::screen(i);
        let named = if e.vendor.is_empty() {
            e.name.clone()
        } else {
            format!("{} ({})", e.name, e.vendor)
        };
        let safe = self.effects.as_ref().and_then(|f| f.safe_start());
        let text = match e.health {
            Health::Off => match safe {
                Some(SafeStart::Asked) => {
                    "Effects off for this start. Click to turn them on".into()
                }
                _ => "Effects off: the CA-72 did not close properly. Click to turn them on".into(),
            },
            Health::Crashed if e.by_settings => {
                format!(
                    "{}: its saved settings crash it. Right-click for choices",
                    e.name
                )
            }
            Health::Crashed => format!(
                "{}: turned off after it crashed. Click to try again",
                e.name
            ),
            Health::NotResponding => {
                format!(
                    "{}: turned off, it stopped responding. Click to try again",
                    e.name
                )
            }
            Health::Missing => format!("{}: not found on this computer", e.name),
            Health::Running if e.try_failed => {
                format!("{}: its old settings still crash it", e.name)
            }
            Health::Running if e.set_aside => format!("{named}, without its saved settings"),
            _ => named,
        };
        Some((text, x + w / 2.0, y - 6.0))
    }

    /// Row `row` of the list chosen: its effect added or put in place; or the computer
    /// scanned again.
    fn choose(&mut self, row: usize) {
        let Some(o) = self.open else { return };
        let Some((_, choice)) = self.rows.get(row) else {
            return;
        };
        let index = match *choice {
            Choice::Effect(i) => i,
            Choice::Scan => {
                if let Some(e) = &self.effects {
                    e.scan(true);
                    self.catalogue = e.catalogue();
                    self.refilter();
                }
                return;
            }
            Choice::All => {
                self.set_order(FxOrder::Name);
                return;
            }
            Choice::Head | Choice::Blank => return,
        };
        let c = &self.catalogue;
        let Some(l) = c.effects.get(index) else {
            return;
        };
        let (generation, name) = (c.generation, l.name.clone());
        self.open = None;
        let at = if o.replace { o.slot } else { self.slots.len() };
        lock(&self.pending).loading = Some((at, name));
        if o.replace {
            self.submit(Job::Replace(o.slot, generation, index));
        } else {
            self.submit(Job::Add(generation, index));
        }
    }

    /// `job` handed to its thread (a bypass, and what the chain does on threads of its own, to
    /// the quick one; the rest in order to the other).
    fn submit(&mut self, job: Job) {
        let lane = if matches!(job, Job::Bypass(..) | Job::Act(_)) {
            &self.quick
        } else {
            &self.loads
        };
        if let Some(tx) = lane {
            let _ = tx.send(job);
        }
        self.polled = None;
    }
}

/// `job` done to `effects`, on one of the drawer's threads; what it did kept in `pending`.
fn run(effects: &dyn Effects, pending: &Mutex<Pending>, job: Job) {
    let result = match &job {
        Job::Parameters(slot, request) => {
            let result = effects.params(*slot).map(|rows| {
                let values: Vec<_> = rows.iter().map(|r| (r.id, r.plain)).collect();
                let texts = effects.param_text(*slot, &values);
                rows.into_iter()
                    .enumerate()
                    .map(|(i, r)| {
                        (
                            r,
                            texts
                                .get(i)
                                .cloned()
                                .flatten()
                                .unwrap_or_else(|| "…".into()),
                        )
                    })
                    .collect()
            });
            lock(pending).parameters = Some((*slot, *request, result));
            Ok(())
        }
        Job::Texts(pins) => {
            let texts = parameters::texts(effects, pins.clone());
            lock(pending).pin_texts = texts;
            Ok(())
        }
        Job::Recent(slot, ids) => {
            let rows = effects.params(*slot).unwrap_or_default();
            lock(pending).recent.push((*slot, ids.clone(), rows));
            Ok(())
        }
        Job::Repin(target, id) => effects.repin(*target, *id),
        Job::Pin(slot, id) => effects.pin(*slot, *id).map(|_| ()),
        Job::Unpin(target) => effects.unpin(*target),
        Job::Add(g, i) => effects.add(*g, *i),
        Job::Replace(s, g, i) => effects.replace(*s, *g, *i),
        Job::Remove(s) => effects.remove(*s),
        Job::Move(f, t) => effects.move_to(*f, *t),
        Job::Bypass(s, on) => effects.set_bypass(*s, *on),
        Job::Window(s) => effects.open_editor(*s),
        Job::Act(FxAction::Retry(s)) => effects.retry(*s),
        Job::Act(FxAction::WithoutSettings(s)) => effects.without_settings(*s),
        Job::Act(FxAction::OldSettings(s)) => effects.old_settings(*s),
        Job::Order(order) => {
            effects.set_order(*order);
            Ok(())
        }
        Job::Act(FxAction::AllOn) => {
            effects.turn_on();
            Ok(())
        }
        Job::Act(
            FxAction::Parameters(_)
            | FxAction::Unpin(_)
            | FxAction::Replace(_)
            | FxAction::Remove(_),
        ) => Ok(()),
    };
    let mut p = lock(pending);
    if matches!(job, Job::Add(..) | Job::Replace(..)) {
        let loading = p.loading.take();
        if result.is_err()
            && let Some((slot, name)) = loading
        {
            p.failed = Some((slot, name, Instant::now()));
        }
    }
    if matches!(job, Job::Pin(..) | Job::Unpin(_) | Job::Repin(..)) {
        p.pin_error = result.clone().err();
    }
    p.done += 1;
    drop(p);
    if let Err(e) = result
        && !matches!(job, Job::Pin(..) | Job::Unpin(_) | Job::Repin(..))
    {
        eprintln!("CA-72 effects: {e}");
    }
}

#[cfg(test)]
pub(crate) mod fake {
    //! A chain for the tests: what is done to it is done at once and noted.

    use super::*;

    type SaveReply = mpsc::Sender<Result<Option<String>, String>>;

    #[derive(Debug, Default)]
    pub struct Fake {
        pub slots: Mutex<Vec<SlotInfo>>,
        pub found: Vec<Listing>,
        pub done: Mutex<Vec<String>>,
        pub scans: Mutex<u32>,
        pub safe: Mutex<Option<SafeStart>>,
        pub unlisted: Mutex<usize>,
        pub parameter_rows: Mutex<std::collections::BTreeMap<usize, Vec<ParamRow>>>,
        pub pins: Mutex<Vec<PinInfo>>,
        pub touched: Mutex<std::collections::BTreeMap<usize, Vec<u32>>>,
        pub pins_busy: std::sync::atomic::AtomicBool,
        pub param_reads: std::sync::atomic::AtomicUsize,
        pub preferences: Mutex<Preferences>,
        pub preset: Mutex<PresetStatus>,
        pub preset_raw: Mutex<Option<String>>,
        pub delay_save: std::sync::atomic::AtomicBool,
        pub save_replies: Mutex<Vec<SaveReply>>,
    }

    impl Fake {
        pub fn with(names: &[&str], found: &[&str]) -> Arc<Fake> {
            let info = |n: &&str| SlotInfo {
                name: (*n).into(),
                vendor: "Maker".into(),
                format: "clap".into(),
                has_state: true,
                ..SlotInfo::default()
            };
            Arc::new(Fake {
                slots: Mutex::new(names.iter().map(info).collect()),
                found: found
                    .iter()
                    .map(|n| Listing {
                        name: (*n).into(),
                        vendor: "Maker".into(),
                        format: "vst3".into(),
                        features: vec!["reverb".into()],
                        key: EffectId {
                            format: "vst3".into(),
                            path: (*n).into(),
                            id: (*n).into(),
                        },
                    })
                    .collect(),
                ..Fake::default()
            })
        }

        pub fn take(&self) -> Vec<String> {
            std::mem::take(&mut lock(&self.done))
        }

        fn note(&self, s: String) {
            lock(&self.done).push(s);
        }
    }

    impl Fake {
        pub fn parameters(&self, slot: usize, names: &[&str]) {
            let rows = names
                .iter()
                .enumerate()
                .map(|(i, name)| ParamRow {
                    id: slot as u32 * 100 + i as u32,
                    name: (*name).into(),
                    short: String::new(),
                    units: if *name == "Decay" { "s" } else { "%" }.into(),
                    steps: 0,
                    value: if *name == "Decay" { 0.42 } else { 0.35 },
                    plain: if *name == "Decay" { 2.4 } else { 35.0 },
                    pinned: None,
                })
                .collect();
            lock(&self.parameter_rows).insert(slot, rows);
        }
    }

    impl Effects for Fake {
        fn params(&self, slot: usize) -> Result<Vec<ParamRow>, String> {
            self.param_reads
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mut rows = lock(&self.parameter_rows)
                .get(&slot)
                .cloned()
                .unwrap_or_default();
            let pins = lock(&self.pins);
            for row in &mut rows {
                row.pinned = pins
                    .iter()
                    .find(|p| p.slot == slot && p.id == row.id)
                    .map(|p| p.target);
            }
            Ok(rows)
        }
        fn param_text(&self, slot: usize, values: &[(u32, f64)]) -> Vec<Option<String>> {
            let rows = lock(&self.parameter_rows);
            values
                .iter()
                .map(|(id, value)| {
                    rows.get(&slot)?
                        .iter()
                        .find(|p| p.id == *id)
                        .map(|p| format!("{value:.2} {}", p.units))
                })
                .collect()
        }
        fn pins(&self) -> Option<Vec<PinInfo>> {
            (!self.pins_busy.load(std::sync::atomic::Ordering::Relaxed))
                .then(|| lock(&self.pins).clone())
        }
        fn pin(&self, slot: usize, id: u32) -> Result<usize, String> {
            let rows = lock(&self.parameter_rows);
            let row = rows
                .get(&slot)
                .and_then(|r| r.iter().find(|r| r.id == id))
                .ok_or("no parameter")?;
            let mut pins = lock(&self.pins);
            if pins.iter().filter(|p| p.slot == slot).count() >= 4 {
                return Err("4 OF 4 PINNED".into());
            }
            if pins.iter().any(|p| p.slot == slot && p.id == id) {
                return Err("already pinned".into());
            }
            let target = (0..16)
                .find(|t| pins.iter().all(|p| p.target != *t))
                .ok_or("16 OF 16 PINNED")?;
            pins.push(PinInfo {
                target,
                slot,
                id,
                effect: lock(&self.slots)[slot].name.clone(),
                name: row.name.clone(),
                short: row.short.clone(),
                value: row.value,
                plain: Some(row.plain),
                steps: row.steps,
                live: true,
                missing: false,
            });
            pins.sort_by_key(|p| p.target);
            self.note(format!("pin {slot} {id}"));
            Ok(target)
        }
        fn touched(&self, slot: usize) -> Option<Vec<u32>> {
            Some(lock(&self.touched).get(&slot).cloned().unwrap_or_default())
        }
        fn repin(&self, target: usize, id: u32) -> Result<(), String> {
            let rows = lock(&self.parameter_rows);
            let mut pins = lock(&self.pins);
            let pin = pins.iter().find(|p| p.target == target).ok_or("no pin")?;
            if pin.id == id {
                return Ok(());
            }
            if pins.iter().any(|p| p.slot == pin.slot && p.id == id) {
                return Err("already pinned".into());
            }
            let pin = pins
                .iter_mut()
                .find(|p| p.target == target)
                .expect("the pin");
            let row = rows
                .get(&pin.slot)
                .and_then(|r| r.iter().find(|r| r.id == id))
                .ok_or("no parameter")?;
            pin.id = id;
            pin.name.clone_from(&row.name);
            pin.short.clone_from(&row.short);
            pin.value = row.value;
            pin.plain = Some(row.plain);
            pin.steps = row.steps;
            self.note(format!("repin {target} {id}"));
            Ok(())
        }
        fn unpin(&self, target: usize) -> Result<(), String> {
            lock(&self.pins).retain(|p| p.target != target);
            self.note(format!("unpin {target}"));
            Ok(())
        }
        fn set_pin_value(&self, target: usize, value: f64) -> Result<(), String> {
            let mut pins = lock(&self.pins);
            let pin = pins
                .iter_mut()
                .find(|p| p.target == target)
                .ok_or("no pin")?;
            if !pin.live {
                return Err("not live".into());
            }
            let scale = if pin.name == "Decay" {
                2.4 / 0.42
            } else {
                100.0
            };
            pin.value = value;
            pin.plain = Some(scale * value);
            self.note(format!("value {target} {value}"));
            Ok(())
        }
        fn prepare_save_preset(&self) -> mpsc::Receiver<Result<Option<String>, String>> {
            let (tx, rx) = mpsc::channel();
            if self.delay_save.load(std::sync::atomic::Ordering::Relaxed) {
                lock(&self.save_replies).push(tx);
            } else {
                let _ = tx.send(self.save_preset());
            }
            rx
        }
        fn preset_status(&self) -> PresetStatus {
            lock(&self.preset).clone()
        }
        fn set_locked(&self, on: bool) {
            lock(&self.preset).locked = on;
            self.note(format!("lock {on}"));
        }
        fn load_preset(&self, section: Option<String>) {
            *lock(&self.preset_raw) = section;
        }
        fn save_preset(&self) -> Result<Option<String>, String> {
            Ok(lock(&self.preset_raw).clone())
        }

        fn slots(&self) -> Option<Vec<SlotInfo>> {
            Some(lock(&self.slots).clone())
        }
        fn active(&self) -> Option<bool> {
            Some(
                lock(&self.slots)
                    .iter()
                    .any(|s| !s.bypass && s.health == Health::Running),
            )
        }
        fn catalogue(&self) -> Catalogue {
            Catalogue {
                scanning: false,
                scanned: *lock(&self.scans) > 0,
                unlisted: *lock(&self.unlisted),
                generation: 1,
                effects: self.found.clone(),
            }
        }
        fn preferences(&self) -> Preferences {
            lock(&self.preferences).clone()
        }
        fn set_order(&self, order: FxOrder) {
            lock(&self.preferences).order = order;
        }
        fn scan(&self, again: bool) {
            let mut n = lock(&self.scans);
            if *n == 0 || again {
                *n += 1;
            }
        }
        fn add(&self, _: u64, effect: usize) -> Result<(), String> {
            let l = self.found.get(effect).ok_or("no such effect")?;
            lock(&self.preferences).chosen(l.key.clone());
            self.note(format!("add {}", l.name));
            lock(&self.slots).push(SlotInfo {
                name: l.name.clone(),
                vendor: l.vendor.clone(),
                format: l.format.clone(),
                ..SlotInfo::default()
            });
            Ok(())
        }
        fn replace(&self, slot: usize, _: u64, effect: usize) -> Result<(), String> {
            let l = self.found.get(effect).ok_or("no such effect")?;
            lock(&self.preferences).chosen(l.key.clone());
            self.note(format!("replace {slot} {}", l.name));
            lock(&self.slots)[slot].name = l.name.clone();
            Ok(())
        }
        fn remove(&self, slot: usize) -> Result<(), String> {
            self.note(format!("remove {slot}"));
            lock(&self.slots).remove(slot);
            Ok(())
        }
        fn move_to(&self, from: usize, to: usize) -> Result<(), String> {
            self.note(format!("move {from} {to}"));
            let mut s = lock(&self.slots);
            let e = s.remove(from);
            s.insert(to, e);
            for pin in lock(&self.pins).iter_mut() {
                pin.slot = if pin.slot == from {
                    to
                } else if from < to && pin.slot > from && pin.slot <= to {
                    pin.slot - 1
                } else if to < from && pin.slot >= to && pin.slot < from {
                    pin.slot + 1
                } else {
                    pin.slot
                };
            }
            Ok(())
        }
        fn set_bypass(&self, slot: usize, on: bool) -> Result<(), String> {
            self.note(format!("bypass {slot} {on}"));
            lock(&self.slots)[slot].bypass = on;
            Ok(())
        }
        fn open_editor(&self, slot: usize) -> Result<(), String> {
            self.note(format!("window {slot}"));
            Ok(())
        }
        fn retry(&self, slot: usize) -> Result<(), String> {
            self.note(format!("retry {slot}"));
            lock(&self.slots)[slot].health = Health::Running;
            Ok(())
        }
        fn without_settings(&self, slot: usize) -> Result<(), String> {
            self.note(format!("without settings {slot}"));
            let mut s = lock(&self.slots);
            s[slot].health = Health::Running;
            s[slot].set_aside = true;
            Ok(())
        }
        fn old_settings(&self, slot: usize) -> Result<(), String> {
            self.note(format!("old settings {slot}"));
            lock(&self.slots)[slot].try_failed = true;
            Ok(())
        }
        fn turn_on(&self) {
            self.note("turn on".into());
            for s in lock(&self.slots).iter_mut() {
                if s.health == Health::Off {
                    s.health = Health::Running;
                }
            }
            *lock(&self.safe) = None;
        }
        fn safe_start(&self) -> Option<SafeStart> {
            *lock(&self.safe)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::Fake;
    use super::*;

    fn key(v: &mut FxView, key: Key) {
        assert!(v.key(&KeyboardEvent {
            key,
            state: KeyState::Down,
            ..KeyboardEvent::default()
        }));
    }

    fn wait(v: &mut FxView, fake: &Fake, jobs: u64) {
        let start = Instant::now();
        while lock(&v.pending).done < jobs && start.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(lock(&v.pending).done >= jobs);
        v.poll(Instant::now());
        assert!(!lock(&fake.done).is_empty());
    }

    #[test]
    fn find_edits_with_the_presets_field_and_escape_clears_then_closes() {
        let fake = Fake::with(&[], &["Hall", "Delay"]);
        let mut v = FxView::new(Some(fake));
        v.open_list(0, false);
        key(&mut v, Key::Character("réverb".into()));
        key(&mut v, Key::Home);
        key(&mut v, Key::Delete);
        key(&mut v, Key::ArrowRight);
        key(&mut v, Key::Backspace);
        assert_eq!(
            v.query,
            Field {
                text: "verb".into(),
                caret: 0
            }
        );
        key(&mut v, Key::End);
        key(&mut v, Key::ArrowLeft);
        key(&mut v, Key::Character("X".into()));
        assert_eq!(v.query.text, "verXb");
        key(&mut v, Key::Escape);
        assert!(v.list_open() && v.query.text.is_empty());
        key(&mut v, Key::Escape);
        assert!(!v.list_open());
        assert!(!v.key(&KeyboardEvent::default()));
    }

    #[test]
    fn keys_move_past_heads_scroll_and_pick_the_original_effect_index() {
        let names: Vec<_> = (0..25).map(|i| format!("Effect {i:02}")).collect();
        let refs: Vec<_> = names.iter().map(String::as_str).collect();
        let fake = Fake::with(&["Old"], &refs);
        let mut v = FxView::new(Some(fake.clone()));
        v.poll(Instant::now());
        v.open_list(0, true);
        v.set_order(FxOrder::Type);
        assert_eq!(v.open.and_then(|o| o.chosen), Some(1), "past the head");
        key(&mut v, Key::PageDown);
        assert_eq!(v.open.and_then(|o| o.chosen), Some(11));
        assert_eq!(v.open.map(|o| o.first), Some(2));
        key(&mut v, Key::PageUp);
        key(&mut v, Key::ArrowUp);
        assert_eq!(v.open.and_then(|o| o.chosen), Some(1));
        key(&mut v, Key::Character("17".into()));
        assert_eq!(v.rows[1].1, Choice::Effect(17));
        key(&mut v, Key::Enter);
        assert!(!v.list_open());
        wait(&mut v, &fake, 2);
        assert_eq!(fake.take(), ["replace 0 Effect 17"]);
        v.open_list(1, false);
        v.set_order(FxOrder::Recent);
        assert_eq!(v.rows[0].1, Choice::Effect(17));
        key(&mut v, Key::Enter);
        wait(&mut v, &fake, 4);
        assert_eq!(fake.take(), ["add Effect 17"]);
    }

    #[test]
    fn an_empty_scan_and_recent_actions_remain_keyboard_accessible() {
        let fake = Fake::with(&[], &[]);
        let mut v = FxView::new(Some(fake.clone()));
        v.open_list(0, false);
        key(&mut v, Key::Enter);
        assert_eq!(*lock(&fake.scans), 2);
        v.set_order(FxOrder::Recent);
        assert_eq!(
            v.open.and_then(|o| o.chosen),
            Some(1),
            "ALL EFFECTS past the blank"
        );
        key(&mut v, Key::Enter);
        assert_eq!(v.preferences.order, FxOrder::Name);
        assert!(v.list_open());
    }

    /// The real chooser’s renderer, using the approved mock-up’s effects, for visual review.
    /// `CA72_FX_SEARCH_PNG=<folder> cargo test -p ca72-plugin the_fx_search_pngs -- --ignored`.
    #[test]
    #[ignore = "writes images for a look"]
    fn the_fx_search_pngs() {
        use ca72_panel::strip::fx::FxRenderer;
        use ca72_panel::{Lettering, Skin};
        let out =
            std::path::PathBuf::from(std::env::var_os("CA72_FX_SEARCH_PNG").expect("image folder"));
        std::fs::create_dir_all(&out).expect("image folder");
        let entries = [
            ("Airwindows Consolidated", "Airwindows", "utility", "vst3"),
            ("ChowMatrix", "chowdsp", "delay", "vst3"),
            ("ChowTapeModel", "chowdsp", "distortion", "clap"),
            (
                "Dragonfly Early Reflections",
                "Michael Willis",
                "reverb",
                "clap",
            ),
            ("Dragonfly Hall Reverb", "Michael Willis", "reverb", "clap"),
            ("Dragonfly Plate Reverb", "Michael Willis", "reverb", "vst3"),
            ("Dragonfly Room Reverb", "Michael Willis", "reverb", "clap"),
            ("kHs Chorus", "Kilohearts", "chorus", "vst3"),
            ("kHs Distortion", "Kilohearts", "distortion", "vst3"),
            ("kHs Gain", "Kilohearts", "utility", "vst3"),
            ("OTT", "Xfer Records", "compressor", "vst3"),
            ("TAL-Chorus-LX", "TAL Software", "chorus", "vst3"),
            ("ValhallaDelay", "Valhalla DSP", "delay", "vst3"),
            ("ValhallaRoom", "Valhalla DSP", "reverb", "vst3"),
            ("ValhallaSupermassive", "Valhalla DSP", "reverb", "vst3"),
            ("ValhallaVintageVerb", "Valhalla DSP", "reverb", "vst3"),
        ];
        let fake = Arc::new(Fake {
            found: entries
                .iter()
                .map(|(name, maker, kind, format)| Listing {
                    name: (*name).into(),
                    vendor: (*maker).into(),
                    format: (*format).into(),
                    features: vec![(*kind).into()],
                    key: EffectId {
                        format: (*format).into(),
                        path: (*name).into(),
                        id: (*name).into(),
                    },
                })
                .collect(),
            ..Fake::default()
        });
        for i in [10, 11, 2, 9, 12, 4] {
            lock(&fake.preferences).chosen(fake.found[i].key.clone());
        }
        let mut v = FxView::new(Some(fake));
        v.open_list(3, false);
        for (skin, name) in [(Skin::Worn, "worn"), (Skin::Drawn, "drawn")] {
            for scale in [0.44, 1.0] {
                let mut r = FxRenderer::new(scale, skin, Lettering::Standard);
                for (state, order, query) in [
                    ("search", FxOrder::Name, "rev"),
                    ("maker", FxOrder::Maker, ""),
                    ("type", FxOrder::Type, ""),
                    ("recent", FxOrder::Recent, ""),
                ] {
                    v.preferences.order = order;
                    v.query = Field::new(query);
                    v.refilter();
                    if state == "search" {
                        v.navigate(1);
                    }
                    r.render(&v.scene());
                    r.under()
                        .last()
                        .expect("list")
                        .0
                        .save_png(out.join(format!("{state}-{name}-{scale}.png")))
                        .expect("image");
                }
            }
        }
    }

    /// The slots show the chain: an effect's name lit, its tab lit while it plays; one bypassed,
    /// its tab dark; one given up on, restarting or missing, its name dimmed and the reason
    /// under it; the first empty slot "+ ADD EFFECT", the rest dark; the key lit while any plays.
    #[test]
    fn the_slots_show_the_chain() {
        let fake = Fake::with(&["Chorus", "Hall", "Delay"], &[]);
        {
            let mut s = lock(&fake.slots);
            s[1].bypass = true;
            s[2].health = Health::Crashed;
        }
        let mut v = FxView::new(Some(fake.clone()));
        v.poll(Instant::now());
        let sc = v.scene();
        assert_eq!(
            (sc.slots[0].text.as_str(), sc.slots[0].lit, sc.slots[0].dim),
            ("Chorus", true, false)
        );
        assert!(!sc.slots[1].lit && !sc.slots[1].dim);
        assert_eq!(
            (sc.slots[2].dim, sc.slots[2].word),
            (true, Some(FxWord::Crashed))
        );
        assert_eq!(sc.slots[3].text, "+ ADD EFFECT");
        assert!(v.active());
        lock(&fake.slots)[0].bypass = true;
        lock(&fake.slots).truncate(2);
        v.polled = None;
        v.poll(Instant::now());
        let sc = v.scene();
        assert!(!v.active(), "nothing plays");
        assert_eq!(sc.slots[2].text, "+ ADD EFFECT");
        assert_eq!(sc.slots[3], FxSlot::default());
        lock(&fake.slots)[1].health = Health::Missing;
        v.polled = None;
        v.poll(Instant::now());
        assert_eq!(v.scene().slots[1].word, Some(FxWord::Missing));
    }

    /// The list shows the effects found by name, maker and format, and SCAN AGAIN; it scrolls no
    /// further than its last row in view; a scan is begun the first time it opens, again only
    /// from SCAN AGAIN.
    #[test]
    fn the_list_shows_the_effects_found_and_scrolls() {
        let names: Vec<String> = (0..15).map(|i| format!("Effect {i}")).collect();
        let found: Vec<&str> = names.iter().map(String::as_str).collect();
        let fake = Fake::with(&["Chorus"], &found);
        let mut v = FxView::new(Some(fake.clone()));
        v.poll(Instant::now());
        assert_eq!(
            *lock(&fake.scans),
            0,
            "nothing scanned before it is asked for"
        );
        v.open_list(1, false);
        assert_eq!(*lock(&fake.scans), 1);
        let l = v.scene().list.expect("the list");
        assert_eq!(l.rows.len(), 16);
        assert_eq!(l.rows[0].format, "VST3");
        assert!(l.rows[15].action && l.rows[15].name == "SCAN AGAIN");
        assert_eq!(l.note, "15 EFFECTS");
        v.scroll(3);
        assert_eq!(v.scene().list.map(|l| l.first), Some(3));
        v.scroll(100);
        assert_eq!(v.scene().list.map(|l| l.first), Some(16 - LIST_ROWS));
        v.scroll(-100);
        assert_eq!(v.scene().list.map(|l| l.first), Some(0));
        v.close_list();
        v.open_list(1, false);
        assert_eq!(*lock(&fake.scans), 1, "scanned once");
        v.choose(15);
        assert_eq!(*lock(&fake.scans), 2, "SCAN AGAIN");
        // Files that crashed or hung their helper as they were listed (R-FX-SAFETY) are counted.
        *lock(&fake.unlisted) = 2;
        v.open_list(1, false);
        assert_eq!(
            v.scene().list.map(|l| l.note),
            Some("15 EFFECTS, 2 NOT LISTED".into())
        );
    }
}
