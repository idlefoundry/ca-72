#!/usr/bin/env python3
"""Build the four CA-72 switch GLBs in metres, with Y up at export."""

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import bmesh

from shaders_switches import FINISHES, new_switch_material


def create_mesh(name, verts, faces, materials, material_indices=None, sharp_angle=32):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    if mesh.validate():
        raise ValueError(f"{name}: invalid geometry")
    for material in materials:
        mesh.materials.append(material)
    for face in mesh.polygons:
        face.use_smooth = True
        if material_indices is not None:
            face.material_index = material_indices[face.index]

    bm = bmesh.new()
    try:
        bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        if any(not edge.is_manifold or not edge.is_contiguous for edge in bm.edges):
            raise ValueError(f"{name}: open or inconsistently oriented surface")
        if bm.calc_volume(signed=True) <= 0:
            raise ValueError(f"{name}: non-positive signed volume")
        bm.to_mesh(mesh)
    finally:
        bm.free()
    mesh.update()
    mesh.set_sharp_from_angle(angle=math.radians(sharp_angle))

    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    return obj


def loft(name, rings, materials, *, caps=None, span_materials=None, sharp_angle=32):
    """Connect equally sampled rings, closing at two poles or around a bore."""
    width = len(rings[0])
    verts = [point for ring in rings for point in ring]
    faces, material_indices = [], []
    spans = len(rings) if caps is None else len(rings) - 1
    for span in range(spans):
        first = span * width
        last = (span + 1) % len(rings) * width
        for i in range(width):
            j = (i + 1) % width
            faces.append((first + i, first + j, last + j, last + i))
            material_indices.append(0 if span_materials is None else span_materials[span])

    if caps is not None:
        bottom, top = len(verts), len(verts) + 1
        verts.extend(((0, 0, caps[0]), (0, 0, caps[1])))
        last = (len(rings) - 1) * width
        for i in range(width):
            j = (i + 1) % width
            faces.extend(((bottom, j, i), (top, last + i, last + j)))
            material_indices.extend((0, 0 if span_materials is None else span_materials[-1]))

    return create_mesh(name, verts, faces, materials, material_indices, sharp_angle)


def rounded_rectangle(x, y, z, radius):
    points = []
    for quadrant, (sign_x, sign_y) in enumerate(((1, 1), (-1, 1), (-1, -1), (1, -1))):
        cx, cy = sign_x * (x - radius), sign_y * (y - radius)
        for step in range(6):
            angle = math.pi / 2 * (quadrant + step / 5)
            points.append((cx + radius * math.cos(angle), cy + radius * math.sin(angle), z))
    return points


def build_rocker_bezel(dim, bezel, well, *, power):
    half_w, half_h = dim["w"] / 2, dim["h"] / 2
    pad = dim["bezel_pad"] if power else dim["bezel_pad"] * 0.87
    outer_x, outer_y = half_w + pad, half_h + pad
    # The coloured switches sit in shallow, narrow snap-in lips.
    opening = 0.00013 if power else 0.00044
    inner_x, inner_y = half_w + opening, half_h + opening
    top, floor = (0.00122 if power else 0.00072), -0.00485
    # Thin flat lip, small mould breaks and a full-depth pocket for the pressed skirt.
    rings = [
        rounded_rectangle(outer_x, outer_y, floor - 0.00035, 0.00034),
        rounded_rectangle(outer_x, outer_y, top - 0.00016, 0.00034),
        rounded_rectangle(outer_x - 0.00016, outer_y - 0.00016, top, 0.00024),
        rounded_rectangle(inner_x + 0.00012, inner_y + 0.00012, top, 0.00023),
        rounded_rectangle(inner_x, inner_y, top - 0.00012, 0.00018),
        rounded_rectangle(inner_x, inner_y, floor, 0.00018),
    ]
    return loft("Rocker_Bezel", rings, [bezel, well], caps=(floor - 0.00035, floor),
                span_materials=[0, 0, 0, 0, 1])


