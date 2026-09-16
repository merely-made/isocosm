# Memory, remembrance, and hagiograph

**Status (2026-09-09): plan.** This is a Paredros implementation plan for
ordinary individual memory and the later memorial boundary. It is intentionally
separate from the world-conditions plan and from the functional-loops plan.
Those plans own world triggers and larger orchestration. This document owns
the memory model, its evidence links, bounded recall, and promotion records.
The adjacent contracts are [world conditions](2026-09-09_world_conditions_plan.md)
and [functional loops](2026-09-09_functional_loops_plan.md).

## Scope and design position

Paredros needs a memory of particular people and events that can affect an
individual's next answer. A memory is more than a recent observation bit: it
can be retold, revised, believed, preferred, or rejected. The first slice is
ordinary episodic memory and beliefs. Preferences and ideology are later
derived reactions to repeated experience and world change. Hagiograph is the
separate memorial view that promotes selected history into a retellable public
memory and, when world conditions admit it, a procedural manifestation.

Existing facts verified on 2026-09-09:

- `paredros-social::DeedLog` is append-only and owns social deeds. A deed has
  `DeedId`, tick, doer, optional recipient, and a small `DeedKind` vocabulary.
  `Relations` folds standing from those deeds; it does not store a second
  authored standing value.
- `paredros-social::EpistemicLog` is append-only and observer-scoped. It
  admits `Observation`, `Claim`, `Report`, and claimant-owned `Correction`,
  checks event ordering and evidence ownership, and replays exact IDs and
  records. `Belief` is currently a derived latest revision with support IDs.
- `Society::consider`, `form`, `exercise`, `propose_change`, and `end` return
  `Response` or `Ruling` values whose ordered `Premise` entries point to deeds
  and thresholds. `say` and `explain` are the current legibility surface.
- `paredros-world::GameState` owns the subject-addressed transition log and
  emits `GameEvent` values including `Observed`, `Moved`, `Took`, `Ate`,
  `Injured`, `Waited`, and `Died`. `Simulation` advances every living subject,
  records `Decision` values, and can report each life, but it currently has no
  epistemic or memory state.
- `SimulationSave` and `GameSave` regrow deterministic state and replay
  intents. Adding memory requires an explicit versioned save field and replay
  validation rather than an untracked side cache.
- `mere/crates/eidetic/hagiograph` is a name reservation. Its README and
  library docs define a view over ordinary history containing retold legends,
  memorials, epithets, and manifestations. It has no implementation.

Proposed by this plan:

- Ordinary memory is owned by the individual epistemic subsystem in
  `paredros-social`; durable history remains owned by the deed/event journals
  that actually record what happened.
- A bounded in-memory cache is disposable. Gameplay recall and forgetting are
  derived from saved policy and accepted attention/forgetting history; their
  effects on later decisions are reproducible. Neither layer rewrites source
  events, claim history, corrections, or promotion records.
- A belief, preference, or ideology reaction carries a pointable support graph
  and deterministic revision, explaining changes after new evidence or events.
- Hagiograph stores promoted retellings and their provenance as a view. A
  retelling can gain or lose attention, but its source references and
  promotion/correction history remain inspectable.
- A hagiograph manifestation requires a promotion plus world conditions.
  Retelling cannot directly edit terrain or mint a procedural fact.

## Ownership and boundaries

`paredros-world` remains the authority for accepted world transitions, actor
identity/body state, simulation rounds, and objective event observations. It
should expose an event stream or stable projection for a memory adapter; it
should not decide what an individual cares about. `paredros-social` owns
observer perspective, support validation, belief folds, memory policy, and
answer premises.

The memory adapter may consume `GameEvent`, `Decision`, and social `Deed`
records, but must not infer an event the authoritative owner did not accept.
An affected subject gets an event observation only when policy permits it. The
observation records observer, event reference, tick, and provenance; it does
not copy mutable world state as a second truth.

`hagiograph` belongs in Mere's Eidetic family as the reusable memorial view.
Paredros supplies the subject/world policy, promotion candidates, and
world-condition admission. `muniment::Journal` or the relevant Paredros
history owner remains the source of objective history. `eidetic::Codicil`
remains an immutable exchange record. `fili` remains descent/lineage. None of
these boundaries should be collapsed into a universal memory store.

