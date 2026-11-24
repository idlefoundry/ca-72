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


def enamel_pebbles(w, h, pitch, rng):
    """Sample jittered cells in native texels. The gap to the next seed gives the shallow valleys."""
    y, x = np.ogrid[:h, :w]
    row, col = y // pitch, x // pitch
    yf = (y % pitch).astype(np.float32) / pitch
    xf = (x % pitch).astype(np.float32) / pitch
    shape = ((h - 1) // pitch + 3, (w - 1) // pitch + 3)
    seeds_x = rng.uniform(0.04, 0.96, shape).astype(np.float32)
    seeds_y = rng.uniform(0.04, 0.96, shape).astype(np.float32)
    first = np.full((h, w), np.inf, dtype=np.float32)
    second = first.copy()
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            index = row + dy + 1, col + dx + 1
            distance = (xf - dx - seeds_x[index]) ** 2 + (yf - dy - seeds_y[index]) ** 2
            second = np.minimum(second, np.maximum(first, distance))
            np.minimum(first, distance, out=first)
    roundness = np.clip((np.sqrt(second) - np.sqrt(first)) * 1.75, 0, 1)
    return roundness * (2 - roundness) - 0.5


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
    ink = ink_mask(tex_dir / "silkscreen_mask.png")
    h, w = ink.shape
    rng = np.random.default_rng(7201)
    pebble = enamel_pebbles(w, h, 4, rng)
    crinkle = enamel_pebbles(w, h, 7, rng)
    scatter = enamel_pebbles(w, h, 2, rng)
    specks = rng.random((h, w), dtype=np.float32) - 0.5
    handling = value_noise(w, h, 45, 12, rng)
    grain = 0.43 * pebble + 0.17 * crinkle + 0.29 * scatter + 0.11 * specks
    albedo = print_colors(
        ink, (30, 29, 28), (237, 233, 223), 7.0 * grain + 1.4 * handling, 4.0 * grain,
    )
    finish = 0.58 + 0.10 * grain + 0.025 * handling
    roughness = finish * (1 - ink) + (0.51 + 0.045 * grain) * ink
    height = 0.45 + 0.75 * grain * (1 - 0.39 * ink) + 0.15 * ink
    write_maps(tex_dir, "panel", albedo, roughness, height)
    print(f"Built pebbled enamel and silkscreen ({w}×{h})")


def build_nameplate_pbr(tex_dir):
    ink = ink_mask(tex_dir / "plate_mask.png")
    h, w = ink.shape
    rng = np.random.default_rng(7202)
    tooling = (
        0.71 * value_noise(w, h, max(8, w // 98), h, rng)
        + 0.29 * value_noise(w, h, max(8, w // 31), h // 3, rng)
    )
    albedo = print_colors(ink, (23, 23, 23), (230, 225, 214), 11 * tooling, 5 * tooling)
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
