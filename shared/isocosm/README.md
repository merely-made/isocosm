# Isocosm

The product-independent simulation core, consumed by Mesocosm's native
specimen bench. Local version 0.1.0 follows the 0.0.1 name reservation; this
implementation has not been published.

Run from the repository root:

```powershell
cargo test --manifest-path shared/isocosm/Cargo.toml
cargo run --manifest-path shared/isocosm/Cargo.toml --bin isocosm-bench -- --ecology --population 48 --sites 3 --lineages 3 --ticks 64 --output world.json
cargo run --manifest-path shared/isocosm/Cargo.toml --bin isocosm-bench -- --draws 16 --ticks 64 --output draws.json
cargo run --release --manifest-path shared/isocosm/Cargo.toml --bin isocosm-scale -- --output scale.json
cargo run --release --manifest-path shared/isocosm/Cargo.toml --bin isocosm-probe -- --draws 1000 --output probe.json
```

`isocosm-scale` times drawn worlds by size through the host API, one tick at a
time, and records evaluations, stored groups, history growth and heap for each.
It counts heap with an allocator wrapper local to that binary; the library has
no `unsafe`. Its seeds come from one master seed, printed and saved like the
bench's. Run it in release: debug timings say nothing about scale.
`--remeasure RECEIPT` (repeatable, with `--family` to narrow) runs an earlier
receipt's points again on the same draws and reports, point by point, the
speedup and whether every deterministic field came out as before.

`isocosm-probe` runs ruling 113's check on ruling 115's competing instance,
feeding when food is short, over drawn worlds of the `probe` module's domain.
Each world runs four ways: twice through the exact individual runner, once as
a crowd, and once as a crowd that averages reserves. Readings are derived from
the definitions; each gets a difference test and an equivalence test against
its bound, Holm-corrected. `--density` runs only the exact and crowd arms, to
measure savings without a verdict; `--members LO HI` overrides the domain's
members per site and lineage; `--water` has every world contest water as well
as food. `--approximate` adds a fifth arm, a crowd whose round takes each
count, the segments, the pairing and the settling alike, in one step near its
mean and variance instead of member by member (ruling 220), checked against
the exact runner and against the exact crowd. `--crowds` runs those two
crowds alone, to time them at densities the exact runner is too slow to
reach.

Omit `--seed` for an unselected seed, printed before a draw run and saved in
its receipt. Use `--load world.json --ticks 0 --individuals` to verify a saved
world through individual execution. `--world genesis.json` admits authored
founding facts and rules using the same validator and interpreter. The
`genesis` object inside a save is such a document. A generated world's
founding parameters accompany its realized rules and topology.

## Current behavior

- Integer account ledgers, declared conditions and requirements, and typed
  effects form a serializable ruleset with a content digest. Admission checks
  its references and material balance. A rejected transition changes nothing,
  including identity grouping and earlier effects within that transition.
- A stable clock and ordered due-process queue drive choices, agentless
  processes and transitions. A tick counts a unit of world time the world's
  rules may state, a minute by default, and process periods are read in it;
  anything finer is the foreground game's (rulings 256 and 257). A due
  process visits only the stored groups carrying the traits it requires of
  its actor, filed as acts commit, and still sees groups as they stood when
  its pass began (ruling 258). Configured operation, history and population
  limits refuse work atomically; the operation budget counts only the
  evaluations that run (ruling 259). They do not discard history to fit a
  budget.
- Populations retain complete per-member states as counted identity intervals.
  Only statically independent unary effects can run once for an interval.
  Shared resources, target selection, risk, parentage and public events use
  individual execution in identity order. Inspection lifts an identity;
  restriction merges only exactly equal states after the collection buffer.
- The reservoir generator varies independent metabolic networks. The ecology
  generator adds shared soil, producers, consumers, decomposers, paid births,
  starvation and age death. Sites are explicitly well-mixed compartments.
  These are testable law families, not the full authored ecology.
- Notes use the Impresa causal core with a Djot/open-data envelope. Events
  spread along directed routes. Deterministic knowing integrates past
  residence and decaying reach; learning fixes a note. Hagiograph judges
  measured account records. A beaten standing mark promotes its event to
  legend, retained with a global reach floor.
