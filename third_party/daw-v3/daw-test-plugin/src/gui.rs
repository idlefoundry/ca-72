//! The test plugin's GUI (M3 plan W10): a small view of one colour, embedded in the
//! host's window (X11, Win32 or Cocoa). It draws nothing of its own, so it needs no event
//! loop: the window system paints its background. That is enough to test opening,
//! moving, closing and crashing a plugin window.

use clap_sys::ext::gui::{clap_plugin_gui, clap_window};
use clap_sys::plugin::clap_plugin;
use std::ffi::{CStr, c_char};

/// The view's size, fixed.
pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 160;

/// The view's colour, `0xRRGGBB`.
const COLOUR: u32 = 0x00d5_7b3a;

#[cfg(all(unix, not(target_os = "macos")))]
const API: &CStr = clap_sys::ext::gui::CLAP_WINDOW_API_X11;
#[cfg(windows)]
const API: &CStr = clap_sys::ext::gui::CLAP_WINDOW_API_WIN32;
#[cfg(target_os = "macos")]
const API: &CStr = clap_sys::ext::gui::CLAP_WINDOW_API_COCOA;

/// A plugin instance's GUI state; touched only on the main thread.
#[derive(Debug, Default)]
pub struct Gui {
    created: bool,
    view: Option<native::View>,
}

/// # Safety
/// `p` must be a plugin of this library.
unsafe fn gui<'a>(p: *const clap_plugin) -> std::sync::MutexGuard<'a, Gui> {
    // SAFETY: as the caller guarantees.
    let pl = unsafe { crate::plugin(p) };
    pl.gui.lock().unwrap_or_else(|e| e.into_inner())
}

/// # Safety
/// `api` must be null or a NUL-terminated string.
unsafe fn is_ours(api: *const c_char) -> bool {
    // SAFETY: as the caller guarantees.
    !api.is_null() && unsafe { CStr::from_ptr(api) } == API
}

unsafe extern "C" fn is_api_supported(
    _: *const clap_plugin,
    api: *const c_char,
    floating: bool,
) -> bool {
    // SAFETY: the host passes a NUL-terminated API name.
    !floating && unsafe { is_ours(api) }
}

unsafe extern "C" fn get_preferred_api(
    _: *const clap_plugin,
    api: *mut *const c_char,
    floating: *mut bool,
) -> bool {
    if api.is_null() || floating.is_null() {
        return false;
    }
    // SAFETY: the host's out-parameters.
    unsafe {
        *api = API.as_ptr();
        *floating = false;
    }
    true
}

unsafe extern "C" fn create(p: *const clap_plugin, api: *const c_char, floating: bool) -> bool {
    // SAFETY: the host passes its plugin and a NUL-terminated API name.
    if floating || !unsafe { is_ours(api) } {
        return false;
    }
    // SAFETY: a plugin of this library.
    unsafe { gui(p) }.created = true;
    true
}

unsafe extern "C" fn destroy(p: *const clap_plugin) {
    // SAFETY: a plugin of this library.
    let mut g = unsafe { gui(p) };
    g.view = None;
    g.created = false;
}

unsafe extern "C" fn set_scale(_: *const clap_plugin, _: f64) -> bool {
    true
}

unsafe extern "C" fn get_size(_: *const clap_plugin, w: *mut u32, h: *mut u32) -> bool {
    if w.is_null() || h.is_null() {
        return false;
    }
    // SAFETY: the host's out-parameters.
    unsafe {
        *w = WIDTH;
        *h = HEIGHT;
    }
    true
}

unsafe extern "C" fn can_resize(_: *const clap_plugin) -> bool {
    false
}

unsafe extern "C" fn no_size(_: *const clap_plugin, _: *mut u32, _: *mut u32) -> bool {
    false
}

unsafe extern "C" fn set_size(_: *const clap_plugin, w: u32, h: u32) -> bool {
    w == WIDTH && h == HEIGHT
}

unsafe extern "C" fn set_parent(p: *const clap_plugin, window: *const clap_window) -> bool {
    // SAFETY: a plugin of this library.
    let mut g = unsafe { gui(p) };
    // SAFETY: a non-null window description from the host.
    if !g.created || window.is_null() || !unsafe { is_ours((*window).api) } {
        return false;
    }
    // SAFETY: the host passes a live window of our API.
    match unsafe { native::View::new(&*window) } {
        Ok(v) => {
            g.view = Some(v);
            true
        }
        Err(e) => {
            eprintln!("daw test plugin: {e}");
            false
        }
    }
}

unsafe extern "C" fn set_transient(_: *const clap_plugin, _: *const clap_window) -> bool {
    false
}

unsafe extern "C" fn suggest_title(_: *const clap_plugin, _: *const c_char) {}

unsafe extern "C" fn show(p: *const clap_plugin) -> bool {
    // SAFETY: a plugin of this library.
    let mut g = unsafe { gui(p) };
    g.view.as_mut().is_some_and(|v| {
        v.set_visible(true);
        true
    })
}

unsafe extern "C" fn hide(p: *const clap_plugin) -> bool {
    // SAFETY: a plugin of this library.
    let mut g = unsafe { gui(p) };
    g.view.as_mut().is_some_and(|v| {
        v.set_visible(false);
        true
    })
}

