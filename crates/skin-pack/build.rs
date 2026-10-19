// A public release's key gate: on when the release sets `CA72_PUBLIC_RELEASE=1` (or the Mac's
// `CA72_REQUIRE_NOTARIZATION=1`), not by Cargo's profile, so that CI's release and bundle
// builds of every change keep building while the slot is empty (decisions.md R-SKIN-KEYS).
fn main() {
    println!("cargo:rustc-check-cfg=cfg(skin_pack_release)");
    println!("cargo:rerun-if-env-changed=CA72_PUBLIC_RELEASE");
    println!("cargo:rerun-if-env-changed=CA72_REQUIRE_NOTARIZATION");
    let on = |name: &str| std::env::var(name).as_deref() == Ok("1");
    if on("CA72_PUBLIC_RELEASE") || on("CA72_REQUIRE_NOTARIZATION") {
        println!("cargo:rustc-cfg=skin_pack_release");
    }
}
