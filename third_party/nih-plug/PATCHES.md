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

To move to a newer upstream commit, copy its `Cargo.toml`, `LICENSE`, `README.md`, `src`
and `nih_plug_derive` here and apply both changes again, unless upstream has fixed them.
Then update the commit above and `nih_plug_xtask`'s `rev` in the workspace's `Cargo.toml`.
