//! DRIVE's AUTO GAIN (decisions.md R-STEREO, the CA-74's R29): the output brought back down by
//! as much as DRIVE made the sound louder, measured for the sound itself. Off the audio thread,
//! the plug-in's helper plays a few short notes through voices made as the engine's first are,
//! with the sound's panel, without DRIVE and at each of its steps, and compares their loudness
//! (K-weighted, as BS.1770 weighs it): the correction at each step, a curve. The curve is saved
//! with the session, so that the session plays and renders the same again; it is measured again
//! only when the sound is changed in the plug-in's own window (a preset chosen there among the
//! changes), never by the host's automation or a learned controller, so that a render does not
//! depend on when a measurement finished. Until a sound has been measured, the factory
//! presets' average stands in ([`Curve::AVERAGE`]). The audio thread only reads the curve,
//! without a lock ([`Calibration::read`]).

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use nih_plug::params::persist::PersistentField;
use serde::{Deserialize, Serialize};

use crate::engine::{Controls, DRIVE_TOP, Probe};

/// The steps of DRIVE the curve is measured at, [`STEP`] dB apart up to [`DRIVE_TOP`]; between
/// them (and from none to the first) it is drawn straight (the CA-74's: measured there at steps
/// half as far apart, its presets' corrections came out no better for twice the work).
pub const STEPS: usize = 4;
pub const STEP: f64 = DRIVE_TOP / STEPS as f64;

/// The notes the measurement plays together, each held [`HELD`] s and heard for [`HEARD`] s:
/// a low, a middle and a high one, as the sound might be played anywhere on the keyboard.
pub const KEYS: [u8; 3] = [36, 55, 72];
pub const HELD: f64 = 0.3;
pub const HEARD: f64 = 0.45;

/// The correction at each of DRIVE's steps, dB (0 without DRIVE).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Curve(pub [f32; STEPS]);

impl Curve {
    /// The factory presets' average, measured (`tests/drive_auto.rs`): the correction for a
    /// sound not yet measured.
    pub const AVERAGE: Curve = Curve([-4.74, -8.43, -10.95, -12.42]);

    /// The correction at DRIVE `db`, dB: drawn straight between the steps.
    pub fn at(&self, db: f64) -> f64 {
        if !db.is_finite() || db <= 0.0 {
            return 0.0;
        }
        let x = (db.min(DRIVE_TOP) / STEP).min(STEPS as f64);
        let i = (x.ceil() as usize).clamp(1, STEPS);
        let below = if i == 1 {
            0.0
        } else {
            f64::from(self.0[i - 2])
        };
        let above = f64::from(self.0[i - 1]);
        below + (above - below) * (x - (i - 1) as f64)
    }
}

/// The curve's key in the plug-in's state.
pub const STATE_KEY: &str = "drive_curve";

/// The curve as a session saves it: the sound it was measured for (0: none, the average's) and
/// its corrections.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Saved {
    pub sound: u64,
    pub db: [f32; STEPS],
}

impl Default for Saved {
    fn default() -> Self {
        Saved {
            sound: 0,
            db: Curve::AVERAGE.0,
        }
    }
}

/// The key of a sound as far as DRIVE's loudness goes: the panel, LOCK, ENTROPY (each voice its
/// own parts) and FEEDBACK (the voice's output into its own external input). Not POLY, UNISON,
/// VOICES, DOUBLE or SPREAD: a note's voices each have their own filter, so a chord, a unison
/// or a pair sum what one voice does (their levels are their own trims'). Never 0.
pub fn sound(c: &Controls) -> u64 {
    // (FNV-1a over the debug form: every field of the panel, the floats to the bit.)
    let text = format!("{:?}", (c.panel, c.lock, c.entropy, c.feedback));
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h | 1
}

