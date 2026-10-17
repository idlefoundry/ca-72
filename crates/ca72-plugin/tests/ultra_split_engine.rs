//! ULTRA's split voices in the engine (decisions.md R-THREADS) give the same samples, to the
//! bit, as every voice whole: POLY's voices, with DOUBLE's twins, UNISON's, and the one
//! instrument (scattering, and with DOUBLE), each with its chords changing inside blocks of
//! 256 frames, with crews for all of them; and with fewer crews than voices, every voice
//! whole. The crews' helpers here are ordinary threads (the engine unpaced).

#![allow(clippy::unwrap_used)]

mod common;

use ca72_plugin::engine::{Controls, Engine, set_split_for_measurement};
use common::{Chords, play_block};

/// `seconds` of chords of `notes` keys under `c` at ULTRA, DRIVE on; with `crews` split crews
/// (None: no threads at all, every voice whole on this one). Returns the samples and how many
/// runs played split.
fn render(c: Controls, notes: usize, crews: Option<usize>, seconds: f64) -> (Vec<f32>, u64) {
    let mut e = Engine::new();
    e.set(&c);
    e.prepare(48_000.0, 1);
    if let Some(n) = crews {
        set_split_for_measurement(Some(n));
        e.start_workers(2, None);
    }
    let mut chords = Chords::new(notes, 48_000.0);
    let (mut l, mut r) = ([0.0f32; 256], [0.0f32; 256]);
    let mut events = Vec::with_capacity(64);
    let mut out = Vec::new();
    for i in 0..(seconds * 48_000.0 / 256.0) as usize {
        play_block(&mut e, &mut chords, i * 256, &mut l, &mut r, &mut events);
        out.extend(l.iter().chain(&r));
    }
    let (runs, held) = e.split_runs();
    if let Some(n) = crews {
        assert_eq!(held, n, "the crews started");
    }
    (out, runs)
}

fn ultra(poly: bool, unison: bool, voices: usize, double: f64) -> Controls {
    let mut c = Controls {
        poly,
        unison,
        voices,
        double,
        ultra: true,
        drive: 6.0,
        entropy: 0.4,
        spread: 0.6,
        ..Controls::default()
    };
    c.panel.osc[1].on = true;
    c.panel.osc[1].freq = 0.52;
    c.panel.emphasis = 0.5;
    c.panel.glide_on = true;
    c.panel.glide = 0.1;
    c
}

#[test]
fn split_voices_are_the_whole_voices_to_the_bit() {
    // (Short: ULTRA is slow, the more so in a debug build.)
    let seconds = if cfg!(debug_assertions) { 0.15 } else { 0.6 };
    for (name, c, notes, crews) in [
        ("POLY", ultra(true, false, 2, 0.0), 2, 2),
        ("POLY with DOUBLE", ultra(true, false, 2, 0.4), 2, 2),
        ("UNISON", ultra(false, true, 2, 0.0), 1, 2),
        ("the one instrument", ultra(false, false, 2, 0.0), 1, 1),
        (
            "the one instrument with DOUBLE",
            ultra(false, false, 2, 0.4),
            1,
            1,
        ),
    ] {
        let (whole, none) = render(c, notes, None, seconds);
        assert_eq!(none, 0);
        let (split, runs) = render(c, notes, Some(crews), seconds);
        assert!(whole.iter().any(|x| x.abs() > 0.01), "{name}: silent");
        assert!(runs > 0, "{name}: nothing played split");
        let first = whole
            .iter()
            .zip(&split)
            .position(|(a, b)| a.to_bits() != b.to_bits());
        assert_eq!(first, None, "{name}: the split samples differ");
    }
    // Fewer crews than sounding voices: every voice whole, the same samples.
    let c = ultra(true, false, 3, 0.0);
    let (whole, _) = render(c, 3, None, seconds);
    let (fewer, runs) = render(c, 3, Some(1), seconds);
    assert!(
        whole
            .iter()
            .zip(&fewer)
            .all(|(a, b)| a.to_bits() == b.to_bits())
    );
    let _ = runs;
    set_split_for_measurement(None);
}
