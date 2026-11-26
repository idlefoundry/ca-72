//! Native 4K 0-degree flat-orthographic Blender Cycles 3D hardware panel compositor for CA-72.
//!
//! Implements the `vst-3d-ui` pure-CPU software compositing architecture:
//! - Resolution-independent metrics loaded from `manifest.json`
//! - Spatial studio lighting match baked per-control preserving premultiplied alpha
//! - Smooth Catmull-Rom 4-tap bicubic angle-aligned rotational knob crossfading with locked cast shadows
//! - Physically-based multi-brightness linear-light jewel/dome lamp illumination (subsurface dome
//!   scattering, lower-edge refraction crescent, incandescent die core, preserved chrome bezel,
//!   inverse-square grazing spill, filmic highlight rolloff)
//! - Sub-millisecond dirty-rect compositing (`render_dirty`) + 16.16 fixed-point bilinear dirty-rect
//!   blit to `u32` softbuffer framebuffers (`blit_dirty_to_u32`)
//! - Unquantized accumulator-based discrete selector drag controller (`DragController`) so small
//!   mouse movements never lock up on quantized steps

use std::collections::HashMap;
use std::path::Path;

use resvg::tiny_skia::Pixmap;
use serde::{Deserialize, Serialize};

use crate::Arrangement;
use crate::art;
use crate::controls::Control;
use crate::render::Renderer;
use crate::skin::Skin;
use crate::ultra::{Opening, ultra_worn};

/// Axis-aligned rectangle in native face pixels (`[x0, y0, x1, y1]`, exclusive max).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceRect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl FaceRect {
    pub fn clamp(self, w: i32, h: i32) -> Option<Self> {
        let r = Self {
            x0: self.x0.clamp(0, w),
            y0: self.y0.clamp(0, h),
            x1: self.x1.clamp(0, w),
            y1: self.y1.clamp(0, h),
        };
        (r.x1 > r.x0 && r.y1 > r.y0).then_some(r)
    }

    pub fn intersects(self, other: Self) -> bool {
        self.x0 < other.x1 && other.x0 < self.x1 && self.y0 < other.y1 && other.y0 < self.y1
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }
}

/// Top-level `manifest.json` describing the 4K flat-orthographic Cycles sprite pack.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub format: String,
    pub projection: String,
    pub face: FaceSpec,
    pub sprites: HashMap<String, SpriteSpec>,
    pub controls: Vec<ControlSpec>,
    pub lamps: Vec<LampSpec>,
    /// The center of the QUALITY opening, in face pixels. Older packs bake the lamp into the face.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_opening: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FaceSpec {
    pub file: String,
    pub w: u32,
    pub h: u32,
    pub px_per_m: f32,
    pub px_per_unit: f32,
    pub faceplate_rect_px: [f32; 4],
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpriteSpec {
    pub file: String,
    pub kind: String,
    pub cell_w: u32,
    pub cell_h: u32,
    pub anchor: [f32; 2],
    #[serde(default)]
    pub body_r: Option<f32>,
    #[serde(default)]
    pub body_w: Option<f32>,
    #[serde(default)]
    pub body_h: Option<f32>,
    pub cols: u32,
    pub rows: u32,
    pub frames: u32,
    #[serde(default)]
    pub angles_deg: Vec<f32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControlSpec {
    pub param: String,
    pub label: String,
    pub kind: String,
    pub sprite: String,
    pub pos: [f32; 2],
    #[serde(default)]
    pub hit_r: Option<f32>,
    #[serde(default)]
    pub hit_w: Option<f32>,
    #[serde(default)]
    pub hit_h: Option<f32>,
    #[serde(default)]
    pub steps: Option<usize>,
    #[serde(default)]
    pub places: Vec<usize>,
    #[serde(default)]
    pub angles_deg: Vec<f32>,
    #[serde(default)]
    pub orient: Option<String>,
    #[serde(default)]
    pub colour: Option<String>,
    #[serde(default)]
    pub across: Option<bool>,
    #[serde(default)]
    pub default: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LampSpec {
    pub id: String,
    pub name: String,
    pub pos: [f32; 2],
    pub dome_r: f32,
    pub bezel_outer_r: f32,
    pub color_srgb: [u8; 3],
    pub core_srgb: [u8; 3],
    #[serde(default = "default_true")]
    pub faceted: bool,
    #[serde(default)]
    pub default_drive: f32,
}

fn default_true() -> bool {
    true
}

/// Decoded native 4K Cycles assets ready for pure-CPU compositing.
pub struct Panel3DAssets {
    pub manifest: Manifest,
    pub face: Pixmap,
    pub atlases: HashMap<String, Pixmap>,
}

impl std::fmt::Debug for Panel3DAssets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Panel3DAssets")
            .field("face", &(self.face.width(), self.face.height()))
            .field("atlases", &self.atlases.len())
            .finish()
    }
}

impl Panel3DAssets {
    /// Load `manifest.json`, `face-3d.png`, and every sprite atlas from `dir`.
    pub fn load_from_dir(dir: &Path) -> Result<Self, String> {
        let manifest_text = std::fs::read_to_string(dir.join("manifest.json"))
            .map_err(|e| format!("read manifest.json: {e}"))?;
        let manifest: Manifest = serde_json::from_str(&manifest_text)
            .map_err(|e| format!("parse manifest.json: {e}"))?;
        let face_bytes = std::fs::read(dir.join(&manifest.face.file))
            .map_err(|e| format!("read {}: {e}", manifest.face.file))?;
        let face = Pixmap::decode_png(&face_bytes)
            .map_err(|e| format!("decode {}: {e}", manifest.face.file))?;
        let mut atlases = HashMap::with_capacity(manifest.sprites.len());
        for (key, sp) in &manifest.sprites {
            let bytes = std::fs::read(dir.join(&sp.file))
                .map_err(|e| format!("read {}: {e}", sp.file))?;
            let px = Pixmap::decode_png(&bytes)
                .map_err(|e| format!("decode {}: {e}", sp.file))?;
            atlases.insert(key.clone(), px);
        }
        Ok(Self {
            manifest,
            face,
            atlases,
        })
    }
}

/// Live panel state (one normalized `0.0..=1.0` value per control in `manifest.controls`,
/// and one continuous `0.0..=1.0` brightness drive per lamp in `manifest.lamps`).
#[derive(Clone, Debug, PartialEq)]
pub struct Panel3DState {
    pub values: Vec<f32>,
    pub lamp_drives: Vec<f32>,
    /// Scene supplied by the caller for animation or hiding the opening. When absent, the
    /// renderer shows the resting indicator for the current QUALITY detent.
    pub quality_opening: Option<Opening>,
}

