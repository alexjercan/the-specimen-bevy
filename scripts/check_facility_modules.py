#!/usr/bin/env python3
import hashlib
import json
import math
import pathlib
import struct
import sys

root = pathlib.Path(__file__).resolve().parent.parent
default_dir = root / "art/visuals/generated/modules"

COMP_SIZES = {5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4}
COMP_FMT = {5120: "b", 5121: "B", 5122: "h", 5123: "H", 5125: "I", 5126: "f"}
TYPE_COUNTS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}


def sign(x):
    return 1.0 if x >= 0.0 else -1.0


def near_int(x, tol):
    return abs(x - round(x)) <= tol


def read_accessor(gltf, bin_data, index):
    acc = gltf["accessors"][index]
    bv = gltf["bufferViews"][acc["bufferView"]]
    n = TYPE_COUNTS[acc["type"]]
    csize = COMP_SIZES[acc["componentType"]]
    stride = bv.get("byteStride", n * csize)
    base = bv.get("byteOffset", 0) + acc.get("byteOffset", 0)
    fmt = "<" + COMP_FMT[acc["componentType"]] * n
    values = []
    for i in range(acc["count"]):
        off = base + i * stride
        values.append(struct.unpack_from(fmt, bin_data, off))
    return values


def parse_glb(data):
    if len(data) < 20:
        return None, None, ["file too small for a GLB header"]
    magic, version, length = struct.unpack_from("<4sII", data, 0)
    if magic != b"glTF":
        return None, None, ["bad GLB magic"]
    if version != 2:
        return None, None, [f"GLB version {version} is not 2"]
    if length != len(data):
        return None, None, [f"GLB header length {length} does not match file size {len(data)}"]
    json_len, json_type = struct.unpack_from("<I4s", data, 12)
    if json_type != b"JSON":
        return None, None, ["first chunk is not JSON"]
    try:
        gltf = json.loads(data[20:20 + json_len])
    except (UnicodeDecodeError, json.JSONDecodeError) as e:
        return None, None, [f"JSON chunk is not valid: {e}"]
    bin_off = 20 + json_len
    if bin_off + 8 > len(data):
        return gltf, None, ["missing BIN chunk"]
    bin_len, bin_type = struct.unpack_from("<I4s", data, bin_off)
    if bin_type != b"BIN\x00":
        return gltf, None, ["second chunk is not BIN"]
    return gltf, data[bin_off + 8:bin_off + 8 + bin_len], []


def check_structure(name, gltf):
    errors = []
    if gltf.get("asset", {}).get("version") != "2.0":
        errors.append("asset.version is not 2.0")
    if len(gltf.get("scenes", [])) != 1:
        errors.append(f"expected exactly one scene, found {len(gltf.get('scenes', []))}")
        return errors
    scene = gltf["scenes"][gltf.get("scene", 0)]
    if len(scene.get("nodes", [])) != 1:
        errors.append(f"expected exactly one root node, found {len(scene.get('nodes', []))}")
        return errors
    node = gltf["nodes"][scene["nodes"][0]]
    if node.get("name") != name:
        errors.append(f"root node named {node.get('name')!r}, expected {name!r}")
    if "mesh" not in node:
        errors.append("root node has no mesh")
    if "children" in node:
        errors.append("root node has children")
    if "matrix" in node:
        errors.append("root node uses a matrix transform")
    if node.get("translation", [0.0, 0.0, 0.0]) != [0.0, 0.0, 0.0]:
        errors.append(f"root node has a translation {node.get('translation')}")
    if node.get("rotation", [0.0, 0.0, 0.0, 1.0]) != [0.0, 0.0, 0.0, 1.0]:
        errors.append(f"root node has a rotation {node.get('rotation')}")
    if node.get("scale", [1.0, 1.0, 1.0]) != [1.0, 1.0, 1.0]:
        errors.append(f"root node has a scale {node.get('scale')}")
    return errors


def check_images_samplers(gltf):
    errors = []
    images = gltf.get("images", [])
    if len(images) > 3:
        errors.append(f"{len(images)} images, expected at most 3")
    for image in images:
        if "uri" in image or "bufferView" not in image:
            errors.append(f"image {image.get('name')} is not embedded")
        if image.get("mimeType") != "image/jpeg":
            errors.append(f"image {image.get('name')} mimeType is {image.get('mimeType')!r}, expected image/jpeg")
    for i, sampler in enumerate(gltf.get("samplers", [])):
        for key in ("wrapS", "wrapT"):
            if key in sampler and sampler[key] != 10497:
                errors.append(f"sampler {i} {key}={sampler[key]}, expected REPEAT (10497)")
    return errors


