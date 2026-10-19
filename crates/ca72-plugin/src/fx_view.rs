//! The effects' drawer in the editor (decisions.md R-FX-DRAWER; the owner's FX-A, 2026-10-18):
//! what it shows of the standalone's chain and what a pointer does on it. The chain is reached
//! through [`Effects`] (the standalone's, `fx.rs`; the tests' own), never waited on: what loads
//! or opens a window (seconds, for some plug-ins) is done on a thread of the drawer's, in order,
//! and a bypass on another, so that a slot switched on or off answers while another loads; the
//! chain is read only when it is free, every tenth of a second, or as soon as something done
//! there is finished.
//!
//! A click on a slot's name opens its effect's window, a drag onto another slot moves it there,
//! its tab bypasses it; a right click offers REPLACE and REMOVE (the editor's menus, MIDI
//! Learn's); the first empty slot reads "+ ADD EFFECT" and drops the effects' list, the effects
//! on this computer, found by a scan on a thread of its own the first time it is asked for.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ca72_panel::strip::fx::{
    self as fx_ui, FxList, FxRow, FxScene, FxSlot, FxTarget, FxWord, LIST_ROWS, SLOTS,
};

/// How an effect in the chain is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    Running,
    /// Its helper died or hung; it is being started again (dry meanwhile).
    Restarting,
    /// Given up on: dry from now on.
    Failed,
    /// Its plug-in could not be loaded here: dry, its state kept.
    Missing,
}

/// An effect in the chain, as the drawer shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct SlotInfo {
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub bypass: bool,
    pub health: Health,
}

/// An effect found on this computer.
#[derive(Clone, Debug, PartialEq)]
pub struct Listing {
    pub name: String,
    pub vendor: String,
    pub format: String,
}

/// The effects found: whether a scan is under way, whether one has finished, which scan this
/// list is (each scan's own number), and the effects.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalogue {
    pub scanning: bool,
    pub scanned: bool,
    pub generation: u64,
    pub effects: Vec<Listing>,
}

/// The chain as the drawer reaches it. What may take a while (loading, a window) is called on
/// the drawer's threads, never the editor's; the rest returns at once.
pub trait Effects: Send + Sync {
    /// The chain's effects in order, or none if it is busy now.
    fn slots(&self) -> Option<Vec<SlotInfo>>;
    /// Whether any effect plays (the FX key's light), or none if the chain is busy now.
    fn active(&self) -> Option<bool>;
    /// The effects found so far.
    fn catalogue(&self) -> Catalogue;
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
}

