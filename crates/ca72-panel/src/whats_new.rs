//! What's New (decisions.md R-WHATS-NEW): the release notes in a dialog over the dimmed
//! editor, in the standalone's AUDIO & MIDI dialog's style, colours and sizes (WN-A, the
//! owner's choice, 2026-10-19). Which notes, and when, is the plug-in's; this is their look.
//!
//! The dialog keeps the AUDIO & MIDI dialog's width; its height follows the editor's view
//! (compact or full). The notes wrap under their bullets and scroll a line at a time when
//! they are longer than the dialog. LARGE lettering sets all of it a quarter larger.

use crate::fonts::{Fonts, Weight};
use crate::svg::{N, Svg, escape, put};
use crate::{Lettering, Pixmap, W};
use resvg::tiny_skia::Transform;

/// The dialog's left edge and width: the AUDIO & MIDI dialog's.
const X: f64 = 360.0;
const WIDTH: f64 = 2380.0;
/// Its top and bottom margins in the full view (1775 units tall), kept in proportion to a
/// taller one.
const TOP: f64 = 155.0 / 1775.0;
const FOOT: f64 = 140.0 / 1775.0;
/// Below this height (the compact view's) the margins narrow and the footnote moves beside
/// GOT IT, to leave the notes more room.
const COMPACT: f64 = 1400.0;
const COMPACT_TOP: f64 = 60.0;
const COMPACT_FOOT: f64 = 55.0;
/// GOT IT: the AUDIO & MIDI dialog's DONE, at its place from the dialog's corner.
const BUTTON_W: f64 = 315.0;
const BUTTON_H: f64 = 75.0;
const BUTTON_RIGHT: f64 = 50.0;
const BUTTON_UP: f64 = 125.0;
/// The scroll bar, inside the dialog's right edge.
const BAR_X: f64 = X + WIDTH - 48.0;
const BAR_W: f64 = 10.0;
/// The notes' strip's sides, and its room above a line's baseline and below it (at the
/// standard lettering): a line's capitals and accents, not the line before's descenders.
const STRIP_X: f64 = X + 40.0;
const STRIP_RIGHT: f64 = BAR_X - 20.0;
const PAD: f64 = 34.0;
const DESCENT: f64 = 12.0;
/// The notes' spacing at the standard lettering: a section's title to its first note, one
/// note to the next, a note's lines, and the last note to the next title (the mock-up's).
const UNDER_HEAD: f64 = 71.3;
const NOTE: f64 = 62.0;
const LINE: f64 = 46.2;
const OVER_HEAD: f64 = 96.1;

const BOX: &str = "#1f2228";
const BOX_RGBA: [u8; 4] = [0x1f, 0x22, 0x28, 0xff];
const EDGE: &str = "#4a4640";
const AMBER: &str = "#f0a030";
const MUTED: &str = "#a6a5a1";
const TEXT: &str = "#ebe6d8";
const WELL: &str = "#15171b";
const BUTTON: &str = "#3a3022";
const THUMB: &str = "#82715c";

/// One note: the release it came from when several are shown together (so that an older
/// claim is not read as the installed build's), and what it says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub release: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
enum Line {
    Head(String),
    /// A note's first line (with its bullet and release) or a following one.
    Item {
        first: bool,
        release: Option<String>,
        text: String,
    },
}

/// The dialog laid out for a view's height and a lettering size.
#[derive(Debug, Default)]
struct Layout {
    height: f64,
    k: f64,
    top: f64,
    bottom: f64,
    subtitle: Vec<String>,
    /// The notes' first baseline and the last one allowed, in the drawing.
    area: (f64, f64),
    /// Each line, and its baseline from the first's.
    lines: Vec<(Line, f64)>,
    /// Where a note's text starts after its bullet.
    indent: f64,
}

#[derive(Debug)]
pub struct View {
    installed: String,
    coverage: String,
    sections: [(String, Vec<Entry>); 3],
    layout: Layout,
    first: usize,
    /// The dialog's pictures at the scale last drawn (see [`View::draw`]).
    pictures: Option<Pictures>,
}

