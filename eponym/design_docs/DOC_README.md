# design_docs Index

*Names, 2026-09-22 (wing design record, rulings 109 and 110): the sim and the family are Isocosm, the second-person game is Eponym (formerly Paredros), the tabletop is Isocosm: VTT, with Isometry retired as a product word and kept only as the plain technical prefix of its crates, and Mesocosm is unchanged. Verbatim rulings, quotations and code paths keep the old words until the rename lane lands.*

**Location, 2026-09-09:** Eponym's product document catalogue within the
[Isometry wing index](../../design_docs/DOC_README.md). Shared wing documents
live in [Mesocosm's catalogue](../../mesocosm/design_docs/DOC_README.md) inside
the same Git repository.

Canonical index for `design_docs/`. Per DOC_POLICY §5, this file wins over
any other index and is updated in the same session as any doc change.

## Working principles for AI assistants

- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase and the sibling repos, not doc-to-doc
  consistency.
- Plans carry done-conditions, not time estimates.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions, do
  not edit unasked.
- Wing-level architecture lives in the Mesocosm repo at
  `mesocosm/design_docs/2026-07-30_games_wing_founding.md` and is cited, never
  copied. The three pipeline laws there govern anything crossing between
  games.
- **The invariant is care granularity, not metaphysical person purity**
  (relaxed 2026-07-30; revised 2026-08-13; wing founding record §1). Eponym
  is care for **individuals**: particular others you know. Ordinary play stays
  with one named creature until death. Control may shift through an explicit
  world event or optional player rule, with its consequences recorded. Drift
  means free roster control or care widening to a squad you administer, which
  is Isometry's granularity.

## Active docs

The native package and directory are `eponym-client`: `paredros-client` as
of 2026-09-13, renamed by the family rename's R2 on 2026-09-24.
Historical receipts retain `paredros-room`; current run commands use the new
package. The `room` binary and existing environment variables retain their names.

**Implementation direction, 2026-09-09:** build connected functional systems;
the crossing is an optional fixture. The following plans own the next wiring
dependencies while the execution plan retains F0-F8 semantic milestones:

- [Functional loops and wiring](2026-09-09_functional_loops_plan.md): session
  authority, injury and directional combat, building, saves and continuation;
  bounded J0 body-sheet safety and J1a controlled-session persistence implemented
  locally; J1a passes 35 library + 3 integration tests. B1 timed limb contributions
  now pass 9 focused model tests, one native handler test and an automated
  captured/presented smoke run. B2 strike adjudication and target consequences
  now pass 104 world tests, 3 native handler tests, an automated window smoke
  and the all-features/all-targets workspace compile. Physical input and the
  full host/contact join remain open. B3 adds atomic session fall/rest with
  anatomy and prepared-action repair, plus native dressing pickup and rest;
  110 world tests, 5 native tests and a reviewed automated window smoke pass.
  J1b adds world-owned fractional terrain motion and v5 archives with v3/v4
  compatibility. 119 world tests, 5 native tests, the full workspace compile
  and a reviewed automated window smoke pass; remaining contact limits are in
  the plan. B4 adds revisioned precise combat geometry; 126 world tests,
  5 native tests and a reviewed automated window smoke pass. The same plan records
  why anatomical stance shapes require explicit support roles first. J1c adds
  explicit support/envelope roles, proportional locomotion speed after support
  loss, native support inspection, and v6 archives retaining older replay;
  132 world tests, 6 native tests, full workspace compile and reviewed automated
  window smoke pass. Physical input acceptance remains open.
- [World conditions and authored laws](2026-09-09_world_conditions_plan.md):
  independent skill/risk surgery, causal composition, proposed stored charge and
  sympathetic coupling, and explicitly scoped rules adapters; planned.
- [Memory and remembrance](2026-09-09_memory_and_remembrance_plan.md):
  observer-relative answers, personal preferences, bounded recall, durable
  history, checkpoint/retention strategy and Hagiograph; planned, with a measured
  20,004-intent equipment-history save baseline. Stage F3b5, the canon
  revision (founded as "hagioglyph"; that word names the whole divinity
  organ since 2026-09-15, general model §7.4), landed
  2026-09-14: canon revisions in accepted history with period, promotion and
  authored causes, founding versus live glyph effects in the reading and the
  session journal, and GameSave version 7.
- [Genet document host](2026-09-13_genet_document_host_plan.md): the
  presentation join. One played session presented through genet and netrender
  with a Eponym scene producer over the shared tracer and live body renderer,
  DOM panels reading the same `GameState`, and scenario acceptance; consumes
  Mesocosm's presentation plan rulings and lane L7. P0 to P4 landed
  2026-09-14: aligned pins, a renderling-free scene producer over the shared
  tracer and live body renderer, the `session` document host with sheet,
  equipment, status and acquisition journal panels, the shared
  `wing-scenario` crate promoted from Mesocosm's bench with a Eponym
  acceptance scenario, and Eponym as the second wing-glyphs consumer.
  The shared-depth scene crate (`wing-scene`) is Mesocosm's lane; P1 is
  the duplicate to retire onto it. Physical input acceptance remains open.
  The plan also carries the **body sheet's retirement rationale**
  (2026-09-15, the isomere plan's M5): what the netrender-drawn
  `body_sheet` was, why a second GUI toolkit existed, what replaced each of
  its capabilities, and what was deliberately not carried over.

**Current design focus, 2026-09-08:** the founding plan's
[denizen generation and character-sheet proposal](2026-07-30_paredros_founding_plan.md#borg-generation-techniques-and-the-character-sheet)
specifies classless capabilities, historically transmitted traditions,
alternative anatomical technique bindings, deliberate learning, and inventory
mapped to the body. The first implementation slice is an authored three-lives
example and read-only technique query, now implemented locally and covered by
the combined 43-test gate, using existing subject/revision and
part addresses. A separate native read-only Body/Actions inspector is now
implemented locally with a combined 52-test gate; durable equipment/learning
remain later joins. A bounds-derived, selectable body schematic with a retained
list alternative is now implemented locally with 61 focused tests and six
visually reviewed native captures. Durable, revision-addressed anatomy admission
is implemented locally with a 79-test gate; the authored comparison remains
read-only. Reconciliation and dressing attachment are implemented locally
with a combined 93-test gate. The native live-equipment join now passes
101 combined tests plus three reviewed native captures and a mouse/keyboard
check: a fixed named subject, admitted anatomy and owned dressings, with
attach/detach intents and a separate authored comparison mode. This is
still a wiring probe rather than the joined adventure. Explicit immutable
Save/F5 and latest-save Load/F9 now pass 107 combined tests and a two-process
save/reopen/load check with reviewed composited captures. The private Cargo
cache bypassed shared-cache contention. Physical save/load input acceptance
remains open because the computer-use helper timed out. Startup and exit do
not automatically load or save. `PAREDROS_EQUIPMENT_SAVES` selects storage.
**Retired 2026-09-15:** every surface in the last four sentences was the
netrender-drawn `body_sheet`, which the `session` document host now covers.
The bin, its window and its two-process persistence check are gone; the
rationale is in the genet document host plan and the store survives as
`eponym_client::equipment_store`.
Current world saves use version 5, read versions 3 and 4, and reject versions 1 and 2.
The execution plan
records the independent G/release crossing
fix and unresolved presentation feedback. Charge and
inhabitants are deferred during this design pass. Historical entry and
inheritance policy live once in the Mesocosm wing founding record.

The founding plan's **2026-09-05 player-experience proposal** covers embodied
action, biological abilities, progression, communication, knowledge surfaces,
structural world differences, and bounded sky-organism/topology experiments.
The execution plan translates it into a proposed playable sequence alongside
F3b1's evidence-to-answer join. Its
[damaged crossing fixture](2026-08-07_paredros_execution_plan.md#first-encounter-the-damaged-crossing)
specifies the player flow, two bodies, local world rules, partial witnesses,
knowledge surfaces, and staged acceptance. Dry movement/contact is the first
implementation slice and now has an authored `crossing` executable consuming
Conatus character movement, with local action/replay rules and a shared-device
Renderling/Netrender host. Player acceptance remains open. The wider encounter
and social/charge proposals remain unimplemented and open to design discussion.

| Doc | What it is |
| --- | ---------- |
| [DOC_POLICY.md](DOC_POLICY.md) | Documentation governance |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Product goals and pillars (maintainer-owned, revised by instruction 2026-08-13): one named life in a persistent generated world; autonomous inhabitants; control changes only through death, an explicit world event, or an optional player rule; culture has pointable causes. |
| [2026-09-25_eponym_overlay_plan.md](2026-09-25_eponym_overlay_plan.md) | **W5 plan, drafted 2026-09-25 at Mark's word ("Overlay plans to RPG"); E0 done 2026-09-26 with all seven of §6's decisions ruled, E1 to E4 proposed, not opened; ruling 231 placed this overlay side by side with the VTT's after Mesocosm's M3, rulings 232 to 235 settled the handoff, the solver's home, the start in time and how a first life begins, rulings 238 and 239 the death with no bonded companion and absorption with driving built only on Isocosm, and rulings 241 and 242 posing as a telling's manner and two peers in E4.** Eponym as a game overlay over the Isocosm sim, from the wing design record's §5.6 (rulings 185 to 187), ruling 60 (driving, with directing composed on it by opinion), ruling 130 (diegetic notes) and rulings 152 to 156, mirroring the Mesocosm overlay plan: the profile (domain, ensemble, mechanics, driving controls, the zoom camera, the moment), the played loop through a death and a succession, Eponym's side of the overlay contract as a module in `shared/isocosm-overlay`, where the contract opens the actuation of one body to this game alone, today's seventeen game intents, two world intents and four control intents mapped onto it, what of `eponym-world` and `eponym-social` Isocosm would absorb, and the forks, all ruled: the handoff and the solver's home (232, 233), the start in time (234), how a first life begins (235), the death with no bonded companion (238), absorption and driving only on Isocosm (239), posing as a telling's manner (241), two peers in E4 (242) and which overlay is W5's second (231). |
| [2026-07-30_paredros_founding_plan.md](2026-07-30_paredros_founding_plan.md) | **Under rewrite per W1 (2026-09-18, ruling 31).** Vessel 2's active founding record, revised 2026-08-13: one embodied life among autonomous named creatures; persistent settlements, dungeons, ruins, and surface/underground places; cooperation without party control; construction as a world verb; causal culture; recorded and socially contested body continuity; composable subject/body/role/lineage facts; wing license split, tone, and Nemesis-patent constraint. Its P0-P5 phase section is preserved as superseded history. |
| [2026-08-07_paredros_execution_plan.md](2026-08-07_paredros_execution_plan.md) | **Under rewrite per W1 (2026-09-18, ruling 31).** **The executable plan; F3 active and F3a landed 2026-08-26.** S0-S3 remain landed foundation receipts rather than the required game loop. Future ordering follows fundamental layers: persistent world, one embodied life, other autonomous lives, memory/standing, coordination, material life, settlement/culture, danger, and death/continuation. Ordinary control stays with one named creature until death; tag-in is an optional player rule or explicit world process. R4 was decided 2026-08-10. **R1 shared traversal landed 2026-08-20 and moved to its platform owner 2026-08-26:** Mere's `modulus` (renamed from `conatus-brick` 2026-08-28, pinned at `33f9b6b6`, published on crates.io) owns the sparse brick ABI and camera-neutral WGSL DDA; Eponym owns its Ground binding and carries that organ in the default compile path while retaining its own camera and presentation policy. **V1 continuous-zoom residency landed 2026-08-21; V1a cache coherence closed 2026-08-26:** the headed planning scene keeps its 127-voxel visible radius within a 1 MiB exact-page budget and recovers after abrupt zooms. **D1 raymarch depth composed with Renderling closed 2026-08-26:** the tracer writes fragment depth against renderling's stored depth surface, judged by a headed witness-pillar receipt. **RG3e caller-owned Renderling encoding landed 2026-09-05:** the 20-pass room tenant records into one encoder, Eponym submits it once, and the graph receipt retains its 466-colour byte-match while reporting the tenant and graph submissions separately. **V1b stable resident brick cache closed 2026-08-26:** one capacity-fixed 1,791-slot cache under the same 1 MiB budget retargets in place with retained slots, per-brick transition uploads, zero texture or bind-group creation, tracer-validated lease epochs, and byte-identical wgpu allocator reports; the shared-engine consolidation chain in the mesocosm engine review is closed. Larger travel footprints and clipmaps remain consumer-gated on a real footprint exceeding that exact cache. **F0 persistent world closed 2026-08-21:** `eponym-world` owns stable surface/underground slots, site meanings, routes, edits, and regrow-plus-replay persistence. Generic multi-subject movement persists accepted inputs while navigation remains derived. **F1 embodied life closed 2026-08-21:** separate body, item, and movement systems compose through one subject-addressed transition grammar covering naming, needs, perception, inventory, capability, injury, recovery, and death. **F2 other lives closed 2026-08-22:** deterministic site and migration origins, durable projects, and a control-neutral scheduler advance every living subject through that same intent grammar. Unattended rounds leave factual reports and pointable decision causes; population, projects, and simulation restore exactly. **F3a pointable memory and belief landed 2026-08-26:** accepted deeds now feed actor-scoped observations, claims, exact reports, claimant-owned correction, deterministic belief folds, and validated exact replay. F3 remains active for forgetting, adjudication, deception/intent, norms, observer-relative standing, and a consequential answer with its evidence chain. `mesocosm-lens` remains a product presentation adapter, not the owner of shared traversal. |
| [2026-09-09_functional_loops_plan.md](2026-09-09_functional_loops_plan.md) | **Under rewrite per W1 (2026-09-18, ruling 31).** **Status: in progress, 2026-09-13.** The next cross-system wiring lanes over the execution plan’s F0-F8 milestones: J joins the existing owners (session authority, persistence, body inspection), B is bodies, injury and directional action, T is materials, construction and work, S is saving, death and continuation. J0 body inspection, J1a controlled-session persistence, B1 charged limb contributions, B2 cardinal strike adjudication, B3 treatment and J1b fractional terrain motion are implemented; broader contact mechanics, construction and full adventure coordination remain open. Row added 2026-09-18 by W1, which found the plan listed only in this index’s prose, an index-shape defect under DOC_POLICY §6. |
| [2026-09-09_world_conditions_plan.md](2026-09-09_world_conditions_plan.md) | **Under rewrite per W1 (2026-09-18, ruling 31).** **Status: plan (2026-09-09).** Conditions-led fantastical mechanics: the condition, operation, relation and invariant schema under a content-addressed rules revision, independent skill/risk surgery, causal composition, proposed stored charge and sympathetic coupling, explicitly scoped rules adapters, and Eponym’s action-evaluation ownership. The surgery baseline and the stored-charge law set are design proposals; no world-conditions code is implemented. Under the record’s ruling 32 its schema founds the sim’s process definition and its own stop rule is lifted. Row added 2026-09-18 by W1, which found the plan listed only in this index’s prose, an index-shape defect under DOC_POLICY §6. |
| [2026-09-09_memory_and_remembrance_plan.md](2026-09-09_memory_and_remembrance_plan.md) | **Under rewrite per W1 (2026-09-18, ruling 31).** **Status (2026-09-09): plan.** Ordinary individual memory and the later memorial boundary: observer evidence into consequential answers (F3b1), episodic memory and bounded recall (F3b2), preferences and retelling (F3b3), hagiograph promotion and conditional manifestation (F3b4), and the canon revision G5 (F3b5, assessed and ruled a lane 2026-09-14, not started), with the long-lived save strategy and a measured 20,004-intent equipment-history baseline. The plan records that hagiograph is still an unimplemented Mere reservation and that these are inspected seams, not landed memory behaviour. Row added 2026-09-18 by W1, which found the plan listed only in this index’s prose, an index-shape defect under DOC_POLICY §6. |
| [2026-09-13_genet_document_host_plan.md](2026-09-13_genet_document_host_plan.md) | **Status: planned, 2026-09-13; P0 to P4 landed 2026-09-14.** The presentation join, and the only Eponym plan whose done-conditions come from the product’s own session: one played session through genet and netrender with a Eponym scene producer over the shared tracer and live body renderer, DOM panels reading the same `GameState`, the shared `wing-scenario` crate and a Eponym acceptance scenario. Carries the body sheet’s retirement rationale (2026-09-15). Owes L7 a Eponym lane: renderling is still an unconditional dependency and the Pins section is stale. Row added 2026-09-18 by W1, which found the plan listed only in this index’s prose, an index-shape defect under DOC_POLICY §6. |

## Archive

`archive_docs/2026-09-18/`: the R4 extraction review, retired by the W1
evaluation of every active plan against the wing design record
([mesocosm/design_docs/2026-09-18_wing_design_plan.md](../../mesocosm/design_docs/2026-09-18_wing_design_plan.md)),
accepted in full by Mark’s ruling 31 of 2026-09-18. The record contradicts
it at §4.1 and §4.2: its ruling 3 promoted `eponym-identity` to the wing
identity crate, where §4.1 puts identity in dramatis, and its one extracted
seam is netrender tenancy for a renderling tenant L7 retires; its
MIT OR Apache-2.0 license line is stale besides, the crate being MPL-2.0
since 2026-09-03. The target was wrong and the code is fine: netrender’s
`TenantNeeds` and `boot_shared`/`boot_on` survive as stack hosting, and
`eponym-identity` stands as Eponym overlay ids until dramatis takes it
under W3 (record ruling 34). The moved file carries a **Retired 2026-09-18**
section stating that verdict.

- [`archive_docs/2026-09-18/2026-08-10_r4_extraction_review.md`](archive_docs/2026-09-18/2026-08-10_r4_extraction_review.md)
  — R4 decided and executed 2026-08-10: the wing frame adopted symmetrically,
  the tenancy seam pushed up into netrender, `eponym-identity` promoted,
  the consequence grammar refused on principle, mesocosm-mesh already shared,
  place identity joining by the pipeline. Rulings 2, 4 and 5 survive as
  decisions.

Retired plans go to `archive_docs/<YYYY-MM-DD>/`, as the folder above shows.
