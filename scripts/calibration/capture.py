#!/usr/bin/env python3
"""Plays a plan's takes into the hardware reference and records it (docs/calibration).

One synchronous stream through the interface: the stimuli leave on the ES-3's outputs
(gates, control voltages, the external input) and the reference's outputs come back on the
interface's inputs, sample-locked. Each take is saved as a float WAV (the recorded channels,
then the stimuli as sent) with a JSON record beside it.

    capture.py PLAN OUTDIR [--only NAME,...] [--repeat-ref MINUTES --hours H]

PLAN names a function in plans.py. Run it where the interface's driver lets the process
record (on macOS, from a terminal that has the microphone permission).
"""

import argparse
import datetime
import hashlib
import json
import os
import sys
import time

import numpy as np

FS = 48000
DEVICE = "828ES"
N_OUT = 24
N_IN = 28

# The interface's computer channels, 1-based (docs/calibration/rig.md).
OUT = {"lc_gate": 9, "fc_gate": 10, "vpo": 11, "cut": 12, "ext": 13, "loop": 14}
IN = {"main": 3, "mix": 5, "loop": 7}
IDLE_IN = (1, 2, 4, 6, 8, 9, 10)

# The ES-3's output at full scale, V (nominal; plans write fractions of full scale).
VOLTS_FS = 10.0
# The reference's 3.5 mm inputs take 0..+5 V; the external input takes audio.
LIMITS = {"lc_gate": (0.0, 0.5), "fc_gate": (0.0, 0.5), "vpo": (-0.5, 0.5),
          "cut": (-0.5, 0.5), "ext": (-0.5, 0.5)}

MARKER_AT = 0.1
QUIET_LEAD = 0.3
# Silence added after each take, s: the stream's latency (about 0.26 s through this
# interface's driver at sounddevice's default) would otherwise cut off its end.
TAIL = 0.6


