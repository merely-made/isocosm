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

- **2026-10-10, Q3 meal-three trial declared:** source-only, uncompiled and
  unrun on `ab3b559b`. Only consumer lineage 1's meal changes from one to
  three; threshold twelve, lifespan 80 to 120 and all other supply/timing
  stay fixed. *Reading, not ruled:* the measured first-child sequence
  projects thirteen units at tick 24, crossing the birth gate; changed
  competition, depletion and Choice history can still reject it. Meal two
  remains unrun source evidence. A bounded seed-0 pilot precedes the fresh
  candidate bare20/bodied20 and original bare20/bodied20 matrix, with
  actual births/heirs, founder turnover, three configured boundaries,
  conservation and both replay modes required. Any failed arm withholds
  subsequent arms. Root checked the declaration/proof/manifest hashes;
  complete declaration SHA-256 is
  `2c73bdd707b095d131205c149b06d0512aff2ba28c209b79b0a105fb2bd0ef17`
  in the existing external Q3 directory. Refresh native inputs after any
  integration before executing. No rates are promoted; A1 stays open.
- **2026-10-10, Q3 unchanged seed-0 balance:** the rejected profile on
  `ab3b559b` reproduced extinction at tick 115, conserved matter every
  tick and replayed in Individuals and Grouped. Founder 17's balance was
  `10 + 10 meals - 12 upkeep - 8 provision = 0`; child 130's was
  `8 provision + 4 meals - 12 upkeep = 0`. The child peaked at nine below
  threshold twelve; no born consumer reproduced. Fourteen/sixteen feeding
  scope blocks and two accepted zero-share meals limited parent/child
  intake. The latter correctly floor prey holding ten across sixteen or
  fourteen one-unit demands under 454; no native Refused receipt occurred.
  Temporary observers were removed, original hashes restored and native
  offline all-target compilation passed. Root verified the six evidence
  hashes under `Code/testing/isometry/after-pass-2026-10-10/q3-reproduction/`;
  `q3-reproduction-seed0-after-prey-findings.md` SHA-256 is
  `d4bde01df88dba6d32bcb958e4669189b7c1cafbc29c28400bcb21bad4f74b23`.
  *Reading, not ruled:* a larger meal is a supported next trial variable;
  the trace gives no reason to change causal rules or the one-process rule.
  A1 remains open at 4/20 bare, with bodied/fresh original arms withheld.
- **2026-10-10, Q3 fresh bare candidate completed:** the unchanged declared
  profile ran seeds 0 to 19 on integrated `ab3b559b`. Seeds 7, 14, 15 and
  19 qualified (4/20), with respectively 8, 20, 26 and 8 living born
  consumers, no living founders and three actual boundaries. All twenty
  had actual played births and born heirs, conserved matter each tick and
  replayed under Individuals and Grouped. Sixteen lineages died before
  tick 180; seed 0 reached tick 115 with one birth and no living consumer.
  The failed bare arm withholds the bodied and fresh original-control arms.
  No rates were promoted. A founder/firstborn trace of unchanged seed 0
  follows before another profile is declared. The complete keyed Genesis,
  source/lock hashes and raw output are in
  `Code/testing/isometry/after-pass-2026-10-10/q3-reproduction/q3-reproduction-after-prey-matrix.log`,
  SHA-256 `6b4f2252c4cd438380f7aaad38298345b0fd63d15f91e5c9f4295ca5c9ec4540`.
  A1 stays open; the configured epoch remains a verification Reading.
- **2026-10-10, Q3 prey timing corrected:** the automatic feeding target
  now resolves once against its own pass's start under 454, keeping the
  tick's chosen process under 683. Explicitly applicable Thing/Act targets
  stay bound; answered nudges follow the eventual target and place. The
  lane passed 28 controls, caught a compiled disabled fix, restored exact
  source bytes and passed the positive control. The merged native offline
  compile gate passed. The earlier rejected reproduction draws remain
  historical evidence; a fresh declared matrix is required before rates
  can be promoted. A1 remains open.
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