impl View {
    pub fn new(installed: String, coverage: String, sections: [(String, Vec<Entry>); 3]) -> Self {
        Self {
            installed,
            coverage,
            sections,
            layout: Layout::default(),
            first: 0,
            pictures: None,
        }
    }

    /// Fit to the actual viewport, including the compact view, without resizing it.
    pub fn fit(&mut self, fonts: &Fonts, height: f64, lettering: Lettering) {
        let k = lettering.text();
        if self.layout.height == height && self.layout.k == k && !self.layout.lines.is_empty() {
            return;
        }
        let (top, bottom) = if height < COMPACT {
            (COMPACT_TOP, height - COMPACT_FOOT)
        } else {
            ((height * TOP).round(), (height * (1.0 - FOOT)).round())
        };
        let subtitle = wrap(
            fonts,
            &format!("CA-72 {} is installed · {}", self.installed, self.coverage),
            31.0 * k,
            WIDTH - 110.0,
            0.0,
        );
        let first = title_y(top, k) + 68.0 * k + 49.0 * k * (subtitle.len() - 1) as f64 + 102.0 * k;
        let last = if height < COMPACT {
            bottom - BUTTON_UP - 50.0 * k
        } else {
            bottom - 155.0 - 70.0 * k
        };
        let indent = fonts.advance("·\u{a0}\u{a0}", 33.0 * k, Weight::Regular, 0.0);
        let width = BAR_X - 40.0 - (X + 95.0 + indent);
        let mut lines = Vec::new();
        let mut y = 0.0;
        for (title, entries) in &self.sections {
            if entries.is_empty() {
                continue;
            }
            if !lines.is_empty() {
                y += OVER_HEAD * k;
            }
            lines.push((Line::Head(title.to_uppercase()), y));
            y += (UNDER_HEAD - NOTE) * k;
            for entry in entries {
                y += (NOTE - LINE) * k;
                let release = entry.release.as_ref().map(|r| format!("{r}\u{a0}\u{a0}"));
                let lead = release
                    .as_ref()
                    .map_or(0.0, |r| fonts.advance(r, 33.0 * k, Weight::Regular, 0.0));
                for (i, text) in wrap(fonts, &entry.text, 33.0 * k, width, lead)
                    .into_iter()
                    .enumerate()
                {
                    y += LINE * k;
                    lines.push((
                        Line::Item {
                            first: i == 0,
                            release: release.clone().filter(|_| i == 0),
                            text,
                        },
                        y,
                    ));
                }
            }
        }
        self.layout = Layout {
            height,
            k,
            top,
            bottom,
            subtitle,
            area: (first, last.max(first)),
            lines,
            indent,
        };
        self.first = self.first.min(self.last());
        self.pictures = None;
    }

    /// The first line that shows the last one at the dialog's foot.
    fn last(&self) -> usize {
        let lines = &self.layout.lines;
        let Some(&(_, end)) = lines.last() else {
            return 0;
        };
        let room = self.layout.area.1 - self.layout.area.0;
        lines
            .iter()
            .position(|&(_, y)| end - y <= room)
            .unwrap_or(0)
    }

    /// How many lines show from the first shown.
    fn shown(&self) -> usize {
        let lines = &self.layout.lines;
        let Some(&(_, from)) = lines.get(self.first) else {
            return 0;
        };
        let room = self.layout.area.1 - self.layout.area.0;
        let mut shown = lines[self.first..]
            .iter()
            .take_while(|&&(_, y)| y - from <= room)
            .count();
        // (Never a section's title alone at the foot, its notes out of sight.)
        while shown > 1 && matches!(lines[self.first + shown - 1].0, Line::Head(_)) {
            shown -= 1;
        }
        shown
    }

    /// Whether the notes are longer than the dialog.
    pub fn scrolls(&self) -> bool {
        self.last() > 0
    }

    /// A note's line pitch, for a trackpad's travel (drawing units).
    pub fn line_height(&self) -> f64 {
        LINE * self.layout.k
    }

    /// The first line shown.
    pub fn position(&self) -> usize {
        self.first
    }

