# Mesocosm overlay: directing, the played loop and the epoch boundary

**Status:** 2026-10-30: cut on 2026-10-28 from `2026-09-25_mesocosm_overlay_plan.md`,
`2026-10-08_directing_interim_m4_plan.md` and `2026-08-01_epoch_boundary_plan.md`,
which are archived in `archive_docs/2026-10-28/`. Built: `mesocosm-core` and its
sixteen-variant `Intent` are deleted; the four crates (`mesocosm-runtime`, `-views`,
`-phenotype`, `-genet`) run a native `Session` and lower `cosmograph_overlay::mesocosm`
envelopes; directing (D1 to D5) is built on native Cosmograph and the switch has landed;
the boundary, review, revision and speciation are native in `shared/cosmograph/src/lineage/`.
Open: certification of M3 and the interim M4 by draws, the headed three-epoch run, the full
M4 (chosen in-site acts and contact reaching the same accepted world), and the
between-epoch work below.

**For:** Mesocosm as a game over the Cosmograph sim: directing (M3, called M3.1 to M3.3 here),
the interim M4 at site grain (M4i.1 to M4i.3, was directing D4 and D5), the full M4 played
loop (W5's done-condition for Mesocosm) with the headed findings carried into it, SP7 (the
place graph's absorption join, named as this plan's), and what happens between epochs
(prefix EB): judging a run, how a lineage splits, goals, and the youth-cost of plasticity.
The Mesocosm side of the overlay contract is built (`shared/cosmograph-overlay/src/mesocosm/`);
what remains of it is the dev and attention follow-on.

Not here: the sim's schema, processes and families certification (`sim`); route and tick
speed bars (`sim-execution`); the host's mode-host and GUI acceptance (`sim-host`); the
store and its lifts (`place-graph-engine`); body appearance, footing, senses and binding
(`bodies`); the body voxel scale and the default-view ruling (`orthographic-voxel-presentation`);
hagiograph significance and the glyph organ (`wing-organs`); the wing's portable body contract
(`wing-phenotype-contract`); parked rungs (`wing-organs` D1).

## Constraints

- **Directing, never driving.** A click draws the critter's attention; it weighs each nudge by
  its bond (rulings 59, 60, 175 to 178, 214 to 216). Non-sapient critters are never driven (671);
  `Idle` has no role and the driving intents are the critter's own acts (679); 679 forbids
  restoring player-driving carve and deposit. From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md` §1, §3.
- **Directing only on Cosmograph, no throwaway adapter** (ruling 194). Absorption order ruled in
  195; the played slice plan retired into the overlay plan at M3 (196). From:
  `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md` §6.
- **The switch is the interim M4, before places** (681 amends 678); in-site play (carve,
  deposit, voxel sight) waits for the full M4. From:
  `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:20-30`.
- **Survival shows what the played critter can know; creative shows the truth and edits
  nothing** (180, 184). Presentation spots never supply sight geometry (741). From:
  `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md` §1, Findings.
- **Losing is not the end** (181). Initiative is descending metabolic complexity, read since
  806 as the union of distinct catalogue functions (797). From:
  `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:97-126`.
- **Tone**, written 2026-07-30 and kept: "RimWorld vanilla. Sincere, affectionate, mortal."
  Incorporation is played "with ritual seriousness (Qud's water ritual), never as a loot
  economy." From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:83-91`.
- **Done-conditions are draws from a declared space, never fixtures** (ruling 15); every moved
  file stays within the 600-line ceiling (193).