## Stage F3b1: observer evidence into consequential answers

This is the first bounded wiring slice. It connects accepted world/social
events to an individual's consequential answer path, with affected-agent
observations and explainable support. It has no scene prerequisite. The slice
must join three decision inputs together: observer-relative standing, explicit
versioned norms, and the individual's supported beliefs. Objective deed
premises alone must not decide the answer with belief text appended as
cosmetics.

The adapter consumes a deterministic batch of accepted events after each world
transition or simulation round. `EpistemicLog::observe` currently accepts a
`DeedId`, not a `GameEvent`; the wiring therefore needs a deduplicated,
validated `GameEvent` to `DeedId` mapping owned at the adapter boundary, or an
explicit generic-source extension with equivalent provenance checks. It must
never pretend that a `GameEvent` is already an epistemic entry. For each
affected subject, policy admits the mapped source and creates an observation.

The answer fold derives that subject's standing by evaluating supported beliefs
under the applicable norm-set revision, then evaluates the request with current
needs and dispositions. A response records these dependencies in premises, with support IDs and source
references. Earlier answers retain the exact belief and norm revisions used at
their tick. A correction received by an absent recipient changes that
recipient's future answer only after a valid report; it cannot retroactively
rewrite a previous response.

The fixture uses two named subjects and one consequential offer. One directly
observes a deed affecting them; the other does not receive that observation.
Only the informed answer changes, with `Society::explain` identifying support;
a later report demonstrates addressed transmission without global knowledge.

Done-conditions:

- Every admitted observer entry points to an existing accepted event and never
  precedes that event's tick.
- An affected subject can receive an observation through the explicit policy;
  an unrelated or excluded subject cannot.
- The consequential `Response`/`Ruling` carries support that resolves to real
  entries, and its rendered explanation names the relevant event or belief
  revision without inventing certainty.
- Replaying the same event batch, epistemic entries, and answer sequence gives
  identical IDs, beliefs, premises, and state hash.
- Norm and standing revisions participate in the answer decision, are
  observer-relative, and are pointable in the explanation beside belief
  support. A correction changes only future answers for recipients who receive
  it, while preserving prior answers, the original claim, supports, and report
  history.

Focused tests cover affected-agent fan-out, excluded observers, the
GameEvent-to-Deed mapping, observation-before-event rejection, norm and
standing divergence, report ownership, absent-recipient correction, answer
premise integrity, and save/restore replay. The receipt includes exact event,
deed, norm, belief, and support IDs, not only equal final booleans.

## Stage F3b2: individual episodic memory and bounded recall

Add a typed individual memory record above the existing epistemic entries. An
episode should identify its holder, event or claim references, participants,
location/reference data where available, first and last access ticks, salience
inputs, and a stable derivation/version. Keep the event reference compact and
pointable. Do not duplicate full `GameEvent` payloads in every memory.

Define separate policies for admission, recall, retelling, and semantic
forgetting. Distinguish semantic forgetting from RAM/cache eviction: cache
eviction changes residency and must not change a decision; semantic forgetting
may change deterministic gameplay recall and therefore is a recorded,
versioned state transition. If history-dependent, emit explicit `Recall` and
`Forget` facts, save the policy/configuration revision and attention state, and
replay them.
Admission is deterministic and bounded per subject and round. Hot recall has a
fixed item/byte budget and deterministic tie-breaking. Define global and
per-agent item, byte, support-edge, and batch limits, plus a disk/archive quota
and deterministic storage-pressure policy. Cold archival does not bound disk
forever: the policy must state whether it rejects new entries, compacts derived
metadata, or requests user-directed pruning. Pins, correction evidence, and
succession references cannot be erased. If retained evidence is unavailable,
the answer reports that gap rather than inventing certainty.
The first implementation retains exact durable history with a bounded resident
cache and segmented archives. On exhausting its configured disk quota, it must
pause durable admission with a visible storage-pressure result, not quietly
forget accepted facts. A later opt-in lossy retention mode needs checkpoints,
an explicit replay floor and missing-detail markers; it must preserve pinned
correction/evidence/succession references. A digest of deleted evidence is an
identity check, not enough material to reconstruct its contents.

