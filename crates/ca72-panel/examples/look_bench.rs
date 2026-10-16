//! What a look costs the panel's renderer: `cargo run --release -p ca72-panel --example
//! look_bench`. For each skin at 0.44 and 0.87 pixels a drawing unit (the window on a 1920 by
//! 1080 screen, and the same on a Retina one): opening it (the renderer made and its
//! background drawn), its first frame, a knob turned, a switch flipped and the power turned off
//! and on, each the median and the slowest of its runs. Run on two builds to compare them.
//! `--packs`: also skin packs' (decisions.md R-SKIN-PACKS): a pack's folder read and checked
//! (the CC0 starter, and the official REALISTIC written out as a folder), and the starter drawn.

#[path = "../packs/official/files.rs"]
#[allow(dead_code)]
mod files;

use std::time::{Duration, Instant};

use ca72_panel::{CONTROLS, Lettering, Look, Renderer, Scene, Skin};

fn main() {
    for (skin, name) in [(Skin::Drawn, "classic"), (Skin::Worn, "realistic")] {
        for scale in [0.44, 0.87] {
            measure(&|| Look::of(skin), name, scale);
        }
    }
    if std::env::args().any(|a| a == "--packs") {
        packs();
    }
}

fn packs() {
    let starter = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/skin-packs/starter"
    ));
    let dir = std::env::temp_dir().join(format!("ca72-look-bench-{}", std::process::id()));
    let realistic = dir.join("realistic");
    for (path, bytes) in files::files("realistic").expect("the official pack") {
        let p = realistic.join(path);
        std::fs::create_dir_all(p.parent().expect("a folder")).expect("a folder");
        std::fs::write(p, bytes).expect("written");
    }
    // (Its id is the makers' own, kept for the built-in packs: renamed, as anyone's.)
    let manifest = realistic.join("skin.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "com.idlefoundry.ca72.realistic",
            "org.example.realistic-copy",
        ),
    )
    .expect("written");
    for (name, path) in [
        ("the starter", starter),
        ("REALISTIC as a folder", realistic.as_path()),
    ] {
        let mut d: Vec<Duration> = (0..7)
            .map(|_| {
                let t = Instant::now();
                Look::open(path).unwrap_or_else(|r| panic!("{name}: {r}"));
                t.elapsed()
            })
            .collect();
        d.sort();
        println!(
            "{name} read and checked: median {:7.2} ms, slowest {:7.2} ms",
            ms(d[d.len() / 2]),
            ms(d[d.len() - 1])
        );
    }
    let look = Look::open(starter).expect("the starter");
    for scale in [0.44, 0.87] {
        measure(
            &|| look.clone(),
            "the starter (its layout and drawing)",
            scale,
        );
    }
    let look = Look::open(&realistic).expect("REALISTIC as a folder");
    for scale in [0.44, 0.87] {
        measure(&|| look.clone(), "REALISTIC as a pack", scale);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

fn measure(look: &dyn Fn() -> Look, name: &str, scale: f64) {
    let half = Scene {
        values: [0.5; CONTROLS.len()],
        ..Scene::default()
    };
    let mut open = Vec::new();
    let mut first = Vec::new();
    for _ in 0..7 {
        let t = Instant::now();
        let mut r = Renderer::with_pack(look(), Lettering::Standard, scale, 1.0);
        open.push(t.elapsed());
        let t = Instant::now();
        r.render(&half);
        first.push(t.elapsed());
    }
    let mut r = Renderer::with_pack(look(), Lettering::Standard, scale, 1.0);
    let mut scene = half.clone();
    r.render(&scene);
    let index = |p: &str| CONTROLS.iter().position(|c| c.param == p).expect(p);
    let cutoff = index("cutoff");
    let knob = runs(60, |i| {
        scene.values[cutoff] = 0.3 + 0.005 * i as f64;
        let t = Instant::now();
        r.render(&scene);
        t.elapsed()
    });
    let switch = index("osc1_on");
    let flip = runs(60, |i| {
        scene.values[switch] = (i % 2) as f64;
        let t = Instant::now();
        r.render(&scene);
        t.elapsed()
    });
    let power = runs(60, |i| {
        scene.power = i % 2 == 0;
        let t = Instant::now();
        r.render(&scene);
        t.elapsed()
    });
    println!("{name} at {scale}:");
    for (what, d) in [
        ("opened", open),
        ("first frame", first),
        ("knob turned", knob),
        ("switch flipped", flip),
        ("power off and on", power),
    ] {
        let mut d = d;
        d.sort();
        println!(
            "  {what:<18} median {:7.2} ms, slowest {:7.2} ms",
            ms(d[d.len() / 2]),
            ms(d[d.len() - 1])
        );
    }
}

fn runs(n: usize, mut f: impl FnMut(usize) -> Duration) -> Vec<Duration> {
    (0..n).map(&mut f).collect()
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}
