# Checkpoint 9, 2026-10-05 to 2026-10-09

S2's checkpoint 9, systems (sim plan §9; its seven rounds of 2026-10-05,
rulings 560 to 590, and rulings 655 to 664), verified on the branch
`checkpoint-9` in `Code/worktrees/isometry-cp9` before it merged. It was
parked on 2026-10-05 (591) and unparked with the open bugs on 2026-10-08
(656). The large raw receipts live out of tree, in
`Code/testing/isometry/receipts/2026-10-08/isocosm/` on the machine that ran
them (ruling 255). The certified check finished at 03:34 EDT on
2026-10-09. Its
recomputation (`c9-probe-bodies-verify.txt`) and the pilots'
(`pilot-c9-20-verify.txt`, `pilot-c9-100-verify.txt`), the density
ladder's summary (`c9-density-bodies.json`), the scripts that make and check
them (`verify.py` and `c6_density.py`, unchanged from checkpoint 8;
`c9_faults.py` with its record `c9-faults.txt`, new), the measurements made
while building it (`c9_measure.rs`) and the source of each run
(`checkpoint-9-source.txt`) stay here. Check a copy against its hash before
relying on it, then recompute with `python verify.py <copy> 1999`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `c9-probe-bodies.json` | 4,479,355 | `1b330e1d35214870a05544e3afbb341f10ff99f3e479b37aa28d73604f6ce2a4` |
| `c9-density-bodies-32.json` | 147,201 | `8898e39550c6b674f3f64dea96f0af2241afbb2df18615389fa6650205812340` |
| `c9-density-bodies-64.json` | 147,796 | `b733c04cc76b1d3778a18b2f615e5b4790490547b0cced0bed9d2db687830664` |
| `c9-density-bodies-128.json` | 148,221 | `ccf3409389bab7941064830f7006fdbcf9d2dc488add13f561f900326494a2ac` |
| `c9-density-bodies-256.json` | 148,580 | `a7095040534187b09ac1a8f0a82b1dafd84e0fb82e5524e88b30c693690b8940` |
| `c9-density-bodies-512.json` | 148,915 | `79f6968178c2ae907e727f4b04fbf5cf3adf76fba25b846a6fd55eabd5469895` |
| `pilot-c9-20.json` | 215,488 | `b2b1222dce04ca9850fc6ffb9d9733deacc2f2078d11ea76a32d4d7f0a658c7d` |
| `pilot-c9-100.json` | 563,673 | `96aef617791c976d93cbb184c926a11b4a7a623ef864081d6e4fec20bb5bbcb2` |
| `isocosm-probe-c9.exe` | 3,446,784 | `10144aca072598ccab4f39207edee2bb9da7b9e3cca985f77a730c1298d3c5ca` |

## The certified check

1,000 draws from a master seed taken from the clock (1791510586298610900),
1,999 permutations, the exact runner against the crowd with the positive
(exact against exact), negative (averaged crowd) and draw (unweighted
crowd) controls, 60 ticks a draw (517), from source `d5eb9440`.
`verify.py` recomputed every distance and every verdict and matched each,
and no recomputed p-value fell on the other side of 0.05 from the
receipt's.

| Check | Readings | Crowd certified | Positive control | Negative control detected | Draw control detected | Evaluations | Wall |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| Bodies (checkpoint 9) | 28 | 28 of 28, none detected | 28 of 28 | 14, starvation among them | 11 | 1.16x fewer | 4.06x |

The readings grew from 26 to 28, `body:graze-0#3` and
`body:graze-0#meal-holdings`: any body that takes in may graze another
lineage (590), so the probe now reads the first lineage's grazing beside
the second's. Its meal reads zero in every draw of both arms and certifies
at distance 0.

The largest distance between the crowd and the exact runner is 0.036
(`inspect:account:tissue:0`, 36.28 mg against 37.27), against 0.032 between
two exact runs on the same reading. The meal's holdings are 12,000 mg
against 12,048, and producers starve 27.48 a draw against 27.21. The
averaged crowd starves 0.02 grazers against 5.06 (distance 0.294) and leaves
34.23 living members of the carried ability against 45.24; its meal falls
to 9,534 mg. The unweighted crowd, prey drawn by members alone, starves
36.86 producers against 27.48 (distance 0.203) and its meal falls to 10,180
mg.

