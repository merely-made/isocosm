# W1: the wing's plans evaluated against the design record

**Date:** 2026-09-18

**Status, 2026-09-18:** all three products evaluated, fifty-seven
documents, and **Mark accepted every recommendation the same day**
(record ruling 31): thirty keep, twenty-one rewrite, six retire. §4's
open items are ruled in the record's §9 (rulings 32 to 34). Application
in progress: each plan gets its one line and the six retirements move to
their archives, one product at a time. Read-only evaluations; nothing in any plan has been
edited by this document's lanes.

**Owns:** W1 of the
[wing design record](2026-09-18_wing_design_plan.md): one row per active
plan in each product's index, its tier by the record's §2 test, whether
the record confirms or contradicts its load-bearing assumption, and a
recommendation. Corrections the evaluations made to the record itself are
folded into the record and noted here by section.

**Does not own:** the record, or any plan's rewrite.

---

## 1. Isometry root, 2026-09-18

Read-only evaluation of every active plan in the tabletop's index against
this record. Rulings on keep, rewrite and retire are Mark's; the
recommendations are the lane's.

| Plan | Tier | Record says | Recommended | Why |
| --- | --- | --- | --- | --- |
| Board on isometer (2026-09-15) | mixed, stack rendering and game overlay | Contradicted: §8 rows 5 and 6, §6, §2; confirmed in part by §4.1, §4.2, §4.3 | rewrite | The lanes landed and the code is good; the done-conditions were parity with the DOM board and a constant, and §3.6 rewrites the geometry under them |
| Games wing consolidation (2026-09-09) | stack | Confirmed, §4.1 | keep | Published and done; decides nothing about the world |
| Watchtower (2026-09-05) | mixed, sim generation and game overlay | W6's height-field premise contradicted by ruling 12 and §3.6; its seeded-pack discipline is ruling 15 confirmed | rewrite | Only the terrain premise fails |
| Side panel diet (2026-09-03) | game overlay | Unaddressed; its done-condition is a chosen size, §6's failure mode by the letter | keep | Cuts landed; restate the target as reachable at the smallest supported display |
| Genet host migration (2026-09-02) | stack, hosting | Confirmed, §4.1 | keep | Exactly §2's boundary |
| Runtime profile (2026-08-23) | stack rendering, wrongly product-owned | Contradicted: §4.2 and §2, a product owns no renderer | retire | A second renderer for the same board. Surviving code: the map-and-token to body binding table and the accepted-event mirror into conatus, as a stack adapter |
| Protocol hardening (2026-08-08) | stack, networking and receipts | Confirmed, §3.8 and Law A | keep | H2 is the only open gate |
| Stickleback migration (2026-08-08) | stack, branching | Confirmed, §4.1 and ruling 7 | keep | Sovereignty over a shared carrier |
| Extracted receipts (2026-08-08) | method | Confirmed, §6 | keep | W1 produces more of these |
| Overmap presentation (2026-08-02) | mixed, sim place graph and game overlay | Contradicted on authored positions by §3.7 and ruling 14; source-time as a feature confirmed as §3.4's reach field | rewrite | The source-time half is the record's own model and is worth more than the presentation half |
| Shared authority (2026-07-09) | stack, federation | Confirmed, §2; §5 reopens the DM question under its gate | keep | The gate is why §5 can stay open safely |
| Environmental surfaces (2026-07-08) | sim, agentless processes, written as one product's tile layer | Contradicted: §3.3, §3.1, ruling 12; also names a crate that moved on 2026-09-15 | rewrite | Nothing landed; the cheapest correction and the first input to W2 |
| Optional intelligence vision (2026-07-07) | mixed | Confirmed, §1 and §4.1's esp row | keep, parked | Name esp when activated |
| Cleromancy generator selection (2026-08-09) | stack, generation | Confirmed, §4.1: ruling 15 already implemented | keep | Not listed in the root index, an index defect |
| PROJECT_DESCRIPTION | game overlay | Confirmed as the source §6 wants, and contradicted three ways inside itself, §9.9 | surface to Mark | Maintainer-owned |

