#!/usr/bin/env python3
"""The knobs' pictures quieted (after `even.py`; the owner, 2026-10-10: "all the knobs look like
they have white halos. and why do the knobs always have 4 shiny places that have nothing to do
with the light?").

The pictures were asked for under a ring light, which left two things the panel's one lamp,
up and to the left, would not light:
- broad reflections round each skirt and grip (four, five, six and more times round):
  `even.py` took out only the lean to one side. Each ring's light (linear, Rec. 709's
  luminance, in sectors of a degree) is taken as it would be all round, and each pixel
  multiplied by its ring's mean over it (at most `LIMIT` times either way), in bands:
  - a skirt's smooth cone and the pointer knob's smooth body, which have no flutes: the light
    smoothed round the ring (`SIGMA_DEG`), so that everything broader than a few degrees goes
    and only the scratches and the dust stay;
  - a fluted grip: the light fitted with its harmonics up to below its flutes' count (nine
    flutes on the knob, thirteen on the big knob, eleven scallops on the pointer knob), so that
    the flutes stay.
  Both drawn together over `SMOOTH` rings of their own band, eased in and out over `EASE`
  pixels. Between a knob's grip and its skirt is left as it is: the grip's edge, a pixel or two
  off the cap's axis, leans as no gain can take out (as `even.py`'s `EDGE`). The marks
  (the white dot, the pointer's wedge and line) are left out of the fits and quieted as their
  surroundings are; the cap (drawn apart, unturned, its sheen the lamp's) is left as it is.
- a bright line round the skirt's outline, lit all round: the halo. Over the outline's band
  each pixel's luminance is held to `RIM_CAP` (its colour kept), the pointer's wedge left out;
  and on the pointer knob, whose scallops' crests shine all round its outline, the crests'
  light over `CREST_KNEE` is pressed down to `CREST_SLOPE` of itself. The panel's lamp lights
  the near side (`art::lamp_over`).

    python3 quiet.py knob.png out.png std
    python3 quiet.py knob-big.png out.png big
    python3 quiet.py pointer.png out.png pointer

Needs only Python 3 and ImageMagick 7 (`magick`). Prints each band's pattern four times round
(its amplitude, a share of its mean) before and after.
"""

import math
import subprocess
import sys

SMOOTH = 1.5
SIGMA_DEG = 4.0
EASE = 4.0
LIMIT = 6.0
RIM_CAP = 0.012  # linear luminance: about 30 of 255
CREST_KNEE = 0.03  # about 48 of 255
CREST_SLOPE = 0.35

# Per picture (pixels from the axis, the pictures 512 across): its bands (from, to, and the
# harmonics fitted, or None: smoothed round), the outline's band, the marks left out of the fits
# ((from, to), degrees either side of twelve o'clock).
KNOBS = {
    "std": {
        "bands": [(150.0, 186.0, 7), (200.0, 238.0, None)],
        "rim": (238.0, 260.0),
        "mark": ((196.0, 232.0), 7.0),
    },
    "big": {
        "bands": [(170.0, 194.0, 11), (204.0, 242.0, None)],
        "rim": (242.0, 260.0),
        "mark": ((205.0, 234.0), 6.0),
    },
    "pointer": {
        "bands": [(112.0, 162.0, None)],
        "rim": (190.0, 210.0),
        "mark": ((95.0, 1e9), 26.0),
        "crests": (160.0, 190.0),
    },
}


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


def solve(a, y):
    """`a` x = `y` by Gaussian elimination with partial pivoting; None if singular."""
    n = len(y)
    m = [row[:] + [y[i]] for i, row in enumerate(a)]
    for k in range(n):
        p = max(range(k, n), key=lambda i: abs(m[i][k]))
        if abs(m[p][k]) < 1e-12:
            return None
        m[k], m[p] = m[p], m[k]
        for i in range(k + 1, n):
            f = m[i][k] / m[k][k]
            for j in range(k, n + 1):
                m[i][j] -= f * m[k][j]
    x = [0.0] * n
    for i in reversed(range(n)):
        x[i] = (m[i][n] - sum(m[i][j] * x[j] for j in range(i + 1, n))) / m[i][i]
    return x


