//! The shipped VST3 test plugin, loaded in-process by the helper's VST3 host (M6 plan W1):
//! its class and parameters, output exactly the documented formula, block-size
//! invariance with parameter changes inside blocks, and a state round trip; the
//! instrument's names for its keys (its program pitch names, D211), and their changes
//! while it runs (D216); every parameter of more than six hundred reaching the processor
//! and the controller, and the editor's edits reaching the processor (D222).

#![allow(unsafe_code, clippy::unwrap_used, clippy::needless_range_loop)]

use daw_helper::vst3::{Instance, Module, cid_hex};
use daw_ipc::Event;
use daw_ipc::layout::{EV_NOTE_OFF, EV_NOTE_ON, EV_PARAM};
use daw_ipc::protocol::{EditKind, ParamEdit};
use daw_test_vst3::{
    P_ANSWER, P_CHANGE_QUIETLY, P_DELAY, P_EDIT, P_EDIT_GAIN, P_GAIN, P_KIT, P_LATENCY, P_LAYER,
    P_LEVEL, P_LOAD_SAMPLE, P_METER, P_MIX, P_NUDGE, P_PAD, P_PRESET, P_PROCESSOR_GAIN,
    P_SAMPLE_PRESET, P_SAMPLE_QUIETLY, P_SIGNAL, P_SPARE, PROCESSOR_CID, SINE_CID, SPARES,
};
use std::path::PathBuf;
use std::sync::Arc;

/// The plugin library cargo built for this test (a dev-dependency).
fn plugin_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    let deps = exe.parent().unwrap();
    let name = if cfg!(windows) {
        "daw_test_vst3.dll"
    } else if cfg!(target_os = "macos") {
        "libdaw_test_vst3.dylib"
    } else {
        "libdaw_test_vst3.so"
    };
    for dir in [deps, deps.parent().unwrap()] {
        let p = dir.join(name);
        if p.exists() {
            return p;
        }
    }
    panic!("{name} not found next to {}", exe.display());
}

fn id() -> String {
    cid_hex(&PROCESSOR_CID)
}

/// Normalized values for plain ones, as the plugin maps them.
fn gain(g: f64) -> f64 {
    g / 2.0
}
fn frames(f: f64, min: f64, max: f64) -> f64 {
    (f - min) / (max - min)
}
fn delay(d: f64) -> f64 {
    frames(d, 1.0, 4800.0)
}
fn latency(l: f64) -> f64 {
    frames(l, 0.0, 4096.0)
}

fn instance(m: &Arc<Module>, params: &[(u32, f64)], block: usize) -> Instance {
    let mut inst = m.instantiate(&id()).unwrap();
    inst.flush_params(params);
    inst.activate(48_000.0, block as u32).unwrap();
    inst
}

/// Runs `input` through a fresh processor in blocks of `block` frames; `events` are
/// (absolute frame, parameter, normalized value).
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
                daw_ipc::Timing::default(),
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
    (0..len)
        .map(|i| ((i * 7919) % 1000) as f32 / 1000.0 - 0.5)
        .collect()
}

#[test]
fn class_and_parameters() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let d = m.descriptors();
    assert_eq!(d.len(), 2, "{d:?}");
    assert_eq!(d[0].id, id());
    assert_eq!(d[0].name, "DAW-v3 Test Gain");
    assert_eq!(d[0].vendor, "DAW-v3");
    assert_eq!(d[0].features, ["Fx"]);
    assert_eq!(d[1].id, cid_hex(&SINE_CID));
    assert_eq!(d[1].features, ["Instrument", "Synth"]);
    let inst = m.instantiate(&id()).unwrap();
    let params = inst.params();
    let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        ["Gain", "Delay mix", "Delay", "Latency", "Crash", "Hang"]
    );
    assert!(params[4].hidden && params[5].hidden && !params[0].hidden);
    assert_eq!(
        (params[0].min, params[0].max, params[0].default),
        (0.0, 1.0, 0.5)
    );
    assert!(params[0].automatable && !params[0].modulatable);
    assert_eq!(inst.main_ports(), (2, 2));
    assert!(!inst.takes_notes());
    assert!(m.instantiate("00000000000000000000000000000000").is_err());
    assert!(m.instantiate("not a class").is_err());
}

