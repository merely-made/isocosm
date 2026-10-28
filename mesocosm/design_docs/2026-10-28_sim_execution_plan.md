# The sim's execution model: speed, waking, parallel evaluation, near and far

**Status:** 2026-10-28: cut from `2026-10-19_sim_execution_plan.md`, plus the speed items of `2026-10-14_sim_host_plan.md`, `2026-09-22_sim_plan.md`, `2026-09-11_orthographic_voxel_presentation_plan.md` and `2026-10-22_foot_placement_plan.md`, which are archived in
`archive_docs/2026-10-28/`. Built and exact: stage bodies and parts by `Arc` (D5, D7), the narrower prey scan (D9a, D11), and the gate index's more-gates and re-filing parts (E1a, E1b); the index kept across advances (E1c). Open: E7 (rates worked out when read) and E8 (decision periods per lineage), then E4 and E5 (D17); E1 as a whole, E2, E3, E6, the tier line's band, and the speed bars. From E7 on, phases are checked in isolation while the sim is wired (D18).

**For:** every tick-latency, frame and search-time bar in the family, and the execution work that serves them: waking (E1), draw keys (E2), parallel evaluation (E3), the crowd in the core (E4), crossings and the tier line (E5), rate models (E6), the host's latency bars, cold-route and startup search time, and the scale and scheduler-waste items. This is the only plan that states a speed number; others name `sim-execution` and do not restate it.

Not here: the sim's phases and the parked rungs (`sim`); the session layer (`session-network`); the host's GUI layer and mode host (`sim-host`); the place store and lifts (`place-graph-engine`); the playable-loop forager route search (`bodies`, senses).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

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

- **D12, D10 tried where provably safe, then dropped (2026-10-23), applies
  Processes 8:** "Reuse where provably safe", then "Drop D10, go to E1
  (Recommended)". An act's effects read more than the pass's start: what
  lineages have learned (a meal), a live identity counter (a birth), notes,
  record highs, polities and projects. So a plan kept its stage only for a
  process of fixed transfers and plain transforms, nothing drawn, no owned
  lots, its shares covering all it took. That was exact and 7% slower,
  cloning the stage costing more than the four ledger effects it saved.
  Shared-ground acts keep both evaluations; E1 is next. This amends D10.

  Links: area Processes and rules · draws on 454, sim-execution D10 · amends sim-execution D10 · governs isocosm::execute · summary Keeping a plan's stage for its act, tried where provably safe, was exact but slower and is dropped; shared-ground acts evaluate twice.

- **D13, E1 in three parts (2026-10-23), applies Processes 7:** "E1a, then
  E1b, then E1c (Recommended)" and, for mood, "Measure, then decide
  (Recommended)". E1a makes more requirements gates: a computed one that
  reads only the actor's body (rest's healing), a site's condition, and
  mood, kept only if it measures faster. E1b re-files only the gates whose
  reads an act changed. E1c keeps the index across advances. Each is
  exact, landed and measured alone. As D7b found, a gate on a chosen
  process leaves deliberative members unfiltered, so no choice moves to a
  later pass.

  Links: area Processes and rules · draws on 258, 286, 683, 813, sim-execution D1, sim-execution D4 · governs isocosm::schedule::Filed · summary Waking is built as more gates, then re-filing by what an act changed, then an index kept across advances, each exact.

- **D14, E1a by cheap necessary gates, mood left out (2026-10-23), applies
  Processes 7:** "Cheap necessary gates (Recommended)". Gates that read a
  requirement whole, rest's healing reading among them, were exact but
  saved nothing: re-filing read the reading as often as acts were blocked
  by it. Rest is filed instead by what its reading needs before it can
  come to one, a lost cell whose loss was recorded; the act still reads it
  whole. A site's conditions, a lookup, stay read whole. Mood, read anew
  nearly every tick, is not gated. This amends D13.

  Links: area Processes and rules · draws on 258, 286, sim-execution D13 · amends sim-execution D13 · governs isocosm::schedule::Filed · summary Rest is filed by a lost cell its healing needs, a site's condition read as the act reads it; mood is not gated.

