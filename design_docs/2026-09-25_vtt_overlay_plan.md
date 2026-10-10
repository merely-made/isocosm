# The VTT's overlay: the W5 plan

**Date:** 2026-09-25

**2026-10-03, rulings 538 to 541:** the VTT plays recognized editions
faithfully as versioned rulesets, keeping the GM's decision points and
recording their rulings, with a toolkit grown from what they repeat; a
world chooses whether the sim or its ruleset leads, so 114's calibration
holds only where the sim leads; and an encounter across systems runs
under a named hybrid profile. Ars Magica 5th Edition joins 5e and
Pathfinder 2e (522). In a ruleset-led world an edition's own state
persists beside the body with declared relations (542), and its
procedures run unattended under declared decision policies (543).

**Status, 2026-10-03:** Rulings 538 to 541; V0 and V1 done. *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status, 2026-09-26:** plan; V0 done 2026-09-26, its eight decisions ruled
(231 and 243 to 250); V1 done 2026-09-26, opened by ruling 253 as a
contract module only; V2 to V4 proposed and not opened, waiting on
Mesocosm's M3 (ruling 231). Drafted at
Mark's word ("Overlay plans to RPG") in the RPG systems session, in parallel
with the Simulation design review session, from the
[wing design record](../mesocosm/design_docs/2026-09-18_wing_design_plan.md)'s
§5.7 (rulings 188 to 191), rulings 114 and 189, ruling 154, the contract's
rulings (197, 203 to 205, 210 to 213), the system-plugin crate and this
repository's plans. It mirrors the
[Mesocosm overlay plan](../mesocosm/design_docs/2026-09-25_mesocosm_overlay_plan.md),
the first game's. Every row cites the ruling or plan it rests on; a reading
of this plan's own is flagged as one. No lane runs until Mark opens it, and
§6's eight decisions are taken: ruling 231 placing this overlay side by side
with Eponym's after Mesocosm's M3, rulings 243 and 244 settling the battlemap
under the sim, projected from the generated volume with the DM's map an edit
over it, and moves within it reaching the sim as per-tick batches, rulings
245 to 248 sharing a character on by default, every player's yes to
downtime, the faction turn retiring at V2 with absorption ruled, and a
sim-off campaign writing its facts as notes, and rulings 249 and 250
Pathfinder 2e calibrated first and an uncalibrated campaign warning at open
with its receipts marked.

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
[watchtower plan](archive_docs/2026-10-10/2026-09-05_watchtower_plan.md), whose pack is the first
adventure-pack fixture; the
[protocol hardening plan](archive_docs/2026-10-10/2026-08-08_protocol_hardening_plan.md), whose
`Intent -> Resolved` envelope is the handoff's shape at the table; the
[shared authority plan](2026-07-09_shared_authority_and_collaborative_building_plan.md)
for edit mode, its tiers and creative mode; the
[overmap presentation plan](archive_docs/2026-10-10/2026-08-02_overmap_presentation_plan.md) for the
far view; the [board-on-isometer plan](2026-09-15_board_on_isometer_plan.md),
held, for the battlemap as a scene; and the
[vessel briefs](../mesocosm/design_docs/2026-08-18_vessel_briefs_and_presentation.md)
§2.

---

## 0. Done-conditions

