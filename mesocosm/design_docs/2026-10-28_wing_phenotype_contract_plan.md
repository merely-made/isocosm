# Wing construction, embodiment and continuity

**Status:** 2026-10-28: cut from `2026-07-31_wing_phenotype_contract_plan.md`, which is archived in
`archive_docs/2026-10-28/`. The shared v0 body and chronicle formats are extracted (`shared/wing-formats`, `shared/wing-integration`); the native sim carries `body_revision` on its entities and drives a scene through `SubjectKey`; `wing-glyphs::Journey` holds reincarnation inside its progression kernel. Open: the shared construction description, continuing-subject and incarnation storage, configured condition carryover, the v1 identity contract (W0 to W6) and the creator and generator lanes (C0 to C5), none of which is built on native Isocosm. The old plan marked its done-conditions "not authoritative" until rewritten (W1 verdict, 2026-09-18); this plan re-expresses them without changing them.

**For:** the identity, incarnation and construction contract that every game shares: continuing subject, body revision, part address, carryover; the W0 to W6 portable-body gates; and the C0 to C5 creator and generator lanes, re-expressed on native Isocosm. Phases W0 to W6 and C0 to C5 keep their labels; the open design work before them is P1 to P6.

Not here: the sim's recipe, development and body geometry (the sim's own plan, `sim`); the binding of bodies to physics, footing and senses are `bodies`; the journey's acquisition and progression rules and divinity are `wing-organs` (same-individual rebirth is `wing-organs G4`); the composition design for magic and construction is `functional-generation`. The sim host's examiner and incarnation controls wait on this plan: see `sim-host`'s deferred examiner item. A projection of this plan in the scene is `orthographic-voxel-presentation`.

## Constraints

Stop rules binding every phase below (old §10):
- No wing-wide capability enum or evaluator.
- No sprite, flattened voxel grid, collision box or cached score as body authority.
- No species plus local part index as provenance.
- No duplication of current anatomy in both body and chronicle v1.
- No foreign vessel mutating anatomy by narration.
- No mandatory exact ancestral geometry for a descendant.
- No v1 before stable subject, revision and part addresses are proven locally.
- No generic body library before a second capability reading exists (the wire contract may precede that extraction).

