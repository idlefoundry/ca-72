//! The official packs' files, as a folder of each holds them: CLASSIC's manifest, layout and
//! licence; REALISTIC's the same with its pictures (the built-in ones). Shared by the tests that
//! check them as packs and by `examples/export_pack.rs`, which writes them out.

/// The files of the official pack `key` (`classic` or `realistic`): each its path and bytes.
pub fn files(key: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let manifest: &str = match key {
        "classic" => include_str!("classic.toml"),
        "realistic" => include_str!("realistic.toml"),
        _ => return None,
    };
    let mut out = vec![
        ("skin.toml".to_owned(), manifest.as_bytes().to_vec()),
        (
            "layout.toml".to_owned(),
            ca72_panel::arrangement::OFFICIAL.as_bytes().to_vec(),
        ),
        (
            "LICENSE.txt".to_owned(),
            include_bytes!("../../../../LICENSE").to_vec(),
        ),
    ];
    if key == "realistic" {
        for (slot, bytes) in ca72_panel::skin::SLOTS {
            out.push((format!("pictures/{slot}.png"), bytes.to_vec()));
        }
    }
    Some(out)
}
