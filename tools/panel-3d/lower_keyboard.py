"""Metric 3D keyboard geometry and materials for the CA-72 lower KEYS page.

Builds the continuous oiled walnut keyboard surround and silkscreened header,
recessed anodized keybed well with rear red wool dust felt, 26 physically
notched aged-ivory white keys, and 18 sculpted glossy ebony black keys.
Generic `rail_key` and `screen` hardware in the header is built separately by
`render_lower_4k.build_fixtures`.
"""

import math
from pathlib import Path

# Official CA-72 lower-page boundaries in local lower-panel units (y = y_abs - PANEL_H).
LEFT_HAND_RIGHT_X = 370.0
TRIM_BOTTOM_H = 6.0
KEYBED_PAD_SIDE = 12.0
KEYBED_PAD_TOP = 12.0
KEYBED_PAD_BOTTOM = 14.0

# Physical Z elevations in meters relative to the lower enamel plate (Z = 0).
SURROUND_TOP_Z = 0.00025
SURROUND_BOT_Z = -0.01500
KEYBED_RABBET_Z = -0.00220
KEYBED_FLOOR_Z = -0.01450
CHASSIS_RECESSED_TOP_Z = -0.01600
CHASSIS_BOTTOM_Z = -0.04130

WHITE_TOP_Z = 0.00200
WHITE_BOT_Z = -0.01100
BLACK_TOP_Z = 0.00920
BLACK_BOT_Z = -0.01100


# ---------------------------------------------------------------------------
# Pure 2D contour utilities (testable without bpy)
# ---------------------------------------------------------------------------


def signed_area_2d(points):
    area = 0.0
    n = len(points)
    for i in range(n):
        x0, y0 = points[i]
        x1, y1 = points[(i + 1) % n]
        area += x0 * y1 - x1 * y0
    return 0.5 * area


def clean_orthogonal_polygon(points, eps=1e-6):
    """Deduplicate coincident/collinear vertices and enforce CCW winding."""
    dedup = []
    for x, y in points:
        if not dedup or math.hypot(x - dedup[-1][0], y - dedup[-1][1]) > eps:
            dedup.append((float(x), float(y)))
    if len(dedup) > 1 and math.hypot(dedup[0][0] - dedup[-1][0], dedup[0][1] - dedup[-1][1]) <= eps:
        dedup.pop()

    # Drop collinear vertices along straight un-notched key edges.
    cleaned = []
    n = len(dedup)
    for i in range(n):
        px, py = dedup[(i - 1) % n]
        cx, cy = dedup[i]
        nx, ny = dedup[(i + 1) % n]
        ux, uy = cx - px, cy - py
        vx, vy = nx - cx, ny - cy
        lu, lv = math.hypot(ux, uy), math.hypot(vx, vy)
        if lu <= eps or lv <= eps:
            continue
        cross = abs(ux * vy - uy * vx) / (lu * lv)
        if cross > 1e-4:
            cleaned.append((cx, cy))

    if len(cleaned) < 3:
        raise ValueError(f"Degenerate key polygon ({len(cleaned)} vertices)")
    if signed_area_2d(cleaned) < 0.0:
        cleaned.reverse()
    return cleaned


def filleted_offset_polygon(
    poly_ccw,
    inset,
    *,
    front_extra_inset=0.0,
    front_y=None,
    outer_r=0.00032,
    inner_r=0.00042,
    corner_segs=3,
):
    """Inset a CCW orthogonal polygon and round every 90-degree corner.

    Preserves an exact constant vertex count (`len(poly_ccw) * (corner_segs + 1)`)
    across loft rings for both convex outer corners and concave notch corners.
    """
    n = len(poly_ccw)
    dirs = []
    shifted = []
    for i in range(n):
        x0, y0 = poly_ccw[i]
        x1, y1 = poly_ccw[(i + 1) % n]
        dx, dy = x1 - x0, y1 - y0
        length = math.hypot(dx, dy)
        ux, uy = dx / length, dy / length
        # CCW inward normal is (-uy, ux).
        nx, ny = -uy, ux
        d = inset
        if front_y is not None and abs(uy) < 1e-4 and abs(y0 - front_y) < 1e-5:
            d += front_extra_inset
        dirs.append((ux, uy))
        shifted.append((x0 + nx * d, y0 + ny * d))

    # Intersect adjacent offset lines (all turns in a white key are 90 degrees).
    mitered = []
    for i in range(n):
        p0x, p0y = shifted[(i - 1) % n]
        u0x, u0y = dirs[(i - 1) % n]
        p1x, p1y = shifted[i]
        u1x, u1y = dirs[i]
        det = u0x * u1y - u0y * u1x
        if abs(det) < 1e-8:
            mitered.append((p1x, p1y))
        else:
            rx, ry = p1x - p0x, p1y - p0y
            t = (rx * u1y - ry * u1x) / det
            mitered.append((p0x + t * u0x, p0y + t * u0y))

    ring = []
    for i in range(n):
        prev_pt = mitered[(i - 1) % n]
        cur_pt = mitered[i]
        next_pt = mitered[(i + 1) % n]
        len_in = math.hypot(cur_pt[0] - prev_pt[0], cur_pt[1] - prev_pt[1])
        len_out = math.hypot(next_pt[0] - cur_pt[0], next_pt[1] - cur_pt[1])
        u_in = dirs[(i - 1) % n]
        u_out = dirs[i]
        cross = u_in[0] * u_out[1] - u_in[1] * u_out[0]
        base_r = max(0.00004, outer_r - 0.35 * inset) if cross > 0 else max(0.00004, inner_r + 0.35 * inset)
        r = min(base_r, 0.38 * min(len_in, len_out))

        ax = cur_pt[0] - u_in[0] * r
        ay = cur_pt[1] - u_in[1] * r
        bx = cur_pt[0] + u_out[0] * r
        by = cur_pt[1] + u_out[1] * r
        cx = ax + (bx - cur_pt[0])
        cy = ay + (by - cur_pt[1])
        vax, vay = ax - cx, ay - cy
        vbx, vby = bx - cx, by - cy
        for step in range(corner_segs + 1):
            ang = 0.5 * math.pi * step / corner_segs
            ca, sa = math.cos(ang), math.sin(ang)
            ring.append((cx + vax * ca + vbx * sa, cy + vay * ca + vby * sa))
    return ring


