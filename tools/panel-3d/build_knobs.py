#!/usr/bin/env python3
"""Build metric knob GLBs from spec.json. Use --verify to render the comparison sheet."""

import argparse
import json
import math
import sys
from itertools import pairwise
from pathlib import Path

import bpy
import bmesh
from mathutils import Vector
from mathutils.geometry import delaunay_2d_cdt

from shaders_knobs import apply_knob_shaders, knob_materials, linear_rgb


BODY, BEVEL, CAP, INK = range(4)


def mix(a, b, t):
    return a + (b - a) * t


def unit(t):
    return min(1.0, max(0.0, t))


def smoothstep(t):
    t = unit(t)
    return t * t * (3.0 - 2.0 * t)


class KnobMesh:
    def __init__(self):
        self.vertices = []
        self.faces = []
        self.slots = []
        self.creases = set()

    def loop(self, points):
        start = len(self.vertices)
        self.vertices.extend(points)
        return list(range(start, len(self.vertices)))

    def face(self, vertices, slot):
        self.faces.append(vertices)
        self.slots.append(slot)

    def bridge(self, lower, upper, slot=BODY):
        for i in range(len(lower)):
            j = (i + 1) % len(lower)
            self.face((lower[i], lower[j], upper[j], upper[i]), slot)

    def crease(self, ring):
        self.creases.update((min(a, b), max(a, b)) for a, b in zip(ring, ring[1:] + ring[:1]))

    def fan(self, ring, z, slot=BODY):
        center, = self.loop([(0, 0, z)])
        for i, a in enumerate(ring):
            self.face((center, a, ring[(i + 1) % len(ring)]), slot)

    def top_surface(self, outside, inside, mark, samples, height, marked_slot):
        """Triangulate a top annulus around the in-mesh print or inlay boundary."""
        vertices = outside + inside + mark + self.loop((x, y, height(x, y)) for x, y in samples)
        contours = (
            list(range(len(outside))),
            list(range(len(outside), len(outside) + len(inside))),
            list(range(len(outside) + len(inside), len(outside) + len(inside) + len(mark))),
        )
        _, _, faces, sources, _, regions = delaunay_2d_cdt(
            [Vector(self.vertices[i][:2]) for i in vertices], [], contours, 1, 1e-9
        )
        if any(len(source) != 1 for source in sources):
            raise ValueError("Knob contours intersect or contain duplicate points")
        indices = [vertices[source[0]] for source in sources]
        for face, region in zip(faces, regions, strict=True):
            if 0 not in region or 1 in region:
                continue
            slot = marked_slot if 2 in region else BODY
            if slot is not None:
                self.face(tuple(indices[i] for i in face), slot)

    def create(self, name, materials):
        mesh = bpy.data.meshes.new(name)
        mesh.from_pydata(self.vertices, [], self.faces)
        if mesh.validate():
            raise ValueError(f"{name}: invalid mesh")
        for material in materials:
            mesh.materials.append(material)
        for poly, slot in zip(mesh.polygons, self.slots, strict=True):
            poly.material_index = slot
            poly.use_smooth = True

        bm = bmesh.new()
        try:
            bm.from_mesh(mesh)
            bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
            for edge in bm.edges:
                a, b = (vert.index for vert in edge.verts)
                if (min(a, b), max(a, b)) in self.creases:
                    edge.smooth = False
            assert_solid(bm, name)
            bm.to_mesh(mesh)
        finally:
            bm.free()

        mesh.update()
        obj = bpy.data.objects.new(name, mesh)
        bpy.context.collection.objects.link(obj)
        return obj


def assert_solid(bm, name):
    if any(not edge.is_manifold or not edge.is_contiguous for edge in bm.edges):
        raise ValueError(f"{name}: open or inconsistently wound edges")
    if bm.calc_volume(signed=True) <= 0:
        raise ValueError(f"{name}: non-positive signed volume")

    # A single connected shell also rules out buried pegs and intersecting wedge parts.
    pending = [next(iter(bm.verts))]
    visited = set(pending)
    while pending:
        vertex = pending.pop()
        for edge in vertex.link_edges:
            neighbor = edge.other_vert(vertex)
            if neighbor not in visited:
                visited.add(neighbor)
                pending.append(neighbor)
    if len(visited) != len(bm.verts):
        raise ValueError(f"{name}: disconnected shells")


def angles(flutes, segments_per_flute):
    count = flutes * segments_per_flute
    return [math.tau * i / count for i in range(count)]


