//! The engine neither allocates nor frees on the audio thread once its voice is built: notes,
//! pitch bend, the modulation wheel, the MIDI resets, every kind of control moving and the
//! side chain driven hard, a block at a time, counted by a global allocator on this thread.
//! A failure names where the first came from.

#![allow(unsafe_code, clippy::unwrap_used)]

use ca72_plugin::engine::{Controls, Engine, Event};
use ca72::tuning::Range;
use ca72::voice::Waveform;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct Counting;
static ARMED: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static FREES: AtomicUsize = AtomicUsize::new(0);
static FIRST: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
thread_local! {
    static HERE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Counts one call on the armed thread, recording the first's backtrace (disarmed meanwhile,
/// so the recording itself is not counted).
fn count(counter: &AtomicUsize) {
    if HERE.with(std::cell::Cell::get) && ARMED.swap(false, Ordering::SeqCst) {
        counter.fetch_add(1, Ordering::SeqCst);
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        if let Ok(mut f) = FIRST.lock() {
            f.get_or_insert(bt);
        }
        ARMED.store(true, Ordering::SeqCst);
    }
}

// SAFETY: forwards to the system allocator; only counts (and records a backtrace) while armed.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(&ALLOCS);
        // SAFETY: the caller's contract is passed through.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        count(&FREES);
        // SAFETY: as above.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static A: Counting = Counting;

const RATE: f64 = 48_000.0;
const BLOCK: usize = 128;

#[test]
fn the_engine_neither_allocates_nor_frees_while_it_plays() {
    let mut e = Engine::new();
    let mut c = Controls::default();
    e.set(&c);
    e.event(Event::Note { key: 57, on: true });
    e.prepare(RATE, 7);
    // Counted from the voice's first sample (its first step records the solvers'
    // eliminations), on this thread only.
    HERE.with(|h| h.set(true));
    ARMED.store(true, Ordering::SeqCst);
    let tau = 2.0 * std::f64::consts::PI;
    let mut sink = 0.0f32;
    let mut at = 0usize;
    while at < 2 * RATE as usize {
        let b = at / BLOCK;
        // Notes over the 44 keys and beyond them, the wheels from MIDI, the resets.
        if b.is_multiple_of(20) {
            e.event(Event::Note { key: (30 + b % 70) as u8, on: true });
        }
        if b % 20 == 15 {
            e.event(Event::Note { key: (30 + b % 70) as u8, on: false });
        }
        e.event(Event::PitchBend((at as f32 * 1e-4).sin()));
        e.event(Event::Modulation(0.5 + 0.5 * (at as f32 * 3e-4).sin()));
        if b.is_multiple_of(97) {
            e.event(Event::ResetControllers);
        }
        if b.is_multiple_of(131) {
            e.event(Event::AllNotesOff);
        }
        // Every kind of control: knobs, switches, selectors, the output and bend range.
        c.panel.cutoff = 0.5 + 0.3 * (at as f64 * 1e-4).sin();
        c.panel.noise_volume = 0.5 + 0.3 * (at as f64 * 3e-4).sin();
        c.panel.noise_on = b % 50 < 25;
        c.panel.ext_on = true;
        c.panel.ext_volume = 1.0;
        c.panel.glide_on = b % 40 < 20;
        c.panel.glide = 0.4;
        c.panel.keyboard_control_1 = b % 30 < 15;
        c.panel.osc[1].on = true;
        c.panel.osc[1].range = [Range::R16, Range::R8, Range::R4][b % 3];
        c.panel.osc[2].waveform = [Waveform::ReverseSawtooth, Waveform::Square][b % 2];
        c.panel.a440 = b % 200 > 190;
        c.volume = 0.7 + 0.3 * (at as f64 * 2e-4).sin();
        c.bend_range = [2.0, 12.0, 0.0][b % 3];
        e.set(&c);
        for i in 0..BLOCK {
            let t = (at + i) as f64 / RATE;
            sink += e.tick((0.9 * (tau * 220.0 * t).sin()) as f32);
        }
        sink += e.end_block(BLOCK);
        at += BLOCK;
    }
    e.release_all();
    ARMED.store(false, Ordering::SeqCst);
    std::hint::black_box(sink);
    let (allocs, frees) = (ALLOCS.load(Ordering::SeqCst), FREES.load(Ordering::SeqCst));
    let first = FIRST
        .lock()
        .ok()
        .and_then(|f| f.clone())
        .unwrap_or_default();
    assert_eq!(
        (allocs, frees),
        (0, 0),
        "the engine allocated {allocs} and freed {frees} times while playing; the first:\n{first}"
    );
}
