//! Writes the stress packs (decisions.md R-SKIN-PACKS, `packs/stress.rs`) into a folder, checks
//! each as the CA-72 will, and measures it: `cargo run --release -p ca72-panel --example
//! stress_packs -- <folder> [--previews]`. Point the folder at a `Skins` folder to try them in
//! the plug-in (LIFECYCLE's are signed with a test key the plug-in does not trust: there, they
//! load as unsigned). With `--previews`, each pack's panel in both letterings, in `<folder>/
//! previews` (outside the packs).

#[path = "../packs/stress.rs"]
mod stress;

use std::time::Instant;

use ca72_panel::look::Look;
use ca72_panel::{CONTROLS, Lettering, Renderer, Scene};
use skin_pack::Limits;

/// A run's name and what it is for.
struct Pack {
    name: String,
    about: &'static str,
}

/// This process's resident memory, in MB (macOS and Linux).
fn rss_mb() -> f64 {
    std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map_or(0.0, |kb| kb / 1024.0)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(dir) = args.first() else {
        eprintln!("stress_packs <folder> [--previews]");
        std::process::exit(2);
    };
    let previews = args.iter().any(|a| a == "--previews");
    let dir = std::path::Path::new(dir);
    std::fs::create_dir_all(dir).expect("the folder");
    let (_, key) = stress::test_key();
    let half = Scene {
        values: [0.5; CONTROLS.len()],
        ..Scene::default()
    };
    // The built-in looks first, as the baseline.
    let mut runs: Vec<(String, &str, Option<std::path::PathBuf>, Option<Look>)> = vec![
        (
            "built-in CLASSIC".into(),
            "the baseline",
            None,
            Some(Look::classic()),
        ),
        (
            "built-in REALISTIC".into(),
            "the baseline",
            None,
            Some(Look::realistic()),
        ),
    ];
    let packs = stress::all();
    for pack in &packs {
        runs.push((
            pack.name.clone(),
            pack.about,
            Some(stress::write(dir, pack)),
            None,
        ));
    }
    for (name, about, path, built_in) in runs {
        let t = Instant::now();
        let before = rss_mb();
        let result = match (&path, built_in) {
            (Some(path), _) => Look::open_with(path, &Limits::default(), &[key]),
            (None, Some(look)) => Ok(look),
            (None, None) => continue,
        };
        let pack = Pack { name, about };
        let read = t.elapsed();
        println!("\n== {} — {}", pack.name, pack.about);
        let look = match result {
            Ok(look) => look,
            Err(report) => {
                print!("{report}");
                println!("REFUSED (read in {:.0} ms)", read.as_secs_f64() * 1000.0);
                continue;
            }
        };
        for f in &look.report.findings {
            if f.severity != skin_pack::Severity::Note || f.code.starts_with("look.") {
                println!("  {f}");
            }
        }
        let after_read = rss_mb();
        for scale in [0.44, 0.87] {
            let t = Instant::now();
            let mut r = Renderer::with_pack(look.clone(), Lettering::Standard, scale, 1.0);
            let open = t.elapsed();
            let t = Instant::now();
            r.render(&half);
            let first = t.elapsed();
            let mut s = half.clone();
            let mut slowest = 0.0f64;
            for i in 0..30 {
                s.values[28] = 0.3 + 0.01 * f64::from(i);
                let t = Instant::now();
                r.render(&s);
                slowest = slowest.max(t.elapsed().as_secs_f64() * 1000.0);
            }
            println!(
                "  at {scale}: opened {:.0} ms, first frame {:.0} ms, a knob's frame at most {:.1} ms, memory {:.0} MB (read {:.0} MB of it)",
                open.as_secs_f64() * 1000.0,
                first.as_secs_f64() * 1000.0,
                slowest,
                rss_mb() - before,
                after_read - before
            );
            if previews && scale == 0.44 {
                let out = dir.join("previews");
                std::fs::create_dir_all(&out).expect("a folder");
                r.frame()
                    .save_png(out.join(format!("{}.png", pack.name)))
                    .expect("a preview");
                let mut large = Renderer::with_pack(look.clone(), Lettering::Large, scale, 1.0);
                large.render(&half);
                large
                    .frame()
                    .save_png(out.join(format!("{}-large.png", pack.name)))
                    .expect("a preview");
            }
        }
        println!(
            "  read and checked in {:.0} ms",
            read.as_secs_f64() * 1000.0
        );
    }
}
