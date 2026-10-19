# Vendored: DAW-v3's plug-in hosting

The standalone application's effects (docs/fx-host.md) are hosted out of process with the
plug-in sandbox of the owner's DAW, DAW-v3 (`git@github.com:Idlewile/DAW-v3.git`, private,
GPL-3.0-or-later, as the CA-72), at commit `cb15558711ea1b3a1c2ce8b27fa7aadb8ef17a63`
(the four crates' files last changed in `356bdb76dd848bef463d8f563556413c07c0185e`,
2026-10-12). Taken as `git archive` gives them, with four patches (PATCHES.md: a run loop for
VST3 editors on Linux; macOS's wait functions looked up at run time; macOS's editor windows
shown in front; the test plug-ins' faults at load, restore and listing):

- `daw-ipc`: the transport the standalone and its helper share: named shared memory,
  cross-process wake-ups, the block layout and the line protocol (DAW-v3 D19);
- `daw-helper`: the helper process's host of CLAP and VST3 plug-ins, their editors' windows
  included (D19, D63, D78), linked only into `ca72-fx-helper`, never into the standalone;
- `daw-test-plugin` and `daw-test-vst3`: DAW-v3's test plug-ins (deterministic gain,
  latency and delay, state, hidden crash and hang parameters), built for the tests only and
  never shipped.

Their manifests take their versions, dependencies and lints from this workspace
(`Cargo.toml`, whose lints and toolchain are DAW-v3's); they are workspace members, so that
their own tests run in the CA-72's CI. Their comments' record ids (D19, D63, M6 and the
like) are DAW-v3's `docs/DECISIONS.md` and `docs/plan`; their unsafe code is listed in
DAW-v3's `docs/unsafe.md`, whose rows for these crates hold here.

The host side, DAW-v3's `daw-sandbox` (helper supervision, the engine node), depends on the
DAW's engine and is not vendored: `crates/ca72-fx` adapts the parts the standalone needs,
each file saying what it takes.

SHA-256 of the tree as taken, before the patches (every file but this one and PATCHES.md,
sorted by path, as `find . -type f ! -name VENDORED.md ! -name PATCHES.md -print0 | sort -z |
xargs -0 sha256sum | sha256sum` from this directory): `7b816b1ef0616f244371a7f1be9eeb8f8155b59a3feab8772d32dceddfb05c0e`.

To update: `git archive <commit> crates/daw-ipc crates/daw-helper crates/daw-test-plugin
crates/daw-test-vst3 | tar -x --strip-components=1` here from a DAW-v3 checkout, then the
hash and commit above, the patches in PATCHES.md again, and `cargo test -p daw-ipc -p
daw-helper -p ca72-fx-helper`.
