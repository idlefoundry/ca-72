//! Watching a running plugin for changes it makes without a word (D234), and telling which
//! of its changes bring its state into the project.
//!
//! - **Its values are polled** on the helper's main thread while its editor is open (where a
//!   person loads presets): at most every [`EVERY`], and never more often than a hundred
//!   times the last poll's cost, so that polling takes a hundredth of the main thread at
//!   most. A value that moved by the plugin's own doing (not an edit of its editor's, not a
//!   value the host gave it, D222) is taken once it has held still for a poll: a value the
//!   plugin keeps moving (a display, an internal modulation) never is.
//! - **Its state is polled** too when saving it is cheap (under [`STATE_COST`]): at most
//!   every [`STATE_EVERY`] and a hundred times its cost. A state that changed while no value
//!   moved and the editor edited nothing since it was last read changed alone (a sample
//!   loaded, a wavetable drawn); it is taken once it has held still for a state poll.
//! - **A reading** (D228: the values read once the plugin has been quiet after its word, or
//!   after a poll found a change) brings its state along when values moved by its own doing
//!   since the last reading, or its state changed alone, as what it said allows ([`Said`]).
//!
//! The values compared are a parameter's bits: the helper reads them from the plugin each
//! time, and the host's values are those it gave the plugin.

use daw_ipc::protocol::{EditKind, ParamEdit};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hasher;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The values are polled at most this often.
pub const EVERY: Duration = Duration::from_millis(250);
/// The state is polled at most this often...
pub const STATE_EVERY: Duration = Duration::from_secs(1);
/// ...and only when saving it took less than this (a save is on the plugin's main thread,
/// where its editor draws).
pub const STATE_COST: Duration = Duration::from_millis(5);
/// A poll waits at least this many times its own cost before the next.
const DUTY: u32 = 100;

/// What a reading follows (D234): how much of a change of its state alone the plugin said.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Said {
    /// Nothing of its state (D228's word of its values, a poll, its processor's own values):
    /// its state changed alone only if no value moved at all and nothing was edited, as the
    /// host's values change a state too.
    Values,
    /// That its state changed (CLAP's `mark_dirty`, which many plugins say at every edit of
    /// their own): alone if nothing moved by its own doing and nothing was edited; the
    /// host's values do not hide it.
    State,
    /// That something besides its parameters changed (VST3's `IComponentHandler2::setDirty`):
    /// alone if nothing moved by its own doing, whatever else did.
    Besides,
}

/// A state's hash, to tell whether it changed (not stored, not sent).
pub fn hash(state: &[u8]) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    h.write(state);
    h.finish()
}

/// The values the host gave a plugin, recorded where they are given (its audio thread
/// included: no allocation, no waiting, a binary search and two atomic stores per value)
/// for its main thread's watch (D234).
#[derive(Debug)]
pub struct Given {
    /// Parameter IDs, sorted.
    ids: Vec<u32>,
    /// By index in `ids`: the latest value's bits (`u64::MAX`: none), and whether it is new.
    values: Vec<AtomicU64>,
    fresh: Vec<AtomicBool>,
    any: AtomicBool,
}

impl Given {
    pub fn new(mut ids: Vec<u32>) -> Given {
        ids.sort_unstable();
        ids.dedup();
        Given {
            values: ids.iter().map(|_| AtomicU64::new(u64::MAX)).collect(),
            fresh: ids.iter().map(|_| AtomicBool::new(false)).collect(),
            ids,
            any: AtomicBool::new(false),
        }
    }

    /// The host gave parameter `id` the value `v`.
    pub fn set(&self, id: u32, v: f64) {
        if let Ok(i) = self.ids.binary_search(&id) {
            self.values[i].store(v.to_bits(), Ordering::Relaxed);
            self.fresh[i].store(true, Ordering::Release);
            self.any.store(true, Ordering::Release);
        }
    }

    /// The value the host last gave parameter `id`.
    pub fn value(&self, id: u32) -> Option<f64> {
        let i = self.ids.binary_search(&id).ok()?;
        let b = self.values[i].load(Ordering::Relaxed);
        (b != u64::MAX).then(|| f64::from_bits(b))
    }

