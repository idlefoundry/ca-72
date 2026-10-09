#!/usr/bin/env python3
"""Session HP: the CA-72's renders against the reference's captures (LO and HI;
decisions.md R-HP).

    compare_hp.py CAPTURES RENDERS OUTDIR

RENDERS from render_hp.sh (ca72-lab stim of each take, its FILTER MODE added to its panel).

Each H is the main output against the MIX (capture) or the bus's Norton current (render),
normalised to its own LO passband at EMPHASIS 0 (median |H| 100-300 Hz at CUT CV -3 V), so
that the two compare whatever their units. Per pair: the free-V fit's b, the floor below the
corner, the lows at 30 Hz, the peak above the corner, and |render - capture| in dB (median
and 90th percentile over 30 Hz to 15 kHz, HI's notches excluded: where either is 25 dB
below its passband).
"""
import json, os, sys
import numpy as np
import analyze, hp_mode as H
from plans import HP_CUTS

def curves(cap, render_dir=None):
    grid = analyze.band_grid(20.0, 20000.0, 24)
    out = {}
    for e in H.EMPHASES:
        for mode in ("lo", "hi"):
            j = H.take_json(cap, f"{mode}_e{e:g}")
            r = os.path.join(render_dir, os.path.basename(j)[:-5] + ".wav") if render_dir else None
            m, c = analyze.load(j, r)
            for cut, h in H.segments_h(c, m["events"]["segments"], grid).items():
                out[(e, mode, cut)] = h
    ref = np.median(np.abs(out[(0.0, "lo", -0.3)][(grid >= 100) & (grid <= 300)]))
    return grid, {k: v / ref for k, v in out.items()}

def main():
    cap, ren, outdir = sys.argv[1:4]
    os.makedirs(outdir, exist_ok=True)
    grid, C = curves(cap)
    _, R = curves(cap, ren)
    band = (grid >= 30) & (grid <= 15000)
    rows = []
    for which, D in (("capture", C), ("render", R)):
        pairs = [(e, cut, D[(e, "lo", cut)], D[(e, "hi", cut)]) for e in H.EMPHASES for cut in HP_CUTS]
        res, _ = H.analyse(pairs, grid)
        rows.append({"source": which, "b": res["free_v"]["b"], "a_v_db": res["free_v"]["a_v_db"],
                     "worst_err_db": max(p["err_free_db"] for p in res["pairs"])})
    table = []
    for e in H.EMPHASES:
        for cut in HP_CUTS:
            row = {"emphasis": e, "cut": cut}
            for mode in ("lo", "hi"):
                c, r = analyze.db(C[(e, mode, cut)]), analyze.db(R[(e, mode, cut)])
                m = band & (c > np.max(c[band]) - 25) & (r > np.max(r[band]) - 25)
                d = np.abs(r - c)[m]
                row[f"{mode}_diff_median_db"] = round(float(np.median(d)), 2)
                row[f"{mode}_diff_p90_db"] = round(float(np.percentile(d, 90)), 2)
            for src, D in (("cap", C), ("ren", R)):
                hi = analyze.db(D[(e, "hi", cut)])
                fc = H.corner_lo(grid, D[(0.0, "lo", cut)])
                row[f"{src}_hi_30hz_db"] = round(float(np.interp(30, grid, hi)), 1)
                if fc and fc < 12000:
                    below = (grid >= 40) & (grid <= fc / 2)
                    row[f"{src}_hi_floor_db"] = round(float(np.min(hi[below])), 1) if below.any() else None
                    above = (grid >= fc) & (grid <= min(4 * fc, 18000))
                    row[f"{src}_hi_peak_db"] = round(float(np.max(hi[above])), 1)
            table.append(row)
    json.dump({"fits": rows, "pairs": table}, open(os.path.join(outdir, "compare_hp.json"), "w"), indent=1)
    import matplotlib; matplotlib.use("Agg"); import matplotlib.pyplot as plt
    fig, ax = plt.subplots(2, 4, figsize=(17, 7.6), sharex=True, sharey=True)
    for k, e in enumerate(H.EMPHASES):
        for r_, mode in enumerate(("lo", "hi")):
            a = ax[r_, k]
            for i, cut in enumerate(HP_CUTS):
                a.semilogx(grid, analyze.db(C[(e, mode, cut)]), color=f"C{i}", lw=1.6, label=f"CUT {cut*10:g} V")
                a.semilogx(grid, analyze.db(R[(e, mode, cut)]), color=f"C{i}", lw=1.1, ls="--")
            a.set_title(f"{mode.upper()}, EMPHASIS {e:g}: reference (solid), CA-72 (dashed)", fontsize=8.5)
            a.set_xlim(20, 20000); a.set_ylim(-60, 35); a.grid(True, which="both", alpha=0.3)
    ax[0, 0].legend(fontsize=7); ax[1, 0].set_xlabel("Hz"); ax[0, 0].set_ylabel("dB re LO's passband")
    fig.tight_layout(); fig.savefig(os.path.join(outdir, "compare_hp.png"), dpi=105)
    print(json.dumps(rows, indent=1))
    for row in table: print(row)

if __name__ == "__main__":
    main()
