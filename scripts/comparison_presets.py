#!/usr/bin/env python3
"""Write the 24 comparison presets, exact monophonic MIDI and the patch guide.

No third-party Python packages. Run from any directory; output stays in this checkout.
The renderer is `cargo run --release -p ca72-plugin --example comparison_pack`.
"""

import json
import math
from pathlib import Path
import struct
import tomllib
from urllib.parse import quote

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "comparison-presets"
SECONDS = 4.8
RATE = 48000
RANGES = ["LO", "32'", "16'", "8'", "4'", "2'"]
WAVES = ["triangle", "shark tooth", "saw", "square", "wide pulse", "narrow pulse"]
NOTES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]

# Every patch specifies every sound parameter, so loading one cannot retain an extension
# or an unused mixer source from the previous patch. Player controls are set by MIDI.
BASE = dict(
    tune=0, glide=0, mod_mix=0, osc_mod=0, osc3_control=1,
    osc1_range=3, osc2_range=3, osc3_range=3,
    osc1_waveform=2, osc2_waveform=2, osc3_waveform=2,
    osc2_frequency=0, osc3_frequency=0,
    osc1_on=0, osc2_on=0, osc3_on=0,
    osc1_volume=0, osc2_volume=0, osc3_volume=0,
    ext_on=0, ext_volume=0, noise_on=0, noise_volume=0, noise_type=0,
    filter_mode=0, filter_mod=0, keyboard_control_1=0, keyboard_control_2=0,
    cutoff=4, emphasis=0, contour_amount=0,
    filter_attack=0, filter_decay=2, filter_sustain=0,
    loudness_attack=0, loudness_decay=2, loudness_sustain=10,
    volume=6, main_output=1, a440=0, glide_on=0, decay_on=0,
    mod_wheel=0, poly=0, voices=2, entropy=0, spread=0, inner=0,
    placement=0, unison=0, double=0, drive=0, level=0, auto_gain=0,
    doubled=0, feedback=0, lock=0, note_priority=0, retrigger=0,
)


def osc(number, footage, wave, level, interval=0):
    return {f"osc{number}_range": footage, f"osc{number}_waveform": wave,
            f"osc{number}_volume": level, f"osc{number}_on": 1,
            **({f"osc{number}_frequency": interval} if number > 1 else {})}


def phrase(keys, length=0.65):
    return [[round(0.2 + i, 3), length, key] for i, key in enumerate(keys)]


LOW = phrase([41, 45, 48, 53])
MID = phrase([53, 57, 60, 65])
TRACK = phrase([41, 53, 65, 77])
HELD = [[0.2, 3.6, 60]]
PATCHES = []


def add(name, purpose, notes, sources=(), settings=None, listen="", hardware="", cc=0):
    values = dict(BASE)
    for source in sources:
        values.update(source)
    values.update(settings or {})
    if cc is not None:
        values["mod_wheel"] = cc / 127
    number = len(PATCHES) + 1
    PATCHES.append(dict(number=number, name=f"CMP {number:02} {name}",
                        purpose=purpose, listen=listen, hardware=hardware,
                        seconds=SECONDS, notes=notes, cc1=cc, values=values))


add("Sub Triangle", "OSC 1 alone, triangle, 32': low-frequency shape and tuning.", LOW,
    [osc(1, 1, 0, 6)], listen="Fundamental and faint upper harmonics; the first played F2 sounds at F0 (21.83 Hz). Check the next three notes too, not just the subsonic end.")
add("Shark Register", "OSC 2 alone, shark tooth, 16': the asymmetric triangle/saw shape.", MID,
    [osc(2, 2, 1, 6)], listen="The character between a triangle and a saw, with OSC 1 and OSC 3 muted.")
add("Reverse Current", "OSC 3 alone, reverse saw, 8', keyboard control on.",
    phrase([48, 55, 60, 67]), [osc(3, 3, 1, 6)],
    listen="The reverse saw's spectrum and transient shape. A reverse saw can sound much like an ordinary saw; inspect polarity and waveform as well as listening.")
add("Square Octave", "OSC 1 alone, square, 4': octave-up oscillator tracking.",
    phrase([60, 64, 67, 72]), [osc(1, 4, 3, 6)],
    listen="Hollow odd-harmonic tone; input C4 is sounding C5, and the final input C5 sounds C6.")
add("Wide Skyline", "OSC 2 alone, wide pulse, 2': two-octave shift and upper-register harmonics.",
    phrase([48, 52, 55, 60]), [osc(2, 5, 4, 6)],
    listen="Compare with 04: the same sounding C5/E5/G5/C6 pitches, but a different pulse width and oscillator.")
add("Narrow Needle", "OSC 3 alone, narrow pulse, 8': high keyboard register.",
    phrase([77, 81, 84, 81]), [osc(3, 3, 5, 6)],
    listen="Thin nasal harmonics on F5/A5/C6; pitch and aliasing at the upper end of the original keyboard.")
add("Twin Beat", "OSC 1 + 2, saws at 8', OSC 2 ten cents sharp.", [[0.2, 3.6, 57]],
    [osc(1, 3, 2, 4), osc(2, 3, 2, 4, 0.1)],
    dict(keyboard_control_1=1),
    listen="Slow beating on held A3. Ten cents is deliberately between lines; use the displayed cents/tuner, rather than an approximate knob angle.",
    hardware="The stored +10 cents is relative to OSC 2's nominal centre. Match the measured beat rate against OSC 1, including each device's natural centre mismatch and drift.")
