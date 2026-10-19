# Effects in the standalone application (issue 82)

Written 2026-10-18, on branch `fx-host`, as decisions.md R-FX-HOST. The design, what the
Linux prototype does and how it was tried, what Windows and macOS still need, and the
questions for the owner.

## What the owner decided (2026-10-18)

- **The standalone application only**, for 0.3.0: the DAW plug-ins never host anything.
- **Effects only**, not instruments.
- **VST3 and CLAP** both (the issue said VST3; the code reused hosts both). AU later.
- **A chain of four**: add, remove, reorder, bypass; each effect opens its own editor
  window; the chain and each effect's state are saved with the standalone's session.
- **Its view, "FX-A"** (chosen from the coordinator's mock-ups after this work began): in
  the standalone only, an FX key on the walnut rail after the ARP key; pressed, it drops a
  one-row drawer inside the window, about 170 panel units deep, over the lower page, like
  the preset and arpeggiator drawers. Four slots side by side, each a dot-matrix name screen
  (the rail's pixel screen, about 400 units wide, 12 characters) with an ON tab (bypass)
  beside it. A click on a name opens the effect's window; a drag reorders; a right-click
  offers replace and remove; an empty slot reads "+ ADD EFFECT" and opens the effect list.
  The key stays lit while any effect is active. Not built tonight: the drawer is to share
  the arpeggiator's drawer mechanics (branch `arpeggiator`), and the engine came first. Until
  it is built, a plain console drives the chain (below).

## Architecture

```
 audio thread (standalone)                          helper processes, one per effect
 ┌──────────────────────────────────────┐          ┌───────────────────────────────┐
 │ Ca72::process: voices → meters →     │  shared  │ ca72-fx-helper (DAW-v3's      │
 │   ca72_fx::Runner ── node 1 ─────────┼─memory───┼─▶ daw-helper): the plug-in,    │
 │                  ── node 2 ─────────┼──────────┼─▶ its audio thread, its editor │
 │                  ── ...  (≤ 4)      │  futex   │   window, its state           │
 └──────────────────────────────────────┘ wake-ups └───────────────────────────────┘
        ▲ queue (lock-free)          control: JSON lines on the helper's stdin/stdout
 ┌──────┴───────────────────────────────┐          ▲
 │ ca72_fx::Chain (control threads):    │──────────┘
 │  catalog, add/remove/move/bypass,    │  supervisor: respawn, restore, watchdog,
 │  editors, save/restore effects.json  │  snapshots (monitor thread)
 └──────────────────────────────────────┘
```

**Reuse of DAW-v3.** The owner's DAW already hosts CLAP and VST3 plug-ins out of process
(its DESIGN 10, D19, D25, D63, D78). Its crates were looked at in turn:

- `daw-ipc` (transport: named shared memory, futex/`os_sync_wait_on_address`/named-event
  wake-ups, the block layout, the JSON-lines control protocol) and `daw-helper` (the helper
  process: CLAP and VST3 hosting, editors in windows it owns, scanning) do not depend on the
  DAW. They are **vendored unmodified** but for one patch, in `third_party/daw-v3`
  (VENDORED.md, PATCHES.md), with DAW-v3's two test plug-ins. They are workspace members, so
  their own tests run in the CA-72's CI. Their manifests use workspace inheritance; the
  CA-72's workspace lints and toolchain are DAW-v3's, so they build as they are.
- `daw-sandbox` (the host side) depends on DAW-v3's engine (`daw-engine::Device`) and
  format. Its supervisor, helper-process wrapper and engine node are **adapted** into
  `crates/ca72-fx`, each file saying what it takes: effects only (no notes, modulation, note
  names), one helper per effect, and two additions (an effect unloaded; an effect loaded
  again at another sample rate in the same helper).
- Copying rather than a git dependency: DAW-v3 is private (CI and the public release's
  source tarball could not fetch it) and its workspace pins far more than this needs.
  Upstream fixes are taken by hand (VENDORED.md says how); the one patch here is to be
  offered back.

**Crates.**

- `ca72-fx` (library, in the standalone): `catalog` (finding effects), `chain` (the control
  side `Chain` and the audio side `Runner`, the saved format), `sandbox` (supervision),
  `node` (one effect on the audio thread), `console` (the temporary interface), `locate`.
  It links no plug-in SDK and does not depend on the synth, so it can move to plugin-kit for
  the other instruments (a question below).
- `ca72-fx-helper` (binary): `daw_helper::run()`, nothing else. Plug-in code, the `vst3`
  and `clap-sys` bindings and `libloading` are linked only here. Its tests (the chain's, with
  the test plug-ins) live in this package, because they need `CARGO_BIN_EXE_ca72-fx-helper`.
- `ca72-plugin`: feature `fx-host` (on with `standalone-production` only), `src/fx.rs` (the
  hand-off of the `Runner`), a field in `Ca72`, two calls (in `initialize` and at the end of
  `process`). The plug-in bundles build without it.

**Process model: one helper per effect.** DAW-v3 measured one per instance, one per plug-in
and all in one the same within noise (its D25); with four effects at most, one each costs
four small processes and gives the best isolation: a crash takes one effect, and only its
state needs restoring. Loading the same plug-in twice gives two helpers.

## Audio routing

The chain runs at the end of `Ca72::process`, after the voices, POWER's fade and the
editor's meters (so the meters still show the synth), on the synth's stereo output in
place, before the device. How an effect with mono ports is fed is DAW-v3's helper's (not
tried here). The side-chain input
(the external input the synth already takes) is not given to effects.

**Synchronous by default.** Each effect's block goes to its helper and is waited for, a
short spin then a futex wait, until a deadline: 0.9 of the block's period from the start of
`process` (`fx::DEADLINE`; the voices have until 0.75, `engine::DEADLINE`). A block that is
not back by then is lost: that effect passes its input on (dry) for that block, a miss is
counted, and no new block is sent to it until the late one has come back. So a slow effect
costs its own sound and never the device's deadline. Effects run one after another, so the
chain's cost adds up within the one period: a heavy chain at small blocks will miss.

**Pipelined** (`--fx-mode pipelined`, DAW-v3's second mode) runs each effect a block
behind: it processes block *k* while the synth plays *k + 1*, so an effect has a whole
period and runs beside the synth. It adds one block of latency per effect.

DAW-v3 chose pipelined for effects in its mixer (D25). For an instrument played live,
synchronous adds no latency, so it is the default here; whether to offer the choice to the
player is a question below.

## Latency

A standalone has no host to compensate latency, and the chain is one path (each effect's
dry/wet is the effect's own), so there is nothing to align: latency only makes the sound
later behind the keys. The chain reports it: `Chain::latency()` (frames: each running,
unbypassed effect's latency as it reported at activation, plus a block for each pipelined
one), shown by the console with milliseconds. The view should show it when it is not zero
(a question below). nih-plug's `set_latency_samples` means nothing in the standalone and is
not used.

Bypass takes an effect out of the path, latency and all: the sound moves earlier by its
latency when bypassed, as in most hosts. A plug-in that changes its latency while running
is not followed (DAW-v3's D19 limitation): the new latency applies when it is next loaded
(a sample-rate change, the next session).

## Sample rate and block size

Effects are loaded at a provisional 48 kHz before an audio device plays (so that a chain
restored at start-up, or built with no device, has live effects and editors), always with
the transport's largest block, 4096 frames. When the device starts or changes,
`Ca72::initialize` (on a control thread, the stream stopped) calls `Runner::prepare(rate,
block)`: at another rate each effect is loaded again in its helper (a new instance with the
old one's latest state and values, then the old one unloaded; the helper stays), and every
node is made again for the block (the pipelined chunk). A block larger than 4096 frames is
given to the effects in pieces. If an effect does not load at the new rate it becomes
missing (dry, its state kept).

Tried: the test effects at 48 kHz then 44.1 kHz and 512 frames, same helper, same state,
same output; the real standalone on a 44.1 kHz device with the chain restored at 48 kHz.

## Crash and hang recovery

From DAW-v3 (DESIGN 10.4, D19), as it was:

- The helper's stdout closing is its death, seen at once by its reader thread; its effect
  is marked down (dry) and its transport dead, so a waiting block returns at once.
- A new helper is started, the effect loaded with its last state snapshot and the values the
  host gave it, and activated; the node takes it up at its next block.
- A block outstanding longer than the hang timeout (2 s) makes the monitor thread kill the
  helper: then as a crash.
- After 5 restarts the effect is given up on: dry from then on, reported `failed`.
- Snapshots: at load, 500 ms after a change the host made, when the helper says the
  plug-in changed itself (its editor's edits, a preset it loaded: DAW-v3's notices), and
  every 60 s.
- Events (`crashed`, `hung`, `restored`, `failed`, `warning`) are kept and logged to stderr
  ("CA-72 effects: ..."); the console shows them; the view is to show them on the slot.

Two changes, both found by the tests: the effect is marked down before its process is
reaped (a crashed process can take a while to end: systemd-coredump writes its core first,
and meanwhile every block waited out its deadline); and after a restore the node drops the
changes it was holding and the values of hidden parameters are not given back (the test
plug-ins' `crash` and `hang` would otherwise take the restored effect down again; a change
made in the instant of a crash may be lost).

An editor window belongs to its helper and goes with it; it is not opened again after a
restore (the player opens it again).

## Finding effects; the cache

Folders searched: `$CA72_FX_PATH` (a path list; for tests, or effects kept elsewhere), then,
unless `CA72_FX_STANDARD_FOLDERS=0`, the standard ones: Linux `~/.clap`, `~/.vst3`,
`/usr/lib/clap`, `/usr/local/lib/clap`, `/usr/lib/vst3`, `/usr/local/lib/vst3`; macOS
`~/Library/Audio/Plug-Ins/{CLAP,VST3}` and `/Library/...`; Windows
`%LOCALAPPDATA%\Programs\Common\{CLAP,VST3}` and `%COMMONPROGRAMFILES%\{CLAP,VST3}`.
`CLAP_PATH` and `VST3_PATH` are not read: they are the player's hosts' business.

Each `.clap` or `.vst3` file or bundle (folders looked into three deep, bundles never) is
listed by a helper of its own (`ca72-fx-helper --scan`, killed after 10 s): a plug-in that
crashes or hangs when loaded costs its own line. The listings are kept in
`effects-cache.json` in the session folder, by path, size and modification time (a bundle's
newest file), so a plug-in is loaded to be listed only when new or changed. Effects are told
from instruments by their categories (CLAP `audio-effect`/`instrument`, VST3 `Fx`,
`Instrument`, `Synth`); one that says neither is listed and checked when loaded, where an
instrument (by its categories, or no audio input) is refused.

Scanning loads every plug-in installed, which the owner's machines have many of (DAW-v3's
D79 found fifty on the Mac). Nothing scans unless asked (the console's `scan`; the view's
"+ ADD EFFECT"). The tests never search the standard folders.

## Saved state

`effects.json` beside the standalone's `session.json` in its session folder (so
`--state-dir` keeps it apart too), written through a temporary file and a rename after every
change to the chain, at most once a second when an effect's state changes, and at exit:

```json
{ "format": 1,
  "slots": [ { "format": "clap", "path": "/…/Reverb.clap", "plugin": "com.example.reverb",
               "name": "Reverb", "vendor": "Example", "bypass": false, "state": "<hex>" } ] }
```

Not in `session.json`'s plug-in state: that is the synth's nih-plug state, serialised as a
JSON array of byte values inside an 8 MB limit, and effects' states can be large (an
impulse response, a sampled room); and it would put the chain in the plug-ins' state and
presets. A slot whose plug-in cannot be loaded (uninstalled, moved, failing) is kept,
missing and dry, its state saved again as it was, so a session opened on a computer without
it loses nothing. A file that cannot be read is left as it is: the chain starts empty and
nothing is written over it that session. States are hex (twice their size); base64 or files
of their own would do if states prove large.

Not covered: presets. A sound preset does not carry the chain, and loading one does not
change it (a question below).

## Real-time safety

On the audio thread (`Runner::process` and the nodes):

- no allocation, no locks: the chain's changes arrive on a lock-free queue (`rtrb`) of
  preallocated messages; a removed effect's node goes back on another queue and is dropped
  on a control thread; every buffer is sized when its node is made;
- atomics for bypass, status, latency and counters;
- system calls: a futex wake per effect per block (`submit`), and a futex wait bounded by the
  deadline when the helper has not answered within a 50 µs spin (`wait_done`), and
  `Instant::now()`. These are the design (DAW-v3 measured the round trip at 1 to 2 µs median);
- no waiting past the deadline, ever; a late, dead or missing effect is dry at once.

The helpers' audio threads ask for real-time scheduling (DAW-v3's `rt::promote`); the
synth's own threads are as before. `crates/ca72-fx-helper/tests/no_alloc.rs` counts every
allocation and free on the playing thread (a counting global allocator) over 400 paced
blocks through two effects while the chain is changed under them (a parameter, a bypass, a move, a helper killed and restored, an
effect taken out): none (and 800 when one small `Vec` a block is slipped in, to prove the
count). The chain's cost to the audio thread (`the_chains_cost`, `--ignored`, release, this
computer, a Ryzen 7 7800X3D, load about 3, the helpers without real-time priority, 128
frames at 48 kHz, paced, 10 s): one test effect, a median of 7.6 µs a block, 14 µs at the
99th percentile, 0.38 ms at worst; four, 28 µs, 42 µs and 1.3 ms (half the 2.7 ms period);
no block missed. So the round trips are cheap and the tail is the scheduler's. Not measured
yet: the slow-machine matrix (the CPU rule in the circuit-plugin playbook), heavy effects,
and the Mac and Windows.

## The temporary interface

`--fx-console`: the standalone reads commands on standard input (`help`, `scan`, `add N`,
`add-file PATH [ID]`, `replace S N`, `remove S`, `move S T`, `bypass S on|off`, `editor S
[close]`, `param S ID VALUE`, `params S`, `chain`, `events`, `kill S`, `save`); `--fx-mode
sync|pipelined`; `CA72_FX_PATH`, `CA72_FX_STANDARD_FOLDERS`, `CA72_FX_HELPER`, `CA72_FX_LOG`
(the helpers' stderr). It drives only `Chain`'s public calls, which the FX-A drawer will
call (`add`, `replace`, `remove`, `move_to`, `set_bypass`, `open_editor`, `status`,
`active` for the key's light, `latency`, `events_from`, and `catalog::scan` for the list,
best on a thread of its own as it may take seconds); on Windows the release build has no console, so it is a Linux and macOS testing aid.

## Tests and what was tried (Linux, omarchy, 2026-10-18)

`cargo test -p ca72-fx -p ca72-fx-helper -p daw-ipc -p daw-helper` (the vendored crates'
own 48 tests pass here), `crates/ca72-fx-helper/tests/chain.rs`, with DAW-v3's test
plug-ins in both formats (`y = gain · x[n − latency] + …`, exact in `f32`):

- the CLAP and VST3 test effects found, the VST3 module's instrument left out, a broken
  file reported, the second scan entirely from the cache; an instrument refused even when
  the list did not say;
- the output exact at unity and at gain 0.5; **bypass identical to the bit**; a latency of
  100 frames taken through the state into a restored chain, reported as 100 and the output
  exactly 100 frames late; a pipelined effect exactly one block late and reported so;
- order, bypass, save to `effects.json`, restore in a new chain with the same sound; a
  missing plug-in kept dry with its state; a damaged file left as it was;
- a sample-rate change: same helper, same state, same output;
- in real time, paced, 30 ms deadlines: **a crash, a hang and a kill** of each format's
  effect at block 40: every block either the effect's or dry, dry while it was down, the
  effect's again after, no block held past its deadline by more than 20 ms, `crashed`,
  `hung` and `restored` reported, one crash counted; an effect killed again and again given
  up on (`failed`) and dry;
- run three times in a row, all pass (a machine load of about 20 meanwhile).

The real standalone (debug build, `--state-dir` and `IDLE_FOUNDRY_SHARED` in the worktree,
`CA72_FX_STANDARD_FOLDERS=0`): with `--no-devices` the console scanned, added both test
effects, bypassed, opened and closed an editor (on screen, with the plug-in's view inside),
saved; then with the default audio device (44.1 kHz) the chain came back from the file,
was loaded again at the device's rate, blocks flowed, and a killed helper and a crashed one
were restored while the synth played on.

**Third-party effects**: none are installed on this computer (only the owner's own
instruments, not loaded). Dragonfly Reverb 3.2.10 (GPL-3, from its GitHub release, kept
under `target/`, never installed): its four effects in both formats (eight) listed, loaded,
played a second of noise with finite, changed output, and restored from their states into
fresh chains with the same state back (`CA72_FX_THIRD_PARTY=… cargo test -p ca72-fx-helper
--test chain -- third_party`). Their CLAP editors open and draw. Their VST3 editors did not
attach until the run-loop patch (PATCHES.md); now they open and draw, but show their knobs
at zero (the controller has the right values): not solved.

Not tried: an editor with `CA72_FX_GUI_TEST` in CI (it opens windows); effects with
sidechains or mono ports; MIDI to effects; heavy effects against the deadline; a long run.

## Windows and macOS

Built and tried mostly on Linux tonight. Cheap checks elsewhere (fresh clones in temporary
folders, removed after): the chain's nine tests (crash, hang and kill in real time
included) and `daw-ipc`'s pass on the Mac (macOS 27, Apple silicon, built for macOS 11) and
on Windows 11 (MSVC 1.97.1, over SSH, so without the editor test); clippy clean on the Mac.
The standalone itself was not built on either. What each needs:

- **Both:** `scripts/standalone.sh` now builds `ca72-fx-helper` and puts it beside the app
  (`target/bundled/ca72-fx-helper[.exe]`; macOS `CA-72.app/Contents/Helpers/`, where
  `locate::helper_path` looks). Not done: `scripts/package.sh` (the ZIP, the Linux tarball,
  the macOS `.pkg`) and the installers must take the helper, and
  `scripts/test-standalone-install.py` check it. DAW-v3's transport and helper run on both
  in DAW-v3's CI (its D78: the window tests pass on the Mac and the Windows desktop), and the
  chain's tests will run in the CA-72's CI with the workspace.
- **macOS:** the helper must be signed and notarised with the app, with the hardened runtime
  and **`com.apple.security.cs.disable-library-validation`** (it loads other vendors'
  libraries) and perhaps `allow-unsigned-executable-memory` for some plug-ins; the
  application itself needs neither, which is the point of the helper. A helper with no Dock
  icon: DAW-v3 makes it an accessory application. The transport's wait functions,
  `os_sync_wait_on_address` and its kin, are macOS 14.4's, and DAW-v3 linked them, so the
  application (minimum macOS 11.0) would not have started before 14.4 once it linked
  `daw-ipc`: found tonight (`nm -m` on the Mac) and fixed (PATCHES.md, 2: looked up at run
  time, a polled wait without them). Not tried on a system before 14.4; whether effects
  should run there at all is a question. DAW-v3's D78 notes
  the module entry gets no bundle reference yet on macOS, which some plug-ins need. Editors
  float (no reparenting across processes on macOS).
- **Windows:** the helper is started without a console (`CREATE_NO_WINDOW`); named events
  and a pagefile-backed mapping for the transport; Win32 windows for editors; waits have
  millisecond granularity. The console interface is not available in the release build (no
  console). Windows Defender/SmartScreen will see a second unsigned executable.

## Licences and what distribution asks

The CA-72 is GPL-3.0-or-later; so is DAW-v3, the owner's (no new licence, but its files
keep their provenance in VENDORED.md and each adapted file's header). New crates, all
compatible with the GPL:

| Crate | Licence | Where linked | Obligation |
|---|---|---|---|
| `vst3` 0.3.0 | MIT OR Apache-2.0 (used under MIT) | helper | its notice; its bindings are generated from Steinberg's VST 3 SDK interfaces, MIT since SDK 3.8, whose notice is added too |
| `com-scrape-types` 0.1.1 | MIT OR Apache-2.0 | helper | notice (it ships no licence file; the standard text is used) |
| `clap-sys` 0.5.0 | MIT OR Apache-2.0 | helper | notice |
| `libloading` 0.8.9 | ISC | helper | notice |
| `rtrb` 0.2 (already in) | MIT OR Apache-2.0 | standalone | as before |

`scripts/notices.py` now walks the helper's dependencies too and adds Steinberg's notice
(THIRD-PARTY-NOTICES.txt regenerated; `scripts/test-notices.py` passes). Hosting other
people's plug-ins at run time adds no obligation: they are separate programs in a separate
process, under their own licences. The VST 3 SDK's MIT licence does not license the "VST"
name or logo: saying "VST3" on screen falls under Steinberg's trademark guidelines, as the
CA-72's own VST3 plug-in already does (a question, given the playbook's rule of no
trademarks in the product). The test plug-ins are never shipped.

## Questions for the owner

1. **Synchronous or pipelined by default**, and should the player choose? Synchronous adds
   no latency but every effect must finish within the block; pipelined is safer with heavy
   effects and adds a block per effect (5.3 ms at 256 frames, 48 kHz). Recommendation:
   synchronous, with a per-chain choice in the settings later if players hit misses.
2. **Where the chain lives in presets.** Kept out of sound presets (as now, and as the
   player's controls are), or saved with a preset as an option? Recommendation: out; the
   chain is the instrument's rig, like the audio device.
3. **The latency and the failures on screen.** Show the chain's latency in the drawer when
   not zero, and a slot's crash/failure on its name screen (e.g. "CRASHED" blinking, then
   the name)? Recommendation: yes to both.
4. **Move `ca72-fx` to plugin-kit** so the CA-74, MC-79 and the 808 standalones get it?
   Recommendation: yes, after FX-A is built and 0.3.0 ships, as the standalone host did.
5. **macOS older than 14.4** (the app supports 11.0): the transport now starts there with
   a polled wait (a wake-up up to 50 µs late, a little CPU). Effects on with it, or off on
   those systems? Recommendation: on, after one try on an old Mac if one is to hand.
6. **Upstream the two patches to DAW-v3** (the run loop is DAW-v3's D78 gap)? Recommendation: yes,
   from this branch's PATCHES.md, by the agent working on DAW-v3.
7. **"VST3" and "CLAP" on screen** in the effect list: allowed, as format names (the
   plug-ins already ship as VST3 and CLAP)? Recommendation: show the format as small print.
8. **Scanning the standard folders** loads every installed plug-in once (in helpers). Scan
   on first "+ ADD EFFECT" with a progress line, then from the cache: acceptable?
9. **Real effects for the release checks** on each OS: which free ones may agents install
   or load (Dragonfly Reverb on Linux worked; on the Mac only ValhallaRoom and Youlean are
   allowed for DAW-v3's tests, D79)?
