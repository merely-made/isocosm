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

- **2026-10-10, Q3 food-access reconstruction:** unchanged saved-native
  flows and accepted watches reconstruct all 299 non-participant final
  ledgers, places and alive flags and three site accounts exactly. At
  feeding tick 123, 64 living producers hold 191 edible units; final tick
  125 retains 64 at site two with 127 units. Final soil totals 4,451.
  Producer production/migration cadence leaves replenished soil zero
  unused after tick 16 and soil one after 14; subsequent intake is at two.
  Thirteen of fourteen feeding passes from 84 through 123 lack eligible
  co-located prey for every consumer. At 111, 37 consumers share site two
  with 74 eligible producers holding 225 units, but no meal is accepted.
  This last absence's actual cached Choice/Foregone/Plan reason remains
  unobserved. Decomposers lack positive eligible carcasses through 48 and
  all exhaust at 50. Reading, not ruled: inspect actual first-choice and
  pass-frame context before changing rates, movement or choice policy.
  The root-verified external `q3-reproduction-meal3-birth14-supply-review.md`
  has SHA-256 `4c95535cb177dd03afaa363888f8ab368a156d6813c9f4f701130e62b049a394`.
  A diagnostic will retain fresh candidate/original receipts, exact
  candidate Genesis/state-hash agreement and all conservation/replay
  requirements. Its source and execution remain separate gates; A1/A4 stay open.
- **2026-10-10, C10 pilot stopped by independent statistics:** the frozen
  release artifact runs seed `2026101001`, twenty draws and all five arms.
  All arms refuse zero draws; the forty native arm-runs conserve unassisted
  matter over 2,400 checked tick observations. Rerun/collection controls
  pass. The frozen Python verifier exits one: for the unweighted control's
  `body:hazard#meal-holdings`, the Holm-adjusted permutation result is
  `0.0245`, versus Rust's `0.0735`, straddling alpha `0.05`. Its comparison
  and top-level detection decisions therefore disagree. Main and positive
  equivalence are not established on twenty draws. This is a failed
  diagnostic, not a C10 certification; larger runs stay withheld. The
  external raw `q8-c10-harm-20.json` SHA-256 is
  `4dc3803c9d863b7d38d1421e0fe93d8e8a7de0161d9b2d64c9a7ab8e097eb247`.
  Root independently recomputed refusal and conservation counts and inspected
  both decisions. Source-only sampling review follows without changing the
  frozen evidence, thresholds or native model; ruling 812 remains open.
- **2026-10-10, meal-three/birth-fourteen pilot rejected:** the frozen bare
  seed-zero profile changes only the candidate consumer's birth period
  from seven to fourteen. Extinction at tick 125 leaves three played-parent
  births, three born heirs and two boundaries. Conservation at every tick
  and both saved replay modes pass before the final qualification fails.
  Native provision flows show 71 lineage births, including 45 by twenty
  born parents. All 79 members die: fourteen accepted age acts and 65
  born-member exhaustion acts. Founder 17 and child 119 retain four and
  two units at their age deaths; child 119 reproduces three times. Positive
  feeding ends at tick 81; from 82 through 125 both accepted feeding
  watches and flows are absent. Earlier zero-credit shared meals persist.
  Accepted-only observations cannot identify each later absence's cause.
  The slower cadence reduces births and peak members, but neither this
  profile nor the preserved birth-seven profile qualifies. This is a
  Reading, not ruled; generator defaults and matrix gates remain unchanged.
  All 267 inputs stay frozen. The thirteen-path external manifest
  `q3-reproduction-meal3-birth14-6f879936-seed0-pilot-receipt-manifest.tsv`
  has SHA-256 `8db610f23949ef5e7d864db743069d3ad2399b345f880c177e2163aeced40ebe`.
  Root recomputed every receipt hash/size and the native death census.
- **2026-10-10, unchanged sortie trajectory qualified:** the preserved
  diagnostic runs the shipped `[-15, -8]` site and `[[-16, 16]]` route,
  returning after 85 ticks with 86 complete trail rows (13 through 98).
  Root independently recomputed maximum drops: Aud/Bram/Sela one cell,
  Odris zero. None exceeds `SAFE_FALL = 4`; no Fell, Wounded, Downed,
  PactInvoked or TaggedIn event occurs. The earlier two sortie failures
  therefore catch absent scene hazards, rather than a refused native wound
  handoff. All nine scoped Eponym contracts passed, including independent
  resolved Blow-to-Wound. Q5 owns scene recalibration and renewed six-sortie
  qualification. No coordinates or expectations changed in Q2. External
  `q2-contracts/trajectory.log` SHA-256 is
  `9501ec92f26386f0f127337b89672fd8da752a7b79b0283badee48c796cb3d32`;
  the earlier four-of-six raw result remains preserved.
