#!/usr/bin/env python3
"""Signal Basin: cuts each generated picture out of its grey ground and sizes it for its slot,
as docs/skin-packs.md lays the slots out. python3 process.py <generated folder> <pack folder>"""
import sys, os, math
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

src, pack = sys.argv[1], sys.argv[2]
only = set(sys.argv[3:])
os.makedirs(os.path.join(pack, "pictures"), exist_ok=True)

def cut(path, thresh=34):
    """The part on a transparent ground: a flood fill from the corners finds the grey."""
    src_im = Image.open(path)
    if src_im.mode in ("RGBA", "LA", "P"):
        rgba = src_im.convert("RGBA")
        al = np.array(rgba)[..., 3]
        if max(al[0, 0], al[-1, -1], al[0, -1], al[-1, 0]) < 8:
            # Generated on a transparent ground: its own alpha, its fringe eased.
            a = Image.fromarray(al).filter(ImageFilter.MinFilter(3)).filter(ImageFilter.GaussianBlur(0.6))
            rgba.putalpha(a)
            return rgba
    im = src_im.convert("RGB")
    w, h = im.size
    marked = im.copy()
    for xy in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]:
        ImageDraw.floodfill(marked, xy, (255, 0, 255), thresh=thresh)
    m = np.array(marked)
    bg = (m[..., 0] == 255) & (m[..., 1] == 0) & (m[..., 2] == 255)
    a = Image.fromarray(np.where(bg, 0, 255).astype(np.uint8))
    a = a.filter(ImageFilter.MinFilter(3)).filter(ImageFilter.GaussianBlur(0.9))
    rgba = im.convert("RGBA")
    rgba.putalpha(a)
    return rgba

def bbox(rgba):
    return Image.fromarray((np.array(rgba)[..., 3] > 128).astype(np.uint8) * 255).getbbox()

def polar(n):
    yy, xx = np.mgrid[0:n, 0:n] + 0.5
    c = n / 2
    return np.hypot(xx - c, yy - c), np.arctan2(yy - c, xx - c)

def even(rgba, r_max, keep=4):
    """Light made the same all round each ring, but for what is finer than `keep` times round
    (the flutes and the line stay): each ring divided by its own light, smoothed round it, and
    multiplied by its mean."""
    a = np.array(rgba).astype(np.float64)
    n = a.shape[0]
    r, t = polar(n)
    lum = 0.3 * a[..., 0] + 0.59 * a[..., 1] + 0.11 * a[..., 2]
    ri = np.floor(r).astype(int)
    bins = 360
    ti = ((t + math.pi) / (2 * math.pi) * bins).astype(int) % bins
    for k in range(0, int(r_max)):
        sel = ri == k
        if sel.sum() < 16:
            continue
        prof = np.zeros(bins); cnt = np.zeros(bins)
        np.add.at(prof, ti[sel], lum[sel]); np.add.at(cnt, ti[sel], 1)
        ok = cnt > 0
        prof[ok] /= cnt[ok]
        if not ok.all():
            idx = np.arange(bins)
            prof = np.interp(idx, idx[ok], prof[ok], period=bins)
        f = np.fft.rfft(prof)
        f[keep + 1:] = 0
        smooth = np.fft.irfft(f, bins)
        mean = prof.mean()
        ratio = np.clip(mean / np.maximum(smooth, 1), 0.6, 1.6)
        a[sel, :3] *= ratio[ti[sel]][:, None]
    return Image.fromarray(np.clip(a, 0, 255).astype(np.uint8))

def flutes(rgba, c, R):
    """The radius where the plain cap ends and the flutes begin, and where they end: the
    rings whose light varies most round them."""
    a = np.array(rgba).astype(np.float64)
    lum = 0.3 * a[..., 0] + 0.59 * a[..., 1] + 0.11 * a[..., 2]
    yy, xx = np.mgrid[0:a.shape[0], 0:a.shape[1]]
    r = np.hypot(xx - c[0], yy - c[1])
    t = np.arctan2(yy - c[1], xx - c[0])
    stds = []
    for k in range(int(R)):
        sel = (r >= k) & (r < k + 1) & (np.abs(t + math.pi / 2) > 0.3)  # leave the line out
        stds.append(lum[sel].std() if sel.sum() > 8 else 0)
    stds = np.array(stds)
    hi = stds > max(6.0, 0.35 * stds[int(0.2 * R):int(0.95 * R)].max())
    ks = np.nonzero(hi[int(0.2 * R):int(0.9 * R)])[0] + int(0.2 * R)
    return ks.min(), ks.max()