The uncommitted work in the tabletop tree from the board plan's last lane
splits: the revision-and-state ordering fix in `BoardGround::sync`, the
body skip over missing ground and `MapTerrain::size` are real fixes of
§4.3's stale-terrain family and stand under any ruling; the budget module,
its tests, the generator warning, the self-test arm and the modulus
dependency, about 780 lines, are machinery for a limit §8 says should not
exist. The three the lane would act on first: the board plan rewrite, the
runtime profile retirement, the environmental surfaces rewrite.


## 2. Mesocosm, 2026-09-18

Read-only evaluation of every active plan in Mesocosm's index against the
record. The founding record and the design record are exempt; their
disagreements are the record's §9.11. Rulings are Mark's; recommendations
are the lane's.

| Plan | Tier | Record says | Recommended | Why |
| --- | --- | --- | --- | --- |
| DOC_POLICY | method | Confirmed, §6 | keep | Governance |
| PROJECT_DESCRIPTION | game overlay | Contradicted three ways: files the sim's defining behaviour under Speculative (`:64-66`); "shares no genre, schedule, or verbs" (`:52-53`) against rulings 3 and 10; pillar 2 withdrawn by its own founding plan | surface to Mark | Maintainer-owned |
| Mesocosm founding (2026-07-30) | mixed, sim world grammar and game overlay | Confirmed as the overlay §5 wants; contradicted by §2 where it owns world-condition grammar, material flow and trophic strategies | rewrite | Split the world half into the sim; keep metabolize, the epoch loop and care for a species |
| Engine and render lane landscape (2026-07-30) | stack, rendering | Contradicted, §4.2: its live decision is renderling as lead mesh tenant (`:38-41`) | retire | Self-labelled historical; the §8.9 cohesion contract and capability profiles survive as stack rendering |
| Dependency ledger (2026-08-07) | method, ordering | Contradicted, §11: W0 to W5 is the order, and its founding rule is product-first where §2 and §6 are sim-first | rewrite | Six good lanes in the wrong sequence |
| Playable ecology (2026-08-31) | mixed, sim and Mesocosm overlay | Confirmed §1 by "one authority, several derived records" (`:193`) and §3.3 by PE0 and PE4; the PE1 and PE3 checkpoints are overlay | rewrite | Split PE lanes by tier; the sim half is W2's largest input |
| Execution waves (2026-07-31) | method, ordering | Contradicted, §11; already demoted by its own audit | retire | Wave 1's core, runtime, host and projection all landed and stand |
| Phenotype (2026-07-31) | mixed, sim bodies, overlay capability, stack rendering | Confirmed §3.5: "the played critter owns the only BodyDocument, every other organism is one VolumeRef and one half_extent" (`:32-33`) is realize-from-a-distribution stated as a limitation | rewrite | The body noun is the sim's; the capability fold is Mesocosm's; the VB lanes are isometer's |
| Epoch boundary (2026-08-01) | mixed, sim record and Mesocosm board | Contradicted in part, §3.4: significance is abnormality against the world record alone (`history.rs:40-42`), missing the relevance gate; the organ already moved (`record.rs:17-23`) | rewrite | Only the significance test and its owner fail; the board, the reckoning and speciation as an act stand |
| ProcessDef (2026-08-01) | sim, §3.3 first shape | Contradicted by receipt: thirty rule shapes in the whole definition space (`process.rs:391-403`, `:350-358`), no scarcity, cost, foregone or cause-link; status line stale (a pack loader and piccolo host exist in mesocosm-phenotype) | rewrite | Answers §9.7 with evidence |
| Wing phenotype contract (2026-07-31) | stack formats and sim nouns | Contradicted, §4.4 and ruling 16: founds `mesocosm.body/v0` and `mesocosm.chronicle/v0` as wing formats where glTF holds bodies and eidetic the record | rewrite | v1 unimplemented, so the correction is free |
| Views founding (2026-08-02) | game overlay over stack | Contradicted in part, §3.7 and ruling 14: the minimap binds to `Places::at`, a two-dimensional nearest-centre scan (`places.rs:170-181`) | rewrite | Adapter-first posture confirmed by §4.1; only the minimap's source rung changes |
| Place-graph engine (2026-08-05) | sim, §3.7 | Confirmed on derived adjacency (`:89`); contradicted on nodes: `PLACE_SIDE = 3` (`world.rs:90`), asserted interiors (`places/grown.rs:42-45`), a 65-cell heightfield as the source (`relief.rs:7-13`) | rewrite | The sim's spatial half; W2 cannot be founded on an undecided answer here |
| General model (2026-08-06) | mixed, sim causal grammar and wing organ | Confirmed §3.3 and §3.1 | rewrite | Split: E0 to E4 are sim; §7.4's hagioglyph is a wing organ no single game owns |
| Vessel briefs (2026-08-18) | game overlays | Confirmed, §5 and ruling 6 | keep | Add the grid line ruling 18 settles |
| Resident views composition (2026-08-14) | stack, rendering and compute | Confirmed, §1 | keep | `ResidentChunk` is §4.1's seam |
| Played slice (2026-08-28) | game overlay | Confirmed, §5 and §3.8 | keep | A playtest of product content is a done-condition §6 allows |
| Engine ecology rulings and review (2026-08-18) | method and stack | Confirmed, §6 practised | keep | W1 produces more of these |
| Terrarium dynamics (2026-08-29) | Mesocosm overlay, timescale | Confirmed, ruling 6 | keep | Its milligram-conserved soil, body and carrion cycle is sim (§3.3 second shape) and should be cited there |
| Scale (2026-08-29) | sim, §3.5 | Contradicted, §4.3 and §8: "±87 is the unwindowed wall" (`:29-31`) is the budget constant read as a limit, and S2 exists to adopt paging that ships; S3 is §3.5's four rules | rewrite | S3 is the wing's only measured route to hundreds of thousands of things |
| Open rulings register (2026-08-29) | method, historical | Says of itself "It will go stale" (`:15`) | retire | Self-superseded; live entries have owners |
| Elements and traits memo (2026-08-29) | sim, fields | Confirmed, §3.1 and §3.5 | keep | "No fields in PE4's first world" is a deferral the base profile cannot keep |
| Traits and perception brief (2026-08-29) | mixed, sim relations and overlay unlocks | Confirmed, §3.2 | keep | The edge is the sim's; unlock evidence is Mesocosm's |
| Forms of life brief (2026-08-29) | sim | Confirmed, rulings 8 and 9 | keep | Hand the representation question to W2 |
| Default creatures (2026-08-30) | Mesocosm overlay, its ensemble | Contradicted, §6 and ruling 15: gates are "seeds 1 to 10" (`:554`, `:570`) and a chosen capsule budget (`:583`) | rewrite | Restate the gates as draws; the archetypes are good overlay content |
| Dev tools (2026-09-01) | stack, becoming the bench | Contradicted in placement, W4: DT1 to DT4 landed inside the product host | rewrite | The tools are right; their home is a tier down |
| Trophic grammar (2026-09-04) | sim, §3.3 first shape | Confirmed; contradicted in method by §6: gates are fixed founder counts and "the same three seeds" (`:304`) | keep | Restate the gates; the grammar is the best sim work in the repo |
| Orthographic voxel presentation (2026-09-11) | stack, rendering | Confirmed, §4.2 and §4.6 | keep | Its benches are the product bench W4 lifts out |
| isometer extraction (2026-09-14) | stack, rendering | Confirmed, §4.1; contradicted in one place by §2: `anatomy`, `body`, `plan`, `wire` are sim nouns filed in the rendering family (`isometer-core/src/lib.rs:32-37`) | keep | One misfiled module set |
| isomere (2026-09-15) | stack, hosting and interface | Confirmed, §4.1 and §9.8 | keep | Rewrite only the keymap row per ruling 21 |
| isometer family (2026-09-14) | stack, rendering | Confirmed, §4.1 | keep | Owes W3 the terrain y-scale |
| Effect pack preset (2026-09-15) | mixed | Superseded by the glyph expression plan on its own account; done-condition is one fixture | retire | `wing-glyphs/src/pack.rs` (sim) and `mesocosm-core/src/effect_pack.rs` (overlay) survive |
| Glyph expression (2026-09-15) | mixed, sim canon and overlay embodiment | Confirmed, §3.1 | keep | Canon and expression table are the sim's; the part-to-process binding is Mesocosm's |
| Trait catalogue (2026-09-15) | mixed | Confirmed, §6: states its sizing as a parameter (`:126-127`) | keep | Independently produced the thirty-shape receipt |
| Functional generation (2026-09-09) | sim | Confirmed, §2: product-neutral in `shared/wing-functions` | keep | Index defect: filed below the Archive heading |
| isoscape family (2026-09-16) | sim and stack generation | Confirmed, §3.4 and ruling 16 | keep | Index defect: absent from the index |
| Soil cycle (2026-09-16) | sim, §3.3 second shape | Confirmed: the only plan whose subject is an agentless process; contradicted in method by ruling 15 (seeds 7, 1 and 42) | keep | Index defect: absent from the index |