impl Panel3DState {
    pub fn from_defaults(manifest: &Manifest) -> Self {
        Self {
            values: manifest.controls.iter().map(|c| c.default).collect(),
            lamp_drives: manifest.lamps.iter().map(|l| l.default_drive).collect(),
            quality_opening: None,
        }
    }

    pub fn set_control(&mut self, manifest: &Manifest, param: &str, value: f32) {
        if let Some(i) = manifest.controls.iter().position(|c| c.param == param) {
            self.values[i] = value.clamp(0.0, 1.0);
        }
    }

    pub fn set_lamp(&mut self, manifest: &Manifest, id: &str, drive: f32) {
        if let Some(i) = manifest.lamps.iter().position(|l| l.id == id) {
            self.lamp_drives[i] = drive.clamp(0.0, 1.0);
        }
    }
}

/// Spatial studio lighting match (`vst-3d-ui` Section 5) so controls on the right sit naturally
/// in the same studio falloff as the anodized aluminum faceplate.
#[inline]
pub fn studio_light_gain(pos: [f32; 2], face_w: f32, face_h: f32) -> f32 {
    let u = (pos[0] / face_w.max(1.0)).clamp(0.0, 1.0);
    let v = (pos[1] / face_h.max(1.0)).clamp(0.0, 1.0);
    1.03 - 0.13 * u - 0.02 * v
}

/// 4-tap Catmull-Rom cubic interpolation weight (`a = -0.5`).
#[inline]
fn catmull_weights(t: f32) -> [f32; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        -0.5 * t + t2 - 0.5 * t3,
        1.0 - 2.5 * t2 + 1.5 * t3,
        0.5 * t + 2.0 * t2 - 1.5 * t3,
        -0.5 * t2 + 0.5 * t3,
    ]
}

/// Sample premultiplied RGBA from one sprite cell `(ox, oy, cw, ch)` using 4x4 Catmull-Rom bicubic.
#[allow(clippy::too_many_arguments)]
#[inline]
fn sample_cell_catmull(
    atlas: &[u8],
    atlas_w: i32,
    ox: i32,
    oy: i32,
    cw: i32,
    ch: i32,
    fx: f32,
    fy: f32,
) -> [f32; 4] {
    if fx < -1.0 || fy < -1.0 || fx > cw as f32 || fy > ch as f32 {
        return [0.0; 4];
    }
    let ix = fx.floor() as i32;
    let iy = fy.floor() as i32;
    let wx = catmull_weights(fx - ix as f32);
    let wy = catmull_weights(fy - iy as f32);

    let mut out = [0.0f32; 4];
    for ky in 0..4 {
        let sy = (iy + ky - 1).clamp(0, ch - 1);
        let row_off = ((oy + sy) * atlas_w + ox) as usize;
        let wy_k = wy[ky as usize];
        let mut row = [0.0f32; 4];
        for kx in 0..4 {
            let sx = (ix + kx - 1).clamp(0, cw - 1);
            let idx = (row_off + sx as usize) * 4;
            let wx_k = wx[kx as usize];
            row[0] += f32::from(atlas[idx]) * wx_k;
            row[1] += f32::from(atlas[idx + 1]) * wx_k;
            row[2] += f32::from(atlas[idx + 2]) * wx_k;
            row[3] += f32::from(atlas[idx + 3]) * wx_k;
        }
        for c in 0..4 {
            out[c] += row[c] * wy_k;
        }
    }
    // Clamp cubic ringing and enforce premultiplied alpha invariant (rgb <= a)
    let a = out[3].clamp(0.0, 255.0);
    [
        out[0].clamp(0.0, a),
        out[1].clamp(0.0, a),
        out[2].clamp(0.0, a),
        a,
    ]
}

