//! LONG is the keyboard's hold capacitor five times larger (decisions.md R-GLIDE-LONG).
//! Measure the middle of each slide, clear of the key contacts' and amplifier's settling.

use ca72::voice::{Contacts, Panel, Quality, Voice};

const RATE: f64 = 12_000.0;

fn middle(mut tick: impl FnMut() -> f64, from: f64, to: f64) -> f64 {
    let mut at = [None; 2];
    let mut before = 0.0;
    for i in 0..(100.0 * RATE) as usize {
        let progress = (tick() - from) / (to - from);
        assert!(progress.is_finite());
        for (k, threshold) in [0.2, 0.8].into_iter().enumerate() {
            if at[k].is_none() && progress >= threshold {
                at[k] = Some(i as f64 + (threshold - before) / (progress - before));
            }
        }
        if let [Some(a), Some(b)] = at {
            return (b - a) / RATE;
        }
        before = progress;
    }
    panic!("the slide did not finish: {at:?}");
}

fn slide(setting: f64, long: bool, falling: bool, beyond: bool) -> f64 {
    let mut p = Panel {
        quality: Quality::HighFidelity,
        ..Panel::default()
    };
    let v = Voice::new(RATE, p);
    let (low, high) = if beyond { (43, 43) } else { (8, 20) };
    let (a, b) = if falling { (high, low) } else { (low, high) };
    let foot = if beyond {
        if falling { (24, 12) } else { (12, 24) }
    } else {
        (0, 0)
    };
    let base_a = v.keyboard().static_out(a).expect("the first key settles");
    let base_b = v.keyboard().static_out(b).expect("the next key settles");
    // The circuit's keyboard scale at its output, including the oscillator and filter loads.
    let step = (v
        .keyboard()
        .static_out(ca72::tuning::HIGH_A as usize)
        .expect("the high A")
        - v.keyboard()
            .static_out(ca72::tuning::LOW_A as usize)
            .expect("the low A"))
        / f64::from(ca72::tuning::HIGH_A - ca72::tuning::LOW_A);
    let (_, _, mut k, _, _, _) = v.into_parts();
    let contacts = |key, foot| Contacts {
        keys: Some((key, key)),
        gate: true,
        foot,
        ..Contacts::default()
    };
    for _ in 0..12_000 {
        k.tick(&p, contacts(a, foot.0));
    }
    p.glide_on = true;
    p.glide = setting;
    p.glide_long = long;
    middle(
        || k.tick(&p, contacts(b, foot.1)),
        base_a + step * foot.0 as f64,
        base_b + step * foot.1 as f64,
    )
}

#[test]
fn long_glide_takes_five_times_as_long_rising_and_falling() {
    for setting in [0.3, 0.5, 0.8, 1.0] {
        for falling in [false, true] {
            for beyond in [false, true] {
                let normal = slide(setting, false, falling, beyond);
                let long = slide(setting, true, falling, beyond);
                let ratio = long / normal;
                eprintln!(
                    "GLIDE {setting}, falling {falling}, beyond {beyond}: {ratio:.8} ({normal:.6} s, {long:.6} s)"
                );
                assert!(
                    (ratio - 5.0).abs() < 0.001,
                    "GLIDE {setting}, falling {falling}, beyond {beyond}: {ratio}"
                );
            }
        }
    }
}
