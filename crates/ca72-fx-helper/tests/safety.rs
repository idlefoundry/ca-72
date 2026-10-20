//! An effect can never make the synth dead, nor keep it from starting (decisions.md
//! R-FX-SAFETY), with DAW-v3's test plug-ins and their injected faults (third_party/daw-v3
//! PATCHES.md, 4): plug-in IDs that crash or hang as they are made, saved states that crash or
//! hang the plug-in as it is given them or crash it as soon as it plays, and files that crash or
//! hang their helper as they are listed.
//!
//! - A chain restored at start is in place at once and plays dry; each effect joins when it
//!   has loaded, one hung at load waits for nothing but its own slot, and is turned off when
//!   its time is up; one that crashes at load is turned off at once.
//! - Turned off, an effect stays off at the next start (not loaded) until tried again, its
//!   state kept, its place and bypass too.
//! - Saved settings that crash an effect are not given to it again; it can be loaded without
//!   them, and they are kept, set aside.
//! - A safe start puts the chain in place without loading any of it, and saves nothing
//!   differently; the effects turned back on load.
//! - A plug-in that crashes or hangs its helper as it is listed is remembered, not listed
//!   again until it changes or the list is asked to try again.

use ca72_fx::chain::{ChainOptions, SavedChain, SavedSlot};
use ca72_fx::{Chain, Fault, Health, Kind, Restore, Runner, catalog};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const CLAP_ID: &str = "org.daw-v3.test-plugin";
const CLAP_CRASH_ID: &str = "org.daw-v3.test-plugin.crash-at-load";
const CLAP_HANG_ID: &str = "org.daw-v3.test-plugin.hang-at-load";
const VST3_ID: &str = "44415733546573744761696E50726F63";
/// The VST3 test plug-in's processor classes that crash and hang as they are made ("CRSH",
/// "HANG").
const VST3_CRASH_ID: &str = "44415733546573744761696E43525348";
const VST3_HANG_ID: &str = "44415733546573744761696E48414E47";
/// The test plug-ins' state pair that injects a fault (1 crash as it is loaded, 2 hang then,
/// 3 crash at the first block after).
const FAULT_ID: u32 = 0xFFFF_FF02;
const P_GAIN: u32 = 1;
const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;
/// The load timeout the tests give a plug-in (the application's is 30 s).
const LOAD_TIMEOUT: Duration = Duration::from_secs(2);

fn helper() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ca72-fx-helper"))
}

/// A test plug-in cargo built beside this test (the newest copy).
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

fn clap_plugin() -> PathBuf {
    built("daw_test_plugin")
}

fn vst3_plugin() -> PathBuf {
    built("daw_test_vst3")
}

fn options(file: Option<PathBuf>) -> ChainOptions {
    let mut o = ChainOptions::new(helper());
    o.sandbox.hang_timeout = Duration::from_millis(400);
    o.sandbox.snapshot_debounce = Duration::from_millis(50);
    o.sandbox.load_timeout = LOAD_TIMEOUT;
    o.file = file;
    o
}

fn chain(file: Option<PathBuf>) -> (Chain, Runner) {
    let (c, mut r) = Chain::new(options(file));
    r.prepare(RATE, BLOCK);
    (c, r)
}

