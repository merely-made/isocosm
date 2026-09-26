# Eponym's overlay: the W5 plan

**Date:** 2026-09-25

**Status, 2026-09-25:** plan; E0 to E4 proposed and not opened. Drafted at
Mark's word ("Overlay plans to RPG") in the RPG systems session, in parallel
with the Simulation design review session, from the
[wing design record](../../mesocosm/design_docs/2026-09-18_wing_design_plan.md)'s
§5.6 (rulings 185 to 187), ruling 60, ruling 130, rulings 152 to 156, the
contract's rulings (197, 203 to 205, 210 to 213), and Eponym's own plans. It
mirrors the [Mesocosm overlay plan](../../mesocosm/design_docs/2026-09-25_mesocosm_overlay_plan.md),
the first game's. Every row cites the ruling or plan it rests on; a reading
of this plan's own is flagged as one. No lane runs until Mark opens it, and
§6's decisions are his before E0 closes.

**Owns:** Eponym's profile as a game over the Isocosm sim (the record's §5);
the Eponym side of the overlay contract (ruling 154), a module beside
Mesocosm's in `shared/isocosm-overlay` (ruling 197); the played loop W5
names; and the order in which `eponym-world`'s and `eponym-social`'s
simulation moves into Isocosm for this overlay's needs (§4, §6 decision 4).
**Does not own:** the sim, whose schema, process definitions, record and
generator the [sim plan](../../mesocosm/design_docs/2026-09-22_sim_plan.md)
owns, so every move lands under its phases; the bench (W4); the other
overlays (the record's §5.5 and §5.7); the stack's identity organ, which
takes `eponym-identity` under W3 (ruling 34); or naming. **Consumes:** the
record; the sim plan; the contract crate's
[README](../../shared/isocosm-overlay/README.md); the
[founding plan](2026-07-30_paredros_founding_plan.md), the charter; the
[execution plan](2026-08-07_paredros_execution_plan.md)'s fundamental-layer
ledger F0 to F8; the [functional loops plan](2026-09-09_functional_loops_plan.md)
for the joins and lane S; the
[memory and remembrance plan](2026-09-09_memory_and_remembrance_plan.md) for
lanes M and H; the [world conditions plan](2026-09-09_world_conditions_plan.md),
which founds the sim's process definition (ruling 32); the
[genet document host plan](2026-09-13_genet_document_host_plan.md) for the
host that stays Eponym's; and the
[vessel briefs](../../mesocosm/design_docs/2026-08-18_vessel_briefs_and_presentation.md)
§2 for the camera.

---

## 0. Done-conditions

From the record's §11, W5 is done for a game when it has:

1. a profile designed to the record's §5 (§1 here);
2. a core implementing the overlay contract (§3);
3. a played loop with receipts drawn from the generator (§2, phase E4).

Ruling 174 made Mesocosm the first overlay; whether Eponym or the VTT goes
second is not ruled (§6 decision 7).

## 1. The profile

Ruling 6's six parts.

| Part | Eponym | Rests on |
| --- | --- | --- |
| Domain | one named creature, a sophont, lived in the second person until it dies: the foregrounded rung is the denizen and its factions, and what play refines is what remembered entities do, the coterie, the party, the base | 35, 36, 60, 200; the record's §5 |
| Ensemble | the named creatures of the places one life reaches, settlements, dungeons, ruins, surface and underground places, and what the played sophont knows of them; its care, the collector's roots, is the sophont and those it knows | 3, 6, 71; founding plan §1 |
| Mechanics | driving one body in real time, its acts resolved by what the body affords; asks to peers as proposals a peer weighs by its opinion of the asker and may refuse or counter; telling and hearing, with beliefs that can be wrong; building and crafting as embodied acts that incorporate matter; harm as vigour and wounds to the body's tree; fights sized up first and, when they escalate, strain against bearing; skills raised by doing, teaching, study and breakthroughs; secrets kept by taboo | 60, 63, 87, 95, 96, 115, 116, 117, 118, 123, 141, 142, 221 to 223 |
| Controls | driving: the actuation of one body, which the contract opens to this game alone; directives to companions compose on top as proposals weighed by opinion, never orders; the player directs only who they play; naming; notes written in the world; pins, and the region the view shows up close | 60, 130, 152, 168, 210, 212; the record's §5.2 point 2 and §9.14; the founding plan's "Leverage without command" |
| Perspective | third person on one continuous zoom, near acts, mid leads, far plans, with first person a setting; survival shows what the creature one lives knows, its diegetic notes included, creative shows the truth and is where tag-in lives; the legibility surface ships with the simulation, not after it | vessel briefs §2; 187; PROJECT_DESCRIPTION pillar 4 |
| Timescale | the moment: one life lived in real time until death, then succession; the world's clock proceeds while played, deep time before it; where in that time a life begins is §6 decision 2 | vessel briefs §2 ("the rhyme"); 61, 93, 104, 105, 186 |

## 2. The played loop

1. **Founding.** A seeded draw from the generator's declared space, the
   region being its first edge (ruling 124). Eponym needs sapient people and
   society (ruling 3), so the draw carries deep time enough for settlements
   to stand (rulings 91, 93); how much, and how the first life begins, are
   §6's decisions 2 and 3. *Reading, not ruled:* the founding flow's
   history parameter (sim plan §5.1) is where that span is set.
2. **The life.** The player drives one body (ruling 60): its motion, its
   timed acts, what it observes. Its needs move the one ledger (ruling 38)
   and its acts score by need, trust and approval (ruling 95, D26). An ask
   to a peer is a proposal (ruling 63): the peer answers by its own
   methodology weighed by its opinion of the asker (ruling 60), refusal and
   counteroffer complete outcomes, and a standing agreement is consent that
   holds until its premises change (rulings 63, 67; the founding plan's
   "Offers become standing agreements"). Telling plants a note on the hearer
   or leaks into the reach field (ruling 87); what is told may be false and
   takes by four things (rulings 117, 118). Building and crafting are the
   one verb pointed outward, items bearing their maker and materials
   (ruling 95), and a carve in the ground is an asserted fact (the record's
   §1, ruling 89). A fight is sized up first, most ending there (ruling
   116); one that escalates strains both sides against their bearing, a
   break advantaging or disadvantaging, and ends when a side yields on
   re-sizing or is spent (rulings 221 to 223); a blow drains vigour and
   wounds a part (ruling 123). The sophont writes notes in the world, a
   bearer that can be found, read, lost or stolen (rulings 127, 130), and
   names what it meets (rulings 36, 168).
3. **Death and succession.** Death is final (ruling 61); the body and the
   consequences remain, and the world keeps what it remembers of the dead
   (ruling 129). The player chooses who to become among the companions with
   a bond to the one who died (ruling 186); the case with no such companion
   is §6 decision 3. Tag-in lives in creative mode only (ruling 187).
4. **Co-op.** Each player lives their own named creature in the same world,
   peers to each other as to anyone (ruling 185); ruling 153's shared
   directing is not Eponym's shape. Whether E4 plays it over the network is
   §6 decision 6.
5. **Modes.** Survival and creative (ruling 187), chosen by the player;
   creative is also where debugging reads the truth.

## 3. The core and the contract

The contract's shape is ruled (ruling 154): a game submits intents, the sim
returns events and a read-only view of each tick, and outcomes a game
settles come back through the handoff. Eponym is the game the contract
opens actuation to: §5.2 point 2 carries "directives, and where the overlay
opens it, the actuation of one body", and §9.14 names driving as Eponym's
(ruling 60). Its module is `shared/isocosm-overlay/src/eponym/`, beside
`mesocosm/` (ruling 197), changing nothing in the core.

| Direction | Eponym's side | Rests on |
| --- | --- | --- |
| In: actuation | the played body's motion input and timed acts, a batch of its fixed-step frames stamped for one tick, targets named by entity handle and places by place-graph handle; the one body the participant plays, never a second | 60, 152, 203, 205; §5.2 points 1 and 2, §9.14; *how the frames cross and where the solver runs is §6 decision 1* |
| In: asks | a proposal to one peer: the work, the terms, the danger; the sim answers by that peer's methodology weighed by its opinion of the asker; a standing agreement is a proposal accepted once and held | 60, 63, 67 |
| In: tellings | a claim told to one hearer; what is told may be false, and the hearer takes it by what it can check, who is telling, what it wants to hear and how it is told | 87, 117, 118; *posing to intimidate, persuade or deceive is the record's flagged reading in §3.4, §6 decision 5* |
| In: the player's acts | naming the creature one lives and what it meets, the name being the doing; writing a note, a bearer in the world | 36, 168, 130, 127 |
| In: checkpoint answers | at a death, the companion to become, among those with a bond to the dead | 61, 186; §6 decision 3 for the case with none |
| In: creative | tag-in to a companion and tag-out to home, in creative mode only | 187; `eponym-identity`'s `ControlIntent::{TagIn, TagOut}` |
| In: attention changes | pin or unpin any pointable thing; the up-close region as the zoom nears a place | 210, 212 |
| In: dev intents | none: the crossing fixture's body presets are receipts, not play (*a reading*) | |
| Out: events | the stream derived from the attention set: what touches the sophont one lives, what it pins, the region it examines; in survival only what it can know, its diegetic notes included; the legibility surface reads this stream | 187, 204, 213 |
| Out: views | the near rung around the played body for the follow camera at every zoom; far zooms read the crowd | 154, 212 |
| Handoff | Eponym's blows: today the foreground resolves a strike geometrically and hands back harm in the sim's terms, vigour drained and wounds to parts, which must pass the sim's invariants and agree with the sim's own fight in distribution; whether this handoff stays or empties is §6 decision 1 | 114, 123, 154; the record's §3.8 |

*Reading, not ruled:* Eponym's willingness rule, three gates in order, can I
do it, would I risk that for you, is it more than I would bear
(`eponym-social/src/willing.rs:7-23`), is the act score of ruling 95 asked
of the peer with the asker's standing as a term, so an ask needs no rule of
its own in the sim; refusal about the work, refusal about the asker and
counteroffer are the three ways the score falls short.

**The intents today.** `eponym-world`'s `GameIntent`
(`src/transitions.rs:59`) holds seventeen variants, its `WorldIntent`
(`src/world.rs:33`) two, and `eponym-identity`'s `ControlIntent`
(`src/control.rs:24`) four. *Reading, not ruled, confirmed intent by intent
in E1:* under driving they split four ways. `Move`, `Observe`, `Take`,
`Eat`, `Rest`, `Wait`, `AdvanceMotion`, `ConfigureMovementProfile`,
`AttachItem` and `DetachItem` are the body's acts and become actuation;
`ResolveVolley` is the handoff's outcome or the sim's own fight, by §6
decision 1; `Name` is the player's act. `Generate`, `Fall`, `AdmitAnatomy`,
`ReconcileAnatomy` and `Carve` are the sim's own: lifting a body from the
roster (ruling 75), a consequence of motion, the body's revisions as the
record's facts, and a world edit asserted by an act; `InheritSite` is
founding, authored content displacing generated (ruling 89); `ReviseCanon`
is the hagioglyph organ's. Of the control intents, `Begin` becomes who the
participant plays, the attention set's `played`; `TagIn` and `TagOut` are
creative mode's; `Succeed` is the death checkpoint. The `Session` that owns
control history and its save cut (`src/session.rs`) is the shape the
contract replaces: intents are the replay log (the record's §5.2 point 2).

## 4. Absorption: `eponym-world` and `eponym-social` into Isocosm

The record's §2 test puts whatever runs with nobody playing in the sim, and
`eponym-world`'s `Simulation` advances every living subject with "no
observer, camera, selected subject, or control identity" (execution plan
F2). Ruling 192 ruled the same absorption for Mesocosm; for Eponym it is §6
decision 4, and the table below is what it would move. `eponym-world` is
11,508 lines of source and `eponym-social` 2,747; every family that moves
is decomposed under the 600-line ceiling as it goes, this repository's own
rule (`eponym/CLAUDE.md`), split along seams the code already has.

| Family | Modules, lines | Destination in Isocosm |
| --- | --- | --- |
| Bodies, needs and holding | `anatomy` 536, `subject_sheet` 482, `items` 323, `bodies` 234, `movement_profile` 162, `equipment` 41 | the critter: part tree, the one ledger with needs, injuries and revisions, gear-limited holding (rulings 36, 38, 53); the sim plan's §2.3 |
| Motion and contact | `contact` 1,029, `timed_action` 690, `movement` 350, `motion` 293, `navigation` 256 | the near rung's kinematics over the stack's conatus, with the accepted transition the sim's and navigation staying derived advice; where the solver runs is §6 decision 1 |
| Fights and technique | `combat` 812, `technique` 799 | the competing instance and harm (rulings 115, 116, 123, 221 to 223), quality as affordance (ruling 96, D27), abilities, skills and techniques (rulings 96, 141 to 143); the strike resolver's fate is §6 decision 1 |
| Places and edits | `world` 272, `sites` 211 | the place graph: a `SlotId` is the record's site and a `Site` its location (rulings 72, 147; the record's §3.7.1); carve and inherit as asserted facts (§1, ruling 89) |
| Lives and rounds | `simulation` 471, `population` 245, `projects` 220, `simulation_record` 177 | the denizen's methodology (ruling 37), needs and the act score (ruling 95, D26), deep time (ruling 93), the record |
| Glyphs and canon | `glyphs` 949 | the world's canon and the hagioglyph organ (ruling 101; sim plan §2.7), beside `wing-glyphs` |
| Session, state and transitions | `state` 959, `session` 697, `transitions` 406 | the contract: the intent grammar splits as §3 says, saves become the sim's log, and the control pointer becomes `played` and the death checkpoint |
| Standing and deeds | `deed` 179, `relation` 113 | alignment and reputation derived from acts (rulings 51, 56) |
| Asks and agreements | `response` 201, `willing` 182, `companion` 140, `agreement` 126, `offer` 79 | proposing and joining (ruling 63), directing weighed by opinion (ruling 60), standing agreements (ruling 67) |
| Settlement | `settlement` 227, `settling` 125 | homes offered under agreements (rulings 54, 63); a settlement a faction until it has a methodology (rulings 8, 63) |
| Knowledge | `epistemic` 570 | notes on knowers, reports and corrections (rulings 84, 86, 117) and the secrets regime (ruling 87); the sim plan's §4 |
| Society | `society` 462 | the join of the above at the faction rung; confirmed as it moves |
| Fixtures and scenes | `fixtures` 786 (world), `scene` 135 and `bin` 140 (social) | receipts, replaced by draws (ruling 15) |

