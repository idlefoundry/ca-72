#!/usr/bin/env python3
"""Send MIDI notes out the MOTU and record the reference on the same clock.

A note is stamped for the audio DAC time of its sample, so the recording (aligned on
the loopback, as capture.py does) shows the reference's own delay from a MIDI note to
its contour. Run from the terminal that has the microphone permission.

    midi_capture.py OUTDIR
"""

import ctypes
import datetime
import hashlib
import json
import os
import sys
import time
from ctypes import CDLL, POINTER, byref, c_char_p, c_int32, c_uint8, c_uint32, c_uint64, c_void_p

import numpy as np
import sounddevice as sd
import soundfile as sf

FS = 48000
DEVICE = "828ES"
N_OUT = 24
N_IN = 28
OUT_LOOP = 13          # computer out 14, 0-based
IN = {"main": 3, "mix": 5, "loop": 7}
QUIET_LEAD = 0.3
MARKER_AT = 0.1
TAIL = 0.6
NOTE = 60
CHANNEL = 0            # MIDI channel 1

core = CDLL("/System/Library/Frameworks/CoreMIDI.framework/CoreMIDI")
cf = CDLL("/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation")
libc = CDLL("/usr/lib/libSystem.B.dylib")

cf.CFStringCreateWithCString.restype = c_void_p
cf.CFStringCreateWithCString.argtypes = [c_void_p, c_char_p, c_uint32]
core.MIDIClientCreate.argtypes = [c_void_p, c_void_p, c_void_p, POINTER(c_uint32)]
core.MIDIClientCreate.restype = c_int32
core.MIDIOutputPortCreate.argtypes = [c_uint32, c_void_p, POINTER(c_uint32)]
core.MIDIOutputPortCreate.restype = c_int32
core.MIDIGetNumberOfDestinations.restype = c_uint64
core.MIDIGetDestination.argtypes = [c_uint64]
core.MIDIGetDestination.restype = c_uint32
core.MIDIPacketListInit.argtypes = [c_void_p]
core.MIDIPacketListInit.restype = c_void_p
core.MIDIPacketListAdd.argtypes = [c_void_p, c_uint64, c_void_p, c_uint64, c_uint64, c_void_p]
core.MIDIPacketListAdd.restype = c_void_p
core.MIDISend.argtypes = [c_uint32, c_uint32, c_void_p]
core.MIDISend.restype = c_int32
libc.mach_absolute_time.restype = c_uint64


class Timebase(ctypes.Structure):
    _fields_ = [("numer", c_uint32), ("denom", c_uint32)]


_tb = Timebase()
libc.mach_timebase_info(byref(_tb))
TICKS_PER_SEC = 1e9 * _tb.denom / _tb.numer


def cfstr(s):
    return cf.CFStringCreateWithCString(None, s.encode(), 0x08000100)


def open_midi():
    client, port = c_uint32(), c_uint32()
    err = core.MIDIClientCreate(cfstr("ca72-cal"), None, None, byref(client))
    if err:
        raise RuntimeError(f"MIDIClientCreate {err}")
    err = core.MIDIOutputPortCreate(client, cfstr("out"), byref(port))
    if err:
        raise RuntimeError(f"MIDIOutputPortCreate {err}")
    n = core.MIDIGetNumberOfDestinations()
    if n < 1:
        raise RuntimeError("no MIDI destination")
    return port, core.MIDIGetDestination(0)


def send(port, dest, status, data1, data2, host_time):
    buf = (c_uint8 * 256)()
    pkt = core.MIDIPacketListInit(buf)
    data = (c_uint8 * 3)(status, data1, data2)
    pkt = core.MIDIPacketListAdd(buf, 256, pkt, c_uint64(host_time), 3, data)
    if not pkt:
        raise RuntimeError("MIDIPacketListAdd failed")
    err = core.MIDISend(port, dest, buf)
    if err:
        raise RuntimeError(f"MIDISend {err}")


def marker():
    n = int(0.002 * FS)
    t = np.arange(n) / FS
    return (0.1 * np.sin(2 * np.pi * 4000 * t) * np.hanning(n)).astype(np.float32)


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def events():
    """Note on/off pairs, in seconds on the output timeline."""
    ev = []
    t = 1.0
    for _ in range(5):
        ev.append((t, "on"))
        ev.append((t + 0.7, "off"))
        t += 1.3
    return ev


