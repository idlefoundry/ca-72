//! What the panel shows of modulation (decisions.md R-LFO), whatever its sources: the LFOs now,
//! the macros later, each a colour and a label. Laid over the window's frame by the editor
//! (`compose`), apart from the panel's renderer:
//!
//! - **The rings** on a modulated knob, nested (the first source outermost), each in its source's
//!   colour, an LFO's bipolar about the knob's setting: the whole swing faint, the half its
//!   positive peak reaches brighter with its end ticked, and a bright point where the source has
//!   the knob now. A macro's are one way (decisions.md R-MACROS): faint from the knob's setting to
//!   where the macro at 100 % takes it, the end ticked, bright as far as the macro is turned now,
//!   with its point there. The editor shows an LFO's only while the LFO page shows, while an
//!   assignment is dragged, or on the knob under the pointer (the owner's "option 2"); every
//!   macro's while the macros show too.
//! - **While an assignment is dragged** from a source's jack: a dotted ring round every knob that
//!   can take it, a bright one round the knob under the plug, and the cord from the jack to the
//!   plug.
//! - **The hover pop-up** over a modulated knob: one small depth dial per source in its colour
//!   (Pigments' way), a source marked SHARED where every voice takes one copy of it.
//!
//! Each knob's rings are a patch of their own, drawn again only when they change; the cord and
//! the pop-up likewise; the damage is where each was and is.

use resvg::tiny_skia::{
    Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, StrokeDash, Transform,
};
use resvg::usvg;

use crate::art::{self, W};
use crate::glow::{self, Coverage, Phosphor, hex};
use crate::svg::{N, Svg, colour, put};

/// One source's ring on a knob: its colour, its depth (-1..1, a share of the knob's travel
/// either way), where it has the knob now (its travel, 0..1), if shown, and whether it moves the
/// knob one way only (a macro's: unipolar).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    pub colour: [u8; 3],
    pub depth: f64,
    pub point: Option<f64>,
    pub one_way: bool,
}

/// A knob's rings: the control's index (`controls::CONTROLS`), its centre and its skirt's radius
/// (the drawing's units, as the look places it), where its pointer stands at its travel's ends
/// (degrees from twelve o'clock, its own sweep: `art::sweep`), its pointer's travel (0..1), its
/// rings outermost first.
#[derive(Clone, Debug, PartialEq)]
pub struct KnobRings {
    pub control: usize,
    pub at: (f64, f64),
    pub skirt: f64,
    pub sweep: (f64, f64),
    pub travel: f64,
    pub rings: Vec<Ring>,
}

impl KnobRings {
    /// The angle of its travel `t`.
    pub fn angle(&self, t: f64) -> f64 {
        angle(self.sweep, t)
    }

    /// How far its pointer turns over its whole travel (degrees).
    pub fn span(&self) -> f64 {
        self.sweep.1 - self.sweep.0
    }
}

/// An assignment being dragged from a source's jack: from the jack to the pointer, in the
/// source's colour, every knob that can take it (its centre and skirt), and the one under the
/// plug.
#[derive(Clone, Debug, PartialEq)]
pub struct Cord {
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub colour: [u8; 3],
    pub targets: Vec<((f64, f64), f64)>,
    pub over: Option<((f64, f64), f64)>,
}

/// One of the pop-up's dials: its source's colour and label, its depth (-1..1), and whether
/// every voice takes one copy of it here (MONO off on a target the voices share).
#[derive(Clone, Debug, PartialEq)]
pub struct PopDial {
    pub colour: [u8; 3],
    pub label: String,
    pub depth: f64,
    pub shared: bool,
}

/// The hover pop-up: under the knob at `knob` (its centre and skirt), how far that knob turns
/// over its travel (degrees, its sweep's span), its dials, the one under the pointer, its
/// lettering's size (the drawing's units).
#[derive(Clone, Debug, PartialEq)]
pub struct Popup {
    pub knob: ((f64, f64), f64),
    pub span: f64,
    pub dials: Vec<PopDial>,
    pub hover: Option<usize>,
    pub size: f64,
}

/// What the overlay shows.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ModScene {
    pub knobs: Vec<KnobRings>,
    pub cord: Option<Cord>,
    pub popup: Option<Popup>,
}

/// A knob's centre and its skirt's radius (the drawing's units).
pub type KnobPlace = ((f64, f64), f64);

/// How far apart nested rings are, and the outermost's inset from the skirt's edge.
const RING_STEP: f64 = 6.5;
const RING_IN: f64 = 4.0;
/// A dotted target ring's radius beyond the skirt.
const TARGET_OUT: f64 = 9.0;
/// How far a ring end answers a pointer (the drawing's units).
const END_HIT: f64 = 9.0;

/// Ring `k`'s radius of `n` on a knob of skirt `skirt`: 6.5 apart, closer where more than four
/// share it, so that all fit between the skirt's edge and the cap.
pub fn ring_radius(skirt: f64, k: usize, n: usize) -> f64 {
    let step = if n > 4 {
        RING_STEP * 3.0 / (n - 1) as f64
    } else {
        RING_STEP
    };
    skirt - RING_IN - step * k as f64
}