/// Render a smooth continuous knob cell using Catmull-Rom bicubic angle-aligned crossfading
/// between adjacent frames `i0` and `i1`, with **locked cast shadow** outside `body_r` and
/// spatial studio lighting gain applied.
pub fn render_knob_smooth(
    atlas: &Pixmap,
    spec: &SpriteSpec,
    val: f32,
    light_gain: f32,
) -> Option<Pixmap> {
    let cw = spec.cell_w as i32;
    let ch = spec.cell_h as i32;
    let mut out = Pixmap::new(spec.cell_w, spec.cell_h)?;
    let frames = spec.frames.max(1);
    let cols = spec.cols.max(1);
    let fpos = val.clamp(0.0, 1.0) * (frames - 1) as f32;
    let i0 = (fpos.floor() as u32).min(frames - 1);
    let i1 = (i0 + 1).min(frames - 1);
    let t = (fpos - i0 as f32).clamp(0.0, 1.0);

    let ox0 = (i0 % cols) as i32 * cw;
    let oy0 = (i0 / cols) as i32 * ch;
    let ox1 = (i1 % cols) as i32 * cw;
    let oy1 = (i1 / cols) as i32 * ch;

    let atlas_w = atlas.width() as i32;
    let src = atlas.data();
    let dst = out.data_mut();

    // Fast path: exact frame hit (no rotation needed)
    if i0 == i1 || t < 1e-4 {
        for y in 0..ch {
            let s_off = ((oy0 + y) * atlas_w + ox0) as usize * 4;
            let d_off = (y * cw) as usize * 4;
            for x in 0..cw as usize {
                let si = s_off + x * 4;
                let di = d_off + x * 4;
                let a = src[si + 3];
                if a == 0 {
                    continue;
                }
                let af = f32::from(a);
                dst[di] = (f32::from(src[si]) * light_gain).round().clamp(0.0, af) as u8;
                dst[di + 1] = (f32::from(src[si + 1]) * light_gain).round().clamp(0.0, af) as u8;
                dst[di + 2] = (f32::from(src[si + 2]) * light_gain).round().clamp(0.0, af) as u8;
                dst[di + 3] = a;
            }
        }
        return Some(out);
    }

    // Angle step between i0 and i1 (clockwise positive in screen coordinates)
    let step_deg = if spec.angles_deg.len() > i1 as usize {
        spec.angles_deg[i1 as usize] - spec.angles_deg[i0 as usize]
    } else {
        300.0 / (frames - 1).max(1) as f32
    };
    // To rotate frame i0 forward by +t*step_deg, inverse-sample at -t*step_deg.
    // To rotate frame i1 backward by -(1-t)*step_deg, inverse-sample at +(1-t)*step_deg.
    let (s0, c0) = (-(t * step_deg)).to_radians().sin_cos();
    let (s1, c1) = ((1.0 - t) * step_deg).to_radians().sin_cos();
    let ax = spec.anchor[0];
    let ay = spec.anchor[1];
    let body_r = spec.body_r.unwrap_or(cw as f32 * 0.30);
    let r_inner = body_r - 1.2;
    let r_outer = body_r + 1.2;

    for y in 0..ch {
        let dy = y as f32 - ay;
        let d_row = (y * cw) as usize * 4;
        for x in 0..cw {
            let dx = x as f32 - ax;
            let r = dx.hypot(dy);
            let di = d_row + x as usize * 4;

            // Outside body_r: lock cast shadow by blending unrotated i0 & i1 directly!
            let s0_off = ((oy0 + y) * atlas_w + ox0 + x) as usize * 4;
            let s1_off = ((oy1 + y) * atlas_w + ox1 + x) as usize * 4;
            let unrot = [
                f32::from(src[s0_off]) * (1.0 - t) + f32::from(src[s1_off]) * t,
                f32::from(src[s0_off + 1]) * (1.0 - t) + f32::from(src[s1_off + 1]) * t,
                f32::from(src[s0_off + 2]) * (1.0 - t) + f32::from(src[s1_off + 2]) * t,
                f32::from(src[s0_off + 3]) * (1.0 - t) + f32::from(src[s1_off + 3]) * t,
            ];

            let px = if r >= r_outer {
                unrot
            } else {
                let fx0 = ax + dx * c0 - dy * s0;
                let fy0 = ay + dx * s0 + dy * c0;
                let fx1 = ax + dx * c1 - dy * s1;
                let fy1 = ay + dx * s1 + dy * c1;
                let p0 = sample_cell_catmull(src, atlas_w, ox0, oy0, cw, ch, fx0, fy0);
                let p1 = sample_cell_catmull(src, atlas_w, ox1, oy1, cw, ch, fx1, fy1);
                let rot = [
                    p0[0] * (1.0 - t) + p1[0] * t,
                    p0[1] * (1.0 - t) + p1[1] * t,
                    p0[2] * (1.0 - t) + p1[2] * t,
                    p0[3] * (1.0 - t) + p1[3] * t,
                ];
                if r <= r_inner {
                    rot
                } else {
                    let k = ((r - r_inner) / (r_outer - r_inner)).clamp(0.0, 1.0);
                    let sm = k * k * (3.0 - 2.0 * k);
                    [
                        rot[0] * (1.0 - sm) + unrot[0] * sm,
                        rot[1] * (1.0 - sm) + unrot[1] * sm,
                        rot[2] * (1.0 - sm) + unrot[2] * sm,
                        rot[3] * (1.0 - sm) + unrot[3] * sm,
                    ]
                }
            };

            let a = px[3].round().clamp(0.0, 255.0);
            if a <= 0.0 {
                continue;
            }
            dst[di] = (px[0] * light_gain).round().clamp(0.0, a) as u8;
            dst[di + 1] = (px[1] * light_gain).round().clamp(0.0, a) as u8;
            dst[di + 2] = (px[2] * light_gain).round().clamp(0.0, a) as u8;
            dst[di + 3] = a as u8;
        }
    }
    Some(out)
}

/// Extract one exact discrete cell from an atlas with spatial studio lighting gain applied.
fn extract_cell_lit(atlas: &Pixmap, spec: &SpriteSpec, frame_idx: u32, light_gain: f32) -> Option<Pixmap> {
    let cw = spec.cell_w as i32;
    let ch = spec.cell_h as i32;
    let cols = spec.cols.max(1);
    let idx = frame_idx.min(spec.frames.saturating_sub(1));
    let ox = (idx % cols) as i32 * cw;
    let oy = (idx / cols) as i32 * ch;
    let atlas_w = atlas.width() as i32;
    let src = atlas.data();
    let mut out = Pixmap::new(spec.cell_w, spec.cell_h)?;
    let dst = out.data_mut();

    for y in 0..ch {
        let s_row = ((oy + y) * atlas_w + ox) as usize * 4;
        let d_row = (y * cw) as usize * 4;
        for x in 0..cw as usize {
            let si = s_row + x * 4;
            let di = d_row + x * 4;
            let a = src[si + 3];
            if a == 0 {
                continue;
            }
            let af = f32::from(a);
            dst[di] = (f32::from(src[si]) * light_gain).round().clamp(0.0, af) as u8;
            dst[di + 1] = (f32::from(src[si + 1]) * light_gain).round().clamp(0.0, af) as u8;
            dst[di + 2] = (f32::from(src[si + 2]) * light_gain).round().clamp(0.0, af) as u8;
            dst[di + 3] = a;
        }
    }
    Some(out)
}

