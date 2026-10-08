# The repin onto mere `329d60d0`

**Status, 2026-10-08:** planned; under way on branch `repin-329d60d0`.

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
   genet (`965b64e2`) and netrender.
2. Every workspace checks over all targets, the VTT's with all features.
3. The tests pass again (the VTT's, isocosm's, Mesocosm's, Eponym's,
   isometer's, isomere's, wing-integration's, isocosm-overlay's), or each
   failure is shown to predate the repin.
4. The tracked locks are regenerated with no stripped sources.
5. Merged to main and pushed; this plan's status says so.

## Progress
