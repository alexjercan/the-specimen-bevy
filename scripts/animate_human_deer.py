import argparse
import hashlib
import json
import math
import pathlib
import shutil
import struct
import subprocess
import sys
import tempfile
import zipfile

import bpy
from bpy_extras import anim_utils
from mathutils import Matrix, Quaternion, Vector

FPS = 24
OUTPUT_NAME = "human_deer_animated"
SOURCE_CLIP = "Take 001"
IDLE = "IDLE"

X = Vector((1.0, 0.0, 0.0))
Y = Vector((0.0, 1.0, 0.0))
Z = Vector((0.0, 0.0, 1.0))

HIP = "C_Hip_J_00"
SPINE = ("C_spine_01_J_018", "C_spine_02_J_019", "C_spine_03_J_020", "C_spine_04_J_021")
NECK = "C_Neck_J_022"
HEAD = "C_Head_J_023"
JAW = "C_jaw__J_026"
TONGUE = tuple(f"C_Tongue_0{i}_J_0{27 + i}" for i in range(1, 7))
TAIL = (
    "C_base_tail_J_01",
    "C_tail_01_J_02",
    "C_tail_02_J_03",
    *(f"C_Bone_tail_{i:02d}_J_0{i + 3}" for i in range(1, 15)),
)

LEGS = {
    "L": ("L_upper_leg_J_072", "L_knee_J_073", "L_ankle_J_074", "L_feet_J_076", "L_toe_J_077"),
    "R": ("R_upper_leg_J_078", "R_knee_J_079", "R_ankle_J_080", "R_feet_J_082", "R_toe_J_083"),
}
ARM_BASE = {"L": 34, "R": 53}
FINGERS = ("thumb", "index", "middle", "ring", "pinkie")

FOLLOWERS = {
    "R_leg_exposed_bone_low": "R_upper_leg_J_078",
    "pPlane19": HIP,
}

WALK = {
    "frames": 36,
    "duty": 0.6,
    "stride": 1.1,
    "lift": 0.32,
}
CHASE = {
    "frames": 16,
    "duty": 0.38,
    "stride": 1.9,
    "hind_lift": 0.55,
    "fore_lift": 0.6,
    "crouch": 0.55,
    "finger_clearance": 0.05,
    "touchdown": {"hind_L": 0.0, "hind_R": 0.08, "fore_L": 0.42, "fore_R": 0.5},
}
EXTRA_NODES = {
    IDLE: ("C_Hip_ctrl", "R_leg_exposed_bone_low"),
    **{name: tuple(FOLLOWERS) for name in ("WALK", "CHASE", "ATTACK")},
}
IDLE_TOLERANCE = {"rotation_deg": 1.0, "translation_m": 0.005, "scale": 0.001}
ATTACK = {
    "frames": 40,
    "hit_frame": 19,
}
CLIP_FRAMES = {"WALK": WALK["frames"], "CHASE": CHASE["frames"], "ATTACK": ATTACK["frames"]}


def arm_bones(side):
    n = ARM_BASE[side]
    names = [f"{side}_shoulder_J_0{n}", f"{side}_elbow_J_0{n + 1}", f"{side}_wrist_J_0{n + 2}", f"{side}_hand_J_0{n + 3}"]
    fingers = {}
    for i, finger in enumerate(FINGERS):
        fingers[finger] = [f"{side}_{finger}_0{k + 1}_J_0{n + 4 + i * 3 + k}" for k in range(3)]
    return names, fingers


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True)
    parser.add_argument("--out", default=str(pathlib.Path(__file__).resolve().parent.parent / "art/visuals/generated/monster"))
    parser.add_argument("--previews", action="store_true")
    return parser.parse_args(argv)


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def smooth(t):
    t = min(max(t, 0.0), 1.0)
    return t * t * (3.0 - 2.0 * t)


def ramp(t, a, b):
    return smooth((t - a) / (b - a))


def wave(p, offset=0.0, harmonic=1.0):
    return math.sin(2.0 * math.pi * (harmonic * p - offset))


def rot(axis, degrees):
    return Quaternion(axis, math.radians(degrees))


def reset_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = FPS


def import_model(path):
    bpy.ops.import_scene.gltf(filepath=str(path))
    for obj in list(bpy.data.objects):
        if obj.type == "MESH" and obj.name.startswith("Icosphere") and obj.parent is None and not obj.data.materials:
            bpy.data.objects.remove(obj)
    for obj in bpy.data.objects:
        if obj.animation_data:
            for track in list(obj.animation_data.nla_tracks):
                obj.animation_data.nla_tracks.remove(track)
    armatures = [o for o in bpy.data.objects if o.type == "ARMATURE"]
    if len(armatures) != 1:
        raise SystemExit(f"expected one armature, found {len(armatures)}")
    return armatures[0]


