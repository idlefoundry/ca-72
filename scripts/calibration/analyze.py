#!/usr/bin/env python3
"""Measures captures and renders the same way (docs/calibration).

    analyze.py TAKE.json [--render RENDER.wav] [--out metrics.json]

A capture's WAV holds the reference's main output, its MIX output and the loopback, then the
stimuli; a render (ca72-lab stim) holds the CA-72's main output, then its mixer bus current
in the MIX's place. Every measure below is of the main output, or of the main output against
the MIX (or bus), so the two compare however their absolute levels differ.
"""

import argparse
import json
import os

import numpy as np
import soundfile as sf

FS = 48000


def load(take_json, render=None):
    with open(take_json) as f:
        meta = json.load(f)
    data, fs = sf.read(os.path.join(os.path.dirname(take_json), meta["wav"]), dtype="float64")
    assert fs == FS
    cols = {c: data[:, i] for i, c in enumerate(meta["columns"])}
    if render:
        r, fs = sf.read(render, dtype="float64")
        assert fs == FS and len(r) >= len(data) - int(meta.get("tail_seconds", 0) * FS)
        n = len(cols["main"])
        pad = lambda x: np.concatenate([x, np.zeros(max(0, n - len(x)))])[:n]
        cols["main"] = pad(r[:, 0])
        cols["mix"] = pad(r[:, 1])
        cols["contour"] = pad(r[:, 3])
    return meta, cols


def seg(x, t0, t1):
    return x[int(round(t0 * FS)):int(round(t1 * FS))]


def harmonics(x, f, n=9):
    """Amplitudes of harmonics 1..n of f (Hann-windowed projection)."""
    w = np.hanning(len(x))
    t = np.arange(len(x)) / FS
    out = []
    for h in range(1, n + 1):
        if h * f >= FS / 2:
            out.append(0.0)
            continue
        out.append(2 * abs(np.sum(x * w * np.exp(-2j * np.pi * h * f * t))) / np.sum(w))
    return np.array(out)


def db(x):
    return 20 * np.log10(np.maximum(np.abs(x), 1e-15))


def levels(meta, c):
    f = meta["events"]["tone_hz"]
    rows = []
    for t0, t1, a in meta["events"]["segments"]:
        a0, a1 = t0 + 0.2, t1 - 0.1
        row = {"level": a}
        for k in ("main", "mix"):
            hm = harmonics(seg(c[k], a0, a1), f)
            row[k] = {"h_db": [round(float(v), 2) for v in db(hm)],
                      "thd_pct": round(float(100 * np.sqrt(np.sum(hm[1:] ** 2)) / hm[0]), 3)}
        if "loop" in c and np.any(c["loop"]):
            row["loop_db"] = round(float(db(harmonics(seg(c["loop"], a0, a1), f, 1)[0])), 2)
        rows.append(row)
    return rows


def band_grid(f0=20.0, f1=20000.0, per_oct=12):
    n = int(np.log2(f1 / f0) * per_oct) + 1
    return f0 * 2 ** (np.arange(n) / per_oct)


def transfer(y, x, grid, per_oct=12):
    """|Y/X| in bands of 1/per_oct octave, dB."""
    Y = np.abs(np.fft.rfft(y)) ** 2
    X = np.abs(np.fft.rfft(x)) ** 2
    f = np.fft.rfftfreq(len(y), 1 / FS)
    half = 2 ** (0.5 / per_oct)
    out = []
    for g in grid:
        m = (f >= g / half) & (f < g * half)
        out.append(10 * np.log10(max(np.sum(Y[m]), 1e-30) / max(np.sum(X[m]), 1e-30)))
    return np.array(out)


def corner(grid, h, ref_band=(60.0, 120.0)):
    """The -3 dB point re the passband (the median over ref_band) and the slope 2..4 times
    above it, dB an octave."""
    m = (grid >= ref_band[0]) & (grid <= ref_band[1])
    ref = float(np.median(h[m]))
    above = np.nonzero((grid > ref_band[1]) & (h < ref - 3.0))[0]
    if len(above) == 0:
        return ref, None, None
    i = above[0]
    g0, g1, h0, h1 = np.log2(grid[i - 1]), np.log2(grid[i]), h[i - 1] - ref, h[i] - ref
    fc = float(2 ** (g0 + (-3.0 - h0) * (g1 - g0) / (h1 - h0)))
    hi = np.interp(np.log2([2 * fc, 4 * fc]), np.log2(grid), h)
    slope = float(hi[1] - hi[0]) if 4 * fc < 16000 else None
    return ref, fc, slope


def sweeps(meta, c):
    grid = band_grid()
    rows = []
    for t0, t1, cut in meta["events"]["segments"]:
        y, x = seg(c["main"], t0, t1 + 0.05), seg(c["mix"], t0, t1 + 0.05)
        h = transfer(y, x, grid)
        ref, fc, slope = corner(grid, h)
        rows.append({"cut": cut, "ref_db": round(ref, 2), "fc_hz": fc and round(fc, 1),
                     "slope_db_oct": slope and round(slope, 1),
                     "h_db": [round(float(v), 2) for v in h]})
    return {"grid_hz": [round(float(g), 2) for g in grid], "rows": rows}


