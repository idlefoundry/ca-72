//! QUALITY's opening as the editor moves it (decisions.md R-ULTRA; the panel draws it,
//! `ca72_panel::ultra`): what each setting shows coming up and going down, one at a time; the
//! synth's level as the lamps show it; LO's hamster's run.

use ca72_panel::ultra::{self, Opening};

use crate::params::QualityMode;

/// The synth's level shown: a voice's output's peak (the engine's, `Engine::levels`) at or
/// under [`FLOOR_DB`] is silent, at [`TOP_DB`] or over it loud, between them as decibels go.
pub const FLOOR_DB: f64 = -42.0;
pub const TOP_DB: f64 = 0.0;
/// The lamps' filament: how fast their light follows the level, rising and falling, seconds.
const RISE_S: f64 = 0.05;
const FALL_S: f64 = 0.3;
/// The hamster: his wheel's turn at a full run (degrees a second), a frame of his run at a full
/// run (seconds), how fast he picks up a run and drops it (per second), how loud the synth must
/// be for him to run, and how long he stands before he dozes off and takes to doze.
const SPIN: f64 = 137.5;
const FRAME_S: f64 = 0.075;
const PACE: f64 = 3.0;
const RUN_AT: f64 = 0.05;
const DOZE_AFTER: f64 = 1.0;
const DOZE_S: f64 = 0.7;

/// A peak as the level shown (0 to 1).
pub fn level_of(peak: f64) -> f64 {
    if peak.is_nan() || peak <= 0.0 {
        return 0.0;
    }
    ((20.0 * peak.log10() - FLOOR_DB) / (TOP_DB - FLOOR_DB)).clamp(0.0, 1.0)
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// What the opening is doing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Motion {
    /// How far each has come: LO's hamster, HI's lamp, ULTRA's Tesla lamp.
    pub hamster: f64,
    pub lamp: f64,
    pub coil: f64,
    /// The lamps' level, as their filament follows the synth's.
    pub level: f64,
    /// The hamster's run (0 to 1), how far through his run's frames (seconds at a full run),
    /// his wheel's turn (degrees), how long he has stood still (seconds).
    pub run: f64,
    pub stride: f64,
    pub turn: f64,
    pub stood: f64,
    /// The lightning's clock (seconds), moving only while the coil sparks.
    pub spark: f64,
}

impl Motion {
    /// As the editor opens at `q`: what it shows there, up; the synth as quiet as `level`
    /// says (the hamster asleep if it is quiet).
    pub fn at(q: QualityMode, level: f64) -> Motion {
        let mut m = Motion {
            level,
            ..Motion::default()
        };
        match q {
            QualityMode::Lo => m.hamster = 1.0,
            QualityMode::Hi => m.lamp = 1.0,
            QualityMode::Ultra => m.coil = 1.0,
        }
        if level < RUN_AT {
            m.stood = DOZE_AFTER + DOZE_S;
        }
        m
    }

    /// Moved on `dt` seconds towards what `q` shows, the synth's level `level` (0 to 1): what is
    /// up and not wanted goes down first, then the wanted one comes up.
    pub fn step(&mut self, dt: f64, q: QualityMode, level: f64) {
        let want = [
            q == QualityMode::Lo,
            q == QualityMode::Hi,
            q == QualityMode::Ultra,
        ];
        let (up, down) = (ultra::OPEN_S, ultra::CLOSE_S);
        let mut all = [self.hamster, self.lamp, self.coil];
        if let Some(k) = (0..3).find(|&k| all[k] > 0.0 && !want[k]) {
            all[k] = (all[k] - dt / down).max(0.0);
        } else if let Some(k) = (0..3).find(|&k| want[k]) {
            all[k] = (all[k] + dt / up).min(1.0);
        }
        [self.hamster, self.lamp, self.coil] = all;
        // The filament.
        let s = if level > self.level { RISE_S } else { FALL_S };
        self.level += (level - self.level) * (1.0 - (-dt / s).exp());
        if (self.level - level).abs() < 1e-3 {
            self.level = level;
        }
        // The hamster: running already as the shutter opens on him and his wheel comes forward
        // (the owner: "don't wait for the hamster to come forward for him to start running. He
        // should already be running"), and as it goes down; up, running while the synth sounds.
        let wanted = if self.hamster <= 0.0 {
            0.0
        } else if self.hamster < 1.0 || level >= RUN_AT {
            1.0
        } else {
            0.0
        };
        if self.hamster < 1.0 {
            self.run = wanted;
        } else {
            self.run += (wanted - self.run) * (1.0 - (-dt * PACE).exp());
            if (self.run - wanted).abs() < 3e-3 {
                self.run = wanted;
            }
        }
        if self.run > 0.0 {
            self.stride += dt * self.run;
            self.turn = (self.turn + dt * self.run * SPIN) % 360.0;
        }
        self.stood = if self.run > 0.01 || self.hamster < 1.0 {
            0.0
        } else {
            self.stood + dt
        };
        // The lightning, while the coil sparks.
        if self.coil > 0.0 && self.level > 0.0 {
            self.spark += dt;
        }
    }

