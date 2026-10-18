//! Makes a marketplace signing key (decisions.md R-SKIN-PACKS): `cargo run -p skin-pack
//! --example pack_keygen -- <secret-key-file>`. Writes the secret, in hexadecimal, to a new file
//! readable by its owner alone, and prints the public half for the plug-ins' `TRUSTED_KEYS`.
//!
//! The secret is the marketplace's alone: make it on the machine that signs, keep it there and in
//! an offline backup, and never put it in a repository. Its randomness is the system's
//! (`/dev/urandom`: macOS and Linux).

use std::io::{Read, Write};

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("pack_keygen <secret-key-file>");
        std::process::exit(2);
    };
    let mut secret = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut secret))
        .expect("the system's randomness (/dev/urandom)");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options
        .open(&path)
        .unwrap_or_else(|e| panic!("{path} (it must not exist yet): {e}"));
    let hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
    file.write_all(hex.as_bytes()).expect("written");
    let key = skin_pack::signature::signing_key(&hex).expect("a key");
    println!("secret written to {path} (keep it there, and offline; never in a repository)");
    println!("public key: {}", skin_pack::signature::public_hex(&key));
}
