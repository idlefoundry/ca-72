//! The Expression page's fixtures and silkscreen, at the live editor's positions.

use ca72_panel::art::{self, Lettering};
use ca72_panel::controls::{CONTROLS, index};
use ca72_panel::strip::mpe;
use serde_json::{Value, json};

use super::super::{part, screen};

pub(super) fn page() -> (Vec<Value>, String) {
    let mut fixtures = vec![
        screen("expression_input", "readout", mpe::INPUT),
        screen("expression_last", "readout", mpe::LAST),
        screen("expression_modulates", "readout", mpe::MODULATES),
        screen("expression_response", "field", mpe::RESPONSE),
    ];
    let values = [0.75, 0.625, 0.35, 0.8125, 0.325, 0.15];
    for (i, param) in mpe::KNOB_PARAMS.into_iter().enumerate() {
        let c = CONTROLS[index(param).expect("Expression control")];
        let r = art::skirt(&c.kind);
        let mut knob = part("knob", c.centre(), (2.0 * r, 2.0 * r));
        knob["param"] = json!(param);
        knob["label"] = json!(c.label);
        knob["sprite"] = json!("knob_std");
        knob["hit_r"] = json!(r + 8.0);
        knob["value"] = json!(values[i]);
        fixtures.push(knob);
        fixtures.push(screen(
            &format!("expression_readout_{i}"),
            "readout",
            mpe::readout(i),
        ));
    }
    (fixtures, mpe::silkscreen(Lettering::Standard))
}
