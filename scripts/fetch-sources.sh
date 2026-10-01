#!/usr/bin/env bash
# Fetches the Minimoog research sources listed in docs/circuit/sources/manifest.tsv into a
# local cache (default ~/.cache/ca72-sources), checking each file's sha256. The
# sources are not committed: several sites ask that their scans not be re-hosted, and
# vendor SPICE models may not be redistributed (docs/circuit/sources.md).
set -euo pipefail
here="$(cd "$(dirname "$0")/../.." && pwd)"
cache="${CA72_SOURCES:-$HOME/.cache/ca72-sources}"
mkdir -p "$cache"
fail=0
for tsv in "$here"/docs/circuit/sources/manifest.tsv; do
  tail -n +2 "$tsv" | while IFS=$'\t' read -r path url sha rest; do
    case "$url" in http*) ;; *) continue ;; esac
    dest="$cache/$path"
    mkdir -p "$(dirname "$dest")"
    if [ ! -f "$dest" ] || [ "$(sha256sum "$dest" | cut -d' ' -f1)" != "$sha" ]; then
      curl -sSL -A "Mozilla/5.0" -o "$dest" "$url" || { echo "FAILED $url"; continue; }
    fi
    got="$(sha256sum "$dest" | cut -d' ' -f1)"
    if [ "$got" != "$sha" ]; then echo "MISMATCH $path ($got)"; else echo "ok $path"; fi
  done
done