def asymmetric_rounded_rect(x0, x1, y0, y1, z, radius, segs=4):
    """CCW rounded rectangle with independent bounds (4 * (segs + 1) vertices)."""
    r = min(radius, 0.45 * (x1 - x0), 0.45 * (y1 - y0))
    corners = (
        (x1 - r, y1 - r, 0.0),
        (x0 + r, y1 - r, 0.5 * math.pi),
        (x0 + r, y0 + r, math.pi),
        (x1 - r, y0 + r, 1.5 * math.pi),
    )
    points = []
    for cx, cy, start in corners:
        for step in range(segs + 1):
            ang = start + 0.5 * math.pi * step / segs
            points.append((cx + r * math.cos(ang), cy + r * math.sin(ang), z))
    return points


def frame_ring_pair(outer_bounds, inner_bounds, z, inner_r, outer_r=0.00025, segs=5):
    """Return matched CCW outer and inner rings of length `4 * (segs + 1)`."""
    ox0, oy0, ox1, oy1 = outer_bounds
    ix0, iy0, ix1, iy1 = inner_bounds
    inner = asymmetric_rounded_rect(ix0, ix1, iy0, iy1, z, inner_r, segs=segs)
    outer = asymmetric_rounded_rect(ox0, ox1, oy0, oy1, z, outer_r, segs=segs)
    return outer, inner


def derive_keybed_bounds(spec):
    """Derive local keybed bounds [x0, y0, x1, y1] from official white_key controls."""
    whites = [c for c in spec.get("controls", []) if c.get("kind") == "white_key"]
    if not whites:
        return (414.0, 197.0, 3038.0, 803.0)
    x0 = min(c["x"] - c["w"] / 2.0 for c in whites)
    # Official white keys subtract a 2-unit inter-key gap from pitch (`WHITE_W - 2.0`).
    x1 = max(c["x"] + c["w"] / 2.0 for c in whites) + 2.0
    y0 = min(c["y"] - c["h"] / 2.0 for c in whites)
    y1 = max(c["y"] + c["h"] / 2.0 for c in whites)
    return (x0, y0, x1, y1)


# ---------------------------------------------------------------------------
# Blender mesh builders and PBR materials
# ---------------------------------------------------------------------------


def _build_closed_tube(name, rings, material, sharp_angle=30):
    """Loft a closed loop of rings (ring[-1] connects back to ring[0]) into a manifold solid."""
    from build_switches import create_mesh

    ring_len = len(rings[0])
    num_rings = len(rings)
    verts = [pt for ring in rings for pt in ring]
    faces = []
    for r in range(num_rings):
        a = r * ring_len
        b = ((r + 1) % num_rings) * ring_len
        for i in range(ring_len):
            j = (i + 1) % ring_len
            faces.append((a + i, a + j, b + j, b + i))
    return create_mesh(name, verts, faces, [material], sharp_angle=sharp_angle)


def _cap_triangles(ring_xyz, ring_offset, flip=False):
    """Triangulate a planar simple (possibly non-convex) ring via Blender's ear-clipper."""
    from mathutils import Vector
    from mathutils.geometry import tessellate_polygon

    poly_2d = [Vector((x, y, 0.0)) for x, y, _ in ring_xyz]
    tris = tessellate_polygon([poly_2d])
    faces = []
    for i, j, k in tris:
        ax, ay = ring_xyz[i][0], ring_xyz[i][1]
        bx, by = ring_xyz[j][0], ring_xyz[j][1]
        cx, cy = ring_xyz[k][0], ring_xyz[k][1]
        cross = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
        if abs(cross) < 1e-13:
            continue
        if (cross > 0.0) ^ flip:
            faces.append((ring_offset + i, ring_offset + j, ring_offset + k))
        else:
            faces.append((ring_offset + i, ring_offset + k, ring_offset + j))
    return faces


