"""Derive the scale analysis from raw isocosm-scale receipts.

usage: python analyze.py analysis.json scale.json [scale-extension.json]

Every number here is computed from the raw receipts. The extension adds two
larger rungs per ladder with 8 ticks each, so the combined fits use every
ladder point's mean over its first 8 ticks. Extrapolations use only those
fits, and each carries its fit.
"""
import json
import math
import statistics
import sys
from collections import defaultdict

raws = [json.load(open(path)) for path in sys.argv[2:]]
points = [p for r in raws for p in r["points"]]
out = {
    "sources": sys.argv[2:],
    "master_seeds": [r["master_seed"] for r in raws],
    "heap_controls": [r["heap_control"] for r in raws],
}
LADDER = ("ladder", "extension")
TARGETS = [10_000, 20_000, 50_000, 100_000, 1_000_000]


def mean(xs):
    xs = list(xs)
    return sum(xs) / len(xs) if xs else 0.0


def first8(p):
    return mean(r["micros"] for r in p["per_tick"][:8]) / 1e6


def ols(pairs):
    xs, ys = [p[0] for p in pairs], [p[1] for p in pairs]
    mx, my = mean(xs), mean(ys)
    sxx = sum((x - mx) ** 2 for x in xs)
    syy = sum((y - my) ** 2 for y in ys)
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    b = sxy / sxx if sxx else 0.0
    r2 = (sxy * sxy / (sxx * syy)) if sxx and syy else None
    return {"intercept": my - b * mx, "slope": b, "r_squared": r2, "points": len(pairs)}


def power(samples):
    """quantity = coefficient * size ** exponent, least squares on logarithms."""
    f = ols([(math.log(n), math.log(v)) for n, v in samples])
    a = math.exp(f["intercept"])
    return {"exponent": f["slope"], "coefficient": a, "r_squared": f["r_squared"],
            "predicted": [(t, a * t ** f["slope"]) for t in TARGETS]}


# Ladder tables, one row per size, every draw shown.
ladder = defaultdict(lambda: defaultdict(list))
for p in points:
    if p["run"] in LADDER and p["per_tick"]:
        ladder[(p["family"], p["mode"])][p["founding"]["population"]].append(p)

tables = []
for (fam, mode), sizes in sorted(ladder.items()):
    rows, prev = [], None
    for n in sorted(sizes):
        ps = sizes[n]
        last = [p["per_tick"][-1] for p in ps]
        row = {
            "population": n,
            "runs": [p["run"] for p in ps],
            "ticks_run": [p["ticks_run"] for p in ps],
            "mean_tick_seconds": [round(p["mean_tick_micros"] / 1e6, 4) for p in ps],
            "first8_mean_tick_seconds": [round(first8(p), 4) for p in ps],
            "evaluations_per_tick": [p["evaluations_per_tick"] for p in ps],
            "micros_per_evaluation": [round(p["mean_tick_micros"] / p["evaluations_per_tick"], 1) for p in ps],
            "heap_peak_mib": [round(p["heap_peak_bytes"] / 2**20, 1) for p in ps],
            "heap_live_end_mib": [round(p["heap_live_end_bytes"] / 2**20, 1) for p in ps],
            "groups_end": [r["groups"] for r in last],
            "alive_end": [r["alive"] for r in last],
            "stored_members_end": [r["stored_members"] for r in last],
            "distinct_states_end": [p["distinct_states"] for p in ps],
            "distinct_read_states_end": [p["distinct_read_states"] for p in ps],
            # Stored members (alive, dead and the world's site bodies) per
            # distinct read state: the most an exact histogram keyed by what
            # the family's processes read could fold them.
            "stored_members_per_read_state": [round(r["stored_members"] / p["distinct_read_states"], 2) for p, r in zip(ps, last)],
            "events_end": [r["events"] for r in last],
            "clone_micros": [p["components"]["clone_micros"] for p in ps],
            "matter_micros": [p["components"]["matter_micros"] for p in ps],
            "state_hash_micros": [p["components"]["state_hash_micros"] for p in ps],
            "stopped": [p["stopped"] for p in ps if p["stopped"]],
        }
        m = mean(first8(p) for p in ps)
        if prev:
            row["local_exponent_first8"] = round(math.log(m / prev[1]) / math.log(n / prev[0]), 2)
        prev = (n, m)
        rows.append(row)
    tables.append({"family": fam, "mode": mode, "rows": rows})
out["ladders"] = tables

# The same drawn world run in both modes must reach the same state.
hashes = defaultdict(dict)
for p in points:
    if p["run"] in LADDER:
        hashes[(p["family"], p["founding"]["seed"], p["founding"]["population"])][p["mode"]] = (p["final_hash"], p["ticks_run"])
pairs = [v for v in hashes.values() if len(v) == 2]
out["mode_hash_check"] = {
    "pairs": len(pairs),
    "identical_final_state": sum(1 for v in pairs if v["grouped"] == v["individuals"]),
}