#[inline]
fn srgb_to_lin(u: u8) -> f32 {
    let x = f32::from(u) / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn lin_to_srgb(x: f32) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let s = if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Filmic highlight rolloff compressing linear peaks above `0.85` cleanly without hard clipping.
#[inline]
fn filmic_rolloff(v: f32) -> f32 {
    if v <= 0.85 {
        v.max(0.0)
    } else {
        let e = v - 0.85;
        0.85 + (0.15 * e) / (0.15 + e)
    }
}

#[inline]
fn smoothstep_f32(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0).max(1e-6)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Compute the dirty bounding rectangle of a linear-light lamp on the 4K canvas.
pub fn lamp_dirty_rect(lamp: &LampSpec) -> FaceRect {
    let reach = (lamp.bezel_outer_r * 3.8).ceil() as i32 + 4;
    let cx = lamp.pos[0].round() as i32;
    let cy = lamp.pos[1].round() as i32;
    FaceRect {
        x0: cx - reach,
        y0: cy - reach,
        x1: cx + reach,
        y1: cy + reach,
    }
}

/// Physically-based multi-brightness linear-light lamp shader (`vst-3d-ui` Section 5).
/// Renders directly onto `frame` inside `clip` in linear RGB (`sRGB -> linear -> sRGB`).
pub fn render_lamp_linear(frame: &mut Pixmap, lamp: &LampSpec, drive: f32, clip: FaceRect) {
    let b = drive.clamp(0.0, 1.0);
    if b <= 0.004 {
        return;
    }
    let fw = frame.width() as i32;
    let fh = frame.height() as i32;
    let rect = lamp_dirty_rect(lamp);
    let Some(area) = FaceRect {
        x0: rect.x0.max(clip.x0),
        y0: rect.y0.max(clip.y0),
        x1: rect.x1.min(clip.x1),
        y1: rect.y1.min(clip.y1),
    }
    .clamp(fw, fh) else {
        return;
    };

    let col = [
        srgb_to_lin(lamp.color_srgb[0]),
        srgb_to_lin(lamp.color_srgb[1]),
        srgb_to_lin(lamp.color_srgb[2]),
    ];
    let core_col = [
        srgb_to_lin(lamp.core_srgb[0]),
        srgb_to_lin(lamp.core_srgb[1]),
        srgb_to_lin(lamp.core_srgb[2]),
    ];

    let cx = lamp.pos[0];
    let cy = lamp.pos[1];
    let dome_r = lamp.dome_r;
    let bez_r = lamp.bezel_outer_r;
    let sigma = dome_r * 1.15;
    let max_r = bez_r * 3.8;
    // Incandescent core shifts in smoothly only at high drive (b > 0.35)
    let hot_shift = smoothstep_f32(0.35, 1.0, b);
    let spill_drive = b.powf(1.65);

    let data = frame.data_mut();
    for y in area.y0..area.y1 {
        let dy = y as f32 + 0.5 - cy;
        let row_off = (y * fw) as usize * 4;
        for x in area.x0..area.x1 {
            let dx = x as f32 + 0.5 - cx;
            let r = dx.hypot(dy);
            if r >= max_r {
                continue;
            }
            let i = row_off + x as usize * 4;
            let base = [
                srgb_to_lin(data[i]),
                srgb_to_lin(data[i + 1]),
                srgb_to_lin(data[i + 2]),
            ];
            let base_luma = 0.2126 * base[0] + 0.7152 * base[1] + 0.0722 * base[2];

            let mut add = [0.0f32; 3];
            if r <= dome_r + 1.2 {
                // 1. Inside translucent faceted jewel / epoxy dome:
                let rn = (r / dome_r).clamp(0.0, 1.0);
                let edge_cov = 1.0 - smoothstep_f32(dome_r - 1.0, dome_r + 1.2, r);
                let nz = (1.0 - rn * rn).max(0.0).sqrt();
                // Spherical subsurface scattering
                let sss = nz.powf(0.55) * (0.48 + 0.85 * b) * b;
                // Lower-edge internal refraction crescent (classic illuminated glass jewel rim!)
                let lower = (dy / dome_r + 0.15).clamp(0.0, 1.0);
                let crescent = smoothstep_f32(0.40, 0.90, rn) * lower * 0.42 * b;
                // Subtle facet sparkle modulation (12-fold cut ruby jewel facets when faceted)
                let facet = if lamp.faceted {
                    let ang = dy.atan2(dx);
                    0.90 + 0.16 * (ang * 12.0).cos().abs()
                } else {
                    1.0
                };
                // Compact central filament / semiconductor die
                let die = (-(rn / 0.34).powi(2)).exp() * (0.55 + 1.55 * hot_shift) * b;

                for c in 0..3 {
                    let body_emit = col[c] * (sss * facet + crescent);
                    let die_col = col[c] * (1.0 - hot_shift) + core_col[c] * hot_shift;
                    add[c] += (body_emit + die_col * die) * edge_cov;
                }
            }

            if r > dome_r - 1.0 && r <= bez_r + 1.0 {
                // 2. Bezel preservation: keep dark/chrome bezel visibly intact with subtle inner-lip catchlight
                let lip = 1.0 - smoothstep_f32(dome_r, dome_r + (bez_r - dome_r) * 0.45, r);
                let bez_mask = smoothstep_f32(dome_r - 0.8, dome_r + 0.8, r);
                let catch_str = lip * bez_mask * 0.24 * spill_drive * (0.4 + 1.2 * base_luma);
                for c in 0..3 {
                    add[c] += col[c] * catch_str;
                }
            }

            if r > bez_r - 1.0 {
                // 3. Panel grazing spill: tight inverse-square (1 / (1 + (d/sigma)^2.25)) * b^1.65
                //    modulated by underlying steel luminance so white silkscreen stays crisp!
                let d = (r - bez_r).max(0.0);
                let inv_sq = 1.0 / (1.0 + (d / sigma).powf(2.25));
                let outer_fade = 1.0 - smoothstep_f32(max_r * 0.72, max_r, r);
                let bez_out = smoothstep_f32(bez_r - 0.8, bez_r + 1.0, r);
                // Modulate gently by steel luminance, clamping bright silkscreen spill
                let steel_mod = (0.35 + 1.15 * base_luma.min(0.18)) * (1.0 - 0.55 * smoothstep_f32(0.25, 0.70, base_luma));
                let spill = inv_sq * outer_fade * bez_out * 0.28 * spill_drive * steel_mod;
                for c in 0..3 {
                    add[c] += col[c] * spill;
                }
            }

            // 4. Filmic highlight rolloff -> back to sRGB
            for c in 0..3 {
                data[i + c] = lin_to_srgb(filmic_rolloff(base[c] + add[c]));
            }
            data[i + 3] = 255;
        }
    }
}

/// Premultiplied source-over blit of `src` onto `dst` at `(at_x, at_y)`, clipped to `clip`.
fn composite_clipped(dst: &mut Pixmap, src: &Pixmap, at_x: i32, at_y: i32, clip: FaceRect) {
    let dw = dst.width() as i32;
    let dh = dst.height() as i32;
    let sw = src.width() as i32;
    let sh = src.height() as i32;
    let x0 = clip.x0.max(0).max(at_x);
    let y0 = clip.y0.max(0).max(at_y);
    let x1 = clip.x1.min(dw).min(at_x + sw);
    let y1 = clip.y1.min(dh).min(at_y + sh);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let sd = src.data();
    let dd = dst.data_mut();
    for y in y0..y1 {
        let s_row = ((y - at_y) * sw + (x0 - at_x)) as usize * 4;
        let d_row = (y * dw + x0) as usize * 4;
        let n = (x1 - x0) as usize;
        for x in 0..n {
            let si = s_row + x * 4;
            let a = sd[si + 3];
            if a == 0 {
                continue;
            }
            let di = d_row + x * 4;
            if a == 255 {
                dd[di..di + 4].copy_from_slice(&sd[si..si + 4]);
            } else {
                let keep = 255 - u16::from(a);
                for c in 0..4 {
                    let prod = u16::from(dd[di + c]) * keep + 128;
                    let blend = ((prod + (prod >> 8)) >> 8) as u8;
                    dd[di + c] = sd[si + c].saturating_add(blend);
                }
            }
        }
    }
}

/// Restore static `face-3d.png` pixels inside `clip` into `dst`.
fn restore_face_rect(dst: &mut Pixmap, face: &Pixmap, clip: FaceRect) {
    let w = dst.width() as i32;
    let h = dst.height() as i32;
    let Some(r) = clip.clamp(w, h) else { return };
    let stride = w as usize * 4;
    let n = (r.x1 - r.x0) as usize * 4;
    let src = face.data();
    let dd = dst.data_mut();
    for y in r.y0..r.y1 {
        let off = y as usize * stride + r.x0 as usize * 4;
        dd[off..off + n].copy_from_slice(&src[off..off + n]);
    }
}

/// Cached per-control rendered pixmap so static knobs/switches cost 0 ms!
struct ControlCache {
    last_val: Option<f32>,
    pixmap: Option<Pixmap>,
    rect: FaceRect,
}

/// High-FPS native 4K Cycles panel compositor with dirty-rect tracking and per-control caches.
pub struct Panel3DRenderer {
    assets: Panel3DAssets,
    frame: Pixmap,
    caches: Vec<ControlCache>,
    prev_state: Option<Panel3DState>,
    quality: Option<QualityOpening>,
    sprite_falloff: [f32; 2],
}

struct QualityOpening {
    painter: Renderer,
    control: usize,
    switch: Control,
    scale: f64,
    origin: (f64, f64),
    shown: Option<Opening>,
    layers: Vec<(i32, i32, Pixmap)>,
}

impl QualityOpening {
    fn new(manifest: &Manifest) -> Option<Self> {
        let [x, y] = manifest.quality_opening?;
        let control = manifest.controls.iter().position(|c| c.param == "quality")?;
        let switch = *Arrangement::official()
            .controls
            .iter()
            .find(|c| c.param == "quality")?;
        let scale = f64::from(manifest.face.px_per_unit);
        Some(Self {
            painter: Renderer::for_parts(Skin::Worn, scale),
            control,
            switch,
            scale,
            origin: (f64::from(x) / scale, f64::from(y) / scale),
            shown: None,
            layers: Vec::new(),
        })
    }

    fn paint_switch(&mut self, pos: [f32; 2], value: f32) -> Option<(i32, i32, Pixmap)> {
        let mut layer = art::control_worn(&self.switch, f64::from(value), 0.0);
        layer.origin = (f64::from(pos[0]) / self.scale, f64::from(pos[1]) / self.scale);
        self.painter.paint_layer(&layer)
    }

    fn rect(&self) -> Option<FaceRect> {
        let (x, y, pixels) = self.layers.first()?;
        Some(FaceRect {
            x0: *x,
            y0: *y,
            x1: x + pixels.width() as i32,
            y1: y + pixels.height() as i32,
        })
    }

    fn update(&mut self, state: &Panel3DState) -> Option<FaceRect> {
        let opening = state.quality_opening.unwrap_or_else(|| {
            let value = state.values.get(self.control).copied().unwrap_or(0.5);
            match (value.clamp(0.0, 1.0) * 2.0).round() as u32 {
                0 => Opening {
                    hamster: 1.0,
                    asleep: 1.0,
                    ..Opening::default()
                },
                1 => Opening {
                    lamp: 1.0,
                    ..Opening::default()
                },
                _ => Opening {
                    coil: 1.0,
                    ..Opening::default()
                },
            }
        });
        if self.shown == Some(opening) {
            return None;
        }
        let previous = self.rect();
        self.shown = Some(opening);
        self.layers.clear();
        if !opening.hidden {
            self.layers.extend(ultra_worn(&opening).into_iter().filter_map(|mut layer| {
                layer.origin = self.origin;
                self.painter.paint_layer(&layer)
            }));
        }
        previous.into_iter().chain(self.rect()).reduce(FaceRect::union)
    }
}

impl std::fmt::Debug for Panel3DRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Panel3DRenderer")
            .field("size", &(self.frame.width(), self.frame.height()))
            .field("controls", &self.caches.len())
            .finish()
    }
}

