//! A detached render through the plug-in's actual `process()` path: its parameter mapping,
//! arpeggiator, clock, LFOs, macros, output and AUTO GAIN. No device, window, helper or
//! real-time worker is opened. A snapshot describes settings, not oscillators already running.
#![allow(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Ca72;
use crate::agent::{ApiError, values};
use crate::params::Ca72Params;

pub const FORMAT: u32 = 1;
pub const BLOCK: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub format: u32,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub parameters: BTreeMap<String, f32>,
    /// Only the sound's persisted fields: assignments, macro names and AUTO GAIN's curve.
    pub fields: BTreeMap<String, String>,
    pub seed: u64,
    pub tempo: f64,
    /// A display preference which also defines a frequency modulation target's travel.
    pub hardware_frequency_dials: bool,
}

impl Snapshot {
    pub fn capture(p: &Ca72Params) -> Self {
        Self {
            format: FORMAT,
            engine_version: env!("CARGO_PKG_VERSION").into(),
            engine_source_sha256: env!("CA72_ENGINE_SOURCE_SHA256").into(),
            parameters: values::table(p)
                .iter()
                .map(|e| (e.id.into(), e.param.normalized()))
                .collect(),
            fields: p
                .serialize_fields()
                .into_iter()
                .filter(|(k, _)| matches!(k.as_str(), "lfo" | "drive_curve"))
                .collect(),
            seed: p.seed(),
            tempo: p.tempo(),
            hardware_frequency_dials: p.freq_law() == ca72::freq_dial::Law::Hardware,
        }
    }

    /// Replay a preset through the same targets and patch loading as the browser.
    pub fn preset(name: &str) -> Result<Self, ApiError> {
        let entry = crate::library::Library::shared()
            .find(name)
            .ok_or_else(|| ApiError::params(format!("no preset `{name}`")))?;
        let mut p = Ca72::default();
        let ctx = TestProcessContext::<Ca72>::new(p.params(), 48_000.0, ProcessMode::Offline);
        let map = p.params.param_map();
        for (id, value) in crate::presets::targets(&p.params, &entry) {
            let ptr = map
                .iter()
                .find(|(k, ..)| k == id)
                .ok_or_else(|| ApiError::refused(format!("missing parameter {id}")))?
                .1;
            if !ctx.automate(ptr, value) {
                return Err(ApiError::refused(format!("cannot restore {id}")));
            }
        }
        p.params.lfo_patch.load_values(&entry.sound.values);
        p.params.lfo_patch.load_names(&entry.sound.macro_names);
        if let Some(db) = entry.sound.curve(p.params.controls().potato) {
            p.params.drive_curve.load_preset(db);
        }
        p.params.noise_seed.store(72, Ordering::Relaxed);
        // No initialise(): it would load this computer's settings and start live workers.
        p.engine.stop_workers();
        Ok(Self::capture(&p.params))
    }

    pub fn validate(&self) -> Result<(), ApiError> {
        if self.format != FORMAT
            || self.engine_version != env!("CARGO_PKG_VERSION")
            || self.engine_source_sha256 != env!("CA72_ENGINE_SOURCE_SHA256")
        {
            return Err(ApiError::refused(
                "snapshot format or engine source differs; capture it again with the bridge built from the same sources as the app",
            ));
        }
        if !self.tempo.is_finite() || !(30.0..=300.0).contains(&self.tempo) {
            return Err(ApiError::params("snapshot tempo must be 30 to 300 BPM"));
        }
        let p = Ca72Params::default();
        let entries = values::table(&p);
        if self.parameters.len() != entries.len()
            || entries.iter().any(|e| !self.parameters.contains_key(e.id))
        {
            return Err(ApiError::params(
                "snapshot must contain exactly every parameter from ca72_snapshot",
            ));
        }
        if self
            .parameters
            .values()
            .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        {
            return Err(ApiError::params(
                "snapshot parameters must be finite normalized values from 0 to 1",
            ));
        }
        if self.fields.len() != 2
            || !self.fields.contains_key("lfo")
            || !self.fields.contains_key("drive_curve")
        {
            return Err(ApiError::params(
                "snapshot must contain its lfo and drive_curve fields only",
            ));
        }
        for value in self.fields.values() {
            serde_json::from_str::<serde_json::Value>(value)
                .map_err(|e| ApiError::params(format!("invalid snapshot field: {e}")))?;
        }
        let curve: crate::drive::Saved = serde_json::from_str(&self.fields["drive_curve"])
            .map_err(|e| ApiError::params(format!("invalid AUTO GAIN curve: {e}")))?;
        if curve
            .db
            .iter()
            .any(|v| !v.is_finite() || !(-120.0..=60.0).contains(v))
        {
            return Err(ApiError::params(
                "AUTO GAIN corrections must be finite and within -120 to 60 dB",
            ));
        }
        Ok(())
    }