def ring(mesh, azimuths, radius, z, flutes=0, depth=0.0):
    return mesh.loop(
        (*polar(radius - depth * abs(math.cos(flutes * a / 2)) ** 2.2, a), z)
        for a in azimuths
    )


def polar(radius, azimuth):
    # Zero azimuth is +Y on the faceplate, or -Z in the Y-up GLB.
    return -radius * math.sin(azimuth), radius * math.cos(azimuth)


def cap_lip_radius(cap_radius):
    return cap_radius * 1.063


def add_cap(mesh, azimuths, lip, radius, height):
    # An exposed 45-degree chamfer sits inside a narrow molding relief. The
    # cap face has a 0.125 mm dish, measured from the chamfer to its center.
    edge_z = height - 0.00004
    groove_z = height - radius * 0.048
    chamfer = radius * 0.018
    arris = radius * 0.005
    face_radius = radius - chamfer - arris
    profile = [
        (radius * 1.049, height - radius * 0.004, BODY),
        (radius * 1.025, groove_z, BODY),
        (radius, groove_z, BODY),
        (radius, edge_z - chamfer - arris * 0.42, BEVEL),
        (radius - chamfer, edge_z - arris * 0.42, BEVEL),
        (face_radius, edge_z, BEVEL),
    ]
    for r, z, slot in profile:
        next_ring = ring(mesh, azimuths, r, z)
        mesh.bridge(lip, next_ring, slot)
        mesh.crease(next_ring)
        lip = next_ring
    dish = 0.000125
    for step in range(15, 0, -1):
        t = step / 16
        next_ring = ring(mesh, azimuths, face_radius * t, edge_z - dish * (1.0 - t * t))
        mesh.bridge(lip, next_ring, CAP)
        lip = next_ring
    mesh.fan(lip, edge_z - dish, CAP)


def build_skirted(name, dim, materials):
    base, height = dim["base_radius"], dim["height"]
    large = name == "knob_big"
    flutes = 14 if large else 10
    azimuths = angles(flutes, 36)
    grip = base * (0.835 if large else 0.772)
    depth = base * (0.073 if large else 0.078)
    cap = dim["cap_radius"]
    dot_radius, dot_y = dim["dot_radius"], dim["dot_radius_pos"]
    mesh = KnobMesh()

    # The indicator sits on a shallow conical skirt with a narrow molded lip.
    skirt_inside = dot_y - dot_radius - base * 0.011
    skirt_outside = base * 0.981
    skirt_low, skirt_high = height * 0.106, height * 0.147

    def skirt_z(x, y):
        t = (skirt_outside - math.hypot(x, y)) / (skirt_outside - skirt_inside)
        return mix(skirt_low, skirt_high, t)

    outer = [
        ring(mesh, azimuths, base * 0.969, 0.0),
        ring(mesh, azimuths, base * 0.993, height * 0.009),
        ring(mesh, azimuths, base, height * 0.027),
        ring(mesh, azimuths, base, height * 0.078),
        ring(mesh, azimuths, base * 0.995, height * 0.094),
        ring(mesh, azimuths, skirt_outside, skirt_low),
    ]
    mesh.fan(outer[0], 0.0)
    for a, b in pairwise(outer):
        mesh.bridge(a, b)
    mesh.crease(outer[-1])

    inner = ring(mesh, azimuths, skirt_inside, skirt_high)
    dot_outline = []
    for a in angles(1, 64):
        x, dy = polar(dot_radius, a)
        y = dot_y + dy
        dot_outline.append((x, y, skirt_z(x, y)))
    dot = mesh.loop(dot_outline)

    # Sample the cone around and through the print. Long triangles running
    # across the cone would facet its normal around the dot.
    samples = []
    for band in range(1, 7):
        radius = mix(skirt_inside, skirt_outside, band / 7)
        for a in azimuths[::2]:
            x, y = polar(radius, a)
            if abs(math.hypot(x, y - dot_y) - dot_radius) > base * 0.003:
                samples.append((x, y))
    mesh.top_surface(outer[-1], inner, dot, samples, skirt_z, INK)

    # The drum rises out of a recessed circular foot. Tight pairs of profile
    # rings define the flute shoulders without smearing the tapered sidewall.
    mesh.crease(inner)
    for r, z in (
        (skirt_inside - base * 0.009, height * 0.137),
        (skirt_inside - base * 0.018, height * 0.137),
        (skirt_inside - base * 0.024, height * 0.158),
    ):
        next_ring = ring(mesh, azimuths, r, z)
        mesh.bridge(inner, next_ring)
        inner = next_ring
    profile = [
        (grip * 0.980, 0.194, 0.90, True),
        (grip, 0.224, 1.00, False),
        (grip * 0.995, 0.420, 1.00, False),
        (grip * 0.976, 0.935, 0.77, False),
        (grip * 0.967, 0.970, 0.67, True),
        (grip * 0.947, 0.994, 0.53, True),
        (cap_lip_radius(cap), 1.0, 0.0, False),
    ]
    for r, z, d, sharp in profile:
        next_ring = ring(mesh, azimuths, r, height * z, flutes, depth * d)
        mesh.bridge(inner, next_ring)
        if sharp:
            mesh.crease(next_ring)
        inner = next_ring
    add_cap(mesh, azimuths, inner, cap, height)
    return mesh.create(name, materials)


