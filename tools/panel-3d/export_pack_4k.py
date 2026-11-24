#!/usr/bin/env python3
"""Export CA-72's native 4K (3840x1135) 0-degree flat-orthographic Blender Cycles sprite pack.

Uses Cycles OPTIX on NVIDIA RTX 4080 SUPER (falling back to CUDA -> CPU automatically),
AgX / AgX - Medium High Contrast, exposure = -0.30, and an identical high broad KeySoftbox
studio rig across `face-3d.png` and every moving sprite atlas.

Outputs into <out_dir>/vst-pack-raw/ (ready for `calibrate_shadows.py`):
  - face-3d.png                    (3840x1135 static cabinet, walnut, silkscreen, chrome fixtures, unlit jewels)
  - knob-std-atlas.png             (7x7 = 49 frames, -150..+150 deg, 224x224 RGBA)
  - knob-big-atlas.png             (7x7 = 49 frames, -150..+150 deg, 256x256 RGBA)
  - knob-pointer-atlas.png         (6x1 = 6 exact SIX detents, 256x256 RGBA)
  - knob-pointer-small-atlas.png   (4x1 = 4 exact middle detents, 192x192 RGBA)
  - rocker-horiz-blue-atlas.png    (2x1 = OFF/ON, 208x128 RGBA)
  - rocker-horiz-red-atlas.png     (2x1 = OFF/ON, 208x128 RGBA)
  - rocker-horiz-ivory-atlas.png   (2x1 = OFF/ON, 208x128 RGBA)
  - rocker-vert-red-atlas.png      (2x1 = OFF/ON top, 136x208 RGBA)
  - rocker-vert-blue-atlas.png     (2x1 = OFF/ON bottom, 136x208 RGBA)
  - rocker-power-atlas.png         (2x1 = OFF/ON top, 120x216 RGBA)
  - toggle-vert-atlas.png          (3x1 = DOWN/MID/UP, 128x168 RGBA)
  - toggle-horiz-atlas.png         (3x1 = LEFT/MID/RIGHT, 168x128 RGBA)
  - manifest.json                  (exact 4K coordinates, anchors, body radii, angles, control metadata)
"""

import argparse
import json
import math
import os
import sys
import tempfile
import time
from pathlib import Path

import bpy
import numpy as np
from mathutils import Vector

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from studio_rig import (
    srgb_to_lin,
    hex_to_lin,
    configure_cycles_device,
    reset_scene,
    setup_studio_lights,
    make_ortho_cam,
)
from shaders_knobs import apply_knob_shaders
from shaders_switches import apply_switch_shaders, add_toggle_reflection_cards
from shaders_cabinet import apply_cabinet_shaders


def import_glb_collection(assets_dir: Path, name: str):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(assets_dir / f"{name}.glb"))
    added = [o for o in bpy.data.objects if o not in before]
    col = bpy.data.collections.new(f"tpl_{name}")
    parent = bpy.data.objects.new(f"root_{name}", None)
    col.objects.link(parent)
    roots = [o for o in added if o.parent is None]
    for o in added:
        for c in list(o.users_collection):
            c.objects.unlink(o)
        col.objects.link(o)
    for r in roots:
        r.parent = parent
    return added, col


def instantiate(col, name, loc=(0.0, 0.0, 0.0), rot_z_deg=0.0):
    inst = bpy.data.objects.new(name, None)
    inst.instance_type = "COLLECTION"
    inst.instance_collection = col
    inst.location = loc
    inst.rotation_euler = (0.0, 0.0, math.radians(rot_z_deg))
    bpy.context.scene.collection.objects.link(inst)
    return inst


