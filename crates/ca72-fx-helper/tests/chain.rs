//! The effects' chain with DAW-v3's test plug-ins (docs/fx-host.md): both formats found and
//! told from instruments, the effects' output exact, bypass transparent to the bit, latency
//! accounted, state saved and restored, a sample-rate change, and an effect that crashes,
//! hangs or is killed: the synth's sound goes on dry, the failure is reported, and the
//! effect comes back.
//!
//! The test plug-ins compute `y[n] = gain * (x[n - latency] + mix * x[n - latency - delay])`
//! in plain `f32`, bit-exact at any block size; their hidden `crash` and `hang` parameters
//! abort their process or stop it answering.

use ca72_fx::chain::{ChainOptions, SavedChain};
use ca72_fx::{Chain, Health, Kind, Mode, Runner, catalog};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const P_GAIN: u32 = 1;
const P_LATENCY: u32 = 4;
const P_CRASH: u32 = 100;
const P_HANG: u32 = 101;
const CLAP_ID: &str = "org.daw-v3.test-plugin";
const VST3_ID: &str = "44415733546573744761696E50726F63";
const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;

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

fn options(realtime: bool) -> ChainOptions {
    let mut o = ChainOptions::new(helper());
    o.sandbox.realtime = realtime;
    o.sandbox.hang_timeout = Duration::from_millis(if realtime { 400 } else { 5000 });
    o.sandbox.snapshot_debounce = Duration::from_millis(50);
    o
}

fn chain(realtime: bool) -> (Chain, Runner) {
    let (c, mut r) = Chain::new(options(realtime));
    r.prepare(RATE, BLOCK);
    (c, r)
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
        found(&clap_plugin(), CLAP_ID, "clap"),
        found(&vst3_plugin(), VST3_ID, "vst3"),
    ]
}

/// A value in the plug-in's units as its parameter takes it: a CLAP parameter in its own
/// units, a VST3 one normalized over its plain range.
fn value(chain: &Chain, at: usize, id: u32, units: f64) -> f64 {
    let params = chain.params(at);
    let p = params.iter().find(|p| p.id == id).expect("the parameter");
    match (p.plain.first(), p.plain.last()) {
        (Some(lo), Some(hi)) if hi > lo => p.min + (p.max - p.min) * (units - lo) / (hi - lo),
        _ => units,
    }
}

/// A deterministic test signal.
fn signal(start: usize, n: usize) -> (Vec<f32>, Vec<f32>) {
    let f = |i: usize, k: u32| {
        let x = (i as u32).wrapping_mul(2_654_435_761).wrapping_add(k) >> 8;
        (x as f32 / (1u32 << 24) as f32) - 0.5
    };
    (
        (start..start + n).map(|i| f(i, 1)).collect(),
        (start..start + n).map(|i| f(i, 7)).collect(),
    )
}

/// Plays `blocks` blocks of the signal from `start` through the chain offline; input and
/// output.
fn play(r: &mut Runner, start: usize, blocks: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let (input, input_r) = signal(start, blocks * BLOCK);
    let (mut l, mut rr) = (input.clone(), input_r);
    for k in 0..blocks {
        let s = k * BLOCK..(k + 1) * BLOCK;
        r.process(&mut l[s.clone()], &mut rr[s], None);
    }
    (input, l, rr)
}

#[test]
fn effects_are_found_and_instruments_left_out() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(clap_plugin(), dir.path().join("test.clap")).unwrap();
    std::fs::copy(vst3_plugin(), dir.path().join("test.vst3")).unwrap();
    std::fs::write(dir.path().join("broken.clap"), b"not a plug-in").unwrap();
    let cache = dir.path().join("cache.json");
    let dirs = [dir.path().to_path_buf()];
    let cat = catalog::scan(&helper(), &dirs, Some(&cache), Duration::from_secs(20));
    let effects: Vec<_> = cat.effects().map(|f| f.desc.id.clone()).collect();
    assert_eq!(effects, [CLAP_ID, VST3_ID], "the two test effects");
    assert!(
        cat.plugins
            .iter()
            .any(|f| f.kind == Kind::Instrument && f.desc.name == "DAW-v3 Test Sine"),
        "the VST3 module's instrument is found and left out: {:?}",
        cat.plugins
    );
    assert_eq!(cat.unlisted.len(), 1, "the broken file: {:?}", cat.unlisted);
    assert_eq!(cat.scanned, 3);
    // Again: every listing from the cache, nothing loaded.
    let again = catalog::scan(&helper(), &dirs, Some(&cache), Duration::from_secs(20));
    assert_eq!(again.scanned, 0);
    assert_eq!(again.plugins, cat.plugins);
    // The instrument is refused.
    let (c, _r) = chain(false);
    let sine = cat
        .plugins
        .iter()
        .find(|f| f.kind == Kind::Instrument)
        .unwrap();
    assert!(c.add(sine).unwrap_err().contains("instrument"));
    // Even when the list does not say so: loaded, it says what it is, and is unloaded.
    let mut unsaid = sine.clone();
    unsaid.kind = Kind::Unknown;
    let e = c.add(&unsaid).unwrap_err();
    assert!(e.contains("only effects"), "{e}");
    assert!(c.is_empty());
}

