#!/usr/bin/env python3
"""Moves the rocker's lamp from its raised half to its flat half (so that, pressed on, the lamp
is at the switch's on end), the raised half filled in from the paddle's shading either side."""
import sys
from PIL import Image, ImageFilter
import numpy as np
path = sys.argv[1]
im = Image.open(path).convert("RGBA"); a = np.array(im).astype(np.float64)
r, g, b = a[..., 0], a[..., 1], a[..., 2]
lamp = (r > 150) & (r > g * 1.25) & (b < 120) & (a[..., 3] > 200)
ys, xs = np.nonzero(lamp)
M = 30
x0, x1 = xs.min() - M, xs.max() + M
y0, y1 = max(ys.min() - M, 0), min(ys.max() + M, a.shape[0] - 1)
W = im.size[0]
fill = a.copy()
t = (np.arange(x0, x1 + 1) - x0 + 1) / (x1 - x0 + 2)
for y in range(y0, y1 + 1):
    fill[y, x0:x1 + 1] = fill[y, x0 - 1] * (1 - t[:, None]) + fill[y, x1 + 1] * t[:, None]
m = np.clip(np.abs(a[..., :3] - fill[..., :3]).sum(-1) / 45, 0, 1)
m = np.array(Image.fromarray((m * 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(1.2))).astype(np.float64) / 255
out = fill.copy()
nx0 = W - 1 - x1
for y in range(y0, y1 + 1):
    k = m[y, x0:x1 + 1, None]
    out[y, nx0:nx0 + (x1 - x0 + 1), :3] = out[y, nx0:nx0 + (x1 - x0 + 1), :3] * (1 - k) + a[y, x0:x1 + 1, :3] * k
Image.fromarray(np.clip(out, 0, 255).astype(np.uint8)).save(path, optimize=True)
