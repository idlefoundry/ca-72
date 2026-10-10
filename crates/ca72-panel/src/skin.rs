//! The panel's skins (decisions.md R-LOOK): DRAWN, the panel as `art` draws it, and WORN, the
//! same panel in pictures of the original's materials, gently used: walnut, the face's textured
//! black over aluminium with the drawing's print laid on it, Moog's modular knobs (a black
//! skirt with a white dot, broad flutes, a spun aluminium cap), the wedge pointer knobs, the
//! blue and orange rockers, the jewels and the jacks, turned and moved as the drawn parts are.
//! Every place, size and legend is the drawing's; only the surfaces are pictures. The pictures
//! were made with an image generator from words and cut out here (`assets/worn/README.md`).

use resvg::tiny_skia::{ColorU8, FilterQuality, Pixmap, PixmapPaint, Transform};

/// A skin of the panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Skin {
    /// The panel as drawn.
    #[default]
    Drawn,
    /// The panel in pictures of the original's materials.
    Worn,
}

pub(crate) static WALNUT: &[u8] = include_bytes!("../assets/worn/walnut.png");
pub(crate) static FACE: &[u8] = include_bytes!("../assets/worn/face.png");
static KNOB: &[u8] = include_bytes!("../assets/worn/knob.png");
static KNOB_BIG: &[u8] = include_bytes!("../assets/worn/knob-big.png");
static POINTER: &[u8] = include_bytes!("../assets/worn/pointer.png");
static ROCKER_BLUE: &[u8] = include_bytes!("../assets/worn/rocker-blue.png");
static ROCKER_ORANGE: &[u8] = include_bytes!("../assets/worn/rocker-orange.png");
static POWER: &[u8] = include_bytes!("../assets/worn/power.png");
static JEWEL: &[u8] = include_bytes!("../assets/worn/jewel.png");
static JACK: &[u8] = include_bytes!("../assets/worn/jack.png");
static SCREW: &[u8] = include_bytes!("../assets/worn/screw.png");
static TOGGLE_UP: &[u8] = include_bytes!("../assets/worn/toggle-up.png");
static TOGGLE_MID: &[u8] = include_bytes!("../assets/worn/toggle-mid.png");
static TOGGLE_DOWN: &[u8] = include_bytes!("../assets/worn/toggle-down.png");
static ULTRA_LENS_OFF: &[u8] = include_bytes!("../assets/worn/ultra-lens-off.png");
static ULTRA_LENS_ON: &[u8] = include_bytes!("../assets/worn/ultra-lens-on.png");
static ULTRA_BEZEL_OFF: &[u8] = include_bytes!("../assets/worn/ultra-bezel-off.png");
static ULTRA_BEZEL_ON: &[u8] = include_bytes!("../assets/worn/ultra-bezel-on.png");
static ULTRA_FLOOR: &[u8] = include_bytes!("../assets/worn/ultra-floor.png");
static TESLA_LENS_OFF: &[u8] = include_bytes!("../assets/worn/tesla-lens-off.png");
static TESLA_LENS_ON: &[u8] = include_bytes!("../assets/worn/tesla-lens-on.png");
static TESLA_BEZEL_OFF: &[u8] = include_bytes!("../assets/worn/tesla-bezel-off.png");
static TESLA_BEZEL_ON: &[u8] = include_bytes!("../assets/worn/tesla-bezel-on.png");
static HAMSTER_WHEEL: &[u8] = include_bytes!("../assets/worn/hamster-wheel.png");
static HAMSTER_RUN: &[u8] = include_bytes!("../assets/worn/hamster-run.png");
static HAMSTER_ASLEEP: &[u8] = include_bytes!("../assets/worn/hamster-asleep.png");

/// The hamster's run: its frames, side by side in its picture.
pub const STRIDE: usize = 8;
static PLATE: &[u8] = include_bytes!("../assets/worn/plate.png");

/// The face's picture is kept grey; its colour, a black a touch warm, is this over it.
const FACE_TINT: [f32; 3] = [1.03, 1.0, 0.94];

