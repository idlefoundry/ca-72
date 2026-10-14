#!/usr/bin/env python3
"""Round 2: original short musical phrases retaining the hardware comparison coverage."""

import copy
import json
import math
from urllib.parse import quote

import comparison_presets as base

OUT = base.ROOT / "docs/musical-comparison-presets"
PATCHES = copy.deepcopy(base.PATCHES)


def score(events, offset=0.2):
    """Quarter-note beats at 120 BPM; durations are gates, not velocities."""
    return [[round(offset + beat * 0.5, 6), round(gate * 0.5, 6), key]
            for beat, gate, key in events]


def revise(number, name, events, musical, listen, settings=None):
    patch = PATCHES[number - 1]
    patch["source_comparison"] = patch["name"]
    patch["name"] = f"MUS {number:02} {name}"
    patch["notes"] = score(events)
    patch["musical_intent"] = musical
    patch["listen"] = listen
    patch["tags"] = ["musical", "round2", "comparison", "mono", f"{patch['seconds']:g}s"]
    if number != 13:
        patch["values"].update(loudness_attack=0, loudness_decay=2,
                              loudness_sustain=8, decay_on=1)
    patch["values"].update(settings or {})
    # Each round has its own instructions; no stale reference to the first round's notes.
    if number != 13:
        patch["hardware"] = ""
    return patch


lead = [(0, .5, 60), (.75, .25, 63), (1.25, .5, 65), (2, .75, 67),
        (3, .25, 70), (3.5, .5, 67), (4.25, .5, 65), (5, .25, 63),
        (5.5, .5, 60), (6.25, .25, 67), (6.75, .25, 70), (7.25, .65, 72)]

revise(1, "Pocket Sub",
    [(0, .65, 65), (.75, .25, 65), (1.5, .4, 72), (2.25, .4, 68),
     (3, .7, 65), (4, .4, 60), (4.75, .25, 63), (5.5, .4, 65),
     (6.25, .45, 53), (7, .9, 41)],
    "Syncopated F-minor bass, ending with a two-octave sub drop.",
    "The main hook sounds at F2/Ab2/C3; the closing keys F3 and F2 sound F1 and F0 on 32'. The final low note is a deliberate range probe, after an audible bass line. Listen to triangle purity and the difference between a rounded gate and a note gap.")
revise(2, "Tooth Groove",
    [(0, .6, 50), (.75, .25, 57), (1.25, .25, 53), (1.75, .6, 50),
     (2.75, .25, 57), (3.25, .45, 59), (4, .7, 60), (5, .25, 59),
     (5.5, .5, 57), (6.25, .25, 53), (6.75, .25, 52), (7.25, .65, 50)],
    "A D-Dorian bass hook: the natural sixth lifts the second half.",
    "OSC 2 alone at 16' gives this line a woody asymmetric edge. Hear repeated D attacks, the B-natural turn, and the return home; the pitch centre stays exposed without detuning or modulation.")
revise(3, "Reverse Reply",
    [(0, .8, 55), (1.25, .25, 58), (1.75, .5, 60), (2.5, .9, 62),
     (3.75, .2, 60), (4.25, .25, 58), (4.75, .8, 55), (6, .25, 65),
     (6.5, .5, 62), (7.25, .65, 55)],
    "A G-minor call and response, with a high answer and a low cadence.",
    "OSC 3's reverse saw carries the hook with CONTROL on. Compare its harmonic balance and attack to an ordinary saw; polarity is a waveform check rather than a promise of a radically different sound.")
revise(4, "Arcade Lead", lead,
    "A climbing C-minor pickup answers with a falling line and a high final C.",
    "The 4' square sounds an octave above the played keys: this melody reaches sounding C6. Listen for hollow odd harmonics and the clean separation between the short pickups and longer answer.")
revise(5, "Glass Pulse", [(b, d, k - 12) for b, d, k in lead],
    "The same lead melody as 04, at the same nominal sounding pitches.",
    "The 2' wide pulse uses input notes an octave below 04 to reach the same sounding melody. The rhythm is identical, making its brighter pulse-width colour easy to compare with the square while preserving the separate-oscillator test.")
