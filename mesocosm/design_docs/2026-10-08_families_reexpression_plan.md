# Re-expressing the legacy families in Isocosm

**Status, 2026-10-08:** assessment; forks ruled but 9, which goes with the bodies
family (666 to 673).
Comes after checkpoint 9 and the open bugs (wing design record, ruling 656).

Ruling 192 had Isocosm absorb `mesocosm-core`, "piece by piece re-expressed
in its process definitions (ruling 32)", Mesocosm's core shrinking to
directing, presentation and the review. Ruling 591 amended the order: the
legacy sims moved into Isocosm first, the games pointing at it and playing
as before, and "families are re-expressed as process definitions inside
Isocosm afterwards". The move landed (`1ffb7294`); this plan is the
afterwards.

## Assessment (2026-10-08)

Surveyed read-only on main `40a31d61`; counts by `wc -l`, callers by the
items each `use isocosm::legacy::…` imports (files and imported items, not
call sites). *Inference* marks what was reasoned rather than read.

**What is there.** `shared/isocosm/src/legacy/` holds 63,277 lines:

- *mesocosm*, 45,230: `world` 15,738 (the enclosure, the sixteen-variant
  `Intent`, genesis, generation, the lineage turn, graft, consume, fixtures);
  bodies in `organism` 7,377, `axis` 3,110, `phenotype` 2,837, `development`
  1,521, `graft` 666, `growth` 330; `places` 2,773 (in-site voxel space);
  matter and processes in `process` 1,033 (the five natives), `flow` 962,
  `matter` 900, `rules` 492, `pressure` 353; lineages in `discovery` 899,
  `species` 732, `program` 632, `deep_time` 413, `score` 242; the record in
  `history` 758, `record` 337, `snapshot` 304, `chronicle` 267; effects in
  `effect_pack` 834, `effect_experiment` 468, `embodiment` 374, `functions`
  195; and `voxel_profile` 394, `cohort` 201, `rng` 88.
- *eponym*, 13,737: `world` (state, combat, technique, session, timed
  actions, anatomy, sheets, movement, navigation and more), `social`
  (epistemic, society, settlement, deeds), `identity`, and `glyphs` 949.
  Eponym's world uses Mesocosm's `places`.
- *campaign*, 4,310: the campaign world and its drafts, generator, factions,
  chronicle, packs, items, construction, overmap, store, map, collaboration.
- Beside it, 13,738 lines of legacy integration tests in `tests/` and 25
  examples, and the founding datasheets the legacy `axis` embeds (626).

**Who uses it.** Mesocosm's `mesocosm-genet` (81 files), `mesocosm-runtime`
(20), `mesocosm-phenotype` (11, pack admission), `mesocosm-views` (11);
Eponym's `eponym-client` (24), `eponym-motion` (7), `eponym-sortie` (7); the
VTT's `isometry-views` (14), `isometry-genet` (13), `isonetry` (16) and
three more, on the campaign tree only; `shared/isometer` (16) and one
wing-integration test. Many Mesocosm imports are `isometer_core` body types
re-exported at legacy paths, which re-point mechanically.

**What Isocosm already has.** Native and legacy code do not reference each
other. Bodies are closest: the native recipe, palette, development, growth
and anatomy re-express Mesocosm's, and checkpoint 6 found rent, fixing and
the grazer's meal matching Mesocosm's formulas act by act; systems
(checkpoint 9), harm (10) and territories (11) are unbuilt, and neither
Mesocosm's `World`/`Organism` nor Eponym's needs and injuries are served.
Matter and processes are partial (the flow record, ported stock and
transport, the epoch rules, the pressures). The record runs in parallel
(native `history`, `journal`, the witness and `reach`). Places have no
equivalent: native space is site-grained, and 591's own evidence was that
"Isocosm has no space inside a site". Effects, the social families, items,
factions, the overmap and storylets have no native counterpart.

**The families and their order.** Mesocosm's is ruled: matter and processes,
bodies, the record, places, lineages and the boundary, then effects (195,
confirmed by 457). Eponym's (239) and the VTT's (247) are ruled in their
overlay plans, and all three predate 591. Places, bodies and the record are
shared between games. *Inference:* one merged order, shared families first:

