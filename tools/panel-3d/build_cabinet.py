#!/usr/bin/env python3
"""Export the walnut cabinet and its panel fixtures as closed, metric Y-up GLBs."""

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import bmesh
from mathutils import Vector

from shaders_cabinet import SURFACES, make_material


def validate_mesh(obj):
    bm = bmesh.new()
    try:
        bm.from_mesh(obj.data)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        non_manifold = [edge for edge in bm.edges if not edge.is_manifold or not edge.is_contiguous]
        if non_manifold:
            raise ValueError(f"{obj.name}: {len(non_manifold)} non-manifold edges")
        volume = bm.calc_volume(signed=True)
        if volume <= 0:
            raise ValueError(f"{obj.name}: non-positive volume {volume}")
        bm.to_mesh(obj.data)
    finally:
        bm.free()
    obj.data.update()


def mesh_object(name, vertices, faces, material):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    validate_mesh(obj)
    return obj


def lathe(name, profile, material, segments=192, closed=False):
    """Turn a profile around Z, with optional angular variation in radius and height."""
    vertices = []
    rings = []
    for radius, z in profile:
        ring = []
        if radius == 0:
            ring.append(len(vertices))
            vertices.append((0, 0, z))
        else:
            for i in range(segments):
                angle = math.tau * i / segments
                r = radius(angle) if callable(radius) else radius
                height = z(angle) if callable(z) else z
                ring.append(len(vertices))
                vertices.append((r * math.cos(angle), r * math.sin(angle), height))
        rings.append(ring)

    faces = []
    ends = rings[1:] + [rings[0]] if closed else rings[1:]
    for a, b in zip(rings, ends):
        for i in range(segments):
            j = (i + 1) % segments
            if len(a) == 1:
                face = (a[0], b[i], b[j])
            elif len(b) == 1:
                face = (a[i], b[0], a[j])
            else:
                face = (a[i], b[i], b[j], a[j])
            faces.append(face)
    obj = mesh_object(name, vertices, faces, material)
    for face in obj.data.polygons:
        face.use_smooth = True
    return obj


def mesh_uv(obj, project):
    mesh = obj.data
    uv = mesh.uv_layers.active or mesh.uv_layers.new(name="UVMap")
    for loop, coord in zip(mesh.loops, uv.data):
        coord.uv = project(mesh.vertices[loop.vertex_index].co)


def beveled_box(name, size, center, material, bevel, segments=4):
    bpy.ops.mesh.primitive_cube_add(size=1, location=center)
    obj = bpy.context.active_object
    obj.name = name
    obj.dimensions = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel:
        mod = obj.modifiers.new("Machined edges", "BEVEL")
        mod.width = bevel
        mod.segments = segments
        bpy.ops.object.modifier_apply(modifier=mod.name)
    obj.data.materials.append(material)
    validate_mesh(obj)
    if bevel:
        for face in obj.data.polygons:
            face.use_smooth = True
        mod = obj.modifiers.new("Weighted normals", "WEIGHTED_NORMAL")
        bpy.ops.object.modifier_apply(modifier=mod.name)
    return obj