revise(6, "Top Shelf",
    [(0, .55, 77), (.75, .25, 81), (1.25, .5, 84), (2, .25, 82),
     (2.5, .55, 81), (3.25, .25, 79), (3.75, .55, 77), (4.5, .25, 79),
     (5, .25, 81), (5.5, .5, 84), (6.25, .25, 82), (6.75, 1.1, 77)],
    "A high F-major hook with a quick flourish and a longer resolving note.",
    "F5 through C6 is the top of the original keyboard. The narrow pulse should remain recognisably nasal across the run; the held final F gives time to hear pitch stability after the fast high notes.")


def musical_sweep(number, name, key):
    patch = revise(number, name, [(0, .2, key)],
        "The same syncopated minor-pentatonic riff opens and closes with the filter.",
        "Four seconds opening, a one-second tonic hold fully open, then the same four-second riff closing. Repeating the pitches and rhythm makes the change in brightness audible without replacing it with a scale exercise.",
        dict(keyboard_control_1=0, keyboard_control_2=0, loudness_decay=1,
             loudness_sustain=10))
    motif = [(0, .5, key), (.75, .25, key), (1, .5, key + 7),
             (1.75, .25, key + 10), (2.5, .5, key + 7), (3.25, .25, key + 5),
             (4, .75, key), (5, .5, key + 3), (5.75, .25, key + 5),
             (6, .5, key + 7), (7, .75, key)]
    patch["notes"] = ([[.2, .125, key]] + score(motif, .4)
                      + [[4.4, 1.0, key]] + score(motif, 5.4)
                      + [[9.4, .1, key]])
    patch["sweep_note_policy"] = "repeating_phrase"
    patch["purpose"] = ("Full-range low-pass sweep with EMPHASIS "
                        + str(patch["values"]["emphasis"]) + "; "
                        + ("two 8' saws ten cents apart." if number == 7 else "three 32'/16'/8' octave layers."))
    patch["hardware"] = "Turn only CUTOFF from -5 to +5 and back using the timed points. Both keyboard-tracking switches are OFF so note changes do not add a cutoff sweep. Contour amount and filter modulation stay at 0. The MIDI does not turn the physical knob."
    if number == 7:
        patch["listen"] += " EMPHASIS 0 gives a gentle opening; the held A3 at the top exposes the ten-cent saw beating. Use a tuner/beat-rate match for the between-line detune setting."
    else:
        patch["listen"] += " EMPHASIS 8 gives the riff a travelling resonant bite. Input F3 voices F1/F2/F3 in the three oscillators."


musical_sweep(7, "Sweep Groove", 57)
revise(8, "Octave Bounce",
    [(0, .6, 53), (.75, .25, 53), (1.5, .5, 60), (2.25, .25, 63),
     (2.75, .55, 65), (3.5, .25, 60), (4.25, .5, 58), (5, .25, 60),
     (5.5, .5, 53), (6.25, .25, 48), (6.75, 1.1, 53)],
    "A buoyant F-minor bass that hops up, then falls back to the sub.",
    "The 16' triangle sits over the 32' square. Short repeated roots expose their blend; octave leaps show that the two layers keep their ratio. Two-thirds filter tracking changes colour across the line.")
revise(9, "Parallel Fifths",
    [(0, .7, 48), (1, .25, 50), (1.5, .5, 51), (2.25, .75, 55),
     (3.25, .4, 58), (4, .7, 60), (5, .25, 58), (5.5, .5, 55),
     (6.25, .25, 51), (6.75, 1.1, 48)],
    "An open-fifth C-minor hook with a lifted octave answer.",
    "OSC 2 triangle and OSC 3 saw play a parallel fifth inside one voice. This gives a harmonised sound while MIDI remains monophonic. Tune the true +7-semitone interval on the hardware before the take.")
