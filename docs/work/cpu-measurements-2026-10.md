# CPU measurement pass, October 2026

Measured on 2026-10-18 on the M4 Pro (14 cores: 10 performance, 4 efficiency; macOS 27.0),
from `main` at `0d478a5` (0.2.5 and its record) plus this branch's measurement knobs
(`d3fc6c0`: `poly_bench`'s `CA72_RATE`, `CA72_QUALITY`, `CA72_DOUBLE`; `ca72-lab worst` and
`crowd --rate`). Measurement only: no DSP or editor change, nothing merged. The owner confirmed
Live and other audio work were closed; every job ran under `nice -n 10`, one heavy job at a
time. Another agent's single-threaded `preset_render` and, for a few minutes, macOS's own
file-provider and Spotlight indexing ran beside some runs; each row carries the load average
it ran under. Scripts, raw logs and `sample` files are in
`~/lab/artifacts/ca-72/cpu-measure-2026-10/` (`results.txt`, `matrix.txt`, `spin.txt`,
`lab.txt`, `*.sample.txt`, `bias-counts.txt`).

QUALITY as in `Engine::panel`: LO is the light model (`light.rs`), HI the circuit's Potato,
ULTRA NoCompromises. "Of a core" is CPU time over wall time: 100 % is one core busy.

## Summary

- **The editor costs the same in every QUALITY and does not stop when hidden.** It adds 6 to
  12 % of a core to the host's main thread in Live and REAPER, about 20 % in GarageBand's
  out-of-process Audio Unit host, plus 0 to 10 % of WindowServer. When the host is in the
  background (Live and GarageBand hide their plug-in windows then; REAPER leaves its window up)
  or the window is covered, it keeps presenting at the same rate (about 30 frames a second
  with four voices playing) and costs as much or more: 12 to 28 % in REAPER and GarageBand, up
  to 16 % in Live.
  baseview's macOS frame timer has no occlusion check, and these numbers say it matters.
- **The partial path is taken on macOS.** In Live and REAPER, every frame of a played run
  after the first was a partial present (`buffer.age() == 1` with damage); whole frames came
  only at each opening, while a tooltip or the preset drawer was up.
- **With nothing changing, the editor is nearly free:** with playback stopped, GarageBand's AU
  host used 1.3 to 1.5 % with the window open.
- **One HI voice costs 63 to 69 % of the process paced, against 18 to 21 % on its own thread**:
  four workers waking for one voice. See items 3 and 4.
- **GarageBand runs HI voices about three times as dear as Live and REAPER** (the same 0.2.5
  code): 4 HI voices cost 132 to 154 % of real-time threads in `AUHostingService`, against about
  48 % more than LO in Live and 57 % in REAPER; `sample` puts the time in `Playing::play`, not spinning. Not explained here.
- **Profiles:** a LO voice is 56 % resampling; a HI voice 48 % the filter; Feedback Funk at
  ULTRA 34 % the preamplifier, 24 % the oscillators, 17 % the filter, with Newton solves 36 %
  and `exp`/`log` 19 % of it.
- **The two bit-identical shortcuts are small:** the OVERLOAD lamp's separate solve is 14 % of
  ULTRA's preamp on Feedback Funk (4.7 % of the voice); `VcfCircuit::bias` runs 1.7 to 2 times
  a sample per ULTRA voice and recomputes `ladder.at(celsius)` every time with an unchanged
  temperature, but `Bjt::at` is at most 0.5 % of the voice (the whole of `bias` 4.1 %).
- **Paced (item 3):** LO is real time everywhere (1 to 10 voices, 44.1 to 96 kHz, 128 and 256
  frames; 9 to 48 % of a core). HI is real time with at most one late block in 20 s (10 voices);
  ten HI voices take 157 to 174 % of the process at 48 kHz. Feedback Funk at ULTRA is not real
  time on this Mac even as one voice (11 to 26 % of blocks late), and the lab's worst case
  misses 10 to 30 blocks in 20 s at 48 kHz with its one ULTRA voice.
- **Worker spin (item 4):** `pool::SPIN` at 50 µs instead of 200 took 12 to 28 points of a
  core (median 19) off the process for 1 to 10 HI voices (one voice: 63 to 68 % down to 43 to
  48 %), with the same late blocks (0 to 3 in 7,500 either way) and the same worst blocks. Measured, not
  changed.

## 1. The editor in real hosts

