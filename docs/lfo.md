# The LFO page: the engine's interface (decisions.md R-LFO)

The owner's decisions are in decisions.md, R-LFO. This note is the interface the panel and the
editor build on: what is a parameter, what is patch state, what the editor calls, and what the
audio thread gives back. The code is `crates/ca72-plugin/src/lfo.rs`.

## Host parameters (automatable, MIDI-learnable)

Four LFOs, N = 1 to 4, after every other parameter (none before them moves). Each is a nested
`lfo::LfoParams`, reached as `params.lfo(n)` (n = 0 to 3) or `params.lfo1` to `params.lfo4`;
the IDs are `lfo::PARAM_IDS[n]`.

| ID | Type | Range, default | The panel's control |
|---|---|---|---|
| `lfoN_waveform` | `EnumParam<params::Wave>` | the oscillators' six, TRIANGLE | WAVEFORM, STEPPED |
| `lfoN_shape_mode` | `EnumParam<params::WaveMode>` | STEPPED, CONTINUOUS; STEPPED | WAVEFORM's menu |
| `lfoN_shape` | `FloatParam` | 0..1 of CONTINUOUS's travel (0 TRIANGLE, 1 the floor), 0 | WAVEFORM, CONTINUOUS |
| `lfoN_rate` | `FloatParam` | 0.01..50 Hz, skewed (factor 0.2, about 1.6 Hz at half travel), 2 Hz | RATE |
| `lfoN_timed` | `BoolParam` | off | RATE's timed key |
| `lfoN_division` | `EnumParam<params::Division>` | the timed oscillators' 18, 1/4 | RATE while timed |
| `lfoN_phase` | `FloatParam` | 0..360 degrees, 0 | PHASE |
| `lfoN_rise` | `FloatParam` | 0..10 s, cube-root travel (factor 1/3: 1 s at 0.46), 0 | RISE |
| `lfoN_mode` | `EnumParam<lfo::LfoMode>` | FREE, RETRIG, ENV; FREE | MODE (three-position toggle) |
| `lfoN_mono` | `BoolParam` | on | MONO |

They are a preset's and a session's like any sound parameter; a preset or session saved before
reads each at its default. They do not glide (they are not knobs on the audio path).

## Patch state: assignments, depths and EDGES (not host parameters)

`params.lfo_patch: Arc<lfo::Patch>`, persisted in the session under `lfo`, and in presets as
values `lfoN_to_<target id>` (the depth in percent) and `lfoN_edges` (1 where SHARP).

- `Patch::assignments(lfo) -> [Option<Assignment>; MOST]`: in the order made, `MOST` = 8.
  `Assignment { target: Target, depth: f32 }`, depth -1..1, a share of the knob's travel.
- `Patch::assign(lfo, target, depth) -> Result<(), Refused>`: adds, or sets the depth if
  already there; `Refused::Full` past 8. A drop on a knob uses `lfo::DEFAULT_DEPTH` (0.25).
- `Patch::set_depth(lfo, target, depth) -> bool`, `Patch::remove(lfo, target) -> bool`,
  `Patch::clear()`.
- `Patch::depth(lfo, target)`, `Patch::on_knob(target) -> [Option<f32>; 4]` (the rings, the
  hover pop-up's dials, the tip, the menu's heads), `Patch::count(lfo)` (MODULATES' screen).
- `Patch::edges(lfo) -> Edges`, `Patch::set_edges(lfo, Edges::Soft | Edges::Sharp)`.
- `Patch::changes() -> u32`: bumped on every change (redraw when it moves).

Setting these from the editor needs no host gesture (they are not parameters). Init clears
them; choosing a preset loads its own (none where it names none).

## Targets: which controls can take an LFO

`lfo::Target` (29): the panel's continuous knobs and the strip's (26), and each oscillator's
CONTINUOUS shape. `Target::id()` is the parameter's ID, `Target::of_id`, `Target::short()` a
name for MODULATES' ten-character screen, `Target::per_voice()` (false for VOLUME and the
strip's ENTROPY, WIDTH, INNER, DETUNE and DRIVE: those always take the shared copy, option C,
which the hover pop-up marks "shared").

- `lfo::target_of(params, control_param_id) -> Option<Target>` and `lfo::is_lfo_target`: by
  the panel control's parameter ID. `oscN_waveform` gives `oscN_shape` while CONTINUOUS, none
  while STEPPED; `osc2_frequency` and `osc3_frequency` none while timed (their knob is then a
  DIVISION dial). Switches, selectors, VOICES, the wheels, the drawers' knobs and the LFOs' own
  knobs are none. An assignment already made stays where its knob cannot take one (it does
  nothing then).
- `Target::travel(params, plain, law)` and `Target::plain_at(params, travel, law)`: the knob's
  travel (its pointer, 0..1) and its parameter's value; FREQUENCY's through the computer's law
  (`params.freq_law()`), as its knob reads. A ring's swing is `travel ± depth`, clamped to 0..1.

## What the audio thread gives back

`params.lfo_scope: Arc<lfo::Scope>`, published after each block while an editor is open.
`Scope::read(lfo) -> Reading { value, phase, since, rise, moving }`:

- `value`: the output, -1..1, after RISE and EDGES (the latest voice's where MONO is off): a
  knob's present point is `travel + depth × value`, clamped.
- `phase`: where in its cycle (0..1, PHASE included).
- `since`: seconds since the note that last started it (the scope's time base starts there).
- `rise`: RISE's gain now (0..1).
- `moving`: assigned, running and not an ENV whose cycle is over (the orb's frame runs only
  while some LFO moves). `Scope::any_moving()`: the rail key's faint glow.

For the still trace: `lfo::settings(params, n, edges) -> Settings`, and
`Settings::output_after(t, tempo)` (the output `t` s after a note, from PHASE, ENV's hold and
RISE's fade, before EDGES), `Settings::hz_at(tempo)`, `lfo::wave(at, phase)` (the shapes).

## Colours: the computer's

`settings::Settings::lfo_colors: [lfo::Phosphor; 4]`, kept in `settings.toml` as `lfo_colors`
(four words); `settings::Change::LfoColor(lfo, Phosphor)` sets one. Defaults
`lfo::COLORS` = amber, green, blue, red. `Phosphor::ALL` (amber, green, blue, red, violet,
white) is the scope's right-click palette, `Phosphor::rgb()` its glow, `Phosphor::name()` its
menu word. The green leans to blue and the red to rose, so the four defaults stay apart for
deuteranopia and protanopia (R-LFO has the check).

## The engine, in short

- Plain generators (`lfo::wave`): the legends' exact shapes, bipolar -1..1; CONTINUOUS blends
  places as the oscillators do; past SQUARE the duty narrows from exactly 50 % through the
  circuit's 30 and 17.9 % to the floor (about 6.4 %).
- FREE runs on (timed: its phase from the host's place each block, never drifting); RETRIG
  starts at PHASE at each new note; ENV runs one cycle from PHASE and holds. A new note follows
  RETRIGGER: off, a key pressed with none held; on, every key. A POLY voice's own LFOs start
  with each note it takes.
- RISE fades the output in from the new note. EDGES SOFT slews it (10 to 90 % in 2 ms).
- Modulation every 32 samples, where the voices are given their panel: travel after the glide
  plus the sum of depth × value, clamped, through the knob's law; each voice's own where MONO
  is off (option C for the shared targets).
- No assignment (or every depth 0): nothing runs, the sound is the same to the bit, and an LFO
  never wakes the sleeping instrument (R68).