Use a global cache budget with per-subject shares and bounded temporary loans
for active queries. Dynamic RAM allocation may evict and reload cache entries;
it cannot alter saved gameplay recall limits or semantic salience. Pin requests
reserve bytes and support edges before admission, so enough pins cannot silently
defeat the bound. If evidence loading must wait, the decision waits or returns
pending; missing cache data is never treated as proof that an event was forgotten.

Done-conditions:

- The same admitted event stream yields the same episodic entries and recall
  order for every subject.
- Hot recall stays within configured global and per-agent item, byte, support,
  and batch budgets, including worst-case serialized overhead; archive storage
  stays within its explicit quota and pressure policy.
- Pinned and recently retold episodes survive ordinary eviction; unpinned
  episodes can be evicted without changing authoritative history or answer
  replay.
- Cache eviction leaves decisions unchanged. Recorded semantic recall/forget
  transitions replay exactly. Archive/compaction round trips preserve source
  references, correction chains, succession references, and the ability to
  explain an answer after cold reload.
- Corrupt, foreign, missing, or over-budget references are rejected
  atomically, leaving the prior memory projection unchanged.

`gcarena` is not a semantic forgetting solution. It may provide allocation or
storage mechanics only after this policy defines what is hot, cold, pinned,
archived, or forgettable.

## Long-lived save strategy

The current save shape is regrowable base state plus complete ordered intent
vectors. `GameState` also retains an in-memory `events` vector, while movement,
projects, and simulation retain parallel event/decision traces; these are
useful validation projections but are duplication risks over years. The shared
snapshot primitive is postcard-style `snapshot::encode` plus `hash_bytes`.
The checkpoint design below is proposed and needs a measured baseline before
choosing hard quotas.

First define a checkpoint representation and replay floor. A checkpoint stores
the generative base identity, edited world chunks and dirty-only publication,
current subject/population state, pending obligations, active agreements,
belief/norm revisions, pinned references, and the recent delta tail. Each
checkpoint records its ancestry and hashes. A complete checkpoint is independently
restorable: its ancestry digest is not a strong storage reference requiring every
previous checkpoint to remain. An incremental checkpoint names each actual base
dependency; periodically materialize a complete checkpoint to cut that dependency
chain before collection. The
delta tail covers the configured recent window; older deltas remain reachable
only when needed by a correction, succession, pin, pending obligation, or
retained evidence reference. Repetitive moves, waits, and equivalent routine
events may become per-subject aggregates only where the aggregate preserves
current semantics needed by decisions and provenance. Exact frame/event playback
before the declared replay floor is then unavailable unless retained separately;
do not describe lossy aggregation as exact historical replay. Do not delete
the full event history required by the current replay contract until a
checkpoint replacement has its own validated replay floor.

Current `GameState::state_hash` serializes the complete owner, including its
intent/event histories. Removing old trace entries necessarily changes that
hash even if the physical world is unchanged. Before checkpoint compaction,
define versioned continuation-state equality separately from retained-history
integrity: compare every state component that can affect future decisions,
including rule/RNG/attention state and unresolved references. Then verify that
full replay and checkpoint-plus-tail produce equal continuation state and equal
subsequent outcomes. Do not silently weaken the meaning of existing save hashes
or claim a compacted historical record is byte-identical to its predecessor.

Publication writes a new checkpoint and dirty chunks to temporary files, fsyncs
them, then atomically publishes a manifest. A crash may leave unreferenced
temporary data but never a partially visible checkpoint. Old immutable
checkpoints are garbage-collectable only by reachability from the current
manifest, pins, correction/evidence/succession references, and the configured
retention window. Keep at least two verified generations before collecting a
superseded generation. Retention windows and archive quotas are settings with a
deterministic pressure policy; reasonable slow world/content growth is allowed,
while per-frame history amplification is a failure signal.

