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
    THERMOSET: ((13, 13, 14), 0.32, 0.0, 0.42),
    CHAMFER: ((221, 221, 216), 0.26, 0.98, 0.5),
    ALUMINUM: ((225, 224, 221), 0.23, 1.0, 0.5),
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


def roughness_variation(tree, bsdf, amount, low, high, y):
    mapped = tree.nodes.new("ShaderNodeMapRange")
    mapped.location = (-50, y)
    mapped.inputs["To Min"].default_value = low
    mapped.inputs["To Max"].default_value = high
    tree.links.new(amount, mapped.inputs["Value"])
    tree.links.new(mapped.outputs["Result"], bsdf.inputs["Roughness"])


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


def satin_reflection(tree, bsdf):
    facing = tree.nodes.new("ShaderNodeLayerWeight")
    facing.location = (-600, -440)
    tint = tree.nodes.new("ShaderNodeMapRange")
    tint.label = "Satin faces and grazing edges"
    tint.location = (-325, -490)
    tint.interpolation_type = "SMOOTHSTEP"
    tint.inputs["From Min"].default_value = 0.045
    tint.inputs["From Max"].default_value = 0.32
    tint.inputs["To Min"].default_value = 0.30
    tint.inputs["To Max"].default_value = 0.86
    tree.links.new(facing.outputs["Facing"], tint.inputs["Value"])
    tree.links.new(tint.outputs["Result"], bsdf.inputs["Specular Tint"])


def angular_sheen(tree, coords):
    # Keep the reflected bands tied to the studio axes as the knob turns.
    frame = tree.nodes.new("ShaderNodeVectorTransform")
    frame.vector_type = "VECTOR"
    frame.convert_from = "OBJECT"
    frame.convert_to = "WORLD"
    frame.location = (-850, -790)
    tree.links.new(coords, frame.inputs["Vector"])
    split = tree.nodes.new("ShaderNodeSeparateXYZ")
    split.location = (-640, -790)
    tree.links.new(frame.outputs["Vector"], split.inputs["Vector"])
    angle = tree.nodes.new("ShaderNodeMath")
    angle.operation = "ARCTAN2"
    angle.location = (-470, -790)
    tree.links.new(split.outputs["Y"], angle.inputs[0])
    tree.links.new(split.outputs["X"], angle.inputs[1])
    doubled = tree.nodes.new("ShaderNodeMath")
    doubled.operation = "MULTIPLY_ADD"
    doubled.location = (-300, -790)
    doubled.inputs[1].default_value = 2.0
    doubled.inputs[2].default_value = 0.33
    tree.links.new(angle.outputs[0], doubled.inputs[0])
    sheen = tree.nodes.new("ShaderNodeMath")
    sheen.operation = "COSINE"
    sheen.location = (45, -790)
    tree.links.new(doubled.outputs[0], sheen.inputs[0])

    # Opposed bright sectors and grey sectors are still visible when the
    # broad area light's directly reflected streak is off the normal.
    ramp = tree.nodes.new("ShaderNodeMapRange")
    ramp.label = "Brushed aluminum directional return"
    ramp.location = (400, -660)
    ramp.interpolation_type = "SMOOTHSTEP"
    ramp.inputs["From Min"].default_value = -0.80
    ramp.inputs["From Max"].default_value = 0.85
    ramp.inputs["To Min"].default_value = 0.42
    ramp.inputs["To Max"].default_value = 1.32
    tree.links.new(sheen.outputs[0], ramp.inputs["Value"])
    return ramp.outputs["Result"]


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
            wear = noise(tree, local, 610, "Handling wear", 260)
            grain = noise(tree, local, 19800, "Compression-molded grain", -130)
            color = color_variation(tree, wear, (11, 11, 12), (18, 18, 19), 260)
            tree.links.new(color, bsdf.inputs["Base Color"])
            finish = blend_factors(tree, wear, grain, 0.25, -40)
            roughness_variation(tree, bsdf, finish, 0.27, 0.38, -30)
            bump(tree, bsdf, grain, 0.17, 0.000030, -370)
            satin_reflection(tree, bsdf)
        elif name == INDICATOR:
            grain = noise(tree, local, 32000, "Pad-print ink grain", 260)
            roughness_variation(tree, bsdf, grain, 0.45, 0.51, 260)
            bump(tree, bsdf, grain, 0.12, 0.000006, -70)
        elif name == ALUMINUM:
            spun.add(material)
            feed = turning_marks(tree, local, 645, 0.0, "Lathe feed", 260)
            brush = turning_marks(tree, local, 1980, 0.0, "Fine turning marks", -80)
            rings = blend_factors(tree, feed, brush, 0.80, 20)
            tool_marks = irregular_turning_marks(tree, local)
            rings = blend_factors(tree, rings, tool_marks, 0.59, -120)
            roughness_variation(tree, bsdf, rings, 0.19, 0.27, 20)
            bump(tree, bsdf, rings, 0.030, 0.000030, -270)
            patina = noise(tree, local, 880, "Faint aluminum patina", -460)
            finish = blend_factors(tree, rings, patina, 0.35, 260)
            color = color_variation(tree, finish, (209, 209, 206), (232, 231, 226), 270)
            contrast = tree.nodes.new("ShaderNodeMixRGB")
            contrast.blend_type = "MULTIPLY"
            contrast.location = (150, 240)
            contrast.inputs["Fac"].default_value = 1.0
            tree.links.new(color, contrast.inputs[1])
            tree.links.new(angular_sheen(tree, local), contrast.inputs[2])
            tree.links.new(contrast.outputs["Color"], bsdf.inputs["Base Color"])

            tangent = tree.nodes.new("ShaderNodeTangent")
            tangent.location = (-50, -490)
            tangent.direction_type = "RADIAL"
            tangent.axis = "Z"
            tree.links.new(tangent.outputs["Tangent"], bsdf.inputs["Tangent"])
            bsdf.inputs["Anisotropic"].default_value = 0.94
            bsdf.inputs["Anisotropic Rotation"].default_value = 0.25

    # Radial Tangent uses Generated coordinates. Center the texture space on
    # the shaft, since the pointer fin moves the mesh's bounds off that axis.
    for mesh in bpy.data.meshes:
        if spun.intersection(mesh.materials):
            mesh.use_auto_texspace = False
            mesh.texspace_location = (0.0, 0.0, 0.0)
            mesh.texspace_size = (0.02, 0.02, 0.02)
