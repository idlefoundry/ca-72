//! What the drawers dropped from the rail share (the arpeggiator's, decisions.md R-ARP; the
//! effects', R-FX-DRAWER): one row inside the window, from under the rail over the top of
//! whichever lower page shows, between the left hand's rule and the strip's trim. Each is a plate
//! of its own: the strip's face (the worn skin's picture, laid from another place so its grain
//! does not run on from the face under it; the drawn skin's black), the rail's shadow across its
//! top as the presets' drawer has, its left edge catching the lamp, its foot in shade, and its
//! shadow cast on the page below. Its lit tabs are the strip's (`worn`), their words on patches
//! of their own laid over the caps.

use plugin_kit_materials::{Glow, Lighting, place};
use resvg::tiny_skia::{IntRect, Pixmap, PixmapPaint, Transform};
use resvg::usvg;

use super::worn::{AMBER, AMBER_LIGHT, TAB_PRINT, edge_eased, svg_over};
use super::{SECTIONS, TAB};
use crate::art::{Lettering, PANEL_H, W};
use crate::fonts::{FAMILY, Fonts};
use crate::skin::Skin;
use crate::svg::{N, Svg, colour, put};

/// A drawer's top, under the rail.
pub const TOP: f64 = PANEL_H + super::RAIL;
/// Its ends: the left hand's section's rule at its left (the rule stays seen), the strip's trim
/// at its right.
pub const LEFT: f64 = SECTIONS[1].0 + 2.0;
pub const RIGHT: f64 = W - 28.0;
/// How far below its foot its shadow reaches.
pub const SHADOW: f64 = 16.0;

/// A drawer's frame's top row in the window's (and the drawing's) pixels, open: its units are the
/// drawing's from `top_px / scale` down, so that a part drawn on the drawing's grid lands on the
/// frame's pixels exactly.
pub fn top_px(scale: f64) -> i32 {
    (TOP * scale).round() as i32
}

/// A drawer's frame's size at `scale`, `depth` deep: the drawing's width, its depth.
pub(super) fn size_at(scale: f64, depth: f64) -> (u32, u32) {
    (
        (W * scale).ceil().max(1.0) as u32,
        (depth * scale).ceil().max(1.0) as u32,
    )
}

/// What a drawer's still is made for: the scale, the skin and the lettering.
pub(super) type Look = (u64, Skin, Lettering);

/// The lettering's options for what a drawer prints (its fonts loaded).
pub(super) fn lettered() -> usvg::Options<'static> {
    let fonts = Fonts::new();
    usvg::Options {
        fontdb: fonts.database(),
        font_family: FAMILY.into(),
        ..usvg::Options::default()
    }
}

/// The plate `depth` deep at `s`: the face, and its edges (the rail's shadow across its top, its
/// left edge lit, its foot in shade).
pub(super) fn plate(s: f64, skin: Skin, depth: f64, options: &usvg::Options<'_>) -> Pixmap {
    let (w, h) = size_at(s, depth);
    let mut plate = Pixmap::new(w, h).expect("a frame of at least a pixel");
    let (x0, x1) = ((LEFT * s).round(), (RIGHT * s).round());
    // The face: the worn skin's picture, 640 units across, mirrored tile to tile, from another
    // place than the strip's under it; the drawn skin's black.
    if skin == Skin::Worn {
        let face = crate::skin::face_picture();
        let k = (640.0 * s) as f32 / face.width() as f32;
        crate::render::fill(
            &mut plate,
            &face,
            (x0, 0.0, x1 - x0, f64::from(h)),
            None,
            Transform::from_scale(k, k).post_translate((520.0 * s) as f32, (-90.0 * s) as f32),
        );
    } else {
        let mut o = Svg::default();
        put!(
            o,
            "<rect x='{}' y='0' width='{}' height='{}' fill='{}'/>",
            N(LEFT),
            N(RIGHT - LEFT),
            N(depth + 2.0),
            colour::PANEL
        );
        svg_over(options, &mut plate, s, &o.0);
    }
    let mut o = Svg::default();
    put!(
        o,
        "<defs><linearGradient id='rail-shade' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.6'/><stop offset='1' stop-color='#000' stop-opacity='0'/></linearGradient><linearGradient id='foot-shade' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0'/><stop offset='1' stop-color='#000' stop-opacity='0.5'/></linearGradient><linearGradient id='left-lit' x1='0' y1='0' x2='1' y2='0'><stop offset='0' stop-color='#ffe6c8' stop-opacity='0.14'/><stop offset='1' stop-color='#ffe6c8' stop-opacity='0'/></linearGradient></defs>"
    );
    put!(
        o,
        "<rect x='{}' y='0' width='{}' height='14' fill='url(#rail-shade)'/><rect x='{}' y='10' width='6' height='{}' fill='url(#left-lit)'/><rect x='{}' y='{}' width='{}' height='9' fill='url(#foot-shade)'/><rect x='{}' y='{}' width='{}' height='1.5' fill='#000' fill-opacity='0.55'/>",
        N(LEFT),
        N(RIGHT - LEFT),
        N(LEFT),
        N(depth - 10.0),
        N(LEFT),
        N(depth - 9.0),
        N(RIGHT - LEFT),
        N(LEFT),
        N(depth - 1.5),
        N(RIGHT - LEFT)
    );
    svg_over(options, &mut plate, s, &o.0);
    plate
}

