//! Skin packs (decisions.md R-SKIN-PACKS): the official looks are packs in the same format as
//! anyone's and draw as the built-in looks do; a pack's layout moves what is drawn and what a
//! click finds; a pack that is refused, or whose typeface cannot print its legends, falls back.

use std::path::Path;

use ca72_panel::look::Look;
use ca72_panel::{CONTROLS, Lettering, Renderer, Scene, Skin, Target, interact};
use skin_pack::{Context, Files, Limits, Pack, Report, Severity};

#[path = "../packs/official/files.rs"]
mod files;

fn official_context<'a>(pictures: &'a [(&'static str, &'static [u8])]) -> Context<'a> {
    Context {
        plugin: "ca-72",
        official: pictures,
        official_licence: "GPL-3.0-or-later",
        built_in: true,
        test_drawings: true,
    }
}

/// The official pack `key`, checked as any pack is.
fn official(key: &str) -> Pack {
    let files = files::files(key).expect("an official pack");
    let mut report = Report::default();
    let files = Files::from_memory(
        files.iter().map(|(p, b)| (p.as_str(), b.as_slice())),
        &Limits::default(),
        &mut report,
    );
    let pictures = ca72_panel::skin::official_pictures();
    Pack::check(
        files,
        report,
        &Limits::default(),
        &official_context(&pictures),
    )
    .unwrap_or_else(|r| panic!("{key}:\n{r}"))
}

#[test]
fn the_official_looks_are_packs_in_the_one_format() {
    for key in ["classic", "realistic"] {
        let pack = official(key);
        // Its only reviews: that its pictures were generated with a tool (whether such pictures
        // are protected differs between countries).
        for f in &pack.report.findings {
            if f.severity == Severity::Review {
                assert_eq!(f.code, "provenance.generated", "{key}: {f}");
            }
        }
        let notes = pack
            .report
            .findings
            .iter()
            .filter(|f| f.code == "asset.official")
            .count();
        assert_eq!(notes, if key == "realistic" { 27 } else { 0 }, "{key}");
        assert_eq!(pack.manifest.license, "GPL-3.0-or-later");
    }
}

fn frame(look: Look, scale: f64) -> Vec<u8> {
    let mut r = Renderer::with_pack(look, Lettering::Standard, scale, 1.0);
    let mut scene = Scene {
        values: [0.5; CONTROLS.len()],
        ..Scene::default()
    };
    scene.values[3] = 0.8;
    r.render(&scene);
    r.frame().data().to_vec()
}

/// Read back as packs, the official looks draw exactly as the built-in ones.
#[test]
fn the_official_packs_draw_as_the_built_in_looks() {
    for (key, skin) in [("classic", Skin::Drawn), ("realistic", Skin::Worn)] {
        let look = Look::from_pack(official(key), &Limits::default())
            .unwrap_or_else(|r| panic!("{key}:\n{r}"));
        assert_eq!(look.base, skin);
        assert!(
            frame(look, 0.3) == frame(Look::of(skin), 0.3),
            "{key} drawn differently"
        );
    }
}

fn starter() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/skin-packs/starter"
    ))
}

/// The CC0 starter: a pack of its own layout and drawing, every control found where it is drawn.
#[test]
fn the_starter_pack_moves_the_controls_and_what_a_click_finds() {
    let look = Look::open(starter()).unwrap_or_else(|r| panic!("{r}"));
    assert_eq!(look.licence, "CC0-1.0");
    assert!(look.art.is_some());
    assert!(look.report.has("declaration.missing"), "{}", look.report);
    let a = look.arrangement.clone();
    let mut r = Renderer::with_pack(look, Lettering::Standard, 0.3, 1.0);
    r.render(&Scene::default());
    let official = ca72_panel::Arrangement::official();
    let mut moved = 0;
    for (i, c) in a.controls.iter().enumerate() {
        let (x, y) = c.centre();
        assert_eq!(
            interact::hit(r.layout(), &a, x, y),
            Some(Target::Control(i)),
            "{}",
            c.param
        );
        moved += usize::from(c.centre() != official.controls[i].centre());
    }
    assert!(moved >= 40, "{moved} moved");
    let (px, py) = a.power_at();
    assert_eq!(interact::hit(r.layout(), &a, px, py), Some(Target::Power));
}

fn copy(from: &Path, to: &Path) {
    for e in std::fs::read_dir(from).expect("a folder") {
        let e = e.expect("an entry");
        let target = to.join(e.file_name());
        if e.file_type().expect("a type").is_dir() {
            std::fs::create_dir_all(&target).expect("a folder");
            copy(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).expect("copied");
        }
    }
}

