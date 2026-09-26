"""Set the lineage sweep after ruling 287 beside it after ruling 286, point by
point: evaluations and acceptances per tick, the share of the members
evaluated whose act was blocked, and whether the point is identical to step
1 (reservoir points should be; ecology points change, since feeding did).

Usage: python lineages_287.py AFTER-286.json AFTER-287.json OUTPUT.json"""
import hashlib
import json
import statistics
import sys
from pathlib import Path


def load(path):
    raw = Path(path).read_bytes()
    return json.loads(raw), hashlib.sha256(raw).hexdigest()


(a, ha), (b, hb) = load(sys.argv[1]), load(sys.argv[2])


def mean(p, key):
    return sum(t[key] for t in p["per_tick"]) / len(p["per_tick"])


def blocked(p):
    t = p["per_tick"]
    return 100 * sum(x["blocked"] for x in t) / max(1, sum(x["represented"] for x in t))


rows = []
for ca, pa, cb, pb in zip(a["comparisons"], a["points"], b["comparisons"], b["points"]):
    assert (ca["seed"], ca["lineages"], ca["mode"]) == (cb["seed"], cb["lineages"], cb["mode"])
    rows.append({
        "family": pa["family"],
        "mode": ca["mode"],
        "population": ca["population"],
        "lineages": ca["lineages"],
        "seed": ca["seed"],
        "ticks": ca["ticks"],
        "evaluations_per_tick": [ca["after_evaluations_per_tick"], cb["after_evaluations_per_tick"]],
        "accepted_per_tick": [round(mean(pa, "accepted"), 2), round(mean(pb, "accepted"), 2)],
        "blocked_share_percent": [round(blocked(pa), 1), round(blocked(pb), 1)],
        "identical_to_step_1": [ca["identical"], cb["identical"]],
    })
summary = []
for fam in ["reservoir", "ecology"]:
    for l in sorted({r["lineages"] for r in rows}):
        pts = [r for r in rows if r["family"] == fam and r["lineages"] == l]
        summary.append({
            "family": fam,
            "lineages": l,
            "evaluations_per_tick": [statistics.mean(r["evaluations_per_tick"][i] for r in pts) for i in (0, 1)],
            "accepted_per_tick": [round(statistics.mean(r["accepted_per_tick"][i] for r in pts), 2) for i in (0, 1)],
            "blocked_share_percent": [[r["blocked_share_percent"][i] for r in pts] for i in (0, 1)],
            "identical_to_step_1": [all(r["identical_to_step_1"][i] for r in pts) for i in (0, 1)],
        })
doc = {
    "kind": "isocosm-lineage-sweep-287",
    "note": "Each pair is [after ruling 286, after ruling 287], the same 20 points of the step-1 lineage sweep (1,024 members, 16 ticks, grouped) re-measured on each core. Evaluations per tick are the scheduler's; accepted per tick counts members; the blocked share is the members evaluated whose act was blocked. Identical means as step 1 in every deterministic field but the evaluation counts.",
    "sources": {
        Path(sys.argv[1]).name: ha,
        Path(sys.argv[2]).name: hb,
    },
    "summary": summary,
    "points": rows,
}
Path(sys.argv[3]).write_text(json.dumps(doc, indent=1) + "\n")
for s in summary:
    print(s["family"], s["lineages"], s["evaluations_per_tick"], s["blocked_share_percent"], s["identical_to_step_1"])
