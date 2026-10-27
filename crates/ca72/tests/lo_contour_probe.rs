//! Diagnostic: LO's filter contour against the circuit voice's at slow settings.

use ca72::light::Light;
use ca72::voice::{ContourKnobs, Panel, Quality, Voice};

#[test]
#[ignore]
fn lo_contour_against_the_circuit() {
    let sr = 48_000.0;
    for (attack, decay, sustain) in [
        (0.95, 0.924, 0.15),
        (0.8, 0.8, 0.15),
        (1.0, 1.0, 0.5),
        (0.6, 0.6, 0.15),
        (0.0, 0.95, 0.15),
        (0.95, 0.0, 1.0),
    ] {
        let mut p = Panel {
            quality: Quality::HighFidelity,
            ..Panel::default()
        };
        p.filter_contour = ContourKnobs {
            attack,
            decay,
            sustain,
        };
        let mut v = Voice::new(sr, p);
        let mut l = Light::new(sr);
        l.set_panel(&p);
        v.note(45, true);
        l.note(45, true);
        let mut rows = Vec::new();
        let marks = [
            0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0,
            30.0,
        ];
        let mut k = 0;
        for i in 0..(30.0 * sr) as usize + 1 {
            v.tick();
            l.tick(0.0);
            if k < marks.len() && i == (marks[k] * sr) as usize {
                rows.push((marks[k], v.probe_contours().0, l.contours().0));
                k += 1;
            }
        }
        println!("ATTACK {attack} DECAY {decay} SUSTAIN {sustain}");
        for (t, a, b) in rows {
            println!("  {t:5.2} s  HI {a:+.4}  LO {b:+.4}  LO-HI {:+.4}", b - a);
        }
    }
}

