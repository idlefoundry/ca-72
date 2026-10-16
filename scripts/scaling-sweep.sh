#!/usr/bin/env bash
# The workers' scaling grid (decisions.md R-THREADS): `tests/scaling_bench.rs` at HI and ULTRA,
# 128 and 256 frames, 1, 2, 4 and 8 instances, 0 to 12 workers shared fairly, and today's
# first-come share of 4; one `scaling:` line a run, to stdout and to $1 (scaling.txt).
# On Linux the grid stops where the host threads and workers together would pass the
# processors less two (the 2026-09-30 stall: more spinning real-time threads than CPUs).
# Quiet machine first; it takes about 50 minutes.
set -euo pipefail
out="${1:-scaling.txt}"
seconds="${SECONDS_EACH:-10}"
cd "$(dirname "$0")/.."
bin=$(cargo test --release -p ca72-plugin --test scaling_bench --no-run --message-format=json 2>/dev/null |
    grep -o '"executable":"[^"]*scaling_bench[^"]*"' | tail -1 | cut -d'"' -f4)
cpus=$(getconf _NPROCESSORS_ONLN)
{
    echo "# $(date -u +%FT%TZ) $(uname -srm) cpus=$cpus $(git rev-parse --short HEAD 2>/dev/null || echo copy)"
    if [ "$(uname)" = Darwin ]; then
        echo "# $(sysctl -n machdep.cpu.brand_string) perflevel0=$(sysctl -n hw.perflevel0.physicalcpu) perflevel1=$(sysctl -n hw.perflevel1.physicalcpu 2>/dev/null || echo 0)"
    else
        echo "# $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2)"
    fi
} | tee -a "$out"
run() {
    env CA72_SECONDS="$seconds" "$@" "$bin" --ignored --nocapture --test-threads 1 2>/dev/null |
        grep -o 'scaling:.*' | tee -a "$out" || echo "# failed: $*" | tee -a "$out"
}
for quality in hi ultra; do
    for block in 256 128; do
        for instances in 1 2 4 8; do
            for workers in 0 1 2 3 4 5 6 7 8 9 10 11 12; do
                if [ "$(uname)" = Linux ] && [ $((instances + workers)) -gt $((cpus - 2)) ]; then
                    continue
                fi
                run CA72_QUALITY=$quality CA72_BLOCK=$block CA72_INSTANCES=$instances CA72_WORKERS=$workers
            done
            if [ "$instances" -gt 1 ]; then
                run CA72_QUALITY=$quality CA72_BLOCK=$block CA72_INSTANCES=$instances CA72_WORKERS=4 CA72_SHARE=first
            fi
        done
    done
done
