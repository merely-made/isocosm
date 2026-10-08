# One build directory for the repo's workspaces

**Status, 2026-10-07:** ruled (wing design record 638, 643 to 646);
adoption under way.

The repo builds eight Cargo workspaces (root, Mesocosm, Eponym, isocosm,
isometer, isomere, wing-integration, isocosm-overlay), each of which
compiled the whole stack into its own `target/`. Ruling 638 asked for a
shared build directory and a measured `opt-level` for isocosm's tests;
643 to 646 settled how.

## Assessment (2026-10-07)

- Disk: the repin's eight per-workspace targets held about 126 GB after
  checks and tests; the same work on one shared target in the H1 worktree
  held 81 GB. Cold checks on the shared target: root 148 s, Mesocosm 98 s,
  Eponym 192 s. No separate-target cold times were taken (645).
- `build.build-dir` is stable since cargo 1.91; this machine runs 1.98.1.
  Two throwaway workspaces under one `build-dir` reused one serde build,
  and each kept its own `target/` holding its final outputs (643).
- Nothing tracked reads `target/debug` or `target/release` by path; the
  one hit is a comment in `eponym/testing/session/acceptance.scenario`
  that passes its own `--target-dir`.
- The root `.cargo/config.toml` is gitignored (`.gitignore` line 15) as
  the machine-local override file, and every workspace inherits it (644).
- Units still split where builds differ: Eponym's whole dev profile is
  `opt-level = 1`, the root raises six render crates to 2, and the
  machine-local `tabletop-local.toml` adds Windows rustflags, so those
  builds keep their own copies of what they touch. Expected, not a fault.
- *Expected, to confirm in use:* cargo locks the build directory, so two
  builds in one checkout queue behind each other where separate targets
  ran side by side. Worktrees each have their own directory.

## The recipe (per checkout, machine-local)

In the checkout's root `.cargo/config.toml` (ignored; create it if absent):

```toml
[build]
# One build directory for every workspace in this checkout (rulings 643, 644).
build-dir = "target/build"
```

Cargo resolves the path from the directory holding `.cargo`, so it lands in
the root's ignored `target/` (*Reading, not ruled:* the location; a root
`cargo clean` clears it with the rest). A new worktree copies the file. The
iMacs and Linux boxes opt in the same way.

## Done-conditions

1. The main checkout carries the recipe, and `cargo metadata` in all eight
   workspaces reports the one `build_directory`.
2. All eight workspaces check over all targets through it, and the hosts'
   binaries still land in their own workspace's `target/`.
3. Isocosm's suite is measured at `opt-level` 0 and 1 on the shared
   directory: compile time, test time and the added disk, taken back to
   Mark before any profile changes (646).

## Progress

- 2026-10-07: the main checkout carries the recipe, and all eight
  workspaces report `target/build` as their build directory while keeping
  their own `target/` (condition 1). Cold checks over all targets through
  it, offline: root 194 s (all features), Mesocosm 93 s, Eponym 125 s,
  isocosm 60 s, isometer 52 s, isomere 19 s, wing-integration 35 s,
  isocosm-overlay 13 s, about ten minutes in all. Debug builds of the two
  hosts took 427 s and 328 s and about 41 GB; `isometry-genet.exe` landed
  in `target/debug/` and `mesocosm-genet.exe` in `mesocosm/target/debug/`
  (condition 2). Mesocosm first refused to resolve: its untracked lock in
  this checkout dated from 2026-09-29 and still pinned mere `32edc2ad`, so
  its `wgpu 30.0.0` met the `^30.0.1` mere `5fecd707` asks. Moved aside, the
  lock re-resolved offline to one mere, genet and netrender. Eponym's,
  isometer's and isomere's untracked locks re-resolved to `5fecd707` by
  themselves on the same run; only Mesocosm's held a version the new pins
  refuse, so a checkout older than the repin needs that one moved aside.
- 2026-10-08: isocosm's suite measured on the shared directory, one thread,
  offline, the `isocosm` package alone changed through
  `--config profile.dev.package.isocosm.opt-level=1` (condition 3). At
  opt-level 0: compile 210 s, run 5,029 s, the directory growing 13.4 GB.
  At 1: compile 239 s, run 1,610 s, 9.3 GB more, that being isocosm held at
  both levels side by side. All 1,237 tests passed both times. One run each
  on a machine shared with other sessions, so the seconds are approximate;
  the factor of three is well outside that. Taken to Mark.
