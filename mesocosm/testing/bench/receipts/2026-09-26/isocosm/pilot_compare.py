"""Compare two probe receipts field by field, masking what is work or time.

Usage: pilot_compare.py <before.json> <after.json>
Masked: every arm's micros, evaluations, represented, accepted and blocked,
and the savings block, which is built from them.
"""
import json, sys

MASK_ARM = {"micros", "evaluations", "represented", "accepted", "blocked"}


def walk(a, b, path, out):
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            if path == "" and k == "savings":
                continue
            if "/arms[" in path and k in MASK_ARM:
                continue
            if k not in a or k not in b:
                out.append(f"{path}/{k}: only in {'after' if k in b else 'before'}")
                continue
            walk(a[k], b[k], f"{path}/{k}", out)
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            out.append(f"{path}: length {len(a)} vs {len(b)}")
        for i, (x, y) in enumerate(zip(a, b)):
            walk(x, y, f"{path}[{i}]", out)
    elif a != b:
        out.append(f"{path}: {str(a)[:80]} vs {str(b)[:80]}")


before = json.load(open(sys.argv[1]))
after = json.load(open(sys.argv[2]))
diffs = []
walk(before, after, "", diffs)
print(f"{len(diffs)} differing fields")
for d in diffs[:40]:
    print(" ", d)
