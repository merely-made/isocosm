# The after-pass: certifying what 732's push moved

**Retired 2026-10-18.** Mark retired this process plan. Its open items
that concern how the sim or a game behaves were carried into the plans
that own them, each under "Carried from the retired process plans
(2026-10-18)"; its certification and receipt items stay here. Relative
links below resolve from `mesocosm/design_docs/`.

**Date:** 2026-10-10

**Status, 2026-10-18:** in progress; this plan orders and tracks, and an item
is done when its owner records it. Done: C10 and C11 (site grain) are
certified (2026-10-14); A1 consumer reproduction qualifies on the unchanged
20/20 gate (2026-10-17); ordinary Mesocosm new worlds select A1 (case 961);
the A2.1 forks F1 to F8 (cases 939 to 946) are built, their failures verified
or retired. Open: the actual generated head's launch and capture (pending when
A1 qualified; see the sim-host plan's H6), smooth continuous-zoom performance,
and the A3 and A4 items under their owning plans. Next: tuning, opened by case
888, and those items.

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

**Status, 2026-10-15:** in progress; the gate is not met. Ruling 888 opens
tuning ("Tuning's open."), ending 761's and 826's deferral; candidates are
declared before they are drawn. 905 to 907 are built on branch `a1-905-907`
(grazing default, hazard under the drawn-lifespan ceiling, the named preset
`Rates::A1`; defaults unchanged). The preset as ruled (ceiling 80 to 120)
draws bare 20/20, bodied 19/20, played 18/20; with the ceiling at 100 plus a
draw (candidate 10, a change to the ruled preset awaiting a ruling) bare
20/20, bodied 20/20, played **19/20** against the stock controls' 0/20.
A1's gate stands unamended (907): the played founding must reach 20/20. The
one miss, seed 9, persists under the declared rates; the later chooser
diagnosis below identifies its missed meals and synchronized movement. *2026-10-16:* 935 rules the 100-base ceiling, and the branch
is reviewed and merged into main (947).

**2026-10-17, 958 complete matrix: A1 qualifies.** Played, bare and bodied
candidates each pass the unchanged 20/20 gate; all three Stock controls are
0/20 with extinction. Every actual round conserves matter, and final native
saves replay exactly in Individuals and Grouped for all 120 draws. The
independent complete read is `a1-2026-10-17-seeded-complete-matrix-summary.json`
in the Q3 receipt directory, pointing at six per-arm verification summaries.
The last Stock arm hit the prewrite receipt cap after seven completed draws;
its thirteen missing receipts were recovered with canonical generator inputs
and exact native saved history, omitting duplicate Genesis bytes. Original
bytes, partial failures and failed compile logs are preserved. Matrix plus
recovery raw artifacts total 95,364,456 bytes; the complete retained scope is
under 100 MB. Current release directing controls pass 52, fail 0, with 17
deliberate ignores. No rates, seed set, extinction check or qualification
criterion changed for recovery. The separate generated physical witness now
passes in Individuals and Grouped, including offer, GameSave and exact replay
(`h6-2026-10-16/generated-physical-witness-gate.json`). Under 961, ordinary
Mesocosm new worlds now select A1, retaining Stock as the shared generator
default. Actual generated headed launch and capture are still pending; the
matrix alone was not used to open that path.

**2026-10-17, 958 pilot:** new generated played worlds explicitly select
seeded movement ties; absent/older rules retain identity order and omit the
new field on serialization. Equal best due/feasible pure moves draw from
world dynamics seed, creature, tick and the tied process IDs; scores and
selected-process commitment stay unchanged. The declared seed-9 pilot reaches
180 with three boundaries, no living founders, seven born heirs and one living
born consumer. Its native hash is `c4a092289dd96a06`; Individuals and Grouped
saved replay both match, and every actual round conserves matter. At tick 16,
the 32 founded consumers split 15/17 between routes 5–2 and 5–4. This single
pilot does not close the unchanged played 20/20 gate.

The old full pilot observer unexpectedly retained 277,375,108 raw bytes,
exceeding its declared 100 MB cap. Those receipts remain preserved and the
overrun is explicit in `a1-2026-10-17-seeded-seed9-summary.json`. Pilot retention
now refuses an over-cap write. The fresh matrix uses a separate declared
compact receipt prefix, retains native replay saves and exact actual
conservation/replay summaries, and omits duplicate whole-state JSON. Its
100 MB total cap is enforced before writes; existing evidence is untouched.

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

### A2.1 The 22 already on main, triaged 2026-10-15 (931)

Branch `triage-931`, cut from main `d0a5cbc4` (the brief named `8318f21a`;
the five commits between move generator rules and docs, and the same 22
fail on both). Logs out of tree in `Code/testing/isometry/triage-931/`,
`cargo test --workspace --no-fail-fast` per workspace:

| Workspace | Before (passed / failed / ignored) | After |
|---|---|---|
| `eponym` | 103 / 15 / 4 | 105 / 13 / 4 |
| `mesocosm` | 73 / 7 / 3 | 77 / 3 / 3 |

Six fixed (category a, stale test, test-only edits); sixteen brought back
as eight forks, F1 to F8. No failure is a determinism defect: the one
`StateDiverged` replays identically from its own log (below), and the
motion archives stop at the generator check before any replay.

