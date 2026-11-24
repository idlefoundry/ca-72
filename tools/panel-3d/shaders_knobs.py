"""Portable knob material slots and their procedural Cycles finish.

Call apply_knob_shaders with the materials of the imported knob. glTF carries
the material names and flat PBR values, but drops the procedural detail.
"""

import bpy


THERMOSET = "Moog_Black_Thermoset"
CHAMFER = "Moog_Aluminum_Chamfer"
ALUMINUM = "Moog_Spun_Aluminum"
INDICATOR = "Moog_White_Indicator"

# Material order is also the order of the face slots in build_knobs.py.
FINISHES = {
    THERMOSET: ((24, 24, 25), 0.48, 0.0, 0.28),
    CHAMFER: ((238, 238, 236), 0.21, 0.97, 0.5),
    ALUMINUM: ((233, 233, 231), 0.25, 0.97, 0.5),
    INDICATOR: ((237, 234, 226), 0.48, 0.0, 0.30),
}


def linear_rgb(rgb):
    red, green, blue = (
        ((v / 255 + 0.055) / 1.055) ** 2.4 if v > 10 else v / (255 * 12.92)
        for v in rgb
    )
    return red, green, blue, 1.0


def base_shader(material, name):
    rgb, roughness, metallic, specular = FINISHES[name]
    material.use_nodes = True
    color = linear_rgb(rgb)
    material.diffuse_color = color
    tree = material.node_tree
    tree.nodes.clear()
    bsdf = tree.nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Specular IOR Level"].default_value = specular
    bsdf.inputs["Coat Weight"].default_value = 0.0
    if name == THERMOSET:
        bsdf.inputs["IOR"].default_value = 1.36
        bsdf.inputs["Diffuse Roughness"].default_value = 0.18
    out = tree.nodes.new("ShaderNodeOutputMaterial")
    out.location = (680, 260)
    bsdf.location = (350, 260)
    tree.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    return tree, bsdf


def knob_materials():
    """Create the glTF-compatible fallback finish for each material slot."""
    materials = []
    for name in FINISHES:
        material = bpy.data.materials.new(name)
        base_shader(material, name)
        materials.append(material)
    return materials


def noise(tree, coords, scale, label, y):
    node = tree.nodes.new("ShaderNodeTexNoise")
    node.label = label
    node.location = (-600, y)
    node.inputs["Scale"].default_value = scale
    node.inputs["Detail"].default_value = 2.0
    tree.links.new(coords, node.inputs["Vector"])
    return node.outputs["Fac"]


def roughness_variation(tree, bsdf, amount, low, high, y):
    mapped = tree.nodes.new("ShaderNodeMapRange")
    mapped.location = (-50, y)
    mapped.inputs["To Min"].default_value = low
    mapped.inputs["To Max"].default_value = high
    tree.links.new(amount, mapped.inputs["Value"])
    tree.links.new(mapped.outputs["Result"], bsdf.inputs["Roughness"])


def color_variation(tree, bsdf, amount, low, high, y):
    ramp = tree.nodes.new("ShaderNodeValToRGB")
    ramp.location = (-50, y)
    ramp.color_ramp.elements[0].color = linear_rgb(low)
    ramp.color_ramp.elements[1].color = linear_rgb(high)
    tree.links.new(amount, ramp.inputs["Fac"])
    tree.links.new(ramp.outputs["Color"], bsdf.inputs["Base Color"])


def blend_factors(tree, first, second, weight, y):
    blend = tree.nodes.new("ShaderNodeMixRGB")
    blend.location = (-325, y)
    blend.inputs["Fac"].default_value = weight
    tree.links.new(first, blend.inputs[1])
    tree.links.new(second, blend.inputs[2])
    return blend.outputs["Color"]


def bump(tree, bsdf, height, strength, distance, y):
    node = tree.nodes.new("ShaderNodeBump")
    node.location = (-50, y)
    node.inputs["Strength"].default_value = strength
    node.inputs["Distance"].default_value = distance
    tree.links.new(height, node.inputs["Height"])
    tree.links.new(node.outputs["Normal"], bsdf.inputs["Normal"])


def apply_knob_shaders(materials):
    """Restore the Cycles finish after GLB import, including on duplicated slots."""
    spun = set()
    for material in set(materials):
        if material is None:
            continue
        name = material.name.split(".")[0]
        if name not in FINISHES:
            continue
        tree, bsdf = base_shader(material, name)
        if name == CHAMFER:
            continue

        coords = tree.nodes.new("ShaderNodeTexCoord")
        coords.location = (-850, 260)
        local = coords.outputs["Object"]

        if name == THERMOSET:
            wear = noise(tree, local, 610, "Soft handling wear", 260)
            grain = noise(tree, local, 17500, "Compression-molded surface", -130)
            color_variation(tree, bsdf, wear, (22, 22, 23), (26, 26, 26), 260)
            finish = blend_factors(tree, wear, grain, 0.35, -40)
            roughness_variation(tree, bsdf, finish, 0.47, 0.50, -30)
            bump(tree, bsdf, grain, 0.40, 0.000026, -370)
        elif name == INDICATOR:
            grain = noise(tree, local, 32000, "Pad-print ink grain", 260)
            roughness_variation(tree, bsdf, grain, 0.45, 0.51, 260)
            bump(tree, bsdf, grain, 0.12, 0.000006, -70)
        elif name == ALUMINUM:
            spun.add(material)
            lathe = tree.nodes.new("ShaderNodeTexWave")
            lathe.label = "Concentric turning marks"
            lathe.location = (-600, 250)
            lathe.wave_type = "RINGS"
            lathe.rings_direction = "Z"
            lathe.inputs["Scale"].default_value = 2200
            lathe.inputs["Distortion"].default_value = 0.065
            tree.links.new(local, lathe.inputs["Vector"])
            rings = lathe.outputs["Fac"]
            roughness_variation(tree, bsdf, rings, 0.238, 0.266, 20)
            bump(tree, bsdf, rings, 0.012, 0.000045, -270)
            patina = noise(tree, local, 690, "Faint aluminum patina", -165)
            finish = blend_factors(tree, patina, rings, 0.48, 260)
            color_variation(tree, bsdf, finish, (228, 228, 226), (236, 236, 234), 270)

            tangent = tree.nodes.new("ShaderNodeTangent")
            tangent.location = (-50, -490)
            tangent.direction_type = "RADIAL"
            tangent.axis = "Z"
            tree.links.new(tangent.outputs["Tangent"], bsdf.inputs["Tangent"])
            bsdf.inputs["Anisotropic"].default_value = 0.85
            bsdf.inputs["Anisotropic Rotation"].default_value = 0.25

    # Radial Tangent uses Generated coordinates. Center the texture space on
    # the shaft, since the pointer fin moves the mesh's bounds off that axis.
    for mesh in bpy.data.meshes:
        if spun.intersection(mesh.materials):
            mesh.use_auto_texspace = False
            mesh.texspace_location = (0.0, 0.0, 0.0)
            mesh.texspace_size = (0.02, 0.02, 0.02)
