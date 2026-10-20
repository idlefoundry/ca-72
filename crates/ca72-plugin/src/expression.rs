//! Each note's own expression as MIDI brings it (decisions.md R-MPE), and the pressure the MOD
//! wheel's aftertouch sources read (R-MODWHEEL-SOURCE): one record of both, kept in fixed
//! storage on the audio thread.
//!
//! - **MPE.** The MPE Configuration Message (RPN 6 on channel 1 or 16) makes a lower or an
//!   upper zone of member channels, each note on a member channel of its own. A member channel's
//!   pitch bend (over its zone's bend range, RPN 0, 48 semitones by default), channel pressure
//!   and control change 74 (timbre) are its note's alone. The zone's master channel is as any
//!   other: its bend is the PITCH wheel's, its pressure and timbre every voice's.
//! - **A host's note expressions** (CLAP's and VST3's pressure, tuning and brightness; a
//!   keyboard's polyphonic aftertouch) are their note's alone, MPE or not.
//! - **Ordinary MIDI.** Outside a zone, channel pressure and control change 74 are every
//!   voice's alike.
//!
//! A note's expression goes to the engine as its key's ([`Event::NoteExpression`]) before the
//! note itself, and whenever it changes while the note is held; the shared pressure and timbre
//! as [`Event::ChannelPressure`] and [`Event::Timbre`]. Each is sent only when it changes, so
//! that MIDI without them sends nothing new.
//!
//! The timbre goes to the engine measured from its rest point, the computer's TIMBRE REST
//! ([`TimbreRest`], decisions.md R-MPE): -1..1 and 0 at rest, a note or channel that
//! never sent one at rest whatever the point. What arrives, per note and on ordinary channels,
//! is counted for the editor's INPUT screen ([`Heard`]).

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use nih_plug::prelude::NoteEvent;

use crate::engine::{BEND_MOST, Event, Expression, farther};

/// A member channel's bend range until RPN 0 says otherwise, semitones (MPE's default).
pub const MEMBER_BEND: f32 = 48.0;

/// The MPE Configuration Message's RPN, and the pitch bend range's.
const MCM: u16 = 6;
const BEND_RANGE: u16 = 0;
/// No RPN chosen (MIDI's null RPN, 127/127).
const NULL: u16 = 0x3fff;

/// The timbre's control change (MPE's third dimension; a keyboard's brightness).
pub const TIMBRE_CC: u8 = 74;

/// Where MPE's zones come from, a setting of the computer's (`settings.rs`): `Auto`, the MPE
/// Configuration Message (the default: without one, ordinary MIDI); `Lower`, a lower zone of
/// fifteen member channels whatever the controller says, for hosts that do not pass the message
/// on; `Off`, none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Auto,
    Lower,
    Off,
}

impl Mode {
    pub fn code(self) -> u8 {
        match self {
            Mode::Auto => 0,
            Mode::Lower => 1,
            Mode::Off => 2,
        }
    }

    pub fn of_code(code: u8) -> Mode {
        match code {
            1 => Mode::Lower,
            2 => Mode::Off,
            _ => Mode::Auto,
        }
    }

    /// As `settings.toml` names it.
    pub fn word(self) -> &'static str {
        match self {
            Mode::Auto => "auto",
            Mode::Lower => "lower",
            Mode::Off => "off",
        }
    }

    pub fn of_word(word: &str) -> Option<Mode> {
        [Mode::Auto, Mode::Lower, Mode::Off]
            .into_iter()
            .find(|m| m.word() == word)
    }
}