def capsule_loop(y0, y1, width):
    """Footprint of a stripe with round printed ends; y0/y1 are arc centers."""
    for center, start in ((y1, 0.0), (y0, math.pi)):
        for i in range(17):
            a = start + math.pi * i / 16
            yield width * math.cos(a), center + width * math.sin(a)


def build_pointer(name, dim, materials):
    body, tip, height = dim["body_radius"], dim["tip_radius"], dim["height"]
    cap = dim["cap_radius"]
    lip = cap_lip_radius(cap)
    flutes = dim["flute_count"]
    azimuths = angles(flutes, 48)
    mesh = KnobMesh()
    depth = body * 0.077

    # A tapered fin with straight flanks tangent to a circular nose. Blend its
    # outline into the scalloped drum, at every height, as one molded shell.
    nose_radius = body * 0.063
    nose_center = tip - nose_radius
    flank = 0.30
    flank_offset = flank * nose_center + nose_radius * math.hypot(1, flank)
    tangent_x = nose_radius / math.hypot(1, flank)
    tangent_y = nose_center + flank * tangent_x
    tangent_angle = math.atan2(tangent_x, tangent_y)

    def outline(a):
        scalloped = body - depth * abs(math.sin(flutes * a / 2)) ** 2.2
        cosine = math.cos(a)
        if cosine <= 0:
            return scalloped
        sine = abs(math.sin(a))
        if math.atan2(sine, cosine) < tangent_angle:
            fin = nose_center * cosine + math.sqrt(nose_radius ** 2 - (nose_center * sine) ** 2)
        else:
            fin = flank_offset / (sine + flank * cosine)
        seam = body * 0.022
        blend = max(seam - abs(fin - scalloped), 0.0) / seam
        return max(fin, scalloped) + blend * blend * seam * 0.25

    def deck_z(x, y):
        r = math.hypot(x, y)
        shoulder = unit((r - lip) / (body - lip))
        round_body = height * (1.0 - 0.405 * shoulder * (0.50 + 0.50 * shoulder))
        if y <= 0.0:
            return round_body
        ridge = height * (1.0 - 0.344 * unit((y - lip) / (tip - lip)))
        # A narrow bevel joins the steep flanks to the fin's sloping deck.
        width = flank_offset - flank * min(y, nose_center)
        flank_blend = smoothstep((0.98 - abs(x) / width) / 0.25)
        return mix(round_body, ridge, flank_blend * smoothstep((y - lip * 0.9) / (body * 0.13)))

    perimeter = [outline(a) for a in azimuths]
    outer = []
    for inset, z_fraction in ((0.028, 0.0), (0.009, 0.011), (0.0, 0.030)):
        outer.append(mesh.loop(
            (*polar(r - inset * body, a), z_fraction * height)
            for a, r in zip(azimuths, perimeter, strict=True)
        ))
    deck_outline = [r - body * 0.027 for r in perimeter]
    deck_points = [polar(r, a) for a, r in zip(azimuths, deck_outline, strict=True)]
    outer.append(mesh.loop(
        (*polar(r, a), deck_z(*point) - body * 0.020)
        for a, r, point in zip(azimuths, perimeter, deck_points, strict=True)
    ))
    outer.append(mesh.loop((*point, deck_z(*point)) for point in deck_points))
    mesh.fan(outer[0], 0.0)
    for a, b in pairwise(outer):
        mesh.bridge(a, b)
    mesh.crease(outer[-2])
    mesh.crease(outer[-1])

    # Cut the stripe into the curved surface. The paint shares its groove floor
    # with the surrounding dark walls, so the GLB stays one watertight shell.
    scale = dim["scale"]
    half_width = 0.00031 * scale
    y0 = lip + 0.00175 * scale
    y1 = tip - 0.00087 * scale
    groove_width = half_width + 0.00011 * scale
    groove_depth = 0.00013 * scale
    groove = mesh.loop((x, y, deck_z(x, y)) for x, y in capsule_loop(y0, y1, groove_width))
    inlay = mesh.loop(
        (x, y, deck_z(x, y) - groove_depth)
        for x, y in capsule_loop(y0, y1, half_width)
    )
    cap_lip = ring(mesh, azimuths, lip, height)
    samples = []
    for row in range(1, 26):
        t = row / 26
        for a, r in zip(azimuths, deck_outline, strict=True):
            x, y = polar(mix(lip, r, t), a)
            dy = y - min(y1, max(y0, y))
            if math.hypot(x, dy) > groove_width + body * 0.006:
                samples.append((x, y))
    mesh.top_surface(outer[-1], cap_lip, groove, samples, deck_z, None)
    mesh.bridge(groove, inlay)
    mesh.crease(groove)
    mesh.crease(inlay)
    floor = mesh.loop(
        (0.0, y, deck_z(0.0, y) - groove_depth)
        for y in (y0, y1)
    )
    half = len(inlay) // 2
    for center, arc in ((floor[1], inlay[:half]), (floor[0], inlay[half:])):
        for a, b in pairwise(arc):
            mesh.face((center, a, b), INK)
    mesh.face((inlay[0], floor[1], floor[0], inlay[-1]), INK)
    mesh.face((inlay[half - 1], inlay[half], floor[0], floor[1]), INK)
    add_cap(mesh, azimuths, cap_lip, cap, height)
    return mesh.create(name, materials)


