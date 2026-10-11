#!/usr/bin/env python3
"""The reference's FILTER MODE switch from session HP (plans.session_hp; docs/calibration).

    hp_mode.py SESSION_DIR [--out DIR]
    hp_mode.py --self-test CAPTURES_E

The hypothesis: HI is the filter's input less its low-pass output, mixed before the same
output stage, so that for every EMPHASIS and CUT CV

    H_hi(f) = a V(f) - b H_lo(f)

where H is the main output against the MIX (the filter's input), V the direct branch's path
to the main output (the same for every take) and a, b constants. Two fits test it:

- free V: the real b that makes H_hi + b H_lo the same across every pair, its mean a V;
- flat V: V = 1 (the direct branch flat against the MIX), a and b real. (LO with the
  filter open is not flat enough to stand for V: its corner at 0 V is about 19 kHz, and its
  coupling capacitors cut the deep bass.)

Each pair's error is |H_hi - model| against |H_hi| over 30 Hz to 15 kHz, dB. Also measured:
HI's slope two to four octaves below LO's corner (the input-less-low-pass trick predicts 6 dB
an octave, a high-pass ladder 24), the lows' level against HI's passband, the depth of the
cancellation with the filter open, the levels takes (the cancellation as the input
overdrives) and the self-oscillation in each mode.

--self-test builds HI from session E's LO takes with a known a and b, adds noise, and checks
that the fit recovers them.
"""

import argparse
import glob
import json
import os
import sys

import numpy as np

import analyze

FS = 48000
EMPHASES = (0.0, 2.5, 5.0, 7.5)
BAND = (30.0, 15000.0)


def ctransfer(y, x, grid, per_oct=24):
    """Y/X in bands of 1/per_oct octave, complex: the cross-spectrum over the input's
    power, so that a band's phase is kept."""
    Y, X = np.fft.rfft(y), np.fft.rfft(x)
    f = np.fft.rfftfreq(len(y), 1 / FS)
    half = 2 ** (0.5 / per_oct)
    out = np.zeros(len(grid), complex)
    for i, g in enumerate(grid):
        m = (f >= g / half) & (f < g * half)
        out[i] = np.sum(Y[m] * np.conj(X[m])) / max(np.sum(np.abs(X[m]) ** 2), 1e-30)
    return out


def take_json(session, name):
    hits = sorted(glob.glob(os.path.join(session, f"[0-9]*_{name}.json")))
    hits = [h for h in hits if not os.path.basename(h).startswith("._")]
    if not hits:
        raise SystemExit(f"{session}: no take {name}")
    return hits[-1]


def segments_h(cols, segments, grid):
    """Each sweep segment's H (main against MIX), keyed by its CUT CV."""
    out = {}
    for t0, t1, cut in segments:
        y, x = analyze.seg(cols["main"], t0, t1 + 0.05), analyze.seg(cols["mix"], t0, t1 + 0.05)
        out[cut] = ctransfer(y, x, grid)
    return out


def corner_lo(grid, h_lo):
    """LO's -3 dB point against its passband (60 to 120 Hz), Hz."""
    return analyze.corner(grid, analyze.db(h_lo))[1]


def fit_free(lo, hi, m):
    """The real b minimising the spread of H_hi + b H_lo across pairs; their mean."""
    L, H = np.array(lo)[:, m], np.array(hi)[:, m]
    dL, dH = L - L.mean(0), H - H.mean(0)
    b = -float(np.sum(np.real(np.conj(dL) * dH))) / float(np.sum(np.abs(dL) ** 2))
    return b, (np.array(hi) + b * np.array(lo)).mean(0)


def fit_fixed(lo, hi, v, m):
    """Real a and b in H_hi = a V - b H_lo, least squares over every pair and band."""
    A = np.concatenate([np.stack([v[m], -l[m]], 1) for l in lo])
    y = np.concatenate([h[m] for h in hi])
    Ar = np.concatenate([A.real, A.imag])
    yr = np.concatenate([y.real, y.imag])
    (a, b), *_ = np.linalg.lstsq(Ar, yr, rcond=None)
    return float(a), float(b)


def err_db(h, model, m):
    return float(20 * np.log10(np.linalg.norm(h[m] - model[m]) / np.linalg.norm(h[m])))


def describe_hi(grid, h_lo, h_hi):
    """HI's slope 2 to 4 octaves below LO's corner, dB an octave; the lows (40-80 Hz) against
    HI's passband (2 to 4 times LO's corner, below 15 kHz), dB."""
    fc = corner_lo(grid, h_lo)
    d = analyze.db(h_hi)
    lg = np.log2(grid)
    row = {"fc_lo_hz": fc and round(fc, 1)}
    if fc and fc / 16 >= 30:
        lo4, lo2 = np.interp(np.log2([fc / 16, fc / 4]), lg, d)
        row["hi_slope_db_oct"] = round(float((lo2 - lo4) / 2), 2)
    if fc and 2 * fc < 15000:
        pb = (grid >= 2 * fc) & (grid <= min(4 * fc, 15000))
        lows = (grid >= 40) & (grid <= 80)
        row["hi_lows_vs_passband_db"] = round(float(np.median(d[lows]) - np.median(d[pb])), 2)
    return row


