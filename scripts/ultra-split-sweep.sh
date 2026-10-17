#!/usr/bin/env bash
# The prototype of ULTRA's split voices (decisions.md R-THREADS): `tests/ultra_split.rs` for 1 to 4
# ULTRA voices, 0 to all of them split six ways, out of budgets of 4 to 14 threads (each split
# voice's five helpers taken first, the pool the rest), at 256 and 128 frames; one `split:`
# line a run, to stdout and to $1 (split.txt). On Linux, budgets past the processors less
# three are left out (the host thread and the system's).
set -euo pipefail
out="${1:-split.txt}"
seconds="${SECONDS_EACH:-10}"
cd "$(dirname "$0")/.."
bin=$(cargo test --release -p ca72-plugin --test ultra_split --no-run --message-format=json 2>/dev/null |
    grep -o '"executable":"[^"]*ultra_split[^"]*"' | tail -1 | cut -d'"' -f4)
cpus=$(getconf _NPROCESSORS_ONLN)
echo "# $(date -u +%FT%TZ) $(uname -srm) cpus=$cpus" | tee -a "$out"
for block in 256 128; do
    for voices in 1 2 3 4; do
        for budget in 4 6 8 10 12 14; do
            if [ "$(uname)" = Linux ] && [ "$budget" -gt $((cpus - 3)) ]; then
                continue
            fi
            for split in $(seq 0 "$voices"); do
                [ $((5 * split)) -le "$budget" ] || continue
                env CA72_SECONDS="$seconds" CA72_BLOCK=$block CA72_VOICES=$voices CA72_BUDGET=$budget CA72_SPLIT=$split \
                    "$bin" split_ultra --ignored --nocapture --test-threads 1 2>/dev/null |
                    grep -o 'split:.*' | tee -a "$out" || echo "# failed: $block $voices $budget $split" | tee -a "$out"
            done
        done
    done
done