    /// The parameters given a value since the last call.
    pub fn take(&self) -> Vec<u32> {
        if !self.any.swap(false, Ordering::AcqRel) {
            return Vec::new();
        }
        self.ids
            .iter()
            .zip(&self.fresh)
            .filter(|(_, f)| f.swap(false, Ordering::AcqRel))
            .map(|(id, _)| *id)
            .collect()
    }
}

/// What changed since a mark that is no change of the plugin's own: its editor's edits and
/// the host's values.
#[derive(Debug, Default, Clone)]
struct Explained {
    edited: BTreeMap<u32, Option<f64>>,
    host: BTreeMap<u32, Option<f64>>,
}

impl Explained {
    fn clear(&mut self) {
        self.edited.clear();
        self.host.clear();
    }

    /// Whether parameter `id` at `v` is explained: its editor's value, the observed host
    /// value, or the host's current value (`host`, to an `f32`'s precision: a plugin may keep its values so; the
    /// review's finding).
    fn explains(&self, id: u32, v: f64, host: &dyn Fn(u32) -> Option<f64>) -> bool {
        let current = host(id);
        self.edited
            .get(&id)
            .is_some_and(|edited| edited.is_none_or(|x| near(x, v)))
            || self.host.get(&id).is_some_and(|h| {
                h.is_none_or(|observed| {
                    near(observed, v)
                        // CLAP audio can advance several values while a getter is
                        // preempted. A moving host explains an intermediate snapshot;
                        // a stable older host value cannot hide a new plugin value.
                        || current.is_some_and(|now| !near(observed, now))
                })
            })
            || current.is_some_and(|h| near(h, v))
    }
}

/// The plugin's values and state at its last reading.
#[derive(Debug, Clone)]
struct Mark {
    values: BTreeMap<u32, u64>,
    /// Its state's hash then; None: not read with these values.
    state: Option<u64>,
}

/// What a reading found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    /// Values moved by the plugin's own doing since the last reading.
    pub own: bool,
    /// Its state changed with no value or edit since the last reading.
    pub alone: bool,
}

impl Reading {
    /// Whether its state goes with the values (D234).
    pub fn with_state(self) -> bool {
        self.own || self.alone
    }
}

/// How polling a plugin went, for `plugin.window` (D234).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Costs {
    pub polls: u64,
    pub values: Duration,
    pub every: Duration,
    pub state: Option<Duration>,
    pub state_bytes: usize,
    pub state_polled: bool,
}

/// One running plugin's watch.
#[derive(Debug, Clone)]
pub struct Watch {
    next: Instant,
    every: Duration,
    state_next: Instant,
    /// The values as the last poll saw them.
    seen: BTreeMap<u32, u64>,
    /// Values found moved by the plugin itself at the last poll, waiting to hold still.
    moving: BTreeSet<u32>,
    /// A state found changed alone at the last state poll (its hash), waiting to hold still.
    state_moving: Option<u64>,
    since_poll: Explained,
    since_mark: Explained,
    mark: Mark,
    costs: Costs,
    /// How long its last three saves took: its state is cheap while the least is under
    /// [`STATE_COST`] (one slow save on a busy machine does not end its polls).
    saves: [Option<Duration>; 3],
}

/// Whether `a` and `b` are one value to an `f32`'s precision.
pub fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6 * a.abs().max(b.abs()).max(1.0)
}

fn bits(values: &[(u32, f64)]) -> BTreeMap<u32, u64> {
    values.iter().map(|(id, v)| (*id, v.to_bits())).collect()
}

impl Watch {
    /// A plugin made active (or given a state by the host) at `now`: its values, and its
    /// state if it could be saved, with how long that took.
    pub fn new(now: Instant, values: &[(u32, f64)], state: Option<(&[u8], Duration)>) -> Watch {
        let mut w = Watch {
            next: now + EVERY,
            every: EVERY,
            state_next: now + STATE_EVERY,
            seen: BTreeMap::new(),
            moving: BTreeSet::new(),
            state_moving: None,
            since_poll: Explained::default(),
            since_mark: Explained::default(),
            mark: Mark {
                values: BTreeMap::new(),
                state: None,
            },
            costs: Costs::default(),
            saves: [None; 3],
        };
        w.reset(now, values, state);
        w
    }

