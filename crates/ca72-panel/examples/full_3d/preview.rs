use super::*;
#[cfg(target_os = "linux")]
use ca72_panel::panel3d::DragController;
#[cfg(target_os = "linux")]
use ca72_panel::strip::{Bank, StripTarget};

pub(super) struct Preview {
    upper: Panel3DRenderer,
    upper_state: Panel3DState,
    pub(super) lower: Panel3DRenderer,
    pub(super) lower_state: Panel3DState,
    strip: StripRenderer,
    pub(super) pages: pages::Pages,
    under: Pixmap,
    pub(super) scene: StripScene,
    pub(super) frame: Pixmap,
    pub(super) top: i32,
    left: i32,
    #[cfg(target_os = "linux")]
    drag: DragController,
    #[cfg(target_os = "linux")]
    drag_lower: bool,
    #[cfg(target_os = "linux")]
    drag_page: bool,
}

impl Preview {
    pub(super) fn load(upper_dir: &Path, lower_dir: &Path) -> Self {
        let upper = Panel3DAssets::load_from_dir(upper_dir).expect("load upper Cycles pack");
        let layout: LowerLayout = serde_json::from_slice(
            &std::fs::read(lower_dir.join("lower.json")).expect("read lower layout"),
        )
        .expect("parse lower layout");
        let scale = f64::from(upper.manifest.face.px_per_unit);
        let top = upper.manifest.face.faceplate_rect_px[3].round() as i32;
        let face =
            Pixmap::load_png(lower_dir.join("lower-3d.png")).expect("load lower Cycles render");
        let lower = lower_assets(&upper, face, layout.cheek_w, &layout.controls);
        let left = lower.manifest.face.faceplate_rect_px[0].round() as i32;
        let lower_state = Panel3DState::from_defaults(&lower.manifest);
        assert_eq!(lower_state.values.len(), strip::KNOBS.len());
        let falloff_path = lower_dir.join("sprite-falloff.json");
        let falloff = if falloff_path.is_file() {
            serde_json::from_slice(&std::fs::read(falloff_path).expect("read studio falloff"))
                .expect("parse studio falloff")
        } else {
            [1.0, 1.0]
        };
        let lower = Panel3DRenderer::with_sprite_falloff(lower, falloff);
        let tab = Pixmap::load_png(lower_dir.join("lower-tab-lit.png")).expect("load lit 3D tab");
        let pages = pages::Pages::load(&upper, lower_dir, &tab, falloff);
        let upper_state = upper_state(&upper);
        let upper = Panel3DRenderer::new(upper);
        let rail_key_h = layout
            .controls
            .iter()
            .find(|part| part.kind == "rail_key" && part.param == "panel")
            .map_or(strip::KEY_H, |part| f64::from(part.h));
        let mut strip = StripRenderer::with_cycles_hardware(scale, tab, rail_key_h);
        strip.set_under_top((PANEL_H * scale).round() as i32);
        let (w, h) = strip::strip_size(scale);
        assert_eq!(h, lower.frame().height());
        assert_eq!(upper.frame().width(), lower.frame().width());
        let under = lower
            .frame()
            .clone_rect(IntRect::from_xywh(left, 0, w, h).expect("nonempty strip dimensions"))
            .expect("strip inside the cheeks");
        let mut frame = Pixmap::new(upper.frame().width(), top as u32 + h).expect("full panel");
        put(&mut frame, lower.frame(), 0, top);

        let mut panel = Self {
            upper,
            upper_state,
            lower,
            lower_state,
            strip,
            pages,
            under,
            scene: demo_strip(),
            frame,
            top,
            left,
            #[cfg(target_os = "linux")]
            drag: DragController::default(),
            #[cfg(target_os = "linux")]
            drag_lower: false,
            #[cfg(target_os = "linux")]
            drag_page: false,
        };
        panel.redraw();
        panel
    }

    pub(super) fn redraw(&mut self) -> Vec<FaceRect> {
        let mut damage = Vec::new();
        for r in self.upper.render_dirty(&self.upper_state) {
            if let Some(r) = r.clamp(self.frame.width() as i32, self.top) {
                copy(&mut self.frame, self.upper.frame(), r, 0, 0, 0, 0);
                damage.push(r);
            }
        }

        let mut under_changed: Option<FaceRect> = None;
        for r in self.lower.render_dirty(&self.lower_state) {
            let local = FaceRect {
                x0: r.x0 - self.left,
                x1: r.x1 - self.left,
                ..r
            };
            if let Some(local) = local.clamp(self.under.width() as i32, self.under.height() as i32)
            {
                copy(
                    &mut self.under,
                    self.lower.frame(),
                    local,
                    self.left,
                    0,
                    0,
                    0,
                );
                under_changed = Some(under_changed.map_or(local, |prior| prior.union(local)));
            }
        }
        let changed = under_changed.map(|r| [r.x0, r.y0, r.x1, r.y1]);
        // The demo field is driven by its scene values, with no running audio clock.
        let now = self.clock();
        if self.strip.render_at(&self.scene, &self.under, changed, now)
            && let Some([x0, y0, x1, y1]) = self.strip.damage()
        {
            let r = FaceRect { x0, y0, x1, y1 };
            copy(
                &mut self.frame,
                self.strip.frame(),
                r,
                0,
                0,
                self.left,
                self.top,
            );
            damage.push(FaceRect {
                x0: x0 + self.left,
                y0: y0 + self.top,
                x1: x1 + self.left,
                y1: y1 + self.top,
            });
        }
        self.pages.paint(
            &mut self.frame,
            self.strip.frame(),
            self.left,
            self.top,
            &mut damage,
        );
        damage
    }

