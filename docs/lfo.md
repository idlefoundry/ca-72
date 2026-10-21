# The LFO page and the macros: the engine's interface (decisions.md R-LFO, R-MACROS)

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
| `lfoN_polarity` | `EnumParam<lfo::Polarity>` | BI (Bipolar), UNI (Unipolar); BI | UNI's key |
| `lfoN_reverse` | `BoolParam` | off | WAVEFORM's menu: REVERSE |
| `lfoN_on` | `BoolParam` | on | ON's key (its word the title) |

POLARITY, REVERSE and ON (R-LFO's corrections) are `lfo::LfoKeys`, nested after the macros so
that no parameter before them moves (`params.lfo_keys(n)`, `params.lfo1_keys` to `lfo4_keys`);
`lfo::sound_params(p, n)` gives all thirteen of an LFO's in `lfo::PARAM_IDS[n]`'s order. UNI's
output runs 0..1 (BI's -1..1), so its knobs move one way, by the sign of their depths; REVERSE
reads the waveform's cycle at 1 − its place; ON off fades the output to nothing in a straight
line over `lfo::MUTE_FADE` (10 ms) and back, the generator running on underneath, its
assignments kept, and the LFO no longer moving (the rail's glow, the orb's frame).

They are a preset's and a session's like any sound parameter; a preset or session saved before
reads each at its default. They do not glide (they are not knobs on the audio path).

## Modulation sources: the LFOs and the macros

Everything that moves a knob by a depth is a **source** (`lfo::Source`): `Source::Lfo(0..4)` and
`Source::Macro(0..6)`. A source has a value; each knob assigned to it
moves by depth × value from its own travel (after its glide), clamped, through its law; the
rings, the hover dials, the tip and the menu show every source on a knob alike.

- `lfo::SOURCES` = `LFOS + MACROS` (4 + 6). Every array of the patch and of the engine's `Lfos`
  is sized by `SOURCES` and indexed by `Source::index()` (the LFOs first).
- `Source::id()`: `lfo1` to `lfo4`, then `macro1` on: what presets (`<source>_to_<target>`) and
  sessions (`SavedSource { source, to, sharp }`) save.
- `Lfos::own(i)`: whether a source has a value of each voice's own (an LFO with MONO off). A
  macro never: it is shared, as a knob.
- **The macros** (R-MACROS): host parameters `macro1` to `macro6` (`lfo::MACRO_IDS`, host names
  `MACRO 1` to `MACRO 6` whatever their typed names; `params.macro_param(n)`), 0 to 100 %, 0 in a
  new patch, gliding as the panel's knobs (`smooth::KNOBS`), MIDI-learnable, a preset's sound
  parameters (one saved before has each at 0) and in an old session at 0 (`Ca72::filter_state`).
  A macro's value is **unipolar**: `Lfos::macros[n]`, its knob's glided value 0..1, filled by
  `lfo::controls` only where the macro has an assignment of a depth other than 0 (so that one
  turned with nothing to move leaves the controls, and the sleeping instrument, as they are).
  `Lfos::source_values(lfos)` gives every source's value: the LFOs' from the bank, the macros'
  from `Lfos::macros`. A knob a macro moves is at its travel + depth × value, clamped: one way.
  Each macro has a typed **name** (`Patch::name`, `set_name`, `names`; 7 printable characters,
  capitals: `lfo::name_within`), patch state as the assignments are. EDGES is an LFO's only.

## Patch state: assignments, depths and EDGES (not host parameters)

`params.lfo_patch: Arc<lfo::Patch>`, persisted in the session under `lfo` (`Saved { sources,
names }`, `names` left out while none is typed), and in presets as values `<source>_to_<target
id>` (the depth in percent: `lfo1_to_cutoff = 40`, `macro2_to_drive = 50`) and `lfoN_edges` (1
where SHARP), the macros' names a list of their own, `macro_names` (`library::Sound`), written
only where one is typed.

- `Patch::assignments(source) -> [Option<Assignment>; MOST]`: in the order made, `MOST` = 8.
  `Assignment { target: Target, depth: f32 }`, depth -1..1, a share of the knob's travel: an
  LFO at +25 % swings its knob 0.25 of its travel (its pointer, not seconds or hertz) each way
  about its setting, a macro at +25 % and 100 % moves it up 0.25, clamped at the travel's ends
  (R-LFO's corrections). The pop-up's depth dial turns as far as the ring's end.
- `Patch::assign(source, target, depth) -> Result<(), Refused>`: adds, or sets the depth if
  already there; `Refused::Full` past 8, `Refused::NoSource`. A drop on a knob uses
  `lfo::DEFAULT_DEPTH` (0.25).
- `Patch::set_depth(source, target, depth) -> bool`, `Patch::remove(source, target) -> bool`,
  `Patch::clear()`.
- `Patch::depth(source, target)`, `Patch::on_knob(target) -> [Option<f32>; SOURCES]` (the
  rings, the hover pop-up's dials, the tip, the menu's heads), `Patch::count(source)`
  (MODULATES' screen).
- `Patch::edges(lfo) -> Edges`, `Patch::set_edges(lfo, Edges::Soft | Edges::Sharp)` (an LFO's,
  by its number 0 to 3).
- `Patch::name(n)`, `Patch::set_name(n, name)`, `Patch::names()`, `Patch::load_names(&[String])`
  (a preset's), `Patch::same_names(&[String])` (the presets' bar marks a preset changed where its
  names differ).
- `Patch::changes() -> u32`: bumped on every change (redraw when it moves).

Setting these from the editor needs no host gesture (they are not parameters). Init clears
them; choosing a preset loads its own (none where it names none); a MIDI pad's preset too.

## Targets: which controls can take an LFO

`lfo::Target` (30), the same for every source (the macros' knobs, and the LFOs' own, are not
targets): the panel's continuous knobs and the strip's (27, the strip's LEVEL last: R-LFO's
corrections), and each oscillator's
CONTINUOUS shape. `Target::id()` is the parameter's ID, `Target::of_id`, `Target::short()` a
name for MODULATES' ten-character screen, `Target::per_voice()` (false for VOLUME and the
strip's ENTROPY, WIDTH, INNER, DETUNE, DRIVE and LEVEL: those always take the shared copy, option C,
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

- `value`: the output, -1..1 (0..1 UNI), after RISE, EDGES and ON's fade (the latest voice's
  where MONO is off): a knob's present point is `travel + depth × value`, clamped.
- `phase`: where in its cycle (0..1, PHASE included).
- `since`: seconds since the note that last started it (how far an ENV is through its cycle).
- `rise`: RISE's gain now (0..1).
- `moving`: assigned, running, not an ENV whose cycle is over, and not muted and faded out (the
  orb's frame runs only while some LFO moves). `Scope::any_moving()`: the rail key's faint glow.

For the still trace: `lfo::settings(params, n, edges) -> Settings`, and
`Settings::cycle_at(x)` (the waveform `x` of the way through one cycle from PHASE, REVERSE and
POLARITY as set, before RISE and EDGES; at 1 the cycle's last value), `Settings::hz_at(tempo)`,
`lfo::wave(at, phase)` (the shapes), `lfo::read(at, phase, reverse)`.

## Colours: the computer's (the macros' too)

`settings::Settings::lfo_colors: [lfo::Phosphor; 4]`, kept in `settings.toml` as `lfo_colors`
(four words); `settings::Change::LfoColor(lfo, Phosphor)` sets one. Defaults
`lfo::COLORS` = amber, green, blue, red. `Phosphor::ALL` (amber, green, blue, red, violet,
white) is the scope's right-click palette, `Phosphor::rgb()` its glow, `Phosphor::name()` its
menu word. The green leans to blue and the red to rose, so the four defaults stay apart for
deuteranopia and protanopia (R-LFO has the check).

The macros' are `settings::Settings::macro_colors: [lfo::MacroColor; 6]`, kept as `macro_colors`,
set with `Change::MacroColor` (a right click on a macro's name or jack). Defaults
`lfo::MACRO_COLORS` = teal, rose, lemon, sage, lavender, white; `MacroColor::ALL` adds round 2's
purple and pink. Which the left area shows, the wheels or the macros, is the computer's too:
`Settings::macros_shown`, kept as `left_area` ("wheels", "macros"), `Change::Macros`.

## MCP

`ca72_modulations`, `ca72_modulation_set` and `ca72_modulation_remove` (docs/mcp.md), carried
out by `agent::modulation` on the patch, for the LFOs and the macros (a macro also by its typed
name; `name` sets one); the LFOs' 40 parameters and the macros' six are in `ca72_describe`'s
table.

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

## The editor's side: the page, the rings, the drag (R-LFO-PAGE)

- **The page** is `ca72_panel::strip::lfo` (`LfoRenderer`, `LfoScene`), a lower page in the
  keyboard's place shown by the rail's LFO key; its 28 controls are `CONTROLS`' `Place::Lfo`
  (`controls::lfo_index(n, k)`, `LFO_PARTS`), found by `interact::hit_lfo` while it shows. The
  editor's state and glue are `crates/ca72-plugin/src/editor/lfo_page.rs` (`Modulating`).
- **The scene** each frame: the controls' values as the panel's scene has them; each scope's
  trace one cycle from PHASE, `lfo::settings(..).cycle_at(x)` (R-LFO's corrections: the owner,
  2026-10-20, "it should be 1"); the orb from the last `Scope::read` (read at most 30 times a
  second, only while the page shows) along that cycle by `phase` less PHASE (`lfo_page::orb_at`;
  an ENV at the cycle's end once over); at `value`, its wake at `rise`'s gain (RISE shows as the
  orb under the trace), or on the trace while it does not move. MODULATES from
  `Patch::assignments(Source::Lfo(n))`. UNI's trace and orb lie above the centre line (0..1);
  muted (`Column::on` off), the trace is drawn at about a third, the orb goes once faded out,
  and MODULATES reads MUTED, dim. A UNI LFO's rings run one way (`Ring::one_way`, as a macro's),
  a muted one's are dimmed (`Ring::dim`); the pop-up marks a dial UNI and MUTED
  (`PopDial::note`), and the knob's tip adds them after the depth. The title is ON's word: a press
  on it (`LfoTarget::Title`) is a press on the key.
- **The overlay** is `ca72_panel::modulation` (`ModRenderer`, `ModScene`): knobs' rings, a
  dragged assignment's cord and targets, the hover pop-up. Nothing in it names an LFO: a ring, a
  dial and a cord each take a colour and (the pop-up's) a label. A knob's rings are on its own
  sweep (`KnobRings::sweep`, from `art::sweep`: a knob's 300 degrees, a CONTINUOUS WAVEFORM's
  199.3), a depth × that sweep from its pointer; the pop-up's dial turns as far (`Popup::span`).

## The macros in the editor (R-MACROS)

- **The area** is `ca72_panel::strip::macros` (`MacroRenderer`, `MacroScene`): the lower page's
  left area (`AREA`, the strip's left trim to the left hand's rule, the rail's foot to the strip's
  foot), laid on the strip's bare face there (`Renderer::left_face`, kept as the background is
  made, before the left hand's shadows, print and parts) so that it lies without a seam. Six
  cells two by three (`knob_at`, `name_at`, `jack_at`: round 2's budget); the knobs are
  `CONTROLS`' `Place::Macro` (`controls::MACRO_FIRST` + n, CONTROL_SET revision 5), found by
  `interact::hit_macro` while the macros show; the names and jacks by `strip::macros::hit`. Its
  still (face, shadows, the dials' print, the screens' glass, the lamp) is made on a thread of its
  own as the editor opens (`MacroRenderer::prepare`); each name, collar with its jack, and knob is
  a patch drawn again only when it changes.
- **The editor's side** is `editor/macros.rs` (`MacroView`): the rail's MACRO key
  (`BarTarget::Macro`, at the far left; `BarScene::macros` lit, `macro_glow` the faint glow),
  `Editing::macros_show`; while they show the left hand's controls (`Place::Column`) are off the
  panel (not drawn over, not found) and go on working. A press on a jack starts
  `Modulating::patching` with `Source::Macro(n)`; the cord, the drop, the pop-up, the ring ends,
  Alt-drag and the menus' heads are `lfo_page`'s, over sources (`colour`, `label` with the typed
  name). A macro's ring is one way (`modulation::Ring::one_way`) and shows on every knob it moves
  whenever the macros show, its point where the macro has it.