    pub fn scroll(&mut self, lines: i32) {
        self.first = self
            .first
            .saturating_add_signed(lines as isize)
            .min(self.last());
    }

    pub fn home(&mut self) {
        self.first = 0;
    }
    pub fn end(&mut self) {
        self.first = self.last();
    }
    /// A page on or back, keeping a line of the one before.
    pub fn page(&mut self, direction: i32) {
        let page = self.shown().saturating_sub(1).max(1) as i32;
        self.scroll(direction * page);
    }

    /// GOT IT's rectangle in the drawing: left, top, width, height.
    pub fn button(&self) -> [f64; 4] {
        [
            X + WIDTH - BUTTON_RIGHT - BUTTON_W,
            self.layout.bottom - BUTTON_UP,
            BUTTON_W,
            BUTTON_H,
        ]
    }

    /// The scroll bar's track (top, bottom) and its thumb's.
    fn bar(&self) -> ((f64, f64), (f64, f64)) {
        let (k, (first, last)) = (self.layout.k, self.layout.area);
        let track = (first - 33.0 * k, last + 12.0 * k);
        let lines = &self.layout.lines;
        let span = lines.get(self.last()).map_or(0.0, |&(_, y)| y);
        let room = last - first;
        let length = (track.1 - track.0) * room / (room + span);
        let at = lines.get(self.first).map_or(0.0, |&(_, y)| y) / span.max(1.0);
        let top = track.0 + (track.1 - track.0 - length) * at;
        (track, (top, top + length))
    }

    /// A press in the drawing: whether it closes the notes (GOT IT). A press on the scroll
    /// bar's track pages towards it. Every press is the notes', so that none reaches the
    /// instrument behind them.
    pub fn press(&mut self, x: f64, y: f64) -> bool {
        let [left, top, width, height] = self.button();
        if (left..left + width).contains(&x) && (top..top + height).contains(&y) {
            return true;
        }
        if self.scrolls() && (BAR_X - 30.0..BAR_X + BAR_W + 30.0).contains(&x) {
            let ((from, to), (thumb, foot)) = self.bar();
            if (from..thumb).contains(&y) {
                self.page(-1);
            } else if (foot..to).contains(&y) {
                self.page(1);
            }
        }
        false
    }

    /// The dialog over `onto`, the editor's frame at `scale`, the frame dimmed round it.
    ///
    /// Its pictures are made for a layout and scale: the dialog without its notes, and a
    /// strip with room for all of the notes, each line drawn into it the first time it
    /// shows. Each frame dims the editor, copies the dialog over it and the strip's lines
    /// that show into it: a few milliseconds even in a Retina window, whether the notes
    /// scroll or the panel behind them changes.
    pub fn draw(&mut self, onto: &mut Pixmap, scale: f64, fonts: &Fonts) {
        if self
            .pictures
            .as_ref()
            .is_none_or(|p| p.scale != scale.to_bits())
        {
            self.pictures = self.make_pictures(scale, fonts);
        }
        let first = self.first;
        let shown = first..first + self.shown();
        let thumb = self.scrolls().then(|| self.bar().1);
        let Some(mut p) = self.pictures.take() else {
            return;
        };
        let missing: Vec<_> = shown.clone().filter(|&i| !p.ready[i]).collect();
        if !missing.is_empty() {
            let width = p.strip.width() as usize * 4;
            for &i in &missing {
                let (from, to) = p.bands[i];
                for pixel in p.strip.data_mut()[from * width..to * width].chunks_exact_mut(4) {
                    pixel.copy_from_slice(&BOX_RGBA);
                }
                p.ready[i] = true;
            }
            render_svg(
                &self.lines_svg(&missing),
                fonts,
                &mut p.strip,
                scale,
                (p.strip_x, 0),
            );
        }
        over(onto, &p.dialog, p.origin, &p.spans);
        // The strip's rows from the first line shown's band to the last's, then the thumb.
        let (k, area) = (self.layout.k, self.layout.area);
        let lines = &self.layout.lines;
        if let (Some(&(_, from)), Some(&(_, to))) = (
            lines.get(shown.start),
            shown.end.checked_sub(1).and_then(|last| lines.get(last)),
        ) {
            let top = ((area.0 - PAD * k) * scale).round() as i32;
            let rows =
                (from * scale).round() as i32..((to + (PAD + DESCENT) * k) * scale).round() as i32;
            paste_rows(onto, &p.strip, (p.strip_x, top), rows);
        }
        if let Some((top, foot)) = thumb {
            let svg = format!(
                "<svg xmlns='http://www.w3.org/2000/svg' width='{W}' height='{}'><rect x='{BAR_X}' y='{}' width='{BAR_W}' height='{}' rx='5' fill='{THUMB}'/></svg>",
                self.layout.height,
                N(top),
                N(foot - top)
            );
            render_svg(&svg, fonts, onto, scale, (0, 0));
        }
        self.pictures = Some(p);
    }