add("Octave Floor", "OSC 1 + 3, a 16' triangle over a 32' square.", LOW,
    [osc(1, 2, 0, 6), osc(3, 1, 3, 4)],
    dict(cutoff=-1, keyboard_control_2=1),
    listen="The square sub-octave and softer upper fundamental; check both voices still track over the four keys.")
add("Fifth Lantern", "OSC 2 + 3, 8' triangle and saw; OSC 3 seven semitones higher.",
    phrase([48, 50, 55, 60]), [osc(2, 3, 0, 5), osc(3, 3, 2, 4, 7)],
    dict(cutoff=2, keyboard_control_1=1, keyboard_control_2=1),
    listen="A fifth inside one synth voice. One MIDI key at a time; this is oscillator interval tuning, not polyphony.",
    hardware="The hardware's +7 marking is only a guide. Tune OSC 3 to exactly +7 semitones relative to OSC 2, then record the actual knob position.")
add("Three Floors", "OSC 1 + 2 + 3 at 32'/16'/8': three octave layers, three shapes.", LOW,
    [osc(1, 1, 2, 4), osc(2, 2, 1, 4), osc(3, 3, 3, 4)],
    dict(cutoff=0, keyboard_control_1=1, keyboard_control_2=1),
    listen="Low saw, middle shark tooth and high square stay in octave relationships; listen for mixer colour as all three sum.")
add("Copper Pluck", "Low-pass filter contour and zero loudness sustain: a short two-saw bass.",
    [[round(0.2 + i * 0.5, 3), 0.18, [41, 48, 50, 53, 41, 48, 50, 53][i]] for i in range(8)],
    [osc(1, 2, 2, 5), osc(2, 2, 2, 5)],
    dict(cutoff=-3, emphasis=6, contour_amount=6, filter_decay=2,
         loudness_decay=2, loudness_sustain=0, decay_on=1, keyboard_control_1=1),
    listen="Filter opens at attack then closes; loudness falls to zero even if a key were held. The gaps expose retrigger and residual tails.")
add("Slow Bloom", "Independent filter/loudness attack, nonzero sustain and released decay.",
    [[0.2, 2.6, 60]], [osc(1, 3, 0, 6), osc(2, 3, 2, 4)],
    dict(cutoff=-3, emphasis=4, contour_amount=6, filter_attack=6,
         filter_decay=3, filter_sustain=6, loudness_attack=4,
         loudness_decay=3, loudness_sustain=8, decay_on=1, keyboard_control_2=1),
    listen="Volume rises before the filter has finished opening; after the 2.80 s note-off, listen to both contour tails.")
with (ROOT / "crates/ca72-plugin/sounds/presets.toml").open("rb") as source:
    WHITE_LOTUS = next(p for p in tomllib.load(source)["preset"] if p["name"] == "White Lotus")
add("White Lotus", "The unchanged factory White Lotus: narrow-pulse bass, a third-harmonic filter ring and OSC 3 LO filter modulation.",
    [[0.2, 1.4, 48], [2.1, 1.4, 60]],
    settings={"auto_gain": 1, **WHITE_LOTUS["values"]}, cc=None,
    listen="The held narrow-pulse fundamental and its resonant ring an octave and a fifth above. C3 followed by C4 exposes full keyboard tracking and the approximately 11 Hz modulation of the ring. The factory's fine positions are preserved rather than rounded to lines.",
    hardware="Set OSC 1 to 8' narrow pulse, mixer 10; all other mixer sources OFF. Use LO filter, cutoff -2.14, emphasis 7, both tracking switches ON, contour amount 0. OSC 3 is a triangle at LO, CONTROL OFF and mixer OFF; tune its rate to approximately 11 Hz. FILTER MODULATION ON, OSCILLATOR MODULATION OFF, MOD MIX 0. Before recording, send CC 1 = 0 to clear incoming MIDI modulation, then load White Lotus on CA-72 and move the hardware MOD DEPTH knob to match its excursion. A MIDI wheel message takes over the hardware depth until that knob moves, so complete resets before setting the hardware knob. The shared clip sends no CC 1; leave the modulation controller untouched during the take, preserving the CA-72 preset's exact 0.17 and the matched hardware knob depth.")
PATCHES[-1]["factory_source"] = "White Lotus"
PATCHES[-1]["frequency_storage"] = "pot"
add("Ladder Whistle", "Filter self-oscillation, no VCO in the mixer, full keyboard tracking.", TRACK,
    settings=dict(noise_on=1, noise_volume=1, cutoff=-2, emphasis=10,
                  keyboard_control_1=1, keyboard_control_2=1),
    listen="Quiet noise excites the resonant ladder. Listen for the filter's near-sine tone and octave tracking. These keys control filter pitch; cutoff -2 does not promise concert tuning.",
    hardware="Keep NOISE at 1 on both devices to start oscillation reproducibly; do not tune the filter between notes in a take.")
