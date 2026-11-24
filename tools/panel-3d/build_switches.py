#!/usr/bin/env python3
"""Build closed-manifold Y-up metric GLB models for CA-72's classic Minimoog switches:
  - rocker_horiz.glb  (horizontal sculpted concave rocker paddle + recessed matte bezel)
  - rocker_vert.glb   (vertical sculpted concave rocker paddle + recessed matte bezel)
  - rocker_power.glb  (vertical black ribbed POWER rocker paddle + recessed matte bezel)
  - toggle_bat.glb    (chrome hex nut + keyed washer + threaded bushing + tapered bat lever)
"""

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import bmesh


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


def create_mesh_obj(name, verts, faces, mat, smooth=True):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    if mesh.validate():
        raise ValueError(f"{name}: invalid mesh")
    mesh.materials.append(mat)
    for p in mesh.polygons:
        p.use_smooth = smooth
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    validate_mesh(obj)
    return obj


def build_rocker_bezel(name, w, h, pad, mat_bezel, mat_well):
    """Build closed-manifold rectangular recessed bezel frame around a rocker opening."""
    # Outer bezel dimensions and inner cavity dimensions
    ox, oy = w * 0.5 + pad, h * 0.5 + pad
    ix, iy = w * 0.5 - 0.00035, h * 0.5 - 0.00035
    z_top = 0.00145
    z_well = 0.00025
    ch = 0.00035  # chamfer

    def rect_ring(rx, ry, z, r_corner=0.0006):
        # 16-vertex rounded rectangle CCW
        pts = []
        corners = [(rx - r_corner, ry - r_corner, 0),
                   (-rx + r_corner, ry - r_corner, math.pi * 0.5),
                   (-rx + r_corner, -ry + r_corner, math.pi),
                   (rx - r_corner, -ry + r_corner, math.pi * 1.5)]
        for cx, cy, a0 in corners:
            for k in range(4):
                a = a0 + (math.pi * 0.5) * k / 3.0
                pts.append((cx + r_corner * math.cos(a), cy + r_corner * math.sin(a), z))
        return pts

    rings = [
        rect_ring(ox, oy, 0.0, 0.0007),
        rect_ring(ox, oy, z_top - ch, 0.0007),
        rect_ring(ox - ch, oy - ch, z_top, 0.0005),
        rect_ring(ix + ch * 0.6, iy + ch * 0.6, z_top, 0.0004),
        rect_ring(ix, iy, z_top - ch * 0.6, 0.00035),
        rect_ring(ix, iy, z_well, 0.00035),
    ]
    n = len(rings[0])
    verts = [p for ring in rings for p in ring]
    bot_c = len(verts)
    verts.append((0.0, 0.0, 0.0))
    well_c = len(verts)
    verts.append((0.0, 0.0, z_well))

    faces = []
    fmats = []
    for i in range(n):
        ni = (i + 1) % n
        faces.append((bot_c, ni, i))
        fmats.append(0)
    for r in range(len(rings) - 1):
        a0 = r * n
        b0 = (r + 1) * n
        m_idx = 1 if r >= 4 else 0
        for i in range(n):
            ni = (i + 1) % n
            faces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
            fmats.append(m_idx)
    last = (len(rings) - 1) * n
    for i in range(n):
        ni = (i + 1) % n
        faces.append((well_c, last + i, last + ni))
        fmats.append(1)

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.materials.append(mat_bezel)
    mesh.materials.append(mat_well)
    for poly in mesh.polygons:
        poly.use_smooth = True
        poly.material_index = fmats[poly.index]
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    validate_mesh(obj)
    return obj