/// Where a timbre rests, a setting of the computer's (`settings.rs`, `timbre_rest`; decisions.md
/// R-MPE). The engine is given each timbre measured from it, -1..1 and 0 at rest
/// ([`TimbreRest::measured`]):
/// - `Centred` (the default, "centre"): control change 74 at 64 and a host's brightness at 0.5
///   rest; 0 is -1, 127 (1.0) is 1. Many MPE controllers rest near 64.
/// - `Zero` ("zero"): 0..1 from 0, as the CA-72 first had it (R-MPE), to the bit.
/// - `NoteStart` ("start"): each note's timbre from where it was as the note began, a fresh
///   touch always at rest (now less then, -1..1). Where a note begins is
///   [`Expressions`]'s to keep: a member channel's last value before the note, else its first
///   after; a host's brightness's first value; ordinary MIDI's shared value as the phrase
///   began (the first key pressed with none held), else its first after.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimbreRest {
    #[default]
    Centred,
    Zero,
    NoteStart,
}

/// No timbre received (a note's, a channel's or the shared one): at rest whatever the rest point.
pub const UNHEARD: f32 = -1.0;

impl TimbreRest {
    pub const ALL: [TimbreRest; 3] = [TimbreRest::Centred, TimbreRest::Zero, TimbreRest::NoteStart];

    /// As an atomic keeps it (`Ca72Params::timbre_rest`): 0, 1, 2 in [`TimbreRest::ALL`]'s order.
    pub fn index(self) -> u8 {
        match self {
            TimbreRest::Centred => 0,
            TimbreRest::Zero => 1,
            TimbreRest::NoteStart => 2,
        }
    }

    /// Anything but 1 or 2, the default.
    pub fn from_index(index: u8) -> TimbreRest {
        match index {
            1 => TimbreRest::Zero,
            2 => TimbreRest::NoteStart,
            _ => TimbreRest::Centred,
        }
    }

    /// As `settings.toml` names it.
    pub fn word(self) -> &'static str {
        match self {
            TimbreRest::Centred => "centre",
            TimbreRest::Zero => "zero",
            TimbreRest::NoteStart => "start",
        }
    }

    pub fn of_word(word: &str) -> Option<TimbreRest> {
        TimbreRest::ALL.into_iter().find(|r| r.word() == word)
    }

    /// A timbre `now` (0..1 as received, or [`UNHEARD`]) measured from this rest, where it was
    /// as its note began being `start` (NoteStart's): -1..1, 0 at rest, and 0 for none heard.
    pub fn measured(self, now: f32, start: f32) -> f32 {
        if now < 0.0 {
            return 0.0;
        }
        match self {
            TimbreRest::Zero => now,
            TimbreRest::Centred => centred(now),
            TimbreRest::NoteStart if start < 0.0 => 0.0,
            TimbreRest::NoteStart => (now - start).clamp(-1.0, 1.0),
        }
    }
}

/// A timbre 0..1 about its centre, -1..1: control change 74's 64 (64/127) and a host's
/// brightness's 0.5 are both at rest (the half-step between them too, so that CC 74's value and
/// the brightness a host makes of it, v/127, measure the same), 0 is -1, 1.0 (127) is 1; below
/// the centre each step of a brightness's, above each of CC 74's 63.
fn centred(x: f32) -> f32 {
    const MIDDLE: f32 = 64.0 / 127.0;
    if x < 0.5 {
        x * 2.0 - 1.0
    } else if x <= MIDDLE {
        0.0
    } else {
        ((x * 127.0 - 64.0) / 63.0).min(1.0)
    }
}

/// What expression has arrived, counted for the editor's INPUT screen (decisions.md
/// R-MPE): each count goes up (wrapping) on the audio thread as a message arrives, and
/// the editor times how recently each changed. `per_note`: a note's own pressure, timbre or
/// bend (an MPE member channel's, polyphonic aftertouch, a host's note expressions);
/// `channel`: ordinary channel pressure or control change 74 (outside a zone's members).
#[derive(Debug, Default)]
pub struct Heard {
    per_note: AtomicU32,
    channel: AtomicU32,
}

impl Heard {
    /// (per_note, channel), as counted so far.
    pub fn counts(&self) -> (u32, u32) {
        (
            self.per_note.load(Ordering::Relaxed),
            self.channel.load(Ordering::Relaxed),
        )
    }

