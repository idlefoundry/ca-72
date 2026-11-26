"""Metric hardware for the lower panel's left-hand controls."""

import math

import bpy

from build_cabinet import beveled_box, build_screw_phillips, lathe
from build_switches import (
    build_rocker_actuator,
    build_rocker_bezel,
    build_toggle,
    create_mesh,
    loft,
    rounded_rectangle,
)
from shaders_cabinet import black_oxide, lathe_coordinates, make_material, tool_marks
from shaders_switches import apply_switch_shaders, new_switch_material


def plastic(name, color, roughness, specular):
    material = bpy.data.materials.new(name)
    material.diffuse_color = (*color, 1.0)
    material.use_nodes = True
    surface = material.node_tree.nodes.get("Principled BSDF")
    surface.inputs["Base Color"].default_value = material.diffuse_color
    surface.inputs["Roughness"].default_value = roughness
    surface.inputs["Specular IOR Level"].default_value = specular
    return material, surface


def wheel_slot(width, height, rim, lining):
    """Draw a rolled edge into a dark liner fitted just above the plate."""
    w, h = width / 2, height / 2
    floor = 0.00011
    return loft(
        "Wheel_recess",
        [
            rounded_rectangle(w - 0.00015, h - 0.00015, 0, 0.0025),
            rounded_rectangle(w, h, 0.00034, 0.0027),
            rounded_rectangle(w - 0.00032, h - 0.00032, 0.00073, 0.0025),
            rounded_rectangle(w - 0.00112, h - 0.00112, 0.00065, 0.0020),
            rounded_rectangle(w - 0.00144, h - 0.00144, floor, 0.0019),
        ],
        [rim, lining],
        caps=(0, floor),
        span_materials=[0, 0, 0, 1],
    )


TEETH = 45


def grip_radius(radius, angle):
    # Broad crowns between rounded valleys, cut across the rim of the molding.
    phase = abs((angle * TEETH / math.tau + 0.5) % 1 - 0.5) * 2
    valley = max(0.0, (phase - 0.46) / 0.54)
    return radius - 0.00059 * valley * valley * (3 - 2 * valley)


def wheel_body(radius, width, material):
    angles = [math.tau * i / (TEETH * 16) for i in range(TEETH * 16)]
    half = width / 2
    profile = [
        (-half, 0.00165), (-half + 0.00021, 0.00105),
        (-half + 0.00062, 0.00043), (-half + 0.00121, 0.0),
        (half - 0.00121, 0.0), (half - 0.00062, 0.00043),
        (half - 0.00021, 0.00105), (half, 0.00165),
    ]
    verts = []
    for x, bevel in profile:
        for angle in angles:
            r = grip_radius(radius, angle) - bevel
            verts.append((x, r * math.sin(angle), r * math.cos(angle)))
    step = len(angles)
    faces = [tuple(range(step - 1, -1, -1))]
    for first in range(0, len(verts) - step, step):
        for i in range(step):
            j = (i + 1) % step
            faces.append((first + i, first + j, first + step + j, first + step + i))
    faces.append(tuple(range(len(verts) - step, len(verts))))
    return create_mesh("Ivory_wheel", verts, faces, [material], sharp_angle=22)


def wheel_mark(radius, width, dot, material):
    """A thin ink index follows the relief of the ribbed wheel."""
    center = math.radians(-7) if dot else 0.0
    half_length = 0.00134 if dot else width / 2 - 0.0018
    half_height = (0.00134 if dot else 0.00060) / radius
    angles = [math.tau * i / 48 for i in range(48)]
    def point(x, angle, lift):
        r = grip_radius(radius, angle) + lift
        return x, r * math.sin(angle), r * math.cos(angle)

    outline = [(math.cos(angle), math.sin(angle)) for angle in angles]
    if not dot:
        outline = [
            (math.copysign(abs(x) ** 0.19, x), math.copysign(abs(y) ** 0.19, y))
            for x, y in outline
        ]
    rings = []
    # Several concentric contours keep the dot's face from crossing into the
    # plastic where it bridges a rib against the black mark.
    for lift, fractions in ((-0.000045, (1,)), (0.000045, (1, 0.75, 0.5, 0.25))):
        rings.extend([
            [point(half_length * x * fraction, center + half_height * y * fraction, lift)
             for x, y in outline]
            for fraction in fractions
        ])
    verts = [vertex for ring in rings for vertex in ring]
    top = len(verts)
    verts.append(point(0, center, 0.000045))
    step = len(angles)
    faces = [tuple(range(step - 1, -1, -1))]
    for first in range(0, top - step, step):
        for i in range(step):
            j = (i + 1) % step
            faces.append((first + i, first + j, first + step + j, first + step + i))
    end = top - step
    for i in range(step):
        faces.append((end + i, end + (i + 1) % step, top))
    return create_mesh("Wheel_index", verts, faces, [material], sharp_angle=35)


