# Packs as they were: never edit these

Every released skin-pack format and revision of the CA-72's panel controls keeps a sample here
(decisions.md R-SKIN-PACKS), and every later build must keep loading each one with only the
outcome `tests/compat.rs` expects. A change that would break packs people have made fails there
before it ships.

Add a folder for each new revision (`v2/`, ...) with packs made for it; never change or remove a
file in an existing one. If a test here fails, the change is what must give, not the sample.

- `v1/` (2026-10-16, the format's first revision, `format = 1`, `controls = 1`):
  - `starter/`: the CC0 starter as it was, its layout without a `controls` key (a format-1
    layout without one is made for the first revision);
  - `official-layout.toml`: the official layout, with `controls = 1`;
  - `edge-layout.toml`: the EDGE stress pack's layout (every limit), without a `controls` key;
  - `lifecycle-1.1.0.ifskin`: a pack file signed with the stress packs' test key
    (`packs/stress.rs`), so that the signature's format stays verifiable.
