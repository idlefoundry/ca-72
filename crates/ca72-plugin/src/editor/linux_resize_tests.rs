//! A real X11 parent/child pair, off screen, with no processor or audio devices.
//! Check the event boundary: the resized child must already contain its new frame
//! before the normal 15 ms frame callback can run again, and show it again as the X server
//! asks.

use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};

use baseview::{Point, WindowHandle};
use nih_plug::context::PluginApi;
use nih_plug::wrapper::state::PluginState;
use raw_window_handle::HasRawWindowHandle;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, CreateWindowAux, WindowClass};

use super::*;

type Connection11 = x11rb::rust_connection::RustConnection;

struct Host {
    connection: Arc<Connection11>,
    top: u32,
    parent: u32,
    params: Arc<Ca72Params>,
    fixed: Arc<AtomicBool>,
}

#[allow(unsafe_code)]
impl GuiContext for Host {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn request_resize(&self) -> bool {
        if self.fixed.load(Ordering::Relaxed) {
            return false;
        }
        let width = self.params.editor_width.load(Ordering::Relaxed);
        let height = height_for_panel(width, self.params.editor_expanded.load(Ordering::Relaxed));
        resize_host(&self.connection, (self.top, self.parent), (width, height));
        true
    }
    unsafe fn raw_begin_set_parameter(&self, _: ParamPtr) {}
    unsafe fn raw_set_parameter_normalized(&self, _: ParamPtr, _: f32) {}
    unsafe fn raw_end_set_parameter(&self, _: ParamPtr) {}
    fn get_state(&self) -> PluginState {
        panic!("no state requested")
    }
    fn set_state(&self, _: PluginState) {}
}

/// The host's window and the one it gives the editor, `width` by `height` (the header above).
fn resize_host(connection: &Connection11, (top, parent): (u32, u32), (width, height): (u32, u32)) {
    for (window, h) in [(top, height + HEADER), (parent, height)] {
        connection
            .configure_window(window, &ConfigureWindowAux::new().width(width).height(h))
            .expect("resize host window");
    }
    connection.flush().expect("flush host resize");
}

#[derive(Debug)]
struct Sample {
    size: (u32, u32),
    ready: bool,
    painted: bool,
    scale: f64,
    micros: u128,
}

/// The editor told its window was uncovered: when, and whether it had a frame of its
/// window's size to show then.
#[derive(Debug)]
struct Damage {
    at: Instant,
    showable: bool,
}

struct Probe {
    panel: PanelWindow,
    toggle: Receiver<()>,
    result: Sender<Sample>,
    damaged: Sender<Damage>,
}

impl WindowHandler for Probe {
    fn on_frame(&mut self, window: &mut Window<'_>) {
        self.panel.on_frame(window);
        if self.toggle.try_recv().is_ok() {
            let (x, y) = strip::centre(self.panel.editing.rail, StripTarget::Bar(BarTarget::Panel));
            let scale = self.panel.editing.renderer.scale() / self.panel.editing.dpr;
            self.panel.on_event(
                window,
                Event::Mouse(MouseEvent::CursorMoved {
                    position: Point {
                        x: x * scale,
                        y: y * scale,
                    },
                    modifiers: Modifiers::empty(),
                }),
            );
            let started = Instant::now();
            let shown = self.panel.shown;
            for event in [
                MouseEvent::ButtonPressed {
                    button: MouseButton::Left,
                    modifiers: Modifiers::empty(),
                },
                MouseEvent::ButtonReleased {
                    button: MouseButton::Left,
                    modifiers: Modifiers::empty(),
                },
            ] {
                self.panel.on_event(window, Event::Mouse(event));
            }
            self.result
                .send(Sample {
                    size: self.panel.physical,
                    ready: self.showable() && self.panel.surface.presented(),
                    painted: self.panel.shown != shown,
                    scale: self.panel.editing.renderer.scale(),
                    micros: started.elapsed().as_micros(),
                })
                .expect("send result");
        }
    }
    fn on_event(&mut self, window: &mut Window<'_>, event: Event) -> EventStatus {
        if matches!(event, Event::Window(WindowEvent::Damaged)) {
            let _ = self.damaged.send(Damage {
                at: Instant::now(),
                showable: self.showable(),
            });
        }
        self.panel.on_event(window, event)
    }
}

