#!/usr/bin/env python3
"""Shared Cycles OPTIX + AgX studio lighting rig for CA-72 3D panel modeling & 4K export.

Key design principles to avoid the "shiny CG plastic" look while making real metal gleam:
  1. Soft, large upper-left overhead diffused studio key + gentle lower-right fill + restrained rim.
  2. World shader splits Diffuse vs Glossy rays via Light Path:
       - Diffuse rays see a quiet, warm-neutral studio room (deep rich blacks, crisp directional shadows).
       - Glossy rays see a photographic softbox-and-negative-fill studio cyclorama so chrome bat-toggles,
         knurled chrome bezels, 1/4" jacks, and radial spun-aluminum knob caps reflect realistic silver
         gradients instead of pitch black — while matte/satin plastics & paint stay dead-accurate matte!
  3. AgX / AgX - Medium High Contrast with calibrated exposure.
"""

import math
import bpy
from mathutils import Vector


def srgb_to_lin(c: float) -> float:
    return ((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92


def hex_to_lin(hx: str):
    r = int(hx[1:3], 16) / 255.0
    g = int(hx[3:5], 16) / 255.0
    b = int(hx[5:7], 16) / 255.0
    return (srgb_to_lin(r), srgb_to_lin(g), srgb_to_lin(b), 1.0)


def configure_cycles_device(scene):
    scene.render.engine = "CYCLES"
    prefs = bpy.context.preferences.addons["cycles"].preferences
    for backend in ("OPTIX", "CUDA"):
        try:
            prefs.compute_device_type = backend
            prefs.refresh_devices()
            gpu_devs = [d for d in prefs.devices if d.type == backend]
            if gpu_devs:
                for d in prefs.devices:
                    d.use = (d.type == backend)
                scene.cycles.device = "GPU"
                scene.cycles.use_denoising = True
                scene.cycles.denoiser = "OPTIX" if backend == "OPTIX" else "OPENIMAGEDENOISE"
                return backend
        except Exception:
            pass
    scene.cycles.device = "CPU"
    scene.cycles.use_denoising = True
    scene.cycles.denoiser = "OPENIMAGEDENOISE"
    return "CPU"


def reset_scene(samples: int = 128, transparent: bool = False):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    backend = configure_cycles_device(scene)
    scene.cycles.samples = samples
    scene.cycles.max_bounces = 8
    scene.cycles.diffuse_bounces = 4
    scene.cycles.glossy_bounces = 6
    scene.cycles.transmission_bounces = 8
    scene.cycles.transparent_max_bounces = 8
    scene.render.film_transparent = transparent
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA" if transparent else "RGB"
    scene.render.image_settings.compression = 85
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = -0.30
    return scene, backend


def setup_studio_lights(scene, center=(0.0, 0.0, 0.0)):
    """Photographic soft-studio lighting rig shared identically between static face,
    moving control atlases, and part verification renders.
    """
    world = bpy.data.worlds.new("StudioWorld")
    world.use_nodes = True
    scene.world = world
    nt = world.node_tree
    nt.nodes.clear()

    out = nt.nodes.new("ShaderNodeOutputWorld")
    lp = nt.nodes.new("ShaderNodeLightPath")
    mix = nt.nodes.new("ShaderNodeMixShader")

    # 1. Diffuse / Camera world: quiet warm-neutral studio room so blacks stay deep & grounded
    bg_diff = nt.nodes.new("ShaderNodeBackground")
    bg_diff.inputs["Color"].default_value = (0.022, 0.022, 0.024, 1.0)
    bg_diff.inputs["Strength"].default_value = 0.72

    # 2. Glossy-only photographic studio tent / cyclorama for real chrome & spun aluminum!
    #    Creates soft overhead-left studio softbox brightness transitioning to dark negative-fill
    #    flags on the lower-right, giving chrome bat handles, knurled bezels, and spun aluminum
    #    authentic high-contrast silver-and-charcoal product-photo reflections without washing out
    #    matte dielectrics!
    tc = nt.nodes.new("ShaderNodeTexCoord")
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    nt.links.new(tc.outputs["Generated"], sep.inputs["Vector"])

    # Combine upper-left gradient: bright overhead (+Z) and upper-left (-X, +Y), dark flag on right/bottom
    # dot = 0.62*Z + 0.34*Y - 0.30*X  (approx in [-1, 1])
    mul_z = nt.nodes.new("ShaderNodeMath")
    mul_z.operation = "MULTIPLY"
    mul_z.inputs[1].default_value = 0.68
    nt.links.new(sep.outputs["Z"], mul_z.inputs[0])

    mul_y = nt.nodes.new("ShaderNodeMath")
    mul_y.operation = "MULTIPLY"
    mul_y.inputs[1].default_value = 0.36
    nt.links.new(sep.outputs["Y"], mul_y.inputs[0])

    mul_x = nt.nodes.new("ShaderNodeMath")
    mul_x.operation = "MULTIPLY"
    mul_x.inputs[1].default_value = -0.32
    nt.links.new(sep.outputs["X"], mul_x.inputs[0])

    add1 = nt.nodes.new("ShaderNodeMath")
    add1.operation = "ADD"
    nt.links.new(mul_z.outputs["Value"], add1.inputs[0])
    nt.links.new(mul_y.outputs["Value"], add1.inputs[1])

    add2 = nt.nodes.new("ShaderNodeMath")
    add2.operation = "ADD"
    nt.links.new(add1.outputs["Value"], add2.inputs[0])
    nt.links.new(mul_x.outputs["Value"], add2.inputs[1])

    mr = nt.nodes.new("ShaderNodeMapRange")
    mr.inputs["From Min"].default_value = -0.65
    mr.inputs["From Max"].default_value = 0.92
    mr.inputs["To Min"].default_value = 0.0
    mr.inputs["To Max"].default_value = 1.0
    nt.links.new(add2.outputs["Value"], mr.inputs["Value"])

    # Color ramp: dark negative-fill charcoal -> mid studio grey -> bright softbox card -> warm highlight
    ramp = nt.nodes.new("ShaderNodeValToRGB")
    elems = ramp.color_ramp.elements
    elems[0].position = 0.0
    elems[0].color = (0.04, 0.045, 0.055, 1.0)
    elems[1].position = 0.92
    elems[1].color = (1.15, 1.12, 1.06, 1.0)
    e1 = elems.new(0.28)
    e1.color = (0.065, 0.072, 0.084, 1.0)
    e2 = elems.new(0.56)
    e2.color = (0.29, 0.31, 0.34, 1.0)
    e3 = elems.new(0.76)
    e3.color = (0.73, 0.75, 0.77, 1.0)
    nt.links.new(mr.outputs["Result"], ramp.inputs["Fac"])

    bg_gloss = nt.nodes.new("ShaderNodeBackground")
    bg_gloss.inputs["Strength"].default_value = 1.15
    nt.links.new(ramp.outputs["Color"], bg_gloss.inputs["Color"])

    # Use glossy cyclorama for Glossy & Transmission rays (so glass jewels & chrome catch studio light)
    ray_sel = nt.nodes.new("ShaderNodeMath")
    ray_sel.operation = "MAXIMUM"
    nt.links.new(lp.outputs["Is Glossy Ray"], ray_sel.inputs[0])
    nt.links.new(lp.outputs["Is Transmission Ray"], ray_sel.inputs[1])

    nt.links.new(ray_sel.outputs["Value"], mix.inputs["Fac"])
    nt.links.new(bg_diff.outputs["Background"], mix.inputs[1])
    nt.links.new(bg_gloss.outputs["Background"], mix.inputs[2])
    nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])

    cx, cy, cz = center

    def area(name, offset, energy, size_x, size_y, color, spread_deg=115.0):
        ldata = bpy.data.lights.new(name, type="AREA")
        ldata.energy = energy
        ldata.shape = "RECTANGLE"
        ldata.size = size_x
        ldata.size_y = size_y
        ldata.color = color
        if hasattr(ldata, "spread"):
            ldata.spread = math.radians(spread_deg)
        lobj = bpy.data.objects.new(name, ldata)
        scene.collection.objects.link(lobj)
        loc = Vector((cx + offset[0], cy + offset[1], cz + offset[2]))
        lobj.location = loc
        direction = Vector((cx, cy, cz)) - loc
        lobj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
        return lobj

    # A tall, broad window above the left side. Its narrow dimension leaves a
    # defined reflection on the curved grip shoulders and enamel crests.
    area("KeySoftbox",  (-0.24,  0.32, 0.60), 9.0, 0.58, 0.30, (1.00, 0.985, 0.96), spread_deg=120.0)
    # Wall bounce and a faint upper rim keep recesses readable.
    area("FillSoftbox", ( 0.44, -0.30, 0.52), 2.2, 0.88, 0.49, (0.93, 0.96, 1.00), spread_deg=130.0)
    area("RimStrip",    (-0.04,  0.54, 0.34), 1.5, 0.96, 0.20, (1.00, 0.97, 0.92), spread_deg=110.0)


def make_ortho_cam(scene, cx, cy, w_px, h_px, px_per_m):
    cam_data = bpy.data.cameras.new("OrthoCam")
    cam_data.type = "ORTHO"
    cam_data.clip_start = 0.01
    cam_data.clip_end = 5.0
    cam_data.sensor_fit = "HORIZONTAL"
    cam_data.ortho_scale = w_px / px_per_m
    cam = bpy.data.objects.new("OrthoCam", cam_data)
    scene.collection.objects.link(cam)
    cam.location = (cx, cy, 0.95)
    cam.rotation_euler = (0.0, 0.0, 0.0)
    scene.camera = cam
    scene.render.resolution_x = w_px
    scene.render.resolution_y = h_px
    scene.render.resolution_percentage = 100
    return cam
