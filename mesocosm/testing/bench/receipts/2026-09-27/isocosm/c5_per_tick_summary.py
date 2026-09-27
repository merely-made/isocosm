"""Recount ruling 371's per-tick draws and compare the historical end states.

Usage: python c5_per_tick_summary.py NEW.jsonl HISTORICAL.jsonl OUT.json
The raw files stay outside the repository. No summary-line success is accepted
without recounting each draw. The old file is read, never changed.
"""
import hashlib
import json
import pathlib
import sys


def read(path):
    path = pathlib.Path(path)
    rows = [json.loads(line) for line in path.read_text().splitlines() if line]
    return rows[:-1], rows[-1], hashlib.sha256(path.read_bytes()).hexdigest()


fresh, summary, fresh_hash = read(sys.argv[1])
old, old_summary, old_hash = read(sys.argv[2])
assert len(fresh) == len(old) == summary["draws"] == old_summary["draws"]
assert summary["master"] == old_summary["master"]
counts = {
    "draws": len(fresh),
    "modes_agree_every_handoff": sum(row["agree"] for row in fresh),
    "every_ledger_reconciled_every_handoff": sum(row["reconciled"] for row in fresh),
    "matter_founded_plus_issued_every_handoff": sum(row["conserved"] for row in fresh),
    "handoffs_per_mode": sum(row["handoffs_per_mode"] for row in fresh),
    "tick_controls_both_modes": sum(row["tick_controls"] for row in fresh),
    "draws_with_tick_controls": sum(row["tick_controls"] > 0 for row in fresh),
    "flows_individual": sum(row["flows_individual"] for row in fresh),
    "flows_grouped": sum(row["flows_grouped"] for row in fresh),
    "cohort_flows": sum(row["cohort_flows"] for row in fresh),
}
for field, reported in [
    ("modes_agree_every_handoff", "agreed"),
    ("every_ledger_reconciled_every_handoff", "reconciled"),
    ("matter_founded_plus_issued_every_handoff", "conserved"),
    ("handoffs_per_mode", "handoffs_per_mode"),
    ("tick_controls_both_modes", "tick_controls"),
    ("cohort_flows", "cohort_flows"),
]:
    assert counts[field] == summary[reported], (field, counts[field], summary[reported])
for field in [
    "modes_agree_every_handoff",
    "every_ledger_reconciled_every_handoff",
    "matter_founded_plus_issued_every_handoff",
    "draws_with_tick_controls",
]:
    assert counts[field] == len(fresh), field
assert not any(row["unreconciled"] for row in fresh)
changed_flow_counts = []
for now, before in zip(fresh, old):
    for field in ["draw", "seed", "state_hash", "issued", "made_by"]:
        assert now[field] == before[field], (now["draw"], field)
    if any(now[k] != before[k] for k in ["flows_individual", "flows_grouped", "cohort_flows"]):
        changed_flow_counts.append(now["draw"])
out = {
    "kind": "isocosm-checkpoint-5-per-tick-handoff",
    "ruling": 371,
    "master_seed": summary["master"],
    "drawn_steps_per_world": summary["steps_per_draw"],
    **counts,
    "historical_final_states_and_issue_match": len(fresh),
    "draws_with_changed_flow_counts": len(changed_flow_counts),
    "flow_count_reading": "Each explicit one-tick transaction applies the existing cohort collection at its end. Cohort batching may differ from the historical one-to-three-tick calls; every per-member ledger reconciles and final states/issued matter match.",
    "fresh_raw_sha256": fresh_hash,
    "historical_raw_sha256": old_hash,
    "scope": "Per-tick/command handoff and ledger reconciliation. Reserve physiology, filial development cost and checkpoint 6 are outside this receipt.",
}
pathlib.Path(sys.argv[3]).write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
print(json.dumps(out, indent=2))
