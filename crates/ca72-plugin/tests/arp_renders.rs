//! Renders of the arpeggiator's changes of 2026-10-19 for listening (decisions.md R-ARP),
//! through the plug-in's own process as the standalone runs it (the transport stopped, the
//! tempo its TEMPO's), each measured as it is made: `CA72_ARP_RENDERS=<folder> cargo test
//! --release -p ca72-plugin --test arp_renders -- --ignored --nocapture --test-threads 1`.
//! Each WAV is 48 kHz stereo floats. What it prints was measured; nobody listened. The GATE
//! cases and `with_hold_off_it_renders_as_before` run on the build before the changes too.

#![allow(unsafe_code, clippy::unwrap_used)]

use std::path::PathBuf;

use ca72_plugin::Ca72;
use nih_plug::context::process::TestProcessContext;
use nih_plug::prelude::*;

const RATE: f64 = 48_000.0;
const BLOCK: usize = 256;

struct Init;
impl InitContext<Ca72> for Init {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Standalone
    }
    fn execute(&self, _: ()) {}
    fn set_latency_samples(&self, _: u32) {}
    fn set_current_voice_capacity(&self, _: u32) {}
}

/// What happens at a sample: a key pressed or let go, a parameter set (normalized; at the
/// block's start that holds the sample), the standalone's TEMPO set.
#[derive(Clone, Copy, Debug)]
enum Ev {
    Key(usize, u8, bool),
    Set(usize, &'static str, f32),
    Tempo(usize, f64),
}

/// `secs` of an instance's output (left, right), as the standalone opens it (its noise seeded
/// the same each time), `set` its parameters (id, normalized) first, the events at their
/// samples.
fn render(set: &[(&str, f32)], secs: f64, events: &[Ev]) -> (Vec<f32>, Vec<f32>) {
    let mut p = Ca72::default();
    p.parameters()
        .noise_seed
        .store(0xca72, std::sync::atomic::Ordering::Relaxed);
    let config = BufferConfig {
        sample_rate: RATE as f32,
        min_buffer_size: None,
        max_buffer_size: BLOCK as u32,
        process_mode: ProcessMode::Realtime,
    };
    assert!(p.initialize(&Ca72::AUDIO_IO_LAYOUTS[0], &config, &mut Init));
    let mut c = TestProcessContext::<Ca72>::new(p.params(), RATE as f32, ProcessMode::Realtime);
    let map = p.params().param_map();
    let ptr = |id: &str| {
        map.iter()
            .find(|(i, ..)| i == id)
            .unwrap_or_else(|| panic!("{id}"))
            .1
    };
    for (id, v) in set {
        assert!(c.automate(ptr(id), *v), "{id}");
    }
    let n = (secs * RATE) as usize;
    let (mut left, mut right) = (Vec::with_capacity(n), Vec::with_capacity(n));
    let mut at = 0;
    while at < n {
        let len = BLOCK.min(n - at);
        for e in events {
            match *e {
                Ev::Key(s, note, on) if (at..at + len).contains(&s) => {
                    let timing = (s - at) as u32;
                    c.push_event(if on {
                        NoteEvent::NoteOn {
                            timing,
                            voice_id: None,
                            channel: 0,
                            note,
                            velocity: 1.0,
                        }
                    } else {
                        NoteEvent::NoteOff {
                            timing,
                            voice_id: None,
                            channel: 0,
                            note,
                            velocity: 0.0,
                        }
                    });
                }
                Ev::Set(s, id, v) if (at..at + len).contains(&s) => {
                    assert!(c.automate(ptr(id), v));
                }
                Ev::Tempo(s, bpm) if (at..at + len).contains(&s) => p.parameters().set_tempo(bpm),
                _ => {}
            }
        }
        let (mut l, mut r) = (vec![0.0f32; len], vec![0.0f32; len]);
        let mut buffer = Buffer::default();
        // SAFETY: both slices are `len` long and live until `process` has returned.
        unsafe {
            buffer.set_slices(len, |s| {
                s.clear();
                s.push(&mut l);
                s.push(&mut r);
            });
        }
        let mut aux = AuxiliaryBuffers {
            inputs: &mut [],
            outputs: &mut [],
        };
        p.process(&mut buffer, &mut aux, &mut c);
        drop(buffer);
        left.extend(l);
        right.extend(r);
        at += len;
    }
    (left, right)
}

/// The audio written as `name`.wav to `CA72_ARP_RENDERS`, if it is set.
fn save(name: &str, (l, r): &(Vec<f32>, Vec<f32>)) {
    let Some(d) = std::env::var_os("CA72_ARP_RENDERS").map(PathBuf::from) else {
        return;
    };
    std::fs::create_dir_all(&d).unwrap();
    let data = (l.len() * 8) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&(RATE as u32).to_le_bytes());
    b.extend_from_slice(&(RATE as u32 * 8).to_le_bytes());
    b.extend_from_slice(&8u16.to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for (x, y) in l.iter().zip(r) {
        b.extend_from_slice(&x.to_le_bytes());
        b.extend_from_slice(&y.to_le_bytes());
    }
    std::fs::write(d.join(format!("{name}.wav")), b).unwrap();
}

fn db(x: f64) -> f64 {
    20.0 * x.max(1e-12).log10()
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / x.len().max(1) as f64).sqrt()
}