    fn clock(&self) -> Instant {
        // A constant clock keeps this visual preview deterministic when stationary.
        static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        *START.get_or_init(Instant::now)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn press(&mut self, x: f32, y: f32) {
        self.drag_lower = y >= self.top as f32;
        self.drag_page = false;
        if self.drag_lower {
            let ly = y - self.top as f32;
            let scale = self.lower.manifest().face.px_per_unit;
            let target = self.strip.hit(
                f64::from((x - self.left as f32) / scale),
                PANEL_H + f64::from(ly / scale),
            );
            if let Some(StripTarget::Bar(target)) = target {
                self.pages.select(target, &mut self.scene.bar);
                return;
            }
            if self.pages.press(x, ly) {
                self.drag_page = true;
                return;
            }
            match target {
                Some(StripTarget::Tab(Bank::Mode, i)) => self.scene.mode = Some(i),
                Some(StripTarget::Tab(Bank::Stereo, i)) => {
                    self.scene.stereo = Some(i);
                    self.refresh_readout("double");
                }
                Some(StripTarget::Tab(Bank::Placement, i)) => self.scene.placement = Some(i),
                Some(StripTarget::Tab(Bank::Auto, _)) => {
                    self.scene.auto = !self.scene.auto;
                    self.scene.readouts[4].1 = self.scene.auto;
                }
                _ => {
                    self.drag
                        .begin_drag(self.lower.manifest(), &mut self.lower_state, x, ly);
                }
            }
        } else if self
            .drag
            .begin_drag(self.upper.manifest(), &mut self.upper_state, x, y)
            .is_some_and(|i| self.upper.manifest().controls[i].param == "power")
        {
            self.sync_power();
        }
    }

    #[cfg(target_os = "linux")]
    pub(super) fn motion(&mut self, pos: Option<(f32, f32)>, dx: f32, dy: f32, shift: bool) {
        if self.drag_page {
            self.pages
                .motion(pos.map(|(x, y)| (x, y - self.top as f32)), dx, dy, shift);
        } else if self.drag_lower {
            self.drag
                .update_drag(self.lower.manifest(), &mut self.lower_state, dx, dy, shift);
            if let Some(drag) = self.drag.active {
                let id = self.lower.manifest().controls[drag.control_idx]
                    .param
                    .clone();
                self.refresh_readout(&id);
            }
        } else {
            self.drag
                .update_drag(self.upper.manifest(), &mut self.upper_state, dx, dy, shift);
        }
    }

    #[cfg(target_os = "linux")]
    pub(super) fn release(&mut self) {
        if self.drag_page {
            self.pages.release();
            self.drag_page = false;
        } else if self.drag_lower {
            self.drag
                .end_drag(self.lower.manifest(), &mut self.lower_state);
        } else {
            self.drag
                .end_drag(self.upper.manifest(), &mut self.upper_state);
        }
    }

    #[cfg(target_os = "linux")]
    pub(super) fn scroll(&mut self, x: f32, y: f32, direction: f32, shift: bool) {
        if y >= self.top as f32 {
            let manifest = self.lower.manifest();
            let y = y - self.top as f32;
            if self.pages.scroll(x, y, direction, shift) {
                return;
            }
            if let Some(i) = DragController::hit_test(manifest, x, y) {
                let id = manifest.controls[i].param.clone();
                DragController::scroll(manifest, &mut self.lower_state, x, y, direction, shift);
                self.refresh_readout(&id);
            }
        } else {
            DragController::scroll(
                self.upper.manifest(),
                &mut self.upper_state,
                x,
                y,
                direction,
                shift,
            );
            self.sync_power();
        }
    }

    #[cfg(target_os = "linux")]
    fn sync_power(&mut self) {
        let manifest = self.upper.manifest();
        if let Some(i) = manifest.controls.iter().position(|c| c.param == "power") {
            self.upper_state
                .set_lamp(manifest, "power", self.upper_state.values[i] * 0.78);
        }
    }

    pub(super) fn refresh_readout(&mut self, id: &str) {
        let Some(i) = self
            .lower
            .manifest()
            .controls
            .iter()
            .position(|c| c.param == id)
        else {
            return;
        };
        let v = self.lower_state.values[i];
        let (i, text, lit) = match id {
            "voices" => (0, format!("{:.0}", 2.0 + 8.0 * v), true),
            "entropy" => (1, format!("{:.0}", 100.0 * v), true),
            "spread" => (2, format!("{:.0}", 100.0 * v), true),
            "double" if self.scene.stereo == Some(1) => (3, format!("{:.0}", 100.0 * v), true),
            "double" => (3, "OFF".into(), false),
            "drive" => (5, format!("{:.1}", 24.0 * v), true),
            "level" => (6, format!("{:.1}", 42.0 * v - 30.0), true),
            "inner" => (7, format!("{:.0}", 100.0 * v), true),
            _ => return,
        };
        self.scene.readouts[i] = (text, lit);
    }
}

pub(super) fn copy(
    dst: &mut Pixmap,
    src: &Pixmap,
    r: FaceRect,
    sx: i32,
    sy: i32,
    dx: i32,
    dy: i32,
) {
    let dw = dst.width() as usize * 4;
    let sw = src.width() as usize * 4;
    let n = (r.x1 - r.x0) as usize * 4;
    for y in r.y0..r.y1 {
        let from = (y + sy) as usize * sw + (r.x0 + sx) as usize * 4;
        let to = (y + dy) as usize * dw + (r.x0 + dx) as usize * 4;
        dst.data_mut()[to..to + n].copy_from_slice(&src.data()[from..from + n]);
    }
}