    /// Whether something is coming up or going down.
    pub fn moving(&self) -> bool {
        [self.hamster, self.lamp, self.coil]
            .iter()
            .any(|&v| v > 0.0 && v < 1.0)
    }

    /// The opening drawn (`hidden`: none of it, the panel blank above QUALITY).
    pub fn opening(&self, hidden: bool) -> Opening {
        let q = |v: f64, n: f64| (v * n).round() / n;
        Opening {
            hamster: self.hamster,
            lamp: self.lamp,
            coil: self.coil,
            hidden,
            level: q(self.level, 100.0),
            run: q(self.run, 50.0),
            stride: ((self.stride / FRAME_S).floor() as i64)
                .rem_euclid(ca72_panel::skin::STRIDE as i64) as u8,
            asleep: if self.hamster >= 1.0 {
                smooth(DOZE_AFTER, DOZE_AFTER + DOZE_S, self.stood)
            } else {
                0.0
            },
            turn: q(self.turn, 4.0),
            spark: q(self.spark, 30.0),
        }
    }
}

/// The opening and the drops move on at most this often, seconds: thirty times a second.
pub const ANIMATE_S: f64 = 1.0 / 30.0;

#[cfg(test)]
mod tests {
    use super::*;

    /// A level: silent at the floor and under, loud at the top and over, between them as
    /// decibels go.
    #[test]
    fn a_peak_shows_as_a_level() {
        assert_eq!(level_of(0.0), 0.0);
        assert_eq!(level_of(f64::NAN), 0.0);
        assert_eq!(level_of(10f64.powf(FLOOR_DB / 20.0)), 0.0);
        assert_eq!(level_of(1.0), 1.0);
        let mid = level_of(10f64.powf((FLOOR_DB + TOP_DB) / 40.0));
        assert!((mid - 0.5).abs() < 1e-9, "{mid}");
    }

    /// Moving from HI to LO: the lamp goes down first, then the hamster comes up; to ULTRA
    /// from LO, the hamster down, then the coil up.
    #[test]
    fn one_goes_down_before_the_next_comes_up() {
        let mut m = Motion::at(QualityMode::Hi, 0.0);
        let mut t = 0.0;
        while m.lamp > 0.0 {
            m.step(0.02, QualityMode::Lo, 0.0);
            assert_eq!(m.hamster, 0.0);
            t += 0.02;
        }
        assert!((t - ultra::CLOSE_S).abs() < 0.03, "{t}");
        while m.hamster < 1.0 {
            m.step(0.02, QualityMode::Lo, 0.0);
            assert!(m.moving() || m.hamster == 1.0);
        }
        m.step(0.02, QualityMode::Ultra, 0.0);
        assert!(m.hamster < 1.0 && m.coil == 0.0);
    }

    /// The lamps' light: quick to rise with the synth, slower to fall.
    #[test]
    fn the_lamps_follow_the_synth_as_a_filament() {
        let mut m = Motion::at(QualityMode::Hi, 0.0);
        m.step(0.05, QualityMode::Hi, 1.0);
        assert!(m.level > 0.6, "{}", m.level);
        let up = m.level;
        m.step(0.05, QualityMode::Hi, 0.0);
        assert!(m.level > 0.5 * up, "{}", m.level);
        for _ in 0..100 {
            m.step(0.05, QualityMode::Hi, 0.0);
        }
        assert_eq!(m.level, 0.0);
        assert_eq!(m.opening(false).level, 0.0);
    }

    /// The hamster runs while the synth sounds, his frames and wheel moving on; quiet, he
    /// slows, stands and dozes off, and then nothing moves.
    #[test]
    fn the_hamster_runs_while_the_synth_sounds() {
        let mut m = Motion::at(QualityMode::Lo, 0.0);
        assert_eq!(
            m.opening(false).asleep,
            1.0,
            "the editor opens on him asleep"
        );
        for _ in 0..60 {
            m.step(1.0 / 30.0, QualityMode::Lo, 0.8);
        }
        let o = m.opening(false);
        assert!(o.run > 0.9 && o.asleep == 0.0 && m.turn > 0.0, "{o:?}");
        for _ in 0..150 {
            m.step(1.0 / 30.0, QualityMode::Lo, 0.0);
        }
        let a = m.opening(false);
        assert!(a.run == 0.0 && a.asleep == 1.0, "{a:?}");
        m.step(1.0 / 30.0, QualityMode::Lo, 0.0);
        assert_eq!(m.opening(false), a, "at rest nothing moves");
    }

    /// The coil's lightning moves only while it sparks.
    #[test]
    fn the_lightning_moves_only_while_the_coil_sparks() {
        let mut m = Motion::at(QualityMode::Ultra, 0.0);
        m.step(0.1, QualityMode::Ultra, 0.0);
        assert_eq!(m.spark, 0.0);
        m.step(0.1, QualityMode::Ultra, 0.7);
        assert!(m.spark > 0.0);
    }
}
