//! ULTRA's lamp over QUALITY (decisions.md R-ULTRA): a round opening in the face, a thin ring of
//! dark steel round it, closed by a camera's iris of eight curved gunmetal blades, the face's
//! warm black. Choosing ULTRA, the blades turn open from the middle; under them, an inch down in
//! a shallow well, a big lamp of the kind old valve amplifiers have, its knurled chrome bezel
//! turned back, waits; it comes up through the opening in one motion, its bezel turning home
//! and its filament lighting as it turns, ULTRA printed dark across its amber lens. Leaving
//! ULTRA plays it back. With the shutter off (the light only), the lamp always stands in the
//! ring, its bezel home, and only its light comes and goes.
//!
//! All of it from one number, how far it has gone (0 closed, or dark; 1 open and lit), which
//! the editor moves on from the time ([`OPEN_S`] and the others).

use crate::art::{COL, Layer, Sprite, TOP, ULTRA_AT};
use crate::skin::Part;
use crate::svg::{N, Svg, put};

/// The opening's radius, and its ring's width beyond it.
pub const APERTURE: f64 = 50.0;
pub const RING: f64 = 3.6;
/// The lamp's radius (its bezel's), and how far out its lens reaches, as a share of it.
pub const LAMP_R: f64 = 46.0;
const LENS: f64 = 0.78;
/// Waiting in the well, the lamp at this share of its size (an inch down), its bezel turned
/// back this far (degrees).
const DEEP: f64 = 0.9;
const TURN: f64 = 40.0;
/// The blades: how many, their edges' curve (its radius, as a share of the opening's) and how
/// far the opening turns as it opens (degrees).
const BLADES: usize = 8;
const CURVE: f64 = 1.25;
const BLADE_TURN: f64 = 72.0;
/// How long it takes, seconds: opening (the shutter, then the lamp up and lit), closing; the
/// light alone coming on, and going out.
pub const OPEN_S: f64 = 2.5;
pub const CLOSE_S: f64 = 1.8;
pub const STILL_ON_S: f64 = 1.1;
pub const STILL_OFF_S: f64 = 0.5;
/// Where its stages fall in the opening (shares of it): the blades open, then the lamp comes
/// up, turning and lighting (the owner: "one motion upwards that turns and the light bulb
/// turns on"), its light a beat behind the turn, as if the turn made the contact.
const OPEN: (f64, f64) = (0.04, 0.44);
const LIFT: (f64, f64) = (0.44, 1.0);
const LIGHT_LAG: f64 = 0.12;
/// The well's floor, a share of the opening's radius.
const FLOOR: f64 = 0.86;
/// The blades' and the ring's metal: the face's warm black (the owner: "plain sort of gunmetal
/// ... I just want the color to match closely").
const BLADE: &str = "#191612";
const RING_METAL: &str = "#201e1a";

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Out with a little overshoot: a part arriving and settling.
fn settle(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    let s = 1.7;
    1.0 + (s + 1.0) * (t - 1.0).powi(3) + s * (t - 1.0).powi(2)
}

/// A valve's warming, `warm` of the way: slow at first, a flicker or two on the way.
fn warming(warm: f64) -> f64 {
    let flick = if warm > 0.0 && warm < 1.0 {
        1.0 - 0.35 * (-((warm - 0.42) / 0.05).powi(2)).exp()
            - 0.25 * (-((warm - 0.63) / 0.04).powi(2)).exp()
    } else {
        1.0
    };
    warm.powf(1.7) * flick
}

/// Where everything is, `p` of the way open (`still`: the light only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// The blades, 0 closed to 1 gone under the ring (1 with the light only).
    pub open: f64,
    /// The lamp: risen (0 waiting in the well, 1 standing in the ring; past 1 a little as it
    /// settles), its bezel turned home (0 turned back, 1 home), lit (0 dark, 1 fully).
    pub rise: f64,
    pub twist: f64,
    pub glow: f64,
}

pub fn look(p: f64, still: bool) -> Look {
    let p = p.clamp(0.0, 1.0);
    if still {
        return Look {
            open: 1.0,
            rise: 1.0,
            twist: 1.0,
            glow: warming(p),
        };
    }
    let (l0, l1) = LIFT;
    Look {
        open: smooth(OPEN.0, OPEN.1, p),
        rise: if p < l0 { 0.0 } else { settle(l0, l1, p) },
        twist: smooth(l0, l1, p),
        glow: warming(smooth(l0 + LIGHT_LAG * (l1 - l0), l1, p)),
    }
}

