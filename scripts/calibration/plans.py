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


# The CA-72's panel for the knob sessions (ca72-lab's keys). Session A's home with both
# SUSTAINs at 10, as the owner set them on 2026-10-08 (loudness from about 6) and
# EXTERNAL INPUT VOLUME 5 as fitted (0.375).
HOME = {
    "osc1_on": False, "osc2_on": False, "osc3_on": False, "noise_on": False,
    "ext_on": True, "ext_volume": 0.375,
    "cutoff": 1.0, "emphasis": 0.0, "contour_amount": 0.0,
    "keyboard_control_1": False, "keyboard_control_2": False,
    "filter_mod": False, "osc_mod": False, "mod_wheel": 0.0, "glide": 0.0, "glide_on": False,
    "filter_attack": 0.0, "filter_sustain": 1.0,
    "loudness_attack": 0.0, "loudness_sustain": 1.0,
    "decay": False, "a440": False, "osc3_control": True,
}


def held_vpo(levels, step=1.2, settle=0.25):
    """LC GATE held while OSC 1V/OCT steps through levels (fractions of full scale), each
    for step s. A segment is [t0, t1, level], settle s after its step."""
    seconds = LEAD + len(levels) * step + 0.4
    lc, vpo = zeros(seconds), zeros(seconds)
    lc[span(LEAD, seconds - 0.1)] = GATE
    segs = []
    for i, v in enumerate(levels):
        t0 = LEAD + i * step
        vpo[span(t0, t0 + step)] = v
        segs.append([round(t0 + settle, 4), round(t0 + step - 0.05, 4), v])
    return seconds, {"lc_gate": lc, "vpo": vpo}, {"segments": segs}


def panel_take(name, panel, set_line, levels=(0.0, 0.1, 0.2), step=1.2, notes=""):
    s, sig, ev = held_vpo(levels, step)
    t = take(name, s, sig, ev, notes)
    t["panel"] = dict(panel)
    t["set"] = set_line
    return t


def session_c():
    """The oscillators, one at a time into the mixer at 8' unless a take says otherwise, the
    filter open, LC GATE held, OSC 1V/OCT stepped 0, +1 and +2 V: each waveform's shape and
    level (MIX beside MAIN), the ranges, the mixer's VOLUME, FREQUENCY's ends and OSC. 3
    CONTROL. The panel moves one knob or switch a take, in this order."""
    t = []
    ref = reference()[0]
    ref["panel"] = dict(HOME)
    ref["set"] = "nothing: the home panel with both SUSTAINs at 10"
    t.append(ref)

    p = dict(HOME, ext_on=False)
    for n in (1, 2, 3):
        p.update({f"osc{n}_range": "8", f"osc{n}_waveform": "triangle", f"osc{n}_volume": 1.0})
        if n > 1:
            p[f"osc{n}_freq"] = 0.5
    p["osc1_on"] = True
    t.append(panel_take(
        "osc1_triangle", p,
        "EXT IN switch OFF; OSC 1 ON, VOLUME 10, RANGE 8', WAVEFORM triangle; OSC 2 and 3 OFF, "
        "each RANGE 8', FREQUENCY 0, WAVEFORM triangle, VOLUME 10; NOISE OFF; OSC 3 CONTROL ON; "
        "OSC MODULATION OFF; FILTER MODULATION OFF; GLIDE OFF; TUNE 0"))

    def step(name, set_line, levels=(0.0, 0.1, 0.2), stp=1.2, **change):
        p.update(change)
        t.append(panel_take(name, p, set_line, levels, stp))

    step("osc1_sharktooth", "OSC 1 WAVEFORM one click clockwise (shark tooth)",
         osc1_waveform="sharktooth")
    step("osc1_sawtooth", "OSC 1 WAVEFORM one more click (sawtooth)",
         levels=(0.0, 0.1, 0.2, 0.3, 0.4), osc1_waveform="sawtooth")
    for r in ("4", "2", "16", "32"):
        step(f"osc1_range{r}", f"OSC 1 RANGE {r}'", osc1_range=r)
    step("osc1_rangelo", "OSC 1 RANGE LO", levels=(0.0, 0.1, 0.2, 0.3), stp=3.0,
         osc1_range="lo")
    step("osc1_vol8", "OSC 1 RANGE back to 8'; OSC 1 VOLUME 8", osc1_range="8",
         osc1_volume=0.8)
    for v in (6, 4, 2, 0):
        step(f"osc1_vol{v}", f"OSC 1 VOLUME {v}", osc1_volume=v / 10)
    step("osc1_square", "OSC 1 VOLUME back to 10; OSC 1 WAVEFORM one click clockwise (square)",
         osc1_volume=1.0, osc1_waveform="square")
    step("osc1_wide", "OSC 1 WAVEFORM one more click (wide rectangle)", osc1_waveform="wide")
    step("osc1_narrow", "OSC 1 WAVEFORM one more click (narrow rectangle)",
         osc1_waveform="narrow")

    step("osc2_triangle", "OSC 1 switch OFF; OSC 2 switch ON", osc1_on=False, osc2_on=True)
    for w, k in (("sharktooth", "shark tooth"), ("sawtooth", "sawtooth"), ("square", "square"),
                 ("wide", "wide rectangle"), ("narrow", "narrow rectangle")):
        step(f"osc2_{w}", f"OSC 2 WAVEFORM one click clockwise ({k})", osc2_waveform=w)
    step("osc2_freq_cw", "OSC 2 FREQUENCY fully clockwise", osc2_freq=1.0)
    step("osc2_freq_ccw", "OSC 2 FREQUENCY fully counterclockwise", osc2_freq=0.0)

    step("osc3_triangle", "OSC 2 FREQUENCY back to 0 and OSC 2 switch OFF; OSC 3 switch ON",
         osc2_freq=0.5, osc2_on=False, osc3_on=True)
    for w, k in (("reverse", "reverse sawtooth"), ("sawtooth", "sawtooth"), ("square", "square"),
                 ("wide", "wide rectangle"), ("narrow", "narrow rectangle")):
        step(f"osc3_{w}", f"OSC 3 WAVEFORM one click clockwise ({k})", osc3_waveform=w)
    step("osc3_freq_cw", "OSC 3 FREQUENCY fully clockwise", osc3_freq=1.0)
    step("osc3_freq_ccw", "OSC 3 FREQUENCY fully counterclockwise", osc3_freq=0.0)
    step("osc3_control_off", "OSC 3 FREQUENCY back to 0; OSC 3 CONTROL OFF", osc3_freq=0.5,
         osc3_control=False)
    return t