def build_wheel(control, unit, mats, at):
    w, h = control["w"] * unit, control["h"] * unit
    slot = wheel_slot(w, h, mats["rim"], mats["lining"])
    slot.location = at
    radius = h * 84 / 146
    tyre_width = w * 0.79
    visible_half = h / 2 - 0.00255
    axis_z = 0.00011 - math.sqrt(radius ** 2 - visible_half ** 2)
    dot = control.get("mark", "dot" if control.get("param") == "mod_wheel" else "line") == "dot"
    span = control.get("span_deg", 76.0 if dot else 70.0)
    value = control.get("value", 0.0 if dot else 0.5)
    turn = math.radians((0.5 - value) * span)
    for obj in (
        wheel_body(radius, tyre_width, mats["ivory"]),
        wheel_mark(radius, tyre_width, dot, mats["index"]),
    ):
        obj.location = (*at[:2], at[2] + axis_z)
        obj.rotation_euler.x = turn


def build_jack(control, unit, mats, at):
    r = control["w"] * unit / 2
    bore = r * 0.37
    sleeve = r * 0.64

    def hex_edge(angle):
        sector = (angle + math.pi / 6) % (math.pi / 3) - math.pi / 6
        return r * 0.85 / math.cos(min(math.radians(26), abs(sector)))

    # Turned chamfers over a hex panel nut, with a separate nickel lead-in.
    nut = lathe(
        "Control_jack_nut",
        [
            (sleeve, 0.00008), (r * 0.89, 0.00008),
            (hex_edge, 0.00033), (hex_edge, 0.00113), (r * 0.83, 0.00186),
            (sleeve, 0.00190), (sleeve, 0.00036),
        ],
        mats["nickel"], segments=120, closed=True,
    )
    collar = lathe(
        "Control_jack_shoulder",
        [
            (bore, 0.00105), (sleeve + 0.00008, 0.00105), (sleeve + 0.00008, 0.00186),
            (sleeve - 0.00012, 0.00211), (bore + 0.00054, 0.00211),
            (bore + 0.00017, 0.00183), (bore, 0.00149),
        ],
        mats["chrome"], segments=96, closed=True,
    )
    socket = lathe(
        "Control_jack_bore",
        [(0.0, 0.00015), (bore + 0.00013, 0.00015), (bore + 0.00013, 0.00149),
         (bore, 0.00144), (bore * 0.94, 0.00032), (0.0, 0.00032)],
        mats["bore"], segments=96,
    )
    for obj in (nut, collar, socket):
        obj.location = at


def build_priority(control, unit, mats, at):
    mount, lever = build_toggle(
        {
            "nut_across_flats": 36.0 * unit,
            "washer_radius": 22.0 * unit,
            "bushing_radius": 11.5 * unit,
            "lever_length": 0.0168,
            "lever_base_radius": 0.00135,
            "lever_tip_radius": 0.0027,
        },
        mats["Nickel_Chrome"], mats["Nickel_Chrome_Satin"], mats["Toggle_Inner_Dark"],
    )
    mount.location = at
    mount.rotation_euler.z = math.pi / 2
    lever.location = (*at[:2], at[2] + lever.location.z)
    val = control.get("value", 0.5)
    if 0.25 < val < 0.75:
        lever.rotation_euler.x = math.radians(7)
    else:
        lever.rotation_euler.y = math.radians(-31 if val <= 0.25 else 31)


