#!/usr/bin/env python3
"""Build closed-manifold Y-up metric GLB models for CA-72's Minimoog cabinet & fixtures:
  - cabinet.glb         (walnut top rail, left/right walnut end cheeks, bottom rail, brushed alu trim)
  - name_plate.glb      (anodized black CA-72/IDLE FOUNDRY badge with bright machined silver chamfer)
  - jewel_power.glb     (turned knurled chrome bezel + faceted ruby-red glass jewel dome)
  - jewel_overload.glb  (turned knurled chrome bezel + darker faceted ruby glass jewel dome)
  - ultra_bezel.glb     (knurled chrome bezel + recessed dark smoked-amber domed glass lens)
  - jack_quarter.glb    (1/4" chrome phone jack: washer, hex nut, threaded collar, deep bore)
  - screw_phillips.glb  (black-oxide Phillips pan-head wood/chassis screw)
"""

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import bmesh
import numpy as np


def srgb_to_lin(c: float) -> float:
    return ((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92


def make_material(name, rgba, roughness, metallic=0.0, coat=0.0, coat_rough=0.14):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    mat.diffuse_color = rgba
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = rgba
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    if coat > 0.0 and "Coat Weight" in bsdf.inputs:
        bsdf.inputs["Coat Weight"].default_value = coat
        bsdf.inputs["Coat Roughness"].default_value = coat_rough
    return mat


def validate_mesh(obj):
    bm = bmesh.new()
    try:
        bm.from_mesh(obj.data)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        non_man = [e for e in bm.edges if not e.is_manifold or not e.is_contiguous]
        if non_man:
            raise ValueError(f"{obj.name}: {len(non_man)} non-manifold edges")
        vol = bm.calc_volume(signed=True)
        if vol <= 0:
            raise ValueError(f"{obj.name}: non-positive volume {vol}")
        bm.to_mesh(obj.data)
    finally:
        bm.free()
    obj.data.update()


def add_planar_uv(obj, axis_u=0, axis_v=1, scale_u=1.0, scale_v=1.0, off_u=0.0, off_v=0.0):
    mesh = obj.data
    uv_layer = mesh.uv_layers.new(name="UVMap")
    for poly in mesh.polygons:
        for li in poly.loop_indices:
            v = mesh.vertices[mesh.loops[li].vertex_index].co
            coords = (v.x, v.y, v.z)
            u = coords[axis_u] * scale_u + off_u
            w = coords[axis_v] * scale_v + off_v
            uv_layer.data[li].uv = (u, w)


def make_beveled_box(name, size, center, mat, bevel_w=0.0012, segs=4):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=center)
    obj = bpy.context.active_object
    obj.name = name
    obj.dimensions = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel_w > 0:
        mod = obj.modifiers.new("Bevel", "BEVEL")
        mod.width = bevel_w
        mod.segments = segs
        mod.limit_method = "ANGLE"
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=mod.name)
    for p in obj.data.polygons:
        p.use_smooth = True
    mod_wn = obj.modifiers.new("WN", "WEIGHTED_NORMAL")
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=mod_wn.name)
    obj.data.materials.append(mat)
    validate_mesh(obj)
    return obj


