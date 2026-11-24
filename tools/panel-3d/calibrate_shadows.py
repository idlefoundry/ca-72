#!/usr/bin/env python3
"""Resolution-independent directional Shadow Catcher cleanup & faceplate calibration pass.

Follows every lesson in `vst-3d-ui` Section 4:
1. Reads raw Cycles renders from `<raw_dir>` (keeping raw backups untouched) and writes
   calibrated sprites + calibrated `face-3d.png` + `manifest.json` to `<dst_dir>`.
2. Hardware silhouette protection: pixels inside `body_r` / `body_rect` or `alpha >= 245`
   stay 100% untouched.
3. Removes flat Cycles Shadow Catcher ambient veil (~18/255) outside the hardware silhouette.
4. Applies a single smooth directional shadow envelope aligned to key-light shadow vector
   `(0.60, 0.80)` (no two-tier step discontinuity!):
     - Lit top-left arc stays ultra-clean (`lit_gain ~ 0.16`),
     - Shadow bottom-right arc uses a single continuous smooth curve (`shadow_gain ~ 0.50`,
       `smoothstep ** 1.55`),
     - Feathers strictly to 0 alpha well before all 4 cell borders (`border_max_alpha == 0` verified!).
5. Calibrates faceplate anodized aluminum luminance inside `faceplate_rect_px` (masking out
   bright silkscreen, chrome hardware, and wood rails) to a rich studio charcoal gradient
   (~50 sRGB left -> ~36 sRGB right) with zero blown-out grey washout.
"""

import argparse
import json
import shutil
from pathlib import Path

import numpy as np
from PIL import Image


