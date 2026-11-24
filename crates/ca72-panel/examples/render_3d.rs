//! Renders CA-72's native 4K Cycles 3D Minimoog panel and all 1:1 4K verification crops:
//!   - ca72-3d-full-4k.png         (full 3840x1135 composited panel)
//!   - crop_knobs_seams.png        (1:1 4K crop: Oscillator Bank & Mixer knobs, spun caps, shadow seams)
//!   - crop_rockers.png            (1:1 4K crop: Blue & Orange/Red rockers ON/OFF, horizontal & vertical)
//!   - crop_toggles_lamps_jack.png (1:1 4K crop: Chrome bat toggle, jewel lamps, POWER rocker, 1/4" jack, walnut + nameplate)
//!   - strip_lamp_brightness.png   (4-stage 0%/22%/55%/100% linear-light lamp strip for POWER/OVERLOAD & ULTRA)
//!   - strip_knob_rotation.png     (5-stage t=0.0/0.25/0.5/0.75/1.0 Catmull-Rom bicubic rotational blend strip)
//!   - strip_toggle_states.png     (Vertical UP/MID/DOWN & Horizontal LEFT/MID/RIGHT chrome toggle strip)
//!
//! Also benchmarks dirty-rect single-knob drag latency + 16.16 fixed-point softbuffer blit latency.
//!
//! Usage:
//!   cargo run --release -p ca72-panel --example render_3d -- [pack_dir] [out_dir]

use std::path::PathBuf;
use std::time::Instant;

use ca72_panel::panel3d::{
    FaceRect, Panel3DAssets, Panel3DRenderer, Panel3DState, blit_dirty_to_u32,
    render_knob_smooth, render_lamp_linear,
};
use resvg::tiny_skia::{Color, Pixmap};

fn crop_pixmap(src: &Pixmap, x0: u32, y0: u32, w: u32, h: u32) -> Pixmap {
    let x1 = (x0 + w).min(src.width());
    let y1 = (y0 + h).min(src.height());
    let cw = x1.saturating_sub(x0).max(1);
    let ch = y1.saturating_sub(y0).max(1);
    let mut out = Pixmap::new(cw, ch).expect("crop pixmap");
    let sw = src.width() as usize;
    let sd = src.data();
    let dd = out.data_mut();
    for y in 0..ch as usize {
        let s_off = ((y0 as usize + y) * sw + x0 as usize) * 4;
        let d_off = y * cw as usize * 4;
        dd[d_off..d_off + cw as usize * 4].copy_from_slice(&sd[s_off..s_off + cw as usize * 4]);
    }
    out
}