- **D15, a kind's decision rate on its lineage (2026-10-28), refines
  Processes 7:** "On the lineage (Recommended)". A lineage carries a
  decision period that paces its members' choices and moves; revisions and
  splits carry it. Asked after a talk on tick cost: "Every tick, every
  member is reconsidered for every act that's due ... That needs changing."

  Links: area Processes and rules · draws on 258, sim-execution D4 · governs isocosm::schema::Lineage, isocosm::schedule · summary A lineage's decision period paces its members' choices and moves.

- **D16, decisions are choices and moves (2026-10-28), refines Processes
  7:** "Choices and moves (Recommended)". Choice processes and movement are
  paced by the lineage's period; strain, death and growth are rates worked
  out when read, their crossings kept as due events per member, never
  slowed.

  Links: area Processes and rules · draws on 159, 227, 906, sim-execution D1 · governs isocosm::rules::mind, isocosm::ecology, isocosm::schedule · summary Choices and moves are paced by the lineage; bodily rates are worked out when read, their crossings due events per member.

- **D17, order (2026-10-28), amends D4 and D13:** "E1c, 2, 1, 3
  (Recommended)". E1c, then E7 (rates worked out when read), then E8
  (decision periods), then E4 and E5. E2 and E3 come back after E8 with
  new figures.

  Links: area Processes and rules · draws on sim-execution D4, sim-execution D13 · amends sim-execution D4, sim-execution D13 · governs isocosm::schedule · summary E1c, then rates worked out when read, then decision periods per lineage, then the crowd and the tier line.

- **D18, wired first, validated after (2026-10-28), refines Method 4:**
  "What I value is everything being wired up at the very least before I
  start making any type of validation scheme to figure out if something is
  consistent or works ... if you want to test each individual part, like at
  that point, it maybe makes a bit more sense to test it in isolation then.
  But like, as we're literally developing and writing each individual part,
  it does not make sense to test each individual part like it is the final
  game." Asked after E1c's gates ("Why do we need fidelity to previous
  behavior?"). The exactness and envelope constraints below are rewritten
  to it.

  Links: area Method and documents · draws on 791, sim-execution D13 · amends sim-execution D13 · summary While the sim is wired, a change is checked in isolation by its own tests, never against the previous build; validation of the whole is designed once it is wired.

## Constraints

- **Checked in isolation while wired (D18).** A phase is done when it compiles and the tests of the piece it touches pass, run alone. No battery, pin or certification runs per phase and nothing is checked against the previous build; a pin that breaks is re-recorded, not a bug. Done-conditions below naming an exact check or the envelope are read this way. Kept every phase: determinism within one build and conservation of matter, both unit tests. Superseded from E1c on: "A pure execution change leaves every state hash identical to the serial, unskipped run ... Any difference is a bug" (rulings 454, 958, D1, D4).
- **Once wired: crowd and tier (E4, E5),** as part of validating the whole: invariants exact (matter conserved every tick, noted members keep their identities, every crossing conserves totals); outcomes judged with the checkpoint 10 harness: paired worlds, Kolmogorov-Smirnov distance, paired permutation, DKW equivalence within the reading's bound, Holm-corrected, as it certified C10 and C11. "A pass is a run that passes; it is not a receipt to keep."
- **No cohort claim on projection-only evidence.** "No cohort execution claim lands on projection-only evidence. It needs an exact zero-tick round trip and a measured evolution envelope against the all-individual reference." (place plan stop rule).
- **Noted members never cross into a crowd; the played critter and every noted member run as individuals (113, D2).**
- **Choices and shared ground.** Anything that changes how a choice is made or kept (683, 813) or how shared ground is shared out (454) comes back to Mark as a question, not an optimisation (D8, D10, D12).
- **Search limits stay.** Do not prune stance states or lower the paid, posture, node or work limits (the 256-node, posture, work and paid-stride budgets stay) while cutting cold-route time.
- **Decided already, not reopened:** similitude (113), background and foreground (75), the boundary (211, 212), the crowd (206, 207, 219), rate models (261, 266, 440), crossings (435, 554, 431), where the tier line lives (742, 293), the crowd's chooser (881), the operation budget (259, 285).

