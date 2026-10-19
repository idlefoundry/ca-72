//! Temporary, plain What's New presentation (decisions.md R-WHATS-NEW). Replace this
//! with the owner's chosen look; version selection and persistence belong to the plug-in.

use crate::fonts::{Fonts, Weight};
use crate::svg::{Svg, escape, put};
use crate::{Lettering, Pixmap, W};
use resvg::tiny_skia::Transform;

#[derive(Debug)]
pub struct View {
    installed: String,
    coverage: String,
    sections: [(String, Vec<String>); 3],
    first: usize,
    rows: Vec<(String, bool)>,
    page: usize,
    height: f64,
    size: f64,
}

impl View {
    pub fn new(installed: String, coverage: String, sections: [(String, Vec<String>); 3]) -> Self {
        Self {
            installed,
            coverage,
            sections,
            first: 0,
            rows: Vec::new(),
            page: 1,
            height: 0.0,
            size: 42.0,
        }
    }

    /// Fit to the actual viewport, including the compact view, without resizing it.
    pub fn fit(&mut self, fonts: &Fonts, height: f64, lettering: Lettering) {
        let size = if lettering == Lettering::Large {
            52.5
        } else {
            42.0
        };
        if self.height == height && self.size == size && !self.rows.is_empty() {
            return;
        }
        self.height = height;
        self.size = size;
        self.rows.clear();
        self.rows.extend(wrap(fonts, &self.coverage, size, false));
        for (title, entries) in &self.sections {
            if entries.is_empty() {
                continue;
            }
            self.rows.push((String::new(), false));
            self.rows.extend(wrap(fonts, title, size, true));
            for entry in entries {
                self.rows
                    .extend(wrap(fonts, &format!("• {entry}"), size, false));
            }
        }
        self.page = ((height - 6.0 * size) / (size * 1.4)).floor().max(1.0) as usize;
        self.first = self.first.min(self.last());
    }

    fn last(&self) -> usize {
        self.rows.len().saturating_sub(self.page)
    }

    pub fn scroll(&mut self, rows: i32) {
        self.first = self
            .first
            .saturating_add_signed(rows as isize)
            .min(self.last());
    }

    pub fn home(&mut self) {
        self.first = 0;
    }
    pub fn end(&mut self) {
        self.first = self.last();
    }
    pub fn page(&mut self, direction: i32) {
        self.scroll(direction * self.page as i32);
    }

    /// The footer's three buttons: previous page, next page and close. Other presses
    /// are consumed, so that they never reach the instrument behind the notes.
    pub fn press(&mut self, x: f64, y: f64) -> bool {
        if y < self.height - 1.7 * self.size || y > self.height - 0.3 * self.size {
            return false;
        }
        if (W - 520.0..W - 70.0).contains(&x) {
            true
        } else {
            if (70.0..520.0).contains(&x) {
                self.page(-1);
            }
            if (560.0..1010.0).contains(&x) {
                self.page(1);
            }
            false
        }
    }

