# The sim's execution model: waking, parallel evaluation, near and far

**Date:** 2026-10-19

**Status, 2026-10-23:** D5, D6 and D7 built (Progress); D9 and D10 next
(deliberation and planning, D8), then E1. Its decisions are linked to the
doctrine (Processes 6 and 7, Directing 4 and 5, Schema 9, Mesocosm 5). Specced at Mark's request after a
probe was stopped for blurring design, implementation and qualification. The
three are kept apart below. Mark answered the four open questions the same
day (Decisions, below).

## Why

Judging mechanics in combination means watching generations across several
seeds, so the sim's speed bounds the design loop itself. A played tick costs
about 0.45 s in the head's seed-9 game (about a hundred members). In the
played A1 battery (seed 0, release, measured 2026-10-20) the 180 played
rounds average 0.46 s a tick (median 0.38 s, p90 1.14 s, max 1.76 s): about
0.8 s over ticks 31 to 90, falling to 48 ms over ticks 151 to 180. All of it
stands against the
design tempo of 0.1 s (TD2; the head runs at 2 ticks a second under case 978).
Cost grows with population because every member is reconsidered for every due
act, and every mechanic added raises it: strain, relief and rest made the
played battery about 45% slower. The aim is a played tick at or under 0.1 s
with a few hundred members.

## Decided already

The near/far model was designed in September; this plan carries it out and
does not reopen it.

- **Similitude (113).** Noted things always run as individuals; the fungible
  agree in distribution; what was watched is logged; exact agreement is a
  world setting, statistical the default.
- **Background and foreground (75).** They agree; the background is cheaper by
  aggregation; transition losses preserve similitude; things of no note are
  fungible behind a configurable buffer.
- **The boundary (211, 212).** An attended group runs its noted members exactly
  and its fungible members as a crowd within tolerance. What the view examines
  up close runs in detail and is logged; far views stay crowd.
- **The crowd (206, 207, 219).** A histogram of exact states with random count
  draws; under shortage, pairwise encounters resolve choices, and the pairing
  draw is the only approximation; query and effect meanings exist once, shared
  by crowd and individual runners.
- **Rate models (261, 266, 440).** Derived where effects commute, a rate model
  per site and lineage where members share resources, targets or
  relationships; the exact crowd runs wherever it still finds identical
  members; rate models are calibrated once per rules digest and ship with the
  rules, and uncalibrated worlds stay on the exact crowd.
- **Crossings (435, 554, 431).** A cohort tallies members per condition cause;
  lifting draws a cause by seed. Parents and unweaned young stay individual.
  Collective modifiers split no crowd.
- **Where the line lives (742, 293).** The tier line belongs to directing;
  per-group phases run in the foreground, the background keeps a shared
  period grid.
- **The crowd's chooser (881).** Autonomous territories use the crowd's own
  chooser and refuse live nudges.
- **Budget (259, 285).** The operation budget counts the evaluations that run,
  and an evaluation counts the members it stands for.

## Design

What the sim does, independent of code.

### Waking

A process is reconsidered for a member only when something it reads about that
member has changed since the member's last outcome for it, or when a time gate
ripens. Unchanged members keep their last outcome. Three kinds of evaluation
always run regardless: nudged deliberative members (their bond and answer
update even when blocked), terrain processes (they capture geometry even when
blocked), and anything whose reads cannot be stated statically. Skipping an
evaluation that would have been blocked changes no act and no draw, because
only accepted acts advance the act counter.

### Parallel evaluation

Within one process pass, members' evaluations read only pass-start state
(454) and can run concurrently. Commits run serially in the existing order
(groups by first id, members in turn), so act numbers, child ids, appended
notes and events, and the budget's early exit come out exactly as in a serial
run. Results never depend on how many threads ran. Draws keyed on the global
act counter (amounts, bites, risk, newborn bodies) cannot be resolved until the
serial order assigns act numbers, which is the first open question.

### Near and far

