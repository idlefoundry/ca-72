//! The editor's pixels on macOS beside other copies of softbuffer (decisions.md R35): a host
//! that loads every plug-in into one process (Bitwig Studio) holds a copy of softbuffer for
//! each plug-in's library, and each registers an Objective-C class for its observers. Here the
//! process already holds the classes an unpatched copy (upstream's `SoftbufferObserver`) and a
//! patched one (`SoftbufferObserver1`) register; this copy's surface is still made (upstream's
//! panicked, the editor blank), its frame reaches the view's layer, and the layer follows the
//! view as the host resizes it. The copy registers one class, its own, for all its surfaces.
//! And the frame is shown in tiles (decisions.md R59): its buffer kept from frame to frame,
//! a present with damage gives new images to the tiles it touches alone.
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
    macos::a_frame_is_shown_in_tiles_and_damage_changes_only_its_own();
    println!("a frame is shown in tiles and damage changes only its own ... ok");
    // What a present costs the editor's thread at a Retina window's size, the whole frame and
    // a frame's usual damage: printed when asked (`CA72_TIMINGS=1`).
    if std::env::var_os("CA72_TIMINGS").is_some() {
        macos::presents_timed();
    }
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

    /// `view` as AppKit draws it for a window (`-cacheDisplayInRect:toBitmapImageRep:`, which
    /// knows its view's flip: softbuffer's layer drawn alone by Core Animation missed a flipped
    /// view's flip over it, and in baseview's view, the editor's, the tiles came out in rows
    /// bottom to top), `w` by `h` pixels from its top left corner: each pixel as softbuffer's
    /// (0RGB).
    fn drawn(view: &View, w: usize, h: usize) -> Vec<u32> {
        // SAFETY: NSView's and NSBitmapImageRep's methods, on the main thread; the bitmap's
        // pixels read inside its size.
        unsafe {
            let bounds: NSRect = msg_send![&*view.0, bounds];
            let rep: Retained<AnyObject> =
                msg_send![&*view.0, bitmapImageRepForCachingDisplayInRect: bounds];
            let _: () = msg_send![&*view.0, cacheDisplayInRect: bounds, toBitmapImageRep: &*rep];
            let (pw, ph): (isize, isize) =
                (msg_send![&*rep, pixelsWide], msg_send![&*rep, pixelsHigh]);
            let row: isize = msg_send![&*rep, bytesPerRow];
            let spp: isize = msg_send![&*rep, samplesPerPixel];
            let format: usize = msg_send![&*rep, bitmapFormat];
            let data: *const u8 = msg_send![&*rep, bitmapData];
            assert!(pw as usize >= w && ph as usize >= h, "{pw} by {ph}");
            // (Its alpha first or last: NSBitmapFormatAlphaFirst is 1.)
            let first = format & 1 != 0;
            let mut out = Vec::with_capacity(w * h);
            for y in 0..h {
                for x in 0..w {
                    let p = data.add(y * row as usize + x * spp as usize);
                    let at = |i: usize| u32::from(*p.add(i));
                    let (r, g, b) = if first {
                        (at(1), at(2), at(3))
                    } else {
                        (at(0), at(1), at(2))
                    };
                    out.push(r << 16 | g << 8 | b);
                }
            }
            out
        }
    }

    /// A view of the test's own, as the host's editor window would be.
    struct View(Retained<AnyObject>);

    impl View {
        fn new(w: f64, h: f64) -> Self {
            Self::of(AnyClass::get(c"NSView").unwrap(), w, h)
        }

        /// A view whose coordinates run down from its top, as baseview's (the editor's) do: its
        /// layer's geometry flipped by AppKit.
        fn flipped(w: f64, h: f64) -> Self {
            extern "C-unwind" fn yes(_: &NSObject, _: objc2::runtime::Sel) -> Bool {
                Bool::YES
            }
            let class = AnyClass::get(c"CA72TestFlippedView").unwrap_or_else(|| {
                let mut b =
                    ClassBuilder::new(c"CA72TestFlippedView", AnyClass::get(c"NSView").unwrap())
                        .unwrap();
                // SAFETY: `-isFlipped`'s signature, which the method overrides.
                unsafe {
                    b.add_method(
                        objc2::sel!(isFlipped),
                        yes as extern "C-unwind" fn(_, _) -> _,
                    );
                }
                b.register()
            });
            Self::of(class, w, h)
        }

        fn of(class: &AnyClass, w: f64, h: f64) -> Self {
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
            // SAFETY: an NSView's layer (it wants one).
            let layer: Retained<AnyObject> = unsafe { msg_send![&*self.0, layer] };
            sublayers(&layer)
        }
    }

    /// A layer's sublayers.
    fn sublayers(layer: &AnyObject) -> Vec<Retained<AnyObject>> {
        // SAFETY: a CALayer's sublayers, an NSArray of CALayers.
        unsafe {
            let sublayers: Option<Retained<AnyObject>> = msg_send![layer, sublayers];
            let Some(sublayers) = sublayers else {
                return Vec::new();
            };
            let n: usize = msg_send![&*sublayers, count];
            (0..n)
                .map(|i| msg_send![&*sublayers, objectAtIndex: i])
                .collect()
        }
    }

    /// A layer's contents (an image) and frame.
    fn contents(layer: &AnyObject) -> (*const AnyObject, NSRect) {
        // SAFETY: a CALayer's contents and frame.
        unsafe {
            let contents: *const AnyObject = msg_send![layer, contents];
            let frame: NSRect = msg_send![layer, frame];
            (contents, frame)
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
        let (_, frame) = contents(&layers[0]);
        let tiles = sublayers(&layers[0]);
        assert_eq!(tiles.len(), 1);
        assert!(!contents(&tiles[0]).0.is_null(), "the frame was not shown");
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

    pub fn presents_timed() {
        use std::time::Instant;
        let (w, h) = (3441u32, 1965u32);
        let view = View::new(f64::from(w) / 2.0, f64::from(h) / 2.0);
        let (_context, mut surface) = shown(&view, w, h);
        let rect = |x: u32, y: u32, rw: u32, rh: u32| softbuffer::Rect {
            x,
            y,
            width: NonZeroU32::new(rw).unwrap(),
            height: NonZeroU32::new(rh).unwrap(),
        };
        // QUALITY's opening and the voices' display, as the played window's frames change.
        let usual = [rect(3248, 148, 193, 196), rect(1176, 1708, 1351, 129)];
        let n = 100;
        for (name, damage) in [
            ("the whole frame", None),
            ("the usual damage", Some(&usual)),
        ] {
            let t = Instant::now();
            for k in 0..n {
                let mut buffer = surface.buffer_mut().unwrap();
                buffer[k] = k as u32;
                match damage {
                    Some(d) => buffer.present_with_damage(d).unwrap(),
                    None => buffer.present().unwrap(),
                }
            }
            println!(
                "{name}: {:.3} ms a present",
                t.elapsed().as_secs_f64() * 1000.0 / n as f64
            );
        }
    }

    pub fn a_frame_is_shown_in_tiles_and_damage_changes_only_its_own() {
        for flipped in [false, true] {
            shown_in_tiles(flipped);
        }
    }

    /// In a view whose coordinates run up (AppKit's own) or down (baseview's, flipped).
    fn shown_in_tiles(flipped: bool) {
        // (Tiles of 256 pixels, as many points as the screen's scale factor gives: 128 on a
        // Retina screen's.)
        let view = if flipped {
            View::flipped(600.0, 300.0)
        } else {
            View::new(600.0, 300.0)
        };
        let (_context, mut surface) = shown(&view, 600, 300);
        let layer = &view.sublayers()[0];
        // SAFETY: a CALayer's scale factor.
        let k: f64 = unsafe { msg_send![&**layer, contentsScale] };
        let images = || -> Vec<(*const AnyObject, NSRect)> {
            sublayers(layer).iter().map(|t| contents(t)).collect()
        };
        let before = images();
        // Three tiles across, two down, each where its pixels are, all shown.
        assert_eq!(before.len(), 6);
        assert!(before.iter().all(|(c, _)| !c.is_null()));
        let at = |(_, f): &(*const AnyObject, NSRect)| (f.origin.x, f.origin.y, f.size.width);
        assert_eq!(at(&before[0]), (0.0, 0.0, 256.0 / k));
        assert_eq!(at(&before[5]), (512.0 / k, 256.0 / k, 88.0 / k));
        let at = if flipped {
            "in a flipped view"
        } else {
            "in a view"
        };
        // The buffer kept: the frame shown, in it still.
        let mut buffer = surface.buffer_mut().unwrap();
        assert_eq!(buffer.age(), 1);
        assert!(buffer.iter().all(|&p| p == 0x003b_2213));
        // A frame shaded across and down, shown: as AppKit draws the view, every pixel where it
        // is (none upside down, none moved, no seam), within the colours' conversion to the
        // screen's (a few dozen levels at the most; a pixel misplaced by a row of tiles is off
        // by a hundred or more).
        let pattern =
            |x: usize, y: usize| ((x * 255 / 599) << 16 | (y * 255 / 299) << 8 | 64) as u32;
        let off = |shown: &[u32], frame: &[u32]| {
            let apart = |a: u32, b: u32| {
                (0..3)
                    .map(|i| ((a >> (8 * i)) & 255).abs_diff((b >> (8 * i)) & 255))
                    .max()
                    .unwrap_or(0)
            };
            shown
                .iter()
                .zip(frame)
                .filter(|&(&a, &b)| apart(a, b) > 48)
                .count()
        };
        for (i, p) in buffer.iter_mut().enumerate() {
            *p = pattern(i % 600, i / 600);
        }
        let frame: Vec<u32> = buffer.to_vec();
        buffer.present().unwrap();
        let n = off(&drawn(&view, 600, 300), &frame);
        assert!(
            n == 0,
            "{at}: the tiles shown not as the frame ({n} pixels)"
        );
        let before = images();
        // A change in the middle tile of the top row alone: its image new, the others kept,
        // the frame drawn as changed.
        let mut buffer = surface.buffer_mut().unwrap();
        assert_eq!(buffer.age(), 1);
        buffer[10 * 600 + 300] = 0x00ff_ffff;
        let frame: Vec<u32> = buffer.to_vec();
        let one = softbuffer::Rect {
            x: 300,
            y: 10,
            width: NonZeroU32::new(1).unwrap(),
            height: NonZeroU32::new(1).unwrap(),
        };
        buffer.present_with_damage(&[one]).unwrap();
        let after = images();
        for (i, (b, a)) in before.iter().zip(&after).enumerate() {
            assert_eq!(b.0 != a.0, i == 1, "tile {i}: changed {}", b.0 != a.0);
        }
        let n = off(&drawn(&view, 600, 300), &frame);
        assert!(
            n == 0,
            "{at}: the change shown not as the frame ({n} pixels)"
        );
        // Resized: the buffer not kept; tiles made again for the new size, every one shown.
        surface
            .resize(NonZeroU32::new(300).unwrap(), NonZeroU32::new(200).unwrap())
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        assert_eq!(buffer.age(), 0);
        buffer.fill(0);
        buffer.present().unwrap();
        let resized = images();
        assert_eq!(resized.len(), 2);
        assert!(resized.iter().all(|(c, _)| !c.is_null()));
    }
}
