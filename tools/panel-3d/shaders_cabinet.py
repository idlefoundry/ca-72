"""Cabinet and fixture materials shared by the static panel exporter and macro renders."""

from pathlib import Path

import bpy


# Linear RGB, roughness, metallic, and dielectric specular IOR level.
SURFACES = {
    "Faceplate_PBR": ((0.0065, 0.0065, 0.0065), 0.62, 0.0, 0.31),
    "Walnut_Horiz": ((0.10, 0.062, 0.039), 0.60, 0.0, 0.25),
    "Walnut_Vert": ((0.10, 0.062, 0.039), 0.60, 0.0, 0.25),
    "Aluminum_Trim": ((0.23, 0.225, 0.212), 0.37, 0.88, 0.50),
    "Steel_Backing": ((0.016, 0.016, 0.017), 0.61, 0.20, 0.35),
    "Plate_Face": ((0.005, 0.005, 0.0055), 0.45, 0.24, 0.085),
    "Plate_Chamfer": ((0.82, 0.84, 0.85), 0.23, 0.97, 0.50),
    "Chrome_Polished": ((0.86, 0.87, 0.88), 0.075, 0.99, 0.50),
    "Chrome_Satin": ((0.66, 0.67, 0.68), 0.16, 0.99, 0.50),
    "Pilot_Nickel": ((0.33, 0.31, 0.27), 0.26, 0.94, 0.50),
    "Overload_Bezel": ((0.13, 0.12, 0.10), 0.31, 0.86, 0.50),
    "Jack_Bore": ((0.011, 0.011, 0.012), 0.79, 0.0, 0.24),
    "Jack_Contact": ((0.58, 0.56, 0.50), 0.24, 0.94, 0.50),
    "Jewel_Ruby_Unlit": ((0.82, 0.34, 0.34), 0.026, 0.0, 0.075),
    "Jewel_Overload_Unlit": ((0.45, 0.20, 0.14), 0.044, 0.0, 0.060),
    "Jewel_Reflector": ((0.12, 0.017, 0.021), 0.32, 0.75, 0.28),
    "Jewel_Socket": ((0.029, 0.006, 0.009), 0.38, 0.46, 0.28),
    "Lamp_Retainer": ((0.021, 0.017, 0.014), 0.41, 0.52, 0.30),
    "Ultra_Amber_Unlit": ((0.94, 0.76, 0.41), 0.018, 0.0, 0.11),
    "Ultra_Bulb_Glass": ((0.98, 0.97, 0.95), 0.017, 0.0, 0.24),
    "Ultra_Brass_Cup": ((0.52, 0.31, 0.095), 0.23, 0.92, 0.40),
    "Ultra_Reflector": ((0.105, 0.065, 0.022), 0.31, 0.89, 0.28),
    "Ultra_Filament": ((0.56, 0.54, 0.49), 0.24, 0.88, 0.50),
    "Screw_BlackOxide": ((0.021, 0.022, 0.024), 0.37, 0.80, 0.26),
}

# Transmission weight and absorption coefficient per meter through the glass wall.
GLASS = {
    "Jewel_Ruby_Unlit": (0.98, 1.52, (0.97, 0.035, 0.065), 1350),
    "Jewel_Overload_Unlit": (0.98, 1.52, (0.94, 0.030, 0.055), 1550),
    "Ultra_Amber_Unlit": (0.98, 1.52, (0.99, 0.56, 0.085), 1050),
    "Ultra_Bulb_Glass": (1.0, 1.50, (1.0, 1.0, 1.0), 0),
}

TEXTURED = {
    "Faceplate_PBR": ("panel", 0.00009, 0.72, 1.0),
    "Walnut_Horiz": ("walnut", 0.000050, 0.48, 0.28),
    "Walnut_Vert": ("walnut", 0.000050, 0.48, 0.28),
    "Plate_Face": ("plate", 0.000012, 0.30, 1.0),
}


def surface(material, name):
    material.use_nodes = True
    tree = material.node_tree
    tree.nodes.clear()
    out = tree.nodes.new("ShaderNodeOutputMaterial")
    bsdf = tree.nodes.new("ShaderNodeBsdfPrincipled")
    color, roughness, metal, specular = SURFACES[name]
    material.diffuse_color = (*color, 1)
    material.roughness = roughness
    material.metallic = metal
    bsdf.inputs["Base Color"].default_value = material.diffuse_color
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metal
    bsdf.inputs["Specular IOR Level"].default_value = specular
    bsdf.inputs["Coat Weight"].default_value = 0
    tree.links.new(bsdf.outputs["BSDF"], out.inputs["Surface"])
    if name == "Faceplate_PBR":
        bsdf.inputs["IOR"].default_value = 1.46
    if name in GLASS:
        transmission, ior, tint, density = GLASS[name]
        bsdf.inputs["Transmission Weight"].default_value = transmission
        bsdf.inputs["IOR"].default_value = ior
        if density:
            absorption = tree.nodes.new("ShaderNodeVolumeAbsorption")
            absorption.inputs["Color"].default_value = (*tint, 1)
            absorption.inputs["Density"].default_value = density
            tree.links.new(absorption.outputs["Volume"], out.inputs["Volume"])
    return tree, bsdf