def session_d():
    """The mixer's overdrive: sawtooths at 8', FREQUENCY 0, the filter open. Oscillator 1 at
    VOLUME 10 (session C has it alone at 2 to 10), oscillator 2 added at VOLUME 2 to 10,
    then oscillator 3 at 2 to 10. LC GATE held, OSC 1V/OCT at 0 and +1 V for 2 s each."""
    t = []
    p = dict(HOME, ext_on=False)
    for n in (1, 2, 3):
        p.update({f"osc{n}_range": "8", f"osc{n}_waveform": "sawtooth", f"osc{n}_volume": 1.0,
                  f"osc{n}_on": n < 3})
        if n > 1:
            p[f"osc{n}_freq"] = 0.5
    lv = (0.0, 0.1)

    def step(name, set_line, **change):
        p.update(change)
        t.append(panel_take(name, p, set_line, lv, 2.0))

    step("osc12_vol2",
         "OSC 3 FREQUENCY back to 0 and OSC 3 switch OFF; OSC 1 and OSC 2 WAVEFORM to sawtooth; "
         "OSC 1 switch ON (VOLUME 10); OSC 2 switch ON at VOLUME 2", osc2_volume=0.2)
    for v in (4, 6, 8, 10):
        step(f"osc12_vol{v}", f"OSC 2 VOLUME {v}", osc2_volume=v / 10)
    step("osc123_vol2", "OSC 3 WAVEFORM to sawtooth; OSC 3 switch ON at VOLUME 2",
         osc3_on=True, osc3_volume=0.2)
    for v in (4, 6, 8, 10):
        step(f"osc123_vol{v}", f"OSC 3 VOLUME {v}", osc3_volume=v / 10)
    return t