def build_ivory_rocker(control, unit, mats, at):
    w, h = control["w"] * unit, control["h"] * unit
    dim = {"w": w, "h": h, "bezel_pad": 0.00074, "height": w * 0.347}
    bezel = build_rocker_bezel(dim, mats["Rocker_Bezel"], mats["Rocker_Well"], power=False)
    bezel.location = at
    actuator = build_rocker_actuator(dim, mats["Rocker_Actuator"], vertical=False)
    actuator.location = (*at[:2], at[2] + actuator.location.z)
    on = control.get("lit", control.get("value", 0.0) >= 0.5)
    actuator.rotation_euler.y = math.radians(17.5 if on else -17.5)


def build_velocity_lamp(control, unit, mats, at):
    w, h = control["w"] * unit, control["h"] * unit
    x, y, z = at
    beveled_box("Velocity_lamp_frame", (w, h, 0.00042), (x, y, z + 0.00021), mats["rim"], 0.00018)
    material = mats["velocity_on" if control.get("lit", False) else "velocity_off"]
    beveled_box(
        "Velocity_lamp_lens", (w - 0.00068, h - 0.00068, 0.00040),
        (x, y, z + 0.00048), material, 0.00017,
    )


def materials():
    rim, _ = plastic("Wheel_mounting_rim", (0.014, 0.013, 0.012), 0.47, 0.30)
    lining, _ = plastic("Wheel_slot_lining", (0.0045, 0.004, 0.0035), 0.77, 0.18)
    ivory, _ = plastic("Molded_ivory_wheel", (0.55, 0.48, 0.35), 0.34, 0.31)
    index, _ = plastic("Wheel_black_index", (0.012, 0.011, 0.010), 0.46, 0.22)
    on, emit = plastic("Velocity_amber_on", (0.37, 0.074, 0.005), 0.42, 0.17)
    emit.inputs["Emission Color"].default_value = (0.9, 0.145, 0.01, 1.0)
    emit.inputs["Emission Strength"].default_value = 0.39
    off, _ = plastic("Velocity_amber_off", (0.053, 0.018, 0.004), 0.38, 0.26)
    result = {
        "rim": rim, "lining": lining, "ivory": ivory, "index": index,
        "velocity_on": on, "velocity_off": off,
        "nickel": make_material("Chrome_Satin"), "chrome": make_material("Chrome_Polished"),
        "bore": make_material("Jack_Bore"), "screw": make_material("Screw_BlackOxide"),
    }
    for material in (ivory, result["nickel"], result["screw"]):
        tree = material.node_tree
        surface = next(node for node in tree.nodes if node.type == "BSDF_PRINCIPLED")
        coords = tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
        if material == ivory:
            tool_marks(tree, surface, coords, (3200, 4600, 4600), 0.045, relief=0.000012)
        elif material == result["nickel"]:
            tool_marks(tree, surface, lathe_coordinates(tree, coords), (18000, 27000, 1), 0.065)
        else:
            black_oxide(tree, surface, coords)
    switches = {
        name: new_switch_material(name) for name in (
            "Nickel_Chrome", "Nickel_Chrome_Satin", "Toggle_Inner_Dark",
            "Rocker_Bezel", "Rocker_Well", "Rocker_Actuator",
        )
    }
    apply_switch_shaders(switches.values(), color_hex="#dcd3ba")
    return result | switches


def build_lower_left(spec, scale, panel_to_blender):
    """Build left-hand hardware at the centers exported by CA-72's layout."""
    mats = materials()
    builders = {
        "wheel": build_wheel,
        "jack": build_jack,
        "toggle": build_priority,
        "rocker": build_ivory_rocker,
        "velocity": build_velocity_lamp,
    }
    for control in spec["controls"]:
        kind = control["kind"]
        at = panel_to_blender(control["x"], control["y"])
        if kind in builders:
            builders[kind](control, scale, mats, at)
        elif kind == "screw":
            screw = build_screw_phillips(
                {"head_radius": control["w"] * scale / 2, "height": 0.00084}, mats["screw"],
            )
            screw.location = at
