//! The panel's laws measured on the hardware reference (docs/calibration): each passes
//! through its measured points, keeps the drawing's law where it was not measured, and
//! rises over the knob's travel.

use ca72::modulation::{midi_wheel, mod_wheel_r, mod_wheel_r_drawn};
use ca72::voice::{
    FILTER_ATTACK, FILTER_DECAY, LOUDNESS_ATTACK, LOUDNESS_DECAY, contour_amount_track,
    cutoff_track, glide_pot, glide_pot_drawn, time_pot, time_pot_drawn, volume_track,
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
        // The dial's 10 ms mark and the tick past it: the drawing's taper's shape, scaled to
        // meet the law's first point.
        let (p0, r0) = law[0];
        for p in [0.0, 0.05, 0.1, 0.15] {
            let want = r0 * time_pot_drawn(p) / time_pot_drawn(p0);
            assert!(
                (time_pot(p, law) - want).abs() <= 1e-9 * want.max(1.0),
                "{name} at {p}"
            );
        }
        let mut last = -1.0;
        for k in 0..=1000 {
            let r = time_pot(k as f64 / 1000.0, law);
            assert!(r.is_finite() && r > last, "{name}: {r} ohm at {k}/1000");
            last = r;
        }
        assert_eq!(time_pot(-1.0, law), 0.0, "{name} below its travel");
        assert_eq!(
            time_pot(2.0, law),
            law[law.len() - 1].1,
            "{name} past its travel"
        );
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

#[test]
fn contour_amount_follows_the_reference_at_its_marks() {
    for (p, t) in [(0.25, 0.2080), (0.4, 0.3796), (0.5, 0.5111), (0.75, 0.7950)] {
        assert!(
            (contour_amount_track(p) - t).abs() < 1e-12,
            "AMOUNT {p}: {}",
            contour_amount_track(p)
        );
    }
    assert_eq!(contour_amount_track(0.0), 0.0);
    assert_eq!(contour_amount_track(1.0), 1.0);
    let mut last = -1.0;
    for k in 0..=1000 {
        let t = contour_amount_track(k as f64 / 1000.0);
        assert!(t > last || k == 0, "AMOUNT {k}/1000: {t}");
        last = t;
    }
}

#[test]
fn glide_follows_the_reference_at_its_marks() {
    for (p, r) in [(0.25, 162e3), (0.5, 257e3), (0.75, 2.1e6), (1.0, 5e6)] {
        assert!(
            (glide_pot(p) / r - 1.0).abs() < 1e-12,
            "GLIDE {p}: {}",
            glide_pot(p)
        );
    }
    // Below 2.5, the drawing's taper scaled to meet it.
    let scale = 162e3 / glide_pot_drawn(0.25);
    assert!((glide_pot(0.1) / (scale * glide_pot_drawn(0.1)) - 1.0).abs() < 1e-12);
    assert_eq!(glide_pot(0.0), 0.0);
    let mut last = -1.0;
    for k in 0..=1000 {
        let r = glide_pot(k as f64 / 1000.0);
        assert!(r > last || k == 0, "GLIDE {k}/1000: {r}");
        last = r;
    }
}

#[test]
fn the_modulation_wheel_follows_the_reference_over_midi() {
    // MIDI's wheel puts the panel's where the reference's resistance is.
    for (cc, r) in [(32.0, 46.3), (64.0, 97.8), (96.0, 261.0), (127.0, 685.0)] {
        let got = mod_wheel_r(midi_wheel(cc / 127.0));
        assert!(
            (got / r - 1.0).abs() < 1e-9,
            "MIDI's wheel at {cc}: {got} ohm"
        );
    }
    assert_eq!(midi_wheel(0.0), 0.0);
    assert_eq!(midi_wheel(1.0), 1.0);
    // The panel's wheel: the reference's MOD DEPTH knob, below a quarter the drawing's
    // shape scaled to meet it.
    for (w, r) in [(0.25, 29.5), (0.5, 95.1), (0.75, 349.0), (1.0, 685.0)] {
        assert!(
            (mod_wheel_r(w) / r - 1.0).abs() < 1e-12,
            "the wheel at {w}: {}",
            mod_wheel_r(w)
        );
    }
    let scaled = 29.5 * mod_wheel_r_drawn(0.1) / mod_wheel_r_drawn(0.25);
    assert!((mod_wheel_r(0.1) / scaled - 1.0).abs() < 1e-12);
    assert_eq!(mod_wheel_r(0.0), 0.0);
    let mut last = -1.0;
    for k in 0..=1000 {
        let w = midi_wheel(k as f64 / 1000.0);
        assert!(w > last || k == 0, "MIDI's wheel at {k}/1000: {w}");
        last = w;
    }
}
