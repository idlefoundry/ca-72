//! The standalone's TEMPO moved by a modulation source (decisions.md R-LFO; the owner,
//! 2026-10-20: "in standalone mode, lfo or macro should also be able to control the BPM, I
//! think"), through the plug-in's own `process`: the arpeggiator, a timed LFO and oscillator 3
//! timed following the beats the moving tempo adds up, without a jump; a timed LFO moving the
//! tempo its own clock keeps (a loop) bounded for ten minutes; the same steps at any block; and
//! nothing of it in a plug-in.

use super::*;
use crate::lfo::{Assignment, RATE_RANGE, Source, Target};
use crate::params::{ArpRate, Division, Footage, QualityMode};
use crate::tests::{block, initialised_with};
use nih_plug::context::process::TestProcessContext;

const RATE: f32 = 48_000.0;

/// At LO (quick in debug), the arpeggiator on at sixteenths over two octaves, oscillator 3 timed
/// at a quarter note (RANGE LO, heard), MONO.
fn arp_params() -> Ca72Params {
    Ca72Params {
        // (One noise for every run, so that two compare to the bit.)
        noise_seed: Arc::new(std::sync::atomic::AtomicU64::new(0xca72)),
        quality: EnumParam::new("Quality", QualityMode::Lo),
        arp_on: BoolParam::new("On", true),
        arp_rate: EnumParam::new("Rate", ArpRate::Sixteenth),
        osc3_timed: BoolParam::new("Timed", true),
        osc3_division: EnumParam::new("Division", Division::Quarter),
        osc3_range: EnumParam::new("Range", Footage::Lo),
        osc3_on: BoolParam::new("On", true),
        ..Ca72Params::default()
    }
}

fn context(p: &Ca72) -> TestProcessContext<Ca72> {
    let params: Arc<dyn Params> = p.params.clone();
    TestProcessContext::new(params, RATE, ProcessMode::Offline)
}

fn key(note: u8, on: bool) -> NoteEvent<()> {
    if on {
        NoteEvent::NoteOn {
            timing: 0,
            voice_id: None,
            channel: 0,
            note,
            velocity: 1.0,
        }
    } else {
        NoteEvent::NoteOff {
            timing: 0,
            voice_id: None,
            channel: 0,
            note,
            velocity: 0.0,
        }
    }
}

/// `p`'s LFO `n` free at `hz`, or timed at a quarter note.
fn lfo_at(p: &Ca72, c: &TestProcessContext<Ca72>, n: usize, hz: Option<f32>) {
    let q = p.params.lfo(n);
    match hz {
        Some(hz) => {
            assert!(c.automate(q.rate.as_ptr(), RATE_RANGE.normalize(hz)));
            assert!(c.automate(q.timed.as_ptr(), 0.0));
        }
        None => {
            assert!(c.automate(q.timed.as_ptr(), 1.0));
            let d = q.division.preview_normalized(Division::Quarter);
            assert!(c.automate(q.division.as_ptr(), d));
        }
    }
}

/// The plug-in made with `params` and its context: LFO 1 a free triangle at 0.5 Hz on TEMPO
/// (+25 %: 120 BPM ±67.5), LFO 2 timed at a quarter note, FREE, on CUTOFF (+10 %); two keys held
/// for the arpeggiator. The standalone's unless `api` says otherwise.
fn moving(api: PluginApi) -> (Ca72, TestProcessContext<Ca72>) {
    let mut p = initialised_with(arp_params(), RATE, ProcessMode::Offline);
    let mut c = context(&p);
    c.set_plugin_api(api);
    lfo_at(&p, &c, 0, Some(0.5));
    lfo_at(&p, &c, 1, None);
    let patch = &p.params.lfo_patch;
    patch.assign(Source::Lfo(0), Target::Tempo, 0.25).unwrap();
    patch.assign(Source::Lfo(1), Target::Cutoff, 0.1).unwrap();
    c.push_event(key(48, true));
    c.push_event(key(55, true));
    let _ = block(&mut p, &mut c, 1);
    (p, c)
}