| Test (target) | Class | Cause | Commit, ruling |
|---|---|---|---|
| `borrowed_section` ×4 (`mesocosm-genet` lib) | a, fixed | The fixture founded a world with no map; the close-up lifts terrain (`placing::lift` needs the atlas footprint). Played worlds are mapped (783). Written that way in `a4fea109`, so they never passed. Fix: a 2×2 grid, the smallest the domain admits (`map/grid.rs:57`). | `03f87b98`, `a4fea109`; 783 |
| `disk_roundtrip_restores_attachment_into_a_fresh_game` (`eponym-client` lib) | a, fixed | `Session::begin` takes up the keeper through native `Take` (`GameState::take_up`, `state.rs:136`), a control fact the bare game log does not carry, so `GameState::save` of a taken-up game cannot restore. Replaying the 21 logged intents on a fresh and a restored world agree at every step; only the take differs. Fix: publish the `Session` save. | `e1cac333`; 779 |
| `a_reconciled_severance_shows_on_the_sheet_and_clears_the_blocker` (`--bin session`) | a, fixed | An authored fall then `ReconcileAnatomy` is refused (`StaleRevision`): a wound no longer moves the revision and a severing reconciles itself (`state/harm.rs:86-95`). Fix: the fixture's own native volley (`sever_target_limb`), the same move `8fb64d3f` made in `eponym-motion`. | 711, 717 |
| `a_stale_anatomy_stays_inspectable_and_says_so` (`--bin session`) | c, F6 | A stale anatomy is unreachable through harm: the record's revision moves only inside a severing that reconciles at once (`harm.rs:92`, `:157`). | 711, 717 |
| `treatment_preserves_severance_…`, `native_support_loss_…` (`--bin timed_action`) | b, F4 | The bin's `injure` key (`timed_action/actions.rs:75`) still applies fall plus `ReconcileAnatomy`; it is refused and injures nobody. | 711, 717 |
| `native_handler_lifecycle_keeps_authoritative_receipt` (`--bin timed_action`) | b, F5 | A volley severing a non-played target's part that wears an item is refused whole, `World(NotPlayed)`: `release_worn` (`state/acts.rs:466`, from `harm.rs:170`) submits the target's drop under the player's participant, which the contract check refuses (`contract.rs:83-86`). Without the prior motion step the volley severs nothing and resolves. | `21edca66` |
| `wrong_direction_is_a_recorded_miss` (`--bin timed_action`) | c, F3 | Native anatomy (root 8 cells, limbs to ±12) replaced the authored bodies, still spaced 12 cells: the keeper's limb already overlaps the target's root, so a backward strike hits it. | `a4fea109` |
| `terrain_hides_a_body_standing_behind_it`, `a_severed_part_leaves_…` (`eponym-client` lib, GPU) | c, F3 | Same geometry: no camera buries a whole native body; the volley now severs part 1, which only a later camera shows, so "the first frame builds geometry" no longer holds (trial edit reverted). | `a4fea109` |
| `an_unchanged_frame_produces_nothing_…` (`eponym-client` lib, GPU) | b, F7 | The producer builds a whole-ground `BrickMap` (`producer/source.rs:276-279`); the native seed-7 world has 3,062 bricks against Mere modulus's 2,047. `terrain_hides` would meet it too once a camera qualifies. | `694a3f0e` |
| `actual_pre_combat_…`, `v4_native_…`, `real_v5_…` (`eponym-motion` combat_saves, motion_compat, precise_combat) | c, F2 | Generator-1 archives are refused, `GeneratorDiverged { saved: 1, current: 4 }`; `eponym-play`'s `save_wire` tests pin that refusal, so the tree asserts both. `8fb64d3f` left "authenticated generator-1 founding and continuation" open. | `8fb64d3f`; 755, 770 |
| `a_tag_in_occurs_mid_action_under_the_pact`, `an_injury_persists_as_a_body_revision_fact` (`eponym-sortie`) | c, F8 | Nobody falls, so nobody is wounded or downed. Not a stale offset: 256 marches over offsets ±64 meet a largest one-tick drop of 1, and a drawn cliff (744) is 64, equal to the walker's `HEADROOM`, so no walker steps it. | `694a3f0e`, `1616dde2`; 744 |
| `the_shipped_pack_admits_…`, `the_manifests_file_order_…`, `the_shipped_pack_is_toml_…` (`mesocosm-phenotype`) | c, F1 | The pack ships five definitions; the native registry carries fifteen. | `635bffa7`; 759 |

*2026-10-16:* Mark took every recommendation: the six fixes merge (938),
and F1 to F8 are ruled 939 to 946 as option (1) each. Each fork is now work
for implementation. F1 to F8 are built and their triaged failures are now
verified or retired below. The separate far-window V1 correctness obligations
close in the 2026-10-17 camera receipt below; smooth zoom performance remains open.

**F1, 2026-10-16, built (939):** the shipped pack now carries all fifteen
native catalogue definitions, including the ten missing TOML files. Admission
matches the native registry exactly. Four native catalogue functions have no
box-role projection; only their canonical native ID and seeding admit an
empty role list. Unknown definitions, changed seeding and emptied nonempty
native roles still refuse. The native anatomy vocabulary is unchanged.
`cargo test -p mesocosm-phenotype` passes all 25 integration tests, including
the three original parity failures and the new refusal controls. The manifest
distinguishes fifteen catalogue definitions from five bound `Process` variants.

