//! On macOS the helper is never a second application in the Dock (decisions.md R-FX-DOCK):
//! LaunchServices lists it as an agent ("UIElement": no Dock icon or menu bar), whether it runs
//! bare (the tests, a development build) or from its own bundle in an application's
//! `Contents/Helpers`, where it carries that bundle's identifier, not the application's.
#![cfg(target_os = "macos")]
#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const HELPER: &str = env!("CARGO_BIN_EXE_ca72-fx-helper");

/// A helper serving (its standard input kept open), ended when dropped.
struct Running(Child);

impl Running {
    fn start(exe: &Path) -> Running {
        Running(
            Command::new(exe)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        )
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// LaunchServices' entry for `pid` (`lsappinfo list`): its type and bundle identifier, once it
/// has one, waiting up to 10 s.
fn registered(pid: u32) -> Option<(String, String)> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        let out = Command::new("lsappinfo").arg("list").output().unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        let mut bundle = String::new();
        for line in text.lines() {
            if let Some(b) = line.trim().strip_prefix("bundleID=") {
                bundle = b.trim().trim_matches('"').to_owned();
            }
            if line.contains(&format!("pid = {pid} "))
                && let Some(t) = line.split("type=\"").nth(1)
            {
                return Some((t.split('"').next().unwrap_or("").to_owned(), bundle));
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

#[test]
fn the_helper_run_bare_is_an_agent_from_its_start() {
    let helper = Running::start(Path::new(HELPER));
    let (kind, _) = registered(helper.0.id()).expect("the helper never checked in");
    assert_eq!(kind, "UIElement");
}

fn plist(path: &Path, entries: &[(&str, &str)]) {
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict>\n",
    );
    for (k, v) in entries {
        if *v == "true" {
            s += &format!("<key>{k}</key><true/>\n");
        } else {
            s += &format!("<key>{k}</key><string>{v}</string>\n");
        }
    }
    s += "</dict></plist>\n";
    std::fs::write(path, s).unwrap();
}

/// The application's layout as `scripts/standalone.sh` makes it: the helper found there, and
/// listed with its own bundle's identifier, an agent.
#[test]
fn the_helper_in_its_own_bundle_is_an_agent_of_its_own() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("CA-72.app/Contents");
    std::fs::create_dir_all(app.join("MacOS")).unwrap();
    plist(
        &app.join("Info.plist"),
        &[
            (
                "CFBundleIdentifier",
                "com.idlefoundry.ca72-dock-test.standalone",
            ),
            ("CFBundleExecutable", "ca72-standalone"),
            ("CFBundlePackageType", "APPL"),
        ],
    );
    std::fs::write(app.join("MacOS/ca72-standalone"), b"").unwrap();
    let host = app.join("Helpers/CA-72 Effects Host.app/Contents");
    std::fs::create_dir_all(host.join("MacOS")).unwrap();
    let id = "com.idlefoundry.ca72-dock-test.standalone.effects-host";
    plist(
        &host.join("Info.plist"),
        &[
            ("CFBundleIdentifier", id),
            ("CFBundleExecutable", "ca72-fx-helper"),
            ("CFBundlePackageType", "APPL"),
            ("LSUIElement", "true"),
        ],
    );
    std::fs::copy(HELPER, host.join("MacOS/ca72-fx-helper")).unwrap();

    let found = ca72_fx::locate::helper_for(&app.join("MacOS/ca72-standalone"))
        .expect("the helper's bundle not found");
    assert_eq!(found, host.join("MacOS/ca72-fx-helper"));
    let helper = Running::start(&found);
    let (kind, bundle) = registered(helper.0.id()).expect("the helper never checked in");
    assert_eq!((kind.as_str(), bundle.as_str()), ("UIElement", id));
}