    /// A save of its state took `cost` and gave `bytes`.
    fn saved(&mut self, cost: Duration, bytes: usize) {
        self.saves.rotate_right(1);
        self.saves[0] = Some(cost);
        self.costs.state = Some(cost);
        self.costs.state_bytes = bytes;
    }

    /// Whether its state is cheap to save: the least of its last saves took under
    /// [`STATE_COST`] (or none was timed yet).
    pub fn cheap(&self) -> bool {
        self.saves
            .iter()
            .flatten()
            .min()
            .is_none_or(|c| *c < STATE_COST)
    }

    /// The host gave it a state (or made it active): what it has now is the mark, and
    /// nothing is waiting.
    pub fn reset(&mut self, now: Instant, values: &[(u32, f64)], state: Option<(&[u8], Duration)>) {
        self.seen = bits(values);
        self.moving.clear();
        self.state_moving = None;
        self.since_poll.clear();
        self.since_mark.clear();
        self.mark = Mark {
            values: self.seen.clone(),
            state: state.map(|s| hash(s.0)),
        };
        if let Some((s, cost)) = state {
            self.saved(cost, s.len());
        }
        self.next = now + self.every;
        self.state_next = now + STATE_EVERY;
    }

    /// Its editor's edits (D222): the values they set are not changes of its own.
    pub fn note_edits(&mut self, edits: &[ParamEdit]) {
        for e in edits.iter().filter(|e| e.kind == EditKind::Value) {
            self.since_poll.edited.insert(e.param, e.value);
            self.since_mark.edited.insert(e.param, e.value);
        }
    }

    /// Values the host gave it, as observed on the helper's control thread. Keep one
    /// value per ID as well as the live host lookup: a getter snapshot can precede a
    /// newer audio update, but an older host ID cannot explain a different plugin value.
    /// An unavailable value retains conservative attribution for that unknown input.
    pub fn note_host(&mut self, ids: &[u32], host: &dyn Fn(u32) -> Option<f64>) {
        for id in ids {
            let value = host(*id);
            self.since_poll.host.insert(*id, value);
            self.since_mark.host.insert(*id, value);
        }
    }

    /// Whether a poll of its values is due.
    pub fn due(&self, now: Instant) -> bool {
        now >= self.next
    }

    /// Whether its state is polled now: it is cheap to save (or its cost is not known yet)
    /// and a state poll is due.
    pub fn state_due(&self, now: Instant) -> bool {
        self.cheap() && now >= self.state_next
    }

    /// A poll of its values, read in `cost`: whether the plugin moved one itself and it has
    /// held still since (a change to take, as its word would be).
    pub fn poll(
        &mut self,
        now: Instant,
        values: &[(u32, f64)],
        cost: Duration,
        host: &dyn Fn(u32) -> Option<f64>,
    ) -> bool {
        self.costs.polls += 1;
        self.costs.values = cost;
        self.every = EVERY.max(cost * DUTY);
        self.costs.every = self.every;
        self.next = now + self.every;
        let mut held = false;
        for (id, v) in values {
            let b = v.to_bits();
            let before = self.seen.insert(*id, b);
            let moved = before.is_some_and(|x| x != b);
            if !moved {
                // Moved by itself at the last poll, and still since: taken.
                held |= self.moving.remove(id);
                continue;
            }
            if self.since_poll.explains(*id, *v, host) {
                self.moving.remove(id);
            } else {
                self.moving.insert(*id);
            }
        }
        self.since_poll.clear();
        held
    }

