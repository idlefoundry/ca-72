#!/usr/bin/env python3
"""Render R-DRAWER-LETTERING's LARGE review frames with the panel's PNG example.

Run once with the saved baseline binary, then with the changed one. ImageMagick crops
both at the same drawing coordinates; no image is resized for the comparison.
"""

import argparse
import os
from pathlib import Path
import subprocess


def crop(source, dest, box, scale):
    x0, y0, x1, y1 = (round(v * scale) for v in box)
    subprocess.run(
        ["magick", str(source), "-crop", f"{x1-x0}x{y1-y0}+{x0}+{y0}",
         "+repage", str(dest)], check=True,
    )


def comparisons(output, skin, scale, baseline):
    prefix = f"{skin}-large-{scale}"
    full = output / "full"
    cutoff = 1602 if baseline else 1615
    rows = []
    font = Path(__file__).resolve().parents[1] / "third_party/urw-gothic/URWGothic-Demi.otf"
    for kind, boxes in (
        ("readout", (("global", "GLOBAL: DRIVE", (2738, 1311, 3038, 1439)),
                     ("mpe", "MPE: CUTOFF", (cutoff + 13, 1117, cutoff + 313, 1245)))),
        ("legend", (("global", "GLOBAL: DRIVE", (2541, 1248, 2741, 1288)),
                    ("mpe", "MPE: CUTOFF", (cutoff - 184, 1054, cutoff + 16, 1094)))),
    ):
        tiles = []
        for page, label, box in boxes:
            tile = output / f"{prefix}-{kind}-{page}.png"
            crop(full / f"{prefix}-{page}.png", tile, box, scale)
            subprocess.run(["magick", str(tile), "-background", "#1d1b1a",
                            "-gravity", "north", "-splice", "0x24", "-font", str(font),
                            "-pointsize", "14", "-fill", "#ebe6d8", "-annotate", "+0+3",
                            label, str(tile)], check=True)
            tiles.append(tile)
        row = output / f"{prefix}-{kind}-comparison.png"
        subprocess.run(["magick", *(str(t) for t in tiles), "+append", str(row)], check=True)
        rows.append(row)
    subprocess.run(["magick", *(str(r) for r in rows), "-background", "#1d1b1a",
                    "-gravity", "center", "-append", str(output / f"{prefix}-comparison.png")],
                   check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--baseline", action="store_true", help="the e9545a1 MPE positions")
    parser.add_argument("--comparisons-only", action="store_true")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    full = args.output / "full"
    full.mkdir(exist_ok=True)
    for skin in ("worn", "drawn"):
        for scale in (0.44, 1.0):
            for page, flag in (("global", None), ("mpe", "CA72_MPE"),
                               ("arp", "CA72_ARP"), ("lfo", "CA72_LFO"),
                               ("fx", "CA72_FX"), ("fx-safety", "CA72_FX")):
                if args.comparisons_only:
                    continue
                name = f"{skin}-large-{scale}-{page}.png"
                env = {k: v for k, v in os.environ.items() if not k.startswith("CA72_")}
                env.update(CA72_SKIN=skin, CA72_LETTERING="large", CA72_MPE_P="-100")
                if flag:
                    env[flag] = "safety" if page == "fx-safety" else "1"
                subprocess.run([str(args.binary.resolve()), str(full / name), str(scale)],
                               env=env, check=True)
                box = (360, 1033, 3090, 1775 if page in ("global", "lfo") else 1330)
                crop(full / name, args.output / name, box, scale)
            comparisons(args.output, skin, scale, args.baseline)


if __name__ == "__main__":
    main()