def build_cabinet(spec, mat_walnut_h, mat_walnut_v, mat_trim, mat_backing):
    """Coordinate system:
      Origin (0, 0, 0) is the center of the FULL cabinet footprint (total_w x total_h) at faceplate Z=0.
      In panel units:
        total_w = cheek_w + face_w + cheek_w
        total_h = wood_top_h + face_h + bottom_rail_h
    """
    pm = spec["panel_meters"]
    pu = spec["panel_units"]
    s = spec["scale_m_per_unit"]

    face_w = pm["face_w"]
    face_h = pm["face_h"]
    top_h = pm["wood_top_h"]
    cheek_w = pm["cheek_w"]
    bot_h = pm["bottom_rail_h"]
    tot_w = pm["total_w"]
    tot_h = pm["total_h"]
    depth = pm["cabinet_depth"]

    # Y coordinates (Blender +Y is UP on screen, -Y is DOWN)
    y_top_edge = tot_h * 0.5
    y_face_top = y_top_edge - top_h
    y_face_bot = y_face_top - face_h
    y_bot_edge = -tot_h * 0.5

    parts = []

    # 1. Backing chassis plate behind faceplate
    backing = make_beveled_box(
        "Chassis_Backing",
        (face_w + 0.004, face_h + 0.004, depth * 0.5),
        (0.0, 0.5 * (y_face_top + y_face_bot), -depth * 0.25 - 0.0008),
        mat_backing,
        bevel_w=0.0,
    )
    parts.append(backing)

    # 2. Top walnut rail (above faceplate, rises +0.0075 m above Z=0 to cast subtle top shadow)
    top_z_rise = 0.0072
    top_rail = make_beveled_box(
        "Walnut_TopRail",
        (face_w + 0.001, top_h, depth + top_z_rise),
        (0.0, y_face_top + top_h * 0.5, (top_z_rise - depth) * 0.5),
        mat_walnut_h,
        bevel_w=0.0016,
        segs=4,
    )
    add_planar_uv(top_rail, 0, 1, 1.0 / tot_w, 1.0 / (top_h * 1.8), 0.5, 0.35)
    parts.append(top_rail)

    # 3. Bottom walnut rail
    bot_z_rise = 0.0052
    bot_rail = make_beveled_box(
        "Walnut_BottomRail",
        (face_w + 0.001, bot_h, depth + bot_z_rise),
        (0.0, y_bot_edge + bot_h * 0.5, (bot_z_rise - depth) * 0.5),
        mat_walnut_h,
        bevel_w=0.0012,
        segs=4,
    )
    add_planar_uv(bot_rail, 0, 1, 1.0 / tot_w, 1.0 / (top_h * 1.8), 0.5, 0.72)
    parts.append(bot_rail)

    # 4. Left & right walnut end cheeks (full height, classic Minimoog contoured cheeks!)
    cheek_z_rise = 0.0082
    for side, label, uv_off in ((-1.0, "Left", 0.20), (1.0, "Right", 0.65)):
        cx = side * (face_w * 0.5 + cheek_w * 0.5)
        cheek = make_beveled_box(
            f"Walnut_Cheek_{label}",
            (cheek_w, tot_h, depth + cheek_z_rise),
            (cx, 0.0, (cheek_z_rise - depth) * 0.5),
            mat_walnut_v,
            bevel_w=0.0018,
            segs=5,
        )
        # Grain runs vertically along the cheeks (Y -> U, X -> V)
        add_planar_uv(cheek, 1, 0, 1.0 / (tot_h * 1.3), 1.0 / (cheek_w * 4.0), 0.5, uv_off)
        parts.append(cheek)

    # 5. Authentic brushed silver-aluminum inner trim strips along left, right, and bottom of faceplate
    trim_l_w = pu["trim_left_w"] * s
    trim_r_w = pu["trim_right_w"] * s
    trim_b_h = pu["trim_bottom_h"] * s
    trim_z = 0.00095

    parts.append(
        make_beveled_box(
            "Trim_Left",
            (trim_l_w, face_h, trim_z),
            (-face_w * 0.5 + trim_l_w * 0.5, 0.5 * (y_face_top + y_face_bot), trim_z * 0.5),
            mat_trim,
            bevel_w=0.00022,
            segs=2,
        )
    )
    parts.append(
        make_beveled_box(
            "Trim_Right",
            (trim_r_w, face_h, trim_z),
            (face_w * 0.5 - trim_r_w * 0.5, 0.5 * (y_face_top + y_face_bot), trim_z * 0.5),
            mat_trim,
            bevel_w=0.00022,
            segs=2,
        )
    )
    parts.append(
        make_beveled_box(
            "Trim_Bottom",
            (face_w, trim_b_h, trim_z),
            (0.0, y_face_bot + trim_b_h * 0.5, trim_z * 0.5),
            mat_trim,
            bevel_w=0.00022,
            segs=2,
        )
    )

    return parts


