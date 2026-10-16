#!/usr/bin/env python3
"""The scaling sweep's lines (scripts/scaling-sweep.sh) as Markdown tables (decisions.md
R-THREADS): for each quality and block, a row per instance count, a column per worker count,
each cell the blocks' mean share of the period / the worst block's / late blocks / voices'
runs not heard. `scaling-table.py FILE [--split|--threads]` (`--split` reads
ultra-split-sweep.sh's, `--threads` threads-measure.sh's: the built engine's own counts)."""

import sys
from collections import defaultdict


def parse(path, tag):
    rows = []
    for line in open(path):
        if not line.startswith(tag):
            continue
        kv = dict(x.split("=", 1) for x in line[len(tag):].split() if "=" in x)
        rows.append(kv)
    return rows


def cell(r):
    late = int(r["late"])
    silent = int(r["silent_runs"])
    s = f"{float(r['mean']):.0f}/{float(r['worst']):.0f}"
    if late or silent:
        s += f" **{late}**/{silent}"
    return s


def scaling(path):
    rows = parse(path, "scaling: ")
    groups = defaultdict(dict)
    first = {}
    for r in rows:
        key = (r["quality"], int(r["block"]))
        if r["share"] == "first":
            first[(key, int(r["instances"]))] = r
        else:
            groups[key][(int(r["instances"]), int(r["workers"]))] = r
    for key in sorted(groups, key=lambda k: (k[0] != "hi", -k[1])):
        g = groups[key]
        ws = sorted({w for _, w in g})
        print(f"\n{key[0].upper()}, {key[1]} frames (mean/worst % of the period; **late**/silent runs):\n")
        print("| instances | " + " | ".join(f"{w} w" for w in ws) + " | today's (4, first come) |")
        print("|---|" + "---|" * (len(ws) + 1))
        for i in sorted({i for i, _ in g}):
            cells = [cell(g[(i, w)]) if (i, w) in g else "" for w in ws]
            f = first.get((key, i))
            print(f"| {i} | " + " | ".join(cells) + f" | {cell(f) if f else ''} |")


def split(path):
    rows = parse(path, "split: ")
    print("\n| block | voices | budget | split | pool workers | threads | mean | p99.9 | worst | late | silent |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for r in rows:
        print(
            f"| {r['block']} | {r['voices']} | {r['budget']} | {r['split']} | {r['workers']} | {r['threads']} | "
            f"{float(r['mean']):.0f} | {float(r['p999']):.0f} | {float(r['worst']):.0f} | {r['late']} of {r['blocks']} | {r['silent_runs']} |"
        )


def threads(path):
    rows = parse(path, "scaling: ")
    print("\n| Frames | Case | Held | Mean | Worst | Late | Not heard |")
    print("|---|---|---|---|---|---|---|")
    for r in rows:
        args = dict(a.split("=", 1) for a in r.get("args", "").split(",") if "=" in a)
        q, poly = r["quality"].upper(), r.get("poly") == "true"
        if q == "ULTRA":
            who = f"{r['voices']} POLY voices" if poly else "the one instrument"
            how = "whole" if args.get("CA72_SPLIT") == "0" else "auto"
            case = f"ULTRA, {who}, {how}"
        else:
            case = f"HI, {r['instances']} instance(s) of {r['voices']} voices"
        print(
            f"| {r['block']} | {case} | {r['held']} | {float(r['mean']):.0f} | {float(r['worst']):.0f} | "
            f"{r['late']} of {r['blocks']} | {r['silent_runs']} |"
        )


if __name__ == "__main__":
    mode = {"--split": split, "--threads": threads}
    next((mode[a] for a in sys.argv if a in mode), scaling)(sys.argv[1])
