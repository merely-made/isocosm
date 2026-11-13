# The VTT over the sim

**Status:** 2026-11-02: cut from `2026-10-28_vtt_overlay_plan.md`, which is archived whole in `archive_docs/2026-11-02/` (DOC_POLICY §8: its Findings and Progress had grown past its open work; the run evidence they held is appended to `archive_docs/receipts/2026-10-28_vtt_overlay_plan_receipts.md`). V0 (the profile ruled) and V1 (the contract's VTT side, `shared/cosmograph-overlay/src/vtt/`) are done. V2 is partly placed in code: native founding, journeys, items, disclosure and the session protocol exist, and since 2026-11-01 telling (case 1066), several bodies per participant (cases 1062, 1065) and the wait ladder (cases 1072, 1073, 1075). The sim-off `CampaignWorld` reading with `party_known`, the inline `TravelResolved` event and the sim-on lifted-site battlemap remain. V2 as a whole, V3 and V4 are open. RS has built Pathfinder 2e creation, encounter mode, Stride and ordinary combat, and Ars Magica's first slice (the `ars5` pack). F10 to F12 are built except F10c and F11c.

**For:** the VTT as a game over the Cosmograph sim: V2 (a campaign over a drawn world), V3 (the table over the sim, calibrated handoff), V4 (the played loop, W5's done-condition for the VTT), the VTT's rulesets and their edition procedures (RS, F10c and F11c), and how a ruleset reads the sim's environment at a battlemap. The VTT side of the overlay contract (ruling 154) lives in `shared/cosmograph-overlay`.
Not here: the sim itself, which is `sim` (S1, S3, S5 are the phases V2 lands under); retiring `isocosm-campaign` and moving its world into Cosmograph, which is `campaign-retirement` R3; drawing the board and the DOM board's fate, which is `board-on-isometer`; the overmap's drawing as a second camera, which is `orthographic-voxel-presentation` L10; the session carrier and host handoff, which is `session-network`; pack terms and distribution, which is `content-packs`; Eponym's recalibration of its sortie scene, which is `eponym-overlay`.

## Decisions

None of its own: no `vtt-overlay D<n>` exists. The old plan's eight decisions (its section 6) were all taken, the eighth by ruling 231 (2026-09-25) and the first to seventh by rulings 243 to 250 (2026-09-26); they stay in the archived copy (`archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:12-69`), their rulings are cases in the design record, and the Constraints below carry what still binds.

## Constraints

- The campaign owns no second simulation authority; a campaign with the sim off plays as today, its downtime the table's own (rulings 188, 247). Case 798: V2 opened without waiting for Mesocosm's M3.
- No peer derives a consequence, and version negotiation refuses, never degrades (the H0 and H1 laws that stand under ruling 801).
- The contract stays coarse: per-tick batches, never a call per token (ruling 244). The VTT module in `shared/cosmograph-overlay` depends on nothing sim-internal (D18, ruling 197).
- Done-conditions are draws, never fixtures (case 15).
- Every file stays under the 600-line ceiling as families move.
- Ruling 312: the environment is the sim's field on places; a ruleset only reads it at a battlemap.

## Phases

### V2. The campaign over a drawn world
The whole of V2, assembled from V2.1 to V2.6 below. It lands under `sim` S1, S3 and S5.
- Done when: "Done when a campaign founded from a seeded draw takes its places, routes, factions, characters, laws and history from the sim; an authored pack displaces generated content where the world meets its requirements and waits where it does not, until the GM forces it (rulings 89, 190); a declared downtime advances the sim with every player's yes (ruling 246) and the factions act from their members, the entropy-tape faction turn retired (ruling 247); the arcs the sim runs reach the DM as hooks (ruling 191); a party's overmap is what its characters know (ruling 84); and the campaign replays to the same hash. Lands under the sim plan's S1, S3 and S5." Headed acceptance is also open.
- Needs: `sim` S1, S3, S5; participant and party binding (a Character's owner metadata and old table discovery do not grant it, and the native campaign refuses the old travel and map-reading UI path until it exists); native polity and time consent; live transport (`session-network`).
- Rulings: 89, 190, 191, 246, 247, 84; case 798.
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:82-87` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:252-261,327-328`)

### V2.1 Native campaign reading
Found a native `Session` from the asserted history and read `CampaignWorld` from it; admit drawn places, routes, factions, characters, laws and history, with packs found natively.
- Done when: "Done when sim-off assertions replay with the sim on to one native state hash and the campaign owns no second simulation authority."
- Needs: `sim` S1, S3, S5; the assertion boundary (Q2, landed)
- Rulings: 795, 799, 801, 247
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:89-94` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:16-22`)

### V2.2 Travel, knowing and polities
Run travel and the overmap conditions (folded under 801) through the contract; replace `party_known` by native reach; require unanimous downtime consent; retire the faction turn and `legacy/campaign` once native polities act.
- Done when: not stated beyond V2; owed.
- Needs: V2.1; sim factions running; V2.4
- Rulings: 801, 246, 247, 84, 72, 205
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:96-101` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:23-26`)

