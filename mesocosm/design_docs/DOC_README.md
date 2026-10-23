# design_docs Index

Mesocosm's documents, and the wing's shared plans and briefs. The wing's
records (founding record, design record, case register, doctrine) live at the
[wing index](../../design_docs/DOC_README.md), which links all three products. Per
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
- A motion receipt must include actual intermediate frames and contact counts;
  a settled capture or a synthetic IK test does not qualify ordinary gait.
- For visual anatomy, verify developed geometry and the captured silhouette:
  disjoint part envelopes can still merge from the game's camera.
- Resolve declared body boxes by actual dimensions and terrain colours by the
  world's material ids. An undrawn volume tag is not a shared shape, and a
  compact material id has no universal colour (2026-10-19 play-view repair).
- Once a physical provider is configured, missing root poses withdraw that
  body's picture. A presentation grid may not fill a missing physical binding.
- New whole-body play checks rotated occupied parts, window bounds, actual
  ground contact and other admitted bodies. Native births supply actual child
  identities; a refused placement keeps the biology without inventing a pose.
- Food-directed walking scores the need served by an eligible native meal.
  Exact placed identities and native poses bound physical reach; a cohort may
  not inherit its representative's position for individual meals.
  New physical foraging reads living sensory sources and current terrain before
  selecting remote food. Cached geometry carries no biological or awareness result.
- A zero-thickness anatomy part is empty geometry with valid attachments;
  it must not turn every drawable part of that organism into a fallback.
- A voxel material is a compact id into saved world-local definitions. Dynamic
  scalar fields remain separately admitted planes, and each field names its
  consumer, honest dimension, cadence, sources/sinks, conservation rule, and
  scale reduction before the resident machinery carries it.
- Conatus schedules spatial systems and owns its resident allocations and
  leases. The host owns shared-device lifecycle, inter-tenant submission order,
  read epochs, completion, presentation, and recovery.

## Active docs