def power_rib_profile():
    """Sample 13 trapezoidal ribs per half, including the fine bevel of each crest."""
    samples = [(u, 0.0) for u in (0.0, 0.035, 0.072, 0.091, 0.104)]
    start, pitch = 0.112, 0.063
    for i in range(13):
        for phase, relief in (
            (0.00, 0.00), (0.13, 0.09), (0.25, 0.83), (0.33, 1.00),
            (0.60, 1.00), (0.70, 0.83), (0.85, 0.09), (0.94, 0.00),
        ):
            samples.append((start + (i + phase) * pitch, 0.00023 * relief))
    samples.extend((u, 0.0) for u in (0.933, 0.949, 0.970, 0.986, 1.0))
    return [(-u, z) for u, z in reversed(samples[1:])] + samples


def build_rocker_actuator(dim, material, *, vertical, ribbed=False):
    w, h, height = dim["w"], dim["h"], dim["height"]
    length, width = (h, w) if vertical else (w, h)
    if ribbed:
        half_length = length / 2 - 0.00066
        half_width = width / 2 - 0.00021
        profile = power_rib_profile()
    else:
        # Leave room lengthwise for the crown to swing past the inner mould break.
        half_length = length / 2 - 0.00023
        half_width = width / 2 + 0.00022
        profile = [(i / 48, 0.0) for i in range(-48, 49)]

    # Closely space samples along the long edges for the mould chamfer and rib runout.
    across = [-1, -0.982, -0.954, -0.925, -0.89, -0.85, -0.80, -0.746, -0.71, -0.42, 0,
              0.42, 0.71, 0.746, 0.80, 0.85, 0.89, 0.925, 0.954, 0.982, 1]

    def point(u, v, relief):
        a, b = abs(u), abs(v)
        # The actuator is a shallow saddle. At a detent one arm settles
        # almost flat while the opposite arm climbs to the rolled end.
        run = max(0.0, a - 0.091) if ribbed else a
        rise = math.hypot(run, 0.035) - 0.035
        if ribbed:
            roll = max(0.0, (a - 0.86) / 0.14)
            edge = max(0.0, (b - 0.92) / 0.08)
            z = height * (0.293 + 0.345 * rise - 0.15 * roll ** 2.2)
            z -= 0.00028 * edge ** 1.55
        else:
            # The wider draft on the side of the crown remains visible
            # in a flat view, especially where it wraps around the tip.
            roll = max(0.0, (a - 0.72) / 0.28)
            edge_start = 0.85 - 0.12 * a
            edge = max(0.0, (b - edge_start) / (1.0 - edge_start))
            z = height * (0.304 + 0.401 * rise - 0.25 * roll ** 2.05)
            z -= (0.00036 + 0.00072 * a) * edge ** 1.7
        z += relief * min(1.0, (1.0 - b) / 0.09)
        x = u * half_length - u * (b ** 12) * 0.00013
        y = v * half_width - v * (a ** 35) * 0.00013
        return (-y, x, z) if vertical else (x, y, z)

    verts = [point(u, v, z) for u, z in profile for v in across]
    rows, cols = len(profile), len(across)
    faces = []
    for i in range(rows - 1):
        for j in range(cols - 1):
            a = i * cols + j
            faces.append((a, a + cols, a + cols + 1, a + 1))

    # Follow the cap perimeter with a mould break and drafted skirt.
    perimeter = (
        [i * cols for i in range(rows)]
        + [(rows - 1) * cols + j for j in range(1, cols)]
        + [i * cols + cols - 1 for i in range(rows - 2, -1, -1)]
        + list(range(cols - 2, 0, -1))
    )
    for scale, z_offset, fixed_z in ((1.001, -0.00012, None), (0.979, 0.0, -0.00110)):
        start = len(verts)
        for index in perimeter:
            x, y, z = verts[index]
            verts.append((scale * x, scale * y, z + z_offset if fixed_z is None else fixed_z))
        for i, a in enumerate(perimeter):
            j = (i + 1) % len(perimeter)
            faces.append((a, start + i, start + j, perimeter[j]))
        perimeter = list(range(start, len(verts)))

    bottom_center = len(verts)
    verts.append((0, 0, -0.00110))
    for i, a in enumerate(perimeter):
        faces.append((a, bottom_center, perimeter[(i + 1) % len(perimeter)]))
    actuator = create_mesh("Rocker_Actuator", verts, faces, [material], sharp_angle=46)
    actuator.location.z = 0.00132
    return actuator