W5's done-conditions are the record's
[§11](../mesocosm/design_docs/2026-09-18_wing_design_plan.md#11-phases-and-done-conditions) (ruling 617). Here, for the VTT, they are
met by §1 (the profile), §3 (the core and the contract) and §2's phase V4
(the played loop).

The order is rulings 174 and 231.

## 1. The profile

Ruling 6's six parts.

| Part | The VTT | Rests on |
| --- | --- | --- |
| Domain | the character inside its polities: a denizen with a faction association, partisan, friendly, antagonistic, factional or unaligned, and the polities that set the stakes, sidequests, alignment, arcs, access to resources | 35, 36, 56, 200; the record's §5 |
| Ensemble | the campaign's cast: the table's characters and what the table authored, the denizens of the places the party reaches, its factions and polities; at a battlemap, the tokens present | 6, 71; the record's §3.4.1 |
| Mechanics | a ruleset's: checks as outcome bands with a margin and a twist, sheets as grants, the one ledger read as the sheet, GM moves as processes the ruleset registers; the substrate's geometry and turns, never a hit point; rulesets calibrate to the sim, Pathfinder 2e first, an uncalibrated one plays in a debug or experimental mode that warns when the campaign opens and marks every receipt and save, and a campaign may switch the sim off; death reversible by rules and magics as re-embodiment, summoning from the planes costly | 38, 41, 61, 62, 114, 188, 189, 249, 250; the record's §5.1; PROJECT_DESCRIPTION pillar 4 |
| Controls | the DM's edit mode: proposes, previews and commits; edits the world and plays any unclaimed entity, on by default at every table; declares downtime with every player's yes; forces a pack the world does not meet; takes up, drops or reshapes a hook; the players': their characters' acts on the board, each directing only who they play, two able to direct one, on by default; pins of any pointable thing; the battlemap as the region examined up close | 104, 105, 152, 153, 156, 190, 191, 210, 212, 245, 246 |
| Perspective | the locked isometric 2D lens, 2:1 diamond tiles and sculpted elevation; scopes over the one world, world, region, area and battlemap, the VTT's own and never the sim's; players see what their characters know and the DM sees the truth, survival and creative split by role. *2026-09-28:* the wing's default view is isometric with quarter turns, precluding no other (382); the tabletop `CLAUDE.md`'s locked-lens don't awaits amendment on Mark's word, amended the same day (386), and the default's pitch is the shipped 2:1 dimetric, 30° (387) | 18, 74, 92, 382, 386, 387; vessel briefs §2; the record's §5.7 readings; PROJECT_DESCRIPTION pillar 2 |
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
   the region that runs in detail (ruling 212), projected from the site's
   generated volume in the game's grid with the DM's map an edit over it
   (ruling 243), the table plays by its ruleset: moves, stances, emotes, and
   adjudicated actions. Moves within the battlemap reach the sim as per-tick
   batches, so a character's place in the site follows its token (ruling
   244). An action
   resolves once, by the ruleset, and comes back through the handoff as an
   outcome in the sim's terms (rulings 114, 154). The DM commits facts and
   edits the world (ruling 156), reveals secrets (ruling 87) and plays any
   entity no player holds (ruling 156).
3. **Downtime.** The DM declares it and every player says yes (ruling
   246), since advancing the trunk past what another's foreground can bear
   is a proposal needing consent (rulings 104, 105); the sim runs forward
   as deep time does (ruling 93). Factions act from their members (ruling
   63), so the tape-drawn faction turn the substrate runs today is the
   far-rung stand-in the record's §3.2.2 names, retired at V2 once the sim's
   factions run (ruling 247), and the "meanwhile" the table reads is the
   record's events reaching its characters (rulings 5, 84). The sim
   offers the arcs it is running as suggested hooks (rulings 103, 191).
4. **Travel.** A party moves across the place graph (rulings 72, 205); what
   it knows of the overmap is what its characters know, arrived by reach and
   possibly wrong (rulings 84, 117), while the DM sees the truth.
5. **Death.** Final in the sim (ruling 61); the VTT's trick is reversal by
   rules, revivify, resurrection and wishes as re-embodiment bought by rules,
   and summoning back from the planes as a costly possibility (rulings 61,
   62).
6. **Sim off.** The plain tabletop as it plays today (ruling 188), its
   downtime the table's own (a reading recorded with ruling 247); with no
   background to agree with, calibration does not apply (the record's §5.7
   reading); its facts are still written as notes, so the sim can be
   switched on later over the same history (ruling 248).

## 3. The core and the contract

The contract's shape is ruling 154's. The VTT is the game that fills the
handoff: the record's §3.8 has the foreground game resolve a blow by its own
rules, and `ActionResolved` (`crates/isonetry/src/protocol.rs:180`) already
carries a resolved action as one envelope every peer applies. Its module is
`shared/isocosm-overlay/src/vtt/`, beside `mesocosm/` (ruling 197),
changing nothing in the core.

