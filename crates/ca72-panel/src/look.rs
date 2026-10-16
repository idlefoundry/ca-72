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
use skin_pack::paths::{self, FileKind};
use skin_pack::signature::{Signed, TrustedKey};
use skin_pack::{Context, Limits, Pack, Report};

use crate::arrangement::Arrangement;
use crate::art::{self, Lettering};
use crate::fonts::{Fonts, Weight};
use crate::skin::{self, Overrides, Skin};

/// The plug-in's id in a pack's manifest.
pub const PLUGIN: &str = "ca-72";

/// The plug-in's own artwork's licence.
pub const OFFICIAL_LICENCE: &str = "GPL-3.0-or-later";

/// The marketplace's public keys this plug-in trusts, each by the id its signatures name.
///
/// None yet: there is no marketplace. Its keys are made by its owner, offline
/// (`cargo run -p skin-pack --example pack_keygen`), the secret kept where the marketplace signs
/// and nowhere else, and only the public half put here, a new one added before an old one is
/// retired (a plug-in does not hear of a key's revocation until it is updated). Until then every
/// pack loads as unsigned.
pub const TRUSTED_KEYS: &[TrustedKey] = &[];

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
    /// Its signature, checked: who published it, if the marketplace did.
    pub signed: Option<Signed>,
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
            signed: None,
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

    /// The pack at `path` (a folder, or a pack file, `<name>.ifskin`), read and checked: its
    /// look, or what rejected it.
    pub fn open(path: &Path) -> Result<Look, Report> {
        Self::open_with(path, &Limits::default(), TRUSTED_KEYS)
    }

    /// The same, within `limits`, its signature checked against `keys`.
    pub fn open_with(path: &Path, limits: &Limits, keys: &[TrustedKey]) -> Result<Look, Report> {
        let official = skin::official_pictures();
        let context = Context {
            plugin: PLUGIN,
            official: &official,
            official_licence: OFFICIAL_LICENCE,
            built_in: false,
            test_drawings: true,
            keys,
        };
        let pack = Pack::open(path, limits, &context)?;
        // The built-in looks' own ids are theirs alone: an installed pack, signed or not, never
        // takes one.
        if pack.manifest.id == CLASSIC || pack.manifest.id == REALISTIC {
            let mut report = pack.report;
            report.reject(
                "manifest.id",
                Some(skin_pack::MANIFEST),
                "the id of a look built into the plug-in",
            );
            return Err(report);
        }
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
        // A file `[look]` names, of a kind it may be: every file the renderer uses has passed
        // the check for its kind (a font named `.txt` would have skipped the font's).
        let file = |path: &str, kinds: &[FileKind], report: &mut Report| -> Option<Vec<u8>> {
            if !paths::kind(path).is_some_and(|k| kinds.contains(&k)) {
                report.reject(
                    "look.file",
                    Some(path),
                    format!("named in `[look]` as one of {kinds:?}, which it is not"),
                );
                return None;
            }
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
                let text = path
                    .ends_with(".toml")
                    .then(|| pack.text(path).map(str::to_owned))
                    .flatten();
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
            let Some(bytes) = file(path, &[FileKind::Png, FileKind::Svg], &mut report) else {
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
                let bytes = file(path, &[FileKind::Svg], &mut report);
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
                let regular = file(&f.regular, &[FileKind::Font], &mut report);
                let bold = match &f.bold {
                    Some(b) => file(b, &[FileKind::Font], &mut report),
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
            signed: pack.signed.clone(),
        })
    }

    /// Published by Idle Foundry (signed as its own).
    pub fn first_party(&self) -> bool {
        self.signed
            .as_ref()
            .is_some_and(|s| s.publisher == skin_pack::signature::Publisher::FirstParty)
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

/// Every pack in the folder at `dir` (a folder or a pack file each), read and checked against
/// [`TRUSTED_KEYS`]: its look, or its name there and what rejected it. Something that is not a
/// pack is passed over (no link is followed). Of the same pack (its id) found more than once,
/// one is kept ([`supersedes`]): a pack's update, added beside it, is the one used.
pub fn discover(dir: &Path) -> Vec<Result<Look, (String, Report)>> {
    discover_with(dir, TRUSTED_KEYS)
}

pub fn discover_with(dir: &Path, keys: &[TrustedKey]) -> Vec<Result<Look, (String, Report)>> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| match e.file_type() {
            Ok(t) if t.is_dir() => e.path().join(skin_pack::MANIFEST).is_file(),
            Ok(t) if t.is_file() => skin_pack::archive::is_pack_file(&e.path()),
            _ => false,
        })
        .collect();
    found.sort_by_key(std::fs::DirEntry::file_name);
    let mut out: Vec<Result<Look, (String, Report)>> = Vec::new();
    for e in found.into_iter().take(64) {
        match Look::open_with(&e.path(), &Limits::default(), keys) {
            Ok(look) => {
                let same = out
                    .iter()
                    .position(|o| o.as_ref().is_ok_and(|l| l.id == look.id));
                match same {
                    Some(i) => {
                        let replaces = out[i].as_ref().is_ok_and(|old| supersedes(&look, old));
                        if replaces {
                            out[i] = Ok(look);
                        }
                    }
                    None => out.push(Ok(look)),
                }
            }
            Err(r) => out.push(Err((e.file_name().to_string_lossy().into_owned(), r))),
        }
    }
    out
}

/// Whether `a` is used rather than `b`, the same pack: a signed one over an unsigned one (an
/// unsigned copy, whatever its version, never takes a published pack's place), then the newer.
fn supersedes(a: &Look, b: &Look) -> bool {
    match (a.signed.is_some(), b.signed.is_some()) {
        (true, false) => true,
        (false, true) => false,
        _ => newer(&a.version, &b.version),
    }
}

/// Whether version `a` (`major.minor.patch`, as a manifest's) is newer than `b`.
fn newer(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> { v.split('.').map(|p| p.parse().unwrap_or(0)).collect() };
    parts(a) > parts(b)
}
