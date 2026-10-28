//! The typed keys on a real X server, its auto-repeat on (decisions.md R-X11-REPEAT): X held,
//! then A held with it, each past the server's delay, then let go; then A struck twice quickly.
//! The keys come through baseview's X11 backend to [`Playing::key`], as the editor gives them.
//! Before baseview's change 14 the server's repeats came as a release and a press each: A was
//! struck again some 25 times a second and X stepped up an octave as often.
//!
//! It needs an X server (`DISPLAY`; under Wayland, XWayland's) with its auto-repeat on, and opens
//! only its own windows, off the screen and unmanaged; its window's own connection takes the
//! keyboard while the keys are held (an active grab: no focus is asked of a window manager):
//! `xvfb-run -a cargo test -p ca72-plugin --lib playing::x11_tests -- --ignored`.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use baseview::{
    Event, EventStatus, Size, Window, WindowHandler, WindowOpenOptions, WindowScalePolicy,
};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle, XcbWindowHandle,
};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AutoRepeatMode, ConnectionExt as _, CreateWindowAux, KEY_PRESS_EVENT, KEY_RELEASE_EVENT,
    WindowClass,
};
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use super::Playing;

/// A and X on an evdev keymap (Xvfb's).
const A: u8 = 38;
const X: u8 = 53;

/// What the keys did: a note struck or let go, or the typed keys' octave.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Did {
    Note(u8, bool),
    Root(u8),
}

struct Typist {
    playing: Playing,
    did: Sender<Did>,
    close: Receiver<()>,
}

impl WindowHandler for Typist {
    fn on_frame(&mut self, window: &mut Window) {
        if self.close.try_recv().is_ok() {
            window.close();
        }
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        let Event::Keyboard(k) = event else {
            return EventStatus::Ignored;
        };
        let root = self.playing.typing_root;
        self.playing.key(&k, true);
        if self.playing.typing_root != root {
            let _ = self.did.send(Did::Root(self.playing.typing_root));
        }
        while let Some(e) = self.playing.notes.pop() {
            let _ = self.did.send(Did::Note(e.note, e.on));
        }
        EventStatus::Captured
    }
}

/// The host's window the test's opens in, as baseview takes it.
struct Parent(u32);

// SAFETY: the host's window outlives the window opened in it.
#[allow(unsafe_code)]
unsafe impl HasRawWindowHandle for Parent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = XcbWindowHandle::empty();
        handle.window = self.0;
        RawWindowHandle::Xcb(handle)
    }
}

/// The keyboard taken for `window` by baseview's own connection, `dpy`, whatever has the focus.
#[allow(unsafe_code)]
fn grab_keyboard(dpy: *mut x11::xlib::Display, window: u32) {
    use x11::xlib;
    // SAFETY: baseview's own display, open for as long as its window, on its window's thread.
    unsafe {
        let grabbed = xlib::XGrabKeyboard(
            dpy,
            u64::from(window),
            xlib::False,
            xlib::GrabModeAsync,
            xlib::GrabModeAsync,
            xlib::CurrentTime,
        );
        assert_eq!(
            grabbed,
            xlib::GrabSuccess,
            "the keyboard taken for the window"
        );
        xlib::XFlush(dpy);
    }
}

fn key(conn: &RustConnection, root: u32, keycode: u8, down: bool) {
    let kind = if down {
        KEY_PRESS_EVENT
    } else {
        KEY_RELEASE_EVENT
    };
    conn.xtest_fake_input(kind, keycode, 0, root, 0, 0, 0)
        .expect("XTEST takes the key");
    conn.sync().expect("the X server answers");
}

#[test]
#[ignore = "requires an X11 display; creates only its own off-screen windows"]
fn a_typed_key_held_through_the_servers_repeats_strikes_once_and_x_steps_once() {
    let (conn, screen) = x11rb::connect(None).expect("an X11 display (DISPLAY)");
    let control = conn.get_keyboard_control().unwrap().reply().unwrap();
    assert!(
        control.global_auto_repeat == AutoRepeatMode::ON,
        "the X server's auto-repeat is off (`xset r on`)"
    );
    let root = conn.setup().roots[screen].root;
    let host = conn.generate_id().unwrap();
    conn.create_window(
        x11rb::COPY_DEPTH_FROM_PARENT,
        host,
        root,
        -16000,
        -16000,
        200,
        100,
        0,
        WindowClass::INPUT_OUTPUT,
        x11rb::COPY_FROM_PARENT,
        &CreateWindowAux::new().override_redirect(1),
    )
    .unwrap();
    conn.map_window(host).unwrap();
    conn.sync().unwrap();

    let (window_tx, window) = mpsc::channel();
    let (did_tx, did) = mpsc::channel();
    let (close_tx, close) = mpsc::channel();
    let options = WindowOpenOptions {
        title: "playing x11_tests".into(),
        size: Size::new(200.0, 100.0),
        scale: WindowScalePolicy::ScaleFactor(1.0),
    };
    let mut handle = Window::open_parented(&Parent(host), options, move |window| {
        let id = match window.raw_window_handle() {
            RawWindowHandle::Xlib(h) => h.window as u32,
            RawWindowHandle::Xcb(h) => h.window,
            h => panic!("not an X11 window: {h:?}"),
        };
        let RawDisplayHandle::Xlib(display) = window.raw_display_handle() else {
            panic!("not an Xlib display");
        };
        grab_keyboard(display.display.cast(), id);
        window_tx.send(id).unwrap();
        Typist {
            playing: Playing::default(),
            did: did_tx,
            close,
        }
    });
    window
        .recv_timeout(Duration::from_secs(10))
        .expect("the window opened");
    thread::sleep(Duration::from_millis(300));

    // X held past the server's delay (660 ms on Xvfb), then A with it: the server repeats the
    // last key pressed. Then A struck twice, 30 ms apart.
    key(&conn, root, X, true);
    thread::sleep(Duration::from_millis(1200));
    key(&conn, root, A, true);
    thread::sleep(Duration::from_millis(1200));
    key(&conn, root, A, false);
    key(&conn, root, X, false);
    for down in [true, false, true, false] {
        thread::sleep(Duration::from_millis(30));
        key(&conn, root, A, down);
    }
    thread::sleep(Duration::from_millis(400));
    let _ = close_tx.send(());
    thread::sleep(Duration::from_millis(100));
    handle.close();
    conn.destroy_window(host).unwrap();
    conn.sync().unwrap();

    let did: Vec<Did> = did.try_iter().collect();
    let strikes = did
        .iter()
        .filter(|d| matches!(d, Did::Note(_, true)))
        .count();
    let steps = did.iter().filter(|d| matches!(d, Did::Root(_))).count();
    assert_eq!(
        did,
        [
            Did::Root(60),
            Did::Note(60, true),
            Did::Note(60, false),
            Did::Note(60, true),
            Did::Note(60, false),
            Did::Note(60, true),
            Did::Note(60, false),
        ],
        "X stepped once ({steps} steps) and A struck once while held, then twice ({strikes} strikes)"
    );
}