## Speed bars

Every bar the family states, with its ruling or case. Measurements stated in old plans are not bars and are not repeated here.
- **The aim (TD2, 0.1 s):** "a played tick at or under 0.1 s with a few hundred members" (design tempo TD2, 10 ticks per second). No phase measured so far reaches it, and every added mechanic raises cost. Done condition of the plan as a whole.
- **Case 966:** the seed-9 save runs 15 ticks "under 1.5 s on the shipping head, the host unchanged" (0.1 s a tick, about a hundred members). Open; the old host plan says it "stays open for a later ruling with the scheduling profile".
- **Case 976:** "done when the seed-9 save runs 15 ticks in under 8 s", replacing 966's 1.5 s. Holds.
- **Case 978:** the head's default is 2 ticks per second (0.5 s a tick budget), ruled because a tick cost about 0.45 s and native scheduling alone 0.3 s; TD2's 10 stays "the aspiration revisited when scheduling drops". The 10-per-second configuration made the clock defer without limit with every callback at the 8-step ceiling.
- **Per-phase reading:** "ms per played tick in a grown world, once per phase" (no number).
- **Interactive frame budget:** none qualified. The 16.7 ms frame is the yardstick in the place plan, whose body-count figures are measurements, not budgets.
- **Search and startup time:** no number stated; the bar is "fluid play" for the cold first food-route round (EX2).
- **Memory for SP8:** no time bar, only memory bounded by the loaded window (`place-graph-engine SP8`).

Which governs today: 976 is the ruled bar for the seed-9 save and holds; 978 governs the head's default tempo; the 0.1 s aim at a few hundred members is what E1 to E5 work toward and is not itself a ruling over 976. 966's 1.5 s stays open. The two populations differ (about a hundred members for 966, a few hundred for the aim), and no document states which is the bar; that is a question for Mark.

## Phases

### E1. Waking
A process is reconsidered for a member only when something it reads about that member has changed since its last outcome, or when a time gate ripens; nudged deliberative members, terrain processes and anything whose reads cannot be stated statically always run. E1 is built in three exact parts (D13, D14): E1a (cheap necessary gates, mood left out), E1b (re-filing only gates whose reads an act changed) and E1c (the index kept across advances, `schedule/filed/kept.rs`) are built. Bounds from findings: a scratch probe put E1's ceiling at 14% of scheduling, and waking on an unchanged body would skip almost nothing, since strain, upkeep and hazard write nearly every member every tick; remaining blocked evaluations are mostly `mind:relief`, `mind:strain`, mood and `body:rest` healing, and moves on site conditions. The playable-loop route cache is waking as E1 means it, keyed by exact inputs; E1b and E1c can follow it.
- Done when: E1: "the played world's ticks skip members whose reads did not change" (read under D18).
- Needs: E1a, E1b, E1c (built)
- Rulings: 258, 286, 683, 813, D13, D14
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:121-127`, `:291-301`, `:377-396`, `:461-472`, `:485-493`.

### E2. Draw keys
Amount, bite, risk and soma draws keyed by actor, tick and process (D1, decided in). First check: a key of actor, tick and process is unique only if one actor cannot run one process twice in a tick; a grouped cohort running with multiplicity may, so E2 settles that first (an occurrence index in the key if so) and brings a fork back to Mark if the collision is real. Bite and amount draws are still keyed by the act number in code.
- Done when: "the pins are rerecorded under that decision and A1 holds at its measured rate."
- Needs: D1; the exactness constraint, qualified "exact once its draws are rekeyed"
- Rulings: 454, 958, D1, D4
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:128-131`, `:356-360`.

