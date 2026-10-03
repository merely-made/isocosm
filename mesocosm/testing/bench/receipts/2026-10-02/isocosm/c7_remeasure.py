"""Checkpoint 7's ecology control: two isocosm-scale --remeasure receipts
of the same points, compared point by point on everything but time and
heap. Prints each differing point, the mean tick ratio, and exits non-zero
if any point differs.

usage: python c7_remeasure.py EARLIER.json LATER.json"""
import json
import sys

TIMED = ('micros', 'heap')


def key(p):
    f = p['founding']
    return (p['run'], p['family'], p['mode'], f['seed'], f['population'], p['ticks_run'])


def kept(v):
    if isinstance(v, dict):
        return {k: kept(x) for k, x in v.items() if not any(t in k for t in TIMED)}
    if isinstance(v, list):
        return [kept(x) for x in v]
    return v


a, b = (json.load(open(p, encoding='utf-8')) for p in sys.argv[1:3])
earlier = {key(p): p for p in a['points']}
later = {key(p): p for p in b['points']}
if set(earlier) != set(later):
    print('the receipts hold different points')
    sys.exit(1)
differing, ratios = [], []
for k in sorted(earlier, key=str):
    x, y = kept(earlier[k]), kept(later[k])
    fields = sorted(f for f in set(x) | set(y) if x.get(f) != y.get(f))
    if fields:
        differing.append((k, fields))
    ratios.append(later[k]['mean_tick_micros'] / max(1, earlier[k]['mean_tick_micros']))
for k, fields in differing[:20]:
    print(k, 'differs in', fields)
print('%d of %d points differ outside time and heap' % (len(differing), len(earlier)))
print('mean tick, later over earlier: %.2f to %.2f' % (min(ratios), max(ratios)))
sys.exit(1 if differing else 0)