`eponym-identity` (509 lines) goes to dramatis under W3 (ruling 34), its
`SubjectId`, body revisions and facets the sim's provenance noun, and its
control pointer this plan's §3. `eponym-sortie` (1,082 lines) is S3's
receipt and retires when the joined session adopts its evidence, as the
functional loops plan already says. What stays Eponym's own:
`eponym-client` (11,951 lines), the genet document host, input, the camera
and its zoom, the scene producer, the panels and journal, and the
legibility surface.

## 5. Phases and done-conditions

Proposed, not opened. Done-conditions are draws, never fixtures (ruling
15).

- **E0, the profile ruled.** Done when §6's decisions are taken; everything
  else in §1 to §4 rests on rulings already made.
- **E1, the contract's Eponym side.** Done when `src/eponym/` exists in
  `shared/isocosm-overlay` (ruling 197) with actuation, asks, tellings, the
  player's acts, the death checkpoint, creative's tag-in and the handoff's
  vocabulary as §3 and §6 decision 1 settle them; every type round-trips
  through bytes and the crate still depends on nothing sim-internal (D18);
  and each of today's seventeen game intents, two world intents and four
  control intents is mapped as §3 splits them.
- **E2, absorption by family.** One sub-phase per family of §4, in the order
  §6 decision 4 rules. Each is done when the family runs in Isocosm under
  its process definitions, conserves its accounts and replays identically,
  its Eponym tests are ported or replaced by draws, every moved file is
  within the 600-line ceiling, and `eponym-world` or `eponym-social` no
  longer owns it. Lands under the sim plan's S1 to S3, the knowledge family
  under S3.
