//! A real X11 parent/child pair, off screen, with no processor or audio devices.
//! Check the event boundary: the resized child must already contain its new frame
//! before the normal 15 ms frame callback can run again.

use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};

use baseview::Point;
use nih_plug::context::PluginApi;
use nih_plug::wrapper::state::PluginState;
use raw_window_handle::HasRawWindowHandle;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, CreateWindowAux, WindowClass};

use super::*;

struct Host {
    connection: Arc<x11rb::rust_connection::RustConnection>,
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
        use x11rb::protocol::xproto::ConfigureWindowAux;
        let width = self.params.editor_width.load(Ordering::Relaxed);
        let height = height_for_panel(width, self.params.editor_expanded.load(Ordering::Relaxed));
        self.connection
            .configure_window(
                self.parent,
                &ConfigureWindowAux::new().width(width).height(height),
            )
            .expect("resize parent");
        self.connection.flush().expect("flush parent resize");
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

#[derive(Debug)]
struct Sample {
    size: (u32, u32),
    ready: bool,
    painted: bool,
    scale: f64,
    micros: u128,
}

struct Probe {
    panel: PanelWindow,
    toggle: Receiver<()>,
    result: Sender<Sample>,
}

impl WindowHandler for Probe {
    fn on_frame(&mut self, window: &mut Window<'_>) {
        self.panel.on_frame(window);
        if self.toggle.try_recv().is_ok() {
            let (x, y) = strip::centre(StripTarget::Bar(BarTarget::Panel));
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
            let ready =
                self.panel.shown.is_some()
                    && self.panel.surface.presented()
                    && self.panel.editing.composed.as_ref().is_some_and(|frame| {
                        (frame.width(), frame.height()) == self.panel.physical
                    });
            self.result
                .send(Sample {
                    size: self.panel.physical,
                    ready,
                    painted: self.panel.shown != shown,
                    scale: self.panel.editing.renderer.scale(),
                    micros: started.elapsed().as_micros(),
                })
                .expect("send result");
        }
    }
    fn on_event(&mut self, window: &mut Window<'_>, event: Event) -> EventStatus {
        self.panel.on_event(window, event)
    }
}

static DESKTOP: Mutex<()> = Mutex::new(());

#[test]
#[ignore = "requires an X11 display; creates only its own off-screen windows"]
fn panel_resize_presents_before_the_next_frame_callback() {
    let _desktop = DESKTOP.lock().expect("desktop test lock");
    let (conn, screen) = x11rb::connect(None).expect("X11 display required");
    let conn = Arc::new(conn);
    let root = conn.setup().roots[screen].root;
    let parent = conn.generate_id().expect("parent id");
    let params = Arc::new(Ca72Params::default());
    params.editor_width.store(MIN_WIDTH, Ordering::Relaxed);
    let tall = height_for_panel(MIN_WIDTH, true);
    conn.create_window(
        x11rb::COPY_DEPTH_FROM_PARENT,
        parent,
        root,
        -16000,
        -16000,
        MIN_WIDTH as u16,
        tall as u16,
        0,
        WindowClass::INPUT_OUTPUT,
        x11rb::COPY_FROM_PARENT,
        &CreateWindowAux::new().override_redirect(1),
    )
    .expect("create parent")
    .check()
    .expect("parent valid");
    conn.map_window(parent).expect("map parent");
    conn.flush().expect("flush parent");
    let fixed = Arc::new(AtomicBool::new(false));
    let host = Arc::new(Host {
        connection: Arc::clone(&conn),
        parent,
        params: Arc::clone(&params),
        fixed: Arc::clone(&fixed),
    });
    let (toggle, actions) = mpsc::channel();
    let (results, result) = mpsc::channel();
    let library = tempfile::tempdir().expect("private library");
    let path = library.path().to_path_buf();
    let mut handle = Window::open_parented(
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
        },
    );
    let child = match handle.raw_window_handle() {
        raw_window_handle::RawWindowHandle::Xcb(h) => h.window,
        raw_window_handle::RawWindowHandle::Xlib(h) => h.window as u32,
        _ => panic!("X11 child required"),
    };
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for expanded in [false, true, false, true] {
            toggle.send(()).expect("send click");
            let sample = result
                .recv_timeout(Duration::from_secs(30))
                .expect("event result");
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
            let until = Instant::now() + Duration::from_secs(2);
            loop {
                let geometry = conn
                    .get_geometry(child)
                    .expect("query child")
                    .reply()
                    .expect("child geometry");
                if (u32::from(geometry.width), u32::from(geometry.height)) == want {
                    break;
                }
                assert!(
                    Instant::now() < until,
                    "native child reaches the requested size"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
            // A refused opposite toggle retains this exact view and does not
            // synchronously repaint a speculative height.
            fixed.store(true, Ordering::Relaxed);
            toggle.send(()).expect("send refused click");
            let refused = result
                .recv_timeout(Duration::from_secs(30))
                .expect("refused result");
            assert_eq!(refused.size, want);
            assert!(refused.ready && !refused.painted, "{refused:?}");
            fixed.store(false, Ordering::Relaxed);
        }
    }));
    handle.close();
    let until = Instant::now() + Duration::from_secs(5);
    while handle.is_open() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!handle.is_open(), "probe closed");
    conn.destroy_window(parent).expect("destroy own parent");
    conn.flush().expect("flush cleanup");
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