def make_material(name):
    material = bpy.data.materials.new(name)
    surface(material, name)
    return material


def lens_surface(tree, bsdf):
    """Keep optical refraction at the glass IOR and meter the reflected glare separately."""
    ior = bsdf.inputs["IOR"].default_value
    roughness = bsdf.inputs["Roughness"].default_value
    reflectance = 2 * bsdf.inputs["Specular IOR Level"].default_value
    transmission = bsdf.inputs["Transmission Weight"].default_value
    tint = bsdf.inputs["Base Color"].default_value
    refraction = tree.nodes.new("ShaderNodeBsdfRefraction")
    refraction.inputs["Color"].default_value = (*(channel * transmission for channel in tint[:3]), 1)
    refraction.inputs["IOR"].default_value = ior
    refraction.inputs["Roughness"].default_value = roughness
    reflection = tree.nodes.new("ShaderNodeBsdfAnisotropic")
    reflection.inputs["Roughness"].default_value = roughness

    fresnel = tree.nodes.new("ShaderNodeFresnel")
    fresnel.inputs["IOR"].default_value = ior
    weight = tree.nodes.new("ShaderNodeMath")
    weight.operation = "MULTIPLY_ADD"
    weight.inputs[1].default_value = 1 - reflectance
    weight.inputs[2].default_value = reflectance
    tree.links.new(fresnel.outputs["Fac"], weight.inputs[0])
    mix_weight = tree.nodes.new("ShaderNodeMath")
    mix_weight.operation = "MULTIPLY"
    tree.links.new(fresnel.outputs["Fac"], mix_weight.inputs[0])
    tree.links.new(weight.outputs["Value"], mix_weight.inputs[1])

    mix = tree.nodes.new("ShaderNodeMixShader")
    tree.links.new(mix_weight.outputs["Value"], mix.inputs[0])
    tree.links.new(refraction.outputs["BSDF"], mix.inputs[1])
    tree.links.new(reflection.outputs["BSDF"], mix.inputs[2])
    output = tree.nodes.get("Material Output")
    tree.links.new(mix.outputs["Shader"], output.inputs["Surface"])
    tree.nodes.remove(bsdf)


def image_maps(tree, bsdf, tex_dir, texture, distance, strength, reflectance_gain):
    uv = tree.nodes.new("ShaderNodeTexCoord").outputs["UV"]
    bump = tree.nodes.new("ShaderNodeBump")
    bump.inputs["Distance"].default_value = distance
    bump.inputs["Strength"].default_value = strength
    for suffix, destination in (
        ("albedo", bsdf.inputs["Base Color"]),
        ("roughness", bsdf.inputs["Roughness"]),
        ("bump", bump.inputs["Height"]),
    ):
        image = tree.nodes.new("ShaderNodeTexImage")
        image.label = f"{texture.capitalize()} {suffix}"
        image.image = bpy.data.images.load(str(tex_dir / f"{texture}_{suffix}.png"), check_existing=True)
        image.image.colorspace_settings.name = "sRGB" if suffix == "albedo" else "Non-Color"
        image.interpolation = "Linear" if suffix == "albedo" else "Cubic"
        tree.links.new(uv, image.inputs["Vector"])
        source = image.outputs["Color"]
        if suffix == "albedo" and reflectance_gain != 1.0:
            # The worn wood texture carries photographic exposure. Convert it to a
            # darker reflectance while preserving the source grain and chromaticity.
            exposure = tree.nodes.new("ShaderNodeVectorMath")
            exposure.operation = "SCALE"
            exposure.inputs["Scale"].default_value = reflectance_gain
            tree.links.new(source, exposure.inputs[0])
            source = exposure.outputs["Vector"]
        tree.links.new(source, destination)
    tree.links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])


def noise_field(tree, coordinates, scale):
    stretch = tree.nodes.new("ShaderNodeVectorMath")
    stretch.operation = "MULTIPLY"
    stretch.inputs[1].default_value = scale
    tree.links.new(coordinates, stretch.inputs[0])

    noise = tree.nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = 1
    noise.inputs["Detail"].default_value = 2
    tree.links.new(stretch.outputs["Vector"], noise.inputs["Vector"])
    return noise.outputs["Fac"]


