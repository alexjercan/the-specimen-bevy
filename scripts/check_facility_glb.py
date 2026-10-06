#!/usr/bin/env python3
import json
import pathlib
import struct
import sys

root = pathlib.Path(__file__).resolve().parent.parent
path = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else root / "art/visuals/generated/facility.glb"
manifest = json.loads(path.with_name("facility.manifest.json").read_text())
data = path.read_bytes()
errors = []

magic, version, length = struct.unpack_from("<4sII", data, 0)
if magic != b"glTF" or version != 2 or length != len(data):
    sys.exit(f"{path}: not a valid GLB 2.0 file")
json_len, json_type = struct.unpack_from("<I4s", data, 12)
gltf = json.loads(data[20:20 + json_len])


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def quat_rotate(q, v):
    u = q[:3]
    w = q[3]
    t = tuple(2.0 * c for c in cross(u, v))
    c = cross(u, t)
    return tuple(v[i] + w * t[i] + c[i] for i in range(3))


def compose(parent, node):
    t = node.get("translation", [0.0, 0.0, 0.0])
    r = node.get("rotation", [0.0, 0.0, 0.0, 1.0])
    s = node.get("scale", [1.0, 1.0, 1.0])
    if "matrix" in node:
        errors.append(f"node {node.get('name')} uses a matrix transform")

    def apply(v):
        v = quat_rotate(r, (v[0] * s[0], v[1] * s[1], v[2] * s[2]))
        return parent((v[0] + t[0], v[1] + t[1], v[2] + t[2]))

    return apply


lo = [float("inf")] * 3
hi = [float("-inf")] * 3
named = {}


def walk(index, parent):
    node = gltf["nodes"][index]
    named[node.get("name")] = (node, compose(parent, node))
    xf = named[node.get("name")][1]
    if "mesh" in node:
        for prim in gltf["meshes"][node["mesh"]]["primitives"]:
            acc = gltf["accessors"][prim["attributes"]["POSITION"]]
            amin, amax = acc["min"], acc["max"]
            for cx in (amin[0], amax[0]):
                for cy in (amin[1], amax[1]):
                    for cz in (amin[2], amax[2]):
                        p = xf((cx, cy, cz))
                        for i in range(3):
                            lo[i] = min(lo[i], p[i])
                            hi[i] = max(hi[i], p[i])
    for child in node.get("children", []):
        walk(child, xf)


for index in gltf["scenes"][gltf.get("scene", 0)]["nodes"]:
    walk(index, lambda v: v)


def near(a, b, tol=0.02):
    return all(abs(x - y) <= tol for x, y in zip(a, b))


expected_lo = manifest["bounds_bevy"]["min"]
expected_hi = manifest["bounds_bevy"]["max"]
if not near(lo, expected_lo) or not near(hi, expected_hi):
    errors.append(f"bounds {lo} {hi} differ from manifest {expected_lo} {expected_hi}")
grid = manifest["grid"]
height = hi[1]
if abs(height - (grid["wall_height_m"] + 0.1)) > 0.02:
    errors.append(f"top of ceiling slab at {height:.3f} m, expected {grid['wall_height_m'] + 0.1} m")

for name in ["facility", "corridor", "room", "structure", "camera_main"] + [light["name"] for light in manifest["lights_bevy"]]:
    if name not in named:
        errors.append(f"missing node {name}")
camera = named.get("camera_main")
if camera:
    position = camera[1]((0.0, 0.0, 0.0))
    if not near(position, manifest["camera_bevy"]["position"], 1e-3):
        errors.append(f"camera_main at {position}, manifest says {manifest['camera_bevy']['position']}")
for light in manifest["lights_bevy"]:
    entry = named.get(light["name"])
    if entry and not near(entry[1]((0.0, 0.0, 0.0)), light["position"], 1e-3):
        errors.append(f"{light['name']} position differs from manifest")

for image in gltf.get("images", []):
    if "uri" in image or image.get("mimeType") != "image/jpeg":
        errors.append(f"image {image.get('name')} is not an embedded JPEG")
if gltf["asset"].get("version") != "2.0":
    errors.append("asset version is not 2.0")

if errors:
    sys.exit("\n".join(f"{path}: {e}" for e in errors))
print(f"{path}: ok, bevy bounds min {[round(v, 3) for v in lo]} max {[round(v, 3) for v in hi]}")
