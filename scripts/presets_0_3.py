"""0.3.0's factory presets (decisions.md R-PRESETS-0.3.0, R-OWNER-PRESETS;
docs/preset-candidates/README.md): those made for 0.3 and the owner's (scripts/presets_owner,
their files as their CA-72 saved them: their edits of 0.3's, their renamed LFO loops and Club
Stab, and three of their own), written into crates/ca72-plugin/sounds/presets.toml after its
marker line (everything after it is this script's), in the file's format, every panel value
named as the factory presets name them. The AUTO GAIN curves already there for a preset of the
same name are kept (they depend on neither VOLUME nor LEVEL; tests/factory_curves.rs writes
them). MAIN OUTPUT VOLUME and LEVEL come from scripts/presets_0_3_levels.json, which
scripts/level_presets.py writes from the measurement (tests/preset_levels.rs).

    python3 scripts/presets_0_3.py [repository root] [levels.json]
"""
import json, pathlib, re, sys, tomllib

ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else pathlib.Path(__file__).parent.parent)
LEVELS_FILE = (pathlib.Path(sys.argv[2]) if len(sys.argv) > 2
               else ROOT / "scripts/presets_0_3_levels.json")
LEVELS = json.loads(LEVELS_FILE.read_text()) if LEVELS_FILE.exists() else {}
OWNER = ROOT / "scripts/presets_owner"

# Indices. Footage: LO 32 16 8 4 2. Wave: tri shark saw square wide narrow.
LO, F32, F16, F8, F4, F2 = range(6)
TRI, SHARK, SAW, SQUARE, WIDE, NARROW = range(6)
# Division (timed oscillators and LFOs).
DIV = {"4bars": 0, "2bars": 1, "1bar": 2, "1/2d": 3, "1/1t": 4, "1/2": 5, "1/4d": 6, "1/2t": 7,
       "1/4": 8, "1/8d": 9, "1/4t": 10, "1/8": 11, "1/16d": 12, "1/8t": 13, "1/16": 14,
       "1/32d": 15, "1/16t": 16, "1/32": 17}
ARP_RATE = {"1/2": 0, "1/4d": 1, "1/2t": 2, "1/4": 3, "1/8d": 4, "1/4t": 5, "1/8": 6,
            "1/16d": 7, "1/8t": 8, "1/16": 9, "1/32d": 10, "1/16t": 11, "1/32": 12}
ARP_DIR = {"up": 0, "down": 1, "updown": 2, "random": 3, "order": 4}
FREE, RETRIG, ENV = range(3)
SHAPE_PLACES = 5.95  # CONTINUOUS's travel in the switch's steps (3 + 2.95)

def place(at):
    """CONTINUOUS's travel (0..1) for a place in the switch's steps."""
    return round(at / SHAPE_PLACES, 4)

PANEL = dict(
    tune=0, glide=0, mod_mix=0, osc_mod=0, osc3_control=1, decay_on=1, glide_on=0, mod_wheel=0,
    osc1_range=F8, osc1_waveform=SAW, osc1_on=1, osc1_volume=8,
    osc2_range=F8, osc2_waveform=SAW, osc2_frequency=0, osc2_on=0, osc2_volume=8,
    osc3_range=F8, osc3_waveform=SAW, osc3_frequency=0, osc3_on=0, osc3_volume=8,
    ext_on=0, ext_volume=5, noise_on=0, noise_volume=8, noise_type=0,
    filter_mode=0, filter_mod=0, keyboard_control_1=1, keyboard_control_2=0,
    cutoff=0, emphasis=0, contour_amount=5, filter_attack=0, filter_decay=4, filter_sustain=5,
    loudness_attack=0, loudness_decay=4, loudness_sustain=8,
    volume=10, main_output=1, a440=0, level=0, feedback=0,
    poly=0, voices=4, entropy=5, spread=0, inner=0, placement=0, unison=0, double=0,
    doubled=0, drive=0, auto_gain=1, lock=0, note_priority=0, retrigger=0,
)
LFO = dict(waveform=TRI, shape_mode=0, shape=0, rate=2.0, timed=0, division=DIV["1/4"],
           phase=0, fade=0, mode=FREE, mono=1, polarity=0, reverse=0, on=1, level=100)