def build_cabinet(spec, mats):
    """Center the cabinet in XY with the faceplate at Z=0 and the body behind it."""
    pm, pu = spec["panel_meters"], spec["panel_units"]
    face_w, face_h = pm["face_w"], pm["face_h"]
    top_h, cheek_w, bottom_h = pm["wood_top_h"], pm["cheek_w"], pm["bottom_rail_h"]
    width, height, depth = pm["total_w"], pm["total_h"], pm["cabinet_depth"]
    y_top = height / 2
    y_face_top = y_top - top_h
    y_face_bottom = y_face_top - face_h
    y_bottom = -y_top

    backing = beveled_box(
        "Chassis_Backing", (face_w, face_h, depth / 2),
        (0, (y_face_top + y_face_bottom) / 2, -depth / 4 - 0.0008),
        mats["Steel_Backing"], 0,
    )
    parts = [backing]

    # The rail and cheeks use separate swaths of one board at the worn UI's grain scale.
    along = 0.965 / width
    across = along * (1400 / 788) * 2.2
    for name, grain_axis, size, center, lip, bevel, band, grain_offset in (
        ("TopRail", 0, (face_w, top_h), (0, y_top - top_h / 2), 0.0072, 0.00145, 0.84, 0.50),
        ("BottomRail", 0, (face_w, bottom_h), (0, y_bottom + bottom_h / 2), 0.0052, 0.00095, 0.62, 0.50),
        ("Cheek_Left", 1, (cheek_w, height), (-(width - cheek_w) / 2, 0), 0.0082, 0.00175, 0.42, 0.36),
        ("Cheek_Right", 1, (cheek_w, height), ((width - cheek_w) / 2, 0), 0.0082, 0.00175, 0.28, 0.64),
    ):
        material = mats["Walnut_Horiz" if grain_axis == 0 else "Walnut_Vert"]
        board = beveled_box(
            f"Walnut_{name}", (*size, depth + lip), (*center, (lip - depth) / 2),
            material, bevel,
        )
        front = (depth + lip) / 2
        mesh_uv(board, lambda v: (
            grain_offset + v[grain_axis] * along,
            band + (v[1 - grain_axis] + (front - v.z) * 0.14) * across,
        ))
        parts.append(board)

    unit = spec["scale_m_per_unit"]
    left_w = pu["trim_left_w"] * unit
    right_w = pu["trim_right_w"] * unit
    bottom_trim = pu["trim_bottom_h"] * unit
    trim_h = face_h - bottom_trim
    trim_y = y_face_top - trim_h / 2
    trim_z = 0.00095
    for where, size, center, axis in (
        ("Left", (left_w, trim_h), ((left_w - face_w) / 2, trim_y), 1),
        ("Right", (right_w, trim_h), ((face_w - right_w) / 2, trim_y), 1),
        ("Bottom", (face_w, bottom_trim), (0, y_face_bottom + bottom_trim / 2), 0),
    ):
        trim = beveled_box(
            f"Trim_{where}", (*size, trim_z), (*center, trim_z / 2),
            mats["Aluminum_Trim"], 0.00019, segments=3,
        )
        mesh_uv(trim, lambda v: (0.5 + v[axis] / size[axis], 0.5 + v[1 - axis] / size[1 - axis]))
        parts.append(trim)
    return parts


def build_name_plate(dim, mats):
    w, h, thickness = dim["w"], dim["h"], dim["thickness"]
    chamfer = 0.00038

    def rounded_rect(inset, z):
        r = 0.00084 - inset
        x, y = w / 2 - inset - r, h / 2 - inset - r
        return [
            (cx + r * math.cos(angle), cy + r * math.sin(angle), z)
            for quarter, (cx, cy) in enumerate(((x, y), (-x, y), (-x, -y), (x, -y)))
            for angle in (math.pi * (quarter + i / 5) / 2 for i in range(6))
        ]

    rings = [
        rounded_rect(0.00009, 0),
        rounded_rect(0, 0.00009),
        rounded_rect(0, thickness - chamfer),
        rounded_rect(chamfer, thickness),
    ]
    n = len(rings[0])
    faces = [tuple(reversed(range(n)))]
    for offset in range(0, n * (len(rings) - 1), n):
        for i in range(n):
            j = (i + 1) % n
            faces.append((offset + i, offset + j, offset + n + j, offset + n + i))
    faces.append(tuple(range(n * (len(rings) - 1), n * len(rings))))
    plate = mesh_object("NamePlate", [v for ring in rings for v in ring], faces, mats["Plate_Face"])
    plate.data.materials.append(mats["Plate_Chamfer"])
    # The cut chamfer is bare aluminum. The body and face retain black anodizing.
    for face in plate.data.polygons[2 * n + 1:3 * n + 1]:
        face.material_index = 1
    mesh_uv(plate, lambda v: (0.5 + v.x / w, 0.5 + v.y / h))
    return plate


def knurled_band(radius, lo, hi, teeth, depth, diamond=False):
    def row(phase, relief):
        def cut(angle):
            tooth = angle * teeth / 2
            if diamond:
                peak = min(abs(math.cos(tooth + phase)), abs(math.cos(tooth - phase)))
            else:
                peak = abs(math.cos(tooth))
            return radius - depth * (1 - peak * relief)
        return cut

    steps = 10 if diamond else 4
    return [
        (row(math.pi * (i / steps - 0.5), min(1, 0.42 + 0.38 * min(i, steps - i))), lo + (hi - lo) * i / steps)
        for i in range(steps + 1)
    ]


def facet_knurl(obj, lo, hi):
    for face in obj.data.polygons:
        if lo < face.center.z < hi:
            face.use_smooth = False


