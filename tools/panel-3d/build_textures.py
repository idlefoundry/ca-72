#!/usr/bin/env python3
"""Build the cabinet's albedo, roughness and height PNGs from the silkscreen masks and worn walnut."""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

WORN = Path(__file__).resolve().parents[2] / "crates/ca72-panel/assets/worn"


def value_noise(w, h, cols, rows, rng):
    grid = rng.random((rows, cols), dtype=np.float32) - 0.5
    return np.asarray(Image.fromarray(grid).resize((w, h), Image.Resampling.BICUBIC))


def wrinkle_enamel(w, h, pitch, rng):
    """Sample warped cell ridges in native texels."""
    y, x = np.ogrid[:h, :w]
    shape = ((h - 1) // pitch + 3, (w - 1) // pitch + 3)
    seeds_x = rng.uniform(0.04, 0.96, shape).astype(np.float32)
    seeds_y = rng.uniform(0.04, 0.96, shape).astype(np.float32)
    first = np.full((h, w), np.inf, dtype=np.float32)
    second = first.copy()
    # Distort the crests at roughly three points per cell. The perspective
    # photographs show curved ridges of unequal width and occasional breaks.
    xf = (x % pitch).astype(np.float32) / pitch
    xf = xf + 0.85 * value_noise(w, h, w // (pitch // 3), h // (pitch // 3), rng)
    yf = (y % pitch).astype(np.float32) / pitch
    yf = yf + 0.70 * value_noise(w, h, w // (pitch // 3), h // (pitch // 3), rng)
    row, col = y // pitch, x // pitch
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            index = row + dy + 1, col + dx + 1
            offset_x = xf - dx - seeds_x[index]
            offset_y = yf - dy - seeds_y[index]
            distance = offset_x ** 2 + offset_y ** 2 + 0.46 * offset_x * offset_y
            second = np.minimum(second, np.maximum(first, distance))
            np.minimum(first, distance, out=first)
    gap = np.sqrt(second) - np.sqrt(first)
    return np.exp(-5.4 * gap)


def write_maps(tex_dir, name, albedo, roughness, height):
    for suffix, pixels in (
        ("albedo", albedo),
        ("roughness", roughness * 255),
        ("bump", height * 255),
    ):
        data = np.rint(np.clip(pixels, 0, 255)).astype(np.uint8)
        Image.fromarray(data).save(tex_dir / f"{name}_{suffix}.png")


def ink_mask(path):
    with Image.open(path) as image:
        return np.asarray(image.convert("RGBA").getchannel("A"), dtype=np.float32) / 255


def print_colors(ink, base, lettering, texture, ink_texture):
    colors = np.empty((*ink.shape, 3), dtype=np.uint8)
    for channel in range(3):
        colors[..., channel] = np.rint(
            (base[channel] + texture) * (1 - ink) + (lettering[channel] + ink_texture) * ink
        )
    return colors


def build_faceplate_pbr(tex_dir):
    mask = tex_dir / "silkscreen_mask.png"
    with Image.open(mask) as image:
        alpha = image.convert("RGBA").getchannel("A")
    # Oversample the relief so the fine baked wrinkle survives the texture filter.
    w, h = alpha.width * 2, alpha.height * 2
    ink = np.asarray(alpha.resize((w, h), Image.Resampling.BICUBIC), dtype=np.float32) / 255
    rng = np.random.default_rng(7201)
    # A 0.68 m plate maps 22 texels to about 1.3 mm of wrinkle pitch.
    # Fine grain in the valleys remains subordinate to the fused crests.
    ridge = wrinkle_enamel(w, h, round(w * 0.0013 / 0.68), rng)
    density = value_noise(w, h, w // 28, h // 28, rng)
    grain = ridge * (0.78 + 0.40 * density)
    fine = value_noise(w, h, w // 4, h // 4, rng)
    handling = value_noise(w, h, 45, 12, rng)
    albedo = print_colors(
        ink, (19, 19, 19), (228, 224, 211), 8 * (grain - 0.28) + 3 * fine + 4 * handling,
        4 * (grain - 0.28),
    )
    finish = 0.69 - 0.17 * grain + 0.035 * handling
    roughness = finish * (1 - ink) + (0.56 - 0.045 * grain) * ink
    height = 0.16 + (0.66 * grain + 0.10 * fine) * (1 - 0.48 * ink) + 0.10 * ink
    write_maps(tex_dir, "panel", albedo, roughness, height)
    print(f"Built wrinkle enamel and silkscreen ({w}×{h})")


def build_nameplate_pbr(tex_dir):
    ink = ink_mask(tex_dir / "plate_mask.png")
    h, w = ink.shape
    rng = np.random.default_rng(7202)
    tooling = (
        0.71 * value_noise(w, h, max(8, w // 98), h, rng)
        + 0.29 * value_noise(w, h, max(8, w // 31), h // 3, rng)
    )
    albedo = print_colors(ink, (16, 16, 17), (230, 225, 214), 10 * tooling, 5 * tooling)
    roughness = (0.45 + 0.07 * tooling) * (1 - ink) + 0.43 * ink
    height = 0.50 + 0.38 * tooling - 0.11 * ink
    write_maps(tex_dir, "plate", albedo, roughness, height)
    print(f"Built brushed anodized nameplate ({w}×{h})")


def match_texture_edges(pixels, margin, axis):
    """Feather opposite edges together in the sampling gutters. Board UVs use the interior."""
    edges = np.swapaxes(pixels, 0, axis)
    fade = (0.25 * (1 + np.cos(np.linspace(0, np.pi, margin))))[:, None]
    if edges.ndim == 3:
        fade = fade[..., None]
    left = edges[:margin]
    right = edges[:-margin - 1:-1]
    difference = (right - left) * fade
    left += difference
    right -= difference


def build_walnut_pbr(tex_dir):
    w, h = 2800, 1024
    rng = np.random.default_rng(7203)
    with Image.open(WORN / "walnut.png") as source:
        # Keep the original board's palette and fibers. Ease the photo's upper-left lighting
        # before adding a studio key in Cycles.
        photo = source.convert("RGB")
        exposure = np.asarray(photo.convert("L").filter(ImageFilter.GaussianBlur(105)), dtype=np.float32)
        correction = (np.median(exposure) / np.maximum(exposure, 1)) ** 0.66
        corrected = np.asarray(photo, dtype=np.float32) * correction[..., None]
        photo = Image.fromarray(np.rint(np.clip(corrected, 0, 255)).astype(np.uint8))
        albedo = np.array(photo.resize((w, h), Image.Resampling.LANCZOS), dtype=np.float32)

    luma = Image.fromarray(np.rint(np.clip(albedo, 0, 255)).astype(np.uint8)).convert("L")
    nearby = np.asarray(luma.filter(ImageFilter.GaussianBlur(2.6)), dtype=np.float32)
    fibers = np.clip((np.asarray(luma, dtype=np.float32) - nearby) / 17, -1, 1)
    vessels = value_noise(w, h, w // 15, h // 2, rng)
    pores = np.clip((-fibers - 0.17) * 0.75 + np.maximum(vessels - 0.26, 0), 0, 0.7)

    # Very fine, broken pores continue where the source resolves only the larger fibers.
    albedo -= (4.0 * pores)[..., None]
    albedo *= np.array([1.10, 0.99, 0.88], dtype=np.float32)
    height = 0.55 + 0.11 * fibers - 0.32 * pores + 0.07 * vessels
    roughness = 0.60 - 0.047 * fibers + 0.085 * pores + 0.027 * vessels
    for pixels in (albedo, roughness, height):
        match_texture_edges(pixels, 36, axis=1)
        match_texture_edges(pixels, 20, axis=0)
    write_maps(tex_dir, "walnut", albedo, roughness, height)
    print(f"Built oiled black walnut from {WORN.name}/walnut.png ({w}×{h})")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=Path("target/panel-3d"))
    tex_dir = parser.parse_args().out / "textures"
    build_faceplate_pbr(tex_dir)
    build_nameplate_pbr(tex_dir)
    build_walnut_pbr(tex_dir)


if __name__ == "__main__":
    main()
