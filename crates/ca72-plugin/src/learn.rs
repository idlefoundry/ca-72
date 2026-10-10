//! MIDI Learn (decisions.md R34): a hardware controller's absolute 7-bit control change, on its
//! own MIDI channel, assigned to one of the plug-in's sound parameters; each instance its own
//! table, saved with the host's project, not with the sound presets.
//!
//! - **What may be learned** ([`LEARNABLE`]): the panel's knobs, selectors and switches and the
//!   strip's POLY, VOICES, ENTROPY and SPREAD, and LOCK (a host parameter without a control: the
//!   drawer's list learns it). Not the wheels (MIDI pitch bend and the modulation wheel, CC 1,
//!   move them already), POWER (the host's bypass), MIDI BEND RANGE (the player's), nor anything
//!   but a sound parameter.
//! - **What is not learned** ([`reserved`]): bank select (CC 0, 32), the modulation wheel (CC 1),
//!   data entry and RPN/NRPN (CC 6, 38, 96–101) and the channel mode messages (CC 120–127). The
//!   ones the plug-in follows (CC 1, 120, 121, 123) keep doing what they did.
//! - **One controller a parameter, one parameter a controller**: learning a parameter replaces its
//!   controller; learning a controller another parameter had moves it ([`MidiMap::assign`] says
//!   which). Removing one leaves the sound as it is.
//! - **Learning**: the editor arms a parameter ([`MidiMap::arm`]); the audio thread takes the next
//!   learnable control change, on its channel, as the one ([`MidiMap::incoming`]), changing
//!   nothing with it; the editor makes the assignment at its next frame ([`MidiMap::poll`]).
//!   Escape, CANCEL and the editor closing disarm it ([`MidiMap::cancel`]); arming another
//!   parameter moves the learning there. Nothing of learning is saved.
//! - **Playing**: a learned control change sets its parameter at its sample, through the host
//!   (`ProcessContext::set_parameter_normalized`, `third_party/nih-plug/PATCHES.md` change 10):
//!   a knob to value / 127 of its travel, a selector (or VOICES) to the position its share of
//!   0–127 falls in, a switch off at 0–63 and on at 64–127. The value jumps there ("jump"
//!   takeover); a knob's voices follow it over [`DEZIP`] ([`Dezip`]).
//!
//! The audio thread reads the table and arms nothing: it only ever loads [`MidiMap`]'s atomics,
//! takes an arming with a compare-and-swap and posts what it caught in an atomic word. Every
//! change of the table is made off it, under a lock it never takes.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};

use nih_plug::prelude::*;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

use crate::params::Ca72Params;

/// How a learned control change's 7-bit value sets a parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A knob or slider: value / 127 of its travel.
    Continuous,
    /// A selector (or VOICES): its positions each an equal share of 0–127.
    Stepped,
    /// A switch: off at 0–63, on at 64–127.
    Switch,
}

/// A parameter MIDI Learn may assign a controller to: its id (as saved), its name as the editor
/// gives it, and its kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Learnable {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: Kind,
}

const fn knob(id: &'static str, label: &'static str) -> Learnable {
    Learnable {
        id,
        label,
        kind: Kind::Continuous,
    }
}

const fn stepped(id: &'static str, label: &'static str) -> Learnable {
    Learnable {
        id,
        label,
        kind: Kind::Stepped,
    }
}

const fn switch(id: &'static str, label: &'static str) -> Learnable {
    Learnable {
        id,
        label,
        kind: Kind::Switch,
    }
}

/// Every parameter MIDI Learn may assign a controller to, in the panel's order, then the strip's,
/// then LOCK. Its order is the drawer's list; the table is saved by id, so it may change.
pub const LEARNABLE: [Learnable; 55] = [
    // CONTROLLERS.
    knob("tune", "TUNE"),
    switch("osc_mod", "OSCILLATOR MODULATION"),
    knob("glide", "GLIDE"),
    knob("mod_mix", "MODULATION MIX"),
    // OSCILLATOR BANK.
    stepped("osc1_range", "OSCILLATOR-1 RANGE"),
    stepped("osc2_range", "OSCILLATOR-2 RANGE"),
    stepped("osc3_range", "OSCILLATOR-3 RANGE"),
    knob("osc2_frequency", "OSCILLATOR-2 FREQUENCY"),
    knob("osc3_frequency", "OSCILLATOR-3 FREQUENCY"),
    stepped("osc1_waveform", "OSCILLATOR-1 WAVEFORM"),
    stepped("osc2_waveform", "OSCILLATOR-2 WAVEFORM"),
    stepped("osc3_waveform", "OSCILLATOR-3 WAVEFORM"),
    switch("osc3_control", "OSC. 3 CONTROL"),
    // MIXER.
    knob("osc1_volume", "OSCILLATOR-1 VOLUME"),
    switch("osc1_on", "OSCILLATOR-1 ON"),
    knob("ext_volume", "EXTERNAL INPUT VOLUME"),
    switch("ext_on", "EXTERNAL INPUT ON"),
    knob("osc2_volume", "OSCILLATOR-2 VOLUME"),
    switch("osc2_on", "OSCILLATOR-2 ON"),
    knob("noise_volume", "NOISE VOLUME"),
    switch("noise_on", "NOISE ON"),
    knob("osc3_volume", "OSCILLATOR-3 VOLUME"),
    switch("osc3_on", "OSCILLATOR-3 ON"),
    switch("noise_type", "NOISE (WHITE or PINK)"),
    switch("filter_mode", "FILTER MODE (LO or HI)"),
    switch("filter_mod", "FILTER MODULATION"),
    switch("keyboard_control_1", "KEYBOARD CONTROL 1"),
    switch("keyboard_control_2", "KEYBOARD CONTROL 2"),
    // MODIFIERS.
    knob("cutoff", "CUTOFF FREQUENCY"),
    knob("emphasis", "EMPHASIS"),
    knob("contour_amount", "AMOUNT OF CONTOUR"),
    knob("filter_attack", "FILTER CONTOUR ATTACK TIME"),
    knob("filter_decay", "FILTER CONTOUR DECAY TIME"),
    knob("filter_sustain", "FILTER CONTOUR SUSTAIN LEVEL"),
    knob("loudness_attack", "LOUDNESS CONTOUR ATTACK TIME"),
    knob("loudness_decay", "LOUDNESS CONTOUR DECAY TIME"),
    knob("loudness_sustain", "LOUDNESS CONTOUR SUSTAIN LEVEL"),
    // OUTPUT.
    knob("volume", "MAIN OUTPUT VOLUME"),
    switch("main_output", "MAIN OUTPUT"),
    switch("a440", "A-440"),
    knob("feedback", "FEEDBACK"),
    // The left hand controller's switches (not its wheels).
    switch("glide_on", "GLIDE (switch)"),
    switch("decay_on", "DECAY (switch)"),
    // The strip, and LOCK.
    switch("poly", "POLY"),
    switch("unison", "UNISON"),
    stepped("voices", "VOICES"),
    knob("entropy", "ENTROPY"),
    knob("spread", "WIDTH"),
    stepped("placement", "SCATTER PLACEMENT"),
    switch("doubled", "DOUBLE"),
    knob("double", "DETUNE (DOUBLE)"),
    knob("drive", "DRIVE"),
    knob("level", "LEVEL"),
    switch("auto_gain", "AUTO GAIN"),
    switch("lock", "LOCK (oscillators identical)"),
];

