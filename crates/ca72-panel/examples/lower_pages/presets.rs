//! Physical glass, caps and silkscreen from the preset browser's own layout.

use ca72_panel::presets::{DRAWER_TOP, PresetKey, drawer_silkscreen, drawer_windows};
use serde_json::{Value, json};

use super::super::{part, screen};

pub(super) fn page() -> (Vec<Value>, String) {
    let mut fixtures: Vec<_> = ["search", "tags", "list", "status"]
        .into_iter()
        .zip(drawer_windows())
        .map(|(id, (x, y, w, h))| screen(id, "preset", (x, y + DRAWER_TOP, w, h)))
        .collect();
    fixtures.extend(PresetKey::ALL.map(|key| {
        let (x, y) = key.centre();
        let mut cap = part("rail_key", (x, y + DRAWER_TOP), key.size());
        cap["label"] = json!(key.legend());
        cap
    }));
    let print = format!(
        "<g transform='translate(0 {DRAWER_TOP})'>{}</g>",
        drawer_silkscreen()
    );
    (fixtures, print)
}