    /// The dialog without its notes, and the notes' empty strip, at `scale`.
    fn make_pictures(&self, scale: f64, fonts: &Fonts) -> Option<Pictures> {
        let l = &self.layout;
        let origin = (
            ((X - 4.0) * scale).floor() as i32,
            ((l.top - 4.0) * scale).floor() as i32,
        );
        let size = (
            ((X + WIDTH + 4.0) * scale).ceil() as i32 - origin.0,
            ((l.bottom + 4.0) * scale).ceil() as i32 - origin.1,
        );
        let mut dialog = Pixmap::new(size.0.max(1) as u32, size.1.max(1) as u32)?;
        render_svg(&self.dialog_svg(), fonts, &mut dialog, scale, origin);
        // (Each of the dialog's rows: the opaque run through its middle, to copy, not blend;
        // a thin edge may be opaque only at the corners.)
        let width = dialog.width() as usize;
        let spans = dialog
            .data()
            .chunks_exact(width * 4)
            .map(|row| {
                let opaque = |x: usize| row[x * 4 + 3] == 255;
                let middle = width / 2;
                if !opaque(middle) {
                    return (0, 0);
                }
                let from = (0..middle).rev().find(|&x| !opaque(x)).map_or(0, |x| x + 1);
                let to = (middle..width).find(|&x| !opaque(x)).unwrap_or(width);
                (from, to)
            })
            .collect();
        // Each line's band of the strip: from under the line before's descenders to under
        // its own, so that the bands tile the strip and a line drawn later changes no other.
        let k = l.k;
        let row = |y: f64| ((y * scale).round().max(0.0)) as usize;
        let height = row(self.strip_height());
        let tops: Vec<_> = std::iter::once(0)
            .chain(
                l.lines
                    .iter()
                    .skip(1)
                    .zip(&l.lines)
                    .map(|(_, &(_, above))| row(above + (PAD + DESCENT) * k).min(height)),
            )
            .collect();
        let bands: Vec<_> = tops
            .iter()
            .enumerate()
            .map(|(i, &top)| (top, tops.get(i + 1).copied().unwrap_or(height)))
            .collect();
        let strip_x = (STRIP_X * scale).round() as i32;
        let strip = Pixmap::new(
            ((STRIP_RIGHT * scale).round() as i32 - strip_x).max(1) as u32,
            height.max(1) as u32,
        )?;
        Some(Pictures {
            scale: scale.to_bits(),
            origin,
            dialog,
            spans,
            strip,
            strip_x,
            ready: vec![false; bands.len()],
            bands,
        })
    }

    /// The notes' strip's height (drawing units): every line, room above the first and
    /// below the last.
    fn strip_height(&self) -> f64 {
        let k = self.layout.k;
        self.layout.lines.last().map_or(0.0, |&(_, y)| y) + (PAD + DESCENT) * k
    }