- **2026-10-10, Q10 body revision preflight:** Q2's source review finds the
  same revision rule at common base `c6bc153a`: native harm increments
  `Entity.body_revision` per successfully severed nonroot branch, while
  Eponym stamps `BodyRecord` and admitted anatomy once per complete cut
  batch. Two independent severings can therefore yield native plus two
  versus local plus one. Local equipment/anatomy staleness checks compare
  local records; this is not a new Q2 regression or an executed fixture.
  *Reading, not ruled:* the local number behaves like an admitted snapshot
  generation, but no explicit association with the native body revision
  was found. Q10 must reconcile the native final address with admitted
  anatomy, including allocation/revival refresh, and qualify a two-cut
  handoff, save/replay and stale/current equipment checks. The external
  `q2-contracts/native-revision-followup.md` preserves exact source sites;
  SHA-256 `ffa985d8ec28974da7c171f93a1f833c342846041cc94980d97b0d9a3dd84caa`.
  Root compared current and common-base source; Q2's frozen inputs stayed
  unchanged and no Cargo ran for this review.
- **2026-10-10, Q6 protocol preflight:** source inspection only. The native
  contract work leaves `isonetry::PROTOCOL_VERSION` at four and the iroh
  ALPN at `isometry/session/v4`. Their source contract requires both to
  change together whenever the postcard wire shape changes. Existing H0
  tests refuse `PROTOCOL_VERSION + 1`, exclude the peer, and prevent host
  logging or client state adoption; a matching-version positive also exists.
  Q2's two current-wire replication tests do not qualify prior-version
  refusal. Q6 must bump both identifiers to five and qualify refusal of
  version four after Q2 integrates, alongside the fifteen dated VTT test
  failures. Q9's later native-Session projection needs another wire-shape
  audit; this planned bump does not prequalify future changes. No protocol
  source changed and no Q6 Cargo gate ran.
- **2026-10-10, rejected meal-three classification:** native accepted watches
  and paid provision flows establish 167 consumer births, including 140
  birth acts by 108 distinct born parents. Child 82's own-matter ledger is
  `8 provision + 25 feeding - 13 upkeep - 16 child provision = 4`;
  founder 17's is `10 + 29 - 14 - 24 = 1`. Both accepted native age acts
  at their deaths (101 and 108). All 175 lineage members died: 45 from age
  and 130 from native exhaustion, of which 38/129 respectively were born
  members. Two tick-54 meals divided ten units across 28/32 accepted hunters
  and credited zero. Native consumer feed credits stop at 67 to 119 and
  resume with 23 meals at 120. Postpass region readings repeatedly hold
  zero producer biomass on meal ticks; they are not feeding-start snapshots.
  The frozen accepted-only observer cannot assign every missing meal to
  absent eligible prey, foregone Choice or a share yielding zero. No new
  causal-policy diagnosis follows. Root verified the twelve-path receipt
  manifest, native birth/age watches and feeding gap; manifest SHA-256
  `430614d1e1cbe4797a7df4ba1ed7a6f843994403bc92bfc64117289064f4c5b8`.
  *Reading, not ruled:* the next source preparation changes only consumer
  birth period seven to fourteen, aiming to reduce multiplication and
  provision debits while preserving meal three. It can also impair renewal
  before age deaths. No trial ran; all other settings and gates remain fixed.
- **2026-10-10, Q3 meal-three pilot rejected:** one fresh bare seed 0 ran on
  native inputs from `6f879936`; all other matrix arms remained withheld.
  Genesis `2e043c6982620a12a2742a9e09a03ee2ea39381770df6e18b9b67e55d6537b72`
  ended at tick 129, with three actual played-parent births, four born heirs,
  two configured boundaries and no living consumer. Tick conservation and
  Individuals/Grouped saved replay passed before the qualification failure.
  Saved holdings first show founder 17 dead at tick 101 with one own unit,
  and child 82, born at seven, dead at 108 with four own units; their peaks
  were fifteen and fourteen. These differ from the prior meal-one starvation
  balances. Native lifespan and birth-opportunity classification remains in
  progress; born heirs alone do not prove that born consumers reproduced.
  Full Session, state/events, holdings/flows and outcome are retained under
  the complete Genesis key in the external Q3 directory. Raw pilot log
  SHA-256 `0de50ae738a73abde4a01f85384311c6df84beacc67302a423698e23d191c979`.
  No source or rate change followed the rejection; A1 remains open.