    /// A poll of its state (`state`, saved in `cost`) with its values now: whether its
    /// state changed alone and has held still since (a change to take). Otherwise, with
    /// nothing of its own since the mark, the mark moves here, so that a state changing
    /// alone later is told from what the host and its editor did before.
    pub fn poll_state(
        &mut self,
        now: Instant,
        values: &[(u32, f64)],
        state: &[u8],
        cost: Duration,
        host: &dyn Fn(u32) -> Option<f64>,
    ) -> bool {
        self.saved(cost, state.len());
        self.costs.state_polled = true;
        self.state_next = now + STATE_EVERY.max(cost * DUTY);
        let h = hash(state);
        let r = self.assess(values, Some(h), Said::Values, host);
        if r.own {
            // The values' poll takes it.
            self.state_moving = None;
            return false;
        }
        if r.alone {
            let held = self.state_moving == Some(h);
            self.state_moving = Some(h);
            return held;
        }
        self.state_moving = None;
        self.mark = Mark {
            values: bits(values),
            state: Some(h),
        };
        self.since_mark.clear();
        false
    }

    /// A settled preset reading carries values only when at least one moved by the
    /// plugin's own doing. Otherwise a full snapshot contains only unchanged or host
    /// values, which may already be stale when the DAW drains the notice.
    pub fn preset_values(
        &self,
        values: &[(u32, f64)],
        host: &dyn Fn(u32) -> Option<f64>,
    ) -> Vec<(u32, f64)> {
        if self.moved(values, host).0 {
            values.to_vec()
        } else {
            Vec::new()
        }
    }

    /// What moved since the mark: whether values moved by the plugin's own doing, and
    /// whether any value moved at all.
    fn moved(&self, values: &[(u32, f64)], host: &dyn Fn(u32) -> Option<f64>) -> (bool, bool) {
        let mut own = false;
        let mut same = values.len() == self.mark.values.len();
        for (id, v) in values {
            let moved = self.mark.values.get(id).is_none_or(|m| *m != v.to_bits());
            if moved {
                same = false;
                if !self.since_mark.explains(*id, *v, host) {
                    own = true;
                }
            }
        }
        (own, same)
    }

    /// What changed since the mark: values the plugin moved itself, or its state (its hash,
    /// if read) alone, as what it `said` allows ([`Said`]).
    fn assess(
        &self,
        values: &[(u32, f64)],
        state: Option<u64>,
        said: Said,
        host: &dyn Fn(u32) -> Option<f64>,
    ) -> Reading {
        let (own, same) = self.moved(values, host);
        let edited = !self.since_mark.edited.is_empty();
        let differs = |m: Option<u64>| state.is_some() && m != state;
        let alone = !own
            && match said {
                Said::Values => {
                    same && !edited
                        && self.since_mark.host.is_empty()
                        && self.mark.state.is_some()
                        && differs(self.mark.state)
                }
                Said::State => !edited && self.mark.state.is_some() && differs(self.mark.state),
                Said::Besides => differs(self.mark.state),
            };
        Reading { own, alone }
    }

    /// Whether a reading after `said` may bring its state, before it is saved (a costly
    /// state is saved only then; the review's finding): values moved by its own doing, or
    /// nothing hides a change of its state alone.
    pub fn may_bring_state(
        &self,
        values: &[(u32, f64)],
        said: Said,
        host: &dyn Fn(u32) -> Option<f64>,
    ) -> bool {
        let (own, same) = self.moved(values, host);
        let edited = !self.since_mark.edited.is_empty();
        own || match said {
            Said::Values => same && !edited && self.since_mark.host.is_empty(),
            Said::State => !edited,
            Said::Besides => true,
        }
    }

    /// A reading (D228's, once the plugin is quiet) after what it `said`: its values and its
    /// state, if it was saved (in `cost`). Says whether its state goes along; the reading
    /// is the mark from now on.
    pub fn read(
        &mut self,
        now: Instant,
        values: &[(u32, f64)],
        state: Option<(&[u8], Duration)>,
        said: Said,
        host: &dyn Fn(u32) -> Option<f64>,
    ) -> Reading {
        let r = self.assess(values, state.map(|s| hash(s.0)), said, host);
        self.reset(now, values, state);
        r
    }

    pub fn costs(&self) -> Costs {
        self.costs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: u32) -> Option<f64> {
        None
    }