def build_name_plate(dim, mat_face, mat_chamfer):
    """Build CA-72 / IDLE FOUNDRY brushed black anodized badge with bright machined chamfer."""
    w, h, t = dim["w"], dim["h"], dim["thickness"]
    ch = 0.00080
    hw, hh = w * 0.5, h * 0.5

    def rect_loop(rx, ry, z, rc=0.0007):
        pts = []
        corners = [(rx - rc, ry - rc, 0),
                   (-rx + rc, ry - rc, math.pi * 0.5),
                   (-rx + rc, -ry + rc, math.pi),
                   (rx - rc, -ry + rc, math.pi * 1.5)]
        for cx, cy, a0 in corners:
            for k in range(3):
                a = a0 + (math.pi * 0.5) * k / 2.0
                pts.append((cx + rc * math.cos(a), cy + rc * math.sin(a), z))
        return pts

    rings = [
        rect_loop(hw, hh, 0.0, 0.0009),
        rect_loop(hw, hh, t - ch, 0.0009),
        rect_loop(hw - ch, hh - ch, t, 0.0005),
    ]
    n = len(rings[0])
    verts = [p for ring in rings for p in ring]
    bc = len(verts); verts.append((0.0, 0.0, 0.0))
    tc = len(verts); verts.append((0.0, 0.0, t))
    faces, fmats = [], []
    for i in range(n):
        ni = (i + 1) % n
        faces.append((bc, ni, i)); fmats.append(1)
    for r in range(len(rings) - 1):
        a0, b0 = r * n, (r + 1) * n
        for i in range(n):
            ni = (i + 1) % n
            faces.append((a0 + i, a0 + ni, b0 + ni, b0 + i)); fmats.append(1)
    last = (len(rings) - 1) * n
    for i in range(n):
        ni = (i + 1) % n
        faces.append((tc, last + i, last + ni)); fmats.append(0)

    mesh = bpy.data.meshes.new("NamePlate")
    mesh.from_pydata(verts, [], faces)
    mesh.materials.append(mat_face)
    mesh.materials.append(mat_chamfer)
    for p in mesh.polygons:
        p.use_smooth = False
        p.material_index = fmats[p.index]
    obj = bpy.data.objects.new("NamePlate", mesh)
    bpy.context.collection.objects.link(obj)
    validate_mesh(obj)
    # Exact planar UV across full w x h for plate_albedo.png
    add_planar_uv(obj, 0, 1, 1.0 / w, 1.0 / h, 0.5, 0.5)
    return obj


