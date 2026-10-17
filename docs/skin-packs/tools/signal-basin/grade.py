#!/usr/bin/env python3
"""Signal Basin: grades the cut pictures towards the concept's colours, after process.py.
python3 grade.py <pack folder> [slot...]"""
import sys, os, colorsys
import numpy as np
from PIL import Image

pack = sys.argv[1]
only = set(sys.argv[2:])
pic = lambda n: os.path.join(pack, "pictures", n + ".png")

def load(n):
    return np.array(Image.open(pic(n)).convert("RGBA")).astype(np.float64)

def save(n, a):
    Image.fromarray(np.clip(a, 0, 255).astype(np.uint8)).save(pic(n), optimize=True)

def lum(a):
    return 0.3 * a[..., 0] + 0.59 * a[..., 1] + 0.11 * a[..., 2]

def hsv_scale(a, s, v):
    rgb = a[..., :3] / 255.0
    mx, mn = rgb.max(-1), rgb.min(-1)
    l = mx[..., None]
    grey = (mx + mn)[..., None] / 2
    out = (grey + (rgb - grey) * s) * v
    a[..., :3] = np.clip(out, 0, 1) * 255
    return a

def walnut():
    # Dark, dull, greyish walnut about #4e3628, its grain kept.
    a = load("walnut")
    L = lum(a); m = L.mean()
    target = np.array([0x4e, 0x36, 0x28], float)
    # Its figure crisper: the light less its blur, a little more of the dark grain.
    from PIL import ImageFilter
    blur = np.array(Image.fromarray(np.clip(L, 0, 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(3))).astype(np.float64)
    L = L + (L - blur) * 0.8
    k = 1 + (L / m - 1) * 0.72
    a[..., :3] = target * k[..., None]
    save("walnut", a)

def knob(n):
    # Satin bakelite cream: the top about #e0d4b4, the highlights pressed down, the edge darker.
    a = load(n)
    al = a[..., 3] > 0
    rgb = a[..., :3]
    L = lum(a)
    top = np.percentile(L[al], 75)
    target = np.array([0xe0, 0xd4, 0xb4], float)
    # The light about the top's kept, above it pressed (a soft knee), then coloured.
    knee = top
    Lp = np.where(L > knee, knee + (L - knee) * 0.35, L)
    ratio = (Lp / np.maximum(L, 1))[..., None]
    rgb = rgb * ratio
    # Its colour towards the target's, keeping each pixel's light.
    hue = target / lum(target[None, None, :])[0, 0]
    rgb = rgb * 0.45 + (lum(rgb[..., :] if False else np.concatenate([rgb, a[..., 3:]], -1))[..., None] * hue) * 0.55
    # Its light overall so that the top sits at the target's.
    Lt = lum(np.concatenate([rgb, a[..., 3:]], -1))
    s = lum(target[None, None, :])[0, 0] / np.percentile(Lt[al], 75)
    a[..., :3] = rgb * s
    save(n, a)

def face():
    # Weathered petrol teal about #172a30, its fine speckle softened, a blotchy patina that
    # tiles (each blotch a whole number of times across the picture).
    from PIL import ImageFilter
    im = Image.open(pic("face")).convert("RGBA")
    a = np.array(im).astype(np.float64)
    soft = np.array(im.filter(ImageFilter.GaussianBlur(2.2))).astype(np.float64)
    a = a * 0.35 + soft * 0.65
    h, w = a.shape[:2]
    a = hsv_scale(a, 0.8, 1.0)
    target = np.array([0x17, 0x2a, 0x30], float)
    m = a[..., :3].reshape(-1, 3).mean(0)
    a[..., :3] *= target / m
    yy, xx = np.mgrid[0:h, 0:w] / np.array([h, w])[:, None, None]
    rng = np.random.default_rng(72)
    mot = np.zeros((h, w))
    for _ in range(40):
        fx, fy = rng.integers(-9, 10, 2)
        if abs(fx) < 3 and abs(fy) < 3:
            continue
        mot += np.sin(2 * np.pi * (fx * xx + fy * yy) + rng.random() * 2 * np.pi) * rng.random()
    mot = mot / np.abs(mot).max()
    a[..., :3] *= (1 + 0.11 * mot)[..., None]
    save("face", a)

def plate():
    # Its face inside the rim made again from its own middle, without the rivets (the plug-in
    # prints its lettering where they were); its copper rim dulled to about #8a5a3a.
    a = load("plate")
    h, w = a.shape[:2]
    x0, y0, x1, y1 = 40, 28, w - 40, h - 28
    mid = a[y0:y1, int(w * 0.3):int(w * 0.7)]
    mw = mid.shape[1]
    tile = np.concatenate([mid, mid[:, ::-1]], 1)
    reps = int(np.ceil((x1 - x0) / tile.shape[1]))
    fill = np.tile(tile, (1, reps, 1))[:, : x1 - x0]
    # Its corners rounded as the rim's inside is.
    yy, xx = np.mgrid[0:y1 - y0, 0:x1 - x0]
    rc = 14
    cx = np.clip(xx, rc, x1 - x0 - 1 - rc); cy = np.clip(yy, rc, y1 - y0 - 1 - rc)
    inside = np.hypot(xx - cx, yy - cy) <= rc
    region = a[y0:y1, x0:x1]
    region[inside] = fill[inside]
    r, b = a[..., 0], a[..., 2]
    cu = (r > 90) & (r > b * 1.4)
    a[cu] = hsv_scale(a[cu][None], 0.7, 0.72)[0]
    save("plate", a)

def screw():
    # Darkened to the drawn screws' smoky bronze (the strip's mountings beside them).
    a = load("screw")
    a = hsv_scale(a, 0.75, 0.62)
    save("screw", a)

jobs = {"walnut": walnut, "knob": lambda: knob("knob"), "knob-big": lambda: knob("knob-big"),
        "pointer": lambda: knob("pointer"), "face": face, "plate": plate, "screw": screw}
for n, j in jobs.items():
    if not only or n in only:
        j(); print("graded", n)
