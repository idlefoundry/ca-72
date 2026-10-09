#!/usr/bin/env python3
"""Session A's figures for the listening page (docs/calibration).

    figures.py CAPTURES_A CAPTURES_B RENDERS OUTDIR

RENDERS holds the CA-72's renders: baseline-07a7dfb/09_filter_sweeps.wav (0.1.3, home panel),
listen-gd/12_gate_widths.{shared,dumpeach}.wav (fitted panel, the gate delayed as the
reference's input delays it), and long_base.wav (0.1.3, B's long gates).
"""

import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

import analyze
import listen

FS = 48000


def main():
    A, B, R, out = sys.argv[1:5]
    os.makedirs(out, exist_ok=True)

    # The filter: absolute, and against its corner.
    _, b = analyze.analyze(f"{A}/09_filter_sweeps.json")
    _, c = analyze.analyze(f"{A}/09_filter_sweeps.json", f"{R}/baseline-07a7dfb/09_filter_sweeps.wav")
    g = np.array(b["grid_hz"])
    fig, ax = plt.subplots(1, 2, figsize=(13, 4.6))
    cuts = (0.0, -0.1, -0.2, -0.3, -0.4, -0.5)
    for rb, rc in zip(b["rows"], c["rows"]):
        if rb["cut"] not in cuts:
            continue
        col = f"C{cuts.index(rb['cut'])}"
        ax[0].semilogx(g, np.array(rb["h_db"]) - rb["ref_db"], color=col, lw=1.6,
                       label=f"{rb['cut'] * 10:+.0f} V")
        ax[0].semilogx(g, np.array(rc["h_db"]) - rc["ref_db"], color=col, lw=1, ls="--")
        if rb["fc_hz"] and rc["fc_hz"] and rb["fc_hz"] < 5000:
            ax[1].semilogx(g / rb["fc_hz"], np.array(rb["h_db"]) - rb["ref_db"], color=col, lw=1.6)
            ax[1].semilogx(g / rc["fc_hz"], np.array(rc["h_db"]) - rc["ref_db"], color=col, lw=1,
                           ls="--")
    ax[0].set(xlim=(50, 20000), ylim=(-45, 3), xlabel="Hz", ylabel="dB re passband",
              title="Filter, EMPHASIS 0, CUT CV: reference (solid), CA-72 0.1.3 (dashed)")
    ax[0].legend(fontsize=8)
    ax[1].set(xlim=(0.1, 8), ylim=(-45, 3), xlabel="frequency / corner",
              title="The same against each corner: the shape")
    for a in ax:
        a.grid(alpha=0.3, which="both")
        a.xaxis.set_minor_formatter(matplotlib.ticker.NullFormatter())
    plt.tight_layout()
    plt.savefig(f"{out}/filter.png", dpi=80)
    plt.close()

    # The release with DECAY off, both contours held.
    m, cb = analyze.load(f"{A}/12_gate_widths.json")
    traces = [("reference", cb["main"])]
    for tag, lab in (("shared", "CA-72 0.1.3"), ("dumpeach", "CA-72, an R1401 for each contour")):
        _, cc = analyze.load(f"{A}/12_gate_widths.json", f"{R}/listen-gd/12_gate_widths.{tag}.wav")
        traces.append((lab, cc["main"]))
    ps = [(t0, w) for t0, w in m["events"]["pulses"] if w >= 0.3]
    fig, ax = plt.subplots(1, 2, figsize=(13, 4.4))
    for lab, y in traces:
        env = analyze.demod(y, 1000.0)
        tr = np.mean([env[int((t0 + w) * FS) - int(0.02 * FS):int((t0 + w) * FS) + int(0.06 * FS)]
                      for t0, w in ps], 0)
        t = (np.arange(len(tr)) - int(0.02 * FS)) / FS * 1e3
        d = 20 * np.log10(tr / tr[int(0.018 * FS)] + 1e-9)
        ax[0].plot(t, d, label=lab)
        k = int(0.02 * FS) + np.nonzero(d[int(0.02 * FS):] < -3)[0][0]
        ax[1].plot(t - t[k], d, label=lab)
    ax[0].axvline(0, c="k", lw=0.5)
    ax[0].set(xlim=(-5, 55), ylim=(-60, 3), xlabel="ms after the gate falls", ylabel="dB",
              title="Release, DECAY off, both contours held (1 kHz; the CA-72's gate delayed 5.9 ms)")
    ax[1].set(xlim=(-10, 35), ylim=(-60, 3), xlabel="ms after each falls 3 dB",
              title="The same, aligned where each has fallen 3 dB: the shape")
    for a in ax:
        a.legend(fontsize=8)
        a.grid(alpha=0.3)
    plt.tight_layout()
    plt.savefig(f"{out}/release.png", dpi=80)
    plt.close()

    # The thump, through the interface's high-pass.
    mb, cb = analyze.load(f"{B}/02_gate_pulses_long.json")
    _, cc = analyze.load(f"{B}/02_gate_pulses_long.json", f"{R}/long_base.wav")
    fig, ax = plt.subplots(figsize=(9, 4.2))
    for lab, y in (("reference", cb["main"]), ("CA-72 0.1.3 (through the interface's high-pass)",
                                                listen.interface_hp(cc["main"]))):
        pre = int(0.05 * FS)
        a = np.mean([y[int(t0 * FS) - pre:int(t0 * FS) + int(3.8 * FS)]
                     for t0, w in mb["events"]["pulses"]], 0)
        a -= np.median(a[:pre - 100])
        t = (np.arange(len(a)) - pre) / FS * 1e3
        ax.plot(t, a / np.max(np.abs(a[:int(0.5 * FS)])), label=lab)
    ax.axvline(0, c="k", lw=0.5)
    ax.axvline(2000, c="k", lw=0.5)
    ax.set(xlabel="ms (gate from 0 to 2000)", title="The VCA's thump, normalised (not changed)")
    ax.legend()
    ax.grid(alpha=0.3)
    plt.tight_layout()
    plt.savefig(f"{out}/thump.png", dpi=80)
    plt.close()
    print("figures in", out)


if __name__ == "__main__":
    main()