/// The learnable parameter of this id.
pub fn index(id: &str) -> Option<usize> {
    LEARNABLE.iter().position(|l| l.id == id)
}

/// Why a control change is not learned, if it is not: what it is.
pub fn reserved(cc: u8) -> Option<&'static str> {
    Some(match cc {
        0 | 32 => "bank select",
        1 => "the modulation wheel",
        6 | 38 => "data entry",
        96 | 97 => "data increment and decrement",
        98..=101 => "an RPN or NRPN number",
        120 => "all sound off",
        121 => "reset all controllers",
        122 => "local control",
        123 => "all notes off",
        124..=127 => "a channel mode message",
        128..=u8::MAX => "not a control change",
        _ => return None,
    })
}

/// The reserved controllers, as the editor lists them.
pub const RESERVED_TEXT: &str = "NOT LEARNED: CC 0 AND 32 (BANK), 1 (MODULATION WHEEL), 6, 38 AND \
                                 96-101 (DATA ENTRY, RPN, NRPN), 120-127 (CHANNEL MODE)";

/// A controller: a MIDI channel (0 to 15; shown 1 to 16) and a control change (0 to 127).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cc {
    pub channel: u8,
    pub cc: u8,
}

impl Cc {
    /// As the editor shows it: `CH 1 · CC 74`.
    pub fn text(self) -> String {
        format!("CH {} · CC {}", u32::from(self.channel) + 1, self.cc)
    }

    fn slot(self) -> usize {
        usize::from(self.channel & 15) * 128 + usize::from(self.cc & 127)
    }

    fn of_slot(slot: usize) -> Cc {
        Cc {
            channel: (slot / 128) as u8,
            cc: (slot % 128) as u8,
        }
    }

    fn learnable(self) -> bool {
        self.channel < 16 && self.cc < 128 && reserved(self.cc).is_none()
    }
}

/// What the audio thread does with a learnable control change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Incoming {
    /// Assigned to no parameter: nothing.
    Unassigned,
    /// Caught for the parameter being learned: nothing else (the sound is not changed by it).
    Caught,
    /// Assigned: set the learnable parameter of this index.
    Assigned(usize),
}

/// What an assignment changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Assigned {
    /// The parameter and its controller now.
    pub param: usize,
    pub cc: Cc,
    /// The controller it had before (replaced), if another.
    pub replaced: Option<Cc>,
    /// The parameter the controller was taken from, if another had it.
    pub displaced: Option<usize>,
}

/// A slot of [`MidiMap::routes`] holds the learnable's index plus one; 0 is none.
const NONE: u8 = 0;

/// The learning word ([`MidiMap::armed`]): the parameter armed (index + 1; 0 none) in its low
/// byte, the arming's number above it.
const TARGET_BITS: u32 = 8;
/// A caught controller ([`MidiMap::caught`]): valid, the arming's number, the parameter, the
/// channel and the control change.
const CAUGHT: u64 = 1 << 63;

/// One instance's MIDI assignments and its learning (decisions.md R34). Saved with the plug-in's
/// state under `midi_map` ([`Saved`]).
pub struct MidiMap {
    /// By channel and controller (channel × 128 + CC): the learnable parameter it sets, its
    /// index plus one ([`NONE`] none). The audio thread only loads these; every change is made
    /// under `writer`, which keeps at most one controller a parameter.
    routes: [AtomicU8; 16 * 128],
    writer: Mutex<()>,
    /// The parameter being learned and the arming's number ([`TARGET_BITS`]).
    armed: AtomicU32,
    /// What the audio thread caught for the arming ([`CAUGHT`]), for [`MidiMap::poll`].
    caught: AtomicU64,
    /// The last reserved control change seen while learning, for the editor to explain: the
    /// arming's number above, the control change + 1 in the low byte (0 none).
    refused: AtomicU32,
    /// Counts every change of the assignments (the editor's lists follow it).
    version: AtomicU32,
}

impl Default for MidiMap {
    fn default() -> Self {
        MidiMap {
            routes: std::array::from_fn(|_| AtomicU8::new(NONE)),
            writer: Mutex::new(()),
            armed: AtomicU32::new(0),
            caught: AtomicU64::new(0),
            refused: AtomicU32::new(0),
            version: AtomicU32::new(0),
        }
    }
}

impl std::fmt::Debug for MidiMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MidiMap")
            .field("assignments", &self.saved().0)
            .field("armed", &self.armed())
            .finish()
    }
}

impl MidiMap {
    // ---- The audio thread's: loads, a compare-and-swap and stores, nothing else.

