//! High EMPHASIS and AMOUNT OF CONTOUR must not turn ultrasonic regeneration into
//! broadband crackle. The whole voice exercises the production oversampling choice.

use ca72::voice::{Panel, Quality, Voice};
use ca72_analysis::fft::{hann, power};

fn render(rate: f64, cutoff: f64, quality: Quality) -> Vec<f64> {
    let panel = Panel {
        cutoff,
        emphasis: 1.0,
        contour_amount: 1.0,
        keyboard_control_1: true,
        keyboard_control_2: true,
        quality,
        ..Panel::default()
    };
    let mut voice = Voice::new(rate, panel);
    let _ = ca72::unconverged::take();
    let mut out = Vec::new();
    for i in 0..rate as usize {
        if i == (0.1 * rate) as usize {
            voice.note(60, true);
        }
        let y = voice.tick();
        assert!(y.is_finite(), "{quality:?} at {rate} Hz, sample {i}: {y}");
        out.push(y);
    }
    assert!(ca72::unconverged::take().is_empty(), "solver cap");
    out
}

/// Mean power between 8 and 20 kHz, over complete Hann windows in the held note.
fn high_band(out: &[f64], rate: f64) -> f64 {
    let n = 4096;
    let w = hann(n);
    let mut total = 0.0;
    let mut windows = 0;
    for frame in out[(0.3 * rate) as usize..(0.8 * rate) as usize].chunks_exact(n) {
        let p = power(frame, &w);
        assert!(p.iter().all(|x| x.is_finite()));
        total += p
            .iter()
            .enumerate()
            .filter(|(k, _)| (8000.0..=20000.0).contains(&(*k as f64 * rate / n as f64)))
            .map(|(_, x)| x)
            .sum::<f64>();
        windows += 1;
    }
    assert!(windows > 0 && total.is_finite() && total > 0.0);
    total / windows as f64
}

#[test]
fn strong_contour_does_not_add_radio_crackle() {
    for rate in [44_100.0, 48_000.0, 96_000.0] {
        for cutoff in [0.5, 0.75] {
            let reference = render(rate, cutoff, Quality::NoCompromises);
            let fast = render(rate, cutoff, Quality::Potato);
            let delta = 10.0 * (high_band(&fast, rate) / high_band(&reference, rate)).log10();
            eprintln!(
                "{rate} Hz, CUTOFF {}: excess 8–20 kHz power {delta:+.2} dB",
                cutoff * 10.0 - 5.0
            );
            assert!(
                delta.is_finite() && delta.abs() < 3.0,
                "{rate} Hz, cutoff {cutoff}: {delta:+.2} dB"
            );
        }
    }
}

#[test]
fn lo_strong_contour_does_not_fold_into_audible_tones() {
    for rate in [44_100.0, 48_000.0, 96_000.0] {
        for cutoff in [0.5, 0.75] {
            let reference = render(rate, cutoff, Quality::NoCompromises);
            let mut voice = ca72::light::Light::new(rate);
            voice.set_panel(&Panel {
                cutoff,
                emphasis: 1.0,
                contour_amount: 1.0,
                keyboard_control_1: true,
                keyboard_control_2: true,
                ..Panel::default()
            });
            let _ = ca72::unconverged::take();
            let mut out = Vec::new();
            for i in 0..rate as usize {
                if i == (0.1 * rate) as usize {
                    voice.note(60, true);
                }
                let y = voice.tick(0.0);
                assert!(y.is_finite());
                out.push(y);
            }
            assert!(ca72::unconverged::take().is_empty(), "LO solver cap");
            let signal = out[(rate * 0.3) as usize..(rate * 0.8) as usize]
                .iter()
                .map(|x| x * x)
                .sum::<f64>();
            assert!(signal > 1.0, "silent LO voice");
            let delta = 10.0 * (high_band(&out, rate) / high_band(&reference, rate)).log10();
            eprintln!("LO {rate} Hz, cutoff {cutoff}: excess 8–20 kHz power {delta:+.2} dB");
            assert!(
                delta.is_finite() && delta < 3.0,
                "LO aliasing: {delta:+.2} dB"
            );
        }
    }
}