Standing design statements from the old plan:
- Construction declares functional properties; geometry supplies named measurements where mechanics require them (405). Geometry, collision, animation and functional readings identify the source revision they reflect. One body can correspond to multiple runtime objects; changing a mesh or camera alone does not mint a new incarnation or change an injury.
- The live body is timing-sensitive state: one simulation authority or ordered session accepts severing, grafting, regrowth and body replacement. The anatomy tree is not a CRDT. Other vessels append facts and proposals; concurrent claims against one revision stay visible until sequenced or branched (set union preserves claims, it does not merge anatomies).
- Keeping an independently mutable second world beside Isocosm and synchronizing answers creates the second authority the adapter exists to avoid. The scene adapter is a derived, replaceable view of accepted construction.
- Four transitions stay distinct: a mode or camera change in one world (same subject, incarnation and body); interpretation under another ruleset (record an adaptation if it changes the body); adaptation or regrowth in another context (a new revision, the prior form stays pointable); a new life (selected carryover applied; a continuing individual's reincarnation is distinct from founding a descendant).
- Mark, 2026-09-07 (generation): start with a seed and a few key criteria; a detailed parts editor follows later.

From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` the stop rules (lines 531-543), the four transitions (lines 43-61), authority (lines 439-453), the 2026-09-28 mapping section (lines 109-215) and generation inputs (from line 773).

## Phases

### P1. The shared construction description and its source-to-part mappings
Define one construction description, and the mappings from accepted source to parts, that is coherent across modes, generation and history before implementation slices are chosen. Coordinate with `functional-generation`'s construction composition (405, 407) and the sim's capability, repertoire and proficiency model.
- Done when: not stated beyond coherence across modes, generation and history; owed.
- Needs: `functional-generation`; `sim`; `wing-organs`
- Rulings: 405, 407
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` "Work still required", lines 91-100.

### P2. Body revisions and incarnations
Define the relation between body revisions and incarnations; continuing-subject and incarnation storage; and multi-part correspondence for contextual adaptations. A first same-body adapter must not claim to settle these.
- Done when: not stated; owed.
- Needs: P1; `wing-organs` (journey)
- Rulings: 405, 406, 407
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 93-94 and 202-205.

### P3. Revision invalidation for derived consumers
Geometry, collision, animation and functional readings identify the source revision they reflect, and a consumer learns when it is stale.
- Done when: not stated; owed.
- Needs: P1
- Rulings: 405
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 43-47 and 93-95.

### P4. Condition adaptation and carryover into a new life (ruling 406)
A new life selects which old conditions to carry forward, granularly, randomized or configured, with a coarse keep-all optional; omitting a condition leaves its history intact. Proposed record shape, not yet ruled or implemented: each candidate cites its source incident and condition, the proposed destination expression and the adaptation rule revision; the transition records what was selected, omitted or transformed and whether configured or drawn; an old part address stays a source reference, not an assumed address in the new body; if several expressions fit, expose the alternatives rather than silently inventing an equivalence; the player-facing choice explains the consequence, not internal IDs. Carryover is distinct from restoration of acquired effects. Mark (406, as the wing organs plan records it): "i think practically it's like choosing what conditions to bring forward to the next life. That decision can be randomized or configured by the player. Or even just a blanket 'keep conditions?' prompt would be fine, though i would prefer the more granular choices." Exact restrictions, defaults and costs of selection remain open; so does the interaction with acquisition restoration.
- Done when: not stated; owed.
- Needs: P2; `wing-organs` (journey, acquisition and progression)
- Rulings: 405, 406, 407
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 63-89; `mesocosm/design_docs/2026-09-26_wing_organs_plan.md` lines 208 and 8-9 (quoted words).

### P5. Sim entities into the shared scene: the body adapter
Drawn entities and selected parts resolve to accepted source addresses. Mapping rules: full `u64` source part IDs in a correspondence table (no cast to `u32`, no inference from vector position, no display ID as a sim command argument); mappings scoped to session and branch, with picks expiring on replacement; the sim's body revision remembered separately from the scene's `BodyDependencyRevision`; attachment offsets, pivots, extents and volumes, absent from the sim source, supplied by accepted construction and never invented; local placement from an accepted source, so an illustrative arrangement is labelled and claims no location, clearance or contact; picking stays isometer's (347), the host validates the receipt and submits an intent through the game boundary, never a `SubjectKey` or renderer `PartId`. Publish scene inputs only after a successful session operation; a refused operation leaves the last mapping intact.
- Done when: "drawn entities and selected parts resolve to their exact accepted source addresses; a changed or severed part updates the same scene after acceptance; stale picks refuse after body/session replacement; and rebuilding presentation from an unchanged saved state preserves correspondence and appearance. Verification must include a source part ID above `u32::MAX`, unchanged rendering without sim mutation, and rejected operations without leaked visual changes. Report source revision, projection work, cache reuse and uploads separately."
- Needs: an accepted construction source and a local placement source (`mesocosm-overlay`'s M2 absorption and the spatial clock); P1
- Rulings: 346 to 352
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 109-215.

### P6. The remainder after the v0 format extraction
Remaining: the v1 identity/topology contract (W1), a common world/checkpoint fixture, minimap/readback adoption and the coordinated renderer dependency migration. Removing the Netrender/RG3 facade bridge needs a Mere/Genet/Netrender API migration, followed by Mesocosm GUI and Eponym room checks and headed composition receipts. That migration is the archived platform-alignment follow-up.
- Done when: not stated; owed.
- Needs: W1; the platform alignment follow-up (`design_docs/archive_docs/2026-10-10/2026-09-09_games_wing_consolidation_plan.md`)
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 1307-1316 and 1354-1355.

### W0. Contract reconciliation
- Done when: "the wing founding record, phenotype plan, body-profile comments, and doc index agree that topology is portable, geometry is an optional projection, and capability is vessel-owned."
- Needs: -
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` §9 W0, lines 487-491.

### W1. Stable addresses
`mesocosm.body/v1` carries subject id, body revision id and causal predecessor, biological-line reference, root part id, per-part id, optional parent, state and source provenance, with optional flattened cells, attribution, collision hints or projection recipe; offsets, pivots, exact volumes, functional links, process allocation and developmental rules only where the profile declares the capability. A part address is source subject, source body revision, source part, biological line when known, and acquisition event; local ids are qualified by the issuing world or history domain. `mesocosm.chronicle/v1` is an event stream scoped to a subject and, where relevant, a body revision, and stops duplicating a flat body snapshot. The envelope is extensible with required and optional features, and uninterpreted payloads are retained losslessly, tested through edit and re-export. Unknown required biology refuses active simulation; an optional baked projection may still permit display. v0 is handled by version refusal, with no permanent migration layer. The field list is conceptual, not a schema.
- Done when: "two organisms of one species can each have `PartId(3)` and a provenance record still identifies the exact source; body revision changes are causally ordered; snapshot and replay preserve those addresses."
- Needs: P1, P2
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` §5 and §6, lines 357-437, §9 W1, lines 493-497, and lines 695-708.

### W2. Local branch proof
Graft rules: the source operation names a living subtree under one source revision; destination part ids are freshly allocated; parents are remapped; the root attaches to one destination part; every destination part keeps its source address; the source's loss and destination's acquisition are causally linked; severing the root removes the imported subtree. Assimilation is distinct from graft and a consumer can tell which occurred.
- Done when: "Mesocosm transfers or assimilates a source subtree according to the local phenotype plan, loss cascades on both sides, and provenance identifies the source revision and part after destination ids are remapped."
- Needs: W1; the native `graft` module (`shared/isocosm/src/graft`)
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 372-386 and §9 W2, lines 499-503.

### W3. Mesocosm to the VTT (v1)
- Done when: "Isometry reads a v1 body using local primitive mirror structs, renders its projection, preserves unknown anatomy data, appends a deed addressed to the body revision, and returns it without linking Mesocosm." (Isometry is the VTT's former name.)
- Needs: W1; `vtt-overlay`
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` §9 W3, lines 505-509.

### W4. VTT to Mesocosm interpretation
- Done when: "Mesocosm distinguishes narration from a granted body-affecting claim, applies an accepted loss to the correct revision, rejects a stale or wrong-subject address, and retains every foreign fact."
- Needs: W1, W3
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 439-453 and §9 W4, lines 511-515.

### W5. Second reader
Written for Paredros, which is Eponym's former name; Eponym is the current second reader, with its identity crate and anatomy admission. Do not extract a generic body library until the second capability reading exists.
- Done when: "a character keeps skills and relationships while changing body revision; losing a relevant subtree changes Paredros-derived capability; and a generated body uses the same slot as a played one."
- Needs: W1; `eponym-overlay`
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` §9 W5, lines 517-521 and 542-543.

### W6. Descendant distinction
- Done when: "returning the same subject preserves body revision identity, while founding a descendant mints a new subject and phenotype with pointable ancestry; neither operation is implemented as a flat star rebuild."
- Needs: W1; the lineages family's absorption into Isocosm (`mesocosm-overlay`, M2 absorption)
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 335-344 and §9 W6, lines 523-527.

### C0. First generator, fixtures and admission matrix
Seed plus key criteria produce several candidate bodies with retained features and structural variation; impossible requests report their conflict or a bounded search failure. Criteria are required, preferred or unconstrained; unsupported requests are visible; a small supported vocabulary begins; required criteria are never silently relaxed. The authored comparison corpus: rooted or radial, articulated mobile, asymmetrically injured or grafted, one unknown optional mechanism, an unnamed start, a factionless named life. The generator is the sim's, `shared/isocosm/src/generate.rs`, not the deleted `mesocosm-core::world::generation`. Wider grammars (articulated, radial), taxonomy, free parts editing and population-level persistence are separate follow-ups.
- Done when: "the first bounded generator produces inspectable candidate bodies with recorded seed, recipe/content revision, conditions, and locks. Identical inputs reproduce the body; varied seeds produce structural variety; retained features survive regeneration; impossible requests report their conflict or bounded search failure. Preview generation does not create a live subject or spend world resources. Compare generated and authored bodies in the same admission matrix, recording accepted/rejected candidates, reasons, generation cost, and visible differences. At least one supported body family must be genuinely generated; the other fixture families may initially be authored. Wider biological generation remains iterative. Novel multi-body colonies still need PE5's subject/body decision."
- Needs: `sim` (the native generator); PE5 (archived playable ecology plan)
- Rulings: Mark, 2026-09-07
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 773-810 and 1262-1277.

### C1. Body inspection and checked preview across two native hosts
- Done when: "two native hosts show the C0 shapes with keyboard and pointer access, preserved selection through rotation, small-window fit, and readable invalid choices. Browsing, randomizing, and cancel leave authoritative state, resources, and event logs unchanged. Present at 960x540 and 1920x1080 as the initial acceptance sizes; dimensions remain configurable. Report frame time, resident bytes, and preview rebuilds with device and fixture, using the local baseline rather than inventing a shared performance threshold."
- Needs: C0; a second host (`eponym-overlay` or `vtt-overlay`); `sim-host`'s deferred examiner item
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 812-823.

### C2. Local creation and entry
- Done when: "a generated and a customized start can enter a supported local scene, perform one characteristic action, save, reopen, and replay. A stale preview, invalid placement, exhausted cost, and failed admission produce no partial subject, sheet, or item grant. Creating two lives from one template does not reuse identity. Each host receives its own receipt; one pass does not close all three. C1/C2 do not wait for portable v1 or world-scale work."
- Needs: C0, C1
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 825-835.

### C3. Portable body and destination preview
- Done when: "W1-W6 have their own evidence, including wrong-world and wrong-subject rejection, stale revision refusal, required-feature refusal, opaque preservation through re-export, and same-subject versus descendant behavior. One portable body has distinct, explained game readings, an Isometry projection, and a causally addressed return fact. File-based transfer suffices for this gate; live federation is a later transport receipt."
- Needs: W0 to W6
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 837-847.

### C4. Body in motion and use
Broad procedural animation stays separate.
- Done when: "a consequential body change remains recognizable in preview and play; appropriate motion, picking, collision, and action sources agree with the current revision. A rooted form is not made to walk. Failed motion or missing rendering has a truthful fallback."
- Needs: P5; `bodies` (footing, and the contact and jointed-collision admission in binding)
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 849-858.

### C5. Historical starts
Needs the world and history lane's small context contract; the archived plan names no plan that owns it.
- Done when: "a player can select a causally possible start and understand the refusal of an impossible one. An earlier branch admits no later knowledge; generated history cannot overwrite accepted events; importing a played history displaces a compatible generated slot."
- Needs: the world/history lane's context contract (owner not named)
- From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 860-869.

## Deferred and parked
- None. Everything open here is a phase or a question.

## Questions for Mark
- **First view of the scene (P5)**: whether the initial scene view is an explicitly illustrative population inspection or a located world scene. The latter requires real local pose; "deterministic display spacing does not provide it." Also open: the owners of the accepted construction source and the local placement source. From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 198-204.
- **Deliberately open design (C0 to C5)**: the first editable morphology vocabulary; the amount of founder authorship; the Paredros (Eponym) learning economy; the public shared component API; whether free voxel sculpting is wanted, which needs an explicit geometry-to-function admission rule; the relation of play progression, editor access and sandbox freedom (no assumed universal unlock ladder); how much taxonomy is authored, generated or inferred (`Kingdom` was a trophic-role reading, not taxonomy); optional natural-language description input, only after deterministic structured criteria work, with the accepted structured request recorded (model size, latency and quality unmeasured); the VTT system adapter and its mapping receipts. Multi-body organisms, arbitrary gravity, a shared combat ruleset, marketplaces, procedural civilizations and online simultaneous editing each need their own consumer evidence. From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 710-769 and 1268-1277.
- **Carryover details (P4)**: the restrictions, defaults and costs of selection, and its interaction with restoration of acquired effects, are still to be designed. From: `archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md` lines 63-89.

## Findings
- 2026-10-28: `shared/wing-formats` and `shared/wing-integration` exist and hold the v0 formats; v1 is not built, and no `carryover` or incarnation storage exists outside `wing-glyphs` tests.
- 2026-10-28: the body adapter (P5) is partly present: `Entity.body_revision` and `spatial.rs` placement exist in `shared/isocosm`, `SubjectKey(u64)` and `PartAddress` are in `shared/isometer/src/bodies.rs`, and `mesocosm-genet/src/section/tenant.rs` caches by `SubjectKey`. No test with a source part ID above `u32::MAX` was found. The old status line says the sim-to-scene connection now exists.
- 2026-10-28: nothing of the creator surface survives: the generator criteria, picker, drafts, held-body habitat comparison, `generate-start`, `--create` and `--draft` lived in the deleted `mesocosm-core` and `mesocosm-genet`; no `criteria` hit exists in the native tree, and `mesocosm-genet --start` now means a deep-time start. The old C0 to C2 implementation history is void on deleted code.
- 2026-10-28: the old plan's §3 "historical product allocation" and its Paredros, Isometry and `OrganismId` wording are pre-rename or deleted; read Paredros as Eponym and Isometry as the VTT.
- 2026-10-28: the old "preparation documented; body adapter unimplemented" note (2026-09-28) is superseded by the plan's own 2026-10-18 status; the 2026-09-28 mapping table survives here as P5's rules.
- 2026-10-28: `shared/isocosm/src/graft` exists (W2's native graft).

## Progress
- 2026-10-28: cut from `2026-07-31_wing_phenotype_contract_plan.md`.
