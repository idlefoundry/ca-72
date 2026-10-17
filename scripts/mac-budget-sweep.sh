#!/usr/bin/env bash
# Whether more than N - 2 - A workers helps many instances on macOS (decisions.md R-THREADS):
# 6 and 8 HI instances (ten voices each), process budgets of 4, 6, 8, 10 and 12 shared evenly
# (`engine::set_budget_for_measurement`), 256 and 128 frames, 3 repeats with the budgets in a
# different order each time so that drift spreads across them; the load and the busiest
# processes before each run. One `scaling:` line a run, to stdout and to $1.
set -euo pipefail
out="${1:-mac-budget.txt}"
seconds="${SECONDS_EACH:-10}"
cd "$(dirname "$0")/.."
bin=$(cargo test --release -p ca72-plugin --test scaling_bench --no-run --message-format=json 2>/dev/null |
    grep -o '"executable":"[^"]*scaling_bench[^"]*"' | tail -1 | cut -d'"' -f4)
echo "# $(date -u +%FT%TZ) $(uname -srm) $(sysctl -n machdep.cpu.brand_string)" | tee -a "$out"
orders=("4 6 8 10 12" "12 8 4 10 6" "6 10 12 4 8")
for rep in 1 2 3; do
    for block in 256 128; do
        for instances in 6 8; do
            for w in ${orders[$((rep - 1))]}; do
                echo "# rep $rep load $(sysctl -n vm.loadavg) top: $(ps -Ao pcpu,comm -r | sed -n 2,4p | awk '{n=split($2,a,"/"); printf "%s %s%%; ", a[n], $1}')" | tee -a "$out"
                env CA72_SECONDS="$seconds" CA72_QUALITY=hi CA72_BLOCK=$block CA72_INSTANCES=$instances CA72_WORKERS=$w \
                    "$bin" --ignored --nocapture --test-threads 1 2>/dev/null |
                    grep -o 'scaling:.*' | sed "s/\$/ rep=$rep/" | tee -a "$out" || echo "# failed: $rep $block $instances $w" | tee -a "$out"
            done
        done
    done
done
