#!/usr/bin/env python3
"""Build the four CA-72 switch GLBs in metres, with Y up at export."""

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import bmesh

from artifacts import output_dir
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
    pad = dim["bezel_pad"] * (0.51 if power else 0.47)
    outer_x, outer_y = half_w + pad, half_h + pad
    inner_x, inner_y = half_w + 0.00010, half_h + 0.00010
    top, floor = (0.00036 if power else 0.00024), -0.00485
    # A thin aperture lip meets the panel, with a dark housing behind it.
    rings = [
        rounded_rectangle(outer_x, outer_y, floor - 0.00035, 0.00027),
        rounded_rectangle(outer_x, outer_y, top - 0.00009, 0.00027),
        rounded_rectangle(outer_x - 0.000085, outer_y - 0.000085, top, 0.00023),
        rounded_rectangle(inner_x + 0.000075, inner_y + 0.000075, top, 0.00020),
        rounded_rectangle(inner_x, inner_y, top - 0.00009, 0.00018),
        rounded_rectangle(inner_x, inner_y, floor, 0.00018),
    ]
    return loft("Rocker_Bezel", rings, [bezel, well], caps=(floor - 0.00035, floor),
                span_materials=[0, 0, 0, 0, 1])


def rocker_paddle_profile(length, height):
    """Cross-section of the rivet-mounted paddle through its pivot."""
    # Two planar wings meet at a fine molded hinge radius. The nose sweeps
    # around that pivot, so the raised end can expose its full flank.
    radius = length / 2 - 0.00020
    nose_angle = math.radians(24)
    corner = 0.00027
    nose_x = radius * math.cos(nose_angle)
    nose_z = radius * math.sin(nose_angle)
    deck = height * 0.235
    slope = (nose_z - deck) / nose_x
    samples = [(0.0, deck)]
    for x, factor in ((0.00022, 0.46), (0.00053, 0.72), (0.00086, 0.83)):
        samples.append((x, deck + slope * x * factor))
    # A shallow molded crown lets a reflection travel along each wing as it pivots.
    wing_start, wing_end = samples[-1][0], nose_x - 2 * corner
    for fraction in (0.25, 0.5, 0.75, 1.0):
        x = wing_start + (wing_end - wing_start) * fraction
        crown = 0.00012 * math.sin(math.pi * fraction)
        samples.append((x, deck + slope * x + crown))
    samples.extend((
        (nose_x - 0.66 * corner, nose_z - 0.60 * corner * slope),
        (nose_x - 0.13 * corner, nose_z - 0.19 * corner),
    ))
    for degrees in (23.0, 21.5, 18.0, 14.0, 9.0, 4.0, 0.0, -5.5, -10.0):
        angle = math.radians(degrees)
        samples.append((radius * math.cos(angle), radius * math.sin(angle)))
    return [(-x, z) for x, z in reversed(samples[1:])] + samples


def build_rocker_actuator(dim, material, *, vertical):
    w, h, height = dim["w"], dim["h"], dim["height"]
    length, width = (h, w) if vertical else (w, h)
    half_width = width / 2 - 0.00014
    profile = rocker_paddle_profile(length, height)
    end_x = profile[-1][0] - 0.00012
    profile.extend(((end_x, -0.00250), (-end_x, -0.00250)))

    # Restrict curvature to the rolled nose and small break around the edge.
    across = (-1, -0.987, -0.973, -0.952, -0.928, 0, 0.928, 0.952, 0.973, 0.987, 1)

    def point(x, z, v):
        edge = max(0.0, (abs(v) - 0.95) / 0.05)
        end = min(1.0, abs(x) / (length / 2)) ** 10
        long = x * (1.0 - edge ** 1.7 * end * 0.009)
        wide = v * (half_width - 0.000085 * end - 0.00012 * max(0.0, -z) / 0.002)
        high = z - 0.00022 * edge ** 1.7
        return (-wide, long, high) if vertical else (long, wide, high)

    # Loft the closed paddle section over its width, then close the sidewalls.
    verts = [point(x, z, v) for v in across for x, z in profile]
    count = len(profile)
    faces = []
    for first in range(0, len(verts) - count, count):
        last = first + count
        for i in range(count):
            j = (i + 1) % count
            faces.append((first + i, first + j, last + j, last + i))
    faces.append(tuple(reversed(range(count))))
    faces.append(tuple(range(len(verts) - count, len(verts))))
    actuator = create_mesh("Rocker_Actuator", verts, faces, [material], sharp_angle=28)
    actuator.location.z = -0.00032
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
    parser.add_argument("--out", type=Path)
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
    args = parser.parse_args(argv)
    args.out = output_dir(args.out)
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
            dims[part], materials[finish], vertical=part != "rocker_horiz",
        )
        export_glb([bezel, actuator], assets / f"{part}.glb")

    toggle = build_toggle(
        dims["toggle"], materials["Nickel_Chrome"], materials["Nickel_Chrome_Satin"],
        materials["Toggle_Inner_Dark"],
    )
    export_glb(toggle, assets / "toggle_bat.glb")


if __name__ == "__main__":
    main()