def marker():
    """A 2 ms burst at 4 kHz, Hann windowed: the loopback's timing mark."""
    n = int(0.002 * FS)
    t = np.arange(n) / FS
    return (0.1 * np.sin(2 * np.pi * 4000 * t) * np.hanning(n)).astype(np.float32)


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def run_take(take, outdir, index, session, sd):
    import soundfile as sf

    n_take = int(round(take["seconds"] * FS))
    n = n_take + int(TAIL * FS)
    out = np.zeros((n, N_OUT), np.float32)
    lead = int(QUIET_LEAD * FS)
    for name, sig in take["signals"].items():
        if name not in OUT or name == "loop":
            raise ValueError(f"unknown output {name}")
        sig = np.asarray(sig, np.float32)
        if len(sig) != n_take:
            raise ValueError(f"{take['name']}: {name} has {len(sig)} samples, not {n_take}")
        if np.any(sig[:lead] != 0):
            raise ValueError(f"{take['name']}: {name} is not silent for its first {QUIET_LEAD} s")
        lo, hi = LIMITS[name]
        if sig.min() < lo - 1e-6 or sig.max() > hi + 1e-6:
            raise ValueError(f"{take['name']}: {name} leaves {lo}..{hi} of full scale")
        out[:n_take, OUT[name] - 1] = sig
    loop = out[:, OUT["ext"] - 1].copy()
    m = marker()
    k = int(MARKER_AT * FS)
    loop[k:k + len(m)] += m
    out[:, OUT["loop"] - 1] = loop
    assert not np.any(out[:, :8]), "the analog outputs stay silent"

    started = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    rec = sd.playrec(out, samplerate=FS, device=DEVICE, channels=N_IN, dtype="float32")
    sd.wait()

    # The timing mark on the loopback gives the stream's latency; the recorded channels are
    # shifted back by it, so that a response sits at its stimulus's time (the reference's
    # own delays remain).
    lp = rec[:int(1.0 * FS), IN["loop"] - 1]
    xc = np.correlate(lp, m, mode="valid")
    lag = int(np.argmax(np.abs(xc))) - k
    mark_db = float(20 * np.log10(np.max(np.abs(xc)) / np.dot(m, m) + 1e-12))
    problems = []
    aligned = 0 < lag < int(0.8 * FS) and abs(mark_db) <= 3
    if aligned:
        rec = np.concatenate([rec[lag:], np.zeros((lag, N_IN), np.float32)])
    else:
        problems.append(f"timing mark not found (lag {lag}, {mark_db:.1f} dB)")

    # Nothing at full scale; the idle inputs quiet over the silent lead (a channel shift
    # would put a signal there).
    touches = {k: int(np.sum(np.abs(rec[:, c - 1]) > 0.9886)) for k, c in IN.items()}
    idle = {}
    for c in IDLE_IN:
        x = rec[int(0.05 * FS):lead, c - 1]
        idle[c] = float(20 * np.log10(np.sqrt(np.mean(x * x)) + 1e-12))
    if any(touches.values()):
        problems.append(f"full scale touched: {touches}")
    loud = {c: v for c, v in idle.items() if v > -95}
    if loud:
        problems.append(f"idle inputs not quiet: {loud}")
    base = f"{index:02d}_{take['name']}"
    cols = ["main", "mix", "loop"] + [s for s in ("ext", "cut", "lc_gate", "fc_gate", "vpo")]
    data = np.zeros((n, len(cols)), np.float32)
    for i, c in enumerate(cols):
        if c in IN:
            data[:, i] = rec[:, IN[c] - 1]
        else:
            data[:, i] = out[:, OUT[c] - 1]
    wav = os.path.join(outdir, base + ".wav")
    sf.write(wav, data, FS, subtype="FLOAT")
    record = {
        "session": session,
        "take": take["name"],
        "index": index,
        "started": started,
        "seconds": take["seconds"],
        "rate": FS,
        "columns": cols,
        "columns_units": "recorded: interface full scale; stimuli: ES-3 full scale",
        "volts_full_scale_nominal": VOLTS_FS,
        "latency_samples": lag,
        "aligned": bool(aligned),
        "tail_seconds": TAIL,
        "timing_mark_db": round(mark_db, 2),
        "idle_inputs_dbfs": {str(c): round(v, 1) for c, v in idle.items()},
        "full_scale_touches": touches,
        "problems": problems,
        "notes": take.get("notes", ""),
        "events": take.get("events", {}),
        "panel": take.get("panel", "home"),
        "wav": os.path.basename(wav),
        "wav_sha256": sha256(wav),
    }
    with open(os.path.join(outdir, base + ".json"), "w") as f:
        json.dump(record, f, indent=1)
    status = "OK" if not problems else "PROBLEM " + "; ".join(problems)
    print(f"{base}: {take['seconds']:.1f} s, latency {lag} samples, {status}", flush=True)
    return not problems


def main():
    import sounddevice as sd

    import plans

    ap = argparse.ArgumentParser()
    ap.add_argument("plan")
    ap.add_argument("outdir")
    ap.add_argument("--only", default="")
    ap.add_argument("--repeat-ref", type=float, default=0.0,
                    help="after the plan, repeat the reference take every this many minutes")
    ap.add_argument("--hours", type=float, default=0.0)
    a = ap.parse_args()

    os.makedirs(a.outdir, exist_ok=True)
    takes = getattr(plans, a.plan)()
    only = [s for s in a.only.split(",") if s]
    session = os.path.basename(os.path.normpath(a.outdir))
    meta = {
        "plan": a.plan,
        "device": DEVICE,
        "rate": FS,
        "outputs": OUT,
        "inputs": IN,
        "capture_py_sha256": sha256(__file__),
        "plans_py_sha256": sha256(plans.__file__),
        "python": sys.version,
    }
    with open(os.path.join(a.outdir, "session.json"), "w") as f:
        json.dump(meta, f, indent=1)
    # (macOS leaves "._" companions on network shares.)
    existing = [p for p in os.listdir(a.outdir) if p.endswith(".wav") and not p.startswith("._")]
    index = len(existing)
    ok = True
    for take in takes:
        if only and take["name"] not in only:
            continue
        ok &= run_take(take, a.outdir, index, session, sd)
        index += 1
        time.sleep(0.5)
    if a.repeat_ref > 0:
        ref = [t for t in plans.reference()]
        end = time.time() + a.hours * 3600
        while time.time() < end:
            time.sleep(a.repeat_ref * 60)
            for take in ref:
                ok &= run_take(take, a.outdir, index, session, sd)
                index += 1
    print("DONE", "all OK" if ok else "with problems", flush=True)


if __name__ == "__main__":
    main()