fn slot(path: &Path, format: &str, plugin: &str, name: &str) -> SavedSlot {
    SavedSlot {
        format: format.into(),
        path: path.to_path_buf(),
        plugin: plugin.into(),
        name: name.into(),
        vendor: "DAW-v3".into(),
        bypass: false,
        state: None,
        disabled: None,
        set_aside: None,
    }
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

/// The test plug-in's saved state at gain 0.5 (both formats keep the same state), in hex,
/// and with a fault pair added.
fn states(format: &str) -> (String, impl Fn(u32) -> String) {
    let (c, mut r) = chain(None);
    let (path, id) = match format {
        "clap" => (clap_plugin(), CLAP_ID),
        _ => (vst3_plugin(), VST3_ID),
    };
    c.add(&found(&path, id, format)).expect("the test effect");
    let params = c.params(0);
    let p = params
        .iter()
        .find(|p| p.id == P_GAIN)
        .expect("its gain parameter");
    let v = match (p.plain.first(), p.plain.last()) {
        (Some(lo), Some(hi)) if hi > lo => p.min + (p.max - p.min) * (0.5 - lo) / (hi - lo),
        _ => 0.5,
    };
    c.set_param(0, P_GAIN, v).expect("its gain set");
    let (mut l, mut rr) = (vec![0.0; BLOCK], vec![0.0; BLOCK]);
    r.process(&mut l, &mut rr, None);
    c.save_now().expect("saved");
    let hex = c.saved().slots[0].state.clone().expect("its state");
    c.shutdown().expect("closed");
    let base = hex.clone();
    let vst3 = format == "vst3";
    (hex, move |fault: u32| {
        let mut b = daw_ipc::protocol::unhex(&base).expect("a state");
        let pair = [
            FAULT_ID.to_le_bytes().as_slice(),
            &f64::from(fault).to_le_bytes(),
        ]
        .concat();
        if vst3 {
            // The helper's VST3 state: "VST3", then the processor's and the controller's
            // parts, each after its length: the pair goes at the processor's part's end.
            let len = u64::from_le_bytes(b[4..12].try_into().expect("a length")) as usize;
            b.splice(12 + len..12 + len, pair);
            b[4..12].copy_from_slice(&((len + 12) as u64).to_le_bytes());
        } else {
            b.extend_from_slice(&pair);
        }
        daw_ipc::protocol::hex(&b)
    })
}

fn write(file: &Path, slots: Vec<SavedSlot>) {
    let chain = SavedChain { format: 1, slots };
    let bytes = serde_json::to_vec_pretty(&chain).expect("the chain as JSON");
    std::fs::write(file, bytes).expect("the chain written");
}

/// Plays in real time, paced, each block's deadline 30 ms after its start, until `done` or
/// `limit`; each block: whether it came out at gain 0.5 (wet), and how long it took.
fn play_live(
    r: &mut Runner,
    limit: Duration,
    mut done: impl FnMut(usize) -> bool,
) -> Vec<(bool, Duration)> {
    let period = Duration::from_secs_f64(BLOCK as f64 / RATE);
    let start = Instant::now();
    let mut out = Vec::new();
    let mut k = 0;
    while start.elapsed() < limit && !done(k) {
        let input: Vec<f32> = (0..BLOCK)
            .map(|i| ((k * BLOCK + i) as f32 * 0.01).sin())
            .collect();
        let (mut l, mut rr) = (input.clone(), input.clone());
        let t = Instant::now();
        r.process(&mut l, &mut rr, Some(t + Duration::from_millis(30)));
        let took = t.elapsed();
        let wet =
            l.iter().zip(&input).all(|(y, x)| *y == x * 0.5) && input.iter().any(|x| *x != 0.0);
        out.push((wet, took));
        std::thread::sleep(period.saturating_sub(t.elapsed()));
        k += 1;
    }
    out
}

fn health(c: &Chain) -> Vec<Health> {
    c.status().iter().map(|s| s.health).collect()
}

/// Waits (at most `limit`) until `cond` holds of the chain; whether it does.
fn until(c: &Chain, limit: Duration, cond: impl Fn(&Chain) -> bool) -> bool {
    let end = Instant::now() + limit;
    while Instant::now() < end {
        if cond(c) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    cond(c)
}

/// Items 1 and 2: a chain saved with an effect that hangs as it loads, one that crashes as it
/// loads (each format), and one that works. Restored as the application restores it, it is in
/// place at once and the synth plays through it from the first block, dry, each block within
/// its deadline; the working effect joins within a moment, while the hung one is still loading;
/// the one that crashes is turned off at once, the hung one when its time is up, both saved so,
/// with their helpers gone. At the next start neither is loaded (no helper started for them),
/// they keep their places, bypass and states, and the working one plays; tried again, they
/// are turned off again (counted); once the plug-in is mended, tried again, it plays.
#[test]
fn effects_that_crash_or_hang_as_they_load_never_hold_the_start_and_stay_off() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("effects.json");
    let (good, _) = states("clap");
    let mut hang = slot(&clap_plugin(), "clap", CLAP_HANG_ID, "Hangs");
    hang.bypass = true;
    hang.state = Some(good.clone());
    let crash_clap = slot(&clap_plugin(), "clap", CLAP_CRASH_ID, "Crashes");
    let crash_vst3 = slot(&vst3_plugin(), "vst3", VST3_CRASH_ID, "Crashes too");
    let mut works = slot(&clap_plugin(), "clap", CLAP_ID, "Works");
    works.state = Some(good.clone());
    write(
        &file,
        vec![hang, crash_clap, crash_vst3.clone(), works.clone()],
    );

    let (c, mut r) = chain(Some(file.clone()));
    let t = Instant::now();
    assert!(c.restore_file_as(Restore::Background).is_empty());
    let took = t.elapsed();
    assert!(took < Duration::from_millis(500), "restored in {took:?}");
    assert_eq!(c.len(), 4, "in place at once");
    let mut joined = None;
    let mut crashed_at = None;
    let blocks = play_live(&mut r, Duration::from_secs(10), |k| {
        let h = health(&c);
        if joined.is_none() && h[3] == Health::Running {
            joined = Some((k, h[0]));
        }
        if crashed_at.is_none() && h[1] == Health::Disabled && h[2] == Health::Disabled {
            crashed_at = Some(t.elapsed());
        }
        h[0] == Health::Disabled && joined.is_some()
    });
    let (k, hung_then) = joined.expect("the working effect joined");
    assert_eq!(hung_then, Health::Loading, "while the hung one still loads");
    // The first block dry, unless the working effect had loaded by then: on a loaded machine
    // it can load before the test plays its first block (each block's health is read before
    // it plays), and the start not waiting for it is `took`'s and `hung_then`'s to show.
    assert!(!blocks[0].0 || k <= 1, "the first block dry ({k})");
    assert!(blocks[k + 2..].iter().all(|b| b.0), "wet once it joined");
    let worst = blocks.iter().map(|b| b.1).max().unwrap();
    assert!(
        worst < Duration::from_millis(40),
        "the worst block {worst:?}"
    );
    let crashed_at = crashed_at.expect("the crashing ones turned off");
    assert!(crashed_at < LOAD_TIMEOUT, "at once: {crashed_at:?}");
    let st = c.status();
    let whys: Vec<_> = st
        .iter()
        .map(|s| s.disabled.as_ref().map(|d| d.why.as_str()))
        .collect();
    assert_eq!(whys, [Some("hung"), Some("crashed"), Some("crashed"), None]);
    assert!(st[..3].iter().all(|s| s.helper_pid.is_none()));
    assert_eq!(
        c.sandbox().helper_pids().len(),
        1,
        "only the working one's helper"
    );
    assert!(st[0].bypass, "bypass kept");
    let saved = SavedChain::load(&file).unwrap().unwrap();
    assert!(saved.slots[..3].iter().all(|s| s.disabled.is_some()));
    assert_eq!(saved.slots[0].state.as_ref(), Some(&good), "its state kept");
    c.shutdown().unwrap();
    drop((c, r));

    // The next start: neither loaded, the working one plays.
    let (c, mut r) = chain(Some(file.clone()));
    let t = Instant::now();
    assert!(c.restore_file_as(Restore::Background).is_empty());
    let h = health(&c);
    assert_eq!(h[..3], [Health::Disabled; 3]);
    assert!(matches!(h[3], Health::Loading | Health::Running));
    assert!(c.wait_loaded(Duration::from_secs(10)));
    assert!(t.elapsed() < Duration::from_secs(1), "{:?}", t.elapsed());
    assert_eq!(c.sandbox().helper_pids().len(), 1, "none started for them");
    let blocks = play_live(&mut r, Duration::from_secs(1), |k| k >= 20);
    assert!(blocks[2..].iter().all(|b| b.0));
    // Tried again: turned off again, counted.
    c.retry(1).unwrap();
    assert_eq!(c.status()[1].health, Health::Loading);
    assert!(until(&c, Duration::from_secs(10), |c| {
        c.status()[1].health == Health::Disabled
    }));
    assert_eq!(c.status()[1].disabled.as_ref().map(|d| d.times), Some(2));
    // The VST3 one "mended" (a new version of the plug-in, as its class): tried, it plays.
    c.shutdown().unwrap();
    drop((c, r));
    let mut saved = SavedChain::load(&file).unwrap().unwrap();
    saved.slots[2].plugin = VST3_ID.into();
    std::fs::write(&file, serde_json::to_vec(&saved).unwrap()).unwrap();
    let (c, _r) = chain(Some(file.clone()));
    c.restore_file_as(Restore::Background);
    assert_eq!(c.status()[2].health, Health::Disabled, "until it is tried");
    c.retry(2).unwrap();
    assert!(c.wait_loaded(Duration::from_secs(10)));
    let st = &c.status()[2];
    assert_eq!((st.health, st.disabled.is_none()), (Health::Running, true));
    assert!(
        SavedChain::load(&file).unwrap().unwrap().slots[2]
            .disabled
            .is_none(),
        "its record cleared"
    );
    c.shutdown().unwrap();
    assert!(c.sandbox().helper_pids().is_empty(), "every helper stopped");
}

/// Item 2, while it plays: an effect that keeps crashing is given up on, turned off and saved
/// so; at the next start it is not loaded.
#[test]
fn an_effect_given_up_on_as_it_plays_stays_off_at_the_next_start() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("effects.json");
    let mut o = options(Some(file.clone()));
    o.sandbox.max_restarts = 2;
    let (c, mut r) = Chain::new(o.clone());
    r.prepare(RATE, BLOCK);
    c.add(&found(&vst3_plugin(), VST3_ID, "vst3")).unwrap();
    play_live(&mut r, Duration::from_secs(30), |k| {
        if k % 250 == 249 && c.status()[0].health == Health::Running {
            let _ = c.kill(0);
        }
        c.status()[0].health == Health::Disabled
    });
    assert!(until(&c, Duration::from_secs(2), |_| {
        SavedChain::load(&file)
            .ok()
            .flatten()
            .is_some_and(|s| s.slots[0].disabled.is_some())
    }));
    let saved = SavedChain::load(&file).unwrap().unwrap();
    assert_eq!(
        saved.slots[0].disabled.as_ref().map(|d| d.why.as_str()),
        Some("crashed")
    );
    assert!(saved.slots[0].state.is_some(), "its last state kept");
    c.shutdown().unwrap();
    drop((c, r));
    let (c, _r) = Chain::new(o);
    c.restore_file_as(Restore::Background);
    assert_eq!(health(&c), [Health::Disabled]);
    assert!(c.sandbox().helper_pids().is_empty());
    c.shutdown().unwrap();
}

