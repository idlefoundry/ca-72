#!/bin/sh
# Installs CA-72 plug-ins and the standalone app from this archive.
#   ./install.sh                   install for the current user
#   ./install.sh --system          install for everyone (uses sudo)
#   ./install.sh --standalone-only install only the application
#   ./install.sh --plugins-only    install only VST3 and CLAP
#   ./install.sh --uninstall       remove selected components, keep presets/session
#   ./install.sh --prefix PATH     use a chosen installation prefix
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
vst3="$HOME/.vst3"
clap="$HOME/.clap"
bin="$HOME/.local/bin"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
use_sudo=""
system=0
uninstall=0
plugins=1
application=1
install_prefix=""
while [ "$#" -gt 0 ]; do
    case "$1" in
    --system) system=1 ;;
    --uninstall) uninstall=1 ;;
    --standalone-only) plugins=0 ;;
    --plugins-only) application=0 ;;
    --prefix)
        shift
        [ "$#" -gt 0 ] || { echo 'install.sh: --prefix needs a path' >&2; exit 2; }
        install_prefix="$1"
        case "$install_prefix" in /*) ;; *) echo 'install.sh: --prefix must be absolute' >&2; exit 2;; esac
        ;;
    -h|--help) sed -n '2,8s/^# \{0,1\}//p' "$0"; exit 0 ;;
    *) echo "install.sh: unknown option $1 (try --help)" >&2; exit 2 ;;
    esac
    shift
done
[ "$plugins$application" != 00 ] || { echo 'install.sh: choose at least one component' >&2; exit 2; }
if [ "$system" = 1 ]; then
    [ -z "$install_prefix" ] || { echo 'install.sh: choose --system or --prefix' >&2; exit 2; }
    install_prefix=/usr
    if [ "$(id -u)" -ne 0 ]; then use_sudo=sudo; fi
fi
if [ -n "$install_prefix" ]; then
    vst3="$install_prefix/lib/vst3"
    clap="$install_prefix/lib/clap"
    bin="$install_prefix/bin"
    data="$install_prefix/share"
fi
app_data="$data/Idle Foundry/CA-72/Application"
desktop="$data/applications/com.idlefoundry.ca72.desktop"
icon="$data/icons/hicolor/512x512/apps/com.idlefoundry.ca72.png"

if [ "$uninstall" = 1 ]; then
    if [ "$plugins" = 1 ]; then
        $use_sudo rm -rf "$vst3/CA-72.vst3"
        $use_sudo rm -f "$clap/CA-72.clap"
    fi
    if [ "$application" = 1 ]; then
        $use_sudo rm -f "$bin/ca72-standalone" "$desktop" "$icon" \
            "$app_data/LICENSE.txt" "$app_data/THIRD-PARTY-NOTICES.txt"
    fi
    echo 'Removed selected CA-72 components. Presets and standalone session are kept.'
    exit 0
fi

# Validate the complete selected payload before replacing any installed file.
if [ "$plugins" = 1 ]; then
    for f in CA-72.vst3 CA-72.clap; do
        [ -e "$here/$f" ] || { echo "install.sh: $here/$f is missing; unpack the whole archive" >&2; exit 1; }
    done
fi
if [ "$application" = 1 ]; then
    for f in ca72-standalone CA-72.desktop com.idlefoundry.ca72.png LICENSE.txt THIRD-PARTY-NOTICES.txt; do
        [ -f "$here/$f" ] || { echo "install.sh: $here/$f is missing; unpack the whole archive" >&2; exit 1; }
    done
fi
if [ "$plugins" = 1 ]; then
    $use_sudo mkdir -p "$vst3" "$clap"
    $use_sudo rm -rf "$vst3/CA-72.vst3"
    $use_sudo cp -R "$here/CA-72.vst3" "$vst3/"
    $use_sudo install -m 644 "$here/CA-72.clap" "$clap/CA-72.clap"
    echo "Installed plug-ins in $vst3 and $clap. Rescan your host to find CA-72."
fi
if [ "$application" = 1 ]; then
    $use_sudo mkdir -p "$bin" "$(dirname "$desktop")" "$(dirname "$icon")" "$app_data"
    $use_sudo install -m 755 "$here/ca72-standalone" "$bin/ca72-standalone"
    $use_sudo install -m 644 "$here/com.idlefoundry.ca72.png" "$icon"
    $use_sudo install -m 644 "$here/LICENSE.txt" "$app_data/LICENSE.txt"
    $use_sudo install -m 644 "$here/THIRD-PARTY-NOTICES.txt" "$app_data/THIRD-PARTY-NOTICES.txt"
    # Desktop Exec escaping is distinct from shell escaping; %% means literal %.
    escaped="$(printf '%s' "$bin/ca72-standalone" | sed 's/\\/\\\\/g; s/"/\\"/g; s/`/\\`/g; s/\$/\\$/g; s/%/%%/g')"
    { sed '/^Exec=/d' "$here/CA-72.desktop"; printf 'Exec="%s"\n' "$escaped"; } | $use_sudo tee "$desktop" > /dev/null
    echo "Installed standalone app in $bin. Open CA-72 from your applications menu."
fi
if command -v ldd > /dev/null 2>&1; then
    check="$bin/ca72-standalone"
    if [ "$application" = 0 ]; then check="$clap/CA-72.clap"; fi
    missing="$(ldd "$check" 2> /dev/null | sed -n 's/^[[:space:]]*\([^ ]*\) => not found.*/\1/p')"
    if [ -n "$missing" ]; then
        echo "Warning: CA-72 needs libraries this system lacks:" $missing >&2
        echo '  Debian/Ubuntu: libx11-6 libxcb1 libx11-xcb1 libasound2 libgl1; Fedora/Arch: corresponding X11, ALSA and OpenGL libraries.' >&2
    fi
fi
