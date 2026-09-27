# Paging integration receipt, 2026-09-27

Rulings 368, 369, 372 and 374 are implemented on Lane E: current omissions
are reported, the player can change the device-bounded atlas budget, the
requested budget persists locally outside campaigns, and the chosen feasible
atlas capacity is allocated upfront. Production growth was not added.
Main `4ecd2ce` (including sim checkpoint 5) merged without conflicts.
Mere `7bb5bfda`, Genet `0cf4f30b`, and Netrender `c8c09f16` remain unchanged.

## Source and gates

Final source is `ace50d61219e9ae2ecc1b9c42a2beabe47e39843`.
The successful headed binary was built at
`dd334861fa032e05caf8c833c1ec88dd0d8497de`; the final change only formats
`hooks.rs` (line wrapping and an optional tuple comma). The preserved diff
and normalization check qualify that small difference explicitly.

| Gate | Source qualification | Result |
| --- | --- | --- |
| Root workspace tests | `232a5dc`; later changes only the gated receipt hook | 411 passed, 7 intentional ignores; no GPU skip diagnostics |
| Root all-features/all-targets check | `232a5dc` | passed, offline/locked |
| Shared isometer workspace tests | shared source unchanged through final head | 293 passed, 3 intentional ignores; no GPU skip diagnostics |
| Mesocosm all-features/all-targets check | combined sim/paging source after main merge | passed, offline/locked |
| Eponym all-features/all-targets check | helper at `e714f6e`; consumer/shared source unchanged through final head | passed, offline/locked, exact primary lock |
| Explicit settings, band and release cost gates | shared/view source unchanged through final head | passed; 150 cost records recomputed |
| Release host build and six headed sessions | `dd33486` | passed |
| Changed Rust source rustfmt and diff checks | final source | passed; every changed Rust file at most 596 lines |

Compiler: Rust 1.98.1, full `rustc -Vv` archived. All builds reused
`C:/t/cargo-targets/isometry`, four jobs. Tests used one test thread.
Exact commands, separate stdout/stderr, exit codes, source and lock hashes
are in the raw directory below. Earlier negative controls remain qualified
by `settings.md` and the current-overflow receipt; they were not rerun merely
because this batch adds a host driver.

Eponym's tracked local Renderling/Crabslab paths do not resolve from the old
hidden Lane E worktree. That failed check is preserved. The successful check
used the existing approved `Code/worktrees/isocosm-lane-e-eponym-20260927`
helper at the combined source. Its old ignored lock and replacement primary
lock are both archived. No manifests, pins or tracked locks changed.
The prior V1b runtime receipt remains exact for Eponym source `915f9fa`,
which is byte-identical to this branch's Eponym tree. Its runtime used wgpu
30.0.1; this batch's primary-lock compile is separately qualified. V1b's six
publications include the initial publication and five later retargets.

## What the final gates demonstrate

- The settings GPU gate fills the actual chosen atlas even with only 169
  resident bricks: 8, 4, 12, 32 (requested 64, device-clamped), then 2 MiB.
  Each setting change rebuilds once, then the unchanged producer parks.
  At 2 MiB a pan and an elevation edit create zero resources, retain capacity
  4095, and upload zero and 2064 bytes respectively.
- The old-walker positive control moves 330 pixels; fixed CPU and GPU maps
  move zero. Both GPU frames differ from the empty-terrain frame at 669,280
  pixels. All three captures have 2,677,120 bytes. The separately reported
  68-pixel difference from the f64-camera diagnostic remains a limitation;
  this is headroom invariance, not universal pixel exactness.
- The headed relief fixture shows 247 omitted bricks at 1 MiB. A real pointer
  click on the disabled minus leaves the setting unchanged. A real click on
  plus changes it to 2 MiB, causes one requested rebuild (2294/4095 resident),
  and the displayed omission count becomes zero. A second host process loads
  the same saved file and displays 2 MiB and zero omissions.
- Child-only `LOCALAPPDATA` isolates the real preference path at
  `Merely/Isometry/terrain.json`. The seed is written only before the first
  process; the restart uses its saved file. User configuration is untouched.
- Fresh default demo and 256-board captures show the 8 MiB control and live
  terrain. Headroom 1 and 0 captures are pixel-identical; the same-run
  overlay-absent control differs at 32,424 pixels. Captures are 2200x1504.
  The default demo/256 steady release windows contain 475/471 frames with
  total-frame medians 5.866/5.821 ms and producer medians 0.003/0.006 ms.
  These are local observations, not latency guarantees or the old debug
  profile. `recomputed-headed-profile.json` preserves the full statistics.

[Final cost tables](final-cost.md) are regenerated from 150 release records:
zero overflow at the default budget; select work medians 0.47–0.69 ms versus
same-run full-regrow baselines 320.90/806.16 ms, distinguished by board/pane.
Old timing summaries remain historical rather than being overwritten.
The cost table's `headed` rows use headed-pane dimensions in a headless
harness; its `frame` timer covers `Board::draw` plus completion wait.
Neither is full host input-to-present latency. The six native sessions
above are separate evidence.

The first new headed attempt correctly failed its overflow positive control:
the normal boot camera was near the corner, holding 1519/2047 bricks with no
omissions. It is preserved under `terrain-change/`. Only the gated receipt
fixture was then centered; the passing six sessions are under `headed-final/`.
The initial format-check failure and corrected pass also remain archived.

## Raw evidence

`Code/testing/isometry/receipts/2026-09-27/lane-e-final-paging/`
contains 112 files named by `SHA256SUMS`. Its SHA-256 is
`d604a92f697aa9b64f1bcb0e1fdbc474c08379e23ede8c2369de5d0a0659c65f`.
`verification.json`, source qualification notes and the command drivers are
included. `final_headed.py` here reproduces the headed set into a fresh output
folder; it refuses existing session folders and leaves earlier evidence alone.

The stable shared target remains reusable. The existing Lane E and Eponym
helper worktrees remain owned by the coordinator until review and integration;
no new target, Cargo home or worktree was created. Main integration belongs
to the coordinator.