musical_sweep(10, "Resonant Run", 53)
revise(11, "Rubber Bass",
    [(0, .4, 41), (.75, .2, 41), (1.25, .3, 48), (1.75, .2, 51),
     (2.5, .4, 53), (3.25, .2, 48), (3.75, .2, 46), (4.25, .4, 41),
     (5, .2, 48), (5.5, .3, 51), (6.25, .2, 50), (6.75, 1.1, 41)],
    "A clipped, syncopated F-minor bass with a chromatic turnaround.",
    "The short filter contour gives every attack a rubbery vowel; sustain zero keeps the final long key from becoming a held organ tone. A mixture of short gates and rests exposes decay and re-arming.",
    dict(loudness_sustain=0, loudness_decay=2))
revise(12, "Slow Reply", [(0, 2.25, 60), (2.75, 2.25, 67), (5.25, 2.65, 65)],
    "A three-note C–G–F answer whose swells overlap in colour, inside a mono voice.",
    "Longer gates let the loudness rise before the filter finishes opening. The falling final interval makes the swell feel like a phrase; the remaining tail exposes both decay contours.",
    dict(filter_attack=5, loudness_attack=3, loudness_decay=3))
revise(13, "White Lotus",
    [(0, .75, 48), (1, .25, 51), (1.5, .5, 55), (2.5, .25, 58),
     (3, .75, 55), (4, .75, 60), (5, .25, 58), (5.5, .5, 55),
     (6.5, .25, 51), (7, .9, 48)],
    "An original C-minor pentatonic hook with a higher call and a low answer.",
    "The longer C3/C4 gates expose the roughly 11 Hz filter motion and third-harmonic ring; the short connecting notes turn it into a hook. Every saved factory setting remains exact, including its fine positions and output volume.")
revise(14, "Ladder Melody",
    [(0, .75, 53), (1, .25, 57), (1.5, .5, 60), (2.25, .6, 65),
     (3.25, .4, 60), (4, .75, 77), (5, .25, 72), (5.5, .5, 69),
     (6.25, .25, 65), (6.75, 1.1, 53)],
    "A soft F-major call and a two-octave whistle answer.",
    "All VCO mixer switches are off: the ladder itself supplies the pitched tone. Full tracking makes the relative melody possible. Printed cutoff -2 is not a concert-pitch promise; tune once before an optional pitch-matched pass, never between notes.")
revise(15, "High Pass Funk",
    [(0, .4, 53), (.75, .25, 53), (1.25, .25, 60), (1.75, .4, 63),
     (2.5, .4, 65), (3.25, .25, 60), (4, .4, 77), (4.75, .25, 72),
     (5.25, .4, 65), (6, .25, 63), (6.75, 1.1, 53)],
    "A wiry F-minor funk line with a high-register reply.",
    "The only HI patch keeps the riff's upper harmonics while thinning its bottom. Play the same pattern in LO for the controlled mode comparison; the wide pulse/reverse-saw combination and two-thirds tracking stay fixed.",
    dict(loudness_sustain=6))
revise(16, "Sliding Hook",
    [(0, 1, 48), (1, .5, 51), (1.5, .5, 53), (2, 1.5, 55),
     (3.5, .5, 67), (4, 1, 60), (5, .5, 58), (5.5, .5, 55),
     (6, 1, 51), (7, .9, 48)],
    "A joined C-minor hook with small scoops and a wide octave slide.",
    "Zero-gap joins keep exactly one key at a time. GLIDE 4 lets the motif move rather than lingering on every transition. The wide jump and falling answer expose glide distance and direction; repeat with glide off, hardware multi-trigger off.",
    dict(glide=4, loudness_sustain=10))
revise(17, "Noise Break",
    [(b, d, 48) for b, d in [(0,.25),(.75,.125),(1.5,.25),(2,.25),
     (2.5,.125),(2.75,.125),(3.5,.25),(4,.25),(5,.25),(5.5,.125),
     (6.25,.125),(6.5,.125),(7,.5)]],
    "A two-bar noise break with offbeat doubles and a spaced closing hit.",
    "Pitch and velocity do not accent this noise patch. The groove comes from timing and gate length. Shorter decays expose fast transient behaviour; change only WHITE to PINK on a repeat for a colour comparison.",
    dict(loudness_decay=1, loudness_sustain=0, filter_decay=1, emphasis=4,
         contour_amount=4))