ARP = dict(arp_on=1, arp_latch=0, arp_direction=0, arp_rate=ARP_RATE["1/16"], arp_octaves=1,
           arp_gate=50, arp_swing=0)

def lfo(n, to=None, edges=False, **kw):
    out = {f"lfo{n}_{k}": v for k, v in {**LFO, **kw}.items()}
    for target, depth in (to or {}).items():
        out[f"lfo{n}_to_{target}"] = depth
    if edges:
        out[f"lfo{n}_edges"] = 1
    return out

def macro(n, to, value=0):
    out = {f"macro{n}": value}
    for target, depth in to.items():
        out[f"macro{n}_to_{target}"] = depth
    return out

def arp(**kw):
    return {**ARP, **kw}

def velocity(to):
    """What a note's velocity moves (decisions.md R-VELOCITY), percent of each knob's travel."""
    return {f"velocity_to_{target}": depth for target, depth in to.items()}

def preset(name, description, tags, *parts, names=(), **values):
    v = dict(PANEL)
    for p in parts:
        v.update(p)
    v.update(values)
    if v["double"] > 0:
        v["doubled"] = 1
    if name in LEVELS:
        v.update(LEVELS[name])
    return dict(name=name, description=description, tags=tags, macro_names=list(names), values=v)

def owner(name, description=None, **changes):
    """The owner's preset, as their CA-72 saved it (scripts/presets_owner/<name>.toml, kept as
    it came: decisions.md R-OWNER-PRESETS): its values, every one, as they are, but `changes`
    (the owner's own, decided after); its tags and macros' names; its description, else
    `description`. An LFO's RISE, as 0.3.0's beta saved it, is its FADE, a fade in
    (decisions.md R-LFO-FADE). MAIN OUTPUT VOLUME and LEVEL as the others', from the levels file
    once levelled. Its AUTO GAIN curve is not taken: the factory measures its own
    (tests/factory_curves.rs)."""
    t = tomllib.loads((OWNER / f"{name}.toml").read_text())
    assert t["name"] == name, name
    effects = t.get("effects", {}).get("chain")
    assert effects is None or not json.loads(effects)["slots"], (
        f"{name}: factory presets must not include hosted effects (VST3 or CLAP)"
    )
    v = dict(t["values"])
    # (Saved by 0.3.0's beta, an LFO's RISE: its FADE now, a fade in, decisions.md R-LFO-FADE.)
    for n in range(1, 5):
        if f"lfo{n}_rise" in v:
            v.setdefault(f"lfo{n}_fade", v.pop(f"lfo{n}_rise"))
    assert set(changes) <= set(v), f"{name}: {set(changes) - set(v)}"
    v.update(changes)
    if name in LEVELS:
        v.update(LEVELS[name])
    return dict(name=name, description=t.get("description") or description or "",
                tags=list(t.get("tags", [])), macro_names=list(t.get("macro_names", [])),
                values=v)

P = [owner("Cycle of Fifths")]

# ---- Sounds the 0.2.8 set does not have.

P.append(preset(
    "Acid Squelch",
    "One sawtooth at 16' into a filter on the edge of singing, a short decay and a quick glide "
    "between held keys (NOTE PRIORITY LAST, single triggering): a squelchy acid line. MACRO 1 "
    "(SQUELCH) pushes the emphasis and the contour; MACRO 2 (DECAY) lengthens the filter's decay.",
    ["bass", "acid", "glide", "macros"],
    macro(1, {"emphasis": 18, "contour_amount": 30}),
    macro(2, {"filter_decay": 35}),
    names=["SQUELCH", "DECAY"],
    osc1_range=F16, osc1_waveform=SAW, osc1_volume=10,
    cutoff=-3.2, emphasis=7.0, contour_amount=6.5, filter_decay=2.6, filter_sustain=0.5,
    loudness_decay=3, loudness_sustain=8,
    glide_on=1, glide=1.2, note_priority=1, drive=9,
))