#[test]
fn output_is_the_documented_formula() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let inst = instance(
        &m,
        &[
            (P_GAIN, gain(0.5)),
            (P_MIX, 0.25),
            (P_DELAY, delay(10.0)),
            (P_LATENCY, latency(3.0)),
        ],
        64,
    );
    assert_eq!(inst.latency(), 3);
    let x = signal(1000);
    // Gain changes to 1.5 at frame 500, sample-accurately.
    let (l, r) = run(&inst, &x, 64, &[(500, P_GAIN, gain(1.5))]);
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
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let params = [
        (P_GAIN, gain(0.8)),
        (P_MIX, 0.5),
        (P_DELAY, delay(100.0)),
        (P_LATENCY, latency(17.0)),
    ];
    let x = signal(5000);
    let events = [(1234, P_MIX, 0.1), (3000, P_DELAY, delay(7.0))];
    let reference = run(&instance(&m, &params, 4096), &x, 4096, &events);
    for block in [1, 7, 64, 333] {
        let got = run(&instance(&m, &params, block), &x, block, &events);
        assert!(got == reference, "block size {block} changed the output");
    }
}

#[test]
fn state_round_trips_into_a_new_instance() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut a = m.instantiate(&id()).unwrap();
    a.flush_params(&[
        (P_GAIN, gain(0.3)),
        (P_MIX, 0.6),
        (P_DELAY, delay(42.0)),
        (P_LATENCY, latency(5.0)),
    ]);
    // Values reach the processor when it is activated (a flush).
    a.activate(48_000.0, 64).unwrap();
    let state = a.save_state().unwrap();
    let mut b = m.instantiate(&id()).unwrap();
    b.load_state(&state).unwrap();
    assert_eq!(
        b.save_state().unwrap(),
        state,
        "saving the loaded state gives the same bytes"
    );
    assert!(b.load_state(b"garbage").is_err());
    b.activate(48_000.0, 64).unwrap();
    assert_eq!(b.latency(), 5);
    let x = signal(300);
    let (l, _) = run(&b, &x, 64, &[]);
    let at = |n: usize, d: usize| if n >= d { x[n - d] } else { 0.0 };
    for n in 0..x.len() {
        assert_eq!(l[n], 0.3f32 * (at(n, 5) + 0.6 * at(n, 47)), "at {n}");
    }
}

/// Runs an instrument for `len` frames in blocks of `block`; `notes` are (frame, on, key,
/// velocity).
fn play(inst: &Instance, len: usize, block: usize, notes: &[(usize, bool, i16, f64)]) -> Vec<f32> {
    let mut p = inst.processor(block);
    assert!(p.start());
    let mut out_l = vec![0.0f32; len];
    let mut out_r = vec![0.0f32; len];
    let mut silence = vec![0.0f32; len];
    let mut pos = 0;
    while pos < len {
        let n = block.min(len - pos);
        let evs: Vec<Event> = notes
            .iter()
            .filter(|(f, ..)| *f >= pos && *f < pos + n)
            .map(|(f, on, key, v)| Event {
                kind: if *on { EV_NOTE_ON } else { EV_NOTE_OFF },
                time: (f - pos) as u32,
                key: *key,
                note_id: i32::from(*key),
                value: *v,
                ..Event::default()
            })
            .collect();
        let s = silence.as_mut_ptr();
        // SAFETY: every buffer holds at least `pos + n` samples.
        let status = unsafe {
            p.process(
                pos as i64,
                daw_ipc::Timing::default(),
                n,
                &evs,
                [s.add(pos), s.add(pos)],
                [out_l.as_mut_ptr().add(pos), out_r.as_mut_ptr().add(pos)],
            )
        };
        assert_eq!(status, 0);
        pos += n;
    }
    p.stop();
    assert_eq!(out_l, out_r);
    out_l
}