/// A part's picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    /// A knob and a big knob (OSC 2 and 3's FREQUENCY), the white dot at twelve o'clock.
    Knob,
    KnobBig,
    /// A selector's wedge pointer knob, the pointer at twelve o'clock.
    Pointer,
    /// A rocker, its right half raised.
    RockerBlue,
    RockerOrange,
    RockerIvory,
    /// The POWER rocker, its upper half raised.
    Power,
    /// The POWER lamp's red jewel in its chrome bezel, and OVERLOAD's darker one.
    Jewel,
    JewelDark,
    Jack,
    Screw,
    /// A knob's spun aluminium cap alone, cut from its picture, for drawing unturned over the
    /// turned knob: a spun disc's sheen lies where the lamp is whichever way it is turned.
    KnobCap,
    KnobBigCap,
    PointerCap,
    /// QUALITY's chrome toggle (decisions.md R-ULTRA), its lever up (ULTRA), out (HI, the
    /// slightest angle down) and down (LO), the three cut on one frame: the nut stays put.
    ToggleUp,
    ToggleMid,
    ToggleDown,
    /// HI's lamp (it was ULTRA's): its domed amber lens (the bulb behind it), dark and lit,
    /// which does not turn; its knurled chrome bezel, the lens cut out of it, dark and lit,
    /// which turns; and the floor of the well the opening's things wait in (the machine's
    /// inside, dim).
    UltraLensOff,
    UltraLensOn,
    UltraBezelOff,
    UltraBezelOn,
    UltraFloor,
    /// ULTRA's Tesla lamp: the same lamp in cobalt glass, a miniature Tesla coil in it where
    /// the bulb was, dark and lit, its lens and bezel cut as the amber one's.
    TeslaLensOff,
    TeslaLensOn,
    TeslaBezelOff,
    TeslaBezelOn,
    /// LO's hamster: his wheel (old brass wire, its back's spokes darker, set back), a frame of
    /// his run (0 to [`STRIDE`] less one), and him asleep, curled up.
    HamsterWheel,
    Hamster(u8),
    HamsterAsleep,
    /// The name plate, blank (its lettering is printed on it: `art::plate_worn`).
    Plate,
}

/// How much of a knob's picture its skirt fills (the picture squared about the cap's axis;
/// the skirt's outline a pixel or two off it) and how far out its cap reaches, as shares of
/// the picture's half width.
pub(crate) const KNOB_SKIRT: f64 = 0.978;
pub(crate) const KNOB_CAP: f64 = 0.578;
pub(crate) const BIG_SKIRT: f64 = 0.989;
pub(crate) const BIG_CAP: f64 = 0.653;
/// The pointer knob's body and its cap, as shares of its picture's half width (the pointer's
/// tip at the picture's edge).
pub(crate) const POINTER_BODY: f64 = 0.78;
pub(crate) const POINTER_CAP: f64 = 0.43;

/// The worn skin's pictures as made (premultiplied), and each part's at the sizes last asked
/// for.
pub struct Pictures {
    pub walnut: Pixmap,
    pub face: Pixmap,
    knob: Pixmap,
    knob_big: Pixmap,
    pointer: Pixmap,
    rocker_blue: Pixmap,
    rocker_orange: Pixmap,
    rocker_ivory: Pixmap,
    power: Pixmap,
    jewel: Pixmap,
    jewel_dark: Pixmap,
    jack: Pixmap,
    screw: Pixmap,
    caps: [Pixmap; 3],
    toggle: [Pixmap; 3],
    ultra: [Pixmap; 5],
    tesla: [Pixmap; 4],
    hamster: [Pixmap; STRIDE],
    wheel: Pixmap,
    asleep: Pixmap,
    plate: Pixmap,
    scaled: Vec<((Part, u32, u32), Pixmap)>,
}

impl std::fmt::Debug for Pictures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pictures")
            .field("scaled", &self.scaled.len())
            .finish()
    }
}

/// The most parts kept at their sizes (a scale's: the knobs, three pointer and jewel sizes,
/// six rockers', the jacks', the screws', QUALITY's toggle, and what QUALITY's opening holds:
/// two lamps, the hamster and his wheel).
const KEPT: usize = 72;

