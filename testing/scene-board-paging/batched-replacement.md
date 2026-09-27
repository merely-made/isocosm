# One-submission atlas replacement, 2026-09-27

Ruling 373 asks for a more tightly batched measurement before deciding
allocation policy. This test-only extension leaves production allocation
unchanged and preserves the [first experiment](replacement.md).

The new path packs changed/new slot boxes into one freshly allocated,
zeroed CPU vector, initializes one fresh mapped staging buffer, and encodes
the old-extent texture copy followed by buffer-to-texture patch copies in
one command encoder and one submission. It does not use queue texture
writes in that path, so a deferred write cannot precede the old copy and
then be overwritten. The timer includes packing and these allocations,
command encoding, submission and completion wait. CPU map generation,
pointer/bind-group recreation, rendering and verification readback remain
outside all three arms.

Device: NVIDIA GeForce RTX 4060 Laptop GPU, Vulkan 610.88, enforced 3D edge
2,048, atlas budget 8 MiB. CPU adapters are rejected. Each arm has three
warmups and thirty measured iterations with rotating order. The tails
below are observed sample statistics, not stable latency guarantees.

| Bricks | Path | Median | Observed p95 | Observed maximum |
| --- | --- | --- | --- | --- |
| 260 to 700 | Full upload | 0.191 ms | 0.276 ms | 0.382 ms |
| 260 to 700 | Previous bulk copy, two submissions | 0.303 ms | 0.400 ms | 0.405 ms |
| 260 to 700 | Packed patches, one submission | 0.348 ms | 0.500 ms | 1.374 ms |
| 2,504 to 5,000 | Full upload | 0.598 ms | 0.653 ms | 0.659 ms |
| 2,504 to 5,000 | Previous bulk copy, two submissions | 3.325 ms | 3.456 ms | 3.622 ms |
| 2,504 to 5,000 | Packed patches, one submission | 1.584 ms | 1.763 ms | 1.890 ms |

The batched path improves the larger copy case but remains slower than
the full-upload baseline in both cases. Its median CPU packing cost is
25.45 / 513.00 microseconds; initializing the mapped buffer takes
106.80 / 508.70 microseconds. These component medians need not sum to the
median total. Other workspace CPU work was not globally stopped; the team
ran no parallel GPU gate.

| Quantity | 260 to 700 | 2,504 to 5,000 |
| --- | --- | --- |
| Full upload logical texels, including empty capacity | 524,288 B | 3,145,728 B |
| Of that, empty capacity | 165,888 B | 585,728 B |
| Changed/new logical texels | 225,792 B | 1,278,464 B |
| New path's exact padded staging payload | 475,136 B | 2,588,672 B |
| Old-extent GPU texture copy | 262,144 B | 1,310,720 B |
| New path's texture / buffer copy calls | 1 / 5 | 1 / 14 |
| Common peak old + new atlas payload | 786,432 B | 4,456,448 B |
| Atlas + explicit staging-buffer payload | 1,261,568 B | 7,045,120 B |
| Separate CPU packing vector | 475,136 B | 2,588,672 B |

Full uploads use one queue texture write/submission; the previous bulk path
uses one texture copy plus five/fourteen queue writes across two submissions.
The new staging bytes include 256-byte row padding. The older queue-write
helpers expose logical texel bytes, while backend staging padding remains
opaque. This is not a bus-traffic measurement or a pure measure of copying
unchanged GPU content. Resource payload is logical size, excluding driver
and allocator overhead, opaque queue staging and the separately reported
CPU vector. The 25,600-byte pointer fixture is identical across arms and
outside the operation.

Every key has an asserted-distinct 512-byte brick, filled with deterministic
nonzero per-key/per-voxel patterns. All 198 full-atlas readbacks, including
warmups, exactly match the new CPU atlas. The old texture is independently
read back first. Same-run broken controls detect:

| Control | Small case differing texels | Larger case differing texels |
| --- | --- | --- |
| Missing retained copy | 132,608 | 1,281,536 |
| Missing patches | 225,792 | 1,278,464 |
| Two swapped retained bricks | 1,022 | 1,022 |

Tested source: `e69e1df`,
`shared/isometer/crates/isometer-lens/src/tracer_tests/growth_batched.rs`.
Run from Lane E with `CARGO_TARGET_DIR=C:\t\cargo-targets\isometry` and
`CARGO_BUILD_JOBS=4`:

```text
cargo test --manifest-path shared/isometer/Cargo.toml --release -p isometer-lens --lib atlas_replacement_batched_experiment --offline --locked -- --ignored --nocapture --test-threads=1
```

Final stdout/stderr, parsed samples, source/lock hashes and exact scope:
`Code/testing/isometry/receipts/2026-09-27/lane-e-atlas-batched/experiment.*`.
`draft-pattern.*` preserves an earlier run with a periodic key pattern and
growing packing vector; it is historical, not this result. The original
experiment directory remains unchanged. Compiler:
`rustc 1.98.1 (48a229cea 2026-09-01)`, `x86_64-pc-windows-msvc`.
Manifest `SHA256SUMS`:
`0e92fcd47d4a026297317fabd6e5aeabed6413f2b12ff4a259a9cd468e86c741`.