def build_jewel_lamp(name, dim, mat_chrome, mat_glass, faceted=True):
    """Build Dialco-style turned chrome bezel + faceted ruby-red (or amber) jewel dome."""
    r_bez = dim["bezel_radius"]
    r_dome = dim["dome_radius"]
    h = dim["height"]

    # 1. Knurled chrome bezel ring (64 segments with 32 subtle knurl ridges)
    segs = 64
    knurl_amp = r_bez * 0.022
    bez_prof = [
        (r_bez * 0.96, 0.0000, 0.0),
        (r_bez,        0.0004, 1.0),
        (r_bez,        h * 0.36, 1.0),
        (r_bez * 0.92, h * 0.52, 0.0),
        (r_dome * 1.06, h * 0.54, 0.0),
        (r_dome * 0.98, h * 0.38, 0.0),
        (r_dome * 0.90, 0.0005, 0.0),
    ]
    verts = []
    for r, z, kn in bez_prof:
        for i in range(segs):
            a = math.tau * i / segs
            rr = r + kn * knurl_amp * math.cos(32.0 * a)
            verts.append((rr * math.cos(a), rr * math.sin(a), z))
    bc = len(verts); verts.append((0.0, 0.0, 0.0))
    tc = len(verts); verts.append((0.0, 0.0, 0.0005))
    faces = []
    for i in range(segs):
        ni = (i + 1) % segs
        faces.append((bc, ni, i))
    for r in range(len(bez_prof) - 1):
        a0, b0 = r * segs, (r + 1) * segs
        for i in range(segs):
            ni = (i + 1) % segs
            faces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(bez_prof) - 1) * segs
    for i in range(segs):
        ni = (i + 1) % segs
        faces.append((tc, last + i, last + ni))
    bezel = bpy.data.objects.new(f"{name}_Bezel", bpy.data.meshes.new(f"{name}_Bezel"))
    bezel.data.from_pydata(verts, [], faces)
    bezel.data.materials.append(mat_chrome)
    for p in bezel.data.polygons:
        p.use_smooth = True
    bpy.context.collection.objects.link(bezel)
    validate_mesh(bezel)

    # 2. Faceted ruby jewel dome (12-sided cut glass facets for POWER/OVERLOAD, smooth dome for ULTRA)
    d_segs = 12 if faceted else 36
    z0 = h * 0.32
    d_prof = [
        (r_dome * 0.98, z0 - 0.0010),
        (r_dome * 1.00, z0),
        (r_dome * 0.92, z0 + (h - z0) * 0.36),
        (r_dome * 0.70, z0 + (h - z0) * 0.72),
        (r_dome * 0.38, z0 + (h - z0) * 0.94),
    ]
    dverts = [
        (r * math.cos(math.tau * i / d_segs), r * math.sin(math.tau * i / d_segs), z)
        for (r, z) in d_prof
        for i in range(d_segs)
    ]
    dbc = len(dverts); dverts.append((0.0, 0.0, z0 - 0.0010))
    dtc = len(dverts); dverts.append((0.0, 0.0, h))
    dfaces = []
    for i in range(d_segs):
        ni = (i + 1) % d_segs
        dfaces.append((dbc, ni, i))
    for r in range(len(d_prof) - 1):
        a0, b0 = r * d_segs, (r + 1) * d_segs
        for i in range(d_segs):
            ni = (i + 1) % d_segs
            dfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(d_prof) - 1) * d_segs
    for i in range(d_segs):
        ni = (i + 1) % d_segs
        dfaces.append((dtc, last + i, last + ni))
    dome = bpy.data.objects.new(f"{name}_Dome", bpy.data.meshes.new(f"{name}_Dome"))
    dome.data.from_pydata(dverts, [], dfaces)
    dome.data.materials.append(mat_glass)
    for p in dome.data.polygons:
        p.use_smooth = not faceted
    bpy.context.collection.objects.link(dome)
    validate_mesh(dome)

    bpy.ops.object.select_all(action="DESELECT")
    bezel.select_set(True)
    dome.select_set(True)
    bpy.context.view_layer.objects.active = bezel
    bpy.ops.object.join()
    bezel.name = name
    return bezel


