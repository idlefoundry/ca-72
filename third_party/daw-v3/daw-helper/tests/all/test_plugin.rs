//! The shipped test plugin, loaded in-process by the helper's CLAP host (M1 plan W5):
//! descriptor and parameters, bit-exact output, block-size invariance, state round trip;
//! names for its keys that change as it runs (D216); its own edits and its requests to
//! flush and to clear what refers to a parameter (D222).

#![allow(unsafe_code, clippy::unwrap_used, clippy::needless_range_loop)]

use daw_helper::clap::{Instance, Library};
use daw_ipc::Event;
use daw_ipc::layout::EV_PARAM;
use daw_ipc::protocol::{EditKind, ParamEdit};
use daw_test_plugin::{
    P_ANSWER, P_ASK_FLUSH, P_CHANGE_QUIETLY, P_CLEAR, P_EDIT, P_GAIN, P_HOLD, P_LOAD_SAMPLE,
    P_METER, P_MIX, P_NAMES, P_NUDGE, P_PRESET, P_PRESET_DIRTY, P_RELEASE, P_SAMPLE_PRESET,
    P_SAMPLE_QUIETLY,
};
use std::path::PathBuf;
use std::sync::Arc;

const ID: &str = "org.daw-v3.test-plugin";

/// The plugin library cargo built for this test (a dev-dependency).
fn plugin_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let deps = exe.parent().unwrap();
    let name = if cfg!(windows) {
        "daw_test_plugin.dll"
    } else if cfg!(target_os = "macos") {
        "libdaw_test_plugin.dylib"
    } else {
        "libdaw_test_plugin.so"
    };
    for dir in [deps, deps.parent().unwrap()] {
        let p = dir.join(name);
        if p.exists() {
            return p;
        }
    }
    panic!("{name} not found next to {}", exe.display());
}

fn instance(lib: &Arc<Library>, params: &[(u32, f64)], block: usize) -> Instance {
    let mut inst = lib.create(ID).unwrap();
    inst.flush_params(params);
    inst.activate(48_000.0, block as u32).unwrap();
    inst
}

/// Runs `input` through a fresh processor in blocks of `block` frames; `events` are
/// (absolute frame, parameter, value).
fn run(
    inst: &Instance,
    input: &[f32],
    block: usize,
    events: &[(usize, u32, f64)],
) -> (Vec<f32>, Vec<f32>) {
    let mut p = inst.processor(block);
    assert!(p.start());
    let mut out_l = vec![0.0f32; input.len()];
    let mut out_r = vec![0.0f32; input.len()];
    let mut in_l = input.to_vec();
    let mut in_r: Vec<f32> = input.iter().map(|x| -x).collect();
    let mut pos = 0;
    while pos < input.len() {
        let n = block.min(input.len() - pos);
        let evs: Vec<Event> = events
            .iter()
            .filter(|(f, _, _)| *f >= pos && *f < pos + n)
            .map(|(f, id, v)| Event {
                kind: EV_PARAM,
                time: (f - pos) as u32,
                param: *id,
                value: *v,
                ..Event::default()
            })
            .collect();
        // SAFETY: every buffer holds at least `pos + n` samples.
        let status = unsafe {
            p.process(
                pos as i64,
                n,
                &evs,
                [in_l.as_mut_ptr().add(pos), in_r.as_mut_ptr().add(pos)],
                [out_l.as_mut_ptr().add(pos), out_r.as_mut_ptr().add(pos)],
            )
        };
        assert_eq!(status, 0);
        pos += n;
    }
    p.stop();
    (out_l, out_r)
}

fn signal(len: usize) -> Vec<f32> {
    // Deterministic, non-trivial input.
    (0..len)
        .map(|i| ((i * 7919) % 1000) as f32 / 1000.0 - 0.5)
        .collect()
}