class Rig:
    def __init__(self, arm):
        self.arm = arm
        scale = arm.matrix_world.to_scale()
        if max(scale) - min(scale) > 1e-5 * max(scale):
            raise SystemExit(f"armature scale {tuple(scale)} is not uniform")
        self.unit = 1.0 / scale.x
        bones = arm.data.bones
        self.order = []
        stack = [b for b in bones if b.parent is None]
        while stack:
            bone = stack.pop(0)
            self.order.append(bone.name)
            stack.extend(bone.children)
        self.parent = {b.name: (b.parent.name if b.parent else None) for b in bones}
        rest = {b.name: b.matrix_local.copy() for b in bones}
        self.offset = {
            n: (rest[p].inverted() @ rest[n]) if p else rest[n] for n, p in self.parent.items()
        }
        self.base = {pb.name: pb.matrix_basis.copy() for pb in arm.pose.bones}
        self.basis = {}
        self.cache = None
        self.reset()
        self.base_pose = {n: m.copy() for n, m in self.pose().items()}

    def reset(self):
        self.basis = {n: m.copy() for n, m in self.base.items()}
        self.cache = None

    def pose(self):
        if self.cache is None:
            mats = {}
            for n in self.order:
                p = self.parent[n]
                mats[n] = ((mats[p] @ self.offset[n]) if p else self.offset[n]) @ self.basis[n]
            self.cache = mats
        return self.cache

    def head(self, n):
        return self.pose()[n].translation.copy()

    def base_head(self, n):
        return self.base_pose[n].translation.copy()

    def world(self, v):
        return v / self.unit

    def local(self, v):
        return v * self.unit

    def apply(self, n, transform):
        mats = self.pose()
        p = self.parent[n]
        holder = (mats[p] @ self.offset[n]) if p else self.offset[n]
        self.basis[n] = holder.inverted() @ transform @ mats[n]
        self.cache = None

    def rotate(self, n, q, pivot=None):
        pivot = self.head(n) if pivot is None else pivot
        self.apply(n, Matrix.Translation(pivot) @ q.to_matrix().to_4x4() @ Matrix.Translation(-pivot))

    def translate(self, n, meters):
        self.apply(n, Matrix.Translation(self.local(meters)))

    def delta(self, n):
        return self.pose()[n].to_quaternion() @ self.base_pose[n].to_quaternion().inverted()

    def axis(self, n, v):
        p = self.parent[n]
        return (self.delta(p) @ v) if p else v.copy()

    def turn(self, n, base_axis, degrees):
        if abs(degrees) > 1e-6:
            self.rotate(n, rot(self.axis(n, base_axis), degrees))

    def aim(self, n, current, wanted, pivot=None):
        if current.length > 1e-9 and wanted.length > 1e-9:
            self.rotate(n, current.rotation_difference(wanted), pivot)

    def bend_normal(self, upper, lower, end):
        h, k, a = self.base_head(upper), self.base_head(lower), self.base_head(end)
        return (a - h).cross(k - h).normalized()

    def two_bone(self, upper, lower, end, target, normal):
        h, k, a = self.head(upper), self.head(lower), self.head(end)
        l1, l2 = (k - h).length, (a - k).length
        reach = target - h
        dist = min(max(reach.length, abs(l1 - l2) * 1.001), (l1 + l2) * 0.999)
        u = reach.normalized()
        along = (l1 * l1 - l2 * l2 + dist * dist) / (2.0 * dist)
        side = math.sqrt(max(l1 * l1 - along * along, 0.0))
        w = normal.cross(u)
        w = (w - u * w.dot(u)).normalized()
        knee = h + u * along + w * side
        self.aim(upper, k - h, knee - h)
        k2, a2 = self.head(lower), self.head(end)
        self.aim(lower, a2 - k2, h + u * dist - k2)
        return abs(reach.length - dist) / self.unit


class Creature:
    def __init__(self, rig):
        self.rig = rig
        self.ground = None
        self.leg_normal = {s: rig.bend_normal(b[0], b[1], b[2]) for s, b in LEGS.items()}
        self.arms = {s: arm_bones(s) for s in ("L", "R")}
        self.arm_normal = {s: rig.bend_normal(a[0][0], a[0][1], a[0][2]) for s, a in self.arms.items()}
        self.reach_error = 0.0
        self.clearance = (math.inf, None, None)

    def pelvis(self, offset, pitch=0.0, roll=0.0, yaw=0.0):
        r = self.rig
        r.translate(HIP, offset)
        r.turn(HIP, Z, yaw)
        r.turn(HIP, X, pitch)
        r.turn(HIP, Y, roll)

    def chain(self, names, base_axis, degrees):
        for n, d in zip(names, degrees):
            self.rig.turn(n, base_axis, d)

    def tail(self, p, yaw_amp, pitch_amp, lag, raise_deg=0.0, harmonic=1.0):
        for i, n in enumerate(TAIL):
            weight = 0.4 + 0.6 * min(i / 6.0, 1.0)
            self.rig.turn(n, Z, yaw_amp * weight * wave(p, lag * i, harmonic))
            self.rig.turn(n, X, raise_deg * (1.0 if i < 5 else 0.0) + pitch_amp * weight * wave(p, lag * i + 0.25, harmonic))

    def tongue(self, p, amp, harmonic=1.0):
        for i, n in enumerate(TONGUE):
            self.rig.turn(n, X, amp * wave(p, 0.12 * i, harmonic))

    def leg(self, side, offset, meta_pitch=0.0, foot_pitch=0.0):
        r = self.rig
        upper, knee, ankle, feet, toe = LEGS[side]
        base_dir = r.base_head(toe) - r.base_head(feet)
        foot_dir = rot(X, foot_pitch) @ base_dir
        foot_target = r.base_head(feet) + r.local(offset) + Z * max(0.0, base_dir.z - foot_dir.z)
        meta = rot(X, meta_pitch) @ (r.base_head(feet) - r.base_head(ankle))
        hock = foot_target - meta
        normal = r.delta(HIP) @ self.leg_normal[side]
        self.reach_error = max(self.reach_error, r.two_bone(upper, knee, ankle, hock, normal))
        r.aim(ankle, r.head(feet) - r.head(ankle), foot_target - r.head(ankle))
        r.aim(feet, r.head(toe) - r.head(feet), foot_dir)

    def arm_swing(self, side, flex, elbow, abduct=0.0, wrist=0.0):
        r = self.rig
        (shoulder, elbow_bone, _wrist, hand), _ = self.arms[side]
        r.turn(shoulder, Y, abduct if side == "L" else -abduct)
        r.turn(shoulder, X, flex)
        r.turn(elbow_bone, X, elbow)
        r.turn(hand, X, wrist)

    def arm_reach(self, side, wrist_target, hand_dir):
        r = self.rig
        (shoulder, elbow_bone, wrist, hand), fingers = self.arms[side]
        normal = r.delta(SPINE[-1]) @ self.arm_normal[side]
        self.reach_error = max(self.reach_error, r.two_bone(shoulder, elbow_bone, wrist, wrist_target, normal))
        r.aim(hand, r.head(fingers["middle"][0]) - r.head(hand), hand_dir)

    def plant_hand(self, side, target, hand_dir, curl, margin):
        r = self.rig
        bones, fingers = self.arms[side]
        chain = [*bones, *(n for names in fingers.values() for n in names)]
        saved = {n: r.basis[n].copy() for n in chain}
        target = target.copy()
        for _ in range(3):
            for n, m in saved.items():
                r.basis[n] = m.copy()
            r.cache = None
            self.arm_reach(side, r.local(target), hand_dir)
            self.curl(side, curl)
            low = min(r.world(r.head(n)).z for names in fingers.values() for n in names) - self.ground
            if low >= margin - 1e-4:
                break
            target.z += margin - low

    def palm(self, side):
        r = self.rig
        (_, _, _, hand), fingers = self.arms[side]
        h = r.head(hand)
        along = (r.head(fingers["middle"][0]) - h).normalized()
        across = (r.head(fingers["index"][0]) - r.head(fingers["pinkie"][0])).normalized()
        thumb = r.head(fingers["thumb"][0]) - h
        normal = thumb - along * thumb.dot(along) - across * thumb.dot(across)
        return along, normal.normalized()

    def curl(self, side, degrees):
        r = self.rig
        _, fingers = self.arms[side]
        along, normal = self.palm(side)
        axis = along.cross(normal).normalized()
        for finger, names in fingers.items():
            amount = degrees * (0.5 if finger == "thumb" else 1.0)
            for k, n in enumerate(names):
                r.rotate(n, Quaternion(axis, math.radians(amount * (0.8 + 0.2 * k))))


