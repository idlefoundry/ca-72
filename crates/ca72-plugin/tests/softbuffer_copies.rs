//! The editor's pixels on macOS beside other copies of softbuffer (decisions.md R35): a host
//! that loads every plug-in into one process (Bitwig Studio) holds a copy of softbuffer for
//! each plug-in's library, and each registers an Objective-C class for its observers. Here the
//! process already holds the classes an unpatched copy (upstream's `SoftbufferObserver`) and a
//! patched one (`SoftbufferObserver1`) register; this copy's surface is still made (upstream's
//! panicked, the editor blank), its frame reaches the view's layer, and the layer follows the
//! view as the host resizes it. The copy registers one class, its own, for all its surfaces.
//!
//! On the main thread, which softbuffer's macOS backend requires and the test harness does not
//! run tests on (`harness = false`); elsewhere it does nothing.

#![allow(clippy::unwrap_used)]

#[cfg(not(target_os = "macos"))]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {
    macos::the_surface_is_made_beside_other_copies_classes();
    println!("the surface is made beside other copies' classes ... ok");
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos {
    use std::ffi::c_void;
    use std::num::NonZeroU32;
    use std::ptr::NonNull;

    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, NSObject};
    use objc2::{ClassType, msg_send};
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    use raw_window_handle_06 as rwh;

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}

    /// A view of the test's own, as the host's editor window would be.
    struct View(Retained<AnyObject>);

    impl View {
        fn new(w: f64, h: f64) -> Self {
            let class = AnyClass::get(c"NSView").unwrap();
            let frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h));
            // SAFETY: `+alloc` and `-initWithFrame:` make an NSView, on the main thread.
            let view: Retained<AnyObject> = unsafe {
                let view: Allocated<AnyObject> = msg_send![class, alloc];
                msg_send![view, initWithFrame: frame]
            };
            // SAFETY: an NSView's method, on the main thread.
            let _: () = unsafe { msg_send![&*view, setWantsLayer: Bool::YES] };
            View(view)
        }

        fn resize(&self, w: f64, h: f64) {
            // SAFETY: an NSView's method, on the main thread.
            let _: () = unsafe { msg_send![&*self.0, setFrameSize: NSSize::new(w, h)] };
        }

        /// The layers softbuffer added to the view's.
        fn sublayers(&self) -> Vec<Retained<AnyObject>> {
            // SAFETY: an NSView's layer (it wants one) and a CALayer's sublayers, an NSArray.
            unsafe {
                let layer: Retained<AnyObject> = msg_send![&*self.0, layer];
                let sublayers: Option<Retained<AnyObject>> = msg_send![&*layer, sublayers];
                let Some(sublayers) = sublayers else {
                    return Vec::new();
                };
                let n: usize = msg_send![&*sublayers, count];
                (0..n)
                    .map(|i| msg_send![&*sublayers, objectAtIndex: i])
                    .collect()
            }
        }
    }

    /// The view as softbuffer takes it.
    #[derive(Clone)]
    struct Target(NonNull<c_void>);

    impl rwh::HasWindowHandle for Target {
        fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
            let raw = rwh::RawWindowHandle::AppKit(rwh::AppKitWindowHandle::new(self.0));
            // SAFETY: the view outlives the surfaces made on it.
            Ok(unsafe { rwh::WindowHandle::borrow_raw(raw) })
        }
    }

    impl rwh::HasDisplayHandle for Target {
        fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
            let raw = rwh::RawDisplayHandle::AppKit(rwh::AppKitDisplayHandle::new());
            // SAFETY: AppKit's display handle carries nothing.
            Ok(unsafe { rwh::DisplayHandle::borrow_raw(raw) })
        }
    }

    /// A surface on `view`, a frame of `w` by `h` pixels shown in it, as the editor's.
    fn shown(
        view: &View,
        w: u32,
        h: u32,
    ) -> (
        softbuffer::Context<Target>,
        softbuffer::Surface<Target, Target>,
    ) {
        let target = Target(NonNull::from(&*view.0).cast());
        let context = softbuffer::Context::new(target.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, target).unwrap();
        let (w, h) = (NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap());
        surface.resize(w, h).unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        buffer.fill(0x003b_2213);
        buffer.present().unwrap();
        (context, surface)
    }

    pub fn the_surface_is_made_beside_other_copies_classes() {
        for name in [c"SoftbufferObserver", c"SoftbufferObserver1"] {
            ClassBuilder::new(name, NSObject::class())
                .unwrap()
                .register();
        }

        let view = View::new(64.0, 48.0);
        let surface = shown(&view, 64, 48);

        // This copy's class, under the first name free.
        assert!(AnyClass::get(c"SoftbufferObserver2").is_some());
        let layers = view.sublayers();
        assert_eq!(layers.len(), 1);
        // SAFETY: a CALayer's contents (the frame's CGImage) and frame.
        let (contents, frame) = unsafe {
            let contents: *const AnyObject = msg_send![&*layers[0], contents];
            let frame: NSRect = msg_send![&*layers[0], frame];
            (contents, frame)
        };
        assert!(!contents.is_null(), "the frame was not shown");
        assert_eq!((frame.size.width, frame.size.height), (64.0, 48.0));

        // The host resizes the view: softbuffer's observer resizes its layer.
        view.resize(80.0, 60.0);
        // SAFETY: a CALayer's frame.
        let frame: NSRect = unsafe { msg_send![&*layers[0], frame] };
        assert_eq!((frame.size.width, frame.size.height), (80.0, 60.0));

        // Gone, and made again: the class registered once.
        drop(surface);
        let view = View::new(32.0, 24.0);
        let _surface = shown(&view, 32, 24);
        assert!(AnyClass::get(c"SoftbufferObserver3").is_none());
    }
}
