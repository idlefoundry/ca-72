//! MIDI Learn's parts of the panel (the CA-72's `docs/decisions.md` R34): a control's menu (a
//! right click), the note over the control being learned, and the ring that marks it. Each is a
//! layer the renderer draws over the controls; a menu's items are found where they are drawn
//! ([`Menu::hit`]), both from one layout.

use crate::art::{H, Layer, W};
use crate::controls::Control;
use crate::fonts::{Fonts, Weight};
use crate::svg::{N, Svg, colour, escape, put};

/// The strip's and the drawer's accent: what MIDI Learn waits on, and what it did.
pub const ACCENT: &str = "#f0a030";
const DIM: &str = "#8a909c";
const WARN: &str = "#e5b94d";
const BACK: &str = "#1f2228";
const BORDER: &str = "#4a4640";
const HOVER: &str = "#3a3022";

/// A line's colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Accent,
    Dim,
    Warn,
}

impl Tone {
    fn fill(self) -> &'static str {
        match self {
            Tone::Plain => colour::LEGEND,
            Tone::Accent => ACCENT,
            Tone::Dim => DIM,
            Tone::Warn => WARN,
        }
    }
}

/// A control's menu: its top left corner (drawing units), its lettering's size (drawing units),
/// its title (dim, not an item), its items (each its label and whether it can be chosen) and
/// the one under the pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub title: String,
    pub items: Vec<(String, bool)>,
    pub hover: Option<usize>,
}

impl Menu {
    fn pad(&self) -> f64 {
        self.size * 0.8
    }

    fn row(&self) -> f64 {
        self.size * 2.0
    }

    /// Its width and height.
    pub fn extent(&self, fonts: &Fonts) -> (f64, f64) {
        let widest = std::iter::once(self.title.as_str())
            .chain(self.items.iter().map(|(t, _)| t.as_str()))
            .map(|t| fonts.advance(t, self.size, Weight::Regular, 0.0))
            .fold(0.0, f64::max);
        (
            widest + 2.0 * self.pad(),
            self.row() * (self.items.len() + 1) as f64 + self.size * 0.4,
        )
    }

    /// The menu moved to fit within `area` (left, top, right, bottom), its top left corner at
    /// (`x`, `y`) where it can be.
    pub fn placed(mut self, fonts: &Fonts, x: f64, y: f64, area: [f64; 4]) -> Menu {
        let (w, h) = self.extent(fonts);
        self.x = x.min(area[2] - w).max(area[0]);
        self.y = y.min(area[3] - h).max(area[1]);
        self
    }

    /// Where item `i`'s label begins, mid-height (drawing units).
    pub fn item_at(&self, i: usize) -> (f64, f64) {
        (
            self.x + self.pad() + self.size,
            self.y + self.row() * (i as f64 + 1.5),
        )
    }

    /// What is at (`x`, `y`): `Some(Some(i))` item `i`, `Some(None)` the menu but no item
    /// (its title, its margin), `None` outside it.
    pub fn hit(&self, fonts: &Fonts, x: f64, y: f64) -> Option<Option<usize>> {
        let (w, h) = self.extent(fonts);
        if !(self.x..self.x + w).contains(&x) || !(self.y..self.y + h).contains(&y) {
            return None;
        }
        let k = ((y - self.y) / self.row()).floor() as usize;
        Some(k.checked_sub(1).filter(|&i| i < self.items.len()))
    }
}

/// A menu's layer: a box in the tips' colours, its title dim, then its items, the one under the
/// pointer lit, one that cannot be chosen dim.
pub fn menu(fonts: &Fonts, m: &Menu) -> Layer {
    let (w, h) = m.extent(fonts);
    let mut s = Svg::default();
    put!(
        s,
        "<rect x='0' y='0' width='{}' height='{}' rx='{}' fill='{BACK}' stroke='{BORDER}' stroke-width='{}'/>",
        N(w),
        N(h),
        N(m.size * 0.3),
        N(m.size / 10.0)
    );
    let row = m.row();
    let base = |k: usize| row * k as f64 + row * 0.5 + m.size * 0.36;
    put!(
        s,
        "<text x='{}' y='{}' font-size='{}' fill='{DIM}'>{}</text>",
        N(m.pad()),
        N(base(0)),
        N(m.size),
        escape(&m.title)
    );
    for (i, (label, enabled)) in m.items.iter().enumerate() {
        let top = row * (i + 1) as f64;
        if m.hover == Some(i) && *enabled {
            put!(
                s,
                "<rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='{HOVER}'/>",
                N(m.size * 0.2),
                N(top),
                N(w - m.size * 0.4),
                N(row),
                N(m.size * 0.2)
            );
        }
        put!(
            s,
            "<text x='{}' y='{}' font-size='{}' fill='{}'>{}</text>",
            N(m.pad()),
            N(base(i + 1)),
            N(m.size),
            if *enabled { colour::LEGEND } else { DIM },
            escape(label)
        );
    }
    let e = m.size / 5.0;
    Layer {
        origin: (m.x, m.y),
        bounds: [-e, -e, w + e, h + e],
        body: s.0,
        ..Layer::default()
    }
}

/// A note: its lines (each its text and colour), its lettering's size (drawing units), centred
/// above (`x`, `y`) in the drawing, kept within the panel's width.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub lines: Vec<(String, Tone)>,
}

impl Note {
    /// Its width and height.
    pub fn extent(&self, fonts: &Fonts) -> (f64, f64) {
        let widest = self
            .lines
            .iter()
            .map(|(t, _)| fonts.advance(t, self.size, Weight::Regular, 0.0))
            .fold(0.0, f64::max);
        (
            widest + self.size * 1.2,
            self.size * (1.5 * self.lines.len() as f64 + 0.3),
        )
    }
}

