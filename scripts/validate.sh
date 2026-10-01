#!/usr/bin/env bash
# Validates the bundles in target/bundled (`cargo xtask bundle ca72-plugin --release`) with
# clap-validator, pluginval at strictness 10 and Steinberg's VST3 validator. Each is fetched,
# or for Steinberg's built from the VST3 SDK, into $CA72_VALIDATORS (default
# target/validators). Needs gh (for the releases), unzip, git and cmake; on Linux without
# a display, xvfb-run for pluginval's editor tests.
set -euo pipefail

CLAP_VALIDATOR=0.4.1
PLUGINVAL=v1.0.4
VST3_SDK=v3.8.1_build_84

root="$(cd "$(dirname "$0")/.." && pwd)"
bundles="$root/target/bundled"
dir="${CA72_VALIDATORS:-$root/target/validators}"
mkdir -p "$dir"

case "$(uname -s)" in
Darwin) os=macos clap_asset='*-macos-universal.zip' pluginval_asset=pluginval_macOS.zip ;;
Linux) os=linux clap_asset='*-ubuntu-22.04.zip' pluginval_asset=pluginval_Linux.zip ;;
MINGW* | MSYS* | CYGWIN*) os=windows clap_asset='*-windows.zip' pluginval_asset=pluginval_Windows.zip ;;
*) echo "validate.sh: $(uname -s) is not a platform the plug-in is built for" >&2; exit 1 ;;
esac
exe=""
[ "$os" = windows ] && exe=.exe

# clap-validator's macOS and Linux archives hold a tarball, keeping the binary executable
# (on macOS, in a folder of its own).
work="$dir/clap-validator-$CLAP_VALIDATOR"
clap=""
[ -d "$work" ] && clap="$(find "$work" -type f -name "clap-validator$exe" | head -n 1)"
if [ -z "$clap" ]; then
    mkdir -p "$work"
    gh release download "$CLAP_VALIDATOR" -R free-audio/clap-validator -p "$clap_asset" -D "$work" --clobber
    (cd "$work" && unzip -o -q ./*.zip && for t in ./*.tar.gz; do [ -e "$t" ] && tar xzf "$t"; done; true)
    clap="$(find "$work" -type f -name "clap-validator$exe" | head -n 1)"
fi

pluginval_dir="$dir/pluginval-$PLUGINVAL"
case "$os" in
macos) pluginval="$pluginval_dir/pluginval.app/Contents/MacOS/pluginval" ;;
*) pluginval="$pluginval_dir/pluginval$exe" ;;
esac
if [ ! -x "$pluginval" ]; then
    mkdir -p "$pluginval_dir"
    gh release download "$PLUGINVAL" -R Tracktion/pluginval -p "$pluginval_asset" -D "$pluginval_dir" --clobber
    (cd "$pluginval_dir" && unzip -o -q "$pluginval_asset")
    chmod +x "$pluginval"
fi

# Only the validator's own submodules, and none of the SDK's examples or VSTGUI.
sdk="$dir/vst3sdk-$VST3_SDK"
vst3=""
[ -d "$sdk/build/bin" ] && vst3="$(find "$sdk/build/bin" -type f -name "validator$exe" | head -n 1)"
if [ -z "$vst3" ]; then
    rm -rf "$sdk"
    git clone -q --depth 1 --branch "$VST3_SDK" https://github.com/steinbergmedia/vst3sdk "$sdk"
    git -C "$sdk" submodule update -q --init --depth 1 base cmake pluginterfaces public.sdk
    cmake -S "$sdk" -B "$sdk/build" -DCMAKE_BUILD_TYPE=Release \
        -DSMTG_ENABLE_VST3_PLUGIN_EXAMPLES=OFF -DSMTG_ENABLE_VST3_HOSTING_EXAMPLES=OFF \
        -DSMTG_ENABLE_VSTGUI_SUPPORT=OFF
    cmake --build "$sdk/build" --config Release --target validator --parallel
    vst3="$(find "$sdk/build/bin" -type f -name "validator$exe" | head -n 1)"
fi

display=()
if [ "$os" = linux ] && [ -z "${DISPLAY:-}" ]; then
    display=(xvfb-run -a)
fi

echo "== clap-validator $CLAP_VALIDATOR"
"$clap" validate "$bundles/CA-72.clap"
echo "== pluginval $PLUGINVAL, strictness 10"
${display[@]+"${display[@]}"} "$pluginval" --strictness-level 10 --validate-in-process --validate "$bundles/CA-72.vst3"
echo "== Steinberg's VST3 validator, SDK $VST3_SDK"
"$vst3" "$bundles/CA-72.vst3"
