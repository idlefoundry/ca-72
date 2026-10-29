# DAW-v3's plug-in hosting, patched

The four crates as VENDORED.md says, with six changes, to be offered to DAW-v3 itself:

1. **A VST3 editor on Linux gets the host's run loop (`IRunLoop`)** (`daw-helper/src/vst3.rs`;
   DAW-v3's D78 lists it as not yet done). DAW-v3's `Frame`, the
   `IPlugFrame` a VST3 view is given, now also answers for `Steinberg::Linux::IRunLoop`: the
   view's file descriptors (`registerEventHandler`) are polled without waiting and its
   timers (`registerTimer`) fired, both from `gui_service` on the helper's main thread, every
   10 ms, each handler called with no lock held (it may register or unregister as it runs);
   every handler is let go when the editor closes. Most Linux VST3 editors need it: without
   it, Dragonfly Reverb 3.2.10's four VST3 editors refused to attach ("the plugin could not
   attach its editor to the window"); with it they open and draw (2026-10-18, this computer,
   XWayland under Hyprland). Their CLAP editors needed nothing: DAW-v3 serves CLAP's
   `timer-support` and `posix-fd-support` already.

   Seen, not fixed: those VST3 editors draw their knobs at zero although the controller
   reports the plug-in's values (0.8 for a default of 80 %), where the CLAP ones show the
   defaults. Telling the controller the processor's state at creation
   (`setComponentState`) did not change it; how they expect their first values is still to
   be found (docs/fx-host.md).

2. **macOS's wait functions are looked up at run time** (`daw-ipc/src/wake.rs`). The
   transport called `os_sync_wait_on_address`, `..._with_timeout` and
   `os_sync_wake_by_address_all` (macOS 14.4 and later) through a plain `extern` block, so
   every program linking `daw-ipc` referred to them strongly (`nm -m`: "(undefined) external
   ... (from libSystem)") and would not start before macOS 14.4. That was DAW-v3's own
   floor; the CA-72's standalone, which now links `daw-ipc`, supports macOS 11. They are now
   looked up once with `dlsym(RTLD_DEFAULT, ...)`; where they are missing a wait polls the
   word every 50 µs until it changes or its time is up, and a wake does nothing. On the Mac
   (macOS 27, 2026-10-18), built for macOS 11: no `os_sync` symbol referred to, clippy
   clean, `daw-ipc`'s tests (with one new for the polled wait) and the chain's pass. Not
   tried on a system before 14.4.

3. **macOS: an editor's window is shown in front of every application's windows**
   (`daw-helper/src/window/cocoa.rs`, `Window::show`: `orderFrontRegardless` after
   `makeKeyAndOrderFront:`). The helper is an accessory application that is not active when the
   host asks for a window: the player has just clicked the host's window, which made the host the
   active application. An inactive application's `makeKeyAndOrderFront:` left the window behind
   the host's: a window opened again after the player closed it (a new one), or an open one
   asked for again, was there but hidden behind the CA-72's (decisions.md R-FX-MAC). On the Mac
   (macOS 27, 2026-10-19), clicks posted to the standalone: before, the window opened again was
   second in the screen's order, under the CA-72's; with it, first, both opened again and brought
   forward. The window still does not take the keyboard (the host keeps it).

4. **The test plug-ins fault as they load, as a host restores them and as they are listed**
   (`daw-test-plugin/src/lib.rs`, `daw-test-vst3/src/lib.rs`; decisions.md R-FX-SAFETY), for
   the CA-72's tests that no effect can keep the standalone from starting or playing. Before,
   they faulted only while processing (their hidden `crash` and `hang`). Now:
   - plug-in IDs that abort or hang as they are made, which the factories do not list: CLAP
     `org.daw-v3.test-plugin.crash-at-load` and `...hang-at-load`; VST3 processor classes
     `44415733546573744761696E43525348` ("CRSH") and `...48414E47` ("HANG");
   - a state pair, (`FAULT_ID` = `0xFFFF_FF02`, value), in the states both keep: 1 aborts
     while the state is loaded, 2 hangs then, 3 is loaded, saved again with the state, and
     aborts at the first block after (a saved state that crashes the plug-in each time it is
     restored);
   - the CLAP library aborts or hangs in its entry's `init` when its file name holds
     `crash-at-scan` or `hang-at-scan`, so as a host lists it (and loads it).
   States without the pair, the listed plug-ins and every other behaviour are as before; their
   own tests pass, and clippy is clean.

5. **A block carries the host's tempo and place to the plugin** (`daw-ipc/src/layout.rs`'s
   `Timing`, `daw-helper/src/{lib,plugin,clap,vst3}.rs`; the CA-72's `docs/decisions.md` R-LFO).
   Before, a block's plugin had no transport at all: CLAP's `transport` and VST3's
   `processContext` were null, so a tempo-synced effect after the CA-72's standalone ran at its
   own default. Now the host writes each block's tempo (BPM), its place in quarter notes and the
   sample rate into the slot, in what was its padding (the slot's size and its events' offset,
   64, unchanged: a test holds them), and the helper gives them as a stopped transport: CLAP's
   `clap_event_transport` with `HAS_TEMPO`, `HAS_BEATS_TIMELINE` and `HAS_TIME_SIGNATURE` (4/4,
   the bar from the place), VST3's `ProcessContext` with `kTempoValid`,
   `kProjectTimeMusicValid`, `kBarPositionValid` and `kTimeSigValid`. A block with no tempo (0)
   gives none, as before. `Transport::write_block` and `Processor::process` take the timing;
   the helper's tests pass none. Tried through the layout's round trip; no third-party effect
   that reads the tempo was tried.

6. **A plugin's parameters listed again, their text, and modulation the VST3 window does not
   see** (`daw-ipc/src/protocol.rs`, `daw-helper/src/{lib,plugin,vst3,clap}.rs`; the CA-72's
   decisions.md R-FX-PARAMS, its effects' pinned parameters). Protocol version 4:
   - `ParamInfo` gains `short` (VST3's `shortTitle`), `units`, `steps` (VST3's `stepCount`, a
     CLAP stepped parameter's range; a VST3 parameter stays normalized, `stepped` false),
     `read_only` and `bypass`;
   - `Op::Params {instance}` lists the parameters again with their values now; `Op::ParamText
     {instance, values}` gives the plugin's own text for each (parameter, value) pair (VST3
     `getParamStringByValue`, its `units` put after a number given alone; CLAP `value_to_text`),
     `null` where it gives none;
   - a `params_changed` notice when the plugin says its parameters' names or list changed
     (VST3 `restartComponent(kParamTitlesChanged)` or `kReloadComponent`, CLAP
     `clap_host_params.rescan` of info, text or all), for the host to list them again;
   - a VST3 controller (its editor) is told a parameter's base alone, never its modulation:
     `EV_PARAM_MOD` reaches the processor as base plus amount and is no longer mirrored every
     10 ms (the window chased an LFO, and a drag there fought it). The processor's own state
     may then hold a modulated instant: the host gives the base back after a state.
   Tried with both test plugins through the CA-72's chain (`ca72-fx-helper/tests/params.rs`):
   listings, units and steps, texts, a title-change word's path (not triggered: the test
   effects never change their titles), the modulation's offsets and the window left at its
   base. DAW-v3's own tests pass.
