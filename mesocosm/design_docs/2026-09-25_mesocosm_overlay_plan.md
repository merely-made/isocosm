# Mesocosm, the first overlay: the W5 plan

**Date:** 2026-09-25

**Status, 2026-09-25:** plan; M0 done the same day (rulings 194 to 196),
M1 done the same day (rulings 197, 202 to 205, 210 to 213), M2 to M4
proposed and not
opened. Drafted at Mark's
word ("Draft the Mesocosm plan") from the
[wing design record](2026-09-18_wing_design_plan.md)'s rulings on the
overlay, 174 to 193, gathered in its §5.5, and from Mesocosm's existing
plans. Every row cites the ruling or plan it rests on; a reading of this
plan's own is flagged as one. No lane runs until Mark opens it.

**Owns:** Mesocosm's profile as a game over the Isocosm sim (the record's
§5); the Mesocosm side of the overlay contract (ruling 154); the played loop
W5 names; and the order in which `mesocosm-core`'s simulation moves into
Isocosm for this overlay's needs (rulings 192 and 195). **Does not own:** the sim,
whose schema, process definitions, record and generator the
[sim plan](2026-09-22_sim_plan.md) owns, so every move lands under its
phases; the bench (W4); the other overlays (the record's §5.6 and §5.7); or
naming. **Consumes:** the record; the sim plan and the
[aggregation research](2026-09-22_aggregation_research.md); the
[played slice plan](2026-08-28_played_slice_plan.md), its control
rewritten for directing (rulings 175 and 199) and retiring into this plan
at M3 (196); the
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
| Controls | directing, never driving: a click draws the critter's attention to a place or a thing, attending by default and another meaning by right click, and never warns; places, priorities and stances grow from that attention as desire paths; the critter weighs each nudge by its bond, and its senses suggest | 59, 60, 175 to 178, 214 to 216 |
| Perspective | the terrarium section at a shallow oblique, the review as the trait graph board; survival shows what the played critter knows, creative shows the truth and edits nothing | vessel briefs §2, 180, 184 |
| Timescale | epochs: a round under a versioned epoch rule, then the boundary, where every lineage adapts and the player's is revised in the shop; the start chosen from the world's habitability for the critter onward | 57, 179, 182; the playable ecology plan's `EpochRule` |

## 2. The played loop

1. **Founding.** A seeded draw from the generator's declared space, the
   region being its first edge (ruling 124). The player picks the start: at
   the world's habitability for their critter, or with as much deep time
   added as they like (ruling 179), deep time being the same sim (ruling 93).
2. **The round.** The critter acts on its own needs, senses and mood; the
   player directs by clicking to draw its attention to a place or a thing,
   a right click picking another meaning (rulings 176, 214), and the
   critter weighs each nudge by its bond, which moves with how well the
   orders served it (ruling 177); its range, home, priorities and stances
   grow from that attention (ruling 216). At a birth the player keeps the parent by
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
| In: nudges | a click drawing the critter's attention to a place or a thing, attending or a right click's act, stamped for a tick, a newtype over the sim's count, for the entity the player plays, one critter or its kin directed whole; places named by a place-graph handle. The standing orders are grown, never sent | 152, 155, 176, 177, 203, 205, 214 to 216 |
| In: the player's acts | splitting the played line and naming it; the name is the doing | 202 |
| In: checkpoint answers | at a birth, keep the parent or take the offspring; at a death, the next life taken up from the cohort (*a reading of 61 and 178*); at the boundary, the review's revision | 57, 61, 178, 183 |
| In: dev intents | end the epoch, force a birth, kill, place matter; the dev tools plan's, never play | playable ecology plan §6 q2 |
| In: attention changes | pin or unpin any pointable thing; examine the region the view shows up close, or stop | 113, 130, 210, 212 |
| Out: events | receipts and record entries, each participant's stream derived from their attention set: what is attended; in survival mode, only what the played critter can know | 113, 117, 180, 204, 213 |
| Out: views | a read-only view of each tick for the terrarium section and the review board | 154 |
| Handoff | empty | *reading below* |

*Reading, not ruled:* Mesocosm has no ruleset beside the sim. Metabolism,
incorporation and contests are the sim's own processes, and the shop is a
revision the sim admits at the boundary, so nothing reaches the handoff
unless a later rule adds a game-resolved outcome.

**The intents today.** `mesocosm-core`'s `Intent`
(`src/world/intent.rs:74`) holds sixteen variants, mapped one by one in
`shared/isocosm-overlay`'s README. `Move`, `Metabolize`, `Consume`,
`Graft`, `Deposit` and `Carve` were driving and become the critter's own
acts, chosen by its methodology, and `Idle`, a tick advanced without
acting, loses its role; that split is a reading, not ruled. `Speciate`
stays the player's act and `Express` becomes the critter's own
development (ruling 202). `TakeControl` and `Resume` answer the birth
checkpoint, `TakeControl` also the death checkpoint (a reading of rulings
61 and 178), and `Revise` the review. `EndEpoch`, `ForceBirth`, `Kill` and
`PlaceMatter` are dev intents.

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
- **M1, the contract's Mesocosm side.** In one shared crate,
  `shared/isocosm-overlay`, game-neutral, with Mesocosm's vocabulary as its
  first game's module (ruling 197). Done when the directives, the player's
  acts, checkpoint answers and dev intents exist as contract types in that
  crate, which depends on nothing sim-internal, every type round-trips
  through bytes (D18), each of today's sixteen intents is mapped as §3
  splits them, and the attention set has its type, with subscription
  derived from it (ruling 204). **Done 2026-09-25:** the attention set
  typed under rulings 210 to 213. The sim's side of it, holding each set and
  deriving each stream, is built with M2 and M3.
- **M2, absorption by family.** One sub-phase per family of §4, in the
  order ruled (195): matter and processes, bodies, the record, places,
  lineages and the boundary, then effects. Each is done when the family runs in Isocosm under its
  process definitions, conserves its accounts and replays identically, its
  `mesocosm-core` tests are ported or replaced by draws, every moved file
  is within the 600-line ceiling (ruling 193), and `mesocosm-core` no
  longer owns it. Lands under the sim plan's S1 and S2.
- **M3, directing on Isocosm,** built only there, with no prototype on the
  current host (194). Done when a played critter acts on its own
  needs, senses and mood under the player's nudges, its range, home,
  priorities and stances grow from that attention, its bond moves
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
- **2026-09-25, M1's crate:** `shared/isocosm-overlay` depends only on
  `serde`, with `serde_json` for its tests. Its crate root is the wing's
  game-neutral core (the tick, opaque handles, intent and handoff envelopes
  generic over a game's vocabulary, event records and the view handle) and
  `src/mesocosm/` the first game's module; Mesocosm's handoff is an
  uninhabited type, so the empty handoff is checked rather than asserted.
  Twenty-one tests round-trip every type through bytes. Building it stopped
  at three forks (rulings 203 to 205) and found `Speciate` and `Express`
  fitting no checkpoint (ruling 202). Review found its README claiming a
  0.0.1 name reservation that crates.io does not hold; corrected on merge.

## Progress

- 2026-09-25: rulings 214 to 216 settle the input: a click draws the
  critter's attention, a right click picks another meaning, no click
  warns, and the standing orders grow from attention. The contract
  narrows to match: `MesocosmIntent::Nudge` replaces the four directive
  kinds, the priorities, places and stances becoming the sim's to grow in
  M3. Nineteen tests pass.
- 2026-09-25: M1 done. The attention set is typed in `isocosm-overlay`
  under rulings 210 to 213: who the participant plays, what they pin,
  the region they examine up close, and the game's own care, changed only
  by attention-change intents; the topic subscription is gone, each
  stream derived from its set. Two readings with it: the envelope names
  the submitting participant (rulings 152, 153), and the survival filter
  applies where a stream is derived (ruling 180). Twenty-three tests pass.
- 2026-09-25: M1's crate landed on main: Lane B's `shared/isocosm-overlay`,
  reviewed, its tests rerun, and fixed on merge for rulings 202 to 205: the
  player's acts carry `Speciate`, `Express` has no contract type, the death
  checkpoint names the next life (a reading of rulings 61 and 178), the tick
  and the place handle are ruled, and the explicit topic set is marked a
  placeholder for the attention set. M1 stays open until the attention set
  has its type (ruling 204).
- 2026-09-25: M1 opened at Mark's word as a parallel lane beside the sim
  plan's S2 probe: the contract crate `shared/isocosm-overlay` (ruling 197),
  built by a Sonnet subagent in its own worktree, to be reviewed, tested in
  the foreground and merged here by pathspec.
- 2026-09-25: M0 done: §6's three decisions ruled (194 to 196). M1 to M4
  proposed; no lane open.
- 2026-09-25: plan drafted at Mark's word from rulings 174 to 193 and
  Mesocosm's plans. No lane open.
