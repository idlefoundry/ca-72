#!/usr/bin/env bash
# Builds the Audio Unit, target/bundled/CA-72.component, from the CLAP bundle in
# target/bundled (decisions.md R30): clap-wrapper's AUv2 wrapper (third_party/clap-wrapper,
# with the CLAP headers and Apple's AudioUnitSDK beside it) around that CLAP, which goes
# inside the component. Build the bundles first (README, Building); for both Apple silicon
# and Intel, `cargo xtask bundle-universal`, and the component takes the same
# architectures. macOS only; needs CMake 3.21 or later and Xcode's command line tools.
#
# To play it, copy it into ~/Library/Audio/Plug-Ins/Components (or the system's
# /Library/Audio/Plug-Ins/Components, as the installer does).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -n 1)"
bundles="$root/target/bundled"
clap="$bundles/CA-72.clap"
build="$root/target/auv2"
third="$root/third_party"

if [ "$(uname -s)" != Darwin ]; then
    echo "auv2.sh: Audio Units are macOS's" >&2
    exit 1
fi
if [ ! -e "$clap/Contents/MacOS/CA-72" ]; then
    echo "auv2.sh: $clap is missing: build the bundles first (README, Building)" >&2
    exit 1
fi
command -v cmake > /dev/null || { echo "auv2.sh: CMake is not installed" >&2; exit 1; }

# The CLAP's architectures, as CMake lists them ("arm64;x86_64").
archs="$(lipo -archs "$clap/Contents/MacOS/CA-72" | tr ' ' ';')"

# Afresh each time: the wrapper reads the CLAP while it builds (the component's name, codes
# and description go into its Info.plist) and copies it in, and CMake would not see that the
# CLAP has changed.
rm -rf "$build"
cmake -S "$root/scripts/auv2" -B "$build" -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_OSX_ARCHITECTURES="$archs" \
    -DCA72_CLAP="$clap" -DCA72_VERSION="$version" \
    -DCLAP_WRAPPER_ROOT="$third/clap-wrapper" -DCLAP_SDK_ROOT="$third/clap" \
    -DAUDIOUNIT_SDK_ROOT="$third/AudioUnitSDK" > "$build.log"
cmake --build "$build" --config Release --parallel >> "$build.log"

# Signed ad hoc, as the bundler signs the others; scripts/package.sh signs it for a release.
rm -rf "$bundles/CA-72.component"
ditto "$build/CA-72.component" "$bundles/CA-72.component"
codesign --force --sign - "$bundles/CA-72.component"
codesign --verify --strict "$bundles/CA-72.component"
echo "Created an Audio Unit bundle at '$bundles/CA-72.component' ($(lipo -archs "$bundles/CA-72.component/Contents/MacOS/CA-72"))"