def walk_pose(c, f):
    r = c.rig
    n = WALK["frames"]
    p = f / n
    duty, stride, lift = WALK["duty"], WALK["stride"], WALK["lift"]
    bob = 0.05 * math.cos(4.0 * math.pi * (p - 0.3))
    sway = -0.06 * math.sin(2.0 * math.pi * (p - 0.05))
    c.pelvis(Vector((sway, 0.0, bob - 0.04)), pitch=1.5 * math.cos(4.0 * math.pi * (p - 0.1)), roll=-3.0 * math.sin(2.0 * math.pi * (p - 0.05)), yaw=-4.0 * math.cos(2.0 * math.pi * p))
    c.chain(SPINE, Z, [1.0 * math.cos(2.0 * math.pi * p), 1.5 * math.cos(2.0 * math.pi * p), 2.0 * math.cos(2.0 * math.pi * p), 2.0 * math.cos(2.0 * math.pi * p)])
    c.chain(SPINE, X, [-0.6 * math.cos(4.0 * math.pi * (p - 0.15))] * 4)
    r.turn(NECK, X, 2.0 * math.cos(4.0 * math.pi * (p - 0.35)) - 1.5 * math.cos(4.0 * math.pi * (p - 0.1)))
    r.turn(NECK, Z, 3.0 * math.sin(2.0 * math.pi * p))
    r.turn(HEAD, X, 1.5 * math.cos(4.0 * math.pi * (p - 0.4)))
    r.turn(JAW, X, -4.0 - 3.0 * (0.5 + 0.5 * wave(p, 0.0, 2.0)))
    c.tongue(p, 4.0, 2.0)
    c.tail(p, 4.0, 1.5, 0.07)
    for side, phase in (("L", 0.0), ("R", 0.5)):
        q = (p + phase) % 1.0
        if q < duty:
            s = q / duty
            c.leg(side, Vector((0.0, stride * (0.5 - s), 0.0)))
        else:
            s = (q - duty) / (1.0 - duty)
            e = smooth(s)
            arc = math.sin(math.pi * s)
            c.leg(side, Vector((0.0, stride * (e - 0.5), lift * arc)), meta_pitch=18.0 * arc, foot_pitch=-28.0 * arc + 6.0 * math.sin(2.0 * math.pi * s))
    for side, phase in (("L", 0.5), ("R", 0.0)):
        swing = math.cos(2.0 * math.pi * (p - phase - 0.06))
        c.arm_swing(side, 16.0 * swing, 6.0 + 6.0 * swing, abduct=1.5, wrist=-6.0 * math.cos(2.0 * math.pi * (p - phase - 0.16)))
        c.curl(side, 6.0 + 4.0 * swing)


def chase_posture(c, p):
    flex = math.sin(2.0 * math.pi * (p - 0.2))
    lift = 0.12 * math.cos(2.0 * math.pi * (p - 0.9)) + 0.04 * math.cos(4.0 * math.pi * (p - 0.2))
    c.pelvis(Vector((0.0, 0.0, -CHASE["crouch"] + lift)), pitch=-10.0 + 6.0 * flex, roll=1.5 * wave(p, 0.1), yaw=2.0 * wave(p, 0.05))
    c.chain(SPINE, X, [-3.5 - 1.5 * flex] * 4)
    c.chain(SPINE, Z, [-1.0 * wave(p, 0.05)] * 4)
    return flex


