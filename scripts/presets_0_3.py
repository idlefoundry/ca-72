"""0.3.0's factory presets (decisions.md R-PRESETS-0.3.0; docs/preset-candidates/README.md):
the seventeen made for 0.3 and the four LFO LOOP presets, written into
crates/ca72-plugin/sounds/presets.toml after its marker line (everything after it is this
script's), in the file's format, every panel value named as the factory presets name them. The
AUTO GAIN curves already there for a preset of the same name are kept (they depend on neither
VOLUME nor LEVEL; tests/factory_curves.rs writes them). MAIN OUTPUT VOLUME and LEVEL come from
scripts/presets_0_3_levels.json, which scripts/level_presets.py writes from the measurement
(tests/preset_levels.rs).

    python3 scripts/presets_0_3.py [repository root] [levels.json]
"""
import json, pathlib, re, sys

ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else pathlib.Path(__file__).parent.parent)
LEVELS_FILE = (pathlib.Path(sys.argv[2]) if len(sys.argv) > 2
               else ROOT / "scripts/presets_0_3_levels.json")
LEVELS = json.loads(LEVELS_FILE.read_text()) if LEVELS_FILE.exists() else {}

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
           phase=0, rise=0, mode=FREE, mono=1, polarity=0, reverse=0, on=1, level=100)
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

P = []

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

P.append(preset(
    "Tonewheel Drift",
    "Triangles at 16', 8' and 4' with a click of contour, POLY's eight voices: a soft drawbar "
    "organ. Three LFOs turn it as a rotating speaker turns (the level, the filter and the width); "
    "MACRO 1 (ROTOR) spins them up from slow to fast, gliding as a rotor does; MACRO 2 (PERC) adds "
    "a percussive bite.",
    ["organ", "keys", "poly", "lfo", "macros"],
    lfo(1, {"volume": -8}, rate=0.8),
    lfo(2, {"cutoff": 6}, rate=0.75, phase=90),
    lfo(3, {"spread": 30}, rate=0.85, phase=180),
    macro(1, {"lfo1_rate": 23, "lfo2_rate": 23, "lfo3_rate": 23}),
    macro(2, {"contour_amount": 40, "emphasis": 20}),
    names=["ROTOR", "PERC"],
    osc1_range=F16, osc1_waveform=TRI, osc1_volume=7,
    osc2_on=1, osc2_range=F8, osc2_waveform=TRI, osc2_volume=8,
    osc3_on=1, osc3_range=F4, osc3_waveform=TRI, osc3_volume=5,
    cutoff=2, keyboard_control_2=1, contour_amount=1.5, filter_decay=0.4, filter_sustain=0,
    loudness_decay=0.8, loudness_sustain=10, volume=9,
    poly=1, voices=8, entropy=15, spread=40,
))

P.append(preset(
    "Glass Bell",
    "A triangle at 8' and a quieter one at 4', both frequency-modulated by oscillator 3 at 2' "
    "(OSCILLATOR MODULATION, the mod wheel a third forward) a little over three octaves above, "
    "struck and left to ring: an inharmonic glass bell, POLY's eight voices across the field, "
    "TUNE a little down to take back the pitch the modulation adds; struck harder, it rings "
    "brighter (velocity on CUTOFF). MACRO 1 (CLANG) retunes oscillator 3 for a harsher strike.",
    ["bell", "poly", "velocity", "macros"],
    macro(1, {"osc3_frequency": 12}),
    velocity({"cutoff": 12}),
    names=["CLANG"],
    osc1_range=F8, osc1_waveform=TRI, osc1_volume=9,
    osc2_on=1, osc2_range=F4, osc2_waveform=TRI, osc2_volume=4,
    osc3_range=F2, osc3_waveform=TRI, osc3_frequency=-2.5, osc3_on=0,
    osc_mod=1, mod_mix=0, mod_wheel=0.3, tune=-0.1,
    cutoff=1, emphasis=2, contour_amount=3, filter_decay=5, filter_sustain=0,
    keyboard_control_2=1, loudness_decay=7, loudness_sustain=0,
    poly=1, voices=8, entropy=10, spread=40,
))

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