### 2.1 Sim versus Mesocosm overlay

The split W2 founds the sim from, by crate and module.

| Crate or module | Tier | Why |
| --- | --- | --- |
| `isometer-core::{body, plan, anatomy, wire, snapshot}` | sim | The Bodies noun, product-neutral by its own doc; misfiled in the rendering family |
| `isometer-core::ground` | sim | Space at the near rung; the tracer is a consumer |
| `mesocosm-core::places` (grown, relief, near, bricks, soil) | sim | §3.7; needs re-derivation from the volume |
| `mesocosm-core::{history, record}` mechanism | sim | Already a newtype over `hagiograph::Record` |
| `record::{Feat, Scale}` axis set | overlay | Six feats at three scales are Mesocosm's vocabulary |
| `mesocosm-core::deep_time` | sim | Runs with nobody playing |
| `mesocosm-core::{flow, matter}` | sim | The conserved ledger; §3.3's agentless half |
| `mesocosm-core::{process, phenotype, development, growth, axis}` | sim | Body realization and the process vocabulary; `ProcessDef` widened first |
| `mesocosm-core::{organism, cohort, pressure}` | sim | §3.5's aggregation, already typed |
| `mesocosm-core::{species, chronicle, program}` | sim | Lineage as a record |
| `mesocosm-core::rules` | sim | §1's rules |
| `mesocosm-core::world::{genesis, terrarium}` | sim, toward isoscape | Generation |
| `mesocosm-core::world::{act, intent, consume, express, graft}` | overlay | Metabolize and its handles |
| `mesocosm-core::world::{review, revise, adapt}`, `discovery` | overlay | The trait board and the epoch's played turn |
| `mesocosm-core::{effect_experiment, effect_pack, embodiment}` | overlay | Mesocosm's execution table and marker forms |
| `shared/wing-glyphs`, `shared/wing-functions` | sim | Product-free vocabularies |
| `mesocosm-core::voxel_profile` | stack adapter | Ground bricks to nisus; §4.3's duplication lives here |
| `mesocosm-phenotype` (pack, admit, express runner) | stack (piccolo) and sim vocabulary | §9.3 |
| `mesocosm-runtime::clock` | sim, owed | W2's due-event scheduler; the rest of the runtime is overlay tempo |
| `mesocosm-views`, `mesocosm-genet` | overlay and stack host | Mesocosm's chrome over isomere and genet |

