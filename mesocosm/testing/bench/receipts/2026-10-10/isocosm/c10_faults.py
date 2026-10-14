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
import shutil
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
    ("coherent uniform living-part choice (716)", "src/harm.rs",
     "let total = cells(e);", "let total = e.living().count() as u64;",
     "a_part_draw_follows_living_cells_instead_of_part_count"),
]

# Index 4 is retained as history: changing only row weights creates an
# inconsistent draw domain and catches an unwrap, not the weighting assertion.
ACTIVE = [i for i in range(len(FAULTS)) if i != 4]
EQ = "assertion `left == right` failed"
# Literal native assertion/call fragments locate current source, so field-only
# fixture additions may move lines without weakening the intended failure.
EXPECTED = {
    0: ("tests/harm.rs", "assert!( parent.parts[&PartId(1)].matter.is_empty() && parent.parts[&PartId(2)].matter.is_empty() );",
        "assertion failed: parent.parts[&PartId(1)].matter.is_empty() &&"),
    1: ("tests/harm.rs", "assert_eq!(child.situs(PartId(0)), Some([0; 3]));", EQ),
    2: ("tests/harm.rs", "assert_eq!(child.alive, fragment);", EQ),
    3: ("tests/harm.rs", "assert!( e.parts[&PartId(0)].matter[key] <= anatomy::bound(half, &e.parts[&PartId(0)], &s.sim.genesis().rules, key) );",
        "assertion failed: e.parts[&PartId(0)].matter[key] <="),
    5: ("tests/harm.rs", "assert!( s.sim.state().population.groups.values().all(|g| g .entity .parts .values() .all(|p| p.lost.is_empty())) );",
        "assertion failed: s.sim.state().population.groups.values().all(|g|"),
    6: ("tests/harm.rs", "assert_eq!( anatomy::books(s.sim.state().population.get(1).unwrap()) .values() .sum::<u64>(), 0 );", EQ),
    7: ("tests/harm.rs", "g.validate() .unwrap_err() .contains(\"tombstone still holds matter\")",
        "called `Result::unwrap_err()` on an `Ok` value: ()"),
    8: ("tests/harm.rs", "assert_eq!( before, after, \"every holder and account reconciles independently\" );",
        EQ + ": every holder and account reconciles independently"),
    9: ("tests/harm/crowd.rs", "panic!(\"the crowd accepted an unsupported harm pass\")",
        "the crowd accepted an unsupported harm pass"),
    10: ("tests/harm/healing.rs", "assert_eq!(actor(&s), &original);", EQ),
    11: ("tests/harm/healing.rs", "assert_eq!( anatomy::held(actor(&s), &s.sim.genesis().rules, \"reserve:1\"), reserve - 12 );", EQ),
    12: ("tests/harm/healing.rs", "assert_eq!(p.cells, original.cells);", EQ),
    13: ("tests/harm/healing.rs", "assert!(actor(&s).lives(PartId(1)) && !actor(&s).lives(PartId(2)));",
         "assertion failed: actor(&s).lives(PartId(1)) && !actor(&s).lives(PartId(2))"),
    14: ("tests/harm/healing.rs", "assert_eq!(p.lost, vec![unknown]);", EQ),
    15: ("tests/harm/healing.rs", "assert_eq!(actor(&s).parts[&PartId(2)].lost, vec![CellId(0)]);", EQ),
    16: ("tests/harm/healing.rs", "assert_eq!(held(\"reserve:1\"), if hungry { 1 } else { 36 });", EQ),
    17: ("tests/harm/healing.rs", "assert_eq!(held(\"reserve:1\"), if hungry { 1 } else { 36 });", EQ),
    18: ("tests/harm/healing.rs", "assert_eq!(actor(&s).parts[id].matter[\"tissue:0\"], p.matter[\"tissue:0\"]);", EQ),
    19: ("tests/harm.rs", "assert!( hits[0] > 190 && hits[1] > 190 && hits[2] < 65, \"8:8:1 cells: {hits:?}\" );", None),
}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def replacements(index):
    _, _, before, after, _ = FAULTS[index]
    pairs = [(before, after)]
    if index == 19:
        pairs.append(("let n = u64::from(anatomy::living_cells(e.extent(id), p));", "let n = 1;"))
    return pairs