def demod(x, f, periods=1):
    """The amplitude of x's component at f: x times e^-j2pi f t, averaged over whole periods
    of f (which removes the image at 2f exactly)."""
    t = np.arange(len(x)) / FS
    z = x * np.exp(-2j * np.pi * f * t)
    k = int(round(periods * FS / f))
    w = np.ones(k) / k
    return 2 * np.abs(np.convolve(z, w, mode="same"))


def edge_times(env, i0, level, rising=True):
    """Samples after i0 until env first crosses level."""
    e = env[i0:]
    idx = np.nonzero(e >= level)[0] if rising else np.nonzero(e <= level)[0]
    return int(idx[0]) if len(idx) else None


def gates(meta, c):
    f = meta["events"]["tone_hz"]
    env = demod(c["main"], f)
    out = []
    pulses = meta["events"]["pulses"]
    full = []
    for t0, w in pulses:
        if w >= 0.3:
            full.append(np.median(seg(env, t0 + 0.2, t0 + w - 0.01)))
    steady = float(np.median(full))
    floor = float(np.median(seg(env, pulses[0][0] - 0.12, pulses[0][0] - 0.02)))
    for t0, w in pulses:
        i0 = int(round(t0 * FS))
        i1 = int(round((t0 + w) * FS))
        peak = float(np.max(env[i0:i1 + int(0.05 * FS)]))
        row = {"width_ms": round(w * 1e3, 3), "peak_rel_db": round(float(db(peak / steady)), 2)}
        for p in (0.1, 0.5, 0.9):
            s = edge_times(env, i0, floor + p * (steady - floor))
            row[f"rise_{int(p * 100)}_ms"] = s is not None and round(s / FS * 1e3, 3) or None
        if w >= 0.3:
            for d in (6, 20, 40):
                s = edge_times(env, i1, steady * 10 ** (-d / 20), rising=False)
                row[f"fall_{d}db_ms"] = s is not None and round(s / FS * 1e3, 3) or None
        out.append(row)
    return {"steady_db": round(float(db(steady)), 2), "floor_db": round(float(db(floor)), 2),
            "pulses": out}


def thump(meta, c):
    """The main output around the gates' edges, averaged: the VCA's thump."""
    pulses = meta["events"]["pulses"]
    pre, post = int(0.02 * FS), int(0.5 * FS)
    on, off = [], []
    for t0, w in pulses:
        i0, i1 = int(round(t0 * FS)), int(round((t0 + w) * FS))
        on.append(c["main"][i0 - pre:i0 + post])
        off.append(c["main"][i1 - pre:i1 + post])
    res = {}
    for name, a in (("on", np.mean(on, 0)), ("off", np.mean(off, 0))):
        a = a - np.median(a[:pre])
        k = int(np.argmax(np.abs(a)))
        pk = float(a[k])
        tail = np.abs(a[k:])
        below = np.nonzero(tail < abs(pk) / np.e)[0]
        res[name] = {"peak": pk, "peak_ms": round((k - pre) / FS * 1e3, 2),
                     "fall_1e_ms": round(float(below[0]) / FS * 1e3, 2) if len(below) else None,
                     "trace_ds": [round(float(v), 7) for v in a[::48]]}
    return res


def tone_levels(meta, c):
    f = meta["events"]["tone_hz"]
    rows = []
    for t0, t1, cut in meta["events"]["segments"]:
        hm = harmonics(seg(c["main"], t0, t1), f, 3)
        hx = harmonics(seg(c["mix"], t0, t1), f, 1)
        rows.append({"cut": cut, "main_db": round(float(db(hm[0])), 3),
                     "mix_db": round(float(db(hx[0])), 3),
                     "main_over_mix_db": round(float(db(hm[0] / hx[0])), 3)})
    return rows


def noise(meta, c):
    out = {}
    for k in ("main", "mix"):
        x = c[k][int(0.4 * FS):int((meta["seconds"] - 0.1) * FS)]
        out[k + "_dbfs"] = round(float(db(np.sqrt(np.mean(x * x)))), 2)
    return out


def f0_of(x, lo=0.5, hi=8000.0):
    """The fundamental of a periodic x, Hz: the lowest strong peak of its spectrum, then the
    frequency whose first 12 harmonics carry the most energy (a fine search about it)."""
    w = np.hanning(len(x))
    n = 1 << int(np.ceil(np.log2(len(x) * 16)))
    X = np.abs(np.fft.rfft(x * w, n))
    f = np.fft.rfftfreq(n, 1 / FS)
    m = (f >= lo) & (f <= hi)
    peak = X[m].max()
    strong = np.nonzero(m & (X > 0.05 * peak))[0]
    i = strong[0]
    while i + 1 < len(X) and X[i + 1] > X[i]:
        i += 1
    t = np.arange(len(x)) / FS

    def energy(g):
        return sum(abs(np.sum(x * w * np.exp(-2j * np.pi * h * g * t))) ** 2
                   for h in range(1, 13) if h * g < FS / 2)

    a, b = f[i] * 0.995, f[i] * 1.005
    for _ in range(40):
        c1, c2 = a + (b - a) * 0.382, a + (b - a) * 0.618
        if energy(c1) > energy(c2):
            b = c2
        else:
            a = c1
    return 0.5 * (a + b)