    pub fn draw(&self, onto: &mut Pixmap, scale: f64, fonts: &Fonts) {
        onto.fill(resvg::tiny_skia::Color::from_rgba8(20, 20, 20, 255));
        let mut svg = Svg::default();
        let (h, size) = (self.height, self.size);
        put!(
            svg,
            "<svg xmlns='http://www.w3.org/2000/svg' width='{W}' height='{h}'><rect width='100%' height='100%' fill='#141414'/><g font-family='URW Gothic' fill='#eeeeee'>"
        );
        put!(
            svg,
            "<text x='70' y='{}' font-size='{}' font-weight='bold'>WHAT'S NEW · CA-72 {}</text>",
            1.5 * size,
            1.15 * size,
            escape(&self.installed)
        );
        put!(
            svg,
            "<text x='70' y='{}' font-size='{}' fill='#bbbbbb'>Temporary presentation</text>",
            2.7 * size,
            size * 0.75
        );
        for (i, (text, bold)) in self
            .rows
            .iter()
            .skip(self.first)
            .take(self.page)
            .enumerate()
        {
            let y = 4.2 * size + i as f64 * size * 1.4;
            let weight = if *bold { "bold" } else { "normal" };
            put!(
                svg,
                "<text x='70' y='{y}' font-size='{size}' font-weight='{weight}'>{}</text>",
                escape(text)
            );
        }
        for (x, text) in [
            (70.0, "PREVIOUS"),
            (560.0, "NEXT"),
            (W - 520.0, "CLOSE · ESC"),
        ] {
            let y = h - 1.7 * size;
            put!(
                svg,
                "<rect x='{x}' y='{y}' width='450' height='{}' fill='#333333'/><text x='{}' y='{}' font-size='{size}'>{text}</text>",
                1.4 * size,
                x + 20.0,
                y + size
            );
        }
        put!(
            svg,
            "<text x='1100' y='{}' font-size='{}' fill='#bbbbbb'>SCROLL / UP, DOWN · {}–{} OF {}</text></g></svg>",
            h - 0.7 * size,
            size * 0.7,
            self.first + 1,
            (self.first + self.page).min(self.rows.len()),
            self.rows.len()
        );
        let options = resvg::usvg::Options {
            fontdb: fonts.database(),
            ..Default::default()
        };
        let tree =
            resvg::usvg::Tree::from_str(&svg.0, &options).expect("the temporary notes' SVG parses");
        resvg::render(
            &tree,
            Transform::from_scale(scale as f32, scale as f32),
            &mut onto.as_mut(),
        );
    }
}

fn wrap(fonts: &Fonts, text: &str, size: f64, bold: bool) -> Vec<(String, bool)> {
    let weight = if bold { Weight::Bold } else { Weight::Regular };
    let mut rows = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && fonts.advance(&next, size, weight, 0.0) > W - 140.0 {
            rows.push((std::mem::take(&mut line), bold));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        rows.push((line, bold));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_wrap_scroll_and_close_in_each_viewport_and_lettering_size() {
        let fonts = Fonts::new();
        for lettering in [Lettering::Standard, Lettering::Large] {
            for height in [1020.0, crate::H] {
                let mut view = View::new(
                    "0.2.7".into(),
                    "Included releases: 0.2.3, 0.2.4, 0.2.5, 0.2.6, 0.2.7.".into(),
                    [
                        (
                            "Features".into(),
                            vec![
                                "A readable sentence about the instrument and its controls. "
                                    .repeat(8);
                                8
                            ],
                        ),
                        (
                            "Performance improvements".into(),
                            vec!["Less work on your computer.".into()],
                        ),
                        (
                            "Bug fixes".into(),
                            vec!["The final fix can be reached.".into()],
                        ),
                    ],
                );
                view.fit(&fonts, height, lettering);
                assert!(view.page > 1 && view.rows.len() > view.page);
                assert!(view.rows.iter().all(|(text, bold)| fonts.advance(
                    text,
                    view.size,
                    if *bold { Weight::Bold } else { Weight::Regular },
                    0.0
                ) <= W - 140.0));
                view.page(1);
                assert!(view.first > 0);
                view.end();
                assert_eq!(
                    view.rows.last().expect("final row").0,
                    "• The final fix can be reached."
                );
                assert_eq!(view.first + view.page, view.rows.len());
                view.scroll(i32::MAX);
                assert_eq!(view.first, view.last());
                view.scroll(i32::MIN);
                assert_eq!(view.first, 0);
                assert!(!view.press(600.0, 30.0));
                assert!(!view.press(600.0, height - view.size));
                assert!(view.first > 0);
                assert!(view.press(W - 300.0, height - view.size));
                for width in [720, 1200, 2400] {
                    let scale = f64::from(width) / W;
                    let mut pixmap =
                        Pixmap::new(width, (height * scale).round() as u32).expect("frame");
                    view.draw(&mut pixmap, scale, &fonts);
                    assert!(pixmap.data().chunks_exact(4).all(|p| p[3] == 255));
                }
            }
        }
    }
}
