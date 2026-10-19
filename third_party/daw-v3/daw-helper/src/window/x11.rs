//! X11 windows, through libX11 loaded at run time.

use super::{Check, WindowEvent};
use clap_sys::ext::gui::clap_window_handle;
use std::collections::HashMap;
use std::ffi::{CString, c_char, c_int, c_long, c_uint, c_ulong, c_void};
use std::sync::{Mutex, OnceLock};

type Display = c_void;
type Xid = c_ulong;
type Atom = c_ulong;

const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;
const CONFIGURE_NOTIFY: c_int = 22;
const CLIENT_MESSAGE: c_int = 33;
const IS_VIEWABLE: c_int = 2;
const XA_ATOM: Atom = 4;
const PROP_MODE_REPLACE: c_int = 0;
const US_POSITION: c_long = 1;
const P_POSITION: c_long = 4;
const P_MIN_SIZE: c_long = 16;
const P_MAX_SIZE: c_long = 32;

#[repr(C)]
struct XEvent {
    data: [c_long; 24],
}

#[repr(C)]
struct XClientMessageEvent {
    kind: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    window: Xid,
    message_type: Atom,
    format: c_int,
    data: [c_long; 5],
}

#[repr(C)]
struct XConfigureEvent {
    kind: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: *mut Display,
    event: Xid,
    window: Xid,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    border_width: c_int,
    above: Xid,
    override_redirect: c_int,
}

#[repr(C)]
#[derive(Default)]
struct XSizeHints {
    flags: c_long,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    min_width: c_int,
    min_height: c_int,
    max_width: c_int,
    max_height: c_int,
    width_inc: c_int,
    height_inc: c_int,
    min_aspect: [c_int; 2],
    max_aspect: [c_int; 2],
    base_width: c_int,
    base_height: c_int,
    win_gravity: c_int,
}

#[repr(C)]
struct XWindowAttributes {
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    border_width: c_int,
    depth: c_int,
    visual: *mut c_void,
    root: Xid,
    class: c_int,
    bit_gravity: c_int,
    win_gravity: c_int,
    backing_store: c_int,
    backing_planes: c_ulong,
    backing_pixel: c_ulong,
    save_under: c_int,
    colormap: Xid,
    map_installed: c_int,
    map_state: c_int,
    all_event_masks: c_long,
    your_event_mask: c_long,
    do_not_propagate_mask: c_long,
    override_redirect: c_int,
    screen: *mut c_void,
}

type ErrorHandler = unsafe extern "C" fn(*mut Display, *mut c_void) -> c_int;

/// The libX11 functions used, and the helper's one connection.
struct X {
    _lib: libloading::Library,
    dpy: *mut Display,
    root: Xid,
    delete: Atom,
    protocols: Atom,
    wm_type: Atom,
    dialog: Atom,
    create_simple_window: unsafe extern "C" fn(
        *mut Display,
        Xid,
        c_int,
        c_int,
        c_uint,
        c_uint,
        c_uint,
        c_ulong,
        c_ulong,
    ) -> Xid,
    store_name: unsafe extern "C" fn(*mut Display, Xid, *const c_char) -> c_int,
    set_wm_protocols: unsafe extern "C" fn(*mut Display, Xid, *mut Atom, c_int) -> c_int,
    select_input: unsafe extern "C" fn(*mut Display, Xid, c_long) -> c_int,
    map_window: unsafe extern "C" fn(*mut Display, Xid) -> c_int,
    unmap_window: unsafe extern "C" fn(*mut Display, Xid) -> c_int,
    move_window: unsafe extern "C" fn(*mut Display, Xid, c_int, c_int) -> c_int,
    resize_window: unsafe extern "C" fn(*mut Display, Xid, c_uint, c_uint) -> c_int,
    destroy_window: unsafe extern "C" fn(*mut Display, Xid) -> c_int,
    change_property: unsafe extern "C" fn(
        *mut Display,
        Xid,
        Atom,
        Atom,
        c_int,
        c_int,
        *const u8,
        c_int,
    ) -> c_int,
    set_wm_normal_hints: unsafe extern "C" fn(*mut Display, Xid, *mut XSizeHints),
    get_window_attributes: unsafe extern "C" fn(*mut Display, Xid, *mut XWindowAttributes) -> c_int,
    query_tree: unsafe extern "C" fn(
        *mut Display,
        Xid,
        *mut Xid,
        *mut Xid,
        *mut *mut Xid,
        *mut c_uint,
    ) -> c_int,
    translate_coordinates: unsafe extern "C" fn(
        *mut Display,
        Xid,
        Xid,
        c_int,
        c_int,
        *mut c_int,
        *mut c_int,
        *mut Xid,
    ) -> c_int,
    free: unsafe extern "C" fn(*mut c_void) -> c_int,
    flush: unsafe extern "C" fn(*mut Display) -> c_int,
    sync: unsafe extern "C" fn(*mut Display, c_int) -> c_int,
    pending: unsafe extern "C" fn(*mut Display) -> c_int,
    next_event: unsafe extern "C" fn(*mut Display, *mut XEvent) -> c_int,
}