/// The curve, shared by the editor (which asks for a sound to be measured), the plug-in's
/// helper thread (which measures it), the audio thread (which reads it) and the host (which
/// saves and loads it with the session).
#[derive(Debug)]
pub struct Calibration {
    /// The curve: its writes (odd while one is under way: a reader then keeps what it had), the
    /// sound it was measured for and its corrections (f32's bits). One writer at a time.
    writes: AtomicU64,
    sound: AtomicU64,
    db: [AtomicU32; STEPS],
    writing: Mutex<()>,
    /// The sound the curve is wanted for (the editor's latest, or a session's), the one to
    /// measure next if it is not the curve's already, and the rate to measure it at (f64's
    /// bits; 0 before the engine is prepared). A measurement finished for another sound than
    /// the one wanted is let go.
    wanted: AtomicU64,
    asked: Mutex<Option<(u64, Controls)>>,
    rate: AtomicU64,
    /// The helper thread, woken when a sound is asked for.
    helper: Mutex<Option<std::thread::Thread>>,
}

impl Default for Calibration {
    fn default() -> Self {
        let c = Calibration {
            writes: AtomicU64::new(0),
            sound: AtomicU64::new(0),
            db: std::array::from_fn(|_| AtomicU32::new(0)),
            writing: Mutex::new(()),
            wanted: AtomicU64::new(0),
            asked: Mutex::new(None),
            rate: AtomicU64::new(0),
            helper: Mutex::new(None),
        };
        c.store(&Saved::default());
        c
    }
}

impl Calibration {
    /// The curve and the sound it was measured for, as it stands (not on the audio thread: it
    /// waits out a write).
    pub fn saved(&self) -> Saved {
        let _w = self.writing.lock();
        Saved {
            sound: self.sound.load(Ordering::Acquire),
            db: std::array::from_fn(|i| f32::from_bits(self.db[i].load(Ordering::Acquire))),
        }
    }

    /// The curve, if it has been written since the reader last had it (`seen`, its writes
    /// then): for the audio thread, without a lock. A write under way leaves it to the next
    /// block.
    pub fn read(&self, seen: &mut u64) -> Option<Curve> {
        let before = self.writes.load(Ordering::Acquire);
        if before == *seen || before % 2 == 1 {
            return None;
        }
        let db: [f32; STEPS] =
            std::array::from_fn(|i| f32::from_bits(self.db[i].load(Ordering::Acquire)));
        std::sync::atomic::fence(Ordering::Acquire);
        if self.writes.load(Ordering::Relaxed) != before {
            return None;
        }
        *seen = before;
        Some(Curve(db))
    }

    /// The curve written: the helper's measurement, or a session's.
    fn store(&self, s: &Saved) {
        let _w = self.writing.lock();
        self.writes.fetch_add(1, Ordering::AcqRel);
        std::sync::atomic::fence(Ordering::Release);
        self.sound.store(s.sound, Ordering::Release);
        for (a, v) in self.db.iter().zip(s.db) {
            let v = if v.is_finite() {
                v.clamp(-60.0, 12.0)
            } else {
                0.0
            };
            a.store(v.to_bits(), Ordering::Release);
        }
        self.writes.fetch_add(1, Ordering::AcqRel);
    }

    /// The sound `c` wanted: measured unless the curve is already its (a measurement under
    /// way for another let go), the helper woken. From the editor, once a change made there is
    /// done.
    pub fn ask(&self, c: &Controls) {
        let key = sound(c);
        if self.wanted.swap(key, Ordering::AcqRel) == key {
            return;
        }
        let measured = self.sound.load(Ordering::Acquire) == key;
        if let Ok(mut a) = self.asked.lock() {
            *a = (!measured).then_some((key, *c));
        }
        if !measured {
            self.wake();
        }
    }

    /// A session's curve: the instance's replaced, nothing left to measure (a measurement under
    /// way is for a sound the session has replaced).
    fn load(&self, s: &Saved) {
        if let Ok(mut a) = self.asked.lock() {
            self.wanted.store(s.sound, Ordering::Release);
            *a = None;
        }
        self.store(s);
    }

    fn wake(&self) {
        if let Ok(h) = self.helper.lock()
            && let Some(t) = h.as_ref()
        {
            t.unpark();
        }
    }

    /// The rate the voices play at, for measuring at it.
    pub fn set_rate(&self, rate: f64) {
        self.rate.store(rate.to_bits(), Ordering::Release);
    }

    /// The helper thread that measures (None: none now).
    pub fn set_helper(&self, t: Option<std::thread::Thread>) {
        if let Ok(mut h) = self.helper.lock() {
            *h = t;
        }
    }

