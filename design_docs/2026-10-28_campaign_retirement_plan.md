# Retiring `isocosm-campaign`

**Status:** 2026-10-28: cut from `2026-10-15_campaign_retirement_plan.md`, which is archived in `archive_docs/2026-10-28/`, with the retirement-relevant lines of the families re-expression plan (`mesocosm/design_docs/archive_docs/2026-10-28/2026-10-08_families_reexpression_plan.md`). R0 (the plan ruled) and R1 (the game layer into `isocosm-native`: `native/`, `contract.rs`, `founding/`) are done, and `isocosm-native` is MIT OR Apache-2.0 with an `endpoint` feature. `isocosm-campaign` is still in the workspace with `content/` (chronicle, construction, fact, generator, item, map, overmap, pack, world), the `CampaignMap` lowering in `lib.rs`, `store.rs` and `collaboration.rs`. Seven crates still depend on it. R2, R3 and R4 are open.

**For:** moving what is left of `isocosm-campaign` into its owning homes (ruling 915), deleting the crate, and getting every crate in `isocosm-native`'s normal-dependency closure published in dependency order. Opened by case 913: "We should retire isocosm-campaign, which is unpublished, and make its contents into components." Asked what "components" meant, Mark chose owning homes (915): each part moves to the crate that owns its subject, with no new crates and no wasm components (D18 keeps first-party code native).
Not here: the VTT's V2 phases that replace the campaign's world with the sim's, which are `vtt-overlay` V2 (this plan's R3 waits on them); the sim phases the world lands under, which are `sim` S1, S3, S5; remaking `isonetry` as the family session layer, which is `session-network` H7; what a pack shows and how terms are authored, which is `content-packs`.

## Constraints

- **F1 (919).** `isocosm-native` carries MIT OR Apache-2.0, the VTT's licence; its MPL endpoint was relicensed by its author. Code that moves between layers takes the destination's licence (doctrine Licensing 3).
- **F2 (920).** `isocosm-native`'s endpoint sits behind an `endpoint` feature, so the session lane and the host take the game layer alone.
- **F3 (921).** `isonetry` holds the GM store and the proposal lifecycle until V2 replaces them with the sim's intent log and branches.
- **F4 (922).** Publishing is R4 of this plan.
- **Publishable all the way (917).** Every crate on the way stays publishable: versioned path dependencies, no git dependencies, licence and metadata in order. Each release is its own decision (doctrine Licensing 2); each publish is Mark's word, crate by crate; releases in mere are mere's to decide and are asked there.
- **Owning homes, no new crates (915).** The split is one-way: `content/`, `store.rs` and `collaboration.rs` name nothing in `native/`, `founding/` or `contract.rs`.
- Every family that moves is decomposed under the 600-line ceiling as it goes, along seams the code already has.
- Crate-naming house rule: at most one hyphen; every new name goes through the naming ledger. A `denizen` crate only once it has a reason of its own (ruling 201); the publish that was once owed for it was struck on 2026-09-25 and is not a task.
- A family's legacy copy leaves once its re-expression is certified and its callers re-point (666). Code the sim's laws exclude is handed back or deleted when its family is reached (669). Tests and examples become draws or retire with their family (672). Anything still under `isocosm-campaign` is the VTT's.
- `isocosm-core` stays pure: no I/O, no sim dependency.

## Phases

### R2a. The map lowering into `isocosm-core`
`CampaignMap` and its lowering of a map proposal into a `MapDocument` move into `isocosm-core`. Both are still in `crates/isocosm-campaign/src/lib.rs`. Consumers: `isocosm-genet`, `isocosm-system`, `isocosm-views`, `isonetry`.
- Done when: "Done when those consumers build and test as in R1." R1's bar: no path names the moved modules; the root workspace's all-target, all-feature check is clean; the root test counts (passed, failed, ignored) match the baseline taken before the move; moved tests pass in their new home; the endpoint's tests and g4 receipt still pass.
- Needs: -
- Rulings: 915, 913
- From: `archive_docs/2026-10-28/2026-10-15_campaign_retirement_plan.md:91`

### R2b. The store and proposals into `isonetry`
`store.rs` and `collaboration.rs` (the host-private GM store and the proposal lifecycle) go to `isonetry` under F3 and stay there until V2 replaces them with the sim's intent log and branches; a proposal's `Branch` mode then becomes the sim's branch (the old overlay's row for these files, at record §5.2 point 2). Until `isonetry` is remade as the family session layer, the Genet app keeps using the old VTT event authority and `isonetry` keeps its VTT job.
- Done when: same as R2a ("build and test as in R1").
- Needs: `session-network` H7 is a separate change (923, 924), not a prerequisite for the move; the final replacement waits on `vtt-overlay` V2
- Rulings: 921, 923, 924, 247
- From: `archive_docs/2026-10-28/2026-10-15_campaign_retirement_plan.md:71-77,91-94`

