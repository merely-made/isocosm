# Atlas replacement experiment, 2026-09-27

**Later annotation, 2026-09-27:** ruling 373 led to a
[batched one-submission measurement](batched-replacement.md), with distinct
key/voxel contents and a swapped-brick control. The first result below is
preserved as historical evidence. Allocation remains open.

Mark asked whether a replacement texture could keep unchanged bricks on the
GPU and upload only changed retained bricks. It can: the test-only harness
at `3375a36` reconstructs both replacement sizes byte-for-byte by copying
stable 3D slot coordinates and uploading one changed retained brick plus
new arrivals. Production still uses the existing capacity-fixed allocation.
This experiment does not select an allocation policy.

The enforced device is an NVIDIA GeForce RTX 4060 Laptop GPU, Vulkan driver
610.88, maximum 3D edge 2,048. The budget is 8 MiB. Source is
`shared/isometer/crates/isometer-lens/src/tracer_tests/growth.rs`, reached
only through a test module in `tracer/residency.rs`.

| Bricks, old to new | Atlas rows | Full upload median / p95 | Bulk copy + patches median / p95 | Per-slot copy + patches median / p95 |
| --- | --- | --- | --- | --- |
| 260 to 700 | 2 to 4 | 0.198 / 0.284 ms | 0.325 / 0.997 ms | 1.301 / 1.467 ms |
| 2,504 to 5,000 | 10 to 24 | 0.589 / 0.731 ms | 3.310 / 3.557 ms | 10.945 / 11.418 ms |

These are CPU wall times from creating the destination texture through a
completion wait. Each variant has two warmups and ten measured samples,
with rotating order. Readback happens outside the timer. The create call
alone took medians of 7–9 microseconds and 22–24 microseconds respectively;
that does not establish when the driver finishes physical allocation.
Other workspace agents were active; global CPU idleness was not enforced.

| Growth | Full CPU upload | Patch CPU upload | Bulk GPU copy | Selective GPU copy | Peak old + new atlas payload |
| --- | --- | --- | --- | --- | --- |
| 260 to 700 | 524,288 B | 225,792 B | 262,144 B | 132,608 B | 786,432 B |
| 2,504 to 5,000 | 3,145,728 B | 1,278,464 B | 1,310,720 B | 1,281,536 B | 4,456,448 B |

Full uploads use one write and one submission. Bulk copies use one GPU
copy; selective copies use 259 or 2,503 copies. Patches use five or fourteen
existing slot-box writes. Copy variants use two submissions so the old
texture copy precedes the queue's patch writes. This avoids overwriting a
changed retained brick with its old value. Fewer transferred CPU bytes did
not yield lower completion time in these measured implementations.

Every output is read back in full and equals the authoritative new CPU
atlas. Same-run missing-copy controls differ at 132,608 and 1,281,536
texels; missing-patch controls differ at 225,792 and 1,278,464 texels.
The old texture itself is also read back and checked first. GPU absence
fails the receipt. This verifies texture contents; it does not render a
production frame or measure pointer/bind-group recreation. Pointer payload
is 25,600 bytes in both fixtures, outside the timed operation. Peak payload
excludes staging, allocator and driver overhead.
CPU map construction and brick generation also precede the timer. These
measurements compare destination allocation and transfer strategies, not
the total cost of a production scene rebuild.

Run from the Lane E root with `CARGO_TARGET_DIR=C:\t\cargo-targets\isometry`
and `CARGO_BUILD_JOBS=4`:

```text
cargo test --manifest-path shared/isometer/Cargo.toml --release -p isometer-lens --lib atlas_replacement_copy_experiment --offline --locked -- --ignored --nocapture --test-threads=1
```

Raw stdout/stderr, compiler, source/lock hashes, exact command/scope and
machine-readable samples are under
`Code/testing/isometry/receipts/2026-09-27/lane-e-atlas-replacement/`.
The initial harness compile error ran no test and is retained separately.
`SHA256SUMS`: `84fc96a31f9b66dfaec9bc19b765cb0c441b8482a300c72b75a774cd13d1ba7d`.
Compiler: `rustc 1.98.1 (48a229cea 2026-09-01)`, host
`x86_64-pc-windows-msvc`; the standalone lock and harness are hashed there.