    /// A learnable control change `cc` on `channel` (0 to 15), on the audio thread: caught for
    /// the parameter being learned (once), else its assignment.
    pub fn incoming(&self, channel: u8, cc: u8) -> Incoming {
        let c = Cc { channel, cc };
        if !c.learnable() {
            return Incoming::Unassigned;
        }
        // (Twice at most: the editor may arm again or cancel between the load and the swap.)
        for _ in 0..2 {
            let a = self.armed.load(Ordering::Acquire);
            let target = a & ((1 << TARGET_BITS) - 1);
            if target == 0 {
                break;
            }
            let disarmed = a & !((1 << TARGET_BITS) - 1);
            if self
                .armed
                .compare_exchange(a, disarmed, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                let word = CAUGHT
                    | (u64::from(a >> TARGET_BITS) << 24)
                    | (u64::from(target - 1) << 16)
                    | (u64::from(channel) << 8)
                    | u64::from(cc);
                self.caught.store(word, Ordering::Release);
                return Incoming::Caught;
            }
        }
        match self.routes[c.slot()].load(Ordering::Acquire) {
            NONE => Incoming::Unassigned,
            k => Incoming::Assigned(usize::from(k) - 1),
        }
    }

    /// A reserved control change seen, on the audio thread: while learning, kept for the editor
    /// to say why it was not learned.
    pub fn refuse(&self, cc: u8) {
        let a = self.armed.load(Ordering::Acquire);
        if a & ((1 << TARGET_BITS) - 1) != 0 {
            let word = (a & !((1 << TARGET_BITS) - 1)) | (u32::from(cc.min(127)) + 1);
            self.refused.store(word, Ordering::Release);
        }
    }

    // ---- The editor's and the host's (state): never on the audio thread.

    /// The learnable parameter set by this controller, if any.
    pub fn target(&self, cc: Cc) -> Option<usize> {
        match self.routes[cc.slot()].load(Ordering::Acquire) {
            NONE => None,
            k => Some(usize::from(k) - 1),
        }
    }

    /// Every learnable parameter's controller, by its index.
    pub fn assignments(&self) -> [Option<Cc>; LEARNABLE.len()] {
        let mut out = [None; LEARNABLE.len()];
        for (slot, r) in self.routes.iter().enumerate() {
            let k = r.load(Ordering::Acquire);
            if let Some(o) = usize::from(k).checked_sub(1).and_then(|i| out.get_mut(i)) {
                *o = Some(Cc::of_slot(slot));
            }
        }
        out
    }

    /// The controller assigned to learnable parameter `param`, if any.
    pub fn assignment(&self, param: usize) -> Option<Cc> {
        self.assignments().get(param).copied().flatten()
    }

    /// How often the assignments have changed (the editor follows it).
    pub fn version(&self) -> u32 {
        self.version.load(Ordering::Acquire)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ()> {
        self.writer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The slot of `param`'s controller, under the writer's lock.
    fn slot_of(&self, param: usize) -> Option<usize> {
        let want = u8::try_from(param + 1).ok()?;
        self.routes
            .iter()
            .position(|r| r.load(Ordering::Acquire) == want)
    }

    /// `cc` assigned to learnable parameter `param`: its controller before is let go (relearning
    /// replaces it), and a parameter that had `cc` loses it (it moves). None if either is not
    /// learnable.
    pub fn assign(&self, param: usize, cc: Cc) -> Option<Assigned> {
        if param >= LEARNABLE.len() || !cc.learnable() {
            return None;
        }
        let _w = self.lock();
        let before = self.slot_of(param);
        if let Some(s) = before {
            self.routes[s].store(NONE, Ordering::Release);
        }
        let had = self.routes[cc.slot()].swap(param as u8 + 1, Ordering::AcqRel);
        self.version.fetch_add(1, Ordering::AcqRel);
        Some(Assigned {
            param,
            cc,
            replaced: before.map(Cc::of_slot).filter(|b| *b != cc),
            displaced: usize::from(had).checked_sub(1).filter(|&d| d != param),
        })
    }

    /// Learnable parameter `param`'s controller removed: which it was.
    pub fn remove(&self, param: usize) -> Option<Cc> {
        let _w = self.lock();
        let s = self.slot_of(param)?;
        self.routes[s].store(NONE, Ordering::Release);
        self.version.fetch_add(1, Ordering::AcqRel);
        Some(Cc::of_slot(s))
    }

    /// The assignments as saved: by learnable index, in its order.
    pub fn saved(&self) -> Saved {
        let _w = self.lock();
        Saved(
            self.assignments()
                .iter()
                .enumerate()
                .filter_map(|(i, c)| c.map(|c| (i, c)))
                .collect(),
        )
    }

    /// Every assignment replaced by `saved`'s (an empty table: none).
    pub fn load(&self, saved: &Saved) {
        let _w = self.lock();
        for r in &self.routes {
            r.store(NONE, Ordering::Release);
        }
        for &(i, c) in &saved.0 {
            if i < LEARNABLE.len() && c.learnable() {
                self.routes[c.slot()].store(i as u8 + 1, Ordering::Release);
            }
        }
        self.version.fetch_add(1, Ordering::AcqRel);
    }

    /// Learnable parameter `param` armed: the next learnable control change is its controller.
    /// Another armed before is no longer (arming another moves the learning).
    pub fn arm(&self, param: usize) {
        if param >= LEARNABLE.len() {
            return;
        }
        self.rearm(param as u32 + 1);
    }

    /// Learning cancelled: the assignments as they were.
    pub fn cancel(&self) {
        self.rearm(0);
    }

    fn rearm(&self, target: u32) {
        let n = (self.armed.load(Ordering::Acquire) >> TARGET_BITS).wrapping_add(1)
            & (u32::MAX >> TARGET_BITS);
        self.armed
            .store((n << TARGET_BITS) | target, Ordering::Release);
    }

    /// The parameter being learned, if any.
    pub fn armed(&self) -> Option<usize> {
        let target = self.armed.load(Ordering::Acquire) & ((1 << TARGET_BITS) - 1);
        (target as usize).checked_sub(1)
    }

    /// The arming's number.
    fn arming(&self) -> u32 {
        self.armed.load(Ordering::Acquire) >> TARGET_BITS
    }

    /// What the audio thread caught for the present arming, assigned (none: nothing caught, or
    /// caught for an arming since replaced or cancelled).
    pub fn poll(&self) -> Option<Assigned> {
        let word = self.caught.swap(0, Ordering::AcqRel);
        if word & CAUGHT == 0
            || ((word >> 24) & u64::from(u32::MAX >> TARGET_BITS)) as u32 != self.arming()
        {
            return None;
        }
        let param = ((word >> 16) & 0xff) as usize;
        let cc = Cc {
            channel: ((word >> 8) & 0xff) as u8,
            cc: (word & 0xff) as u8,
        };
        self.assign(param, cc)
    }

    /// The last reserved control change seen while this arming waits, if any.
    pub fn refused(&self) -> Option<u8> {
        let word = self.refused.load(Ordering::Acquire);
        let cc = word & 0xff;
        (cc != 0 && word >> TARGET_BITS == self.arming()).then(|| (cc - 1) as u8)
    }
}

/// The assignments as the plug-in's state holds them (`midi_map`): learnable index and controller,
/// in [`LEARNABLE`]'s order, at most one a parameter and one a controller. As JSON:
/// `{"version":1,"assignments":[{"param":"cutoff","channel":1,"cc":74}]}`, the channel 1 to 16 as
/// shown. Read leniently ([`Saved::from_json`]); an armed learning is never part of it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Saved(pub Vec<(usize, Cc)>);

/// The table's format.
pub const VERSION: u64 = 1;

impl Saved {
    /// A table as saved, checked: anything but an object of version 1 is an empty table; an
    /// assignment of a parameter not learnable here, a channel outside 1 to 16 or a controller
    /// outside 0 to 127 or reserved, or of a parameter or a controller already assigned above it,
    /// is left out.
    pub fn from_json(v: &serde_json::Value) -> Saved {
        let mut out: Vec<(usize, Cc)> = Vec::new();
        if v.get("version").and_then(serde_json::Value::as_u64) != Some(VERSION) {
            return Saved(out);
        }
        let Some(list) = v.get("assignments").and_then(serde_json::Value::as_array) else {
            return Saved(out);
        };
        for a in list {
            let param = a
                .get("param")
                .and_then(serde_json::Value::as_str)
                .and_then(index);
            let channel = a.get("channel").and_then(serde_json::Value::as_u64);
            let cc = a.get("cc").and_then(serde_json::Value::as_u64);
            let (Some(param), Some(channel @ 1..=16), Some(cc @ 0..=127)) = (param, channel, cc)
            else {
                continue;
            };
            let c = Cc {
                channel: (channel - 1) as u8,
                cc: cc as u8,
            };
            if c.learnable() && !out.iter().any(|&(p, o)| p == param || o == c) {
                out.push((param, c));
            }
        }
        out.sort_by_key(|&(p, _)| p);
        Saved(out)
    }

    /// The table from the state's text: none (an empty table) if it is not JSON.
    pub fn from_text(text: &str) -> Saved {
        serde_json::from_str::<serde_json::Value>(text)
            .map(|v| Saved::from_json(&v))
            .unwrap_or_default()
    }
}

impl Serialize for Saved {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        struct Entry(usize, Cc);
        impl Serialize for Entry {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut m = s.serialize_map(Some(3))?;
                m.serialize_entry("param", LEARNABLE[self.0].id)?;
                m.serialize_entry("channel", &(u32::from(self.1.channel) + 1))?;
                m.serialize_entry("cc", &self.1.cc)?;
                m.end()
            }
        }
        struct List<'a>(&'a [(usize, Cc)]);
        impl Serialize for List<'_> {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut seq = s.serialize_seq(Some(self.0.len()))?;
                for &(i, c) in self.0 {
                    seq.serialize_element(&Entry(i, c))?;
                }
                seq.end()
            }
        }
        let mut m = s.serialize_map(Some(2))?;
        m.serialize_entry("version", &VERSION)?;
        m.serialize_entry("assignments", &List(&self.0))?;
        m.end()
    }
}