def build_jack_quarter(dim, mat_chrome, mat_chrome_satin, mat_bore):
    """Build authentic 1/4-inch Switchcraft chrome phone jack."""
    r_out = dim["outer_radius"]
    r_hex = dim["nut_across_flats"] / math.sqrt(3.0)
    r_slv = dim["sleeve_radius"]
    r_bore = dim["bore_radius"]
    h = dim["height"]

    parts = []
    segs = 48

    # 1. Flat chrome washer (Z = 0..0.0007)
    w_prof = [(r_out * 0.98, 0.0), (r_out, 0.00015), (r_out, 0.00060), (r_out * 0.96, 0.00072), (r_slv, 0.00072)]
    wv = [(r * math.cos(math.tau * i / segs), r * math.sin(math.tau * i / segs), z) for r, z in w_prof for i in range(segs)]
    wbc = len(wv); wv.append((0.0, 0.0, 0.0))
    wtc = len(wv); wv.append((0.0, 0.0, 0.00072))
    wf = [(wbc, (i + 1) % segs, i) for i in range(segs)]
    for r in range(len(w_prof) - 1):
        a0, b0 = r * segs, (r + 1) * segs
        for i in range(segs):
            ni = (i + 1) % segs
            wf.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(w_prof) - 1) * segs
    wf.extend((wtc, last + i, last + (i + 1) % segs) for i in range(segs))
    w_obj = bpy.data.objects.new("Jack_Washer", bpy.data.meshes.new("Jack_Washer"))
    w_obj.data.from_pydata(wv, [], wf)
    w_obj.data.materials.append(mat_chrome_satin)
    for p in w_obj.data.polygons: p.use_smooth = True
    bpy.context.collection.objects.link(w_obj)
    validate_mesh(w_obj)
    parts.append(w_obj)

    # 2. Chamfered chrome hex nut (Z = 0.0007 .. 0.0031)
    def hex_r(rad, z):
        return [(rad * math.cos(math.radians(20.0 + 60.0 * k)), rad * math.sin(math.radians(20.0 + 60.0 * k)), z) for k in range(6)]
    h_rings = [hex_r(r_hex * 0.97, 0.0007), hex_r(r_hex, 0.0010), hex_r(r_hex, 0.0027), hex_r(r_hex * 0.90, 0.0031), hex_r(r_slv * 0.95, 0.0031)]
    hv = [p for ring in h_rings for p in ring]
    hbc = len(hv); hv.append((0.0, 0.0, 0.0007))
    htc = len(hv); hv.append((0.0, 0.0, 0.0031))
    hf = [(hbc, (i + 1) % 6, i) for i in range(6)]
    for r in range(len(h_rings) - 1):
        a0, b0 = r * 6, (r + 1) * 6
        for i in range(6):
            ni = (i + 1) % 6
            hf.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(h_rings) - 1) * 6
    hf.extend((htc, last + i, last + (i + 1) % 6) for i in range(6))
    h_obj = bpy.data.objects.new("Jack_Nut", bpy.data.meshes.new("Jack_Nut"))
    h_obj.data.from_pydata(hv, [], hf)
    h_obj.data.materials.append(mat_chrome)
    for p in h_obj.data.polygons: p.use_smooth = False
    bpy.context.collection.objects.link(h_obj)
    validate_mesh(h_obj)
    parts.append(h_obj)

    # 3. Threaded chrome sleeve + deep dark 1/4" bore
    s_prof = [
        (r_slv,          0.0029, 0),
        (r_slv * 0.96,   0.0034, 0),
        (r_slv,          0.0039, 0),
        (r_slv * 0.96,   0.0044, 0),
        (r_slv * 0.98,   h - 0.0004, 0),
        (r_slv * 0.88,   h, 0),
        (r_bore * 1.12,  h, 0),
        (r_bore,         h - 0.0005, 1),
        (r_bore,         0.0005, 1),
    ]
    sv = [(r * math.cos(math.tau * i / segs), r * math.sin(math.tau * i / segs), z) for r, z, _ in s_prof for i in range(segs)]
    sbc = len(sv); sv.append((0.0, 0.0, 0.0029))
    stc = len(sv); sv.append((0.0, 0.0, 0.0005))
    sf, sfm = [], []
    for i in range(segs):
        sf.append((sbc, (i + 1) % segs, i)); sfm.append(0)
    for r in range(len(s_prof) - 1):
        a0, b0 = r * segs, (r + 1) * segs
        m = s_prof[r][2]
        for i in range(segs):
            ni = (i + 1) % segs
            sf.append((a0 + i, a0 + ni, b0 + ni, b0 + i)); sfm.append(m)
    last = (len(s_prof) - 1) * segs
    for i in range(segs):
        sf.append((stc, last + i, last + (i + 1) % segs)); sfm.append(1)
    s_obj = bpy.data.objects.new("Jack_Sleeve", bpy.data.meshes.new("Jack_Sleeve"))
    s_obj.data.from_pydata(sv, [], sf)
    s_obj.data.materials.append(mat_chrome)
    s_obj.data.materials.append(mat_bore)
    for p in s_obj.data.polygons:
        p.use_smooth = True
        p.material_index = sfm[p.index]
    bpy.context.collection.objects.link(s_obj)
    validate_mesh(s_obj)
    parts.append(s_obj)

    bpy.ops.object.select_all(action="DESELECT")
    for o in parts: o.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.join()
    parts[0].name = "Jack_Quarter"
    return parts[0]