    pub(crate) fn note(&self) {
        self.per_note.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn channel(&self) {
        self.channel.fetch_add(1, Ordering::Relaxed);
    }
}

/// The record (fixed storage: some 42 KB, nothing allocated after it is made).
#[derive(Debug)]
pub struct Expressions {
    pub mode: Mode,
    /// Where a timbre rests (the computer's setting).
    pub rest: TimbreRest,
    /// What has arrived, for the editor (`Ca72Params::heard`).
    heard: Arc<Heard>,
    /// The member channels each zone's MCM gave (lower, upper; 0, none) and each zone's
    /// members' bend range, semitones.
    mcm: [u8; 2],
    range: [f32; 2],
    /// Each channel's RPN chosen, and its pitch bend (-1..1), pressure and timbre (0..1; the
    /// timbre [`UNHEARD`] until one arrives).
    rpn: [u16; 16],
    bend: [f32; 16],
    pressure: [f32; 16],
    timbre: [f32; 16],
    /// The keys held, by channel; each note's own pressure, timbre (0..1, [`UNHEARD`] until one
    /// arrives) and tuning (semitones), as a host's note expressions or polyphonic aftertouch
    /// give them.
    held: [u128; 16],
    key_pressure: [[f32; 128]; 16],
    key_timbre: [[f32; 128]; 16],
    key_tuning: [[f32; 128]; 16],
    /// Where each note's timbre began (TimbreRest::NoteStart's; [`UNHEARD`], not yet): its own
    /// (a host's brightness), its first after the note; its member channel's, the channel's as
    /// the note began, or its first after.
    key_start: [[f32; 128]; 16],
    member_start: [[f32; 128]; 16],
    /// The pressure and timbre every voice shares, as last received outside a zone (the timbre
    /// [`UNHEARD`] until one arrives), and the timbre as the phrase began, or its first after.
    shared: [f32; 2],
    shared_start: f32,
    /// What the engine was last told: each key's expression, and the shared pressure and timbre.
    sent: [Expression; 128],
    sent_shared: [f32; 2],
}

impl Default for Expressions {
    fn default() -> Self {
        Expressions {
            mode: Mode::Auto,
            rest: TimbreRest::default(),
            heard: Arc::default(),
            mcm: [0; 2],
            range: [MEMBER_BEND; 2],
            rpn: [NULL; 16],
            bend: [0.0; 16],
            pressure: [0.0; 16],
            timbre: [UNHEARD; 16],
            held: [0; 16],
            key_pressure: [[0.0; 128]; 16],
            key_timbre: [[UNHEARD; 128]; 16],
            key_tuning: [[0.0; 128]; 16],
            key_start: [[UNHEARD; 128]; 16],
            member_start: [[UNHEARD; 128]; 16],
            shared: [0.0, UNHEARD],
            shared_start: UNHEARD,
            sent: [Expression::NONE; 128],
            sent_shared: [0.0; 2],
        }
    }
}

/// A value 0..1 as received (one not finite, 0).
fn unit(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

impl Expressions {
    /// A record counting what arrives in `heard` (the editor's, `Ca72Params::heard`).
    pub fn counting(heard: Arc<Heard>) -> Box<Expressions> {
        let mut x = Box::<Expressions>::default();
        x.heard = heard;
        x
    }

    /// The member channels of the lower and upper zones (0, no zone), as the mode has them.
    pub fn zones(&self) -> [u8; 2] {
        match self.mode {
            Mode::Auto => self.mcm,
            Mode::Lower => [15, 0],
            Mode::Off => [0, 0],
        }
    }

    /// The zone (0 lower, 1 upper) channel `channel` (0..16) is a member of, if any: the lower's
    /// members are channels 2 up, the upper's 15 down (1 and 16 their masters).
    pub fn zone_of(&self, channel: u8) -> Option<usize> {
        let [lower, upper] = self.zones();
        if channel >= 1 && channel <= lower {
            Some(0)
        } else if channel <= 14 && channel + upper >= 15 && upper > 0 {
            Some(1)
        } else {
            None
        }
    }

    pub fn is_member(&self, channel: u8) -> bool {
        self.zone_of(channel).is_some()
    }

    pub fn held(&self, channel: u8, key: u8) -> bool {
        channel < 16 && key < 128 && self.held[usize::from(channel)] & (1 << key) != 0
    }

    /// Key `key`'s own pressure on `channel`: its polyphonic one, or its member channel's.
    pub fn note_pressure(&self, channel: u8, key: u8) -> f32 {
        let (c, k) = (usize::from(channel), usize::from(key));
        let own = self.key_pressure[c][k];
        if self.is_member(channel) {
            own.max(self.pressure[c])
        } else {
            own
        }
    }

    /// The note `key` on `channel`'s own expression (the shared pressure and timbre apart).
    pub fn of(&self, channel: u8, key: u8) -> Expression {
        let (c, k) = (usize::from(channel), usize::from(key));
        let rest = self.rest;
        let mut e = Expression {
            pressure: self.key_pressure[c][k],
            timbre: rest.measured(self.key_timbre[c][k], self.key_start[c][k]),
            bend: self.key_tuning[c][k],
        };
        if let Some(zone) = self.zone_of(channel) {
            e.pressure = e.pressure.max(self.pressure[c]);
            // (Its own and its channel's, the one farther from rest: R-MPE.)
            let member = rest.measured(self.timbre[c], self.member_start[c][k]);
            e.timbre = farther(e.timbre, member);
            e.bend += self.bend[c] * self.range[zone];
        }
        e.bend = e.bend.clamp(-BEND_MOST, BEND_MOST);
        e
    }

    /// The highest of the held keys' own pressures for which `fresh` (by channel) is set: the
    /// MOD wheel's polyphonic aftertouch.
    pub fn most_pressure(&self, fresh: &[u128; 16]) -> f32 {
        let mut most = 0.0f32;
        for channel in 0..16u8 {
            let mut keys = self.held[usize::from(channel)] & fresh[usize::from(channel)];
            while keys != 0 {
                let key = keys.trailing_zeros() as u8;
                keys &= keys - 1;
                most = most.max(self.note_pressure(channel, key));
            }
        }
        most
    }

    /// The keys held on `channel`, by number.
    pub fn held_on(&self, channel: u8) -> u128 {
        self.held.get(usize::from(channel)).copied().unwrap_or(0)
    }

    /// A MIDI event, the engine told what it changes through `out`. Whether it is a member
    /// channel's bend, pressure or timbre, its note's alone and nothing else's (not the PITCH
    /// wheel's, nor learned, nor the MOD wheel's channel aftertouch).
    pub fn event(&mut self, e: NoteEvent<()>, out: &mut impl FnMut(Event)) -> bool {
        match e {
            NoteEvent::NoteOn { channel, note, .. } if channel < 16 && note < 128 => {
                let (c, k) = (usize::from(channel), usize::from(note));
                // (A key pressed with none held begins a phrase: the shared timbre from here,
                // TimbreRest::NoteStart's.)
                if self.held.iter().all(|&h| h == 0) {
                    self.shared_start = self.shared[1];
                    self.tell_shared(out);
                }
                self.held[c] |= 1 << note;
                // (A new note's own expression starts from nothing; its member channel's, as
                // MPE sends it before the note, stays, and is where its timbre began.)
                self.key_pressure[c][k] = 0.0;
                self.key_timbre[c][k] = UNHEARD;
                self.key_tuning[c][k] = 0.0;
                self.key_start[c][k] = UNHEARD;
                self.member_start[c][k] = self.timbre[c];
                self.tell(channel, note, out);
                false
            }
            NoteEvent::NoteOff { channel, note, .. } | NoteEvent::Choke { channel, note, .. }
                if channel < 16 && note < 128 =>
            {
                // (The voice letting it go keeps what it had: nothing is sent.)
                self.held[usize::from(channel)] &= !(1 << note);
                self.key_pressure[usize::from(channel)][usize::from(note)] = 0.0;
                false
            }
            NoteEvent::PolyPressure {
                channel,
                note,
                pressure,
                ..
            } => {
                self.heard.note();
                if self.held(channel, note) {
                    self.key_pressure[usize::from(channel)][usize::from(note)] = unit(pressure);
                    self.tell(channel, note, out);
                }
                false
            }
            NoteEvent::PolyBrightness {
                channel,
                note,
                brightness,
                ..
            } => {
                self.heard.note();
                if self.held(channel, note) {
                    let (c, k) = (usize::from(channel), usize::from(note));
                    let v = unit(brightness);
                    self.key_timbre[c][k] = v;
                    // (Its first since the note began, where it began: often at the note's own
                    // time, just after it.)
                    if self.key_start[c][k] < 0.0 {
                        self.key_start[c][k] = v;
                    }
                    self.tell(channel, note, out);
                }
                false
            }
            NoteEvent::PolyTuning {
                channel,
                note,
                tuning,
                ..
            } => {
                self.heard.note();
                if self.held(channel, note) {
                    let t = if tuning.is_finite() { tuning } else { 0.0 };
                    self.key_tuning[usize::from(channel)][usize::from(note)] = t;
                    self.tell(channel, note, out);
                }
                false
            }
            NoteEvent::MidiChannelPressure {
                channel, pressure, ..
            } if channel < 16 => {
                if self.is_member(channel) {
                    self.heard.note();
                    self.pressure[usize::from(channel)] = unit(pressure);
                    self.tell_channel(channel, out);
                    true
                } else {
                    self.heard.channel();
                    self.shared[0] = unit(pressure);
                    self.tell_shared(out);
                    false
                }
            }
            NoteEvent::MidiPitchBend { channel, value, .. } if channel < 16 => {
                if !self.is_member(channel) {
                    return false;
                }
                self.heard.note();
                let b = if value.is_finite() {
                    (value * 2.0 - 1.0).clamp(-1.0, 1.0)
                } else {
                    0.0
                };
                self.bend[usize::from(channel)] = b;
                self.tell_channel(channel, out);
                true
            }
            NoteEvent::MidiCC {
                channel, cc, value, ..
            } if channel < 16 => self.cc(channel, cc, value, out),
            _ => false,
        }
    }

    /// A control change: RPN 0 and 6 followed, 74 the timbre, 121 the controllers at rest.
    fn cc(&mut self, channel: u8, cc: u8, value: f32, out: &mut impl FnMut(Event)) -> bool {
        let c = usize::from(channel);
        let seven = (unit(value) * 127.0).round() as u16;
        match cc {
            101 => self.rpn[c] = (seven << 7) | (self.rpn[c] & 0x7f),
            100 => self.rpn[c] = (self.rpn[c] & !0x7f) | seven,
            98 | 99 => self.rpn[c] = NULL,
            6 => self.data_entry(channel, seven as u8, out),
            TIMBRE_CC => {
                let v = unit(value);
                if self.is_member(channel) {
                    self.heard.note();
                    self.timbre[c] = v;
                    // (The channel's first for a note begun with none: where it began.)
                    let mut keys = self.held[c];
                    while keys != 0 {
                        let k = keys.trailing_zeros() as usize;
                        keys &= keys - 1;
                        if self.member_start[c][k] < 0.0 {
                            self.member_start[c][k] = v;
                        }
                    }
                    self.tell_channel(channel, out);
                    return true;
                }
                self.heard.channel();
                self.shared[1] = v;
                if self.shared_start < 0.0 {
                    self.shared_start = v;
                }
                self.tell_shared(out);
            }
            121 => self.reset_controllers(),
            _ => {}
        }
        false
    }

    /// Data entry's coarse value for channel `channel`'s RPN: the MPE Configuration Message on
    /// a zone's master channel (its member channels; 0, the zone gone), or a member channel's
    /// bend range (every member of its zone's).
    fn data_entry(&mut self, channel: u8, value: u8, out: &mut impl FnMut(Event)) {
        match self.rpn[usize::from(channel)] {
            MCM if channel == 0 || channel == 15 => {
                let zone = usize::from(channel == 15);
                let n = value.min(15);
                self.mcm[zone] = n;
                // (MPE's: a zone's MCM resets its bend range; the other zone gives way to it.)
                self.range[zone] = MEMBER_BEND;
                let other = 1 - zone;
                self.mcm[other] = self.mcm[other].min(14u8.saturating_sub(n));
                self.retell(out);
            }
            BEND_RANGE => {
                if let Some(zone) = self.zone_of(channel) {
                    self.range[zone] = f32::from(value.min(96));
                    self.retell(out);
                }
            }
            _ => {}
        }
    }

    /// Reset All Controllers (control change 121, on any channel): every bend and pressure at
    /// rest, the timbre, a sound controller, as it was (MIDI's RP-015). The engine rests its own
    /// ([`Event::ResetControllers`]); what it was told is so kept.
    pub fn reset_controllers(&mut self) {
        self.bend = [0.0; 16];
        self.pressure = [0.0; 16];
        self.key_pressure = [[0.0; 128]; 16];
        self.key_tuning = [[0.0; 128]; 16];
        self.shared[0] = 0.0;
        self.sent_shared[0] = 0.0;
        for e in &mut self.sent {
            (e.pressure, e.bend) = (0.0, 0.0);
        }
    }

    /// Every key let go (All Notes Off, All Sound Off, a panic, the host's reset): no note's
    /// expression left (the voices letting go keep theirs).
    pub fn release(&mut self) {
        self.held = [0; 16];
        self.key_pressure = [[0.0; 128]; 16];
        self.key_timbre = [[UNHEARD; 128]; 16];
        self.key_tuning = [[0.0; 128]; 16];
        self.key_start = [[UNHEARD; 128]; 16];
    }

    /// The engine told the note `key` on `channel`'s expression, if it changed.
    fn tell(&mut self, channel: u8, key: u8, out: &mut impl FnMut(Event)) {
        let e = self.of(channel, key);
        if e != self.sent[usize::from(key)] {
            self.sent[usize::from(key)] = e;
            out(Event::NoteExpression { key, expression: e });
        }
    }

    /// The engine told the expression of every note held on `channel`.
    fn tell_channel(&mut self, channel: u8, out: &mut impl FnMut(Event)) {
        let mut keys = self.held_on(channel);
        while keys != 0 {
            let key = keys.trailing_zeros() as u8;
            keys &= keys - 1;
            self.tell(channel, key, out);
        }
    }

    /// The engine told every held note's expression (the zones or a bend range changed).
    fn retell(&mut self, out: &mut impl FnMut(Event)) {
        for channel in 0..16 {
            self.tell_channel(channel, out);
        }
    }

    fn tell_shared(&mut self, out: &mut impl FnMut(Event)) {
        if self.shared[0] != self.sent_shared[0] {
            self.sent_shared[0] = self.shared[0];
            out(Event::ChannelPressure(self.shared[0]));
        }
        let timbre = self.rest.measured(self.shared[1], self.shared_start);
        if timbre != self.sent_shared[1] {
            self.sent_shared[1] = timbre;
            out(Event::Timbre(timbre));
        }
    }

    /// The mode the computer's setting chose (`settings.rs`); held notes told what it changes.
    pub fn set_mode(&mut self, mode: Mode, out: &mut impl FnMut(Event)) {
        if mode != self.mode {
            self.mode = mode;
            self.retell(out);
        }
    }

    /// Where a timbre rests, as the computer's setting chose (`settings.rs`); held notes and the
    /// shared timbre told what it changes.
    pub fn set_rest(&mut self, rest: TimbreRest, out: &mut impl FnMut(Event)) {
        if rest != self.rest {
            self.rest = rest;
            self.retell(out);
            self.tell_shared(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cc(channel: u8, cc: u8, value: u8) -> NoteEvent<()> {
        NoteEvent::MidiCC {
            timing: 0,
            channel,
            cc,
            value: f32::from(value) / 127.0,
        }
    }

    fn run(x: &mut Expressions, events: &[NoteEvent<()>]) -> Vec<Event> {
        let mut told = Vec::new();
        for &e in events {
            x.event(e, &mut |t| told.push(t));
        }
        told
    }

    /// The MCM counts only on a zone's master channel and as RPN 6; an NRPN chosen since, or
    /// none, leaves data entry alone; zones are bounded by the channels there are.
    #[test]
    fn the_configuration_message_is_taken_only_as_mpe_sends_it() {
        let mut x = Expressions::default();
        run(&mut x, &[cc(5, 101, 0), cc(5, 100, 6), cc(5, 6, 15)]);
        assert_eq!(x.zones(), [0, 0], "not a master channel");
        run(&mut x, &[cc(0, 6, 15)]);
        assert_eq!(x.zones(), [0, 0], "no RPN chosen");
        run(
            &mut x,
            &[cc(0, 101, 0), cc(0, 100, 6), cc(0, 99, 0), cc(0, 6, 15)],
        );
        assert_eq!(x.zones(), [0, 0], "an NRPN chosen since");
        run(&mut x, &[cc(0, 101, 0), cc(0, 100, 6), cc(0, 6, 127)]);
        assert_eq!(x.zones(), [15, 0]);
        assert!(x.is_member(15) && !x.is_member(0));
        run(&mut x, &[cc(15, 101, 0), cc(15, 100, 6), cc(15, 6, 3)]);
        assert_eq!(x.zones(), [11, 3]);
        assert!(x.is_member(11) && x.is_member(12) && x.is_member(14) && !x.is_member(15));
    }

    /// Without MPE, a note's own expression is what the host's note expressions and polyphonic
    /// aftertouch gave it; the shared pressure and timbre are sent once each as they change,
    /// nothing again for the same value.
    #[test]
    fn only_changes_are_sent() {
        let mut x = Expressions::default();
        let on = NoteEvent::NoteOn {
            timing: 0,
            voice_id: None,
            channel: 2,
            note: 60,
            velocity: 1.0,
        };
        assert!(run(&mut x, &[on]).is_empty(), "nothing new");
        let press = |v: f32| NoteEvent::MidiChannelPressure {
            timing: 0,
            channel: 2,
            pressure: v,
        };
        assert_eq!(
            run(&mut x, &[press(0.5), press(0.5)]),
            [Event::ChannelPressure(0.5)]
        );
        assert_eq!(
            run(&mut x, &[cc(4, 74, 127), cc(9, 74, 127)]),
            [Event::Timbre(1.0)]
        );
        let poly = NoteEvent::PolyPressure {
            timing: 0,
            voice_id: None,
            channel: 2,
            note: 60,
            pressure: 0.25,
        };
        assert_eq!(
            run(&mut x, &[poly, poly]),
            [Event::NoteExpression {
                key: 60,
                expression: Expression {
                    pressure: 0.25,
                    timbre: 0.0,
                    bend: 0.0
                }
            }]
        );
        // A member's bend range is clamped to what MPE allows.
        run(&mut x, &[cc(0, 101, 0), cc(0, 100, 6), cc(0, 6, 15)]);
        run(&mut x, &[cc(2, 101, 0), cc(2, 100, 0), cc(2, 6, 127)]);
        assert_eq!(x.range[0], 96.0);
    }

    #[test]
    fn the_modes_words_and_codes_go_both_ways() {
        for m in [Mode::Auto, Mode::Lower, Mode::Off] {
            assert_eq!(Mode::of_code(m.code()), m);
            assert_eq!(Mode::of_word(m.word()), Some(m));
        }
        assert_eq!(Mode::of_code(9), Mode::Auto);
    }

    /// TIMBRE REST's words and indexes go both ways, anything else the centre (R-MPE).
    #[test]
    fn the_timbre_rests_words_and_indexes_go_both_ways() {
        for (i, r) in TimbreRest::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(r.index()), i);
            assert_eq!(TimbreRest::from_index(r.index()), r);
            assert_eq!(TimbreRest::of_word(r.word()), Some(r));
        }
        assert_eq!(TimbreRest::from_index(9), TimbreRest::Centred);
        assert_eq!(TimbreRest::of_word("center"), None);
        assert_eq!(TimbreRest::default(), TimbreRest::Centred);
    }

    /// The centre's law (R-MPE): CC 74's 0, 64 and 127 (as a host gives them, v/127) are
    /// -1, 0 exactly and 1, and a host's brightness 0, 0.5 and 1 the same; it never falls as
    /// either rises; none heard is at rest at every rest point, and ZERO is what was received.
    #[test]
    fn the_centres_law_rests_at_64_and_at_half() {
        let c = |x: f32| TimbreRest::Centred.measured(x, UNHEARD);
        let seven = |v: u8| f32::from(v) / 127.0;
        assert_eq!((c(seven(0)), c(seven(64)), c(seven(127))), (-1.0, 0.0, 1.0));
        assert_eq!((c(0.0), c(0.5), c(1.0)), (-1.0, 0.0, 1.0));
        for v in 1..128u8 {
            assert!(c(seven(v)) >= c(seven(v - 1)), "{v}");
            assert!(c(seven(v)) > c(seven(v - 1)) || v == 64, "{v}");
        }
        for k in 1..=1000 {
            let x = k as f32 / 1000.0;
            assert!(c(x) >= c(x - 0.001) && (-1.0..=1.0).contains(&c(x)), "{x}");
        }
        for r in TimbreRest::ALL {
            assert_eq!(r.measured(UNHEARD, UNHEARD), 0.0);
            assert_eq!(r.measured(UNHEARD, 0.5), 0.0);
        }
        assert_eq!(TimbreRest::Zero.measured(seven(64), 0.25), seven(64));
        assert_eq!(TimbreRest::NoteStart.measured(0.75, 0.25), 0.5);
        assert_eq!(TimbreRest::NoteStart.measured(0.75, UNHEARD), 0.0);
    }

    /// Of a member channel's CC 74 and its note's own brightness (both in one note, which MPE
    /// and a host would rarely both send), the one farther from rest; at ZERO the larger, as
    /// R-MPE had it (R-MPE).
    #[test]
    fn a_notes_own_timbre_and_its_channels_take_the_one_farther_from_rest() {
        let mut x = Expressions::default();
        run(&mut x, &[cc(0, 101, 0), cc(0, 100, 6), cc(0, 6, 15)]);
        let on = NoteEvent::NoteOn {
            timing: 0,
            voice_id: None,
            channel: 1,
            note: 60,
            velocity: 1.0,
        };
        let bright = |b: f32| NoteEvent::PolyBrightness {
            timing: 0,
            voice_id: None,
            channel: 1,
            note: 60,
            brightness: b,
        };
        run(&mut x, &[cc(1, 74, 32), on, bright(0.6)]);
        assert_eq!(x.of(1, 60).timbre, 32.0 / 127.0 * 2.0 - 1.0);
        run(&mut x, &[bright(0.0)]);
        assert_eq!(x.of(1, 60).timbre, -1.0);
        x.rest = TimbreRest::Zero;
        assert_eq!(x.of(1, 60).timbre, 32.0 / 127.0);
        run(&mut x, &[bright(0.75)]);
        assert_eq!(x.of(1, 60).timbre, 0.75);
    }
}
