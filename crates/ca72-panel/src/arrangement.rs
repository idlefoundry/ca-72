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

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use skin_pack::Report;

use crate::art::{self, PH, PW, TOP};
use crate::controls::{CONTROL_SET, CONTROLS, Colour, Control, ControlSet, Kind, Orient, Place};

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

/// What a layout's tables hold that this build does not read: noted, not refused (a field a
/// later format adds is passed over by an earlier plug-in).
type Rest = BTreeMap<String, toml::Value>;

#[derive(Deserialize)]
struct File {
    format: u32,
    /// The revision of the panel's controls it was made for (`controls::ControlSet`).
    #[serde(default)]
    controls: Option<u32>,
    fixtures: Fixtures,
    #[serde(default)]
    control: Vec<ControlEntry>,
    #[serde(default)]
    line: Vec<LineEntry>,
    #[serde(default)]
    title: Vec<TitleEntry>,
    #[serde(default)]
    text: Vec<TextEntry>,
    #[serde(flatten)]
    rest: Rest,
}

#[derive(Deserialize)]
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
    #[serde(flatten)]
    rest: Rest,
}

#[derive(Deserialize)]
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
    #[serde(flatten)]
    rest: Rest,
}

#[derive(Deserialize)]
struct LineEntry {
    from: [f64; 2],
    to: [f64; 2],
    width: f64,
    #[serde(flatten)]
    rest: Rest,
}

#[derive(Deserialize)]
struct TitleEntry {
    text: String,
    span: [f64; 2],
    baseline: f64,
    #[serde(default)]
    size: Option<f64>,
    #[serde(flatten)]
    rest: Rest,
}