/// Regenerate LO's relative high-cutoff fit from the calibrated converter and a
/// high-rate circuit ladder. Keep the already fitted dial pitch at the join.
#[test]
#[ignore = "regenerates the high-cutoff fit; run explicitly after converter changes"]
fn lo_cutoff_saturation_fit() {
    use ca72::{filter_cal, light::laws, vcf::Vcf, voice::Jacks};
    let mut panel = Panel {
        cutoff: 1.0,
        contour_amount: 0.0,
        keyboard_control_1: false,
        keyboard_control_2: false,
        ..Panel::default()
    };
    for osc in &mut panel.osc {
        osc.on = false;
    }
    let mut voice = Voice::new(48_000.0, panel);
    let mut base = 0.0;
    for (i, octaves) in [0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 10.0]
        .into_iter()
        .enumerate()
    {
        voice.tick_jacks(&Jacks {
            filter: Some(octaves * ca72::voice::FILTER_VOLTS_PER_OCTAVE),
            ..Jacks::default()
        });
        let i0 = voice.filter_probe().0;
        let rate = 192_000.0;
        let mut filter = Vcf::new(rate, 4);
        filter.circuit = filter_cal::CALIBRATED.circuit();
        filter.circuit.r14 = 0.0;
        filter.prewarp_hz = 0.0;
        let _ = ca72::unconverged::take();
        let out: Vec<_> = (0..19_200)
            .map(|n| filter.tick(if n < 8 { 20e-6 } else { 0.0 }, filter_cal::G_BUS_OFF, i0))
            .collect();
        assert!(out.iter().all(|x| x.is_finite()));
        assert!(ca72::unconverged::take().is_empty(), "fit solver cap");
        let window = &out[9_600..];
        let mean = window.iter().sum::<f64>() / window.len() as f64;
        let ups: Vec<_> = window
            .windows(2)
            .enumerate()
            .filter(|(_, w)| w[0] < mean && w[1] >= mean)
            .map(|(n, w)| n as f64 + (mean - w[0]) / (w[1] - w[0]))
            .collect();
        assert!(ups.len() > 100, "not oscillating");
        let hz = rate * (ups.len() - 1) as f64 / (ups[ups.len() - 1] - ups[0]);
        if i == 0 {
            base = hz.log2();
        }
        let fitted = laws::CUTOFF_SAT_AT[0] + hz.log2() - base;
        eprintln!("CUT CV {octaves} octaves: {i0:.8} A, {hz:.2} Hz, fitted log2 {fitted:.8}");
        assert!((fitted - laws::CUTOFF_SAT[i]).abs() < 1e-5);
    }
}

#[test]
fn moving_emphasis_does_not_reset_the_resamplers() {
    let rate = 48_000.0;
    let mut voice = Voice::new(
        rate,
        Panel {
            cutoff: 0.3,
            emphasis: 0.74,
            contour_amount: 0.0,
            quality: Quality::Potato,
            ..Panel::default()
        },
    );
    voice.note(60, true);
    let mut out = Vec::new();
    for i in 0..72_000 {
        if i == 33_600 {
            voice.panel.emphasis = 0.76;
        }
        if i == 52_800 {
            voice.panel.emphasis = 0.74;
        }
        let y = voice.tick();
        assert!(y.is_finite());
        out.push(y);
    }
    let jump = |a: usize, b: usize| {
        out[a..b]
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0, f64::max)
    };
    for i in [33_600, 52_800] {
        let normal = jump(i - 1000, i - 100);
        let moved = jump(i - 10, i + 100);
        assert!(normal > 1e-5, "silent voice");
        assert!(
            moved < 2.0 * normal,
            "EMPHASIS click: {moved:.4} against {normal:.4}"
        );
    }
}

#[test]
fn ultrasonic_regeneration_stays_outside_the_audio_band() {
    use ca72::vcf::Vcf;
    use ca72_lab::{bench::Solver, vcf as lab, work_dir};
    let Some(spice) = ca72_spice::for_test("ultrasonic_regeneration") else {
        return;
    };
    let trims = ca72::filter_cal::CALIBRATED;
    let b = lab::VcfBench {
        cutoff: 14.0,
        r14: 1.0,
        r39: trims.r39,
        r49: trims.r49,
        r73: trims.r73_pos,
        bus_load: Some(lab::MIXER_REST),
        ext_ctl: Some(0.0),
        ..lab::VcfBench::default()
    };
    let work = work_dir("ultrasonic-regeneration");
    let i0 = lab::ladder_current(&spice, &work.join("op"), &b, Solver::default()).expect("op");
    let ng = lab::transient(
        &spice,
        &work.join("tran"),
        &b,
        "vsrc src 0 pulse(0 0.2 1m 1u 1u 100u 1)",
        0.04,
        0.25e-6,
        Solver::default(),
    )
    .expect("transient");
    let reference = ca72_spice::resample(ng.vec("time"), ng.vec("out"), 0.02, 192_000.0, 2048);
    let p = power(&reference, &hann(reference.len()));
    let peak = p
        .iter()
        .enumerate()
        .skip(10)
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .expect("peak")
        .0 as f64
        * 192_000.0
        / reference.len() as f64;
    let mean = reference.iter().sum::<f64>() / reference.len() as f64;
    let reference_power = reference
        .iter()
        .map(|x| (x - mean) * (x - mean))
        .sum::<f64>()
        / reference.len() as f64;
    eprintln!("ngspice I0 {i0:.6}, peak {peak:.0} Hz, output power {reference_power:.4}");
    assert!(peak.is_finite() && (30_000.0..45_000.0).contains(&peak));
    assert!(
        reference_power.is_finite() && reference_power > 0.01,
        "silent reference"
    );
    let rate = 48_000.0;
    let mut f = Vcf::new(rate, 4);
    f.circuit = trims.circuit();
    f.circuit.r14 = b.r14;
    f.prepare(2);
    f.set_quality(Quality::Potato);
    let out: Vec<f64> = (0..(0.2 * rate) as usize)
        .map(|i| {
            let t = i as f64 / rate;
            let kick = if (0.001..0.0011).contains(&t) {
                0.2 / b.mix_r
            } else {
                0.0
            };
            f.tick(kick, ca72::filter_cal::G_BUS_OFF, i0)
        })
        .collect();
    let frame = &out[out.len() - 4096..];
    let p = power(frame, &hann(frame.len()));
    let high: f64 = p
        .iter()
        .enumerate()
        .filter(|(k, _)| (8000.0..=20000.0).contains(&(*k as f64 * rate / frame.len() as f64)))
        .map(|(_, x)| x)
        .sum();
    let w = hann(frame.len());
    let high_power = 2.0 * high / (frame.len() as f64 * w.iter().map(|x| x * x).sum::<f64>());
    let delta = 10.0 * (high_power / reference_power).log10();
    assert!(out.iter().all(|x| x.is_finite()));
    assert!(
        f.state()[..4].iter().map(|x| x * x).sum::<f64>() > 1e-5,
        "filter stopped oscillating"
    );
    eprintln!("4x audible power relative to ngspice's ultrasonic oscillation: {delta:.1} dB");
    assert!(
        delta.is_finite() && delta < -40.0,
        "ultrasonic oscillation folded into audio: {delta:.1} dB"
    );
}