    /// The lines `which` of the notes, as SVG in drawing units with the strip's top at 0 (the
    /// first baseline `PAD` below it), on nothing.
    pub fn lines_svg(&self, which: &[usize]) -> String {
        let l = &self.layout;
        let k = l.k;
        let mut svg = Svg::default();
        put!(
            svg,
            "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}'><g font-family='URW Gothic' fill='{TEXT}'>",
            STRIP_RIGHT,
            N(self.strip_height())
        );
        for (line, y) in which.iter().filter_map(|&i| l.lines.get(i)) {
            let y = N(PAD * k + y);
            match line {
                Line::Head(text) => put!(
                    svg,
                    "<text x='{}' y='{y}' font-size='{}' fill='{MUTED}' letter-spacing='3'>{}</text>",
                    X + 65.0,
                    N(30.0 * k),
                    escape(text)
                ),
                Line::Item {
                    first,
                    release,
                    text,
                } => {
                    if *first {
                        put!(
                            svg,
                            "<text x='{}' y='{y}' font-size='{}'>·</text>",
                            X + 95.0,
                            N(33.0 * k)
                        );
                    }
                    put!(
                        svg,
                        "<text x='{}' y='{y}' font-size='{}'>",
                        N(X + 95.0 + l.indent),
                        N(33.0 * k)
                    );
                    if let Some(release) = release {
                        put!(svg, "<tspan fill='{MUTED}'>{}</tspan>", escape(release));
                    }
                    put!(svg, "{}</text>", escape(text));
                }
            }
        }
        svg.put(format_args!("</g></svg>"));
        svg.0
    }

    /// The dialog without its notes or the scroll bar's thumb, as SVG in the drawing's units.
    pub fn dialog_svg(&self) -> String {
        let l = &self.layout;
        let (k, h, top, bottom) = (l.k, l.height, l.top, l.bottom);
        let mut svg = Svg::default();
        put!(
            svg,
            "<svg xmlns='http://www.w3.org/2000/svg' width='{W}' height='{h}'><rect x='{X}' y='{top}' width='{WIDTH}' height='{}' rx='20' fill='{BOX}' stroke='{EDGE}' stroke-width='4'/><g font-family='URW Gothic' fill='{TEXT}'>",
            bottom - top
        );
        let title = title_y(top, k);
        put!(
            svg,
            "<text x='{}' y='{}' font-size='{}' fill='{AMBER}'>WHAT’S NEW</text>",
            X + 55.0,
            N(title),
            N(52.0 * k)
        );
        for (i, line) in l.subtitle.iter().enumerate() {
            put!(
                svg,
                "<text x='{}' y='{}' font-size='{}' fill='{MUTED}'>{}</text>",
                X + 55.0,
                N(title + 68.0 * k + 49.0 * k * i as f64),
                N(31.0 * k),
                escape(line)
            );
        }
        if self.scrolls() {
            let ((from, to), _) = self.bar();
            put!(
                svg,
                "<rect x='{BAR_X}' y='{}' width='{BAR_W}' height='{}' rx='5' fill='{WELL}'/>",
                N(from),
                N(to - from)
            );
        }
        let [x, y, width, height] = self.button();
        let note = if h < COMPACT {
            y + BUTTON_H / 2.0 + 0.37 * 27.0 * k
        } else {
            bottom - 155.0
        };
        put!(
            svg,
            "<text x='{}' y='{}' font-size='{}' fill='{MUTED}'>Shown once after an update · SETTINGS › WHAT’S NEW shows it again</text>",
            X + 55.0,
            N(note),
            N(27.0 * k)
        );
        put!(
            svg,
            "<rect x='{x}' y='{y}' width='{width}' height='{height}' rx='8' fill='{BUTTON}' stroke='{EDGE}' stroke-width='3'/><text x='{}' y='{}' font-size='{}' fill='{AMBER}' text-anchor='middle'>GOT IT</text></g></svg>",
            x + width / 2.0,
            N(y + height / 2.0 + 0.37 * 35.0 * k),
            N(35.0 * k)
        );
        svg.0
    }
}

/// The dialog's pictures at a scale (see [`View::draw`]).
#[derive(Debug)]
struct Pictures {
    scale: u64,
    /// The dialog's picture's corner in the frame (pixels).
    origin: (i32, i32),
    dialog: Pixmap,
    /// Each row's opaque pixels in the dialog's picture: from, to (none: 0, 0).
    spans: Vec<(usize, usize)>,
    /// The notes' lines, each in its band of rows (from, to), drawn when first shown.
    strip: Pixmap,
    bands: Vec<(usize, usize)>,
    ready: Vec<bool>,
    /// The strip's left edge in the frame.
    strip_x: i32,
}

