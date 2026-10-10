#!/usr/bin/env bash
# render_hp.sh LAB_BIN OUTDIR [QUALITY]: session HP's takes through the CA-72 (ca72-lab stim),
# each take's panel with its FILTER MODE (events.filter_mode) added (decisions.md R-HP).
# CAPTURES overrides the session's directory on the lab's share.
set -euo pipefail
bin=$1 out=$2 q=${3:-no-compromises}
C=${CAPTURES:-/mnt/lab/artifacts/ca-72/calibration/captures/2026-10-09-HP}
mkdir -p "$out"
for j in "$C"/[0-9][0-9]_*.json; do
  b=$(basename "$j" .json)
  python3 - "$j" "$out/$b.patch.json" <<'PY'
import json, sys
t = json.load(open(sys.argv[1]))
panel = dict(t["panel"])
panel["filter_mode"] = t["events"].get("filter_mode", "lo")
json.dump({"panel": panel}, open(sys.argv[2], "w"))
PY
  nice -n19 "$bin" stim "$C/$b.wav" "$out/$b.patch.json" "$out/$b.wav" --quality "$q" >/dev/null
  echo "$b"
done
