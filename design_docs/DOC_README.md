# design_docs Index

The repository's entry index, and the VTT's documents. Per DOC_POLICY §5 this
file is the canonical index for this directory, updated in the same session
as any doc change. Each row is a link, what the doc is for and a dated status
(wing design record, ruling 612); the docs hold the rest.

Names: the sim and the family are Isocosm, the second-person game is Eponym,
the tabletop is Isocosm: VTT (rulings 109 to 111).

## Games wing entry points

- [Mesocosm index](../mesocosm/design_docs/DOC_README.md): Mesocosm's index, and the wing's shared documents.
- [Eponym index](../eponym/design_docs/DOC_README.md): Eponym's index.
- [2026-07-30_games_wing_founding.md](../mesocosm/design_docs/2026-07-30_games_wing_founding.md): the wing founding record: vessels, pipeline laws, shared vocabulary.
- [2026-09-18_wing_design_plan.md](../mesocosm/design_docs/2026-09-18_wing_design_plan.md): the wing design record: every ruling (§0), the current architecture, and the W0 to W5 plan.
- [2026-09-22_sim_plan.md](../mesocosm/design_docs/2026-09-22_sim_plan.md): the sim, Isocosm.

## Working principles for AI assistants

- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase, not doc-to-doc consistency.
- Plans carry done-conditions, not time estimates.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions,
  do not edit unasked.
- The substrate/system split is load-bearing: geometry and turns in the
  substrate, rules in system plugins. Keep it that way in every doc.

## Active docs

| Doc | For | Status |
| --- | --- | --- |
| [DOC_POLICY.md](DOC_POLICY.md) | Documentation governance for all three products. | The one copy since 2026-10-06 (ruling 615). |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | The VTT's goals and pillars. | Maintainer-owned. |
| [2026-09-25_vtt_overlay_plan.md](2026-09-25_vtt_overlay_plan.md) | The VTT's overlay (W5): rulesets over the sim. | 2026-10-10: V0/V1 done; V2 open through contract (794, 795, 798); 809 rules native cell/owner metadata, implementation/replication pending; H2/overmap folded (801), legacy/campaign retirement pending. |
| [2026-09-15_board_on_isometer_plan.md](2026-09-15_board_on_isometer_plan.md) | The VTT's board on the isometer family. | 2026-10-10: authored board/paging implemented; sim-off MapTerrain retained, sim-on lifted-site integration belongs to V2 (799). |
| [2026-08-08_stickleback_migration_plan.md](2026-08-08_stickleback_migration_plan.md) | Campaign sync onto Stickleback, gates K0 to K2. | 2026-10-10: K0 to K2 open; campaign_sync still assembles LogSync/SyncedSpace directly. |
| [2026-08-08_extracted_receipts.md](2026-08-08_extracted_receipts.md) | Residues extracted from the ten plans archived 2026-08-08, each pointing where it lands. | Ledger, 2026-08-08. |
| [2026-07-09_shared_authority_and_collaborative_building_plan.md](2026-07-09_shared_authority_and_collaborative_building_plan.md) | Shared authority and collaborative building: the no-second-runtime gate and the campaign grammars. | Re-scoped 2026-08-08; the gate stands. |
| [2026-07-08_environmental_surfaces_plan.md](2026-07-08_environmental_surfaces_plan.md) | Environment as the sim's field on places, read by a ruleset at the battlemap. | A VTT note since 2026-09-26 (ruling 312), not a lane. |
| [2026-07-07_optional_intelligence_vision.md](2026-07-07_optional_intelligence_vision.md) | Optional intelligence: a vision record. | Parked. |

## Archive

Each archived file carries its own note saying why it moved. Retired plans go
to `archive_docs/<YYYY-MM-DD>/`.

