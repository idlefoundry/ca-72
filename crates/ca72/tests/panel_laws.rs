//! The panel's laws measured on the hardware reference (docs/calibration): each passes
//! through its measured points, keeps the drawing's law where it was not measured, and
//! rises over the knob's travel.

use ca72::voice::{
    FILTER_ATTACK, FILTER_DECAY, LOUDNESS_ATTACK, LOUDNESS_DECAY, cutoff_track, time_pot,
    time_pot_drawn, volume_track,
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

#[test]
fn the_mixers_volume_follows_the_reference_at_its_marks() {
    for (p, t) in [(0.2, 0.1548), (0.4, 0.3777), (0.6, 0.6226), (0.8, 0.8453)] {
        assert!(
            (volume_track(p) - t).abs() < 1e-12,
            "VOLUME {p}: {}",
            volume_track(p)
        );
    }
    assert_eq!(volume_track(0.0), 0.0);
    assert_eq!(volume_track(1.0), 1.0);
    assert_eq!(volume_track(-1.0), 0.0);
    assert_eq!(volume_track(2.0), 1.0);
    let mut last = -1.0;
    for k in 0..=1000 {
        let t = volume_track(k as f64 / 1000.0);
        assert!(t > last || k == 0, "VOLUME {k}/1000: {t}");
        last = t;
    }
}

#[test]
fn cutoff_follows_the_reference_at_its_marks() {
    for (p, t) in [(0.1, 0.0582), (0.3, 0.2669), (0.7, 0.7304), (0.9, 0.9346)] {
        assert!(
            (cutoff_track(p) - t).abs() < 1e-12,
            "CUTOFF {p}: {}",
            cutoff_track(p)
        );
    }
    // The stops and the centre as drawn: where the filter's trims were fitted.
    assert_eq!(cutoff_track(0.0), 0.0);
    assert_eq!(cutoff_track(0.5), 0.5);
    assert_eq!(cutoff_track(1.0), 1.0);
    let mut last = -1.0;
    for k in 0..=1000 {
        let t = cutoff_track(k as f64 / 1000.0);
        assert!(t > last || k == 0, "CUTOFF {k}/1000: {t}");
        last = t;
    }
}
