//! The CA-72 as a VST3 and CLAP plug-in: the circuit model's voice in its real-time quality,
//! played by MIDI, its EXTERNAL INPUT the side chain; one voice, or with POLY one per note
//! (up to ten), placed across the stereo output by SPREAD.

pub mod character;
pub mod editor;
pub mod engine;
pub mod library;
pub mod params;
pub mod presets;

use std::sync::Arc;

use nih_plug::prelude::*;

use crate::editor::{Ca72Editor, Meters};
use crate::engine::{
    ALL_NOTES_OFF, ALL_SOUND_OFF, Engine, Event, MODULATION_WHEEL, RESET_CONTROLLERS,
};
use crate::params::Ca72Params;

pub struct Ca72 {
    params: Arc<Ca72Params>,
    engine: Engine,
    meters: Arc<Meters>,
}

impl std::fmt::Debug for Ca72 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ca72")
            .field("engine", &self.engine)
            .finish()
    }
}

impl Default for Ca72 {
    fn default() -> Self {
        Ca72 {
            params: Arc::new(Ca72Params::default()),
            engine: Engine::new(),
            meters: Arc::new(Meters::default()),
        }
    }
}

/// The side chain at sample `i`, its channels mixed to mono.
fn mono(channels: &[&mut [f32]], i: usize) -> f32 {
    if channels.is_empty() {
        return 0.0;
    }
    channels.iter().map(|c| c[i]).sum::<f32>() / channels.len() as f32
}

impl Ca72 {
    fn midi(&mut self, e: NoteEvent<()>) {
        let event = match e {
            NoteEvent::NoteOn { note, .. } => Event::Note {
                key: note,
                on: true,
            },
            NoteEvent::NoteOff { note, .. } | NoteEvent::Choke { note, .. } => Event::Note {
                key: note,
                on: false,
            },
            NoteEvent::MidiPitchBend { value, .. } => Event::PitchBend(value * 2.0 - 1.0),
            NoteEvent::MidiCC { cc, value, .. } => match cc {
                MODULATION_WHEEL => Event::Modulation(value),
                RESET_CONTROLLERS => Event::ResetControllers,
                ALL_SOUND_OFF | ALL_NOTES_OFF => Event::AllNotesOff,
                _ => return,
            },
            _ => return,
        };
        self.engine.event(event);
    }
}

/// EXTERNAL INPUT: a side chain of as many channels as the output (mixed to mono at the jack).
const SIDE_CHAIN_STEREO: &[std::num::NonZeroU32] = &[new_nonzero_u32(2)];
const SIDE_CHAIN_MONO: &[std::num::NonZeroU32] = &[new_nonzero_u32(1)];

const fn layout(channels: u32, name: &'static str) -> AudioIOLayout {
    AudioIOLayout {
        main_input_channels: None,
        main_output_channels: Some(new_nonzero_u32(channels)),
        aux_input_ports: if channels == 1 {
            SIDE_CHAIN_MONO
        } else {
            SIDE_CHAIN_STEREO
        },
        names: PortNames {
            layout: Some(name),
            main_output: Some("Output"),
            aux_inputs: &["External Input"],
            ..PortNames::const_default()
        },
        ..AudioIOLayout::const_default()
    }
}

impl Plugin for Ca72 {
    const NAME: &'static str = "CA-72";
    const VENDOR: &'static str = "Idle Foundry";
    const URL: &'static str = "https://github.com/Idlewile/ca-72";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[layout(2, "Stereo"), layout(1, "Mono")];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(Ca72Editor::new(
            self.params.clone(),
            self.meters.clone(),
        )))
    }

    fn initialize(
        &mut self,
        _layout: &AudioIOLayout,
        config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.engine.set(&self.params.controls());
        self.engine
            .prepare(f64::from(config.sample_rate), self.params.seed());
        // What this machine plays in real time with POLY at the host's largest block
        // (decisions.md, "POLY in real time"): shown beside VOICES.
        let block = config.max_buffer_size.max(1) as usize;
        let rate = f64::from(config.sample_rate);
        let fits = crate::engine::realtime_voices(rate, block, false);
        let fits_feedback = crate::engine::realtime_voices(rate, block, true);
        self.meters.measured(fits, fits_feedback, block as u32);
        true
    }

    fn reset(&mut self) {
        self.engine.release_all();
        self.engine.event(Event::ResetControllers);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set(&self.params.controls());
        let len = buffer.samples();
        let side = aux.inputs.first().map(|b| b.as_slice_immutable());
        let out = buffer.as_slice();
        let mut next = context.next_event();
        for i in 0..len {
            while let Some(e) = next {
                if e.timing() as usize > i {
                    break;
                }
                self.midi(e);
                next = context.next_event();
            }
            // Left and right (SPREAD's voices placed); on a mono output, the two together.
            let (l, r) = self.engine.tick_stereo(side.map_or(0.0, |s| mono(s, i)));
            match &mut out[..] {
                [left, right, ..] => {
                    left[i] = l;
                    right[i] = r;
                }
                [only] => only[i] = 0.5 * (l + r),
                [] => {}
            }
        }
        while let Some(e) = next {
            self.midi(e);
            next = context.next_event();
        }
        let lamp = self.engine.end_block(len);
        self.meters.publish(lamp, self.engine.midi_wheels());
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for Ca72 {
    const CLAP_ID: &'static str = "com.idlefoundry.ca-72";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A synthesizer modelled from its circuit, one voice or up to ten");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Mono,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for Ca72 {
    const VST3_CLASS_ID: [u8; 16] = *b"IdleFoundryCA72!";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

#[cfg(test)]
mod tests {
    use super::*;
    use ca72::modulation::PITCH_WHEEL_SEMITONES;

    #[test]
    fn the_hosts_midi_reaches_the_wheels_on_any_channel() {
        let mut p = Ca72::default();
        p.midi(NoteEvent::MidiPitchBend {
            timing: 0,
            channel: 3,
            value: 0.0,
        });
        p.midi(NoteEvent::MidiCC {
            timing: 0,
            channel: 9,
            cc: MODULATION_WHEEL,
            value: 1.0,
        });
        let (bend, modulation) = p.engine.wheels();
        assert!((bend + 2.0 / PITCH_WHEEL_SEMITONES).abs() < 1e-9, "{bend}");
        assert_eq!(modulation, 1.0);
        p.midi(NoteEvent::MidiCC {
            timing: 0,
            channel: 0,
            cc: RESET_CONTROLLERS,
            value: 0.0,
        });
        assert_eq!(p.engine.wheels(), (0.0, 0.0));
    }
}

#[allow(unsafe_code)]
mod export {
    use super::Ca72;
    use nih_plug::prelude::*;

    nih_export_clap!(Ca72);
    nih_export_vst3!(Ca72);
}
