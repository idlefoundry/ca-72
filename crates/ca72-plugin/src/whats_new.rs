//! Offline release notes and their version ranges (decisions.md R-WHATS-NEW).
//! Only the visible editor claims an automatic showing; this module never starts a check
//! for updates or touches the sound. The temporary presentation lives in ca72-panel.

use std::sync::OnceLock;

use semver::Version;
use serde::Deserialize;

use crate::update::VERSION;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    release: Vec<Release>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Release {
    version: String,
    features: Vec<String>,
    performance_improvements: Vec<String>,
    bug_fixes: Vec<String>,
}

fn catalog() -> &'static Catalog {
    static NOTES: OnceLock<Catalog> = OnceLock::new();
    NOTES.get_or_init(|| {
        toml::from_str(include_str!("../../../docs/releases/notes.toml"))
            .expect("the bundled release notes parse (checked by the release tests)")
    })
}

/// Notes ready for a presentation, grouped in the order requested in issue #75.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notes {
    pub installed: String,
    /// None for highlights: no upgrade history is claimed.
    pub previous: Option<String>,
    /// Exactly the releases included; older Markdown notes may be unavailable.
    pub versions: Vec<String>,
    pub sections: [(String, Vec<String>); 3],
}

impl Notes {
    pub fn coverage(&self) -> String {
        match &self.previous {
            Some(previous) => format!(
                "Since {previous}. Included releases: {}.",
                self.versions.join(", ")
            ),
            None => format!("Highlights from {}.", self.installed),
        }
    }

    pub fn view(self) -> ca72_panel::whats_new::View {
        let coverage = self.coverage();
        ca72_panel::whats_new::View::new(self.installed, coverage, self.sections)
    }
}

/// The current release's highlights, also used by SETTINGS > WHAT'S NEW.
pub fn current() -> Notes {
    assemble(None, VERSION).expect("this build has bundled release notes")
}