impl Probe {
    /// Whether the editor has a frame shown, composed at its window's size.
    fn showable(&self) -> bool {
        self.panel.shown.is_some()
            && self
                .panel
                .editing
                .composed
                .as_ref()
                .is_some_and(|frame| (frame.width(), frame.height()) == self.panel.physical)
    }
}

static DESKTOP: Mutex<()> = Mutex::new(());
const ORIGIN: (i16, i16) = (-16000, -16000);
/// The host's window's header, above the window it gives the editor.
const HEADER: u32 = 40;

/// The editor in a window of a host's own, off screen, and what drives and watches it.
struct Rig {
    conn: Arc<Connection11>,
    top: u32,
    child: u32,
    handle: WindowHandle,
    fixed: Arc<AtomicBool>,
    toggle: Sender<()>,
    result: Receiver<Sample>,
    damaged: Receiver<Damage>,
}

impl Rig {
    fn open() -> Rig {
        let (conn, screen) = x11rb::connect(None).expect("X11 display required");
        let conn = Arc::new(conn);
        let root = conn.setup().roots[screen].root;
        let (top, parent) = (
            conn.generate_id().expect("top id"),
            conn.generate_id().expect("parent id"),
        );
        let params = Arc::new(Ca72Params::default());
        params.editor_width.store(MIN_WIDTH, Ordering::Relaxed);
        let tall = height_for_panel(MIN_WIDTH, true);
        // The host's own window, room to spare in it, and the window it gives the editor in it
        // (Bitwig's: its plug-in window's header above).
        for (window, within, (x, y), (w, h)) in [
            (top, root, ORIGIN, (MIN_WIDTH, tall + HEADER)),
            (parent, top, (0, HEADER as i16), (MIN_WIDTH, tall)),
        ] {
            conn.create_window(
                x11rb::COPY_DEPTH_FROM_PARENT,
                window,
                within,
                x,
                y,
                w as u16,
                h as u16,
                0,
                WindowClass::INPUT_OUTPUT,
                x11rb::COPY_FROM_PARENT,
                &CreateWindowAux::new().override_redirect(1),
            )
            .expect("create window")
            .check()
            .expect("window valid");
            conn.map_window(window).expect("map window");
        }
        conn.flush().expect("flush windows");
        let fixed = Arc::new(AtomicBool::new(false));
        let host = Arc::new(Host {
            connection: Arc::clone(&conn),
            top,
            parent,
            params: Arc::clone(&params),
            fixed: Arc::clone(&fixed),
        });
        let (toggle, actions) = mpsc::channel();
        let (results, result) = mpsc::channel();
        let (damages, damaged) = mpsc::channel();
        let library = tempfile::tempdir().expect("private library");
        let path = library.path().to_path_buf();
        let handle = Window::open_parented(
            &window::Parent(ParentWindowHandle::X11Window(parent)),
            WindowOpenOptions {
                title: "CA-72 resize probe".into(),
                size: Size::new(f64::from(MIN_WIDTH), f64::from(tall)),
                scale: WindowScalePolicy::ScaleFactor(1.0),
            },
            move |window| Probe {
                panel: PanelWindow::new(
                    window,
                    host,
                    params,
                    Arc::new(Meters::default()),
                    MIN_WIDTH,
                    1.0,
                    Library::at(path),
                    Arc::new(HostKeys::default()),
                ),
                toggle: actions,
                result: results,
                damaged: damages,
            },
        );
        let child = match handle.raw_window_handle() {
            raw_window_handle::RawWindowHandle::Xcb(h) => h.window,
            raw_window_handle::RawWindowHandle::Xlib(h) => h.window as u32,
            _ => panic!("X11 child required"),
        };
        Rig {
            conn,
            top,
            child,
            handle,
            fixed,
            toggle,
            result,
            damaged,
        }
    }