def binding(root, index):
    relative, anchor, marker = EXPECTED[index]
    source = root / relative
    text = source.read_text(encoding="utf-8")
    owner = {8: "reconciles", 9: "refuse"}.get(index, FAULTS[index][4].split("::")[-1])
    functions = list(re.finditer(r"^fn\s+(\w+)\s*\(", text, re.M))
    owners = [i for i, function in enumerate(functions) if function[1] == owner]
    if len(owners) != 1:
        raise ValueError("intended failure has no unique native source function")
    start = functions[owners[0]].start()
    end = functions[owners[0] + 1].start() if owners[0] + 1 < len(functions) else len(text)
    pattern = r"\s+".join(re.escape(token) for token in anchor.split())
    matches = list(re.finditer(pattern, text[start:end]))
    if len(matches) != 1:
        raise ValueError(f"intended assertion is not unique in {relative}")
    point = start + matches[0].start()
    if index == 7:
        point = text.index(".unwrap_err()", point, start + matches[0].end()) + 1
    return dict(source=relative, function=owner, line=text.count("\n", 0, point) + 1,
                column=point - text.rfind("\n", 0, point), marker=marker,
                assertion=anchor, source_sha256=sha(source.read_bytes()))


def failure_verdict(code, stdout, stderr, test, expected, filtered):
    """Read the actual Cargo/libtest boundary, not merely a nonzero exit."""
    if type(code) is not int or code != 101 or type(filtered) is not int or filtered < 0:
        return False
    stdout, stderr = stdout.replace("\r\n", "\n"), stderr.replace("\r\n", "\n")
    footer = "error: test failed, to rerun pass `--test harm`"
    errors = re.findall(r"^error(?::|\[).*$", stderr, re.M)
    finished = re.findall(r"^\s*Finished `test`.*$", stderr, re.M)
    named = re.findall(r"^test (\S+) \.\.\. (ok|FAILED|ignored)$", stdout, re.M)
    summary_lines = re.findall(r"^test result:.*$", stdout, re.M)
    summaries = re.findall(r"^test result: FAILED\. 0 passed; 1 failed; 0 ignored; 0 measured; "
                           r"(\d+) filtered out; finished in \d+(?:\.\d+)?s$", stdout, re.M)
    if (errors != [footer] or not stderr.strip().endswith(footer) or len(finished) != 1
            or re.search(r"could not compile|^error(?::|\[)", stdout, re.M)
            or re.search(r"could not compile|^thread .* panicked at", stderr, re.M)
            or named != [(test, "FAILED")] or re.findall(r"^running (\d+) tests?$", stdout, re.M) != ["1"]
            or len(summary_lines) != 1 or summaries != [str(filtered)]
            or len(re.findall(r"^---- (.+) stdout ----$", stdout, re.M)) != 1
            or re.findall(r"^---- (.+) stdout ----$", stdout, re.M) != [test]):
        return False
    if "failures:\n" not in stdout:
        return False
    failed_names = stdout.rsplit("failures:\n", 1)[1].split("\ntest result:", 1)[0]
    if [line.strip() for line in failed_names.splitlines() if line.strip()] != [test]:
        return False
    if re.search(r"^(?:FAILED|ok|ignored)$", stdout, re.M):
        return False
    location = re.escape(expected["source"]).replace("/", r"[/\\]")
    panic = (r"^thread '" + re.escape(test) + r"'(?: \(\d+\))? panicked at " + location
             + ":" + str(expected["line"]) + ":" + str(expected["column"]) + r":\n([^\n]*)$")
    markers = re.findall(panic, stdout, re.M)
    if len(markers) != 1 or len(re.findall(r"^thread .* panicked at .*$", stdout, re.M)) != 1:
        return False
    if expected["marker"] is not None:
        return (markers == [expected["marker"]]
                and len(re.findall(r"^" + re.escape(expected["marker"]) + "$", stdout, re.M)) == 1)
    hits = re.fullmatch(r"8:8:1 cells: \[(\d+), (\d+), (\d+)\]", markers[0])
    return (hits is not None and sum(map(int, hits.groups())) == 512
            and not (int(hits[1]) > 190 and int(hits[2]) > 190 and int(hits[3]) < 65))