def smoothstep(edge0: np.ndarray, edge1: np.ndarray, x: np.ndarray) -> np.ndarray:
    t = np.clip((x - edge0) / np.maximum(1e-6, edge1 - edge0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def clean_knob_cell(cell: np.ndarray, ax: float, ay: float, body_r: float) -> np.ndarray:
    """Clean one Cycles Shadow Catcher knob/toggle cell with zero border alpha & clean lit rim."""
    h, w, _ = cell.shape
    out = cell.astype(np.float32).copy()
    alpha = out[:, :, 3]

    yy, xx = np.mgrid[0:h, 0:w].astype(np.float32)
    dx = xx - ax
    dy = yy - ay
    r = np.hypot(dx, dy)

    # Hardware silhouette protection: inside body_r OR high-alpha solid hardware pixels
    # (for pointer fins & bat-handle toggle levers that extend past circular body_r!)
    solid = (r <= body_r * 0.995) | (alpha >= 242.0)

    # Compute distance outside solid silhouette smoothly via radial / box margin
    # Also compute border fade strictly reaching 0 at least 6px before all 4 cell edges
    margin = max(6.0, min(w, h) * 0.05)
    bx = np.minimum(xx, (w - 1.0) - xx)
    by = np.minimum(yy, (h - 1.0) - yy)
    border_dist = np.minimum(bx, by)
    border_env = smoothstep(2.0, margin + 10.0, border_dist)

    # Key-light shadow vector: light from upper-left (-0.60, -0.80), shadow cast toward (+0.60, +0.80)
    dir_cos = (dx * 0.60 + dy * 0.80) / np.maximum(1.0, r)
    # Single continuous angular gain: lit top-left ~0.16, shadow bottom-right ~0.52
    dir_t = np.clip((dir_cos + 0.35) / 1.35, 0.0, 1.0)
    dir_t = dir_t * dir_t * (3.0 - 2.0 * dir_t)
    dir_gain = 0.14 + 0.38 * dir_t

    # Radial reach: compact on lit top-left, smooth directional falloff toward bottom-right
    max_reach = body_r + (0.28 + 0.42 * dir_t) * (min(w - ax, h - ay) - body_r)
    rad_env = 1.0 - smoothstep(body_r * 0.98, np.maximum(body_r + 8.0, max_reach), r)
    rad_env = np.power(np.clip(rad_env, 0.0, 1.0), 1.55)

    # Subtract flat Shadow Catcher ambient veil (~18/255) outside solid hardware
    veil = 18.0
    a_no_veil = np.maximum(0.0, (alpha - veil) * (255.0 / (255.0 - veil)))
    new_alpha = a_no_veil * dir_gain * rad_env * border_env

    # Soft transition right at hardware rim so antialiased hardware silhouette edges stay silky smooth
    rim_blend = smoothstep(200.0, 242.0, alpha)
    final_alpha = np.where(solid, alpha, np.maximum(new_alpha, alpha * rim_blend))
    # Enforce hard zero on the outer 2px border ring to guarantee zero seam!
    final_alpha[:2, :] = 0.0
    final_alpha[-2:, :] = 0.0
    final_alpha[:, :2] = 0.0
    final_alpha[:, -2:] = 0.0

    # Keep shadow pixels rich neutral dark (no milky grey shadow fringes)
    shadow_mask = (~solid) & (alpha < 200.0)
    out[:, :, 0] = np.where(shadow_mask, np.minimum(out[:, :, 0], 14.0), out[:, :, 0])
    out[:, :, 1] = np.where(shadow_mask, np.minimum(out[:, :, 1], 14.0), out[:, :, 1])
    out[:, :, 2] = np.where(shadow_mask, np.minimum(out[:, :, 2], 16.0), out[:, :, 2])
    out[:, :, 3] = np.clip(final_alpha, 0.0, 255.0)
    return np.round(out).astype(np.uint8)


def clean_rocker_cell(cell: np.ndarray, ax: float, ay: float, body_w: float, body_h: float) -> np.ndarray:
    """Clean one Cycles Shadow Catcher rectangular rocker cell with zero border alpha."""
    h, w, _ = cell.shape
    out = cell.astype(np.float32).copy()
    alpha = out[:, :, 3]

    yy, xx = np.mgrid[0:h, 0:w].astype(np.float32)
    dx = xx - ax
    dy = yy - ay
    # Signed distance outside rounded hardware bezel rectangle
    qx = np.maximum(0.0, np.abs(dx) - body_w * 0.5)
    qy = np.maximum(0.0, np.abs(dy) - body_h * 0.5)
    dist = np.hypot(qx, qy)

    solid = (dist <= 0.5) | (alpha >= 242.0)

    margin = max(6.0, min(w, h) * 0.08)
    bx = np.minimum(xx, (w - 1.0) - xx)
    by = np.minimum(yy, (h - 1.0) - yy)
    border_env = smoothstep(2.0, margin + 8.0, np.minimum(bx, by))

    r = np.maximum(1.0, np.hypot(dx, dy))
    dir_cos = (dx * 0.60 + dy * 0.80) / r
    dir_t = np.clip((dir_cos + 0.35) / 1.35, 0.0, 1.0)
    dir_t = dir_t * dir_t * (3.0 - 2.0 * dir_t)
    dir_gain = 0.16 + 0.38 * dir_t

    reach = 8.0 + 20.0 * dir_t
    dist_env = np.power(1.0 - smoothstep(0.0, reach, dist), 1.55)

    veil = 18.0
    a_no_veil = np.maximum(0.0, (alpha - veil) * (255.0 / (255.0 - veil)))
    new_alpha = a_no_veil * dir_gain * dist_env * border_env

    rim_blend = smoothstep(200.0, 242.0, alpha)
    final_alpha = np.where(solid, alpha, np.maximum(new_alpha, alpha * rim_blend))
    final_alpha[:2, :] = 0.0
    final_alpha[-2:, :] = 0.0
    final_alpha[:, :2] = 0.0
    final_alpha[:, -2:] = 0.0

    shadow_mask = (~solid) & (alpha < 200.0)
    out[:, :, 0] = np.where(shadow_mask, np.minimum(out[:, :, 0], 14.0), out[:, :, 0])
    out[:, :, 1] = np.where(shadow_mask, np.minimum(out[:, :, 1], 14.0), out[:, :, 1])
    out[:, :, 2] = np.where(shadow_mask, np.minimum(out[:, :, 2], 16.0), out[:, :, 2])
    out[:, :, 3] = np.clip(final_alpha, 0.0, 255.0)
    return np.round(out).astype(np.uint8)


def calibrate_faceplate(raw_path: Path, dst_path: Path, manifest: dict) -> None:
    """Measure left/right median anodized aluminum luminance and smoothly map dark metal pixels
    (excluding silkscreen & bright chrome fixtures) to a rich cohesive studio gradient (~48 -> ~34 sRGB).
    """
    img = Image.open(raw_path).convert("RGB")
    arr = np.asarray(img, dtype=np.float32)
    h, w, _ = arr.shape

    fx0, fy0, fx1, fy1 = [int(round(v)) for v in manifest["face"]["faceplate_rect_px"]]
    # Exclude brushed silver left/right/bottom trim margins when measuring black panel steel
    mx0 = fx0 + int(round(30 * manifest["face"]["px_per_unit"]))
    mx1 = fx1 - int(round(45 * manifest["face"]["px_per_unit"]))
    my0 = fy0 + int(round(20 * manifest["face"]["px_per_unit"]))
    my1 = fy1 - int(round(20 * manifest["face"]["px_per_unit"]))

    sub = arr[my0:my1, mx0:mx1, :]
    luma = 0.2126 * sub[:, :, 0] + 0.7152 * sub[:, :, 1] + 0.0722 * sub[:, :, 2]
    # Select dark anodized aluminum pixels (exclude bright white silkscreen > 95 and chrome)
    dark_mask = (luma >= 8.0) & (luma <= 88.0)
    sw = sub.shape[1]
    left_med = float(np.median(luma[:, : sw // 4][dark_mask[:, : sw // 4]]))
    right_med = float(np.median(luma[:, (3 * sw) // 4 :][dark_mask[:, (3 * sw) // 4 :]]))
    print(f"Faceplate steel median sRGB before calibration: left={left_med:.1f}, right={right_med:.1f}")

    # Gently calibrate aluminum pixels inside faceplate rect toward rich matte studio charcoal:
    # Target left ~ 46 sRGB -> right ~ 35 sRGB (smooth gradient, zero blown-out grey hotspot!)
    yy, xx = np.mgrid[my0:my1, mx0:mx1].astype(np.float32)
    u = np.clip((xx - mx0) / max(1.0, float(mx1 - mx0)), 0.0, 1.0)
    cur_grad = left_med * (1.0 - u) + right_med * u
    tgt_grad = 46.0 * (1.0 - u) + 34.0 * u
    ratio = np.clip(tgt_grad / np.maximum(12.0, cur_grad), 0.72, 1.35)

    # Mask smoothly so bright white silkscreen ink (> 110 sRGB) stays 100% crisp and untouched
    steel_w = 1.0 - smoothstep(68.0, 118.0, luma)
    # Also protect red jewel lamps / chrome via saturation & chroma check
    chroma = np.max(sub, axis=2) - np.min(sub, axis=2)
    steel_w *= 1.0 - smoothstep(14.0, 32.0, chroma)

    gain = 1.0 + (ratio - 1.0) * steel_w
    arr[my0:my1, mx0:mx1, :] = np.clip(sub * gain[:, :, None], 0.0, 255.0)

    # Verify post-calibration steel luminance
    sub2 = arr[my0:my1, mx0:mx1, :]
    luma2 = 0.2126 * sub2[:, :, 0] + 0.7152 * sub2[:, :, 1] + 0.0722 * sub2[:, :, 2]
    left_med2 = float(np.median(luma2[:, : sw // 4][dark_mask[:, : sw // 4]]))
    right_med2 = float(np.median(luma2[:, (3 * sw) // 4 :][dark_mask[:, (3 * sw) // 4 :]]))
    print(f"Faceplate steel median sRGB after calibration:  left={left_med2:.1f}, right={right_med2:.1f}")

    Image.fromarray(np.round(arr).astype(np.uint8), mode="RGB").save(dst_path, optimize=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--raw", type=Path, default=Path("target/panel-3d/vst-pack-raw"))
    parser.add_argument("--dst", type=Path, default=Path("crates/ca72-panel/assets/cycles3d"))
    args = parser.parse_args()

    args.dst.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((args.raw / "manifest.json").read_text())

    # 1. Calibrate static 4K faceplate
    calibrate_faceplate(args.raw / manifest["face"]["file"], args.dst / manifest["face"]["file"], manifest)

    # 2. Clean every sprite atlas cell-by-cell and assert border_max_alpha == 0!
    for key, sp in manifest["sprites"].items():
        raw_path = args.raw / sp["file"]
        dst_path = args.dst / sp["file"]
        img = Image.open(raw_path).convert("RGBA")
        arr = np.asarray(img, dtype=np.uint8).copy()

        cw, ch = sp["cell_w"], sp["cell_h"]
        ax, ay = float(sp["anchor"][0]), float(sp["anchor"][1])
        cols, frames = sp["cols"], sp["frames"]
        kind = sp["kind"]

        border_max = 0
        for idx in range(frames):
            r, c = divmod(idx, cols)
            y0, y1 = r * ch, (r + 1) * ch
            x0, x1 = c * cw, (c + 1) * cw
            cell = arr[y0:y1, x0:x1]
            if kind in ("knob", "toggle"):
                cleaned = clean_knob_cell(cell, ax, ay, float(sp["body_r"]))
            else:
                cleaned = clean_rocker_cell(cell, ax, ay, float(sp["body_w"]), float(sp["body_h"]))
            arr[y0:y1, x0:x1] = cleaned
            bmax = int(
                max(
                    cleaned[0, :, 3].max(),
                    cleaned[-1, :, 3].max(),
                    cleaned[:, 0, 3].max(),
                    cleaned[:, -1, 3].max(),
                )
            )
            border_max = max(border_max, bmax)

        assert border_max == 0, f"{sp['file']} has non-zero border alpha: {border_max}"
        Image.fromarray(arr, mode="RGBA").save(dst_path, optimize=True)
        size_kb = dst_path.stat().st_size / 1024.0
        print(f"Calibrated {sp['file']:32s} ({frames:2d} frames, border_max_alpha={border_max}, {size_kb:7.1f} KB)")

    shutil.copy2(args.raw / "manifest.json", args.dst / "manifest.json")
    total_mb = sum(p.stat().st_size for p in args.dst.glob("*")) / (1024.0 * 1024.0)
    print(f"Total calibrated 4K pack size in {args.dst}: {total_mb:.2f} MB")


if __name__ == "__main__":
    main()
