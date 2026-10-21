#!/usr/bin/env python3
"""Exercise reference fitting through ca72-mcp stdio and retain a listening report.

Usage: reference-audio-demo.py /absolute/ca72-mcp /absolute/new-output-directory
The reference is synthetic, based on the factory White Lotus preset. It is not the
original studio recording. No live instance, device, personal preset or network is used.
"""
import html
import json
import os
from pathlib import Path
import subprocess
import sys


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    binary, output = map(lambda s: Path(s).resolve(), sys.argv[1:])
    output.mkdir()
    # Isolate library/settings discovery from the player's files.
    env = {**os.environ, "IDLE_FOUNDRY_SHARED": str(output / "isolated-presets"),
           "XDG_DATA_HOME": str(output / "isolated-data"),
           "CA72_AGENT_DIR": str(output / "no-live-instance")}
    process = subprocess.Popen([str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               text=True, env=env)
    serial = 0

    def rpc(method, params):
        nonlocal serial
        serial += 1
        process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": serial,
                                        "method": method, "params": params}) + "\n")
        process.stdin.flush()
        line = process.stdout.readline()
        if not line:
            raise RuntimeError("MCP bridge ended before answering")
        reply = json.loads(line)
        if "error" in reply:
            raise RuntimeError(reply["error"])
        return reply["result"]

    def tool(name, args):
        reply = rpc("tools/call", {"name": name, "arguments": args})
        if reply["isError"]:
            raise RuntimeError(reply["structuredContent"])
        return reply["structuredContent"]

    try:
        rpc("initialize", {"protocolVersion": "2025-11-25",
                           "clientInfo": {"name": "reference-audio-demo"}})
        notes = [{"note": 38, "start": 0.1, "duration": 1.7},
                 {"note": 36, "start": 2.0, "duration": 1.5}]
        common = {"effects": "dry", "notes": notes, "seconds": 4, "sample_rate": 48000}
        reference = tool("ca72_render", {**common, "preset": "White Lotus", "seed": 271,
            "values": {"quality": "HI", "cutoff": -2.14},
            "output_dir": str(output / "reference")})
        rows = []
        for i, cutoff in enumerate([-3.0, -2.5, -2.14, -1.75, -1.25]):
            candidate = tool("ca72_render", {**common, "snapshot": reference["snapshot"],
                "seed": 72, "values": {"cutoff": cutoff},
                "output_dir": str(output / f"candidate-{i}")})
            comparison = tool("ca72_compare", {"reference": reference["path"],
                "candidate": candidate["path"], "include_image": False,
                "output_dir": str(output / f"comparison-{i}")})
            score = sum(r["spectral_mae_db"] for r in comparison["resolutions"]) / 3
            rows.append({"cutoff": cutoff, "spectral_mae_db": score,
                         "envelope_mae_db": comparison["envelope_mae_db"],
                         "candidate_gain_db": comparison["candidate_gain_db"],
                         "candidate": candidate, "comparison": comparison})
            print(json.dumps({k: rows[-1][k] for k in ("cutoff", "spectral_mae_db", "envelope_mae_db")}), flush=True)
        best = min(rows, key=lambda r: r["spectral_mae_db"])
        if best["cutoff"] != -2.14:
            raise RuntimeError(f"Known held-bass fixture was not recovered: {best['cutoff']}")
        analysis = tool("ca72_analyze", {"path": best["candidate"]["path"], "start": 0.35,
            "seconds": 1.0, "fundamental_hz": 73.4162, "modulation_hz": 11.3,
            "include_image": False, "output_dir": str(output / "held-note")})
        summary = {"reference_kind": "synthetic White Lotus held-bass fixture",
                   "reference_seed": 271, "candidate_seed": 72, "best_cutoff": best["cutoff"],
                   "rows": rows, "held_note_analysis": analysis,
                   "limits": "Controlled one-parameter recovery. No original recording, automatic transcription, source separation or perceptual pass is claimed."}
        (output / "summary.json").write_text(json.dumps(summary, indent=2))
        table = "".join(f"<tr><td>{r['cutoff']}</td><td>{r['spectral_mae_db']:.3f}</td>"
                        f"<td>{r['envelope_mae_db']:.3f}</td><td>{r['candidate_gain_db']:.3f}</td></tr>" for r in rows)
        relative = lambda p: html.escape(str(Path(p).relative_to(output)))
        comparison = best["comparison"]
        report = f'''<!doctype html><html lang="en"><meta charset="utf-8">
<title>CA-72 reference fitting verification</title><style>
body{{max-width:1000px;margin:40px auto;padding:0 20px;background:#14171c;color:#e6e8ed;font:16px/1.55 system-ui}}
table{{border-collapse:collapse;width:100%}}td,th{{text-align:left;padding:10px;border-bottom:1px solid #455}}
img{{max-width:100%;display:block}}audio{{width:100%}}a{{color:#9bcaff}}</style>
<h1>CA-72 reference fitting verification</h1>
<p>A synthetic held-bass reference based on White Lotus, rendered at QUALITY HI. The reference
uses seed 271 and candidates use seed 72, so phase and component variation differ. The tools
recover the reference's CUTOFF of −2.14 from five candidates using spectral measurements.</p>
<p>This verifies a controlled one-parameter fitting loop. It does not establish a match to the
original studio recording or an automatic sound-recreation system.</p>
<table><tr><th>CUTOFF</th><th>Mean spectral error, dB</th><th>Envelope error, dB</th><th>Matching gain, dB</th></tr>{table}</table>
<h2>Level-matched listening pair</h2><p>Reference</p><audio controls src="{relative(comparison['reference_ab'])}"></audio>
<p>Selected candidate</p><audio controls src="{relative(comparison['candidate_ab'])}"></audio>
<h2>Reference spectrogram</h2><img alt="Reference spectrogram with labelled frequency and time axes" src="{relative(comparison['reference_spectrogram'])}">
<h2>Selected candidate, same colour scale</h2><img alt="Candidate spectrogram on the reference scale" src="{relative(comparison['candidate_spectrogram'])}">
<p>Full source hashes, settings and measurements: <a href="summary.json">summary.json</a>.</p></html>'''
        (output / "report.html").write_text(report)
        print(output / "report.html", flush=True)
    finally:
        process.stdin.close()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


if __name__ == "__main__":
    main()
