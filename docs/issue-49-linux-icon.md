# Linux standalone window identity (issue #49)

**Agent decisions, 2026-10-18; pending the owner's Ubuntu check.** This is a local source
fix for [issue #49](https://github.com/idlefoundry/ca-72/issues/49), whose report names Ubuntu
26.10 and a source build. The report does not identify the build command, desktop-entry
installation, session type or window properties. The symptom has not been reproduced here.

## Cause supported by the source

- `scripts/standalone/CA-72.desktop` supplies `Icon=com.idlefoundry.ca72` and
  `StartupWMClass=CA-72`. `scripts/installer/install.sh` already installs the PNG in
  `icons/hicolor/512x512/apps/` and the entry as `applications/com.idlefoundry.ca72.desktop`.
- Both the production binary (`scripts/standalone.sh`, `standalone-production`) and the
  development example (`--example standalone --features standalone`) open their top-level
  window through nih-plug with the initial title `P::NAME`, which is `CA-72`.
- nih-plug previously used a separate upstream baseview at `579130e` for these windows.
  The local patched baseview was used only for the parented editor. Both X11 backends mapped
  their window before setting `WM_NAME` and did not set `WM_CLASS` at all.
- [GNOME Shell's window tracker](https://github.com/GNOME/gnome-shell/blob/main/src/shell-window-tracker.c)
  matches the instance and class from `WM_CLASS` against `StartupWMClass` in installed desktop
  entries. This missing identity explains a failure to associate the window with its launcher;
  other heuristics may still associate it in some launch contexts. It is a source-supported
  cause, not confirmation of the reporter's exact failure.
- Building or running a raw executable does not install its launcher or icon. Missing desktop
  registration is a separate possible cause; `WM_CLASS` alone does not supply an icon.

The fix selects the local baseview for both Linux standalone features, retaining the existing
standalone dependency on macOS and Windows. It sets the top-level window's initial title as
both instance and class before mapping (`CA-72\0CA-72\0`, `STRING/8`). Parented plug-in editors
do not receive `WM_CLASS`. The existing title and close protocol are also set before mapping.
Linux shares the editor's backend without OpenGL; both standalone wrappers omit the
feature-gated `gl_config` field there. Keeping that feature off avoids changing the editor's
window options when Cargo unifies the shared dependency's features.
The [ICCCM](https://www.x.org/releases/current/doc/xorg-docs/icccm/icccm.html)
specifies the two strings and initial property timing; the
[desktop entry specification](https://specifications.freedesktop.org/desktop-entry/latest/recognized-keys.html)
defines `StartupWMClass` and named icon lookup. No image asset or installation destination
changes are needed.

## Automated checks

Run these from the repository, with a build target of your own:

```sh
CARGO_TARGET_DIR="$PWD/target-identity" CARGO_BUILD_JOBS=2 \
  cargo test --offline --manifest-path third_party/baseview/Cargo.toml --lib
python3 scripts/test-linux-window-identity.py
```

The Rust tests record X11 requests without creating a window or opening audio devices. They
check the exact class payload and type, identity before mapping, parented editors, another
instrument's title and a transport failure preventing mapping. The Python checks resolve both
standalone features for Linux, macOS and Windows; they require Linux's top-level window backend
to be the patched local crate, and the other platforms' to remain at their existing revision.
They also require OpenGL to remain disabled on Linux's shared backend and enabled on the
unchanged standalone backend on the other platforms. They do not compile those targets.

**Evidence (the Mac, 2026-10-18).** Four protocol tests passed. Removing only `WM_CLASS`
emission in a separate source copy made two fail, with the other two passing. Both dependency
tests passed, covering six feature/platform combinations. A retained installation fixture,
using a dummy executable and a custom prefix containing spaces and `%`, confirmed the existing
installer preserves `StartupWMClass`, the icon name, exact PNG content and its 512-pixel path,
and creates only the selected standalone components. No executable was launched.

Workspace and baseview formatting checks passed. Baseview Clippy passed with and without all
features, with `-D warnings` and allowances for unchanged upstream code: `unexpected_cfgs`
(Objective-C macros), `clippy::ptr_eq` (`src/macos/view.rs`) and, for all features,
`clippy::unnecessary_unwrap` (`src/gl/macos.rs`). Strict Clippy without these allowances does
not pass on the baseline either; those files were not changed. No full application build or
workspace DSP test suite was run. Linux CI now runs both new test commands; it was not
dispatched from this local investigation.

## Ubuntu 26.10 validation still required

Use an isolated checkout and build target. Run the checks above on Ubuntu, then build the
production app with `scripts/standalone.sh`. To prepare a standalone-only installation from
source without building the plug-in bundles:

```sh
mkdir -p target/standalone-install
cp target/bundled/ca72-standalone scripts/standalone/CA-72.desktop \
  scripts/standalone/com.idlefoundry.ca72.png scripts/installer/install.sh \
  THIRD-PARTY-NOTICES.txt target/standalone-install/
cp LICENSE target/standalone-install/LICENSE.txt
sh target/standalone-install/install.sh --standalone-only
```

That last command installs for the current user. It is a validation instruction, not a command
run during this Mac investigation. Keep other CA-72 sessions closed while comparing builds.

1. Record the desktop and session type (`echo "$XDG_CURRENT_DESKTOP $XDG_SESSION_TYPE"`).
   Test the normal Wayland session through XWayland, and Xorg if available. This backend is X11.
2. Verify the installed entry contains `StartupWMClass=CA-72` and `Icon=com.idlefoundry.ca72`,
   and the PNG exists beneath `${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/512x512/apps/`.
3. First run the production binary without devices or the normal session data:
   `target/bundled/ca72-standalone --no-devices --state-dir "$PWD/target/icon-check-session"`.
   In another terminal run `xprop WM_CLASS WM_NAME` and select its window. Expect
   `WM_CLASS(STRING) = "CA-72", "CA-72"` and `WM_NAME(STRING) = "CA-72"`.
4. Inspect the dock while that window is open. With the entry and icon registered, it should
   use the CA-72 icon and group with a pinned CA-72 launcher, without a separate gear item.
   Close it. Separately launch the installed CA-72 from the applications menu and repeat the
   icon and grouping checks; that normal launch may access the selected audio and MIDI devices.
5. If the report concerns the development example, build that command separately and inspect
   its `WM_CLASS` too. It uses the same Linux backend, but remains the legacy development host.
6. Compare the unmodified production commit using a separate build target and session directory.
   Record window properties and screenshots for both builds. A bare executable with no desktop
   entry installed is not a sufficient test of launcher matching.

No Ubuntu or XWayland window was opened during this investigation. Linux compilation, actual
window properties, dock icon selection and grouping remain unverified until that check.
