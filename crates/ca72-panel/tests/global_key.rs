//! The three lower pages' keys (decisions.md R-GLOBAL-KEY), shared by both skins.

use ca72_panel::presets::{BarScene, BarTarget};
use ca72_panel::strip::{self, Rail, StripTarget};

#[test]
fn global_precedes_the_keyboard_on_both_rails_without_closing_the_gaps() {
    for (rail, width, global) in [
        (Rail::Plugin, 1700.0, 2512.0),
        (Rail::Effects, 1600.0, 2412.0),
    ] {
        let keys = rail.keys();
        let i = keys.iter().position(|k| k.2 == BarTarget::Global).unwrap();
        assert_eq!(
            &keys[i - 1..i + 3],
            &[
                (global - 100.0, 86.0, BarTarget::Panel),
                (global, 86.0, BarTarget::Global),
                (global + 100.0, 86.0, BarTarget::Keyboard),
                (global + 200.0, 86.0, BarTarget::Lfo),
            ]
        );
        assert_eq!(rail.name(), (362.0, strip::RAIL_Y - 25.0, width, 50.0));
        // Name to NEXT, NEXT to SAVE and SAVE to PANEL keep their former clearances too.
        assert_eq!(keys[3].0 - keys[3].1 / 2.0 - (362.0 + width), 21.0);
        assert_eq!(
            keys[4].0 - keys[4].1 / 2.0 - (keys[3].0 + keys[3].1 / 2.0),
            16.0
        );
        assert_eq!(
            keys[5].0 - keys[5].1 / 2.0 - (keys[4].0 + keys[4].1 / 2.0),
            14.0
        );
        for &(x, w, t) in keys {
            let target = Some(StripTarget::Bar(t));
            assert_eq!(strip::centre(rail, StripTarget::Bar(t)), (x, strip::RAIL_Y));
            for dx in [-w / 2.0 - 4.0, 0.0, w / 2.0 + 4.0] {
                for dy in [-strip::KEY_H / 2.0 - 6.0, 0.0, strip::KEY_H / 2.0 + 6.0] {
                    assert_eq!(
                        strip::hit(rail, x + dx, strip::RAIL_Y + dy),
                        target,
                        "{rail:?} {t:?}"
                    );
                }
            }
            assert_eq!(strip::hit(rail, x + w / 2.0 + 4.1, strip::RAIL_Y), None);
        }
        assert_eq!(
            strip::hit(rail, 362.0 + width - 0.1, strip::RAIL_Y),
            Some(StripTarget::Bar(BarTarget::Name))
        );
        assert_eq!(strip::hit(rail, 362.0 + width + 0.1, strip::RAIL_Y), None);
    }
}

#[test]
fn one_page_is_lit_and_lfo_keeps_its_sound_glow() {
    use BarTarget::{Global, Keyboard, Lfo};
    for page in [Global, Keyboard, Lfo] {
        for panel in [false, true] {
            let bar = BarScene {
                panel,
                keyboard: page == Keyboard,
                lfo_page: panel && page == Lfo,
                lfo: true,
                ..Default::default()
            };
            for key in [Global, Keyboard, Lfo] {
                let expected = if panel && page == key {
                    2
                } else {
                    u8::from(key == Lfo)
                };
                assert_eq!(
                    strip::key_light_of(&bar, key),
                    expected,
                    "{page:?}, PANEL {panel}, {key:?}"
                );
            }
        }
    }
}