## Before it

Two pilots at fixed seeds, from source `d5eb9440`: 20 draws (seed
20261008) and 100 (seed 20261009), 28 readings each. At 20 draws the
largest distance between the crowd and the exact runner was 0.25
(`inspect:cells:function:fix`, 3.45 against 5.85), against 0.25 between two
exact runs (`inspect:account:tissue:1`), and nothing was detected, the
controls included. At 100 the largest was 0.10 (the fixing cells, 4.06
against 3.67, and `inspect:lineage`), against 0.13 between two exact runs;
nothing was detected between the crowd and the exact runner, while the
averaged crowd's starvation (`body:starve-1#2`, 2.34 against 0.00, p 0.014)
and the unweighted crowd were both detected. The crowd made 1.16 times
fewer evaluations in both, its wall time 3.67 and 4.21 times shorter. The
full run was started after.

## What building it found

Unparking the branch onto main and finishing the nervous system's joining
(659 to 664) found these, each now pinned by a test:

- The nervous system routed `contract`, so a lamed grazer still carried the
  limbs' share through its eye's one cell and the muscular control could
  not fail. Ruled by 657: the nervous and respiratory systems name no
  effects and carry no matter, and a system naming no effects is realized
  by its sources alone.
- A carriage landing nothing (the limbs' share, the glandular dose)
  weighted its parts by their cells and apportioned the ask under a cap of
  each part's weight, so an 11 mg share arrived as 6 mg, one per
  contracting cell. Cells now weigh a share scaled by the ask.
- Two of checkpoint 8's feeding tests held ruling 505's rates, which 571
  now draws per world. They now found their world at those rates, with
  carriage that never binds; the draws and bounds are certified in
  `systems.rs`.
- A test that grew a frond's gland by hand left it without the system its
  cells realize (587). It now takes the system up as a development would.

## Planted faults

Eleven faults, each a one-line edit the named test must catch, in
`c9_faults.py`, recorded in `c9-faults.txt`. The first pass caught nine.
The nervous system carrying the limbs' share again (657) was missed because
the muscular control asked only that a lamed grazer eat less on some seed;
it now holds that a lamed grazer bites exactly as a limbless one, and
catches it. A part carrying whatever its cells (560, 564) named the wrong
test, one that severs a part and so never reads capacity; the trunk test
pins capacity and catches it. All eleven are caught.

## Savings by density

`isocosm-probe --bodies --density --draws 32 --members N N`, N producers
per site and a quarter as many grazers, 60 ticks; `c9-density-bodies.json`.

| Producers a site | Exact evaluations | Crowd | Ratio | Exact s | Crowd s | Wall | Living a state |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 694,535 | 582,987 | 1.19 | 140.4 | 40.8 | 3.4 | 1.03 |
| 64 | 1,397,722 | 1,234,575 | 1.13 | 486.3 | 99.2 | 4.9 | 1.04 |
| 128 | 2,989,217 | 2,589,820 | 1.15 | 1,273.8 | 155.7 | 8.2 | 1.08 |
| 256 | 5,942,678 | 5,191,169 | 1.14 | 5,329.1 | 410.0 | 13.0 | 1.06 |
| 512 | 10,786,379 | 9,062,190 | 1.19 | 13,136.7 | 578.1 | 22.7 | 1.05 |

The regime is checkpoint 8's: nearly every living member is a state of its
own, 1.03 to 1.08 a state at the medians, so the crowd saves 1.13 to 1.19
times the evaluations, against 1.15 to 1.21 then. Bodies now carry systems
that vary at birth (578) and riff (573), which separates members further.
The exact runner made 1.16 to 1.42 times checkpoint 8's evaluations at each
density (*inference:* more acts land now that the systems carry them part
by part), and its wall time grew more; the crowd's wall advantage widened to 3.4 to 22.7
times, against 3.1 to 19.8.

## Cost

Other sessions were working on the machine through these runs, so the
times are for comparison within this checkpoint only. A certified draw of
60 ticks took 20.7 s on average, the run 5.7 hours.
