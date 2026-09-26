"""Compare two differential driver logs with ruling 259's counters set apart.

usage: python compare_logs.py OLD NEW [OUTPUT.json]

An advance's work is printed as `Work { evaluations, represented, accepted,
blocked }`. Since ruling 259 an evaluation that would be blocked is not
made, so evaluations, represented and blocked may fall; accepted must not
change. Every line is compared with those three counters removed; the
counters are summed on each side and reported.
"""
import hashlib
import json
import re
import sys

WORK = re.compile(r"Work \{ evaluations: (\d+), represented: (\d+), accepted: (\d+), blocked: (\d+) \}")


def masked(line):
    return WORK.sub(lambda m: f"Work {{ accepted: {m.group(3)} }}", line)


def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read().splitlines()


old, new = read(sys.argv[1]), read(sys.argv[2])
totals = {side: [0, 0, 0, 0] for side in ("old", "new")}
for side, lines in (("old", old), ("new", new)):
    for line in lines:
        for m in WORK.finditer(line):
            for i in range(4):
                totals[side][i] += int(m.group(i + 1))
first_difference = None
differing = 0
for i, (a, b) in enumerate(zip(old, new)):
    if masked(a) != masked(b):
        differing += 1
        if first_difference is None:
            first_difference = {"line": i + 1, "old": a[:300], "new": b[:300]}
same = len(old) == len(new) and differing == 0
names = ["evaluations", "represented", "accepted", "blocked"]
result = {
    "old": sys.argv[1].split("/")[-1],
    "new": sys.argv[2].split("/")[-1],
    "lines": [len(old), len(new)],
    "sha256": [digest(sys.argv[1]), digest(sys.argv[2])],
    "byte_identical": digest(sys.argv[1]) == digest(sys.argv[2]),
    "identical_apart_from_evaluation_counts": same,
    "lines_differing_apart_from_evaluation_counts": differing,
    "first_difference": first_difference,
    "work_totals": {side: dict(zip(names, t)) for side, t in totals.items()},
}
print(json.dumps(result, indent=1))
if len(sys.argv) > 3:
    open(sys.argv[3], "w").write(json.dumps(result, indent=2) + "\n")
