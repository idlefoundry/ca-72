//! The standalone application's effects' helper (docs/fx-host.md): DAW-v3's plug-in helper
//! (third_party/daw-v3), one process per effect, started and watched by `ca72-fx`. Plug-in
//! code runs only here: a crash or a hang ends this process, never the synth.
//!
//! `ca72-fx-helper --scan <file>` prints the plug-ins in a file as JSON and exits.
//!
//! On macOS it is an accessory application from its first instruction (no Dock icon, no menu
//! bar, its windows still shown in front), before a plug-in it loads can make it an ordinary
//! one (decisions.md R-FX-DOCK). The application ships it in a bundle of its own whose
//! Info.plist says so too (`LSUIElement`), so that LaunchServices never takes it for the synth.

#[cfg(target_os = "macos")]
mod macos;

fn main() {
    #[cfg(target_os = "macos")]
    macos::accessory();
    std::process::exit(daw_helper::run());
}
