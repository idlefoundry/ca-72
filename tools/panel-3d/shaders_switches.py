"""GLB material defaults and the switch finishes restored for Cycles."""

import math

import bpy


# Editable glTF PBR defaults also serve as the baseline for Cycles.
# Values are sRGB hex, roughness, metallic and dielectric specular IOR level.
FINISHES = {
    "Rocker_Actuator": ("#3a7eb5", 0.41, 0.0, 0.22),
    "Rocker_Power_Black": ("#181819", 0.42, 0.0, 0.18),
    "Rocker_Bezel": ("#101011", 0.51, 0.0, 0.18),
    "Rocker_Well": ("#0d0d0e", 0.82, 0.0, 0.10),
    "Nickel_Chrome": ("#b5b4b1", 0.09, 0.99, 0.50),
    "Nickel_Chrome_Satin": ("#a1a2a0", 0.17, 0.99, 0.50),
    "Toggle_Inner_Dark": ("#292729", 0.43, 0.75, 0.50),
}


def _linear_rgba(color_hex, reflectance=1.0):
    channels = (int(color_hex[i:i + 2], 16) / 255.0 for i in (1, 3, 5))
    linear = tuple(
        ((c + 0.055) / 1.055) ** 2.4 if c > 0.04045 else c / 12.92
        for c in channels
    )
    return (*(c * reflectance for c in linear), 1.0)


def _base_finish(material, name, color_hex=None):
    color, roughness, metallic, specular = FINISHES[name]
    # Calibrate the nylon pigment in linear light so custom tints keep their hue
    # under the full-energy studio rig. glTF defaults use the same reflectance.
    reflectance = 0.40 if name == "Rocker_Actuator" else 1.0
    rgba = _linear_rgba(color_hex or color, reflectance)
    material.use_nodes = True
    material.diffuse_color = rgba
    material.roughness = roughness
    material.metallic = metallic
    tree = material.node_tree
    tree.nodes.clear()
    bsdf = tree.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.location = (320, 0)
    output = tree.nodes.new("ShaderNodeOutputMaterial")
    output.location = (670, 0)
    tree.links.new(bsdf.outputs["BSDF"], output.inputs["Surface"])
    bsdf.inputs["Base Color"].default_value = rgba
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Specular IOR Level"].default_value = specular
    bsdf.inputs["Coat Weight"].default_value = 0.0
    return tree, bsdf


def new_switch_material(name):
    """Create a material using only inputs that survive glTF export."""
    material = bpy.data.materials.new(name)
    _base_finish(material, name)
    return material


def _noise(tree, vector, label, scale, detail, position):
    node = tree.nodes.new("ShaderNodeTexNoise")
    node.label = label
    node.location = position
    node.inputs["Scale"].default_value = scale
    node.inputs["Detail"].default_value = detail
    tree.links.new(vector, node.inputs["Vector"])
    return node.outputs["Fac"]


def _map(tree, signal, label, low, high, position):
    node = tree.nodes.new("ShaderNodeMapRange")
    node.label = label
    node.location = position
    # Noise is concentrated near 0.5. Expand the useful middle of its range.
    node.inputs["From Min"].default_value = 0.24
    node.inputs["From Max"].default_value = 0.76
    node.inputs["To Min"].default_value = low
    node.inputs["To Max"].default_value = high
    tree.links.new(signal, node.inputs["Value"])
    return node.outputs["Result"]


def _bump(tree, bsdf, height, strength, distance):
    node = tree.nodes.new("ShaderNodeBump")
    node.location = (80, -340)
    node.inputs["Strength"].default_value = strength
    node.inputs["Distance"].default_value = distance
    tree.links.new(height, node.inputs["Height"])
    tree.links.new(node.outputs["Normal"], bsdf.inputs["Normal"])


def _nylon(tree, bsdf, vector, colored):
    grain = _noise(tree, vector, "Fine mould grain", 6100, 1.7, (-690, -390))
    wear = _noise(tree, vector, "Uneven satin finish", 1450, 2.1, (-690, 0))
    roughness = bsdf.inputs["Roughness"].default_value
    rough = _map(tree, wear, "Satin roughness", roughness - 0.04, roughness + 0.04, (-390, 0))
    tree.links.new(rough, bsdf.inputs["Roughness"])
    _bump(tree, bsdf, grain, 0.31, 0.000033)

    pigment = tree.nodes.new("ShaderNodeMixRGB")
    pigment.label = "Moulding and handling variation"
    pigment.location = (65, 105)
    pigment.blend_type = "MULTIPLY"
    pigment.inputs[0].default_value = 1.0
    pigment.inputs[1].default_value = bsdf.inputs["Base Color"].default_value
    variation = _map(tree, wear, "Pigment variation", 0.90, 1.05, (-390, 285))
    tree.links.new(variation, pigment.inputs[2])
    tree.links.new(pigment.outputs[0], bsdf.inputs["Base Color"])

    if colored:
        bsdf.inputs["Subsurface Weight"].default_value = 0.025
        bsdf.inputs["Subsurface Radius"].default_value = (1.0, 0.44, 0.24)
        bsdf.inputs["Subsurface Scale"].default_value = 0.00028