out["fits_recorded_by_binary"] = [dict(f, source=i) for i, r in enumerate(raws) for f in r["fits"]]

# Combined fits over ladder and extension, on first-8-tick means.
combined = []
for (fam, mode), sizes in sorted(ladder.items()):
    ns = sorted(sizes)
    for basis, chosen in (("all", ns), ("top3", ns[-3:])):
        samples = [(n, first8(p)) for n in chosen for p in sizes[n]]
        if len({n for n, _ in samples}) >= 2:
            combined.append(dict(power(samples), family=fam, mode=mode, quantity="first8_mean_tick_seconds", sizes=chosen, basis=basis))
    heap = [(n, p["heap_peak_bytes"]) for n in ns for p in sizes[n] if p["heap_peak_bytes"] > 0]
    combined.append(dict(power(heap), family=fam, mode=mode, quantity="heap_peak_bytes", sizes=ns, basis="all"))
    evals = [(n, p["evaluations_per_tick"]) for n in ns for p in sizes[n]]
    combined.append(dict(power(evals), family=fam, mode=mode, quantity="evaluations_per_tick", sizes=ns, basis="all"))
out["fits"] = combined

# Cost per evaluation against stored groups: the intercept is the constant
# per evaluation, the slope the part that grows with the stored world (the
# staging clone, the matter check and target scans all walk every group).
# Individual mode only: in grouped mode members lifted during a tick merge
# back at its end, so end-of-tick groups understate what evaluations saw.
overhead = []
for (fam, mode), sizes in sorted(ladder.items()):
    if mode != "individuals":
        continue
    pts, per_group = [], []
    for ps in sizes.values():
        for p in ps:
            pts.append((mean(r["groups"] for r in p["per_tick"]), p["mean_tick_micros"] / p["evaluations_per_tick"]))
            g_end = p["per_tick"][-1]["groups"]
            per_group.append((p["components"]["clone_micros"] / g_end, p["components"]["matter_micros"] / g_end))
    fit = ols(pts)
    fit.update({
        "family": fam, "mode": mode,
        "clone_micros_per_group_median": statistics.median(c for c, _ in per_group),
        "matter_micros_per_group_median": statistics.median(m for _, m in per_group),
    })
    overhead.append(fit)
out["micros_per_evaluation_vs_groups"] = overhead

# Early event rate by size (ecology ladders, both modes), for the history projection.
rate = []
for mode in ("grouped", "individuals"):
    for n, ps in ladder.get(("ecology", mode), {}).items():
        for p in ps:
            rows8 = p["per_tick"][:8]
            if len(rows8) == 8 and rows8[-1]["events"] > 0:
                rate.append((n, rows8[-1]["events"] / 8))
if len({n for n, _ in rate}) >= 2:
    out["ecology_event_rate_fit"] = dict(power(rate), quantity="events per tick over the first 8 ticks", samples=rate)

# Lineage sweep.
sweep = defaultdict(list)
for p in points:
    if p["run"] == "lineages":
        sweep[p["family"]].append(p)
out["lineage_sweep"] = {
    fam: [
        {
            "lineages": p["founding"]["lineages"],
            "mean_tick_seconds": round(p["mean_tick_micros"] / 1e6, 4),
            "evaluations_per_tick": p["evaluations_per_tick"],
            "evaluations_per_member_tick": round(p["evaluations_per_tick"] / p["founding"]["population"], 2),
            "blocked_share": round(sum(r["blocked"] for r in p["per_tick"]) / max(1, sum(r["represented"] for r in p["per_tick"])), 3),
        }
        for p in sorted(ps, key=lambda p: p["founding"]["lineages"])
    ]
    for fam, ps in sweep.items()
}