/// `svg` (drawing units) drawn into `onto`, whose corner is `origin` in the frame at `scale`.
fn render_svg(svg: &str, fonts: &Fonts, onto: &mut Pixmap, scale: f64, origin: (i32, i32)) {
    let options = resvg::usvg::Options {
        fontdb: fonts.database(),
        ..Default::default()
    };
    let tree = resvg::usvg::Tree::from_str(svg, &options).expect("the notes' SVG parses");
    resvg::render(
        &tree,
        Transform::from_scale(scale as f32, scale as f32)
            .post_translate(-origin.0 as f32, -origin.1 as f32),
        &mut onto.as_mut(),
    );
}

/// `rows` of the opaque `strip` copied into `dialog` (or a frame) at (`x`, `top`), inside it.
fn paste_rows(
    dialog: &mut Pixmap,
    strip: &Pixmap,
    (x, top): (i32, i32),
    rows: std::ops::Range<i32>,
) {
    let (dw, dh) = (dialog.width() as i32, dialog.height() as i32);
    let sw = strip.width() as i32;
    let (x0, x1) = (x.max(0), (x + sw).min(dw));
    if x1 <= x0 {
        return;
    }
    let n = (x1 - x0) as usize * 4;
    for row in rows.start.max(0)..rows.end.min(strip.height() as i32) {
        let to = top + row - rows.start;
        if !(0..dh).contains(&to) {
            continue;
        }
        let from = ((row * sw + x0 - x) * 4) as usize;
        let at = ((to * dw + x0) * 4) as usize;
        dialog.data_mut()[at..at + n].copy_from_slice(&strip.data()[from..from + n]);
    }
}

/// The frame dimmed (the AUDIO & MIDI dialog's #08090b at .72) and `dialog` put over it at
/// `origin`: its opaque rows' middles copied, its edges blended.
fn over(frame: &mut Pixmap, dialog: &Pixmap, origin: (i32, i32), spans: &[(usize, usize)]) {
    static DIM: std::sync::OnceLock<[[u8; 256]; 3]> = std::sync::OnceLock::new();
    let dim = DIM.get_or_init(|| {
        let mut lut = [[0; 256]; 3];
        for (c, under) in [8.0, 9.0, 11.0].into_iter().enumerate() {
            for (v, out) in lut[c].iter_mut().enumerate() {
                *out = (v as f64 * 0.28 + under * 0.72).round() as u8;
            }
        }
        lut
    });
    let width = frame.width() as usize;
    let (dw, dh) = (dialog.width() as i32, dialog.height() as i32);
    let (fd, dd) = (frame.data_mut(), dialog.data());
    for (y, row) in fd.chunks_exact_mut(width * 4).enumerate() {
        let r = y as i32 - origin.1;
        let (from, to) = if (0..dh).contains(&r) {
            let (a, b) = spans[r as usize];
            (origin.0 + a as i32, origin.0 + b as i32)
        } else {
            (0, 0)
        };
        let (from, to) = (
            from.clamp(0, width as i32) as usize,
            to.clamp(0, width as i32) as usize,
        );
        for (x, p) in row.chunks_exact_mut(4).enumerate() {
            if (from..to).contains(&x) {
                continue;
            }
            for c in 0..3 {
                p[c] = dim[c][p[c] as usize];
            }
            let dx = x as i32 - origin.0;
            if (0..dh).contains(&r) && (0..dw).contains(&dx) {
                let at = ((r * dw + dx) * 4) as usize;
                let s = &dd[at..at + 4];
                if s[3] > 0 {
                    let keep = 255 - s[3];
                    for c in 0..4 {
                        p[c] = s[c] + crate::render::mul255(p[c], keep);
                    }
                }
            }
        }
        if to > from {
            let at = ((r * dw) as usize + from - origin.0 as usize) * 4;
            row[from * 4..to * 4].copy_from_slice(&dd[at..at + (to - from) * 4]);
        }
    }
}

