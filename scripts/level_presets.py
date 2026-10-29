"""Levels the factory presets (decisions.md R15, R-PRESETS-0.3.0): measures each one's loudness
at QUALITY LO, HI and ULTRA by tests/preset_levels.rs (its phrase through the plug-in's own
process, the momentary maximum), then sets MAIN OUTPUT VOLUME (a loud one down) or LEVEL (a quiet
one up, VOLUME at 10) so that

- each of 0.3.0's presets (scripts/presets_0_3.py) reads -18 LUFS midway between LO and HI;
- every other preset reads within -18 +/- 1.5 LUFS at each quality measured: one within it is
  left as it is, one outside it is levelled as 0.3.0's are;
- where midway leaves ULTRA outside the tolerance, the level moves (as far as LO and HI stay
  0.25 dB within it) to bring ULTRA in, or as near as it comes;

and measures again until each is within 0.1 dB of where it was sent. A preset whose MAIN OUTPUT
VOLUME an LFO, a macro or velocity moves keeps its VOLUME, and LEVEL alone moves (VOLUME moved
would move the modulation along the pot's taper, or leave it nothing to raise at 10:
decisions.md R-OWNER-PRESETS). 0.3.0's levels go into
scripts/presets_0_3_levels.json and the generator is run; another preset's VOLUME and LEVEL
are written into crates/ca72-plugin/sounds/presets.toml in place. Prints each preset's readings
before and after, and with --report writes them as Markdown.

    python3 scripts/level_presets.py [--qualities lo,hi,ultra] [--passes 3] [--jobs 12]
        [--only NAME ...] [--check] [--report FILE]

--check measures and reports, changing nothing. The AUTO GAIN curves depend on neither VOLUME
nor LEVEL, so levelling leaves them as they are (tests/factory_curves.rs writes them).
"""
import argparse, concurrent.futures, json, math, os, pathlib, re, subprocess, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
FILE = ROOT / "crates/ca72-plugin/sounds/presets.toml"
LEVELS = ROOT / "scripts/presets_0_3_levels.json"
GENERATOR = ROOT / "scripts/presets_0_3.py"
MARKER = "# ---- 0.3.0's presets"
TARGET, TOLERANCE, CLOSE, MARGIN = -18.0, 1.5, 0.1, 0.25
LEVEL_RANGE = (-30.0, 12.0)

def output_gain(volume):
    """MAIN OUTPUT VOLUME's gain (engine.rs' output_gain: R20, 5K audio taper, into 10K)."""
    p = min(max(volume / 10.0, 0.0), 1.0)
    t = (81.0 ** p - 1.0) / 80.0
    top, bottom = 5e3 * (1.0 - t), 5e3 * t
    load = bottom * 10e3 / (bottom + 10e3) if bottom > 0 else 0.0
    return load / (top + load) if load > 0 else 0.0

def db_of(volume, level):
    g = output_gain(volume)
    return (20 * math.log10(g) if g > 0 else -math.inf) + level

def volume_level_for(db, volume=None):
    """VOLUME and LEVEL for a gain of `db`: VOLUME alone at or under unity, else VOLUME at 10
    and LEVEL the rest; with `volume` (a modulated VOLUME, kept), LEVEL the rest."""
    if volume is not None:
        return volume, round(min(max(db - db_of(volume, 0), LEVEL_RANGE[0]), LEVEL_RANGE[1]), 2)
    if db >= 0:
        return 10, round(min(db, LEVEL_RANGE[1]), 2)
    lo, hi = 0.0, 10.0
    for _ in range(60):
        mid = (lo + hi) / 2
        if db_of(mid, 0) < db:
            lo = mid
        else:
            hi = mid
    return round((lo + hi) / 2, 3), 0

def presets():
    """(name, generated, volume, level, volume modulated) of every factory preset, in the
    file's order."""
    text = FILE.read_text()
    at = text.find("\n" + MARKER)
    out = []
    for part, generated in ((text[:at] if at >= 0 else text, False),
                            (text[at:] if at >= 0 else "", True)):
        for block in part.split("[[preset]]")[1:]:
            name = json.loads(re.search(r'^name = (".*")$', block, re.M).group(1))
            vol = re.search(r"^volume = (\S+)$", block, re.M)
            lev = re.search(r"^level = (\S+)$", block, re.M)
            moved = any(float(v) != 0 for v in
                        re.findall(r"^\w+_to_volume = (\S+)$", block, re.M))
            out.append((name, generated, float(vol.group(1)) if vol else 10.0,
                        float(lev.group(1)) if lev else 0.0, moved))
    return out

def binary():
    """The preset_levels test's executable, built in release."""
    cmd = ["cargo", "test", "--release", "-p", "ca72-plugin", "--test", "preset_levels",
           "--no-run", "--message-format=json"]
    out = subprocess.run(cmd, cwd=ROOT, check=True, capture_output=True, text=True).stdout
    for line in out.splitlines():
        m = json.loads(line)
        if m.get("reason") == "compiler-artifact" and m.get("executable") \
                and m["target"]["name"] == "preset_levels":
            return m["executable"]
    sys.exit("no preset_levels executable")