- [`2026-10-10/2026-08-08_protocol_hardening_plan.md`](archive_docs/2026-10-10/2026-08-08_protocol_hardening_plan.md): versioned intents and travel as `Resolved`, H0 and H1 landed; folded into the VTT overlay plan's V2 (801).
- [`2026-10-10/2026-08-02_overmap_presentation_plan.md`](archive_docs/2026-10-10/2026-08-02_overmap_presentation_plan.md): the overmap as a far view; folded into the VTT overlay plan's V2, its drawing into L10 (801).
- [`2026-10-10/2026-10-09_mere_bdc89a05_repin_plan.md`](archive_docs/2026-10-10/2026-10-09_mere_bdc89a05_repin_plan.md): the repin onto mere `bdc89a05`, landed 2026-10-09; done (ruling 793).
- [`2026-10-10/2026-10-08_mere_329d60d0_repin_plan.md`](archive_docs/2026-10-10/2026-10-08_mere_329d60d0_repin_plan.md): the repin onto mere `329d60d0`, landed 2026-10-08; done (ruling 793).
- [`2026-10-10/2026-10-07_shared_build_dir_plan.md`](archive_docs/2026-10-10/2026-10-07_shared_build_dir_plan.md): one build directory for the workspaces, landed 2026-10-08; done (ruling 793).
- [`2026-10-10/2026-10-06_stable_repin_plan.md`](archive_docs/2026-10-10/2026-10-06_stable_repin_plan.md): the repin onto stable Burn, landed 2026-10-07; done (ruling 793).
- [`2026-10-10/2026-10-06_docs_dedup_plan.md`](archive_docs/2026-10-10/2026-10-06_docs_dedup_plan.md): the repo-wide docs dedup, done 2026-10-06 (ruling 793).
- [`2026-10-10/2026-09-09_games_wing_consolidation_plan.md`](archive_docs/2026-10-10/2026-09-09_games_wing_consolidation_plan.md): how the three products came into one repository, published 2026-09-09; done (ruling 793).
- [`2026-10-10/2026-09-05_watchtower_plan.md`](archive_docs/2026-10-10/2026-09-05_watchtower_plan.md): the first connected campaign, W1 to W13 verified; done (ruling 793), its follow-ons in the VTT overlay plan's §7.
- [`2026-10-10/2026-09-03_side_panel_diet_plan.md`](archive_docs/2026-10-10/2026-09-03_side_panel_diet_plan.md): the side panel's diet, all 187 rows passing; done (ruling 793), its smallest-display target in the VTT overlay plan's §7.
- [`2026-10-10/2026-09-02_genet_host_migration_plan.md`](archive_docs/2026-10-10/2026-09-02_genet_host_migration_plan.md): the desktop host onto the shared genet host, landed 2026-09-03; done (ruling 793), its Z5 design-height note in the VTT overlay plan's §7.
- [`2026-10-06/2026-08-09_cleromancy_generator_selection.md`](archive_docs/2026-10-06/2026-08-09_cleromancy_generator_selection.md): superseded when Cleromancy left the VTT (ruling 600).
- [`2026-09-18/2026-08-23_runtime_profile_plan.md`](archive_docs/2026-09-18/2026-08-23_runtime_profile_plan.md): retired by W1 (ruling 31); `isometry-runtime` itself retired by ruling 299.
- [`2026-09-04/2026-07-20_perf_and_cambification_plan.md`](archive_docs/2026-09-04/2026-07-20_perf_and_cambification_plan.md): retired with every done-condition met.
- [`2026-08-08/2026-07-05_isometry_bootstrap_plan.md`](archive_docs/2026-08-08/2026-07-05_isometry_bootstrap_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-07_board_to_text_narration_plan.md`](archive_docs/2026-08-08/2026-07-07_board_to_text_narration_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-07_next_horizons_landscape.md`](archive_docs/2026-08-08/2026-07-07_next_horizons_landscape.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-07_viewport_windowing_and_chrome_plan.md`](archive_docs/2026-08-08/2026-07-07_viewport_windowing_and_chrome_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-08_campaign_packs_plan.md`](archive_docs/2026-08-08/2026-07-08_campaign_packs_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-09_worldbuilding_generation_plan.md`](archive_docs/2026-08-08/2026-07-09_worldbuilding_generation_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-14_adjudication_and_representation_plan.md`](archive_docs/2026-08-08/2026-07-14_adjudication_and_representation_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-14_gameplay_roadmap_plan.md`](archive_docs/2026-08-08/2026-07-14_gameplay_roadmap_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-17_tile_geometry_seam_plan.md`](archive_docs/2026-08-08/2026-07-17_tile_geometry_seam_plan.md): the audit pass; residues in the extracted-receipts ledger.
- [`2026-08-08/2026-07-18_exploration_mode_plan.md`](archive_docs/2026-08-08/2026-07-18_exploration_mode_plan.md): the audit pass; residues in the extracted-receipts ledger.