def sweep_take(name, panel, set_line, cuts, vpos=None, amp=0.005, midi=None, notes=""):
    """Session A's sweeps into EXT at each CUT CV (and OSC 1V/OCT, if given, held through
    each sweep), with the panel and the MIDI note last sent to the reference (its keyboard's
    pitch; ca72-lab stim --key)."""
    s, sig, ev = sweeps(amp, cuts)
    if vpos is not None:
        vpo = zeros(s)
        for (t0, t1, _), v in zip(ev["segments"], vpos):
            vpo[span(t0 - 0.4, t1 + 0.2)] = v
        sig["vpo"] = vpo
        ev["vpo"] = list(vpos)
    t = take(name, s, sig, ev, notes)
    t["panel"], t["set"] = dict(panel), set_line
    if midi is not None:
        t["midi_note"] = midi
    return t


def cutoff_knob(mark):
    """CUTOFF's printed mark (-5..+5) as the CA-72's knob (0..1)."""
    return (mark + 5) / 10


def session_e():
    """The filter: EMPHASIS 2.5, 5 and 7.5 (sweeps at a clean level, CUT CV -1 to -5 V),
    EMPHASIS 10 self-oscillating against CUT CV, CUTOFF at its printed marks (CUT CV chosen
    to keep the corner between about 300 Hz and 5 kHz), and KEYBOARD CONTROL 1, 2 and both
    at MIDI notes 41 and 65 (two octaves) with CUTOFF at 0."""
    t = []
    ref = reference()[0]
    ref["panel"] = dict(HOME)
    ref["set"] = "OSC 1, 2 and 3 switches OFF; EXT IN switch ON (EXT VOLUME stays at 5)"
    t.append(ref)
    p = dict(HOME)
    cuts = [-0.1, -0.2, -0.3, -0.4, -0.5]
    for e, line in ((2.5, "FILTER EMPHASIS 2.5 (halfway between 2 and 3)"),
                    (5.0, "FILTER EMPHASIS 5"), (7.5, "FILTER EMPHASIS 7.5")):
        p["emphasis"] = e / 10
        t.append(sweep_take(f"emph{e:g}", p, line, cuts))

    p["emphasis"] = 1.0
    steps = [round(-0.05 * i, 2) for i in range(11)]
    s = LEAD + len(steps) * 0.8 + 0.4
    lc, cv = zeros(s), zeros(s)
    lc[span(LEAD, s - 0.1)] = GATE
    segs = []
    for i, c in enumerate(steps):
        t0 = LEAD + i * 0.8
        cv[span(t0, t0 + 0.8)] = c
        segs.append([round(t0 + 0.25, 4), round(t0 + 0.75, 4), c])
    x = take("emph10_selfosc", s, {"lc_gate": lc, "cut": cv}, {"segments": segs},
             "no input, CUT CV stepped 0 to -5 V: the self-oscillation's pitch and level")
    x["panel"], x["set"] = dict(p), "FILTER EMPHASIS 10"
    t.append(x)

    p["emphasis"] = 0.0
    first = True
    for mark, line in ((4, "CUTOFF FREQUENCY +4"), (2, "CUTOFF FREQUENCY +2"),
                       (0, "CUTOFF FREQUENCY 0"), (-2, "CUTOFF FREQUENCY -2"),
                       (-4, "CUTOFF FREQUENCY -4"), (-5, "CUTOFF FREQUENCY fully counterclockwise")):
        p["cutoff"] = cutoff_knob(mark)
        cs = sorted({round(min(0.5, max(-0.5, (d - mark) / 10)), 2) for d in (2, 0, -1)},
                    reverse=True)
        if first:
            line = "FILTER EMPHASIS back to 0; " + line
            first = False
        t.append(sweep_take(f"cutoff{mark:+d}", p, line, cs))

    p["cutoff"] = cutoff_knob(0)
    for name, line, k1, k2, note, vpos in (
            ("kbd1_n41", "CUTOFF FREQUENCY back to 0; KEYBOARD CONTROL 1 ON", True, False, 41,
             None),
            ("kbd1_n65", "nothing (I send MIDI note 65)", True, False, 65, None),
            ("kbd12_n65", "KEYBOARD CONTROL 2 ON as well", True, True, 65, None),
            ("kbd12_n41", "nothing (I send MIDI note 41)", True, True, 41, [0.0, 0.2]),
            ("kbd2_n41", "KEYBOARD CONTROL 1 OFF (2 stays ON)", False, True, 41, None),
            ("kbd2_n65", "nothing (I send MIDI note 65)", False, True, 65, None)):
        p["keyboard_control_1"], p["keyboard_control_2"] = k1, k2
        t.append(sweep_take(name, p, line, [0.0] * (len(vpos) if vpos else 1), vpos,
                            midi=note))
    return t


