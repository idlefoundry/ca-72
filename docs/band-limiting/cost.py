#!/usr/bin/env python3
"""preset_cost for each candidate (`ONLY`: a comma-separated list of their names), interleaved in rounds, the least of each preset's shares
kept (decisions.md R-BLEP), one thread at 48 kHz in 256-frame blocks. Usage: cost.py <hi|lo> <rounds> <out.json>"""
import json, os, re, subprocess, sys

MODE, ROUNDS, OUT = sys.argv[1], int(sys.argv[2]), sys.argv[3]
import glob
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "../..")
# `cargo test --release -p ca72-plugin --test preset_cost --no-run` first.
BIN = max(glob.glob(os.path.join(ROOT, "target/release/deps/preset_cost-*[!.d]")), key=os.path.getmtime)
CWD = os.path.join(ROOT, "crates/ca72-plugin")

CONFIGS = {
    "hi": [
        ("plugin", {}),
        ("poly2+blamp", {"CA72_BLEP": "poly2+blamp"}),
        ("lagrange4+blamp", {"CA72_BLEP": "lagrange4+blamp"}),
        ("bspline4+blamp", {"CA72_BLEP": "bspline4+blamp"}),
        ("sinc8+blamp", {"CA72_BLEP": "sinc8+blamp"}),
        ("sinc16+blamp", {"CA72_BLEP": "sinc16+blamp"}),
        ("sinc32+blamp", {"CA72_BLEP": "sinc32+blamp"}),
        ("min16+blamp", {"CA72_BLEP": "min16+blamp"}),
        ("os2", {"CA72_HI_OSC_OS": "2"}),
        ("os2 poly2+blamp", {"CA72_HI_OSC_OS": "2", "CA72_BLEP": "poly2+blamp"}),
    ],
    "lo": [
        ("plugin", {}),
        ("poly2", {"CA72_BLEP": "poly2"}),
        ("poly2+blamp", {"CA72_BLEP": "poly2+blamp"}),
        ("lagrange4+blamp", {"CA72_BLEP": "lagrange4+blamp"}),
        ("bspline4+blamp", {"CA72_BLEP": "bspline4+blamp"}),
        ("sinc8+blamp", {"CA72_BLEP": "sinc8+blamp"}),
        ("sinc16+blamp", {"CA72_BLEP": "sinc16+blamp"}),
        ("sinc32+blamp", {"CA72_BLEP": "sinc32+blamp"}),
    ],
}[MODE]
if os.environ.get("ONLY"):
    keep = os.environ["ONLY"].split(",")
    CONFIGS = [c for c in CONFIGS if c[0] in keep]

best = {}
for r in range(ROUNDS):
    for name, extra in CONFIGS:
        env = dict(os.environ)
        env["CA72_SECONDS"] = env.get("CA72_SECONDS", "4")
        if MODE == "lo":
            env["CA72_POTATO"] = "1"
        env.update(extra)
        out = subprocess.run([BIN, "--ignored", "--nocapture"], cwd=CWD, env=env,
                             capture_output=True, text=True).stderr
        for line in out.splitlines():
            m = re.match(r"^(.+?) \(POLY 10\)\s+([\d.]+) %\s+(\d+) %", line)
            if m:
                k = (name, m.group(1))
                v = float(m.group(2))
                best[k] = min(best.get(k, 1e9), v)
        print(f"round {r} {name} done", flush=True)
        json.dump({f"{a}\t{b}": v for (a, b), v in best.items()}, open(OUT, "w"), indent=0)
