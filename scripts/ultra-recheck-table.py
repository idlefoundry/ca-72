#!/usr/bin/env python3
"""ULTRA's split measured again (scripts/ultra-recheck.sh) beside the earlier figures in
docs/threads (decisions.md R-THREADS): `ultra-recheck-table.py NEW SPLIT_BEFORE BUILT_BEFORE`.
Each cell: the mean and the worst block, % of the period · late blocks · runs not heard."""

import sys
from collections import defaultdict


def lines(path, tag):
    out = []
    for line in open(path, encoding="utf-8-sig"):
        i = line.find(tag)
        if i < 0 or line.startswith("#"):
            continue
        out.append(dict(x.split("=", 1) for x in line[i + len(tag):].split() if "=" in x))
    return out


def cell(rs):
    if not rs:
        return "–"
    m = "/".join(f"{float(r['mean']):.0f}" for r in rs)
    w = "/".join(f"{float(r['worst']):.0f}" for r in rs)
    late = ", ".join(r["late"] for r in rs)
    sil = ", ".join(r["silent_runs"] for r in rs)
    return f"{m} / {w} · {late} · {sil}"


def split_rows(path):
    g = defaultdict(list)
    for r in lines(path, "split: "):
        g[(r["block"], r["voices"], r["budget"], r["split"])].append(r)
    return g


def built_rows(path):
    g = defaultdict(list)
    for r in lines(path, "scaling: "):
        if r.get("quality") != "ultra" or r.get("instances", "1") != "1":
            continue
        args = r.get("args", "")
        whole = "CA72_SPLIT=0" in args
        who = "one" if r.get("poly") == "false" else f"{r['voices']} POLY"
        if who not in ("one", "2 POLY"):
            continue
        g[(r["block"], who, "whole" if whole else "split")].append(r)
    return g


new, split_before, built_before = sys.argv[1:4]
ns, os_ = split_rows(new), split_rows(split_before)
print("\nPrototype (`ultra_split`): voices, threads, split — before → now\n")
print("| Frames | Voices | Threads | How | Before | Now |\n|---|---|---|---|---|---|")
for b in ("256", "128"):
    for v, t, s in (("1", "4", "0"), ("1", "6", "1"), ("1", "12", "1"), ("2", "12", "0"), ("2", "12", "2")):
        how = "whole" if s == "0" else ("split" if s == v else "some split")
        print(f"| {b} | {v} | {t} | {how} | {cell(os_.get((b, v, t, s)))} | {cell(ns.get((b, v, t, s)))} |")
nb, ob = built_rows(new), built_rows(built_before)
print("\nBuilt engine (the plug-in's own counts) — before → now\n")
print("| Frames | Voices | How | Before | Now |\n|---|---|---|---|---|")
for b in ("256", "128"):
    for who in ("one", "2 POLY"):
        for how in ("split", "whole"):
            print(f"| {b} | {who} | {how} | {cell(ob.get((b, who, how)))} | {cell(nb.get((b, who, how)))} |")