P.append(owner("Tonewheel Drift"))

P.append(owner("Glass Bell"))

P.append(preset(
    "Night Surf",
    "Pink noise alone through a resonant filter, four UNISON instruments spread wide: slow LFOs "
    "roll the filter, the level and the emphasis in and out of step, and a fourth carries it "
    "across the field. Waves on a beach at night. Hold one key.",
    ["fx", "ambient", "noise", "lfo"],
    lfo(1, {"cutoff": 25}, rate=0.09),
    lfo(2, {"volume": -20}, rate=0.13, phase=120),
    lfo(3, {"emphasis": 15}, rate=0.21, phase=240),
    lfo(4, {"spread": 40}, rate=0.07),
    osc1_on=0, noise_on=1, noise_volume=10, noise_type=1,
    cutoff=-1, emphasis=5.5, contour_amount=0, keyboard_control_1=0,
    loudness_attack=6, loudness_decay=6, loudness_sustain=10, volume=8,
    unison=1, voices=4, entropy=60, spread=50,
))

P.append(preset(
    "Ray Gun",
    "Oscillators 2 and 3 at 4' falling fifteen semitones in a quarter of a second: an LFO in ENV "
    "mode, a falling sawtooth, sweeps both FREQUENCY knobs from the top down to where they are "
    "set, RETRIGGER firing it on every key. A ray gun. MACRO 1 (SWEEP) slows the fall.",
    ["fx", "zap", "percussive", "lfo", "macros"],
    lfo(1, {"osc2_frequency": 100, "osc3_frequency": 100},
        waveform=SAW, reverse=1, polarity=1, mode=ENV, rate=4.0),
    macro(1, {"lfo1_rate": -20}),
    names=["SWEEP"],
    osc1_on=0,
    osc2_on=1, osc2_range=F4, osc2_waveform=SQUARE, osc2_frequency=-7.5, osc2_volume=10,
    osc3_on=1, osc3_range=F4, osc3_waveform=SAW, osc3_frequency=-7.0, osc3_volume=7,
    cutoff=2, emphasis=6, contour_amount=3, filter_decay=2, filter_sustain=0,
    loudness_decay=2.5, loudness_sustain=0, retrigger=1,
))

P.append(preset(
    "Pitch Tom",
    "A triangle at 16' with a tick of white noise, its pitch dropping into the note as it is "
    "struck (an LFO in ENV mode on oscillator 2's FREQUENCY): the CA-72's first tom. Play it down "
    "the keys for low, middle and high toms.",
    ["drums", "tom", "percussive", "lfo"],
    lfo(1, {"osc2_frequency": 30},
        waveform=SAW, reverse=1, polarity=1, mode=ENV, rate=3.0),
    osc1_on=0,
    osc2_on=1, osc2_range=F16, osc2_waveform=TRI, osc2_volume=10,
    noise_on=1, noise_volume=2,
    cutoff=0, emphasis=3, contour_amount=4, filter_decay=1.2, filter_sustain=0,
    keyboard_control_2=1, loudness_decay=3.2, loudness_sustain=0, retrigger=1,
))

P.append(preset(
    "Deep Boom",
    "Triangles at 32' and 16' dropping three semitones into each note (an LFO in ENV mode), "
    "driven 15 dB and left to ring out, gliding between held keys: a long, saturated sub bass "
    "for hip-hop and trap lines. MACRO 1 (GROWL) drives it harder.",
    ["bass", "sub", "glide", "lfo", "macros"],
    lfo(1, {"osc2_frequency": 16, "osc3_frequency": 16},
        waveform=SAW, reverse=1, polarity=1, mode=ENV, rate=6.0),
    macro(1, {"drive": 35}),
    names=["GROWL"],
    osc1_on=0,
    osc2_on=1, osc2_range=F32, osc2_waveform=TRI, osc2_volume=10,
    osc3_on=1, osc3_range=F16, osc3_waveform=TRI, osc3_volume=4,
    cutoff=-1.5, contour_amount=2, filter_decay=1.5, filter_sustain=3,
    loudness_decay=6, loudness_sustain=0,
    glide_on=1, glide=0.6, note_priority=1, retrigger=1, drive=15,
))