/// The starter copied, its `edit` made to its files.
fn edited(edit: impl FnOnce(&Path)) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a folder");
    copy(starter(), dir.path());
    edit(dir.path());
    dir
}

fn replace(path: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(path).expect("read");
    assert!(text.contains(from), "{from}");
    std::fs::write(path, text.replacen(from, to, 1)).expect("written");
}

#[test]
fn a_pack_whose_layout_is_refused_is_not_loaded() {
    let dir = edited(|d| replace(&d.join("layout.toml"), "at = [333, 120]", "at = [130, 130]"));
    let r = Look::open(dir.path()).expect_err("refused");
    assert!(r.has("layout.overlap"), "{r}");
    let dir = edited(|d| {
        replace(
            &d.join("skin.toml"),
            "base = \"classic\"",
            "base = \"neon\"",
        )
    });
    assert!(
        Look::open(dir.path())
            .expect_err("refused")
            .has("look.base")
    );
    let dir = edited(|d| {
        std::fs::write(
            d.join("art/panel.svg"),
            "<svg xmlns='http://www.w3.org/2000/svg'><script>x()</script></svg>",
        )
        .expect("written");
    });
    assert!(
        Look::open(dir.path())
            .expect_err("refused")
            .has("svg.refused")
    );
}

#[test]
fn a_packs_pictures_replace_the_built_in_ones_by_slot() {
    let dir = edited(|d| {
        std::fs::write(
            d.join("art/knob.svg"),
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10'><circle cx='5' cy='5' r='5' fill='#c33'/></svg>",
        )
        .expect("written");
        replace(
            &d.join("skin.toml"),
            "base = \"classic\"",
            "base = \"realistic\"",
        );
        let text = std::fs::read_to_string(d.join("skin.toml")).expect("read");
        std::fs::write(
            d.join("skin.toml"),
            text.replace(
                "[look]",
                "[[assets]]\npath = \"art/knob.svg\"\norigin = \"original\"\n\n[look]",
            ) + "\n[look.pictures]\nknob = \"art/knob.svg\"\n",
        )
        .expect("written");
    });
    let look = Look::open(dir.path()).unwrap_or_else(|r| panic!("{r}"));
    assert_eq!(look.pictures.len(), 1);
    let knob = &look.pictures["knob"];
    assert_eq!((knob.width(), knob.height()), (512, 512));
    let base = Look::realistic();
    assert!(frame(look, 0.3) != frame(base, 0.3));
    // A slot that is not one of the plug-in's is refused.
    let dir = edited(|d| {
        let text = std::fs::read_to_string(d.join("skin.toml")).expect("read");
        std::fs::write(
            d.join("skin.toml"),
            text + "\n[look.pictures]\nlogo = \"art/panel.svg\"\n",
        )
        .expect("written");
    });
    assert!(
        Look::open(dir.path())
            .expect_err("refused")
            .has("look.picture")
    );
}

/// A typeface that cannot print a legend is not used: the built-in one is, and the pack goes
/// to review; one that can is used.
#[test]
fn a_packs_typeface_is_used_only_if_it_prints_every_legend() {
    let font = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../third_party/urw-gothic/URWGothic-Demi.otf"
    ))
    .expect("the built-in font");
    let with_font = |legend: &'static str| {
        let font = font.clone();
        edited(move |d| {
            std::fs::create_dir_all(d.join("fonts")).expect("a folder");
            std::fs::write(d.join("fonts/Gothic-Demi.otf"), &font).expect("written");
            let text = std::fs::read_to_string(d.join("skin.toml")).expect("read");
            std::fs::write(
                d.join("skin.toml"),
                text.replace(
                    "[look]",
                    "[[assets]]\npath = \"fonts/Gothic-Demi.otf\"\norigin = \"third-party\"\nsource = \"URW base 35\"\nlicense = \"AGPL-3.0-only WITH Font-exception-2.0\"\n\n[look]",
                ) + "fonts = { regular = \"fonts/Gothic-Demi.otf\" }\n",
            )
            .expect("written");
            replace(&d.join("layout.toml"), "text = \"TUNE\"", legend);
        })
    };
    let dir = with_font("text = \"TUNE\"");
    let look = Look::open(dir.path()).unwrap_or_else(|r| panic!("{r}"));
    assert!(look.fonts.is_some(), "{}", look.report);
    let dir = with_font("text = \"TUNE 調律\"");
    let look = Look::open(dir.path()).unwrap_or_else(|r| panic!("{r}"));
    assert!(look.fonts.is_none());
    assert!(look.report.has("look.fonts"), "{}", look.report);
    // Drawn all the same, in the built-in typeface.
    let _ = frame(look, 0.3);
}