/// The angle (degrees from twelve o'clock) of a knob's travel `t` on its `sweep` (where its
/// pointer stands at its travel's ends: `art::sweep`).
pub fn angle(sweep: (f64, f64), t: f64) -> f64 {
    sweep.0 + (sweep.1 - sweep.0) * t.clamp(0.0, 1.0)
}

/// How far a pop-up's depth dial turns from twelve o'clock for `depth` (degrees, positive
/// clockwise) on a knob whose pointer turns `span` over its travel: as far as the ring's end is
/// from the knob's pointer (the owner, 2026-10-20: "+25% on the selector, but then the actual
/// automation goes +-12.5%": it turned at half the ring's scale, ±150 degrees for ±100 %).
pub fn dial_angle(depth: f64, span: f64) -> f64 {
    span * depth.clamp(-1.0, 1.0)
}

/// Where ring `k`'s end (its positive peak, ticked) is: the drawing's units.
pub fn ring_end(knob: &KnobRings, k: usize) -> (f64, f64) {
    let r = ring_radius(knob.skirt, k, knob.rings.len());
    let end = knob.angle(knob.travel + knob.rings[k].depth);
    let (x, y) = art::polar(r, end);
    (knob.at.0 + x, knob.at.1 + y)
}

/// The ring end at (`x`, `y`), if one is there: its knob's place in `knobs` and the ring's.
pub fn hit_ring_end(knobs: &[KnobRings], x: f64, y: f64) -> Option<(usize, usize)> {
    knobs.iter().enumerate().find_map(|(i, k)| {
        (0..k.rings.len())
            .find(|&j| {
                let (ex, ey) = ring_end(k, j);
                (x - ex).hypot(y - ey) <= END_HIT
            })
            .map(|j| (i, j))
    })
}

/// The depth a ring's end dragged to (`x`, `y`) gives: the angle from the knob's pointer to it,
/// a share of the knob's own sweep, within -1..1.
pub fn depth_at(knob: &KnobRings, (x, y): (f64, f64)) -> f64 {
    let a = (x - knob.at.0).atan2(-(y - knob.at.1)).to_degrees();
    ((a - knob.angle(knob.travel)) / knob.span()).clamp(-1.0, 1.0)
}

/// What `ring` draws on knob `k` (degrees from twelve o'clock): its faint swing's ends and its
/// ticked end, depth × the knob's own sweep from its pointer, within that sweep (R-LFO's
/// corrections); one way, from the pointer to the ticked end.
pub fn swing(k: &KnobRings, ring: &Ring) -> (f64, f64, f64) {
    let base = k.angle(k.travel);
    // (Its own sweep's ends: a knob's stops, a CONTINUOUS switch's TRIANGLE and floor.)
    let (a0, a1) = (k.sweep.0.min(k.sweep.1), k.sweep.0.max(k.sweep.1));
    let reach = k.span() * ring.depth;
    let end = (base + reach).clamp(a0, a1);
    if ring.one_way {
        (base, end, end)
    } else {
        (
            (base - reach.abs()).max(a0),
            (base + reach.abs()).min(a1),
            end,
        )
    }
}

/// The pop-up's box (left, top, width, height) and each dial's centre and radius.
pub fn popup_layout(p: &Popup) -> ((f64, f64, f64, f64), Vec<KnobPlace>) {
    let s = p.size;
    let (r, pitch) = (1.9 * s, 6.2 * s);
    let n = p.dials.len().max(1) as f64;
    let (w, h) = (pitch * n + 0.6 * s, 8.4 * s);
    let ((kx, ky), skirt) = p.knob;
    let x = (kx - w / 2.0).clamp(8.0, W - w - 8.0);
    let y = ky + skirt + 18.0;
    let dials = (0..p.dials.len())
        .map(|i| ((x + 0.3 * s + pitch * (i as f64 + 0.5), y + 3.9 * s), r))
        .collect();
    ((x, y, w, h), dials)
}

/// The pop-up's dial at (`x`, `y`), if one is there.
pub fn hit_popup(p: &Popup, x: f64, y: f64) -> Option<usize> {
    let (_, dials) = popup_layout(p);
    dials
        .iter()
        .position(|((dx, dy), r)| (x - dx).hypot(y - dy) <= r + 0.8 * p.size)
}

/// Whether (`x`, `y`) is on the pop-up.
pub fn on_popup(p: &Popup, x: f64, y: f64) -> bool {
    let ((bx, by, bw, bh), _) = popup_layout(p);
    (bx..bx + bw).contains(&x) && (by - 18.0..by + bh).contains(&y)
}

/// A patch laid over the frame: its top left corner (the frame's pixels) and pixels.
struct Patch {
    at: (i32, i32),
    pixels: Pixmap,
}