def polar_ring(radius_at, z_at, count):
    points = []
    for i in range(count):
        angle = math.tau * i / count
        r = radius_at(angle)
        points.append((r * math.cos(angle), r * math.sin(angle), z_at(angle)))
    return points


def round_ring(radius, z, count=96):
    return polar_ring(lambda _: radius, lambda _: z, count)


def angular_distance(angle, center, period=math.tau):
    return abs((angle - center + period / 2) % period - period / 2)


def build_washer(radius, bushing_radius, satin):
    def outer(r, theta):
        # Small indexing notch with a radiused root at the edge of the stamping.
        t = max(0, 1 - (angular_distance(theta, math.pi / 2) / 0.105) ** 2)
        return r - 0.00028 * t

    def bore(r, theta):
        # The internal key engages the slot in the bushing's mounting journal.
        key = max(0, min(1, (0.17 - angular_distance(theta, math.pi / 2)) / 0.045))
        return r - 0.00030 * key

    profile = [
        (outer, radius - 0.00009, 0.0),
        (outer, radius, 0.00009),
        (outer, radius, 0.00050),
        (outer, radius - 0.00008, 0.00060),
        (bore, bushing_radius + 0.00009, 0.00060),
        (bore, bushing_radius + 0.00009, 0.0),
    ]
    rings = [polar_ring(lambda a: shape(r, a), lambda _: z, 120) for shape, r, z in profile]
    return loft("Toggle_Washer", rings, [satin])


def build_nut(across_flats, bushing_radius, satin):
    apothem = across_flats / 2
    corner = apothem / math.cos(math.pi / 6)
    shoulder = apothem - 0.00016

    def hex_radius(angle):
        # Flat normals are spaced every 60 degrees, with small clipped corners.
        flat = apothem / math.cos(angular_distance(angle, 0, math.pi / 3))
        cut = (corner - 0.000085) / math.cos(angular_distance(angle, math.pi / 6, math.pi / 3))
        return min(flat, cut)

    height = 0.00283
    bore = bushing_radius + 0.00003
    rings = [
        polar_ring(lambda a: hex_radius(a) - 0.00013, lambda _: 0.00060, 120),
        polar_ring(hex_radius, lambda _: 0.00074, 120),
        # Intersect the flats with a turning cone to produce the circular top shoulder.
        polar_ring(hex_radius, lambda a: height - 0.92 * (hex_radius(a) - shoulder), 120),
        round_ring(shoulder, height, 120),
        round_ring(bore + 0.000075, height, 120),
        round_ring(bore, height - 0.00010, 120),
        round_ring(bore, 0.00060, 120),
    ]
    return loft("Toggle_HexNut", rings, [satin], sharp_angle=24)


def build_bushing(radius, satin, dark):
    count = 120
    pitch = 0.00053
    thread_bottom, thread_top = 0.00278, 0.00485

    def journal(theta):
        slot = max(0, min(1, (0.185 - angular_distance(theta, math.pi / 2)) / 0.045))
        return radius - 0.00024 * slot

    def thread_ring(z):
        def threaded_radius(theta):
            phase = (z - thread_bottom) / pitch - theta / math.tau
            crest = max(0.0, min(1.0, (0.43 - abs(phase - round(phase))) / 0.30))
            runout = min(1, (z - thread_bottom) / (0.5 * pitch), (thread_top - z) / (0.65 * pitch))
            return radius - 0.00015 * runout * (1 - crest)
        return polar_ring(threaded_radius, lambda _: z, count)

    rings = [polar_ring(journal, lambda _: z, count) for z in (0.0, 0.00061)]
    for step in range(81):
        rings.append(thread_ring(thread_bottom + (thread_top - thread_bottom) * step / 80))
    rings.extend(round_ring(r, z, count) for r, z in (
        (radius - 0.00016, 0.00495),
        (radius - 0.00017, 0.00511),
        (radius - 0.000035, 0.00521),
        (radius - 0.000035, 0.00539),
        (radius - 0.00016, 0.00553),
        (radius - 0.00049, 0.00553),
        (radius - 0.00066, 0.00537),
        (radius - 0.00072, 0.00515),
        (radius - 0.00092, 0.00510),
        (radius - 0.00100, 0.00455),
        (radius - 0.00117, 0.00325),
    ))
    span_materials = [0] * (len(rings) - 4) + [1] * 3
    return loft("Toggle_Bushing", rings, [satin, dark], caps=(0, 0.00325),
                span_materials=span_materials, sharp_angle=34)


