//! NOTE PRIORITY (decisions.md R-PRIORITY): which of the keys held sounds. LOW is the keyboard
//! circuit's own (the lowest and highest keys' contacts, as drawn); HIGH and LAST hand the
//! circuit the key they choose alone, so that the keyboard's output with keys held together is
//! its output with that key held alone. Beyond the instrument's 44 keys the chosen key continues
//! the string as a key alone does there.

use ca72::voice::{Panel, Priority, Quality, Voice};

const SR: f64 = 48_000.0;

fn voice(priority: Priority) -> Voice {
    let mut v = Voice::new(SR, Panel::default());
    v.panel.quality = Quality::HighFidelity;
    v.panel.priority = priority;
    v
}

/// The keyboard's output once settled (GLIDE off: within milliseconds), V.
fn settled(v: &mut Voice) -> f64 {
    for _ in 0..(0.03 * SR) as usize {
        v.tick();
    }
    v.probe().0
}

/// The keyboard's output with `key` held alone, V.
fn alone(key: i32) -> f64 {
    let mut v = voice(Priority::Low);
    v.note(key, true);
    settled(&mut v)
}

/// Each step's key pressed (positive) or released (negative MIDI note), and the key then
/// sounding: the keyboard's output where that key's alone is, within 0.05 mV (0.06 cent).
fn plays(priority: Priority, steps: &[(i32, i32)]) {
    plays_within(priority, steps, 5e-5);
}

fn plays_within(priority: Priority, steps: &[(i32, i32)], within: f64) {
    let mut v = voice(priority);
    for &(key, sounding) in steps {
        v.note(key.abs(), key > 0);
        let got = settled(&mut v);
        let want = alone(sounding);
        assert!(
            (got - want).abs() < within,
            "{priority:?}: after {key:+}, {:.3} mV from key {sounding}'s",
            (got - want) * 1e3
        );
    }
    assert_eq!(v.keyboard().failed, 0);
}

/// LOW is the circuit as drawn: with a higher key held too, its contact shorts the string
/// between the two and the lowest sounds a little sharp of itself alone (key 57 with 64 held:
/// 0.98 mV, 1.2 cents), as the instrument's does. Within 1.5 mV.
#[test]
fn low_plays_the_lowest_key_held() {
    plays_within(
        Priority::Low,
        &[(57, 57), (64, 57), (50, 50), (-50, 57), (-57, 64)],
        1.5e-3,
    );
}

#[test]
fn high_plays_the_highest_key_held() {
    plays(
        Priority::High,
        &[(57, 57), (64, 64), (50, 64), (-64, 57), (-57, 50)],
    );
}

#[test]
fn last_plays_the_key_pressed_last_then_the_last_still_held() {
    plays(
        Priority::Last,
        &[
            (57, 57),
            (64, 64),
            (50, 50),
            (-50, 64),
            (60, 60),
            (-64, 60),
            (-60, 57),
        ],
    );
}

/// Keys beyond the instrument's 44 (F to C, MIDI 41 to 84): LOW leaves one above C silent
/// while a key of its own is held, as the instrument does; HIGH and LAST play it.
#[test]
fn beyond_the_keys_the_chosen_key_plays_as_alone() {
    plays_within(Priority::Low, &[(60, 60), (100, 60), (30, 30)], 1.5e-3);
    plays(
        Priority::High,
        &[(60, 60), (100, 100), (30, 100), (-100, 60)],
    );
    plays(Priority::Last, &[(100, 100), (30, 30), (60, 60), (-60, 30)]);
}

/// RETRIGGER (R-PRIORITY): a key pressed while another is held starts the contours again (off,
/// they go on from where they are, the instrument's single triggering). The trigger contact
/// opens for `RETRIGGER_GAP` first, the keys held as they were, so the pitch moves to the new
/// key (LAST) only then. The loudness contour decayed to SUSTAIN 2 with key 57 held a while;
/// key 64 pressed: its most over the next 80 ms.
#[test]
fn retrigger_starts_the_contours_again_with_keys_held() {
    let run = |retrigger: bool| {
        let mut v = voice(Priority::Last);
        v.panel.retrigger = retrigger;
        v.panel.loudness_contour.decay = 0.3;
        v.panel.loudness_contour.sustain = 0.2;
        v.note(57, true);
        for _ in 0..(0.5 * SR) as usize {
            v.tick();
        }
        let before = v.envelopes().1;
        v.note(64, true);
        // Within the gap the pitch is still key 57's.
        for _ in 0..(0.008 * SR) as usize {
            v.tick();
        }
        let early = v.probe().0;
        let mut most: f64 = 0.0;
        for _ in 0..(0.08 * SR) as usize {
            v.tick();
            most = most.max(v.envelopes().1);
        }
        (before, most, early, v.probe().0)
    };
    let (k57, k64) = (alone(57), alone(64));
    let (before, most, early, late) = run(false);
    assert!(
        (most - before).abs() < 0.05,
        "single triggering: {before:.3} V, then at most {most:.3} V"
    );
    assert!(
        (early - k64).abs() < 5e-5,
        "single triggering: the pitch at once"
    );
    let (before, most, early, late_re) = run(true);
    assert!(
        most > before + 2.0,
        "RETRIGGER: {before:.3} V, then at most {most:.3} V"
    );
    assert!(
        (early - k57).abs() < 5e-5,
        "RETRIGGER: key 57's within the gap"
    );
    assert!((late_re - k64).abs() < 5e-5 && (late - k64).abs() < 5e-5);
}

/// With RETRIGGER, a key pressed soon after the last was let go waits until the trigger
/// contact has been open `RETRIGGER_GAP`, so that it starts the contours again too; one pressed
/// later comes down at once, as without it.
#[test]
fn retrigger_waits_out_the_gap_after_a_release() {
    let mut v = voice(Priority::Low);
    v.panel.retrigger = true;
    v.note(57, true);
    settled(&mut v);
    v.note(57, false);
    for _ in 0..(0.004 * SR) as usize {
        v.tick();
    }
    v.note(64, true);
    for _ in 0..(0.004 * SR) as usize {
        v.tick();
    }
    // 8 ms after the release: still where key 57 left it (the keyboard's hold).
    let held = v.probe().0;
    assert!((held - alone(57)).abs() < 1e-3, "{held} V within the gap");
    let at = settled(&mut v);
    assert!((at - alone(64)).abs() < 5e-5, "{at} V");
}