As decided above. Near is what attention roots (played, pinned, examined, and
their cared-for) plus the tier line's hysteresis (promote within one hop,
demote at two or more; `directing/tier.rs`, unwired today) select. Near runs as
individuals through the exact scheduler. Far runs as the crowd per site and
lineage. A member crossing far to near is drawn from its bin by seed with its
condition causes drawn from the tally (435), and keeps every identity the game
has made pointable; near to far folds it back, conserving totals. Noted members
never cross into a crowd (113).

## Implementation

Phases in order, each with its done-condition.

- **D7a. Parts by `Arc`.** `Entity.parts` holds each part by `Arc`; a write
  takes the part with `Arc::make_mut`. Done when the exact check below
  holds and the battery's time is read against D5's.
- **D7b. A cheaper gate index.** Re-filing reads a group's books once, and
  moves are filed by the site their actor lives at. Done when the exact
  check holds and the debug filing check (`passed_by`) still passes.
- **D9a. A narrower prey scan.** Candidates indexed by the test that
  rejects most of them, found first by a probe. Done when the exact check
  holds and the debug comparison against a full scan still passes.
- **D9b. Unchanged re-choices skipped.** Dropped (D11): under 1% after D9a.
- **D11. The prey scan narrowed further.** Living and dead filed apart;
  prey-independent requirements tested once per search. Done when the
  exact check holds and the debug comparison with a full scan passes.
- **D10. The planned stage kept.** Done when the exact check holds.
- **E1. Waking.** The per-advance gate index (`schedule/filed.rs`, rulings 258,
  286) becomes persistent across advances, re-filed at commit from each
  process's read-set (`Gates::of`, `framed`, `Expr::reads`). Done when the
  played world's ticks skip members whose reads did not change, and the
  exact check below holds.
- **E2. Draw keys.** Only if Mark chooses order-free draws (question 1):
  amount, bite, risk and soma draws keyed by actor, tick and process, as
  movement ties already are (958). Done when the pins are rerecorded under
  that decision and A1 holds at its measured rate.
- **E3. Parallel evaluation.** Evaluate a pass's members concurrently against
  pass-start state; commit serially in order. Done when the exact check holds
  at one, two and all threads.
- **E4. The crowd in the core.** The probe's crowd (`probe/crowd/`) becomes an
  execution form for far sites, keeping the shared query and effect meanings
  (219). Identity-bound processes stay on the exact tier. Done when the
  envelope check below certifies on Mesocosm's played worlds.
- **E5. Crossings and the tier line.** Wire `TierLine` into directing (742);
  lift from a bin by seed and fold back conserving totals. Done when the
  invariants below hold across crossings and the envelope still certifies.
- **E6. Rate models.** Later: calibration per rules digest (440), for shared
  resources where the crowd stops paying (261, 266).

## Qualification

How each phase is known to be right, fixed before it is built.

- **E1, E3, and E2 once its draws are rekeyed: exact.** A pure execution change
  leaves every state hash identical to the serial, unskipped run: the H3 pins
  6/6 without rerecording, and the played A1 battery's first seeds printing
  identical lines. Any difference is a bug.
- **E4 and E5: invariants exact, outcomes in distribution.** Matter is
  conserved every tick, noted members keep their identities, and every
  crossing conserves totals. Outcomes against the all-individual world are
  judged with the checkpoint 10 harness: paired worlds, Kolmogorov-Smirnov
  distance, paired permutation, DKW equivalence within the reading's bound,
  Holm-corrected, as it certified C10 and C11. A pass is a run that passes; it
  is not a receipt to keep.
- **Speed** is read from the same runs: ms per played tick in a grown world,
  once per phase.

## Questions put to Mark

1. **Draw keys.** Keep act-number-keyed draws, so evaluation runs in parallel
   but draws resolve at the serial commit (less speedup, no hash change), or
   rekey them by actor, tick and process so evaluation is order-free (full
   parallelism, one deliberate hash move, outcomes the same in distribution).
2. **Exactness setting for Mesocosm (113).** Statistical by default, or exact
   for Mesocosm's worlds, which would keep the crowd out of its played worlds.
3. **What near means.** Attention roots plus one hop from the played critter's
   site, as the tier line's hysteresis has it, or a different reach.