The three the lane would act on first: the place-graph engine plan
rewrite, the ProcessDef rewrite, the scale plan rewrite. The record's
corrections from this evaluation were folded the same day (§3.3, §3.4,
§3.7, §4.3, W4, §9.11).

## 3. Paredros, 2026-09-18

Read-only evaluation of every active document in Paredros's index against
the record. Nine documents; no archive exists. The index lists four plans
only in prose rather than in its table, an index-shape defect under
DOC_POLICY §6. Rulings are Mark's; recommendations are the lane's.

| Plan | Tier | Record says | Recommended | Why |
| --- | --- | --- | --- | --- |
| DOC_POLICY | method | Confirmed, §6 | keep | Canonical core plus a three-line addendum |
| PROJECT_DESCRIPTION | game overlay | Confirmed as §5's overlay (pillars 1 to 3); contradicted twice: pillar 3 states the sim's definition as a Paredros pillar (`:34-36`), and "no genre, schedule, or verbs" (`:50-53`) is §9.11; its Speculative row describes ruling 7's branching as unplanned | surface to Mark | Maintainer-owned |
| Founding plan (2026-07-30) | mixed, sim world rules and generation, plus the overlay | Confirmed as the overlay by the care-granularity canary and ruling 6; contradicted by §2 where it owns world-rule generation (`:307-340`), where the world comes from (`:554-563`) and a per-vessel renderer (`:583-592`) | rewrite | Split the world half into the sim; second person, standing agreements and place lineage stand |
| Execution plan (2026-08-07) | mixed | Contradicted, §3.5 and §6: F2 closes "other lives" on a step that ticks every living member every round (`simulation.rs:119-124,129`) with residents capped at sixteen a site (`population.rs:17-18`); §11 reorders it; S0's frame spans name no build; the workspace runs a nonstandard dev profile (`paredros/Cargo.toml:50-51`) | rewrite | F0 to F8 are the right layers; the ordering is product-first and the receipts are fixtures |
| R4 extraction review (2026-08-10) | stack | Contradicted, §4.1 and §4.2: names paredros-identity the wing identity crate where §4.1 puts identity in dramatis, and its one extracted seam is netrender tenancy for a renderling tenant L7 retires; its license line is stale (the crate is MPL-2.0) | retire | Target wrong, code fine: netrender's `TenantNeeds` and `boot_shared` survive as stack hosting; paredros-identity stands as overlay ids until dramatis takes them |
| Functional loops (2026-09-09) | mixed, sim bodies and places, Paredros verbs | Contradicted, §3.7 and ruling 14: every place fact is Mesocosm's heightfield partition (`world.rs:119`, `sites.rs:126-128`); also §3.5 per the execution row. Confirmed: T2 is §4.3's one-edit-all-consumers rule and ruling 12 | rewrite | T2 and the B and J lanes are the strongest live lanes; the place rung under them is the wrong one, cheaper to correct before T lands |
| World conditions (2026-09-09) | sim, §3.3 first shape and fields, written as product-owned | Contradicted, §2 and §3.3: its condition, operation, relation and invariant schema under a content-addressed rules revision (`:65-123`) is the sim's process definition, and its own stop rule forbids promoting it (`:443-444`) | rewrite | Only the tier is wrong; the best answer in the wing to §9.7 |
| Memory and remembrance (2026-09-09) | mixed, sim record and Paredros recall | Contradicted by receipt: calls the hagiograph "a name reservation, no implementation" (`:42-44`) where it is 831 lines live in mesocosm-core; models reach as per-subject observation admission (`:97-100`) where §3.4 makes it a field on the place graph | rewrite | Bounded recall, the cache and semantic-forgetting split and the checkpoint design survive; its save-growth baseline is a single fixture and says so |
| Genet document host (2026-09-13) | stack rendering plus Paredros chrome | Confirmed, §4.1 and §4.2; contradicted as a fact by §4.3: renderling is still an unconditional dependency (`paredros-client/Cargo.toml:66`) and two parley loaders remain; its Pins section is stale | keep | Right tier, landed, and the only Paredros plan whose done-conditions come from the product's own session; owes L7 a Paredros lane |

