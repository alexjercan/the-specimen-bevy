import argparse
import hashlib
import json
import pathlib
import shutil


NAMES = ("sign_label_lab", "sign_label_security", "sign_label_prep")
SOURCE = pathlib.Path("art/visuals/generated/modules")
SHIPPED = pathlib.Path("assets/facility/modules")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("generated", type=pathlib.Path)
    args = parser.parse_args()
    generated = args.generated
    manifest = json.loads((generated / "modules.manifest.json").read_text())
    bundles = []
    for name in NAMES:
        entry = manifest["modules"][name]
        data = (generated / entry["file"]).read_bytes()
        if hashlib.sha256(data).hexdigest() != entry["glb_sha256"]:
            raise SystemExit(f"invalid generated hash for {name}")
        bundles.append((name, entry, data))

    for destination in (SOURCE, SHIPPED):
        path = destination / "modules.manifest.json"
        existing = json.loads(path.read_text())
        for name, entry, data in bundles:
            asset = destination / entry["file"]
            if asset.exists() and asset.read_bytes() != data:
                raise SystemExit(f"refusing to replace existing {asset}")
        for name, entry, data in bundles:
            asset = destination / entry["file"]
            if not asset.exists():
                shutil.copyfile(generated / entry["file"], asset)
            existing["modules"][name] = entry
        path.write_text(json.dumps(existing, indent=2) + "\n")


if __name__ == "__main__":
    main()