#[test]
fn the_output_is_the_effects_bypass_is_transparent_and_latency_is_accounted() {
    for p in plugins() {
        let (c, mut r) = chain(false);
        c.add(&p).unwrap();
        // At its defaults (gain 1, no delay mix, no latency) the effect is exact.
        let (input, l, _) = play(&mut r, 0, 4);
        assert_eq!(l, input, "{}: unity", p.desc.format);
        let gain = value(&c, 0, P_GAIN, 0.5);
        c.set_param(0, P_GAIN, gain).unwrap();
        let (input, l, rr) = play(&mut r, 4 * BLOCK, 4);
        let (_, input_r) = signal(4 * BLOCK, 4 * BLOCK);
        for i in 0..l.len() {
            assert_eq!(
                l[i].to_bits(),
                (input[i] * 0.5).to_bits(),
                "{} left {i}",
                p.desc.format
            );
            assert_eq!(rr[i].to_bits(), (input_r[i] * 0.5).to_bits());
        }
        // Bypassed: the input to the bit.
        c.set_bypass(0, true).unwrap();
        let (input, l, rr) = play(&mut r, 8 * BLOCK, 4);
        let (_, input_r) = signal(8 * BLOCK, 4 * BLOCK);
        assert!(
            l.iter()
                .zip(&input)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        );
        assert!(
            rr.iter()
                .zip(&input_r)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        );
        assert_eq!(c.latency(), 0);
        c.set_bypass(0, false).unwrap();
        // A latency the plug-in reports: taken at its activation, so through its state into
        // a chain restored from it.
        c.set_param(0, P_LATENCY, value(&c, 0, P_LATENCY, 100.0))
            .unwrap();
        play(&mut r, 12 * BLOCK, 2);
        c.save_now().unwrap();
        let saved = c.saved();
        let (c2, mut r2) = chain(false);
        assert!(c2.restore(&saved).is_empty());
        assert_eq!(c2.status()[0].latency, 100, "{}", p.desc.format);
        assert_eq!(c2.latency(), 100);
        let (input, l, _) = play(&mut r2, 0, 4);
        assert!(l[..100].iter().all(|x| *x == 0.0));
        for i in 100..l.len() {
            assert_eq!(
                l[i].to_bits(),
                (input[i - 100] * 0.5).to_bits(),
                "{} {i}",
                p.desc.format
            );
        }
    }
}

#[test]
fn a_pipelined_effect_is_one_block_late_and_says_so() {
    let mut o = options(false);
    o.mode = Mode::Pipelined;
    let (c, mut r) = Chain::new(o);
    r.prepare(RATE, BLOCK);
    c.add(&plugins()[0]).unwrap();
    assert_eq!(c.latency(), BLOCK as u32);
    let (input, l, _) = play(&mut r, 0, 6);
    assert!(l[..BLOCK].iter().all(|x| *x == 0.0));
    assert_eq!(&l[BLOCK..], &input[..input.len() - BLOCK]);
}

