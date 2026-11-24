#!/usr/bin/env python3
"""Build closed-manifold Y-up metric GLB models for CA-72's classic Minimoog knobs:
  - knob_std.glb           (9-flute skirted knob, spun-aluminum cap, white skirt dot)
  - knob_big.glb           (large OSC 2/3 Frequency knob, wider spun-aluminum cap)
  - knob_pointer.glb       (classic Minimoog wedge-pointer rotary selector knob)
  - knob_pointer_small.glb (0.75x wedge-pointer for FILTER POLES & FILTER MODE)
"""

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import bmesh


SEGMENTS_PER_FLUTE = 14


def srgb_to_lin(c: float) -> float:
    return ((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92


def make_material(name, rgba, roughness, metallic=0.0, coat=0.0, coat_rough=0.14, spec=0.5, aniso=0.0):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    mat.diffuse_color = rgba
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = rgba
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    if "Specular IOR Level" in bsdf.inputs:
        bsdf.inputs["Specular IOR Level"].default_value = spec
    if coat > 0.0 and "Coat Weight" in bsdf.inputs:
        bsdf.inputs["Coat Weight"].default_value = coat
        bsdf.inputs["Coat Roughness"].default_value = coat_rough
    if aniso > 0.0 and "Anisotropic" in bsdf.inputs:
        bsdf.inputs["Anisotropic"].default_value = aniso
    return mat


def validate_and_create_obj(name, vertices, faces, materials, face_materials=None, smooth=True):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    if mesh.validate():
        raise ValueError(f"{name}: invalid mesh")
    for m in materials:
        mesh.materials.append(m)
    for poly in mesh.polygons:
        poly.use_smooth = smooth
        if face_materials is not None:
            poly.material_index = face_materials[poly.index]

    bm = bmesh.new()
    try:
        bm.from_mesh(mesh)
        bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
        non_man = [e for e in bm.edges if not e.is_manifold or not e.is_contiguous]
        if non_man:
            raise ValueError(f"{name}: {len(non_man)} non-manifold edges")
        vol = bm.calc_volume(signed=True)
        if vol <= 0:
            raise ValueError(f"{name}: non-positive volume {vol}")
        bm.to_mesh(mesh)
    finally:
        bm.free()

    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    return obj


def flute_xy(radius, flute_depth, count, segs_per_flute, index):
    """Classic Minimoog broad concave 9-flute grip: crest at 12 o'clock (+Y)."""
    angle = math.tau * index / count
    phase = math.tau * index / segs_per_flute
    # Broad smooth scallop trough between rounded crests (characteristic Moog profile)
    trough = (0.5 - 0.5 * math.cos(phase)) ** 1.35
    r = radius - flute_depth * trough
    # index=0 at +Y (12 o'clock), going clockwise/CCW consistently
    return -r * math.sin(angle), r * math.cos(angle)


def build_skirted_knob(name, dim, mat_body, mat_cap_chamfer, mat_cap_face, mat_white):
    """Build classic Minimoog Model D fluted skirted knob with spun-aluminum insert cap."""
    r_base = dim["base_radius"]
    r_cap = dim["cap_radius"]
    h = dim["height"]
    flutes = dim["flute_count"]
    segs = flutes * SEGMENTS_PER_FLUTE

    # Fluted body crest radius tapers gently from flared skirt up to top crown
    r_grip_bot = r_base * 0.925
    r_grip_top = r_cap + max(0.00165, (r_base - r_cap) * 0.48)
    depth = 0.118 * r_base

    # Profile stations: (crest_radius, z, flute_depth, mat_idx_above)
    # mat_idx: 0 = satin black thermoset, 1 = bright machined aluminum chamfer, 2 = spun aluminum face
    profile = [
        (r_base - 0.00025, 0.0000, 0.0, 0),            # 0: seated base chamfer
        (r_base,           0.0003, 0.0, 0),            # 1: flared skirt outer rim bottom
        (r_base,           0.0014, 0.0, 0),            # 2: flared skirt outer rim top
        (r_base - 0.00042, 0.0022, 0.0, 0),            # 3: flared skirt bevel inward
        (r_grip_bot,       0.0029, 0.10 * depth, 0),   # 4: start of flutes
        (r_grip_bot,       0.0046, 0.88 * depth, 0),   # 5: full flute depth lower
        (0.55 * r_grip_bot + 0.45 * r_grip_top, 0.56 * h, 1.00 * depth, 0),  # 6: mid barrel
        (r_grip_top + 0.00022, h - 0.0026, 0.90 * depth, 0),  # 7: upper flute
        (r_grip_top,           h - 0.0010, 0.30 * depth, 0),  # 8: flute fadeout shoulder
        (r_grip_top - 0.00032, h - 0.00038, 0.0, 0),   # 9: black top crown rim
        (r_cap + 0.00024,      h - 0.00038, 0.0, 0),   # 10: black recess lip
        (r_cap + 0.00016,      h - 0.00072, 0.0, 0),   # 11: cap seat groove
        # Spun aluminum insert cap: crisp bright 45-deg machined chamfer + flat top
        (r_cap,                h - 0.00062, 0.0, 1),   # 12: cap outer wall
        (r_cap - 0.00005,      h - 0.00026, 0.0, 1),   # 13: cap bevel outer
        (r_cap - 0.00060,      h,           0.0, 2),   # 14: cap bevel inner (bright rim!)
        (r_cap * 0.65,         h + 0.00003, 0.0, 2),   # 15: subtle lathe crown
        (r_cap * 0.22,         h + 0.00005, 0.0, 2),   # 16: inner disc
    ]

    verts = [
        (*flute_xy(r, d, segs, SEGMENTS_PER_FLUTE, i), z)
        for (r, z, d, _) in profile
        for i in range(segs)
    ]
    # Add bottom & top center vertices for clean triangle fans (no ngons!)
    bot_c = len(verts)
    verts.append((0.0, 0.0, 0.0))
    top_c = len(verts)
    verts.append((0.0, 0.0, h + 0.00007))

    faces = []
    fmats = []
    # Bottom fan (outward = -Z)
    for i in range(segs):
        ni = (i + 1) % segs
        faces.append((bot_c, ni, i))
        fmats.append(0)
    # Side rings
    for ring in range(len(profile) - 1):
        a0 = ring * segs
        b0 = (ring + 1) * segs
        m_idx = profile[ring][3]
        for i in range(segs):
            ni = (i + 1) % segs
            faces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
            fmats.append(m_idx)
    # Top fan (outward = +Z)
    last = (len(profile) - 1) * segs
    for i in range(segs):
        ni = (i + 1) % segs
        faces.append((top_c, last + i, last + ni))
        fmats.append(2)

    body = validate_and_create_obj(
        f"{name}_body",
        verts,
        faces,
        [mat_body, mat_cap_chamfer, mat_cap_face],
        fmats,
    )

    # High-contrast crisp white pointer dot cleanly centered on the black shoulder/skirt
    # outside the aluminum cap so it forms a complete crisp circle with zero clipping!
    dot_r = min(dim["dot_radius"], (r_base - r_cap - 0.00065) * 0.42)
    dot_y_final = r_cap + 0.00038 + dot_r + 0.35 * max(0.0, (r_base - r_cap - 0.00075 - 2.0 * dot_r))
    # Sit just proud of the sloped shoulder crest at 12 o'clock (+Y)
    dot_z_top = h - 0.00015
    dot_segs = 32
    dot_rings = [
        (dot_r * 1.02, 0.0005),
        (dot_r * 1.00, dot_z_top - 0.00012),
        (dot_r * 0.88, dot_z_top),
        (dot_r * 0.45, dot_z_top + 0.00008),
    ]
    dverts = [
        (r * math.cos(math.tau * i / dot_segs), dot_y_final + r * math.sin(math.tau * i / dot_segs), z)
        for (r, z) in dot_rings
        for i in range(dot_segs)
    ]
    dbc = len(dverts)
    dverts.append((0.0, dot_y_final, 0.0005))
    dtc = len(dverts)
    dverts.append((0.0, dot_y_final, dot_z_top + 0.00010))
    dfaces = []
    for i in range(dot_segs):
        ni = (i + 1) % dot_segs
        dfaces.append((dbc, ni, i))
    for ring in range(len(dot_rings) - 1):
        a0 = ring * dot_segs
        b0 = (ring + 1) * dot_segs
        for i in range(dot_segs):
            ni = (i + 1) % dot_segs
            dfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last_d = (len(dot_rings) - 1) * dot_segs
    for i in range(dot_segs):
        ni = (i + 1) % dot_segs
        dfaces.append((dtc, last_d + i, last_d + ni))

    dot_obj = validate_and_create_obj(f"{name}_dot", dverts, dfaces, [mat_white])

    # Join body + dot into single clean GLB node with identity transform
    bpy.ops.object.select_all(action="DESELECT")
    body.select_set(True)
    dot_obj.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    body.name = name
    return body


def build_pointer_knob(name, dim, mat_body, mat_cap_chamfer, mat_cap_face, mat_white):
    """Build classic Minimoog Model D fluted wedge-pointer selector knob (RANGE / WAVEFORM / FILTER)."""
    r_body = dim["body_radius"]
    r_tip = dim["tip_radius"]
    r_cap = dim["cap_radius"]
    h = dim["height"]
    flutes = dim["flute_count"]
    segs = flutes * SEGMENTS_PER_FLUTE

    # 1. Fluted base + spun aluminum cap (same authentic Minimoog construction)
    r_grip_top = r_cap + (r_body - r_cap) * 0.56
    depth = 0.085 * r_body
    profile = [
        (r_body - 0.00022, 0.0000, 0.0, 0),
        (r_body,           0.0003, 0.0, 0),
        (r_body,           0.0012, 0.0, 0),
        (r_body - 0.00040, 0.0020, 0.0, 0),
        (r_body * 0.91,    0.0028, 0.12 * depth, 0),
        (r_body * 0.90,    0.0044, 0.85 * depth, 0),
        (0.55 * r_body * 0.90 + 0.45 * r_grip_top, 0.58 * h, 1.00 * depth, 0),
        (r_grip_top + 0.00018, h - 0.0022, 0.82 * depth, 0),
        (r_grip_top,           h - 0.0009, 0.18 * depth, 0),
        (r_grip_top - 0.00030, h - 0.0004, 0.0, 0),
        (r_cap + 0.00020,      h - 0.0004, 0.0, 0),
        (r_cap + 0.00014,      h - 0.0007, 0.0, 0),
        (r_cap,                h - 0.0006, 0.0, 1),
        (r_cap - 0.00004,      h - 0.00024, 0.0, 1),
        (r_cap - 0.00052,      h,          0.0, 2),
        (r_cap * 0.62,         h + 0.00004, 0.0, 2),
        (r_cap * 0.22,         h + 0.00006, 0.0, 2),
    ]

    verts = [
        (*flute_xy(r, d, segs, SEGMENTS_PER_FLUTE, i), z)
        for (r, z, d, _) in profile
        for i in range(segs)
    ]
    bot_c = len(verts)
    verts.append((0.0, 0.0, 0.0))
    top_c = len(verts)
    verts.append((0.0, 0.0, h + 0.00007))

    faces, fmats = [], []
    for i in range(segs):
        ni = (i + 1) % segs
        faces.append((bot_c, ni, i))
        fmats.append(0)
    for ring in range(len(profile) - 1):
        a0 = ring * segs
        b0 = (ring + 1) * segs
        m_idx = profile[ring][3]
        for i in range(segs):
            ni = (i + 1) % segs
            faces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
            fmats.append(m_idx)
    last = (len(profile) - 1) * segs
    for i in range(segs):
        ni = (i + 1) % segs
        faces.append((top_c, last + i, last + ni))
        fmats.append(2)

    body = validate_and_create_obj(
        f"{name}_body", verts, faces, [mat_body, mat_cap_chamfer, mat_cap_face], fmats
    )

    # 2. Sculpted black wedge-pointer fin extending along +Y to r_tip
    #    Tapers cleanly from wide base flanking the cap down to a rounded pointed nose at +Y = r_tip
    s = dim["scale"]
    y_back = -r_body * 0.15
    y_mid = r_cap * 0.85
    y_nose = r_tip
    w_back_b = 0.0044 * s
    w_nose_b = 0.0017 * s
    w_back_t = 0.0030 * s
    w_nose_t = 0.00105 * s
    z_top_back = h - 0.00042
    z_top_nose = h * 0.62

    def wedge_loop(scale_xy, z_back, z_nose):
        # 10-point rounded wedge polygon in XY, interpolated Z by Y
        pts_2d = [
            (-w_back_b * scale_xy, y_back),
            (-w_back_b * scale_xy * 0.96, y_mid),
            (-w_nose_b * scale_xy, y_nose - 0.0009 * s),
            (-w_nose_b * scale_xy * 0.55, y_nose - 0.0002 * s),
            (0.0, y_nose),
            (w_nose_b * scale_xy * 0.55, y_nose - 0.0002 * s),
            (w_nose_b * scale_xy, y_nose - 0.0009 * s),
            (w_back_b * scale_xy * 0.96, y_mid),
            (w_back_b * scale_xy, y_back),
            (0.0, y_back - 0.0004 * s),
        ]
        # Reverse for CCW winding when looking from +Z
        pts_2d = list(reversed(pts_2d))
        out = []
        for x, y in pts_2d:
            t = max(0.0, min(1.0, (y - y_mid) / max(1e-6, y_nose - y_mid)))
            z = z_back * (1.0 - t) + z_nose * t
            out.append((x, y, z))
        return out

    # Actually let's loft vertical cross-sections cleanly for a crisp chamfered wedge
    w_rings = [
        wedge_loop(1.00, 0.0000, 0.0000),
        wedge_loop(1.00, 0.0012, 0.0012),
        wedge_loop(0.84, z_top_back - 0.0012, z_top_nose - 0.0009),
        wedge_loop(0.68, z_top_back - 0.0002, z_top_nose - 0.00015),
        wedge_loop(0.55, z_top_back, z_top_nose),
    ]
    n_p = len(w_rings[0])
    wverts = [p for ring in w_rings for p in ring]
    wbc = len(wverts)
    wverts.append((0.0, 0.5 * (y_back + y_nose), 0.0))
    wtc = len(wverts)
    wverts.append((0.0, 0.5 * (y_mid + y_nose), 0.5 * (z_top_back + z_top_nose)))
    wfaces = []
    for i in range(n_p):
        ni = (i + 1) % n_p
        wfaces.append((wbc, ni, i))
    for r in range(len(w_rings) - 1):
        a0 = r * n_p
        b0 = (r + 1) * n_p
        for i in range(n_p):
            ni = (i + 1) % n_p
            wfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last_w = (len(w_rings) - 1) * n_p
    for i in range(n_p):
        ni = (i + 1) % n_p
        wfaces.append((wtc, last_w + i, last_w + ni))
    wedge = validate_and_create_obj(f"{name}_wedge", wverts, wfaces, [mat_body])

    # 3. High-contrast crisp cream-white indicator line inlaid along top of the wedge fin
    y0 = r_cap + 0.00055 * s
    y1 = r_tip - 0.00055 * s
    hw = 0.00062 * s
    # Compute Z along wedge top ridge
    def ridge_z(y):
        t = max(0.0, min(1.0, (y - y_mid) / max(1e-6, y_nose - y_mid)))
        return z_top_back * (1.0 - t) + z_top_nose * t

    z0 = ridge_z(y0)
    z1 = ridge_z(y1)
    # Build closed beveled bar
    bar_stations = [
        (y0, z0, 0.82),
        (y0 + 0.0005 * s, ridge_z(y0 + 0.0005 * s), 1.0),
        (y1 - 0.0005 * s, ridge_z(y1 - 0.0005 * s), 1.0),
        (y1, z1, 0.72),
    ]
    sec = [
        (-1.0, -0.0008),
        (1.0, -0.0008),
        (1.0, 0.00018),
        (0.75, 0.00032),
        (-0.75, 0.00032),
        (-1.0, 0.00018),
    ]
    n_s = len(sec)
    bverts = []
    for y, z, wf in bar_stations:
        for sx, dz in sec:
            bverts.append((sx * hw * wf, y, z + dz))
    bfaces = [tuple(reversed(range(n_s)))]
    for r in range(len(bar_stations) - 1):
        a0 = r * n_s
        b0 = (r + 1) * n_s
        for i in range(n_s):
            ni = (i + 1) % n_s
            bfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last_b = (len(bar_stations) - 1) * n_s
    bfaces.append(tuple(range(last_b, last_b + n_s)))
    bar = validate_and_create_obj(f"{name}_line", bverts, bfaces, [mat_white])

    bpy.ops.object.select_all(action="DESELECT")
    for o in (body, wedge, bar):
        o.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.object.join()
    body.name = name
    return body


def export_glb(obj, out_path: Path):
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
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

    # Authentic Minimoog Model D materials
    black = (srgb_to_lin(20 / 255), srgb_to_lin(20 / 255), srgb_to_lin(21 / 255), 1.0)
    alu_chamfer = (srgb_to_lin(222 / 255), srgb_to_lin(223 / 255), srgb_to_lin(220 / 255), 1.0)
    alu_spun = (srgb_to_lin(196 / 255), srgb_to_lin(197 / 255), srgb_to_lin(192 / 255), 1.0)
    white_inlay = (srgb_to_lin(246 / 255), srgb_to_lin(243 / 255), srgb_to_lin(234 / 255), 1.0)

    mat_body = make_material("Moog_Black_Thermoset", black, roughness=0.23, metallic=0.02, coat=0.35, coat_rough=0.12)
    mat_chamfer = make_material("Moog_Aluminum_Chamfer", alu_chamfer, roughness=0.14, metallic=0.96)
    mat_spun = make_material("Moog_Spun_Aluminum", alu_spun, roughness=0.25, metallic=0.95, aniso=0.85)
    mat_white = make_material("Moog_White_Indicator", white_inlay, roughness=0.22, metallic=0.0, coat=0.15)

    for nm in ("knob_std", "knob_big"):
        obj = build_skirted_knob(nm, dims[nm], mat_body, mat_chamfer, mat_spun, mat_white)
        export_glb(obj, assets_dir / f"{nm}.glb")
        bpy.data.objects.remove(obj, do_unlink=True)

    for nm in ("knob_pointer", "knob_pointer_small"):
        obj = build_pointer_knob(nm, dims[nm], mat_body, mat_chamfer, mat_spun, mat_white)
        export_glb(obj, assets_dir / f"{nm}.glb")
        bpy.data.objects.remove(obj, do_unlink=True)


if __name__ == "__main__":
    main()