impl Patch {
    fn rect(&self) -> [i32; 4] {
        [
            self.at.0,
            self.at.1,
            self.at.0 + self.pixels.width() as i32,
            self.at.1 + self.pixels.height() as i32,
        ]
    }
}

/// The overlay's renderer at a scale, its frame beginning at the drawing's row `origin`.
pub struct ModRenderer {
    scale: f64,
    origin: i32,
    options: usvg::Options<'static>,
    knobs: Vec<(KnobRings, Patch)>,
    targets: Option<(Vec<KnobPlace>, [u8; 3], Patch)>,
    cord: Option<(Cord, Patch)>,
    popup: Option<(Popup, Patch)>,
    damages: Vec<[i32; 4]>,
    /// Where it showed before a rescale, to be drawn again under it.
    pending: Vec<[i32; 4]>,
}

impl std::fmt::Debug for ModRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModRenderer")
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

impl ModRenderer {
    pub fn new(scale: f64) -> Self {
        let fonts = crate::fonts::Fonts::new();
        ModRenderer {
            scale,
            origin: 0,
            options: usvg::Options {
                fontdb: fonts.database(),
                font_family: crate::fonts::FAMILY.into(),
                ..usvg::Options::default()
            },
            knobs: Vec::new(),
            targets: None,
            cord: None,
            popup: None,
            damages: Vec::new(),
            pending: Vec::new(),
        }
    }

    /// Drawn again at `scale`, the frame beginning at the drawing's row `origin`.
    pub fn rescale(&mut self, scale: f64, origin: i32) {
        if (scale, origin) != (self.scale, self.origin) {
            let shown = self.shown_rects();
            self.pending.extend(shown);
            (self.scale, self.origin) = (scale, origin);
            self.knobs.clear();
            self.targets = None;
            self.cord = None;
            self.popup = None;
        }
    }

    fn shown_rects(&self) -> Vec<[i32; 4]> {
        let mut out: Vec<[i32; 4]> = self.knobs.iter().map(|(_, p)| p.rect()).collect();
        out.extend(self.targets.as_ref().map(|t| t.2.rect()));
        out.extend(self.cord.as_ref().map(|c| c.1.rect()));
        out.extend(self.popup.as_ref().map(|p| p.1.rect()));
        out
    }

    /// The places the last render changed (the frame's pixels: left, top, right, bottom).
    pub fn damages(&self) -> &[[i32; 4]] {
        &self.damages
    }

    /// Whether anything is shown.
    pub fn shows(&self) -> bool {
        !self.knobs.is_empty()
            || self.targets.is_some()
            || self.cord.is_some()
            || self.popup.is_some()
    }

    /// Drawn for `scene`: whether anything changed.
    pub fn render(&mut self, scene: &ModScene) -> bool {
        let mut damage = std::mem::take(&mut self.pending);
        // The knobs' rings: each drawn again only if it changed.
        let mut kept: Vec<(KnobRings, Patch)> = Vec::new();
        let mut old = std::mem::take(&mut self.knobs);
        for k in &scene.knobs {
            if let Some(i) = old.iter().position(|(o, _)| o == k) {
                kept.push(old.swap_remove(i));
                continue;
            }
            if let Some(p) = self.rings_patch(k) {
                damage.push(p.rect());
                kept.push((k.clone(), p));
            }
        }
        damage.extend(old.iter().map(|(_, p)| p.rect()));
        self.knobs = kept;
        // The targets' dotted rings while an assignment is dragged (the bright one apart, with
        // the cord).
        let targets = scene.cord.as_ref().map(|c| (c.targets.clone(), c.colour));
        if targets.as_ref()
            != self
                .targets
                .as_ref()
                .map(|(t, c, _)| (t.clone(), *c))
                .as_ref()
        {
            damage.extend(self.targets.take().map(|t| t.2.rect()));
            if let Some((t, c)) = targets
                && let Some(p) = self.targets_patch(&t, c)
            {
                damage.push(p.rect());
                self.targets = Some((t, c, p));
            }
        }
        if scene.cord.as_ref() != self.cord.as_ref().map(|c| &c.0) {
            damage.extend(self.cord.take().map(|c| c.1.rect()));
            if let Some(c) = &scene.cord
                && let Some(p) = self.cord_patch(c)
            {
                damage.push(p.rect());
                self.cord = Some((c.clone(), p));
            }
        }
        if scene.popup.as_ref() != self.popup.as_ref().map(|p| &p.0) {
            damage.extend(self.popup.take().map(|p| p.1.rect()));
            if let Some(pp) = &scene.popup
                && let Some(p) = self.popup_patch(pp)
            {
                damage.push(p.rect());
                self.popup = Some((pp.clone(), p));
            }
        }
        self.damages = damage;
        !self.damages.is_empty()
    }

