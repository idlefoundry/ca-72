//! The effects' parameters and pins (decisions.md R-FX-PARAMS) with DAW-v3's test plug-ins in
//! both formats: their parameters listed again with their values, units, steps and their own
//! value text; pinned and unpinned; the base following the plug-in's window's edits; the
//! synth's modulation arriving at its offsets with its values (CLAP's own modulation, VST3's
//! base plus offset to the processor alone); the base given back after a restart and a
//! restore; no snapshot while modulated; a missing or crashed effect's pins kept, inert; and
//! the pins and their assignments through the saved chain.
//!
//! The test plug-ins compute `y[n] = gain * x[n]` at their defaults, each parameter change
//! taking effect at its sample, so a constant input shows the gain at every sample.

use ca72_fx::chain::{ChainOptions, SavedChain};
use ca72_fx::{Chain, Health, Kind, MAX_PINS, PinMods, Runner, catalog};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const P_GAIN: u32 = 1;
const P_MIX: u32 = 2;
const P_DELAY: u32 = 3;
/// The CLAP plug-in's hidden `nudge gain`: a stand-in for a knob turned in its window (an
/// output value event). (The VST3 effect has none, its instrument's stand-ins being the
/// instrument's: its `performEdit` reaches the host as the same notice, DAW-v3's D222.)
const CLAP_NUDGE: u32 = 104;
const P_CRASH: u32 = 100;
const CLAP_ID: &str = "org.daw-v3.test-plugin";
const VST3_ID: &str = "44415733546573744761696E50726F63";
const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;

fn helper() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ca72-fx-helper"))
}

fn built(stem: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("test exe");
    let dir = exe.parent().expect("deps dir");
    let names = [
        format!("lib{stem}.so"),
        format!("lib{stem}.dylib"),
        format!("{stem}.dll"),
    ];
    [dir.to_path_buf(), dir.join("deps")]
        .iter()
        .flat_map(|d| names.iter().map(move |n| d.join(n)))
        .filter_map(|p| Some((std::fs::metadata(&p).ok()?.modified().ok()?, p)))
        .max_by_key(|(t, _)| *t)
        .map(|(_, p)| p)
        .unwrap_or_else(|| panic!("{stem} was not built beside the test"))
}

fn found(path: &Path, id: &str, format: &str) -> catalog::Found {
    catalog::Found {
        path: path.to_path_buf(),
        kind: Kind::Effect,
        desc: daw_ipc::protocol::PluginDesc {
            id: id.into(),
            name: format!("test {format}"),
            format: format.into(),
            ..Default::default()
        },
    }
}

fn plugins() -> [catalog::Found; 2] {
    [
        found(&built("daw_test_plugin"), CLAP_ID, "clap"),
        found(&built("daw_test_vst3"), VST3_ID, "vst3"),
    ]
}

fn chain() -> (Chain, Runner) {
    let mut o = ChainOptions::new(helper());
    o.sandbox.realtime = false;
    o.sandbox.hang_timeout = Duration::from_millis(5000);
    o.sandbox.snapshot_debounce = Duration::from_millis(50);
    let (c, mut r) = Chain::new(o);
    r.prepare(RATE, BLOCK);
    (c, r)
}

/// A synth's assignments, as a test keeps them.
#[derive(Default)]
struct Mods {
    by: Mutex<[Vec<(String, f32)>; MAX_PINS]>,
    rev: std::sync::atomic::AtomicU64,
}