add("High Pass Reed", "High-pass filter, resonance, contour, two-thirds keyboard tracking.", TRACK,
    [osc(1, 3, 4, 5), osc(3, 4, 1, 4)],
    dict(filter_mode=1, cutoff=1, emphasis=6, contour_amount=3,
         filter_decay=3, filter_sustain=2, keyboard_control_2=1),
    listen="Low fundamentals thin out, with the upper harmonics retained. Repeat the same take in LO to isolate the filter-mode change.")
add("Joined Glide", "GLIDE with a continuous monophonic phrase and single triggering.",
    [[0.2, 0.6, 48], [0.8, 0.6, 52], [1.4, 0.6, 55], [2.0, 0.6, 60],
     [2.6, 0.6, 55], [3.2, 0.8, 48]],
    [osc(1, 3, 2, 5), osc(2, 2, 4, 4)],
    dict(cutoff=-2, emphasis=4, contour_amount=4, filter_decay=4,
         filter_sustain=3, loudness_decay=3, loudness_sustain=10,
         glide=6, glide_on=1, decay_on=1,
         keyboard_control_1=1, keyboard_control_2=1),
    listen="Joined notes have zero gap, ordered note-off before note-on: never two held keys. Listen to up/down slide time and whether the filter attack restarts. Hardware multi-trigger must be OFF.",
    hardware="Set GLIDE to 6. For manual playing, use a single-note legato line; the supplied MIDI uses zero-gap joins. Repeat with GLIDE at 0 for the control take.")
add("White Snap", "White noise through fast contours: percussion without oscillators.",
    phrase([48, 48, 48, 48], 0.1),
    settings=dict(noise_on=1, noise_volume=6, cutoff=0, emphasis=2,
                  contour_amount=2, filter_decay=2, loudness_decay=2,
                  loudness_sustain=0, decay_on=1),
    listen="Broadband hiss, transient attack and the decay slope. MIDI C3 triggers the contour; with tracking off it does not pitch the noise.")
add("Pink Weather", "Pink noise, slow filter/loudness attack and sustained resonant wind.",
    [[0.2, 3.0, 48]],
    settings=dict(noise_on=1, noise_type=1, noise_volume=6, cutoff=-2,
                  emphasis=6, contour_amount=4, filter_attack=6, filter_decay=3,
                  filter_sustain=8, loudness_attack=4, loudness_decay=3,
                  loudness_sustain=8, decay_on=1),
    listen="Darker noise spectrum than 17, with a moving resonant band. For a controlled noise-colour comparison, repeat 17 changing only WHITE to PINK.")
add("Triangle Motion", "OSC 3 as an untracked LO triangle LFO, pitch destination only.", HELD,
    [osc(1, 3, 3, 6)],
    dict(osc3_range=0, osc3_waveform=0, osc3_control=0,
         osc_mod=1, cutoff=2, keyboard_control_1=1), cc=6,
    listen="Smooth pitch modulation on held C4. OSC 3's mixer switch stays OFF; it remains a modulation source. Repeat an octave higher to check the LFO rate stays fixed.",
    hardware="Select OSC 3 as source, NOISE/MOD SRC rather than the separate LFO, MOD MIX fully to OSC 3, MOD DEPTH 10. Use CC 1 = 6 as a reproducible starting depth, then match measured pitch excursion.")
add("Falling Window", "OSC 3 as an untracked LO reverse-saw LFO, filter destination only.",
    [[0.2, 3.6, 48]], [osc(1, 2, 2, 6)],
    dict(osc3_range=0, osc3_waveform=1, osc3_control=0,
         filter_mod=1, cutoff=-1, emphasis=6), cc=26,
    listen="A repeated asymmetric cutoff sweep with a sharp reset. Pitch modulation is OFF; listen to filter motion rather than a pitch slide.",
    hardware="Select OSC 3 and NOISE/MOD SRC, MOD DEPTH 10, CC 1 = 26. Match rate and cutoff excursion by measurement; the MIDI wheel transfer curve is not guaranteed identical on another hardware unit.")
add("Audio Alloy", "Keyboard-tracked OSC 3 at audio rate modulates oscillator pitch.",
    phrase([60, 64, 67, 72]), [osc(1, 3, 0, 6)],
    dict(osc3_range=4, osc3_waveform=0, osc3_control=1, osc_mod=1,
         cutoff=3, keyboard_control_1=1, keyboard_control_2=1), cc=13,
    listen="Metallic sidebands from audio-rate exponential FM. OSC 3 is inaudible in the mixer but audible through modulation. This is not oscillator sync or ring modulation.",
    hardware="Select OSC 3/NOISE sources, MOD DEPTH 10 and CC 1 = 13; keep OSC 3 mixer OFF and OSC 3 CONTROL ON. Match modulation depth before comparing sidebands.")
add("Mixed Turbulence", "Midpoint MOD MIX: LO square OSC 3 plus noise, both destinations.",
    [[0.2, 3.6, 48]], [osc(1, 3, 4, 6)],
    dict(osc3_range=0, osc3_waveform=3, osc3_control=0, mod_mix=5,
         osc_mod=1, filter_mod=1, cutoff=0, emphasis=4, noise_on=0), cc=13,
    listen="Stepped motion with random flutter. The mixer noise switch is OFF: noise still reaches the modulation circuit. Repeat at MOD MIX 0 and 10 to separate the two sources.",
    hardware="Select OSC 3 and NOISE/MOD SRC, MOD MIX 5, MOD DEPTH 10, CC 1 = 13. Use the built-in noise source with no cable in MOD SOURCE; match depth, then compare distributions rather than individual noise samples.")
