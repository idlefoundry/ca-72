#!/usr/bin/env python3
"""Reproduce issue #67's matrix without modifying the sound or opening an audio device.

Create a local venv with aliasing-requirements.txt, then run:
  python scripts/measurements/aliasing.py render --work target/aliasing
  python scripts/measurements/aliasing.py analyse --work target/aliasing \
      --out docs/measurements/aliasing-2026-10
  python scripts/measurements/aliasing.py self-test
"""

import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

# Freeze the comparison; moving branch tips would make a later run a different experiment.
BUILDS = {
    "v0.2.5": "212967e3efe0945c60c44fa4272f51a854d5f3eb",
    "main": "1a1dd6cd2c241f3f2fa94e79ab93cdd489e4a219",
    "fidelity": "9b0f9e8e3bbe3f4a51567475311423f61ece5b55",
}
N = 1 << 17
LOBE = 8.5
ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "crates/ca72-plugin/examples/aliasing_render.rs"


def command(args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render(work):
    work.mkdir(parents=True, exist_ok=True)
    settings = {
        "builds": BUILDS, "renderer_sha256": sha(SOURCE), "samples": N,
        "warm_seconds": 1, "block": 256, "seed": 1,
        "platform": platform.platform(), "rustc": command(["rustc", "--version"]),
        "workers": 0, "sample_format": "mono little-endian f64 (engine output is f32)",
    }
    settings_path = work / "builds.json"
    if settings_path.exists() and json.loads(settings_path.read_text()) != settings:
        raise ValueError("existing run has different settings; choose another --work directory")
    settings_path.write_text(json.dumps(settings, indent=2) + "\n")
    for label, commit in BUILDS.items():
        tree = work / "worktrees" / label
        if not tree.exists():
            tree.parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(["git", "worktree", "add", "--detach", str(tree), commit],
                           cwd=ROOT, check=True)
        if command(["git", "rev-parse", "HEAD"], cwd=tree) != commit:
            raise ValueError(f"wrong commit in {tree}")
        if command(["git", "status", "--porcelain", "--untracked-files=no"], cwd=tree):
            raise ValueError(f"tracked changes in pinned worktree {tree}")
        target_source = tree / "crates/ca72-plugin/examples/aliasing_render.rs"
        target_source.parent.mkdir(exist_ok=True)
        shutil.copyfile(SOURCE, target_source)
        env = dict(os.environ, CARGO_TARGET_DIR=str(tree / "target"), CARGO_BUILD_JOBS="4",
                   IDLE_FOUNDRY_SHARED=str(tree / "target/shared"))
        out = work / "raw" / label
        logdir = work / "logs"
        logdir.mkdir(exist_ok=True)
        if not (out / "renders.tsv").exists():
            with (logdir / f"{label}-build.txt").open("w") as log:
                subprocess.run(["nice", "-n19", "ionice", "-c3", "cargo", "build",
                                "--locked", "--release", "-p", "ca72-plugin", "--example",
                                "aliasing_render"], cwd=tree, env=env, stdout=log,
                               stderr=subprocess.STDOUT, check=True)
            binary = tree / "target/release/examples/aliasing_render"
            with (logdir / f"{label}-render.txt").open("w") as log:
                subprocess.run(["nice", "-n19", "ionice", "-c3", str(binary), str(out)],
                               cwd=tree, env=env, stdout=log, stderr=subprocess.STDOUT,
                               check=True)
        print(f"{label}: {commit}, renders complete", flush=True)


def window(n):
    import numpy as np
    # Same seven-term Blackman-Harris window as fidelity's ca72-lab/src/alias.rs.
    a = [0.27105140069342, 0.43329793923448, 0.21812299954311,
         0.06592544638803, 0.01081174209837, 0.00077658482522, 0.00001388721735]
    t = 2 * np.pi * np.arange(n) / n
    return sum((-1)**k * v * np.cos(k * t) for k, v in enumerate(a))


def frequency(x, rate, nominal):
    import numpy as np
    from scipy.fft import rfft
    n = len(x)
    w = window(n)
    y = (x - np.sum(x*w)/np.sum(w))*w
    p8 = np.abs(rfft(y, n=8*n))**2
    lo, hi = int(nominal*0.97*8*n/rate), int(nominal*1.03*8*n/rate)
    k = lo + np.argmax(p8[lo:hi+1])
    a, b, c = np.log(np.maximum(p8[k-1:k+2], 1e-300))
    return float((k + 0.5*(a-c)/(a-2*b+c))*rate/(8*n))


def spectrum(x, rate, nominal, f0_reset=0):
    import numpy as np
    from scipy.fft import rfft
    from scipy.signal import find_peaks
    if len(x) != N or not np.isfinite(x).all() or np.std(x) < 1e-8:
        raise ValueError("short, non-finite or silent render")
    w = window(N)
    y = (x - np.sum(x * w) / np.sum(w)) * w
    # Interpolate the fundamental on an 8x zero-padded FFT. Octave selection is bounded
    # by the requested MIDI note, not by whichever partial happens to be strongest.
    f0_fft = frequency(x, rate, nominal)
    f0 = f0_reset or f0_fft
    if abs(f0-f0_fft) > rate/N/4:
        raise ValueError(f"reset and spectral frequencies disagree: {f0}, {f0_fft}")
    drift = frequency(x[N//2:], rate, nominal) - frequency(x[:N//2], rate, nominal)
    if abs(drift) > rate/N/4:
        raise ValueError(f"unstable fundamental: {drift} Hz between segment halves")
    p = np.abs(rfft(y))**2
    f = np.arange(len(p)) * rate/N
    band = (f >= 20) & (f <= min(20_000, rate/2))
    h = np.maximum(1, np.rint(f/f0))
    harmonic = np.abs(f-h*f0) <= LOBE*rate/N
    fundamental = np.abs(f-f0) <= LOBE*rate/N
    signal = np.sum(p[band & harmonic])
    fund = np.sum(p[fundamental])
    residual = np.sum(p[band & ~harmonic])
    below10k = np.sum(p[band & (f <= 10_000) & ~harmonic])
    if signal <= 0 or fund <= 0:
        raise ValueError("missing harmonic signal")
    db = lambda power: float(10 * np.log10(max(float(power), 1e-300)))
    peaks, _ = find_peaks(p, distance=18)
    peaks = [int(k) for k in peaks if band[k] and not harmonic[k]]
    peaks.sort(key=lambda k: p[k], reverse=True)
    components = []
    # Identify foldbacks by f_alias = |((m*f0+fs/2) mod fs)-fs/2|, m*f0 > fs/2.
    # A narrow line must also stand 15 dB above its surrounding floor and above -140 dB
    # relative to signal (clear of the synthetic leakage floor). Broadband model hiss is
    # retained in residual, rather than labelled as oscillator aliasing.
    partials = np.arange(int(rate/2/f0)+1, 257)
    folds = np.abs((partials*f0 + rate/2) % rate - rate/2)
    identified = np.zeros(len(p), dtype=bool)
    for k in peaks[:100]:
        bins = np.arange(max(0, k-8), min(len(p), k+9))
        bins = bins[band[bins] & ~harmonic[bins]]
        energy = np.sum(p[bins])
        floor_bins = np.r_[max(1, k-80):max(1, k-20),
                           min(len(p), k+20):min(len(p), k+80)]
        floor_bins = floor_bins[~harmonic[floor_bins]]
        floor = np.median(p[floor_bins]) if len(floor_bins) else 1e-300
        contrast = db(p[k]/max(floor, 1e-300))
        nearest = int(np.argmin(abs(folds-f[k])))
        match = (abs(folds[nearest]-f[k]) < 1.5*rate/N and contrast > 15
                 and db(energy/signal) > -140)
        if match:
            identified[bins] = True
        if len(components) < 8:
            components.append({"hz": float(f[k]), "db_signal": db(energy/signal),
                               "folded_partial": int(partials[nearest]) if match else None,
                               "contrast_db": contrast})
    row = {
        "f0_hz": float(f0), "f0_fft_hz": float(f0_fft),
        "f0_drift_hz": drift,
        "rms": float(np.sqrt(np.mean(x*x))),
        "inharmonic_db_signal": db(residual/signal),
        "inharmonic_db_fundamental": db(residual/fund),
        "inharmonic_below10k_db_signal": db(below10k/signal),
        "identified_alias_db_signal": (db(np.sum(p[identified])/signal)
                                       if np.any(identified) else None),
        "strongest_hz": components[0]["hz"] if components else 0,
        "strongest_db_signal": components[0]["db_signal"] if components else -300,
        "strongest_folded_partial": components[0]["folded_partial"] if components else None,
        "foldback_candidate_in_20hz_20khz": bool(np.any(identified)),
        "components": components,
    }
    return row, (f, p/fund, harmonic)


def analyse(work, out):
    import numpy as np
    os.environ["MPLCONFIGDIR"] = str(work / "matplotlib")
    out.mkdir(parents=True, exist_ok=True)
    settings = json.loads((work / "builds.json").read_text())
    if settings["builds"] != BUILDS or settings["samples"] != N:
        raise ValueError("run belongs to other builds or sample count")
    if settings["renderer_sha256"] != sha(SOURCE):
        raise ValueError("renderer changed since the run")
    rows = []
    spectra = {}
    for build in BUILDS:
        base = work / "raw" / build
        with (base / "renders.tsv").open() as source:
            cases = list(csv.DictReader(source, delimiter="\t"))
        expected = {(stage, mode, str(rate), str(midi), wave)
                    for stage in ["engine", "oscillator"]
                    for mode in (["LO", "HI", "ULTRA"] if stage == "engine" else ["HI", "ULTRA"])
                    for rate in [44_100, 48_000, 96_000] for midi in [84, 96, 108]
                    for wave in ["triangle", "shark", "saw", "square", "wide", "narrow"]}
        actual = {(c["stage"], c["mode"], c["rate"], c["midi"], c["wave"]) for c in cases}
        if len(cases) != len(expected) or actual != expected:
            raise ValueError(f"{build}: incomplete or duplicated render matrix")
        for case in cases:
            if case["file"] != "-".join(case[k] for k in
                                      ["stage", "mode", "rate", "midi", "wave"]) + ".f64":
                raise ValueError("render filename disagrees with its metadata")
            path = base / case["file"]
            if path.stat().st_size != N*8:
                raise ValueError(f"wrong render byte count: {path}")
            x = np.fromfile(path, dtype="<f8")
            rate, midi = int(case["rate"]), int(case["midi"])
            measured, spec = spectrum(x, rate, 440*2**((midi-69)/12),
                                      float(case["f0_reset_hz"]))
            row = dict(build=build, stage=case["stage"], mode=case["mode"], rate=rate,
                       midi=midi, wave=case["wave"], sha256=sha(path), **measured)
            rows.append(row)
            spectra[(build, case["file"])] = spec
        print(f"{build}: {len(cases)} spectra analysed", flush=True)
    (out / "results.json").write_text(json.dumps(rows, indent=2) + "\n")
    fields = [k for k in rows[0] if k != "components"]
    with (out / "results.tsv").open("w") as stream:
        writer = csv.DictWriter(stream, fields, delimiter="\t", extrasaction="ignore",
                                lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    settings["python"] = platform.python_version()
    import scipy
    import matplotlib
    settings["analysis_versions"] = dict(numpy=np.__version__, scipy=scipy.__version__,
                                        matplotlib=matplotlib.__version__)
    settings["executables_sha256"] = {build: sha(work / "worktrees" / build /
        "target/release/examples/aliasing_render") for build in BUILDS}
    settings["cargo_lock_sha256"] = {build: sha(work / "worktrees" / build / "Cargo.lock")
                                    for build in BUILDS}
    settings["analyser_sha256"] = sha(Path(__file__))
    (out / "builds.json").write_text(json.dumps(settings, indent=2) + "\n")
    comparisons = {}
    for stage in ["engine", "oscillator"]:
        baseline = {tuple(r[k] for k in ["mode", "rate", "midi", "wave"]): r["sha256"]
                    for r in rows if r["stage"] == stage and r["build"] == "v0.2.5"}
        main = {tuple(r[k] for k in ["mode", "rate", "midi", "wave"]): r["sha256"]
                for r in rows if r["stage"] == stage and r["build"] == "main"}
        comparisons[stage] = dict(cases=len(baseline), bit_identical=sum(
            baseline[k] == main[k] for k in baseline))
    (out / "comparisons.json").write_text(json.dumps(comparisons, indent=2) + "\n")
    examples(work, out, rows, spectra)


def examples(work, out, rows, spectra):
    import numpy as np
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from scipy.io import wavfile
    # Worst main engine case per mode, worst 96 kHz case and worst HI oscillator:
    # chosen by measured inharmonic energy, never by listening or a preferred waveform.
    selected = []
    for mode in ["LO", "HI", "ULTRA"]:
        selected.append(max((r for r in rows if r["build"] == "main" and
                             r["stage"] == "engine" and r["mode"] == mode),
                            key=lambda r: r["inharmonic_db_signal"]))
    selected.append(max((r for r in rows if r["build"] == "main" and
                         r["stage"] == "engine" and r["rate"] == 96_000),
                        key=lambda r: r["inharmonic_db_signal"]))
    selected.append(max((r for r in rows if r["build"] == "main" and
                         r["stage"] == "oscillator" and r["mode"] == "HI"),
                        key=lambda r: r["inharmonic_db_signal"]))
    selected = list({tuple(r[k] for k in ["stage", "mode", "rate", "midi", "wave"]): r
                     for r in selected}.values())
    listing = []
    for case in selected:
        mode, rate, midi, wave = (case[k] for k in ["mode", "rate", "midi", "wave"])
        stem = f'{case["stage"]}-{mode}-{rate}-{midi}-{wave}'
        raw = {}
        gains = {}
        for build in BUILDS:
            x = np.fromfile(work / "raw" / build / f"{stem}.f64", dtype="<f8")
            # A common RMS target, reduced for all three if any peak would exceed -1 dBFS.
            raw[build] = x
            gains[build] = 10**(-18/20) / np.sqrt(np.mean(x*x))
        reduction = min(1., 10**(-1/20)/max(np.max(abs(raw[b]))*gains[b] for b in BUILDS))
        fig, axes = plt.subplots(3, 1, figsize=(12, 9), sharex=True, sharey=True)
        for ax, build in zip(axes, BUILDS):
            x = raw[build]*gains[build]*reduction
            fade = min(int(0.02*rate), len(x)//2)
            x[:fade] *= np.linspace(0, 1, fade)
            x[-fade:] *= np.linspace(1, 0, fade)
            name = f"{stem}-{build}.wav"
            wavfile.write(out / name, rate, x.astype(np.float32))
            f, p, harmonic = spectra[(build, stem+".f64")]
            band = (f >= 20) & (f <= 20_000)
            level = 10*np.log10(np.maximum(p, 1e-20))
            ax.plot(f[band], level[band], lw=0.6, color="#234e70")
            aliases = band & ~harmonic
            ax.plot(f[aliases], np.where(level[aliases] > -140, level[aliases], np.nan),
                    ".", ms=0.4, color="#c63e36", alpha=0.6)
            matching = next(r for r in rows if all(r[k] == case[k] for k in
                            ["stage", "mode", "rate", "midi", "wave"]) and r["build"] == build)
            for j, component in enumerate(matching["components"][:3]):
                ax.annotate(f'{component["hz"]:.0f} Hz',
                            (component["hz"], level[int(round(component["hz"]*N/rate))]),
                            xytext=(4, 6+12*j), textcoords="offset points", fontsize=7,
                            arrowprops={"arrowstyle": "-", "lw": 0.4})
            ax.set_title(f'{build}: inharmonic/signal {matching["inharmonic_db_signal"]:.1f} dB',
                         loc="left", fontsize=10)
            ax.set_ylim(-140, 5)
            ax.grid(alpha=0.2)
            listing.append(dict(file=name, rms_target_dbfs=-18 + 20*np.log10(reduction),
                                rms_after_fades_dbfs=20*np.log10(np.sqrt(np.mean(x*x))),
                                gain_db=20*np.log10(gains[build]*reduction)))
        fig.suptitle(f'{case["stage"]}: {mode}, {rate} Hz, C{midi//12-1}, {wave}; '
                     'red = outside harmonic bands')
        axes[1].set_ylabel("FFT power / integrated fundamental (dB)")
        axes[-1].set_xlabel("Frequency (Hz)")
        fig.tight_layout()
        fig.savefig(out / f"{stem}.png", dpi=150)
        plt.close(fig)
    (out / "examples.json").write_text(json.dumps(listing, indent=2) + "\n")


def self_test():
    import numpy as np
    rate, f0 = 48_000, 1046.502261
    t = np.arange(N)/rate
    clean = np.sin(2*np.pi*f0*t) + 0.3*np.sin(2*np.pi*3*f0*t)
    m, _ = spectrum(clean, rate, f0)
    assert m["inharmonic_db_signal"] < -145, m
    assert not m["foldback_candidate_in_20hz_20khz"], m
    # The 24th partial folds to 22883.95 Hz (outside the scored band); the 30th to
    # 16604.93 Hz. Inject the latter at -40 dB relative to the intended harmonic signal.
    alias = abs((30*f0+rate/2) % rate-rate/2)
    dirty = clean + np.sqrt(1.09)*0.01*np.sin(2*np.pi*alias*t)
    m, _ = spectrum(dirty, rate, f0)
    assert abs(m["inharmonic_db_signal"] + 40) < 0.02, m
    assert abs(m["identified_alias_db_signal"] + 40) < 0.02, m
    assert m["strongest_folded_partial"] == 30, m
    for broken in [np.zeros(N), np.ones(N-1), np.full(N, np.nan)]:
        try:
            spectrum(broken, rate, f0)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid input accepted")
    print("Self-test passed: harmonic leakage below -145 dB, -40 dB fold recovered, invalid input refused")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    for action in ["render", "analyse"]:
        p = sub.add_parser(action)
        p.add_argument("--work", type=Path, default=ROOT / "target/aliasing")
        if action == "analyse":
            p.add_argument("--out", type=Path, default=ROOT / "docs/measurements/aliasing-2026-10")
    sub.add_parser("self-test")
    args = parser.parse_args()
    if args.action == "render":
        render(args.work.resolve())
    elif args.action == "analyse":
        analyse(args.work.resolve(), args.out.resolve())
    else:
        self_test()


if __name__ == "__main__":
    main()
