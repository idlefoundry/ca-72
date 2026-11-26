#!/usr/bin/env python3
"""Render the lower cabinet and its button and display hardware in Cycles.

Prepare the enamel and walnut maps with a Python that has Pillow and NumPy:
    python tools/panel-3d/render_lower_4k.py --out BUILD --prepare-textures
Render with Blender's Python:
    blender-python tools/panel-3d/render_lower_4k.py --out BUILD
Calibrate the opaque background with the same finishing pass as the upper panel:
    python tools/panel-3d/render_lower_4k.py --out BUILD --calibrate

BUILD contains lower.json and the lower-print.png silkscreen mask. The body is
written to lower-raw.png and calibrated to lower-3d.png. The unprinted lamp
cap sprite is written to lower-tab-lit.png.

Pass --page keys, --page lfo, or --page macros to each step for a
replacement bay. The JSON, print mask, and renders then use page-<name>
stems in the same build directory. The pages share the base's lit tab atlas.

For the assembled preview, pass --upper-manifest to anchor the rig in
the upper panel's scene, and --grade-reference at calibration time to
carry over the original enamel grade without canceling the new falloff.
"""

import argparse
import json
import math
from pathlib import Path


def export_size(spec, face_w):
    pixels_per_unit = face_w / (spec["width"] + 2 * spec["cheek_w"])
    return pixels_per_unit, round(spec["height"] * pixels_per_unit)


def prepare_textures(build_dir, spec, name="lower"):
    from PIL import Image
    from build_textures import build_faceplate_pbr, build_walnut_pbr

    directory = build_dir / f"{name}-textures"
    directory.mkdir(parents=True, exist_ok=True)
    mask_width = 5760
    mask_height = round(mask_width * (spec["height"] - spec["rail_h"]) / spec["width"])
    print_path = build_dir / f"{name}-print.png"
    with Image.open(print_path) as print_mask:
        top = spec["rail_h"] / spec["height"] * print_mask.height
        print_mask.convert("RGBA").resize(
            (mask_width, mask_height), Image.Resampling.BICUBIC,
            box=(0, top, print_mask.width, print_mask.height),
        ).save(directory / "silkscreen_mask.png")
    build_faceplate_pbr(directory)
    build_walnut_pbr(directory)


def calibrate(build_dir, spec, face_w, name="lower", grade_reference=None):
    from PIL import Image, ImageDraw
    from calibrate_shadows import calibrate_faceplate, measure_sprite_falloff

    density, height = export_size(spec, face_w)
    cheek = spec["cheek_w"] * density
    base = build_dir / "lower-raw.png"
    calibrate_faceplate(
        build_dir / f"{name}-raw.png",
        build_dir / f"{name}-3d.png",
        {
            "face": {
                "faceplate_rect_px": [cheek, spec["rail_h"] * density, face_w - cheek, height],
                "px_per_unit": density,
            }
        },
        reference_path=grade_reference or (base if name != "lower" and base.is_file() else None),
    )
    if name != "lower":
        if spec.get("page") == "keys":
            from lower_keyboard import prepare_key_travel

            prepare_key_travel(build_dir)
        return
    falloff_file = build_dir / "sprite-falloff.json"
    if grade_reference is not None:
        falloff_file.write_text(json.dumps(measure_sprite_falloff(base, grade_reference, cheek)) + "\n")
    else:
        falloff_file.unlink(missing_ok=True)
    # The reconstruction filter can leave a few alpha levels at the cell edge.
    # Clear those pixels before the lit cap is composited onto the dark one.
    cap_path = build_dir / "lower-tab-lit.png"
    with Image.open(cap_path) as raw:
        cap = raw.convert("RGBA")
    alpha = cap.getchannel("A")
    ImageDraw.Draw(alpha).rectangle((0, 0, cap.width - 1, cap.height - 1), outline=0)
    cap.putalpha(alpha)
    cap.save(cap_path, optimize=True)