/// One sounding voice must fit within 75 % of 128- and 256-frame periods on the
/// reference machine. Run under the exclusive timing lock, with the machine quiet.
#[test]
#[ignore = "paced CPU measurement; needs a quiet reference machine"]
fn high_contour_cost() {
    use std::time::{Duration, Instant};
    enum BenchVoice {
        Hi(Box<Voice>),
        Lo(Box<ca72::light::Light>),
    }
    impl BenchVoice {
        fn note(&mut self, key: i32, on: bool) {
            match self {
                Self::Hi(v) => v.note(key, on),
                Self::Lo(v) => v.note(key, on),
            }
        }
        fn tick(&mut self) -> f64 {
            match self {
                Self::Hi(v) => v.tick(),
                Self::Lo(v) => v.tick(0.0),
            }
        }
    }
    let rate = 48_000.0;
    for mode in ["HI", "LO"] {
        for block in [128, 256] {
            for emphasis in [0.7, 1.0] {
                let panel = Panel {
                    emphasis,
                    contour_amount: 1.0,
                    quality: Quality::Potato,
                    ..Panel::default()
                };
                let mut voice = if mode == "LO" {
                    let mut light = ca72::light::Light::new(rate);
                    light.set_panel(&panel);
                    BenchVoice::Lo(Box::new(light))
                } else {
                    BenchVoice::Hi(Box::new(Voice::new(rate, panel)))
                };
                voice.note(60, true);
                for _ in 0..4800 {
                    std::hint::black_box(voice.tick());
                }
                let period = Duration::from_secs_f64(block as f64 / rate);
                let blocks = (rate / block as f64) as usize;
                let start = Instant::now();
                let mut worst = Duration::ZERO;
                let mut total = Duration::ZERO;
                let mut late = 0;
                let stride = (0.2 * rate / block as f64) as usize;
                let gap = (0.02 * rate / block as f64).ceil() as usize;
                let mut note = 60;
                for i in 0..blocks {
                    let t = Instant::now();
                    if i % stride == 0 {
                        voice.note(note, false);
                    }
                    if i % stride == gap {
                        note = 60 + (i / stride % 5) as i32;
                        voice.note(note, true);
                    }
                    for _ in 0..block {
                        std::hint::black_box(voice.tick());
                    }
                    let cost = t.elapsed();
                    worst = worst.max(cost);
                    total += cost;
                    late += usize::from(cost > period.mul_f64(0.75));
                    if let Some(left) =
                        (start + period * (i as u32 + 1)).checked_duration_since(Instant::now())
                    {
                        std::thread::sleep(left);
                    }
                }
                eprintln!(
                    "{mode}, {block} frames, EMPHASIS {emphasis}: mean {:.3} ms, worst {:.3} ms, {late} blocks over 75 % of {:.3} ms",
                    total.as_secs_f64() * 1000.0 / blocks as f64,
                    worst.as_secs_f64() * 1000.0,
                    period.as_secs_f64() * 1000.0
                );
                assert_eq!(late, 0, "single voice exceeds its block budget");
            }
        }
    }
}