- Sessions save founding facts, accepted and refused intents, outcome receipts
  and epoch hashes. Loading replays and verifies them, including across
  execution modes. Branches retain their source. Merge returns a reviewable
  proposal and reports changed accepted outcomes as well as refusals.
- A world may carry a dynamics seed apart from its founding seed, so one
  founded world can run under independent draws. Absent, it is the world seed,
  and such worlds serialize and hash as before it existed.
- A world may carry a mind (rulings 159, 164 and 227). Mood is read from the
  needs its rules declare, each a query on the member and a weight, and is
  never kept; strain is kept in a strain account. The rules' own processes
  build strain while mood reads low and ease it off while it does not. A mind
  bears strain up to a bearing set by its traits. Worlds without a mind
  serialize and hash as before.
- The `probe` module runs ruling 115's competition, pairwise contest, share
  and yield, both member by member through the interpreter and as a crowd:
  counts per exact state, advanced by integer count draws that follow the
  member-by-member round's distribution. Two contesters who size each other
  up as a close match fight in rounds (rulings 221 to 223). The side standing
  lower loses each exchange unless an upset turns it; both take the round's
  strain, and the loser spends reserve, which strains it more. A side past
  its bearing breaks once, up or down by its traits, its mood and a draw,
  and its standing shifts for the rest of the fight. The fight ends when a
  side is spent or, sized up again, outmatched. Both runners fight through
  one function and apply effects through the interpreter's own meanings.
  A world's competitions are keyed by the site account each contests
  (ruling 236) and run at once in a tick (ruling 240): each resolves against
  its members' state at the tick's start, fights on copies, and at the
  tick's end each member's rounds, reserve spent (capped at what it holds)
  and winnings settle in one order. `isocosm-probe --water` has every world
  contest water as well as food. The definitions and the per-reading
  similitude bounds are part of the world's rules and its rules digest. The
  rounds still run in the probe, not the core's scheduler.

`Founding -> Genesis -> Session` is the host API. Hosts send `Command`s and
advance the clock explicitly. Views read `Simulation::state`; drawing does
not inspect an entity or advance time. The core has no renderer dependency.

## Current limits

This is the first executable foundation of the
[sim plan](../../mesocosm/design_docs/2026-09-22_sim_plan.md), not completion of
S1-S6. The schema includes several future authoring seams whose full behavior
is not implemented: goals and deliberation, ranked polity consent, genotype
epochs and realignment, world crossing, magic, body-volume mechanics, and
the complete thirty-process vocabulary. Periodic processes use authored
cadences; they are not a general planner.

Collection is conservative exact compression. It retains deviations and
history; graded stubs, event pruning and reconstruction from a sparse log
remain open. Epoch checkpoints are verification hashes, not fast state
snapshots. There is no parallel sharding or network authority implementation.

Reach currently uses one earliest-arrival route per place, with strongest
arrival and lowest predecessor breaking ties. Later independent carriers,
relevance weighting and secret/taboo willingness remain open. Exposure uses
capped integer strength summed over residence ticks; it is an explicit first
law, not a calibrated social model. The backward arrival path is inspectable;
it does not yet materialize a carrier or teller.

An act stages copies of only what it binds, its bodies and its site, and
lists what it adds; the world is written once the act is accepted. Receipts
read the world's matter from a total summed at founding, which every accepted
act conserves. During an advance, a scheduled act with a target looks for it
only among the stored groups filed at the place and under the lineage its
selector could match, an index kept current as acts commit; and the advance
journals the first state of whatever it changes, so a refused advance is
undone from the journal instead of from a copy of the world taken before it
began (ruling 237). Debug builds check the first against the full search and
the second against a full copy. Grouping reduces interpreter calls for
independent processes, but gives no general time or memory bound as a world
diversifies. The
[aggregation research](../../mesocosm/design_docs/2026-09-22_aggregation_research.md)
separates exact equivalence from possible approximate reductions.

