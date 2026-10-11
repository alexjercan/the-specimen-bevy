import argparse
import hashlib
import json
import math
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import bmesh
import bpy
from mathutils import Matrix, Vector

from generate_facility import (
    DOOR_H,
    DOOR_W,
    FUSE_PICKUP_R,
    KIT_UV_PERIOD,
    POST,
    SLAB_T,
    SNAPS,
    TILE,
    WALL_H,
    WALL_T,
    Builder,
    Palette,
    export_glb,
    kit_point,
    material,
    mesh_bounds,
    reset_scene,
)


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True)
    parser.add_argument("--out", required=True)
    return parser.parse_args(argv)


def make_halloween_materials():
    return {
        "candy_orange": material(
            "candy_orange", (0.95, 0.4, 0.05), roughness=0.45,
            emission=(1.0, 0.35, 0.05), strength=0.45,
        ),
        "candy_stripe": material("candy_stripe", (0.08, 0.04, 0.1), roughness=0.5),
        "candy_wrapper": material("candy_wrapper", (0.55, 0.15, 0.65), roughness=0.35, metallic=0.1),
        "conduit": material("conduit", (0.32, 0.33, 0.31), metallic=0.85, roughness=0.4),
        "steel_dark": material("steel_dark", (0.07, 0.075, 0.08), metallic=0.8, roughness=0.45),
        "pumpkin_skin": material("pumpkin_skin", (0.82, 0.38, 0.06), roughness=0.7),
        "pumpkin_glow": material(
            "pumpkin_glow", (1.0, 0.75, 0.25), roughness=0.4,
            emission=(1.0, 0.65, 0.15), strength=6.0,
        ),
        "pumpkin_stem": material("pumpkin_stem", (0.3, 0.26, 0.1), roughness=0.75),
        "pumpkin_cut": material("pumpkin_cut", (0.11, 0.035, 0.015), roughness=0.9),
    }


def frustum(b, start, end, r0, r1, mat, segments=12):
    start = Vector(start)
    end = Vector(end)
    axis = end - start
    geom = bmesh.ops.create_cone(
        b.bm, cap_ends=True, cap_tris=b.canonical, segments=segments,
        radius1=r0, radius2=r1, depth=axis.length,
    )
    rot = Vector((0, 0, 1)).rotation_difference(axis.normalized()).to_matrix().to_4x4()
    bmesh.ops.transform(b.bm, matrix=Matrix.Translation((start + end) / 2) @ rot, verts=geom["verts"])
    b._finish(geom["verts"], mat, 0.0, smooth=True)


def ribbed_pumpkin_body(b, radius, half_height, ribs, rib_amp, mat, u_segments=24, v_segments=8):
    geom = bmesh.ops.create_uvsphere(b.bm, u_segments=u_segments, v_segments=v_segments, radius=1.0)
    verts = geom["verts"]
    for v in verts:
        x, y, z = v.co
        theta = math.atan2(y, x)
        rib = 1.0 + rib_amp * math.cos(ribs * theta)
        v.co.x = x * radius * rib
        v.co.y = y * radius * rib
        v.co.z = z * half_height + half_height
    b._finish(verts, mat, 0.0, smooth=True)


CANDY_BODY_X = 0.05
CANDY_BODY_R = 0.026
CANDY_PINCH_X = 0.065
CANDY_PINCH_R = 0.008
CANDY_FLARE_X = 0.085
CANDY_FLARE_R = FUSE_PICKUP_R
CANDY_TERMINAL_X0 = 0.085
CANDY_TERMINAL_X1 = 0.1
CANDY_TERMINAL_HALF_T = 0.003
CANDY_TERMINAL_HALF_H = 0.016
CANDY_STRIPE_X = (0.015, 0.035)
CANDY_STRIPE_HALF_W = 0.004
CANDY_STRIPE_R = CANDY_BODY_R + 0.001
CANDY_Z = FUSE_PICKUP_R


