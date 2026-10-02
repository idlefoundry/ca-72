//! The plug-in's own controls, drawn under the panel and always with it (decisions.md, "POLY"
//! and "ANALOG and SPREAD"): not the instrument's, so not on its face. POLY (a switch), VOICES
//! (2 to 10, stepped) with how many this machine plays in real time at the host's block
//! size, VOICES marked when it asks more; ANALOG and SPREAD, each a slider (0 to 100 %) and
//! a switch that turns it off and back on at its amount. Drawn in the panel's colours and
//! lettering, at the panel's scale, `STRIP_H` panel units tall across its whole width.

use std::fmt;

use resvg::tiny_skia::{Color, Pixmap, Transform};
use resvg::usvg;

use crate::art::{self, W};
use crate::fonts::{FAMILY, Fonts, Weight};
use crate::svg::{N, Svg, colour, escape, put};

/// The strip's height, panel units.
pub const STRIP_H: f64 = 96.0;

/// One of the two amounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amount {
    Analog,
    Spread,
}

/// What a pointer finds on the strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StripTarget {
    Poly,
    /// VOICES' step down and up.
    Fewer,
    More,
    /// An amount's slider, and its switch.
    Slider(Amount),
    Switch(Amount),
}

/// What the strip shows.
#[derive(Clone, Debug, PartialEq)]
pub struct StripScene {
    pub poly: bool,
    pub voices: u32,
    /// How many voices this machine plays in real time at the host's block size (`block`
    /// frames), when known.
    pub fits: Option<(u32, u32)>,
    /// ANALOG and SPREAD, 0..1.
    pub analog: f64,
    pub spread: f64,
    pub hover: Option<StripTarget>,
}

impl Default for StripScene {
    fn default() -> Self {
        StripScene {
            poly: false,
            voices: 4,
            fits: None,
            analog: 0.0,
            spread: 0.0,
            hover: None,
        }
    }
}

// The layout, panel units across and strip units down (0 at the strip's top).
const MID: f64 = STRIP_H / 2.0;
const TEXT: f64 = 26.0;
const NAME_X: f64 = 40.0;
const POLY_X: f64 = 470.0;
const SWITCH_W: f64 = 92.0;
const SWITCH_H: f64 = 44.0;
const VOICES_X: f64 = 720.0;
const STEP_W: f64 = 46.0;
/// Where the real-time hint starts: clear of VOICES' "+" (which ends at 1032).
const HINT_X: f64 = 1062.0;
/// Each amount: its label's x, its slider's track from and to, its value's x, its switch's x.
const AMOUNTS: [(Amount, &str, f64, f64, f64, f64, f64); 2] = [
    (
        Amount::Analog,
        "ANALOG",
        1720.0,
        1880.0,
        2280.0,
        2350.0,
        2420.0,
    ),
    (
        Amount::Spread,
        "SPREAD",
        2620.0,
        2780.0,
        3180.0,
        3250.0,
        3320.0,
    ),
];
/// The track's thickness and the thumb's radius.
const TRACK: f64 = 8.0;
const THUMB: f64 = 17.0;

/// The amber the strip marks a count that asks too much with (DAW-v3's warning colour).
const WARN: &str = "#e5b94d";
const DIM: &str = "#8a909c";
const RAISED: &str = "#272b32";
const BORDER: &str = "#4a4640";
const ACCENT: &str = "#f0a030";

/// An amount's slider track: from, to.
fn track(a: Amount) -> (f64, f64) {
    AMOUNTS
        .iter()
        .find(|t| t.0 == a)
        .map_or((0.0, 1.0), |t| (t.3, t.4))
}

/// What the strip says beside VOICES: how many voices play in real time here at the host's
/// block size, and, when VOICES asks more, that it will drop out (kept short enough to end
/// before ANALOG).
fn hint(n: u32, block: u32, over: bool) -> String {
    if over {
        format!("{n} in real time here: expect dropouts")
    } else {
        format!("{n} in real time here ({block}-sample blocks)")
    }
}

