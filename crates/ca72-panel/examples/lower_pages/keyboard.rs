//! Fixtures and silkscreen lettering for the CA-72 lower KEYS page.

use ca72_panel::art::PANEL_H;
use ca72_panel::fonts::FAMILY;
use ca72_panel::keyboard::{self, BUTTON_H, HEADER, KEYS, Scene, UP_X};
use ca72_panel::strip::{CV, CW, KEY_W};
use serde_json::{Value, json};

// Matches the private octave display dimensions in `ca72_panel::keyboard`.
const RANGE_W: f64 = 348.0;
const RANGE_H: f64 = 50.0;

pub(super) fn page() -> (Vec<Value>, String) {
    let range_x = CW - RANGE_W / 2.0;
    let down_x = 2.0 * CW - UP_X;

    let mut controls = Vec::with_capacity(KEYS + 3);

    for (x, param, label) in [(down_x, "octave_down", "−"), (UP_X, "octave_up", "+")] {
        let mut button = super::super::part("rail_key", (x, HEADER), (KEY_W, BUTTON_H));
        button["param"] = json!(param);
        button["label"] = json!(label);
        button["lit"] = json!(false);
        controls.push(button);
    }

    controls.push(super::super::screen(
        "octave",
        "octave",
        (range_x, HEADER - RANGE_H / 2.0, RANGE_W, RANGE_H),
    ));

    let root = Scene::default().root;
    for dark in [false, true] {
        for relative in 0..KEYS {
            let note = root + relative as u8;
            if keyboard::black(note) != dark {
                continue;
            }
            let [x0, y0, x1, y1] = keyboard::rect(root, note).expect("official keyboard key");
            let kind = if dark { "black_key" } else { "white_key" };
            let mut key =
                super::super::part(kind, ((x0 + x1) / 2.0, (y0 + y1) / 2.0), (x1 - x0, y1 - y0));
            key["param"] = json!(note.to_string());
            key["label"] = json!(keyboard::label(note));
            if !dark {
                let outline =
                    keyboard::white_shape(root, note).expect("official natural key outline");
                let shape: [[f64; 2]; 8] = outline.map(|[x, y]| [x, y - PANEL_H]);
                key["shape"] = json!(shape);
            }
            controls.push(key);
        }
    }

    // Static header legend in absolute CA-72 coordinates; octave range text is drawn live.
    let octave_size = 28.0;
    let octave_baseline = HEADER + octave_size * 0.36;
    let print = format!(
        "<g font-family='{FAMILY}'><text x='{CV}' y='{octave_baseline:.3}' font-size='{octave_size}' text-anchor='middle' fill='#ebe6d8'>OCTAVE</text></g>"
    );

    (controls, print)
}