def build_rocker_actuator(name, length, width, height, mat_actuator, along_y=False, ribbed=False):
    """Build sculpted concave Minimoog rocker actuator centered at its internal hinge pivot (0,0,0).
    When `along_y=False`, length runs along X (horizontal rocker).
    When `along_y=True`, length runs along Y (vertical rocker).
    """
    hl = length * 0.5 - 0.00055
    hw = width * 0.5 - 0.00055
    # Authentic Minimoog Carling/Switchcraft rocker profile:
    # Smooth concave saddle top with gently rising rounded crests toward both ends,
    # deeper skirt body descending into bezel well.
    nu = 28  # samples along length
    nv = 14  # samples across width
    z_bot = -0.0022

    def top_z(u_norm, v_norm):
        """u_norm, v_norm in [-1, 1]."""
        # Concave saddle along length + rounded lip dropoff near outer ends
        au = abs(u_norm)
        # Saddle rises from center (0.0038) to crest at |u|=0.78 (0.0056), then rounds smoothly over end
        if au <= 0.80:
            t = au / 0.80
            z = height * (0.50 + 0.24 * (t ** 1.85))
        else:
            t = (au - 0.80) / 0.20
            z = height * (0.74 - 0.14 * (t ** 2.2))
        # Subtle ergonomic transverse rounding along long edges
        av = abs(v_norm)
        edge_drop = height * 0.11 * (max(0.0, (av - 0.72) / 0.28) ** 2.0)
        z -= edge_drop
        if ribbed and 0.30 <= au <= 0.84 and av <= 0.82:
            # 4 crisp tactile grip ribs near each end of the black POWER rocker
            rib_phase = (au - 0.30) / (0.84 - 0.30) * 4.0 * math.tau
            z += 0.00028 * max(0.0, math.sin(rib_phase))
        return z

    # Build structured closed manifold box grid (top surface, bottom surface, 4 sidewalls)
    # Using a parametric (u, v) grid with chamfered perimeter
    us = [(-1.0 + 2.0 * i / nu) for i in range(nu + 1)]
    vs = [(-1.0 + 2.0 * j / nv) for j in range(nv + 1)]

    # Slight taper inward near top for molded draft angle
    def map_xy(u, v, draft=1.0):
        # Soft corner rounding in XY
        ax, ay = u * hl * draft, v * hw * draft
        return (ay, ax) if along_y else (ax, ay)

    # Top grid vertices: (nu+1) x (nv+1)
    verts = []
    for i, u in enumerate(us):
        for j, v in enumerate(vs):
            x, y = map_xy(u, v, draft=0.96)
            verts.append((x, y, top_z(u, v)))
    # Bottom grid vertices: (nu+1) x (nv+1)
    bot_off = len(verts)
    for i, u in enumerate(us):
        for j, v in enumerate(vs):
            x, y = map_xy(u, v, draft=1.00)
            verts.append((x, y, z_bot))

    cols = nv + 1
    faces = []
    # Top faces (+Z outward)
    for i in range(nu):
        for j in range(nv):
            a = i * cols + j
            b = (i + 1) * cols + j
            c = (i + 1) * cols + (j + 1)
            d = i * cols + (j + 1)
            if along_y:
                faces.append((a, d, c, b))
            else:
                faces.append((a, b, c, d))
    # Bottom faces (-Z outward)
    for i in range(nu):
        for j in range(nv):
            a = bot_off + i * cols + j
            b = bot_off + (i + 1) * cols + j
            c = bot_off + (i + 1) * cols + (j + 1)
            d = bot_off + i * cols + (j + 1)
            if along_y:
                faces.append((a, b, c, d))
            else:
                faces.append((a, d, c, b))
    # Sidewalls: j=0, j=nv, i=0, i=nu
    for i in range(nu):
        t0, t1 = i * cols + 0, (i + 1) * cols + 0
        b0, b1 = bot_off + t0, bot_off + t1
        faces.append((b0, b1, t1, t0) if not along_y else (t0, t1, b1, b0))

        t0, t1 = i * cols + nv, (i + 1) * cols + nv
        b0, b1 = bot_off + t0, bot_off + t1
        faces.append((t0, t1, b1, b0) if not along_y else (b0, b1, t1, t0))

    for j in range(nv):
        t0, t1 = 0 * cols + j, 0 * cols + (j + 1)
        b0, b1 = bot_off + t0, bot_off + t1
        faces.append((t0, t1, b1, b0) if not along_y else (b0, b1, t1, t0))

        t0, t1 = nu * cols + j, nu * cols + (j + 1)
        b0, b1 = bot_off + t0, bot_off + t1
        faces.append((b0, b1, t1, t0) if not along_y else (t0, t1, b1, b0))

    obj = create_mesh_obj(name, verts, faces, mat_actuator, smooth=True)
    # Place actuator pivot at Z = 0.0018 above faceplate
    obj.location = (0.0, 0.0, 0.0018)
    return obj


