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

/// Fresh host processes read their own computer settings before restoring MIDI assignments.
#[test]
fn computer_wheel_source_survives_restart_and_precedes_project_restore() {
    use ca72_plugin::{
        learn::{Cc, index},
        mod_wheel::Source,
        settings::{Change, Settings},
    };
    const CHILD: &str = "CA72_WHEEL_SOURCE_CHILD";
    if let Ok(code) = std::env::var(CHILD) {
        let source = Source::of_code(code.parse().expect("source code"));
        let plugin = Ca72::default();
        let params = plugin.parameters().clone();
        assert_eq!(
            params.midi_map.wheel_source(),
            source,
            "before project restoration"
        );
        let control = index("cutoff").expect("cutoff");
        let cc = Cc {
            channel: 3,
            cc: if source == Source::DEFAULT { 11 } else { 1 },
        };
        params
            .midi_map
            .assign(control, cc)
            .expect("freed CC assigned");
        let (_original_processor, session) =
            Processor::new(plugin, &["pitch_wheel", "mod_wheel"]).expect("session");
        let saved = session.save().expect("project");
        let state: PluginState = serde_json::from_slice(&saved).expect("project state");
        assert!(!state.fields.contains_key("mod_wheel_source"));
        assert!(!state.params.contains_key("mod_wheel_source"));
        let restarted = Ca72::default();
        let restored = restarted.parameters().clone();
        let (mut processor, session) =
            Processor::new(restarted, &["pitch_wheel", "mod_wheel"]).expect("restarted host");
        session
            .restore(&saved)
            .expect("project restored before activation");
        assert_eq!(restored.midi_map.assignment(control), Some(cc));
        processor
            .prepare(ProcessConfig {
                sample_rate: 48_000.0,
                max_buffer_frames: 256,
            })
            .expect("activation");
        assert_eq!(
            restored.midi_map.wheel_source(),
            source,
            "editor never opened"
        );
        assert_eq!(
            restored.midi_map.assignment(control),
            Some(cc),
            "freed CC survives restore"
        );
        processor.deactivate();
        return;
    }
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("worktree").join("target"));
    std::fs::create_dir_all(&target).expect("target folder");
    let dir = tempfile::tempdir_in(target).expect("isolated computer");
    for source in [
        Source::DEFAULT,
        Source::Cc(11),
        Source::Cc(23),
        Source::ChannelPressure,
        Source::PolyPressure,
    ] {
        Settings::keep(
            &dir.path().join("CA-72/settings.toml"),
            Change::ModWheelSource(source),
        )
        .expect("computer setting");
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "computer_wheel_source_survives_restart_and_precedes_project_restore",
                "--nocapture",
            ])
            .env(CHILD, source.code().to_string())
            .env("IDLE_FOUNDRY_SHARED", dir.path())
            .env("CA72_SEATS_DIR", dir.path().join("seats"))
            .output()
            .expect("fresh host process");
        assert!(
            output.status.success(),
            "{source:?}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn standalone_decodes_every_wheel_source_without_callback_allocation() {
    use ca72_plugin::mod_wheel::Source;
    for source in [
        Source::Cc(1),
        Source::Cc(2),
        Source::Cc(4),
        Source::Cc(11),
        Source::Cc(74),
        Source::Cc(23),
        Source::ChannelPressure,
        Source::PolyPressure,
    ] {
        let reference = Ca72::default();
        let changed = Ca72::default();
        let original = reference.parameters().clone();
        let params = changed.parameters().clone();
        for p in [&original, &params] {
            p.noise_seed.store(42, Ordering::Relaxed);
        }
        let (mut before, original_session) =
            Processor::new(reference, &["pitch_wheel", "mod_wheel"]).expect("reference");
        let (mut after, session) =
            Processor::new(changed, &["pitch_wheel", "mod_wheel"]).expect("source");
        original_session.set_parameter(original.osc_mod.as_ptr(), 1.0);
        session.set_parameter(params.osc_mod.as_ptr(), 1.0);
        for processor in [&mut before, &mut after] {
            processor
                .prepare(ProcessConfig {
                    sample_rate: 48_000.0,
                    max_buffer_frames: 256,
                })
                .expect("prepared");
        }
        original.midi_map.set_wheel_source(Source::DEFAULT);
        params.midi_map.set_wheel_source(source);
        let mut heard = false;
        for b in 0..12 {
            let value = (b * 11 % 128) as u8;
            let message = if b == 0 {
                [0x95, 60, 100]
            } else {
                [0xb5, 1, value]
            };
            let changed = if b == 0 {
                message
            } else {
                match source {
                    Source::Cc(cc) => [0xb5, cc, value],
                    Source::ChannelPressure => [0xd5, value, 0],
                    Source::PolyPressure => [0xa5, 60, value],
                }
            };
            let run = |processor: &mut Processor<Ca72>, message: [u8; 3]| {
                let (mut left, mut right) = ([0.0; 256], [0.0; 256]);
                let midi = [MidiEvent {
                    source: 1,
                    data: message,
                    len: if message[0] & 0xf0 == 0xd0 { 2 } else { 3 },
                }];
                OPERATIONS.with(|n| n.set(0));
                TRACK.with(|track| track.set(true));
                processor.process(AudioBlock {
                    left: &mut left,
                    right: &mut right,
                    input_left: &[],
                    input_right: &[],
                    midi: &midi,
                    panic: false,
                });
                TRACK.with(|track| track.set(false));
                assert_eq!(
                    OPERATIONS.with(Cell::get),
                    0,
                    "{source:?} allocated or freed"
                );
                (left, right)
            };
            let was = run(&mut before, message);
            let now = run(&mut after, changed);
            assert_eq!(now, was, "{source:?}, block {b}");
            heard |= now.0.iter().any(|sample| sample.abs() > 0.0001);
        }
        assert!(heard);
        before.deactivate();
        after.deactivate();
    }
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
    let mut malformed: PluginState = serde_json::from_slice(&before).expect("state");
    malformed.fields.insert("midi_map".into(), "[broken".into());
    assert!(
        session
            .restore(&serde_json::to_vec(&malformed).expect("nested malformed JSON"))
            .is_err()
    );
    assert_eq!(
        session
            .save()
            .expect("preserved after invalid persistent field"),
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

#[test]
fn standalone_validation_accepts_the_plugins_existing_state_migrations() {
    let plugin = Ca72::default();
    let params = plugin.parameters().clone();
    let (_adapter, session) = Processor::new(plugin, &[]).expect("adapter");
    let mut old: PluginState =
        serde_json::from_slice(&session.save().expect("current state")).expect("state");
    old.version = "0.1.5".into();
    old.params
        .insert("placement".into(), ParamValue::String("edges".into()));
    old.params.insert("emphasis".into(), ParamValue::F32(5.0));
    session
        .restore(&serde_json::to_vec(&old).expect("old state"))
        .expect("compatible migration");
    assert_eq!(params.placement.value(), ca72_plugin::params::Scatter::Even);
    let expected = ca72_plugin::compat::translated("emphasis", 5.0)
        .expect("documented emphasis translation") as f32;
    assert_eq!(
        params.emphasis.value(),
        expected,
        "legacy law conversion must apply once"
    );
}

#[test]
fn wrong_typed_persisted_fields_reject_the_entire_session_without_mutation() {
    let plugin = Ca72::default();
    let params = plugin.parameters().clone();
    let (_adapter, session) = Processor::new(plugin, &[]).expect("adapter");
    session.set_parameter(params.cutoff.as_ptr(), 0.2);
    params.editor_width.store(1250, Ordering::Relaxed);
    params.editor_panel.store(false, Ordering::Relaxed);
    *params.preset.write().expect("preset lock") = "Current unsaved sound".into();
    let before = session.save().expect("original session");
    for (field, wrong_type) in [
        ("editor_width", "\"wrong type\""),
        ("editor_width", "-1"),
        ("editor_width", "1250.5"),
        ("preset", "false"),
        ("editor_panel", "\"false\""),
    ] {
        let mut state: PluginState = serde_json::from_slice(&before).expect("state");
        state
            .params
            .insert("cutoff".into(), ParamValue::F32(1000.0));
        state
            .fields
            .insert("preset".into(), "\"Replacement sound\"".into());
        state.fields.insert(field.into(), wrong_type.into());
        let error = session
            .restore(&serde_json::to_vec(&state).expect("valid JSON with wrong type"))
            .expect_err("typed field must reject the whole restore");
        assert!(
            error.contains(field),
            "error must identify {field}: {error}"
        );
        assert_eq!(
            session.save().expect("state after rejected restore"),
            before,
            "a rejected {field} must not change any parameter or persisted field"
        );
    }
}

#[test]
fn typed_preflight_retains_ca72s_documented_midi_map_repair() {
    let plugin = Ca72::default();
    let params = plugin.parameters().clone();
    let (_adapter, session) = Processor::new(plugin, &[]).expect("adapter");
    let target = ca72_plugin::learn::LEARNABLE
        .iter()
        .position(|parameter| parameter.id == "cutoff")
        .expect("cutoff learnable");
    let cc = ca72_plugin::learn::Cc { channel: 4, cc: 74 };
    params.midi_map.assign(target, cc).expect("MIDI assignment");
    let before = session.save().expect("state with mapping");
    for repaired in ["null", "[]", "\"unknown MIDI table\""] {
        params.midi_map.assign(target, cc).expect("MIDI assignment");
        let mut state: PluginState = serde_json::from_slice(&before).expect("state");
        state
            .params
            .insert("cutoff".into(), ParamValue::F32(1000.0));
        state.fields.insert("midi_map".into(), repaired.into());
        let mut hosted = state.clone();
        Ca72::filter_state(&mut hosted);
        session
            .restore(&serde_json::to_vec(&state).expect("compatible MIDI table repair"))
            .expect("CA72 deliberately repairs unknown MIDI table shapes");
        assert_eq!(params.cutoff.value(), 1000.0);
        assert_eq!(params.midi_map.assignment(target), None);
        let after: PluginState =
            serde_json::from_slice(&session.save().expect("repaired session")).expect("state");
        assert_eq!(after.fields["midi_map"], hosted.fields["midi_map"]);
    }
}

#[derive(Default)]
struct TaskParameters;
// SAFETY: this test parameter object exposes no pointers at all.
unsafe impl Params for TaskParameters {
    fn param_map(&self) -> Vec<(String, ParamPtr, String)> {
        Vec::new()
    }
}
#[derive(Default)]
struct TaskFlood {
    params: Arc<TaskParameters>,
}
impl Plugin for TaskFlood {
    const NAME: &'static str = "Task flood test";
    const VENDOR: &'static str = "Idle Foundry";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = "0.0.0";
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_output_channels: Some(new_nonzero_u32(2)),
        ..AudioIOLayout::const_default()
    }];
    type SysExMessage = ();
    type BackgroundTask = u32;
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for task in 0..1024 {
            context.execute_gui(task);
            context.execute_background(task);
        }
        for channel in buffer.as_slice() {
            channel.fill(0.0);
        }
        ProcessStatus::KeepAlive
    }
}