// SAFETY: the connection is used only from the helper's main thread; `OnceLock` needs
// the value to be shareable, and nothing else ever touches it.
unsafe impl Send for X {}
// SAFETY: as above.
unsafe impl Sync for X {}

/// An X error (a window gone under us, say) is logged: libX11's default handler would
/// exit the helper.
unsafe extern "C" fn on_error(_: *mut Display, e: *mut c_void) -> c_int {
    // SAFETY: libX11 passes a valid `XErrorEvent`; its error code is the byte at offset 32
    // on LP64 (type, display, resourceid, serial, then error_code).
    let code = unsafe { *(e as *const u8).add(4 * std::mem::size_of::<c_long>()) };
    eprintln!("X error {code} (ignored)");
    0
}

fn x() -> Result<&'static X, String> {
    static X_: OnceLock<Result<X, String>> = OnceLock::new();
    X_.get_or_init(|| {
        // SAFETY: loading the system's libX11 and looking up functions by their
        // documented signatures.
        unsafe {
            let lib = libloading::Library::new("libX11.so.6")
                .or_else(|_| libloading::Library::new("libX11.so"))
                .map_err(|e| format!("plugin windows need X11 (libX11): {e}"))?;
            macro_rules! f {
                ($name:literal) => {
                    *lib.get(concat!($name, "\0").as_bytes())
                        .map_err(|e| format!("libX11: {}: {e}", $name))?
                };
            }
            let open: unsafe extern "C" fn(*const c_char) -> *mut Display = f!("XOpenDisplay");
            let set_error: unsafe extern "C" fn(Option<ErrorHandler>) -> Option<ErrorHandler> =
                f!("XSetErrorHandler");
            let default_screen: unsafe extern "C" fn(*mut Display) -> c_int = f!("XDefaultScreen");
            let root_window: unsafe extern "C" fn(*mut Display, c_int) -> Xid = f!("XRootWindow");
            let intern: unsafe extern "C" fn(*mut Display, *const c_char, c_int) -> Atom =
                f!("XInternAtom");
            let dpy = open(std::ptr::null());
            if dpy.is_null() {
                return Err(
                    "plugin windows need an X11 display (DISPLAY is not set, or XWayland is not running)"
                        .into(),
                );
            }
            set_error(Some(on_error));
            let root = root_window(dpy, default_screen(dpy));
            Ok(X {
                dpy,
                root,
                delete: intern(dpy, c"WM_DELETE_WINDOW".as_ptr(), 0),
                protocols: intern(dpy, c"WM_PROTOCOLS".as_ptr(), 0),
                wm_type: intern(dpy, c"_NET_WM_WINDOW_TYPE".as_ptr(), 0),
                dialog: intern(dpy, c"_NET_WM_WINDOW_TYPE_DIALOG".as_ptr(), 0),
                create_simple_window: f!("XCreateSimpleWindow"),
                store_name: f!("XStoreName"),
                set_wm_protocols: f!("XSetWMProtocols"),
                select_input: f!("XSelectInput"),
                map_window: f!("XMapWindow"),
                unmap_window: f!("XUnmapWindow"),
                move_window: f!("XMoveWindow"),
                resize_window: f!("XResizeWindow"),
                destroy_window: f!("XDestroyWindow"),
                change_property: f!("XChangeProperty"),
                set_wm_normal_hints: f!("XSetWMNormalHints"),
                get_window_attributes: f!("XGetWindowAttributes"),
                query_tree: f!("XQueryTree"),
                translate_coordinates: f!("XTranslateCoordinates"),
                free: f!("XFree"),
                flush: f!("XFlush"),
                sync: f!("XSync"),
                pending: f!("XPending"),
                next_event: f!("XNextEvent"),
                _lib: lib,
            })
        }
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// Events read by [`pump`], by window, until each window takes its own.
fn queued() -> &'static Mutex<HashMap<Xid, Vec<WindowEvent>>> {
    static Q: OnceLock<Mutex<HashMap<Xid, Vec<WindowEvent>>>> = OnceLock::new();
    Q.get_or_init(Mutex::default)
}

/// Reads what the X server sent: close requests and new sizes.
pub fn pump() {
    let Some(x) = X_LOADED.get().and_then(|_| x().ok()) else {
        return;
    };
    let mut q = queued().lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: reading events from the helper's connection on its main thread; the event
    // is read as the struct its type names.
    unsafe {
        while (x.pending)(x.dpy) > 0 {
            let mut e = XEvent { data: [0; 24] };
            (x.next_event)(x.dpy, &mut e);
            let kind = *(e.data.as_ptr() as *const c_int);
            if kind == CLIENT_MESSAGE {
                let m = &*(&e as *const XEvent as *const XClientMessageEvent);
                if m.message_type == x.protocols && m.data[0] as Atom == x.delete {
                    q.entry(m.window).or_default().push(WindowEvent::Closed);
                }
            } else if kind == CONFIGURE_NOTIFY {
                let c = &*(&e as *const XEvent as *const XConfigureEvent);
                if c.width > 0 && c.height > 0 {
                    q.entry(c.window)
                        .or_default()
                        .push(WindowEvent::Resized(c.width as u32, c.height as u32));
                }
            }
        }
    }
}

/// Set once a window has been made: [`pump`] does nothing before (and never loads
/// libX11 for a helper that shows no GUI).
static X_LOADED: OnceLock<()> = OnceLock::new();

/// A top-level X11 window.
#[derive(Debug)]
pub struct Window {
    id: Xid,
    size: (u32, u32),
}

impl Window {
    /// A window of `width` by `height` titled `title`, at `at` on the screen if given;
    /// fixed in size unless `resizable`. Not yet shown.
    pub fn create(
        title: &str,
        width: u32,
        height: u32,
        at: Option<(i32, i32)>,
        resizable: bool,
    ) -> Result<Window, String> {
        let x = x()?;
        let _ = X_LOADED.set(());
        let title = CString::new(title.replace('\0', " ")).unwrap_or_default();
        let (px, py) = at.unwrap_or((0, 0));
        // SAFETY: X calls on the helper's connection, main thread.
        unsafe {
            let id = (x.create_simple_window)(
                x.dpy,
                x.root,
                px,
                py,
                width.max(1),
                height.max(1),
                0,
                0,
                0x0020_2020,
            );
            if id == 0 {
                return Err("X11: cannot create a window".into());
            }
            (x.store_name)(x.dpy, id, title.as_ptr());
            let mut delete = x.delete;
            (x.set_wm_protocols)(x.dpy, id, &mut delete, 1);
            (x.select_input)(x.dpy, id, STRUCTURE_NOTIFY_MASK);
            // A dialog: tiling window managers float it.
            let dialog: [c_long; 1] = [x.dialog as c_long];
            (x.change_property)(
                x.dpy,
                id,
                x.wm_type,
                XA_ATOM,
                32,
                PROP_MODE_REPLACE,
                dialog.as_ptr() as *const u8,
                1,
            );
            let mut hints = XSizeHints {
                flags: if at.is_some() {
                    US_POSITION | P_POSITION
                } else {
                    0
                },
                x: px,
                y: py,
                width: width as c_int,
                height: height as c_int,
                ..XSizeHints::default()
            };
            if !resizable {
                hints.flags |= P_MIN_SIZE | P_MAX_SIZE;
                hints.min_width = width as c_int;
                hints.max_width = width as c_int;
                hints.min_height = height as c_int;
                hints.max_height = height as c_int;
            }
            (x.set_wm_normal_hints)(x.dpy, id, &mut hints);
            (x.flush)(x.dpy);
            Ok(Window {
                id,
                size: (width, height),
            })
        }
    }

    /// What a plugin embeds into.
    pub fn handle(&self) -> clap_window_handle {
        clap_window_handle { x11: self.id }
    }

    /// The window's X11 ID, for messages.
    pub fn id(&self) -> u64 {
        // `c_ulong` is 32 bits on some Unix systems.
        #[allow(clippy::useless_conversion)]
        u64::from(self.id)
    }

    pub fn show(&mut self) {
        if let Ok(x) = x() {
            // SAFETY: X calls on the helper's connection, main thread.
            unsafe {
                (x.map_window)(x.dpy, self.id);
                (x.sync)(x.dpy, 0);
            }
        }
    }

    pub fn hide(&mut self) {
        if let Ok(x) = x() {
            // SAFETY: as above.
            unsafe {
                (x.unmap_window)(x.dpy, self.id);
                (x.flush)(x.dpy);
            }
        }
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        if let Ok(x) = x() {
            // SAFETY: as above.
            unsafe {
                (x.resize_window)(x.dpy, self.id, width.max(1), height.max(1));
                (x.flush)(x.dpy);
            }
        }
    }

    pub fn move_to(&self, px: i32, py: i32) {
        if let Ok(x) = x() {
            // SAFETY: as above.
            unsafe {
                (x.move_window)(x.dpy, self.id, px, py);
                (x.flush)(x.dpy);
            }
        }
    }

    /// Where the window's content is on the screen.
    pub fn position(&self) -> Option<(i32, i32)> {
        let x = x().ok()?;
        let (mut px, mut py, mut child) = (0, 0, 0);
        // SAFETY: as above.
        let ok = unsafe {
            (x.translate_coordinates)(x.dpy, self.id, x.root, 0, 0, &mut px, &mut py, &mut child)
        };
        (ok != 0).then_some((px, py))
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Whether it is on screen, and what is inside it.
    pub fn check(&self) -> Check {
        let Ok(x) = x() else {
            return Check::default();
        };
        // SAFETY: X calls on the helper's connection, main thread; `XQueryTree`'s list is
        // freed with `XFree`.
        unsafe {
            (x.sync)(x.dpy, 0);
            let mut a: XWindowAttributes = std::mem::zeroed();
            let visible = (x.get_window_attributes)(x.dpy, self.id, &mut a) != 0
                && a.map_state == IS_VIEWABLE;
            let (mut root, mut parent, mut kids, mut n) = (0, 0, std::ptr::null_mut(), 0);
            let children =
                if (x.query_tree)(x.dpy, self.id, &mut root, &mut parent, &mut kids, &mut n) != 0 {
                    if !kids.is_null() {
                        (x.free)(kids as *mut c_void);
                    }
                    n as usize
                } else {
                    0
                };
            Check { visible, children }
        }
    }

    /// What happened to it since the last call (after [`pump`]).
    pub fn events(&mut self) -> Vec<WindowEvent> {
        let mut q = queued().lock().unwrap_or_else(|e| e.into_inner());
        let events = q.remove(&self.id).unwrap_or_default();
        for e in &events {
            if let WindowEvent::Resized(w, h) = e {
                self.size = (*w, *h);
            }
        }
        events
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if let Ok(x) = x() {
            // SAFETY: destroying our own window on the helper's connection, main thread.
            unsafe {
                (x.destroy_window)(x.dpy, self.id);
                (x.flush)(x.dpy);
            }
        }
        queued()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.id);
    }
}