def build_jack_quarter(dim, mats):
    outer, sleeve = dim["outer_radius"], dim["sleeve_radius"]
    bore, h = dim["bore_radius"], dim["height"]
    chrome = mats["Chrome_Polished"]
    nut = lathe("Jack_KnurledRingNut", [
        (sleeve - 0.00010, 0), (outer - 0.00033, 0),
        *knurled_band(outer, 0.00025, 0.00190, 64, 0.00027),
        (outer - 0.00027, 0.00213), (outer - 0.00043, 0.00226),
        (sleeve + 0.00077, 0.00274), (sleeve + 0.00057, 0.00276),
        (sleeve + 0.00042, 0.00268), (sleeve + 0.00025, 0.00263),
        (sleeve - 0.00010, 0.00267),
    ], mats["Chrome_Satin"], segments=512, closed=True)
    facet_knurl(nut, 0.00025, 0.00190)

    # Six shallow concentric turnings above the ring nut, followed by a rolled lip.
    thread = []
    for turn in range(6):
        z = 0.00292 + 0.00035 * turn
        thread.extend(((sleeve - 0.00018, z), (sleeve, z + 0.00013), (sleeve, z + 0.00020)))
    barrel = lathe("Jack_ThreadedBushing", [
        (bore + 0.00048, 0.00022), (sleeve - 0.00015, 0.00022),
        *thread, (sleeve - 0.00008, h - 0.00030),
        (sleeve - 0.00020, h - 0.00008), (sleeve - 0.00037, h),
        (bore + 0.00088, h), (bore + 0.00071, h - 0.00012),
    ], chrome, segments=224, closed=True)

    insulator = lathe("Jack_InsulatingSocket", [
        (0, 0), (bore + 0.00053, 0), (bore + 0.00060, h - 0.00075),
        (bore + 0.00068, h - 0.00039), (bore + 0.00020, h - 0.00034),
        (bore, h - 0.00047), (bore, 0.00028), (0, 0.00028),
    ], mats["Jack_Bore"], segments=160)

    # The spring starts at the lower wall and curves into the bore below the mouth.
    vertices = []
    count = 22
    for i in range(count + 1):
        t = i / count
        y = bore * (-0.98 + 0.70 * t)
        z = 0.00038 + 0.0022 * math.sin(math.pi * t / 1.74)
        half_width = 0.00082 + 0.00016 * math.sin(math.pi * t)
        vertices.extend([(-half_width, y, z), (half_width, y, z),
                         (half_width, y, z + 0.00015), (-half_width, y, z + 0.00015)])
    faces = [(0, 1, 2, 3)]
    for i in range(0, count * 4, 4):
        for j in range(4):
            k = (j + 1) % 4
            faces.append((i + j, i + 4 + j, i + 4 + k, i + k))
    faces.append(tuple(range(count * 4, (count + 1) * 4)))
    leaf = mesh_object("Jack_SprungNickelContact", vertices, faces, mats["Jack_Contact"])
    for face in leaf.data.polygons:
        face.use_smooth = len(face.vertices) == 4 and face.index % 2 == 1
    return [nut, barrel, insulator, leaf]


def glass_dome(name, radius, rise, seat, thickness, material, segments=224):
    def layer(r, z, height):
        return [
            (r * math.cos(angle), z + height * math.sin(angle))
            for angle in (i * math.pi / 64 for i in range(32))
        ] + [(0, z + height)]

    outer = layer(radius, seat, rise)
    inner = layer(radius - thickness, seat - 0.00006, rise - thickness)
    return lathe(name, [*reversed(outer), (radius - 0.00006, seat - 0.00012), *inner], material, segments)