P.append(preset(
    "Club Stab",
    "Two sawtooths and a square at 4', a quick filter blip and no sustain, POLY's eight voices "
    "spread wide: a chord stab for house and garage, brighter the harder it is struck "
    "(velocity on CUTOFF). MACRO 1 (OPEN) opens the filter and lengthens it; MACRO 2 (WIDE) "
    "spreads the voices and DOUBLE further.",
    ["stab", "keys", "poly", "velocity", "macros"],
    macro(1, {"cutoff": 25, "filter_decay": 20}),
    macro(2, {"spread": 50, "double": 20}),
    velocity({"cutoff": 15}),
    names=["OPEN", "WIDE"],
    osc1_volume=10, osc2_on=1, osc2_volume=9, osc2_frequency=0.08,
    osc3_on=1, osc3_range=F4, osc3_waveform=SQUARE, osc3_volume=5,
    cutoff=-1.5, emphasis=3.5, contour_amount=5.5, filter_decay=2.4, filter_sustain=1.5,
    loudness_decay=3.4, loudness_sustain=0,
    poly=1, voices=8, entropy=10, spread=45, double=8,
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

P.append(preset(
    "Swing Keys",
    "The arpeggiator UP and DOWN at swung eighths through the chord held, on a mellow triangle "
    "with a quiet 4' tine above it, POLY so each note rings into the next, with a gentle "
    "tremolo. MACRO 1 (TINE) brings the tine and the contour forward.",
    ["arp", "keys", "poly", "lfo", "macros"],
    arp(arp_direction=ARP_DIR["updown"], arp_rate=ARP_RATE["1/8"], arp_octaves=1,
        arp_gate=70, arp_swing=60),
    lfo(1, {"volume": -6}, rate=4.2),
    macro(1, {"osc2_volume": 40, "contour_amount": 15}),
    names=["TINE"],
    osc1_waveform=TRI, osc1_volume=9,
    osc2_on=1, osc2_range=F4, osc2_waveform=TRI, osc2_volume=4,
    cutoff=-0.5, keyboard_control_2=1, contour_amount=3.5, filter_decay=2.4, filter_sustain=2,
    loudness_decay=4.4, loudness_sustain=2, volume=9,
    poly=1, voices=6, entropy=20, spread=50,
))

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

P.append(preset(
    "Pressure Solo",
    "A sawtooth and a square at 8', gliding, with a vibrato that rises in after each note starts "
    "(an LFO on TUNE, RISE about a second). Pressure, a note's own (MPE) or the keyboard's, opens "
    "the filter up to an octave and a half; MPE's slide (timbre) adds a noisy growl through the "
    "mod wheel. An expressive solo lead.",
    ["lead", "expressive", "glide", "lfo"],
    lfo(1, {"tune": 3}, rate=5.4, rise=0.9, mode=RETRIG),
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

P.append(preset(
    "Bubble Computer",
    "A square into a ringing filter that four timed LFOs step about: sixteenths, chopped by "
    "dotted eighths, against quarter-note triplets, under a two-bar swell of emphasis. A 1970s "
    "film computer thinking aloud, in time with the host. Hold one key.",
    ["fx", "tempo", "lfo"],
    lfo(1, {"cutoff": 30}, waveform=SQUARE, timed=1, division=DIV["1/16"], edges=True),
    lfo(2, {"lfo1_level": -70}, timed=1, division=DIV["1/8d"]),
    lfo(3, {"cutoff": 15}, waveform=SAW, timed=1, division=DIV["1/4t"]),
    lfo(4, {"emphasis": 10}, timed=1, division=DIV["2bars"]),
    osc1_waveform=SQUARE, osc1_volume=6,
    cutoff=-2, emphasis=8.4, contour_amount=0, keyboard_control_2=1,
    loudness_sustain=10,
))


# ---- The LFO LOOP presets (decisions.md R-LFO's addendum): the LFOs moving each other's knobs
# and their own. Every value named, as they were made (docs/lfo-loops until 0.3.0).

LOOP_VALUES, LOOP_DESCRIPTIONS = {}, {}
LOOP_TAGS = {
    "LFO LOOP Coupled Rates": ["pad", "poly", "lfo", "loop"],
    "LFO LOOP Phase Chase": ["lead", "lfo", "loop"],
    "LFO LOOP Level Pump": ["pad", "poly", "lfo", "loop"],
    "LFO LOOP Self Rate": ["lead", "lfo", "loop"],
}

LOOP_VALUES['LFO LOOP Coupled Rates'] = {
    'a440': 0, 'auto_gain': 0, 'contour_amount': 3, 'cutoff': 0.5, 'decay_on': 1, 'double': 0,
    'doubled': 0, 'drive': 0, 'emphasis': 3, 'entropy': 5, 'ext_on': 0, 'feedback': 0,
    'filter_attack': 0, 'filter_decay': 4, 'filter_mod': 0, 'filter_mode': 0, 'filter_sustain': 6,
    'glide': 0, 'glide_on': 0, 'inner': 0, 'keyboard_control_1': 1, 'keyboard_control_2': 0,
    'level': 7, 'lfo1_level': 100, 'lfo1_mode': 0, 'lfo1_mono': 1, 'lfo1_phase': 0,
    'lfo1_rate': 1.2, 'lfo1_rise': 0.0, 'lfo1_timed': 0, 'lfo1_to_cutoff': 30,
    'lfo1_to_lfo2_rate': -30, 'lfo1_waveform': 0, 'lfo2_level': 100, 'lfo2_mode': 0,
    'lfo2_mono': 1, 'lfo2_phase': 0, 'lfo2_rate': 0.3, 'lfo2_rise': 0.0, 'lfo2_timed': 0,
    'lfo2_to_lfo1_rate': 35, 'lfo2_to_osc2_frequency': 2, 'lfo2_waveform': 0, 'lock': 0,
    'loudness_attack': 1, 'loudness_decay': 5, 'loudness_sustain': 10, 'main_output': 1,
    'mod_mix': 0, 'mod_wheel': 0, 'noise_on': 0, 'note_priority': 0, 'osc1_on': 1,
    'osc1_range': 3, 'osc1_volume': 6, 'osc1_waveform': 2, 'osc2_frequency': 0.08, 'osc2_on': 1,
    'osc2_range': 3, 'osc2_volume': 6, 'osc2_waveform': 2, 'osc3_frequency': 0, 'osc3_on': 0,
    'osc3_range': 3, 'osc3_volume': 6, 'osc3_waveform': 2, 'osc_mod': 0, 'poly': 1,
    'retrigger': 0, 'spread': 0, 'unison': 0, 'voices': 6, 'volume': 7,
}
LOOP_DESCRIPTIONS['LFO LOOP Coupled Rates'] = "Two LFOs each setting the other's speed, a loop: LFO 1 (TRIANGLE, 1.2 Hz) sweeps CUTOFF; LFO 2 (TRIANGLE, 0.3 Hz) pushes LFO 1's RATE up and down (+35 %) while LFO 1 pulls LFO 2's (-30 %), so the sweep speeds up, slows down and never quite repeats. Hold a chord."

LOOP_VALUES['LFO LOOP Phase Chase'] = {
    'a440': 0, 'auto_gain': 0, 'contour_amount': 3, 'cutoff': 1.5, 'decay_on': 1, 'double': 0,
    'doubled': 0, 'drive': 0, 'emphasis': 3, 'entropy': 5, 'ext_on': 0, 'feedback': 0,
    'filter_attack': 0, 'filter_decay': 4, 'filter_mod': 0, 'filter_mode': 0, 'filter_sustain': 6,
    'glide': 0, 'glide_on': 0, 'inner': 0, 'keyboard_control_1': 1, 'keyboard_control_2': 0,
    'level': 12, 'lfo1_level': 100, 'lfo1_mode': 0, 'lfo1_mono': 1, 'lfo1_phase': 0,
    'lfo1_rate': 2.0, 'lfo1_rise': 0.0, 'lfo1_timed': 0, 'lfo1_to_cutoff': 25,
    'lfo1_to_lfo3_phase': 40, 'lfo1_waveform': 2, 'lfo3_level': 100, 'lfo3_mode': 0,
    'lfo3_mono': 1, 'lfo3_phase': 90, 'lfo3_rate': 2.0, 'lfo3_rise': 0.0, 'lfo3_timed': 0,
    'lfo3_to_lfo1_phase': 40, 'lfo3_to_osc1_volume': -25, 'lfo3_waveform': 0, 'lock': 0,
    'loudness_attack': 1, 'loudness_decay': 5, 'loudness_sustain': 10, 'main_output': 1,
    'mod_mix': 0, 'mod_wheel': 0, 'noise_on': 0, 'note_priority': 0, 'osc1_on': 1,
    'osc1_range': 3, 'osc1_volume': 6, 'osc1_waveform': 2, 'osc2_frequency': 0.08, 'osc2_on': 1,
    'osc2_range': 3, 'osc2_volume': 6, 'osc2_waveform': 2, 'osc3_frequency': 0, 'osc3_on': 0,
    'osc3_range': 3, 'osc3_volume': 6, 'osc3_waveform': 2, 'osc_mod': 0, 'poly': 0,
    'retrigger': 0, 'spread': 0, 'unison': 0, 'voices': 4, 'volume': 7,
}
LOOP_DESCRIPTIONS['LFO LOOP Phase Chase'] = "Two LFOs at the same speed pushing each other's PHASE, a loop: LFO 1 (SAWTOOTH, 2 Hz) on CUTOFF, LFO 3 (TRIANGLE, 2 Hz) on OSCILLATOR 1's VOLUME, each moving the other's PHASE by +40 %. The rhythm leans, stumbles and catches itself. Hold one note."

LOOP_VALUES['LFO LOOP Level Pump'] = {
    'a440': 0, 'auto_gain': 0, 'contour_amount': 3, 'cutoff': 0.5, 'decay_on': 1, 'double': 0,
    'doubled': 0, 'drive': 0, 'emphasis': 3, 'entropy': 5, 'ext_on': 0, 'feedback': 0,
    'filter_attack': 0, 'filter_decay': 4, 'filter_mod': 0, 'filter_mode': 0, 'filter_sustain': 6,
    'glide': 0, 'glide_on': 0, 'inner': 0, 'keyboard_control_1': 1, 'keyboard_control_2': 0,
    'level': -2, 'lfo1_level': 100, 'lfo1_mode': 0, 'lfo1_mono': 1, 'lfo1_phase': 0,
    'lfo1_rate': 4.0, 'lfo1_rise': 0.0, 'lfo1_timed': 0, 'lfo1_to_cutoff': 15,
    'lfo1_to_lfo2_level': -20, 'lfo1_to_volume': -45, 'lfo1_waveform': 3, 'lfo2_level': 100,
    'lfo2_mode': 0, 'lfo2_mono': 1, 'lfo2_phase': 0, 'lfo2_rate': 0.15, 'lfo2_rise': 0.0,
    'lfo2_timed': 0, 'lfo2_to_lfo1_level': -100, 'lfo2_waveform': 0, 'lock': 0,
    'loudness_attack': 1, 'loudness_decay': 5, 'loudness_sustain': 10, 'main_output': 1,
    'mod_mix': 0, 'mod_wheel': 0, 'noise_on': 0, 'note_priority': 0, 'osc1_on': 1,
    'osc1_range': 3, 'osc1_volume': 6, 'osc1_waveform': 2, 'osc2_frequency': 0.08, 'osc2_on': 1,
    'osc2_range': 3, 'osc2_volume': 6, 'osc2_waveform': 2, 'osc3_frequency': 0, 'osc3_on': 0,
    'osc3_range': 3, 'osc3_volume': 6, 'osc3_waveform': 2, 'osc_mod': 0, 'poly': 1,
    'retrigger': 0, 'spread': 0, 'unison': 0, 'voices': 6, 'volume': 7,
}
LOOP_DESCRIPTIONS['LFO LOOP Level Pump'] = "An LFO fading another in and out by its LEVEL: LFO 1 (SQUARE, 4 Hz, SOFT) pumps MAIN OUTPUT VOLUME (-45 %) and CUTOFF; LFO 2 (TRIANGLE, 0.15 Hz) takes LFO 1's LEVEL down (-100 %) and back, and LFO 1 dents LFO 2's own LEVEL (-20 %), a loop. The pumping swells in and dies away. Hold a chord."

LOOP_VALUES['LFO LOOP Self Rate'] = {
    'a440': 0, 'auto_gain': 0, 'contour_amount': 3, 'cutoff': 1.5, 'decay_on': 1, 'double': 0,
    'doubled': 0, 'drive': 0, 'emphasis': 3, 'entropy': 5, 'ext_on': 0, 'feedback': 0,
    'filter_attack': 0, 'filter_decay': 4, 'filter_mod': 0, 'filter_mode': 0, 'filter_sustain': 6,
    'glide': 0, 'glide_on': 0, 'inner': 0, 'keyboard_control_1': 1, 'keyboard_control_2': 0,
    'level': 12, 'lfo1_level': 100, 'lfo1_mode': 1, 'lfo1_mono': 1, 'lfo1_phase': 0,
    'lfo1_rate': 0.8, 'lfo1_rise': 0.0, 'lfo1_timed': 0, 'lfo1_to_cutoff': 35,
    'lfo1_to_lfo1_rate': 60, 'lfo1_to_osc2_frequency': 3, 'lfo1_waveform': 1, 'lock': 0,
    'loudness_attack': 1, 'loudness_decay': 5, 'loudness_sustain': 10, 'main_output': 1,
    'mod_mix': 0, 'mod_wheel': 0, 'noise_on': 0, 'note_priority': 0, 'osc1_on': 1,
    'osc1_range': 3, 'osc1_volume': 6, 'osc1_waveform': 2, 'osc2_frequency': 0.08, 'osc2_on': 1,
    'osc2_range': 3, 'osc2_volume': 6, 'osc2_waveform': 2, 'osc3_frequency': 0, 'osc3_on': 0,
    'osc3_range': 3, 'osc3_volume': 6, 'osc3_waveform': 2, 'osc_mod': 0, 'poly': 0,
    'retrigger': 1, 'spread': 0, 'unison': 0, 'voices': 4, 'volume': 7,
}
LOOP_DESCRIPTIONS['LFO LOOP Self Rate'] = "An LFO on its own RATE: LFO 1 (the shark tooth, 0.8 Hz, RETRIG) on CUTOFF and OSCILLATOR 2's FREQUENCY, and on its own RATE (+60 %), so it races up its rise and drags down its fall; RETRIGGER on, each note starts it again the same. Play notes."

for name, tags in LOOP_TAGS.items():
    v = dict(LOOP_VALUES[name])
    if name in LEVELS:
        v.update(LEVELS[name])
    P.append(dict(name=name, description=LOOP_DESCRIPTIONS[name], tags=tags, macro_names=[],
                  values=v))



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
# The seventeen made for 0.3 (sounds the 0.2.8 set left out, and the arpeggiator, the LFOs, the
# macros, velocity, the timed and CONTINUOUS oscillators and pressure in use), then the four LFO
# LOOP presets (the LFOs moving each other's knobs and their own). Levelled as the rest, -18 LUFS
# midway between LO and HI (scripts/level_presets.py; decisions.md R-PRESETS-0.3.0).
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
