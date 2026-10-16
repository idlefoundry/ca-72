//! The panel's layout as data (decisions.md R-SKIN-PACKS): where each control, legend, rule,
//! title and fixture is, read from a skin pack's `layout.toml`. The official panel is one
//! (`packs/official/layout.toml`, built in); a pack may bring its own.
//!
//! A layout says where things are and how they look, never what they do: each control's
//! parameter, its kind (knob, selector, switch, toggle), its dial's or its legends' values and
//! how it is operated are the plug-in's (`controls::CONTROLS`). A layout may move a control,
//! make a knob large or small, size a switch or a toggle, colour a rocker and choose its "on"
//! end, and set every printed legend, rule and title. The left hand's controls and the strip's
//! are the plug-in's own and stay where they are.

use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use skin_pack::Report;

use crate::art::{self, PH, PW, TOP};
use crate::controls::{CONTROLS, Colour, Control, Kind, Orient, Place};

/// The layout data's format.
pub const FORMAT: u32 = 1;

/// The official layout's text.
pub const OFFICIAL: &str = include_str!("../packs/official/layout.toml");

/// What a layout may hold.
const MOST_TEXTS: usize = 300;
const MOST_LINES: usize = 300;
const MOST_TITLES: usize = 16;
const MOST_SCREWS: usize = 32;
const MOST_CHARS: usize = 48;

/// A legend: `at` its centre (the middle of its capitals), in the face's units; `size` its
/// lettering's (the panel's legend size, by the lettering, if `None`); `large` its place in
/// the large lettering.
#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub at: (f64, f64),
    pub large: Option<(f64, f64)>,
    pub text: String,
    pub size: Option<f64>,
}

impl Text {
    /// Its place in `lettering`.
    pub fn at_in(&self, lettering: art::Lettering) -> (f64, f64) {
        match (lettering, self.large) {
            (art::Lettering::Large, Some(p)) => p,
            _ => self.at,
        }
    }
}

/// A rule: from, to, width, in the face's units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub width: f64,
}

/// A section title along the foot: spaced out to its printed extent `span` (left and right),
/// on its `baseline`.
#[derive(Clone, Debug, PartialEq)]
pub struct Title {
    pub text: String,
    pub span: (f64, f64),
    pub baseline: f64,
    pub size: f64,
}

