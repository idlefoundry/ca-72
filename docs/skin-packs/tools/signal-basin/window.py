#!/usr/bin/env python3
"""Signal Basin: the tabs' lamp window made larger, the same in the unlit and the lit cap (the
reviewer: "too small and flat"): the window and the glow round it scaled about its middle,
moved a little in from the cap's edge, and laid back with a feathered edge.
python3 window.py <pictures folder>"""
import sys
import numpy as np
from PIL import Image, ImageFilter

P = sys.argv[1].rstrip("/") + "/"
lit = np.array(Image.open(P + "tab-lit.png").convert("RGBA")).astype(float)
m = (lit[..., 0] > 150) & (lit[..., 0] > lit[..., 1] * 1.3) & (lit[..., 3] > 200)
ys, xs = np.nonzero(m)
cx, cy = (xs.min() + xs.max()) / 2, (ys.min() + ys.max()) / 2
hw, hh = 23, 26                      # the patch about the window, its glow within it
K = 1.85                             # how much larger
to = (cx - 12, cy)                   # its new middle, a little further in
for name in ("tab", "tab-lit"):
    im = Image.open(P + name + ".png").convert("RGBA")
    patch = im.crop((int(cx - hw), int(cy - hh), int(cx + hw), int(cy + hh)))
    pw, ph = round(patch.size[0] * K), round(patch.size[1] * K)
    patch = patch.resize((pw, ph), Image.LANCZOS)
    # A feathered edge, so that the cap round it shows through where the patch ends.
    mask = Image.new("L", (pw, ph), 0)
    inner = Image.new("L", (pw - 24, ph - 24), 255)
    mask.paste(inner, (12, 12))
    mask = mask.filter(ImageFilter.GaussianBlur(6))
    a = np.array(patch).astype(float)
    a[..., 3] *= np.array(mask).astype(float) / 255
    out = im.copy()
    out.alpha_composite(Image.fromarray(np.clip(a, 0, 255).astype(np.uint8)),
                        (round(to[0] - pw / 2), round(to[1] - ph / 2)))
    out.save(P + name + ".png", optimize=True)
print("windows enlarged about", (cx, cy))