#[test]
fn an_instrument_plays_its_notes_at_their_frames() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let sine = cid_hex(&SINE_CID);
    let make = |block: usize| {
        let mut i = m.instantiate(&sine).unwrap();
        i.activate(48_000.0, block as u32).unwrap();
        i
    };
    let inst = make(64);
    assert!(inst.takes_notes());
    assert_eq!(inst.main_ports(), (0, 2));
    // A4 at half velocity from frame 10 to 500, E5 from 200 to 700.
    let notes = [
        (10, true, 69, 0.5),
        (200, true, 76, 0.25),
        (500, false, 69, 0.0),
        (700, false, 76, 0.0),
    ];
    let got = play(&inst, 1000, 64, &notes);
    // The sum of the sounding sines, each from its own note-on, as the plugin computes it
    // (gain 1, no delay mix).
    let step = |key: f64| std::f64::consts::TAU * 440.0 * ((key - 69.0) / 12.0).exp2() / 48_000.0;
    let (mut pa, mut pe) = (0.0f64, 0.0f64);
    for n in 0..1000 {
        let mut y = 0.0f64;
        if (10..500).contains(&n) {
            y += 0.5 * pa.sin();
            pa += step(69.0);
        }
        if (200..700).contains(&n) {
            y += 0.25 * pe.sin();
            pe += step(76.0);
        }
        assert_eq!(got[n], y as f32, "at {n}");
    }
    for block in [1, 7, 333] {
        assert!(
            play(&make(block), 1000, block, &notes) == got,
            "block {block}"
        );
    }
}

/// D211: the instrument's names for its keys are the program pitch names of the program its
/// note input's unit (the drums, not the root) plays, chosen by that unit's program-change
/// parameter as VST3 makes a discrete value; not another unit's list or parameter. The
/// effect, with no program list, names none.
#[test]
fn an_instruments_keys_are_named_by_its_programs_pitch_names() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let names = |i: &Instance| -> Vec<(i16, i16, String)> {
        i.note_names()
            .into_iter()
            .map(|n| (n.key, n.channel, n.name))
            .collect()
    };
    let kit = |names: &[(i16, &str)]| -> Vec<(i16, i16, String)> {
        // A unit of the first channel's (not the root's): channel 0.
        names
            .iter()
            .map(|(k, n)| (*k, 0, (*n).to_owned()))
            .collect()
    };
    let kit_a = kit(&[(36, "VST Kick"), (38, "VST Snare"), (42, "VST Hat")]);
    let kit_b = kit(&[
        (36, "VST Deep Kick"),
        (39, "VST Clap"),
        (46, "VST Open Hat"),
    ]);
    let kit_c = kit(&[(35, "VST Low Kick"), (40, "VST Rim"), (49, "VST Crash")]);
    let sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    // The kit parameter is listed but not automatable; the other units' are.
    let params = sine.params();
    let automatable = |id: u32| params.iter().find(|p| p.id == id).map(|p| p.automatable);
    assert_eq!(
        [automatable(P_LAYER), automatable(P_KIT), automatable(P_PAD)],
        [Some(true), Some(false), Some(true)]
    );
    // Only the note input's unit's program change chooses names (D216): the cache of a
    // device made alone is keyed with its value alone.
    let programs: Vec<u32> = params.iter().filter(|p| p.program).map(|p| p.id).collect();
    assert_eq!(programs, [P_KIT]);
    assert_eq!(names(&sine), kit_a);
    // The other units' programs move nothing.
    sine.flush_params(&[(P_LAYER, 0.0), (P_PAD, 0.0)]);
    assert_eq!(names(&sine), kit_a);
    sine.flush_params(&[(P_LAYER, 1.0), (P_PAD, 1.0)]);
    assert_eq!(names(&sine), kit_a);
    // VST3's discrete value of three programs, `min(2, v * 3)` (rounding `v * 2` would
    // give Kit B at 0.3 and at 0.67).
    for (v, want) in [
        (0.3, &kit_a),
        (0.34, &kit_b),
        (0.5, &kit_b),
        (0.66, &kit_b),
        (0.67, &kit_c),
        (1.0, &kit_c),
    ] {
        sine.flush_params(&[(P_KIT, v)]);
        assert_eq!(&names(&sine), want, "Kit at {v}");
    }
    let gain = m.instantiate(&id()).unwrap();
    assert_eq!(names(&gain), []);
}

