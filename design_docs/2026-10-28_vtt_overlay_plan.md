# The VTT over the sim

**Status:** 2026-10-28: cut from `2026-09-25_vtt_overlay_plan.md`, `2026-07-08_environmental_surfaces_plan.md`, the VTT parts of `2026-08-08_extracted_receipts.md` and the scene-board path of `2026-09-15_board_on_isometer_plan.md`, which are archived in `archive_docs/2026-10-28/`. V0 (the profile ruled) and V1 (the contract's VTT side, `shared/isocosm-overlay/src/vtt/`) are done. V2 is partly placed in code: native founding, journeys, items, disclosure and the session protocol exist. The sim-off `CampaignWorld` reading with `party_known`, the inline `TravelResolved` event and the sim-on lifted-site battlemap remain. V2 as a whole, V3 and V4 are open.

**For:** the VTT as a game over the Isocosm sim: V2 (a campaign over a drawn world), V3 (the table over the sim, calibrated handoff), V4 (the played loop, W5's done-condition for the VTT), the VTT's rulesets, and how a ruleset reads the sim's environment at a battlemap. The VTT side of the overlay contract (ruling 154) lives in `shared/isocosm-overlay`.
Not here: the sim itself, which is `sim` (S1, S3, S5 are the phases V2 lands under); retiring `isometry-campaign` and moving its world into Isocosm, which is `campaign-retirement` R3; drawing the board and the DOM board's fate, which is `board-on-isometer`; the overmap's drawing as a second camera, which is `orthographic-voxel-presentation` L10; the session carrier and host handoff, which is `session-network`; pack terms and distribution, which is `content-packs`; Eponym's recalibration of its sortie scene, which is `eponym-overlay`.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Decisions

Copied verbatim from the old overlay's §6. All eight are taken. The "Owed" stylesheet-versus-voxel appearance row that headed the section has moved to `board-on-isometer`.

## 6. Decisions for Mark

All eight taken: the eighth by ruling 231 on 2026-09-25; the first to
seventh by rulings 243 to 250 on 2026-09-26. V0 and V1 are done.

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

## Constraints

- The campaign owns no second simulation authority; a campaign with the sim off plays as today, its downtime the table's own (rulings 188, 247). Case 798: V2 opened without waiting for Mesocosm's M3.
- No peer derives a consequence, and version negotiation refuses, never degrades (the H0 and H1 laws that stand under ruling 801).
- The contract stays coarse: per-tick batches, never a call per token (ruling 244). The VTT module in `shared/isocosm-overlay` depends on nothing sim-internal (D18, ruling 197).
- Done-conditions are draws, never fixtures (case 15).
- Every file stays under the 600-line ceiling as families move.
- Ruling 312: the environment is the sim's field on places; a ruleset only reads it at a battlemap.

## Phases

### V2. The campaign over a drawn world
The whole of V2, assembled from V2.1 to V2.6 below. It lands under `sim` S1, S3 and S5.
- Done when: "Done when a campaign founded from a seeded draw takes its places, routes, factions, characters, laws and history from the sim; an authored pack displaces generated content where the world meets its requirements and waits where it does not, until the GM forces it (rulings 89, 190); a declared downtime advances the sim with every player's yes (ruling 246) and the factions act from their members, the entropy-tape faction turn retired (ruling 247); the arcs the sim runs reach the DM as hooks (ruling 191); a party's overmap is what its characters know (ruling 84); and the campaign replays to the same hash. Lands under the sim plan's S1, S3 and S5." Headed acceptance is also open.
- Needs: `sim` S1, S3, S5; participant and party binding (a Character's owner metadata and old table discovery do not grant it, and the native campaign refuses the old travel and map-reading UI path until it exists); native polity and time consent; live transport (`session-network`).
- Rulings: 89, 190, 191, 246, 247, 84; case 798.
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:252-261,327-328`

### V2.1 Native campaign reading
Found a native `Session` from the asserted history and read `CampaignWorld` from it; admit drawn places, routes, factions, characters, laws and history, with packs found natively.
- Done when: "Done when sim-off assertions replay with the sim on to one native state hash and the campaign owns no second simulation authority."
- Needs: `sim` S1, S3, S5; the assertion boundary (Q2, landed)
- Rulings: 795, 799, 801, 247
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:16-22`

### V2.2 Travel, knowing and polities
Run travel and the overmap conditions (folded under 801) through the contract; replace `party_known` by native reach; require unanimous downtime consent; retire the faction turn and `legacy/campaign` once native polities act.
- Done when: not stated beyond V2; owed.
- Needs: V2.1; sim factions running; V2.4
- Rulings: 801, 246, 247, 84, 72, 205
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:23-26`

### V2.3 Battlemap and replay
Keep `MapTerrain` with the sim off. With it on, lift an isometer-space site (799), lower held map edits there, and prove one accepted assertion stream converges across connected peers. The lifted-site battlemap is drawn on the scene board; the scene lowering, selection and presentation gates for it are `board-on-isometer`'s. Open from the Q9 finding: native assertion admission can accept a held map edit naming a drawn `site:<id>` even when the sim-off `Asserted` fold cannot resolve it, so a native replay applies the caller's ordered entries through `Command::Assert` without that fold as a preliminary authority. Attachment (binding an existing site, or supplying native geometry for a new one) and native placement validation are open. Ruling 809: a Character's cell and optional participant owner persist in native table metadata beside the named site; native placement and control remain V2's work.
- Done when: not stated beyond V2; owed.
- Needs: V2.1; `board-on-isometer`
- Rulings: 799, 243, 244, 89, 769, 809
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:27-29,34-40,421-429`; `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:5-12`

### V2.4 Overmap travel resolves once
`GameEvent::TravelResolved` is an inline-struct variant whose clock, encounter and exhaustion consequences peers apply as they always did. It needs its own protocol break and qualification beyond the current protocol version. (This was the protocol hardening plan's H2, folded into V2 by ruling 801; the H here is not `sim-host`'s.) Still open from H0: a peer that never sends `Hello` is not version-gated and can push `Rolled`. The late-join replay receipt from the adjudication residue is carried here.
- Done when: "Done when an overmap journey is one explicit `Resolved` naming every consequence; split-party clocks reconcile from the payload, never from peer derivation; a late joiner rebuilds the same state from the log alone; and a headed two-peer receipt exists."
- Needs: V2.2
- Rulings: 801
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:286-306`; `archive_docs/2026-10-28/2026-08-08_extracted_receipts.md:29-33`

### V2.5 The overmap's readings
The overmap shows territory around each site the party knows, tinted by meaning the VTT owns (faction control where a claim exists, biome otherwise), over a generated backdrop, with cells that select their site and answer hover with its facts, and never a cell for a site the party does not know. The watchtower atlas (fixed region backdrop, clickable polygon areas) changes its source to the sim's place graph and reach field. The drawing itself is `orthographic-voxel-presentation` L10 (the overmap as a second camera held at R4, the canvas renderer retired, ruling 437).
- Done when: "Done when the sites and their discovery come from the sim through the contract, never from authored positions or `party_known`; the campaign shown is a draw under a seed nobody chose; and one capture shows the overmap from a past standpoint differing from the present on at least one site's version (the reach field's arrival entries: what was believed then, what the table knows now, what was retconned, which map version a character held)."
- Needs: V2.1, V2.2; `orthographic-voxel-presentation` L10
- Rulings: 801, 437, 84
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:307-326`

### V2.6 Table time and downtime consent
The VTT's own downtime logic in `crates/isometry-views/src/downtime.rs`: every saved bound table player, disconnected included, holds downtime unless they release their own hold (851); a proposal binds its original tick and player roster, and a change needs a new proposal (852); a self-released hold covers current and future proposals until the player retakes it (855); the DM explicitly chooses which ready proposal advances (856). Pace (860) is native sim code outside the society layer and is not here. The sim's society plan does not test any of this; the tests belong here.
- Done when: not stated; owed.
- Needs: V2.2 (unanimous consent, ruling 246)
- Rulings: 851, 852, 855, 856, 246; 860 for pace only
- From: `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-14_stock_title_plan.md:162-166`

### V3. The table over the sim
An adjudicated action at a battlemap returns through the handoff and passes the sim's invariants. The bench (the design record's W4) runs the calibration check over seeded draws.
- Done when: "Done when an adjudicated action at a battlemap returns through the handoff and passes the sim's invariants (ruling 154); the bench check of ruling 114 runs over seeded draws for Pathfinder 2e's skeleton first (ruling 249), a fight with no player choices resolving in the same distribution as the sim's own (rulings 115, 116, 123, 221 to 223) within the world's tolerance (ruling 209), and a ruleset that fails it plays only in a debug or experimental mode that warns when the campaign opens and marks every receipt and save uncalibrated (rulings 189, 250); a campaign with the sim off plays as today, its downtime the table's own, and still writes its facts as notes so the sim can be switched on later over the same history (rulings 188, 248); and a death at the table is reversed by the rules as re-embodiment (rulings 61, 62)."
- Needs: V1 (done); V2's world; the bench (W4); the grain and condition questions below
- Rulings: 114, 154, 249, 250, 189, 188, 248, 61, 62, 209
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:262-273`

### V4. The played loop
W5's done-condition for the VTT.
- Done when: "Done when, from a seed nobody chose, a campaign hosted by one DM and joined by one player over the session lane plays three sessions with downtime between them on Isocosm: a battlemap fight resolved by the ruleset, a downtime every player agreed to, in which factions act and the DM takes up a hook, a travel that reveals what the characters know, a death reversed by rules, and facts the DM committed; players see what their characters know and the DM the truth; the receipts replay on both peers to the same hash; and the fungible agree in distribution within the world's stated tolerance (ruling 113)."
- Needs: V1 (done), V2, V3; `session-network` H7
- Rulings: 113
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:274-284`

### RS. Rulesets as versioned editions
Rulings 538 to 543 (2026-10-03): the VTT plays recognized editions faithfully as versioned rulesets, keeping the GM's decision points and recording their rulings, with a toolkit grown from what they repeat. A world chooses whether the sim or its ruleset leads, and 114's calibration holds only where the sim leads. An encounter across systems runs under a named hybrid profile. Ars Magica 5th Edition joins 5e and Pathfinder 2e (522). In a ruleset-led world an edition's own state persists beside the body with declared relations (542), and its procedures run unattended under declared decision policies (543). No phase or done-condition carried these.
- Done when: not stated; owed.
- Needs: V3; the sim-led versus ruleset-led world switch; `content-packs` for which rulesets may ship (doctrine Licensing 1)
- Rulings: 538, 539, 540, 541, 522, 542, 543, 114
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:42-50`

## Deferred and parked

- **Follow-ons waiting on V2**: a reusable character library and Knot prose editing (W5); initializing parties other than the session party and their tactical doorway updates (W9); independent per-party map views and party-membership editing (W7). Parties and their knowledge then come from the sim. Waits on V2. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:410-417`.
- **The environment interaction matrix** (fire on grease, water plus lightning, water plus cold): expressed as process definitions a world declares, with area spells as ruleset intents whose effects those processes carry; a pack forces a process where the world lacks it. No VTT work is scheduled; the processes are the sim's. Waits on `sim` (processes and field passes) and V3. Rulings 12, 32, 72, 98, 154, 189, 190, 243, 249, 250, 257. From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:24-61`.

## Questions for Mark

- **Overmap choices (still Mark's)**: faction tint against biome tint where both exist; whether a known cell's undiscovered edges clip hard or fade; whether baked map thumbnails are worth a backdrop. Bears on V2.5. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:322-325`.
- **Readings awaiting a ruling or confirmation**: (a) `ActionResolved`'s fields (request id, rolls, sheet deltas, beats, defeated, displaced, conditions) are the handoff's outcome, with dice and beats left at the table; (b) stances and emotes stay the table's; (c) the 23-variant `GameEvent` split: map token moves are per-tick batches; turn order, `Rolled`, `SheetSet`, `Emoted`, `StanceSet`, `MapStored`, `MapActivated` and `Generation` stay the table's; `Fact`, `CharacterCreated`, `ItemModifierRevealed` and `World` are assertions; `PartyMoved`, `PartyPaceSet`, `TransitionResolved` and `TravelResolved` travel; `NodeRevealed` is knowledge; `InventorySet` and `ItemTransfer` are exchange; `TimeAdvanced` is downtime (confirmed variant by variant in V1); (d) a sim-off campaign's downtime is the table's own. Rulings 154, 244, 247, 188. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:156,167-190,371-373`.
- **Grain of the field read at a battlemap (open, for V3)**: the cell, or the tile's five-voxel face (ruling 244's per-tick batches). From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:62-66`.
- **Conditions the world has no process for**: whether a ruleset may declare one, and what an uncalibrated one shows (rulings 189, 250). From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:66-67`.

## Findings

- 2026-10-30: `functional-generation` F12 connects an experimental Magic Weapon free-play loop through the shipping `FamilyTable` / `VttEndpoint`, with edition facts in the native world's opaque product sections. It reuses system casting/attack/concentration, native time consent and family persistence. Table equipment is explicitly unmarked, and root/terrain reach requires declared scale or an explicit DM ruling. This is not V3 calibration, native magical item manufacture or V4's multiplayer played-loop acceptance. The remaining edition and consumer work stays in F11c/F12 of the functional generation plan.
- 2026-10-28: the entropy-tape faction turn and `legacy/campaign` are gone from code (no `FactionVerb`); V2.2's retirement clause is satisfied, the rest of V2.2 is not. `party_known` still lives in `crates/isocosm-vtt/src/founding/content.rs` and `crates/isometry-campaign/src/content/world/ops.rs`.
- 2026-10-28: `GameEvent::TravelResolved` is still emitted inline at `crates/isometry-genet/src/overmap.rs:133`; the wire is at `PROTOCOL_VERSION` 22 (`crates/isonetry/src/protocol.rs:26`). The protocol hardening plan's H0 and H1 landed (DOC_README).
- 2026-10-28: the old plan's "five failing watchtower atlas and door tests" are not carried: all `isometry-genet` tests passed today.
- 2026-10-28: the desktop host boots only the `core` pack (`crates/isometry-genet/src/boot.rs:25`); `demo` and `watchtower` are test fixtures. The old overlay's §2 "Founding a campaign" and §7 predate that.
- 2026-10-28: the old overlay's §4 line counts and its `faction` module row are stale; the destination column still stands and is `campaign-retirement` R3's.
- 2026-10-28: the design height is settled (`DESIGN_SIZE`, 820 logical px); reachability at the smallest supported display is `board-on-isometer`'s. Ruling 346's binding shape (never built, "built when Mark says") is `bodies`'.
- 2026-10-28: the old §6 stylesheet-versus-voxel appearance row went to `board-on-isometer`; Eponym's unrecalibrated sortie hazard went to `eponym-overlay`.

## Progress

- 2026-10-28: cut from the VTT overlay, environmental surfaces and extracted-receipts plans.