**Setup.** A renamed build of this tree, **CA-72 MEASURE** (its own name, CLAP id
`com.idlefoundry.ca-72-measure`, VST3 class `IdleFoundryC72M!`, AU `aumu C72M IdlF`; the
`bundle` profile, fat LTO, arm64), with one addition: `Surface::show` counted its presents by
path (partial: damage and `age() == 1`; damage but another age; whole with age 1; whole
otherwise) and appended the counts to `$TMPDIR/ca72-measure.log` every 2 s
(`measure-build.patch` in the artifacts folder). Installed in `~/Library/Audio/Plug-Ins` for
the runs, in the Trash afterwards. Live and REAPER loaded it; GarageBand loaded the installed
0.2.5 instead (below). Four POLY voices (MODE POLY, VOICES 4, the default patch), a looped clip
of four-note chords, a new chord every 2 s held 1.75 s (`ca72-chords-4.mid`). Live and REAPER
ran at 48 kHz with 256-frame buffers into "Virtual Desktop Speakers" (a silent virtual device:
the MOTU was not connected); GarageBand into the default output (48 kHz) with its master volume
at minimum.

**Method.** `cpu.py`: each process's CPU over 20 s after 3 s settling, from `ps -M`'s per-thread
user and system times, and of it the main thread (where baseview draws, in Live, REAPER and
`AUHostingService`) and the threads at real-time priority. The editor's cost is the main
thread open against closed, measured alternately; whole-process differences are given too but
carry the audio's noise (ULTRA's cost moves with the chord). WindowServer's whole-process CPU
moved 33 to 57 % with other apps' drawing; differences under about 5 % are noise.

**States.** *Front*: the host active, the plug-in window visible. *Covered*: a Finder window
over all of the plug-in window but a 7-pt strip, Finder active. *Background*: another app
active, nothing over the plug-in window. Live and GarageBand order their plug-in windows out
when another app is active (on screen: no), so in them covered and background are the same
state: the editor alive, its window hidden. REAPER leaves its FX window on screen.

Main thread, % of a core (closed → open; the editor's share in brackets):

| Host | QUALITY | Closed | Open, front | Open, covered | Open, background |
|---|---|---|---|---|---|
| Live 12.4 beta (VST3) | LO | 16.8 | 23.2 (+6.4) | = background | 22.6 (+5.7), hidden |
| | HI | 16.5–16.9 | 23.0 (+6.1) | = background | 24.3 (+7.8), hidden |
| | ULTRA | 18.1–18.9 | 28.8 (+10.7) | = background | 35.2 (+16.3), hidden |
| REAPER 7.82 (VST3) | LO | 5.8–6.2 | 16.8 (+11.0) | 25.0 (+18.8) | 28.1 (+21.9) |
| | HI | 5.9–7.5 | 17.9 (+12.0) | 27.5 (+20.0) | 26.5 (+19.0) |
| | ULTRA | 3.0–3.7 | 12.2 (+9.2) | 15.7 (+12.0) | 15.7 (+12.0) |
| GarageBand 10.4.14 (AU, `AUHostingService`) | LO | 0.4–0.5 | 18.5–19.8 (+19) | = background | 28.2–28.5 (+28), hidden |
| | HI | 0.4 | 16.9 (+16.5) | = background | 22.9–23.1 (+22.6), hidden |

Whole processes, % of a core (closed → open, front):

| Host | QUALITY | Host process | WindowServer |
|---|---|---|---|
| Live | LO | 64.2 → 69.5 | 45.0 → 33.5 (noise) |
| | HI | 112.4 → 117.3 | 45.7 → 44.3 |
| | ULTRA | 204.2 → 213.8 | 42.8 → 41.7 |
| REAPER | LO | 35.9 → 35.9 (load 25 during the open run) | 42.6 → 44.8 |
| | HI | 97.8 → 99.8 | 39.3 → 49.3 |
| | ULTRA | 174.5 → 190.2 | 43.4 → 45.0 |
| GarageBand + AU host | LO | 21.4 → 38.1 | 45.9 → 51.8 |
| | HI | 168.5 → 161.5 (the HI voices' noise) | 47.5 → 50.4 |

**Presents.** In Live, 600 to 608 presents in each 20 s run with the editor open, front or
hidden (about 30 a second; the timer fires every 15 ms but the editor shows a frame only when
something changed); none with it closed. Of Live's 22,611 presents, 22,375 took the partial
path; 230 were whole frames with age 1 (damage withheld while a tooltip floated over the
panel: hovering QUALITY), 6 whole frames at age 0 (each opening of the window). REAPER: 14,417
presents, 14,402 partial, 12 whole while the preset drawer was open (a stray drag opened it;
that run is marked invalid in `results.txt`), 3 at openings. So on macOS the partial path is
the normal one, in both hosts, with the window on screen or hidden. GarageBand ran the
installed build and was not counted.

**Hidden is not idle.** With Live or GarageBand in the background the plug-in window is
ordered out, and the editor kept presenting 608 frames in 20 s, as in front. In REAPER the same
with the window covered. And the hidden editor cost *more* main-thread time than the visible
one in six of the eight cases (Live ULTRA +16 against +11; REAPER +12 to +22 against +9 to
+12; GarageBand +23 to +28 against +17 to +19); in Live at LO and HI they were about equal.
A guess, not measured: a background app's main thread runs at a lower QoS on the efficiency cores, so the same frames take more CPU time.
REAPER's own threads cost more in the background too, with the editor closed (HI: 113.8
against 97.8 %).

**GarageBand.** The AU runs in `AUHostingService`; its view lives in an off-screen window of
that process (on screen: no, always) and GarageBand shows it through a floating window of its
own. The editor there costs about twice what it costs in Live (main thread +19 against +6 at
LO), and WindowServer about 6 % more. GarageBand reopened its most recent project
(`CA72-pitchbend-MIDI`, closed without saving) even with `-ApplePersistenceIgnoreState YES`,
which loaded the installed CA-72 into `AUHostingService`; the service kept the library mapped
after the project closed and was still running 2 s after a TERM. To keep one build per process
GarageBand then ran the installed 0.2.5 (chosen from its Recent list; a new `AUHostingService`
loaded it from `/Library/Audio/Plug-Ins/Components`, checked with `lsof`). ULTRA was
not measured there: clicks on QUALITY's ULTRA legend did not take in its hosted view (HI's
did), and the mouse wheel and arrow keys did not reach the hosted view at all.

