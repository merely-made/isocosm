# Retiring `isometry-campaign` into its owning homes

**Date:** 2026-10-15

**Status, 2026-10-15:** plan. Opened by ruling 913 (wing design record),
shaped by 915 (owning homes, no new crates), 916 (planned now, moves after
the A1 lane merges) and 917 (the crates aim to be published). Forks F1 to F4
below are with Mark. Nothing has moved.

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
| `content/` | 3,418 | the campaign world, generator, packs, items, facts, chronicle, construction, maps, overmap data | the VTT's crates | the sim at V2, per 247's table; map lowering to `isometry-core` (598) |
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

## Forks for Mark

- **F1. The game layer's licence in `isocosm-vtt`.** (1) `isocosm-vtt`
  becomes MIT OR Apache-2.0 like the VTT's other crates, its MPL endpoint
  relicensed by its author (recommended); (2) the moved code becomes MPL;
  (3) per-file licences in one crate.
- **F2. `isocosm-vtt`'s endpoint behind a feature.** (1) an `endpoint`
  feature, as Mesocosm's and Eponym's endpoints are (903), so the session
  lane and the host take the game layer alone (recommended); (2) always on.
- **F3. The store and proposals.** (1) `isonetry` holds them until V2
  replaces them with the sim's intent log and branches (recommended);
  (2) `isocosm-vtt`, with the game layer; (3) they stay until V2.
- **F4. Publishing.** (1) its own plan once this one's R1 lands, listing
  each crate's blockers, with mere's releases asked in mere (recommended);
  (2) folded into this plan.

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

## Progress

- **2026-10-15:** plan written from the reading above; rulings 913 and 915
  to 918 recorded.