/// What is at (`x`, `y`) on the strip (panel units across, strip units down).
pub fn hit(x: f64, y: f64) -> Option<StripTarget> {
    if !(0.0..STRIP_H).contains(&y) {
        return None;
    }
    let near = |y0: f64| (y - MID).abs() <= y0;
    if near(SWITCH_H / 2.0 + 6.0) && (POLY_X..POLY_X + SWITCH_W).contains(&x) {
        return Some(StripTarget::Poly);
    }
    let vx = VOICES_X + 150.0;
    if near(SWITCH_H / 2.0 + 6.0) && (vx..vx + STEP_W).contains(&x) {
        return Some(StripTarget::Fewer);
    }
    if near(SWITCH_H / 2.0 + 6.0) && (vx + STEP_W + 70.0..vx + 2.0 * STEP_W + 70.0).contains(&x) {
        return Some(StripTarget::More);
    }
    for (a, _, _, t0, t1, _, sw) in AMOUNTS {
        if near(THUMB + 8.0) && (t0 - THUMB..t1 + THUMB).contains(&x) {
            return Some(StripTarget::Slider(a));
        }
        if near(SWITCH_H / 2.0 + 6.0) && (sw..sw + SWITCH_W).contains(&x) {
            return Some(StripTarget::Switch(a));
        }
    }
    None
}

/// An amount's value (0..1) for a pointer at `x` along its slider.
pub fn slider_value(a: Amount, x: f64) -> f64 {
    let (t0, t1) = track(a);
    ((x - t0) / (t1 - t0)).clamp(0.0, 1.0)
}