- **2026-10-10, ruling 811:** unknown legacy loss allocations remain until
  explicitly repaired, while recorded losses in the same body can heal.
  Q8's existing archive validation remains in force. Exact JSON/genesis and
  postcard/witness compatibility are separate checks: field decoding is not
  proof of a saved history loading and replaying. Q2's character metadata
  likewise needs its native witness compatibility qualified and verified.
  These are open implementation/gate items, not fresh compatibility receipts.
- **2026-10-10, Q15 query refresh:** the unchanged tactile target passed
  4/4 and its former critter-before-ground failure is repaired. Omitting
  the refresh compiled and reproduced that failure; exact restoration and
  positive rerun passed. Eponym's contact/action suites passed 12/12 plus
  one compatibility refusal. Its eighteen fixture/admission/archive
  failures occur before the changed query setup and remain A2/Q5 work.
  Both touched consumer compile gates passed offline. The bounded refresh
  lane closes without certifying those motion/save paths or generic body
  bindings. `q15-refresh/results.json` under the external after-pass
  directory retains exact commands, source/log hashes and limitations.
- **2026-10-10, structural forks settled (809, 810):** character cell and
  owner become optional native table metadata, saved and replicated without
  granting control or moving a body. Healing restores each lost cell's
  recorded former function or free-pool status, preserving authored
  allocation and inherited variation. Q2's round-trip/replication gates
  and Q8's paid healing/revival, faults and crowd certification remain open;
  both lanes resume source work. No new test or checkpoint receipt is claimed.
- 2026-10-10, Q8 integration: the independent native checkpoint `9b0ae46d`
  passes all five downstream workspace/all-target checks offline, including
  root and Eponym all-features. Its final 27 code/test/script hashes match
  the tested inputs; raw commands/log hashes are in `q8-root-integration.json`.
  Healing/revival, the unanswered allocation fork, remaining fault controls
  and independent crowd statistics remain open. A1 will rerun against this
  integrated founding source; the earlier rejected profiles remain evidence.
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

- 2026-10-10, Q8 pre-gate findings. Reading, not ruled: after a total-cell wound, the wounded top part spills over its zero bound, while surviving descendants' ledgers move into the fragment. Re-rooting retains lost cells; every lineage eligible under486 begins living and physiology decides whether a zero-cell root can survive, without free restoration. The probe's optional hazard numerator is drawn per world over64; wounds and rot amounts are drawn per world, and each site's world actor chooses a matter-weighted target by the existing native selector. These bounds and cadence are balance choices for after-pass. Rot preserves material account provenance on the site; existing site mineralization performs the later conversion. Independent conservation, zero-hazard and flow controls are being written; native/crowd statistics and planted faults remain unrun. Healing allocation is pending the user's ruling and is not inferred from a recipe.
- 2026-10-10, Q8 focused findings: ten independent harm tests and two compiling planted controls passed in the qualified restored-source receipt; this is not C10 certification. The generated crowd hazard uses exactly one agentless world actor per site, one local target and one harmful operation per process. Reading, not ruled: under that bounded domain, reading current target state equals reading the pass start; competing actors/hits require separate pass semantics. Source-only guards now reject the unsupported cases and cover commitments, pending verification. Lost-cell inspection includes injury history on parent tombstones and on transferred fragments; a living-cell loss control excludes tombstones. Hazard/rot bounds and cadence remain balancing observations for after-pass. Healing allocation remains unanswered.
- 2026-10-10, Q8 bounded guard evidence: all sixteen harm tests now pass on integrated base `f6976e85` plus the final `q8-fragment-source.json` hashes; the native workspace all-target check and founding trait/bounds positive passed too. The guard removal fault compiled and failed its multiple-actor rejection test under the separately preserved `q8-scoped-source.json` inputs, then original bytes were restored. Crowd commitments dispatch correctly; unsupported risk/notes, nonlocal or identity selectors, competing actors and multiple harmful operations are refused. The legal C9 route-cut fixture severs the full subtree and conserves its ledgers in the site. Checkpoint scope defers ordinary HEAL assignment; FRAGMENT assignment changes ordinary genesis, and opt-in probe HEAL remains a dormant marker. No full statistical certification or downstream consumer gate is claimed. Healing allocation and revival remain pending; the hazard/rot cadence and bounds remain after-pass balance choices.