#[test]
fn descriptor_and_parameters() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let d = lib.descriptors();
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].id, ID);
    assert!(d[0].features.contains(&"audio-effect".to_owned()));
    let inst = lib.create(ID).unwrap();
    let params = inst.params();
    let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Gain",
            "Delay mix",
            "Delay",
            "Latency",
            "Crash",
            "Hang",
            "Names",
            "Edit names",
            "Nudge gain",
            "Ask flush",
            "Clear refs",
            "Answer gain",
            "Load preset",
            "Load preset quietly",
            "Hold gain",
            "Release gain",
            "Meter",
            "Load sample",
            "Load sample preset",
            "Change quietly",
            "Load sample quietly"
        ]
    );
    assert!(params[4..].iter().all(|p| p.hidden) && !params[0].hidden);
    // The names' enumeration chooses names (D216); nothing else does.
    assert!(params[6].program && params.iter().filter(|p| p.program).count() == 1);
    assert!(params[2].stepped);
    assert_eq!(inst.main_ports(), (2, 2));
    assert!(!inst.takes_notes());
    assert!(lib.create("no.such.plugin").is_err());
}

#[test]
fn output_is_the_documented_formula() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let inst = instance(&lib, &[(1, 0.5), (2, 0.25), (3, 10.0), (4, 3.0)], 64);
    assert_eq!(inst.latency(), 3);
    assert_eq!(inst.tail(), 13);
    let x = signal(1000);
    // Gain changes to 1.5 at frame 500, sample-accurately.
    let (l, r) = run(&inst, &x, 64, &[(500, 1, 1.5)]);
    let at = |n: usize, d: usize| if n >= d { x[n - d] } else { 0.0 };
    for n in 0..x.len() {
        let g = if n < 500 { 0.5f32 } else { 1.5 };
        let y = g * (at(n, 3) + 0.25 * at(n, 13));
        assert_eq!(l[n], y, "left at {n}");
        assert_eq!(r[n], -y, "right at {n}");
    }
}

#[test]
fn output_does_not_depend_on_block_size() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let params = [(1, 0.8), (2, 0.5), (3, 100.0), (4, 17.0)];
    let x = signal(5000);
    let events = [(1234, 2, 0.1), (3000, 3, 7.0)];
    let reference = run(&instance(&lib, &params, 4096), &x, 4096, &events);
    for block in [1, 7, 64, 333] {
        let got = run(&instance(&lib, &params, block), &x, block, &events);
        assert!(got == reference, "block size {block} changed the output");
    }
}

#[test]
fn state_round_trips_into_a_new_instance() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let a = lib.create(ID).unwrap();
    a.flush_params(&[(1, 0.3), (2, 0.6), (3, 42.0), (4, 5.0), (100, 1.0)]);
    let state = a.save_state().unwrap();
    let mut b = lib.create(ID).unwrap();
    b.load_state(&state).unwrap();
    let value = |i: &Instance| -> Vec<(String, f64)> {
        i.params()
            .into_iter()
            .map(|p| (p.name, p.default))
            .collect()
    };
    // Same parameter table; values compared through output below.
    assert_eq!(value(&a), value(&b));
    assert_eq!(
        b.save_state().unwrap(),
        state,
        "saving the loaded state gives the same bytes"
    );
    assert!(b.load_state(b"garbage").is_err());
    // The crash parameter was set on `a` but is not part of the state, so `b` processes.
    b.activate(48_000.0, 64).unwrap();
    assert_eq!(b.latency(), 5);
    let x = signal(300);
    let (l, _) = run(&b, &x, 64, &[]);
    let at = |n: usize, d: usize| if n >= d { x[n - d] } else { 0.0 };
    for n in 0..x.len() {
        assert_eq!(l[n], 0.3f32 * (at(n, 5) + 0.6 * at(n, 47)));
    }
}

