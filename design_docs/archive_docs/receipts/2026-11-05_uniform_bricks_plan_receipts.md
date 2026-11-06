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

## 2026-11-05: UB4, Mesocosm's ground clones

A Sonnet lane added per-site `AtomicU64` counters in `mesocosm-genet`
(`MESOCOSM_CLONE_COUNT=1`), ran from `mesocosm/`, then reverted them:
`mesocosm-genet.exe --play --seed 1793177320550931600 --mode {creative|survival} [--dev --paused] --frames 900 --no-trace`.
`--seed 1` and clock seeds fail A1 candidate selection.

| Scene | State | Ground clones | `refresh_scene` calls | `brick_count()` at clone |
|---|---|---|---|---|
| creative | running | 15 in about 15 s | 894 | 0 |
| survival | running | 15 in about 15 s | 805 | 0 |
| creative | paused | 1 (startup) in about 24 s | about 900 | 0 |
| survival | paused | 1 (startup) in about 24 s | about 899 | 0 |

Median frame interval, dev build, busy machine: about 8 ms running, about
23.6 ms paused (the dev lane's pacing, unexamined).

## 2026-11-05: UB2 and UB3, the uniform form and its encoding

Same terrain, temporary `#[ignore]` test with a counting allocator appended to
`columns.rs`, release, then reverted. Memory is the live heap after one grow.

```text
1x extent 160 bricks 7478 uniform 4869 mem 1.6 MB grow best-of-3 6.6703ms
1x clone 653.95µs encode 753.2µs bytes 1375813 hash 1.2605ms 29a6ac119755a9b2 kept 10
10x extent 506 bricks 72819 uniform 48115 mem 15.8 MB grow best-of-3 72.0348ms
100x extent 1600 bricks 715705 uniform 484454 mem 150.1 MB grow best-of-3 730.618ms
```

Gates: `cargo test -p isometer-core` 54 passed. Eponym
`cargo test -p eponym-client -p eponym-play -p eponym-sortie`: all passed (112).
Mesocosm `cargo check --workspace --all-targets` clean; `cargo test -p
mesocosm-runtime` all passed (91). isometer `cargo test --workspace
--no-fail-fast -- --test-threads=1`: 378 passed, 3 failed; the 3 fail
identically with the UB2 edits set aside (lens roster and unlit tracer tests,
mesh `attachment`).

Parallel stall, `isometer` lib test binary built from each tree and alternated,
90 s timeout:

```text
head run 1: ok 9s 80 passed; 0 failed
ub2 run 1: ok 9s 80 passed; 0 failed
head run 2: exit 124 100s
ub2 run 2: exit 124 101s
head run 3: ok 6s 80 passed; 0 failed
ub2 run 3: ok 7s 80 passed; 0 failed
head run 4: exit 124 101s
ub2 run 4: exit 124 100s
```

In a stalled run nearly every test, pure ones included (`skin`, `volumes`),
never reports; the same binary passes 80 in 18 s with `--test-threads=1`.