def mesh_of(gltf, name):
    scene = gltf["scenes"][gltf.get("scene", 0)]
    node = gltf["nodes"][scene["nodes"][0]]
    return gltf["meshes"][node["mesh"]]


def check_bounds(gltf, bin_data, entry, tol=1e-3, decode_tol=1e-4):
    errors = []
    mesh = mesh_of(gltf, entry["file"][:-len(".glb")])
    lo = [math.inf] * 3
    hi = [-math.inf] * 3
    for prim in mesh["primitives"]:
        acc = gltf["accessors"][prim["attributes"]["POSITION"]]
        amin, amax = acc.get("min"), acc.get("max")
        if amin is None or amax is None:
            errors.append("POSITION accessor is missing min/max")
            continue
        for i in range(3):
            lo[i] = min(lo[i], amin[i])
            hi[i] = max(hi[i], amax[i])
        if acc["componentType"] != 5126:
            errors.append(f"POSITION componentType {acc['componentType']} is not FLOAT")
            continue
        positions = read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])
        pmin = [min(p[i] for p in positions) for i in range(3)]
        pmax = [max(p[i] for p in positions) for i in range(3)]
        for i in range(3):
            if abs(pmin[i] - amin[i]) > decode_tol or abs(pmax[i] - amax[i]) > decode_tol:
                errors.append(f"decoded POSITION bounds {pmin}/{pmax} differ from accessor min/max {amin}/{amax}")
                break
    expected_lo = entry["bounds_bevy"]["min"]
    expected_hi = entry["bounds_bevy"]["max"]
    if any(abs(lo[i] - expected_lo[i]) > tol for i in range(3)) or any(abs(hi[i] - expected_hi[i]) > tol for i in range(3)):
        errors.append(f"bounds {lo}/{hi} differ from manifest bounds_bevy {expected_lo}/{expected_hi}")
    return errors, lo, hi


