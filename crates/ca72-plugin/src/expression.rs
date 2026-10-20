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

use nih_plug::prelude::NoteEvent;

use crate::engine::{BEND_MOST, Event, Expression};

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

/// The record (fixed storage: some 26 KB, nothing allocated after it is made).
#[derive(Debug)]
pub struct Expressions {
    pub mode: Mode,
    /// The member channels each zone's MCM gave (lower, upper; 0, none) and each zone's
    /// members' bend range, semitones.
    mcm: [u8; 2],
    range: [f32; 2],
    /// Each channel's RPN chosen, and its pitch bend (-1..1), pressure and timbre (0..1).
    rpn: [u16; 16],
    bend: [f32; 16],
    pressure: [f32; 16],
    timbre: [f32; 16],
    /// The keys held, by channel; each note's own pressure, timbre (0..1) and tuning
    /// (semitones), as a host's note expressions or polyphonic aftertouch give them.
    held: [u128; 16],
    key_pressure: [[f32; 128]; 16],
    key_timbre: [[f32; 128]; 16],
    key_tuning: [[f32; 128]; 16],
    /// The pressure and timbre every voice shares, as last received outside a zone.
    shared: [f32; 2],
    /// What the engine was last told: each key's expression, and the shared pressure and timbre.
    sent: [Expression; 128],
    sent_shared: [f32; 2],
}

impl Default for Expressions {
    fn default() -> Self {
        Expressions {
            mode: Mode::Auto,
            mcm: [0; 2],
            range: [MEMBER_BEND; 2],
            rpn: [NULL; 16],
            bend: [0.0; 16],
            pressure: [0.0; 16],
            timbre: [0.0; 16],
            held: [0; 16],
            key_pressure: [[0.0; 128]; 16],
            key_timbre: [[0.0; 128]; 16],
            key_tuning: [[0.0; 128]; 16],
            shared: [0.0; 2],
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
        let mut e = Expression {
            pressure: self.key_pressure[c][k],
            timbre: self.key_timbre[c][k],
            bend: self.key_tuning[c][k],
        };
        if let Some(zone) = self.zone_of(channel) {
            e.pressure = e.pressure.max(self.pressure[c]);
            e.timbre = e.timbre.max(self.timbre[c]);
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
                self.held[c] |= 1 << note;
                // (A new note's own expression starts from nothing; its member channel's, as
                // MPE sends it before the note, stays.)
                self.key_pressure[c][k] = 0.0;
                self.key_timbre[c][k] = 0.0;
                self.key_tuning[c][k] = 0.0;
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
                if self.held(channel, note) {
                    self.key_timbre[usize::from(channel)][usize::from(note)] = unit(brightness);
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
                    self.pressure[usize::from(channel)] = unit(pressure);
                    self.tell_channel(channel, out);
                    true
                } else {
                    self.shared[0] = unit(pressure);
                    self.tell_shared(out);
                    false
                }
            }
            NoteEvent::MidiPitchBend { channel, value, .. } if channel < 16 => {
                if !self.is_member(channel) {
                    return false;
                }
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
                if self.is_member(channel) {
                    self.timbre[c] = unit(value);
                    self.tell_channel(channel, out);
                    return true;
                }
                self.shared[1] = unit(value);
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
        self.key_timbre = [[0.0; 128]; 16];
        self.key_tuning = [[0.0; 128]; 16];
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
        if self.shared[1] != self.sent_shared[1] {
            self.sent_shared[1] = self.shared[1];
            out(Event::Timbre(self.shared[1]));
        }
    }

    /// The mode the computer's setting chose (`settings.rs`); held notes told what it changes.
    pub fn set_mode(&mut self, mode: Mode, out: &mut impl FnMut(Event)) {
        if mode != self.mode {
            self.mode = mode;
            self.retell(out);
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
}