def _build_capped_loft(name, rings, material, sharp_angle=30):
    """Loft stacked CCW rings from bottom to top and close both planar ends cleanly."""
    from build_switches import create_mesh

    ring_len = len(rings[0])
    verts = [pt for ring in rings for pt in ring]
    faces = _cap_triangles(rings[0], 0, flip=True)
    for r in range(len(rings) - 1):
        a = r * ring_len
        b = (r + 1) * ring_len
        for i in range(ring_len):
            j = (i + 1) % ring_len
            faces.append((a + i, a + j, b + j, b + i))
    top_offset = (len(rings) - 1) * ring_len
    faces.extend(_cap_triangles(rings[-1], top_offset, flip=False))
    return create_mesh(name, verts, faces, [material], sharp_angle=sharp_angle)


def _add_uv_layer(obj, name, project, *, active=False):
    mesh = obj.data
    layer = mesh.uv_layers.get(name) or mesh.uv_layers.new(name=name)
    for loop, coord in zip(mesh.loops, layer.data):
        coord.uv = project(mesh.vertices[loop.vertex_index].co)
    if active:
        mesh.uv_layers.active = layer
    return layer


def _trim_lower_body_for_keybed(spec, scale, to_world):
    """Crop the enamel plate to the left-hand column and recess the backing steel."""
    import bpy
    from build_cabinet import beveled_box, mesh_uv

    width, height, rail_h = spec["width"], spec["height"], spec["rail_h"]
    face_height = (height - rail_h) * scale

    old_panel = bpy.data.objects.get("Lower_Enamel_Plate")
    if old_panel is not None and old_panel.data.materials:
        enamel = old_panel.data.materials[0]
        bpy.data.objects.remove(old_panel, do_unlink=True)
        lh_w = LEFT_HAND_RIGHT_X
        lh_panel = beveled_box(
            "Lower_Enamel_Plate",
            (lh_w * scale, face_height, 0.0023),
            to_world(lh_w / 2.0, (height + rail_h) / 2.0, -0.00115),
            enamel,
            0.00011,
        )
        # Preserve exact full-panel silkscreen UV coordinates on the cropped left-hand plate.
        mesh_uv(
            lh_panel,
            lambda v: ((lh_w / 2.0 + v.x / scale) / width, 0.5 + v.y / face_height),
        )

    old_chassis = bpy.data.objects.get("Lower_Folded_Chassis")
    if old_chassis is not None and old_chassis.data.materials:
        steel = old_chassis.data.materials[0]
        bpy.data.objects.remove(old_chassis, do_unlink=True)
        recessed_depth = CHASSIS_RECESSED_TOP_Z - CHASSIS_BOTTOM_Z
        center_z = 0.5 * (CHASSIS_RECESSED_TOP_Z + CHASSIS_BOTTOM_Z)
        beveled_box(
            "Lower_Folded_Chassis",
            (width * scale, height * scale, recessed_depth),
            to_world(width / 2.0, height / 2.0, center_z),
            steel,
            0.0008,
        )


def _walnut_case_material(texture_dir):
    """Oiled black walnut matching `Walnut_Horiz`, with optional silkscreen ink on top."""
    import bpy
    from shaders_cabinet import TEXTURED, image_maps, surface

    material = bpy.data.materials.new("Keyboard_Walnut_Case")
    tree, bsdf = surface(material, "Walnut_Horiz")
    image_maps(tree, bsdf, texture_dir, *TEXTURED["Walnut_Horiz"])

    # Point the walnut maps explicitly at UVMap so SilkscreenUV can drive header lettering.
    uv_wood = tree.nodes.new("ShaderNodeUVMap")
    uv_wood.uv_map = "UVMap"
    for node in tree.nodes:
        if node.type == "TEX_IMAGE":
            tree.links.new(uv_wood.outputs["UV"], node.inputs["Vector"])

    mask_path = texture_dir / "silkscreen_mask.png"
    if not mask_path.is_file():
        return material

    uv_ink = tree.nodes.new("ShaderNodeUVMap")
    uv_ink.uv_map = "SilkscreenUV"
    ink_tex = tree.nodes.new("ShaderNodeTexImage")
    ink_tex.label = "Keyboard Header Silkscreen"
    ink_tex.image = bpy.data.images.load(str(mask_path), check_existing=True)
    ink_tex.image.colorspace_settings.name = "Non-Color"
    ink_tex.interpolation = "Cubic"
    ink_tex.extension = "CLIP"
    tree.links.new(uv_ink.outputs["UV"], ink_tex.inputs["Vector"])

    # Gate silkscreen strictly to upward-facing header surfaces so bevel walls stay bare wood.
    geom = tree.nodes.new("ShaderNodeNewGeometry")
    sep = tree.nodes.new("ShaderNodeSeparateXYZ")
    tree.links.new(geom.outputs["Normal"], sep.inputs["Vector"])
    up_mask = tree.nodes.new("ShaderNodeMath")
    up_mask.operation = "GREATER_THAN"
    up_mask.inputs[1].default_value = 0.85
    tree.links.new(sep.outputs["Z"], up_mask.inputs[0])

    ink_fac = tree.nodes.new("ShaderNodeMath")
    ink_fac.operation = "MULTIPLY"
    tree.links.new(ink_tex.outputs["Alpha"], ink_fac.inputs[0])
    tree.links.new(up_mask.outputs["Value"], ink_fac.inputs[1])

    # Blend warm vintage ivory screenprint ink onto the oiled walnut base color.
    wood_color_link = bsdf.inputs["Base Color"].links[0]
    wood_color_out = wood_color_link.from_socket
    tree.links.remove(wood_color_link)
    color_mix = tree.nodes.new("ShaderNodeMixRGB")
    color_mix.inputs[2].default_value = (0.72, 0.67, 0.56, 1.0)
    tree.links.new(ink_fac.outputs["Value"], color_mix.inputs["Fac"])
    tree.links.new(wood_color_out, color_mix.inputs[1])
    tree.links.new(color_mix.outputs["Color"], bsdf.inputs["Base Color"])

    # Silkscreen paint is slightly smoother than open-pore walnut.
    wood_rough_link = bsdf.inputs["Roughness"].links[0]
    wood_rough_out = wood_rough_link.from_socket
    tree.links.remove(wood_rough_link)
    rough_mix = tree.nodes.new("ShaderNodeMix")
    rough_mix.data_type = "FLOAT"
    rough_mix.inputs[3].default_value = 0.44
    tree.links.new(ink_fac.outputs["Value"], rough_mix.inputs[0])
    tree.links.new(wood_rough_out, rough_mix.inputs[2])
    tree.links.new(rough_mix.outputs[0], bsdf.inputs["Roughness"])

    return material