impl PinMods for Mods {
    fn get(&self, target: usize) -> Vec<(String, f32)> {
        self.by.lock().expect("the mods")[target].clone()
    }
    fn set(&self, target: usize, mods: &[(String, f32)]) {
        self.by.lock().expect("the mods")[target] = mods.to_vec();
        self.rev.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
    fn revision(&self) -> u64 {
        self.rev.load(std::sync::atomic::Ordering::Acquire)
    }
}

/// Plays `blocks` blocks of a constant 0.5 through the chain, the pins in `moved` at offset
/// `offset(frame)` at each step of 32 samples; the left output.
fn play(
    r: &mut Runner,
    blocks: usize,
    moved: u32,
    offset: impl Fn(usize) -> f32,
    from: usize,
) -> Vec<f32> {
    let mut out = Vec::new();
    for b in 0..blocks {
        let (mut l, mut rr) = (vec![0.5f32; BLOCK], vec![0.5f32; BLOCK]);
        let at: Vec<u32> = (0..BLOCK as u32 / 32).map(|k| k * 32).collect();
        let offsets: Vec<[f32; MAX_PINS]> = at
            .iter()
            .map(|a| [offset(from + b * BLOCK + *a as usize); MAX_PINS])
            .collect();
        r.process_modulated(&mut l, &mut rr, None, moved, &at, &offsets);
        out.extend(l);
    }
    out
}

/// Waits until `f` holds, for at most two seconds (the housekeeping runs every 100 ms).
fn eventually(mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn parameters_are_listed_with_units_steps_and_their_own_text() {
    for p in plugins() {
        let (c, _r) = chain();
        c.add(&p).unwrap();
        let params = c.list_params(0).unwrap();
        let gain = params.iter().find(|x| x.id == P_GAIN).unwrap();
        assert_eq!(gain.name, "Gain", "{}", p.desc.format);
        assert!(ca72_fx::pinnable(gain));
        assert!(gain.value.is_some());
        let delay = params.iter().find(|x| x.id == P_DELAY).unwrap();
        assert_eq!(
            delay.steps, 4799,
            "{}: Delay steps 1 to 4800",
            p.desc.format
        );
        let crash = params.iter().find(|x| x.id == P_CRASH).unwrap();
        assert!(
            !ca72_fx::pinnable(crash),
            "a hidden parameter is not offered"
        );
        let at = |v: f64| if p.desc.format == "vst3" { v / 2.0 } else { v };
        let delay_at = |v: f64| {
            if p.desc.format == "vst3" {
                (v - 1.0) / 4799.0
            } else {
                v
            }
        };
        let text = c
            .param_text(
                0,
                &[(P_GAIN, at(1.5)), (P_DELAY, delay_at(480.0)), (9999, 0.0)],
            )
            .unwrap();
        assert_eq!(text[0].as_deref(), Some("1.500"), "{}", p.desc.format);
        // (VST3 gives the number and its units apart: put together.)
        let delay = if p.desc.format == "vst3" {
            "480 frames"
        } else {
            "480 samples"
        };
        assert_eq!(text[1].as_deref(), Some(delay), "{}", p.desc.format);
        assert_eq!(text[2], None, "no such parameter: no text");
        if p.desc.format == "vst3" {
            assert_eq!(
                params.iter().find(|x| x.id == P_DELAY).unwrap().units,
                "frames"
            );
        }
    }
}

#[test]
fn pins_are_made_refused_and_let_go_of() {
    for p in plugins() {
        let (c, _r) = chain();
        let mods = Arc::new(Mods::default());
        c.set_pin_mods(mods.clone());
        c.add(&p).unwrap();
        assert_eq!(c.pin(0, P_GAIN).unwrap(), 0);
        assert_eq!(c.pin(0, P_MIX).unwrap(), 1);
        assert!(c.pin(0, P_GAIN).unwrap_err().contains("pinned already"));
        assert!(c.pin(0, P_CRASH).unwrap_err().contains("cannot be pinned"));
        assert!(c.pin(1, P_GAIN).is_err(), "no second effect");
        let pins = c.pins();
        assert_eq!(pins.len(), 2);
        assert_eq!((pins[0].name.as_str(), pins[0].live), ("Gain", true));
        assert!((pins[0].value - 0.5).abs() < 1e-9, "gain 1 of 0..2");
        // What moved it is let go of with it.
        mods.set(1, &[("lfo1".into(), 0.3)]);
        c.unpin(1).unwrap();
        assert!(mods.get(1).is_empty());
        assert_eq!(c.pins().len(), 1);
        // A slot taken out takes its pins.
        mods.set(0, &[("macro2".into(), 0.4)]);
        c.remove(0).unwrap();
        assert!(c.pins().is_empty());
        assert!(mods.get(0).is_empty());
    }
}

#[test]
fn the_plugins_window_edits_become_the_pins_base() {
    for p in plugins() {
        let (c, mut r) = chain();
        c.add(&p).unwrap();
        c.pin(0, P_GAIN).unwrap();
        if p.desc.format == "clap" {
            c.set_param(0, CLAP_NUDGE, 0.75).unwrap();
            play(&mut r, 2, 0, |_| 0.0, 0);
            assert!(
                eventually(|| c.pins()[0].base.is_some_and(|b| (b - 0.75).abs() < 1e-9)),
                "{:?}",
                c.pins()[0]
            );
            // Its base now, which modulation adds to and a restart gives back.
            let out = play(&mut r, 2, 1, |_| 0.125, 0);
            assert!(
                (out[BLOCK] - 0.5).abs() < 1e-6,
                "0.5 × (0.75 + 0.25): {}",
                out[BLOCK]
            );
            c.kill(0).unwrap();
            let out = play(&mut r, 2, 0, |_| 0.0, 0);
            assert!(
                (out[BLOCK] - 0.375).abs() < 1e-6,
                "after a restart: {}",
                out[BLOCK]
            );
        }
        // The drawer's knob the other way: the plug-in's value, and its base.
        c.set_pin_value(0, 0.25).unwrap();
        let out = play(&mut r, 2, 0, |_| 0.0, 0);
        assert!(
            (out[BLOCK + 10] - 0.25).abs() < 1e-6,
            "{}: gain 0.5",
            p.desc.format
        );
        let want = if p.desc.format == "vst3" { 0.25 } else { 0.5 };
        assert!(
            eventually(|| {
                let listed = c.list_params(0).unwrap();
                let v = listed
                    .iter()
                    .find(|x| x.id == P_GAIN)
                    .unwrap()
                    .value
                    .unwrap();
                (v - want).abs() < 1e-6
            }),
            "{}: its window follows",
            p.desc.format
        );
    }
}

#[test]
fn modulation_arrives_at_its_offsets_and_leaves_the_window_alone() {
    for p in plugins() {
        let (c, mut r) = chain();
        c.add(&p).unwrap();
        c.pin(0, P_GAIN).unwrap();
        // A ramp: each step of 32 samples 0.01 of the travel further (gain 1 + 2 × offset).
        let offset = |f: usize| (f / 32) as f32 * 0.01;
        let out = play(&mut r, 3, 1, offset, 0);
        for (i, y) in out.iter().enumerate() {
            let want = 0.5 * (1.0 + 2.0 * offset(i).min(0.5));
            assert!(
                (y - want).abs() < 1e-5,
                "{} sample {i}: {y} not {want}",
                p.desc.format
            );
        }
        // The base is the plug-in's own value still: its window shows it, not the LFO.
        std::thread::sleep(Duration::from_millis(150));
        let listed = c.list_params(0).unwrap();
        let v = listed
            .iter()
            .find(|x| x.id == P_GAIN)
            .unwrap()
            .value
            .unwrap();
        let base = if p.desc.format == "vst3" { 0.5 } else { 1.0 };
        assert!(
            (v - base).abs() < 1e-9,
            "{}: its window at {v}",
            p.desc.format
        );
        // Nothing moves it: back to its base, at once, and nothing sent after.
        let out = play(&mut r, 2, 0, offset, 3 * BLOCK);
        assert!(
            out.iter().all(|y| (y - 0.5).abs() < 1e-6),
            "{}",
            p.desc.format
        );
    }
}

#[test]
fn a_large_block_is_thinned_to_32_points() {
    let (c, mut r) = chain();
    c.add(&plugins()[0]).unwrap();
    c.pin(0, P_GAIN).unwrap();
    let n = 4096;
    let (mut l, mut rr) = (vec![0.5f32; n], vec![0.5f32; n]);
    let at: Vec<u32> = (0..n as u32 / 32).map(|k| k * 32).collect();
    let offsets: Vec<[f32; MAX_PINS]> = at
        .iter()
        .map(|a| [(*a / 32) as f32 * 0.001; MAX_PINS])
        .collect();
    r.process_modulated(&mut l, &mut rr, None, 1, &at, &offsets);
    // Every 128 samples: the value holds between points four steps apart.
    for (i, y) in l.iter().enumerate() {
        let k = (i / 128) * 4;
        let want = 0.5 * (1.0 + 2.0 * k as f32 * 0.001);
        assert!((y - want).abs() < 1e-5, "sample {i}: {y} not {want}");
    }
}

#[test]
fn the_vst3_base_comes_back_after_a_restart_and_a_restore() {
    let p = &plugins()[1];
    let (c, mut r) = chain();
    c.add(p).unwrap();
    c.pin(0, P_GAIN).unwrap();
    c.set_pin_value(0, 0.25).unwrap();
    // Held up by half its travel: its processor's own state holds gain 1.5, its base 0.5.
    let out = play(&mut r, 4, 1, |_| 0.5, 0);
    assert!(
        (out[3 * BLOCK] - 0.75).abs() < 1e-6,
        "modulated: {}",
        out[3 * BLOCK]
    );
    c.save_now().unwrap();
    let saved = c.saved();
    assert_eq!(saved.slots[0].pins[0].base, Some(0.25));
    // Its helper killed and started again: no longer moved, it plays at its base.
    c.kill(0).unwrap();
    let out = play(&mut r, 4, 0, |_| 0.0, 0);
    assert_eq!(c.status()[0].health, Health::Running);
    assert!(
        (out[3 * BLOCK] - 0.25).abs() < 1e-6,
        "its base after the restart, not the state's instant: {}",
        out[3 * BLOCK]
    );
    // A session restored from the saved chain: the same.
    let (c2, mut r2) = chain();
    assert!(c2.restore(&saved).is_empty());
    let out = play(&mut r2, 2, 0, |_| 0.0, 0);
    assert!((out[BLOCK] - 0.25).abs() < 1e-6, "restored: {}", out[BLOCK]);
    assert_eq!(c2.pins()[0].base, Some(0.25));
}

#[test]
fn modulation_takes_no_snapshot_and_writes_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("effects.json");
    for p in plugins() {
        let mut o = ChainOptions::new(helper());
        o.sandbox.realtime = false;
        o.sandbox.snapshot_debounce = Duration::from_millis(50);
        o.file = Some(file.clone());
        let (c, mut r) = Chain::new(o);
        r.prepare(RATE, BLOCK);
        c.add(&p).unwrap();
        c.pin(0, P_GAIN).unwrap();
        play(&mut r, 4, 0, |_| 0.0, 0);
        std::thread::sleep(Duration::from_millis(1500));
        let rev = c.sandbox().state_rev();
        let written = std::fs::metadata(&file).unwrap().modified().unwrap();
        // A second and a half of an LFO moving it, block after block in time.
        let start = Instant::now();
        let mut b = 0;
        while start.elapsed() < Duration::from_millis(1500) {
            play(
                &mut r,
                1,
                1,
                |f| ((f as f32) * 0.001).sin() * 0.3,
                b * BLOCK,
            );
            b += 1;
            std::thread::sleep(Duration::from_millis(5));
        }
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            c.sandbox().state_rev(),
            rev,
            "{}: no snapshot",
            p.desc.format
        );
        assert_eq!(
            std::fs::metadata(&file).unwrap().modified().unwrap(),
            written,
            "{}: effects.json not written",
            p.desc.format
        );
        c.shutdown().unwrap();
    }
}