def candy_fuse(palette):
    b = Builder("candy_fuse", palette)
    z = CANDY_Z
    b.cylinder((-CANDY_BODY_X, 0.0, z), (CANDY_BODY_X, 0.0, z), CANDY_BODY_R, "candy_orange", segments=16)
    for s in (-1.0, 1.0):
        for bx in CANDY_STRIPE_X:
            x = s * bx
            b.cylinder(
                (x - CANDY_STRIPE_HALF_W, 0.0, z), (x + CANDY_STRIPE_HALF_W, 0.0, z),
                CANDY_STRIPE_R, "candy_stripe", segments=16,
            )
    for s in (-1.0, 1.0):
        x0, x1, x2 = s * CANDY_BODY_X, s * CANDY_PINCH_X, s * CANDY_FLARE_X
        frustum(b, (x0, 0.0, z), (x1, 0.0, z), CANDY_BODY_R, CANDY_PINCH_R, "candy_wrapper", segments=16)
        frustum(b, (x1, 0.0, z), (x2, 0.0, z), CANDY_PINCH_R, CANDY_FLARE_R, "candy_wrapper", segments=16)
    for s in (-1.0, 1.0):
        lo, hi = sorted((s * CANDY_TERMINAL_X0, s * CANDY_TERMINAL_X1))
        b.box(
            (lo, -CANDY_TERMINAL_HALF_T, z - CANDY_TERMINAL_HALF_H),
            (hi, CANDY_TERMINAL_HALF_T, z + CANDY_TERMINAL_HALF_H),
            "conduit", bevel=0.001,
        )
    return b.build()


JACKO_RADIUS = 0.23
JACKO_HALF_HEIGHT = 0.17
JACKO_RIBS = 9
JACKO_RIB_AMP = 0.08
JACKO_STEM_R0 = 0.032
JACKO_STEM_R1 = 0.015
JACKO_STEM_H = 0.075
JACKO_LIGHT_ANCHOR = (0.0, 0.0, 0.17)
JACKO_LIGHT_COLOR = (1.0, 0.65, 0.15)


def face_patch(b, points, mat, lift=0.023):
    cx = sum(x for x, _ in points) / len(points)
    cz = sum(z for _, z in points) / len(points)

    def surface(x, z):
        theta = math.atan2(1.0, x)
        rib = 1.0 + JACKO_RIB_AMP * math.cos(JACKO_RIBS * theta)
        shell = max(0.0, 1.0 - ((z - JACKO_HALF_HEIGHT) / JACKO_HALF_HEIGHT) ** 2)
        return math.sqrt(max(0.0, (JACKO_RADIUS * rib) ** 2 * shell - x * x)) + lift

    center = b.bm.verts.new((cx, surface(cx, cz), cz))
    edges = [b.bm.verts.new((x, surface(x, z), z)) for x, z in points]
    for i, vertex in enumerate(edges):
        face = b.bm.faces.new((center, edges[(i + 1) % len(edges)], vertex))
        face.material_index = b.slot(mat)
        face.normal_update()
        if face.normal.y < 0:
            face.normal_flip()


def jack_o_lantern(palette):
    b = Builder("jack_o_lantern", palette)
    ribbed_pumpkin_body(b, JACKO_RADIUS, JACKO_HALF_HEIGHT, JACKO_RIBS, JACKO_RIB_AMP, "pumpkin_skin")
    body_top = 2.0 * JACKO_HALF_HEIGHT
    frustum(
        b, (0.0, 0.0, body_top), (0.0, 0.0, body_top + JACKO_STEM_H),
        JACKO_STEM_R0, JACKO_STEM_R1, "pumpkin_stem", segments=8,
    )
    for s in (-1.0, 1.0):
        face_patch(b, [(s * 0.035, 0.225), (s * 0.115, 0.235), (s * 0.105, 0.17), (s * 0.067, 0.165)], "pumpkin_cut")
        face_patch(b, [(s * 0.05, 0.215), (s * 0.103, 0.223), (s * 0.095, 0.178), (s * 0.069, 0.176)], "pumpkin_glow", 0.026)
        face_patch(b, [(s * 0.008, 0.16), (s * 0.023, 0.14), (s * 0.006, 0.132)], "pumpkin_glow")
    face_patch(b, [(-0.105, 0.19), (-0.065, 0.17), (0.0, 0.166), (0.065, 0.17), (0.105, 0.19),
                   (0.09, 0.128), (0.045, 0.121), (0.0, 0.12), (-0.045, 0.121), (-0.09, 0.128)], "pumpkin_cut")
    for x in (-0.092, -0.046, 0.0, 0.046, 0.092):
        face_patch(b, [(x - 0.01, 0.163), (x + 0.01, 0.163), (x + 0.008, 0.142), (x - 0.008, 0.142)], "pumpkin_glow", 0.026)
    for x in (-0.066, 0.066):
        face_patch(b, [(x - 0.012, 0.129), (x + 0.012, 0.129), (x, 0.148)], "pumpkin_glow", 0.026)
    return b.build()


