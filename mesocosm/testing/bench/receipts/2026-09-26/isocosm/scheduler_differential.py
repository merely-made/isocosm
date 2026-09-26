"""Write scheduler-differential.json from the driver logs in this directory.

usage: python scheduler_differential.py OUTPUT
Runs compare_logs.py and tight_baseline.py over the logs and gathers them.
"""
import json
import subprocess
import sys


def run(*args):
    out = subprocess.check_output([sys.executable, *args], text=True)
    return json.loads(out[: out.rindex("}") + 1])


fuzz = []
for command, old, new in [
    ("drive fuzz 7 24 400", "fuzz-old.txt", "fuzz-index7.txt"),
    ("drive fuzz 11 60 1500", "fuzz-old-11.txt", "fuzz-index11.txt"),
]:
    fuzz.append({"command": command, **run("compare_logs.py", old, new)})
tight = {"command": "drive tight 13 48 600", **run("compare_logs.py", "tight-old.txt", "tight-index.txt")}
tight["baseline"] = run("tight_baseline.py", "tight-old.txt", "tight-index.txt")
doc = {
    "kind": "isocosm-differential",
    "note": "The same driver source (differential-drive.rs of 2026-09-26) built against the core before ruling 258 (be9d863) and after it (the commit scheduler-source.json names), run on the same arguments. The old logs are the ones checkpoints 1 and 2 kept as baselines; the core before this change reproduced all three byte for byte. An advance's work prints its evaluations, the members they represent, acceptances and blocks; since ruling 259 an evaluation that would be blocked by a missing trait is not made, so each line is compared with evaluations, represented and blocked set apart, and acceptances kept. The fuzz runs carry no operation budget beyond the default; the tight sessions run under small ones, so a new baseline is expected there, and its summary says where each session first diverges and why. Logs are not kept; their digests are.",
    "fuzz": fuzz,
    "tight": tight,
}
open(sys.argv[1], "w").write(json.dumps(doc, indent=2) + "\n")
print(json.dumps({"fuzz": [(f["command"], f["identical_apart_from_evaluation_counts"]) for f in fuzz],
                  "tight": tight["baseline"]["first_difference_causes"]}, indent=1))