4. **Order.** Speed first (E1 to E3, exact, no design risk) or the tier first
   (E4, E5, where the largest saving lies once worlds grow).

## Decisions

Mark's answers, keyed `sim-execution D1` to `D6` in the order he gave them
(keys are mechanical under the decision-links plan, Mark 2026-10-20: "Key
the six now, build L1–L2 next"). Their links and outcomes were recorded on
2026-10-22 (that plan's L5), Mark choosing each outcome in one card round;
each title names its outcome against the doctrine.

- **D1, draw keys (2026-10-19), refines Processes 7:** "Actor, tick and process (Recommended)". Amount, bite, risk
  and soma draws are keyed by actor, tick and process, so evaluation is
  order-free; one deliberate hash move, outcomes the same in distribution.
  E2 is in.

  Links: area Processes and rules · draws on 454, 958 · governs isocosm::execute, isocosm::stage · summary Amount, bite, risk and soma draws are keyed by actor, tick and process, never by commit order, so evaluation is order-free; one deliberate hash move.

- **D2, exactness (2026-10-19), applies Processes 6, refines Mesocosm 5:** "Statistical (Recommended)". Mesocosm's far sites run as the
  crowd; the played critter and every noted member always run as
  individuals.

  Links: area Processes and rules · draws on 75, 113, 211 · governs isocosm::probe::crowd · summary Mesocosm's worlds are statistical: far sites run as the crowd, the played critter and every noted member as individuals.

- **D3, near (2026-10-19), refines Directing 5:** "Roots plus one hop (Recommended)". Attention roots and everything
  within one hop of the played critter's site; promote within one hop, demote
  at two or more.

  Links: area Directing and play · draws on 212, 742 · governs isocosm::directing::tier · summary Near is the attention roots and everything within one hop of the played critter's site; the band is owed before E5.

- **D4, order (2026-10-19), refines Processes 7:** "Speed first (Recommended)". E1, E2, E3, then E4 and E5.

  Links: area Processes and rules · draws on 978 · governs isocosm::schedule · summary Speed first: waking, draw keys and parallel evaluation (E1 to E3) before the crowd and the tier line (E4, E5).

- **D5, stage bodies by `Arc` (2026-10-20), applies Schema 9:** "Accept A, you number it". The
  stage holds the `Arc<Entity>` that `body_shared_at_start` returns and
  copies a body only on its first write, so planned and blocked acts copy
  nothing and commit installs the staged `Arc` instead of copying again in
  `Population::lift`. Carries ruling 975 into the stage; ruling 338's reads
  are unchanged; serialisation, hashes and saves unchanged. Whether `Part`
  also goes behind an `Arc` comes back once this is measured.

  Links: area The sim's schema and saves · draws on 338, 975 · governs isocosm::stage::Stage, isocosm::population::Population · summary The stage holds each body by the Arc it starts with and copies it only on its first write; serialisation, hashes and saves unchanged.

- **D6, `foregone` only on direct acts (2026-10-20), applies Directing 4:** "Accept B, you number it".
  `run_inner` fills `Receipt.foregone` for `Run::Act` only; scheduled and
  planned receipts, whose callers keep only `.accepted()`, carry an empty
  list. Law A's record and ruling 685's reading are kept receipts, which
  still carry it.

  Links: area Directing and play · draws on 685 · governs isocosm::execute, isocosm::simulation::Receipt · summary A receipt's foregone list is filled for direct acts only; scheduled and planned receipts carry an empty one.

- **D7, parts by `Arc` before E1 (2026-10-23), applies Schema 9:** "Part by
  Arc, then gate index (Recommended)". A body's parts map holds each part
  by `Arc`, so copying a body copies handles and a write copies only the
  part it touches; serialisation, hashes and saves unchanged. Then the gate
  index re-files a group reading its books once, not once per gate, and
  files moves by the site their actor lives at. E1 comes back after both,
  with new figures. This settles D5's open half and amends D4's order on
  the measured split (Findings, 2026-10-23, where an accepted act goes).

  Links: area The sim's schema and saves · draws on 975, sim-execution D5 · amends sim-execution D4 · governs isocosm::schema::Entity, isocosm::schedule::Filed · summary A body's parts are held by Arc and copied only when written; that and a cheaper gate index come before E1.

- **D8, deliberation and planning before E1 (2026-10-23), applies Directing
  4 and Processes 8:** "Deliberation and planning". After D7, a
  deliberative member's choice and a shared-ground pass's plan are each
  about 15% of scheduling (Findings, 2026-10-23, after D7). Each is probed
  first; anything that would change how a choice is made or kept (683,
  813) or how shared ground is shared out (454) comes back as a question.
  E1 waits; this amends D7's order.

  Links: area Directing and play · draws on 454, 683, 813, sim-execution D7 · amends sim-execution D7 · governs isocosm::directing::choice, isocosm::schedule · summary After D7, deliberation and a shared-ground pass's plan are probed and cut before E1, keeping how choices are made and ground shared.

- **D9, deliberation cut without changing a choice (2026-10-23), applies
  Directing 4:** "Narrow the prey scan (Recommended), Skip unchanged
  re-choices". A prey search visits only the candidates the cheap test
  that rejects most of them would pass, indexed by that test, so the same
  prey are accepted and the same draw made; and an empty choice is scored
  again at a later pass start (813) only where the world has changed since
  it was last scored, every change bumping a revision. Scoring feeding on
  any acceptable prey, which would change the prey a suggestion names, was
  not chosen.

  Links: area Directing and play · draws on 683, 813, sim-execution D8 · governs isocosm::targets, isocosm::directing::choice · summary A prey search visits only possible prey, and an empty choice is scored again only when the world has changed since.

- **D10, a shared-ground act keeps its planned writes (2026-10-23), applies
  Processes 8:** "Reuse the plan's stage (Recommended)". When an act's
  shares cover what it planned to take, it keeps the stage its plan staged
  and only re-checks that it fits and commits, rather than evaluating
  again; this holds only if effects read nothing but the pass's start, and
  the exact check decides it, any difference coming back as a question.

  Links: area Processes and rules · draws on 454, sim-execution D8 · governs isocosm::schedule, isocosm::execute · summary An act whose shares cover its plan keeps the planned stage and only re-checks fit before commit.

- **D11, D9b dropped and the prey scan narrowed further (2026-10-23),
  applies Directing 4:** "Drop D9b (Recommended)" and "Both narrowings
  (Recommended)". After D9a, skipping unchanged re-choices would save 0.6
  s of 93.6 s of scheduling, under 1%, for a revision every kind of change
  must bump; 813 stays as it is. A successful prey search, now about 11%,
  reads only the prey a liveness-naming selector could accept, the index
  filing living and dead bodies apart, and tests the requirements that do
  not read the prey once per search, those that do once per candidate; the
  same prey are accepted and the same draw made. This amends D9.

  Links: area Directing and play · draws on 683, 813, sim-execution D9 · amends sim-execution D9 · governs isocosm::targets · summary Skipping unchanged re-choices is dropped; a prey search files living and dead apart and tests prey-independent requirements once.

## Findings

- **2026-10-19, the tick as built** (read from `shared/isocosm/src`, nothing
  run): passes are process-major, ordered by due tick, priority and id; inside
  a pass, groups by first id; inside a group, members. Each act commits before
  the next. Gate, target and journey indices are rebuilt every advance and
  dropped; nothing carries "what changed" across advances. Non-actor requires
  (site conditions, mood, targets, computed) are evaluated and counted blocked.
  Every evaluation also re-checks every other choice process for its receipt
  (`execute.rs:182-196`) and hashes JSON for its event id (`execute.rs:99`).
- **2026-10-19, `Grouped` is exact compression, not a far tier.** It runs a
  bulk-safe cohort once with multiplicity, and falls back to individuals for
  shared writes, so ecological worlds gain nothing (the aggregation research's
  339,731 evaluations in both modes). The crowd lives only in the probe; no
  crowd-to-core or core-to-crowd transition exists. `TierLine` exists, unwired.
- **2026-10-19, what parallelism must preserve:** the act counter and child-id
  sequence, note and event append order, record marks, and the budget's early
  exit. Live reads inside a pass that break pass-start isolation: `known`,
  `Record`, commit-time `fits` and merges, nudge answers, spread direction,
  society checks, terrain revision, geometry readiness. All are commit-side or
  can move there.

- **2026-10-20, where a played tick goes** (run at `30e5d97d`; receipts and
  proposals in `Code/testing/isometry/sim-profile-2026-10-19/`). An ETW
  sampling trace of the played A1 battery, seed 0, release: hashing is not
  the cost (SHA-256, serde_json and `digest`/`draw` together 0.3% of process
  CPU; the event-id digest is 1.4% of `run_inner`). Inside `run_inner`,
  commit 35%, staging 28%, the `foregone` re-check 16%, queries 7%. The
  biggest single cost is copying bodies: `stage` deep-clones each actor's
  and target's `Entity` (its part map, each part's cells, tracts and losses)
  before any check runs, 15.6% of process CPU, and `Arc::make_mut` copies one
  again on write, 7.5%; about 10 of `String::clone`'s 12.5 points are inside
  these copies. The battery's two replays (75 s and 76 s) cost more than its
  played rounds (84 s); the plan's earlier 1.2 s figure was battery wall time
  over 180 ticks with the replays included. Under the `cargo test` profile
  the same seed takes 10,150 s, 41 times release. Dependencies at opt 2
  recover 18% of that and isocosm at opt 3 35%; debug assertions off, opt
  levels unchanged, brings it to 493 s, twice release. The debug-only
  whole-world checks (the journal's full `State` clone to verify each rollback,
  `journal.rs:66`; `passed_by`'s population rescan, `schedule/mod.rs:314`) are
  the likely source, not strictly separated from dependencies' assertions.

- **2026-10-20, two checks the decisions owe** (read, not run). D1: a key of
  actor, tick and process is unique only if one actor cannot run one process
  twice in a tick; a grouped cohort running with multiplicity may, so E2
  settles that first (an occurrence index in the key if so) and brings a
  fork back if it is real. D3: `TierLine::default()` is promote at 1 hop,
  demote at 2 (`shared/isocosm/src/directing/tier.rs`), and hops are whole
  numbers, so the band its doc comment calls memory is empty: a member
  stepping between one and two hops changes tier every tick, each change a
  crossing (E5). The comment's reason stands (a 3x3 enclosure has diameter
  two); the test named `the_band_between_promotion_and_demotion_is_memory`
  tests no band. To put to Mark before E5: keep 1 and 2, widen where worlds
  are larger, or a band in ticks.
- **2026-10-23, D5 and D6 measured** (release, the same tree before and
  after the change, the played A1 battery's `played-candidate` arm on one
  worker, the two binaries alternated on a machine other sessions were
  building on). Seeds 0 to 2 print identical lines on both and take
  1,049.7 s before and 804.5 s after, 23% less. Seed 0 alone, twice each:
  309.4 and 284.9 s before, 197.8 and 181.4 s after, a mean of 297 s to
  190 s, 36% less. Wall time includes the battery's two replays, which run
  the same acts; a figure for played rounds alone needs the scratch timing
  lines of 2026-10-20 and was not taken.
- **2026-10-23, where an accepted act goes** (scratch counters at
  `e764c9d0`, a throwaway worktree, nothing committed; the played A1
  battery's seed 0 in release, play and both replays, about 175 to 190 s
  of scheduling; each run printed the seed's usual line). Accepted acts
  are about half of it, 868,527 of them; `mind:strain` alone, 399,627
  acts adding a few points to one account, is a quarter. Inside an
  accepted act, effects are 52%, re-filing the gate index 20%, the write
  10%, queries 6%, the matter check 5%, and the stage 0.4% (D5). An extra
  copy of the actor's body, made and dropped aside, averaged 56 µs, more
  than half an accepted act: the copy on first write is what effects cost.
  Re-filing reads the body's books again for each gate it tests. Blocked
  evaluations, 4.9 million, are 11 to 14%; 70% of them are moves refused
  because the actor does not live where the route starts, which a gate on
  residency would pass by (about 9 s). Waking on an unchanged body would
  skip almost nothing: blocked again with the body untouched since its last
  block came to 0.3 s, because strain, upkeep and hazard write nearly every
  member every tick. Deliberation for members who then chose another
  process is about 10%, planning about 9%. Separately, where a world holds
  owned projects, `begin_process_debits` digests the whole state for every
  act; A1 holds none.
- **2026-10-23, D7a measured** (release, D5's tree and D7a's alternated,
  the same battery arm on one worker, a quieter machine than D5's runs).
  Seeds 0 to 2 print identical lines and take 403.2 s with D5 and 278.0 s
  with D7a, 31% less. Seed 0 twice each: 110.5 and 110.0 s, then 62.9 and
  62.1 s, 43% less.
- **2026-10-23, D7b measured** (as D7a, against D7a's tree). Seeds 0 to 2
  identical, 277.9 s before and 254.1 s after, 9% less; seed 0 twice
  each, 62.8 and 61.8 s, then 55.9 and 53.6 s, 12% less. Seed 0 under
  release with debug assertions on, so the filing check runs over the
  played world, reached its hour's limit without finishing and without an
  assertion firing.
- **2026-10-23, after D7** (the same scratch counters at `62f69614`, seed 0
  in release, play and both replays, a quiet machine; nothing committed).
  Scheduling takes 56.6 s. Accepted acts are 45% of it, re-filing the gate
  index after them 12% of the whole, effects 9%, queries 6%. Blocked
  evaluations are 15%, now 1.5 million of them (4.9 million before D7b);
  blocked again with the body untouched is 0.2 s, and what blocks now reads
  wounds and mood, which change nearly every tick, or site conditions,
  which seldom do. Deliberation for members who then chose another process
  is 15%, and the plans of shared-ground passes (`produce-3`, `feed-1`),
  which evaluate each member before acting it, 15%.
- **2026-10-23, deliberation and planning** (scratch counters at
  `ade7d189`, seed 0 as before, nothing committed; the machine was busy,
  so shares, not seconds). A choice was worked out 131,388 times; most of
  its cost is `feed-1`'s prey search, the only costly targeted choice:
  53,397 searches, each scanning about 573 candidate groups, about 90% of
  them failing `target_matches` before any requirement is read. Hunters
  search once to score the choice and again at the feeding pass (454). An
  empty choice was scored again at a later pass 100,710 times, 36% of them
  with no commit since. Planning's 900 passes evaluated 205,833 planned
  acts, each evaluated again to act; deliberation inside planning is the
  other half of planning's time.
- **2026-10-23, the test that refuses most prey** (scratch counters on
  `target_matches`, seed 0): of 30.6 million candidates the prey searches
  read, 24.7 million failed only colocation, 2.6 million liveness, 0.26
  million diet, and 3.0 million passed. The index already files groups by
  place; the colocation refusals are searches by travellers, whom the index
  files under their last site and whom colocation refuses everywhere.
- **2026-10-23, D9a measured** (D7b's tree and D9a's alternated, seed 0,
  three pairs, process CPU time, as other sessions' builds made wall time
  unreliable: D7b's seed 0 took 115 s against 55 s on a quiet machine).
  106.0, 107.1 and 98.5 s before, 97.0, 101.5 and 87.9 s after, 8% less,
  every line identical.
- **2026-10-23, deliberation after D9a** (the same counters, seed 0, a
  busy machine, 93.6 s of scheduling). An empty choice scored again costs
  17 µs, not 59; those with no commit since come to 0.6 s. A choice that
  finds prey costs about 560 µs, 18,561 of them, 10.4 s: the search reads
  the hunter's own site, liveness refusing 2.6 million candidates, and
  every requirement read for each of about 56 that pass.

## Progress

- **2026-10-19:** specced; Mark answered its four questions; nothing built.
- **2026-10-20:** a played tick profiled and timed (Findings); the baseline
  above replaced with the measured figures; Mark accepted staging bodies by
  `Arc` and `foregone` only on direct acts (Decisions), not yet numbered or
  built. Mark then held all six decisions until the decision-links plan
  settles how decisions are keyed and linked
  ([2026-10-20_decision_links_plan.md](../../design_docs/archive_docs/2026-10-22/2026-10-20_decision_links_plan.md)),
  then had them keyed D1 to D6 the same evening so D5 and D6 can be briefed.
- **2026-10-22:** the six decisions linked under the decision-links plan's
  L5. D1 and D4 refine Processes 7 (waking, parallel evaluation and
  order-free draws, not yet built); D2 is filed under Processes 6 and named
  in Mesocosm 5; D3 refines Directing 5 with the reach only, the band
  staying owed before E5; D5 and D6 are filed under Schema 9 and
  Directing 4 unchanged. `wing-decisions lookup sim-execution` lists them.
- **2026-10-23:** D5 and D6 built. The stage holds each bound body by the
  `Arc` it starts with and copies it on its first write; commit installs
  the staged handle where the live body is still the pass's base, merges
  as before where an earlier act changed it, and leaves alone a body the
  act never wrote. `Population::lift_shared` lifts a member without
  copying it. `run_inner` fills `foregone` for direct acts only. Exact:
  H3's pins 6/6 with nothing rerecorded, the played A1 battery's seeds 0
  to 2 identical, and isocosm's 264 lib tests pass in release and under
  the test profile. Battery wall time falls 23 to 36% (Findings). Whether
  `Part` also goes behind an `Arc` (D5) comes back now, with E1.
- **2026-10-23, later:** asked first to build E1 and trace after ("E1
  first, then trace"), a scratch probe of blocked work put E1's ceiling at
  14% of scheduling, and the round turned to probing accepted acts ("Probe
  accepted acts first"). That probe (Findings) made the body copy the
  largest cost; D7 orders parts by `Arc`, then the gate index, then E1.
- **2026-10-23, D7a built.** `Entity.parts` holds each part by `Arc`;
  writes go through `Entity::part_mut` and `parts_mut`, or `Arc::make_mut`
  where only some parts change, and merging a pass's changes skips a part
  the act never wrote. The state preimage reads the same bytes, so hashes
  and saves are unchanged. Exact: H3's pins 6/6 with nothing rerecorded,
  the battery's seeds 0 to 2 identical, isocosm's 717 tests in release and
  its 264 lib tests under the test profile, `mesocosm-phenotype`'s 25 and
  the VTT's native campaign 8. Battery time falls 31 to 43% against D5
  (Findings).
- **2026-10-23, D7b built.** Filing reads each of a group's holdings once
  (`Reading`), however many gates ask. A timed move's gates hold the sites
  whose routes reach its destinations, worked out per advance (routes
  change only when asserted), and a member is filed for it only where it
  resides there. A deliberative member is filed for a move it may choose
  wherever it lives: its choice for the tick is made at the first choice
  pass that visits it (683, 813), and passing it by would move that choice
  to a later pass and a different world. Exact: H3's pins 6/6, the battery's
  seeds 0 to 2 identical, isocosm's 717 tests in release and its 264 lib
  tests under the test profile, where the filing check (`passed_by`) runs.
  Battery time falls 9 to 12% against D7a (Findings).
- **2026-10-23, after D7:** with new figures (Findings), E1 came back as
  D7 said; the round chose deliberation and planning first (D8).
- **2026-10-23, deliberation and planning probed** (Findings); D9 and
  D10 chosen in one round: narrow the prey scan, skip unchanged
  re-choices, keep the plan's stage.
- **2026-10-23, D9a built.** A traveller's search for prey at its own place
  ends before it reads anyone (`choose_target`): colocation needs both
  bodies resident, and the index files a traveller under its last site.
  The same prey are found and the same draws made. Exact: H3's pins 6/6,
  the battery's seeds 0 to 2 identical, isocosm's 717 tests in release and
  its 264 lib tests under the test profile, where the filed search is
  still compared with a full scan. 8% less CPU time (Findings).
- **2026-10-23, after D9a:** a probe of deliberation (Findings) put D9b at
  under 1%; the round dropped it and chose to narrow the prey scan
  further (D11).
