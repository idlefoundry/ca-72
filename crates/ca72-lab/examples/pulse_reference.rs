//! Reproduce fixed-pulse DC and transient reductions from the full SPICE oscillator.
//! Run under the circuit-lab/timing locks with at most four ngspice slots.
use ca72_lab::{
    bench::{Solver, Supplies},
    vco as lab,
};
use ca72_spice::{Ngspice, crossings, resample};
use serde_json::json;
use std::{fs, path::PathBuf};
fn duty(t: &[f64], y: &[f64]) -> f64 {
    let lo = y.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = y.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let r = crossings(t, y, (hi + lo) / 2., true);
    let f = crossings(t, y, (hi + lo) / 2., false);
    let mut d = vec![];
    for p in r.windows(2).skip(3) {
        if let Some(f) = f.iter().find(|&&f| f > p[0] && f < p[1]) {
            d.push((f - p[0]) / (p[1] - p[0]));
        }
    }
    d.iter().sum::<f64>() / d.len() as f64
}
fn main() {
    let out = PathBuf::from(std::env::args().nth(1).expect("out"));
    fs::create_dir_all(&out).expect("pulse circuit reference measurement");
    let spice = Ngspice::find().expect("pulse circuit reference measurement");
    let mut rows = vec![];
    let cases = [
        ("factory", true, 0., 390000., 43000.),
        ("reference", true, 0., 374000., 41200.),
    ];
    for (tag, divider, volume, r44, r1) in cases {
        let pc = ca72::pulse::PulseCircuit {
            r1,
            r44,
            ..ca72::pulse::PulseCircuit::default()
        };
        for key in [0., 12., 24., 36., 48., 60., 72.] {
            for width in [0., -1.5, -2.5] {
                let ctl = lab::Controls {
                    kbd: lab::key_volts(key),
                    range: lab::Range::R8,
                    tune: 7.5315,
                    width,
                    fixed_width: divider,
                    pulse: pc,
                    ..lab::Controls::default()
                };
                let trims = lab::Trims {
                    r11: 150.33,
                    a8: 0.13516,
                    octave_step: 0.29889,
                    ..lab::Trims::default()
                };
                let solver = Solver {
                    steps_per_period: 4000.,
                    ..Solver::default()
                };
                let net = lab::netlist(&trims, &ctl, Supplies::default(), solver);
                let mut net = net;
                if volume > 0. {
                    let p = ca72::voice::volume_track(volume);
                    let load = 25000. * (1. - p) + (25000. * p * 33000.) / (25000. * p + 33000.);
                    net += &format!("rmixer rect 0 {load}\n");
                }
                net += ".save v(ramp) v(buf) v(saw) v(tri) v(rect) v(width) v(x1.b11) i(v.x1.vexp)\nvhold hold 0 -2\nshold ramp hold hctl 0 swhold\nvhctl hctl 0 pwl(0 1 1u 1 1.001u 0)\n.model swhold sw(vt=0.5 vh=0 ron=1 roff=1e14)\n";
                let dir = out.join(format!("{tag}-key-{key}-width-{width}"));
                let op = spice
                    .run(&net, &["op"], &dir.join("op"))
                    .expect("pulse circuit reference measurement");
                let current = op[0].scalar("i(v.x1.vexp)");
                assert!(current > 1e-12);
                let period_est = 0.01e-6 * 3.9 / current;
                let tmax = period_est / 4000.;
                let stop = period_est * 14. * 1.25;
                let tran = format!("tran {tmax:e} {stop:e} 0 {tmax:e}");
                let plot = spice
                    .run(&net, &[&tran], &dir.join("tran"))
                    .expect("pulse circuit reference measurement")
                    .remove(0);
                let t = plot.vec("time");
                let f = crossings(t, plot.vec("ramp"), -2., false);
                let periods: Vec<_> = f.windows(2).skip(3).map(|p| p[1] - p[0]).collect();
                let period = periods.iter().sum::<f64>() / periods.len() as f64;
                let hz = 1. / period;
                let mut amplitudes = serde_json::Map::new();
                for name in ["saw", "tri", "rect"] {
                    let m = 16384;
                    let z = resample(
                        t,
                        plot.vec(name),
                        t[t.len() - 1] - 1.5 * period,
                        m as f64 / period,
                        m,
                    );
                    let mut h = vec![];
                    for k in 1..=16 {
                        let mut re = 0.;
                        let mut im = 0.;
                        for (i, v) in z.iter().enumerate() {
                            let ph = 2. * std::f64::consts::PI * k as f64 * i as f64 / m as f64;
                            re += v * ph.cos();
                            im -= v * ph.sin();
                        }
                        h.push(2. * (re * re + im * im).sqrt() / m as f64);
                    }
                    amplitudes.insert(name.to_string(), json!(h));
                }
                let wi = if width == 0. {
                    0
                } else if width == -1.5 {
                    1
                } else {
                    2
                };
                let threshold = pc
                    .thresholds(ca72::pulse::WidthSource::FIXED[wi])
                    .expect("pulse circuit reference measurement");
                let fc = crossings(t, plot.vec("buf"), threshold.fall_buf, false);
                let fall = crossings(t, plot.vec("rect"), -2.1477, false);
                let mut fd = vec![];
                let mut slopes = vec![];
                for &tf in fc.iter().skip(3) {
                    if let Some(&tr) = fall
                        .iter()
                        .find(|&&tr| tr > tf && tr - tf < (0.2 * period).max(50e-6))
                    {
                        fd.push(tr - tf);
                        let i = t.partition_point(|&x| x < tf).clamp(1, t.len() - 2);
                        slopes.push(
                            -(plot.vec("buf")[i + 1] - plot.vec("buf")[i - 1])
                                / (t[i + 1] - t[i - 1]),
                        );
                    }
                }
                let reset = crossings(t, plot.vec("ramp"), -2., true);
                let rise = crossings(t, plot.vec("rect"), -2.1477, true);
                let mut rd = vec![];
                for &tr in reset.iter().skip(3) {
                    let i = t.partition_point(|&x| x < tr);
                    let mut j = i;
                    while j > 0 && t[j] > tr - 20e-6 {
                        j -= 1;
                    }
                    let k = (j..i)
                        .min_by(|&a, &b| plot.vec("ramp")[a].total_cmp(&plot.vec("ramp")[b]))
                        .expect("pulse circuit reference measurement");
                    if let Some(&rr) = rise.iter().find(|&&rr| rr > t[k] && rr - t[k] < 30e-6) {
                        rd.push(rr - t[k]);
                    }
                }
                let mean = |x: &Vec<f64>| x.iter().sum::<f64>() / x.len() as f64;
                rows.push(json!({"tag":tag,"divider":divider,"volume":volume,"r44_ohms":r44,"r1_ohms":r1,"width_bias_nominal_v":width,"key":key,"spice_hz":hz,"duty_midpoint":duty(t,plot.vec("rect")),"fall_threshold_buffer_v":threshold.fall_buf,"rise_threshold_buffer_v":threshold.rise_buf,"fall_delay_s":mean(&fd),"buffer_slope_v_per_s":mean(&slopes),"rise_delay_s":mean(&rd),"harmonic_peak_amplitudes":amplitudes}));
                fs::write(
                    out.join("topology-results.json"),
                    serde_json::to_string_pretty(&rows)
                        .expect("pulse circuit reference measurement"),
                )
                .expect("pulse circuit reference measurement");
                println!(
                    "{tag} {width}: {hz:.6}Hz duty {:.8}",
                    duty(t, plot.vec("rect"))
                );
            }
        }
    }
}