def chase_plants(c):
    r = c.rig
    saved = dict(r.basis)
    chase_posture(c, 0.25)
    plants = {}
    for side in ("L", "R"):
        (shoulder, _, _, _), _ = c.arms[side]
        s = r.world(r.head(shoulder))
        plants[side] = Vector((s.x * 1.2, s.y + 0.45, c.ground + 0.42))
    r.basis = saved
    r.cache = None
    return plants


def chase_pose(c, f, plants):
    r = c.rig
    n = CHASE["frames"]
    p = f / n
    duty, stride = CHASE["duty"], CHASE["stride"]
    flex = chase_posture(c, p)
    r.turn(NECK, X, 17.0 - 3.0 * flex)
    r.turn(HEAD, X, 8.0 - 3.0 * flex)
    r.turn(NECK, Z, 1.0 * wave(p, 0.05))
    r.turn(JAW, X, -16.0 - 8.0 * (0.5 + 0.5 * wave(p, 0.6)))
    c.tongue(p, 9.0, 1.0)
    c.tail(p, 5.0, 4.0, 0.06, raise_deg=3.0)
    for side in ("L", "R"):
        q = (p - CHASE["touchdown"][f"hind_{side}"]) % 1.0
        if q < duty:
            s = q / duty
            c.leg(side, Vector((0.0, stride * (0.5 - s), 0.0)), foot_pitch=-12.0 * smooth((s - 0.7) / 0.3))
        else:
            s = (q - duty) / (1.0 - duty)
            arc = math.sin(math.pi * s)
            c.leg(side, Vector((0.0, stride * (smooth(s) - 0.5), CHASE["hind_lift"] * arc)), meta_pitch=30.0 * arc, foot_pitch=-12.0 - 26.0 * arc + 12.0 * smooth(s))
    for side in ("L", "R"):
        q = (p - CHASE["touchdown"][f"fore_{side}"]) % 1.0
        plant = plants[side]
        if q < duty:
            s = q / duty
            target = plant + Vector((0.0, stride * (0.5 - s), 0.0))
            pitch = -40.0 - 25.0 * smooth((s - 0.6) / 0.4)
            curl = 18.0
        else:
            s = (q - duty) / (1.0 - duty)
            arc = math.sin(math.pi * s)
            target = plant + Vector((0.0, stride * (smooth(s) - 0.5), CHASE["fore_lift"] * arc))
            pitch = -65.0 - 45.0 * arc + 25.0 * smooth(s)
            curl = 18.0 + 22.0 * arc
        c.plant_hand(side, target, rot(X, pitch) @ Y, curl, CHASE["finger_clearance"])


def attack_curves(f):
    rear = ramp(f, 0.0, 12.0) * (1.0 - ramp(f, 14.0, 19.0))
    lunge = ramp(f, 15.0, 19.0) * (1.0 - ramp(f, 25.0, 40.0))
    return rear, lunge


def attack_pose(c, f):
    r = c.rig
    rear, lunge = attack_curves(f)
    bite = ramp(f, 16.0, 20.0) * (1.0 - ramp(f, 22.0, 30.0))
    c.pelvis(Vector((0.0, -0.18 * rear + 0.42 * lunge, -0.1 * rear - 0.18 * lunge)), pitch=9.0 * rear - 8.0 * lunge, yaw=4.0 * lunge)
    c.chain(SPINE, X, [5.0 * rear - 4.0 * lunge] * 4)
    c.chain(SPINE, Z, [-1.5 * lunge] * 4)
    r.turn(NECK, X, 9.0 * rear - 4.0 * lunge)
    r.turn(HEAD, X, 10.0 * rear - 8.0 * lunge)
    r.turn(JAW, X, -32.0 * rear - 14.0 * ramp(f, 14.0, 17.0) * (1.0 - bite) * (1.0 - ramp(f, 25.0, 34.0)) + 1.0 * bite)
    c.tongue(f / ATTACK["frames"], 10.0 * rear + 4.0, 3.0)
    whip = math.sin(math.pi * ramp(f, 15.0, 30.0))
    for i, n in enumerate(TAIL):
        lag = ramp(f, 15.0 + i * 0.6, 22.0 + i * 0.6) * (1.0 - ramp(f, 26.0 + i * 0.5, 40.0))
        r.turn(n, X, (4.0 if i < 6 else 1.0) * rear - 2.0 * lunge)
        r.turn(n, Z, 5.0 * lag * (1.0 if i < 8 else 0.5) - 2.0 * whip * (1.0 if i >= 8 else 0.0))
    for side, shift in (("R", 0.0), ("L", 2.0)):
        a, b = attack_curves(f - shift * (1.0 - ramp(f, 22.0, 32.0)))
        c.arm_swing(side, 120.0 * a + 62.0 * b, 50.0 * a + 8.0 * b, abduct=32.0 * a - 4.0 * b, wrist=25.0 * a - 15.0 * b)
        c.curl(side, -14.0 * a + 38.0 * b)
    for side in ("L", "R"):
        c.leg(side, Vector((0.0, 0.0, 0.0)))


def follower_world(rig, arm, bone, base_world):
    current = arm.matrix_world @ rig.pose()[bone]
    base = arm.matrix_world @ rig.base_pose[bone]
    return current @ base.inverted() @ base_world


def continuous(quats):
    out = []
    for q in quats:
        q = q.copy()
        if out and out[-1].dot(q) < 0.0:
            q.negate()
        out.append(q)
    return out