def round_part(name, skirt_frac, cap_frac, out_size, pointer=False, keep=4, evened=True):
    rgba = cut(os.path.join(src, name + ".png"))
    x0, y0, x1, y1 = bbox(rgba)
    if pointer:
        # The body: its widest rows (the pointer runs past it above, and may below).
        al = np.array(rgba)[..., 3] > 128
        widths = al.sum(1)
        rows = np.nonzero(widths >= 0.985 * widths.max())[0]
        R = widths.max() / 2                       # the body's radius
        c = ((x0 + x1) / 2, (rows.min() + rows.max()) / 2)
    else:
        R = ((x1 - x0) + (y1 - y0)) / 4
        c = ((x0 + x1) / 2, (y0 + y1) / 2)
    side = 2 * R / skirt_frac
    # Square about the axis.
    s = round(side)
    left, top = round(c[0] - s / 2), round(c[1] - s / 2)
    crop = rgba.crop((left, top, left + s, top + s))
    n = crop.size[0]
    if cap_frac and not pointer:
        # The plain cap grown to the plug-in's unturned disc, the flutes' ring pressed outside it.
        rc, rg = flutes(crop, (n / 2, n / 2), n / 2 * skirt_frac)
        want = cap_frac * n / 2 + 2
        print(f"  {name}: cap {2 * rc / n:.3f} of the picture, flutes to {2 * rg / n:.3f}; the unturned disc {cap_frac:.3f}")
        if rc < want < rg:
            a = np.array(crop).astype(np.float64)
            r, t = polar(n)
            rr = np.where(r < want, r * rc / want,
                          np.where(r < rg, rc + (r - want) * (rg - rc) / (rg - want), r))
            xs = n / 2 + rr * np.cos(t) - 0.5
            ys = n / 2 + rr * np.sin(t) - 0.5
            xi = np.clip(np.round(xs).astype(int), 0, n - 1)
            yi = np.clip(np.round(ys).astype(int), 0, n - 1)
            # bilinear
            x0f = np.clip(np.floor(xs).astype(int), 0, n - 2); y0f = np.clip(np.floor(ys).astype(int), 0, n - 2)
            fx = (xs - x0f)[..., None]; fy = (ys - y0f)[..., None]
            out = (a[y0f, x0f] * (1 - fx) * (1 - fy) + a[y0f, x0f + 1] * fx * (1 - fy)
                   + a[y0f + 1, x0f] * (1 - fx) * fy + a[y0f + 1, x0f + 1] * fx * fy)
            crop = Image.fromarray(np.clip(out, 0, 255).astype(np.uint8))
    if evened:
        crop = even(crop, n / 2 * (0.78 if pointer else skirt_frac), keep=keep)
    crop = crop.resize(out_size, Image.LANCZOS)
    crop.save(os.path.join(pack, "pictures", name + ".png"), optimize=True)

def boxed(name, out_size, thresh=34):
    rgba = cut(os.path.join(src, name + ".png"), thresh)
    rgba = rgba.crop(bbox(rgba)).resize(out_size, Image.LANCZOS)
    rgba.save(os.path.join(pack, "pictures", name + ".png"), optimize=True)

def texture(name, out_size, tile=False):
    im = Image.open(os.path.join(src, name + ".png")).convert("RGB")
    w, h = im.size
    tw, th = out_size
    s = max(tw / w, th / h)
    im = im.resize((math.ceil(w * s), math.ceil(h * s)), Image.LANCZOS)
    l, t = (im.size[0] - tw) // 2, (im.size[1] - th) // 2
    im = im.crop((l, t, l + tw, t + th))
    if tile:
        # Its broad light made even first (each pixel divided by the light about it, 60
        # pixels round, times the mean), or the tiles would show as blotches.
        a = np.array(im).astype(np.float64)
        broad = np.array(im.filter(ImageFilter.GaussianBlur(60))).astype(np.float64)
        a = a / np.maximum(broad, 1) * a.reshape(-1, 3).mean(0)
        im = Image.fromarray(np.clip(a, 0, 255).astype(np.uint8))
        # Seamless: the picture in the middle, its half-turned copy (whose edges meet) at the edges.
        a = np.array(im).astype(np.float64)
        b = np.roll(np.roll(a, th // 2, 0), tw // 2, 1)
        y = np.sin(np.pi * (np.arange(th) + 0.5) / th)[:, None]
        x = np.sin(np.pi * (np.arange(tw) + 0.5) / tw)[None, :]
        wgt = np.clip((y * x) * 1.6, 0, 1)[..., None]
        im = Image.fromarray(np.clip(a * wgt + b * (1 - wgt), 0, 255).astype(np.uint8))
    im.save(os.path.join(pack, "pictures", name + ".png"), optimize=True)

jobs = {
    "knob": lambda: round_part("knob", 0.978, None, (512, 512), evened=False),
    "knob-big": lambda: round_part("knob-big", 0.989, None, (512, 512), evened=False),
    "pointer": lambda: round_part("pointer", 0.78, 0.43, (512, 512), pointer=True, evened=False),
    "rocker-blue": lambda: boxed("rocker-blue", (512, 194)),
    "rocker-orange": lambda: boxed("rocker-orange", (512, 181)),
    "power": lambda: boxed("power", (112, 320)),
    "jewel": lambda: round_part("jewel", 0.98, None, (256, 252)),
    "jack": lambda: round_part("jack", 0.98, None, (256, 251)),
    "screw": lambda: round_part("screw", 0.98, None, (96, 96)),
    "plate": lambda: boxed("plate", (1044, 328)),
    "walnut": lambda: texture("walnut", (1400, 788)),
    "face": lambda: texture("face", (1024, 1024), tile=True),
    # The strip's (STRIP_SLOTS): square caps, widened by the strip; the glass.
    "tab": lambda: boxed("tab", (256, 255)),
    "tab-lit": lambda: boxed("tab-lit", (256, 255)),
    "key": lambda: boxed("key", (256, 255)),
    "glass": lambda: texture("glass", (900, 507)),
}
for name, job in jobs.items():
    if (only and name not in only) or not os.path.exists(os.path.join(src, name + ".png")):
        continue
    job()
    print("made", name)
