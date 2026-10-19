# DAW-v3's plug-in hosting, patched

The four crates as VENDORED.md says, with four changes, to be offered to DAW-v3 itself:

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
