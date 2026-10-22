#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Inspect actual GPU frames; package their unchanged pixels as a WebP preview."""
import argparse
import collections
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageChops


def overlap(a, b):
    return any(all(lo[k] < bh[k] and bl[k] < hi[k] for k in range(3))
               for lo, hi in a for bl, bh in b)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("frames", type=Path, help="fresh recorder directory containing frames.json")
    parser.add_argument("--preview", required=True, type=Path)
    parser.add_argument("--summary", required=True, type=Path)
    parser.add_argument("--save", type=Path)
    parser.add_argument("--duration-ms", type=int, default=200)
    args = parser.parse_args()
    for output in [args.preview, args.summary]:
        if output.exists():
            parser.error(f"preserving existing receipt: {output}")
    if args.duration_ms <= 0:
        parser.error("duration must be positive")
    records = json.loads((args.frames / "frames.json").read_text())
    if not records:
        parser.error("the recorder supplied no frames")
    images = []
    hashes = []
    for record in records:
        source = args.frames / record["image"]
        hashes.append(hashlib.sha256(source.read_bytes()).hexdigest())
        with Image.open(source) as image:
            images.append(image.convert("RGBA"))
    changes = []
    for index in range(1, len(images)):
        previous, current = records[index-1:index+1]
        if previous.get("revision") != current.get("revision") or previous["roots"] != current["roots"]:
            continue
        diff = ImageChops.difference(images[index-1].convert("RGB"), images[index].convert("RGB"))
        changes.append({"frames": [index-1, index],
                        "changed_pixels": sum(pixel != (0, 0, 0) for pixel in diff.getdata()),
                        "bounds": diff.getbbox()})
    collisions = []
    latest = {}
    slopes = {}
    for index, record in enumerate(records):
        roots = record["roots"]
        for root in roots:
            latest[root["id"]] = root
            heights = {root["at"][1] * 1024 + sole[1] for sole in root.get("feet", {}).values()}
            if len(heights) > 1:
                slopes[root["id"]] = sorted(height / 1024 for height in heights)
        for at, a in enumerate(roots):
            for b in roots[at+1:]:
                if a.get("bounds") and b.get("bounds") and overlap(a["bounds"], b["bounds"]):
                    collisions.append({"frame": index, "ids": [a["id"], b["id"]]})
    summary = {
        "frames": len(records), "source_png_sha256": hashes,
        "timing": f"preview uses {args.duration_ms} ms per actual frame; raw elapsed_ms remains in frames.json",
        "swing_frames": sum(record["swinging"] > 0 for record in records),
        "supported_swing_frames": sum(record["swinging"] > 0 and record["planted"] > 0 for record in records),
        "max_swinging": max(record["swinging"] for record in records),
        "max_planted": max(record["planted"] for record in records),
        "projection_failures": max(record["projection_failures"] for record in records),
        "same_native_cut_changes": changes,
        "visible_native_box_intersections": collisions,
        "observed_facings": dict(collections.Counter(root.get("yaw", "unknown") for root in latest.values())),
        "bodies_with_feet_on_different_heights": slopes,
        "window": records[-1].get("window"),
    }
    if args.save:
        saved = json.loads(args.save.read_text())
        native = saved["native"]
        summary["native"] = {"seed": native["genesis"]["seed"], "tick": native["tick"],
                             "hash": native["state_hash"], "footprint": native["genesis"]["world"]["footprint"]}
    images[0].save(args.preview, save_all=True, append_images=images[1:],
                   duration=args.duration_ms, loop=0, lossless=True, exact=True, method=3)
    args.summary.write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps({key: summary[key] for key in ["frames", "swing_frames", "supported_swing_frames",
                                                  "projection_failures", "observed_facings"]}, indent=2))


if __name__ == "__main__":
    main()