Cache residency, gameplay memory, and durable retention are separate budgets.
Cache eviction cannot alter decisions. Gameplay memory may change recall only
through recorded `Recall`/`Forget` policy. Durable retention may compact or
aggregate permitted history, but must preserve obligations, corrections,
provenance links, pins, and the replay floor. Missing cold evidence is reported
as missing rather than reconstructed as certainty.

The measurement gate runs unattended years at representative populations and
records elapsed simulated ticks, subjects, intents, emitted events, births,
deaths, reports, changed world cells/chunks, checkpoint bytes, delta-tail
bytes, aggregate bytes, and archive bytes. It reports bytes per tick and per
subject/event class, plus projected growth under each retention setting. It
must compare at least two verified generations and exercise crash recovery;
targets are admitted only after this receipt, never presented as universal GB
claims.

Done-conditions:

- A checkpoint restores the exact current state from its base plus reachable
  deltas and rejects a broken hash, missing required base, or false reference.
  An independently complete checkpoint restores after its unreferenced ancestors
  have been collected; historical playback stops at its declared replay floor.
- Dirty-only publication and crash recovery are atomic; two successive
  checkpoints verify independently before old-generation GC is allowed.
- Aggregates have per-kind semantic proofs/tests and retain obligations,
  corrections, provenance, pins, and replay-floor references.
- The unattended-years receipt exposes rates and projections for multiple
  populations, changed-cell rates, births/deaths, reports, and all three
  storage classes.

### Save growth baseline, 2026-09-09

The runnable [save-growth probe](../crates/paredros-world/examples/save_growth.rs)
uses a generated world, one admitted wetland body and one carried dressing.
Each pair attaches and detaches that dressing. At every checkpoint, the current
world, body and items equal their initial values; restoring the complete save
equals the original `GameState`, including recorded history.

| Attach/detach pairs | Accepted intents | Encoded save bytes | Save ms | Restore ms |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 4 | 407 | 0.620 | 9.739 |
| 100 | 204 | 1,485 | 0.581 | 9.463 |
| 1,000 | 2,004 | 12,284 | 1.048 | 10.345 |
| 10,000 | 20,004 | 123,905 | 3.454 | 15.731 |

Receipt: [CSV](../testing/save_growth/2026-09-09.csv). Command:
`cargo run -p paredros-world --example save_growth --locked --offline -j 2
--target-dir target-contact`, with
`CARGO_HOME=C:/Users/mark_/Code/cargo-homes/paredros-save-check-20260908`.
This is the repository's optimized dev profile with debug information, one run
amid concurrent work. Timings are diagnostic samples, not a performance target.
All four restored-equality checks passed. These are encoded bytes without an
additional compression pass; neither RAM overhead nor retained previous saves
is included. It demonstrates history amplification with unchanged physical
state, not representative population, terrain-edit, memory or years-long growth.
The broader measurement gate above remains open.

## Stage F3b3: preferences, ideology reactions, and meaningful retelling

Preferences may begin as authored, generated, or inherited temperaments and
norms. Experience can then revise them through supported episodes; memory is a
route for change, not the only source of initial disposition. Ideology
reactions combine a typed world change with the person's observer-relative
norms, beliefs, preferences, needs, property/access position, and discovery
knowledge. Keep them bounded, revisioned, and explainable. A reaction names
the change, inputs, rule revision, and resulting disposition, rather than
becoming a universal alignment score.

Retelling is an explicit action or social transmission. It creates a report or
retelling record that points to the remembered episode and speaker, records
the listener(s), and may update attention. Retelling does not make the listener
believe the proposition automatically; the existing report/evidence rules
decide what can be claimed. Repeated attention may make an episode eligible
for hagiograph promotion, subject to the promotion policy.

Done-conditions:

- Authored/generated/inherited initial dispositions and later preference or
  ideology changes replay exactly and expose rule, source episodes, world
  change, needs, property/access, and discovery-knowledge references.
- A retelling is pointable to its source memory and speaker, and addressed
  listeners receive only the intended report.
- Contradictory observations and claimant corrections remain visible in the
  explanation surface; derived reactions do not erase disagreement.
- Tests cover repeated retelling, attention decay, contradictory support,
  discovery gating, affected access/needs, bounded reaction state, and replay.