- **E3, driving on Isocosm.** Where it is built is §6 decision 4. Done when a
  played sophont is driven on Isocosm, its acts resolved by what its body
  affords; an ask is refused about the work, refused about the asker and
  countered, each with its premises pointable; a note written is later found
  by another; a fight is sized up and ends there, and another escalates and
  ends by a yield or a side spent; a death is followed by a succession the
  player chose among bonded companions; survival shows only what the
  creature knows; and a seeded run replays to the same hash.
- **E4, the played loop.** Done when, from a seed nobody chose and a start
  the player picked, the headed host lives one named life end to end on
  Isocosm: a home made or joined, asks agreed and refused, a fight, notes
  written and found, a death and a succession, then the successor's life
  continuing in the same history; survival and creative modes both work;
  the receipts replay to the same hash; and the fungible agree in
  distribution within the world's stated tolerance (ruling 113). This is
  W5's done-condition for Eponym.

E1 and E2 run side by side; E3 needs E2's bodies, places, lives and asks;
E4 needs all of E2 and E3.

## 6. Decisions for Mark

None taken. Each is a fork this plan found and did not settle.

1. **The handoff and the solver.** Two halves. (a) Eponym's blows: the
   foreground strike system keeps resolving them geometrically and hands
   outcomes back through the handoff, calibrated to the sim's fight under
   ruling 114; or the sim's competing instance runs at foreground fidelity
   with Eponym supplying actuation alone, its handoff uninhabited as
   Mesocosm's is. (b) The fixed-step motion and contact solver: the game
   side, over the stack's conatus, with the accepted transition per tick
   the intent; or the sim's own process, fed the input frames. The record's
   §3.8 says the sim never resolves a single blow and §5.2 point 1 says the
   contract is batches per tick, never a call per entity; the two halves
   decide what a batch carries.
