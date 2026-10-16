#!/usr/bin/env python3
"""Markdown tables for decisions.md R-BLEP from the matrix (`aliasing.tsv`, or `matrix.sh`'s
output): `tables.py <tsv> base|cand|light ..`."""
import sys
from collections import defaultdict

rows = defaultdict(dict)
for line in open(sys.argv[1]):
    f = line.rstrip("\n").split("\t")
    if len(f) < 12 or f[0] == "model":
        continue
    model, kern, os_, rate, wave, key = f[:6]
    f0, total, loud, hz, top, worst = map(float, f[6:12])
    rows[(model, kern, int(os_))][(int(float(rate)), wave, int(float(key)))] = (total, loud, top, worst)

NOTES = [(4, "A4"), (16, "A5"), (28, "A6"), (40, "A7")]
RATES = [(44100, "44.1"), (48000, "48"), (96000, "96")]


def baseline(model, kern, os_, waves, title):
    d = rows[(model, kern, os_)]
    print(f"\n**{title}**\n")
    print("| Waveform | kHz | " + " | ".join(n for _, n in NOTES) + " |")
    print("|---|---:|" + "---:|" * len(NOTES))
    for w in waves:
        for r, rn in RATES:
            cells = []
            for k, _ in NOTES:
                t = d.get((r, w, k))
                cells.append(f"{t[0]:.0f} / {t[1]:.0f}" if t else "")
            print(f"| {w} | {rn} | " + " | ".join(cells) + " |")


CIRCUIT_WAVES = ["saw", "tri", "square", "wide", "narrow"]
LIGHT_WAVES = ["saw", "tri", "shark", "square", "wide", "narrow"]

if "base" in sys.argv[2:]:
    baseline("circuit", "plugin", 1, CIRCUIT_WAVES, "HI (circuit, 1x, two-point polyBLEP)")
    baseline("circuit", "plugin", 4, CIRCUIT_WAVES, "ULTRA (circuit, 4x)")
    baseline("light", "plugin", 1, LIGHT_WAVES, "LO (light model, 1x)")


def candidates(model, cands, waves, title):
    print(f"\n**{title}**\n")
    hdr = []
    for w in waves:
        for _, n in [(4, "A4"), (40, "A7")]:
            hdr.append(f"{w} {n}")
    print("| Candidate | " + " | ".join(hdr) + " | saw A7, 44.1 kHz | top, dB | largest, dB |")
    print("|---|" + "---:|" * (len(hdr) + 3))
    for name, kern, o in cands:
        d = rows.get((model, kern, o))
        if not d:
            continue
        cells = []
        for w in waves:
            for k in (4, 40):
                t = d.get((48000, w, k))
                # (minBLEP cannot carry the triangle: decisions.md R-BLEP.)
                cells.append("not usable" if w == "tri" and kern.startswith("min") else f"{t[0]:.0f}")
        t441 = d.get((44100, "saw", 40))
        top = d.get((48000, "saw", 4))[2]
        # The largest harmonic difference of the saw and rectangles, every rate and note.
        worst = min(
            (v[3] for (r, w, k), v in d.items() if w in ("saw", "square", "wide", "narrow")),
            key=lambda x: -abs(x),
        )
        print(f"| {name} | " + " | ".join(cells) + f" | {t441[0]:.0f} | {top:+.1f} | {worst:+.1f} |")


C = [
    ("polyBLEP, 2 points (HI today)", "plugin", 1),
    ("polyBLEP, 2 points, and BLAMP", "poly2+blamp", 1),
    ("polyBLEP, 4 points, Lagrange, and BLAMP", "lagrange4+blamp", 1),
    ("polyBLEP, 4 points, B-spline, and BLAMP", "bspline4+blamp", 1),
    ("sinc, 8 taps, and BLAMP", "sinc8+blamp", 1),
    ("sinc, 16 taps, and BLAMP", "sinc16+blamp", 1),
    ("sinc, 32 taps, and BLAMP", "sinc32+blamp", 1),
    ("minBLEP, 8 taps, and BLAMP", "min8+blamp", 1),
    ("minBLEP, 16 taps, and BLAMP", "min16+blamp", 1),
    ("minBLEP, 32 taps, and BLAMP", "min32+blamp", 1),
    ("polyBLEP at 2x", "plugin", 2),
    ("polyBLEP at 2x, and BLAMP", "poly2+blamp", 2),
    ("polyBLEP at 4x (ULTRA today)", "plugin", 4),
    ("polyBLEP at 4x, and BLAMP", "poly2+blamp", 4),
    ("sinc, 16 taps, at 4x, and BLAMP", "sinc16+blamp", 4),
    ("sinc, 32 taps, at 4x, and BLAMP", "sinc32+blamp", 4),
]
if "cand" in sys.argv[2:]:
    candidates("circuit", C, ["saw", "tri", "narrow"], "The circuit's oscillator 1 at 48 kHz, total aliasing, dB")

L = [
    ("polyBLEP, 2 points (LO today)", "plugin", 1),
    ("polyBLEP, 2 points (the lab's tables)", "poly2", 1),
    ("polyBLEP, 2 points, and BLAMP", "poly2+blamp", 1),
    ("polyBLEP, 4 points, Lagrange, and BLAMP", "lagrange4+blamp", 1),
    ("polyBLEP, 4 points, B-spline, and BLAMP", "bspline4+blamp", 1),
    ("sinc, 8 taps, and BLAMP", "sinc8+blamp", 1),
    ("sinc, 16 taps, and BLAMP", "sinc16+blamp", 1),
    ("sinc, 32 taps, and BLAMP", "sinc32+blamp", 1),
]
if "light" in sys.argv[2:]:
    candidates("light", L, ["saw", "tri", "narrow"], "The light model's oscillator at 48 kHz, total aliasing, dB")