The first baseline, measured on 2026-09-25 in the receipts under
`mesocosm/testing/bench/receipts/2026-09-25/isocosm/`, found time per tick
growing about with the square of the population at the largest sizes run,
because every evaluation first totalled matter over the whole stored world and
every accepted one cloned the whole simulation, reach field included. With
both costs cut, the baseline's 30 ecology points ran again on the same draws
(`remeasure-ecology.json`) with every hash and count unchanged, 12 to 113
times faster per tick, 40 times over all. Time per evaluation still rose
slowly with size then, from about 3 to 4 microseconds grouped and 4 to 8
individually as members doubled from 2,048 to 4,096. Ruling 237 cut the two
costs left, the target search across the whole population and the copy of the
world before each advance. The same points ran again on 2026-09-26
(`remeasure-237.json` in that day's receipts) with every hash and count
unchanged, 2.2 times faster over all than at checkpoint 1 and 4 times at 4,096
members individually, and took about 1.1 to 1.9 microseconds per evaluation
at 2,048 and 4,096 members in both modes. Every due process was still
evaluated for every living group, so evaluations grew with lineages, and at
eight lineages about 98% of ecology evaluations ended without effect. Ruling
258 has a due process visit only the groups carrying the traits it requires.
On the lineage sweep at 1,024 members (`scheduler-lineages.json` in the same
day's receipts), ecology evaluations per tick fell from 219,408 to 7,457 at 32
lineages and ticks from 196 to 22 ms, with every hash unchanged. What still
grows with lineages is the ecology generator's one feeding process per pair of
lineages. Of the evaluations that remain, 84 to 96% still end without effect:
a lineage's own processes, death and age among them, run for each of its
members on their periods, and their other requirements are mostly unmet. The
dead stay stored and each noted event keeps an arrival at every site it
reached, so ticks grow dearer as history accumulates.
Founding in cohorts of 32 puts each cohort on one site, and those ecology
worlds died out without a birth.

The probe's first certified receipt, `probe.json` of 2026-09-25, drew 1,000
worlds. In each, the crowd stayed within 0.2 Kolmogorov-Smirnov distance of the
exact individual runner on all 16 readings the definitions yield, and no
difference was detected beyond chance. The exact runner against itself passed
the same test, and a crowd that averages reserves failed the starvation
readings. The crowd used 11 times fewer evaluations there; across the density
ladder, 6 to 40 times fewer as members per site and lineage rose from 32 to
512. Its count draws are exact in distribution, so the receipt certifies this
reduction's implementation and the instrument, not an approximation's error,
and only for this one process.

Under the fight in rounds and the mind (rulings 221 to 223 and 227), the check
ran again on 2026-09-26, in `mesocosm/testing/bench/receipts/2026-09-26/isocosm/`.
With food contested (`probe.json`), all 30 readings were certified within
their bounds, the largest distance 0.052 on an inspected member's strain, and
no difference was detected. With water contested as well (`probe-water.json`,
two competitions a tick), all 40 were, the largest 0.032. In both, the exact
runner passed against itself and the averaged crowd failed the starvation
readings. The crowd used 6.5 and 2.4 times fewer evaluations and ran 12.5 and
6.0 times faster; with two competitions a tick, members spread over more
states, a median of 2.1 alive members per state at the end against 9.8.

`probe-approximate.json` qualifies ruling 220's approximate pairing draw at
512 to 1,024 members per site and lineage, eight times the probe's default,
where the median state held 56 alive members at the end. Against the exact
runner all 30 readings were certified, the largest distance 0.036 beside the
exact crowd's 0.041 and the exact runner's own 0.022, and none differed after
correction; against the exact crowd, all 30 again, the largest 0.058. The
largest distances fell on single inspected members. It saved nothing there:
the exact crowd already used 23 times fewer evaluations than the exact runner
and ran 36 times faster, and the approximate crowd took the same evaluations
and the same time, each within 1%. Timed alone (`probe-crowds-*.json`, four
draws each), it ran 1.02, 1.10 and 1.33 times faster than the exact crowd
from 512, 2,048 and 8,192 members per site and lineage, so at these densities
the count draws are a small part of the crowd's time. The receipt qualifies
the draw only at the density it ran at.