    fn ms(t0: Instant, n: u64) -> Instant {
        t0 + Duration::from_millis(n)
    }

    /// A value the plugin moved itself is taken once it has held still for a poll; one it
    /// keeps moving never is; the host's values and its editor's edits are not its own.
    #[test]
    fn a_value_the_plugin_moves_itself_is_taken_once_it_holds_still() {
        let t0 = Instant::now();
        let d = Duration::from_micros(50);
        let mut w = Watch::new(t0, &[(1, 0.5), (2, 0.5), (3, 0.5)], None);
        assert!(!w.due(ms(t0, 100)));
        assert!(w.due(ms(t0, 250)));
        assert!(!w.poll(ms(t0, 250), &[(1, 0.5), (2, 0.5), (3, 0.5)], d, &none));
        // It moves 1 itself; the host gives 2; its editor edits 3.
        w.note_host(&[2], &none);
        w.note_edits(&[ParamEdit {
            param: 3,
            kind: EditKind::Value,
            value: Some(0.7),
        }]);
        assert!(
            !w.poll(ms(t0, 500), &[(1, 0.8), (2, 0.6), (3, 0.7)], d, &none),
            "moved"
        );
        assert!(
            w.poll(ms(t0, 750), &[(1, 0.8), (2, 0.6), (3, 0.7)], d, &none),
            "held"
        );
        assert!(
            !w.poll(ms(t0, 1000), &[(1, 0.8), (2, 0.6), (3, 0.7)], d, &none),
            "once"
        );
        // The host's value alone, held: not the plugin's.
        w.note_host(&[2], &none);
        assert!(!w.poll(ms(t0, 1010), &[(1, 0.8), (2, 0.65), (3, 0.7)], d, &none));
        assert!(
            !w.poll(ms(t0, 1020), &[(1, 0.8), (2, 0.65), (3, 0.7)], d, &none),
            "the host's"
        );
        // Kept moving: never.
        for k in 0..10u32 {
            let v = 0.1 * f64::from(k);
            assert!(!w.poll(
                ms(t0, 1250 + 250 * u64::from(k)),
                &[(1, v), (2, 0.65), (3, 0.7)],
                d,
                &none
            ));
        }
        // The host's last value is the host's, even without a word of it this poll.
        let host = |id: u32| (id == 2).then_some(0.9);
        assert!(!w.poll(ms(t0, 5000), &[(1, 0.95), (2, 0.9), (3, 0.7)], d, &host));
        assert!(
            w.poll(ms(t0, 5250), &[(1, 0.95), (2, 0.9), (3, 0.7)], d, &host),
            "1 held"
        );
    }

    /// Polls wait at least a hundred times their cost.
    #[test]
    fn a_costly_poll_waits_longer() {
        let t0 = Instant::now();
        let mut w = Watch::new(t0, &[], None);
        w.poll(t0, &[], Duration::from_millis(5), &none);
        assert!(!w.due(ms(t0, 499)));
        assert!(w.due(ms(t0, 500)));
        assert_eq!(w.costs().every, Duration::from_millis(500));
    }

