# The VTT's overlay: the W5 plan

**Date:** 2026-09-25

**Status, 2026-09-25:** plan; V0 to V4 proposed and not opened. Drafted at
Mark's word ("Overlay plans to RPG") in the RPG systems session, in parallel
with the Simulation design review session, from the
[wing design record](../mesocosm/design_docs/2026-09-18_wing_design_plan.md)'s
§5.7 (rulings 188 to 191), rulings 114 and 189, ruling 154, the contract's
rulings (197, 203 to 205, 210 to 213), the system-plugin crate and this
repository's plans. It mirrors the
[Mesocosm overlay plan](../mesocosm/design_docs/2026-09-25_mesocosm_overlay_plan.md),
the first game's. Every row cites the ruling or plan it rests on; a reading
of this plan's own is flagged as one. No lane runs until Mark opens it, and
§6's decisions are his before V0 closes.

**Owns:** the VTT's profile as a game over the Isocosm sim (the record's
§5); the VTT side of the overlay contract (ruling 154), a module beside
Mesocosm's in `shared/isocosm-overlay` (ruling 197), the handoff among it,
which this is the game most likely to fill; the played loop W5 names; and
the order in which `isometry-campaign`'s world moves into Isocosm for this
overlay's needs (§4). **Does not own:** the sim, whose schema, process
definitions, record and generator the
[sim plan](../mesocosm/design_docs/2026-09-22_sim_plan.md) owns, so every
move lands under its phases; the bench (W4), where calibration is checked;
the other overlays (the record's §5.5 and §5.6); the substrate's geometry and
turns, which stay this product's; the rulesets, which are packs (ruling 41,
the record's §5.1); or naming. **Consumes:** the record; the sim plan; the
contract crate's [README](../shared/isocosm-overlay/README.md);
[PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) for the pillars; the
[watchtower plan](2026-09-05_watchtower_plan.md), whose pack is the first
adventure-pack fixture; the
[protocol hardening plan](2026-08-08_protocol_hardening_plan.md), whose
`Intent -> Resolved` envelope is the handoff's shape at the table; the
[shared authority plan](2026-07-09_shared_authority_and_collaborative_building_plan.md)
for edit mode, its tiers and creative mode; the
[overmap presentation plan](2026-08-02_overmap_presentation_plan.md) for the
far view; the [board-on-isometer plan](2026-09-15_board_on_isometer_plan.md),
held, for the battlemap as a scene; and the
[vessel briefs](../mesocosm/design_docs/2026-08-18_vessel_briefs_and_presentation.md)
§2.

---

## 0. Done-conditions

From the record's §11, W5 is done for a game when it has:

1. a profile designed to the record's §5 (§1 here);
2. a core implementing the overlay contract (§3);
3. a played loop with receipts drawn from the generator (§2, phase V4).

Ruling 174 made Mesocosm the first overlay; whether the VTT or Eponym goes
second is not ruled (§6 decision 8).

## 1. The profile

Ruling 6's six parts.

| Part | The VTT | Rests on |
| --- | --- | --- |
| Domain | the character inside its polities: a denizen with a faction association, partisan, friendly, antagonistic, factional or unaligned, and the polities that set the stakes, sidequests, alignment, arcs, access to resources | 35, 36, 56, 200; the record's §5 |
| Ensemble | the campaign's cast: the table's characters and what the table authored, the denizens of the places the party reaches, its factions and polities; at a battlemap, the tokens present | 6, 71; the record's §3.4.1 |
| Mechanics | a ruleset's: checks as outcome bands with a margin and a twist, sheets as grants, the one ledger read as the sheet, GM moves as processes the ruleset registers; the substrate's geometry and turns, never a hit point; rulesets calibrate to the sim, an uncalibrated one plays in a debug or experimental mode with a warning, and a campaign may switch the sim off; death reversible by rules and magics as re-embodiment, summoning from the planes costly | 38, 41, 61, 62, 114, 188, 189; the record's §5.1; PROJECT_DESCRIPTION pillar 4 |
| Controls | the DM's edit mode: proposes, previews and commits; edits the world and plays any unclaimed entity; declares downtime with the players' consent; forces a pack the world does not meet; takes up, drops or reshapes a hook; the players': their characters' acts on the board, each directing only who they play, two able to direct one; pins of any pointable thing; the battlemap as the region examined up close | 104, 105, 152, 153, 156, 190, 191, 210, 212 |
| Perspective | the locked isometric 2D lens, 2:1 diamond tiles and sculpted elevation; scopes over the one world, world, region, area and battlemap, the VTT's own and never the sim's; players see what their characters know and the DM sees the truth, survival and creative split by role | 18, 74, 92; vessel briefs §2; the record's §5.7 readings; PROJECT_DESCRIPTION pillar 2 |
| Timescale | the turn, in the initiative modes a ruleset chooses; the campaign's time passes as the table plays; downtime runs the sim forward as deep time does, with the players' consent; between sessions only if the founder turned the unattended rate on | 6, 93, 104, 105; the record's §5.7 readings; vessel briefs §2 ("the rhyme") |

## 2. The played loop

1. **Founding a campaign.** From a drawn world, a seeded draw with the
   region its first edge (rulings 89, 124), or from the DM's own maps and
   packs, authored content filling or displacing generated content (ruling
   89), or with the sim off (ruling 188). A pack whose requirements the
   world lacks waits, dormant, or the GM forces it (ruling 190). Player
   characters are created contingent on groups even by absence (ruling
   36), members or outsiders like anyone, where they start being the DM's
   setup (the record's §5.7 readings).
2. **The scene.** At a battlemap, the region the view shows up close and so
   the region that runs in detail (ruling 212), the table plays by its
   ruleset: moves, stances, emotes, and adjudicated actions. An action
   resolves once, by the ruleset, and comes back through the handoff as an
   outcome in the sim's terms (rulings 114, 154). The DM commits facts and
   edits the world (ruling 156), reveals secrets (ruling 87) and plays any
   entity no player holds (ruling 156).
3. **Downtime.** The DM declares it and the players consent, since
   advancing the trunk past what another's foreground can bear is a
   proposal needing consent (rulings 104, 105); the sim runs forward as
   deep time does (ruling 93). Factions act from their members (ruling 63),
   so the tape-drawn faction turn the substrate runs today is the far-rung
   stand-in the record's §3.2.2 names, and the "meanwhile" the table reads
   is the record's events reaching its characters (rulings 5, 84). The sim
   offers the arcs it is running as suggested hooks (rulings 103, 191).
4. **Travel.** A party moves across the place graph (rulings 72, 205); what
   it knows of the overmap is what its characters know, arrived by reach and
   possibly wrong (rulings 84, 117), while the DM sees the truth.
5. **Death.** Final in the sim (ruling 61); the VTT's trick is reversal by
   rules, revivify, resurrection and wishes as re-embodiment bought by rules,
   and summoning back from the planes as a costly possibility (rulings 61,
   62).
6. **Sim off.** The plain tabletop as it plays today (ruling 188); with no
   background to agree with, calibration does not apply (the record's §5.7
   reading).

## 3. The core and the contract

The contract's shape is ruled (ruling 154): a game submits intents, the sim
returns events and a read-only view of each tick, and outcomes a game
settles come back through the handoff. The VTT is the game that fills the
handoff: the record's §3.8 has the foreground game resolve a blow by its own
rules, and `ActionResolved` (`crates/isonetry/src/protocol.rs:180`) already
carries a resolved action as one envelope every peer applies. Its module is
`shared/isocosm-overlay/src/vtt/`, beside `mesocosm/` (ruling 197),
changing nothing in the core.

| Direction | The VTT's side | Rests on |
| --- | --- | --- |
| In: table acts | an adjudicated action, actor, action key and target; a move on the board; a stance or an emote; batched per tick, never a call per token; a player's acts for the characters they play, two players able to share one, the DM for any unclaimed | 152, 153, 154; the record's §5.2 point 1; *which of these reach the sim is §6 decision 2* |
| In: assertions | a fact committed, a secret revealed, a world edit, a character created, a storylet's effects applied, a pack forced over generated content; the DM's edit mode is the world editor's | 89, 156, 184, 190 |
| In: time | downtime declared for a span, with the players' consent; the campaign's sim switched off or on | 104, 105, 188 |
| In: travel | a party's move to a place-graph node, and its pace | 72, 205 |
| In: hooks | a running arc taken up, dropped or reshaped | 103, 191 |
| In: attention changes | pin or unpin any pointable thing; examine the battlemap the view shows up close, the overmap staying a far view | 210, 212 |
| In: dev intents | none of its own: the DM's edit mode is play at the table (156); a debug or experimental campaign is a setting with a warning (189) | 156, 189 |
| Out: events | each participant's stream: the DM's the truth, a player's what their characters know; record entries reaching the characters, faction acts, and arcs offered as hooks | 84, 191, 204, 213 |
| Out: views | the examined site's near rung for the battlemap, projected as the game's grid; the overmap's far view read from the crowd | 12, 18, 212; *what the battlemap is under the sim is §6 decision 1* |
| Handoff | the ruleset's resolved action in the sim's terms: vigour drained and wounds to parts, a defeat or a death, a condition applied or cleared, a displacement within the site, an item transferred; passing the sim's invariants, the accounts conserved; calibrated under 114 or flagged under 189 | 114, 123, 154, 189 |

*Reading, not ruled:* `ActionResolved`'s fields, a request id, the rolls,
sheet deltas, beats, the defeated, the displaced and the conditions, are the
handoff's outcome with the dice and the beats left at the table: the sim
records the band, never the dice (the record's §5.1), and a beat is
representational by the protocol's own word.

**The events today.** `isonetry`'s replicated `GameEvent`
(`src/protocol.rs:299`) holds twenty-three variants, six of them the
substrate's `SessionEvent` map mutations and twelve the campaign's
`WorldEvent`. *Reading, not ruled, confirmed variant by variant in V1:* they
split three ways. `Map`, the turn order's `TurnAdd`, `TurnRemove`,
`TurnAdvance` and `TurnSetOrder`, `Rolled`, `SheetSet`, `Emoted`,
`StanceSet`, `MapStored`, `MapActivated` and `Generation` stay the table's,
geometry, turns, dice and sheets the sim never knows (the record's §3.8).
`ActionResolved` and `ConditionSet` are the handoff. `Fact`,
`CharacterCreated`, `ItemModifierRevealed` and `World`'s factions, places,
characters, routes, laws, history and storylets are assertions; `World`'s
`PartyMoved`, `PartyPaceSet`, `TransitionResolved` and `TravelResolved` are
travel; `NodeRevealed` is knowledge arriving (ruling 84); `InventorySet` and
`ItemTransfer` are exchange and possession (rulings 53, 94); `TimeAdvanced`
is downtime.

## 4. Absorption: `isometry-campaign` into Isocosm

The record's §2 test puts whatever runs with nobody playing in the sim, and
the substrate's own doctrine keeps geometry and turns here and rules in
plugins (`CLAUDE.md`). `isometry-campaign` is 4,234 lines of source and is
where this product simulates a world: factions that act between scenes,
places and routes, characters, laws, history, secrets. It is what moves.
Whether Isocosm absorbs it as ruling 192 had it absorb `mesocosm-core` is
§6 decision 5's larger half; every family that moves is decomposed under the
600-line ceiling as it goes, this repository's own rule (`CLAUDE.md`), split
along seams the code already has.

| Family | Modules, lines | Destination in Isocosm |
| --- | --- | --- |
| The campaign world | `world` 1,223 | factions and polities derived from their members (rulings 63 to 68); places and routes as the place graph (ruling 72); characters as denizens with a faction (rulings 36, 200); laws as world-scope rules (ruling 41); history as the record (the record's §3.4); storylets as a pack's requirements and role slots (rulings 89, 190); a party's known places as knowledge (ruling 84) and its stores as the ledger (ruling 38) |
| The faction turn | `faction` 459 | faction acts derived from members and the world's seed (rulings 15, 63); the entropy tape's far-rung stand-in retires when the sim is on, §6 decision 5 |
| Facts and items | `item` 311, `fact` 78 | notes and secrets (rulings 80, 87); items bearing their provenance, a hidden modifier a note revealed, an item of note a relic (rulings 143, 157) |
| Generator and packs | `generator` 491, `pack` 340 | the generator's declared space (sim plan §5, ruling 89) and packs as authored content (rulings 89, 190); cleromancy's selection stays host-local by its own decision record |
| Campaign maps | `map` 258 | the battlemap as a projection of a site's volume and its transitions as nesting steps (rulings 12, 18, 74); §6 decision 1 |
| Chronicle | `chronicle` 380 | arrivals from other games as crossings and descent (rulings 105, 126); `wing-formats` keeps the wire |
| Construction | `construction` 284 | world edits as asserted facts (the record's §1) |
| Store and proposals | `store` 200, `collaboration` 160 | the intent log and branches: a proposal's `Branch` mode is ruling 126's branch at campaign scale (the record's §5.2 point 2) |
| Crate root | `lib` 50 | dissolves |

What stays this product's: `isometry-core` (2,822 lines), the substrate's
maps, tiles, tokens, turn order, templates, dice, sheets, paths, line of
sight and narration, none of which the sim runs (the record's §3.8), its
`overmap` reading the place graph as the far view; `isometry-system`
(4,373), the rulesets and their packs (the record's §5.1); `isonetry`
(5,061), the session lane whose ordered log carries the contract's
envelopes between peers; `isometry-views` (14,890), `isometry-genet`
(9,253) and `isometry-graphshell` (573), presentation and host; and
`isometry-runtime` (1,308), whose plan retired on 2026-09-18 and whose
binding table survives one tier down as a stack adapter.

## 5. Phases and done-conditions

Proposed, not opened. Done-conditions are draws, never fixtures (ruling
15).

- **V0, the profile ruled.** Done when §6's decisions are taken; everything
  else in §1 to §4 rests on rulings already made.
- **V1, the contract's VTT side.** Done when `src/vtt/` exists in
  `shared/isocosm-overlay` (ruling 197) with the table's acts, assertions,
  time, travel, hooks and the handoff's outcome vocabulary as §3 and §6
  decisions 1 and 2 settle them; every type round-trips through bytes and
  the crate still depends on nothing sim-internal (D18); and each of today's
  twenty-three replicated events is mapped as §3 splits them.
- **V2, the campaign over a drawn world.** Done when a campaign founded from
  a seeded draw takes its places, routes, factions, characters, laws and
  history from the sim; an authored pack displaces generated content where
  the world meets its requirements and waits where it does not, until the
  GM forces it (rulings 89, 190); a declared downtime advances the sim with
  the players' consent and the factions act from their members, the
  entropy-tape faction turn no longer running when the sim is on; the arcs
  the sim runs reach the DM as hooks (ruling 191); a party's overmap is what
  its characters know (ruling 84); and the campaign replays to the same
  hash. Lands under the sim plan's S1, S3 and S5.
- **V3, the table over the sim.** Done when an adjudicated action at a
  battlemap returns through the handoff and passes the sim's invariants
  (ruling 154); the bench check of ruling 114 runs over seeded draws for the
  first-party ruleset, a fight with no player choices resolving in the same
  distribution as the sim's own (rulings 115, 116, 123, 221 to 223) within
  the world's tolerance (ruling 209), and a ruleset that fails it plays only
  in a debug or experimental mode with a warning (ruling 189); a campaign
  with the sim off plays as today (ruling 188); and a death at the table is
  reversed by the rules as re-embodiment (rulings 61, 62).
- **V4, the played loop.** Done when, from a seed nobody chose, a campaign
  hosted by one DM and joined by one player over the session lane plays
  three sessions with downtime between them on Isocosm: a battlemap fight
  resolved by the ruleset, a downtime in which factions act and the DM takes
  up a hook, a travel that reveals what the characters know, a death
  reversed by rules, and facts the DM committed; players see what their
  characters know and the DM the truth; the receipts replay on both peers to
  the same hash; and the fungible agree in distribution within the world's
  stated tolerance (ruling 113). This is W5's done-condition for the VTT.

V1 and V2 run side by side; V3 needs V1 and V2's world; V4 needs all three.

## 6. Decisions for Mark

None taken. Each is a fork this plan found and did not settle.

1. **The battlemap under the sim.** When the sim is on, a battlemap is the
   DM's authored map asserted over the site's volume (ruling 89), a
   projection of the generated volume in the game's grid (rulings 12, 18),
   or both, the map an edit over what was generated. The board-on-isometer
   plan's held scene board is where either is drawn.
2. **The table's grain.** Which board events reach the sim: only outcomes,
   assertions and travel, with a token's position on the board the table's
   own; or moves within a battlemap too, as per-tick batches, so the
   character's place in the site follows the token. §5.2 point 1 sets the
   grain coarse either way.
3. **Sharing a character.** Two players directing one character (ruling
   153) and the DM playing any unclaimed one (ruling 156): both on by
   default at the table, or the DM's campaign setting.
4. **Consent to downtime.** The players' consent (rulings 63, 104, 105)
   gathered how at the table: each player's yes, a majority as the shared
   authority plan's Tier 3 table actions, or the DM's declaration with a
   veto.
5. **The faction turn's fate, and absorption.** The tape-drawn faction turn
   retires at V2 for the sim's factions, or stays as the sim-off campaign's
   downtime; and whether Isocosm absorbs `isometry-campaign`'s world as
   ruling 192 had it absorb `mesocosm-core`, in the order proposed in §4.
6. **The first calibration.** Which ruleset the bench calibrates first, the
   5e SRD in the repository or Pathfinder 2e's skeleton, against the sim's
   fight at the first tolerance (ruling 209); and what a debug or
   experimental campaign shows the table (ruling 189).
7. **A sim-off campaign's record.** Whether a campaign played with the sim
   off still writes its facts as notes, so the sim can be switched on later
   over the same history, or switching it on founds a world from that point
   (ruling 126).
8. **Which overlay is W5's second.** Ruling 174 put Mesocosm first and
   nothing orders the VTT and Eponym; the same question sits in the Eponym
   plan's §6.

## Findings

- **2026-09-25:** `isometry-campaign` counted at 4,234 lines,
  `isometry-core` 2,822, `isometry-system` 4,373, `isonetry` 5,061,
  `isometry-views` 14,890, `isometry-genet` 9,253, `isometry-runtime` 1,308,
  `isometry-graphshell` 573 and `isocosm-vtt` 16 (`wc -l` over each `src`
  tree's `.rs` files); per-module counts in §4 are from the same pass, the
  `world` directory summed with its file.
- **2026-09-25:** `GameEvent` at `crates/isonetry/src/protocol.rs:299`
  holds the twenty-three variants §3 names; `SessionEvent`
  (`crates/isometry-core/src/event.rs:14`) six; the campaign's `WorldEvent`
  (`crates/isometry-campaign/src/world/draft.rs`) twelve; `FactionVerb`
  (`src/faction.rs`) five, drawn from a host entropy tape with moves banked
  by world time, the turn "pure and replayable" by its own doc.
- **2026-09-25:** `ActionResolved` (`protocol.rs:180`) carries a request
  id, actor, target, action key, the attack and damage rolls, sheet deltas,
  beats, the defeated, the displaced and conditions; `resolve_action`
  (`crates/isometry-system/src/sys/system_actions.rs`) is "the only path
  from an intent to a change in game state", so the handoff has a resolver
  already.
- **2026-09-25:** `CampaignWorld` (`src/world/types.rs`) keeps
  `party_known`, the overmap a party has discovered, "fog at overmap scale,
  with explored memory", which §4 maps to knowledge by reach (ruling 84);
  `CampaignProposalMode::Branch` (`src/collaboration.rs`) forks a campaign
  revision under a branch name.
- **2026-09-25:** the watchtower pack's README states "The generator
  proposes a campaign; it never resolves an encounter", the posture ruling
  190 keeps for packs; its five inhabitants use the `5e-srd` sheet
  vocabulary, the ruleset §6 decision 6 would calibrate first.

## Progress

- 2026-09-25: plan drafted at Mark's word ("Overlay plans to RPG") from the
  record's §5.7, rulings 114, 154, 188 to 191 and the contract's rulings,
  the system-plugin crate and this repository's plans. V0 to V4 proposed;
  no lane open; §6's eight decisions await Mark.