def build_body(spec, scale, to_world, texture_dir):
    import bpy
    from build_cabinet import beveled_box, mesh_uv
    from shaders_cabinet import TEXTURED, image_maps, make_material, surface

    width, height = spec["width"], spec["height"]
    rail_h, cheek = spec["rail_h"], spec["cheek_w"]
    full_width = (width + 2 * cheek) * scale
    face_height = (height - rail_h) * scale
    depth = 0.039
    trim = make_material("Aluminum_Trim")
    walnut = make_material("Walnut_Horiz")
    walnut_vertical = make_material("Walnut_Vert")
    enamel = bpy.data.materials.new("Lower_Wrinkle_Enamel")
    tree, paint = surface(enamel, "Faceplate_PBR")
    image_maps(tree, paint, texture_dir, *TEXTURED["Faceplate_PBR"])
    panel = beveled_box(
        "Lower_Enamel_Plate",
        (width * scale, face_height, 0.0023),
        to_world(width / 2, (height + rail_h) / 2, -0.00115),
        enamel,
        0.00011,
    )
    mesh_uv(panel, lambda v: (0.5 + v.x / (width * scale), 0.5 + v.y / face_height))
    beveled_box(
        "Lower_Folded_Chassis",
        (width * scale, height * scale, depth),
        to_world(width / 2, height / 2, -depth / 2 - 0.0023),
        make_material("Steel_Backing"),
        0.0008,
    )

    # The preset rail has a square joint into the two continuous side cheeks.
    wood_scale = 0.965 / full_width
    across = wood_scale * (1400 / 788) * 2.2
    for name, x, w, h, front, axis, band, offset, material in (
        ("Preset_Rail", width / 2, width, rail_h, 0.0064, 0, 0.62, 0.50, walnut),
        ("Cheek_Left", -cheek / 2, cheek, height, 0.0049, 1, 0.42, 0.36, walnut_vertical),
        ("Cheek_Right", width + cheek / 2, cheek, height, 0.0049, 1, 0.28, 0.64, walnut_vertical),
    ):
        thickness = depth + front
        board = beveled_box(
            name, (w * scale, h * scale, thickness), to_world(x, h / 2, (front - depth) / 2),
            material, 0.00115 if axis == 0 else 0.00165,
        )
        mesh_uv(board, lambda v: (
            offset + v[axis] * wood_scale,
            band + (v[1 - axis] + (thickness / 2 - v.z) * 0.14) * across,
        ))

    for name, x, y, w, h, axis in (
        ("Left", 7.5, (height + rail_h - 6) / 2, 15, height - rail_h - 6, 1),
        ("Right", width - 14, (height + rail_h - 6) / 2, 28, height - rail_h - 6, 1),
        ("Foot", width / 2, height - 3, width, 6, 0),
    ):
        size = (w * scale, h * scale)
        piece = beveled_box(
            f"Lower_Trim_{name}", (*size, 0.00135), to_world(x, y, 0.000675),
            trim, 0.00035, segments=5,
        )
        mesh_uv(piece, lambda v: (0.5 + v[axis] / size[axis], 0.5 + v[1 - axis] / size[1 - axis]))


def molded_finish(name, color, roughness, metallic=0.0, specular=0.38):
    import bpy

    material = bpy.data.materials.new(name)
    material.use_nodes = True
    nodes = material.node_tree.nodes
    bsdf = nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Specular IOR Level"].default_value = specular
    return material, bsdf


def cap_finish(name, color, roughness):
    from shaders_cabinet import tool_marks

    material, bsdf = molded_finish(name, color, roughness)
    tree = material.node_tree
    coordinates = tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
    tool_marks(tree, bsdf, coordinates, (2580, 2410, 2620), 0.052, relief=0.000011)
    return material, bsdf


def etched_glass(field=False):
    from shaders_cabinet import lens_surface

    name = "Field_Filter_Glass" if field else "Smoked_Filter_Glass"
    material, bsdf = molded_finish(
        name, (0.63, 0.73, 0.65), 0.15, specular=0.064 if field else 0.055,
    )
    bsdf.inputs["IOR"].default_value = 1.48
    bsdf.inputs["Transmission Weight"].default_value = 0.90 if field else 0.82
    # Keep a thin Fresnel reflection on the filter without burying the mesh
    # and live readout under an image of the entire window-sized softbox.
    lens_surface(material.node_tree, bsdf)
    return material