    /// The measurement a step further, on the helper thread: a sound newly asked for begun
    /// (one under way given up for it), else the next of its renders. Whether there is more
    /// to do.
    pub fn step(&self, job: &mut Option<Job>) -> bool {
        let rate = f64::from_bits(self.rate.load(Ordering::Acquire));
        if rate > 0.0
            && let Some((key, c)) = self.asked.lock().ok().and_then(|mut a| a.take())
        {
            *job = Some(Job::new(key, &c, rate));
            return true;
        }
        let Some(j) = job.as_mut() else {
            return false;
        };
        let wanted = self.wanted.load(Ordering::Acquire);
        if j.sound == wanted && j.step() {
            return true;
        }
        if let Ok(_a) = self.asked.lock()
            && j.sound == self.wanted.load(Ordering::Acquire)
            && let Some(saved) = j.done()
        {
            self.store(&saved);
        }
        *job = None;
        false
    }

    /// The sound asked for measured now, on this thread (the tests; an engine without its
    /// helper).
    pub fn measure_now(&self) {
        let mut job = None;
        while self.step(&mut job) {}
    }
}

/// A sound's measurement under way: its key, the voices to copy for each render, and the
/// loudness without DRIVE and at the steps measured so far.
#[derive(Debug)]
pub struct Job {
    sound: u64,
    probe: Probe,
    reference: Option<f64>,
    db: Vec<f32>,
}

impl Job {
    fn new(sound: u64, c: &Controls, rate: f64) -> Job {
        Job {
            sound,
            probe: Probe::new(c, rate),
            reference: None,
            db: Vec::with_capacity(STEPS),
        }
    }

    /// The next render: without DRIVE first, then each step. Whether there are more.
    fn step(&mut self) -> bool {
        match self.reference {
            None => {
                self.reference = Some(self.probe.energy(0.0));
                true
            }
            Some(r) if self.db.len() < STEPS => {
                let at = STEP * (self.db.len() + 1) as f64;
                let e = self.probe.energy(at);
                let db = if r > 0.0 && e > 0.0 {
                    10.0 * libm::log10(r / e)
                } else {
                    0.0
                };
                self.db.push(db as f32);
                self.db.len() < STEPS
            }
            Some(_) => false,
        }
    }

    fn done(&self) -> Option<Saved> {
        (self.db.len() == STEPS).then(|| Saved {
            sound: self.sound,
            db: std::array::from_fn(|i| self.db[i]),
        })
    }
}

/// K-weighting (BS.1770's two stages at any rate, as libebur128 derives them) and the energy
/// of what passes through it.
#[derive(Debug, Clone)]
pub struct KWeighted {
    stages: [([f64; 3], [f64; 2]); 2],
    state: [[f64; 4]; 2],
    pub energy: f64,
}

impl KWeighted {
    pub fn new(rate: f64) -> KWeighted {
        use std::f64::consts::PI;
        let (f0, g, q) = (
            1_681.974_450_955_533,
            3.999_843_853_973_347,
            0.707_175_236_955_419_6,
        );
        let k = libm::tan(PI * f0 / rate);
        let vh = libm::pow(10.0, g / 20.0);
        let vb = libm::pow(vh, 0.499_666_774_154_541_6);
        let a0 = 1.0 + k / q + k * k;
        let shelf = (
            [
                (vh + vb * k / q + k * k) / a0,
                2.0 * (k * k - vh) / a0,
                (vh - vb * k / q + k * k) / a0,
            ],
            [2.0 * (k * k - 1.0) / a0, (1.0 - k / q + k * k) / a0],
        );
        let (f0, q) = (38.135_470_876_024_44, 0.500_327_037_323_877_3);
        let k = libm::tan(PI * f0 / rate);
        let d = 1.0 + k / q + k * k;
        let high_pass = (
            [1.0, -2.0, 1.0],
            [2.0 * (k * k - 1.0) / d, (1.0 - k / q + k * k) / d],
        );
        KWeighted {
            stages: [shelf, high_pass],
            state: [[0.0; 4]; 2],
            energy: 0.0,
        }
    }

    pub fn push(&mut self, x: f64) {
        let mut y = x;
        for ((b, a), s) in self.stages.iter().zip(&mut self.state) {
            let z = b[0] * y + b[1] * s[0] + b[2] * s[1] - a[0] * s[2] - a[1] * s[3];
            *s = [y, s[0], z, s[2]];
            y = z;
        }
        self.energy += y * y;
    }
}