def session_h():
    """EXTERNAL INPUT VOLUME at 2, 4, 6, 8 and 10, then back at 5: session A's 1 kHz steps
    (LEVELS) at each, the filter open, LC GATE held. MIX gives the pot's taper with nothing
    after it; the top steps the mixer's and the filter's overdrive."""
    t = []
    p = dict(HOME)
    first = "KEYBOARD CONTROL 2 OFF; CUTOFF FREQUENCY fully clockwise; "
    for v in (2, 4, 6, 8, 10, 5):
        p["ext_volume"] = 0.375 if v == 5 else v / 10
        s, sig, ev = level_steps(1000.0, LEVELS, 0.0)
        x = take(f"extvol{v}_levels", s, sig, ev, "1 kHz steps into EXT, LC GATE held")
        x["panel"] = dict(p)
        x["set"] = first + f"EXT VOLUME {v}" if first else (
            "EXT VOLUME back to 5" if v == 5 else f"EXT VOLUME {v}")
        first = ""
        t.append(x)
    return t


# Printed ATTACK/DECAY marks on the reference (1 ms fully counterclockwise to 10 s fully
# clockwise). Knob values are the CA-72's audio-taper guess so a render starts near the
# mark; the capture is what fits them.
ATTACK_MARKS = (
    ("1ms", 0.00, 0.8, 0.5, 2),
    ("10ms", 0.18, 0.8, 0.5, 2),
    ("100ms", 0.38, 1.2, 0.5, 2),
    ("1s", 0.62, 3.5, 0.8, 1),
    ("10s", 1.00, 14.0, 1.0, 1),
)
DECAY_MARKS = (
    ("1ms", 0.00, 0.6, 0.8, 2),
    ("10ms", 0.18, 0.6, 1.0, 2),
    ("100ms", 0.38, 0.6, 1.5, 2),
    ("1s", 0.62, 0.6, 5.0, 1),
    ("10s", 1.00, 0.6, 16.0, 1),
)


def contour_take(name, panel, set_line, pulses, with_tone=True, notes=""):
    """Gate pulses for the contour session. Each pulse is (gate, on, off) with gate one of
    lc, fc or both. A 1 kHz tone rides EXT so the loudness contour is also in the main
    output. Optical returns (FILT CONT, LOUD CONT) are recorded with the take."""
    t = LEAD + 0.2
    for _, on, off in pulses:
        t += on + off
    seconds = t + 0.3
    ext, lc, fc = zeros(seconds), zeros(seconds), zeros(seconds)
    if with_tone:
        put(ext, max(LEAD - 0.05, 0.0), tone(1000.0, 0.005, seconds - LEAD + 0.05))
    ev = []
    t = LEAD + 0.2
    for gate, on, off in pulses:
        if gate in ("lc", "both"):
            lc[span(t, t + on)] = GATE
        if gate in ("fc", "both"):
            fc[span(t, t + on)] = GATE
        ev.append([round(t, 4), round(on, 4), gate])
        t += on + off
    x = take(name, seconds, {"ext": ext, "lc_gate": lc, "fc_gate": fc} if with_tone else
             {"lc_gate": lc, "fc_gate": fc},
             {"pulses": ev, "tone_hz": 1000.0 if with_tone else None}, notes)
    x["panel"], x["set"], x["record_optical"] = dict(panel), set_line, True
    return x


