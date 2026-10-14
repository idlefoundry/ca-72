//! Fatal app errors remain visible when a desktop launch has no console.

use std::fmt::Write;

use baseview::{
    Event, EventStatus, MouseButton, MouseEvent, Size, Window, WindowEvent, WindowHandler,
    WindowOpenOptions, WindowScalePolicy,
};
use ca72_panel::{
    Pixmap,
    fonts::{FAMILY, Fonts, Weight},
};
use keyboard_types::{Key, KeyState};

use crate::editor::window::Surface;

const WIDTH: f64 = 720.0;

/// Called on the main thread only, after the audio runtime has stopped. This
/// window uses the editor's existing renderer and has no hardware access.
pub fn show(message: &str) {
    if !crate::editor::window::display() {
        return; // The executable also reports the full error on stderr.
    }
    let fonts = Fonts::new();
    let lines = wrap(message, &fonts);
    let height = 162.0 + lines.len() as f64 * 24.0;
    let svg = document(&lines, height);
    let options = resvg::usvg::Options {
        fontdb: fonts.database(),
        ..Default::default()
    };
    let Ok(tree) = resvg::usvg::Tree::from_str(&svg, &options) else {
        return;
    };
    Window::open_blocking(
        WindowOpenOptions {
            title: "CA-72 — error".into(),
            size: Size::new(WIDTH, height),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        move |window| ErrorWindow {
            surface: Surface::new(window, (WIDTH as u32, height as u32)),
            frame: Pixmap::new(WIDTH as u32, height as u32).expect("error window pixels"),
            tree,
            height,
            physical: (WIDTH as u32, height as u32),
            scale: 1.0,
            logical_scale: 1.0,
            cursor: (0.0, 0.0),
            redraw: true,
        },
    );
}

fn wrap(message: &str, fonts: &Fonts) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for c in message
        .chars()
        .filter(|c| (!c.is_control() || *c == '\n') && !matches!(c, '\u{fffe}' | '\u{ffff}'))
    {
        if c == '\n' {
            lines.push(std::mem::take(&mut line));
            continue;
        }
        line.push(c);
        if fonts.advance(&line, 18.0, Weight::Regular, 0.0) > WIDTH - 78.0 {
            line.pop();
            lines.push(std::mem::take(&mut line));
            line.push(c);
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

fn document(lines: &[String], height: f64) -> String {
    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{WIDTH}' height='{height}' viewBox='0 0 {WIDTH} {height}'><rect width='100%' height='100%' fill='#1f2228'/><g font-family='{FAMILY}' fill='#ebe6d8'><text x='32' y='48' font-size='26'>CA-72 could not finish</text>"
    );
    for (i, line) in lines.iter().enumerate() {
        let escaped = line
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let _ = write!(
            svg,
            "<text x='32' y='{}' font-size='18'>{escaped}</text>",
            88.0 + i as f64 * 24.0
        );
    }
    let button_y = height - 58.0;
    let _ = write!(
        svg,
        "<rect x='552' y='{button_y}' width='136' height='38' rx='5' fill='#3a3022' stroke='#f0a030'/><text x='620' y='{}' text-anchor='middle' font-size='17'>CLOSE</text></g></svg>",
        button_y + 25.0
    );
    svg
}

struct ErrorWindow {
    surface: Surface,
    frame: Pixmap,
    tree: resvg::usvg::Tree,
    height: f64,
    physical: (u32, u32),
    scale: f64,
    logical_scale: f64,
    cursor: (f64, f64),
    redraw: bool,
}
impl WindowHandler for ErrorWindow {
    fn on_frame(&mut self, _: &mut Window<'_>) {
        if !self.redraw {
            return;
        }
        self.redraw = false;
        self.frame
            .fill(resvg::tiny_skia::Color::from_rgba8(31, 34, 40, 255));
        resvg::render(
            &self.tree,
            resvg::tiny_skia::Transform::from_scale(self.scale as f32, self.scale as f32),
            &mut self.frame.as_mut(),
        );
        self.surface.present(self.physical, &[&self.frame], None);
    }
    fn on_event(&mut self, window: &mut Window<'_>, event: Event) -> EventStatus {
        match event {
            Event::Window(WindowEvent::Resized(info)) => {
                let size = info.physical_size();
                self.physical = (size.width.max(1), size.height.max(1));
                self.scale = f64::from(self.physical.0) / WIDTH;
                self.logical_scale = info.logical_size().width / WIDTH;
                self.surface.resize(self.physical);
                self.frame =
                    Pixmap::new(self.physical.0, self.physical.1).expect("error window pixels");
                self.redraw = true;
            }
            Event::Window(WindowEvent::Damaged) => self.redraw = true,
            Event::Mouse(MouseEvent::CursorMoved { position, .. }) => {
                self.cursor = (position.x, position.y)
            }
            Event::Mouse(MouseEvent::ButtonReleased {
                button: MouseButton::Left,
                ..
            }) if (552.0..688.0).contains(&(self.cursor.0 / self.logical_scale))
                && (self.height - 58.0..self.height - 20.0)
                    .contains(&(self.cursor.1 / self.logical_scale)) =>
            {
                window.close()
            }
            Event::Keyboard(key)
                if key.state == KeyState::Down && matches!(key.key, Key::Escape | Key::Enter) =>
            {
                window.close()
            }
            _ => return EventStatus::Ignored,
        }
        EventStatus::Captured
    }
}
