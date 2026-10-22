# Mesocosm, the first overlay: the W5 plan

**Date:** 2026-09-25

**Status, 2026-10-18:** in progress. M0 and M1 are done (2026-09-25). M2 is
the families re-expression plan's Mesocosm share; M3 and the interim M4 are
the directing plan's (cases 677 to 681, 800). Generated worlds now declare
spawn and arrival templates (case 889) and a global-collapse predicate (case
891), merged and pushed 2026-10-15, uncertified; the producer capture path
(case 890) is built. Open: full M4, where chosen in-site acts and contact
reach the same accepted world, and a declared process for generated bodies'
step, terrain and birth inputs, which no template can satisfy. Next:
certification, and the headed findings carried below.

## Current state, 2026-10-10

`mesocosm-core`, its sixteen-variant driving `Intent` and the legacy
Mesocosm world are deleted. Native Isocosm owns simulation, development,
lineages and history. The absorption tables below are dated migration
inventories; the families plan's Progress and the sim plan's current-state
section locate the new owners.

Mesocosm's four crates are runtime, views, phenotype and genet.
`mesocosm/crates/mesocosm-runtime/src/runtime.rs` runs a native `Session`;
`runtime/translate.rs` lowers `isocosm_overlay::mesocosm` envelopes.
The player directs a non-sapient critter; a sophont is driven (671, 779).
The review's authored expression now uses native `Command::Express` (787).
M3 and site-grain interim M4 are the
[directing plan](2026-10-08_directing_interim_m4_plan.md)'s; migration is
not their draw or headed certification.

Full M4 still must connect chosen in-site acts and contact to the same
accepted world. `mesocosm-runtime::perception` now refines native awareness
through 741's ray with explicit actual cells and range; `Patch` remains
native, and scene lowering withdraws departed or refused inputs. The shared
host registers Mesocosm's actual Section on its rendering device. These
source changes leave native targeting at site grain and chosen-process
`Edit` binding open; presentation spots never supply sight geometry.

*The dated profile, absorption survey and receipts below remain as written (793).*