add("External Window", "External audio through preamp, low-pass filter and keyboard-gated contours.",
    TRACK, settings=dict(ext_on=1, ext_volume=6, cutoff=-2, emphasis=6,
                         contour_amount=4, filter_decay=3, filter_sustain=2,
                         keyboard_control_1=1, keyboard_control_2=1),
    listen="All oscillator/noise mixer switches OFF, FEEDBACK 0. Play external-input.wav into the hardware EXT IN and the plug-in side chain at the same measured input level. Keys change filtering and gating; the source's pitch remains 220 Hz.",
    hardware="Connect the supplied mono test signal to EXT IN, which breaks the hardware's automatic output return. The plug-in receives that same file on its side chain. Match input voltage; an interface's dBFS is not an analog voltage.")
add("Return Furnace", "External-input feedback and mixer overload from the hardware-accessible loop.",
    LOW, [osc(1, 2, 2, 6), osc(2, 2, 3, 4)],
    dict(ext_on=1, ext_volume=6, feedback=6, cutoff=-2, emphasis=4,
         contour_amount=4, filter_decay=3, filter_sustain=2,
         loudness_decay=3, loudness_sustain=8, decay_on=1,
         keyboard_control_1=1),
    listen="Feedback changes the mixer/preamp distortion, not just listening volume; observe OVERLOAD. Compare repeat takes with the return muted and present at matched listening loudness.",
    hardware="For the closest topology, patch one PHONES output channel into EXT IN and use the independent PHONES volume; start at its 6 mark and EXT IN 6. CA-72 FEEDBACK 6 is a starting setting, not a promise of equal loop gain. The normalled uncabled return also works but depends on MAIN VOLUME; record which route and measured return level you used.")


def note_name(key):
    return f"{NOTES[key % 12]}{key // 12 - 1}"


def frequency_plain(semitones, number):
    """Invert ca72::freq_dial::Pot; files store pot position, not displayed semitones."""
    coefficients = ([16.753480, -0.651459, 0.194997, -0.005174, -0.000512]
                    if number == 2 else
                    [16.634609, -0.799108, 0.818439, -0.073963, 0.039155])
    low, high = 0.0, 1.0
    for _ in range(60):
        w = (low + high) / 2
        x = w - 0.5
        actual = sum(coefficient * x ** (i + 1) for i, coefficient in enumerate(coefficients))
        if actual < semitones:
            low = w
        else:
            high = w
    return round(((low + high) / 2 - 0.5) * 15, 9)


def file_values(patch):
    values = dict(patch["values"])
    if patch.get("frequency_storage") == "pot":
        return values
    for number in (2, 3):
        key = f"osc{number}_frequency"
        values[key] = frequency_plain(values[key], number)
    return values


def toml_values(values):
    return "\n".join(f"{key} = {value}" for key, value in values.items())


def vlq(value):
    result = [value & 127]
    value >>= 7
    while value:
        result.insert(0, (value & 127) | 128)
        value >>= 7
    return bytes(result)


def midi(patch):
    # 120 BPM, PPQN 960: exactly 1920 ticks/s. Timing resolves to <= 0.521 ms.
    events = [(0, 0, b"\xff\x51\x03\x07\xa1\x20"),
              (0, 1, b"\xe0\x00\x40")]
    if patch["cc1"] is not None:
        events.append((0, 2, bytes([0xB0, 1, patch["cc1"]])))
    label = patch["name"].encode()
    events.append((0, 0, b"\xff\x03" + vlq(len(label)) + label))
    for start, length, key in patch["notes"]:
        events.extend([(round(start * 1920), 4, bytes([0x90, key, 100])),
                       (round((start + length) * 1920), 3, bytes([0x80, key, 0]))])
    events += [(round(SECONDS * 1920), 5, b"\xb0\x7b\x00"),
               (round(SECONDS * 1920), 6, b"\xff\x2f\x00")]
    last, track = 0, bytearray()
    for tick, _, event in sorted(events, key=lambda e: (e[0], e[1])):
        track.extend(vlq(tick - last))
        track.extend(event)
        last = tick
    return (b"MThd" + struct.pack(">IHHH", 6, 0, 1, 960) + b"MTrk"
            + struct.pack(">I", len(track)) + track)


def write_source():
    # Low-level, repeatable harmonic probe: 220/440/660/880 Hz, 0.1 peak full scale.
    signal = []
    for i in range(round(RATE * SECONDS)):
        t = i / RATE
        ramp = min(1.0, t / 0.01, (SECONDS - t) / 0.01)
        x = sum(math.sin(2 * math.pi * 220 * h * t) / h for h in range(1, 5))
        signal.append(x * ramp)
    gain = 0.1 / max(abs(x) for x in signal)
    data = b"".join(struct.pack("<f", x * gain) for x in signal)
    fmt = struct.pack("<HHIIHH", 3, 1, RATE, RATE * 4, 4, 32)
    wav = b"RIFF" + struct.pack("<I", 36 + len(data)) + b"WAVEfmt " + struct.pack("<I", 16)
    wav += fmt + b"data" + struct.pack("<I", len(data)) + data
    (OUT / "external-input.wav").write_bytes(wav)


