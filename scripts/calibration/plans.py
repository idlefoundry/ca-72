"""Capture plans: each a list of takes for capture.py.

A take is {"name", "seconds", "signals": {output: samples}, "events": {...}, "notes"}.
Signals are fractions of the ES-3's full scale (VOLTS_FS volts); every output is silent for
the first QUIET_LEAD seconds (the loopback's timing mark sits there). "events" records what
each segment is, for the analysis.

session_a: what the gates and control voltages reach with the panel left at its home
settings (oscillators and noise off, EXTERNAL INPUT on at VOLUME 5, CUTOFF fully clockwise,
EMPHASIS and AMOUNT OF CONTOUR at 0, KEYBOARD CONTROL off, both contours with ATTACK 0 and
SUSTAIN 10, the DECAY switches off, main VOLUME 10).
"""

import numpy as np

FS = 48000
GATE = 0.4      # about 4 V
LEAD = 0.5      # first signal, s (after capture.QUIET_LEAD)


def zeros(seconds):
    return np.zeros(int(round(seconds * FS)), np.float32)


def span(t0, t1):
    return slice(int(round(t0 * FS)), int(round(t1 * FS)))


def fade(x, ms=10.0):
    k = min(int(ms * FS / 1000), len(x) // 2)
    if k > 0:
        r = 0.5 - 0.5 * np.cos(np.pi * np.arange(k) / k)
        x[:k] *= r
        x[-k:] *= r[::-1]
    return x


def tone(f, amp, seconds, phase=0.0):
    t = np.arange(int(round(seconds * FS))) / FS
    return fade(amp * np.sin(2 * np.pi * f * t + phase))


def sweep(f1, f2, amp, seconds):
    """An exponential sine sweep (Farina), faded at both ends."""
    t = np.arange(int(round(seconds * FS))) / FS
    k = seconds / np.log(f2 / f1)
    return fade(amp * np.sin(2 * np.pi * f1 * k * (np.exp(t / k) - 1)), 20.0)


def saw(f, amp, seconds, phase=0.0):
    """A band-limited sawtooth (harmonics below 20 kHz), falling ramp, peak about amp."""
    t = np.arange(int(round(seconds * FS))) / FS
    x = np.zeros_like(t)
    for h in range(1, int(20000 / f) + 1):
        x += np.sin(2 * np.pi * h * f * t + h * phase) / h
    return amp * x / 1.8519


def put(dst, t0, x):
    i = int(round(t0 * FS))
    dst[i:i + len(x)] += x[:len(dst) - i]


def level_steps(f, levels, cut, step=0.8, gap=0.3):
    n = len(levels)
    seconds = LEAD + 0.5 + n * (step + gap) + 0.5
    ext, lc, cv = zeros(seconds), zeros(seconds), zeros(seconds)
    lc[span(LEAD, seconds - 0.2)] = GATE
    cv[span(LEAD, seconds - 0.2)] = cut
    segs = []
    for i, a in enumerate(levels):
        t0 = LEAD + 0.5 + i * (step + gap)
        put(ext, t0, tone(f, a, step))
        segs.append([round(t0, 4), round(t0 + step, 4), a])
    return seconds, {"ext": ext, "lc_gate": lc, "cut": cv}, {"tone_hz": f, "cut": cut,
                                                            "segments": segs}


LEVELS = [0.0003, 0.0006, 0.001, 0.002, 0.004, 0.007, 0.01, 0.015, 0.02, 0.03, 0.05, 0.07,
          0.1, 0.15, 0.2, 0.3, 0.5]
CUTS = [0.2, 0.1, 0.0, -0.05, -0.1, -0.15, -0.2, -0.25, -0.3, -0.35, -0.4, -0.45, -0.5]


def sweeps(amp, cuts, dur=3.0, settle=0.4):
    seconds = LEAD + len(cuts) * (dur + settle + 0.2) + 0.3
    ext, lc, cv = zeros(seconds), zeros(seconds), zeros(seconds)
    lc[span(LEAD, seconds - 0.1)] = GATE
    segs = []
    for i, c in enumerate(cuts):
        t0 = LEAD + i * (dur + settle + 0.2)
        cv[span(t0, t0 + dur + settle + 0.2)] = c
        put(ext, t0 + settle, sweep(10.0, 24000.0, amp, dur))
        segs.append([round(t0 + settle, 4), round(t0 + settle + dur, 4), c])
    return seconds, {"ext": ext, "lc_gate": lc, "cut": cv}, {
        "sweep": [10.0, 24000.0, dur], "amp": amp, "segments": segs}


def take(name, seconds, signals, events, notes=""):
    return {"name": name, "seconds": seconds, "signals": signals, "events": events,
            "notes": notes}


def reference():
    s = LEAD + 4.6
    ext, lc, cv = zeros(s), zeros(s), zeros(s)
    lc[span(LEAD, s - 0.1)] = GATE
    put(ext, LEAD + 0.2, tone(1000.0, 0.005, 4.2))
    cv[span(LEAD + 2.3, s - 0.1)] = -0.3
    return [take("ref", s, {"ext": ext, "lc_gate": lc, "cut": cv},
                 {"tone_hz": 1000.0, "amp": 0.005,
                  "segments": [[LEAD + 0.5, LEAD + 2.2, 0.0], [LEAD + 2.6, LEAD + 4.3, -0.3]]},
                 "drift reference: 1 kHz, CUT CV 0 then -0.3 of full scale")]


def riff_signals(amp, seconds=9.0):
    ext, lc, cv = zeros(seconds), zeros(seconds), zeros(seconds)
    notes = [41, 41, 53, 41, 48, 41, 51, 53, 41, 41, 53, 41, 56, 55, 51, 48]
    step = 0.25
    t = LEAD
    events = []
    for rep in range(2):
        for i, m in enumerate(notes):
            f = 440.0 * 2 ** ((m - 69) / 12)
            length = step * (0.8 if i % 4 != 3 else 0.45)
            put(ext, t, fade(saw(f, amp, step), 2.0))
            lc[span(t, t + length)] = GATE
            events.append([round(t, 4), round(length, 4), m])
            t += step
    tt = np.arange(len(cv)) / FS
    sweep_cv = -0.3 + 0.18 * np.sin(2 * np.pi * (tt - LEAD) / 8.0 - np.pi / 2)
    cv[span(LEAD, seconds - 0.2)] = sweep_cv[span(LEAD, seconds - 0.2)].astype(np.float32)
    return seconds, {"ext": ext, "lc_gate": lc, "cut": cv}, {"notes": events, "amp": amp}


def session_a():
    t = []

    s = 3.0
    t.append(take("noise", s, {}, {}, "nothing played: the noise floors"))

    s = 4.0
    lc = zeros(s)
    lc[span(LEAD, 3.5)] = GATE
    t.append(take("gate_noise", s, {"lc_gate": lc}, {"gate": [LEAD, 3.5]},
                  "LC GATE held 3 s, no input: the VCA's thump and its open noise"))

    s = 13.0
    ext = zeros(s)
    put(ext, LEAD + 0.5, sweep(10.0, 24000.0, 0.1, 10.0))
    t.append(take("loop_cal", s, {"ext": ext}, {"sweep": [LEAD + 0.5, 10.0, 0.1]},
                  "the loopback's response (the gate closed: the main output silent)"))

    for name, f, cut in (("ext_1k_levels", 1000.0, 0.0), ("ext_200_levels", 200.0, 0.0),
                         ("ext_200_levels_cut-0.3", 200.0, -0.3)):
        s, sig, ev = level_steps(f, LEVELS, cut)
        t.append(take(name, s, sig, ev, "1 kHz/200 Hz steps into EXT, LC GATE held"))

    s, sig, ev = sweeps(0.005, CUTS)
    t.append(take("filter_sweeps", s, sig, ev,
                  "sweeps at a clean level, CUT CV from +0.2 to -0.5 of full scale"))
    s, sig, ev = sweeps(0.05, [0.0, -0.2, -0.4])
    t.append(take("filter_sweeps_hot", s, sig, ev, "sweeps where the filter overdrives"))

    s = LEAD + 10 * 1.2 + 0.5
    lc = zeros(s)
    for i in range(10):
        lc[span(LEAD + 1.2 * i, LEAD + 1.2 * i + 0.6)] = GATE
    t.append(take("gate_pulses", s, {"lc_gate": lc},
                  {"pulses": [[LEAD + 1.2 * i, 0.6] for i in range(10)]},
                  "ten 0.6 s gates, no input: the thump, averaged"))

    widths = [0.0005, 0.001, 0.002, 0.003, 0.005, 0.01, 0.02, 0.05, 0.1, 0.3]
    s = LEAD + sum(w + 0.5 for w in widths) + 10 * 0.7 + 0.6
    ext, lc = zeros(s), zeros(s)
    put(ext, LEAD - 0.15, tone(1000.0, 0.005, s - LEAD))
    pulses = []
    tt = LEAD + 0.2
    for w in widths:
        lc[span(tt, tt + w)] = GATE
        pulses.append([round(tt, 5), w])
        tt += w + 0.5
    for i in range(10):
        lc[span(tt, tt + 0.3)] = GATE
        pulses.append([round(tt, 5), 0.3])
        tt += 0.7
    t.append(take("gate_widths", s, {"ext": ext, "lc_gate": lc},
                  {"tone_hz": 1000.0, "amp": 0.005, "pulses": pulses},
                  "1 kHz in EXT, LC GATE pulses 0.5 ms to 300 ms, then ten of 300 ms"))

    s = LEAD + 6.0
    ext, lc, fc = zeros(s), zeros(s), zeros(s)
    put(ext, LEAD - 0.15, tone(1000.0, 0.005, s - LEAD))
    lc[span(LEAD, s - 0.1)] = GATE
    for i in range(5):
        fc[span(LEAD + 0.5 + i, LEAD + 1.0 + i)] = GATE
    t.append(take("fc_gate_check", s, {"ext": ext, "lc_gate": lc, "fc_gate": fc},
                  {"pulses": [[LEAD + 0.5 + i, 0.5] for i in range(5)]},
                  "FC GATE pulses with AMOUNT OF CONTOUR at 0: nothing should move"))

    s = LEAD + 13.0
    ext, lc, cv = zeros(s), zeros(s), zeros(s)
    put(ext, LEAD, fade(saw(110.0, 0.01, 12.4), 20.0))
    lc[span(LEAD, s - 0.2)] = GATE
    tt = np.arange(int(round(12.0 * FS))) / FS
    ramp = np.where(tt < 6.0, -0.5 + 0.1 * tt, 0.1 - 0.1 * (tt - 6.0)).astype(np.float32)
    cv[span(LEAD + 0.2, LEAD + 12.2)] = ramp[:len(cv[span(LEAD + 0.2, LEAD + 12.2)])]
    t.append(take("cut_ramp_saw", s, {"ext": ext, "lc_gate": lc, "cut": cv},
                  {"saw_hz": 110.0, "amp": 0.01, "ramp": [LEAD + 0.2, 12.0, -0.5, 0.1]},
                  "a 110 Hz sawtooth into EXT, CUT CV ramped up 6 s and down 6 s"))

    for name, amp in (("riff_clean", 0.01), ("riff_hot", 0.08)):
        s, sig, ev = riff_signals(amp)
        t.append(take(name, s, sig, ev, "a bass riff of sawtooth notes into EXT, gated"))

    s = LEAD + 9.0
    ext, lc, cv = zeros(s), zeros(s), zeros(s)
    rng = np.random.default_rng(72)
    put(ext, LEAD, fade((0.005 * rng.standard_normal(int(8.6 * FS))).astype(np.float32), 20))
    lc[span(LEAD, s - 0.2)] = GATE
    tt = np.arange(int(round(8.0 * FS))) / FS
    cv[span(LEAD + 0.3, LEAD + 8.3)] = (-0.5 + 0.075 * tt).astype(np.float32)
    t.append(take("noise_sweep", s, {"ext": ext, "lc_gate": lc, "cut": cv},
                  {"amp": 0.005, "ramp": [LEAD + 0.3, 8.0, -0.5, 0.1]},
                  "white noise into EXT, CUT CV ramped up over 8 s"))

    t.extend(reference())
    return t


def session_b():
    """Follow-ups to session A, same panel: both contours released together, the
    interface's low-frequency response, and the thump's whole recovery."""
    t = []
    a = [x for x in session_a() if x["name"] == "gate_widths"][0]
    lc = a["signals"]["lc_gate"]
    t.append(take("gate_widths_both", a["seconds"],
                  {"ext": a["signals"]["ext"], "lc_gate": lc, "fc_gate": lc.copy()},
                  a["events"], "as gate_widths, with FC GATE pulsed with LC GATE"))

    s = 9.0
    ext = zeros(s)
    ext[span(1.0, 5.0)] = 0.1
    t.append(take("dc_step", s, {"ext": ext}, {"step": [1.0, 5.0, 0.1]},
                  "a DC step on the loopback (and EXT, the gate closed): the interface's "
                  "low-frequency response"))

    s = LEAD + 5 * 4.0 + 0.5
    lc = zeros(s)
    for i in range(5):
        lc[span(LEAD + 4.0 * i, LEAD + 4.0 * i + 2.0)] = GATE
    t.append(take("gate_pulses_long", s, {"lc_gate": lc},
                  {"pulses": [[LEAD + 4.0 * i, 2.0] for i in range(5)]},
                  "five 2 s gates, no input: the thump's whole recovery"))
    t.extend(reference())
    return t
