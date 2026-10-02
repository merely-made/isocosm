# Checkpoint 6, 2026-10-01

S2's checkpoint 6, the minimal-body probe (sim plan §9, rulings 262 and 446
to 456), verified on the branch `checkpoint-6` in
`Code/worktrees/isometry-cp6` before it merged. The large raw receipts live
out of tree, in `Code/testing/isometry/receipts/2026-10-01/isocosm/` on the
machine that ran them (ruling 255). Their recomputations (`*-verify.txt`),
the density ladder's summary (`c6-density-bodies.json`), the scripts that
make them (`verify.py`, unchanged from 2026-09-26, and `c6_density.py`) and
the source of each run (`checkpoint-6-source.json`) stay here. Check a copy
against its hash before relying on it, then recompute with
`python verify.py <copy> 1999` or `python c6_density.py <out> <ladder…>`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `c6-probe-predators.json` | 5,349,910 | `75245934e0277f070033ba82241fa00370b023ddc4b9af952e244c710132e78e` |
| `c6-probe-predators-water.json` | 6,283,645 | `189c4e38ba311b0dca93419e6167c12672bc254ec74b6ef4b050cf3b581124be` |
| `c6-probe-bodies.json` | 3,402,070 | `cac8ecc26b4646bf065a0f4a5740203a0296ccc3745b8efecfc1449956c53ba9` |
| `c6-density-bodies-32.json` | 70,378 | `da104eff50755f535720d9b028bc974ebe722f56f1cc358eb0be038b244bba43` |
| `c6-density-bodies-64.json` | 70,582 | `90f9de12f8b2a360e349d843428344b90e0977dcd50f8f495dc1f97a68aadcd3` |
| `c6-density-bodies-128.json` | 71,042 | `c8f57267947e74a0f75adf4ed0cca3a80ac7a7c6def83e02dbbbe9b55474204e` |
| `c6-density-bodies-256.json` | 71,442 | `1bbafd3fc591e1c95fba24d6c6b881e03eaa94590e86ad1d1add07452996c807` |
| `c6-density-bodies-512.json` | 71,834 | `82961253147c56647220882363439cf6a0df60c4459027ef02917191d8d46516` |
| `c6-remeasure-ecology.json` | 393,894 | `83828594980a1759d571e0ad75db7c8014c944f74224307d9e80a99736a7360d` |

## The certified checks

Each is 1,000 draws from a master seed taken from the clock, 1,999
permutations, the exact runner against the crowd with the positive
(exact against exact), negative (averaged crowd) and draw (unweighted
crowd) controls. `verify.py` recomputed every distance and equivalence
verdict and matched each; the difference verdicts it disagrees with, all
at the 0.05 line under its own permutations, are in controls only.

| Check | Readings | Crowd certified | Positive control | Negative control detected | Draw control detected | Evaluations | Wall |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| Hunting, food (4b's again, ruling 454) | 37 | 37 of 37, none detected | 37 of 37 | 26 | 19 | 2.42x fewer | 5.95x |
| Hunting, food and water | 47 | 47 of 47, none detected | 47 of 47 | 33 | 7 | 1.65x fewer | 5.00x |
| Bodies (checkpoint 6) | 16 | 16 of 16, none detected | 16 of 16 | 5, starvation among them | 6 | 1.42x fewer | 4.39x |

The disagreements: with food, `inspect:alive` in the negative control and
`mind#need:3` in the draw control; with water, `probe:hunt-2#2` in the
negative control. None in the bodies' check.

The hunting checks ran on the branch at `e98c208`, before the language
extensions and the crowd's whole-act hunt (`2987e11`); 16-draw pilots at
seed 20261001, with food and with water, are equal on both builds in every
field but wall time and the note, so the certification holds for the
merged code. The pilots without hunters, food and water at seed 20260926,
equal the branch before ruling 454 (`93e58f6`) in the same way.

## The bodies' check

The largest distance between the crowd and the exact runner is 0.047, the
producers' inspected tissue, against 0.036 between two exact runs. The
averaged crowd, every lineage's reserves at a site replaced by their
average each tick, leaves 39.3 producers alive at the end against 42.9 and
starves 25.7 against 22.2. The unweighted crowd, prey drawn by members
alone, starves 26.6 producers, distance 0.154. Grazers never starve within
24 ticks in this domain, so their alive and starvation readings stand
still in every arm. Both halves of ruling 454's sharing were exercised:
the crowd shared out a prey too small for its grazers 2,249 times in its
1,000 draws, and fixing's soil ran short in 473 of them, 7,896 site
accounts a pass (counted by a separate run of the crowd over the same
worlds and dynamics seeds).

## Savings by density

`isocosm-probe --bodies --density --draws 32 --members N N`, N producers
per site and a quarter as many grazers; `c6-density-bodies.json`.

| Producers a site | Exact evaluations | Crowd | Ratio | Exact s | Crowd s | Wall | Living a state |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 84,708 | 59,420 | 1.43 | 5.37 | 1.36 | 3.9 | 1.15 |
| 64 | 185,315 | 138,162 | 1.34 | 18.47 | 3.50 | 5.3 | 1.14 |
| 128 | 367,037 | 277,170 | 1.32 | 54.42 | 7.03 | 7.7 | 1.18 |
| 256 | 653,971 | 514,785 | 1.27 | 153.0 | 11.74 | 13.0 | 1.13 |
| 512 | 1,384,977 | 1,080,051 | 1.28 | 604.6 | 27.01 | 22.4 | 1.16 |

Once bodies carry cells and amounts are computed of each body, the crowd
finds about 1.15 living members a state at every density, so grouping
saves little; the crowd's widening lead in wall time is the exact
runner's cost per member growing with the population.

## The ecology scale points again

`isocosm-scale --remeasure` over the 09-25 points, as `SCALE_REMEASURE.md`
ran them this morning on `099decf`, now on the branch's core. Evaluations a
tick are identical at every point; the mean tick costs 1.04 to 1.41 times
this morning's (33.0 ms to 41.0 ms grouped at 4,096 members, 36.7 to 48.2
individually), planning the passes that take shared ground and keeping
their start. Fitted over 512 to 4,096 members grouped, cost grows as
n^1.01. Every final state differs, as it must: the epoch boundary (step 4)
and ruling 454's hunts both change these worlds. The history run, 1,024
members over 512 ticks, costs 0.62 times this morning's tick.
