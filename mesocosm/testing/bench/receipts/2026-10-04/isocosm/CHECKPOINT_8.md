# Checkpoint 8, 2026-10-03 to 2026-10-04

S2's checkpoint 8, development (sim plan §9; rulings 510 to 531, 535, 544
and 547 to 554), verified on the branch `checkpoint-8` in
`Code/worktrees/isometry-cp8` before it merged. The large raw receipts live
out of tree, in `Code/testing/isometry/receipts/2026-10-04/isocosm/` on the
machine that ran them (ruling 255). The certified check finished at 18:07
EDT on 2026-10-04. Its recomputation (`c8-probe-bodies-verify.txt`) and the
100-draw pilot's (`pilot-c8-100-verify.txt`), the density ladder's summary
(`c8-density-bodies.json`), the scripts that make and check them
(`verify.py` and `c6_density.py`, unchanged from checkpoint 6;
`c8h_faults.py`, `c8i_faults.py` and `c8_calibrate.py` with its rows, new),
the measurements behind rulings 551 to 554 (`c8_measure.rs`) and the source
of each run (`checkpoint-8-source.json`) stay here. Check a copy against its
hash before relying on it, then recompute with `python verify.py <copy>
1999`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `c8-probe-bodies.json` | 4,294,573 | `73d4b6befa72fc586c143c35be99195e8766e889fb486b192c13471f89fcbe4e` |
| `c8-density-bodies-32.json` | 117,868 | `d62a22194069e0674b8f26658462e8869162d3febd38dc98a912fba0471a0cd4` |
| `c8-density-bodies-64.json` | 118,501 | `703142061c3d33e749ea784c363b6e6324ae5d0d099bdcda3b8bd4f1c799c705` |
| `c8-density-bodies-128.json` | 118,794 | `3f99ecaf4f9048bb31c2a2c0d1a1971f226aaf04ce90905f977e18a622158513` |
| `c8-density-bodies-256.json` | 119,125 | `1a5b7538fe0e1d897c7b0b3eb1cdef14594df2e42aa89b8b7b6e1e3c3d52f952` |
| `pilot-c8-20.json` | 182,312 | `fa2db035ca371b69653d250a7453af2d7c8e5e6418d4315805f04451cb5b74d5` |
| `pilot-c8-100.json` | 517,878 | `4765a1bc46e08ce84840795f6524b4946fc3e595f69908b5f4fabd7a55bcc4b6` |
| `isocosm-probe-c8.exe` | 3,006,976 | `aae7da31b7578d291bd8127a53896235317dc9dad05ffbb947ebb3ff46642172` |

## The certified check

1,000 draws from a master seed taken from the clock (1791137636009757700),
1,999 permutations, the exact runner against the crowd with the positive
(exact against exact), negative (averaged crowd) and draw (unweighted
crowd) controls, 60 ticks a draw (517). `verify.py` recomputed every
distance and every verdict and matched each.

| Check | Readings | Crowd certified | Positive control | Negative control detected | Draw control detected | Evaluations | Wall |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| Bodies (checkpoint 8) | 26 | 26 of 26, none detected | 26 of 26 | 17, starvation among them | 10 | 1.18x fewer | 3.58x |

The readings grew from 15 to 26. A lineage's traits now carry its life
history (548), so the probe counts the living by each: a producer's
`strategy:bud` and a grazer's `strategy:brood`, `strategy:egg` and
`care:milk` read the same members as its identity. The new natives add their
thresholds (bear, bud, wean), and the inspection reads the provision
accounts. Some read zero in every draw of both arms, such as the
provisioned bear thresholds at the run's end and the inspected soil; they
certify at distance 0.

The largest distance between the crowd and the exact runner is 0.031
(`inspect:account:tissue:1`, 60.57 mg against 66.35), against 0.024 between
two exact runs (`inspect:cells:function:fix`). The meal's holdings are
12,370 mg against 12,371. Producers starve 25.79 a draw in both. The averaged
crowd, each lineage's own matter at a site filled to one level after every
tick (507), starves no grazer against 3.67 (distance 0.297) and leaves
33.26 grazers alive against 44.84. The unweighted crowd, prey drawn
by members alone, starves 35.28 producers against 25.79 (distance 0.221).

