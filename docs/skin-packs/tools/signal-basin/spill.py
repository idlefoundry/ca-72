#!/usr/bin/env python3
"""Signal Basin: a soft amber wash round the lit tab's lamp window, spilt onto its face (the
reviewer). python3 spill.py <pictures folder>"""
import sys
import numpy as np
from PIL import Image

P = sys.argv[1].rstrip("/") + "/"
im = Image.open(P + "tab-lit.png").convert("RGBA")
a = np.array(im).astype(float)
m = (a[..., 0] > 200) & (a[..., 0] > a[..., 1] * 1.3) & (a[..., 3] > 200)
ys, xs = np.nonzero(m)
cx, cy = xs.mean(), ys.mean()
yy, xx = np.mgrid[0:a.shape[0], 0:a.shape[1]]
d = np.hypot((xx - cx) / 1.0, (yy - cy) / 1.0)
wash = np.clip(1 - d / 70.0, 0, 1) ** 2 * 0.16
for c, v in enumerate((255, 150, 40)):
    a[..., c] = a[..., c] * (1 - wash) + v * wash * (a[..., 3] / 255)
Image.fromarray(np.clip(a, 0, 255).astype(np.uint8)).save(P + "tab-lit.png", optimize=True)
print("spilt about", round(cx), round(cy))