def export_glb(obj, path):
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    result = bpy.ops.export_scene.gltf(
        filepath=str(path), export_format="GLB", export_yup=True,
        export_normals=True, use_selection=True,
    )
    if result != {"FINISHED"}:
        raise RuntimeError(f"Failed to export {path}")


def add_faceplate():
    """Plain pebbled black faceplate for the shadow and material comparison."""
    bpy.ops.mesh.primitive_plane_add(size=0.5)
    plate = bpy.context.object
    plate.name = "Black enamel faceplate"
    material = bpy.data.materials.new(plate.name)
    plate.data.materials.append(material)
    material.use_nodes = True
    tree = material.node_tree
    bsdf = tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = linear_rgb((23, 22, 20))
    bsdf.inputs["Roughness"].default_value = 0.64
    bsdf.inputs["Specular IOR Level"].default_value = 0.22
    bsdf.inputs["IOR"].default_value = 1.44
    coords = tree.nodes.new("ShaderNodeTexCoord")
    pebble = tree.nodes.new("ShaderNodeTexNoise")
    # The plane primitive has unit object scale, so these are cycles per meter.
    pebble.inputs["Scale"].default_value = 4600
    pebble.inputs["Detail"].default_value = 2.0
    tree.links.new(coords.outputs["Object"], pebble.inputs["Vector"])
    relief = tree.nodes.new("ShaderNodeBump")
    relief.inputs["Strength"].default_value = 0.40
    relief.inputs["Distance"].default_value = 0.000097
    tree.links.new(pebble.outputs["Fac"], relief.inputs["Height"])
    tree.links.new(relief.outputs["Normal"], bsdf.inputs["Normal"])


def import_knob(path):
    before = set(bpy.data.objects)
    result = bpy.ops.import_scene.gltf(filepath=str(path))
    imported = set(bpy.data.objects) - before
    if result != {"FINISHED"} or len(imported) != 1:
        raise ValueError(f"{path.name}: expected one mesh node")
    obj = imported.pop()
    if obj.type != "MESH" or obj.name != path.stem:
        raise ValueError(f"{path.name}: unexpected mesh name or type")

    # glTF primitives split vertices at material boundaries. Check their
    # reassembled geometry on the actual exported and re-imported artifact.
    bm = bmesh.new()
    try:
        bm.from_mesh(obj.data)
        bmesh.ops.remove_doubles(bm, verts=list(bm.verts), dist=1e-8)
        assert_solid(bm, obj.name)
    finally:
        bm.free()
    apply_knob_shaders(obj.data.materials)
    return obj


