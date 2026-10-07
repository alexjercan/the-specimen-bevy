import argparse
import hashlib
import json
import math
import pathlib
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

TILE = 2.5
WALL_H = 3.0
WALL_T = 0.2
POST = 0.36
SLAB_T = 0.1
DOOR_W = 1.2
DOOR_H = 2.2
TEXTURE_METERS = 3.0
KIT_UV_PERIOD = TILE
TEXTURE = "concrete_floor_worn_001"

CORRIDOR_CELLS = 5
ROOM_I = (-1, 0, 1)
ROOM_J = (5, 6, 7)

LIGHT_ANCHOR_Z = WALL_H - 0.3
EYE_HEIGHT = 1.6

ROUTE_GLOW = 0.5
SIGN_GLOW = 0.8
GLYPH_PX = 0.016
GLYPH_ROWS = 7


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True)
    parser.add_argument("--target", choices=("facility", "kit"), required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--manifest")
    args = parser.parse_args(argv)
    if args.target == "facility" and not args.manifest:
        parser.error("--target facility needs --manifest")
    return args


def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.unit_settings.system = "METRIC"
    scene.unit_settings.scale_length = 1.0
    scene.unit_settings.length_unit = "METERS"
    return scene


def image(root, kind, colorspace):
    path = root / "art/visuals/sources/polyhaven" / TEXTURE / f"{TEXTURE}_{kind}_1k.jpg"
    if not path.is_file():
        raise SystemExit(f"missing texture {path}; run scripts/fetch-facility-textures.sh")
    img = bpy.data.images.load(str(path), check_existing=True)
    img.colorspace_settings.name = colorspace
    return img


def material(name, color, metallic=0.0, roughness=0.6, emission=None, strength=0.0, textures=None, alpha=1.0):
    mat = bpy.data.materials.new(name)
    mat.use_backface_culling = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    bsdf = nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Alpha"].default_value = alpha
    if alpha < 1.0:
        mat.blend_method = "BLEND"
    if emission:
        bsdf.inputs["Emission Color"].default_value = (*emission, 1.0)
        bsdf.inputs["Emission Strength"].default_value = strength
    if textures:
        diff, nor, arm = textures
        uv = nodes.new("ShaderNodeUVMap")
        uv.uv_map = "UVMap"
        tex = nodes.new("ShaderNodeTexImage")
        tex.image = diff
        links.new(uv.outputs["UV"], tex.inputs["Vector"])
        mix = nodes.new("ShaderNodeMix")
        mix.data_type = "RGBA"
        mix.blend_type = "MULTIPLY"
        mix.inputs["Factor"].default_value = 1.0
        mix.inputs[7].default_value = (*color, 1.0)
        links.new(tex.outputs["Color"], mix.inputs[6])
        links.new(mix.outputs[2], bsdf.inputs["Base Color"])
        ntex = nodes.new("ShaderNodeTexImage")
        ntex.image = nor
        links.new(uv.outputs["UV"], ntex.inputs["Vector"])
        nmap = nodes.new("ShaderNodeNormalMap")
        links.new(ntex.outputs["Color"], nmap.inputs["Color"])
        links.new(nmap.outputs["Normal"], bsdf.inputs["Normal"])
        atex = nodes.new("ShaderNodeTexImage")
        atex.image = arm
        links.new(uv.outputs["UV"], atex.inputs["Vector"])
        sep = nodes.new("ShaderNodeSeparateColor")
        links.new(atex.outputs["Color"], sep.inputs["Color"])
        links.new(sep.outputs["Green"], bsdf.inputs["Roughness"])
        links.new(sep.outputs["Blue"], bsdf.inputs["Metallic"])
    return mat


def make_materials(root):
    textures = (
        image(root, "diff", "sRGB"),
        image(root, "nor_gl", "Non-Color"),
        image(root, "arm", "Non-Color"),
    )
    return {
        "concrete_floor": material("concrete_floor", (0.78, 0.76, 0.72), textures=textures),
        "concrete_wall": material("concrete_wall", (0.62, 0.66, 0.64), textures=textures),
        "paint_lower": material("paint_lower", (0.16, 0.24, 0.22), textures=textures),
        "steel_dark": material("steel_dark", (0.07, 0.075, 0.08), metallic=0.8, roughness=0.45),
        "steel_door": material("steel_door", (0.19, 0.27, 0.28), metallic=0.55, roughness=0.5),
        "steel_ceiling": material("steel_ceiling", (0.11, 0.115, 0.12), metallic=0.6, roughness=0.65),
        "paint_hazard": material("paint_hazard", (0.55, 0.42, 0.08), roughness=0.75),
        "conduit": material("conduit", (0.32, 0.33, 0.31), metallic=0.85, roughness=0.4),
        "glass_dark": material("glass_dark", (0.02, 0.025, 0.03), roughness=0.08),
        "lamp_dead": material("lamp_dead", (0.35, 0.36, 0.34), roughness=0.3),
        "lamp_cool": material("lamp_cool", (0.85, 0.92, 1.0), roughness=0.3, emission=(0.78, 0.88, 1.0), strength=6.0),
        "lamp_amber": material("lamp_amber", (1.0, 0.7, 0.35), roughness=0.3, emission=(1.0, 0.58, 0.2), strength=6.0),
        "lamp_red": material("lamp_red", (0.9, 0.1, 0.06), roughness=0.3, emission=(1.0, 0.06, 0.03), strength=8.0),
    }


def make_kit_materials(mats):
    return {
        **mats,
        "lamp_green": material("lamp_green", (0.2, 0.9, 0.4), roughness=0.3, emission=(0.12, 1.0, 0.35), strength=5.0),
        "paint_crate": material("paint_crate", (0.2, 0.24, 0.15), metallic=0.2, roughness=0.7),
        "paint_drum": material("paint_drum", (0.34, 0.11, 0.05), metallic=0.4, roughness=0.65),
        "route_orange": material("route_orange", (1.0, 0.3, 0.02), roughness=0.6, emission=(1.0, 0.3, 0.02), strength=ROUTE_GLOW),
        "route_blue": material("route_blue", (0.02, 0.25, 1.0), roughness=0.6, emission=(0.02, 0.25, 1.0), strength=ROUTE_GLOW),
        "route_green": material("route_green", (0.05, 0.8, 0.2), roughness=0.6, emission=(0.05, 0.8, 0.2), strength=ROUTE_GLOW),
        "sign_text": material("sign_text", (0.85, 0.85, 0.8), roughness=0.6, emission=(0.85, 0.85, 0.8), strength=SIGN_GLOW),
        "sign_plate": material("sign_plate", (0.03, 0.035, 0.04), roughness=0.5),
        "lamp_fire": material("lamp_fire", (1.0, 0.35, 0.08), roughness=0.4, emission=(1.0, 0.35, 0.08), strength=6.0),
        "void_black": material("void_black", (0.004, 0.004, 0.004), roughness=1.0),
        "plastic_bin": material("plastic_bin", (0.1, 0.3, 0.55), roughness=0.5),
        "glass_lab": material("glass_lab", (0.65, 0.88, 0.82), roughness=0.08, alpha=0.2),
        "specimen_fluid": material("specimen_fluid", (0.02, 0.14, 0.12), roughness=0.15, emission=(0.05, 0.3, 0.25), strength=0.3),
        "glass_edge": material("glass_edge", (0.75, 0.9, 0.9), roughness=0.15, emission=(0.5, 0.8, 0.8), strength=0.15),
        "paper": material("paper", (0.7, 0.68, 0.6), roughness=0.85),
        "spill_dark": material("spill_dark", (0.05, 0.03, 0.02), roughness=0.2),
        "fuse_ceramic": material("fuse_ceramic", (0.78, 0.74, 0.64), roughness=0.55),
    }


class Palette:
    def __init__(self, mats, uv_period, canonical):
        self.mats = mats
        self.uv_period = uv_period
        self.canonical = canonical


class Builder:
    def __init__(self, name, palette):
        self.name = name
        self.mats = palette.mats
        self.uv_period = palette.uv_period
        self.canonical = palette.canonical
        self.slots = []
        self.bm = bmesh.new()

    def slot(self, mat):
        if mat not in self.slots:
            self.slots.append(mat)
        return self.slots.index(mat)

    def box(self, mn, mx, mat, bevel=0.0):
        mn = Vector(mn)
        mx = Vector(mx)
        size = mx - mn
        geom = bmesh.ops.create_cube(self.bm, size=1.0)
        verts = geom["verts"]
        center = (mn + mx) / 2
        bmesh.ops.transform(self.bm, matrix=Matrix.Translation(center) @ Matrix.Diagonal((*size, 1.0)), verts=verts)
        self._finish(verts, mat, bevel)

    def cylinder(self, start, end, radius, mat, segments=12):
        start = Vector(start)
        end = Vector(end)
        axis = end - start
        geom = bmesh.ops.create_cone(
            self.bm, cap_ends=True, cap_tris=self.canonical, segments=segments,
            radius1=radius, radius2=radius, depth=axis.length,
        )
        rot = Vector((0, 0, 1)).rotation_difference(axis.normalized()).to_matrix().to_4x4()
        bmesh.ops.transform(self.bm, matrix=Matrix.Translation((start + end) / 2) @ rot, verts=geom["verts"])
        self._finish(geom["verts"], mat, 0.0, smooth=True)

    def _finish(self, verts, mat, bevel, smooth=False):
        vset = set(verts)
        faces = [f for f in self.bm.faces if all(v in vset for v in f.verts)]
        index = self.slot(mat)
        for face in faces:
            face.material_index = index
            face.smooth = smooth
        if bevel > 0.0:
            edges = [e for e in self.bm.edges if all(v in vset for v in e.verts)]
            result = bmesh.ops.bevel(
                self.bm, geom=edges, offset=bevel, segments=1, profile=0.5,
                affect="EDGES", clamp_overlap=True,
            )
            for face in result["faces"]:
                face.material_index = index

    def build(self):
        mesh = bpy.data.meshes.new(self.name)
        if self.canonical:
            bmesh.ops.triangulate(self.bm, faces=self.bm.faces[:], quad_method="FIXED", ngon_method="EAR_CLIP")
            canonical_mesh(self.bm, mesh)
        else:
            bmesh.ops.triangulate(self.bm, faces=self.bm.faces[:])
            self.bm.to_mesh(mesh)
        self.bm.free()
        for mat in self.slots:
            mesh.materials.append(self.mats[mat])
        box_uv(mesh, self.uv_period)
        return mesh


def rounded(co):
    return tuple(round(c, 5) for c in co)


def canonical_mesh(bm, mesh):
    centers = {f: rounded(f.calc_center_median()) for f in bm.faces}
    verts = sorted(bm.verts, key=lambda v: (rounded(v.co), sorted(centers[f] for f in v.link_faces)))
    rank = {v: i for i, v in enumerate(verts)}
    faces = []
    for f in bm.faces:
        ids = [rank[v] for v in f.verts]
        start = ids.index(min(ids))
        faces.append((f.material_index, centers[f], ids[start:] + ids[:start], f.smooth))
    faces.sort()
    mesh.from_pydata([tuple(v.co) for v in verts], [], [f[2] for f in faces])
    for poly, f in zip(mesh.polygons, faces):
        poly.material_index = f[0]
        poly.use_smooth = f[3]
    mesh.update()


def box_uv(mesh, period):
    uv = mesh.uv_layers.new(name="UVMap")
    for poly in mesh.polygons:
        n = poly.normal
        axis = max(range(3), key=lambda i: abs(n[i]))
        for li in poly.loop_indices:
            co = mesh.vertices[mesh.loops[li].vertex_index].co
            if axis == 0:
                u, v = co.y * math.copysign(1, n.x), co.z
            elif axis == 1:
                u, v = -co.x * math.copysign(1, n.y), co.z
            else:
                u, v = co.x, co.y * math.copysign(1, n.z)
            uv.data[li].uv = (u / period, v / period)


def rotated_box(b, mn, mx, mat, angle, pivot, bevel=0.0, axis="Z"):
    mn = Vector(mn)
    mx = Vector(mx)
    size = mx - mn
    center = (mn + mx) / 2
    geom = bmesh.ops.create_cube(b.bm, size=1.0)
    verts = geom["verts"]
    matrix = (
        Matrix.Translation(pivot)
        @ Matrix.Rotation(angle, 4, axis)
        @ Matrix.Translation(center)
        @ Matrix.Diagonal((*size, 1.0))
    )
    bmesh.ops.transform(b.bm, matrix=matrix, verts=verts)
    b._finish(verts, mat, bevel)


def valve_wheel(b, x, z, y_face, mat, r=0.1):
    b.cylinder((x, y_face - 0.03, z), (x, y_face, z), r, mat, segments=16)
    b.cylinder((x, y_face - 0.1, z), (x, y_face + 0.02, z), r * 0.25, mat, segments=8)
    s = r * 0.85
    b.box((x - s, y_face - 0.03, z - 0.012), (x + s, y_face, z + 0.012), mat)
    b.box((x - 0.012, y_face - 0.03, z - s), (x + 0.012, y_face, z + s), mat)


GLYPHS = {
    "B": ("11110", "10001", "10001", "11110", "10001", "10001", "11110"),
    "O": ("01110", "10001", "10001", "10001", "10001", "10001", "01110"),
    "I": ("11111", "00100", "00100", "00100", "00100", "00100", "11111"),
    "L": ("10000", "10000", "10000", "10000", "10000", "10000", "11111"),
    "E": ("11111", "10000", "10000", "11110", "10000", "10000", "11111"),
    "R": ("11110", "10001", "10001", "11110", "10100", "10010", "10001"),
    "M": ("10001", "11011", "10101", "10101", "10001", "10001", "10001"),
    "S": ("01111", "10000", "10000", "01110", "00001", "00001", "11110"),
    "T": ("11111", "00100", "00100", "00100", "00100", "00100", "00100"),
    "A": ("01110", "10001", "10001", "11111", "10001", "10001", "10001"),
    "G": ("01111", "10000", "10000", "10111", "10001", "10001", "01111"),
    "N": ("10001", "11001", "10101", "10101", "10011", "10001", "10001"),
    "C": ("01111", "10000", "10000", "10000", "10000", "10000", "01111"),
    "F": ("11111", "10000", "10000", "11110", "10000", "10000", "10000"),
    "P": ("11110", "10001", "10001", "11110", "10000", "10000", "10000"),
    "U": ("10001", "10001", "10001", "10001", "10001", "10001", "01110"),
    "Y": ("10001", "10001", "01010", "00100", "00100", "00100", "00100"),
    "X": ("10001", "10001", "01010", "00100", "01010", "10001", "10001"),
    " ": ("000", "000", "000", "000", "000", "000", "000"),
}


def row_runs(row):
    runs = []
    start = None
    for c, bit in enumerate(row + "0"):
        if bit == "1" and start is None:
            start = c
        elif bit == "0" and start is not None:
            runs.append((start, c - 1))
            start = None
    return runs


def text_width(s, px=GLYPH_PX):
    return sum((len(GLYPHS[ch][0]) + 1) * px for ch in s) - px


def glyph_text(b, s, right_x, mid_z, y0, y1, mat, px=GLYPH_PX):
    cursor = right_x
    cap = GLYPH_ROWS * px
    top_z = mid_z + cap / 2
    for ch in s:
        rows = GLYPHS[ch]
        width = len(rows[0]) * px
        for r, row in enumerate(rows):
            for c0, c1 in row_runs(row):
                x0 = cursor - (c1 + 1) * px
                x1 = cursor - c0 * px
                z0 = top_z - (r + 1) * px
                z1 = top_z - r * px
                b.box((x0, y0, z0), (x1, y1, z1), mat)
        cursor -= width + px


def pixel_grid(b, grid, left_x, mid_z, y0, y1, mat, px=GLYPH_PX):
    rows = len(grid)
    top_z = mid_z + rows * px / 2
    for r, row in enumerate(grid):
        for c0, c1 in row_runs(row):
            x0 = left_x + c0 * px
            x1 = left_x + (c1 + 1) * px
            z0 = top_z - (r + 1) * px
            z1 = top_z - r * px
            b.box((x0, y0, z0), (x1, y1, z1), mat)


def arrow_grid():
    rows, shaft_cols, head_cols, center = 7, 7, 4, 3
    grid = []
    for r in range(rows):
        d = abs(r - center)
        row = ["1" if d <= 1 else "0" for _ in range(shaft_cols)]
        for k in range(head_cols):
            max_d = 3 - k
            row.append("1" if d <= max_d else "0")
        grid.append("".join(row))
    return grid


def floor_tile(palette):
    b = Builder("floor_tile", palette)
    h = TILE / 2
    b.box((-h, -h, -SLAB_T), (h, h, 0.0), "concrete_floor", bevel=0.012)
    return b.build()


def floor_tile_marked(palette):
    b = Builder("floor_tile_marked", palette)
    h = TILE / 2
    b.box((-h, -h, -SLAB_T), (h, h, 0.0), "concrete_floor", bevel=0.012)
    for x in (-0.95, 0.95):
        b.box((x - 0.04, -h + 0.02, 0.0), (x + 0.04, h - 0.02, 0.003), "paint_hazard")
    return b.build()


def ceiling_tile(palette):
    b = Builder("ceiling_tile", palette)
    h = TILE / 2
    b.box((-h, -h, WALL_H), (h, h, WALL_H + SLAB_T), "steel_ceiling")
    rib = 0.07
    for s in (-1, 1):
        b.box((-h, s * h - (rib if s > 0 else 0), WALL_H - 0.12), (h, s * h + (rib if s < 0 else 0), WALL_H), "steel_dark")
        b.box((s * h - (rib if s > 0 else 0), -h + rib, WALL_H - 0.12), (s * h + (rib if s < 0 else 0), h - rib, WALL_H), "steel_dark")
    b.box((-h + rib, -0.03, WALL_H - 0.05), (h - rib, 0.03, WALL_H), "steel_dark")
    return b.build()


WALL_BANDS = (
    (0.0, 0.16, WALL_T + 0.05, "steel_dark", 0.01),
    (0.16, 1.1, WALL_T, "paint_lower", 0.0),
    (1.1, 1.16, WALL_T + 0.04, "steel_dark", 0.008),
    (1.16, 2.78, WALL_T, "concrete_wall", 0.0),
    (2.78, WALL_H, WALL_T + 0.03, "steel_dark", 0.008),
)


def wall_bands(b, opening):
    h = TILE / 2
    for z0, z1, t, mat, bevel in WALL_BANDS:
        spans = [(-h, h, z0, z1)]
        if opening:
            ow = DOOR_W / 2 + 0.1
            spans = [(-h, -ow, z0, z1), (ow, h, z0, z1)]
            if z1 > DOOR_H + 0.1:
                spans.append((-ow, ow, max(z0, DOOR_H + 0.1), z1))
        for x0, x1, s0, s1 in spans:
            b.box((x0, -t / 2, s0), (x1, t / 2, s1), mat, bevel=bevel)


def wall(palette, name, conduit=False):
    b = Builder(name, palette)
    wall_bands(b, opening=False)
    if conduit:
        h = TILE / 2
        y = WALL_T / 2 + 0.09
        for z, r in ((2.55, 0.045), (2.43, 0.03)):
            b.cylinder((-h, y, z), (h, y, z), r, "conduit")
        for x in (-0.6, 0.6):
            b.box((x - 0.03, WALL_T / 2, 2.38), (x + 0.03, y + 0.06, 2.62), "steel_dark")
    return b.build()


def wall_doorway(palette):
    b = Builder("wall_doorway", palette)
    wall_bands(b, opening=True)
    ow = DOOR_W / 2
    f = 0.1
    d = WALL_T / 2 + 0.035
    b.box((-ow - f, -d, 0.0), (-ow, d, DOOR_H + f), "steel_door", bevel=0.01)
    b.box((ow, -d, 0.0), (ow + f, d, DOOR_H + f), "steel_door", bevel=0.01)
    b.box((-ow - f, -d, DOOR_H), (ow + f, d, DOOR_H + f), "steel_door", bevel=0.01)
    for s in (-1, 1):
        b.box((-ow - f - 0.03, s * d - 0.004, DOOR_H + f + 0.06), (ow + f + 0.03, s * d + 0.004, DOOR_H + f + 0.14), "paint_hazard")
    return b.build()


def door_panel(palette):
    b = Builder("door_panel", palette)
    w = DOOR_W - 0.02
    t = 0.06
    b.box((0.0, -t / 2, 0.01), (w, t / 2, DOOR_H - 0.01), "steel_door", bevel=0.008)
    b.box((0.35, -t / 2 - 0.006, 1.45), (0.85, t / 2 + 0.006, 1.85), "glass_dark")
    b.box((0.05, -t / 2 - 0.004, 0.05), (w - 0.05, t / 2 + 0.004, 0.3), "steel_dark")
    for s in (-1, 1):
        b.box((w - 0.16, s * (t / 2 + 0.02) - 0.015, 1.0), (w - 0.12, s * (t / 2 + 0.02) + 0.015, 1.12), "conduit")
    return b.build()


def post(palette):
    b = Builder("wall_post", palette)
    p = POST / 2
    b.box((-p, -p, 0.0), (p, p, WALL_H), "steel_dark", bevel=0.015)
    b.box((-p - 0.01, -p - 0.01, 1.4), (p + 0.01, p + 0.01, 1.5), "paint_hazard")
    return b.build()


def ceiling_light(palette, name, lamp):
    b = Builder(name, palette)
    z = WALL_H - 0.12
    b.box((-0.7, -0.16, z - 0.09), (0.7, 0.16, z), "steel_dark", bevel=0.01)
    b.box((-0.64, -0.1, z - 0.1), (0.64, 0.1, z - 0.09), lamp)
    for x in (-0.55, 0.55):
        b.cylinder((x, 0.0, z), (x, 0.0, WALL_H), 0.012, "steel_dark", segments=6)
    return b.build()


def wall_lamp(palette, name, lamp):
    b = Builder(name, palette)
    b.box((-0.12, 0.0, -0.16), (0.12, 0.05, 0.16), "steel_dark", bevel=0.008)
    b.cylinder((0.0, 0.05, 0.0), (0.0, 0.14, 0.0), 0.075, lamp, segments=16)
    for a in range(4):
        ang = a * math.pi / 2 + math.pi / 4
        x = math.cos(ang) * 0.09
        z = math.sin(ang) * 0.09
        b.cylinder((x, 0.05, z), (x, 0.16, z), 0.008, "steel_dark", segments=6)
    return b.build()


def exit_sign(palette):
    b = Builder("exit_sign", palette)
    b.box((-0.25, 0.0, -0.1), (0.25, 0.06, 0.1), "steel_dark", bevel=0.008)
    b.box((-0.22, 0.06, -0.075), (0.22, 0.066, 0.075), "lamp_green")
    b.box((-0.16, 0.066, -0.05), (-0.06, 0.07, 0.05), "glass_dark")
    b.box((-0.03, 0.066, -0.012), (0.12, 0.07, 0.012), "glass_dark")
    b.cylinder((0.14, 0.066, 0.0), (0.14, 0.07, 0.0), 0.04, "glass_dark", segments=3)
    return b.build()


def wall_vent(palette):
    b = Builder("wall_vent", palette)
    b.box((-0.27, 0.0, -0.17), (0.27, 0.012, 0.17), "glass_dark")
    for s in (-1, 1):
        b.box((-0.3, 0.0, s * 0.17 - (0.03 if s < 0 else 0.0)), (0.3, 0.04, s * 0.17 + (0.03 if s > 0 else 0.0)), "steel_dark", bevel=0.004)
        b.box((s * 0.27 - (0.03 if s < 0 else 0.0), 0.0, -0.17), (s * 0.27 + (0.03 if s > 0 else 0.0), 0.04, 0.17), "steel_dark", bevel=0.004)
    for k in range(-2, 3):
        z = k * 0.065
        b.box((-0.26, 0.014, z - 0.012), (0.26, 0.034, z + 0.012), "conduit")
    return b.build()


FUSE_PANEL_HALF_W = 0.55
FUSE_PANEL_HALF_H = 0.65
FUSE_PANEL_DOOR_HALF_H = FUSE_PANEL_HALF_H - 0.16
FUSE_PANEL_SWITCH_ROWS = (0.24, 0.12, 0.0, -0.12, -0.24)
FUSE_PANEL_SWITCH_COLS = (-0.16, 0.0, 0.16)
FUSE_PANEL_SWITCH_W = 0.07
FUSE_PANEL_SWITCH_TRACK_H = 0.1
FUSE_PANEL_SWITCH_NUB_H = 0.035
FUSE_PANEL_TRIPPED = {(1, 2), (3, 0)}


def fuse_panel(palette):
    b = Builder("fuse_panel", palette)
    hw, hh = FUSE_PANEL_HALF_W, FUSE_PANEL_HALF_H
    dw, dh = hw - 0.05, FUSE_PANEL_DOOR_HALF_H
    b.box((-hw, 0.0, -hh), (hw, 0.05, hh), "steel_dark", bevel=0.01)
    b.box((-hw + 0.03, 0.05, hh - 0.1), (hw - 0.03, 0.054, hh - 0.06), "paint_hazard")
    b.box((-dw, 0.05, -dh), (dw, 0.072, dh), "steel_door", bevel=0.006)
    b.box((dw - 0.06, 0.072, -0.02), (dw - 0.025, 0.09, 0.02), "steel_dark")
    glyph_text(b, "MAIN", dw - 0.07, dh - 0.1, 0.072, 0.0755, "sign_text")
    for r, z in enumerate(FUSE_PANEL_SWITCH_ROWS):
        for c, x in enumerate(FUSE_PANEL_SWITCH_COLS):
            hw2, hth = FUSE_PANEL_SWITCH_W / 2, FUSE_PANEL_SWITCH_TRACK_H / 2
            b.box((x - hw2, 0.072, z - hth), (x + hw2, 0.076, z + hth), "steel_dark")
            tripped = (r, c) in FUSE_PANEL_TRIPPED
            nub_mat = "paint_hazard" if tripped else "steel_door"
            nub_z = z - hth + FUSE_PANEL_SWITCH_NUB_H / 2 if tripped else z + hth - FUSE_PANEL_SWITCH_NUB_H / 2
            nhh = FUSE_PANEL_SWITCH_NUB_H / 2
            b.box((x - hw2 + 0.006, 0.076, nub_z - nhh), (x + hw2 - 0.006, 0.086, nub_z + nhh), nub_mat, bevel=0.004)
    return b.build()


FUSE_PICKUP_R = 0.03
FUSE_PICKUP_BODY_X = 0.055
FUSE_PICKUP_CAP_X = 0.075
FUSE_PICKUP_BLADE_X = 0.1


def fuse_pickup(palette):
    b = Builder("fuse_pickup", palette)
    r, z = FUSE_PICKUP_R, FUSE_PICKUP_R
    bx, cx, tx = FUSE_PICKUP_BODY_X, FUSE_PICKUP_CAP_X, FUSE_PICKUP_BLADE_X
    b.cylinder((-bx, 0.0, z), (bx, 0.0, z), r - 0.004, "fuse_ceramic", segments=16)
    b.cylinder((-0.022, 0.0, z), (0.022, 0.0, z), r - 0.0035, "paint_hazard", segments=16)
    for s in (-1.0, 1.0):
        b.cylinder((s * bx, 0.0, z), (s * cx, 0.0, z), r, "conduit", segments=16)
        b.cylinder((s * (bx - 0.004), 0.0, z), (s * bx, 0.0, z), r - 0.002, "steel_dark", segments=16)
        lo, hi = sorted((s * cx, s * tx))
        b.box((lo, -0.003, z - 0.016), (hi, 0.003, z + 0.016), "conduit", bevel=0.001)
    return b.build()


def storage_crate(palette):
    b = Builder("storage_crate", palette)
    b.box((-0.5, -0.4, 0.06), (0.5, 0.4, 0.7), "paint_crate", bevel=0.02)
    for x in (-0.38, 0.38):
        b.box((x - 0.05, -0.4, 0.0), (x + 0.05, 0.4, 0.06), "steel_dark")
    for z in (0.18, 0.56):
        b.box((-0.51, -0.41, z), (0.51, 0.41, z + 0.05), "steel_dark")
    b.box((-0.2, 0.41, 0.3), (0.2, 0.415, 0.46), "paint_hazard")
    return b.build()


def steel_drum(palette):
    b = Builder("steel_drum", palette)
    b.cylinder((0.0, 0.0, 0.0), (0.0, 0.0, 0.88), 0.29, "paint_drum", segments=20)
    for z in (0.28, 0.58):
        b.cylinder((0.0, 0.0, z), (0.0, 0.0, z + 0.03), 0.3, "steel_dark", segments=20)
    b.cylinder((0.0, 0.0, 0.86), (0.0, 0.0, 0.89), 0.295, "steel_dark", segments=20)
    b.cylinder((0.15, 0.0, 0.89), (0.15, 0.0, 0.91), 0.03, "conduit", segments=8)
    return b.build()


def shelf_unit(palette):
    b = Builder("shelf_unit", palette)
    for x in (-0.875, 0.875):
        for y in (-0.225, 0.225):
            b.box((x - 0.025, y - 0.025, 0.0), (x + 0.025, y + 0.025, 2.0), "steel_dark", bevel=0.004)
    for z in (0.12, 0.6, 1.1, 1.6, 1.98):
        b.box((-0.9, -0.25, z - 0.02), (0.9, 0.25, z + 0.02), "steel_door", bevel=0.004)
    for x in (-0.875, 0.875):
        b.cylinder((x, -0.25, 0.14), (-x, -0.25, 1.58), 0.008, "steel_dark", segments=6)
    b.box((-0.7, -0.15, 0.14), (-0.3, 0.15, 0.44), "paint_crate", bevel=0.01)
    b.box((0.1, -0.18, 0.62), (0.6, 0.18, 0.92), "paint_crate", bevel=0.01)
    for x in (-0.55, -0.3):
        b.cylinder((x, 0.0, 1.12), (x, 0.0, 1.37), 0.1, "paint_drum", segments=12)
    b.box((0.3, -0.2, 1.62), (0.75, 0.2, 1.84), "steel_ceiling", bevel=0.01)
    return b.build()


def workbench(palette):
    b = Builder("workbench", palette)
    b.box((-0.8, -0.35, 0.86), (0.8, 0.35, 0.9), "steel_door", bevel=0.005)
    for x in (-0.74, 0.74):
        for y in (-0.29, 0.29):
            b.box((x - 0.03, y - 0.03, 0.0), (x + 0.03, y + 0.03, 0.86), "steel_dark")
    b.box((-0.77, -0.32, 0.2), (0.77, 0.32, 0.23), "steel_dark")
    b.box((-0.8, -0.35, 0.9), (0.8, -0.32, 1.5), "steel_ceiling")
    b.box((0.52, 0.18, 0.9), (0.7, 0.34, 1.0), "conduit", bevel=0.006)
    b.box((-0.5, 0.0, 0.9), (-0.1, 0.22, 1.05), "paint_drum", bevel=0.008)
    b.box((-0.45, 0.06, 1.05), (-0.15, 0.16, 1.08), "steel_dark")
    return b.build()


ROUTE_LINE_HALF_W = 0.05
ROUTE_LINE_H = 0.003


def route_line(palette, name, mat):
    b = Builder(name, palette)
    b.box((-0.5, -ROUTE_LINE_HALF_W, 0.0), (0.5, ROUTE_LINE_HALF_W, ROUTE_LINE_H), mat)
    return b.build()


SIGN_W = 0.65
SIGN_H = 0.1
SIGN_CHIP_W = 0.08
SIGN_MARGIN = 0.04
SIGN_CAP = GLYPH_ROWS * GLYPH_PX


def sign_label(palette, name, text, plate_mat, chip_mat):
    b = Builder(name, palette)
    if chip_mat:
        chip_x0 = SIGN_W - SIGN_CHIP_W
        b.box((-SIGN_W, 0.0, -SIGN_H), (chip_x0, 0.03, SIGN_H), plate_mat)
        b.box((chip_x0, 0.0, -SIGN_H), (SIGN_W, 0.03, SIGN_H), chip_mat)
        text_left, text_right = -SIGN_W + SIGN_MARGIN, chip_x0 - SIGN_MARGIN
    else:
        b.box((-SIGN_W, 0.0, -SIGN_H), (SIGN_W, 0.03, SIGN_H), plate_mat)
        text_left, text_right = -SIGN_W + SIGN_MARGIN, SIGN_W - SIGN_MARGIN
    center_x = (text_left + text_right) / 2
    right_x = center_x + text_width(text) / 2
    glyph_text(b, text, right_x, 0.0, 0.03, 0.034, "sign_text")
    return b.build()


def sign_arrow(palette):
    b = Builder("sign_arrow", palette)
    b.box((-0.1, 0.0, -0.1), (0.1, 0.03, 0.1), "sign_plate")
    grid = arrow_grid()
    width = len(grid[0]) * GLYPH_PX
    pixel_grid(b, grid, -width / 2, 0.0, 0.03, 0.034, "sign_text")
    return b.build()


def sign_hanger(palette):
    b = Builder("sign_hanger", palette)
    b.box((-0.85, -0.015, 2.2), (0.85, 0.015, 2.46), "sign_plate", bevel=0.006)
    for x in (-0.7, 0.7):
        b.cylinder((x, 0.0, 2.46), (x, 0.0, 3.0), 0.012, "steel_dark", segments=8)
    return b.build()


def boiler_unit(palette):
    b = Builder("boiler_unit", palette)
    b.box((-0.55, -0.55, 0.0), (0.55, 0.55, 0.12), "steel_dark", bevel=0.01)
    b.cylinder((0.0, 0.0, 0.12), (0.0, 0.0, 2.3), 0.62, "steel_door", segments=20)
    for z in (0.3, 0.9, 1.5, 2.1):
        b.cylinder((0.0, 0.0, z), (0.0, 0.0, z + 0.04), 0.635, "steel_dark", segments=20)
    b.box((-0.3, 0.35, 0.3), (0.3, 0.64, 1.0), "steel_door", bevel=0.01)
    b.box((-0.12, 0.5, 0.45), (0.12, 0.66, 0.6), "lamp_fire")
    for x, z in ((-0.2, 1.6), (0.2, 1.8)):
        b.cylinder((x, 0.55, z), (x, 0.68, z), 0.07, "glass_dark", segments=12)
        b.cylinder((x, 0.52, z), (x, 0.56, z), 0.075, "steel_dark", segments=12)
    valve_wheel(b, 0.35, 1.2, 0.68, "steel_dark", r=0.12)
    b.cylinder((0.0, 0.0, 2.3), (0.0, 0.0, WALL_H), 0.15, "steel_dark", segments=16)
    return b.build()


def pipe_manifold(palette):
    b = Builder("pipe_manifold", palette)
    for z in (-0.15, 0.0, 0.15):
        b.cylinder((-0.9, 0.1, z), (0.9, 0.1, z), 0.035, "conduit", segments=12)
        for x in (-0.6, 0.0, 0.6):
            b.cylinder((x - 0.015, 0.1, z), (x + 0.015, 0.1, z), 0.05, "steel_dark", segments=12)
    valve_wheel(b, -0.3, 0.15, 0.22, "steel_dark", r=0.07)
    valve_wheel(b, 0.3, -0.15, 0.22, "steel_dark", r=0.07)
    return b.build()


def tool_pegboard(palette):
    b = Builder("tool_pegboard", palette)
    b.box((-0.6, 0.0, -0.4), (0.6, 0.03, 0.4), "steel_dark", bevel=0.006)
    b.box((-0.42, 0.03, 0.1), (-0.12, 0.06, 0.14), "steel_door")
    b.box((-0.5, 0.03, 0.06), (-0.4, 0.06, 0.2), "steel_door")
    b.box((-0.02, 0.03, 0.05), (0.02, 0.06, 0.22), "steel_door")
    b.box((-0.08, 0.03, 0.2), (0.1, 0.06, 0.28), "steel_door")
    b.box((0.2, 0.03, -0.3), (0.5, 0.06, -0.24), "conduit")
    b.box((0.44, 0.03, -0.32), (0.5, 0.06, -0.08), "conduit")
    b.box((-0.5, 0.03, -0.3), (-0.46, 0.06, 0.0), "steel_door")
    b.box((-0.56, 0.03, -0.36), (-0.4, 0.06, -0.3), "steel_door")
    for x0, x1, z0, z1 in (
        (0.25, 0.55, 0.26, 0.28),
        (0.25, 0.55, 0.03, 0.05),
        (0.25, 0.27, 0.03, 0.28),
        (0.53, 0.55, 0.03, 0.28),
    ):
        b.box((x0, 0.03, z0), (x1, 0.045, z1), "paint_hazard")
    return b.build()


def shelf_unit_low(palette):
    b = Builder("shelf_unit_low", palette)
    for x in (-0.575, 0.575):
        for y in (-0.2, 0.2):
            b.box((x - 0.02, y - 0.02, 0.0), (x + 0.02, y + 0.02, 1.0), "steel_dark", bevel=0.004)
    for z in (0.02, 0.48, 0.98):
        b.box((-0.6, -0.225, z), (0.6, 0.225, z + 0.02), "steel_door", bevel=0.004)
    b.box((-0.45, -0.17, 0.05), (-0.05, 0.17, 0.4), "paint_crate", bevel=0.01)
    b.box((0.05, -0.19, 0.51), (0.5, 0.19, 0.78), "paint_crate", bevel=0.01)
    return b.build()


def bin_row(b, x0, x1, z0, z1, depth):
    b.box((x0, -depth / 2, z0), (x1, depth / 2, z0 + 0.02), "plastic_bin")
    b.box((x0, depth / 2 - 0.015, z0), (x1, depth / 2, z1), "plastic_bin")
    b.box((x0, -depth / 2, z0), (x0 + 0.015, depth / 2, z1), "plastic_bin")
    b.box((x1 - 0.015, -depth / 2, z0), (x1, depth / 2, z1), "plastic_bin")


def shelf_unit_bins(palette):
    b = Builder("shelf_unit_bins", palette)
    for x in (-0.875, 0.875):
        for y in (-0.225, 0.225):
            b.box((x - 0.025, y - 0.025, 0.0), (x + 0.025, y + 0.025, 2.0), "steel_dark", bevel=0.004)
    for z in (0.12, 0.6, 1.1, 1.6, 1.98):
        b.box((-0.9, -0.25, z - 0.02), (0.9, 0.25, z + 0.02), "steel_door", bevel=0.004)
    for z0 in (0.14, 0.62, 1.12):
        for x0 in (-0.82, -0.33, 0.16):
            bin_row(b, x0, x0 + 0.43, z0, z0 + 0.3, 0.44)
    return b.build()


def concept_table(palette):
    b = Builder("concept_table", palette)
    b.box((-0.9, -0.45, 0.75), (0.9, 0.45, 0.8), "steel_door", bevel=0.01)
    for x in (-0.84, 0.84):
        for y in (-0.42, 0.42):
            b.box((x - 0.025, y - 0.025, 0.0), (x + 0.025, y + 0.025, 0.75), "steel_dark", bevel=0.005)
    b.box((-0.7, -0.46, 0.3), (0.7, -0.44, 0.75), "paint_crate")
    return b.build()


def concept_locker(palette):
    b = Builder("concept_locker", palette)
    b.box((-0.3, -0.3, 0.0), (0.3, -0.27, 1.95), "steel_dark", bevel=0.004)
    b.box((-0.3, -0.27, 0.0), (0.3, 0.2, 0.03), "steel_dark", bevel=0.004)
    b.box((-0.3, -0.27, 1.92), (0.3, 0.2, 1.95), "steel_dark", bevel=0.004)
    b.box((-0.3, -0.27, 0.0), (-0.27, 0.2, 1.95), "steel_dark", bevel=0.004)
    b.box((0.27, -0.27, 0.0), (0.3, 0.2, 1.95), "steel_dark", bevel=0.004)
    b.box((-0.27, -0.27, 0.03), (0.27, 0.17, 1.92), "void_black")
    angle = -math.radians(25)
    pivot = (0.3, 0.2, 0.0)
    rotated_box(b, (-0.6, 0.0, 0.05), (0.0, 0.04, 1.15), "steel_door", angle, pivot, bevel=0.006)
    rotated_box(b, (-0.6, 0.0, 1.78), (0.0, 0.04, 1.9), "steel_door", angle, pivot, bevel=0.006)
    for x0 in (-0.6, -0.09):
        rotated_box(b, (x0, 0.0, 1.15), (x0 + 0.09, 0.04, 1.78), "steel_door", angle, pivot, bevel=0.004)
    for i in range(13):
        z0 = 1.185 + i * 0.045
        rotated_box(b, (-0.51, 0.012, z0), (-0.09, 0.028, z0 + 0.028), "conduit", angle, pivot)
    return b.build()


def concept_crawl_vent(palette):
    b = Builder("concept_crawl_vent", palette)
    b.box((-0.45, 0.0, 0.25), (0.45, 0.12, 0.35), "steel_dark", bevel=0.004)
    b.box((-0.45, 0.0, -0.35), (0.45, 0.12, -0.25), "steel_dark", bevel=0.004)
    b.box((-0.45, 0.0, -0.25), (-0.35, 0.12, 0.25), "steel_dark", bevel=0.004)
    b.box((0.35, 0.0, -0.25), (0.45, 0.12, 0.25), "steel_dark", bevel=0.004)
    for x, z in ((-0.42, -0.32), (0.42, -0.32), (-0.42, 0.32), (0.42, 0.32)):
        b.box((x - 0.015, 0.1, z - 0.015), (x + 0.015, 0.14, z + 0.015), "steel_dark")
    b.box((-0.35, 0.0, -0.25), (0.35, 0.005, 0.25), "void_black")
    return b.build()


def vent_grille(palette):
    b = Builder("vent_grille", palette)
    b.box((-0.425, -0.325, 0.0), (0.425, 0.325, 0.015), "steel_dark", bevel=0.004)
    for x0 in (-0.38, -0.25, -0.12, 0.01, 0.14, 0.27):
        b.box((x0, -0.3, 0.015), (x0 + 0.1, 0.3, 0.03), "steel_dark")
    return b.build()


TANK_SHELL_R = 0.6
TANK_CAGE_R = 0.68
TANK_SHELL_PANELS = 14
TANK_HOLE_INDICES = (3, 4)
TANK_BENT_BARS = (2, 3)
TANK_RIM = 0.012
TANK_HOLE_STEPS = (
    ((0.78, 0.64, 0.7), (1.52, 1.64, 1.45)),
    ((0.6, 0.86, 0.66), (1.6, 1.4, 1.68)),
)
TANK_PUDDLE = (
    (0.0, 0.5, 0.3, 0.2, 10, 0.001),
    (0.16, 0.64, 0.2, 0.14, 35, 0.0016),
    (-0.2, 0.62, 0.15, 0.12, -20, 0.0022),
    (0.04, 0.78, 0.12, 0.07, 60, 0.0028),
)


def concept_containment_tank(palette):
    b = Builder("concept_containment_tank", palette)
    b.box((-0.65, -0.65, 0.0), (0.65, 0.65, 0.25), "steel_dark", bevel=0.015)

    n = TANK_SHELL_PANELS
    chord = 2 * TANK_SHELL_R * math.sin(math.pi / n)
    for i in range(n):
        ang = i * 2 * math.pi / n
        pivot = (TANK_SHELL_R * math.cos(ang), TANK_SHELL_R * math.sin(ang), 0.0)
        if i in TANK_HOLE_INDICES:
            low, high = TANK_HOLE_STEPS[TANK_HOLE_INDICES.index(i)]
            rotated_box(b, (-0.01, -chord / 2, 0.25), (0.01, chord / 2, 0.55), "glass_lab", ang, pivot)
            rotated_box(b, (-0.01, -chord / 2, 1.75), (0.01, chord / 2, 2.3), "glass_lab", ang, pivot)
            for k in range(3):
                y0 = -chord / 2 + k * chord / 3
                y1 = y0 + chord / 3
                rotated_box(b, (-0.01, y0, 0.55), (0.01, y1, low[k]), "glass_lab", ang, pivot)
                rotated_box(b, (-0.01, y0, high[k]), (0.01, y1, 1.75), "glass_lab", ang, pivot)
                rotated_box(b, (-0.013, y0, low[k] - TANK_RIM), (0.013, y1, low[k]), "glass_edge", ang, pivot)
                rotated_box(b, (-0.013, y0, high[k]), (0.013, y1, high[k] + TANK_RIM), "glass_edge", ang, pivot)
                if k < 2:
                    for a0, a1 in ((low[k], low[k + 1]), (high[k], high[k + 1])):
                        rotated_box(b, (-0.013, y1 - TANK_RIM / 2, min(a0, a1)), (0.013, y1 + TANK_RIM / 2, max(a0, a1)), "glass_edge", ang, pivot)
            side = 0 if i == TANK_HOLE_INDICES[0] else 2
            y_side = -chord / 2 if side == 0 else chord / 2 - TANK_RIM
            rotated_box(b, (-0.013, y_side, low[side]), (0.013, y_side + TANK_RIM, high[side]), "glass_edge", ang, pivot)
        else:
            rotated_box(b, (-0.01, -chord / 2, 0.25), (0.01, chord / 2, 2.3), "glass_lab", ang, pivot)

    b.cylinder((0.0, 0.0, 2.3), (0.0, 0.0, 2.55), 0.62, "steel_dark", segments=n)
    b.cylinder((0.2, 0.0, 2.55), (0.2, 0.0, 2.68), 0.03, "conduit", segments=8)
    b.cylinder((-0.2, 0.0, 2.55), (-0.2, 0.0, 2.7), 0.03, "conduit", segments=8)

    cage_n = 10
    for i in range(cage_n):
        ang = i * 2 * math.pi / cage_n
        x0 = TANK_CAGE_R * math.cos(ang)
        y0 = TANK_CAGE_R * math.sin(ang)
        if i in TANK_BENT_BARS:
            x1 = (TANK_CAGE_R + 0.22) * math.cos(ang)
            y1 = (TANK_CAGE_R + 0.22) * math.sin(ang)
            b.cylinder((x0, y0, 0.0), (x0, y0, 0.3), 0.022, "steel_dark", segments=8)
            b.cylinder((x0, y0, 0.3), (x1, y1, 1.65), 0.022, "steel_dark", segments=8)
        else:
            b.cylinder((x0, y0, 0.0), (x0, y0, 2.25), 0.022, "steel_dark", segments=8)

    for i in range(22):
        spread = math.radians(-50 + 100 * ((i * 0.381) % 1.0))
        r = 0.7 + 0.22 * ((i * 0.618) % 1.0)
        cx, cy = r * math.sin(spread), r * math.cos(spread)
        size = 0.02 + 0.03 * ((i * 7) % 5) / 5
        ang = (i * 0.9) % math.pi
        rotated_box(b, (-size, -size * 0.45, 0.003), (size, size * 0.45, 0.006), "glass_lab", ang, (cx, cy, 0.0))

    for cx, cy, hx, hy, ang_deg, top in TANK_PUDDLE:
        rotated_box(b, (-hx, -hy, 0.0), (hx, hy, top), "specimen_fluid", math.radians(ang_deg), (cx, cy, 0.0))
    b.box((-0.25, -0.25, 0.25), (0.25, 0.25, 0.252), "specimen_fluid")
    return b.build()


def lab_console(palette):
    b = Builder("lab_console", palette)
    b.box((-0.75, -0.35, 0.86), (0.75, 0.35, 0.9), "steel_door", bevel=0.01)
    for x in (-0.68, 0.68):
        for y in (-0.28, 0.28):
            b.box((x - 0.03, y - 0.03, 0.0), (x + 0.03, y + 0.03, 0.86), "steel_dark")
    angle = -math.radians(20)
    pivot = (0.0, -0.25, 0.9)
    rotated_box(b, (-0.4, -0.02, 0.0), (0.4, 0.02, 0.42), "steel_door", angle, pivot, bevel=0.006, axis="X")
    rotated_box(b, (-0.25, -0.03, 0.22), (0.25, 0.0, 0.36), "glass_dark", angle, pivot, axis="X")
    rotated_box(b, (-0.18, -0.032, 0.28), (0.18, -0.028, 0.3), "steel_dark", angle, pivot, axis="X")
    for x in (-0.15, 0.0, 0.15):
        rotated_box(b, (x - 0.02, -0.025, 0.05), (x + 0.02, -0.018, 0.09), "steel_dark", angle, pivot, axis="X")
    b.box((0.35, 0.05, 0.9), (0.55, 0.22, 0.908), "steel_dark")
    b.box((0.36, 0.06, 0.908), (0.54, 0.21, 0.911), "paper")
    b.box((-0.55, 0.0, 0.9), (-0.35, 0.18, 0.93), "steel_dark")
    for k in range(4):
        x0 = -0.52 + k * 0.045
        b.cylinder((x0, 0.02, 0.935), (x0 + 0.05, 0.16, 0.955), 0.012, "glass_dark", segments=8)
    b.cylinder((0.62, 0.15, 0.9), (0.62, 0.15, 0.975), 0.035, "steel_dark", segments=10)
    return b.build()


def work_island(palette):
    b = Builder("work_island", palette)
    b.box((-0.95, -0.45, 0.88), (0.95, 0.45, 0.92), "steel_door", bevel=0.01)
    for x in (-0.88, 0.88):
        for y in (-0.38, 0.38):
            b.box((x - 0.035, y - 0.035, 0.0), (x + 0.035, y + 0.035, 0.88), "steel_dark")
    b.box((-0.85, -0.35, 0.18), (0.85, 0.35, 0.22), "steel_dark")
    b.box((-0.75, -0.25, 0.22), (-0.45, 0.1, 0.42), "steel_door", bevel=0.01)
    b.box((-0.35, -0.25, 0.22), (-0.1, 0.05, 0.34), "plastic_bin")
    b.box((-0.05, -0.25, 0.22), (0.2, 0.05, 0.34), "plastic_bin")
    b.box((0.65, 0.25, 0.92), (0.9, 0.42, 1.0), "steel_dark")
    b.box((0.68, 0.3, 1.0), (0.76, 0.42, 1.08), "steel_dark")
    b.box((0.78, 0.3, 1.0), (0.86, 0.42, 1.08), "steel_dark")
    for x0, y0, ang_deg in ((-0.3, 0.1, 15), (-0.55, -0.15, 70)):
        rotated_box(b, (-0.08, -0.11, 0.92), (0.08, 0.11, 0.922), "paper", math.radians(ang_deg), (x0, y0, 0.0))
    rotated_box(b, (-0.22, -0.015, 0.92), (0.22, 0.015, 0.928), "steel_dark", math.radians(40), (0.1, -0.3, 0.0))
    b.cylinder((0.25, -0.05, 0.925), (0.5, 0.05, 0.925), 0.015, "steel_dark", segments=8)
    b.cylinder((0.4, 0.2, 0.93), (0.4, 0.2, 1.0), 0.035, "steel_dark", segments=10)
    rotated_box(b, (-0.14, -0.1, 0.0), (0.14, 0.1, 0.04), "steel_door", math.radians(-25), (-0.1, 0.25, 0.92), axis="X")
    return b.build()


def clutter_papers(palette):
    b = Builder("clutter_papers", palette)
    positions = (
        (-0.4, -0.3, 10), (-0.15, -0.25, -20), (0.1, -0.29, 35), (0.35, -0.2, -10),
        (-0.3, 0.15, 55), (0.0, 0.25, -35), (0.3, 0.3, 15), (0.45, 0.05, -50),
    )
    for x, y, ang_deg in positions:
        rotated_box(b, (-0.08, -0.1, 0.0), (0.08, 0.1, 0.0012), "paper", math.radians(ang_deg), (x, y, 0.0))
    b.box((-0.08, -0.08, 0.0012), (0.1, 0.12, 0.014), "steel_dark")
    b.box((-0.07, -0.07, 0.014), (0.09, 0.11, 0.0155), "paper")
    for dx, dy in ((0.0, 0.03), (0.025, 0.045), (0.05, 0.03)):
        b.box((dx - 0.012, dy - 0.012, 0.0155), (dx + 0.012, dy + 0.012, 0.016), "spill_dark")
    return b.build()


def clutter_tools(palette):
    b = Builder("clutter_tools", palette)
    b.box((-0.28, -0.13, 0.0), (0.28, 0.13, 0.18), "steel_door", bevel=0.01)
    rotated_box(b, (-0.26, 0.0, 0.0), (0.26, 0.2, 0.018), "steel_door", math.radians(28), (0.0, 0.13, 0.18), axis="X")
    for k in range(6):
        x = -0.4 + k * 0.14
        y = -0.3 + 0.08 * (k % 3)
        b.cylinder((x, y, 0.012), (x, y, 0.024), 0.012, "steel_dark", segments=8)
    rotated_box(b, (-0.22, -0.015, 0.0), (0.22, 0.015, 0.015), "steel_dark", math.radians(30), (0.1, -0.22, 0.0))
    b.cylinder((-0.3, 0.2, 0.03), (-0.08, 0.26, 0.03), 0.014, "steel_dark", segments=8)
    b.box((-0.1, 0.21, 0.0), (-0.02, 0.29, 0.03), "steel_dark")
    return b.build()


def chair_tipped(palette):
    b = Builder("chair_tipped", palette)
    b.box((-0.22, -0.02, 0.0), (0.22, 0.02, 0.42), "steel_dark", bevel=0.006)
    b.box((-0.22, -0.3, 0.38), (0.22, -0.02, 0.42), "steel_dark", bevel=0.006)
    b.cylinder((-0.18, -0.02, 0.03), (-0.4, -0.3, 0.03), 0.018, "steel_dark", segments=8)
    b.cylinder((0.18, -0.02, 0.03), (0.4, -0.3, 0.03), 0.018, "steel_dark", segments=8)
    b.cylinder((-0.18, 0.02, 0.4), (-0.35, 0.3, 0.1), 0.018, "steel_dark", segments=8)
    b.cylinder((0.18, 0.02, 0.4), (0.35, 0.3, 0.1), 0.018, "steel_dark", segments=8)
    return b.build()


def drum_spilled(palette):
    b = Builder("drum_spilled", palette)
    b.box((-0.4, -0.04, 0.0), (0.4, 0.04, 0.001), "steel_dark")
    b.cylinder((-0.4, 0.0, 0.29), (0.4, 0.0, 0.29), 0.29, "paint_drum", segments=20)
    for x in (-0.25, 0.05):
        b.cylinder((x, 0.0, 0.29), (x + 0.03, 0.0, 0.29), 0.285, "steel_dark", segments=20)
    b.cylinder((-0.41, 0.0, 0.29), (-0.4, 0.0, 0.29), 0.28, "steel_dark", segments=20)
    b.cylinder((0.4, 0.0, 0.29), (0.41, 0.0, 0.29), 0.28, "spill_dark", segments=20)
    b.box((0.3, -0.35, 0.0), (0.75, 0.35, 0.004), "spill_dark")
    return b.build()


TRACE_CLAW_MARKS = ((-0.18, 0.05, 25), (-0.06, 0.0, 28), (0.06, -0.02, 22), (0.18, -0.08, 26))


def trace_claw_marks(palette):
    b = Builder("trace_claw_marks", palette)
    for dx, dz, ang_deg in TRACE_CLAW_MARKS:
        ang = math.radians(ang_deg)
        pivot = (dx, 0.0, dz)
        rotated_box(b, (-0.15, 0.0, -0.01), (0.15, 0.012, 0.01), "steel_dark", ang, pivot, axis="Y")
        rotated_box(b, (-0.13, 0.0, 0.009), (0.13, 0.004, 0.013), "concrete_wall", ang, pivot, axis="Y")
    return b.build()


def trace_drag_marks(palette):
    b = Builder("trace_drag_marks", palette)
    for x_center, seed in ((-0.16, 0), (0.0, 1), (0.15, 2)):
        for seg in range(6):
            y0 = -0.9 + seg * 0.3
            w = 0.012 + 0.02 * ((seg + seed) % 3) / 3
            drift = 0.025 * math.sin(seg * 1.7 + seed)
            ang = math.radians(4 * math.sin(seg + seed * 2.1))
            top = 0.002 + 0.0005 * ((seg + seed) % 2)
            rotated_box(b, (-w, -0.16, 0.0), (w, 0.16, top), "spill_dark", ang, (x_center + drift, y0 + 0.15, 0.0))
    return b.build()


class Scene:
    def __init__(self, scene):
        self.scene = scene
        self.root = bpy.data.objects.new("facility", None)
        scene.collection.objects.link(self.root)
        self.groups = {}
        self.counts = {}

    def group(self, name):
        if name not in self.groups:
            obj = bpy.data.objects.new(name, None)
            obj.parent = self.root
            self.scene.collection.objects.link(obj)
            self.groups[name] = obj
        return self.groups[name]

    def place(self, group, mesh, loc, rot_z=0.0):
        key = f"{group}.{mesh.name}"
        n = self.counts.get(key, 0)
        self.counts[key] = n + 1
        obj = bpy.data.objects.new(f"{group}_{mesh.name}_{n:02d}", mesh)
        obj.location = loc
        obj.rotation_euler = (0.0, 0.0, rot_z)
        obj.parent = self.group(group)
        self.scene.collection.objects.link(obj)
        return obj

    def anchor(self, group, name, loc, **props):
        obj = bpy.data.objects.new(name, None)
        obj.location = loc
        obj.parent = self.group(group)
        for k, v in props.items():
            obj[k] = v
        self.scene.collection.objects.link(obj)
        return obj


def to_bevy(v):
    return [round(v[0], 4), round(v[2], 4), round(-v[1], 4)]


def compose(scene, m):
    s = Scene(scene)
    h = TILE / 2
    q = math.pi / 2

    corridor = [(0, j) for j in range(CORRIDOR_CELLS)]
    room = [(i, j) for i in ROOM_I for j in ROOM_J]
    for i, j in corridor:
        s.place("corridor", m["floor_tile_marked"], (i * TILE, j * TILE + h, 0.0))
        s.place("corridor", m["ceiling_tile"], (i * TILE, j * TILE + h, 0.0))
    for i, j in room:
        s.place("room", m["floor_tile"], (i * TILE, j * TILE + h, 0.0))
        s.place("room", m["ceiling_tile"], (i * TILE, j * TILE + h, 0.0))

    for j in range(CORRIDOR_CELLS):
        y = j * TILE + h
        if j == 2:
            s.place("corridor", m["wall_doorway"], (-h, y, 0.0), -q)
            s.place("corridor", m["door_panel"], (-h, y - DOOR_W / 2 + 0.01, 0.0), q)
        else:
            s.place("corridor", m["wall_conduit"], (-h, y, 0.0), -q)
        s.place("corridor", m["wall"], (h, y, 0.0), q)
    s.place("corridor", m["wall"], (0.0, 0.0, 0.0), 0.0)

    end_y = CORRIDOR_CELLS * TILE
    s.place("room", m["wall_doorway"], (0.0, end_y, 0.0), math.pi)
    door = s.place("room", m["door_panel"], (DOOR_W / 2 - 0.01, end_y + WALL_T / 2 + 0.04, 0.0), math.pi)
    door.rotation_euler.z = math.pi - math.radians(62)
    for i in (-1, 1):
        s.place("room", m["wall"], (i * TILE, end_y, 0.0), math.pi)
    far_y = (ROOM_J[-1] + 1) * TILE
    for i in ROOM_I:
        s.place("room", m["wall"], (i * TILE, far_y, 0.0), 0.0)
    for j in ROOM_J:
        y = j * TILE + h
        s.place("room", m["wall_conduit"], (-1.5 * TILE, y, 0.0), -q)
        s.place("room", m["wall"], (1.5 * TILE, y, 0.0), q)

    posts = {(-h, 0.0), (h, 0.0)}
    posts |= {(x, j * TILE) for x in (-h, h) for j in range(1, CORRIDOR_CELLS + 1)}
    posts |= {(x, end_y) for x in (-1.5 * TILE, 1.5 * TILE)}
    posts |= {(x, far_y) for x in (-1.5 * TILE, -h, h, 1.5 * TILE)}
    posts |= {(x, j * TILE) for x in (-1.5 * TILE, 1.5 * TILE) for j in ROOM_J[1:]}
    for x, y in sorted(posts):
        s.place("structure", m["wall_post"], (x, y, 0.0))

    lights = []
    states = ("lamp_cool", "lamp_cool", "lamp_dead", "lamp_cool", "lamp_dead")
    for j, state in enumerate(states):
        y = j * TILE + h
        mesh = m["ceiling_light_cool"] if state == "lamp_cool" else m["ceiling_light_dead"]
        s.place("corridor", mesh, (0.0, y, 0.0))
        if state == "lamp_cool":
            lights.append(("corridor", f"light_corridor_{j}", (0.0, y, LIGHT_ANCHOR_Z), "ceiling", (0.78, 0.88, 1.0)))
    room_center_y = ROOM_J[1] * TILE + h
    s.place("room", m["ceiling_light_amber"], (0.0, room_center_y, 0.0), q)
    lights.append(("room", "light_room_amber", (0.0, room_center_y, LIGHT_ANCHOR_Z), "ceiling", (1.0, 0.58, 0.2)))
    red = (2.2, far_y - WALL_T / 2, 2.2)
    s.place("room", m["wall_lamp_red"], red, math.pi)
    lights.append(("room", "light_room_fault_red", (red[0], red[1] - 0.25, red[2]), "wall", (1.0, 0.06, 0.03)))

    for group, name, loc, kind, color in lights:
        s.anchor(group, name, loc, anchor="light", kind=kind, color=list(color))

    eye = (0.0, 0.6, EYE_HEIGHT)
    target = (0.0, end_y + 4.0, 1.3)
    s.anchor("structure", "camera_main", eye, anchor="camera", target=to_bevy(target))
    return lights, eye, target


def bounds(objs):
    mn = Vector((1e9, 1e9, 1e9))
    mx = -mn
    for obj in objs:
        for v in obj.data.vertices:
            w = obj.matrix_world @ v.co
            mn = Vector(map(min, mn, w))
            mx = Vector(map(max, mx, w))
    return mn, mx


FACILITY_MODULES = (
    "floor_tile", "floor_tile_marked", "ceiling_tile", "wall", "wall_conduit", "wall_doorway",
    "door_panel", "wall_post", "ceiling_light_cool", "ceiling_light_dead", "ceiling_light_amber",
    "wall_lamp_red",
)

LAMP_ANCHOR = (0.0, 0.0, LIGHT_ANCHOR_Z)
WALL_LAMP_ANCHOR = (0.0, 0.25, 0.0)
BOILER_LIGHT_ANCHOR = (0.0, 0.82, 0.5)
BOILER_LIGHT_COLOR = (1.0, 0.35, 0.08)

SIGN_LABELS = {
    "sign_label_boiler_room": ("BOILER ROOM", "sign_plate", "route_orange"),
    "sign_label_storage": ("STORAGE", "sign_plate", "route_blue"),
    "sign_label_maintenance": ("MAINTENANCE", "sign_plate", "steel_dark"),
    "sign_label_utility": ("UTILITY", "sign_plate", "steel_dark"),
    "sign_label_office": ("OFFICE", "sign_plate", "steel_dark"),
    "sign_label_lab": ("LAB", "sign_plate", "steel_dark"),
    "sign_label_security": ("SECURITY", "sign_plate", "steel_dark"),
    "sign_label_prep": ("PREP", "sign_plate", "steel_dark"),
    "sign_label_exit": ("EXIT", "route_green", None),
}

CONCEPT_TABLE_NOTE = "Visual concept only: a sturdy worktable with open space underneath for crouching or storage. No hiding, collision, or interaction."
CONCEPT_LOCKER_NOTE = "Visual concept only: a locker interior dark enough to suggest something could be hidden inside. No hiding, collision, or interaction."
CONCEPT_VENT_NOTE = "Visual concept only: a crawlspace vent suggesting a hidden passage beyond the wall. No hiding, collision, or interaction."
CONCEPT_GRILLE_NOTE = "Visual concept only: a removed vent grille lying on the floor, suggesting an opening nearby. No hiding, collision, or interaction."
CONCEPT_TANK_NOTE = "Visual concept only: a broken laboratory containment vessel as a first-encounter storytelling landmark. No monster, encounter, collision, or interaction."

KIT = {
    "floor_tile": ("floor", "cell_center", None, None),
    "floor_tile_marked": ("floor", "cell_center", None, None),
    "ceiling_tile": ("ceiling", "cell_center", None, None),
    "wall": ("wall", "edge_center", None, None),
    "wall_conduit": ("wall", "edge_center", None, None),
    "wall_doorway": ("wall", "edge_center", None, None),
    "door_panel": ("door", "door_hinge", None, None),
    "wall_post": ("structure", "vertex", None, None),
    "ceiling_light_cool": ("fixture", "cell_center", (LAMP_ANCHOR, (0.78, 0.88, 1.0)), None),
    "ceiling_light_dead": ("fixture", "cell_center", (LAMP_ANCHOR, None), None),
    "ceiling_light_amber": ("fixture", "cell_center", (LAMP_ANCHOR, (1.0, 0.58, 0.2)), None),
    "wall_lamp_red": ("fixture", "wall_mount", (WALL_LAMP_ANCHOR, (1.0, 0.06, 0.03)), None),
    "exit_sign": ("decoration", "wall_mount", ((0.0, 0.2, 0.0), (0.12, 1.0, 0.35)), None),
    "wall_vent": ("decoration", "wall_mount", None, None),
    "fuse_panel": ("decoration", "wall_mount", None, None),
    "fuse_pickup": ("prop", "floor", None, None),
    "storage_crate": ("prop", "floor", None, None),
    "steel_drum": ("prop", "floor", None, None),
    "shelf_unit": ("prop", "floor", None, None),
    "workbench": ("prop", "floor", None, None),
    "route_line_orange": ("route", "floor_line", None, {"walkable": True}),
    "route_line_blue": ("route", "floor_line", None, {"walkable": True}),
    "route_line_green": ("route", "floor_line", None, {"walkable": True}),
    "sign_label_boiler_room": ("sign", "wall_mount", None, {"text": "BOILER ROOM", "text_cap_m": SIGN_CAP}),
    "sign_label_storage": ("sign", "wall_mount", None, {"text": "STORAGE", "text_cap_m": SIGN_CAP}),
    "sign_label_maintenance": ("sign", "wall_mount", None, {"text": "MAINTENANCE", "text_cap_m": SIGN_CAP}),
    "sign_label_utility": ("sign", "wall_mount", None, {"text": "UTILITY", "text_cap_m": SIGN_CAP}),
    "sign_label_office": ("sign", "wall_mount", None, {"text": "OFFICE", "text_cap_m": SIGN_CAP}),
    "sign_label_lab": ("sign", "wall_mount", None, {"text": "LAB", "text_cap_m": SIGN_CAP}),
    "sign_label_security": ("sign", "wall_mount", None, {"text": "SECURITY", "text_cap_m": SIGN_CAP}),
    "sign_label_prep": ("sign", "wall_mount", None, {"text": "PREP", "text_cap_m": SIGN_CAP}),
    "sign_label_exit": ("sign", "wall_mount", None, {"text": "EXIT", "text_cap_m": SIGN_CAP}),
    "sign_arrow": ("sign", "wall_mount", None, {"points": "+X"}),
    "sign_hanger": ("sign", "ceiling_hang", None, None),
    "boiler_unit": ("prop", "floor", (BOILER_LIGHT_ANCHOR, BOILER_LIGHT_COLOR), None),
    "pipe_manifold": ("decoration", "wall_mount", None, None),
    "tool_pegboard": ("decoration", "wall_mount", None, None),
    "shelf_unit_low": ("prop", "floor", None, None),
    "shelf_unit_bins": ("prop", "floor", None, None),
    "concept_table": ("concept", "floor", None, {
        "concept": CONCEPT_TABLE_NOTE,
        "clearance_bevy": {"min": [-0.8, 0.0, -0.38], "max": [0.8, 0.74, 0.38]},
    }),
    "concept_locker": ("concept", "floor", None, {"concept": CONCEPT_LOCKER_NOTE}),
    "concept_crawl_vent": ("concept", "wall_mount", None, {
        "concept": CONCEPT_VENT_NOTE,
        "opening_bevy": {"min": [-0.35, -0.25], "max": [0.35, 0.25]},
    }),
    "vent_grille": ("concept", "floor", None, {"concept": CONCEPT_GRILLE_NOTE}),
    "concept_containment_tank": ("concept", "floor", None, {
        "concept": CONCEPT_TANK_NOTE,
        "breach_bevy": {"min": [-0.18, 0.9, -1.0], "max": [0.18, 1.35, -0.3]},
    }),
    "lab_console": ("prop", "floor", None, None),
    "work_island": ("prop", "floor", None, None),
    "clutter_papers": ("prop", "floor", None, {"walkable": True}),
    "clutter_tools": ("prop", "floor", None, None),
    "chair_tipped": ("prop", "floor", None, None),
    "drum_spilled": ("prop", "floor", None, None),
    "trace_claw_marks": ("decoration", "wall_mount", None, None),
    "trace_drag_marks": ("decoration", "floor", None, {"walkable": True}),
}

SNAPS = {
    "cell_center": "Origin at the cell center on the floor top (y = 0). Place at (2.5 i, 0, 2.5 k).",
    "edge_center": "Origin at the middle of a cell edge on the floor top. Length along +-X (2.5 m). The front (conduit, frame stripes) faces -Z. Rotate about Y to face the interior.",
    "vertex": "Origin at a grid vertex on the floor top.",
    "door_hinge": "Origin at the hinge edge on the floor plane. The leaf extends along +X. Closed: hinge at the edge center - 0.59 m along the wall, yaw of the doorway.",
    "wall_mount": "Origin at the back center, on the wall face. The fixture protrudes to -Z. Choose the height at placement.",
    "floor": "Origin at the bottom center on the floor top. The front faces -Z.",
    "floor_line": "Origin at the line center on the floor top (y = 0). x in [-0.5, 0.5], 0.1 m wide along Z, 3 mm high. Bevy scales along X to the run length.",
    "ceiling_hang": "Origin on the floor (y = 0) under the board center. The board hangs from rods to the ceiling.",
}


def kit_point(v):
    return [c + 0.0 for c in to_bevy(v)]


def build_modules(palette, names):
    makers = {
        "floor_tile": lambda: floor_tile(palette),
        "floor_tile_marked": lambda: floor_tile_marked(palette),
        "ceiling_tile": lambda: ceiling_tile(palette),
        "wall": lambda: wall(palette, "wall"),
        "wall_conduit": lambda: wall(palette, "wall_conduit", conduit=True),
        "wall_doorway": lambda: wall_doorway(palette),
        "door_panel": lambda: door_panel(palette),
        "wall_post": lambda: post(palette),
        "ceiling_light_cool": lambda: ceiling_light(palette, "ceiling_light_cool", "lamp_cool"),
        "ceiling_light_dead": lambda: ceiling_light(palette, "ceiling_light_dead", "lamp_dead"),
        "ceiling_light_amber": lambda: ceiling_light(palette, "ceiling_light_amber", "lamp_amber"),
        "wall_lamp_red": lambda: wall_lamp(palette, "wall_lamp_red", "lamp_red"),
        "exit_sign": lambda: exit_sign(palette),
        "wall_vent": lambda: wall_vent(palette),
        "fuse_panel": lambda: fuse_panel(palette),
        "fuse_pickup": lambda: fuse_pickup(palette),
        "storage_crate": lambda: storage_crate(palette),
        "steel_drum": lambda: steel_drum(palette),
        "shelf_unit": lambda: shelf_unit(palette),
        "workbench": lambda: workbench(palette),
        "route_line_orange": lambda: route_line(palette, "route_line_orange", "route_orange"),
        "route_line_blue": lambda: route_line(palette, "route_line_blue", "route_blue"),
        "route_line_green": lambda: route_line(palette, "route_line_green", "route_green"),
        "sign_arrow": lambda: sign_arrow(palette),
        "sign_hanger": lambda: sign_hanger(palette),
        "boiler_unit": lambda: boiler_unit(palette),
        "pipe_manifold": lambda: pipe_manifold(palette),
        "tool_pegboard": lambda: tool_pegboard(palette),
        "shelf_unit_low": lambda: shelf_unit_low(palette),
        "shelf_unit_bins": lambda: shelf_unit_bins(palette),
        "concept_table": lambda: concept_table(palette),
        "concept_locker": lambda: concept_locker(palette),
        "concept_crawl_vent": lambda: concept_crawl_vent(palette),
        "vent_grille": lambda: vent_grille(palette),
        "concept_containment_tank": lambda: concept_containment_tank(palette),
        "lab_console": lambda: lab_console(palette),
        "work_island": lambda: work_island(palette),
        "clutter_papers": lambda: clutter_papers(palette),
        "clutter_tools": lambda: clutter_tools(palette),
        "chair_tipped": lambda: chair_tipped(palette),
        "drum_spilled": lambda: drum_spilled(palette),
        "trace_claw_marks": lambda: trace_claw_marks(palette),
        "trace_drag_marks": lambda: trace_drag_marks(palette),
    }
    for label_name, (text, plate_mat, chip_mat) in SIGN_LABELS.items():
        makers[label_name] = (lambda n, t, p, c: lambda: sign_label(palette, n, t, p, c))(label_name, text, plate_mat, chip_mat)
    return {name: makers[name]() for name in names}


def mesh_bounds(mesh):
    lo = Vector(map(min, *(v.co for v in mesh.vertices)))
    hi = Vector(map(max, *(v.co for v in mesh.vertices)))
    return lo, hi


def export_glb(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(path), export_format="GLB", export_yup=True, export_apply=True,
        export_extras=True, export_lights=False, export_cameras=False,
        export_tangents=True, export_image_format="AUTO", export_materials="EXPORT",
    )


def export_kit(scene, mats, out_dir):
    out_dir.mkdir(parents=True, exist_ok=True)
    stale = {p.name for p in out_dir.glob("*.glb")} - {f"{name}.glb" for name in KIT}
    if stale:
        raise SystemExit(f"unexpected module files in {out_dir}: {sorted(stale)}")
    modules = {}
    for name, mesh in build_modules(Palette(mats, KIT_UV_PERIOD, canonical=True), KIT).items():
        obj = bpy.data.objects.new(name, mesh)
        scene.collection.objects.link(obj)
        bpy.context.view_layer.update()
        path = out_dir / f"{name}.glb"
        export_glb(path)
        bpy.data.objects.remove(obj, do_unlink=True)
        category, snap, light, extra = KIT[name]
        lo, hi = mesh_bounds(mesh)
        entry = {
            "file": path.name,
            "category": category,
            "snap": snap,
            "triangles": sum(len(p.vertices) - 2 for p in mesh.polygons),
            "materials": [mat.name for mat in mesh.materials],
            "textured": any(mat.name in ("concrete_floor", "concrete_wall", "paint_lower") for mat in mesh.materials),
            "bounds_bevy": {"min": kit_point((lo.x, hi.y, lo.z)), "max": kit_point((hi.x, lo.y, hi.z))},
        }
        if light:
            anchor, color = light
            entry["light_anchor_bevy"] = kit_point(anchor)
            entry["light_color_linear"] = list(color) if color else None
        if extra:
            entry.update(extra)
        entry["glb_sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        entry["glb_bytes"] = path.stat().st_size
        modules[name] = entry
    data = {
        "generator": "scripts/generate_facility.py",
        "blender": bpy.app.version_string,
        "units": "1 unit = 1 meter (scene.unit_settings.scale_length = 1.0)",
        "axes": "glTF +Y up. The front of each module faces -Z. bevy = (x, z, -y) from Blender",
        "grid": {"tile_m": TILE, "wall_height_m": WALL_H, "wall_thickness_m": WALL_T, "post_m": POST, "door_opening_m": [DOOR_W, DOOR_H], "slab_m": SLAB_T, "uv_period_m": KIT_UV_PERIOD},
        "snaps": SNAPS,
        "modules": modules,
    }
    (out_dir / "modules.manifest.json").write_text(json.dumps(data, indent=2) + "\n")
    print(f"wrote {len(modules)} modules to {out_dir}")


def export_facility(scene, mats, out, manifest):
    m = build_modules(Palette(mats, TEXTURE_METERS, canonical=False), FACILITY_MODULES)
    lights, eye, target = compose(scene, m)
    bpy.context.view_layer.update()

    meshes = [o for o in scene.objects if o.type == "MESH"]
    mn, mx = bounds(meshes)
    modules = {}
    for name, mesh in sorted(m.items()):
        lo = Vector(map(min, *(v.co for v in mesh.vertices))) if len(mesh.vertices) > 1 else mesh.vertices[0].co
        hi = Vector(map(max, *(v.co for v in mesh.vertices))) if len(mesh.vertices) > 1 else mesh.vertices[0].co
        modules[name] = {
            "triangles": sum(len(p.vertices) - 2 for p in mesh.polygons),
            "materials": [mat.name for mat in mesh.materials],
            "local_min_blender": [round(c, 4) for c in lo],
            "local_max_blender": [round(c, 4) for c in hi],
            "instances": sum(1 for o in meshes if o.data == mesh),
        }

    export_glb(out)

    data = {
        "generator": "scripts/generate_facility.py",
        "blender": bpy.app.version_string,
        "units": "1 unit = 1 meter (scene.unit_settings.scale_length = 1.0)",
        "axes": "Blender +Z up, corridor along +Y; GLB is glTF +Y up, corridor along -Z. bevy = (x, z, -y)",
        "grid": {"tile_m": TILE, "wall_height_m": WALL_H, "wall_thickness_m": WALL_T, "post_m": POST, "door_opening_m": [DOOR_W, DOOR_H], "texture_tile_m": TEXTURE_METERS},
        "bounds_bevy": {"min": to_bevy((mn.x, mx.y, mn.z)), "max": to_bevy((mx.x, mn.y, mx.z))},
        "triangles_total": sum(len(p.vertices) - 2 for o in meshes for p in o.data.polygons),
        "objects": len(meshes),
        "modules": modules,
        "lights_bevy": [{"name": n, "group": g, "kind": k, "position": to_bevy(p), "color_linear": list(c)} for g, n, p, k, c in lights],
        "camera_bevy": {"position": to_bevy(eye), "look_at": to_bevy(target), "up": [0.0, 1.0, 0.0]},
        "glb_sha256": hashlib.sha256(out.read_bytes()).hexdigest(),
        "glb_bytes": out.stat().st_size,
    }
    manifest.write_text(json.dumps(data, indent=2) + "\n")
    print(f"wrote {out} ({data['glb_bytes']} bytes, {data['triangles_total']} triangles)")


def main():
    args = parse_args()
    root = pathlib.Path(args.root).resolve()
    out = pathlib.Path(args.out).resolve()
    scene = reset_scene()
    mats = make_materials(root)
    if args.target == "kit":
        export_kit(scene, make_kit_materials(mats), out)
    else:
        export_facility(scene, mats, out, pathlib.Path(args.manifest).resolve())


main()
