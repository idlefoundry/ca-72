//! The production host exercises the same Plugin::process path with no devices.
#![cfg(feature = "standalone-production")]
#![allow(unsafe_code)]

#[cfg(not(feature = "assert_process_allocs"))]
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use ca72_plugin::Ca72;
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;
use nih_plug::wrapper::production::Processor;
use nih_plug::wrapper::state::{ParamValue, PluginState};
use plugin_kit_standalone::{AudioBlock, AudioProcessor, MidiEvent, ProcessConfig, SessionState};

thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static OPERATIONS: Cell<usize> = const { Cell::new(0) };
}
#[cfg(not(feature = "assert_process_allocs"))]
struct Counting;
// SAFETY: every allocation operation is forwarded unchanged to System.
#[cfg(not(feature = "assert_process_allocs"))]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = TRACK.try_with(|track| {
            if track.get() {
                OPERATIONS.with(|n| n.set(n.get() + 1));
            }
        });
        // SAFETY: forward the allocator caller's contract.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = TRACK.try_with(|track| {
            if track.get() {
                OPERATIONS.with(|n| n.set(n.get() + 1));
            }
        });
        // SAFETY: forward the allocator caller's contract.
        unsafe {
            System.dealloc(ptr, layout);
        }
    }
}
#[global_allocator]
#[cfg(not(feature = "assert_process_allocs"))]
static ALLOCATOR: Counting = Counting;

struct Initial;
impl InitContext<Ca72> for Initial {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Standalone
    }
    fn execute(&self, _task: ()) {}
    fn set_latency_samples(&self, _samples: u32) {}
    fn set_current_voice_capacity(&self, _capacity: u32) {}
}

#[test]
fn direct_plugin_and_adapter_are_sample_identical_without_callback_allocation() {
    let mut direct = Ca72::default();
    let wrapped = Ca72::default();
    let wrapped_params = wrapped.parameters().clone();
    direct.parameters().noise_seed.store(42, Ordering::Relaxed);
    wrapped.parameters().noise_seed.store(42, Ordering::Relaxed);
    let params: Arc<dyn Params> = direct.parameters().clone();
    let mut context = TestProcessContext::<Ca72>::new(params, 48_000.0, ProcessMode::Realtime);
    let (mut adapter, session) =
        Processor::new(wrapped, &["pitch_wheel", "mod_wheel"]).expect("adapter");
    context.automate(direct.parameters().ext_on.as_ptr(), 1.0);
    context.automate(direct.parameters().ext_volume.as_ptr(), 0.25);
    session.set_parameter(wrapped_params.ext_on.as_ptr(), 1.0);
    session.set_parameter(wrapped_params.ext_volume.as_ptr(), 0.25);
    let config = BufferConfig {
        sample_rate: 48_000.0,
        min_buffer_size: None,
        max_buffer_size: 256,
        process_mode: ProcessMode::Realtime,
    };
    assert!(direct.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Initial));
    direct.reset();
    adapter
        .prepare(ProcessConfig {
            sample_rate: 48_000.0,
            max_buffer_frames: 256,
        })
        .expect("prepared");

    let (mut left, mut right, mut app_left, mut app_right) =
        ([0.0; 256], [0.0; 256], [0.0; 256], [0.0; 256]);
    let mut input_left = std::array::from_fn::<_, 256, _>(|sample| sample as f32 * 0.001);
    let mut input_right =
        std::array::from_fn::<_, 256, _>(|sample| (sample as f32 * 0.02).sin() * 0.2);
    input_left[2] = f32::NAN;
    input_right[5] = f32::INFINITY;
    let messages = [
        [0x90, 60, 100],
        [0xe2, 0, 70],
        [0xb3, 1, 50],
        [0x90, 67, 90],
        [0x80, 60, 0],
        [0x90, 67, 0],
        [0xb0, 121, 0],
        [0xb0, 123, 0],
    ];
    let mut heard = false;
    for (iteration, message) in messages.into_iter().enumerate() {
        let frames = if iteration % 2 == 0 { 128 } else { 256 };
        context.push_event(NoteEvent::from_midi(0, &message).expect("MIDI"));
        let mut buffer = Buffer::default();
        let mut side = Buffer::default();
        // SAFETY: channels are distinct, 256 samples long, alive through process.
        unsafe {
            buffer.set_slices(frames, |s| {
                s.push(&mut left[..frames]);
                s.push(&mut right[..frames]);
            });
            side.set_slices(frames, |s| {
                s.push(&mut input_left[..frames]);
                s.push(&mut input_right[..frames]);
            });
        }
        let mut sides = [side];
        direct.process(
            &mut buffer,
            &mut AuxiliaryBuffers {
                inputs: &mut sides,
                outputs: &mut [],
            },
            &mut context,
        );
        let midi = [MidiEvent {
            source: 1,
            data: message,
            len: 3,
        }];
        OPERATIONS.with(|n| n.set(0));
        TRACK.with(|track| track.set(true));
        adapter.process(AudioBlock {
            left: &mut app_left[..frames],
            right: &mut app_right[..frames],
            input_left: &input_left[..frames],
            input_right: &input_right[..frames],
            midi: &midi,
            panic: false,
        });
        TRACK.with(|track| track.set(false));
        assert_eq!(
            OPERATIONS.with(Cell::get),
            0,
            "adapter callback allocated or freed"
        );
        assert_eq!(left, app_left, "left samples changed through adapter");
        assert_eq!(right, app_right, "right samples changed through adapter");
        heard |= app_left.iter().any(|sample| sample.abs() > 0.0001);
    }
    assert!(heard, "parity must compare audible output");
    direct.deactivate();
    adapter.deactivate();
}