def analyse(pairs, grid, v_open=None):
    """pairs: [(emphasis, cut, H_lo, H_hi)]. The two fits and each pair's numbers."""
    m = (grid >= BAND[0]) & (grid <= BAND[1])
    lo, hi = [p[2] for p in pairs], [p[3] for p in pairs]
    b_free, s_mean = fit_free(lo, hi, m)
    res = {"band_hz": list(BAND), "free_v": {"b": round(b_free, 4)}, "pairs": []}
    s_db = analyze.db(s_mean)
    res["free_v"]["a_v_db"] = {"median": round(float(np.median(s_db[m])), 2),
                               "spread_db": round(float(np.ptp(s_db[m])), 2)}
    fixed = None
    if v_open is not None:
        a, b = fit_fixed(lo, hi, v_open, m)
        fixed = (a, b)
        res["flat_v"] = {"a": round(a, 4), "b": round(b, 4)}
    for e, cut, h_lo, h_hi in pairs:
        row = {"emphasis": e, "cut": cut,
               "err_free_db": round(err_db(h_hi, s_mean - b_free * h_lo, m), 2)}
        if fixed:
            row["err_flat_db"] = round(err_db(h_hi, fixed[0] * v_open - fixed[1] * h_lo, m), 2)
        row.update(describe_hi(grid, h_lo, h_hi))
        if cut == 0.0:
            mid = (grid >= 100) & (grid <= 2000)
            row["open_null_db"] = round(float(np.median(analyze.db(h_hi[mid])
                                                        - analyze.db(h_lo[mid]))), 2)
        res["pairs"].append(row)
    return res, s_mean


def levels_rows(session):
    out = {}
    for mode in ("lo", "hi"):
        meta, cols = analyze.load(take_json(session, f"{mode}_e0_levels"))
        out[mode] = analyze.levels(meta, cols)
    rows = []
    for rl, rh in zip(out["lo"], out["hi"]):
        rows.append({"level": rl["level"],
                     "hi_minus_lo_fund_db": round(rh["main"]["h_db"][0] - rl["main"]["h_db"][0], 2),
                     "thd_lo_pct": rl["main"]["thd_pct"], "thd_hi_pct": rh["main"]["thd_pct"]})
    return rows


def selfosc_rows(session):
    out = {}
    for mode in ("lo", "hi"):
        meta, cols = analyze.load(take_json(session, f"{mode}_e10_selfosc"))
        rows = []
        for t0, t1, cut in meta["events"]["segments"]:
            x = analyze.seg(cols["main"], t0, t1)
            f0 = analyze.f0_of(x, 20.0, 23000.0)
            rows.append({"cut": cut, "f0_hz": f0 and round(f0, 1),
                         "rms_dbfs": round(float(analyze.db(np.sqrt(np.mean(x * x)))), 2)})
        out[mode] = rows
    return out


def plot(path, grid, pairs, s_mean, b):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    es = sorted({p[0] for p in pairs})
    fig, ax = plt.subplots(1, len(es), figsize=(4.2 * len(es), 4.2), sharey=True)
    ax = np.atleast_1d(ax)
    for k, e in enumerate(es):
        rows = [p for p in pairs if p[0] == e]
        for i, (_, cut, h_lo, h_hi) in enumerate(rows):
            c = f"C{i}"
            ax[k].semilogx(grid, analyze.db(h_lo), color=c, lw=0.8, alpha=0.5)
            ax[k].semilogx(grid, analyze.db(h_hi), color=c, lw=1.6, label=f"CUT {cut * 10:g} V")
            ax[k].semilogx(grid, analyze.db(s_mean - b * h_lo), color=c, lw=1.0, ls="--")
        ax[k].set_title(f"EMPHASIS {e:g}: HI (solid), LO (faint), model (dashed)", fontsize=9)
        ax[k].set_xlim(20, 20000)
        ax[k].set_ylim(-70, 40)
        ax[k].grid(True, which="both", alpha=0.3)
        ax[k].set_xlabel("Hz")
    ax[0].set_ylabel("main against MIX, dB")
    ax[0].legend(fontsize=7)
    fig.tight_layout()
    fig.savefig(path, dpi=110)