revise(18, "Pink Flute",
    [(0, 1.25, 65), (1.5, .5, 69), (2.25, .6, 72), (3.25, .5, 69),
     (4, 1.25, 77), (5.5, .5, 74), (6.25, 1.5, 65)],
    "A breathy resonant-filter melody, with an octave reply.",
    "Pink noise excites a strong tracking resonance; there is no audible oscillator. The keys move the filter's spectral focus. Hear a breathier, noisier contour than the self-oscillating ladder in 14, rather than treating this as a perfectly tuned flute.",
    dict(noise_volume=4, emphasis=8, filter_attack=1, filter_decay=3,
         filter_sustain=6, loudness_attack=1, loudness_decay=3,
         keyboard_control_1=1, keyboard_control_2=1))
revise(19, "Wavering Lead",
    [(0, 2, 60), (2.5, .4, 63), (3.25, .4, 65), (4, .75, 67),
     (5, .4, 70), (5.75, .4, 67), (6.5, 1.4, 60)],
    "A singing C-minor line, framed by two longer vibrato notes.",
    "The held ends expose the LO triangle's smooth pitch modulation; shorter middle notes give it a melodic purpose. OSC 3 CONTROL and its mixer stay off, so the LFO rate does not follow this melody.",
    dict(osc3_frequency=3))
revise(20, "Filter Ostinato",
    [(0, .45, 53), (.5, .3, 53), (1.5, .3, 60), (2, .45, 53),
     (2.5, .3, 63), (3.25, .45, 60), (4, .45, 53), (4.5, .3, 53),
     (5.5, .3, 65), (6, .45, 63), (6.5, .3, 60), (7, .9, 53)],
    "A clipped F-minor ostinato that plays against the free-running filter rhythm.",
    "OSC 3's LO reverse saw moves only the filter. The repeating note rhythm lets its asymmetric fall/reset drift across the groove; it is free-running, not DAW-synchronised. Compare another take changing only the LFO shape.")
revise(21, "Metal Arpeggio",
    [(0, .4, 60), (.5, .4, 67), (1, .4, 63), (1.5, .4, 70),
     (2, .6, 74), (3, .4, 70), (3.5, .4, 67), (4, .4, 63),
     (4.75, .4, 60), (5.25, .4, 67), (5.75, .4, 70), (6.5, 1.4, 72)],
    "A C-minor-nine arpeggio with a displaced answer and an octave landing.",
    "The bright metallic sidebands follow the melody because OSC 3 CONTROL is on. OSC 3 remains out of the mixer: its contribution is audio-rate exponential FM, without ring modulation or oscillator sync.")
revise(22, "Broken Square",
    [(0, .65, 50), (1, .3, 50), (1.75, .4, 53), (2.5, .5, 57),
     (3.25, .4, 56), (4, .65, 62), (5, .3, 60), (5.75, .4, 57),
     (6.5, 1.4, 50)],
    "A lurching D-minor riff with a flattened-fifth passing note.",
    "LO square motion and modulation noise roughen the pitch and cutoff together. The repeated D motif lets you hear the random variation against a stable gesture. The mixer noise remains off; compare MOD MIX 0/5/10 on separate takes.")
revise(23, "External Rhythm",
    [(0, .5, 53), (.75, .25, 53), (1.25, .25, 60), (1.75, .5, 65),
     (2.5, .25, 60), (3.25, .5, 53), (4, .5, 65), (4.75, .25, 63),
     (5.25, .25, 60), (6, .5, 53), (7, .9, 65)],
    "A rhythmic vowel-like phrase made by gating and filtering the external source.",
    "The source stays at 220 Hz; the MIDI keys alter filter tracking and gating rather than the source's pitch. Follow the same external probe on both devices, with all VCO/noise mixer switches off and feedback zero.",
    dict(contour_amount=6, emphasis=8, loudness_sustain=6))