The forks, each with a recommendation first:

- **F1, the shipped pack's parity (3).** (1) Ship the ten other catalogue
  functions as pack definitions, so admission stays `==` native, as the
  pack's own note claims (recommended); (2) narrow the receipt to the five
  bound natives; (3) retire it under 672.
- **F2, generator-1 motion archives (3).** The world that authenticated them
  was deleted, so no native regrow can match their base hash. (1) Keep them
  as decode receipts (bytes already pinned by `save_wire`) and retire the
  three restore-and-continue tests under 672 (recommended); (2) build an
  authenticated generator-1 reader; (3) ignore them until Q5/Q11's
  authentic-fixture gate.
- **F3, fixtures spaced for authored bodies (3).** (1) Space the shared
  fixture and the timed-action app (`fixtures/session.rs`,
  `timed_action/model.rs`) so native bodies stand clear, then recalibrate
  the direction, burial and severance receipts (recommended); (2) give the
  fixtures small authored bodies again; (3) retire those receipts.
  *Reading, not ruled:* nothing in movement keeps two bodies from
  overlapping.
- **F4, the timed-action injure key (2).** (1) Injure through a native volley
  from a practice striker, `eponym-motion`'s `tests/fixtures/native_severing.rs`
  promoted to `eponym_play::fixtures` (recommended); (2) through 704's
  hazard once built; (3) remove the key and its two tests.
- **F5, worn items on a severed non-played part (1).** (1) Drop them through
  the sim's own handoff, `EponymHandoff::Dropped` (`contract.rs:292`), which
  carries no participant (recommended); (2) submit `release_worn` with no
  participant; (3) refuse a severing that would drop a worn item.
- **F6, the stale-anatomy panel (1).** (1) Retire it under 672, severing's
  self-reconciliation checked by `native_severing`'s asserts (recommended);
  (2) find a path that moves the revision without reconciling and rebuild
  the test on it.
- **F7, the producer's brick cap (1).** (1) Build the producer's terrain from
  the resident window, V1b's capacity-fixed cache (`residency.rs`)
  (recommended, unmeasured); (2) raise modulus's cap in Mere; (3) a smaller
  fixture world.
- **F8, the sortie's scarp (2).** (1) Carve the scarp as a native material
  edit in the scene, so the hazard stays a world fact (recommended);
  (2) wound through 704's hazard; (3) retire the fall-dependent receipts.

One hazard seen in passing, not a failing test: `GameState::save` on a game
whose participant has taken up a subject writes a save its own restore
refuses (`StateDiverged`). Nothing in production calls it that way today.

**F2/F6, 2026-10-16, built (940, 944):** retired three generator-1
continuation assertions and the unreachable stale-anatomy panel case.
Historical archive bytes are preserved. Current `save_wire` tests retain
byte preservation and explicit refusal of old anatomy/unknown generators;
`motion_compat` and `precise_combat` retain current motion/contact replay.
These retirement controls pass in `f-2026-10-16/eponym-runtime.log`.

**F5, 2026-10-16, built (943):** involuntary worn-item release now hands the
same item back through native `Dropped` under its actual wearer. Projection
changes follow native acceptance. A non-played wearer loses the worn part,
retains the controlled keeper, drops the exact item at its feet, and saves
and restores exactly. All 27 active `eponym-play` library tests passed; the
one genuine WorldSave2 archive-input obligation stays ignored.

**F8, 2026-10-16, built (946):** the sortie founds a six-cell trench through
one logged native material edit. Its real fall drives injury, pact tag-in,
rescue and return. All six original sortie tests and the added native-edit /
world-replay test pass; final sortie hash `2b75f82c9bfe5c65`, 125 ticks,
22 events. No fall threshold or wound is fabricated. The world edit enum is
appended so existing archive tags retain their order. Receipts:
`Code/testing/isometry/f-2026-10-16/eponym-runtime.log` (100 MB lane cap).

**F3/F4/F7, verified 2026-10-17 (941, 942, 945):** the shared practice
fixture levels its real native ground before arrival and spaces bodies at their
native bounds. The timed-action app consumes that fixture. Its injure key uses
a native practice striker and a recorded external volley; public ordinary
intent/batch paths refuse that marker. Save version 14 and its replay retain
the accepted non-played actor, separate game/native body revisions and atomic
refusals, including old-version marker rejection. Rest retains wounds under
the existing zero-healing law. Three worn-item/native-source and four external
volley controls pass after the final fixture changes.

The six timed-action cases pass across `native-level-suite.log` and
`remaining-client-controls.log`. All six GPU producer cases pass across that
suite and `producer-silhouette.log`: actual terrain hides the target while its
control remains visible; severing vacates visible leaf pixels and retains the
remaining body without static reupload. Captures were visually inspected.
The complete native workspace run before the final fixture changes qualified
motion, current archive, play and sortie cases; final affected controls then
passed separately. The last aggregate runtime invocation was interrupted and
is not a green aggregate receipt. Final all-feature/all-target compilation
passes in `all-feature-final-check.log`. Receipts and exact source hashes live
under `Code/testing/isometry/f-2026-10-16` with a 100 MB retained-byte cap.