    pub fn override_values(&mut self, input: &serde_json::Value) -> Result<(), ApiError> {
        let input = input.as_object().ok_or_else(|| {
            ApiError::params("values must be an object, in ca72_set's panel units")
        })?;
        let p = Ca72Params::default();
        let table = values::table(&p);
        let planned = input
            .iter()
            .map(|(key, v)| {
                let e = values::find(&table, key).ok_or_else(|| {
                    ApiError::params(format!(
                        "unknown parameter `{key}`; use ca72_describe's IDs"
                    ))
                })?;
                Ok((e.id.to_owned(), e.parse(v)?))
            })
            .collect::<Result<Vec<_>, ApiError>>()?;
        self.parameters.extend(planned);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub note: u8,
    pub start: f64,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub notes: Vec<Note>,
    pub seconds: f64,
    pub sample_rate: u32,
}

impl Request {
    pub fn validate(&self) -> Result<(), ApiError> {
        if !(8_000..=192_000).contains(&self.sample_rate)
            || !self.seconds.is_finite()
            || !(0.05..=60.0).contains(&self.seconds)
        {
            return Err(ApiError::params(
                "render 0.05 to 60 seconds at 8000 to 192000 Hz",
            ));
        }
        if self.notes.is_empty() || self.notes.len() > 4096 {
            return Err(ApiError::params("supply 1 to 4096 notes"));
        }
        if (self.seconds * self.sample_rate as f64).round() < 1024.0 {
            return Err(ApiError::params("render at least 1024 samples"));
        }
        for (i, n) in self.notes.iter().enumerate() {
            if n.note > 127
                || !n.start.is_finite()
                || !n.duration.is_finite()
                || n.start < 0.0
                || n.duration <= 0.0
                || n.start + n.duration > self.seconds
            {
                return Err(ApiError::params(format!(
                    "note {i} must be MIDI 0 to 127, have positive duration, and end within seconds"
                )));
            }
            if ((n.start + n.duration) * self.sample_rate as f64).round()
                <= (n.start * self.sample_rate as f64).round()
            {
                return Err(ApiError::params("each note must last at least one sample"));
            }
            if self.notes[..i].iter().any(|other| {
                other.note == n.note
                    && other.start < n.start + n.duration
                    && n.start < other.start + other.duration
            }) {
                return Err(ApiError::params(
                    "overlapping instances of the same MIDI note are ambiguous; use non-overlapping gates",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Render {
    pub channels: Vec<Vec<f32>>,
    pub latency_compensated_samples: usize,
}

pub fn render(snapshot: &Snapshot, request: &Request) -> Result<Render, ApiError> {
    nih_plug::util::with_flush_to_zero(|| render_inner(snapshot, request))
}

fn render_inner(snapshot: &Snapshot, request: &Request) -> Result<Render, ApiError> {
    snapshot.validate()?;
    request.validate()?;
    let mut p = Ca72::default();
    #[cfg(feature = "fx-host")]
    {
        p.fx = None;
    }
    p.params.deserialize_fields(&snapshot.fields);
    let restored = p.params.serialize_fields();
    for (key, value) in &snapshot.fields {
        let actual = restored
            .get(key)
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
        let expected = serde_json::from_str::<serde_json::Value>(value).ok();
        if actual != expected {
            return Err(ApiError::params(format!(
                "invalid or unsupported snapshot field `{key}`"
            )));
        }
    }
    let mut ctx = TestProcessContext::<Ca72>::new(
        p.params(),
        request.sample_rate as f32,
        ProcessMode::Offline,
    );
    let map = p.params.param_map();
    for (id, value) in &snapshot.parameters {
        let ptr = map
            .iter()
            .find(|(k, ..)| k == id)
            .ok_or_else(|| ApiError::params(format!("unknown parameter `{id}`")))?
            .1;
        if !ctx.automate(ptr, *value) {
            return Err(ApiError::refused(format!("could not restore {id}")));
        }
    }
    p.params.noise_seed.store(snapshot.seed, Ordering::Relaxed);
    p.params.set_tempo(snapshot.tempo);
    p.params.set_freq_law(if snapshot.hardware_frequency_dials {
        ca72::freq_dial::Law::Hardware
    } else {
        ca72::freq_dial::Law::Semitones
    });
    p.params.set_standalone(true);
    p.rate = request.sample_rate as f64;
    p.engine.set(&p.params.controls());
    p.engine.prepare(p.rate, snapshot.seed);
    p.engine.calibrate_with(p.params.drive_curve.clone());
    p.arp.seed(snapshot.seed);
    p.smooth.prepare(p.rate, BLOCK, &p.params);
    let latency = p.engine.latency() as usize;
    let frames = (request.seconds * p.rate).round() as usize;
    let mut events: Vec<(usize, u8, bool)> = request
        .notes
        .iter()
        .flat_map(|n| {
            [
                ((n.start * p.rate).round() as usize, n.note, true),
                (
                    ((n.start + n.duration) * p.rate).round() as usize,
                    n.note,
                    false,
                ),
            ]
        })
        .collect();
    // Release before attack at the same sample. Stable ordering keeps chords reproducible.
    events.sort_by_key(|(at, _, on)| (*at, *on));
    let mut next = 0;
    let mut channels: Vec<Vec<f32>> = (0..2)
        .map(|_| Vec::with_capacity(frames + latency))
        .collect();
    for at in (0..frames + latency).step_by(BLOCK) {
        let n = BLOCK.min(frames + latency - at);
        while let Some(&(sample, note, on)) = events.get(next) {
            if sample >= at + n {
                break;
            }
            let timing = sample.saturating_sub(at) as u32;
            ctx.push_event(if on {
                NoteEvent::NoteOn {
                    timing,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity: 1.0,
                }
            } else {
                NoteEvent::NoteOff {
                    timing,
                    voice_id: None,
                    channel: 0,
                    note,
                    velocity: 0.0,
                }
            });
            next += 1;
        }
        let (mut left, mut right) = (vec![0.0; n], vec![0.0; n]);
        let mut buffer = Buffer::default();
        // SAFETY: both owned slices have n frames and remain alive and exclusively borrowed
        // until process returns and buffer is dropped. No input/device buffer is shared.
        unsafe {
            buffer.set_slices(n, |out| {
                out.clear();
                out.push(&mut left);
                out.push(&mut right);
            });
        }
        p.process(
            &mut buffer,
            &mut AuxiliaryBuffers {
                inputs: &mut [],
                outputs: &mut [],
            },
            &mut ctx,
        );
        drop(buffer);
        if p.engine.render_faulted() {
            return Err(ApiError::refused(
                "DSP recovery or output sanitisation occurred; this render is not valid calibration audio",
            ));
        }
        if left.iter().chain(&right).any(|v| !v.is_finite()) {
            return Err(ApiError::refused("the renderer produced non-finite audio"));
        }
        channels[0].extend(left);
        channels[1].extend(right);
    }
    for c in &mut channels {
        c.drain(..latency);
    }
    Ok(Render {
        channels,
        latency_compensated_samples: latency,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noise_renders_repeat_and_a_snapshot_includes_the_new_controls() {
        let p = Ca72Params::default();
        let mut s = Snapshot::capture(&p);
        s.seed = 72;
        s.override_values(&serde_json::json!({"quality":"LO", "noise_on":true, "noise_volume":4, "lfo1_rate":3, "macro1":25})).expect("values");
        let req = Request {
            notes: vec![Note {
                note: 48,
                start: 0.0,
                duration: 0.08,
            }],
            seconds: 0.12,
            sample_rate: 48_000,
        };
        let a = render(&s, &req).expect("render");
        let b = render(&s, &req).expect("repeat");
        assert_eq!(a.channels, b.channels);
        assert_eq!(a.channels[0].len(), 5760);
        assert!(a.channels[0].iter().any(|v| v.abs() > 0.00001));
        assert!(s.parameters.contains_key("lfo1_rate") && s.parameters.contains_key("macro1"));
        s.seed += 1;
        assert_ne!(a.channels, render(&s, &req).expect("new seed").channels);
        assert!(!p.noise_on.value(), "detached render leaves source alone");
    }

    #[test]
    fn frequency_modulation_keeps_the_captured_dial_law() {
        let p = Ca72Params::default();
        p.set_freq_law(ca72::freq_dial::Law::Hardware);
        crate::agent::modulation::set(
            &p,
            &serde_json::json!({"source":"lfo1","target":"osc2_frequency","depth":35}),
        )
        .expect("assignment");
        let mut snapshot = Snapshot::capture(&p);
        snapshot
            .override_values(
                &serde_json::json!({"quality":"LO","osc1_on":false,"osc2_on":true,"lfo1_rate":8}),
            )
            .expect("values");
        let request = Request {
            notes: vec![Note {
                note: 48,
                start: 0.0,
                duration: 0.12,
            }],
            seconds: 0.16,
            sample_rate: 48000,
        };
        assert!(snapshot.hardware_frequency_dials);
        let hardware = render(&snapshot, &request).expect("hardware");
        snapshot.hardware_frequency_dials = false;
        let semitones = render(&snapshot, &request).expect("semitones");
        assert_ne!(
            hardware.channels, semitones.channels,
            "the law affects frequency modulation"
        );
        snapshot.hardware_frequency_dials = true;
        assert_eq!(
            hardware.channels,
            render(&snapshot, &request)
                .expect("repeat hardware")
                .channels
        );
    }

    #[test]
    fn invalid_curves_snapshots_and_zero_sample_gates_fail_closed() {
        let mut snapshot = Snapshot::capture(&Ca72Params::default());
        snapshot.fields.insert(
            "drive_curve".into(),
            serde_json::json!({"sound":0,"db":[1e30,1e30,1e30,1e30]}).to_string(),
        );
        assert!(snapshot.validate().is_err());
        let req = Request {
            notes: vec![Note {
                note: 48,
                start: 0.5 / 48000.0,
                duration: 0.6 / 48000.0,
            }],
            seconds: 0.1,
            sample_rate: 48000,
        };
        assert!(req.validate().is_err());
        let mut snapshot = Snapshot::capture(&Ca72Params::default());
        snapshot.engine_source_sha256 = "different engine".into();
        assert!(snapshot.validate().is_err());
    }
}