fn peak(x: &[f32]) -> f64 {
    x.iter().fold(0.0, |m, v| m.max(f64::from(v.abs())))
}

fn secs(t: f64) -> usize {
    (t * RATE) as usize
}

/// The `n` steps `step` seconds long from the start.
fn steps(x: &[f32], step: f64, n: usize) -> Vec<&[f32]> {
    (0..n)
        .map(|k| {
            let b = secs((k + 1) as f64 * step).min(x.len());
            &x[secs(k as f64 * step).min(b)..b]
        })
        .collect()
}

/// A step sounds: its peak above -60 dBFS.
fn sounds(x: &[f32]) -> bool {
    db(peak(x)) > -60.0
}

/// A step's attack: its envelope (3 ms RMS, a millisecond apart) rises after the trigger's
/// 7.8 ms to at least twice (6 dB) the least it was before it, and above -60 dBFS.
fn attacked(s: &[f32]) -> bool {
    let ms = secs(0.001);
    if s.len() < 31 * ms {
        return false;
    }
    let env = |k: usize| rms(&s[k * ms..(k + 3) * ms]);
    let before = (0..6).map(env).fold(f64::INFINITY, f64::min);
    let after = (8..27).map(env).fold(0.0, f64::max);
    after > 2.0 * before && db(after) > -60.0
}

/// A percussive loudness contour (sustain 0, a quick decay), so that each attack is heard as
/// its own and a note whose contours were not started again decays away.
const PLUCK: [(&str, f32); 3] = [
    ("loudness_attack", 0.0),
    ("loudness_decay", 0.1),
    ("loudness_sustain", 0.0),
];

/// RATE's `k`th of 13 (12: 1/32, 9: 1/16, 6: 1/8), normalized; its step at `bpm`, seconds.
fn rate(k: usize) -> f32 {
    k as f32 / 12.0
}
fn step_of(k: usize, bpm: f64) -> f64 {
    ca72_plugin::arp::RATES[k].1 * 60.0 / bpm
}

fn held(at: usize, keys: &[u8]) -> Vec<Ev> {
    keys.iter().map(|&k| Ev::Key(at, k, true)).collect()
}

const C_MAJOR: [u8; 3] = [60, 64, 67];