def _keyboard_materials(texture_dir):
    import bpy
    from shaders_cabinet import noise_field, tool_marks

    def new_bsdf(name, color, roughness, *, metallic=0.0, specular=0.40):
        mat = bpy.data.materials.new(name)
        mat.diffuse_color = (*color, 1.0)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        bsdf.inputs["Base Color"].default_value = (*color, 1.0)
        bsdf.inputs["Roughness"].default_value = roughness
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Specular IOR Level"].default_value = specular
        return mat, mat.node_tree, bsdf

    walnut = _walnut_case_material(texture_dir)

    # Brushed satin aluminum/nickel section divider between left-hand enamel and walnut case.
    divider, div_tree, div_bsdf = new_bsdf(
        "Keyboard_Divider_Trim", (0.25, 0.245, 0.232), 0.34, metallic=0.86, specular=0.50,
    )
    div_coords = div_tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
    tool_marks(div_tree, div_bsdf, div_coords, (1200, 95, 1200), 0.06, relief=0.000002)

    # Recessed dark anodized keybed inner rabbet frame and deep matte floor.
    bed_frame, bf_tree, bf_bsdf = new_bsdf(
        "Keyboard_Keybed_Frame", (0.011, 0.012, 0.012), 0.56, metallic=0.32, specular=0.28,
    )
    bf_coords = bf_tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
    tool_marks(bf_tree, bf_bsdf, bf_coords, (3600, 3600, 3600), 0.04, relief=0.000005)

    bed_floor, _, _ = new_bsdf(
        "Keyboard_Keybed_Well", (0.0038, 0.0042, 0.0042), 0.88, metallic=0.08, specular=0.14,
    )

    # Classic vintage crimson-burgundy wool dust felt strip behind the rear key line.
    felt, f_tree, f_bsdf = new_bsdf(
        "Keyboard_Red_Felt", (0.056, 0.007, 0.009), 0.95, metallic=0.0, specular=0.07,
    )
    f_coords = f_tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
    f_noise = f_tree.nodes.new("ShaderNodeTexNoise")
    f_noise.inputs["Scale"].default_value = 3400.0
    f_noise.inputs["Detail"].default_value = 8.0
    f_noise.inputs["Roughness"].default_value = 0.72
    f_tree.links.new(f_coords, f_noise.inputs["Vector"])
    f_mix = f_tree.nodes.new("ShaderNodeMixRGB")
    f_mix.inputs[1].default_value = (0.040, 0.004, 0.005, 1.0)
    f_mix.inputs[2].default_value = (0.073, 0.010, 0.012, 1.0)
    f_tree.links.new(f_noise.outputs["Fac"], f_mix.inputs["Fac"])
    f_tree.links.new(f_mix.outputs["Color"], f_bsdf.inputs["Base Color"])
    f_bump = f_tree.nodes.new("ShaderNodeBump")
    f_bump.inputs["Distance"].default_value = 0.00018
    f_bump.inputs["Strength"].default_value = 0.65
    f_tree.links.new(f_noise.outputs["Fac"], f_bump.inputs["Height"])
    f_tree.links.new(f_bump.outputs["Normal"], f_bsdf.inputs["Normal"])

    # Realistic aged ivory ABS white keys with per-key patina variation, soft SSS, and satin polish.
    ivory, i_tree, i_bsdf = new_bsdf(
        "Keyboard_White_Ivory", (0.56, 0.49, 0.375), 0.32, metallic=0.0, specular=0.34,
    )
    i_bsdf.inputs["IOR"].default_value = 1.52
    i_bsdf.inputs["Subsurface Weight"].default_value = 0.08
    i_bsdf.inputs["Subsurface Radius"].default_value = (1.0, 0.85, 0.62)
    i_bsdf.inputs["Subsurface Scale"].default_value = 0.0012
    i_bsdf.inputs["Coat Weight"].default_value = 0.08
    i_bsdf.inputs["Coat Roughness"].default_value = 0.24

    i_coords = i_tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
    i_obj = i_tree.nodes.new("ShaderNodeObjectInfo")
    i_patina = noise_field(i_tree, i_coords, (38.0, 26.0, 38.0))
    i_grain = noise_field(i_tree, i_coords, (2200.0, 520.0, 2200.0))

    # Combine subtle per-key UV-age shift with gentle tonal patina across each keytop.
    i_age_blend = i_tree.nodes.new("ShaderNodeMath")
    i_age_blend.operation = "MULTIPLY_ADD"
    i_age_blend.inputs[1].default_value = 0.55
    i_tree.links.new(i_obj.outputs["Random"], i_age_blend.inputs[0])
    i_patina_scale = i_tree.nodes.new("ShaderNodeMath")
    i_patina_scale.operation = "MULTIPLY"
    i_patina_scale.inputs[1].default_value = 0.45
    i_tree.links.new(i_patina, i_patina_scale.inputs[0])
    i_tree.links.new(i_patina_scale.outputs["Value"], i_age_blend.inputs[2])

    i_color = i_tree.nodes.new("ShaderNodeMixRGB")
    i_color.inputs[1].default_value = (0.49, 0.416, 0.303, 1.0)
    i_color.inputs[2].default_value = (0.63, 0.56, 0.448, 1.0)
    i_tree.links.new(i_age_blend.outputs["Value"], i_color.inputs["Fac"])
    i_tree.links.new(i_color.outputs["Color"], i_bsdf.inputs["Base Color"])

    i_rough = i_tree.nodes.new("ShaderNodeMapRange")
    i_rough.inputs["To Min"].default_value = 0.26
    i_rough.inputs["To Max"].default_value = 0.39
    i_tree.links.new(i_grain, i_rough.inputs["Value"])
    i_tree.links.new(i_rough.outputs["Result"], i_bsdf.inputs["Roughness"])

    i_bump = i_tree.nodes.new("ShaderNodeBump")
    i_bump.inputs["Distance"].default_value = 0.000006
    i_bump.inputs["Strength"].default_value = 0.35
    i_tree.links.new(i_grain, i_bump.inputs["Height"])
    i_tree.links.new(i_bump.outputs["Normal"], i_bsdf.inputs["Normal"])

    # Sculpted glossy vintage ebony/black ABS sharp keys.
    ebony, e_tree, e_bsdf = new_bsdf(
        "Keyboard_Black_Ebony", (0.011, 0.0115, 0.012), 0.26, metallic=0.0, specular=0.30,
    )
    e_bsdf.inputs["IOR"].default_value = 1.52
    e_bsdf.inputs["Coat Weight"].default_value = 0.10
    e_bsdf.inputs["Coat Roughness"].default_value = 0.21

    e_coords = e_tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
    e_obj = e_tree.nodes.new("ShaderNodeObjectInfo")
    e_wear = noise_field(e_tree, e_coords, (1800.0, 260.0, 1800.0))

    e_var = e_tree.nodes.new("ShaderNodeMath")
    e_var.operation = "MULTIPLY_ADD"
    e_var.inputs[1].default_value = 0.25
    e_tree.links.new(e_obj.outputs["Random"], e_var.inputs[0])
    e_wear_s = e_tree.nodes.new("ShaderNodeMath")
    e_wear_s.operation = "MULTIPLY"
    e_wear_s.inputs[1].default_value = 0.75
    e_tree.links.new(e_wear, e_wear_s.inputs[0])
    e_tree.links.new(e_wear_s.outputs["Value"], e_var.inputs[2])

    e_rough = e_tree.nodes.new("ShaderNodeMapRange")
    e_rough.inputs["To Min"].default_value = 0.20
    e_rough.inputs["To Max"].default_value = 0.32
    e_tree.links.new(e_var.outputs["Value"], e_rough.inputs["Value"])
    e_tree.links.new(e_rough.outputs["Result"], e_bsdf.inputs["Roughness"])

    e_bump = e_tree.nodes.new("ShaderNodeBump")
    e_bump.inputs["Distance"].default_value = 0.000004
    e_bump.inputs["Strength"].default_value = 0.28
    e_tree.links.new(e_wear, e_bump.inputs["Height"])
    e_tree.links.new(e_bump.outputs["Normal"], e_bsdf.inputs["Normal"])

    return {
        "walnut": walnut,
        "divider": divider,
        "bed_frame": bed_frame,
        "bed_floor": bed_floor,
        "felt": felt,
        "ivory": ivory,
        "ebony": ebony,
    }


