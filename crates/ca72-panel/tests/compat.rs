//! Packs as they were (decisions.md R-SKIN-PACKS, `tests/compat`): every released format and
//! revision of the panel's controls keeps samples, and every later build must load each one as
//! it was made: each control it places where it placed it, with no finding but the expected.
//! Never edit the samples: a change that fails here is what must give.

use std::path::Path;

use ca72_panel::arrangement::Arrangement;
use ca72_panel::look::Look;
use skin_pack::{Limits, Report, Severity};

fn v1(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/compat/v1")
        .join(path)
}

/// A frozen layout: read, without a rejection or a review but `reviews`, each control it names
/// where it put it.
fn layout_as_it_was(file: &str, reviews: &[&str]) {
    let text = std::fs::read_to_string(v1(file)).expect("the sample");
    let mut report = Report::default();
    let a = Arrangement::read(&text, &mut report).unwrap_or_else(|| panic!("{file}:\n{report}"));
    for f in &report.findings {
        if f.severity == Severity::Review {
            assert!(reviews.contains(&f.code), "{file}: {f}");
        }
    }
    let table: toml::Table = toml::from_str(&text).expect("TOML");
    for entry in table["control"].as_array().expect("controls") {
        let param = entry["param"].as_str().expect("a param");
        let at = entry["at"].as_array().expect("a place");
        let (x, y) = (
            at[0].as_float().or(at[0].as_integer().map(|i| i as f64)),
            at[1].as_float().or(at[1].as_integer().map(|i| i as f64)),
        );
        let c = a
            .controls
            .iter()
            .find(|c| c.param == param)
            .unwrap_or_else(|| panic!("{file}: `{param}` gone"));
        assert_eq!((Some(c.x), Some(c.y)), (x, y), "{file}: `{param}` moved");
    }
}

#[test]
fn the_official_layout_of_v1_loads_as_it_was() {
    layout_as_it_was("official-layout.toml", &[]);
}

#[test]
fn edge_of_v1_loads_as_it_was() {
    layout_as_it_was("edge-layout.toml", &["layout.print-outside"]);
}

#[test]
fn the_starter_of_v1_loads_as_it_was() {
    let look = Look::open(&v1("starter")).unwrap_or_else(|r| panic!("{r}"));
    for f in &look.report.findings {
        if f.severity == Severity::Review {
            assert_eq!(f.code, "declaration.missing", "{f}");
        }
    }
    layout_as_it_was("starter/layout.toml", &[]);
}

/// The stress packs' test key (`packs/stress.rs`: from fixed bytes, never trusted by the
/// plug-in), so that a pack file signed in v1 stays verifiable.
#[test]
fn a_pack_file_signed_in_v1_stays_verifiable() {
    let secret = "5a".repeat(32);
    let key = skin_pack::signature::signing_key(&secret).expect("a key");
    let public = skin_pack::signature::public_key(&skin_pack::signature::public_hex(&key))
        .expect("its public half");
    let trusted = skin_pack::signature::TrustedKey {
        id: "stress-test",
        public,
    };
    let look = Look::open_with(
        &v1("lifecycle-1.1.0.ifskin"),
        &Limits::default(),
        &[trusted],
    )
    .unwrap_or_else(|r| panic!("{r}"));
    assert!(look.signed.is_some(), "{}", look.report);
    assert_eq!(look.version, "1.1.0");
}