/// The title's baseline: the AUDIO & MIDI dialog's, 85 units under its top.
fn title_y(top: f64, k: f64) -> f64 {
    top + 40.0 + 45.0 * k
}

/// `text` in lines no wider than `width` at `size`, broken between words, the first line
/// starting `lead` further in.
fn wrap(fonts: &Fonts, text: &str, size: f64, width: f64, lead: f64) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        let room = if lines.is_empty() {
            width - lead
        } else {
            width
        };
        if !line.is_empty() && fonts.advance(&next, size, Weight::Regular, 0.0) > room {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        } else {
            line = next;
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notes(long: bool) -> View {
        let entry = |text: &str| Entry {
            release: None,
            text: text.into(),
        };
        View::new(
            "0.2.7".into(),
            "Changes since 0.2.3, including 0.2.4, 0.2.5 and 0.2.6".into(),
            [
                (
                    "Features".into(),
                    vec![
                        Entry {
                            release: Some("0.2.6".into()),
                            text: "A readable sentence about the instrument and its controls. "
                                .repeat(if long { 6 } else { 1 }),
                        };
                        if long { 8 } else { 1 }
                    ],
                ),
                (
                    "Performance improvements".into(),
                    vec![entry("Less work on your computer.")],
                ),
                (
                    "Bug fixes".into(),
                    vec![entry("The final fix can be reached.")],
                ),
            ],
        )
    }

    #[test]
    fn notes_wrap_scroll_and_close_in_each_viewport_and_lettering_size() {
        let fonts = Fonts::new();
        for lettering in [Lettering::Standard, Lettering::Large] {
            for height in [1041.0, crate::H] {
                let mut view = notes(true);
                view.fit(&fonts, height, lettering);
                let l = &view.layout;
                // Every line inside the dialog, clear of the scroll bar.
                for (line, _) in &l.lines {
                    let (x, width) = match line {
                        Line::Head(t) => {
                            (X + 65.0, fonts.advance(t, 30.0 * l.k, Weight::Regular, 3.0))
                        }
                        Line::Item { release, text, .. } => (
                            X + 95.0 + l.indent,
                            release.as_ref().map_or(0.0, |r| {
                                fonts.advance(r, 33.0 * l.k, Weight::Regular, 0.0)
                            }) + fonts.advance(text, 33.0 * l.k, Weight::Regular, 0.0),
                        ),
                    };
                    assert!(x + width <= BAR_X - 40.0, "{line:?}");
                }
                assert!(l.lines.len() > view.shown() && view.scrolls());
                // The heading, the notes and the foot keep apart, inside the view.
                let [_, button, ..] = view.button();
                assert!(l.area.0 < l.area.1 && l.area.1 + 30.0 * l.k < button);
                assert!(l.top > 0.0 && l.bottom < height);
                view.page(1);
                assert!(view.first > 0);
                view.end();
                assert_eq!(
                    view.layout.lines[view.first + view.shown() - 1].0,
                    Line::Item {
                        first: true,
                        release: None,
                        text: "The final fix can be reached.".into()
                    }
                );
                assert_eq!(view.first + view.shown(), view.layout.lines.len());
                view.scroll(i32::MAX);
                assert_eq!(view.first, view.last());
                view.scroll(i32::MIN);
                assert_eq!(view.first, 0);
                // The scroll bar's track below the thumb pages on; a press elsewhere does
                // nothing but is the notes'.
                let ((_, to), (_, foot)) = view.bar();
                assert!(!view.press(600.0, 30.0));
                assert_eq!(view.first, 0);
                assert!(!view.press(BAR_X + 5.0, (foot + to) / 2.0));
                assert!(view.first > 0);
                let [x, y, w, h] = view.button();
                assert!(view.press(x + w / 2.0, y + h / 2.0));
                for width in [720, 1200, 2400] {
                    let scale = f64::from(width) / W;
                    let mut pixmap =
                        Pixmap::new(width, (height * scale).round() as u32).expect("frame");
                    pixmap.fill(resvg::tiny_skia::Color::WHITE);
                    view.draw(&mut pixmap, scale, &fonts);
                    // Opaque everywhere: the dialog's thin edges blended, not copied.
                    let open = pixmap.data().chunks_exact(4).position(|p| p[3] != 255);
                    assert_eq!(open, None, "{width} wide, {height} tall, {lettering:?}");
                    // The editor dimmed around the dialog.
                    assert!(pixmap.pixel(2, 2).expect("corner").red() < 100);
                }
            }
        }
    }

    #[test]
    fn short_notes_do_not_scroll_and_draw_once_for_an_unchanged_dialog() {
        let fonts = Fonts::new();
        let mut view = notes(false);
        view.fit(&fonts, crate::H, Lettering::Standard);
        assert!(!view.scrolls());
        assert_eq!(view.shown(), view.layout.lines.len());
        let all: Vec<_> = (0..view.layout.lines.len()).collect();
        let (dialog, strip) = (view.dialog_svg(), view.lines_svg(&all));
        assert!(dialog.contains("WHAT’S NEW") && dialog.contains("GOT IT"));
        assert!(dialog.contains("CA-72 0.2.7 is installed · Changes since 0.2.3"));
        assert!(strip.contains("PERFORMANCE IMPROVEMENTS"));
        assert!(strip.contains("<tspan fill='#a6a5a1'>0.2.6\u{a0}\u{a0}</tspan>"));
        assert!(!dialog.contains(&format!("fill='{WELL}'")), "no scroll bar");
        // Drawn again over a changed frame, nothing is made again.
        let mut frame = Pixmap::new(1554, 888).expect("frame");
        view.draw(&mut frame, 0.5, &fonts);
        let made = |view: &View| {
            let p = view.pictures.as_ref().expect("kept");
            (
                p.dialog.data().as_ptr(),
                p.strip.data().as_ptr(),
                p.ready.clone(),
            )
        };
        let before = made(&view);
        assert!(before.2.iter().all(|&ready| ready), "every line shows");
        frame.fill(resvg::tiny_skia::Color::WHITE);
        view.draw(&mut frame, 0.5, &fonts);
        assert_eq!(made(&view), before);
        // The editor dimmed as the AUDIO & MIDI dialog dims it; the dialog's own colour
        // inside, its button's at GOT IT.
        let at = |x: f64, y: f64| {
            let p = frame
                .pixel((x * 0.5) as u32, (y * 0.5) as u32)
                .expect("pixel");
            [p.red(), p.green(), p.blue()]
        };
        assert_eq!(at(10.0, 10.0), [77, 78, 79]);
        assert_eq!(
            at(X + 1200.0, view.layout.area.1 + 40.0),
            [0x1f, 0x22, 0x28]
        );
        let [x, y, ..] = view.button();
        assert_eq!(at(x + 20.0, y + 20.0), [0x3a, 0x30, 0x22]);
        view.fit(&fonts, crate::H, Lettering::Large);
        assert!(view.pictures.is_none(), "a new size draws it again");
    }

    /// Lines drawn as they come into view, in whatever order, make the same picture as
    /// lines drawn all at once.
    #[test]
    fn lines_drawn_as_they_show_make_the_same_picture() {
        let fonts = Fonts::new();
        let (scale, height) = (1554.0 / W, 1041.0);
        let frame = || Pixmap::new(1554, (height * scale).round() as u32).expect("frame");
        let mut scrolled = notes(true);
        scrolled.fit(&fonts, height, Lettering::Large);
        let mut seen = frame();
        for step in [0, 3, 1, 9, 2] {
            scrolled.scroll(step);
            scrolled.draw(&mut seen, scale, &fonts);
        }
        let mut direct = notes(true);
        direct.fit(&fonts, height, Lettering::Large);
        direct.first = scrolled.first;
        let mut fresh = frame();
        direct.draw(&mut fresh, scale, &fonts);
        let mut again = frame();
        scrolled.draw(&mut again, scale, &fonts);
        assert!(scrolled.first > 0);
        assert!(again.data() == fresh.data());
    }
}