def _build_walnut_surround(spec, scale, to_world, cut_bounds, material):
    """Build one continuous oiled walnut surround + header with beveled keybed cutout."""
    width, height = spec["width"], spec["height"]
    rail_h, cheek = spec["rail_h"], spec["cheek_w"]

    area_x0, area_y0 = LEFT_HAND_RIGHT_X, rail_h
    area_x1, area_y1 = width - spec["side_trim_w"], height - TRIM_BOTTOM_H
    cut_x0, cut_y0, cut_x1, cut_y1 = cut_bounds

    # Convert local lower-panel rects to Blender XY (remember Y flips: top y0 -> wy1, bottom y1 -> wy0).
    ox0, oy1, _ = to_world(area_x0, area_y0, 0.0)
    ox1, oy0, _ = to_world(area_x1, area_y1, 0.0)
    ix0, iy1, _ = to_world(cut_x0, cut_y0, 0.0)
    ix1, iy0, _ = to_world(cut_x1, cut_y1, 0.0)

    def inset_rect(bounds, d):
        return (bounds[0] + d, bounds[1] + d, bounds[2] - d, bounds[3] - d)

    def outset_rect(bounds, d):
        return (bounds[0] - d, bounds[1] - d, bounds[2] + d, bounds[3] + d)

    outer_b = (ox0, oy0, ox1, oy1)
    inner_b = (ix0, iy0, ix1, iy1)
    inner_r = 0.00145

    # Closed cross-section around the frame: outer wall -> top face -> beveled inner lip -> inner well wall.
    profile = [
        ("outer", outer_b, SURROUND_BOT_Z, 0.00025),
        ("outer", outer_b, SURROUND_TOP_Z - 0.00018, 0.00025),
        ("outer", inset_rect(outer_b, 0.00018), SURROUND_TOP_Z, 0.00020),
        ("inner", outset_rect(inner_b, 0.00095), SURROUND_TOP_Z, inner_r + 0.00095),
        ("inner", outset_rect(inner_b, 0.00046), SURROUND_TOP_Z, inner_r + 0.00046),
        ("inner", outset_rect(inner_b, 0.00022), SURROUND_TOP_Z - 0.00010, inner_r + 0.00022),
        ("inner", outset_rect(inner_b, 0.00007), SURROUND_TOP_Z - 0.00028, inner_r + 0.00007),
        ("inner", inner_b, SURROUND_TOP_Z - 0.00055, inner_r),
        ("inner", inner_b, SURROUND_BOT_Z, inner_r),
    ]
    rings = [
        asymmetric_rounded_rect(b[0], b[2], b[1], b[3], z, r, segs=5)
        for _side, b, z, r in profile
    ]
    surround = _build_closed_tube("Keyboard_Walnut_Surround", rings, material, sharp_angle=35)

    # Horizontal walnut grain matching `Preset_Rail` in `render_lower_4k.py`.
    full_width = (width + 2.0 * cheek) * scale
    wood_scale = 0.965 / full_width
    across = wood_scale * (1400.0 / 788.0) * 2.2
    _add_uv_layer(
        surround,
        "UVMap",
        lambda v: (
            0.50 + v.x * wood_scale,
            0.36 + (v.y + (SURROUND_TOP_Z - v.z) * 0.14) * across,
        ),
        active=True,
    )

    # Panel silkscreen UV matching `Lower_Enamel_Plate` so header lettering lands on exact coordinates.
    face_height = (height - rail_h) * scale
    face_mid_y = (height / 2.0 - (height + rail_h) / 2.0) * scale
    _add_uv_layer(
        surround,
        "SilkscreenUV",
        lambda v: (0.5 + v.x / (width * scale), 0.5 + (v.y - face_mid_y) / face_height),
        active=False,
    )
    return surround