#[test]
fn the_chain_is_ordered_saved_to_its_file_and_restored() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("effects.json");
    let mut o = options(false);
    o.file = Some(file.clone());
    let (c, mut r) = Chain::new(o.clone());
    r.prepare(RATE, BLOCK);
    let [clap, vst3] = plugins();
    c.add(&clap).unwrap();
    c.add(&vst3).unwrap();
    c.set_param(0, P_GAIN, value(&c, 0, P_GAIN, 0.5)).unwrap();
    c.set_param(1, P_GAIN, value(&c, 1, P_GAIN, 0.25)).unwrap();
    play(&mut r, 0, 2);
    c.move_to(1, 0).unwrap();
    c.set_bypass(1, true).unwrap();
    assert_eq!(c.status()[0].format, "vst3");
    let (input, l, _) = play(&mut r, 0, 2);
    assert_eq!(
        l,
        input.iter().map(|x| x * 0.25).collect::<Vec<_>>(),
        "VST3 alone"
    );
    c.shutdown().unwrap();
    drop(r);
    drop(c);

    let saved = SavedChain::load(&file).unwrap().unwrap();
    assert_eq!(saved.slots.len(), 2);
    assert_eq!(saved.slots[0].format, "vst3");
    assert!(!saved.slots[0].bypass && saved.slots[1].bypass);
    assert!(saved.slots.iter().all(|s| s.state.is_some()));

    // A new session: the same chain, the same sound.
    let (c, mut r) = Chain::new(o.clone());
    r.prepare(RATE, BLOCK);
    assert!(c.restore_file().is_empty());
    let st = c.status();
    assert_eq!((st[0].format.as_str(), st[0].bypass), ("vst3", false));
    assert_eq!((st[1].format.as_str(), st[1].bypass), ("clap", true));
    let (input, l, _) = play(&mut r, 0, 2);
    assert_eq!(l, input.iter().map(|x| x * 0.25).collect::<Vec<_>>());
    c.set_bypass(1, false).unwrap();
    let (input, l, _) = play(&mut r, 0, 2);
    assert_eq!(l, input.iter().map(|x| x * 0.25 * 0.5).collect::<Vec<_>>());
    // Replaced: the CLAP effect in place of the VST3 one, at unity, the other kept.
    assert!(c.active());
    c.replace(0, &plugins()[0]).unwrap();
    let st = c.status();
    assert_eq!(
        (st[0].format.as_str(), st[1].format.as_str()),
        ("clap", "clap")
    );
    let (input, l, _) = play(&mut r, 0, 2);
    assert_eq!(l, input.iter().map(|x| x * 0.5).collect::<Vec<_>>());
    c.remove(0).unwrap();
    assert_eq!(c.status().len(), 1);
    c.set_bypass(0, true).unwrap();
    assert!(!c.active(), "only a bypassed effect");
    c.set_bypass(0, false).unwrap();
    c.shutdown().unwrap();
    drop(r);
    drop(c);

    // A plug-in that is not installed any more: kept, dry, its state saved as it was.
    let mut saved = SavedChain::load(&file).unwrap().unwrap();
    saved.slots[0].path = dir.path().join("gone.clap");
    let state = saved.slots[0].state.clone();
    std::fs::write(&file, serde_json::to_vec(&saved).unwrap()).unwrap();
    let (c, mut r) = Chain::new(o.clone());
    r.prepare(RATE, BLOCK);
    let errors = c.restore_file();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(c.status()[0].health, Health::Missing);
    let (input, l, _) = play(&mut r, 0, 2);
    assert_eq!(l, input, "dry");
    c.shutdown().unwrap();
    assert_eq!(
        SavedChain::load(&file).unwrap().unwrap().slots[0].state,
        state
    );

    // A file that cannot be read is left as it is.
    std::fs::write(&file, b"{damaged").unwrap();
    let (c, _r) = Chain::new(o);
    assert_eq!(c.restore_file().len(), 1);
    c.add(&plugins()[0]).unwrap();
    let _ = c.shutdown();
    assert_eq!(std::fs::read(&file).unwrap(), b"{damaged");
}