## Stage F3b4: hagiograph promotion and conditional manifestation

Implement the reserved hagiograph boundary only after ordinary memory has
stable references. A promotion candidate is a proposal derived from durable
history and an authored promotion policy. Retelling/attention can qualify it,
but a noteworthy event may qualify without popular retelling when that policy
explicitly says so. Promotion does not depend on a full ideology model. The
candidate names source event/episode, subject and community witnesses,
qualifying rule revision, attention history, and exclusions. Admission is
deterministic and records a promotion ID, source references, and policy version.

The hagiograph view keeps promoted retellings, epithets, memorial status,
attention/retelling counts, corrections, and archive state. It may present a
legendary reading while retaining the factual source and contested versions.
Promotion is not a rewrite of the event journal. A manifestation proposal is a
separate, reviewable output consumed by the world-conditions owner.

The world-conditions plan owns whether a manifestation can be admitted under
current site, epoch, material, and causal constraints. The functional-loops
plan owns how that accepted consequence enters the wider loop. The hagiograph
adapter must reject a manifestation without an accepted condition receipt and
must retain the causal chain when one is admitted.

Done-conditions:

- Promotion requires qualifying source references and an admitted policy;
  attention evidence is required where that policy calls for it, while an
  authored noteworthy-event rule may qualify without popular retelling.
- Every promoted entry can be traced through retellings, observations/claims,
  and the authoritative event record, including corrections and disagreement.
- Promotion, archive, compaction, and attention updates are deterministic,
  bounded, versioned, and replayable.
- A manifestation is emitted as a proposal and becomes world state only via
  the world-condition owner; its receipt names both the promotion and the
  admitted conditions.
- Tests prove that retelling alone cannot mutate terrain or create a procedural
  event, while an accepted condition can produce the documented manifestation.

## Stage F3b5: canon revision, glyphs that vary over time by world criteria

**Terminology, 2026-09-15.** This stage was founded under the name
"hagioglyph". Mark ruled that day that hagioglyph names the whole divinity
organ (general model §7.4), so this stage's subject is now called the canon
revision, G5; the receipts below keep their original file names.

**Assessment, 2026-09-14 (ruled as a lane by Mark; not started).** A
glyph under revision is a base glyph whose live meaning follows a canon
revision the world publishes, while every acquisition keeps the meaning it
was acquired under. The kernel already has the pieces: `Canon::shuffled(seed, revision)`
makes a new monotonic revision that permutes the correspondence without
breaking its one-to-one mapping, and a `Journey` embeds its founding canon.
What it lacks is any record of which revision an acquisition was accepted
under: `Acquisition` carries glyph, provenance, tick and life only. The
general model §7.4 rule governs: canon changes during play need explicit
migration and must neither revoke a god nor complete an old collection under
different meanings. Completion and ascension therefore stay judged against
the journey's founding canon; only the live effect of an owned glyph moves.

**Drivers, all accepted history, never a rewrite.** A revision has a cause
the world records: a period settling (a `DomainWindow` receipt), a hagiograph
promotion admitted with its condition receipt (F3b4), or an authored epoch.
Paredros admits it as `GameIntent::ReviseCanon { tick, revision, seed,
cause }` producing `GameEvent::CanonRevised`; replay is deterministic and the
prior correspondence stays in the saved definition, as §7.4 requires.

**Scope.**

- `wing-glyphs` (shared; Paredros is the second consumer that challenges
  it): `Acquisition` and `GrantRecord` gain `canon_revision` with a serde
  default of the founding revision, and `Canon` gains a correspondence diff
  naming the bases whose effect moved between two revisions. No other
  kernel semantics change; the 16 kernel tests stay and gain their own.