def _build_keybed_recess(spec, scale, to_world, bed_bounds, cut_bounds, mats):
    """Build the recessed dark anodized keybed frame, deep well floor, and rear red felt."""
    from build_cabinet import beveled_box

    width, height, rail_h = spec["width"], spec["height"], spec["rail_h"]
    bed_x0, bed_y0, bed_x1, bed_y1 = bed_bounds
    cut_x0, cut_y0, cut_x1, cut_y1 = cut_bounds

    # Satin aluminum vertical divider separating the left-hand panel from the walnut keyboard case.
    div_h = (height - TRIM_BOTTOM_H - rail_h) * scale
    beveled_box(
        "Keyboard_Section_Trim",
        (2.2 * scale, div_h, 0.00175),
        to_world(LEFT_HAND_RIGHT_X + 0.5, 0.5 * (rail_h + height - TRIM_BOTTOM_H), -0.000125),
        mats["divider"],
        0.00016,
        segments=4,
    )

    # Dark anodized inner rabbet lining the keybed opening below the walnut lip.
    ox0, oy1, _ = to_world(cut_x0 - 0.8, cut_y0 - 0.8, 0.0)
    ox1, oy0, _ = to_world(cut_x1 + 0.8, cut_y1 + 0.8, 0.0)
    ix0, iy1, _ = to_world(bed_x0 - 2.2, bed_y0 - 2.2, 0.0)
    ix1, iy0, _ = to_world(bed_x1 + 2.2, bed_y1 + 2.2, 0.0)
    inner_r = 0.00075
    rabbet_profile = [
        ((ox0, oy0, ox1, oy1), KEYBED_FLOOR_Z - 0.0004, 0.0014),
        ((ox0, oy0, ox1, oy1), KEYBED_RABBET_Z, 0.0014),
        ((ix0 - 0.0003, iy0 - 0.0003, ix1 + 0.0003, iy1 + 0.0003), KEYBED_RABBET_Z, inner_r + 0.0003),
        ((ix0, iy0, ix1, iy1), KEYBED_RABBET_Z - 0.00025, inner_r),
        ((ix0, iy0, ix1, iy1), KEYBED_FLOOR_Z - 0.0004, inner_r),
    ]
    rabbet_rings = [
        asymmetric_rounded_rect(b[0], b[2], b[1], b[3], z, r, segs=4)
        for b, z, r in rabbet_profile
    ]
    _build_closed_tube("Keyboard_Bed_Frame", rabbet_rings, mats["bed_frame"], sharp_angle=35)

    # Deep matte dark keybed cavity floor.
    floor_w = (cut_x1 - cut_x0 + 4.0) * scale
    floor_h = (cut_y1 - cut_y0 + 4.0) * scale
    beveled_box(
        "Keyboard_Bed_Floor",
        (floor_w, floor_h, 0.0016),
        to_world(0.5 * (cut_x0 + cut_x1), 0.5 * (cut_y0 + cut_y1), KEYBED_FLOOR_Z - 0.0008),
        mats["bed_floor"],
        0.00010,
    )

    # Classic vintage red baize dust felt strip tucked along the rear of the keybed.
    felt_x0, felt_x1 = bed_x0 - 1.8, bed_x1 + 1.8
    felt_y0, felt_y1 = bed_y0 - 6.8, bed_y0 + 1.5
    felt_h = 0.0044
    felt_top = 0.00105
    beveled_box(
        "Keyboard_Dust_Felt",
        ((felt_x1 - felt_x0) * scale, (felt_y1 - felt_y0) * scale, felt_h),
        to_world(0.5 * (felt_x0 + felt_x1), 0.5 * (felt_y0 + felt_y1), felt_top - 0.5 * felt_h),
        mats["felt"],
        0.00028,
        segments=3,
    )