- **Epoch boundary stop rules** (record rulings 2, 3, 4, 202, 280):
  - Do not speciate on a similarity threshold; splitting is an act with a record of why.
  - Do not edit a lineage revision in place or merge concurrent developmental proposals cell
    by cell; agreement adopts one child, disagreement preserves descent.
  - Do not author a significance table.
  - Do not collapse retroactive abnormality and selected goals into one system.
  - Do not let the storyteller pick outcomes.
  - Do not offer a goal that has not been verified achievable and novel.
  - Do not make plasticity free; youth buys change with capability.
  - Do not store a temperament; it is a fold over substrate and history.
  - Do not grow the event log without a selector.
  - Do not make the world record answer traversal questions.
  - Do not note a zero.
  - Do not compute a range from where a lineage is; it is a high-water set.
  - Do not break the semilattice: `Mark::join` stays commutative, associative and idempotent.

  From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:611-638`.

## Phases

### Directing (M3, interim M4)

M3 as a whole is done when: "Done when a played critter acts on its own needs, senses and
mood under the player's nudges, its range, home, priorities and stances grow from that
attention, its bond moves with outcomes and passes across generations as the world setting
says, its suggestions surface, and a seeded run replays to the same hash." The three phases
below build it; each is built and awaits certification by draws.

### M3.1. Participants, the bond and the nudge (was D1)
Certify the placeless participant entity, the weighted bond relation (688), the logged native
`Nudge` command (686) naming a site or a thing to attend or to act (690), and 178's three
settings.
- Done when: "a seeded run with nudges replays to the same hash, a v3 save from before D1 still
  loads, and in the bond's controls outcomes that serve the critter raise the weight and
  outcomes that harm it lower it, over draws."
- Needs: -
- Rulings: 177, 178, 194, 216, 686, 688, 690
- From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:82-91`, status line 5.

### M3.2. The deliberative methodology (was D2)
Certify `Method::Deliberative`: a deliberative critter chooses one Choice process by needs,
mood, nudges and bond, the rest going to `Receipt.foregone`.
- Done when: "an un-nudged critter acts on its own needs; a nudge moves its choice in
  proportion to the bond, and with a zero bond it moves nothing (the negative control); every
  other methodology's runs are unchanged (their hashes before and after match); and choice is
  deterministic under the seed."
- Needs: M3.1
- Rulings: 683
- From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:92-101`.

### M3.3. Readings (was D3)
Certify suggestions (685), standing orders (690), regions (687, 689; each trophic level's slot
capacity a world rule, regions growing along site routes) and the survival filter (691), each
a pure function of state and log. Deciding M3.3 also decides the deliberative scorer's terms
(see Questions, scorer readings).
- Done when: "each reads the same from a replayed run as from the live one, a planted fault in
  each is caught by its test, and regions merge into one as a draw's biomass falls."
- Needs: M3.2
- Rulings: 685, 687, 689, 690, 691
- From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:102-108`.

### M3.4. Diagnostic residue
Remove the test-only scratch diagnostic `directing/interim/diag.rs` (a consumer's income,
upkeep and age, left by the reproduction lane), or move it to an example.
- Done when: not stated; owed
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:143-145`.

### M4i.1. The interim loop, headless (was D4)
Certify founding from a seed, a start the player picks (179), rounds and epochs, a birth
keeping the parent by default (183), death handing the next life from the cohort (61), the
boundary where every lineage adapts by grow-a-copy scoring over `fork_at` (684), a regional
collapse (225) played on elsewhere, and survival and creative modes.
- Done when: "Done headless, by draws: three epochs end to end from seeds nobody chose,
  receipts replaying to the same hash, controls for a collapse that does and one that does not
  happen."
- Needs: M3.1 to M3.3; the forks in Questions (move acts, a region that never held a level, the
  boundary candidate, a shared participant); M4i.3 for journeys that take route time;
  `sim` consumer reproduction in a real run (its tuning item) for epochs of stock length
- Rulings: 61, 179, 183, 225, 684
- From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:109-117`, `:124-139`.

### M4i.2. The switch, headed (was D5)
Section, HUD, succession and review lanes run over the `Session` runtime; legacy driving
removed. Certify in the headed host (case 680).
- Done when: "680's condition holds in the headed host: three epochs end to end at site grain
  on Cosmograph, both modes, receipts replaying."
- Needs: M3.1 to M3.3; M4i.1; `sim-host I3` (the mode host and its headed acceptance)
- Rulings: 671, 679, 680, 681, 686
- From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:118-123`, status line 5.

### M4i.3. Route-duration journeys
Native transit (817, 820 to 823): migration departs and arrives after the traversed route's
authored travel time; the traveller keeps ageing and paying upkeep, a paid birth shares the
remaining journey, and death carries the corpse to the destination. Native transit state,
scheduling and replay are to be verified; the old archives, the rejected reproduction
candidate, the eighty-world A1 matrix and headed certification remain open.
- Done when: not stated; owed
- Needs: the compile and test gate deferred under 826; M3.1
- Rulings: 817, 820, 821, 822, 823, 826
- From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:261-292`.

### Full M4