def write_curves(bag, data_path, group, frames, values, width):
    for i in range(width):
        curve = bag.fcurves.new(data_path, index=i, group_name=group)
        curve.keyframe_points.add(len(frames))
        flat = []
        for frame, value in zip(frames, values):
            flat.extend((float(frame), float(value[i])))
        curve.keyframe_points.foreach_set("co", flat)
        for point in curve.keyframe_points:
            point.interpolation = "LINEAR"
        curve.update()


def build_action(name, arm, objects, frames, sample_fn):
    bones = [pb.name for pb in arm.pose.bones]
    samples = {n: ([], []) for n in bones}
    others = {o.name: ([], []) for o in objects}
    for f in frames:
        pose, nodes = sample_fn(f)
        for n in bones:
            loc, q, _ = pose[n].decompose()
            samples[n][0].append(loc)
            samples[n][1].append(q)
        for o in objects:
            loc, q, _ = nodes[o.name].decompose()
            others[o.name][0].append(loc)
            others[o.name][1].append(q)
    action = bpy.data.actions.new(name)
    action.use_fake_user = True
    slot = action.slots.new(id_type="OBJECT", name=arm.name)
    bag = anim_utils.action_ensure_channelbag_for_slot(action, slot)
    for n in bones:
        locs, quats = samples[n]
        write_curves(bag, f'pose.bones["{n}"].location', n, frames, locs, 3)
        write_curves(bag, f'pose.bones["{n}"].rotation_quaternion', n, frames, continuous(quats), 4)
    for o in objects:
        node_slot = action.slots.new(id_type="OBJECT", name=o.name)
        node_bag = anim_utils.action_ensure_channelbag_for_slot(action, node_slot)
        locs, quats = others[o.name]
        write_curves(node_bag, "location", o.name, frames, locs, 3)
        write_curves(node_bag, "rotation_quaternion", o.name, frames, continuous(quats), 4)
    return action


def authored_sampler(arm, rig, creature, followers, pose_fn):
    base_world = {o.name: o.matrix_world.copy() for o in followers}

    def sample(f):
        rig.reset()
        pose_fn(f)
        low = min((rig.world(rig.head(n)).z - creature.ground, n) for n in rig.order)
        creature.clearance = min(creature.clearance, (low[0], low[1], f))
        nodes = {}
        for o in followers:
            world = follower_world(rig, arm, FOLLOWERS[o.name], base_world[o.name])
            nodes[o.name] = (o.parent.matrix_world @ o.matrix_parent_inverse).inverted() @ world
        return dict(rig.basis), nodes

    return sample


def source_sampler(arm, objects):
    scene = bpy.context.scene

    def sample(f):
        scene.frame_set(f)
        return {pb.name: pb.matrix_basis.copy() for pb in arm.pose.bones}, {o.name: o.matrix_basis.copy() for o in objects}

    return sample


def slot_target(slot, objects):
    target = next((o for o in objects if slot.identifier == "OB" + o.name), None)
    if target is None:
        raise SystemExit(f"no object for action slot {slot.identifier}")
    return target


def bind_action(action, objects):
    for slot in action.slots:
        target = slot_target(slot, objects)
        if target.animation_data is None:
            target.animation_data_create()
        target.animation_data.action = action
        target.animation_data.action_slot = slot


def push_to_nla(action, objects):
    for slot in action.slots:
        target = slot_target(slot, objects)
        if target.animation_data is None:
            target.animation_data_create()
        track = target.animation_data.nla_tracks.new()
        track.name = action.name
        strip = track.strips.new(action.name, int(action.frame_range[0]), action)
        strip.action_slot = slot
        track.mute = True
    for o in objects:
        if o.animation_data:
            o.animation_data.action = None


def generate(source_dir, out_dir):
    reset_scene()
    arm = import_model(source_dir / "scene.gltf")
    scene = bpy.context.scene
    idle = bpy.data.actions.get(SOURCE_CLIP)
    if idle is None:
        raise SystemExit(f"source clip {SOURCE_CLIP!r} missing")
    if len(arm.data.bones) != 101:
        raise SystemExit(f"expected 101 joints, found {len(arm.data.bones)}")
    idle_objects = [o for o in scene.objects if o.type != "ARMATURE" and o.animation_data and o.animation_data.action == idle]
    followers = [bpy.data.objects[n] for n in FOLLOWERS]
    for o in {*idle_objects, *followers}:
        o.rotation_mode = "QUATERNION"
    start, end = (int(round(x)) for x in idle.frame_range)
    if (end - start) / FPS != 6.25:
        raise SystemExit(f"source clip spans {(end - start) / FPS} s, expected 6.25 s")
    actions = [build_action(IDLE, arm, idle_objects, list(range(start, end + 1)), source_sampler(arm, idle_objects))]
    scene.frame_set(start)
    rig = Rig(arm)
    creature = Creature(rig)
    depsgraph = bpy.context.evaluated_depsgraph_get()
    lowest = math.inf
    for obj in scene.objects:
        if obj.type == "MESH":
            evaluated = obj.evaluated_get(depsgraph)
            mesh = evaluated.to_mesh()
            for v in mesh.vertices:
                lowest = min(lowest, (obj.matrix_world @ v.co).z)
            evaluated.to_mesh_clear()
    creature.ground = lowest
    plants = chase_plants(creature)
    clips = {
        "WALK": (range(WALK["frames"] + 1), lambda f: walk_pose(creature, f)),
        "CHASE": (range(CHASE["frames"] + 1), lambda f: chase_pose(creature, f, plants)),
        "ATTACK": (range(ATTACK["frames"] + 1), lambda f: attack_pose(creature, f)),
    }
    reach, clearance = {}, {}
    for name, (frames, fn) in clips.items():
        creature.reach_error = 0.0
        creature.clearance = (math.inf, None, None)
        actions.append(build_action(name, arm, followers, list(frames), authored_sampler(arm, rig, creature, followers, fn)))
        reach[name] = creature.reach_error
        clearance[name] = creature.clearance
    targets = list({o.name: o for o in [arm, *idle_objects, *followers]}.values())
    bpy.data.actions.remove(idle)
    for action in actions:
        push_to_nla(action, targets)
    scene.frame_start = 0
    scene.frame_end = 150
    out_dir.mkdir(parents=True, exist_ok=True)
    glb = out_dir / f"{OUTPUT_NAME}.glb"
    bpy.ops.export_scene.gltf(
        filepath=str(glb),
        export_format="GLB",
        export_animations=True,
        export_animation_mode="NLA_TRACKS",
        export_merge_animation="NLA_TRACK",
        export_force_sampling=True,
        export_optimize_animation_size=False,
        export_reset_pose_bones=True,
        export_apply=False,
        export_yup=True,
    )
    return glb, {"ground_z": lowest, "reach_shortfall_m": reach, "lowest_joint_above_ground_m": clearance, "unit": arm.matrix_world.to_scale().x}