2. **The start in time.** Eponym needs society (ruling 3): how much deep
   time a draw carries before the first life, and where the player may begin
   in it: from the world's habitability for the creature onward, as ruling
   179 gives Mesocosm, or only where settlements already stand.
3. **The first life and the outsider.** How the first life begins: a newly
   generated outsider arriving, a birth into one of the world's lineages, or
   a choice among the drawn world's denizens; and at a death with no
   bonded companion (ruling 186), the execution plan's F8 fallback, a
   generated outsider so lack of allies never ends the world, or another
   rule; and whether an answer that lets the line end exists at all, which
   Mesocosm's contract has none of.
4. **Absorption, its order, and where driving is built.** Whether Isocosm
   absorbs `eponym-world`'s and `eponym-social`'s simulation as ruling 192
   had it absorb `mesocosm-core`; the order, proposed as bodies, places,
   lives and rounds, standing and asks, knowledge, glyphs, then fights and
   technique; and whether driving is built only on Isocosm, as ruling 194
   had Mesocosm's directing, or first over today's `Session`.
5. **Posing.** The record's §3.4 flags posing, to intimidate, persuade or
   deceive, as a reading of ruling 117 and not a ruling. Eponym is the game
   that needs it in play. Rule it, so E1 carries a posing intent, or leave it
   out of E1.
