# Agents: the CA-72 through MCP

Issue [#76](https://github.com/idlefoundry/ca-72/issues/76); decisions.md R-MCP. Written
2026-10-18 for 0.3.0. An AI agent operates the CA-72 the way a player does, through the
[Model Context Protocol](https://modelcontextprotocol.io): it reads every control, turns them,
chooses, saves and organises presets, changes the settings and the view, uses MIDI Learn and
plays notes. Reference-sound matching (#77) and an agent skill for skins (#78) will build on
this interface. They are not part of it.

**Status.** The first version controls the **standalone application**. Plug-ins in a host
come later (see [Next: plug-ins in a host](#next-plug-ins-in-a-host)). It is built and tested
on Linux. macOS should work unchanged (Unix sockets), but it has not been tried. Windows needs
a named pipe and is not built yet. Several choices below are the owner's to confirm; they are
marked **owner's call**.

## Using it

1. In the CA-72 application, open SETTINGS (the gear on the rail) and choose
   **AGENTS › ALLOW**. Or start it with `ca72-standalone --agents`, which allows agents for that
   session only.
2. Tell the agent's MCP client to start the bridge, `ca72-mcp`, over standard input and output.
   In Claude Code, for example:

   ```sh
   claude mcp add ca-72 -- /path/to/ca72-mcp
   ```

   In a client configured with JSON, the entry is
   `{"command": "/path/to/ca72-mcp", "args": []}`.
3. Ask the agent to start with `ca72_describe`.

`ca72-mcp --instance ID` acts on one CA-72 when several are open; otherwise the agent chooses
with `ca72_select`. Building from source: `cargo build --release -p ca72-plugin --bin ca72-mcp`
(the bridge) and `scripts/standalone.sh` (the application). The installers do not ship
`ca72-mcp` yet.

## Design

### Transport: a bridge, and a socket in the application

An MCP client normally starts its server as a child process and talks to it over stdio. The
CA-72 is a window the player opens, so it cannot be that child. Instead it is split in two:

```
agent's MCP client ──stdio, MCP (JSON-RPC 2.0, a message a line)──▶ ca72-mcp (the bridge)
                                                                        │
             Unix socket in the user's own runtime folder, JSON-RPC 2.0 a line
                                                                        ▼
            CA-72 standalone: the socket's threads ──inbox──▶ the editor's window thread
```

- **The bridge** (`crates/ca72-plugin/src/agent/bridge.rs`, binary `ca72-mcp`) speaks MCP on
  stdio (`agent/mcp.rs`), with the protocol revisions 2025-11-25, 2025-06-18, 2025-03-26 and
  2024-11-05. It offers the tools (`agent/tools.rs`). It finds the CA-72s that are open
  (`agent/discovery.rs`) and passes each tool's call to the chosen one as a JSON-RPC request.
  The MCP layer copies the shape of the owner's DAW's server (DAW-v3,
  `crates/daw-api/src/mcp.rs`, DESIGN 7.6): `initialize` agreeing a revision, `tools/list`
  with schemas, and a failed `tools/call` answered as a result with `isError` rather than a
  JSON-RPC error, so that the agent reads why. As in DAW-v3, one table generates the tools'
  schemas from the API's methods (D165, item 1). The parameters' schema is generated from the
  parameters themselves (`agent/values.rs`), so it cannot fall out of step with them. As in
  DAW-v3's `daw rpc` relaying to a running `daw serve`, the bridge is a client of a
  long-running process.
- **The application** (`agent/server.rs`) listens on a Unix socket while agents are allowed.
  Each connection has a thread that reads requests, a line at a time. `hello` and `ping` are
  answered there. Every other request goes into an inbox, and the editor takes from the inbox
  at each frame (`PanelWindow::on_frame` → `Editing::serve_agents`, `editor/agent.rs`). The
  editor carries each request out on its own thread and answers it.
- Why not let the client launch the application? A client that starts the standalone itself
  would own its lifetime, which breaks the case of a player already working in the window. It
  would also need a second, headless mode of the application. The socket serves both cases.
  The bridge could later gain a `--launch` option that starts the application with
  `--agents` when none is open; this is not built.

### Access: off unless the player allows it (owner's call)

- **Off by default.** Until the player chooses SETTINGS › AGENTS › ALLOW (kept in this
  computer's `settings.toml` as `agents = true`) or starts the application with `--agents`,
  no socket exists, so nothing can connect.
- **Local and the user's alone.** Only a Unix socket is used, never a network port. It lives
  in `$XDG_RUNTIME_DIR/ca-72/` (on macOS, the user's own temporary folder), and that folder is
  made readable by its user alone (mode 700, the socket 600). So only the user's own programs
  can reach it. `CA72_AGENT_DIR` moves it, which the tests use.
- **An agent can turn agents off but not on.** `ca72_settings_set {"agents": false}` closes
  the socket after answering. Turning agents on again is the player's job.
- **Everything is visible.** Every change shows on the panel at once, as if the player had
  made it.
- Not done: a per-connection approval prompt, or a token the player copies to the client. The
  owner may want one of these. Recommendation: the setting plus the user-only socket is enough
  for a local, single-user instrument; add a prompt only if agents from other users' sessions
  become possible.

### Instances

- **Discovery.** Each listening CA-72 writes a record in the discovery folder,
  `<kind>-<pid>.json`, holding its id, pid, kind (`standalone`), version, protocol and socket
  path. The record is written whole (written aside, then renamed) and removed when the CA-72
  stops listening. The bridge lists the records and drops any whose socket no longer answers,
  so a crashed instance leaves nothing behind for long.
- **Selection.** Calls go to the only CA-72 that is open. With several open and none chosen, a
  call is refused (`AMBIGUOUS`) with their ids. `ca72_instances` lists them; `ca72_select` or
  `--instance` chooses one.
- **Plug-ins.** These are designed in, not built. A plug-in instance would write the same kind
  of record with `kind` `clap`, `vst3` or `au`, plus its host's name and pid, and its track or
  instance name if the host gives one. `ca72_instances` would then list every CA-72 on the
  computer, in the application and in hosts, and the agent would choose among them as it does
  now.

### Consistency: the editor's own paths, never the audio thread

Each request is carried out by the editor, on its window's thread, through the same code its
controls use:

| What | The path taken (`editor/agent.rs`) |
|---|---|
| A parameter set | `Agentic::set_once`: `ParamSetter` begin, set, end through the host's `GuiContext`, a gesture of its own, as a click or typed value sets it. In the standalone, `Session::set_parameter` sets the atomic and the audio thread updates its smoother. |
| DETUNE, VOICES | Refused where the panel won't turn them (`operable`/`voices_inert`'s rules), with the reason. |
| MODE | The strip's tabs' rule: MONO turns POLY and UNISON off, POLY turns POLY on and UNISON off, UNISON leaves POLY as it is. |
| A preset loaded, stepped | `Browser::choose` and `Browser::step`, as the drawer and the rail's arrows do. |
| Saved, renamed | `Browser::save_sound` and `Browser::rename_preset`, which SAVE AS and RENAME now share with the agent. |
| Tagged, starred, deleted, reverted, restored | The drawer's `Library` calls, followed by `Browser::reread`. The drawer's hint line says what was done, as it does for the player. |
| A setting | `set_look`, `set_lettering`, `set_frequency_dials`, `keep_settings`, and the SIZE menu's `gripped` width, which the host is asked for at the next frame. |
| The view | The rail's PANEL and KEYBOARD keys (`toggle_panel`, `strip_press`), the drawer's `Browser::show`, the keyboard's octave keys, and the drawer's MIDI list. |
| A note | `Playing::agent`: owned like the pointer's and the typed keys', through `LocalNotes`, the lock-free queue the on-screen keyboard already uses. The audio thread takes them at its next block. |
| PANIC | The keyboard's PANIC (`Editing::panic`): `AppHandle::panic_tagged` in the standalone. |
| MIDI Learn | `Learning::learn`, `cancel` and `remove`; `MidiMap::assign` for an assignment given outright. |

After any change, the editor's `touched` flag asks for AUTO GAIN's curve once nothing is held,
as it does after the player's edits.

The parameters' only route to the audio thread is the atomics the editor already sets. Notes
go through the queue the on-screen keyboard uses. The socket's threads never touch the plug-in
except through the inbox (a mutex the editor holds only to take one request). The editor takes
at most 32 requests a frame. Nothing on the audio thread waits on any of it.

### Errors

A failed call is a tool result with `isError: true`. Its structured content has `error` (the
reason, written for the agent), `code`, and `details` that help put it right: the parameter's
range and choices, the similar preset names, the list of instances. Codes
(`crate::agent::codes`):

| Code | Meaning |
|---|---|
| -32602 | Invalid arguments: an unknown parameter or preset, a value out of range or not one of the choices, a malformed note. Nothing in the call is set. |
| -32004 | Refused as the panel would refuse it: DETUNE in SCATTER, a name already taken without `replace`, a control MIDI Learn never takes, a reserved CC. |
| -32003 | No CA-72 to talk to: none allows agents, the chosen one has closed, or the connection was lost. |
| -32005 | Several are open and none is chosen. |
| -32001 | The window didn't take the request within 10 s. Nothing was done. |
| -32002 | Busy: eight agents are already connected. |

An unknown tool, or arguments that aren't an object, are JSON-RPC errors, as MCP specifies.

### Connection lifetime and disconnects

- The bridge connects at the first call that needs a CA-72 and keeps the connection.
- If the application quits, the next call is refused (`NO_INSTANCE`). If none was chosen and
  another CA-72 is open by then, for instance after a restart, the bridge connects to that one
  and the call goes through. So an agent's session outlives a restart.
- A request whose client goes away before the window has taken it is dropped, not carried out.
  The connection thread notices the close while it waits. A request the window has already
  taken is carried out, and its answer is discarded.
- A request the window doesn't take within 10 s is answered as timed out and never carried out
  afterwards. A request can be in one of three states (waiting, taken, dropped), and whichever
  side changes it first wins, so a request is never both refused and done.
- When agents are turned off, or the window closes, the inbox is closed. Waiting requests are
  answered at once, the threads end within 200 ms, and the socket and its record are removed.
- Each connection's requests are answered in order, one at a time. A line is at most 1 MiB.

### The wire between bridge and application

JSON-RPC 2.0, one object a line, over the socket. The methods are the tools' `method`s
(`describe`, `parameters.set`, ...), plus:

- `hello {name}`: names the client and answers `{name, id, kind, version, protocol, pid}`.
- `ping`.

The protocol number is 1. The bridge refuses an instance that speaks another, and tells the
agent to use the bridge that came with that instance.

## Tools

| Tool | Does | Arguments |
|---|---|---|
| `ca72_instances` | Lists the CA-72s open with agents allowed (bridge) | none |
| `ca72_select` | Chooses one (bridge) | `instance` |
| `ca72_describe` | Returns the whole state: version; every parameter with its label, section, value, printed value, range or choices, default, and why it's inert if it is; MODE; the preset and whether it has changed; the view; the settings; MIDI Learn; the notes held; audio | none |
| `ca72_get` | Reads parameters | `ids?` |
| `ca72_set` | Sets parameters (see below) | `values {id: value}` |
| `ca72_reset` | Sets controls to their defaults (a double click) | `ids` |
| `ca72_presets` | Lists presets, filtered as the drawer filters | `search? tags? favorites? mine?` |
| `ca72_preset_load` | Chooses a preset | `name` |
| `ca72_preset_step` | Steps to the next or previous preset (the rail's arrows) | `by` (1 or -1) |
| `ca72_preset_save` | SAVE AS | `name, tags?, replace?` |
| `ca72_preset_rename` | RENAME | `name, to` |
| `ca72_preset_tags` | TAGS | `name, tags` |
| `ca72_preset_favorite` | The star | `name, favorite` |
| `ca72_preset_delete` | DELETE (a factory preset is hidden) | `name` |
| `ca72_preset_revert` | REVERT | `name` |
| `ca72_presets_restore` | RESTORE | none |
| `ca72_settings` | Reads the settings and window sizes | none |
| `ca72_settings_set` | Changes settings as the SETTINGS menu does | `look? lettering? frequency_dials? quality_indicator? mod_wheel_source? mpe_input? timbre_rest? size? agents?` (`agents` only false) |
| `ca72_view` | Changes the view as the rail and keyboard keys do | `presets? panel? keyboard? keyboard_octave? midi_list?` |
| `ca72_note_on` / `ca72_note_off` | Presses or lets go of keys (no velocity: the instrument has none); `note_off` with no notes lets go of all the agent's | `notes` (numbers 0 to 127, or names, C4 = 60) |
| `ca72_play` | Plays notes for a while, then lets go | `notes, seconds` (0.01 to 30) |
| `ca72_panic` | PANIC | none |
| `ca72_midi_learn` | Learns, cancels, or assigns outright | `control?, cancel?, channel?, cc?, note?` (`note` for PREVIOUS and NEXT PRESET only) |
| `ca72_midi_assignments` | Lists MIDI Learn's assignments | none |
| `ca72_midi_forget` | REMOVE MIDI ASSIGNMENT | `control` |
| `ca72_audio` | Reads AUDIO & MIDI (read only) | none |
| `ca72_version` | Returns the version and the update check | `check?` |

### Values in `ca72_set`

A parameter is named by its id (as sessions and presets save it), its panel label, or its host
name, in any case. Values are in the units the panel prints:

- Knobs take their dial's numbers.
- FREQUENCY takes semitones from the centre, or a string such as `"+12 cents"`.
- ENTROPY, WIDTH and INNER take %, DETUNE takes cents, and DRIVE and LEVEL take dB.
- A string with its unit is accepted too (`"6 dB"`, `"50 %"`).
- A selector takes its position's name (`"16'"`, `"Sawtooth"`, `"Shark Tooth"`), its id
  (`"16"`, `"shark_tooth"`) or its index from 0.
- A switch takes `true`/`false` or `"on"`/`"off"`.

Every value is checked before anything is set. A value outside the control's range is refused,
not clamped (the error gives the range), and nothing in that call is set. The answer gives the
values as now set, which changed, MODE, and the preset's changed mark.

## Inventory: every user-facing control and its MCP equivalent

Sources: `ca72-panel` (`controls.rs`, `strip.rs`, `presets.rs`, `presets/drawer.rs`,
`keyboard.rs`, `learn.rs`, `ultra.rs`), and `ca72-plugin` (`params.rs`, `editor.rs`,
`learning.rs`, `presets.rs`, `library.rs`, `settings.rs`, `standalone_settings.rs`,
`update.rs`). Each parameter is listed with its range in the units printed. The MCP column is
the parameter's id in `ca72_set`/`ca72_get`, unless it says otherwise.

### The panel

| Section | Control (label) | Kind | Values (as printed) | MCP |
|---|---|---|---|---|
| CONTROLLERS | TUNE | knob | -2.5 to +2.5 (dial -2..+2; 0) | `tune` |
| | OSCILLATOR MODULATION | rocker | on/off (off) | `osc_mod` |
| | GLIDE | knob | 0 to 10 (0) | `glide` |
| | MODULATION MIX | knob | 0 (OSC. 3) to 10 (NOISE) (0) | `mod_mix` |
| OSCILLATOR BANK | OSCILLATOR-1/2/3 RANGE | selector | LO, 32', 16', 8', 4', 2' (8') | `osc1_range`, `osc2_range`, `osc3_range` |
| | OSCILLATOR-2/3 FREQUENCY | big knob | semitones from the centre: oscillator 2 -8.56 to +8.24, oscillator 3 -8.63 to +8.22 (0). Printed "+7.00 semitones" or "+3.4 cents"; the dial reads in SETTINGS › FREQUENCY DIALS' law | `osc2_frequency`, `osc3_frequency` |
| | OSCILLATOR-1/2 WAVEFORM | selector | Triangle, Shark Tooth, Sawtooth, Square, Wide Rectangle, Narrow Rectangle (Sawtooth) | `osc1_waveform`, `osc2_waveform` |
| | OSCILLATOR-3 WAVEFORM | selector | Triangle, Reverse Sawtooth, Sawtooth, Square, Wide Rectangle, Narrow Rectangle (Sawtooth) | `osc3_waveform` |
| | OSC. 3 CONTROL | rocker | on/off (on) | `osc3_control` |
| MIXER | OSCILLATOR-1/2/3 VOLUME, EXTERNAL INPUT VOLUME, NOISE VOLUME | knobs | 0 to 10 (8; EXTERNAL INPUT 5) | `osc1_volume`, `osc2_volume`, `osc3_volume`, `ext_volume`, `noise_volume` |
| | each one's ON | rockers | on/off (oscillator 1 on, the rest off) | `osc1_on`, `osc2_on`, `osc3_on`, `ext_on`, `noise_on` |
| | NOISE (WHITE or PINK) | rocker | White, Pink (White) | `noise_type` |
| MODIFIERS | FILTER MODE (LO or HI) | rocker | LO, HI (LO) | `filter_mode` |
| | FILTER MODULATION | rocker | on/off (off) | `filter_mod` |
| | KEYBOARD CONTROL 1 and 2 | rockers | on/off (1 on, 2 off) | `keyboard_control_1`, `keyboard_control_2` |
| | CUTOFF FREQUENCY | knob | -5 to +5 (0) | `cutoff` |
| | EMPHASIS | knob | 0 to 10 (0) | `emphasis` |
| | AMOUNT OF CONTOUR | knob | 0 to 10 (5) | `contour_amount` |
| | FILTER CONTOUR ATTACK, DECAY TIME; SUSTAIN LEVEL | knobs | 0 to 10 (0, 4, 5) | `filter_attack`, `filter_decay`, `filter_sustain` |
| | LOUDNESS CONTOUR ATTACK, DECAY TIME; SUSTAIN LEVEL | knobs | 0 to 10 (0, 4, 8) | `loudness_attack`, `loudness_decay`, `loudness_sustain` |
| OUTPUT | MAIN OUTPUT VOLUME | knob | 0 to 10 (10) | `volume` |
| | MAIN OUTPUT, A-440 | rockers | on/off (on, off) | `main_output`, `a440` |
| | FEEDBACK (the phones' VOLUME into EXTERNAL INPUT) | knob | 0 to 10 (0) | `feedback` |
| | QUALITY | 3-way toggle | LO, HI, ULTRA (LO) | `quality` |
| | POWER | switch | on/off (on) | `bypass` (true = off) |
| | QUALITY's menu: SHOW / HIDE ITS INDICATOR | menu | shown/hidden | `ca72_settings_set {quality_indicator}` |
| | The OVERLOAD lamp | indicator | | not exposed (see below) |

### The left hand controller and the strip

| Section | Control | Kind | Values | MCP |
|---|---|---|---|---|
| LEFT HAND | GLIDE, DECAY | rockers | on/off (off, on) | `glide_on`, `decay_on` |
| | NOTE PRIORITY | 3-way toggle | LOW, LAST, HIGH (LOW) | `note_priority` |
| | RETRIGGER | rocker | on/off (off) | `retrigger` |
| | PITCH wheel | wheel | -1 to +1, 0 in its detent (0) | `pitch_wheel`. It stays where the agent sets it; the panel settles it into its detent near 0. Not kept in presets or the standalone's session. |
| | MODULATION wheel | wheel | 0 to 1 (0) | `mod_wheel` |
| STRIP: VOICES | MODE: MONO, POLY, UNISON | tabs | | `ca72_set {mode}`; or `poly`, `unison` |
| | VOICES | stepped knob, readout | 2 to 10 (4); inert in MONO with DOUBLE or WIDTH 0 | `voices` (refused while inert) |
| | ENTROPY | knob, readout switch | 0 to 100 % (0) | `entropy` |
| STRIP: STEREO | VOICES PLAYED AS: SCATTER, DOUBLE | tabs | (SCATTER) | `doubled` |
| | PLACEMENT: EVEN, CENTER | tabs | (EVEN) | `placement` |
| | WIDTH | knob, readout switch | 0 to 100 % (0) | `spread` |
| | INNER | knob, readout switch | 0 to 100 % (0) | `inner` |
| | DETUNE | knob, readout | 0 to 100 cents; "Off" at 0 (0); only in DOUBLE | `double` (refused in SCATTER) |
| STRIP: OUTPUT | DRIVE | knob, readout | 0 to 24 dB (0) | `drive` |
| | AUTO GAIN | tab | on/off (on); its readout shows the dB taken back | `auto_gain` |
| | LEVEL | knob, readout | -30 to +12 dB (0) | `level` |
| | The readout switches (ENTROPY, WIDTH, INNER: off and back on at the last amount) | switch | | Set the amount itself; 0 is off. The switch's memory of the last amount is the panel's alone. |
| | Typing a value into a readout | text | | `ca72_set` (the agent's values are range checked; the panel's typed ones are clamped) |
| | WHERE THE VOICES SOUND | display | | not exposed (see below) |
| Not on the panel | LOCK (oscillators identical) | switch | on/off (off); in the MIDI list only | `lock` |
| | MIDI BEND RANGE | host parameter | 0 to 8.07 semitones (2) | `midi_bend_range` |

### The rail, the presets' drawer and the update check

| Control | MCP |
|---|---|
| Star key | `ca72_preset_favorite` (for the current preset: its name from `ca72_describe`) |
| PREV / NEXT keys | `ca72_preset_step {by: -1 / 1}` |
| The name: opens or shuts the drawer | `ca72_view {presets}` |
| SAVE key, SAVE AS | `ca72_preset_save` |
| PANEL key | `ca72_view {panel}` |
| KEYBOARD key | `ca72_view {keyboard}` |
| SETTINGS key: see Settings | |
| FIND: SEARCH, tag chips | `ca72_presets {search, tags}` |
| ALL / FAVORITES / MINE tabs | `ca72_presets {favorites, mine}` |
| A row: click chooses it; double click chooses it and closes the drawer | `ca72_preset_load` |
| A row's star | `ca72_preset_favorite` |
| RENAME, TAGS, DELETE (asks first), REVERT, RESTORE | `ca72_preset_rename`, `_tags`, `_delete` (no second press: the tool is marked destructive for the client to confirm), `_revert`, `ca72_presets_restore` |
| MIDI LEARN (the drawer's list) | `ca72_view {midi_list}`; `ca72_midi_assignments` |
| CLOSE | `ca72_view {presets: false}` |
| The status line, the hint | `ca72_describe` (the preset); every preset tool's answer |
| CHECK FOR UPDATES | `ca72_version {check: true}`. DOWNLOAD and RELEASES PAGE open the browser, which an agent cannot do. |
| Init preset | There is none in this version. A future Init (R-INIT, another branch) will be a preset like any other, loaded with `ca72_preset_load`. |

### Settings

| Control | MCP |
|---|---|
| LOOK: REALISTIC, CLASSIC | `ca72_settings_set {look}` |
| LETTERING: STANDARD, LARGE | `ca72_settings_set {lettering}` |
| FREQUENCY DIALS: SEMITONES, HARDWARE | `ca72_settings_set {frequency_dials}` (the sound stays as it is) |
| MOD WHEEL SOURCE (R-MODWHEEL-SOURCE) | `ca72_settings_set {mod_wheel_source}`: `"CC 11"` (any CC but the reserved), `"CHANNEL AFTERTOUCH"` or `"POLYPHONIC AFTERTOUCH"`. Learning it from a controller (the wheel's MIDI LEARN) is not offered. |
| MPE: INPUT & TIMBRE… › MPE INPUT: AUTO, LOWER ZONE, OFF (R-MPE) | `ca72_settings_set {mpe_input}` |
| MPE: INPUT & TIMBRE… › TIMBRE REST: CENTERED ON 64, FROM 0, FROM THE NOTE'S START | `ca72_settings_set {timbre_rest}` |
| WINDOW: SIZE… (75 % to 200 %, FIT TO SCREEN) | `ca72_settings_set {size}`: one of the widths `ca72_settings` lists, or `"FIT"` |
| The resize grip | the same, at the SIZE menu's widths (any other width is not offered) |
| STANDALONE: AUDIO & MIDI… | `ca72_audio` reads it. Changing devices is not offered yet (see below). |
| AGENTS: ALLOW, OFF (new) | `ca72_settings_set {agents: false}` only |

### The on-screen keyboard and MIDI Learn

| Control | MCP |
|---|---|
| Keys (44, F2 to C6 by default) | `ca72_note_on`, `ca72_note_off`, `ca72_play`. Any note 0 to 127, not only the 44 shown. |
| OCTAVE - / + | `ca72_view {keyboard_octave}` |
| TYPING (the computer's keyboard) | not exposed: it's the player's own keyboard, and `ca72_note_on` does more |
| PANIC | `ca72_panic` |
| A control's menu: MIDI LEARN, CANCEL MIDI LEARN | `ca72_midi_learn {control}`, `{cancel: true}` |
| REMOVE MIDI ASSIGNMENT | `ca72_midi_forget` |
| MIDI ASSIGNMENTS… | `ca72_midi_assignments`, `ca72_view {midi_list: true}` |
| (new) assigning a controller outright | `ca72_midi_learn {control, channel, cc}` |
| PREVIOUS PRESET, NEXT PRESET (R-PRESET-KEYS; kept on this computer) | `ca72_midi_learn {control: "previous_preset"}` (a note or a CC), `{control, channel, note}` or `{control, channel, cc}`; `ca72_midi_forget`. The arrow keys are the player's keyboard, as TYPING is: `ca72_preset_step` does the same. |

### Left out, and why

- **Visual only.** Hover, tooltips, animation (QUALITY's shutter, the hamster, the lamps),
  dimming, caret and focus, and decoration (the PHONES jack, the name plate). These change no
  state.
- **Indicators.** OVERLOAD, the voices' display, and AUTO GAIN's readout are left out. Their
  values move with the audio. `ca72_describe` gives the voices sounding and the keys held.
  OVERLOAD and the levels could be added for #77 if wanted.
- **Gestures.** A drag, the mouse wheel and Shift's fine control are ways to reach a value;
  `ca72_set` reaches it directly. A double click is `ca72_reset`.
- **ULTRA's note.** It is shown once and closed by a press; it's the player's to read.
- **TYPING.** Covered by `ca72_note_on` (see above).
- **Opening the browser** (DOWNLOAD, RELEASES PAGE). An agent can read whether there is an
  update, but it can't download one.
- **AUDIO & MIDI changes** (device, rate, buffer, MIDI inputs and channel). Not offered in this
  version. They go through `AppHandle::apply_tracked` after `negotiate`, as the dialog does,
  and can stop the sound. Recommendation: offer them in 0.3.0's second step, after a
  confirmation design. **Owner's call.**
- **Rendering and listening** (#77) are not part of this interface (the issue says so). See
  below.

## Next: plug-ins in a host

What is needed, in order:

1. **A server without the editor.** In a host the editor is often closed, so requests can't
   wait for its frames. The parameter paths would move to the plug-in, keeping the editor's
   when it is open:
   - parameters through the host's context: CLAP's output events and VST3's
     `outputParameterChanges`, in the next process call, as MIDI Learn's
     `ProcessContext::set_parameter_normalized` does (nih-plug patch 10). This touches the
     audio thread, through a bounded lock-free queue only; or through the GUI context while the
     editor is open.
   - presets through `Library` and `presets::targets`.
   - notes through `LocalNotes`.

   Whatever the editor alone does would be refused while it is closed: the view, the look,
   window size, and MIDI Learn's waiting.
2. **Instance identity.** The record gets the format, the host's name and pid, and a name the
   player recognises: the host's track name where CLAP's `track-info` or VST3's
   `IInfoListener` gives it, otherwise the preset's name and an index.
3. **Many instances in one process.** Several plug-ins share a process, so one socket per
   instance is fine, but the discovery folder will have many records. `ca72_instances` should
   show the host and track names so the agent can choose.
4. **Access in a host.** The same setting applies. A host's sandbox may refuse the runtime
   folder: Logic's AU runs in its own process (AUHostingService), and Bitwig's plug-in host is
   separate too. This needs testing in each host.
5. **Windows.** A named pipe in place of the Unix socket, or AF_UNIX, which Windows 10 and 11
   support but Rust's standard library does not.
6. **Shipping the bridge.** The installers and `scripts/standalone.sh` should ship `ca72-mcp`
   beside the application, with a line in the README.
7. **For #77.** A tool that renders a preset or the current sound offline to a WAV, which the
   CA-72's own `preset_render` already does in tests. Its home (the standalone, or a headless
   `ca72-mcp render`) is to be decided with #77.

## Tests

- `cargo test -p ca72-plugin --lib agent` covers the following:
  - **The dispatcher and schemas** (`agent::mcp`, `agent::tools`): the handshake and revision,
    the tool list, calls and their failures, unknown tools and methods, bad JSON, batches
    refused, an over-long line; every tool's schema well formed; `ca72_set`'s schema naming
    every parameter.
  - **The values** (`agent::values`): every host parameter in the table once; the panel's
    units; parsing and range checks.
  - **Discovery**: records, the dead ones removed, the folder made private.
  - **The server**: hello, ping and methods answered in order; unknown methods; bad JSON;
    socket permissions; a request not taken is dropped; a client gone mid-request; at most
    eight clients; stopping answers what waits.
  - **The bridge**: none open, one, two, choosing, the chosen one closing, reconnecting to
    another after the first one stops.
- **Through the bridge to a real editor** (`editor::agent::tests`): an editor (the window's
  `Editing`, with a host that applies its changes as the standalone's does) behind a real socket,
  driven by `ca72-mcp`'s code in MCP's text through pipes. These tests cover parameters,
  presets, notes, settings, the view, MIDI Learn, agents turned off, and a client gone
  mid-request.
- **End to end**: the real `ca72-standalone --no-devices --agents` on Linux, driven through
  the built `ca72-mcp` by a JSON-RPC script, and by a real MCP client (decisions.md R-MCP has
  what was run).