| Direction | The VTT's side | Rests on |
| --- | --- | --- |
| In: table acts | an adjudicated action, actor, action key and target; a move within the battlemap, reaching the sim as a per-tick batch so the character's place in the site follows its token; batched per tick, never a call per token; a player's acts for the characters they play, two players able to share one and the DM playing any unclaimed, both on by default at every table; stances and emotes stay the table's (*a reading, from the events split below*) | 152, 153, 154, 244, 245; the record's §5.2 point 1 |
| In: assertions | a fact committed, a secret revealed, a world edit, a character created, a storylet's effects applied, a pack forced over generated content; the DM's edit mode is the world editor's | 89, 156, 184, 190 |
| In: time | downtime declared for a span, with every player's yes; the campaign's sim switched off or on, a sim-off campaign still writing its facts as notes so the sim can be switched on later over the same history | 104, 105, 188, 246, 248 |
| In: travel | a party's move to a place-graph node, and its pace | 72, 205 |
| In: hooks | a running arc taken up, dropped or reshaped | 103, 191 |
| In: attention changes | pin or unpin any pointable thing; examine the battlemap the view shows up close, the overmap staying a far view | 210, 212 |
| In: dev intents | none of its own: the DM's edit mode is play at the table (156); a debug or experimental campaign is a setting that warns when it opens and marks every receipt and save uncalibrated, with no standing banner (189, 250) | 156, 189, 250 |
| Out: events | each participant's stream: the DM's the truth, a player's what their characters know; record entries reaching the characters, faction acts, and arcs offered as hooks | 84, 191, 204, 213 |
| Out: views | the examined site's near rung for the battlemap, projected from the generated volume in the game's grid, with the DM's map an edit over it; the overmap's far view read from the crowd | 12, 18, 89, 212, 243 |
| Handoff | the ruleset's resolved action in the sim's terms: vigour drained and wounds to parts, a defeat or a death, a condition applied or cleared, a displacement within the site, an item transferred; passing the sim's invariants, the accounts conserved; calibrated under 114, Pathfinder 2e first, or marked uncalibrated under 189 and 250 | 114, 123, 154, 189, 249, 250 |