def build_screw_phillips(dim, mat_screw):
    """Build closed-manifold black-oxide Phillips pan-head screw."""
    radius = dim["head_radius"]
    height = dim["height"]
    arm = radius * 0.62
    hw = radius * 0.165
    fillet = radius * 0.032

    corners = [(-hw, -arm), (hw, -arm), (hw, -hw), (arm, -hw), (arm, hw), (hw, hw),
               (hw, arm), (-hw, arm), (-hw, hw), (-arm, hw), (-arm, -hw), (-hw, -hw)]
    corners = [np.array(p, dtype=float) for p in corners]
    outline = []
    for i, c in enumerate(corners):
        inc = corners[i - 1] - c
        out = corners[(i + 1) % len(corners)] - c
        p0 = c + inc * (fillet / np.linalg.norm(inc))
        p1 = c + out * (fillet / np.linalg.norm(out))
        for t in (0.0, 0.5, 1.0):
            outline.append((1 - t) ** 2 * p0 + 2 * (1 - t) * t * c + t * t * p1)

    turn = math.tau
    corner_angles = {math.atan2(y, x) % turn for x, y in outline}
    circle_angles = (i * turn / 96 for i in range(96))
    angles = sorted(corner_angles | {a for a in circle_angles if all(abs(math.remainder(a - b, turn)) >= 0.004 for b in corner_angles)})
    dirs = [(math.cos(a), math.sin(a)) for a in angles]

    def recess_r(dx, dy):
        for i, (ax, ay) in enumerate(outline):
            bx, by = outline[(i + 1) % len(outline)]
            ex, ey = bx - ax, by - ay
            denom = dx * ey - dy * ex
            if abs(denom) < 1e-12:
                continue
            et = (ax * dy - ay * dx) / denom
            if -1e-6 <= et <= 1.0 + 1e-6:
                l = (ax * ey - ay * ex) / denom
                if l > 0:
                    return l
        return hw

    rrs = [recess_r(dx, dy) for dx, dy in dirs]
    dome = lambda r: height - height * 0.12 * (max(0.0, r - radius * 0.25) / radius) ** 2

    rings = [
        [(r * dx, r * dy, z) for dx, dy in dirs]
        for r, z in (
            (radius * 0.94, 0.0),
            (radius, height * 0.12),
            (radius, height * 0.30),
            (radius * 0.93, height * 0.58),
            (radius * 0.78, dome(radius * 0.78)),
        )
    ]
    for blend in (0.35, 0.72, 1.0):
        rads = [radius * 0.78 * (1 - blend) + rr * blend for rr in rrs]
        rings.append([(r * dx, r * dy, dome(r)) for r, (dx, dy) in zip(rads, dirs)])
    for sc, drop in ((0.94, height * 0.08), (0.72, height * 0.45), (0.60, height * 0.65)):
        rings.append([(rr * sc * dx, rr * sc * dy, max(height * 0.28, dome(rr) - drop)) for rr, (dx, dy) in zip(rrs, dirs)])

    cnt = len(dirs)
    verts = [p for ring in rings for p in ring]
    bc = len(verts); verts.append((0.0, 0.0, 0.0))
    fc = len(verts); verts.append((0.0, 0.0, height * 0.28))
    faces = []
    for r in range(len(rings) - 1):
        a0, b0 = r * cnt, (r + 1) * cnt
        for i in range(cnt):
            ni = (i + 1) % cnt
            faces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(rings) - 1) * cnt
    for i in range(cnt):
        ni = (i + 1) % cnt
        faces.append((bc, ni, i))
        faces.append((fc, last + i, last + ni))
    obj = bpy.data.objects.new("Screw_Phillips", bpy.data.meshes.new("Screw_Phillips"))
    obj.data.from_pydata(verts, [], faces)
    obj.data.materials.append(mat_screw)
    for p in obj.data.polygons: p.use_smooth = len(p.vertices) == 4
    bpy.context.collection.objects.link(obj)
    validate_mesh(obj)
    return obj


