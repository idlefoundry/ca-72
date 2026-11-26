//! Independent, replaceable lower bays. Their Cycles plates stay cached while hidden.

use super::*;
use ca72_panel::fonts::{FAMILY, Fonts};
use ca72_panel::keyboard::{self, Target};
use ca72_panel::panel3d::DragController;
use ca72_panel::presets::BarTarget;
use resvg::tiny_skia::{FillRule, Paint, PathBuilder, Rect, Stroke};
use resvg::usvg;

const FIRST_NOTE: u8 = 41;
const NAMES: [&str; 3] = ["keys", "lfo", "macros"];

pub(super) struct Pages {
    panes: [Option<Pane>; 3],
    shown: [bool; 3],
    visible: [bool; 3],
    pointer: Option<usize>,
}

struct Pane {
    model: Panel3DRenderer,
    state: Panel3DState,
    picture: Pixmap,
    bounds: FaceRect,
    scale: f32,
    cheek: f32,
    decorations: Vec<Decoration>,
    lit_tab: Pixmap,
    options: usvg::Options<'static>,
    started: bool,
    drag: DragController,
    held: Option<u8>,
    key_drag: bool,
    octave: i8,
}

struct Decoration {
    fixture: Fixture,
    rect: FaceRect,
    content: Option<Content>,
}

#[derive(PartialEq)]
enum Content {
    Key(bool),
    Tab(bool),
    Trace([f32; 7], usize),
    Macro(usize),
    Display(String),
}

impl Pages {
    pub(super) fn load(upper: &Panel3DAssets, build: &Path, lit_tab: &Pixmap) -> Self {
        Self {
            panes: NAMES.map(|name| Pane::load(upper, build, name, lit_tab)),
            shown: [false; 3],
            visible: [false; 3],
            pointer: None,
        }
    }

    pub(super) fn select(&mut self, target: BarTarget, bar: &mut BarScene) {
        match target {
            BarTarget::Global => {
                self.visible[0] = false;
                self.visible[1] = false;
            }
            BarTarget::Keyboard if self.panes[0].is_some() => {
                self.visible[0] = !self.visible[0];
                self.visible[1] = false;
            }
            BarTarget::Lfo if self.panes[1].is_some() => {
                self.visible[1] = !self.visible[1];
                self.visible[0] = false;
            }
            BarTarget::Macro if self.panes[2].is_some() => self.visible[2] = !self.visible[2],
            _ => return,
        }
        bar.keyboard = self.visible[0];
        bar.lfo_page = self.visible[1];
        bar.macros = self.visible[2];
        bar.page = Some(if self.visible[0] {
            BarTarget::Keyboard
        } else if self.visible[1] {
            BarTarget::Lfo
        } else {
            BarTarget::Global
        });
    }

    pub(super) fn has(&self, name: &str) -> bool {
        NAMES
            .iter()
            .position(|n| *n == name)
            .is_some_and(|i| self.panes[i].is_some())
    }

    pub(super) fn paint(
        &mut self,
        frame: &mut Pixmap,
        global: &Pixmap,
        left: i32,
        top: i32,
        damage: &mut Vec<FaceRect>,
    ) {
        let base_damage = damage.len();
        for (i, pane) in self.panes.iter().enumerate() {
            if self.shown[i]
                && !self.visible[i]
                && let Some(pane) = pane
            {
                preview::copy(frame, global, pane.bounds, -left, 0, 0, top);
                damage.push(offset(pane.bounds, top));
            }
        }
        for (i, pane) in self.panes.iter_mut().enumerate() {
            let Some(pane) = pane else { continue };
            let rect = pane.bounds;
            if self.visible[i] {
                let mut changed = pane.paint();
                let area = offset(rect, top);
                if !self.shown[i] || damage[..base_damage].iter().any(|r| r.intersects(area)) {
                    changed.clear();
                    changed.push(rect);
                }
                for r in changed {
                    preview::copy(frame, &pane.picture, r, 0, 0, 0, top);
                    damage.push(offset(r, top));
                }
            }
        }
        self.shown = self.visible;
    }

    fn at(&self, x: f32, y: f32) -> Option<usize> {
        (0..self.panes.len()).find(|&i| {
            self.visible[i]
                && self.panes[i].as_ref().is_some_and(|p| {
                    let r = p.bounds;
                    x >= r.x0 as f32 && x < r.x1 as f32 && y >= r.y0 as f32 && y < r.y1 as f32
                })
        })
    }

