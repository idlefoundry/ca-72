//! QUALITY's opening (decisions.md R-ULTRA): a round opening in the face over QUALITY, a thin
//! ring of dark steel round it, closed by a camera's iris of eight curved gunmetal blades, the
//! face's warm black. Under it, an inch down in a shallow well, waits what each setting shows,
//! one at a time: at LO a hamster in an old brass wheel, at HI a big lamp of the kind old valve
//! amplifiers have, its amber lens plain, at ULTRA its twin in cobalt glass with a miniature
//! Tesla coil in it. Choosing one, the blades turn open from the middle and it comes up through
//! the opening in one motion: a lamp turning its knurled bezel home and lighting as it turns,
//! the hamster's wheel coming forward as he starts to run. Choosing another, what is up goes
//! back down and the blades close before they open on the next. With the shutter off (always
//! open), what each setting shows stands in the ring, fading in and out.
//!
//! The lamps follow the synth (the owner: "Perhaps something that can match the intensity of
//! the synth itself?"): dim at rest, brightening as it plays, the filament flaring, a warm glow
//! spreading a little round the lamp; the Tesla lamp's coil throws blue lightning to the inside
//! of its glass as the synth sounds, more of it, longer and brighter, the louder it is. The
//! hamster runs while the synth sounds and dozes off when it has been quiet a while. At rest
//! nothing moves.
//!
//! All of it from [`Opening`], which the editor moves on from the time and the synth's level.

use crate::art::{COL, Layer, Sprite, TOP, ULTRA_AT};
use crate::skin::Part;
use crate::svg::{N, Svg, put};

/// The opening's radius, and its ring's width beyond it.
pub const APERTURE: f64 = 50.0;
pub const RING: f64 = 3.6;
/// The lamps' radius (their bezels'), and how far out their lenses reach, as a share of it.
pub const LAMP_R: f64 = 46.0;
const LENS: f64 = 0.78;
/// Waiting in the well, a lamp at this share of its size (an inch down), its bezel turned
/// back this far (degrees).
const DEEP: f64 = 0.9;
const TURN: f64 = 40.0;
/// The blades: how many, their edges' curve (its radius, as a share of the opening's) and how
/// far the opening turns as it opens (degrees).
const BLADES: usize = 8;
const CURVE: f64 = 1.25;
const BLADE_TURN: f64 = 72.0;
/// How long it takes, seconds: opening (the shutter, then what comes up), closing; with the
/// shutter off, what stands in the ring coming (a lamp lighting), and going.
pub const OPEN_S: f64 = 2.5;
pub const CLOSE_S: f64 = 1.8;
pub const STILL_ON_S: f64 = 1.1;
pub const STILL_OFF_S: f64 = 0.5;
/// Where its stages fall in the opening (shares of it): the blades open, then what waits comes
/// up, a lamp turning and lighting (the owner: "one motion upwards that turns and the light
/// bulb turns on"), its light a beat behind the turn, as if the turn made the contact.
const OPEN: (f64, f64) = (0.04, 0.44);
const LIFT: (f64, f64) = (0.44, 1.0);
const LIGHT_LAG: f64 = 0.12;
/// With the shutter off, how much of its coming it takes to fade in.
const FADE: f64 = 0.35;
/// The well's floor, a share of the opening's radius.
const FLOOR: f64 = 0.86;
/// The blades' and the ring's metal: the face's warm black (the owner: "plain sort of gunmetal
/// ... I just want the color to match closely").
const BLADE: &str = "#191612";
const RING_METAL: &str = "#201e1a";

/// A lamp's light at rest, a share of its full; the filament's flare and the glow round the
/// lamp at the synth's loudest (the mock-up's, matched by eye).
const IDLE: f64 = 0.28;
const FLARE: f64 = 0.65;
const SPILL: f64 = 0.16;

/// The hamster's wheel: its radius (the opening's is 50), its picture's rim (a share of its
/// half width), waiting in the well at this share of its size; the hamster's length, his
/// feet's place (a share of the wheel's radius), how far up the rim he climbs at a run
/// (degrees), and asleep, his width.
const WHEEL_R: f64 = 45.0;
const WHEEL_RIM: f64 = 0.99;
const WHEEL_DEEP: f64 = 0.72;
const HAMSTER_LEN: f64 = 0.95;
const FOOT: f64 = 0.91;
const CLIMB: f64 = 12.0;
const ASLEEP_W: f64 = 0.6;
/// The hamster's pictures' sizes, pixels: a frame of his run, him asleep.
const FRAME_PX: (f64, f64) = (230.0, 120.0);
const ASLEEP_PX: (f64, f64) = (300.0, 289.0);