One recomputed p-value falls on the other side of 0.05 from the receipt's:
the averaged crowd's `inspect:account:tissue:1`, Holm-adjusted 0.044 in the
receipt and 0.055 recomputed, each from its own 1,999 permutations. It is
the permutation test's own noise on a negative-control reading, and no
verdict turns on it: the control's starvation readings are detected far
beyond it.

The receipt's note still describes the averaged crowd as checkpoint 6's,
averaging reserves; since 507 it averages all of a lineage's own matter,
and since this checkpoint it fills each class to one level as far as each
body may hold. The binary's text was not changed for this run.

## Before it

Two pilots at fixed seeds: 20 draws (seed 20261004) and 100 (seed
20261005). At 20 draws the inspected grazer's tissue read 56.6 mg against
116.8 (distance 0.20); at 100 it read 55.5 against 60.6 (0.08), and nothing
was detected. The full run was started after.

## What building it found

The probe on recipes (8h) found three defects and one stuck rule, each now
pinned by a test:

- Starvation read rent minus funds as a computed query, true whenever it
  is not zero, so every body starved at tick 1. Fixed, with a boundary
  test.
- A pass landed a part an act took whole, or severed as a bud, as the pass
  found it, though an earlier act of the pass had bitten it, and put back a
  part an earlier act had removed. Both made matter, the core's own
  conservation assertion firing. Fixed in the core's pass frame, with two
  tests that run such passes.
- No probe grazer could ever be fed: TD5's horizon of 100 ticks of rent
  exceeded every store (0 of 712), so no grazer brooded, gave milk or took
  a part whole. Ruled by 551.
- No bud severed within a run, a whole frond filled a provision at a time
  in worlds whose matter mostly could not fill their producers. Ruled by
  552 and 553.

Teaching the crowd (8i) found two more: the crowd clipped a bite to what its
part still held where the core clips to what it held as the pass began and
then refuses an act that no longer fits; and the crowd marked a once-only
producer dead before taking its seedling from it, so the seedling was born
dead. Both are fixed and tested. The milk question, the crowd keeping no
identities, was ruled by 554.

## Planted faults

Twenty faults, each caught by the test named beside it in `c8h_faults.py`
(12) and `c8i_faults.py` (8). Four were caught only after a test was added
or strengthened: the hungry-parent fault first failed its test only through
the pass defect above, and is now caught by a direct nursing test; a
removed part's later change needed a pass test that gives to it; a parent
nursing another's young needed two parents in the nursing test; and the
crowd's bite clipping needed a direct test, no single-frond meal showing it.

## Savings by density

`isocosm-probe --bodies --density --draws 32 --members N N`, N producers
per site and a quarter as many grazers, 60 ticks; `c8-density-bodies.json`.

| Producers a site | Exact evaluations | Crowd | Ratio | Exact s | Crowd s | Wall | Living a state |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 597,649 | 517,636 | 1.15 | 77.2 | 24.9 | 3.1 | 1.05 |
| 64 | 1,187,202 | 1,027,825 | 1.16 | 213.6 | 48.9 | 4.4 | 1.04 |
| 128 | 2,102,477 | 1,807,522 | 1.16 | 579.2 | 84.2 | 6.9 | 1.06 |
| 256 | 4,217,195 | 3,597,754 | 1.17 | 1,992.9 | 170.3 | 11.7 | 1.10 |

The 512-producer point was still running at the merge and is added when it
finishes.
Checkpoint 7 grouped 1.44 to 2.81 living members a state over the same
densities and saved 1.77 to 2.64 times the evaluations. Checkpoint 8's
bodies differ member by member: each cohort draws its soma within its
recipe's variance and absence odds, each child draws its own, bodies grow
toward their recipes, and bonded grazers run one by one (554). So nearly
every living member is a state of its own, and the crowd saves about 1.16
times the evaluations at every density. Its wall time still falls further
behind the exact runner's as density grows, 3.1 to 11.7 times, since an
evaluation over a state costs it less than one over a member costs the
exact runner. This is the regime 439 named: grouping stops paying below a
world rule's members per state.

## Cost

Other sessions were working on the machine through these runs, so the
times are for comparison within this checkpoint only. A certified draw of
60 ticks took 14.0 s on average, the run 3.9 hours.
