//! AppKit windows, through the Objective-C runtime. The helper becomes an accessory
//! application (no Dock icon) when its first window opens.

use super::{Check, WindowEvent};
use clap_sys::ext::gui::clap_window_handle;
use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::OnceLock;

type Id = *mut c_void;
type Sel = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    static NSDefaultRunLoopMode: Id;
}

const TITLED: u64 = 1;
const CLOSABLE: u64 = 2;
const MINIATURIZABLE: u64 = 4;
const RESIZABLE: u64 = 8;
const BACKING_BUFFERED: u64 = 2;
const ACCESSORY: i64 = 1;

fn class(name: &CStr) -> Id {
    // SAFETY: looking up a class by its NUL-terminated name.
    unsafe { objc_getClass(name.as_ptr()) }
}

fn sel(name: &CStr) -> Sel {
    // SAFETY: registering a NUL-terminated selector name.
    unsafe { sel_registerName(name.as_ptr()) }
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

/// A method returning a rectangle. On arm64 it is an ordinary send; on x86-64 a struct
/// this large comes back through `objc_msgSend_stret`.
fn rect(obj: Id, name: &CStr) -> NSRect {
    #[cfg(target_arch = "aarch64")]
    {
        send!(obj, name; unsafe extern "C" fn(Id, Sel) -> NSRect)
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        #[link(name = "objc", kind = "dylib")]
        unsafe extern "C" {
            fn objc_msgSend_stret();
        }
        let mut r = NSRect::default();
        // SAFETY: the stret variant writes the rectangle through its first argument.
        unsafe {
            let f: unsafe extern "C" fn(*mut NSRect, Id, Sel) =
                std::mem::transmute(objc_msgSend_stret as unsafe extern "C" fn());
            f(&mut r, obj, sel(name));
        }
        r
    }
}

struct Pool(*mut c_void);

impl Pool {
    fn new() -> Pool {
        // SAFETY: pushing an autorelease pool on this thread.
        Pool(unsafe { objc_autoreleasePoolPush() })
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        // SAFETY: popping the pool pushed by `new`, on the same thread.
        unsafe { objc_autoreleasePoolPop(self.0) }
    }
}

/// The shared application, set up once.
fn app() -> Id {
    struct App(usize);
    static APP: OnceLock<App> = OnceLock::new();
    APP.get_or_init(|| {
        let app = send!(class(c"NSApplication"), c"sharedApplication"; unsafe extern "C" fn(Id, Sel) -> Id);
        send!(app, c"setActivationPolicy:", ACCESSORY; unsafe extern "C" fn(Id, Sel, i64) -> i8);
        send!(app, c"finishLaunching"; unsafe extern "C" fn(Id, Sel));
        App(app as usize)
    })
    .0 as Id
}

static STARTED: OnceLock<()> = OnceLock::new();

/// Runs the application's pending events without blocking.
pub fn pump() {
    if STARTED.get().is_none() {
        return;
    }
    let _pool = Pool::new();
    let app = app();
    let past = send!(class(c"NSDate"), c"distantPast"; unsafe extern "C" fn(Id, Sel) -> Id);
    loop {
        // SAFETY: reading Foundation's run loop mode constant.
        let mode = unsafe { NSDefaultRunLoopMode };
        let ev = send!(app, c"nextEventMatchingMask:untilDate:inMode:dequeue:", u64::MAX, past, mode, 1i8;
            unsafe extern "C" fn(Id, Sel, u64, Id, Id, i8) -> Id);
        if ev.is_null() {
            break;
        }
        send!(app, c"sendEvent:", ev; unsafe extern "C" fn(Id, Sel, Id));
    }
    send!(app, c"updateWindows"; unsafe extern "C" fn(Id, Sel));
}

/// The main screen's height, to turn top-left coordinates into AppKit's bottom-left ones.
fn screen_height() -> f64 {
    let screen = send!(class(c"NSScreen"), c"mainScreen"; unsafe extern "C" fn(Id, Sel) -> Id);
    if screen.is_null() {
        return 0.0;
    }
    rect(screen, c"frame").size.height
}

/// A top-level AppKit window.
#[derive(Debug)]
pub struct Window {
    window: Id,
    view: Id,
    size: (u32, u32),
    shown: bool,
}

impl Window {
    /// A window whose content is `width` by `height` points, titled `title`, with its
    /// top-left corner at `at` on the screen if given; fixed in size unless `resizable`.
    /// Not yet shown.
    pub fn create(
        title: &str,
        width: u32,
        height: u32,
        at: Option<(i32, i32)>,
        resizable: bool,
    ) -> Result<Window, String> {
        let _pool = Pool::new();
        app();
        let _ = STARTED.set(());
        let content = NSRect {
            origin: NSPoint::default(),
            size: NSSize {
                width: f64::from(width),
                height: f64::from(height),
            },
        };
        let mut mask = TITLED | CLOSABLE | MINIATURIZABLE;
        if resizable {
            mask |= RESIZABLE;
        }
        let alloc = send!(class(c"NSWindow"), c"alloc"; unsafe extern "C" fn(Id, Sel) -> Id);
        let window = send!(alloc, c"initWithContentRect:styleMask:backing:defer:", content, mask, BACKING_BUFFERED, 0i8;
            unsafe extern "C" fn(Id, Sel, NSRect, u64, u64, i8) -> Id);
        if window.is_null() {
            return Err("AppKit: cannot create a window".into());
        }
        send!(window, c"setReleasedWhenClosed:", 0i8; unsafe extern "C" fn(Id, Sel, i8));
        let t = CString::new(title.replace('\0', " ")).unwrap_or_default();
        let s = send!(class(c"NSString"), c"stringWithUTF8String:", t.as_ptr(); unsafe extern "C" fn(Id, Sel, *const c_char) -> Id);
        send!(window, c"setTitle:", s; unsafe extern "C" fn(Id, Sel, Id));
        let w = Window {
            window,
            view: send!(window, c"contentView"; unsafe extern "C" fn(Id, Sel) -> Id),
            size: (width, height),
            shown: false,
        };
        match at {
            Some((x, y)) => w.move_to(x, y),
            None => send!(window, c"center"; unsafe extern "C" fn(Id, Sel)),
        }
        Ok(w)
    }

    pub fn handle(&self) -> clap_window_handle {
        clap_window_handle { cocoa: self.view }
    }

    pub fn id(&self) -> u64 {
        send!(self.window, c"windowNumber"; unsafe extern "C" fn(Id, Sel) -> i64) as u64
    }

    pub fn show(&mut self) {
        send!(self.window, c"makeKeyAndOrderFront:", std::ptr::null_mut::<c_void>(); unsafe extern "C" fn(Id, Sel, Id));
        self.shown = true;
    }

    pub fn hide(&mut self) {
        send!(self.window, c"orderOut:", std::ptr::null_mut::<c_void>(); unsafe extern "C" fn(Id, Sel, Id));
        self.shown = false;
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        let s = NSSize {
            width: f64::from(width),
            height: f64::from(height),
        };
        send!(self.window, c"setContentSize:", s; unsafe extern "C" fn(Id, Sel, NSSize));
    }

    /// Puts the window's top-left corner at (`x`, `y`), from the main screen's top left.
    pub fn move_to(&self, x: i32, y: i32) {
        let p = NSPoint {
            x: f64::from(x),
            y: screen_height() - f64::from(y),
        };
        send!(self.window, c"setFrameTopLeftPoint:", p; unsafe extern "C" fn(Id, Sel, NSPoint));
    }

    /// The window's top-left corner, from the main screen's top left.
    pub fn position(&self) -> Option<(i32, i32)> {
        let f = rect(self.window, c"frame");
        Some((
            f.origin.x.round() as i32,
            (screen_height() - f.origin.y - f.size.height).round() as i32,
        ))
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn check(&self) -> Check {
        let visible = send!(self.window, c"isVisible"; unsafe extern "C" fn(Id, Sel) -> i8) != 0;
        let subviews = send!(self.view, c"subviews"; unsafe extern "C" fn(Id, Sel) -> Id);
        let children = if subviews.is_null() {
            0
        } else {
            send!(subviews, c"count"; unsafe extern "C" fn(Id, Sel) -> u64) as usize
        };
        Check { visible, children }
    }

    /// A close is seen as the window going off screen while shown; a resize as a new
    /// content size.
    pub fn events(&mut self) -> Vec<WindowEvent> {
        let mut events = Vec::new();
        if self.shown && !self.check().visible {
            self.shown = false;
            events.push(WindowEvent::Closed);
        }
        let f = rect(self.view, c"frame");
        let size = (f.size.width.round() as u32, f.size.height.round() as u32);
        if size != self.size && size.0 > 0 && size.1 > 0 {
            self.size = size;
            events.push(WindowEvent::Resized(size.0, size.1));
        }
        events
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _pool = Pool::new();
        send!(self.window, c"orderOut:", std::ptr::null_mut::<c_void>(); unsafe extern "C" fn(Id, Sel, Id));
        send!(self.window, c"release"; unsafe extern "C" fn(Id, Sel));
    }
}
