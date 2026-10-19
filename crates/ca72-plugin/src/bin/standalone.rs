//! The shipped CA-72 desktop application. The development example is kept apart.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

fn main() {
    // A safe start asked for (decisions.md R-FX-SAFETY): the keys read first, as near the
    // opening as can be.
    let mut safe_start = ca72_plugin::fx::safe_start_keys_held();
    ca72_plugin::identify_standalone();
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
            "--safe-start" => safe_start = true,
            "--fx-mode" => match arguments.next().map(|m| m.parse()) {
                Some(Ok(m)) => fx_mode = m,
                _ => {
                    eprintln!("--fx-mode requires sync or pipelined. Use --help.");
                    std::process::exit(2);
                }
            },
            "--help" | "-h" => {
                println!(
                    "CA-72 standalone\n\nAudio and MIDI settings are in the editor's settings menu.\n--no-devices     Open the editor without accessing audio or MIDI hardware\n--state-dir DIR  Keep this session in a separate directory\n--agents         Allow AI agents (MCP, through ca72-mcp) for this session\n--safe-start     Start with the effects off, as holding Shift (or Option on macOS) as it opens does\n\nEffects (the FX key's drawer; these for testing):\n--fx-console     Drive the effects chain with commands on standard input (help lists them)\n--fx-mode MODE   sync (no added latency, the default) or pipelined (one block late)\nCA72_FX_PATH     Folders searched for effects before the standard CLAP and VST3 folders\nCA72_FX_STANDARD_FOLDERS=0  Search only CA72_FX_PATH"
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
    let effects = effects(&state_dir, fx_mode, fx_console, safe_start);
    let options = plugin_kit_standalone::AppOptions {
        // This identifies saved sessions and MIDI clients, not a Wayland surface.
        // Linux's X11 windows take their class from identify_standalone, Ca72::NAME.
        app_id: "com.idlefoundry.ca-72".into(),
        state_dir,
        device_access,
    };
    let result = nih_plug::wrapper::production::run_with_midi_reset_scope::<ca72_plugin::Ca72>(
        options,
        &["pitch_wheel", "mod_wheel"],
        plugin_kit_standalone::MidiResetScope::Global,
    );
    // The audio has stopped: the effects' states saved with the session, their helpers ended;
    // and the run closed properly (its marker taken away: the next start is not a safe one).
    if let Some((chain, launch)) = effects {
        if let Err(e) = chain.shutdown() {
            eprintln!("CA-72 effects: {e}");
        }
        launch.end();
    }
    if let Err(error) = result {
        eprintln!("CA-72: {error}");
        ca72_plugin::standalone_error::show(&error);
        std::process::exit(1);
    }
}

/// The effects chain (docs/fx-host.md), restored from the session's `effects.json`, its
/// audio side handed to the synth, and this run's marker; none without the helper beside the
/// application, or while another instance uses the session folder.
///
/// Nothing here waits for an effect (decisions.md R-FX-SAFETY): the chain is put in place and
/// each effect loads on a thread of its own while the window opens and the synth plays, dry
/// until it joins. A start after a run that did not close, with effects to load, or asked for
/// (`safe_start`: Shift held, `--safe-start`), loads none of them until the player turns them
/// back on.
fn effects(
    state_dir: &std::path::Path,
    mode: ca72_fx::Mode,
    console: bool,
    safe_start: bool,
) -> Option<(ca72_fx::Chain, ca72_fx::launch::Launch)> {
    let helper = match ca72_fx::locate::helper_path() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("CA-72 effects: none this session: {e}");
            return None;
        }
    };
    let (launch, last) = ca72_fx::launch::Launch::begin(state_dir);
    if last == ca72_fx::launch::Last::Running {
        eprintln!("CA-72 effects: none: another CA-72 is using this session folder");
        return None;
    }
    let file = state_dir.join("effects.json");
    let has_effects = ca72_fx::SavedChain::load(&file)
        .ok()
        .flatten()
        .is_some_and(|c| c.has_effects());
    let safe = ca72_fx::launch::safe_start(last, has_effects, safe_start);
    let mut opts = ca72_fx::ChainOptions::new(helper.clone());
    opts.mode = mode;
    opts.file = Some(file);
    let (chain, runner) = ca72_fx::Chain::new(opts);
    let how = match safe {
        Some(why) => {
            eprintln!(
                "CA-72 effects: a safe start ({}): the effects are off until they are turned back on in the FX drawer",
                match why {
                    ca72_fx::launch::SafeStart::DidNotClose => "the last run did not close",
                    ca72_fx::launch::SafeStart::Asked => "asked for",
                }
            );
            ca72_fx::Restore::Off
        }
        None => ca72_fx::Restore::Background,
    };
    let t = std::time::Instant::now();
    for e in chain.restore_file_as(how) {
        eprintln!("CA-72 effects: {e}");
    }
    if !chain.is_empty() {
        eprintln!(
            "CA-72 effects: {} put back in {:.1} ms{}",
            chain.len(),
            t.elapsed().as_secs_f64() * 1000.0,
            if how == ca72_fx::Restore::Off {
                ""
            } else {
                "; loading"
            }
        );
    }
    ca72_plugin::fx::install(runner);
    // The editor's effects' drawer (decisions.md R-FX-DRAWER).
    ca72_plugin::fx::install_host(
        chain.clone(),
        helper.clone(),
        ca72_fx::locate::search_dirs(),
        Some(state_dir.join("effects-cache.json")),
        safe,
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
    Some((chain, launch))
}