/// Any JSON reads ([`Saved::from_json`]): a table that is not understood is an empty one, never
/// an error, so a state is never half loaded over the assignments an instance had.
impl<'de> Deserialize<'de> for Saved {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Saved::from_json(&serde_json::Value::deserialize(d)?))
    }
}

/// The table in the plug-in's state (nih-plug's persistent fields): saved as [`Saved`], loaded
/// over every assignment the instance had.
impl<'a> nih_plug::params::persist::PersistentField<'a, Saved> for std::sync::Arc<MidiMap> {
    fn set(&self, new_value: Saved) {
        self.load(&new_value);
    }

    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&Saved) -> R,
    {
        f(&self.saved())
    }
}

/// The key of the table in the plug-in's state.
pub const STATE_KEY: &str = "midi_map";

/// A plug-in state's table made good before it is loaded (`Plugin::filter_state`): a state
/// without one (saved before MIDI Learn, or by a host's preset of another version) gets an empty
/// table, so that loading it into an instance with assignments leaves none; one that is not
/// JSON, or not understood, is an empty table too; one understood is written back checked.
pub fn filter_state(fields: &mut std::collections::BTreeMap<String, String>) {
    let saved = fields
        .get(STATE_KEY)
        .map(|t| Saved::from_text(t))
        .unwrap_or_default();
    let text = serde_json::to_string(&saved)
        .unwrap_or_else(|_| format!("{{\"version\":{VERSION},\"assignments\":[]}}"));
    fields.insert(STATE_KEY.to_owned(), text);
}

/// A parameter as MIDI Learn sets it, whatever its type.
pub trait Target {
    /// The pointer the host knows it by.
    fn ptr(&self) -> ParamPtr;
    /// The normalized value a 7-bit `value` sets it to: through the parameter's own
    /// normalization, a knob value / 127 of its travel, a stepped parameter the position its
    /// share of 0–127 holds (a switch: off below 64).
    fn normalized_for(&self, value: u8) -> f32;
    /// Whether it is at `normalized` already.
    fn at(&self, normalized: f32) -> bool;
    /// How many positions it has (none: continuous).
    fn positions(&self) -> Option<usize>;
}

impl<P: Param> Target for P {
    fn ptr(&self) -> ParamPtr {
        self.as_ptr()
    }

    fn normalized_for(&self, value: u8) -> f32 {
        let value = value.min(127);
        match self.step_count() {
            None => f32::from(value) / 127.0,
            Some(steps) => {
                let n = steps + 1;
                let position = (usize::from(value) * n / 128).min(steps);
                self.preview_normalized(self.preview_plain(position as f32 / steps.max(1) as f32))
            }
        }
    }

