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
ARM_INDEX = dict(zip(ARMS, [0, 1, 2, 3, 5]))
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


def require(ok, why):
    if not ok:
        raise ValueError(why)


def exact(a, b):
    """JSON equality must not admit bool/float substitutes for native integers."""
    if type(a) is not type(b):
        return False
    if type(b) is dict:
        return a.keys() == b.keys() and all(exact(a[k], b[k]) for k in b)
    if type(b) is list:
        return len(a) == len(b) and all(exact(x, y) for x, y in zip(a, b))
    return a == b


def uint(value, why, bits=64):
    require(type(value) is int and 0 <= value < (1 << bits), why)


def decode(raw):
    def record(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON field")
            result[key] = value
        return result
    def nonfinite(value):
        raise ValueError("nonfinite JSON number: " + value)
    return json.loads(raw, object_pairs_hook=record, parse_constant=nonfinite)


def validate_shape(receipt, draws, seed, pilot):
    """Cheap native wire/declaration checks, before any permutation work."""
    uint(seed, "invalid declared master seed")
    uint(draws, "invalid declared draw count")
    require(draws > 0 and type(pilot) is bool and (pilot or draws == 1000),
            "certification needs the declared 1000 draws; pilot must be explicit")
    require(type(receipt) is dict, "receipt is not a record")
    require(exact(receipt.get("version"), 1), "unsupported native probe version")
    require(receipt.get("kind") == "isocosm-probe", "not the full comparison probe")
    require(exact(receipt.get("master_seed"), seed), "master seed differs from declaration")
    require(exact(receipt.get("arms"), ARMS), "arms differ from declaration")
    settings = dict(seed=draw_seed(seed, "probe-check", []), alpha=0.05, permutations=1999)
    require(exact(receipt.get("settings"), settings), "statistical settings/types differ")
    require(exact(receipt.get("domain"), DOMAIN), "founding domain/types differ")
    require(exact(receipt.get("checks"), dict(exact_rerun_identical=True,
                 collect_changes_no_outcome=True)), "native replay/collection checks differ")
    read_set = receipt["read_set"]
    require(type(read_set) is list and all(type(key) is str and key for key in read_set)
            and read_set == sorted(set(read_set)), "invalid native read-set identities/order")
    info = receipt["readings"]
    require(type(info) is list and bool(info), "reading metadata is empty or not a list")
    keys = []
    for row in info:
        require(type(row) is dict and type(row.get("key")) is str and bool(row["key"])
                and type(row.get("source")) is str and bool(row["source"])
                and type(row.get("probe")) is dict and bool(row["probe"]), "invalid reading identity/source")
        require(type(row.get("starvation")) is bool and exact(row.get("bound_per_mille"), 200),
                "reading flag/bound types differ")
        keys.append(row["key"])
    require(len(keys) == len(set(keys)), "duplicate reading keys")
    require(INJURY <= set(keys) and TRAITS <= set(keys)
            and any(k.startswith("inspect:cells:") for k in keys), "missing declared harm readings")
    native_draws = receipt["draws"]
    require(type(native_draws) is list and len(native_draws) == draws, "draw count differs")
    refused = {arm: [] for arm in ARMS}
    for k, draw in enumerate(native_draws):
        require(type(draw) is dict and exact(draw.get("k"), k), "draw order/type differs")
        require(exact(draw.get("seed"), draw_seed(seed, "probe-world", [k])), "world seed differs")
        for field in ["sites", "members", "margin", "cost", "upset", "advantage"]:
            uint(draw[field], f"draw{k}/{field}: invalid native counter")
        require(type(draw["arms"]) is list and len(draw["arms"]) == len(ARMS)
                and all(type(a) is dict for a in draw["arms"]), "draw arm count/type differs")
        require([a.get("arm") for a in draw["arms"]] == ARMS, "draw arm identity/order differs")
        native_matter = []
        for arm, row in zip(ARMS, draw["arms"]):
            require(type(row) is dict, "arm is not a record")
            require(exact(row.get("dynamics"), draw_seed(seed, "probe-dynamics", [k, ARM_INDEX[arm]])),
                    f"draw{k}/{arm}: dynamics seed differs")
            for field in ["micros", "evaluations", "represented", "accepted", "blocked", "stored",
                          "alive", "alive_states"]:
                uint(row[field], f"draw{k}/{arm}/{field}: invalid native counter")
            if "shortfalls" in row:
                uint(row["shortfalls"], "invalid shortfall count")
            why = row.get("refused")
            require(why is None or (type(why) is str and bool(why)), "invalid refusal representation")
            require(type(row["readings"]) is list, "arm readings are not a list")
            if why is not None:
                require(arm not in ARMS[:2] and row["readings"] == [] and row.get("matter") is None,
                        "native refusal or nonempty refused arm")
                refused[arm].append(k)
                continue
            require(len(row["readings"]) == len(keys), "arm reading width differs")
            for value in row["readings"]:
                uint(value, "invalid native reading scalar")
            if arm in ARMS[:2]:
                matter = row["matter"]
                require(type(matter) is dict and set(matter) == {"initial", "final_total", "issued", "verified_ticks"},
                        "native matter audit fields differ")
                for field in ["initial", "final_total", "issued"]:
                    uint(matter[field], "invalid native matter scalar", bits=128)
                uint(matter["verified_ticks"], "invalid verified tick scalar")
                require(matter["verified_ticks"] == DOMAIN["ticks"] and matter["issued"] == 0
                        and matter["final_total"] == matter["initial"] + matter["issued"],
                        "native matter conservation/span/issuance differs")
                native_matter.append(matter["initial"])
        require(len(native_matter) == 2 and native_matter[0] == native_matter[1], "native founding totals differ")
    require(exact(receipt.get("refused", {}), {a: v for a, v in refused.items() if v}), "refusal indices/types differ")
    comparisons = receipt["comparisons"]
    require(type(comparisons) is list and len(comparisons) == len(PAIRS)
            and all(type(c) is dict for c in comparisons), "comparison count/type differs")
    require([c.get("name") for c in comparisons] == list(PAIRS), "comparison identity/order differs")
    for comparison in comparisons:
        name = comparison["name"]
        right = PAIRS[name][1]
        kept = draws - len(refused[right])
        require(exact(comparison["settings"], settings) and exact(comparison["draws"], kept),
                "comparison settings/count types differ")
        bounded = right == "crowd"
        within = not bounded or len(refused[right]) * 1000 <= 10 * draws
        require(exact(comparison["refusals"], dict(refused=len(refused[right]), of=draws,
                    within=within, bound_per_mille=10 if bounded else None)), "comparison refusal summary differs")
        require(type(comparison["readings"]) is list and len(comparison["readings"]) == len(keys)
                and all(type(r) is dict for r in comparison["readings"])
                and [r.get("key") for r in comparison["readings"]] == keys, "comparison reading count/order differs")
        for field in ["equivalent", "different", "pass"]:
            require(type(comparison[field]) is bool, "invalid comparison flag type")
        for row in comparison["readings"]:
            for field in ["distance", "bound", "mean_a", "mean_b", "p_difference", "p_difference_holm",
                          "p_equivalence", "p_equivalence_holm"]:
                require(type(row[field]) is float and math.isfinite(row[field]) and row[field] >= 0,
                        "invalid native statistical scalar")
            require(row["bound"] == 0.2, "native comparison bound differs")
            for field in ["distance", "p_difference", "p_difference_holm", "p_equivalence", "p_equivalence_holm"]:
                require(row[field] <= 1, "native probability/distance outside bounds")
            require(type(row["certified"]) is bool and type(row["detected"]) is bool, "invalid reading verdict types")
    verdicts = receipt["verdicts"]
    require(type(verdicts) is dict, "verdicts are not a record")
    for field in ["main_pass", "main_equivalent", "main_different", "positive_control_pass",
                  "negative_control_failed_starvation_check", "negative_control_detected_starvation_difference",
                  "draw_control_detected"]:
        require(type(verdicts[field]) is bool, "invalid top-level verdict flag")
    require(exact(verdicts["starvation_readings"], [r["key"] for r in info if r["starvation"]]),
            "top-level starvation columns differ")


def verify(receipt, draws, seed, pilot):
    try:
        validate_shape(receipt, draws, seed, pilot)
    except (KeyError, TypeError, ValueError, IndexError, OverflowError) as error:
        return {"pass": False, "certified": False, "failures": [str(error)], "comparisons": []}
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
    if args.output and args.output.exists():
        parser.error("output already exists; use a fresh receipt name")
    raw = args.receipt.read_bytes()
    try:
        result = verify(decode(raw), args.draws, args.seed, args.pilot)
    except (KeyError, TypeError, ValueError, ZeroDivisionError, IndexError) as error:
        result = {"pass": False, "certified": False, "failures": [str(error)]}
    result.update(receipt=str(args.receipt), receipt_sha256=hashlib.sha256(raw).hexdigest(),
                  verifier_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    if args.output:
        with args.output.open("x", encoding="utf-8") as output:
            output.write(json.dumps(result, indent=2))
    print(json.dumps({k: v for k, v in result.items() if k != "comparisons"}, indent=2))
    return 0 if result["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