# (Club Stab, the owner's, renamed: decisions.md R-OWNER-PRESETS.)
P.append(owner(
    "Zither Me Timbres",
    "Two sawtooths and a square at 4', a quick filter blip and no sustain, POLY's eight voices "
    "spread wide: a plucked chord stab. Struck harder, its contour opens further and it rings "
    "longer (velocity on AMOUNT OF CONTOUR and the loudness DECAY). MACRO 1 (OPEN) opens the "
    "filter and lengthens it; MACRO 2 (WIDE) spreads the voices and DOUBLE further.",
))

# ---- The arpeggiator.

P.append(preset(
    "Octave Runner",
    "The arpeggiator at sixteenths over two octaves, a short GATE, on a sawtooth and a square at "
    "16'; a timed LFO opens and closes the filter over two bars. Hold one key: an octave-bouncing "
    "disco bass. MACRO 1 (BITE) adds emphasis and contour.",
    ["bass", "arp", "lfo", "macros"],
    arp(arp_direction=ARP_DIR["up"], arp_rate=ARP_RATE["1/16"], arp_octaves=2, arp_gate=35),
    lfo(1, {"cutoff": 18}, timed=1, division=DIV["2bars"]),
    macro(1, {"emphasis": 25, "contour_amount": 20}),
    names=["BITE"],
    osc1_range=F16, osc1_volume=10,
    osc2_on=1, osc2_range=F16, osc2_waveform=SQUARE, osc2_volume=6, osc2_frequency=0.05,
    cutoff=-2.5, emphasis=4.5, contour_amount=5, filter_decay=1.8, filter_sustain=1,
    loudness_decay=2, loudness_sustain=6, note_priority=1, retrigger=1,
))

P.append(preset(
    "Random Glass",
    "The arpeggiator in RANDOM order over three octaves at sixteenth-note triplets, HOLD on: play "
    "a chord, let go, and it goes on. A triangle and a CONTINUOUS pulse whose width a four-bar "
    "LFO turns, through a filter that follows the keys and a one-bar sweep. Glassy, generative "
    "plucks.",
    ["arp", "hold", "ambient", "plucked", "lfo"],
    arp(arp_latch=1, arp_direction=ARP_DIR["random"], arp_rate=ARP_RATE["1/16t"],
        arp_octaves=3, arp_gate=25),
    lfo(1, {"cutoff": 12}, timed=1, division=DIV["1bar"]),
    lfo(2, {"osc2_shape": 25}, timed=1, division=DIV["4bars"]),
    osc1_range=F8, osc1_waveform=TRI, osc1_volume=9,
    osc2_on=1, osc2_range=F4, osc2_volume=5,
    osc2_shape_mode=1, osc2_shape=place(3.3),
    cutoff=0, emphasis=5, contour_amount=4, filter_decay=1.6, filter_sustain=0,
    keyboard_control_2=1, loudness_decay=3.5, loudness_sustain=0,
    double=6, spread=40,
))

P.append(owner("Swing Keys"))

# ---- The LFOs, the macros, the timed and CONTINUOUS oscillators, pressure.

