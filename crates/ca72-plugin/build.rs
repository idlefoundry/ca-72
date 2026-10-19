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
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let tag = format!("v{version}");
    let ci_tag = std::env::var("GITHUB_REF").is_ok_and(|r| r == format!("refs/tags/{tag}"));
    let at_tag =
        ci_tag || git(&["tag", "--points-at", "HEAD"]).is_some_and(|t| t.lines().any(|l| l == tag));
    let build = if at_tag {
        String::new()
    } else {
        git(&["rev-parse", "--short=7", "HEAD"]).unwrap_or_default()
    };
    println!("cargo:rustc-env=CA72_BUILD={build}");

    // Again when the checkout moves to another commit or the tags change.
    println!("cargo:rerun-if-env-changed=GITHUB_REF");
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
