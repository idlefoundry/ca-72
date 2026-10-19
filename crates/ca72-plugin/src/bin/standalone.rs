//! The shipped CA-72 desktop application. The development example is kept apart.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

fn main() {
    ca72_plugin::identify_standalone();
    let mut device_access = true;
    let mut state_dir = None;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--no-devices" => device_access = false,
            "--state-dir" => match arguments.next().filter(|path| !path.starts_with("--")) {
                Some(path) => state_dir = Some(std::path::PathBuf::from(path)),
                None => {
                    eprintln!("--state-dir requires a directory. Use --help.");
                    std::process::exit(2);
                }
            },
            "--help" | "-h" => {
                println!(
                    "CA-72 standalone\n\nAudio and MIDI settings are in the editor's settings menu.\n--no-devices     Open the editor without accessing audio or MIDI hardware\n--state-dir DIR  Keep this session in a separate directory"
                );
                return;
            }
            _ => {
                eprintln!("Unknown argument: {argument}. Use --help.");
                std::process::exit(2);
            }
        }
    }
    let options = plugin_kit_standalone::AppOptions {
        // This identifies saved sessions and MIDI clients, not a Wayland surface.
        // Linux's X11 windows take their class from identify_standalone, Ca72::NAME.
        app_id: "com.idlefoundry.ca-72".into(),
        state_dir: state_dir.unwrap_or_else(|| {
            ca72_plugin::library::shared_dir()
                .join("CA-72")
                .join("Standalone")
        }),
        device_access,
    };
    if let Err(error) = nih_plug::wrapper::production::run_with_midi_reset_scope::<ca72_plugin::Ca72>(
        options,
        &["pitch_wheel", "mod_wheel"],
        plugin_kit_standalone::MidiResetScope::Global,
    ) {
        eprintln!("CA-72: {error}");
        ca72_plugin::standalone_error::show(&error);
        std::process::exit(1);
    }
}
