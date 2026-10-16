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
        keys: &[],
        plugin_version: None,
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
    // Kept as the drawing it is; drawn at the built-in knob's size as a renderer needs it.
    let knob = look.pictures["knob"].picture((512, 512)).expect("drawn");
    assert_eq!((knob.width(), knob.height()), (512, 512));
    assert!(matches!(
        look.pictures["knob"],
        ca72_panel::skin::Source::Svg(_)
    ));
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
    // A font named as another kind of file is refused: it would have skipped the font's checks.
    let dir = edited(|d| {
        std::fs::write(d.join("notes.txt"), "plain text, not a font").expect("written");
        let text = std::fs::read_to_string(d.join("skin.toml")).expect("read");
        std::fs::write(
            d.join("skin.toml"),
            text.replace(
                "[look]",
                "[[assets]]\npath = \"notes.txt\"\norigin = \"original\"\n\n[look]",
            ) + "fonts = { regular = \"notes.txt\" }\n",
        )
        .expect("written");
    });
    assert!(
        Look::open(dir.path())
            .expect_err("refused")
            .has("look.file")
    );
    // Drawn all the same, in the built-in typeface.
    let _ = frame(look, 0.3);
}

// ---- Pack files, signatures and updates (the marketplace's: decisions.md R-SKIN-PACKS).

use skin_pack::signature::{self, Publisher, TrustedKey};

/// A key for the tests (from fixed bytes: never one the plug-in trusts).
fn test_key() -> (String, TrustedKey) {
    let secret = "07".repeat(32);
    let key = signature::signing_key(&secret).expect("a key");
    let public = signature::public_key(&signature::public_hex(&key)).expect("its public half");
    (
        secret,
        TrustedKey {
            id: "test-1",
            public,
        },
    )
}

/// The starter's files, its manifest's id and version changed, read from a folder.
fn starter_files(id: &str, version: &str) -> Files {
    let dir = edited(|d| {
        replace(&d.join("skin.toml"), "org.example.ca72-starter", id);
        replace(
            &d.join("skin.toml"),
            "version = \"1.0.0\"",
            &format!("version = \"{version}\""),
        );
    });
    let mut r = Report::default();
    Files::read_dir(dir.path(), &Limits::default(), &mut r)
}

/// `files` signed with the test key, as a pack file in `dir`, named `name`.
fn signed_pack_file(
    dir: &Path,
    name: &str,
    files: &Files,
    (id, version): (&str, &str),
    publisher: Publisher,
) -> std::path::PathBuf {
    let (secret, _) = test_key();
    let key = signature::signing_key(&secret).expect("a key");
    let sig = signature::sign(
        files,
        (id, version),
        ("test-1", &key),
        publisher,
        "2026-10-16",
    );
    let mut r = Report::default();
    let with_sig = Files::from_memory(
        files.iter().chain([(signature::SIGNATURE, sig.as_bytes())]),
        &Limits::default(),
        &mut r,
    );
    let path = dir.join(name);
    std::fs::write(
        &path,
        skin_pack::archive::write(&with_sig).expect("written"),
    )
    .expect("saved");
    path
}

#[test]
fn a_signed_pack_file_loads_as_published_and_unsigned_without_its_key() {
    let out = tempfile::tempdir().expect("a folder");
    let files = starter_files("org.example.night", "1.0.0");
    let path = signed_pack_file(
        out.path(),
        "night.ifskin",
        &files,
        ("org.example.night", "1.0.0"),
        Publisher::Marketplace,
    );
    let (_, key) = test_key();
    let look = Look::open_with(&path, &Limits::default(), &[key]).unwrap_or_else(|r| panic!("{r}"));
    assert_eq!(
        look.signed.as_ref().map(|s| s.publisher),
        Some(Publisher::Marketplace)
    );
    assert!(!look.first_party());
    // Without the key (an older plug-in, a newer key): as if unsigned, and it loads.
    let look = Look::open(&path).unwrap_or_else(|r| panic!("{r}"));
    assert!(look.signed.is_none());
    assert!(look.report.has("signature.unknown-key"), "{}", look.report);
}

