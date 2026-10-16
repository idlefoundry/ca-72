//! A look: what the editor's panel is drawn with (decisions.md R-SKIN-PACKS). It is one of the
//! two built in, CLASSIC and REALISTIC, or a skin pack's: a folder of artwork and declarative
//! appearance data in the format every pack uses (`docs/skin-packs.md`; the built-in two are
//! packs too, `packs/official/`). A pack brings no code: the open-source renderer draws the
//! controls, the pack says where they are and what they look like.
//!
//! A pack is untrusted input. It is read and checked by `skin_pack` (its paths, its files'
//! kinds and sizes, its SVG, its fonts, its manifest), then here: its `[look]` table, its
//! layout (`crate::arrangement`), its pictures' slots, and whether its fonts print every legend
//! and fit them on the panel. A pack that fails is not loaded, and the editor draws the
//! built-in look instead; nothing a look does reaches the sound, the presets, automation or
//! what a session saves (the look chosen is the computer's setting, not the session's).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;
use serde::Deserialize;
use skin_pack::{Context, Limits, Pack, Report};

use crate::arrangement::Arrangement;
use crate::art::{self, Lettering};
use crate::fonts::{Fonts, Weight};
use crate::skin::{self, Overrides, Skin};

/// The plug-in's id in a pack's manifest.
pub const PLUGIN: &str = "ca-72";

/// The plug-in's own artwork's licence.
pub const OFFICIAL_LICENCE: &str = "GPL-3.0-or-later";

/// The built-in looks' ids.
pub const CLASSIC: &str = "com.idlefoundry.ca72.classic";
pub const REALISTIC: &str = "com.idlefoundry.ca72.realistic";

/// A look the panel can be drawn in.
#[derive(Clone)]
pub struct Look {
    pub id: String,
    pub name: String,
    pub version: String,
    /// Which of the renderer's two ways of drawing the controls it uses.
    pub base: Skin,
    pub arrangement: Arc<Arrangement>,
    /// Its pictures, where they are not the built-in ones (decoded), and its face picture's
    /// colour (each channel scaled).
    pub pictures: Arc<Overrides>,
    pub face_tint: Option<[f32; 3]>,
    /// Its drawing over the panel's surfaces, under its print.
    pub art: Option<Arc<usvg::Tree>>,
    /// Its typeface (regular, bold), if it brings one that prints and fits every legend.
    pub fonts: Option<(&'static [u8], &'static [u8])>,
    /// Its licence (an SPDX expression), its creators, and what its checks found.
    pub licence: String,
    pub creators: Vec<String>,
    pub report: Report,
}

impl std::fmt::Debug for Look {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Look")
            .field("id", &self.id)
            .field("base", &self.base)
            .field("pictures", &self.pictures.len())
            .field("art", &self.art.is_some())
            .field("fonts", &self.fonts.is_some())
            .finish()
    }
}

impl Look {
    fn built_in(id: &str, name: &str, base: Skin) -> Look {
        Look {
            id: id.into(),
            name: name.into(),
            version: "1.0.0".into(),
            base,
            arrangement: Arrangement::official(),
            pictures: Arc::new(Overrides::new()),
            face_tint: None,
            art: None,
            fonts: None,
            licence: OFFICIAL_LICENCE.into(),
            creators: vec!["Idle Foundry Ltd.".into()],
            report: Report::default(),
        }
    }

    /// CLASSIC: the panel as drawn.
    pub fn classic() -> Look {
        Self::built_in(CLASSIC, "CLASSIC", Skin::Drawn)
    }

    /// REALISTIC: the panel in pictures of the original's materials.
    pub fn realistic() -> Look {
        Self::built_in(REALISTIC, "REALISTIC", Skin::Worn)
    }

    /// The built-in look drawn in `skin`.
    pub fn of(skin: Skin) -> Look {
        match skin {
            Skin::Drawn => Self::classic(),
            Skin::Worn => Self::realistic(),
        }
    }

    /// Whether it is one of the plug-in's own.
    pub fn is_built_in(&self) -> bool {
        self.id == CLASSIC || self.id == REALISTIC
    }

    /// The pack in the folder at `dir`, read and checked: its look, or what rejected it.
    pub fn open(dir: &Path) -> Result<Look, Report> {
        Self::open_with(dir, &Limits::default())
    }

    pub fn open_with(dir: &Path, limits: &Limits) -> Result<Look, Report> {
        let official = skin::official_pictures();
        let context = Context {
            plugin: PLUGIN,
            official: &official,
            official_licence: OFFICIAL_LICENCE,
            built_in: false,
            test_drawings: true,
        };
        let pack = Pack::open(dir, limits, &context)?;
        Self::from_pack(pack, limits)
    }