def run(outdir):
    os.makedirs(outdir, exist_ok=True)
    ev = events()
    seconds = ev[-1][0] + 1.2
    n_take = int(round(seconds * FS))
    n = n_take + int(TAIL * FS)
    out = np.zeros((n, N_OUT), np.float32)
    m = marker()
    k = int(MARKER_AT * FS)
    out[k:k + len(m), OUT_LOOP] = m
    # A steady tone on EXT, so the main output shows the loudness contour too.
    tone = np.zeros(n, np.float32)
    tt = np.arange(n) / FS
    tone[int(QUIET_LEAD * FS):n_take] = (0.005 * np.sin(2 * np.pi * 1000 * tt[int(QUIET_LEAD * FS):n_take])).astype(np.float32)
    out[:n_take, 12] = tone[:n_take]   # computer out 13, EXT
    out[:n_take, OUT_LOOP] += tone[:n_take]

    port, dest = open_midi()
    pending = [(int(round(t * FS)), kind) for t, kind in ev]
    sent = []
    rec = np.zeros((n, N_IN), np.float32)
    cursor = 0
    slack = []

    def callback(indata, outdata, frames, time_info, status):
        nonlocal cursor
        end = min(cursor + frames, n)
        got = end - cursor
        outdata[:] = 0
        outdata[:got] = out[cursor:end]
        rec[cursor:end] = indata[:got]
        now = libc.mach_absolute_time()
        for sample, kind in list(pending):
            if cursor <= sample < cursor + frames:
                dac = time_info.outputBufferDacTime + (sample - cursor) / FS
                ahead = dac - time_info.currentTime
                host = now + int(ahead * TICKS_PER_SEC)
                status_b = 0x90 | CHANNEL if kind == "on" else 0x80 | CHANNEL
                vel = 100 if kind == "on" else 0
                send(port, dest, status_b, NOTE, vel, host)
                sent.append({"t": round(sample / FS, 6), "kind": kind, "ahead_s": round(ahead, 6)})
                pending.remove((sample, kind))
                slack.append(ahead)
        cursor = end

    started = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    with sd.Stream(samplerate=FS, device=DEVICE, channels=(N_IN, N_OUT), dtype="float32",
                   callback=callback, latency="high"):
        deadline = time.time() + n / FS + 5
        while cursor < n and time.time() < deadline:
            sd.sleep(50)

    if pending:
        raise RuntimeError(f"unsent events: {pending}")

    lp = rec[:int(1.0 * FS), IN["loop"] - 1]
    xc = np.correlate(lp, m, mode="valid")
    lag = int(np.argmax(np.abs(xc))) - k
    mark_db = float(20 * np.log10(np.max(np.abs(xc)) / np.dot(m, m) + 1e-12))
    aligned = 0 < lag < int(0.8 * FS) and abs(mark_db) <= 3
    if aligned:
        rec = np.concatenate([rec[lag:], np.zeros((lag, N_IN), np.float32)])

    extra = [f"in{c}" for c in range(11, 27)]
    cols = ["main", "mix", "loop", "ext", "cut", "lc_gate", "fc_gate", "vpo"] + extra
    # Stimuli were not on those columns except ext, which rode the loop. Keep ext as sent.
    data = np.zeros((n, len(cols)), np.float32)
    stimuli = {"ext": out[:, 12], "cut": out[:, 11], "lc_gate": out[:, 8], "fc_gate": out[:, 9],
               "vpo": out[:, 10]}
    for i, c in enumerate(cols):
        if c in IN:
            data[:, i] = rec[:, IN[c] - 1]
        elif c.startswith("in") and c[2:].isdigit():
            data[:, i] = rec[:, int(c[2:]) - 1]
        else:
            data[:, i] = stimuli[c][:n]

    base = os.path.join(outdir, "00_midi_notes")
    sf.write(base + ".wav", data, FS, subtype="FLOAT")
    record = {
        "session": os.path.basename(os.path.normpath(outdir)),
        "take": "midi_notes",
        "index": 0,
        "started": started,
        "seconds": seconds,
        "rate": FS,
        "columns": cols,
        "latency_samples": lag,
        "aligned": bool(aligned),
        "timing_mark_db": round(mark_db, 2),
        "tail_seconds": TAIL,
        "midi_note": NOTE,
        "midi_channel": CHANNEL + 1,
        "midi_ahead_s": [round(float(s), 6) for s in slack],
        "events": {"notes": sent},
        "notes": "MIDI notes from the MOTU MIDI OUT, stamped to the audio DAC time",
        "wav": os.path.basename(base + ".wav"),
        "wav_sha256": sha256(base + ".wav"),
        "problems": [] if aligned else [f"timing mark not found (lag {lag}, {mark_db:.1f} dB)"],
    }
    with open(base + ".json", "w") as f:
        json.dump(record, f, indent=1)
    print(f"00_midi_notes: {seconds:.1f} s, latency {lag} samples, "
          f"{'OK' if aligned else 'PROBLEM'}, ahead {min(slack):.4f}..{max(slack):.4f} s",
          flush=True)


if __name__ == "__main__":
    run(sys.argv[1])