/// D216: a running plugin's names are asked again when it says they changed
/// (`clap_host_note_name.changed`, on the main thread after the host's `names` value
/// reached it), and when it edits a parameter itself (its editor's output event); not for
/// another parameter's value.
#[test]
fn a_running_plugins_names_follow_its_changes() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let inst = instance(&lib, &[], 64);
    let mut p = inst.processor(64);
    assert!(p.start());
    let x = signal(64);
    let (mut il, mut ir) = (x.clone(), x);
    let (mut ol, mut or) = (vec![0.0f32; 64], vec![0.0f32; 64]);
    let mut block = |events: &[(u32, f64)]| {
        let evs: Vec<Event> = events
            .iter()
            .map(|(id, v)| Event {
                kind: EV_PARAM,
                param: *id,
                value: *v,
                ..Event::default()
            })
            .collect();
        // SAFETY: every buffer holds 64 samples.
        let status = unsafe {
            p.process(
                0,
                64,
                &evs,
                [il.as_mut_ptr(), ir.as_mut_ptr()],
                [ol.as_mut_ptr(), or.as_mut_ptr()],
            )
        };
        assert_eq!(status, 0);
    };
    let names = |i: &Instance| -> Vec<String> {
        i.note_names()
            .into_iter()
            .filter(|n| n.channel == -1)
            .map(|n| n.name)
            .collect()
    };
    assert!(!inst.take_names_changed());
    assert_eq!(names(&inst), ["Test Kick", "Test Snare", "Test Hat"]);

    // Another value: no word.
    block(&[(P_GAIN, 0.5)]);
    inst.service();
    assert!(!inst.take_names_changed());

    // The host sets `names`: the plugin says so on the main thread.
    block(&[(P_NAMES, 1.0)]);
    assert!(
        !inst.take_names_changed(),
        "said on the main thread, not in process"
    );
    inst.service();
    assert!(inst.take_names_changed(), "clap_host_note_name.changed");
    assert_eq!(names(&inst), ["Alt Kick", "Alt Snare", "Alt Hat"]);

    // Its editor chooses the third set itself: an output event, and no word.
    block(&[(P_EDIT, 2.0)]);
    assert!(inst.take_names_changed(), "the editor's output event");
    inst.service();
    assert!(
        !inst.take_names_changed(),
        "no word through the note-name extension"
    );
    assert_eq!(names(&inst), ["Edit Kick", "Edit Snare", "Edit Hat"]);
    p.stop();
}

/// D222: a running plugin's own edits are kept for the project: a value with no gesture,
/// a gesture (its beginning and end with the value it has then); a flush it asks for from
/// the main thread is made between blocks, on the audio thread, or is the next block; and
/// what it asks to clear is kept.
#[test]
fn a_running_plugins_own_edits_and_requests_are_kept() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let inst = instance(&lib, &[], 64);
    let mut p = inst.processor(64);
    assert!(p.start());
    let x = signal(64);
    let (mut il, mut ir) = (x.clone(), x);
    let (mut ol, mut or) = (vec![0.0f32; 64], vec![0.0f32; 64]);
    let mut block = |p: &mut daw_helper::clap::Processor, events: &[(u32, f64)]| {
        let evs: Vec<Event> = events
            .iter()
            .map(|(id, v)| Event {
                kind: EV_PARAM,
                param: *id,
                value: *v,
                ..Event::default()
            })
            .collect();
        // SAFETY: every buffer holds 64 samples.
        let status = unsafe {
            p.process(
                0,
                64,
                &evs,
                [il.as_mut_ptr(), ir.as_mut_ptr()],
                [ol.as_mut_ptr(), or.as_mut_ptr()],
            )
        };
        assert_eq!(status, 0);
    };
    let edit = |param, kind, value| ParamEdit {
        param,
        kind,
        value: Some(value),
    };
    let gesture = |param, value| {
        vec![
            edit(param, EditKind::Begin, value),
            edit(param, EditKind::Value, value),
            edit(param, EditKind::End, value),
        ]
    };
    assert_eq!(inst.take_edits(), []);
    // A value with no gesture.
    block(&mut p, &[(P_NUDGE, 1.5)]);
    assert_eq!(inst.take_edits(), [edit(P_GAIN, EditKind::Value, 1.5)]);
    // A gesture, its beginning and end with the value it has then.
    block(&mut p, &[(P_EDIT, 1.0)]);
    assert_eq!(inst.take_edits(), gesture(P_NAMES, 1.0));

    // A flush asked for from the main thread: made between blocks.
    block(&mut p, &[(P_ASK_FLUSH, 0.5)]);
    inst.service();
    assert_eq!(inst.take_edits(), [], "not before the flush");
    p.idle();
    assert_eq!(
        inst.take_edits(),
        gesture(P_GAIN, 0.5),
        "the flush between blocks"
    );
    // Or the next block is the flush: none between blocks after it.
    block(&mut p, &[(P_ASK_FLUSH, 0.7)]);
    inst.service();
    block(&mut p, &[]);
    p.idle();
    assert_eq!(
        inst.take_edits(),
        gesture(P_GAIN, 0.7),
        "the next block, once"
    );

    // Its answers to the host's values (the same value sent back, or one marked not to be
    // recorded) are not edits (the review's finding).
    block(&mut p, &[(P_ANSWER, 1.0), (P_GAIN, 0.8)]);
    assert_eq!(inst.take_edits(), [], "the value sent back");
    block(&mut p, &[(P_ANSWER, 2.0), (P_GAIN, 1.2)]);
    assert_eq!(inst.take_edits(), [], "the value not to be recorded");
    block(&mut p, &[(P_ANSWER, 0.0)]);

    // What it asks to clear.
    block(&mut p, &[(P_CLEAR, 2.0)]);
    assert_eq!(inst.take_clears(), []);
    inst.service();
    assert_eq!(inst.take_clears(), [(P_GAIN, 2)]);
    p.stop();

    // Not active: the flush it asks for is made on the main thread.
    let idle = lib.create(ID).unwrap();
    idle.flush_params(&[(P_ASK_FLUSH, 1.2)]);
    idle.service();
    assert_eq!(
        idle.take_edits(),
        gesture(P_GAIN, 1.2),
        "flushed on the main thread"
    );
}