def run(session, outdir):
    grid = analyze.band_grid(20.0, 20000.0, 24)
    pairs = []
    for e in EMPHASES:
        ml, cl = analyze.load(take_json(session, f"lo_e{e:g}"))
        mh, ch = analyze.load(take_json(session, f"hi_e{e:g}"))
        assert ml["events"]["segments"] == mh["events"]["segments"]
        hl = segments_h(cl, ml["events"]["segments"], grid)
        hh = segments_h(ch, mh["events"]["segments"], grid)
        for cut in hl:
            pairs.append((e, cut, hl[cut], hh[cut]))
    res, s_mean = analyse(pairs, grid, np.ones(len(grid), complex))
    ma, ca = analyze.load(take_json(session, "lo_e0_again"))
    again = segments_h(ca, ma["events"]["segments"], grid)
    m = (grid >= BAND[0]) & (grid <= BAND[1])
    first = {p[1]: p[2] for p in pairs if p[0] == 0.0}
    res["lo_e0_repeat_db"] = {str(c): round(err_db(first[c], again[c], m), 2) for c in again}
    res["levels"] = levels_rows(session)
    res["selfosc"] = selfosc_rows(session)
    os.makedirs(outdir, exist_ok=True)
    with open(os.path.join(outdir, "hp_mode.json"), "w") as f:
        json.dump(res, f, indent=1)
    plot(os.path.join(outdir, "hp_mode.png"), grid, pairs, s_mean, res["free_v"]["b"])
    return res


def self_test(captures_e):
    """HI built from session E's LO takes: a known a V - b LO in the time domain (V flat:
    the MIX scaled to LO's passband), noise added at the capture's floor; the fit must
    recover b and explain every pair. Then the negative control: HI as a 24 dB an octave
    high-pass of the MIX that is not made from LO (a Butterworth at each segment's LO
    corner), which the fit must fail to explain."""
    from scipy.signal import butter, sosfilt

    grid = analyze.band_grid(20.0, 20000.0, 24)
    a_true, b_true = 0.83, 1.07
    rng = np.random.default_rng(1)
    pairs, negative = [], []
    g = None
    for name, e in (("emph2.5", 2.5), ("emph5", 5.0), ("emph7.5", 7.5)):
        meta, c = analyze.load(take_json(captures_e, name))
        segs = meta["events"]["segments"]
        h = segments_h(c, segs, grid)
        if g is None:
            # One direct branch for every take, as in the circuit.
            g = float(np.median(np.abs(h[segs[0][2]][(grid >= 60) & (grid <= 120)])))
        hi_main = a_true * g * c["mix"] - b_true * c["main"]
        hi_main = hi_main + rng.normal(0, 10 ** (-85 / 20), len(hi_main))
        hh = segments_h({"main": hi_main, "mix": c["mix"]}, segs, grid)
        hp_main = np.zeros_like(c["mix"])
        for t0, t1, cut in segs:
            fc = corner_lo(grid, h[cut]) or 1000.0
            i0, i1 = int(round((t0 - 0.3) * FS)), int(round((t1 + 0.1) * FS))
            hp_main[i0:i1] = g * sosfilt(butter(4, fc, "highpass", fs=FS, output="sos"),
                                         c["mix"][i0:i1])
        hn = segments_h({"main": hp_main, "mix": c["mix"]}, segs, grid)
        for cut in h:
            pairs.append((e, cut, h[cut], hh[cut]))
            negative.append((e, cut, h[cut], hn[cut]))
    res, _ = analyse(pairs, grid)
    neg, _ = analyse(negative, grid)
    neg_best = min(p["err_free_db"] for p in neg["pairs"])
    b = res["free_v"]["b"]
    worst = max(p["err_free_db"] for p in res["pairs"])
    slopes = [p.get("hi_slope_db_oct") for p in res["pairs"]]
    print(json.dumps({"b_true": b_true, "b_fit": b, "worst_err_db": worst,
                      "a_v_db": res["free_v"]["a_v_db"], "hi_slopes_e2.5": slopes[:5],
                      "negative_best_err_db": neg_best,
                      "negative_slopes_e2.5": [p.get("hi_slope_db_oct")
                                               for p in neg["pairs"]][:5]}, indent=1))
    ok = abs(b - b_true) < 0.01 and worst < -30 and neg_best > -10
    print("SELF-TEST", "OK" if ok else "FAILED")
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("session", nargs="?")
    ap.add_argument("--out", default="")
    ap.add_argument("--self-test", metavar="CAPTURES_E")
    a = ap.parse_args()
    if a.self_test:
        sys.exit(0 if self_test(a.self_test) else 1)
    if not a.session:
        ap.error("a session directory, or --self-test")
    res = run(a.session, a.out or os.path.join(a.session, "analysis"))
    print(json.dumps({k: res[k] for k in ("free_v", "flat_v", "lo_e0_repeat_db")}, indent=1))
    for p in res["pairs"]:
        print(p)


if __name__ == "__main__":
    main()