    /// A reading brings the state along when values moved by the plugin's own doing, or its
    /// state changed alone as what it said allows: after a word of its values only if
    /// nothing else moved; after `mark_dirty` whatever the host moved, but not after an
    /// edit; after `setDirty` whatever else moved.
    #[test]
    fn a_reading_brings_the_state_when_the_plugin_changed_it() {
        let t0 = Instant::now();
        let d = Duration::from_micros(10);
        let (s0, s1, s2, s3): (&[u8], &[u8], &[u8], &[u8]) = (b"zero", b"one", b"two", b"three");
        let edit = |v| {
            [ParamEdit {
                param: 1,
                kind: EditKind::Value,
                value: Some(v),
            }]
        };
        let mut w = Watch::new(t0, &[(1, 0.5)], Some((s0, d)));
        // Its own preset: the value and the state.
        let r = w.read(t0, &[(1, 0.8)], Some((s1, d)), Said::Values, &none);
        assert_eq!(
            r,
            Reading {
                own: true,
                alone: false
            }
        );
        // Its editor's edit (the state changes with it), even with `mark_dirty`: nothing.
        w.note_edits(&edit(0.6));
        let r = w.read(t0, &[(1, 0.6)], Some((s2, d)), Said::State, &none);
        assert!(!r.with_state(), "{r:?}");
        // The host's value, the state with it, after a word of its values: nothing.
        w.note_host(&[1], &none);
        let r = w.read(t0, &[(1, 0.4)], Some((s0, d)), Said::Values, &none);
        assert!(!r.with_state(), "{r:?}");
        // After `mark_dirty` the host's value does not hide its state alone.
        w.note_host(&[1], &none);
        let r = w.read(t0, &[(1, 0.3)], Some((s1, d)), Said::State, &none);
        assert_eq!(
            r,
            Reading {
                own: false,
                alone: true
            }
        );
        // After `setDirty` neither does an edit.
        w.note_edits(&edit(0.2));
        let r = w.read(t0, &[(1, 0.2)], Some((s2, d)), Said::Besides, &none);
        assert_eq!(
            r,
            Reading {
                own: false,
                alone: true
            }
        );
        // Nothing moved, its state did: alone.
        let r = w.read(t0, &[(1, 0.2)], Some((s3, d)), Said::Values, &none);
        assert_eq!(
            r,
            Reading {
                own: false,
                alone: true
            }
        );
        // The same state again: nothing.
        let r = w.read(t0, &[(1, 0.2)], Some((s3, d)), Said::State, &none);
        assert!(!r.with_state(), "{r:?}");
        // Not read: never alone.
        let r = w.read(t0, &[(1, 0.2)], None, Said::Besides, &none);
        assert!(!r.with_state(), "{r:?}");
        // Whether it may bring its state before a costly one is saved.
        assert!(
            w.may_bring_state(&[(1, 0.9)], Said::Values, &none),
            "its own value"
        );
        assert!(
            w.may_bring_state(&[(1, 0.2)], Said::Values, &none),
            "nothing moved"
        );
        w.note_host(&[1], &none);
        assert!(
            !w.may_bring_state(&[(1, 0.1)], Said::Values, &none),
            "the host's"
        );
        assert!(w.may_bring_state(&[(1, 0.1)], Said::State, &none));
        w.note_edits(&edit(0.1));
        assert!(
            !w.may_bring_state(&[(1, 0.1)], Said::State, &none),
            "edited"
        );
        assert!(w.may_bring_state(&[(1, 0.1)], Said::Besides, &none));
    }

    /// The host's value to an `f32`'s precision is the host's (a plugin that keeps `f32`).
    #[test]
    fn the_hosts_value_is_known_to_an_f32s_precision() {
        let t0 = Instant::now();
        let d = Duration::from_micros(10);
        let host = |_: u32| Some(f64::from(0.3f32));
        let mut w = Watch::new(t0, &[(1, 0.5)], None);
        assert!(!w.poll(ms(t0, 250), &[(1, 0.3)], d, &host));
        assert!(!w.poll(ms(t0, 500), &[(1, 0.3)], d, &host), "the host's");
        assert!(near(0.3, f64::from(0.3f32)) && !near(0.3, 0.3001));
    }

    /// A state is cheap while the least of its last three saves took under 5 ms: one slow
    /// save does not end its polls, three do.
    #[test]
    fn one_slow_save_does_not_end_a_states_polls() {
        let t0 = Instant::now();
        let fast = Duration::from_micros(50);
        let slow = Duration::from_millis(8);
        let v = [(1, 0.5)];
        let mut w = Watch::new(t0, &v, Some((b"a", fast)));
        w.poll_state(ms(t0, 1000), &v, b"a", slow, &none);
        assert!(w.cheap(), "one slow save");
        w.poll_state(ms(t0, 2000), &v, b"a", slow, &none);
        assert!(w.cheap());
        w.poll_state(ms(t0, 3000), &v, b"a", slow, &none);
        assert!(!w.cheap(), "three");
        assert!(!w.state_due(ms(t0, 60_000)));
    }

