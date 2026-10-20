#!/usr/bin/env bash
# ULTRA's split measured again on a later build (decisions.md R-THREADS): the prototype's split
# grid for one and two voices (`tests/ultra_split.rs`: whole on 4 or 12 threads, split on 6 or
# 12) and the built engine's checks (`tests/scaling_bench.rs` with the plug-in's own counts:
# the one instrument and two POLY voices at ULTRA, split by this machine's rule and whole, in
# turn, twice), at 256 and 128 frames, 10 s each. One line a run, to stdout and to $1.
set -euo pipefail
out="${1:-ultra-recheck.txt}"
seconds="${SECONDS_EACH:-10}"
cd "$(dirname "$0")/.."
exe() {
    cargo test --release -p ca72-plugin --test "$1" --no-run --message-format=json 2>/dev/null |
        grep -o "\"executable\":\"[^\"]*$1[^\"]*\"" | tail -1 | cut -d'"' -f4
}
split=$(exe ultra_split)
bench=$(exe scaling_bench)
echo "# $(date -u +%FT%TZ) $(uname -srm) cpus=$(getconf _NPROCESSORS_ONLN) $(git rev-parse --short HEAD 2>/dev/null || echo copy) load $(uptime | sed 's/.*load average[s]*: //')" | tee -a "$out"
for block in 256 128; do
    for c in "1 4 0" "1 6 1" "1 12 1" "2 12 0" "2 12 2"; do
        set -- $c
        echo "# load $(uptime | sed 's/.*load average[s]*: //')" | tee -a "$out"
        env CA72_SECONDS="$seconds" CA72_BLOCK=$block CA72_VOICES=$1 CA72_BUDGET=$2 CA72_SPLIT=$3 \
            "$split" split_ultra --ignored --nocapture --test-threads 1 2>/dev/null |
            grep -o 'split:.*' | tee -a "$out" || echo "# failed: split $block $c" | tee -a "$out"
    done
    for rep in 1 2; do
        for s in auto 0; do
            for who in "CA72_POLY=0 CA72_VOICES=1" "CA72_VOICES=2"; do
                extra=$who; [ "$s" = 0 ] && extra="$who CA72_SPLIT=0"
                echo "# load $(uptime | sed 's/.*load average[s]*: //')" | tee -a "$out"
                env CA72_AUTO=1 CA72_SECONDS="$seconds" CA72_QUALITY=ultra CA72_BLOCK=$block $extra \
                    "$bench" --ignored --nocapture --test-threads 1 2>/dev/null |
                    grep -o 'scaling:.*' | sed "s/\$/ args=$(echo "$extra" | tr ' ' ',') rep=$rep/" | tee -a "$out" ||
                    echo "# failed: built $block $extra" | tee -a "$out"
            done
        done
    done
done
