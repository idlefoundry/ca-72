//! The CA-72 as a VST3 and CLAP plug-in: the circuit model's voice in its real-time quality,
//! played by MIDI, its EXTERNAL INPUT the side chain, mono on every output channel.

pub mod engine;
pub mod params;

use std::sync::Arc;

use nih_plug::prelude::*;

use crate::engine::{
    ALL_NOTES_OFF, ALL_SOUND_OFF, Engine, Event, MODULATION_WHEEL, RESET_CONTROLLERS,
};
use crate::params::Ca72Params;

pub struct Ca72 {
    params: Arc<Ca72Params>,
    engine: Engine,
}

impl std::fmt::Debug for Ca72 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ca72").field("engine", &self.engine).finish()
    }
}

impl Default for Ca72 {
    fn default() -> Self {
        Ca72 {
            params: Arc::new(Ca72Params::default()),
            engine: Engine::new(),
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
            NoteEvent::NoteOn { note, .. } => Event::Note { key: note, on: true },
            NoteEvent::NoteOff { note, .. } | NoteEvent::Choke { note, .. } => {
                Event::Note { key: note, on: false }
            }
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

/// EXTERNAL INPUT: a stereo side chain (mixed to mono at the jack).
const SIDE_CHAIN: &[std::num::NonZeroU32] = &[new_nonzero_u32(2)];

const fn layout(channels: u32, name: &'static str) -> AudioIOLayout {
    AudioIOLayout {
        main_input_channels: None,
        main_output_channels: Some(new_nonzero_u32(channels)),
        aux_input_ports: SIDE_CHAIN,
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

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] =
        &[layout(2, "Stereo"), layout(1, "Mono")];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
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
            let y = self.engine.tick(side.map_or(0.0, |s| mono(s, i)));
            for channel in out.iter_mut() {
                channel[i] = y;
            }
        }
        while let Some(e) = next {
            self.midi(e);
            next = context.next_event();
        }
        self.engine.end_block(len);
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for Ca72 {
    const CLAP_ID: &'static str = "com.idlefoundry.ca-72";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A monophonic synthesizer modelled from its circuit");
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
