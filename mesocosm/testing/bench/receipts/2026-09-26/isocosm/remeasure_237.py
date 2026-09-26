"""Compare the ecology re-measure after ruling 237's cuts with checkpoint 1's.

usage: python remeasure_237.py CHECKPOINT1 AFTER [OUTPUT]
Both are isocosm-scale --remeasure receipts of the same step-1 points; each
point's "after" mean tick is compared. Writes a JSON summary when OUTPUT is
given.
"""
import json
import sys

one = json.load(open(sys.argv[1]))
two = json.load(open(sys.argv[2]))


def key(c):
    return (c["source"].split("/")[-1], c["run"], c["mode"], c["population"], c["lineages"], c["cohort_size"], c["seed"])


before = {key(c): c for c in one["comparisons"]}
rows = []
for c in two["comparisons"]:
    b = before[key(c)]
    assert b["ticks"] == c["ticks"] and b["evaluations_per_tick"] == c["evaluations_per_tick"], key(c)
    rows.append({
        "source": key(c)[0],
        "run": c["run"],
        "mode": c["mode"],
        "population": c["population"],
        "lineages": c["lineages"],
        "cohort_size": c["cohort_size"],
        "seed": c["seed"],
        "ticks": c["ticks"],
        "evaluations_per_tick": c["evaluations_per_tick"],
        "checkpoint_1_mean_tick_micros": b["after_mean_tick_micros"],
        "after_237_mean_tick_micros": c["after_mean_tick_micros"],
        "speedup": b["after_mean_tick_micros"] / max(1, c["after_mean_tick_micros"]),
        "identical_to_step_1": c["identical"],
        "micros_per_evaluation": [
            b["after_mean_tick_micros"] / max(1, c["evaluations_per_tick"]),
            c["after_mean_tick_micros"] / max(1, c["evaluations_per_tick"]),
        ],
    })
seconds = [sum(r[k] * r["ticks"] for r in rows) / 1e6 for k in ("checkpoint_1_mean_tick_micros", "after_237_mean_tick_micros")]
speedups = sorted(r["speedup"] for r in rows)
for r in sorted(rows, key=lambda r: (r["mode"], r["run"], r["population"], r["lineages"])):
    print(f"{r['source']:22} {r['run']:10} {r['mode']:11} pop {r['population']:5} lin {r['lineages']:2} coh {r['cohort_size']:3}"
          f"  {r['checkpoint_1_mean_tick_micros'] / 1e3:9.2f} ms -> {r['after_237_mean_tick_micros'] / 1e3:9.2f} ms"
          f"  x{r['speedup']:5.2f}  us/eval {r['micros_per_evaluation'][0]:6.2f} -> {r['micros_per_evaluation'][1]:6.2f}"
          f"  identical {r['identical_to_step_1']}")
print(f"points {len(rows)}; tick seconds {seconds[0]:.2f} -> {seconds[1]:.2f}, overall x{seconds[0] / seconds[1]:.2f};"
      f" speedup min {speedups[0]:.2f} median {speedups[len(speedups) // 2]:.2f} max {speedups[-1]:.2f};"
      f" all identical to step 1: {all(r['identical_to_step_1'] for r in rows)}")
if len(sys.argv) > 3:
    doc = {
        "kind": "isocosm-remeasure-237-comparison",
        "note": "Each step-1 ecology point's mean tick as checkpoint 1 re-measured it (remeasure-ecology.json of 2026-09-25, at d12e430) and as it re-measured after ruling 237's two cuts. Identical is against the step-1 receipt: the final state hash, every per-tick count and the distinct states.",
        "points": len(rows),
        "tick_seconds": {"checkpoint_1": round(seconds[0], 2), "after_237": round(seconds[1], 2)},
        "overall_speedup": round(seconds[0] / seconds[1], 2),
        "speedup_min_median_max": [round(speedups[0], 2), round(speedups[len(speedups) // 2], 2), round(speedups[-1], 2)],
        "all_identical_to_step_1": all(r["identical_to_step_1"] for r in rows),
        "rows": rows,
    }
    open(sys.argv[3], "w").write(json.dumps(doc, indent=2) + "\n")
    print("wrote", sys.argv[3])
