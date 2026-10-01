//! The plug-in's audio engine, apart from any plug-in format: the circuit model's voice in its
//! real-time (Potato) quality, played by MIDI notes, pitch bend and the modulation wheel, with
//! the side chain at EXTERNAL INPUT. Once the voice is built nothing here allocates
//! (`tests/realtime.rs`).

use ca72::modulation::PITCH_WHEEL_SEMITONES;
use ca72::resample::{Decimator, Interpolator};
use ca72::voice::{Jacks, Panel, Quality, Voice, audio_taper};

/// What the parameters set: the panel (its wheels where the parameters leave them), MAIN
/// OUTPUT's VOLUME (0..1 of its travel) and switch, how far a MIDI keyboard's full pitch
/// bend moves the PITCH wheel, in semitones, and POWER (off: bypassed, silent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Controls {
    pub panel: Panel,
    pub volume: f64,
    pub main_output: bool,
    pub bend_range: f64,
    pub power: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Controls {
            panel: Panel::default(),
            volume: 1.0,
            main_output: true,
            bend_range: 2.0,
            power: true,
        }
    }
}

/// MIDI input, as the engine takes it (any channel).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Note {
        key: u8,
        on: bool,
    },
    /// The pitch bend, -1..1, 0 at rest.
    PitchBend(f32),
    /// The modulation wheel (control change 1), 0..1.
    Modulation(f32),
    /// Reset All Controllers (control change 121): the bend and modulation wheel at rest.
    ResetControllers,
    /// All Notes Off or All Sound Off (control changes 123 and 120): every key released.
    AllNotesOff,
}

/// The control change of a keyboard's modulation wheel.
pub const MODULATION_WHEEL: u8 = 1;
pub const ALL_SOUND_OFF: u8 = 120;
pub const RESET_CONTROLLERS: u8 = 121;
pub const ALL_NOTES_OFF: u8 = 123;

/// How fast the reported OVERLOAD lamp falls from a peak, s: the lamp's own thermal lag is
/// not modelled, and a display polling now and then should still see a flash.
const LAMP_FALL: f32 = 0.05;

/// The lowest rate the voice runs at, Hz: below it, at 2, 4 or 8 times the host's. The top
/// key's fundamental on 2' (4.2 kHz) stays under its Nyquist frequency, and the model's
/// factory calibration fails at much lower rates (it gives NaN at 1.2 kHz).
pub const MIN_VOICE_RATE: f64 = 8_000.0;

/// How long POWER takes to fade the output out or in, s (switched at once, it clicks).
const POWER_FADE: f64 = 0.01;

/// The voice at a multiple of the host's rate: the side chain interpolated up, the output
/// decimated down.
#[derive(Debug)]
struct Oversampled {
    factor: usize,
    up: Interpolator,
    down: Decimator,
}

/// The voice, the MIDI input's state and the controls it plays with.
#[derive(Debug)]
pub struct Engine {
    voice: Option<Box<Voice>>,
    over: Option<Oversampled>,
    rate: f64,
    seed: u64,
    controls: Controls,
    gain: f64,
    /// Keys held (MIDI numbers), played on by a voice built for a new rate.
    held: [bool; 128],
    bend: f32,
    modulation: f32,
    lamp: f32,
    /// The output's gain under POWER (0 off, 1 on), and its step a sample while it fades.
    fade: f64,
    fade_step: f64,
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

impl Engine {
    /// An engine without a voice (silent until [`Engine::prepare`]).
    pub fn new() -> Engine {
        let controls = Controls::default();
        Engine {
            voice: None,
            over: None,
            rate: 0.0,
            seed: 0,
            gain: output_gain(controls.volume, controls.main_output),
            controls,
            held: [false; 128],
            bend: 0.0,
            modulation: 0.0,
            lamp: 0.0,
            fade: 1.0,
            fade_step: 1.0,
        }
    }

    /// A voice for `rate` Hz, the noise generator seeded by `seed`, playing the keys held.
    /// Not on the audio thread: a voice at a rate not yet built takes up to a second (the
    /// oscillators' factory tuning, the tables); later ones are copies.
    pub fn prepare(&mut self, rate: f64, seed: u64) {
        self.seed = seed;
        if rate == self.rate
            && let Some(v) = &mut self.voice
        {
            v.set_seed(seed);
            return;
        }
        self.rate = rate;
        self.fade_step = (1.0 / (POWER_FADE * rate)).min(1.0);
        let factor = [1, 2, 4, 8]
            .into_iter()
            .find(|f| rate * *f as f64 >= MIN_VOICE_RATE)
            .unwrap_or(8);
        self.over = (factor > 1).then(|| Oversampled {
            factor,
            up: Interpolator::new(factor),
            down: Decimator::new(factor),
        });
        let mut v = Box::new(Voice::prototype(rate * factor as f64));
        v.set_seed(seed);
        v.panel = self.panel();
        for (key, _) in self.held.iter().enumerate().filter(|(_, h)| **h) {
            v.note(key as i32, true);
        }
        self.voice = Some(v);
    }

    /// The controls for the samples that follow.
    pub fn set(&mut self, c: &Controls) {
        if *c != self.controls {
            self.controls = *c;
            self.gain = output_gain(c.volume, c.main_output);
            self.apply();
        }
    }

