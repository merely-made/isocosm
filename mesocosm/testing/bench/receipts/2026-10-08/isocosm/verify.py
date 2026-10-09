"""Recompute the probe receipt's statistics independently of the Rust check.

usage: python verify.py probe.json

From the raw per-draw readings of each arm this recomputes every
Kolmogorov-Smirnov distance and DKW-Massart equivalence p-value exactly,
with Holm's adjustment, and the difference test by its own within-pair
permutations drawn with Python's generator. It asserts the distances and
equivalence verdicts match the receipt, reports the difference verdicts
side by side, and prints the summary tables. A draw an arm refused has no
readings there and is left out of that arm's comparisons, as in the check,
and a crowd under certification that refused more than one draw in a
hundred fails its comparison; a control's refusals are only counted. Where
the worlds hunted, the only way a crowd refuses, it prints each arm's
refusals beside its bound.
"""
import json
import math
import random
import sys

receipt = json.load(open(sys.argv[1]))
draws = [{a["arm"]: a for a in d["arms"]} for d in receipt["draws"]]


def paired(left, right):
    """Each arm's readings over the draws both ran to the end."""
    kept = [(d[left], d[right]) for d in draws if left in d and right in d]
    kept = [(a, b) for a, b in kept if not a.get("refused") and not b.get("refused")]
    return [a["readings"] for a, _ in kept], [b["readings"] for _, b in kept]

# The most draws, per mille of those it ran, a crowd under certification may
# refuse; the other arms are the reference and the controls.
BOUND_PER_MILLE = 10
CERTIFIED = {"crowd", "crowd-approximate"}


def refusals(arm):
    """The draws an arm refused, of those it ran, and whether within bound."""
    ran = [d[arm] for d in draws if arm in d]
    refused = sum(1 for a in ran if a.get("refused"))
    within = arm not in CERTIFIED or refused * 1000 <= BOUND_PER_MILLE * len(ran)
    return refused, len(ran), within


pairs = {
    "exact against crowd": ("exact", "crowd"),
    "exact against exact (positive control)": ("exact", "exact-control"),
    "exact against averaged crowd (negative control)": ("exact", "crowd-averaged"),
    "exact against approximate crowd": ("exact", "crowd-approximate"),
    "crowd against approximate crowd": ("crowd", "crowd-approximate"),
    "exact against unweighted crowd (draw control)": ("exact", "crowd-unweighted"),
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


rng = random.Random(20260926)
alpha = receipt["settings"]["alpha"]
rounds = int(sys.argv[2]) if len(sys.argv) > 2 else 499
print(f"draws {len(receipt['draws'])}, readings {len(receipt['readings'])}, master seed {receipt['master_seed']}")
print("verdicts", json.dumps(receipt["verdicts"]))
for comparison in receipt["comparisons"]:
    left, right = pairs[comparison["name"]]
    rows = comparison["readings"]
    lefts, rights = paired(left, right)
    n = len(lefts)
    ds, pes, pds = [], [], []
    for r, row in enumerate(rows):
        a = [v[r] for v in lefts]
        b = [v[r] for v in rights]
        d = distance(a, b)
        assert abs(d - row["distance"]) < 1e-12, (comparison["name"], row["key"], d, row["distance"])
        ds.append(d)
        pes.append(equivalence(d, row["bound"], n))
        pds.append(permutation(a, b, rounds, rng))
    pe_holm, pd_holm = holm(pes), holm(pds)
    refused, ran, within = refusals(right)
    if "refusals" in comparison:
        r = comparison["refusals"]
        assert (r["refused"], r["of"], r["within"]) == (refused, ran, within), comparison["name"]
    if not within:
        assert not comparison["pass"], comparison["name"]
    print(f"\n{comparison['name']}: receipt equivalent {comparison['equivalent']}, different {comparison['different']}, pass {comparison['pass']}")
    for row, d, pe, pd in zip(rows, ds, pe_holm, pd_holm):
        assert abs(pe - row["p_equivalence_holm"]) < 1e-9 * max(1.0, pe), (row["key"], pe, row["p_equivalence_holm"])
        assert (pe <= alpha) == row["certified"], row["key"]
        agree = (pd <= alpha) == row["detected"]
        print(f"  {row['key']:<32} D={d:.4f} means {row['mean_a']:8.2f} {row['mean_b']:8.2f}  certified {row['certified']!s:<5} (p {pe:.2e})  detected {row['detected']!s:<5} (receipt p {row['p_difference_holm']:.4f}, recomputed {pd:.4f}{'' if agree else ', DISAGREES'})")
print("\nAll distances and equivalence verdicts recomputed and matched.")
if (receipt.get("domain") or {}).get("predators"):
    arms = [a["arm"] for a in receipt["draws"][0]["arms"]]
    told = []
    for arm in arms:
        refused, ran, within = refusals(arm)
        bound = f"bound 1%, {'within' if within else 'PAST IT'}" if arm in CERTIFIED else "no bound"
        told.append(f"{arm} {refused} of {ran} ({bound})")
    print("refusals: " + "; ".join(told))
print("savings", json.dumps(receipt["savings"]))
print("density", json.dumps(receipt["density"]))
print("checks", json.dumps(receipt["checks"]))