def read_glb(path):
    data = pathlib.Path(path).read_bytes()
    magic, _, length = struct.unpack_from("<4sII", data, 0)
    if magic != b"glTF" or length != len(data):
        raise SystemExit(f"bad glb header in {path}")
    offset = 12
    doc, binary = None, b""
    while offset < length:
        size, kind = struct.unpack_from("<I4s", data, offset)
        chunk = data[offset + 8: offset + 8 + size]
        if kind == b"JSON":
            doc = json.loads(chunk)
        elif kind == b"BIN\x00":
            binary = chunk
        offset += 8 + size
    return doc, binary


def read_gltf(path):
    doc = json.loads(pathlib.Path(path).read_text())
    binary = (pathlib.Path(path).parent / doc["buffers"][0]["uri"]).read_bytes()
    return doc, binary


def accessor(doc, binary, index):
    acc = doc["accessors"][index]
    if acc["componentType"] != 5126:
        raise SystemExit(f"accessor {index} is not float")
    width = {"SCALAR": 1, "VEC3": 3, "VEC4": 4}[acc["type"]]
    view = doc["bufferViews"][acc["bufferView"]]
    start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
    stride = view.get("byteStride", 4 * width)
    return [struct.unpack_from(f"<{width}f", binary, start + i * stride) for i in range(acc["count"])]


def channels(doc, binary, anim):
    out = {}
    for ch in anim["channels"]:
        sampler = anim["samplers"][ch["sampler"]]
        node = doc["nodes"][ch["target"]["node"]].get("name")
        out[(node, ch["target"]["path"])] = (
            [t[0] for t in accessor(doc, binary, sampler["input"])],
            accessor(doc, binary, sampler["output"]),
            sampler.get("interpolation", "LINEAR"),
        )
    return out


def sample(times, values, t):
    if t <= times[0]:
        return values[0]
    if t >= times[-1]:
        return values[-1]
    lo, hi = 0, len(times) - 1
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if times[mid] <= t:
            lo = mid
        else:
            hi = mid
    k = (t - times[lo]) / (times[hi] - times[lo])
    a, b = values[lo], values[hi]
    if len(a) == 4 and sum(x * y for x, y in zip(a, b)) < 0.0:
        b = [-x for x in b]
    return [x + (y - x) * k for x, y in zip(a, b)]


def value_error(path, a, b):
    if path == "rotation":
        qa, qb = Quaternion((a[3], a[0], a[1], a[2])).normalized(), Quaternion((b[3], b[0], b[1], b[2])).normalized()
        angle = qa.rotation_difference(qb).angle
        return math.degrees(min(angle, 2.0 * math.pi - angle))
    return max(abs(x - y) for x, y in zip(a, b))


def node_default(node, path):
    return node.get(path, {"translation": [0.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0, 1.0], "scale": [1.0, 1.0, 1.0]}[path])


def compare_idle(src_doc, source, exported, unit):
    worst = {"rotation_deg": 0.0, "translation_m": 0.0, "scale": 0.0}
    missing = []
    src_nodes = {n.get("name"): n for n in src_doc["nodes"]}
    duration = max(t for (ts, _, _) in source.values() for t in ts)
    for key, (etimes, evalues, _) in exported.items():
        node, path = key
        if node not in src_nodes:
            missing.append([*key, "not in source"])
            continue
        if abs(etimes[-1] - duration) > 1e-4:
            missing.append([*key, f"duration {etimes[-1]} != {duration}"])
        if key in source:
            times, values, _ = source[key]
        else:
            times, values = [0.0], [node_default(src_nodes[node], path)]
        field = {"rotation": "rotation_deg", "translation": "translation_m", "scale": "scale"}[path]
        scale = unit if path == "translation" else 1.0
        for t, v in zip(times, values):
            worst[field] = max(worst[field], value_error(path, v, sample(etimes, evalues, t)) * scale)
        for t, v in zip(etimes, evalues):
            worst[field] = max(worst[field], value_error(path, sample(times, values, t), v) * scale)
    for key in source:
        if key not in exported:
            missing.append(list(key))
    return {"max_error": worst, "missing": missing}


def compare_rest(src_doc, doc, unit):
    src_nodes = {n.get("name"): n for n in src_doc["nodes"]}
    worst = {"rotation_deg": 0.0, "translation_m": 0.0, "scale": 0.0}
    for j in doc["skins"][0]["joints"]:
        node = doc["nodes"][j]
        for path, field in (("rotation", "rotation_deg"), ("translation", "translation_m"), ("scale", "scale")):
            err = value_error(path, node_default(src_nodes[node["name"]], path), node_default(node, path))
            worst[field] = max(worst[field], err * (unit if path == "translation" else 1.0))
    return worst