P.append(preset(
    "Tempo Wobble",
    "Sawtooth, square and a 32' triangle sub, driven 10 dB, with two timed LFOs on the filter: "
    "MACRO 1 (WOB 8) fades in an eighth-note wobble, set most of the way up, and MACRO 2 (WOB 16) "
    "a sixteenth-note one, so the two can be crossfaded as it plays; MACRO 3 (GROWL) adds "
    "emphasis and drive. Both restart on every note.",
    ["bass", "wobble", "lfo", "macros"],
    lfo(1, {"cutoff": 35}, timed=1, division=DIV["1/8"], mode=RETRIG, level=0),
    lfo(2, {"cutoff": 35}, timed=1, division=DIV["1/16"], mode=RETRIG, level=0),
    macro(1, {"lfo1_level": 100}, value=80),
    macro(2, {"lfo2_level": 100}),
    macro(3, {"emphasis": 20, "drive": 25}),
    names=["WOB 8", "WOB 16", "GROWL"],
    osc1_range=F16, osc1_volume=10,
    osc2_on=1, osc2_range=F16, osc2_waveform=SQUARE, osc2_volume=8, osc2_frequency=-0.06,
    osc3_on=1, osc3_range=F32, osc3_waveform=TRI, osc3_volume=8,
    cutoff=-3, emphasis=5.5, contour_amount=1.5, filter_decay=4, filter_sustain=8,
    loudness_decay=1.5, loudness_sustain=10, drive=10,
))

P.append(preset(
    "Pulse Strings",
    "Two pulses in CONTINUOUS between SQUARE and WIDE, their widths swept by each voice's own "
    "oscillator 3 at LO through MOD (one narrowing, one both ways), a slow swell: a classic "
    "pulse-width string ensemble, POLY's eight voices drifting apart. The mod wheel sets how far "
    "the widths move.",
    ["strings", "poly", "pwm", "slow"],
    osc1_volume=8, osc1_shape_mode=1, osc1_shape=place(3.6), osc1_shape_mod=1,
    osc1_shape_mod_dir=0,
    osc2_on=1, osc2_volume=8, osc2_frequency=0.07, osc2_shape_mode=1, osc2_shape=place(3.6),
    osc2_shape_mod=1, osc2_shape_mod_dir=2,
    osc3_range=LO, osc3_waveform=TRI, osc3_frequency=-6, osc3_control=0,
    mod_mix=0, mod_wheel=0.55,
    cutoff=-0.5, emphasis=1, contour_amount=2.5, filter_attack=5, filter_decay=6,
    filter_sustain=7, loudness_attack=4.5, loudness_decay=6, loudness_sustain=10,
    poly=1, voices=8, entropy=35, spread=60,
))

P.append(preset(
    "Gated Chords",
    "Two sawtooths a hair apart, POLY's eight voices, chopped into sixteenths by oscillator 3 "
    "TIMED at LO (a square) through FILTER MODULATION, in step with the host's tempo, and a "
    "two-bar LFO sweeping the filter: a trance-gate pad. The mod wheel sets how hard it chops.",
    ["pad", "poly", "gate", "tempo", "lfo"],
    lfo(1, {"cutoff": 12}, timed=1, division=DIV["2bars"]),
    osc1_volume=9, osc2_on=1, osc2_volume=9, osc2_frequency=0.1,
    osc3_range=LO, osc3_waveform=SQUARE, osc3_timed=1, osc3_division=DIV["1/16"],
    osc3_control=0, filter_mod=1, mod_mix=0, mod_wheel=0.8,
    cutoff=-5, emphasis=3.5, contour_amount=1, filter_attack=2, filter_decay=6,
    filter_sustain=6, loudness_attack=1, loudness_decay=5, loudness_sustain=10,
    poly=1, voices=8, entropy=15, spread=55, double=9,
))

# (The owner's, saved from Gated Chords as it was before 0.3.0 made it gate harder:
# decisions.md R-OWNER-PRESETS.)
P.append(owner(
    "Gatekeeper",
    "Two sawtooths a hair apart, POLY's eight voices, chopped into sixteenths by oscillator 3 "
    "TIMED at LO (a square) through FILTER MODULATION, in step with the host's tempo, and a "
    "two-bar LFO sweeping the filter: Gated Chords with the filter further open, so the gate "
    "is gentler. The mod wheel sets how hard it chops.",
))

