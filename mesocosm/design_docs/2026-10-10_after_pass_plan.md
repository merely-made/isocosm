# The after-pass: certifying what 732's push moved

**Date:** 2026-10-10

**Status, 2026-10-10:** in progress. Opened by ruling 789; this plan is 790's
single home for every deferred done-condition.

Wing rulings 732 (realign now, compile-gated; testing after), 761 (no tuning
or playing during the migration; findings to this list), 789 to 792
(`2026-09-18_wing_design_plan.md`). 791 replaces 667's parity draws against
legacy readings with each plan's own invariants: conservation, replay and its
controls. 792 puts consumer reproduction first, because every played draw
needs a lineage that outlives one lifespan.

Each item names its owning plan. An item is done when its owner's
done-condition is met and recorded there; this plan only orders and tracks.

## Current state, 2026-10-10

The baseline below is a dated run, not a claim that Q1 reran the tests.
Legacy Mesocosm and Eponym are deleted; the retained campaign is 3,968
Rust lines, including `shared/isocosm/src/legacy/campaign.rs`, until V2.

Rulings 800 and 801 update the owner labels in the original A3 list:
M3 and the interim M4, including D5's headed condition, belong to the
[directing plan](2026-10-08_directing_interim_m4_plan.md); the Mesocosm
overlay keeps full M4. The protocol bump and H2 travel belong to the
[VTT overlay's V2](../../design_docs/2026-09-25_vtt_overlay_plan.md),
whose folded protocol plan is archived. Eponym's functional and memory
conditions belong to its overlay's E3. Ecology residues belong to sim
S8 to S10 (802).

V2 and E3 are open under the compile gate before M3 certification (798).
That does not close A1, the 77-failure baseline, the families' invariants,
or the native/headed receipts. Consumer reproduction remains first (792).
The older owner labels below are read through this annotation.

## A1. Consumer reproduction (792; owner: the sim plan)

Generated consumers cannot reproduce: a birth needs 30 units of body, a
consumer nets about one unit every 7 to 8 ticks, and age takes it at 20 to 60
ticks (directing Findings, 2026-10-09). Rates are a world's; fix it as a
generator or rules change and record the ruling it needs. Done when a played
consumer lineage survives three epochs on draws.

## A2. The test baseline (owner: each workspace's plan)

Full runs on main `b7669871`'s predecessor, 2026-10-10, logs out of tree in
`Code/testing/isometry/after-pass-2026-10-10/`. No compile errors; 77
failures:

| Workspace | Passed | Failed | Failing groups |
|---|---:|---:|---|
| `shared/isocosm` | 411 | 8 | registry agreement, grazer's meal, riff routing, varied cells, old-save loading (3) |
| root (VTT, `--all-features`) | 361 | 15 | storylets (2), watchtower atlas and doors (5), party context, demo pack, overmap swatch gates (6) |
| `mesocosm` | 61 | 4 | the shipped pack lowering to the native ruleset (3), tactile picking |
| `eponym` | 42 | 47 | room, equipment store, residency (5), producer (6), panels (4), model (6), motion and archive receipts (17), sortie (6) |
| `shared/isometer` | 316 | 3 | tracer roster, unlit rung, eating mass balance |
| isomere, wing-integration, isocosm-overlay | 98 | 0 | |

Some of these pinned deleted shapes and retire under 672 rather than pass;
each retirement names what the test meant and where its invariant is now
checked. `Command::Express` (787) has its own tests since this baseline.

## A3. Deferred done-conditions, by owner

- **Families plan (all seven families):** each runs under process
  definitions, conserves its accounts, replays identically, has its tests
  ported or replaced by draws (672), under 791's invariants.
- **Directing plan, D1 to D4:** a real pre-D1 v3 save loading; hashes of
  non-deliberative runs unchanged by D2; a nudge moving choice by the bond,
  zero bond the control; regions merging as biomass falls; three epochs
  headless on draws. 754's second clause is sharpened here (761).
- **Directing D5 and the Mesocosm overlay's interim M4 (680):** the headed
  run, three epochs at site grain, both modes, receipts replaying.
- **Mesocosm overlay, M3:** close by its done-condition once D1 to D4 hold.
- **Eponym overlay, E2:** its families' conditions; the sortie's six
  failures belong here.
- **Sim plan, checkpoint 10:** certify what `harm.rs` holds (wounds, lost
  cells, spill, severing), then build the hazard (704), healing (712),
  fragments (713 to 715) and rot (718); its faults file and controls.
- **VTT:** bump `PROTOCOL_VERSION` and the ALPN for 768's `WorldEvent`
  reshaping (protocol hardening plan).
- **Presentation plan:** L7's S0 replay hash; rg3c and d1_depth's witness
  rewritten for the tenant; L3 and L5 re-proved without the deleted bench.

## A4. Tuning and headed findings (761)

The torch's brightness and shadows (749); body voxel scale against ground
cells; cliff frequency on hilly worlds (744 readings); flow windows' births
and deaths against members; the 3x2 map layout; forced births on bodied
lineages; healing and starvation in Eponym; DC5 colour and the critter
review (default creatures residue); VB3 to VB5 visual acceptance.

*Carried 2026-10-10 from the plans archived under ruling 793 (owner: the
Mesocosm overlay plan, its M4):* the default creatures plan's §6.6, which
the roster must still satisfy on native bodies: every fauna body senses
and contracts, the kingdom floor holds, and the captures read as critters;
its CP1 clearing-and-burrow review, run beside the wing's default view at
the 2:1 dimetric pitch, which rules Mesocosm's opening view (382, 387,
388); and the habitat's canopy, contact and dressing, which CP1 left open.
From the phenotype plan, archived the same day: P0's judgment, whether the
headed meal choice feels tense rather than clerical.

## Findings

- **2026-10-10:** the baseline above. Legacy stands at 3,968 lines, all
  `legacy/campaign`, held by the faction turn until V2 (247).

## Progress

- **2026-10-10, Q1:** current state/status and native source paths verified
  under 793 to 802; dated receipts preserved. This documentation pass adds
  no compile, test, draw or headed certification.

- **2026-10-10:** plan written; A2's baseline taken.
