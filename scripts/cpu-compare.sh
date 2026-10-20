#!/usr/bin/env bash
# Two builds' processor cost compared, paced as a host plays them (issue #62,
# `tests/cpu_compare.rs`): each case run with one build, then the other (the order turned
# each round), so that the computer's drift falls on both alike. A case run while other
# processes took more than MAXOTHERS cores is run again, up to twice, and every run is kept in
# the log with `busy=` and `try=`; `scripts/cpu-compare-table.py` takes the first quiet one.
#
# usage: cpu-compare.sh <cpu_compare executable, before> <the same, after> <log> (logged as
# `ver=028` and `ver=030`, the versions first compared)
# (build each with `cargo test --release -p ca72-plugin --test cpu_compare --no-run`; an
# older version needs the test file copied into its tree, with `libc` (macOS) and winapi's
# `processthreadsapi` and `realtimeapiset` (Windows) among its dev-dependencies.)
# Environment: SOUNDS, QUALS, BLOCKS, RATE (48000), ROUNDS (2), SECS (20), MAXOTHERS (1.0),
# PAUSE (2 s between runs). Set CA72_SEATS_DIR and IDLE_FOUNDRY_SHARED to folders of the
# measurement's own, so that no instance the computer's owner has open is touched.
set -u
B028=$1
B030=$2
OUT=$3
SOUNDS=${SOUNDS:-"one ten brass horn unison idle one-knob ten-knob"}
QUALS=${QUALS:-"lo hi ultra"}
BLOCKS=${BLOCKS:-"128 512"}
RATE=${RATE:-48000}
ROUNDS=${ROUNDS:-2}
SECS=${SECS:-20}
MAXOTHERS=${MAXOTHERS:-1.0}
PAUSE=${PAUSE:-2}
ERR="$OUT.stderr"
load() { uptime 2>/dev/null | sed 's/.*load average[s]*: *//; s/,//g' | awk '{print $1}'; }
for round in $(seq 1 "$ROUNDS"); do
  if [ $((round % 2)) -eq 1 ]; then order="028 030"; else order="030 028"; fi
  for s in $SOUNDS; do for q in $QUALS; do for b in $BLOCKS; do
    for v in $order; do
      if [ "$v" = 028 ]; then bin=$B028; else bin=$B030; fi
      try=0
      while :; do
        l0=$(load)
        echo "== v$v r$round $s $q $b try$try" >>"$ERR"
        line=$(CA72_SOUND=$s CA72_QUALITY=$q CA72_BLOCK=$b CA72_RATE=$RATE CA72_SECONDS=$SECS \
          "$bin" --ignored --nocapture --exact paced 2>>"$ERR" | grep '^RESULT')
        others=$(echo "$line" | sed -n 's/.*others=\([0-9.na]*\).*/\1/p')
        busy=$(awk -v o="${others:-99}" -v m="$MAXOTHERS" 'BEGIN{print (o+0>m+0)?1:0}')
        echo "ver=$v round=$round try=$try load=${l0:-na} busy=$busy ${line:-RESULT failed sound=$s quality=$q block=$b}" >>"$OUT"
        if [ "$busy" = 0 ] || [ $try -ge 2 ]; then break; fi
        try=$((try + 1))
        sleep 30
      done
      sleep "$PAUSE"
    done
  done; done; done
done
echo "DONE $(date)" >>"$OUT"
