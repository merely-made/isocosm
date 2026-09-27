"""Rebuild c5-draws.json from checkpoint 5's two raw draw runs.

usage: python c5_summary.py <c5-draws-parts.jsonl> <c5-draws-flows.jsonl> <out.json>

Each raw run is one JSON line a draw and a summary line last, as
c5-draws.rs writes them. This recounts every draw rather than trusting the
summary line, and checks the two agree."""
import json, sys


def lines(path):
    rows = [json.loads(l) for l in open(path, encoding="utf-8") if l.strip()]
    return rows[:-1], rows[-1]


parts, parts_summary = lines(sys.argv[1])
flows, flows_summary = lines(sys.argv[2])

bound = {}
for d in parts:
    bound[d["slice"]] = bound.get(d["slice"], 0) + d["bound"]
parts_out = {
    "master_seed": parts_summary["master"],
    "draws": len(parts),
    "modes_agree": sum(d["agree"] for d in parts),
    "functions_per_draw": [min(d["functions"] for d in parts), max(d["functions"] for d in parts)],
    "functions_bound_per_draw": [min(d["bound"] for d in parts), max(d["bound"] for d in parts)],
    "functions_never_bound": parts_summary["functions_never_bound"],
    "times_each_function_bound": [parts_summary["least_bound"], parts_summary["most_bound"]],
    "draws_with_marks_read_back": sum(d["rested"] > 0 for d in parts),
    "members_represented": sum(d["represented"] for d in parts),
    "evaluations_individual": sum(d["evaluations_individual"] for d in parts),
    "evaluations_grouped": sum(d["evaluations_grouped"] for d in parts),
}
assert parts_out["modes_agree"] == parts_summary["agreed"]
assert parts_out["evaluations_individual"] == parts_summary["evaluations_individual"]

made = set()
for d in flows:
    made.update(d["made_by"])
flows_out = {
    "master_seed": flows_summary["master"],
    "draws": len(flows),
    "steps_per_draw": flows_summary["steps_per_draw"],
    "modes_agree_every_step": sum(d["agree"] for d in flows),
    "every_ledger_reconciled_every_step": sum(d["reconciled"] for d in flows),
    "matter_founded_plus_issued_every_step": sum(d["conserved"] for d in flows),
    "flows_individual": sum(d["flows_individual"] for d in flows),
    "flows_grouped": sum(d["flows_grouped"] for d in flows),
    "cohort_flows": sum(d["cohort_flows"] for d in flows),
    "draws_with_cohort_flows": sum(d["cohort_flows"] > 0 for d in flows),
    "issued_per_draw": [min(int(d["issued"]) for d in flows), max(int(d["issued"]) for d in flows)],
    "made_by": sorted(made),
    "unreconciled": [u for d in flows for u in d["unreconciled"]],
}
assert flows_out["modes_agree_every_step"] == flows_summary["agreed"]
assert flows_out["every_ledger_reconciled_every_step"] == flows_summary["reconciled"]
assert flows_out["cohort_flows"] == flows_summary["cohort_flows"]

out = {"kind": "isocosm-checkpoint-5-draws", "parts": parts_out, "flows": flows_out}
json.dump(out, open(sys.argv[3], "w", encoding="utf-8"), indent=1)
print(json.dumps(out, indent=1))
