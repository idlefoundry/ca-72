//! The test plugin's editor (M6 plan W5): a view of one colour embedded in the host's
//! window (X11, Win32 or Cocoa), the CLAP test plugin's view behind VST3's `IPlugView`. It
//! draws nothing of its own, so it needs no event loop: the window system paints its
//! background. That is enough to test opening, moving and closing a plugin's editor.

use std::ffi::{CStr, c_char, c_void};
use std::sync::Mutex;
use vst3::Steinberg::*;
use vst3::{Class, ComPtr};

/// The view's size, fixed.
pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 160;

/// The view's colour, `0xRRGGBB`.
const COLOUR: u32 = 0x003a_7bd5;

/// The platform type the view embeds into (VST3's `kPlatformType…`).
#[cfg(all(unix, not(target_os = "macos")))]
pub const PLATFORM: &CStr = c"X11EmbedWindowID";
#[cfg(windows)]
pub const PLATFORM: &CStr = c"HWND";
#[cfg(target_os = "macos")]
pub const PLATFORM: &CStr = c"NSView";

/// The editor: its native view while attached, and the host's frame.
#[derive(Default)]
pub struct Editor {
    view: Mutex<Option<native::View>>,
    frame: Mutex<Option<ComPtr<IPlugFrame>>>,
}

impl std::fmt::Debug for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Editor").finish_non_exhaustive()
    }
}

impl Class for Editor {
    type Interfaces = (IPlugView,);
}

/// # Safety
/// `t` must be null or a NUL-terminated string.
unsafe fn is_ours(t: FIDString) -> bool {
    // SAFETY: as the caller guarantees.
    !t.is_null() && unsafe { CStr::from_ptr(t as *const c_char) } == PLATFORM
}

impl IPlugViewTrait for Editor {
    unsafe fn isPlatformTypeSupported(&self, t: FIDString) -> tresult {
        // SAFETY: the host passes a NUL-terminated platform type.
        if unsafe { is_ours(t) } {
            kResultTrue
        } else {
            kResultFalse
        }
    }

    unsafe fn attached(&self, parent: *mut c_void, t: FIDString) -> tresult {
        // SAFETY: the host passes a NUL-terminated platform type.
        if parent.is_null() || !unsafe { is_ours(t) } {
            return kResultFalse;
        }
        // SAFETY: the host passes a live parent of our platform type.
        match unsafe { native::View::new(parent) } {
            Ok(v) => {
                if let Ok(mut g) = self.view.lock() {
                    *g = Some(v);
                }
                kResultOk
            }
            Err(e) => {
                eprintln!("daw test vst3: {e}");
                kResultFalse
            }
        }
    }

    unsafe fn removed(&self) -> tresult {
        if let Ok(mut g) = self.view.lock() {
            *g = None;
        }
        kResultOk
    }

    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyDown(&self, _key: char16, _code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyUp(&self, _key: char16, _code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult {
        // SAFETY: the host passes a rectangle to fill, or null.
        let Some(r) = (unsafe { size.as_mut() }) else {
            return kInvalidArgument;
        };
        *r = ViewRect {
            left: 0,
            top: 0,
            right: WIDTH as int32,
            bottom: HEIGHT as int32,
        };
        kResultOk
    }

    unsafe fn onSize(&self, _size: *mut ViewRect) -> tresult {
        kResultOk
    }

    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setFrame(&self, frame: *mut IPlugFrame) -> tresult {
        if let Ok(mut f) = self.frame.lock() {
            // SAFETY: the host's frame, or null; we hold a reference until replaced.
            *f = unsafe { vst3::ComRef::from_raw(frame) }.map(|r| r.to_com_ptr());
        }
        kResultOk
    }

    unsafe fn canResize(&self) -> tresult {
        kResultFalse
    }

    unsafe fn checkSizeConstraint(&self, rect: *mut ViewRect) -> tresult {
        // SAFETY: the host passes a rectangle to adjust, or null.
        if let Some(r) = unsafe { rect.as_mut() } {
            r.right = r.left + WIDTH as int32;
            r.bottom = r.top + HEIGHT as int32;
        }
        kResultOk
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod native {
    //! A child window of one colour, on the plugin's own X11 connection (libX11 loaded
    //! at run time).
    use super::{COLOUR, HEIGHT, WIDTH};
    use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void};

    type Display = c_void;

    #[derive(Debug)]
    pub struct View {
        lib: libloading::Library,
        dpy: *mut Display,
        id: c_ulong,
    }

    // SAFETY: used only on the host's main thread (VST3's GUI rule); the mutex around it
    // only satisfies the plugin struct's `Sync`.
    unsafe impl Send for View {}

    impl View {
        /// # Safety
        /// `parent` must be a live X11 window's ID.
        pub unsafe fn new(parent: *mut std::ffi::c_void) -> Result<View, String> {
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
                    parent as c_ulong,
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
    use super::{COLOUR, HEIGHT, WIDTH};
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Gdi::CreateSolidBrush;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, WNDCLASSEXW, WS_CHILD,
        WS_VISIBLE,
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

    // SAFETY: used only on the host's main thread (VST3's GUI rule).
    unsafe impl Send for View {}

    impl View {
        /// # Safety
        /// `parent` must be a live Win32 window.
        pub unsafe fn new(parent: *mut std::ffi::c_void) -> Result<View, String> {
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
                    parent as HWND,
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
    use super::{COLOUR, HEIGHT, WIDTH};
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

    // SAFETY: used only on the host's main thread (VST3's GUI rule).
    unsafe impl Send for View {}

    impl View {
        /// # Safety
        /// `parent` must be a live `NSView`.
        pub unsafe fn new(parent: *mut c_void) -> Result<View, String> {
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
    }

    impl Drop for View {
        fn drop(&mut self) {
            send!(self.view, c"removeFromSuperview"; unsafe extern "C" fn(Id, Sel));
            send!(self.view, c"release"; unsafe extern "C" fn(Id, Sel));
        }
    }
}