/// D216: while the instrument runs, its names follow a program change the host sends its
/// processor (the controller is given the value on the main thread, and the names are asked
/// again only when the program moves), an edit its editor makes (`performEdit`), and its
/// word that its programs, units or parameters changed (`IUnitHandler`, `IUnitHandler2`,
/// `restartComponent`); a latency change is a restart and not a word about names; a rename
/// without a word is not seen.
#[test]
fn a_running_instruments_names_follow_its_changes() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 64).unwrap();
    let mut p = sine.processor(64);
    assert!(p.start());
    let (mut l, mut r) = (vec![0.0f32; 64], vec![0.0f32; 64]);
    let mut silence = vec![0.0f32; 64];
    // One block with parameter values sent to the processor, then the main thread's turn.
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
        let s = silence.as_mut_ptr();
        // SAFETY: every buffer holds 64 samples.
        let status = unsafe {
            p.process(
                0,
                daw_ipc::Timing::default(),
                64,
                &evs,
                [s, s],
                [l.as_mut_ptr(), r.as_mut_ptr()],
            )
        };
        assert_eq!(status, 0);
    };
    let names =
        |i: &Instance| -> Vec<String> { i.note_names().into_iter().map(|n| n.name).collect() };
    let kit = |v: f64| {
        sine.params()
            .iter()
            .find(|p| p.id == P_KIT)
            .and_then(|p| p.value)
            == Some(v)
    };
    assert!(!sine.take_names_changed());
    assert_eq!(names(&sine), ["VST Kick", "VST Snare", "VST Hat"]);

    // The host's program change: the controller learns it on the main thread.
    block(&[(P_KIT, 0.5)]);
    assert!(
        !kit(0.5),
        "the controller was given the value on the audio thread"
    );
    sine.service();
    assert!(
        kit(0.5),
        "the controller has the value the processor was sent"
    );
    assert!(sine.take_names_changed(), "the program moved");
    assert_eq!(names(&sine), ["VST Deep Kick", "VST Clap", "VST Open Hat"]);
    // Within the same program: not asked again.
    block(&[(P_KIT, 0.6), (P_GAIN, 0.3)]);
    sine.service();
    assert!(kit(0.6));
    assert!(!sine.take_names_changed(), "the program did not move");

    // Its editor chooses Kit C itself (the stand-in, sent to the controller as any value).
    block(&[(P_EDIT, 1.0)]);
    sine.service();
    assert!(sine.take_names_changed(), "performEdit");
    assert_eq!(names(&sine), ["VST Low Kick", "VST Rim", "VST Crash"]);

    // The host's kit again after the editor's choice: the controller had Kit C, and is
    // given Kit B's value (the program the host last sent): asked again (the review's
    // finding: compared with the controller's program, not the host's last).
    block(&[(P_KIT, 0.6)]);
    sine.service();
    assert!(sine.take_names_changed(), "the editor's program replaced");
    assert_eq!(names(&sine), ["VST Deep Kick", "VST Clap", "VST Open Hat"]);
    block(&[(P_EDIT, 1.0)]);
    sine.service();
    assert!(sine.take_names_changed());

    // Renamed, with each word, then without one.
    for (step, word) in [
        (1, "notifyProgramListChange"),
        (2, "notifyUnitSelection"),
        (3, "restartComponent(kParamValuesChanged)"),
        (4, "restartComponent(kParamTitlesChanged)"),
        (5, "notifyUnitByBusChange"),
    ] {
        block(&[(P_SIGNAL, f64::from(step) / 7.0)]);
        sine.service();
        assert!(sine.take_names_changed(), "{word}");
        assert_eq!(
            names(&sine),
            [
                format!("VST Low Kick {step}"),
                format!("VST Rim {step}"),
                format!("VST Crash {step}")
            ],
            "{word}"
        );
        // A change of values or titles is not a restart.
        assert!(!sine.restart_requested(), "{word}");
    }
    // A latency change: a restart, not a word about names.
    block(&[(P_SIGNAL, 6.0 / 7.0)]);
    sine.service();
    assert!(
        sine.restart_requested(),
        "restartComponent(kLatencyChanged)"
    );
    assert!(!sine.take_names_changed(), "kLatencyChanged");
    block(&[(P_SIGNAL, 1.0)]);
    sine.service();
    assert!(!sine.take_names_changed(), "a rename without a word");
    p.stop();
}