    /// The rings laid over `c` (the window's frame) inside `place`: under the drawers.
    pub fn lay_rings(&self, c: &mut Pixmap, place: [i32; 4]) {
        for (_, p) in &self.knobs {
            glow::lay(c, &p.pixels, p.at, place);
        }
    }

    /// The drag's and the pop-up's laid over `c` inside `place`: over everything but a tip, a
    /// note or a menu.
    pub fn lay_over(&self, c: &mut Pixmap, place: [i32; 4]) {
        if let Some((_, _, p)) = &self.targets {
            glow::lay(c, &p.pixels, p.at, place);
        }
        if let Some((_, p)) = &self.cord {
            glow::lay(c, &p.pixels, p.at, place);
        }
        if let Some((_, p)) = &self.popup {
            glow::lay(c, &p.pixels, p.at, place);
        }
    }

    /// A patch for the drawing's rectangle (left, top, right, bottom) and where it is in the
    /// frame; the pixels' offset from the drawing's.
    fn blank(&self, (x0, y0, x1, y1): (f64, f64, f64, f64)) -> Option<(Pixmap, (i32, i32))> {
        let s = self.scale;
        let (px0, py0) = ((x0 * s).floor() as i32, (y0 * s).floor() as i32);
        let (px1, py1) = ((x1 * s).ceil() as i32, (y1 * s).ceil() as i32);
        Some((
            Pixmap::new((px1 - px0).max(1) as u32, (py1 - py0).max(1) as u32)?,
            (px0, py0),
        ))
    }

    fn rings_patch(&self, k: &KnobRings) -> Option<Patch> {
        let s = self.scale;
        let m = k.skirt + 14.0;
        let (mut p, (px0, py0)) = self.blank((k.at.0 - m, k.at.1 - m, k.at.0 + m, k.at.1 + m))?;
        let local = |(x, y): (f64, f64)| ((x * s) as f32 - px0 as f32, (y * s) as f32 - py0 as f32);
        let (w, h) = (p.width(), p.height());
        let n = k.rings.len();
        let base = k.angle(k.travel);
        for (j, ring) in k.rings.iter().enumerate() {
            let r = ring_radius(k.skirt, j, n);
            let ph = Phosphor::of(ring.colour);
            let line = mix(ring.colour, [255; 3], 0.25);
            let glow_c = mix(ring.colour, ph.halo, 0.5);
            let (lo, hi, end) = swing(k, ring);
            let arc = |a0: f64, a1: f64| arc_path(k.at, r, a0, a1, &local);
            // The whole swing, faint (one way: to where 100 % takes the knob).
            if let Some(path) = arc(lo, hi) {
                stroke(
                    &mut p,
                    &path,
                    line,
                    if ring.one_way { 0.4 } else { 0.38 },
                    2.2 * s,
                    None,
                );
            }
            // The half its positive peak reaches, glowing, its end ticked; one way, as far as
            // the source has it now.
            let bright = match (ring.one_way, ring.point) {
                (true, Some(t)) => Some(k.angle(t)),
                (true, None) => None,
                (false, _) => Some(end),
            };
            if let Some(path) = bright.and_then(|b| arc(base, b)) {
                let mut g = Coverage::new(w, h);
                g.stroke(&path, (5.0 * s) as f32, 0.55);
                g.blur((2.2 * s) as f32);
                g.tint(&mut p, glow_c, 1.0, (0, 0));
                stroke(&mut p, &path, line, 1.0, 2.6 * s, None);
            }
            let (ex0, ey0) = art::polar(r - 3.5, end);
            let (ex1, ey1) = art::polar(r + 3.5, end);
            let mut pb = PathBuilder::new();
            let (a, b) = (
                local((k.at.0 + ex0, k.at.1 + ey0)),
                local((k.at.0 + ex1, k.at.1 + ey1)),
            );
            pb.move_to(a.0, a.1);
            pb.line_to(b.0, b.1);
            if let Some(path) = pb.finish() {
                stroke(&mut p, &path, ph.core, 1.0, 2.2 * s, None);
            }
            // Where the source has the knob now.
            if let Some(t) = ring.point {
                let (x, y) = art::polar(r, k.angle(t));
                let c = local((k.at.0 + x, k.at.1 + y));
                let mut g = Coverage::new(w, h);
                g.disc(c, (6.0 * s) as f32, 0.7);
                g.blur((2.2 * s) as f32);
                g.tint(&mut p, glow_c, 1.0, (0, 0));
                fill_disc(&mut p, c, (2.8 * s) as f32, mix(ph.core, [255; 3], 0.5));
            }
        }
        Some(Patch {
            at: (px0, py0 - self.origin),
            pixels: p,
        })
    }