def build_lamp(name, dim, mats, glass_name, ultra=False):
    bezel, dome, h = dim["bezel_radius"], dim["dome_radius"], dim["height"]
    seat = h * (0.55 if ultra else 0.65)
    metal = mats["Chrome_Polished"]
    if ultra:
        outside = [
            (bezel - 0.00031, 0),
            *knurled_band(bezel, 0.00029, seat - 0.00031, 72, 0.00024, diamond=True),
            (bezel - 0.00028, seat - 0.00010), (bezel - 0.00044, seat + 0.00003),
            (bezel - 0.00058, seat + 0.00010),
        ]
    else:
        outside = [
            (bezel - 0.00024, 0), (bezel - 0.00006, 0.00017), (bezel, 0.00031),
            (bezel, seat - 0.00051), (bezel - 0.00012, seat - 0.00012),
            (bezel - 0.00028, seat + 0.00011), (bezel - 0.00041, seat + 0.00022),
        ]
    ring = lathe(f"{name}_ChromeBezel", [
        (dome - 0.00019, 0), *outside,
        (dome + 0.00048, seat + 0.00025), (dome + 0.00035, seat + 0.00020),
        (dome + 0.00027, seat + 0.00008), (dome + 0.00020, seat + 0.00013),
        (dome + 0.00008, seat), (dome + 0.00006, seat - 0.00026),
        (dome - 0.00019, seat - 0.00039),
    ], metal, segments=576 if ultra else 224, closed=True)
    if ultra:
        facet_knurl(ring, 0.00029, seat - 0.00031)
    gasket = lathe(f"{name}_LensRetainer", [
        (dome - 0.00016, seat - 0.00035), (dome + 0.00018, seat - 0.00035),
        (dome + 0.00022, seat - 0.00012), (dome + 0.00012, seat + 0.000045),
        (dome - 0.000065, seat + 0.000045),
    ], mats["Lamp_Retainer"], closed=True)
    lens = glass_dome(
        f"{name}_Glass", dome, h - seat, seat, 0.00068 if ultra else 0.00061, mats[glass_name],
    )

    reflector_mat = mats["Ultra_Reflector" if ultra else "Jewel_Reflector"]
    reflector = lathe(f"{name}_OpticalCup", [
        (0, 0.00011), (dome * 0.92, 0.00011), (dome * 0.95, 0.00022),
        (dome * 0.95, seat - 0.0004), (dome * 0.90, seat - 0.00033),
        (dome * 0.87, seat - 0.00046), (dome * 0.85, seat - 0.00072),
        (dome * 0.84, seat - 0.00073), (dome * 0.80, seat - 0.00072),
        (dome * 0.77, seat - 0.00115), (dome * 0.65, seat - 0.00170),
        (dome * 0.46, seat - 0.00245), (dome * 0.26, seat - 0.00268),
        (0, seat - 0.00269),
    ], reflector_mat)
    parts = [ring, gasket, lens, reflector]
    if ultra:
        parts.extend(build_ultra_bulb(dome, seat, mats))
    return parts


def wire(name, path, radius, material, sides=10):
    points = [Vector(p) for p in path]
    vertices = []
    for i, point in enumerate(points):
        tangent = (points[min(i + 1, len(points) - 1)] - points[max(i - 1, 0)]).normalized()
        normal = tangent.cross(Vector((0, 1, 0)))
        if normal.length_squared < 1e-6:
            normal = tangent.cross(Vector((1, 0, 0)))
        normal.normalize()
        binormal = tangent.cross(normal)
        for j in range(sides):
            angle = math.tau * j / sides
            vertices.append(point + radius * (normal * math.cos(angle) + binormal * math.sin(angle)))
    faces = [tuple(range(sides))]
    for offset in range(0, (len(points) - 1) * sides, sides):
        for i in range(sides):
            j = (i + 1) % sides
            faces.append((offset + i, offset + j, offset + sides + j, offset + sides + i))
    faces.append(tuple(range(len(vertices) - sides, len(vertices))))
    obj = mesh_object(name, vertices, faces, material)
    for face in obj.data.polygons:
        face.use_smooth = True
    return obj


