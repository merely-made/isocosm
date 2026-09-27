"""Two probe receipts compared in every field but wall times and what is
derived from them; prints each difference.

usage: python same_but_time.py A B"""
import json, sys

TIME = {"micros", "wall_ratio", "wall_ratio_to_exact", "wall_ratio_to_crowd", "per_draw_wall_ratio_to_crowd"}


def walk(a, b, path, out):
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            if k in TIME:
                continue
            if k not in a or k not in b:
                out.append(f"{path}/{k}: only one side")
                continue
            walk(a[k], b[k], f"{path}/{k}", out)
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            out.append(f"{path}: length {len(a)} vs {len(b)}")
        for i, (x, y) in enumerate(zip(a, b)):
            walk(x, y, f"{path}[{i}]", out)
    elif a != b:
        out.append(f"{path}: {str(a)[:50]} vs {str(b)[:50]}")


a, b = (json.load(open(p)) for p in sys.argv[1:3])
out = []
walk(a, b, "", out)
print(len(out), "fields differ apart from time")
print("\n".join(out[:12]))