    /// A checked pack's look.
    pub fn from_pack(pack: Pack, limits: &Limits) -> Result<Look, Report> {
        let mut report = pack.report.clone();
        let m = &pack.manifest;
        let at = Some(skin_pack::MANIFEST);
        let Some(table) = m.look.clone() else {
            report.reject("look.missing", at, "no `[look]` table: what the pack draws");
            return Err(report);
        };
        let spec: LookTable = match table.try_into() {
            Ok(t) => t,
            Err(e) => {
                report.reject("look.fields", at, e.to_string());
                return Err(report);
            }
        };
        let base = match spec.base.as_str() {
            "classic" => Skin::Drawn,
            "realistic" => Skin::Worn,
            b => {
                report.reject(
                    "look.base",
                    at,
                    format!("`{b}`: a look is drawn as `classic` or `realistic`"),
                );
                return Err(report);
            }
        };
        let file = |path: &str, report: &mut Report| -> Option<Vec<u8>> {
            let bytes = pack.files.get(path).map(<[u8]>::to_vec);
            if bytes.is_none() {
                report.reject(
                    "look.file",
                    Some(path),
                    "named in `[look]`, not in the pack",
                );
            }
            bytes
        };

        // The layout: the pack's, or the official one.
        let arrangement = match &spec.layout {
            None => Arrangement::official(),
            Some(path) => {
                let text = pack.text(path).map(str::to_owned);
                match text {
                    None => {
                        report.reject(
                            "look.file",
                            Some(path),
                            "the layout is not a text file in the pack",
                        );
                        return Err(report);
                    }
                    Some(t) => match Arrangement::read_at(&t, path, &mut report) {
                        Some(a) => Arc::new(a),
                        None => return Err(report),
                    },
                }
            }
        };

        // The pictures, by slot: PNG as it is, SVG drawn at the built-in picture's size.
        let mut pictures = Overrides::new();
        for (name, path) in &spec.pictures {
            let Some(&(slot, official)) = skin::SLOTS.iter().find(|(n, _)| n == name) else {
                report.reject(
                    "look.picture",
                    at,
                    format!("`{name}`: no such picture (the slots are in docs/skin-packs.md)"),
                );
                continue;
            };
            let Some(bytes) = file(path, &mut report) else {
                continue;
            };
            let picture = if path.ends_with(".svg") {
                let size = official_size(official);
                skin_pack::svg::parse(&bytes, limits)
                    .ok()
                    .and_then(|(tree, _)| draw_at(&tree, size))
            } else if path.ends_with(".png") {
                Pixmap::decode_png(&bytes).ok()
            } else {
                None
            };
            match picture {
                Some(p) => {
                    let (ow, oh) = official_size(official);
                    let (k0, k1) = (
                        f64::from(p.width()) / f64::from(p.height()),
                        f64::from(ow) / f64::from(oh),
                    );
                    if (k0 / k1 - 1.0).abs() > 0.02 {
                        report.note(
                            "look.picture",
                            Some(path),
                            format!(
                                "drawn at the built-in {name}'s proportions ({ow} by {oh}): it is {} by {}",
                                p.width(),
                                p.height()
                            ),
                        );
                    }
                    pictures.insert(slot, p);
                }
                None => report.reject("look.picture", Some(path), "a PNG or SVG picture"),
            }
        }

        if let Some(t) = spec.face_tint
            && !t.iter().all(|v| (0.0..=2.0).contains(v))
        {
            report.reject("look.face-tint", at, "a face tint of 0 to 2 a channel");
        }

        // The drawing over the panel.
        let art = match &spec.art {
            None => None,
            Some(path) if path.ends_with(".svg") => {
                let bytes = file(path, &mut report);
                match bytes.map(|b| skin_pack::svg::parse(&b, limits)) {
                    Some(Ok((tree, _))) => Some(Arc::new(tree)),
                    Some(Err(why)) => {
                        report.reject("look.art", Some(path), why);
                        None
                    }
                    None => None,
                }
            }
            Some(path) => {
                report.reject("look.art", Some(path), "the panel's drawing is SVG");
                None
            }
        };

        // The typeface: it must print every legend and keep them on the panel, or the
        // built-in one is used (and the pack goes to review).
        let fonts = match &spec.fonts {
            None => None,
            Some(f) => {
                let regular = file(&f.regular, &mut report);
                let bold = match &f.bold {
                    Some(b) => file(b, &mut report),
                    None => regular.clone(),
                };
                match (regular, bold) {
                    (Some(r), Some(b)) => {
                        let pair = (keep(r), keep(b));
                        fitting(pair, &arrangement, &mut report).then_some(pair)
                    }
                    _ => None,
                }
            }
        };
        if report.rejected() {
            return Err(report);
        }
        Ok(Look {
            id: m.id.clone(),
            name: m.name.clone(),
            version: m.version.clone(),
            base,
            arrangement,
            pictures: Arc::new(pictures),
            face_tint: spec.face_tint,
            art,
            fonts,
            licence: m.license.clone(),
            creators: m.creators.iter().map(|c| c.name.clone()).collect(),
            report,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LookTable {
    base: String,
    #[serde(default)]
    layout: Option<String>,
    #[serde(default)]
    art: Option<String>,
    #[serde(default)]
    fonts: Option<FontsTable>,
    #[serde(default)]
    pictures: BTreeMap<String, String>,
    #[serde(default)]
    face_tint: Option<[f32; 3]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FontsTable {
    regular: String,
    #[serde(default)]
    bold: Option<String>,
}

/// A built-in picture's size, from its header.
fn official_size(png: &[u8]) -> (u32, u32) {
    let be = |i: usize| u32::from_be_bytes([png[i], png[i + 1], png[i + 2], png[i + 3]]);
    (be(16), be(20))
}

/// A drawing drawn `size` pixels, stretched to it.
fn draw_at(tree: &usvg::Tree, (w, h): (u32, u32)) -> Option<Pixmap> {
    let mut p = Pixmap::new(w, h)?;
    let s = tree.size();
    resvg::render(
        tree,
        Transform::from_scale(w as f32 / s.width(), h as f32 / s.height()),
        &mut p.as_mut(),
    );
    Some(p)
}

/// A pack's font's bytes kept for as long as the plug-in runs (its faces borrow them), each
/// distinct font once however often a pack is chosen.
fn keep(bytes: Vec<u8>) -> &'static [u8] {
    static KEPT: OnceLock<Mutex<Vec<&'static [u8]>>> = OnceLock::new();
    let kept = KEPT.get_or_init(|| Mutex::new(Vec::new()));
    let mut kept = match kept.lock() {
        Ok(k) => k,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(k) = kept.iter().find(|k| **k == bytes.as_slice()) {
        return k;
    }
    let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    kept.push(leaked);
    leaked
}

/// Whether a pack's typeface prints every legend of `arrangement` and keeps each on the panel
/// in both letterings (reported if not).
fn fitting(
    (regular, bold): (&'static [u8], &'static [u8]),
    arrangement: &Arrangement,
    report: &mut Report,
) -> bool {
    let fonts = match Fonts::with(regular, bold) {
        Ok(f) => f,
        Err(why) => {
            report.review(
                "look.fonts",
                None,
                format!("{why}: the built-in typeface is used"),
            );
            return false;
        }
    };
    let texts = arrangement
        .texts
        .iter()
        .map(|t| t.text.as_str())
        .chain(arrangement.titles.iter().map(|t| t.text.as_str()))
        .chain([art::PLATE_NAME]);
    for t in texts {
        if !fonts.covers(t) {
            report.review(
                "look.fonts",
                None,
                format!(
                    "its typeface has no glyph for some of \"{t}\": the built-in typeface is used"
                ),
            );
            return false;
        }
    }
    for lettering in [Lettering::Standard, Lettering::Large] {
        for t in &arrangement.texts {
            let size = t.size.unwrap_or(lettering.legend());
            let (x, y) = t.at_in(lettering);
            let half = fonts.advance(&t.text, size, Weight::Bold, 0.0) * art::STRETCH / 2.0;
            let inside = x - half >= 0.0
                && x + half <= art::PW
                && y - size >= -art::TOP
                && y + size <= art::PH;
            if !inside {
                report.review(
                    "look.fonts",
                    None,
                    format!(
                        "in its typeface \"{}\" runs off the panel: the built-in typeface is used",
                        t.text
                    ),
                );
                return false;
            }
        }
    }
    true
}

/// Every pack in the folder at `dir` (each a folder of its own), read and checked: its look, or
/// its folder's name and what rejected it. A folder that is not a pack's is passed over.
pub fn discover(dir: &Path) -> Vec<Result<Look, (String, Report)>> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut folders: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|e| e.path().join(skin_pack::MANIFEST).is_file())
        .collect();
    folders.sort_by_key(std::fs::DirEntry::file_name);
    folders
        .into_iter()
        .take(64)
        .map(|e| {
            Look::open(&e.path()).map_err(|r| (e.file_name().to_string_lossy().into_owned(), r))
        })
        .collect()
}
