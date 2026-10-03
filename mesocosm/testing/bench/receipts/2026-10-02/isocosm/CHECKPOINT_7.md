# Checkpoint 7, 2026-10-02 to 2026-10-03

S2's checkpoint 7, matter in parts (sim plan §9; rulings 459, 460, 463,
464, 492 to 494 and 504 to 507), verified on the branch `checkpoint-7` in
`Code/worktrees/isometry-cp7` before it merged. The large raw receipts live
out of tree, in `Code/testing/isometry/receipts/2026-10-02/isocosm/` on the
machine that ran them (ruling 255). The certified check finished at 01:50
EDT on 2026-10-03. Its recomputation (`c7-probe-bodies-verify.txt`), those
of the failed run and of ruling 507's two pilots
(`c7-probe-bodies-edbdc0b-failed-verify.txt`, `pilot-507-*-verify.txt`),
the density ladder's summary (`c7-density-bodies.json`), the scripts that
make and compare them (`verify.py` and `c6_density.py`, unchanged from
checkpoint 6; `c7_compare.py`, `c7_remeasure.py` and `c7_faults.py`, new)
and the source of each run (`checkpoint-7-source.json`) stay here. Check a
copy against its hash before relying on it, then recompute with `python
verify.py <copy> 1999`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `c7-probe-bodies.json` | 3,307,583 | `05a17267c853b080f2e46bc5e27f63090b7de745762576df31dd458ceb1746cc` |
| `c7-density-bodies-32.json` | 66,059 | `8068cad328905a98c0809935dcaa942d2f61829dc3be6b1accdb237b4ec6a1ed` |
| `c7-density-bodies-64.json` | 66,310 | `972367a8e7c0b5225d8a1e6e179ff5962989935645c217274f009b16941fd451` |
| `c7-density-bodies-128.json` | 66,546 | `b4d237a29c37ed01e914ddf7fe20495a51aee91c6ab8c2543142c94e00111595` |
| `c7-density-bodies-256.json` | 66,876 | `adec38e7ed8911b08f0bbe1ff9c57db950c7814b3dd5a790e1b6eedaea6e8046` |
| `c7-density-bodies-512.json` | 67,166 | `34fafc9ff4869a2018d66e749fe2572dd7f2b8d63ff501666a06f708c4bd3703` |
| `c7-remeasure-ecology.json` | 394,310 | `7bc9e17eae4a6d11caa4a86111750ab82ceb31a1a59ede42a163af293fcef019` |
| `c7-probe-bodies-edbdc0b-failed.json` | 3,301,568 | `546d6360c978a327a885acaafcc77dc4f961a881569dd7cb227747aefdbcebc5` |
| `pilot-507/avg-own-200.json` | 694,984 | `032eee72329de5a676cac40c85d20dc1018a384df43c6c627db79ded254e3fff` |
| `pilot-507/avg-reserve-200.json` | 695,603 | `866a6d0389ccc2f63ed553a924ccf9f519a03d5e11971972e0fae571c60cd08c` |
| `pilot-food-main.json` | 157,852 | `6004ad5af792effa0448a034932e8c3f1b0be7baf6f700a22359068480a3e425` |
| `pilot-food-cp7.json` | 157,884 | `76125b830404367a21f6638122ee5f02ba2ee6dfa88f2d0f5de6f93c32fa1721` |
| `pilot-water-main.json` | 191,097 | `30f958479aba088768351eb82fa5fcf581fc2ed1b622ef8451f11ef082d8c5ed` |
| `pilot-water-cp7.json` | 191,132 | `bc6a585a6105731aa6b873e4b14097628e784067f11305124e048984ed8a6c42` |
| `ab/ab-main-1.json` | 70,252 | `116f561cd51acb22f72c5e283f4875323cac3ec01ee7f02add47d877f81ccfa0` |
| `ab/ab-main-2.json` | 70,364 | `8dbc51be80e16221ee3d997b312e46b71d3aa88483451d44a6479495256a1db4` |
| `ab/ab-main-3.json` | 70,465 | `d2732f9bd21f9d0c7b94cf796bd87773632db6ec1a357f3433899f4a7021b3e0` |
| `ab/ab-cp7-1.json` | 65,944 | `df3a20051b54cc9c929b05c4e76b182fab9958a32d861db98e72d230729669a3` |
| `ab/ab-cp7-2.json` | 65,968 | `0015bc0abd605716ab9585d230b34954fea6619228bf7ce389e2859197ddfc2c` |
| `ab/ab-cp7-3.json` | 65,985 | `3c96c0b82f96f7ed6402c101c459deec8ef0ea7725d23cd05c94d59a532d4900` |
| `ab/remeasure-main-1.json` | 394,304 | `0ba25abcf2d96b6ac0a906a86b5bfe3de21e33c822783837d66343789d34768f` |
| `ab/remeasure-main-2.json` | 394,304 | `b925927c3c33b31c8aa203b26da8438591e1e0d06c7d3202d99c157db6dcb8a4` |
| `ab/remeasure-cp7-1.json` | 394,309 | `67640ef7064eaf9edbadf5c459c4fea697c9590786e9e13177ba4729fee7d6d2` |
| `ab/remeasure-cp7-2.json` | 394,301 | `4bc344193aae20418c717add74242176e76fd3553af048ed614e5ba8a28f7cab` |

