//! The drawn fixed selector and its reduced comparator against full-board SPICE.
use ca72::pulse::{PulseCircuit, WidthSource};
use ca72_lab::{
    bench::{Solver, Supplies},
    vco::{self, Controls, Trims},
};
fn edge(buf: &[f64], rect: &[f64]) -> f64 {
    let i = (0..rect.len() - 1)
        .max_by(|&a, &b| {
            (rect[a + 1] - rect[a])
                .abs()
                .total_cmp(&(rect[b + 1] - rect[b]).abs())
        })
        .expect("pulse circuit reference measurement");
    (buf[i] + buf[i + 1]) / 2.
}
#[test]
fn fixed_selector_thresholds_follow_the_transistor_circuit() {
    let Some(spice) = ca72_spice::for_test("pulse_width_thresholds") else {
        return;
    };
    for (tag, circuit) in [
        ("factory", PulseCircuit::default()),
        ("reference", PulseCircuit::reference()),
    ] {
        for (i, width) in WidthSource::FIXED.into_iter().enumerate() {
            let ctl = Controls {
                width: width.volts,
                fixed_width: true,
                pulse: circuit,
                ..Controls::default()
            };
            let net = vco::netlist(
                &Trims::default(),
                &ctl,
                Supplies::default(),
                Solver::default(),
            );
            let expected = circuit
                .thresholds(width)
                .expect("pulse circuit reference measurement");
            for falling in [true, false] {
                let (from, to, step) = if falling {
                    (0.2, -4.3, -0.0001)
                } else {
                    (-4.3, 0.2, 0.0001)
                };
                let p = spice
                    .run(
                        &format!("{net}vhold ramp 0 {from}\n.save v(buf) v(rect)\n"),
                        &[&format!("dc vhold {from} {to} {step}")],
                        &ca72_lab::work_dir(&format!("pulse-{tag}-{i}-{falling}")),
                    )
                    .expect("pulse circuit reference measurement")
                    .remove(0);
                let measured = edge(p.vec("buf"), p.vec("rect"));
                let target = if falling {
                    expected.fall_buf
                } else {
                    expected.rise_buf
                };
                eprintln!(
                    "{tag} width{} falling{falling} SPICE{measured:.7}V reduced{target:.7}V",
                    width.volts
                );
                assert!(
                    (measured - target).abs() < 0.0003,
                    "{tag} {} falling{falling}: {measured} vs {target}",
                    width.volts
                );
            }
        }
    }
}
#[test]
fn the_same_drawn_divider_reaches_each_oscillator() {
    for osc in [
        vco::Osc::One,
        vco::Osc::Two { freq: 0.5 },
        vco::Osc::Three {
            freq: 0.5,
            control: true,
        },
    ] {
        for width in [-1.5, -2.5] {
            let ctl = Controls {
                osc,
                width,
                fixed_width: true,
                pulse: PulseCircuit::reference(),
                ..Controls::default()
            };
            let net = vco::netlist(
                &Trims::default(),
                &ctl,
                Supplies::default(),
                Solver::default(),
            );
            assert!(!net.contains("vwidth width 0"));
            assert!(net.contains("r034 "));
            assert!(net.contains("r033 "));
            assert!(net.contains("r032 "));
            assert!(net.contains("pr1=41200"));
            assert!(net.contains("pr44=374000"));
        }
    }
}