def build_toggle_bat(dim, mat_chrome, mat_chrome_satin, mat_dark):
    """Build classic turned chrome bat-handle toggle switch:
      - Toggle_Hardware: keyed anti-rotation washer + chamfered hex nut + threaded bushing + pivot well
      - Toggle_Lever:    tapered turned chrome bat handle (pivot at Z=0.0044)
    """
    r_wash = dim["washer_radius"]
    af = dim["nut_across_flats"]
    r_hex = af / math.sqrt(3.0)  # circumradius of hex nut
    r_bush = dim["bushing_radius"]
    l_len = dim["lever_length"] * 0.84
    r_base = dim["lever_base_radius"]
    r_tip = dim["lever_tip_radius"] * 1.02

    parts = []

    # 1. Keyed flat washer (Z = 0 .. 0.00065)
    segs = 48
    w_prof = [
        (r_wash - 0.00012, 0.0000),
        (r_wash,           0.00012),
        (r_wash,           0.00055),
        (r_wash - 0.00015, 0.00068),
        (r_bush * 0.90,    0.00068),
    ]
    wverts = [
        (r * math.cos(math.tau * i / segs), r * math.sin(math.tau * i / segs), z)
        for (r, z) in w_prof
        for i in range(segs)
    ]
    wbc = len(wverts); wverts.append((0.0, 0.0, 0.0))
    wtc = len(wverts); wverts.append((0.0, 0.0, 0.00068))
    wfaces = []
    for i in range(segs):
        ni = (i + 1) % segs
        wfaces.append((wbc, ni, i))
    for r in range(len(w_prof) - 1):
        a0, b0 = r * segs, (r + 1) * segs
        for i in range(segs):
            ni = (i + 1) % segs
            wfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(w_prof) - 1) * segs
    for i in range(segs):
        ni = (i + 1) % segs
        wfaces.append((wtc, last + i, last + ni))
    parts.append(create_mesh_obj("Toggle_Washer", wverts, wfaces, mat_chrome_satin, smooth=True))

    # 2. Chamfered 6-sided chrome hex mounting nut (Z = 0.00065 .. 0.0033)
    #    Use 6-vertex hex with subtle corner rounding (24 verts) + top machined chamfer!
    def hex_ring(radius, z, flat_k=1.0):
        # Exact 6-sided hex rotated 30 deg so flat faces sit cleanly left/right
        pts = []
        for k in range(6):
            a = math.radians(30.0 + 60.0 * k)
            pts.append((radius * math.cos(a), radius * math.sin(a), z))
        return pts

    h_rings = [
        hex_ring(r_hex * 0.97, 0.00065),
        hex_ring(r_hex,        0.00090),
        hex_ring(r_hex,        0.00285),
        hex_ring(r_hex * 0.91, 0.00330),
        hex_ring(r_bush * 0.95, 0.00330),
    ]
    hverts = [p for ring in h_rings for p in ring]
    hbc = len(hverts); hverts.append((0.0, 0.0, 0.00065))
    htc = len(hverts); hverts.append((0.0, 0.0, 0.00330))
    hfaces = []
    for i in range(6):
        ni = (i + 1) % 6
        hfaces.append((hbc, ni, i))
    for r in range(len(h_rings) - 1):
        a0, b0 = r * 6, (r + 1) * 6
        for i in range(6):
            ni = (i + 1) % 6
            hfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(h_rings) - 1) * 6
    for i in range(6):
        ni = (i + 1) % 6
        hfaces.append((htc, last + i, last + ni))
    # Flat-shaded hex faces catch ultra-crisp chrome facet reflections!
    parts.append(create_mesh_obj("Toggle_HexNut", hverts, hfaces, mat_chrome, smooth=False))

    # 3. Threaded chrome bushing + recessed dark pivot cup (Z = 0.0031 .. 0.0056)
    b_prof = [
        (r_bush,           0.0031, 0),
        (r_bush * 0.94,    0.0035, 0),
        (r_bush,           0.0039, 0),
        (r_bush * 0.94,    0.0043, 0),
        (r_bush,           0.0047, 0),
        (r_bush * 0.94,    0.0051, 0),
        (r_bush * 0.97,    0.0054, 0),
        (r_bush * 0.82,    0.0056, 0),
        (r_bush * 0.68,    0.0054, 1),
        (r_bush * 0.62,    0.0036, 1),
    ]
    bverts = [
        (r * math.cos(math.tau * i / segs), r * math.sin(math.tau * i / segs), z)
        for (r, z, _) in b_prof
        for i in range(segs)
    ]
    bbc = len(bverts); bverts.append((0.0, 0.0, 0.0031))
    btc = len(bverts); bverts.append((0.0, 0.0, 0.0036))
    bfaces, bfmats = [], []
    for i in range(segs):
        ni = (i + 1) % segs
        bfaces.append((bbc, ni, i)); bfmats.append(0)
    for r in range(len(b_prof) - 1):
        a0, b0 = r * segs, (r + 1) * segs
        m = b_prof[r][2]
        for i in range(segs):
            ni = (i + 1) % segs
            bfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i)); bfmats.append(m)
    last = (len(b_prof) - 1) * segs
    for i in range(segs):
        ni = (i + 1) % segs
        bfaces.append((btc, last + i, last + ni)); bfmats.append(1)

    bmesh_d = bpy.data.meshes.new("Toggle_Bushing")
    bmesh_d.from_pydata(bverts, [], bfaces)
    bmesh_d.materials.append(mat_chrome_satin)
    bmesh_d.materials.append(mat_dark)
    for p in bmesh_d.polygons:
        p.use_smooth = True
        p.material_index = bfmats[p.index]
    bush_obj = bpy.data.objects.new("Toggle_Bushing", bmesh_d)
    bpy.context.collection.objects.link(bush_obj)
    validate_mesh(bush_obj)
    parts.append(bush_obj)

    # Join static hardware parts into Toggle_Hardware
    bpy.ops.object.select_all(action="DESELECT")
    for o in parts:
        o.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.join()
    hw = parts[0]
    hw.name = "Toggle_Hardware"

    # 4. Tapered chrome bat-handle lever (origin at internal pivot Z = 0.0044)
    #    Extends from z = -0.0012 (pivot ball) up to z = l_len (spherical bat tip)
    l_segs = 36
    l_prof = []
    # Pivot ball base
    l_prof.append((r_base * 0.70, -0.0010))
    l_prof.append((r_base * 0.95, 0.0000))
    # Tapered conical bat shaft
    shaft_end = l_len - r_tip
    for k in range(1, 10):
        t = k / 9.0
        z = t * shaft_end
        # Smooth iconic bat flare curve
        r = r_base + (r_tip - r_base) * (t ** 0.92)
        l_prof.append((r, z))
    # Hemispherical polished bat tip
    for k in range(1, 10):
        phi = (math.pi * 0.5) * k / 10.0
        r = r_tip * math.cos(phi)
        z = shaft_end + r_tip * math.sin(phi)
        l_prof.append((r, z))

    lverts = [
        (r * math.cos(math.tau * i / l_segs), r * math.sin(math.tau * i / l_segs), z)
        for (r, z) in l_prof
        for i in range(l_segs)
    ]
    lbc = len(lverts); lverts.append((0.0, 0.0, -0.0011))
    ltc = len(lverts); lverts.append((0.0, 0.0, l_len))
    lfaces = []
    for i in range(l_segs):
        ni = (i + 1) % l_segs
        lfaces.append((lbc, ni, i))
    for r in range(len(l_prof) - 1):
        a0, b0 = r * l_segs, (r + 1) * l_segs
        for i in range(l_segs):
            ni = (i + 1) % l_segs
            lfaces.append((a0 + i, a0 + ni, b0 + ni, b0 + i))
    last = (len(l_prof) - 1) * l_segs
    for i in range(l_segs):
        ni = (i + 1) % l_segs
        lfaces.append((ltc, last + i, last + ni))
    lever = create_mesh_obj("Toggle_Lever", lverts, lfaces, mat_chrome, smooth=True)
    lever.location = (0.0, 0.0, 0.0044)
    return [hw, lever]


