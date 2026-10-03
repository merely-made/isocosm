"""Summarize checkpoint 6's density ladder of isocosm-probe --bodies
--density receipts: for each density, the exact runner's and the crowd's
evaluations and time, their ratios, and the crowd's living members per
state at the end.

usage: python c6_density.py OUTPUT.json RECEIPT [RECEIPT ...]"""
import hashlib
import json
import sys
from pathlib import Path

rows = []
for path in sys.argv[2:]:
    raw = Path(path).read_bytes()
    d = json.loads(raw)
    s, den = d["savings"], d["density"]
    rows.append({
        "file": Path(path).name,
        "sha256": hashlib.sha256(raw).hexdigest(),
        "master_seed": d["master_seed"],
        "draws": len(d["draws"]),
        "producers_per_site": d["domain"]["producers"],
        "grazers_per_site": d["domain"]["grazers"],
        "exact_evaluations": s["exact_evaluations"],
        "crowd_evaluations": s["crowd_evaluations"],
        "evaluation_ratio": round(s["evaluation_ratio"], 2),
        "per_draw_evaluation_ratio": {k: round(v, 2) for k, v in s["per_draw_evaluation_ratio"].items()},
        "exact_seconds": round(s["exact_micros"] / 1e6, 2),
        "crowd_seconds": round(s["crowd_micros"] / 1e6, 3),
        "wall_ratio": round(s["wall_ratio"], 1),
        "alive_end": {k: round(v, 1) for k, v in den["alive_end"].items()},
        "alive_per_state_end": {k: round(v, 2) for k, v in den["alive_per_state_end"].items()},
        "shortfalls": sum(a.get("shortfalls", 0) for dr in d["draws"] for a in dr["arms"]),
    })
rows.sort(key=lambda r: r["producers_per_site"][0])
for r in rows:
    print(r["producers_per_site"], "evaluations", r["exact_evaluations"], "->", r["crowd_evaluations"],
          f"({r['evaluation_ratio']}x)", "time", r["exact_seconds"], "->", r["crowd_seconds"],
          f"({r['wall_ratio']}x)", "alive per state", r["alive_per_state_end"]["median"],
          "shortfalls", r["shortfalls"])
json.dump({"kind": "isocosm-probe-bodies-density-ladder", "rows": rows}, open(sys.argv[1], "w"), indent=1)
