# design_docs Index

*Names, 2026-09-22 (wing design record, rulings 109 and 110): the sim and the family are Isocosm, the second-person game is Eponym (formerly Paredros), the tabletop is Isocosm: VTT, with Isometry retired as a product word and kept only as the plain technical prefix of its crates, and Mesocosm is unchanged. Verbatim rulings, quotations and code paths keep the old words until the rename lane lands.*

**Platform verification, 2026-09-29:** the published scroll repair is adopted
at Mere `32edc2ad` / Genet `7a60ad79`. The ordinary 187-row test, four workspace
checks and 27 spine/lift tests pass; [the side-panel plan](2026-09-03_side_panel_diet_plan.md)
records source controls and retained renderer qualifications. Pre.4 and S13
remain separate gates; older dated hold notices are historical.

**Pre.4 diagnosis, 2026-09-30 (411):** after remote allocator reclamation
failed, Mark authorized bounded diagnosis with the zero-active baseline
preserved. Diagnosis now shows active allocations released after explicit
completion, while both test runs still fail. The repair-location fork is
pending; production repair, migration acceptance and integration remain held. The
[wing record](../mesocosm/design_docs/2026-09-18_wing_design_plan.md#progress)
preserves the failure and its evidence limits. Current consumer pins stand.

## Games wing entry points

This is the canonical repository entry index. The tabletop documents remain
here; [Mesocosm's product index](../mesocosm/design_docs/DOC_README.md) and
[Eponym's product index](../eponym/design_docs/DOC_README.md) retain their
local document catalogues. Shared wing design lives once in the imported
[founding record](../mesocosm/design_docs/2026-07-30_games_wing_founding.md),
with [magic and generator discussions](../mesocosm/design_docs/archive_docs/2026-09-26/2026-08-06_general_model_plan.md)
(archived 2026-09-26, ruling 310; the organs moved to the
[wing organs plan](../mesocosm/design_docs/2026-09-26_wing_organs_plan.md)).

- [Current wing architecture](../mesocosm/design_docs/2026-09-18_wing_design_plan.md#current-architecture-2026-09-28):
  one host and world save, independent play/knowledge/camera/detail choices,
  meaningful construction, capability/repertoire/proficiency, contextual
  effect composition and condition carryover between lives. The 2026-09-28
  refinements (404 to 407) are documented design, with implementation status
  and open questions retained in the linked owner plans.
  The first integration batch, native-verified 2026-09-29, connects generated terrain to the
  existing specimen bench ([SP3](../mesocosm/design_docs/2026-08-05_place_graph_engine_plan.md#a10-sp3s-handoff-2026-09-28-rulings-408-and-409));
  body mapping is documented in the body contract, with concrete construction
  and local placement still needing accepted sources.
- [Games wing consolidation](2026-09-09_games_wing_consolidation_plan.md):
  history-preserving repository import, source/worktree preservation, separate
  build workspaces, aligned platform dependencies, single-renderer migration,
  and consumer verification receipts with remaining upstream blockers.
- [Orthographic voxel presentation](../mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md):
  Resident geometry with shared depth is the selected spatial path. The
  equivalent-yaw and geometry-variety probes and Bench A's posed bounds and
  visible-part queries are complete. Bench B embeds one scene in Genet with
  Cambium producer lifecycle, CSS appearance and native pointer routing.
  Generation comparison now holds the original beside four admitted shape
  alternatives, with native inspection, scale comparison and saved replay.
  Generation wiring adds seeded parent trees, eight starting anatomies and
  independent size/mass controls; native acceptance passes. Composable anatomy extends the grammar; native acceptance and saved replay pass. A native-verified planar glyph-effect experiment separates guaranteed responses from seeded receiver rules, with animated marks, compatible strings and saved replay. Opaque spatial forms cover orbit, mesh-face inscription, tether and emission with shared body depth, seeded appearance and two orthographic views. A 21-configuration sparse population receipt measures body/design counts independently through 1000, including native CPU stages, uploads and foreground coverage; 23 dense/layered configurations add up to 95% foreground coverage and a separate completion diagnostic. Integrated real-scene budgets remain open.
  A bounded disposable idle trial preserves source generation and projects accepted movement and feeding records through the same scene. General CSS 3D, large-DOM scaling and planar retention landed in genet and netrender 2026-09-15/16; the appearance crate merger and broader world-intent trials remain separate work.
- [Cross-port sharing audit](../mesocosm/design_docs/2026-08-18_engine_ecology_rulings_and_review.md#7-cross-port-sharing-audit-2026-09-09):
  live-code findings across all three ports; common formats and integration
  proofs, dependency alignment, readback/surface reuse and minimap adoption.

Canonical index for `design_docs/`. Per DOC_POLICY §5, this file wins over
any other index and is updated in the same session as any doc change.

## Working principles for AI assistants

- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase, not doc-to-doc consistency.
- Plans carry done-conditions, not time estimates.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions,
  do not edit unasked.
- The substrate/system split is load-bearing: geometry and turns in the
  substrate, rules in system plugins. Keep it that way in every doc.

## Active docs

Rebuilt 2026-08-08 after the wing audit's archive pass: ten plans moved to
`archive_docs/2026-08-08/` with residues extracted to the receipts ledger.

| Doc | What it is |
| --- | ---------- |
| [DOC_POLICY.md](DOC_POLICY.md) | Documentation governance |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Product goals and pillars (maintainer-owned) |
| [2026-09-25_vtt_overlay_plan.md](2026-09-25_vtt_overlay_plan.md) | **2026-10-03: rulings 538 to 541: faithful versioned editions with a toolkit grown from them, Ars Magica joining 5e and Pathfinder 2e (522); authority per world.** **W5 plan, drafted 2026-09-25 at Mark's word ("Overlay plans to RPG"); V0 and V1 done 2026-09-26, the eight decisions ruled and the contract module `src/vtt/` landed in `shared/isocosm-overlay` (ruling 253); V2 to V4 proposed, waiting on Mesocosm's M3; ruling 231 placed this overlay side by side with Eponym's after Mesocosm's M3, rulings 243 and 244 settled the battlemap under the sim and moves within it reaching the sim as per-tick batches, rulings 245 to 248 sharing a character on by default, every player's yes to downtime, the faction turn retiring at V2 with absorption ruled, and a sim-off campaign writing its facts as notes, and rulings 249 and 250 Pathfinder 2e calibrated first and an uncalibrated campaign warning at open with its receipts marked.** The VTT as a game overlay over the Isocosm sim, from the wing design record's §5.7 (rulings 188 to 191), rulings 114 and 189 (rulesets calibrate; an uncalibrated one plays in a debug or experimental mode with a warning) and ruling 154, mirroring the Mesocosm overlay plan: the profile (the character inside its polities, the campaign's cast, a ruleset's mechanics over the substrate's geometry and turns, the DM's and the players' controls, the locked isometric lens with its own scopes, the turn), the played loop from founding through scenes, downtime, travel, death and the sim switched off, the VTT's side of the overlay contract as a module in `shared/isocosm-overlay`, whose handoff this game fills with the ruleset's resolved actions, today's twenty-three replicated events mapped onto it, what of `isometry-campaign` Isocosm would absorb, and the forks, all ruled: the battlemap under the sim (243), the table's grain (244), sharing a character (245), consent to downtime (246), the faction turn's fate and absorption (247), the first calibration (249, 250), a sim-off campaign's record (248) and which overlay is W5's second (231). |
| [2026-09-15_board_on_isometer_plan.md](2026-09-15_board_on_isometer_plan.md) | **Original lanes landed 2026-09-16; traversal repinned and verified 2026-09-27; paging integrated 2026-09-27 under 368/369/372/374 with counted overflow, a persistent device-bounded live budget and upfront allocation. Final gates pass 411 root and 293 shared tests, three consumer checks and six native sessions; the integration receipt records source and timing qualifications. The switch is held by ruling. Under rewrite per W1 (2026-09-18, wing design record ruling 31): until the rewrite lands its done-conditions are not authoritative.** The board through the wing's shared scene, `isometer`, instead of one DOM element per tile and token, keeping the locked 2:1 lens, the tileset-as-stylesheet contract and every tile and token selectable. `ISOMETRY_SCENE_BOARD=1` draws, picks, selects, tints overlays, edits by changed slots and hides above a focus elevation, with tiles subdivided five voxels across and two to a step so cliffs match the DOM board. Measured on one machine, a steady frame is 12.50 ms against the DOM board's 32.16, the gap being element emission and accessibility rather than rendering, and the scene arm emits zero board elements. Mark held the DOM board's fate until the family's brick cap is resolved: the scene board stops near 70 tiles square while the campaign generator authors boards to 256. Props, the checkerboard shade and free hover are the parity gaps left, and a cost model plus every improvement found in isometer are recorded for its owner. |
| [2026-09-05_watchtower_plan.md](2026-09-05_watchtower_plan.md) | **Rewritten to the wing design record 2026-09-26 (ruling 280): the height-field board is a far view of a grown volume (rulings 12, 13), W6's visibility a line-of-sight reading over bricks, seeds are draws (15), discovery is knowledge by reach through the VTT overlay plan's V2 (84, 117); every receipt stands as taken.** **W1-W10 verified; W11-W13 first implementation is in the working tree: focused atlas/native tests, workspace all-features/all-targets, a Windows build/capture, and the CPU interaction benchmark pass. M4 deployment and presented FPS remain separate.** The atlas slice selects fixed source-time region bounds, caches decorated terrain, filters fields by party knowledge, uses shared source-bound polygons and deterministic priority, and routes drag through normalized motion offsets while preserving campaign coordinates. Signalman remains a future consumer. Published pins, independent player map views and cross-game interchange remain separate. |
| [2026-09-03_side_panel_diet_plan.md](2026-09-03_side_panel_diet_plan.md) | **2026-09-28: published Mere `5ce144ff` / Genet `7b48f94d` passes all 187 text rows (zero short), and the detector now runs ordinarily; native consumer/source gates are recorded in the dated plan.** **2026-09-26: the figures moved under genet's rounded line boxes, idle 768 expanded and 533 collapsed, composing 790, a target pick 802, so every state now fits the 820 design (wing design record ruling 329); the row-holds-its-text receipt waits on genet's text fragment fix.** **P0 and P1 landed 2026-09-03 (uncommitted): idle panel 1024 to 796; two calls open (three-column mode grid, hints while composing) and one catalog stop (a state-carrying `disclosure`).** The side panel measured 1038 logical pixels against the 820 design height that host UI zoom now fits to; ordered cuts (mode grid, one-line map row, dice beside measure, hints only when relevant, a Turns disclosure last) bring its last painted edge to 800 with nothing removed. Gates P0-P1. |
| [2026-09-02_genet_host_migration_plan.md](2026-09-02_genet_host_migration_plan.md) | **M0-M4 landed in the working trees 2026-09-03, uncommitted; closing commits and pin bumps are Mark's.** The desktop host no longer resolved from a clean checkout: genet retired the layout cone `isometry-genet` reached around the shared host for. Moves the host onto `cambium-genet-winit-host` (the woodshed-donated assembly), pins genet at mere main's revision, moves the board gestures into view handlers, adopts `caret_text_field` for the whisper and search lanes, and needs one small genet change for right-click. Gates M0-M4; the closing genet commit and pin bumps are Mark's. |
| [2026-08-08_protocol_hardening_plan.md](2026-08-08_protocol_hardening_plan.md) | **Active, second in the audit order behind the host migration. H0 and H1 landed 2026-08-08.** Versioned `Intent -> Resolved` for doorway transitions and overmap travel (replacing `Traveled { token }` peer derivation, a live resolve-once violation); protocol version, request identity, idempotency, and unsupported-version refusal on the envelope; the late-join replay receipt, now taken. Gates H0-H2, H2 open. |
| [2026-08-08_stickleback_migration_plan.md](2026-08-08_stickleback_migration_plan.md) | **Planned, second in the audit order; not yet started (its own status: plan, no K gate landed).** `campaign_sync.rs` off hand-assembled `LogSync`/`SyncedSpace` onto Stickleback `JoinedSpace`; Isometry keeps domain grammar, authorization, materializer, tactical sequencer; the no-second-runtime gate governs. Gates K0-K2. |
| [2026-08-08_extracted_receipts.md](2026-08-08_extracted_receipts.md) | The archive pass's extraction ledger: every residue from the ten archived plans (unmet headed/network receipts, N3, campaign-pack splits, worldbuilding residue, C7 receipts, the preserved diagonal ruling, the exploration headed receipt), each pointing where it lands. |
| [2026-08-02_overmap_presentation_plan.md](2026-08-02_overmap_presentation_plan.md) | **Rewritten to the wing design record 2026-09-26 (ruling 280): sites are the world map's nodes held as adjacency and discovery is knowledge by reach, through the VTT overlay plan's V2, not authored `at` positions or `party_known`; source-time restated as four readings of the reach field (new §3.5); the overmap stays a far view (ruling 212).** **Active**, with two audit prerequisites reopened: a neutral region-paint seam (sprigging's `GraphCanvas` privately owns paint order; Mesocosm's minimap is the second consumer justifying the neutral layer) and hulls derived from final displayed positions incl. overrides, with uniform-position/unplaced/override/parallel-route/headed receipts. Also carries the recorded product direction: **source-time as a feature** (believed-then vs known-now vs retconned), the wing's claim carrier at campaign scale. |
| [2026-07-09_shared_authority_and_collaborative_building_plan.md](2026-07-09_shared_authority_and_collaborative_building_plan.md) | Re-scoped 2026-08-08: the **no-second-runtime sequencing gate stands**; the earlier tiers (host-owned stores, peer Lua revalidation, secrets, commit-reveal) are superseded by the Stickleback migration plan. Kept for the gate and the campaign grammars. |
| [2026-07-08_environmental_surfaces_plan.md](2026-07-08_environmental_surfaces_plan.md) | **Rewritten 2026-09-26 as a VTT note (wing design record ruling 312), not a lane.** Environment is the sim's field on places, moved by agentless processes and diffused by the one mechanism (sim plan §2.4, §3.3); a ruleset at a battlemap reads the field and the cells' conditions and adjudicates by its own rules through the handoff (V3), never owning spread; the Larian interaction matrix is a set of process definitions a world declares, not a ruleset dial. The 2026-07-08 tile-layer plan is in git history. |
| [2026-07-07_optional_intelligence_vision.md](2026-07-07_optional_intelligence_vision.md) | Vision record, **parked**; refresh authority and model assumptions only when activated. |

## Archive

`archive_docs/2026-10-06/`: the Cleromancy generator selection record,
superseded when Cleromancy left the VTT as a stale dependency of marginal
benefit (wing design record ruling 600, amending 298). `>choose` is the VTT's
own seeded draw (ruling 323), the default path since 2026-09-26.

`archive_docs/2026-09-18/`: the runtime profile plan, retired by the W1
evaluation of every active plan against the wing design record (accepted in
full by its ruling 31, 2026-09-18): a product-owned `isometry-runtime` is a
second renderer for the same board, where the record's §4.2 and §2 give no
product a renderer. Its landed map-and-token to body binding table and its
accepted-event mirror into conatus survive one tier down, as a stack adapter.
*(2026-09-27: the crate retired, wing design record ruling 299; neither
piece had reached the stack, and ruling 346 documents one binding shape in
conatus's docs, built when Mark says.)*

`archive_docs/2026-09-04/`: the perf and cambification plan, retired with every
done-condition met. Its two last live items closed on 2026-09-03 (the search
and whisper text fields onto `caret_text_field`) and 2026-09-04 (the file-size
debt: the last seven over-ceiling files split, nothing in the repo above 600
lines). The 600-LOC ledger and the rule for how a file splits moved to
`CLAUDE.md` beside the ceiling itself rather than retiring with the plan.

`archive_docs/2026-08-08/` (audit pass; residues in the extracted-receipts
ledger): bootstrap (I0-I6 landed), next-horizons landscape, board
narration (N1/N2 landed), viewport/windowing, campaign packs (bulk
landed), worldbuilding (W0-W5 landed), adjudication (complete; law
preserved in the protocol plan), gameplay roadmap (C1-C7, C9a landed),
tile geometry seam (diagonal ruling preserved in the ledger), exploration
mode (E0-E6 landed).

Retired plans go to `archive_docs/<YYYY-MM-DD>/`, as the folders above show.

- [Functional generation](../mesocosm/design_docs/2026-09-09_functional_generation_plan.md): shared charge networks, operators, body bindings, and generator proposal carriage; first slice implemented and tested locally.
- [Glyph canon and divinity](../mesocosm/design_docs/2026-09-26_wing_organs_plan.md) (the general model's §7.4 until 2026-09-26, ruling 310): configurable world vocabulary, provenance and ordered reacquisition across lives, journey constraints, fixed-period divine power, and the shared kernel's first Mesocosm consumer.
