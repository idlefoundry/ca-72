//! Signs a pack as the marketplace does once it has reviewed it (decisions.md R-SKIN-PACKS):
//! `cargo run -p skin-pack --example pack_sign -- <pack-folder> <secret-key-file> <key-id>
//! <marketplace|first-party> <YYYY-MM-DD>`. Writes `signature.toml` into the folder (replacing
//! one there). A pack its checks reject is not signed. Run the plug-in's own checks first
//! (`ca72-panel`'s `check_pack`): these are the format's alone.

use std::path::Path;

use skin_pack::signature::{self, Publisher};
use skin_pack::{Context, Files, Limits, Pack, Report};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir, key_file, key_id, publisher, date] = args.as_slice() else {
        eprintln!(
            "pack_sign <pack-folder> <secret-key-file> <key-id> <marketplace|first-party> <YYYY-MM-DD>"
        );
        std::process::exit(2);
    };
    let publisher = match publisher.as_str() {
        "marketplace" => Publisher::Marketplace,
        "first-party" => Publisher::FirstParty,
        _ => {
            eprintln!("the publisher is `marketplace` or `first-party`");
            std::process::exit(2);
        }
    };
    let secret = std::fs::read_to_string(key_file).expect("the secret key file");
    let key = signature::signing_key(&secret).expect("a secret key, 64 hexadecimal digits");
    let dir = Path::new(dir);
    let limits = Limits::default();
    let mut report = Report::default();
    let files = Files::read_dir(dir, &limits, &mut report);
    // (Its old signature aside: what is signed is the rest.)
    let rest: Vec<(&str, &[u8])> = files
        .iter()
        .filter(|(p, _)| *p != signature::SIGNATURE)
        .collect();
    let files = Files::from_memory(rest, &limits, &mut report);
    let plugin = skin_pack::manifest::read(
        files.get(skin_pack::MANIFEST).unwrap_or_default(),
        &mut report,
    )
    .map(|m| m.plugin)
    .unwrap_or_default();
    let context = Context {
        plugin: &plugin,
        official: &[],
        official_licence: "",
        built_in: false,
        test_drawings: true,
        keys: &[],
        plugin_version: None,
    };
    let pack = match Pack::check(files.clone(), report, &limits, &context) {
        Ok(p) => p,
        // (A first-party id is refused unsigned: signing it is what allows it.)
        Err(r) if publisher == Publisher::FirstParty && only_the_id(&r) => {
            let mut report = Report::default();
            let m = skin_pack::manifest::read(
                files.get(skin_pack::MANIFEST).unwrap_or_default(),
                &mut report,
            )
            .expect("its manifest");
            write(
                dir,
                &files,
                (&m.id, &m.version),
                key_id,
                &key,
                publisher,
                date,
            );
            return;
        }
        Err(r) => {
            print!("{r}");
            eprintln!("not signed: the pack is rejected");
            std::process::exit(1);
        }
    };
    let m = &pack.manifest;
    write(
        dir,
        &files,
        (&m.id, &m.version),
        key_id,
        &key,
        publisher,
        date,
    );
}

/// Whether the only rejection is the makers' id prefix, which a first-party signature allows.
fn only_the_id(r: &Report) -> bool {
    r.findings
        .iter()
        .filter(|f| f.severity == skin_pack::Severity::Reject)
        .all(|f| f.code == "manifest.id" && f.message.contains("kept for the plug-in's makers"))
}

fn write(
    dir: &Path,
    files: &Files,
    pack: (&str, &str),
    key_id: &str,
    key: &ed25519_dalek::SigningKey,
    publisher: Publisher,
    date: &str,
) {
    let text = signature::sign(files, pack, (key_id, key), publisher, date);
    std::fs::write(dir.join(signature::SIGNATURE), text).expect("the signature written");
    println!("{} {} signed with `{key_id}`", pack.0, pack.1);
}
