#!/usr/bin/env python3
"""Build the cabinet's albedo, roughness and height PNGs from the silkscreen masks and worn walnut."""

import argparse
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

from artifacts import output_dir

WORN = Path(__file__).resolve().parents[2] / "crates/ca72-panel/assets/worn"


def value_noise(w, h, cols, rows, rng):
    grid = rng.random((rows, cols), dtype=np.float32) - 0.5
    return np.asarray(Image.fromarray(grid).resize((w, h), Image.Resampling.BICUBIC))


def wrinkle_enamel(w, h, pitch_x, pitch_y, rng):
    """Sample the elongated boundaries where shrinking enamel has buckled."""
    y, x = np.ogrid[:h, :w]
    shape = ((h - 1) // pitch_y + 3, (w - 1) // pitch_x + 3)
    seeds_x = rng.uniform(0.09, 0.91, shape).astype(np.float32)
    seeds_y = rng.uniform(0.09, 0.91, shape).astype(np.float32)
    first = np.full((h, w), np.inf, dtype=np.float32)
    second = first.copy()
    # Bend neighboring boundaries together, then vary their width within each
    # cell. The two independent disturbances keep the ridges from looking woven.
    flow = value_noise(w, h, max(2, w // (pitch_x * 2)), max(2, h // (pitch_y * 2)), rng)
    xf = (x % pitch_x).astype(np.float32) / pitch_x
    xf = xf + 0.66 * flow + 0.45 * value_noise(w, h, w // (pitch_x // 2), h // (pitch_y // 2), rng)
    yf = (y % pitch_y).astype(np.float32) / pitch_y
    yf = yf + 0.43 * value_noise(w, h, w // (pitch_x * 2), h // pitch_y, rng)
    row, col = y // pitch_y, x // pitch_x
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            index = row + dy + 1, col + dx + 1
            offset_x = xf - dx - seeds_x[index]
            offset_y = yf - dy - seeds_y[index]
            distance = offset_x ** 2 + offset_y ** 2 + 0.46 * offset_x * offset_y
            second = np.minimum(second, np.maximum(first, distance))
            np.minimum(first, distance, out=first)
    gap = np.sqrt(second) - np.sqrt(first)
    return np.exp(-4.7 * gap)


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
    # Scale the wrinkle bands from the photographed 1974 hardware, using the
    # adjacent knob diameters as a ruler.
    ridge = wrinkle_enamel(w, h, round(w * 0.0009 / 0.68), round(w * 0.0019 / 0.68), rng)
    density = value_noise(w, h, w // 39, h // 31, rng)
    handling = value_noise(w, h, 51, 13, rng)
    breaks = value_noise(w, h, w // 16, h // 13, rng)
    # Uneven relief interrupts the crests, leaving long and short arcs instead
    # of a seamless grid of closed cells.
    grain = ridge * np.clip(0.62 + 0.7 * density + 1.0 * breaks - 0.46 * handling, 0.10, 1.0)
    fine = value_noise(w, h, w // 4, h // 4, rng)
    # Dry ink settles into the wrinkles. Abrasion thins the white on the
    # highest ridges while the original vector mask holds the letter edges.
    raised = np.clip((grain - 0.29) * 1.8, 0.0, 1.0)
    ink_wear = 5.0 * fine + 3.0 * handling - 21.0 * raised
    albedo = print_colors(
        ink, (18, 19, 22), (222, 220, 209), 4.0 * grain + 2.0 * fine + 5.0 * handling,
        ink_wear,
    )
    finish = 0.67 - 0.31 * grain + 0.065 * handling
    roughness = finish * (1 - ink) + (0.53 - 0.095 * grain) * ink
    height = 0.15 + (0.72 * grain + 0.09 * fine) * (1 - 0.14 * ink) + 0.055 * ink
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
    nearby = np.asarray(luma.filter(ImageFilter.GaussianBlur(3.2)), dtype=np.float32)
    fibers = np.clip((np.asarray(luma, dtype=np.float32) - nearby) / 16, -1, 1)
    vessels = value_noise(w, h, w // 19, h // 2, rng)
    pores = np.clip((-fibers - 0.19) * 0.95 + np.maximum(vessels - 0.24, 0) * 1.65, 0, 0.8)

    # The open pores attenuate reflectance and break up the oil sheen.
    albedo = np.clip(albedo - (12.0 * pores)[..., None], 0.0, 255.0)
    albedo *= np.array([1.10, 0.98, 0.85], dtype=np.float32)
    height = 0.56 + 0.12 * fibers - 0.41 * pores + 0.045 * vessels
    roughness = 0.54 - 0.078 * fibers + 0.19 * pores + 0.041 * vessels
    for pixels in (albedo, roughness, height):
        match_texture_edges(pixels, 36, axis=1)
        match_texture_edges(pixels, 20, axis=0)
    write_maps(tex_dir, "walnut", albedo, roughness, height)
    print(f"Built oiled black walnut from {WORN.name}/walnut.png ({w}×{h})")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path)
    tex_dir = output_dir(parser.parse_args().out) / "textures"
    build_faceplate_pbr(tex_dir)
    build_nameplate_pbr(tex_dir)
    build_walnut_pbr(tex_dir)


if __name__ == "__main__":
    main()