#[test]
fn stopped_edits_and_session_restore_preserve_sound_and_reset_live_controls() {
    let plugin = Ca72::default();
    let params = plugin.parameters().clone();
    let (mut processor, session) =
        Processor::new(plugin, &["pitch_wheel", "mod_wheel"]).expect("adapter");
    session.set_parameter(params.cutoff.as_ptr(), 0.75);
    session.set_parameter(params.pitch_wheel.as_ptr(), 1.0);
    session.set_parameter(params.mod_wheel.as_ptr(), 1.0);
    params.editor_width.store(1250, Ordering::Relaxed);
    let cc = ca72_plugin::learn::Cc { channel: 4, cc: 74 };
    let assignment = ca72_plugin::learn::LEARNABLE
        .iter()
        .position(|parameter| parameter.id == "cutoff")
        .expect("cutoff learnable");
    params
        .midi_map
        .assign(assignment, cc)
        .expect("MIDI assignment");
    *params.preset.write().expect("preset lock") = "Edited sound".into();
    assert_eq!(
        params.cutoff.unmodulated_normalized_value(),
        0.75,
        "GUI edit must work without callbacks"
    );
    let saved = session.save().expect("saved");
    let state: PluginState = serde_json::from_slice(&saved).expect("state");
    assert!(matches!(state.params["pitch_wheel"], ParamValue::F32(value) if value == 0.0));
    assert!(matches!(state.params["mod_wheel"], ParamValue::F32(value) if value == 0.0));
    session.set_parameter(params.cutoff.as_ptr(), 0.2);
    params.midi_map.remove(assignment);
    session.restore(&saved).expect("restored");
    assert_eq!(params.cutoff.unmodulated_normalized_value(), 0.75);
    assert_eq!(params.pitch_wheel.value(), 0.0);
    assert_eq!(params.mod_wheel.value(), 0.0);
    assert_eq!(params.editor_width.load(Ordering::Relaxed), 1250);
    assert_eq!(*params.preset.read().expect("preset"), "Edited sound");
    assert_eq!(params.midi_map.assignment(assignment), Some(cc));
    let before = session.save().expect("before malformed");
    assert!(session.restore(b"{broken").is_err());
    assert_eq!(session.save().expect("preserved"), before);
    let mut malformed: PluginState = serde_json::from_slice(&before).expect("state");
    malformed
        .params
        .insert("osc1_range".into(), ParamValue::String("unknown".into()));
    assert!(
        session
            .restore(&serde_json::to_vec(&malformed).expect("malformed JSON"))
            .is_err()
    );
    assert_eq!(
        session.save().expect("preserved after invalid enum"),
        before
    );
    processor
        .prepare(ProcessConfig {
            sample_rate: 48_000.0,
            max_buffer_frames: 128,
        })
        .expect("prepared after stopped edit");
    processor.deactivate();
    assert_eq!(params.cutoff.unmodulated_normalized_value(), 0.75);
    processor
        .prepare(ProcessConfig {
            sample_rate: 44_100.0,
            max_buffer_frames: 256,
        })
        .expect("sample rate changed off callback");
    session.set_parameter(params.cutoff.as_ptr(), 0.6);
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    processor.process(AudioBlock {
        left: &mut left,
        right: &mut right,
        input_left: &[],
        input_right: &[],
        midi: &[],
        panic: false,
    });
    assert_eq!(params.cutoff.unmodulated_normalized_value(), 0.6);
    processor.deactivate();
}

#[test]
fn idle_adapter_overwrites_every_sample_and_preserves_circuit_settling() {
    let plugin = Ca72::default();
    let params = plugin.parameters().clone();
    let (mut adapter, _) = Processor::new(plugin, &[]).expect("adapter");
    adapter
        .prepare(ProcessConfig {
            sample_rate: 48_000.0,
            max_buffer_frames: 256,
        })
        .expect("prepared");
    let (mut left, mut right) = ([1.0; 128], [1.0; 128]);
    adapter.process(AudioBlock {
        left: &mut left,
        right: &mut right,
        input_left: &[],
        input_right: &[],
        midi: &[],
        panic: false,
    });
    assert!(
        left.into_iter()
            .chain(right)
            .all(|sample| sample.is_finite() && sample != 1.0),
        "the circuit's startup output overwrites every supplied sample"
    );
    // R68's existing engine tests allow 30 seconds for slow circuit state to
    // settle. The wrapper must retain those samples, then inherit exact sleep.
    let mut settled = false;
    for _ in 0..(30 * 48_000 / 128) {
        adapter.process(AudioBlock {
            left: &mut left,
            right: &mut right,
            input_left: &[],
            input_right: &[],
            midi: &[],
            panic: false,
        });
        if left.into_iter().chain(right).all(|sample| sample == 0.0) {
            settled = true;
            break;
        }
    }
    assert!(
        settled,
        "idle must inherit the engine's established sleep bound"
    );
    let events = [
        MidiEvent {
            source: 1,
            data: [0xe0, 0, 0],
            len: 3,
        },
        MidiEvent {
            source: 1,
            data: [0xb0, 1, 127],
            len: 3,
        },
    ];
    adapter.process(AudioBlock {
        left: &mut left,
        right: &mut right,
        input_left: &[],
        input_right: &[],
        midi: &events,
        panic: false,
    });
    adapter.process(AudioBlock {
        left: &mut left,
        right: &mut right,
        input_left: &[],
        input_right: &[],
        midi: &[],
        panic: true,
    });
    assert_eq!(params.pitch_wheel.value(), 0.0);
    assert_eq!(params.mod_wheel.value(), 0.0);
    adapter.deactivate();
}
