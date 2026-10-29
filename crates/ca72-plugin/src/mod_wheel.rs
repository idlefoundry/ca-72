//! The computer's MOD wheel source (decisions.md R-MODWHEEL-SOURCE). Every source feeds the
//! engine's existing MIDI modulation path; the wheel's parameter and presets stay as they are.
//! Its aftertouch reads the pressure each voice's expression reads (R-MPE).

use nih_plug::prelude::NoteEvent;

use crate::expression::Expressions;
use crate::learn::reserved;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Cc(u8),
    ChannelPressure,
    PolyPressure,
}

impl Default for Source {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Source {
    pub const DEFAULT: Self = Self::Cc(1);

    pub fn cc(cc: u8) -> Option<Self> {
        reserved(cc).is_none().then_some(Self::Cc(cc))
    }

    pub fn code(self) -> u8 {
        match self {
            Self::Cc(cc) => cc,
            Self::ChannelPressure => 128,
            Self::PolyPressure => 129,
        }
    }

    pub fn of_code(code: u8) -> Self {
        match code {
            128 => Self::ChannelPressure,
            129 => Self::PolyPressure,
            cc => Self::cc(cc).unwrap_or(Self::DEFAULT),
        }
    }

    pub fn text(self) -> String {
        match self {
            Self::Cc(cc) => format!("CC {cc}"),
            Self::ChannelPressure => "CHANNEL AFTERTOUCH".into(),
            Self::PolyPressure => "POLYPHONIC AFTERTOUCH".into(),
        }
    }
}

/// The wheel's source and what it has heard. The pressures themselves are the notes' own
/// record (`expression.rs`, shared with each voice's expression: decisions.md R-MPE); the wheel
/// keeps only which keys' pressure arrived since its source was chosen, so that pressure from a
/// released key cannot keep the wheel up, nor can an old source's value return when the player
/// switches back to it.
#[derive(Debug)]
pub struct Wheel {
    pub seen: u64,
    pub source: Source,
    fresh: [u128; 16],
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            seen: u64::from(Source::DEFAULT.code()),
            source: Source::DEFAULT,
            fresh: [0; 16],
        }
    }
}

impl Wheel {
    pub fn reset_pressure(&mut self) {
        self.fresh = [0; 16];
    }

    pub fn release(&mut self) {
        self.reset_pressure();
    }

    /// The modulation value, if this event moves it, read from `notes` as the event has left
    /// them. Note ownership is followed even with another source chosen, so aftertouch also
    /// works on keys already held when selected. An MPE member channel's pressure is its note's
    /// own, polyphonic aftertouch's; outside a zone channel pressure is channel aftertouch.
    pub fn event(&mut self, event: NoteEvent<()>, notes: &Expressions) -> Option<f32> {
        let poly = self.source == Source::PolyPressure;
        match event {
            NoteEvent::NoteOn { channel, note, .. }
            | NoteEvent::NoteOff { channel, note, .. }
            | NoteEvent::Choke { channel, note, .. }
                if channel < 16 && note < 128 =>
            {
                self.fresh[channel as usize] &= !(1 << note);
                poly.then(|| notes.most_pressure(&self.fresh))
            }
            NoteEvent::PolyPressure { channel, note, .. } if poly && channel < 16 && note < 128 => {
                if !notes.held(channel, note) {
                    return None;
                }
                self.fresh[channel as usize] |= 1 << note;
                Some(notes.most_pressure(&self.fresh))
            }
            NoteEvent::MidiChannelPressure {
                channel, pressure, ..
            } if channel < 16 => {
                if notes.is_member(channel) {
                    poly.then(|| {
                        self.fresh[channel as usize] |= notes.held_on(channel);
                        notes.most_pressure(&self.fresh)
                    })
                } else {
                    (self.source == Source::ChannelPressure).then_some(pressure)
                }
            }
            _ => None,
        }
    }
}
