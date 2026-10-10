# Repin onto mere `bdc89a05` (rapier 0.36)

**Archived 2026-10-10 (wing design record ruling 793): done.** Landed
2026-10-09. Its findings have homes: the isometry-views parallel hang was
closed by ruling 730 (`ad206d26`), and 729's spirv-std carriage retired with
renderling under 749. Nothing was left open.

**Date:** 2026-10-09

**Status, 2026-10-09:** landed.

Wing design record rulings 692 to 695 and 726 to 728
(`mesocosm/design_docs/2026-09-18_wing_design_plan.md`). mere's
`rapier-036` landed on mere main at `bdc89a05`: rapier 0.36, parry 0.31.1
with `enhanced-determinism`, seiche and conatus ported, and D2's ten
compatible updates. The previous repin is
[2026-10-08_mere_329d60d0_repin_plan.md](2026-10-08_mere_329d60d0_repin_plan.md).

## What moves

- 50 mere rows, `329d60d0` to `bdc89a05`.
- 17 genet rows, `965b64e2` to `15713014`, mere's own genet pin (68 commits,
  a fast-forward). netrender stays at `9607d16f`, as mere's does.
- parry-ground: `parry3d = "=0.29.0"` to `"0.31.1"`, a caret (693).
- One online fetch, mere's source (728); everything else is cached.

## Done-conditions

1. Every workspace holds one mere, one genet and one netrender, at the new
   revs, with checks over all targets passing.
2. Tests pass in each workspace as they did on `329d60d0`, Eponym's two
   Sortie failures excepted as before.
3. The receipts that run through conatus or parry are rerun: Mesocosm's
   tactile tests, eponym-motion's tests, isometer-lens's resident-ground
   example and parry-ground's receipt. Any figure that moves stops the
   sweep and goes to Mark with its before and after (694).
4. Merged to main and pushed; the worktree and branch removed.

## Findings

- **The root lock held `glamx` at 0.3.0** below rapier 0.36's `^0.3.1`; one
  offline `cargo update -p glamx --precise 0.3.1`, mere's own version, moved
  it. The registry rows that moved are rapier3d 0.33.0 to 0.36.0, parry3d
  0.28.0 to 0.31.1, glamx and nisus; wgpu stays 30.0.1.
- **Eponym hit a real `libm` conflict**, renderling's rust-gpu `05b34493`
  capping `spirv-std`'s `libm` at 0.2.11 against simba 0.10.2's `^0.2.15`.
  Ruled 729: a one-line carriage, branch `mark-ik/spirv-std-libm-uncap`
  (`24451e7f`) at `Code/crates/rust-gpu-libm`, path-patched for both
  sources, until L7 drops renderling on the kiss3d tenant.
- **mere's cambium now renders app text fields as `role="textbox"`
  elements marked `data-cambium-text-value`** (mere `019e07a0`), not
  `<input>`. The VTT's three typing tests failed: `focused_lane` looked for
  an `<input>`, and the theme's four field rules (`.cmd-line`,
  `.character-name`/`.character-owner`, `.compose-line`, `.search-field`)
  targeted `input`, so the owner field lost its width and a click on it
  dropped focus. Both now follow cambium's contract.
- **isometry-views' library tests hang when run in parallel**, stalled in
  the GPU-backed `scene` tests (`paging_tests`, `parity_tests`), each
  creating its own wgpu device. Two of three parallel runs hung on the new
  pins and one of three on main's old pins; serially all 119 pass in 17 s.
  It predates this repin: ruling 656's isometry-views fault, reproduced at
  last.
- **parry-ground compiled unchanged on parry 0.31.1** and reproduced every
  figure of its 2026-08-21 receipt.
- **isomere counts 51 tests with all features and 42 without**; the last
  repin's 51 was the all-features count.

## Progress

- 2026-10-09: 50 mere rows to `bdc89a05`, 17 genet rows to `15713014`,
  parry-ground to parry3d `0.31.1` (caret). One online fetch, mere's
  source; all else offline. Checks over all targets passed in all nine
  workspaces: root 133 s, Mesocosm 79 s, Eponym 44 s (with 729's
  carriage), isometer 48 s, isocosm 39 s, wing-integration 37 s,
  parry-ground 23 s, isomere 16 s, isocosm-overlay 7 s. Tests: isocosm
  1,265, the VTT 371 with all features (isometry-views' 119 run serially),
  Mesocosm 336 (tactile among them), isometer 294, isomere 51 with all
  features, wing-integration 4, isocosm-overlay 43, Eponym 81
  (eponym-motion among them) with the two Sortie failures that predate the
  stable repin. Receipts: parry-ground's and isometer-lens's resident-ground
  (patch 4 bytes into the 262,144-byte atlas at read epoch 11, 184 changed
  pixels, the 112-byte delta replaying) both unchanged; no figure moved
  (694).