fn origin() -> (f64, f64) {
    (COL + ULTRA_AT.0, TOP + ULTRA_AT.1)
}

/// The layer's extent: the ring and the lamp's light round it.
const BOUNDS: [f64; 4] = [-88.0, -88.0, 88.0, 88.0];

/// ULTRA's lamp in the worn skin, `p` of the way open (`still`: the light only).
pub fn ultra_worn(p: f64, still: bool) -> Layer {
    let l = look(p, still);
    let sc = DEEP + (1.0 - DEEP) * l.rise;
    let mut body = Svg::default();
    let mut over = Svg::default();
    let mut sprites = Vec::new();
    let shown = l.open > 0.0;
    if shown {
        well(&mut body);
        if l.rise > 0.0 {
            // Its shadow on the well's wall as it stands up out of it.
            put!(
                body,
                "<defs><radialGradient id='u-shade'><stop offset='0.75' stop-color='#000' stop-opacity='{}'/><stop offset='1' stop-color='#000' stop-opacity='0'/></radialGradient></defs><circle cx='{}' cy='{}' r='{}' fill='url(#u-shade)'/>",
                N(0.55 * l.rise.min(1.0)),
                N(3.0 * l.rise),
                N(5.0 * l.rise),
                N(LAMP_R * sc * 1.12)
            );
        }
        let d = 2.0 * APERTURE * FLOOR;
        sprites.push(picture(Part::UltraFloor, d, 1.0, 0.0, 1.0));
        let (d, deg) = (2.0 * LAMP_R, -TURN * (1.0 - l.twist));
        sprites.push(picture(Part::UltraLensOff, d, sc, 0.0, 1.0));
        if l.glow > 0.0 {
            sprites.push(picture(Part::UltraLensOn, d, sc, 0.0, l.glow));
        }
        sprites.push(picture(Part::UltraBezelOff, d, sc, deg, 1.0));
        if l.glow > 0.0 {
            sprites.push(picture(Part::UltraBezelOn, d, sc, deg, l.glow));
        }
        lettering(&mut over, sc, l.glow);
    }
    if l.open < 1.0 {
        shutter(&mut over, l.open);
    }
    ring(&mut over);
    if l.glow > 0.0 {
        halo(&mut over, l.glow);
    }
    Layer {
        origin: origin(),
        bounds: BOUNDS,
        body: body.0,
        sprites,
        over: over.0,
        ..Layer::default()
    }
}

fn picture(part: Part, d: f64, zoom: f64, deg: f64, alpha: f64) -> Sprite {
    Sprite {
        part,
        size: (d, d),
        at: (0.0, 0.0),
        deg,
        flip: (false, false),
        zoom,
        alpha,
    }
}

/// The well under the shutter: dark, its wall going down to the floor (the floor a picture),
/// lit a little on its far side, four screws round the floor.
fn well(s: &mut Svg) {
    let (r, rf) = (APERTURE, APERTURE * FLOOR);
    put!(
        s,
        "<defs><radialGradient id='u-wall' gradientUnits='userSpaceOnUse' cx='0' cy='0' r='{}'><stop offset='{}' stop-color='#0c0c0c'/><stop offset='1' stop-color='#2a2927'/></radialGradient><linearGradient id='u-far' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='#000' stop-opacity='0.55'/><stop offset='0.55' stop-color='#000' stop-opacity='0'/><stop offset='1' stop-color='#fffaf0' stop-opacity='0.12'/></linearGradient></defs><circle r='{}' fill='url(#u-wall)'/><circle r='{}' fill='url(#u-far)'/>",
        N(r),
        N(rf / r),
        N(r),
        N(r)
    );
    for k in 0..4 {
        let a = (45.0 + 90.0 * f64::from(k)).to_radians();
        let (x, y) = ((rf + 3.4) * a.cos(), (rf + 3.4) * a.sin());
        put!(
            s,
            "<circle cx='{}' cy='{}' r='2.2' fill='#3a3a38' stroke='#111' stroke-width='0.6'/><line x1='{}' y1='{}' x2='{}' y2='{}' stroke='#111' stroke-width='0.6'/>",
            N(x),
            N(y),
            N(x - 1.4),
            N(y),
            N(x + 1.4),
            N(y)
        );
    }
}