def build_lever(dim, chrome):
    pivot_z = 0.00461
    length = dim["lever_length"]
    ball = dim["lever_base_radius"] * 0.92
    tip_radius = dim["lever_tip_radius"]
    # A shallow domed end uses the spec's crown curvature without a ball-shaped grip.
    tip_width = tip_radius * 0.54
    tip_height = tip_width ** 2 / tip_radius
    neck = dim["lever_base_radius"] * 0.66
    shoulder = length - tip_height
    profile = []
    # Spherical pivot with its equator at the local origin.
    for i in range(14):
        angle = math.radians(-84 + i * 10)
        profile.append((ball * math.cos(angle), ball * math.sin(angle)))
    profile.extend(((neck, 0.00114), (neck, 0.00155)))
    for i in range(1, 25):
        t = i / 24
        flare = t * (1.32 - 0.32 * t)
        profile.append((neck + (tip_width - neck) * flare, 0.00155 + (shoulder - 0.00155) * t))
    for i in range(1, 18):
        angle = math.pi / 2 * i / 18
        profile.append((tip_width * math.cos(angle), shoulder + tip_height * math.sin(angle)))

    lever = loft("Toggle_Lever", [round_ring(r, z) for r, z in profile], [chrome],
                 caps=(-ball, length))
    lever.location.z = pivot_z
    return lever


def build_toggle(dim, chrome, satin, dark):
    bushing_radius = dim["bushing_radius"]
    parts = [
        build_washer(dim["washer_radius"], bushing_radius, satin),
        build_nut(dim["nut_across_flats"], bushing_radius, satin),
        build_bushing(bushing_radius, satin, dark),
    ]
    bpy.ops.object.select_all(action="DESELECT")
    for obj in parts:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.join()
    hardware = parts[0]
    hardware.name = "Toggle_Hardware"
    return [hardware, build_lever(dim, chrome)]


def export_glb(objects, path):
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    result = bpy.ops.export_scene.gltf(
        filepath=str(path), export_format="GLB", export_yup=True, use_selection=True,
    )
    if result != {"FINISHED"}:
        raise RuntimeError(f"Failed to export {path}")
    print(f"Exported {path.name} ({path.stat().st_size:,} bytes)", flush=True)
    for obj in objects:
        bpy.data.objects.remove(obj, do_unlink=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=Path("target/panel-3d"))
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
    args = parser.parse_args(argv)
    dims = json.loads((args.out / "spec.json").read_text())["part_dimensions_meters"]
    assets = args.out / "assets"
    assets.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.unit_settings.system = "METRIC"
    materials = {name: new_switch_material(name) for name in FINISHES}

    for part in ("rocker_horiz", "rocker_vert", "rocker_power"):
        power = part == "rocker_power"
        finish = "Rocker_Power_Black" if power else "Rocker_Actuator"
        bezel = build_rocker_bezel(
            dims[part], materials["Rocker_Bezel"], materials["Rocker_Well"], power=power,
        )
        actuator = build_rocker_actuator(
            dims[part], materials[finish], vertical=part != "rocker_horiz", ribbed=power,
        )
        export_glb([bezel, actuator], assets / f"{part}.glb")

    toggle = build_toggle(
        dims["toggle"], materials["Nickel_Chrome"], materials["Nickel_Chrome_Satin"],
        materials["Toggle_Inner_Dark"],
    )
    export_glb(toggle, assets / "toggle_bat.glb")


if __name__ == "__main__":
    main()