### 3.1 Sim versus Paredros overlay

`paredros-world` becomes `paredros-core` under §9.1, and it is not one
tier and not one authority: `contact` is a second fixed-step body world
with its own save, which the functional loops plan's own finding records.

| Crate or module | Tier | Why |
| --- | --- | --- |
| `paredros-world::{world, sites}` | sim | Space at the place rung; needs re-derivation from the volume per §3.7 |
| `paredros-world::{bodies, anatomy, motion, movement, movement_profile}` | sim | The Bodies noun, already product-neutral over isometer-core |
| `paredros-world::items` | sim | A body without agency |
| `paredros-world::{population, projects, simulation, simulation_record}` | sim | Runs with nobody playing, literally (`simulation.rs:119-121`); owes §3.5 a due-event queue in place of the round |
| `paredros-world::{state, transitions}` | sim grammar, overlay vocabulary | The subject-addressed intent and event log is §3.3's record shape; the admitted verbs are Paredros's |
| `paredros-world::navigation` | overlay | §3.8: the sim never pathfinds an individual; correctly derived and absent from saves |
| `paredros-world::{combat, timed_action}` | overlay | Paredros's verbs; the sim records that a blow landed, not how |
| `paredros-world::contact` | overlay, and a duplicate | A second fixed-step body world; conatus owns that tier |
| `paredros-world::{equipment, subject_sheet, technique}` | overlay | Paredros's reading of sim facts |
| `paredros-world::glyphs` | overlay reading over a sim vocabulary | wing-glyphs is the sim's; the per-event grant table is Paredros's and carries no significance gate |
| `paredros-world::fixtures` | test data | Seed 7 and a default world of side 8, extent 64: a chosen size, §6 |
| `paredros-social::{deed, epistemic, relation}` | sim | The record and its observer scoping; §3.4's journal half |
| `paredros-social::{offer, agreement, willing, society, response, companion, settlement, settling}` | overlay | Companions as peers, confidence bands, refusal |
| `paredros-sortie` | overlay, superseded | Retired authority by its own plan |
| `paredros-identity` | sim provenance, misfiled as a product crate | §3.1's Provenance noun; §4.1 puts identity in dramatis |
| `paredros-client::{producer, scene, gpu, frame_health}` | stack rendering, product-owned | A thin scene source over isometer; right shape, wrong owner for the camera preset and palette |
| `paredros-client::{residency, brick}` | stack rendering | The paging isometer-lens wraps; used only by two opt-in receipt bins |
| `paredros-client::bin/{session, timed_action}` | overlay | Real-time pause and play, the charge grammar, panel chrome; where the wall clock is (`model.rs:66-67`, `actions.rs:94-130`) |
| `paredros-client::bin/{room, d1_depth, crossing, v1 residency}` | stack receipts, renderling-bound | L7's retirement set |

