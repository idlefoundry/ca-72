# Vendored: clap-wrapper

The Audio Unit's wrapper (decisions.md R30): its AUv2 wrapper, built by `scripts/auv2.sh`
around the CLAP bundle, is what the component's own code is. Not modified.

[free-audio/clap-wrapper](https://github.com/free-audio/clap-wrapper), tag `v0.16.0`, commit
`1cca996e96f29ab2be7ae9f8cfe532bbc92e1dd6` (MIT, `LICENSE`): `CMakeLists.txt`, `LICENSE`,
`README.md`, `cmake/`, `include/`, `libs/` and `src/`; its tests and CI are left out. Of
`libs/`, the AUv2 wrapper compiles `fmt/` (the {fmt} library's headers, MIT, its licence in
`fmt/fmt/format.h`); `psl/` is the VST3 wrapper's. The SDKs it builds against are vendored
beside it, at the versions its own `cmake/base_sdks.cmake` fetches: `../clap` (CLAP 1.2.6)
and `../AudioUnitSDK` (AudioUnitSDK 1.1.0).

SHA-256 of the tree (every file but this one, sorted by path, as
`find . -type f ! -name VENDORED.md -print0 | sort -z | xargs -0 shasum -a 256 | shasum -a 256`
from this directory): `8a2ba5677ba806cee610fef7247df7787fc34455f07608c9181eedd76026b8ae`.