#[test]
fn a_missing_or_crashed_effect_keeps_its_pins_inert() {
    let (c, mut r) = chain();
    let mods = Arc::new(Mods::default());
    c.set_pin_mods(mods.clone());
    c.add(&plugins()[0]).unwrap();
    c.pin(0, P_GAIN).unwrap();
    mods.set(0, &[("lfo1".into(), 0.3)]);
    // Crashed and restarting: inert meanwhile, kept, and back once it runs.
    c.kill(0).unwrap();
    play(&mut r, 2, 1, |_| 0.1, 0);
    assert!(eventually(|| c.pins()[0].live), "back when it runs");
    assert_eq!(mods.get(0), [("lfo1".to_owned(), 0.3)]);
    // Missing (not installed here): the pin and what moves it kept, dimmed and inert.
    let mut saved = c.saved();
    saved.slots[0].path = PathBuf::from("/nowhere/gone.clap");
    let (c2, mut r2) = chain();
    let mods2 = Arc::new(Mods::default());
    c2.set_pin_mods(mods2.clone());
    c2.restore(&saved);
    assert_eq!(c2.status()[0].health, Health::Missing);
    let pins = c2.pins();
    assert_eq!(pins.len(), 1);
    assert_eq!(
        (pins[0].name.as_str(), pins[0].live, pins[0].missing),
        ("Gain", false, false)
    );
    assert_eq!(mods2.get(0), [("lfo1".to_owned(), 0.3)]);
    let out = play(&mut r2, 2, 1, |_| 0.4, 0);
    assert!(out.iter().all(|y| *y == 0.5), "dry, nothing sent");
    // Saved again, it is all still there.
    let again = c2.saved();
    assert_eq!(again.slots[0].pins[0].mods, [("lfo1".to_owned(), 0.3)]);
    assert_eq!(again.slots[0].pins[0].target, Some(0));
}