/// Changes made at once on several threads are saved as they were made: once they are
/// done, the file is the chain as it shows. Each save takes the chain and writes it while
/// holding it, so an older chain is never written over a newer one (a save's chain was once
/// taken before its turn to write; on Windows, where a write and its flush take milliseconds,
/// the file lagged what the chain showed).
#[test]
fn changes_made_at_once_are_saved_as_the_chain_shows() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("effects.json");
    let mut o = options(false);
    o.file = Some(file.clone());
    let (c, mut r) = Chain::new(o);
    r.prepare(RATE, BLOCK);
    let [clap, vst3] = plugins();
    for p in [&clap, &vst3, &clap, &vst3] {
        c.add(p).unwrap();
    }
    for round in 0..40 {
        std::thread::scope(|s| {
            for at in 0..4 {
                let c = &c;
                s.spawn(move || {
                    for k in 0..6 {
                        c.set_bypass(at, (round + k * at) % 3 == 0).unwrap();
                    }
                });
            }
        });
        let saved = SavedChain::load(&file).unwrap().unwrap();
        let shown: Vec<bool> = c.status().iter().map(|s| s.bypass).collect();
        let on_disk: Vec<bool> = saved.slots.iter().map(|s| s.bypass).collect();
        assert_eq!(on_disk, shown, "round {round}");
    }
    c.shutdown().unwrap();
}

#[test]
fn a_sample_rate_change_loads_the_effects_again_with_their_state() {
    for p in plugins() {
        let (c, mut r) = chain(false);
        c.add(&p).unwrap();
        c.set_param(0, P_GAIN, value(&c, 0, P_GAIN, 0.5)).unwrap();
        play(&mut r, 0, 2);
        let pid = c.status()[0].helper_pid;
        r.prepare(44_100.0, 512);
        assert_eq!(c.sample_rate(), 44_100.0);
        // (Loaded again on a thread of its own: the device waits for none of it.)
        assert!(c.wait_loaded(Duration::from_secs(20)));
        let st = &c.status()[0];
        assert_eq!(st.health, Health::Running);
        assert_eq!(st.helper_pid, pid, "{}: the same helper", p.desc.format);
        let (input, l, _) = play(&mut r, 0, 4);
        assert_eq!(
            l,
            input.iter().map(|x| x * 0.5).collect::<Vec<_>>(),
            "{}",
            p.desc.format
        );
    }
}

/// Plays in real time, paced, each block's deadline `budget` after its start, until `done`
/// or `limit`; each block's frames and whether they came out dry or wet (gain 0.5).
fn play_live(
    r: &mut Runner,
    budget: Duration,
    limit: Duration,
    mut each: impl FnMut(usize) -> bool,
) -> Vec<(bool, Duration)> {
    let period = Duration::from_secs_f64(BLOCK as f64 / RATE);
    let start = Instant::now();
    let mut out = Vec::new();
    let mut k = 0;
    while start.elapsed() < limit {
        if !each(k) {
            break;
        }
        let (input, input_r) = signal(k * BLOCK, BLOCK);
        let (mut l, mut rr) = (input.clone(), input_r.clone());
        let t = Instant::now();
        r.process(&mut l, &mut rr, Some(t + budget));
        let took = t.elapsed();
        let wet = l.iter().zip(&input).all(|(a, b)| *a == b * 0.5);
        let dry = l == input && rr == input_r;
        assert!(
            wet || dry,
            "block {k}: neither the effect's nor the dry sound"
        );
        out.push((wet, took));
        k += 1;
        std::thread::sleep(period.saturating_sub(t.elapsed()));
    }
    out
}

fn kinds(c: &Chain) -> Vec<String> {
    c.events_from(0).0.into_iter().map(|e| e.kind).collect()
}