def _build_white_key(entry, scale, to_world, material):
    """Build one physically notched aged-ivory white key from its official shape outline."""
    cx, cy, _ = to_world(entry["x"], entry["y"], 0.0)
    raw_shape = entry.get("shape")
    if raw_shape:
        local_pts = []
        for px, py in raw_shape:
            wx, wy, _ = to_world(px, py, 0.0)
            local_pts.append((wx - cx, wy - cy))
    else:
        hw = 0.5 * entry["w"] * scale
        hh = 0.5 * entry["h"] * scale
        local_pts = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]

    poly = clean_orthogonal_polygon(local_pts)
    front_y = min(y for _, y in poly)

    # Loft from deep key skirt up to a crisp beveled flat playing surface + coplanar buffer ring.
    profile = [
        (WHITE_BOT_Z, 0.00018, 0.00000),
        (WHITE_TOP_Z - 0.00065, 0.00000, 0.00000),
        (WHITE_TOP_Z - 0.00032, 0.00007, 0.00008),
        (WHITE_TOP_Z - 0.00010, 0.00021, 0.00020),
        (WHITE_TOP_Z, 0.00038, 0.00030),
        (WHITE_TOP_Z, 0.00085, 0.00030),
    ]
    rings = [
        [
            (x, y, z)
            for x, y in filleted_offset_polygon(
                poly,
                inset,
                front_extra_inset=front_extra,
                front_y=front_y,
                outer_r=0.00032,
                inner_r=0.00044,
                corner_segs=3,
            )
        ]
        for z, inset, front_extra in profile
    ]
    tag = entry.get("param", entry.get("label", "note"))
    obj = _build_capped_loft(f"Keyboard_White_{tag}", rings, material, sharp_angle=28)
    obj.location = (cx, cy, 0.0)
    return obj


def _build_black_key(entry, scale, to_world, material):
    """Build one sculpted glossy black key with planar tapered flanks and angled front ramp."""
    cx, cy, _ = to_world(entry["x"], entry["y"], 0.0)
    w = entry["w"] * scale
    h = entry["h"] * scale
    hw, hh = 0.5 * w, 0.5 * h

    side_taper = w * 0.080
    front_ramp = h * 0.052
    back_taper = 0.00045
    rise = BLACK_TOP_Z - WHITE_TOP_Z

    # In local Blender coords, -hh is the front playing nose and +hh is the rear end.
    def ring(z, side, front, back, radius):
        return asymmetric_rounded_rect(
            -hw + side, hw - side, -hh + front, hh - back, z, radius, segs=4,
        )

    shoulder_z = BLACK_TOP_Z - 0.00060
    t_sh = (shoulder_z - WHITE_TOP_Z) / rise
    rings = [
        ring(BLACK_BOT_Z, 0.00015, 0.00015, 0.00015, 0.00030),
        ring(WHITE_TOP_Z, 0.0, 0.0, 0.0, 0.00032),
        ring(shoulder_z, side_taper * t_sh, front_ramp * t_sh, back_taper * t_sh, 0.00038),
        ring(BLACK_TOP_Z - 0.00026, side_taper * 0.985, front_ramp * 0.985, back_taper * 0.985, 0.00042),
        ring(BLACK_TOP_Z - 0.00007, side_taper + 0.00014, front_ramp + 0.00016, back_taper + 0.00014, 0.00046),
        ring(BLACK_TOP_Z, side_taper + 0.00032, front_ramp + 0.00038, back_taper + 0.00032, 0.00048),
        ring(BLACK_TOP_Z, side_taper + 0.00075, front_ramp + 0.00080, back_taper + 0.00075, 0.00044),
    ]
    tag = entry.get("param", entry.get("label", "note"))
    obj = _build_capped_loft(f"Keyboard_Black_{tag}", rings, material, sharp_angle=26)
    obj.location = (cx, cy, 0.0)
    return obj


