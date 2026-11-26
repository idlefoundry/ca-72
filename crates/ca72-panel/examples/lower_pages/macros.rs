//! Macro left-panel fixtures and ivory silkscreen for the 3D Cycles lower-panel exporter.

use ca72_panel::Arrangement;
use ca72_panel::art::{self, Lettering};
use ca72_panel::controls::MACRO_FIRST;
use ca72_panel::strip::macros;
use serde_json::{Value, json};

use super::super::{part, screen};

const MACRO_HIT_R: f64 = 56.0;
const DEMO_VALUES: [f64; macros::MACROS] = [0.55, 0.30, 0.0, 0.20, 0.70, 0.0];

pub(super) fn page() -> (Vec<Value>, String) {
    let arrangement = Arrangement::official();
    let jack_d = 2.0 * macros::JACK_R;

    let mut fixtures = Vec::new();
    for (n, value) in DEMO_VALUES.into_iter().enumerate() {
        // Name screen (0-based `macro_name_0`..`macro_name_5`), purely left of x = 367.
        fixtures.push(screen(
            &format!("macro_name_{n}"),
            "readout",
            macros::name_at(n),
        ));

        // Standard macro knob from official arrangement (`macro1`..`macro6`).
        let c = arrangement.controls[MACRO_FIRST + n];
        let at = c.centre();
        let r = art::skirt(&c.kind);
        let mut knob = part("knob", at, (2.0 * r, 2.0 * r));
        knob["param"] = json!(c.param);
        knob["label"] = json!(c.label);
        knob["sprite"] = json!("knob_std");
        knob["hit_r"] = json!(MACRO_HIT_R);
        knob["value"] = json!(value);
        fixtures.push(knob);

        // Macro OUT jack using `2.0 * macros::JACK_R`.
        let mut jack = part("jack", macros::jack_at(n), (jack_d, jack_d));
        jack["param"] = json!(format!("macro{}_jack", n + 1));
        fixtures.push(jack);
    }

    // Official `Dial::Macro` ivory silkscreen (0..10 ticks and end numerals), purely left of x = 367.
    let svg = macros::silkscreen(Lettering::Standard, "×");
    (fixtures, svg)
}
