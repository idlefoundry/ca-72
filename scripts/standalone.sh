#!/usr/bin/env bash
# Build the shipped app into target/bundled. The legacy standalone example remains
# available separately for JACK and development. macOS builds both architectures.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
profile="${CA72_STANDALONE_PROFILE:-bundle}"
target_dir="${CARGO_TARGET_DIR:-$root/target}"
mkdir -p "$root/target/bundled"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
build() {
    cargo build --locked -p ca72-plugin --bin ca72-standalone \
        --features standalone-production --profile "$profile" "$@"
}
case "$(uname -s)" in
Darwin)
    build --target aarch64-apple-darwin
    build --target x86_64-apple-darwin
    app="$root/target/bundled/CA-72.app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    lipo -create "$target_dir/aarch64-apple-darwin/$profile/ca72-standalone" \
        "$target_dir/x86_64-apple-darwin/$profile/ca72-standalone" \
        -output "$app/Contents/MacOS/ca72-standalone"
    cp scripts/standalone/CA-72.icns "$app/Contents/Resources/"
    cp LICENSE THIRD-PARTY-NOTICES.txt "$app/Contents/Resources/"
    python3 - "$app/Contents/Info.plist" "$version" <<'PY'
import plistlib
import sys
with open(sys.argv[1], "wb") as stream:
    plistlib.dump({
        "CFBundleIdentifier": "com.idlefoundry.ca72.standalone",
        "CFBundleName": "CA-72", "CFBundleDisplayName": "CA-72",
        "CFBundleExecutable": "ca72-standalone", "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": sys.argv[2], "CFBundleVersion": sys.argv[2],
        "CFBundleIconFile": "CA-72.icns", "LSMinimumSystemVersion": "11.0",
        "NSHighResolutionCapable": True,
        "NSMicrophoneUsageDescription": "CA-72 uses your selected audio input to play external sound through its synthesizer circuit.",
        "NSHumanReadableCopyright": "Copyright © 2026 Idle Foundry Ltd. GNU GPL version 3 or later.",
    }, stream)
PY
    lipo -verify_arch arm64 x86_64 "$app/Contents/MacOS/ca72-standalone"
    printf '%s\n' "$app"
    ;;
Linux)
    build
    cp "$target_dir/$profile/ca72-standalone" "$root/target/bundled/ca72-standalone"
    cp scripts/standalone/CA-72.desktop scripts/standalone/com.idlefoundry.ca72.png "$root/target/bundled/"
    printf '%s\n' "$root/target/bundled/ca72-standalone"
    ;;
MINGW*|MSYS*|CYGWIN*)
    build
    cp "$target_dir/$profile/ca72-standalone.exe" "$root/target/bundled/CA-72.exe"
    cp scripts/standalone/CA-72.ico "$root/target/bundled/"
    printf '%s\n' "$root/target/bundled/CA-72.exe"
    ;;
*) printf 'standalone.sh: unsupported platform\n' >&2; exit 1 ;;
esac