1. Matter and processes (the ledger, conversions and the five natives;
   Mesocosm's `Registry` becoming rules-pack admission).
2. Bodies (Mesocosm's organism, phenotype, development and axis; Eponym's
   anatomy and bodies), after checkpoints 9 to 11.
3. The record (both histories, snapshots and chronicles, Eponym's
   simulation record and epistemic layer, the campaign's store and facts).
4. Places (Mesocosm's and Eponym's in-site space, the campaign's overmap
   and map), waiting on the place-graph plan's SP3 to SP5.
5. Lineages and the boundary.
6. Effects (effect packs, embodiment, functions, Eponym's glyphs).
7. Eponym's own families (239's order) and the VTT's (247's).

## Done-conditions (per family, by the overlay plan's M2)

A family is re-expressed when it runs under process definitions, conserves
its accounts, replays identically, has its tests ported or replaced by
draws, keeps every file under the 600-line ceiling, and the legacy copy no
longer owns it; where the sim plan certifies a family (as the bodies family
is), its certification passes on draws with its controls.

## Forks for Mark

1. *When a legacy copy retires.* 263: "Build, retire together" at M3; 591 and
   `legacy.rs`: each family "leaves this tree" once re-expressed.
2. *What "playing as today" holds a re-expressed family to:* replay
   identical to the legacy saves and hashes, or parity by draws with legacy
   as the control. Nothing covers legacy save formats (608 covered
   Isocosm's own v1).
3. *One merged order or three* (195, 239 and 247 predate 591).
4. *Where in-site space goes:* a sim noun, waiting on SP3 to SP5 and 343's
   columns, or isometer or the games.
5. *Code the sim's laws exclude* ("the sim never renders … resolves a
   blow"): `voxel_profile`, Eponym's sheets, techniques, timed actions,
   strike resolver and movement (597 kept motion game-side), the campaign's
   store and collaboration, `effect_experiment`, the fixture worlds (15:
   receipts become draws). Re-express, hand back to the game, or delete.
6. *Bodies before or after checkpoints 10 and 11:* Mesocosm's carcasses and
   decomposers need harm and territories.
7. *Directing:* 194 builds it only on Isocosm; whether Mesocosm keeps direct
   driving after its world family moves, or waits on M3.
8. *Parked rungs* (279): glyphs, magic and neighbouring worlds pull no code
   lane yet; whether Eponym's glyphs and the chronicles wait.
9. *The founding datasheets and the two `Founding`s* (625, 626).
10. *The legacy tests and examples:* port, replace by draws, or retire with
    their family.

## Rulings so far

- 666: a family's legacy copy leaves once its re-expression is certified
  and its callers re-point (fork 1; amends 263).
- 667: parity by draws against the legacy implementation's readings, no
  legacy save reader (fork 2).
- 668: one merged order, the shared families first in 195's order, then
  Eponym's (239) and the VTT's (247) (fork 3).
- 669: code the sim's laws exclude is handed back to its game or host, or
  deleted, when its family is reached (fork 5).
- 670: in-site space is isometer's, and the places family takes it through
  isometer, after SP3 to SP5 (fork 4).
- Fork 6 is settled by 668: bodies come after checkpoints 10 and 11.
- 671: non-sapient critters are never driven; Mesocosm's legacy driving
  ends with its world family's move, unplayable until M3 (fork 7).
- 673: glyphs and the chronicles move as code with their families; 279's
  rungs stay parked (fork 8).
- 672: legacy tests become draws or retire with their family (fork 10).
- 674 to 676: a body's geometry goes through isometer and its physiology
  stays the sim's, keyed by isometer's part ids; movement and physical
  reach go through isometer's in-site space, the reach field staying the
  sim's record; this lands with the bodies family, checkpoints 9 to 11
  certifying on the sim's geometry as built.

Open: fork 9, the founding datasheets and the two `Founding`s, taken with
the bodies family.

## Progress
