"""Explain the budgeted sessions' new baseline against the old one.

usage: python tight_baseline.py OLD NEW [OUTPUT.json]

For each world, finds the first step whose line differs apart from ruling
259's counters, and says what the step was on each side. Also tallies
advances accepted and refused, and each world's load in the other mode.
"""
import collections
import json
import re
import sys

WORK = re.compile(r"Work \{ evaluations: (\d+), represented: (\d+), accepted: (\d+), blocked: (\d+) \}")


def masked(line):
    return WORK.sub(lambda m: f"Work {{ accepted: {m.group(3)} }}", line)


def worlds(path):
    by = collections.defaultdict(list)
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        m = re.match(r"^(?:world )?(\d+)[. ]", line)
        if m:
            by[int(m.group(1))].append(line)
    return by


def tally(lines):
    advances = collections.Counter()
    load = None
    for line in lines:
        parts = line.split()
        if len(parts) > 3 and parts[1] == "advance":
            advances["refused" if parts[3].startswith("Err") else "accepted"] += 1
        m = re.search(r" end .* loaded (Ok|Err)\(\"?([^\")]*)", line)
        if m:
            load = m.group(1) if m.group(1) == "Ok" else "Err: " + m.group(2)
    return dict(advances), load


old, new = worlds(sys.argv[1]), worlds(sys.argv[2])
rows = []
causes = collections.Counter()
for w in sorted(old):
    a, b = old[w], new[w]
    first = next((i for i, (x, y) in enumerate(zip(a, b)) if masked(x) != masked(y)), None)
    row = {"world": w, "old": tally(a), "new": tally(b)}
    if first is None:
        row["same"] = True
    else:
        x, y = a[first].split(), b[first].split()
        cause = f"old {x[1]} {x[3][:3] if len(x) > 3 else ''} / new {y[1]} {y[3][:3] if len(y) > 3 else ''}"
        budget = "operation budget" in a[first]
        row["first_difference"] = {"step": a[first].split()[0], "old": a[first][:200], "new": b[first][:200]}
        if budget and y[3].startswith("Ok"):
            row["cause"] = "an advance the old budget refused runs"
        elif budget and y[3].startswith("Err") and x[-5] == y[-5]:
            row["cause"] = "refused on both sides, the world unchanged; now by the session's sum over epochs"
        else:
            row["cause"] = cause
        causes[row["cause"]] += 1
    rows.append(row)
summary = {
    "worlds": len(rows),
    "identical_apart_from_counts": sum(1 for r in rows if r.get("same")),
    "first_difference_causes": dict(causes),
    "advances": {
        side: dict(sum((collections.Counter(r[side][0]) for r in rows), collections.Counter()))
        for side in ("old", "new")
    },
    "loads": {
        side: dict(collections.Counter(r[side][1] for r in rows)) for side in ("old", "new")
    },
    "loads_failing": {
        side: [r["world"] for r in rows if r[side][1] != "Ok"] for side in ("old", "new")
    },
}
print(json.dumps(summary, indent=1))
if len(sys.argv) > 3:
    open(sys.argv[3], "w").write(json.dumps({"summary": summary, "worlds": rows}, indent=2) + "\n")
