#!/usr/bin/env python3
"""The knobs' pictures with the light that leans to one side taken out (the CA-74's
`assets/worn/even.py`, its marks and edges the CA-72's).

The pictures were asked for lit evenly all round (a ring light), so that they can turn
without their light turning; the generated ones still lean: the black grip of both knobs 18 to
30 levels brighter towards twelve o'clock than six. Turned, that side would go round with the
knob, against the panel's fixed lamp.

On a polar grid about the axis (rings a pixel apart, sectors of `STEP` degrees), the
picture's light (linear, Rec. 709's luminance) is smoothed round each ring (`SIGMA_DEG`, wide
enough to pass over the flutes and the scratches) and outwards (`SIGMA_R`); each pixel is then
multiplied by its ring's mean over that smoothed light, at most `LIMIT` times either way. So
each ring's light is the same all round, and everything finer (the flutes, the scratches, the
dust) stays. The white dot on each skirt, and the pointer knob's wedge with its line, are left out of the smoothing and evened as its
surroundings are. The skirt's outline, a few pixels off the cap's axis, is left as it was
(`EDGE`).

    python3 even.py knob.png out.png std
    python3 even.py knob-big.png out.png big
    python3 even.py pointer.png out.png pointer

Needs only Python 3 and ImageMagick 7 (`magick`). Prints each ring's lean (its first harmonic,
as a share of its mean, the fits drawn together over `SMOOTH` pixels) before and after.
"""

import math
import subprocess
import sys

SMOOTH = 4.0
# The marks left out of the fits: (from, to) pixels from the axis, and degrees either side of
# twelve o'clock.
MARKS = {"std": ((240.0, 290.0), 7.0), "big": ((255.0, 300.0), 5.0), "pointer": ((110.0, 1e9), 18.0)}
# Where the evening stops, in pixels from the axis: the skirt's outline, a few pixels off the
# cap's axis (a lean no gain can take out, a quarter of a pixel on the panel at its usual
# size), is left as it was; the gain eases to none over `EASE`.
EDGE = {"std": 296.0, "big": 298.0, "pointer": 1e9}
EASE = 6.0


def srgb_to_linear(v):
    v /= 255.0
    return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4


def linear_to_srgb(v):
    v = min(max(v, 0.0), 1.0)
    v = v * 12.92 if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055
    return round(v * 255.0)


LIN = [srgb_to_linear(float(i)) for i in range(256)]


def read(path):
    size = subprocess.run(
        ["magick", "identify", "-format", "%w %h", path], check=True, capture_output=True, text=True
    ).stdout.split()
    w, h = int(size[0]), int(size[1])
    raw = subprocess.run(
        ["magick", path, "-depth", "8", "rgba:-"], check=True, capture_output=True
    ).stdout
    if len(raw) != w * h * 4:
        sys.exit(f"{path}: {len(raw)} bytes, not {w} by {h} RGBA")
    return w, h, bytearray(raw)


def write(path, w, h, data):
    subprocess.run(
        ["magick", "-size", f"{w}x{h}", "-depth", "8", "rgba:-", path], input=bytes(data), check=True
    )


def solve3(a, y):
    """`a` (3 by 3, symmetric) times x is `y`, by Cramer's rule; None if singular."""

    def det(m):
        return (
            m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
        )

    d = det(a)
    if abs(d) < 1e-12:
        return None
    out = []
    for k in range(3):
        m = [row[:] for row in a]
        for i in range(3):
            m[i][k] = y[i]
        out.append(det(m) / d)
    return out


