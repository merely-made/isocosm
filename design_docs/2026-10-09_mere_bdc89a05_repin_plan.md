# Repin onto mere `bdc89a05` (rapier 0.36)

**Date:** 2026-10-09

**Status, 2026-10-09:** in progress.

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

## Progress