/// Item 4: saved settings that crash or hang an effect as they are given to it (each format)
/// turn it off at once, said to be its settings; loaded without them it plays as the plug-in
/// starts, the settings kept aside in the file, and at the next start it loads as it now is;
/// given them back on purpose, they still crash it, and it plays on as it was, the settings
/// still aside. Settings that crash it as soon as it plays, every time it is restored, are
/// thought the cause too.
#[test]
fn settings_that_crash_an_effect_are_not_given_again_and_kept_aside() {
    for format in ["clap", "vst3"] {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("effects.json");
        let (good, with_fault) = states(format);
        let (path, id) = match format {
            "clap" => (clap_plugin(), CLAP_ID),
            _ => (vst3_plugin(), VST3_ID),
        };
        let mut crashes = slot(&path, format, id, "Crashed by its settings");
        crashes.state = Some(with_fault(1));
        let mut hangs = slot(&path, format, id, "Hung by its settings");
        hangs.state = Some(with_fault(2));
        write(&file, vec![crashes.clone(), hangs]);
        let (c, mut r) = chain(Some(file.clone()));
        c.restore_file_as(Restore::Background);
        assert!(c.wait_loaded(LOAD_TIMEOUT * 3), "{format}");
        let st = c.status();
        for s in &st {
            assert_eq!(s.health, Health::Disabled, "{format}: {}", s.name);
            assert_eq!(
                s.disabled.as_ref().map(|d| d.why.as_str()),
                Some("settings"),
                "{format}: {}",
                s.name
            );
        }
        // Loaded without them: it plays, its settings kept aside.
        c.load_without_settings(0).unwrap();
        assert!(c.wait_loaded(Duration::from_secs(10)));
        let st = &c.status()[0];
        assert_eq!(
            (st.health, st.set_aside),
            (Health::Running, true),
            "{format}"
        );
        let blocks = play_live(&mut r, Duration::from_secs(1), |k| k >= 10);
        assert!(
            blocks.iter().all(|b| !b.0),
            "{format}: the plug-in's own gain, 1"
        );
        let saved = SavedChain::load(&file).unwrap().unwrap();
        assert_eq!(
            saved.slots[0].set_aside, crashes.state,
            "{format}: kept aside"
        );
        assert!(saved.slots[0].disabled.is_none());
        assert_ne!(saved.slots[0].state, crashes.state);
        // Given them back on purpose: they still crash it, it plays on as it was.
        let pid = c.status()[0].helper_pid;
        c.try_set_aside(0).unwrap();
        assert!(c.wait_loaded(Duration::from_secs(10)));
        let st = &c.status()[0];
        assert_eq!(st.health, Health::Running, "{format}");
        assert_eq!(st.helper_pid, pid, "{format}: the same helper");
        assert!(st.set_aside && st.try_failed.is_some(), "{format}");
        c.shutdown().unwrap();
        drop((c, r));
        // The next start: as it now is, its settings still aside.
        let (c, _r) = chain(Some(file.clone()));
        assert!(
            c.restore_file_as(Restore::Wait)
                .iter()
                .all(|e| e.contains("Hung"))
        );
        assert_eq!(c.status()[0].health, Health::Running, "{format}");
        assert_eq!(
            c.saved().slots[0].set_aside,
            crashes.state,
            "{format}: still kept"
        );
        c.shutdown().unwrap();

        // Settings that crash it at once as it plays, each time it is restored.
        let mut o = options(Some(file.clone()));
        o.sandbox.max_restarts = 2;
        let mut quick = slot(&path, format, id, "Crashes as it plays");
        quick.state = Some(with_fault(3));
        write(&file, vec![quick]);
        let (c, mut r) = Chain::new(o);
        r.prepare(RATE, BLOCK);
        c.restore_file_as(Restore::Background);
        assert!(c.wait_loaded(Duration::from_secs(10)));
        play_live(&mut r, Duration::from_secs(15), |_| {
            c.status()[0].health == Health::Disabled
        });
        let st = &c.status()[0];
        assert_eq!(
            st.disabled.as_ref().map(|d| d.why.as_str()),
            Some("settings"),
            "{format}: {:?}",
            st.disabled
        );
        c.load_without_settings(0).unwrap();
        assert!(c.wait_loaded(Duration::from_secs(10)));
        assert_eq!(c.status()[0].health, Health::Running, "{format}");
        let _ = good;
        c.shutdown().unwrap();
    }
}