### V2.3 Battlemap and replay
Keep `MapTerrain` with the sim off. With it on, lift an isometer-space site (799), lower held map edits there, and prove one accepted assertion stream converges across connected peers. The lifted-site battlemap is drawn on the scene board; the scene lowering, selection and presentation gates for it are `board-on-isometer`'s. Open from the Q9 finding: native assertion admission can accept a held map edit naming a drawn `site:<id>` even when the sim-off `Asserted` fold cannot resolve it, so a native replay applies the caller's ordered entries through `Command::Assert` without that fold as a preliminary authority. Attachment (binding an existing site, or supplying native geometry for a new one) and native placement validation are open. Ruling 809: a Character's cell and optional participant owner persist in native table metadata beside the named site; native placement and control remain V2's work.
- Done when: not stated beyond V2; owed.
- Needs: V2.1; `board-on-isometer`
- Rulings: 799, 243, 244, 89, 769, 809
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:103-108` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:27-29,34-40,421-429`; `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:5-12`)

### V2.4 Overmap travel resolves once
`GameEvent::TravelResolved` is an inline-struct variant whose clock, encounter and exhaustion consequences peers apply as they always did. It needs its own protocol break and qualification beyond the current protocol version. (This was the protocol hardening plan's H2, folded into V2 by ruling 801; the H here is not `sim-host`'s.) Still open from H0: a peer that never sends `Hello` is not version-gated and can push `Rolled`. The late-join replay receipt from the adjudication residue is carried here.
- Done when: "Done when an overmap journey is one explicit `Resolved` naming every consequence; split-party clocks reconcile from the payload, never from peer derivation; a late joiner rebuilds the same state from the log alone; and a headed two-peer receipt exists."
- Needs: V2.2
- Rulings: 801
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:110-115` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:286-306`; `archive_docs/2026-10-28/2026-08-08_extracted_receipts.md:29-33`)

### V2.5 The overmap's readings
The overmap shows territory around each site the party knows, tinted by meaning the VTT owns (faction control where a claim exists, biome otherwise), over a generated backdrop, with cells that select their site and answer hover with its facts, and never a cell for a site the party does not know. The watchtower atlas (fixed region backdrop, clickable polygon areas) changes its source to the sim's place graph and reach field. The drawing itself is `orthographic-voxel-presentation` L10 (the overmap as a second camera held at R4, the canvas renderer retired, ruling 437).
- Done when: "Done when the sites and their discovery come from the sim through the contract, never from authored positions or `party_known`; the campaign shown is a draw under a seed nobody chose; and one capture shows the overmap from a past standpoint differing from the present on at least one site's version (the reach field's arrival entries: what was believed then, what the table knows now, what was retconned, which map version a character held)."
- Needs: V2.1, V2.2; `orthographic-voxel-presentation` L10
- Rulings: 801, 437, 84
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:117-122` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:307-326`)