/// GATE (the owner's change 5): 1/32 at 120 with GATE 12 % (the case that was silent); GATE 0
/// % at every RATE at 120 and at 1/32 at 300; GATE 90 % at 1/32 in MONO (single triggering),
/// MONO with RETRIGGER, and POLY, its attacks counted with a plucked contour.
#[test]
#[ignore = "renders for listening, run by hand"]
fn gate() {
    let tag = std::env::var("CA72_ARP_TAG").unwrap_or_else(|_| "after".into());
    let (step, n) = (step_of(12, 120.0), 32);
    let a = render(
        &[("arp_on", 1.0), ("arp_rate", rate(12)), ("arp_gate", 0.12)],
        n as f64 * step + 0.3,
        &held(0, &C_MAJOR),
    );
    save(&format!("gate-1-32-120bpm-gate12-{tag}"), &a);
    let st = steps(&a.0, step, n);
    println!(
        "[{tag}] GATE 12 %, 1/32, 120 BPM: {}/{n} steps sound; RMS {:.1} dBFS; loudest step {:.1} dBFS",
        st.iter().filter(|s| sounds(s)).count(),
        db(rms(&a.0[..secs(n as f64 * step)])),
        st.iter()
            .map(|s| db(peak(s)))
            .fold(f64::NEG_INFINITY, f64::max)
    );
    let mut line = format!("[{tag}] GATE 0 %, 120 BPM, steps sounding by RATE:");
    for k in 0..13 {
        let step = step_of(k, 120.0);
        let n = ((2.0 / step).floor() as usize).max(4);
        let a = render(
            &[("arp_on", 1.0), ("arp_rate", rate(k)), ("arp_gate", 0.0)],
            n as f64 * step + 0.2,
            &held(0, &C_MAJOR),
        );
        let s = steps(&a.0, step, n).iter().filter(|s| sounds(s)).count();
        line += &format!(" {}={s}/{n}", ca72_plugin::arp::RATES[k].0);
        if k == 12 || k == 9 || k == 3 {
            save(
                &format!("gate0-{}-120bpm-{tag}", ca72_plugin::arp::RATES[k].0),
                &a,
            );
        }
    }
    println!("{line}");
    let (step, n) = (step_of(12, 300.0), 40);
    let a = render(
        &[("arp_on", 1.0), ("arp_rate", rate(12)), ("arp_gate", 0.0)],
        n as f64 * step + 0.2,
        &[vec![Ev::Tempo(0, 300.0)], held(0, &C_MAJOR)].concat(),
    );
    save(&format!("gate0-32-300bpm-{tag}"), &a);
    println!(
        "[{tag}] GATE 0 %, 1/32, 300 BPM (25 ms steps): {}/{n} steps sound",
        steps(&a.0, step, n).iter().filter(|s| sounds(s)).count()
    );
    let (step, n) = (step_of(12, 120.0), 32);
    for (name, set) in [
        ("mono", vec![]),
        ("mono-retrigger", vec![("retrigger", 1.0)]),
        ("poly", vec![("poly", 1.0)]),
    ] {
        let mut s = vec![("arp_on", 1.0), ("arp_rate", rate(12)), ("arp_gate", 0.9)];
        s.extend(PLUCK);
        s.extend(set);
        // The chord, then a single key (each step the same key again).
        for (what, keys) in [("chord", &C_MAJOR[..]), ("one-key", &[60][..])] {
            let a = render(&s, n as f64 * step + 0.3, &held(0, keys));
            save(&format!("gate90-1-32-120bpm-{name}-{what}-{tag}"), &a);
            let st = steps(&a.0, step, n);
            let missed: Vec<usize> = (0..n).filter(|&k| !attacked(st[k])).collect();
            println!(
                "[{tag}] GATE 90 %, 1/32, 120 BPM, {name}, {what}: steps without an attack {missed:?}"
            );
            println!(
                "[{tag}] GATE 90 %, 1/32, 120 BPM, {name}, {what}: {}/{n} steps attack, {}/{n} sound, the last step's peak {:.1} dBFS",
                st.iter().filter(|s| attacked(s)).count(),
                st.iter().filter(|s| sounds(s)).count(),
                db(peak(st[n - 1]))
            );
        }
    }
}

/// With HOLD off, as before (unchanged by the drone): a phrase played with the arpeggiator off
/// renders the same samples on either build (its hash printed to compare).
#[test]
#[ignore = "renders for listening, run by hand"]
fn with_hold_off_it_renders_as_before() {
    let tag = std::env::var("CA72_ARP_TAG").unwrap_or_else(|_| "after".into());
    let mut events = Vec::new();
    for (i, k) in [60u8, 64, 67, 72, 67, 64].iter().enumerate() {
        events.push(Ev::Key(secs(0.3 * i as f64), *k, true));
        events.push(Ev::Key(secs(0.3 * i as f64 + 0.25), *k, false));
    }
    events.push(Ev::Key(secs(2.0), 48, true));
    events.push(Ev::Key(secs(2.0), 55, true));
    events.push(Ev::Key(secs(3.0), 48, false));
    events.push(Ev::Key(secs(3.0), 55, false));
    for (name, set) in [
        ("mono", vec![]),
        ("poly", vec![("poly", 1.0)]),
        (
            "mono-glide-retrigger",
            vec![("glide_on", 1.0), ("retrigger", 1.0)],
        ),
    ] {
        let a = render(&set, 4.0, &events);
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in a.0.iter().chain(&a.1) {
            for b in v.to_bits().to_le_bytes() {
                h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
            }
        }
        save(&format!("hold-off-{name}-{tag}"), &a);
        println!(
            "[{tag}] HOLD off, ARP off, {name}: FNV-1a {h:016x}, RMS {:.2} dBFS",
            db(rms(&a.0))
        );
    }
}

