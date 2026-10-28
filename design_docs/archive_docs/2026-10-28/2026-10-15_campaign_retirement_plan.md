# Retiring `isometry-campaign` into its owning homes

**Date:** 2026-10-15

**Status, 2026-10-18:** R0 and R1 are done: `native/`, `contract.rs` and
`founding/` now sit in `isocosm-vtt`, and the root all-target, all-feature
check, the runtime counts (492 passed, 0 failed, 6 ignored, matching the
baseline) and the endpoint g4 receipt pass. R2 (map lowering into
`isometry-core`, the store and proposals), R3 (the world into the sim at V2,
then the crate retires) and R4 (publishing) remain, and R2 is next. The GM
store and proposals stay under case 921 until V2; the H7 family endpoint work
is a separate change, and the Genet app still uses the old VTT event authority.
Opened by case 913 and shaped by cases 915 and 917.

## Why

Mark, 2026-10-15 (913): "We should retire isometry-campaign, which is
unpublished, and make its contents into components." Asked what
"components" means, he chose owning homes (915): each part moves to the
crate that owns its subject, with no new crates and no wasm components
(D18 keeps first-party code native). Ruling 247 already moves the
campaign's *world* into Isocosm at V2 (the
[VTT overlay plan](2026-09-25_vtt_overlay_plan.md), §4). Since that table was
written (4,234 lines), the crate has doubled: the VTT's game over the native
sim grew inside it. This plan carries 247's absorption and adds the rest.

## Findings, 2026-10-15

`crates/isometry-campaign` is 10,764 lines, MIT OR Apache-2.0, never
published. Its consumers are all the VTT's: `isonetry` (110 uses),
`isometry-genet` (73), `isometry-views` (66), `isocosm-vtt` (12),
`isometry-system` (9), `isometry-graphshell` (1), plus one test in
`shared/wing-integration`.

| Part | Lines | What it is | Used by | Home under 915 |
|---|---|---|---|---|
| `native/` | 5,464 | the VTT's game over the native sim: game bindings, party, time and holds, joint journeys, overmap presentation, packs, storylets, items, physical maps, institutions, grafts, hooks, placement, eye, readings | isocosm-vtt, isonetry, isometry-genet, isometry-views | `isocosm-vtt` |
| `contract.rs` | 468 | the VTT's assertions lowered onto native entries in one place (795) | isonetry | `isocosm-vtt` |
| `founding/` | 489 | native campaign founding from a `Setup` | isonetry, isometry-genet, isometry-views | `isocosm-vtt`, founding through the sim-host core (H1) |
| `content/` | 3,418 | the campaign world, generator, packs, items, facts, chronicle, construction, maps, overmap data | the VTT's crates | the sim at V2, per 247's table; map lowering to `isometry-core` (915) |
| `lib.rs` | 109 | `CampaignMap` and lowering a map proposal into a `MapDocument` | isometry-genet, isometry-system, isometry-views, isonetry | `isometry-core` |
| `store.rs`, `collaboration.rs` | 396 | the host-private GM store and the proposal lifecycle | isonetry, isometry-genet | F3 |

- **The split is one-way.** `content/`, `store.rs` and `collaboration.rs`
  name nothing in `native/`, `founding/` or `contract.rs`. Those three use a
  handful of content types (`ItemId`, `Inventory`, the storylet types,
  `CampaignDraft`), so the game layer can move into `isocosm-vtt` with
  `isocosm-vtt` depending on what remains of the crate until V2.
- **Licences differ.** The crate is MIT OR Apache-2.0, the VTT's licence;
  `isocosm-vtt` is MPL-2.0, its endpoint ported from the retired shell's MPL
  panels (902). Moving the game layer into it crosses that line (F1).
- **The endpoint is heavy for the session lane.** `isocosm-vtt` depends on
  `isocosm-host` and Graphshell's crates unconditionally. Once `isonetry`,
  `isometry-genet` and `isometry-views` depend on `isocosm-vtt` for the game
  layer, they would carry all of it (F2).
- **Two readings of the store.** 915's summary put the store and proposals in
  `isonetry`; 247 makes a proposal's branch mode the sim's branch and the log
  its intent log at V2 (F3).
- **Publishing (917).** Every crate on the way must stay publishable:
  versioned path dependencies, no git dependencies, licence and metadata in
  order. `isocosm-vtt` today takes `chirograph` and `graphshell-endpoint` from
  mere by git revision, which crates.io refuses, and `isocosm`,
  `isocosm-overlay` and `isocosm-host` are `publish = false` (F4).

## Forks, ruled 2026-10-15

- **F1 (919).** `isocosm-vtt` carries MIT OR Apache-2.0, the VTT's licence;
  its MPL endpoint is relicensed by its author.
