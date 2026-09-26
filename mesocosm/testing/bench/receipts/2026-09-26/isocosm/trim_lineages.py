"""Write the lineage sweep's points of a scale receipt alone.

usage: python trim_lineages.py SCALE.json OUTPUT.json
"""
import json
import sys

d = json.load(open(sys.argv[1]))
d["points"] = [p for p in d["points"] if p["run"] == "lineages"]
d["fits"] = []
d["note_trimmed"] = (
    "The lineage sweep's points of receipts/2026-09-25/isocosm/scale.json, alone: "
    "every other point and the fits removed, nothing else changed."
)
open(sys.argv[2], "w").write(json.dumps(d, indent=2))
print(len(d["points"]), sorted({(p["family"], p["founding"]["lineages"]) for p in d["points"]}))
