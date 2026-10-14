# CA-72 musical comparisons: round 2

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
`%APPDATA%\Idle Foundry\CA-72\Presets\` on Windows. A shared-library override changes these.

The separate Round 2 Ableton project contains matching MIDI, listening and raw references,
numbered locators and an empty mono hardware-recording lane. Choose your MIDI output,
channel 1 and mono audio input. Configure each hardware patch while stopped; it has no
preset recall. All reference clips have Warp, fades and pitch changes off.

Use MONO, priority LOW, RETRIGGER off. UNISON/DOUBLE, ENTROPY/WIDTH/INNER/DRIVE are zero;
LEVEL is 0 dB and LOCK is off. QUALITY is ULTRA for the renders. Ordinary oscillator
mismatch and drift remain. MAIN OUTPUT is on, TUNE 0, A-440 off. Patch knobs use 0–10
travel positions rather than seconds; integer positions are on GUI lines. CUTOFF uses
-5 (bottom stop) through 0 (centre) to +5 (top stop). FREQUENCY intervals below are true
semitones from nominal centre; files store the calibrated pot conversion. The +10-cent
detune is the deliberate between-line exception.

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
| 01 | Pocket Sub | Syncopated F-minor bass, ending with a two-octave sub drop. | OSC 1 alone, triangle, 32': low-frequency shape and tuning. |
| 02 | Tooth Groove | A D-Dorian bass hook: the natural sixth lifts the second half. | OSC 2 alone, 16' shark tooth: waveform colour in a D-Dorian groove. |
| 03 | Reverse Reply | A G-minor call and response, with a high answer and a low cadence. | OSC 3 alone, reverse saw, 8', keyboard control on. |
| 04 | Arcade Lead | A climbing C-minor pickup answers with a falling line and a high final C. | OSC 1 alone, square, 4': octave-up oscillator tracking. |
| 05 | Glass Pulse | The same lead melody as 04, at the same nominal sounding pitches. | OSC 2 alone, wide pulse, 2': two-octave shift and upper-register harmonics. |
| 06 | Top Shelf | A high F-major hook with a quick flourish and a longer resolving note. | OSC 3 alone, narrow pulse, 8': high keyboard register. |
| 07 | Sweep Groove | The same syncopated minor-pentatonic riff opens and closes with the filter. | Full-range low-pass sweep with EMPHASIS 0; two 8' saws ten cents apart. |
| 08 | Octave Bounce | A buoyant F-minor bass that hops up, then falls back to the sub. | OSC 1 + 3, a 16' triangle over a 32' square. |
| 09 | Parallel Fifths | An open-fifth C-minor hook with a lifted octave answer. | OSC 2 + 3, 8' triangle and saw; OSC 3 seven semitones higher. |
| 10 | Resonant Run | The same syncopated minor-pentatonic riff opens and closes with the filter. | Full-range low-pass sweep with EMPHASIS 8; three 32'/16'/8' octave layers. |
| 11 | Rubber Bass | A clipped, syncopated F-minor bass with a chromatic turnaround. | Low-pass filter contour and zero loudness sustain: a short two-saw bass. |
| 12 | Slow Reply | A three-note C–G–F answer whose swells overlap in colour, inside a mono voice. | Independent filter/loudness attack in a three-note swell phrase. |
| 13 | White Lotus | An original C-minor pentatonic hook with a higher call and a low answer. | The unchanged factory White Lotus: narrow-pulse bass, a third-harmonic filter ring and OSC 3 LO filter modulation. |
| 14 | Ladder Melody | A soft F-major call and a two-octave whistle answer. | Filter self-oscillation, no VCO in the mixer, full keyboard tracking. |
| 15 | High Pass Funk | A wiry F-minor funk line with a high-register reply. | High-pass filter, resonance, contour, two-thirds keyboard tracking. |
| 16 | Sliding Hook | A joined C-minor hook with small scoops and a wide octave slide. | GLIDE with a continuous monophonic phrase and single triggering. |
| 17 | Noise Break | A two-bar noise break with offbeat doubles and a spaced closing hit. | White noise through fast contours: percussion without oscillators. |
| 18 | Pink Flute | A breathy resonant-filter melody, with an octave reply. | Pink noise excites tracking resonance: a breathy filter melody. |
| 19 | Wavering Lead | A singing C-minor line, framed by two longer vibrato notes. | OSC 3 as an untracked LO triangle LFO, pitch destination only. |
| 20 | Filter Ostinato | A clipped F-minor ostinato that plays against the free-running filter rhythm. | OSC 3 as an untracked LO reverse-saw LFO, filter destination only. |
| 21 | Metal Arpeggio | A C-minor-nine arpeggio with a displaced answer and an octave landing. | Keyboard-tracked OSC 3 at audio rate modulates oscillator pitch. |
| 22 | Broken Square | A lurching D-minor riff with a flattened-fifth passing note. | Midpoint MOD MIX: LO square OSC 3 plus noise, both destinations. |
| 23 | External Rhythm | A rhythmic vowel-like phrase made by gating and filtering the external source. | External audio through preamp, low-pass filter and keyboard-gated contours. |
| 24 | Feedback Stomp | An overdriven F-minor stomp, with a descending chromatic turnaround. | External-input feedback and mixer overload from the hardware-accessible loop. |

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

### 01. Pocket Sub

Syncopated F-minor bass, ending with a two-octave sub drop.

- Duration: 4.8 s including release. Oscillators: 1: 32' triangle, level 6. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.325 / F4 (65); 0.575 / 0.125 / F4 (65); 0.950 / 0.200 / C5 (72); 1.325 / 0.200 / G#4 (68); 1.700 / 0.350 / F4 (65); 2.200 / 0.200 / C4 (60); 2.575 / 0.125 / D#4 (63); 2.950 / 0.200 / F4 (65); 3.325 / 0.225 / F3 (53); 3.700 / 0.450 / F2 (41).

Listen for: The main hook sounds at F2/Ab2/C3; the closing keys F3 and F2 sound F1 and F0 on 32'. The final low note is a deliberate range probe, after an audible bass line. Listen to triangle purity and the difference between a rounded gate and a note gap.

[Preset](presets/MUS%2001%20Pocket%20Sub.toml) · [MIDI](midi/MUS%2001%20Pocket%20Sub.mid) · [Listen](audio/listening/MUS%2001%20Pocket%20Sub.wav) · [Raw](audio/raw/MUS%2001%20Pocket%20Sub.wav) · [Panel](panels/MUS%2001%20Pocket%20Sub.png)

### 02. Tooth Groove

A D-Dorian bass hook: the natural sixth lifts the second half.

- Duration: 4.8 s including release. Oscillators: 2: 16' shark tooth, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.300 / D3 (50); 0.575 / 0.125 / A3 (57); 0.825 / 0.125 / F3 (53); 1.075 / 0.300 / D3 (50); 1.575 / 0.125 / A3 (57); 1.825 / 0.225 / B3 (59); 2.200 / 0.350 / C4 (60); 2.700 / 0.125 / B3 (59); 2.950 / 0.250 / A3 (57); 3.325 / 0.125 / F3 (53); 3.575 / 0.125 / E3 (52); 3.825 / 0.325 / D3 (50).

Listen for: OSC 2 alone at 16' gives this line a woody asymmetric edge. Hear repeated D attacks, the B-natural turn, and the return home; the pitch centre stays exposed without detuning or modulation.

[Preset](presets/MUS%2002%20Tooth%20Groove.toml) · [MIDI](midi/MUS%2002%20Tooth%20Groove.mid) · [Listen](audio/listening/MUS%2002%20Tooth%20Groove.wav) · [Raw](audio/raw/MUS%2002%20Tooth%20Groove.wav) · [Panel](panels/MUS%2002%20Tooth%20Groove.png)

### 03. Reverse Reply

A G-minor call and response, with a high answer and a low cadence.

- Duration: 4.8 s including release. Oscillators: 3: 8' reverse saw, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.400 / G3 (55); 0.825 / 0.125 / A#3 (58); 1.075 / 0.250 / C4 (60); 1.450 / 0.450 / D4 (62); 2.075 / 0.100 / C4 (60); 2.325 / 0.125 / A#3 (58); 2.575 / 0.400 / G3 (55); 3.200 / 0.125 / F4 (65); 3.450 / 0.250 / D4 (62); 3.825 / 0.325 / G3 (55).

Listen for: OSC 3's reverse saw carries the hook with CONTROL on. Compare its harmonic balance and attack to an ordinary saw; polarity is a waveform check rather than a promise of a radically different sound.

[Preset](presets/MUS%2003%20Reverse%20Reply.toml) · [MIDI](midi/MUS%2003%20Reverse%20Reply.mid) · [Listen](audio/listening/MUS%2003%20Reverse%20Reply.wav) · [Raw](audio/raw/MUS%2003%20Reverse%20Reply.wav) · [Panel](panels/MUS%2003%20Reverse%20Reply.png)

### 04. Arcade Lead

A climbing C-minor pickup answers with a falling line and a high final C.

- Duration: 4.8 s including release. Oscillators: 1: 4' square, level 6. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.250 / C4 (60); 0.575 / 0.125 / D#4 (63); 0.825 / 0.250 / F4 (65); 1.200 / 0.375 / G4 (67); 1.700 / 0.125 / A#4 (70); 1.950 / 0.250 / G4 (67); 2.325 / 0.250 / F4 (65); 2.700 / 0.125 / D#4 (63); 2.950 / 0.250 / C4 (60); 3.325 / 0.125 / G4 (67); 3.575 / 0.125 / A#4 (70); 3.825 / 0.325 / C5 (72).

Listen for: The 4' square sounds an octave above the played keys: this melody reaches sounding C6. Listen for hollow odd harmonics and the clean separation between the short pickups and longer answer.

[Preset](presets/MUS%2004%20Arcade%20Lead.toml) · [MIDI](midi/MUS%2004%20Arcade%20Lead.mid) · [Listen](audio/listening/MUS%2004%20Arcade%20Lead.wav) · [Raw](audio/raw/MUS%2004%20Arcade%20Lead.wav) · [Panel](panels/MUS%2004%20Arcade%20Lead.png)

### 05. Glass Pulse

The same lead melody as 04, at the same nominal sounding pitches.

- Duration: 4.8 s including release. Oscillators: 2: 2' wide pulse, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.250 / C3 (48); 0.575 / 0.125 / D#3 (51); 0.825 / 0.250 / F3 (53); 1.200 / 0.375 / G3 (55); 1.700 / 0.125 / A#3 (58); 1.950 / 0.250 / G3 (55); 2.325 / 0.250 / F3 (53); 2.700 / 0.125 / D#3 (51); 2.950 / 0.250 / C3 (48); 3.325 / 0.125 / G3 (55); 3.575 / 0.125 / A#3 (58); 3.825 / 0.325 / C4 (60).

Listen for: The 2' wide pulse uses input notes an octave below 04 to reach the same sounding melody. The rhythm is identical, making its brighter pulse-width colour easy to compare with the square while preserving the separate-oscillator test.

[Preset](presets/MUS%2005%20Glass%20Pulse.toml) · [MIDI](midi/MUS%2005%20Glass%20Pulse.mid) · [Listen](audio/listening/MUS%2005%20Glass%20Pulse.wav) · [Raw](audio/raw/MUS%2005%20Glass%20Pulse.wav) · [Panel](panels/MUS%2005%20Glass%20Pulse.png)

### 06. Top Shelf

A high F-major hook with a quick flourish and a longer resolving note.

- Duration: 4.8 s including release. Oscillators: 3: 8' narrow pulse, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.275 / F5 (77); 0.575 / 0.125 / A5 (81); 0.825 / 0.250 / C6 (84); 1.200 / 0.125 / A#5 (82); 1.450 / 0.275 / A5 (81); 1.825 / 0.125 / G5 (79); 2.075 / 0.275 / F5 (77); 2.450 / 0.125 / G5 (79); 2.700 / 0.125 / A5 (81); 2.950 / 0.250 / C6 (84); 3.325 / 0.125 / A#5 (82); 3.575 / 0.550 / F5 (77).

Listen for: F5 through C6 is the top of the original keyboard. The narrow pulse should remain recognisably nasal across the run; the held final F gives time to hear pitch stability after the fast high notes.

[Preset](presets/MUS%2006%20Top%20Shelf.toml) · [MIDI](midi/MUS%2006%20Top%20Shelf.mid) · [Listen](audio/listening/MUS%2006%20Top%20Shelf.wav) · [Raw](audio/raw/MUS%2006%20Top%20Shelf.wav) · [Panel](panels/MUS%2006%20Top%20Shelf.png)

### 07. Sweep Groove

The same syncopated minor-pentatonic riff opens and closes with the filter.

- Duration: 10 s including release. Oscillators: 1: 8' saw, level 4; 2: 8' saw, level 4, freq +0.1 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -5, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 1 / 10. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.125 / A3 (57); 0.400 / 0.250 / A3 (57); 0.775 / 0.125 / A3 (57); 0.900 / 0.250 / E4 (64); 1.275 / 0.125 / G4 (67); 1.650 / 0.250 / E4 (64); 2.025 / 0.125 / D4 (62); 2.400 / 0.375 / A3 (57); 2.900 / 0.250 / C4 (60); 3.275 / 0.125 / D4 (62); 3.400 / 0.250 / E4 (64); 3.900 / 0.375 / A3 (57); 4.400 / 1.000 / A3 (57); 5.400 / 0.250 / A3 (57); 5.775 / 0.125 / A3 (57); 5.900 / 0.250 / E4 (64); 6.275 / 0.125 / G4 (67); 6.650 / 0.250 / E4 (64); 7.025 / 0.125 / D4 (62); 7.400 / 0.375 / A3 (57); 7.900 / 0.250 / C4 (60); 8.275 / 0.125 / D4 (62); 8.400 / 0.250 / E4 (64); 8.900 / 0.375 / A3 (57); 9.400 / 0.100 / A3 (57).

Listen for: Four seconds opening, a one-second tonic hold fully open, then the same four-second riff closing. Repeating the pitches and rhythm makes the change in brightness audible without replacing it with a scale exercise. EMPHASIS 0 gives a gentle opening; the held A3 at the top exposes the ten-cent saw beating. Use a tuner/beat-rate match for the between-line detune setting.

Hardware: Turn only CUTOFF from -5 to +5 and back using the timed points. Both keyboard-tracking switches are OFF so note changes do not add a cutoff sweep. Contour amount and filter modulation stay at 0. The MIDI does not turn the physical knob.

[Preset](presets/MUS%2007%20Sweep%20Groove.toml) · [MIDI](midi/MUS%2007%20Sweep%20Groove.mid) · [Listen](audio/listening/MUS%2007%20Sweep%20Groove.wav) · [Raw](audio/raw/MUS%2007%20Sweep%20Groove.wav) · [Panel](panels/MUS%2007%20Sweep%20Groove.png)

### 08. Octave Bounce

A buoyant F-minor bass that hops up, then falls back to the sub.

- Duration: 4.8 s including release. Oscillators: 1: 16' triangle, level 6; 3: 32' square, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -1, EMPHASIS 0, CONTOUR 0; tracking K1/K2 0/1.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.300 / F3 (53); 0.575 / 0.125 / F3 (53); 0.950 / 0.250 / C4 (60); 1.325 / 0.125 / D#4 (63); 1.575 / 0.275 / F4 (65); 1.950 / 0.125 / C4 (60); 2.325 / 0.250 / A#3 (58); 2.700 / 0.125 / C4 (60); 2.950 / 0.250 / F3 (53); 3.325 / 0.125 / C3 (48); 3.575 / 0.550 / F3 (53).

Listen for: The 16' triangle sits over the 32' square. Short repeated roots expose their blend; octave leaps show that the two layers keep their ratio. Two-thirds filter tracking changes colour across the line.

[Preset](presets/MUS%2008%20Octave%20Bounce.toml) · [MIDI](midi/MUS%2008%20Octave%20Bounce.mid) · [Listen](audio/listening/MUS%2008%20Octave%20Bounce.wav) · [Raw](audio/raw/MUS%2008%20Octave%20Bounce.wav) · [Panel](panels/MUS%2008%20Octave%20Bounce.png)

### 09. Parallel Fifths

An open-fifth C-minor hook with a lifted octave answer.

- Duration: 4.8 s including release. Oscillators: 2: 8' triangle, level 5, freq +0 st; 3: 8' saw, level 4, freq +7 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +2, EMPHASIS 0, CONTOUR 0; tracking K1/K2 1/1.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.350 / C3 (48); 0.700 / 0.125 / D3 (50); 0.950 / 0.250 / D#3 (51); 1.325 / 0.375 / G3 (55); 1.825 / 0.200 / A#3 (58); 2.200 / 0.350 / C4 (60); 2.700 / 0.125 / A#3 (58); 2.950 / 0.250 / G3 (55); 3.325 / 0.125 / D#3 (51); 3.575 / 0.550 / C3 (48).

Listen for: OSC 2 triangle and OSC 3 saw play a parallel fifth inside one voice. This gives a harmonised sound while MIDI remains monophonic. Tune the true +7-semitone interval on the hardware before the take.

[Preset](presets/MUS%2009%20Parallel%20Fifths.toml) · [MIDI](midi/MUS%2009%20Parallel%20Fifths.mid) · [Listen](audio/listening/MUS%2009%20Parallel%20Fifths.wav) · [Raw](audio/raw/MUS%2009%20Parallel%20Fifths.wav) · [Panel](panels/MUS%2009%20Parallel%20Fifths.png)

### 10. Resonant Run

The same syncopated minor-pentatonic riff opens and closes with the filter.

- Duration: 10 s including release. Oscillators: 1: 32' saw, level 4; 2: 16' shark tooth, level 4, freq +0 st; 3: 8' square, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -5, EMPHASIS 8, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 1 / 10. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.125 / F3 (53); 0.400 / 0.250 / F3 (53); 0.775 / 0.125 / F3 (53); 0.900 / 0.250 / C4 (60); 1.275 / 0.125 / D#4 (63); 1.650 / 0.250 / C4 (60); 2.025 / 0.125 / A#3 (58); 2.400 / 0.375 / F3 (53); 2.900 / 0.250 / G#3 (56); 3.275 / 0.125 / A#3 (58); 3.400 / 0.250 / C4 (60); 3.900 / 0.375 / F3 (53); 4.400 / 1.000 / F3 (53); 5.400 / 0.250 / F3 (53); 5.775 / 0.125 / F3 (53); 5.900 / 0.250 / C4 (60); 6.275 / 0.125 / D#4 (63); 6.650 / 0.250 / C4 (60); 7.025 / 0.125 / A#3 (58); 7.400 / 0.375 / F3 (53); 7.900 / 0.250 / G#3 (56); 8.275 / 0.125 / A#3 (58); 8.400 / 0.250 / C4 (60); 8.900 / 0.375 / F3 (53); 9.400 / 0.100 / F3 (53).

Listen for: Four seconds opening, a one-second tonic hold fully open, then the same four-second riff closing. Repeating the pitches and rhythm makes the change in brightness audible without replacing it with a scale exercise. EMPHASIS 8 gives the riff a travelling resonant bite. Input F3 voices F1/F2/F3 in the three oscillators.

Hardware: Turn only CUTOFF from -5 to +5 and back using the timed points. Both keyboard-tracking switches are OFF so note changes do not add a cutoff sweep. Contour amount and filter modulation stay at 0. The MIDI does not turn the physical knob.

[Preset](presets/MUS%2010%20Resonant%20Run.toml) · [MIDI](midi/MUS%2010%20Resonant%20Run.mid) · [Listen](audio/listening/MUS%2010%20Resonant%20Run.wav) · [Raw](audio/raw/MUS%2010%20Resonant%20Run.wav) · [Panel](panels/MUS%2010%20Resonant%20Run.png)

### 11. Rubber Bass

A clipped, syncopated F-minor bass with a chromatic turnaround.

- Duration: 4.8 s including release. Oscillators: 1: 16' saw, level 5; 2: 16' saw, level 5, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -3, EMPHASIS 6, CONTOUR 6; tracking K1/K2 1/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 0. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.200 / F2 (41); 0.575 / 0.100 / F2 (41); 0.825 / 0.150 / C3 (48); 1.075 / 0.100 / D#3 (51); 1.450 / 0.200 / F3 (53); 1.825 / 0.100 / C3 (48); 2.075 / 0.100 / A#2 (46); 2.325 / 0.200 / F2 (41); 2.700 / 0.100 / C3 (48); 2.950 / 0.150 / D#3 (51); 3.325 / 0.100 / D3 (50); 3.575 / 0.550 / F2 (41).

Listen for: The short filter contour gives every attack a rubbery vowel; sustain zero keeps the final long key from becoming a held organ tone. A mixture of short gates and rests exposes decay and re-arming.

[Preset](presets/MUS%2011%20Rubber%20Bass.toml) · [MIDI](midi/MUS%2011%20Rubber%20Bass.mid) · [Listen](audio/listening/MUS%2011%20Rubber%20Bass.wav) · [Raw](audio/raw/MUS%2011%20Rubber%20Bass.wav) · [Panel](panels/MUS%2011%20Rubber%20Bass.png)

### 12. Slow Reply

A three-note C–G–F answer whose swells overlap in colour, inside a mono voice.

- Duration: 4.8 s including release. Oscillators: 1: 8' triangle, level 6; 2: 8' saw, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -3, EMPHASIS 4, CONTOUR 6; tracking K1/K2 0/1.
- ADS (attack / decay / sustain): filter 5 / 3 / 6; loudness 3 / 3 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 1.125 / C4 (60); 1.575 / 1.125 / G4 (67); 2.825 / 1.325 / F4 (65).

Listen for: Longer gates let the loudness rise before the filter finishes opening. The falling final interval makes the swell feel like a phrase; the remaining tail exposes both decay contours.

[Preset](presets/MUS%2012%20Slow%20Reply.toml) · [MIDI](midi/MUS%2012%20Slow%20Reply.mid) · [Listen](audio/listening/MUS%2012%20Slow%20Reply.wav) · [Raw](audio/raw/MUS%2012%20Slow%20Reply.wav) · [Panel](panels/MUS%2012%20Slow%20Reply.png)

### 13. White Lotus

An original C-minor pentatonic hook with a higher call and a low answer.

- Duration: 4.8 s including release. Oscillators: 1: 8' narrow pulse, level 10. OSC 3 CONTROL OFF.
- Filter: LO, CUTOFF -2.14, EMPHASIS 7, CONTOUR 0; tracking K1/K2 1/1.
- ADS (attack / decay / sustain): filter 0 / 3 / 10; loudness 0.5 / 3.5 / 8.5. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER ON; CC1 absent; factory depth retained.
- GLIDE 0 (OFF); VOLUME 9.78; FEEDBACK 0.
- OSC 3 modulation source: LO, triangle, factory pot 7.47627, mixer OFF.
- Notes: 0.200 / 0.375 / C3 (48); 0.700 / 0.125 / D#3 (51); 0.950 / 0.250 / G3 (55); 1.450 / 0.125 / A#3 (58); 1.700 / 0.375 / G3 (55); 2.200 / 0.375 / C4 (60); 2.700 / 0.125 / A#3 (58); 2.950 / 0.250 / G3 (55); 3.450 / 0.125 / D#3 (51); 3.700 / 0.450 / C3 (48).

Listen for: The longer C3/C4 gates expose the roughly 11 Hz filter motion and third-harmonic ring; the short connecting notes turn it into a hook. Every saved factory setting remains exact, including its fine positions and output volume.

Hardware: Set OSC 1 to 8' narrow pulse, mixer 10; all other mixer sources OFF. Use LO filter, cutoff -2.14, emphasis 7, both tracking switches ON, contour amount 0. OSC 3 is a triangle at LO, CONTROL OFF and mixer OFF; tune its rate to approximately 11 Hz. FILTER MODULATION ON, OSCILLATOR MODULATION OFF, MOD MIX 0. Before recording, send CC 1 = 0 to clear incoming MIDI modulation, then load White Lotus on CA-72 and move the hardware MOD DEPTH knob to match its excursion. A MIDI wheel message takes over the hardware depth until that knob moves, so complete resets before setting the hardware knob. The shared clip sends no CC 1; leave the modulation controller untouched during the take, preserving the CA-72 preset's exact 0.17 and the matched hardware knob depth.

[Preset](presets/MUS%2013%20White%20Lotus.toml) · [MIDI](midi/MUS%2013%20White%20Lotus.mid) · [Listen](audio/listening/MUS%2013%20White%20Lotus.wav) · [Raw](audio/raw/MUS%2013%20White%20Lotus.wav) · [Panel](panels/MUS%2013%20White%20Lotus.png)

### 14. Ladder Melody

A soft F-major call and a two-octave whistle answer.

- Duration: 4.8 s including release. Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 10, CONTOUR 0; tracking K1/K2 1/1.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Noise: WHITE, level 1.
- Notes: 0.200 / 0.375 / F3 (53); 0.700 / 0.125 / A3 (57); 0.950 / 0.250 / C4 (60); 1.325 / 0.300 / F4 (65); 1.825 / 0.200 / C4 (60); 2.200 / 0.375 / F5 (77); 2.700 / 0.125 / C5 (72); 2.950 / 0.250 / A4 (69); 3.325 / 0.125 / F4 (65); 3.575 / 0.550 / F3 (53).

Listen for: All VCO mixer switches are off: the ladder itself supplies the pitched tone. Full tracking makes the relative melody possible. Printed cutoff -2 is not a concert-pitch promise; tune once before an optional pitch-matched pass, never between notes.

[Preset](presets/MUS%2014%20Ladder%20Melody.toml) · [MIDI](midi/MUS%2014%20Ladder%20Melody.mid) · [Listen](audio/listening/MUS%2014%20Ladder%20Melody.wav) · [Raw](audio/raw/MUS%2014%20Ladder%20Melody.wav) · [Panel](panels/MUS%2014%20Ladder%20Melody.png)

### 15. High Pass Funk

A wiry F-minor funk line with a high-register reply.

- Duration: 4.8 s including release. Oscillators: 1: 8' wide pulse, level 5; 3: 4' reverse saw, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: HI, CUTOFF +1, EMPHASIS 6, CONTOUR 3; tracking K1/K2 0/1.
- ADS (attack / decay / sustain): filter 0 / 3 / 2; loudness 0 / 2 / 6. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.200 / F3 (53); 0.575 / 0.125 / F3 (53); 0.825 / 0.125 / C4 (60); 1.075 / 0.200 / D#4 (63); 1.450 / 0.200 / F4 (65); 1.825 / 0.125 / C4 (60); 2.200 / 0.200 / F5 (77); 2.575 / 0.125 / C5 (72); 2.825 / 0.200 / F4 (65); 3.200 / 0.125 / D#4 (63); 3.575 / 0.550 / F3 (53).

Listen for: The only HI patch keeps the riff's upper harmonics while thinning its bottom. Play the same pattern in LO for the controlled mode comparison; the wide pulse/reverse-saw combination and two-thirds tracking stay fixed.

[Preset](presets/MUS%2015%20High%20Pass%20Funk.toml) · [MIDI](midi/MUS%2015%20High%20Pass%20Funk.mid) · [Listen](audio/listening/MUS%2015%20High%20Pass%20Funk.wav) · [Raw](audio/raw/MUS%2015%20High%20Pass%20Funk.wav) · [Panel](panels/MUS%2015%20High%20Pass%20Funk.png)

### 16. Sliding Hook

A joined C-minor hook with small scoops and a wide octave slide.

- Duration: 4.8 s including release. Oscillators: 1: 8' saw, level 5; 2: 16' wide pulse, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 4, CONTOUR 4; tracking K1/K2 1/1.
- ADS (attack / decay / sustain): filter 0 / 4 / 3; loudness 0 / 2 / 10. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 4 (ON); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.500 / C3 (48); 0.700 / 0.250 / D#3 (51); 0.950 / 0.250 / F3 (53); 1.200 / 0.750 / G3 (55); 1.950 / 0.250 / A#3 (58); 2.200 / 0.500 / C4 (60); 2.700 / 0.250 / A#3 (58); 2.950 / 0.250 / G3 (55); 3.200 / 0.500 / D#3 (51); 3.700 / 0.450 / C3 (48).

Listen for: Zero-gap joins keep exactly one key at a time. GLIDE 4 lets the motif move rather than lingering on every transition. The wide jump and falling answer expose glide distance and direction; repeat with glide off, hardware multi-trigger off.

[Preset](presets/MUS%2016%20Sliding%20Hook.toml) · [MIDI](midi/MUS%2016%20Sliding%20Hook.mid) · [Listen](audio/listening/MUS%2016%20Sliding%20Hook.wav) · [Raw](audio/raw/MUS%2016%20Sliding%20Hook.wav) · [Panel](panels/MUS%2016%20Sliding%20Hook.png)

### 17. Noise Break

A two-bar noise break with offbeat doubles and a spaced closing hit.

- Duration: 4.8 s including release. Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +0, EMPHASIS 4, CONTOUR 4; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 1 / 0; loudness 0 / 1 / 0. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Noise: WHITE, level 6.
- Notes: 0.200 / 0.125 / C3 (48); 0.575 / 0.062 / C3 (48); 0.950 / 0.125 / C3 (48); 1.200 / 0.125 / C3 (48); 1.450 / 0.062 / C3 (48); 1.575 / 0.062 / C3 (48); 1.950 / 0.125 / C3 (48); 2.200 / 0.125 / C3 (48); 2.700 / 0.125 / C3 (48); 2.950 / 0.062 / C3 (48); 3.325 / 0.062 / C3 (48); 3.450 / 0.062 / C3 (48); 3.700 / 0.250 / C3 (48).

Listen for: Pitch and velocity do not accent this noise patch. The groove comes from timing and gate length. Shorter decays expose fast transient behaviour; change only WHITE to PINK on a repeat for a colour comparison.

[Preset](presets/MUS%2017%20Noise%20Break.toml) · [MIDI](midi/MUS%2017%20Noise%20Break.mid) · [Listen](audio/listening/MUS%2017%20Noise%20Break.wav) · [Raw](audio/raw/MUS%2017%20Noise%20Break.wav) · [Panel](panels/MUS%2017%20Noise%20Break.png)

### 18. Pink Flute

A breathy resonant-filter melody, with an octave reply.

- Duration: 4.8 s including release. Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 8, CONTOUR 4; tracking K1/K2 1/1.
- ADS (attack / decay / sustain): filter 1 / 3 / 6; loudness 1 / 3 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Noise: PINK, level 4.
- Notes: 0.200 / 0.625 / F4 (65); 0.950 / 0.250 / A4 (69); 1.325 / 0.300 / C5 (72); 1.825 / 0.250 / A4 (69); 2.200 / 0.625 / F5 (77); 2.950 / 0.250 / D5 (74); 3.325 / 0.750 / F4 (65).

Listen for: Pink noise excites a strong tracking resonance; there is no audible oscillator. The keys move the filter's spectral focus. Hear a breathier, noisier contour than the self-oscillating ladder in 14, rather than treating this as a perfectly tuned flute.

[Preset](presets/MUS%2018%20Pink%20Flute.toml) · [MIDI](midi/MUS%2018%20Pink%20Flute.mid) · [Listen](audio/listening/MUS%2018%20Pink%20Flute.wav) · [Raw](audio/raw/MUS%2018%20Pink%20Flute.wav) · [Panel](panels/MUS%2018%20Pink%20Flute.png)

### 19. Wavering Lead

A singing C-minor line, framed by two longer vibrato notes.

- Duration: 4.8 s including release. Oscillators: 1: 8' square, level 6. OSC 3 CONTROL OFF.
- Filter: LO, CUTOFF +2, EMPHASIS 0, CONTOUR 0; tracking K1/K2 1/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC ON, FILTER OFF; CC1 6 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- OSC 3 modulation source: LO, triangle, 3 true semitones, mixer OFF.
- Notes: 0.200 / 1.000 / C4 (60); 1.450 / 0.200 / D#4 (63); 1.825 / 0.200 / F4 (65); 2.200 / 0.375 / G4 (67); 2.700 / 0.200 / A#4 (70); 3.075 / 0.200 / G4 (67); 3.450 / 0.700 / C4 (60).

Listen for: The held ends expose the LO triangle's smooth pitch modulation; shorter middle notes give it a melodic purpose. OSC 3 CONTROL and its mixer stay off, so the LFO rate does not follow this melody.

Hardware: Select OSC 3 as source, NOISE/MOD SRC rather than the separate LFO, MOD MIX fully to OSC 3, MOD DEPTH 10. Use CC 1 = 6 as a reproducible starting depth, then match measured pitch excursion.

[Preset](presets/MUS%2019%20Wavering%20Lead.toml) · [MIDI](midi/MUS%2019%20Wavering%20Lead.mid) · [Listen](audio/listening/MUS%2019%20Wavering%20Lead.wav) · [Raw](audio/raw/MUS%2019%20Wavering%20Lead.wav) · [Panel](panels/MUS%2019%20Wavering%20Lead.png)

### 20. Filter Ostinato

A clipped F-minor ostinato that plays against the free-running filter rhythm.

- Duration: 4.8 s including release. Oscillators: 1: 16' saw, level 6. OSC 3 CONTROL OFF.
- Filter: LO, CUTOFF -1, EMPHASIS 6, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER ON; CC1 26 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- OSC 3 modulation source: LO, reverse saw, 0 true semitones, mixer OFF.
- Notes: 0.200 / 0.225 / F3 (53); 0.450 / 0.150 / F3 (53); 0.950 / 0.150 / C4 (60); 1.200 / 0.225 / F3 (53); 1.450 / 0.150 / D#4 (63); 1.825 / 0.225 / C4 (60); 2.200 / 0.225 / F3 (53); 2.450 / 0.150 / F3 (53); 2.950 / 0.150 / F4 (65); 3.200 / 0.225 / D#4 (63); 3.450 / 0.150 / C4 (60); 3.700 / 0.450 / F3 (53).

Listen for: OSC 3's LO reverse saw moves only the filter. The repeating note rhythm lets its asymmetric fall/reset drift across the groove; it is free-running, not DAW-synchronised. Compare another take changing only the LFO shape.

Hardware: Select OSC 3 and NOISE/MOD SRC, MOD DEPTH 10, CC 1 = 26. Match rate and cutoff excursion by measurement; the MIDI wheel transfer curve is not guaranteed identical on another hardware unit.

[Preset](presets/MUS%2020%20Filter%20Ostinato.toml) · [MIDI](midi/MUS%2020%20Filter%20Ostinato.mid) · [Listen](audio/listening/MUS%2020%20Filter%20Ostinato.wav) · [Raw](audio/raw/MUS%2020%20Filter%20Ostinato.wav) · [Panel](panels/MUS%2020%20Filter%20Ostinato.png)

### 21. Metal Arpeggio

A C-minor-nine arpeggio with a displaced answer and an octave landing.

- Duration: 4.8 s including release. Oscillators: 1: 8' triangle, level 6. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +3, EMPHASIS 0, CONTOUR 0; tracking K1/K2 1/1.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC ON, FILTER OFF; CC1 13 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- Notes: 0.200 / 0.200 / C4 (60); 0.450 / 0.200 / G4 (67); 0.700 / 0.200 / D#4 (63); 0.950 / 0.200 / A#4 (70); 1.200 / 0.300 / D5 (74); 1.700 / 0.200 / A#4 (70); 1.950 / 0.200 / G4 (67); 2.200 / 0.200 / D#4 (63); 2.575 / 0.200 / C4 (60); 2.825 / 0.200 / G4 (67); 3.075 / 0.200 / A#4 (70); 3.450 / 0.700 / C5 (72).

Listen for: The bright metallic sidebands follow the melody because OSC 3 CONTROL is on. OSC 3 remains out of the mixer: its contribution is audio-rate exponential FM, without ring modulation or oscillator sync.

Hardware: Select OSC 3/NOISE sources, MOD DEPTH 10 and CC 1 = 13; keep OSC 3 mixer OFF and OSC 3 CONTROL ON. Match modulation depth before comparing sidebands.

[Preset](presets/MUS%2021%20Metal%20Arpeggio.toml) · [MIDI](midi/MUS%2021%20Metal%20Arpeggio.mid) · [Listen](audio/listening/MUS%2021%20Metal%20Arpeggio.wav) · [Raw](audio/raw/MUS%2021%20Metal%20Arpeggio.wav) · [Panel](panels/MUS%2021%20Metal%20Arpeggio.png)

### 22. Broken Square

A lurching D-minor riff with a flattened-fifth passing note.

- Duration: 4.8 s including release. Oscillators: 1: 8' wide pulse, level 6. OSC 3 CONTROL OFF.
- Filter: LO, CUTOFF +0, EMPHASIS 4, CONTOUR 0; tracking K1/K2 0/0.
- ADS (attack / decay / sustain): filter 0 / 2 / 0; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 5; OSC ON, FILTER ON; CC1 13 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- OSC 3 modulation source: LO, square, 0 true semitones, mixer OFF.
- Notes: 0.200 / 0.325 / D3 (50); 0.700 / 0.150 / D3 (50); 1.075 / 0.200 / F3 (53); 1.450 / 0.250 / A3 (57); 1.825 / 0.200 / G#3 (56); 2.200 / 0.325 / D4 (62); 2.700 / 0.150 / C4 (60); 3.075 / 0.200 / A3 (57); 3.450 / 0.700 / D3 (50).

Listen for: LO square motion and modulation noise roughen the pitch and cutoff together. The repeated D motif lets you hear the random variation against a stable gesture. The mixer noise remains off; compare MOD MIX 0/5/10 on separate takes.

Hardware: Select OSC 3 and NOISE/MOD SRC, MOD MIX 5, MOD DEPTH 10, CC 1 = 13. Use the built-in noise source with no cable in MOD SOURCE; match depth, then compare distributions rather than individual noise samples.

[Preset](presets/MUS%2022%20Broken%20Square.toml) · [MIDI](midi/MUS%2022%20Broken%20Square.mid) · [Listen](audio/listening/MUS%2022%20Broken%20Square.wav) · [Raw](audio/raw/MUS%2022%20Broken%20Square.wav) · [Panel](panels/MUS%2022%20Broken%20Square.png)

### 23. External Rhythm

A rhythmic vowel-like phrase made by gating and filtering the external source.

- Duration: 4.8 s including release. Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 8, CONTOUR 6; tracking K1/K2 1/1.
- ADS (attack / decay / sustain): filter 0 / 3 / 2; loudness 0 / 2 / 6. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 0.
- EXTERNAL INPUT ON, level 6.
- Notes: 0.200 / 0.250 / F3 (53); 0.575 / 0.125 / F3 (53); 0.825 / 0.125 / C4 (60); 1.075 / 0.250 / F4 (65); 1.450 / 0.125 / C4 (60); 1.825 / 0.250 / F3 (53); 2.200 / 0.250 / F4 (65); 2.575 / 0.125 / D#4 (63); 2.825 / 0.125 / C4 (60); 3.200 / 0.250 / F3 (53); 3.700 / 0.450 / F4 (65).

Listen for: The source stays at 220 Hz; the MIDI keys alter filter tracking and gating rather than the source's pitch. Follow the same external probe on both devices, with all VCO/noise mixer switches off and feedback zero.

Hardware: Connect the supplied mono test signal to EXT IN, which breaks the hardware's automatic output return. The plug-in receives that same file on its side chain. Match input voltage; an interface's dBFS is not an analog voltage.

[Preset](presets/MUS%2023%20External%20Rhythm.toml) · [MIDI](midi/MUS%2023%20External%20Rhythm.mid) · [Listen](audio/listening/MUS%2023%20External%20Rhythm.wav) · [Raw](audio/raw/MUS%2023%20External%20Rhythm.wav) · [Panel](panels/MUS%2023%20External%20Rhythm.png)

### 24. Feedback Stomp

An overdriven F-minor stomp, with a descending chromatic turnaround.

- Duration: 4.8 s including release. Oscillators: 1: 16' saw, level 6; 2: 16' square, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 4, CONTOUR 4; tracking K1/K2 1/0.
- ADS (attack / decay / sustain): filter 0 / 3 / 2; loudness 0 / 2 / 8. DECAY ON.
- Modulation: MIX 0; OSC OFF, FILTER OFF; CC1 0 / 127.
- GLIDE 0 (OFF); VOLUME 6; FEEDBACK 6.
- EXTERNAL INPUT ON, level 6.
- Notes: 0.200 / 0.300 / F3 (53); 0.575 / 0.125 / F3 (53); 0.950 / 0.200 / C4 (60); 1.325 / 0.200 / D#4 (63); 1.700 / 0.300 / F4 (65); 2.200 / 0.300 / F3 (53); 2.700 / 0.125 / C4 (60); 2.950 / 0.200 / B3 (59); 3.325 / 0.125 / A#3 (58); 3.575 / 0.550 / F3 (53).

Listen for: The PHONES return gives the phrase a thicker nonlinear edge, with DRIVE still zero. Short attacks versus the held closing root expose the mixer/preamp's behaviour. Match listening gain on a return-off repeat; a louder take alone is not the distortion test.

Hardware: For the closest topology, patch one PHONES output channel into EXT IN and use the independent PHONES volume; start at its 6 mark and EXT IN 6. CA-72 FEEDBACK 6 is a starting setting, not a promise of equal loop gain. The normalled uncabled return also works but depends on MAIN VOLUME; record which route and measured return level you used.

[Preset](presets/MUS%2024%20Feedback%20Stomp.toml) · [MIDI](midi/MUS%2024%20Feedback%20Stomp.mid) · [Listen](audio/listening/MUS%2024%20Feedback%20Stomp.wav) · [Raw](audio/raw/MUS%2024%20Feedback%20Stomp.wav) · [Panel](panels/MUS%2024%20Feedback%20Stomp.png)

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
