//! The actual real-time source network against the loaded nonlinear circuit reference.
use ca72::{
    tuning::{Range, Tuning, osc1_inputs},
    vco::Vco,
    voice::{self, Waveform},
};
use ca72_lab::{
    bench::{Solver, Supplies},
    vco as lab,
};
use ca72_spice::crossings;
use std::f64::consts::TAU;
fn spectrum(x: &[f64], sr: f64, hz: f64) -> Vec<f64> {
    let n = x.len();
    (1..=24)
        .map(|k| {
            let mut re = 0.;
            let mut im = 0.;
            let mut sum = 0.;
            for (i, &v) in x.iter().enumerate() {
                let w = 0.5 - 0.5 * (TAU * i as f64 / (n - 1) as f64).cos();
                let p = TAU * k as f64 * hz * i as f64 / sr;
                re += v * w * p.cos();
                im -= v * w * p.sin();
                sum += w;
            }
            2. * re.hypot(im) / sum
        })
        .collect()
}
#[test]
fn realtime_hybrid_source_tracks_the_loaded_physical_network() {
    let Some(spice) = ca72_spice::for_test("waveform_source_port") else {
        return;
    };
    let tune = Tuning {
        r11: 150.33,
        a8: 0.13516,
        octave_step: 0.29889,
        tune: 7.5315,
    };
    let trims = lab::Trims {
        r11: tune.r11,
        a8: tune.a8,
        octave_step: tune.octave_step,
        ..Default::default()
    };
    let ctl = lab::Controls {
        kbd: lab::key_volts(16.),
        range: lab::Range::R8,
        tune: tune.tune,
        ..Default::default()
    };
    for track in [1.0f64, 0.8375] {
        let mut net = lab::netlist(&trims, &ctl, Supplies::default(), Solver::default());
        net.push_str(&format!(".include {}\nxwave saw tri wave mm_shark_source r030={} r031={}\nrpot_up wave wiper {}\nrpot_down wiper 0 {}\nrchannel wiper 0 33000\n",ca72_lab::circuits_dir().join("boards/front-waveform.lib").display(),voice::SHARK_SAW_OHMS,voice::SHARK_TRIANGLE_OHMS,(25000.*(1.-track)).max(1e-3),25000.*track));
        net.push_str("vhold hold 0 -2\nshold ramp hold hctl 0 swhold\nvhctl hctl 0 pwl(0 1 1u 1 1.001u 0)\n.model swhold sw(vt=0.5 vh=0 ron=1 roff=1e14)\n");
        let plots = spice
            .run(
                &net,
                &["tran 1e-6 0.055 0 1e-6"],
                &ca72_lab::work_dir(&format!("wave-port-{track}")),
            )
            .expect("loaded circuit");
        let p = &plots[0];
        let ts = p.vec("time");
        let falls = crossings(ts, p.vec("ramp"), -2., false);
        let period = falls[falls.len() - 1] - falls[falls.len() - 2];
        let start = ts[ts.len() - 1] - period;
        let wave = p.vec("wave");
        const N: usize = 8192;
        let mut j = 1;
        let uniform: Vec<f64> = (0..N)
            .map(|i| {
                let t = start + period * i as f64 / N as f64;
                while j < ts.len() - 1 && ts[j] < t {
                    j += 1;
                }
                let f = (t - ts[j - 1]) / (ts[j] - ts[j - 1]);
                wave[j - 1] + f * (wave[j] - wave[j - 1])
            })
            .collect();
        let physical: Vec<f64> = (1..=24)
            .map(|k| {
                let mut re = 0.;
                let mut im = 0.;
                for (i, &v) in uniform.iter().enumerate() {
                    let a = TAU * k as f64 * i as f64 / N as f64;
                    re += v * a.cos();
                    im -= v * a.sin();
                }
                2. * re.hypot(im) / N as f64
            })
            .collect();
        let mut v = Vco::new(48000., 1);
        v.expo.r11 = tune.r11;
        v.expo.a8 = tune.a8;
        let input = v.expo.input_current(&osc1_inputs(&tune, 16., Range::R8));
        for _ in 0..12000 {
            v.tick(input, 0.);
        }
        let first = v.last_reset();
        let down = 25000. * track;
        let load = 25000. * (1. - track) + down * 33000. / (down + 33000.);
        let mut samples = Vec::new();
        for _ in 0..48000 {
            let o = v.tick(input, 0.);
            let (source, r) = voice::waveform_source(Waveform::SharkTooth, &o, None);
            samples.push(source * load / (r + load));
        }
        let last = v.last_reset();
        let hz = 48000. * (last.1 - first.1) as f64 / (last.0 - first.0);
        let real = spectrum(&samples[7200..40800], 48000., hz);
        let level = 20. * (real[0] / physical[0]).log10();
        let errors: Vec<f64> = (1..24)
            .map(|k| 20. * ((real[k] / real[0]) / (physical[k] / physical[0])).log10())
            .collect();
        let rms = (errors.iter().map(|d| d * d).sum::<f64>() / 23.).sqrt();
        eprintln!("source track{track}:level{level:.4}dB,H2–24RMS{rms:.4}dB");
        assert!(level.abs() < 0.05, "source level{level}");
        assert!(rms < 0.2, "loaded source spectrum{rms}");
    }
}