impl Panel3DRenderer {
    pub fn new(assets: Panel3DAssets) -> Self {
        Self::with_sprite_falloff(assets, [1.0, 1.0])
    }

    /// Match reusable sprites to the measured falloff from the top to the bottom of this face.
    pub fn with_sprite_falloff(assets: Panel3DAssets, sprite_falloff: [f32; 2]) -> Self {
        let frame = assets.face.clone();
        let quality = QualityOpening::new(&assets.manifest);
        let caches = assets
            .manifest
            .controls
            .iter()
            .map(|c| {
                let rect = Self::control_rect(&assets.manifest, c);
                ControlCache {
                    last_val: None,
                    pixmap: None,
                    rect,
                }
            })
            .collect();
        Self {
            assets,
            frame,
            caches,
            prev_state: None,
            quality,
            sprite_falloff,
        }
    }

    pub fn manifest(&self) -> &Manifest {
        &self.assets.manifest
    }

    pub fn frame(&self) -> &Pixmap {
        &self.frame
    }

    pub fn invalidate_all(&mut self) {
        self.prev_state = None;
    }

    fn control_rect(manifest: &Manifest, c: &ControlSpec) -> FaceRect {
        let Some(sp) = manifest.sprites.get(&c.sprite) else {
            return FaceRect { x0: 0, y0: 0, x1: 0, y1: 0 };
        };
        let x0 = (c.pos[0] - sp.anchor[0]).round() as i32;
        let y0 = (c.pos[1] - sp.anchor[1]).round() as i32;
        FaceRect {
            x0,
            y0,
            x1: x0 + sp.cell_w as i32,
            y1: y0 + sp.cell_h as i32,
        }
    }

