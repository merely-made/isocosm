# Isometer's failing tests

**Status:** 2026-11-06: in progress. D1 to D4 answered; IT1 and IT2 done; IT3's roster half fixed, its unlit half awaiting a ruling.

**For:** isometer's three failing tests and its parallel library-test stall,
found while gating `2026-11-05_uniform_bricks_plan.md`. The failing three were
owed as the lane queue's Q4 ("Fix or retire the failing tests … one catching
a real regression is fixed",
`mesocosm/design_docs/archive_docs/2026-10-18/2026-10-10_lane_queue_plan.md:148`),
which no live plan carried after that plan was archived on 2026-10-18.

## Decisions

Answered by Mark on 2026-11-06, in answer order.

- **D1, the mass test (2026-11-06):** "Fix the test (Recommended)". The
  meal lands at x 12, a shift of about 1.3 voxels; `centre_of_mass` keeps
  its integer rounding, which the sim reads. Offered against rounding to
  nearest in core and a sub-voxel centre.

  Links: area Stack and engine · draws on stack.5 · summary The eating test lands its meal farther out; centre of mass keeps its integer rounding.

- **D2, the tracer tests (2026-11-06):** "Diagnose at HEAD (Recommended)".
  Read each test against today's tracer and see what it renders; no bisect
  over old revisions. Offered against bisecting with rebuilt locks and
  retiring the tests.

  Links: area Stack and engine · draws on stack.3 · summary Diagnose the roster and unlit tracer tests against today's tracer, not by bisect.

- **D3, the stall (2026-11-06):** "1, but i am also ok with a debugger."
  One shared adapter and device per test binary; a debugger to find the
  exact hang is acceptable if that does not end it. Offered against
  serializing GPU tests, capping test threads and digging first.

  Links: area Stack and engine · draws on stack.3 · summary GPU tests share one device per test binary; a debugger is allowed if the stall persists.

- **D4, the home (2026-11-06):** "New root plan (Recommended)". This plan.
  Offered against the uniform-bricks plan and commits alone.

  Links: area Method and documents · summary Isometer's failing tests and stall get their own plan.

## Phases

### IT1. The eating test lands its meal farther out (done 2026-11-06)

- Done when: `isometer-mesh`'s `attachment` tests pass, the meal at x 12,
  with its other assertions (collision box, geometry) unchanged.

### IT2. One GPU device per test binary (done 2026-11-06)

A test-only helper in `isometer` hands every GPU test the same adapter,
device and queue, replacing the seven per-test `request_adapter` and
`request_device` sites (`glyphs/tests.rs`, `producer/tests.rs`,
`query/perspective_tests.rs`, `query/tests.rs`, `scene/palette_tests.rs`,
`tokens.rs`).

- Done when: `isometer`'s library tests pass in 10 consecutive parallel runs
  at the default thread count, each under 60 s, and on one thread; GPU tests
  still skip when no adapter exists. If the stall survives, a debugger finds
  the hang (D3), named to Mark before it is downloaded.

### IT3. The roster and unlit tracer tests, diagnosed

- Done when: for each, what the frame shows against what the test expects,
  with the mechanism in today's tracer (file:line), and either a fix that
  makes it pass or a fork to Mark on whether its expectation is stale.

## Findings

- 2026-11-06: the eating test's centre of mass moves 243/249 voxel on x and
  truncates to 0 (`isometer-core/src/body.rs:377`). Its meal weighed 400
  until `8848f4e0` (2026-10-09) made mass the product's ledger (box volume,
  27). Callers that keep the rounding: `shared/cosmograph/src/lineage/body.rs:243`,
  `shared/cosmograph/src/spatial/embodiment/support.rs:19`.
- 2026-11-06: the stall is GPU-tests-only and concurrency-bound. GPU tests
  alone at the default 16 threads stalled 2 of 4 runs; at 8, 4 and 2 threads
  12 of 12 passed; the pure tests never stalled (4 of 4). In a stall the GPU
  is idle (0%, 2.8 of 8 GB on an RTX 4060 Laptop) while the CPU spins; the
  first wave of 16 tests, all GPU tests, never reports. Each test makes its
  own `wgpu::Instance::default()` (DX12 and Vulkan enabled, wgpu 30.0.1).
- 2026-11-06: a bisect of the tracer tests did not run: `shared/isometer`'s
  lock is tracked only from `af9e0e96` (2026-11-03), and resolving an older
  revision sat over 25 minutes. Both tests failed already on 2026-10-10
  (`mesocosm/design_docs/archive_docs/2026-10-18/2026-10-10_after_pass_plan.md:123`)
  and at `adb2bb63`'s parent by its own message.

- 2026-11-06: one shared device ends the stall. `isometer/src/test_gpu.rs`
  holds one adapter's device and queue in a `OnceLock`; the seven sites take
  clones. Ten consecutive parallel runs of the library tests passed (80 each,
  4 to 13 s), and one thread took 11 s against 18 s before. No debugger was
  needed (D3).

- 2026-11-06: neither tracer test shows a tracer regression; both went
  stale when `cce81ae3` (2026-10-09) swapped their ground from
  `Places::grown(4_242, 4, 64)` to `isometer_core::fixtures::ground(4_242, 64)`
  (`isometer-lens/src/tracer_tests.rs:20-22`), the gentle `Rolling` relief
  (`isometer-core/src/fixtures.rs:36-48`). Roster: both bodies stood at
  `surface(4, 4) + 3..5` (y 7 to 9), and the relief in front of x 8.5 reaches
  y 10, so the second body drew no pixels (108 against 108) while the
  diagnostics counted it (`roster_members` 2, `roster_dropped` 0). Each body
  now stands clear of the highest ground between it and the camera; the test
  passes (160 pixels for one body, 280 for two, per the diagnosis lane).
  Unlit: soil passes (739 unfogged ambient-lit faces, all on the first rung)
  but rock finds none, because `soil_over_rock` (`isometer-core/src/ground.rs:45-47`)
  keeps rock two voxels down and this relief never steps three voxels, so
  every rock pixel is the section's cut wall (3,180 of 3,180 and 3,500 of
  3,500), whose +y normal is lit. The old ground's cliffs exposed rock. The
  test's own notes (`unlit.rs:380-399`) assume those faces exist.

## Progress

- 2026-11-06: plan written; D1 to D4 answered.
- 2026-11-06: IT1 done (the meal at x 12; `attachment` 4 of 4). IT2 done
  (`test_gpu`; 10 of 10 parallel runs). IT3: roster fixed test-locally;
  unlit's rock half is a question.