### E3. Parallel evaluation
Evaluate a pass's members concurrently against pass-start state (454) and commit serially in the existing order, so act numbers, child ids, notes, events, record marks and the budget's early exit come out as serial; results never depend on thread count. Live reads that break pass-start isolation must move commit-side: `known`, `Record`, commit-time `fits` and merges, nudge answers, spread direction, society checks, terrain revision, geometry readiness. No threads exist in the sim's evaluation today.
- Done when: "the exact check holds at one, two and all threads."
- Needs: E2 (act-keyed draws cannot resolve until serial order assigns numbers); D4's order
- Rulings: 454, D1
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:78-87`, `:132-134`, `:333-335`.

### E4. The crowd in the core
The probe's crowd (`probe/crowd/`) becomes an execution form for far sites, keeping the shared query and effect meanings (219); identity-bound processes stay on the exact tier; Mesocosm's worlds are statistical (D2): far sites crowd, the played critter and noted members individual.
- Done when: "the envelope check below certifies on Mesocosm's played worlds." (invariants exact, outcomes in distribution: the crowd-and-tier constraint above.)
- Needs: E1 to E3 first (D4, speed first); D2; the decided-already near/far rulings
- Rulings: 75, 113, 206, 207, 211, 212, 219, 259, 285, 435, 554, 431, 881, D2
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:135-138`, `:153-159`.

### E5. Crossings and the tier line
Wire `TierLine` into directing (742): lift a member from its bin by seed with condition causes drawn from the tally (435), fold back conserving totals; noted members never cross into a crowd. `TierLine` is exported from `directing/mod.rs` and referenced nowhere else.
- Done when: "the invariants below hold across crossings and the envelope still certifies."
- Needs: E4; the band ruling (Questions for Mark); D3
- Rulings: 113, 293, 431, 435, 554, 742, D3
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:139-141`, `:360-367`.

### E6. Rate models
Calibration per rules digest (440), for shared resources where the crowd stops paying (261, 266); uncalibrated worlds stay on the exact crowd. The hybrid background (exact crowd where grouping pays, a certified rate model below the members-per-state bound, every switch receipted) has no rate model today: with bodies the crowd groups only slightly more than one member per state, so its evaluation savings are small. A century at a minute-tick is far past the default advance limit; evaluation, not ticks, is the budget (284).
- Done when: "A rate model is fitted once per rules digest on the bench, certified against the exact runner as 207's crowd was" (§5.3, 440); worlds under rules without a certified model stay exact.
- Needs: E4; the bench (W4); `sim` S5 for the region run it serves
- Rulings: 207, 261, 266, 284, 439, 440, 441
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:142-143`; `archive_docs/2026-10-28/2026-09-22_sim_plan.md:727-745`, `:1713-1717`, `:1908-1912`.

