# The repin onto mere `329d60d0`

**Status, 2026-10-08:** landed on main and pushed.

Carries out the wing design record's ruling 650: isocosm takes mere's
`state-witness` crate (mere's F116 to F131) by moving every mere pin to
`329d60d0`, where it landed, keeping one mere per graph (621). H2 of the
state witness plan (`mesocosm/design_docs/2026-10-06_state_witness_plan.md`)
builds on it, starting with F119's switch of `isometer_core`'s `hash_bytes`.

## Assessment (2026-10-08)

- `5fecd707..329d60d0` is 288 mere commits. Mere's own genet pin moved
  `bd3e8861` to `965b64e2`; netrender stays at `9607d16f`; the owned p2panda
  fork stays at tag `mere-p2panda-net-0.7.5`, so the root's mirror of it
  does not move.
- Isometry carries 50 mere rows at `5fecd707` and 17 genet rows at
  `bd3e8861` across its manifests, the root's mirror of mere's genet patches
  (`layout-dom-api`, `genet-scripted-dom`, `taffy`) among them.
- Mere's manifest turned `default-features = false` on several genet and
  cambium rows (`genet-render` with `livery` and `accesskit`, `taproot`,
  `genet-livery`, `cambium-rootstock`, `pictograph`); a consumer naming them
  may need the features restated. `errand` went 0.3.4 to 0.4.0 and
  `gopher-protocol` `=0.1.1` to `=0.2.0`.
- `state-witness` depends on serde, postcard (default features off, `alloc`)
  and `mere-framing` 0.0.2, which was absent at `5fecd707`.
- The work builds on the shared build-dir (643): the worktree copies the
  main checkout's ignored `.cargo/config.toml`.

## Done-conditions

1. Every workspace resolves with one revision each of mere (`329d60d0`),
   genet (`965b64e2`) and netrender. *(Met.)*
2. Every workspace checks over all targets, the VTT's with all features.
   *(Met.)*
3. The tests pass again (the VTT's, isocosm's, Mesocosm's, Eponym's,
   isometer's, isomere's, wing-integration's, isocosm-overlay's), or each
   failure is shown to predate the repin. *(Met.)*
4. The tracked locks are regenerated with no stripped sources. *(Met.)*
5. Merged to main and pushed; this plan's status says so. *(Met.)*

## Progress

- 2026-10-08: 50 mere rows moved to `329d60d0` and 17 genet rows to
  `965b64e2` across eight manifests, the probe `eponym/probes/ambience-lease`
  included. The root resolved online, fetching both sources while its
  tracked lock held every other version (58 packages locked); the other
  workspaces resolved offline from the cache. Each holds one mere, genet and
  netrender. Checks over all targets passed in all eight with no code change:
  root 117 s, Mesocosm 60 s, Eponym 77 s, isometer 39 s, isocosm 37 s,
  wing-integration 30 s, isomere 18 s, isocosm-overlay 9 s, on a fresh
  shared build-dir. Tests, one thread each: isocosm 1,237, the VTT 371,
  Mesocosm 336, isometer 293, isomere 51, wing-integration 4,
  isocosm-overlay 43; Eponym 81 with the two Sortie failures that predate
  the stable repin. The tracked locks' only source-less rows are each
  workspace's own path crates.
- Found on the way, not the repin's: isomere's own `cubecl-runtime` patch
  row is unused (the same warning ruling 639 cleared from three other
  workspaces); and the probe `ambience-lease` does not compile, at
  `src/lease.rs:253`, where wgpu 30.0.1's `get_mapped_range` now returns a
  `Result`. It fails the same way on main's old pins, so it predates this.
- Mere's coordinator noted that mere main has since moved to `4c796590`,
  bringing G2's change to cartography's projection API (its F86, F87, F132).
  No isometry crate calls those functions or depends on cartography, so the
  next repin past it is a no-op there.