    pub fn event(&mut self, e: Event) {
        match e {
            Event::Note { key, on } => {
                let Some(h) = self.held.get_mut(usize::from(key)) else {
                    return;
                };
                *h = on;
                if let Some(v) = &mut self.voice {
                    v.note(i32::from(key), on);
                }
            }
            Event::PitchBend(b) => {
                self.bend = b.clamp(-1.0, 1.0);
                self.apply();
            }
            Event::Modulation(m) => {
                self.modulation = m.clamp(0.0, 1.0);
                self.apply();
            }
            Event::ResetControllers => {
                (self.bend, self.modulation) = (0.0, 0.0);
                self.apply();
            }
            Event::AllNotesOff => self.release_all(),
        }
    }

    /// Every held key released (the contours then run out as the DECAY switch has them).
    pub fn release_all(&mut self) {
        for key in 0..self.held.len() {
            if std::mem::take(&mut self.held[key])
                && let Some(v) = &mut self.voice
            {
                v.note(key as i32, false);
            }
        }
    }

    /// One output sample with `ext` at EXTERNAL INPUT (1.0 the same volts as the output's
    /// 1.0); silence before [`Engine::prepare`]. With POWER off, once faded out, the voice
    /// rests (the MIDI input still reaches it).
    pub fn tick(&mut self, ext: f32) -> f32 {
        let on = self.controls.power;
        if !on && self.fade == 0.0 {
            return 0.0;
        }
        let Some(v) = &mut self.voice else {
            return 0.0;
        };
        let jacks = |ext| Jacks {
            ext,
            ..Jacks::default()
        };
        let y = match &mut self.over {
            None => v.tick_jacks(&jacks(f64::from(ext))),
            Some(o) => {
                let mut ins = [0.0; 8];
                o.up.push(f64::from(ext), &mut ins[..o.factor]);
                let mut y = 0.0;
                for x in &ins[..o.factor] {
                    if let Some(d) = o.down.push(v.tick_jacks(&jacks(*x))) {
                        y = d;
                    }
                }
                y
            }
        };
        self.fade = if on {
            (self.fade + self.fade_step).min(1.0)
        } else {
            (self.fade - self.fade_step).max(0.0)
        };
        (y * self.gain * self.fade) as f32
    }

    /// After a block of `len` samples: the OVERLOAD lamp (0 off, 1 fully lit), its peak held
    /// and falling with a time constant of [`LAMP_FALL`].
    pub fn end_block(&mut self, len: usize) -> f32 {
        let peak = self.voice.as_mut().map_or(0.0, |v| v.take_overload_peak());
        let fall = (-(len as f32) / (self.rate as f32 * LAMP_FALL)).exp();
        let fall = if fall.is_finite() { fall } else { 0.0 };
        self.lamp = (self.lamp * fall).max(peak.clamp(0.0, 1.0) as f32);
        self.lamp
    }

    pub fn lamp(&self) -> f32 {
        self.lamp
    }

    /// The wheels where the parameters and the MIDI input leave them: PITCH -1..1 and
    /// MODULATION 0..1.
    pub fn wheels(&self) -> (f64, f64) {
        let p = self.panel();
        (p.pitch_wheel, p.mod_wheel)
    }

    /// Where a MIDI keyboard's wheels move the panel's, as the panel draws them: its pitch
    /// bend's share of the PITCH wheel's travel (0..1, 0.5 at rest) and its modulation wheel
    /// (0..1).
    pub fn midi_wheels(&self) -> (f32, f32) {
        let share = (self.controls.bend_range / PITCH_WHEEL_SEMITONES).clamp(0.0, 1.0);
        let share = if share.is_finite() { share as f32 } else { 0.0 };
        (0.5 + self.bend * share / 2.0, self.modulation)
    }

    /// The panel the voice plays: the controls', in Potato, the MIDI input's bend moving
    /// the PITCH wheel by its share of the wheel's travel and its modulation wheel holding
    /// MODULATION up to its own position.
    fn panel(&self) -> Panel {
        let c = &self.controls;
        let share = (c.bend_range / PITCH_WHEEL_SEMITONES).clamp(0.0, 1.0);
        let share = if share.is_finite() { share } else { 0.0 };
        let mut p = c.panel;
        p.quality = Quality::Potato;
        p.pitch_wheel = (p.pitch_wheel + f64::from(self.bend) * share).clamp(-1.0, 1.0);
        p.mod_wheel = p.mod_wheel.max(f64::from(self.modulation));
        p
    }

    fn apply(&mut self) {
        let p = self.panel();
        if let Some(v) = &mut self.voice
            && v.panel != p
        {
            v.panel = p;
        }
    }
}

/// MAIN OUTPUT VOLUME: R20 (5K, audio taper) from the output into the jack's 10K load, and
/// its switch; `volume` 0..1 of the knob's travel.
pub fn output_gain(volume: f64, on: bool) -> f64 {
    if !on {
        return 0.0;
    }
    let t = audio_taper(volume);
    let (top, bottom) = (5e3 * (1.0 - t), 5e3 * t);
    let load = bottom * 10e3 / (bottom + 10e3);
    if load <= 0.0 {
        0.0
    } else {
        load / (top + load)
    }
}