/// What one sample's block leaves: the beat it began at (the arpeggiator's clock), the tempo in
/// force, LFO 2's phase, the key sounding, oscillator 3's resets so far.
#[derive(Clone, Copy, Debug)]
struct Seen {
    beat: f64,
    per: f64,
    tempo: Option<f64>,
    phase: f64,
    key: Option<u8>,
    resets: u64,
}

fn seen(p: &Ca72) -> Seen {
    let (beat, per) = p.arp.position();
    Seen {
        beat,
        per,
        tempo: p.engine.tempo(),
        phase: f64::from(p.engine.lfo_readings()[1].phase),
        key: p.arp.sounding(),
        resets: p.engine.osc_reset(2).map_or(0, |r| r.1),
    }
}

/// Two seconds a sample at a time (each block's start is each sample: the clock read at every
/// one), then as `blocks` of `len`.
fn played(p: &mut Ca72, c: &mut TestProcessContext<Ca72>, samples: usize) -> Vec<Seen> {
    (0..samples)
        .map(|_| {
            let y = block(p, c, 1);
            assert!(y.iter().all(|v| v.is_finite()));
            seen(p)
        })
        .collect()
}

/// The tempo moved by a triangle LFO (0.5 Hz, SOFT, 52.5 to 187.5 BPM): it moves smoothly, and
/// the beats it adds up go on from each sample to the next at the tempo in force, never jumping;
/// the timed LFO's phase is the beat's fraction at every sample (its clock is the beats, not the
/// tempo); each arpeggiator step lands on its grid of sixteenths; oscillator 3 timed resets on
/// the quarter notes; and the steps come closer and further apart with the tempo, smoothly.
#[test]
fn a_moving_tempo_moves_the_arp_and_the_timed_lfos_and_oscillators_without_a_jump() {
    let (mut p, mut c) = moving(PluginApi::Standalone);
    let s = played(&mut p, &mut c, 2 * RATE as usize);
    let tempos: Vec<f64> = s.iter().filter_map(|x| x.tempo).collect();
    assert_eq!(tempos.len(), s.len(), "moved from the first sample on");
    let (lo, hi) = tempos
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &t| (a.min(t), b.max(t)));
    assert!(lo >= 52.0 && hi <= 188.0 && hi - lo > 100.0, "{lo}..{hi}");
    // (135 BPM a second at most: 0.07 a tick. From the second tick: the first reads the LFO
    // before its first tick, as an LFO's knobs do, R-LFO's addendum.)
    let worst = tempos[48..]
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f64::max);
    assert!(worst < 0.1, "the tempo jumped {worst} BPM");
    for (k, w) in s.windows(2).enumerate() {
        let step = w[1].beat - w[0].beat;
        // (Each sample's beats at the tempo in force over it: no jump, nothing lost.)
        assert!(
            (step - w[0].per).abs() < 1e-9 || (step - w[1].per).abs() < 1e-9,
            "sample {k}: {step} beats, {} or {} a sample",
            w[0].per,
            w[1].per
        );
        // (The tempo read after a sample is the one in force for the next.)
        let bpm = w[1].per * 60.0 * f64::from(RATE);
        assert!((bpm - w[0].tempo.unwrap_or(0.0)).abs() < 1e-6, "sample {k}");
    }
    for (k, x) in s.iter().enumerate() {
        let d = (x.phase - x.beat.rem_euclid(1.0)).abs();
        assert!(
            d.min(1.0 - d) < 2e-5,
            "sample {k}: LFO 2 at {}, the beat {}",
            x.phase,
            x.beat
        );
    }
    // The arpeggiator's steps: each where its sixteenth falls (the first sample on or after it),
    // on the grid from the first seen (itself within a sample of its own).
    let steps: Vec<usize> = (1..s.len())
        .filter(|&k| s[k].key.is_some() && s[k].key != s[k - 1].key)
        .collect();
    assert!(steps.len() > 12, "{} steps", steps.len());
    let anchor = s[steps[0]].beat;
    for &k in &steps {
        let x = (s[k].beat - anchor) / 0.25;
        let off = x - x.round();
        assert!(
            off.abs() <= 2.0 * s[k - 1].per / 0.25,
            "step at {k}: {off} of a step from its sixteenth"
        );
    }
    let gaps: Vec<f64> = steps.windows(2).map(|w| (w[1] - w[0]) as f64).collect();
    let (short, long) = gaps
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &g| (a.min(g), b.max(g)));
    assert!(
        long > short * 1.8,
        "the steps follow the tempo: {short}..{long}"
    );
    // (Each step as long as a sixteenth at the tempos in force over it: its mean tempo between
    // their least and their most.)
    for w in steps.windows(2) {
        let mean = 0.25 / (w[1] - w[0]) as f64 * 60.0 * f64::from(RATE);
        let over = &tempos[w[0]..w[1]];
        let (a, b) = over
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &t| (a.min(t), b.max(t)));
        assert!(
            (a - 0.5..=b + 0.5).contains(&mean),
            "steps {w:?}: {mean} BPM, the tempo {a}..{b}"
        );
    }
    // Oscillator 3 timed: a reset at each quarter note.
    let resets: Vec<usize> = (1..s.len())
        .filter(|&k| s[k].resets > s[k - 1].resets)
        .collect();
    assert!(resets.len() >= 3, "{} resets", resets.len());
    for &k in &resets {
        let f = s[k].beat.rem_euclid(1.0);
        assert!(
            f.min(1.0 - f) < 3.0 * s[k].per,
            "reset at {k}: {f} of a quarter from the beat"
        );
    }
}