#[test]
fn pins_and_what_moves_them_go_through_the_saved_chain() {
    for p in plugins() {
        let (c, _r) = chain();
        let mods = Arc::new(Mods::default());
        c.set_pin_mods(mods.clone());
        c.add(&p).unwrap();
        c.pin(0, P_GAIN).unwrap();
        c.pin(0, P_MIX).unwrap();
        c.unpin(0).unwrap();
        // Gain again, now on the second target: targets stay as they were given.
        assert_eq!(c.pin(0, P_GAIN).unwrap(), 0);
        mods.set(0, &[("lfo2".into(), -0.5), ("macro1".into(), 0.25)]);
        mods.set(1, &[("lfo1".into(), 1.0)]);
        c.save_now().unwrap();
        let json = serde_json::to_string(&c.saved()).unwrap();
        let saved: SavedChain = serde_json::from_str(&json).unwrap();
        let pins = &saved.slots[0].pins;
        assert_eq!(pins.len(), 2);
        assert_eq!((pins[0].id, pins[0].target), (P_MIX, Some(1)));
        assert_eq!((pins[1].id, pins[1].target), (P_GAIN, Some(0)));
        assert_eq!(pins[1].mods.len(), 2);
        // Restored, before and after the synth's side is given.
        let (c2, _r2) = chain();
        assert!(c2.restore(&saved).is_empty());
        let mods2 = Arc::new(Mods::default());
        c2.set_pin_mods(mods2.clone());
        assert_eq!(mods2.get(0), mods.get(0), "{}", p.desc.format);
        assert_eq!(mods2.get(1), mods.get(1));
        let targets: Vec<(usize, u32)> = c2.pins().iter().map(|x| (x.target, x.id)).collect();
        assert_eq!(targets, [(0, P_GAIN), (1, P_MIX)]);
        // A chain saved before pins has none, and reads as before.
        let old =
            r#"{"format":1,"slots":[{"format":"clap","path":"/x.clap","plugin":"a","name":"A"}]}"#;
        let old: SavedChain = serde_json::from_str(old).unwrap();
        assert!(old.slots[0].pins.is_empty());
        assert!(!serde_json::to_string(&old).unwrap().contains("pins"));
    }
}