/// The panel's layout.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrangement {
    /// Every control, in [`CONTROLS`]'s order (the left hand's and the strip's as they are).
    pub controls: Vec<Control>,
    pub lines: Vec<Line>,
    pub titles: Vec<Title>,
    pub texts: Vec<Text>,
    /// POWER's switch, its lamp, OVERLOAD's lamp and QUALITY's opening: centres, in the face's
    /// units.
    pub power: (f64, f64),
    pub lamp: (f64, f64),
    pub overload: (f64, f64),
    pub ultra: (f64, f64),
    /// The name plate's top left corner, in the face's units.
    pub plate: (f64, f64),
    /// The PHONES jack: centre and radius.
    pub phones: Option<(f64, f64, f64)>,
    /// The wooden strip's screws: centre and radius, in the face's units.
    pub screws: Vec<(f64, f64, f64)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    format: u32,
    fixtures: Fixtures,
    #[serde(default)]
    control: Vec<ControlEntry>,
    #[serde(default)]
    line: Vec<LineEntry>,
    #[serde(default)]
    title: Vec<TitleEntry>,
    #[serde(default)]
    text: Vec<TextEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixtures {
    power: [f64; 2],
    lamp: [f64; 2],
    overload: [f64; 2],
    ultra: [f64; 2],
    plate: [f64; 2],
    #[serde(default)]
    phones: Option<[f64; 3]>,
    #[serde(default)]
    screws: Vec<[f64; 3]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlEntry {
    param: String,
    at: [f64; 2],
    #[serde(default)]
    big: Option<bool>,
    #[serde(default)]
    size: Option<[f64; 2]>,
    #[serde(default)]
    colour: Option<String>,
    #[serde(default)]
    on: Option<String>,
    #[serde(default)]
    across: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LineEntry {
    from: [f64; 2],
    to: [f64; 2],
    width: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TitleEntry {
    text: String,
    span: [f64; 2],
    baseline: f64,
    #[serde(default)]
    size: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextEntry {
    text: String,
    at: [f64; 2],
    #[serde(default)]
    size: Option<f64>,
    #[serde(default)]
    large: Option<[f64; 2]>,
}

/// The section titles' size (their capitals 38.5 units tall).
pub const TITLE_SIZE: f64 = 52.0;

/// The canvas a layout draws on, in the face's units: the face and the wooden strip above it.
const CANVAS: [f64; 4] = [0.0, -TOP, PW, PH];

fn inside(r: [f64; 4]) -> bool {
    r[0] >= CANVAS[0] && r[1] >= CANVAS[1] && r[2] <= CANVAS[2] && r[3] <= CANVAS[3]
}

fn overlap(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

fn finite(p: [f64; 2]) -> bool {
    p.iter().all(|v| v.is_finite())
}

/// The extent a control may be clicked in, about its centre (`interact::hit`'s).
pub fn hit_bounds(kind: &Kind) -> [f64; 4] {
    match *kind {
        Kind::Knob { big, .. } => {
            let r = if big { art::BIG.skirt } else { art::STD.skirt } + 8.0;
            [-r, -r, r, r]
        }
        Kind::Selector(_) => [-60.0, -60.0, 60.0, 60.0],
        _ => art::bounds(kind),
    }
}

impl Arrangement {
    /// The official panel's layout (read once).
    pub fn official() -> Arc<Arrangement> {
        static OFFICIAL_ONE: OnceLock<Arc<Arrangement>> = OnceLock::new();
        OFFICIAL_ONE
            .get_or_init(|| {
                let mut report = Report::default();
                Arc::new(
                    Arrangement::read(OFFICIAL, &mut report)
                        .expect("the official layout is read (its tests say so)"),
                )
            })
            .clone()
    }

    /// A layout read from its text, and checked: `None` if rejected (reported, `path` the
    /// file's in its pack).
    pub fn read(text: &str, report: &mut Report) -> Option<Arrangement> {
        Self::read_at(text, "layout.toml", report)
    }

    pub fn read_at(text: &str, path: &str, report: &mut Report) -> Option<Arrangement> {
        let at = Some(path);
        let file: File = match toml::from_str(text) {
            Ok(f) => f,
            Err(e) => {
                report.reject("layout.syntax", at, e.to_string());
                return None;
            }
        };
        if file.format != FORMAT {
            report.reject(
                "layout.format",
                at,
                format!("layout format {}: this plug-in reads {FORMAT}", file.format),
            );
            return None;
        }
        let before = report.findings.len();
        let mut reject = |code, msg: String| report.reject(code, at, msg);

        // The controls: every one of the panel's, once each, its kind's look only.
        let mut controls: Vec<Control> = CONTROLS.to_vec();
        let mut placed = vec![false; CONTROLS.len()];
        for e in &file.control {
            let Some(i) = CONTROLS.iter().position(|c| c.param == e.param) else {
                reject("layout.control", format!("`{}`: no such control", e.param));
                continue;
            };
            let official = CONTROLS[i];
            if official.place != Place::Panel {
                reject(
                    "layout.control",
                    format!(
                        "`{}` is the plug-in's own, on the strip: not placed by a layout",
                        e.param
                    ),
                );
                continue;
            }
            if placed[i] {
                reject("layout.control", format!("`{}` placed twice", e.param));
                continue;
            }
            placed[i] = true;
            if !finite(e.at) {
                reject(
                    "layout.number",
                    format!("`{}`'s place is not a number", e.param),
                );
                continue;
            }
            let kind = match look_of(official.kind, e) {
                Ok(k) => k,
                Err(why) => {
                    reject("layout.control", format!("`{}`: {why}", e.param));
                    continue;
                }
            };
            controls[i] = Control {
                x: e.at[0],
                y: e.at[1],
                kind,
                ..official
            };
        }
        for (i, c) in CONTROLS.iter().enumerate() {
            if c.place == Place::Panel && !placed[i] {
                reject(
                    "layout.control",
                    format!(
                        "`{}` is not placed: a layout places every control of the panel",
                        c.param
                    ),
                );
            }
        }

        // Each control's extent on the canvas, and no two where one click would find both.
        let panel: Vec<(usize, [f64; 4])> = controls
            .iter()
            .enumerate()
            .filter(|(i, c)| c.place == Place::Panel && placed[*i])
            .map(|(i, c)| {
                let b = hit_bounds(&c.kind);
                (i, [c.x + b[0], c.y + b[1], c.x + b[2], c.y + b[3]])
            })
            .collect();
        let pw = art::POWER_BOUNDS;
        let power = file.fixtures.power;
        let power_box = [
            power[0] + pw[0],
            power[1] + pw[1],
            power[0] + pw[2],
            power[1] + pw[3],
        ];
        for (k, (i, b)) in panel.iter().enumerate() {
            if !inside(*b) {
                reject(
                    "layout.outside",
                    format!("`{}` reaches outside the panel", CONTROLS[*i].param),
                );
            }
            for (j, other) in &panel[k + 1..] {
                if overlap(*b, *other) {
                    reject(
                        "layout.overlap",
                        format!(
                            "`{}` and `{}` overlap: a click on one would find the other",
                            CONTROLS[*i].param, CONTROLS[*j].param
                        ),
                    );
                }
            }
            if overlap(*b, power_box) {
                reject(
                    "layout.overlap",
                    format!("`{}` overlaps POWER", CONTROLS[*i].param),
                );
            }
        }

        // The fixtures.
        let fx = &file.fixtures;
        let fixtures: [(&str, [f64; 2], f64); 4] = [
            ("power", fx.power, 64.0),
            ("lamp", fx.lamp, 24.0),
            ("overload", fx.overload, 34.0),
            (
                "ultra",
                fx.ultra,
                crate::ultra::APERTURE + crate::ultra::RING,
            ),
        ];
        for (name, p, r) in fixtures {
            if !finite(p) || !inside([p[0] - r, p[1] - r, p[0] + r, p[1] + r]) {
                reject(
                    "layout.outside",
                    format!("`{name}` reaches outside the panel"),
                );
            }
        }
        let plate = fx.plate;
        if !finite(plate)
            || !inside([
                plate[0],
                plate[1],
                plate[0] + art::PLATE_W,
                plate[1] + art::PLATE_H,
            ])
        {
            reject(
                "layout.outside",
                "the name plate reaches outside the panel".into(),
            );
        }
        let round = |v: &[f64; 3]| {
            v.iter().all(|x| x.is_finite())
                && v[2] > 0.0
                && v[2] <= 60.0
                && inside([v[0] - v[2], v[1] - v[2], v[0] + v[2], v[1] + v[2]])
        };
        if let Some(p) = &fx.phones
            && !round(p)
        {
            reject("layout.outside", "PHONES reaches outside the panel".into());
        }
        if fx.screws.len() > MOST_SCREWS {
            reject("layout.count", format!("more than {MOST_SCREWS} screws"));
        }
        if !fx.screws.iter().all(round) {
            reject("layout.outside", "a screw reaches outside the panel".into());
        }

        // The print.
        if file.line.len() > MOST_LINES
            || file.text.len() > MOST_TEXTS
            || file.title.len() > MOST_TITLES
        {
            reject(
                "layout.count",
                format!(
                    "at most {MOST_LINES} rules, {MOST_TEXTS} legends and {MOST_TITLES} titles"
                ),
            );
        }
        for l in &file.line {
            let ok = finite(l.from)
                && finite(l.to)
                && (0.5..=20.0).contains(&l.width)
                && inside([
                    l.from[0].min(l.to[0]),
                    l.from[1].min(l.to[1]),
                    l.from[0].max(l.to[0]),
                    l.from[1].max(l.to[1]),
                ]);
            if !ok {
                reject(
                    "layout.line",
                    "a rule outside the panel, or its width not 0.5 to 20".into(),
                );
            }
        }
        let plain = |t: &str| {
            !t.trim().is_empty()
                && t.chars().count() <= MOST_CHARS
                && !t.chars().any(char::is_control)
        };
        for t in &file.text {
            let at_ok = finite(t.at) && t.large.is_none_or(finite);
            let size_ok = t.size.is_none_or(|s| (6.0..=80.0).contains(&s));
            if !plain(&t.text) || !at_ok || !size_ok {
                reject(
                    "layout.text",
                    format!(
                        "legend `{}`: 1 to {MOST_CHARS} characters, its size 6 to 80",
                        t.text.chars().take(MOST_CHARS).collect::<String>()
                    ),
                );
            }
        }
        for t in &file.title {
            let ok = plain(&t.text)
                && t.span[0].is_finite()
                && t.span[1].is_finite()
                && t.span[0] < t.span[1]
                && t.baseline.is_finite()
                && t.size.is_none_or(|s| (6.0..=80.0).contains(&s))
                && inside([t.span[0], t.baseline - 60.0, t.span[1], t.baseline]);
            if !ok {
                reject(
                    "layout.title",
                    format!("title `{}`: its extent, baseline or size", t.text),
                );
            }
        }
        if report.findings[before..]
            .iter()
            .any(|f| f.severity == skin_pack::Severity::Reject)
        {
            return None;
        }
        let pair = |p: [f64; 2]| (p[0], p[1]);
        Some(Arrangement {
            controls,
            lines: file
                .line
                .iter()
                .map(|l| Line {
                    from: pair(l.from),
                    to: pair(l.to),
                    width: l.width,
                })
                .collect(),
            titles: file
                .title
                .into_iter()
                .map(|t| Title {
                    text: t.text,
                    span: pair(t.span),
                    baseline: t.baseline,
                    size: t.size.unwrap_or(TITLE_SIZE),
                })
                .collect(),
            texts: file
                .text
                .into_iter()
                .map(|t| Text {
                    at: pair(t.at),
                    large: t.large.map(pair),
                    text: t.text,
                    size: t.size,
                })
                .collect(),
            power: pair(fx.power),
            lamp: pair(fx.lamp),
            overload: pair(fx.overload),
            ultra: pair(fx.ultra),
            plate: pair(fx.plate),
            phones: fx.phones.map(|p| (p[0], p[1], p[2])),
            screws: fx.screws.iter().map(|s| (s[0], s[1], s[2])).collect(),
        })
    }

    /// The control at `i` in [`CONTROLS`]'s order, where this layout has it.
    pub fn control(&self, i: usize) -> &Control {
        &self.controls[i]
    }

    /// POWER's centre in the drawing.
    pub fn power_at(&self) -> (f64, f64) {
        (art::COL + self.power.0, TOP + self.power.1)
    }

    /// The name plate's top left corner in the drawing.
    pub fn plate_at(&self) -> (f64, f64) {
        (art::COL + self.plate.0, TOP + self.plate.1)
    }

    /// QUALITY's opening's centre in the drawing.
    pub fn ultra_at(&self) -> (f64, f64) {
        (art::COL + self.ultra.0, TOP + self.ultra.1)
    }
}

/// A control's kind as a layout's entry makes it look: its official kind, its look changed
/// where the entry says, nothing else.
fn look_of(official: Kind, e: &ControlEntry) -> Result<Kind, String> {
    let size = |w: f64, h: f64| -> Result<(f64, f64), String> {
        match e.size {
            None => Ok((w, h)),
            Some([w, h]) if (16.0..=240.0).contains(&w) && (16.0..=240.0).contains(&h) => {
                Ok((w, h))
            }
            Some(_) => Err("a size of 16 to 240 each way".into()),
        }
    };
    let none = |field: &str, given: bool| -> Result<(), String> {
        if given {
            Err(format!("`{field}` does not apply to it"))
        } else {
            Ok(())
        }
    };
    Ok(match official {
        Kind::Knob { dial, big } => {
            none("size", e.size.is_some())?;
            none("colour", e.colour.is_some())?;
            none("on", e.on.is_some())?;
            none("across", e.across.is_some())?;
            Kind::Knob {
                dial,
                big: e.big.unwrap_or(big),
            }
        }
        Kind::Selector(legends) => {
            none("big", e.big.is_some())?;
            none("size", e.size.is_some())?;
            none("colour", e.colour.is_some())?;
            none("on", e.on.is_some())?;
            none("across", e.across.is_some())?;
            Kind::Selector(legends)
        }
        Kind::Rocker {
            w,
            h,
            colour,
            orient,
        } => {
            none("big", e.big.is_some())?;
            none("across", e.across.is_some())?;
            let (w, h) = size(w, h)?;
            let colour = match e.colour.as_deref() {
                None => colour,
                Some("blue") => Colour::Blue,
                Some("red") => Colour::Red,
                Some("black") => Colour::Black,
                Some("ivory") => Colour::Ivory,
                Some(c) => return Err(format!("`{c}`: a rocker is blue, red, black or ivory")),
            };
            let orient = match e.on.as_deref() {
                None => orient,
                Some("right") => Orient::Right,
                Some("top") => Orient::Top,
                Some("bottom") => Orient::Bottom,
                Some(o) => {
                    return Err(format!("`{o}`: a rocker is on at its right, top or bottom"));
                }
            };
            Kind::Rocker {
                w,
                h,
                colour,
                orient,
            }
        }
        Kind::Toggle { w, h, across } => {
            none("big", e.big.is_some())?;
            none("colour", e.colour.is_some())?;
            none("on", e.on.is_some())?;
            let (w, h) = size(w, h)?;
            Kind::Toggle {
                w,
                h,
                across: e.across.unwrap_or(across),
            }
        }
        Kind::Wheel { .. } => return Err("a wheel is the plug-in's own".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The official layout places every control where the plug-in's table has it, as the
    /// approved panel did before the layout became data.
    #[test]
    fn the_official_layout_is_the_panel_as_it_was() {
        let mut report = Report::default();
        let a = Arrangement::read(OFFICIAL, &mut report).unwrap_or_else(|| panic!("{report}"));
        assert!(!report.rejected(), "{report}");
        assert_eq!(a.controls.len(), CONTROLS.len());
        for (c, o) in a.controls.iter().zip(CONTROLS.iter()) {
            assert_eq!(c, o, "{}", o.param);
        }
        assert_eq!(a.power, art::POWER);
        assert_eq!(a.texts.len(), 64);
        assert_eq!(a.titles.len(), 5);
        assert_eq!(a.lines.len(), 15);
    }

    fn rejected(text: &str) -> Report {
        let mut report = Report::default();
        assert!(Arrangement::read(text, &mut report).is_none(), "accepted");
        report
    }

    #[test]
    fn a_layout_places_every_control_once_and_only_the_panels() {
        let without_tune = OFFICIAL.replacen("param = \"tune\"", "param = \"glide\"", 1);
        let r = rejected(&without_tune);
        assert!(
            r.findings
                .iter()
                .any(|f| f.message.contains("placed twice")),
            "{r}"
        );
        assert!(
            r.findings
                .iter()
                .any(|f| f.message.contains("`tune` is not placed")),
            "{r}"
        );
        let strip = format!("{OFFICIAL}\n[[control]]\nparam = \"drive\"\nat = [10, 10]\n");
        assert!(
            rejected(&strip).findings[0]
                .message
                .contains("plug-in's own")
        );
    }

    #[test]
    fn a_layout_changes_a_controls_look_not_what_it_is() {
        let r = rejected(&OFFICIAL.replacen(
            "param = \"tune\"\nat = [229, 273]",
            "param = \"tune\"\nat = [229, 273]\ncolour = \"red\"",
            1,
        ));
        assert!(r.findings[0].message.contains("does not apply"), "{r}");
        let mut report = Report::default();
        let a = Arrangement::read(
            &OFFICIAL.replacen(
                "colour = \"red\"\non = \"right\"",
                "colour = \"black\"\non = \"top\"",
                1,
            ),
            &mut report,
        )
        .unwrap_or_else(|| panic!("{report}"));
        let i = crate::controls::index("osc_mod").expect("a control");
        assert!(matches!(
            a.controls[i].kind,
            Kind::Rocker {
                colour: Colour::Black,
                orient: Orient::Top,
                ..
            }
        ));
    }

    #[test]
    fn controls_outside_the_panel_or_on_one_another_are_refused() {
        let r = rejected(&OFFICIAL.replacen("at = [229, 273]", "at = [-10, 273]", 1));
        assert!(r.has("layout.outside"), "{r}");
        let r = rejected(&OFFICIAL.replacen("at = [229, 273]", "at = [126, 470]", 1));
        assert!(r.has("layout.overlap"), "{r}");
        let r = rejected(&OFFICIAL.replacen("at = [229, 273]", "at = [nan, 273]", 1));
        assert!(r.has("layout.number") || r.has("layout.syntax"), "{r}");
    }

    #[test]
    fn print_within_its_limits() {
        let long = format!(
            "{OFFICIAL}\n[[text]]\ntext = \"{}\"\nat = [10, 10]\n",
            "X".repeat(60)
        );
        assert!(rejected(&long).has("layout.text"));
        let wide = format!("{OFFICIAL}\n[[line]]\nfrom = [0, 0]\nto = [10, 0]\nwidth = 90\n");
        assert!(rejected(&wide).has("layout.line"));
        let newer = OFFICIAL.replacen("format = 1", "format = 2", 1);
        assert!(rejected(&newer).has("layout.format"));
    }
}
