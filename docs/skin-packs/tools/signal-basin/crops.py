#!/usr/bin/env python3
"""Writes matching crops of the concept and the render for a reviewer: the whole of each at the
same width, and the same regions of each (left, middle, right), so they can be set side by side."""
import sys
from PIL import Image
concept, render, out = sys.argv[1], sys.argv[2], sys.argv[3]
c = Image.open(concept).convert("RGB")
r = Image.open(render).convert("RGB")
W = 1800
c = c.resize((W, round(c.size[1] * W / c.size[0])), Image.LANCZOS)
r = r.resize((W, round(r.size[1] * W / r.size[0])), Image.LANCZOS)
c.save(f"{out}/concept-whole.jpg", quality=92); r.save(f"{out}/render-whole.jpg", quality=92)
for name, (a, b) in {"left": (0, 0.36), "middle": (0.32, 0.68), "right": (0.64, 1.0)}.items():
    for tag, im in (("concept", c), ("render", r)):
        im.crop((round(a * W), 0, round(b * W), im.size[1])).save(f"{out}/{tag}-{name}.jpg", quality=92)