def display_matrix(field=False):
    name = "Field_Conductive_Mesh" if field else "Display_Conductive_Mesh"
    material, bsdf = molded_finish(name, (0.011, 0.017, 0.022), 0.76)
    tree = material.node_tree
    coordinates = tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
    cells = tree.nodes.new("ShaderNodeTexVoronoi")
    cells.voronoi_dimensions = "2D"
    cells.inputs["Scale"].default_value = 680 if field else 880
    cells.feature = "DISTANCE_TO_EDGE" if field else "F1"
    cells.inputs["Randomness"].default_value = 0
    tree.links.new(coordinates, cells.inputs["Vector"])
    tint = tree.nodes.new("ShaderNodeValToRGB")
    tint.color_ramp.elements[0].position = 0.02 if field else 0.22
    tint.color_ramp.elements[0].color = (0.040, 0.078, 0.040, 1) if field else (0.020, 0.037, 0.033, 1)
    tint.color_ramp.elements[1].position = 0.15 if field else 0.47
    tint.color_ramp.elements[1].color = (0.003, 0.009, 0.006, 1)
    tree.links.new(cells.outputs["Distance"], tint.inputs["Fac"])
    tree.links.new(tint.outputs["Color"], bsdf.inputs["Base Color"])
    return material


def tab_material(lit):
    color = (0.25, 0.047, 0.004) if lit else (0.052, 0.011, 0.002)
    material, bsdf = cap_finish("Amber_Resin_Lit" if lit else "Amber_Resin_Off", color, 0.35)
    bsdf.inputs["Specular IOR Level"].default_value = 0.14
    bsdf.inputs["Subsurface Weight"].default_value = 0.24
    bsdf.inputs["Subsurface Radius"].default_value = (1.0, 0.43, 0.10)
    bsdf.inputs["Subsurface Scale"].default_value = 0.002
    if lit:
        tree = material.node_tree
        coordinates = tree.nodes.new("ShaderNodeTexCoord").outputs["Generated"]
        distance = tree.nodes.new("ShaderNodeVectorMath")
        distance.operation = "DISTANCE"
        distance.inputs[1].default_value = (0.49, 0.53, 1.18)
        tree.links.new(coordinates, distance.inputs[0])
        ramp = tree.nodes.new("ShaderNodeValToRGB")
        ramp.color_ramp.elements[0].position = 0.20
        ramp.color_ramp.elements[0].color = (0.75, 0.16, 0.008, 1)
        ramp.color_ramp.elements[1].position = 0.70
        ramp.color_ramp.elements[1].color = (0.22, 0.044, 0.002, 1)
        ramp.color_ramp.interpolation = "EASE"
        tree.links.new(distance.outputs["Value"], ramp.inputs["Fac"])
        tree.links.new(ramp.outputs["Color"], bsdf.inputs["Emission Color"])
        bsdf.inputs["Emission Strength"].default_value = 0.62
    return material


def place_tab(entry, scale, to_world, material):
    from build_cabinet import beveled_box

    w, h = ((entry[key] - 2.6) * scale for key in ("w", "h"))
    depth = 0.0039
    return beveled_box(
        "Translucent_Tab", (w, h, depth), to_world(entry["x"], entry["y"], depth / 2),
        material, 0.00105, segments=5,
    )


