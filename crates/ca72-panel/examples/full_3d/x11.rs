//! Linux review window over X11/XWayland. The panel itself is the CPU compositor.

use super::preview::Preview;
use ca72_panel::panel3d::{FaceRect, blit_dirty_to_u32};
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::wrapper::ConnectionExt as _;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) fn open(mut panel: Preview) -> Result<()> {
    let (conn, screen) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen];
    let depth = screen.root_depth;
    let width = (screen.width_in_pixels * 4 / 5).min(1520);
    let height = (width as u32 * panel.frame.height() / panel.frame.width()) as u16;
    let window = conn.generate_id()?;
    let gc = conn.generate_id()?;
    let delete = conn.intern_atom(false, b"WM_DELETE_WINDOW")?.reply()?.atom;
    let protocols = conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom;
    conn.create_window(
        depth,
        window,
        screen.root,
        0,
        0,
        width,
        height,
        0,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &CreateWindowAux::new()
            .background_pixel(0x161616)
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::KEY_PRESS,
            ),
    )?
    .check()?;
    conn.create_gc(gc, window, &CreateGCAux::new())?.check()?;
    conn.change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        b"CA-72 3D Panel Preview",
    )?
    .check()?;
    conn.change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        b"ca72-3d-preview\0Ca72-3D-Preview\0",
    )?
    .check()?;
    conn.change_property32(
        PropMode::REPLACE,
        window,
        protocols,
        AtomEnum::ATOM,
        &[delete],
    )?
    .check()?;
    conn.map_window(window)?.check()?;
    conn.flush()?;

    let aspect = panel.frame.width() as f32 / panel.frame.height() as f32;
    let mut viewport = Viewport::fit(width, height, aspect);
    let mut pixels = Vec::new();
    let mut previous = None;
    let mut repaint = true;
    loop {
        let mut event = conn.wait_for_event()?;
        loop {
            match event {
                Event::ClientMessage(e) if e.data.as_data32()[0] == delete => return Ok(()),
                Event::DestroyNotify(_) => return Ok(()),
                Event::KeyPress(e) if e.detail == 9 => return Ok(()), // Xorg Escape
                Event::Expose(_) => repaint = true,
                Event::ConfigureNotify(e) => {
                    viewport = Viewport::fit(e.width, e.height, aspect);
                    repaint = true;
                }
                Event::ButtonPress(e) => {
                    if let Some((x, y)) = viewport.point(e.event_x, e.event_y, &panel.frame) {
                        match e.detail {
                            1 => {
                                panel.press(x, y);
                                previous = Some((e.event_x, e.event_y));
                            }
                            4 | 5 => panel.scroll(
                                x,
                                y,
                                if e.detail == 4 { 1.0 } else { -1.0 },
                                e.state.contains(KeyButMask::SHIFT),
                            ),
                            _ => {}
                        }
                    }
                }
                Event::MotionNotify(e) => {
                    if let Some((x, y)) = previous.as_mut() {
                        panel.motion(
                            f32::from(e.event_x) - f32::from(*x),
                            f32::from(e.event_y) - f32::from(*y),
                            e.state.contains(KeyButMask::SHIFT),
                        );
                        (*x, *y) = (e.event_x, e.event_y);
                    }
                }
                Event::ButtonRelease(e) if e.detail == 1 => {
                    panel.release();
                    previous = None;
                }
                Event::Error(e) => return Err(format!("X11: {e:?}").into()),
                _ => {}
            }
            let Some(next) = conn.poll_for_event()? else {
                break;
            };
            event = next;
        }

        let mut dirty = panel.redraw();
        if repaint {
            conn.clear_area(false, window, 0, 0, 0, 0)?;
            dirty = vec![FaceRect {
                x0: 0,
                y0: 0,
                x1: panel.frame.width() as i32,
                y1: panel.frame.height() as i32,
            }];
            repaint = false;
        }
        if dirty.is_empty() {
            continue;
        }
        pixels.resize(usize::from(viewport.w) * usize::from(viewport.h), 0);
        let damage = blit_dirty_to_u32(
            &panel.frame,
            &dirty,
            &mut pixels,
            viewport.w.into(),
            viewport.h.into(),
        );
        for [x, y, w, h] in damage {
            let bytes_per_row = w as usize * 4;
            // Keep requests below the core X11 request-size limit.
            let rows_per_request = (60_000 / bytes_per_row).max(1);
            for row in (0..h as usize).step_by(rows_per_request) {
                let rows = rows_per_request.min(h as usize - row);
                let mut bytes = Vec::with_capacity(bytes_per_row * rows);
                for i in row..row + rows {
                    let start = (y as usize + i) * usize::from(viewport.w) + x as usize;
                    for pixel in &pixels[start..start + w as usize] {
                        bytes.extend_from_slice(&pixel.to_le_bytes());
                    }
                }
                conn.put_image(
                    ImageFormat::Z_PIXMAP,
                    window,
                    gc,
                    w as u16,
                    rows as u16,
                    viewport.x + x as i16,
                    viewport.y + y as i16 + row as i16,
                    0,
                    depth,
                    &bytes,
                )?;
            }
        }
        conn.flush()?;
    }
}

struct Viewport {
    x: i16,
    y: i16,
    w: u16,
    h: u16,
}

impl Viewport {
    fn fit(width: u16, height: u16, aspect: f32) -> Self {
        let w = width.min((f32::from(height) * aspect) as u16).max(1);
        let h = height.min((f32::from(w) / aspect) as u16).max(1);
        Self {
            x: ((width - w) / 2) as i16,
            y: ((height - h) / 2) as i16,
            w,
            h,
        }
    }

    fn point(&self, x: i16, y: i16, frame: &ca72_panel::Pixmap) -> Option<(f32, f32)> {
        let x = f32::from(x) - f32::from(self.x);
        let y = f32::from(y) - f32::from(self.y);
        let w = f32::from(self.w);
        let h = f32::from(self.h);
        (x >= 0.0 && x < w && y >= 0.0 && y < h)
            .then_some((x * frame.width() as f32 / w, y * frame.height() as f32 / h))
    }
}