**HI voices in GarageBand.** With the editor closed, 4 HI voices took 144.5 to 154.2 % of the
AU host's real-time threads (LO: 5.9 to 6.5 %), where Live's real-time threads went from 39 to
88 % (LO to HI) and REAPER's 1.6 to 59 %. A 3 s `sample` (`gb-hi-auhost.sample.txt`) puts each
of the four `ca72-voices` workers at about 44 % of the samples inside `Playing::play` and 6 %
spinning; the audio thread `AUOOPRenderingServer` played in its turn. So the voices themselves
took about three times the CPU time they take in Live and REAPER. Not explained by this pass: the render
threads of an out-of-process AU may run on the efficiency cores, or at a different clock. Worth
its own look, with Logic (which hosts AUs the same way) when it is on the Mac.

**Not tried:** Logic (not installed; GarageBand stood in with the owner's yes), Live's and
REAPER's CLAP, Bitwig, a window covered while its host stays active (Live and GarageBand keep
their plug-in windows above their own), an Intel Mac, Windows and Linux.

**Commands.** `state.sh LABEL PID...` (3 s settle, `cpu.py 20`, the present counts before and
after, the window's on-screen flag), run from a shell while the host played; windows opened,
covered and closed by hand through the screen (Live: the device's wrench; REAPER: a
`__startup.lua` reading commands from a file, `TrackFX_Show`; GarageBand: the plug-in slot and
the window's close button). Baseline for comparison: `the_window_played` gave 9.5 % of a core
on the places path and 13.7 % with whole frames on this Mac earlier the same day.

## 2. Sampling profiles

`/usr/bin/sample PID 20 1` of `preset_cost` (release, no `profile` feature, one thread, not
paced), 20 s from 3 s in, one voice of Feedback Funk in each QUALITY:

```
prof.sh lo-1voice-feedback-funk CA72_POTATO=1 CA72_VOICES=1 CA72_ONLY="Feedback Funk" CA72_SECONDS=1800
prof.sh hi-1voice-feedback-funk CA72_VOICES=1 CA72_ONLY="Feedback Funk" CA72_SECONDS=300
prof.sh ultra-1voice-feedback-funk CA72_ULTRA=1 CA72_VOICES=1 CA72_ONLY="Feedback Funk" CA72_SECONDS=40
```

(`prof.sh` runs `preset_cost`'s ignored test with those variables at nice 10 and samples it;
`top.py` and `incl.py` read the result: the self time of the top frames, and the inclusive
time of named frames, as shares of the busy samples.) The voice's cost alone in each:
LO 2.9 %, HI 22.3 %, ULTRA 194.3 % of a core (with FEEDBACK; one voice).

| LO (light model) | self | | HI (circuit, Potato) | self | | ULTRA (NoCompromises) | self |
|---|---|---|---|---|---|---|---|
| `Light::tick` | 36.4 % | | `Vcf::tick` | 27.9 % | | `mna::Circuit::newton` | 17.3 % |
| `Interpolator::push` | 26.1 % | | `Interpolator::push` | 7.0 % | | libm `exp` | 12.7 % |
| `Halfband::sum` | 21.7 % | | `vcf::plain_pair` | 5.0 % | | `ExpoCircuit::solve_current` | 7.3 % |
| `Halfband::push` | 4.3 % | | `BjtAt::currents_d` | 4.6 % | | libm `log` | 6.6 % |
| `Character::offsets` | 3.6 % | | `VcfCircuit::eval_sparse` | 4.2 % | | `Circuit::stamp_at` | 6.5 % |
| `Decimator::push` | 2.6 % | | `Vco::tick_current` | 3.7 % | | `Vcf::tick` | 6.2 % |
| `Playing::play_from` | 2.0 % | | `BjtAt::base_law` | 2.9 % | | `Circuit::load` | 5.7 % |
| `Light::control` | 1.1 % | | `linear::Discrete::step` | 2.7 % | | `BjtAt::base_current` | 5.1 % |

Inclusive: **LO**, resampling (`resample::*`) 55.9 % of the voice, `Light::tick` 92 %.
**HI**, the filter (`Vcf::tick`) 48 %, the contours 11 %, `RevSaw` 9 %, the VCA 8.7 %, the
oscillators (`Vco`) 6.4 %, the keyboard 3.9 %, the preamplifier 2.3 %. **ULTRA**, the
preamplifier 33.9 %, the oscillators 24.2 %, the filter 17.4 %, the contours 9.1 %, the VCA
6.0 %; Newton solves 35.6 %, `ExpoCircuit` 20.5 %, `exp` 12.6 %, `log` 6.6 %.

## 3. The paced matrix

**`poly_bench`, paced** (`matrix.sh`): the engine as the plug-in plays it, its caller thread
promoted as an audio thread, 4 workers (what the plug-in starts on this Mac, `workers_for(14)`),
each block given the plug-in's deadline, 20 s after 2 s; Feedback Funk (the heaviest preset,
FEEDBACK on), POLY with 1, 4 and 10 voices playing changing chords; 44.1, 48 and 96 kHz; 128 and
256 frames; DOUBLE at 100 % on it at ULTRA and HI. "Late" is a block that took longer than its
period; "process CPU" is the bench's user and system time over its wall time (setup and the
2 s warm-up included), so it counts the workers and their spinning as a host's CPU meter does.

