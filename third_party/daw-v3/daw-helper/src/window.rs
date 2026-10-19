//! Plugin windows (DESIGN 10.5): top-level windows the helper creates and owns, one per
//! open plugin GUI, which the plugin embeds its view into. The DAW positions them; the
//! platform's window manager decorates and stacks them. Everything here runs on the
//! helper's main thread.
//!
//! - Linux and other Unix systems: X11 (also under Wayland, through XWayland: most plugins
//!   only speak X11). libX11 is loaded when the first window opens, so a helper on a
//!   machine without a display still hosts plugins for audio.
//! - Windows: Win32.
//! - macOS: AppKit, through the Objective-C runtime.

#![allow(unsafe_code)]

use std::ffi::CStr;

/// Something that happened to a window since the last [`Window::events`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowEvent {
    /// The user closed it (the window is hidden, not destroyed).
    Closed,
    /// Its content area has a new size.
    Resized(u32, u32),
}

/// What the helper can say about a window it shows, as evidence that the plugin is in it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Check {
    /// The window is mapped and viewable.
    pub visible: bool,
    /// Child windows or views inside its content area: the plugin's.
    pub children: usize,
}

#[cfg(all(unix, not(target_os = "macos")))]
mod x11;
#[cfg(all(unix, not(target_os = "macos")))]
pub use x11::{Window, pump};

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::{Window, pump};

#[cfg(target_os = "macos")]
mod cocoa;
#[cfg(target_os = "macos")]
pub use cocoa::{Window, pump};

/// The CLAP window API plugins embed into on this platform.
pub fn api() -> &'static CStr {
    #[cfg(all(unix, not(target_os = "macos")))]
    return clap_sys::ext::gui::CLAP_WINDOW_API_X11;
    #[cfg(windows)]
    return clap_sys::ext::gui::CLAP_WINDOW_API_WIN32;
    #[cfg(target_os = "macos")]
    return clap_sys::ext::gui::CLAP_WINDOW_API_COCOA;
}
