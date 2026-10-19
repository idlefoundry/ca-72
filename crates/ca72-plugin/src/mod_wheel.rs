//! The computer's MOD wheel source (decisions.md R-MODWHEEL-SOURCE). Every source feeds the
//! engine's existing MIDI modulation path; the wheel's parameter and presets stay as they are.

use nih_plug::prelude::NoteEvent;

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

/// Fixed storage, indexed by MIDI channel and key. Pressure from a released key cannot keep
/// the wheel up, nor can an old source's value return when the player switches back to it.
#[derive(Debug)]
pub struct Wheel {
    pub seen: u64,
    pub source: Source,
    held: [u128; 16],
    pressure: [[f32; 128]; 16],
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            seen: u64::from(Source::DEFAULT.code()),
            source: Source::DEFAULT,
            held: [0; 16],
            pressure: [[0.0; 128]; 16],
        }
    }
}

impl Wheel {
    pub fn reset_pressure(&mut self) {
        self.pressure = [[0.0; 128]; 16];
    }

    pub fn release(&mut self) {
        self.held = [0; 16];
        self.reset_pressure();
    }

    fn maximum(&self) -> f32 {
        self.pressure.iter().flatten().copied().fold(0.0, f32::max)
    }

    /// The modulation value, if this event moves it. Note ownership is followed even with
    /// another source chosen, so aftertouch also works on keys already held when selected.
    pub fn event(&mut self, event: NoteEvent<()>) -> Option<f32> {
        match event {
            NoteEvent::NoteOn { channel, note, .. } if channel < 16 && note < 128 => {
                self.held[channel as usize] |= 1 << note;
                self.pressure[channel as usize][note as usize] = 0.0;
                (self.source == Source::PolyPressure).then(|| self.maximum())
            }
            NoteEvent::NoteOff { channel, note, .. } | NoteEvent::Choke { channel, note, .. }
                if channel < 16 && note < 128 =>
            {
                self.held[channel as usize] &= !(1 << note);
                self.pressure[channel as usize][note as usize] = 0.0;
                (self.source == Source::PolyPressure).then(|| self.maximum())
            }
            NoteEvent::PolyPressure {
                channel,
                note,
                pressure,
                ..
            } if self.source == Source::PolyPressure && channel < 16 && note < 128 => {
                if self.held[channel as usize] & (1 << note) == 0 {
                    return None;
                }
                self.pressure[channel as usize][note as usize] = if pressure.is_finite() {
                    pressure.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                Some(self.maximum())
            }
            NoteEvent::MidiChannelPressure { pressure, .. }
                if self.source == Source::ChannelPressure =>
            {
                Some(pressure)
            }
            _ => None,
        }
    }
}