*Reading, not ruled:* `ActionResolved`'s fields, a request id, the rolls,
sheet deltas, beats, the defeated, the displaced and the conditions, are the
handoff's outcome with the dice and the beats left at the table: the sim
records the band, never the dice (the record's §5.1), and a beat is
representational by the protocol's own word.

**The events today.** `isonetry`'s replicated `GameEvent`
(`src/protocol.rs:299`) holds twenty-three variants, six of them the
substrate's `SessionEvent` map mutations and twelve the campaign's
`WorldEvent`. *Reading, not ruled, confirmed variant by variant in V1:* they
split as follows. `Map`'s token moves reach the sim as per-tick batches
(ruling 244) and its tile and elevation edits are the DM's map as an edit
over the generated volume (ruling 243); the turn order's `TurnAdd`,
`TurnRemove`, `TurnAdvance` and `TurnSetOrder`, `Rolled`, `SheetSet`,
`Emoted`, `StanceSet`, `MapStored`, `MapActivated` and `Generation` stay
the table's, turns, dice, sheets and beats the sim never knows (the
record's §3.8).
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
places and routes, characters, laws, history, secrets. It is what moves,
and ruling 247 has Isocosm absorb it as ruling 192 had it absorb
`mesocosm-core`, in the order below; every family that moves is decomposed
under the 600-line ceiling as it goes, this repository's own rule
(`CLAUDE.md`), split along seams the code already has.

| Family | Modules, lines | Destination in Isocosm |
| --- | --- | --- |
| The campaign world | `world` 1,223 | factions and polities derived from their members (rulings 63 to 68); places and routes as the place graph (ruling 72); characters as denizens with a faction (rulings 36, 200); laws as world-scope rules (ruling 41); history as the record (the record's §3.4); storylets as a pack's requirements and role slots (rulings 89, 190); a party's known places as knowledge (ruling 84) and its stores as the ledger (ruling 38) |
| The faction turn | `faction` 459 | faction acts derived from members and the world's seed (rulings 15, 63); the entropy tape's far-rung stand-in retires entirely at V2, once the sim's factions run (ruling 247) |
| Facts and items | `item` 311, `fact` 78 | notes and secrets (rulings 80, 87); items bearing their provenance, a hidden modifier a note revealed, an item of note a relic (rulings 143, 157) |
| Generator and packs | `generator` 491, `pack` 340 | the generator's declared space (sim plan §5, ruling 89) and packs as authored content (rulings 89, 190); cleromancy's selection stays host-local by its own decision record |
| Campaign maps | `map` 258 | the battlemap as a projection of a site's generated volume with the DM's map an edit over it (rulings 12, 18, 89, 243), and its transitions as nesting steps (ruling 74) |
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
binding table survives one tier down as a stack adapter. *(2026-09-27: the
crate retired, ruling 299, and neither its binding table nor its event
mirror had reached the stack; ruling 346 documents one binding shape in
conatus's docs, built when Mark says.)*

## 5. Phases and done-conditions

Proposed, not opened. Done-conditions are draws, never fixtures (ruling
15).

- **V0, the profile ruled.** Done when §6's decisions are taken; everything
  else in §1 to §4 rests on rulings already made. **Done 2026-09-26**
  (rulings 231 and 243 to 250).
- **V1, the contract's VTT side.** Done when `src/vtt/` exists in
  `shared/isocosm-overlay` (ruling 197) with the table's acts, assertions,
  time, travel, hooks and the handoff's outcome vocabulary as §3 has them
  under rulings 243 and 244; every type round-trips through bytes and
  the crate still depends on nothing sim-internal (D18); and each of today's
  twenty-three replicated events is mapped as §3 splits them. **Done
  2026-09-26** (ruling 253): `src/vtt/` landed on main at 1996ecc with ten
  round-trip tests and the README's mapping table, and the shapes it shared
  with Eponym's module, a voxel point, an act key and harm, were lifted to
  the crate root at 269ffc5 at Mark's word.
- **V2, the campaign over a drawn world.** Done when a campaign founded from
  a seeded draw takes its places, routes, factions, characters, laws and
  history from the sim; an authored pack displaces generated content where
  the world meets its requirements and waits where it does not, until the
  GM forces it (rulings 89, 190); a declared downtime advances the sim with
  every player's yes (ruling 246) and the factions act from their members,
  the entropy-tape faction turn retired (ruling 247); the arcs
  the sim runs reach the DM as hooks (ruling 191); a party's overmap is what
  its characters know (ruling 84); and the campaign replays to the same
  hash. Lands under the sim plan's S1, S3 and S5.
- **V3, the table over the sim.** Done when an adjudicated action at a
  battlemap returns through the handoff and passes the sim's invariants
  (ruling 154); the bench check of ruling 114 runs over seeded draws for Pathfinder
  2e's skeleton first (ruling 249), a fight with no player choices resolving
  in the same distribution as the sim's own (rulings 115, 116, 123, 221 to
  223) within the world's tolerance (ruling 209), and a ruleset that fails
  it plays only in a debug or experimental mode that warns when the campaign
  opens and marks every receipt and save uncalibrated (rulings 189, 250); a campaign
  with the sim off plays as today, its downtime the table's own, and still
  writes its facts as notes so the sim can be switched on later over the
  same history (rulings 188, 248); and a death at the table is
  reversed by the rules as re-embodiment (rulings 61, 62).
- **V4, the played loop.** Done when, from a seed nobody chose, a campaign
  hosted by one DM and joined by one player over the session lane plays
  three sessions with downtime between them on Isocosm: a battlemap fight
  resolved by the ruleset, a downtime every player agreed to, in which factions act and the DM
  takes up a hook, a travel that reveals what the characters know, a death
  reversed by rules, and facts the DM committed; players see what their
  characters know and the DM the truth; the receipts replay on both peers to
  the same hash; and the fungible agree in distribution within the world's
  stated tolerance (ruling 113). This is W5's done-condition for the VTT.

V1 and V2 run side by side; V3 needs V1 and V2's world; V4 needs all three.

### 5.1 What V2 carries from the folded plans (2026-10-10, ruling 801)

Ruling 801 folded the protocol hardening plan's H2 and the overmap
presentation plan into V2; both are archived at `archive_docs/2026-10-10/`
with their findings. Travel over the sim is a party's move to a place-graph
node (72, 205), so the travel resolver and what the overmap shows are both
V2's.

- **Overmap travel resolves once (H2).** Overmap travel is still
  `GameEvent::TravelResolved`, an inline-struct variant whose clock,
  encounter and exhaustion consequences peers apply as they always did
  (`crates/isonetry/src/protocol.rs`). Done when an overmap journey is one
  explicit `Resolved` naming every consequence; split-party clocks reconcile
  from the payload, never from peer derivation; a late joiner rebuilds the
  same state from the log alone; and a headed two-peer receipt exists. H0
  and H1's laws stand: no peer derives a consequence, and version
  negotiation refuses, never degrades. The wire is at `PROTOCOL_VERSION` 4
  with ALPN `isometry/session/v4`, and the after-pass's bump for 768's
  `WorldEvent` reshaping is the next break; H2 moves it again. Still open
  from H0: a peer that never sends `Hello` is not version-gated and can
  push `Rolled`.
- **The overmap's readings.** What the overmap shows: territory around each
  site the party knows, tinted by meaning the VTT owns (faction control
  where a claim exists, biome otherwise), with a generated backdrop and
  cells that select their site and answer hover with its facts; never a
  cell for a site the party does not know. The watchtower's atlas (W11 to
  W13, 2026-09-09) built a fixed region backdrop and clickable polygon
  areas over the campaign's region map; V2 changes their source to the
  sim's place graph and reach field. Done when the sites and their
  discovery come from the sim through the contract, never from authored
  positions or `party_known`; the campaign shown is a draw under a seed
  nobody chose; and one capture shows the overmap from a past standpoint
  differing from the present on at least one site's version (the reach
  field's arrival entries: what was believed then, what the table knows now,
  what was retconned, which map version a character held). The drawing
  itself moves to the presentation plan's L10: the overmap becomes a second
  camera held at R4 and the canvas renderer retires (437). Still Mark's:
  faction against biome tint where both exist; whether a known cell's
  undiscovered edges clip hard or fade; and whether baked map thumbnails
  are worth a backdrop.