def session_f():
    """The contours at their printed ATTACK and DECAY marks and SUSTAIN 0, 5 and 10.
    Loudness first, then the filter, each contour's output patched to a DC-coupled modular
    return (computer inputs 11 to 26). LC GATE and FC GATE pulsed separately so each
    contour is seen on its jack and, for loudness, on the main output."""
    t = []
    p = dict(HOME)
    pulses_id = (("lc", 0.4, 0.6), ("fc", 0.4, 0.6))
    t.append(contour_take(
        "identify", p,
        "Patch FILT CONT and LOUD CONT into two DC-coupled modular inputs that return "
        "through the MOTU's optical ins; leave EXT VOLUME at 5, CUTOFF fully clockwise, "
        "KEYBOARD CONTROL off, both ATTACK at 1 ms, both SUSTAIN at 10, both DECAY switches off",
        pulses_id, notes="which optical inputs the two contour jacks land on"))

    def add(name, line, pulses, **change):
        p.update(change)
        t.append(contour_take(name, p, line, pulses))

    for mark, knob, on, off, n in ATTACK_MARKS:
        line = (f"LOUDNESS ATTACK to the {mark} mark" if mark != "1ms" else
                "nothing: both ATTACK already at 1 ms")
        add(f"loud_att_{mark}", line, (("lc", on, off),) * n, loudness_attack=knob)
    add("loud_att_cw", "LOUDNESS ATTACK fully clockwise (past the 10 s mark)",
        (("lc", 14.0, 1.0),), loudness_attack=1.0)
    add("loud_att_1ms_again", "LOUDNESS ATTACK back to 1 ms",
        (("lc", 0.8, 0.5),) * 2, loudness_attack=0.0)

    add("loud_dec_on_1ms", "LOUDNESS DECAY switch ON (FILTER DECAY stays off); LOUDNESS DECAY at 1 ms",
        (("lc", 0.6, 0.8),) * 2, decay=True, loudness_decay=0.0)
    for mark, knob, on, off, n in DECAY_MARKS[1:]:
        add(f"loud_dec_{mark}", f"LOUDNESS DECAY to the {mark} mark",
            (("lc", on, off),) * n, loudness_decay=knob)
    add("loud_dec_cw", "LOUDNESS DECAY fully clockwise (past the 10 s mark)",
        (("lc", 0.6, 16.0),), loudness_decay=1.0)

    add("loud_sus_0", "LOUDNESS DECAY back to the 100 ms mark; LOUDNESS SUSTAIN 0",
        (("lc", 2.0, 1.5),) * 2, loudness_decay=0.38, loudness_sustain=0.0)
    add("loud_sus_5", "LOUDNESS SUSTAIN 5",
        (("lc", 2.0, 1.5),) * 2, loudness_sustain=0.5)
    add("loud_sus_10", "LOUDNESS SUSTAIN back to 10",
        (("lc", 2.0, 1.5),) * 2, loudness_sustain=1.0)

    add("loud_dec_off", "LOUDNESS DECAY switch OFF; LOUDNESS DECAY back to 1 ms",
        (("lc", 0.6, 0.8),) * 2, decay=False, loudness_decay=0.0)

    add("filt_att_1ms", "FILTER ATTACK at 1 ms (already); I pulse FC GATE",
        (("fc", 0.8, 0.5),) * 2, filter_attack=0.0)
    for mark, knob, on, off, n in ATTACK_MARKS[1:]:
        add(f"filt_att_{mark}", f"FILTER ATTACK to the {mark} mark",
            (("fc", on, off),) * n, filter_attack=knob)
    add("filt_att_cw", "FILTER ATTACK fully clockwise (past the 10 s mark)",
        (("fc", 14.0, 1.0),), filter_attack=1.0)
    add("filt_att_1ms_again", "FILTER ATTACK back to 1 ms",
        (("fc", 0.8, 0.5),) * 2, filter_attack=0.0)

    add("filt_dec_on_1ms", "FILTER DECAY switch ON (LOUDNESS DECAY stays off); FILTER DECAY at 1 ms",
        (("fc", 0.6, 0.8),) * 2, decay=True, filter_decay=0.0)
    for mark, knob, on, off, n in DECAY_MARKS[1:]:
        add(f"filt_dec_{mark}", f"FILTER DECAY to the {mark} mark",
            (("fc", on, off),) * n, filter_decay=knob)
    add("filt_dec_cw", "FILTER DECAY fully clockwise (past the 10 s mark)",
        (("fc", 0.6, 16.0),), filter_decay=1.0)

    add("filt_sus_0", "FILTER DECAY back to the 100 ms mark; FILTER SUSTAIN 0",
        (("fc", 2.0, 1.5),) * 2, filter_decay=0.38, filter_sustain=0.0)
    add("filt_sus_5", "FILTER SUSTAIN 5",
        (("fc", 2.0, 1.5),) * 2, filter_sustain=0.5)
    add("filt_sus_10", "FILTER SUSTAIN back to 10",
        (("fc", 2.0, 1.5),) * 2, filter_sustain=1.0)

    add("filt_dec_off", "FILTER DECAY switch OFF; FILTER DECAY back to 1 ms",
        (("fc", 0.6, 0.8),) * 2, decay=False, filter_decay=0.0)
    return t


