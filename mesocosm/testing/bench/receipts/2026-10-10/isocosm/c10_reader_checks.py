"""Small checks of the production C10 readers. No Cargo, faults or permutations.

The retained native receipt checks shape only; its failed certificate stays failed.
The fault transcripts below are explicitly fabricated parser inputs, not native credit.
"""
import copy
import hashlib
import json
from pathlib import Path
import unittest

import c10_faults as faults
import c10_verify as verify


ROOT = Path(__file__).resolve().parents[6] / "shared/isocosm"
RETAINED = Path(r"C:\Users\mark_\Code\testing\isometry\receipts\2026-10-10\isocosm\q8-c10-postheal-stat-1000.json")
RETAINED_SHA = "74c6649a86404e226860e0f3c23317c7e77b513af6a4c9139846c6da43b59be0"


class ReaderChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        raw = RETAINED.read_bytes()
        if hashlib.sha256(raw).hexdigest() != RETAINED_SHA:
            raise ValueError("retained native failure receipt bytes changed")
        cls.native = verify.decode(raw)

    def test_actual_retained_shape_and_individual_seeds(self):
        verify.validate_shape(self.native, 1000, 2026101003, False)
        self.assertFalse(self.native["verdicts"]["positive_control_pass"])
        self.assertEqual(verify.ARM_INDEX["crowd-unweighted"], 5)

    def test_native_shape_tampering(self):
        cases = [
            (("version",), True), (("master_seed",), 2026101003.0),
            (("settings", "permutations"), 1999.0), (("settings", "seed"), False),
            (("domain", "harm", "heals"), 500.0),
            (("checks", "exact_rerun_identical"), 1),
            (("draws", 0, "k"), False), (("draws", 0, "seed"), 1),
            (("draws", 0, "arms", 0, "dynamics"), 1),
            (("draws", 0, "arms", 4, "dynamics"), verify.draw_seed(2026101003, "probe-dynamics", [0, 4])),
            (("draws", 0, "arms", 0, "accepted"), 1.0),
            (("draws", 0, "arms", 0, "readings", 0), False),
            (("draws", 0, "arms", 0, "matter", "verified_ticks"), 60.0),
            (("draws", 0, "arms", 0, "matter", "issued"), 1),
            (("draws", 0, "arms", 0, "refused"), "unexpected refusal"),
            (("draws", 0, "arms", 2, "refused"), ""),
            (("draws", 0, "arms", 2, "refused"), False),
            (("draws", 0, "arms", 4, "arm"), "crowd"),
            (("readings", 0, "starvation"), 1), (("readings", 0, "bound_per_mille"), 200.0),
            (("readings", 1, "key"), self.native["readings"][0]["key"]),
            (("comparisons", 0, "draws"), 1000.0),
            (("comparisons", 0, "refusals", "within"), 1),
            (("comparisons", 0, "readings", 0, "p_difference"), float("nan")),
            (("comparisons", 0, "readings", 0, "certified"), 1),
            (("verdicts", "main_pass"), 1),
        ]
        for path, value in cases:
            with self.subTest(path=path):
                native = copy.deepcopy(self.native)
                target = native
                for key in path[:-1]:
                    target = target[key]
                target[path[-1]] = value
                with self.assertRaises((ValueError, TypeError, KeyError)):
                    verify.validate_shape(native, 1000, 2026101003, False)
                result = verify.verify(native, 1000, 2026101003, False)
                self.assertFalse(result["pass"])
                self.assertFalse(result["certified"])
                self.assertEqual(result["comparisons"], [])

    def test_json_and_argument_types(self):
        for raw in [b'{"x":1,"x":2}', b'{"x":NaN}', b'{"x":Infinity}']:
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                verify.decode(raw)
        for draws, seed, pilot in [(1000.0, 2026101003, False), (1000, True, False),
                                    (1000, 2026101003, 0), (0, 2026101003, True)]:
            with self.subTest(args=(draws, seed, pilot)), self.assertRaises(ValueError):
                verify.validate_shape(self.native, draws, seed, pilot)

    def test_literal_fault_bindings_are_current_and_unique(self):
        self.assertEqual(len(faults.ACTIVE), 19)
        self.assertNotIn(4, faults.ACTIVE)
        for index in faults.ACTIVE:
            with self.subTest(index=index):
                row = faults.audit(ROOT, index)
                self.assertEqual(row["verdict"], "SOURCE-READY")
                self.assertTrue(all(n == 1 for n in row["anchors"]))
        self.assertEqual(faults.audit(ROOT, 4)["verdict"], "SKIPPED-LEGACY")
        self.assertEqual(len(faults.replacements(19)), 2)

    def test_single_fault_parser_and_adverse_transcripts(self):
        test = faults.FAULTS[0][4]
        expected = faults.binding(ROOT, 0)
        stdout = (f"running 1 test\ntest {test} ... FAILED\n\nfailures:\n\n---- {test} stdout ----\n"
                  f"thread '{test}' panicked at {expected['source']}:{expected['line']}:{expected['column']}:\n"
                  f"{expected['marker']}\n\nfailures:\n    {test}\n\n"
                  "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.01s\n")
        stderr = "Finished `test` profile [optimized + debuginfo] target(s) in 0.01s\nerror: test failed, to rerun pass `--test harm`\n"
        self.assertTrue(faults.failure_verdict(101, stdout, stderr, test, expected, 31))
        for code in [0, 1, False, True, 101.0, None, "101"]:
            with self.subTest(code=code):
                self.assertFalse(faults.failure_verdict(code, stdout, stderr, test, expected, 31))
        changed = [
            stdout.replace("running 1 test", "running 0 tests"),
            stdout.replace(f"test {test} ... FAILED", "test wrong ... FAILED"),
            stdout.replace(expected["marker"], "unrelated panic"),
            stdout.replace(expected["source"], "tests/wrong.rs"),
            stdout.replace("31 filtered", "30 filtered"),
            stdout + f"test {test} ... FAILED\n",
            stdout + "thread 'other' panicked at other.rs:1:1:\nwrong\n",
            stdout.replace(f"    {test}\n", f"    {test}\n    other\n"),
            stdout.replace(f"test {test} ... FAILED", f"test {test} ... \nFAILED"),
            stdout + "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.01s\n",
        ]
        for altered in changed:
            with self.subTest(stdout=altered[-80:]):
                self.assertFalse(faults.failure_verdict(101, altered, stderr, test, expected, 31))
        for altered in [stderr + stderr, stderr.replace("Finished", "Unfinished"),
                        stderr.replace("--test harm", "--lib"), stderr + "error[E0001]: compiler error\n"]:
            with self.subTest(stderr=altered):
                self.assertFalse(faults.failure_verdict(101, stdout, altered, test, expected, 31))


if __name__ == "__main__":
    unittest.main()