    pub(super) fn press(&mut self, x: f32, y: f32) -> bool {
        let Some(i) = self.at(x, y) else { return false };
        let Some(pane) = &mut self.panes[i] else {
            return false;
        };
        self.pointer = Some(i);
        if i == 0 {
            match pane.key_at(x, y) {
                Some(Target::Down) => pane.octave = (pane.octave - 1).max(-3),
                Some(Target::Up) => pane.octave = (pane.octave + 1).min(3),
                Some(Target::Note(note)) => {
                    pane.held = Some(note);
                    pane.key_drag = true;
                }
                _ => {}
            }
        } else if let Some(tab) = pane.decorations.iter_mut().find(|d| {
            d.fixture.kind == "tab"
                && x >= d.rect.x0 as f32
                && x < d.rect.x1 as f32
                && y >= d.rect.y0 as f32
                && y < d.rect.y1 as f32
        }) {
            tab.fixture.value = 1.0 - tab.fixture.value;
        } else {
            pane.drag
                .begin_drag(pane.model.manifest(), &mut pane.state, x, y);
        }
        true
    }

    pub(super) fn motion(&mut self, point: Option<(f32, f32)>, dx: f32, dy: f32, fine: bool) {
        let Some(i) = self.pointer else { return };
        let Some(pane) = &mut self.panes[i] else {
            return;
        };
        if i == 0 && pane.key_drag {
            pane.held = point.and_then(|(x, y)| match pane.key_at(x, y) {
                Some(Target::Note(note)) => Some(note),
                _ => None,
            });
        } else {
            pane.drag
                .update_drag(pane.model.manifest(), &mut pane.state, dx, dy, fine);
        }
    }

    pub(super) fn release(&mut self) {
        if let Some(i) = self.pointer.take()
            && let Some(pane) = &mut self.panes[i]
        {
            pane.held = None;
            pane.key_drag = false;
            pane.drag.end_drag(pane.model.manifest(), &mut pane.state);
        }
    }

    pub(super) fn scroll(&mut self, x: f32, y: f32, delta: f32, fine: bool) -> bool {
        let Some(i) = self.at(x, y) else { return false };
        let Some(pane) = &mut self.panes[i] else {
            return false;
        };
        if i != 0 {
            DragController::scroll(pane.model.manifest(), &mut pane.state, x, y, delta, fine);
        }
        true
    }
}

impl Pane {
    fn load(upper: &Panel3DAssets, dir: &Path, name: &str, lit_tab: &Pixmap) -> Option<Self> {
        let path = dir.join(format!("page-{name}.json"));
        let image = dir.join(format!("page-{name}-3d.png"));
        if !path.is_file() || !image.is_file() {
            return None;
        }
        let layout: LowerLayout =
            serde_json::from_slice(&std::fs::read(path).expect("read page layout"))
                .expect("parse page layout");
        let scale = upper.manifest.face.px_per_unit;
        let cheek = layout.cheek_w * scale;
        let [x0, y0, x1, y1] = layout.bounds.expect("lower page bounds");
        let bounds = FaceRect {
            x0: (cheek + x0 * scale).round() as i32,
            x1: (cheek + x1 * scale).round() as i32,
            y0: (y0 * scale).round() as i32,
            y1: (y1 * scale).round() as i32,
        };
        let mut fixtures = layout.controls;
        fixtures.retain(|f| (x0..x1).contains(&f.x) && (y0..y1).contains(&f.y));
        let face = Pixmap::load_png(image).expect("load Cycles lower page");
        let picture = face.clone();
        let assets = lower_assets(upper, face, layout.cheek_w, &fixtures);
        let state = Panel3DState::from_defaults(&assets.manifest);
        let decorations = fixtures
            .into_iter()
            .filter(|f| {
                matches!(f.kind.as_str(), "tab" | "white_key" | "black_key")
                    || f.param.starts_with("octave")
                    || f.param.starts_with("macro_name_")
                    || f.param.starts_with("lfo_scope_")
                    || f.param.starts_with("lfo_screen_")
            })
            .map(|fixture| {
                let x = cheek + fixture.x * scale;
                let y = fixture.y * scale;
                let w = fixture.w * scale / 2.0;
                let h = fixture.h * scale / 2.0;
                let rect = FaceRect {
                    x0: (x - w).floor() as i32,
                    x1: (x + w).ceil() as i32,
                    y0: (y - h).floor() as i32,
                    y1: (y + h).ceil() as i32,
                };
                Decoration {
                    fixture,
                    rect,
                    content: None,
                }
            })
            .collect();
        let fonts = Fonts::new();
        Some(Self {
            model: Panel3DRenderer::new(assets),
            state,
            picture,
            bounds,
            scale,
            cheek,
            decorations,
            lit_tab: lit_tab.clone(),
            started: false,
            drag: DragController::default(),
            held: None,
            key_drag: false,
            octave: 0,
            options: usvg::Options {
                fontdb: fonts.database(),
                font_family: FAMILY.into(),
                ..usvg::Options::default()
            },
        })
    }