**Owns:** Mesocosm's profile as a game over the Isocosm sim (the record's
§5); the Mesocosm side of the overlay contract (ruling 154); the played loop
W5 names; and the order in which `mesocosm-core`'s simulation moves into
Isocosm for this overlay's needs (rulings 192 and 195). **Does not own:** the sim,
whose schema, process definitions, record and generator the
[sim plan](2026-09-22_sim_plan.md) owns, so every move lands under its
phases; the bench (W4); the other overlays (the record's §5.6 and §5.7); or
naming. **Consumes:** the record; the sim plan and the
[aggregation research](2026-09-22_aggregation_research.md); the
[played slice plan](archive_docs/2026-10-10/2026-08-28_played_slice_plan.md), its control
rewritten for directing (rulings 175 and 199) and retiring into this plan
at M3 (196); the
[playable ecology plan](archive_docs/2026-10-10/2026-08-31_playable_ecology_plan.md), whose open
rulings are now answered (rulings 180 to 183 and two readings); the
[epoch boundary plan](2026-08-01_epoch_boundary_plan.md); and the
[vessel briefs](2026-08-18_vessel_briefs_and_presentation.md) §2.

---

## 0. Done-conditions

W5's done-conditions are the record's
[the wing's phases](../../design_docs/2026-07-30_games_wing_founding.md#9-the-wings-phases) (ruling 617). Here, for Mesocosm, they are
met by §1 (the profile), §3 (the core and the contract) and §2's phase M4
(the played loop).

## 1. The profile

Ruling 6's six parts.

| Part | Mesocosm | Rests on |
| --- | --- | --- |
| Domain | the critter and its lineage, refined over the ages; what a player plays follows the lineage's traits, kin ranging from ecological competitors to extensions of one's own critter, a fungus monocreature or a swarm of germs directed whole | 35, 58, 155 |
| Ensemble | the roster of relevant critters in the region the lineage inhabits | 6, 124 |
| Mechanics | metabolize, the one verb, performed by the critter itself; incorporation where its traits allow; contests sized up and seldom reaching blows; harm as vigour and wounds; mood read and strain kept, a break going down or up | 38, 96, 115, 116, 123, 158 to 164 |
| Controls | directing, never driving: a click draws the critter's attention to a place or a thing, attending by default and another meaning by right click, shown as a ring of acts, and never warns; places, priorities and stances grow from that attention as desire paths; the critter weighs each nudge by its bond, and its senses suggest | 59, 60, 175 to 178, 214 to 216, 228 |
| Perspective | the terrarium section at a shallow oblique, the review as the trait graph board; survival shows what the played critter knows, creative shows the truth and edits nothing. *2026-09-28:* the wing has one default view, isometric with quarter turns, precluding no other (382); whether Mesocosm opens in it or in the section is open, ruled on CP1's review comparing its 2026-09-05 arm with the default at the 2:1 dimetric pitch (387, 388) | vessel briefs §2, 180, 184, 382, 387, 388 |
| Timescale | epochs: a round under a versioned epoch rule, then the boundary, where every lineage adapts and the player's is revised in the shop; the start chosen from the world's habitability for the critter onward | 57, 179, 182; the playable ecology plan's `EpochRule` |

### 1.1 Tone

Carried word for word from the founding plan's Tone section on 2026-09-26,
when that plan was archived and this one became Mesocosm's charter (ruling
308). Written 2026-07-30.

> RimWorld vanilla. Sincere, affectionate, mortal. Dark events may happen;
> the endless organ-theft and incest-joke register is out. This matters more
> than usual here, because incorporation carries real ethical weight once
> critters are not necessarily unintelligent — play it with ritual
> seriousness (Qud's water ritual), never as a loot economy.

The traits brief and the trait catalogue plan cite this against a loot
economy for incorporation; pillar 5 of the product description names where
scarcity lives.

### 1.2 The epoch loop's turn structure

Carried from the founding plan's "The epoch loop" section on 2026-09-26
(ruling 308); the ruled sentences keep their words. Ruled 2026-07-30 and
2026-07-31; clarified 2026-08-31.

> **Every species takes a turn**, spending its own accumulated resources to
> adapt, in initiative order — and the same initiative order carries over
> from the epoch. Mutations can be swapped, not merely added.

**Initiative is descending metabolic complexity** (ruled 2026-07-31). The
most complex lineages commit first; simpler lineages act later and can
respond to what those expensive, slower-generating forms just became, which
compresses generation time into one legible adaptation round without adding
visible turns to imitate elapsed generations. The player is one species
among many at this table, which is what makes the world feel like it is
playing too and where trophic cascades become legible. The adaptation phase
is third person, deliberately, the vessel's sanctioned person shift, passing
all three guardrails of the wing founding record §1. Reproduction is the
individual-scale checkpoint inside this structure, not the boundary
(2026-08-31). Whether the epoch ends on a timer or a condition is the
versioned epoch rule (the playable ecology plan's `EpochRule`); whether a run
is limited or unlimited is answered by ruling 181 and phase M4: losing is
not the end, and the played loop's proof is three epochs end to end.

Today this is `World::adapt_round`, in descending recipe complexity with
ties by id, commits landing immediately so a later line answers a world the
earlier ones changed (epoch boundary plan, PE3a); over Isocosm every lineage
weighs its adaptation against the rest of the trophic web (ruling 182),
under M2's lineages-and-the-boundary family.

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
   abilities allow. A region has collapsed when one of its trophic levels is
   gone (ruling 225); regions are ecological, following the world's
   biomass and merging as it falls, so a global collapse is the one
   region's (226). When the player's lineage ends they pick another here or
   a new world, their call (229); a total collapse ends the world as a
   game, and it stays watchable (230).
5. **Modes.** Survival and creative (rulings 180 and 184), chosen by the
   player; creative is also where debugging reads the truth.

## 3. The core and the contract

The contract's shape is ruling 154's.

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

*Amended 2026-10-05 by ruling 591:* `mesocosm-core` moves into Isocosm
whole first and the host points at Isocosm; the families below are then
re-expressed in place.

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
- *Annotation, 2026-10-08 (677 to 680):* M2 is the
  [families re-expression plan](2026-10-08_families_reexpression_plan.md)'s
  Mesocosm share, in its merged order (668), each family retiring as it
  lands (666), certified by draws (667), keeping this phase's web
  condition below. Mesocosm's world family moves last, its re-expression
  being the switch to M3's directing (678), driving ending there (671).
  The driving intents are the critter's own acts and `Idle` goes (679). An
  interim M4 at site grain comes before the full M4 (680).
- *Annotation, 2026-10-08 (681 to 691), trimmed to a pointer 2026-10-10
  under ruling 800:* the interim M4 is the switch (681), and M3's and the
  interim M4's shape (682 to 691) is the
  [directing plan](2026-10-08_directing_interim_m4_plan.md)'s, which owns
  both; the text it held is there, rulings first.
- **M2, absorption by family.** One sub-phase per family of §4, in the
  order ruled (195): matter and processes, bodies, the record, places,
  lineages and the boundary, then effects. Each is done when the family runs in Isocosm under its
  process definitions, conserves its accounts and replays identically, its
  `mesocosm-core` tests are ported or replaced by draws, every moved file
  is within the 600-line ceiling (ruling 193), `mesocosm-core` no longer
  owns it, and the level of the web it enables stays alive in most draws:
  producers alone, then with consumers, then with decomposers closing the
  loop (ruling 260), measured by persistence, turnover, collapse and the
  web's response to an intervention (267): disease, an overperforming
  lineage, or a keystone lost (283). Lands under the sim plan's S1 and S2.
- **M3, directing on Isocosm,** and the **interim M4** at site grain (680):
  owned by the [directing plan](2026-10-08_directing_interim_m4_plan.md)
  since ruling 800 (2026-10-10), which carries M3's done-condition as
  written here on 2026-09-25 and builds both as D1 to D5. The played slice
  plan retired as 196 ruled and is archived at `archive_docs/2026-10-10/`.
- **M4, the played loop.** Done when, from a seed nobody chose and a start
  the player picked, the headed host plays three epochs end to end on
  Isocosm: rounds under directing, a birth keeping the parent by default,
  boundaries where every lineage adapts against the web and the player
  revises in the shop, and a local collapse played on; survival and
  creative modes both work; the receipts replay to the same hash; and the
  fungible agree in distribution within the world's stated tolerance
  (ruling 113). This is W5's done-condition.

M1 and M2 run side by side; M3 needs M2's matter, bodies and places; M4
needs all of M2 and M3. *Annotation, 2026-10-08:* superseded in part by
680 to 682: M3 and the interim M4 run at site grain on a native lineage,
before places and the bodies family; the full M4 still needs both.

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

## Carried from the retired process plans (2026-10-18)

- Body voxel scale against the ground cells is a headed finding still to be
  looked at and tuned. (from after-pass plan §A4)
- Flow windows' births and deaths (the `mesocosm-runtime` readings) are still
  to be compared with the members' actual births and deaths in a real run.
  (from after-pass plan §A4)
- The generated 3x2 map layout is a headed finding still to be looked at in a
  real run. (from after-pass plan §A4)
- Forced births on bodied lineages (the dev ForceBirth) are a headed finding
  still to be tried in a real run. (from after-pass plan §A4)
- DC5's colour and the critter review, the default creatures plan's residue,
  are still open. (from after-pass plan §A4)
- VB3 to VB5's visual acceptance is still to be judged on native bodies:
  pointing to a body and reading what happened, a life changing the picture so
  a descendant grows, and the integrated roster with its cost. (from
  after-pass plan §A4)
- The default roster must still meet its three conditions on native bodies:
  every fauna body senses and contracts, the kingdom floor holds, and the
  captures read as critters. The habitat's canopy, contact and dressing, which
  CP1 left open, are open too. (from after-pass plan §A4)
- P0's judgment is open: whether the headed meal choice feels tense rather
  than clerical. (from after-pass plan §A4)

## Findings

- **2026-10-11, Q16 source integration:** `mesocosm-runtime::perception`
  derives the edited native site's `Volume` and `Places`. Its `Sight` takes
  explicit caller eye/target cells and range, validates living residency and
  admitted patches, then refines the native survival knowledge view through
  `Places::sees` (740, 741). Patch identity anchors and presentation spots
  supply neither eye position nor range. Runtime and owned/borrowed scene
  APIs use the same refinement when actual geometry is supplied. Default
  site-grain survival scenes use the native view; departure or failed lifting
  withdraws the old owned scene, and terrain refresh also tracks condition
  changes rather than only edit count. These are source changes, uncompiled
  in this lane. Native process targets remain at site grain. Carve/deposit
  still need a chosen-process resolution binding to `Command::Edit { by }`;
  679 prevents restoring their player-driving intents. The actual display
  path presents an empty frame after withdrawal, clears captured body/terrain
  pixels and inspection queries, and retains the current native chrome.
  Borrowed producers publish an empty texture instead of returning the
  previous-frame-retaining `None`. Full M4 is open.

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

- **2026-10-12, 847/848:** resident corpses now enter the same native
  awareness and admitted close-up. Real scene consumers use only current
  accepted corpse poses; living observers/actions/crowd counts stay living.
  Authored biological birth profiles select exact native children and resolve
  explicit parent-relative slots, windows and clearance. Their clock/save
  source is assembling; missing/refused placement never undoes conserved
  native birth or inherited journey. Unverified; acceptance/tuning wait826.

- **2026-10-12, biological newborn source (848):** the new native
  AdvanceBirthSpatial path and owned/borrowed/dev clocks resolve real newborn
  IDs from the parent's admitted pose, with explicitly authored ordinal slots,
  clearance, medium and support. Native Step/Birth publication order is kept
  for the new saved path. Missing or refused geometry preserves conserved
  birth and a travelling child's remaining journey. Resident corpse views847
  use current admitted placement and native awareness. Source qualification
  remains deferred under826; full M4 is open.

- **2026-10-12, dev newborn consumer source (848):** real Runtime ForceBirth
  now uses the same authored biological placement provider through an appended
  ForceBirth-only BirthSpatial command. Its original native outcome is retained
  in the new saved record; old ForceBirth wire and replay remain unchanged.
  Unsupported geometry records absence without undoing conserved birth. This
  completes that production consumer connection, not compiler/test acceptance.

- **2026-10-13, exact produced-output native selectors (863):** references
  name genuine accepted operation/output creation facts and exact quantities.
  Current title, holder, registered identity and physical remainder must
  agree; ambiguous or spent claims refuse. Provenance stages with titles and
  respects pass-start state. Unmarked old rules receive no retrospective
  metadata. Existing operator startup remains the consumer; source is
  uncompiled and untested826, and full M4/C11 acceptance remains open.
