"""Write c4-differential.json: the differential driver's logs after ruling 286
against after ruling 287, world by world, with the driver's new build shown
to reproduce the old one's logs byte for byte on the core before Part A.

usage: python c4_differential.py OUTPUT.json LOG_DIR

LOG_DIR holds the driver's logs, which are not kept: a-before-* from
differential-drive.rs and d3-before-* from differential-drive-287.rs on the
core at 71e0b58, a-due-* from differential-drive.rs at 5cc1291, and d3-new-*
from differential-drive-287.rs at 32b8338, each of `drive fuzz 7 24 400`,
`drive fuzz 11 60 1500` and `drive tight 13 48 600`. Their hashes are in
the output."""
import hashlib, json, re, sys
from pathlib import Path

base = Path(sys.argv[2])
WORLD = re.compile(r"^(?:world )?(\d+)[ .]")


def sha(p):
    return hashlib.sha256((base / p).read_bytes()).hexdigest()


def worlds(p):
    out = {}
    for line in (base / p).read_text(encoding="utf-8").splitlines():
        m = WORLD.match(line)
        out.setdefault(int(m.group(1)) if m else -1, []).append(line)
    return out


def loads(p):
    other, own = [], []
    for line in (base / p).read_text(encoding="utf-8").splitlines():
        m = re.match(r"^world (\d+) end .* loaded (Ok|Err)\((.*?)\) same (Ok|Err)\((.*)\)$", line.strip())
        if m:
            if m.group(2) != "Ok":
                other.append(int(m.group(1)))
            if m.group(4) != "Ok":
                own.append(int(m.group(1)))
    return other, own


runs = []
for name, before, after in [
    ("fuzz 7 24 400", "a-due-fuzz7.txt", "d3-new-fuzz7.txt"),
    ("fuzz 11 60 1500", "a-due-fuzz11.txt", "d3-new-fuzz11.txt"),
    ("tight 13 48 600", "a-due-tight.txt", "d3-new-tight.txt"),
]:
    a, b = worlds(before), worlds(after)
    differ = [w for w in sorted(a) if a[w] != b.get(w)]
    run = {
        "command": f"drive {name}",
        "after_286": {"file": before, "sha256": sha(before)},
        "after_287": {"file": after, "sha256": sha(after)},
        "worlds": len(a),
        "reservoir_worlds_identical": f"{sum(1 for w in a if w % 2 == 0 and w not in differ)} of {sum(1 for w in a if w % 2 == 0)}",
        "ecology_worlds_identical": f"{sum(1 for w in a if w % 2 == 1 and w not in differ)} of {sum(1 for w in a if w % 2 == 1)}",
        "differing_worlds": differ,
    }
    if name.startswith("tight"):
        other, own = loads(after)
        run["saves_failing_to_load_in_the_other_mode"] = other
        run["saves_failing_to_load_in_their_own_mode"] = own
    runs.append(run)
same = []
for name, old, new in [
    ("fuzz 7 24 400", "a-before-fuzz7.txt", "d3-before-fuzz7.txt"),
    ("fuzz 11 60 1500", "a-before-fuzz11.txt", "d3-before-fuzz11.txt"),
    ("tight 13 48 600", "a-before-tight.txt", "d3-before-tight.txt"),
]:
    same.append({"command": f"drive {name}", "byte_identical": sha(old) == sha(new), "sha256": sha(new)})
doc = {
    "kind": "isocosm-differential-287",
    "driver": "differential-drive.rs with the one target selector it builds read from JSON, so it builds against the core before ruling 287 added fields to the selector and after (drive3.rs, kept beside it as differential-drive-287.rs)",
    "driver_reproduces_the_old_on_the_core_before_part_a": same,
    "note": "The driver makes odd worlds ecology worlds and even ones reservoir worlds. Ruling 287 changes only the ecology's feeding, so its rules and results. The ecology worlds left identical are exactly those founded with one lineage, a producer alone, which has no feeding process (recomputed from the driver's draws); every ecology world with a consumer or decomposer changed.",
    "runs": runs,
}
Path(sys.argv[1]).write_text(json.dumps(doc, indent=1) + "\n")
for r in runs:
    print(r["command"], r["reservoir_worlds_identical"], r["ecology_worlds_identical"], r.get("saves_failing_to_load_in_the_other_mode"))
print(same)