fn blit_opaque(dst: &mut Pixmap, src: &Pixmap, ox: u32, oy: u32) {
    let dw = dst.width() as usize;
    let dh = dst.height() as usize;
    let sw = src.width() as usize;
    let sh = src.height() as usize;
    let sd = src.data();
    let dd = dst.data_mut();
    for y in 0..sh {
        let dy = oy as usize + y;
        if dy >= dh {
            continue;
        }
        let s_row = y * sw * 4;
        let d_row = (dy * dw + ox as usize) * 4;
        let max_w = sw.min(dw.saturating_sub(ox as usize));
        for x in 0..max_w {
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

fn main() {
    let pack_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("crates/ca72-panel/assets/cycles3d"));
    let out_dir = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/panel-3d/renders"));
    std::fs::create_dir_all(&out_dir).expect("create render output dir");

    let t0 = Instant::now();
    let assets = Panel3DAssets::load_from_dir(&pack_dir).expect("load 3D assets");
    println!(
        "Loaded 4K Cycles pack ({}x{}, {} atlases) in {:.1} ms",
        assets.face.width(),
        assets.face.height(),
        assets.atlases.len(),
        t0.elapsed().as_secs_f64() * 1000.0
    );

    // Clone copies needed for isolated verification strips before moving assets into renderer
    let face_copy = assets.face.clone();
    let manifest = assets.manifest.clone();
    let knob_big_atlas = assets.atlases.get("knob_big").expect("knob_big").clone();
    let knob_big_spec = manifest.sprites.get("knob_big").expect("knob_big spec").clone();
    let toggle_vert_atlas = assets.atlases.get("toggle_vert").expect("toggle_vert").clone();
    let toggle_vert_spec = manifest.sprites.get("toggle_vert").expect("toggle_vert spec").clone();
    let toggle_horiz_atlas = assets.atlases.get("toggle_horiz").expect("toggle_horiz").clone();
    let toggle_horiz_spec = manifest.sprites.get("toggle_horiz").expect("toggle_horiz spec").clone();

    let mut renderer = Panel3DRenderer::new(assets);
    let mut state = Panel3DState::from_defaults(&manifest);

    // Classic musical Minimoog patch settings so every knob/rocker/selector shows natural variety!
    let patch: &[(&str, f32)] = &[
        ("tune", 0.50),
        ("osc_mod", 1.0),
        ("glide", 0.22),
        ("mod_mix", 0.35),
        ("osc1_range", 0.60),       // 8'
        ("osc2_range", 0.60),       // 8'
        ("osc3_range", 0.40),       // 16'
        ("osc2_frequency", 0.52),   // slight warm detune (+0.28 st, fractional mid-frame!)
        ("osc3_frequency", 0.48),   // slight warm detune (-0.28 st, fractional mid-frame!)
        ("osc1_waveform", 0.40),    // Sawtooth
        ("osc2_waveform", 0.40),    // Sawtooth
        ("osc3_waveform", 0.60),    // Square
        ("osc3_control", 1.0),      // ON (top)
        ("osc1_volume", 0.78),
        ("osc1_on", 1.0),
        ("ext_volume", 0.30),
        ("ext_on", 0.0),
        ("osc2_volume", 0.72),
        ("osc2_on", 1.0),
        ("noise_volume", 0.18),
        ("noise_on", 0.0),
        ("osc3_volume", 0.65),
        ("osc3_on", 1.0),
        ("noise_type", 1.0),        // PINK (bottom)
        ("filter_poles", 1.0),      // 4-pole 24dB ladder
        ("filter_mode", 0.0),       // Lowpass
        ("filter_mod", 0.0),
        ("keyboard_control_1", 1.0),
        ("keyboard_control_2", 0.0),
        ("cutoff", 0.61),
        ("emphasis", 0.38),
        ("contour_amount", 0.58),
        ("filter_attack", 0.08),
        ("filter_decay", 0.44),
        ("filter_sustain", 0.32),
        ("loudness_attack", 0.04),
        ("loudness_decay", 0.52),
        ("loudness_sustain", 0.76),
        ("volume", 0.74),
        ("main_output", 1.0),
        ("a440", 0.0),
        ("feedback", 0.25),
        ("quality", 1.0),           // ULTRA (toggle UP)
        ("power", 1.0),             // POWER ON
    ];
    for &(param, val) in patch {
        state.set_control(&manifest, param, val);
    }
    state.set_lamp(&manifest, "power", 0.94);
    state.set_lamp(&manifest, "overload", 0.42);
    state.set_lamp(&manifest, "ultra", 0.85);

    let t_full = Instant::now();
    let initial_dirty = renderer.render_dirty(&state);
    println!(
        "Initial full 4K render: {:.2} ms ({} dirty rects)",
        t_full.elapsed().as_secs_f64() * 1000.0,
        initial_dirty.len()
    );

    // 1. Save full native 4K (3840x1135) composited panel
    let full_path = out_dir.join("ca72-3d-full-4k.png");
    renderer.frame().save_png(&full_path).expect("save full 4k");
    println!("Saved {}", full_path.display());

    // 2. Save 1:1 4K close-up verification crops
    // Crop A: Knobs & shadow seams (Oscillator Bank: RANGE, big FREQUENCY knobs, WAVEFORM, OSC 3 CONTROL)
    let crop_knobs = crop_pixmap(renderer.frame(), 580, 150, 1020, 820);
    crop_knobs
        .save_png(out_dir.join("crop_knobs_seams.png"))
        .expect("save crop_knobs_seams");

    // Crop B: Rockers (Mixer blue horizontal/vertical rockers + Filter column orange/red rockers + OVERLOAD jewel)
    let crop_rockers = crop_pixmap(renderer.frame(), 1560, 150, 960, 820);
    crop_rockers
        .save_png(out_dir.join("crop_rockers.png"))
        .expect("save crop_rockers");

    // Crop C: Output/Power section (Walnut rail + CA-72 nameplate + screw + ULTRA lamp + QUALITY chrome bat toggle + POWER jewel + POWER black ribbed rocker + 1/4" PHONES jack)
    let crop_right = crop_pixmap(renderer.frame(), 3020, 0, 820, 1135);
    crop_right
        .save_png(out_dir.join("crop_toggles_lamps_jack.png"))
        .expect("save crop_toggles_lamps_jack");

    // 3. 4-stage multi-brightness linear-light lamp strip (0% / 22% / 55% / 100%)
    //    Row 1: OVERLOAD red faceted jewel; Row 2: POWER red faceted jewel; Row 3: ULTRA amber dome
    let tile = 160u32;
    let drives = [0.0f32, 0.22, 0.55, 1.0];
    let mut lamp_strip = Pixmap::new(tile * 4, tile * 3).expect("lamp strip");
    lamp_strip.fill(Color::from_rgba8(24, 24, 24, 255));
    for (row, lamp_id) in ["overload", "power", "ultra"].iter().enumerate() {
        let lamp = manifest
            .lamps
            .iter()
            .find(|l| l.id == *lamp_id)
            .expect("lamp spec");
        let x0 = (lamp.pos[0].round() as i32 - (tile / 2) as i32).max(0) as u32;
        let y0 = (lamp.pos[1].round() as i32 - (tile / 2) as i32).max(0) as u32;
        for (col, &drv) in drives.iter().enumerate() {
            let mut patch_px = crop_pixmap(&face_copy, x0, y0, tile, tile);
            let mut local_lamp = lamp.clone();
            local_lamp.pos = [lamp.pos[0] - x0 as f32, lamp.pos[1] - y0 as f32];
            render_lamp_linear(
                &mut patch_px,
                &local_lamp,
                drv,
                FaceRect { x0: 0, y0: 0, x1: tile as i32, y1: tile as i32 },
            );
            blit_opaque(&mut lamp_strip, &patch_px, col as u32 * tile, row as u32 * tile);
        }
    }
    lamp_strip
        .save_png(out_dir.join("strip_lamp_brightness.png"))
        .expect("save lamp strip");

    // 4. 5-stage Catmull-Rom mid-frame rotational blend strip (t = 0.0 / 0.25 / 0.5 / 0.75 / 1.0)
    //    Between frame 24 (0 deg) and frame 25 (+6.25 deg) of knob_big!
    let cw = knob_big_spec.cell_w;
    let ch = knob_big_spec.cell_h;
    let mut rot_strip = Pixmap::new(cw * 5, ch).expect("rot strip");
    rot_strip.fill(Color::from_rgba8(42, 41, 40, 255));
    for (i, &frac) in [0.0f32, 0.25, 0.50, 0.75, 1.0].iter().enumerate() {
        let val = (24.0 + frac) / 48.0;
        let px = render_knob_smooth(&knob_big_atlas, &knob_big_spec, val, 1.0).expect("knob blend");
        blit_opaque(&mut rot_strip, &px, i as u32 * cw, 0);
    }
    rot_strip
        .save_png(out_dir.join("strip_knob_rotation.png"))
        .expect("save rot strip");

    // 5. Chrome bat-handle toggle states strip: vertical (DOWN / MID / UP) + horizontal (LEFT / MID / RIGHT)
    let sw = 180u32;
    let sh = 180u32;
    let mut tog_strip = Pixmap::new(sw * 3, sh * 2).expect("toggle strip");
    tog_strip.fill(Color::from_rgba8(40, 39, 38, 255));
    for i in 0..3u32 {
        let cell_v = crop_pixmap(
            &toggle_vert_atlas,
            i * toggle_vert_spec.cell_w,
            0,
            toggle_vert_spec.cell_w,
            toggle_vert_spec.cell_h,
        );
        blit_opaque(
            &mut tog_strip,
            &cell_v,
            i * sw + (sw - toggle_vert_spec.cell_w) / 2,
            (sh - toggle_vert_spec.cell_h) / 2,
        );
        let cell_h = crop_pixmap(
            &toggle_horiz_atlas,
            i * toggle_horiz_spec.cell_w,
            0,
            toggle_horiz_spec.cell_w,
            toggle_horiz_spec.cell_h,
        );
        blit_opaque(
            &mut tog_strip,
            &cell_h,
            i * sw + (sw - toggle_horiz_spec.cell_w) / 2,
            sh + (sh - toggle_horiz_spec.cell_h) / 2,
        );
    }
    tog_strip
        .save_png(out_dir.join("strip_toggle_states.png"))
        .expect("save toggle strip");

    // 6. Verify dirty-rect equivalence & benchmark live drag + 16.16 fixed-point softbuffer blit!
    let mut dst_buf = vec![0u32; 1920 * 568];
    let mut total_drag_ms = 0.0;
    let iters = 40;
    for k in 0..iters {
        state.set_control(&manifest, "cutoff", 0.20 + 0.015 * k as f32);
        let t_d = Instant::now();
        let dirty = renderer.render_dirty(&state);
        let dmg = blit_dirty_to_u32(renderer.frame(), &dirty, &mut dst_buf, 1920, 568);
        total_drag_ms += t_d.elapsed().as_secs_f64() * 1000.0;
        assert!(!dmg.is_empty());
    }
    let t_idle = Instant::now();
    let idle_dirty = renderer.render_dirty(&state);
    let idle_ms = t_idle.elapsed().as_secs_f64() * 1000.0;
    assert!(idle_dirty.is_empty());

    println!(
        "Live 4K single-knob drag + dirty bilinear blit (1920x568): avg {:.2} ms/frame | idle: {:.4} ms",
        total_drag_ms / iters as f64,
        idle_ms
    );

    // Verify dirty-rect frame matches a fresh full render bit-for-bit!
    let dirty_bytes = renderer.frame().data().to_vec();
    renderer.invalidate_all();
    renderer.render_dirty(&state);
    assert_eq!(
        dirty_bytes,
        renderer.frame().data(),
        "dirty-rect composition must equal full composition bit-for-bit"
    );
    println!("VERIFIED: dirty-rect == full render bit-for-bit!");
}