/// What QUALITY's opening shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Opening {
    /// How far each has come (0 down in the well under the shutter, or gone; 1 up, lit): LO's
    /// hamster, HI's lamp, ULTRA's Tesla lamp. One at a time is past 0.
    pub hamster: f64,
    pub lamp: f64,
    pub coil: f64,
    /// The shutter off (always open).
    pub still: bool,
    /// The synth's level (0 silent to 1 loud) as a lamp's filament follows it.
    pub level: f64,
    /// The hamster: how fast he runs (0 standing to 1 a full run), the frame of his run, how
    /// far asleep (0 awake to 1), and his wheel's turn, degrees.
    pub run: f64,
    pub stride: u8,
    pub asleep: f64,
    pub turn: f64,
    /// The Tesla coil's lightning: its clock, seconds (it moves only while the coil sparks).
    pub spark: f64,
}

/// Which lamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lamp {
    Amber,
    Tesla,
}

impl Lamp {
    /// Its lens and bezel, dark and lit.
    fn parts(self) -> [Part; 4] {
        match self {
            Lamp::Amber => [
                Part::UltraLensOff,
                Part::UltraLensOn,
                Part::UltraBezelOff,
                Part::UltraBezelOn,
            ],
            Lamp::Tesla => [
                Part::TeslaLensOff,
                Part::TeslaLensOn,
                Part::TeslaBezelOff,
                Part::TeslaBezelOn,
            ],
        }
    }

    /// Its light's colour, and its filament's flare (hot, warm, out) and where in the lens it
    /// is (a share of the radius down from the middle).
    fn light(self) -> (&'static str, [&'static str; 3], f64) {
        match self {
            Lamp::Amber => ("#ff9628", ["#ffe2a0", "#ffaa3c", "#ff9628"], 0.05),
            Lamp::Tesla => ("#468cff", ["#d7e8ff", "#5a96ff", "#3c78ff"], TORUS.1),
        }
    }
}

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

/// Where everything is, `p` of the way come (`still`: the shutter off).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// The blades, 0 closed to 1 gone under the ring (1 with the shutter off).
    pub open: f64,
    /// What comes up: risen (0 waiting in the well, 1 standing in the ring; past 1 a little
    /// as it settles), a lamp's bezel turned home (0 turned back, 1 home), lit (0 dark, 1
    /// fully), and how much of it shows (with the shutter off, fading in; else 1).
    pub rise: f64,
    pub twist: f64,
    pub glow: f64,
    pub shown: f64,
}

pub fn look(p: f64, still: bool) -> Look {
    let p = p.clamp(0.0, 1.0);
    if still {
        return Look {
            open: 1.0,
            rise: 1.0,
            twist: 1.0,
            glow: warming(p),
            shown: smooth(0.0, FADE, p),
        };
    }
    let (l0, l1) = LIFT;
    Look {
        open: smooth(OPEN.0, OPEN.1, p),
        rise: if p < l0 { 0.0 } else { settle(l0, l1, p) },
        twist: smooth(l0, l1, p),
        glow: warming(smooth(l0 + LIGHT_LAG * (l1 - l0), l1, p)),
        shown: 1.0,
    }
}

/// A lamp's light, `glow` of the way lit, at the synth's `level`: from its idle up to full.
pub fn driven(glow: f64, level: f64) -> f64 {
    glow * (IDLE + (1.0 - IDLE) * level.clamp(0.0, 1.0))
}

fn origin() -> (f64, f64) {
    (COL + ULTRA_AT.0, TOP + ULTRA_AT.1)
}

/// Whether (`x`, `y`) in the drawing is on the opening (its ring and what is in it): a right
/// click there opens its menu (its shutter, or always open).
pub fn on_lamp(x: f64, y: f64) -> bool {
    let (ox, oy) = origin();
    (x - ox).hypot(y - oy) <= APERTURE + RING + 2.0
}

/// The layer's extent: the ring and the lamps' light round it.
const BOUNDS: [f64; 4] = [-88.0, -88.0, 88.0, 88.0];

/// What the opening shows: LO's hamster while he is up, else ULTRA's lamp while it is, else
/// HI's (nothing, at 0: the shutter closed, or with it off the empty well).
enum Shown {
    Hamster(f64),
    Lamp(Lamp, f64),
}

fn shown(o: &Opening) -> Shown {
    if o.hamster > 0.0 {
        Shown::Hamster(o.hamster)
    } else if o.coil > 0.0 {
        Shown::Lamp(Lamp::Tesla, o.coil)
    } else {
        Shown::Lamp(Lamp::Amber, o.lamp)
    }
}

