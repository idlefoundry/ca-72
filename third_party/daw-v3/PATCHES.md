# DAW-v3's plug-in hosting, patched

The four crates as VENDORED.md says, with one change, in `daw-helper/src/vst3.rs`, to be
offered to DAW-v3 itself (its D78 lists it as not yet done):

1. **A VST3 editor on Linux gets the host's run loop (`IRunLoop`).** DAW-v3's `Frame`, the
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