def basis(t, harmonics):
    out = [1.0]
    for k in range(1, harmonics + 1):
        out += [math.cos(k * t), math.sin(k * t)]
    return out


def lum(data, i):
    return 0.2126 * LIN[data[i]] + 0.7152 * LIN[data[i + 1]] + 0.0722 * LIN[data[i + 2]]


def in_mark(r, t, mark):
    (lo, hi), half = mark
    d = abs((math.degrees(t) - 90.0 + 180.0) % 360.0 - 180.0)
    return lo <= r <= hi and d <= half


def sectors(w, h, data, k):
    """Each ring's light in sectors of a degree (sum, count), the marks left out."""
    c = (w - 1) / 2.0
    n = int(math.hypot(c, c)) + 2
    sums = [[0.0] * 360 for _ in range(n)]
    counts = [[0] * 360 for _ in range(n)]
    for y in range(h):
        for x in range(w):
            i = 4 * (y * w + x)
            if data[i + 3] < 250:
                continue
            dx, dy = x - c, c - y
            r = math.hypot(dx, dy)
            if not lo(k) - 2 * SMOOTH <= r <= hi(k) + 2 * SMOOTH:
                continue
            t = math.atan2(dy, dx)
            if in_mark(r, t, k["mark"]):
                continue
            b = int(r)
            s = int(math.degrees(t) % 360.0)
            sums[b][s] += lum(data, i)
            counts[b][s] += 1
    return sums, counts


def lo(k):
    return min(b[0] for b in k["bands"])


def hi(k):
    return max(b[1] for b in k["bands"])


def fits(sums, counts, harmonics, band):
    """Each ring's fit to its `harmonics`th harmonic, the sums drawn together over `SMOOTH`
    rings of its band (none from another band's: a fluted grip's fit does not take the smooth
    skirt's reflections, or the other way about)."""
    n = len(sums)
    p = 1 + 2 * harmonics
    rows = [basis(math.radians(s + 0.5), harmonics) for s in range(360)]
    out = [None] * n
    for b in range(n):
        ata = [[0.0] * p for _ in range(p)]
        aty = [0.0] * p
        whole = 0
        for d in range(-int(2 * SMOOTH), int(2 * SMOOTH) + 1):
            bb = b + d
            if not 0 <= bb < n or not inside(bb, band):
                continue
            g = math.exp(-0.5 * (d / SMOOTH) ** 2)
            for s in range(360):
                if counts[bb][s] == 0:
                    continue
                v = sums[bb][s] / counts[bb][s]
                row = rows[s]
                for i in range(p):
                    aty[i] += g * row[i] * v
                    for j in range(i, p):
                        ata[i][j] += g * row[i] * row[j]
                if d == 0:
                    whole += 1
        if whole < 300:
            continue
        for i in range(p):
            for j in range(i):
                ata[i][j] = ata[j][i]
        out[b] = solve(ata, aty)
    return out


def inside(b, band):
    """Whether ring `b` is in `band` (or within its crossfade)."""
    return band[0] - EASE / 2.0 <= b + 0.5 <= band[1] + EASE / 2.0


def smoothed(sums, counts, band):
    """Each ring's light round it, smoothed (`SIGMA_DEG`, and over `SMOOTH` rings of its band):
    its mean, and each sector's; None where the ring is not whole enough."""
    n = len(sums)
    half = int(3 * SIGMA_DEG)
    wa = [math.exp(-0.5 * (d / SIGMA_DEG) ** 2) for d in range(-half, half + 1)]
    hr = int(2 * SMOOTH)
    wr = [math.exp(-0.5 * (d / SMOOTH) ** 2) for d in range(-hr, hr + 1)]
    out = [None] * n
    for b in range(n):
        if sum(1 for s in range(360) if counts[b][s]) < 300:
            continue
        S = [0.0] * 360
        C = [0.0] * 360
        for d, g in zip(range(-hr, hr + 1), wr):
            bb = b + d
            if not 0 <= bb < n or not inside(bb, band):
                continue
            for s in range(360):
                S[s] += g * sums[bb][s]
                C[s] += g * counts[bb][s]
        mean = sum(S) / max(sum(C), 1e-12)
        ring = []
        for s in range(360):
            num = den = 0.0
            for d, g in zip(range(-half, half + 1), wa):
                t = (s + d) % 360
                num += g * S[t]
                den += g * C[t]
            ring.append(num / den if den > 0 else mean)
        out[b] = (mean, ring)
    return out