/// The strip's SVG body, in panel units.
fn body(fonts: &Fonts, s: &StripScene) -> String {
    let mut out = Svg::default();
    put!(
        out,
        "<rect x='0' y='0' width='{}' height='{}' fill='{}'/>",
        N(W),
        N(STRIP_H),
        colour::PANEL
    );
    put!(
        out,
        "<line x1='0' y1='1' x2='{}' y2='1' stroke='{}' stroke-width='2'/>",
        N(W),
        colour::WOOD_DARK
    );
    let text = |out: &mut Svg, x: f64, t: &str, fill: &str, anchor: &str| {
        put!(
            out,
            "<text x='{}' y='{}' font-size='{}' fill='{fill}' text-anchor='{anchor}' letter-spacing='1'>{}</text>",
            N(x),
            N(MID + TEXT * 0.36),
            N(TEXT),
            escape(t)
        );
    };
    let switch = |out: &mut Svg, x: f64, on: bool, hover: bool| {
        put!(
            out,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='8' fill='{RAISED}' stroke='{}' stroke-width='{}'/>",
            N(x),
            N(MID - SWITCH_H / 2.0),
            N(SWITCH_W),
            N(SWITCH_H),
            if on { ACCENT } else { BORDER },
            if hover { 4 } else { 2 }
        );
        put!(
            out,
            "<text x='{}' y='{}' font-size='{}' fill='{}' text-anchor='middle'>{}</text>",
            N(x + SWITCH_W / 2.0),
            N(MID + 22.0 * 0.36),
            N(22.0),
            if on { colour::LEGEND } else { DIM },
            if on { "ON" } else { "OFF" }
        );
    };
    text(&mut out, NAME_X, "NOT THE INSTRUMENT'S:", DIM, "start");
    // POLY.
    text(&mut out, POLY_X - 20.0, "POLY", colour::LEGEND, "end");
    switch(&mut out, POLY_X, s.poly, s.hover == Some(StripTarget::Poly));
    // VOICES: the count between its steps, and what plays in real time here.
    let over = s.poly && s.fits.is_some_and(|(n, _)| s.voices > n);
    text(&mut out, VOICES_X + 130.0, "VOICES", colour::LEGEND, "end");
    let vx = VOICES_X + 150.0;
    for (x, t, target) in [
        (vx, "\u{2212}", StripTarget::Fewer),
        (vx + STEP_W + 70.0, "+", StripTarget::More),
    ] {
        put!(
            out,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='6' fill='{RAISED}' stroke='{BORDER}' stroke-width='{}'/>",
            N(x),
            N(MID - SWITCH_H / 2.0),
            N(STEP_W),
            N(SWITCH_H),
            if s.hover == Some(target) { 4 } else { 2 }
        );
        put!(
            out,
            "<text x='{}' y='{}' font-size='30' fill='{}' text-anchor='middle'>{t}</text>",
            N(x + STEP_W / 2.0),
            N(MID + 30.0 * 0.36),
            if s.poly { colour::LEGEND } else { DIM }
        );
    }
    let count = s.voices.to_string();
    text(
        &mut out,
        vx + STEP_W + 35.0,
        &count,
        if over {
            WARN
        } else if s.poly {
            colour::LEGEND
        } else {
            DIM
        },
        "middle",
    );
    if let Some((n, block)) = s.fits {
        text(
            &mut out,
            HINT_X,
            &hint(n, block, over),
            if over { WARN } else { DIM },
            "start",
        );
    }
    // ANALOG and SPREAD.
    for (a, label, lx, t0, t1, vx, sw) in AMOUNTS {
        let v = match a {
            Amount::Analog => s.analog,
            Amount::Spread => s.spread,
        };
        text(&mut out, lx, label, colour::LEGEND, "start");
        put!(
            out,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='{BORDER}'/>",
            N(t0),
            N(MID - TRACK / 2.0),
            N(t1 - t0),
            N(TRACK),
            N(TRACK / 2.0)
        );
        let x = t0 + (t1 - t0) * v.clamp(0.0, 1.0);
        put!(
            out,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='{ACCENT}'/>",
            N(t0),
            N(MID - TRACK / 2.0),
            N(x - t0),
            N(TRACK),
            N(TRACK / 2.0)
        );
        put!(
            out,
            "<circle cx='{}' cy='{}' r='{}' fill='{}' stroke='{}' stroke-width='{}'/>",
            N(x),
            N(MID),
            N(THUMB),
            colour::CAP,
            colour::CAP_DARK,
            if s.hover == Some(StripTarget::Slider(a)) {
                4
            } else {
                2
            }
        );
        text(
            &mut out,
            vx,
            &format!("{} %", (v * 100.0).round()),
            if v > 0.0 { colour::LEGEND } else { DIM },
            "middle",
        );
        switch(
            &mut out,
            sw,
            v > 0.0,
            s.hover == Some(StripTarget::Switch(a)),
        );
    }
    let _ = fonts.advance("", TEXT, Weight::Regular, 0.0);
    out.0
}

/// The strip's renderer at a scale (the panel's: pixels a panel unit).
pub struct StripRenderer {
    fonts: Fonts,
    options: usvg::Options<'static>,
    scale: f64,
    frame: Pixmap,
    shown: Option<StripScene>,
}

impl fmt::Debug for StripRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StripRenderer")
            .field("scale", &self.scale)
            .finish()
    }
}

/// The strip's size at `scale` pixels a panel unit.
pub fn strip_size(scale: f64) -> (u32, u32) {
    (
        (W * scale).ceil().max(1.0) as u32,
        (STRIP_H * scale).ceil().max(1.0) as u32,
    )
}

impl StripRenderer {
    pub fn new(scale: f64) -> Self {
        let fonts = Fonts::new();
        let options = usvg::Options {
            fontdb: fonts.database(),
            font_family: FAMILY.into(),
            ..usvg::Options::default()
        };
        let (w, h) = strip_size(scale);
        StripRenderer {
            fonts,
            options,
            scale,
            frame: Pixmap::new(w, h).expect("a strip of at least a pixel"),
            shown: None,
        }
    }

    pub fn rescale(&mut self, scale: f64) {
        if scale != self.scale {
            let (w, h) = strip_size(scale);
            self.scale = scale;
            self.frame = Pixmap::new(w, h).expect("a strip of at least a pixel");
            self.shown = None;
        }
    }

