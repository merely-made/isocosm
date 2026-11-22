# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0

"""Check against the ordinary native founding and reopening paths."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

from serve import Editor, TEMPLATE, native


class EditorTest(unittest.TestCase):
    def test_entry_matches_ordinary_founding_and_reopen(self):
        for edit in ({}, {"repeat": 2, "absence": [1, 4]}, {"mapped_sites": True, "direct_grazers": True}):
            with self.subTest(edit=edit), tempfile.TemporaryDirectory() as tmp:
                preview = self.editor.preview(json.dumps(edit))
                source = Path(tmp) / "founding.toml"
                source.write_text(preview["source"])
                if not edit:
                    self.assertEqual(source.read_bytes(), TEMPLATE.read_bytes())
                for tick in (0, 17, 35):
                    witness = preview["frames"][tick]["hash"]
                    exported = self.editor.save(preview, tick, witness)
                    ordinary = Path(tmp) / f"ordinary-{tick}.json"
                    native([self.editor.bench, "--founding", source, "--ticks", str(tick),
                            "--output", ordinary])
                    self.assertEqual(exported, ordinary.read_bytes())
                    self.assertEqual(json.loads(exported)["state_hash"], int(witness, 16))
                resumed = json.loads(native([self.editor.bench, "--load",
                    Path(tmp) / "ordinary-17.json", "--ticks", "18"]))
                direct = json.loads(ordinary.read_bytes())
                self.assertEqual(resumed["tick"], 35)
                self.assertEqual(resumed["genesis_digest"], direct["genesis_digest"])
                self.assertEqual(resumed["witness"], direct["witness"])
                with self.assertRaisesRegex(ValueError, "different preview"):
                    self.editor.save(preview, 17, "0000000000000000")
                with self.assertRaisesRegex(ValueError, "preview state mismatch"):
                    native([self.editor.bench, "--founding", source, "--ticks", "17",
                            "--expect-state", "0"])

    def test_resident_departure_forks_the_two_founders_and_reopens(self):
        opts = {"study": "pocket", "seed": 9, "leave_at": 6}
        preview = self.editor.preview(json.dumps(opts))
        staying, leaving = preview["frames"], preview["leave"]
        self.assertTrue(staying[0]["contact"])
        self.assertEqual(staying[:6], leaving[:6])
        self.assertEqual(staying[6]["flows"], leaving[6]["flows"])
        self.assertGreater(leaving[6]["flows"]["sugar"], 0)
        for who in ("host", "resident"):
            self.assertEqual(staying[6][who], leaving[6][who])
            self.assertEqual(leaving[-1][who]["id"], staying[0][who]["id"])
        self.assertTrue(staying[6]["contact"])
        self.assertTrue(all(not f["contact"] for f in leaving[6:]))
        self.assertTrue(all(f["flows"] == leaving[6]["flows"] for f in leaving[6:]))
        for key in ("soil", "sugar", "fee"):
            self.assertGreater(staying[-1]["flows"][key], leaving[-1]["flows"][key])
        for who in ("host", "resident"):
            self.assertNotEqual(staying[-1][who]["tissue"], leaving[-1][who]["tissue"])
        with tempfile.TemporaryDirectory() as folder:
            source, bare_trace = Path(folder)/"founding.toml", Path(folder)/"ordinary.trace"
            source.write_text(preview["source"])
            native([self.editor.bench, "--founding", source, "--ticks", "36", "--trace", bare_trace])
            self.assertEqual([line.split()[1] for line in bare_trace.read_text().splitlines()],
                             [f["hash"] for f in staying[1:]])
            for branch, frames in [("stay", staying), ("leave", leaving)]:
                selected = self.editor.preview(json.dumps({**opts, "branch": branch}))
                path, trace = Path(folder)/"saved.json", Path(folder)/"trace"
                path.write_bytes(self.editor.save(selected, 6, frames[6]["hash"]))
                native([self.editor.bench, "--load", path, "--ticks", "30", "--trace", trace])
                self.assertEqual(trace.read_text().splitlines()[-1].split(), ["36",frames[-1]["hash"]])
            with self.assertRaisesRegex(ValueError, "different preview"):
                self.editor.save(selected, 6, staying[6]["hash"])

    def test_pocket_requires_both_mapped_suppliers(self):
        for withdrawn in ["pocket", "port"]:
            trial = self.editor.preview(json.dumps({"study":"pocket",withdrawn:False}))
            frame = trial["frames"][0]
            self.assertEqual(bool(frame["host"]["opening"]), withdrawn != "pocket")
            self.assertEqual(bool(frame["resident"]["port"]), withdrawn != "port")
            self.assertIsNotNone(frame["resident"]["frond"])
            self.assertTrue(all(not f["contact"] and sum(f["flows"].values()) == 0
                                for f in trial["frames"] + trial["leave"]))

    def test_native_trait_bank_attachment_and_founder_grain(self):
        base = self.editor.preview("{}")
        options = {"organ": "part:thorn-stem", "stem_variance": 1, "side": "Left", "anchor": "Base"}
        varied = self.editor.preview(json.dumps(options))
        self.assertEqual(base["frames"][0]["living"], varied["frames"][0]["living"])
        self.assertEqual(varied["map"]["regions"]["region:gland"], options["organ"])
        counts = {sum("function:fix" in p["cells"] for p in f["parts"]) for f in varied["founders"]}
        self.assertEqual(counts, {1, 2, 3})
        for founder in varied["founders"]:
            root = next(p for p in founder["parts"] if p["situs"] == [0, 0, 0])
            side = next(p for p in founder["parts"] if p["situs"] == [1, 0, 0])
            self.assertEqual(side["center"][1:], root["center"][1:])
            self.assertEqual(side["center"][0] + side["half"][0], root["center"][0] - root["half"][0])
        end = varied["frames"][-1]
        self.assertGreater(end["totals"]["spine_wounds"], 0)
        self.assertEqual(end["totals"]["wounds"], 0)
        self.assertEqual(json.loads(self.editor.save(varied, end["tick"], end["hash"]))["state_hash"], int(end["hash"], 16))
        separate = self.editor.preview(json.dumps({**options, "cohort_size": 1}))
        self.assertTrue(all(f["count"] == 1 for f in separate["founders"]))
        with self.assertRaisesRegex(ValueError, "outside the defensive trait bank"):
            self.editor.preview(json.dumps({"organ": "part:made-up"}))

    def test_native_attachments_and_survival_evidence(self):
        edit = {"mapped_sites": True, "direct_grazers": True}
        front = self.editor.preview(json.dumps(edit))
        below = self.editor.preview(json.dumps({**edit, "frond_socket": "Below"}))
        for preview, above in [(front, True), (below, False)]:
            for f in preview["founders"]:
                parts = {p["id"]: p for p in f["parts"]}
                self.assertEqual(sum(p["parent"] is None for p in parts.values()), 1)
                for p in parts.values():
                    if "function:fix" in p["cells"]:
                        self.assertIn(p["parent"], parts)
                        self.assertEqual(p["center"][1] > parts[p["parent"]]["center"][1], above)
                        self.assertEqual(p["canopy"], above)
                self.assertEqual(f["trophic"] == "Producer", above)
            end = preview["frames"][-1]
            if above:
                self.assertTrue(all(fixed > 0 for fixed in end["totals"]["soil_fixed"]))
            else:
                self.assertEqual(end["totals"]["soil_fixed"], [0, 0])
            self.assertTrue(preview["inputs"]["needs_fixing_canopy"])
            self.assertGreater(sum(end["totals"]["upkeep"]), 0)
            self.assertEqual(end["totals"]["defence_upkeep"][0], 0)
            self.assertEqual(sum(end["totals"]["defence_upkeep"]), end["totals"]["rent"])
            self.assertEqual(json.loads(self.editor.save(preview, 35, end["hash"]))["state_hash"], int(end["hash"], 16))
        self.assertNotEqual(front["frames"][0]["hash"], below["frames"][0]["hash"])
        # A reference habitat is an ordinary founding with just its seed replaced.
        with tempfile.TemporaryDirectory() as tmp:
            for run in front["reference"]:
                source = Path(tmp) / "reference.toml"
                source.write_text(front["source"].replace("seed = 9\n", f"seed = {run['seed']}\n", 1))
                observed = json.loads(native([self.editor.observer, source], b"{}"))
                self.assertEqual(run["initial"], observed["frames"][0]["living"])
                self.assertEqual(run["living"], observed["frames"][-1]["living"])
                self.assertEqual(run["soil_fixed"], observed["frames"][-1]["totals"]["soil_fixed"])
        with self.assertRaisesRegex(ValueError, "frond socket"):
            self.editor.preview(json.dumps({"frond_socket": "Front"}))

    def test_odds_and_repeat_keep_the_same_founder_cohorts(self):
        original = self.editor.preview("{}")
        identities = [(f["id"], f["count"]) for f in original["founders"]]
        self.assertEqual(original["frames"][0]["living"], [10, 8])
        self.assertEqual(original["frames"][-1]["living"], [23, 2])
        self.assertEqual(original["frames"][-1]["totals"]["rent"], 142)
        self.assertEqual(original["frames"][-1]["totals"]["wounds"], 312)
        for odds, gland in [([1, 1], False), ([0, 1], True)]:
            preview = self.editor.preview(json.dumps({"absence": odds, "repeat": 2}))
            self.assertEqual([(f["id"], f["count"]) for f in preview["founders"]], identities)
            for founder in preview["founders"]:
                self.assertEqual(founder["gland"], gland)
                self.assertEqual(sum("function:fix" in p["cells"] for p in founder["parts"]), 3)
                self.assertEqual(sum("function:secrete" in p["cells"] for p in founder["parts"]), int(gland))


if __name__ == "__main__":
    EditorTest.editor = Editor(Path(sys.argv.pop(1)))
    unittest.main()
