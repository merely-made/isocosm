"""Checkpoint 10 controls. Run only while owning the queue's Cargo grant.

One source mutation at a time; compilation must succeed and the specified
test must fail. Original bytes are restored even if the runner is interrupted.
The JSON result records source hashes and logs, not just a failing exit code.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


FAULTS = [
    ("fragment duplicates parent matter (707, 715)", "src/harm/fragments.rs",
     "copied.matter = std::mem::take(&mut p.matter);", "copied.matter = p.matter.clone();",
     "severing_keeps_matter_in_site_fragment_and_empty_tombstones"),
    ("severing fails to reroot the fragment (714)", "src/harm/fragments.rs",
     "g.situs = Some([0; 3]);", "g.situs = g.situs;",
     "severing_keeps_matter_in_site_fragment_and_empty_tombstones"),
    ("fragment ignores its lineage trait (713)", "src/harm/fragments.rs",
     "child.alive = alive;", "child.alive = true;",
     "severing_keeps_matter_in_site_fragment_and_empty_tombstones"),
    ("a wound leaves matter over its lowered bound (708)", "src/harm.rs",
     "if over > 0 {", "if false && over > 0 {",
     "a_wound_spills_all_accounts_over_the_lowered_bound_without_changing_geometry"),
    ("part draw counts parts instead of living cells (716)", "src/harm.rs",
     "let n = u64::from(anatomy::living_cells(e.extent(id), p));", "let n = 1;",
     "a_part_draw_follows_living_cells_instead_of_part_count"),
    ("hazard zero still wounds (704)", "src/probe/found/body/harm.rs",
     "Box::new(c(rate - 1)),", "Box::new(c(63)),",
     "hazard_zero_wounds_nothing_in_native_and_crowd_runs"),
    ("rot silently drops its site deposit (718)", "src/harm/rot.rs",
     "credit(accounts, key, *n)?;", "credit(accounts, key, 0)?;",
     "rot_returns_every_dead_part_account_to_the_site"),
    ("admission hides matter in tombstones", "src/validation/body.rs",
     "if !e.lives(id) && part.matter.values().any(|n| *n > 0) {",
     "if false && !e.lives(id) && part.matter.values().any(|n| *n > 0) {",
     "admission_refuses_matter_hidden_in_a_tombstone"),
    ("fragment flow names the wrong source part", "src/harm/fragments.rs",
     "from: (Holder::Part(parent, *old), key.clone()),",
     "from: (Holder::Part(parent, PartId(0)), key.clone()),",
     "severing_keeps_matter_in_site_fragment_and_empty_tombstones"),
    ("crowd permits competing hazard actors without pass-start targets", "src/probe/crowd/harm.rs",
     "if counts.values().any(|n| *n > 1) {", "if false && counts.values().any(|n| *n > 1) {",
     "crowd_tests::multiple_world_actors_at_one_site_are_refused"),
    ("healing ignores its lineage trait (485)", "src/meaning/heal.rs",
     "if !p.lineage_trait(&lineage, HEAL) {", "if false && !p.lineage_trait(&lineage, HEAL) {",
     "healing_tests::a_lineage_without_healing_keeps_its_recorded_losses"),
    ("healing restores cells for free (496, 810)", "src/meaning/heal.rs",
     "let price = u64::from(anatomy::cell_mass(body.extent(id), rules.body()));", "let price = 0;",
     "healing_tests::healing_restores_recorded_cells_and_pays_reserve_before_tissue"),
    ("healing discards the recorded function (810)", "src/harm/healing.rs",
     "if let Some(function) = allocation {", "if let Some(function) = None::<Key> {",
     "healing_tests::healing_restores_recorded_cells_and_pays_reserve_before_tissue"),
    ("regrowth leaves the original tombstone severed (719)", "src/harm/healing.rs",
     ".revive(id)", ".is_living(PartId(0))",
     "healing_tests::one_paid_cell_revives_only_the_top_tombstone"),
    ("unknown losses block known healing (811)", "src/harm/healing.rs",
     ".find(|c| p.losses.contains_key(c));", ".find(|_| true);",
     "healing_tests::unknown_legacy_losses_do_not_block_known_healing_in_the_same_body"),
    ("parent tombstone descendants keep unpaid cells (711, 719)", "src/harm/fragments.rs",
     "super::take(p, half, u32::MAX);", "super::take(p, half, 0);",
     "healing_tests::tombstones_revive_in_place_without_reclaiming_fragment_matter"),
    ("fed landing fills tissue before healing (712)", "src/probe/found/body/physiology.rs",
     "vec![healed(), tissue_in()]", "vec![tissue_in(), healed()]",
     "healing_tests::generated_landing_heals_before_tissue_and_after_a_hungry_reserve"),
    ("hungry landing heals before reserve (712)", "src/probe/found/body/physiology.rs",
     "vec![reserve_in(), healed(), tissue_in()]", "vec![healed(), reserve_in(), tissue_in()]",
     "healing_tests::generated_landing_heals_before_tissue_and_after_a_hungry_reserve"),
    ("a full wounded producer cannot carry healing income", "src/meaning/carriage.rs",
     "if heals && !lands.is_empty() {", "if false && heals && !lands.is_empty() {",
     "healing_tests::a_full_wounded_producer_draws_healing_income_through_its_working_route"),
]


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def audit(root, index):
    name, relative, before, after, test = FAULTS[index]
    original = (root / relative).read_bytes()
    anchors = original.count(before.encode())
    tests = [root / "tests/harm.rs", *(root / "tests/harm").glob("*.rs")]
    pattern = rf"\bfn\s+{re.escape(test.split('::')[-1])}\s*\("
    matches = [str(p.relative_to(root)) for p in tests
               if re.search(pattern, p.read_text(encoding="utf-8"))]
    return {"index": index, "fault": name, "source": relative, "test": test,
            "original_sha256": sha(original), "anchors": anchors,
            "test_sources": matches,
            "verdict": "SOURCE-READY" if anchors == 1 and len(matches) == 1 else "INVALID",
            "compiled": False, "mutated": False}


def run(root, index):
    name, relative, before, after, test = FAULTS[index]
    source = root / relative
    original = source.read_bytes()
    anchor = before.encode()
    if original.count(anchor) != 1:
        raise ValueError(f"{name}: anchor is not unique in {relative}")
    mutant = original.replace(anchor, after.encode())
    command = ["cargo", "test", "--offline", "-j1", "--test", "harm", test, "--", "--exact"]
    result = {"index": index, "fault": name, "source": relative, "test": test,
              "original_sha256": sha(original), "mutant_sha256": sha(mutant), "command": command}
    try:
        source.write_bytes(mutant)
        trial = subprocess.run(command, cwd=root, capture_output=True, text=True)
        result.update(exit_code=trial.returncode, stdout=trial.stdout, stderr=trial.stderr)
        compiled = ("Finished `test`" in trial.stderr and "error[" not in trial.stderr
                    and "could not compile" not in trial.stderr)
        caught = (trial.returncode != 0 and "running 1 test" in trial.stdout
                  and "FAILED" in trial.stdout and "1 failed" in trial.stdout)
        result.update(compiled=compiled, caught=caught)
        result["verdict"] = "CAUGHT" if compiled and caught else "MISSED" if compiled else "NO-COMPILE"
    finally:
        source.write_bytes(original)
        if source.read_bytes() != original:
            raise RuntimeError(f"failed to restore {source}")
    result["restored_sha256"] = sha(source.read_bytes())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path)
    parser.add_argument("--audit", action="store_true", help="read source anchors/tests without Cargo or mutation")
    parser.add_argument("picks", nargs="*", type=int)
    args = parser.parse_args()
    root = args.workspace.resolve()
    if not (root / "src/harm.rs").is_file() or not (root / "Cargo.toml").is_file():
        parser.error("--workspace must name the assigned shared/isocosm workspace")
    results = []
    for index in args.picks or range(len(FAULTS)):
        if index not in range(len(FAULTS)):
            parser.error(f"unknown fault index {index}")
        result = audit(root, index) if args.audit else run(root, index)
        results.append(result)
        print(f"{index}: {result['verdict']} {result['fault']}", flush=True)
        if args.output:
            args.output.write_text(json.dumps(results, indent=2), encoding="utf-8")
    expected = "SOURCE-READY" if args.audit else "CAUGHT"
    return 0 if all(r["verdict"] == expected for r in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
