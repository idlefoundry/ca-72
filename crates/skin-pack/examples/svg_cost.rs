//! What a pack's drawing costs (decisions.md R-SKIN-PACKS): `cargo run --release -p skin-pack
//! --example svg_cost -- <drawing.svg> [side]`. Prints its counts against the limits and how long
//! it takes to draw `side` pixels across (3000 by default; waited for up to a minute), then the
//! same with each of its costly kinds of part left out, to find what costs.

use std::time::Instant;

use skin_pack::Limits;
use skin_pack::svg::{self, Drawn};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("svg_cost <drawing.svg> [side]");
        std::process::exit(2);
    };
    let side: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let text = std::fs::read_to_string(path).expect("the drawing");
    let limits = Limits::default();
    let time = |label: &str, text: &str| {
        let t = Instant::now();
        match svg::parse(text.as_bytes(), &limits) {
            Ok((tree, cost)) => {
                let parsed = t.elapsed().as_secs_f64() * 1000.0;
                let drawn = match svg::test_draw(&tree, side, 60_000) {
                    Drawn::In(ms) => format!("{ms} ms"),
                    other => format!("{other:?}"),
                };
                println!("{label:<28} read {parsed:6.0} ms, drawn {drawn:>10}   {cost:?}");
            }
            Err(e) => println!("{label:<28} refused: {e}"),
        }
    };
    time("whole", &text);
    let without = |pat: &str| -> String {
        // Each element whose text starts with `pat` taken out (a rough knife, for measuring).
        let mut out = String::new();
        let mut rest = text.as_str();
        while let Some(i) = rest.find(pat) {
            out.push_str(&rest[..i]);
            let tail = &rest[i..];
            let end = tail.find("/>").map(|e| e + 2).unwrap_or(tail.len());
            rest = &tail[end..];
        }
        out.push_str(rest);
        out
    };
    for (label, pat) in [
        ("without <use>", "<use "),
        ("without <path>", "<path "),
        ("without filtered <rect>", "<rect x='1"),
        ("without <image>", "<image "),
    ] {
        time(label, &without(pat));
    }
}
