# Scene board paging receipts

2026-09-27 final integration run. Regenerated with `receipts.summarise` from
`lane-e-final-paging/release-cost.stdout.log` and `.stderr.log`; see `integration.md`.
Medians over each run's samples. Every timing is one build on one
machine and moves with both; every count and byte figure does not.

Here, `headed` names a headless harness using headed-pane dimensions.
`frame` measures `Board::draw` plus completion wait, not full host
input-to-present latency. The six native host sessions are recorded
separately in [integration.md](integration.md).

## One click at 256 by 256

A click selects a tile, which tints its top voxel. Before paging it
raised the whole ground and compared it brick by brick; now it reads
one column per tile, diffs them and remakes the held bricks it touched.

| build | board | pane | before: regrow + diff, ms | now: columns + diff + fill, ms | frame, ms | slots | upload, bytes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| release | flat | harness | 320.90 | 0.51 | 1.28 | 3.5 | 1,806 |
| release | flat | headed | 320.90 | 0.68 | 2.37 | 4.0 | 2,064 |
| release | relief | harness | 806.16 | 0.47 | 1.31 | 8.0 | 4,128 |
| release | relief | headed | 806.16 | 0.69 | 2.54 | 8.0 | 4,128 |

## One raise, one pan

A raise lifts one tile a step. A pan moves the camera a tile across or
half a tile down: it reads nothing and retargets, a shrinking framing
included, and rebuilds only when the camera outgrows the pointer volume.

| build | board | pane | kind | n | columns + diff + fill, ms | frame, ms | loaded | retargets | rebuilds | upload, bytes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| release | flat | harness | raise | 6 | 0.45 | 1.15 | 0.0 | 0 | 0 | 774 |
| release | flat | harness | pan | 12 | 0.01 | 1.23 | 24.0 | 12 | 0 | 29,960 |
| release | flat | headed | raise | 6 | 0.62 | 2.21 | 0.0 | 0 | 0 | 1,032 |
| release | flat | headed | pan | 12 | 0.02 | 2.48 | 37.0 | 12 | 0 | 54,856 |
| release | relief | harness | raise | 6 | 0.59 | 1.77 | 1.5 | 5 | 0 | 29,952 |
| release | relief | harness | pan | 12 | 0.02 | 1.24 | 30.5 | 12 | 0 | 43,264 |
| release | relief | headed | raise | 6 | 0.68 | 2.47 | 0.0 | 0 | 0 | 2,064 |
| release | relief | headed | pan | 12 | 0.06 | 3.26 | 56.5 | 12 | 0 | 84,416 |

## What the frame holds

| build | board | pane | bricks on the board | resident | capacity | past the atlas | first frame, ms | first fill, ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| release | flat | harness | 25600 | 682 to 720 | 16383 | 0 | 30.30 | 0.35 |
| release | flat | headed | 25600 | 1554 to 1613 | 16383 | 0 | 30.15 | 0.70 |
| release | relief | harness | 39482 | 1008 to 1132 | 16383 | 0 | 28.05 | 0.71 |
| release | relief | headed | 39482 | 2413 to 2504 | 16383 | 0 | 33.73 | 1.66 |

## What each spare layer of headroom costs

The pointer volume reserves brick layers above the board's tallest tile,
so an edit that lifts it into them retargets rather than rebuilding the
map. The default, one spare layer, is provisional: Mark takes it back with
these numbers. A spare layer widens the volume a little as well as raising
it, because the camera leans. The pointer volume is uploaded whole with
every retarget, so its bytes are also what each pan uploads before its
bricks. The atlas does not move with the headroom.

| board | pane | headroom | reserved layers | pointer extent | pointer, bytes | per layer, bytes | atlas, bytes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| demo | harness | 0 | 0 to 1 | 16 by 2 by 16 | 2,048 | - | 8,388,608 |
| demo | harness | 1 | 0 to 2 | 16 by 3 by 16 | 3,072 | +1,024 | 8,388,608 |
| demo | harness | 2 | 0 to 3 | 16 by 4 by 16 | 4,096 | +1,024 | 8,388,608 |
| demo | harness | 3 | 0 to 4 | 16 by 5 by 16 | 5,120 | +1,024 | 8,388,608 |
| demo | headed | 0 | 0 to 1 | 16 by 2 by 16 | 2,048 | - | 8,388,608 |
| demo | headed | 1 | 0 to 2 | 16 by 3 by 16 | 3,072 | +1,024 | 8,388,608 |
| demo | headed | 2 | 0 to 3 | 16 by 4 by 16 | 4,096 | +1,024 | 8,388,608 |
| demo | headed | 3 | 0 to 4 | 16 by 5 by 16 | 5,120 | +1,024 | 8,388,608 |
| flat | harness | 0 | 0 to 0 | 46 by 1 by 46 | 8,464 | - | 8,388,608 |
| flat | harness | 1 | 0 to 1 | 47 by 2 by 47 | 17,672 | +9,208 | 8,388,608 |
| flat | harness | 2 | 0 to 2 | 48 by 3 by 48 | 27,648 | +9,976 | 8,388,608 |
| flat | harness | 3 | 0 to 3 | 49 by 4 by 49 | 38,416 | +10,768 | 8,388,608 |
| flat | headed | 0 | 0 to 0 | 65 by 1 by 65 | 16,900 | - | 8,388,608 |
| flat | headed | 1 | 0 to 1 | 67 by 2 by 67 | 35,912 | +19,012 | 8,388,608 |
| flat | headed | 2 | 0 to 2 | 68 by 3 by 68 | 55,488 | +19,576 | 8,388,608 |
| flat | headed | 3 | 0 to 3 | 69 by 4 by 69 | 76,176 | +20,688 | 8,388,608 |
| relief | harness | 0 | 0 to 1 | 47 by 2 by 47 | 17,672 | - | 8,388,608 |
| relief | harness | 1 | 0 to 2 | 48 by 3 by 48 | 27,648 | +9,976 | 8,388,608 |
| relief | harness | 2 | 0 to 3 | 49 by 4 by 49 | 38,416 | +10,768 | 8,388,608 |
| relief | harness | 3 | 0 to 4 | 51 by 5 by 51 | 52,020 | +13,604 | 8,388,608 |
| relief | headed | 0 | 0 to 1 | 67 by 2 by 67 | 35,912 | - | 8,388,608 |
| relief | headed | 1 | 0 to 2 | 68 by 3 by 68 | 55,488 | +19,576 | 8,388,608 |
| relief | headed | 2 | 0 to 3 | 69 by 4 by 69 | 76,176 | +20,688 | 8,388,608 |
| relief | headed | 3 | 0 to 4 | 70 by 5 by 70 | 98,000 | +21,824 | 8,388,608 |