```
CA72_PRESET="Feedback Funk" CA72_WORKERS=4 CA72_SECONDS=20 CA72_QUALITY=lo|hi|ultra \
  CA72_VOICES=1|4|10 CA72_RATE=44100|48000|96000 CA72_BLOCK=128|256 [CA72_DOUBLE=100] \
  nice -n 10 target/release/deps/poly_bench-* --ignored --nocapture poly_chords_in_real_time
```

| QUALITY | DOUBLE | voices | kHz | frames | period ms | mean | p99 | worst | late blocks | process CPU |
|---|---|---|---|---|---|---|---|---|---|---|
| LO | off | 1 | 44.1 | 128 | 2.902 | 0.169 | 0.263 | 0.386 | 0/6890 | 9 % |
| LO | off | 1 | 44.1 | 256 | 5.805 | 0.354 | 0.474 | 0.599 | 0/3445 | 9 % |
| LO | off | 1 | 48 | 128 | 2.667 | 0.160 | 0.239 | 0.288 | 0/7500 | 10 % |
| LO | off | 1 | 48 | 256 | 5.333 | 0.343 | 0.464 | 0.585 | 0/3750 | 10 % |
| LO | off | 1 | 96 | 128 | 1.333 | 0.173 | 0.228 | 0.338 | 0/15000 | 17 % |
| LO | off | 1 | 96 | 256 | 2.667 | 0.340 | 0.448 | 0.583 | 0/7500 | 16 % |
| LO | off | 4 | 44.1 | 128 | 2.902 | 0.352 | 0.513 | 0.580 | 0/6890 | 15 % |
| LO | off | 4 | 44.1 | 256 | 5.805 | 0.723 | 1.008 | 1.123 | 0/3445 | 16 % |
| LO | off | 4 | 48 | 128 | 2.667 | 0.341 | 0.532 | 0.745 | 0/7500 | 16 % |
| LO | off | 4 | 48 | 256 | 5.333 | 0.683 | 1.002 | 1.451 | 0/3750 | 16 % |
| LO | off | 4 | 96 | 128 | 1.333 | 0.328 | 0.415 | 0.508 | 0/15000 | 28 % |
| LO | off | 4 | 96 | 256 | 2.667 | 0.637 | 0.822 | 0.961 | 0/7500 | 27 % |
| LO | off | 10 | 44.1 | 128 | 2.902 | 0.803 | 1.187 | 1.381 | 0/6890 | 30 % |
| LO | off | 10 | 44.1 | 256 | 5.805 | 1.520 | 2.255 | 2.706 | 0/3445 | 29 % |
| LO | off | 10 | 48 | 128 | 2.667 | 0.807 | 1.212 | 1.449 | 0/7500 | 33 % |
| LO | off | 10 | 48 | 256 | 5.333 | 1.510 | 2.105 | 2.807 | 0/3750 | 31 % |
| LO | off | 10 | 96 | 128 | 1.333 | 0.610 | 0.766 | 0.862 | 0/15000 | 48 % |
| LO | off | 10 | 96 | 256 | 2.667 | 1.190 | 1.498 | 1.705 | 0/7500 | 47 % |
| HI | off | 1 | 44.1 | 128 | 2.902 | 0.513 | 0.918 | 1.506 | 0/6890 | 63 % |
| HI | off | 1 | 44.1 | 256 | 5.805 | 1.155 | 1.932 | 2.617 | 0/3445 | 65 % |
| HI | off | 1 | 48 | 128 | 2.667 | 0.503 | 0.849 | 1.524 | 0/7500 | 68 % |
| HI | off | 1 | 48 | 256 | 5.333 | 1.136 | 1.785 | 2.230 | 0/3750 | 69 % |
| HI | off | 1 | 96 | 128 | 1.333 | 0.443 | 0.760 | 1.067 | 0/15000 | 124 % |
| HI | off | 1 | 96 | 256 | 2.667 | 0.945 | 1.469 | 2.172 | 0/7500 | 121 % |
| HI | off | 4 | 44.1 | 128 | 2.902 | 0.538 | 0.919 | 1.598 | 0/6890 | 100 % |
| HI | off | 4 | 44.1 | 256 | 5.805 | 1.291 | 2.154 | 2.615 | 0/3445 | 105 % |
| HI | off | 4 | 48 | 128 | 2.667 | 0.523 | 0.859 | 1.503 | 0/7500 | 107 % |
| HI | off | 4 | 48 | 256 | 5.333 | 1.154 | 1.893 | 2.632 | 0/3750 | 104 % |
| HI | off | 4 | 96 | 128 | 1.333 | 0.447 | 0.796 | 1.241 | 0/15000 | 184 % |
| HI | off | 4 | 96 | 256 | 2.667 | 0.912 | 1.432 | 1.984 | 0/7500 | 168 % |
| HI | off | 10 | 44.1 | 128 | 2.902 | 0.818 | 1.400 | 3.240 | 1/6890 (0 %) | 161 % |
| HI | off | 10 | 44.1 | 256 | 5.805 | 1.747 | 2.808 | 3.458 | 0/3445 | 159 % |
| HI | off | 10 | 48 | 128 | 2.667 | 0.814 | 1.437 | 3.258 | 1/7500 (0 %) | 174 % |
| HI | off | 10 | 48 | 256 | 5.333 | 1.564 | 2.560 | 3.511 | 0/3750 | 157 % |
| HI | off | 10 | 96 | 128 | 1.333 | 0.680 | 0.978 | 1.326 | 0/15000 | 300 % |
| HI | off | 10 | 96 | 256 | 2.667 | 1.348 | 1.932 | 7.767 | 1/7500 (0 %) | 272 % |
| ULTRA | off | 1 | 44.1 | 128 | 2.902 | 2.623 | 4.269 | 5.372 | 793/6890 (12 %) | 174 % |
| ULTRA | off | 1 | 44.1 | 256 | 5.805 | 5.485 | 9.051 | 11.973 | 772/3445 (22 %) | 164 % |
| ULTRA | off | 1 | 48 | 128 | 2.667 | 2.624 | 4.218 | 5.387 | 1295/7500 (17 %) | 177 % |
| ULTRA | off | 1 | 48 | 256 | 5.333 | 5.419 | 10.551 | 11.789 | 979/3750 (26 %) | 170 % |
| ULTRA | off | 1 | 96 | 128 | 1.333 | 2.565 | 4.031 | 5.097 | 14988/15000 (100 %) | 193 % |
| ULTRA | off | 1 | 96 | 256 | 2.667 | 5.683 | 10.448 | 11.544 | 7490/7500 (100 %) | 142 % |
| ULTRA | off | 4 | 44.1 | 128 | 2.902 | 2.884 | 5.197 | 9.108 | 1819/6890 (26 %) | 322 % |
| ULTRA | off | 4 | 44.1 | 256 | 5.805 | 6.099 | 17.903 | 26.518 | 1072/3445 (31 %) | 294 % |
| ULTRA | off | 4 | 48 | 128 | 2.667 | 2.865 | 5.035 | 5.974 | 3224/7500 (43 %) | 292 % |
| ULTRA | off | 4 | 48 | 256 | 5.333 | 6.042 | 17.157 | 26.357 | 1900/3750 (51 %) | 266 % |
| ULTRA | off | 4 | 96 | 128 | 1.333 | 2.755 | 4.721 | 5.888 | 14988/15000 (100 %) | 360 % |
| ULTRA | off | 4 | 96 | 256 | 2.667 | 5.556 | 17.768 | 23.663 | 7500/7500 (100 %) | 238 % |
| ULTRA | off | 10 | 44.1 | 128 | 2.902 | 5.968 | 11.059 | 31.274 | 6887/6890 (100 %) | 425 % |
| ULTRA | off | 10 | 44.1 | 256 | 5.805 | 26.015 | 52.507 | 74.128 | 3445/3445 (100 %) | 181 % |
| ULTRA | off | 10 | 48 | 128 | 2.667 | 5.919 | 11.175 | 32.315 | 7490/7500 (100 %) | 397 % |
| ULTRA | off | 10 | 48 | 256 | 5.333 | 24.370 | 51.226 | 62.069 | 3750/3750 (100 %) | 192 % |
| ULTRA | off | 10 | 96 | 128 | 1.333 | 5.612 | 9.168 | 11.617 | 15000/15000 (100 %) | 455 % |
| ULTRA | off | 10 | 96 | 256 | 2.667 | 22.959 | 47.863 | 59.163 | 7500/7500 (100 %) | 196 % |
| ULTRA | on | 1 | 48 | 128 | 2.667 | 5.267 | 8.917 | 11.027 | 7496/7500 (100 %) | 180 % |
| ULTRA | on | 1 | 48 | 256 | 5.333 | 12.192 | 21.283 | 24.766 | 3748/3750 (100 %) | 136 % |
| ULTRA | on | 4 | 48 | 128 | 2.667 | 5.747 | 10.260 | 21.370 | 7493/7500 (100 %) | 317 % |
| ULTRA | on | 4 | 48 | 256 | 5.333 | 11.894 | 41.166 | 48.633 | 3747/3750 (100 %) | 233 % |
| ULTRA | on | 10 | 48 | 128 | 2.667 | 11.863 | 22.400 | 58.533 | 7500/7500 (100 %) | 423 % |
| ULTRA | on | 10 | 48 | 256 | 5.333 | 48.127 | 109.732 | 155.482 | 3750/3750 (100 %) | 200 % |
| HI | on | 4 | 48 | 128 | 2.667 | 0.979 | 1.604 | 1.931 | 0/7500 | 172 % |
| HI | on | 4 | 48 | 256 | 5.333 | 1.822 | 2.919 | 3.893 | 0/3750 | 152 % |
| HI | on | 10 | 48 | 128 | 2.667 | 1.438 | 2.169 | 2.577 | 0/7500 | 287 % |
| HI | on | 10 | 48 | 256 | 5.333 | 2.827 | 4.245 | 4.761 | 0/3750 | 271 % |

