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

**2026-10-10, Q7 additions (presentation plan):** certify Mesocosm's lit
body/terrain occlusion, palette and selection under each camera and after
resize; compare the captured final layered master with the window, including
the chromeless route. The tenant adapter's two CPU clipping/projection
controls passed (Q7 Progress); they execute no GPU frame. Measure the
presentation-only `--body-light` setting, per-frame
CPU palette reconstruction and per-face pose validation, mesh uploads on
motion/camera cuts, marked-face subdivision and the retained
terrain receipt's extra import. The body receipt's mesh-byte estimates omit
tenant camera/light uploads; obtain measured counters before making an
upload-silence claim. The old body rasteriser still has VTT/Eponym scene
consumers; migrate those before retiring LiveBody.

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

- **2026-10-10, Q3's native seed-0 receipt diagnosis:** three bounded
  probes passed after the rejected second candidate. Accepted meals credit
  the actor correctly, but 233 lineage feeding plans refused because their
  cached target no longer satisfied the declared scope. `choice.rs`
  chooses a process and prey early in the tick; generated movement at
  priority zero runs before feeding at ten, moving that prey outside the
  feeding site. The played actor accepted five feeds and fifteen upkeeps,
  had no births, peaked at eleven units and exhausted its reserve at tick
  100. Ruling 454 already requires hunters to draw prey from the feeding
  pass's start, while 683 keeps one chosen process. Q3 is correcting the
  automatic target's timing, preserving explicitly named nudge targets
  and recomputing their answers against the eventual target and place.
  This correction is unverified and no rates have changed. Temporary
  production instrumentation was removed and its three source hashes
  matched the original files. Raw receipts are retained in
  `Code/testing/isometry/after-pass-2026-10-10/q3-reproduction/`; the scope
  trace's SHA256 is
  `52f718cf5745628a356869b0919e4a9521c2a6b0f9f6290e2c82cf188c3480d9`.
- **2026-10-10, Q3's second declared candidate completed:** threshold 12,
  original one-unit meals and drawn lifespans 80 to 120 completed all 20
  bare cases. Only seeds 1, 3, 14, 15 and 17 qualified (5/20). Every case
  conserved matter at each tick and replayed under Individuals and Grouped;
  the stricter played-birth, born-heir and three-boundary gate still failed.
  The bodied candidate and fresh original-control arms did not run because
  the bare gate rejected. No rates were promoted. Native receipt/held
  diagnosis of rejected seed 0 follows before another rate profile is
  declared. A1 remains open; this is a rejected tuning receipt under 761.
- **2026-10-10, Q3's declared reproduction draws:** seeds `0..20`, three
  sites, 60 members in cohorts of two, three lineages, played consumer
  `lineage:1` with regions of two sites, individuals, 180 played ticks and
  three configured 60-tick epochs. Original rates failed the lineage gate
  on 20/20 bare and 20/20 bodied draws. The first candidate changed all
  three lineages' birth thresholds from 30 to 14 and consumer/decomposer
  meals from one to two, retaining the original 20 to 60-tick lifespans.
  Only 3/20 bare draws passed (5, 6, 17). Bodied candidate draws 0 through
  9 completed, with only draw 1 reaching 180; the already-failed run was
  interrupted, so no 20-draw bodied candidate receipt exists. Each completed
  case conserved matter every tick and replayed to the same hash. These
  candidate rates remain outside generator defaults.
  *Reading, not ruled:* the configured 60-tick epoch is a verification
  scope, not a settlement of the stock 525,600-tick year/epoch or headed
  readiness. The next declared candidate keeps one-unit meals, child
  provision eight, upkeep one per five ticks and birth cadence seven;
  all three lineages use threshold 12 and drawn lifespans 80 to 120. Its
  promoted gate must count actual later births and born heirs, with no
  living founded consumer remaining at tick 180, besides replay and
  conservation. A1 stays open until that gate passes on at least 20 draws.
- **2026-10-10, Q2/Q3's arrived-body boundary:** `Arrive` may admit own
  reserve on an entity ledger, then `Embody` keeps that ledger while adding
  geometry with a free lattice and no tissue (`arrival.rs:110`). Native
  `anatomy::held` includes that loose balance but `take_within` spends only
  anatomical parts. Q2's hunger command consequently refuses insufficient
  reserve. Generated-body Q3 checks cover valid founded tissue in parts,
  not this admitted balance. Carry the arrival-to-embodiment accounting
  decision to Q10's Eponym body admission gate; do not generalize Q3's
  generated-body receipt to it.
- **2026-10-10:** the baseline above. Legacy stands at 3,968 lines, all
  `legacy/campaign`, held by the faction turn until V2 (247).

## Progress

- **2026-10-10, Q2 native checkpoint integrated:** native deed events now
  retain ruling 808's optional agreement ID with causal-event validation
  intact, alongside asserted map/storylet/pack records and their witnesses.
  The lane passed native event compatibility (1/1), assertion tests (7/7)
  and the native compile gate; the orchestrator's merged native
  workspace/all-targets check passed offline. Q2's consumers, the shared
  overlay, character metadata and the broader A2 baseline remain open.
- **2026-10-10, Q7 integration:** tenant migration `2c86e5f7` passed its
  four workspace compile gates and two CPU controls; the orchestrator also
  passed all four merged workspace gates offline. These certify source
  integration and CPU clipping/projection, while A4 retains the GPU/headed
  and performance checks above. `LiveBody` has live VTT/Eponym/query
  consumers and is retained. Q8 now owns the finished renderer worktree
  for checkpoint 10, with healing allocation awaiting Mark's ruling.
- **2026-10-10, Q3 checkpoint integrated:** the generated-body account
  and turn-order slice is integrated after 42 focused tests and a merged
  workspace/all-targets offline check. A1's survival and reproduction gate
  remains open; these checks do not certify Arrive/Embody admission or
  generator cadence.
- **2026-10-10, ruling 808:** Q2's focused contracts exposed agreement
  actions using a non-event as an event cause. Mark chose a typed optional
  agreement link on the native event; Q2's association and replay checks
  gate its repair. The native `Arrive` then `Embody` path also leaves loose
  own matter in the entity ledger which `held` reads but anatomical takes
  cannot spend; Q3 records this body-admission residual for E3, with its
  generated-body accounting tests qualified to that valid envelope.
- **2026-10-10, Q1:** current state/status and native source paths verified
  under 793 to 802; dated receipts preserved. This documentation pass adds
  no compile, test, draw or headed certification.
- **2026-10-10, Q3 lane checkpoint:** native generated-body account and
  turn-order repairs built and checked (sim and directing Progress); A1
  remains open after the rejected first cadence. No generator rates were
  promoted from that failed profile.
- **2026-10-10:** plan written; A2's baseline taken.
- **2026-10-10, handoff:** three lanes stopped at Mark's weekly budget,
  their work kept on branches, each closing with a WIP commit
  (unverified): `lane-plans` (archive, folds and rewrites per 793 to 802; six
  commits done), `lane-repro` (A1 and 797; diagnosis begun), `lane-contracts`
  (794, 795; Eponym routing begun). Worktrees under `Code/worktrees/isometry-
  {plans,repro,contracts}`. Resume each from its branch; nothing merged.