def check_snap(entry, grid, lo, hi, tol=1e-3):
    category = entry["category"]
    snap = entry["snap"]
    tile = grid["tile_m"]
    wall_h = grid["wall_height_m"]
    slab = grid["slab_m"]
    post = grid["post_m"]
    door_w, door_h = grid["door_opening_m"]
    errors = []

    def square(axis):
        if abs(lo[axis] + tile / 2) > tol:
            errors.append(f"axis {axis} min {lo[axis]} is not -tile/2")
        if abs(hi[axis] - tile / 2) > tol:
            errors.append(f"axis {axis} max {hi[axis]} is not +tile/2")

    if snap == "cell_center" and category == "floor":
        square(0)
        square(2)
        if abs(lo[1] + slab) > tol:
            errors.append(f"floor min y {lo[1]} is not -slab")
        if not (-tol <= hi[1] <= 0.005 + tol):
            errors.append(f"floor max y {hi[1]} is not in [0, 0.005]")
    elif snap == "cell_center" and category == "ceiling":
        square(0)
        square(2)
        if abs(hi[1] - (wall_h + slab)) > tol:
            errors.append(f"ceiling max y {hi[1]} is not wall_height+slab")
        if lo[1] < wall_h - 0.15 - tol:
            errors.append(f"ceiling min y {lo[1]} is below wall_height-0.15")
    elif snap == "cell_center" and category == "fixture":
        if abs(lo[0]) > tile / 2 + tol or abs(hi[0]) > tile / 2 + tol or abs(lo[2]) > tile / 2 + tol or abs(hi[2]) > tile / 2 + tol:
            errors.append(f"fixture footprint {lo}/{hi} is not inside +-tile/2")
        if lo[1] < wall_h - 0.3 - tol or hi[1] > wall_h + tol:
            errors.append(f"fixture y range {lo[1]}/{hi[1]} is not within [wall_height-0.3, wall_height]")
    elif snap == "edge_center":
        square(0)
        if abs(lo[1]) > tol or abs(hi[1] - wall_h) > tol:
            errors.append(f"edge_center y range {lo[1]}/{hi[1]} is not [0, wall_height]")
        if max(abs(lo[2]), abs(hi[2])) > post / 2 + 0.1 + tol:
            errors.append(f"edge_center |z| {max(abs(lo[2]), abs(hi[2]))} exceeds post/2+0.1")
    elif snap == "vertex":
        if abs(lo[0] + hi[0]) > tol or abs(lo[2] + hi[2]) > tol:
            errors.append(f"vertex footprint {lo}/{hi} is not centred")
        if abs(lo[1]) > tol or abs(hi[1] - wall_h) > tol:
            errors.append(f"vertex y range {lo[1]}/{hi[1]} is not [0, wall_height]")
        half = max(hi[0] - lo[0], hi[2] - lo[2]) / 2
        if half > post / 2 + 0.02 + tol:
            errors.append(f"vertex half width {half} exceeds post/2+0.02")
    elif snap == "door_hinge":
        if abs(lo[0]) > tol:
            errors.append(f"door_hinge min x {lo[0]} is not 0")
        if hi[0] > door_w - 0.01 + tol:
            errors.append(f"door_hinge max x {hi[0]} exceeds door width-0.01")
        if lo[1] < -tol:
            errors.append(f"door_hinge min y {lo[1]} is below 0")
        if hi[1] > door_h + tol:
            errors.append(f"door_hinge max y {hi[1]} exceeds door height")
        if max(abs(lo[2]), abs(hi[2])) > 0.1 + tol:
            errors.append(f"door_hinge |z| {max(abs(lo[2]), abs(hi[2]))} exceeds 0.1")
    elif snap == "wall_mount":
        if hi[2] > 1e-3 + tol:
            errors.append(f"wall_mount max z {hi[2]} is not <= 0")
        if -lo[2] > 0.3 + tol:
            errors.append(f"wall_mount depth {-lo[2]} exceeds 0.3")
    elif snap == "floor":
        if abs(lo[1]) > tol:
            errors.append(f"floor-snap min y {lo[1]} is not 0")
        if hi[1] > wall_h + tol:
            errors.append(f"floor-snap max y {hi[1]} exceeds wall_height")
    elif snap == "floor_line":
        if abs(lo[0] + 0.5) > tol or abs(hi[0] - 0.5) > tol:
            errors.append(f"floor_line x range {lo[0]}/{hi[0]} is not [-0.5,0.5]")
        if abs(lo[1]) > tol or abs(hi[1] - 0.003) > tol:
            errors.append(f"floor_line y range {lo[1]}/{hi[1]} is not [0,0.003]")
        if abs(lo[2] + 0.05) > tol or abs(hi[2] - 0.05) > tol:
            errors.append(f"floor_line z range {lo[2]}/{hi[2]} is not [-0.05,0.05]")
        if entry.get("walkable") is not True:
            errors.append("floor_line module is not marked walkable")
    elif snap == "ceiling_hang":
        if abs(hi[1] - wall_h) > tol:
            errors.append(f"ceiling_hang max y {hi[1]} is not wall_height")
        if lo[1] < 2.1 - tol:
            errors.append(f"ceiling_hang min y {lo[1]} is below 2.1")
        if max(abs(lo[2]), abs(hi[2])) > 0.05 + tol:
            errors.append(f"ceiling_hang |z| {max(abs(lo[2]), abs(hi[2]))} exceeds 0.05")
        if max(abs(lo[0]), abs(hi[0])) > tile / 2 + tol:
            errors.append(f"ceiling_hang |x| {max(abs(lo[0]), abs(hi[0]))} exceeds tile/2")
    else:
        errors.append(f"no snap contract implemented for snap={snap!r} category={category!r}")
    return errors


def check_light_anchor(entry, grid, lo, hi, tol=1e-3):
    ax, ay, az = entry["light_anchor_bevy"]
    errors = []
    if not (lo[0] - 0.3 - tol <= ax <= hi[0] + 0.3 + tol):
        errors.append(f"light anchor x {ax} is outside module footprint +-0.3")
    if not (lo[2] - 0.3 - tol <= az <= hi[2] + 0.3 + tol):
        errors.append(f"light anchor z {az} is outside module footprint +-0.3")
    if ay > grid["wall_height_m"] + tol:
        errors.append(f"light anchor y {ay} is not below the ceiling")
    return errors


