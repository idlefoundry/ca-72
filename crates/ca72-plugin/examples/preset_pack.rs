//! The presets retired from the plug-in (`sounds/retired.toml`, decisions.md R55) written
//! into a folder as the user's presets, the pack `scripts/package.sh` zips beside a release's
//! installers: `cargo run -p ca72-plugin --example preset_pack -- <folder>`. Prints each file;
//! writes none when no preset is retired.

use ca72_plugin::library::{parse_factory, write_pack};

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("a folder to write the pack into");
    let retired = parse_factory(include_str!("../sounds/retired.toml"))
        .unwrap_or_else(|e| panic!("sounds/retired.toml: {e}"));
    if retired.is_empty() {
        return;
    }
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{dir}: {e}"));
    for f in write_pack(dir.as_ref(), &retired).unwrap_or_else(|e| panic!("{e}")) {
        println!("{}", f.display());
    }
}