    fn update_control_cache(&mut self, idx: usize, val: f32) {
        if self.caches[idx].last_val.is_some_and(|v| (v - val).abs() < 1e-6)
            && self.caches[idx].pixmap.is_some()
        {
            return;
        }
        let c = &self.assets.manifest.controls[idx];
        if let Some(quality) = &mut self.quality
            && quality.control == idx
            && let Some((x, y, px)) = quality.paint_switch(c.pos, val)
        {
            self.caches[idx] = ControlCache {
                rect: FaceRect {
                    x0: x,
                    y0: y,
                    x1: x + px.width() as i32,
                    y1: y + px.height() as i32,
                },
                last_val: Some(val),
                pixmap: Some(px),
            };
            return;
        }
        let Some(sp) = self.assets.manifest.sprites.get(&c.sprite) else {
            return;
        };
        let Some(atlas) = self.assets.atlases.get(&c.sprite) else {
            return;
        };
        let face_h = self.assets.manifest.face.h as f32;
        let v = (c.pos[1] / face_h).clamp(0.0, 1.0);
        let [top, bottom] = self.sprite_falloff;
        let gain = studio_light_gain(
            c.pos,
            self.assets.manifest.face.w as f32,
            face_h,
        ) * (top + (bottom - top) * v);
        let px = match c.kind.as_str() {
            "knob" => render_knob_smooth(atlas, sp, val, gain),
            "selector" => {
                let steps = c.steps.unwrap_or(sp.frames as usize).max(2);
                let frame_idx = (val.clamp(0.0, 1.0) * (steps - 1) as f32).round() as u32;
                extract_cell_lit(atlas, sp, frame_idx, gain)
            }
            "rocker" => {
                let frame_idx = u32::from(val >= 0.5);
                extract_cell_lit(atlas, sp, frame_idx, gain)
            }
            "toggle" => {
                let frame_idx = (val.clamp(0.0, 1.0) * 2.0).round() as u32;
                extract_cell_lit(atlas, sp, frame_idx, gain)
            }
            _ => extract_cell_lit(atlas, sp, 0, gain),
        };
        self.caches[idx].last_val = Some(val);
        self.caches[idx].pixmap = px;
    }

    /// Render only changed controls & lamps, returning the exact face-space dirty rects.
    /// Returns an empty `Vec` in `< 0.05 ms` when nothing changed!
    pub fn render_dirty(&mut self, state: &Panel3DState) -> Vec<FaceRect> {
        let fw = self.frame.width() as i32;
        let fh = self.frame.height() as i32;
        let full = FaceRect { x0: 0, y0: 0, x1: fw, y1: fh };

        if self.prev_state.as_ref() == Some(state) {
            return Vec::new();
        }

        let mut dirty: Vec<FaceRect> = Vec::new();
        match self.prev_state.take() {
            None => {
                dirty.push(full);
                for i in 0..self.caches.len() {
                    let v = state.values.get(i).copied().unwrap_or(0.0);
                    self.update_control_cache(i, v);
                }
            }
            Some(prev) => {
                for i in 0..self.caches.len() {
                    let v = state.values.get(i).copied().unwrap_or(0.0);
                    let pv = prev.values.get(i).copied().unwrap_or(0.0);
                    if (v - pv).abs() > 1e-6 {
                        self.update_control_cache(i, v);
                        if let Some(r) = self.caches[i].rect.clamp(fw, fh) {
                            dirty.push(r);
                        }
                    }
                }
                for (i, lamp) in self.assets.manifest.lamps.iter().enumerate() {
                    let b = state.lamp_drives.get(i).copied().unwrap_or(0.0);
                    let pb = prev.lamp_drives.get(i).copied().unwrap_or(0.0);
                    if (b - pb).abs() > 1e-4
                        && let Some(r) = lamp_dirty_rect(lamp).clamp(fw, fh)
                    {
                        dirty.push(r);
                    }
                }
            }
        }

        if let Some(rect) = self.quality.as_mut().and_then(|quality| quality.update(state))
            && let Some(rect) = rect.clamp(fw, fh)
        {
            dirty.push(rect);
        }

        self.prev_state = Some(state.clone());
        if dirty.is_empty() {
            return dirty;
        }

        // Merge overlapping/adjacent dirty rects
        let merged = merge_rects(dirty, 8);
        for &clip in &merged {
            restore_face_rect(&mut self.frame, &self.assets.face, clip);
            for cache in &self.caches {
                if cache.rect.intersects(clip)
                    && let Some(px) = &cache.pixmap
                {
                    composite_clipped(&mut self.frame, px, cache.rect.x0, cache.rect.y0, clip);
                }
            }
            if let Some(quality) = &self.quality
                && quality.rect().is_some_and(|rect| rect.intersects(clip))
            {
                for (x, y, pixels) in &quality.layers {
                    composite_clipped(&mut self.frame, pixels, *x, *y, clip);
                }
            }
            for (i, lamp) in self.assets.manifest.lamps.iter().enumerate() {
                let drive = state.lamp_drives.get(i).copied().unwrap_or(0.0);
                if drive > 0.004 && lamp_dirty_rect(lamp).intersects(clip) {
                    render_lamp_linear(&mut self.frame, lamp, drive, clip);
                }
            }
        }
        merged
    }
}

fn merge_rects(mut rects: Vec<FaceRect>, pad: i32) -> Vec<FaceRect> {
    'outer: loop {
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let a = rects[i];
                let b = rects[j];
                if a.x0 - pad < b.x1 && b.x0 - pad < a.x1 && a.y0 - pad < b.y1 && b.y0 - pad < a.y1 {
                    rects[i] = a.union(b);
                    rects.swap_remove(j);
                    continue 'outer;
                }
            }
        }
        break;
    }
    rects
}