/// Runs one block of 256 frames with parameter values and a held A4 (velocity 0.5) on
/// `p`, and gives its peak.
fn block_peak(p: &mut daw_helper::vst3::Processor, params: &[(u32, f64)], note_on: bool) -> f32 {
    let mut evs: Vec<Event> = params
        .iter()
        .map(|(id, v)| Event {
            kind: EV_PARAM,
            param: *id,
            value: *v,
            ..Event::default()
        })
        .collect();
    if note_on {
        evs.push(Event {
            kind: EV_NOTE_ON,
            key: 69,
            note_id: 69,
            value: 0.5,
            ..Event::default()
        });
    }
    let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
    let mut silence = vec![0.0f32; 256];
    let s = silence.as_mut_ptr();
    // SAFETY: every buffer holds 256 samples.
    let status = unsafe {
        p.process(
            0,
            daw_ipc::Timing::default(),
            256,
            &evs,
            [s, s],
            [l.as_mut_ptr(), r.as_mut_ptr()],
        )
    };
    assert_eq!(status, 0);
    l.iter().fold(0.0f32, |m, x| m.max(x.abs()))
}

/// D222: every parameter the instrument has reaches its processor and its controller, its
/// more than eight hundred: `Level` (its 806th) in a block halves the note and the controller
/// has it after the main thread's turn; all its spares and `Level` set before it is made
/// active, or in one block, reach the processor too.
#[test]
fn every_parameter_reaches_the_processor_and_the_controller() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let level = |i: &Instance| {
        i.params()
            .iter()
            .find(|p| p.id == P_LEVEL)
            .and_then(|p| p.value)
    };
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    assert!(
        sine.params().len() > SPARES,
        "{} parameters",
        sine.params().len()
    );
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let full = block_peak(&mut p, &[], true);
    assert!((full - 0.5).abs() < 0.01, "{full}");
    let half = block_peak(&mut p, &[(P_LEVEL, 0.25)], false);
    assert!((half - 0.25).abs() < 0.01, "Level in a block: {half}");
    assert_eq!(
        level(&sine),
        Some(0.5),
        "the controller is told on the main thread"
    );
    sine.service();
    assert_eq!(level(&sine), Some(0.25));
    p.stop();

    // All of them before it is active: Level, last, still reaches the processor.
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    let mut all: Vec<(u32, f64)> = (0..SPARES as u32 - 1).map(|k| (P_SPARE + k, 0.0)).collect();
    all.push((P_LEVEL, 0.25));
    sine.flush_params(&all);
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let peak = block_peak(&mut p, &[], true);
    assert!(
        (peak - 0.25).abs() < 0.01,
        "Level set before activation: {peak}"
    );
    p.stop();

    // As many as one block from the sandbox carries (512 events, all of them different
    // parameters): Level, last, reaches the processor in that block.
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let full: Vec<(u32, f64)> = all[all.len() - daw_ipc::MAX_EVENTS..].to_vec();
    let peak = block_peak(&mut p, &full, true);
    assert!(
        (peak - 0.25).abs() < 0.01,
        "Level in a block of every parameter: {peak}"
    );
    p.stop();
}

/// D222: the editor's edits reach the processor in its next block (the host takes them
/// there, as `IComponentHandler` is for), and are kept for the project: a gesture of two
/// values with its beginning and end, and a value with none.
#[test]
fn the_editors_edits_reach_the_processor_and_are_kept() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let full = block_peak(&mut p, &[], true);
    assert!((full - 0.5).abs() < 0.01, "{full}");
    assert_eq!(sine.take_edits(), []);
    // The editor sets the gain to 0.25 (a plain 0.5) in one gesture.
    block_peak(&mut p, &[(P_EDIT_GAIN, 0.25)], false);
    sine.service();
    let edit = |kind, value| ParamEdit {
        param: P_GAIN,
        kind,
        value: Some(value),
    };
    assert_eq!(
        sine.take_edits(),
        [
            edit(EditKind::Begin, 0.25),
            edit(EditKind::Value, 0.125),
            edit(EditKind::Value, 0.25),
            edit(EditKind::End, 0.25),
        ]
    );
    let half = block_peak(&mut p, &[], false);
    assert!(
        (half - 0.25).abs() < 0.01,
        "the editor's gain in the processor: {half}"
    );
    // Values with no gesture.
    block_peak(&mut p, &[(P_NUDGE, 0.75)], false);
    sine.service();
    assert_eq!(
        sine.take_edits(),
        [edit(EditKind::Value, 0.375), edit(EditKind::Value, 0.75)]
    );
    let more = block_peak(&mut p, &[], false);
    assert!((more - 0.75).abs() < 0.01, "the nudged gain: {more}");
    p.stop();
}

