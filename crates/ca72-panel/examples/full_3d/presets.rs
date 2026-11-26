//! The modeled drawer with the editor's live browser artwork and a local review library.

use super::*;
use ca72_panel::art::{SIDE_TRIM_W, W};
use ca72_panel::presets::{Current, DRAWER_H, DrawerRenderer, DrawerScene, Row};
#[cfg(target_os = "linux")]
use ca72_panel::presets::{
    DrawerTarget, FieldId, PresetKey, ROWS_SHOWN, Tab, caret_at, drawer_hit,
};

pub(super) struct Presets {
    drawer: DrawerRenderer,
    scene: DrawerScene,
    library: Vec<Row>,
    selected: usize,
    origin: (i32, i32),
    #[cfg(target_os = "linux")]
    scale: f64,
    pub(super) bounds: FaceRect,
    pub(super) shown: bool,
}

impl Presets {
    pub(super) fn load(dir: &Path, scale: f64, left: i32, top: i32) -> Option<Self> {
        let path = dir.join("page-presets-3d.png");
        if !path.is_file() {
            return None;
        }
        let face = Pixmap::load_png(path).expect("load the Cycles preset browser");
        let row = (strip::RAIL * scale).round() as i32;
        let width = (W * scale).ceil() as u32;
        let height = (DRAWER_H * scale).ceil() as u32;
        let crop = IntRect::from_xywh(left, row, width, height).expect("preset browser dimensions");
        let plate = face
            .clone_rect(crop)
            .expect("preset browser inside lower plate");
        let side = (SIDE_TRIM_W * scale).round() as i32;
        let bounds = FaceRect {
            x0: left + side,
            x1: left + ((W - SIDE_TRIM_W) * scale).round() as i32,
            y0: top + row,
            y1: top + face.height() as i32 - (6.0 * scale).round() as i32,
        };
        let library = [
            ("Brass Tutti", "brass bright"),
            ("Deep Mod Bass", "bass warm"),
            ("Glass Lead", "lead bright"),
            ("Night Pad", "pad warm"),
            ("Pluck Ladder", "pluck bright"),
            ("Soft Brass", "brass soft"),
            ("Sub Pressure", "bass dark"),
            ("Wire Lead", "lead bright"),
            ("Slow Drift", "pad soft"),
            ("Rubber Bass", "bass soft"),
            ("Hollow Keys", "keys dark"),
            ("Low Octave", "bass dark"),
            ("Air Brass", "brass soft"),
            ("Bent Wire", "lead dark"),
            ("Slow Strings", "pad warm"),
            ("Hard Sync", "lead bright"),
            ("Quiet Fifths", "keys soft"),
            ("After Hours", "pad dark"),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (name, tags))| Row {
            name: name.into(),
            tags: tags.into(),
            origin: (i >= 14).then_some("YOURS"),
            favorite: i % 4 == 0,
            current: i == 0,
        })
        .collect::<Vec<_>>();
        let scene = DrawerScene {
            chips: [
                "bass", "brass", "bright", "dark", "keys", "lead", "pad", "pluck", "soft", "warm",
            ]
            .map(|tag| (tag.into(), false))
            .to_vec(),
            total: library.len(),
            ..DrawerScene::default()
        };
        let mut presets = Self {
            drawer: DrawerRenderer::with_cycles_plate(scale, plate),
            scene,
            library,
            selected: 0,
            origin: (left, top + row),
            #[cfg(target_os = "linux")]
            scale,
            bounds,
            shown: false,
        };
        presets.refresh();
        Some(presets)
    }

    fn refresh(&mut self) {
        let row = &self.library[self.selected];
        self.scene.current = Some(Current {
            name: row.name.clone(),
            tags: row.tags.clone(),
            origin: row.origin,
            favorite: row.favorite,
        });
        let search = self.scene.search.text.trim().to_ascii_lowercase();
        self.scene.rows =
            self.library
                .iter()
                .filter(|row| {
                    (!self.scene.factory || row.origin != Some("YOURS"))
                        && (!self.scene.mine || row.origin == Some("YOURS"))
                        && (!self.scene.favourites || row.favorite)
                        && (row.name.to_ascii_lowercase().contains(&search)
                            || row.tags.contains(&search))
                        && self.scene.chips.iter().all(|(tag, on)| {
                            !on || row.tags.split_whitespace().any(|word| word == tag)
                        })
                })
                .cloned()
                .collect();
    }

    pub(super) fn paint(&mut self, frame: &mut Pixmap, damage: &mut Vec<FaceRect>) {
        let changed = self.drawer.render(&self.scene);
        if !self.shown || changed || damage.iter().any(|r| r.intersects(self.bounds)) {
            preview::copy(
                frame,
                self.drawer.frame(),
                self.bounds,
                -self.origin.0,
                -self.origin.1,
                0,
                0,
            );
            damage.push(self.bounds);
        }
        self.shown = true;
    }

    pub(super) fn clear_pointer(&mut self) {
        self.scene.hover = None;
        self.scene.focus = None;
    }

    #[cfg(target_os = "linux")]
    fn point(&self, x: f32, y: f32) -> Option<(f64, f64)> {
        ((self.bounds.x0 as f32..self.bounds.x1 as f32).contains(&x)
            && (self.bounds.y0 as f32..self.bounds.y1 as f32).contains(&y))
        .then(|| {
            (
                (f64::from(x) - f64::from(self.origin.0)) / self.scale,
                (f64::from(y) - f64::from(self.origin.1)) / self.scale,
            )
        })
    }

    #[cfg(target_os = "linux")]
    pub(super) fn hover(&mut self, point: Option<(f32, f32)>) {
        self.scene.hover = point
            .and_then(|(x, y)| self.point(x, y))
            .and_then(|(x, y)| drawer_hit(&self.scene, x, y));
    }

    #[cfg(target_os = "linux")]
    pub(super) fn press(&mut self, x: f32, y: f32, bar: &mut BarScene) -> bool {
        let Some((x, y)) = self.point(x, y) else {
            return false;
        };
        let target = drawer_hit(&self.scene, x, y);
        self.scene.focus = None;
        match target {
            Some(DrawerTarget::Field(FieldId::Search)) => {
                self.scene.search.caret = caret_at(&self.scene, FieldId::Search, x);
                self.scene.focus = Some(FieldId::Search);
            }
            Some(DrawerTarget::Tab(tab)) => {
                self.scene.factory = tab == Tab::Factory;
                self.scene.mine = tab == Tab::Mine;
                self.scene.favourites = tab == Tab::Favourites;
                self.scene.first = 0;
            }
            Some(DrawerTarget::Chip(i)) => {
                self.scene.chips[i].1 ^= true;
                self.scene.first = 0;
            }
            Some(DrawerTarget::Row(i) | DrawerTarget::Star(i)) => {
                let name = &self.scene.rows[i].name;
                if let Some(index) = self.library.iter().position(|row| row.name == *name) {
                    if target == Some(DrawerTarget::Star(i)) {
                        self.library[index].favorite ^= true;
                    } else {
                        self.library[self.selected].current = false;
                        self.selected = index;
                        self.library[index].current = true;
                    }
                    let row = &self.library[self.selected];
                    bar.name.clone_from(&row.name);
                    bar.favorite = row.favorite;
                }
            }
            Some(DrawerTarget::Key(PresetKey::Close)) => {
                bar.open = false;
                bar.below = false;
                self.clear_pointer();
            }
            _ => {}
        }
        self.refresh();
        self.scene.first = self
            .scene
            .first
            .min(self.scene.rows.len().saturating_sub(ROWS_SHOWN));
        true
    }

    #[cfg(target_os = "linux")]
    pub(super) fn scroll(&mut self, x: f32, y: f32, direction: f32) -> bool {
        let Some((x, y)) = self.point(x, y) else {
            return false;
        };
        if matches!(
            drawer_hit(&self.scene, x, y),
            Some(DrawerTarget::Row(_) | DrawerTarget::Star(_))
        ) {
            let max = self.scene.rows.len().saturating_sub(ROWS_SHOWN);
            self.scene.first = self
                .scene
                .first
                .saturating_add_signed(-direction.signum() as isize)
                .min(max);
            self.hover(None);
        }
        true
    }

    #[cfg(target_os = "linux")]
    pub(super) fn key(&mut self, symbol: u32) {
        if self.scene.focus != Some(FieldId::Search) {
            return;
        }
        let search = &mut self.scene.search;
        match symbol {
            0xff08 => search.backspace(),
            0xffff => search.delete(),
            0xff51 => search.left(),
            0xff53 => search.right(),
            0xff50 => search.home(),
            0xff57 => search.end(),
            0xff0d => self.scene.focus = None,
            0x20..=0x7e => search.insert(&char::from(symbol as u8).to_string()),
            _ => return,
        }
        self.scene.first = 0;
        self.refresh();
    }
}