#[test]
#[ignore]
fn the_circuits_rest_against_decay() {
    let sr = 48_000.0;
    for switch in [true, false] {
        for decay in [0.0, 0.2, 0.4, 0.5, 0.6, 0.7, 0.8, 0.85, 0.9, 0.95, 1.0] {
            let mut p = Panel {
                quality: Quality::HighFidelity,
                decay: switch,
                ..Panel::default()
            };
            p.filter_contour = ContourKnobs {
                attack: 0.0,
                decay,
                sustain: 0.5,
            };
            p.loudness_contour = ContourKnobs {
                attack: 0.0,
                decay,
                sustain: 0.5,
            };
            let mut v = Voice::new(sr, p);
            for _ in 0..(0.5 * sr) as usize {
                v.tick();
            }
            let fresh = v.probe_contours();
            // A note, released, then 20 s.
            v.note(45, true);
            for _ in 0..(2.0 * sr) as usize {
                v.tick();
            }
            v.note(45, false);
            let mut after = Vec::new();
            for i in 0..(20.0 * sr) as usize {
                v.tick();
                if [0.5, 2.0, 5.0, 10.0, 19.9]
                    .iter()
                    .any(|t| i == (t * sr) as usize)
                {
                    after.push(v.probe_contours());
                }
            }
            let mut l = Light::new(sr);
            l.set_panel(&p);
            l.tick(0.0);
            let lo = l.contours();
            println!(
                "switch {switch} DECAY {decay}: fresh HI {:+.4}/{:+.4} LO {:+.4}/{:+.4}; after release {}",
                fresh.0,
                fresh.1,
                lo.0,
                lo.1,
                after
                    .iter()
                    .map(|a| format!("{:+.3}/{:+.3}", a.0, a.1))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
}

fn hz(x: &[f64], sr: f64) -> f64 {
    let ups: Vec<f64> = (1..x.len())
        .filter(|&i| x[i - 1] <= 0.0 && x[i] > 0.0)
        .map(|i| i as f64 - x[i] / (x[i] - x[i - 1]))
        .collect();
    if ups.len() < 3 {
        return f64::NAN;
    }
    (ups.len() - 1) as f64 * sr / (ups[ups.len() - 1] - ups[0])
}

#[test]
#[ignore]
fn held_contour_grid() {
    let sr = 48_000.0;
    for cutoff in [0.1, 0.3] {
        for amount in [0.0, 0.25, 0.4, 0.5, 0.75, 1.0] {
            for sustain in [0.0, 0.2, 0.4, 0.6, 0.8, 1.0] {
                let mut p = Panel {
                    quality: Quality::HighFidelity,
                    cutoff,
                    emphasis: 1.0,
                    contour_amount: amount,
                    ..Panel::default()
                };
                for o in &mut p.osc {
                    o.on = false;
                }
                p.filter_contour = ContourKnobs {
                    attack: 0.0,
                    decay: 0.0,
                    sustain,
                };
                let mut v = Voice::new(sr, p);
                let mut l = Light::new(sr);
                l.set_panel(&p);
                v.note(45, true);
                l.note(45, true);
                let n = (1.6 * sr) as usize;
                let (mut xv, mut xl) = (Vec::new(), Vec::new());
                for i in 0..n {
                    let a = v.tick();
                    let b = l.tick(0.0);
                    if i > (1.0 * sr) as usize {
                        xv.push(a);
                        xl.push(b);
                    }
                }
                println!(
                    "GRID {cutoff} {amount} {sustain} {:.4} {:.3} {:.4} {:.3}",
                    v.probe_contours().0,
                    hz(&xv, sr),
                    l.contours().0,
                    hz(&xl, sr)
                );
            }
        }
    }
}

/// Times for the filter contour's decay from its peak (ATTACK 0) toward SUSTAIN 0 to reach
/// 25, 50, 75 and 90 % of the way, and its attack (SUSTAIN 10, DECAY 0) 10, 50 and 90 % of
/// the way to its peak: HI against LO.
#[test]
#[ignore]
fn contour_times_grid() {
    let sr = 48_000.0;
    let times = |which: &str, knob: f64, lo: bool| -> Vec<f64> {
        let mut p = Panel {
            quality: Quality::HighFidelity,
            ..Panel::default()
        };
        p.filter_contour = if which == "decay" {
            ContourKnobs {
                attack: 0.0,
                decay: knob,
                sustain: 0.0,
            }
        } else {
            ContourKnobs {
                attack: knob,
                decay: 0.0,
                sustain: 1.0,
            }
        };
        let mut v = Voice::new(sr, p);
        let mut l = Light::new(sr);
        l.set_panel(&p);
        if lo {
            l.note(45, true)
        } else {
            v.note(45, true)
        }
        let mut xs = Vec::new();
        for _ in 0..(60.0 * sr) as usize {
            let x = if lo {
                l.tick(0.0);
                l.contours().0
            } else {
                v.tick();
                v.probe_contours().0
            };
            xs.push(x);
        }
        let start = xs[0];
        let (pk_i, pk) = xs
            .iter()
            .enumerate()
            .fold((0, f64::MIN), |a, (i, &x)| if x > a.1 { (i, x) } else { a });
        if which == "decay" {
            let end = xs[xs.len() - 1];
            [0.25, 0.5, 0.75, 0.9]
                .iter()
                .map(|f| {
                    let lvl = pk - f * (pk - end);
                    (pk_i + xs[pk_i..].iter().position(|&x| x < lvl).unwrap_or(0)) as f64 / sr
                        - pk_i as f64 / sr
                })
                .collect()
        } else {
            [0.1, 0.5, 0.9, 0.99]
                .iter()
                .map(|f| {
                    let lvl = start + f * (pk - start);
                    xs.iter().position(|&x| x > lvl).unwrap_or(0) as f64 / sr
                })
                .collect()
        }
    };
    for which in ["decay", "attack"] {
        for knob in [0.6, 0.8, 0.95] {
            let h = times(which, knob, false);
            let l = times(which, knob, true);
            println!(
                "TIMES {which} {knob}: HI {} | LO/HI {}",
                h.iter()
                    .map(|t| format!("{t:.3}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                l.iter()
                    .zip(&h)
                    .map(|(a, b)| format!("{:.3}", a / b))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
}

#[test]
#[ignore]
fn export_decay_curves() {
    use ca72::voice::{FILTER_DECAY, time_pot};
    let sr = 48_000.0;
    let out = std::env::var("DECAY_OUT").expect("DECAY_OUT");
    let mut text = String::new();
    for decay in [0.6, 0.7, 0.8, 0.85, 0.9, 0.95, 1.0] {
        for sustain in [0.0, 0.15, 0.5] {
            let mut p = Panel {
                quality: Quality::HighFidelity,
                ..Panel::default()
            };
            p.filter_contour = ContourKnobs {
                attack: 0.0,
                decay,
                sustain,
            };
            let mut v = Voice::new(sr, p);
            let mut l = Light::new(sr);
            l.set_panel(&p);
            v.note(45, true);
            l.note(45, true);
            let r = time_pot(decay, &FILTER_DECAY);
            let (mut hs, mut ls) = (Vec::new(), Vec::new());
            for i in 0..(40.0 * sr) as usize {
                v.tick();
                l.tick(0.0);
                if i % 480 == 0 {
                    hs.push(v.probe_contours().0);
                    ls.push(l.contours().0);
                }
            }
            text += &format!(
                "{decay} {sustain} {r} HI {}\n",
                hs.iter()
                    .map(|x| format!("{x:.5}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            text += &format!(
                "{decay} {sustain} {r} LO {}\n",
                ls.iter()
                    .map(|x| format!("{x:.5}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
    std::fs::write(out, text).unwrap();
}

/// The circuit's numbers for light.rs's `the_contours_follow_the_references_laws`.
#[test]
#[ignore]
fn the_circuits_numbers_for_the_laws_test() {
    let sr = 48_000.0;
    for lo in [false, true] {
        let mut p = Panel {
            quality: Quality::HighFidelity,
            ..Panel::default()
        };
        p.loudness_contour = ContourKnobs {
            attack: 0.6,
            decay: 0.5,
            sustain: 1.0,
        };
        p.filter_contour = ContourKnobs {
            attack: 0.0,
            decay: 0.6,
            sustain: 0.0,
        };
        let mut v = Voice::new(sr, p);
        let mut l = Light::new(sr);
        l.set_panel(&p);
        if lo {
            l.note(60, true)
        } else {
            v.note(60, true)
        }
        let (mut peak, mut top, mut half, mut at, mut ninety) = (0.0f64, 0.0f64, None, 0.0, None);
        for i in 0..96_000 {
            let (f, vv) = if lo {
                l.tick(0.0);
                l.contours()
            } else {
                v.tick();
                v.probe_contours()
            };
            if f > peak {
                peak = f;
            }
            if half.is_none() && peak > 4.0 && f < 0.5 * (peak + 0.0431) {
                half = Some(i as f64 / sr);
            }
            if ninety.is_none() && peak > 4.0 && f < 0.1 * peak + 0.9 * 0.0431 {
                ninety = Some(i as f64 / sr);
            }
            if vv > top + 1e-9 {
                top = vv;
                at = i as f64 / sr;
            }
        }
        println!(
            "{}: peak {peak:.4} plateau {at:.4} half {:.4} ninety {:.4}",
            if lo { "LO" } else { "HI" },
            half.unwrap() - 0.0086,
            ninety.unwrap() - 0.0086
        );
    }
}