def validate():
    if len(PATCHES) != 24:
        raise ValueError("Expected exactly 24 presets")
    if not 1 <= sum(p["values"]["filter_mode"] == 1 for p in PATCHES) <= 2:
        raise ValueError("Expected one or two high-pass examples")
    lotus = next(p for p in PATCHES if p.get("factory_source") == "White Lotus")
    if any(file_values(lotus).get(k) != v for k, v in WHITE_LOTUS["values"].items()):
        raise ValueError("White Lotus differs from its factory values")
    if lotus["cc1"] is not None:
        raise ValueError("White Lotus MIDI must preserve its exact preset modulation depth")
    combinations = set()
    ranges, waves, tracking = set(), set(), set()
    for patch in PATCHES:
        end = 0
        if not patch["notes"]:
            raise ValueError(f"No notes: {patch['name']}")
        for start, length, key in patch["notes"]:
            if start < end - 1e-9 or length <= 0 or start < 0 or start + length > SECONDS:
                raise ValueError(f"Overlapping or out-of-window notes: {patch['name']}")
            if not 41 <= key <= 84:
                raise ValueError(f"Outside the original key range: {patch['name']}")
            end = start + length
        values = patch["values"]
        for key in ("poly", "unison", "double", "doubled", "entropy", "spread", "inner", "drive", "level", "lock", "retrigger"):
            if values[key] != 0:
                raise ValueError(f"Extension {key} enabled: {patch['name']}")
        combinations.add(tuple(i for i in range(1, 4) if values[f"osc{i}_on"]))
        for i in range(1, 4):
            if values[f"osc{i}_on"] or i == 3 and (values["osc_mod"] or values["filter_mod"]):
                ranges.add(values[f"osc{i}_range"])
                waves.add((i, values[f"osc{i}_waveform"]))
        tracking.add((values["keyboard_control_1"], values["keyboard_control_2"]))
    required = {(1,), (2,), (3,), (1, 2), (1, 3), (2, 3), (1, 2, 3)}
    if not required <= combinations or ranges != set(range(6)):
        raise ValueError("Missing oscillator combination or range")
    if {w for _, w in waves} != set(range(6)) or (3, 1) not in waves:
        raise ValueError("Missing waveform")
    if len(tracking) != 4:
        raise ValueError("Missing keyboard tracking state")


def oscillator_description(values):
    result = []
    for i in range(1, 4):
        if values[f"osc{i}_on"]:
            wave = values[f"osc{i}_waveform"]
            name = "reverse saw" if i == 3 and wave == 1 else WAVES[wave]
            interval = values.get(f"osc{i}_frequency", 0)
            result.append(f"{i}: {RANGES[values[f'osc{i}_range']]} {name}, level {values[f'osc{i}_volume']}"
                          + (f", freq {interval:+g} st" if i > 1 else ""))
    return "; ".join(result) or "All three OFF"