def check_door_clearance(name, gltf, bin_data, grid, tol=1e-3):
    errors = []
    door_w, door_h = grid["door_opening_m"]
    mesh = mesh_of(gltf, name)
    if name == "wall_doorway":
        ox0, ox1 = -(door_w / 2 - 0.005), door_w / 2 - 0.005
        oy0, oy1 = 0.005, door_h - 0.005
        for prim in mesh["primitives"]:
            positions = read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])
            indices = read_accessor(gltf, bin_data, prim["indices"])
            flat = [i[0] for i in indices]
            for t in range(0, len(flat) - 2, 3):
                tri = [positions[flat[t]], positions[flat[t + 1]], positions[flat[t + 2]]]
                tx0 = min(v[0] for v in tri)
                tx1 = max(v[0] for v in tri)
                ty0 = min(v[1] for v in tri)
                ty1 = max(v[1] for v in tri)
                if tx0 < ox1 and ox0 < tx1 and ty0 < oy1 and oy0 < ty1:
                    errors.append(f"triangle at x[{tx0:.3f},{tx1:.3f}] y[{ty0:.3f},{ty1:.3f}] overlaps the door opening")
    elif name == "door_panel":
        lo = [min(min(p[i] for p in read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])) for prim in mesh["primitives"]) for i in range(3)]
        hi = [max(max(p[i] for p in read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])) for prim in mesh["primitives"]) for i in range(3)]
        if hi[0] - lo[0] > door_w - 0.01 + tol:
            errors.append(f"door_panel x extent {hi[0] - lo[0]} exceeds door_width-0.01")
        if hi[1] - lo[1] > door_h + tol:
            errors.append(f"door_panel y extent {hi[1] - lo[1]} exceeds door_height")
    return errors


def check_texture_continuity(gltf, bin_data, grid, uv_tol=1e-3):
    errors = []
    period = grid["uv_period_m"]
    tile = grid["tile_m"]
    if abs(tile / period - round(tile / period)) > 1e-6:
        errors.append(f"uv_period_m {period} does not divide tile_m {tile}")
    checked = 0
    for mesh in gltf.get("meshes", []):
        for prim in mesh["primitives"]:
            material = gltf["materials"][prim["material"]]
            bct = material.get("pbrMetallicRoughness", {}).get("baseColorTexture")
            if not bct:
                continue
            if bct.get("texCoord", 0) != 0:
                errors.append(f"material {material.get('name')} baseColorTexture uses texCoord {bct.get('texCoord')}, expected 0")
                continue
            positions = read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])
            normals = read_accessor(gltf, bin_data, prim["attributes"]["NORMAL"])
            uvs = read_accessor(gltf, bin_data, prim["attributes"]["TEXCOORD_0"])
            for p, n, uv in zip(positions, normals, uvs):
                axis = max(range(3), key=lambda i: abs(n[i]))
                if abs(n[axis]) < 0.99:
                    continue
                if axis == 0:
                    eu, ev = -p[2] * sign(n[0]), p[1]
                elif axis == 1:
                    eu, ev = p[0], -p[2] * sign(n[1])
                else:
                    eu, ev = p[0] * sign(n[2]), p[1]
                eu /= period
                ev = 1.0 - ev / period
                checked += 1
                if not near_int(uv[0] - eu, uv_tol) or not near_int(uv[1] - ev, uv_tol):
                    errors.append(f"texture UV {uv} does not match box projection ({eu}, {ev}) for material {material.get('name')}")
    return errors, checked


def check_concept_note(entry):
    errors = []
    if entry.get("category") == "concept":
        note = entry.get("concept")
        if not note or not isinstance(note, str) or not note.strip():
            errors.append("concept module is missing a non-empty 'concept' note")
        elif "collision" not in note or "interaction" not in note:
            errors.append(f"concept note {note!r} does not mention both 'collision' and 'interaction'")
    return errors


def check_walkable(entry, hi, tol=1e-3):
    errors = []
    if entry.get("walkable"):
        if hi[1] > 0.03 + tol:
            errors.append(f"walkable module max y {hi[1]} exceeds 0.03")
        if entry.get("category") not in ("prop", "decoration", "route"):
            errors.append(f"walkable module category {entry.get('category')!r} is not 'prop', 'decoration', or 'route'")
    return errors