### V2.6 Table time and downtime consent
The VTT's own downtime logic in `crates/isocosm-views/src/downtime.rs`: every saved bound table player, disconnected included, holds downtime unless they release their own hold (851); a proposal binds its original tick and player roster, and a change needs a new proposal (852); a self-released hold covers current and future proposals until the player retakes it (855); the DM explicitly chooses which ready proposal advances (856). Pace (860) is native sim code outside the society layer and is not here. The sim's society plan does not test any of this; the tests belong here.
- Done when: not stated; owed.
- Needs: V2.2 (unanimous consent, ruling 246)
- Rulings: 851, 852, 855, 856, 246; 860 for pace only
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:124-129` (itself from `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-14_stock_title_plan.md:162-166`)

### V3. The table over the sim
An adjudicated action at a battlemap returns through the handoff and passes the sim's invariants. The bench (the design record's W4) runs the calibration check over seeded draws.
- Done when: "Done when an adjudicated action at a battlemap returns through the handoff and passes the sim's invariants (ruling 154); the bench check of ruling 114 runs over seeded draws for Pathfinder 2e's skeleton first (ruling 249), a fight with no player choices resolving in the same distribution as the sim's own (rulings 115, 116, 123, 221 to 223) within the world's tolerance (ruling 209), and a ruleset that fails it plays only in a debug or experimental mode that warns when the campaign opens and marks every receipt and save uncalibrated (rulings 189, 250); a campaign with the sim off plays as today, its downtime the table's own, and still writes its facts as notes so the sim can be switched on later over the same history (rulings 188, 248); and a death at the table is reversed by the rules as re-embodiment (rulings 61, 62)."
- Case 1014 (2026-10-30, Mark: "Ruleset leads"): a table campaign defaults to ruleset-led, so the bench clause above binds an edition only where a world it plays in is sim-led; ruleset-led play needs no calibration.
- Needs: V1 (done); V2's world; the bench (W4); the grain and condition questions below
- Rulings: 114, 154, 249, 250, 189, 188, 248, 61, 62, 209, 1014
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:131-137` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:262-273`)

### V4. The played loop
W5's done-condition for the VTT.
- Done when: "Done when, from a seed nobody chose, a campaign hosted by one DM and joined by one player over the session lane plays three sessions with downtime between them on Cosmograph: a battlemap fight resolved by the ruleset, a downtime every player agreed to, in which factions act and the DM takes up a hook, a travel that reveals what the characters know, a death reversed by rules, and facts the DM committed; players see what their characters know and the DM the truth; the receipts replay on both peers to the same hash; and the fungible agree in distribution within the world's stated tolerance (ruling 113)."
- Needs: V1 (done), V2, V3; `session-network` H7
- Rulings: 113
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:139-144` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:274-284`)

### RS. Rulesets as versioned editions
Rulings 538 to 543 (2026-10-03): the VTT plays recognized editions faithfully as versioned rulesets, keeping the GM's decision points and recording their rulings, with a toolkit grown from what they repeat. A world chooses whether the sim or its ruleset leads, and 114's calibration holds only where the sim leads. An encounter across systems runs under a named hybrid profile. Ars Magica 5th Edition joins 5e and Pathfinder 2e (522). In a ruleset-led world an edition's own state persists beside the body with declared relations (542), and its procedures run unattended under declared decision policies (543). No phase or done-condition carried these.
- Built, by the old plan's Progress (`archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:279-290`): Pathfinder 2e creation from a saved, explained draft (cases 1020, 1021, 1026, 1029 to 1031), the founding panel's edition picker (case 1033), encounter mode, Stride over admitted characters (case 1042) and ordinary combat (cases 1047, 1050); Ars Magica's first slice as the `ars5` pack (cases 1063, 1068, 1069).
- Open, by the same lines: flanking is tested as geometry, not yet over a native party in the running game; "Take a class kit" is refused for want of an ORC record (case 1042); Ars troupe play (case 1065's several bodies), recovery rolls at sunrise and sunset, aging and the Wound-cell question below; the 5e SRD, specified meanwhile.
- Done when: not stated; owed.
- Needs: V3; the sim-led versus ruleset-led world switch; `content-packs` for which rulesets may ship (doctrine Licensing 1)
- Order (case 1013, 2026-10-30, Mark: "PF2e first"): Pathfinder 2e's Player Core Remaster is completed first for character creation and ordinary play; 5e SRD and Ars Magica are specified meanwhile. The default is ruleset-led (case 1014).
- Rulings: 538, 539, 540, 541, 522, 542, 543, 114, 1013, 1014
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:146-152` (itself from `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:42-50`)