**F7 boundary, 2026-10-16, historical checkpoint:** the ordinary producer reuses the existing
V1b fixed cache under `r1-proof`, choosing the real oriented camera footprint
and refreshing retained slots after ground edits. Its original unchanged-frame
case and three new CPU controls pass. The older `v1-proof` residency module
retains its existing feature boundary. Its far native window selects 6,901
bricks against the 2,047 atlas ceiling (1,791 under its stricter probe budget),
so `one_budget_pages_a_region_that_cannot_be_wholly_resident`,
`travel_within_one_page_band_advances_projection_identity` and
`the_stable_cache_retargets_without_changing_its_extents` remain explicit
open V1/V1b obligations. Exposing the residency implementation to the ordinary
producer does not qualify that far window or all-feature runtime. The
all-feature compiler receipt is separate. A 2026-10-17 camera-frustum probe
still selected 2,573 conservative near-camera and 5,078 far-camera AABB
candidates, beyond both capacities. The probe source was reverted and its
failed log plus `v1-far-window-analysis.md` retained. The subsequent exact
1280 by 720 pixel-ray diagnostic counts 121 near and 634 far first-hit
bricks, with no traversal exhaustion. Both fit 1,791 slots; each Ground walk
takes about 3.4 seconds in the current build. Its passed log is
`v1-exact-ray-measurement-controls.log`. This is measurement only: production
selection is unchanged and those three controls still stop at page creation.
A bounded CPU source-map diagnostic is being checked against the existing
shared DDA before any selection change. No atlas budget is enlarged and
these three tests are not deleted or renamed.

**F camera controls, 2026-10-17:** the V1 and V1b callers now select the
exact first-hit brick union for their actual camera and viewport using the
shared CPU DDA. Their CPU source map and view cache have separate bounded
budgets; GPU residency stays below the original 1 MiB cap. All nine affected
CPU controls pass, including the three previously failing paging, travel and
retarget cases, same-key Ground content refresh and atomic refusal/recovery.
Both native RTX 4060 runs complete 96 frames with inspected captures and
equal-extent travel. V1 holds 933,888 resident bytes; V1b holds 959,120 bytes,
creates no replacement textures or bind groups after initialization, and
reports zero allocator growth across samples taken after GPU completion.
The close/far recovery measurements are two frames and one frame respectively.
Uncached 1280 by 720 camera queries take roughly 1.3 to 1.8 seconds; this
closes residency correctness, not smooth continuous-zoom performance or E3.
The fresh receipt is
`Code/testing/isometry/f-2026-10-16/camera-residency-closeout.md`, with source,
binary, failed and successful log, JSON and capture bindings in its manifest.
Older failed camera and allocator receipts remain intact under the same cap.

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

- **2026-10-16, A1 seed-9 diagnosis (936), unchanged preset:** native
  Individuals reaches extinction at tick 162, conserving matter each round;
  Individuals and Grouped replay to `b6e8c6fed1207e8f`. Its full-run
  `decisions` arrays are empty because the observer read the chooser cache
  after `advance_to` cleared it. That file proves accepted acts, census,
  conservation and replay, not chooser menus. `births=1` counts the played
  parent's `Happening::Born`; the native consumer birth count is 84.
  The original file is preserved with this limitation explicitly recorded.

  Three bounded windows observe choices where made, actual feeding-pass
  feasibility and outcomes, and selected-prey residence. The 24-tick windows
  both replay to `00001e60582e3964`; the 16-tick departure window to
  `cf7e41b9090331c3`, in both native modes, with conservation each round.
  At tick 14 consumer birth doubles 32 to 64; sample actor 70 holds 17 before
  birth and 9 at tick 15, with mood -10. At 15 every consumer chooses feeding
  while a resident producer is eligible. All 52 producers at site 5 then
  depart 5 to 2 before priority 10 feeding. Every selected prey is still alive
  and carries food, but its journey departed 15/arrives 16, so no resident
  replacement is eligible and all 64 meals block. At 16 the consumers'
  `move:5-2` and `move:5-4` both score 0; identity order selects 5 to 2 for all 64.
  Producers continue ahead, and the consumers bounce 1 to 2 and back. Movement
  has no destination-food term; non-played automatic producers also migrate
  when hungry. This is a migration/cadence interaction, not proof of a
  broken 813 cache or a rate fix.

  All 13 pass-start and 5 feeding controls pass, including the intentional
  selected-blocked-meal/no-fallback rule and replacement prey resolution.
  Proposed next ruling: deterministic actor/seed/tick draws among equal
  best movement options, keeping non-movement ties, scope, rates and the
  20/20 gate unchanged. Mark approved seeded movement ties as 958; implementation and fresh
  seed-9 acceptance follow. No successful candidate or gate is claimed yet.
  Food-aware destination scoring remains a larger policy alternative.
  Declarations, original-source files matching their declared SHA256,
  compact windows and `a1-2026-10-16-seed9-analysis.json` live under
  `Code/testing/isometry/after-pass-2026-10-10/q3-reproduction/`; the four
  raw diagnostic JSON files total 2,988,360 bytes against the 100 MB cap.

  Historical `a1-2026-10-14-declaration-hashes.txt` and
  `a1-2026-10-15-*-head*.txt` citations below were absent on this laptop
  before this pass. On 2026-10-17 a read-only filename search of `~/Code/testing`
  on both `imac-intel` and `imac-m4` also found neither pattern. Archive contents
  and other locations were not searched, so deletion history remains unknown.
  `h6-2026-10-16/historical-a1-receipt-audit.json` records that scope. These
  unresolved historical citations are not promoted as reproducible evidence.