    fn targets_patch(&self, targets: &[((f64, f64), f64)], colour: [u8; 3]) -> Option<Patch> {
        if targets.is_empty() {
            return None;
        }
        let s = self.scale;
        let mut b = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for &((x, y), r) in targets {
            let m = r + TARGET_OUT + 4.0;
            b = (
                b.0.min(x - m),
                b.1.min(y - m),
                b.2.max(x + m),
                b.3.max(y + m),
            );
        }
        let (mut p, (px0, py0)) = self.blank(b)?;
        let line = mix(colour, [255; 3], 0.25);
        for &((x, y), r) in targets {
            let c = ((x * s) as f32 - px0 as f32, (y * s) as f32 - py0 as f32);
            if let Some(path) = PathBuilder::from_circle(c.0, c.1, ((r + TARGET_OUT) * s) as f32) {
                let dash = StrokeDash::new(vec![(4.0 * s) as f32, (6.0 * s) as f32], 0.0);
                stroke(&mut p, &path, line, 0.8, 3.0 * s, dash);
            }
        }
        Some(Patch {
            at: (px0, py0 - self.origin),
            pixels: p,
        })
    }

    fn cord_patch(&self, c: &Cord) -> Option<Patch> {
        let s = self.scale;
        let (jx, jy) = c.from;
        let (px, py) = (c.to.0 + 70.0, c.to.1 + 92.0);
        // (The cord sags from the jack to the plug held a little below and right of the pointer.)
        let ctrl = [
            (jx + 40.0, jy - 120.0),
            (px - 260.0, py + 560.0),
            (px, py + 46.0),
        ];
        let mut b = (jx.min(px), jy.min(py), jx.max(px), jy.max(py));
        for (x, y) in ctrl {
            b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
        }
        if let Some(((x, y), r)) = c.over {
            let m = r + TARGET_OUT + 10.0;
            b = (
                b.0.min(x - m),
                b.1.min(y - m),
                b.2.max(x + m),
                b.3.max(y + m),
            );
        }
        let b = (b.0 - 40.0, b.1 - 80.0, b.2 + 40.0, b.3 + 40.0);
        let (mut p, (px0, py0)) = self.blank(b)?;
        let local = |(x, y): (f64, f64)| ((x * s) as f32 - px0 as f32, (y * s) as f32 - py0 as f32);
        let ph = Phosphor::of(c.colour);
        // The bright ring round the knob under the plug.
        if let Some(((x, y), r)) = c.over {
            let ctr = local((x, y));
            if let Some(path) =
                PathBuilder::from_circle(ctr.0, ctr.1, ((r + TARGET_OUT) * s) as f32)
            {
                let (w, h) = (p.width(), p.height());
                let mut g = Coverage::new(w, h);
                g.stroke(&path, (7.0 * s) as f32, 0.7);
                g.blur((2.5 * s) as f32);
                g.tint(&mut p, mix(c.colour, ph.halo, 0.5), 1.0, (0, 0));
                stroke(&mut p, &path, ph.core, 1.0, 2.6 * s, None);
            }
        }
        let mut pb = PathBuilder::new();
        let a = local((jx, jy));
        pb.move_to(a.0, a.1);
        let (c1, c2, e) = (local(ctrl[0]), local(ctrl[1]), local(ctrl[2]));
        pb.cubic_to(c1.0, c1.1, c2.0, c2.1, e.0, e.1);
        let path = pb.finish()?;
        let shadow = path.clone().transform(Transform::from_translate(
            (8.0 * s) as f32,
            (14.0 * s) as f32,
        ))?;
        stroke(&mut p, &shadow, [0, 0, 0], 0.25, 14.0 * s, None);
        stroke(&mut p, &path, [11, 11, 10], 1.0, 10.0 * s, None);
        if let Some(hl) = path.clone().transform(Transform::from_translate(
            (-2.0 * s) as f32,
            (-2.0 * s) as f32,
        )) {
            stroke(&mut p, &hl, [255, 255, 255], 0.16, 2.4 * s, None);
        }
        // The plug: its sleeve, its barrel, its tip, pointing up at the knob.
        let mut o = Svg::default();
        put!(
            o,
            "<defs><linearGradient id='plug' x1='0' y1='0' x2='1' y2='0'><stop offset='0' stop-color='#6d6a64'/><stop offset='0.35' stop-color='#e6e2d9'/><stop offset='0.6' stop-color='#9a968e'/><stop offset='1' stop-color='#3d3b37'/></linearGradient></defs><g transform='translate({} {}) rotate(-35)'><rect x='-9' y='-6' width='18' height='58' rx='4' fill='#151413'/><rect x='-7' y='-46' width='14' height='42' rx='2' fill='url(#plug)'/><rect x='-3.2' y='-66' width='6.4' height='22' rx='3' fill='url(#plug)'/><rect x='-7' y='-30' width='14' height='3' fill='#2a2825'/><rect x='-9' y='14' width='18' height='4' fill='{}' fill-opacity='0.85'/></g>",
            N(px - f64::from(px0) / s),
            N(py - f64::from(py0) / s),
            hex(c.colour)
        );
        self.svg_on(&mut p, &o.0);
        Some(Patch {
            at: (px0, py0 - self.origin),
            pixels: p,
        })
    }