    /// The frame last drawn (premultiplied RGBA, opaque).
    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    /// Draws `scene`: whether the frame changed.
    pub fn render(&mut self, scene: &StripScene) -> bool {
        if self.shown.as_ref() == Some(scene) {
            return false;
        }
        self.shown = Some(scene.clone());
        let (w, h) = (self.frame.width(), self.frame.height());
        let view = [
            0.0,
            0.0,
            f64::from(w) / self.scale,
            f64::from(h) / self.scale,
        ];
        let doc = art::document(view, w, h, &body(&self.fonts, scene));
        self.frame.fill(Color::from_rgba8(0x1d, 0x1b, 0x1a, 0xff));
        if let Ok(tree) = usvg::Tree::from_str(&doc, &self.options) {
            resvg::render(&tree, Transform::identity(), &mut self.frame.as_mut());
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every control is found where it is drawn, and nothing between them.
    #[test]
    fn each_control_is_found_where_it_is_drawn() {
        assert_eq!(hit(POLY_X + 10.0, MID), Some(StripTarget::Poly));
        assert_eq!(hit(VOICES_X + 160.0, MID), Some(StripTarget::Fewer));
        assert_eq!(
            hit(VOICES_X + 150.0 + STEP_W + 80.0, MID),
            Some(StripTarget::More)
        );
        for (a, _, _, t0, t1, _, sw) in AMOUNTS {
            assert_eq!(hit((t0 + t1) / 2.0, MID), Some(StripTarget::Slider(a)));
            assert_eq!(hit(sw + 10.0, MID), Some(StripTarget::Switch(a)));
            assert_eq!(slider_value(a, t0 - 50.0), 0.0);
            assert_eq!(slider_value(a, t1 + 50.0), 1.0);
            assert!((slider_value(a, (t0 + t1) / 2.0) - 0.5).abs() < 1e-12);
        }
        assert_eq!(hit(NAME_X, MID), None);
        assert_eq!(hit(POLY_X + 10.0, STRIP_H + 1.0), None);
    }

    /// The real-time hint, at its longest, starts clear of VOICES' "+" and ends clear of
    /// ANALOG, measured in the strip's own face (it once ran into ANALOG).
    #[test]
    fn the_hint_clears_its_neighbours() {
        let r = StripRenderer::new(1.0);
        let plus_end = VOICES_X + 150.0 + STEP_W + 70.0 + STEP_W;
        let analog = AMOUNTS[0].2;
        for t in [hint(10, 4096, false), hint(10, 4096, true)] {
            let doc = art::document(
                [0.0, 0.0, W, STRIP_H],
                W as u32,
                STRIP_H as u32,
                &format!(
                    "<text x='{}' y='{}' font-size='{}' letter-spacing='1'>{}</text>",
                    N(HINT_X),
                    N(MID + TEXT * 0.36),
                    N(TEXT),
                    escape(&t)
                ),
            );
            let tree = usvg::Tree::from_str(&doc, &r.options).expect("the hint's document");
            let b = tree.root().abs_bounding_box();
            let (left, right) = (f64::from(b.left()), f64::from(b.right()));
            assert!(right - left > 100.0, "{t}: not measured ({b:?})");
            assert!(
                left >= plus_end + 20.0 && right <= analog - 40.0,
                "{t}: {left} to {right}, between {plus_end} and {analog}"
            );
            eprintln!(
                "{t}: {left:.0} to {right:.0} (\"+\" ends at {plus_end}, ANALOG at {analog})"
            );
        }
    }

    /// The strip draws, and draws again only when its scene changes.
    #[test]
    fn it_draws_when_its_scene_changes() {
        let mut r = StripRenderer::new(0.4);
        let mut s = StripScene::default();
        assert!(r.render(&s));
        assert!(!r.render(&s));
        s.poly = true;
        s.fits = Some((3, 256));
        s.voices = 6;
        assert!(r.render(&s));
        assert_eq!(r.frame().width(), strip_size(0.4).0);
    }
}