impl Pictures {
    /// The pictures decoded (once, as the worn skin is first shown); the face's grey coloured,
    /// the ivory rockers made from the blue ones and OVERLOAD's jewel from POWER's.
    pub fn load() -> Pictures {
        let png = |b: &[u8]| Pixmap::decode_png(b).expect("a built-in picture decodes");
        let face = recoloured(&png(FACE), |c| {
            [
                f32::from(c[0]) * FACE_TINT[0],
                f32::from(c[1]) * FACE_TINT[1],
                f32::from(c[2]) * FACE_TINT[2],
            ]
        });
        let rocker_blue = png(ROCKER_BLUE);
        // The ivory of the controller's switches: the blue's shading, the drawing's ivory.
        let rocker_ivory = recoloured(&rocker_blue, |c| {
            let l =
                (0.3 * f32::from(c[0]) + 0.59 * f32::from(c[1]) + 0.11 * f32::from(c[2])) / 255.0;
            let k = (l / 0.42).min(1.22);
            [224.0 * k, 215.0 * k, 190.0 * k]
        });
        let jewel = png(JEWEL);
        // OVERLOAD's lens: dark, a glass with the bulb behind it; the chrome as it is.
        let jewel_dark = recoloured(&jewel, |c| {
            let (r, g) = (f32::from(c[0]), f32::from(c[1]));
            if r > g * 1.5 && r > 40.0 {
                [r * 0.42, r * 0.3, r * 0.2]
            } else {
                [r, g, f32::from(c[2])]
            }
        });
        let (knob, knob_big, pointer) = (png(KNOB), png(KNOB_BIG), png(POINTER));
        let caps = [
            capped(&knob, KNOB_CAP),
            capped(&knob_big, BIG_CAP),
            capped(&pointer, POINTER_CAP),
        ];
        Pictures {
            walnut: png(WALNUT),
            face,
            knob,
            knob_big,
            pointer,
            rocker_blue,
            rocker_orange: png(ROCKER_ORANGE),
            rocker_ivory,
            power: png(POWER),
            jewel,
            jewel_dark,
            jack: png(JACK),
            screw: png(SCREW),
            caps,
            toggle: [png(TOGGLE_UP), png(TOGGLE_MID), png(TOGGLE_DOWN)],
            ultra: [
                png(ULTRA_LENS_OFF),
                png(ULTRA_LENS_ON),
                png(ULTRA_BEZEL_OFF),
                png(ULTRA_BEZEL_ON),
                png(ULTRA_FLOOR),
            ],
            tesla: [
                png(TESLA_LENS_OFF),
                png(TESLA_LENS_ON),
                png(TESLA_BEZEL_OFF),
                png(TESLA_BEZEL_ON),
            ],
            hamster: frames(&png(HAMSTER_RUN)),
            wheel: png(HAMSTER_WHEEL),
            asleep: png(HAMSTER_ASLEEP),
            plate: png(PLATE),
            scaled: Vec::new(),
        }
    }

    fn source(&self, part: Part) -> &Pixmap {
        match part {
            Part::Knob => &self.knob,
            Part::KnobBig => &self.knob_big,
            Part::Pointer => &self.pointer,
            Part::RockerBlue => &self.rocker_blue,
            Part::RockerOrange => &self.rocker_orange,
            Part::RockerIvory => &self.rocker_ivory,
            Part::Power => &self.power,
            Part::Jewel => &self.jewel,
            Part::JewelDark => &self.jewel_dark,
            Part::Jack => &self.jack,
            Part::Screw => &self.screw,
            Part::KnobCap => &self.caps[0],
            Part::KnobBigCap => &self.caps[1],
            Part::PointerCap => &self.caps[2],
            Part::ToggleUp => &self.toggle[0],
            Part::ToggleMid => &self.toggle[1],
            Part::ToggleDown => &self.toggle[2],
            Part::UltraLensOff => &self.ultra[0],
            Part::UltraLensOn => &self.ultra[1],
            Part::UltraBezelOff => &self.ultra[2],
            Part::UltraBezelOn => &self.ultra[3],
            Part::UltraFloor => &self.ultra[4],
            Part::TeslaLensOff => &self.tesla[0],
            Part::TeslaLensOn => &self.tesla[1],
            Part::TeslaBezelOff => &self.tesla[2],
            Part::TeslaBezelOn => &self.tesla[3],
            Part::HamsterWheel => &self.wheel,
            Part::Hamster(k) => &self.hamster[usize::from(k) % STRIDE],
            Part::HamsterAsleep => &self.asleep,
            Part::Plate => &self.plate,
        }
    }