/// Item 3: a safe start puts the chain in place and loads none of it (no helper started), the
/// file kept as it was; the synth plays dry; the effects turned back on load and play, those
/// turned off for crashing staying off.
#[test]
fn a_safe_start_loads_no_effect_until_they_are_turned_back_on() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("effects.json");
    let (good, _) = states("vst3");
    let mut works = slot(&vst3_plugin(), "vst3", VST3_ID, "Works");
    works.state = Some(good);
    let mut off = slot(&clap_plugin(), "clap", CLAP_CRASH_ID, "Turned off");
    off.disabled = Some(ca72_fx::Disabled {
        why: "crashed".into(),
        times: 1,
        at: 0,
        detail: String::new(),
    });
    write(&file, vec![works, off]);
    let before = std::fs::read(&file).unwrap();
    let saved = SavedChain::load(&file).unwrap().unwrap();
    assert!(saved.has_effects());
    assert!(ca72_fx::launch::safe_start(ca72_fx::launch::Last::DidNotClose, true, false).is_some());

    let (c, mut r) = chain(Some(file.clone()));
    assert!(c.restore_file_as(Restore::Off).is_empty());
    assert_eq!(health(&c), [Health::Off, Health::Disabled]);
    assert!(c.is_off() && !c.active());
    assert!(c.sandbox().helper_pids().is_empty(), "nothing loaded");
    let blocks = play_live(&mut r, Duration::from_secs(1), |k| k >= 10);
    assert!(blocks.iter().all(|b| !b.0), "dry");
    c.save_now().unwrap();
    assert_eq!(
        SavedChain::load(&file).unwrap().unwrap(),
        SavedChain::load_bytes(&before).unwrap(),
        "saved as it was"
    );
    c.turn_on();
    assert!(c.wait_loaded(Duration::from_secs(10)));
    assert_eq!(health(&c), [Health::Running, Health::Disabled]);
    assert!(!c.is_off() && c.active());
    let blocks = play_live(&mut r, Duration::from_secs(1), |k| k >= 10);
    assert!(blocks[2..].iter().all(|b| b.0), "wet");
    c.shutdown().unwrap();
}