### R3. The world into the sim, and the crate retires
The remaining `content/` families move into Cosmograph by 247's table under the sim's phases, then `isocosm-campaign` is deleted, including its dependents' imports: `isonetry`, `isocosm-genet`, `isocosm-views`, `isocosm-native`, `isocosm-system`, `isocosm-graphshell` and one `wing-integration` test. The table, kept here so its destination column is not lost (modules named, destinations as ruled):
- the campaign world: factions and polities derived from members (63 to 68); places and routes as the place graph (72); characters as denizens with a faction (36, 200); laws as world-scope rules (41); history as the record; storylets as a pack's requirements and role slots (89, 190); a party's known places as knowledge (84) and its stores as the ledger (38);
- the faction turn: gone from code; the entropy tape's far-rung stand-in retires entirely once the sim's factions run (247);
- facts and items: notes and secrets (80, 87); items bearing provenance, a hidden modifier a note revealed, an item of note a relic (143, 157);
- generator and packs: the generator's declared space (sim plan, 89) and packs as authored content (89, 190); the manifest's home is the question below, and the pack-authoring tool's import of the manifest schema moves with it;
- campaign maps: the battlemap as a projection of a site's generated volume with the DM's map an edit over it (12, 18, 89, 243), transitions as nesting steps (74);
- chronicle: arrivals from other games as crossings and descent (105, 126), `wing-formats` keeps the wire;
- construction: world edits as asserted facts;
- store and proposals: R2b;
- crate root: dissolves.
V2 and `eponym-overlay` E3 (cases 798, 794, 795) are what retire the campaign copy; the copy cannot leave in full before V2.
- Done when: "Done when `isocosm-campaign` is deleted from the workspace and every consumer builds and tests."
- Needs: `vtt-overlay` V2; `sim` S1, S3, S5; R2a and R2b; `sim` certification of the VTT's family
- Rulings: 247, 913, 915, 192, 794, 795, 798
- From: `archive_docs/2026-10-28/2026-10-15_campaign_retirement_plan.md:95-98`; `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:192-232`; `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-08_families_reexpression_plan.md:3,115-122,130,219-222`

### R4. Publishing
Every crate in `isocosm-native`'s normal-dependency closure reaches crates.io in dependency order. Measured 2026-10-15 (dated facts, re-measure before use): the closure's path crates included nine marked `publish = false` (`isocosm`, `cosmograph-host`, `cosmograph-overlay`, `isometer-core`, `isometer-space`, `wing-formats`, `wing-functions`, `wing-glyphs`, `wing-impresa`) and ten mere crates taken by git revision (`chirograph`, `graphshell-endpoint`, `hagiograph`, `mere-curation`, `mere-framing`, `sceno`, `scenograph`, `scenomise`, `scenotime`, `state-witness`). On crates.io `isocosm` was our reservation, nine of mere's ten had earlier versions out, and `state-witness`, the `isocosm-*`, `isometer-*` and `wing-*` names and `isocosm-core` were free.
- Done when: "Done when every path dependency carries a version; no git dependency is left in the closure, mere's crates released at the pinned revision (asked in mere); each crate's licence, description, repository and readme are set; each new name has gone through the naming ledger; and `cargo publish --dry-run` passes for each crate in order. Each publish is Mark's word, crate by crate."
- Needs: mere releasing its ten crates at the pinned revision; the naming ledger; the crate-naming house rule for any new name. R3 changes the closure (the retired crate and its dependents), so the final order is set after it.
- Rulings: 917, 922
- From: `archive_docs/2026-10-28/2026-10-15_campaign_retirement_plan.md:59-63,100-116`

## Deferred and parked

- **Family endpoint and network work**: `isonetry` remade as the wing's session layer is a separate change; until it lands the Genet app uses the old VTT event authority and `isonetry` keeps the GM store and proposals. Waits on `session-network` H7. Rulings 923, 924. From: `archive_docs/2026-10-28/2026-10-15_campaign_retirement_plan.md:5-13,74-77`.

## Questions for Mark

- **Where the pack manifest lands**: `ContentPackManifest`, its validator and the new optional `license` and `credit` live in `content/pack.rs`, which R3 moves. Ruling 247's table sends "generator and packs" to the sim's declared space and "packs as authored content", but the manifest schema is also the real schema the private `pack-author` tool (`isocosm-system`) and `GeneratorPack::load` decode, and the desktop host loads packs through it. Candidates, none decided: the Cosmograph sim (with packs as authored content), `isocosm-native`, `isocosm-system`, or a small shared crate. The answer fixes where licence and credit are validated and where `pack-author` imports the schema from. `content-packs` waits on it. Rulings 89, 190, 247. From: `crates/isocosm-campaign/src/content/pack.rs:30,34,132`; `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:213`.

## Findings

- 2026-10-28: R1's fact stands: `native/`, `contract.rs` and `founding/` are in `crates/isocosm-native`, and no path names `isocosm_campaign::native`, `::contract` or `::founding`.
- 2026-10-28: the old faction turn and `FactionVerb` no longer exist in code; R3's table row for it is already satisfied.
- 2026-10-28: the old plan's F1 follow-up (confirm no MPL notice remains in `isocosm-native`) is closed: `Cargo.toml` reads `MIT OR Apache-2.0` and every SPDX header in the crate is `MIT OR Apache-2.0`; no MPL header or notice found.
- 2026-10-28: the line counts in the old overlay §4 and the old retirement plan are superseded; what remains is `content/` plus the top-level files (`lib.rs`, `store.rs`, `collaboration.rs`).
- 2026-10-28: the R4 closure and crates.io availability are measurements from 2026-10-15 and need re-measuring.

## Progress

- 2026-10-28: cut from the campaign retirement plan (R2 to R4) and the retirement lines of the VTT overlay §4 and the families re-expression plan.