# History runs: growth with accumulated history at a fixed founding size.
# "history" founds cohorts of 32 (one site each); "living" places members one
# by one.
def history(h):
    ticks = h["per_tick"]
    xs = [r["tick"] for r in ticks]

    def slope(key, scale=1.0):
        return ols([(x, r[key] * scale) for x, r in zip(xs, ticks)])["slope"]

    window = max(1, len(ticks) // 8)
    return {
        "founding": h["founding"],
        "ticks_run": h["ticks_run"],
        "stopped": h["stopped"],
        "window_ticks": window,
        "first_window_mean_tick_seconds": round(mean(r["micros"] for r in ticks[:window]) / 1e6, 4),
        "last_window_mean_tick_seconds": round(mean(r["micros"] for r in ticks[-window:]) / 1e6, 4),
        "tick_seconds_growth_per_tick": slope("micros", 1e-6),
        "events_per_tick": slope("events"),
        "notes_per_tick": slope("notes"),
        "arrivals_per_tick": slope("arrivals"),
        "stored_members_per_tick": slope("stored_members"),
        "groups_per_tick": slope("groups"),
        "heap_bytes_per_tick": slope("heap_live_bytes"),
        "heap_bytes_per_event": ols([(r["events"], r["heap_live_bytes"]) for r in ticks])["slope"],
        "arrivals_per_event": ticks[-1]["arrivals"] / ticks[-1]["events"] if ticks[-1]["events"] else None,
        "alive_first_last": [ticks[0]["alive"], ticks[-1]["alive"]],
        "distinct_states_end": h["distinct_states"],
        "distinct_read_states_end": h["distinct_read_states"],
        "stored_members_per_read_state_end": round(ticks[-1]["stored_members"] / h["distinct_read_states"], 3),
        "components_end": h["components"],
        "births": ticks[-1]["stored_members"] - ticks[0]["stored_members"],
        "end": ticks[-1],
        "samples_every_32_ticks": ticks[31::32],
    }


for tag in ("history", "living"):
    found = [p for p in points if p["run"] == tag]
    if found:
        out[tag] = history(found[0])

out["largest"] = [
    {
        "family": p["family"],
        "founding": p["founding"],
        "generate_seconds": round(p["generate_micros"] / 1e6, 3),
        "heap_live_mib": round(p["heap_live_end_bytes"] / 2**20, 1),
        "heap_peak_mib": round(p["heap_peak_bytes"] / 2**20, 1),
        "clone_micros": p["components"]["clone_micros"],
        "matter_micros": p["components"]["matter_micros"],
        "state_hash_micros": p["components"]["state_hash_micros"],
        "distinct_states": p["distinct_states"],
    }
    for p in points
    if p["run"] == "largest"
]

# Region gap, ruling 124. The core has no calendar, so a century is shown for
# several tick lengths; "a few minutes" is taken as 300 s for the arithmetic.
per_century = {"year": 100, "month": 1200, "week": 5218, "day": 36525}
budget = 300.0
gap = []
for f in combined:
    if f["quantity"] != "first8_mean_tick_seconds" or f["basis"] != "top3":
        continue
    for n, tick in f["predicted"]:
        if n in (10_000, 50_000):
            gap.append({
                "family": f["family"], "mode": f["mode"], "population": n,
                "fit": {k: f[k] for k in ("exponent", "coefficient", "r_squared", "sizes")},
                "predicted_mean_tick_seconds": tick,
                "century_seconds_by_tick_length": {k: tick * v for k, v in per_century.items()},
                "gap_factor_vs_300s": {k: tick * v / budget for k, v in per_century.items()},
            })
out["region_gap"] = {
    "assumptions": "History growth ignored, so these are lower bounds on time; 256 sites and 8 lineages as measured; a century is 100, 1,200, 5,218 or 36,525 ticks; 300 s stands for a few minutes; fits are the top-three-size fits on first-8-tick means.",
    "required_mean_tick_seconds": {k: budget / v for k, v in per_century.items()},
    "rows": gap,
}

json.dump(out, open(sys.argv[1], "w"), indent=2)

for t in tables:
    print(f"\n{t['family']} {t['mode']}")
    for r in t["rows"]:
        print(f"  n={r['population']:>7} runs={r['runs']} tick={r['mean_tick_seconds']} first8={r['first8_mean_tick_seconds']} evals={r['evaluations_per_tick']} us/eval={r['micros_per_evaluation']} heap={r['heap_peak_mib']}MiB groups={r['groups_end']} stored={r['stored_members_end']} alive={r['alive_end']} distinct={r['distinct_states_end']}/{r['distinct_read_states_end']} per_state={r['stored_members_per_read_state']} exp={r.get('local_exponent_first8')} stopped={r['stopped']}")
print("\nmode hash check", out["mode_hash_check"])
for f in combined:
    print(f"fit {f['family']} {f['mode']} {f['quantity']} {f['basis']}: exp={f['exponent']:.3f} r2={f['r_squared']:.4f} sizes={f['sizes']}")
    print("   predicted", [(n, f"{v:.4g}") for n, v in f["predicted"]])
print("\nper-evaluation cost vs groups", json.dumps(overhead, indent=1))
print("\necology event rate fit", json.dumps(out.get("ecology_event_rate_fit"), indent=1))
print("\nlineage sweep", json.dumps(out["lineage_sweep"], indent=1))
for tag in ("history", "living"):
    if tag in out:
        print(f"\n{tag}", json.dumps({k: v for k, v in out[tag].items() if k not in ("end", "samples_every_32_ticks")}, indent=1))
        print(f"{tag} end", out[tag]["end"])
        for r in out[tag]["samples_every_32_ticks"]:
            print("  ", r)
print("\nlargest", json.dumps(out["largest"], indent=1))
print("\nregion gap required tick", out["region_gap"]["required_mean_tick_seconds"])
for g in gap:
    print(f"  {g['family']} {g['mode']} n={g['population']}: tick={g['predicted_mean_tick_seconds']:.4g}s gap(day)={g['gap_factor_vs_300s']['day']:.3g} gap(week)={g['gap_factor_vs_300s']['week']:.3g} gap(year)={g['gap_factor_vs_300s']['year']:.3g}")