def add_shadow_catcher():
    bpy.ops.mesh.primitive_plane_add(size=0.40, location=(0.0, 0.0, 0.0))
    plane = bpy.context.active_object
    plane.name = "ShadowCatcher"
    plane.is_shadow_catcher = True
    mat = bpy.data.materials.new("CatcherMat")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (0.008, 0.008, 0.009, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.58
    plane.data.materials.append(mat)
    return plane


def read_png_rgba(path: Path) -> np.ndarray:
    img = bpy.data.images.load(str(path), check_existing=False)
    w, h = img.size
    arr = np.array(img.pixels[:], dtype=np.float32).reshape((h, w, 4))
    bpy.data.images.remove(img)
    return np.flipud(arr)


def save_rgba_png(arr_top_down: np.ndarray, out_path: Path, mode="RGBA"):
    h, w, _ = arr_top_down.shape
    flipped = np.flipud(arr_top_down).astype(np.float32)
    img = bpy.data.images.new("AtlasOut", width=w, height=h, alpha=(mode == "RGBA"))
    img.pixels.foreach_set(flipped.ravel())
    img.filepath_raw = str(out_path)
    img.file_format = "PNG"
    bpy.context.scene.render.image_settings.color_mode = mode
    img.save()
    bpy.data.images.remove(img)


class PackExporter:
    def __init__(self, build_dir: Path, face_w: int, samples_face: int, samples_sprite: int):
        self.build_dir = build_dir
        self.assets_dir = build_dir / "assets"
        self.tex_dir = build_dir / "textures"
        self.out_dir = build_dir / "vst-pack-raw"
        self.out_dir.mkdir(parents=True, exist_ok=True)

        self.spec = json.loads((build_dir / "spec.json").read_text())
        self.scale = self.spec["scale_m_per_unit"]
        pu = self.spec["panel_units"]
        pm = self.spec["panel_meters"]

        self.face_w_u = pu["face_w"]
        self.face_h_u = pu["face_h"]
        self.wood_top_u = pu["wood_top_h"]
        self.cheek_w_u = pu["cheek_w"]
        self.total_w_u = pu["total_w"]
        self.total_h_u = pu["total_h"]

        self.cx_u = self.face_w_u * 0.5
        self.cy_u = -self.wood_top_u + self.total_h_u * 0.5

        self.face_w = face_w
        self.face_h = int(round(face_w * self.total_h_u / self.total_w_u))
        self.ortho_w = pm["total_w"]
        self.px_per_m = self.face_w / self.ortho_w
        self.px_per_unit = self.face_w / self.total_w_u

        self.samples_face = samples_face
        self.samples_sprite = samples_sprite

    def panel_to_blender(self, ux: float, uy: float, z: float = 0.0):
        return ((ux - self.cx_u) * self.scale, -(uy - self.cy_u) * self.scale, z)

    def panel_to_face_px(self, ux: float, uy: float):
        bx, by, _ = self.panel_to_blender(ux, uy, 0.0)
        return [
            round(self.face_w * 0.5 + bx * self.px_per_m, 2),
            round(self.face_h * 0.5 - by * self.px_per_m, 2),
        ]

    def build_static_scene(self):
        """Assemble the cabinet and hardware fixtures for the face or a detail render."""
        scene, backend = reset_scene(self.samples_face, transparent=False)
        setup_studio_lights(scene, (0.0, 0.0, 0.0))

        pm = self.spec["panel_meters"]
        # Dark studio backdrop beneath cabinet
        bpy.ops.mesh.primitive_plane_add(size=3.0, location=(0.0, 0.0, -pm["cabinet_depth"] - 0.002))
        desk = bpy.context.active_object
        dmat = bpy.data.materials.new("DeskMat")
        dmat.use_nodes = True
        dmat.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = (0.008, 0.009, 0.010, 1.0)
        desk.data.materials.append(dmat)

        # Import cabinet (walnut rails, cheeks, brushed aluminum trim)
        _, cab_col = import_glb_collection(self.assets_dir, "cabinet")
        instantiate(cab_col, "Cabinet")

        # High-resolution wrinkle enamel and vector silkscreen.
        y_face_center = self.panel_to_blender(self.cx_u, self.face_h_u * 0.5, 0.0)[1]
        bpy.ops.mesh.primitive_plane_add(size=1.0, location=(0.0, y_face_center, 0.00008))
        fp = bpy.context.active_object
        fp.name = "Faceplate"
        fp.scale = (pm["face_w"], pm["face_h"], 1.0)
        bpy.ops.object.transform_apply(scale=True)
        fp_mat = bpy.data.materials.new("Faceplate_PBR")
        fp_mat.use_nodes = True
        fp.data.materials.append(fp_mat)

        # Name plate mounted on top walnut rail (z = 0.0072)
        _, plate_col = import_glb_collection(self.assets_dir, "name_plate")
        fix = self.spec["fixtures"]
        pl_tl = fix["plate_top_left"]
        pl_sz = fix["plate_size"]
        pl_cx, pl_cy = pl_tl[0] + pl_sz[0] * 0.5, pl_tl[1] + pl_sz[1] * 0.5
        instantiate(plate_col, "NamePlate", self.panel_to_blender(pl_cx, pl_cy, 0.0072))

        # Static hardware fixtures on faceplate & walnut rail
        tpls = {}
        for nm in ("jewel_power", "jewel_overload", "jack_quarter", "screw_phillips"):
            _, col = import_glb_collection(self.assets_dir, nm)
            tpls[nm] = col

        # Apply shared Cycles shaders across cabinet, walnut, enamel faceplate, nameplate, glass lamps, chrome jack & screws
        apply_cabinet_shaders(bpy.data.materials, self.tex_dir)

        # Unlit optical jewels. QUALITY's opening is composited from the worn artwork.
        instantiate(tpls["jewel_power"], "Jewel_Power", self.panel_to_blender(*fix["lamp"], 0.0001))
        instantiate(tpls["jewel_overload"], "Jewel_Overload", self.panel_to_blender(*fix["overload"], 0.0001))

        # 1/4" Chrome PHONES jack
        if fix["phones"] is not None:
            px, py, _ = fix["phones"]
            instantiate(tpls["jack_quarter"], "Jack_Phones", self.panel_to_blender(px, py, 0.0001))

        # Chassis screws along walnut top rail (seated at walnut top z = 0.0072)
        for idx, (sx, sy, _) in enumerate(fix["screws"]):
            instantiate(tpls["screw_phillips"], f"Screw_{idx}", self.panel_to_blender(sx, sy, 0.0072), (idx * 47) % 180)
        return scene, backend

    def render_static_face(self):
        t0 = time.time()
        scene, backend = self.build_static_scene()
        print(f"Rendering static face-3d.png ({self.face_w}x{self.face_h}, {self.samples_face} samples, {backend})...", flush=True)
        make_ortho_cam(scene, 0.0, 0.0, self.face_w, self.face_h, self.px_per_m)
        scene.render.filepath = str(self.out_dir / "face-3d.png")
        bpy.ops.render.render(write_still=True)
        print(f"EXPORTED face-3d.png in {time.time() - t0:.1f}s", flush=True)

    def render_knob_atlas(self, glb_name, angles_deg, cell_size, anchor_xy, cols, out_name, body_r_px):
        t0 = time.time()
        scene, _ = reset_scene(self.samples_sprite, transparent=True)
        setup_studio_lights(scene, (0.0, 0.0, 0.0))
        add_shadow_catcher()

        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(self.assets_dir / f"{glb_name}.glb"))
        added = [o for o in bpy.data.objects if o not in before]
        apply_knob_shaders(bpy.data.materials)

        knob_roots = [o for o in added if o.parent is None]
        for r in knob_roots:
            r.location = (r.location.x, r.location.y, r.location.z + 0.00005)
            r.rotation_mode = "XYZ"

        ax, ay = anchor_xy
        cam_cx = (cell_size * 0.5 - ax) / self.px_per_m
        cam_cy = -(cell_size * 0.5 - ay) / self.px_per_m
        make_ortho_cam(scene, cam_cx, cam_cy, cell_size, cell_size, self.px_per_m)

        rows = math.ceil(len(angles_deg) / cols)
        atlas = np.zeros((rows * cell_size, cols * cell_size, 4), dtype=np.float32)

        with tempfile.TemporaryDirectory() as td:
            tmp_png = Path(td) / "frame.png"
            scene.render.filepath = str(tmp_png)
            for i, deg in enumerate(angles_deg):
                # Clockwise positive dial deg = negative Z rotation in Blender
                for r in knob_roots:
                    r.rotation_euler = (0.0, 0.0, math.radians(-deg))
                bpy.ops.render.render(write_still=True)
                row_i, col_i = divmod(i, cols)
                atlas[row_i * cell_size : (row_i + 1) * cell_size, col_i * cell_size : (col_i + 1) * cell_size] = read_png_rgba(tmp_png)

        save_rgba_png(atlas, self.out_dir / out_name, mode="RGBA")
        print(f"EXPORTED {out_name} ({cols}x{rows}, {len(angles_deg)} frames) in {time.time() - t0:.1f}s", flush=True)
        return {
            "file": out_name,
            "kind": "knob",
            "cell_w": cell_size,
            "cell_h": cell_size,
            "anchor": [ax, ay],
            "body_r": round(body_r_px, 2),
            "cols": cols,
            "rows": rows,
            "frames": len(angles_deg),
            "angles_deg": angles_deg,
        }

    def render_rocker_atlas(self, glb_name, color_hex, orient, cell_w, cell_h, anchor_xy, body_wh_px, out_name):
        """Render 2 frames (frame 0 = OFF, frame 1 = ON) natively at the rocker's real orientation."""
        t0 = time.time()
        scene, _ = reset_scene(self.samples_sprite, transparent=True)
        setup_studio_lights(scene, (0.0, 0.0, 0.0))
        add_shadow_catcher()

        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(self.assets_dir / f"{glb_name}.glb"))
        added = [o for o in bpy.data.objects if o not in before]
        apply_switch_shaders(bpy.data.materials, color_hex=color_hex)

        actuator = None
        for o in added:
            if o.type == "MESH" and "actuator" in o.name.lower():
                actuator = o

        ax, ay = anchor_xy
        cam_cx = (cell_w * 0.5 - ax) / self.px_per_m
        cam_cy = -(cell_h * 0.5 - ay) / self.px_per_m
        make_ortho_cam(scene, cam_cx, cam_cy, cell_w, cell_h, self.px_per_m)

        # Tilt angles (deg) around actuator's hinge axis:
        # IMPORTANT: glTF import leaves rotation_mode='QUATERNION', so switch to 'XYZ' first!
        # orient == "right":  ON presses right (+X down -> +Y rot in Blender), OFF presses left (-X down -> -Y rot)
        # orient == "top":    ON presses top (+Y down -> -X rot in Blender), OFF presses bottom (-Y down -> +X rot)
        # orient == "bottom": ON presses bottom (-Y down -> +X rot in Blender), OFF presses top (+Y down -> -X rot)
        tilt = 17.5
        if orient == "right":
            states = [(0.0, -tilt, 0.0), (0.0, +tilt, 0.0)]  # [OFF, ON]
        elif orient == "top":
            states = [(+tilt, 0.0, 0.0), (-tilt, 0.0, 0.0)]  # [OFF, ON]
        else:  # bottom
            states = [(-tilt, 0.0, 0.0), (+tilt, 0.0, 0.0)]  # [OFF, ON]

        if actuator is not None:
            actuator.rotation_mode = "XYZ"

        atlas = np.zeros((cell_h, cell_w * 2, 4), dtype=np.float32)
        with tempfile.TemporaryDirectory() as td:
            tmp_png = Path(td) / "rk.png"
            scene.render.filepath = str(tmp_png)
            for i, (rx, ry, rz) in enumerate(states):
                if actuator is not None:
                    actuator.rotation_euler = (math.radians(rx), math.radians(ry), math.radians(rz))
                bpy.ops.render.render(write_still=True)
                atlas[:, i * cell_w : (i + 1) * cell_w] = read_png_rgba(tmp_png)

        save_rgba_png(atlas, self.out_dir / out_name, mode="RGBA")
        print(f"EXPORTED {out_name} (2 frames) in {time.time() - t0:.1f}s", flush=True)
        return {
            "file": out_name,
            "kind": "rocker",
            "orient": orient,
            "cell_w": cell_w,
            "cell_h": cell_h,
            "anchor": [ax, ay],
            "body_w": round(body_wh_px[0], 2),
            "body_h": round(body_wh_px[1], 2),
            "cols": 2,
            "rows": 1,
            "frames": 2,
        }

    def render_toggle_atlas(self, across: bool, cell_w: int, cell_h: int, anchor_xy, out_name: str):
        """Render 3 frames of chrome bat-handle toggle natively under fixed studio light:
        vertical (across=False): [0=DOWN, 1=MID, 2=UP]
        horizontal (across=True): [0=LEFT, 1=MID, 2=RIGHT]
        """
        t0 = time.time()
        scene, _ = reset_scene(self.samples_sprite, transparent=True)
        setup_studio_lights(scene, (0.0, 0.0, 0.0))
        add_toggle_reflection_cards(scene, across=across)
        add_shadow_catcher()

        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(self.assets_dir / "toggle_bat.glb"))
        added = [o for o in bpy.data.objects if o not in before]
        apply_switch_shaders(bpy.data.materials)

        lever = None
        for o in added:
            if o.type == "MESH" and "lever" in o.name.lower():
                lever = o

        ax, ay = anchor_xy
        cam_cx = (cell_w * 0.5 - ax) / self.px_per_m
        cam_cy = -(cell_h * 0.5 - ay) / self.px_per_m
        make_ortho_cam(scene, cam_cx, cam_cy, cell_w, cell_h, self.px_per_m)

        # In Blender Z-up (+Y is UP on screen, +X is RIGHT on screen):
        # IMPORTANT: glTF import sets rotation_mode='QUATERNION', so switch to 'XYZ' first!
        # Rotating lever around +X tilts its +Z tip toward -Y (DOWN on screen).
        # Rotating lever around +Y tilts its +Z tip toward +X (RIGHT on screen).
        if not across:
            # 0=DOWN (-Y), 1=MID (slight angle down), 2=UP (+Y)
            states = [(+31.0, 0.0, 0.0), (+7.0, 0.0, 0.0), (-31.0, 0.0, 0.0)]
        else:
            # 0=LEFT (-X), 1=MID (slight angle down), 2=RIGHT (+X)
            states = [(0.0, -31.0, 0.0), (+7.0, 0.0, 0.0), (0.0, +31.0, 0.0)]

        if lever is not None:
            lever.rotation_mode = "XYZ"

        atlas = np.zeros((cell_h, cell_w * 3, 4), dtype=np.float32)
        with tempfile.TemporaryDirectory() as td:
            tmp_png = Path(td) / "tg.png"
            scene.render.filepath = str(tmp_png)
            for i, (rx, ry, rz) in enumerate(states):
                if lever is not None:
                    lever.rotation_euler = (math.radians(rx), math.radians(ry), math.radians(rz))
                bpy.ops.render.render(write_still=True)
                atlas[:, i * cell_w : (i + 1) * cell_w] = read_png_rgba(tmp_png)

        save_rgba_png(atlas, self.out_dir / out_name, mode="RGBA")
        print(f"EXPORTED {out_name} (3 frames) in {time.time() - t0:.1f}s", flush=True)
        return {
            "file": out_name,
            "kind": "toggle",
            "across": across,
            "cell_w": cell_w,
            "cell_h": cell_h,
            "anchor": [ax, ay],
            "body_r": round(24.0 * self.px_per_unit, 2),
            "cols": 3,
            "rows": 1,
            "frames": 3,
        }

    def export_all(self, face_only=False):
        self.render_static_face()
        if face_only:
            print("CA72_FACE_ONLY_COMPLETE", flush=True)
            return

        cont_angles = [round(-150.0 + 300.0 * i / 48.0, 4) for i in range(49)]
        six_angles = [float(a) for a in self.spec["six_angles_deg"]]
        four_angles = six_angles[1:5]

        u = self.px_per_unit
        sprites = {}

        # 1. Classic Minimoog fluted skirted knobs (standard & big) - 49 frames (7x7)
        sprites["knob_std"] = self.render_knob_atlas(
            "knob_std", cont_angles, 224, (96, 96), 7, "knob-std-atlas.png", 54.0 * u
        )
        sprites["knob_big"] = self.render_knob_atlas(
            "knob_big", cont_angles, 256, (112, 112), 7, "knob-big-atlas.png", 65.0 * u
        )

        # 2. Classic Minimoog wedge-pointer rotary selectors (6 exact detents + 4 middle detents)
        dims = self.spec["part_dimensions_meters"]
        for part, detents, cell, anchor in (
            ("knob_pointer", six_angles, 256, 112),
            ("knob_pointer_small", four_angles, 192, 84),
        ):
            body_r = dims[part]["body_radius"] * self.px_per_m
            sprites[part] = self.render_knob_atlas(
                part, detents, cell, (anchor, anchor), len(detents),
                f"{part.replace('_', '-')}-atlas.png", body_r,
            )

        # 3. Classic Minimoog colored rockers (native horizontal & vertical orientations!)
        # Calibrated against the cerulean and orange rockers in the 1974 close-ups.
        sprites["rocker_horiz_blue"] = self.render_rocker_atlas(
            "rocker_horiz", "#3a7eb5", "right", 208, 128, (96, 56), (112 * u, 54 * u), "rocker-horiz-blue-atlas.png"
        )
        sprites["rocker_horiz_red"] = self.render_rocker_atlas(
            "rocker_horiz", "#f05d0c", "right", 208, 128, (96, 56), (112 * u, 54 * u), "rocker-horiz-red-atlas.png"
        )
        sprites["rocker_horiz_ivory"] = self.render_rocker_atlas(
            "rocker_horiz", "#dcd3ba", "right", 208, 128, (96, 56), (112 * u, 54 * u), "rocker-horiz-ivory-atlas.png"
        )
        sprites["rocker_vert_red"] = self.render_rocker_atlas(
            "rocker_vert", "#f05d0c", "top", 136, 208, (60, 96), (56 * u, 116 * u), "rocker-vert-red-atlas.png"
        )
        sprites["rocker_vert_blue"] = self.render_rocker_atlas(
            "rocker_vert", "#3a7eb5", "bottom", 136, 208, (60, 96), (56 * u, 116 * u), "rocker-vert-blue-atlas.png"
        )
        sprites["rocker_power"] = self.render_rocker_atlas(
            "rocker_power", None, "top", 120, 216, (52, 100), (48 * u, 124 * u), "rocker-power-atlas.png"
        )

        # 4. Classic chrome bat-handle toggles (vertical & horizontal)
        sprites["toggle_vert"] = self.render_toggle_atlas(False, 128, 168, (56, 76), "toggle-vert-atlas.png")
        sprites["toggle_horiz"] = self.render_toggle_atlas(True, 168, 128, (76, 56), "toggle-horiz-atlas.png")

        # Build manifest.json mapping every CA-72 panel control and lamp to exact 4K pixel coordinates
        controls_manifest = []
        for c in self.spec["controls"]:
            k = c["kind"]
            kt = k["type"]
            if kt == "key":
                continue  # TIMED / MOD dynamic keys are conditional in CA-72
            pos = self.panel_to_face_px(c["x_units"], c["y_units"])
            if kt == "knob_std":
                sprite_key = "knob_std"
                entry = {
                    "param": c["param"],
                    "label": c["label"],
                    "kind": "knob",
                    "sprite": sprite_key,
                    "pos": pos,
                    "hit_r": round(62.0 * u, 2),
                    "default": 0.5,
                }
            elif kt == "knob_big":
                entry = {
                    "param": c["param"],
                    "label": c["label"],
                    "kind": "knob",
                    "sprite": "knob_big",
                    "pos": pos,
                    "hit_r": round(73.0 * u, 2),
                    "default": 0.5,
                }
            elif kt in ("knob_pointer", "knob_pointer_small"):
                entry = {
                    "param": c["param"],
                    "label": c["label"],
                    "kind": "selector",
                    "sprite": kt,
                    "pos": pos,
                    "hit_r": round(60.0 * k["scale"] * u, 2),
                    "steps": len(k["places"]),
                    "places": k["places"],
                    "angles_deg": k["angles_deg"],
                    "default": 0.4,
                }
            elif kt == "rocker":
                col = k["colour"]
                ori = k["orient"]
                if ori == "right":
                    sprite_key = f"rocker_horiz_{col}"
                elif ori == "top":
                    sprite_key = "rocker_vert_red" if col == "red" else "rocker_vert_blue"
                else:
                    sprite_key = "rocker_vert_blue"
                entry = {
                    "param": c["param"],
                    "label": c["label"],
                    "kind": "rocker",
                    "sprite": sprite_key,
                    "pos": pos,
                    "hit_w": round(k["w_units"] * u, 2),
                    "hit_h": round(k["h_units"] * u, 2),
                    "orient": ori,
                    "colour": col,
                    "default": 0.0,
                }
            elif kt == "toggle":
                entry = {
                    "param": c["param"],
                    "label": c["label"],
                    "kind": "toggle",
                    "sprite": "toggle_horiz" if k["across"] else "toggle_vert",
                    "pos": pos,
                    "hit_w": round(k["w_units"] * u, 2),
                    "hit_h": round(k["h_units"] * u, 2),
                    "across": k["across"],
                    "steps": 3,
                    "default": 0.5,
                }
            else:
                continue
            controls_manifest.append(entry)

        # Add POWER rocker switch as an interactive fixture control
        fix = self.spec["fixtures"]
        controls_manifest.append({
            "param": "power",
            "label": "POWER",
            "kind": "rocker",
            "sprite": "rocker_power",
            "pos": self.panel_to_face_px(*fix["power"]),
            "hit_w": round(48.0 * u, 2),
            "hit_h": round(124.0 * u, 2),
            "orient": "top",
            "colour": "black",
            "default": 1.0,
        })

        # Physical multi-brightness linear-light lamps (rendered procedurally in Rust, no PNG halo!)
        lamps_manifest = [
            {
                "id": "power",
                "name": "POWER",
                "pos": self.panel_to_face_px(*fix["lamp"]),
                "dome_r": round(13.5 * u, 2),
                "bezel_outer_r": round(20.0 * u, 2),
                "color_srgb": [255, 36, 18],
                "core_srgb": [255, 224, 170],
                "faceted": True,
                "default_drive": 0.92,
            },
            {
                "id": "overload",
                "name": "OVERLOAD",
                "pos": self.panel_to_face_px(*fix["overload"]),
                "dome_r": round(16.0 * u, 2),
                "bezel_outer_r": round(24.0 * u, 2),
                "color_srgb": [255, 28, 12],
                "core_srgb": [255, 212, 148],
                "faceted": True,
                "default_drive": 0.0,
            },
        ]

        face_tl = self.panel_to_face_px(0.0, 0.0)
        face_br = self.panel_to_face_px(self.face_w_u, self.face_h_u)
        manifest = {
            "format": "ca72-3d-sprite-pack-v1",
            "projection": "flat-orthographic-0deg",
            "face": {
                "file": "face-3d.png",
                "w": self.face_w,
                "h": self.face_h,
                "px_per_m": round(self.px_per_m, 4),
                "px_per_unit": round(self.px_per_unit, 6),
                "faceplate_rect_px": [face_tl[0], face_tl[1], face_br[0], face_br[1]],
            },
            "sprites": sprites,
            "controls": controls_manifest,
            "lamps": lamps_manifest,
            "quality_opening": self.panel_to_face_px(*fix["ultra"]),
        }
        (self.out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        print("CA72_4K_EXPORT_COMPLETE", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=Path("target/panel-3d"))
    parser.add_argument("--face-w", type=int, default=int(os.environ.get("CA72_FACE_W", "3840")))
    parser.add_argument("--samples-face", type=int, default=int(os.environ.get("CA72_SAMPLES_FACE", "256")))
    parser.add_argument("--samples-sprite", type=int, default=int(os.environ.get("CA72_SAMPLES_SPRITE", "48")))
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
    args = parser.parse_args(argv)

    exp = PackExporter(args.out, args.face_w, args.samples_face, args.samples_sprite)
    exp.export_all(face_only=(os.environ.get("CA72_FACE_ONLY") == "1"))


if __name__ == "__main__":
    main()
