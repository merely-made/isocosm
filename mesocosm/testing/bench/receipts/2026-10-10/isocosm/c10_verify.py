"""Independently recompute and gate a predeclared checkpoint 10 harm receipt.

Uses the C9 statistics and independently implements the receipt's declared
permutation tape. A pilot checks integrity but has no certification verdict.
Never present reanalysis of historical draws as a fresh run.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path


ARMS = ["exact", "exact-control", "crowd", "crowd-averaged", "crowd-unweighted"]
PAIRS = {
    "exact against crowd": ("exact", "crowd"),
    "exact against exact (positive control)": ("exact", "exact-control"),
    "exact against averaged crowd (negative control)": ("exact", "crowd-averaged"),
    "exact against unweighted crowd (draw control)": ("exact", "crowd-unweighted"),
}
INJURY = {"inspect:living-cells", "inspect:lost-cells", "inspect:severed"}
TRAITS = {"alive:growth:heal", "alive:growth:fragment"}
HARM = {"hazard": [2, 8], "cells": [1, 8], "rot": [1, 8], "heals": 500, "fragments": 500}
DOMAIN = {
    "seed": 1, "ticks": 60, "sites": [1, 2], "cohort": 8,
    "producers": [24, 64], "grazers": [4, 16], "frond": [3, 4],
    "limb": [3, 4], "limbs": [1, 2], "store": [1, 4], "reproduce": [1, 1],
    "variance": [1, 2], "absence": [12, 12], "clutch": [1, 4],
    "riff": [12, 100], "vary": [12, 100], "fixes": [6, 22], "grazes": [6, 24],
    "carriage": [7, 14], "candidates": 250, "semelparous": 500,
    "tissue": [150, 1000], "reserve": [0, 1000], "soil": [200, 12000],
    "mineralization": [0, 8], "bound_per_mille": 200, "harm": HARM,
}


def distance(a, b):
    diff = {v: 0 for v in set(a) | set(b)}
    for v in a:
        diff[v] += 1
    for v in b:
        diff[v] -= 1
    run = best = 0
    for v in sorted(diff):
        run += diff[v]
        best = max(best, abs(run))
    return best / len(a)


def equivalence(d, bound, n):
    if d >= bound:
        return 1.0
    return min(1.0, 4 * math.exp(-2 * n * ((bound - d) / 2) ** 2))


def holm(p):
    adjusted, running = [0.0] * len(p), 0.0
    for rank, i in enumerate(sorted(range(len(p)), key=lambda i: p[i])):
        running = max(running, min(1.0, (len(p) - rank) * p[i]))
        adjusted[i] = running
    return adjusted


def draw_seed(seed, domain, values):
    encoded = domain.encode("utf-8")
    payload = seed.to_bytes(8, "little") + len(encoded).to_bytes(8, "little") + encoded
    payload += b"".join(value.to_bytes(8, "little") for value in values)
    return int.from_bytes(hashlib.sha256(payload).digest()[:8], "little")


class Stream:
    def __init__(self, seed):
        self.state = seed

    def next(self):
        mask = (1 << 64) - 1
        self.state = (self.state + 0x9E3779B97F4A7C15) & mask
        z = self.state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & mask
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & mask
        return z ^ (z >> 31)

    def coin(self):
        return self.next() >> 63 == 1


def permutation(a, b, rounds, seed):
    values = sorted(set(a) | set(b))
    indices = {v: i for i, v in enumerate(values)}
    pairs = [(indices[p], indices[q]) for p, q in zip(a, b)]
    observed = round(distance(a, b) * len(a))
    stream = Stream(seed)
    extreme = 0
    for _ in range(rounds):
        diff = [0] * len(values)
        for p, q in pairs:
            sign = 1 if stream.coin() else -1
            diff[p] += sign
            diff[q] -= sign
        run = best = 0
        for n in diff:
            run += n
            best = max(best, abs(run))
        extreme += best >= observed
    return (1 + extreme) / (1 + rounds)


def close(a, b):
    return abs(a - b) <= 1e-9 * max(1.0, abs(a), abs(b))


def verify(receipt, draws, seed, pilot):
    failures = []

    def need(ok, why):
        if not ok:
            failures.append(why)

    need(receipt.get("kind") == "isocosm-probe", "not the full comparison probe")
    need(receipt.get("master_seed") == seed, "master seed differs from declaration")
    need(receipt.get("arms") == ARMS, "arms differ from declaration")
    settings = receipt["settings"]
    need(settings["seed"] == draw_seed(seed, "probe-check", []),
         "permutation settings seed differs from declaration")
    need(settings["alpha"] == 0.05 and settings["permutations"] == 1999,
         "alpha/permutations differ from declaration")
    domain = receipt["domain"]
    need(domain == DOMAIN, "founding domain differs from declaration")
    need(len(receipt["draws"]) == draws, "draw count differs from declaration")
    need(pilot or draws == 1000, "certification needs the declared 1000 draws")
    info = receipt["readings"]
    keys = [r["key"] for r in info]
    need(len(keys) == len(set(keys)), "duplicate reading keys")
    need(INJURY <= set(keys), "missing injury inspection readings")
    need(TRAITS <= set(keys), "missing optional trait readings")
    need(any(k.startswith("inspect:cells:") for k in keys), "missing function-cell readings")
    need(all(r["bound_per_mille"] == 200 for r in info), "reading bound changed")
    need(receipt["checks"]["exact_rerun_identical"], "draw0 exact rerun differs")
    need(receipt["checks"]["collect_changes_no_outcome"], "draw0 collection changes outcome")
    raw = []
    for k, draw in enumerate(receipt["draws"]):
        need(draw["k"] == k, f"draw order differs at {k}")
        need([a["arm"] for a in draw["arms"]] == ARMS, f"draw{k} arms differ")
        by = {a["arm"]: a for a in draw["arms"]}
        for arm, row in by.items():
            need(arm not in ARMS[:2] or not row.get("refused"), f"draw{k}/{arm} native refusal")
            if not row.get("refused"):
                need(len(row["readings"]) == len(keys), f"draw{k}/{arm} reading width")
                need(all(type(v) is int and v >= 0 for v in row["readings"]),
                     f"draw{k}/{arm} non-integer readings")
            if arm in ARMS[:2] and not row.get("refused"):
                matter = row["matter"]
                need(all(type(v) is int and v >= 0 for v in matter.values()),
                     f"draw{k}/{arm} invalid matter audit")
                need(matter["verified_ticks"] == domain["ticks"],
                     f"draw{k}/{arm} matter audit span differs")
                need(matter["final_total"] == matter["initial"] + matter["issued"],
                     f"draw{k}/{arm} matter not conserved")
                need(matter["issued"] == 0, f"draw{k}/{arm} undeclared dev assistance")
        need(by["exact"]["matter"]["initial"] == by["exact-control"]["matter"]["initial"],
             f"draw{k} native founding matter differs")
        raw.append(by)
    refused_by_arm = {arm: [k for k, d in enumerate(raw) if d[arm].get("refused")]
                      for arm in ARMS}
    need(receipt.get("refused", {}) == {a: v for a, v in refused_by_arm.items() if v},
         "top-level refusal indices differ")
    if failures:
        return {"pass": False, "certified": False, "failures": failures, "comparisons": []}
    checked = []
    comparisons = receipt["comparisons"]
    need([c["name"] for c in comparisons] == list(PAIRS), "comparisons differ")
    for c in comparisons:
        need(c["settings"] == settings, c["name"] + ": settings differ")
        left, right = PAIRS[c["name"]]
        kept = [(d[left], d[right]) for d in raw
                if not d[left].get("refused") and not d[right].get("refused")]
        refused = sum(bool(d[right].get("refused")) for d in raw)
        bounded = right == "crowd"
        within = not bounded or refused * 1000 <= 10 * len(raw)
        need(c["draws"] == len(kept), c["name"] + ": paired count differs")
        r = c["refusals"]
        need((r["refused"], r["of"], r["within"], r["bound_per_mille"]) ==
             (refused, len(raw), within, 10 if bounded else None),
             c["name"] + ": refusal summary differs")
        need([r["key"] for r in c["readings"]] == keys, c["name"] + ": reading order differs")
        if not kept:
            need(False, c["name"] + ": no paired draws")
            continue
        rows, pes, pds, stored_pds = [], [], [], []
        for i, row in enumerate(c["readings"]):
            a = [x["readings"][i] for x, _ in kept]
            b = [y["readings"][i] for _, y in kept]
            d = distance(a, b)
            pe = equivalence(d, 0.2, len(kept))
            permutation_seed = draw_seed(settings["seed"], "probe-permutation", [i])
            pd = permutation(a, b, 1999, permutation_seed)
            need(close(row["distance"], d) and row["bound"] == 0.2,
                 c["name"] + "/" + keys[i] + ": distance/bound differs")
            need(close(row["mean_a"], sum(a) / len(a)) and
                 close(row["mean_b"], sum(b) / len(b)),
                 c["name"] + "/" + keys[i] + ": means differ")
            need(close(row["p_equivalence"], pe), keys[i] + ": equivalence p differs")
            pes.append(pe)
            pds.append(pd)
            need(0 <= row["p_difference"] <= 1, keys[i] + ": invalid difference p")
            need(close(row["p_difference"], pd),
                 c["name"] + "/" + keys[i] + ": raw permutation p differs")
            stored_pds.append(row["p_difference"])
            rows.append({"key": keys[i], "distance": d,
                         "mean_a": sum(a) / len(a), "mean_b": sum(b) / len(b),
                         "p_difference": pd, "extreme": round(pd * 2000) - 1})
        for row, stored, pe, pd, stored_pd in zip(
                rows, c["readings"], holm(pes), holm(pds), holm(stored_pds)):
            certified, detected = pe <= 0.05, pd <= 0.05
            need(close(stored["p_equivalence_holm"], pe) and
                 stored["certified"] == certified, row["key"] + ": certification differs")
            need(stored["detected"] == detected, c["name"] + "/" + row["key"] +
                 ": independent permutation detection differs")
            need(close(stored["p_difference_holm"], pd) and
                 close(stored["p_difference_holm"], stored_pd) and
                 stored["detected"] == (stored_pd <= 0.05),
                 c["name"] + "/" + row["key"] + ": stored difference summary differs")
            row.update(p_equivalence_holm=pe, p_difference_holm=pd,
                       certified=certified, detected=detected)
        equivalent = all(r["certified"] for r in rows)
        different = any(r["detected"] for r in rows)
        passed = equivalent and not different and within
        need((c["equivalent"], c["different"], c["pass"]) ==
             (equivalent, different, passed), c["name"] + ": verdict differs")
        checked.append({"name": c["name"], "draws": len(kept), "refused": refused,
                        "within": within, "equivalent": equivalent,
                        "different": different, "pass": passed, "readings": rows})
    if len(checked) == 4:
        main, positive, negative, unweighted = checked
        starvation = {r["key"] for r in info if r["starvation"]}
        detected_starvation = any(r["detected"] for r in negative["readings"]
                                 if r["key"] in starvation)
        if not pilot:
            need(main["pass"], "crowd is not certified equivalent without detected differences")
            need(positive["pass"], "positive control did not pass")
            need(detected_starvation, "negative control did not detect starvation")
            need(unweighted["different"], "unweighted draw control was not detected")
            for arm in ["exact", "crowd"]:
                for key in ["inspect:lost-cells", "inspect:severed"]:
                    i = keys.index(key)
                    need(any(d[arm]["readings"][i] > 0 for d in raw
                             if not d[arm].get("refused")), arm + "/" + key + ": no injury observed")
        v = receipt["verdicts"]
        need(v["starvation_readings"] == [r["key"] for r in info if r["starvation"]],
             "top-level starvation columns differ")
        failed_starvation = any(r["detected"] or not r["certified"]
                                for r in negative["readings"] if r["key"] in starvation)
        need((v["main_equivalent"], v["main_different"],
              v["negative_control_failed_starvation_check"]) ==
             (main["equivalent"], main["different"], failed_starvation),
             "top-level equivalence/difference summary differs")
        need((v["main_pass"], v["positive_control_pass"],
              v["negative_control_detected_starvation_difference"], v["draw_control_detected"]) ==
             (main["pass"], positive["pass"], detected_starvation, unweighted["different"]),
             "top-level control verdicts differ")
    return {"pass": not failures, "certified": not pilot and not failures,
            "pilot": pilot, "failures": failures, "comparisons": checked}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("receipt", type=Path)
    parser.add_argument("--seed", type=int, default=2026101003)
    parser.add_argument("--draws", type=int, default=1000)
    parser.add_argument("--pilot", action="store_true")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    raw = args.receipt.read_bytes()
    try:
        result = verify(json.loads(raw), args.draws, args.seed, args.pilot)
    except (KeyError, TypeError, ValueError, ZeroDivisionError, IndexError) as error:
        result = {"pass": False, "certified": False, "failures": [str(error)]}
    result.update(receipt=str(args.receipt), receipt_sha256=hashlib.sha256(raw).hexdigest(),
                  verifier_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    if args.output:
        args.output.write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps({k: v for k, v in result.items() if k != "comparisons"}, indent=2))
    return 0 if result["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
