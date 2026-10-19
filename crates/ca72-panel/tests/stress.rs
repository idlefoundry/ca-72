//! The stress packs (decisions.md R-SKIN-PACKS, `packs/stress.rs`): each loads, or is refused,
//! as it should; EDGE's controls are found where they are drawn, in both letterings, with MIDI
//! Learn's ring and a tip on them; HEAVY and EVERY PICTURE draw every state; LIFECYCLE's forms
//! resolve to the one that should be used.

#[path = "../packs/stress.rs"]
mod stress;

use ca72_panel::look::{self, Look};
use ca72_panel::ultra::Opening;
use ca72_panel::{CONTROLS, Lettering, Renderer, Scene, Target, interact};
use skin_pack::Limits;
use stress::{Expect, Pack};

fn opened(pack: &Pack, dir: &std::path::Path) -> Result<Look, skin_pack::Report> {
    let (_, key) = stress::test_key();
    Look::open_with(&stress::write(dir, pack), &Limits::default(), &[key])
}

fn check(pack: &Pack) -> Option<Look> {
    let dir = tempfile::tempdir().expect("a folder");
    match (&pack.expect, opened(pack, dir.path())) {
        (Expect::Loads { reviews }, Ok(look)) => {
            for f in &look.report.findings {
                if f.severity == skin_pack::Severity::Review {
                    assert!(reviews.contains(&f.code), "{}: unexpected {f}", pack.name);
                }
            }
            Some(look)
        }
        (Expect::Loads { .. }, Err(r)) => panic!("{} refused:\n{r}", pack.name),
        (Expect::Refused { codes }, Err(r)) => {
            for c in *codes {
                assert!(r.has(c), "{}: not refused for {c}:\n{r}", pack.name);
            }
            None
        }
        (Expect::Refused { .. }, Ok(_)) => panic!("{} loaded", pack.name),
    }
}

/// Every state a frame can be in: values, the lamps, QUALITY's opening's three, a ring, a tip.
fn scenes() -> Vec<Scene> {
    let mut out = Vec::new();
    let half = Scene {
        values: [0.5; CONTROLS.len()],
        ..Scene::default()
    };
    out.push(half.clone());
    for opening in [
        Opening {
            hamster: 1.0,
            run: 1.0,
            stride: 3,
            turn: 40.0,
            ..Opening::default()
        },
        Opening {
            lamp: 1.0,
            level: 0.7,
            ..Opening::default()
        },
        Opening {
            coil: 1.0,
            spark: 0.4,
            level: 1.0,
            ..Opening::default()
        },
    ] {
        out.push(Scene {
            opening,
            power: false,
            overload: 0.8,
            learning: Some(5),
            tip: Some(("A TIP".into(), 1500.0, 300.0)),
            ..half.clone()
        });
    }
    out
}

fn draws(look: &Look) {
    for lettering in [Lettering::Standard, Lettering::Large] {
        let mut r = Renderer::with_pack(look.clone(), lettering, 0.3, 1.0);
        for s in scenes() {
            r.render(&s);
        }
    }
}

#[test]
fn edge_finds_every_control_where_it_is_drawn() {
    let look = check(&stress::edge()).expect("EDGE loads");
    let a = look.arrangement.clone();
    for lettering in [Lettering::Standard, Lettering::Large] {
        let r = Renderer::with_pack(look.clone(), lettering, 0.3, 1.0);
        for (i, c) in a.controls.iter().enumerate() {
            let (x, y) = c.centre();
            // (The arpeggiator's drawer's knobs, the plug-in's own, are found while it is open.)
            let found = if c.place == ca72_panel::controls::Place::Arp {
                interact::hit_arp(x, y)
            } else {
                interact::hit(r.layout(), &a, x, y)
            };
            assert!(
                matches!(found, Some(Target::Control(j) | Target::Legend(j, _)) if j == i),
                "{}: {found:?}",
                c.param
            );
        }
        let (px, py) = a.power_at();
        assert_eq!(interact::hit(r.layout(), &a, px, py), Some(Target::Power));
    }
    draws(&look);
}

#[test]
fn heavy_loads_and_draws() {
    let look = check(&stress::heavy()).expect("HEAVY loads");
    assert_eq!(look.pictures.len(), 27);
    draws(&look);
}

#[test]
fn every_picture_draws_every_state() {
    let look = check(&stress::every_picture()).expect("EVERY PICTURE loads");
    assert_eq!(look.pictures.len(), 27);
    // The two at other proportions noted.
    assert!(
        look.report
            .findings
            .iter()
            .filter(|f| f.code == "look.picture")
            .count()
            >= 2,
        "{}",
        look.report
    );
    draws(&look);
}

#[test]
fn the_minimal_packs_load_and_draw() {
    for pack in stress::minimal() {
        let look = check(&pack).expect("loads");
        draws(&look);
    }
}

#[test]
fn lifecycle_uses_the_signed_newest_and_refuses_the_broken() {
    let packs = stress::lifecycle();
    for p in &packs {
        check(p);
    }
    let skins = tempfile::tempdir().expect("a folder");
    for p in &packs {
        stress::write(skins.path(), p);
    }
    let (_, key) = stress::test_key();
    let found = look::discover_with(skins.path(), &[key]);
    let used: Vec<&Look> = found.iter().filter_map(|f| f.as_ref().ok()).collect();
    assert_eq!(used.len(), 1, "{found:?}");
    assert_eq!(used[0].version, "1.1.0");
    assert!(used[0].signed.is_some());
    let refused: Vec<&str> = found
        .iter()
        .filter_map(|f| f.as_ref().err().map(|(n, _)| n.as_str()))
        .collect();
    assert_eq!(
        refused,
        [
            "stress-lifecycle-broken.ifskin",
            "stress-lifecycle-changed.ifskin"
        ]
    );
}

/// BABEL: a wide typeface used; legends the built-in typeface cannot print flagged; right-to-left
/// legends in a typeface that has them; a typeface without a legend's glyphs not used.
#[test]
fn babel_prints_what_it_can_and_says_what_it_cannot() {
    let packs = stress::babel_packs();
    let looks: Vec<Look> = packs.iter().map(|p| check(p).expect("loads")).collect();
    let font = |i: usize| looks[i].fonts.is_some();
    assert!(font(0), "{}", looks[0].report);
    assert!(!font(1));
    assert!(font(2), "{}", looks[2].report);
    assert!(!font(3));
    for (i, look) in looks.iter().enumerate() {
        if i == 0 || i == 2 {
            let licence = look
                .report
                .findings
                .iter()
                .find(|f| f.code == "font.licence")
                .expect("its font's licence scanned");
            assert_eq!(licence.severity, skin_pack::Severity::Note, "{licence}");
        }
        draws(look);
    }
}