/// HOLD's drone (the owner's change 1): ON off, HOLD on, a chord pressed and let go keeps
/// sounding as played; a new chord replaces it; HOLD off lets it go. In MONO, POLY and UNISON.
#[test]
#[ignore = "renders for listening, run by hand"]
fn drone() {
    for (name, set) in [
        ("mono", vec![]),
        ("poly", vec![("poly", 1.0)]),
        ("unison", vec![("unison", 1.0)]),
    ] {
        let mut s = vec![("arp_latch", 1.0)];
        s.extend(set);
        let mut e = held(0, &[48, 55, 60]);
        e.extend([48, 55, 60].map(|k| Ev::Key(secs(0.5), k, false)));
        e.extend(held(secs(3.0), &[50, 57, 62]));
        e.extend([50, 57, 62].map(|k| Ev::Key(secs(3.4), k, false)));
        e.push(Ev::Set(secs(6.0), "arp_latch", 0.0));
        let a = render(&s, 9.0, &e);
        save(&format!("drone-hold-{name}"), &a);
        let win = |t: f64| db(rms(&a.0[secs(t)..secs(t + 0.5)]));
        println!(
            "HOLD drone, {name}: let go at 0.5 s, RMS 1.5-2.0 s {:.1} dBFS, 2.5-3.0 s {:.1}; new chord at 3.0 s, RMS 4.5-5.0 s {:.1}, 5.5-6.0 s {:.1}; HOLD off at 6.0 s, RMS 8.0-8.5 s {:.1} dBFS",
            win(1.5),
            win(2.5),
            win(4.5),
            win(5.5),
            win(8.0)
        );
    }
    // With the sustain pedal down under it, and ON switched on and off while it holds.
    let mut e = held(0, &[48, 55, 60]);
    e.extend([48, 55, 60].map(|k| Ev::Key(secs(0.3), k, false)));
    e.push(Ev::Set(secs(2.0), "arp_on", 1.0));
    e.push(Ev::Set(secs(5.0), "arp_on", 0.0));
    e.push(Ev::Set(secs(7.0), "arp_latch", 0.0));
    let a = render(&[("arp_latch", 1.0), ("arp_rate", rate(6))], 9.5, &e);
    save("drone-hold-arp-on-and-off", &a);
    let win = |t: f64| db(rms(&a.0[secs(t)..secs(t + 0.5)]));
    println!(
        "HOLD drone, ON on at 2 s and off at 5 s: RMS 1.0-1.5 s {:.1} dBFS (held), 3.0-3.5 s {:.1} (arpeggiated), 6.0-6.5 s {:.1} (held again), HOLD off at 7 s, 9.0-9.5 s {:.1}",
        win(1.0),
        win(3.0),
        win(6.0),
        win(9.0)
    );
}

/// The notes' pitches, a step `step` long each from the start, `n` of them, as MIDI notes: from
/// the upward zero crossings a second over the step's middle (the calibration below: as the
/// keys played by hand to 12.5 kHz, where the pitch tracker an octave or more out above 2 kHz).
fn notes(x: &[f32], step: f64, n: usize) -> Vec<u8> {
    steps(x, step, n)
        .iter()
        .map(|s| {
            let q = s.len() / 8;
            let y = &s[q..s.len() - 2 * q];
            let ups = y.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count() as f64;
            let hz = ups * RATE / y.len() as f64;
            (69.0 + 12.0 * (hz / 440.0).log2())
                .round()
                .clamp(0.0, 255.0) as u8
        })
        .collect()
}

