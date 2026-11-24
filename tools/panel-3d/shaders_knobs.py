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
    THERMOSET: ((13, 13, 14), 0.29, 0.0, 0.45),
    CHAMFER: ((221, 221, 216), 0.23, 0.98, 0.5),
    ALUMINUM: ((226, 225, 222), 0.30, 1.0, 0.5),
    INDICATOR: ((232, 222, 199), 0.47, 0.0, 0.30),
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
        bsdf.inputs["IOR"].default_value = 1.52
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


def remap(tree, amount, limits, y, *, source=(0, 1)):
    mapped = tree.nodes.new("ShaderNodeMapRange")
    mapped.location = (-50, y)
    mapped.inputs["From Min"].default_value, mapped.inputs["From Max"].default_value = source
    mapped.inputs["To Min"].default_value, mapped.inputs["To Max"].default_value = limits
    mapped.interpolation_type = "SMOOTHSTEP"
    tree.links.new(amount, mapped.inputs["Value"])
    return mapped.outputs["Result"]


def color_variation(tree, amount, low, high, y):
    ramp = tree.nodes.new("ShaderNodeValToRGB")
    ramp.location = (-50, y)
    ramp.color_ramp.elements[0].color = linear_rgb(low)
    ramp.color_ramp.elements[1].color = linear_rgb(high)
    tree.links.new(amount, ramp.inputs["Fac"])
    return ramp.outputs["Color"]


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


def satin_reflection(tree, bsdf):
    # The broad center web of the black molding is quiet, but the rolled
    # shoulders keep enough grazing reflection to describe the scallops.
    facing = tree.nodes.new("ShaderNodeLayerWeight")
    facing.location = (-600, -480)
    tint = remap(tree, facing.outputs["Facing"], (0.38, 0.83), -490, source=(0.10, 0.37))
    tree.links.new(tint, bsdf.inputs["Specular Tint"])


def turning_marks(tree, coords, scale, distortion, label, y):
    lathe = tree.nodes.new("ShaderNodeTexWave")
    lathe.label = label
    lathe.location = (-600, y)
    lathe.wave_type = "RINGS"
    lathe.rings_direction = "Z"
    lathe.inputs["Scale"].default_value = scale
    lathe.inputs["Distortion"].default_value = distortion
    tree.links.new(coords, lathe.inputs["Vector"])
    return lathe.outputs["Fac"]


def crossgrain_scuffs(tree, coords):
    stretch = tree.nodes.new("ShaderNodeVectorMath")
    stretch.operation = "MULTIPLY"
    stretch.location = (-850, -1000)
    stretch.inputs[1].default_value = (0.30, 7.0, 1.0)
    tree.links.new(coords, stretch.inputs[0])
    scuffs = noise(tree, stretch.outputs["Vector"], 1100, "Fine crossgrain scuffs", -1000)
    return remap(tree, scuffs, (0.0, 1.0), -1000, source=(0.66, 0.80))


def directional_return(tree, coords, color):
    # A flat orthographic camera sees very little of the off-axis return from
    # the brushed grooves. Approximate the remaining two-lobe return in studio
    # coordinates so the highlights stay still as the part turns.
    frame = tree.nodes.new("ShaderNodeVectorTransform")
    frame.vector_type = "VECTOR"
    frame.convert_from = "OBJECT"
    frame.convert_to = "WORLD"
    frame.location = (-850, -1360)
    tree.links.new(coords, frame.inputs["Vector"])
    radial = tree.nodes.new("ShaderNodeVectorMath")
    radial.operation = "MULTIPLY"
    radial.location = (-625, -1360)
    radial.inputs[1].default_value = (1.0, 1.0, 0.0)
    tree.links.new(frame.outputs["Vector"], radial.inputs[0])
    normal = tree.nodes.new("ShaderNodeVectorMath")
    normal.operation = "NORMALIZE"
    normal.location = (-455, -1360)
    tree.links.new(radial.outputs["Vector"], normal.inputs[0])
    align = tree.nodes.new("ShaderNodeVectorMath")
    align.operation = "DOT_PRODUCT"
    align.location = (-290, -1360)
    align.inputs[1].default_value = (0.85, 0.53, 0.0)
    tree.links.new(normal.outputs["Vector"], align.inputs[0])
    paired = tree.nodes.new("ShaderNodeMath")
    paired.operation = "ABSOLUTE"
    paired.location = (-165, -1360)
    tree.links.new(align.outputs["Value"], paired.inputs[0])
    lobes = remap(tree, paired.outputs["Value"], (0.63, 1.06), -1360, source=(0.40, 0.96))
    return_color = tree.nodes.new("ShaderNodeVectorMath")
    return_color.operation = "SCALE"
    return_color.location = (135, 270)
    tree.links.new(color, return_color.inputs["Vector"])
    tree.links.new(lobes, return_color.inputs["Scale"])
    return return_color.outputs["Vector"]


