# The repin onto stable Burn

**Status, 2026-10-07:** landed on main (`0de3a393`) and pushed.

Carries out the wing design record's ruling 621 (amending 572): the mere pins
the stable Burn landing left at `32edc2ad` move to mere `5fecd707`, the
stable core commit mere's burn plan §13.47 publishes, so each graph holds one
mere. Briefed from §13.47 and the Knot repin's notes (ruling 585).

## Assessment (2026-10-06)

- The landing (`359c765f`, `863636f3`, `9b5cc5cc`) moved 9 of 53 mere pins
  (conatus and the cubecl-runtime patch); 44 stayed at `32edc2ad`, 562
  commits behind. The root graph held `nisus` at both revisions.
- mere `5fecd707` pins genet `bd3e8861`, where Isometry pinned `7a60ad79` in
  16 places; Cambium's types cross into genet's, so genet moves with it.
  Netrender agrees at `9607d16`.
- mere's own patch tables moved the owned p2panda fork from tag
  `mere-p2panda-net-0.7.4` to `0.7.5` and its exact requirement to `=0.7.5`;
  the root's mirror follows.
- Everything is on disk: mere `5fecd707` reached cargo's git cache from the
  local checkout (ruling 620); genet `bd3e8861` and the p2panda tag were
  cached; every crates.io crate resolves offline.
- §13.47 verified on macOS only and left Eponym unverified (its renderling
  path was missing there); this lane is the Windows and Eponym check.

## Done-conditions

1. Every workspace resolves offline with one revision each of mere, genet
   and netrender. *(Met at resolution: root, Mesocosm, Eponym, isocosm,
   isomere and isometer, no doubles.)*
2. Every workspace checks over all targets, the VTT's with all features.
   *(Met 2026-10-07.)*
3. The tests the migration ran pass again (the VTT's 370, isocosm's suite,
   Mesocosm's and Eponym's), or each failure is shown to predate the repin.
   *(Met 2026-10-07; see Progress.)*
4. The tracked locks are regenerated with no stripped sources (trap 9).
   *(Met: every mere/genet/netrender row carries its git source; the only
   source-less rows are each workspace's own path crates.)*
5. Merged to main and pushed; this plan's status says so.
   *(Met 2026-10-07.)*

## Progress

- 2026-10-07: pins swept (44 mere to `5fecd707`, 16 genet to `bd3e8861`,
  the p2panda fork to 0.7.5) and the isometer producer test given the host
  render core cambium's `ProducerContext` now requires. Checks pass over all
  targets in all eight workspaces. Tests, one thread each: isocosm 1,216,
  the VTT 370 (all features), isometer 293, isomere 51, Mesocosm 330 (one
  creator test failed once on its 30-second generation deadline under load
  and passed alone), wing-integration 4, isocosm-overlay 43; Eponym 81 with
  two failures, the Sortie scenarios that fail on untouched main too.
  Remaining: merge to main and the push, which waits on Mark's word.
- 2026-10-07: main merged in (`0979f6af`; 18 commits, whose only manifest
  change is the datasheets work's `toml = "1"` in four crates; weave resolved
  one entity). Re-verified: each workspace still resolves offline to one
  mere, genet and netrender; the root lock passes `--locked` unchanged and
  wing-integration's gains toml's registry rows. Checks pass with
  `--workspace` over all targets in all eight (Mesocosm's, Eponym's and
  isometer's roots are packages, so without `--workspace` a check covers
  only the root). Tests, one thread each: isocosm 1,231, the VTT 371,
  isometer 293, isomere 51 (with `host`; 42 without), Mesocosm 336,
  wing-integration 4, isocosm-overlay 43; Eponym 81 with the same two Sortie
  failures. The isometer producer test is rustfmt-clean again, at 600 lines.
