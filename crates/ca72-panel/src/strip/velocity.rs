//! The left hand's VELOCITY (decisions.md R-VELOCITY): a jack under GLIDE and DECAY, the
//! velocity source's, from which a cord is dragged onto the knobs as from an LFO's OUT, and a
//! meter of ten lamps beside it lit to the latest note's velocity. Shown with the wheels, not
//! with the macros (which are laid over the left hand).

use crate::art::{self, Layer};
use crate::svg::{N, Svg};

/// The velocity's colour, its cords', lamps' and rings': the panel's lamps' orange, as
/// the presets' bar's dots and the strip's screens and readouts light (the owner: "a native
/// feeling element to the UI"; decisions.md R-VELOCITY). In a look, its lamps' (`colour`).
pub const RGB: [u8; 3] = [0xFF, 0x70, 0x28];
/// The jack's radius (the left hand's, GLIDE's and DECAY's). No collar: a jack as an LFO's OUT
/// is (the owner, 2026-10-27, of a ring round it always: "not our default behaviour").
pub const JACK_R: f64 = 22.0;
/// The meter's lamps: how many, and each one's place from the jack's centre.
pub const LAMPS: usize = 10;
const LAMP_LEFT: f64 = 46.0;
const LAMP_PITCH: f64 = 21.0;
const LAMP_W: f64 = 17.0;
const LAMP_H: f64 = 12.0;
const LAMP_TOP: f64 = 5.0;

/// The jack's centre in the drawing.
pub fn jack_at() -> (f64, f64) {
    let (lx, ly) = crate::strip::LEFT_HAND;
    (lx + art::LH_JACK, ly + art::LH_VELOCITY)
}

/// Whether (x, y) in the drawing is on the jack (a little beyond its nut, as an LFO's OUT).
pub fn hit_jack(x: f64, y: f64) -> bool {
    let (jx, jy) = jack_at();
    (x - jx).hypot(y - jy) <= JACK_R + 10.0
}

/// The lamps lit for velocity `v` (0..1): one a tenth, the first at any note above 0; none
/// before a note.
pub fn lit(v: Option<f64>) -> usize {
    v.filter(|v| v.is_finite())
        .map_or(0, |v| (v.clamp(0.0, 1.0) * LAMPS as f64).ceil() as usize)
}

fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// The velocity's colour in a look: its pack's `velocity` colour where it has one
/// (`look::Colours::velocity`), else its `lamp` (the screens' and readouts'), else the built-in
/// `RGB`.
pub fn colour(colours: &crate::look::Colours) -> [u8; 3] {
    colours
        .velocity
        .or(colours.lamp)
        .map_or(RGB, |(r, g, b)| [r, g, b])
}

/// The meter, `lit` lamps of it lit in `rgb` (the look's `colour`): a layer of its own, drawn
/// again only as the lamps change.
pub fn meter(lit: usize, rgb: [u8; 3]) -> Layer {
    let colour = hex(rgb);
    let mut s = Svg::default();
    s.0.push_str(&format!(
        "<filter id='vlit' x='-1' y='-1' width='3' height='3'><feDropShadow dx='0' dy='0' \
         stdDeviation='2.5' flood-color='{colour}'/></filter>"
    ));
    for i in 0..LAMPS {
        let x = LAMP_LEFT + LAMP_PITCH * i as f64;
        s.0.push_str(&format!(
            "<rect x='{}' y='{}' width='{}' height='{}' rx='2' fill='#140c08' stroke='#000' \
             stroke-width='1.5'/>",
            N(x),
            N(LAMP_TOP),
            N(LAMP_W),
            N(LAMP_H)
        ));
        let on = i < lit;
        s.0.push_str(&format!(
            "<rect x='{}' y='{}' width='{}' height='{}' rx='1.5' fill='{colour}' \
             fill-opacity='{}'{}/>",
            N(x + 1.5),
            N(LAMP_TOP + 1.5),
            N(LAMP_W - 3.0),
            N(LAMP_H - 3.0),
            if on { "1" } else { "0.12" },
            if on { " filter='url(#vlit)'" } else { "" }
        ));
    }
    Layer {
        origin: jack_at(),
        bounds: [
            LAMP_LEFT - 8.0,
            LAMP_TOP - 8.0,
            LAMP_LEFT + LAMP_PITCH * LAMPS as f64 + 8.0,
            LAMP_TOP + LAMP_H + 8.0,
        ],
        body: s.0,
        ..Layer::default()
    }
}

/// The meter's legend's centre, from the jack's: above the lamps, centred on them.
pub fn legend_at() -> (f64, f64) {
    (
        art::LH_JACK + LAMP_LEFT + (LAMP_PITCH * (LAMPS - 1) as f64 + LAMP_W) / 2.0,
        art::LH_VELOCITY - 11.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lamps_light_by_tenths() {
        assert_eq!(lit(None), 0);
        assert_eq!(lit(Some(0.0)), 0);
        assert_eq!(lit(Some(0.01)), 1);
        assert_eq!(lit(Some(0.5)), 5);
        assert_eq!(lit(Some(1.0)), LAMPS);
        assert_eq!(lit(Some(f64::NAN)), 0);
    }

    #[test]
    fn a_looks_colour_dresses_the_lamps() {
        use crate::look::Colours;
        // Without a pack's colour, the built-in one, drawn as before.
        assert_eq!(colour(&Colours::default()), RGB);
        let built_in = meter(6, RGB).body;
        assert!(built_in.contains("#ff7028"));
        // With one, every part in it and none in the built-in one; the wells as they were.
        let c = colour(&Colours {
            velocity: Some((0x40, 0xc0, 0xff)),
            ..Colours::default()
        });
        assert_eq!(c, [0x40, 0xc0, 0xff]);
        let dressed = meter(6, c).body;
        assert!(!dressed.contains("#ff7028"));
        assert_eq!(dressed.replace("#40c0ff", "#ff7028"), built_in);
        // (A look's lamps' colour, without a `velocity` of its own.)
        let lamps = Colours {
            lamp: Some((0x40, 0xc0, 0xff)),
            ..Colours::default()
        };
        assert_eq!(colour(&lamps), [0x40, 0xc0, 0xff]);
    }

    #[test]
    fn the_jack_is_found_where_it_is_drawn() {
        let (x, y) = jack_at();
        assert!(hit_jack(x, y));
        assert!(hit_jack(x + JACK_R, y));
        assert!(!hit_jack(x + JACK_R + 20.0, y));
    }

    #[test]
    fn the_row_fits_between_decay_and_the_wheels() {
        // (The jacks' nuts.)
        const NUT_R: f64 = JACK_R + 6.0;
        // (DECAY's jack and nut above; the wheels' slots below.)
        let (_, y) = jack_at();
        let ly = crate::strip::LEFT_HAND.1;
        assert!(y - NUT_R > ly + art::LH_ROWS[1] + NUT_R);
        let slot_top = ly + art::WHEEL_Y - 73.0 * art::WHEEL_SCALE;
        assert!(y + NUT_R < slot_top, "{} {}", y + NUT_R, slot_top);
        // (The lamps within the left hand's column.)
        let right = crate::strip::LEFT_HAND.0 + art::LH_JACK + LAMP_LEFT;
        assert!(right + LAMP_PITCH * LAMPS as f64 <= crate::strip::SECTIONS[0].1);
    }
}