/// ORDER (change 3), against UP: G, C, E pressed in that order, then A added; OCTAVES 2. The
/// pitches measured each step.
#[test]
#[ignore = "renders for listening, run by hand"]
fn order() {
    let (step, n) = (step_of(6, 120.0), 14);
    let keys = [
        Ev::Key(0, 55, true),
        Ev::Key(10, 48, true),
        Ev::Key(20, 52, true),
        Ev::Key(secs(6.0 * step + 0.05), 57, true),
    ];
    for (name, direction) in [("up", 0.0), ("order", 1.0)] {
        let set = [
            ("arp_on", 1.0),
            ("arp_rate", rate(6)),
            ("arp_gate", 0.9),
            ("arp_octaves", 0.25),
            ("arp_direction", direction),
        ];
        let a = render(&set, n as f64 * step + 0.2, &keys);
        save(&format!("order-vs-up-{name}"), &a);
        println!(
            "DIRECTION {name}, G C E pressed in that order, A added at step 7, OCTAVES 2: notes {:?}",
            notes(&a.0, step, n)
        );
    }
}

/// OCTAVES 5 (the owner's addition), and high notes in the highest octave they have: G7 (103)
/// held, UP, OCTAVES 5 plays G7 G8 G9 G9 G9 (none silent); C4 with OCTAVES 5 for the spread.
#[test]
#[ignore = "renders for listening, run by hand"]
fn octaves() {
    let (step, n) = (step_of(6, 120.0), 10);
    for (name, key, direction) in [
        ("c4-up", 60u8, 0.0),
        ("g7-up", 103, 0.0),
        ("g7-updown", 103, 0.5),
        ("g7-random", 103, 0.75),
    ] {
        let set = [
            ("arp_on", 1.0),
            ("arp_rate", rate(6)),
            ("arp_gate", 0.9),
            ("arp_octaves", 1.0),
            ("arp_direction", direction),
        ];
        let a = render(&set, n as f64 * step + 0.2, &held(0, &[key]));
        save(&format!("octaves5-{name}"), &a);
        let st = steps(&a.0, step, n);
        println!(
            "OCTAVES 5, {name}: {}/{n} steps sound; notes {:?}",
            st.iter().filter(|s| sounds(s)).count(),
            notes(&a.0, step, n)
        );
    }
}

/// The standalone's TEMPO (change 4) reaching the arpeggiator: set to 96 as its screen sets it,
/// the transport stopped, a sixteenth's attacks 156.25 ms apart.
#[test]
#[ignore = "renders for listening, run by hand"]
fn tempo() {
    let (step, n) = (step_of(9, 96.0), 16);
    let mut s = vec![("arp_on", 1.0), ("arp_gate", 0.5)];
    s.extend(PLUCK);
    let a = render(
        &s,
        n as f64 * step + 0.2,
        &[Ev::Tempo(0, 96.0), Ev::Key(0, 60, true)],
    );
    save("tempo-96bpm-1-16", &a);
    let ms = secs(0.001);
    let onsets: Vec<f64> = (0..n)
        .filter_map(|k| {
            let from = secs(k as f64 * step);
            (from..from + 60 * ms)
                .find(|&i| db(f64::from(a.0[i].abs())) > -40.0)
                .map(|i| i as f64 / RATE)
        })
        .collect();
    let gaps: Vec<f64> = onsets.windows(2).map(|w| (w[1] - w[0]) * 1000.0).collect();
    let (lo, hi) = gaps
        .iter()
        .fold((f64::INFINITY, 0.0f64), |(l, h), g| (l.min(*g), h.max(*g)));
    let mean = (onsets[onsets.len() - 1] - onsets[0]) * 1000.0 / (onsets.len() - 1) as f64;
    println!(
        "TEMPO 96 (standalone, stopped), 1/16: {}/{n} attacks; onsets {mean:.2} ms apart on average, {lo:.2} to {hi:.2} (a sixteenth at 96: {:.2} ms; the onsets found where the wave first passes -40 dBFS, within a period of the attack)",
        steps(&a.0, step, n).iter().filter(|s| attacked(s)).count(),
        step * 1000.0
    );
}

