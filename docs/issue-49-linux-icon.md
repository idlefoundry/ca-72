# Linux standalone window identity (issue #49)

[Issue #49](https://github.com/idlefoundry/ca-72/issues/49) reports a gear in Ubuntu 26.10's
dock after building from source. Two things are needed: the window's identity and an installed
desktop entry and icon. The window does not register the desktop files itself.

## The identifiers

| Identifier | Value | Purpose |
|---|---|---|
| Installed desktop file | `com.idlefoundry.ca72.desktop` | Desktop registration |
| `Icon` and installed PNG's stem | `com.idlefoundry.ca72` | Icon lookup |
| `StartupWMClass` | `CA-72` | Match the X11 window to the desktop entry |
| X11 `WM_CLASS`, instance and class | `CA-72`, `CA-72` | The standalone's window identity |
| `AppOptions.app_id` | `com.idlefoundry.ca-72` | Saved sessions and MIDI clients |

Linux's baseview backend is X11. In a Wayland session it uses XWayland; it does not create an
`xdg_toplevel` or set a native Wayland `app_id`. The similarly named `AppOptions.app_id` belongs
to plugin-kit's runtime, not the window backend. Changing it would reject existing saved
sessions, so it stays as it is. A future native Wayland backend would need its surface's
`app_id` to match the desktop file's stem, `com.idlefoundry.ca72`.

Both nih-plug standalone wrappers use `P::NAME` (`CA-72`) as the top-level window's initial
title. Previously they used an upstream baseview that did not set `WM_CLASS`. On Linux both
now use the local patched baseview, which sets the title as the instance and class before
mapping the window (`CA-72\0CA-72\0`, `STRING/8`). The production standalone also names
itself first (`identify_standalone`, `Ca72::NAME`), and that application class, not the title,
is every top-level window's class: the fatal-error window, titled `CA-72 — error`, keeps
`CA-72` too. Without an application class the initial title stays the class. Parented plug-in
editors do not receive their own class. The title and close protocol are also set before mapping. Linux's shared backend
keeps OpenGL disabled, as the editor requires; macOS and Windows retain their existing
standalone dependency and features.

[GNOME Shell's window tracker](https://github.com/GNOME/gnome-shell/blob/main/src/shell-window-tracker.c)
uses `StartupWMClass` to associate X11 windows with installed entries. The
[ICCCM](https://www.x.org/releases/current/doc/xorg-docs/icccm/icccm.html) specifies the class's
two strings and initial timing; the
[desktop entry specification](https://specifications.freedesktop.org/desktop-entry/latest/recognized-keys.html)
defines `StartupWMClass` and named icon lookup. Matching `StartupWMClass` lets this X11 class
differ from the desktop file's stem.

## Installing a source build

The README's [Building](../README.md#building) section gives the standalone-only installation
commands after `scripts/standalone.sh`. They stage the binary, desktop entry, PNG, installer and
licences, then run `install.sh --standalone-only`. This installs the application and desktop
files for the current user, without installing plug-ins. A packaged archive supports the same
installer option.

The installer puts the desktop entry in `${XDG_DATA_HOME:-$HOME/.local/share}/applications/`
and the PNG in its `icons/hicolor/512x512/apps/` directory. It installs the executable in
`~/.local/bin` and writes that path into the desktop entry. Running a bare executable alone
cannot supply an icon to a Wayland compositor through X11's class property.

## Automated checks

From the repository, with a build target of your own:

```sh
export CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=4
cargo test --manifest-path third_party/baseview/Cargo.toml --lib
cargo test -p ca72-plugin --test linux_window_identity
python3 scripts/test-linux-window-identity.py
python3 scripts/test-standalone-install.py
```

The four protocol tests record real X11 requests without creating a window. They check the
class payload and type, identity before mapping, parented editors, another instrument's title
and a transport failure preventing mapping. The product test compares `Ca72::NAME`, which both
wrappers use, with the launcher's `StartupWMClass`.

The Python identity checks resolve both standalone features on Linux, macOS and Windows.
Linux must select the local backend without OpenGL; the other platforms keep their existing
revision with OpenGL. Metadata uses the lockfile and may fetch dependencies for those targets.
An isolated installation fixture also checks the installed desktop file's stem against `Icon`,
the class and the installed PNG's content. Changing `Icon` in a disposable source copy makes
that check fail. The installer suite checks the component selections and desktop command.

## Desktop evidence, 2026-10-18

On the Linux reference machine (Hyprland 0.56.2, a Wayland session), the release-profile
production build ran with no devices and scratch shared/session directories. `hyprctl clients`
reported class and initial class `CA-72`, title `CA-72`, and `xwayland: true`. `xprop` reported
`WM_CLASS(STRING) = "CA-72", "CA-72"` and `WM_NAME(STRING) = "CA-72"`.

In a new QEMU/KVM VM, quickget's Ubuntu 26.10 desktop ISO passed its published SHA-256 check.
The live desktop ran GNOME Shell 51.0 in a Wayland session. The same production binary, checked
by SHA-256 after transfer and installation, showed a gear when run before desktop registration.
After the README's payload was installed with `scripts/installer/install.sh --standalone-only`,
the installed binary showed the CA-72 icon and tooltip. Launching the installed desktop entry
with `gtk-launch com.idlefoundry.ca72` also opened the application. Pinning that entry kept one
CA-72 dock item for the running window. Its X11 client had the expected class and title.
Screenshots and window-property logs are retained with the job's summary.

The production and development release builds passed on Linux. The production-feature plug-in
suite passed 308 tests, with 31 ignored; nih-plug's library suite passed 68. The four protocol,
one product-identity, three Python identity and four installer tests passed. Plug-in clippy
passed with default and all features; baseview's default-feature clippy passed. Direct vendored
clippy also exposes existing warnings in unchanged upstream files: five lint categories in
nih-plug, and `useless_borrows_in_formatting` in baseview's optional X11 OpenGL code. With only
those categories allowed, those checks passed. Workspace and vendored formatting, whitespace
and notices checks passed. Independent source review found no remaining defects.

Not tested: a native Xorg desktop session, macOS/Windows builds after rebasing, or audio/MIDI
playback. Native Wayland is not a supported backend. The GNOME test used the live desktop,
not an Ubuntu installation on the VM's disk. The reporter's original build and session are
unknown; the bare-executable reproduction confirms one cause of the reported symptom, not
which cause they encountered. The full workspace suite was not repeated beside the other
agents' lab and workspace runs.