Milliseconds are per block. **LO** is real time everywhere: no late block in 18 runs, the worst
block at most 2.8 ms of 5.3 (10 voices, 48 kHz, 256), 9 to 48 % of a core; it does not use the
workers (process CPU about the caller thread's). **HI** is real time with a margin: at most one
late block in a run (10 voices at 128 frames, and once at 96 kHz/256), the worst block 3.5 ms of
5.3; but one voice already takes 63 to 69 % of a core (124 % at 96 kHz) while its caller thread
shows 18 to 21 %: the four workers wake for a single voice (item 4). Four voices take 100 to
107 %, ten 157 to 174 % (272 to 300 % at 96 kHz); DOUBLE on ten 271 to 287 %, still without a
late block. **ULTRA on Feedback Funk is not real time on this Mac even as one voice:** 11 to 26 %
of blocks late at 44.1 and 48 kHz with one voice, 26 to 51 % with four, all of them with ten or
at 96 kHz. One ULTRA Feedback Funk voice costs about two cores (194 % alone, `preset_cost`) and a
voice is not split between workers. **Under overload, 256 frames used fewer cores than 128:**
ten ULTRA voices averaged 24 ms a block at 256 frames on 192 % of a core, against 5.9 ms on 397 %
at 128 (with DOUBLE 48 ms on 200 % against 11.9 ms on 423 %), as if the pool gave up sharing
the voices once the deadline passed. Not followed further here.

**`ca72-lab worst --paced`** (`lab.sh`): one voice with every control moving, fast notes, glide,
the external input; the threaded voice, the threads at audio priority.

| quality | kHz | frames | mean ms | p99.9 | worst (of the block) | over the block | process CPU |
|---|---|---|---|---|---|---|---|
| no-compromises (ULTRA) | 44.1 | 128 | 1.30 | 4.36 | 5.58 (192 %) | 28 of 6891 | 299 % |
| | 44.1 | 256 | 2.58 | 7.95 | 9.69 (167 %) | 10 of 3446 | 273 % |
| | 48 | 128 | 1.28 | 4.36 | 5.58 (209 %) | 30 of 7500 | 324 % |
| | 48 | 256 | 2.54 | 7.79 | 9.05 (170 %) | 15 of 3750 | 294 % |
| | 96 | 128 | 2.31 | 6.71 | 12.11 (908 %) | 13530 of 15000 | 432 % |
| | 96 | 256 | 4.59 | 11.86 | 31.00 (1162 %) | 6733 of 7500 | 430 % |
| potato (HI) | 44.1 | 128 | 0.47 | 0.85 | 0.96 (33 %) | 0 | 52 % |
| | 44.1 | 256 | 0.99 | 1.62 | 1.64 (28 %) | 0 | 37 % |
| | 48 | 128 | 0.45 | 0.78 | 0.86 (32 %) | 0 | 56 % |
| | 48 | 256 | 0.95 | 1.58 | 1.76 (33 %) | 0 | 39 % |
| | 96 | 128 | 0.32 | 0.56 | 0.68 (51 %) | 0 | 97 % |
| | 96 | 256 | 0.71 | 1.12 | 1.28 (48 %) | 0 | 66 % |

**`ca72-lab crowd --paced --pool`**: Potato (HI) voices under the worst case's load, as the
engine's pool on 13 threads.

| voices | kHz | frames | mean ms | worst (of the block) | over the block | process CPU |
|---|---|---|---|---|---|---|
| 100 | 48 | 128 | 3.80 | 5.63 (211 %) | all 7500 | 947 % |
| 100 | 48 | 256 | 7.44 | 8.52 (160 %) | all 3750 | 947 % |
| 100 | 96 | 128 | 3.65 | 4.79 (360 %) | all 15000 | 956 % |
| 100 | 96 | 256 | 7.14 | 8.70 (326 %) | all 7500 | 957 % |
| 30 | 48 | 128 | 1.33 | 2.25 (85 %) | 0 | 539 % |
| 30 | 48 | 256 | 2.46 | 3.63 (68 %) | 0 | 505 % |
| 30 | 96 | 128 | 1.39 | 3.57 (268 %) | 8426 of 15000 | 816 % |
| 30 | 96 | 256 | 2.37 | 19.23 (721 %) | 1119 of 7500 | 818 % |

`ca72-lab {worst|crowd} --paced [--pool] --quality nc|potato --rate R --block N --seconds 20`
(`--rate` is this branch's). Even the worst case's one ULTRA voice misses 10 to 30 blocks in 20 s
at 44.1 and 48 kHz; the Potato voice passes everywhere with a third of the block to spare.

**`preset_cost`** (unpaced, one thread, 10 POLY voices, 48 kHz, 256 frames; `preset-cost.txt`):
LO 13.9 to 17.3 % of a core for all 19 presets (1.4 to 1.7 % a voice); HI 76.7 to 126.5 %
(7.7 to 12.7 % a voice; Feedback Funk and Singing Drift dearest); ULTRA 345 to 1125 % (34.5 to
112 % a voice; Feedback Funk dearest, then Saw Monolith and Singing Drift at 65 %). DOUBLE at
100 % on Feedback Funk: LO 29.3 %, HI 243.7 %, ULTRA 2216 %. One voice of Feedback Funk alone:
LO 2.9 %, HI 22.3 %, ULTRA 194.3 % (item 2), more than a tenth of ten voices in each.
`CA72_SECONDS=4 [CA72_POTATO=1 | CA72_ULTRA=1 with 1 s] [CA72_DOUBLE=100 CA72_ONLY="Feedback Funk"]
cargo test --release -p ca72-plugin --test preset_cost -- --ignored --nocapture`.

## 4. Worker spin: `pool::SPIN` at 200 against 50 µs

`pool::SPIN` (`crates/ca72-plugin/src/pool.rs:60`) is how long an idle worker spins for the next
run before it parks. A build with 50 µs was made into a target folder of its own
(`CARGO_TARGET_DIR=<scratch>/target-spin50 cargo test --release -p ca72-plugin --test poly_bench
--no-run`, the source put back at once), and `spin.sh` ran it and this tree's 200 µs build
alternately, two rounds in opposite orders: `poly_bench` paced as in item 3 (Feedback Funk,
4 workers, 48 kHz, 20 s). Process CPU as in item 3; each cell is round 1 / round 2.

| QUALITY | voices | frames | process CPU, 200 µs | 50 µs | late, 200 µs | late, 50 µs | worst ms, 200 / 50 |
|---|---|---|---|---|---|---|---|
| HI | 1 | 128 | 68 / 66 % | 46 / 47 % | 0 / 0 | 0 / 0 | 1.65, 1.29 / 1.51, 1.08 |
| HI | 1 | 256 | 66 / 63 % | 43 / 48 % | 0 / 0 | 0 / 0 | 3.21, 3.18 / 2.75, 3.07 |
| HI | 4 | 128 | 98 / 98 % | 82 / 70 % | 0 / 0 | 0 / 1 | 1.59, 1.79 / 1.64, 2.68 |
| HI | 4 | 256 | 101 / 84 % | 82 / 72 % | 0 / 0 | 0 / 0 | 3.14, 2.21 / 2.45, 3.44 |
| HI | 10 | 128 | 168 / 171 % | 150 / 159 % | 0 / 2 | 0 / 3 | 2.28, 3.63 / 2.00, 3.93 |
| HI | 10 | 256 | 156 / 157 % | 160 / 143 % | 0 / 0 | 0 / 0 | 3.89, 4.13 / 3.25, 4.61 |
| HI, DOUBLE | 10 | 128 | 286 / 285 % | 262 / 265 % | 0 / 0 | 0 / 0 | 2.56, 2.57 / 2.56, 2.57 |
| ULTRA | 4 | 128 | 296 / 289 % | 285 / 282 % | 2968 / 3224 | 2997 / 2982 | 5.98, 6.04 / 6.02, 6.23 |

(The second round's 50 µs run with four voices at 128 frames ran at load 11, another
process's.) **At 50 µs the process used 12 to 28 points of a core less with one to ten HI
voices** (median 19; about a third less for one voice), except one 10-voice, 256-frame pair
(+4 and -14);
late blocks stayed where they were (0 to 3 in 7,500, as often at 200 µs as at 50) and so did
the worst blocks, within the runs' scatter. ULTRA overloaded: no difference. A reading, not measured: at 48 kHz and 128 frames the
next block is 2.7 ms away, so a worker parks and wakes for every block whether it spins 50 or
200 µs, and the longer spin is mostly spent after the block's last voice; it would help only
for runs that follow within 200 µs, as a block's second `CHUNK` does. This pass measured the
cost only; it did not look at wake-up latency on a slower machine or the Windows and Linux
schedulers, where the spin was chosen (decisions.md R21, R23).

## 5. Sizes for the bit-identical voice shortcuts

**The OVERLOAD lamp's separate solve.** `Preamp::tick` at ULTRA steps the preamplifier's net
four times a sample (`SUBSTEPS`), then `Preamp::outputs` steps the lamp driver's circuit once
(`preamp.rs:537`). Both are inlined into `tick`; the ULTRA profile above separates them by call
site (`dsymutil` of the test binary, then `atos -i` on each call's address): the four net steps
at `preamp.rs:519` took 1,152 to 1,215 samples each (4,690), the lamp step 790, the rest of the
preamp (resampling) 164. **The lamp's solve is 14.0 % of ULTRA's preamplifier on Feedback
Funk, 4.7 % of the voice.**

**`VcfCircuit::bias` and `ladder.at(celsius)`.** A temporary counter (not committed;
`bias-counter.patch`) in `Vcf::drive`, each preset with one voice for 4 s:

| QUALITY | ladder steps a sample | new `(i0, celsius)` a sample | `VcfCircuit::bias` a sample | of them, same temperature |
|---|---|---|---|---|
| ULTRA | 7.5 (8.0 for 4 presets) | 1.70 to 2.00 (0.14 for the three hats) | the same | all |
| HI | 7.5 (8.0) | 1.70 to 2.00 (0.19 for the hats) | 0 (Potato's table) | – |
| LO | 0 (the light model has no ladder) | 0 | 0 | – |

So at ULTRA every second to fourth ladder step works the bias out afresh, and each of those
calls `ladder.at(celsius)` (three `exp` and a `log`) with the temperature it had last time.
What that costs, from the ULTRA profile: `VcfCircuit::bias` 678 samples, 4.1 % of the voice;
`Bjt::at`, from every caller, 85 samples, 0.5 %. **Keeping `ladder.at` while the temperature
stands would save at most 0.5 % of an ULTRA voice; the rest of `bias` (the base currents'
iterations, the junction capacitances) is 3.6 %.**

## Commands and files

All under `~/lab/artifacts/ca-72/cpu-measure-2026-10/`:
- `chords.c` (a CoreMIDI chord source; Live did not take it), `ca72-chords-4.mid` (the hosts'
  clip; dropped from a folder in `~/Music`, now in the Trash), `wins.swift` (a process's windows and their
  on-screen flags), `cpu.py`, `state.sh`, `results.txt` (item 1, every run in order, with the
  invalid one marked), `gb-hi-auhost.sample.txt`, `measure-build.patch`, `source-commit.txt`,
  `src/` (the renamed build's tree; bundles moved to the Trash after the runs).
- `prof.sh`, `top.py`, `incl.py`, `{lo,hi,ultra}-1voice-feedback-funk.{txt,sample.txt}` (item 2).
- `matrix.sh`, `matrix.txt`, `matrix-table.md`, `lab.sh`, `lab.txt`, `preset-cost.txt` (item 3).
- `spin.sh`, `spin.txt` (item 4).
- `bias-counter.patch`, `bias-counts.txt` (item 5).

Live's audio settings were left at Virtual Desktop Speakers, 48 kHz, 256 samples (they were
"No Device" with the MOTU away); REAPER ran from its own config folder; GarageBand's two
untitled test projects were discarded, `CA72-pitchbend-MIDI` closed without saving.