The three the lane would act on first: the world conditions rewrite,
since it answers §9.7 with a schema better than a widened ProcessDef and
its promotion costs one ruling and no code; the functional loops rewrite,
since its lanes are live and about to bind navigation, contact and
rendering to the wrong place rung; and an L7 lane under the genet document
host plan, dropping the unconditional renderling row and pointing the
producer at the paging, which closes two record corrections at once. The
record's corrections from this evaluation were folded the same day (§3.3,
§3.4, §4.1, §4.2, §4.3 twice, §7).

## 4. What W1 leaves for Mark

Across the three products: fifty-seven documents read; thirty keep,
twenty-one rewrite, six retire. The retirements: runtime profile
(Isometry); engine and render lane landscape, execution waves, open
rulings register, effect pack preset (Mesocosm); R4 extraction review
(Paredros). Every retirement names surviving code and its tier. The
rewrites all share one of three causes: the plan owns a sim-tier subject
as one product's feature, its done-conditions are fixtures or chosen
sizes, or its place rung is a heightfield partition where the record wants
volume-derived nodes.

Beyond the rows, the record now carries for Mark's ruling: §9.7, the
process definition, with the world-conditions schema as the strongest
candidate; §9.9, three PROJECT_DESCRIPTION contradictions per product;
§9.10, the vertical scale; §9.11, the founding record's three
disagreements; and the paredros-identity promotion under §4.1. No plan
has been edited by W1. When the rulings are made, each plan gets its one
line, retirements move to `archive_docs/<date>/` with rationale, and W2
begins from the two sim-versus-overlay splits.