def check_sign_label_fields(name, entry):
    errors = []
    if name.startswith("sign_label_"):
        if not entry.get("text"):
            errors.append("sign_label module is missing 'text'")
        if entry.get("text_cap_m", 0.0) < 0.1 - 1e-9:
            errors.append(f"sign_label text_cap_m {entry.get('text_cap_m')} is below 0.1")
    return errors


def check_sign_labels_consistent(modules):
    errors = []
    bounds = [m["bounds_bevy"] for n, m in modules.items() if n.startswith("sign_label_")]
    if len(bounds) > 1 and any(b != bounds[0] for b in bounds[1:]):
        errors.append(f"sign_label_* modules do not share the same bounds_bevy: {bounds}")
    return errors


def check_sign_arrow(gltf, bin_data, name):
    errors = []
    mesh = mesh_of(gltf, name)
    max_pos, max_neg = -math.inf, -math.inf
    for prim in mesh["primitives"]:
        material = gltf["materials"][prim["material"]]
        if material.get("name") != "sign_text":
            continue
        for p in read_accessor(gltf, bin_data, prim["attributes"]["POSITION"]):
            if p[0] > 0:
                max_pos = max(max_pos, abs(p[1]))
            elif p[0] < 0:
                max_neg = max(max_neg, abs(p[1]))
    if not (max_pos > max_neg):
        errors.append(f"sign_arrow max|y| at x>0 ({max_pos}) is not greater than at x<0 ({max_neg}); head should be at +X")
    return errors


def check_clearance_box(gltf, bin_data, name, clearance, tol=1e-3):
    errors = []
    cmin, cmax = clearance["min"], clearance["max"]
    mesh = mesh_of(gltf, name)
    for prim in mesh["primitives"]:
        positions = read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])
        indices = [i[0] for i in read_accessor(gltf, bin_data, prim["indices"])]
        for t in range(0, len(indices) - 2, 3):
            tri = [positions[indices[t + k]] for k in range(3)]
            tlo = [min(v[i] for v in tri) for i in range(3)]
            thi = [max(v[i] for v in tri) for i in range(3)]
            if all(tlo[i] < cmax[i] - tol and thi[i] > cmin[i] + tol for i in range(3)):
                errors.append(f"triangle at {tlo}/{thi} overlaps the clearance box {cmin}/{cmax}")
    return errors


def check_crawl_vent_opening(gltf, bin_data, name, opening, tol=1e-3):
    errors = []
    omin, omax = opening["min"], opening["max"]
    mesh = mesh_of(gltf, name)
    for prim in mesh["primitives"]:
        material = gltf["materials"][prim["material"]]
        if material.get("name") == "void_black":
            continue
        positions = read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])
        indices = [i[0] for i in read_accessor(gltf, bin_data, prim["indices"])]
        for t in range(0, len(indices) - 2, 3):
            tri = [positions[indices[t + k]] for k in range(3)]
            tx0 = min(v[0] for v in tri)
            tx1 = max(v[0] for v in tri)
            ty0 = min(v[1] for v in tri)
            ty1 = max(v[1] for v in tri)
            tz0 = min(v[2] for v in tri)
            if tz0 < -0.01 - tol and tx0 < omax[0] - tol and omin[0] < tx1 - tol and ty0 < omax[1] - tol and omin[1] < ty1 - tol:
                errors.append(f"triangle at x[{tx0:.3f},{tx1:.3f}] y[{ty0:.3f},{ty1:.3f}] blocks the crawl vent opening")
    return errors


def check_breach(gltf, bin_data, name, breach, tol=1e-3):
    errors = []
    bmin, bmax = breach["min"], breach["max"]
    for prim in mesh_of(gltf, name)["primitives"]:
        positions = read_accessor(gltf, bin_data, prim["attributes"]["POSITION"])
        indices = [i[0] for i in read_accessor(gltf, bin_data, prim["indices"])]
        for t in range(0, len(indices) - 2, 3):
            tri = [positions[indices[t + k]] for k in range(3)]
            lo = [min(v[a] for v in tri) for a in range(3)]
            hi = [max(v[a] for v in tri) for a in range(3)]
            if all(lo[a] < bmax[a] - tol and bmin[a] < hi[a] - tol for a in range(3)):
                errors.append(f"triangle at {lo}..{hi} closes the breach")
                break
    return errors


