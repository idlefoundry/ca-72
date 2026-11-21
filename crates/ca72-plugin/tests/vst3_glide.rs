//! Headless GLIDE automation through the CA-72's VST3 processor.

#![allow(unsafe_code, clippy::unwrap_used)]

mod vst3_support;

use std::ptr::null_mut;

use ca72_plugin::Ca72;
use nih_plug::wrapper::vst3::{Wrapper, vst3_sys};
use vst3_support::{com, id, parameter_changes, parameter_queue, pointer};
use vst3_sys as vst3_com;
use vst3_sys::base::{IPluginBase, IUnknown, kResultFalse, kResultOk, tresult};
use vst3_sys::vst::{
    AudioBusBuffers, Event, EventData, EventTypes, IAudioProcessor, IComponent, IEditController,
    IEventList, IParameterChanges, NoteOffEvent, NoteOnEvent, ProcessData, ProcessModes,
    ProcessSetup, SymbolicSampleSizes,
};
use vst3_sys::{VST3, VstPtr};

#[VST3(implements(IEventList))]
struct Notes {
    event: Option<Event>,
}

impl IEventList for Notes {
    unsafe fn get_event_count(&self) -> i32 {
        i32::from(self.event.is_some())
    }

    unsafe fn get_event(&self, index: i32, event: *mut Event) -> tresult {
        match (index, self.event) {
            (0, Some(note)) => {
                // SAFETY: the wrapper provides a pointer to its local event for this call.
                unsafe { event.write(note) };
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn add_event(&self, _event: *mut Event) -> tresult {
        kResultFalse
    }
}

/// Exercise CA-72's actual VST3 parameter queue across block splits and rate changes (#58).
#[test]
fn ca72_vst3_glide_automation_during_overlapping_notes() {
    const MAX: usize = 31;
    let w = Box::leak(Wrapper::<Ca72>::new());
    let offline = ProcessModes::kOffline as i32;
    let float32 = SymbolicSampleSizes::kSample32 as i32;
    let mut left = [0.0f32; MAX];
    let mut right = [0.0f32; MAX];

    // SAFETY: a host's lifecycle calls in order. The buffers and COM objects remain alive for
    // every process call, and the wrapper's original reference is released after termination.
    unsafe {
        assert_eq!(w.initialize(null_mut()), kResultOk);
        for rate in [44_100.0, 48_000.0, 96_000.0] {
            let setup = ProcessSetup {
                process_mode: offline,
                symbolic_sample_size: float32,
                max_samples_per_block: MAX as i32,
                sample_rate: rate,
            };
            assert_eq!(w.setup_processing(&setup), kResultOk);
            assert_eq!(w.set_active(1), kResultOk);
            assert_eq!(w.set_processing(1), kResultOk);

            let mut peak = 0.0f32;
            for block in 0..320 {
                let frames = [1, 7, 17, MAX][block % 4];
                let glide: Vec<_> = (0..frames)
                    .map(|i| {
                        let value = match (block + i) % 8 {
                            0 => 0.0,
                            1 => 1.0,
                            2 => 1e-40,
                            _ => ((block + i) % 33) as f64 / 32.0,
                        };
                        (i as i32, value)
                    })
                    .collect();
                let last = glide.last().unwrap().1;
                let mut queues = vec![parameter_queue(id("glide"), glide)];
                if block == 0 {
                    for (param, value) in [("glide_on", 1.0), ("quality", 0.0), ("poly", 0.0)] {
                        queues.push(parameter_queue(id(param), vec![(0, value)]));
                    }
                }
                let changes: VstPtr<dyn IParameterChanges> = parameter_changes(queues);

                // Keep 60 held while 48 enters and leaves, so mono LOW priority glides both ways.
                let note = match block % 80 {
                    4 => Some((
                        EventTypes::kNoteOnEvent,
                        EventData {
                            note_on: NoteOnEvent {
                                pitch: 60,
                                velocity: 1.0,
                                note_id: 60,
                                ..Default::default()
                            },
                        },
                    )),
                    24 => Some((
                        EventTypes::kNoteOnEvent,
                        EventData {
                            note_on: NoteOnEvent {
                                pitch: 48,
                                velocity: 1.0,
                                note_id: 48,
                                ..Default::default()
                            },
                        },
                    )),
                    44 => Some((
                        EventTypes::kNoteOffEvent,
                        EventData {
                            note_off: NoteOffEvent {
                                pitch: 48,
                                note_id: 48,
                                ..Default::default()
                            },
                        },
                    )),
                    64 => Some((
                        EventTypes::kNoteOffEvent,
                        EventData {
                            note_off: NoteOffEvent {
                                pitch: 60,
                                note_id: 60,
                                ..Default::default()
                            },
                        },
                    )),
                    _ => None,
                };
                let events: VstPtr<dyn IEventList> =
                    com(Notes::allocate(note.map(|(kind, event)| Event {
                        bus_index: 0,
                        sample_offset: 0,
                        ppq_position: 0.0,
                        flags: 0,
                        type_: kind as u16,
                        event,
                    })));
                left.fill(f32::NAN);
                right.fill(f32::NAN);
                let mut channels = [left.as_mut_ptr(), right.as_mut_ptr()];
                let mut output = AudioBusBuffers {
                    num_channels: 2,
                    silence_flags: 0,
                    buffers: channels.as_mut_ptr().cast(),
                };
                let mut data: ProcessData = std::mem::zeroed();
                data.process_mode = offline;
                data.symbolic_sample_size = float32;
                data.num_samples = frames as i32;
                data.num_outputs = 1;
                data.outputs = &mut output;
                data.input_param_changes = pointer(changes.as_ptr().cast());
                data.input_events = pointer(events.as_ptr().cast());
                assert_eq!(w.process(&mut data), kResultOk, "{rate} Hz, block {block}");
                assert!(
                    (w.get_param_normalized(id("glide")) - last).abs() < 1e-6,
                    "GLIDE queue did not deliver its last point at {rate} Hz, block {block}"
                );
                for sample in left[..frames].iter().chain(&right[..frames]) {
                    assert!(sample.is_finite(), "{rate} Hz, block {block}");
                    peak = peak.max(sample.abs());
                }
            }

            assert!(peak > 1e-4, "note events produced no audio at {rate} Hz");
            assert_eq!(w.set_processing(0), kResultOk);
            assert_eq!(w.set_active(0), kResultOk);
        }
        assert_eq!(w.terminate(), kResultOk);
        w.release();
    }
}