#[test]
fn a_crash_a_hang_or_a_kill_leaves_the_synth_playing_and_is_reported() {
    let budget = Duration::from_millis(30);
    for p in plugins() {
        for fault in ["crash", "hang", "kill"] {
            let (c, mut r) = chain(true);
            c.add(&p).unwrap();
            c.set_param(0, P_GAIN, value(&c, 0, P_GAIN, 0.5)).unwrap();
            let mut faulted = None;
            let mut back = false;
            let blocks = play_live(&mut r, budget, Duration::from_secs(20), |k| {
                let wet_now = c.status()[0].health == Health::Running;
                if k == 40 {
                    match fault {
                        "crash" => c.set_param(0, P_CRASH, 1.0).unwrap(),
                        "hang" => c.set_param(0, P_HANG, 1.0).unwrap(),
                        _ => c.kill(0).unwrap(),
                    }
                    faulted = Some(k);
                }
                if faulted.is_some() && k > 60 && wet_now && kinds(&c).contains(&"restored".into())
                {
                    back = true;
                }
                // Played on for a while after it came back.
                !(back && k > 200)
            });
            let what = format!("{} {fault}", p.desc.format);
            let first_wet = blocks.iter().position(|b| b.0);
            assert!(
                first_wet.is_some_and(|i| i < 40),
                "{what}: wet before the fault"
            );
            let dry_after: usize = blocks[41..].iter().filter(|b| !b.0).count();
            // (A killed helper can be back before the next block is due.)
            assert!(
                dry_after > 0 || fault == "kill",
                "{what}: dry while it was down"
            );
            assert!(
                blocks.last().is_some_and(|b| b.0),
                "{what}: wet again at the end: {:?} {:?} {:?}",
                blocks.iter().map(|b| u8::from(b.0)).collect::<Vec<_>>(),
                c.status()[0],
                kinds(&c)
            );
            // Never held past its deadline by more than the scheduler's slack.
            let worst = blocks.iter().map(|b| b.1).max().unwrap();
            assert!(
                worst < budget + Duration::from_millis(20),
                "{what}: a block took {worst:?}"
            );
            let k = kinds(&c);
            assert!(k.contains(&"crashed".to_owned()), "{what}: {k:?}");
            assert!(k.contains(&"restored".to_owned()), "{what}: {k:?}");
            if fault == "hang" {
                assert!(k.contains(&"hung".to_owned()), "{what}: {k:?}");
            }
            let st = &c.status()[0];
            assert_eq!(st.crashes, 1, "{what}");
            assert_eq!(st.health, Health::Running);
            c.shutdown().unwrap();
        }
    }
}

/// The chain as the synth runs it (decisions.md R-FX-MAC): asked before each block whether it
/// is empty, and run only if not; in real time, paced, at a device's block size, each block due
/// by 0.9 of its period (the standalone's `fx::DEADLINE`). An effect added to the empty chain
/// while it plays is heard from its first blocks on, and every block after: none comes out dry
/// or is missed. (On the Mac the chain was asked before it had taken the effect, and never took
/// it: every block came out dry, the effect shown running.)
#[test]
fn an_effect_added_while_the_synth_plays_is_heard_at_once() {
    for p in plugins() {
        let what = p.desc.format.clone();
        let (c, mut r) = chain(true);
        let period = Duration::from_secs_f64(BLOCK as f64 / RATE);
        let mut blocks = Vec::new();
        for k in 0..400 {
            if k == 50 {
                c.add(&p).unwrap();
                c.set_param(0, P_GAIN, value(&c, 0, P_GAIN, 0.5)).unwrap();
            }
            let (input, input_r) = signal(k * BLOCK, BLOCK);
            let (mut l, mut rr) = (input.clone(), input_r.clone());
            let t = Instant::now();
            if !r.is_empty() {
                r.process(&mut l, &mut rr, Some(t + period.mul_f64(0.9)));
            }
            let wet = l.iter().zip(&input).all(|(a, b)| *a == b * 0.5);
            let dry = l == input && rr == input_r;
            assert!(wet || dry, "{what}: block {k} neither wet nor dry");
            blocks.push(wet);
            std::thread::sleep(period.saturating_sub(t.elapsed()));
        }
        assert!(
            !blocks[..50].iter().any(|w| *w),
            "{what}: dry before the effect"
        );
        let first = blocks.iter().position(|w| *w);
        assert!(
            first.is_some_and(|i| i <= 52),
            "{what}: heard from the block it was added in: first wet {first:?}"
        );
        let dry_after = blocks[first.unwrap_or(0)..].iter().filter(|w| !**w).count();
        let st = &c.status()[0];
        assert_eq!(
            (dry_after, st.misses),
            (0, 0),
            "{what}: every block after it wet, none missed"
        );
        c.shutdown().unwrap();
    }
}

#[test]
fn an_effect_that_keeps_crashing_is_given_up_on_and_passes_its_input() {
    let mut o = options(true);
    o.sandbox.max_restarts = 1;
    let (c, mut r) = Chain::new(o);
    r.prepare(RATE, BLOCK);
    c.add(&plugins()[0]).unwrap();
    c.set_param(0, P_GAIN, value(&c, 0, P_GAIN, 0.5)).unwrap();
    let blocks = play_live(
        &mut r,
        Duration::from_millis(30),
        Duration::from_secs(20),
        |k| {
            if k % 40 == 39 && c.status()[0].health == Health::Running {
                let _ = c.kill(0);
            }
            c.status()[0].health != Health::Disabled || k < 300
        },
    );
    assert_eq!(c.status()[0].health, Health::Disabled);
    assert!(kinds(&c).contains(&"failed".to_owned()));
    assert!(blocks.iter().rev().take(20).all(|b| !b.0), "dry at the end");
}