/// D228: a preset its controller loads itself (`restartComponent(kParamValuesChanged)`, no
/// edits) is read again: the helper hears that its values changed, and its controller's
/// values are the preset's. Its processor was not told, so it plays as before until the
/// host sends them.
#[test]
fn a_preset_the_controller_loads_is_read_again() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let full = block_peak(&mut p, &[], true);
    assert!((full - 0.5).abs() < 0.01, "{full}");
    sine.service();
    assert!(!sine.take_values_changed());
    // At its rest it loads nothing.
    block_peak(&mut p, &[(P_PRESET, 0.0)], false);
    sine.service();
    assert!(!sine.take_values_changed(), "at rest");
    block_peak(&mut p, &[(P_PRESET, 0.8)], false);
    sine.service();
    assert!(sine.take_values_changed(), "its values changed");
    assert!(!sine.take_values_changed(), "once");
    let values = sine.param_values();
    let of = |id: u32| values.iter().find(|v| v.0 == id).map(|v| v.1);
    assert_eq!(of(P_GAIN), Some(0.8));
    assert_eq!(of(P_LEVEL), Some(0.8));
    // Every parameter but the one it shows and does not take (its meter, D234).
    assert_eq!(values.len(), sine.params().len() - 1, "every parameter");
    let same = block_peak(&mut p, &[], false);
    assert!((same - full).abs() < 0.01, "its processor not told: {same}");
    // Sent by the host, they are heard: the gain and twice the level, 0.8 * 1.6 of a note.
    let loaded = block_peak(&mut p, &[(P_GAIN, 0.8), (P_LEVEL, 0.8)], false);
    let after = block_peak(&mut p, &[], false);
    assert!(
        (after - 0.8 * 1.6).abs() < 0.02,
        "the preset's values once sent: {loaded}, {after}"
    );
    p.stop();
}

/// D222: the editor's value reaches the processor in a block whose events change one
/// parameter at many points (the review's finding: they were counted as many parameters,
/// and the editor's value waited while they kept coming).
#[test]
fn the_editors_value_is_not_kept_waiting_by_a_busy_block() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 1024).unwrap();
    let mut p = sine.processor(1024);
    assert!(p.start());
    block_peak(&mut p, &[(P_NUDGE, 0.75)], false);
    sine.service();
    // More points of one parameter (the delay's mix, 0) than the instrument has
    // parameters, as automation of it gives a long block.
    let busy: Vec<(u32, f64)> = (0..sine.params().len() + 10)
        .map(|_| (P_MIX, 0.0))
        .collect();
    let peak = block_peak(&mut p, &busy, true);
    assert!(
        (peak - 0.75).abs() < 0.01,
        "the nudged gain in a busy block: {peak}"
    );
    p.stop();
}

/// D222: a controller answering the values the host gives it with `performEdit` of the same
/// values makes no edit for the project (the review's finding: it was taken as one).
#[test]
fn a_controllers_answers_to_the_host_are_not_edits() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.flush_params(&[(P_ANSWER, 1.0)]);
    sine.activate(48_000.0, 256).unwrap();
    sine.take_edits();
    let mut p = sine.processor(256);
    assert!(p.start());
    block_peak(&mut p, &[(P_GAIN, 0.3)], false);
    sine.service();
    assert_eq!(sine.take_edits(), [], "the controller's answer");
    p.stop();
}

/// D222: what the editor did while the instrument was set up (a controller answering the
/// values it was set with) is not sent to the processor once it is active, as its edits
/// then are not kept (the review's finding).
#[test]
fn the_editors_values_from_before_activation_are_not_sent() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.flush_params(&[(P_NUDGE, 0.75)]);
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let peak = block_peak(&mut p, &[], true);
    assert!((peak - 0.5).abs() < 0.01, "the gain as set: {peak}");
    p.stop();
}