    /// PANEL clicked: what the editor had done by the end of the click's events.
    fn click(&self) -> Sample {
        self.toggle.send(()).expect("send click");
        self.result
            .recv_timeout(Duration::from_secs(30))
            .expect("event result")
    }

    /// Waits for `window` to be `want` pixels.
    fn wait_for(&self, window: u32, want: (u32, u32)) {
        let until = Instant::now() + Duration::from_secs(2);
        loop {
            let geometry = self
                .conn
                .get_geometry(window)
                .expect("query window")
                .reply()
                .expect("window geometry");
            if (u32::from(geometry.width), u32::from(geometry.height)) == want {
                break;
            }
            assert!(
                Instant::now() < until,
                "native window reaches the requested size"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Runs `test` on the rig, then closes the editor and its parent.
    fn run(mut self, test: impl FnOnce(&Rig)) {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(&self)));
        self.handle.close();
        let until = Instant::now() + Duration::from_secs(5);
        while self.handle.is_open() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!self.handle.is_open(), "probe closed");
        self.conn
            .destroy_window(self.top)
            .expect("destroy own windows");
        self.conn.flush().expect("flush cleanup");
        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }
}

#[test]
#[ignore = "requires an X11 display; creates only its own off-screen windows"]
fn panel_resize_presents_before_the_next_frame_callback() {
    let _desktop = DESKTOP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Rig::open().run(|rig| {
        for expanded in [false, true, false, true] {
            let sample = rig.click();
            let want = (MIN_WIDTH, height_for_panel(MIN_WIDTH, expanded));
            assert_eq!(sample.size, want);
            assert_eq!(sample.scale, f64::from(MIN_WIDTH) / art::W);
            eprintln!(
                "expanded={expanded}: click-to-present {} us; same-event frame={}",
                sample.micros, sample.ready
            );
            assert!(
                sample.ready && sample.painted,
                "native resize returned before its correctly sized frame was presented"
            );
            rig.wait_for(rig.child, want);
            // A refused opposite toggle retains this exact view and does not
            // synchronously repaint a speculative height.
            rig.fixed.store(true, Ordering::Relaxed);
            let refused = rig.click();
            assert_eq!(refused.size, want);
            assert!(refused.ready && !refused.painted, "{refused:?}");
            rig.fixed.store(false, Ordering::Relaxed);
        }
    });
}

/// Issue #65 (decisions.md R-LINUX-RESIZE; baseview's PATCHES.md, change 12): the X server
/// asks for the editor's window to be drawn again (an `Expose`) once it has taken the height
/// PANEL asked for, and the editor shows its frame again then, not at its next repaint
/// (`REPAINT`, up to 250 ms later). In Bitwig, which grows its own window after the editor's,
/// that showing is what fills the panel's lower half.
#[test]
#[ignore = "requires an X11 display; creates only its own off-screen windows"]
fn a_panel_grown_is_shown_again_as_x_asks() {
    let _desktop = DESKTOP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Rig::open().run(|rig| {
        for expanded in [false, true, false, true, false, true] {
            while rig.damaged.try_recv().is_ok() {}
            let sample = rig.click();
            let want = (MIN_WIDTH, height_for_panel(MIN_WIDTH, expanded));
            assert_eq!(sample.size, want);
            rig.wait_for(rig.child, want);
            if !expanded {
                continue;
            }
            let clicked = Instant::now();
            let damage = rig
                .damaged
                .recv_timeout(Duration::from_millis(100))
                .expect("the editor asked to show its grown window again");
            eprintln!(
                "told {} us after the click's events; {damage:?}",
                damage.at.saturating_duration_since(clicked).as_micros()
            );
            assert!(damage.showable, "a frame of the window's size to show");
        }
    });
}