/// QUALITY's opening in the worn skin: two layers, the back (the well, its floor, the shadows
/// in it and the ring round it), drawn again only as something comes or goes, and the front
/// (what is in it, the shutter over it, its light), drawn again as it moves.
pub fn ultra_worn(o: &Opening) -> [Layer; 2] {
    let mut back = Svg::default();
    let body = Svg::default();
    let mut over = Svg::default();
    let mut sprites = Vec::new();
    let p = match shown(o) {
        Shown::Hamster(p) | Shown::Lamp(_, p) => p,
    };
    let l = look(p, o.still);
    let mut floor = Vec::new();
    if l.open > 0.0 {
        well(&mut back);
        floor.push(picture(
            Part::UltraFloor,
            2.0 * APERTURE * FLOOR,
            1.0,
            0.0,
            1.0,
        ));
        match shown(o) {
            Shown::Hamster(_) => hamster(o, &l, &mut back, &mut over, &mut sprites),
            Shown::Lamp(lamp, _) => lamp_worn(lamp, o, p, &l, &mut back, &mut over, &mut sprites),
        }
    }
    let mut rim = Svg::default();
    ring(&mut rim);
    if l.open < 1.0 {
        shutter(&mut over, l.open);
    }
    if let Shown::Lamp(lamp, _) = shown(o) {
        let lit = driven(l.glow, o.level) * l.shown;
        if lit > 0.0 {
            halo(&mut over, lamp.light().0, lit);
        }
        let k = l.glow * l.shown * o.level;
        if k > 0.0 {
            spill(&mut over, lamp.light().0, k);
        }
    }
    [
        Layer {
            origin: origin(),
            bounds: BOUNDS,
            body: back.0,
            sprites: floor,
            over: rim.0,
            ..Layer::default()
        },
        Layer {
            origin: origin(),
            bounds: BOUNDS,
            body: body.0,
            sprites,
            over: over.0,
            ..Layer::default()
        },
    ]
}

/// A lamp come `p` of the way (as `l`), lit as the synth plays; the Tesla lamp's lightning on
/// it.
fn lamp_worn(
    lamp: Lamp,
    o: &Opening,
    p: f64,
    l: &Look,
    body: &mut Svg,
    over: &mut Svg,
    sprites: &mut Vec<Sprite>,
) {
    let sc = DEEP + (1.0 - DEEP) * l.rise;
    let glow = driven(l.glow, o.level);
    if l.rise > 0.0 {
        // Its shadow on the well's wall as it stands up out of it.
        put!(
            body,
            "<defs><radialGradient id='u-shade'><stop offset='0.75' stop-color='#000' stop-opacity='{}'/><stop offset='1' stop-color='#000' stop-opacity='0'/></radialGradient></defs><circle cx='{}' cy='{}' r='{}' fill='url(#u-shade)'/>",
            N(0.55 * l.rise.min(1.0) * l.shown),
            N(3.0 * l.rise),
            N(5.0 * l.rise),
            N(LAMP_R * sc * 1.12)
        );
    }
    let [lens_off, lens_on, bezel_off, bezel_on] = lamp.parts();
    let (d, deg) = (2.0 * LAMP_R, -TURN * (1.0 - l.twist));
    sprites.push(picture(lens_off, d, sc, 0.0, l.shown));
    if glow > 0.0 {
        sprites.push(picture(lens_on, d, sc, 0.0, glow * l.shown));
    }
    sprites.push(picture(bezel_off, d, sc, deg, l.shown));
    if glow > 0.0 {
        sprites.push(picture(bezel_on, d, sc, deg, glow * l.shown));
    }
    let k = l.glow * l.shown * o.level;
    if k > 0.0 {
        flare(over, lamp, sc, k);
    }
    // The lightning, once the coil stands lit above the ring.
    if lamp == Lamp::Tesla && (o.still || l.rise > 0.5) {
        let power = if o.still {
            smooth(0.3, 1.0, p)
        } else {
            smooth(LIFT.0 + 0.3 * (LIFT.1 - LIFT.0), LIFT.1, p)
        } * l.shown
            * o.level;
        if power > 0.0 {
            streaks(over, sc, o.spark, power);
        }
    }
}

/// The filament flaring with `k` (0 to 1) inside the lens: a hot core round it fading out.
fn flare(s: &mut Svg, lamp: Lamp, sc: f64, k: f64) {
    let (_, [hot, warm, out], at) = lamp.light();
    let r = LAMP_R * sc;
    put!(
        s,
        "<defs><radialGradient id='u-flare' gradientUnits='userSpaceOnUse' cx='0' cy='{}' r='{}'><stop offset='0' stop-color='{hot}' stop-opacity='{}'/><stop offset='0.4' stop-color='{warm}' stop-opacity='{}'/><stop offset='1' stop-color='{out}' stop-opacity='0'/></radialGradient></defs><circle r='{}' fill='url(#u-flare)'/>",
        N(at * r),
        N(0.55 * r),
        N(FLARE * k),
        N(0.5 * FLARE * k),
        N(r * LENS)
    );
}