    fn popup_patch(&self, pp: &Popup) -> Option<Patch> {
        let s = self.scale;
        let ((x, y, w, h), dials) = popup_layout(pp);
        let m = 8.0;
        let (mut p, (px0, py0)) = self.blank((x - m, y - m, x + w + m, y + h + m))?;
        let (ox, oy) = (f64::from(px0) / s, f64::from(py0) / s);
        let size = pp.size;
        let mut o = Svg::default();
        put!(
            o,
            "<rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='#000' fill-opacity='0.3'/><rect x='{}' y='{}' width='{}' height='{}' rx='{}' fill='#1f2228' fill-opacity='0.96' stroke='#4a4640' stroke-width='{}'/>",
            N(x - ox + 2.0),
            N(y - oy + 4.0),
            N(w),
            N(h),
            N(0.5 * size),
            N(x - ox),
            N(y - oy),
            N(w),
            N(h),
            N(0.5 * size),
            N(0.12 * size)
        );
        for (i, (d, ((cx, cy), r))) in pp.dials.iter().zip(&dials).enumerate() {
            let (cx, cy) = (cx - ox, cy - oy);
            let c = hex(d.colour);
            let core = hex(Phosphor::of(d.colour).core);
            // Its well, the depth's arc from twelve o'clock (positive clockwise) on the knob's
            // scale, its pointer.
            put!(
                o,
                "<circle cx='{}' cy='{}' r='{}' fill='#0b0b0c' stroke='#ffffff' stroke-opacity='{}' stroke-width='{}'/>",
                N(cx),
                N(cy),
                N(*r),
                if pp.hover == Some(i) { "0.35" } else { "0.12" },
                N(0.1 * size)
            );
            let a = dial_angle(d.depth, pp.span);
            if a.abs() > 0.5 {
                let rr = r - 0.25 * size;
                let (x0, y0) = art::polar(rr, 0.0);
                let (x1, y1) = art::polar(rr, a);
                put!(
                    o,
                    "<path d='M {} {} A {} {} 0 {} {} {} {}' fill='none' stroke='{c}' stroke-width='{}' stroke-linecap='round'/>",
                    N(cx + x0),
                    N(cy + y0),
                    N(rr),
                    N(rr),
                    u8::from(a.abs() > 180.0),
                    u8::from(a > 0.0),
                    N(cx + x1),
                    N(cy + y1),
                    N(0.32 * size)
                );
            }
            let (px, py) = art::polar(r * 0.55, a);
            put!(
                o,
                "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{core}' stroke-width='{}' stroke-linecap='round'/>",
                N(cx),
                N(cy),
                N(cx + px),
                N(cy + py),
                N(0.18 * size)
            );
            let label = |o: &mut Svg, ty: f64, t: &str, fill: &str, k: f64| {
                put!(
                    o,
                    "<text x='{}' y='{}' font-size='{}' font-weight='700' text-anchor='middle' dominant-baseline='central' fill='{fill}'>{}</text>",
                    N(cx),
                    N(ty),
                    N(size * k),
                    crate::svg::escape(t)
                );
            };
            label(&mut o, cy - r - 0.8 * size, &d.label, &c, 0.78);
            label(
                &mut o,
                cy + r + 0.85 * size,
                &percent(d.depth),
                colour::LEGEND,
                0.82,
            );
            if d.shared {
                label(&mut o, cy + r + 1.9 * size, "SHARED", "#8a909c", 0.62);
            }
        }
        self.svg_on(&mut p, &o.0);
        Some(Patch {
            at: (px0, py0 - self.origin),
            pixels: p,
        })
    }

    /// `body` (the patch's units: the drawing's, from its top left corner) drawn over `p`.
    fn svg_on(&self, p: &mut Pixmap, body: &str) {
        let s = self.scale;
        let view = [
            0.0,
            0.0,
            f64::from(p.width()) / s,
            f64::from(p.height()) / s,
        ];
        let doc = art::document(view, p.width(), p.height(), body);
        if let Ok(tree) = usvg::Tree::from_str(&doc, &self.options) {
            resvg::render(&tree, Transform::identity(), &mut p.as_mut());
        }
    }
}

/// A depth as the pop-up, the tip and the menu print it: `+40 %`, `–25 %`, `0 %`.
pub fn percent(depth: f64) -> String {
    let v = (depth * 100.0).round();
    if v > 0.0 {
        format!("+{v:.0} %")
    } else if v < 0.0 {
        format!("\u{2013}{:.0} %", -v)
    } else {
        "0 %".into()
    }
}

fn mix(a: [u8; 3], b: [u8; 3], k: f64) -> [u8; 3] {
    std::array::from_fn(|i| {
        (f64::from(a[i]) + (f64::from(b[i]) - f64::from(a[i])) * k).round() as u8
    })
}