def build_keyboard(spec, scale, to_world, texture_dir):
    """Build the CA-72 KEYS page walnut surround, recessed keybed, and 44 physical keys."""
    tex_dir = Path(texture_dir).resolve()
    bed_bounds = derive_keybed_bounds(spec)
    cut_bounds = (
        bed_bounds[0] - KEYBED_PAD_SIDE,
        bed_bounds[1] - KEYBED_PAD_TOP,
        bed_bounds[2] + KEYBED_PAD_SIDE,
        bed_bounds[3] + KEYBED_PAD_BOTTOM,
    )

    _trim_lower_body_for_keybed(spec, scale, to_world)
    mats = _keyboard_materials(tex_dir)
    _build_walnut_surround(spec, scale, to_world, cut_bounds, mats["walnut"])
    _build_keybed_recess(spec, scale, to_world, bed_bounds, cut_bounds, mats)

    keys = []
    for entry in spec.get("controls", []):
        kind = entry.get("kind")
        if kind == "white_key":
            keys.append((entry, _build_white_key(entry, scale, to_world, mats["ivory"])))
        elif kind == "black_key":
            keys.append((entry, _build_black_key(entry, scale, to_world, mats["ebony"])))
    return keys


def render_key_travel(scene, keys, spec, density, build_dir):
    """Render each depressed key in place, with its neighbours at rest."""
    import bpy
    import json
    from mathutils import Vector

    out = build_dir / "page-keys-travel-raw"
    out.mkdir(exist_ok=True)
    frames = {}
    render = scene.render
    camera = scene.camera
    face_w, face_h = render.resolution_x, render.resolution_y
    scale = spec["scale_m_per_unit"]
    ppm = density / scale
    padding = math.ceil(0.0035 * ppm)
    render.use_persistent_data = True
    scene.cycles.samples = min(scene.cycles.samples, 64)

    for entry, key in keys:
        note = entry["param"]
        x = (entry["x"] + spec["cheek_w"]) * density
        y = entry["y"] * density
        half_w, half_h = (entry[k] * density / 2 for k in ("w", "h"))
        x0, x1 = math.floor(x - half_w) - padding, math.ceil(x + half_w) + padding
        y0, y1 = math.floor(y - half_h) - padding, math.ceil(y + half_h) + padding
        w, h = x1 - x0, y1 - y0

        # The crop maintains the original ortho scale and pixel grid.
        render.resolution_x, render.resolution_y = w, h
        camera.data.ortho_scale = w / ppm
        camera.location.x = ((x0 + x1) / 2 - face_w / 2) / ppm
        camera.location.y = (face_h / 2 - (y0 + y1) / 2) / ppm

        # The rear hinge stays fixed. Limit front travel to the well clearance.
        length = entry["h"] * scale
        pivot = Vector((0, length / 2, -0.005))
        rest = key.location.copy()
        key.rotation_euler.x = math.asin(0.003 / length)
        key.location = rest + pivot - key.rotation_euler.to_matrix() @ pivot
        render.filepath = str(out / f"{note}.png")
        bpy.ops.render.render(write_still=True)
        key.location = rest
        key.rotation_euler.x = 0
        frames[note] = [x0, y0, x1, y1]

    render.resolution_x, render.resolution_y = face_w, face_h
    camera.data.ortho_scale = face_w / ppm
    camera.location.x = camera.location.y = 0
    (out / "frames.json").write_text(json.dumps(frames, indent=2) + "\n")
    print(f"Rendered {len(frames)} individual depressed keys", flush=True)


def prepare_key_travel(build_dir):
    """Match the panel grade and feather away small render differences at crop edges."""
    import json
    import numpy as np
    from PIL import Image

    source = build_dir / "page-keys-travel-raw"
    manifest = source / "frames.json"
    if not manifest.is_file():
        return
    destination = build_dir / "page-keys-travel"
    destination.mkdir(exist_ok=True)
    frames = json.loads(manifest.read_text())
    with Image.open(build_dir / "page-keys-raw.png") as raw, Image.open(
        build_dir / "page-keys-3d.png"
    ) as graded:
        for note, bounds in frames.items():
            base = np.asarray(raw.crop(bounds), dtype=np.int16)
            grade = np.asarray(graded.crop(bounds), dtype=np.int16)
            with Image.open(source / f"{note}.png") as frame:
                pressed = np.asarray(frame.convert("RGB"), dtype=np.int16)
            color = np.clip(grade + pressed - base, 0, 255).astype(np.uint8)
            h, w, _ = color.shape
            y, x = np.ogrid[:h, :w]
            edge = np.minimum(np.minimum(x, w - 1 - x), np.minimum(y, h - 1 - y))
            t = np.clip(edge / 8, 0, 1)
            alpha = np.round(255 * t * t * (3 - 2 * t)).astype(np.uint8)
            Image.fromarray(np.dstack((color, alpha))).save(destination / f"{note}.png")
    (destination / "frames.json").write_text(json.dumps(frames, indent=2) + "\n")