/// An automatic showing, or none for the same version or a downgrade. Build metadata does
/// not change precedence; an unsafe previous value falls back to current highlights.
/// Only notes actually present in the bundle are included, never invented history.
pub fn assemble(previous: Option<&str>, installed: &str) -> Option<Notes> {
    let installed_version = Version::parse(installed).ok()?;
    let previous_version = previous.and_then(|p| Version::parse(p).ok());
    if previous_version
        .as_ref()
        .is_some_and(|p| p.cmp_precedence(&installed_version).is_ge())
    {
        return None;
    }
    let mut releases: Vec<_> = catalog()
        .release
        .iter()
        .filter_map(|r| Some((Version::parse(&r.version).ok()?, r)))
        .filter(|(v, _)| match &previous_version {
            Some(p) => v.cmp_precedence(p).is_gt() && v.cmp_precedence(&installed_version).is_le(),
            None => v.cmp_precedence(&installed_version).is_eq(),
        })
        .collect();
    releases.sort_by(|(a, _), (b, _)| a.cmp_precedence(b));
    if !releases
        .iter()
        .any(|(v, _)| v.cmp_precedence(&installed_version).is_eq())
    {
        return None;
    }
    let mut notes = Notes {
        installed: installed.to_owned(),
        previous: previous_version.and(previous.map(str::to_owned)),
        versions: Vec::new(),
        sections: [
            ("Features".into(), Vec::new()),
            ("Performance improvements".into(), Vec::new()),
            ("Bug fixes".into(), Vec::new()),
        ],
    };
    let combined = releases.len() > 1;
    for (_, release) in releases {
        notes.versions.push(release.version.clone());
        for (section, entries) in notes.sections.iter_mut().zip([
            &release.features,
            &release.performance_improvements,
            &release.bug_fixes,
        ]) {
            section.1.extend(entries.iter().map(|entry| {
                if combined {
                    format!("{}: {entry}", release.version)
                } else {
                    entry.clone()
                }
            }));
        }
    }
    Some(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_notes_match_the_build_and_the_maintained_sources() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let workspace: toml::Value = std::fs::read_to_string(root.join("Cargo.toml"))
            .expect("workspace manifest")
            .parse()
            .expect("workspace TOML");
        assert_eq!(
            workspace["workspace"]["package"]["version"].as_str(),
            Some(VERSION)
        );
        let mut versions = Vec::new();
        for release in &catalog().release {
            let version = Version::parse(&release.version).expect("safe release version");
            assert!(
                versions.last().is_none_or(|last| last < &version),
                "ordered, unique releases"
            );
            assert!(version >= Version::new(0, 2, 0));
            assert!(version <= Version::parse(VERSION).expect("build version"));
            versions.push(version);
            let source = root.join(format!("docs/releases/{}.md", release.version));
            let source = std::fs::read_to_string(source).expect("maintained Markdown source");
            assert_eq!(
                source.lines().next(),
                Some(format!("# CA-72 {}", release.version).as_str())
            );
            let entries: Vec<_> = [
                &release.features,
                &release.performance_improvements,
                &release.bug_fixes,
            ]
            .into_iter()
            .flatten()
            .collect();
            assert!(!entries.is_empty());
            assert!(
                entries
                    .iter()
                    .all(|e| !e.trim().is_empty() && e.ends_with('.'))
            );
        }
        for entry in std::fs::read_dir(root.join("docs/releases")).expect("release sources") {
            let path = entry.expect("release file").path();
            if path.extension().is_some_and(|ext| ext == "md")
                && let Some(version) = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| Version::parse(s).ok())
                && version >= Version::new(0, 2, 0)
            {
                assert!(
                    versions.contains(&version),
                    "missing bundled highlights for {version}"
                );
            }
        }
        assert_eq!(current().installed, VERSION);
        assert_eq!(current().versions, [VERSION]);
    }

    #[test]
    fn clean_install_and_unsafe_history_use_only_current_highlights() {
        for previous in [
            None,
            Some(""),
            Some("latest"),
            Some("0.2"),
            Some("0.02.3"),
            Some("v0.2.3"),
        ] {
            let notes = assemble(previous, "0.2.7").expect("highlights");
            assert!(notes.previous.is_none());
            assert_eq!(notes.versions, ["0.2.7"]);
            assert_eq!(notes.coverage(), "Highlights from 0.2.7.");
        }
    }

    #[test]
    fn ordinary_and_skipped_updates_group_all_available_releases() {
        let ordinary = assemble(Some("0.2.6"), "0.2.7").expect("update");
        assert_eq!(ordinary.previous.as_deref(), Some("0.2.6"));
        assert_eq!(ordinary.versions, ["0.2.7"]);
        let skipped = assemble(Some("0.2.3"), "0.2.7").expect("skipped update");
        assert_eq!(skipped.versions, ["0.2.4", "0.2.5", "0.2.6", "0.2.7"]);
        assert_eq!(
            skipped.sections.clone().map(|s| s.0),
            ["Features", "Performance improvements", "Bug fixes"]
        );
        for (i, section) in skipped.sections.iter().enumerate() {
            let expected: Vec<_> = (4..=7)
                .flat_map(|v| {
                    assemble(None, &format!("0.2.{v}"))
                        .expect("release")
                        .sections[i]
                        .1
                        .iter()
                        .map(|entry| format!("0.2.{v}: {entry}"))
                        .collect::<Vec<_>>()
                })
                .collect();
            assert_eq!(section.1, expected);
        }
        let old = assemble(Some("0.2.0"), "0.2.7").expect("available history");
        assert_eq!(old.versions, ["0.2.3", "0.2.4", "0.2.5", "0.2.6", "0.2.7"]);
        assert!(old.coverage().contains("Included releases: 0.2.3, 0.2.4"));
    }

    #[test]
    fn downgrades_and_repeated_versions_do_not_show_automatically() {
        for previous in [
            "0.2.7",
            "0.2.7+other-build",
            "0.2.8",
            "1.0.0",
            "0.3.0-beta.1",
        ] {
            assert!(assemble(Some(previous), "0.2.7").is_none());
        }
        assert!(assemble(Some("0.2.7-rc.1"), "0.2.7+build").is_some());
        assert!(
            assemble(None, "0.2.8").is_none(),
            "never invent a missing release"
        );
    }
}