/// The editor's window, on a display (`CA72_FX_GUI_TEST=1`; it opens a small window of one
/// colour for a moment).
#[test]
fn the_editor_opens_in_its_helper() {
    if std::env::var_os("CA72_FX_GUI_TEST").is_none() {
        eprintln!("skipped: set CA72_FX_GUI_TEST=1 on a desktop to open the editors");
        return;
    }
    for p in plugins() {
        let (c, _r) = chain(true);
        c.add(&p).unwrap();
        // Opened, closed and opened again; opened again while open (brought forward): on screen
        // each time (decisions.md R-FX-MAC).
        for (k, close) in [true, false, true].into_iter().enumerate() {
            let v = c.open_editor(0).unwrap();
            assert_eq!(v["open"], true, "{} {k}: {v}", p.desc.format);
            assert_eq!(v["visible"], true, "{} {k}: {v}", p.desc.format);
            std::thread::sleep(Duration::from_millis(300));
            if close {
                let v = c.close_editor(0).unwrap();
                assert_eq!(v["open"], false, "{} {k}: {v}", p.desc.format);
            }
        }
    }
}

/// Third-party effects where they are (`CA72_FX_THIRD_PARTY`: folders or plug-in files, the
/// platform's path list): each listed, loaded, played a second of noise with a finite
/// output that is not the input, and restored from its state into a chain of its own; its
/// editor opened too with `CA72_FX_GUI_TEST=1`.
#[test]
fn third_party_effects_load_play_and_keep_their_state() {
    let Some(paths) = std::env::var_os("CA72_FX_THIRD_PARTY") else {
        eprintln!("skipped: set CA72_FX_THIRD_PARTY to effects to try");
        return;
    };
    let dirs: Vec<PathBuf> = std::env::split_paths(&paths).collect();
    let cat = catalog::scan(&helper(), &dirs, None, Duration::from_secs(30));
    assert!(cat.unlisted.is_empty(), "{:?}", cat.unlisted);
    let effects: Vec<_> = cat.effects().cloned().collect();
    assert!(!effects.is_empty(), "no effects in {dirs:?}");
    for f in &effects {
        let what = format!("{} ({}, {})", f.desc.name, f.desc.format, f.path.display());
        let (c, mut r) = chain(false);
        c.add(f).unwrap_or_else(|e| panic!("{what}: {e}"));
        let st = &c.status()[0];
        eprintln!(
            "{what}: {} parameters, latency {}, helper {:?}",
            c.params(0).len(),
            st.latency,
            st.helper_pid
        );
        let blocks = RATE as usize / BLOCK;
        let (input, l, rr) = play(&mut r, 0, blocks);
        assert!(l.iter().chain(&rr).all(|x| x.is_finite()), "{what}: finite");
        assert!(l != input, "{what}: the effect changed the sound");
        if std::env::var_os("CA72_FX_GUI_TEST").is_some() {
            // Said, not failed: a VST3 editor that needs the host's run loop on Linux
            // (`IRunLoop`, which DAW-v3's helper does not give yet) does not attach.
            match c.open_editor(0) {
                Ok(v) => {
                    eprintln!("{what}: editor {v}");
                    let hold = std::env::var("CA72_FX_GUI_HOLD_MS")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(1000);
                    std::thread::sleep(Duration::from_millis(hold));
                    let _ = c.close_editor(0);
                }
                Err(e) => eprintln!("{what}: NO EDITOR: {e}"),
            }
        }
        c.save_now().unwrap();
        let saved = c.saved();
        assert!(saved.slots[0].state.is_some(), "{what}: a state");
        let (c2, mut r2) = chain(false);
        assert!(c2.restore(&saved).is_empty(), "{what}: restored");
        let (_, l2, _) = play(&mut r2, 0, blocks);
        assert!(l2.iter().all(|x| x.is_finite()));
        c2.save_now().unwrap();
        assert_eq!(
            c2.saved().slots[0].state,
            saved.slots[0].state,
            "{what}: the same state back"
        );
        c.shutdown().unwrap();
        c2.shutdown().unwrap();
    }
}