    /// A state polled that changed alone is taken once it has held still for a state poll;
    /// one that keeps changing never is; one changed with the host's values moves the mark.
    #[test]
    fn a_state_that_changes_alone_is_taken_once_it_holds_still() {
        let t0 = Instant::now();
        let d = Duration::from_micros(10);
        let v = [(1, 0.5)];
        let mut w = Watch::new(t0, &v, Some((b"a", d)));
        assert!(!w.state_due(ms(t0, 999)));
        assert!(w.state_due(ms(t0, 1000)));
        assert!(!w.poll_state(ms(t0, 1000), &v, b"a", d, &none));
        assert!(!w.poll_state(ms(t0, 2000), &v, b"b", d, &none), "changed");
        assert!(w.poll_state(ms(t0, 3000), &v, b"b", d, &none), "held");
        // Read (the word it stands for): the mark moves.
        assert!(
            w.read(ms(t0, 3100), &v, Some((b"b", d)), Said::Values, &none)
                .alone
        );
        // Keeps changing: never.
        for (k, s) in [b"c", b"d", b"e", b"f"].iter().enumerate() {
            assert!(!w.poll_state(ms(t0, 4000 + 1000 * k as u64), &v, *s, d, &none));
        }
        // The host's value changed the state: the mark moves with it.
        let mut w = Watch::new(t0, &v, Some((b"a", d)));
        w.note_host(&[1], &none);
        assert!(!w.poll_state(ms(t0, 1000), &[(1, 0.7)], b"x", d, &none));
        assert!(!w.poll_state(ms(t0, 2000), &[(1, 0.7)], b"x", d, &none));
        assert!(
            !w.poll_state(ms(t0, 3000), &[(1, 0.7)], b"y", d, &none),
            "changed alone"
        );
        assert!(
            w.poll_state(ms(t0, 4000), &[(1, 0.7)], b"y", d, &none),
            "held"
        );
        // A costly state is not polled.
        let w = Watch::new(t0, &v, Some((b"a", Duration::from_millis(6))));
        assert!(!w.state_due(ms(t0, 60_000)));
    }
}

#[cfg(test)]
mod reading_tests {
    use super::*;

    #[test]
    fn an_independent_value_after_a_host_restore_is_not_swallowed() {
        let now = Instant::now();
        let cost = Duration::from_micros(10);
        let mut watch = Watch::new(now, &[(1, 0.7)], Some((b"restored", cost)));
        watch.note_host(&[1], &|id| (id == 1).then_some(0.7));
        let host = |id| (id == 1).then_some(0.7);
        // Both host windows expired, but no subsequent state poll cleared the mark.
        // A processor or a quiet preset now changes the same parameter itself.
        let values = [(1, 0.3)];
        assert_eq!(
            watch.preset_values(&values, &host),
            values,
            "the old host ID must not hide a different independent value"
        );
        assert!(!watch.poll(now, &values, cost, &host));
        assert!(
            watch.poll(now, &values, cost, &host),
            "the quiet value held"
        );
        let r = watch.read(
            now,
            &values,
            Some((b"independent", cost)),
            Said::Values,
            &host,
        );
        assert!(
            r.own && !r.alone,
            "the processor's new value brings its state: {r:?}"
        );
    }