def export_glb(objs, out_path: Path):
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

    mat_bezel = make_material(
        "Rocker_Bezel",
        (srgb_to_lin(16 / 255), srgb_to_lin(16 / 255), srgb_to_lin(17 / 255), 1.0),
        roughness=0.48,
        metallic=0.0,
    )
    mat_well = make_material("Rocker_Well", (0.004, 0.004, 0.004, 1.0), roughness=0.85)
    # Default blue actuator material (exporter tints cleanly for blue/red/ivory!)
    mat_act = make_material(
        "Rocker_Actuator",
        (srgb_to_lin(52 / 255), srgb_to_lin(118 / 255), srgb_to_lin(178 / 255), 1.0),
        roughness=0.22,
        metallic=0.0,
        coat=0.35,
        coat_rough=0.14,
    )
    mat_power_act = make_material(
        "Rocker_Power_Black",
        (srgb_to_lin(22 / 255), srgb_to_lin(22 / 255), srgb_to_lin(23 / 255), 1.0),
        roughness=0.30,
        metallic=0.0,
        coat=0.20,
        coat_rough=0.18,
    )
    mat_chrome = make_material(
        "Nickel_Chrome",
        (srgb_to_lin(226 / 255), srgb_to_lin(225 / 255), srgb_to_lin(220 / 255), 1.0),
        roughness=0.11,
        metallic=0.98,
    )
    mat_chrome_satin = make_material(
        "Nickel_Chrome_Satin",
        (srgb_to_lin(208 / 255), srgb_to_lin(207 / 255), srgb_to_lin(202 / 255), 1.0),
        roughness=0.22,
        metallic=0.96,
    )
    mat_dark = make_material("Toggle_Inner_Dark", (0.005, 0.005, 0.006, 1.0), roughness=0.80)

    # 1. Horizontal rocker
    rh = dims["rocker_horiz"]
    objs = [
        build_rocker_bezel("Rocker_Bezel", rh["w"], rh["h"], rh["bezel_pad"], mat_bezel, mat_well),
        build_rocker_actuator("Rocker_Actuator", rh["w"], rh["h"], rh["height"], mat_act, along_y=False),
    ]
    export_glb(objs, assets_dir / "rocker_horiz.glb")
    for o in objs:
        bpy.data.objects.remove(o, do_unlink=True)

    # 2. Vertical colored rocker
    rv = dims["rocker_vert"]
    objs = [
        build_rocker_bezel("Rocker_Bezel", rv["w"], rv["h"], rv["bezel_pad"], mat_bezel, mat_well),
        build_rocker_actuator("Rocker_Actuator", rv["h"], rv["w"], rv["height"], mat_act, along_y=True),
    ]
    export_glb(objs, assets_dir / "rocker_vert.glb")
    for o in objs:
        bpy.data.objects.remove(o, do_unlink=True)

    # 3. Vertical black ribbed POWER rocker
    rp = dims["rocker_power"]
    objs = [
        build_rocker_bezel("Rocker_Bezel", rp["w"], rp["h"], rp["bezel_pad"], mat_bezel, mat_well),
        build_rocker_actuator(
            "Rocker_Actuator", rp["h"], rp["w"], rp["height"], mat_power_act, along_y=True, ribbed=True
        ),
    ]
    export_glb(objs, assets_dir / "rocker_power.glb")
    for o in objs:
        bpy.data.objects.remove(o, do_unlink=True)

    # 4. Chrome bat-handle toggle
    objs = build_toggle_bat(dims["toggle"], mat_chrome, mat_chrome_satin, mat_dark)
    export_glb(objs, assets_dir / "toggle_bat.glb")
    for o in objs:
        bpy.data.objects.remove(o, do_unlink=True)


if __name__ == "__main__":
    main()
