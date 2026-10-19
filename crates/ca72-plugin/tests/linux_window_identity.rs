//! The standalone's X11 class, also under XWayland, matches its installed launcher.
#![cfg(target_os = "linux")]

use ca72_plugin::Ca72;
use nih_plug::prelude::Plugin;

#[test]
fn the_standalone_name_matches_the_launchers_window_class() {
    let desktop = include_str!("../../../scripts/standalone/CA-72.desktop");
    let wm_class = desktop
        .lines()
        .find_map(|line| line.strip_prefix("StartupWMClass="))
        .expect("the launcher needs an X11 window class");
    // Both nih-plug standalone wrappers pass P::NAME as the top-level title.
    // Baseview's protocol tests check that title reaches WM_CLASS before mapping.
    assert_eq!(Ca72::NAME, wm_class);
}

/// The class is the application's, not a title: the fatal-error window, titled
/// "CA-72 — error", still matches the launcher (standalone_error.rs).
#[cfg(feature = "standalone-production")]
#[test]
fn every_standalone_window_has_the_launchers_class_whatever_its_title() {
    let desktop = include_str!("../../../scripts/standalone/CA-72.desktop");
    let wm_class = desktop
        .lines()
        .find_map(|line| line.strip_prefix("StartupWMClass="))
        .expect("the launcher needs an X11 window class");
    ca72_plugin::identify_standalone();
    assert_eq!(baseview::application_class(), Some(wm_class));
    let main = include_str!("../src/bin/standalone.rs");
    let body = main
        .split("fn main() {")
        .nth(1)
        .expect("the standalone's main");
    assert!(
        body.trim_start()
            .starts_with("ca72_plugin::identify_standalone();"),
        "the standalone names itself before anything can open a window"
    );
}