/// A note's layer: a box with the accent's border above its point, its lines centred.
pub fn note(fonts: &Fonts, n: &Note) -> Layer {
    let (w, h) = n.extent(fonts);
    let gap = n.size * 0.4;
    // Centred on x, but within the panel.
    let left = (n.x - w / 2.0).clamp(0.0, (W - w).max(0.0));
    let top = (n.y - gap - h).max(0.0);
    let (ox, oy) = (left + w / 2.0, top);
    let mut s = Svg::default();
    put!(
        s,
        "<rect x='{}' y='0' width='{}' height='{}' rx='{}' fill='{BACK}' stroke='{ACCENT}' stroke-width='{}'/>",
        N(-w / 2.0),
        N(w),
        N(h),
        N(n.size * 0.3),
        N(n.size / 8.0)
    );
    for (k, (t, tone)) in n.lines.iter().enumerate() {
        put!(
            s,
            "<text x='0' y='{}' text-anchor='middle' font-size='{}' fill='{}'>{}</text>",
            N(n.size * (1.5 * k as f64 + 1.25)),
            N(n.size),
            tone.fill(),
            escape(t)
        );
    }
    let e = n.size / 4.0;
    Layer {
        origin: (ox, oy),
        bounds: [-w / 2.0 - e, -e, w / 2.0 + e, h + e],
        body: s.0,
        ..Layer::default()
    }
}

/// The note's place for a control: centred above its top.
pub fn above(c: &Control) -> (f64, f64) {
    let (x, y) = c.centre();
    (x, y + crate::art::bounds(&c.kind)[1])
}

/// The ring around control `c` being learned: its extent, a little outside it, in the accent.
pub fn ring(c: &Control) -> Layer {
    let b = crate::art::bounds(&c.kind);
    ring_around(c.centre(), [b[0] - 8.0, b[1] - 8.0, b[2] + 8.0, b[3] + 8.0])
}

/// A ring at `origin` around `b` (left, top, right, bottom about it).
pub fn ring_around(origin: (f64, f64), b: [f64; 4]) -> Layer {
    let mut s = Svg::default();
    put!(
        s,
        "<rect x='{}' y='{}' width='{}' height='{}' rx='14' fill='none' stroke='{ACCENT}' stroke-width='6' stroke-dasharray='18 10'/>",
        N(b[0]),
        N(b[1]),
        N(b[2] - b[0]),
        N(b[3] - b[1])
    );
    Layer {
        origin,
        bounds: [b[0] - 5.0, b[1] - 5.0, b[2] + 5.0, b[3] + 5.0],
        body: s.0,
        ..Layer::default()
    }
}

/// The panel's area, for placing a menu within it.
pub const PANEL: [f64; 4] = [0.0, 0.0, W, H];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::{CONTROLS, index};

    fn menu_at(x: f64, y: f64) -> Menu {
        Menu {
            x,
            y,
            size: 26.0,
            title: "CUTOFF FREQUENCY · CH 1 · CC 74".into(),
            items: vec![
                ("MIDI LEARN".into(), true),
                ("REMOVE MIDI ASSIGNMENT".into(), true),
                ("MIDI ASSIGNMENTS…".into(), true),
            ],
            hover: None,
        }
    }

    /// Each item is found where it is drawn, the title and the margin are the menu's but no
    /// item, and outside it nothing.
    #[test]
    fn a_menus_items_are_found_where_drawn() {
        let fonts = Fonts::new();
        let m = menu_at(100.0, 200.0);
        let (w, h) = m.extent(&fonts);
        let row = m.size * 2.0;
        assert_eq!(m.hit(&fonts, 110.0, 210.0), Some(None), "the title");
        for i in 0..3 {
            let y = 200.0 + row * (i as f64 + 1.5);
            assert_eq!(m.hit(&fonts, 100.0 + w / 2.0, y), Some(Some(i)), "item {i}");
        }
        assert_eq!(m.hit(&fonts, 100.0 + w / 2.0, 200.0 + h - 1.0), Some(None));
        assert_eq!(m.hit(&fonts, 99.0, 210.0), None);
        assert_eq!(m.hit(&fonts, 100.0 + w + 1.0, 210.0), None);
        assert_eq!(m.hit(&fonts, 110.0, 200.0 + h + 1.0), None);
    }

    /// A menu opened at the panel's bottom right corner is moved to fit inside it.
    #[test]
    fn a_menu_fits_within_the_panel() {
        let fonts = Fonts::new();
        let m = menu_at(0.0, 0.0).placed(&fonts, W - 5.0, H - 5.0, PANEL);
        let (w, h) = m.extent(&fonts);
        assert!(m.x >= 0.0 && m.x + w <= W + 1e-9, "{} {w}", m.x);
        assert!(m.y >= 0.0 && m.y + h <= H + 1e-9, "{} {h}", m.y);
    }

    /// A note over a control at the panel's left edge stays on the panel.
    #[test]
    fn a_note_stays_on_the_panel() {
        let fonts = Fonts::new();
        let tune = &CONTROLS[index("tune").expect("a control")];
        let (x, y) = above(tune);
        let n = Note {
            x,
            y,
            size: 26.0,
            lines: vec![(
                "A LINE MUCH WIDER THAN THE KNOB IT IS ABOUT, AND THEN SOME MORE".into(),
                Tone::Plain,
            )],
        };
        let l = note(&fonts, &n);
        assert!(l.origin.0 + l.bounds[0] >= -10.0, "{:?}", l);
        assert!(l.origin.1 + l.bounds[1] >= -10.0, "{:?}", l);
    }
}