Whole-site habitat work is tracked in the [presentation continuation](2026-09-11_orthographic_voxel_presentation_plan.md#whole-site-habitat-2026-10-22): full territory, individual food/body placement, native local physics, adaptive terrain paging and the unchosen moving/profile receipt are qualified (2026-10-23). Cold Places and forecasts remain costly; distant readability remains open.

| Doc | For | Status |
| --- | --- | --- |
| [2026-10-23_senses_foraging_plan.md](2026-10-23_senses_foraging_plan.md) | Living sensory parts, cached terrain sight and native food choice. | 2026-10-23: qualified; 129 checks, four older replays, first clock draw with five verified sight-directed steps and 24 moving frames, 17 with supported swing. Route quality remains open. |
| [2026-10-22_foot_placement_plan.md](2026-10-22_foot_placement_plan.md) | Native paid-foot placement, support on slopes, spacing and an ordinary walking capture. | 2026-10-22: shared native/rendered chain geometry, supported foot swing captured in the optimized build; 481 checks and old/new replay hashes pass. |
| [DOC_POLICY.md](DOC_POLICY.md) | A link to the repository's one documentation policy. | Since 2026-10-06 (ruling 615). |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Mesocosm's goals and pillars. | Maintainer-owned. |
| [2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md) | W1: every plan evaluated against the design record. | Ruled and applied 2026-09-18 (ruling 31). |
| [2026-09-18_sim_prior_art_brief.md](2026-09-18_sim_prior_art_brief.md) | Prior art for the simulator, by design question. | Brief for W2, 2026-09-18. |
| [2026-09-22_sim_plan.md](2026-09-22_sim_plan.md) | The sim, W2: Isocosm's schema, processes, record and phases S1 to S10. | 2026-10-18: checkpoints 9 to 11 certified and A1 consumer reproduction passes; the ecology phases S8 to S10 and tuning are next. |
| [2026-10-22_body_appearance_plan.md](2026-10-22_body_appearance_plan.md) | Bodies that read: tissue colour, native shapes and proportions, paid chains, blending per connection, actuation by area and mass by volume. | 2026-10-22: B2 native proportions captured across five clock draws; matched walker 42 to 26 mm, 21 supported swing frames, 119 focused checks and 20/20 three-epoch A1 draws. Curved plate contents, B4/B5 open. |
| [2026-10-19_sim_execution_plan.md](2026-10-19_sim_execution_plan.md) | The sim's execution model: waking only what changed, parallel evaluation, and the near/far crowd tier, with design, implementation and qualification kept apart. | 2026-10-19: specced and decided (draws by actor, statistical far tier, roots plus one hop, speed first); nothing built. |
| [2026-10-14_sim_host_plan.md](2026-10-14_sim_host_plan.md) | One embeddable sim host (`isocosm-host` as a library) under three uniform product hosts: founding, drive and pace, live input, saves; the Mesocosm head first. | 2026-10-20: shared geometry, authenticated forecast sources and batched birth refusals verified; crowded round 34.1 to 10.1 s, resumed epoch 110.1 to 82.6 s; ordinary median 5.34 s and active-update latency remain open. |
| [2026-10-14_stock_title_plan.md](2026-10-14_stock_title_plan.md) | Native tests for the society layer (owned stock, projects, custody, party, institutions, polities, goals) and its `society` feature gate (886, 893, 894): phases with done-conditions and planted faults. | 2026-10-14: gate and T1 to T6 (placement) landed locally (48 tests, 51 planted faults); party statements, strain and produced titles, and direct-command consumers open. |
| [2026-10-08_families_reexpression_plan.md](2026-10-08_families_reexpression_plan.md) | Re-expressing the legacy sims in Isocosm's process definitions, family by family (rulings 192, 591, 656). | 2026-10-18: migration integrated and legacy Mesocosm and Eponym deleted; only legacy/campaign remains until V2. |
| [2026-09-22_aggregation_research.md](2026-09-22_aggregation_research.md) | The reduction literature checked, and the executable boundary of the sim's first implementation. | Research, applied 2026-09-22. |
| [2026-10-02_anatomy_brief.md](2026-10-02_anatomy_brief.md) | Bodies: parts and cells, the function catalogue, organ systems, matter's place, how parts arrive, wounds. | 2026-10-04: checkpoint 8 built how parts arrive. |
| [2026-10-06_balaur_wing_brief.md](2026-10-06_balaur_wing_brief.md) | Balaur read against the wing's rulings. | 2026-10-06: read; its forks ruled as 604 to 607. |
| [2026-10-06_scenevm_wing_brief.md](2026-10-06_scenevm_wing_brief.md) | Eldiron's scenevm read against kiss3d as a tenant and balaur's structure; what is worth learning from it. | 2026-10-06: read; a donor of techniques only (ruling 622). |
| [2026-09-25_mesocosm_overlay_plan.md](2026-09-25_mesocosm_overlay_plan.md) | Mesocosm as the first overlay (W5). | 2026-10-18: M0 and M1 done, M3 and the interim M4 are the directing plan's; full M4 open. |
| [2026-10-08_directing_interim_m4_plan.md](2026-10-08_directing_interim_m4_plan.md) | Directing (M3) and the interim M4 at site grain: the bond, the deliberative choice, the readings, the switch. | 2026-10-18: D1 to D5 built on native Isocosm; consumer reproduction, certification and the headed three-epoch run remain. |
| [2026-09-26_wing_organs_plan.md](2026-09-26_wing_organs_plan.md) | The wing's two organs no single game owns: the hagioglyph and the impresa. | 2026-10-10: shared kernels and native effects; higher rungs parked, 406 carryover refinement still partly designed. |
| [2026-09-26_body_binding_plan.md](2026-09-26_body_binding_plan.md) | One binding adapter keeping critters and tokens against conatus bodies. | 2026-10-19: native whole-body play connection uses the spatial clock; generic Conatus binding remains unbuilt. |
| [2026-09-15_isomere_plan.md](2026-09-15_isomere_plan.md) | isomere: the wing's GUI layer and mode host. | 2026-10-18: M0 to M5 landed and M6 deferred; the mode host awaits runtime and headed two-window acceptance. |
| [2026-09-11_orthographic_voxel_presentation_plan.md](2026-09-11_orthographic_voxel_presentation_plan.md) | The wing's orthographic voxel presentation, lanes L1 to L10. | 2026-10-23: whole sites, dispersed actual bodies/food, overlapping native physics windows and adaptive terrain captured and profiled; 188 focused checks and four replay hashes pass. Cold Places, forecasts, distant readability and load dynamics remain open. |
| [2026-09-15_glyph_expression_plan.md](2026-09-15_glyph_expression_plan.md) | How a canon's glyphs come to be expressed by traits. | 2026-10-10: effects native, trait catalogue folded into §10 (801); higher glyph/divinity rungs parked (279, 673). |
| [2026-09-09_functional_generation_plan.md](2026-09-09_functional_generation_plan.md) | Functional networks, operators and generation: constructions, cantrips, exchange costs. | 2026-09-28: composition designed (404 to 407); no implementation lane open. |
| [2026-08-05_place_graph_engine_plan.md](2026-08-05_place_graph_engine_plan.md) | The spatial spine: places derived from the volume. | 2026-10-19: narrow Mesocosm whole-body SP7 connection; SP1 to SP5 built, SP6 waits on T2, broader SP7 and SP8 remain open. |
| [2026-08-14_resident_views_composition_plan.md](2026-08-14_resident_views_composition_plan.md) | How the voxel world composes with Burn/CubeCL, the tracer, collision and admitted fields on one device. | 2026-10-18: the sun is the first admitted field, built and merged (cases 971 to 984); SP6's nisus join open. |
| [2026-08-18_vessel_briefs_and_presentation.md](2026-08-18_vessel_briefs_and_presentation.md) | The ratified vessel briefs and camera rulings. | Ratified 2026-08-18; direct control superseded by directing (rulings 60, 175). |
| [2026-08-18_engine_ecology_rulings_and_review.md](2026-08-18_engine_ecology_rulings_and_review.md) | Engine and ecology rulings, reviewed against the live code. | Review through 2026-08-26; sharing audit 2026-09-09 (§7). |
| [2026-07-31_wing_phenotype_contract_plan.md](2026-07-31_wing_phenotype_contract_plan.md) | Wing construction, embodiment and continuity: body identity across games. | 2026-10-18: native development and the scene connection exist; incarnation mapping and condition carryover open. |
| [2026-08-01_epoch_boundary_plan.md](2026-08-01_epoch_boundary_plan.md) | The epoch boundary: significance, speciation and what youth costs. | 2026-10-18: native boundary, review, revision and speciation implemented; reproduction and certification remain. |
| [2026-08-29_elements_and_traits_memo.md](2026-08-29_elements_and_traits_memo.md) | How a generated vocabulary becomes real: typed matter, coefficients, exchange. | Memo; material scheme ruled 2026-09-02. |
| [2026-08-29_traits_and_perception_brief.md](2026-08-29_traits_and_perception_brief.md) | Traits, incorporation cost and trait-relative perception. | Design brief, refreshed 2026-09-01. |
| [2026-08-29_forms_of_life_brief.md](2026-08-29_forms_of_life_brief.md) | Composable animal, plant, fungal and microbial forms of life. | Research brief; 2026-10-01: its reproduction axis ruled (447 to 450). |

## Archive

Each archived file carries its own paragraph saying why it moved and what
was carried where. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.

- [`2026-10-18/2026-10-10_lane_queue_plan.md`](archive_docs/2026-10-18/2026-10-10_lane_queue_plan.md), [`after_pass`](archive_docs/2026-10-18/2026-10-10_after_pass_plan.md), [`assembled_certification`](archive_docs/2026-10-18/2026-10-13_assembled_certification.md) and [`plan_review`](archive_docs/2026-10-18/2026-10-10_plan_review.md): the process plans of the 2026-10-10 push, retired by Mark on 2026-10-18; their behavioural open items carried to their owning plans.
- `2026-10-18/*_retired.md`: what the 2026-10-18 cleanup moved out of each active plan and of the design record, verbatim.
- [`2026-10-10/2026-09-15_trait_catalogue_plan.md`](archive_docs/2026-10-10/2026-09-15_trait_catalogue_plan.md): the trait catalogue, never built; folded into the glyph expression plan's §10 (801).
- [`2026-10-10/2026-09-01_dev_tools_plan.md`](archive_docs/2026-10-10/2026-09-01_dev_tools_plan.md): dev tools, DT0 to DT4 landed in the host; folded into the record's W4, the bench (801).
- [`2026-10-10/2026-08-02_views_founding_plan.md`](archive_docs/2026-10-10/2026-08-02_views_founding_plan.md): adapter-first UI and the minimap, first slice landed; folded into the presentation plan's L10 (801).
- [`2026-10-10/2026-09-16_isoscape_family_plan.md`](archive_docs/2026-10-10/2026-09-16_isoscape_family_plan.md): worldgen as a family and deep time; superseded (793), deep time native and the lift in isometer-space, its open items in the sim plan's S5.
- [`2026-10-10/2026-08-29_scale_plan.md`](archive_docs/2026-10-10/2026-08-29_scale_plan.md): the scale ladder, S1 landed on legacy; superseded by residency, native aggregation and L10 (793), S5 carried to the sim plan.
- [`2026-10-10/2026-08-28_played_slice_plan.md`](archive_docs/2026-10-10/2026-08-28_played_slice_plan.md): Mesocosm's first played slice; superseded (793), retired into M3 (196), built as directing's D1 to D5.
- [`2026-10-10/2026-07-31_phenotype_plan.md`](archive_docs/2026-10-10/2026-07-31_phenotype_plan.md): Mesocosm's body rules and VB lanes, P1 to P4 landed on legacy; superseded by native development, mosaic and revision (793).
- [`2026-10-10/2026-09-16_soil_cycle_plan.md`](archive_docs/2026-10-10/2026-09-16_soil_cycle_plan.md): the soil cycle, S1 landed on legacy; superseded (793, 802), M1 to M4 now the sim plan's S10.
- [`2026-10-10/2026-09-04_trophic_grammar_plan.md`](archive_docs/2026-10-10/2026-09-04_trophic_grammar_plan.md): PE4's trophic grammar, TG1 to TG3a landed and native; superseded (793, 802), TG3 to TG7 now the sim plan's S8.
- [`2026-10-10/2026-08-31_playable_ecology_plan.md`](archive_docs/2026-10-10/2026-08-31_playable_ecology_plan.md): Mesocosm's integration plan, PE0 to PE3 landed on legacy; superseded (793, 802), PE4 to the sim plan's S8 and S9, PE5 to checkpoint 11, PE6 to S4, PE7 to M4 and D4.
- [`2026-10-10/2026-10-06_state_witness_plan.md`](archive_docs/2026-10-10/2026-10-06_state_witness_plan.md): the state hash onto the family's witness, H1 and H2 landed 2026-10-08; done (ruling 793).
- [`2026-10-10/2026-09-22_family_rename_plan.md`](archive_docs/2026-10-10/2026-09-22_family_rename_plan.md): the family rename, R0 to R5 landed 2026-09-24; done (ruling 793).
- [`2026-10-10/2026-09-14_isometer_family_plan.md`](archive_docs/2026-10-10/2026-09-14_isometer_family_plan.md): the isometer family, steps 1 to 11 landed 2026-09-15; done (ruling 793).
- [`2026-10-10/2026-09-14_isometer_extraction_plan.md`](archive_docs/2026-10-10/2026-09-14_isometer_extraction_plan.md): `Section` into `shared/isometer`, L9 closed 2026-09-14; done (ruling 793).
- [`2026-10-10/2026-08-30_default_creatures_plan.md`](archive_docs/2026-10-10/2026-08-30_default_creatures_plan.md): the default roster, DC1 to DC4 landed; done (ruling 793), its residue in the after-pass plan's A4.
- [`2026-10-10/2026-08-29_terrarium_dynamics_plan.md`](archive_docs/2026-10-10/2026-08-29_terrarium_dynamics_plan.md): the terrarium dynamics series, TD1 to TD11, closed 2026-08-30; done (ruling 793).
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
