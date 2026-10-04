"""Ruling 505's rates calibrated again for checkpoint 8's recipe bodies
(513). c8_calibration_rows.txt is the calibration test's founded bodies,
seeds 1 to 1,000, one row each: lineage (0 producers, 1 grazers), the rate
checkpoint 6's tissue formula gives it, and for a producer its fixing area,
for a grazer its intake volume, priced mass and ceiling, then its cohort's
count. For each lineage this finds the range of rates whose member-weighted
median equals checkpoint 6's, by bisection, and prints its middle as the
nearest fraction with a denominator of at most 400: 11/144 per face and
12/269 per voxel, the medians 5 and 12 mg.

usage: python c8_calibrate.py"""
import os
from fractions import Fraction

here = os.path.dirname(os.path.abspath(__file__))
rows = [[], []]
for line in open(os.path.join(here, 'c8_calibration_rows.txt')):
    _, i, old, a, b, c, n = line.split()
    rows[int(i)].append((int(old), int(a), int(b), int(c), int(n)))


def median(values):
    values = sorted(values)
    total = sum(n for _, n in values)
    seen = 0
    for v, n in values:
        seen += n
        if seen * 2 >= total:
            return v


def med(i, r):
    return median([(max(1, (r.numerator * a * b) // (r.denominator * c)), n)
                   for _, a, b, c, n in rows[i]])


def threshold(i, value):
    """The least rate (to 1e-9) whose median reaches `value`."""
    lo, hi = Fraction(0), Fraction(1)
    while hi - lo > Fraction(1, 10**9):
        mid = (lo + hi) / 2
        if med(i, mid) >= value:
            hi = mid
        else:
            lo = mid
    return hi


for i in (0, 1):
    target = median([(r[0], r[4]) for r in rows[i]])
    lo, hi = threshold(i, target), threshold(i, target + 1)
    # The simplest fraction in [lo, hi): smallest denominator, then numerator.
    simplest = None
    for d in range(1, 2000):
        n = -(-lo.numerator * d // lo.denominator)
        if Fraction(n, d) < hi and med(i, Fraction(n, d)) == target:
            simplest = Fraction(n, d)
            break
    mid = (lo + hi) / 2
    near = mid.limit_denominator(400)
    print(i, 'target', target, 'interval', float(lo), float(hi),
          'simplest', simplest, 'mid', float(mid), 'near-mid/400', near,
          'check', med(i, simplest), med(i, near))
    for prev in (Fraction(4, 63), Fraction(11, 294)):
        print('   prev', prev, float(prev), med(i, prev))
