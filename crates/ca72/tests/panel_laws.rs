//! The panel's laws measured on the hardware reference (docs/calibration): each passes
//! through its measured points, keeps the drawing's law where it was not measured, and
//! rises over the knob's travel.

use ca72::voice::{
    FILTER_ATTACK, FILTER_DECAY, LOUDNESS_ATTACK, LOUDNESS_DECAY, time_pot, time_pot_drawn,
};

#[test]
fn attack_and_decay_follow_the_reference_at_its_marks() {
    for (name, law) in [
        ("filter ATTACK", &FILTER_ATTACK),
        ("filter DECAY", &FILTER_DECAY),
        ("loudness ATTACK", &LOUDNESS_ATTACK),
        ("loudness DECAY", &LOUDNESS_DECAY),
    ] {
        for &(p, r) in law {
            let got = time_pot(p, law);
            assert!(
                (got / r - 1.0).abs() < 1e-12,
                "{name} at {p}: {got} ohm, the reference's {r}"
            );
        }
        // The dial's 10 ms mark and the tick past it: the drawing's taper.
        for p in [0.0, 0.05, 0.1, 0.15] {
            assert_eq!(time_pot(p, law), time_pot_drawn(p), "{name} at {p}");
        }
        let mut last = -1.0;
        for k in 0..=1000 {
            let r = time_pot(k as f64 / 1000.0, law);
            assert!(r.is_finite() && r > last, "{name}: {r} ohm at {k}/1000");
            last = r;
        }
        assert_eq!(time_pot(-1.0, law), 0.0, "{name} below its travel");
        assert_eq!(time_pot(2.0, law), law[2].1, "{name} past its travel");
    }
}
