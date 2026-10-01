# Scale remeasure, 2026-10-01

The 2026-09-25 ecology scale points, run again on today's core to open the
time-at-scale thread (session notes §10). Same founding, seed, mode and tick
count per point; the rules have changed since (rulings 258 to 287), so no
point is identical to its 09-25 state, and the comparison is of cost.

- Core: isometry `099decf`, `shared/isocosm` clean, release build of
  `isocosm-scale`.
- Command, from `shared/isocosm`:
  `isocosm-scale --remeasure ../../mesocosm/testing/bench/receipts/2026-09-25/isocosm/scale.json --remeasure ../../mesocosm/testing/bench/receipts/2026-09-25/isocosm/scale-extension.json --family ecology --output <raw>`
- Raw: `Code/testing/isometry/receipts/2026-10-01/isocosm/remeasure-ecology.json`,
  392,891 bytes, SHA-256
  `48f5959654df5b302ed6a1e01a877be0972bd16981b2009c52832f977752e6b5`.
- Wall time for the whole remeasure: 7.4 s.

## Cost per tick, grouped, 8 lineages

| Members | Run | Ticks | Mean ms a tick | Evaluations a tick | µs an evaluation | 09-25 ms a tick |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 256 | ladder | 16 | 2.0 | 220 | 9.3 | 148.0 |
| 512 | ladder | 16 | 4.0 | 375 | 10.6 | 349.9 |
| 1,024 | ladder | 16 | 7.0 | 667 | 10.4 | 1,071.6 |
| 2,048 | ladder | 16 | 15.2 | 1,230 | 12.4 | 5,859.2 |
| 2,048 | extension | 8 | 14.6 | 1,088 | 13.4 | 4,181.7 |
| 4,096 | extension | 8 | 33.0 | 2,008 | 16.5 | 18,156.6 |

Individual mode runs within 10% of grouped at every size (36.7 ms a tick at
4,096). Fitted over 512 to 4,096 members, cost grows as n^1.02 grouped and
n^1.03 individually, against n^2.09 on 09-25; the fit extrapolates to about
77 ms a tick grouped at 10,000 members, a fit and not a measurement.

## What else the points show

- **Survival.** The history run's 1,024 members are all dead by tick 65,
  with no births, as on 09-25. Every other point runs 8 or 16 ticks and
  keeps every member alive.
- **Grouping decays.** At 4,096 members the groups grow from 383 to 723 in
  eight ticks, from 10.7 to 5.7 members a group; at 2,048 over sixteen
  ticks, 320 to 427 and 470.
- **Blocked evaluations.** Over the 4,096-member run 3,968 of 16,064
  evaluations, 25%, were blocked, 1,984 of 2,720 in its worst tick.
- **Idle ticks.** The scheduler jumps between due ticks (`advance_to`,
  `schedule/mod.rs`), so a tick with nothing due costs only the receipt's
  own bookkeeping.

## Reading for ruling 124

At ruling 256's periods a century is 36,525 daily passes. At this core's
13 µs an evaluation, 300 s buys about 23 million evaluations; one daily
process over 10,000 members at nine members a group needs about 41 million.
A cost-only reading of today's founding, a tick read as a day, puts a
century at 4,096 members near 20 minutes. Neither figure is a century
receipt, which waits on a world that lives that long.
