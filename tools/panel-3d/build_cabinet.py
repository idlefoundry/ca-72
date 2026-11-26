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

from artifacts import output_dir
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


def lathe(name, profile, material, segments=192, closed=False, knurl_rows=range(0)):
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
    flat_faces = set()
    for row, (a, b) in enumerate(zip(rings, ends)):
        for i in range(segments):
            j = (i + 1) % segments
            if len(a) == 1:
                face = (a[0], b[i], b[j])
            elif len(b) == 1:
                face = (a[i], b[0], a[j])
            else:
                face = (a[i], b[i], b[j], a[j])
            if row in knurl_rows:
                # Split each cell along the trough shared by two neighboring diamonds.
                if (i + row - knurl_rows.start) % 2 == 0:
                    face = face[1:] + face[:1]
                flat_faces.update((len(faces), len(faces) + 1))
                faces.extend((face[:3], (face[0], face[2], face[3])))
                continue
            faces.append(face)
    obj = mesh_object(name, vertices, faces, material)
    for face in obj.data.polygons:
        face.use_smooth = face.index not in flat_faces
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
    cheek_h = face_h
    cheek_y = (y_face_top + y_face_bottom) / 2

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
        ("TopRail", 0, (width, top_h), (0, y_top - top_h / 2), 0.0072, 0.00185, 0.84, 0.50),
        ("BottomRail", 0, (width, bottom_h), (0, y_bottom + bottom_h / 2), 0.0041, 0.00105, 0.62, 0.50),
        ("Cheek_Left", 1, (cheek_w, cheek_h), (-(width - cheek_w) / 2, cheek_y), 0.0049, 0.00165, 0.42, 0.36),
        ("Cheek_Right", 1, (cheek_w, cheek_h), ((width - cheek_w) / 2, cheek_y), 0.0049, 0.00165, 0.28, 0.64),
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
    trim_z = 0.00135
    for where, size, center, axis in (
        ("Left", (left_w, trim_h), ((left_w - face_w) / 2, trim_y), 1),
        ("Right", (right_w, trim_h), ((face_w - right_w) / 2, trim_y), 1),
        ("Bottom", (face_w, bottom_trim), (0, y_face_bottom + bottom_trim / 2), 0),
    ):
        trim = beveled_box(
            f"Trim_{where}", (*size, trim_z), (*center, trim_z / 2),
            mats["Aluminum_Trim"], 0.00035, segments=5,
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


def diamond_knurl(teeth, rows):
    """Raised diamond tips on alternating rings of a turned metal blank."""
    profile = []
    for index, (root, crest, height) in enumerate(rows):
        def radius(angle, root=root, crest=crest, parity=index % 2):
            column = round(angle * 2 * teeth / math.tau)
            return root if (column + parity) % 2 else crest

        profile.append((radius, height))
    return profile


def build_jack_quarter(dim, mats):
    outer, sleeve = dim["outer_radius"], dim["sleeve_radius"]
    bore, h = dim["bore_radius"], dim["height"]
    chrome = mats["Chrome_Polished"]
    teeth = 56
    band = diamond_knurl(teeth, [
        (outer - 0.00060, outer - 0.00060, 0.00027),
        (outer - 0.00040, outer - 0.00004, 0.00069),
        (outer - 0.00038, outer, 0.00111),
        (outer - 0.00045, outer - 0.00006, 0.00153),
        (outer - 0.00056, outer - 0.00016, 0.00194),
        (outer - 0.00072, outer - 0.00032, 0.00231),
        (outer - 0.00100, outer - 0.00100, 0.00260),
    ])
    nut = lathe("Jack_KnurledRingNut", [
        (sleeve - 0.00010, 0), (outer - 0.00063, 0), *band,
        (outer - 0.00107, 0.00266), (outer - 0.00113, 0.00273),
        (sleeve - 0.00010, 0.00280),
    ], chrome, segments=2 * teeth, closed=True, knurl_rows=range(2, 2 + len(band) - 1))

    across_flats = dim["nut_across_flats"] * 0.80

    def hex_edge(inset):
        def radius(angle):
            sector = (angle + math.pi / 6) % (math.pi / 3) - math.pi / 6
            return (across_flats / 2 - inset) / math.cos(min(abs(sector), math.radians(29)))

        return radius

    collar = lathe("Jack_HexCollar", [
        (sleeve - 0.00005, 0.00269), (hex_edge(0.00022), 0.00269),
        (hex_edge(0), 0.00288), (hex_edge(0), 0.00313),
        (hex_edge(0.00023), 0.00357), (sleeve + 0.00015, 0.00360),
        (sleeve - 0.00005, 0.00347),
    ], chrome, segments=192, closed=True)

    # The visible lead-in continues up the chamfer, so the turning rings read head-on.
    threads = []
    for turn in range(4):
        r = sleeve - 0.00029 * turn
        z = 0.00375 + 0.00028 * turn
        threads.extend(((r, z), (r - 0.00006, z + 0.00015), (r - 0.00015, z + 0.000025)))
    barrel = lathe("Jack_ThreadedBushing", [
        (bore + 0.00048, 0.00022), (sleeve - 0.00015, 0.00022),
        (sleeve - 0.00015, 0.00337), (sleeve + 0.00003, 0.00346),
        *threads, (bore + 0.00082, h - 0.00016),
        (bore + 0.00068, h), (bore + 0.00038, h),
        (bore + 0.00025, h - 0.00012), (bore + 0.00023, h - 0.00045),
    ], mats["Chrome_Satin"], segments=224, closed=True)

    insulator = lathe("Jack_InsulatingSocket", [
        (0, 0), (bore + 0.00053, 0), (bore + 0.00035, h - 0.00075),
        (bore + 0.00029, h - 0.00039), (bore + 0.00020, h - 0.00034),
        (bore, h - 0.00047), (bore, 0.00028), (0, 0.00028),
    ], mats["Jack_Bore"], segments=160)

    # The spring starts at the lower wall and curves into the bore below the mouth.
    vertices = []
    count = 22
    for i in range(count + 1):
        t = i / count
        y = bore * (-0.98 + 0.53 * t)
        z = 0.00038 + 0.00175 * math.sin(math.pi * t / 1.74)
        half_width = 0.00070 + 0.00012 * math.sin(math.pi * t)
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
    return [nut, collar, barrel, insulator, leaf]


def glass_dome(name, radius, rise, seat, thickness, material, segments=224, *, faceted=False):
    def layer(r, z, height, steps):
        return [
            (r * math.cos(angle), z + height * math.sin(angle))
            for angle in (i * math.pi / (2 * steps) for i in range(steps))
        ] + [(0, z + height)]

    outer = layer(radius, seat, rise, 32)
    inner = layer(radius - thickness, seat - 0.00006, rise - thickness, 5 if faceted else 32)
    profile = [*reversed(outer), (radius - 0.00006, seat - 0.00012), *inner]
    dome = lathe(name, profile, material, 24 if faceted else segments)
    if faceted:
        # The molded outside stays smooth. The prism faces are on the inside of the cap.
        first_prism = (len(outer) + 1) * 24
        for face in dome.data.polygons[first_prism:]:
            face.use_smooth = False
    return dome


def build_lamp(name, dim, mats, glass_name, ultra=False):
    bezel, dome, h = dim["bezel_radius"], dim["dome_radius"], dim["height"]
    seat = h * (0.55 if ultra else 0.65)
    metal = mats["Chrome_Polished" if ultra else (
        "Overload_Bezel" if glass_name == "Jewel_Overload_Unlit" else "Pilot_Nickel"
    )]
    knurl_rows = range(0)
    segments = 224
    if ultra:
        teeth = 72
        segments = 2 * teeth
        band = diamond_knurl(teeth, [
            (bezel - 0.00075, bezel - 0.00075, 0.00027),
            (bezel - 0.00044, bezel - 0.00007, 0.00077),
            (bezel - 0.00040, bezel, 0.00127),
            (bezel - 0.00049, bezel - 0.00006, 0.00177),
            (bezel - 0.00053, bezel - 0.00010, 0.00227),
            (bezel - 0.00062, bezel - 0.00018, 0.00277),
            (bezel - 0.00074, bezel - 0.00028, 0.00327),
            (bezel - 0.00098, bezel - 0.00098, seat - 0.00040),
        ])
        knurl_rows = range(2, 2 + len(band) - 1)
        outside = [
            (bezel - 0.00076, 0), *band,
            (bezel - 0.00103, seat - 0.00027),
            (dome + 0.00083, seat + 0.00039), (dome + 0.00069, seat + 0.00046),
        ]
    else:
        outside = [
            (bezel - 0.00024, 0), (bezel - 0.00006, 0.00017), (bezel, 0.00031),
            (bezel, seat - 0.00067), (bezel - 0.00007, seat - 0.00054),
            (bezel - 0.00018, seat - 0.00053), (bezel - 0.00025, seat - 0.00049),
            (bezel - 0.00037, seat - 0.00027), (bezel - 0.00052, seat + 0.00005),
            (bezel - 0.00063, seat + 0.00024), (bezel - 0.00076, seat + 0.00039),
            (bezel - 0.00088, seat + 0.00045),
        ]
    ring = lathe(f"{name}_ChromeBezel", [
        (dome - 0.00019, 0), *outside,
        (dome + 0.00044, seat + 0.00041), (dome + 0.00031, seat + 0.00035),
        (dome + 0.00015, seat + 0.00011), (dome + 0.00008, seat - 0.00025),
        (dome - 0.00019, seat - 0.00039),
    ], metal, segments=segments, closed=True, knurl_rows=knurl_rows)
    gasket = lathe(f"{name}_LensRetainer", [
        (dome - 0.00016, seat - 0.00035), (dome + 0.00018, seat - 0.00035),
        (dome + 0.00022, seat - 0.00012), (dome + 0.00012, seat + 0.000045),
        (dome - 0.000065, seat + 0.000045),
    ], mats["Lamp_Retainer"], closed=True)
    lens = glass_dome(
        f"{name}_Glass", dome, h - seat, seat, 0.00065 if ultra else 0.00074, mats[glass_name],
        faceted=not ultra,
    )

    shoulder = dome * (0.66 if ultra else 0.56)
    reflector = lathe(f"{name}_OpticalCup", [
        (0, 0.00011), (dome * 0.92, 0.00011), (dome * 0.95, 0.00022),
        (dome * 0.95, seat - 0.00045), (dome * 0.87, seat - 0.00040),
        (dome * 0.80, seat - 0.00059), (dome * 0.78, seat - 0.00085),
        (dome * 0.73, seat - 0.00087), (dome * 0.70, seat - 0.00094),
        (shoulder, seat - 0.00166), (shoulder * 0.90, seat - 0.00182),
        (shoulder * 0.87, seat - 0.00201), (shoulder * 0.72, seat - 0.00204),
        (shoulder * 0.59, seat - 0.00257), (dome * 0.26, seat - 0.00268),
        (0, seat - 0.00269),
    ], mats["Ultra_Reflector" if ultra else "Jewel_Reflector"])
    parts = [ring, gasket, lens, reflector]
    if ultra:
        parts.extend(build_ultra_bulb(dome, seat, mats))
    else:
        socket = lathe(f"{name}_DarkRubySocket", [
            (0, seat - 0.00272), (dome * 0.32, seat - 0.00272),
            (dome * 0.35, seat - 0.00232), (dome * 0.32, seat - 0.00222),
            (dome * 0.23, seat - 0.00218), (dome * 0.19, seat - 0.00230),
            (dome * 0.11, seat - 0.00244), (0, seat - 0.00245),
        ], mats["Jewel_Socket"])
        parts.append(socket)
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
    bulb_z = seat - 0.00008
    scale = 1.35
    r = dome * 0.31 * scale
    rise = dome * 0.72 * scale
    bulb = glass_dome("Ultra_ClearBulbEnvelope", r, rise, 0, 0.00016 * scale, mats["Ultra_Bulb_Glass"], segments=128)
    bulb.scale.y = 0.75
    bulb.rotation_euler.x = -math.pi / 2
    bulb.location = (0, bulb_y, bulb_z)

    base = lathe("Ultra_BulbBrassBase", [
        (radius * r, depth * scale)
        for radius, depth in (
            (0, -0.00128), (0.71, -0.00128), (0.81, -0.00119), (0.84, -0.00105),
            (0.79, -0.00092), (0.79, -0.00082), (0.84, -0.00073),
            (0.87, -0.00025), (0.82, -0.00009), (0, -0.00009),
        )
    ], mats["Ultra_Brass_Cup"], segments=96)
    base.location = bulb.location
    base.rotation_euler = bulb.rotation_euler
    base.scale.y = bulb.scale.y

    tungsten = mats["Ultra_Filament"]
    end_x = 0.00071 * scale
    filament_y = bulb_y + rise * 0.57
    supports = [
        wire(f"Ultra_Support_{label}", [
            (sign * 0.00038 * scale, bulb_y - 0.00014 * scale, bulb_z),
            (sign * 0.00050 * scale, bulb_y + 0.00094 * scale, bulb_z),
            (sign * 0.00065 * scale, filament_y - 0.00046 * scale, bulb_z),
            (sign * end_x, filament_y, bulb_z),
        ], 0.000062 * scale, tungsten)
        for sign, label in ((-1, "L"), (1, "R"))
    ]
    path = []
    for i in range(201):
        t = i / 200
        phase = math.tau * 12 * t
        coil = 0.000063 * scale * min(1, 25 * t, 25 * (1 - t))
        path.append((
            end_x * (2 * t - 1),
            filament_y + 0.00016 * scale * math.sin(math.pi * t) + coil * math.sin(phase),
            bulb_z + coil * math.cos(phase),
        ))
    filament = wire("Ultra_CoiledTungstenFilament", path, 0.000026 * scale, tungsten, sides=8)
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
    parser.add_argument("--out", type=Path)
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
    out = output_dir(parser.parse_args(argv).out)
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