/// A lamp's light spreading onto the panel round it, `k` (0 to 1) of its most, at the synth's
/// loudest (the owner: "just add a warm glow to it"): from the ring out, the lamp itself clear
/// of it (its lens lit by its own light).
fn spill(s: &mut Svg, colour: &str, k: f64) {
    let r = BOUNDS[2] - 2.0;
    let from = APERTURE + RING;
    put!(
        s,
        "<defs><radialGradient id='u-spill' gradientUnits='userSpaceOnUse' cx='0' cy='0' r='{}'><stop offset='{}' stop-color='{colour}' stop-opacity='0'/><stop offset='{}' stop-color='{colour}' stop-opacity='{}'/><stop offset='1' stop-color='{colour}' stop-opacity='0'/></radialGradient></defs><circle r='{}' fill='url(#u-spill)'/>",
        N(r),
        N((from - 0.5) / r),
        N(from / r),
        N(SPILL * k),
        N(r)
    );
}

/// LO's hamster, his wheel come `l` of the way: the wheel turning, the hamster on its rungs at
/// its foot, a little up the rising side as he runs, or curled up asleep.
fn hamster(o: &Opening, l: &Look, body: &mut Svg, over: &mut Svg, sprites: &mut Vec<Sprite>) {
    let rise = l.rise.min(1.0);
    let sc = WHEEL_DEEP + (1.0 - WHEEL_DEEP) * l.rise;
    // Its shadow on the well's floor as it comes forward.
    if rise > 0.0 {
        put!(
            body,
            "<defs><radialGradient id='u-wheel-shade' gradientUnits='userSpaceOnUse' cx='{}' cy='{}' r='{}'><stop offset='0.86' stop-color='#000' stop-opacity='0'/><stop offset='0.94' stop-color='#000' stop-opacity='{}'/><stop offset='1' stop-color='#000' stop-opacity='0'/></radialGradient></defs><circle cx='{}' cy='{}' r='{}' fill='url(#u-wheel-shade)'/>",
            N(3.0 * rise),
            N(5.0 * rise),
            N(WHEEL_R * sc * 1.08),
            N(0.5 * rise * l.shown),
            N(3.0 * rise),
            N(5.0 * rise),
            N(WHEEL_R * sc * 1.08)
        );
    }
    let r = WHEEL_R;
    sprites.push(picture(
        Part::HamsterWheel,
        2.0 * r / WHEEL_RIM,
        sc,
        o.turn,
        l.shown,
    ));
    let foot = FOOT * r;
    let (w, h) = (HAMSTER_LEN * r, HAMSTER_LEN * r * FRAME_PX.1 / FRAME_PX.0);
    let climb = (-CLIMB * o.run).to_radians();
    let (cx, cy) = (0.0, foot - h / 2.0);
    let at = (
        (cx * climb.cos() - cy * climb.sin()) * sc,
        (cx * climb.sin() + cy * climb.cos()) * sc,
    );
    let awake = (1.0 - o.asleep) * l.shown;
    if awake > 0.0 {
        sprites.push(Sprite {
            part: Part::Hamster(o.stride),
            size: (w, h),
            at,
            deg: -CLIMB * o.run,
            flip: (false, false),
            zoom: sc,
            alpha: awake,
        });
    }
    if o.asleep > 0.0 {
        let (w, h) = (ASLEEP_W * r, ASLEEP_W * r * ASLEEP_PX.1 / ASLEEP_PX.0);
        sprites.push(Sprite {
            part: Part::HamsterAsleep,
            size: (w, h),
            at: (0.0, (foot - h / 2.0) * sc),
            deg: 0.0,
            flip: (false, false),
            zoom: sc,
            alpha: o.asleep * l.shown,
        });
    }
    // Deep in the well: darker.
    if rise < 1.0 {
        put!(
            over,
            "<circle r='{}' fill='#000' fill-opacity='{}'/>",
            N(APERTURE),
            N(0.55 * (1.0 - rise))
        );
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

// ---- The Tesla lamp's lightning: drawn, never the same twice (the mock-up's `tesla.js`).

/// The coil's toroid in the lamp's picture: its middle and half axes, shares of its radius.
const TORUS: (f64, f64, f64, f64) = (-0.018, -0.112, 0.204, 0.089);
/// Where the streaks meet the glass (a share of the radius), how many at most at once, and how
/// long a streak's jagged shape holds before it is struck again along the same way, seconds.
const GLASS: f64 = 0.74;
const SLOTS: u32 = 6;
const JAG_S: f64 = 0.045;

/// A small seeded random source: the same seed, the same numbers (so a frame held still draws
/// the same streaks).
struct Seeded(u32);

impl Seeded {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let a = self.0;
        let mut t = (a ^ (a >> 15)).wrapping_mul(1 | a);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

/// A jagged path from `a` to `b` (appended to `out`): its middle pushed aside by a share of its
/// length, again and again; at some of its corners a short branch (appended to `branches`).
fn bolt(
    rnd: &mut Seeded,
    a: (f64, f64),
    b: (f64, f64),
    depth: u32,
    out: &mut Vec<(f64, f64)>,
    branches: Option<&mut Vec<Vec<(f64, f64)>>>,
) {
    const ROUGH: f64 = 0.3;
    if depth == 0 {
        out.push(b);
        return;
    }
    let len = (b.0 - a.0).hypot(b.1 - a.1).max(1e-9);
    let (nx, ny) = (-(b.1 - a.1) / len, (b.0 - a.0) / len);
    let off = (rnd.next() - 0.5) * ROUGH * len;
    let m = ((a.0 + b.0) / 2.0 + nx * off, (a.1 + b.1) / 2.0 + ny * off);
    let mut branches = branches;
    if let Some(list) = branches.as_deref_mut()
        && depth >= 3
        && rnd.next() < 0.35
    {
        let side = if rnd.next() < 0.5 { -1.0 } else { 1.0 };
        let ang = (b.1 - a.1).atan2(b.0 - a.0) + side * (0.4 + rnd.next() * 0.6);
        let bl = len * (0.35 + rnd.next() * 0.4);
        let mut branch = vec![m];
        bolt(
            rnd,
            m,
            (m.0 + ang.cos() * bl, m.1 + ang.sin() * bl),
            depth - 2,
            &mut branch,
            None,
        );
        list.push(branch);
    }
    bolt(rnd, a, m, depth - 1, out, branches.as_deref_mut());
    bolt(rnd, m, b, depth - 1, out, branches);
}

/// Where a ray from `s` (from the lamp's middle) at angle `d` meets the circle of radius `r`.
fn ray_to(s: (f64, f64), d: f64, r: f64) -> (f64, f64) {
    let (dx, dy) = (d.cos(), d.sin());
    let b = s.0 * dx + s.1 * dy;
    let c = s.0 * s.0 + s.1 * s.1 - r * r;
    let k = -b + (b * b - c).max(0.0).sqrt();
    (s.0 + dx * k, s.1 + dy * k)
}

fn path(points: &[(f64, f64)]) -> String {
    let mut d = String::with_capacity(points.len() * 14);
    for (i, &(x, y)) in points.iter().enumerate() {
        d.push_str(if i == 0 { "M" } else { " L" });
        d.push_str(&format!("{} {}", N(x), N(y)));
    }
    d
}

/// The streaks at the lightning's clock `t`, `power` of their full (fewer, shorter and fainter
/// as the synth plays softer), the lamp at `sc` of its size. Each of [`SLOTS`] lives its own
/// life, a tenth to a quarter of a second, leaving the toroid's rim (upwards and to the sides,
/// not down through the coil) for the glass a little off straight out, drifting as it lives,
/// its jagged shape struck again every [`JAG_S`]. Drawn as light: a wide faint blue glow, a
/// paler stroke, a white-blue core, a bright spot where it touches the glass; the toroid
/// brighter and the lamp's light round it flickering with them.
fn streaks(s: &mut Svg, sc: f64, t: f64, power: f64) {
    let r = LAMP_R * sc;
    let (cx, cy) = (TORUS.0 * r, TORUS.1 * r);
    let jag = (t / JAG_S).floor() as i64;
    let mut drawn = Vec::new();
    for k in 0..SLOTS {
        let life = 0.1 + 0.15 * Seeded(k.wrapping_mul(97).wrapping_add(5)).next();
        let at = t / life + f64::from(k) * 0.37;
        let e = at.floor();
        let into = at - e;
        let mut rnd = Seeded(
            (e as i64 as u32)
                .wrapping_mul(131)
                .wrapping_add(k.wrapping_mul(7919))
                .wrapping_add(3),
        );
        if rnd.next() > 0.2 + 0.6 * power {
            continue;
        }
        let a = (-200.0 + rnd.next() * 220.0).to_radians();
        let start = (cx + a.cos() * TORUS.2 * r, cy + a.sin() * TORUS.3 * r);
        let dir = a + (rnd.next() - 0.5) * 0.8 + (t * 2.3 + f64::from(k) * 1.7).sin() * 0.12;
        let end = ray_to(start, dir, GLASS * r);
        // (Softer: the streak reaches only part of the way.)
        let reach = 0.45 + 0.55 * power;
        let end = (
            start.0 + (end.0 - start.0) * reach,
            start.1 + (end.1 - start.1) * reach,
        );
        let mut j = Seeded(
            (jag as u32)
                .wrapping_mul(7)
                .wrapping_add(k.wrapping_mul(1013))
                .wrapping_add(1),
        );
        let mut points = vec![start];
        let mut branches = Vec::new();
        bolt(&mut j, start, end, 5, &mut points, Some(&mut branches));
        // (Inside the glass: a corner pushed past it brought back to it.)
        let inside = |q: &mut (f64, f64)| {
            let d = q.0.hypot(q.1);
            if d > GLASS * r {
                (q.0, q.1) = (q.0 * GLASS * r / d, q.1 * GLASS * r / d);
            }
        };
        points.iter_mut().for_each(inside);
        branches.iter_mut().flatten().for_each(inside);
        let env = if into < 0.12 {
            into / 0.12
        } else {
            1.0 - 0.55 * (into - 0.12) / 0.88
        };
        drawn.push((
            points,
            branches,
            end,
            env * (0.7 + 0.3 * j.next()) * (0.35 + 0.65 * power),
        ));
    }
    if drawn.is_empty() {
        return;
    }
    put!(
        s,
        "<defs><radialGradient id='t-spot'><stop offset='0' stop-color='#96c8ff'/><stop offset='1' stop-color='#3c78ff' stop-opacity='0'/></radialGradient></defs><g id='t-glass' fill='none' stroke-linecap='round' stroke-linejoin='round'>"
    );
    for (points, branches, end, v) in &drawn {
        let d = path(points);
        for (w, colour, a) in [
            (7.0, "#286eff", 0.08),
            (3.8, "#468cff", 0.2),
            (1.7, "#82b9ff", 0.55),
            (0.7, "#ebf4ff", 0.95),
        ] {
            put!(
                s,
                "<path d='{d}' stroke='{colour}' stroke-opacity='{}' stroke-width='{}'/>",
                N(a * v),
                N(w * sc)
            );
        }
        for b in branches {
            let d = path(b);
            put!(
                s,
                "<path d='{d}' stroke='#468cff' stroke-opacity='{}' stroke-width='{}'/><path d='{d}' stroke='#aad2ff' stroke-opacity='{}' stroke-width='{}'/>",
                N(0.12 * v),
                N(3.0 * sc),
                N(0.55 * v),
                N(0.9 * sc)
            );
        }
        put!(
            s,
            "<circle cx='{}' cy='{}' r='{}' fill='url(#t-spot)' fill-opacity='{}' stroke='none'/>",
            N(end.0),
            N(end.1),
            N(4.5 * sc),
            N(0.6 * v)
        );
    }
    put!(
        s,
        "<circle cx='{}' cy='{}' r='{}' fill='url(#t-spot)' fill-opacity='{}' stroke='none'/></g>",
        N(cx),
        N(cy),
        N(TORUS.2 * r * 1.3),
        N(0.12 * power * (drawn.len() as f64 / 3.0).min(1.0))
    );
    // The lamp's light round it flickering with them.
    let flash = drawn.iter().map(|d| d.3).sum::<f64>() / f64::from(SLOTS);
    spill(s, Lamp::Tesla.light().0, 0.5 * flash);
}

// ---- The opening.

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

/// A lamp's light on the panel round it, `glow` lit: very faint, just past the ring (the
/// owner: "No need for that much yellow glow around the lamp. Keep it very subtle").
fn halo(s: &mut Svg, colour: &str, glow: f64) {
    let r = 1.3 * LAMP_R;
    put!(
        s,
        "<defs><radialGradient id='u-halo' gradientUnits='userSpaceOnUse' cx='0' cy='0' r='{}'><stop offset='{}' stop-color='{colour}' stop-opacity='0'/><stop offset='{}' stop-color='{colour}' stop-opacity='{}'/><stop offset='1' stop-color='{colour}' stop-opacity='0'/></radialGradient></defs><circle r='{}' fill='url(#u-halo)'/>",
        N(r),
        N((APERTURE + RING - 1.0) / r),
        N((APERTURE + RING + 1.0) / r),
        N(0.06 * glow),
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

/// QUALITY's opening as drawn (the drawn skin): shut, nothing (the drawing is the approved
/// mock-up's, which has none); opening, the shutter in its ring; open, a lamp a chrome ring
/// round an amber or a blue lens, lit as it lights, or the hamster's wheel a brass ring.
pub fn ultra(o: &Opening) -> Layer {
    let p = match shown(o) {
        Shown::Hamster(p) | Shown::Lamp(_, p) => p,
    };
    let l = look(p, o.still);
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
        put!(s, "<circle r='{}' fill='#111'/>", N(APERTURE));
        match shown(o) {
            Shown::Hamster(_) => {
                let sc = WHEEL_DEEP + (1.0 - WHEEL_DEEP) * l.rise;
                put!(
                    s,
                    "<g opacity='{}'><circle r='{}' fill='none' stroke='#b08a3c' stroke-width='2'/><ellipse cy='{}' rx='{}' ry='{}' fill='#c98a3e'/></g>",
                    N(l.shown),
                    N(WHEEL_R * sc),
                    N((FOOT * WHEEL_R - 7.0) * sc),
                    N(HAMSTER_LEN * WHEEL_R * 0.4 * sc),
                    N(7.0 * sc)
                );
            }
            Shown::Lamp(lamp, _) => {
                let sc = DEEP + (1.0 - DEEP) * l.rise;
                let (dark, lit) = match lamp {
                    Lamp::Amber => ("#5a2a08", "#ffa030"),
                    Lamp::Tesla => ("#0c1a5a", "#3c7cff"),
                };
                put!(
                    s,
                    "<g opacity='{}'><circle r='{}' fill='#bdbdb8' stroke='#6a6a66'/><circle r='{}' fill='{dark}'/><circle r='{}' fill='{lit}' fill-opacity='{}'/></g>",
                    N(l.shown),
                    N(LAMP_R * sc),
                    N(LAMP_R * LENS * sc),
                    N(LAMP_R * LENS * sc),
                    N(driven(l.glow, o.level))
                );
            }
        }
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

    /// Closed, only the shutter; open and lit, a lamp at its size with its bezel home; with the
    /// shutter off, it always up, fading in, only its light changing; the light lags the turn.
    #[test]
    fn a_lamp_comes_up_turning_and_lighting() {
        let shut = look(0.0, false);
        assert_eq!((shut.open, shut.rise, shut.glow), (0.0, 0.0, 0.0));
        let up = look(1.0, false);
        assert_eq!((up.open, up.twist, up.shown), (1.0, 1.0, 1.0));
        assert!((up.rise - 1.0).abs() < 1e-12 && (up.glow - 1.0).abs() < 1e-12);
        let mid = look(0.6, false);
        assert!(mid.open == 1.0 && mid.rise > 0.0 && mid.twist > mid.glow);
        let still = look(0.0, true);
        assert_eq!(
            (still.open, still.rise, still.twist, still.glow, still.shown),
            (1.0, 1.0, 1.0, 0.0, 0.0)
        );
        assert_eq!(look(1.0, true).glow, 1.0);
        assert_eq!(look(FADE, true).shown, 1.0);
        // Its layer: the shutter alone at first; the lamp's pictures once it opens, the floor
        // under them.
        let [back, front] = ultra_worn(&Opening::default());
        assert!(back.sprites.is_empty() && front.sprites.is_empty());
        assert!(front.over.contains("u-aperture") && !back.over.is_empty());
        let [back, open] = ultra_worn(&Opening {
            lamp: 1.0,
            ..Opening::default()
        });
        assert_eq!(back.sprites[0].part, Part::UltraFloor);
        assert_eq!(open.sprites.len(), 4);
        assert!(open.sprites.iter().all(|s| (s.zoom - 1.0).abs() < 1e-12));
        assert_eq!(open.sprites[0].part, Part::UltraLensOff);
        // At rest, lit and still, the back stays as it was while the front's light follows.
        let quieter = ultra_worn(&Opening {
            lamp: 1.0,
            level: 0.4,
            ..Opening::default()
        });
        assert_eq!(quieter[0], back);
        assert_ne!(quieter[1], open);
    }

    /// The lamps follow the synth: dim at rest, full at its loudest, the filament flaring and
    /// the light spilling only as it plays.
    #[test]
    fn a_lamp_s_light_follows_the_synth() {
        assert_eq!(driven(1.0, 0.0), IDLE);
        assert_eq!(driven(1.0, 1.0), 1.0);
        assert_eq!(driven(0.0, 1.0), 0.0);
        let at = |level| {
            let [_, front] = ultra_worn(&Opening {
                lamp: 1.0,
                level,
                ..Opening::default()
            });
            front
        };
        let (rest, loud) = (at(0.0), at(1.0));
        assert!(!rest.over.contains("u-flare") && !rest.over.contains("u-spill"));
        assert!(loud.over.contains("u-flare") && loud.over.contains("u-spill"));
        let lit = |l: &Layer| {
            l.sprites
                .iter()
                .find(|s| s.part == Part::UltraLensOn)
                .unwrap()
                .alpha
        };
        assert!((lit(&rest) - IDLE).abs() < 1e-12 && (lit(&loud) - 1.0).abs() < 1e-12);
    }

    /// ULTRA's Tesla lamp: its own pictures; lightning only while the synth sounds, the same
    /// for the same clock, struck again as the clock moves on.
    #[test]
    fn the_tesla_lamp_sparks_as_the_synth_sounds() {
        let at = |level, spark| {
            let [_, front] = ultra_worn(&Opening {
                coil: 1.0,
                level,
                spark,
                ..Opening::default()
            });
            front
        };
        let quiet = at(0.0, 1.0);
        assert!(quiet.sprites.iter().any(|s| s.part == Part::TeslaLensOff));
        assert!(!quiet.over.contains("t-glass"));
        let loud = at(1.0, 1.0);
        assert!(loud.over.contains("t-glass"), "streaks while it sounds");
        assert_eq!(loud, at(1.0, 1.0));
        assert_ne!(loud.over, at(1.0, 1.0 + 2.0 * JAG_S).over);
        // Over many moments, usually several at once.
        let counts: Vec<usize> = (0..100)
            .map(|k| {
                at(1.0, f64::from(k) * 0.037)
                    .over
                    .matches("stroke-width")
                    .count()
                    / 4
            })
            .collect();
        let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
        assert!(mean > 2.0, "{mean}");
    }

    /// LO's hamster: his wheel and him, coming forward from the well; turning; asleep he is
    /// curled up; one thing in the opening at a time.
    #[test]
    fn the_hamster_comes_up_in_his_wheel() {
        let o = Opening {
            hamster: 1.0,
            run: 1.0,
            stride: 3,
            turn: 30.0,
            ..Opening::default()
        };
        let [_, up] = ultra_worn(&o);
        let parts: Vec<Part> = up.sprites.iter().map(|s| s.part).collect();
        assert_eq!(parts, [Part::HamsterWheel, Part::Hamster(3)]);
        assert_eq!(up.sprites[0].deg, 30.0);
        assert!(
            up.sprites[1].at.0 > 0.0,
            "a run takes him up the rising side"
        );
        // (Up and running, the front has no drawing to parse: only his pictures.)
        assert!(up.body.is_empty() && up.over.is_empty());
        let [_, asleep] = ultra_worn(&Opening {
            run: 0.0,
            asleep: 1.0,
            ..o
        });
        assert_eq!(asleep.sprites.last().unwrap().part, Part::HamsterAsleep);
        let [_, deep] = ultra_worn(&Opening { hamster: 0.5, ..o });
        assert!(deep.sprites[0].zoom < 1.0 && deep.over.contains("fill-opacity"));
        // The hamster is shown over a lamp still on its way down.
        assert!(matches!(
            shown(&Opening { lamp: 0.3, ..o }),
            Shown::Hamster(_)
        ));
    }

    /// What a frame of the opening costs to draw at the window's opening size and twice it
    /// (a Retina screen), coming up and then lit and sparking: printed (`-- --ignored
    /// --nocapture`).
    #[test]
    #[ignore = "prints timings"]
    fn ultra_timings() {
        use crate::render::{Renderer, Scene};
        use crate::skin::Skin;
        for scale in [1720.0 / crate::art::W, 3440.0 / crate::art::W] {
            for (name, set) in [
                ("the amber lamp", 0usize),
                ("the Tesla lamp", 1),
                ("the hamster", 2),
            ] {
                let mut r = Renderer::with_skin(Skin::Worn, scale, 1.0);
                let mut scene = Scene::default();
                r.render(&scene);
                let mut times = Vec::new();
                for k in 0..=240 {
                    let p = (f64::from(k) / 160.0).min(1.0);
                    let mut o = Opening {
                        level: 0.5 + 0.5 * (f64::from(k) * 0.3).sin(),
                        spark: f64::from(k) / 30.0,
                        run: 1.0,
                        stride: (k % 8) as u8,
                        turn: f64::from(k) * 4.0,
                        ..Opening::default()
                    };
                    match set {
                        0 => o.lamp = p,
                        1 => o.coil = p,
                        _ => o.hamster = p,
                    }
                    scene.opening = o;
                    let t = std::time::Instant::now();
                    r.render(&scene);
                    times.push(t.elapsed().as_secs_f64() * 1000.0);
                }
                let mean = |t: &[f64]| t.iter().sum::<f64>() / t.len() as f64;
                let max = times.iter().copied().fold(0.0, f64::max);
                println!(
                    "scale {scale:.3}, {name}: coming up {:.2} ms a frame on average, {max:.2} at most; up, {:.2}",
                    mean(&times[..161]),
                    mean(&times[161..])
                );
            }
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