### M4.1. A declared process for generated bodies' step, terrain and birth inputs
Generated worlds declare spawn and arrival templates (889) and a global-collapse predicate
(891); generated bodies' step, terrain and birth inputs are ones "which no template can
satisfy" and need a declared process.
- Done when: not stated; owed
- Needs: `sim` generator and process definitions
- Rulings: 889, 891
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:5-14`.

### M4.2. Certify the generated worlds' templates and collapse predicate
Certify the spawn and arrival templates (889) and the global-collapse predicate (891), merged
but uncertified; the producer capture path (890) is built.
- Done when: not stated; owed (certification by draws, ruling 15)
- Needs: M3.1 to M3.3
- Rulings: 15, 889, 890, 891
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:5-14`.

### M4.3. Chosen-process `Edit` binding
Native process targets are site grain; carve and deposit still need a chosen-process
resolution binding to `Command::Edit { by }`. The Q16 perception and scene source (741's ray,
withdrawal of departed or refused inputs, an empty frame after withdrawal) is source changes
whose compile and test qualification is outstanding.
- Done when: not stated; owed (a gate of the full M4)
- Needs: M4i.2; `bodies` binding (contact and jointed collision admission); compile and test
  qualification of the Q16 source
- Rulings: 679, 740, 741
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:33-38`, `:327-343`.

### M4.4. Newborn placement and exact produced-output selectors
The biological newborn and `ForceBirth` placement source (847, 848: authored birth profiles,
the `AdvanceBirthSpatial` path, dev `ForceBirth` through an appended `BirthSpatial` command)
and exact produced-output native selectors (863). Both are source "uncompiled and untested";
"full M4/C11 acceptance remains open."
- Done when: not stated; owed
- Needs: the compile and test gate deferred under 826; M4.1
- Rulings: 826, 847, 848, 863
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:366-396`.

### M4.5. The web condition
After `mesocosm-core`'s deletion, the web condition that M2 set remains to be shown by draws.
- Done when: each family "runs in Cosmograph under its process definitions, conserves its
  accounts and replays identically, its `mesocosm-core` tests are ported or replaced by draws,
  every moved file is within the 600-line ceiling (ruling 193), `mesocosm-core` no longer owns
  it, and the level of the web it enables stays alive in most draws: producers alone, then
  with consumers, then with decomposers closing the loop (ruling 260), measured by
  persistence, turnover, collapse and the web's response to an intervention (267): disease, an
  overperforming lineage, or a keystone lost (283)."
- Needs: `sim` families certification and S1, S2
- Rulings: 193, 195, 260, 267, 283, 666, 667, 668, 677 to 680
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:258-268`.

### M4.6. SP7, the absorption join
Mesocosm's part of the place graph's join: full M4 in-site play over Ground and Places, of
which the existing slice only connects stands, walking and biological child placement through
the native spatial clock. "full M4, T2 and re-certification of the broader substrate remain
separate."
- Done when: the place graph plan's wording was "Done under the Mesocosm overlay plan's M2 for
  places" (SP7); no condition is stated here; owed
- Needs: M4.5; `place-graph-engine SP4` and `SP5`; `eponym-overlay E3` and `vtt-overlay V2`
  for the other two parts of the join; `place-graph-engine SP6` for T2
- Rulings: 89, 195, 681, 755, 776, 799
- From: `archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md:3-9`, `:297-301`, `:532-555`.

### M4.7. Headed findings
To look at and tune in a real headed run:
- flow windows' births and deaths (the `mesocosm-runtime` readings) against the members'
  actual births and deaths;
- the generated 3x2 map layout;
- forced births on bodied lineages (dev `ForceBirth`; needs M4.4);
- DC5's colour and the critter review (the default creatures plan's residue);
- VB3 to VB5 visual acceptance on native bodies: pointing to a body and reading what happened,
  a life changing the picture so a descendant grows, and the integrated roster with its cost;
- the default roster meeting its three conditions on native bodies: "every fauna body senses
  and contracts, the kingdom floor holds, and the captures read as critters", with the
  habitat's canopy, contact and dressing that CP1 left open;
- P0's judgment, by Mark: whether the headed meal choice feels tense rather than clerical.

Body voxel scale against the ground cells is the same headed run's finding and belongs to
`orthographic-voxel-presentation`.
- Done when: not stated; owed (the roster's three conditions as quoted; P0 is Mark's judgment)
- Needs: a headed host run on native bodies; M4i.2
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:301-323`.

