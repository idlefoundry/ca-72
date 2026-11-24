#!/usr/bin/env python3
"""Build high-resolution (>1.5x 4K) PBR texture maps for CA-72's Minimoog faceplate,
anodized nameplate, and oiled American black walnut cabinet.

Inputs (from `cargo run --release -p ca72-panel --example export_silkscreen`):
  <out_dir>/textures/silkscreen_mask.png   (5760x1473 RGBA white silkscreen)
  <out_dir>/textures/plate_mask.png        (2088x656 RGBA white nameplate text)

Outputs:
  <out_dir>/textures/panel_albedo.png      (5760x1473 RGB)
  <out_dir>/textures/panel_roughness.png   (5760x1473 L)
  <out_dir>/textures/panel_bump.png        (5760x1473 L)
  <out_dir>/textures/plate_albedo.png      (2088x656 RGB)
  <out_dir>/textures/plate_roughness.png   (2088x656 L)
  <out_dir>/textures/plate_bump.png        (2088x656 L)
  <out_dir>/textures/walnut_albedo.png     (2800x1024 RGB)
  <out_dir>/textures/walnut_roughness.png  (2800x1024 L)
  <out_dir>/textures/walnut_bump.png       (2800x1024 L)
"""

import argparse
from pathlib import Path
import numpy as np
from PIL import Image


def value_noise_2d(h: int, w: int, gy: int, gx: int, rng: np.random.Generator) -> np.ndarray:
    """Fast smooth tileable bilinear value noise at grid resolution (gy, gx)."""
    grid = rng.uniform(0.0, 1.0, (gy, gx)).astype(np.float32)
    img = Image.fromarray((grid * 255.0).astype(np.uint8), mode="L").resize(
        (w, h), resample=Image.Resampling.BICUBIC
    )
    return np.asarray(img, dtype=np.float32) / 255.0