- **2026-10-14, A1 reopened under 888; the bodied arm fails on a drawn
  mouth, not on rates.** Source: main `ce01da2d` plus the test-only harness
  on branch `a1-reproduction` (`shared/isocosm/src/directing/interim/
  reproduction{.rs,/}`; it gained a form-aware profile, Mesocosm's own
  played founding, and per-arm runners). The 812 and 813 timing controls ran
  first: 22 pass-start and journey positives, two planted 813 faults caught
  (5 and 3 failures), 4 adapter positives and the planted 812 fault caught
  (3 failures), each restored to exact bytes. Declared pilots, seed 0: bare
  and played original-rate controls end at tick 41 with no births; the
  bare candidate qualifies. Declaration 1's candidate (the 2026-10-10
  meal-three/birth-fourteen profile) on seeds 0 to 19: **bare 20/20**,
  conserved each tick and replayed in both modes; **bodied 10/20**. The ten
  bodied failures are extinct at ticks 75 to 117.
  Cause, checked on all twenty bodied genesis files: `bodied.rs::drawn`
  gives a limbed consumer a jaw (`rod`) at its head instead of `block`, and
  `bodied/feeding.rs::configure` then restricts its feed process to the
  lineages that mouth's port admits (832): `function:contract` admits
  fauna, `function:intake` admits producers. All eight jawed worlds
  (seeds 3, 4, 5, 6, 11, 12, 13, 19) fed on `{lineage:1}`, their own
  lineage, the only fauna there, and all eight died at ticks 75 to 100.
  Eleven rate variants over two selection batteries (consumer meal, upkeep,
  feed period, producer yield) were all at or below 1 of 8 on bodied
  seeds 100 to 107: **rates are inert for that failure.** The unrated
  fix is the opt-in flag `Bodies::grazing` (limbs stay, the mouth is
  intake): grazer form, same rates, seeds 0 to 19: **15/20**; with the
  consumer upkeep period 10 and birth gate 9, **16/20** (the battery's 8/8
  was luck). The remaining failures die at ticks 117 to 176 after the last
  births at ticks 70 to 89, with consumers boomed to 75 to 195 on one site,
  feeding strong and births zero; a lineage's lifespan is one draw, so its
  generations die together. Receipts, declarations 1 to 7 and their
  SHA-256 are in the external `q3-reproduction` directory
  (`a1-2026-10-14-declaration-hashes.txt`, `-arms.log`, `-battery.log`).
  The pilot receipts of 2026-10-14 name pre-rebase source hashes `00fde080`
  and `a796ae67`, which are `8c22e46a` and `8367a063` after rebasing onto
  the reviewer's commits (same code). *Reading, not ruled:* the grazing
  mouth, and then a per-individual lifespan or hazard, are generator
  choices for Mark; no rate or default is promoted and A1 stays open.

- **2026-10-15, 905 to 907 built; the ruled preset draws 20/19/18, and a
  100-base ceiling draws 20/20/19 (branch `a1-905-907`, unpushed; source
  heads in `q3-reproduction/a1-2026-10-15-*-head*.txt`).** `Bodies::grazing`
  is the default (an old saved founding that never named it reads off, is
  not rewritten, and keeps its digest). The generator declares
  `ecology:hazard-<i>` (2.5 percent a tick from age 60, the engine's
  `risk`) beside `ecology:age-<i>`, whose drawn lifespan is the ceiling,
  only where the ceiling is above 60, so stock worlds carry none.
  `Founding.ecology_rates` names the preset (`a1`, written only when set);
  `Rates::Stock` is the default and is the control. Stock/preset numbers
  are pinned by `tests/ecology_rates_907.rs`; the A1 harness draws both
  through the generator and refuses a drifted control.
  *Candidate 9 tested a constant ceiling of 120 for every lineage; 906 as
  ruled says the drawn lifespan.* Mark chose the drawn ceiling on a card
  once told. Declaration 11, seeds 0 to 19, source `e360d1cb`: stock
  0/20 in all three forms; preset bare 20/20, bodied 19/20 (seed 4 extinct
  at tick 153), played 18/20 (seeds 9 at tick 142 and 15 at 176). Battery
  5 (declaration 12, bodied, fresh seeds 300 to 315): preset 14/16,
  ceiling base 90 14/16, **100 16/16**, 110 16/16, consumer birth period 10
  11/16. Candidate 10 (declaration 13; base 100, the smaller departure on
  the tie), seeds 0 to 19: bare 20/20, bodied 20/20, played 19/20 (seed 9
  extinct at tick 162); the stock controls are declaration 11's receipts
  (same worlds, same engine: the harness refuses to overwrite them).
  Seed 9 of the played founding failed under every ceiling tried: the
  constant 120 on the 890 source (extinct at tick 176), the drawn 80 to 120
  (tick 142) and the drawn 100 to 140 (tick 162). Its census: two
  cohorts of 32 consumers hop together between sites, feeding falls from
  523 to 102 accepted acts per 20 ticks while 1,859 producers stand
  elsewhere, births stop after tick 60, exhaustion deaths follow. That is
  the choice and movement policy (empty-choice timing, 813; the chooser),
  not a rate, so tuning stops there pending a ruling. Native suite on the
  final base (`4c1c51fc` on `02f8f59e`): 670 passed, 0 failed, 13 ignored
  across 59 targets; `mesocosm-runtime` 15, 6, 3 and 6 passed, including
  H3's pinned traces, which run on the 879 fixture candidate, an
  `ecology: false` founding that none of this touches. Attribution of
  moved digests: sunlight and the bodied reseed are 890 (929); the
  extinction predicate and spatial templates are 889 and 891 (932, 933);
  grazing, the hazard and the preset are 905 to 907. The 80-to-100 change
  needs a recorded ruling before the preset's numbers change on main.