PUMPKIN_SMALL_RADIUS = 0.135
PUMPKIN_SMALL_HALF_HEIGHT = 0.105
PUMPKIN_SMALL_RIBS = 8
PUMPKIN_SMALL_RIB_AMP = 0.08
PUMPKIN_SMALL_STEM_R0 = 0.02
PUMPKIN_SMALL_STEM_R1 = 0.01
PUMPKIN_SMALL_STEM_H = 0.05


def pumpkin_small(palette):
    b = Builder("pumpkin_small", palette)
    ribbed_pumpkin_body(
        b, PUMPKIN_SMALL_RADIUS, PUMPKIN_SMALL_HALF_HEIGHT, PUMPKIN_SMALL_RIBS, PUMPKIN_SMALL_RIB_AMP,
        "pumpkin_skin",
    )
    body_top = 2.0 * PUMPKIN_SMALL_HALF_HEIGHT
    frustum(
        b, (0.0, 0.0, body_top), (0.0, 0.0, body_top + PUMPKIN_SMALL_STEM_H),
        PUMPKIN_SMALL_STEM_R0, PUMPKIN_SMALL_STEM_R1, "pumpkin_stem", segments=8,
    )
    return b.build()


HALLOWEEN_KIT = {
    "candy_fuse": ("prop", "floor", None, None),
    "jack_o_lantern": ("decoration", "floor", (JACKO_LIGHT_ANCHOR, JACKO_LIGHT_COLOR), None),
    "pumpkin_small": ("decoration", "floor", None, None),
}

HALLOWEEN_MAKERS = {
    "candy_fuse": candy_fuse,
    "jack_o_lantern": jack_o_lantern,
    "pumpkin_small": pumpkin_small,
}


def export_halloween_kit(scene, mats, out_dir):
    out_dir.mkdir(parents=True, exist_ok=True)
    stale = {p.name for p in out_dir.glob("*.glb")} - {f"{name}.glb" for name in HALLOWEEN_KIT}
    if stale:
        raise SystemExit(f"unexpected module files in {out_dir}: {sorted(stale)}")
    palette = Palette(mats, KIT_UV_PERIOD, canonical=True)
    modules = {}
    for name in HALLOWEEN_KIT:
        mesh = HALLOWEEN_MAKERS[name](palette)
        obj = bpy.data.objects.new(name, mesh)
        scene.collection.objects.link(obj)
        bpy.context.view_layer.update()
        path = out_dir / f"{name}.glb"
        export_glb(path)
        bpy.data.objects.remove(obj, do_unlink=True)
        category, snap, light, extra = HALLOWEEN_KIT[name]
        lo, hi = mesh_bounds(mesh)
        entry = {
            "file": path.name,
            "category": category,
            "snap": snap,
            "triangles": sum(len(p.vertices) - 2 for p in mesh.polygons),
            "materials": [mat.name for mat in mesh.materials],
            "textured": False,
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
        "generator": "scripts/generate_halloween.py",
        "blender": bpy.app.version_string,
        "units": "1 unit = 1 meter (scene.unit_settings.scale_length = 1.0)",
        "axes": "glTF +Y up. The front of each module faces -Z. bevy = (x, z, -y) from Blender",
        "grid": {"tile_m": TILE, "wall_height_m": WALL_H, "wall_thickness_m": WALL_T, "post_m": POST, "door_opening_m": [DOOR_W, DOOR_H], "slab_m": SLAB_T, "uv_period_m": KIT_UV_PERIOD},
        "snaps": SNAPS,
        "modules": modules,
    }
    (out_dir / "modules.manifest.json").write_text(json.dumps(data, indent=2) + "\n")
    print(f"wrote {len(modules)} modules to {out_dir}")


def main():
    args = parse_args()
    out = pathlib.Path(args.out).resolve()
    scene = reset_scene()
    mats = make_halloween_materials()
    export_halloween_kit(scene, mats, out)


if __name__ == "__main__":
    main()