    fn key_at(&self, x: f32, y: f32) -> Option<Target> {
        keyboard::hit(
            FIRST_NOTE,
            f64::from((x - self.cheek) / self.scale),
            PANEL_H + f64::from(y / self.scale),
        )
    }

    fn value(&self, id: &str) -> f32 {
        self.model
            .manifest()
            .controls
            .iter()
            .position(|c| c.param == id)
            .map(|i| self.state.values[i])
            .or_else(|| {
                self.decorations
                    .iter()
                    .find(|d| d.fixture.param == id)
                    .map(|d| d.fixture.value)
            })
            .unwrap_or_default()
    }

    fn paint(&mut self) -> Vec<FaceRect> {
        let hardware = self.model.render_dirty(&self.state);
        for &r in &hardware {
            preview::copy(&mut self.picture, self.model.frame(), r, 0, 0, 0, 0);
        }
        let mut changed = hardware.clone();
        let mut overlays = Vec::new();
        for i in 0..self.decorations.len() {
            let patch = &self.decorations[i];
            let live = self.content(&patch.fixture);
            if self.started
                && patch.content.as_ref() == Some(&live)
                && !hardware.iter().any(|r| r.intersects(patch.rect))
            {
                continue;
            }
            let r = patch.rect;
            preview::copy(&mut self.picture, self.model.frame(), r, 0, 0, 0, 0);
            overlays.push((i, live));
            changed.push(r);
        }
        for (i, live) in overlays {
            self.draw_patch(i, &live);
            self.decorations[i].content = Some(live);
        }
        self.started = true;
        changed
    }

    fn content(&self, fixture: &Fixture) -> Content {
        if fixture.kind == "tab" {
            return Content::Tab(fixture.value >= 0.5);
        }
        if matches!(fixture.kind.as_str(), "white_key" | "black_key") {
            return Content::Key(
                self.held
                    .is_some_and(|note| fixture.param.parse() == Ok(note)),
            );
        }
        if fixture.param == "octave" {
            let root = (FIRST_NOTE as i8 + 12 * self.octave) as u8;
            return Content::Display(format!(
                "{}  –  {}",
                keyboard::label(root),
                keyboard::label(root + 43)
            ));
        }
        if let Some(index) = page_index(&fixture.param, "macro_name_") {
            return Content::Macro(index);
        }
        if let Some(i) = page_index(&fixture.param, "lfo_scope_") {
            return Content::Trace(
                [
                    "waveform", "rate", "phase", "fade", "polarity", "on", "timed",
                ]
                .map(|tag| self.value(&format!("lfo{}_{tag}", i + 1))),
                i,
            );
        }
        Content::Display(if fixture.param.starts_with("octave_") {
            fixture.label.clone()
        } else {
            "NOT IN USE".into()
        })
    }

    fn draw_patch(&mut self, i: usize, content: &Content) {
        let patch = &self.decorations[i];
        let f = &patch.fixture;
        let r = patch.rect;
        let w = (r.x1 - r.x0) as f32;
        let h = (r.y1 - r.y0) as f32;
        match content {
            Content::Key(false) => {}
            Content::Key(true) => {
                let mut path = PathBuilder::new();
                if let Some(([x, y], rest)) = f.shape.split_first() {
                    path.move_to(self.cheek + x * self.scale, y * self.scale);
                    for [x, y] in rest {
                        path.line_to(self.cheek + x * self.scale, y * self.scale);
                    }
                    path.close();
                } else if let Some(rect) =
                    Rect::from_xywh(r.x0 as f32 + 1.5, r.y0 as f32 + 1.0, w - 3.0, h - 2.0)
                {
                    path.push_rect(rect);
                }
                fill(&mut self.picture, path, [0, 0, 0, 87]);
            }
            Content::Tab(lit) => {
                if *lit {
                    let transform = Transform::from_scale(
                        w / self.lit_tab.width() as f32,
                        h / self.lit_tab.height() as f32,
                    )
                    .post_translate(r.x0 as f32, r.y0 as f32);
                    put_scaled(&mut self.picture, &self.lit_tab, transform);
                }
                let ink = if *lit { "#251205" } else { "#d6b18b" };
                let body = text_svg(&f.label, w, h, ink, 0.39);
                self.print(r, &body);
            }
            Content::Trace(values, index) => {
                let inks = [
                    [248, 167, 57],
                    [94, 195, 166],
                    [116, 169, 245],
                    [233, 132, 99],
                ];
                draw_trace(&mut self.picture, r, *values, inks[*index]);
            }
            Content::Macro(index) => {
                let names = ["CUTOFF", "DETUNE", "CONTOUR", "DRIVE", "WIDTH", "GLIDE"];
                let s = f64::from(self.scale);
                strip::macros::name_dots(
                    &mut self.picture,
                    s,
                    (
                        f64::from(r.x0) / s,
                        f64::from(r.y0) / s,
                        f64::from(w) / s,
                        f64::from(h) / s,
                    ),
                    names[*index],
                    [248, 166, 70],
                    false,
                );
            }
            Content::Display(text) => {
                let ink = match f.param.as_str() {
                    "octave" => "#ffae55",
                    "octave_down" | "octave_up" => "#e4d8bf",
                    _ => "#95643c",
                };
                let body = text_svg(text, w, h, ink, 0.46);
                self.print(r, &body);
            }
        }
    }

