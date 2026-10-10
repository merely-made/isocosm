# design_docs Index

Mesocosm's documents, and the wing's shared ones, which live here once. The
[wing index](../../design_docs/DOC_README.md) links all three products. Per
DOC_POLICY §5 this file is the canonical index for this directory, updated in
the same session as any doc change. Each row is a link, what the doc is for
and a dated status (wing design record, ruling 612); the docs hold the rest.

Names: the sim and the family are Isocosm, the second-person game is Eponym,
the tabletop is Isocosm: VTT (rulings 109 to 111).

## Working principles for AI assistants

- Construction design should establish the relationships among source,
  embodiment, action, history and presentation before selecting implementation
  slices. Keep accepted decisions separate from proposed schemas and cost
  formulas; explainable surprise and powerful combinations are design goals
  (wing design record 404 to 407).

- Follow the question, ruling and verification method restated by Mark on
  2026-09-27 in [session notes §8.2](archive_docs/2026-10-06/2026-09-22_sim_design_session_notes.md#82-the-method-as-practised).
  Its three refinements were accepted in ruling 366 on 2026-09-27.
- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase and the sibling repos, not doc-to-doc
  consistency. This wing's founding record was corrected once already for
  trusting a stale index line over the plan it indexed.
- Plans carry done-conditions, not time estimates.
- The wing founding record's **Game tastes, integrated 2026-09-09** section
  owns the shared influence/preferences record. Preserve the distinction
  between endorsed directions, exploratory mechanics, and verified features;
  each vessel interprets the tastes through its own rules and care granularity.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions, do
  not edit unasked.
- The substrate/system split is load-bearing across the whole wing: one
  substrate, many rule-dressings. Keep it that way in every doc.
- The three pipeline laws (choices-not-morphology, pointable inheritance, no
  homework) govern anything that crosses between games. A violation is a
  design bug, not a preference.
- Reproduction is the individual checkpoint; the epoch boundary is the
  lineage checkpoint. Do not make either one a hidden invocation of the other.
- Trophic warnings, unlock evidence, and epoch explanations derive from the
  same accepted world transitions as the ecology. A projection may summarize
  authority; it may not grow a parallel counter-based answer.
- Materialized individuals and far cohorts are two execution/storage forms of
  one world authority. Their explicit transitions preserve conserved totals,
  sufficient reading state, and every identity the game has made pointable;
  supported far-tier evaluators declare exact invariants and comparison
  envelopes against an all-individual reference; macro trophic views are
  derived rather than a third simulation.
- For visual anatomy, verify developed geometry and the captured silhouette:
  disjoint part envelopes can still merge from the game's camera.
- A voxel material is a compact id into saved world-local definitions. Dynamic
  scalar fields remain separately admitted planes, and each field names its
  consumer, honest dimension, cadence, sources/sinks, conservation rule, and
  scale reduction before the resident machinery carries it.
- Conatus schedules spatial systems and owns its resident allocations and
  leases. The host owns shared-device lifecycle, inter-tenant submission order,
  read epochs, completion, presentation, and recovery.

## Active docs

| Doc | For | Status |
| --- | --- | --- |
| [DOC_POLICY.md](DOC_POLICY.md) | A link to the repository's one documentation policy. | Since 2026-10-06 (ruling 615). |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Mesocosm's goals and pillars. | Maintainer-owned. |
| [2026-07-30_games_wing_founding.md](2026-07-30_games_wing_founding.md) | The wing's founding record: the vessels and their care granularities, the pipeline laws, the shared vocabulary, the game tastes. | Founding record, 2026-07-30; wing-level, cited by every product. |
| [2026-09-18_wing_design_plan.md](2026-09-18_wing_design_plan.md) | The wing as a simulator: the design record of rulings (§0) and the W0 to W5 plan (§11). | Living record. |
| [2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md) | W1: every plan evaluated against the design record. | Ruled and applied 2026-09-18 (ruling 31). |
| [2026-09-18_sim_prior_art_brief.md](2026-09-18_sim_prior_art_brief.md) | Prior art for the simulator, by design question. | Brief for W2, 2026-09-18. |
| [2026-09-22_sim_plan.md](2026-09-22_sim_plan.md) | The sim, W2: Isocosm's schema, processes, record and phases S1 to S6. | 2026-10-09: checkpoint 9 merged at `df6f0527`; checkpoint 10 briefed. 2026-10-10 Q3 lane: native accounts and 806 checked; reproduction cadence gate open, integration pending. |
| [2026-10-10_after_pass_plan.md](2026-10-10_after_pass_plan.md) | Certifying what 732's push moved: every deferred done-condition, its owner and its order (rulings 789 to 792). | In progress 2026-10-10. |
| [2026-10-10_plan_review.md](2026-10-10_plan_review.md) | The plans reviewed after the push: classification, live set, overlaps, order, forks. | Review complete 2026-10-10; forks to Mark. |
| [2026-10-08_families_reexpression_plan.md](2026-10-08_families_reexpression_plan.md) | Re-expressing the legacy sims in Isocosm's process definitions, family by family (rulings 192, 591, 656). | Assessment 2026-10-08; forks to Mark. |
| [2026-10-06_state_witness_plan.md](2026-10-06_state_witness_plan.md) | Isocosm's state hash onto the family's FNV witness, labelled (rulings 607 to 610). | Complete 2026-10-08: H1 (641, 642) and H2 (647 to 653). |
| [2026-09-22_aggregation_research.md](2026-09-22_aggregation_research.md) | The reduction literature checked, and the executable boundary of the sim's first implementation. | Research, applied 2026-09-22. |
| [2026-10-02_anatomy_brief.md](2026-10-02_anatomy_brief.md) | Bodies: parts and cells, the function catalogue, organ systems, matter's place, how parts arrive, wounds. | 2026-10-04: checkpoint 8 built how parts arrive. |
| [2026-10-06_balaur_wing_brief.md](2026-10-06_balaur_wing_brief.md) | Balaur read against the wing's rulings. | 2026-10-06: read; its forks ruled as 604 to 607. |
| [2026-10-06_scenevm_wing_brief.md](2026-10-06_scenevm_wing_brief.md) | Eldiron's scenevm read against kiss3d as a tenant and balaur's structure; what is worth learning from it. | 2026-10-06: read; a donor of techniques only (ruling 622). |
| [2026-09-25_mesocosm_overlay_plan.md](2026-09-25_mesocosm_overlay_plan.md) | Mesocosm as the first overlay (W5). | M0 and M1 done 2026-09-25; M2 is the families plan's share (677); M3 and the interim M4 in the directing plan (681 to 691). |
| [2026-10-08_directing_interim_m4_plan.md](2026-10-08_directing_interim_m4_plan.md) | Directing (M3) and the interim M4 at site grain: the bond, the deliberative choice, the readings, the switch. | 2026-10-10 Q3 lane: 806 native boundary order checked, integration pending; reproduction gate and headed run open. |
| [2026-09-22_family_rename_plan.md](2026-09-22_family_rename_plan.md) | The family rename: Isocosm, Eponym, Isocosm: VTT. | Landed 2026-09-24: R0 to R5. |
| [2026-09-26_wing_organs_plan.md](2026-09-26_wing_organs_plan.md) | The wing's two organs no single game owns: the hagioglyph and the impresa. | Carried 2026-09-26; carryover distinction 2026-09-28 (ruling 406). |
| [2026-09-26_body_binding_plan.md](2026-09-26_body_binding_plan.md) | One binding adapter keeping critters and tokens against conatus bodies. | Ruled 2026-09-26 (346 to 352); documented, not built. |
| [2026-09-15_isomere_plan.md](2026-09-15_isomere_plan.md) | isomere: the wing's GUI layer and mode host. | 2026-10-01: the mode host ruled (442 to 445); waits on Mesocosm's M3. |
| [2026-09-16_isoscape_family_plan.md](2026-09-16_isoscape_family_plan.md) | isoscape: worldgen as a family, the three generation buckets and deep time. | Assessment complete, 2026-09-16; rulings 1 to 14 recorded. |
| [2026-09-16_soil_cycle_plan.md](2026-09-16_soil_cycle_plan.md) | The soil cycle: decay, carrion return and decomposer reach. | S1 landed 2026-09-16 (`ca836e0`). |
| [2026-09-14_isometer_family_plan.md](2026-09-14_isometer_family_plan.md) | The isometer family: lens, render and mesh as neutral components for all three games. | Assessment, 2026-09-14. |
| [2026-09-14_isometer_extraction_plan.md](2026-09-14_isometer_extraction_plan.md) | Moving Mesocosm's `Section` into `shared/isometer` (presentation lane L9). | Assessment, 2026-09-14. |
| [2026-09-11_orthographic_voxel_presentation_plan.md](2026-09-11_orthographic_voxel_presentation_plan.md) | The wing's orthographic voxel presentation, lanes L1 to L10. | 2026-10-01: L10, the detail ladder, briefed. |
| [2026-09-15_glyph_expression_plan.md](2026-09-15_glyph_expression_plan.md) | How a canon's glyphs come to be expressed by traits. | Assessment, 2026-09-15; no code moved. |
| [2026-09-15_trait_catalogue_plan.md](2026-09-15_trait_catalogue_plan.md) | The trait catalogue the glyph expression plan consumes. | Assessment, 2026-09-15; no code moved. |
| [2026-09-09_functional_generation_plan.md](2026-09-09_functional_generation_plan.md) | Functional networks, operators and generation: constructions, cantrips, exchange costs. | 2026-09-28: composition designed (404 to 407); no implementation lane open. |
| [2026-08-05_place_graph_engine_plan.md](2026-08-05_place_graph_engine_plan.md) | The spatial spine: places derived from the volume. | 2026-09-30: SP4 and SP5 designed (412 to 422); SP4 next. |
| [2026-08-14_resident_views_composition_plan.md](2026-08-14_resident_views_composition_plan.md) | How the voxel world composes with Burn/CubeCL, the tracer, collision and admitted fields on one device. | Founded 2026-08-14. |
| [2026-08-18_vessel_briefs_and_presentation.md](2026-08-18_vessel_briefs_and_presentation.md) | The ratified vessel briefs and camera rulings. | Ratified 2026-08-18; direct control superseded by directing (rulings 60, 175). |
| [2026-08-18_engine_ecology_rulings_and_review.md](2026-08-18_engine_ecology_rulings_and_review.md) | Engine and ecology rulings, reviewed against the live code. | Review through 2026-08-26; sharing audit 2026-09-09 (§7). |
| [2026-08-31_playable_ecology_plan.md](2026-08-31_playable_ecology_plan.md) | Mesocosm's integration and technical-architecture plan, PE0 to PE4. | Under rewrite per W1 (ruling 31); PE0 to PE3 landed, PE4 open. |
| [2026-09-04_trophic_grammar_plan.md](2026-09-04_trophic_grammar_plan.md) | PE4's first build: shared typed trophic admission. | TG1 complete 2026-09-05. |
| [2026-07-31_phenotype_plan.md](2026-07-31_phenotype_plan.md) | Mesocosm's body rules and proof plan, with the voxel-body lanes VB0 to VB5. | Under rewrite per W1 (ruling 31). |
| [2026-07-31_wing_phenotype_contract_plan.md](2026-07-31_wing_phenotype_contract_plan.md) | Wing construction, embodiment and continuity: body identity across games. | 2026-09-28: body mapping prepared; adapter unimplemented. |
| [2026-08-01_epoch_boundary_plan.md](2026-08-01_epoch_boundary_plan.md) | The epoch boundary: significance, speciation and what youth costs. | Rewritten to the record 2026-09-26 (ruling 280); partially built. |
| [2026-08-02_views_founding_plan.md](2026-08-02_views_founding_plan.md) | Adapter-first UI, with the minimap as first chrome. | Rewritten to the record 2026-09-26 (ruling 280); first slice landed. |
| [2026-08-28_played_slice_plan.md](2026-08-28_played_slice_plan.md) | Mesocosm's first played slice. | Control rewritten for directing 2026-09-25 (ruling 199); retires into the overlay plan's M3. |
| [2026-08-29_terrarium_dynamics_plan.md](2026-08-29_terrarium_dynamics_plan.md) | The terrarium dynamics series answering the first playtests. | Landed and closed through TD11, 2026-08-31. |
| [2026-08-29_scale_plan.md](2026-08-29_scale_plan.md) | The scale ladder: measured terrain, atlas, place-graph, population and snapshot limits. | Under rewrite per W1 (ruling 31); S1 landed. |
| [2026-08-30_default_creatures_plan.md](2026-08-30_default_creatures_plan.md) | Mesocosm's default creature roster and its gates. | Rewritten to the record 2026-09-26 (ruling 280); DC1 to DC4 landed. |
| [2026-09-01_dev_tools_plan.md](2026-09-01_dev_tools_plan.md) | Dev tools for sitting in a run and interrogating it. | Rewritten to the record 2026-09-26 (ruling 280): the tools move to the bench (W4). |
| [2026-08-29_elements_and_traits_memo.md](2026-08-29_elements_and_traits_memo.md) | How a generated vocabulary becomes real: typed matter, coefficients, exchange. | Memo; material scheme ruled 2026-09-02. |
| [2026-08-29_traits_and_perception_brief.md](2026-08-29_traits_and_perception_brief.md) | Traits, incorporation cost and trait-relative perception. | Design brief, refreshed 2026-09-01. |
| [2026-08-29_forms_of_life_brief.md](2026-08-29_forms_of_life_brief.md) | Composable animal, plant, fungal and microbial forms of life. | Research brief; 2026-10-01: its reproduction axis ruled (447 to 450). |

## Archive

Each archived file carries its own paragraph saying why it moved and what
was carried where. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.

- [`2026-10-06/2026-10-06_wing_datasheets_plan.md`](archive_docs/2026-10-06/2026-10-06_wing_datasheets_plan.md): packs in TOML, the pack JSON check, and every founding and the seeded kinds as datasheets, landed (rulings 623 to 632); its findings handed off.
- [`2026-10-06/2026-09-22_sim_design_session_notes.md`](archive_docs/2026-10-06/2026-09-22_sim_design_session_notes.md): the sim design sessions' notes, 2026-09-16 to 2026-09-30 (ruling 614).
- [`2026-10-06/2026-09-22_sim_plan_progress.md`](archive_docs/2026-10-06/2026-09-22_sim_plan_progress.md): the sim plan's full progress log to 2026-10-06 (ruling 613).
- [`2026-10-06/2026-09-18_wing_design_plan_progress.md`](archive_docs/2026-10-06/2026-09-18_wing_design_plan_progress.md): the design record's full progress log to 2026-10-06 (ruling 613).
- [`2026-09-26/2026-07-30_mesocosm_founding_plan.md`](archive_docs/2026-09-26/2026-07-30_mesocosm_founding_plan.md): ruling 308; Mesocosm's charter is now the overlay plan.
- [`2026-09-26/2026-08-01_processdef_plan.md`](archive_docs/2026-09-26/2026-08-01_processdef_plan.md): ruling 309; its shapes are the sim's process definition.
- [`2026-09-26/2026-08-06_general_model_plan.md`](archive_docs/2026-09-26/2026-08-06_general_model_plan.md): ruling 310; its organs went to the wing organs plan.
- [`2026-09-26/2026-08-07_dependency_ledger.md`](archive_docs/2026-09-26/2026-08-07_dependency_ledger.md): ruling 311; the order lives in the record's §11.
- [`2026-09-24/2026-09-21_wing_readings_docket.md`](archive_docs/2026-09-24/2026-09-21_wing_readings_docket.md): every reading ruled in the record.
- [`2026-09-18/2026-07-30_engine_and_render_lane_landscape.md`](archive_docs/2026-09-18/2026-07-30_engine_and_render_lane_landscape.md): retired by W1 (ruling 31).
- [`2026-09-18/2026-07-31_execution_waves_plan.md`](archive_docs/2026-09-18/2026-07-31_execution_waves_plan.md): retired by W1 (ruling 31).
- [`2026-09-18/2026-08-29_open_rulings_register.md`](archive_docs/2026-09-18/2026-08-29_open_rulings_register.md): retired by W1 (ruling 31).
- [`2026-09-18/2026-09-15_effect_pack_preset_plan.md`](archive_docs/2026-09-18/2026-09-15_effect_pack_preset_plan.md): retired by W1 (ruling 31).
- [`2026-08-07/2026-07-30_body_pipeline_and_host_probe_plan.md`](archive_docs/2026-08-07/2026-07-30_body_pipeline_and_host_probe_plan.md): R0, R1 and R3 complete.
