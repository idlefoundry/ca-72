#!/usr/bin/env bash
# macOS: the effects' helper is never a second application in the Dock (decisions.md R-FX-DOCK).
# Starts a built CA-72.app with a throwaway session (no audio or MIDI devices) and one effect,
# either restored with the session (EFFECTS.json, a chain saved by the app: the helpers then
# start as the app does) or added from the console (a plug-in file), optionally opens its
# window, and checks what LaunchServices lists: one
# Foreground application with the app's bundle identifier, and each helper an agent
# ("UIElement") with the helper bundle's identifier. The app is opened through LaunchServices
# (`open`), as the Finder and the Dock open it: started from a shell, the old helper did not
# show the fault. Then ends the app (its session is thrown away, so the unclean end matters
# to nothing) and checks that no helper is left.
#
#   scripts/test-macos-effects-host.sh APP.app EFFECT.(vst3|clap)|EFFECTS.json [--editor]
#
# The CA-72's window opens on the screen while it runs (about 15 s). A plug-in file's path
# must have no spaces (the console splits on them).
set -euo pipefail
app="${1:?usage: $0 APP.app EFFECT [--editor]}"
effect="${2:?usage: $0 APP.app EFFECT [--editor]}"
editor="${3:-}"
id="$(plutil -extract CFBundleIdentifier raw "$app/Contents/Info.plist")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/state" "$work/shared"
case "$effect" in
*.json) cp "$effect" "$work/state/effects.json" ;;
esac
mkfifo "$work/in"
open -n -a "$app" --stdin "$work/in" --stdout "$work/out" --stderr "$work/err" \
    --env IDLE_FOUNDRY_SHARED="$work/shared" --env CA72_FX_STANDARD_FOLDERS=0 \
    --args --no-devices --fx-console --state-dir "$work/state" &
exec 3> "$work/in"
pid=""
for _ in $(seq 1 100); do
    pid="$(pgrep -f -- "--state-dir $work/state" | head -n 1 || true)"
    [ -n "$pid" ] && break
    sleep 0.1
done
[ -n "$pid" ] || { echo "FAIL: the app did not start" >&2; exit 1; }
sleep 4
case "$effect" in
*.json) ;;
*) echo "add-file $effect" >&3 ;;
esac
sleep 6
if [ "$editor" = --editor ]; then
    echo "editor 1" >&3
    sleep 3
fi
echo "chain" >&3
sleep 1

# Each LaunchServices entry as "pid type bundle".
entries() {
    lsappinfo list | awk '
        /^ *[0-9]+\) "/ { bundle = "-" }
        /bundleID=/ { b = $0; sub(/.*bundleID=/, "", b); gsub(/"/, "", b); gsub(/ /, "", b); bundle = b }
        /pid = [0-9]+ / {
            p = $0; sub(/.*pid = /, "", p); sub(/ .*/, "", p)
            t = $0; sub(/.*type="/, "", t); sub(/".*/, "", t)
            print p, t, bundle
        }'
}
failed=0
listed="$(entries)"
apps="$(awk -v id="$id" '$3 == id' <<< "$listed")"
echo "with the app's identifier ($id):"
echo "${apps:-  none}"
if [ "$(grep -c . <<< "$apps")" != 1 ] || [ "$(awk '{print $2}' <<< "$apps")" != Foreground ] ||
    [ "$(awk '{print $1}' <<< "$apps")" != "$pid" ]; then
    echo "FAIL: not exactly one Foreground application, the app's own" >&2
    failed=1
fi
helpers="$(pgrep -P "$pid" || true)"
if [ -z "$helpers" ]; then
    echo "FAIL: no helper running" >&2
    failed=1
fi
for h in $helpers; do
    row="$(awk -v p="$h" '$1 == p' <<< "$listed")"
    echo "helper $h: ${row:-not listed} ($(ps -o comm= -p "$h"))"
    case "$row" in
    "$h UIElement $id.effects-host") ;;
    "") echo "FAIL: helper $h never checked in as an agent" >&2; failed=1 ;;
    *) echo "FAIL: helper $h is not an agent with the helper bundle's identifier" >&2; failed=1 ;;
    esac
done
grep -v '^$' "$work/out" | sed 's/^/console: /' | tail -8
kill -TERM "$pid" 2>/dev/null || true
exec 3>&-
for _ in $(seq 1 50); do kill -0 "$pid" 2>/dev/null || break; sleep 0.1; done
kill -0 "$pid" 2>/dev/null && kill -KILL "$pid"
sleep 1
for h in $helpers; do
    if kill -0 "$h" 2>/dev/null; then
        echo "FAIL: helper $h left running" >&2
        failed=1
    fi
done
if [ "$failed" = 0 ]; then echo "OK"; fi
exit "$failed"
