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
TEXTURE = "concrete_floor_worn_001"

CORRIDOR_CELLS = 5
ROOM_I = (-1, 0, 1)
ROOM_J = (5, 6, 7)

LIGHT_ANCHOR_Z = WALL_H - 0.3
EYE_HEIGHT = 1.6


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--manifest", required=True)
    return parser.parse_args(argv)


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


def material(name, color, metallic=0.0, roughness=0.6, emission=None, strength=0.0, textures=None):
    mat = bpy.data.materials.new(name)
    mat.use_backface_culling = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    bsdf = nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
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


class Builder:
    def __init__(self, name, mats):
        self.name = name
        self.mats = mats
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
            self.bm, cap_ends=True, cap_tris=False, segments=segments,
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
        bmesh.ops.triangulate(self.bm, faces=self.bm.faces[:])
        mesh = bpy.data.meshes.new(self.name)
        self.bm.to_mesh(mesh)
        self.bm.free()
        for mat in self.slots:
            mesh.materials.append(self.mats[mat])
        box_uv(mesh)
        return mesh


def box_uv(mesh):
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
            uv.data[li].uv = (u / TEXTURE_METERS, v / TEXTURE_METERS)


def floor_tile(mats):
    b = Builder("floor_tile", mats)
    h = TILE / 2
    b.box((-h, -h, -SLAB_T), (h, h, 0.0), "concrete_floor", bevel=0.012)
    return b.build()


def floor_tile_marked(mats):
    b = Builder("floor_tile_marked", mats)
    h = TILE / 2
    b.box((-h, -h, -SLAB_T), (h, h, 0.0), "concrete_floor", bevel=0.012)
    for x in (-0.95, 0.95):
        b.box((x - 0.04, -h + 0.02, 0.0), (x + 0.04, h - 0.02, 0.003), "paint_hazard")
    return b.build()


def ceiling_tile(mats):
    b = Builder("ceiling_tile", mats)
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


def wall(mats, name, conduit=False):
    b = Builder(name, mats)
    wall_bands(b, opening=False)
    if conduit:
        h = TILE / 2
        y = WALL_T / 2 + 0.09
        for z, r in ((2.55, 0.045), (2.43, 0.03)):
            b.cylinder((-h, y, z), (h, y, z), r, "conduit")
        for x in (-0.6, 0.6):
            b.box((x - 0.03, WALL_T / 2, 2.38), (x + 0.03, y + 0.06, 2.62), "steel_dark")
    return b.build()


def wall_doorway(mats):
    b = Builder("wall_doorway", mats)
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


def door_panel(mats):
    b = Builder("door_panel", mats)
    w = DOOR_W - 0.02
    t = 0.06
    b.box((0.0, -t / 2, 0.01), (w, t / 2, DOOR_H - 0.01), "steel_door", bevel=0.008)
    b.box((0.35, -t / 2 - 0.006, 1.45), (0.85, t / 2 + 0.006, 1.85), "glass_dark")
    b.box((0.05, -t / 2 - 0.004, 0.05), (w - 0.05, t / 2 + 0.004, 0.3), "steel_dark")
    for s in (-1, 1):
        b.box((w - 0.16, s * (t / 2 + 0.02) - 0.015, 1.0), (w - 0.12, s * (t / 2 + 0.02) + 0.015, 1.12), "conduit")
    return b.build()


def post(mats):
    b = Builder("wall_post", mats)
    p = POST / 2
    b.box((-p, -p, 0.0), (p, p, WALL_H), "steel_dark", bevel=0.015)
    b.box((-p - 0.01, -p - 0.01, 1.4), (p + 0.01, p + 0.01, 1.5), "paint_hazard")
    return b.build()


def ceiling_light(mats, name, lamp):
    b = Builder(name, mats)
    z = WALL_H - 0.12
    b.box((-0.7, -0.16, z - 0.09), (0.7, 0.16, z), "steel_dark", bevel=0.01)
    b.box((-0.64, -0.1, z - 0.1), (0.64, 0.1, z - 0.09), lamp)
    for x in (-0.55, 0.55):
        b.cylinder((x, 0.0, z), (x, 0.0, WALL_H), 0.012, "steel_dark", segments=6)
    return b.build()


def wall_lamp(mats, name, lamp):
    b = Builder(name, mats)
    b.box((-0.12, 0.0, -0.16), (0.12, 0.05, 0.16), "steel_dark", bevel=0.008)
    b.cylinder((0.0, 0.05, 0.0), (0.0, 0.14, 0.0), 0.075, lamp, segments=16)
    for a in range(4):
        ang = a * math.pi / 2 + math.pi / 4
        x = math.cos(ang) * 0.09
        z = math.sin(ang) * 0.09
        b.cylinder((x, 0.05, z), (x, 0.16, z), 0.008, "steel_dark", segments=6)
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


def main():
    args = parse_args()
    root = pathlib.Path(args.root).resolve()
    out = pathlib.Path(args.out).resolve()
    manifest = pathlib.Path(args.manifest).resolve()
    scene = reset_scene()
    mats = make_materials(root)
    m = {}
    for mesh in (
        floor_tile(mats), floor_tile_marked(mats), ceiling_tile(mats),
        wall(mats, "wall"), wall(mats, "wall_conduit", conduit=True), wall_doorway(mats),
        door_panel(mats), post(mats),
        ceiling_light(mats, "ceiling_light_cool", "lamp_cool"),
        ceiling_light(mats, "ceiling_light_dead", "lamp_dead"),
        ceiling_light(mats, "ceiling_light_amber", "lamp_amber"),
        wall_lamp(mats, "wall_lamp_red", "lamp_red"),
    ):
        m[mesh.name] = mesh
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

    out.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(out), export_format="GLB", export_yup=True, export_apply=True,
        export_extras=True, export_lights=False, export_cameras=False,
        export_tangents=True, export_image_format="AUTO", export_materials="EXPORT",
    )

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


main()