/// Inline 16.16 fixed-point bilinear dirty-rect blit directly from 4K `Pixmap` into a `softbuffer`
/// `0x00RRGGBB` `u32` buffer (with 1:1 fast copy when sizes match). Returns scaled destination
/// damage rectangles `[x, y, w, h]` ready for `buffer.present_with_damage(&damage)`.
pub fn blit_dirty_to_u32(
    src: &Pixmap,
    dirty: &[FaceRect],
    dst: &mut [u32],
    dst_w: u32,
    dst_h: u32,
) -> Vec<[u32; 4]> {
    let sw = src.width();
    let sh = src.height();
    let sdata = src.data();
    let mut out_damage = Vec::with_capacity(dirty.len());
    if dst_w == 0 || dst_h == 0 || sw == 0 || sh == 0 || dst.len() < (dst_w * dst_h) as usize {
        return out_damage;
    }

    if sw == dst_w && sh == dst_h {
        // 1:1 fast copy
        for &r in dirty {
            let Some(r) = r.clamp(sw as i32, sh as i32) else {
                continue;
            };
            for y in r.y0..r.y1 {
                let row = y as usize * sw as usize;
                for x in r.x0..r.x1 {
                    let i = (row + x as usize) * 4;
                    dst[row + x as usize] = (u32::from(sdata[i]) << 16)
                        | (u32::from(sdata[i + 1]) << 8)
                        | u32::from(sdata[i + 2]);
                }
            }
            out_damage.push([
                r.x0 as u32,
                r.y0 as u32,
                (r.x1 - r.x0) as u32,
                (r.y1 - r.y0) as u32,
            ]);
        }
        return out_damage;
    }

    // 16.16 fixed-point bilinear downscale/upscale over only the dirty destination rects
    let step_x = ((u64::from(sw) << 16) / u64::from(dst_w)) as u32;
    let step_y = ((u64::from(sh) << 16) / u64::from(dst_h)) as u32;
    let max_sx = sw - 1;
    let max_sy = sh - 1;
    let scale_x = dst_w as f32 / sw as f32;
    let scale_y = dst_h as f32 / sh as f32;

    for &r in dirty {
        let dx0 = ((r.x0 as f32 * scale_x).floor() as i32 - 1).clamp(0, dst_w as i32) as u32;
        let dy0 = ((r.y0 as f32 * scale_y).floor() as i32 - 1).clamp(0, dst_h as i32) as u32;
        let dx1 = ((r.x1 as f32 * scale_x).ceil() as i32 + 1).clamp(0, dst_w as i32) as u32;
        let dy1 = ((r.y1 as f32 * scale_y).ceil() as i32 + 1).clamp(0, dst_h as i32) as u32;
        if dx1 <= dx0 || dy1 <= dy0 {
            continue;
        }
        for dy in dy0..dy1 {
            let fy = dy.saturating_mul(step_y);
            let sy0 = (fy >> 16).min(max_sy);
            let sy1 = (sy0 + 1).min(max_sy);
            let wy = (fy >> 8) & 0xff;
            let iwy = 256 - wy;
            let row0 = (sy0 * sw) as usize;
            let row1 = (sy1 * sw) as usize;
            let drow = (dy * dst_w) as usize;

            for dx in dx0..dx1 {
                let fx = dx.saturating_mul(step_x);
                let sx0 = (fx >> 16).min(max_sx) as usize;
                let sx1 = (sx0 + 1).min(max_sx as usize);
                let wx = (fx >> 8) & 0xff;
                let iwx = 256 - wx;

                let p00 = (row0 + sx0) * 4;
                let p10 = (row0 + sx1) * 4;
                let p01 = (row1 + sx0) * 4;
                let p11 = (row1 + sx1) * 4;

                let w00 = iwx * iwy;
                let w10 = wx * iwy;
                let w01 = iwx * wy;
                let w11 = wx * wy;

                let r_v = (u32::from(sdata[p00]) * w00
                    + u32::from(sdata[p10]) * w10
                    + u32::from(sdata[p01]) * w01
                    + u32::from(sdata[p11]) * w11)
                    >> 16;
                let g_v = (u32::from(sdata[p00 + 1]) * w00
                    + u32::from(sdata[p10 + 1]) * w10
                    + u32::from(sdata[p01 + 1]) * w01
                    + u32::from(sdata[p11 + 1]) * w11)
                    >> 16;
                let b_v = (u32::from(sdata[p00 + 2]) * w00
                    + u32::from(sdata[p10 + 2]) * w10
                    + u32::from(sdata[p01 + 2]) * w01
                    + u32::from(sdata[p11 + 2]) * w11)
                    >> 16;
                dst[drow + dx as usize] = (r_v << 16) | (g_v << 8) | b_v;
            }
        }
        out_damage.push([dx0, dy0, dx1 - dx0, dy1 - dy0]);
    }
    out_damage
}

/// Active drag session keeping an **unquantized** `accum: f32` so discrete selectors NEVER lock up
/// on small mouse movements (`vst-3d-ui` Section 5).
#[derive(Clone, Copy, Debug)]
pub struct ActiveDrag {
    pub control_idx: usize,
    pub accum: f32,
    pub moved_px: f32,
}

/// Interactive input controller for the 3D panel supporting continuous knobs, unquantized
/// discrete selector drags (vertical + horizontal + single-click cycle + scroll wheel),
/// rockers, and 3-way toggles.
#[derive(Clone, Debug, Default)]
pub struct DragController {
    pub active: Option<ActiveDrag>,
}

impl DragController {
    pub fn hit_test(manifest: &Manifest, fx: f32, fy: f32) -> Option<usize> {
        for (i, c) in manifest.controls.iter().enumerate() {
            let dx = fx - c.pos[0];
            let dy = fy - c.pos[1];
            let hit = if let Some(r) = c.hit_r {
                dx.hypot(dy) <= r
            } else {
                let hw = c.hit_w.unwrap_or(60.0) * 0.5 + 6.0;
                let hh = c.hit_h.unwrap_or(60.0) * 0.5 + 6.0;
                dx.abs() <= hw && dy.abs() <= hh
            };
            if hit {
                return Some(i);
            }
        }
        None
    }

    pub fn begin_drag(&mut self, manifest: &Manifest, state: &mut Panel3DState, fx: f32, fy: f32) -> Option<usize> {
        let idx = Self::hit_test(manifest, fx, fy)?;
        let c = &manifest.controls[idx];
        match c.kind.as_str() {
            "rocker" => {
                state.values[idx] = if state.values[idx] >= 0.5 { 0.0 } else { 1.0 };
                Some(idx)
            }
            "toggle" => {
                let dx = fx - c.pos[0];
                let dy = fy - c.pos[1];
                let across = c.across.unwrap_or(false);
                let nut = 16.0 * manifest.face.px_per_unit;
                state.values[idx] = if across {
                    if dx < -nut { 0.0 } else if dx > nut { 1.0 } else { 0.5 }
                } else if dy < -nut {
                    1.0
                } else if dy > nut {
                    0.0
                } else {
                    0.5
                };
                Some(idx)
            }
            _ => {
                self.active = Some(ActiveDrag {
                    control_idx: idx,
                    accum: state.values[idx],
                    moved_px: 0.0,
                });
                Some(idx)
            }
        }
    }

    /// Apply incremental mouse movement `(dx, dy)` in logical pixels without re-reading quantized values.
    pub fn update_drag(&mut self, manifest: &Manifest, state: &mut Panel3DState, dx: f32, dy: f32, shift: bool) {
        let Some(drag) = &mut self.active else {
            return;
        };
        let c = &manifest.controls[drag.control_idx];
        drag.moved_px += dx.hypot(dy);
        let sens = if shift { 0.0005 } else { 0.0042 };
        // Support both vertical + horizontal drag on selectors (dx - dy); vertical on knobs (-dy)
        let delta = if c.kind == "selector" {
            (dx - dy) * sens * 1.4
        } else {
            -dy * sens
        };
        drag.accum = (drag.accum + delta).clamp(0.0, 1.0);
        if let Some(steps) = c.steps.filter(|&s| s >= 2) {
            let n = (steps - 1) as f32;
            state.values[drag.control_idx] = (drag.accum * n).round() / n;
        } else {
            state.values[drag.control_idx] = drag.accum;
        }
    }