def check_manifest_sanity(name, entry, gltf, snaps):
    errors = []
    if "category" not in entry:
        errors.append("manifest entry is missing category")
    if entry.get("snap") not in snaps:
        errors.append(f"snap {entry.get('snap')!r} is not a key of manifest snaps")
    mesh = mesh_of(gltf, name)
    total = sum(gltf["accessors"][prim["indices"]]["count"] // 3 for prim in mesh["primitives"])
    if entry.get("triangles") != total:
        errors.append(f"manifest triangles {entry.get('triangles')} != decoded {total}")
    return errors


def check_module(name, entry, modules_dir, grid, snaps):
    path = modules_dir / entry["file"]
    if not path.is_file():
        return [f"{entry['file']}: missing on disk"], 0
    data = path.read_bytes()
    errors = []
    if entry.get("glb_bytes") != len(data):
        errors.append(f"glb_bytes {entry.get('glb_bytes')} != actual {len(data)}")
    digest = hashlib.sha256(data).hexdigest()
    if entry.get("glb_sha256") != digest:
        errors.append(f"glb_sha256 mismatch: manifest {entry.get('glb_sha256')} actual {digest}")

    gltf, bin_data, parse_errors = parse_glb(data)
    errors.extend(parse_errors)
    if gltf is None or bin_data is None:
        return [f"{entry['file']}: {e}" for e in errors], 0

    checked = 0
    try:
        errors.extend(check_structure(name, gltf))
        errors.extend(check_images_samplers(gltf))
        bounds_errors, lo, hi = check_bounds(gltf, bin_data, entry)
        errors.extend(bounds_errors)
        if math.isfinite(lo[0]):
            errors.extend(check_snap(entry, grid, lo, hi))
            if "light_anchor_bevy" in entry:
                errors.extend(check_light_anchor(entry, grid, lo, hi))
            errors.extend(check_walkable(entry, hi))
        errors.extend(check_door_clearance(name, gltf, bin_data, grid))
        tex_errors, checked = check_texture_continuity(gltf, bin_data, grid)
        errors.extend(tex_errors)
        errors.extend(check_manifest_sanity(name, entry, gltf, snaps))
        errors.extend(check_concept_note(entry))
        errors.extend(check_sign_label_fields(name, entry))
        if name == "sign_arrow":
            errors.extend(check_sign_arrow(gltf, bin_data, name))
        if "clearance_bevy" in entry:
            errors.extend(check_clearance_box(gltf, bin_data, name, entry["clearance_bevy"]))
        if "opening_bevy" in entry:
            errors.extend(check_crawl_vent_opening(gltf, bin_data, name, entry["opening_bevy"]))
        if "breach_bevy" in entry:
            errors.extend(check_breach(gltf, bin_data, name, entry["breach_bevy"]))
    except (KeyError, IndexError, struct.error) as e:
        errors.append(f"malformed glTF content: {type(e).__name__}: {e}")
    return [f"{entry['file']}: {e}" for e in errors], checked


def main():
    modules_dir = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else default_dir
    manifest_path = modules_dir / "modules.manifest.json"
    if not manifest_path.is_file():
        sys.exit(f"{manifest_path}: not found")
    manifest = json.loads(manifest_path.read_text())
    grid = manifest["grid"]
    snaps = manifest["snaps"]
    modules = manifest["modules"]

    errors = []
    disk_names = {p.stem for p in modules_dir.glob("*.glb")}
    manifest_names = set(modules.keys())
    for extra in sorted(disk_names - manifest_names):
        errors.append(f"{extra}.glb: present on disk but not in manifest")
    for missing in sorted(manifest_names - disk_names):
        errors.append(f"{missing}.glb: listed in manifest but missing on disk")

    errors.extend(check_sign_labels_consistent(modules))

    oks = []
    for name in sorted(modules):
        entry = modules[name]
        module_errors, checked = check_module(name, entry, modules_dir, grid, snaps)
        if module_errors:
            errors.extend(module_errors)
        else:
            oks.append((entry["file"], entry["triangles"], checked))

    for file, tris, checked in oks:
        suffix = f", {checked} textured vertices checked" if checked else ""
        print(f"{file}: ok, {tris} triangles{suffix}")

    if errors:
        print("\n".join(errors), file=sys.stderr)
        sys.exit(1)
    print(f"ok: {len(oks)} modules checked in {modules_dir}")


main()