    fn at(&self, normalized: f32) -> bool {
        self.preview_plain(normalized) == self.preview_plain(self.unmodulated_normalized_value())
    }

    fn positions(&self) -> Option<usize> {
        self.step_count().map(|s| s + 1)
    }
}

/// The learnable parameter of index `i` (see [`LEARNABLE`]).
pub fn target(p: &Ca72Params, i: usize) -> Option<&dyn Target> {
    Some(match LEARNABLE.get(i)?.id {
        "tune" => &p.tune,
        "osc_mod" => &p.osc_mod,
        "glide" => &p.glide,
        "mod_mix" => &p.mod_mix,
        "osc1_range" => &p.osc1_range,
        "osc2_range" => &p.osc2_range,
        "osc3_range" => &p.osc3_range,
        "osc2_frequency" => &p.osc2_frequency,
        "osc3_frequency" => &p.osc3_frequency,
        "osc1_waveform" => &p.osc1_waveform,
        "osc2_waveform" => &p.osc2_waveform,
        "osc3_waveform" => &p.osc3_waveform,
        "osc3_control" => &p.osc3_control,
        "osc1_volume" => &p.osc1_volume,
        "osc1_on" => &p.osc1_on,
        "ext_volume" => &p.ext_volume,
        "ext_on" => &p.ext_on,
        "osc2_volume" => &p.osc2_volume,
        "osc2_on" => &p.osc2_on,
        "noise_volume" => &p.noise_volume,
        "noise_on" => &p.noise_on,
        "osc3_volume" => &p.osc3_volume,
        "osc3_on" => &p.osc3_on,
        "noise_type" => &p.noise_type,
        "filter_mode" => &p.filter_mode,
        "filter_mod" => &p.filter_mod,
        "keyboard_control_1" => &p.keyboard_control_1,
        "keyboard_control_2" => &p.keyboard_control_2,
        "cutoff" => &p.cutoff,
        "emphasis" => &p.emphasis,
        "contour_amount" => &p.contour_amount,
        "filter_attack" => &p.filter_attack,
        "filter_decay" => &p.filter_decay,
        "filter_sustain" => &p.filter_sustain,
        "loudness_attack" => &p.loudness_attack,
        "loudness_decay" => &p.loudness_decay,
        "loudness_sustain" => &p.loudness_sustain,
        "volume" => &p.volume,
        "main_output" => &p.main_output,
        "a440" => &p.a440,
        "feedback" => &p.feedback,
        "glide_on" => &p.glide_on,
        "decay_on" => &p.decay_on,
        "poly" => &p.poly,
        "voices" => &p.voices,
        "entropy" => &p.entropy,
        "spread" => &p.spread,
        "lock" => &p.lock,
        "placement" => &p.placement,
        "unison" => &p.unison,
        "doubled" => &p.doubled,
        "double" => &p.double,
        "drive" => &p.drive,
        "level" => &p.level,
        "auto_gain" => &p.auto_gain,
        _ => return None,
    })
}

/// The knob of learnable index `i`, if it is one (the ones [`Dezip`] moves).
pub fn knob_param(p: &Ca72Params, i: usize) -> Option<&FloatParam> {
    Some(match LEARNABLE.get(i)?.id {
        "tune" => &p.tune,
        "glide" => &p.glide,
        "mod_mix" => &p.mod_mix,
        "osc2_frequency" => &p.osc2_frequency,
        "osc3_frequency" => &p.osc3_frequency,
        "osc1_volume" => &p.osc1_volume,
        "ext_volume" => &p.ext_volume,
        "osc2_volume" => &p.osc2_volume,
        "noise_volume" => &p.noise_volume,
        "osc3_volume" => &p.osc3_volume,
        "cutoff" => &p.cutoff,
        "emphasis" => &p.emphasis,
        "contour_amount" => &p.contour_amount,
        "filter_attack" => &p.filter_attack,
        "filter_decay" => &p.filter_decay,
        "filter_sustain" => &p.filter_sustain,
        "loudness_attack" => &p.loudness_attack,
        "loudness_decay" => &p.loudness_decay,
        "loudness_sustain" => &p.loudness_sustain,
        "volume" => &p.volume,
        "feedback" => &p.feedback,
        "entropy" => &p.entropy,
        "spread" => &p.spread,
        "double" => &p.double,
        "drive" => &p.drive,
        "level" => &p.level,
        _ => return None,
    })
}

/// How long the voices take to follow a knob a learned controller moves, s: a 7-bit controller
/// moves a knob in steps of 1/127 of its travel, which the voices would otherwise take as steps
/// (MAIN OUTPUT VOLUME's click: decisions.md R34 measures them). The parameter itself (the
/// host's, the editor's, the state's) is at the new value at once.
pub const DEZIP: f64 = 0.010;

/// While a knob glides, the voices' controls are set again every this many samples.
pub const DEZIP_STEP: usize = 32;

/// A knob gliding to where a learned controller put it: the knob (its parameter's address, to
/// know it by), from and to (its plain values), and the samples gone and in all.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Glide {
    knob: usize,
    from: f32,
    to: f32,
    done: u32,
    len: u32,
}

impl Glide {
    fn now(&self) -> f32 {
        if self.done >= self.len {
            self.to
        } else {
            self.from + (self.to - self.from) * (self.done as f32 / self.len as f32)
        }
    }
}

/// The knobs that learned controllers move, gliding over [`DEZIP`] to their parameters' values
/// (the voices only: the parameters are at the new values at once). Only these knobs glide, and
/// only after a learned change: the host's automation, the editor and presets set the voices as
/// they always have. A glide ends early where anything else sets the parameter meanwhile.
#[derive(Clone, Debug)]
pub struct Dezip {
    glides: [Option<Glide>; LEARNABLE.len()],
    moving: usize,
    len: u32,
}

impl Default for Dezip {
    fn default() -> Self {
        Dezip {
            glides: [None; LEARNABLE.len()],
            moving: 0,
            len: 1,
        }
    }
}

impl Dezip {
    /// The glides' length at `rate` Hz.
    pub fn prepare(&mut self, rate: f64) {
        self.len = (DEZIP * rate).round().max(1.0) as u32;
        self.clear();
    }