#[test]
fn editor_touches_are_recent_unique_and_follow_the_effect() {
    let (c, mut r) = chain();
    c.add(&plugins()[0]).unwrap();
    assert!(c.touched(0).is_empty());
    c.set_param(0, P_MIX, 0.7).unwrap();
    play(&mut r, 2, 0, |_| 0.0, 0);
    assert!(
        c.touched(0).is_empty(),
        "a host value is not an editor touch"
    );
    c.set_param(0, CLAP_NUDGE, 0.7).unwrap();
    play(&mut r, 2, 0, |_| 0.0, 0);
    assert!(eventually(|| c.touched(0) == [P_GAIN]));
    // The existing test effect's other editor stand-in chooses its note names.
    c.set_param(0, 103, 1.0).unwrap();
    play(&mut r, 2, 0, |_| 0.0, 0);
    assert!(eventually(|| c.touched(0) == [102, P_GAIN]));
    c.set_param(0, CLAP_NUDGE, 0.8).unwrap();
    play(&mut r, 2, 0, |_| 0.0, 0);
    assert!(eventually(|| c.touched(0) == [P_GAIN, 102]));
    c.add(&plugins()[0]).unwrap();
    c.move_to(0, 1).unwrap();
    assert!(c.touched(0).is_empty());
    assert_eq!(c.touched(1), [P_GAIN, 102]);
    let saved = c.saved();
    let (restored, _runner) = chain();
    assert!(restored.restore(&saved).is_empty());
    assert!(restored.touched(1).is_empty(), "touch hints are not saved");
}

