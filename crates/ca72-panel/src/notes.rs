//! The note values printed beside oscillators 2's and 3's TIMED keys (decisions.md R-OSC-TIMED):
//! engraved notes from GNU FreeSerif's Western Musical Symbols (`third_party/gnu-freefont`, a
//! subset of the font: the notes, the augmentation dot and the double whole note; GPL-3.0 or
//! later with the font exception), at their natural proportions (the owner, 2026-10-20: "You
//! can extend the note height to regular note height"). The panel's typeface has no musical
//! signs but the eighth note. Each glyph's outline is read once, as SVG path data in the font's
//! units, y down, its notehead's middle on y 0 and its left edge on x 0.

use std::fmt::Write as _;
use std::sync::OnceLock;

use rustybuzz::ttf_parser::{Face, OutlineBuilder};

/// The subset of FreeSerif the notes come from.
static FONT: &[u8] = include_bytes!("../../../third_party/gnu-freefont/FreeSerif-notes.otf");

/// The glyphs used: a note value's sign by its value (0 a whole note to 5 a thirty-second), the
/// double whole note and the augmentation dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Note {
    /// 0 whole, 1 half, 2 quarter, 3 eighth, 4 sixteenth, 5 thirty-second.
    Value(u8),
    DoubleWhole,
    Dot,
}

impl Note {
    fn char(self) -> char {
        match self {
            Note::Value(v) => char::from_u32(0x1d15d + u32::from(v.min(5))).unwrap_or('?'),
            Note::DoubleWhole => '\u{1d15c}',
            Note::Dot => '\u{1d16d}',
        }
    }

    fn slot(self) -> usize {
        match self {
            Note::Value(v) => usize::from(v.min(5)),
            Note::DoubleWhole => 6,
            Note::Dot => 7,
        }
    }
}

/// A glyph's outline: SVG path data in the font's units (y down, the notehead's middle on y 0,
/// its left edge on x 0), and its extent (left, top, right, bottom).
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub path: String,
    pub bounds: [f64; 4],
}

/// A notehead's height in the font's units (the whole note's), and where its middle is above the
/// baseline: the glyphs' units are scaled by the sign's notehead height over it.
pub const HEAD: f64 = 200.0;
const HEAD_MIDDLE: f64 = 90.0;

struct Path {
    d: String,
}

impl OutlineBuilder for Path {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.d, "M{} {}", x, HEAD_MIDDLE as f32 - y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.d, "L{} {}", x, HEAD_MIDDLE as f32 - y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let m = HEAD_MIDDLE as f32;
        let _ = write!(self.d, "Q{} {} {} {}", x1, m - y1, x, m - y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let m = HEAD_MIDDLE as f32;
        let _ = write!(
            self.d,
            "C{} {} {} {} {} {}",
            x1,
            m - y1,
            x2,
            m - y2,
            x,
            m - y
        );
    }
    fn close(&mut self) {
        self.d.push('Z');
    }
}

/// The glyph of `note`, read from the font once.
pub fn glyph(note: Note) -> &'static Glyph {
    static GLYPHS: OnceLock<Vec<Glyph>> = OnceLock::new();
    let all = GLYPHS.get_or_init(|| {
        let face = Face::parse(FONT, 0).expect("the notes' font parses");
        [
            Note::Value(0),
            Note::Value(1),
            Note::Value(2),
            Note::Value(3),
            Note::Value(4),
            Note::Value(5),
            Note::DoubleWhole,
            Note::Dot,
        ]
        .iter()
        .map(|n| {
            let id = face.glyph_index(n.char()).expect("the note is in the font");
            let mut p = Path { d: String::new() };
            let b = face
                .outline_glyph(id, &mut p)
                .expect("the note has an outline");
            // (The left edge on x 0.)
            let x0 = f64::from(b.x_min);
            Glyph {
                path: format!("<path d='{}' transform='translate({} 0)'/>", p.d, -x0),
                bounds: [
                    0.0,
                    HEAD_MIDDLE - f64::from(b.y_max),
                    f64::from(b.x_max) - x0,
                    HEAD_MIDDLE - f64::from(b.y_min),
                ],
            }
        })
        .collect()
    });
    &all[note.slot()]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every note is in the font, its notehead on the line it is placed by, its stem as long as
    /// engraving has it (three and a half noteheads and more above its head's middle).
    #[test]
    fn the_notes_are_read_at_their_proportions() {
        let whole = glyph(Note::Value(0));
        assert!(
            (whole.bounds[3] - whole.bounds[1] - HEAD).abs() < 1.0,
            "{:?}",
            whole.bounds
        );
        assert!(
            (whole.bounds[1] + whole.bounds[3]).abs() < 2.0,
            "its middle on the line"
        );
        for v in 1..=5 {
            let g = glyph(Note::Value(v));
            assert!(g.path.starts_with("<path d='M"));
            let above = -g.bounds[1] / HEAD;
            assert!(
                (3.3..=4.2).contains(&above),
                "value {v}: {above} noteheads up"
            );
            assert!(
                (g.bounds[3] - HEAD / 2.0).abs() < 2.0,
                "value {v}: its head's foot"
            );
        }
        assert!(glyph(Note::Dot).bounds[2] < HEAD);
        assert!(glyph(Note::DoubleWhole).bounds[2] > whole.bounds[2]);
    }
}
