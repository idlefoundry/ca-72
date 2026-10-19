//! The standalone's initial title becomes its X11 class, also under XWayland.
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