- **F2 (920).** `isocosm-vtt`'s endpoint sits behind an `endpoint` feature,
  so the session lane and the host take the game layer alone.
- **F3 (921).** `isonetry` holds the GM store and the proposal lifecycle
  until V2 replaces them with the sim's intent log and branches.
- **F4 (922).** Publishing is this plan's R4.
- **`isonetry` (923, 924).** It is remade as the Isocosm family's session
  layer, the sim-host plan's H7. Until then it keeps its VTT job, and under
  921 the GM store and proposals.

## Phases

- **R0. The plan ruled.** Done when F1 to F4 are ruled and recorded.
- **R1. The game layer into `isocosm-vtt`** (after A1 merges, 916).
  `native/`, `contract.rs` and `founding/` move into `isocosm-vtt` as
  modules of the same names, split along their existing seams under the
  600-line ceiling, with F1's licence and F2's feature. Consumers repoint.
  Done when no path names `isometry_campaign::native`, `::contract` or
  `::founding`; the root workspace's all-target, all-feature check is clean;
  the root workspace's test counts match the baseline taken before the move
  (passed, failed, ignored); the moved tests, the VTT campaign runtime tests
  included, pass in `isocosm-vtt`; and the endpoint's tests and g4 receipt
  still pass.
- **R2. The map lowering and the store.** `CampaignMap` and its lowering
  move into `isometry-core`, which stays pure (no I/O, no sim dependency);
  the store and proposals go where F3 says. Done when those consumers build
  and test as in R1.
- **R3. The world into the sim, and the crate retires** (V2, 247). The
  remaining `content/` families move into Isocosm by 247's table under the
  sim plan's phases. Done when `isometry-campaign` is deleted from the
  workspace and every consumer builds and tests.

- **R4. Publishing (917, 922).** Every crate in `isocosm-vtt`'s
  normal-dependency closure reaches crates.io in dependency order. On
  2026-10-15 that closure held 116 packages, 12 of them path crates (nine
  `publish = false`: `isocosm`, `isocosm-host`, `isocosm-overlay`,
  `isometer-core`, `isometer-space`, `wing-formats`, `wing-functions`,
  `wing-glyphs`, `wing-impresa`) and 10 mere crates taken by git revision
  (`chirograph`, `graphshell-endpoint`, `hagiograph`, `mere-curation`,
  `mere-framing`, `sceno`, `scenograph`, `scenomise`, `scenotime`,
  `state-witness`). On crates.io `isocosm` is our 0.0.1 reservation, nine of
  mere's ten have earlier versions out, and `state-witness`, the
  `isocosm-*`, `isometer-*` and `wing-*` names and `isometry-core` were free.
  Done when every path dependency carries a version; no git dependency is
  left in the closure, mere's crates released at the pinned revision (asked
  in mere); each crate's licence, description, repository and readme are
  set; each new name has gone through the naming ledger; and
  `cargo publish --dry-run` passes for each crate in order. Each publish is
  Mark's word, crate by crate.

## Progress

- **2026-10-17:** R1 is complete. The root all-target, all-feature compiler
  check passes. The serial post-move runtime passes 492, fails zero, and
  ignores six across 40 targets, matching the pre-move test counts (492/0/6).
  The endpoint g4 runner passes all nine sessions and four endpoints using
  the freshly built VTT endpoint. The retained receipt is
  `Code/testing/isometry/r1-2026-10-17/post-move-serial-runtime-controls.log`
  (SHA-256 `60D65C7D0206C3F91B3732E40FA1F2FBD9B1DD3ACF33877D5031CCAA8637959E`);
  `Code/testing/isometry/r1-2026-10-17/retirement-gate.json` records the
  complete gate. Its frozen VTT executable is 28,294,656 bytes, SHA-256
  `e7c7d2016d157e5cb6e8f9ea8fa3bf37583e44756e863c45dfc89455584373dd`;
  the run's HTML is `g4-sessions.html`. The receipt directory retains
  28,662,608 bytes against its 100 MB cap. The H7 family endpoint/network
  implementation is a separate change; the Genet app still uses the old VTT
  event authority, and GM store and proposals stay under 921 until V2.

- **2026-10-15:** plan written from the reading above; rulings 913 and 915
  to 918 recorded. R0 done: F1 to F4 ruled as 919 to 922, and R4 added with
  the closure and crates.io names measured that day.


---

*Archived 2026-10-28.* Cut, with the other active plans, into a new set: what was
still open, deferred or a follow-on here now lives in `2026-10-28_campaign_retirement_plan.md` (in the product's
`design_docs/`), which cites this file by line. This copy is history: its
status lines, receipts and claims are as they stood, not current.