#[derive(Deserialize)]
struct TextEntry {
    text: String,
    at: [f64; 2],
    #[serde(default)]
    size: Option<f64>,
    #[serde(default)]
    large: Option<[f64; 2]>,
    #[serde(flatten)]
    rest: Rest,
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
        Self::read_with(text, path, &CONTROL_SET, report)
    }

    /// The same, against the panel controls' history `set` (this build's, or a test's).
    pub fn read_with(
        text: &str,
        path: &str,
        set: &ControlSet,
        report: &mut Report,
    ) -> Option<Arrangement> {
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
        // The revision of the controls it was made for (a format-1 layout without one: the first).
        let made_for = file.controls.unwrap_or(1);
        let before = report.findings.len();
        let mut notes: Vec<(&'static str, String)> = Vec::new();
        if made_for == 0 {
            report.reject("layout.controls", at, "`controls` is 1 or more".to_owned());
            return None;
        }
        if made_for > set.revision {
            notes.push((
                "layout.newer",
                format!(
                    "made for revision {made_for} of the panel's controls, this CA-72 has {}: \
                     what it does not know is passed over",
                    set.revision
                ),
            ));
        }
        // What this build does not read, noted.
        let mut unknown: Vec<String> = Vec::new();
        let mut rest = |table: &str, r: &Rest| {
            for k in r.keys() {
                let name = format!("{table}{k}");
                if !unknown.contains(&name) {
                    unknown.push(name);
                }
            }
        };
        rest("", &file.rest);
        rest("fixtures.", &file.fixtures.rest);
        file.control.iter().for_each(|e| rest("control.", &e.rest));
        file.line.iter().for_each(|e| rest("line.", &e.rest));
        file.title.iter().for_each(|e| rest("title.", &e.rest));
        file.text.iter().for_each(|e| rest("text.", &e.rest));
        if !unknown.is_empty() {
            notes.push((
                "layout.unknown",
                format!(
                    "passed over, not read by this CA-72: {}",
                    unknown
                        .iter()
                        .take(12)
                        .map(|k| format!("`{k}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
        let mut reject = |code, msg: String| report.reject(code, at, msg);

        // The controls: every one of the panel's, once each, its kind's look only; one renamed
        // since the layout was made found by its old name, one retired since passed over.
        let mut controls: Vec<Control> = CONTROLS.to_vec();
        let mut placed = vec![false; CONTROLS.len()];
        for e in &file.control {
            let name = set.current(&e.param, made_for);
            if name != e.param {
                notes.push(("layout.renamed", format!("`{}` is now `{name}`", e.param)));
            }
            if set.retired_since(name, made_for) {
                notes.push((
                    "layout.retired",
                    format!("`{name}` is no longer a control: passed over"),
                ));
                continue;
            }
            let Some(i) = CONTROLS.iter().position(|c| c.param == name) else {
                if made_for > set.revision {
                    notes.push((
                        "layout.newer",
                        format!("`{name}`: not a control this CA-72 has, passed over"),
                    ));
                } else {
                    reject("layout.control", format!("`{name}`: no such control"));
                }
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
                // Its kind changed since: drawn in the new kind's own look.
                Err(_) if set.kind_changed_since(name, made_for) => {
                    notes.push((
                        "layout.kind",
                        format!("`{name}` is another kind of control now: drawn in its own look"),
                    ));
                    official.kind
                }
                Err(why) => {
                    reject("layout.control", format!("`{name}`: {why}"));
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
        // A control the layout does not place: one added since it was made, placed by the
        // plug-in below; one it should have placed, refused.
        let mut added: Vec<usize> = Vec::new();
        for (i, c) in CONTROLS.iter().enumerate() {
            if c.place == Place::Panel && !placed[i] {
                if set.since(c.param) > made_for {
                    added.push(i);
                } else {
                    reject(
                        "layout.control",
                        format!(
                            "`{}` is not placed: a layout places every control of the panel",
                            c.param
                        ),
                    );
                }
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

        // A control whose printed dial or legends would be cut off at the panel's edges: drawn
        // so, and put to review (its print is the plug-in's, its place the layout's).
        let cut: Vec<&str> = controls
            .iter()
            .enumerate()
            .filter(|(i, c)| c.place == Place::Panel && placed[*i])
            .filter(|(_, c)| {
                let e = art::print_extent(&c.kind);
                e != [0.0; 4] && !inside([c.x + e[0], c.y + e[1], c.x + e[2], c.y + e[3]])
            })
            .map(|(_, c)| c.param)
            .collect();
        let cut_off = (!cut.is_empty()).then(|| {
            format!(
                "the printed dial or legends of {} would be cut off at the panel's edge",
                cut.iter()
                    .map(|p| format!("`{p}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });

        // The fixtures: POWER by its switch's extent, the lamps and QUALITY's opening by their
        // radius.
        let fx = &file.fixtures;
        if !finite(fx.power) || !inside(power_box) {
            reject("layout.outside", "`power` reaches outside the panel".into());
        }
        let fixtures: [(&str, [f64; 2], f64); 3] = [
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
        if let Some(why) = cut_off {
            report.review("layout.print-outside", at, why);
        }
        for (code, why) in notes {
            report.note(code, at, why);
        }
        if report.findings[before..]
            .iter()
            .any(|f| f.severity == skin_pack::Severity::Reject)
        {
            return None;
        }
        // The controls added since the layout was made, placed by the plug-in: each at its
        // official place if nothing of the layout's is there, else at the free place nearest it.
        // A panel with no room: the official layout instead (the pack's pictures, drawing and
        // typeface still its own), and a review.
        if !added.is_empty() {
            let mut occupied: Vec<[f64; 4]> = panel.iter().map(|(_, b)| *b).collect();
            occupied.push(power_box);
            for (_, p, r) in fixtures {
                occupied.push([p[0] - r, p[1] - r, p[0] + r, p[1] + r]);
            }
            occupied.push([
                plate[0],
                plate[1],
                plate[0] + art::PLATE_W,
                plate[1] + art::PLATE_H,
            ]);
            if let Some(p) = fx.phones {
                occupied.push([p[0] - p[2], p[1] - p[2], p[0] + p[2], p[1] + p[2]]);
            }
            for i in added {
                let official = CONTROLS[i];
                match free_spot(&official, &occupied) {
                    Some((x, y)) => {
                        let b = hit_bounds(&official.kind);
                        occupied.push([x + b[0], y + b[1], x + b[2], y + b[3]]);
                        controls[i] = Control { x, y, ..official };
                        report.note(
                            "layout.added",
                            at,
                            format!(
                                "`{}` came after this layout was made: placed {}",
                                official.param,
                                if (x, y) == (official.x, official.y) {
                                    "at its official place".to_owned()
                                } else {
                                    format!(
                                        "at {x:.0}, {y:.0}, the free place nearest its official one"
                                    )
                                }
                            ),
                        );
                    }
                    None => {
                        report.review(
                            "layout.full",
                            at,
                            format!(
                                "no room on the panel for `{}`, which came after this layout was \
                                 made: the official layout is used instead",
                                official.param
                            ),
                        );
                        return Some((*Self::official()).clone());
                    }
                }
            }
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

/// Where a control added since a layout was made goes: at its official place (where `c` is) if
/// it is free there (its click area and its printed dial on the panel, over nothing in
/// `occupied`), else at the free place nearest it, searched outward in rings 8 units apart;
/// `None` if the panel has none.
pub fn free_spot(c: &Control, occupied: &[[f64; 4]]) -> Option<(f64, f64)> {
    let b = hit_bounds(&c.kind);
    let e = art::print_extent(&c.kind);
    let fits = |(x, y): (f64, f64)| {
        let hit = [x + b[0], y + b[1], x + b[2], y + b[3]];
        inside(hit)
            && (e == [0.0; 4] || inside([x + e[0], y + e[1], x + e[2], y + e[3]]))
            && !occupied.iter().any(|o| overlap(hit, *o))
    };
    let (x0, y0) = (c.x, c.y);
    if fits((x0, y0)) {
        return Some((x0, y0));
    }
    const STEP: f64 = 8.0;
    let rings = ((PW.max(PH + TOP)) / STEP).ceil() as i64;
    for n in 1..=rings {
        let r = n as f64 * STEP;
        let mut ring: Vec<(f64, f64)> = Vec::with_capacity(8 * n as usize);
        for k in -n..=n {
            let d = k as f64 * STEP;
            ring.push((x0 + d, y0 - r));
            ring.push((x0 + d, y0 + r));
            if k.abs() < n {
                ring.push((x0 - r, y0 + d));
                ring.push((x0 + r, y0 + d));
            }
        }
        let dist = |p: &(f64, f64)| (p.0 - x0).hypot(p.1 - y0);
        ring.sort_by(|a, b| dist(a).total_cmp(&dist(b)));
        if let Some(p) = ring.into_iter().find(|p| fits(*p)) {
            return Some(p);
        }
    }
    None
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

    // ---- A later CA-72, simulated: the panel's controls as they might change.

    /// A later revision: A-440 added in revision 2, OSC MOD once called `oscillator_modulation`
    /// (renamed in 2), an `old_knob` retired in 2, TUNE's kind changed in 2.
    const LATER: ControlSet = ControlSet {
        revision: 2,
        added: &[("a440", 2)],
        renamed: &[("oscillator_modulation", "osc_mod", 2)],
        retired: &[("old_knob", 2)],
        kind_changed: &[("tune", 2)],
    };

    /// The official layout as a revision-1 layout without A-440.
    fn without_a440() -> String {
        let start = OFFICIAL.find("param = \"a440\"").expect("there");
        let block = OFFICIAL[..start].rfind("[[control]]").expect("its block");
        let end = start + OFFICIAL[start..].find("[[control]]").expect("the next");
        format!("{}{}", &OFFICIAL[..block], &OFFICIAL[end..])
    }

    #[test]
    fn a_control_added_later_goes_to_its_official_place_when_it_is_free() {
        let mut report = Report::default();
        let a = Arrangement::read_with(&without_a440(), "layout.toml", &LATER, &mut report)
            .unwrap_or_else(|| panic!("{report}"));
        let i = crate::controls::index("a440").expect("a control");
        assert_eq!(
            (a.controls[i].x, a.controls[i].y),
            (CONTROLS[i].x, CONTROLS[i].y)
        );
        assert!(report.has("layout.added") && !report.rejected(), "{report}");
        // Without the later revision, the same layout is refused: it should have placed it.
        let mut report = Report::default();
        assert!(Arrangement::read(&without_a440(), &mut report).is_none());
        assert!(report.has("layout.control"));
    }

    #[test]
    fn a_control_added_later_goes_to_the_nearest_free_place() {
        // FEEDBACK moved onto A-440's official place.
        let text = without_a440().replacen("at = [2680, 580]", "at = [2682, 375]", 1);
        let mut report = Report::default();
        let a = Arrangement::read_with(&text, "layout.toml", &LATER, &mut report)
            .unwrap_or_else(|| panic!("{report}"));
        let i = crate::controls::index("a440").expect("a control");
        let c = a.controls[i];
        assert_ne!((c.x, c.y), (CONTROLS[i].x, CONTROLS[i].y));
        // On the panel, over nothing, and found where it is.
        let b = hit_bounds(&c.kind);
        let mine = [c.x + b[0], c.y + b[1], c.x + b[2], c.y + b[3]];
        assert!(inside(mine));
        for (j, o) in a.controls.iter().enumerate() {
            if j != i && o.place == Place::Panel {
                let ob = hit_bounds(&o.kind);
                assert!(
                    !overlap(mine, [o.x + ob[0], o.y + ob[1], o.x + ob[2], o.y + ob[3]]),
                    "{}",
                    o.param
                );
            }
        }
        let layout = crate::Layout::measure(&crate::fonts::Fonts::new());
        let (x, y) = c.centre();
        assert_eq!(
            crate::interact::hit(&layout, &a, x, y),
            Some(crate::Target::Control(i))
        );
        let note = report
            .findings
            .iter()
            .find(|f| f.code == "layout.added")
            .expect("noted");
        assert!(note.message.contains("nearest"), "{note}");
    }

    #[test]
    fn a_full_panel_has_no_free_place() {
        let i = crate::controls::index("a440").expect("a control");
        assert_eq!(free_spot(&CONTROLS[i], &[CANVAS]), None);
        assert!(free_spot(&CONTROLS[i], &[]).is_some());
    }

    #[test]
    fn renamed_retired_and_changed_controls_are_followed() {
        let text = OFFICIAL
            .replacen(
                "param = \"osc_mod\"",
                "param = \"oscillator_modulation\"",
                1,
            )
            .replacen(
                "param = \"tune\"\nat = [229, 273]",
                "param = \"tune\"\nat = [229, 273]\ncolour = \"red\"",
                1,
            )
            + "\n[[control]]\nparam = \"old_knob\"\nat = [10, 10]\n";
        let mut report = Report::default();
        let a = Arrangement::read_with(&text, "layout.toml", &LATER, &mut report)
            .unwrap_or_else(|| panic!("{report}"));
        for code in ["layout.renamed", "layout.retired", "layout.kind"] {
            assert!(report.has(code), "{code}: {report}");
        }
        let i = crate::controls::index("osc_mod").expect("a control");
        assert_eq!((a.controls[i].x, a.controls[i].y), (444.0, 269.0));
        // The same on this build's revision: refused (no such names; a field that does not apply).
        let mut report = Report::default();
        assert!(Arrangement::read(&text, &mut report).is_none());
    }

    /// A layout made for a later CA-72, read by this one: names and fields it does not know
    /// passed over, with notes.
    #[test]
    fn a_layout_for_a_later_ca72_loads_what_this_one_knows() {
        let text = OFFICIAL.replacen("controls = 1", "controls = 3", 1)
            + "\n[[control]]\nparam = \"future_knob\"\nat = [10, 10]\nglow = true\n\n\
               [[sticker]]\nat = [5, 5]\n";
        let mut report = Report::default();
        let a = Arrangement::read(&text, &mut report).unwrap_or_else(|| panic!("{report}"));
        assert!(
            report.has("layout.newer") && report.has("layout.unknown"),
            "{report}"
        );
        assert_eq!(a.controls.len(), CONTROLS.len());
        // Made for this revision, a name it does not know is a mistake: refused.
        let text = OFFICIAL.to_owned() + "\n[[control]]\nparam = \"future_knob\"\nat = [10, 10]\n";
        let mut report = Report::default();
        assert!(Arrangement::read(&text, &mut report).is_none());
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