#[test]
fn a_pack_changed_after_it_was_signed_is_refused() {
    let out = tempfile::tempdir().expect("a folder");
    let files = starter_files("org.example.night", "1.0.0");
    let path = signed_pack_file(
        out.path(),
        "night.ifskin",
        &files,
        ("org.example.night", "1.0.0"),
        Publisher::Marketplace,
    );
    // Unpacked, one legend changed, packed again.
    let mut r = Report::default();
    let read = skin_pack::archive::read_file(&path, &Limits::default(), &mut r);
    let layout = String::from_utf8(read.get("layout.toml").expect("its layout").to_vec())
        .expect("text")
        .replacen("\"TUNE\"", "\"TUNED\"", 1);
    let changed = Files::from_memory(
        read.iter().map(|(p, b)| {
            if p == "layout.toml" {
                (p, layout.as_bytes())
            } else {
                (p, b)
            }
        }),
        &Limits::default(),
        &mut r,
    );
    std::fs::write(&path, skin_pack::archive::write(&changed).expect("written")).expect("saved");
    let (_, key) = test_key();
    let report = Look::open_with(&path, &Limits::default(), &[key]).expect_err("refused");
    assert!(report.has("signature.invalid"), "{report}");
}

/// The makers' id prefix: refused unsigned, or signed as a seller's; allowed signed as theirs.
#[test]
fn only_a_first_party_signature_allows_the_makers_ids() {
    let out = tempfile::tempdir().expect("a folder");
    let id = "com.idlefoundry.ca72.dusk";
    let files = starter_files(id, "1.0.0");
    let (_, key) = test_key();
    let first = signed_pack_file(
        out.path(),
        "a.ifskin",
        &files,
        (id, "1.0.0"),
        Publisher::FirstParty,
    );
    let look =
        Look::open_with(&first, &Limits::default(), &[key]).unwrap_or_else(|r| panic!("{r}"));
    assert!(look.first_party());
    let seller = signed_pack_file(
        out.path(),
        "b.ifskin",
        &files,
        (id, "1.0.0"),
        Publisher::Marketplace,
    );
    let r = Look::open_with(&seller, &Limits::default(), &[key]).expect_err("refused");
    assert!(r.has("manifest.id"), "{r}");
    let r = Look::open_with(&first, &Limits::default(), &[]).expect_err("refused unsigned");
    assert!(r.has("manifest.id"), "{r}");
    // And never the built-in looks' own ids, signed or not.
    let builtin = ca72_panel::look::REALISTIC;
    let files = starter_files(builtin, "1.0.0");
    let p = signed_pack_file(
        out.path(),
        "c.ifskin",
        &files,
        (builtin, "1.0.0"),
        Publisher::FirstParty,
    );
    assert!(Look::open_with(&p, &Limits::default(), &[key]).is_err());
}

/// An update added beside a pack is the one used; an unsigned copy never replaces a published
/// pack, whatever its version.
#[test]
fn a_packs_update_replaces_it_and_an_unsigned_copy_does_not() {
    let skins = tempfile::tempdir().expect("a folder");
    let (_, key) = test_key();
    let id = "org.example.night";
    signed_pack_file(
        skins.path(),
        "night-1.0.0.ifskin",
        &starter_files(id, "1.0.0"),
        (id, "1.0.0"),
        Publisher::Marketplace,
    );
    signed_pack_file(
        skins.path(),
        "night-1.1.0.ifskin",
        &starter_files(id, "1.1.0"),
        (id, "1.1.0"),
        Publisher::Marketplace,
    );
    let found = ca72_panel::look::discover_with(skins.path(), &[key]);
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].as_ref().map(|l| l.version.as_str()).ok(),
        Some("1.1.0")
    );
    // An unsigned folder of the same id, "version 9": the published 1.1.0 stays.
    let copy = skins.path().join("night-copy");
    std::fs::create_dir_all(&copy).expect("a folder");
    copy_dir_files(&starter_files(id, "9.0.0"), &copy);
    let found = ca72_panel::look::discover_with(skins.path(), &[key]);
    assert_eq!(found.len(), 1);
    let kept = found[0].as_ref().expect("a look");
    assert_eq!(
        (kept.version.as_str(), kept.signed.is_some()),
        ("1.1.0", true)
    );
}

fn copy_dir_files(files: &Files, to: &Path) {
    for (p, b) in files.iter() {
        let path = to.join(p);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a folder");
        std::fs::write(path, b).expect("written");
    }
}

/// A `[look]` field this CA-72 does not read (a later one's) is noted, not refused.
#[test]
fn a_look_field_from_a_later_ca72_is_passed_over() {
    let dir = edited(|d| {
        let text = std::fs::read_to_string(d.join("skin.toml")).expect("read");
        std::fs::write(d.join("skin.toml"), text + "glow = 0.5\n").expect("written");
    });
    let look = Look::open(dir.path()).unwrap_or_else(|r| panic!("{r}"));
    assert!(look.report.has("look.unknown"), "{}", look.report);
}
