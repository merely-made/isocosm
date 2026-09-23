# Aggregation research and the executable boundary

**Status, 2026-09-22:** research applied to the sim implementation at Mark's
request. Primary sources below were checked during this session. The first
implemented reduction is exact grouping, with an individual reference mode
that executes the same process definitions. This does not select an
approximation scheme for the whole sim.

## What the literature establishes

[Ganguly, Petrov and Koeppl](https://arxiv.org/html/1303.4532) distinguish a
partition of states from one that preserves dynamics. Strong lumpability
requires equal outgoing probability totals to each target class for every
state within a source class. Their weaker construction also specifies how
the distribution inside an aggregate depends on the initial distribution.
Counts and material totals alone do not identify a heterogeneous group's
future behavior.

[Feret et al.](https://arxiv.org/abs/1011.0496) derive reductions from rule
dependencies. The useful implication here is to make process read/write
restrictions executable. A process reading an individual relationship cannot
silently run on a cohort that omitted that relationship.

[Rathinam, Petzold, Cao and Gillespie](https://epubs.siam.org/doi/10.1137/040603206)
establish first-order consistency for tau-leaping and particular convergence
results for linear propensities. Tau-leaping has conditions and approximation
error. It does not establish identical statistics for arbitrary foreground
and background models.

[Kevrekidis et al.](https://arxiv.org/abs/physics/0209043) supply the
lifting/restriction framework already named by the design record. Its
applicability assumes a suitable macroscopic description. Returning the
original aggregate immediately after lifting is a consistency check; it
does not establish agreement of subsequent evolution.

## Current code and the implementation decision

Mesocosm's existing `cohort.rs` stores count, biomass, energy and summed age,
then divides those evenly when splitting. Individual authority is retained
elsewhere, so this can serve as a reading. It cannot stand alone for an
arbitrary threshold process. Two populations may have equal total energy
but different numbers of starving individuals. The new tests include that
counterexample.

`shared/isocosm/src/population.rs` stores complete member state with an
identity interval and multiplicity. Different reserves, traits, body
revisions, places, skills, dispositions or tenets stay in different rows.
Lifting splits an interval without changing its state. Restriction combines
only adjacent identical rows, protecting observed identities. Compression
may deteriorate as a world diversifies; the implementation does not promise
that every world compresses well.

`Process::bulk_safe` admits a deliberately strict subset: local requirements,
immutable place conditions, actor-local transforms, trait changes and
practice. Shared writes, targets, identity-dependent risk, offspring and
public events run individually. Both modes call the same transactional
interpreter and preserve process/identity ordering. Bulk execution multiplies
independent identical transitions; it does not estimate a rate.

This is an engineering inference from the reduction criteria, not a claim
that a chemical CTMC theorem directly proves a discrete game simulator.
The implemented argument is that these local operations commute across
identical members because they cannot read or write each other and their
predicates read identical state.

## What the bench checks

The first generator domain varies directed site routes, lineage count,
population, reservoir-network wiring, integer transfer quantities, initial
stocks and process periods. Parameters and realized rules accompany each
seed. The comparison checks the entire normalized world after each tick,
including changing observation, rather than only an end-of-run mean.
Saved histories replay through the other execution mode. Adversarial checks
cover depleted stock, failure after an earlier effect, disconnected routes,
and the averaging counterexample.

A second declared generator family uses well-mixed sites with shared soil,
producers, consumers, decomposers, paid reproduction, starvation and age
death. Feeding and shared-resource writes use the individual fallback. This
checks coupled execution and its bookkeeping without claiming a reduction
of competitive interactions. Roles are registered even for lineages that
the initial population did not draw; a broader run exposed that admission
case after the first small regressions had passed.

This domain tests interpreter equivalence, not ecological viability or
arbitrary social/spatial aggregation. Its receipt says so. Evaluation counts
measure work eliminated; elapsed time, memory and native frame cost are
separate measurements. Additional generator families must be qualified in
their own terms instead of silently broadening this receipt.

The first 16-world draw receipt uses master seed **1790133116439677600**,
selected from the system clock before the run. Eight worlds from each family
ran for 64 ticks: 1,024 full-state comparisons, 16 saved replays through the
other execution mode, and 16 inert-refusal checks passed. Independent worlds
used 23,470 grouped evaluations against 139,162 individual evaluations, a
5.93-fold reduction in calls. Ecological worlds used 339,731 evaluations in
both modes because their shared-resource processes correctly fell back to
individual execution. That result provides no ecological aggregation speedup.
These counts are work receipts, not wall-clock benchmarks.

## Next reductions to investigate

Histogram keys can eventually omit fields proven irrelevant to all admitted
future processes. That requires analyzing the ruleset, not assuming the
current action's read set is sufficient forever. A rules revision must
revalidate that reduction before any omitted field becomes causal.

Shared-resource reactions need explicit competition semantics. Sequential
deterministic allocation and stochastic well-mixed interaction describe
different worlds. Changing between them is a rules change, not a harmless
optimization. Spatial reductions also need mixing or contact information:
being in the same site does not prove equal access to its resources.

Tau-leaping needs a propensity model, adaptive step criterion, nonnegativity
handling, critical-event fallback and measured error envelopes. Extinction,
threshold crossings and covariance matter alongside means. It should be
introduced against a specified process family with an exact reference.

Social reductions must preserve relevant correlations among needs, beliefs,
influence and relationships. A mean alignment can erase opposed blocs.
Equation-free methods become useful only after identifying a viable coarse
state and testing whether reconstructed microstates relax consistently.

These are research obligations for broader reductions. They do not prevent
the exact runner, reversible identity grouping, or ordinary sim work.