def spectrum_of(x, f, n=24):
    """Harmonics 1..n of f: levels in dB re the first, and phases re the first's (each
    harmonic's phase less h times the first's, degrees)."""
    w = np.hanning(len(x))
    t = np.arange(len(x)) / FS
    z = np.array([np.sum(x * w * np.exp(-2j * np.pi * h * f * t)) if h * f < FS / 2 else 0
                  for h in range(1, n + 1)])
    lev = db(np.abs(z) / np.abs(z[0]))
    ph = np.degrees(np.angle(z * np.exp(-1j * np.arange(1, n + 1) * np.angle(z[0]))))
    return lev, ph, 2 * np.abs(z[0]) / np.sum(w)


def osc(meta, c):
    """An oscillator take (plans.session_c): each V/OCT step's pitch, the MIX's level and
    harmonics, and the main output against the MIX."""
    rows = []
    for t0, t1, v in meta["events"]["segments"]:
        mx, mn = seg(c["mix"], t0, t1), seg(c["main"], t0, t1)
        mx, mn = mx - mx.mean(), mn - mn.mean()
        f = f0_of(mx)
        lev, ph, h1 = spectrum_of(mx, f)
        lev_m, _, h1_m = spectrum_of(mn, f)
        rows.append({
            "vpo": v, "hz": round(f, 4),
            "mix_rms_db": round(float(db(np.sqrt(np.mean(mx * mx)))), 2),
            "mix_peak": round(float(np.max(np.abs(mx))), 4),
            "mix_h1_db": round(float(db(h1)), 2),
            "main_rms_db": round(float(db(np.sqrt(np.mean(mn * mn)))), 2),
            "main_over_mix_h1_db": round(float(db(h1_m / h1)), 3),
            "mix_h_db": [round(float(x), 2) for x in lev],
            "mix_h_deg": [round(float(x), 1) for x in ph],
            "main_h_db": [round(float(x), 2) for x in lev_m],
        })
    for a, b in zip(rows, rows[1:]):
        b["octaves_from_prev"] = round(float(np.log2(b["hz"] / a["hz"])), 5)
    return rows


ANALYSES = {
    "noise": noise, "gate_noise": noise,
    "ext_1k_levels": levels, "ext_200_levels": levels, "ext_200_levels_cut-0.3": levels,
    "filter_sweeps": sweeps, "filter_sweeps_hot": sweeps,
    "gate_widths": gates, "gate_pulses": thump,
    "ref": tone_levels,
}


def analyze(take_json, render=None):
    meta, c = load(take_json, render)
    name = meta["take"]
    fn = ANALYSES.get(name) or (osc if name.startswith("osc") else
                                levels if name.endswith("_levels") else None)
    return meta, (fn(meta, c) if fn else None)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("take")
    ap.add_argument("--render")
    ap.add_argument("--out")
    a = ap.parse_args()
    meta, res = analyze(a.take, a.render)
    s = json.dumps({"take": meta["take"], "render": a.render, "result": res}, indent=1)
    if a.out:
        with open(a.out, "w") as f:
            f.write(s)
    else:
        print(s[:4000])


if __name__ == "__main__":
    main()


def release(meta, c, f=1000.0):
    """The 300 ms gates' envelope, averaged: the held level 0.1 s and just before the gate
    falls (dB re the attack's peak), the decay's time constant, and the times after the
    gate falls at which the level crosses -6, -12, -20, -30 and -40 dB re its value then."""
    env = demod(c["main"], f)
    ps = [(t0, w) for t0, w in meta["events"]["pulses"] if w >= 0.3]
    pre, post = int(0.005 * FS), int(0.65 * FS)
    tr = np.mean([env[int(t0 * FS) - pre:int(t0 * FS) + post] for t0, w in ps], 0)
    t = (np.arange(len(tr)) - pre) / FS
    pk = tr.max()
    w = ps[0][1]
    held = np.interp([0.1, w - 0.002], t, tr)
    off = held[1]
    out = {"held_100ms_db": round(float(db(held[0] / pk)), 2),
           "held_end_db": round(float(db(off / pk)), 2)}
    # The decay toward the held level: a time constant from 30 to 200 ms.
    a, b = np.interp([0.03, 0.2], t, tr)
    fin = np.interp(w - 0.002, t, tr)
    if a - fin > 0 and b - fin > 0:
        out["decay_tau_ms"] = round(float(0.17 / np.log((a - fin) / (b - fin)) * 1e3), 1)
    k = np.searchsorted(t, w)
    for d in (6, 12, 20, 30, 40):
        idx = np.nonzero(tr[k:] < off * 10 ** (-d / 20))[0]
        out[f"t{d}_ms"] = round(float(t[k + idx[0]] - w) * 1e3, 2) if len(idx) else None
    return out
