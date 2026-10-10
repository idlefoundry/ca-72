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