def weight(r, band):
    """How much of a band's quieting reaches radius `r` (the bands crossfading)."""
    a, b, _ = band
    up = min(max((r - a) / EASE + 0.5, 0.0), 1.0)
    down = min(max((b - r) / EASE + 0.5, 0.0), 1.0)
    return up * down


def pattern(sums, counts, lo, hi, k=4):
    """The band's pattern `k` times round, a share of its mean."""
    s4 = c4 = mean = 0.0
    m = 0
    for b in range(int(lo), int(hi)):
        for s in range(360):
            if counts[b][s] == 0:
                continue
            v = sums[b][s] / counts[b][s]
            t = math.radians(s + 0.5)
            s4 += v * math.sin(k * t)
            c4 += v * math.cos(k * t)
            mean += v
            m += 1
    if m == 0 or mean == 0:
        return 0.0
    return 2.0 * math.hypot(s4, c4) / mean


def main():
    if len(sys.argv) != 4 or sys.argv[3] not in KNOBS:
        sys.exit(__doc__)
    src, dst, kind = sys.argv[1], sys.argv[2], sys.argv[3]
    k = KNOBS[kind]
    w, h, data = read(src)
    sums, counts = sectors(w, h, data, k)
    for a, b, _ in k["bands"]:
        print(f"{kind} {a:.0f}-{b:.0f}: four times round, before: {pattern(sums, counts, a, b):.3f}")
    lights = []
    for band in k["bands"]:
        lights.append(fits(sums, counts, band[2], band) if band[2] else smoothed(sums, counts, band))
    c = (w - 1) / 2.0
    rim0, rim1 = k["rim"]
    for y in range(h):
        for x in range(w):
            i = 4 * (y * w + x)
            if data[i + 3] == 0:
                continue
            dx, dy = x - c, c - y
            r = math.hypot(dx, dy)
            t = math.atan2(dy, dx)
            L = lum(data, i)
            gain = 1.0
            if rim0 <= r <= rim1:
                if not in_mark(r, t, k["mark"]) and L > RIM_CAP:
                    gain = RIM_CAP / L
            else:
                for band, light in zip(k["bands"], lights):
                    wgt = weight(r, band)
                    ring = light[int(r)] if 0 <= int(r) < len(light) else None
                    if wgt == 0.0 or ring is None:
                        continue
                    if band[2]:
                        if ring[0] <= 0:
                            continue
                        mean, fit = ring[0], sum(a * b for a, b in zip(ring, basis(t, band[2])))
                    else:
                        mean, fit = ring[0], ring[1][int(math.degrees(t) % 360.0)]
                    if fit <= 0:
                        continue
                    g = min(max(mean / fit, 1.0 / LIMIT), LIMIT)
                    gain += wgt * (g - 1.0)
            crests = k.get("crests")
            if crests and crests[0] <= r < crests[1] and not in_mark(r, t, k["mark"]):
                L2 = L * gain
                if L2 > CREST_KNEE:
                    gain *= (CREST_KNEE + (L2 - CREST_KNEE) * CREST_SLOPE) / L2
            if gain != 1.0:
                for ch in range(3):
                    data[i + ch] = linear_to_srgb(LIN[data[i + ch]] * gain)
    write(dst, w, h, data)
    w2, h2, data2 = read(dst)
    s2, c2 = sectors(w2, h2, data2, k)
    for a, b, _ in k["bands"]:
        print(f"{kind} {a:.0f}-{b:.0f}: four times round, after: {pattern(s2, c2, a, b):.3f}")


if __name__ == "__main__":
    main()