- **2026-10-15, A1's candidate 9 meets the matrix; the played founding
  is 19/20 (source `7b330d02` on branch `a1-reproduction`).** After the
  grazing mouth (15/20), the lockstep reading was tested: consumer upkeep
  period 10 and birth gate 9 with the fixed lifespan gave 16/20 (gate seeds
  0 to 19), its battery's 8/8 being luck (battery 4, sixteen fresh seeds:
  13/16, and every producer or meal variant at or below it). Replacing the
  fixed age by a post-maturity hazard (60 ticks, 25,000 per million per
  tick, the engine's existing `Process.risk`, mean life held at 100) kept
  the lineage alive to tick 180 in all 20 draws but left one founder
  alive in seven (13/20 against the unchanged `living_founded == 0` and
  `born_heirs > 0` terms). With the lineage's fixed age kept as a ceiling
  at 120 (`hazardcap`): **grazer 20/20, bare 20/20, grazer original-rate
  control 0/20, played-grazer (Mesocosm's own 6-site, 256-member bodied
  founding) 19/20, played-grazer original 0/20.** Every draw conserved
  matter each tick and replayed identically in Individuals and Grouped. The
  one played miss is seed 15: extinct at tick 176 with two boundaries
  (the same late-extinction shape as before). Played births by the played
  body occur in all 20 draws, so the harness's played-births term and the
  reading without it both give 19/20. The candidate's exact profile:
  declaration 1's rates (threshold 12, meal 3 and birth period 14 for
  consumers, provision 8, upkeep 1 per 5 ticks, drawn lifespans 80 to 120
  now the ceiling), consumer upkeep period 10, consumer birth gate 9, the
  grazing mouth and the hazard. Declarations 1 to 9 and their SHA-256 are
  in `q3-reproduction/a1-2026-10-14-declaration-hashes.txt`. These
  receipts predate ruling 890's capture path (below). *Reading, not ruled:* the grazing
  mouth and individual hazard-plus-cap lifespans are generator choices for
  Mark (draft questions in the notes folder); no rate or default is
  promoted and A1 stays open until those rulings.
- **2026-10-15, ruling 890 built: producers capture light (branch
  `a1-reproduction`, `d873d320` in the receipts and `360ee3a9` after
  rebasing onto main, unmerged), and candidate 9 reproduces on it.** The `sim:energy` variant, because a reserve-account credit only
  counts when it lands in body parts (`evidence.rs`), which bare worlds
  cannot. Every ecology site holds light (seeded, then topped up by an
  agentless `ecology:sunlight` while below its cap; abundant by declaration,
  it never binds). A producer's synthesis captures one unit and spends it in
  an effect of its own, so the soil-to-tissue transform stays matter-only
  and declarable as `Synthesis`; energy is not matter (879), tissue is not
  relabelled as fuel. `tests/ecology_capture_890.rs`: default bare worlds
  and grazing-bodied worlds pass all four joint-witness roles (capture,
  return, birth, counterplay) in 60 ticks; a world without the capture
  effects fails the capture role; light never moves the matter total.
  Default bodied worlds always capture, but jawed ones can miss the
  counterplay role against the producers (the diet finding above). The full
  native suite on this source: 604 passed, 0 failed, 13 ignored (47
  targets) after two fixes the first run found: sites needed light at
  genesis (a tick-0 act was blocked "insufficient sim:energy") and a flows
  test declared `Synthesis` on every producer transform. `mesocosm-runtime`:
  21 and 6 passed. Candidate 9 rerun on this source (declaration 10),
  seeds 0 to 19: grazer 20/20, bare 20/20, grazer original 0/20,
  played-grazer 19/20 (seed 9 extinct at tick 176; the earlier miss was
  seed 15), played original 0/20. The 875 offer also needs an authored
  `JointSpec`, the independence criterion and the generated basis (879);
  that and 889/891 stay with the generator lane. Receipts total 16 GB in
  `q3-reproduction` (per-draw genesis, session, state, conservation).