revise(24, "Feedback Stomp",
    [(0, .6, 53), (.75, .25, 53), (1.5, .4, 60), (2.25, .4, 63),
     (3, .6, 65), (4, .6, 53), (5, .25, 60), (5.5, .4, 59),
     (6.25, .25, 58), (6.75, 1.1, 53)],
    "An overdriven F-minor stomp, with a descending chromatic turnaround.",
    "The PHONES return gives the phrase a thicker nonlinear edge, with DRIVE still zero. Short attacks versus the held closing root expose the mixer/preamp's behaviour. Match listening gain on a return-off repeat; a louder take alone is not the distortion test.")
PATCHES[23]["hardware"] = base.PATCHES[23]["hardware"]
PATCHES[22]["hardware"] = base.PATCHES[22]["hardware"]
for n in [19, 20, 21, 22]:
    PATCHES[n - 1]["hardware"] = base.PATCHES[n - 1]["hardware"]
PATCHES[1]["purpose"] = "OSC 2 alone, 16' shark tooth: waveform colour in a D-Dorian groove."
PATCHES[11]["purpose"] = "Independent filter/loudness attack in a three-note swell phrase."
PATCHES[17]["purpose"] = "Pink noise excites tracking resonance: a breathy filter melody."


def guide():
    text = """# CA-72 musical comparisons: round 2

24 original musical examples for comparing CA-72 with a Behringer Model D. This round
uses hooks, syncopation, call and response, register jumps and contrasting gate lengths.
The first diagnostic round remains separate. No backing tracks or effects conceal the
instrument: every example is dry mono and MIDI holds at most one key at a time.

07/10 last **10 seconds**, with a complete bottom-to-top-to-bottom cutoff sweep.
The other 22 last **4.8 seconds including the tail**. Tempo is **120 BPM**, velocity
always **100**, channel **1**. Accents come from timing, register, rests and gate length;
neither instrument needs velocity sensitivity. C4 means MIDI 60 here; Live may label that
key C3. The MIDI numbers are definitive. 04/05 deliberately share the same nominal sounding
melody and rhythm, voiced through different pulse shapes and oscillator ranges.

## Load and record

Copy the individual `presets/` files into your CA-72 user library and search `MUS` or
`musical`. `bank.toml` is the renderer's source, not a user-library file. The normal folders
are `~/.local/share/Idle Foundry/CA-72/Presets/` on Linux,
`~/Library/Application Support/Idle Foundry/CA-72/Presets/` on macOS and
`%APPDATA%\\Idle Foundry\\CA-72\\Presets\\` on Windows. A shared-library override changes these.

The separate Round 2 Ableton project contains matching MIDI, listening and raw references,
numbered locators and an empty mono hardware-recording lane. Choose your MIDI output,
channel 1 and mono audio input. Configure each hardware patch while stopped; it has no
preset recall. All reference clips have Warp, fades and pitch changes off.

Use MONO, priority LOW, RETRIGGER off. UNISON/DOUBLE, ENTROPY/WIDTH/INNER/DRIVE are zero;
LEVEL is 0 dB and LOCK is off. QUALITY is ULTRA for the renders. Ordinary oscillator
mismatch and drift remain. MAIN OUTPUT is on, TUNE 0, A-440 off. Patch knobs use 0–10
travel positions rather than seconds; integer positions are on GUI lines. CUTOFF uses
-5 (bottom stop) through 0 (centre) to +5 (top stop). OSC 2 FREQUENCY and OSC 3 with CONTROL
ON use true semitones from nominal centre; files store the calibrated pot conversion.
With OSC 3 CONTROL OFF, FREQUENCY gives a dial position, not a semitone interval: match
the actual LFO rate in Hz. The +10-cent detune is the deliberate between-line exception.

For the hardware, set both filter and loudness DECAY switches to the listed DECAY state.
LFO/audio-rate examples use OSC 3, NOISE/MOD SRC and the modulation mix; leave the separate
hardware LFO and Filter-EG source unused. Start MOD DEPTH at 10 with the listed CC1 for
19–22, then match measured excursion/rate. A free-running LFO is not tempo-synchronised.

**White Lotus 13:** every factory value is preserved, including its fine settings,
VOLUME 9.78, inactive mixer levels and inactive AUTO GAIN at DRIVE 0. Before its take,
clear incoming modulation with CC1=0, then load CA-72 White Lotus and set the hardware
MOD DEPTH knob to the matched depth. Do that reset before setting the hardware knob.
The performance MIDI sends no CC1. Live's automatic controller reset at clip start is
disabled in the project. Leave the wheel/depth control untouched during the take.

## The musical ideas

| # | Preset | Phrase | What it reveals |
|---|---|---|---|
"""
    for p in PATCHES:
        text += f"| {p['number']:02} | {p['name'][7:]} | {p['musical_intent']} | {p['purpose']} |\n"
    text += """
## Full-range sweeps

07 has two detuned 8' saws and EMPHASIS 0. 10 has 32'/16'/8' layers and EMPHASIS 8.
Both repeat the same riff on opening and closing, with a tonic hold fully open. Both
keyboard tracking switches, contour amount and filter modulation are OFF/0: only the
cutoff knob adds filter movement. The notes and loudness gates still articulate the riff.

| Relative time | Cutoff | Action |
|---|---|---|
| 0.00 s | -5 | Bottom stop |
| 0.40 s | -5 | Start opening |
| 2.40 s | 0 | Centre |
| 4.40 s | +5 | Top stop |
| 5.40 s | +5 | Start closing |
| 7.40 s | 0 | Centre |
| 9.40 s | -5 | Bottom stop |

Move linearly in knob position between anchors. MIDI markers and clip annotations are
timing cues; they do not turn the hardware cutoff knob. Draw CA-72 host automation or
perform the gesture manually. The reference render updates cutoff at most 1 ms apart.
The final key releases at 9.50 s, leaving 0.50 s before the 10-second clip ends.

## Exact patches and performance

Every note is written as **start / gate length / key (MIDI number)** in seconds.
Multiple oscillator intervals stay inside one mono voice. Disabled mixer sources have
their switches off and levels 0, except the factory's saved inactive White Lotus levels.
"""
    for p in PATCHES:
        v = p["values"]
        text += f"\n### {p['number']:02}. {p['name'][7:]}\n\n{p['musical_intent']}\n\n"
        text += f"- Duration: {p['seconds']:g} s including release. Oscillators: {base.oscillator_description(v)}. OSC 3 CONTROL {'ON' if v['osc3_control'] else 'OFF'}.\n"
        text += f"- Filter: {'HI' if v['filter_mode'] else 'LO'}, CUTOFF {v['cutoff']:+g}, EMPHASIS {v['emphasis']}, CONTOUR {v['contour_amount']}; tracking K1/K2 {v['keyboard_control_1']}/{v['keyboard_control_2']}.\n"
        text += f"- ADS (attack / decay / sustain): filter {v['filter_attack']} / {v['filter_decay']} / {v['filter_sustain']}; loudness {v['loudness_attack']} / {v['loudness_decay']} / {v['loudness_sustain']}. DECAY {'ON' if v['decay_on'] else 'OFF'}.\n"
        text += f"- Modulation: MIX {v['mod_mix']}; OSC {'ON' if v['osc_mod'] else 'OFF'}, FILTER {'ON' if v['filter_mod'] else 'OFF'}; CC1 {'absent; factory depth retained' if p['cc1'] is None else str(p['cc1'])+' / 127'}.\n"
        text += f"- GLIDE {v['glide']} ({'ON' if v['glide_on'] else 'OFF'}); VOLUME {v['volume']}; FEEDBACK {v['feedback']}.\n"
        if v["osc3_range"] == 0:
            wave = "reverse saw" if v["osc3_waveform"] == 1 else base.WAVES[v["osc3_waveform"]]
            if p.get("frequency_storage") == "pot":
                frequency = "factory pot " + str(v["osc3_frequency"])
            elif v["osc3_control"]:
                frequency = f"{v['osc3_frequency']:+g} true semitones"
            else:
                frequency = f"FREQUENCY dial {v['osc3_frequency']:+g} (CONTROL OFF; match actual LFO rate in Hz)"
            text += f"- OSC 3 modulation source: LO, {wave}, {frequency}, mixer {'ON' if v['osc3_on'] else 'OFF'}.\n"
        if v["noise_on"]:
            text += f"- Noise: {'PINK' if v['noise_type'] else 'WHITE'}, level {v['noise_volume']}.\n"
        if v["ext_on"]:
            text += f"- EXTERNAL INPUT ON, level {v['ext_volume']}.\n"
        text += "- Notes: " + "; ".join(f"{at:.3f} / {gate:.3f} / {base.note_name(key)} ({key})" for at, gate, key in p["notes"]) + ".\n"
        text += f"\nListen for: {p['listen']}\n"
        if p["hardware"]:
            text += f"\nHardware: {p['hardware']}\n"
        name = quote(p["name"])
        text += f"\n[Preset](presets/{name}.toml) · [MIDI](midi/{name}.mid) · [Listen](audio/listening/{name}.wav) · [Raw](audio/raw/{name}.wav) · [Panel](panels/{name}.png)\n"
    text += """
## Informative A/B takes

Keep the phrase, input gain and every other control fixed when isolating a feature.
Repeat 07 with OSC2 muted to isolate a single saw, or with cutoff fixed +5 to hear beating.
Repeat 10 at EMPHASIS 0/8 using the same sweep. Compare 15 in HI/LO. Repeat 16 at GLIDE
0/4. Compare 17 changing only WHITE/PINK. Repeat 19 an octave higher for LFO-rate invariance;
20 changing only OSC3 shape; 21 with OSC3 CONTROL off/on; 22 at MOD MIX 0/5/10. Repeat 24
with the return absent/present, levelling listening copies afterwards. 04/05 share sounding
notes but change both pulse shape and oscillator: use a same-oscillator repeat to isolate
waveform alone. The first diagnostic round supplies the more isolated held-note tests.

Warm and tune the hardware before recording. Keep raw dry mono takes at 48 kHz, with fixed
analogue input gain and headroom. Listening copies use only a constant gain, targeting
-18 LUFS maximum momentary where peak headroom allows, capped at -1 dBTP. No limiter,
per-note normalisation or backing music. Compare pitch/intervals, harmonic colour, gate
and contour shape, modulation rate/excursion and feedback behaviour; noise and drifting
oscillators will not null sample for sample. These renders do not establish hardware fidelity.

23 uses the same 4.8-second 220 Hz external probe, four harmonics, peak 0.1, equivalent
to 0.5 V peak under CA-72's 1.0 = 5 V convention. Start it at clip time zero and match the
actual hardware input voltage. 24 uses a PHONES channel into EXT IN to approximate the
pre-main-volume loop; the automatic main-output return has a different volume dependency.

The set covers all seven nonempty oscillator combinations, all six ranges/wave positions,
four filter-tracking states, both noise colours, contours, glide, LO/HI, self-oscillation,
OSC3 LFO/audio-rate modulation, modulation mix, external audio and feedback. Synth features
are unchanged; most articulation settings are now chosen for the riff. White Lotus is exact.

## Regeneration

```sh
python3 scripts/musical_comparison_presets.py
CARGO_BUILD_JOBS=4 nice -n19 ionice -c3 cargo run --release -p ca72-plugin --example comparison_pack -- --pack docs/musical-comparison-presets
```

The notes are original phrases composed for this second round. Source patch values come
from the first pack and current CA-72 0.2.0 factory White Lotus. Render seed is 72, ULTRA,
48 kHz mono. The shared generator checks coverage, mono timing, exact factory values,
neutral extensions and the full sweep endpoints. No hardware recording is included.
"""
    (OUT / "README.md").write_text(text)


def main():
    for p in PATCHES:
        for start, gate, _ in p["notes"]:
            for t in (start, gate, start + gate):
                if not math.isfinite(t) or abs(t * 1920 - round(t * 1920)) > 1e-8:
                    raise ValueError(f"Note timing must match the MIDI grid: {p['name']}")
    base.OUT = OUT
    base.PATCHES = PATCHES
    base.guide = guide
    base.main()


if __name__ == "__main__":
    main()