/// D228: a preset the plugin loads itself is read again: it says so on the main thread with
/// a rescan of values, or only by marking its state dirty, and its values are the preset's.
/// D227's stand-in for its editor's knob held: a gesture begun with a value, ended later.
#[test]
fn a_preset_the_plugin_loads_is_read_again() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let inst = instance(&lib, &[], 64);
    let mut p = inst.processor(64);
    assert!(p.start());
    let x = signal(64);
    let (mut il, mut ir) = (x.clone(), x);
    let (mut ol, mut or) = (vec![0.0f32; 64], vec![0.0f32; 64]);
    let mut block = |p: &mut daw_helper::clap::Processor, events: &[(u32, f64)]| {
        let evs: Vec<Event> = events
            .iter()
            .map(|(id, v)| Event {
                kind: EV_PARAM,
                param: *id,
                value: *v,
                ..Event::default()
            })
            .collect();
        // SAFETY: every buffer holds 64 samples.
        let status = unsafe {
            p.process(
                0,
                64,
                &evs,
                [il.as_mut_ptr(), ir.as_mut_ptr()],
                [ol.as_mut_ptr(), or.as_mut_ptr()],
            )
        };
        assert_eq!(status, 0);
    };
    let of = |inst: &Instance, id: u32| inst.param_values().iter().find(|v| v.0 == id).map(|v| v.1);
    inst.service();
    assert!(!inst.take_values_changed());
    // At rest, nothing.
    block(&mut p, &[(P_PRESET, 0.0)]);
    inst.service();
    assert!(!inst.take_values_changed(), "at rest");
    // Loaded with a rescan of values.
    block(&mut p, &[(P_PRESET, 1.2)]);
    assert!(!inst.take_values_changed(), "said on the main thread");
    inst.service();
    assert!(inst.take_values_changed(), "the rescan");
    assert_eq!(of(&inst, P_GAIN), Some(1.2));
    assert_eq!(of(&inst, P_MIX), Some(0.6));
    // Not what it shows but does not take (the review's finding).
    assert_eq!(of(&inst, P_METER), None, "a read-only parameter");
    assert_eq!(inst.take_edits(), [], "no edits of its own");
    // Loaded quietly: its state marked dirty.
    block(&mut p, &[(P_PRESET_DIRTY, 0.4)]);
    inst.service();
    assert!(inst.take_values_changed(), "the state marked dirty");
    assert_eq!(of(&inst, P_GAIN), Some(0.4));
    assert_eq!(of(&inst, P_MIX), Some(0.2));

    // D227: its editor's knob held (a gesture begun with a value), let go later.
    let edit = |kind, value| ParamEdit {
        param: P_GAIN,
        kind,
        value: Some(value),
    };
    block(&mut p, &[(P_HOLD, 0.8)]);
    assert_eq!(
        inst.take_edits(),
        [edit(EditKind::Begin, 0.8), edit(EditKind::Value, 0.8)]
    );
    block(&mut p, &[]);
    assert_eq!(inst.take_edits(), [], "held");
    block(&mut p, &[(P_RELEASE, 1.0)]);
    assert_eq!(inst.take_edits(), [edit(EditKind::End, 0.8)]);
    p.stop();
}