unsafe extern "C" fn resize_hints(
    _: *const clap_plugin,
    _: *mut clap_sys::ext::gui::clap_gui_resize_hints,
) -> bool {
    false
}

pub static GUI_EXT: clap_plugin_gui = clap_plugin_gui {
    is_api_supported: Some(is_api_supported),
    get_preferred_api: Some(get_preferred_api),
    create: Some(create),
    destroy: Some(destroy),
    set_scale: Some(set_scale),
    get_size: Some(get_size),
    can_resize: Some(can_resize),
    get_resize_hints: Some(resize_hints),
    adjust_size: Some(no_size),
    set_size: Some(set_size),
    set_parent: Some(set_parent),
    set_transient: Some(set_transient),
    suggest_title: Some(suggest_title),
    show: Some(show),
    hide: Some(hide),
};

#[cfg(all(unix, not(target_os = "macos")))]
mod native {
    //! A child window of one colour, on the plugin's own X11 connection (libX11 loaded
    //! at run time).
    use super::{COLOUR, HEIGHT, WIDTH, clap_window};
    use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};

    type Display = c_void;

    #[derive(Debug)]
    pub struct View {
        lib: libloading::Library,
        dpy: *mut Display,
        id: c_ulong,
    }

    // SAFETY: used only on the host's main thread (CLAP's GUI rule); the mutex around it
    // only satisfies the plugin struct's `Sync`.
    unsafe impl Send for View {}

    impl View {
        /// # Safety
        /// `w` must describe a live X11 window.
        pub unsafe fn new(w: &clap_window) -> Result<View, String> {
            // SAFETY: loading libX11 and calling it by its documented signatures.
            unsafe {
                let lib = libloading::Library::new("libX11.so.6")
                    .or_else(|_| libloading::Library::new("libX11.so"))
                    .map_err(|e| e.to_string())?;
                let open: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *mut Display> =
                    lib.get(b"XOpenDisplay\0").map_err(|e| e.to_string())?;
                let create: libloading::Symbol<
                    unsafe extern "C" fn(
                        *mut Display,
                        c_ulong,
                        c_int,
                        c_int,
                        c_uint,
                        c_uint,
                        c_uint,
                        c_ulong,
                        c_ulong,
                    ) -> c_ulong,
                > = lib
                    .get(b"XCreateSimpleWindow\0")
                    .map_err(|e| e.to_string())?;
                let dpy = open(std::ptr::null());
                if dpy.is_null() {
                    return Err("no X11 display".into());
                }
                let id = create(
                    dpy,
                    w.specific.x11,
                    0,
                    0,
                    WIDTH,
                    HEIGHT,
                    0,
                    0,
                    c_ulong::from(COLOUR),
                );
                let v = View { lib, dpy, id };
                v.call(b"XMapWindow\0");
                Ok(v)
            }
        }

        /// Calls `name(display, window)` and flushes.
        fn call(&self, name: &[u8]) {
            // SAFETY: an X call on our own connection and window.
            unsafe {
                if let Ok(f) = self
                    .lib
                    .get::<unsafe extern "C" fn(*mut Display, c_ulong) -> c_int>(name)
                {
                    f(self.dpy, self.id);
                }
                if let Ok(flush) = self
                    .lib
                    .get::<unsafe extern "C" fn(*mut Display) -> c_int>(b"XFlush\0")
                {
                    flush(self.dpy);
                }
            }
        }

        pub fn set_visible(&mut self, on: bool) {
            self.call(if on {
                b"XMapWindow\0"
            } else {
                b"XUnmapWindow\0"
            });
        }
    }

    impl Drop for View {
        fn drop(&mut self) {
            self.call(b"XDestroyWindow\0");
            // SAFETY: closing our own connection last.
            unsafe {
                if let Ok(close) = self
                    .lib
                    .get::<unsafe extern "C" fn(*mut Display) -> c_int>(b"XCloseDisplay\0")
                {
                    close(self.dpy);
                }
            }
        }
    }
}

