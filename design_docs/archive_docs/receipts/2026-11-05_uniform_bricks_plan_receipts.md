# Uniform bricks: receipts

Run evidence for `design_docs/2026-11-05_uniform_bricks_plan.md`.

## 2026-11-05: baseline

Temporary `#[ignore]` tests appended to
`shared/isometer/crates/isometer-core/src/ground/columns.rs`, run with
`cargo test --release -p isometer-core --lib uniform_measure -- --ignored --nocapture`
from `shared/isometer`, then reverted. Terrain: surface
`30 + ((x / 9 + z / 13) % 7)`, no cavities, `Ground::grow`.

Timing run, extent 160:

```text
bricks 7478 uniform 4869 grow 215.3973ms clone 2.59008ms encode 6.8707ms bytes 3866132 hash 5.2275ms
```

Memory run: a counting global allocator; "collapsed" is a
`BTreeMap<[i16; 3], enum { Uniform(u8), Dense(Box<[u8; 512]>) }>` built from
the same ground.

```text
1x extent 160 bricks 7478 uniform 4869 now 4.2 MB collapsed 1.5 MB grow 119.3342ms
10x extent 506 bricks 72819 uniform 48115 now 41.4 MB collapsed 14.4 MB grow 1.9503156s
100x extent 1600 bricks 715705 uniform 484454 now 408.5 MB collapsed 135.6 MB grow 22.9966928s
```

Eponym's world, measured earlier the same day: 20,610 bricks, about 70%
uniform soil.

## 2026-11-05: UB1, brick-wise growth

Same terrain and extents, temporary `#[ignore]` test in `columns.rs`, best of
three runs each, release, then reverted. The baseline above was a single run.

```text
1x extent 160 bricks 7478 grow best-of-3 15.2604ms
10x extent 506 bricks 72819 grow best-of-3 190.5944ms
100x extent 1600 bricks 715705 grow best-of-3 2.3862904s
```

Positive control: removing every brick layer 1 in `fill_columns` made
`brick_wise_growth_matches_the_per_voxel_path` fail (panic at `fill.rs:126`);
restored. `cargo test -p isometer-core`: 51 passed.