/// D234: what the instrument keeps besides its parameters (its sample, its processor's) is
/// in its state: loaded with a preset's values (`kParamValuesChanged`), alone (its
/// controller's `IComponentHandler2::setDirty`) or without a word; values its controller
/// changes without a word; the host's values known as the host's; the state's sample heard
/// in a new instance it is loaded into.
#[test]
fn a_samples_state_changes_with_a_word_or_without_one() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let of = |i: &Instance, id: u32| i.param_values().iter().find(|v| v.0 == id).map(|v| v.1);
    let full = block_peak(&mut p, &[], true);
    assert!((full - 0.5).abs() < 0.01, "{full}");
    sine.service();
    assert!(!sine.take_values_changed());
    let before = sine.save_state().unwrap();
    // A sample loaded alone: `setDirty`, its values as they were; heard at its level.
    block_peak(&mut p, &[(P_LOAD_SAMPLE, 0.5)], false);
    sine.service();
    assert!(sine.take_values_changed(), "setDirty");
    assert_eq!(of(&sine, P_GAIN), Some(0.5));
    let half = block_peak(&mut p, &[], false);
    assert!((half - 0.25).abs() < 0.01, "the sample's level: {half}");
    let with = sine.save_state().unwrap();
    assert_ne!(with, before, "in its state");
    // With a preset's values: `kParamValuesChanged`.
    block_peak(&mut p, &[(P_SAMPLE_PRESET, 0.8)], false);
    sine.service();
    assert!(sine.take_values_changed(), "its values changed");
    assert_eq!(of(&sine, P_GAIN), Some(0.8));
    assert_eq!(of(&sine, P_LEVEL), Some(0.8));
    // Without a word: values, or a sample.
    block_peak(&mut p, &[(P_CHANGE_QUIETLY, 0.6)], false);
    sine.service();
    assert!(!sine.take_values_changed(), "no word");
    assert_eq!(of(&sine, P_GAIN), Some(0.6));
    block_peak(&mut p, &[(P_SAMPLE_QUIETLY, 0.25)], false);
    sine.service();
    assert!(!sine.take_values_changed(), "no word");
    let given = sine.take_host_given();
    assert!(
        given.contains(&P_SAMPLE_QUIETLY) && !given.contains(&P_GAIN),
        "the host's values: {given:?}"
    );
    assert_eq!(sine.host_value(P_SAMPLE_QUIETLY), Some(0.25));
    assert_eq!(sine.host_value(P_GAIN), None);
    p.stop();
    // The state with the sample (0.5), in a new instance: heard there.
    let mut other = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    other.load_state(&with).unwrap();
    other.activate(48_000.0, 256).unwrap();
    let mut q = other.processor(256);
    assert!(q.start());
    let heard = block_peak(&mut q, &[], true);
    assert!((heard - 0.25).abs() < 0.01, "the state's sample: {heard}");
    // A state without one, given while it runs, takes it back.
    other.load_state(&before).unwrap();
    let back = block_peak(&mut q, &[], false);
    assert!((back - 0.5).abs() < 0.01, "no sample: {back}");
    q.stop();
}

/// D234: what the processor reports itself (`outputParameterChanges`) reaches the
/// controller on the main thread, the latest of each: its meter (read-only) is only shown;
/// its own gain is a change of the plugin's own (read again, as its word would be).
#[test]
fn the_processors_own_values_reach_the_controller() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let shown =
        |i: &Instance, id: u32| i.params().iter().find(|x| x.id == id).and_then(|x| x.value);
    let full = block_peak(&mut p, &[], true);
    assert_eq!(shown(&sine, P_METER), Some(0.0), "not yet told");
    sine.service();
    let meter = shown(&sine, P_METER).unwrap();
    assert!(
        (meter - f64::from(full)).abs() < 1e-6,
        "its meter: {meter}, {full}"
    );
    assert!(!sine.take_values_changed(), "a meter is only shown");
    // Its own gain (0.3, a plain 0.6): reported, shown, and a change of its own.
    let set = block_peak(&mut p, &[(P_PROCESSOR_GAIN, 0.3)], false);
    sine.service();
    assert_eq!(shown(&sine, P_GAIN), Some(0.3), "{set}");
    assert!(sine.take_values_changed(), "its own change");
    let after = block_peak(&mut p, &[], false);
    assert!((after - 0.3).abs() < 0.01, "its gain: {after}");
    // It reports its gain every block: the same value again is no change, nor is the host's
    // value it reports back (the review's finding: its word kept coming).
    sine.service();
    sine.take_values_changed();
    block_peak(&mut p, &[], false);
    sine.service();
    assert!(!sine.take_values_changed(), "reported again: no change");
    block_peak(&mut p, &[(P_GAIN, 0.4)], false);
    block_peak(&mut p, &[], false);
    sine.service();
    assert_eq!(shown(&sine, P_GAIN), Some(0.4));
    assert!(
        !sine.take_values_changed(),
        "the host's value reported back"
    );
    p.stop();
}

