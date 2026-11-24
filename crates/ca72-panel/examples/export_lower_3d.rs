//! Export the official lower panel's geometry and its transparent silkscreen.
//!
//! cargo run --release -p ca72-panel --example export_lower_3d -- <out_dir>

use std::{fs, path::PathBuf};

use ca72_panel::Arrangement;
use ca72_panel::art::{self, Layout, Lettering, PANEL_H, STRIP_H, W};
use ca72_panel::controls::{Colour, Kind, Mark, Orient, Place};
use ca72_panel::fonts::{FAMILY, Fonts};
use ca72_panel::presets::{BarScene, BarTarget};
use ca72_panel::strip::{self, Bank, Rail, Readout, velocity};
use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg,
};
use serde_json::{Value, json};

fn part(kind: &str, (x, y): (f64, f64), (w, h): (f64, f64)) -> Value {
    json!({"kind": kind, "x": x, "y": y - PANEL_H, "w": w, "h": h})
}

fn window(kind: &str, (x, y, w, h): (f64, f64, f64, f64)) -> Value {
    part(kind, (x + w / 2.0, y + h / 2.0), (w, h))
}

fn screen(param: &str, role: &str, rect: (f64, f64, f64, f64)) -> Value {
    let mut p = window("screen", rect);
    p["param"] = json!(param);
    p["role"] = json!(role);
    p
}

fn demo_value(param: &str) -> f64 {
    match param {
        "voices" => 0.75,
        "entropy" => 0.4,
        "spread" => 0.7,
        "inner" => 0.3,
        "double" => 0.1,
        "drive" => 8.4 / 24.0,
        "level" => 30.0 / 42.0,
        "retrigger" | "decay_on" => 1.0,
        "glide_on" => 0.0,
        "mod_wheel" => 0.16,
        _ => 0.5,
    }
}

fn controls(arrangement: &Arrangement) -> Vec<Value> {
    let rail = Rail::Plugin;
    let bar = BarScene {
        panel: true,
        page: Some(BarTarget::Global),
        ..BarScene::default()
    };
    let mut controls = Vec::new();
    for &(x, w, target) in rail.keys() {
        let mut p = part("rail_key", (x, strip::RAIL_Y), (w, strip::KEY_H));
        p["param"] = json!(format!("{target:?}").to_ascii_lowercase());
        p["lit"] = json!(strip::key_light_of(&bar, target) != 0);
        controls.push(p);
    }
    controls.push(screen("preset_name", "preset", rail.name()));

    for (bank, active) in Bank::ALL.into_iter().zip([1, 0, 0, 0]) {
        controls.push(window("bank", bank.recess()));
        for (i, label) in bank.words().iter().enumerate() {
            let mut p = part("tab", bank.tab(i), strip::TAB);
            p["label"] = json!(label);
            p["lit"] = json!(i == active);
            controls.push(p);
        }
    }
    for readout in Readout::ALL {
        let param = match readout {
            Readout::Voices => "voices",
            Readout::Entropy => "entropy",
            Readout::Width => "spread",
            Readout::Detune => "double",
            Readout::Auto => "auto_gain",
            Readout::Drive => "drive",
            Readout::Level => "level",
            Readout::Inner => "inner",
        };
        controls.push(screen(param, "readout", readout.window()));
    }
    controls.push(screen("stereo_field", "field", strip::DISPLAY));

    for c in arrangement
        .controls
        .iter()
        .filter(|c| matches!(c.place, Place::Column | Place::Strip))
    {
        let at = c.centre();
        let mut p = match c.kind {
            Kind::Knob { big, .. } => {
                let r = art::skirt(&c.kind);
                let mut p = part("knob", at, (2.0 * r, 2.0 * r));
                p["sprite"] = json!(if big { "knob_big" } else { "knob_std" });
                p
            }
            Kind::Rocker {
                w,
                h,
                colour,
                orient,
            } => {
                let mut p = part("rocker", at, (w, h));
                p["colour"] = json!(match colour {
                    Colour::Ivory => "ivory",
                    Colour::Blue => "blue",
                    Colour::Red => "red",
                    Colour::Black => "black",
                });
                p["orient"] = json!(match orient {
                    Orient::Right => "right",
                    Orient::Top => "top",
                    Orient::Bottom => "bottom",
                });
                p
            }
            Kind::Toggle { w, h, across } => {
                let mut p = part("toggle", at, (w, h));
                p["across"] = json!(across);
                p
            }
            Kind::Wheel { mark, detent, span } => {
                // The wheel bounds include two units outside the slot on each edge.
                let [x0, y0, x1, y1] = art::bounds(&c.kind);
                let mut p = part("wheel", at, (x1 - x0 - 4.0, y1 - y0 - 4.0));
                p["mark"] = json!(match mark {
                    Mark::Line => "line",
                    Mark::Dot => "dot",
                });
                p["detent"] = json!(detent);
                p["span_deg"] = json!(span);
                p
            }
            _ => continue,
        };
        p["param"] = json!(c.param);
        p["label"] = json!(c.label);
        p["value"] = json!(demo_value(c.param));
        controls.push(p);
    }
    for ((x, y, r, _), param) in art::jacks(arrangement)
        .into_iter()
        .filter(|&(_, y, _, _)| y >= PANEL_H)
        .zip(["glide_jack", "decay_jack", "velocity_jack"])
    {
        let mut p = part("jack", (x, y), (2.0 * r, 2.0 * r));
        p["param"] = json!(param);
        controls.push(p);
    }
    for (i, rect) in velocity::lamp_windows().enumerate() {
        let mut p = window("velocity", rect);
        p["lit"] = json!(i < velocity::lit(Some(0.6)));
        controls.push(p);
    }
    for (x, y, r) in art::screws(arrangement) {
        if y >= PANEL_H {
            controls.push(part("screw", (x, y), (2.0 * r, 2.0 * r)));
        }
    }
    controls
}

fn main() {
    let out = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("target/panel-3d/lower"), PathBuf::from);
    fs::create_dir_all(&out).expect("create lower panel output directory");
    let arrangement = Arrangement::official();
    let spec = json!({
        "scale_m_per_unit": 0.680 / art::PW,
        "width": W,
        "height": STRIP_H,
        "cheek_w": 72.0,
        "rail_h": strip::RAIL,
        "controls": controls(&arrangement),
    });
    let json = serde_json::to_string_pretty(&spec).expect("serialize lower panel");
    fs::write(out.join("lower.json"), json + "\n").expect("write lower.json");

    let fonts = Fonts::new();
    let layout = Layout::measure_in(&fonts, Lettering::Standard, &arrangement);
    let print = art::strip_printed(&layout, &arrangement);
    let w = 5760;
    let h = (f64::from(w) * STRIP_H / W).round() as u32;
    let document = art::document([0.0, PANEL_H, W, STRIP_H], w, h, &print);
    let options = usvg::Options {
        fontdb: fonts.database(),
        font_family: FAMILY.into(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&document, &options).expect("parse lower panel silkscreen");
    let mut px = Pixmap::new(w, h).expect("allocate lower panel silkscreen");
    resvg::render(&tree, Transform::identity(), &mut px.as_mut());
    px.save_png(out.join("lower-print.png"))
        .expect("write lower-print.png");
    println!("Wrote lower.json and lower-print.png ({w}×{h})");
}