- **2026-10-15, review of `a1-reproduction` on Windows; merged into local
  main as `3a338423` (ten commits rebased onto `9dbea601`), the push
  waiting on Mark.** The suites ran on the rebased branch and on main side
  by side (`Code/testing/isometry/a1-review-2026-10-15/`). The native
  suite: 655 passed, 13 ignored, against main's 648 and 10, the
  difference the branch's seven new tests, one renamed harness test and
  three more ignored harness runs. The root VTT with every feature: 484
  passed and 6 ignored on both, the same tests. Eponym and Mesocosm with
  `--no-fail-fast`: the same 103 and 70 passing and the same 15 and 7
  failing on both, the H3 pins among Mesocosm's passes. Those failures are
  main's own, unseen because earlier gates stopped at the first failing
  target or never ran these suites: in Eponym, `eponym-client`'s three
  producer tests (a scene of 3,062 bricks against a 2,047 cap, a volley
  that no longer severs, a camera search that finds no hidden body) and
  `equipment_store`'s disk round trip (`StateDiverged`), its `session` and
  `timed_action` binaries, three `eponym-motion` targets including both
  motion-archive replays, and `eponym-sortie`; in Mesocosm,
  `mesocosm-genet`'s four `borrowed_section` tests ("a world without a
  footprint") and three `mesocosm-phenotype` admission tests. *The digest
  moves.* A probe fingerprinted generated ecology worlds tick by tick with
  `sim:energy` stripped (seeds 0 to 5, bare and bodied, 120 ticks): bare
  worlds match main exactly, so there 890 moves nothing but energy. Bodied
  worlds do not: from tick 3 the same tissue sits in different parts of the
  same body. Removing the `ecology:sunlight` process, the producer's own
  capture kept, makes every world match main again, and moving the capture
  effects after the tissue changes nothing, so the sun process's presence
  shifts which part receives new tissue. Over 240 ticks that moves one
  death in three of five bodied seeds (ticks 25, 40 and 63), matter
  conserved. 890's note that "the ecology is unchanged" holds for bare
  worlds only. 905 to 907 are not in this branch. Under 929 the note is
  narrowed to bare worlds and the coupling is open: a process that moves
  no part's matter still changes which part receives new tissue, so any
  added process reseeds bodied trajectories. Under 931 a triage session
  takes the 22 failures.

- **2026-10-15, review of `lane-generator-templates` (889, 891) on Windows;
  merged into local main, the push waiting on Mark.** The one code commit
  (`b5d86d14`) was rebased alone onto main (`4cccf7dd`); its base was A1's
  pre-rebase tip, whose commits main already holds. The code applied
  cleanly and four plan status lines were reconciled. Suites
  (`Code/testing/isometry/gen-review-2026-10-15/`), against the A1 review's
  branch run, whose code main matches: native 667 passed and 13 ignored,
  the baseline's 655 plus the branch's twelve new tests; the core 48 and 5,
  unchanged; the root VTT with every feature 484 and 6, the same tests;
  Eponym the same 103 passing and 15 failing; Mesocosm the same set plus
  the branch's three runtime tests, with one H3 pin failing. *The digest
  moves.* A probe over fourteen generated worlds (plain, bare and bodied
  ecologies, two played map worlds): every genesis digest moved, and with
  `global_collapse` and `spatial_templates` removed each equals main's;
  per-tick population, sites and event counts match main over 240 ticks
  (30 for the map worlds), the worlds that die out included. The state hash
  folds the genesis in, so it moves everywhere. The failing pin was that:
  its fixture world now declares the collapse predicate, every recorded
  action and outcome replayed unchanged, and the two traces were rerecorded
  for their final hash alone. A small mismatch for the generator lane: its
  overlay-plan text says `Game::found_generated` seats "the first embodied
  member of each cohort", while the code seats up to four residents per
  site in id order. Nothing calls `found_generated` yet but its tests.

- **2026-10-11, Q3 fresh bare seed zero classified:** the original arm
  ends at tick 41 without births; the independently frozen 454/813 candidate
  reaches tick 180 with three played births, eight played heirs and three
  played boundaries. Its native record contains 81 birth joins, comprising
  28 children of founder parents and 53 children of born parents. Their
  generations are 28/34/18/1; native births by epoch are 55/26/0. Only one
  born consumer remains alive at tick 180, with no third-epoch birth. Native
  birth joins and the three played births are distinct measures. Root
  recomputed the full receipt/state/history classification; review
  `q3-consumer-birth-root-review.json` in the external reproduction directory
  has SHA-256
  `3fd4a11f341755cfddb6c025513ccf2458111ae51839504ce6132dfcfeef1477`.
  The retained Session replays its final saved state in Individuals and
  Grouped modes; conservation was checked each tick. This is one bare seed,
  not a per-tick replay or an A1 matrix certification. The bodied seed-zero
  proposal and its strict parser are source-qualified; actual post-HEAL
  source refresh, original/candidate bodied pilots and the remaining draws
  are still required. No rate/default promotion is made.

- **2026-10-11, Q3 supply classification root-verified:** all 39 referenced
  files, 173,364 contiguous observer rows and all 267 restored native inputs
  match. Root independently confirms 68 same-pass mutated-target choices,
  37 empty choices before later food at tick 111 and 1,274 local soil blocks.
  External `q3-reproduction-meal3-birth14-supply-trace-classification-root-review.json`
  has SHA-256 `e3cff2a5943ab3f4d638f680ecffdb1d3468ff391b5fd3ab3d0997c2d311368f`.
  The diagnostic conserved both fresh arms and passed both saved replay modes;
  the candidate remains rejected at 125 and original control ends at 41.
  Existing 454 authorizes the bounded pass-start body/index correction; its
  source is preparing separately. Empty-choice timing is a pending user fork
  after 812. No rate/default promotion or full A1 gate follows this diagnostic.