## The certified check

1,000 draws from a master seed taken from the clock, 1,999 permutations,
the exact runner against the crowd with the positive (exact against
exact), negative (averaged crowd) and draw (unweighted crowd) controls.
`verify.py` recomputed every distance and every verdict and matched each;
no difference verdict falls on the other side of the 0.05 line.

| Check | Readings | Crowd certified | Positive control | Negative control detected | Draw control detected | Evaluations | Wall |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| Bodies (checkpoint 7) | 15 | 15 of 15, none detected | 15 of 15 | 6, starvation among them | 6 | 1.77x fewer | 5.40x |

The readings are checkpoint 6's but one: producers keep no reserve (506),
so `inspect:account:reserve:0` is gone. The largest distance between the
crowd and the exact runner is 0.027 (`inspect:place`), against 0.029
between two exact runs (`inspect:account:reserve:1`). The meal's holdings,
the prey's matter as each bite began and now read over its parts, are
9,841 mg against 9,855. The averaged crowd, every lineage's tissue and
reserve at a site replaced by their averages each tick (507), starves 5.32
producers against 8.49 (distance 0.307) and leaves 58.87 alive against
55.70. The unweighted crowd, prey drawn by members alone, starves 14.49
(distance 0.256). Grazers never starve within 24 ticks in this domain. The
crowd shared out a prey too small for its grazers 471 times in 108 of its
1,000 draws (checkpoint 6: 2,249); the site shortfalls checkpoint 6 counted
by a separate run were not counted again.

These are not checkpoint 6's worlds. Under 505 and 506, and with founding
tissue a share of each part's adult mass, producers live longer: 55.70
alive at the end and 8.49 starved, against 42.91 and 22.17.

## Two failed runs before it

The first, on `12690f2` (master seed 1790991944854085300), stopped at draw
137: a founding whose store drew no reserve left the account out, so that
draw read a different set of accounts from draw 0. Every store now keeps
the account (`edbdc0b`).

The second, on `edbdc0b` (master seed 1790993524767155700, 1,000 draws,
`c7-probe-bodies-edbdc0b-failed.json`), failed twice over. The exact
runner's watch weighed a prey's entity ledger for its meal holdings, empty
since matter moved into parts, so it read 0 against the crowd's 9,830: a
fault, fixed at `4d60160`. And the averaged crowd, averaging reserves as
ruling 209 put it, could not move starvation: producers keep no reserve
(506) and starvation reads tissue below 21 mg, so 8.19 against 8.42, not
detected. A 200-draw pilot of each averaging at seed 20261002
(`pilot-507/`) put the choice to Mark: reserve-only averaging left
producers identical to the exact runner (alive 60.65 against 60.65,
starving 6.29 against 6.28), and averaging tissue and reserve cut their
starvation to 3.25 (distance 0.335). He ruled all own matter (507).

## What did not change

The hunting family's parts have no extents, so its members keep their
entity ledgers (504). Its 16-draw pilots at seed 20261001, with food and
with water, are equal on main's build and on `4d60160` in every field but
wall time and the note (`c7_compare.py`); food against water, the control,
differs in 4,235 fields.

The ecology family's 29 scale points, run again with
`isocosm-scale --remeasure` over the 09-25 receipts, are identical to
checkpoint 6's remeasure in every final state hash, per-tick count and
distinct state (`c7_remeasure.py`). The comparator finds 0 of 29 points
differing between a receipt and itself, and 29 of 29 between checkpoint
6's remeasure and the morning's before it.

## Cost

