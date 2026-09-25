"""Recompute the probe receipt's statistics independently of the Rust check.

usage: python verify.py probe.json

From the raw per-draw readings of each arm this recomputes every
Kolmogorov-Smirnov distance and DKW-Massart equivalence p-value exactly,
with Holm's adjustment, and the difference test by its own within-pair
permutations drawn with Python's generator. It asserts the distances and
equivalence verdicts match the receipt, reports the difference verdicts
side by side, and prints the summary tables.
"""
import json
import math
import random
import sys

receipt = json.load(open(sys.argv[1]))
arms = {}
for d in receipt["draws"]:
    for a in d["arms"]:
        arms.setdefault(a["arm"], []).append(a["readings"])
pairs = {
    "exact against crowd": ("exact", "crowd"),
    "exact against exact (positive control)": ("exact", "exact-control"),
    "exact against averaged crowd (negative control)": ("exact", "crowd-averaged"),
}


def distance(a, b):
    values = sorted(set(a) | set(b))
    diff = {v: 0 for v in values}
    for x in a:
        diff[x] += 1
    for y in b:
        diff[y] -= 1
    run = best = 0
    for v in values:
        run += diff[v]
        best = max(best, abs(run))
    return best / len(a)


def equivalence(d, bound, n):
    if d >= bound:
        return 1.0
    t = (bound - d) / 2
    return min(1.0, 4 * math.exp(-2 * n * t * t))


def permutation(a, b, rounds, rng):
    observed = distance(a, b)
    extreme = 0
    for _ in range(rounds):
        x, y = [], []
        for p, q in zip(a, b):
            if rng.random() < 0.5:
                p, q = q, p
            x.append(p)
            y.append(q)
        if distance(x, y) >= observed - 1e-12:
            extreme += 1
    return (1 + extreme) / (1 + rounds)


def holm(p):
    m = len(p)
    order = sorted(range(m), key=lambda i: p[i])
    adjusted, running = [0.0] * m, 0.0
    for rank, i in enumerate(order):
        running = max(running, min(1.0, (m - rank) * p[i]))
        adjusted[i] = running
    return adjusted


rng = random.Random(20260925)
alpha = receipt["settings"]["alpha"]
rounds = int(sys.argv[2]) if len(sys.argv) > 2 else 499
print(f"draws {len(receipt['draws'])}, readings {len(receipt['readings'])}, master seed {receipt['master_seed']}")
print("verdicts", json.dumps(receipt["verdicts"]))
for comparison in receipt["comparisons"]:
    left, right = pairs[comparison["name"]]
    rows = comparison["readings"]
    n = len(arms[left])
    ds, pes, pds = [], [], []
    for r, row in enumerate(rows):
        a = [v[r] for v in arms[left]]
        b = [v[r] for v in arms[right]]
        d = distance(a, b)
        assert abs(d - row["distance"]) < 1e-12, (comparison["name"], row["key"], d, row["distance"])
        ds.append(d)
        pes.append(equivalence(d, row["bound"], n))
        pds.append(permutation(a, b, rounds, rng))
    pe_holm, pd_holm = holm(pes), holm(pds)
    print(f"\n{comparison['name']}: receipt equivalent {comparison['equivalent']}, different {comparison['different']}, pass {comparison['pass']}")
    for row, d, pe, pd in zip(rows, ds, pe_holm, pd_holm):
        assert abs(pe - row["p_equivalence_holm"]) < 1e-9 * max(1.0, pe), (row["key"], pe, row["p_equivalence_holm"])
        assert (pe <= alpha) == row["certified"], row["key"]
        agree = (pd <= alpha) == row["detected"]
        print(f"  {row['key']:<32} D={d:.4f} means {row['mean_a']:8.2f} {row['mean_b']:8.2f}  certified {row['certified']!s:<5} (p {pe:.2e})  detected {row['detected']!s:<5} (receipt p {row['p_difference_holm']:.4f}, recomputed {pd:.4f}{'' if agree else ', DISAGREES'})")
print("\nAll distances and equivalence verdicts recomputed and matched.")
print("savings", json.dumps(receipt["savings"]))
print("density", json.dumps(receipt["density"]))
print("checks", json.dumps(receipt["checks"]))