P.append(preset(
    "Pressure Solo",
    "A sawtooth and a square at 8', gliding, with a vibrato that rises in after each note starts "
    "(an LFO on TUNE, FADE in about a second). Pressure, a note's own (MPE) or the keyboard's, opens "
    "the filter up to an octave and a half; MPE's slide (timbre) adds a noisy growl through the "
    "mod wheel. An expressive solo lead.",
    ["lead", "expressive", "glide", "lfo"],
    lfo(1, {"tune": 3}, rate=5.4, fade=0.9, mode=RETRIG),
    pressure_filter=1.5, timbre_mod=70,
    osc1_volume=10, osc2_on=1, osc2_waveform=SQUARE, osc2_volume=6, osc2_frequency=0.04,
    mod_mix=10, filter_mod=1, mod_wheel=0,
    cutoff=-1.5, emphasis=4.5, contour_amount=4, filter_attack=0.6, filter_decay=4,
    filter_sustain=6, loudness_attack=0.4, loudness_decay=3, loudness_sustain=10,
    glide_on=1, glide=2.2, note_priority=1, drive=8,
))

P.append(preset(
    "Morph Bloom",
    "Two CONTINUOUS waveforms between shark tooth and sawtooth, each voice's own LFOs turning "
    "them, and a 4' triangle above: a slow pad for four macros. MACRO 1 (BRIGHT) opens the "
    "filter and the 4'; MACRO 2 (DRIFT) pulls the voices apart; MACRO 3 (MOTION) fades in a slow "
    "filter sweep; MACRO 4 (FAST) shortens the swells. All four start at nothing.",
    ["pad", "poly", "slow", "lfo", "macros"],
    lfo(1, {"osc1_shape": 12}, rate=0.31, mono=0),
    lfo(2, {"osc2_shape": 12}, rate=0.23, phase=180, mono=0),
    lfo(3, {"cutoff": 20}, rate=0.12, level=0),
    macro(1, {"cutoff": 30, "emphasis": 15, "osc3_volume": 30}),
    macro(2, {"entropy": 50, "double": 25, "spread": 30}),
    macro(3, {"lfo3_level": 100}),
    macro(4, {"loudness_attack": -40, "filter_attack": -40}),
    names=["BRIGHT", "DRIFT", "MOTION", "FAST"],
    osc1_volume=8, osc1_shape_mode=1, osc1_shape=place(1.5),
    osc2_on=1, osc2_volume=8, osc2_frequency=0.09, osc2_shape_mode=1, osc2_shape=place(1.3),
    osc3_on=1, osc3_range=F4, osc3_waveform=TRI, osc3_volume=3,
    cutoff=-1.5, emphasis=2.5, contour_amount=3, filter_attack=6, filter_decay=7,
    filter_sustain=6, loudness_attack=5.5, loudness_decay=7, loudness_sustain=10,
    poly=1, voices=8, entropy=20, spread=50, inner=20, double=6,
))

P.append(owner("Bubble Computer"))

# ---- The owner's (decisions.md R-OWNER-PRESETS).

P.append(owner(
    "Velocihator",
    "The hat's noise, short, played by velocity: struck harder, it is louder and brighter and "
    "rings longer (velocity on VOLUME, CUTOFF, the filter's DECAY and the loudness DECAY). "
    "Any key.",
))

P.append(owner(
    "Droning On",
    "Wide rectangles at 8', 4' and 2', oscillators 2 and 3 tuned apart, UNISON's four "
    "instruments spread across the field, DOUBLE and FEEDBACK thickening them; timed LFOs "
    "turn the first's width and the pitch over bars, a slow one the filter, and HOLD keeps "
    "the notes sounding until the next. A dark drone.",
    ext_on=0,  # (EXTERNAL INPUT off: the owner, 2026-10-28.)
))


# ---- The LFO loops (decisions.md R-LFO's addendum): the LFOs moving each other's knobs and
# their own. 0.3.0's four LFO LOOP presets, as the owner renamed them (R-OWNER-PRESETS).

P.append(owner(
    "Finger in a Light Socket",
    "Two LFOs each setting the other's speed, a loop: LFO 1 (TRIANGLE, 1.2 Hz) sweeps CUTOFF; "
    "LFO 2 (TRIANGLE, 0.3 Hz) pushes LFO 1's RATE up and down (+35 %) while LFO 1 pulls LFO 2's "
    "(-30 %), so the sweep speeds up, slows down and never quite repeats; DOUBLE on and the "
    "voices spread. Hold a chord.",
))