# The marks of the reference's ATTACK and DECAY dials not taken in session F (its manual's
# drawing and the owner's photographs: 200 ms at -90 degrees, 600 ms at -30, the unlabelled
# top tick at 0, 5 s at 60): (name, knob, gate on, gate off, pulses).
ATTACK_MARKS_J = (
    ("200ms", 0.2, 1.0, 0.5, 2),
    ("600ms", 0.4, 1.5, 0.6, 2),
    ("top", 0.5, 2.5, 0.8, 1),
    ("5s", 0.7, 7.0, 1.0, 1),
)
DECAY_MARKS_J = (
    ("200ms", 0.2, 0.6, 1.5, 2),
    ("600ms", 0.4, 0.6, 2.5, 2),
    ("top", 0.5, 0.6, 3.5, 1),
    ("5s", 0.7, 0.6, 9.0, 1),
)


def amount_take(name, panel, set_line):
    """AMOUNT OF CONTOUR: the filter self-oscillating, FC GATE off 0.8 s then on 1.6 s (the
    contour held at its SUSTAIN after a fast decay), twice: CUT CV at 0, then at +0.1 of full
    scale (the filter's own octaves a volt, against which the contour's are counted)."""
    s = LEAD + 2 * 2.4 + 0.4
    fc, cv, lc = zeros(s), zeros(s), zeros(s)
    lc[span(LEAD, s - 0.1)] = GATE           # the VCA open throughout
    segs = []
    for i, c in enumerate((0.0, 0.1)):
        t0 = LEAD + i * 2.4
        cv[span(t0, t0 + 2.4)] = c
        fc[span(t0 + 0.8, t0 + 2.4)] = GATE
        segs.append([round(t0 + 0.3, 4), round(t0 + 0.75, 4), c, "off"])
        segs.append([round(t0 + 1.6, 4), round(t0 + 2.35, 4), c, "held"])
    x = take(name, s, {"fc_gate": fc, "cut": cv, "lc_gate": lc}, {"segments": segs},
             "the filter self-oscillating; FC GATE held over its SUSTAIN; CUT CV 0 then +1 V")
    x["panel"], x["set"], x["record_optical"] = dict(panel), set_line, True
    return x


