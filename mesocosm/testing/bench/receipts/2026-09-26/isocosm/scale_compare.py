"""Compare two isocosm-scale receipts point by point, masking timings, heap
and the evaluation counters later rulings change; report per point whether
the rest is identical, by family.

usage: python scale_compare.py OLD NEW"""
import json, sys

MASK = {"micros", "mean_tick_micros", "median_tick_micros", "generate_micros", "heap_peak_bytes",
        "heap_live_end_bytes", "heap_live_bytes", "evaluations", "represented", "blocked",
        "evaluations_per_tick", "fits", "fit", "machine", "note"}


def strip(x):
    if isinstance(x, dict):
        return {k: strip(v) for k, v in x.items() if k not in MASK and not k.endswith("_micros")}
    if isinstance(x, list):
        return [strip(v) for v in x]
    return x


a = json.load(open(sys.argv[1]))
b = json.load(open(sys.argv[2]))
print("top-level keys", sorted(a.keys()))
pa, pb = a.get("points", []), b.get("points", [])
print("points", len(pa), len(pb))
for x, y in zip(pa, pb):
    same = strip(x) == strip(y)
    fam = x.get("family")
    ev = (x.get("evaluations_per_tick"), y.get("evaluations_per_tick"))
    print(f"  {x.get('run')!s:10} {fam!s:10} {x.get('mode')!s:12} l={x.get('founding', {}).get('lineages')} identical={same} evaluations/tick {ev[0]} -> {ev[1]}")
for key in a:
    if key == "points":
        continue
    if strip(a[key]) != strip(b.get(key)):
        print("differs at top level:", key)