    /// No glide.
    pub fn clear(&mut self) {
        self.glides = [None; LEARNABLE.len()];
        self.moving = 0;
    }

    /// Whether a knob glides.
    pub fn moving(&self) -> bool {
        self.moving > 0
    }

    /// Learnable knob `i`, `knob`, set from `from` (where the voices had it) to `to`: it glides
    /// there, from where it is if it was gliding.
    pub fn start(&mut self, i: usize, knob: &FloatParam, from: f32, to: f32) {
        let Some(slot) = self.glides.get_mut(i) else {
            return;
        };
        let from = slot.map_or(from, |g| g.now());
        if slot.is_none() {
            self.moving += 1;
        }
        *slot = Some(Glide {
            knob: std::ptr::from_ref(knob) as usize,
            from,
            to,
            done: 0,
            len: self.len.max(1),
        });
    }

    /// `n` samples on; a glide whose knob's parameter has been set elsewhere meanwhile (not at its
    /// end) ends.
    pub fn advance(&mut self, n: usize, p: &Ca72Params) {
        if self.moving == 0 {
            return;
        }
        for (i, slot) in self.glides.iter_mut().enumerate() {
            let Some(g) = slot else { continue };
            g.done = g.done.saturating_add(n as u32);
            let set_elsewhere = knob_param(p, i).is_none_or(|k| k.value() != g.to);
            if g.done >= g.len || set_elsewhere {
                *slot = None;
                self.moving -= 1;
            }
        }
    }

    /// The glides ended where anything other than a learned controller set their knobs: at a
    /// run's start, before the voices' controls are taken.
    pub fn follow(&mut self, p: &Ca72Params) {
        self.advance(0, p);
    }

    /// Where the voices have `knob` now: its glide's place, else its value.
    pub fn value(&self, knob: &FloatParam) -> f32 {
        if self.moving > 0 {
            let at = std::ptr::from_ref(knob) as usize;
            for g in self.glides.iter().flatten() {
                if g.knob == at {
                    return g.now();
                }
            }
        }
        knob.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cc(channel: u8, cc: u8) -> Cc {
        Cc { channel, cc }
    }

    fn at(id: &str) -> usize {
        index(id).expect("learnable")
    }

    /// The list is explicit and whole: every sound parameter a preset sets is learnable but the
    /// MODULATION wheel and MIDI BEND RANGE (the PITCH wheel and POWER are no preset's either),
    /// each id once, each resolving to its parameter, its kind its parameter's steps, the knobs
    /// (and only they) gliding.
    #[test]
    fn every_sound_parameter_but_the_wheels_bypass_and_bend_range_is_learnable() {
        let p = Ca72Params::default();
        let mut ids: Vec<&str> = LEARNABLE.iter().map(|l| l.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), LEARNABLE.len(), "an id twice");
        let mut sound: Vec<&str> = crate::presets::sound_params(&p)
            .into_iter()
            .map(|(id, _)| id)
            .filter(|id| !["mod_wheel", "midi_bend_range"].contains(id))
            .collect();
        sound.sort_unstable();
        assert_eq!(ids, sound);
        for id in ["pitch_wheel", "mod_wheel", "bypass", "midi_bend_range"] {
            assert_eq!(index(id), None, "{id} is not learnable");
        }
        for (i, l) in LEARNABLE.iter().enumerate() {
            let t = target(&p, i).expect(l.id);
            let kind = match t.positions() {
                None => Kind::Continuous,
                Some(2) => Kind::Switch,
                Some(_) => Kind::Stepped,
            };
            assert_eq!(kind, l.kind, "{}", l.id);
            assert_eq!(
                knob_param(&p, i).is_some(),
                l.kind == Kind::Continuous,
                "{}",
                l.id
            );
            if let Some(k) = knob_param(&p, i) {
                assert_eq!(t.ptr(), k.as_ptr(), "{}", l.id);
            }
        }
        let counts = |k: Kind| LEARNABLE.iter().filter(|l| l.kind == k).count();
        assert_eq!(
            (
                counts(Kind::Continuous),
                counts(Kind::Stepped),
                counts(Kind::Switch)
            ),
            (26, 8, 21)
        );
    }

    /// Bank select, the modulation wheel, data entry and RPN/NRPN, and the channel mode messages
    /// are not learned; every other control change is.
    #[test]
    fn the_reserved_controllers_are_not_learned() {
        let reserved_ccs: Vec<u8> = (0..=127).filter(|&c| reserved(c).is_some()).collect();
        let mut want = vec![0, 1, 6, 32, 38, 96, 97, 98, 99, 100, 101];
        want.extend(120..=127);
        assert_eq!(reserved_ccs, want);
        let m = MidiMap::default();
        for &c in &want {
            assert_eq!(m.assign(0, cc(0, c)), None, "CC {c}");
        }
        m.arm(at("cutoff"));
        assert_eq!(m.incoming(0, 1), Incoming::Unassigned);
        assert_eq!(m.armed(), Some(at("cutoff")), "a reserved CC is not caught");
        m.refuse(1);
        assert_eq!(m.refused(), Some(1));
        m.arm(at("emphasis"));
        assert_eq!(m.refused(), None, "said for the arming it came in");
        m.cancel();
        m.refuse(7);
        assert_eq!(m.refused(), None, "nothing said while not learning");
    }

