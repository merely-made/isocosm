"""Checkpoint 7's hunting control: two probe receipts equal in every field
but wall time and the note. Prints each difference outside those and exits
non-zero if there is one.

Usage: python c7_compare.py <a.json> <b.json>
"""
import json, sys

IGNORED = ('wall', 'note', 'seconds', 'elapsed', 'nanos', 'millis', 'micros')


def ignored(key):
    k = str(key).lower()
    return any(word in k for word in IGNORED)


def walk(a, b, path, out):
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b), key=str):
            if ignored(k):
                continue
            if k not in a or k not in b:
                out.append('%s.%s present in one only' % (path, k))
                continue
            walk(a[k], b[k], '%s.%s' % (path, k), out)
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            out.append('%s: lengths %d and %d' % (path, len(a), len(b)))
        for i, (x, y) in enumerate(zip(a, b)):
            walk(x, y, '%s[%d]' % (path, i), out)
    elif a != b:
        out.append('%s: %r against %r' % (path, a, b))


a, b = (json.load(open(p, encoding='utf-8')) for p in sys.argv[1:3])
diffs = []
walk(a, b, '', diffs)
for d in diffs[:20]:
    print(d)
print('%d differences outside wall time and the note' % len(diffs))
sys.exit(1 if diffs else 0)