def verification_sheet(out, dims, tile=928, samples=128):
    from tempfile import TemporaryDirectory

    from PIL import Image, ImageDraw, ImageFont

    from studio_rig import make_ortho_cam, reset_scene, setup_studio_lights

    worn = Path(__file__).resolve().parents[2] / "crates/ca72-panel/assets/worn"
    gap = round(tile * 24 / 928)
    margin = round(tile * 28 / 928)
    label_height = round(tile * 50 / 928)
    header_height = round(tile * 140 / 928)
    sheet = Image.new(
        "RGB", (tile * 4 + gap * 3 + margin * 2, header_height + 3 * (tile + label_height + gap)),
        "#171819",
    )
    text = ImageDraw.Draw(sheet)
    title_font = ImageFont.load_default(size=round(tile * 46 / 928))
    label_font = ImageFont.load_default(size=round(tile * 27 / 928))
    text.text((margin, margin), "CA-72  /  Rotary controls", fill="#f0eee8", font=title_font)
    text.text(
        (margin, margin * 2 + round(tile * 34 / 928)),
        "Worn artwork and Cycles renders on a pebbled faceplate",
        fill="#b9b6af", font=label_font,
    )

    def place(row, col, image, label):
        x = margin + col * (tile + gap)
        y = header_height + row * (tile + label_height + gap)
        sheet.paste(image, (x, y))
        text.text((x, y + tile + gap / 2), label, fill="#e8e4dc", font=label_font)

    scene, backend = reset_scene(samples=samples)
    scene.render.image_settings.compression = 35
    setup_studio_lights(scene)
    add_faceplate()
    print(f"Rendering knob comparison on {backend}", flush=True)

    with TemporaryDirectory(prefix="ca72-knobs-") as tmp:
        path = Path(tmp) / "frame.png"
        scene.render.filepath = str(path)

        def frame(radius, tilt=0.0):
            cam = make_ortho_cam(scene, 0.0, 0.0, tile, tile, tile / (2.64 * radius))
            if tilt:
                center = Vector((0.0, 0.0, 0.0055))
                angle = math.radians(tilt)
                cam.location = center + Vector((0.0, math.sin(angle), math.cos(angle))) * 0.40
                cam.rotation_euler = (center - cam.location).to_track_quat("-Z", "Y").to_euler()
            bpy.ops.render.render(write_still=True)
            bpy.data.objects.remove(cam, do_unlink=True)
            with Image.open(path) as rendered:
                return rendered.convert("RGB")

        for row, (name, reference, second_angle) in enumerate((
            ("knob_std", "knob.png", -60),
            ("knob_big", "knob-big.png", 45),
            ("knob_pointer", "pointer.png", -50),
        )):
            dim = dims[name]
            radius = dim.get("base_radius", dim.get("body_radius"))
            plate = frame(radius)
            with Image.open(worn / reference) as sprite:
                full = round(tile * 2.0 / 2.64)
                if name == "knob_pointer":
                    # The sprite is centered on the shaft and padded horizontally.
                    left, _, right, _ = sprite.getbbox()
                    full = round(full * sprite.width / (right - left))
                scaled = sprite.resize((full, full), Image.Resampling.LANCZOS)
                offset = (tile - full) // 2
                plate.paste(scaled, (offset, offset), scaled)
            place(row, 0, plate, f"Worn 2D  ·  {reference}")

            obj = import_knob(out / "assets" / f"{name}.glb")
            obj.rotation_mode = "XYZ"
            for col, angle in enumerate((0, second_angle), 1):
                obj.rotation_euler.z = math.radians(-angle)
                label = f"{name}  ·  {angle:+d}°" if angle else f"{name}  ·  0°"
                place(row, col, frame(radius), label)

            if name == "knob_pointer":
                bpy.data.objects.remove(obj, do_unlink=True)
                small = import_knob(out / "assets/knob_pointer_small.glb")
                place(row, 3, frame(radius), "knob_pointer_small  ·  0°")
                bpy.data.objects.remove(small, do_unlink=True)
            else:
                obj.rotation_euler.z = 0.0
                place(row, 3, frame(radius, tilt=43.0), f"{name}  ·  oblique")
                bpy.data.objects.remove(obj, do_unlink=True)

    sheet.save(out / "verify_knobs.png")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=Path("target/panel-3d"))
    parser.add_argument("--verify", action="store_true", help="Render the comparison sheet (requires Pillow)")
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
    args = parser.parse_args(argv)

    dims = json.loads((args.out / "spec.json").read_text())["part_dimensions_meters"]
    assets = args.out / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.unit_settings.system = "METRIC"
    materials = knob_materials()
    for build, names in (
        (build_skirted, ("knob_std", "knob_big")),
        (build_pointer, ("knob_pointer", "knob_pointer_small")),
    ):
        for name in names:
            obj = build(name, dims[name], materials)
            export_glb(obj, assets / f"{name}.glb")
            bpy.data.objects.remove(obj, do_unlink=True)
    if args.verify:
        verification_sheet(args.out, dims)


if __name__ == "__main__":
    main()