def build_faceplate_pbr(tex_dir: Path) -> None:
    mask_img = Image.open(tex_dir / "silkscreen_mask.png").convert("RGBA")
    w, h = mask_img.size
    mask_arr = np.asarray(mask_img, dtype=np.float32)
    # Alpha of silkscreen ink (0..1)
    ink = np.clip(mask_arr[:, :, 3] / 255.0, 0.0, 1.0)

    rng = np.random.default_rng(7201)

    # Classic Minimoog matte anodized / lightly textured black aluminum faceplate:
    # Two-frequency fine grain + subtle horizontal brushed/anodized micro-streak.
    fine = rng.normal(0.0, 1.0, (h, w)).astype(np.float32)
    coarse = value_noise_2d(h, w, h // 8, w // 8, rng) - 0.5
    med = value_noise_2d(h, w, h // 3, w // 3, rng) - 0.5
    # Subtle fine horizontal anodized grain
    horiz = value_noise_2d(h, w, h // 2, max(16, w // 48), rng) - 0.5
    # Gentle large-scale tonal variation so the aluminum feels alive, never flat CG
    broad = value_noise_2d(h, w, 12, 36, rng) - 0.5

    grain = 0.35 * np.clip(fine * 0.25, -0.5, 0.5) + 0.35 * med + 0.20 * coarse + 0.25 * horiz

    # Albedo: rich warm-charcoal anodized black (~23..27/255 sRGB, slightly warm per CA-72)
    base_r = 25.0 + grain * 4.5 + broad * 2.2
    base_g = 24.3 + grain * 4.3 + broad * 2.0
    base_b = 23.0 + grain * 4.0 + broad * 1.8

    # Warm ivory/off-white silkscreen ink (#f0ece2) with subtle texture bleed
    ink_r = 240.0 + grain * 5.0
    ink_g = 236.0 + grain * 5.0
    ink_b = 226.0 + grain * 5.0

    alb = np.empty((h, w, 3), dtype=np.uint8)
    alb[:, :, 0] = np.clip(base_r * (1.0 - ink) + ink_r * ink, 0, 255).astype(np.uint8)
    alb[:, :, 1] = np.clip(base_g * (1.0 - ink) + ink_g * ink, 0, 255).astype(np.uint8)
    alb[:, :, 2] = np.clip(base_b * (1.0 - ink) + ink_b * ink, 0, 255).astype(np.uint8)
    Image.fromarray(alb, mode="RGB").save(tex_dir / "panel_albedo.png", optimize=True)

    # Roughness: anodized aluminum ~0.46 ± 0.045, silkscreen paint smoother ~0.27
    rgh_base = 0.46 + grain * 0.07 + broad * 0.025
    rgh_ink = 0.27 + grain * 0.02
    rgh = np.clip((rgh_base * (1.0 - ink) + rgh_ink * ink) * 255.0, 0, 255).astype(np.uint8)
    Image.fromarray(rgh, mode="L").save(tex_dir / "panel_roughness.png", optimize=True)

    # Bump: subtle anodized micro-grain + slightly raised silkscreen ink (+0.24)
    bmp_base = 0.42 + grain * 0.24
    bmp = np.clip((bmp_base + ink * 0.24) * 255.0, 0, 255).astype(np.uint8)
    Image.fromarray(bmp, mode="L").save(tex_dir / "panel_bump.png", optimize=True)
    print(f"Built faceplate PBR maps ({w}x{h})")


def build_nameplate_pbr(tex_dir: Path) -> None:
    mask_img = Image.open(tex_dir / "plate_mask.png").convert("RGBA")
    w, h = mask_img.size
    ink = np.clip(np.asarray(mask_img, dtype=np.float32)[:, :, 3] / 255.0, 0.0, 1.0)
    rng = np.random.default_rng(7202)

    # Fine horizontal brushed black anodized aluminum
    brush_fine = value_noise_2d(h, w, h, max(8, w // 64), rng) - 0.5
    brush_med = value_noise_2d(h, w, h // 2, max(8, w // 32), rng) - 0.5
    grain = 0.6 * brush_fine + 0.4 * brush_med

    # Rich brushed satin-black base + crisp bright etched aluminum/off-white lettering
    base_r = 20.0 + grain * 9.0
    base_g = 20.0 + grain * 9.0
    base_b = 20.5 + grain * 9.0

    ink_r = 230.0 + grain * 10.0
    ink_g = 226.0 + grain * 10.0
    ink_b = 216.0 + grain * 10.0

    alb = np.empty((h, w, 3), dtype=np.uint8)
    alb[:, :, 0] = np.clip(base_r * (1.0 - ink) + ink_r * ink, 0, 255).astype(np.uint8)
    alb[:, :, 1] = np.clip(base_g * (1.0 - ink) + ink_g * ink, 0, 255).astype(np.uint8)
    alb[:, :, 2] = np.clip(base_b * (1.0 - ink) + ink_b * ink, 0, 255).astype(np.uint8)
    Image.fromarray(alb, mode="RGB").save(tex_dir / "plate_albedo.png", optimize=True)

    rgh = np.clip(((0.34 + grain * 0.08) * (1.0 - ink) + 0.22 * ink) * 255.0, 0, 255).astype(np.uint8)
    Image.fromarray(rgh, mode="L").save(tex_dir / "plate_roughness.png", optimize=True)

    bmp = np.clip((0.45 + grain * 0.22 + ink * 0.22) * 255.0, 0, 255).astype(np.uint8)
    Image.fromarray(bmp, mode="L").save(tex_dir / "plate_bump.png", optimize=True)
    print(f"Built nameplate PBR maps ({w}x{h})")


def build_walnut_pbr(tex_dir: Path) -> None:
    """Synthesize rich oiled American black walnut wood grain (2800x1024, horizontal grain)."""
    w, h = 2800, 1024
    rng = np.random.default_rng(7203)

    y = np.linspace(0.0, 1.0, h, dtype=np.float32)[:, None]
    x = np.linspace(0.0, 1.0, w, dtype=np.float32)[None, :]

    # Low-frequency cathedral / flowing grain warp
    warp1 = value_noise_2d(h, w, 10, 14, rng) - 0.5
    warp2 = value_noise_2d(h, w, 28, 36, rng) - 0.5
    warp3 = value_noise_2d(h, w, 72, 24, rng) - 0.5

    coord = y * 15.0 + warp1 * 3.6 + warp2 * 1.15 + warp3 * 0.30 + 0.35 * np.sin(x * np.pi * 2.0)
    rings = 0.5 + 0.5 * np.sin(coord * np.pi * 2.0)
    # Sharper dark latewood bands vs warmer heartwood earlywood
    rings = np.power(rings, 0.72)

    # Fine elongated wood pores / vessels (signature of real oiled black walnut!)
    pores_raw = value_noise_2d(h, w, h // 2, w // 14, rng)
    pores = np.clip((pores_raw - 0.72) / 0.28, 0.0, 1.0) ** 1.6
    fine_fiber = value_noise_2d(h, w, h // 2, w // 28, rng) - 0.5

    # Warm rich walnut palette (oiled American black walnut: deep chocolate-espresso to honey-umber)
    tone = np.clip(0.30 + 0.52 * rings + 0.14 * warp1 + 0.10 * fine_fiber - 0.38 * pores, 0.0, 1.0)

    # Deep dark brown (36, 19, 10) -> rich oiled mid-walnut (116, 68, 36) -> warm amber highlights (148, 92, 50)
    r = 34.0 + 110.0 * np.power(tone, 0.92)
    g = 18.0 + 68.0 * np.power(tone, 1.08)
    b = 10.0 + 36.0 * np.power(tone, 1.25)

    alb = np.empty((h, w, 3), dtype=np.uint8)
    alb[:, :, 0] = np.clip(r, 0, 255).astype(np.uint8)
    alb[:, :, 1] = np.clip(g, 0, 255).astype(np.uint8)
    alb[:, :, 2] = np.clip(b, 0, 255).astype(np.uint8)
    Image.fromarray(alb, mode="RGB").save(tex_dir / "walnut_albedo.png", optimize=True)

    # Satin-oiled wood roughness: smooth satin (~0.30) on heartwood, rougher (~0.62) inside open pores
    rgh = np.clip((0.30 + 0.10 * (1.0 - rings) + 0.32 * pores + 0.05 * fine_fiber) * 255.0, 0, 255).astype(np.uint8)
    Image.fromarray(rgh, mode="L").save(tex_dir / "walnut_roughness.png", optimize=True)

    # Bump: recessed pores + gentle fiber relief
    bmp = np.clip((0.58 + 0.10 * rings + 0.08 * fine_fiber - 0.45 * pores) * 255.0, 0, 255).astype(np.uint8)
    Image.fromarray(bmp, mode="L").save(tex_dir / "walnut_bump.png", optimize=True)
    print(f"Built walnut PBR maps ({w}x{h})")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=Path("target/panel-3d"))
    args = parser.parse_args()
    tex_dir = args.out / "textures"
    build_faceplate_pbr(tex_dir)
    build_nameplate_pbr(tex_dir)
    build_walnut_pbr(tex_dir)


if __name__ == "__main__":
    main()