    fn print(&mut self, r: FaceRect, body: &str) {
        let w = r.x1 - r.x0;
        let h = r.y1 - r.y0;
        let doc = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}'>{body}</svg>"
        );
        let tree = usvg::Tree::from_str(&doc, &self.options).expect("live display text");
        resvg::render(
            &tree,
            Transform::from_translate(r.x0 as f32, r.y0 as f32),
            &mut self.picture.as_mut(),
        );
    }
}

fn offset(mut rect: FaceRect, y: i32) -> FaceRect {
    rect.y0 += y;
    rect.y1 += y;
    rect
}

fn page_index(id: &str, head: &str) -> Option<usize> {
    id.strip_prefix(head)?.parse().ok()
}

fn text_svg(text: &str, w: f32, h: f32, ink: &str, size: f32) -> String {
    let text = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        "<text x='{}' y='{}' text-anchor='middle' font-family='{}' font-size='{}' font-weight='700' fill='{ink}'>{text}</text>",
        w * 0.5,
        h * (0.50 + 0.35 * size),
        FAMILY,
        h * size
    )
}

fn put_scaled(dst: &mut Pixmap, src: &Pixmap, transform: Transform) {
    dst.draw_pixmap(
        0,
        0,
        src.as_ref(),
        &PixmapPaint {
            quality: resvg::tiny_skia::FilterQuality::Bicubic,
            ..PixmapPaint::default()
        },
        transform,
        None,
    );
}

fn fill(target: &mut Pixmap, path: PathBuilder, [r, g, b, a]: [u8; 4]) {
    if let Some(path) = path.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(r, g, b, a);
        target.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn draw_trace(
    target: &mut Pixmap,
    r: FaceRect,
    [wave, rate, phase, fade, uni, on, timed]: [f32; 7],
    [red, green, blue]: [u8; 3],
) {
    let w = (r.x1 - r.x0) as f32;
    let h = (r.y1 - r.y0) as f32;
    let mut line = PathBuilder::new();
    let cycles = if timed >= 0.5 {
        1.0 + (rate * 3.0).round()
    } else {
        0.75 + 2.25 * rate
    };
    for i in 0..=512 {
        let t = i as f32 / 512.0;
        let cycle = (t * cycles + phase).fract();
        let triangle = 1.0 - (cycle * 4.0 - 2.0).abs();
        let signal = match (wave * 5.0).round() as u8 {
            0 => triangle,
            1 => triangle * 0.57 + (2.0 * cycle - 1.0) * 0.43,
            2 => 2.0 * cycle - 1.0,
            n => {
                if cycle < [0.5, 0.68, 0.22][usize::from(n.min(5)) - 3] {
                    1.0
                } else {
                    -1.0
                }
            }
        };
        let signal = if uni >= 0.5 {
            (signal + 1.0) * 0.5
        } else {
            signal
        };
        let signal = signal * (1.0 - (1.0 - t).powi(2) * fade * 0.63);
        let x = r.x0 as f32 + w * (0.055 + 0.89 * t);
        let y = r.y0 as f32 + h * (0.50 - 0.29 * signal);
        if i == 0 {
            line.move_to(x, y);
        } else {
            line.line_to(x, y);
        }
    }
    if let Some(path) = line.finish() {
        for (width, alpha) in if on >= 0.5 {
            [(7.0, 25), (3.3, 97), (1.25, 235)]
        } else {
            [(7.0, 0), (3.3, 11), (1.25, 65)]
        } {
            let mut ink = Paint::default();
            ink.set_color_rgba8(red, green, blue, alpha);
            target.stroke_path(
                &path,
                &ink,
                &Stroke {
                    width,
                    ..Stroke::default()
                },
                Transform::identity(),
                None,
            );
        }
    }
}