def leans(w, h, data, mark):
    """Each ring's fit (mean, cos, sin) with its sums drawn together over `SMOOTH`, or None
    where the ring is not whole enough to fit."""
    c = (w - 1) / 2.0
    n = int(math.hypot(c, c)) + 2
    ata = [[[0.0] * 3 for _ in range(3)] for _ in range(n)]
    aty = [[0.0] * 3 for _ in range(n)]
    sectors = [set() for _ in range(n)]
    (lo, hi), half = mark
    for y in range(h):
        for x in range(w):
            i = 4 * (y * w + x)
            if data[i + 3] < 250:
                continue
            dx, dy = x - c, c - y
            r = math.hypot(dx, dy)
            t = math.atan2(dy, dx)
            if lo <= r <= hi and abs(math.degrees(t) - 90.0) <= half:
                continue
            L = 0.2126 * LIN[data[i]] + 0.7152 * LIN[data[i + 1]] + 0.0722 * LIN[data[i + 2]]
            b = int(r)
            v = (1.0, math.cos(t), math.sin(t))
            for p in range(3):
                aty[b][p] += v[p] * L
                for q in range(3):
                    ata[b][p][q] += v[p] * v[q]
            sectors[b].add(int((math.degrees(t) + 180.0) // 10.0))
    k = int(3 * SMOOTH)
    weights = [math.exp(-0.5 * (j / SMOOTH) ** 2) for j in range(-k, k + 1)]
    fits = []
    for b in range(n):
        if len(sectors[b]) < 30:
            fits.append(None)
            continue
        sa = [[0.0] * 3 for _ in range(3)]
        sy = [0.0] * 3
        for j, wt in zip(range(b - k, b + k + 1), weights):
            if 0 <= j < n and len(sectors[j]) >= 30:
                for p in range(3):
                    sy[p] += wt * aty[j][p]
                    for q in range(3):
                        sa[p][q] += wt * ata[j][p][q]
        fits.append(solve3(sa, sy))
    return fits


def report(label, fits):
    rows = []
    for lo in range(0, len(fits), 40):
        ring = [f for f in fits[lo : lo + 40] if f and f[0] > 0]
        if ring:
            mean = sum(f[0] for f in ring) / len(ring)
            swing = sum(math.hypot(f[1], f[2]) for f in ring) / len(ring)
            rows.append(f"{lo}-{lo + 39}: {100 * swing / mean:4.1f} %")
    print(f"  {label}: " + ", ".join(rows))


# The light round a ring is drawn together over this many degrees (a Gaussian's sigma): wide
# enough to pass over the flutes (about 10 degrees apart) and the scratches, narrow enough to
# follow a highlight that lies along one side of a ring; and over this many pixels outwards.
SIGMA_DEG = 20.0
SIGMA_R = 1.5
# The most a pixel's light is multiplied or divided by: a highlight that lies along one side
# of a ring (the small knob's rim) is spread round it, not cut.
LIMIT = 6.0
STEP = 5.0  # degrees a sector


def gaussian(sigma, step):
    k = int(3 * sigma / step)
    return [(j, math.exp(-0.5 * (j * step / sigma) ** 2)) for j in range(-k, k + 1)]


def evened(w, h, data, mark, edge):
    """Each pixel's light divided by its ring's light smoothed round the ring, times the
    ring's mean: the gain on a polar grid (pixels out, sectors round), and the rings beyond
    the knob's whole circle."""
    c = (w - 1) / 2.0
    n = int(math.hypot(c, c)) + 2
    m = int(360 / STEP)
    total = [[0.0] * m for _ in range(n)]
    count = [[0.0] * m for _ in range(n)]
    (lo, hi), half = mark
    for y in range(h):
        for x in range(w):
            i = 4 * (y * w + x)
            if data[i + 3] < 250:
                continue
            dx, dy = x - c, c - y
            r = math.hypot(dx, dy)
            t = math.degrees(math.atan2(dy, dx))
            if lo <= r <= hi and abs(t - 90.0) <= half:
                continue
            L = 0.2126 * LIN[data[i]] + 0.7152 * LIN[data[i + 1]] + 0.0722 * LIN[data[i + 2]]
            b, a = int(r), int((t % 360.0) / STEP) % m
            total[b][a] += L
            count[b][a] += 1.0
    whole = [sum(1 for a in range(m) if count[b][a] > 0) >= 0.5 * m for b in range(n)]
    # Smoothed round each ring (it closes on itself), then outwards over whole rings only.
    ka, kr = gaussian(SIGMA_DEG, STEP), gaussian(SIGMA_R, 1.0)
    def round_ring(rows):
        return [[sum(wt * row[(a + j) % m] for j, wt in ka) for a in range(m)] for row in rows]
    t1, c1 = round_ring(total), round_ring(count)
    t2 = [[0.0] * m for _ in range(n)]
    c2 = [[0.0] * m for _ in range(n)]
    for b in range(n):
        for j, wt in kr:
            if 0 <= b + j < n and whole[b + j]:
                for a in range(m):
                    t2[b][a] += wt * t1[b + j][a]
                    c2[b][a] += wt * c1[b + j][a]
    gain = [[1.0] * m for _ in range(n)]
    last = None
    for b in range(n):
        if whole[b]:
            light = [t2[b][a] / c2[b][a] if c2[b][a] > 0 else 0.0 for a in range(m)]
            mean = sum(light) / m
            if mean > 0:
                gain[b] = [min(max(mean / v, 1 / LIMIT), LIMIT) if v > 0 else 1.0 for v in light]
                last = gain[b]
        elif last is not None:
            gain[b] = last
    first = next((b for b in range(n) if whole[b]), 0)
    for b in range(first):
        gain[b] = gain[first]
    for y in range(h):
        for x in range(w):
            i = 4 * (y * w + x)
            if data[i + 3] == 0:
                continue
            dx, dy = x - c, c - y
            r = min(math.hypot(dx, dy), n - 1.001)
            t = (math.degrees(math.atan2(dy, dx)) % 360.0) / STEP - 0.5
            b, a = int(r), math.floor(t)
            fr, fa = r - b, t - a
            a0, a1 = a % m, (a + 1) % m
            g = (gain[b][a0] * (1 - fa) + gain[b][a1] * fa) * (1 - fr) + (
                gain[b + 1][a0] * (1 - fa) + gain[b + 1][a1] * fa
            ) * fr
            ease = min(max((r - edge) / EASE, 0.0), 1.0)
            g = g * (1 - ease) + ease
            for p in range(3):
                data[i + p] = linear_to_srgb(LIN[data[i + p]] * g)


def main():
    if len(sys.argv) != 4 or sys.argv[3] not in MARKS:
        sys.exit(__doc__)
    src, dst, kind = sys.argv[1:]
    w, h, data = read(src)
    mark = MARKS[kind]
    print(f"{src}: the lean round each ring of 40 pixels, as a share of its mean light")
    report("before", leans(w, h, data, mark))
    evened(w, h, data, mark, EDGE[kind])
    report("after", leans(w, h, data, mark))
    write(dst, w, h, data)


if __name__ == "__main__":
    main()
