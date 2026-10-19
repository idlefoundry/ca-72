//! Checks a skin pack as the CA-72 will (decisions.md R-SKIN-PACKS) and draws a preview of it:
//! `cargo run --release -p ca72-panel --example check_pack -- <folder> [preview.png]`.
//!
//! Prints what the checks found: what rejects the pack (the plug-in will not load it), what a
//! marketplace's review must look at (the pack loads, but is not cleared for sale: the checks
//! never clear anything), and notes. With a PNG's path, writes the panel drawn in the pack's
//! look, every control at the middle of its travel, in both letterings (`<name>.png` and
//! `<name>-large.png`). Exits 1 if the pack is rejected. The pack may be a folder or a pack file
//! (`.ifskin`). `--trust <key-id>:<public-key>` checks its signature against that key too (the
//! marketplace's, before the plug-in has it built in).

use std::path::Path;

use ca72_panel::{CONTROLS, Lettering, Look, Renderer, Scene};

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut keys: Vec<skin_pack::signature::TrustedKey> = ca72_panel::look::TRUSTED_KEYS.to_vec();
    while let Some(i) = args.iter().position(|a| a == "--trust") {
        let given = args.get(i + 1).cloned().unwrap_or_default();
        let (id, public) = given.split_once(':').unwrap_or(("", ""));
        let Some(public) = skin_pack::signature::public_key(public) else {
            eprintln!("--trust <key-id>:<public key, 64 hexadecimal digits>");
            std::process::exit(2);
        };
        keys.push(skin_pack::signature::TrustedKey {
            id: Box::leak(id.to_owned().into_boxed_str()),
            public,
        });
        args.drain(i..=(i + 1).min(args.len() - 1));
    }
    let Some(dir) = args.first() else {
        eprintln!("check_pack <folder or .ifskin> [preview.png] [--trust <key-id>:<public-key>]");
        std::process::exit(2);
    };
    let look = match Look::open_with(Path::new(dir), &skin_pack::Limits::default(), &keys) {
        Ok(look) => look,
        Err(report) => {
            print!("{report}");
            println!("REJECTED: the CA-72 will not load this pack.");
            std::process::exit(1);
        }
    };
    print!("{}", look.report);
    println!(
        "LOADS: {} {} ({}), drawn {}, {}{}{}.",
        look.name,
        look.version,
        look.licence,
        if look.base == ca72_panel::Skin::Worn {
            "realistic"
        } else {
            "classic"
        },
        if look.first_party() {
            "published by Idle Foundry, "
        } else if look.signed.is_some() {
            "published by the marketplace, "
        } else {
            "unsigned, "
        },
        if look.fonts.is_some() {
            "in its own typeface"
        } else {
            "in the built-in typeface"
        },
        if look.report.needs_review() {
            "; a review must look at the findings above before it is listed"
        } else {
            ""
        }
    );
    let Some(out) = args.get(1) else { return };
    let scene = Scene {
        values: [0.5; CONTROLS.len()],
        ..Scene::default()
    };
    for (lettering, suffix) in [(Lettering::Standard, ""), (Lettering::Large, "-large")] {
        let mut r = Renderer::with_pack(look.clone(), lettering, 0.5, 1.0);
        r.render(&scene);
        let path = match out.strip_suffix(".png") {
            Some(stem) => format!("{stem}{suffix}.png"),
            None => format!("{out}{suffix}.png"),
        };
        r.frame().save_png(&path).expect("the preview written");
        println!("preview: {path}");
    }
}