- **2026-10-10, Q5 fresh failure classification:** the integrated consumer
  source at `21edca66` yields 55 passing, 43 failing and three ignored tests,
  with ten failed targets. Root rehashed all 620 source/config/archive inputs
  before, after and against the clean worktree. Thirty-five failures stop at
  setup: the live `Ground::surface` query scans only through height 24 while
  native terrain has taller solid voxels. Six producer tests use the current
  tenant renderer and retain their live invariants. One room receipt assumes
  cube carving where native `WorldIntent::Carve` uses a sphere; four remaining
  fall receipts need real terrain calibration. Three actual consumer archives
  fail outer decoding before native admission or replay. A bounded surface
  query/room receipt source repair is granted, with later Cargo gates withheld
  until review. Archive bytes remain immutable; rulings 608/641/651's native
  reader obligations do not imply every deleted consumer grammar must migrate.
  External `q5-eponym/baseline-root-review.json` records the root audit.
- **2026-10-10, Q8 verifier repair stopped before reanalysis:** thirteen
  fixed-vector and enumeration controls passed, but the first corrupted receipt
  hit a Python `UnboundLocalError`: an existing local `draw` variable shadows
  the new seed helper. This interpreter error is not a caught corruption.
  The failed source and raw attempt are retained separately. Only a helper
  rename and separately frozen control envelope are granted next; no changed
  native model, thresholds, historical pilot status or larger draw run follows.
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

- **2026-10-14, valid C10 rejection:** frozen pushed source6b5d00a0 completed
  the one fixed1000 release draw and independent reader. Grazer meal holdings
  differ (distance0.036, Holm p0.0245) despite equivalence0.2; positive and
  negative controls, refusal/injury/conservation/integrity gates pass. C10
  remains open. Next owner task is to diagnose `body:graze-1#meal-holdings`
  against the actual saved paired worlds and native/crowd feeding paths;
  retain this dataset and criteria and identify a concrete defect before any
  code adjustment or fresh certification. No seed/threshold tuning or reroll.
  C11 stock-clock kernel passes independently; autonomous chooser/title scope
  remains open. Full queue/headed/reproduction/ecology acceptance is separate.

- **2026-10-14, C10 defect identified and fixed:** the saved paired worlds
  show only `body:graze-1#meal-holdings` shifted (paired crowd minus exact
  +229 per world, z 6.4; exact-control minus exact -2, z -0.06); the crowd
  also blocked 246 more acts per world (z 6.1). Tracing every `graze-1`
  attempt on four C10 worlds found the core accepting all but three
  unplanned attempts and the crowd refusing 425, every one a coincident
  bite, on prey of mean mass 33.5 against about 190 for accepted meals.
  Cause: core `8548cd2d` (Q3, 454 and 459) shares a bitten part's shortfall
  among the pass's bites before any act writes; the crowd hunt
  (`shared/isocosm/src/probe/crowd/hunt.rs`) shared only the whole prey and
  then refused bites its drawn part could no longer hold. The crowd now
  draws every coincident bite's part first and shares each part's
  pass-start holding (`part_quotas`). Traced refusals fell to zero and the
  crowd's mean prey mass returned between the two exact arms. Two tests
  were added (part quotas mirroring the core's four sharing cases; one-tick
  coincident grazing counted against the core); two planted faults (no
  part quotas; a per-bite cap) were each caught and exact bytes restored.
  The diagnostic instrumentation was temporary and is not committed. Raw
  traces and fault logs: `Code/testing/isometry/c10-diagnosis/`.
  Recertification uses fresh seed 2026101401 (880) under unchanged
  criteria; its harness is `Code/testing/isometry/c10-recert-2026-10-14/`.

- **2026-10-14, C11 certified at site grain:** the crowd's own territory
  chooser (881) compares exact with native over 48 drawn deliberative and
  free timelines, with priority, tie, feasibility, free-pass and refusal
  cases; eleven compiled controls caught with exact restoration (a first
  attempt caught nine, one blind fixture and one equivalent mutant, both
  replaced). Native library 238, `checkpoint11_spread` 10, probe binary 7
  and the all-target check pass at lane head `d795d758`. Titles stay outside
  C11 (882) and nudged directing native-tested. Receipts:
  `Code/testing/isometry/c11-chooser-2026-10-14/`.

- **2026-10-14, C10 certified:** the fixed1000 recertification ran at
  `8cbe3e76` under seed 2026101401 (880): native exit 0 after 10,097
  seconds with stable source; the independent reader certifies it with no
  failure. Main is equivalent with no detected difference and no refusal;
  grazer meal holdings read 12304.8 exact against 12319.0 crowd (p 0.73);
  the positive control passes and the averaged and unweighted controls are
  detected. Raw SHA-256 `bb54e8f9e4b323469a35a29614ccb638921d7f6259252da18d94a28db4fdf948`, verdict `ca015d2e9b0eed828202f530da49ad12c5195f427abbf8c2bcf7e6a2b0f15b2b`, in
  `Code/testing/isometry/c10-recert-2026-10-14/`. Main's later commits
  (`d1b78f3d` to `77d73505`) touch only the spread crowd, which the harm
  worlds never construct. The nineteen C10 fault targets are byte-unchanged
  since `6b5d00a0`, so their source bridge holds with its earlier scope.