/// Item 2, the list: a plug-in file that crashes or hangs its helper as it is listed is
/// remembered so (by its path, size and time) and not listed again (nor loaded) while it is
/// unchanged; the list's SCAN AGAIN tries it again; changed, it is listed again.
#[test]
fn a_plugin_that_crashes_or_hangs_as_it_is_listed_is_not_listed_again_until_asked() {
    let dir = tempfile::tempdir().unwrap();
    let plugins = dir.path().join("plugins");
    std::fs::create_dir(&plugins).unwrap();
    std::fs::copy(clap_plugin(), plugins.join("good.clap")).unwrap();
    std::fs::copy(clap_plugin(), plugins.join("crash-at-scan.clap")).unwrap();
    std::fs::copy(clap_plugin(), plugins.join("hang-at-scan.clap")).unwrap();
    let cache = dir.path().join("cache.json");
    let dirs = [plugins.clone()];
    let timeout = Duration::from_secs(2);
    let cat = catalog::scan(&helper(), &dirs, Some(&cache), timeout);
    assert_eq!(cat.scanned, 3);
    assert_eq!(cat.effects().count(), 1);
    let mut faults: Vec<_> = cat.unlisted.iter().map(|u| u.fault).collect();
    faults.sort_by_key(|f| format!("{f:?}"));
    assert_eq!(faults, [Fault::Crashed, Fault::Hung], "{:?}", cat.unlisted);
    let t = Instant::now();
    let cat = catalog::scan(&helper(), &dirs, Some(&cache), timeout);
    assert_eq!((cat.scanned, cat.unlisted.len()), (0, 2), "remembered");
    assert!(t.elapsed() < Duration::from_secs(1), "{:?}", t.elapsed());
    let cat = catalog::scan_as(&helper(), &dirs, Some(&cache), timeout, true);
    assert_eq!((cat.scanned, cat.unlisted.len()), (2, 2), "SCAN AGAIN");
    // "Updated": the crashing file changed, it is listed again.
    std::fs::remove_file(plugins.join("crash-at-scan.clap")).unwrap();
    std::fs::write(
        plugins.join("crash-at-scan.clap"),
        b"not a plug-in any more",
    )
    .unwrap();
    let cat = catalog::scan(&helper(), &dirs, Some(&cache), timeout);
    assert_eq!(cat.scanned, 1);
    assert!(
        cat.unlisted
            .iter()
            .any(|u| u.fault == Fault::Refused && u.path.ends_with("crash-at-scan.clap"))
    );
}