- `paredros-world::glyphs`: the reading tracks the live canon revision from
  accepted `CanonRevised` events, stamps each evidence record and grant with
  the revision it was accepted under, and answers `founding_effect(glyph)`
  and `live_effect(glyph)` separately. Eligibility is unchanged. A promotion
  cause without an admitted condition receipt is refused (F3b4's rule).
- Session host: the acquisition journal shows the founding effect and, when
  it differs, the live effect and the revision's cause; the scenario asserts
  both after a scripted revision.
- Glyph marks in the scene, drawn against isometer's shared depth, may take
  the live display mark; that is presentation and follows the isometer
  family lane.

**Done when:** a revision mid-history leaves eligibility, ascension basis
and every founding meaning unchanged and replays identically from a restored
save; the same glyph acquired before and after a revision reports different
live effects with the revision and cause pointable from each; a promotion
cause is refused without a condition receipt and admitted with one; the
kernel's diff names exactly the moved bases; and the journal and scenario
show both meanings. Reincarnation, wishes and divine spending stay open.
Mesocosm's §7.4 gains the time-variation rule as a G-gate in its own doc,
since the shared design lives there.

**Landed 2026-09-14.** Kernel: `Acquisition` and `GrantRecord` carry
`canon_revision`, restored to the founding revision for older snapshots;
`Journey::grant_at_revision` stamps an accepted revision while `grant` keeps
its signature, so mesocosm-runtime compiles unchanged; and
`Canon::correspondence_diff` names the moved bases and refuses a different
canon or base set. 16 kernel tests became 19. World:
`GameIntent::ReviseCanon` and `GameEvent::CanonRevised` with
`CanonRevisionCause::{Period, Promotion, Authored}`; a promotion without
its condition receipt is refused at apply time; the reading derives each
revision from the founding canon by seed and revision, stamps its records,
and answers `founding_effect` and `live_effect` separately while eligibility
and the ascension basis stay on the founding canon. `GameIntent::subject`
became optional because a revision names no body, and `GameSave` moved to
version 7 with its gates reshaped so v3 to v6 archives restore exactly as
before; a pre-v7 archive carrying the new intent is refused. 140 world tests
became 143. Host: each journal row shows the founding effect and, when it
moved, the live effect and the revision's cause; `act revise-canon` is a
fixed-seed fixture control; the acceptance scenario asserts the revision
moved, the acquisition did not, and the live effect differs (106 frames,
ok). Rerun by the root: 19 kernel, 143 world, 45 lib tests, the clean
Paredros workspace check, the acceptance scenario and the final frame.
Receipt at `testing/session/receipts/2026-09-14/acceptance-hagioglyph.json`.
A formatter pass at the workspace level reflowed files other lanes own and
was reverted; `cargo fmt --check` fails at HEAD for the combat files, which
their owner will meet when they next format.

## Open decisions and risks

Choose initial salience inputs from event/deed facts and retelling counts before
adding personality axes. Measure initial global/per-subject budgets and any
per-class reservations, including serialized size in the receipt. Decide the
minimum stable reference vocabulary shared with Hagiograph; do not make the
reservation crate depend on Paredros.

The principal risks are accidental global knowledge, a second mutable copy of
world truth, unbounded support graphs, and promotion being treated as a direct
terrain authoring API. The tests and ownership gates above are admission
criteria for each risk. No phase requires a scene, renderer, or player-facing
screen; a native legibility surface can consume the same pointable records
later.

## Findings

**2026-09-09.** `EpistemicLog::observe` is currently deed-scoped, so an
event adapter needs a validated deduplicated source mapping or an explicitly
extended source vocabulary. `Society` already exposes ordered answer premises,
but standing is folded from deeds and norms are not yet a decision input.
`Simulation` persists decisions and game transitions without epistemic state.
`GameState`, movement, projects, and simulation retain parallel traces, and
`snapshot::encode`/`hash_bytes` is the shared serialization/hash primitive.
Hagiograph is still an unimplemented Mere reservation. These are inspected
seams, not landed memory behavior.

## Progress

**2026-09-09.** Plan drafted from the live Paredros social/world code,
Mesocosm memorial terminology, and the Hagiograph reservation. Linked from the
canonical index and execution plan. Memory/Hagiograph implementation remains
planned; the separate J0 body-sheet safety slice does not close these stages.

**2026-09-09 update.** Added checkpoint/replay-floor, dirty-only publication,
crash atomicity, reachability GC, retention settings, and unattended-years
measurement gates. No implementation or deletion occurred.
