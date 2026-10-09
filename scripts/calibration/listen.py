#!/usr/bin/env python3
"""Builds a listening page: the reference's recordings beside the CA-72's renders of the same
stimuli, and the CA-72 before and after a change (docs/calibration).

    listen.py PAGE.json OUTDIR

PAGE.json: {"title", "intro", "sections": [{"title", "text", "clips": [{"label", "kind",
"path", "start", "end"}], "plot": "file.png"}]}. A clip's kind is "capture" (a take's WAV:
its main output), "render" (ca72-lab stim's WAV: channel 0, passed through the interface's
input high-pass so that it is heard as the captures are) or "f32" (preset_render's stereo
float file). Each clip is levelled to -20 dBFS RMS (above 50 Hz) and written as FLAC and as
MP3; the page embeds the MP3s and the plots, so it is one file. Each clip is levelled to -20
dB (ITU-R BS.1770's K-weighting, ungated), the reference's against the CA-72's; a section
with "same_gain" gives its clips the first one's gain, so that a change's own level
difference stays.
"""

import base64
import html
import json
import os
import subprocess
import sys

import numpy as np
import soundfile as sf
from scipy.signal import lfilter

FS = 48000
TAU_INTERFACE = 0.1695   # docs/calibration: the interface's input high-pass


def interface_hp(y):
    a = np.exp(-1 / (FS * TAU_INTERFACE))
    return lfilter([(1 + a) / 2, -(1 + a) / 2], [1, -a], y)


def load(clip):
    kind, path = clip["kind"], clip["path"]
    if kind == "capture":
        with open(path) as f:
            meta = json.load(f)
        x, fs = sf.read(os.path.join(os.path.dirname(path), meta["wav"]), dtype="float64")
        y = x[:, meta["columns"].index("main")]
    elif kind == "render":
        x, fs = sf.read(path, dtype="float64")
        y = interface_hp(x[:, 0])
    elif kind == "f32":
        y = np.fromfile(path, dtype="<f4").astype(np.float64).reshape(-1, 2)
        fs = FS
    else:
        raise ValueError(kind)
    assert fs == FS
    i0 = int(clip.get("start", 0) * FS)
    i1 = int(clip["end"] * FS) if "end" in clip else len(y)
    return y[i0:i1]


def k_weighted_rms(y):
    """ITU-R BS.1770's K-weighting at 48 kHz (its shelf and its high-pass), ungated."""
    m = y if y.ndim == 1 else y.mean(axis=1)
    m = lfilter([1.53512485958697, -2.69169618940638, 1.19839281085285],
                [1.0, -1.69065929318241, 0.73248077421585], m)
    m = lfilter([1.0, -2.0, 1.0], [1.0, -1.99004745483398, 0.99007225036621], m)
    return np.sqrt(np.mean(m * m))


def gain_for(y, target_db=-20.0):
    return 10 ** (target_db / 20) / max(k_weighted_rms(y), 1e-12)


def finish(y, g):
    out = y * g
    k = int(0.005 * FS)
    ramp = np.linspace(0, 1, k)
    if out.ndim == 1:
        out[:k] *= ramp
        out[-k:] *= ramp[::-1]
    else:
        out[:k] *= ramp[:, None]
        out[-k:] *= ramp[::-1, None]
    return out


def data_uri(path, mime):
    with open(path, "rb") as f:
        return f"data:{mime};base64," + base64.b64encode(f.read()).decode()


def main():
    page_json, outdir = sys.argv[1], sys.argv[2]
    with open(page_json) as f:
        page = json.load(f)
    os.makedirs(outdir, exist_ok=True)
    parts = [f"<!doctype html><meta charset=utf-8><title>{html.escape(page['title'])}</title>",
             "<style>body{font:15px/1.45 system-ui,sans-serif;max-width:980px;margin:2em auto;"
             "padding:0 1em;color:#222}h1{font-size:1.5em}h2{font-size:1.15em;margin-top:2em}"
             ".clip{display:flex;align-items:center;gap:.8em;margin:.35em 0}.clip span{width:17em}"
             "img{max-width:100%;border:1px solid #ddd;margin:.6em 0}small{color:#666}</style>",
             f"<h1>{html.escape(page['title'])}</h1>", f"<p>{page['intro']}</p>"]
    n = 0
    for sec in page["sections"]:
        parts.append(f"<h2>{html.escape(sec['title'])}</h2><p>{sec.get('text', '')}</p>")
        clips = [(clip, load(clip)) for clip in sec.get("clips", [])]
        gains = [gain_for(y) for _, y in clips]
        if sec.get("same_gain") and gains:
            gains = [gains[0]] * len(gains)
        # No clip of a section above -1 dBFS: the section's gains lowered together.
        peak = max([np.max(np.abs(y)) * g for (_, y), g in zip(clips, gains)] or [0])
        if peak > 0.89:
            gains = [g * 0.89 / peak for g in gains]
        for (clip, y0), g in zip(clips, gains):
            n += 1
            y = finish(y0, g)
            base = f"{n:02d}_" + "".join(ch if ch.isalnum() else "_" for ch in clip["label"])[:48]
            flac = os.path.join(outdir, base + ".flac")
            mp3 = os.path.join(outdir, base + ".mp3")
            sf.write(flac, y, FS, subtype="PCM_24")
            subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", flac, "-b:a", "192k", mp3],
                           check=True)
            parts.append(f"<div class=clip><span>{html.escape(clip['label'])}</span>"
                         f"<audio controls preload=none src='{data_uri(mp3, 'audio/mpeg')}'></audio>"
                         f"<small><a href='{os.path.basename(flac)}'>lossless</a></small></div>")
        if sec.get("plot"):
            parts.append(f"<img src='{data_uri(sec['plot'], 'image/png')}'>")
    parts.append(f"<p><small>{page.get('footer', '')}</small></p>")
    with open(os.path.join(outdir, "index.html"), "w") as f:
        f.write("\n".join(parts))
    print(os.path.join(outdir, "index.html"), n, "clips")


if __name__ == "__main__":
    main()
