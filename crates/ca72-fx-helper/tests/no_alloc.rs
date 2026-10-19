//! The chain's audio side never allocates or frees memory (docs/fx-host.md): a counting
//! global allocator, counting only on the thread that plays, while blocks go through two
//! effects in real time and the chain is changed under them (a parameter, a bypass, a move,
//! an effect taken out) and one effect's helper is killed and restored. A binary of its own:
//! the allocator is the process's.

#![allow(unsafe_code)]

use ca72_fx::chain::ChainOptions;
use ca72_fx::{Chain, Kind, catalog};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

struct Counting;

static COUNT: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static WATCHED: Cell<bool> = const { Cell::new(false) };
}

fn note() {
    if WATCHED.try_with(Cell::get).unwrap_or(false) {
        COUNT.fetch_add(1, Ordering::Relaxed);
    }
}

// SAFETY: every call is passed to the system's allocator unchanged; the count is a side
// effect without allocation (a thread-local flag with a constant initialiser).
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note();
        // SAFETY: as the caller's contract.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        note();
        // SAFETY: as the caller's contract.
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new: usize) -> *mut u8 {
        note();
        // SAFETY: as the caller's contract.
        unsafe { System.realloc(ptr, layout, new) }
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn built(stem: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("test exe");
    let dir = exe.parent().expect("deps dir");
    [
        format!("lib{stem}.so"),
        format!("lib{stem}.dylib"),
        format!("{stem}.dll"),
    ]
    .iter()
    .map(|n| dir.join(n))
    .filter_map(|p| Some((std::fs::metadata(&p).ok()?.modified().ok()?, p)))
    .max_by_key(|(t, _)| *t)
    .map(|(_, p)| p)
    .unwrap_or_else(|| panic!("{stem} was not built beside the test"))
}

fn found(stem: &str, id: &str, format: &str) -> catalog::Found {
    catalog::Found {
        path: built(stem),
        kind: Kind::Effect,
        desc: daw_ipc::protocol::PluginDesc {
            id: id.into(),
            name: format.into(),
            format: format.into(),
            ..Default::default()
        },
    }
}

#[test]
fn the_audio_side_never_allocates() {
    let mut o = ChainOptions::new(PathBuf::from(env!("CARGO_BIN_EXE_ca72-fx-helper")));
    o.sandbox.hang_timeout = Duration::from_millis(400);
    let (c, mut r) = Chain::new(o);
    r.prepare(48_000.0, 256);
    c.add(&found("daw_test_plugin", "org.daw-v3.test-plugin", "clap"))
        .unwrap();
    c.add(&found(
        "daw_test_vst3",
        "44415733546573744761696E50726F63",
        "vst3",
    ))
    .unwrap();
    let (mut l, mut rr) = (vec![0.1f32; 256], vec![-0.1f32; 256]);
    let period = Duration::from_secs_f64(256.0 / 48_000.0);
    // A first block outside the count: lazily made things (the clock's, the thread's).
    r.process(&mut l, &mut rr, Some(Instant::now() + period));
    for k in 0..400 {
        match k {
            50 => c.set_param(0, 1, 0.5).unwrap(),
            100 => c.set_bypass(1, true).unwrap(),
            150 => c.move_to(1, 0).unwrap(),
            200 => c.kill(1).unwrap(),
            300 => c.remove(0).unwrap(),
            _ => {}
        }
        let t = Instant::now();
        WATCHED.with(|w| w.set(true));
        r.process(&mut l, &mut rr, Some(t + Duration::from_millis(30)));
        WATCHED.with(|w| w.set(false));
        std::thread::sleep(period.saturating_sub(t.elapsed()));
    }
    assert_eq!(
        COUNT.load(Ordering::Relaxed),
        0,
        "allocations on the audio side"
    );
    assert_eq!(c.len(), 1);
    c.shutdown().unwrap();
}
