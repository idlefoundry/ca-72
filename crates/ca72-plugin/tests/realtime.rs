//! The engine neither allocates nor frees on the audio thread once its voice is built: notes,
//! pitch bend, the modulation wheel, the MIDI resets, All Sound Off putting the voices to rest
//! (decisions.md R18), every kind of control moving, the side chain driven hard (samples that
//! are not finite among them) and POLY's workers asked for and let go (R21), a block at a
//! time, and asking the plug-in's helper thread for its work (R23), counted by a global
//! allocator on this thread.
//! A failure names where the first came from. (Built with `assert_process_allocs`, nih-plug's
//! own global allocator watches instead.)

#![cfg(not(feature = "assert_process_allocs"))]
#![allow(unsafe_code, clippy::unwrap_used)]

use ca72::tuning::Range;
use ca72::voice::Waveform;
use ca72_plugin::engine::{Controls, Engine, Event};
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

/// The tests share the counters: one at a time, each from zero.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn start() -> std::sync::MutexGuard<'static, ()> {
    let guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    ALLOCS.store(0, Ordering::SeqCst);
    FREES.store(0, Ordering::SeqCst);
    if let Ok(mut f) = FIRST.lock() {
        *f = None;
    }
    guard
}

const RATE: f64 = 48_000.0;
const BLOCK: usize = 128;

/// All Sound Off now and then, and the spares it used made again (off the count, as the
/// plug-in's helper thread does) at other times, so that it both finds them made and
/// waits for them.
fn hush(e: &mut Engine, b: usize) {
    if b % 89 == 7 {
        e.event(Event::AllSoundOff);
    }
    if b % 178 == 60 && e.take_mending() {
        ARMED.store(false, Ordering::SeqCst);
        e.spares().mend();
        ARMED.store(true, Ordering::SeqCst);
    }
}

/// POLY's workers started and stopped as the engine asks (decisions.md R21), off the count as
/// the plug-in's helper thread does, a few blocks late, so that the voices also play on
/// this thread while the workers are asked for.
fn serve(e: &mut Engine, b: usize, due: &mut bool) {
    *due |= e.take_serving();
    if *due && b.is_multiple_of(5) {
        ARMED.store(false, Ordering::SeqCst);
        e.crew().serve();
        ARMED.store(true, Ordering::SeqCst);
        *due = false;
    }
}

#[test]
fn the_engine_neither_allocates_nor_frees_while_it_plays() {
    let _serial = start();
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
            e.event(Event::Note {
                key: (30 + b % 70) as u8,
                on: true,
            });
        }
        if b % 20 == 15 {
            e.event(Event::Note {
                key: (30 + b % 70) as u8,
                on: false,
            });
        }
        e.event(Event::PitchBend((at as f32 * 1e-4).sin()));
        e.event(Event::Modulation(0.5 + 0.5 * (at as f32 * 3e-4).sin()));
        if b.is_multiple_of(97) {
            e.event(Event::ResetControllers);
        }
        if b.is_multiple_of(131) {
            e.event(Event::AllNotesOff);
        }
        hush(&mut e, b);
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
            let x = if b % 70 == 3 && i == 5 {
                f32::NAN
            } else {
                (0.9 * (tau * 220.0 * t).sin()) as f32
            };
            sink += e.tick(x);
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