#[cfg(windows)]
mod native {
    //! A child window of one colour.
    use super::{COLOUR, HEIGHT, WIDTH, clap_window};
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Gdi::CreateSolidBrush;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, SW_HIDE, SW_SHOW,
        ShowWindow, WNDCLASSEXW, WS_CHILD, WS_VISIBLE,
    };

    const CLASS: &[u16] = &[
        b'D' as u16,
        b'a' as u16,
        b'w' as u16,
        b'T' as u16,
        b'e' as u16,
        b's' as u16,
        b't' as u16,
        b'V' as u16,
        b'i' as u16,
        b'e' as u16,
        b'w' as u16,
        0,
    ];

    #[derive(Debug)]
    pub struct View {
        hwnd: HWND,
    }

    // SAFETY: used only on the host's main thread (CLAP's GUI rule).
    unsafe impl Send for View {}

    impl View {
        /// # Safety
        /// `w` must describe a live Win32 window.
        pub unsafe fn new(w: &clap_window) -> Result<View, String> {
            // SAFETY: registering our class (again is harmless) and creating a child of
            // the host's window.
            unsafe {
                // COLORREF is 0x00BBGGRR.
                let bgr = ((COLOUR & 0xFF) << 16) | (COLOUR & 0xFF00) | ((COLOUR >> 16) & 0xFF);
                let wc = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: 0,
                    lpfnWndProc: Some(DefWindowProcW),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: GetModuleHandleW(std::ptr::null()),
                    hIcon: std::ptr::null_mut(),
                    hCursor: std::ptr::null_mut(),
                    hbrBackground: CreateSolidBrush(bgr),
                    lpszMenuName: std::ptr::null(),
                    lpszClassName: CLASS.as_ptr(),
                    hIconSm: std::ptr::null_mut(),
                };
                RegisterClassExW(&wc);
                let hwnd = CreateWindowExW(
                    0,
                    CLASS.as_ptr(),
                    CLASS.as_ptr(),
                    WS_CHILD | WS_VISIBLE,
                    0,
                    0,
                    WIDTH as i32,
                    HEIGHT as i32,
                    w.specific.win32,
                    std::ptr::null_mut(),
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null(),
                );
                if hwnd.is_null() {
                    return Err("cannot create the view window".into());
                }
                Ok(View { hwnd })
            }
        }

        pub fn set_visible(&mut self, on: bool) {
            // SAFETY: our own window.
            unsafe { ShowWindow(self.hwnd, if on { SW_SHOW } else { SW_HIDE }) };
        }
    }

    impl Drop for View {
        fn drop(&mut self) {
            // SAFETY: our own window.
            unsafe { DestroyWindow(self.hwnd) };
        }
    }
}

#[cfg(target_os = "macos")]
mod native {
    //! A layer-backed `NSView` of one colour, through the Objective-C runtime.
    use super::{COLOUR, HEIGHT, WIDTH, clap_window};
    use std::ffi::{CStr, c_char, c_void};

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct NSRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    }

    #[link(name = "objc", kind = "dylib")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
    }

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGColorCreateSRGB(r: f64, g: f64, b: f64, a: f64) -> *mut c_void;
        fn CGColorRelease(c: *mut c_void);
    }

    fn sel(n: &CStr) -> Sel {
        // SAFETY: registering a NUL-terminated selector name.
        unsafe { sel_registerName(n.as_ptr()) }
    }

    /// Sends `name` to `obj` with the argument and return types of `$ty`.
    macro_rules! send {
        ($obj:expr, $name:expr $(, $arg:expr)* ; $ty:ty) => {{
            // SAFETY: `objc_msgSend` called with the signature of the method named, on an
            // object that responds to it.
            unsafe {
                let f: $ty = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
                f($obj, sel($name) $(, $arg)*)
            }
        }};
    }

    #[derive(Debug)]
    pub struct View {
        view: Id,
    }

    // SAFETY: used only on the host's main thread (CLAP's GUI rule).
    unsafe impl Send for View {}

    impl View {
        /// # Safety
        /// `w` must describe a live `NSView`.
        pub unsafe fn new(w: &clap_window) -> Result<View, String> {
            // SAFETY: the parent is the host's live view.
            let parent = unsafe { w.specific.cocoa };
            if parent.is_null() {
                return Err("no parent view".into());
            }
            // SAFETY: looking up a class by name.
            let class = unsafe { objc_getClass(c"NSView".as_ptr()) };
            let alloc = send!(class, c"alloc"; unsafe extern "C" fn(Id, Sel) -> Id);
            let frame = NSRect {
                x: 0.0,
                y: 0.0,
                w: f64::from(WIDTH),
                h: f64::from(HEIGHT),
            };
            let view =
                send!(alloc, c"initWithFrame:", frame; unsafe extern "C" fn(Id, Sel, NSRect) -> Id);
            if view.is_null() {
                return Err("cannot create the view".into());
            }
            send!(view, c"setWantsLayer:", 1i8; unsafe extern "C" fn(Id, Sel, i8));
            let layer = send!(view, c"layer"; unsafe extern "C" fn(Id, Sel) -> Id);
            let c = |shift: u32| f64::from((COLOUR >> shift) & 0xFF) / 255.0;
            // SAFETY: creating a colour; the layer retains it, and ours is released.
            let colour = unsafe { CGColorCreateSRGB(c(16), c(8), c(0), 1.0) };
            send!(layer, c"setBackgroundColor:", colour; unsafe extern "C" fn(Id, Sel, *mut c_void));
            // SAFETY: releasing the colour created above.
            unsafe { CGColorRelease(colour) };
            send!(parent, c"addSubview:", view; unsafe extern "C" fn(Id, Sel, Id));
            Ok(View { view })
        }

        pub fn set_visible(&mut self, on: bool) {
            send!(self.view, c"setHidden:", i8::from(!on); unsafe extern "C" fn(Id, Sel, i8));
        }
    }

    impl Drop for View {
        fn drop(&mut self) {
            send!(self.view, c"removeFromSuperview"; unsafe extern "C" fn(Id, Sel));
            send!(self.view, c"release"; unsafe extern "C" fn(Id, Sel));
        }
    }
}