P.append(owner(
    "Phase Chase",
    "Two LFOs at the same speed pushing each other's PHASE, a loop: LFO 1 (SAWTOOTH, 2 Hz) on "
    "CUTOFF, LFO 3 (TRIANGLE, 2 Hz) on OSCILLATOR 1's VOLUME, each moving the other's PHASE by "
    "+40 %. The rhythm leans, stumbles and catches itself. NOTE PRIORITY LAST. Hold one note.",
))

P.append(owner(
    "Level Pump",
    "An LFO fading another in and out by its LEVEL: LFO 1 (SQUARE, 4 Hz, SOFT) pumps MAIN "
    "OUTPUT VOLUME (-45 %) and CUTOFF; LFO 2 (TRIANGLE, 0.15 Hz) takes LFO 1's LEVEL down "
    "(-100 %) and back, and LFO 1 dents LFO 2's own LEVEL (-20 %), a loop. The pumping swells in "
    "and dies away. Hold a chord.",
))

P.append(owner(
    "Hold Me Long Time",
    "An LFO on its own RATE: LFO 1 (the shark tooth, 0.8 Hz, RETRIG) on CUTOFF and OSCILLATOR "
    "2's FREQUENCY, and on its own RATE (+60 %), so it races up its rise and drags down its "
    "fall; RETRIGGER on, each note starts it again the same. Play notes.",
))


def fmt(x):
    if isinstance(x, float):
        s = f"{x:.4f}".rstrip("0").rstrip(".")
        return s if s not in ("-0", "") else "0"
    return str(x)

def q(s):
    return json.dumps(s, ensure_ascii=False)

def table(p, curves):
    lines = ["", "[[preset]]"]
    lines += [f"name = {q(p['name'])}", f"description = {q(p['description'])}",
              "tags = [" + ", ".join(q(t) for t in p["tags"]) + "]"]
    lines += curves
    if p["macro_names"]:
        lines.append("macro_names = [" + ", ".join(q(n) for n in p["macro_names"]) + "]")
    lines += ["", "[preset.values]"]
    lines += [f"{k} = {fmt(v)}" for k, v in p["values"].items()]
    return "\n".join(lines) + "\n"

FILE = ROOT / "crates/ca72-plugin/sounds/presets.toml"
MARKER = "# ---- 0.3.0's presets"
HEAD = MARKER + """: generated from here to the end by scripts/presets_0_3.py (edit them there).
# Those made for 0.3 (sounds the 0.2.8 set left out, and the arpeggiator, the LFOs, the macros,
# velocity, the timed and CONTINUOUS oscillators and pressure in use) and the owner's (their
# edits, their own, and the LFO loops as they renamed them: the LFOs moving each other's knobs
# and their own). Levelled as the rest, -18 LUFS midway between LO and HI
# (scripts/level_presets.py; decisions.md R-PRESETS-0.3.0, R-OWNER-PRESETS).
"""

text = FILE.read_text()
at = text.find("\n" + MARKER)
before, after = (text, "") if at < 0 else (text[:at + 1], text[at + 1:])
# The curves already measured for each, by name.
curves, name = {}, None
for line in after.splitlines():
    if line.startswith("name = "):
        name = json.loads(line[len("name = "):])
    elif line.startswith("drive_curve") and name:
        curves.setdefault(name, []).append(line)
names = [p["name"] for p in P]
assert len(set(n.lower() for n in names)) == len(names), "a name twice"
taken = set(n.lower() for n in re.findall(r'^name = "(.*)"$', before, re.M))
assert not taken & set(n.lower() for n in names), "a factory preset's name taken"
FILE.write_text(before.rstrip("\n") + "\n\n" + HEAD
                + "".join(table(p, curves.get(p["name"], [])) for p in P))
print(len(P), "presets written into", FILE.relative_to(ROOT))