/// D234: what it keeps besides its parameters (its sample) is in its state: a sample loaded
/// with a preset's values (a rescan), alone (its state marked dirty) or without a word; values
/// changed without a word; the host's values and the plugin's own told apart; the state's
/// sample heard in a new instance it is loaded into, and taken back by a state without one.
#[test]
fn a_samples_state_changes_with_a_word_or_without_one() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let inst = instance(&lib, &[], 64);
    let mut p = inst.processor(64);
    assert!(p.start());
    let x = signal(64);
    let (mut il, mut ir) = (x.clone(), x.clone());
    let (mut ol, mut or) = (vec![0.0f32; 64], vec![0.0f32; 64]);
    let mut block = |p: &mut daw_helper::clap::Processor, events: &[(u32, f64)]| -> f32 {
        let evs: Vec<Event> = events
            .iter()
            .map(|(id, v)| Event {
                kind: EV_PARAM,
                param: *id,
                value: *v,
                ..Event::default()
            })
            .collect();
        il.copy_from_slice(&x);
        ir.copy_from_slice(&x);
        // SAFETY: every buffer holds 64 samples.
        let status = unsafe {
            p.process(
                0,
                64,
                &evs,
                [il.as_mut_ptr(), ir.as_mut_ptr()],
                [ol.as_mut_ptr(), or.as_mut_ptr()],
            )
        };
        assert_eq!(status, 0);
        ol.iter().fold(0.0f32, |m, v| m.max(v.abs()))
    };
    let of = |inst: &Instance, id: u32| inst.param_values().iter().find(|v| v.0 == id).map(|v| v.1);
    let full = block(&mut p, &[]);
    inst.service();
    assert!(!inst.take_values_changed());
    let before = inst.save_state().unwrap();
    // A sample loaded alone: its state marked dirty, its values as they were.
    block(&mut p, &[(P_LOAD_SAMPLE, 0.5)]);
    inst.service();
    assert!(inst.take_values_changed(), "its state marked dirty");
    assert_eq!(of(&inst, P_GAIN), Some(1.0));
    let half = block(&mut p, &[]);
    assert!(
        (half - full / 2.0).abs() < 1e-6,
        "the sample's level: {half}"
    );
    let with = inst.save_state().unwrap();
    assert_ne!(with, before, "in its state");
    // A sample with a preset's values: a rescan.
    block(&mut p, &[(P_SAMPLE_PRESET, 1.2)]);
    inst.service();
    assert!(inst.take_values_changed(), "the rescan");
    assert_eq!(of(&inst, P_GAIN), Some(1.2));
    assert_eq!(of(&inst, P_MIX), Some(0.6));
    // Without a word: values, or a sample.
    block(&mut p, &[(P_CHANGE_QUIETLY, 0.8)]);
    inst.service();
    assert!(!inst.take_values_changed(), "no word");
    assert_eq!(of(&inst, P_GAIN), Some(0.8));
    block(&mut p, &[(P_SAMPLE_QUIETLY, 0.25)]);
    inst.service();
    assert!(!inst.take_values_changed(), "no word");
    // The host's values are known as the host's (its watch tells them from the plugin's).
    let given = inst.take_host_given();
    assert!(
        given.contains(&P_SAMPLE_QUIETLY) && !given.contains(&P_GAIN),
        "the host's values: {given:?}"
    );
    assert_eq!(inst.host_value(P_SAMPLE_QUIETLY), Some(0.25));
    assert_eq!(inst.host_value(P_GAIN), None);
    // The state with the sample, in a new instance: heard there; one without takes it back.
    let other = instance(&lib, &[], 64);
    other.load_state(&with).unwrap();
    let mut q = other.processor(64);
    assert!(q.start());
    let heard = block(&mut q, &[]);
    assert!(
        (heard - full / 2.0).abs() < 1e-6,
        "the state's sample: {heard}"
    );
    q.stop();
    other.load_state(&before).unwrap();
    let mut q = other.processor(64);
    assert!(q.start());
    let back = block(&mut q, &[]);
    assert!((back - full).abs() < 1e-6, "no sample: {back}");
    p.stop();
}