def guide():
    text = """# CA-72: 24 hardware comparison presets

24 loadable user presets for CA-72 0.2.0, with 24 monophonic MIDI phrases. Each take is
**4.8 seconds including its release window**, 48 kHz mono. All values are initial patch
settings: no knob automation, no effects, no polyphony, and never two active MIDI keys.
The set uses only sounds obtainable on a Behringer Model D. Names are proposed test labels;
the factory library is unchanged. **13 is the existing White Lotus preset**, with its
factory settings preserved. **15 is the only high-pass preset**; the other 23 use LO.

## Load and play

Copy the **24 individual files in `presets/`** into the CA-72 user preset folder, then reopen
the preset browser and search `comparison`. Do not copy `bank.toml` there: that is the
renderer's combined source. The normal library paths are:

- Linux: `~/.local/share/Idle Foundry/CA-72/Presets/`
- macOS: `~/Library/Application Support/Idle Foundry/CA-72/Presets/`
- Windows: `%APPDATA%\\Idle Foundry\\CA-72\\Presets\\`

An `IDLE_FOUNDRY_SHARED` override changes the root. This pack does not install itself or
overwrite the user's library. Use the matching file in `midi/` on MIDI channel 1, at the
embedded **120 BPM**, with warp/time-stretch disabled. Names use **C4 = MIDI 60**; MIDI
numbers below are definitive when a host labels middle C differently. Velocity is always
100; neither instrument needs velocity response for these tests.

## Common setup and dial notation

Use **MONO**, UNISON/DOUBLE off, ENTROPY/WIDTH/INNER/DRIVE zero, LEVEL 0 dB, LOCK off,
NOTE PRIORITY LOW and RETRIGGER off. These are explicitly neutral in every preset. Set
QUALITY to **ULTRA for offline comparison**, or HI for real-time playing: quality is a
computer preference, so the preset files do not change it. Pitch bend is centred in each
MIDI file; MODULATION (CC 1) is explicitly initialized to the listed value except in 13.
**Before a White Lotus take:** send CC 1 = 0 to clear incoming MIDI modulation, then load
the CA-72 preset and move the hardware MOD DEPTH knob to its matched depth. Complete this
reset before setting the hardware knob: a MIDI wheel message takes over its depth until
the knob moves. White Lotus's shared performance clip sends no CC 1, preserving the
factory's saved 0.17 and the matched hardware knob setting. Leave the modulation controller
untouched during the take. No
pitch-bend range setting is needed because the phrases do not bend pitch.

With LOCK off, CA-72 retains its normal small oscillator mismatch and drift even at
ENTROPY 0. A FREQUENCY of 0 is a nominal centre, not a guarantee that the oscillators
are perfectly in unison. For a pitch-matched control pass, match the measured sounding
pitches and intervals on the hardware. Keep that pass labelled separately from the
identical-position pass.

MAIN OUTPUT on, VOLUME **6**, TUNE **0**, A-440 off. Disabled mixer sources have their
switch OFF and level 0. **White Lotus (13) is the exception:** its VOLUME 9.78, inactive
mixer levels, VOICES 4 and AUTO GAIN on at DRIVE 0 are retained as in the factory preset.
Its tone uses no extension effect; the exact exceptions are listed below. GLIDE is off
except 16; FEEDBACK is 0 except 24. The hardware has
separate FILTER DECAY and LOUDNESS DECAY switches: set **both** to the listed CA-72 DECAY
state. Select **OSC 3**, **NOISE/MOD SRC**, not Filter EG or the separate LFO, for the
Model D modulation-source switches. Leave its dedicated LFO unused; CA-72's LFO tests use
**OSC 3 at LO**. Leave MOD SOURCE unpatched. For the non-modulated presets set MOD DEPTH 0;
for 19–22 start at MOD DEPTH 10 and the listed CC 1, then match modulation excursion.
White Lotus (13) also uses filter modulation; follow its own depth/rate instructions.

Numbers on 0–10 knobs mean pointer positions. CUTOFF is the signed printed dial; use whole
numbers (-3, -2, -1, 0, +1, +2, +3, +4). **ATTACK/DECAY also use 0–10 travel positions,
not seconds:** 0 is the counterclockwise stop, 2 the `200 M-SEC.` line, 3 the next line,
4 the `600 M-SEC.` line, 5 the noon line and 6 the `1 SEC.` line. These choices lie on
actual GUI ticks. ADS below means ATTACK / DECAY / SUSTAIN, in that order. K1/K2 = on/off
states: 0/0 no tracking, 1/0 one-third, 0/1 two-thirds, 1/1 full tracking.
White Lotus retains its original fine settings, including cutoff -2.14, loudness ADS
0.5 / 3.5 / 8.5 and VOLUME 9.78; these deliberately override the preference for lines.

FREQUENCY is a **true interval from the oscillator's centre**, as CA-72 displays it with
the SEMITONES knob preference. Files store the correctly converted pot positions. All
are at 0 except 07 (+10 cents, a deliberate between-line setting) and 09 (+7 semitones,
on a GUI line), plus White Lotus's LO oscillator near the clockwise stop (stored
OSC 3 pot value 7.47627, approximately 11 Hz with CONTROL off). The Model D manual describes its semitone markings as a guide; use a
tuner to reproduce these intervals on hardware. A setting at 8' sounds at the entered
note; 32'/16' sound two/one octaves down, 4'/2' one/two octaves up. LO is used as modulation
in 13, 19, 20 and 22, not promised as an ordinary concert-pitch register.

## The 24 patches at a glance

| # | Preset | Audible oscillators (range, shape, mixer level) | Main test | MIDI notes played |
|---|---|---|---|---|
"""
    for patch in PATCHES:
        notes = " → ".join(note_name(n[2]) for n in patch["notes"])
        text += f"| {patch['number']:02} | {patch['name'][7:]} | {oscillator_description(patch['values'])} | {patch['purpose']} | {notes} |\n"
    text += "\n## Exact settings and timing\n\nAll unspecified controls use the common setup above. Each timing is **start / held length / key** in seconds. The last column of the table is the input key, not always the sounding octave.\n"
    for patch in PATCHES:
        v = patch["values"]
        num = patch["number"]
        text += f"\n### {num:02}. {patch['name'][7:]}\n\n{patch['purpose']}\n\n"
        text += f"- Oscillators: {oscillator_description(v)}. OSC 3 CONTROL {'ON' if v['osc3_control'] else 'OFF'}.\n"
        if num == 13:
            text += "- Modulator: OSC 3 LO triangle, mixer OFF; original FREQUENCY pot value 7.47627, near the clockwise stop, approximately 11 Hz with CONTROL OFF.\n"
        elif num in (19, 20, 21, 22):
            wave = v["osc3_waveform"]
            name = "reverse saw" if wave == 1 else WAVES[wave]
            text += f"- Modulator: OSC 3 {RANGES[v['osc3_range']]} {name}, FREQUENCY 0, mixer OFF.\n"
        text += f"- Filter: {'HI' if v['filter_mode'] else 'LO'}, CUTOFF {v['cutoff']:+g}, EMPHASIS {v['emphasis']}, AMOUNT {v['contour_amount']}; K1/K2 {v['keyboard_control_1']}/{v['keyboard_control_2']}.\n"
        text += f"- Filter ADS: {v['filter_attack']} / {v['filter_decay']} / {v['filter_sustain']}; loudness ADS: {v['loudness_attack']} / {v['loudness_decay']} / {v['loudness_sustain']}; DECAY {'ON' if v['decay_on'] else 'OFF'}.\n"
        depth = (f"preset MODULATION **{v['mod_wheel']}**, no CC 1 in the performance clip; clear incoming state during setup" if num == 13
                 else f"CC 1 **{patch['cc1']} / 127**")
        text += f"- Modulation: MIX {v['mod_mix']}, OSC {'ON' if v['osc_mod'] else 'OFF'}, FILTER {'ON' if v['filter_mod'] else 'OFF'}, {depth}.\n"
        if num == 13:
            text += "- Original factory exceptions: MAIN VOLUME 9.78; LO filter; inactive OSC 2/OSC 3/NOISE/EXTERNAL INPUT mixer levels 8, all their switches OFF; VOICES 4; AUTO GAIN ON with DRIVE 0 (no gain correction). All saved factory parameter values match White Lotus exactly.\n"
        if v["glide_on"]:
            text += f"- GLIDE ON, amount {v['glide']}.\n"
        if v["noise_on"]:
            text += f"- Noise: {'PINK' if v['noise_type'] else 'WHITE'} ON, level {v['noise_volume']}.\n"
        if v["ext_on"]:
            text += f"- EXTERNAL INPUT ON, level {v['ext_volume']}; FEEDBACK {v['feedback']}.\n"
        text += "- Notes: " + "; ".join(f"{a:.2f} / {b:.2f} / {note_name(k)} ({k})" for a, b, k in patch["notes"]) + ".\n"
        text += f"\nListen for: {patch['listen']}\n"
        if patch["hardware"]:
            text += f"\nHardware: {patch['hardware']}\n"
        stem = quote(patch['name'])
        text += (f"\n[Preset](presets/{stem}.toml) · [MIDI](midi/{stem}.mid) · "
                 f"[Listen](audio/listening/{stem}.wav) · [Raw](audio/raw/{stem}.wav) · "
                 f"[Panel](panels/{stem}.png)\n")
    text += """
## Coverage and useful control takes

The seven nonempty VCO combinations are explicit: **1 (01/04), 2 (02/05), 3 (03/06),
1+2 (07), 1+3 (08), 2+3 (09), 1+2+3 (10)**. All mixer VCOs are off for 14, 17, 18 and
23. The six switch ranges are 32' (01), 16' (02), 8' (03), 4' (04), 2' (05) and
LO (19/20/22). Patches 01–06 isolate six distinct shapes, including OSC 3's reverse saw;
the ordinary saw switch position is covered in 07 and isolated by its control take below.
K1/K2's four states occur in 01, 07, 08 and
09. Musical keys span F2 (41) through C6 (84); sounding VCO fundamentals span F0 (21.83 Hz)
through C6 (1046.50 Hz), with upper harmonics above that. LFOs and filter oscillation need
separate rate/pitch measurements rather than the oscillator-note assumption.

For more rigorous isolation, repeat a phrase after changing **one control only**:

- 01: TUNE 0 versus its +1 line, keeping all other settings fixed. Repeat 02 and 03 to
  check global tuning reaches each keyboard-controlled oscillator; the hardware markings
  are a guide, so measure the resulting interval rather than assuming exactly one semitone.
- 06: during the held C6 at 2.20–2.85 s, move the MIDI controller's pitch wheel fully up
  and back to centre.
  Before this manual control take, set the same MIDI bend range on both devices (for example
  +/-2 semitones). The shipped MIDI centres bend but does not perform this move. Note the
  upward excursion should reach D6 with that range; the full take still lasts 4.8 s.
- 07: OSC 2 level 0, then 4; the first take isolates a single ordinary saw. Keep recorded
  listening gains fixed until levelling copies.
- 10: all three mixer levels 4, then 8; tests mixer saturation without the DRIVE extension.
- 11: AMOUNT 0, then 6; or K1/K2 0/0, 1/0, 0/1, 1/1 in four separate 4.8 s takes.
- 12: both hardware DECAY switches on, then off; keep every other setting fixed.
- 13: repeat White Lotus with FILTER MODULATION off, then on, to isolate its 11 Hz
  movement without changing the oscillator or filter's resonant tuning.
- 14: K1/K2 0/0, 1/0, 0/1, 1/1: compare filter-frequency ratios on octave-spaced keys.
- 15: HI versus LO on exactly the same keys; this is the direct filter-mode A/B.
- 16: GLIDE 0 versus 6; then add 50 ms gaps if testing contour re-arming separately.
- 17: WHITE versus PINK without borrowing 18's other envelope settings.
- 19: hold C4, then C5 in a second take. With OSC 3 CONTROL off, the LFO rate stays fixed.
- 20: reverse saw versus saw versus square, keeping the modulation rate/depth fixed.
- 21: OSC 3 CONTROL off versus on; the first no longer follows the carrier's keyboard pitch.
- 22: MOD MIX 0 / 5 / 10 in separate takes; oscillator only, mixture, noise only.
- 23: EXT level 3 / 6 / 9 in separate takes at unchanged source voltage; tests the preamp.
- 24: return absent versus present. Use downstream recording gain to level listening copies.

These are repeat takes of the 24 presets, not extra bank entries. Every repeat uses a 4.8 s
window. No long decay or sub-0.2 Hz modulation can be characterized completely inside five
seconds; this set samples useful short behaviour rather than claiming full time-range coverage.
The Model D's separate LFO, Filter-EG modulation source and extra CV/gate routings are outside
this pack because they are not shared front-panel functions of CA-72. Its independently
switchable filter/loudness releases are represented by matching both switches to CA-72's
shared DECAY switch. A-440 is for tuning before a take; it is off in musical presets.

## Comparison procedure

Warm the hardware for approximately **30 minutes before tuning/calibration** (manual
section 5.1.1), tune OSC 1 at A3 on 8', then centre/tune OSC 2
and OSC 3 against it. Disable poly chain, set key priority LOW and multi-trigger OFF.
Record firmware, the unit, temperature, patch positions, and CA-72 version/QUALITY. Both
devices receive the same MIDI file. No chorus, reverb, stereo widening, dynamics or DAW
time-stretch. Record dry mono at 48 kHz with fixed analog input gain and headroom.

Run **two passes** when calibration matters: first identical physical positions; then a
clearly labelled pass matching measured cutoff, modulation excursion/rate or envelope time.
Do not retune the controls between the notes of a pass. Identical knob angles are a useful
control test, but not proof of identical circuit values or oscillator intervals.

Keep the raw recording for gain, preamp and overload comparison. For listening, apply only a
constant downstream gain to each take, targeting about -18 LUFS maximum momentary loudness,
subject to peak headroom. Never normalize individual notes and never use a limiter. Match
the hardware to the CA-72 listening take, then randomize A/B order. Noise and free-running
oscillators will not null sample for sample; compare pitch in cents, harmonic levels,
noise spectral slope, attack/decay curves, modulation rate/excursion, glide transition time
and repeat-to-repeat variation. Repeat noise, detuned and feedback takes at least three
times; the same 4.8 s window applies to each. A successful render alone does not establish
hardware fidelity; no hardware recordings are included in this pack.

For 23, `external-input.wav` is a 4.8 s mono 220 Hz harmonic probe (harmonics 1–4), peak
-20 dBFS. Start its playback at the MIDI clip's time zero on both devices. Record its actual
analog voltage at the hardware input and match the plug-in's input voltage convention;
the plug-in treats 1.0 as 5 V, so this probe's 0.1 peak represents 0.5 V peak at the jack.
Do not infer hardware voltage from the DAW meter. For 24, use the independent PHONES return to approximate
CA-72's pre-main-volume topology. The Model D's built-in return uses its main output and
therefore cannot be assumed equivalent at the same printed settings.

## Source and regeneration

Hardware controls and mappings were checked against the manufacturer's
[Model D user manual](https://www.adorama.com/col/productManuals/BEMODELD.pdf), especially
sections 3.1.2–3.1.6, 4.3–4.8 and the MIDI settings. The manual is hosted by Adorama.
CA-72 source: `params.rs`, `freq_dial.rs`, `ca72-panel/src/art.rs`, `library.rs` and the
engine at **aedde6670cd5210e51bec9b6b19288c3d4327dd6**, the public 0.2.0 line checked
on 2026-10-13. The pack was revised on 2026-10-14 to include the unchanged White Lotus
settings from `crates/ca72-plugin/sounds/presets.toml`; the other preset settings and all
phrases were designed for this pack.

From this checkout:

```sh
python3 scripts/comparison_presets.py
CARGO_BUILD_JOBS=4 nice -n19 ionice -c3 cargo run --release -p ca72-plugin --example comparison_pack
```

The Python generator validates count, monophonic timing, range/shape/combination coverage,
one or two high-pass examples, exact White Lotus factory values and neutral extensions.
The renderer validates the bank through the actual preset library,
renders the same engine used by the plug-in, checks finite mono samples and measurements,
and writes `audio/raw/`, `audio/listening/`, `panels/` and `measurements.json`. Raw output
uses the patch's VOLUME (6 except White Lotus's original 9.78); listening output has a
documented constant gain only. The render
uses a fixed seed for reproducibility; a hardware noise stream has no corresponding seed.
"""
    (OUT / "README.md").write_text(text)


