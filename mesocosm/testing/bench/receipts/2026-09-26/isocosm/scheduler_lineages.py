"""Set the lineage sweep re-measured on the core before ruling 258 beside
the core after it, point by point.

usage: python scheduler_lineages.py OUTPUT.json BEFORE-1.json AFTER-1.json [BEFORE-2.json AFTER-2.json ...]

Each is an isocosm-scale --remeasure receipt of the same points, the lineage
sweep of receipts/2026-09-25/isocosm/scale.json, the two cores run in turn
on one shared machine, round after round. A point's time is its fastest
round on each core, since other work on the machine only ever slows one.
Each receipt compares its own run against that step-1 receipt; this sets
the two cores beside each other.
"""
import json
import sys


def runs(path):
    receipt = json.load(open(path))
    out = {}
    for c, p in zip(receipt["comparisons"], receipt["points"]):
        key = (p["family"], c["mode"], c["population"], c["lineages"], c["cohort_size"], c["seed"])
        out[key] = (c, p)
    return out


def mean(xs):
    return sum(xs) / len(xs)


paths = sys.argv[2:]
befores, afters = [runs(p) for p in paths[0::2]], [runs(p) for p in paths[1::2]]
keys = befores[0].keys()
assert all(r.keys() == keys for r in befores + afters), "the receipts ran different points"
rows = []
for key in sorted(keys, key=lambda k: (k[0], k[3], k[5])):
    b = [r[key] for r in befores]
    a = [r[key] for r in afters]
    fastest = [min(p["mean_tick_micros"] for _, p in side) for side in (b, a)]
    hashes = {p["final_hash"] for _, p in b + a}
    evaluations = [{p["evaluations_per_tick"] for _, p in side} for side in (b, a)]
    assert all(len(e) == 1 for e in evaluations), "counts differ between rounds"
    rows.append({
        "family": key[0],
        "lineages": key[3],
        "seed": key[5],
        "ticks": a[0][0]["ticks"],
        "mean_tick_micros_by_round": [[p["mean_tick_micros"] for _, p in side] for side in (b, a)],
        "mean_tick_micros": fastest,
        "speedup": fastest[0] / max(1, fastest[1]),
        "evaluations_per_tick": [e.pop() for e in evaluations],
        "final_hash_equal": len(hashes) == 1,
        "identical_to_step_1": all(c["identical"] for c, _ in b + a),
    })
groups = {}
for r in rows:
    g = groups.setdefault((r["family"], r["lineages"]), [[], [], [], []])
    for i, v in enumerate(r["mean_tick_micros"] + r["evaluations_per_tick"]):
        g[i].append(v)
summary = []
for (fam, lineages), (t0, t1, e0, e1) in sorted(groups.items()):
    summary.append({
        "family": fam,
        "lineages": lineages,
        "mean_tick_ms": [round(mean(t0) / 1e3, 2), round(mean(t1) / 1e3, 2)],
        "speedup": round(mean(t0) / max(1, mean(t1)), 2),
        "evaluations_per_tick": [round(mean(e0)), round(mean(e1))],
    })
    print(f"{fam:9} L={lineages:2}  tick {mean(t0) / 1e3:8.2f} ms -> {mean(t1) / 1e3:7.2f} ms"
          f"  x{mean(t0) / max(1, mean(t1)):5.1f}   evaluations/tick {mean(e0):9.0f} -> {mean(e1):7.0f}")
identical = all(r["identical_to_step_1"] and r["final_hash_equal"] for r in rows)
print("final hashes equal and every run identical to step 1:", identical)
doc = {
    "kind": "isocosm-scheduler-lineages",
    "note": "The lineage sweep of receipts/2026-09-25/isocosm/scale.json (1,024 members in cohorts of 32 over 256 sites, 16 ticks grouped, two draws per lineage count, both families), re-measured on the core before ruling 258 (be9d863) and after it, in turn, three rounds each on a shared machine; each point's time is its fastest round on each core. Identical to step 1 is each run's own comparison with that receipt: final state hash, per-tick acceptances, members, groups, events, notes and arrivals, and distinct states. Evaluations are those counted: before, every due process against every living group; after, only against groups carrying the traits it requires.",
    "rounds": len(befores),
    "all_identical": identical,
    "summary": summary,
    "rows": rows,
}
open(sys.argv[1], "w").write(json.dumps(doc, indent=2) + "\n")
print("wrote", sys.argv[1])
