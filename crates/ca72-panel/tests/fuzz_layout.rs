//! Fuzzing the CA-72's layout reader and its looks (decisions.md R-SKIN-PACKS): the official
//! layout and the starter's mutated (numbers made vast, strange or negative; blocks repeated or
//! cut), each read; one that is read is drawn and clicked. Nothing may panic. `SKIN_PACK_FUZZ`
//! sets the cases (200 by default).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use ca72_panel::arrangement::{self, Arrangement};
use ca72_panel::look::Look;
use ca72_panel::{Lettering, Renderer, Scene, interact};
use skin_pack::Report;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// `text`'s lines mutated: numbers swapped for strange ones, lines repeated, cut or moved.
fn mutate(text: &str, rng: &mut Rng) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    for _ in 0..1 + rng.below(6) {
        let i = rng.below(lines.len());
        match rng.below(5) {
            0 => {
                let strange = [
                    "1e308",
                    "-1e308",
                    "nan",
                    "inf",
                    "-inf",
                    "0",
                    "-0",
                    "3108",
                    "-130",
                    "795",
                    "1e-300",
                    "240.0000001",
                    "15.99",
                ];
                let w = strange[rng.below(strange.len())];
                let line = lines[i].clone();
                // The first number on the line, swapped.
                if let Some(start) = line.find(|c: char| c.is_ascii_digit()) {
                    let end = line[start..]
                        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                        .map_or(line.len(), |e| start + e);
                    lines[i] = format!("{}{w}{}", &line[..start], &line[end..]);
                }
            }
            1 => {
                let l = lines[i].clone();
                lines.insert(i, l);
            }
            2 => {
                lines.remove(i);
            }
            3 => {
                let j = rng.below(lines.len());
                lines.swap(i, j);
            }
            _ => {
                let l = lines[i].clone();
                lines[i] = l.chars().rev().collect();
            }
        }
        if lines.is_empty() {
            break;
        }
    }
    lines.join("\n")
}

#[test]
fn layouts_never_panic() {
    let cases: usize = std::env::var("SKIN_PACK_FUZZ")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let seeds = [
        arrangement::OFFICIAL.to_owned(),
        include_str!("../../../docs/skin-packs/starter/layout.toml").to_owned(),
    ];
    let mut rng = Rng(0x51_7cc1_b727_220a);
    let mut drawn = 0;
    for k in 0..cases {
        let text = mutate(&seeds[k % 2], &mut rng);
        let ok = catch_unwind(AssertUnwindSafe(|| {
            let mut report = Report::default();
            if let Some(a) = Arrangement::read(&text, &mut report) {
                // Read: drawn small (one in ten, for time), and clicked at every control.
                if k % 10 == 0 {
                    let look = Look {
                        arrangement: Arc::new(a.clone()),
                        ..Look::classic()
                    };
                    let mut r = Renderer::with_pack(look, Lettering::Large, 0.1, 1.0);
                    r.render(&Scene::default());
                    for c in &a.controls {
                        let (x, y) = c.centre();
                        let _ = interact::hit(r.layout(), &a, x, y);
                    }
                    drawn += 1;
                }
            }
        }))
        .is_ok();
        if !ok {
            let path = std::env::temp_dir().join(format!("ca72-layout-fuzz-{k}.toml"));
            let _ = std::fs::write(&path, &text);
            panic!(
                "layout case {k} panicked; the layout is in {}",
                path.display()
            );
        }
    }
    let _ = drawn;
}