/// The CLAP fixture uses the same baseline acknowledgement as its VST3 counterpart;
/// audio progress alone cannot assume the helper has made a state poll.
#[test]
fn a_quiet_sample_waits_for_its_baseline_before_changing_under_a_slow_host() {
    let lib = Arc::new(Library::load(&plugin_path()).unwrap());
    let inst = instance(&lib, &[], 256);
    let mut p = inst.processor(256);
    assert!(p.start());
    let now = std::time::Instant::now();
    let cost = std::time::Duration::from_micros(10);
    let initial = inst.save_state().unwrap();
    let mut watch =
        daw_helper::watch::Watch::new(now, &inst.param_values(), Some((&initial, cost)));
    let (mut input_l, mut input_r) = ([0.0f32; 256], [0.0f32; 256]);
    let (mut output_l, mut output_r) = ([0.0f32; 256], [0.0f32; 256]);
    let mut block = |events: &[Event]| {
        assert_eq!(
            // SAFETY: each channel buffer holds the advertised 256 samples.
            unsafe {
                p.process(
                    0,
                    256,
                    events,
                    [input_l.as_mut_ptr(), input_r.as_mut_ptr()],
                    [output_l.as_mut_ptr(), output_r.as_mut_ptr()],
                )
            },
            0
        );
    };
    block(&[Event {
        kind: EV_PARAM,
        param: P_SAMPLE_QUIETLY,
        value: 0.25,
        ..Event::default()
    }]);
    inst.service();
    watch.note_host(&inst.take_host_given(), &|id| inst.host_value(id));
    for _ in 0..600 {
        block(&[]);
    }
    let sample = |bytes: &[u8]| {
        bytes
            .windows(12)
            .find(|w| w[..4] == 0xFFFF_FF01u32.to_le_bytes())
            .map_or(1.0, |w| f64::from_le_bytes(w[4..12].try_into().unwrap()))
    };
    let baseline = inst.save_state().unwrap();
    assert_eq!(
        sample(&baseline),
        1.0,
        "the quiet change raced its first state poll"
    );
    let values = inst.param_values();
    let host = |id| inst.host_value(id);
    assert!(
        !watch.poll_state(now, &values, &baseline, cost, &host),
        "the trigger is the host's"
    );
    for _ in 0..300 {
        block(&[]);
    }
    let changed = inst.save_state().unwrap();
    assert_eq!(sample(&changed), 0.25);
    assert!(
        !inst.take_values_changed(),
        "the fixture must still say nothing"
    );
    assert!(!watch.poll_state(now, &values, &changed, cost, &host));
    assert!(
        watch.poll_state(now, &values, &changed, cost, &host),
        "held for a state poll"
    );
    assert!(
        watch
            .read(
                now,
                &values,
                Some((&changed, cost)),
                daw_helper::watch::Said::Values,
                &host
            )
            .alone
    );
    p.stop();
}