def validate(glb, source_dir, unit):
    doc, binary = read_glb(glb)
    src_doc, src_binary = read_gltf(source_dir / "scene.gltf")
    joints = {doc["nodes"][j]["name"] for j in doc["skins"][0]["joints"]}
    report = {"joints": len(joints), "skins": len(doc.get("skins", [])), "images": len(doc.get("images", [])), "materials": len(doc.get("materials", [])), "clips": {}}
    problems = []
    if len(joints) != 101:
        problems.append(f"joint count {len(joints)}")
    anims = {a["name"]: a for a in doc.get("animations", [])}
    expected = {IDLE, "WALK", "CHASE", "ATTACK"}
    if set(anims) != expected or len(doc.get("animations", [])) != len(expected):
        problems.append(f"clips {sorted(a['name'] for a in doc.get('animations', []))}")
    for name in (IDLE, "WALK", "CHASE", "ATTACK"):
        if name not in anims:
            problems.append(f"missing clip {name}")
            continue
        ch = channels(doc, binary, anims[name])
        times = [t for (ts, _, _) in ch.values() for t in ts]
        values = [x for (_, vs, _) in ch.values() for v in vs for x in v]
        animated_joints = {n for (n, _) in ch if n in joints}
        entry = {
            "duration_s": round(max(times), 6),
            "channels": len(ch),
            "paths": {p: sum(1 for (_, q) in ch if q == p) for p in ("translation", "rotation", "scale")},
            "animated_joints": len(animated_joints),
            "extra_nodes": sorted({n for (n, _) in ch if n not in joints}),
            "interpolation": sorted({i for (_, _, i) in ch.values()}),
            "finite": all(math.isfinite(x) for x in values),
        }
        expected = 6.25 if name == IDLE else CLIP_FRAMES[name] / FPS
        if abs(entry["duration_s"] - expected) > 1e-4:
            problems.append(f"{name} lasts {entry['duration_s']} s, expected {expected} s")
        if entry["interpolation"] != ["LINEAR"]:
            problems.append(f"{name} interpolation {entry['interpolation']}")
        if name != IDLE:
            seam = 0.0
            for (_, vs, _) in ch.values():
                seam = max(seam, max(abs(a - b) for a, b in zip(vs[0], vs[-1])))
            entry["first_last_max_delta"] = seam
            if seam > 1e-6:
                problems.append(f"{name} first and last keys differ by {seam}")
        if len(animated_joints) != 101:
            problems.append(f"{name} animates {len(animated_joints)} joints")
        if not entry["finite"]:
            problems.append(f"{name} has non-finite values")
        report["clips"][name] = entry
    if IDLE in anims:
        src = channels(src_doc, src_binary, src_doc["animations"][0])
        idle = compare_idle(src_doc, src, channels(doc, binary, anims[IDLE]), unit)
        report["idle_vs_source"] = idle
        if idle["missing"] or any(idle["max_error"][k] > v for k, v in IDLE_TOLERANCE.items()):
            problems.append("IDLE differs from source clip")
    report["bind_rest_vs_source_defaults"] = compare_rest(src_doc, doc, unit)
    structure = {k: [len(src_doc.get(k, [])), len(doc.get(k, []))] for k in ("nodes", "meshes", "materials", "images", "skins")}
    report["structure_source_vs_output"] = structure
    for k, (a, b) in structure.items():
        if a != b:
            problems.append(f"{k} count {b} != source {a}")
    src_alpha = {m["name"]: m.get("alphaMode", "OPAQUE") for m in src_doc["materials"]}
    out_alpha = {m["name"]: m.get("alphaMode", "OPAQUE") for m in doc["materials"]}
    if src_alpha != out_alpha:
        problems.append(f"material alpha modes {out_alpha} != source {src_alpha}")
    textures = {}
    for image in doc["images"]:
        view = doc["bufferViews"][image["bufferView"]]
        blob = binary[view.get("byteOffset", 0): view.get("byteOffset", 0) + view["byteLength"]]
        source_png = source_dir / "textures" / f"{image['name']}.png"
        textures[image["name"]] = source_png.is_file() and hashlib.sha256(blob).digest() == hashlib.sha256(source_png.read_bytes()).digest()
    report["textures_identical_to_source"] = textures
    if not all(textures.values()):
        problems.append("embedded textures differ from source")
    report["problems"] = problems
    return report


def setup_preview(scene):
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 480
    scene.render.resolution_y = 400
    scene.render.fps = FPS
    world = bpy.data.worlds.new("preview")
    scene.world = world
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.22, 0.22, 0.25, 1.0)
    for i, (angles, energy) in enumerate((((50, 0, 30), 4.0), ((60, 0, -140), 2.0))):
        light = bpy.data.objects.new(f"preview_sun_{i}", bpy.data.lights.new(f"preview_sun_{i}", "SUN"))
        light.data.energy = energy
        light.rotation_euler = [math.radians(a) for a in angles]
        scene.collection.objects.link(light)
    camera = bpy.data.objects.new("preview_camera", bpy.data.cameras.new("preview_camera"))
    camera.data.lens = 35
    scene.collection.objects.link(camera)
    scene.camera = camera
    return camera


def place_camera(camera, position, target):
    camera.location = position
    camera.rotation_euler = (Vector(target) - Vector(position)).to_track_quat("-Z", "Y").to_euler()


