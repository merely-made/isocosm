# Wing design record: readings docket

**Docket for Mark, 2026-09-21.** Every reading, recommendation and verdict
the [wing design record](2026-09-18_wing_design_plan.md) flags as its own and
not yet ruled, gathered so they can be ruled in one pass, the way W1 was.
Each item says where it sits, what it claims, what follows if accepted, and
this record's suggestion: **accept**, **amend**, **park** (leave open, no
harm in waiting) or **hold** (belongs to a later phase).

Nothing here is ruled. A ruled item moves into the record's §0 in Mark's
words and leaves this docket; when the docket is empty it is archived.

Why it exists: the record gained 1,275 lines between rulings 62 and 78, and
its own readings were piling up faster than they could be ruled, some of
them carrying later ones (review of 2026-09-21, §5.2).

## Factions and polities (§3.2.2)

| # | Reading | If accepted | Suggest |
| --- | --- | --- | --- |
| D1 | The "few general protocols" for when peer to peer agreement fails belong to the world's ruleset, not to the faction (ruling 63) | a faction stays stateless and free to the scheduler; rejected, factions carry state | accept |
| D2 | A form of governance is a pair: who qualifies to decide, a predicate over standing, possessions, body, lineage and provenance; and the act by which deciding is done | forms are authored and generated data, never an enum | accept |
| D3 | Ranked preference is the sim's mechanism for which form emerges when a polity is founded, weighted by influence, counted over cohort blocs, ties broken on the seed; not necessarily an election anyone in the world holds | one count at a rung transition, its winner asserted as the constitution | accept |
| D4 | A polity enforces with what it provides and what it asserts: provision withheld, rank, property, membership, home and borders revoked; force is one more means | enforcement needs no new machinery, only processes with a cost | accept |
| D5 | Whether a means is efficacious is ruling 50's trust, a track record read from the record | efficacy is never asserted; the gap between a constitution and its members' preference is readable | accept |
| D6 | Ruling 66's four conditions are each read off the record: inactive (no support), suppressed (another's enforcement stops it, members go underground as a faction), superseded (another polity binds its focus), subordinated (a faction fills its deciding party or holds its means) | conditions overlap and reverse while the polity lives | accept |
| D7 | An institution seen clinging on is one subordinated to the faction of its own officers | no built-in will to continue, per ruling 68, and the appearance is still explained | accept |
| D8 | A polity has two alignments, the one it professes in its constitution and the one it practises, derived from its acts; change is the practised one moving first and reform catching up | two readable gaps per polity; the most speculative item here, and more bookkeeping | park |
| D9 | What makes a host an institution is that agreements under it hold without their parties enforcing them; contingency is borrowed enforcement, cheap to found, and a host's failure cascades | settled worlds grow institutions and wild ones do not | accept |

## Keeping (§3.4.1)

| # | Reading | If accepted | Suggest |
| --- | --- | --- | --- |
| D10 | "Of note" is the derivation rule with a name: a thing is of note exactly when regenerating it would lose something | no separate notability score for the middle tier | amend: "carries a deviation that has not been folded back", since a grazed meadow deviates and is folded at once |
| D11 | Collection is roots and reachability; collecting folds a thing back into its distribution, conserving counts and losing only deviations; the safety test is that nothing reachable from a player could tell it from its regenerated self | the invalidation criteria Mark called crucial get a shape, not yet their values | accept |
| D12 | A thing of note sheds in steps: the individual, then a stub (kind, lineage, faction, name), then the event alone; events name participants by stub | a participant can be collected without breaking the event that mentions it | accept |
| D13 | The of-note tier is one module of the sim, not a crate per word (recommendation) | `denizen` stays a reserved name, not a crate to build | accept |

## Places (§3.7.1)

| # | Reading | If accepted | Suggest |
| --- | --- | --- | --- |
| D14 | A location's kind is read, not stored, as a biome is: stored are its extent, its modifiers and conditions, what happened there and who claims it | Paredros's five-kind enum gives way to open data; a "potential" location is a derived candidate | accept |
| D15 | One nesting step declares four things: down (how the finer map is generated), up (how it reads from above), across (how adjacency crosses), ratio (how much finer); the last step meets the volume at a size the world sets | steps compose and can be added at either end or in the middle | accept |
| D16 | For a world resting on a critter, what the critter does reaches the world as its environment (written into §3.7.1 unflagged) | the terrain-body is an agentless process source for the world on it | accept |

## Processes (§3.3)

| # | Reading | If accepted | Suggest |
| --- | --- | --- | --- |
| D17 | The two transitions are lifting (aggregate to individuals, sampled from the seed conditioned on asserted facts) and restriction (individuals to aggregate, accounts conserved); similitude is two bench checks on seeded draws; so a process must be declarative enough to derive its aggregate form, and scripted hooks run only in the foreground | ruling 32's schema carries ruling 75; piccolo authors and lowers, it does not run per entity in the background | accept |

## The boundary (§5.2 to §5.4), held under review

| # | Reading | If accepted | Suggest |
| --- | --- | --- | --- |
| D18 | What survives the review cheaply: ruling 78 as a discipline; two bindings as an intent, the component binding built when the first rung 3 consumer exists; a byte round-trip test and a dependency-direction rule in place of a second runtime; the probe and `libm` as notes | no Wasmtime, no CI duty, nothing owed before W5 | accept |
| D19 | §5.2's contract shape: intents in, events and views out, one attention set per player, a resolution handoff | it is the overlay contract's first draft | hold for W5 |
| D20 | Who may direct which entity as a provider over `mere-capability`'s order | one authority algebra across the stack | hold for W3, beside ruling 34 |

## Open questions parked in the record, not on this docket

These are questions, not claims, and wait on nothing: which fallback
protocols (§3.2.2); whether the deciding act constrains the means or only
defaults it; charters as bearers, who must agree to a polity's death, reform
and secession; further polity conditions and the limits of a subordinating
faction; whether an event of note lapses and what "little incident"
measures; whether collection must be deterministic across peers; the anchor
of a location when the ground moves; the size of a site against the paged
chunk; the thing and event words of the of-note naming round.
