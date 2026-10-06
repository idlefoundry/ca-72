# nih-plug, patched

The `nih_plug` crate and its derive macros from
[robbert-vdh/nih-plug](https://github.com/robbert-vdh/nih-plug) at commit
`de421011f41a6d10fc8c7a6084e4f4dee0143683` (ISC, `LICENSE`; its VST3 bindings are
GPL-3.0). The manifest's `[workspace]` and `[profile]` tables are removed, and two of
upstream's own compiler warnings are allowed. The examples, the GUI adapters and the other
plug-ins are left out.

The commit is the project's latest. It fails four of clap-validator 0.4.1's tests, and so
do its own example plug-ins (`sine`, `gain`). These two changes, both in
`src/wrapper/clap/wrapper.rs`, fix them:

1. **A corrupt state no longer aborts the host.** `ext_state_load` reserved the length
   the stream's first eight bytes claim before reading any of it. With random bytes that
   is about 10^18 bytes, and the failed allocation aborts the process (clap-validator's
   `state-invalid-random`). The buffer now grows as the data arrives, 64 KiB at a time,
   so a short or corrupt stream fails the load instead.
2. **The host is told the values changed after a state load.** `set_state_inner` now also
   asks the host to rescan the parameters' values (`CLAP_PARAM_RESCAN_VALUES`). Without
   that, a host may keep showing the values from before the load
   (`state-reproducibility-basic`, `-binary` and `-buffered`).

A third change, in `src/wrapper/vst3/wrapper.rs`, comes from Steinberg's VST3 validator
(SDK 3.8.1):

3. **A bus arrangement is matched against the right buses.** `set_bus_arrangements`
   took a layout's first auxiliary bus to be the host's bus 1 when the layout has no main
   bus, and bus 0 when it has one: the wrong way round (the rest of the file has it
   right). For an instrument with a side chain it read past the end of the host's
   one-element array, and so refused every arrangement the host asked for (the
   validator's mono test: "Mono Input-SpeakerArrangement is not supported").

Five more came from a review of the plug-in before its release (2026-10-03; the CA-72's
`docs/decisions.md` R18):

4. **A VST3 host's refusal to resize reaches the editor.** `request_resize` always
   returned `true`, whatever the host answered, so an editor could not tell that its
   window had not grown (the CA-72's presets' drawer, opening below the panel, went
   unseen while it held the keyboard). On the GUI thread, where the host answers at once
   (macOS, Windows), it now returns the host's answer; elsewhere (Linux, where baseview's
   window has a thread of its own) it still schedules the request and returns `true`.
   `WrapperView::request_resize` (`src/wrapper/vst3/view.rs`) no longer asserts that the
   host agreed, which stopped a debug build at a refusal.
5. **Auxiliary buses are bounded by the host's own count.** In `process`, the VST3 wrapper
   bounded the auxiliary input buses by the host's number of output buses, and both
   wrappers compared with `>`, so a bus at the host's count was read one past the end of its
   array (`src/wrapper/vst3/wrapper.rs`: inputs by `num_inputs`, both with `>=`;
   `src/wrapper/clap/wrapper.rs`: both with `>=`).
6. **A side chain's missing channels are as long as the block.** Channels the host did not
   supply (too few of them, or no pointers) were zeroed at whatever length an earlier block
   left them, which may be shorter than this block's, and a plug-in reading them by the
   block's samples panicked. They are now resized to the block first, within the room
   reserved for them (`src/wrapper/util/buffer_management.rs`).
7. **VST3's buffer configuration carries the processing mode just set.**
   `setup_processing` stored it with the mode from before the call, so `initialize` saw the
   previous mode: realtime for an offline render, offline for playback after one
   (`src/wrapper/vst3/wrapper.rs`).
8. **The processing mode in `process`.** `ProcessContext::process_mode()`
   (`src/context/process.rs`; the VST3, CLAP and standalone contexts). CLAP's `render`
   extension may change the mode while the plug-in is active, and the plug-in is not
   initialized again for it, so `BufferConfig::process_mode` alone may be out of date.

One more came from a report: the CA-72 crashed in Sandyne, a DAW built with JUCE (2026-10-06;
the CA-72's `docs/decisions.md` R31):

9. **A channel the host gives as a null pointer is not read.** A VST3 host may give a null
   pointer for a channel, and JUCE's hosts do for every channel of a bus they deactivated,
   with the bus's full channel count (`HostBufferMapper::associateBufferTo`); setting up an
   instrument with no inputs deactivates its side chain. `create_buffers` read every input
   channel through its pointer, so an instrument with a side chain read from address 0 in
   its first block, an access violation. A null input channel is now read as silence, and a
   null output channel is backed by scratch storage, allocated with the rest and discarded,
   so that every channel the plug-in sees is as long as the block
   (`src/wrapper/util/buffer_management.rs`, with a test of each kind of channel).

To move to a newer upstream commit, copy its `Cargo.toml`, `LICENSE`, `README.md`, `src`
and `nih_plug_derive` here and apply the nine changes again, unless upstream has fixed them.
Then update the commit above and `nih_plug_xtask`'s `rev` in the workspace's `Cargo.toml`.
