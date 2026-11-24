//! Exports CA-72's single-source-of-truth 3D hardware spec (`spec.json`) and ultra-high-res
//! (>1.5x 4K) vector silkscreen & nameplate alpha masks using CA-72's exact layout and typeface.
//!
//! Usage:
//!   cargo run --release -p ca72-panel --example export_silkscreen -- <out_dir>

use std::path::PathBuf;

use ca72_panel::Arrangement;
use ca72_panel::art::{self, Layout, Lettering, PH, PLATE_H, PLATE_W, PW, TOP};
use ca72_panel::controls::{Colour, Kind, Orient, Place};
use ca72_panel::fonts::{FAMILY, Fonts};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;
use serde_json::json;

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/panel-3d"));
    let tex_dir = out_dir.join("textures");
    std::fs::create_dir_all(&tex_dir).expect("create texture output directory");

    let fonts = Fonts::new();
    let arrangement = Arrangement::official();
    let layout = Layout::measure_in(&fonts, Lettering::Standard, &arrangement);
    let options = usvg::Options {
        fontdb: fonts.database(),
        font_family: FAMILY.into(),
        ..usvg::Options::default()
    };

    // 1. Render exact vector faceplate silkscreen at 5760 x 1473 (>1.5x of 4K width!)
    //    `art::panel_print` wraps its body in `translate(COL, TOP)`, so viewBox starts at (0, TOP).
    let mask_w: u32 = 5760;
    let mask_h: u32 = ((mask_w as f64) * PH / PW).round() as u32;
    let print_body = art::panel_print(&layout, &arrangement);
    let doc = art::document([art::COL, TOP, PW, PH], mask_w, mask_h, &print_body);
    let tree = usvg::Tree::from_str(&doc, &options).expect("parse panel_print SVG");
    let mut pixmap = Pixmap::new(mask_w, mask_h).expect("allocate silkscreen pixmap");
    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
    pixmap
        .save_png(tex_dir.join("silkscreen_mask.png"))
        .expect("save silkscreen_mask.png");
    println!("Wrote silkscreen_mask.png ({mask_w}x{mask_h})");

    // 2. Render exact vector CA-72 / IDLE FOUNDRY nameplate lettering mask at 2088 x 656 (8x).
    let plate_w: u32 = (PLATE_W * 8.0).round() as u32;
    let plate_h: u32 = (PLATE_H * 8.0).round() as u32;
    let plate_body = format!(
        "<text x='24' y='44.68' text-anchor='start' font-size='38' font-weight='700' fill='#ffffff'>{}</text>\
         <text x='26' y='68.32' text-anchor='start' font-size='12' font-weight='700' letter-spacing='4.2' fill='#ffffff'>IDLE FOUNDRY</text>",
        art::PLATE_NAME
    );
    let plate_doc = art::document([0.0, 0.0, PLATE_W, PLATE_H], plate_w, plate_h, &plate_body);
    let plate_tree = usvg::Tree::from_str(&plate_doc, &options).expect("parse nameplate SVG");
    let mut plate_px = Pixmap::new(plate_w, plate_h).expect("allocate plate pixmap");
    resvg::render(&plate_tree, Transform::identity(), &mut plate_px.as_mut());
    plate_px
        .save_png(tex_dir.join("plate_mask.png"))
        .expect("save plate_mask.png");
    println!("Wrote plate_mask.png ({plate_w}x{plate_h})");

    // 3. Emit single-source-of-truth spec.json in real Minimoog Model D metric dimensions.
    //    Classic Minimoog panel width = 0.680 m for 3108 panel units -> 0.0002187902 m/unit.
    let scale_m_per_unit: f64 = 0.680 / PW;
    let cheek_w_units: f64 = 72.0;
    let bottom_rail_units: f64 = 36.0;
    let total_w_units: f64 = PW + 2.0 * cheek_w_units;
    let total_h_units: f64 = TOP + PH + bottom_rail_units;

    let mut controls_json = Vec::new();
    for (idx, c) in arrangement.controls.iter().enumerate() {
        if c.place != Place::Panel {
            continue;
        }
        let kind_obj = match c.kind {
            Kind::Knob { dial, big } => json!({
                "type": if big { "knob_big" } else { "knob_std" },
                "big": big,
                "dial": format!("{dial:?}"),
                "skirt_units": if big { art::BIG.skirt } else { art::STD.skirt },
                "sweep_deg": [art::KNOB_SWEEP.0, art::KNOB_SWEEP.1],
            }),
            Kind::Selector(set) => {
                let places: Vec<usize> = set.places().collect();
                let angles: Vec<f64> = places.iter().map(|&k| art::SIX[k]).collect();
                json!({
                    "type": if set.scale() < 0.9 { "knob_pointer_small" } else { "knob_pointer" },
                    "legends": format!("{set:?}"),
                    "scale": set.scale(),
                    "places": places,
                    "angles_deg": angles,
                })
            }
            Kind::Rocker {
                w,
                h,
                colour,
                orient,
            } => json!({
                "type": "rocker",
                "w_units": w,
                "h_units": h,
                "colour": match colour {
                    Colour::Blue => "blue",
                    Colour::Red => "red",
                    Colour::Black => "black",
                    Colour::Ivory => "ivory",
                },
                "orient": match orient {
                    Orient::Right => "right",
                    Orient::Top => "top",
                    Orient::Bottom => "bottom",
                },
            }),
            Kind::Toggle { w, h, across } => json!({
                "type": "toggle",
                "w_units": w,
                "h_units": h,
                "across": across,
            }),
            Kind::Key => json!({
                "type": "key",
                "w_units": art::KEY.0,
                "h_units": art::KEY.1,
            }),
            _ => continue,
        };
        controls_json.push(json!({
            "index": idx,
            "param": c.param,
            "label": c.label,
            "x_units": c.x,
            "y_units": c.y,
            "kind": kind_obj,
        }));
    }

    // The c. 1974 Echo Tone close-up gives the relative diameters and witness-dot positions.
    // The detached Syntaur #4734 selector shows the full round skirt and narrower cap.
    let spec = json!({
        "format": "ca72-3d-spec-v1",
        "scale_m_per_unit": scale_m_per_unit,
        "panel_units": {
            "face_w": PW,
            "face_h": PH,
            "wood_top_h": TOP,
            "cheek_w": cheek_w_units,
            "bottom_rail_h": bottom_rail_units,
            "total_w": total_w_units,
            "total_h": total_h_units,
            "trim_left_w": 15.0,
            "trim_right_w": 28.0,
            "trim_bottom_h": 6.0,
        },
        "panel_meters": {
            "face_w": PW * scale_m_per_unit,
            "face_h": PH * scale_m_per_unit,
            "wood_top_h": TOP * scale_m_per_unit,
            "cheek_w": cheek_w_units * scale_m_per_unit,
            "bottom_rail_h": bottom_rail_units * scale_m_per_unit,
            "total_w": total_w_units * scale_m_per_unit,
            "total_h": total_h_units * scale_m_per_unit,
            "cabinet_depth": 0.048,
        },
        "part_dimensions_meters": {
            "knob_std": {
                "base_radius": art::STD.skirt * scale_m_per_unit,
                "cap_radius": 32.5 * scale_m_per_unit,
                "dot_radius_pos": 48.5 * scale_m_per_unit,
                "dot_radius": 2.2 * scale_m_per_unit,
                "height": 0.0152,
                "flute_count": 8,
            },
            "knob_big": {
                "base_radius": art::BIG.skirt * scale_m_per_unit,
                "cap_radius": 42.0 * scale_m_per_unit,
                "dot_radius_pos": 58.5 * scale_m_per_unit,
                "dot_radius": 2.5 * scale_m_per_unit,
                "height": 0.0165,
                "flute_count": 8,
            },
            "knob_pointer": {
                "body_radius": 58.0 * scale_m_per_unit,
                "tip_radius": 58.0 * scale_m_per_unit,
                "cap_radius": 23.5 * scale_m_per_unit,
                "height": 0.0152,
                "scale": 1.0,
            },
            "knob_pointer_small": {
                "body_radius": 58.0 * 0.75 * scale_m_per_unit,
                "tip_radius": 58.0 * 0.75 * scale_m_per_unit,
                "cap_radius": 23.5 * 0.75 * scale_m_per_unit,
                "height": 0.0132,
                "scale": 0.75,
            },
            "rocker_horiz": {
                "w": 108.0 * scale_m_per_unit,
                "h": 50.0 * scale_m_per_unit,
                "bezel_pad": 0.0009,
                "height": 0.0082,
            },
            "rocker_vert": {
                "w": 50.0 * scale_m_per_unit,
                "h": 112.0 * scale_m_per_unit,
                "bezel_pad": 0.0009,
                "height": 0.0082,
            },
            "rocker_power": {
                "w": 43.0 * scale_m_per_unit,
                "h": 119.0 * scale_m_per_unit,
                "bezel_pad": 0.0010,
                "height": 0.0088,
            },
            "toggle": {
                "nut_across_flats": 36.0 * scale_m_per_unit,
                "washer_radius": 22.0 * scale_m_per_unit,
                "bushing_radius": 11.5 * scale_m_per_unit,
                "lever_length": 0.0168,
                "lever_base_radius": 0.00135,
                "lever_tip_radius": 0.00270,
            },
            "jewel_power": {
                "bezel_radius": 20.0 * scale_m_per_unit,
                "dome_radius": 13.5 * scale_m_per_unit,
                "height": 0.0072,
            },
            "jewel_overload": {
                "bezel_radius": 24.0 * scale_m_per_unit,
                "dome_radius": 16.0 * scale_m_per_unit,
                "height": 0.0080,
            },
            "ultra_lamp": {
                "bezel_radius": 44.0 * scale_m_per_unit,
                "dome_radius": 34.0 * scale_m_per_unit,
                "height": 0.0075,
            },
            "jack_quarter": {
                "outer_radius": 34.0 * scale_m_per_unit,
                "nut_across_flats": 62.0 * scale_m_per_unit,
                "sleeve_radius": 23.0 * scale_m_per_unit,
                "bore_radius": 14.5 * scale_m_per_unit,
                "height": 0.0055,
            },
            "screw_phillips": {
                "head_radius": 8.0 * scale_m_per_unit,
                "height": 0.0014,
            },
            "name_plate": {
                "w": PLATE_W * scale_m_per_unit,
                "h": PLATE_H * scale_m_per_unit,
                "thickness": 0.0018,
            }
        },
        "fixtures": {
            "power": [arrangement.power.0, arrangement.power.1],
            "lamp": [arrangement.lamp.0, arrangement.lamp.1],
            "overload": [arrangement.overload.0, arrangement.overload.1],
            "ultra": [arrangement.ultra.0, arrangement.ultra.1],
            "plate_top_left": [arrangement.plate.0, arrangement.plate.1],
            "plate_size": [PLATE_W, PLATE_H],
            "phones": arrangement.phones.map(|(x, y, r)| [x, y, r]),
            "screws": arrangement.screws.iter().map(|&(x, y, r)| [x, y, r]).collect::<Vec<_>>(),
        },
        "six_angles_deg": art::SIX,
        "controls": controls_json,
    });

    let spec_path = out_dir.join("spec.json");
    std::fs::write(
        &spec_path,
        serde_json::to_string_pretty(&spec).expect("serialize spec") + "\n",
    )
    .expect("write spec.json");
    println!(
        "Wrote {} ({} panel controls)",
        spec_path.display(),
        controls_json.len()
    );
}
