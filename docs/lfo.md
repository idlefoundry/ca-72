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
| `lfoN_level` | `FloatParam` | 0..100 %, linear, 100 | its OUT jack (no knob: the wheel, typed, a cord) |

POLARITY, REVERSE and ON (R-LFO's corrections) are `lfo::LfoKeys`, nested after the macros so
that no parameter before them moves (`params.lfo_keys(n)`, `params.lfo1_keys` to `lfo4_keys`);
`lfo::sound_params(p, n)` gives all fourteen of an LFO's in `lfo::PARAM_IDS[n]`'s order. LEVEL
(R-LFO's addendum) is `lfo::LfoLevel`, a nest after the keys (`params.lfo_level(n)`): the share
of every one of its LFO's assignments it gives, its output × LEVEL wherever it goes (`Settings::level`;
at 100 % × 1, the same to the bit). UNI's
output runs 0..1 (BI's -1..1), so its knobs move one way, by the sign of their depths; REVERSE
reads the waveform's cycle at 1 − its place; ON off fades the output to nothing in a straight
line over `lfo::MUTE_FADE` (10 ms) and back, the generator running on underneath, its
assignments kept, and the LFO no longer moving (the rail's glow, the orb's frame).

They are a preset's and a session's like any sound parameter; a preset or session saved before
reads each at its default. They do not glide (they are not knobs on the audio path).

## Modulation sources: the LFOs and the macros

Everything that moves a knob by a depth is a **source** (`lfo::Source`): `Source::Lfo(0..4)`,
`Source::Macro(0..6)` and `Source::Velocity` (decisions.md R-VELOCITY). A source has a value; each knob assigned to it
moves by depth × value from its own travel (after its glide), clamped, through its law; the
rings, the hover dials, the tip and the menu show every source on a knob alike.

- `lfo::SOURCES` = `LFOS + MACROS + VELOCITIES` (4 + 6 + 1). Every array of the patch and of the engine's `Lfos`
  is sized by `SOURCES` and indexed by `Source::index()` (the LFOs first).
- `Source::id()`: `lfo1` to `lfo4`, then `macro1` on, then `velocity`: what presets (`<source>_to_<target>`) and
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
- **The velocity** (R-VELOCITY): `Source::Velocity`, after the macros (`lfo::VELOCITY`), its ID
  `velocity`. Unipolar as a macro, and each voice's own (`Lfos::own` true: its note's), its shared
  copy the latest note's. `Lfos::source_values(lfos, velocity)` gives its value for a note's
  velocity through `Lfos::velocity`, the drawer's CURVE and FLOOR (`lfo::Response`, from
  `velocity_curve` and `velocity_floor`, filled only where it is assigned). A key's velocity
  reaches the engine as `Event::NoteVelocity` before its note. It moves neither an LFO's own
  knobs nor TEMPO (`Source::takes`).

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

