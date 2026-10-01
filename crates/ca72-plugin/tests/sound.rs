//! The engine plays as the instrument does: the default parameters are the default panel, a
//! key sounds at its pitch, MIDI's full bend moves it by MIDI BEND RANGE, the side chain
//! reaches the output through EXTERNAL INPUT, and a key held through a change of sample rate
//! plays on.

#![allow(clippy::unwrap_used)]

use ca72::voice::Panel;
use ca72_analysis::pitch::pitch;
use ca72_plugin::engine::{Controls, Engine, Event, output_gain};
use ca72_plugin::params::Ca72Params;

const RATE: f64 = 48_000.0;

fn ready(c: &Controls) -> Engine {
    let mut e = Engine::new();
    e.set(c);
    e.prepare(RATE, 1);
    e
}

/// `secs` of output, `ext` at the side chain.
fn run(e: &mut Engine, secs: f64, ext: impl Fn(f64) -> f32) -> Vec<f64> {
    (0..(secs * RATE) as usize)
        .map(|i| f64::from(e.tick(ext(i as f64 / RATE))))
        .collect()
}

fn rms(x: &[f64]) -> f64 {
    (x.iter().map(|y| y * y).sum::<f64>() / x.len() as f64).sqrt()
}

fn cents(hz: f64, want: f64) -> f64 {
    1200.0 * (hz / want).log2()
}

/// The pitch of a key held for a second, its attack left out.
fn pitch_of(e: &mut Engine, key: u8) -> f64 {
    e.event(Event::Note { key, on: true });
    let y = run(e, 1.0, |_| 0.0);
    pitch(&y[(0.3 * RATE) as usize..], RATE as u32)
        .median_hz
        .unwrap()
}

#[test]
fn the_default_parameters_are_the_default_panel() {
    let c = Ca72Params::default().controls();
    assert_eq!(c.panel, Panel::default());
    assert_eq!((c.volume, c.main_output, c.bend_range), (1.0, true, 2.0));
}

#[test]
fn main_output_volume_follows_its_taper() {
    assert_eq!(output_gain(1.0, true), 1.0);
    assert_eq!(output_gain(0.0, true), 0.0);
    assert_eq!(output_gain(1.0, false), 0.0);
    let half = output_gain(0.5, true);
    assert!(half > 0.05 && half < 0.2, "{half}");
}

#[test]
fn a_key_sounds_at_its_pitch() {
    let mut e = ready(&Controls::default());
    // The A below middle C, on 8': 220 Hz.
    let hz = pitch_of(&mut e, 57);
    assert!(cents(hz, 220.0).abs() < 10.0, "{hz} Hz");
}

#[test]
fn below_its_lowest_rate_the_voice_runs_at_a_multiple_of_the_hosts() {
    // (Where the model alone gives NaN: clap-validator's lowest rate.)
    let rate = 1234.5678;
    let mut e = Engine::new();
    e.set(&Controls::default());
    e.prepare(rate, 1);
    e.event(Event::Note { key: 57, on: true });
    let y: Vec<f64> = (0..(2.0 * rate) as usize)
        .map(|_| f64::from(e.tick(0.0)))
        .collect();
    assert!(y.iter().all(|y| y.is_finite()));
    assert!(rms(&y[(0.3 * rate) as usize..]) > 0.01);
}

#[test]
fn midi_bend_moves_the_pitch_by_its_range() {
    for (range, semitones) in [(2.0, 2.0), (5.0, 5.0)] {
        let c = Controls {
            bend_range: range,
            ..Controls::default()
        };
        let mut e = ready(&c);
        e.event(Event::PitchBend(1.0));
        let up = pitch_of(&mut e, 57);
        let want = 220.0 * 2f64.powf(semitones / 12.0);
        assert!(
            cents(up, want).abs() < 15.0,
            "range {range}: {up} Hz, wanted {want}"
        );
    }
}

#[test]
fn the_side_chain_reaches_the_output_through_external_input() {
    let tau = 2.0 * std::f64::consts::PI;
    let tone = |t: f64| (0.5 * (tau * 440.0 * t).sin()) as f32;
    let mut on = Controls::default();
    on.panel.osc[0].on = false;
    on.panel.ext_on = true;
    let mut off = on;
    off.panel.ext_on = false;
    let level = |c: &Controls| {
        let mut e = ready(c);
        e.event(Event::Note { key: 57, on: true });
        rms(&run(&mut e, 0.5, tone)[(0.2 * RATE) as usize..])
    };
    let (with, without) = (level(&on), level(&off));
    assert!(with > 0.01, "the side chain at {with}");
    assert!(without < with * 1e-2, "{without} with EXTERNAL INPUT off");
}

#[test]
fn silent_until_prepared_and_a_held_key_plays_on_at_a_new_rate() {
    let mut e = Engine::new();
    e.set(&Controls::default());
    e.event(Event::Note { key: 57, on: true });
    assert!(run(&mut e, 0.01, |_| 0.0).iter().all(|y| *y == 0.0));
    e.prepare(44_100.0, 1);
    e.prepare(RATE, 1);
    let y = run(&mut e, 0.5, |_| 0.0);
    assert!(rms(&y[(0.2 * RATE) as usize..]) > 0.01);
}

#[test]
fn reset_controllers_returns_the_wheels() {
    let mut e = Engine::new();
    e.event(Event::PitchBend(1.0));
    e.event(Event::Modulation(0.5));
    let (bend, modulation) = e.wheels();
    assert!((bend - 2.0 / ca72::modulation::PITCH_WHEEL_SEMITONES).abs() < 1e-9);
    assert_eq!(modulation, 0.5);
    e.event(Event::ResetControllers);
    assert_eq!(e.wheels(), (0.0, 0.0));
}