#[test]
fn callback_task_queues_exhaust_without_spinning_allocating_or_dropping_heap_payloads() {
    let (mut adapter, _) = Processor::new(TaskFlood::default(), &[]).expect("task adapter");
    let (mut gui, mut background) = adapter
        .take_audio_task_receivers()
        .expect("single consumer transfer");
    assert!(
        adapter.take_audio_task_receivers().is_none(),
        "SPSC consumers cannot be duplicated"
    );
    adapter
        .prepare(ProcessConfig {
            sample_rate: 48_000.0,
            max_buffer_frames: 128,
        })
        .expect("prepared");
    let (mut left, mut right) = ([0.0; 128], [0.0; 128]);
    for _ in 0..2 {
        OPERATIONS.with(|n| n.set(0));
        TRACK.with(|track| track.set(true));
        // The consumers are deliberately stopped. 1024 pushes exhaust each
        // 256-item queue and must immediately reject the Copy payloads.
        adapter.process(AudioBlock {
            left: &mut left,
            right: &mut right,
            input_left: &[],
            input_right: &[],
            midi: &[],
            panic: false,
        });
        TRACK.with(|track| track.set(false));
        assert_eq!(
            OPERATIONS.with(Cell::get),
            0,
            "task scheduling allocated or freed inside callback"
        );
        let mut accepted = 0;
        while let Ok(task) = gui.pop() {
            assert_eq!(task, accepted);
            accepted += 1;
        }
        assert_eq!(accepted, 256);
        let mut accepted = 0;
        while let Ok(task) = background.pop() {
            assert_eq!(task, accepted);
            accepted += 1;
        }
        assert_eq!(accepted, 256);
    }
    adapter.deactivate();
}