/// An arc about `at` of radius `r` from angle `a0` to `a1` (degrees from twelve o'clock), as a
/// polyline in a patch's pixels (`local`).
fn arc_path(
    at: (f64, f64),
    r: f64,
    a0: f64,
    a1: f64,
    local: &dyn Fn((f64, f64)) -> (f32, f32),
) -> Option<resvg::tiny_skia::Path> {
    let (lo, hi) = if a0 <= a1 { (a0, a1) } else { (a1, a0) };
    if hi - lo < 0.2 {
        return None;
    }
    let n = ((hi - lo) / 2.0).ceil().max(1.0) as usize;
    let mut pb = PathBuilder::new();
    for i in 0..=n {
        let a = lo + (hi - lo) * i as f64 / n as f64;
        let (x, y) = art::polar(r, a);
        let p = local((at.0 + x, at.1 + y));
        if i == 0 {
            pb.move_to(p.0, p.1);
        } else {
            pb.line_to(p.0, p.1);
        }
    }
    pb.finish()
}

fn stroke(
    p: &mut Pixmap,
    path: &resvg::tiny_skia::Path,
    c: [u8; 3],
    alpha: f64,
    width: f64,
    dash: Option<StrokeDash>,
) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba8(
        c[0],
        c[1],
        c[2],
        (alpha * 255.0).round() as u8,
    ));
    paint.anti_alias = true;
    p.stroke_path(
        path,
        &paint,
        &Stroke {
            width: width.max(0.6) as f32,
            line_cap: LineCap::Round,
            dash,
            ..Stroke::default()
        },
        Transform::identity(),
        None,
    );
}