### E7. Rates worked out when read
E7a: `mind:strain` and `mind:relief` (`rules/mind.rs`, both every tick for every member) become one rated account: a value, the tick it was settled and a rate, building while mood is low and bleeding while not, never below nought; reads settle it, a commit that changes what mood reads re-derives the rate, the state preimage settles first, and a crossing of the bearing is a due event. Mood must change only at a commit: where a world's needs read anything but the actor's body, strain stays on its processes (Mesocosm's played needs read only the body, `directing/found.rs`). E7b: `ecology:hazard-{i}`'s risk draw each tick from maturity becomes one draw of the tick of death at maturity, at the same per-tick odds and keyed by actor, tick and process (D1), kept as a due event; age and starvation wake at their ticks (`ecology.rs`). Upkeep stays a process: its matter reaches a site's soil, which producers read. Removing strain's acts moves the act counter, so bite and amount draws move until E2.
- Done when: strain and death run without their per-tick passes and their own tests pass (D18).
- Needs: E1c (built); D16
- Rulings: 159, 163, 227, 906, D1, D16

### E8. Decision periods per lineage
A lineage carries a decision period, one by default, pacing its members' choices and moves (D15, D16): the queue keys passes by rate class (`schedule/mod.rs`), filing holds each group under its class (`schedule/filed.rs`), and a rate-type effect run less often is scaled by its period. Mesocosm's periods per kind are then set as a ruling, offered as options.
- Done when: a slow lineage's choices and moves run at its period in a unit test (D18).
- Needs: E7; D15, D16
- Rulings: 258, D15, D16, D17

### EX1. The head's interactive latency
Active-update latency in the head: native scheduling, clock preparation and physical refresh over the active ticks; Session and Game copying and refresh checks need separate timings before further optimisation; walking preparation, physical forecasts, source replay and the resumed-tick boundary remain expensive, and forecast and source-replay reuse is open. The cohort and stage `Arc` sharing has landed. "This pass does not qualify an interactive frame budget."
- Done when: not stated beyond cases 966 and 976 (Speed bars); no interactive frame budget is qualified
- Needs: E1 to E3 (scheduling is the floor); forecast and source-replay reuse; the two owed rulings under Questions for Mark
- Rulings: 966, 975, 976, 978
- From: `archive_docs/2026-10-28/2026-10-14_sim_host_plan.md:412-499`, `:324-357`.

### EX2. Cold-start search and derivation
Three measured pressure points for fluid play and larger sites: (pres.1) difficult cold first food-route rounds still pause too long for fluid play; remaining pressure is distinct sampled terrain-clearance work, ridge's distinct arrival checks and repeated frontier trials, with about half the search time outside the measured stride spans; profile endpoint grounding, candidate generation and graph search separately before changing search policy; exact-reuse owners are `route::search`, `steps::articulated_neighbours`, `footholds::arrivals`, `walking::samples` and `check_frame`. (pres.2) Startup and viability search: a fresh draw's setup is dominated by founder placement and terrain-place reads during viability search (128-candidate bound, many rejected for physical stands or nearby food); startup optimization is named "a later slice". (pres.3) Cold Places derivation, forecast walking and rest work, and new-page fills are the pressure points for larger sites; a 512-square materialized reference is deliberately not attempted and Places at that size is not derived. The foot-placement plan's startup and repeated-work item (cold terrain and place builds, route searches, stride plans) is taken up here; its remainder is pres.1 and pres.2.
- Done when: not stated; the follow-up prose says "the cold first round is still too long for fluid play"
- Needs: E1 to E5 for any structural fix (near/far tier, waking); the search-limits constraint
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:81-86`, `:171-177`, `:263-269`, `:362-385`; `archive_docs/2026-10-28/2026-10-22_foot_placement_plan.md:506-507`.

### EX3. Scale stress
A matrix of world size against population with tick cost, hash cost, memory and residency, every number reproducible from a cargo run; the `isocosm-scale` remeasure is its first native instance. The aggregate form and cohort execution it once covered are E4 to E6; zoom is the presentation plan's L10.
- Done when: "every number reproducible from a cargo run"; no other threshold stated
- Needs: -
- Rulings: 793
- From: `archive_docs/2026-10-28/2026-09-22_sim_plan.md:1393`.

### EX4. Scheduler waste and remaining scheduling cost
After the narrowed prey scan, shared-ground passes (`produce-3`, `feed-1`) evaluate each member before acting it, deliberation for members who then chose another process, and accepted acts (`mind:strain` the largest part) remain the cost with no phase; D12 leaves shared-ground acts both evaluations. Where a world holds owned projects, `begin_process_debits` digests the whole state for every act (A1 holds none). Of the older findings only one is live: the generator's competitions are stored by id with only one allowed (the probe refuses worlds with more than one; how several share a round is undesigned).
- Done when: not stated; owed
- Needs: D8 to D12; anything changing a choice or shared ground returns as a question (constraint)
- Rulings: 206, 220, 306, 454, 683, 813
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:236-244`, `:394-396`, `:408-428`; `archive_docs/2026-10-28/2026-09-22_sim_plan.md:1658-1659`, `:1712-1717`.

## Deferred and parked
- **Playable-loop forager route search** (the `playable-loop` branch, unpushed at 2026-10-23): its cost is deliberation the A1 battery does not exercise and no battery measures it. Owned by `bodies`; `sim-execution` reads its cost once a battery exists. From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:461-472`.

## Questions for Mark
- **The tier line's band (owed before E5).** `TierLine::default()` is promote at 1 hop, demote at 2, and hops are whole numbers, so the band the doc comment calls memory is empty; a member stepping between one and two hops changes tier every tick, each change a crossing. The 3x3 enclosure has diameter two. Options stated: "keep 1 and 2, widen where worlds are larger, or a band in ticks." From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:360-367`.
- **Which speed bar is the bar.** 966's 1.5 s on the seed-9 save (about a hundred members) stays open for a ruling "with the scheduling profile"; 976's 8 s holds; 978 sets the head at 2 ticks per second with 10 the aspiration; the aim is 0.1 s at a few hundred members. Rule whether the aim replaces 966 and when the head's tempo is raised (the 1,440-tick day of the light field, 973, is 12 minutes of play at 2 ticks per second). From: `archive_docs/2026-10-28/2026-10-14_sim_host_plan.md:7-13`, `:359-370`.
- **Is the tier line independent of the viewing dimension (gap raised 2026-10-27).** Whether the tier line must be independent of the 2D, 2.5D and 3D views, so E5's seeded lift keys on site, lineage and tick and never on a view event. Intrinsic world dimension may legitimately change aggregate results; viewing dimension may not. No old plan states it; it is raised here as a question, not a ruling. From: gap raised 2026-10-27.
- **The `cargo test` profile.** The played seed 0 runs far slower under the `cargo test` profile than release, and with debug assertions off drops substantially. The likely source is named but "not strictly separated": the journal's full `State` clone verifying each rollback (`journal.rs:66`) and `passed_by`'s population rescan (`schedule/mod.rs:314`). No action decided. From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:350-355`.
- **Occurrence index in E2's key (conditional).** If the actor, tick and process key collides for a grouped cohort, E2 brings the fork to Mark. From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:128-131`.

## Findings
- 2026-10-28: E1c built (`schedule/filed/kept.rs`): the index is reconciled at each advance by each group's body handle and each site's conditions, rebuilt on other rules, movement mode or routes, dropped on a refused advance. A played tick now costs well under 978's premise of 0.45 s (receipts: `archive_docs/2026-10-28/2026-10-28_sim_execution_plan_receipts.md`), so its tempo question is open again. No threads exist in the sim's evaluation (thread scopes only in the probe bin and interim reproduction), so E3 is not built.
- 2026-10-28: E2 is not built: bite and amount draws are still keyed by the act number (`stage/staged/meal.rs`, `stage/staged/mod.rs`); soma draws by id.
- 2026-10-28: The crowd lives only in `shared/isocosm/src/probe/crowd/`; `TierLine` (`directing/tier.rs`) is exported and unwired.
- 2026-10-28: The head's default is 2 ticks per second (`mesocosm-genet/src/app/config.rs`).
- 2026-10-28: What D15 to D17 touch: a process has one `period` for the world (`rules/mod.rs:193`); the queue is keyed by due tick, priority and id, and a pass walks every group or every filed one (`schedule/mod.rs`). Strain and relief run every tick for every member (`rules/mind.rs:50`); so do each lineage's death, age and, from maturity, hazard processes (`ecology.rs:133-190`). The agentless `body:hazard` acts once per world body, not per member (`harm/hazard.rs`). Bite and amount draws are keyed by the act number, so any phase removing accepted acts moves them until E2.
- 2026-10-28: D4's order ("speed first: E1, E2, E3, then E4 and E5") was amended by D7 and D8, which inserted D7a, D7b, D9a, D11 and E1a before E1 completes; no later decision restates the order.
- 2026-10-28: D9b and D10 are dropped (D11, D12); the old plan's status line and its phase list disagreed, and the cut lists neither as a phase.
- 2026-10-28: The old host plan's earlier receipt figures (cohort sharing on the core-tick lane) are not comparable with the execution plan's battery figures: D5 and D7a redid the sharing as stage-body and part `Arc`s.

## Progress
- 2026-10-28: cut from `2026-10-19_sim_execution_plan.md` and the speed items of the sim host, sim, orthographic voxel presentation and foot placement plans.
- 2026-10-28: D15 to D17 answered (decision periods per lineage, rates worked out when read, order); E1c built; D18 (checked in isolation while wired); E7 and E8 added.