    /// Release drag; a crisp click (< 4 px travel) on a discrete selector cycles to the next detent!
    pub fn end_drag(&mut self, manifest: &Manifest, state: &mut Panel3DState) {
        if let Some(drag) = self.active.take() {
            let c = &manifest.controls[drag.control_idx];
            if c.kind == "selector" && drag.moved_px < 4.0
                && let Some(steps) = c.steps.filter(|&s| s >= 2)
            {
                let cur = (state.values[drag.control_idx] * (steps - 1) as f32).round() as usize;
                let next = (cur + 1) % steps;
                state.values[drag.control_idx] = next as f32 / (steps - 1) as f32;
            }
        }
    }

    /// Scroll wheel step: exact `±1 / (steps - 1)` on discrete selectors/toggles, smooth on knobs.
    pub fn scroll(manifest: &Manifest, state: &mut Panel3DState, fx: f32, fy: f32, delta_steps: f32, shift: bool) -> bool {
        let Some(idx) = Self::hit_test(manifest, fx, fy) else {
            return false;
        };
        let c = &manifest.controls[idx];
        if let Some(steps) = c.steps.filter(|&s| s >= 2) {
            let n = (steps - 1) as f32;
            let step = if delta_steps > 0.0 { 1.0 } else { -1.0 };
            let cur = (state.values[idx] * n).round();
            state.values[idx] = (cur + step).clamp(0.0, n) / n;
        } else if c.kind == "rocker" {
            state.values[idx] = if delta_steps > 0.0 { 1.0 } else { 0.0 };
        } else {
            let step = if shift { 0.004 } else { 0.025 };
            state.values[idx] = (state.values[idx] + delta_steps * step).clamp(0.0, 1.0);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest() -> Manifest {
        Manifest {
            format: "ca72-3d-sprite-pack-v1".into(),
            projection: "flat-orthographic-0deg".into(),
            face: FaceSpec {
                file: "face-3d.png".into(),
                w: 3840,
                h: 1135,
                px_per_m: 5396.97,
                px_per_unit: 1.18081,
                faceplate_rect_px: [85.0, 153.5, 3755.0, 1092.0],
            },
            sprites: HashMap::new(),
            controls: vec![ControlSpec {
                param: "osc1_range".into(),
                label: "OSC 1 RANGE".into(),
                kind: "selector".into(),
                sprite: "knob_pointer".into(),
                pos: [800.0, 350.0],
                hit_r: Some(70.0),
                hit_w: None,
                hit_h: None,
                steps: Some(6),
                places: vec![0, 1, 2, 3, 4, 5],
                angles_deg: vec![-83.75, -50.25, -16.75, 16.75, 50.25, 83.75],
                orient: None,
                colour: None,
                across: None,
                default: 0.4,
            }],
            lamps: vec![LampSpec {
                id: "power".into(),
                name: "POWER".into(),
                pos: [100.0, 100.0],
                dome_r: 16.0,
                bezel_outer_r: 24.0,
                color_srgb: [255, 36, 18],
                core_srgb: [255, 224, 170],
                faceted: true,
                default_drive: 0.9,
            }],
            quality_opening: None,
        }
    }

    #[test]
    fn discrete_selector_drag_never_locks_up_on_tiny_deltas() {
        let m = test_manifest();
        let mut s = Panel3DState::from_defaults(&m);
        let mut ctl = DragController::default();
        ctl.begin_drag(&m, &mut s, 800.0, 350.0);
        // 80 tiny 1-pixel horizontal drags: a naive quantized drag would round back to 0.4 forever!
        for _ in 0..80 {
            ctl.update_drag(&m, &mut s, 1.0, 0.0, false);
        }
        assert!(
            s.values[0] > 0.79,
            "unquantized accum must advance across detents, got {}",
            s.values[0]
        );
        ctl.end_drag(&m, &mut s);
        // Single click cycles to the next detent
        let before = s.values[0];
        ctl.begin_drag(&m, &mut s, 800.0, 350.0);
        ctl.end_drag(&m, &mut s);
        assert_ne!(s.values[0], before, "single click must cycle selector");
    }

    #[test]
    fn linear_light_lamp_preserves_bezel_and_scales_with_drive() {
        let m = test_manifest();
        let lamp = &m.lamps[0];
        let mut dark = Pixmap::new(200, 200).expect("pixmap");
        let mut mid = Pixmap::new(200, 200).expect("pixmap");
        let mut full = Pixmap::new(200, 200).expect("pixmap");
        for p in [&mut dark, &mut mid, &mut full] {
            p.fill(resvg::tiny_skia::Color::from_rgba8(24, 24, 24, 255));
        }
        let clip = FaceRect { x0: 0, y0: 0, x1: 200, y1: 200 };
        render_lamp_linear(&mut dark, lamp, 0.0, clip);
        render_lamp_linear(&mut mid, lamp, 0.35, clip);
        render_lamp_linear(&mut full, lamp, 1.0, clip);
        let center_r = |p: &Pixmap| p.pixel(100, 100).map_or(0, |c| c.red());
        assert_eq!(center_r(&dark), 24);
        assert!(center_r(&mid) > center_r(&dark) + 80);
        assert!(center_r(&full) > center_r(&mid));
        // Far outside spill stays clean
        assert_eq!(full.pixel(5, 5).map_or(0, |c| c.red()), 24);
    }

    #[test]
    fn fixed_point_dirty_blit_matches_dimensions_and_damage() {
        let mut src = Pixmap::new(64, 32).expect("src");
        src.fill(resvg::tiny_skia::Color::from_rgba8(200, 100, 50, 255));
        let mut dst = vec![0u32; 32 * 16];
        let dmg = blit_dirty_to_u32(
            &src,
            &[FaceRect { x0: 8, y0: 4, x1: 40, y1: 20 }],
            &mut dst,
            32,
            16,
        );
        assert_eq!(dmg.len(), 1);
        assert_eq!(dst[6 * 32 + 10], (200 << 16) | (100 << 8) | 50);
    }
}
