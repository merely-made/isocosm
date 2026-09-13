"""Verify native carving receipts; usage: python verify.py RAW_DIRECTORY."""
import json
import sys
from pathlib import Path
from PIL import Image, ImageChops

raw = Path(sys.argv[1])
rows = []
for name in ("axial", "roots", "cat"):
    path = raw / f"{name}-receipt.json"
    receipt = json.loads(path.read_text())
    assert receipt["ok"], name
    captures = {c["name"]: c for c in receipt["captures"]}
    fields = {k: c["fields"] for k, c in captures.items()}
    before, carved, rejected = (fields[k] for k in ("baseline", "carved", "rejected"))
    action = json.loads(carved["trial-carving-action"])
    removed = int(carved["trial-carving-removed"])
    assert action["before_solid_voxels"] - action["after_solid_voxels"] == removed > 0
    assert "Rejected(OutOfReach(" in rejected["trial-carving-outcomes"]
    assert rejected["trial-carving-before-revision"] == rejected["trial-carving-after-revision"]
    assert rejected["trial-carving-before-solid"] == rejected["trial-carving-after-solid"]
    assert rejected["terrain-upload-bytes"] == carved["terrain-upload-bytes"]
    for capture in captures.values():
        x, y, w, h = capture["viewport"]["border"]
        scale = capture["viewport"]["pixel_scale"]
        assert min(x, y) >= 0
        assert (x + w) * scale <= capture["width"]
        assert (y + h) * scale <= capture["height"]
    shown, hidden = (captures[k] for k in ("carved", "carving-hidden"))
    assert shown["viewport"] == hidden["viewport"]
    x, y, w, h = shown["viewport"]["border"]
    scale = shown["viewport"]["pixel_scale"]
    bounds = tuple(round(v * scale) for v in (x, y, x + w, y + h))
    a = Image.open(shown["path"]).convert("RGB").crop(bounds)
    b = Image.open(hidden["path"]).convert("RGB").crop(bounds)
    diff = ImageChops.difference(a, b).tobytes()
    changed = sum(bool(r | g | b) for r, g, b in zip(diff[0::3], diff[1::3], diff[2::3]))
    if name == "axial":
        assert changed >= 32, "qualified visible-marker fixture"
    rows.append({
        "name": name, "ok": True, "frames": receipt["frames"],
        "independent_removed_voxels": removed,
        "terrain_upload_delta": int(carved["terrain-upload-bytes"]) - int(before["terrain-upload-bytes"]),
        "terrain_write_call_delta": int(carved["terrain-write-calls"]) - int(before["terrain-write-calls"]),
        "carving_marker_changed_pixels": changed,
        "full_viewport_captured": True,
        "captures": [{"name": k, "path": c["path"], "viewport": c["viewport"],
            "fields": {f: v for f, v in c["fields"].items() if
                f.startswith(("trial-carving", "terrain-", "section-")) or
                f in ("trial-hash", "trial-steps", "trial-trace", "world-ground-revision")}}
            for k, c in captures.items()],
        "pixel_checks": receipt["pixel_checks"],
    })
print(json.dumps(rows, indent=2))
