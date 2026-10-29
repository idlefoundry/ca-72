//! This build's identity, for the editor to show beside the version (`update::shown`): empty
//! for a build of the release's tag (`v<version>`, as CI builds a release and as a release is
//! made from a clean checkout of it: docs/macos-release.md), otherwise the commit it was built
//! from, so a local build cannot pass for the release (the owner, 2026-10-17, after a local
//! build that also read 0.2.4: "why don't we have versions shown in the plugin settings menu so
//! there's no confusion?"). A source tree without git builds as the version alone.

use std::process::Command;

/// One line of git's answer, if git is there and answers.
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!s.is_empty()).then_some(s)
}

fn main() {
    engine_fingerprint();
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let tag = format!("v{version}");
    let ci_tag = std::env::var("GITHUB_REF").is_ok_and(|r| r == format!("refs/tags/{tag}"));
    let at_tag =
        ci_tag || git(&["tag", "--points-at", "HEAD"]).is_some_and(|t| t.lines().any(|l| l == tag));
    // A beta names itself in place of its commit (CA72_BUILD_LABEL=beta.1: `0.3.0 · BETA.1`),
    // keeping the version a release's, so the update check and What's New treat it as 0.3.0.
    let label = std::env::var("CA72_BUILD_LABEL").unwrap_or_default();
    let build = if !label.is_empty() {
        label
    } else if at_tag {
        String::new()
    } else {
        git(&["rev-parse", "--short=7", "HEAD"]).unwrap_or_default()
    };
    println!("cargo:rustc-env=CA72_BUILD={build}");

    // Again when the checkout moves to another commit or the tags change.
    println!("cargo:rerun-if-env-changed=GITHUB_REF");
    println!("cargo:rerun-if-env-changed=CA72_BUILD_LABEL");
    for path in ["HEAD", "packed-refs", "refs/tags"] {
        if let Some(p) = git(&["rev-parse", "--git-path", path]) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    if let Some(r) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(p) = git(&["rev-parse", "--git-path", &r])
    {
        println!("cargo:rerun-if-changed={p}");
    }
}

// A render records the source it used, including work not committed yet. This is a source
// fingerprint, not a promise of bit-identical floating point between different CPUs.
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn sources(root: &Path, paths: &mut Vec<PathBuf>) {
    let mut children: Vec<_> = std::fs::read_dir(root)
        .expect("source directory")
        .map(|p| p.expect("source file").path())
        .collect();
    children.sort();
    for p in children {
        if p.is_dir() {
            sources(&p, paths);
        } else if p.extension().is_some_and(|e| e == "rs") {
            paths.push(p);
        }
    }
}
fn engine_fingerprint() {
    let base = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest"));
    let root = base.parent().expect("crates").parent().expect("workspace");
    let mut paths = Vec::new();
    for part in [
        "crates/ca72/src",
        "crates/ca72-plugin/src",
        "third_party/nih-plug/src",
    ] {
        sources(&root.join(part), &mut paths);
    }
    for part in [
        "Cargo.lock",
        "Cargo.toml",
        "crates/ca72/Cargo.toml",
        "crates/ca72-plugin/Cargo.toml",
    ] {
        paths.push(root.join(part));
    }
    paths.sort();
    let mut hash = Sha256::new();
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        hash.update(
            path.strip_prefix(root)
                .expect("relative")
                .to_string_lossy()
                .replace('\\', "/"),
        );
        hash.update([0]);
        hash.update(std::fs::read(path).expect("source bytes"));
        hash.update([0]);
    }
    println!(
        "cargo:rustc-env=CA72_ENGINE_SOURCE_SHA256={:x}",
        hash.finalize()
    );
}
