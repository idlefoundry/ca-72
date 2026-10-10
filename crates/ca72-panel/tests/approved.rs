//! The panel as drawn matches the design the owner approved (`approved.png`: the mock-up at one
//! panel unit a pixel, every control at half its travel). The switch column between MIXER and
//! MODIFIERS (the mock-up's x 2100 to 2330, y 225 to 690) is the renderer's own, as the owner
//! chose it on 2026-10-09 with FILTER MODE at its head (decisions.md R-HP). The mock-up had the
//! left hand's controls in a column left of the panel, 330 units wide; they are on the strip
//! below now (A6, R-LOOK), so the panel is compared with the mock-up right of its column.

use ca72_panel::art::{PANEL_H, PLATE_H, PLATE_X, PLATE_Y, W};
use ca72_panel::{CONTROLS, Renderer, Scene};
use resvg::tiny_skia::Pixmap;

/// The width of the mock-up's column, left of the panel.
const COLUMN: u32 = 330;

/// The mock-up drew its name plate in a fallback face, so the plate is left out.
fn outside_the_plate(x: u32, y: u32) -> bool {
    let (x, y) = (f64::from(x), f64::from(y));
    !(PLATE_X - 8.0..PLATE_X + 290.0).contains(&x)
        || !(PLATE_Y - 8.0..PLATE_Y + PLATE_H + 12.0).contains(&y)
}

#[test]
fn the_panel_is_drawn_as_approved() {
    let approved = Pixmap::load_png(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/approved.png"))
        .expect("the approved panel");
    let mut r = Renderer::new(1.0, 1.0);
    r.render(&Scene {
        values: [0.5; CONTROLS.len()],
        ..Scene::default()
    });
    let drawn = r.frame();
    assert_eq!(
        (drawn.width(), PANEL_H as u32),
        (approved.width() - COLUMN, approved.height())
    );
    assert_eq!(drawn.width(), W as u32);

    let mut differences: Vec<u8> = Vec::new();
    for y in 0..approved.height() {
        for x in 0..drawn.width() {
            let a = drawn.pixel(x, y).expect("in the frame");
            let b = approved.pixel(x + COLUMN, y).expect("in the mock-up");
            if !outside_the_plate(x, y) {
                continue;
            }
            let d = [
                a.red().abs_diff(b.red()),
                a.green().abs_diff(b.green()),
                a.blue().abs_diff(b.blue()),
            ];
            differences.push(d.into_iter().max().unwrap_or(0));
        }
    }
    let mean = differences.iter().map(|&d| f64::from(d)).sum::<f64>() / differences.len() as f64;
    differences.sort_unstable();
    let p99 = differences[differences.len() * 99 / 100];
    // The margin, for the record on each machine (`-- --nocapture`).
    println!(
        "the panel as drawn against approved.png: mean difference {mean:.3}, 99th percentile {p99} (passes under 1.5 and 32)"
    );
    // Text rasterises a little lighter here than in the browser that drew the mock-up.
    assert!(
        mean < 1.5 && p99 < 32,
        "mean difference {mean:.2}, 99th percentile {p99}"
    );
}