/// The print in `body` (the frame's units) worn into the plate, as the panel's is (the worn
/// skin's; the drawn skin's laid over it).
pub(super) fn print(
    plate: &mut Pixmap,
    s: f64,
    skin: Skin,
    options: &usvg::Options<'_>,
    body: &str,
) {
    let (w, h) = (plate.width(), plate.height());
    if let (Some(mut print), Some(under)) = (
        Pixmap::new(w, h),
        IntRect::from_xywh(0, 0, w, h).and_then(|r| plate.clone_rect(r)),
    ) {
        svg_over(options, &mut print, s, body);
        if skin == Skin::Worn {
            crate::render::worn_into(&mut print, &under);
        }
        plate.draw_pixmap(
            0,
            0,
            print.as_ref(),
            &PixmapPaint {
                opacity: if skin == Skin::Worn { 0.94 } else { 1.0 },
                ..PixmapPaint::default()
            },
            Transform::identity(),
            None,
        );
    }
}

/// The lamp's light over a plate whose top left pixel is the drawing's pixel `at`, as over the
/// panel (the worn skin's).
pub(super) fn lamp(plate: &mut Pixmap, s: f64, skin: Skin, at: (i32, i32)) {
    if skin == Skin::Worn {
        crate::render::lamp_at(plate, s, at);
    }
}

/// The shadow a drawer casts on the page under its foot, softest furthest from it: `SHADOW` deep,
/// from `LEFT` across, premultiplied, laid over what is under it.
pub(super) fn cast(s: f64, options: &usvg::Options<'_>) -> Pixmap {
    let (w, _) = size_at(s, 1.0);
    let sh = (SHADOW * s).ceil().max(1.0) as u32;
    let mut p = Pixmap::new(w, sh).expect("a shadow of at least a pixel");
    let mut o = Svg::default();
    put!(
        o,
        "<defs><linearGradient id='cast' x1='0' y1='0' x2='0' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.55'/><stop offset='0.35' stop-color='#000' stop-opacity='0.28'/><stop offset='1' stop-color='#000' stop-opacity='0'/></linearGradient></defs><rect x='{}' y='0' width='{}' height='{}' fill='url(#cast)'/>",
        N(LEFT + 4.0),
        N(RIGHT - LEFT - 4.0),
        N(SHADOW)
    );
    svg_over(options, &mut p, s, &o.0);
    p
}

/// The tabs' caps lit from inside and unlit, as the strip's.
pub(super) fn tab_lighting() -> Lighting {
    let button = Pixmap::decode_png(crate::skin::BUTTON).expect("a built-in picture decodes");
    edge_eased(plugin_kit_materials::lighting(
        &button,
        TAB.0 / TAB.1,
        AMBER,
    ))
}

/// Each tab's word (its middle in the drawing's units, the frame's top `oy` in them), on a patch
/// of its own round its tab, to lay over its cap lit or not: its pixels and their top left corner
/// in the frame's.
pub(super) fn tab_words(
    options: &usvg::Options<'_>,
    s: f64,
    oy: f64,
    tabs: &[((f64, f64), &str)],
) -> Vec<(i32, i32, Pixmap)> {
    let mut words = Vec::new();
    for &((x, y), word) in tabs {
        let (left, ptop) = (
            ((x - TAB.0 / 2.0) * s).floor(),
            ((y - oy - TAB.1 / 2.0) * s).floor(),
        );
        let (pw, ph) = ((TAB.0 * s).ceil() as u32 + 2, (TAB.1 * s).ceil() as u32 + 2);
        if let Some(mut p) = Pixmap::new(pw, ph) {
            let mut o = Svg::default();
            put!(
                o,
                "<text x='{}' y='{}' font-size='17' font-weight='700' text-anchor='middle' dominant-baseline='central' fill='{TAB_PRINT}' fill-opacity='0.85'>{word}</text>",
                N(x - left / s),
                N(y - oy - ptop / s)
            );
            svg_over(options, &mut p, s, &o.0);
            words.push((left as i32, ptop as i32, p));
        }
    }
    words
}

/// The lit tabs' caps and light, kept for a scale: the lit cap resampled once, each tab's light
/// on the face round it made once where the tab is.
#[derive(Default)]
pub(super) struct LitTabs {
    cap: Option<Pixmap>,
    glows: Vec<((f64, f64), Glow)>,
}

impl LitTabs {
    /// Forgotten (another scale or look).
    pub(super) fn clear(&mut self) {
        self.cap = None;
        self.glows.clear();
    }

    /// The tab at `at` (the drawing's units; the frame's top `oy` in them) lit in `frame`: its
    /// light on the face round it, then the lit cap over the unlit.
    pub(super) fn light(
        &mut self,
        frame: &mut Pixmap,
        lighting: &Lighting,
        s: f64,
        at: (f64, f64),
        oy: f64,
    ) {
        if self.cap.is_none() {
            let px = |v: f64| (v * s).round().max(1.0) as u32;
            self.cap = Some(plugin_kit_materials::resample(
                &lighting.lit,
                px(TAB.0),
                px(TAB.1),
            ));
        }
        let local = (at.0, at.1 - oy);
        if !self.glows.iter().any(|(g, _)| *g == at)
            && let Some(g) = Glow::new(s, local, TAB, AMBER_LIGHT)
        {
            self.glows.push((at, g));
        }
        if let Some((_, g)) = self.glows.iter().find(|(g, _)| *g == at) {
            g.light(frame);
        }
        if let Some(cap) = &self.cap {
            place(frame, cap, s, local, &PixmapPaint::default());
        }
    }
}