Other sessions were compiling on the machine through these runs, so the
times here are not comparable with checkpoint 6's: the ecology points'
mean tick came out 2.26 to 2.94 times checkpoint 6's receipt, and one
build, `4d60160`'s, took 48.5 seconds for the density ladder's 32-producer
point and 10 to 17 for the same point an hour later. Interleaved runs of
main's build and this branch's, sharing the same load, measure the change
instead:

- The ecology points (`ab/remeasure-*`), main then the branch, twice: every
  state identical, and the branch's mean tick 1.01 and 1.04 times main's
  at the median point (0.74 to 1.20). At 4,096 members grouped, 103.8
  against 107.1 ms and 103.9 against 115.7.
- The bodies at 32 producers a site, 32 draws (`ab/ab-*`), seeds 20261001
  to 20261003: the branch's exact runner spends 82 µs an evaluation
  against main's 109, its crowd 31 against 40. A draw runs about 1.6
  times the evaluations, since more producers live, so it costs about
  1.3 times main's.

## Savings by density

`isocosm-probe --bodies --density --draws 32 --members N N`, N producers
per site and a quarter as many grazers; `c7-density-bodies.json`.

| Producers a site | Exact evaluations | Crowd | Ratio | Exact s | Crowd s | Wall | Living a state |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 142,227 | 80,312 | 1.77 | 48.5 | 11.3 | 4.3 | 1.44 |
| 64 | 303,178 | 172,369 | 1.76 | 114.1 | 17.0 | 6.7 | 1.57 |
| 128 | 552,262 | 292,875 | 1.89 | 311.3 | 36.2 | 8.6 | 1.80 |
| 256 | 1,148,279 | 541,512 | 2.12 | 1,170.6 | 67.4 | 17.4 | 2.19 |
| 512 | 2,384,841 | 902,280 | 2.64 | 1,923.2 | 50.0 | 38.5 | 2.81 |

Unlike checkpoint 6's, where about 1.15 living members shared a state at
every density, the crowd now finds more to group as the population grows,
1.44 living members a state at 32 producers a site and 2.81 at 512, so the
evaluations it saves grow from 1.77 to 2.64 times. The evaluations are
exact counts; the times were taken under the load above, so the wall
column is indicative only.

## Controls

Each check that could pass by saying nothing was shown to fail. Planted
one at a time by `c7_faults.py`, each fault was caught by the test named,
and each was restored:

| Fault planted | Caught by |
| --- | --- |
| A take spread evenly, not by what each part holds | `anatomy::a_take_spreads_over_the_parts_by_what_each_holds` |
| The reserve bounded by the whole part, not its store cells | `anatomy::the_reserve_lives_only_in_what_stores` |
| A bite drawn evenly among parts, not by what each holds | `anatomy::a_bite_lands_on_a_part_drawn_by_what_each_holds` |
| A rod with one child read as a branch | `anatomy::a_part_is_named_by_its_box_and_the_tree` |
| Flow legs left whole, not split among the parts | `flows::the_record_names_the_parts_a_bodys_matter_moves_through` ("the record accounts for -13") |
| A body's books leaving out its parts | `probe_bodies::bodies_run_alike_grouped_and_individually` |
| Fixing at 5 mg per 63 faces, not 4 | `calibration::the_median_body_earns_what_it_did` |
| Grazing at 12 mg per 294 voxels, not 11 | the same |

Two of these found a hole in their instrument first. The books fault passed
at first: the core refused every act that moved matter ("matter invariant
would be violated"), the world stood still, and its matter held trivially.
The test now watches producers fix and pay rent and fails when either never
ran. The calibration test allowed a median a milligram off, and on medians
of 4 and 11 mg it passed both rate faults, fixing a quarter over and
grazing an eleventh over; both medians equal checkpoint 6's exactly, so it
now requires equality, which catches both. The script itself had one: it
restored each file from a copy keeping the copy's older time, cargo, which
fingerprints by time, kept the last faulted build, and the gates failed on
restored code until the files were touched. It now stamps each restored
file.

The two faults the certified check found have tests of their own, each
shown failing on the code before its fix: a watched graze that weighs its
prey at nothing (the old watch fails at seed 0, tick 1), and an averaged
crowd that leaves any own account unflattened at a site (reserve-only
averaging fails, producers' tissue spread from 84 to 182 mg at one site).
Genesis refuses a declared name its box and tree do not read and a body
with parts keeping its own matter on its ledger, each beside a world that
validates.
