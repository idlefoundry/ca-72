//! Writes an official look out as a skin pack's folder: `cargo run -p ca72-panel --example
//! export_pack -- <classic|realistic> <folder>`. The folder is a pack in the format every pack
//! uses (docs/skin-packs.md), under the GPL as the plug-in is: a pack made from it is too (start
//! a pack of your own from the CC0 starter, docs/skin-packs/starter). Its id is the makers' own
//! (`com.idlefoundry.*`, kept for the built-in packs): a copy installed must be given another.

#[path = "../packs/official/files.rs"]
mod files;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(key), Some(dir)) = (args.first(), args.get(1)) else {
        eprintln!("export_pack <classic|realistic> <folder>");
        std::process::exit(2);
    };
    let Some(files) = files::files(key) else {
        eprintln!("no official pack `{key}`");
        std::process::exit(2);
    };
    let dir = std::path::Path::new(dir);
    if dir.exists() {
        eprintln!("{} exists: give a new folder", dir.display());
        std::process::exit(1);
    }
    for (path, bytes) in files {
        let p = dir.join(&path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("the folder");
        }
        std::fs::write(&p, bytes).expect("written");
    }
    println!("{key} written to {}", dir.display());
}
