#!/usr/bin/env bash
# Build the shipped app into target/bundled. The legacy standalone example remains
# available separately for JACK and development. macOS builds both architectures.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
profile="${CA72_STANDALONE_PROFILE:-bundle}"
profile_dir="$profile"
if [ "$profile" = dev ]; then profile_dir=debug; fi
target_dir="${CARGO_TARGET_DIR:-$root/target}"
mkdir -p "$root/target/bundled"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
# The effects' helper (docs/fx-host.md) goes beside the application: plug-ins are loaded only
# in it. On macOS it is an application bundle of its own in Contents/Helpers, "CA-72 Effects
# Host.app" (LSUIElement: no Dock icon or menu bar; its own identifier), where the application
# looks for it; loose in Contents/Helpers it showed in the Dock as a second CA-72 (R-FX-DOCK).
build() {
    cargo build --locked -p ca72-plugin --bin ca72-standalone \
        --features standalone-production --profile "$profile" "$@"
    cargo build --locked -p ca72-plugin --bin ca72-mcp --profile "$profile" "$@"
    cargo build --locked -p ca72-fx-helper --bin ca72-fx-helper --profile "$profile" "$@"
}
case "$(uname -s)" in
Darwin)
    build --target aarch64-apple-darwin
    build --target x86_64-apple-darwin
    name="CA-72"
    id="com.idlefoundry.ca72.standalone"
    app="$root/target/bundled/$name.app"
    host="$app/Contents/Helpers/$name Effects Host.app"
    rm -rf "$app/Contents/Helpers"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" \
        "$host/Contents/MacOS" "$host/Contents/Resources"
    lipo -create "$target_dir/aarch64-apple-darwin/$profile_dir/ca72-standalone" \
        "$target_dir/x86_64-apple-darwin/$profile_dir/ca72-standalone" \
        -output "$app/Contents/MacOS/ca72-standalone"
    lipo -create "$target_dir/aarch64-apple-darwin/$profile_dir/ca72-mcp" \
        "$target_dir/x86_64-apple-darwin/$profile_dir/ca72-mcp" \
        -output "$app/Contents/MacOS/ca72-mcp"
    lipo -create "$target_dir/aarch64-apple-darwin/$profile_dir/ca72-fx-helper" \
        "$target_dir/x86_64-apple-darwin/$profile_dir/ca72-fx-helper" \
        -output "$host/Contents/MacOS/ca72-fx-helper"
    cp scripts/standalone/CA-72.icns "$app/Contents/Resources/"
    cp scripts/standalone/CA-72.icns "$host/Contents/Resources/"
    cp LICENSE THIRD-PARTY-NOTICES.txt "$app/Contents/Resources/"
    python3 - "$app/Contents/Info.plist" "$host/Contents/Info.plist" "$version" "$name" "$id" <<'PY'
import plistlib
import sys
app, host, version, name, ident = sys.argv[1:]
common = {
    "CFBundlePackageType": "APPL",
    "CFBundleShortVersionString": version, "CFBundleVersion": version,
    "CFBundleIconFile": "CA-72.icns", "LSMinimumSystemVersion": "11.0",
    "NSHighResolutionCapable": True,
    "NSHumanReadableCopyright": "Copyright © 2026 Idle Foundry Ltd. GNU GPL version 3 or later.",
}
with open(app, "wb") as stream:
    plistlib.dump(common | {
        "CFBundleIdentifier": ident,
        "CFBundleName": name, "CFBundleDisplayName": name,
        "CFBundleExecutable": "ca72-standalone",
        "NSMicrophoneUsageDescription": "CA-72 uses your selected audio input to play external sound through its synthesizer circuit.",
    }, stream)
# The effects' helper: an agent application (no Dock icon or menu bar) with its own identifier,
# so that macOS never takes it for the synth.
with open(host, "wb") as stream:
    plistlib.dump(common | {
        "CFBundleIdentifier": ident + ".effects-host",
        "CFBundleName": name + " Effects Host", "CFBundleDisplayName": name + " Effects Host",
        "CFBundleExecutable": "ca72-fx-helper",
        "LSUIElement": True,
    }, stream)
PY
    # Apple lipo versions differ in parsing architecture lists. Check each slice.
    for arch in arm64 x86_64; do
        lipo "$app/Contents/MacOS/ca72-standalone" -verify_arch "$arch"
        lipo "$app/Contents/MacOS/ca72-mcp" -verify_arch "$arch"
        lipo "$host/Contents/MacOS/ca72-fx-helper" -verify_arch "$arch"
    done
    printf '%s\n' "$app"
    ;;
Linux)
    build
    cp "$target_dir/$profile_dir/ca72-standalone" "$root/target/bundled/ca72-standalone"
    cp "$target_dir/$profile_dir/ca72-mcp" "$root/target/bundled/ca72-mcp"
    cp "$target_dir/$profile_dir/ca72-fx-helper" "$root/target/bundled/ca72-fx-helper"
    cp scripts/standalone/CA-72.desktop scripts/standalone/com.idlefoundry.ca72.png "$root/target/bundled/"
    printf '%s\n' "$root/target/bundled/ca72-standalone"
    ;;
MINGW*|MSYS*|CYGWIN*)
    build
    cp "$target_dir/$profile_dir/ca72-standalone.exe" "$root/target/bundled/CA-72.exe"
    cp "$target_dir/$profile_dir/ca72-mcp.exe" "$root/target/bundled/ca72-mcp.exe"
    cp "$target_dir/$profile_dir/ca72-fx-helper.exe" "$root/target/bundled/ca72-fx-helper.exe"
    cp scripts/standalone/CA-72.ico "$root/target/bundled/"
    printf '%s\n' "$root/target/bundled/CA-72.exe"
    ;;
*) printf 'standalone.sh: unsupported platform\n' >&2; exit 1 ;;
esac