The whole proceeds after Mesocosm's M3, side by side with Eponym's plan
(ruling 231).

## 6. Decisions for Mark

All eight taken: the eighth by ruling 231 on 2026-09-25; the first to
seventh by rulings 243 to 250 on 2026-09-26. V0 is done on paper; opening
V1 is Mark's.

1. **The battlemap under the sim.** When the sim is on, a battlemap is the
   DM's authored map asserted over the site's volume (ruling 89), a
   projection of the generated volume in the game's grid (rulings 12, 18),
   or both, the map an edit over what was generated. The board-on-isometer
   plan's held scene board is where either is drawn. **Ruled 243
   (2026-09-26): "Both: an edit."** With the sim on, a battlemap is
   projected from the generated volume in the game's grid (rulings 12, 18),
   with the DM's map as an edit over it (ruling 89).
2. **The table's grain.** Which board events reach the sim: only outcomes,
   assertions and travel, with a token's position on the board the table's
   own; or moves within a battlemap too, as per-tick batches, so the
   character's place in the site follows the token. §5.2 point 1 sets the
   grain coarse either way. **Ruled 244 (2026-09-26): "Moves too."** Moves
   within a battlemap reach the sim as per-tick batches, so a character's
   place in the site follows its token; the contract stays coarse, batches
   per tick and never a call per token.
3. **Sharing a character.** Two players directing one character (ruling
   153) and the DM playing any unclaimed one (ruling 156): both on by
   default at the table, or the DM's campaign setting. **Ruled 245
   (2026-09-26): "On by default."** Both are on at every table unless
   turned off.
4. **Consent to downtime.** The players' consent (rulings 63, 104, 105)
   gathered how at the table: each player's yes, a majority as the shared
   authority plan's Tier 3 table actions, or the DM's declaration with a
   veto. **Ruled 246 (2026-09-26): "Each player's yes."** Downtime needs
   every player's consent.
5. **The faction turn's fate, and absorption.** The tape-drawn faction turn
   retires at V2 for the sim's factions, or stays as the sim-off campaign's
   downtime; and whether Isocosm absorbs `isometry-campaign`'s world as
   ruling 192 had it absorb `mesocosm-core`, in the order proposed in §4.
   **Ruled 247 (2026-09-26): "Retires at V2."** Isocosm absorbs
   `isometry-campaign`'s world in §4's order, and the tape-drawn faction
   turn retires entirely once the sim's factions run. *Reading recorded with
   it, not ruled:* a sim-off campaign's downtime is then the table's own, as
   the plain tabletop plays today (ruling 188).
6. **The first calibration.** Which ruleset the bench calibrates first, the
   5e SRD in the repository or Pathfinder 2e's skeleton, against the sim's
   fight at the first tolerance (ruling 209); and what a debug or
   experimental campaign shows the table (ruling 189). **Ruled 249 and 250
   (2026-09-26): "Pathfinder 2e first" and "Warn at open, mark
   receipts."** The bench calibrates Pathfinder 2e's skeleton first; an
   uncalibrated campaign warns when it opens and marks every receipt and
   save uncalibrated, with no standing banner.
7. **A sim-off campaign's record.** Whether a campaign played with the sim
   off still writes its facts as notes, so the sim can be switched on later
   over the same history, or switching it on founds a world from that point
   (ruling 126). **Ruled 248 (2026-09-26): "Writes notes."** A campaign
   played with the sim off still writes its facts as notes, so the sim can
   be switched on later over the same history.