def irregular_turning_marks(tree, coords):
    radial = tree.nodes.new("ShaderNodeVectorMath")
    radial.operation = "MULTIPLY"
    radial.location = (-650, -1080)
    radial.inputs[1].default_value = (1.0, 1.0, 0.0)
    tree.links.new(coords, radial.inputs[0])
    radius = tree.nodes.new("ShaderNodeVectorMath")
    radius.operation = "LENGTH"
    radius.location = (-460, -1100)
    tree.links.new(radial.outputs["Vector"], radius.inputs[0])
    marks = tree.nodes.new("ShaderNodeTexNoise")
    marks.label = "Irregular concentric tool marks"
    marks.location = (-280, -1260)
    marks.noise_dimensions = "1D"
    marks.inputs["Scale"].default_value = 12200
    marks.inputs["Detail"].default_value = 2.0
    tree.links.new(radius.outputs["Value"], marks.inputs["W"])
    return marks.outputs["Fac"]


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
            wear = noise(tree, local, 410, "Handling wear", 260)
            grain = noise(tree, local, 11800, "Compression-molded grain", -130)
            color = color_variation(tree, wear, (10, 10, 11), (19, 19, 20), 260)
            tree.links.new(color, bsdf.inputs["Base Color"])
            finish = blend_factors(tree, wear, grain, 0.32, -40)
            rough = remap(tree, finish, (0.22, 0.34), -30, source=(0.25, 0.76))
            tree.links.new(rough, bsdf.inputs["Roughness"])
            bump(tree, bsdf, grain, 0.25, 0.000035, -370)
            satin_reflection(tree, bsdf)
        elif name == INDICATOR:
            grain = noise(tree, local, 32000, "Pad-print ink grain", 260)
            tree.links.new(remap(tree, grain, (0.45, 0.51), 260), bsdf.inputs["Roughness"])
            bump(tree, bsdf, grain, 0.12, 0.000006, -70)
        elif name == ALUMINUM:
            spun.add(material)
            feed = turning_marks(tree, local, 645, 0.0, "Lathe feed", 260)
            brush = turning_marks(tree, local, 1980, 0.0, "Fine turning marks", -80)
            rings = blend_factors(tree, feed, brush, 0.80, 20)
            tool_marks = irregular_turning_marks(tree, local)
            rings = blend_factors(tree, rings, tool_marks, 0.59, -120)
            patina = noise(tree, local, 2050, "Oxidized aluminum mottling", -460)
            pits = noise(tree, local, 8600, "Fine oxide pits", -710)
            oxide = blend_factors(tree, patina, pits, 0.41, -600)
            scuffs = crossgrain_scuffs(tree, local)
            finish = blend_factors(tree, oxide, scuffs, 0.24, -440)
            polish = blend_factors(tree, rings, finish, 0.65, 20)
            rough = remap(tree, polish, (0.18, 0.43), 20, source=(0.23, 0.78))
            tree.links.new(rough, bsdf.inputs["Roughness"])
            grain = blend_factors(tree, rings, pits, 0.28, -250)
            bump(tree, bsdf, grain, 0.10, 0.000032, -270)
            clean_metal = remap(tree, finish, (1.0, 0.0), 260, source=(0.36, 0.66))
            color = color_variation(tree, clean_metal, (179, 177, 173), (237, 235, 230), 270)
            tree.links.new(directional_return(tree, local, color), bsdf.inputs["Base Color"])

            tangent = tree.nodes.new("ShaderNodeTangent")
            tangent.location = (-50, -490)
            tangent.direction_type = "RADIAL"
            tangent.axis = "Z"
            tree.links.new(tangent.outputs["Tangent"], bsdf.inputs["Tangent"])
            bsdf.inputs["Anisotropic"].default_value = 0.90

    # Radial Tangent uses Generated coordinates. Center the texture space on
    # the shaft, since the pointer fin moves the mesh's bounds off that axis.
    for mesh in bpy.data.meshes:
        if spun.intersection(mesh.materials):
            mesh.use_auto_texspace = False
            mesh.texspace_location = (0.0, 0.0, 0.0)
            mesh.texspace_size = (0.02, 0.02, 0.02)
