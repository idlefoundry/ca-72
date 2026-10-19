// A release's key gate follows Cargo's profile, even if debug assertions are enabled in it.
// Cargo reports `release` for the bundle profile too (decisions.md R-SKIN-KEYS).
fn main() {
    println!("cargo:rustc-check-cfg=cfg(skin_pack_release)");
    println!("cargo:rerun-if-env-changed=PROFILE");
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        println!("cargo:rustc-cfg=skin_pack_release");
    }
}