def build_fixtures(spec, scale, to_world):
    from build_cabinet import beveled_box

    gasket, _ = molded_finish("Black_Fixture_Gasket", (0.004, 0.0045, 0.0048), 0.67)
    rim, _ = molded_finish("Milled_Black_Aluminum", (0.030, 0.032, 0.035), 0.35, 0.60)
    collar, _ = molded_finish("Inner_Display_Rabbet", (0.006, 0.007, 0.0065), 0.39)
    rail_cap, cap_shader = cap_finish("Charcoal_Preset_Key", (0.017, 0.016, 0.015), 0.46)
    cap_shader.inputs["Specular IOR Level"].default_value = 0.18
    amber = tab_material(False)
    glass = etched_glass()
    matrix = display_matrix()
    field_glass = etched_glass(field=True)
    field_matrix = display_matrix(field=True)
    active_tabs = []
    for entry in spec["controls"]:
        kind = entry["kind"]
        if kind not in ("tab", "rail_key", "screen"):
            continue
        x, y = entry["x"], entry["y"]
        w, h = entry["w"] * scale, entry["h"] * scale
        label = entry.get("label", entry.get("param", kind))
        if kind == "tab":
            tab = place_tab(entry, scale, to_world, amber)
            if entry.get("lit", False):
                active_tabs.append(tab)
        elif kind == "rail_key":
            base = 0.0064 if y < spec["rail_h"] else 0.00025
            depth = 0.00165
            beveled_box(
                f"{label}_Key_Cap", (w, h, depth),
                to_world(x, y, base + depth / 2), rail_cap, 0.00028, segments=5,
            )
        else:
            # Each opening has a raised nickel-black lip, a dark inset
            # rebate, and a continuous pane sitting in the rebate.
            rail = y < spec["rail_h"]
            field = entry.get("role") == "field"
            scope = entry.get("param", "").startswith("lfo_scope_")
            octave = entry.get("role") == "octave"
            base = 0.0064 if rail else 0.00025 if octave else 0.0
            low_profile = rail or octave
            surround = 0.00043 if low_profile else 0.00092
            if scope:
                surround = 0.0024
                beveled_box(
                    f"{label}_Scope_Gasket", (w + 0.0065, h + 0.0065, 0.0015),
                    to_world(x, y, 0.00030), gasket, 0.00060, segments=6,
                )
            if low_profile:
                layers = (
                    ("Bezel", 2 * surround, 0.00082, 0.00034, rim, 0.00016),
                    ("Rebate", 0.00018, 0.00023, 0.00068, collar, 0.00009),
                    ("Matrix", -0.00014, 0.00015, 0.00083, matrix, 0.00005),
                    ("Filter", 0.0, 0.00026, 0.00105, glass, 0.00009),
                )
            else:
                layers = (
                    ("Bezel", 2 * surround, 0.0020, 0.00086, rim, 0.00080),
                    ("Rebate", 0.00035, 0.00062, 0.00160, collar, 0.00028),
                    ("Matrix", -0.00014, 0.00032, 0.00186, field_matrix if field else matrix, 0.00010),
                    ("Filter", 0.0, 0.00075, 0.00253, field_glass if field else glass, 0.00029),
                )
            for name, edge, depth, centre, material, bevel in layers:
                beveled_box(
                    f"{label}_Display_{name}", (w + edge, h + edge, depth),
                    to_world(x, y, base + centre), material, bevel, segments=5,
                )
    return active_tabs


def build_macro_collars(spec, scale, to_world):
    from lower_left import lathe

    material, _ = molded_finish("Dark_Bronze_Macro_Jack_Collars", (0.077, 0.039, 0.018), 0.42, 0.56)
    for entry in spec["controls"]:
        if entry["kind"] != "jack" or entry["x"] >= spec["bounds"][2]:
            continue
        r = entry["w"] * scale / 2
        outer = 24.5 * scale
        bezel = lathe(
            "Macro_Jack_Collar",
            [
                (r * 0.83, 0.00010), (outer - 0.00035, 0.00010),
                (outer, 0.00040), (outer - 0.00028, 0.00092),
                (r * 0.83, 0.00092),
            ],
            material, segments=96, closed=True,
        )
        bezel.location = to_world(entry["x"], entry["y"])


def model_proof_knobs(spec, to_world, knob_glb):
    import bpy
    from export_pack_4k import import_glb_collection, instantiate
    from shaders_knobs import apply_knob_shaders

    _, knob = import_glb_collection(knob_glb.parent, knob_glb.stem)
    apply_knob_shaders(bpy.data.materials)
    for entry in spec["controls"]:
        if entry["kind"] == "knob":
            angle = 150 - 300 * entry["value"]
            instantiate(
                knob, entry["param"], to_world(entry["x"], entry["y"], 0.00005),
                rot_z_deg=angle,
            )


def studio_anchor(upper_manifest, density, height, pixels_per_meter):
    if upper_manifest is None:
        return (0, 0, 0)
    upper = json.loads(upper_manifest.read_text())["face"]
    if not math.isclose(upper["px_per_unit"], density, rel_tol=1e-5):
        raise ValueError("The upper and lower renders must use the same pixel density")
    join = round(upper["faceplate_rect_px"][3])
    # Lower meshes are centered in their own camera. The original light rig
    # is that far up in their local coordinates, with its original aim.
    return (0, (join + (height - upper["h"]) / 2) / pixels_per_meter, 0)