### M4.8. Dev intents and attention changes
The contract's dev side (end epoch, force birth, kill, place matter) and attention side (pin
and unpin any pointable thing; examine the region up close) remain a follow-on; pin and unpin
are in source only.
- Done when: not stated; owed
- Needs: `sim-host I3`
- Rulings: 113, 130, 210, 212
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:171-172`.

### M4. The played loop (W5's done-condition)
Rounds under directing, a birth keeping the parent by default, boundaries where every lineage
adapts against the web and the player revises in the shop, a local collapse played on, in both
modes.
- Done when: "from a seed nobody chose and a start the player picked, the headed host plays
  three epochs end to end on Cosmograph: rounds under directing, a birth keeping the parent by
  default, boundaries where every lineage adapts against the web and the player revises in the
  shop, and a local collapse played on; survival and creative modes both work; the receipts
  replay to the same hash; and the fungible agree in distribution within the world's stated
  tolerance (ruling 113)."
- Needs: M3.1 to M3.3, M4i.1 to M4i.3, M4.1 to M4.5, EB1 and EB5 (where every lineage adapts)
- Rulings: 57, 61, 113, 124, 178, 179, 181, 182, 183, 225, 226, 229, 230, 680, 800
- From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:274-281`, `:128-160`.

### The epoch boundary (EB)

### EB1. Boundary certification
Broader invariant and headed certification of the native boundary, review, revision and
speciation, and the metabolic order of case 806.
- Done when: not stated; owed
- Needs: `sim` lineages-family certification; the headed run (M4i.2)
- Rulings: 791, 797, 806
- From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:3`, `:750-754`.

### EB2. Plasticity as a life stage
Juvenile malleable and weak, mature capable and set; plasticity a depleting resource; `Stage`
(`Juvenile | Mature | Carrion | Spent`) gets mechanical teeth. Mark, ruled: "more ability to
develop, at the cost of the debuffs and vulnerabilities of youth." Stop rule: "Do not make
plasticity free. Youth buys change with capability." No juvenile cost exists in native code.
Revisions are priced flat for now (see Questions, plasticity).
- Done when: not stated; owed
- Needs: the plasticity question; the adaptation bridge for revising a developmental program
- Rulings: epoch plan §5 (Mark), stop rules
- From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:239-264`.