def session_j():
    """What the CA-72's laws still interpolate or have not measured (after session I, the
    first preset played on the reference). The reference tuned to A-440 first, by hand with
    `midi_capture.py` (not a take here). Then: AMOUNT OF CONTOUR at 0, 2.5, 5, 7.5 and 10;
    ATTACK and DECAY at the 200 ms, 600 ms, top and 5 s marks, loudness then filter;
    EMPHASIS 6, 7 and 8.5; KEYBOARD CONTROL 1 and 2 over five MIDI notes (each sent before
    its take); oscillators 2 and 3's and the noise's VOLUME at 4 and 8 against 10. The mod
    wheel and GLIDE are MIDI phrases (`phrases/modwheel.json`, `phrases/glide.json`)."""
    t = []
    # AMOUNT OF CONTOUR: CUTOFF -2, EMPHASIS 10, the filter contour fast to a SUSTAIN of 5.
    p = dict(HOME, ext_on=False, cutoff=cutoff_knob(-2), emphasis=1.0, filter_attack=0.0,
             filter_decay=0.0, filter_sustain=0.5, decay=False)
    first = ("EXT IN switch OFF; CUTOFF -2; EMPHASIS 10; KEYBOARD CONTROL 1 and 2 OFF; "
             "FILTER ATTACK and DECAY fully anticlockwise, FILTER SUSTAIN 5; FILTER DECAY switch "
             "OFF; AMOUNT OF CONTOUR 0")
    for a, line in ((0.0, first), (0.25, "AMOUNT OF CONTOUR 2.5 (halfway between 2 and 3)"),
                    (0.5, "AMOUNT OF CONTOUR 5"), (0.75, "AMOUNT OF CONTOUR 7.5"),
                    (1.0, "AMOUNT OF CONTOUR 10 (fully clockwise)")):
        p["contour_amount"] = a
        t.append(amount_take(f"amount{a * 10:g}", p, line))

    # The contours at the marks session F did not take (as session F: EXT's tone through the
    # VCA, both contours' jacks on the modular's DC inputs).
    p = dict(HOME, contour_amount=0.0, emphasis=0.0, cutoff=1.0, ext_on=True,
             filter_sustain=1.0, filter_decay=0.0, loudness_decay=0.0, decay=False)
    for which, gate in (("loudness", "lc"), ("filter", "fc")):
        short = which.upper()
        for j, (mark, knob, on, off, n) in enumerate(ATTACK_MARKS_J):
            line = f"{short} ATTACK to the {mark} mark" if mark != "top" else \
                f"{short} ATTACK to the unlabelled top (12 o'clock) tick"
            if j == 0 and which == "loudness":
                line = ("AMOUNT OF CONTOUR back to 0; EMPHASIS 0; CUTOFF fully clockwise; EXT IN "
                        "switch ON (VOLUME 5); FILTER SUSTAIN 10; both DECAY knobs fully "
                        "anticlockwise, both DECAY switches OFF; " + line)
            elif j == 0:
                line = ("LOUDNESS DECAY back fully anticlockwise and its switch OFF; " + line)
            p[f"{which}_attack"] = knob
            t.append(contour_take(f"{which[:4]}_att_{mark}", p, line, ((gate, on, off),) * n))
        p[f"{which}_attack"] = 0.0
        for i, (mark, knob, on, off, n) in enumerate(DECAY_MARKS_J):
            line = f"{short} DECAY to the {mark} mark" if mark != "top" else \
                f"{short} DECAY to the unlabelled top (12 o'clock) tick"
            if i == 0:
                line = (f"{short} ATTACK back fully anticlockwise; {short} DECAY switch ON; "
                        + line)
            p["decay"] = True
            p[f"{which}_decay"] = knob
            t.append(contour_take(f"{which[:4]}_dec_{mark}", p, line, ((gate, on, off),) * n))
        p[f"{which}_decay"] = 0.0
        p["decay"] = False

    # EMPHASIS between session E's 5 and 7.5, and toward regeneration (as session E).
    p = dict(HOME)
    cuts = [-0.1, -0.2, -0.3, -0.4, -0.5]
    for e, line in ((6.0, "both DECAY switches OFF, both DECAY knobs anticlockwise; "
                          "FILTER EMPHASIS 6"),
                    (7.0, "FILTER EMPHASIS 7"), (8.5, "FILTER EMPHASIS 8.5 (halfway between 8 "
                                                      "and 9)")):
        p["emphasis"] = e / 10
        t.append(sweep_take(f"emph{e:g}", p, line, cuts))

    # KEYBOARD CONTROL 1 and 2 over five notes (CUTOFF 0; each note sent before its take).
    p = dict(HOME, emphasis=0.0, cutoff=cutoff_knob(0), keyboard_control_1=True,
             keyboard_control_2=True)
    for i, note in enumerate((29, 41, 53, 65, 77)):
        line = (f"(I send MIDI note {note})" if i else
                "FILTER EMPHASIS back to 0; CUTOFF 0; KEYBOARD CONTROL 1 and 2 ON "
                f"(I send MIDI note {note})")
        t.append(sweep_take(f"kbd12_n{note}", p, line, [0.0], midi=note))

    # The mixer's VOLUME on oscillators 2 and 3 and the noise (as session C: MIX beside MAIN).
    p = dict(HOME, ext_on=False, keyboard_control_1=False, keyboard_control_2=False,
             cutoff=1.0)
    for n in (1, 2, 3):
        p.update({f"osc{n}_range": "8", f"osc{n}_waveform": "sawtooth", f"osc{n}_volume": 1.0,
                  f"osc{n}_on": False})
        if n > 1:
            p[f"osc{n}_freq"] = 0.5
    for src, label in (("osc2", "OSC 2"), ("osc3", "OSC 3"), ("noise", "NOISE")):
        for v in (1.0, 0.8, 0.4):
            if v == 1.0:
                line = (f"{label} switch ON at VOLUME 10, the others OFF"
                        + ("; WAVEFORM sawtooth, RANGE 8', FREQUENCY 0" if src != "noise"
                           else "; WHITE") +
                        ("; KEYBOARD CONTROL 1 and 2 OFF; EXT IN switch OFF; CUTOFF fully "
                         "clockwise" if src == "osc2" else ""))
            else:
                line = f"{label} VOLUME {v * 10:g}"
            p[f"{src}_on"] = True
            p[f"{src}_volume"] = v
            t.append(panel_take(f"{src}_vol{v * 10:g}", p, line))
        p[f"{src}_on"] = False
    return t
