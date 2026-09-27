# Current omissions and local terrain memory, 2026-09-27

Rulings 368/369/372 select the existing centre-first overflow selection with
a current omitted count, a player-controlled default 8 MiB atlas budget,
and local per-device persistence. They do not select allocation growth.
The host continues allocating the chosen capacity up front.

`Residency::overflow()` reports the latest successful framing independently
of `ResidencyStats`, whose last-change meaning stays intact. Before the
fix, a same-keys/no-dirty regression reported 7 when the frame said 12.
After `8ceb59d`, it reports 12, 3, then 0 without changing the atlas bytes,
projection revision, residency change count or last-change stats; refresh
returns `Current`. The host-facing `GroundCost` substitutes that current
count and its profile line is tested too. Raw fail/pass receipts are in
`Code/testing/isometry/receipts/2026-09-27/lane-e-current-overflow/`, whose
`SHA256SUMS` is
`6f1ec1406904c9d7e0973dbd5c7c2bd60e5600eb815b580e3bf613aa23bfdadb`.
Two earlier command/standalone-lock launch errors there are not regression
evidence; `before-fix-resolved.*` is the behavioral failure.

The scene board's Map area now contains a terrain-memory stepper and current
omitted count. Memory is measured in MiB, defaults to 8 and is bounded by
the scene's enforced device limits. This is atlas storage only; pointer
storage remains separate. Current count and limits reach the panel after
each presented frame, outside the profiling flag, with one redraw on a
changed report. Settings enter `SceneSignature.host`; the producer's generic
skip rule remains unchanged.

The real-GPU parked-scene regression draws once, changes 8 to 4 to 12 to
64 (clamped to this device's 32) to 2 MiB with unchanged camera and
`needs_frame=false`, and checks exactly one
rebuild followed by a skip for each change. A headroom change also rebuilds
once then skips. Deliberately removing the settings signature fails at
"changed budget must draw"; restoring it first passed with five total
renders. The final gate adds the above-cap request and verifies six total
renders, plus inert controls before device limits arrive and at the cap.
This is an automated GPU receipt, not a new headed UI capture.

The small host-owned JSON store contains only version 1 and requested
`atlas_budget_mib`. Windows uses
`%LOCALAPPDATA%\Merely\Isometry\terrain.json`; macOS uses
`~/Library/Application Support/Isometry/terrain.json`; other platforms use
`$XDG_CONFIG_HOME/isometry/terrain.json` or `~/.config/isometry/terrain.json`.
It loads before the scene board is constructed and saves a changed request
at dispatch, with a frame check for programmatic changes. It never stores
the observed device cap or omitted count. A requested value above a later
device's cap remains requested and is clamped only when applied.

Missing files leave the default. Malformed, oversized, unsupported or zero
values are reported and leave the default without a boot overwrite. Saves
write a process-owned temporary file, sync it, then replace the destination.
Failed saves leave the session setting usable and report the failure; the
same failed value is not retried every frame. Tests inject temporary paths
and do not read or write the developer's application preferences.
Both persistence tests pass: missing/corrupt/unsupported/zero input,
restart roundtrip, replacing an existing file, requested 64 MiB surviving
a reported 32 MiB cap, and failed-save session survival. An independent
read-only review found no blocking correctness issue.

Commands, compiler, source/lock hashes and stdout/stderr for the settings
and persistence gates are under
`Code/testing/isometry/receipts/2026-09-27/lane-e-live-terrain-settings/`.
All gates use `C:\t\cargo-targets\isometry`, four build jobs, release mode,
offline locked resolution and one test thread. The compiler is
`rustc 1.98.1 (48a229cea 2026-09-01)`, `x86_64-pc-windows-msvc`.
Tested implementation: `46e0402`. Receipt `SHA256SUMS`:
`55f5a55ee00412b7a8cee3f058a378c60f26234f3fb80740179bb10a038a9a75`.
The final GPU command spent most of its elapsed build time waiting on an
unrelated global Cargo cache owner; the test itself completed in 3.09 s.

The [replacement experiment](replacement.md) answers the separate transfer
cost inquiry. Broader paging timing and headed integration receipts remain
separate gates before main integration.

## 2026-09-27 host integration follow-up

The real headed controls now have a dispatch-and-restart receipt:
1 MiB displays 247 omitted bricks; a disabled minimum click stays unchanged;
the increment rebuilds at 2 MiB and clears omissions; a second process restores
2 MiB from the same isolated local preference file. The upfront GPU gate also
asserts full chosen capacity for a small map and no allocation on ordinary
pan/edit. The portable XDG resolver now ignores empty/relative XDG candidates
before falling back to HOME. See [integration.md](integration.md) for final
source, gates, pictures, historical failed fixture and raw hashes.
