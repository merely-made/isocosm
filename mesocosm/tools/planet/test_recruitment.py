# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

from recruitment import inspect


def resident(body, dormant=False):
    return dict(first=body, count=1, tissue=8, lineage="lineage:0",
                kingdom="kingdom:flora", dormant=dormant)


def transfer(process, source, dest):
    return dict(process=process, debit="matter:0-0", amount=1,
                **{"from": source, "to": dest})


class RecruitmentTest(unittest.TestCase):
    def test_dormant_daughter_becomes_established_and_carrion_is_excluded(self):
        # The founder and seed share a site. The daughter wakes at age seven,
        # then travels. The animal returns to the founder's dead tissue.
        g = {
            "population": {"groups": {"10": {"count": 1, "entity": {"lineage": "lineage:0"}}}},
            "sites": {"0": {}},
            "lineages": {"lineage:0": {"kingdom": "kingdom:flora"},
                         "lineage:1": {"kingdom": "kingdom:fauna"},
                         "lineage:2": {"kingdom": "kingdom:myco"}},
            "rules": {"processes": {"ecology:birth-0": {"period": 7}}},
        }
        flows = {1: [transfer("ecology:birth-0", 10, 20)],
                 6: [transfer("ecology:feed-2", 20, 60)],
                 8: [transfer("ecology:feed-1", 20, 50)],
                 9: [transfer("ecology:feed-1", 10, 50)]}
        family = {"tick": 9, "members": [
            dict(id=10, parent=None, generation=0, alive=False, dormant=False, site=None),
            dict(id=20, parent=10, generation=1, alive=True, dormant=False,
                 site=None, journey={"to": 1}),
        ]}
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp).resolve()
            (directory / "founding.json").write_text(json.dumps({"native": {"genesis": g}}))
            with sqlite3.connect(directory / "field.sqlite3") as db:
                db.executescript("CREATE TABLE frames(tick,data); CREATE TABLE head(id,tick,save);")
                db.execute("INSERT INTO head VALUES(1,9,'replayed')")
                for t in range(10):
                    bodies = [resident(10)] if t < 8 else []
                    if 0 < t < 9:
                        bodies.append(resident(20, dormant=t < 8))
                    f = dict(tick=t, hash=str(t), branch="test",
                             sites=[dict(residents=bodies, flows=flows.get(t, []))])
                    db.execute("INSERT INTO frames VALUES(?,?)", (t, json.dumps(f)))
            with patch("recruitment.reopened", return_value=({"hash": "9"}, family)):
                result = inspect(directory, None, ancestor=10, site=0)
            founder, daughter = result["family"]
            self.assertEqual((result["daughters"], result["established"], result["daughtersSurvivedBite"]), (1, 1, 1))
            self.assertEqual(daughter["established"], [8, 0])
            self.assertEqual(daughter["bites"], [[8, 0, 50, 1]])
            self.assertEqual(founder["bites"], [])
            self.assertEqual(result["confirmedGrazingOnSiteResidents"], 1)