- **2026-10-10, Q2 sortie gate:** four of six tests passed. The two failures
  stop at missing `PactInvoked` and missing Aud wound, after the original
  agreement-link failures were fixed. `eponym-sortie/src/sortie.rs::walk`
  calls native harm only after a drop greater than `SAFE_FALL = 4`, then
  downs the home body; `advance` requires that downed body before the pact
  fires. A refused native wound would panic earlier inside `wound`, which
  did not happen. `scene.rs` explicitly says its calibrated generated-terrain
  route should fail when a regenerated world no longer supplies the cliff.
  *Reading, not ruled:* stale terrain calibration is consistent with both
  failures; a preserved trajectory diagnostic and generator history audit
  must establish it before Q5 repairs the scene. No coordinates or hazard
  law changed. The raw six-sortie log SHA-256 is
  `4555e3248e6f5bbb94a516203b72f0a39439bcc7609c994c9344ff49415fabed`,
  under `Code/testing/isometry/receipts/2026-10-10/q2-contracts/`.
- **2026-10-10, Q11 headed preflight:** read-only, without Cargo or a headed
  run. Pinned Genet `15713014` exports `taproot::Scenario`; Mesocosm's
  `mesocosm-genet/src/app/drive.rs` pumps it after rendered frames, routes
  named actions through the host, and captures through the host. No OS
  input is needed. Pointer delivery is explicitly unrouted in this host;
  its named-action path is available. `HostConfig` and the CLI currently
  lack a play-mode setting: `mesocosm-runtime/src/runtime.rs` founds Creative
  play and `mesocosm-genet/src/played.rs::resume` also selects Creative.
  Q11 must expose a mode setting and certify Survival and Creative (691)
  separately from native Individuals/Grouped replay. Q3/Q4 prerequisites
  remain open; this inspection supplies no headed receipt. D1 commit
  `f76a79044af0b285b666afe755f9a3188e3fd3e2` has parent
  `d4d7f0d516dcf10af7b2f596ddf82905f2f696d7`: its native `history.rs`
  writes `SAVE_VERSION = 3` and its tree contains no directing module.
  Its saved fixtures are v1/v2, without a pre-D1 v3 fixture. Producing one
  on that actual historical source remains Q11's authentic-fixture gate;
  a save assembled with today's source does not supply that provenance.
- **2026-10-10, Q4/Q5 archive preflight:** source inspection only; no reader,
  fixture or test was changed and no Cargo gate ran. Q4's v1 checkpoint
  fixture carries the former Part fields `parent`, `traits` and `severed`;
  current physiology is keyed to isometer geometry under 674. The Process
  causation field still reads its former `shape` spelling. The two known
  historical witness failures stop at the genesis digest, before checkpoint
  checking. Kept-reader guarantees under 608, 641 and 651 require a reader
  audit; 672 alone does not classify those failures as retired tests.
  Q5's three Eponym motion archives decode as `TimedActionSave`, wrapping
  `SessionSave`, `GameSave` and `WorldSave`; their three failing tests stop
  at decoding before `restore_solving`. The v3 fixture is the actual headed
  `b9ae9a2` output identified in `combat_saves.rs`. That commit's
  `paredros/crates/paredros-world/src/transitions.rs` carries `AdmitAnatomy` with a
  `mesocosm_core::BodyDocument` (`species/root/plan/parts`); today's
  `eponym/crates/eponym-play/src/transitions.rs` carries `isocosm::lineage::LineageBody`
  (`doc/species/lineals`). The preceding GameIntent variant order is
  preserved. *Reading, not ruled:* payload drift is consistent with the
  decoding failure, but its cause is not yet proved. Today's world reader
  requires generator 2, so a decoded generator-1 world would still refuse. Preserve
  the raw archives and qualify decoding, admission and replay separately
  in Q5; the recorded 47 failures remain a dated baseline, not a fresh run.
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
- **2026-10-10, Q8 known-loss healing evidence:** the opt-in HEAL trait now
  works under 810 and 811; the native all-target check, 31 focused harm tests
  and nine compiling planted faults passed their final restored controls.
  The exact source and receipt scope is in `CHECKPOINT_10.md` under
  `testing/bench/receipts/2026-10-10/isocosm/`. Known function/free losses
  heal while unknown IDs remain; original tombstones revive with paid new
  cells and no matter copied back from a fragment. Empty Part loss metadata
  preserves JSON/postcard witness preimages, a narrower result than archive
  compatibility. The historical v1/v2 tests still stop at the same genesis
  digest gate as A2: 8/10 witness tests passed, two unchanged failures stay
  with Q4. Fresh v3 loss-bearing save/replay and tamper controls passed.
  Shared BodyFounding definitions change generated rules/genesis even
  without HEAL; ordinary `bodied.rs` HEAL assignment remains deferred.
  Adapter allocation (812), ordinary assignment, downstream gates, remaining
  fresh faults and statistical crowd certification remain open. Hazard/rot
  bounds and cadence remain after-pass balance choices. Cargo released.