def lathe_coordinates(tree, position):
    """Measure distance from the spindle and height along it, in meters."""
    xy = tree.nodes.new("ShaderNodeVectorMath")
    xy.operation = "MULTIPLY"
    xy.inputs[1].default_value = (1, 1, 0)
    tree.links.new(position, xy.inputs[0])
    radius = tree.nodes.new("ShaderNodeVectorMath")
    radius.operation = "LENGTH"
    tree.links.new(xy.outputs["Vector"], radius.inputs[0])
    axis = tree.nodes.new("ShaderNodeSeparateXYZ")
    tree.links.new(position, axis.inputs["Vector"])
    profile = tree.nodes.new("ShaderNodeCombineXYZ")
    tree.links.new(radius.outputs["Value"], profile.inputs["X"])
    tree.links.new(axis.outputs["Z"], profile.inputs["Y"])
    return profile.outputs["Vector"]


def tool_marks(tree, bsdf, coordinates, scale, spread, relief=0):
    noise = noise_field(tree, coordinates, scale)

    roughness = bsdf.inputs["Roughness"].default_value
    remap = tree.nodes.new("ShaderNodeMapRange")
    remap.inputs["To Min"].default_value = roughness - spread
    remap.inputs["To Max"].default_value = roughness + spread
    tree.links.new(noise, remap.inputs["Value"])
    tree.links.new(remap.outputs["Result"], bsdf.inputs["Roughness"])
    if relief:
        bump = tree.nodes.new("ShaderNodeBump")
        bump.inputs["Distance"].default_value = relief
        tree.links.new(noise, bump.inputs["Height"])
        tree.links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])


def black_oxide(tree, bsdf, position):
    mottling = noise_field(tree, position, (2700, 3100, 1900))
    color = tree.nodes.new("ShaderNodeMixRGB")
    color.inputs[1].default_value = (0.008, 0.008, 0.009, 1)
    color.inputs[2].default_value = (0.044, 0.043, 0.041, 1)
    tree.links.new(mottling, color.inputs["Fac"])
    tree.links.new(color.outputs["Color"], bsdf.inputs["Base Color"])
    profile = lathe_coordinates(tree, position)
    tool_marks(tree, bsdf, profile, (42000, 33500, 1), 0.07, relief=0.0000018)


def apply_cabinet_shaders(materials, tex_dir):
    """Rebuild the Cycles graphs on imported/exporter-created cabinet materials in place."""
    tex_dir = Path(tex_dir).resolve()
    for material in materials:
        # Blender adds .001, .002, etc. when separate GLBs share a finish.
        name = material.name.partition(".")[0]
        if name not in SURFACES:
            continue
        tree, bsdf = surface(material, name)
        if name in GLASS:
            lens_surface(tree, bsdf)
            continue
        if name in TEXTURED:
            image_maps(tree, bsdf, tex_dir, *TEXTURED[name])
        if name in ("Aluminum_Trim", "Plate_Face"):
            tangent = tree.nodes.new("ShaderNodeTangent")
            tangent.direction_type = "UV_MAP"
            tree.links.new(tangent.outputs["Tangent"], bsdf.inputs["Tangent"])
            bsdf.inputs["Anisotropic"].default_value = 0.42 if name == "Aluminum_Trim" else 0.29
        if name == "Aluminum_Trim":
            uv = tree.nodes.new("ShaderNodeTexCoord").outputs["UV"]
            tool_marks(tree, bsdf, uv, (28, 185, 1), 0.04)
        if name in ("Chrome_Polished", "Jack_Contact"):
            position = tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
            tool_marks(tree, bsdf, position, (8900, 12000, 10600), 0.02)
        if name in ("Chrome_Satin", "Screw_BlackOxide"):
            position = tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
            if name == "Chrome_Satin":
                profile = lathe_coordinates(tree, position)
                tool_marks(tree, bsdf, profile, (48000, 40500, 1), 0.02, relief=0.0000013)
            else:
                black_oxide(tree, bsdf, position)
        if name.startswith("Chrome_") or name == "Screw_BlackOxide":
            tangent = tree.nodes.new("ShaderNodeTangent")
            tangent.direction_type = "RADIAL"
            tangent.axis = "Z"
            tree.links.new(tangent.outputs["Tangent"], bsdf.inputs["Tangent"])
            bsdf.inputs["Anisotropic"].default_value = 0.12 if name == "Chrome_Polished" else 0.27