def main():
    validate()
    for folder in (OUT, OUT / "presets", OUT / "midi"):
        folder.mkdir(parents=True, exist_ok=True)
    bank = "# 24 Model D-accessible comparison patches. See README.md for true interval and timing notation.\nformat = 1\n"
    for patch in PATCHES:
        header = (f"name = {json.dumps(patch['name'])}\n"
                  f"description = {json.dumps(patch['purpose'])}\n"
                  'tags = ["comparison", "mono", "4.8s"]\n')
        values = toml_values(file_values(patch))
        bank += f"\n[[preset]]\n{header}\n[preset.values]\n{values}\n"
        (OUT / "presets" / f"{patch['name']}.toml").write_text(f"format = 1\n{header}\nfavorite = false\n\n[values]\n{values}\n")
        (OUT / "midi" / f"{patch['name']}.mid").write_bytes(midi(patch))
    (OUT / "bank.toml").write_text(bank)
    (OUT / "phrases.json").write_text(json.dumps(dict(format=1, sample_rate=RATE, seconds=SECONDS, presets=PATCHES), indent=2) + "\n")
    write_source()
    guide()
    print(f"{OUT}: 24 presets, 24 monophonic MIDI clips, 4.8 s each; coverage passed")


if __name__ == "__main__":
    main()