def _chrome(tree, bsdf, vector, satin):
    wear = _noise(tree, vector, "Plating and handling variation", 1950, 2.0, (-690, 0))
    roughness = bsdf.inputs["Roughness"].default_value
    spread = 0.026 if satin else 0.018
    rough = _map(tree, wear, "Plating roughness", roughness - spread, roughness + spread, (-390, 0))
    tree.links.new(rough, bsdf.inputs["Roughness"])

    if satin:
        tangent = tree.nodes.new("ShaderNodeTangent")
        tangent.direction_type = "RADIAL"
        tangent.axis = "Z"
        tangent.location = (80, 155)
        bsdf.inputs["Anisotropic"].default_value = 0.29
        tree.links.new(tangent.outputs["Tangent"], bsdf.inputs["Tangent"])

        tooling = tree.nodes.new("ShaderNodeTexWave")
        tooling.label = "Circular machining marks"
        tooling.location = (-400, -280)
        tooling.wave_type = "RINGS"
        tooling.rings_direction = "Z"
        tooling.inputs["Scale"].default_value = 2200
        tooling.inputs["Distortion"].default_value = 2.2
        tooling.inputs["Detail Scale"].default_value = 0.24
        tree.links.new(vector, tooling.inputs["Vector"])
        _bump(tree, bsdf, tooling.outputs["Fac"], 0.25, 0.000006)


def apply_switch_shaders(materials, color_hex=None):
    """Restore switch finishes just after glTF import, with optional rocker tint.

    Accepts an iterable of Blender materials. Other materials are left alone.
    Blender's numeric name suffix on repeated imports has no effect.
    """
    for material in materials:
        if material is None:
            continue
        name = material.name.partition(".")[0]
        if name not in FINISHES:
            continue
        colored = name == "Rocker_Actuator"
        tree, bsdf = _base_finish(material, name, color_hex if colored else None)
        if name in {"Rocker_Well", "Toggle_Inner_Dark"}:
            continue
        coordinates = tree.nodes.new("ShaderNodeTexCoord")
        coordinates.location = (-920, 0)
        # Local metric coordinates keep grain fixed to the switch while it moves.
        vector = coordinates.outputs["Object"]
        if name.startswith("Rocker_"):
            _nylon(tree, bsdf, vector, colored)
        else:
            _chrome(tree, bsdf, vector, satin=name == "Nickel_Chrome_Satin")


def add_toggle_reflection_cards(scene, *, across=False):
    """Set up low bounce cards for a toggle at the origin, over a dark plate.

    The top of the lever sees the overhead softboxes. Its sloping sides see
    mostly the plate, so low white cards add a narrow silver band to the shaft.
    Use this once in a toggle atlas scene, with the studio lights still in place.
    """
    white = bpy.data.materials.new("Toggle_Bounce_Card")
    white.use_nodes = True
    tree = white.node_tree
    tree.nodes.clear()
    emission = tree.nodes.new("ShaderNodeEmission")
    emission.inputs["Color"].default_value = (1.0, 0.97, 0.94, 1.0)
    # Retain the shaft's silver band with the lower-reflectance chrome finish.
    emission.inputs["Strength"].default_value = 1.20
    output = tree.nodes.new("ShaderNodeOutputMaterial")
    tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])
    white.cycles.emission_sampling = "NONE"

    # Both detents see a card. Glossy visibility preserves the actual reflection
    # and keeps the cards out of the image, the cast shadow and the diffuse fill.
    for side in (-1, 1):
        mesh = bpy.data.meshes.new("Toggle_Bounce_Card")
        mesh.from_pydata([(-0.005, -0.012, 0), (0.005, -0.012, 0),
                          (0.005, 0.012, 0), (-0.005, 0.012, 0)], [], [(0, 1, 2, 3)])
        mesh.materials.append(white)
        card = bpy.data.objects.new("Toggle_Bounce_Card", mesh)
        scene.collection.objects.link(card)
        offset, lengthwise = -0.0038, side * 0.018
        card.location = (-lengthwise, offset, 0.00019) if across else (offset, lengthwise, 0.00019)
        card.rotation_euler.z = math.pi / 2 if across else 0
        card.visible_camera = False
        card.visible_diffuse = False
        card.visible_shadow = False
        card.visible_transmission = False