fn fill_disc(p: &mut Pixmap, (x, y): (f32, f32), r: f32, c: [u8; 3]) {
    if let Some(path) = PathBuilder::from_circle(x, y, r.max(0.5)) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(c[0], c[1], c[2], 255);
        paint.anti_alias = true;
        p.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::CONTROLS;

    fn knob() -> KnobRings {
        KnobRings {
            control: 29,
            at: (2081.0, 170.0),
            skirt: art::STD.skirt,
            sweep: art::KNOB_SWEEP,
            travel: 0.5,
            rings: vec![
                Ring {
                    colour: [0xff, 0xb0, 0x20],
                    depth: 0.4,
                    point: Some(0.6),
                    one_way: false,
                },
                Ring {
                    colour: [0x3c, 0xe8, 0xa0],
                    depth: -0.25,
                    point: None,
                    one_way: true,
                },
            ],
        }
    }

    /// A ring's end is found where it is drawn, and dragged there it gives the ring's depth; the
    /// rings nest inside the skirt; depths print as the panel's menus do.
    #[test]
    fn a_rings_end_is_found_and_gives_its_depth() {
        let k = knob();
        for j in 0..2 {
            let (x, y) = ring_end(&k, j);
            assert_eq!(hit_ring_end(std::slice::from_ref(&k), x, y), Some((0, j)));
            assert!((depth_at(&k, (x, y)) - k.rings[j].depth).abs() < 1e-9);
        }
        assert!(ring_radius(k.skirt, 3, 4) > art::STD.skirt * 0.5);
        assert!(ring_radius(k.skirt, 7, 8) > art::STD.skirt * 0.5);
        assert_eq!(percent(0.4), "+40 %");
        assert_eq!(percent(-0.25), "\u{2013}25 %");
        assert_eq!(percent(0.0), "0 %");
    }

    /// The pop-up's dial and the ring read a depth alike: the dial turns from twelve o'clock as
    /// far as the ring's ticked end is from the knob's pointer, +25 % a quarter of the knob's
    /// sweep (the owner, 2026-10-20, saw it turn half as far), a knob's or a WAVEFORM's.
    #[test]
    fn the_popups_dial_turns_as_far_as_the_rings_end() {
        for sweep in [art::KNOB_SWEEP, art::sweep(&CONTROLS[waveform()].kind)] {
            for d in [0.25, -0.25, 0.1, 0.4, -0.5] {
                let k = KnobRings {
                    sweep,
                    rings: vec![Ring {
                        depth: d,
                        ..knob().rings[0]
                    }],
                    ..knob()
                };
                let (x, y) = ring_end(&k, 0);
                let end = (x - k.at.0).atan2(-(y - k.at.1)).to_degrees();
                assert!(
                    (dial_angle(d, k.span()) - (end - k.angle(k.travel))).abs() < 1e-9,
                    "{d}"
                );
            }
        }
        assert_eq!(dial_angle(0.25, 300.0), 75.0);
        assert_eq!(dial_angle(-1.0, 300.0), -300.0);
    }

    fn waveform() -> usize {
        CONTROLS
            .iter()
            .position(|c| c.param == "osc1_waveform")
            .expect("OSC 1 WAVEFORM")
    }

    /// Each control's rings take its own sweep, where its pointer stands at its travel's ends
    /// (R-LFO's corrections: the owner, 2026-10-20, "the % seems to be calculated as the same
    /// radial distance for each knob regardless the total active radius that knob has"). Every
    /// knob turns 300 degrees stop to stop, as its ticks run; the three WAVEFORM switches,
    /// CONTINUOUS, 199.325 from TRIANGLE to the floor, their pointers as drawn. A ring at +25 %
    /// draws exactly a quarter of the sweep each way, its ticked end there and dragged there
    /// giving +25 %, on ATTACK TIME, CUTOFF and a WAVEFORM; a one way ring (a macro's) a quarter
    /// one way; at a sweep's end the swing stops there.
    #[test]
    fn a_ring_takes_its_knobs_own_sweep() {
        use crate::controls::{Kind, Legends};
        for c in CONTROLS.iter() {
            match c.kind {
                Kind::Knob { .. } => {
                    assert_eq!(art::sweep(&c.kind), (-150.0, 150.0), "{}", c.param)
                }
                Kind::Selector(Legends::Waves12 | Legends::Waves3) => {
                    let (a0, a1) = art::sweep(&c.kind);
                    assert_eq!(a0, -83.75, "{}", c.param);
                    assert!((a1 - a0 - 5.95 * 33.5).abs() < 1e-9, "{}", c.param);
                    for t in [0.0, 0.3, 0.5, 1.0] {
                        let drawn = art::shape_angle(t * art::SHAPE_PLACES);
                        assert!((angle((a0, a1), t) - drawn).abs() < 1e-9, "{}", c.param);
                    }
                }
                _ => {}
            }
        }
        let of = |param: &str| {
            let c = CONTROLS.iter().find(|c| c.param == param).expect(param);
            (c.kind, art::sweep(&c.kind))
        };
        for param in ["filter_attack", "cutoff", "osc1_waveform"] {
            let (kind, sweep) = of(param);
            let quarter = 0.25 * (sweep.1 - sweep.0);
            for travel in [0.5, 0.3] {
                for one_way in [false, true] {
                    let ring = Ring {
                        depth: 0.25,
                        one_way,
                        ..knob().rings[0]
                    };
                    let k = KnobRings {
                        skirt: art::skirt(&kind),
                        sweep,
                        travel,
                        rings: vec![ring],
                        ..knob()
                    };
                    let base = k.angle(travel);
                    let (lo, hi, end) = swing(&k, &ring);
                    let want = if one_way {
                        (base, base + quarter)
                    } else {
                        (base - quarter, base + quarter)
                    };
                    assert!(
                        (lo - want.0).abs() < 1e-9 && (hi - want.1).abs() < 1e-9,
                        "{param} at {travel}: {lo}..{hi}, not {want:?}"
                    );
                    assert!((end - (base + quarter)).abs() < 1e-9, "{param}");
                    let (x, y) = ring_end(&k, 0);
                    let at = (x - k.at.0).atan2(-(y - k.at.1)).to_degrees();
                    assert!((at - end).abs() < 1e-9, "{param}: its end where drawn");
                    assert!((depth_at(&k, (x, y)) - 0.25).abs() < 1e-9, "{param}");
                }
            }
            // (At the travel's foot the swing stops at the sweep's start.)
            let k = KnobRings {
                sweep,
                travel: 0.1,
                rings: vec![Ring {
                    depth: 0.25,
                    ..knob().rings[0]
                }],
                ..knob()
            };
            let (lo, hi, _) = swing(&k, &k.rings[0]);
            assert!((lo - sweep.0).abs() < 1e-9, "{param}");
            assert!((hi - k.angle(0.35)).abs() < 1e-9, "{param}");
        }
    }

    /// Only what changed is drawn again: the same scene twice draws nothing; a ring's point
    /// moving damages only its knob; the pop-up's dials are found where they are drawn.
    #[test]
    fn only_what_changed_is_drawn_again() {
        let mut r = ModRenderer::new(0.44);
        let mut scene = ModScene {
            knobs: vec![knob()],
            ..ModScene::default()
        };
        assert!(r.render(&scene));
        assert!(!r.render(&scene));
        scene.knobs[0].rings[0].point = Some(0.7);
        assert!(r.render(&scene));
        assert_eq!(r.damages().len(), 2, "where it was and is");
        let popup = Popup {
            knob: ((2081.0, 170.0), art::STD.skirt),
            span: 300.0,
            dials: vec![
                PopDial {
                    colour: [0xff, 0xb0, 0x20],
                    label: "LFO 1".into(),
                    depth: 0.4,
                    shared: false,
                },
                PopDial {
                    colour: [0x3c, 0xe8, 0xa0],
                    label: "LFO 2".into(),
                    depth: -0.25,
                    shared: true,
                },
            ],
            hover: None,
            size: 27.0,
        };
        let (_, dials) = popup_layout(&popup);
        for (i, ((x, y), _)) in dials.iter().enumerate() {
            assert_eq!(hit_popup(&popup, *x, *y), Some(i));
            assert!(on_popup(&popup, *x, *y));
        }
        scene.popup = Some(popup);
        assert!(r.render(&scene));
        assert!(r.shows());
    }
}