6. **Co-op in E4.** Each player their own creature is ruled (ruling 185).
   Whether E4's done-condition includes two peers over the session lane, or
   single-player first as the founding plan parked real-time co-op netcode.
7. **Which overlay is W5's second.** Ruling 174 put Mesocosm first and
   nothing orders Eponym and the VTT; the same question sits in the VTT
   plan's §6.

## Findings

- **2026-09-25:** `eponym-world` counted at 11,508 lines, `eponym-social`
  2,747, `eponym-identity` 509, `eponym-sortie` 1,082 and `eponym-client`
  11,951 (`wc -l` over each `src` tree's `.rs` files); per-module counts in
  §4 are from the same pass, a directory module summed with its file.
- **2026-09-25:** `GameIntent` at `eponym-world/src/transitions.rs:59` holds
  the seventeen variants §3 names and `GameEvent` twenty; `WorldIntent` at
  `src/world.rs:33` holds `Carve` and `InheritSite`; `ControlIntent` at
  `eponym-identity/src/control.rs:24` holds `Begin`, `TagIn`, `TagOut` and
  `Succeed`, and `Control::played` is one subject by the shape of the type,
  which is ruling 152 already enforced in code.
- **2026-09-25:** the willingness rule's three gates
  (`eponym-social/src/willing.rs:7-23`) and `RulingKind`
  (`src/response.rs:156`: formed, performed, declined, outside terms,
  renegotiated, ended) are the vocabulary §3's asks row maps; an agreement
  "is still not a command" (`src/agreement.rs:14-17`).
- **2026-09-25:** `shared/isocosm-overlay`'s attention set already names
  Eponym's care as "Eponym's sophont and those it knows"
  (`src/attention.rs`), from the record's §3.4.1, so §1's ensemble row adds
  no type to the core.

## Progress

- 2026-09-25: plan drafted at Mark's word ("Overlay plans to RPG") from the
  record's §5.6, rulings 60, 130, 152 to 156, 185 to 187 and the contract's
  rulings, Eponym's plans and its code. E0 to E4 proposed; no lane open;
  §6's seven decisions await Mark.