/// ULTRA printed dark across the lens above the bulb, as a pilot lamp's legend is; dark, a
/// faint light edge under it keeps it readable on the dark amber.
fn lettering(s: &mut Svg, sc: f64, glow: f64) {
    let size = 11.0 * sc;
    let y = -LAMP_R * LENS * 0.5 * sc + size * 0.36;
    for (dy, fill, a) in [
        (0.7 * sc, "#ffe8c4", 0.3 * (1.0 - glow)),
        (0.0, "#220c02", 0.72 + 0.22 * glow),
    ] {
        if a > 0.0 {
            put!(
                s,
                "<text x='0' y='{}' text-anchor='middle' font-size='{}' font-weight='700' letter-spacing='{}' fill='{fill}' fill-opacity='{}'>ULTRA</text>",
                N(y + dy),
                N(size),
                N(1.4 * sc),
                N(a)
            );
        }
    }
}

/// The ring round the opening: dark steel, thin, a soft bevel catching the light up and to
/// the left; its inner edge dark where it drops into the opening.
fn ring(s: &mut Svg) {
    let (r0, r1) = (APERTURE, APERTURE + RING);
    let annulus = format!(
        "M {} 0 A {r1} {r1} 0 1 0 {} 0 A {r1} {r1} 0 1 0 {} 0 Z M {} 0 A {r0} {r0} 0 1 1 {} 0 A {r0} {r0} 0 1 1 {} 0 Z",
        N(r1),
        N(-r1),
        N(r1),
        N(r0),
        N(-r0),
        N(r0)
    );
    put!(
        s,
        "<defs><linearGradient id='u-bevel' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='#fff8ec' stop-opacity='0.2'/><stop offset='0.5' stop-color='#fff' stop-opacity='0.03'/><stop offset='1' stop-color='#000' stop-opacity='0.4'/></linearGradient></defs><path d='{annulus}' fill='{RING_METAL}' fill-rule='evenodd'/><path d='{annulus}' fill='url(#u-bevel)' fill-rule='evenodd'/><circle r='{}' fill='none' stroke='#000' stroke-opacity='0.8' stroke-width='0.8'/>",
        N(r0 + 0.3)
    );
}

/// The lamp's light on the panel round it.
fn halo(s: &mut Svg, glow: f64) {
    let r = 1.55 * LAMP_R;
    put!(
        s,
        "<defs><radialGradient id='u-halo' gradientUnits='userSpaceOnUse' cx='0' cy='0' r='{}'><stop offset='{}' stop-color='#ff9628' stop-opacity='0'/><stop offset='{}' stop-color='#ff9628' stop-opacity='{}'/><stop offset='1' stop-color='#ff9628' stop-opacity='0'/></radialGradient></defs><circle r='{}' fill='url(#u-halo)'/>",
        N(r),
        N((APERTURE + RING - 1.0) / r),
        N((APERTURE + RING + 1.0) / r),
        N(0.16 * glow),
        N(r)
    );
}

/// An arc of the circle about (`cx`, `cy`), radius `r`, from angle `a` to `b` (radians,
/// clockwise on the panel when `cw`), as a path.
fn arc(cx: f64, cy: f64, r: f64, a: f64, b: f64, cw: bool) -> String {
    let mut d = b - a;
    if cw {
        while d < 0.0 {
            d += std::f64::consts::TAU;
        }
    } else {
        while d > 0.0 {
            d -= std::f64::consts::TAU;
        }
    }
    format!(
        "M {} {} A {} {} 0 {} {} {} {}",
        N(cx + r * a.cos()),
        N(cy + r * a.sin()),
        N(r),
        N(r),
        u8::from(d.abs() > std::f64::consts::PI),
        u8::from(cw),
        N(cx + r * b.cos()),
        N(cy + r * b.sin())
    )
}