    /// A knob: value / 127 of its travel. A switch: off at 0–63, on at 64–127. A selector: its six
    /// positions each an equal share of 0–127; VOICES its nine.
    #[test]
    fn a_value_sets_each_kind_through_its_parameters_normalization() {
        let p = Ca72Params::default();
        let cutoff = target(&p, at("cutoff")).expect("a knob");
        assert_eq!(cutoff.normalized_for(0), 0.0);
        assert_eq!(cutoff.normalized_for(127), 1.0);
        assert_eq!(cutoff.normalized_for(64), 64.0 / 127.0);
        let on = target(&p, at("osc1_on")).expect("a switch");
        assert_eq!(on.normalized_for(63), 0.0);
        assert_eq!(on.normalized_for(64), 1.0);
        let pink = target(&p, at("noise_type")).expect("a two-way switch");
        assert_eq!(
            (pink.normalized_for(63), pink.normalized_for(64)),
            (0.0, 1.0)
        );
        let range = target(&p, at("osc1_range")).expect("a selector");
        let position = |v: u8| (range.normalized_for(v) * 5.0).round() as u8;
        let firsts: Vec<u8> = (1..=127)
            .filter(|&v| position(v) != position(v - 1))
            .collect();
        assert_eq!(firsts, [22, 43, 64, 86, 107]);
        assert_eq!((position(0), position(127)), (0, 5));
        let voices = target(&p, at("voices")).expect("VOICES");
        let plain = |v: u8| p.voices.preview_plain(voices.normalized_for(v));
        assert_eq!((plain(0), plain(127)), (2, 10));
        let mut seen: Vec<i32> = (0..=127).map(plain).collect();
        seen.dedup();
        assert_eq!(seen, (2..=10).collect::<Vec<_>>());
    }

    /// Learning: the next learnable controller, on its channel, is caught (changing nothing),
    /// and assigned at the editor's next look; what follows sets the parameter.
    #[test]
    fn learning_catches_the_next_controller_and_assigns_it() {
        let m = MidiMap::default();
        let cutoff = at("cutoff");
        m.arm(cutoff);
        assert_eq!(m.armed(), Some(cutoff));
        assert_eq!(m.incoming(1, 74), Incoming::Caught);
        assert_eq!(m.armed(), None);
        // (Before the editor assigns it, the knob's next moves do nothing.)
        assert_eq!(m.incoming(1, 74), Incoming::Unassigned);
        let a = m.poll().expect("assigned");
        assert_eq!(
            a,
            Assigned {
                param: cutoff,
                cc: cc(1, 74),
                replaced: None,
                displaced: None
            }
        );
        assert_eq!(m.poll(), None, "once");
        assert_eq!(m.incoming(1, 74), Incoming::Assigned(cutoff));
        assert_eq!(m.assignment(cutoff), Some(cc(1, 74)));
        assert_eq!(cc(1, 74).text(), "CH 2 · CC 74");
    }

    /// Cancelling keeps every assignment; a controller caught for an arming since replaced or
    /// cancelled is not assigned.
    #[test]
    fn cancelling_keeps_the_assignments() {
        let m = MidiMap::default();
        let (cutoff, emphasis) = (at("cutoff"), at("emphasis"));
        m.assign(emphasis, cc(0, 71));
        m.arm(cutoff);
        m.cancel();
        assert_eq!(m.armed(), None);
        assert_eq!(m.incoming(0, 72), Incoming::Unassigned, "not caught");
        assert_eq!(m.incoming(0, 71), Incoming::Assigned(emphasis));
        m.arm(cutoff);
        assert_eq!(m.incoming(0, 72), Incoming::Caught);
        m.cancel();
        assert_eq!(m.poll(), None, "caught, then cancelled: not assigned");
        assert_eq!(m.saved(), Saved(vec![(emphasis, cc(0, 71))]));
    }

    /// Arming another control moves the learning to it.
    #[test]
    fn arming_another_control_moves_the_learning() {
        let m = MidiMap::default();
        m.arm(at("cutoff"));
        m.arm(at("emphasis"));
        assert_eq!(m.incoming(3, 20), Incoming::Caught);
        assert_eq!(m.poll().map(|a| a.param), Some(at("emphasis")));
        assert_eq!(m.assignment(at("cutoff")), None);
        // Caught for one, then another armed before the editor looked: the other waits.
        m.arm(at("glide"));
        assert_eq!(m.incoming(3, 21), Incoming::Caught);
        m.arm(at("tune"));
        assert_eq!(m.poll(), None);
        assert_eq!(m.armed(), Some(at("tune")));
    }

    /// One controller a parameter (relearning replaces it), one parameter a controller (reusing
    /// one moves it, and says from where); removing one leaves the others.
    #[test]
    fn relearning_replaces_and_reusing_moves() {
        let m = MidiMap::default();
        let (cutoff, emphasis) = (at("cutoff"), at("emphasis"));
        m.assign(cutoff, cc(0, 74));
        let a = m.assign(cutoff, cc(0, 75)).expect("assigned");
        assert_eq!((a.replaced, a.displaced), (Some(cc(0, 74)), None));
        assert_eq!(m.target(cc(0, 74)), None);
        let b = m.assign(emphasis, cc(0, 75)).expect("assigned");
        assert_eq!((b.replaced, b.displaced), (None, Some(cutoff)));
        assert_eq!(m.assignment(cutoff), None);
        assert_eq!(m.assignment(emphasis), Some(cc(0, 75)));
        assert_eq!(
            m.assign(emphasis, cc(0, 75))
                .map(|a| (a.replaced, a.displaced)),
            Some((None, None))
        );
        assert_eq!(m.remove(emphasis), Some(cc(0, 75)));
        assert_eq!(m.remove(emphasis), None);
        assert_eq!(m.incoming(0, 75), Incoming::Unassigned);
    }

    /// The same controller number on two channels is two controllers.
    #[test]
    fn channels_are_told_apart() {
        let m = MidiMap::default();
        let (cutoff, emphasis) = (at("cutoff"), at("emphasis"));
        m.assign(cutoff, cc(0, 74));
        assert_eq!(m.incoming(1, 74), Incoming::Unassigned);
        m.assign(emphasis, cc(1, 74));
        assert_eq!(m.incoming(0, 74), Incoming::Assigned(cutoff));
        assert_eq!(m.incoming(1, 74), Incoming::Assigned(emphasis));
        assert_eq!(m.incoming(15, 74), Incoming::Unassigned);
    }

    /// Saved as JSON by id, the channel 1 to 16, and read back the same.
    #[test]
    fn the_table_is_saved_by_id_and_read_back() {
        let m = MidiMap::default();
        m.assign(at("emphasis"), cc(15, 71));
        m.assign(at("cutoff"), cc(0, 74));
        m.assign(at("voices"), cc(2, 20));
        let text = serde_json::to_string(&m.saved()).expect("JSON");
        assert_eq!(
            text,
            r#"{"version":1,"assignments":[{"param":"cutoff","channel":1,"cc":74},{"param":"emphasis","channel":16,"cc":71},{"param":"voices","channel":3,"cc":20}]}"#
        );
        let back = MidiMap::default();
        back.load(&Saved::from_text(&text));
        assert_eq!(back.assignments(), m.assignments());
        let empty = MidiMap::default();
        assert_eq!(
            serde_json::to_string(&empty.saved()).expect("JSON"),
            r#"{"version":1,"assignments":[]}"#
        );
    }

