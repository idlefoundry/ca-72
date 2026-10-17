#!/usr/bin/env bash
# The built engine's threads by the machine (decisions.md R-THREADS), against the prototype's
# figures: `tests/scaling_bench.rs` with the plug-in's own counts (`CA72_AUTO=1`). ULTRA's one
# instrument and two POLY voices, split (this machine's crews) and whole (`CA72_SPLIT=0`) in
# turn, twice; four ULTRA voices (more than split); HI's ten voices in 1, 2, 4 and 8 instances.
# 256 and 128 frames, 10 s each; one `scaling:` line a run, to stdout and to $1.
set -euo pipefail
out="${1:-threads.txt}"
seconds="${SECONDS_EACH:-10}"
cd "$(dirname "$0")/.."
bin=$(cargo test --release -p ca72-plugin --test scaling_bench --no-run --message-format=json 2>/dev/null |
    grep -o '"executable":"[^"]*scaling_bench[^"]*"' | tail -1 | cut -d'"' -f4)
echo "# $(date -u +%FT%TZ) $(uname -srm) cpus=$(getconf _NPROCESSORS_ONLN) load $(uptime | sed 's/.*load average[s]*: //')" | tee -a "$out"
run() {
    env CA72_AUTO=1 CA72_SECONDS="$seconds" "$@" "$bin" --ignored --nocapture --test-threads 1 2>/dev/null |
        grep -o 'scaling:.*' | sed "s/\$/ args=$(echo "$*" | tr ' ' ',')/" | tee -a "$out" || echo "# failed: $*" | tee -a "$out"
}
for block in 256 128; do
    for rep in 1 2; do
        for split in auto 0; do
            s=(); [ "$split" = 0 ] && s=(CA72_SPLIT=0)
            run ${s[@]+"${s[@]}"} CA72_QUALITY=ultra CA72_BLOCK=$block CA72_POLY=0 CA72_VOICES=1
            run ${s[@]+"${s[@]}"} CA72_QUALITY=ultra CA72_BLOCK=$block CA72_VOICES=2
        done
    done
    run CA72_QUALITY=ultra CA72_BLOCK=$block CA72_VOICES=4
    for instances in 1 2 4 8; do
        run CA72_QUALITY=hi CA72_BLOCK=$block CA72_INSTANCES=$instances
    done
done