/// The same patch in blocks of 256: the arpeggiator's steps in the blocks the sample-at-a-time
/// run has them in, and the beats at the end the same within rounding (the tempo changes on
/// the LFOs' ticks, fixed in time, whatever the blocks).
#[test]
fn a_moving_tempo_steps_the_same_in_any_block() {
    let samples = RATE as usize;
    let (mut p, mut c) = moving(PluginApi::Standalone);
    let one = played(&mut p, &mut c, samples);
    let (mut q, mut d) = moving(PluginApi::Standalone);
    let len = 256;
    let mut blocks = Vec::new();
    for _ in 0..samples / len {
        let _ = block(&mut q, &mut d, len);
        blocks.push(seen(&q));
    }
    // (Where each block's step is: the block's last sample's key against the block before's.)
    let stepped = |keys: &[Option<u8>]| -> Vec<usize> {
        (1..keys.len())
            .filter(|&k| keys[k].is_some() && keys[k] != keys[k - 1])
            .collect()
    };
    let by_sample: Vec<Option<u8>> = (0..blocks.len())
        .map(|b| one[(b + 1) * len - 1].key)
        .collect();
    let by_block: Vec<Option<u8>> = blocks.iter().map(|x| x.key).collect();
    assert_eq!(stepped(&by_sample), stepped(&by_block));
    // (A block's place as the samples after its last change of tempo have it: its end there.)
    let (a, b) = (one[blocks.len() * len - 1], blocks[blocks.len() - 1]);
    let (by_sample, by_block) = (a.beat + a.per, b.beat + len as f64 * b.per);
    assert!(
        (by_sample - by_block).abs() < 1e-6,
        "{by_sample} and {by_block} beats"
    );
}