    /// A table read leniently: another version or no object is an empty table; an entry of an
    /// unknown or unlearnable parameter, a channel or controller out of range or reserved, a
    /// value of the wrong type, or a parameter or controller already assigned above it, is left
    /// out (the first one kept).
    #[test]
    fn malformed_duplicate_and_unknown_entries_are_left_out() {
        for text in [
            "",
            "garbage",
            "[]",
            "42",
            r#"{"assignments":[{"param":"cutoff","channel":1,"cc":74}]}"#,
            r#"{"version":2,"assignments":[{"param":"cutoff","channel":1,"cc":74}]}"#,
            r#"{"version":"1","assignments":[{"param":"cutoff","channel":1,"cc":74}]}"#,
            r#"{"version":1,"assignments":{"param":"cutoff","channel":1,"cc":74}}"#,
        ] {
            assert_eq!(Saved::from_text(text), Saved::default(), "{text}");
        }
        let text = r#"{"version":1,"extra":true,"assignments":[
            {"param":"cutoff","channel":1,"cc":74},
            {"param":"cutoff","channel":1,"cc":75},
            {"param":"emphasis","channel":1,"cc":74},
            {"param":"emphasis","channel":2,"cc":74,"later":"ignored"},
            {"param":"no_such","channel":1,"cc":10},
            {"param":"pitch_wheel","channel":1,"cc":11},
            {"param":"bypass","channel":1,"cc":12},
            {"param":"midi_bend_range","channel":1,"cc":13},
            {"param":"glide","channel":0,"cc":14},
            {"param":"glide","channel":17,"cc":14},
            {"param":"glide","channel":1,"cc":128},
            {"param":"glide","channel":1,"cc":-1},
            {"param":"glide","channel":"1","cc":14},
            {"param":"glide","channel":1,"cc":1},
            {"param":"glide","channel":1,"cc":120},
            {"param":"glide","channel":1},
            "nonsense",
            {"param":"tune","channel":16,"cc":127},
            {"param":"tune","channel":16,"cc":119}
        ]}"#;
        assert_eq!(
            Saved::from_text(text),
            Saved(vec![
                (at("tune"), cc(15, 119)),
                (at("cutoff"), cc(0, 74)),
                (at("emphasis"), cc(1, 74)),
            ])
        );
    }

    /// Loaded over an instance's assignments, a table replaces every one of them; an armed
    /// learning is never part of what is saved.
    #[test]
    fn a_table_loaded_replaces_every_assignment_and_learning_is_not_saved() {
        let m = MidiMap::default();
        m.assign(at("cutoff"), cc(0, 74));
        m.arm(at("glide"));
        let saved = m.saved();
        assert_eq!(saved, Saved(vec![(at("cutoff"), cc(0, 74))]));
        let text = serde_json::to_string(&saved).expect("JSON");
        assert!(!text.contains("glide"), "{text}");
        m.load(&Saved(vec![(at("emphasis"), cc(2, 3))]));
        assert_eq!(m.assignment(at("cutoff")), None);
        assert_eq!(m.assignment(at("emphasis")), Some(cc(2, 3)));
        m.load(&Saved::default());
        assert!(m.assignments().iter().all(Option::is_none));
    }

    /// A state's table made good before it loads: none (saved before MIDI Learn) or not
    /// understood is an empty one; one understood is written back checked.
    #[test]
    fn a_states_table_is_made_good_before_it_loads() {
        use std::collections::BTreeMap;
        let empty = r#"{"version":1,"assignments":[]}"#;
        let mut fields = BTreeMap::new();
        filter_state(&mut fields);
        assert_eq!(fields.get(STATE_KEY).map(String::as_str), Some(empty));
        fields.insert(STATE_KEY.into(), "{not json".into());
        filter_state(&mut fields);
        assert_eq!(fields.get(STATE_KEY).map(String::as_str), Some(empty));
        fields.insert(
            STATE_KEY.into(),
            r#"{"version":1,"assignments":[{"param":"cutoff","channel":1,"cc":74},{"param":"x","channel":1,"cc":2}]}"#.into(),
        );
        filter_state(&mut fields);
        assert_eq!(
            fields.get(STATE_KEY).map(String::as_str),
            Some(r#"{"version":1,"assignments":[{"param":"cutoff","channel":1,"cc":74}]}"#)
        );
    }

    /// A knob glides over `DEZIP` to the value a learned controller set, from where the voices
    /// had it (from where it was gliding, if it was), and stops gliding when anything else sets
    /// its parameter.
    #[test]
    fn a_knob_glides_to_a_learned_value_and_stops_where_set_elsewhere() {
        let p = Ca72Params::default();
        let i = at("cutoff");
        let k = knob_param(&p, i).expect("a knob");
        let mut d = Dezip::default();
        d.prepare(48_000.0);
        assert_eq!(d.value(k), 0.0);
        // (The parameter itself stays at its default here: as if set elsewhere unless `to`
        // is its value.)
        d.start(i, k, 4.0, 0.0);
        assert!(d.moving());
        assert_eq!(d.value(k), 4.0);
        d.advance(240, &p);
        assert!((d.value(k) - 2.0).abs() < 1e-5, "{}", d.value(k));
        d.advance(240, &p);
        assert!(!d.moving());
        assert_eq!(d.value(k), 0.0);
        // Set elsewhere meanwhile (its value is not the glide's end): the glide ends.
        d.start(i, k, 0.0, 3.0);
        d.follow(&p);
        assert!(!d.moving());
        assert_eq!(d.value(k), k.value());
        // A second change mid-glide glides on from where it was.
        d.start(i, k, 4.0, 0.0);
        d.advance(120, &p);
        let mid = d.value(k);
        d.start(i, k, 99.0, 0.0);
        assert_eq!(d.value(k), mid);
    }
}
