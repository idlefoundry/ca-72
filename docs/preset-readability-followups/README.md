# Preset readability: the original candidates

The final readability implementation from the earlier branch is already on main in
[PR #85](https://github.com/idlefoundry/ca-72/pull/85), commit `425ed41`. Its decision record
records owner approval of the final treatment. Main at `1a1dd6c` (0.2.6) retains it.
The final candidate `6c1e144` and the Init branch render identically in this matched fixture.
There are no remaining readability code changes to rebase. This branch, based on
`init-and-version`, preserves the historical review sequence and adds a reproducible drawer
mode to the panel PNG example. The original candidate records that said approval was pending
predate #85; that later approval record is authoritative.

## Matched PNGs

Every pair comes from the panel PNG example with `CA72_SKIN=worn CA72_PRESETS=1` and
identical contents: Brass Tutti current, Hollow Glider hovered, Search focused. The status
reads RENDER FIXTURE. It is deterministic fixture text, not an update check or installed
version. The surrounding panel and strip use current main; only the two preset-renderer
source files are taken from each historical revision. This isolates the visual changes.

The linked images are lossless drawer crops of the full PNGs at native widths 860 and 1035.
The historical full panels labelled 860 are actually 861 pixels wide: the original scale
rounds just above 860 before the renderer rounds upwards. The archived drawer crops omit
that final column and are exactly 860 pixels wide. The reproduction command below uses a
slightly lower scale for an exactly 860-pixel-wide full panel.
Full before/after panels for every commit, including documentation-only commits, are in
`target/night-init/readability/<original-commit>/before-<width>.png` and
`target/night-init/readability/<original-commit>/after-<width>.png` in the integration worktree.
The manifest there lists full paths and changed-pixel counts. Documentation-only pairs are
pixel-identical. Original commits are identified below for provenance; superseded palette
settings are historical review points, not additional changes awaiting application.

| Original commit | Visual change | 860 px before / after | 1035 px before / after |
| --- | --- | --- | --- |
| `07244d8` | Larger captions on larger action caps; larger section titles; brighter amber text, quieter glass and unlit dots; selected tags and tabs gain markers. | [before](images/33cec21-860.png) / [after](images/07244d8-860.png) | [before](images/33cec21-1035.png) / [after](images/07244d8-1035.png) |
| `4392363` | A one-dot bar replaces the selected tag arrow, leaving two clear dot columns before the caption. | [before](images/07244d8-860.png) / [after](images/4392363-860.png) | [before](images/07244d8-1035.png) / [after](images/4392363-1035.png) |
| `fe428a5` | Returns display text towards orange (255, 140, 58), with ordinary/secondary/faint levels 0.78/0.62/0.50; keeps the larger captions. | [before](images/4392363-860.png) / [after](images/fe428a5-860.png) | [before](images/4392363-1035.png) / [after](images/fe428a5-1035.png) |
| `a64685a` | Evidence and decision records only, including Settings version evidence. No visual change. | [before](images/fe428a5-860.png) / [after](images/fe428a5-860.png) | [before](images/fe428a5-1035.png) / [after](images/fe428a5-1035.png) |
| `cac1799` | Current preset becomes gold with a filled triangle; hover stays amber with an outline arrow. Selection changes restore the previous row correctly. | [before](images/fe428a5-860.png) / [after](images/cac1799-860.png) | [before](images/fe428a5-1035.png) / [after](images/cac1799-1035.png) |
| `792e19e` | Selection candidate evidence only. No visual change. | [before](images/cac1799-860.png) / [after](images/cac1799-860.png) | [before](images/cac1799-1035.png) / [after](images/cac1799-1035.png) |
| `381185d` | Lifts ordinary/secondary/faint text to 0.85/0.70/0.60, retaining the gold current row and hover/focus treatment. | [before](images/cac1799-860.png) / [after](images/381185d-860.png) | [before](images/cac1799-1035.png) / [after](images/381185d-1035.png) |
| `e853b2f` | Brightness candidate evidence only. No visual change. | [before](images/381185d-860.png) / [after](images/381185d-860.png) | [before](images/381185d-1035.png) / [after](images/381185d-1035.png) |
| `a3195b0` | Darkens inactive text to 0.815/0.66/0.55; preserves the preceding appearance inside hovered, focused and chosen controls. | [before](images/381185d-860.png) / [after](images/a3195b0-860.png) | [before](images/381185d-1035.png) / [after](images/a3195b0-1035.png) |
| `8139a7c` | Midpoint and cursor-state evidence only. No visual change. | [before](images/a3195b0-860.png) / [after](images/a3195b0-860.png) | [before](images/a3195b0-1035.png) / [after](images/a3195b0-1035.png) |
| `a4b4359` | Darkens inactive text again to 0.80/0.60/0.50, preserving active controls. | [before](images/a3195b0-860.png) / [after](images/a4b4359-860.png) | [before](images/a3195b0-1035.png) / [after](images/a4b4359-1035.png) |
| `2005956` | Darker-text evidence only. No visual change. | [before](images/a4b4359-860.png) / [after](images/a4b4359-860.png) | [before](images/a4b4359-1035.png) / [after](images/a4b4359-1035.png) |
| `6c1e144` | Substantially darkens inactive text to 0.65/0.475/0.40, preserving active controls, gold selection and amber hover. | [before](images/a4b4359-860.png) / [after](images/6c1e144-860.png) | [before](images/a4b4359-1035.png) / [after](images/6c1e144-1035.png) |
| `b56df23` | Luminance measurements and final candidate evidence only. No visual change. | [before](images/6c1e144-860.png) / [after](images/6c1e144-860.png) | [before](images/6c1e144-1035.png) / [after](images/6c1e144-1035.png) |

## Reproduce the current panel

```sh
export CARGO_TARGET_DIR=$PWD/target CARGO_BUILD_JOBS=4
export IDLE_FOUNDRY_SHARED=$PWD/target/review-shared
CA72_SKIN=worn CA72_PRESETS=1 cargo run -p ca72-panel --example png -- target/presets-1035.png 0.333011583011583
CA72_SKIN=worn CA72_PRESETS=1 cargo run -p ca72-panel --example png -- target/presets-860.png 0.27670527670527667
```

The renderer uses the same preset-drawer materials in both looks. The PNG mode adds no
plug-in behaviour. Native-host interaction and owner hands-on acceptance of Init remain
separate from these offscreen comparisons.

## Verification

Both crates' default tests and all-target clippy pass, the latter with default and all
features. The full workspace tests with `ca72-plugin/standalone-production` pass on Linux.
The 14 historical pairs at each scale were rendered with the same example and fixtures;
the six documentation-only pairs at each scale are pixel-identical. A separate source and
image-path review checked the links, crop fidelity, pixel counts and approval record.