def measure(exe, name, qualities):
    with tempfile.NamedTemporaryFile(suffix=".json") as f:
        env = dict(os.environ, CA72_ONLY=name, CA72_QUALITY=",".join(qualities),
                   CA72_LEVELS_JSON=f.name)
        subprocess.run([exe, "--ignored", "--exact", "each_preset_in_its_register",
                        "--nocapture"], cwd=ROOT / "crates/ca72-plugin", env=env, check=True,
                       capture_output=True)
        readings = json.loads(pathlib.Path(f.name).read_text())
    return readings[name]

def measure_all(names, qualities, jobs):
    exe = binary()
    with concurrent.futures.ThreadPoolExecutor(jobs) as pool:
        futures = {n: pool.submit(measure, exe, n, qualities) for n in names}
        return {n: f.result() for n, f in futures.items()}

def gain_for(reading, generated):
    """The change, dB. A preset already within -18 +/- 1.5 at every quality measured keeps its
    level, but one of 0.3.0's; the others are sent to -18 midway between LO and HI, then within
    the tolerance (less MARGIN) at ULTRA if they can be, then at LO and HI, which win."""
    if not generated and all(abs(x - TARGET) <= TOLERANCE for x in reading.values()):
        return 0.0
    g = TARGET - (reading["lo"] + reading["hi"]) / 2
    for qs in (["ultra"], ["lo", "hi"]):
        r = [reading[q] for q in qs if q in reading]
        if not r:
            continue
        low = max(TARGET - TOLERANCE + MARGIN - x for x in r)
        high = min(TARGET + TOLERANCE - MARGIN - x for x in r)
        if low <= high:
            g = min(max(g, low), high)
    return g

def write_existing(name, volume, level):
    text = FILE.read_text()
    start = text.index(f'name = {json.dumps(name)}\n')
    end = text.find("[[preset]]", start)
    end = len(text) if end < 0 else end
    block = text[start:end]
    for key, v in (("volume", volume), ("level", level)):
        line = f"{key} = {v:g}"
        if re.search(rf"^{key} = \S+$", block, re.M):
            block = re.sub(rf"^{key} = \S+$", line, block, count=1, flags=re.M)
        else:
            block = block.replace("[preset.values]\n", f"[preset.values]\n{line}\n", 1)
    FILE.write_text(text[:start] + block + text[end:])

def main():
    a = argparse.ArgumentParser()
    a.add_argument("--qualities", default="lo,hi,ultra")
    a.add_argument("--passes", type=int, default=3)
    a.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2) - 2))
    a.add_argument("--only", nargs="*")
    a.add_argument("--check", action="store_true")
    a.add_argument("--report")
    args = a.parse_args()
    qualities = args.qualities.split(",")
    assert "lo" in qualities and "hi" in qualities, "LO and HI are measured"
    chosen = [p for p in presets() if not args.only or p[0] in args.only]
    names = [p[0] for p in chosen]
    first = measure_all(names, qualities, args.jobs)
    now, changed = first, {}
    for n in range(args.passes if not args.check else 0):
        levels = json.loads(LEVELS.read_text()) if LEVELS.exists() else {}
        moves = []
        for name, generated, volume, level, modulated in presets():
            if name not in now:
                continue
            g = gain_for(now[name], generated)
            if abs(g) < CLOSE:
                continue
            v, l = volume_level_for(db_of(volume, level) + g, volume if modulated else None)
            if (v, l) == (volume, level):
                continue
            moves.append(name)
            changed[name] = (volume, level, v, l) if name not in changed \
                else changed[name][:2] + (v, l)
            if generated:
                levels[name] = {"volume": v, "level": l}
            else:
                write_existing(name, v, l)
        if not moves:
            break
        LEVELS.write_text(json.dumps(levels, indent=1) + "\n")
        subprocess.run([sys.executable, str(GENERATOR)], cwd=ROOT, check=True)
        print(f"pass {n + 1}: moved {', '.join(moves)}", flush=True)
        now = {**now, **measure_all(moves, qualities, args.jobs)}
    rows = ["| Preset | " + " | ".join(q.upper() for q in qualities)
            + " | VOLUME, LEVEL |", "|---|" + "---|" * (len(qualities) + 1)]
    for name in names:
        cell = lambda r: " | ".join(f"{r[q]:.2f}" for q in qualities)
        if name in changed:
            v0, l0, v, l = changed[name]
            rows.append(f"| {name} | " + " | ".join(
                f"{first[name][q]:.2f} -> {now[name][q]:.2f}" for q in qualities)
                + f" | {v0:g}, {l0:g} -> {v:g}, {l:g} |")
        else:
            rows.append(f"| {name} | {cell(now[name])} | |")
    table = "\n".join(rows) + "\n"
    print(table)
    out = [n for n in names
           if any(abs(now[n][q] - TARGET) > TOLERANCE for q in qualities)]
    if out:
        print("outside -18 +/- 1.5:", ", ".join(out))
    if args.report:
        pathlib.Path(args.report).write_text(
            "Momentary maximum on each preset's phrase, LUFS (tests/preset_levels.rs).\n\n"
            + table)

main()
