# CA-72

A synthesizer plug-in, VST3 and CLAP, for macOS, Windows and Linux, by Idle Foundry:
monophonic like the instrument it models, or one whole instrument per note with POLY. Its
voice is modelled from the circuit of a 1970s analogue synthesizer, the
Minimoog Model D of about 1972-73 with the CA3046 ("old") oscillator board. Every board
was transcribed from the instrument's service documents into SPICE netlists and simulated
in ngspice. The real-time models were then derived from those simulations and tested
against them, the factory calibration procedures included.

![The CA-72's panel](docs/panel.png)

It is free software under the GNU General Public License, version 3 or later.

**Status:** a pre-release. The builds are not yet signed or notarised for macOS.

## Installing

Build the bundles (see [Building](#building)), then copy `CA-72.vst3` and `CA-72.clap`
from `target/bundled/` into your plug-in folders:

| | VST3 | CLAP |
|---|---|---|
| macOS | `~/Library/Audio/Plug-Ins/VST3/` | `~/Library/Audio/Plug-Ins/CLAP/` |
| Windows | `C:\Program Files\Common Files\VST3\` | `C:\Program Files\Common Files\CLAP\` |
| Linux | `~/.vst3/` | `~/.clap/` |

On macOS, a bundle downloaded rather than built carries the quarantine flag, and an
unsigned bundle with that flag will not load. Clear it with
`xattr -dr com.apple.quarantine ~/Library/Audio/Plug-Ins/VST3/CA-72.vst3`, and the same
for the CLAP bundle.

## Playing it

- **Keys:** every MIDI note plays. Notes 41 to 84 are the instrument's 44 keys, F to C
  (with RANGE at 8'); beyond them the plug-in continues the key string's scale, a key a
  semitone, which the instrument cannot do. Like the instrument, the keyboard gives the
  lowest note held priority and triggers once while keys are held.
- **POLY** (under the panel, not the instrument's): off, the one instrument as above. On,
  each note plays a whole instrument of its own, with its own keyboard circuit, contours
  and glide, up to VOICES of them (2 to 10). A new note takes a free voice, else the one
  let go longest ago, else the one held longest; a voice that was still held lifts its key
  and presses the new one 13 ms later, so its contours start again (the circuit re-arms
  its trigger only after about 12 ms, and so the note's pitch and attack come 13 ms
  late). The wheels reach every voice. Switching POLY stops the notes sounding.
- **ANALOG** (0 to 100 %, off by default): each voice its own parts, as if built from a
  different batch: the three oscillators' tuning, the filter's cutoff and EMPHASIS, the
  contours' times and GLIDE, each off by a little; the oscillators and the cutoff drift
  slowly and waver. It applies to the one voice with POLY off too. At 0 the voice is the
  circuit as drawn, to the sample.
- **SPREAD** (0 to 100 %, off by default): POLY's voices across the stereo field, the
  first in the centre and the next alternately left and right. With POLY off the voice
  stays in the centre.
- **Wheels:** MIDI pitch bend moves the pitch by MIDI BEND RANGE semitones (2 by
  default), and the modulation wheel (CC 1) moves the MODULATION wheel. All sound off,
  reset all controllers and all notes off (CC 120, 121 and 123) are followed.
- **External input:** the host's side chain is the panel's EXTERNAL INPUT, through its
  MIXER switch and VOLUME.
- **Output:** stereo, the same on both channels unless SPREAD places POLY's voices; on a
  mono output, the two channels together.
- **POWER** is the host's bypass: the output fades out over 10 ms and back in.

The editor works like the panel:

- **Knobs:** drag up or down, or use the mouse wheel. Hold Shift to turn finely, and
  double-click to return a control to its default.
- **RANGE and WAVEFORM:** drag or scroll, click either side to step, or click a printed
  position to go straight to it.
- **Switches:** click.
- **Wheels:** drag. The PITCH wheel settles into its centre detent when let go near it.
- **Size:** the editor opens at 80 % of the screen's usable width. Drag the grip at the
  bottom right corner to change it. The panel keeps its proportions, and a size you set
  is saved with the session.

Hovering over a control shows its name and value. The name plate does something when
clicked.

Under the panel, a strip holds the plug-in's own controls: POLY (click), VOICES (click
either side to step), and ANALOG and SPREAD (drag the slider; the switch beside each turns
it off and back on at its amount). Beside VOICES it shows how many voices this machine
plays in real time at the host's block size, and marks VOICES when it asks for more.

## Building

The toolchain is Rust 1.97.1 (`rust-toolchain.toml`, which rustup installs on first use).

```sh
cargo xtask bundle ca72-plugin --release
```

This writes `target/bundled/CA-72.vst3` and `target/bundled/CA-72.clap`. On Linux, the
editor's window needs the X11 development libraries (Debian and Ubuntu:
`libx11-dev libx11-xcb-dev libxcb1-dev`).

To try the plug-in without a host, run it as an application of its own:

```sh
cargo run --release -p ca72-plugin --example standalone --features standalone
```

On Linux this also needs the ALSA, JACK and OpenGL development libraries
(`libasound2-dev libjack-jackd2-dev libgl-dev`).

## Testing

```sh
cargo test --workspace
```

The tests check the real-time models against the circuit lab's reference measurements.
They also check that the plug-in neither allocates nor frees memory on the audio thread,
and that the editor's gestures reach the host as single gestures. One test compares the
panel as drawn with the approved design.

`scripts/validate.sh` runs clap-validator, pluginval (at strictness 10) and Steinberg's
VST3 validator on the bundles, downloading or building each the first time. CI does all of
this on macOS, Windows and Linux.

The tests that run the circuit simulations need ngspice 47. Without it they say so and
pass; set `CA72_REQUIRE_NGSPICE=1` to make a missing ngspice a failure.
[docs/circuit/README.md](docs/circuit/README.md) describes building ngspice and running
the circuit lab.

## The repository

| Path | Contents |
|---|---|
| `crates/ca72` | The real-time models of the boards, and the voice built from them |
| `crates/ca72-plugin` | The plug-in, built with nih-plug, and its editor |
| `crates/ca72-panel` | The panel's art and its renderer |
| `crates/ca72-lab` | The circuit lab: ngspice benches of the boards, the factory calibration, reference measurements |
| `crates/ca72-spice` | ngspice as the offline reference: runs netlists, reads their results |
| `crates/ca72-analysis` | Measurements of rendered audio for the tests: pitch, loudness, spectrum, onsets |
| `crates/ca72-rt` | Real-time threads for the lab and the tests |
| `circuits/` | The transcribed netlists and the device models |
| `docs/circuit/` | How the model was derived, board by board, with its sources and assumptions |
| `docs/decisions.md` | The release's decisions |
| `docs/history.md` | The model's decisions, from its development in DAW-v3 |
| `evidence/` | Test and performance logs from the reference machine |
| `third_party/` | nih-plug (patched) and the TeX Gyre Adventor font |
| `xtask/` | Builds the bundles |

The service documents and datasheets the model was derived from are not included.
[docs/circuit/sources.md](docs/circuit/sources.md) lists them, and
`scripts/fetch-sources.sh` downloads them and checks their hashes.

## Limitations

- Only the real-time quality, Potato, is in the plug-in. The model's two more exact
  qualities do not run in real time; they remain in `crates/ca72` and the lab.
- POLY's voices all run on the host's audio thread, so how many play in real time depends
  on the machine and the host's block size (see [docs/decisions.md](docs/decisions.md),
  "POLY in real time"); the strip shows the number. With more, expect dropouts.
- Ten voices at full level with SPREAD can peak above 0 dBFS; lower MAIN OUTPUT's VOLUME.
- The rear panel's control voltage and trigger inputs are left out.
- PHONES and the jacks in the controller's column are drawn but do nothing.
- Building the voice when the plug-in is activated takes about 0.8 s the first time in a
  process, since it includes the factory calibration. A new sample rate after that takes
  0.1 to 0.4 s; the same rate again is immediate.
- Below 8 kHz the voice runs at 2, 4 or 8 times the host's rate.
- The editor has no keyboard control, and a host cannot resize it; use the grip.
- The first size comes from the main screen on macOS and the primary monitor on Windows.
  On Linux it comes from the whole X11 screen, which spans every monitor, so it can open
  wider than the monitor it is on.
- The macOS builds are unsigned.

## Credits

- The panel's layout was measured from a photograph of the instrument, "Minimoog
  panel.jpg" on Wikimedia Commons: a photograph by Alex Harden, cropped by Clusternote,
  under [CC BY 2.0](https://creativecommons.org/licenses/by/2.0/). The panel is drawn
  anew; the photograph is not included.
- The panel's lettering is TeX Gyre Adventor, by the GUST e-foundry, under the GUST Font
  License (`third_party/tex-gyre-adventor/`).
- The plug-in is built with [nih-plug](https://github.com/robbert-vdh/nih-plug), by Robbert
  van der Helm (ISC; its VST3 bindings GPL-3.0), patched as
  `third_party/nih-plug/PATCHES.md` describes. The editor uses
  [baseview](https://github.com/RustAudio/baseview),
  [softbuffer](https://github.com/rust-windowing/softbuffer) and
  [resvg](https://github.com/linebender/resvg).

## Trademarks

Minimoog and Model D are trademarks of Moog Music Inc. The CA-72 is not affiliated with,
sponsored or endorsed by Moog Music. Those names are used here only to say which
instrument's published circuit the model was derived from. VST is a registered trademark
of Steinberg Media Technologies GmbH.
