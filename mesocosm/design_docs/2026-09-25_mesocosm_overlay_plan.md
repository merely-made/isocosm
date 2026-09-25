# Mesocosm, the first overlay: the W5 plan

**Date:** 2026-09-25

**Status, 2026-09-25:** plan; M0 done the same day (rulings 194 to 196),
M1 to M4 proposed and not opened. Drafted at Mark's
word ("Draft the Mesocosm plan") from the
[wing design record](2026-09-18_wing_design_plan.md)'s rulings on the
overlay, 174 to 193, gathered in its §5.5, and from Mesocosm's existing
plans. Every row cites the ruling or plan it rests on; a reading of this
plan's own is flagged as one. No lane runs until Mark opens it.

**Owns:** Mesocosm's profile as a game over the Isocosm sim (the record's
§5); the Mesocosm side of the overlay contract (ruling 154); the played loop
W5 names; and the order in which `mesocosm-core`'s simulation moves into
Isocosm for this overlay's needs (ruling 192). **Does not own:** the sim,
whose schema, process definitions, record and generator the
[sim plan](2026-09-22_sim_plan.md) owns, so every move lands under its
phases; the bench (W4); the other overlays (the record's §5.6 and §5.7); or
naming. **Consumes:** the record; the sim plan and the
[aggregation research](2026-09-22_aggregation_research.md); the
[played slice plan](2026-08-28_played_slice_plan.md), under rewrite for
control (ruling 175); the
[playable ecology plan](2026-08-31_playable_ecology_plan.md), whose open
rulings are now answered (rulings 180 to 183 and two readings); the
[epoch boundary plan](2026-08-01_epoch_boundary_plan.md); and the
[vessel briefs](2026-08-18_vessel_briefs_and_presentation.md) §2.

---

## 0. Done-conditions

From the record's §11, W5 is done when Mesocosm has:

1. a profile designed to the record's §5 (§1 here);
2. a core implementing the overlay contract (§3);
3. a played loop with receipts drawn from the generator (§2, phase M4).

## 1. The profile

Ruling 6's six parts.

| Part | Mesocosm | Rests on |
| --- | --- | --- |
| Domain | the critter and its lineage, refined over the ages; what a player plays follows the lineage's traits, kin ranging from ecological competitors to extensions of one's own critter, a fungus monocreature or a swarm of germs directed whole | 35, 58, 155 |
| Ensemble | the roster of relevant critters in the region the lineage inhabits | 6, 124 |
| Mechanics | metabolize, the one verb, performed by the critter itself; incorporation where its traits allow; contests sized up and seldom reaching blows; harm as vigour and wounds; mood read and strain kept, a break going down or up | 38, 96, 115, 116, 123, 158 to 164 |
| Controls | directing, never driving: priorities, places, stances and nudges, which the critter weighs by its bond; its senses suggest | 59, 60, 175 to 178 |
| Perspective | the terrarium section at a shallow oblique, the review as the trait graph board; survival shows what the played critter knows, creative shows the truth and edits nothing | vessel briefs §2, 180, 184 |
| Timescale | epochs: a round under a versioned epoch rule, then the boundary, where every lineage adapts and the player's is revised in the shop; the start chosen from the world's habitability for the critter onward | 57, 179, 182; the playable ecology plan's `EpochRule` |

## 2. The played loop

1. **Founding.** A seeded draw from the generator's declared space, the
   region being its first edge (ruling 124). The player picks the start: at
   the world's habitability for their critter, or with as much deep time
   added as they like (ruling 179), deep time being the same sim (ruling 93).
2. **The round.** The critter acts on its own needs, senses and mood; the
   player directs with priorities, places, stances and nudges (ruling 176),
   and the critter weighs them by its bond, which moves with how well the
   orders served it (ruling 177). At a birth the player keeps the parent by
   default and may take the offspring (ruling 183); at a death the cohort is
   the pool of further lives (ruling 61), and the bond passes as the world
   setting says, seeded by the lineage by default (ruling 178).
3. **The boundary.** The epoch rule ends the round. Every lineage, played or
   not, adapts, inherits and develops, weighing its adaptation against the
   rest of the trophic web (ruling 182), the unplayed in initiative order;
   the player's turn is the review, the reckoning, the priced candidate
   table and the committed revision (ruling 57; the playable ecology plan's
   PE3b).
4. **Collapse.** A trophic collapse may be local or global and is never the
   end (ruling 181): play goes on to see what happens; after a local one the
   player can start again elsewhere, even with their lineage if it
   survives, or their critter can help the world recover as far as its
   abilities allow.
5. **Modes.** Survival and creative (rulings 180 and 184), chosen by the
   player; creative is also where debugging reads the truth.

## 3. The core and the contract

The contract's shape is ruled (ruling 154): a game submits intents, the sim
returns events and a read-only view of each tick, and outcomes a game
settles come back through the handoff.

| Direction | Mesocosm's side | Rests on |
| --- | --- | --- |
| In: directives | priorities, places, stances and nudges, stamped for a tick, for the one critter the player plays | 152, 176, 177 |
| In: checkpoint answers | at a birth, keep the parent or take the offspring; at the boundary, the review's revision | 57, 183 |
| In: dev intents | end the epoch, force a birth, kill, place matter; the dev tools plan's, never play | playable ecology plan §6 q2 |
| Out: events | receipts and record entries by subscription; in survival mode, only what the played critter can know | 113, 117, 180 |
| Out: views | a read-only view of each tick for the terrarium section and the review board | 154 |
| Handoff | empty | *reading below* |

*Reading, not ruled:* Mesocosm has no ruleset beside the sim. Metabolism,
incorporation and contests are the sim's own processes, and the shop is a
revision the sim admits at the boundary, so nothing reaches the handoff
unless a later rule adds a game-resolved outcome.

**The intents today.** `mesocosm-core`'s `Intent`
(`src/world/intent.rs:74`) holds sixteen variants. *Reading, not ruled,
confirmed intent by intent in M1:* under directing they split three ways.
`Move`, `Metabolize`, `Consume`, `Graft`, `Deposit`, `Carve` and `Idle`, a
tick advanced without acting, are driving, and become the critter's own
acts, chosen by its methodology, or lose their role. `Speciate`, `Express`,
`Revise`, `Resume` and `TakeControl` answer checkpoints and the review, and
stay. `EndEpoch`, `ForceBirth`, `Kill` and `PlaceMatter` are dev intents.

## 4. Absorption: `mesocosm-core` into Isocosm

Ruling 192: Isocosm absorbs `mesocosm-core`, piece by piece, re-expressed in
its process definitions (ruling 32), and Mesocosm's own core shrinks to
directing, presentation and the review. `mesocosm-core` is 44,901 lines of
source and almost all of it is sim. Every family that moves is decomposed
under the 600-line ceiling as it goes, split along seams the code already
has (ruling 193).

| Family | Modules, lines | Destination in Isocosm |
| --- | --- | --- |
| Matter and processes | `matter` 897, `flow` 962, `process` 1,033, `pressure` 353, `rules` 483 | the conserved ledger, fields, world conditions and process definitions; the thirty `ProcessDef` shapes expressed in the sim plan's definition (its S2) |
| Bodies | `organism` 7,360, `phenotype` 2,728, `development` 1,502, `axis` 3,091, `growth` 329, `graft` 663 | the critter: body plans, genotype and phenotype, growth, incorporation |
| Places | `places` 2,740 | the place graph; sites and locations (rulings 72, 147) |
| Lineages and the boundary | `species` 726, `program` 626, `discovery` 893, `score` 242, `deep_time` 410 | lineages, what a descendant expresses, adaptation against the web (ruling 182), deep time |
| Record | `history` 755, `record` 337, `chronicle` 267, `snapshot` 297 | the record, replay and save; significance stays the hagiograph's in mere |
| Effects | `effect_pack` 834, `effect_experiment` 468, `embodiment` 372, `functions` 194 | glyph expression and the shared functional evaluator, beside `wing-glyphs` and `wing-functions` |
| Populations | `cohort` 201 | superseded by Isocosm's population rows (the aggregation research) |
| Determinism | `rng` 88 | Isocosm's seeded stream |
| The world | `world` 15,512 | dissolves across the families above; its intent vocabulary splits as §3 says |

*Reading, not ruled:* `voxel_profile` (394) is presentation's and stays
with Mesocosm's host side; each family's destination is confirmed as it
moves, not fixed by this table.

What stays Mesocosm's own: the directing layer, the review screen, the
presentation readings (`mesocosm-views`), the runtime (`mesocosm-runtime`)
and the host (`mesocosm-genet`).

## 5. Phases and done-conditions

Proposed, not opened. Done-conditions are draws, never fixtures (ruling
15).

- **M0, the profile ruled.** Done when §6's decisions are taken; everything
  else in §1 to §4 rests on rulings already made. **Done 2026-09-25**
  (rulings 194 to 196).
- **M1, the contract's Mesocosm side.** Done when the directives,
  checkpoint answers and dev intents exist as contract types in a crate
  that depends on nothing sim-internal, every type round-trips through
  bytes (D18), and each of today's sixteen intents is mapped as §3 splits
  them.
- **M2, absorption by family.** One sub-phase per family of §4, in the
  order ruled (195): matter and processes, bodies, the record, places,
  lineages and the boundary, then effects. Each is done when the family runs in Isocosm under its
  process definitions, conserves its accounts and replays identically, its
  `mesocosm-core` tests are ported or replaced by draws, every moved file
  is within the 600-line ceiling (ruling 193), and `mesocosm-core` no
  longer owns it. Lands under the sim plan's S1 and S2.
- **M3, directing on Isocosm,** built only there, with no prototype on the
  current host (194). Done when a played critter acts on its own
  needs, senses and mood under the four kinds of directive, its bond moves
  with outcomes and passes across generations as the world setting says,
  its suggestions surface, and a seeded run replays to the same hash. When
  M3 lands, the played slice plan retires into this plan (196).
- **M4, the played loop.** Done when, from a seed nobody chose and a start
  the player picked, the headed host plays three epochs end to end on
  Isocosm: rounds under directing, a birth keeping the parent by default,
  boundaries where every lineage adapts against the web and the player
  revises in the shop, and a local collapse played on; survival and
  creative modes both work; the receipts replay to the same hash; and the
  fungible agree in distribution within the world's stated tolerance
  (ruling 113). This is W5's done-condition.

M1 and M2 run side by side; M3 needs M2's matter, bodies and places; M4
needs all of M2 and M3.

## 6. Decisions for Mark

All three taken on 2026-09-25, the day the plan was drafted.

1. **Directing before absorption, or after.** Ruled 194: only on Isocosm,
   once M2's first families have moved; no throwaway adapter.
2. **The absorption order.** Proposed as matter and processes, bodies,
   places, lineages and the boundary, the record, and effects. Ruled 195:
   the record moves ahead of places, so replay and save are on Isocosm
   sooner.
3. **The played slice plan's future.** Ruled 196: it retires into this
   plan when M3 lands.

## Findings

- **2026-09-25:** `mesocosm-core` counted at 44,901 lines and
  `shared/isocosm` at 3,208 (`wc -l` over each `src` tree's `.rs` files);
  `mesocosm-genet` is Isocosm's only consumer, and `mesocosm-core` does not
  depend on it. Per-module counts in §4 are from the same pass; `rules` was
  counted separately at 483.
- **2026-09-25:** `Intent` at `mesocosm-core/src/world/intent.rs:74` holds
  the sixteen variants §3 names; the driving seven assume direct control,
  which ruling 175 retires for play.

## Progress

- 2026-09-25: M0 done: §6's three decisions ruled (194 to 196). M1 to M4
  proposed; no lane open.
- 2026-09-25: plan drafted at Mark's word from rulings 174 to 193 and
  Mesocosm's plans. No lane open.