/// With POLY on (eight of the ten voices, chords of ten taking held voices, keys beyond the
/// 44), ENTROPY and SPREAD on, POLY switched off and on and VOICES changed while it plays: no
/// allocation, no free.
#[test]
fn poly_neither_allocates_nor_frees_while_it_plays() {
    let _serial = start();
    let mut e = Engine::new();
    let mut c = Controls {
        poly: true,
        voices: 8,
        entropy: 0.7,
        spread: 0.9,
        ..Controls::default()
    };
    e.set(&c);
    e.prepare(RATE, 7);
    HERE.with(|h| h.set(true));
    ARMED.store(true, Ordering::SeqCst);
    let mut sink = 0.0f32;
    let mut at = 0usize;
    while at < 2 * RATE as usize {
        let b = at / BLOCK;
        if b.is_multiple_of(40) {
            for k in [24u8, 45, 52, 57, 61, 64, 69, 73, 76, 110] {
                e.event(Event::Note {
                    key: k + (b / 40 % 3) as u8,
                    on: true,
                });
            }
        }
        if b % 40 == 30 {
            for k in [24u8, 45, 52, 57, 61, 64] {
                e.event(Event::Note {
                    key: k + (b / 40 % 3) as u8,
                    on: false,
                });
            }
        }
        e.event(Event::PitchBend((at as f32 * 1e-4).sin()));
        hush(&mut e, b);
        if b % 150 == 149 {
            c.poly = !c.poly;
        }
        c.voices = [8, 10, 3][b / 100 % 3];
        c.panel.cutoff = 0.5 + 0.3 * (at as f64 * 1e-4).sin();
        e.set(&c);
        for _ in 0..BLOCK {
            let (l, r) = e.tick_stereo(0.0);
            sink += l + r;
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
        "POLY allocated {allocs} and freed {frees} times while playing; the first:\n{first}"
    );
}

/// POLY's voices shared with workers (decisions.md R11), played between the events as the
/// plug-in plays its host's blocks, each block given its deadline, the side chain driven,
/// chords of ten taking held voices, VOICES changed and POLY switched while it plays, its
/// workers let go and asked for again as it is (R21): no allocation, no free on the audio
/// thread.
#[test]
fn poly_with_workers_neither_allocates_nor_frees_while_it_plays() {
    use ca72_plugin::engine::DEADLINE;
    use std::time::{Duration, Instant};
    let _serial = start();
    let mut e = Engine::new();
    let mut c = Controls {
        poly: true,
        voices: 10,
        entropy: 0.7,
        spread: 0.9,
        ..Controls::default()
    };
    c.panel.ext_on = true;
    e.set(&c);
    e.prepare(RATE, 7);
    // (Ordinary threads: this plays as fast as it goes.)
    assert!(e.start_workers(3, None) > 0, "no workers");
    let period = Duration::from_secs_f64(BLOCK as f64 / RATE);
    let (mut l, mut r, mut ext) = ([0.0f32; BLOCK], [0.0f32; BLOCK], [0.0f32; BLOCK]);
    // (Blocks with POLY switched on again that its workers shared, once they came.)
    let (mut due, mut shared) = (false, 0);
    HERE.with(|h| h.set(true));
    ARMED.store(true, Ordering::SeqCst);
    let mut sink = 0.0f32;
    let mut at = 0usize;
    while at < 2 * RATE as usize {
        let b = at / BLOCK;
        e.set_deadline(Some(Instant::now() + period.mul_f64(DEADLINE)));
        if b > 150 && c.poly && e.workers() > 0 {
            shared += 1;
        }
        for (i, x) in ext.iter_mut().enumerate() {
            *x = if b % 70 == 3 && i == 5 {
                f32::INFINITY
            } else {
                0.3 * ((at + i) as f32 * 0.03).sin()
            };
        }
        if b.is_multiple_of(30) {
            for k in [24u8, 45, 52, 57, 61, 64, 69, 73, 76, 110] {
                e.event(Event::Note {
                    key: k + (b / 30 % 3) as u8,
                    on: true,
                });
            }
        }
        if b % 30 == 20 {
            for k in [24u8, 45, 52, 57, 61, 64] {
                e.event(Event::Note {
                    key: k + (b / 30 % 3) as u8,
                    on: false,
                });
            }
        }
        hush(&mut e, b);
        if b % 150 == 149 {
            c.poly = !c.poly;
        }
        c.voices = [10, 6, 3][b / 100 % 3];
        c.feedback = if b % 60 < 30 { 0.0 } else { 0.4 };
        c.panel.cutoff = 0.5 + 0.3 * (at as f64 * 1e-4).sin();
        e.set(&c);
        // (The block in two runs, a pitch bend between them.)
        e.render(&ext[..BLOCK / 2], &mut l[..BLOCK / 2], &mut r[..BLOCK / 2]);
        e.event(Event::PitchBend((at as f32 * 1e-4).sin()));
        e.render(&ext[BLOCK / 2..], &mut l[BLOCK / 2..], &mut r[BLOCK / 2..]);
        sink += l.iter().chain(&r).sum::<f32>();
        sink += e.end_block(BLOCK);
        serve(&mut e, b, &mut due);
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
        "POLY with workers allocated {allocs} and freed {frees} times while playing; the first:\n{first}"
    );
    assert!(shared > 0, "no workers once POLY was switched on again");
}

/// What the plug-in asks of its helper thread at a block's end (decisions.md R23), the
/// spares mended and POLY's workers served, asked again and again while the helper does it:
/// no allocation, no free on the asking thread.
#[test]
fn asking_the_helper_neither_allocates_nor_frees() {
    use ca72_plugin::helper::{Helper, MEND, SERVE};
    let _serial = start();
    let mut e = Engine::new();
    e.set(&Controls {
        poly: true,
        ..Controls::default()
    });
    e.prepare(RATE, 7);
    let helper = Helper::start(e.spares(), e.crew(), std::sync::Arc::default()).unwrap();
    HERE.with(|h| h.set(true));
    ARMED.store(true, Ordering::SeqCst);
    for k in 0..2_000u32 {
        helper.ask([MEND, SERVE, MEND | SERVE, 0][k as usize % 4]);
        std::hint::spin_loop();
    }
    ARMED.store(false, Ordering::SeqCst);
    drop(helper);
    let (allocs, frees) = (ALLOCS.load(Ordering::SeqCst), FREES.load(Ordering::SeqCst));
    let first = FIRST
        .lock()
        .ok()
        .and_then(|f| f.clone())
        .unwrap_or_default();
    assert_eq!(
        (allocs, frees),
        (0, 0),
        "asking the helper allocated {allocs} and freed {frees} times; the first:\n{first}"
    );
}

/// MIDI Learn on the audio thread (decisions.md R34), through the plug-in's own `process`: learned
/// controllers setting knobs (gliding in the voices), switches, a selector and VOICES through the
/// host, a controller caught while learning, the reserved ones refused, the host's automation
/// ending a glide: no allocation, no free. (The stand-in host's record of the changes is reserved
/// first, as the wrappers reserve theirs; its events are pushed off the count.)
#[test]
fn midi_learn_neither_allocates_nor_frees() {
    use ca72_plugin::Ca72;
    use ca72_plugin::learn::{Cc, index};
    use nih_plug::context::process::TestProcessContext;
    use nih_plug::prelude::*;
    use std::sync::Arc;

    struct Init;
    impl InitContext<Ca72> for Init {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Clap
        }
        fn execute(&self, _: ()) {}
        fn set_latency_samples(&self, _: u32) {}
        fn set_current_voice_capacity(&self, _: u32) {}
    }

    let _serial = start();
    let mut p = Ca72::default();
    let config = BufferConfig {
        sample_rate: RATE as f32,
        min_buffer_size: None,
        max_buffer_size: BLOCK as u32,
        process_mode: ProcessMode::Realtime,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    let params = p.params();
    let map = Arc::clone(&p.parameters().midi_map);
    let learned = |id: &str, cc: u8| {
        map.assign(index(id).unwrap(), Cc { channel: 0, cc })
            .unwrap();
    };
    learned("cutoff", 74);
    learned("emphasis", 71);
    learned("osc2_on", 20);
    learned("osc1_range", 21);
    learned("voices", 22);
    let mut c = TestProcessContext::<Ca72>::new(params, RATE as f32, ProcessMode::Realtime);
    c.reported.reserve(1 << 16);
    let cutoff = p.parameters().cutoff.as_ptr();
    let (mut l, mut r) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
    let mut buffer = Buffer::default();
    // SAFETY: both slices are `BLOCK` long and live as long as the buffer.
    unsafe {
        buffer.set_slices(BLOCK, |s| {
            s.clear();
            s.push(&mut l);
            s.push(&mut r);
        });
    }
    let mut aux = AuxiliaryBuffers {
        inputs: &mut [],
        outputs: &mut [],
    };
    let cc = |timing: u32, cc: u8, value: f32| NoteEvent::MidiCC {
        timing,
        channel: 0,
        cc,
        value,
    };
    HERE.with(|h| h.set(true));
    for b in 0..400usize {
        // (The host's queue filled off the count.)
        ARMED.store(false, Ordering::SeqCst);
        if b.is_multiple_of(50) {
            c.push_event(NoteEvent::NoteOn {
                timing: 0,
                voice_id: None,
                channel: 0,
                note: 45 + (b / 50 % 12) as u8,
                velocity: 1.0,
            });
        }
        for k in 0..4u32 {
            let v = ((b as f32 * 4.0 + k as f32) * 0.07).sin() * 0.5 + 0.5;
            c.push_event(cc(k * 30, 74, v));
            c.push_event(cc(k * 30 + 3, 71, 1.0 - v));
        }
        c.push_event(cc(5, 20, if b % 6 < 3 { 0.0 } else { 1.0 }));
        c.push_event(cc(6, 21, (b % 7) as f32 / 6.0));
        c.push_event(cc(7, 22, (b % 9) as f32 / 8.0));
        c.push_event(cc(9, 1, 0.5));
        c.push_event(cc(9, 100, 0.5));
        if b % 40 == 10 {
            map.arm(index("glide").unwrap());
            c.push_event(cc(11, 30 + (b / 40) as u8, 0.5));
        }
        if b % 40 == 11 {
            map.poll();
        }
        if b % 30 == 29 {
            c.automate(cutoff, 0.25);
        }
        ARMED.store(true, Ordering::SeqCst);
        p.process(&mut buffer, &mut aux, &mut c);
    }
    ARMED.store(false, Ordering::SeqCst);
    let (allocs, frees) = (ALLOCS.load(Ordering::SeqCst), FREES.load(Ordering::SeqCst));
    let first = FIRST
        .lock()
        .ok()
        .and_then(|f| f.clone())
        .unwrap_or_default();
    assert_eq!(
        (allocs, frees),
        (0, 0),
        "MIDI Learn allocated {allocs} and freed {frees} times while playing; the first:\n{first}"
    );
    assert!(c.reported.len() > 1000, "{} changes told", c.reported.len());
}