/// A timed LFO moving the tempo its own clock keeps (a loop through its own cycle, option B:
/// the tempo of each tick from the LFO at the tick before): ten minutes on, the tempo always
/// within TEMPO's range and still moving, the beats adding up, every value finite.
#[test]
fn a_timed_lfo_on_its_own_tempo_stays_bounded_for_ten_minutes() {
    let mut p = initialised_with(arp_params(), RATE, ProcessMode::Offline);
    let mut c = context(&p);
    lfo_at(&p, &c, 0, None);
    p.params
        .lfo_patch
        .assign(Source::Lfo(0), Target::Tempo, 0.6)
        .unwrap();
    let len = 4096;
    let blocks = (600.0 * RATE) as usize / len;
    let (range, mut last) = (crate::params::TEMPO_RANGE, f64::MIN);
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for b in 0..blocks {
        // (A phrase now and then: the arpeggiator plays at the moving tempo.)
        if b % 400 == 0 {
            c.push_event(key(60, true));
        }
        if b % 400 == 20 {
            c.push_event(key(60, false));
        }
        let y = block(&mut p, &mut c, len);
        assert!(y.iter().all(|v| v.is_finite()), "block {b}");
        let t = p.engine.tempo().expect("moved");
        assert!((range.0..=range.1).contains(&t), "block {b}: {t} BPM");
        let (beat, per) = p.arp.position();
        assert!(beat.is_finite() && beat > last && per > 0.0, "block {b}");
        last = beat;
        let v = p.engine.lfo_readings()[0].value;
        assert!(v.is_finite() && v.abs() <= 1.0, "block {b}: {v}");
        if b > blocks / 2 {
            (lo, hi) = (lo.min(t), hi.max(t));
        }
    }
    assert!(hi - lo > 50.0, "the tempo still moving: {lo}..{hi}");
}

/// In a plug-in the host gives the tempo: an assignment on TEMPO is kept in the patch but never
/// applied, the engine as if it were not there, to the bit, the arpeggiator at the tempo as
/// before.
#[test]
fn a_plug_in_keeps_a_tempo_assignment_but_never_applies_it() {
    let render = |with: bool| {
        let (mut p, mut c) = moving(PluginApi::Clap);
        if !with {
            p.params.lfo_patch.remove(Source::Lfo(0), Target::Tempo);
        }
        let mut out = Vec::new();
        for _ in 0..200 {
            out.extend(block(&mut p, &mut c, 256));
            assert_eq!(p.engine.tempo(), None);
            assert_eq!(p.arp.position().1, 120.0 / 60.0 / f64::from(RATE));
        }
        assert_eq!(
            p.params.lfo_patch.depth(Source::Lfo(0), Target::Tempo),
            with.then_some(0.25)
        );
        assert!(!lfo::target_of(&p.params, "tempo").is_some());
        assert!(
            !p.params.controls().lfo.to[Source::Lfo(0).index()]
                .iter()
                .flatten()
                .any(|a: &Assignment| a.target == Target::Tempo)
        );
        out
    };
    let (with, without) = (render(true), render(false));
    assert!(
        with.iter()
            .zip(&without)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "the same to the bit"
    );
}

/// A macro on TEMPO: its value moves the tempo by its depth, one way (macros are unipolar), and
/// TEMPO's own setting is where it starts.
#[test]
fn a_macro_moves_the_tempo_by_its_share_of_temps_range() {
    let mut p = initialised_with(arp_params(), RATE, ProcessMode::Offline);
    let mut c = context(&p);
    p.params.set_tempo(100.0);
    p.params
        .lfo_patch
        .assign(Source::Macro(0), Target::Tempo, 0.5)
        .unwrap();
    assert!(c.automate(p.params.macro_param(0).as_ptr(), 0.5));
    for _ in 0..40 {
        let _ = block(&mut p, &mut c, 256);
    }
    // 100 BPM + 50 % × 50 % of 270: 167.5.
    let t = p.engine.tempo().expect("moved");
    assert!((t - 167.5).abs() < 1e-4, "{t}");
    assert!((p.arp.position().1 * 60.0 * f64::from(RATE) - t).abs() < 1e-9);
    // (Clamped: past 300 it stays there.)
    assert!(c.automate(p.params.macro_param(0).as_ptr(), 1.0));
    p.params.set_tempo(280.0);
    for _ in 0..40 {
        let _ = block(&mut p, &mut c, 256);
    }
    assert_eq!(p.engine.tempo(), Some(300.0));
}
