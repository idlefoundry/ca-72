#!/usr/bin/env python3
"""Signal Basin: the switch's two pictures (off, its lamp dark; on, lit) cut out by their own
transparency, cropped on one frame (the union of their outlines, so the two line up) and sized
for the rocker slots. python3 switches.py <generated folder> <pictures folder>"""
import sys
import numpy as np
from PIL import Image, ImageFilter

src, out = sys.argv[1].rstrip("/") + "/", sys.argv[2].rstrip("/") + "/"
ims = [Image.open(src + n + ".png").convert("RGBA") for n in ("switch", "switch-on")]
boxes = [Image.fromarray((np.array(i)[..., 3] > 128).astype(np.uint8) * 255).getbbox() for i in ims]
box = (min(b[0] for b in boxes), min(b[1] for b in boxes), max(b[2] for b in boxes), max(b[3] for b in boxes))
for im, names in zip(ims, (("rocker-blue", "rocker-ivory"), ("rocker-blue-on", "rocker-ivory-on"))):
    a = Image.fromarray(np.array(im)[..., 3]).filter(ImageFilter.MinFilter(3)).filter(ImageFilter.GaussianBlur(0.6))
    im.putalpha(a)
    pic = im.crop(box).resize((512, 194), Image.LANCZOS)
    for n in names:
        pic.save(out + n + ".png", optimize=True)
print("cut on", box)