def render(build_dir, spec, face_w, samples, oblique, knob_glb, name="lower", upper_manifest=None):
    import bpy
    from lower_left import build_lower_left
    from shaders_cabinet import apply_cabinet_shaders
    from studio_rig import make_ortho_cam, reset_scene, setup_studio_lights

    scale = spec["scale_m_per_unit"]
    density, height = export_size(spec, face_w)
    pixels_per_meter = density / scale
    texture_dir = build_dir / f"{name}-textures"
    if not (texture_dir / "panel_albedo.png").is_file():
        raise FileNotFoundError(f"{texture_dir}: run --prepare-textures first")

    def to_world(x, y, z=0):
        return ((x - spec["width"] / 2) * scale, (spec["height"] / 2 - y) * scale, z)

    scene, backend = reset_scene(samples, transparent=False)
    scene.unit_settings.system = "METRIC"
    setup_studio_lights(scene, studio_anchor(upper_manifest, density, height, pixels_per_meter))
    build_body(spec, scale, to_world, texture_dir)
    active_tabs = build_fixtures(spec, scale, to_world)
    apply_cabinet_shaders(bpy.data.materials, texture_dir)
    build_lower_left({
        "controls": [entry for entry in spec["controls"] if not entry.get("sprite")]
    }, scale, to_world)
    keyboard = None
    if spec.get("page") == "keys":
        from lower_keyboard import build_keyboard, render_key_travel

        keyboard = build_keyboard(spec, scale, to_world, texture_dir)
    elif spec.get("page") == "macros":
        build_macro_collars(spec, scale, to_world)
    make_ortho_cam(scene, 0.0, 0.0, face_w, height, pixels_per_meter)
    scene.render.filepath = str(build_dir / f"{name}-raw.png")
    bpy.ops.render.render(write_still=True)
    print(f"Rendered {name}-raw.png at {face_w}×{height} with {backend}", flush=True)

    if keyboard is not None:
        render_key_travel(scene, keyboard, spec, density, build_dir)

    if oblique:
        amber = tab_material(True)
        for tab in active_tabs:
            tab.data.materials[0] = amber
        if knob_glb:
            model_proof_knobs(spec, to_world, knob_glb.resolve())
        camera = make_ortho_cam(scene, 0.0, 0.0, 1920, 650, pixels_per_meter / 2.18)
        x, y, z = (0.104, -0.152, 0.620)
        camera.location = (x, y, z)
        camera.rotation_euler = (math.atan2(-y, math.hypot(x, z)), math.atan2(x, z), 0)
        scene.render.filepath = str(build_dir / f"{name}-oblique.png")
        bpy.ops.render.render(write_still=True)

    if name != "lower":
        return
    tab = next(entry for entry in spec["controls"] if entry["kind"] == "tab")
    scene, _ = reset_scene(samples, transparent=True)
    setup_studio_lights(scene)
    place_tab(tab, scale, lambda _x, _y, z=0: (0, 0, z), tab_material(True))
    w, h = (round(tab[key] * density) for key in ("w", "h"))
    make_ortho_cam(scene, 0, 0, w, h, pixels_per_meter)
    scene.render.filepath = str(build_dir / "lower-tab-lit.png")
    bpy.ops.render.render(write_still=True)
    print(f"Rendered lower-tab-lit.png at {w}×{h}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--face-w", type=int, default=3840)
    parser.add_argument("--samples", type=int, default=160)
    parser.add_argument("--page", choices=("keys", "lfo", "macros"))
    parser.add_argument("--oblique", action="store_true", help="also render an angled hardware view")
    parser.add_argument("--knob-glb", type=Path, help="include the standard knob in the angled view")
    parser.add_argument("--upper-manifest", type=Path, help="anchor the light rig to this upper render")
    parser.add_argument("--grade-reference", type=Path, help="use the enamel grade measured from this raw lower render")
    command = parser.add_mutually_exclusive_group()
    command.add_argument("--prepare-textures", action="store_true")
    command.add_argument("--calibrate", action="store_true")
    args = parser.parse_args()
    name = "lower" if args.page is None else f"page-{args.page}"
    spec = json.loads((args.out / f"{name}.json").read_text())
    if args.prepare_textures:
        prepare_textures(args.out, spec, name)
    elif args.calibrate:
        calibrate(args.out, spec, args.face_w, name, args.grade_reference)
    else:
        render(args.out.resolve(), spec, args.face_w, args.samples, args.oblique, args.knob_glb, name, args.upper_manifest)


if __name__ == "__main__":
    main()