/// The curve saved and loaded with the session: a session's replaces the instance's (one
/// saved before DRIVE has none, and is given the average by `filter_state`).
impl<'a> PersistentField<'a, Saved> for Arc<Calibration> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_is_drawn_straight_between_its_steps() {
        let c = Curve([-6.0, -11.0, -15.0, -18.0]);
        assert_eq!(c.at(0.0), 0.0);
        assert_eq!(c.at(-3.0), 0.0);
        assert!((c.at(3.0) + 3.0).abs() < 1e-9);
        assert!((c.at(6.0) + 6.0).abs() < 1e-9);
        assert!((c.at(9.0) + 8.5).abs() < 1e-9);
        assert!((c.at(24.0) + 18.0).abs() < 1e-9);
        assert!((c.at(30.0) + 18.0).abs() < 1e-9);
        assert_eq!(c.at(f64::NAN), 0.0);
    }

    #[test]
    fn a_reader_sees_each_write_once_and_never_half_of_one() {
        let c = Calibration::default();
        let mut seen = 0;
        assert_eq!(c.read(&mut seen), Some(Curve::AVERAGE));
        assert_eq!(c.read(&mut seen), None);
        c.store(&Saved {
            sound: 7,
            db: [-1.0, -2.0, -3.0, -4.0],
        });
        assert_eq!(c.read(&mut seen), Some(Curve([-1.0, -2.0, -3.0, -4.0])));
        assert_eq!(c.saved().sound, 7);
        // A write under way (odd): the reader keeps what it had.
        c.writes.fetch_add(1, Ordering::AcqRel);
        let mut fresh = 0;
        assert_eq!(c.read(&mut fresh), None);
    }

    #[test]
    fn a_sound_is_asked_for_once_and_not_when_it_is_the_curves() {
        let c = Calibration::default();
        let controls = Controls::default();
        c.ask(&controls);
        assert!(c.asked.lock().unwrap().is_some());
        c.store(&Saved {
            sound: sound(&controls),
            db: [0.0; STEPS],
        });
        let mut other = controls;
        other.panel.cutoff = 0.2;
        assert_ne!(sound(&other), sound(&controls));
        c.ask(&other);
        c.ask(&controls);
        assert!(c.asked.lock().unwrap().is_none());
        let mut louder = controls;
        (louder.drive, louder.level, louder.spread, louder.poly) = (12.0, -6.0, 1.0, true);
        assert_eq!(sound(&louder), sound(&controls));
        let mut fed = controls;
        fed.feedback = 0.5;
        assert_ne!(sound(&fed), sound(&controls), "FEEDBACK is the sound's");
    }

    /// A measurement under way when the sound is changed back to the curve's, or when a
    /// session is loaded, is let go: the curve stays the one wanted. (At the voices' lowest
    /// rate: a debug build's voice is slow.)
    #[test]
    fn a_measurement_for_a_sound_no_longer_wanted_is_let_go() {
        let c = Calibration::default();
        c.set_rate(crate::engine::MIN_VOICE_RATE);
        let first = Controls::default();
        let mut job = None;
        c.load(&Saved {
            sound: sound(&first),
            db: [-1.0; STEPS],
        });
        let mut other = first;
        other.panel.cutoff = 0.2;
        c.ask(&other);
        assert!(c.step(&mut job) && job.is_some());
        assert!(c.step(&mut job));
        c.ask(&first);
        while c.step(&mut job) {}
        assert_eq!(c.saved().db, [-1.0; STEPS]);
        c.ask(&other);
        assert!(c.step(&mut job) && c.step(&mut job));
        c.load(&Saved {
            sound: 99,
            db: [-2.0; STEPS],
        });
        while c.step(&mut job) {}
        assert_eq!(
            c.saved(),
            Saved {
                sound: 99,
                db: [-2.0; STEPS]
            }
        );
        // Measured to the end, the curve is the sound's.
        c.ask(&other);
        c.measure_now();
        let s = c.saved();
        assert_eq!(s.sound, sound(&other));
        assert!(s.db.iter().all(|d| d.is_finite() && *d < 0.0), "{s:?}");
    }
}
