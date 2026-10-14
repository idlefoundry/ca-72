# CA-72: 24 hardware comparison presets

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
- Windows: `%APPDATA%\Idle Foundry\CA-72\Presets\`

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
| 01 | Sub Triangle | 1: 32' triangle, level 6 | OSC 1 alone, triangle, 32': low-frequency shape and tuning. | F2 → A2 → C3 → F3 |
| 02 | Shark Register | 2: 16' shark tooth, level 6, freq +0 st | OSC 2 alone, shark tooth, 16': the asymmetric triangle/saw shape. | F3 → A3 → C4 → F4 |
| 03 | Reverse Current | 3: 8' reverse saw, level 6, freq +0 st | OSC 3 alone, reverse saw, 8', keyboard control on. | C3 → G3 → C4 → G4 |
| 04 | Square Octave | 1: 4' square, level 6 | OSC 1 alone, square, 4': octave-up oscillator tracking. | C4 → E4 → G4 → C5 |
| 05 | Wide Skyline | 2: 2' wide pulse, level 6, freq +0 st | OSC 2 alone, wide pulse, 2': two-octave shift and upper-register harmonics. | C3 → E3 → G3 → C4 |
| 06 | Narrow Needle | 3: 8' narrow pulse, level 6, freq +0 st | OSC 3 alone, narrow pulse, 8': high keyboard register. | F5 → A5 → C6 → A5 |
| 07 | Twin Beat | 1: 8' saw, level 4; 2: 8' saw, level 4, freq +0.1 st | OSC 1 + 2, saws at 8', OSC 2 ten cents sharp. | A3 |
| 08 | Octave Floor | 1: 16' triangle, level 6; 3: 32' square, level 4, freq +0 st | OSC 1 + 3, a 16' triangle over a 32' square. | F2 → A2 → C3 → F3 |
| 09 | Fifth Lantern | 2: 8' triangle, level 5, freq +0 st; 3: 8' saw, level 4, freq +7 st | OSC 2 + 3, 8' triangle and saw; OSC 3 seven semitones higher. | C3 → D3 → G3 → C4 |
| 10 | Three Floors | 1: 32' saw, level 4; 2: 16' shark tooth, level 4, freq +0 st; 3: 8' square, level 4, freq +0 st | OSC 1 + 2 + 3 at 32'/16'/8': three octave layers, three shapes. | F2 → A2 → C3 → F3 |
| 11 | Copper Pluck | 1: 16' saw, level 5; 2: 16' saw, level 5, freq +0 st | Low-pass filter contour and zero loudness sustain: a short two-saw bass. | F2 → C3 → D3 → F3 → F2 → C3 → D3 → F3 |
| 12 | Slow Bloom | 1: 8' triangle, level 6; 2: 8' saw, level 4, freq +0 st | Independent filter/loudness attack, nonzero sustain and released decay. | C4 |
| 13 | White Lotus | 1: 8' narrow pulse, level 10 | The unchanged factory White Lotus: narrow-pulse bass, a third-harmonic filter ring and OSC 3 LO filter modulation. | C3 → C4 |
| 14 | Ladder Whistle | All three OFF | Filter self-oscillation, no VCO in the mixer, full keyboard tracking. | F2 → F3 → F4 → F5 |
| 15 | High Pass Reed | 1: 8' wide pulse, level 5; 3: 4' reverse saw, level 4, freq +0 st | High-pass filter, resonance, contour, two-thirds keyboard tracking. | F2 → F3 → F4 → F5 |
| 16 | Joined Glide | 1: 8' saw, level 5; 2: 16' wide pulse, level 4, freq +0 st | GLIDE with a continuous monophonic phrase and single triggering. | C3 → E3 → G3 → C4 → G3 → C3 |
| 17 | White Snap | All three OFF | White noise through fast contours: percussion without oscillators. | C3 → C3 → C3 → C3 |
| 18 | Pink Weather | All three OFF | Pink noise, slow filter/loudness attack and sustained resonant wind. | C3 |
| 19 | Triangle Motion | 1: 8' square, level 6 | OSC 3 as an untracked LO triangle LFO, pitch destination only. | C4 |
| 20 | Falling Window | 1: 16' saw, level 6 | OSC 3 as an untracked LO reverse-saw LFO, filter destination only. | C3 |
| 21 | Audio Alloy | 1: 8' triangle, level 6 | Keyboard-tracked OSC 3 at audio rate modulates oscillator pitch. | C4 → E4 → G4 → C5 |
| 22 | Mixed Turbulence | 1: 8' wide pulse, level 6 | Midpoint MOD MIX: LO square OSC 3 plus noise, both destinations. | C3 |
| 23 | External Window | All three OFF | External audio through preamp, low-pass filter and keyboard-gated contours. | F2 → F3 → F4 → F5 |
| 24 | Return Furnace | 1: 16' saw, level 6; 2: 16' square, level 4, freq +0 st | External-input feedback and mixer overload from the hardware-accessible loop. | F2 → A2 → C3 → F3 |

## Exact settings and timing

All unspecified controls use the common setup above. Each timing is **start / held length / key** in seconds. The last column of the table is the input key, not always the sounding octave.

### 01. Sub Triangle

OSC 1 alone, triangle, 32': low-frequency shape and tuning.

- Oscillators: 1: 32' triangle, level 6. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / F2 (41); 1.20 / 0.65 / A2 (45); 2.20 / 0.65 / C3 (48); 3.20 / 0.65 / F3 (53).

Listen for: Fundamental and faint upper harmonics; the first played F2 sounds at F0 (21.83 Hz). Check the next three notes too, not just the subsonic end.

[Preset](presets/CMP%2001%20Sub%20Triangle.toml) · [MIDI](midi/CMP%2001%20Sub%20Triangle.mid) · [Listen](audio/listening/CMP%2001%20Sub%20Triangle.wav) · [Raw](audio/raw/CMP%2001%20Sub%20Triangle.wav) · [Panel](panels/CMP%2001%20Sub%20Triangle.png)

### 02. Shark Register

OSC 2 alone, shark tooth, 16': the asymmetric triangle/saw shape.

- Oscillators: 2: 16' shark tooth, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / F3 (53); 1.20 / 0.65 / A3 (57); 2.20 / 0.65 / C4 (60); 3.20 / 0.65 / F4 (65).

Listen for: The character between a triangle and a saw, with OSC 1 and OSC 3 muted.

[Preset](presets/CMP%2002%20Shark%20Register.toml) · [MIDI](midi/CMP%2002%20Shark%20Register.mid) · [Listen](audio/listening/CMP%2002%20Shark%20Register.wav) · [Raw](audio/raw/CMP%2002%20Shark%20Register.wav) · [Panel](panels/CMP%2002%20Shark%20Register.png)

### 03. Reverse Current

OSC 3 alone, reverse saw, 8', keyboard control on.

- Oscillators: 3: 8' reverse saw, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / C3 (48); 1.20 / 0.65 / G3 (55); 2.20 / 0.65 / C4 (60); 3.20 / 0.65 / G4 (67).

Listen for: The reverse saw's spectrum and transient shape. A reverse saw can sound much like an ordinary saw; inspect polarity and waveform as well as listening.

[Preset](presets/CMP%2003%20Reverse%20Current.toml) · [MIDI](midi/CMP%2003%20Reverse%20Current.mid) · [Listen](audio/listening/CMP%2003%20Reverse%20Current.wav) · [Raw](audio/raw/CMP%2003%20Reverse%20Current.wav) · [Panel](panels/CMP%2003%20Reverse%20Current.png)

### 04. Square Octave

OSC 1 alone, square, 4': octave-up oscillator tracking.

- Oscillators: 1: 4' square, level 6. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / C4 (60); 1.20 / 0.65 / E4 (64); 2.20 / 0.65 / G4 (67); 3.20 / 0.65 / C5 (72).

Listen for: Hollow odd-harmonic tone; input C4 is sounding C5, and the final input C5 sounds C6.

[Preset](presets/CMP%2004%20Square%20Octave.toml) · [MIDI](midi/CMP%2004%20Square%20Octave.mid) · [Listen](audio/listening/CMP%2004%20Square%20Octave.wav) · [Raw](audio/raw/CMP%2004%20Square%20Octave.wav) · [Panel](panels/CMP%2004%20Square%20Octave.png)

### 05. Wide Skyline

OSC 2 alone, wide pulse, 2': two-octave shift and upper-register harmonics.

- Oscillators: 2: 2' wide pulse, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / C3 (48); 1.20 / 0.65 / E3 (52); 2.20 / 0.65 / G3 (55); 3.20 / 0.65 / C4 (60).

Listen for: Compare with 04: the same sounding C5/E5/G5/C6 pitches, but a different pulse width and oscillator.

[Preset](presets/CMP%2005%20Wide%20Skyline.toml) · [MIDI](midi/CMP%2005%20Wide%20Skyline.mid) · [Listen](audio/listening/CMP%2005%20Wide%20Skyline.wav) · [Raw](audio/raw/CMP%2005%20Wide%20Skyline.wav) · [Panel](panels/CMP%2005%20Wide%20Skyline.png)

### 06. Narrow Needle

OSC 3 alone, narrow pulse, 8': high keyboard register.

- Oscillators: 3: 8' narrow pulse, level 6, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / F5 (77); 1.20 / 0.65 / A5 (81); 2.20 / 0.65 / C6 (84); 3.20 / 0.65 / A5 (81).

Listen for: Thin nasal harmonics on F5/A5/C6; pitch and aliasing at the upper end of the original keyboard.

[Preset](presets/CMP%2006%20Narrow%20Needle.toml) · [MIDI](midi/CMP%2006%20Narrow%20Needle.mid) · [Listen](audio/listening/CMP%2006%20Narrow%20Needle.wav) · [Raw](audio/raw/CMP%2006%20Narrow%20Needle.wav) · [Panel](panels/CMP%2006%20Narrow%20Needle.png)

### 07. Twin Beat

OSC 1 + 2, saws at 8', OSC 2 ten cents sharp.

- Oscillators: 1: 8' saw, level 4; 2: 8' saw, level 4, freq +0.1 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +4, EMPHASIS 0, AMOUNT 0; K1/K2 1/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 3.60 / A3 (57).

Listen for: Slow beating on held A3. Ten cents is deliberately between lines; use the displayed cents/tuner, rather than an approximate knob angle.

Hardware: The stored +10 cents is relative to OSC 2's nominal centre. Match the measured beat rate against OSC 1, including each device's natural centre mismatch and drift.

[Preset](presets/CMP%2007%20Twin%20Beat.toml) · [MIDI](midi/CMP%2007%20Twin%20Beat.mid) · [Listen](audio/listening/CMP%2007%20Twin%20Beat.wav) · [Raw](audio/raw/CMP%2007%20Twin%20Beat.wav) · [Panel](panels/CMP%2007%20Twin%20Beat.png)

### 08. Octave Floor

OSC 1 + 3, a 16' triangle over a 32' square.

- Oscillators: 1: 16' triangle, level 6; 3: 32' square, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -1, EMPHASIS 0, AMOUNT 0; K1/K2 0/1.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / F2 (41); 1.20 / 0.65 / A2 (45); 2.20 / 0.65 / C3 (48); 3.20 / 0.65 / F3 (53).

Listen for: The square sub-octave and softer upper fundamental; check both voices still track over the four keys.

[Preset](presets/CMP%2008%20Octave%20Floor.toml) · [MIDI](midi/CMP%2008%20Octave%20Floor.mid) · [Listen](audio/listening/CMP%2008%20Octave%20Floor.wav) · [Raw](audio/raw/CMP%2008%20Octave%20Floor.wav) · [Panel](panels/CMP%2008%20Octave%20Floor.png)

### 09. Fifth Lantern

OSC 2 + 3, 8' triangle and saw; OSC 3 seven semitones higher.

- Oscillators: 2: 8' triangle, level 5, freq +0 st; 3: 8' saw, level 4, freq +7 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +2, EMPHASIS 0, AMOUNT 0; K1/K2 1/1.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / C3 (48); 1.20 / 0.65 / D3 (50); 2.20 / 0.65 / G3 (55); 3.20 / 0.65 / C4 (60).

Listen for: A fifth inside one synth voice. One MIDI key at a time; this is oscillator interval tuning, not polyphony.

Hardware: The hardware's +7 marking is only a guide. Tune OSC 3 to exactly +7 semitones relative to OSC 2, then record the actual knob position.

[Preset](presets/CMP%2009%20Fifth%20Lantern.toml) · [MIDI](midi/CMP%2009%20Fifth%20Lantern.mid) · [Listen](audio/listening/CMP%2009%20Fifth%20Lantern.wav) · [Raw](audio/raw/CMP%2009%20Fifth%20Lantern.wav) · [Panel](panels/CMP%2009%20Fifth%20Lantern.png)

### 10. Three Floors

OSC 1 + 2 + 3 at 32'/16'/8': three octave layers, three shapes.

- Oscillators: 1: 32' saw, level 4; 2: 16' shark tooth, level 4, freq +0 st; 3: 8' square, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +0, EMPHASIS 0, AMOUNT 0; K1/K2 1/1.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / F2 (41); 1.20 / 0.65 / A2 (45); 2.20 / 0.65 / C3 (48); 3.20 / 0.65 / F3 (53).

Listen for: Low saw, middle shark tooth and high square stay in octave relationships; listen for mixer colour as all three sum.

[Preset](presets/CMP%2010%20Three%20Floors.toml) · [MIDI](midi/CMP%2010%20Three%20Floors.mid) · [Listen](audio/listening/CMP%2010%20Three%20Floors.wav) · [Raw](audio/raw/CMP%2010%20Three%20Floors.wav) · [Panel](panels/CMP%2010%20Three%20Floors.png)

### 11. Copper Pluck

Low-pass filter contour and zero loudness sustain: a short two-saw bass.

- Oscillators: 1: 16' saw, level 5; 2: 16' saw, level 5, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -3, EMPHASIS 6, AMOUNT 6; K1/K2 1/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 0; DECAY ON.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.18 / F2 (41); 0.70 / 0.18 / C3 (48); 1.20 / 0.18 / D3 (50); 1.70 / 0.18 / F3 (53); 2.20 / 0.18 / F2 (41); 2.70 / 0.18 / C3 (48); 3.20 / 0.18 / D3 (50); 3.70 / 0.18 / F3 (53).

Listen for: Filter opens at attack then closes; loudness falls to zero even if a key were held. The gaps expose retrigger and residual tails.

[Preset](presets/CMP%2011%20Copper%20Pluck.toml) · [MIDI](midi/CMP%2011%20Copper%20Pluck.mid) · [Listen](audio/listening/CMP%2011%20Copper%20Pluck.wav) · [Raw](audio/raw/CMP%2011%20Copper%20Pluck.wav) · [Panel](panels/CMP%2011%20Copper%20Pluck.png)

### 12. Slow Bloom

Independent filter/loudness attack, nonzero sustain and released decay.

- Oscillators: 1: 8' triangle, level 6; 2: 8' saw, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -3, EMPHASIS 4, AMOUNT 6; K1/K2 0/1.
- Filter ADS: 6 / 3 / 6; loudness ADS: 4 / 3 / 8; DECAY ON.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 2.60 / C4 (60).

Listen for: Volume rises before the filter has finished opening; after the 2.80 s note-off, listen to both contour tails.

[Preset](presets/CMP%2012%20Slow%20Bloom.toml) · [MIDI](midi/CMP%2012%20Slow%20Bloom.mid) · [Listen](audio/listening/CMP%2012%20Slow%20Bloom.wav) · [Raw](audio/raw/CMP%2012%20Slow%20Bloom.wav) · [Panel](panels/CMP%2012%20Slow%20Bloom.png)

### 13. White Lotus

The unchanged factory White Lotus: narrow-pulse bass, a third-harmonic filter ring and OSC 3 LO filter modulation.

- Oscillators: 1: 8' narrow pulse, level 10. OSC 3 CONTROL OFF.
- Modulator: OSC 3 LO triangle, mixer OFF; original FREQUENCY pot value 7.47627, near the clockwise stop, approximately 11 Hz with CONTROL OFF.
- Filter: LO, CUTOFF -2.14, EMPHASIS 7, AMOUNT 0; K1/K2 1/1.
- Filter ADS: 0 / 3 / 10; loudness ADS: 0.5 / 3.5 / 8.5; DECAY ON.
- Modulation: MIX 0, OSC OFF, FILTER ON, preset MODULATION **0.17**, no CC 1 in the performance clip; clear incoming state during setup.
- Original factory exceptions: MAIN VOLUME 9.78; LO filter; inactive OSC 2/OSC 3/NOISE/EXTERNAL INPUT mixer levels 8, all their switches OFF; VOICES 4; AUTO GAIN ON with DRIVE 0 (no gain correction). All saved factory parameter values match White Lotus exactly.
- Notes: 0.20 / 1.40 / C3 (48); 2.10 / 1.40 / C4 (60).

Listen for: The held narrow-pulse fundamental and its resonant ring an octave and a fifth above. C3 followed by C4 exposes full keyboard tracking and the approximately 11 Hz modulation of the ring. The factory's fine positions are preserved rather than rounded to lines.

Hardware: Set OSC 1 to 8' narrow pulse, mixer 10; all other mixer sources OFF. Use LO filter, cutoff -2.14, emphasis 7, both tracking switches ON, contour amount 0. OSC 3 is a triangle at LO, CONTROL OFF and mixer OFF; tune its rate to approximately 11 Hz. FILTER MODULATION ON, OSCILLATOR MODULATION OFF, MOD MIX 0. Before recording, send CC 1 = 0 to clear incoming MIDI modulation, then load White Lotus on CA-72 and move the hardware MOD DEPTH knob to match its excursion. A MIDI wheel message takes over the hardware depth until that knob moves, so complete resets before setting the hardware knob. The shared clip sends no CC 1; leave the modulation controller untouched during the take, preserving the CA-72 preset's exact 0.17 and the matched hardware knob depth.

[Preset](presets/CMP%2013%20White%20Lotus.toml) · [MIDI](midi/CMP%2013%20White%20Lotus.mid) · [Listen](audio/listening/CMP%2013%20White%20Lotus.wav) · [Raw](audio/raw/CMP%2013%20White%20Lotus.wav) · [Panel](panels/CMP%2013%20White%20Lotus.png)

### 14. Ladder Whistle

Filter self-oscillation, no VCO in the mixer, full keyboard tracking.

- Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 10, AMOUNT 0; K1/K2 1/1.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Noise: WHITE ON, level 1.
- Notes: 0.20 / 0.65 / F2 (41); 1.20 / 0.65 / F3 (53); 2.20 / 0.65 / F4 (65); 3.20 / 0.65 / F5 (77).

Listen for: Quiet noise excites the resonant ladder. Listen for the filter's near-sine tone and octave tracking. These keys control filter pitch; cutoff -2 does not promise concert tuning.

Hardware: Keep NOISE at 1 on both devices to start oscillation reproducibly; do not tune the filter between notes in a take.

[Preset](presets/CMP%2014%20Ladder%20Whistle.toml) · [MIDI](midi/CMP%2014%20Ladder%20Whistle.mid) · [Listen](audio/listening/CMP%2014%20Ladder%20Whistle.wav) · [Raw](audio/raw/CMP%2014%20Ladder%20Whistle.wav) · [Panel](panels/CMP%2014%20Ladder%20Whistle.png)

### 15. High Pass Reed

High-pass filter, resonance, contour, two-thirds keyboard tracking.

- Oscillators: 1: 8' wide pulse, level 5; 3: 4' reverse saw, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: HI, CUTOFF +1, EMPHASIS 6, AMOUNT 3; K1/K2 0/1.
- Filter ADS: 0 / 3 / 2; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Notes: 0.20 / 0.65 / F2 (41); 1.20 / 0.65 / F3 (53); 2.20 / 0.65 / F4 (65); 3.20 / 0.65 / F5 (77).

Listen for: Low fundamentals thin out, with the upper harmonics retained. Repeat the same take in LO to isolate the filter-mode change.

[Preset](presets/CMP%2015%20High%20Pass%20Reed.toml) · [MIDI](midi/CMP%2015%20High%20Pass%20Reed.mid) · [Listen](audio/listening/CMP%2015%20High%20Pass%20Reed.wav) · [Raw](audio/raw/CMP%2015%20High%20Pass%20Reed.wav) · [Panel](panels/CMP%2015%20High%20Pass%20Reed.png)

### 16. Joined Glide

GLIDE with a continuous monophonic phrase and single triggering.

- Oscillators: 1: 8' saw, level 5; 2: 16' wide pulse, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 4, AMOUNT 4; K1/K2 1/1.
- Filter ADS: 0 / 4 / 3; loudness ADS: 0 / 3 / 10; DECAY ON.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- GLIDE ON, amount 6.
- Notes: 0.20 / 0.60 / C3 (48); 0.80 / 0.60 / E3 (52); 1.40 / 0.60 / G3 (55); 2.00 / 0.60 / C4 (60); 2.60 / 0.60 / G3 (55); 3.20 / 0.80 / C3 (48).

Listen for: Joined notes have zero gap, ordered note-off before note-on: never two held keys. Listen to up/down slide time and whether the filter attack restarts. Hardware multi-trigger must be OFF.

Hardware: Set GLIDE to 6. For manual playing, use a single-note legato line; the supplied MIDI uses zero-gap joins. Repeat with GLIDE at 0 for the control take.

[Preset](presets/CMP%2016%20Joined%20Glide.toml) · [MIDI](midi/CMP%2016%20Joined%20Glide.mid) · [Listen](audio/listening/CMP%2016%20Joined%20Glide.wav) · [Raw](audio/raw/CMP%2016%20Joined%20Glide.wav) · [Panel](panels/CMP%2016%20Joined%20Glide.png)

### 17. White Snap

White noise through fast contours: percussion without oscillators.

- Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF +0, EMPHASIS 2, AMOUNT 2; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 0; DECAY ON.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Noise: WHITE ON, level 6.
- Notes: 0.20 / 0.10 / C3 (48); 1.20 / 0.10 / C3 (48); 2.20 / 0.10 / C3 (48); 3.20 / 0.10 / C3 (48).

Listen for: Broadband hiss, transient attack and the decay slope. MIDI C3 triggers the contour; with tracking off it does not pitch the noise.

[Preset](presets/CMP%2017%20White%20Snap.toml) · [MIDI](midi/CMP%2017%20White%20Snap.mid) · [Listen](audio/listening/CMP%2017%20White%20Snap.wav) · [Raw](audio/raw/CMP%2017%20White%20Snap.wav) · [Panel](panels/CMP%2017%20White%20Snap.png)

### 18. Pink Weather

Pink noise, slow filter/loudness attack and sustained resonant wind.

- Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 6, AMOUNT 4; K1/K2 0/0.
- Filter ADS: 6 / 3 / 8; loudness ADS: 4 / 3 / 8; DECAY ON.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- Noise: PINK ON, level 6.
- Notes: 0.20 / 3.00 / C3 (48).

Listen for: Darker noise spectrum than 17, with a moving resonant band. For a controlled noise-colour comparison, repeat 17 changing only WHITE to PINK.

[Preset](presets/CMP%2018%20Pink%20Weather.toml) · [MIDI](midi/CMP%2018%20Pink%20Weather.mid) · [Listen](audio/listening/CMP%2018%20Pink%20Weather.wav) · [Raw](audio/raw/CMP%2018%20Pink%20Weather.wav) · [Panel](panels/CMP%2018%20Pink%20Weather.png)

### 19. Triangle Motion

OSC 3 as an untracked LO triangle LFO, pitch destination only.

- Oscillators: 1: 8' square, level 6. OSC 3 CONTROL OFF.
- Modulator: OSC 3 LO triangle, FREQUENCY 0, mixer OFF.
- Filter: LO, CUTOFF +2, EMPHASIS 0, AMOUNT 0; K1/K2 1/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC ON, FILTER OFF, CC 1 **6 / 127**.
- Notes: 0.20 / 3.60 / C4 (60).

Listen for: Smooth pitch modulation on held C4. OSC 3's mixer switch stays OFF; it remains a modulation source. Repeat an octave higher to check the LFO rate stays fixed.

Hardware: Select OSC 3 as source, NOISE/MOD SRC rather than the separate LFO, MOD MIX fully to OSC 3, MOD DEPTH 10. Use CC 1 = 6 as a reproducible starting depth, then match measured pitch excursion.

[Preset](presets/CMP%2019%20Triangle%20Motion.toml) · [MIDI](midi/CMP%2019%20Triangle%20Motion.mid) · [Listen](audio/listening/CMP%2019%20Triangle%20Motion.wav) · [Raw](audio/raw/CMP%2019%20Triangle%20Motion.wav) · [Panel](panels/CMP%2019%20Triangle%20Motion.png)

### 20. Falling Window

OSC 3 as an untracked LO reverse-saw LFO, filter destination only.

- Oscillators: 1: 16' saw, level 6. OSC 3 CONTROL OFF.
- Modulator: OSC 3 LO reverse saw, FREQUENCY 0, mixer OFF.
- Filter: LO, CUTOFF -1, EMPHASIS 6, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER ON, CC 1 **26 / 127**.
- Notes: 0.20 / 3.60 / C3 (48).

Listen for: A repeated asymmetric cutoff sweep with a sharp reset. Pitch modulation is OFF; listen to filter motion rather than a pitch slide.

Hardware: Select OSC 3 and NOISE/MOD SRC, MOD DEPTH 10, CC 1 = 26. Match rate and cutoff excursion by measurement; the MIDI wheel transfer curve is not guaranteed identical on another hardware unit.

[Preset](presets/CMP%2020%20Falling%20Window.toml) · [MIDI](midi/CMP%2020%20Falling%20Window.mid) · [Listen](audio/listening/CMP%2020%20Falling%20Window.wav) · [Raw](audio/raw/CMP%2020%20Falling%20Window.wav) · [Panel](panels/CMP%2020%20Falling%20Window.png)

### 21. Audio Alloy

Keyboard-tracked OSC 3 at audio rate modulates oscillator pitch.

- Oscillators: 1: 8' triangle, level 6. OSC 3 CONTROL ON.
- Modulator: OSC 3 4' triangle, FREQUENCY 0, mixer OFF.
- Filter: LO, CUTOFF +3, EMPHASIS 0, AMOUNT 0; K1/K2 1/1.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC ON, FILTER OFF, CC 1 **13 / 127**.
- Notes: 0.20 / 0.65 / C4 (60); 1.20 / 0.65 / E4 (64); 2.20 / 0.65 / G4 (67); 3.20 / 0.65 / C5 (72).

Listen for: Metallic sidebands from audio-rate exponential FM. OSC 3 is inaudible in the mixer but audible through modulation. This is not oscillator sync or ring modulation.

Hardware: Select OSC 3/NOISE sources, MOD DEPTH 10 and CC 1 = 13; keep OSC 3 mixer OFF and OSC 3 CONTROL ON. Match modulation depth before comparing sidebands.

[Preset](presets/CMP%2021%20Audio%20Alloy.toml) · [MIDI](midi/CMP%2021%20Audio%20Alloy.mid) · [Listen](audio/listening/CMP%2021%20Audio%20Alloy.wav) · [Raw](audio/raw/CMP%2021%20Audio%20Alloy.wav) · [Panel](panels/CMP%2021%20Audio%20Alloy.png)

### 22. Mixed Turbulence

Midpoint MOD MIX: LO square OSC 3 plus noise, both destinations.

- Oscillators: 1: 8' wide pulse, level 6. OSC 3 CONTROL OFF.
- Modulator: OSC 3 LO square, FREQUENCY 0, mixer OFF.
- Filter: LO, CUTOFF +0, EMPHASIS 4, AMOUNT 0; K1/K2 0/0.
- Filter ADS: 0 / 2 / 0; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 5, OSC ON, FILTER ON, CC 1 **13 / 127**.
- Notes: 0.20 / 3.60 / C3 (48).

Listen for: Stepped motion with random flutter. The mixer noise switch is OFF: noise still reaches the modulation circuit. Repeat at MOD MIX 0 and 10 to separate the two sources.

Hardware: Select OSC 3 and NOISE/MOD SRC, MOD MIX 5, MOD DEPTH 10, CC 1 = 13. Use the built-in noise source with no cable in MOD SOURCE; match depth, then compare distributions rather than individual noise samples.

[Preset](presets/CMP%2022%20Mixed%20Turbulence.toml) · [MIDI](midi/CMP%2022%20Mixed%20Turbulence.mid) · [Listen](audio/listening/CMP%2022%20Mixed%20Turbulence.wav) · [Raw](audio/raw/CMP%2022%20Mixed%20Turbulence.wav) · [Panel](panels/CMP%2022%20Mixed%20Turbulence.png)

### 23. External Window

External audio through preamp, low-pass filter and keyboard-gated contours.

- Oscillators: All three OFF. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 6, AMOUNT 4; K1/K2 1/1.
- Filter ADS: 0 / 3 / 2; loudness ADS: 0 / 2 / 10; DECAY OFF.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- EXTERNAL INPUT ON, level 6; FEEDBACK 0.
- Notes: 0.20 / 0.65 / F2 (41); 1.20 / 0.65 / F3 (53); 2.20 / 0.65 / F4 (65); 3.20 / 0.65 / F5 (77).

Listen for: All oscillator/noise mixer switches OFF, FEEDBACK 0. Play external-input.wav into the hardware EXT IN and the plug-in side chain at the same measured input level. Keys change filtering and gating; the source's pitch remains 220 Hz.

Hardware: Connect the supplied mono test signal to EXT IN, which breaks the hardware's automatic output return. The plug-in receives that same file on its side chain. Match input voltage; an interface's dBFS is not an analog voltage.

[Preset](presets/CMP%2023%20External%20Window.toml) · [MIDI](midi/CMP%2023%20External%20Window.mid) · [Listen](audio/listening/CMP%2023%20External%20Window.wav) · [Raw](audio/raw/CMP%2023%20External%20Window.wav) · [Panel](panels/CMP%2023%20External%20Window.png)

### 24. Return Furnace

External-input feedback and mixer overload from the hardware-accessible loop.

- Oscillators: 1: 16' saw, level 6; 2: 16' square, level 4, freq +0 st. OSC 3 CONTROL ON.
- Filter: LO, CUTOFF -2, EMPHASIS 4, AMOUNT 4; K1/K2 1/0.
- Filter ADS: 0 / 3 / 2; loudness ADS: 0 / 3 / 8; DECAY ON.
- Modulation: MIX 0, OSC OFF, FILTER OFF, CC 1 **0 / 127**.
- EXTERNAL INPUT ON, level 6; FEEDBACK 6.
- Notes: 0.20 / 0.65 / F2 (41); 1.20 / 0.65 / A2 (45); 2.20 / 0.65 / C3 (48); 3.20 / 0.65 / F3 (53).

Listen for: Feedback changes the mixer/preamp distortion, not just listening volume; observe OVERLOAD. Compare repeat takes with the return muted and present at matched listening loudness.

Hardware: For the closest topology, patch one PHONES output channel into EXT IN and use the independent PHONES volume; start at its 6 mark and EXT IN 6. CA-72 FEEDBACK 6 is a starting setting, not a promise of equal loop gain. The normalled uncabled return also works but depends on MAIN VOLUME; record which route and measured return level you used.

[Preset](presets/CMP%2024%20Return%20Furnace.toml) · [MIDI](midi/CMP%2024%20Return%20Furnace.mid) · [Listen](audio/listening/CMP%2024%20Return%20Furnace.wav) · [Raw](audio/raw/CMP%2024%20Return%20Furnace.wav) · [Panel](panels/CMP%2024%20Return%20Furnace.png)

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
