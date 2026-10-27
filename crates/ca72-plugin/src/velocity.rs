//! The velocity each note reaching the engine plays at (decisions.md R-VELOCITY): a key's as the
//! player struck it; the arpeggiator's notes, which come by key alone, the velocity of the key
//! they come from.

use crate::engine::{Engine, Event};

/// A note from a source with no velocity, or before any key was struck: full.
pub const FULL: f32 = 1.0;

/// Each key's velocity as last struck, and when.
#[derive(Debug, Clone)]
pub struct Played {
    velocity: [f32; 128],
    at: [u64; 128],
    count: u64,
    last: f32,
}

impl Default for Played {
    fn default() -> Self {
        Played {
            velocity: [FULL; 128],
            at: [0; 128],
            count: 0,
            last: FULL,
        }
    }
}

impl Played {
    /// Key `key` struck at `velocity` (0..1).
    pub fn struck(&mut self, key: u8, velocity: f32) {
        let i = usize::from(key);
        if i >= 128 {
            return;
        }
        let v = if velocity.is_finite() {
            velocity.clamp(0.0, 1.0)
        } else {
            FULL
        };
        self.count += 1;
        (self.velocity[i], self.at[i], self.last) = (v, self.count, v);
    }

    /// The velocity a note on `key` plays at: that of the key of its pitch class struck latest
    /// (the key itself, or the one the arpeggiator's octaves come from), else the latest struck.
    pub fn of(&self, key: u8) -> f32 {
        (usize::from(key % 12)..128)
            .step_by(12)
            .filter(|&k| self.at[k] > 0)
            .max_by_key(|&k| self.at[k])
            .map_or(self.last, |k| self.velocity[k])
    }

    /// A note to the engine: its key's velocity first when it starts.
    pub fn to_engine(&self, engine: &mut Engine, key: u8, on: bool) {
        if on {
            engine.event(Event::NoteVelocity {
                key,
                velocity: self.of(key),
            });
        }
        engine.event(Event::Note { key, on });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_plays_at_its_own_velocity() {
        let mut p = Played::default();
        p.struck(60, 0.25);
        p.struck(64, 0.75);
        assert_eq!(p.of(60), 0.25);
        assert_eq!(p.of(64), 0.75);
    }

    #[test]
    fn an_octave_takes_the_key_it_comes_from() {
        let mut p = Played::default();
        p.struck(72, 0.9);
        p.struck(60, 0.3);
        // (72 was struck before 60: the arpeggiator's octave of 60 takes 60's.)
        assert_eq!(p.of(72), 0.3);
        assert_eq!(p.of(84), 0.3);
        assert_eq!(p.of(48), 0.3);
    }

    #[test]
    fn a_key_never_struck_takes_the_latest() {
        let mut p = Played::default();
        assert_eq!(p.of(61), FULL);
        p.struck(60, 0.4);
        assert_eq!(p.of(61), 0.4);
    }

    #[test]
    fn velocities_are_kept_within_0_to_1() {
        let mut p = Played::default();
        p.struck(60, 1.5);
        assert_eq!(p.of(60), 1.0);
        p.struck(60, -1.0);
        assert_eq!(p.of(60), 0.0);
        p.struck(60, f32::NAN);
        assert_eq!(p.of(60), FULL);
        p.struck(200, 0.5);
        assert_eq!(p.of(60), FULL);
    }
}
