#!/usr/bin/env python3
"""Tables from `scripts/cpu-compare.sh`'s log, and `preset_cost`'s at each QUALITY (issue #62: 0.2.8,
`ver=028`, against 0.3.0, `ver=030`): `cpu-compare-table.py LABEL PACED_LOG [COST_LOG]`.
"""

import re
import statistics as st
import sys
from collections import defaultdict

SOUNDS = ["one", "ten", "brass", "horn", "unison", "idle", "one-knob", "ten-knob"]
NAMES = {
    "one": "One voice (Felt Bass)",
    "ten": "Ten POLY voices",
    "brass": "Brass Choir as set",
    "horn": "Horn Bloom as set",
    "unison": "Swarm Riser (UNISON)",
    "idle": "Idle (ten voices, no note)",
    "one-knob": "One voice, CUTOFF moving",
    "ten-knob": "Ten voices, CUTOFF moving",
}
QS = ["lo", "hi", "ultra"]


def kv(line):
    return dict(re.findall(r"(\w+)=(\S+)", line))


def paced(path):
    runs = defaultdict(list)  # (sound, q, block, rate, ver) -> [dict]
    tries = defaultdict(list)
    for line in open(path):
        if "RESULT" not in line:
            continue
        d = kv(line)
        if "mean" not in d:
            print(f"(failed run: {line.strip()})", file=sys.stderr)
            continue
        key = (d["sound"], d["quality"], int(d["block"]), float(d["rate"]), d["ver"])
        tries[key + (d["round"],)].append(d)
    # Each round's kept run: the first not busy, else the last try.
    for k, ds in tries.items():
        kept = next((d for d in ds if d["busy"] == "0"), ds[-1])
        kept["discarded"] = len(ds) - 1
        runs[k[:5]].append(kept)
    return runs


def f(d, k):
    return float(d[k])


def summary(ds):
    m = st.mean(f(d, "mean") for d in ds)
    p = st.mean(f(d, "p99") for d in ds)
    tot = st.mean(f(d, "total_cpu") for d in ds)
    late = "+".join(d["late"] for d in ds)
    unheard = "+".join(d["unheard"] for d in ds)
    spread = max(f(d, "mean") for d in ds) - min(f(d, "mean") for d in ds)
    busy = sum(int(d["busy"]) for d in ds)
    demoted = sum(int(d["demoted"]) for d in ds)
    others = max(f(d, "others") for d in ds)
    discarded = sum(d["discarded"] for d in ds)
    return dict(mean=m, p99=p, tot=tot, late=late, unheard=unheard, spread=spread,
                busy=busy, demoted=demoted, others=others, discarded=discarded, n=len(ds))


def ratio(a, b):
    return f"{(b / a - 1) * 100:+.0f} %" if a > 0.5 else "—"


def paced_tables(label, path):
    runs = paced(path)
    rates = sorted({k[3] for k in runs})
    blocks = sorted({k[2] for k in runs})
    out = []
    notes = []
    for rate in rates:
        for block in blocks:
            if not any(k[2] == block and k[3] == rate for k in runs):
                continue
            out.append(f"\n**{label}, {rate / 1000:g} kHz, {block} frames** (each cell the mean of two runs of 20 s)\n")
            out.append("| Sound | QUALITY | Host thread mean / p99, 0.2.8 | 0.3.0 | Change (mean) | Total CPU, 0.2.8 | 0.3.0 | Late blocks 0.2.8 → 0.3.0 | Voice runs unheard 0.2.8 → 0.3.0 |")
            out.append("|---|---|---:|---:|---:|---:|---:|---|---|")
            for s in SOUNDS:
                for q in QS:
                    a = runs.get((s, q, block, rate, "028"))
                    b = runs.get((s, q, block, rate, "030"))
                    if not a or not b:
                        continue
                    A, B = summary(a), summary(b)
                    flag = ""
                    for v, S in (("0.2.8", A), ("0.3.0", B)):
                        if S["busy"]:
                            notes.append(f"{label} {rate:g} {block} {s} {q} {v}: kept a busy run (others {S['others']:.2f} cores)")
                        if S["demoted"]:
                            flag = " ‡"
                        if S["n"] < 2:
                            notes.append(f"{label} {rate:g} {block} {s} {q} {v}: only {S['n']} run(s)")
                        if S["mean"] > 2 and S["spread"] / S["mean"] > 0.2:
                            notes.append(f"{label} {rate:g} {block} {s} {q} {v}: runs differ, means {', '.join(d['mean'] for d in (a if v == '0.2.8' else b))}")
                        if S["discarded"]:
                            notes.append(f"{label} {rate:g} {block} {s} {q} {v}: {S['discarded']} run(s) discarded as busy")
                    out.append(
                        f"| {NAMES[s]} | {q.upper()} | {A['mean']:.1f} / {A['p99']:.1f} % | {B['mean']:.1f} / {B['p99']:.1f} % | {ratio(A['mean'], B['mean'])} | {A['tot']:.0f} % | {B['tot']:.0f} % | {A['late']} → {B['late']}{flag} | {A['unheard']} → {B['unheard']} |"
                    )
    return "\n".join(out), notes


def cost(path):
    """preset_cost: per (ver, quality) the per-voice cost of each preset, least of the rounds."""
    cur = None
    vals = defaultdict(lambda: defaultdict(list))
    for line in open(path):
        m = re.match(r"== round=(\d+) ver=(\d+) quality=(\w+)", line)
        if m:
            cur = (m.group(2), m.group(3))
            continue
        m = re.match(r"(.+?) \(POLY (\d+)\)\s+([\d.]+) %\s+([\d.]+) %", line)
        if m and cur:
            vals[cur][m.group(1).strip()].append(float(m.group(3)) / int(m.group(2)))
    res = {}
    for k, presets in vals.items():
        res[k] = {p: min(v) for p, v in presets.items()}
    return res


def cost_table(label, path):
    res = cost(path)
    out = [f"\n**{label}, offline cost per voice** (`preset_cost`, one thread, 256-frame blocks at 48 kHz, the 19 factory presets' sounds on POLY's voices without DOUBLE: ten voices at LO and HI, four at ULTRA; % of a core a voice, the least of two rounds)\n"]
    out.append("| QUALITY | 0.2.8 mean (range) | 0.3.0 mean (range) | Change, mean of the presets' (range) |")
    out.append("|---|---:|---:|---:|")
    for q in QS:
        a, b = res.get(("028", q)), res.get(("030", q))
        if not a or not b:
            continue
        common = sorted(set(a) & set(b))
        ch = [b[p] / a[p] - 1 for p in common]
        out.append(
            f"| {q.upper()} | {st.mean(a.values()):.2f} ({min(a.values()):.2f}–{max(a.values()):.2f}) | {st.mean(b.values()):.2f} ({min(b.values()):.2f}–{max(b.values()):.2f}) | {st.mean(ch) * 100:+.1f} % ({min(ch) * 100:+.1f} to {max(ch) * 100:+.1f}) |"
        )
    return "\n".join(out)


if __name__ == "__main__":
    label, p = sys.argv[1], sys.argv[2]
    t, notes = paced_tables(label, p)
    print(t)
    if len(sys.argv) > 3:
        print(cost_table(label, sys.argv[3]))
    if notes:
        print("\nNotes:")
        for n in notes:
            print("- " + n)