def audit(root, index):
    name, relative, before, after, test = FAULTS[index]
    original = (root / relative).read_bytes()
    anchors = [original.count(a.encode()) for a, _ in replacements(index)]
    tests = [root / "tests/harm.rs", *(root / "tests/harm").glob("*.rs")]
    pattern = rf"\bfn\s+{re.escape(test.split('::')[-1])}\s*\("
    matches = [str(p.relative_to(root)) for p in tests
               if re.search(pattern, p.read_text(encoding="utf-8"))]
    expected = binding(root, index) if index in ACTIVE else None
    return {"index": index, "fault": name, "source": relative, "test": test,
            "original_sha256": sha(original), "anchors": anchors,
            "test_sources": matches, "intended_panic": expected,
            "verdict": "SKIPPED-LEGACY" if index == 4 else
                "SOURCE-READY" if all(n == 1 for n in anchors) and len(matches) == 1 else "INVALID",
            "compiled": False, "mutated": False}


def run(root, index, cargo, filtered):
    if index not in ACTIVE:
        raise ValueError("legacy single-anchor fault 4 cannot certify weighting; use coherent control 19")
    if type(filtered) is not int or filtered < 0:
        raise ValueError("control requires an actual integer filtered-test count")
    ready = audit(root, index)
    if ready["verdict"] != "SOURCE-READY":
        raise ValueError("control source/test/assertion is not uniquely bound")
    name, relative, before, after, test = FAULTS[index]
    source = root / relative
    original = source.read_bytes()
    mutant = original
    for before, after in replacements(index):
        mutant = mutant.replace(before.encode(), after.encode())
    command = [cargo, "test", "--locked", "--offline", "-j1", "--test", "harm", test, "--", "--exact"]
    result = {"index": index, "fault": name, "source": relative, "test": test,
              "original_sha256": sha(original), "mutant_sha256": sha(mutant), "command": command,
              "intended_panic": ready["intended_panic"], "expected_filtered": filtered,
              "compiled": False, "caught": False, "verdict": "INVALID"}
    child, first_error = None, None
    stdout, stderr = b"", b""
    try:
        source.write_bytes(mutant)
        if source.read_bytes() != mutant:
            raise ValueError("mutant source write differs")
        flags = (getattr(subprocess, "CREATE_NO_WINDOW", 0)
                 | getattr(subprocess, "BELOW_NORMAL_PRIORITY_CLASS", 0))
        child = subprocess.Popen(command, cwd=root, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, creationflags=flags)
        result["pid"] = child.pid
        while True:
            try:
                stdout, stderr = child.communicate()
                break
            except BaseException as error:
                if first_error is None:
                    first_error = error
        result.update(exit_code=child.returncode, child_drained=True)
        out, err = stdout.decode("utf-8"), stderr.decode("utf-8")
        result.update(exit_code=child.returncode, stdout=out, stderr=err, child_drained=True)
        compile_err = err.replace("\r\n", "\n")
        compiled = (len(re.findall(r"^\s*Finished `test`.*$", compile_err, re.M)) == 1
                    and re.findall(r"^error(?::|\[).*$", compile_err, re.M)
                    == ["error: test failed, to rerun pass `--test harm`"]
                    and "could not compile" not in compile_err)
        caught = first_error is None and failure_verdict(child.returncode, out, err, test,
                                                        ready["intended_panic"], filtered)
        result.update(compiled=compiled, caught=caught)
        result["verdict"] = "CAUGHT" if caught else "INVALID"
    except BaseException as error:
        if first_error is None:
            first_error = error
    finally:
        # An interrupted communicate must finish before source restoration.
        while child is not None and child.returncode is None:
            try:
                stdout, stderr = child.communicate()
            except BaseException as error:
                if first_error is None:
                    first_error = error
        try:
            source.write_bytes(original)
            restored = source.read_bytes() == original
            if not restored:
                raise RuntimeError(f"failed to restore {source}")
            result["restored_sha256"] = sha(original)
        except BaseException as error:
            result["restoration_error"] = repr(error)
            if first_error is None:
                first_error = error
    result["restored_exact"] = "restoration_error" not in result
    result["_stdout"], result["_stderr"] = stdout, stderr
    if first_error is not None:
        result.update(caught=False, verdict="INVALID", error=repr(first_error))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path)
    parser.add_argument("--audit", action="store_true", help="read source anchors/tests without Cargo or mutation")
    parser.add_argument("--cargo", default=shutil.which("cargo"), help="actual Cargo executable; toolchain remains caller-owned")
    parser.add_argument("--expected-filtered", type=int, help="actual positive harm registry total minus one")
    parser.add_argument("picks", nargs="*", type=int)
    args = parser.parse_args()
    root = args.workspace.resolve()
    if not (root / "src/harm.rs").is_file() or not (root / "Cargo.toml").is_file():
        parser.error("--workspace must name the assigned shared/isocosm workspace")
    picks = args.picks or ACTIVE
    if len(picks) != len(set(picks)) or any(i not in range(len(FAULTS)) for i in picks):
        parser.error("controls must be distinct known indices")
    if not args.audit and (4 in picks or not args.output or not args.cargo
                          or args.expected_filtered is None or args.expected_filtered < 0):
        parser.error("run requires fresh --output, --cargo and actual --expected-filtered; use 19 instead of legacy 4")
    if args.output:
        planned = [args.output]
        if not args.audit:
            planned += [args.output.with_name(args.output.stem + f"-{i:02d}" + suffix)
                        for i in picks for suffix in [".json", ".stdout.log", ".stderr.log"]]
        if any(path.exists() for path in planned) or not args.output.parent.is_dir():
            parser.error("output names must be fresh in an existing receipt directory")
    results = []
    try:
        for index in picks:
            result = audit(root, index) if args.audit else run(root, index, args.cargo, args.expected_filtered)
            results.append(result)
            if not args.audit:
                for stream in ["stdout", "stderr"]:
                    raw = result.pop("_" + stream)
                    path = args.output.with_name(args.output.stem + f"-{index:02d}.{stream}.log")
                    with path.open("xb") as output:
                        output.write(raw)
                    result[stream + "_raw"] = dict(file=str(path), sha256=sha(raw), bytes=len(raw))
                path = args.output.with_name(args.output.stem + f"-{index:02d}.json")
                with path.open("x", encoding="utf-8") as output:
                    json.dump(result, output, indent=2)
            print(f"{index}: {result['verdict']} {result['fault']}", flush=True)
            if result["verdict"] not in ["SOURCE-READY", "CAUGHT", "SKIPPED-LEGACY"]:
                break
    except BaseException as error:
        for result in results:
            result.pop("_stdout", None)
            result.pop("_stderr", None)
        results.append(dict(verdict="INVALID", error=repr(error)))
    finally:
        if args.output:
            with args.output.open("x", encoding="utf-8") as output:
                json.dump(results, output, indent=2)
    expected = "SOURCE-READY" if args.audit else "CAUGHT"
    return 0 if all(r["verdict"] == expected for r in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
