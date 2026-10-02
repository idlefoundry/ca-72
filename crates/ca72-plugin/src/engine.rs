//! The plug-in's audio engine, apart from any plug-in format: the circuit model's voice in its
//! real-time (Potato) quality, played by MIDI notes, pitch bend and the modulation wheel, with
//! the side chain at EXTERNAL INPUT. Once the voices are built nothing here allocates
//! (`tests/realtime.rs`).
//!
//! POLY off, the one instrument takes every key through its keyboard circuit (lowest-note
//! priority, single triggering), as before. POLY on, one voice per note, up to VOICES (2 to
//! 10) of the ten built: each the whole instrument with its own keyboard circuit, which its
//! note reaches as one key (`PolyKey`; decisions.md, "POLY"). ANALOG makes each voice its
//! own parts, SPREAD puts the voices across the stereo field (`character.rs`); both 0 by
//! default, the circuit as drawn.

use ca72::modulation::PITCH_WHEEL_SEMITONES;
use ca72::resample::{Decimator, Interpolator};
use ca72::voice::{Jacks, Panel, Quality, RETRIGGER_GAP, Voice, audio_taper};

use crate::character::Character;

/// How far ANALOG goes at 100 %, in the character's figures (`character.rs`, DAW-v3's): twice
/// them, as DAW-v3's Minimoog (the owner, 2026-10-02: at once they were "extremely subtle").
pub const ANALOG_DEPTH: f64 = 2.0;

/// The voices POLY may play: the fewest, by default, the most (all of them built).
pub const POLY_VOICES: (usize, usize, usize) = (2, 4, 10);

/// ANALOG's tolerances on the panel: EMPHASIS, the filter contour's ATTACK and DECAY, the
/// loudness contour's, GLIDE.
const ANALOG_KNOBS: usize = 6;

/// What the parameters set: the panel (its wheels where the parameters leave them), MAIN
/// OUTPUT's VOLUME (0..1 of its travel) and switch, how far a MIDI keyboard's full pitch
/// bend moves the PITCH wheel, in semitones, POWER (off: bypassed, silent), and POLY,
/// VOICES, ANALOG and SPREAD (0..1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Controls {
    pub panel: Panel,
    pub volume: f64,
    pub main_output: bool,
    pub bend_range: f64,
    pub power: bool,
    pub poly: bool,
    pub voices: usize,
    pub analog: f64,
    pub spread: f64,
}