def previews(glb, out_dir, ground):
    reset_scene()
    import_model(glb)
    scene = bpy.context.scene
    camera = setup_preview(scene)
    floor = bpy.data.objects.new("preview_floor", bpy.data.meshes.new("preview_floor"))
    floor.data.from_pydata([(-20, -20, ground), (20, -20, ground), (20, 20, ground), (-20, 20, ground)], [], [(0, 1, 2, 3)])
    scene.collection.objects.link(floor)
    objects = list(scene.objects)
    for o in objects:
        if o.animation_data:
            o.animation_data.action = None
    target = (0.0, -2.2, -1.9)
    views = {"side": (15.5, -2.2, -1.0), "front": (9.0, 10.0, -0.2)}
    preview_dir = out_dir / "previews"
    if preview_dir.exists():
        shutil.rmtree(preview_dir)
    preview_dir.mkdir(parents=True)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        for name in (IDLE, "WALK", "CHASE", "ATTACK"):
            action = bpy.data.actions[name]
            bind_action(action, objects)
            start, end = (int(round(x)) for x in action.frame_range)
            step = 2 if name == IDLE else 1
            frames = list(range(start, end + 1, step))
            for view, position in views.items():
                place_camera(camera, position, target)
                for i, f in enumerate(frames):
                    scene.frame_set(f)
                    scene.render.filepath = str(tmp / f"{name}_{view}_{i:04d}.png")
                    bpy.ops.render.render(write_still=True)
            rate = FPS / step
            subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-framerate", str(rate), "-i", str(tmp / f"{name}_side_%04d.png"), "-framerate", str(rate), "-i", str(tmp / f"{name}_front_%04d.png"), "-filter_complex", "hstack=inputs=2,format=yuv420p", "-c:v", "libx264", "-crf", "26", str(preview_dir / f"{name.lower()}.mp4")], check=True)
            picks = [round(k * (len(frames) - 1) / 7) for k in range(8)]
            rows = []
            for view in views:
                row = tmp / f"{name}_{view}_row.png"
                inputs = []
                for k in picks:
                    inputs += ["-i", str(tmp / f"{name}_{view}_{k:04d}.png")]
                subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *inputs, "-filter_complex", f"hstack=inputs={len(picks)},scale=iw/2:ih/2", str(row)], check=True)
                rows.append(row)
            subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", str(rows[0]), "-i", str(rows[1]), "-filter_complex", "vstack=inputs=2", str(preview_dir / f"{name.lower()}_sheet.png")], check=True)
            for o in objects:
                if o.animation_data:
                    o.animation_data.action = None
            scene.frame_set(0)
            print(f"preview {name}: frames {picks}")


def main():
    args = parse_args()
    source = pathlib.Path(args.source).resolve()
    out_dir = pathlib.Path(args.out).resolve()
    with tempfile.TemporaryDirectory() as tmp:
        if source.is_file() and zipfile.is_zipfile(source):
            with zipfile.ZipFile(source) as archive:
                archive.extractall(tmp)
            source_dir = pathlib.Path(tmp)
            source_hash = sha256(source)
        else:
            source_dir = source
            source_hash = None
        license_text = (source_dir / "license.txt").read_text()
        glb, facts = generate(source_dir, out_dir)
        report = validate(glb, source_dir, facts["unit"])
        for name, shortfall in facts["reach_shortfall_m"].items():
            if shortfall > 1e-4:
                report["problems"].append(f"{name} IK target out of reach by {shortfall:.4f} m")
        for name, (height, joint, frame) in facts["lowest_joint_above_ground_m"].items():
            if height < 0.0:
                report["problems"].append(f"{name} joint {joint} is {-height:.4f} m below ground at frame {frame}")
        for name, nodes in EXTRA_NODES.items():
            found = report["clips"].get(name, {}).get("extra_nodes")
            if found != sorted(nodes):
                report["problems"].append(f"{name} extra animated nodes {found} != {sorted(nodes)}")
        manifest = {
            "output": glb.name,
            "output_sha256": sha256(glb),
            "source": source.name,
            "source_sha256": source_hash,
            "source_license_txt": license_text.strip().splitlines(),
            "blender": bpy.app.version_string,
            "fps": FPS,
            "ground_z_m": round(facts["ground_z"], 4),
            "reach_shortfall_m": {k: round(v, 4) for k, v in facts["reach_shortfall_m"].items()},
            "lowest_joint_above_ground": {k: {"height_m": round(v[0], 4), "joint": v[1], "frame": v[2]} for k, v in facts["lowest_joint_above_ground_m"].items()},
            "clips": {
                IDLE: {"source_name": SOURCE_CLIP, "loop": True},
                "WALK": {"loop": True, "in_place_speed_m_s": round(WALK["stride"] / (WALK["duty"] * WALK["frames"] / FPS), 3)},
                "CHASE": {"loop": True, "in_place_speed_m_s": round(CHASE["stride"] / (CHASE["duty"] * CHASE["frames"] / FPS), 3)},
                "ATTACK": {"loop": False, "hit_time_s": round(ATTACK["hit_frame"] / FPS, 4)},
            },
            "validation": report,
        }
        for name, entry in report["clips"].items():
            manifest["clips"][name].update(entry)
        (out_dir / f"{OUTPUT_NAME}.manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        print(json.dumps({k: manifest[k] for k in ("reach_shortfall_m", "lowest_joint_above_ground")}, indent=2))
        print(json.dumps(report, indent=2))
        if report["problems"]:
            raise SystemExit(f"validation failed: {report['problems']}")
        if args.previews:
            previews(glb, out_dir, facts["ground_z"])


if __name__ == "__main__":
    main()