`lfo::Target` (`TARGETS`, 51), the same for every source (the macros' knobs are not targets):
the panel's continuous knobs and the strip's (27, the strip's LEVEL last: R-LFO's corrections),
each oscillator's CONTINUOUS shape (`KNOB_TARGETS`, 30, first), then each LFO's own five
(R-LFO's addendum), LFO by LFO: `Target::LfoRate(n)`, `LfoPhase(n)`, `LfoRise(n)`,
`LfoShape(n)` (its CONTINUOUS shape) and `LfoLevel(n)`, `lfo::LfoKnob` their kind
(`Target::lfo_knob`, `Target::of_lfo`, `Target::is_lfo`). Their IDs are their parameters'
(`lfo2_rate`, `lfo2_shape`, `lfo2_level`), their screen names `LFO2 LEVEL` and so on (ten
characters; the strip's is `LEVEL`), `Target::name()` "LFO 2 LEVEL". `Target::id()` is the parameter's ID, `Target::of_id`, `Target::short()` a
name for MODULATES' ten-character screen, `Target::per_voice()` (false for VOLUME and the
strip's ENTROPY, WIDTH, INNER, DETUNE, DRIVE and LEVEL: those always take the shared copy, option C,
which the hover pop-up marks "shared").

- `lfo::target_of(params, control_param_id) -> Option<Target>` and `lfo::is_lfo_target`: by
  the panel control's parameter ID. `oscN_waveform` gives `oscN_shape` while CONTINUOUS, none
  while STEPPED; `osc2_frequency` and `osc3_frequency` none while timed (their knob is then a
  DIVISION dial); `lfoN_waveform` gives `LfoShape(n)` while CONTINUOUS, `lfoN_rate` none while
  timed, `lfoN_phase`, `lfoN_rise` and `lfoN_level` always. Switches, selectors, VOICES, the
  wheels and the drawers' knobs are none. An assignment already made stays where its knob cannot
  take one (it does nothing then).
- `lfo::takes_shared(params, source, target)`: whether a MONO-off LFO moves `target` by its
  shared copy (a control every voice shares, or a MONO LFO's own knob: option C); the pop-up's
  SHARED and the agents' `shared`.
- **TEMPO** (`Target::Tempo`, `tempo`, last; R-LFO's "The standalone's TEMPO a target"): the
  standalone's alone (`target_of` none in a plug-in, `Ca72Params::is_standalone`); its travel
  30 to 300 BPM straight along it (`lfo::tempo_travel`, `lfo::tempo_at`, the law
  `Ca72Params::tempo_knob`'s); moved on the ticks below (`Target::ticks`); `lfo::controls`
  leaves its assignments out in a plug-in, which keeps them. The engine's `Engine::tempo()` is
  the tempo of the latest tick while a source moves it, `Engine::next_tempo_tick()` where in the
  block the next falls; the plug-in's `process` retimes the arpeggiator (`Arp::retime`) and the
  timed clock (`timed::Clock::retime`, `Engine::retime`) there, so they go on from the beats
  they had come to.
- `Target::travel(params, plain, law)` and `Target::plain_at(params, travel, law)`: the knob's
  travel (its pointer, 0..1) and its parameter's value; FREQUENCY's through the computer's law
  (`params.freq_law()`), as its knob reads. A ring's swing is `travel ± depth`, clamped to 0..1.

## The LFOs moving the LFOs (R-LFO's addendum)

While any source moves an LFO's own knob or TEMPO (`Lfos::cross()`), the engine's generators run on
**ticks** fixed in audio time, `lfo::TICK` (0.5 ms, `TICKS_A_SECOND` 2000), numbered from the
engine's own count of host samples since it was prepared (the engine's `played`; `lfo::tick_at`), not
from blocks or steps; otherwise they run as before, to the bit (`Bank::step`, `advance`).

- **A tick** (`Bank::cross_to`, run by `Engine::modulate` every step, before a note starts an
  LFO and at a run's end): each copy (the shared, each voice's where MONO is off) is moved on
  from the latest tick at the settings in force since (`Gen::tick_on`: its cycle at its RATE or
  the tempo's, the time since its note, ON's fade); then its knobs are set from the sources'
  values **at the latest tick** (`lfo::moved`: the knob's travel + Σ depth × value, clamped,
  through its parameter's own range, `RATE_RANGE` and the rest), so nothing feeds back within
  one; then its value there (`Gen::tick_value`: waveform, RISE, POLARITY, ON's fade, LEVEL; not
  EDGES' slew, which smooths only what reaches the sound). `lfo::Cross` keeps each copy's
  settings in force and its value, and TEMPO in force (`Cross::tempo`, by the shared copies'
  values at the latest tick).
- **The sound** reads each generator at its own time between ticks (`Bank::peek`: its cycle on
  at the settings in force, RISE, EDGES' slew, ON's fade, LEVEL), as before every step.
- **The glide's end** (R-LFO-STRESS), where an LFO moves a pitch or CUTOFF: the same ticks and
  reading run on a copy of the bank (`Ahead::bank`, kept so the step allocates nothing) to the
  step's end and halfway, so the glide ends where the next step's knobs will be, the LFOs'
  moves of each other included. With TEMPO moved, the plug-in ends a run at each tick, so no
  step spans a change of tempo and the copy's clock is the step's own.
- **Per voice:** a voice's own LFO (MONO off) is moved by the sources' own values for that voice,
  a shared LFO by their shared copies (option C); a macro is shared.
- **A note** is taken at the latest tick; a RETRIG or ENV LFO it starts forgets its moved knobs
  and its value (its settings as set, its value its start's), so a loop of them starts again
  exactly at each note. A FREE one runs on.
- **What is so** (`engine_lfo_cross_tests.rs`): every value within -1..1 and finite, every knob
  within its range, for ten minutes of loops; the same values, to the bit, every 10 ms at 44.1, 48
  and 96 kHz in blocks of 64 and 1024 (untimed LFOs: their ticks' arithmetic is in time alone;
  a timed one follows the host's clock, the same within its rounding); live and offline alike.

## What the audio thread gives back

`params.lfo_scope: Arc<lfo::Scope>`, published after each block while an editor is open.
`Scope::read(lfo) -> Reading { value, phase, since, rise, moving, shape, offset, level, moved }`:

- `value`: the output, -1..1 (0..1 UNI), after RISE, EDGES and ON's fade (the latest voice's
  where MONO is off): a knob's present point is `travel + depth × value`, clamped.
- `phase`: where in its cycle (0..1, PHASE included).
- `since`: seconds since the note that last started it (how far an ENV is through its cycle).
- `rise`: RISE's gain now (0..1).
- `moving`: assigned, running, not an ENV whose cycle is over, and not muted and faded out (the
  orb's frame runs only while some LFO moves). `Scope::any_moving()`: the rail key's faint glow.
- `shape`, `offset`, `level`, `moved`: its WAVEFORM's place, PHASE (cycles) and LEVEL as in
  force, and whether a source moves them (`Lfos::trace_moved`): the scope then draws its trace
  from them (`lfo_page::drawn_settings`, `trace_of`; R-LFO's "Scope traces as the engine plays
  them"), by the cheaper glow (`Column::live`).

The strip's screens of controls a source moves read the engine's modulated controls
(`Engine::controls_now`, published by the editor's `Meters::publish_live`; R-LFO's "Screens
reading the modulated value"), and the TEMPO screen the moving tempo (`Meters::publish_tempo`).

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
table. `tempo` is a target in the standalone alone (refused in a plug-in, "standalone only";
one a session brought there listed `inert`).

## The engine, in short

- Plain generators (`lfo::wave`): the legends' exact shapes, bipolar -1..1; CONTINUOUS blends
  places as the oscillators do; past SQUARE the duty narrows from exactly 50 % through the
  circuit's 30 and 17.9 % to the floor (about 6.4 %).
- FREE runs on (timed: its phase from the host's place each block, never drifting); RETRIG
  starts at PHASE and resets RISE at every note-on, independently of the main contour's
  RETRIGGER switch (R-LFO-NOTE-RETRIG). ENV runs one cycle from PHASE and holds. Shared ENV
  and FREE's RISE follow main RETRIGGER: off, a key from silence; on, every key. A POLY voice's own LFOs start
  with each note it takes.
- RISE fades the output in from the new note. EDGES SOFT slews it (10 to 90 % in 2 ms).
- Modulation every 32 samples, where the voices are given their panel: travel after the glide
  plus the sum of depth × value, clamped, through the knob's law; each voice's own where MONO
  is off (option C for the shared targets).
- Between those steps (R-LFO-STRESS): where an LFO moves TUNE, oscillator 2's or 3's FREQUENCY
  or CUTOFF (`Target::fast`, `Lfos::fast`), each voice glides what it reads of them in a
  straight line from the step's value to the next step's, a sample at a time (`Sweep`; in the
  circuit `Jacks::swept`, after the knob's law; in the light voice `light::swept_laws`: the
  pitches' semitones and CUTOFF's current into the control node, `Light::set_cutoff_swept`). The
  next step's values come from copies of the generators moved on (`Bank::ahead`; on the ticks,
  a copy of the bank run to the next step's ticks), so the glide adds no delay and is exact at
  each step. A step an LFO jumps in (halfway across, more than `lfo::JUMP` off the line between
  its ends: a SHARP edge, a cycle's restart) is held, so the edge stays abrupt. A macro's part
  and the velocity's (each voice's own, where the next step's glide will have it:
  `Velocity::ahead`) are in both ends. A macro or the velocity alone glides nothing: each
  holds still between steps, apart from the velocity's 4 ms settling under a legato phrase.
- MAIN OUTPUT's VOLUME moved by a source ramps a sample at a time across the step to the step's
  gain, unless only SHARP LFOs move it (`Lfos::sharp_on`); the controls set from outside land at
  once, as before. Every other knob still steps every 32 samples. With nothing assigned none of
  this runs.
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

## The LFOs' own knobs in the editor (R-LFO's addendum)

- With the page showing, its controls are in view for modulation (`lfo_page::in_view`,
  `Editing::mod_hit` finding the page's): RATE, PHASE, RISE and WAVEFORM (CONTINUOUS) take a cord,
  rings on their own sweeps, the hover pop-up, the tip's depths and the menu's heads, as any knob.
- **LEVEL is the OUT jack's:** each LFO's `lfoN_level` control, `Kind::Jack` at the jack
  (`controls::lfo_index(n, LFO_LEVEL_PART)`, from `LFO_LEVEL_FIRST`; CONTROL_SET revision 7), drawn
  by the page as the jack (`LfoRenderer` draws nothing for it), its rings round the jack
  (`art::skirt`). A press on it still draws the LFO's cord (`lfo_page::page_press`); a cord let go
  on a jack gives that LFO's LEVEL, unless it is its own jack's and never left it (`Patching::away`:
  a click); a double click there sets 100 %; Ctrl or Cmd and a click types it; the wheel turns
  it; its tip is LEVEL's with what the jack does after; its menu has LEVEL's heads, then the LFO's
  COLOR and EDGES. A pop-up that would leave the window under its knob stands beside it
  (`modulation::popup_layout`).

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

## Effect parameters (R-FX-PARAMS)

The standalone's effects' parameters as modulation targets (decisions.md R-FX-PARAMS; the
single label strip, R-FX-STRIP; drawings in `ca72-night/fx-strip-mockups`). Up to four parameters of each effect
are **pinned**; each pin is one of sixteen fixed targets, `lfo::Target::Fx(0..16)` (ids `fx1` to
`fx16`), which stays the pin's while its effect is moved in the chain. The plug-ins have none.

- **The engine** (`engine.rs`, `Engine::modulate`): at each step of the LFOs (32 samples) where a
  source moves a pin, `FxPoints` records the step's frame in the block and each pin's offset on
  its travel (Σ depth × source, the sources' shared copies: an effect has no voices), asleep or
  not. `Lfos::modulated` leaves the pins out: they move no control of the synth's, and the synth's
  sound with a pin moved is the same to the bit. `Ca72::process` hands the block's points to the
  chain (`Runner::process_modulated`) after the synth.
- **The chain** (`ca72_fx::chain`): `Chain::pin`/`repin`/`unpin`, `touched`, `list_params`, `param_text`, `pins`,
  `set_pin_value`; each slot's pins saved beside its state (`SavedSlot::pins`, `SavedPin {id,
  name, base, target, mods}`) in `effects.json`, and so in a session and in a preset that carries
  the chain. The runner turns the points into the effect's events: CLAP's own modulation
  (`EV_PARAM_MOD`, the amount in its units) where the parameter is modulatable, else (CLAP) base
  plus offset as its value; VST3 an `EV_PARAM_MOD` the helper adds to the base for the processor
  alone (the window never chases it). At most 32 points a block (thinned past 1024 samples); a pin
  no longer moved gets one point at its base; nothing is sent for a pin nothing moves. The points
  never go through the values the host set (no snapshot, no `effects.json` written while an LFO
  moves a pin).
- **The base** is the plug-in's own value: its window's edits (the helper's `param_edits`
  notices), the drawer's knob (`set_pin_value`), a learned controller (`Runner::set_pin_value`,
  on the audio thread). It is saved, and given to the plug-in after its state at every load,
  restart and restore (its state may hold a modulated instant).
- **The assignments** to the pins are the chain's, not the sound's: `lfo::Patch` keeps them for
  the engine and the editor (rings, cords, menus as any knob's), but `Patch::saved` and
  `Patch::values` leave them out, and `Patch::load`, `clear` and `copy_from` keep them as they
  are. The chain reads and gives them through `ca72_fx::PinMods` (`fx::PatchMods`), and saves
  again when `Patch::fx_changes` moves.
- **Missing, crashed, restarting:** the pin and its assignments are kept, inert (nothing sent);
  `PinStatus::live` false (dimmed). A pin whose parameter the plug-in no longer lists reads
  `missing` (MISSING, UNPIN in its menu). They work again when the effect runs.
- **MIDI Learn** learns a pin by its target (`learn::LEARNABLE`'s `fx1` to `fx16`, EFFECT PIN 1 to
  16): the controller follows the pin, not its slot. In a plug-in they do nothing.
- **MCP:** `ca72_modulations` lists the pins (`pins`) and labels their assignments with the
  effect's and parameter's names; `ca72_modulation_set`/`_remove` take `fx1` to `fx16` while
  pinned to an effect that runs; `ca72_effect_parameters`, `ca72_effect_pin` and
  `ca72_effect_repin` and `ca72_effect_unpin` list, pin, replace and unpin.

**The editor's hooks, for the pins' view.** All on `fx_view::Effects` (the drawer reaches
the chain through it; `fx_view::installed()` is `None` in the plug-ins), and free functions:

| Hook | What it gives |
|---|---|
| `Effects::params(slot) -> Result<Vec<ParamRow>>` | The effect's parameters a player may pin (`ca72_fx::pinnable`: automatable, not hidden, read-only, a program change or the bypass), with their values now (`value` 0..1, `plain` in the plug-in's units), names, short names, units, steps, and the pin target if pinned. Asks the helper (milliseconds): call it as the PARAMETERS list opens and again when it is told the parameters changed, not every frame. |
| `fx_view::search_params(rows, typed, pinned) -> Vec<usize>` | The rows the list shows, by index: every typed word in the name, short name or units (case aside); `pinned` for ALL · PINNED's PINNED; in name order, then ID (the owner: "the parameter list, with nothing typed, is in name order"). |
| `Effects::param_text(slot, &[(id, plain)]) -> Vec<Option<String>>` | The plug-in's own text for each value ("2.40 s"; VST3's units put after a bare number): the list's value column, the tip and the pop-up. Asks the helper: once a list's rows, or a tip's value as it changes, not every frame. |
| `Effects::pins() -> Option<Vec<PinInfo>>` | Every pin by target: its slot now, effect, parameter, names, base (`value` 0..1, `plain`), steps, `live` (dim it when false), `missing`. Cheap (the chain's own table; `None` while the chain is busy: keep the last). Also publishes which targets can take a cord. |
| `Effects::pin(slot, id) -> Result<usize>` / `Effects::unpin(target)` | A click or Enter on a row; UNPIN. A fifth pin of an effect is refused (the error says so). Unpinning lets go of what moved it. |
| `Effects::touched(slot) -> Option<Vec<u32>>` | Up to four editor touches, newest first and unique; `None` while busy. The chain follows the helper's existing notices, preserving their order; these hints are not saved. A changed touch asks for the pinnable catalogue on the drawer's worker, never in the drawing path. Caches follow the effect's stable identity and request generation; failed requests retry after one second. |
| `Effects::repin(target, id)` | Another parameter of the same effect under the same target; keeps source assignments and MIDI Learn, takes the new parameter's current value as its base, and saves as a pin. The old parameter's modulation is released at its base. |
| `Effects::set_pin_value(target, travel)` | The pin's knob turned in the drawer: the plug-in's value from the next block, its window following. |
| `fx_view::pin_target(params, target) -> Option<lfo::Target>` | The target a cord or a macro may land on: `Some(Target::Fx(n))` while pinned to an effect that runs (`lfo::target_of(p, "fxN")`, from `lfo::FX_PINNED`). Its rings and menus are any target's: `Patch::on_knob(Target::Fx(n))`, `Patch::assign`, `set_depth`, `remove`. |
| `learn::fx_pin(id)`, `learn::is_fx_pin(i)` | MIDI Learn per pin: arm `learn::index("fxN")` as any learnable; the learning list should leave these out where `fx_view::installed()` is `None`. |

**The single label strip (R-FX-STRIP).** Every loaded effect has one piece of glass under its
name and ON key, with four equal fields exactly above four knob places. The first free field
reads **+ PIN** and opens the parameter list. Turn a pinnable control in the effect's own
window and that field offers **+ NAME**: click its name to pin in one click, or its arrow to
choose another parameter from the list. A touch of an already pinned or unpinnable parameter
offers nothing. The same split works in the last free field. Empty places have no knob.

Click a pinned field's name (its arrow appears on hover), or choose **REPLACE…** in its knob's
menu, to replace its parameter. The list is titled **REPLACE DECAY**, for example. Its rows
replace under the same target, retaining LFOs, macros and MIDI Learn; choosing the current
parameter or Escape changes nothing. **PARAMETERS…** stays in the slot's right-click menu.

The list hangs from the original row's foot, over the pins, fitting inside the window. The
title and pin count share line 0, FIND and ALL · PINNED line 1, and ten visible rows follow.
With FIND empty and ALL selected, **JUST TOUCHED** lists up to four parameters newest first,
then **ALL PARAMETERS** introduces the usual name order. Both heads are ruled and inert.
Each row gives its pin bullet, name, the effect's value text and its sources. Matched letters
are brighter and underlined. Click or Enter toggles a pin in PARAMETERS, replaces in REPLACE;
a refusal stays on the plate. Up/Down and Page Up/Down skip heads, Escape clears FIND then
closes, and Tab switches ALL · PINNED. The list holds the keyboard throughout.

The drawer is 370 drawing units deep with any effect loaded, 170 with none, so the first pin
does not move it. Each 570-unit strip spans the renderer's name screen, gap and ON key; its
four fields and knobs have 142.5-unit centres. Thin column rules use the LFO page's print.
A field shows the parameter's short name, or its own brighter value text while its knob is
hovered or turned. Its still parts (glass, shadows and ticks) and knob materials are made on
the drawer's thread. Changed fields are drawn as bounded parts, so neighbours' glass and
letters cannot be blended twice. The worker asks for value text as bases change. Pins and
touches are polled every tenth of a second, retaining the last while the chain is busy.
Closing and reopening the list reads a changed parameter catalogue again.

`editor/fx_pins.rs` gives each pin a control number past the panel's controls, as TEMPO has one,
so `editor/lfo_page.rs` supplies the same cord, coloured rings, one-way macro rings, pop-up depth
dials, Alt-drag, typed depth and source heads as a panel knob. Pins' rings are composed over
their drawer, under its list. The tip names the parameter and effect, gives its own value text,
and adds every source's depth. A turn, including scrolling, sets its base and observes its steps.
A pin travels with its slot while its fixed target and MIDI Learn stay its own.

Unavailable pins keep their names, rings and assignments, dimmed, refusing turns and new cords;
a parameter no longer found reads MISSING. REPLACE… and UNPIN are in each pin's menu, followed by MIDI Learn.
The learning list names pins by parameter and effect on the standalone's effects rail, never
on the plug-ins' rail. Both built-in skins and LARGE lettering use the same geometry.