/// The shutter `open` of the way open. The opening is where all eight blades' edges leave room:
/// each blade is what lies outside a circle (its curved edge) through two neighbouring corners
/// of the opening, which grows from the middle and turns as it grows, as a camera's iris does.
/// All the blades are one metal, so where they overlap only their edges show: each blade's edge
/// runs on past the opening across the next blade to the rim, a pinwheel when closed, the next
/// blade in its shade along it.
fn shutter(s: &mut Svg, open: f64) {
    let r = APERTURE;
    let rho = r * CURVE;
    let step = std::f64::consts::TAU / BLADES as f64;
    let a = (open * (r / (std::f64::consts::PI / BLADES as f64).cos() + 1.5)).max(0.02);
    let phi = (-90.0 + BLADE_TURN * open).to_radians();
    let corners: Vec<(f64, f64)> = (0..BLADES)
        .map(|k| {
            let t = phi + k as f64 * step;
            (a * t.cos(), a * t.sin())
        })
        .collect();
    // Each blade's edge: its circle's middle, and the angles at the two corners it joins.
    let edges: Vec<(f64, f64, f64, f64, bool)> = (0..BLADES)
        .map(|k| {
            let (v, w) = (corners[k], corners[(k + 1) % BLADES]);
            let (mx, my) = ((v.0 + w.0) / 2.0, (v.1 + w.1) / 2.0);
            let half = ((w.0 - v.0).hypot(w.1 - v.1)) / 2.0;
            let na = phi + (k as f64 + 0.5) * step;
            let h = (rho * rho - half * half).max(0.0).sqrt();
            let (cx, cy) = (mx - na.cos() * h, my - na.sin() * h);
            let t1 = (v.1 - cy).atan2(v.0 - cx);
            let t2 = (w.1 - cy).atan2(w.0 - cx);
            let mut dt = t2 - t1;
            while dt > std::f64::consts::PI {
                dt -= std::f64::consts::TAU;
            }
            while dt < -std::f64::consts::PI {
                dt += std::f64::consts::TAU;
            }
            (cx, cy, t1, t2, dt >= 0.0)
        })
        .collect();
    let disc = |rr: f64| {
        format!(
            "M {} 0 A {rr} {rr} 0 1 0 {} 0 A {rr} {rr} 0 1 0 {} 0 Z",
            N(rr),
            N(-rr),
            N(rr)
        )
    };
    let circle_at = |cx: f64, cy: f64| {
        format!(
            "M {} {} A {rho} {rho} 0 1 0 {} {} A {rho} {rho} 0 1 0 {} {} Z",
            N(cx + rho),
            N(cy),
            N(cx - rho),
            N(cy),
            N(cx + rho),
            N(cy)
        )
    };
    put!(
        s,
        "<defs><clipPath id='u-aperture'><circle r='{}'/></clipPath><linearGradient id='u-sheen' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='#dce2ec' stop-opacity='0.16'/><stop offset='0.5' stop-color='#dce2ec' stop-opacity='0.03'/><stop offset='1' stop-color='#000' stop-opacity='0.25'/></linearGradient><radialGradient id='u-rim' gradientUnits='userSpaceOnUse' cx='{}' cy='{}' r='{}' fx='{}' fy='{}'><stop offset='{}' stop-color='#000' stop-opacity='0'/><stop offset='1' stop-color='#000' stop-opacity='0.6'/></radialGradient></defs><g clip-path='url(#u-aperture)'>",
        N(r),
        N(r * 0.06),
        N(r * 0.06),
        N(r * 1.04),
        N(r * 0.12),
        N(r * 0.12),
        N(0.82 / 1.04)
    );
    // Their shade on the lamp, just inside the opening's edge.
    if open > 0.01 {
        for &(cx, cy, t1, t2, cw) in &edges {
            put!(
                s,
                "<path d='{}' fill='none' stroke='#000' stroke-opacity='0.45' stroke-width='3'/>",
                arc(cx, cy, rho - 1.2, t1, t2, cw)
            );
        }
    }
    // The blades: the disc but the opening, in their metal (one fill: no seam where they lie).
    for &(cx, cy, ..) in &edges {
        let blade = format!("{} {}", disc(r + 2.0), circle_at(cx, cy));
        put!(
            s,
            "<path d='{blade}' fill='{BLADE}' fill-rule='evenodd'/><path d='{blade}' fill='url(#u-sheen)' fill-rule='evenodd'/>"
        );
    }
    // Their edges: round the opening (the cut edge catching the light) and on across the next
    // blade to the rim, the next blade shaded along it.
    for &(cx, cy, t1, t2, cw) in &edges {
        let on = t2 + if cw { 1.0 } else { -1.0 } * 110f64.to_radians();
        put!(
            s,
            "<path d='{}' fill='none' stroke='#000' stroke-opacity='0.38' stroke-width='3'/><path d='{}' fill='none' stroke='#000' stroke-opacity='0.85' stroke-width='0.9'/><path d='{}' fill='none' stroke='#fff' stroke-opacity='0.13' stroke-width='0.5'/>",
            arc(cx, cy, rho - 1.4, t2, on, cw),
            arc(cx, cy, rho, t2, on, cw),
            arc(cx, cy, rho + 0.55, t2, on, cw)
        );
        if open > 0.01 {
            put!(
                s,
                "<path d='{}' fill='none' stroke='#fff' stroke-opacity='0.22' stroke-width='0.6'/>",
                arc(cx, cy, rho, t1, t2, cw)
            );
        }
    }
    // The ring's shade on them, under its upper left.
    put!(s, "<circle r='{}' fill='url(#u-rim)'/></g>", N(r + 2.0));
}