#[test]
fn a_repin_keeps_the_target_and_assignments_and_releases_the_old_modulation() {
    for p in plugins() {
        let (c, mut r) = chain();
        let mods = Arc::new(Mods::default());
        c.set_pin_mods(mods.clone());
        c.add(&p).unwrap();
        let target = c.pin(0, P_GAIN).unwrap();
        mods.set(target, &[("lfo1".into(), 0.3), ("macro2".into(), 0.4)]);
        c.set_param(0, P_MIX, 0.6).unwrap();
        play(&mut r, 3, 0, |_| 0.0, 0);
        assert!(eventually(|| c
            .list_params(0)
            .unwrap()
            .iter()
            .find(|p| p.id == P_MIX)
            .unwrap()
            .value
            .is_some_and(|v| (v - 0.6).abs() < 1e-6)));
        let out = play(&mut r, 2, 1, |_| 0.25, 0);
        assert!((out[BLOCK] - 1.2).abs() < 1e-6, "gain is modulated");
        c.repin(target, P_MIX).unwrap();
        let out = play(&mut r, 2, 1, |_| 0.0, 0);
        assert!(
            (out[BLOCK] - 0.8).abs() < 1e-6,
            "{}: old gain returned to base: {}",
            p.desc.format,
            out[BLOCK]
        );
        let pins = c.pins();
        assert_eq!((pins[0].target, pins[0].id), (target, P_MIX));
        assert!((pins[0].base.unwrap() - 0.6).abs() < 1e-6);
        assert_eq!(mods.get(target).len(), 2);
        let before = c.saved();
        c.repin(target, P_MIX).unwrap();
        assert_eq!(
            c.saved().slots[0].pins,
            before.slots[0].pins,
            "same parameter does nothing"
        );
        c.pin(0, P_DELAY).unwrap();
        assert!(c.repin(target, P_DELAY).is_err(), "already pinned");
        assert!(c.repin(target, P_CRASH).is_err(), "hidden");
        c.save_now().unwrap();
        let saved = c.saved();
        assert_eq!(saved.slots[0].pins[0].mods, mods.get(target));
        let (restored, _runner) = chain();
        let mods2 = Arc::new(Mods::default());
        restored.set_pin_mods(mods2.clone());
        assert!(restored.restore(&saved).is_empty());
        assert_eq!(restored.pins()[0].id, P_MIX);
        assert_eq!(mods2.get(target), mods.get(target));
    }
}