### F10c and F11c. Edition procedures still open
F10a, F10b, F11a, F11b and F12 are built, with their receipts in `archive_docs/receipts/2026-10-28_vtt_overlay_plan_receipts.md`. These stay open. F10c: native persistent-effect bindings and permanent devices; F11 supplies the VTT lifecycle for the three temporary weapon spells, and a timed note is not an enchantment. F11c: complete edition exceptions, desktop casting for the other eight procedures, scene body and lighting consumers, concentration integration with other damage sources, live multiplayer admission and cross-map migration. Named work: D&D confined-space fitting, equipment meshes, object targets and Strength advantage rolls; PF2e group casting; Ars stress and botch, full resistance and laboratory devices. F12's first adapter is free play: initiative, class progression and other damage sources are not wired; native physical item enchantment and articulated weapon reach stay open, and the live peer gate is the session lane's. Native magic and calibration for unattended worlds are separate gates (`functional-generation`).
- Done when: not stated; owed.
- Needs: the existing native `rules::Effect` and `Process` grammar; `isocosm-system` for edition content and inspection; `functional-generation` F4 for F10c; `session-network` for the live peer gate.
- Rulings: `functional-generation` D1.
- From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:162-231`.
- Usage and exact supported profiles: [effect procedures](../crates/isocosm-system/data/effects/README.md).

## Deferred and parked

- **Follow-ons waiting on V2**: a reusable character library and Knot prose editing (W5); initializing parties other than the session party and their tactical doorway updates (W9); independent per-party map views and party-membership editing (W7). Parties and their knowledge then come from the sim. Waits on V2. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:410-417`.
- **The environment interaction matrix** (fire on grease, water plus lightning, water plus cold): expressed as process definitions a world declares, with area spells as ruleset intents whose effects those processes carry; a pack forces a process where the world lacks it. No VTT work is scheduled; the processes are the sim's. Waits on `sim` (processes and field passes) and V3. Rulings 12, 32, 72, 98, 154, 189, 190, 243, 249, 250, 257. From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:24-61`.

## Questions for Mark

- **Overmap choices (still Mark's)**: faction tint against biome tint where both exist; whether a known cell's undiscovered edges clip hard or fade; whether baked map thumbnails are worth a backdrop. Bears on V2.5. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:322-325`.
- **Readings awaiting a ruling or confirmation**: (a) `ActionResolved`'s fields (request id, rolls, sheet deltas, beats, defeated, displaced, conditions) are the handoff's outcome, with dice and beats left at the table; (b) stances and emotes stay the table's; (c) the 23-variant `GameEvent` split: map token moves are per-tick batches; turn order, `Rolled`, `SheetSet`, `Emoted`, `StanceSet`, `MapStored`, `MapActivated` and `Generation` stay the table's; `Fact`, `CharacterCreated`, `ItemModifierRevealed` and `World` are assertions; `PartyMoved`, `PartyPaceSet`, `TransitionResolved` and `TravelResolved` travel; `NodeRevealed` is knowledge; `InventorySet` and `ItemTransfer` are exchange; `TimeAdvanced` is downtime (confirmed variant by variant in V1); (d) a sim-off campaign's downtime is the table's own. Rulings 154, 244, 247, 188. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:156,167-190,371-373`.
- **Grain of the field read at a battlemap (open, for V3)**: the cell, or the tile's five-voxel face (ruling 244's per-tick batches). From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:62-66`.
- **Conditions the world has no process for**: whether a ruleset may declare one, and what an uncalibrated one shows (rulings 189, 250). From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:66-67`.
- **Wound cells scaled by body size**: a native Wound lands on one drawn part (the admitted body: 38 parts of 8 or 9 cells, 314 in all) and takes no more than it holds, so body-scaled counts sever a part at every Wound level; the provisional Ars 5/15/25/50 and Pathfinder 5 (cells over the living parts, at least one) stand. Case 1069. From: `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:282,285`.