    /// A part's picture `w` by `h` pixels, made once for each size (the last `KEPT` kept).
    pub fn part(&mut self, part: Part, w: u32, h: u32) -> &Pixmap {
        let key = (part, w.max(1), h.max(1));
        if let Some(i) = self.scaled.iter().position(|(k, _)| *k == key) {
            return &self.scaled[i].1;
        }
        let p = resample(self.source(part), key.1, key.2);
        if self.scaled.len() >= KEPT {
            self.scaled.remove(0);
        }
        self.scaled.push((key, p));
        &self.scaled[self.scaled.len() - 1].1
    }

    /// Clears the parts made for a scale.
    pub fn forget_scale(&mut self) {
        self.scaled.clear();
    }
}

/// A strip of [`STRIDE`] frames side by side, each as wide as the others, cut apart.
fn frames(strip: &Pixmap) -> [Pixmap; STRIDE] {
    let w = strip.width() / STRIDE as u32;
    std::array::from_fn(|k| {
        let mut f = Pixmap::new(w, strip.height()).expect("a frame of at least a pixel");
        f.draw_pixmap(
            -((k as u32 * w) as i32),
            0,
            strip.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        f
    })
}

/// `p` with each pixel's colour as `f` makes it from its own (0 to 255; clamped), its alpha
/// kept.
fn recoloured(p: &Pixmap, f: impl Fn([u8; 3]) -> [f32; 3]) -> Pixmap {
    let mut out = p.clone();
    for px in out.pixels_mut() {
        let c = px.demultiply();
        if c.alpha() == 0 {
            continue;
        }
        let v = f([c.red(), c.green(), c.blue()]).map(|v| v.round().clamp(0.0, 255.0) as u8);
        *px = ColorU8::from_rgba(v[0], v[1], v[2], c.alpha()).premultiply();
    }
    out
}

/// A knob's picture with all but its cap cleared: a disc `share` of its half width across about
/// its middle, its rim eased over a pixel and a half.
fn capped(p: &Pixmap, share: f64) -> Pixmap {
    let mut out = p.clone();
    let w = p.width();
    let (c, r) = ((f64::from(w) - 1.0) / 2.0, share * f64::from(w) / 2.0);
    for (i, px) in out.pixels_mut().iter_mut().enumerate() {
        let (x, y) = (f64::from(i as u32 % w) - c, f64::from(i as u32 / w) - c);
        let k = ((r - x.hypot(y)) / 1.5).clamp(0.0, 1.0);
        if k < 1.0 {
            let c = px.demultiply();
            let a = (f64::from(c.alpha()) * k).round() as u8;
            *px = ColorU8::from_rgba(c.red(), c.green(), c.blue(), a).premultiply();
        }
    }
    out
}

/// `src` resized to `w` by `h`: halved in steps while it is more than twice as large (each
/// step the mean of four pixels, so a large picture does not alias), then drawn to size (the
/// CA-74's `skin::resample`).
pub fn resample(src: &Pixmap, w: u32, h: u32) -> Pixmap {
    let mut cur = src.clone();
    while cur.width() >= 2 * w && cur.height() >= 2 * h && cur.width() > 1 && cur.height() > 1 {
        let (hw, hh) = (cur.width().div_ceil(2), cur.height().div_ceil(2));
        let mut half = Pixmap::new(hw, hh).expect("a picture of at least a pixel");
        half.draw_pixmap(
            0,
            0,
            cur.as_ref(),
            &PixmapPaint {
                quality: FilterQuality::Bilinear,
                ..PixmapPaint::default()
            },
            Transform::from_scale(
                hw as f32 / cur.width() as f32,
                hh as f32 / cur.height() as f32,
            ),
            None,
        );
        cur = half;
    }
    let mut out = Pixmap::new(w, h).expect("a picture of at least a pixel");
    out.draw_pixmap(
        0,
        0,
        cur.as_ref(),
        &PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        },
        Transform::from_scale(
            w as f32 / cur.width() as f32,
            h as f32 / cur.height() as f32,
        ),
        None,
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARTS: [Part; 24] = [
        Part::Knob,
        Part::KnobBig,
        Part::Pointer,
        Part::RockerBlue,
        Part::RockerOrange,
        Part::RockerIvory,
        Part::Power,
        Part::Jewel,
        Part::JewelDark,
        Part::Jack,
        Part::Screw,
        Part::ToggleUp,
        Part::ToggleMid,
        Part::ToggleDown,
        Part::UltraLensOff,
        Part::UltraLensOn,
        Part::UltraFloor,
        Part::TeslaLensOff,
        Part::TeslaLensOn,
        Part::HamsterWheel,
        Part::Hamster(0),
        Part::Hamster(5),
        Part::HamsterAsleep,
        Part::Plate,
    ];

    #[test]
    fn the_pictures_decode_and_the_parts_are_cut_out() {
        let mut p = Pictures::load();
        assert!(p.walnut.width() > 1000 && p.face.width() > 900);
        for part in PARTS {
            let src = p.source(part);
            let at = |x: u32, y: u32| src.pixel(x, y).map_or(0, |c| c.alpha());
            let (w, h) = (src.width(), src.height());
            assert_eq!(at(0, 0), 0, "{part:?}'s corner");
            assert_eq!(at(w / 2, h / 2), 255, "{part:?}'s middle");
            let small = p.part(part, 40, 30);
            assert_eq!((small.width(), small.height()), (40, 30));
        }
        // The lamps' bezels are rings: their middles, where the lenses show, clear.
        for part in [
            Part::UltraBezelOff,
            Part::UltraBezelOn,
            Part::TeslaBezelOff,
            Part::TeslaBezelOn,
        ] {
            let src = p.source(part);
            let (w, h) = (src.width(), src.height());
            assert_eq!(
                src.pixel(w / 2, h / 2).map(|c| c.alpha()),
                Some(0),
                "{part:?}"
            );
            assert_eq!(
                src.pixel(w / 2, h / 20).map(|c| c.alpha()),
                Some(255),
                "{part:?}"
            );
        }
        // The face is a black, a touch warm.
        let face = p.face.pixel(10, 10).expect("inside").demultiply();
        assert!(face.red() < 70 && face.red() >= face.blue(), "{face:?}");
    }

    /// A cap cut from a knob is its picture's middle, opaque there and clear beyond its rim.
    #[test]
    fn the_caps_are_cut_from_the_knobs() {
        let p = Pictures::load();
        for (part, cap, share) in [
            (Part::Knob, Part::KnobCap, KNOB_CAP),
            (Part::KnobBig, Part::KnobBigCap, BIG_CAP),
            (Part::Pointer, Part::PointerCap, POINTER_CAP),
        ] {
            let (src, cut) = (p.source(part), p.source(cap));
            let mid = src.width() / 2;
            assert_eq!(cut.pixel(mid, mid), src.pixel(mid, mid), "{cap:?}'s middle");
            let rim = (share * f64::from(mid)) as u32;
            assert_eq!(
                cut.pixel(mid + rim + 3, mid).map(|c| c.alpha()),
                Some(0),
                "{cap:?}"
            );
            assert!(
                cut.pixel(mid + rim - 3, mid)
                    .is_some_and(|c| c.alpha() > 240),
                "{cap:?}"
            );
        }
    }

    /// `a` times x is `y` (3 by 3), by Cramer's rule.
    fn solve3(a: [[f64; 3]; 3], y: [f64; 3]) -> [f64; 3] {
        let det = |m: [[f64; 3]; 3]| {
            m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
        };
        let d = det(a);
        std::array::from_fn(|k| {
            let mut m = a;
            for (i, row) in m.iter_mut().enumerate() {
                row[k] = y[i];
            }
            det(m) / d
        })
    }

    /// A sRGB level's light (0 to 1, linear) and back.
    fn linear(v: u8) -> f64 {
        let v = f64::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    fn level(v: f64) -> f64 {
        let v = v.clamp(0.0, 1.0);
        255.0
            * if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            }
    }

    /// The knobs' pictures are lit evenly all round, so that a knob turned does not turn its
    /// light (`assets/worn/even.py`; the lamp is the panel's, over them): from the axis out to
    /// the skirt's outline, in rings of a 32nd of the picture, one side is at most 5 levels
    /// brighter than the other (the light's first harmonic round the ring), but where the
    /// white dot or the pointer's cream line lies, at the cap's rim and its very centre. As
    /// generated, the grips leaned 18 to 30 levels towards twelve o'clock.
    #[test]
    fn the_knobs_pictures_are_lit_evenly_all_round() {
        let p = Pictures::load();
        // What each picture leaves out (from and to, as shares of its half width, and degrees
        // either side of twelve o'clock): the machining's centre (a pixel or two on the
        // panel), the cap's rim, the dot or the pointer, and the skirt's outline (a few pixels
        // off the axis).
        // (From, to, degrees either side.)
        type Skip = (f64, f64, f64);
        let skips: [(Part, &[Skip]); 3] = [
            (
                Part::Knob,
                &[
                    (0.0, 0.07, 180.0),
                    (0.53, 0.63, 180.0),
                    (0.76, 0.90, 180.0),
                    (0.91, 2.0, 180.0),
                ],
            ),
            (
                Part::KnobBig,
                &[
                    (0.0, 0.07, 180.0),
                    (0.60, 0.70, 180.0),
                    (0.82, 0.93, 180.0),
                    (0.93, 2.0, 180.0),
                ],
            ),
            // (The wedge is broad where it meets the body.)
            (
                Part::Pointer,
                &[
                    (0.0, 0.07, 180.0),
                    (0.38, 0.48, 180.0),
                    (0.33, 0.62, 40.0),
                    (0.62, 2.0, 22.0),
                    (0.74, 2.0, 180.0),
                ],
            ),
        ];
        for (part, skip) in skips {
            let src = p.source(part);
            let c = f64::from(src.width() - 1) / 2.0;
            let ring = c / 16.0;
            // Each ring's sums for a least-squares fit of its light to a mean and a lean (its
            // first harmonic round the ring), which a ring with a part left out does not bias.
            let mut rings = [([[0.0f64; 3]; 3], [0.0f64; 3]); 32];
            for y in 0..src.height() {
                for x in 0..src.width() {
                    let (dx, dy) = (f64::from(x) - c, c - f64::from(y));
                    let r = dx.hypot(dy);
                    let (share, from_twelve) = (r / c, dx.atan2(dy).to_degrees().abs());
                    if skip
                        .iter()
                        .any(|&(a, b, half)| (a..b).contains(&share) && from_twelve <= half)
                    {
                        continue;
                    }
                    let px = src.pixel(x, y).expect("inside").demultiply();
                    if px.alpha() < 250 {
                        continue;
                    }
                    let light = 0.2126 * linear(px.red())
                        + 0.7152 * linear(px.green())
                        + 0.0722 * linear(px.blue());
                    let t = dy.atan2(dx);
                    let v = [1.0, t.cos(), t.sin()];
                    let (ata, aty) = &mut rings[(r / ring) as usize];
                    for p in 0..3 {
                        aty[p] += v[p] * light;
                        for q in 0..3 {
                            ata[p][q] += v[p] * v[q];
                        }
                    }
                }
            }
            for (i, (ata, aty)) in rings.into_iter().enumerate() {
                if ata[0][0] < 200.0 {
                    continue;
                }
                let [mean, along_x, along_y] = solve3(ata, aty);
                let lean = along_x.hypot(along_y);
                let apart = level(mean + lean) - level((mean - lean).max(0.0));
                assert!(
                    apart <= 5.0,
                    "{part:?}: ring {i} (of 16 to the edge), one side {apart:.1} levels brighter"
                );
            }
        }
    }
}
