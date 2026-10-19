# The sim's execution model: waking, parallel evaluation, near and far

**Date:** 2026-10-19

**Status, 2026-10-19:** plan, nothing built. Specced at Mark's request after a
probe was stopped for blurring design, implementation and qualification. The
three are kept apart below. Mark answered the four open questions the same
day (Decisions, below); E1 is next when he opens it.

## Why

Judging mechanics in combination means watching generations across several
seeds, so the sim's speed bounds the design loop itself. A played tick costs
about 0.45 s in the head's seed-9 game (about a hundred members) and about
1.2 s in a played A1 world grown to tick 180 (a few hundred), against the
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

## Decisions, 2026-10-19

Mark's answers, unnumbered until he numbers them:

- **Draw keys:** "Actor, tick and process (Recommended)". Amount, bite, risk
  and soma draws are keyed by actor, tick and process, so evaluation is
  order-free; one deliberate hash move, outcomes the same in distribution.
  E2 is in.
- **Exactness:** "Statistical (Recommended)". Mesocosm's far sites run as the
  crowd; the played critter and every noted member always run as
  individuals.
- **Near:** "Roots plus one hop (Recommended)". Attention roots and everything
  within one hop of the played critter's site; promote within one hop, demote
  at two or more.
- **Order:** "Speed first (Recommended)". E1, E2, E3, then E4 and E5.

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

## Progress

- **2026-10-19:** specced; Mark answered its four questions; nothing built.