8. **Which overlay is W5's second.** Ruling 174 put Mesocosm first and
   nothing ordered the VTT and Eponym; the same question sat in the Eponym
   plan's §6. **Ruled 231 (2026-09-25): "Side by side."** Both proceed
   after Mesocosm's M3, each on its own plan.

## 7. Carried from archived plans (2026-10-10)

Ruling 793 archived the VTT's done plans to `archive_docs/2026-10-10/`;
what each left open lands here, so it has a live owner.

- **The design height, from the genet host migration's Z5.** The fit
  figure is 820 logical pixels (`DESIGN_SIZE` in
  `crates/isometry-genet`), not 1040: the side panel diet made the panel
  fit 820, and `the_side_panel_fits_the_design_height`
  (`crates/isometry-genet/src/host_zoom.rs`) asserts its last row at or
  above 800. Mere's ruling S60 had sent the 820-versus-1040 question here
  as Mark's; the diet answered it by fitting the panel, so nothing is open
  on the figure itself.
- **Reachability at the smallest supported display,** the side panel
  diet's W1 restatement of its target: every control reachable at the
  smallest display the VTT supports. Open; no smallest display is named
  yet.
- **From the watchtower plan:** a reusable character library and Knot prose
  editing (W5's follow-ons); initializing parties other than the session
  party, and their tactical doorway updates (W9); independent per-party
  map views and party-membership editing (W7). Each waits on V2's campaign
  over a drawn world, since parties and their knowledge then come from the
  sim. The watchtower's five failing atlas and door tests are in the
  [after-pass plan](../mesocosm/design_docs/2026-10-10_after_pass_plan.md)'s
  A2.

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
- 2026-10-10: the campaign session carries its assertions (ruling 768; lane
  `lane-assert`, under 732's compile gate). `CampaignWorld` saves and
  replicates its asserted entries and folds factions, places, routes,
  characters, laws and history from them; `WorldEvent::Assert` replaces the
  six authored variants, and the packs author worlds as entries. Native
  `Command::Assert` takes every authored noun (769), so a native session can
  replay the same entries when the sim switches on (248). The legacy
  campaign tree is otherwise unmoved (3,943 lines), the faction turn marked
  for V2. Detail in the
  [families plan](../mesocosm/design_docs/2026-10-08_families_reexpression_plan.md)'s
  Progress.
- 2026-10-10: the record family's lane (`lane-record-2`, under 732's compile
  gate) built `Command::Assert` in native Isocosm (ruling 757): an authored
  faction asserts a polity with no members carrying its name, tags, claims,
  governance and focus (760), and an authored fact asserts a note, whether
  or not the sim runs (248). The campaign world is not yet a reading of
  native state: where a campaign's native world lives, and the native homes
  of places, characters, laws and history lines, went back to Mark as forks.
  `isocosm::legacy::campaign` is unchanged at 3,950 lines; the faction
  turn is marked to retire at V2 (247). Detail in the
  [families plan](../mesocosm/design_docs/2026-10-08_families_reexpression_plan.md)'s
  Progress.
- 2026-10-09: §4's absorption begun under 732's compile gate (lane
  `lane-campaign`; no tests, draws or receipts run). The last row went first,
  by 669 rather than §4's destination: the store and proposals (360 lines)
  are handed back to `isometry-campaign`. The rest stays in
  `isocosm::legacy::campaign` (3,950 lines), each piece waiting on its
  family: the record, bodies, places, native founding, or an assertion path
  native lacks. The faction turn retires at V2 (247), so the legacy copy
  cannot leave in full before then. The detail and the forks are in the
  [families plan](../mesocosm/design_docs/2026-10-08_families_reexpression_plan.md)'s
  Progress.
- 2026-09-26: V1 done. Ruling 253 opened E1 and V1 as contract modules
  only; `src/vtt/` was built in a worktree, reviewed and merged by the
  Simulation design review session (main 1996ecc), then the shapes shared
  with Eponym's module were lifted to the crate root at Mark's word
  (269ffc5). fmt, clippy and 43 tests clean. Everything from V2 on waits
  for Mesocosm's M3 (ruling 231).
- 2026-09-26: V0 done, its eight decisions ruled (rulings 231, 243 to 250).
- 2026-09-25: plan drafted at Mark's word ("Overlay plans to RPG") from the
  record's §5.7, rulings 114, 154, 188 to 191 and the contract's rulings,
  the system-plugin crate and this repository's plans. V0 to V4 proposed;
  no lane open; §6's eight decisions await Mark.