impl Default for Controls {
    fn default() -> Self {
        Controls {
            panel: Panel::default(),
            volume: 1.0,
            main_output: true,
            bend_range: 2.0,
            power: true,
            poly: false,
            voices: POLY_VOICES.1,
            analog: 0.0,
            spread: 0.0,
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

/// A POLY voice falls silent, and is free for another note, once its key is up and its
/// output has stayed under this for a whole block.
const SILENT: f64 = 1e-6;

/// The voices at a multiple of the host's rate: the side chain interpolated up, the left and
/// right outputs decimated down.
#[derive(Debug)]
struct Oversampled {
    factor: usize,
    up: Interpolator,
    down: [Decimator; 2],
}

/// One POLY voice's key (decisions.md, "POLY"): its keyboard circuit gets its note as a key
/// down while the voice's gate is high. A new note on the voice while its gate was high
/// (the gate dropped for a sample between them) lifts the key and presses the new one once
/// the trigger contact has been open [`RETRIGGER_GAP`]: the circuit re-arms its trigger only
/// then, so the contours retrigger; the new note's pitch and attack come that much later.
#[derive(Debug, Clone, Copy)]
struct PolyKey {
    down: Option<i32>,
    /// Samples the trigger contact has been open, and how many it must be.
    open: u32,
    gap: u32,
}

impl PolyKey {
    fn new(rate: f64) -> PolyKey {
        let gap = (RETRIGGER_GAP * rate).ceil() as u32;
        PolyKey {
            down: None,
            open: gap,
            gap,
        }
    }

    /// The voice's gate and key for the next sample, played on `voice`'s keys.
    fn step(&mut self, gate: bool, key: i32, voice: &mut Voice) {
        if gate {
            match self.down {
                None if self.open >= self.gap => {
                    voice.note(key, true);
                    self.down = Some(key);
                }
                None => {}
                Some(old) if old != key => {
                    voice.note(key, true);
                    voice.note(old, false);
                    self.down = Some(key);
                }
                Some(_) => {}
            }
        } else if let Some(old) = self.down.take() {
            voice.note(old, false);
            self.open = 0;
        }
        if self.down.is_none() {
            self.open = self.open.saturating_add(1);
        }
    }

    /// Its key lifted at once (POLY switched, the voices rebuilt).
    fn lift(&mut self, voice: &mut Voice) {
        if let Some(old) = self.down.take() {
            voice.note(old, false);
        }
    }
}

/// A voice and its place among POLY's: its note, gate and age, whether it still sounds.
#[derive(Debug)]
struct Slot {
    voice: Box<Voice>,
    character: Character,
    key: PolyKey,
    note: u8,
    gate: bool,
    /// The gate the voice saw at its last sample, and a one-sample drop pending (a new note
    /// on a gate that was high).
    shown: bool,
    drop: bool,
    age: u64,
    /// Whether it is played (from its note until it falls silent), and its output's peak
    /// over the block so far.
    active: bool,
    peak: f64,
}

/// The voices, the MIDI input's state and the controls they play with.
#[derive(Debug)]
pub struct Engine {
    slots: Vec<Slot>,
    over: Option<Oversampled>,
    rate: f64,
    seed: u64,
    controls: Controls,
    gain: f64,
    /// Keys held (MIDI numbers), played on by voices built for a new rate (POLY off).
    held: [bool; 128],
    /// POLY's notes taken so far (each voice's age).
    counter: u64,
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
    /// An engine without voices (silent until [`Engine::prepare`]).
    pub fn new() -> Engine {
        let controls = Controls::default();
        Engine {
            slots: Vec::new(),
            over: None,
            rate: 0.0,
            seed: 0,
            gain: output_gain(controls.volume, controls.main_output),
            controls,
            held: [false; 128],
            counter: 0,
            bend: 0.0,
            modulation: 0.0,
            lamp: 0.0,
            fade: 1.0,
            fade_step: 1.0,
        }
    }

    /// The voices for `rate` Hz, their noise seeded from `seed` (the first voice's is `seed`
    /// itself, as with one voice), playing the keys held. Not on the audio thread: voices
    /// at a rate not yet built take up to a second (the oscillators' factory tuning, the
    /// tables); later ones are copies.
    pub fn prepare(&mut self, rate: f64, seed: u64) {
        self.seed = seed;
        if rate == self.rate && !self.slots.is_empty() {
            for (k, s) in self.slots.iter_mut().enumerate() {
                s.voice.set_seed(voice_seed(seed, k));
            }
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
            down: [Decimator::new(factor), Decimator::new(factor)],
        });
        let voice_rate = rate * factor as f64;
        self.slots = (0..POLY_VOICES.2)
            .map(|k| {
                // (Each its own copy from the prototype, its key lists' room reserved: a key
                // press must not allocate on the audio thread.)
                let mut voice = Box::new(Voice::prototype(voice_rate));
                voice.set_seed(voice_seed(seed, k));
                Slot {
                    voice,
                    character: Character::new(voice_seed(seed, k), k, 3, ANALOG_KNOBS),
                    key: PolyKey::new(voice_rate),
                    note: 0,
                    gate: false,
                    shown: false,
                    drop: false,
                    age: 0,
                    active: false,
                    peak: 0.0,
                }
            })
            .collect();
        self.apply();
        if !self.controls.poly {
            let v = &mut self.slots[0].voice;
            for (key, _) in self.held.iter().enumerate().filter(|(_, h)| **h) {
                v.note(key as i32, true);
            }
        }
    }

    /// The controls for the samples that follow. POLY switched lifts every key first (the
    /// notes then sounding stop; the next notes play in the new way).
    pub fn set(&mut self, c: &Controls) {
        if *c != self.controls {
            if c.poly != self.controls.poly {
                self.lift_all();
            }
            if c.voices < self.controls.voices {
                // The voices past the new count let go (they run on until silent).
                for s in self.slots.iter_mut().skip(c.voices.max(1)) {
                    s.gate = false;
                }
            }
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
                if self.slots.is_empty() {
                    return;
                }
                if !self.controls.poly {
                    self.slots[0].voice.note(i32::from(key), on);
                } else if on {
                    self.poly_on(key);
                } else {
                    for s in &mut self.slots {
                        if s.gate && s.note == key {
                            s.gate = false;
                        }
                    }
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

    /// A POLY note: a free voice by its number, else the oldest let go, else the oldest held
    /// (its key lifted and pressed again: `PolyKey`).
    fn poly_on(&mut self, key: u8) {
        let n = self.voices();
        let pick = self.slots[..n]
            .iter()
            .position(|s| !s.active)
            .or_else(|| {
                (0..n)
                    .filter(|&k| !self.slots[k].gate)
                    .min_by_key(|&k| self.slots[k].age)
            })
            .or_else(|| (0..n).min_by_key(|&k| self.slots[k].age))
            .unwrap_or(0);
        self.counter += 1;
        let s = &mut self.slots[pick];
        s.drop = s.active && s.shown;
        (s.note, s.gate, s.age, s.active, s.peak) = (key, true, self.counter, true, 0.0);
    }

    /// The voices POLY plays.
    fn voices(&self) -> usize {
        self.controls
            .voices
            .clamp(POLY_VOICES.0, POLY_VOICES.2)
            .min(self.slots.len())
    }

    /// Every held key released (the contours then run out as the DECAY switch has them).
    pub fn release_all(&mut self) {
        for key in 0..self.held.len() {
            if std::mem::take(&mut self.held[key])
                && !self.controls.poly
                && let Some(s) = self.slots.first_mut()
            {
                s.voice.note(key as i32, false);
            }
        }
        for s in &mut self.slots {
            s.gate = false;
        }
    }

    /// Every voice's key lifted at once and the held keys forgotten (POLY switched).
    fn lift_all(&mut self) {
        let poly = self.controls.poly;
        for (k, s) in self.slots.iter_mut().enumerate() {
            s.key.lift(&mut s.voice);
            if k == 0 && !poly {
                for (key, h) in self.held.iter().enumerate() {
                    if *h {
                        s.voice.note(key as i32, false);
                    }
                }
            }
            s.gate = false;
        }
        self.held = [false; 128];
    }

    /// One output sample (the left and right channels together, as on a mono output) with
    /// `ext` at EXTERNAL INPUT (1.0 the same volts as the output's 1.0).
    pub fn tick(&mut self, ext: f32) -> f32 {
        let (l, r) = self.tick_stereo(ext);
        0.5 * (l + r)
    }

    /// One output sample, left and right, with `ext` at EXTERNAL INPUT; silence before
    /// [`Engine::prepare`]. With POWER off, once faded out, the voices rest (the MIDI input
    /// still reaches them).
    pub fn tick_stereo(&mut self, ext: f32) -> (f32, f32) {
        let on = self.controls.power;
        if (!on && self.fade == 0.0) || self.slots.is_empty() {
            return (0.0, 0.0);
        }
        let rate = self.voice_rate();
        let (l, r) = match self.over.take() {
            None => self.sample(f64::from(ext), rate),
            Some(mut o) => {
                let mut ins = [0.0; 8];
                o.up.push(f64::from(ext), &mut ins[..o.factor]);
                let (mut l, mut r) = (0.0, 0.0);
                for x in &ins[..o.factor] {
                    let (a, b) = self.sample(*x, rate);
                    if let Some(d) = o.down[0].push(a) {
                        l = d;
                    }
                    if let Some(d) = o.down[1].push(b) {
                        r = d;
                    }
                }
                self.over = Some(o);
                (l, r)
            }
        };
        self.fade = if on {
            (self.fade + self.fade_step).min(1.0)
        } else {
            (self.fade - self.fade_step).max(0.0)
        };
        // (As the one voice was: the gain, then the fade.)
        (
            (l * self.gain * self.fade) as f32,
            (r * self.gain * self.fade) as f32,
        )
    }

    /// One sample of the voices at their rate (`rate` Hz): left and right.
    fn sample(&mut self, ext: f64, rate: f64) -> (f64, f64) {
        let (analog, spread) = (self.controls.analog * ANALOG_DEPTH, self.controls.spread);
        let jacks = |s: &mut Slot| {
            let mut j = Jacks {
                ext,
                ..Jacks::default()
            };
            if analog > 0.0 {
                (j.detune, j.cutoff) = s.character.offsets(analog, rate);
            }
            j
        };
        if !self.controls.poly {
            let s = &mut self.slots[0];
            let j = jacks(s);
            let y = s.voice.tick_jacks(&j);
            let (gl, gr) = s.character.gains(spread);
            return (y * f64::from(gl), y * f64::from(gr));
        }
        let (mut l, mut r) = (0.0, 0.0);
        for s in self.slots.iter_mut().filter(|s| s.active) {
            let gate = if s.drop {
                s.drop = false;
                false
            } else {
                s.gate
            };
            s.shown = s.gate;
            s.key.step(gate, i32::from(s.note), &mut s.voice);
            let j = jacks(s);
            let y = s.voice.tick_jacks(&j);
            s.peak = s.peak.max(y.abs());
            let (gl, gr) = s.character.gains(spread);
            l += y * f64::from(gl);
            r += y * f64::from(gr);
        }
        (l, r)
    }

    fn voice_rate(&self) -> f64 {
        self.rate * self.over.as_ref().map_or(1, |o| o.factor) as f64
    }

    /// After a block of `len` samples: the OVERLOAD lamp (0 off, 1 fully lit), its peak held
    /// and falling with a time constant of [`LAMP_FALL`]; POLY's voices let go and fallen
    /// silent over the block freed.
    pub fn end_block(&mut self, len: usize) -> f32 {
        let mut peak = 0.0f64;
        for s in &mut self.slots {
            peak = peak.max(s.voice.take_overload_peak());
            if s.active && !s.gate && s.peak < SILENT {
                s.active = false;
            }
            s.peak = 0.0;
        }
        let fall = (-(len as f32) / (self.rate as f32 * LAMP_FALL)).exp();
        let fall = if fall.is_finite() { fall } else { 0.0 };
        self.lamp = (self.lamp * fall).max(peak.clamp(0.0, 1.0) as f32);
        self.lamp
    }

    pub fn lamp(&self) -> f32 {
        self.lamp
    }

    /// The voices sounding now (POLY on), or 1 (off).
    pub fn sounding(&self) -> usize {
        if self.controls.poly {
            self.slots.iter().filter(|s| s.active).count()
        } else {
            1
        }
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

    /// The panel the voices play: the controls', in Potato, the MIDI input's bend moving
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

    /// The panel to every voice (each with its tolerances under ANALOG).
    fn apply(&mut self) {
        let p = self.panel();
        let analog = self.controls.analog * ANALOG_DEPTH;
        for s in &mut self.slots {
            let pv = if analog > 0.0 {
                analog_panel(&s.character, p, analog)
            } else {
                p
            };
            if s.voice.panel != pv {
                s.voice.panel = pv;
            }
        }
    }
}

/// The share of a block's period the voices may take: the rest is the host's, its other
/// plug-ins' and the machine's own jitter (DAW-v3's figure, DMM).
pub const HEADROOM: f64 = 0.6;

/// How many POLY voices this machine plays in real time at `rate` Hz in blocks of `block`
/// frames, the plug-in's voices all on the host's thread: a voice's cost in a block where its
/// key changes (the median over a few changes, legato and gliding, oscillators 2 and 3 on,
/// oscillator 3 on the filter through the MODULATION wheel, ANALOG's offsets on), and how
/// many such fit in [`HEADROOM`] of the block's period. Measured on the calling thread (not
/// the audio thread's): a quarter of a second or so, once a process for each rate and block.
pub fn realtime_voices(rate: f64, block: usize) -> u32 {
    use std::sync::{Mutex, OnceLock};
    static MEASURED: OnceLock<Mutex<Vec<(u64, usize, u32)>>> = OnceLock::new();
    let cache = MEASURED.get_or_init(|| Mutex::new(Vec::new()));
    let key = (rate.to_bits(), block);
    if let Some(n) = cache
        .lock()
        .ok()
        .and_then(|c| c.iter().find(|e| (e.0, e.1) == key).map(|e| e.2))
    {
        return n;
    }
    let n = measure_voices(rate, block);
    if let Ok(mut c) = cache.lock() {
        c.push((key.0, key.1, n));
    }
    n
}

fn measure_voices(rate: f64, block: usize) -> u32 {
    let factor = [1, 2, 4, 8]
        .into_iter()
        .find(|f| rate * *f as f64 >= MIN_VOICE_RATE)
        .unwrap_or(8);
    let voice_rate = rate * factor as f64;
    let mut v = Voice::prototype(voice_rate);
    v.set_seed(1);
    let mut p = Panel::default();
    p.osc[1].on = true;
    p.osc[2].on = true;
    p.filter_mod = true;
    p.mod_wheel = 0.4;
    p.glide = 0.15;
    p.glide_on = true;
    p.emphasis = 0.5;
    p.quality = Quality::Potato;
    v.panel = p;
    let mut character = Character::new(1, 1, 3, ANALOG_KNOBS);
    let keys = [45, 57, 52, 64, 40, 69, 48, 60];
    let mut costs = Vec::with_capacity(keys.len());
    let mut sink = 0.0;
    let mut last = None;
    for (n, &key) in keys.iter().cycle().take(2 * keys.len()).enumerate() {
        v.note(key, true);
        if let Some(old) = last.replace(key) {
            v.note(old, false);
        }
        let mut dearest = 0.0f64;
        for b in 0..4 {
            let t = std::time::Instant::now();
            for _ in 0..block * factor {
                let (detune, cutoff) = character.offsets(0.5, voice_rate);
                sink += v.tick_jacks(&Jacks {
                    detune,
                    cutoff,
                    ..Jacks::default()
                });
            }
            if b < 2 {
                dearest = dearest.max(t.elapsed().as_secs_f64());
            }
        }
        // (The first pass warms the caches and the voice up.)
        if n >= keys.len() {
            costs.push(dearest);
        }
    }
    std::hint::black_box(sink);
    costs.sort_by(f64::total_cmp);
    let cost = costs.get(costs.len() / 2).copied().unwrap_or(0.0);
    let period = block as f64 / rate.max(1.0);
    if cost > 0.0 {
        ((HEADROOM * period / cost).floor() as u32).min(POLY_VOICES.2 as u32)
    } else {
        0
    }
}

/// Voice `k`'s noise seed from the instance's: the first voice's the instance's own.
fn voice_seed(seed: u64, k: usize) -> u64 {
    if k == 0 {
        seed
    } else {
        seed ^ ((k as u64 + 1) << 48)
    }
}

/// The panel with voice `c`'s tolerances, by `amount` (0..1) of ANALOG.
fn analog_panel(c: &Character, mut p: Panel, amount: f64) -> Panel {
    let k = |i: usize, x: f64| c.knob(i, x, amount);
    p.emphasis = k(0, p.emphasis);
    p.filter_contour.attack = k(1, p.filter_contour.attack);
    p.filter_contour.decay = k(2, p.filter_contour.decay);
    p.loudness_contour.attack = k(3, p.loudness_contour.attack);
    p.loudness_contour.decay = k(4, p.loudness_contour.decay);
    p.glide = k(5, p.glide);
    p
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