/// D234: what polling the instrument costs, with its more than eight hundred parameters: a
/// poll (every value read from its controller and compared) and a save of its state, on the
/// helper's main thread. Printed (`--no-capture`); held under 10 ms each in any build, so that
/// a poll every 250 ms (or a hundred times its cost) stays a small part of that thread.
#[test]
fn polling_the_instrument_of_eight_hundred_parameters_is_cheap() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 256).unwrap();
    let n = sine.params().len();
    assert!(n > 800, "{n} parameters");
    let t0 = std::time::Instant::now();
    let values = sine.param_values();
    let mut w = daw_helper::watch::Watch::new(t0, &values, None);
    let mut polls = Vec::new();
    let mut saves = Vec::new();
    for k in 0..40u64 {
        let t = std::time::Instant::now();
        let v = sine.param_values();
        let read = t.elapsed();
        w.poll(
            t0 + std::time::Duration::from_secs(k + 1),
            &v,
            read,
            &|id| sine.host_value(id),
        );
        polls.push(t.elapsed());
        let t = std::time::Instant::now();
        let s = sine.save_state().unwrap();
        saves.push((t.elapsed(), s.len()));
    }
    polls.sort();
    saves.sort();
    let (median, max) = (polls[polls.len() / 2], polls[polls.len() - 1]);
    eprintln!(
        "polling {n} parameters: median {median:?}, max {max:?} ({:?} a parameter); saving its state ({} bytes): median {:?}",
        median / n as u32,
        saves[0].1,
        saves[saves.len() / 2].0
    );
    assert!(
        median < std::time::Duration::from_millis(10),
        "a poll: {median:?}"
    );
    assert!(saves[saves.len() / 2].0 < std::time::Duration::from_millis(10));
}

/// The quiet-sample fixture must separate its host trigger from its state-only change,
/// even when the helper cannot poll until more than 1.5 audio seconds have passed.
#[test]
fn a_quiet_sample_waits_for_its_baseline_before_changing_under_a_slow_host() {
    let m = Arc::new(Module::load(&plugin_path()).unwrap());
    let mut sine = m.instantiate(&cid_hex(&SINE_CID)).unwrap();
    sine.activate(48_000.0, 256).unwrap();
    let mut p = sine.processor(256);
    assert!(p.start());
    let now = std::time::Instant::now();
    let cost = std::time::Duration::from_micros(10);
    let initial = sine.save_state().unwrap();
    let mut watch =
        daw_helper::watch::Watch::new(now, &sine.param_values(), Some((&initial, cost)));
    block_peak(&mut p, &[(P_SAMPLE_QUIETLY, 0.25)], false);
    sine.service();
    watch.note_host(&sine.take_host_given(), &|id| sine.host_value(id));
    // Three audio seconds with no state poll: the old fixture changed after 1.5.
    for _ in 0..600 {
        block_peak(&mut p, &[], false);
    }
    let sample = |bytes: &[u8]| {
        bytes
            .windows(12)
            .find(|w| w[..4] == 0xFFFF_FF01u32.to_le_bytes())
            .map_or(1.0, |w| f64::from_le_bytes(w[4..12].try_into().unwrap()))
    };
    let baseline = sine.save_state().unwrap();
    assert_eq!(
        sample(&baseline),
        1.0,
        "the quiet change raced its first state poll"
    );
    let values = sine.param_values();
    let host = |id| sine.host_value(id);
    assert!(
        !watch.poll_state(now, &values, &baseline, cost, &host),
        "the trigger is the host's"
    );
    // The baseline has been saved; the silent sample can now change independently.
    for _ in 0..300 {
        block_peak(&mut p, &[], false);
    }
    let changed = sine.save_state().unwrap();
    assert_eq!(sample(&changed), 0.25);
    assert!(
        !sine.take_values_changed(),
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