def export_glb(objs, out_path: Path):
    if not isinstance(objs, (list, tuple)):
        objs = [objs]
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    res = bpy.ops.export_scene.gltf(
        filepath=str(out_path),
        export_format="GLB",
        export_yup=True,
        export_materials="EXPORT",
        export_normals=True,
        use_selection=True,
    )
    if res != {"FINISHED"}:
        raise RuntimeError(f"Failed to export {out_path}")
    print(f"Exported {out_path.name} ({out_path.stat().st_size:,} bytes)", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=Path("target/panel-3d"))
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
    args = parser.parse_args(argv)

    spec = json.loads((args.out / "spec.json").read_text())
    dims = spec["part_dimensions_meters"]
    assets_dir = args.out / "assets"
    assets_dir.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.unit_settings.system = "METRIC"

    mat_walnut_h = make_material("Walnut_Horiz", (0.14, 0.06, 0.025, 1.0), roughness=0.35, coat=0.22, coat_rough=0.22)
    mat_walnut_v = make_material("Walnut_Vert", (0.13, 0.055, 0.022, 1.0), roughness=0.36, coat=0.22, coat_rough=0.22)
    mat_trim = make_material("Aluminum_Trim", (0.76, 0.77, 0.75, 1.0), roughness=0.26, metallic=0.92)
    mat_backing = make_material("Steel_Backing", (0.015, 0.015, 0.017, 1.0), roughness=0.55, metallic=0.2)

    mat_plate_face = make_material("Plate_Face", (0.02, 0.02, 0.022, 1.0), roughness=0.32, metallic=0.35)
    mat_plate_chamfer = make_material("Plate_Chamfer", (0.85, 0.86, 0.84, 1.0), roughness=0.16, metallic=0.96)

    mat_chrome = make_material("Chrome_Polished", (0.86, 0.86, 0.84, 1.0), roughness=0.11, metallic=0.98)
    mat_chrome_satin = make_material("Chrome_Satin", (0.76, 0.76, 0.74, 1.0), roughness=0.24, metallic=0.95)
    mat_bore = make_material("Jack_Bore", (0.002, 0.002, 0.003, 1.0), roughness=0.92)

    # Physical unlit translucent faceted jewel glass (Rust linear-light shader illuminates them!)
    mat_ruby = make_material("Jewel_Ruby_Unlit", (0.12, 0.007, 0.005, 1.0), roughness=0.10, coat=0.75, coat_rough=0.05)
    mat_ruby_dark = make_material("Jewel_Overload_Unlit", (0.045, 0.004, 0.003, 1.0), roughness=0.10, coat=0.75, coat_rough=0.05)
    mat_amber_dark = make_material("Ultra_Amber_Unlit", (0.055, 0.025, 0.005, 1.0), roughness=0.12, coat=0.65, coat_rough=0.06)

    mat_screw = make_material("Screw_BlackOxide", (0.025, 0.024, 0.026, 1.0), roughness=0.32, metallic=0.82)

    cab = build_cabinet(spec, mat_walnut_h, mat_walnut_v, mat_trim, mat_backing)
    export_glb(cab, assets_dir / "cabinet.glb")
    for o in cab: bpy.data.objects.remove(o, do_unlink=True)

    plate = build_name_plate(dims["name_plate"], mat_plate_face, mat_plate_chamfer)
    export_glb(plate, assets_dir / "name_plate.glb")
    bpy.data.objects.remove(plate, do_unlink=True)

    jp = build_jewel_lamp("Jewel_Power", dims["jewel_power"], mat_chrome, mat_ruby, faceted=True)
    export_glb(jp, assets_dir / "jewel_power.glb")
    bpy.data.objects.remove(jp, do_unlink=True)

    jo = build_jewel_lamp("Jewel_Overload", dims["jewel_overload"], mat_chrome, mat_ruby_dark, faceted=True)
    export_glb(jo, assets_dir / "jewel_overload.glb")
    bpy.data.objects.remove(jo, do_unlink=True)

    ub = build_jewel_lamp("Ultra_Bezel", dims["ultra_lamp"], mat_chrome, mat_amber_dark, faceted=False)
    export_glb(ub, assets_dir / "ultra_bezel.glb")
    bpy.data.objects.remove(ub, do_unlink=True)

    jk = build_jack_quarter(dims["jack_quarter"], mat_chrome, mat_chrome_satin, mat_bore)
    export_glb(jk, assets_dir / "jack_quarter.glb")
    bpy.data.objects.remove(jk, do_unlink=True)

    sc = build_screw_phillips(dims["screw_phillips"], mat_screw)
    export_glb(sc, assets_dir / "screw_phillips.glb")
    bpy.data.objects.remove(sc, do_unlink=True)


if __name__ == "__main__":
    main()