    #[test]
    fn a_host_value_captured_before_a_newer_audio_update_is_still_explained() {
        let now = Instant::now();
        let cost = Duration::from_micros(10);
        let mut watch = Watch::new(now, &[(1, 0.5)], Some((b"initial", cost)));
        watch.note_host(&[1], &|id| (id == 1).then_some(0.7));
        // The helper saw host 0.7; the audio thread has reached 0.8 by the time
        // the earlier getter snapshot is assessed. It is still host automation.
        let values = [(1, 0.7)];
        let host = |id| (id == 1).then_some(0.8);
        assert!(watch.preset_values(&values, &host).is_empty());
        assert!(!watch.poll(now, &values, cost, &host));
        assert!(!watch.poll(now, &values, cost, &host));
        assert!(
            !watch
                .read(
                    now,
                    &values,
                    Some((b"automated", cost)),
                    Said::Values,
                    &host
                )
                .with_state()
        );
    }
    #[test]
    fn an_intermediate_host_snapshot_during_audio_progress_is_not_a_preset() {
        let now = Instant::now();
        let cost = Duration::from_micros(10);
        let mut watch = Watch::new(now, &[(1, 0.5)], Some((b"initial", cost)));
        watch.note_host(&[1], &|id| (id == 1).then_some(0.7));
        // The audio thread can process more than one update while the helper's
        // getter is preempted: the snapshot lies between the observed endpoints.
        let values = [(1, 0.75)];
        let host = |id| (id == 1).then_some(0.8);
        assert!(
            watch.preset_values(&values, &host).is_empty(),
            "host motion became a preset"
        );
        assert!(!watch.poll(now, &values, cost, &host));
        assert!(!watch.poll(now, &values, cost, &host));
        assert!(
            !watch
                .read(
                    now,
                    &values,
                    Some((b"automated", cost)),
                    Said::Values,
                    &host
                )
                .with_state()
        );
    }

    #[test]
    fn host_moves_returning_to_the_mark_do_not_make_a_state_only_preset() {
        let now = Instant::now();
        let cost = Duration::from_micros(10);
        let values = [(1, 0.6), (2, 0.3), (3, 0.4)];
        let mut watch = Watch::new(now, &values, Some((b"initial", cost)));
        watch.note_host(&[3], &|id| (id == 3).then_some(0.4));
        // Automation moved and returned to 0.4 before the values were read. The audio
        // thread moves it again between param_values and save_state. This is still
        // host state, even though the settled parameter vector equals its old mark.
        let r = watch.read(
            now,
            &values,
            Some((b"automated", cost)),
            Said::Values,
            &|id| (id == 3).then_some(0.4),
        );
        assert!(
            !r.with_state(),
            "an asynchronous host snapshot became a state-only preset: {r:?}"
        );
    }

    #[test]
    fn a_preset_after_an_editor_edit_still_forwards_its_new_value() {
        let now = Instant::now();
        let mut watch = Watch::new(now, &[(1, 0.5)], None);
        watch.note_edits(&[ParamEdit {
            param: 1,
            kind: EditKind::Value,
            value: Some(0.6),
        }]);
        assert!(
            watch.preset_values(&[(1, 0.6)], &|_| None).is_empty(),
            "the editor's own value was already taken"
        );
        assert_eq!(
            watch.preset_values(&[(1, 0.8)], &|_| None),
            vec![(1, 0.8)],
            "a subsequent preset is independent of the earlier editor value"
        );
    }

    #[test]
    fn a_repeated_preset_does_not_forward_an_old_automation_value() {
        let now = Instant::now();
        let cost = Duration::from_micros(10);
        let initial = [(1, 0.5), (2, 0.5), (3, 0.0)];
        let mut watch = Watch::new(now, &initial, Some((b"initial", cost)));
        let preset = [(1, 0.6), (2, 0.3), (3, 0.0)];
        watch.read(now, &preset, Some((b"preset", cost)), Said::Values, &|_| {
            None
        });
        // A lane has changed since the mark. The repeated preset's settled reading
        // includes that host value; by the time the DAW drains it, the lane is elsewhere.
        watch.note_host(&[3], &|id| (id == 3).then_some(0.4));
        let repeated = [(1, 0.6), (2, 0.3), (3, 0.4)];
        let sent = watch.preset_values(&repeated, &|id| (id == 3).then_some(0.4));
        assert!(
            sent.is_empty(),
            "stale host values can be mistaken for a new preset: {sent:?}"
        );
        let r = watch.read(
            now,
            &repeated,
            Some((b"automated", cost)),
            Said::Values,
            &|id| (id == 3).then_some(0.4),
        );
        assert!(!r.with_state());
        // A later actual editor preset still sends its changed musical parameters.
        assert_eq!(
            watch.preset_values(&[(1, 0.7), (2, 0.35), (3, 0.8)], &|id| (id == 3)
                .then_some(0.8)),
            vec![(1, 0.7), (2, 0.35), (3, 0.8)]
        );
    }
}
