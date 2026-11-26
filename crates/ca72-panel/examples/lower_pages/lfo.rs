//! LFO lower page fixtures and ivory silkscreen for the 3D Cycles lower-panel exporter.

use ca72_panel::Arrangement;
use ca72_panel::art::{self, KEY, Layout, Lettering};
use ca72_panel::controls::{Kind, LFO_PARTS, lfo_index};
use ca72_panel::fonts::Fonts;
use ca72_panel::strip::lfo;
use serde_json::{Value, json};

use super::super::{part, screen};

fn demo_value(n: usize, k: usize) -> f64 {
    match k {
        // WAVEFORM: Triangle, Sawtooth, Square, Shark across LFO 1..4.
        0 => [0.0, 0.4, 0.6, 0.2][n],
        // RATE (normalized skewed 0.01..50 Hz).
        1 => [0.42, 0.30, 0.56, 0.24][n],
        // TIMED key (0/1).
        2 => 0.0,
        // PHASE (0..360 deg).
        3 => [0.0, 0.25, 0.5, 0.75][n],
        // FADE (0.5 = OFF at 12 o'clock).
        4 => [0.5, 0.68, 0.35, 0.5][n],
        // MODE toggle (0 = FREE, 0.5 = RETRIG, 1 = ENV).
        5 => [0.0, 0.5, 1.0, 0.0][n],
        // MONO key (0/1).
        6 => [1.0, 1.0, 0.0, 1.0][n],
        // POLARITY UNI key (0/1).
        7 => [0.0, 1.0, 0.0, 0.0][n],
        // ON key (0/1).
        8 => [1.0, 1.0, 1.0, 0.0][n],
        // OUT jack LEVEL.
        _ => 1.0,
    }
}

fn key_label(k: usize) -> &'static str {
    match k {
        2 => "TIMED",
        6 => "MONO",
        7 => "UNI",
        8 => "ON",
        _ => "",
    }
}

pub(super) fn page() -> (Vec<Value>, String) {
    let arrangement = Arrangement::official();
    let fonts = Fonts::new();
    let lettering = Lettering::Standard;
    let layout = Layout::measure_in(&fonts, lettering, &arrangement);

    let mut fixtures = Vec::new();
    for n in 0..4 {
        let p = lfo::places(n);

        // Scope glass aperture (`lfo::glass`) and MODULATES readout window (`p.screen`).
        fixtures.push(screen(&format!("lfo_scope_{n}"), "field", lfo::glass(n)));
        fixtures.push(screen(&format!("lfo_screen_{n}"), "readout", p.screen));

        for k in 0..LFO_PARTS.len() {
            let c = arrangement.controls[lfo_index(n, k)];
            let at = c.centre();
            let val = demo_value(n, k);
            let fixture = match c.kind {
                Kind::Selector(_) => {
                    let r = art::skirt(&c.kind);
                    let mut entry = part("selector", at, (2.0 * r, 2.0 * r));
                    entry["param"] = json!(c.param);
                    entry["label"] = json!(c.label);
                    entry["sprite"] = json!("knob_pointer");
                    entry["steps"] = json!(6);
                    entry["hit_r"] = json!(r);
                    entry["value"] = json!(val);
                    entry
                }
                Kind::Knob { .. } => {
                    let r = art::skirt(&c.kind);
                    let mut entry = part("knob", at, (2.0 * r, 2.0 * r));
                    entry["param"] = json!(c.param);
                    entry["label"] = json!(c.label);
                    entry["sprite"] = json!("knob_std");
                    entry["hit_r"] = json!(art::STD.skirt + 8.0);
                    entry["value"] = json!(val);
                    entry
                }
                Kind::Toggle { w, h, across } => {
                    let mut entry = part("toggle", at, (w, h));
                    entry["param"] = json!(c.param);
                    entry["label"] = json!(c.label);
                    entry["across"] = json!(across);
                    entry["sprite"] = json!("toggle_horiz");
                    entry["steps"] = json!(3);
                    entry["value"] = json!(val);
                    entry
                }
                Kind::Key => {
                    let mut entry = part("tab", at, KEY);
                    entry["param"] = json!(c.param);
                    entry["label"] = json!(key_label(k));
                    entry["value"] = json!(val);
                    entry["lit"] = json!(val >= 0.5);
                    entry
                }
                Kind::Jack => {
                    let d = 2.0 * lfo::JACK_R;
                    let mut entry = part("jack", at, (d, d));
                    entry["param"] = json!(c.param);
                    entry["label"] = json!(c.label);
                    entry
                }
                _ => continue,
            };
            fixtures.push(fixture);
        }
    }

    // Official LFO ivory silkscreen in absolute CA-72 coordinates: column rules, titles,
    // legends, dial scales (RATE/PHASE/FADE), and official WAVEFORM glyphs; no baked tab caps.
    let svg = lfo::silkscreen(lettering, &layout);
    (fixtures, svg)
}