/// ULTRA's lamp as drawn (the drawn skin): shut, nothing (the drawing is the approved
/// mock-up's, which has no lamp); opening, the shutter in its ring; open, the lamp a chrome
/// ring round an amber lens, lit as it lights.
pub fn ultra(p: f64, still: bool) -> Layer {
    let l = look(p, still);
    let mut s = Svg::default();
    if l.open == 0.0 {
        return Layer {
            origin: origin(),
            bounds: BOUNDS,
            ..Layer::default()
        };
    }
    if l.open < 1.0 && l.rise == 0.0 {
        shutter(&mut s, l.open);
    } else {
        let sc = DEEP + (1.0 - DEEP) * l.rise;
        put!(
            s,
            "<circle r='{}' fill='#111'/><circle r='{}' fill='#bdbdb8' stroke='#6a6a66'/><circle r='{}' fill='#5a2a08'/><circle r='{}' fill='#ffa030' fill-opacity='{}'/>",
            N(APERTURE),
            N(LAMP_R * sc),
            N(LAMP_R * LENS * sc),
            N(LAMP_R * LENS * sc),
            N(l.glow)
        );
        lettering(&mut s, sc, l.glow);
    }
    ring(&mut s);
    Layer {
        origin: origin(),
        bounds: BOUNDS,
        body: s.0,
        ..Layer::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Closed, only the shutter; open and lit, the lamp at its size with its bezel home; the
    /// light only, the lamp always up, only its light changing; the light lags the turn.
    #[test]
    fn the_lamp_comes_up_turning_and_lighting() {
        let shut = look(0.0, false);
        assert_eq!((shut.open, shut.rise, shut.glow), (0.0, 0.0, 0.0));
        let up = look(1.0, false);
        assert_eq!((up.open, up.twist), (1.0, 1.0));
        assert!((up.rise - 1.0).abs() < 1e-12 && (up.glow - 1.0).abs() < 1e-12);
        let mid = look(0.6, false);
        assert!(mid.open == 1.0 && mid.rise > 0.0 && mid.twist > mid.glow);
        let still = look(0.0, true);
        assert_eq!(
            (still.open, still.rise, still.twist, still.glow),
            (1.0, 1.0, 1.0, 0.0)
        );
        assert_eq!(look(1.0, true).glow, 1.0);
        // Its layer: the shutter alone at first; the lamp's pictures once it opens.
        assert!(ultra_worn(0.0, false).sprites.is_empty());
        let open = ultra_worn(1.0, false);
        assert_eq!(open.sprites.len(), 5);
        assert!(open.sprites.iter().all(|s| (s.zoom - 1.0).abs() < 1e-12));
    }

    /// What a frame of the opening costs to draw at the window's opening size and twice it
    /// (a Retina screen): printed (`-- --ignored --nocapture`).
    #[test]
    #[ignore = "prints timings"]
    fn ultra_timings() {
        use crate::render::{Renderer, Scene};
        use crate::skin::Skin;
        for scale in [1720.0 / crate::art::W, 3440.0 / crate::art::W] {
            let mut r = Renderer::with_skin(Skin::Worn, scale, 1.0);
            let mut scene = Scene::default();
            r.render(&scene);
            let mut times = Vec::new();
            for k in 0..=160 {
                scene.ultra = f64::from(k) / 160.0;
                let t = std::time::Instant::now();
                r.render(&scene);
                times.push(t.elapsed().as_secs_f64() * 1000.0);
            }
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            let max = times.iter().copied().fold(0.0, f64::max);
            println!(
                "scale {scale:.3}: a frame of ULTRA's opening {mean:.2} ms on average, {max:.2} at most"
            );
        }
    }

    /// An arc's path goes the short way, or round, as asked.
    #[test]
    fn an_arc_goes_the_way_it_is_asked() {
        let a = arc(0.0, 0.0, 10.0, 0.0, 1.0, true);
        assert!(a.contains(" 0 0 1 "), "{a}");
        let b = arc(0.0, 0.0, 10.0, 0.0, 1.0, false);
        assert!(b.contains(" 0 1 0 "), "{b}");
    }
}
