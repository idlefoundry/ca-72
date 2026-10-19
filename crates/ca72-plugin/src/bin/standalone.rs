//! The shipped CA-72 desktop application. The development example is kept apart.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

fn main() {
    let mut device_access = true;
    let mut state_dir = None;
    let mut fx_console = false;
    let mut fx_mode = ca72_fx::Mode::Sync;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--no-devices" => device_access = false,
            // AI agents allowed for this session, whatever the settings say (docs/mcp.md).
            "--agents" => ca72_plugin::agent::force(true),
            "--state-dir" => match arguments.next().filter(|path| !path.starts_with("--")) {
                Some(path) => state_dir = Some(std::path::PathBuf::from(path)),
                None => {
                    eprintln!("--state-dir requires a directory. Use --help.");
                    std::process::exit(2);
                }
            },
            "--fx-console" => fx_console = true,
            "--fx-mode" => match arguments.next().map(|m| m.parse()) {
                Some(Ok(m)) => fx_mode = m,
                _ => {
                    eprintln!("--fx-mode requires sync or pipelined. Use --help.");
                    std::process::exit(2);
                }
            },
            "--help" | "-h" => {
                println!(
                    "CA-72 standalone\n\nAudio and MIDI settings are in the editor's settings menu.\n--no-devices     Open the editor without accessing audio or MIDI hardware\n--state-dir DIR  Keep this session in a separate directory\n--agents         Allow AI agents (MCP, through ca72-mcp) for this session\n\nEffects (the FX key's drawer; these for testing):\n--fx-console     Drive the effects chain with commands on standard input (help lists them)\n--fx-mode MODE   sync (no added latency, the default) or pipelined (one block late)\nCA72_FX_PATH     Folders searched for effects before the standard CLAP and VST3 folders\nCA72_FX_STANDARD_FOLDERS=0  Search only CA72_FX_PATH"
                );
                return;
            }
            _ => {
                eprintln!("Unknown argument: {argument}. Use --help.");
                std::process::exit(2);
            }
        }
    }
    let state_dir = state_dir.unwrap_or_else(|| {
        ca72_plugin::library::shared_dir()
            .join("CA-72")
            .join("Standalone")
    });
    let effects = effects(&state_dir, fx_mode, fx_console);
    let options = plugin_kit_standalone::AppOptions {
        // This identifies saved sessions and MIDI clients, not a Wayland surface.
        // Linux's X11 window uses Ca72::NAME and the launcher's StartupWMClass.
        app_id: "com.idlefoundry.ca-72".into(),
        state_dir,
        device_access,
    };
    let result = nih_plug::wrapper::production::run_with_midi_reset_scope::<ca72_plugin::Ca72>(
        options,
        &["pitch_wheel", "mod_wheel"],
        plugin_kit_standalone::MidiResetScope::Global,
    );
    // The audio has stopped: the effects' states saved with the session, their helpers ended.
    if let Some(chain) = effects
        && let Err(e) = chain.shutdown()
    {
        eprintln!("CA-72 effects: {e}");
    }
    if let Err(error) = result {
        eprintln!("CA-72: {error}");
        ca72_plugin::standalone_error::show(&error);
        std::process::exit(1);
    }
}

/// The effects chain (docs/fx-host.md), restored from the session's `effects.json`, its
/// audio side handed to the synth; none without the helper beside the application.
fn effects(
    state_dir: &std::path::Path,
    mode: ca72_fx::Mode,
    console: bool,
) -> Option<ca72_fx::Chain> {
    let helper = match ca72_fx::locate::helper_path() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("CA-72 effects: none this session: {e}");
            return None;
        }
    };
    let mut opts = ca72_fx::ChainOptions::new(helper.clone());
    opts.mode = mode;
    opts.file = Some(state_dir.join("effects.json"));
    let (chain, runner) = ca72_fx::Chain::new(opts);
    for e in chain.restore_file() {
        eprintln!("CA-72 effects: {e}");
    }
    ca72_plugin::fx::install(runner);
    // The editor's effects' drawer (decisions.md R-FX-DRAWER).
    ca72_plugin::fx::install_host(
        chain.clone(),
        helper.clone(),
        ca72_fx::locate::search_dirs(),
        Some(state_dir.join("effects-cache.json")),
    );
    if console {
        let mut console = ca72_fx::console::Console::new(
            chain.clone(),
            helper,
            ca72_fx::locate::search_dirs(),
            Some(state_dir.join("effects-cache.json")),
        );
        let _ = std::thread::Builder::new()
            .name("CA-72 effects console".into())
            .spawn(move || {
                use std::io::BufRead;
                println!("{}", console.show());
                for line in std::io::stdin().lock().lines() {
                    let Ok(line) = line else { break };
                    println!("{}", console.run(&line));
                }
            });
    }
    Some(chain)
}