/// Item 1: an effect that hangs as it is made, of either format, holds nothing but its own
/// slot: the chain is in place at once, the slot loading; when the load's time is up it is
/// turned off and its helper killed.
#[test]
fn an_effect_that_hangs_as_it_loads_is_turned_off_when_its_time_is_up() {
    let (c, _r) = chain(None);
    let saved = SavedChain {
        format: 1,
        slots: vec![
            slot(&clap_plugin(), "clap", CLAP_HANG_ID, "Hangs"),
            slot(&vst3_plugin(), "vst3", VST3_HANG_ID, "Hangs too"),
        ],
    };
    let t = Instant::now();
    assert!(c.restore_as(&saved, Restore::Background).is_empty());
    assert!(
        t.elapsed() < Duration::from_millis(500),
        "{:?}",
        t.elapsed()
    );
    assert_eq!(health(&c), [Health::Loading, Health::Loading]);
    assert!(!c.wait_loaded(LOAD_TIMEOUT / 2), "still loading");
    assert!(c.wait_loaded(LOAD_TIMEOUT * 3));
    let st = c.status();
    for s in &st {
        assert_eq!(s.health, Health::Disabled, "{}", s.name);
        assert_eq!(s.disabled.as_ref().map(|d| d.why.as_str()), Some("hung"));
    }
    assert!(c.sandbox().helper_pids().is_empty(), "their helpers killed");
    c.shutdown().unwrap();
}
