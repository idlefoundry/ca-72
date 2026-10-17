#!/usr/bin/env bash
# Hosts that run each plug-in in a process of its own (decisions.md R-THREADS): `$1` processes
# of the scaling bench, one instance each (HI, ten voices, the plug-in's own counts), started
# together, each thinking itself alone. Guarded: an ordinary-priority canary sleeps 10 ms at a
# time and logs its worst oversleep; past CANARY_LIMIT_MS (100) every bench process is killed
# at once (ABORT); each process has a hard timeout, and the round one too. Lines to $2.
set -u
n="$1"; out="$2"; limit="${CANARY_LIMIT_MS:-100}"
cd "$(dirname "$0")/.."
bin=$(ls -t target/release/deps/scaling_bench-* | grep -v '\.d$' | head -1)
# (Matched by its process name, the binary's first 15 characters: nothing else has it.)
name=$(basename "$bin" | cut -c1-15)
dir=$(mktemp -d)
# (The processes' own seats folder, shared by them alone: decisions.md R-THREADS.)
export CA72_SEATS_DIR="$dir/seats"
echo "# $(date -u +%FT%TZ) processes=$n load $(cut -d' ' -f1-3 /proc/loadavg)" | tee -a "$out"
# The canary: its worst oversleep, in ms, rewritten every 100 ms.
python3 -I - "$dir/canary" <<'PY' &
import sys, time
path = sys.argv[1]; worst = 0.0; last = time.monotonic()
while True:
    t = time.monotonic(); time.sleep(0.01)
    worst = max(worst, (time.monotonic() - t - 0.01) * 1e3)
    if time.monotonic() - last > 0.1:
        open(path, "w").write(f"{worst:.2f}\n"); last = time.monotonic()
PY
canary=$!
sleep 0.5
# A second guard above the audio threads (SCHED_FIFO 98, the most this user may take), which
# an ordinary monitor could not be were they to starve the machine: it sleeps between looks.
chrt -f 98 bash -c '
    while sleep 0.1; do
        read -r w < "$1" 2>/dev/null || continue
        if (( ${w%.*} > $2 )); then pkill -KILL -x "$3"; echo "rt guard: $w ms" > "$1.guard"; fi
        [ -e "$1.stop" ] && exit 0
    done' guard "$dir/canary" "$limit" "$name" &
guard=$!
# (Every process's blocks from the same moment, once all have started and taken their seats.)
start_at=$(python3 -I -c 'import time; print(time.time() + 8)')
for k in $(seq 1 "$n"); do
    timeout -s KILL 50 env CA72_START_AT="$start_at" CA72_AUTO=1 CA72_INSTANCES=1 CA72_QUALITY=hi CA72_SECONDS=8 \
        "$bin" --ignored --nocapture --test-threads 1 > "$dir/out.$k" 2> "$dir/err.$k" &
done
abort=0; t0=$(date +%s)
while pgrep -x "$name" > /dev/null; do
    w=$(cat "$dir/canary" 2>/dev/null || echo 0)
    if awk -v w="$w" -v l="$limit" 'BEGIN { exit !(w > l) }'; then
        pkill -KILL -x "$name"; abort=1
        echo "# ABORT: the canary overslept ${w} ms" | tee -a "$out"; break
    fi
    if [ $(( $(date +%s) - t0 )) -gt 60 ]; then
        pkill -KILL -x "$name"; abort=1; echo "# ABORT: the round's timeout" | tee -a "$out"; break
    fi
    sleep 0.2
done
touch "$dir/canary.stop"
wait "$guard" 2>/dev/null
[ -e "$dir/canary.guard" ] && { abort=1; echo "# ABORT by the real-time guard: $(cat "$dir/canary.guard")" | tee -a "$out"; }
kill "$canary" 2>/dev/null
wait 2>/dev/null
echo "# canary worst ${n} processes: $(cat "$dir/canary") ms; watchdog: $(cat "$dir"/err.* | grep -c '^ca72:') messages" | tee -a "$out"
cat "$dir"/err.* | grep '^ca72:' | sort | uniq -c | head -5 | sed 's/^/# /' | tee -a "$out"
for k in $(seq 1 "$n"); do grep -o 'scaling:.*' "$dir/out.$k" | sed "s/\$/ process=$k/" | tee -a "$out"; done
rm -rf "$dir"
exit $abort