/// What the drawer's threads are doing and have done: an effect being loaded (its slot and
/// name), one that would not load (its slot, name and when), and how many jobs are finished.
#[derive(Debug, Default)]
struct Pending {
    loading: Option<(usize, String)>,
    failed: Option<(usize, String, Instant)>,
    done: u64,
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
    press: Option<Press>,
    pub hover: Option<FxTarget>,
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
            press: None,
            hover: None,
        }
    }

    /// Whether there is a chain (the standalone's): the rail has its FX key.
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
            self.catalogue = e.catalogue();
            if let Some(o) = &mut self.open {
                o.first = o.first.min(rows(&self.catalogue).saturating_sub(LIST_ROWS));
            }
        }
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
                    Health::Running => (false, None),
                    Health::Restarting => (true, Some(FxWord::Restarting)),
                    Health::Failed => (true, Some(FxWord::Crashed)),
                    Health::Missing => (true, Some(FxWord::Missing)),
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
        FxScene {
            slots,
            hover: self
                .hover
                .filter(|h| matches!(h, FxTarget::Name(_) | FxTarget::Tab(_))),
            drag: self.press.filter(|p| p.carried).map(|p| (p.slot, p.over)),
            list: self.open.map(|o| self.list(o)),
        }
    }

    /// The list as it shows for `o`.
    fn list(&self, o: Open) -> FxList {
        let c = &self.catalogue;
        let mut rows: Vec<FxRow> = c
            .effects
            .iter()
            .map(|l| FxRow {
                name: l.name.clone(),
                maker: l.vendor.clone(),
                format: l.format.to_uppercase(),
                action: false,
            })
            .collect();
        if !c.scanning {
            rows.push(FxRow {
                name: "SCAN AGAIN".into(),
                action: true,
                ..FxRow::default()
            });
        }
        let n = c.effects.len();
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
            } else if n == 0 {
                "NONE FOUND".into()
            } else if n == 1 {
                "1 EFFECT".into()
            } else {
                format!("{n} EFFECTS")
            },
            rows,
            first: o.first,
            hover: o.hover,
        }
    }

    /// Whether the effects' list is open.
    pub fn list_open(&self) -> bool {
        self.open.is_some()
    }

    /// The effects' list dropped for slot `slot`, to add an effect there or to replace its own;
    /// the effects found the first time it is asked for.
    pub fn open_list(&mut self, slot: usize, replace: bool) {
        self.press = None;
        self.open = Some(Open {
            slot: slot.min(SLOTS - 1),
            replace,
            first: 0,
            hover: None,
        });
        if let Some(e) = &self.effects {
            e.scan(false);
            self.catalogue = e.catalogue();
        }
    }

    pub fn close_list(&mut self) {
        self.open = None;
    }

    /// The list scrolled by `rows` (+ down).
    pub fn scroll(&mut self, rows_by: i32) {
        let last = rows(&self.catalogue).saturating_sub(LIST_ROWS);
        if let Some(o) = &mut self.open {
            o.first = o.first.saturating_add_signed(rows_by as isize).min(last);
        }
    }

    /// The list as it shows (for the hit test).
    fn shown_list(&self) -> Option<FxList> {
        self.open.map(|o| self.list(o))
    }

    /// A press at (`x`, `y`) (drawing units) on the drawer or its list.
    pub fn press(&mut self, x: f64, y: f64) {
        let list = self.shown_list();
        if let Some(l) = &list {
            match fx_ui::hit(x, y, Some(l)) {
                Some(FxTarget::Row(r)) => self.choose(r),
                Some(FxTarget::List) => {}
                _ => self.open = None,
            }
            return;
        }
        match fx_ui::hit(x, y, None) {
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
        let found = fx_ui::hit(x, y, list.as_ref());
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
    /// its effect's window opened.
    pub fn released(&mut self) {
        let Some(p) = self.press.take() else { return };
        match (p.carried, p.over) {
            (true, Some(to)) => self.submit(Job::Move(p.slot, to)),
            (true, None) => {}
            (false, _) => self.submit(Job::Window(p.slot)),
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

    /// The slot whose menu a right click at (`x`, `y`) opens, and its effect's name.
    pub fn menu_for(&self, x: f64, y: f64) -> Option<(usize, String)> {
        match fx_ui::hit(x, y, None) {
            Some(FxTarget::Name(i) | FxTarget::Tab(i)) if !self.busy() => {
                self.effect(i).map(|e| (i, e.name.clone()))
            }
            _ => None,
        }
    }

    /// The menu's REPLACE: the list dropped to replace slot `slot`'s effect.
    pub fn replace(&mut self, slot: usize) {
        if self.effect(slot).is_some() {
            self.open_list(slot, true);
        }
    }

    /// The menu's REMOVE.
    pub fn remove(&mut self, slot: usize) {
        if self.effect(slot).is_some() {
            self.submit(Job::Remove(slot));
        }
    }

    /// The tip over slot `i`'s name: its effect's whole name and maker.
    pub fn tip(&self) -> Option<(String, f64, f64)> {
        let Some(FxTarget::Name(i)) = self.hover else {
            return None;
        };
        if self.press.is_some() {
            return None;
        }
        let e = self.effect(i)?;
        let (x, y, w, _) = fx_ui::screen(i);
        let text = if e.vendor.is_empty() {
            e.name.clone()
        } else {
            format!("{} ({})", e.name, e.vendor)
        };
        Some((text, x + w / 2.0, y - 6.0))
    }

    /// Row `row` of the list chosen: its effect added or put in place; or the computer
    /// scanned again.
    fn choose(&mut self, row: usize) {
        let Some(o) = self.open else { return };
        let c = &self.catalogue;
        if row == c.effects.len() && !c.scanning {
            if let Some(e) = &self.effects {
                e.scan(true);
                self.catalogue = e.catalogue();
            }
            return;
        }
        let Some(l) = c.effects.get(row) else { return };
        let (generation, name) = (c.generation, l.name.clone());
        self.open = None;
        let at = if o.replace { o.slot } else { self.slots.len() };
        lock(&self.pending).loading = Some((at, name));
        if o.replace {
            self.submit(Job::Replace(o.slot, generation, row));
        } else {
            self.submit(Job::Add(generation, row));
        }
    }

    /// `job` handed to its thread (a bypass to the quick one, the rest in order to the other).
    fn submit(&mut self, job: Job) {
        let lane = if matches!(job, Job::Bypass(..)) {
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

/// How many rows the list has: the effects, and SCAN AGAIN.
fn rows(c: &Catalogue) -> usize {
    c.effects.len() + usize::from(!c.scanning)
}

/// `job` done to `effects`, on one of the drawer's threads; what it did kept in `pending`.
fn run(effects: &dyn Effects, pending: &Mutex<Pending>, job: Job) {
    let result = match &job {
        Job::Add(g, i) => effects.add(*g, *i),
        Job::Replace(s, g, i) => effects.replace(*s, *g, *i),
        Job::Remove(s) => effects.remove(*s),
        Job::Move(f, t) => effects.move_to(*f, *t),
        Job::Bypass(s, on) => effects.set_bypass(*s, *on),
        Job::Window(s) => effects.open_editor(*s),
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
    p.done += 1;
    drop(p);
    if let Err(e) = result {
        eprintln!("CA-72 effects: {e}");
    }
}

#[cfg(test)]
pub(crate) mod fake {
    //! A chain for the tests: what is done to it is done at once and noted.

    use super::*;

    #[derive(Debug, Default)]
    pub struct Fake {
        pub slots: Mutex<Vec<SlotInfo>>,
        pub found: Vec<Listing>,
        pub done: Mutex<Vec<String>>,
        pub scans: Mutex<u32>,
    }

    impl Fake {
        pub fn with(names: &[&str], found: &[&str]) -> Arc<Fake> {
            let info = |n: &&str| SlotInfo {
                name: (*n).into(),
                vendor: "Maker".into(),
                format: "clap".into(),
                bypass: false,
                health: Health::Running,
            };
            Arc::new(Fake {
                slots: Mutex::new(names.iter().map(info).collect()),
                found: found
                    .iter()
                    .map(|n| Listing {
                        name: (*n).into(),
                        vendor: "Maker".into(),
                        format: "vst3".into(),
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

    impl Effects for Fake {
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
                generation: 1,
                effects: self.found.clone(),
            }
        }
        fn scan(&self, again: bool) {
            let mut n = lock(&self.scans);
            if *n == 0 || again {
                *n += 1;
            }
        }
        fn add(&self, _: u64, effect: usize) -> Result<(), String> {
            let l = self.found.get(effect).ok_or("no such effect")?;
            self.note(format!("add {}", l.name));
            lock(&self.slots).push(SlotInfo {
                name: l.name.clone(),
                vendor: l.vendor.clone(),
                format: l.format.clone(),
                bypass: false,
                health: Health::Running,
            });
            Ok(())
        }
        fn replace(&self, slot: usize, _: u64, effect: usize) -> Result<(), String> {
            let l = self.found.get(effect).ok_or("no such effect")?;
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
    }
}

#[cfg(test)]
mod tests {
    use super::fake::Fake;
    use super::*;

    /// The slots show the chain: an effect's name lit, its tab lit while it plays; one bypassed,
    /// its tab dark; one given up on, restarting or missing, its name dimmed and the reason
    /// under it; the first empty slot "+ ADD EFFECT", the rest dark; the key lit while any plays.
    #[test]
    fn the_slots_show_the_chain() {
        let fake = Fake::with(&["Chorus", "Hall", "Delay"], &[]);
        {
            let mut s = lock(&fake.slots);
            s[1].bypass = true;
            s[2].health = Health::Failed;
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
    }
}
