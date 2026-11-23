#!/usr/bin/env python3
"""Bundle the built desktop and its Pathfinder table utility with their packs and Djinn grants."""
import argparse
import json
from pathlib import Path
import subprocess
import tomllib

root = Path(__file__).resolve().parents[1]
crate = root / "crates" / "isocosm-genet"
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("desktop", type=Path, help="built isocosm-genet executable")
parser.add_argument("table", type=Path, help="built pf2e_table example")
parser.add_argument("output", type=Path, help="directory for Isocosm.app")
args = parser.parse_args()

with (crate / "Cargo.toml").open("rb") as manifest:
    package = tomllib.load(manifest)["package"]
config = dict(package["metadata"]["packager"], version=package["version"])
supplied = {"isocosm-genet": args.desktop, "pf2e_table": args.table}
for binary in config["binaries"]:
    path = supplied[binary["path"]].resolve(strict=True)
    if path.name != binary["path"]:
        parser.error(f"expected {binary['path']}, got {path.name}")
    binary["path"] = str(path)
for resource in config["resources"]:
    resource["src"] = str((crate / resource["src"]).resolve(strict=True))
output = args.output.resolve()
if (output / "Isocosm.app").exists():
    parser.error(f"{output / 'Isocosm.app'} already exists")
subprocess.run(["cargo-packager", "--config", json.dumps(config), "--out-dir", str(output)], check=True)
