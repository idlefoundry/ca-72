"""Cabinet and fixture materials shared by the static panel exporter and macro renders."""

from pathlib import Path

import bpy


# Linear RGB, roughness, metallic, and dielectric specular IOR level.
SURFACES = {
    "Faceplate_PBR": ((0.013, 0.012, 0.011), 0.58, 0.09, 0.11),
    "Walnut_Horiz": ((0.10, 0.062, 0.039), 0.60, 0.0, 0.25),
    "Walnut_Vert": ((0.10, 0.062, 0.039), 0.60, 0.0, 0.25),
    "Aluminum_Trim": ((0.66, 0.67, 0.68), 0.30, 0.97, 0.50),
    "Steel_Backing": ((0.016, 0.016, 0.017), 0.61, 0.20, 0.35),
    "Plate_Face": ((0.010, 0.010, 0.010), 0.45, 0.24, 0.11),
    "Plate_Chamfer": ((0.82, 0.84, 0.85), 0.23, 0.97, 0.50),
    "Chrome_Polished": ((0.61, 0.64, 0.67), 0.16, 0.98, 0.50),
    "Chrome_Satin": ((0.53, 0.55, 0.57), 0.20, 0.98, 0.50),
    "Jack_Bore": ((0.011, 0.011, 0.012), 0.79, 0.0, 0.24),
    "Jack_Contact": ((0.64, 0.61, 0.53), 0.22, 0.94, 0.50),
    "Jewel_Ruby_Unlit": ((0.76, 0.25, 0.27), 0.051, 0.0, 0.20),
    "Jewel_Overload_Unlit": ((0.63, 0.21, 0.22), 0.054, 0.0, 0.18),
    "Jewel_Reflector": ((0.067, 0.014, 0.012), 0.40, 0.62, 0.25),
    "Lamp_Retainer": ((0.031, 0.023, 0.015), 0.36, 0.62, 0.37),
    "Ultra_Amber_Unlit": ((0.72, 0.37, 0.13), 0.049, 0.0, 0.18),
    "Ultra_Bulb_Glass": ((0.98, 0.97, 0.95), 0.031, 0.0, 0.22),
    "Ultra_Brass_Cup": ((0.36, 0.20, 0.065), 0.34, 0.88, 0.40),
    "Ultra_Reflector": ((0.060, 0.033, 0.014), 0.46, 0.80, 0.28),
    "Ultra_Filament": ((0.48, 0.44, 0.35), 0.31, 0.90, 0.50),
    "Screw_BlackOxide": ((0.020, 0.021, 0.023), 0.43, 0.72, 0.26),
}

# Transmission weight and absorption coefficient per meter through the glass wall.
GLASS = {
    "Jewel_Ruby_Unlit": (0.98, 1.52, (0.96, 0.09, 0.09), 1450),
    "Jewel_Overload_Unlit": (0.98, 1.52, (0.91, 0.08, 0.08), 1780),
    "Ultra_Amber_Unlit": (0.92, 1.52, (0.97, 0.49, 0.085), 1050),
    "Ultra_Bulb_Glass": (1.0, 1.50, (1.0, 1.0, 1.0), 0),
}

TEXTURED = {
    "Faceplate_PBR": ("panel", 0.00019, 0.86, 1.0),
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


def tool_marks(tree, bsdf, coordinates, scale, spread, relief=0):
    stretch = tree.nodes.new("ShaderNodeVectorMath")
    stretch.operation = "MULTIPLY"
    stretch.inputs[1].default_value = scale
    tree.links.new(coordinates, stretch.inputs[0])

    noise = tree.nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = 1
    noise.inputs["Detail"].default_value = 2
    tree.links.new(stretch.outputs["Vector"], noise.inputs["Vector"])

    roughness = bsdf.inputs["Roughness"].default_value
    remap = tree.nodes.new("ShaderNodeMapRange")
    remap.inputs["To Min"].default_value = roughness - spread
    remap.inputs["To Max"].default_value = roughness + spread
    tree.links.new(noise.outputs["Fac"], remap.inputs["Value"])
    tree.links.new(remap.outputs["Result"], bsdf.inputs["Roughness"])
    if relief:
        bump = tree.nodes.new("ShaderNodeBump")
        bump.inputs["Distance"].default_value = relief
        tree.links.new(noise.outputs["Fac"], bump.inputs["Height"])
        tree.links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])


def apply_cabinet_shaders(materials, tex_dir):
    """Rebuild the Cycles graphs on imported/exporter-created cabinet materials in place."""
    tex_dir = Path(tex_dir).resolve()
    for material in materials:
        # Blender adds .001, .002, etc. when separate GLBs share a finish.
        name = material.name.partition(".")[0]
        if name not in SURFACES:
            continue
        tree, bsdf = surface(material, name)
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
        if name.startswith("Chrome_") or name == "Jack_Contact":
            position = tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
            tool_marks(tree, bsdf, position, (18000, 18000, 18000), 0.038)
        if name.startswith("Chrome_"):
            tangent = tree.nodes.new("ShaderNodeTangent")
            tangent.direction_type = "RADIAL"
            tangent.axis = "Z"
            tree.links.new(tangent.outputs["Tangent"], bsdf.inputs["Tangent"])
            bsdf.inputs["Anisotropic"].default_value = 0.18
        if name == "Screw_BlackOxide":
            position = tree.nodes.new("ShaderNodeTexCoord").outputs["Object"]
            tool_marks(tree, bsdf, position, (43000, 43000, 43000), 0.049, relief=0.0000024)