/// (Calibration, not a render: the pitch measured of keys played by hand, and the attacks
/// counted of notes played by hand with and without a gap of 13 ms, to know what the measures
/// above can tell.)
#[test]
#[ignore = "calibration, run by hand"]
fn calibration() {
    for key in [60u8, 84, 96, 103, 108, 115, 120, 127] {
        let a = render(&[], 0.6, &[Ev::Key(0, key, true)]);
        let x = &a.0[secs(0.1)..secs(0.5)];
        let y: Vec<f64> = x.iter().map(|v| f64::from(*v)).collect();
        let hz = ca72_analysis::pitch::pitch(&y, RATE as u32).median_hz;
        // Upward zero crossings a second.
        let ups = x.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count() as f64 / 0.4;
        println!(
            "key {key} ({:.0} Hz): measured {hz:?} Hz, {ups:.0} upward crossings a second",
            440.0 * 2f64.powf((f64::from(key) - 69.0) / 12.0)
        );
    }
    for decay in [0.0f32, 0.1, 0.25] {
        for gap_ms in [6.0, 13.0, 20.0] {
            let step = 0.0625;
            let mut e = Vec::new();
            for k in 0..16 {
                e.push(Ev::Key(secs(k as f64 * step), 60, true));
                e.push(Ev::Key(
                    secs((k + 1) as f64 * step - gap_ms / 1000.0),
                    60,
                    false,
                ));
            }
            let a = render(
                &[
                    ("loudness_attack", 0.0),
                    ("loudness_decay", decay),
                    ("loudness_sustain", 0.0),
                ],
                1.2,
                &e,
            );
            println!(
                "by hand, decay {decay}, gap {gap_ms} ms: {}/16 steps attack",
                steps(&a.0, step, 16).iter().filter(|s| attacked(s)).count()
            );
        }
    }
}

/// The shortest note as a floor (the owner, 2026-10-19: "NEVER below the minimum gate
/// length"): 1/32 at 300 BPM (25 ms steps) and with SWING 100 % (its short steps 12.5 ms, so each
/// note is held over the next's start), GATE 0 and 100 %, in MONO, MONO with RETRIGGER, POLY
/// and UNISON, NOTE PRIORITY LOW and HIGH: every step's window sounds with the default contour;
/// with a plucked one (sustain 0), where single triggering cannot start the contours again in a
/// gap under the circuit's re-arm time, the steps that fade are counted.
#[test]
#[ignore = "renders for listening, run by hand"]
fn shortest_floor() {
    let quarter = 60.0 / 300.0;
    let step = ca72_plugin::arp::RATES[12].1 * quarter;
    let n = 48;
    for swing in [0.0f32, 1.0] {
        // The steps' starts (SWING delays every second by SWING × half a step).
        let starts: Vec<f64> = (0..=n)
            .map(|k| {
                k as f64 * step
                    + if k % 2 == 1 {
                        f64::from(swing) * step / 2.0
                    } else {
                        0.0
                    }
            })
            .collect();
        for gate in [0.0f32, 1.0] {
            for (mode, set) in [
                ("mono", vec![]),
                ("mono-high", vec![("note_priority", 1.0)]),
                ("mono-retrigger", vec![("retrigger", 1.0)]),
                ("poly", vec![("poly", 1.0)]),
                ("unison", vec![("unison", 1.0)]),
            ] {
                for pluck in [false, true] {
                    let mut s = vec![
                        ("arp_on", 1.0),
                        ("arp_rate", rate(12)),
                        ("arp_gate", gate),
                        ("arp_swing", swing),
                    ];
                    s.extend(set.clone());
                    if pluck {
                        s.extend(PLUCK);
                    }
                    let a = render(
                        &s,
                        starts[n] + 0.2,
                        &[vec![Ev::Tempo(0, 300.0)], held(0, &[72, 76, 79])].concat(),
                    );
                    let windows: Vec<&[f32]> = (0..n)
                        .map(|k| &a.0[secs(starts[k])..secs(starts[k + 1])])
                        .collect();
                    let quietest = windows
                        .iter()
                        .map(|w| db(rms(w)))
                        .fold(f64::INFINITY, f64::min);
                    let name = format!(
                        "floor-1-32-300bpm-swing{}-gate{}-{mode}{}",
                        swing * 100.0,
                        gate * 100.0,
                        if pluck { "-pluck" } else { "" }
                    );
                    if swing > 0.0 && gate == 0.0 {
                        save(&name, &a);
                    }
                    println!(
                        "{name}: {}/{n} steps sound, the quietest step {quietest:.1} dBFS",
                        windows.iter().filter(|w| sounds(w)).count()
                    );
                }
            }
        }
    }
}
