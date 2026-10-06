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
- [2026-09-09_games_wing_consolidation_plan.md](2026-09-09_games_wing_consolidation_plan.md): how the three products came into one repository.

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
| [DOC_POLICY.md](DOC_POLICY.md) | Documentation governance. | Canonical core; the one copy under ruling 615. |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | The VTT's goals and pillars. | Maintainer-owned. |
| [2026-10-06_docs_dedup_plan.md](2026-10-06_docs_dedup_plan.md) | The repo-wide docs dedup (rulings 593, 612 to 615). | In progress, 2026-10-06. |
| [2026-09-25_vtt_overlay_plan.md](2026-09-25_vtt_overlay_plan.md) | The VTT's overlay (W5): rulesets over the sim. | 2026-10-03: rulings 538 to 541; V0 and V1 done. |
| [2026-09-15_board_on_isometer_plan.md](2026-09-15_board_on_isometer_plan.md) | The VTT's board on the isometer family. | Lanes landed 2026-09-16; paging integrated 2026-09-27. |
| [2026-09-05_watchtower_plan.md](2026-09-05_watchtower_plan.md) | The ruined watchtower, the first connected campaign. | Rewritten to the record 2026-09-26 (ruling 280); W1 to W10 verified. |
| [2026-09-03_side_panel_diet_plan.md](2026-09-03_side_panel_diet_plan.md) | The side panel's diet: text layout costs and the 187-row receipt. | 2026-09-28: all 187 rows pass at Mere `5ce144ff` / Genet `7b48f94d`. |
| [2026-09-02_genet_host_migration_plan.md](2026-09-02_genet_host_migration_plan.md) | The desktop host onto the shared Cambium genet host. | M0 to M4 landed and committed 2026-09-03. |
| [2026-09-09_games_wing_consolidation_plan.md](2026-09-09_games_wing_consolidation_plan.md) | How the three products came into one repository. | Published 2026-09-09; standalone repositories archived. |
| [2026-08-08_protocol_hardening_plan.md](2026-08-08_protocol_hardening_plan.md) | Versioned intents and resolution for doorway transitions and overmap travel. | H0 and H1 landed 2026-08-08; H2 open. |
| [2026-08-08_stickleback_migration_plan.md](2026-08-08_stickleback_migration_plan.md) | Campaign sync onto Stickleback, gates K0 to K2. | Planned; not started. |
| [2026-08-08_extracted_receipts.md](2026-08-08_extracted_receipts.md) | Residues extracted from the ten plans archived 2026-08-08, each pointing where it lands. | Ledger, 2026-08-08. |
| [2026-08-02_overmap_presentation_plan.md](2026-08-02_overmap_presentation_plan.md) | The overmap as a far view, discovery as knowledge by reach. | Rewritten to the record 2026-09-26 (ruling 280). |
| [2026-07-09_shared_authority_and_collaborative_building_plan.md](2026-07-09_shared_authority_and_collaborative_building_plan.md) | Shared authority and collaborative building: the no-second-runtime gate and the campaign grammars. | Re-scoped 2026-08-08; the gate stands. |
| [2026-07-08_environmental_surfaces_plan.md](2026-07-08_environmental_surfaces_plan.md) | Environment as the sim's field on places, read by a ruleset at the battlemap. | A VTT note since 2026-09-26 (ruling 312), not a lane. |
| [2026-07-07_optional_intelligence_vision.md](2026-07-07_optional_intelligence_vision.md) | Optional intelligence: a vision record. | Parked. |

## Archive

Each archived file carries its own note saying why it moved. Retired plans go
to `archive_docs/<YYYY-MM-DD>/`.

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
