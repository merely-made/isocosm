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

`isocosm-probe` runs ruling 113's check on ruling 115's competing instance,
feeding when food is short, over drawn worlds of the `probe` module's domain.
Each world runs four ways: twice through the exact individual runner, once as
a crowd, and once as a crowd that averages reserves. Readings are derived from
the definitions; each gets a difference test and an equivalence test against
its bound, Holm-corrected. `--density` runs only the exact and crowd arms, to
measure savings without a verdict; `--members LO HI` overrides the domain's
members per site and lineage.

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
  processes and transitions. Configured operation, history and population
  limits refuse work atomically. They do not discard history to fit a budget.
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
- The `probe` module runs ruling 115's competition, pairwise contest, share
  and yield, both member by member through the interpreter and as a crowd:
  counts per exact state, advanced by integer count draws that follow the
  member-by-member round's distribution. Both read queries and apply effects
  through the interpreter's own meanings, written once. A competition's
  definition and the per-reading similitude bounds are part of the world's
  rules and its rules digest; worlds without them serialize and hash as
  before. The rounds that resolve a competition still run in the probe, not
  the core's scheduler.

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

Transaction staging copies mutable state. Grouping reduces interpreter calls
for independent processes, but gives no general time or memory bound as a
world diversifies. The [aggregation research](../../mesocosm/design_docs/2026-09-22_aggregation_research.md)
separates exact equivalence from possible approximate reductions.

Measured on 2026-09-25, in the receipts under
`mesocosm/testing/bench/receipts/2026-09-25/isocosm/`: time per tick grows
about with the square of the population at the largest sizes run, because
every evaluation first totals matter over the whole stored world and every
accepted one clones the whole simulation, reach field included. At eight
lineages about 98% of ecology evaluations end without effect, since every
periodic process is evaluated for every member. The dead stay stored and each
noted event keeps an arrival at every site it reached, so ticks grow dearer as
history accumulates. Founding in cohorts of 32 puts each cohort on one site,
and those ecology worlds died out without a birth.

The probe's certified receipt, `probe.json` in the same directory, drew 1,000
worlds. In each, the crowd stayed within 0.2 Kolmogorov-Smirnov distance of the
exact individual runner on all 16 readings the definitions yield, and no
difference was detected beyond chance. The exact runner against itself passed
the same test, and a crowd that averages reserves failed the starvation
readings. The crowd used 11 times fewer evaluations there; across the density
ladder, 6 to 40 times fewer as members per site and lineage rose from 32 to
512. Its count draws are exact in distribution, so the receipt certifies this
reduction's implementation and the instrument, not an approximation's error,
and only for this one process.
