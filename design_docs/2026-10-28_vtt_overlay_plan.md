# The VTT over the sim

**Status:** 2026-10-28: cut from `2026-09-25_vtt_overlay_plan.md`, `2026-07-08_environmental_surfaces_plan.md`, the VTT parts of `2026-08-08_extracted_receipts.md` and the scene-board path of `2026-09-15_board_on_isometer_plan.md`, which are archived in `archive_docs/2026-10-28/`. V0 (the profile ruled) and V1 (the contract's VTT side, `shared/cosmograph-overlay/src/vtt/`) are done. V2 is partly placed in code: native founding, journeys, items, disclosure and the session protocol exist. The sim-off `CampaignWorld` reading with `party_known`, the inline `TravelResolved` event and the sim-on lifted-site battlemap remain. V2 as a whole, V3 and V4 are open. F10 to F12 (moved from `functional-generation` 2026-10-30) are built: the effect catalogue, nine experimental edition procedures and Magic Weapon at the shipping table; F11c is open.

**For:** the VTT as a game over the Cosmograph sim: V2 (a campaign over a drawn world), V3 (the table over the sim, calibrated handoff), V4 (the played loop, W5's done-condition for the VTT), the VTT's rulesets and their edition procedures (F10 to F12), and how a ruleset reads the sim's environment at a battlemap. The VTT side of the overlay contract (ruling 154) lives in `shared/cosmograph-overlay`.
Not here: the sim itself, which is `sim` (S1, S3, S5 are the phases V2 lands under); retiring `isocosm-campaign` and moving its world into Cosmograph, which is `campaign-retirement` R3; drawing the board and the DOM board's fate, which is `board-on-isometer`; the overmap's drawing as a second camera, which is `orthographic-voxel-presentation` L10; the session carrier and host handoff, which is `session-network`; pack terms and distribution, which is `content-packs`; Eponym's recalibration of its sortie scene, which is `eponym-overlay`.

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
   downtime; and whether Cosmograph absorbs `isocosm-campaign`'s world as
   ruling 192 had it absorb `mesocosm-core`, in the order proposed in §4.
   **Ruled 247 (2026-09-26): "Retires at V2."** Cosmograph absorbs
   `isocosm-campaign`'s world in §4's order, and the tape-drawn faction
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
The VTT's own downtime logic in `crates/isocosm-views/src/downtime.rs`: every saved bound table player, disconnected included, holds downtime unless they release their own hold (851); a proposal binds its original tick and player roster, and a change needs a new proposal (852); a self-released hold covers current and future proposals until the player retakes it (855); the DM explicitly chooses which ready proposal advances (856). Pace (860) is native sim code outside the society layer and is not here. The sim's society plan does not test any of this; the tests belong here.
- Done when: not stated; owed.
- Needs: V2.2 (unanimous consent, ruling 246)
- Rulings: 851, 852, 855, 856, 246; 860 for pace only
- From: `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-14_stock_title_plan.md:162-166`

### V3. The table over the sim
An adjudicated action at a battlemap returns through the handoff and passes the sim's invariants. The bench (the design record's W4) runs the calibration check over seeded draws.
- Done when: "Done when an adjudicated action at a battlemap returns through the handoff and passes the sim's invariants (ruling 154); the bench check of ruling 114 runs over seeded draws for Pathfinder 2e's skeleton first (ruling 249), a fight with no player choices resolving in the same distribution as the sim's own (rulings 115, 116, 123, 221 to 223) within the world's tolerance (ruling 209), and a ruleset that fails it plays only in a debug or experimental mode that warns when the campaign opens and marks every receipt and save uncalibrated (rulings 189, 250); a campaign with the sim off plays as today, its downtime the table's own, and still writes its facts as notes so the sim can be switched on later over the same history (rulings 188, 248); and a death at the table is reversed by the rules as re-embodiment (rulings 61, 62)."
- Case 1014 (2026-10-30, Mark: "Ruleset leads"): a table campaign defaults to ruleset-led, so the bench clause above binds an edition only where a world it plays in is sim-led; ruleset-led play needs no calibration.
- Needs: V1 (done); V2's world; the bench (W4); the grain and condition questions below
- Rulings: 114, 154, 249, 250, 189, 188, 248, 61, 62, 209, 1014
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:262-273`

### V4. The played loop
W5's done-condition for the VTT.
- Done when: "Done when, from a seed nobody chose, a campaign hosted by one DM and joined by one player over the session lane plays three sessions with downtime between them on Cosmograph: a battlemap fight resolved by the ruleset, a downtime every player agreed to, in which factions act and the DM takes up a hook, a travel that reveals what the characters know, a death reversed by rules, and facts the DM committed; players see what their characters know and the DM the truth; the receipts replay on both peers to the same hash; and the fungible agree in distribution within the world's stated tolerance (ruling 113)."
- Needs: V1 (done), V2, V3; `session-network` H7
- Rulings: 113
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:274-284`

### RS. Rulesets as versioned editions
Rulings 538 to 543 (2026-10-03): the VTT plays recognized editions faithfully as versioned rulesets, keeping the GM's decision points and recording their rulings, with a toolkit grown from what they repeat. A world chooses whether the sim or its ruleset leads, and 114's calibration holds only where the sim leads. An encounter across systems runs under a named hybrid profile. Ars Magica 5th Edition joins 5e and Pathfinder 2e (522). In a ruleset-led world an edition's own state persists beside the body with declared relations (542), and its procedures run unattended under declared decision policies (543). No phase or done-condition carried these.
- Done when: not stated; owed.
- Needs: V3; the sim-led versus ruleset-led world switch; `content-packs` for which rulesets may ship (doctrine Licensing 1)
- Order (case 1013, 2026-10-30, Mark: "PF2e first"): Pathfinder 2e's Player Core Remaster is completed first for character creation and ordinary play; 5e SRD and Ars Magica are specified meanwhile. The default is ruleset-led (case 1014).
- Rulings: 538, 539, 540, 541, 522, 542, 543, 114, 1013, 1014
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:42-50`

### F10 to F12. Edition procedures
Moved here from `functional-generation` on 2026-10-30 with their labels (Mark,
of whether F10 to F12 move to a VTT plan: "Yeah, sure."). They are RS's first
built work: the effect catalogue with edition correspondences, nine bounded
procedures and Magic Weapon at the shipping table. The decision they carry
out is `functional-generation` D1; native magic, the six axes and the cost
curve stay there.

#### F10. Effect catalogue and edition correspondences
The initial coverage map has ten families: resources and harm, defence and checks, anatomy, movement, perception and knowledge, agency and relationships, matter and environment, spatial access, life and identity, and persistence and interaction. Catalogue entries name the native primitives they may use and keep that evidence distinct from an implemented ruleset procedure. The ruleset lane owns reference mappings to D&D 5e SRD 5.1, Pathfinder 2e Player Core (2023), and Ars Magica 5e; these are source correspondences, not conversions of spell rank, energy, damage, duration or authority. Unknown and optional correspondences stay explicit. This tranche does not adopt a universal magic grammar or cost curve.

- F10a done when: a versioned TOML catalogue covers every family and carries a mapping or an explicit gap for all three sources on every entry; admission rejects duplicate identities, unknown native operations, missing source references and unjustified implementation claims; a runnable inspector reports the coverage and the actual native process candidates.
- F10b done when: the first body and sensory bindings are exercised through accepted native processes, refusal controls and exact save/replay, with their actual scope stated. A matching primitive alone does not establish a full spell implementation.
- F10c: F11 supplies the VTT lifecycle for the three temporary weapon spells: identity, source, bearer, activation, termination, interference and overlapping changes. Native persistent-effect bindings and permanent devices remain open; a timed note is not an enchantment. Complete edition execution still needs each exception and casting method.
- Needs: the existing native `rules::Effect` and `Process` grammar; `isocosm-system` for edition content and inspection; `functional-generation` F4 for F10c.

#### F11. The agreed small set: executable edition procedures

Mark's follow-up, "What about the small set?", identifies the unfinished half
of `functional-generation` D1. Catalogue metadata and native allocation/growth controls did not meet
the promised spell execution. This phase connects specific procedures rather
than treating those controls as implementations of tabletop magic.

- Scope: D&D SRD 5.1 Darkvision, Enlarge/Reduce and Magic Weapon; Pathfinder
  Player Core Remaster Darkvision, Enlarge and Runic Weapon; Ars Magica
  Definitive Fifth Edition Eyes of the Cat, Preternatural Growth and Shrinking,
  and Edge of the Razor. Each remains an experimental, bounded procedure.
- F11a done when: accepted casts consume declared resources/actions, preserve
  edition targeting and duration, retain source definitions and dice, survive
  save/replay, and refuse atomically. The calm unmastered Ars profile is
  explicit; stress, resistance and counteract decisions are not guessed.
- F11b done when: senses change the terrain visibility reading, size changes
  project the current part graph, and lasting weapon changes feed the existing
  damage consumer. Expiry preserves later base edits; suppression sources and
  dispel outcomes are addressable; overlap cannot add the same spell twice.
- F11c open: complete edition exceptions, desktop casting for the other eight
  procedures, scene body/lighting consumers, concentration integration with
  other damage sources, live multiplayer admission and cross-map migration.
  F12 connects Magic Weapon, its weapon attacks and explicit table damage.
  D&D confined-space fitting, equipment
  meshes, object targets and Strength advantage rolls; PF2e group casting;
  Ars stress/botch, full resistance and laboratory devices remain named work.
  Native magic and calibration for unattended worlds remain separate gates.
- Owner: `isocosm-system::active_effects`, using the substrate's uninterpreted
  `MapDocument.system_data`, existing inventories, dice and height visibility;
  native body documents and native clock/sky reads are inputs. There is no new
  biological simulator, universal spell level or material cost curve.
- Usage and exact supported profiles: [effect procedures](../crates/isocosm-system/data/effects/README.md).

#### F12. Magic Weapon in the shipping table

2026-10-30, done for the bounded free-play loop. Mark approved: choose a
weapon from a sheet, accept the cast through the current authority, spend the
slot, resolve an enhanced attack, lose concentration on damage, expire with
elapsed time, and restore/replay the same accepted results.

- Owner: `isocosm-native` endpoint over the existing family host, with rules in
  `isocosm-system`. The retired mutable GameSnapshot route is not extended.
- Done when: rendered character-sheet actions invoke this endpoint; refused
  or stale requests spend no resource or RNG; attack damage triggers the
  existing concentration procedure in the same transaction; native time ends
  the effect; checkpoints and family replicas preserve accepted edition state.
- Native participants/seat cuts determine control and disclosure. Edition HP,
  slots and unmarked tabletop weapons are labelled ruleset facts beside the
  native body, not a second biological or item-custody simulation. Native
  physical items are not silently counted again through legacy equipment.
- Physical range uses current native placement when a table scale is declared;
  otherwise an attack requires an explicit DM reach ruling, retained in history.
  Self-touch casting on one's equipped weapon requires no guessed terrain pose.
- The first range adapter measures native root distance and a terrain ray
  through the sim's existing site geometry cache. Articulated weapon reach,
  body size and native physical equipment bindings remain separate work.
- Scope: D&D SRD 5.1 Magic Weapon and its weapon attack consumer. Other spells,
  native magical manufacture, item migration and edition-complete combat remain
  F11c work. Production peer transport/admission remains the session lane's gate.
- Receipts: family admission/replay and save controls, view routing checks,
  feature-enabled compile checks, and a bounded desktop capture, in the one
  receipt file.

## Deferred and parked

- **Follow-ons waiting on V2**: a reusable character library and Knot prose editing (W5); initializing parties other than the session party and their tactical doorway updates (W9); independent per-party map views and party-membership editing (W7). Parties and their knowledge then come from the sim. Waits on V2. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:410-417`.
- **The environment interaction matrix** (fire on grease, water plus lightning, water plus cold): expressed as process definitions a world declares, with area spells as ruleset intents whose effects those processes carry; a pack forces a process where the world lacks it. No VTT work is scheduled; the processes are the sim's. Waits on `sim` (processes and field passes) and V3. Rulings 12, 32, 72, 98, 154, 189, 190, 243, 249, 250, 257. From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:24-61`.

## Questions for Mark

- **Overmap choices (still Mark's)**: faction tint against biome tint where both exist; whether a known cell's undiscovered edges clip hard or fade; whether baked map thumbnails are worth a backdrop. Bears on V2.5. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:322-325`.
- **Readings awaiting a ruling or confirmation**: (a) `ActionResolved`'s fields (request id, rolls, sheet deltas, beats, defeated, displaced, conditions) are the handoff's outcome, with dice and beats left at the table; (b) stances and emotes stay the table's; (c) the 23-variant `GameEvent` split: map token moves are per-tick batches; turn order, `Rolled`, `SheetSet`, `Emoted`, `StanceSet`, `MapStored`, `MapActivated` and `Generation` stay the table's; `Fact`, `CharacterCreated`, `ItemModifierRevealed` and `World` are assertions; `PartyMoved`, `PartyPaceSet`, `TransitionResolved` and `TravelResolved` travel; `NodeRevealed` is knowledge; `InventorySet` and `ItemTransfer` are exchange; `TimeAdvanced` is downtime (confirmed variant by variant in V1); (d) a sim-off campaign's downtime is the table's own. Rulings 154, 244, 247, 188. From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:156,167-190,371-373`.
- **Grain of the field read at a battlemap (open, for V3)**: the cell, or the tile's five-voxel face (ruling 244's per-tick batches). From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:62-66`.
- **Conditions the world has no process for**: whether a ruleset may declare one, and what an uncalibrated one shows (rulings 189, 250). From: `archive_docs/2026-10-28/2026-07-08_environmental_surfaces_plan.md:66-67`.

## Findings

- 2026-10-31, RS: Stride (case 1042) carries `native::submit_table`'s cell batch inside the ruleset's accepted transaction and spends one action. It is measured on the root distance reach already uses, at the campaign's cells per square, against Speed less armour and shield penalties (armour's eased by 5 feet when its Strength is met). With no scale, a GM seat's recorded ruling stands in. Only characters admitted through native recipe admission and placement move; other bodies are refused with that reason. The card offers the four compass directions at one square and at full Speed; the board click remains the richer interface. `tests/support/admitted.rs` is the first admitted-character fixture in the repository; it places each character on the highest clear cell of a resident's column, because resident poses can sit at the volume's ceiling, which placement refuses.
- 2026-10-31, RS: the PF2e sheet card recomputes Stride destinations, which means lifting the site volume, on every snapshot. The admitted-bench test takes about 40 s; a cached ground per position revision would cut that.
- 2026-10-30, RS: Pathfinder play runs in encounter mode held in edition state (`pf2e::play::Encounter`): Perception initiative, three actions, a reaction and an attack count per turn. Strikes reuse the adapter's measured reach or a GM seat's recorded reach ruling; ranged Strikes wait for ranged measurement. Hit Points, dying, wounded and unconscious are sheet fields; a critical hit taken and a drop to dying each publish `Command::Wound` (one cell, part drawn by the sim, provisional until the size is ruled), and death publishes `Command::Kill`, inside the same accepted transaction. A native death shows on the sheet.
- 2026-10-30, RS: languages offer only generic common names (Draconic, Dwarven, Elven, Fey, Gnomish, Goblin, Halfling, Orcish); Jotun, Sakvroth, Taldane and the uncommon list are held back pending a reserved-material check.
- 2026-10-30, RS: the edition is a field of the campaign's founding record, `founding::Content::edition` (case 1029), absent and 5e in older saves; `ruleset::edition` reads it, and `State::read` refuses a sheet of another system or a creation draft under a 5e founding. How the founding flow sets it (the world-profile file, a picker in the founding panel, re-founding an existing world) is open.
- 2026-10-30, RS: a Pathfinder 2e creation draft stores only its ordered choices (`isocosm_system::pf2e::draft::Draft`); walking them against the records rebuilds the build and names the next decision, each option with its summary and each refused option with its reason. A stale or tampered choice refuses the draft rather than repairing it; numbers are derived (`pf2e::derive`), never stored. Only `hp_current` persists on the sheet.
- 2026-10-30, RS: the 1016 filter is licence ORC and `remaster: true` together, enforced by the record loader. Rage of Elements carries OGL with `remaster: true`; Beginner Box heritages pass the filter but sit outside Player Core and the 1021 slice. The foundryvtt/pf2e default branch is `v14-dev`; `tree/master` links are dead.
- 2026-10-30, RS: Player Core leaves some background Lores to the player to name (a city, a cult). The draft records the book's description for those; terrain Lores offer Player Core's nine terrain types. Raised by Belief needs a deity and is refused with that reason; languages beyond the ancestry's fixed ones are not chosen yet.
- 2026-10-30, F12: the shipping desktop's authority is `FamilyTable` / `VttEndpoint`, not the retained legacy `GameSnapshot` handlers. `isocosm-native::ruleset` stores sheets, unmarked table equipment, active effects, the dice stream and result history in an opaque world section. Its changes share the existing atomic world acceptance and family checkpoint/record replay. It creates neither native anatomy nor native items.
- 2026-10-30, F12: `isocosm-native::sheets` advertises controls through the current seat cut; a player can cast on their own wielded weapon but cannot grant slots, edit sheets or override reach. Weapon and explicit external damage call `active_effects::concentration_check`, retaining the actual faces. Native downtime settles expiry in the same accepted transaction. The first adapter is free play; initiative, class progression and other damage sources are not wired here.
- 2026-10-30, F12: the native simulation's `spatial_poses` and admitted VTT `PositionBinding` both supply physical roots. A generated founding stand alone has no admitted terrain patch; the physical test fixture publishes `Command::Patch` just as Mesocosm does. `ruleset::frame` refuses stale or unplaced bodies and reuses `Simulation::site_geometry` and `Places::sees` for current measured reach. No rules-map coordinates are published as placement.
- 2026-10-30, F12: D&D critical hits now repeat damage dice while adding flat modifiers once, through the existing dice parser and action resolver. PF2e retains its whole-damage multiplier. The desktop readout now shows zero remaining time after concentration ends; the initial misleading capture is retained with the corrected result in the dated receipts.
- 2026-10-28, F11: the three procedure packs in `crates/isocosm-system/data/effects/` declare specific casting profiles. D&D uses slots and concentration; PF2e prepared casts share `actions_spent` with Strikes; Ars uses calm formulaic totals, fatigue and the accepted next sunrise/sunset. Player Core's base Darkvision is personal and Enlarge lasts five minutes. Ars cat eyes do not see in absolute darkness. The source and profile boundaries are kept with the data.
- 2026-10-28, F11: `active_effects` stores accepted definitions, cast costs, dice, bearer IDs, independent suppressors and end causes. Current sheets and inventories are projected rather than restored from old copies. `System::effective_map_sheet`, the attack resolver, the height tracer and the shared body graph consume the changes. Exact quantized scaling refuses when a joint would round; Ars Size does not imply an exact native geometric ratio. This is VTT state beside native bodies, not native biological mutation.
- 2026-10-28, F10: `crates/isocosm-system/data/effects.toml` holds 54 vocabulary entries across ten families and pins three reference corpora. Nine related correspondences cover sensory allocation, recipe growth and lasting enchantment; 153 mappings are explicitly unreviewed. This is an initial coverage map, not a complete spell inventory or three executable rulesets. Source content stays in the system lane; no tabletop text or costs were added to native simulation rules.
- 2026-10-28, F10: native `rules::Effect::kind` and `Process::effect_declarations` inspect the existing grammar, including commitments, guarded alternatives and risks. Catalogue candidates are declarations, not accepted acts. A sensory allocation candidate must both bind and allocate `function:sense`; an unrelated gland allocation does not qualify. The catalogue does not admit new operations or alter world saves.
- 2026-10-28, F10b: the existing allocation and growth fixtures now also execute through `Session` and restore the exact state, entries and witness. Sensory allocation changes the native area reading without creating matter. Recipe growth adds paid parts without creating matter. Neither demonstrates darkvision, temporary resizing, polymorph, or an edition-native spell procedure. Persistent enchantment, suppression and dispelling have no native binding; a note's text lifetime and the function evaluator's charge storage do not provide one.
- 2026-10-28, prior art for F10: Fantasy Grounds' [CoreRPG layering](https://fantasygroundsunity.atlassian.net/wiki/spaces/FGCP/pages/996644522/Ruleset%2B-%2BLayering), Foundry's [Active Effects](https://foundryvtt.com/article/active-effects/) and [PF2e rule elements](https://github.com/foundryvtt/pf2e/wiki/Quickstart-guide-for-rule-elements) inform the separation of common infrastructure, specific procedures and inspectable changes. They are research references, not adopted dependencies or proof of a complete simulation adapter.
- 2026-10-28: the entropy-tape faction turn and `legacy/campaign` are gone from code (no `FactionVerb`); V2.2's retirement clause is satisfied, the rest of V2.2 is not. `party_known` still lives in `crates/isocosm-native/src/founding/content.rs` and `crates/isocosm-campaign/src/content/world/ops.rs`.
- 2026-10-28: `GameEvent::TravelResolved` is still emitted inline at `crates/isocosm-genet/src/overmap.rs:133`; the wire is at `PROTOCOL_VERSION` 22 (`crates/isonetry/src/protocol.rs:26`). The protocol hardening plan's H0 and H1 landed (DOC_README).
- 2026-10-28: the old plan's "five failing watchtower atlas and door tests" are not carried: all `isocosm-genet` tests passed today.
- 2026-10-28: the desktop host boots only the `core` pack (`crates/isocosm-genet/src/boot.rs:25`); `demo` and `watchtower` are test fixtures. The old overlay's §2 "Founding a campaign" and §7 predate that.
- 2026-10-28: the old overlay's §4 line counts and its `faction` module row are stale; the destination column still stands and is `campaign-retirement` R3's.
- 2026-10-28: the design height is settled (`DESIGN_SIZE`, 820 logical px); reachability at the smallest supported display is `board-on-isometer`'s. Ruling 346's binding shape (never built, "built when Mark says") is `bodies`'.
- 2026-10-28: the old §6 stylesheet-versus-voxel appearance row went to `board-on-isometer`; Eponym's unrecalibrated sortie hazard went to `eponym-overlay`.

## Progress

- 2026-10-31, RS: Stride over admitted characters (case 1042), ruled and measured; the Adventurer's Pack and its gear; "Take a class kit" refused for want of an ORC record (case 1042). Headed Stride trial in the receipts file.
- 2026-10-30, RS: the founding panel picks the edition (case 1033). Starting gear is bought with 15 gp from the 92 Player Core level-0 weapons, armour and shields (uncommon ones need access); languages are chosen. In encounter mode a character readies gear, Strikes with the multiple attack penalty, Raises a Shield and makes skill checks against a GM seat's DC; GM-ruled damage runs the dying track to native death. Headed play trial in the receipts file. Stride, class kits and the rest of the gear wait on rulings.
- 2026-10-30, RS: Pathfinder 2e creation (cases 1020, 1021, 1026, 1029 to 1031). 131 Player Core records (2 ancestries, 7 heritages, all 40 backgrounds, Fighter and Rogue with their 1st-level features and class feats, 14 ancestry, 14 general and 35 skill feats) in `crates/isocosm-system/data/pf2e/`. The adapter admits the edition beside 5e without changing 5e's saved bytes; the seat controlling a body starts, answers, takes back and finishes a saved draft through `VttEndpoint`, and any controlling seat, GM seats included, may discard one. Headed trial in the native app and disk reopen in the receipts file. Equipment, actions and incapacitation are next.
- 2026-10-30: F10 to F12 moved here from `functional-generation`, with their findings and progress lines.
- 2026-10-30: F12 connects Magic Weapon to the shipping character-sheet endpoint, accepted slots and dice, enhanced attacks, damage-driven concentration, native downtime expiry, family replay and disk reload. Headed active/lost/expired states are verified; a misleading ended-effect countdown was corrected. Native physical item enchantment, articulated weapon reach, the other desktop procedures and the live peer gate remain open.
- 2026-10-28: F11a/F11b executable API and headless demonstration built for all nine named procedures, with saved lifecycle and existing consumer integration. F11c remains open; the previous catalogue-only landing did not fulfill the whole agreed small set. Use `cargo run -p isocosm-system --example cast_effects` for execution and `inspect_effects` for coverage.
- 2026-10-28: F10a initial catalogue, admission checks and native candidate inspector built; F10b native allocation/growth conservation and exact-history controls passed. F10c and edition-native execution remain open. Run `cargo run -p isocosm-system --example inspect_effects -- --markdown` for the readable coverage matrix, `--json` for the revisioned report, or add `--genesis PATH` to inspect a native Genesis rather than the labelled reference founding. Run evidence belongs in the plan's receipt file.
- 2026-10-28: cut from the VTT overlay, environmental surfaces and extracted-receipts plans.
