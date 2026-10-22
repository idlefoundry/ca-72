//! Prints the strip's places in the strip's own units (y from the rail's top): for laying out a
//! skin pack's drawing of the strip (decisions.md R-SKIN-PACKS). `cargo run -p ca72-panel
//! --example strip_places`.
use ca72_panel::art::PANEL_H;
use ca72_panel::strip::{self, Bank, Readout};

fn main() {
    let y = |v: f64| v - PANEL_H;
    println!(
        "rail: 0 to {}, keys' middle {}",
        strip::RAIL,
        y(strip::RAIL_Y)
    );
    for (a, b, t) in strip::SECTIONS {
        println!("section {t:?}: x {a} to {b}");
    }
    println!("rows: {:?}", strip::ROWS.map(y));
    // The rail's keys: the plug-ins' (ARP last but the settings), and the standalone's with FX
    // after ARP and the name a key shorter; both with the wider GLOBAL before KEYS
    // (decisions.md R-ARP, R-FX-DRAWER, R-GLOBAL-KEY).
    for rail in [strip::Rail::Plugin, strip::Rail::Effects] {
        let (x, yy, w, h) = rail.name();
        println!("{rail:?} rail: name {x} {} {w} {h}", y(yy));
        for (x, w, t) in rail.keys() {
            println!("{rail:?} rail: key {t:?} middle {x}, {w} wide");
        }
    }
    let (x, yy, w, h) = strip::DISPLAY;
    println!("display: {x} {} {w} {h}", y(yy));
    for b in Bank::ALL {
        let (x, yy, w, h) = b.recess();
        println!(
            "bank {:?}: recess {x} {} {w} {h}; tabs {:?}",
            b.words(),
            y(yy),
            (0..b.words().len())
                .map(|i| {
                    let (a, c) = b.tab(i);
                    (a, y(c))
                })
                .collect::<Vec<_>>()
        );
    }
    for r in Readout::ALL {
        let (x, yy, w, h) = r.window();
        println!("readout {r:?}: {x} {} {w} {h}", y(yy));
    }
    for (p, x, yy) in strip::KNOBS {
        println!("knob {p}: {x} {}", y(yy));
    }
    let (lx, ly) = strip::LEFT_HAND;
    println!("left hand origin: {lx} {}", y(ly));
}