def build_ultra_bulb(dome, seat, mats):
    # The bulb's axis lies upright on the panel, suspended beneath the amber cover.
    bulb_y = -dome * 0.48
    bulb_z = seat - 0.00051
    r = dome * 0.29
    rise = dome * 0.72
    bulb = glass_dome("Ultra_ClearBulbEnvelope", r, rise, 0, 0.00016, mats["Ultra_Bulb_Glass"], segments=128)
    bulb.rotation_euler.x = -math.pi / 2
    bulb.location = (0, bulb_y, bulb_z)

    base = lathe("Ultra_BulbBrassBase", [
        (0, -0.00115), (r * 0.82, -0.00115), (r * 0.88, -0.00098),
        (r * 0.88, -0.00016), (r * 0.82, -0.00004), (0, -0.00004),
    ], mats["Ultra_Brass_Cup"], segments=96)
    base.location = bulb.location
    base.rotation_euler = bulb.rotation_euler

    tungsten = mats["Ultra_Filament"]
    end_x = 0.00071
    filament_y = bulb_y + rise * 0.57
    supports = [
        wire(f"Ultra_Support_{label}", [
            (sign * 0.00038, bulb_y - 0.00014, bulb_z),
            (sign * 0.00050, bulb_y + 0.00094, bulb_z),
            (sign * 0.00065, filament_y - 0.00046, bulb_z),
            (sign * end_x, filament_y, bulb_z),
        ], 0.000062, tungsten)
        for sign, label in ((-1, "L"), (1, "R"))
    ]
    path = []
    for i in range(201):
        t = i / 200
        phase = math.tau * 12 * t
        coil = 0.000063 * min(1, 25 * t, 25 * (1 - t))
        path.append((
            end_x * (2 * t - 1),
            filament_y + 0.00016 * math.sin(math.pi * t) + coil * math.sin(phase),
            bulb_z + coil * math.cos(phase),
        ))
    filament = wire("Ultra_CoiledTungstenFilament", path, 0.000026, tungsten, sides=8)
    return [bulb, base, *supports, filament]


def build_screw_phillips(dim, material):
    r, h = dim["head_radius"], dim["height"]

    def round_min(a, b):
        fillet = 0.095
        blend = max(fillet - abs(a - b), 0) / fillet
        return min(a, b) - 0.25 * fillet * blend ** 2

    def cross(arm, width):
        def distance(angle):
            x, y = max(abs(math.cos(angle)), 1e-9), max(abs(math.sin(angle)), 1e-9)
            horizontal = round_min(arm / x, width / y)
            vertical = round_min(width / x, arm / y)
            return -round_min(-horizontal, -vertical) * r
        return distance

    def crown(radius):
        return h * (1 - 0.23 * max(0, (radius / r) ** 2 - 0.10))

    def driver_loop(arm, width, drop):
        radius = cross(arm, width)
        return radius, lambda angle: crown(radius(angle)) - drop * h

    return lathe("Screw_PhillipsPanHead", [
        (0, 0), (r * 0.79, 0), (r * 0.95, 0.000055), (r, 0.00014),
        (r, h * 0.43), (r * 0.98, h * 0.56), (r * 0.92, h * 0.71),
        (r * 0.81, h * 0.84), (r * 0.69, crown(r * 0.69)), (r * 0.62, crown(r * 0.62)),
        driver_loop(0.558, 0.172, 0), driver_loop(0.544, 0.158, 0.006),
        driver_loop(0.532, 0.146, 0.035), driver_loop(0.521, 0.138, 0.089),
        (cross(0.46, 0.119), h * 0.46), (cross(0.45, 0.109), h * 0.41),
        (cross(0.435, 0.097), h * 0.40), (cross(0.421, 0.086), h * 0.40), (0, h * 0.40),
    ], material, segments=384)


def export_glb(objects, path):
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    result = bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        export_yup=True,
        export_materials="EXPORT",
        export_normals=True,
        use_selection=True,
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
    out = parser.parse_args(argv).out
    spec = json.loads((out / "spec.json").read_text())
    dims = spec["part_dimensions_meters"]
    assets = out / "assets"
    assets.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.unit_settings.system = "METRIC"
    mats = {name: make_material(name) for name in SURFACES}

    export_glb(build_cabinet(spec, mats), assets / "cabinet.glb")
    export_glb([build_name_plate(dims["name_plate"], mats)], assets / "name_plate.glb")
    for asset, name, glass in (
        ("jewel_power", "Jewel_Power", "Jewel_Ruby_Unlit"),
        ("jewel_overload", "Jewel_Overload", "Jewel_Overload_Unlit"),
        ("ultra_bezel", "Ultra", "Ultra_Amber_Unlit"),
    ):
        ultra = asset == "ultra_bezel"
        dim = dims["ultra_lamp" if ultra else asset]
        export_glb(build_lamp(name, dim, mats, glass, ultra), assets / f"{asset}.glb")
    export_glb(build_jack_quarter(dims["jack_quarter"], mats), assets / "jack_quarter.glb")
    export_glb([build_screw_phillips(dims["screw_phillips"], mats["Screw_BlackOxide"])], assets / "screw_phillips.glb")


if __name__ == "__main__":
    main()