### EB3. Goals between epochs
The player picks from roughly five randomly generated goals, each "a guaranteed special event,
verified and rated against the world record." The generator that guarantees achievable, novel
and lineage-suited goals "is the expensive part of this plan and it wants bounding before it is
built: a goal that turns out to be impossible is worse than no goal at all, and a goal that is
trivially achievable is worse than either." Retroactive abnormality and selected goals are two
reward channels and both stay. Not built (native `goals/` is the society layer's).
- Done when: not stated; stop rule: "Do not offer a goal that has not been verified achievable
  and novel."
- Needs: the goal-failure question; `sim S5` (a generated run reaching the epoch board, 887)
- Rulings: epoch plan §4 (Mark); 887
- From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:209-237`.

### EB4. Speciation pays
A reward ladder (a boon to the running lineage; the right to fork, name the new line and claim
a distinguishing ability; "more at the top, unspecified for now"). Reward and split are the
same event. The top rungs are unspecified.
- Done when: not stated; owed
- Needs: -
- Rulings: epoch plan §2 (Mark)
- From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:88-101`.

### EB5. Significance
Code applies only the novelty (abnormality) gate; the quality and relevance gates (ruling 4)
are not applied. The judging organ is the hagiograph (`wing-organs`), not this plan. Two axes
stay empty by design (Symbiosis, Construction: nothing gives to another creature or changes
the enclosure; "Do not note a zero").
- Done when: not stated; owed
- Needs: the hagiograph (`wing-organs`)
- Rulings: 4, 280
- From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:183-206`, `:566-569`.

### EB6. Shared lineage authorship
Adopt-or-branch with immutable child revisions; agreement covers the heritable developmental
program, not the allocation mosaic (Mark, 2026-08-03). Revision and review are native
(`lineage/revise.rs`, `review.rs`); concurrent proposal handling and the rule for persistent
and asynchronous adoption are open.
- Done when: not stated; owed
- Needs: the adoption question
- Rulings: epoch plan §2 (Mark), 2026-08-03
- From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:103-131`.

## Deferred and parked

- **NPC speciation**: an unplayed lineage splits on a significant event; the player's split
  stays the player's act (202). Waits on the rule for what counts as significant enough to
  split a line (EB5, and the unplayed-lineage naming question). From:
  `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:594-596`, `:483-494`.
- **Contested flow (carried P5)**: lift finite local resource results into the boundary's
  evidence. Waits on the full M4's boundaries where every lineage adapts against the web (182).
  Done when "niche overlap can cause a lineage to fail, distinct resource paths can coexist,
  and changing spatial access changes what a line's boundary weighs, without a global
  fitness-share rule; crossing the local resolution boundary conserves biomass and lineage
  state rather than rerolling the ecology." From:
  `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:678-686`.
- **Events carry place and tick**: sim events are stamped (203) and sited and an event's reach
  is a field on the graph (5, 84); naming a place is a location asserted when noted (ruling
  200 promotion) and events of note become tales (157). Place-naming is not located in code.
  Waits on the places family (M4.6). From:
  `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:584-608`.
- **Source qualification under 826**: M4.3, M4.4 and M4i.3 are source whose compile, test,
  fixture and tuning gate is deferred under ruling 826. Waits on that deferral being lifted.
  From: `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:366-396`.

## Questions for Mark

Directing forks (put back by the directing lane, 2026-10-09); each blocks M4i.1.
- **Which acts move a body between sites.** Native `Effect::Move` has a fixed destination, so a
  generated move needs a process per route; against starting again elsewhere by `Take`
  (`OnCollapse`). Playing on after a regional collapse needs a generated process that uses
  `Effect::Move`. Rulings 225, 817, 820 to 823. From:
  `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:128-129`, `:134-139`, `:244-247`.
- **Whether a region that never held a level has collapsed.** Ruling 225 says "a level gone";
  at site grain most generated founding draws show such regions on round one. The tests use a
  region as wide as the world. From:
  `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:138-139`, `:239-243`.
- **What a native lineage's boundary candidate is.** No native operation revises a lineage
  (`interim::no_candidates`). Ruling 684; needs the `sim` lineages family. From:
  `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:136-138`, `:244-247`.
- **Whether a participant entity can be shared by games beyond Mesocosm** (the VTT's players,
  Eponym's sophont player). Ruling 688. From:
  `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:132-133`.
- **Scorer readings not ruled**: the deliberative scorer's terms (needs fed times mood, nudge
  sway, priority, identity order for ties); "served" as no worse in mood and matter; the
  deep-time span as the world's own plus the player's added epochs; boundary initiative by
  members (legacy's complexity has no native reading; 797 and 806 later set it to the union of
  distinct catalogue functions). Ask whether to fold these into rulings. Rulings 683, 797, 806.
  From: `archive_docs/2026-10-28/2026-10-08_directing_interim_m4_plan.md:248-252`, `:217-226`.
- **Mesocosm contract readings**: (a) Mesocosm's handoff is empty, with no ruleset beside the
  sim, "unless a later rule adds a game-resolved outcome" (still stands); (b) the sixteen-intent
  split, where `Move/Metabolize/Consume/Graft/Deposit/Carve` become the critter's own acts,
  `Idle` loses its role, and `TakeControl/Resume` answer the birth checkpoint and `TakeControl`
  the death checkpoint (a reading of 61 and 178; built, never confirmed); (c) `voxel_profile`
  stays presentation's (the module exists). Rulings 61, 178, 202, 671, 679. From:
  `archive_docs/2026-10-28/2026-09-25_mesocosm_overlay_plan.md:175-192`, `:219-221`.

Epoch boundary (record: open, the record does not decide).
- **Does an unplayed lineage's speciation get a name?** If naming is the act, something must
  name NPC forks; a procedural name is a different kind of fact from a player's. The record names
  the condition (an unplayed branch needs a plan to grow and to compete or cooperate); the name
  is a denizen-style promotion (200) "still unruled for lineages". Rulings 57, 182, 200, 202.
  From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:649-650`, `:667-670`.
- **Can a goal be failed, or only unmet?** Failure implies stakes and a record; unmet implies it
  simply expires. From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:651-652`, `:670-674`.
- **Does plasticity apply to anatomy, temperament, or both?** Partly ruled by Mark, 2026-09-01:
  "flat first": a revision is priced flat; whether youth later buys developmental change more
  cheaply stays open, and the one place a multiplier would land is `program::express`. From:
  `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:652-657`, `:671-673`.
- **How does ancestral distance survive a world boundary?** Species ids mean nothing across
  worlds; cross-world distance needs the wing contract's subject identity
  (`wing-phenotype-contract`). From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:658-661`.
- **Who must adopt a shared continuation?** The same-session case is clear; persistent and
  asynchronous play needs a standing rule. Requiring every historical inhabitant would freeze
  old lineages; treating silence as consent would rewrite somebody else's continuation. From:
  `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:662-666`.
- **Does the old "exact reproduction choice and epoch-ending rule remain open" still stand?**
  The epoch rule's three kinds and deep time are in `rules/epoch.rs` and reproduction is native;
  only the boundary's own review, weighing of evidence and world readings is left. Rulings 451,
  452. From: `archive_docs/2026-10-28/2026-08-01_epoch_boundary_plan.md:45-48`.

## Findings
- 2026-10-30: `mesocosm-runtime`'s integration tests ran 56 passed and 36 failed at `76f8f5cc`, with four lib failures, the same with the frame call disabled (`sim-execution` Findings). Many pin behaviour the game has since changed (decision periods, D19). Fixing them is not a work item (Mark, 2026-10-30).

- 2026-10-28: `mesocosm/crates` holds `mesocosm-genet`, `-phenotype`, `-runtime`, `-views`
  and probes; `mesocosm-core` is gone. `cosmograph::legacy` appears only in design docs, so the
  directing plan's "legacy driving" survey is stale. `mesocosm-runtime/src/runtime/translate.rs`
  lowers `cosmograph_overlay::mesocosm`, so the claim that the overlay crate is "depended on by
  nothing" is stale. No Rust file under `mesocosm/crates`, `eponym/crates` or
  `shared/cosmograph/src` exceeds 600 lines.
- 2026-10-28: habitability for a critter is read natively
  (`shared/cosmograph/src/directing/readings/habitable.rs`, citing 179 and 751; the interim
  `begin` waits for a habitable site). The 2026-10-09 fork on it is closed in code; the
  player-picked start still needs the headed run (M4).
- 2026-10-28: `OnCollapse` has `Stay` and `Elsewhere` (`directing/interim/mod.rs`) and
  `Effect::MovePaced` exists (`rules/effect.rs`), so the move-acts fork has both options
  expressed; the choice is Mark's. `no_candidates` is still the only boundary candidate
  supplier in `directing/interim`.
- 2026-10-28: `directing/interim/diag.rs` is still present (M3.4).
- 2026-10-28: done: wounds feed mood (227) (`directing/wound_tests.rs`). The Q3 items (the
  empty-choice retry 813, pass-start prey 454, seeded movement ties 958) are integrated with
  their tests (`choice_tests.rs`, `pass_start_tests`, `movement_tie_tests.rs`); not open unless
  certification finds otherwise. The 754 clause "or that the world holds within reach" is
  unbuilt. Generated-ecology consumer reproduction belongs to `sim`'s ecology tuning.
- 2026-10-28: `Command::Edit { by: Some(..) }` is handled in `mesocosm-runtime/src/glyphs.rs`
  and `terrain.rs`; the chosen-process binding itself is not confirmed. `perception.rs` and
  `voxel_profile.rs` exist.
- 2026-10-28: `shared/cosmograph/src/journey.rs` and `journey/` exist; their verification state is
  unknown (M4i.3).
- 2026-10-28: in `shared/cosmograph/src/lineage/` the boundary, review, revision, speciation,
  reckoning and tree exist; no plasticity or life-stage cost, no goal generator and no NPC
  speciation were found. `history.rs` stamps events (203).
- 2026-10-28: stale in the archived plans: the epoch plan's `mesocosm-core::{history, species,
  record, places, score}`, `World::end_epoch` and `Intent::Speciate` are deleted, with native
  equivalents in `lineage/`, `history.rs` and `reach.rs`; the overlay plan's "Proposed, not
  opened" labels and §4 absorption tables describe migration inventory that is finished; the
  directing plan's eight-tick epoch note is superseded by A1 consumer reproduction passing in
  `sim`.
- 2026-10-28: the isomere plan's M0 to M6 and the wing phenotype plan's W0 to W6 are other
  namespaces that reuse these labels; this plan's M labels are the overlay plan's only.

## Progress
- 2026-10-28: cut from the Mesocosm overlay plan, the directing and interim M4 plan and the
  epoch boundary plan.