## Findings

Findings of landed work stay in the archived plan (`archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:245-276`) and their numbers in the receipts file; these bear on what is open.

- 2026-11-01, 1066: visit notes are not written by visits. A visit is recorded in three places (`journey::leave`, the staged pace act `stage/staged/pace.rs:32`, and custody following `custody/follow.rs:90,100`), two of them without the simulation's note writer, and the note budget is 100,000 per world; the visits themselves are already kept on each body. Telling reads residence and visits to copy a place note instead, so every told place is a note while no journey adds one.
- 2026-11-01, 1066: offered events by hop depth. On the mill-ford quick-start table (6 living bodies, 22 events, seeds 1 to 3, up to tick 300) every body knows every event, so nothing is offered at any depth. In the small generated worlds (3 sites, 48 bodies) co-located pairs differ by at most 5 events and depth 1 already offers all of them. In a larger world (8 sites, 240 bodies, seed 5, tick 50; 40 bodies paired regardless of site) the unknown set is median 23, p90 78, max 92, and depth 1 offers median 15, p90 45, max 62; depths 2 and 3 offer exactly what depth 1 does (seed 23 alike). No readability cap is needed on today's tables; a large world's reunions could offer dozens on one card.
- 2026-10-30, RS: languages offer only generic common names (Draconic, Dwarven, Elven, Fey, Gnomish, Goblin, Halfling, Orcish); Jotun, Sakvroth, Taldane and the uncommon list are held back pending a reserved-material check.
- 2026-10-30, RS: the 1016 filter is license ORC and `remaster: true` together, enforced by the record loader. Rage of Elements carries OGL with `remaster: true`; Beginner Box heritages pass the filter but sit outside Player Core and the 1021 slice. The foundryvtt/pf2e default branch is `v14-dev`; `tree/master` links are dead.
- 2026-10-30, RS: Player Core leaves some background Lores to the player to name (a city, a cult). The draft records the book's description for those; terrain Lores offer Player Core's nine terrain types. Raised by Belief needs a deity and is refused with that reason; languages beyond the ancestry's fixed ones are not chosen yet.
- 2026-10-30, F12: `isocosm-native::sheets` advertises controls through the current seat cut; a player can cast on their own wielded weapon but cannot grant slots, edit sheets or override reach. Weapon and explicit external damage call `active_effects::concentration_check`, retaining the actual faces. Native downtime settles expiry in the same accepted transaction. The first adapter is free play; initiative, class progression and other damage sources are not wired here.
- 2026-10-28, F10b: the existing allocation and growth fixtures now also execute through `Session` and restore the exact state, entries and witness. Sensory allocation changes the native area reading without creating matter. Recipe growth adds paid parts without creating matter. Neither demonstrates darkvision, temporary resizing, polymorph, or an edition-native spell procedure. Persistent enchantment, suppression and dispelling have no native binding; a note's text lifetime and the function evaluator's charge storage do not provide one.
- 2026-11-02: re-checked against the tree for V2.2 and V2.4: `GameEvent::TravelResolved` is still an inline-struct variant (`crates/isonetry/src/protocol.rs:416`), emitted at `crates/isocosm-genet/src/overmap.rs:133`, and `PROTOCOL_VERSION` is 25 (`protocol.rs:27`); `party_known` still lives in `crates/isocosm-native/src/founding/content.rs:18` and `crates/isocosm-campaign/src/content/world/{types,ops}.rs`; no `FactionVerb` remains. This replaces the 2026-10-28 lines of the old plan (`archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md:269-270`), which read protocol 22 at `protocol.rs:26`.

## Progress

- 2026-11-02: cut from `2026-10-28_vtt_overlay_plan.md`, archived whole as `archive_docs/2026-11-02/2026-10-28_vtt_overlay_plan.md`. Its landings to date, one line each, are in that file's Progress (lines 277-295); their run evidence is in `archive_docs/receipts/2026-10-28_vtt_overlay_plan_receipts.md`.