/// What the chain costs the audio thread (run by hand, `--ignored`, best in release): four
/// test effects in sync, 128 frames at 48 kHz, paced, 10 s; the time `Runner::process`
/// takes per block, its median, 99th percentile and worst, and the blocks missed.
#[test]
#[ignore]
fn the_chains_cost() {
    for effects in [1usize, 4] {
        let (c, mut r) = Chain::new(options(true));
        r.prepare(RATE, 128);
        for k in 0..effects {
            c.add(&plugins()[k % 2]).unwrap();
        }
        let period = Duration::from_secs_f64(128.0 / RATE);
        let (mut l, mut rr) = signal(0, 128);
        let mut took = Vec::new();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            let t = Instant::now();
            r.process(&mut l, &mut rr, Some(t + period.mul_f64(0.9)));
            took.push(t.elapsed());
            std::thread::sleep(period.saturating_sub(t.elapsed()));
        }
        took.sort();
        let at = |q: f64| took[((took.len() - 1) as f64 * q) as usize];
        let misses: u64 = c.status().iter().map(|s| s.misses).sum();
        eprintln!(
            "{effects} effect(s), {} blocks: median {:?}, p99 {:?}, worst {:?}, misses {misses}",
            took.len(),
            at(0.5),
            at(0.99),
            took[took.len() - 1]
        );
        c.shutdown().unwrap();
    }
}

/// Third-party effects heard in real time (`CA72_FX_THIRD_PARTY`, as above; the rate and
/// block `CA72_FX_RATE` and `CA72_FX_BLOCK`, else 48 kHz and 256): two seconds of noise,
/// paced, each block's deadline 0.9 of its period as the standalone gives it; how many
/// blocks came out dry and how many were missed. (Diagnostic, said and not failed.)
#[test]
fn third_party_effects_are_heard_in_real_time() {
    let Some(paths) = std::env::var_os("CA72_FX_THIRD_PARTY") else {
        eprintln!("skipped: set CA72_FX_THIRD_PARTY to effects to try");
        return;
    };
    let env = |k: &str, d: f64| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(d)
    };
    let (rate, block) = (
        env("CA72_FX_RATE", RATE),
        env("CA72_FX_BLOCK", BLOCK as f64) as usize,
    );
    let dirs: Vec<PathBuf> = std::env::split_paths(&paths).collect();
    let cat = catalog::scan(&helper(), &dirs, None, Duration::from_secs(30));
    let period = Duration::from_secs_f64(block as f64 / rate);
    for f in cat.effects() {
        let what = format!("{} ({})", f.desc.name, f.desc.format);
        let (c, mut r) = Chain::new(ChainOptions::new(helper()));
        r.prepare(rate, block);
        if let Err(e) = c.add(f) {
            eprintln!("{what}: not loaded: {e}");
            continue;
        }
        let blocks = (2.0 * rate) as usize / block;
        let (mut dry, mut sq_in, mut sq_diff) = (0, 0.0f64, 0.0f64);
        let mut took = Duration::ZERO;
        for k in 0..blocks {
            let (input, input_r) = signal(k * block, block);
            let (mut l, mut rr) = (input.clone(), input_r.clone());
            let t = Instant::now();
            r.process(&mut l, &mut rr, Some(t + period.mul_f64(0.9)));
            took = took.max(t.elapsed());
            if l == input && rr == input_r {
                dry += 1;
            }
            for (a, b) in l.iter().zip(&input) {
                sq_in += f64::from(*b).powi(2);
                sq_diff += f64::from(a - b).powi(2);
            }
            std::thread::sleep(period.saturating_sub(t.elapsed()));
        }
        let st = &c.status()[0];
        eprintln!(
            "{what}: {dry} of {blocks} blocks dry, {} missed, worst {took:?}, \
             difference {:.1} dB below the input, {:?}",
            st.misses,
            10.0 * (sq_in / sq_diff.max(1e-30)).log10(),
            st.health
        );
        c.shutdown().unwrap();
    }
}
