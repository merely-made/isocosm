# SP3: generated terrain in the retained specimen bench

Verified 2026-09-29, source baseline `e32890e` plus the source hashes beside
this receipt. Rust 1.98.1, Windows native debug build; unchanged published
Mere `5ce144ff` / Genet `7b48f94d` dependency graph. Stable reusable target:
`C:\t\cargo-targets\isometry`. No worktree or isolated Cargo home.

## Checks

- `cargo test -p mesocosm-genet --lib spine --locked -j 4`: five passed,
  158 filtered, 4.77 seconds executing. Seeds 0 through 7 test both views;
  seeds 0 through 3 test the perturbed source. Material-ID reorder, exact
  material transition samples, repeatability and world immutability pass.
- `cargo build -p mesocosm-genet --bin mesocosm-genet --locked -j 4`: passed.
- `cargo check -p mesocosm-genet --all-targets --locked -j 4`: passed.
- `spine.scenario`: native pass, 37 frames, six captures. Root visually
  inspected border, overview and perturbed captures. The ordinary border
  has no artificial split; changing one source produces a visible cliff.
- `spine-boundary.scenario`: native pass, 55 frames, two captures. Holds
  tick 100, trial hash and model revision through 20 settled frames while
  terrain is open; returning completes epoch 1 at tick 1000. This exercises
  the existing specimen trial, not an Isocosm foreground simulation join.

Commands run from `mesocosm/`, after setting the stable target:

```powershell
& C:\t\cargo-targets\isometry\debug\mesocosm-genet.exe --bench --seed 7 --size 1280x900 --scenario testing/bench/spine.scenario --receipt testing/bench/receipts/2026-09-29/spine/native.json --capture testing/bench/receipts/2026-09-29/spine/native.png --frames 2400
& C:\t\cargo-targets\isometry\debug\mesocosm-genet.exe --bench --seed 7 --size 1280x900 --scenario testing/bench/spine-boundary.scenario --receipt testing/bench/receipts/2026-09-29/spine/boundary.json --capture testing/bench/receipts/2026-09-29/spine/boundary.png --frames 2400
```

## Source and projection evidence

Seed 7, plane grid 3 by 4, site side 2000 base units, sites 0 and 1.
Elevation range 0..2000, relief 0..250, sea fraction 408 per mille.
Default dimetric camera; opaque water palette. Receipts contain the complete
grid parameters, world-local material keys, common local datum and bounds.

| View | Columns / cell size | Occupied bricks | Filled voxels | Material samples |
| --- | --- | --- | --- | --- |
| Shared border | 32 by 32 / 1 | 51 | 20,797 | 6,144 |
| Whole two-site overview | 126 by 63 / 32 | 237 | 73,823 | 47,628 |
| Perturbed border | 32 by 32 / 1 | 74 | 32,713 | 6,144 |

Both unmodified exact border digests are `b4803bca3ffa773e`. The control
raises only the far source's site elevation by 64 base units; its digest
becomes `5fc08f782c5d9e42`. It changes 158,634 of 918,544 sampled viewport
pixels. Returning from overview and removing the perturbation each restore
the original viewport exactly, zero changed pixels. Source and lowered
material digests agree. Each draw's world-before/world-after digests agree.

`spine-render` in the native JSON records the actual scene diagnostics;
source voxel counts are not render upload counts or frame-time estimates.
The existing scenario camera label also retains the specimen's selected
preset; SP3's actual presented camera is supplied by its renderer and uses
`SlabCamera::dimetric_2_1`.

## Failure found and corrected

`initial-capacity-failure/` preserves the first native failure and the
subsequent failing capacity tests. A full-site-sized perturbation made
3,664 bricks, beyond the current `GroundTerrain` default capacity of 2,047.
The renderer refused it, and the viewport-change assertion failed instead
of accepting the old image as proof. A 64 by 64 normal border also exceeded
capacity for one seeded test.

The final close window is 32 by 32 and the control is 64 base units. Before
allocating Ground, the adapter counts exact occupied bricks against
`AtlasLimits::DEFAULT`; it also bounds filled voxels and estimated residency.
Over-budget draws refuse without replacing the accepted landscape. This
is bounded terrain presentation, not paging acceptance, a laptop performance
benchmark, entity-body absorption, simulation edits or one-host mode switching.

Checksums in `SOURCE_SHA256SUMS` identify the tested source, scenarios,
manifests and locks. `ARTIFACT_SHA256SUMS` identifies captures and logs.
