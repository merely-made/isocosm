# The sim's execution model: speed, waking, parallel evaluation, near and far

**Status:** 2026-10-31: cut on 2026-10-28 from `2026-10-19_sim_execution_plan.md`, plus the speed items of `2026-10-14_sim_host_plan.md`, `2026-09-22_sim_plan.md`, `2026-09-11_orthographic_voxel_presentation_plan.md` and `2026-10-22_foot_placement_plan.md`, which are archived in
`archive_docs/2026-10-28/`. Built and exact: stage bodies and parts by `Arc` (D5, D7), the narrower prey scan (D9a, D11), and the gate index's more-gates and re-filing parts (E1a, E1b); the index kept across advances (E1c); strain worked out when read (E7a); decision periods per lineage (E8) at 10 / 2 / 5 with meals scaled (D19, D20), a paced Grouped span run a tick at a time so saves replay (D41); the head at 10 ticks a second (D21); a hazard whose only outcome is death drawn once, its tick a due event (E7b, case 1181). Built in part: E4a (far members fold into bands with a pool, where a world declares banding) and E5: a camera frame roots its posed members, and elsewhere members leave the physical path once a world has attention windows (E5a-b); members are drawn into a frame by its share of the site (E5c); the runtime holds the critter's frame as attention, and far arrivals and births land without a pose (E5d). The frame is on in main since D36 to D38 (far foragers wait at site grain and read prey by their own intake; resting bodies re-settle); a crowded frame draws by the ordinary search (D39) and stops at its first failure (D40). Open: E4b (bands act once for all members, effects scaled by count), the record's stores per component (D42, E9), E1 as a whole, E2, E3, E6, the tier line's band and the speed bars. The Places fix D30 asks for is `place-graph-engine` PG7. From E7 on, phases are checked in isolation while the sim is wired (D18).

**For:** every tick-latency, frame and search-time bar in the family, and the execution work that serves them: waking (E1), draw keys (E2), parallel evaluation (E3), the crowd in the core (E4), crossings and the tier line (E5), rate models (E6), the host's latency bars, cold-route and startup search time, and the scale and scheduler-waste items. This is the only plan that states a speed number; others name `sim-execution` and do not restate it.

Not here: the sim's phases and the parked rungs (`sim`); the session layer (`session-network`); the host's GUI layer and mode host (`sim-host`); the place store and lifts (`place-graph-engine`); the playable-loop forager route search (`bodies`, senses).

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

  Links: area Processes and rules · draws on 454, 958 · governs cosmograph::execute, cosmograph::stage · summary Amount, bite, risk and soma draws are keyed by actor, tick and process, never by commit order, so evaluation is order-free; one deliberate hash move.

- **D2, exactness (2026-10-19), applies Processes 6, refines Mesocosm 5:** "Statistical (Recommended)". Mesocosm's far sites run as the
  crowd; the played critter and every noted member always run as
  individuals.

  Links: area Processes and rules · draws on 75, 113, 211 · governs cosmograph::probe::crowd · summary Mesocosm's worlds are statistical: far sites run as the crowd, the played critter and every noted member as individuals.

- **D3, near (2026-10-19), refines Directing 5:** "Roots plus one hop (Recommended)". Attention roots and everything
  within one hop of the played critter's site; promote within one hop, demote
  at two or more.

  Links: area Directing and play · draws on 212, 742 · governs cosmograph::directing::tier · summary Near is the attention roots and everything within one hop of the played critter's site; the band is owed before E5.

- **D4, order (2026-10-19), refines Processes 7:** "Speed first (Recommended)". E1, E2, E3, then E4 and E5.

  Links: area Processes and rules · draws on 978 · governs cosmograph::schedule · summary Speed first: waking, draw keys and parallel evaluation (E1 to E3) before the crowd and the tier line (E4, E5).

- **D5, stage bodies by `Arc` (2026-10-20), applies Schema 9:** "Accept A, you number it". The
  stage holds the `Arc<Entity>` that `body_shared_at_start` returns and
  copies a body only on its first write, so planned and blocked acts copy
  nothing and commit installs the staged `Arc` instead of copying again in
  `Population::lift`. Carries ruling 975 into the stage; ruling 338's reads
  are unchanged; serialisation, hashes and saves unchanged. Whether `Part`
  also goes behind an `Arc` comes back once this is measured.

  Links: area The sim's schema and saves · draws on 338, 975 · governs cosmograph::stage::Stage, cosmograph::population::Population · summary The stage holds each body by the Arc it starts with and copies it only on its first write; serialisation, hashes and saves unchanged.

- **D6, `foregone` only on direct acts (2026-10-20), applies Directing 4:** "Accept B, you number it".
  `run_inner` fills `Receipt.foregone` for `Run::Act` only; scheduled and
  planned receipts, whose callers keep only `.accepted()`, carry an empty
  list. Law A's record and ruling 685's reading are kept receipts, which
  still carry it.

  Links: area Directing and play · draws on 685 · governs cosmograph::execute, cosmograph::simulation::Receipt · summary A receipt's foregone list is filled for direct acts only; scheduled and planned receipts carry an empty one.

- **D7, parts by `Arc` before E1 (2026-10-23), applies Schema 9:** "Part by
  Arc, then gate index (Recommended)". A body's parts map holds each part
  by `Arc`, so copying a body copies handles and a write copies only the
  part it touches; serialisation, hashes and saves unchanged. Then the gate
  index re-files a group reading its books once, not once per gate, and
  files moves by the site their actor lives at. E1 comes back after both,
  with new figures. This settles D5's open half and amends D4's order on
  the measured split (Findings, 2026-10-23, where an accepted act goes).

  Links: area The sim's schema and saves · draws on 975, sim-execution D5 · amends sim-execution D4 · governs cosmograph::schema::Entity, cosmograph::schedule::Filed · summary A body's parts are held by Arc and copied only when written; that and a cheaper gate index come before E1.

- **D8, deliberation and planning before E1 (2026-10-23), applies Directing
  4 and Processes 8:** "Deliberation and planning". After D7, a
  deliberative member's choice and a shared-ground pass's plan are each
  about 15% of scheduling (Findings, 2026-10-23, after D7). Each is probed
  first; anything that would change how a choice is made or kept (683,
  813) or how shared ground is shared out (454) comes back as a question.
  E1 waits; this amends D7's order.

  Links: area Directing and play · draws on 454, 683, 813, sim-execution D7 · amends sim-execution D7 · governs cosmograph::directing::choice, cosmograph::schedule · summary After D7, deliberation and a shared-ground pass's plan are probed and cut before E1, keeping how choices are made and ground shared.

- **D9, deliberation cut without changing a choice (2026-10-23), applies
  Directing 4:** "Narrow the prey scan (Recommended), Skip unchanged
  re-choices". A prey search visits only the candidates the cheap test
  that rejects most of them would pass, indexed by that test, so the same
  prey are accepted and the same draw made; and an empty choice is scored
  again at a later pass start (813) only where the world has changed since
  it was last scored, every change bumping a revision. Scoring feeding on
  any acceptable prey, which would change the prey a suggestion names, was
  not chosen.

  Links: area Directing and play · draws on 683, 813, sim-execution D8 · governs cosmograph::targets, cosmograph::directing::choice · summary A prey search visits only possible prey, and an empty choice is scored again only when the world has changed since.

- **D10, a shared-ground act keeps its planned writes (2026-10-23), applies
  Processes 8:** "Reuse the plan's stage (Recommended)". When an act's
  shares cover what it planned to take, it keeps the stage its plan staged
  and only re-checks that it fits and commits, rather than evaluating
  again; this holds only if effects read nothing but the pass's start, and
  the exact check decides it, any difference coming back as a question.

  Links: area Processes and rules · draws on 454, sim-execution D8 · governs cosmograph::schedule, cosmograph::execute · summary An act whose shares cover its plan keeps the planned stage and only re-checks fit before commit.

- **D11, D9b dropped and the prey scan narrowed further (2026-10-23),
  applies Directing 4:** "Drop D9b (Recommended)" and "Both narrowings
  (Recommended)". After D9a, skipping unchanged re-choices would save 0.6
  s of 93.6 s of scheduling, under 1%, for a revision every kind of change
  must bump; 813 stays as it is. A successful prey search, now about 11%,
  reads only the prey a liveness-naming selector could accept, the index
  filing living and dead bodies apart, and tests the requirements that do
  not read the prey once per search, those that do once per candidate; the
  same prey are accepted and the same draw made. This amends D9.

  Links: area Directing and play · draws on 683, 813, sim-execution D9 · amends sim-execution D9 · governs cosmograph::targets · summary Skipping unchanged re-choices is dropped; a prey search files living and dead apart and tests prey-independent requirements once.

- **D12, D10 tried where provably safe, then dropped (2026-10-23), applies
  Processes 8:** "Reuse where provably safe", then "Drop D10, go to E1
  (Recommended)". An act's effects read more than the pass's start: what
  lineages have learned (a meal), a live identity counter (a birth), notes,
  record highs, polities and projects. So a plan kept its stage only for a
  process of fixed transfers and plain transforms, nothing drawn, no owned
  lots, its shares covering all it took. That was exact and 7% slower,
  cloning the stage costing more than the four ledger effects it saved.
  Shared-ground acts keep both evaluations; E1 is next. This amends D10.

  Links: area Processes and rules · draws on 454, sim-execution D10 · amends sim-execution D10 · governs cosmograph::execute · summary Keeping a plan's stage for its act, tried where provably safe, was exact but slower and is dropped; shared-ground acts evaluate twice.

- **D13, E1 in three parts (2026-10-23), applies Processes 7:** "E1a, then
  E1b, then E1c (Recommended)" and, for mood, "Measure, then decide
  (Recommended)". E1a makes more requirements gates: a computed one that
  reads only the actor's body (rest's healing), a site's condition, and
  mood, kept only if it measures faster. E1b re-files only the gates whose
  reads an act changed. E1c keeps the index across advances. Each is
  exact, landed and measured alone. As D7b found, a gate on a chosen
  process leaves deliberative members unfiltered, so no choice moves to a
  later pass.

  Links: area Processes and rules · draws on 258, 286, 683, 813, sim-execution D1, sim-execution D4 · governs cosmograph::schedule::Filed · summary Waking is built as more gates, then re-filing by what an act changed, then an index kept across advances, each exact.

- **D14, E1a by cheap necessary gates, mood left out (2026-10-23), applies
  Processes 7:** "Cheap necessary gates (Recommended)". Gates that read a
  requirement whole, rest's healing reading among them, were exact but
  saved nothing: re-filing read the reading as often as acts were blocked
  by it. Rest is filed instead by what its reading needs before it can
  come to one, a lost cell whose loss was recorded; the act still reads it
  whole. A site's conditions, a lookup, stay read whole. Mood, read anew
  nearly every tick, is not gated. This amends D13.

  Links: area Processes and rules · draws on 258, 286, sim-execution D13 · amends sim-execution D13 · governs cosmograph::schedule::Filed · summary Rest is filed by a lost cell its healing needs, a site's condition read as the act reads it; mood is not gated.

- **D15, a kind's decision rate on its lineage (2026-10-28), refines
  Processes 7:** "On the lineage (Recommended)". A lineage carries a
  decision period that paces its members' choices and moves; revisions and
  splits carry it. Asked after a talk on tick cost: "Every tick, every
  member is reconsidered for every act that's due ... That needs changing."

  Links: area Processes and rules · draws on 258, sim-execution D4 · governs cosmograph::schema::Lineage, cosmograph::schedule · summary A lineage's decision period paces its members' choices and moves.

- **D16, decisions are choices and moves (2026-10-28), refines Processes
  7:** "Choices and moves (Recommended)". Choice processes and movement are
  paced by the lineage's period; strain, death and growth are rates worked
  out when read, their crossings kept as due events per member, never
  slowed.

  Links: area Processes and rules · draws on 159, 227, 906, sim-execution D1 · governs cosmograph::rules::mind, cosmograph::ecology, cosmograph::schedule · summary Choices and moves are paced by the lineage; bodily rates are worked out when read, their crossings due events per member.

- **D17, order (2026-10-28), amends D4 and D13:** "E1c, 2, 1, 3
  (Recommended)". E1c, then E7 (rates worked out when read), then E8
  (decision periods), then E4 and E5. E2 and E3 come back after E8 with
  new figures.

  Links: area Processes and rules · draws on sim-execution D4, sim-execution D13 · amends sim-execution D4, sim-execution D13 · governs cosmograph::schedule · summary E1c, then rates worked out when read, then decision periods per lineage, then the crowd and the tier line.

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

- **D19, how often each kind decides (2026-10-28), applies Processes 7:**
  "10 / 2 / 5 (Recommended)". A played world's producers decide every 10
  ticks, consumers every 2, decomposers every 5 (`ecology::rates::DECIDES`).

  Links: area Processes and rules · draws on sim-execution D15, sim-execution D16 · governs cosmograph::ecology::rates, cosmograph::directing::found · summary Producers decide every 10 ticks, consumers every 2, decomposers every 5.

- **D20, meals scale with the decision period (2026-10-28), applies
  Processes 7:** "Scale meals by period (Recommended)". A meal a choice
  takes is its amount times its eater's period, so a slower decider eats as
  much over the ticks it lets pass.

  Links: area Processes and rules · draws on sim-execution D15, sim-execution D19 · governs cosmograph::stage · summary A meal taken by a choice is its amount times the eater's decision period.

- **D21, the head at 10 ticks a second (2026-10-28), amends case 978:**
  "10 a second (Recommended)". TD2's aspiration becomes the head's default
  once decision periods brought a played tick near 20 ms.

  Links: area Processes and rules · draws on 978, sim-execution D19 · amends 978 · governs mesocosm_genet::app::config · summary The head's default tempo is 10 ticks a second.

- **D22, E7b before the crowd tier (2026-10-28), applies D17:** "E7b, then
  the crowd tier". E8 was built before E7b once rest's deliberation proved
  the larger cost; E7b comes next, then E4 and E5.

  Links: area Processes and rules · draws on sim-execution D17 · governs cosmograph::ecology · summary E7b, then the crowd and the tier line.

- **D23, three tiers (2026-10-28), applies Processes 6:** "I think the near
  tier is necessarily dependent on the viewing dimension, but the far tier,
  well, there's like a far tier, like in the way that people use
  billboarding, where like if something is really, really, really far away,
  then they'll just billboard it, you know? And then there's the far tier of
  like, it's not in this area. You could not possibly see it. Then that's
  like aggregate tier, right?" Recorded by another session as a constraint
  (below) and keyed here. How near as the rendered area sits with D3's
  "roots plus one hop" comes back with the band question before E5.

  Links: area Processes and rules · draws on 113, 742, sim-execution D2, sim-execution D3 · governs cosmograph::directing::tier · summary Near is the rendered area, view-dependent; far within it is presentation; the aggregate tier lies outside it and never depends on the view.

- **D24, the aggregate runs a population model (2026-10-28), refines
  Processes 6:** "Population model (Recommended)". Per site and lineage:
  counts by age band and condition (fed, hungry, wounded), its matter, and
  births, deaths, meals, upkeep and moves as rates; bodies are drawn from the
  lineage's recipe and recorded variation when lifted; noted members never
  fold. Asked after a measure of about 1.2 members per exact body state in
  played worlds, so the exact-state crowd would barely compress them.

  Links: area Processes and rules · draws on 113, 207, 261, 266, sim-execution D2, sim-execution D23 · amends sim-execution D2 · governs cosmograph::probe::crowd · summary Outside the rendered area, a population model per site and lineage, not a histogram of exact states.

- **D25, near is the rendered area plus the roots (2026-10-28), refines
  Directing 5:** "Rendered area plus roots (Recommended)". Everything
  rendered, plus the played critter, pinned and examined things and those
  they care for, wherever they are. This replaces D3's one hop.

  Links: area Directing and play · draws on 212, 742, sim-execution D3, sim-execution D23 · amends sim-execution D3 · governs cosmograph::directing::tier · summary Near is the rendered area plus the attention roots and those they care for.

- **D26, a band in ticks (2026-10-28), refines Directing 5:** "A band in
  ticks (Recommended)". A member keeps its tier until it has been out of
  that tier's area for a band of ticks, a world rule.

  Links: area Directing and play · draws on 742, sim-execution D25 · governs cosmograph::directing::tier · summary A member keeps its tier until out of its area for a band of ticks.

- **D27, bands run through the scheduler (2026-10-28), applies Processes
  6:** "Through the scheduler (Recommended)". Each band is a representative
  body with a count, run by the existing scheduler and process definitions
  with multiplicity (219); feeding, risk and death gain band-aware forms.

  Links: area Processes and rules · draws on 219, sim-execution D24 · governs cosmograph::schedule, cosmograph::population · summary A band is a representative body with a count, run by the scheduler with multiplicity.

- **D28, a representative plus a pool (2026-10-28), applies Processes 6:**
  "Representative plus pool (Recommended)". The band keeps a ledger of the
  difference between its members' summed matter and count times the
  representative's, so totals are conserved; lifting draws from it.

  Links: area Processes and rules · draws on sim-execution D24, sim-execution D27 · governs cosmograph::population · summary A band's pool holds the difference between its members' matter and the representative's times its count.

- **D29, bands by age and felt needs (2026-10-28), applies Processes 6:**
  "Age band and felt needs (Recommended)". Besides site and lineage, a band
  is an age band of a world-rule width (default a third of maturity) and
  the set of the mind's needs its members feel.

  Links: area Processes and rules · draws on 227, sim-execution D24 · governs cosmograph::population · summary A band is a site, lineage, age band and set of felt needs.

- **D30, the Places cache, then far members off the physical path
  (2026-10-28), amends D22:** "Places cache, then far off-path
  (Recommended)". Asked after the head measured about 12.5 s a played tick,
  nearly all of it foraging route search, Places derivation and walking
  plans (Findings). First why Places is derived anew so often, then the
  aggregate tier's members stop walking and route-searching (E4b, E5).

  Links: area Processes and rules · draws on sim-execution D22, sim-execution D24 · amends sim-execution D22 · governs cosmograph::spatial · summary Fix the Places re-derivation, then take far members off the physical path.

- **D31, the head stays at 10 ticks a second (2026-10-28), applies D21:**
  "Keep 10, revisit after (Recommended)". The clock defers at either tempo
  while a tick costs seconds; re-measure once the physical work is cut.

  Links: area Processes and rules · draws on sim-execution D21 · governs mesocosm_genet::app::config · summary The head's target stays 10 ticks a second, revisited after the physical work is cut.

- **D32, the camera's frame is a region (2026-10-28), applies Directing 5:**
  "Camera frame as a region (Recommended)". The head reports its view
  windows (a site and voxel bounds) as an attention region; near is inside
  a frame or a root. Far members hold no pose and forage at site grain;
  members are drawn into a frame by its share of the site, placed by the
  arrival policy, and fold back after the collection buffer once outside.
  Asked after the Places cache proved to thrash because every member of a
  one-site world walks it (a 1,024-window cache cut derivations from 356 to
  156 at a 6.1 GB peak; reverted).

  Links: area Directing and play · draws on 742, sim-execution D23, sim-execution D25, sim-execution D26, sim-execution D30 · governs cosmograph::attention, cosmograph::spatial · summary The head's view windows are an attention region; outside them members hold no pose.

- **D33, the frame off on main for now (2026-10-28), applies D18:** "Frame
  off on main for now (Recommended)". A headed look at E5d found the played
  critter's view blank after it migrated (its arrival pose fails the balance
  check) and deaths up from 2 to 22 by tick 20; the runtime's frame attach
  is held behind `FRAME_TIER` until the fixes land together on the branch.

  Links: area Processes and rules · draws on sim-execution D32 · governs mesocosm_runtime::runtime · summary The head reports no frame on main until E5's fixes land together.

- **D34, far meals at the walkers' measured rate (2026-10-28), applies
  Processes 6:** "The walkers' measured rate (Recommended)". A far forager's
  meal succeeds at the per-tick rate near foragers actually eat, measured
  from the near tier and kept as a world rule, so the far tier agrees with
  the near in distribution.

  Links: area Processes and rules · draws on 113, sim-execution D2, sim-execution D24 · governs cosmograph::spatial::foraging · summary A far forager eats at the rate near foragers are measured to eat.

- **D35, far meals by an encounter rate from prey density (2026-10-28),
  replaces D34:** "Encounter rate from prey density (Recommended)". A far
  forager finds food with chance M / (M + K): M the prey matter its diet
  can eat at its site, shared among that diet's foragers there; K one
  meal's worth times a half-saturation constant, a world rule. Asked after
  D34's live measurement proved biased: near foragers are few and see only
  posed food (5 meals per 61 hungry ticks), so paced far foragers starved
  (11 dead by tick 21) and unpaced ones overgrazed and crashed (9 dead).

  Links: area Processes and rules · draws on 113, sim-execution D24, sim-execution D34 · replaces sim-execution D34 · governs cosmograph::spatial::foraging · summary A far forager finds food by a saturating encounter rate on its site's prey matter.

- **D36, far foragers wait at site grain (2026-10-28), applies Processes
  6:** "Wait at site grain (Recommended)". A far forager's reach is its
  site: where its site holds food its intake admits, the feed's need score
  is its wait score, so weaker moves are skipped until the feed is due, as a
  posed forager waits with food in reach. Asked after the frame's extra
  deaths were traced: a far forager could neither walk to food nor wait for
  it, so between feeds its only choice was a move, and consumers left the
  only site with prey and starved elsewhere.

  Links: area Processes and rules · draws on 113, sim-execution D32, sim-execution D35 · governs cosmograph::spatial::foraging · summary A far forager whose site holds food it can eat waits there for its feed, as a posed forager waits with food in reach.

- **D37, far prey read by the forager's own intake (2026-10-28), applies
  Processes 6:** "Each forager's intake (Recommended)". D35's M is the prey
  matter the ordinary target search accepts for that forager at its site,
  the weight its draw shares out, so admitted by the forager's own intake;
  the far-food table keeps only foragers per site and feed. Asked after the
  table was found counting matter no diet there admits (a site holding only
  consumers and the ground read as prey), so far draws hit and then found
  nothing.

  Links: area Processes and rules · draws on sim-execution D35 · governs cosmograph::spatial::tier, cosmograph::targets · summary A far forager's prey matter is what the ordinary search accepts for it, by its own intake.

- **D38, a resting body re-settles (2026-10-28), applies Bodies 2:**
  "Re-settle at advance end (Recommended)". At the end of each advance a
  posed member whose stance no longer balances takes the nearest balanced
  stance around it by the placement search arrivals use, published as a
  draw the game applies. Asked after the critter's blank view was traced:
  a landing admitted at the critter's matter failed the balance check once
  it lost matter standing still, its stance near the edge of its feet, and
  nothing re-poses a resting body, since matter does not change its body
  revision; the same happens with the frame off.

  Links: area Bodies and anatomy · draws on sim-execution D32, sim-execution D33 · governs cosmograph::spatial · summary A posed member whose stance stops balancing takes the nearest balanced stance at the end of the advance.

- **D39, a crowded frame draws by the ordinary search (2026-10-28),
  applies Directing 5:** "Fall back, skip repeats (Recommended)". A member
  drawn into a frame stands near its placed food by the arrival policy;
  where no stance near that food is free, it takes the search a member with
  no food placed takes, around the frame's anchor, and walks to food from
  there. After the first such failure in an advance, later members with the
  same placed food go straight to that search. Asked after `physical_swing`,
  unpaced, was traced to the frame draw: consumers whose food was already
  surrounded each ran the near-food search to its candidate budget and
  failed, and the draw retried them every advance.

  Links: area Directing and play · draws on sim-execution D32, sim-execution D36 · governs cosmograph::spatial · summary A drawn member with no free stance near its food takes the ordinary search, and that food counts as crowded for the rest of the advance.

- **D40, a frame's draw stops at its first failure (2026-10-29), applies
  Directing 5:** "Stop at first failure (Recommended)". A frame's draw ends
  for the advance at its first member that finds no stance in the frame;
  the next advance tries again in a new draw order, so a frame on full
  ground fills over several advances. This replaces the cap of twice the
  shortfall. Asked after D39 was measured: the frame's site held about half
  the members its share asked for, and the members that found no stance
  anywhere were searched for again every advance.

  Links: area Directing and play · draws on sim-execution D32, sim-execution D39 · governs cosmograph::spatial · summary A frame's draw stops for the advance at its first member with no stance in the frame.

- **D41, a paced Grouped span runs a tick at a time (2026-10-30), applies
  Processes 7:** "Regroup every tick (Recommended)". A paced cohort
  chooses by its first identity (E8) and Grouped cohorts regroup at an
  advance's end, so the same ticks advanced singly or in one span grouped
  members differently and chose on different ticks; replay advances in
  spans where nothing was recorded, so such a world's saves did not replay
  (609, 610). A Grouped advance with a paced lineage now runs one tick at a
  time, regrouping at each boundary; E8's offset and D15's spread stay.
  Asked after the runtime triage traced a paced world's failed replay to it
  (Findings, 2026-10-30), carrying out D43's "Fix replay".

  Links: area Processes and rules · draws on 609, 610, sim-execution D15, sim-execution D19, sim-execution D43 · governs cosmograph::schedule · summary A Grouped advance with a paced lineage runs one tick at a time, so a span groups and chooses as its ticks taken singly.

- **D42, the record is a store per component, keyed by identity
  (2026-10-31), applies 483:** Mark, on the recommendation below: "an
  interesting direction, go for it". The record keeps its identities
  (`Population`'s intervals, never reused) and each component of `Entity`
  moves, one at a time, into a store chosen for its shape, after the
  per-component stores of Haskell's apecs: an interval store for what a
  cohort shares, paged columns behind `Arc` for fixed-shape state every pass
  reads, an ordered map for sparse state, and a single value for the world
  and the played critter. Data-defined accounts are interned per world and
  each held as a column. Every store lists its members in identity order,
  so a process is a merge of stores in that order, saves and hashes do not
  depend on storage history, and Processes 7 and E2's keys hold unchanged;
  the scheduler stays the sim's (482). No ECS crate holds the record: hecs,
  bevy_ecs and shipyard allocate and reuse their own identities, iterate in
  storage order and have no member standing for many, so each would need a
  table from our identities to theirs, a second answer to who a member is
  (483). hecs stays where it fits, as storage for conatus's near-tier
  runtime, rebuilt from the record and never saved. The stores start as a
  module of `cosmograph` and move into the stack when a second consumer
  needs them (600).

  Links: area The sim's schema and saves · draws on 482, 483, 600, sim-execution D27, sim-execution D28, sim-execution D29 · governs cosmograph::population · summary The record holds each component in a store suited to its shape, keyed and listed by the sim's own identities; no ECS crate holds it.

- **D43, the runtime suite's paced fixtures (2026-10-28), applies Method
  4:** "Fix replay, unpace where allowed". D19's pacing broke 30 of
  `mesocosm-runtime`'s tests, whose fixtures expected a choice on their
  first tick, and under them a paced world's saves did not replay. The
  replay is fixed (D41). Fixtures that need a first-tick choice run
  unpaced, as D19's own choice tests do: `held()`, the foraging fixtures
  and `physical_swing`. The offer-gate
  tests stay paced, the played 879 witness running ten ticks to the
  critter's first choice. A resume replays its history in the seal's
  execution, and paced, a Grouped cohort chooses on its first member's
  phase and an individual on its own, so a seal tampered to the other
  execution now refuses at the replay, before the seals are compared.
  Asked in the runtime triage (Findings, 2026-10-28). After the suite was
  set aside as a work item (2026-10-30), Mark, 2026-10-31: "How long would
  it take? Can you do it now to just settle the matter?", then "Groups 1–3
  now (Recommended)".

  Links: area Method and documents · draws on 609, 610, sim-execution D18, sim-execution D19 · summary Test fixtures that need a first-tick choice run unpaced; the offer-gate tests stay paced, the played witness run to the critter's first choice.

- **D44, the trace pin leaves out its hash while the sim is wired
  (2026-10-28), applies Method 4:** "Drop the hash while wiring".
  `runtime_pin`'s recorded traces compare actions, outcomes, ticks and
  entries; the final state hash is recorded but not compared, and comes
  back when validation of the whole is designed (D18). Asked when the trace
  failed on its hash alone from E7a (`21838c0d`); settled with D43.

  Links: area Method and documents · draws on sim-execution D18 · summary The runtime's recorded traces compare actions, outcomes, ticks and entries, not the final hash, while the sim is wired.

- **D45, `physical_play` walks on version four (2026-10-28), applies
  Method 4:** "Move it to version 4". Founding opts into weight-transfer
  walking, which `Templates::validate` refuses below version four
  (`699f8f61`, the foot placement plan's weight-shifting follow-up), and
  the suite forced version two to keep its version-two locomotion
  controls. It now runs on the generated version-four rules with
  undirected walks. Asked in the runtime triage; settled with D43.

  Links: area Method and documents · draws on sim-execution D18 · summary physical_play runs on the generated version-four walking rules instead of forcing version two.

- **D46, generated consumers sense (2026-10-28), refines World 8:** "Fix
  founding (Recommended)", keyed 2026-10-31 as "Refine World 8, bodies owes
  it (Recommended)". Generated consumer founders carry living sense
  tissue, so an ordinary played critter can detect remote food. Asked when
  `foraging_play`'s ordinary A1 test was traced to `edbb3b95`, which gave
  the control fixture's actor a sense cell but not the generator's
  founders; the test stays as written. Owed by `bodies` BO18.

  Links: area World generation · plans bodies · summary Generated consumer founders carry living sense tissue, so an ordinary played critter can detect remote food.

- **D47, the hungry starter is nudged to travel (2026-11-01), applies
  Method 4:** "Nudge it to travel (Recommended)", then "Raise the
  fixture's sway (Recommended)". `physical_swing`'s hungry-starter test
  checked that its critter, unchosen, travels by tick 8 and that its old
  site's pose withdraws. Under case 1057 its food stayed in reach and it
  ate instead of leaving (Findings, 2026-11-01). After its food step the
  test now nudges it toward a neighbouring site. A nudge's sway at a fresh
  bond was 250 against a meal's 1110, so the fixture's sway is 5000 (a
  nudge of 1250); without a live nudge sway does nothing. Asked when the
  test failed after the 1057 merge.

  Links: area Method and documents · draws on 1057, sim-execution D18, sim-execution D43 · summary physical_swing's hungry starter is nudged to travel at a sway that outweighs a meal, rather than left to choose travel on its seed.

- **D48, fixed-point sunlight is the default for new generated worlds
  (2026-11-15):** "Accept it; re-record the five values (Recommended)".
  Work left uncommitted on 2026-11-12 made every new generated world record
  the fixed-point sunlight credit rule, so native and WebAssembly agree;
  preexisting saves keep the legacy rule through the absent field. It had no
  ruling and had not been run against the genesis digest pin
  (`directing/interim/reproduction.rs`) or the exact-replay hashes
  (`tests/exact_memo.rs`), which it moves: five values, re-recorded under
  this decision. The exact-replay test passes at the lineage-variation tip
  without the sunlight work, so the move is that work's alone; Grouped and
  Individuals still agree. Offered against keeping the legacy rule as the
  default and against taking the work off main.

  Links: area Processes and rules · draws on sim-execution D16 · plans sim-execution · summary New generated worlds credit sunlight by the fixed-point rule; the genesis digest and exact-replay pins were re-recorded for it.

- **D49, parts stay the truth and gain stored hashes, per part (2026-11-16):**
  "Parts stay truth; add stored hashes now; tree stays derived
  (Recommended)", then "Per part (Recommended)", then "Plan first, then a
  lane (Recommended)". A scratch measurement on generated populations
  (`Code/scratch/nested-voxel-measure`, `pop_run1.md` to `pop_run3.md`;
  the body geometry brief has the tables) found that rehashing every body's
  document, as the witness does, costs about 3.7 ms a tick for 150 plants,
  that a remembered hash per body is about 40x faster at 1 percent of
  bodies changing and a hash per part about 100x, and that a nested voxel
  tree with stored hashes adds an edge only when about 10 percent change.
  Those bodies and document sizes are estimates, not the simulator's. The
  state hash changes once, so its pins are re-recorded once. Phase E11.

  Links: area Processes and rules · draws on 633, 649, 652 · plans sim-execution · summary The part tree stays the simulator's truth and gains remembered per-part hashes; the nested voxel tree stays derived. Not yet built.

- **D50, the shared stepper lives in cosmograph-host (2026-11-16):** "In cosmograph-host, beside the pacing (Recommended)". The code that walks a foreground body along a route with strides and builds its handback sits beside `drive/round.rs`, calling the simulator's route and foot code, for both games. Eponym steps only the steered player and has no path or foot code; conatus has neither. Path and feet may move to conatus later, as case 1084 intends. Offered against conatus now and a new shared crate.

  Links: area Stack and ownership · draws on 1084 · plans sim-execution · summary The foreground stepper (path, strides, handback) lives in cosmograph-host beside round pacing, shared by both games. Not yet built.

- **D51, headless play runs the same stepper (2026-11-16):** "Run the same stepper without a renderer (Recommended)". With no window, the foreground set is still stepped sixty times a world minute and handed back, so headless and windowed runs reach the same hashes. Offered against everyone travelling and against a handback straight to the goal.

  Links: area Processes and rules · draws on 1080, 1081 · plans sim-execution · summary Headless runs step the foreground set and hand back as a window would. Not yet built.

- **D52, a world rule says the foreground walks in-view bodies; an accepted handback clears the journey (2026-11-16):** "A world rule; an accepted handback clears the journey (Recommended)". A world without the rule behaves as today, so older saves keep their bytes and hashes. Found by reading: `spatial/handback.rs` never touches a body's pending travel, and `arrive_travellers` would land the body again at its due tick.

  Links: area Processes and rules · draws on 1080, 1084 · plans sim-execution · summary A world rule marks foreground-walked worlds; accepting a handback ends that body's pending journey. Refined by D57.

- **D53, the baseline is measured on today's main first (2026-11-16):** "Measure today's main first (Recommended)". Before E10's remaining steps are built, one release run counts route searches, cache hits and misses, which bodies were in view and the cost of each search. A search cost about 98 ms on 2026-11-02 and 3.0 to 4.3 s on 2026-11-12 for the same command and seed. With "A new bar after the measurement", the speed bar is set from this and from E10's re-measurement.

  Links: area Method and documents · draws on 976, 978 · plans sim-execution · summary Today's played step is measured (searches, cache hits, in-view bodies, cost per search) before E10's remaining steps are built.

- **D54, a remembered route search is valid against the bodies it consulted (2026-11-16):** "Only the bodies the search actually consulted (Recommended)". Today the route cache's key holds every other posed body's boxes in the window (`spatial/foraging/route/cache.rs`), so a standing body's search is redone whenever anyone moves: 22.1 s of 32.6 s in the measured run, 18 of 22 repeats reproducing the earlier result. A search is to record what of the other bodies its step checks read and stay valid until that changes. Outcomes and hashes must not move. Offered against ignoring other bodies and rechecking the path, and against leaving it.

  Links: area Processes and rules · draws on sim-execution D53 · plans sim-execution · summary The route cache keys on what a search read of other bodies, not on every body's boxes; outcomes unchanged. Not yet built.

- **D55, the step check is made cheaper before any limit on refusals (2026-11-16):** "Make the check cheaper first; outcomes unchanged (Recommended)". A refused step costs about 18 ms and restarts the search, with no limit on restarts. The untimed half of the check is profiled and repeated work cut without changing what the check accepts. Stopping at the first refusal took 20 steps from 36 s to 3.4 s and changed the outcome (148 living at tick 20 for 150), so a limit on refusals stays unruled.

  Links: area Processes and rules · draws on sim-execution D53 · plans sim-execution · summary The full-body step check is profiled and made cheaper with outcomes unchanged; no refusal limit is ruled. Not yet built.

- **D56, a body that found no route retries as now (2026-11-16):** "Leave retries as they are (Recommended)". With D54, a retry that would give the same answer costs almost nothing. Offered against a back-off after failure.

  Links: area Processes and rules · draws on sim-execution D54 · plans sim-execution · summary No back-off after a failed route search; D54's cache makes an unchanged retry cheap.

- **D57, the world rule field lands now, before its first reader (2026-11-18):** "Keep it now (Recommended)". Refines D52. The lane found by test that an accepted handback already ends the body's pending journey in every world (publishing the pose writes the body afresh with no travel), so D52's reading of `spatial/handback.rs` was wrong in effect and the rule clears nothing. The field (`Rules::foreground_walks`, absent unless true) does nothing until step 4, where it means the sim does not search routes or plan walking for in-view bodies. Offered against adding it in step 4 and against a choice of named modes.

  Links: area Processes and rules · draws on sim-execution D52 · plans sim-execution · summary The world rule is a field absent unless true, landed inert; an accepted handback ends the journey in every world; step 4 gives the rule its meaning.

- **D58, the generated-driver path carries handbacks through one added entry (2026-11-18):** "Keep it (Recommended)". Mesocosm advances a round two ways and the generated-driver way had no way to carry handbacks. Cosmograph gains `try_advance_round_spatial_with_fallback`; the older function calls it with none. Offered against the runtime refusing handbacks under a generated driver.

  Links: area Processes and rules · draws on sim-execution D52 · plans sim-execution · summary Both of Mesocosm's round paths carry handbacks; Cosmograph gains one additive public entry.

- **D59, a foreground body walks at its own speed (2026-11-18):** "At the body's speed (Recommended)". The stepper walks a body along its route at the body's speed and stops at the meal or the route's end. Measured over 10 seeds, one minute each: stepped bodies moved a median 3.0 cells (mean 4.2), natively walked ones 1.0 (mean 1.8, none in 5 of 10), because in-tick walking publishes only a route's first edge (`foraging/route.rs`, `path.take(2)`) while the speed allows about 2,970 cells a minute at the 1 mm cell and the handback's limit (`within_reach`) is that speed. Offered against one edge a minute and against a separate cap on reach. This answers the plan's unruled reach in sixty world seconds.

  Links: area Processes and rules · draws on 1080, 1084, sim-execution D50 · plans sim-execution · summary In view a body covers what its speed allows in a minute, the same limit the handback and out-of-view travel use; not one edge a minute.

- **D60, the stepper is checked against out-of-view travel (2026-11-18):** "Out-of-view travel, over several minutes (Recommended)". Replaces step 3's comparison with native displacement, which D59 makes meaningless. A stepped body and a body travelling out of view by route travel time should cover the same ground at the same speed: compared over several consecutive minutes, medians within one cell. The first attempt at this control showed no movement and is diagnosed first. Offered against accepted-and-within-reach only.

  Links: area Method and documents · draws on sim-execution D59 · plans sim-execution · summary Step 3 is done when stepped bodies and out-of-view travellers agree over several minutes, medians within one cell. Replaced by D63; retired by D64.

- **D61, a body with no route or a blocked one stays where the sim has it (2026-11-18):** "Keep it (Recommended)". The stepper takes the route once at the start of the minute; with none it sends no handback, and a handback the sim's check refuses leaves the body as it was, to try again next minute. Offered against stopping at the last clear stance and against searching again inside the minute.

  Links: area Processes and rules · draws on sim-execution D50 · plans sim-execution · summary No route, no handback; a refused handback leaves the body in place; no search inside the minute.

- **D62, the sim's place-grain choice picks an in-view body's meal (2026-11-18):** "The sim's place-grain choice (Recommended)". The stepper asks the sim for the target a traveller would get (`travel_target`: nearest by place route) and finds the walked route to it, so a body heads for the same meal watched or not, as case 1084 says. As first built the stepper took the nearest meal by walked route (`food_route`), and in 3 of 9 measured bodies that was a different meal from the traveller's. Cosmograph gains one narrow public entry for the choice. Offered against nearest by walked route and against changing the out-of-view choice.

  Links: area Processes and rules · draws on 1084, sim-execution D50 · plans sim-execution · summary In view and out of view a body heads for the same meal, chosen by the sim at place grain; the stepper only finds the path.

- **D63, the stepper is checked on arrival, not distance (2026-11-18):** "Arrival, not distance (Recommended)". Replaces D60's medians within one cell. With the same meal, a stepped body stops where its route first touches the meal and a traveller is placed by the sim at a free stance touching it (`habitat.rs` `arrive_near`), which can be far round it: seed 27, a 2-edge route, 2.0 cells stepped against 59.5 travelled; over 10 seeds and 4 minutes the medians were 3.0 and 13.6. So the check is that both head for the same meal, both end touching it, and the minute they arrive agrees within one. Where they stand may differ. The 59-cell placement is not traced. Offered against the same end stance and against tracing the placement first.

  Links: area Method and documents · draws on sim-execution D60, sim-execution D62 · plans sim-execution · summary Step 3 is done when stepped bodies and travellers reach the same meal, touching it, within one minute of each other. Retired by D64.

- **D64, watching is checked on a whole run, not body by body (2026-11-19):** "Go ahead." Mark, on the comparisons of D60 and D63: "This feels to me like we're chasing the edgiest of cases that might not matter, big picture", and "Prior to more systems being implemented, is agreement achievable or desirable at all? What would it mean, practically?" Agreement means a creature's fate does not depend on whether someone is looking at it. Case 1084 gives two movement models on purpose, so a watched and an unwatched body never share a cell; the outcomes that matter (meals, reserves, survival) wait on systems not yet built or tuned; and the out-of-view side has untraced behaviour of its own. D60 and D63 are retired as gates. The stepper lands as built, with its handbacks accepted on 10 of 10 seeds. D62 stands and is built in step 4. After step 4, one run of the same world with the window open and closed compares the living curve, within seed-to-seed noise.

  Links: area Method and documents · draws on 1084, sim-execution D60, sim-execution D63 · plans sim-execution · summary No body-by-body comparison of watched and unwatched movement; one whole-run comparison of the living curve after step 4.

- **D65, the whole-run check passes (2026-11-19):** "Close enough, go on (Recommended)". D64's check, living count at tick 20 with the rule off then on, six seeds: 150 to 148, 107 to 106, 78 to 76, 128 to 130, 138 to 138, and seed 50, 93 to 77. Mean difference -3.2; different worlds range 78 to 150 with the rule off. Seed 50 is the outlier and is not traced. Offered against tracing seed 50 first and against more seeds first.

  Links: area Method and documents · draws on sim-execution D64 · plans sim-execution · summary Watched and unwatched runs of one world agree closely enough to go on; seed 50 (93 to 77) is noted, untraced.

- **D66, a body in view with nothing to walk to stands still (2026-11-19):** "Stands still (Recommended)". Under the rule the sim plans nothing for a body in view, so one with no meal to head for, or whose stepped minute the sim refuses, sends no handback and stays where the sim has it until the next minute. Carries D61 into the rule. Offered against the sim's own one-edge walking as a fallback.

  Links: area Processes and rules · draws on sim-execution D61, sim-execution D57 · plans sim-execution · summary Under the rule an in-view body the stepper gives nothing for stands still that minute; no native fallback.

- **D67, Cosmograph gains two entries for the stepper, not one (2026-11-19):** "Keep both (Recommended)". Refines D62. `Simulation::meal_target` is the place-grain choice and `Simulation::route_to_meal` the walked route to it, since the sim's route search is private. Offered against one entry returning the route to the chosen meal.

  Links: area Processes and rules · draws on sim-execution D62 · plans sim-execution · summary The meal choice and the route to it are two public entries; the stepper calls one then the other.

- **D68, the rule is founded by `--foreground-walks` (2026-11-19):** "Keep `--foreground-walks` (Recommended)". A command-line flag named for the rule field turns the rule on for a founded world; no default, fixture or save sets it. Offered against a test-only switch.

  Links: area Processes and rules · draws on sim-execution D57 · plans sim-execution · summary `--foreground-walks` founds a world with the rule; nothing sets it by default.

- **D69, step 7 lands with walking not yet visible (2026-11-19):** "Land it as built (Recommended)". The window draws the stepper's poses and a windowed round replays to the headless hash, but in ten captured minutes there were 0 or 1 walks a minute, each 1 to 2 cells, finished in the first of sixty steps at about 49 cells a step. Visible walking waits on bodies having longer journeys, not on more window work. Offered against proving the drawing on a long route first and against spreading a walk over the minute.

  Links: area Method and documents · draws on sim-execution D59, sim-execution D50 · plans sim-execution · summary Step 7's mechanism is on main; a capture of walking waits on longer journeys.

- **D70, the window attaches the foreground set before it first draws (2026-11-19):** "Attach before first draw (Recommended)". In the first minute the window drew 1 stepped body and the clock handed back 4, because the round's first step attaches the set; 3 bodies jumped at tick 1. Offered against not drawing the first minute and against leaving it. Not yet built.

  Links: area Processes and rules · draws on sim-execution D50 · plans sim-execution · summary Window and clock see the same foreground bodies from the first minute.

- **D71, no cap on a foreground minute for now (2026-11-19):** "No cap for now (Recommended)". Case 1082's measurement over 20 minutes, 3 runs: median 3.9 ms, 17 of 20 under 40 ms, minute 18 about 570 ms, the first minute 5.6 to 6.2 s from one heavy search. Ticks run on the runtime's worker, so a slow minute holds stepped bodies at their last pose and does not freeze the window. Taken up again with the heavy first-time searches. Offered against a node budget per body per minute and against searching a minute ahead.

  Links: area Processes and rules · draws on 1082 · plans sim-execution · summary A foreground minute has no cap yet; one 6 s first minute is absorbed by the tick worker.

- **D72, a rule world's window runs at one world minute a real minute (2026-11-19):** "Keep 1x, dev ladder only (Recommended)". No speed flag; the dev ladder (0.25x to 4x) is the only control. A nudge applies on the next round, so it can wait up to a minute; speed control and that wait are taken up when the rule becomes the default way to play. Offered against a speed setting now and against fixing the input wait first.

  Links: area Processes and rules · draws on 1080, sim-execution D51 · plans sim-execution · summary Rule worlds play at 1x in the window with the dev ladder only; input can wait a round.

- **D73, the tick bar is a 0.5 s guard with 0.1 s the aim (2026-11-20):** "0.5 s guard, 0.1 s aim (Recommended)". Mark first asked: "is more speed work plausible, or is systems work required before efficiency?" Both: the known costs can be cut now (one cold search is 99% of stepping time, the window and the clock each run a body's search, `spatial.prepare` is about 0.15 s a tick), but longer journeys will put cost where the tick is already heaviest, so tuning to 0.1 s now tunes a world about to change. A played tick with the rule on stays under 0.5 s as systems go in, and a phase that breaks it stops for speed work; 0.1 s (TD2) stays the aim, not a gate. Measured 2026-11-19 in release at 67 to about 150 bodies: 0.6 to 0.7 s a tick with the rule off, about 0.3 to 0.4 s with it on. This is the "new bar after the measurement" of D53. Offered against speed work first to 0.1 s and against no bar until journeys exist.

  Links: area Method and documents · draws on 978, sim-execution D53 · plans sim-execution · summary A played tick under the rule stays under 0.5 s; breaking it stops a phase for speed work; 0.1 s is the aim.

- **D74, the bar is stated by tier (2026-11-20):** "By tier (the third option)", after "we can measure 3". The bar covers the near tier, the bodies in view and their roots; the rest of the world must cost by sites and lineages, not by members (D24). The two are measured apart. Offered against the founded world at tick 20 and against a few hundred members.

  Links: area Method and documents · draws on sim-execution D24, sim-execution D73 · plans sim-execution · summary The tick bar is the near tier's; the far tier is judged by how its cost grows, and each is measured on its own.

- **D75, cases 966 and 976 retire as bars (2026-11-20):** "Both retire (Recommended)". Both were written for the seed-9 save at the old tempo of many ticks a second. D73 replaces them. Offered against keeping 976 as a floor and against leaving both.

  Links: area Method and documents · draws on 966, 976, sim-execution D73 · plans sim-execution · summary 966's 1.5 s and 976's 8 s are no longer bars.

- **D76, the first minute has its own startup allowance (2026-11-20):** "Its own startup allowance (Recommended)". A session's first minute costs 5.6 to 6.2 s from one cold route search against a 3.9 ms median foreground minute. The tick bar applies from the second minute; the first is judged with the heavy first-time searches, still unruled. Offered against the same bar.

  Links: area Method and documents · draws on sim-execution D71, sim-execution D73 · plans sim-execution · summary The tick bar starts at the second minute; the cold first minute is judged separately.

## Constraints

- **Three tiers, and which ones the view may move (D23).** Mark: "I think the near tier is necessarily dependent on the viewing dimension, but the far tier, well, there's like a far tier, like in the way that people use billboarding, where like if something is really, really, really far away, then they'll just billboard it, you know? And then there's the far tier of like, it's not in this area. You could not possibly see it. Then that's like aggregate tier, right?" So: *near* is inside the rendered area and runs as individuals; it depends on the view, and the viewing dimension may change it. *Far within the rendered area* is presentation, the billboard end of `orthographic-voxel-presentation` L10, and reads the same individuals more coarsely. *Aggregate* is everything outside the rendered area, cohorts and crowds grouped to save calculation; it never depends on the view, so E4 and E5's seeded lift and the tier line's crossings key on site, lineage and tick, never on a view event. From: the 2026-10-27 discussion and Mark's answer.
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

Which governs, from 2026-11-20: a played tick under the foreground rule stays under 0.5 s, with 0.1 s the aim (D73), stated for the near tier with the far tier judged by how its cost grows (D74), from a session's second minute (D76). Cases 966 and 976 are retired as bars (D75); 978's head tempo stands.

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

### E4. The population model
Outside the rendered area each site and lineage runs as a population (D24), in bands of age and felt needs (D29), each a representative body with a count and a matter pool (D28), run by the scheduler with multiplicity (D27): counts by age band and condition (fed, hungry, wounded), its matter, and births, deaths, meals, upkeep and moves as rates read from the same process definitions (219), matter conserved every tick. The probe's exact-state crowd (`probe/crowd/`) is not the form: played worlds hold about 1.2 members per exact body state. Identity-bound processes and noted members stay individual (113, D2).
- Done when: a far site and lineage advances as a population, conserving matter, in a unit test (D18).
- Needs: D23, D24
- 2026-11-08, case 1163: the population model is the one sim's cohort form (counts by stage and condition, the heritable trait spread, pooled matter), advanced over a span by the mass law and mass-derived encounter terms, per the [coarse-step brief](2026-11-05_coarse_step_brief.md) §6; it waits on the mass law and the two-year bar (1166).
- Rulings: 75, 113, 207, 219, 261, 266, 440, 1163, 1166, D2, D23, D24

### E5. Crossings and the tier line
Wire the tier line into directing (742): near is the rendered area plus the attention roots and those they care for (D25); a member keeps its tier until out of its area for a band of ticks, a world rule (D26). Lifting draws a member's body from the lineage's recipe and recorded variation, its condition from the population's counts (435); folding back adds it to the counts, conserving matter; noted members never fold. Keyed on site, lineage and tick, never on a view event (D23). `TierLine` (`directing/tier.rs`) is hop-based and unwired today.
- Done when: a member lifted and folded back conserves matter and keeps its identity in a unit test (D18).
- Needs: E4; D25, D26
- Rulings: 113, 293, 431, 435, 554, 742, D23, D25, D26
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:139-141`, `:360-367`.

### E6. Rate models
Calibration per rules digest (440), for shared resources where the crowd stops paying (261, 266); uncalibrated worlds stay on the exact crowd. The hybrid background (exact crowd where grouping pays, a certified rate model below the members-per-state bound, every switch receipted) has no rate model today: with bodies the crowd groups only slightly more than one member per state, so its evaluation savings are small. A century at a minute-tick is far past the default advance limit; evaluation, not ticks, is the budget (284).
- 2026-11-08, case 1163 amends 440: nothing is fitted per rules digest. Interaction terms are derived from the processes' parameters after the field's models; a term is fitted per web shape, on the bench, only where 1165's checks fail. Certification is 1165's.
- Done when: the cohort form passes 1165's checks against the unwatched fine sim; a term that fails is fitted per web shape and passes them; worlds whose form is uncertified stay exact.
- Needs: E4; the bench (W4); `sim` S5 for the region run it serves
- Rulings: 207, 261, 266, 284, 439, 440, 441, 1163, 1165
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:142-143`; `archive_docs/2026-10-28/2026-09-22_sim_plan.md:727-745`, `:1713-1717`, `:1908-1912`.

### E7. Rates worked out when read
E7a: `mind:strain` and `mind:relief` (`rules/mind.rs`, both every tick for every member) become one rated account: a value, the tick it was settled and a rate, building while mood is low and bleeding while not, never below nought; reads settle it, a commit that changes what mood reads re-derives the rate, the state preimage settles first, and a crossing of the bearing is a due event. Mood must change only at a commit: where a world's needs read anything but the actor's body, strain stays on its processes (Mesocosm's played needs read only the body, `directing/found.rs`). E7b: `ecology:hazard-{i}`'s risk draw each tick from maturity becomes one draw of the tick of death at maturity, at the same per-tick odds and keyed by actor, tick and process (D1), kept as a due event; age and starvation wake at their ticks (`ecology.rs`). Upkeep stays a process: its matter reaches a site's soil, which producers read. Removing strain's acts moves the act counter, so bite and amount draws move until E2.
- Done when: strain and death run without their per-tick passes and their own tests pass (D18).
- Status, 2026-11-08 (case 1181): E7b built as first planned. A rising hazard whose only outcome is death, run every tick, is drawn once (`death::Drawn`). The draw is keyed by actor, tick and process (D1) and made when the actor first comes under the hazard: at maturity, or at the world's first tick for a founder already mature. Survival at the per-tick odds is worked out a year of age at a time in integer fixed point (`Aging::falls_after`). The index files the actor for the hazard only from that tick, so the act is a due event: the hazard falls on the drawn tick and on any later tick it is still due. The odds read only age and rules, which never change in a world, so no redraw is ever due. A hazard whose gates could open and close again keeps its draw every tick: an account, a site, a trait some process sets, or a world that bands its far members. No world-time hazard is of that kind. In distribution, 20,000 replicates per world-time role match the per-tick draw: Kolmogorov-Smirnov distance 0.006 to 0.008, every reading certified within 50 per mille and none detected, both families Holm-corrected at 5%. The doubled-odds control fails, with distance about 0.25 and every reading detected (`tests/death_drawn.rs`). The bar's two-month fixture runs in 16.7 s where it ran in 28.5 s. Its new hash is `5242331604158281548`, alike in both modes. Earlier hashes and trajectories moved, as case 1181 accepted. The comparator and the bar were rerun on the new draws on 2026-11-08 (sim plan S10.3): every seed fails both, and seed 9, which passed the comparator with the per-tick draw on the same food code, collapses at month 21. Hazard deaths there were 226 by month 20 against 214 before, so the draws moved the trajectory, not the death rate. Two-year runs took 776 to 1,341 s on a shared machine.
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
A matrix of world size against population with tick cost, hash cost, memory and residency, every number reproducible from a cargo run; the `cosmograph-scale` remeasure is its first native instance. The aggregate form and cohort execution it once covered are E4 to E6; zoom is the presentation plan's L10.
- Done when: "every number reproducible from a cargo run"; no other threshold stated
- Needs: -
- Rulings: 793
- From: `archive_docs/2026-10-28/2026-09-22_sim_plan.md:1393`.

### EX4. Scheduler waste and remaining scheduling cost
After the narrowed prey scan, shared-ground passes (`produce-3`, `feed-1`) evaluate each member before acting it, deliberation for members who then chose another process, and accepted acts (`mind:strain` the largest part) remain the cost with no phase; D12 leaves shared-ground acts both evaluations. Where a world holds owned projects, `begin_process_debits` digests the whole state for every act (A1 holds none). Of the older findings only one is live: the generator's competitions are stored by id with only one allowed (the probe refuses worlds with more than one; how several share a round is undesigned).
- 2026-11-08, case 1174: profiled and attacked first, before the two-year bar's long runs; the state hash must not move.
- Status, 2026-11-08: profiled and cut on the bar's founding, every outcome unchanged. The fixture is `tests/world_time_year.rs` at seed 9 with `WORLD_TIME_MONTHS=2` (`WORLD_TIME_EXECUTION` picks the mode): terminal state hash `1988914608126411875`, with the same history digest and monthly samples in Grouped and Individuals before and after every change. `COSMOGRAPH_PROFILE_ACTIVE=sum` totals the spans and each process's evaluations and acceptances in memory. Of 230 s, two costs dominated, neither named above: a site condition shift (`world:weather`, every three ticks at four sites) re-filed every resident against every gate, and the rising hazards' 21.2 million evaluations each staged a whole act to miss its draw. No `mind:strain` pass ran in this world. Shared-ground plans and deliberation were under a sixth. Five exact changes landed. Residents are re-filed only when a condition some gate reads moves (`Filed::reads_shift`). An act reads its definition from the shared genesis rather than copying it. A risk-only act draws before its other checks, and a scheduled one that misses is not staged at all (`Simulation::risk_misses`). An agentless pass goes past groups that are not world bodies. The fixture fell from 230 to 28.5 s Grouped and from 276 to 28 s Individuals (Findings, receipts). `begin_process_debits` is untouched: the bar's world holds no owned projects. What remains is the hazard draws (about a third), shared-ground plans with deliberation (about a third) and accepted acts. Cutting the draws needs E7b as first planned, which moves draws and the hash; that is a question for Mark. Case 1181 answered it (E7).
- Done when: not stated; owed
- Needs: D8 to D12; anything changing a choice or shared ground returns as a question (constraint)
- Rulings: 206, 220, 306, 454, 683, 813, 1174
- From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:236-244`, `:394-396`, `:408-428`; `archive_docs/2026-10-28/2026-09-22_sim_plan.md:1658-1659`, `:1712-1717`.

### E9. Stores per component
**Boundary, 2026-11-04:** the [scene composition experiment](2026-11-04_scene_composition_plan.md)
adds native-identity `ReadStore`, `SparseStore` and ordered joins in
Cosmograph, used by an optional derived scene projection. It does not move
authoritative `Entity` fields, bands, admission or saves into component
stores. E9 remains open; the measured capture/materialization adapter is
not the hot-record or packed-store comparison described here.

**Product boundary, 2026-11-12:** `cosmograph-overlay` defines the exchanged
values. `cosmograph-host::World` owns the live session but currently lends
the native `Session` to product readers. For E9, put product-facing population
and ledger reads behind host methods that return handles, counts and the
facts needed by the existing views. Preserve window disclosure and native
ordering. The storage change qualifies once it builds and runs through the
three products without edits to their production callers or save witnesses.

D42's layout, built with the bands (E4), which already want columns. A
store trait (get, set, remove, members in identity order) and four stores:
interval, paged column, ordered map, single. Splitting a cohort splits its
interval in every store, as `lift_shared` splits it today; `restrict`'s
merge compares components store by store. Fields leave `Entity` one at a
time, first those every pass reads (alive, place, settled, the accounts
metabolism, strain and meals touch); mind state, if cohorts grow into
individual minds, is the next heavy user. The save format takes a version
with a conversion from the last, as earlier versions did. The journal's
rollback check (`journal.rs:66`) gets a form that does not clone the world.
- Done when: a world whose hot fields live in stores advances, saves and
  reloads to the same state in a unit test (D18).
- Needs: D42; E4 alongside; the spatial costs of the 2026-10-31 finding gone first
- Rulings: 482, 483, 600, D42

### E10. Rounds: the sixty-step foreground and movement by grain
Boundary with Presentation 2 (case 1161): L10's near-live/far-cached choice and
cached-pose refresh are image settings. Current roots/depth still follow the
foreground's authoritative motion; image refresh does not choose E8 decision
periods, E10 stepping frequency, physical grain or handback rules. Presentation
acceptance compares the same input/outcome traces at different image settings;
its frame costs stay separate from this plan's simulation speed bars.

A round runs world time at 1, 2, 4 or 8 times real time (case 1080). Bodies
in the foreground step sixty times a real second in the product, and the sim
records only each minute's handback, checked by its invariants. The sim
chooses at place grain; in view the foreground finds the path, and out of
view a body arrives after its route's travel time (case 1084). Route search,
walking plans and settling leave the tick. The tick runs off the frame thread
by default, or in step as a setting (case 1081). The foreground cap is a
world rule whose default is set from a measured step (case 1082). In order:
the handback contract and the foreground set (cosmograph); round pacing
(`cosmograph-host`); arrival by travel time out of view; Mesocosm's
foreground layer; Eponym's motion loop handing back through the same
contract.
- Done when: a round at each speed replays its save to the recorded hash; a
  refused handback leaves the source unchanged; foreground and native
  displacement agree in distribution; a played tick no longer runs route
  search or walking plans, re-measured against the 2026-10-31 finding; a
  Mesocosm round renders smooth walking (capture); a foreground step is
  timed and its cap brought to Mark.
- **Where it stands, 2026-11-16.** Built: the handback contract, the
  foreground set, round pacing and arrival by travel time (Progress,
  2026-11-02). Eponym uses the contract (`eponym-play/src/world/rounds.rs`,
  `contract/mode/clock/driver.rs`). Mesocosm does not: nothing under
  `mesocosm/crates` names a foreground or a handback, so its in-view bodies
  still run route search and walking plans inside the tick. Mark, asked what
  comes first on the played step: "Continue E10's remaining steps"; and on
  which speed bar governs: "A new bar after the measurement", taken as this
  phase's own re-measurement. Unexplained and not rerun: the 2026-11-12
  profile's 109 to 158 s for 20 steps against 14.0 s on 2026-11-02 for the
  same command and seed.
- **Remaining steps, 2026-11-16** (from a read-only assessment; D50 to
  D53). In view means in some participant's foreground, cut at
  `Limits::foreground` (16), so at most 16 bodies search for routes; walking
  plans are a search's inner work, about 70 a search.
  0. Baseline on today's main (D53). Done when the receipts give searches
     and cache hits a tick, the in-view bodies, and the cost of a search.
     **Measured 2026-11-16** ([receipts](archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md#2026-11-16-e10-step-0-baseline-on-main)):
     20 steps take 36 s, 90 percent of it 33 route searches. Twenty-one cost
     about 90 ms; nine cost 2.2 to 5.3 s and are three bodies repeating one
     many-trial search each, 30 s in all. The 2026-11-12 figure was this
     workload on a busy machine. The stepper of step 3 calls the same route
     code, so moving these searches to the foreground would not make them
     cheaper.
     **Diagnosed 2026-11-16** ([receipts](archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md#2026-11-16-e10-step-0-the-heavy-route-searches-diagnosed)):
     the time is the full-body step check on refused edges (about 18 ms
     each, hundreds a search), not the search; and a standing body's search
     is redone because the cache key holds every other body's boxes, which
     change when anyone moves. Repeats are 22.1 s of 32.6 s and 18 of 22
     reproduce the earlier result exactly.
  1. The world rule (D52). Done when an older Mesocosm save loads, replays
     and hashes to its stored value.
  2. `mesocosm-runtime` carries handbacks: passed to the clock
     (`spatial/driver.rs`), kept in the record, accepted in replay. Done
     when a round with a handback replays to its hash and a refused one
     leaves the source unchanged.
  3. The shared stepper in `cosmograph-host` (D50, D59, D61). **Built
     2026-11-18**: every stepped minute accepted on 10 of 10 seeds,
     deterministic. The comparison with unwatched bodies is a whole-run
     check after step 4 (D64).
  4. **Built 2026-11-19.** The sim stops route search and walking plans for in-view bodies under
     the rule (`foraging/food.rs`, `stage/staged/step.rs`); the journey
     already clears (D57). The stepper takes the sim's meal choice (D62).
     Done when a played tick shows none
     for foreground bodies, against step 0.
  5. **Built 2026-11-19.** Headless stepping (D51). Done when a round at 1, 2, 4 and 8 times
     replays to its hash with no window.
  6. **Measured 2026-11-19 (Findings); no cap for now (D71).** A foreground step with path and feet is timed and the cap brought to
     Mark (case 1082).
  7. **Built 2026-11-19; walking not yet visible (D69); the first minute's set built (D70).** The window drives the pacer and draws foreground poses. Done when a
     capture shows walking without a jump each minute.
  Reach in sixty world seconds: the body's speed (D59).
- Needs: PG7's Places (landed); conatus for paths and foot placement
- Rulings: 1080, 1081, 1082, 1083, 1084, D30

### E11. Stored hashes per part (D49)
Proposed 2026-11-16 from D49; no step started. Today `Simulation::witness`
puts the whole population in as one entry (`simulation/witness.rs:28`), so
`state_hash` encodes and hashes every body's document on every call.
Remembered per-part hashes make that proportional to what changed.

- **H0, where the time goes.** Before anything is built, measure in a real
  played run how often `state_hash`, `witness` and saving are called and
  what share of a tick and of a save they take. Outside tests the callers
  found on 2026-11-16 are benches, probes and replay checks, not the played
  tick, and the 2026-11-12 played-tick profile is dominated by food route
  search and walking plans. Done when the receipts hold those numbers for
  the A1 and world-time worlds at their founded sizes and at 10x.
- **H1, per-part hashes in the witness.** Each part's hash is remembered and
  dropped when the part changes; a body's hash is built from its parts' and
  the population entry from its bodies'. Done when a run's hash after N
  ticks equals the hash computed with every remembered hash discarded;
  Individuals and Grouped still agree; replay stays exact; the pins that
  move are listed and re-recorded once under D49; and H0's measurement,
  repeated, shows the gain.
- **H2, saves write unchanged bodies once.** Needs its own read of the save
  path (`history.rs`, `cosmograph-host/src/world.rs`) before it is specified.
  Done when consecutive saves share unchanged bodies, an old save still
  loads and verifies, and the save format carries its version.
- **H0 measured, 2026-11-16** ([receipts](archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md#2026-11-16-e11-h0-where-hashing-time-goes)).
  In ordinary play the hash is taken in about 1 step in 4 to 1 in 15 and is
  0.02 to 0.5 percent of a played step, which walking and route work
  dominate. It is 28 to 400 percent of a tick where a test or bench hashes
  every tick, 55 to 80 percent of a save, and about 1 percent of a load,
  which is replay from genesis. `witnessed()` clones the whole state on each
  call, a quarter of A1's hash cost. Real documents are about 1.2 to 1.4 KB
  a body, not the scratch probe's 27.8 KB, so that probe's absolute figures
  are wrong while its ratios stand. **Parked, 2026-11-16** (Mark: "Park E11; go after the played step (Recommended)"): H1 and H2 stay owed and unstarted.
- Needs: nothing outside this repository; the witness crate
  (`state-witness`, Eidetic) is used as it is.
- Rulings: D49; 633, 649, 652

## Deferred and parked
- **Playable-loop forager route search** (the `playable-loop` branch, unpushed at 2026-10-23): its cost is deliberation the A1 battery does not exercise and no battery measures it. Owned by `bodies`; `sim-execution` reads its cost once a battery exists. From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:461-472`.

## Questions for Mark
- **Which speed bar is the bar.** Ruled 2026-11-20: D73 to D76.
- **The `cargo test` profile.** The played seed 0 runs far slower under the `cargo test` profile than release, and with debug assertions off drops substantially. The likely source is named but "not strictly separated": the journal's full `State` clone verifying each rollback (`journal.rs:66`) and `passed_by`'s population rescan (`schedule/mod.rs:314`). No action decided. From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:350-355`.
- **Occurrence index in E2's key (conditional).** If the actor, tick and process key collides for a grouped cohort, E2 brings the fork to Mark. From: `archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:128-131`.
- **Where near-tier movement lives (raised 2026-10-30).** Ruled 2026-11-02 as case 1084, split by grain (doctrine Stack 12); the round's foreground is E10. As raised: Doctrine Stack 12 says the sim "never pathfinds an individual, checks line of sight or resolves a single blow"; rulings 841 and 842 say chosen native steps are "resolved by a game"; yet food-route search, Places derivation and walking plans run in the sim's tick (`shared/isocosm/src/spatial/`, `foraging`, `embodiment`), and that was most of the head's 12.5 s tick (D30). Options: keep it in the tick (one authority, but cost grows with members times path and the region of case 124 is out of reach); the game layer resolves it (Stack 12 and conatus's "Product profiles conduct. Conatus advances spatial state.", but the sim's choice loses its reachability read); or split by grain. Recommendation (2026-10-31, not ruled): split by grain. The sim chooses the goal at the place graph's grain (which place, which meal, whether a route exists, from `place-graph-engine` PG7's Places); the game and conatus find the path, pose the body and place the feet; the sim commits the arrival it accepts, which replay already records as a command. Route search, walking and foot placement move toward conatus (`bodies`).
- **The tier band's length (D26's "a world rule").** Options: a fixed number of ticks (simple, blind to pace); a multiple of the member's decision period (tied to D19, kinds differ); a span of play time (moves with the tempo). Recommendation (2026-10-31, not ruled): twice the member's decision period by default (20 ticks for producers, 4 for consumers, 10 for decomposers), so a member decides at least once on each side before it switches.
- **Threads against shards.** E2 (draws keyed by actor, tick and process) is cheap and unblocks both; E3 (parallel evaluation in a pass) pays where work is per individual but gives nothing on the web's one Worker, and much of that work may leave the tick (above); S6's shards (`sim` S6) serve a played region and the web as separate Workers, but the merge is undesigned. Recommendation (2026-10-31, not ruled): E2 now, with every lane keying new draws that way; E3 after movement's home is settled; S6 when a played world spans a region. Thread count never changes a result (Processes 7); default all cores natively, one on the web.

## Findings

- 2026-11-20: where a rule-on round's time goes (release, main db88e205, `--play --curve 20 --seed 18181373014846284518 --foreground-walks`, both profiles on; controls 150 living rule off, 148 on). The machine ran about 2.5x slower than on 2026-11-19 (first minute 16.4 s against about 6 s), so shares are the reading, not seconds. Of 27.6 s over 20 rounds: the stepper 18.9 s (68%), all of it 15 route searches, 14.6 s of that in 662 walking plans; the clock 8.1 s (29%), of which the sim's own tick (`spatial.prepare`) 6.5 s and answering births 1.0 s. The first minute is 16.9 s (61%). Minutes 2 to 20: mean 0.56 s, median 0.19 s; six of nineteen over 0.5 s on this run (0.78, 0.56, 2.84, 0.85, 1.44, 2.11 s). The 2.84 s minute is the clock's (2.80 s), not the stepper's; the 2.11 s one is a 1.55 s search. Not attributed further. Against D73 the median minute is inside the guard and the tail is not, on a slow machine.
- 2026-11-19: E10 steps 6 and 7 measured and built on `e10-foreground` (case 1082 not set; seed 18181373014846284518, release, `--play --curve 20 --foreground-walks`, `round.foreground_step` per world minute, three runs). Minute by minute, ms: 5647 209 23 1.2 8.8 3.8 1.4 2.5 1.9 1.5 4.0 3.9 0.9 0.8 38.6 13.3 37.4 567 9.7 10.1 (runs two and three: first minute 6162 and 5861, the 18th 570 and 536, the rest within 10 percent). Distribution: median 3.9 ms, 17 of 20 under 40 ms, three above (5.6 to 6.2 s, 210 ms, 540 to 570 ms); the first minute is 99 percent of the total 6.4 s. The window's own `Walk::begin` for the same first minute takes 18 ms (one body, actor 38), so the first minute's 6 s is not that walk: the round's first step sets the participant's foreground before the clock asks (the clock's minute 0 hands back four bodies, 38, 57, 41 and 58; the window at that state sees one), and the heavy search is among the three it adds, not attributed to an actor here. Walking is short: in minutes 0 to 9 the stepper gave 0 or 1 walk a minute, of 1 or 2 edges, each reaching its end at step 1 of 60 (D59's reach is about 49 cells a step). A capture therefore shows one step of motion and fifty-nine standing, and the walkers are not the bodies that move far: bodies 25 and 52 crossed 203 and 280 cells at the minute, out-of-view arrivals the stepper does not own. Options for the cap (case 1082, Mark's): (a) no cap, 16 as now: worst minute 6 s, once; (b) cap searches by node budget per body per minute (a body over budget stands, D66), leaving the 5.6 s search to be refused; (c) spread a minute's searches over the previous minute's steps on a worker, so none lands in the tick. Window: with a tick on the clock's worker the frame is never stalled by it (the first minute took 6.1 s and the window kept drawing); the same minute run in step stalled the frame 6 s and tripped the lane's 10 s presentation watchdog. Capture: `testing/isocosm/e10-foreground/` (f00 to f13, `across_minute.scn`): step 0, the stance at step 1 held to step 60, the tick landing with no change to the walker, a body arriving at the right edge at tick 1; the camera followed the sim's pose until corrected (it jumped 43 px at the tick), now follows the drawn one.
- 2026-11-19: E10 step 4 and 5 measured (D64's check), seed 18181373014846284518, release, 20 steps, `--play --curve 20`. Rule off, the same curve lines as main (living 67 at tick 1, 150 at tick 20; 23 searches). Rule on (`--foreground-walks`): living 67 and 148; the tick asks for no walk of an in-view body (zero step requests answered); 15 route searches remain, all the stepper's (24 route reads, 8.9 s of 10.9 s read time; the stepper's 20 minutes took 9.1 s, 8.0 s of it the first minute's one many-trial search for actor 61, the same search step 0 found). Wall time 28.6 s off and 18.4 s on on a busy machine; `step_micros` 24.7 s against 16.0 s. The stepper calls the same route code, so the cost moved rather than fell, as step 0 said it would; only the single heavy search is large. Whole-run comparison over 6 founding seeds (11 others did not found: no stand for the controlled body), living at tick 1/5/10/20, off then on: 1818: 67/67/99/150 then 67/67/97/148; 15: 67/67/86/107 then 67/67/85/106; 28: 67/67/67/78 then 67/67/67/76; 29: 67/67/90/128 then 67/67/87/130; 50: 67/67/67/93 then 67/67/67/77; 58: 67/67/99/138 then 67/67/99/138. Rule-off tick-20 values run 78 to 150 (mean 115.7); on minus off at tick 20 is -2, -1, -2, +2, -16, 0 (mean -3.2). Seed 50's -16 is the largest and a fifth of its count; no seed is outside the seed-to-seed spread, and no tolerance is asserted here. The remaining differences are in-view bodies walking the whole route's reach in a minute (D59) instead of one edge.
- 2026-11-18: verified on the branch (`e10-stepper`), step 3's control and what keeps stepped bodies and out-of-view travellers apart. The control's zeros: a traveller departs in the minute the sim decides its walk (`foraging/food.rs` `travel_target`, `stage/staged/step.rs`, journey made by `departure`, `foraging/travel.rs`) and `arrive_travellers` (`schedule/mod.rs:214`, `travel.rs:112`) lands it once `arrives <= tick`; every journey seen arrived at its departure advance's start plus 2, so a body lands at the end of the minute after the one it left in, and a minute before its decision shows nothing. Positive control: with the window closed, bodies that did travel moved 1.0 to 59.5 cells (`stepper_tests`). Over 10 seeds, four consecutive minutes, medians stepped 3.0 cells and travelled 13.6, not within one: of ten bodies, four chose a different meal (the stepper's is `food_route`'s nearest by physical route, the traveller's `travel_target`'s nearest by place route) and, with the same meal, the traveller lands where `arrive_near` (`embodiment/habitat.rs`) puts a stance touching the meal or, failing that, the nearest free stance, while the stepper stops at the route's first contact (seed 20: 8.6 against 34.5 cells; seed 27: 2.0 against 59.5, route two edges). Native in-view walking publishes one edge a minute (`foraging/route.rs`, `path.take(2)`): median 1.0 cell, none in half the cases. The D60 condition is not met; the fork is with Mark.
- 2026-11-18: verified on the branch, Mesocosm carried no handbacks. `Game::advance_clock_with_geometry` (`mesocosm-runtime/src/spatial/driver.rs`) staged the minute through the handback-less `try_advance_birth_spatial_with_flows` and `try_advance_birth_spatial_with_fallback` (the latter wrote `handbacks: Vec::new()` into its command); `profile::Record` held none; and `spatial/replay.rs` matched `AdvanceBirthSpatial { ticks, steps, arrivals, .. }` and rebuilt the outcome without the `; handbacks <digest>` suffix the native entry carries, so a round with handbacks would have failed replay. Now the clock takes them (`advance_clock_round`; `advance_clock` is the empty case), `Record` keeps the handbacks and the sim's reports (both absent when empty, so older saves keep their bytes), and replay checks the record's handbacks against the entry's and rebuilds the suffix. The generated-driver path needed an additive cosmograph entry point, `try_advance_round_spatial_with_fallback` (`spatial/birth_fallback.rs`); the old one calls it with none. `tests/round_handbacks.rs`: an accepted handback is drawn into the game, kept in the record, saved, loaded and replayed to the recorded hash (a record with its handbacks removed is refused); a refused one leaves sim and game as an advance without it and its report is in the record.
- 2026-11-18: verified on the branch, D52's premise. `Simulation::hand_back` publishes an accepted pose through `keep_spatial_pose` (`spatial/foraging.rs`), which writes the body's `Resident` afresh with `travel: None`, so an accepted handback has always ended that body's journey and `arrive_travellers` (`foraging/travel.rs`) never lands it again; the earlier reading that `handback.rs` never touches a body's pending travel was wrong in effect. A refused handback publishes nothing and the journey stands. `spatial/handback_tests.rs` shows it with a control: a journey due next tick lands the body when no handback comes, and not when an accepted one does, in a world with the rule and one without; a refused handback leaves the journey to land as without it. The rule therefore changes nothing until step 4 stops route search for in-view bodies under it.
- 2026-11-16: verified on the branch, D54 and D55 (code in `shared/cosmograph/src/spatial`). Every read of other bodies' boxes by a route search goes through `Blockers::hits` (`embodiment/blockers.rs`), which notes the box asked about: node admission (`foraging/route/search.rs`, `trial`), `Checks::intersects_residents` (`embodiment/walking/checks.rs`, every sampled posture of a stride, pivot or stage check), and for a body without articulated stepping `sweep_clear_in` (`embodiment/contacts.rs`: its union box, `swing::check_clear` for a turn, `narrow::clear_rotated` and `clear_prepared` for each swing sample). The cache's window filter (`route/cache.rs` `find`) drops boxes wholly outside the search volume before either side reads them. Other users of a body list (`habitat.rs`, `founding.rs`, `geometry.rs`, `check_stride`, `plan_stride`) are not route reads and keep plain slices. No read needed the exact comparison. The record is a 16-voxel cell grid of every region asked, found empty or not, plus the boxes standing in those cells when the search ended; an entry holds while the same boxes stand there, so a body entering, leaving or moving inside a consulted cell ends it and one in no consulted cell does not (a coarse cell only ends entries sooner). Soundness: a search is a deterministic function of its other inputs and of the answers to its box queries, and unchanged boxes in every region asked give unchanged answers. Proof by test: `route/cache/differential.rs` and `COSMOGRAPH_ROUTE_CACHE_VERIFY` (97 of 97 hits equal a fresh search on the 20-step run). Measurements: [receipts](archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md#2026-11-16-e10-step-0-route-cache-by-consulted-regions-d54-and-a-cheaper-step-check-d55). The remaining 23 searches are 21 cheap ones and three heavy first-time searches (actors 61, 58 and 142: 3.2, 1.8 and 2.0 s), run once each instead of three, two and four times.
- 2026-11-12: new worlds record the fixed-point sunlight rule. Staged credit and climate both read it, and preexisting saves select the legacy rule through the absent field. The light and witness tests cover replay. The [execution receipts](archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md#2026-11-12-sunlight-replay) record the native and WebAssembly check.
- 2026-11-12: the ordinary release played-tick profile still spends most of its time in food route search and walking plans. The measured case is in the [execution receipts](archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md#2026-11-12-played-tick-profile). E9 still needs a profile that isolates storage cost before it can claim a played-tick speedup. Exact grouping serves state-equivalent processes. To qualify far-tier scale after E4 and E6's fidelity checks, hold the sites, lineages, band widths and occupied bands fixed, vary the represented population, and record evaluations, retained state and wall time. Report exact execution alongside it as the reference.
- 2026-11-08: E7b on the bar's founding (seed 9, two world months, release): 16.7 s Grouped and 16.8 s Individuals, against 28.6 s and 28.1 s with the per-tick draw. The two modes agree on hash `5242331604158281548`, history digest and monthly samples. Hazard passes fell from 36% of the profiled time to 3%; sunlight, feeding passes and their plans remain. Living at the samples was 240, 267 and 298, where the per-tick draw gave 240, 273 and 311. On a test world whose hazards run a year to the day, 96 hazard deaths in two days each fell on the tick their draw names, matter was conserved, and a save replayed in either mode and went on alike (`tests/death_drawn.rs`).
- 2026-11-08: EX4 measured on the bar's founding (`tests/world_time_year.rs`, seed 9, two world months, release). Before: 230 s Grouped, 276 s Individuals, hash `1988914608126411875` in both. A shifted site condition re-filed every resident afresh (`Filed::touch_site`) though no gate read `world:weather`; the hazards' 21.2 million evaluations each built a receipt, read its requirements and staged nothing (`execute.rs`); per evaluated act the process definition was copied. After the five exact changes: 28.5 s and 28 s, hash, history digest and monthly samples unchanged. One bodied two-year run on that founding then took 953 s (13.5 s in month 1 at 273 members, 69 s in month 17 at 765), about 0.09 s per living member per month. Profiled remainder over the two months: hazard passes 36%, shared-ground plans and feeding passes 30%, sunlight 8%. Each mature member still makes two keyed hashes a tick (its act's name and its draw); only a death tick drawn once at maturity (E7b as first planned) removes them, moving draws and hashes.
- 2026-11-02: E10's third step built (case 1084, doctrine Bodies 16, case 1112). Where a world states a walking speed (`Rules::speed`, set by new foundings that declare foraging), a posed forager outside every participant's foreground (`Simulation::in_view`) chooses its meal at place grain, the detected food nearest by its route over the site's Places and no nearer than its gap (`spatial/foraging/travel.rs`), leaves with that way's travel time at its speed (`cosmograph::speed`), keeps its stance and lands touching its meal at the end of its due advance, the game drawing the arrival; no step, route search or walking plan runs for it. Bodies in view keep the game's steps until their handbacks replace them. In such a world the sim re-settles no one (D38): out of view no stance is seen, and in view the foreground places feet; one in-view body had failed its settle search at 1.16 s on every tick. On the 2026-10-31 world (`--play --curve`, release, seed 18181373014846284518), 20 ticks took 14.0 s where main took 27.3 s and 60 ticks 28.6 s where it took 41.7 s; the cold-route outliers of 3.4 to 4 s a tick are gone, route search remains only for bodies in view (45 searches, 4.4 s in 60 ticks), and deliberation (`body:rest`, 7.3 s), smell reads (5 s) and Mesocosm's lineage review replaying the record are what is left. Owed: the route's difficulty beyond its length; wading and climbing costs; a hand-back that skips the Step act of a body handed back.
- 2026-11-01: With PG7 built, the profiled world (`--play --curve 8`, the same seed as 2026-10-31, release) derives its 14 Places windows in 1.9 s where it took 9.9 s, all at startup; on main of this date its played ticks derive none, so a tick's cost is now route search, walking plans and resting bodies' settling, and the 8-tick run took 26 s where it took 32 s.
- 2026-10-31: A played tick on main is spatial work, not bookkeeping. Headless play (`mesocosm-genet --play --curve 20`, release, one seed, 67 to 94 members) took about 1.9 s a tick under `sample`. Route search for food (`foraging.route.search`, about 0.4 s a cold search), walking plans and resting bodies' settling took nearly all of it, with Places derived again on a geometry-cache miss (`site_geometry`, 0.6 to 2.8 s each, a few a tick). The rule interpreter (staged effects, `meaning`), account reads, schedule filing and digests together took under 3%. So interning account keys (E9's first step) buys nothing yet; D42's layout pays only once these costs are gone, and E9 waits for them. They are EX2's (cold route search), `place-graph-engine` PG7's (Places from the column rule) and the near-tier movement question's (route search and walking out of the tick).
- 2026-10-28: E1c built (`schedule/filed/kept.rs`): the index is reconciled at each advance by each group's body handle and each site's conditions, rebuilt on other rules, movement mode or routes, dropped on a refused advance. A played tick now costs well under 978's premise of 0.45 s (receipts: `archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md`), so its tempo question is open again. No threads exist in the sim's evaluation (thread scopes only in the probe bin and interim reproduction), so E3 is not built.
- 2026-10-28: E2 is not built: bite and amount draws are still keyed by the act number (`stage/staged/meal.rs`, `stage/staged/mod.rs`); soma draws by id.
- 2026-10-28: The crowd lives only in `shared/cosmograph/src/probe/crowd/`; `TierLine` (`directing/tier.rs`) is exported and unwired.
- 2026-10-28: The head's tick is physical work, not scheduling bookkeeping: a headless play run (`mesocosm-genet --play --curve 60`) spent about 12.5 s a tick in scheduling, mostly foraging route search, Places derivation (`terrain.places`, re-derived hundreds of times in 60 ticks) and walking plans; one boundary forecast replay took minutes (receipts). The battery's site-grain worlds exercise none of it.
- 2026-10-28: After E7a, rest (`harm/rest.rs`) is the largest pass: a Choice act and the first choice pass of each tick, so it carries every deciding member's deliberation; E8's periods are what cut it. The death hazards are a small share, so E7b is minor. A write outside an act (a command such as a wound) does not settle strain first, so the ticks before it accrue at the new mood.
- 2026-10-28: What D15 to D17 touch: a process has one `period` for the world (`rules/mod.rs:193`); the queue is keyed by due tick, priority and id, and a pass walks every group or every filed one (`schedule/mod.rs`). Strain and relief run every tick for every member (`rules/mind.rs:50`); so do each lineage's death, age and, from maturity, hazard processes (`ecology.rs:133-190`). The agentless `body:hazard` acts once per world body, not per member (`harm/hazard.rs`). Bite and amount draws are keyed by the act number, so any phase removing accepted acts moves them until E2.
- 2026-10-28: D4's order ("speed first: E1, E2, E3, then E4 and E5") was amended by D7 and D8, which inserted D7a, D7b, D9a, D11 and E1a before E1 completes; no later decision restates the order.
- 2026-10-28: D9b and D10 are dropped (D11, D12); the old plan's status line and its phase list disagreed, and the cut lists neither as a phase.
- 2026-10-28: The old host plan's earlier receipt figures (cohort sharing on the core-tick lane) are not comparable with the execution plan's battery figures: D5 and D7a redid the sharing as stage-body and part `Arc`s.
- 2026-10-28: The frame's extra deaths were far consumers leaving food, not the far-meal draw: between feeds a posed forager walks toward food or waits with it in reach, while a far one could do neither (`foraging_walk_matches` and `foraging_wait_score` refuse unposed actors), so a move was its only choice (D36). The far-food table also counted every living body at a site, the foragers' own among them, as prey (D37). With both fixed, frame-on deaths match frame-off over the same opening (receipts).
- 2026-10-28: The critter's blank view after a migration is not the frame's: a stance is admitted against the body's matter as it is posed, and matter gained or lost standing still moves the matter centre, which leaves a stance near the edge of its feet; `realized` then fails and the scene drops the body. Matter does not change `body_revision`, so nothing re-posed it (D38). The same failure was traced with the frame off.
- 2026-10-28: A large part of `mesocosm-runtime`'s test suite already fails at `76f8f5cc` (among them `held_advance_tests`, `verified_question_tests`, `ecology_offer`, `live_input`, `physical_play`), the same set with D36 to D38; the lib tests among them fail the same with the frame call disabled (receipts). Set aside as a work item (Mark, 2026-10-30), then settled at his word on 2026-10-31 (D43 to D45); of the triage's 36, the ordinary A1 foraging test still fails, owed by D46.
- 2026-10-28: Unpaced, `physical_swing`'s hungry-starter test ran past half an hour at the frame-on head, against 222 s at `76f8f5cc`, all of it in the frame draw: its frame covers the whole site, and the consumers whose food was already surrounded each ran the near-food search to its candidate budget and failed, every advance (D39); the site's ground also held only about half the frame's share, and the rest failed the ordinary search every advance (D40). D36 to D38's own work (wait score, prey matter, re-settle) came to under half a second a tick (receipts).
- 2026-10-30: Under D19's pacing a Grouped world's saves could fail to replay (`Session::load`, `history.rs:282`), first seen in the 879 offer fixture's world with its joint witness lengthened past the played critter's first paced choice. `decides()` (`schedule/mod.rs`) offsets a cohort's choices by its first identity, and `restrict` regroups cohorts only at an advance's end, so the same ticks advanced singly or in one span grouped members differently and chose on different ticks; `replay_until` advances in spans where nothing was recorded. Individuals and unpaced worlds were unaffected; the head plays Individuals, and placed play records an entry every tick, so ordinary saves replayed. Fixed by D41.
- 2026-10-31: Case 1057's cause: a torus two sites high joins each pair of rows by two routes (`map/grid.rs`), and `Templates::declare` made one crossing per route at the destination's first stand, so the A1 witness's v1 profile held two arrivals for one (actor, from, to) and `Profile::validate` refused the world ("arrival input has duplicate or absent native actor/route/destination"), about one seed in three (seeds 1793482854536437400 and 1 among them). The second route's move act was also lost: generated moves were named `move:{site}-{to}`, so the later route's process overwrote the earlier. Native travel took the first route to a destination and recorded no route. The check entered at `39a597c9`.
- 2026-10-31: `spatial_clock`'s two birth tests failed from `a3051e84` (D19): their played fixture's producer lineage decides every 10 ticks, so no birth is asked for on the first tick, the only one they read. Mark chose to keep the pacing and advance to the first birth ("Advance to the first birth"): both tests now step the world until its clock first asks for one (tick 8, capped at 64) and compare from there.
- 2026-11-01: Case 1057 moved `physical_swing`'s hungry starter off travel. Before it (`75612a3c`) the critter's food went out of reach at tick 8 and a seeded tie among three moves sent it to site 3. With arrivals landing at the edge they entered by, other members stand near it, its food stays in reach, it eats at tick 9 and it is still resident at tick 14, where a birth question holds the run. Departure itself works: 40 others are travelling at tick 8 (D47).

- 2026-11-01: A1's three-epoch gate did not run on main: each draw is recorded in Individuals and replayed in both modes, and the Grouped replay refused a take-up ("only a living deliberative critter or sophont is taken up") because the heir had died in it. Pacing keyed a group's choice tick on its first identity (`decides`), so a cohort's members chose together in Grouped and apart in Individuals; on A1 seed 0 the two modes first differ at tick 4 (`next_action`). Exposed by D41 (`2df3a180`), whose per-tick regrouping made the split visible; fixed by case 1076. The matrix arms also stop at the compact receipt cap of the shared receipt directory.

## Progress

- 2026-11-19: D70 built on `e10-foreground`: `Runtime::attach_foreground` (`mesocosm-runtime/src/runtime.rs`) commits the participant's frame attention exactly as the first round would (same command, same state; the round then finds it held), and the window calls it when founding a rule world, before minute 0 is drawn. Rule-off worlds never call it. A windowed round and a headless one still end at one hash and replay to it; `the_walks_the_window_draws_are_the_handbacks_the_clock_takes` and the windowed test now compare every minute from 0 with the recorded handbacks. No saved state or Cosmograph surface changed.
- 2026-11-19: E10 step 7 built on `e10-foreground` (not merged): in a world founded with `--foreground-walks` the window's frame loop drives a `Pacer` (`mesocosm-genet` `app/foreground.rs`; `Pacer::minute_step`, `world_foreground_walks` in `cosmograph-host`, which `world_foreground_minute` now uses), authorises each minute on the runtime's own tick worker, begins the minute's walks when it lands, and draws each walking body at the stepper's stance for the current step with the walked edges and strides as its trail; a body with no walk, or not yet off its first stance, stands where the sim has it (D66); while an authorised minute has not landed the walkers hold at step 60, the pose handed back. Tests: `foreground_tests` (the walks drawn are the clock's handbacks for minutes 1 and 2; a windowed round and a headless one end at one hash and replay; no jump at the tick; bodies drawn standing and stepped) and `round_tests` (`minute_step`). Rule absent, nothing changes: the window runs the core's clock as before. Open, Mark's: see the report on the first minute's foreground, the pace default (`HostConfig::round_speed`, `Speed::X1`, no flag), input latency up to a minute at 1x, and the walks' brevity.
- 2026-11-19: E10 steps 4 and 5 built (D62, D51, D52, D57). Step 4: under `rules.foreground_walks` in a travelling world, `foraging_target` (`foraging/food.rs`) returns no walk target for a body in view, so the tick neither searches nor plans for it (`staged/step.rs` is then never reached for it); rule absent or false, the branch is the old one. Cosmograph gains two public entries, `Simulation::meal_target` (the place-grain choice an out-of-view traveller gets, nil when there is no need) and `Simulation::route_to_meal` (the walked route to a chosen meal); the stepper uses both. `spatial/handback_tests.rs` `under_the_rule_the_tick_asks_for_no_walk_of_a_body_in_view` counts step requests for in-view bodies over 40 minutes (off more than none, on none). Step 5: `cosmograph_host::drive::world_foreground_minute` steps each foreground body sixty times and hands back, and `continuation/round.rs` passes the handbacks to the clock every world minute, windowed or not; the world rule is founded by `WorldRules::foreground_walks` and the `--foreground-walks` flag only, no default. `mesocosm-genet` `foreground_tests`: a rule world's round at speeds 1, 2, 4 and 8 (ticks from the pacer over one real minute) replays its save to the recorded hash with handbacks accepted. Measurements: Findings, same date.
- 2026-11-18: E10's remaining step 3 built, its done-condition (D60) not met (Findings, same date): the shared stepper (`cosmograph-host/src/drive/stepper.rs`, `Walk`) takes the sim's own route (`food_route`) with its strides and feet, walks it sixty steps a minute at the body's speed and hands back; the sim accepts every stepped minute of 10 seeds and the stepper is deterministic. Stepped and out-of-view bodies differ by meal choice and landing stance. No Cosmograph code changed.
- 2026-11-18: E10's remaining step 2 built: `mesocosm-runtime` passes a round's handbacks to the clock (`Game::advance_clock_round`), keeps them and the sim's reports in `profile::Record`, and replay accepts them. No game code produces handbacks yet; the tests construct them.
- 2026-11-18: E10's remaining step 1 built (D52): `Rules::foreground_walks`, absent unless true, so a world without it serializes, loads, replays and hashes as before (`tests/foreground_1080.rs` `an_older_spatial_save_keeps_its_bytes_and_hash` writes the 2026-11-02 save's bytes again; `spatial/handback_tests.rs` checks no field is written when absent). Nothing reads it yet: step 4 will. An accepted handback already ended the body's pending journey in every world (Findings, 2026-11-18).
- 2026-11-16: D54 and D55 built on `e10-baseline`, outcomes unchanged (D56 needs no code). D54: a route search records the regions its checks asked about other bodies, and a remembered search is reused while the boxes standing in them are the same (`embodiment/blockers.rs` `Footprint`, `route/cache.rs`); 20 steps 36.4 s to 15.4 to 16.7 s, searches 33 to 23, hits 87 to 97. D55: the step check's untimed half is counted (`walking.plan_cost`), pivot balance is read once per posture (`Checks::balance_at`) and the posture store holds 65,536 (`walking/checks.rs`); 20 steps to 13.3 to 14.4 s, route search 31.4 s to 8.2 to 8.5 s. Same `curve:` lines (67 at tick 1, 150 at tick 20), no pinned value changed. No limit on refusals added. [receipts](archive_docs/receipts/2026-10-28_sim_execution_plan_receipts.md#2026-11-16-e10-step-0-route-cache-by-consulted-regions-d54-and-a-cheaper-step-check-d55)).
- 2026-11-12: fixed-point sunlight is the founding default. Done when dawn and dusk across the seasonal cycle produce matching native and WebAssembly witnesses, and old saves still verify under the legacy rule.
- 2026-10-28: cut from `2026-10-19_sim_execution_plan.md` and the speed items of the sim host, sim, orthographic voxel presentation and foot placement plans.
- 2026-10-28: D15 to D17 answered (decision periods per lineage, rates worked out when read, order); E1c built; D18 (checked in isolation while wired); E7 and E8 added.
- 2026-10-28: E7a built: a played world keeps strain as a rate (`strain.rs`, `Mind::rate`, `Entity::settled`).
- 2026-10-28: E8 built: `Lineage::decides` paces a lineage's choice and move passes, offset by first identity.
- 2026-10-28: D19 to D22: periods 10 / 2 / 5 in played worlds, meals scaled by period, the head at 10 ticks a second; choice-rule tests run unpaced (`directing::tests::unpaced`).
- 2026-10-28: E7b built, simpler than planned: the per-tick hazard keeps its draw, but a draw that misses on a process with no other effect makes no act (`execute.rs`), so the empty accepted act each mature member took each tick is gone; the death tick is not drawn once.
- 2026-10-28: D24 to D26: the aggregate runs a population model per site and lineage; near is the rendered area plus the roots; a band in ticks. Measured first: about 1.2 members per exact body state in played worlds (seeds 0 to 2, ticks 60 to 180).
- 2026-10-28: D27 to D29: bands run through the scheduler, a representative plus a pool, bands by age and felt needs.
- 2026-10-28: the head measured (D30, D31): Places re-derivation next, then far members off the physical path.
- 2026-10-28: D32: the camera's frame is an attention region; the Places misses were first visits and LRU eviction across a one-site world's windows, not terrain change.
- 2026-10-28: E4a built (`band.rs`, `Cohort::pool`, `Rules::banding`): far members fold into bands, conserving every account; off unless a world declares banding.
- 2026-10-28: E5a-b built (`Region::Frame`, `spatial/tier.rs`): a frame roots its posed members; with attention windows, posed members neither near nor buffered lose their pose and forage at site grain.
- 2026-10-28: E5d built (`mesocosm-runtime/src/runtime/frame.rs`, `Simulation::near`): the critter's frame is the participant's attention; far arrivals and births stay off the physical path. The headless play run's cost fell about sixteenfold (receipts).
- 2026-10-28: E5c built (`Simulation::draw_into_frames`, `embodiment::place_within`): frames fill toward their share by area, at most twice the shortfall tried an advance.
- 2026-10-28: D33, D34: the head's frame off on main (`FRAME_TIER`); far meals to be paced at the walkers' measured rate. The tier publications (`Publication::Unplaced`, `Drawn`) are on `sim-e4-probe`, not main.
- 2026-10-28: D35 replaces D34: far meals by an encounter rate from prey density.
- 2026-10-28: D36 to D38 built (`Simulation::prey_matter`, `foraging_wait_score`, `spatial/settle.rs`); `FRAME_TIER` on in main, lifting D33.
- 2026-10-29: D39, D40 built (`embodiment::draw_within`, `Simulation::draw_into_frames`).
- 2026-10-29: `physical_swing`'s fixture unpaced, carrying out Mark's answer to the runtime triage's group 1 ("Fix replay, unpace where allowed", 2026-10-28; D43); its three tests pass.
- 2026-10-30: D41 built (`Simulation::advance_to`, `Simulation::paced`): a paced Grouped span runs a tick at a time; `directing::choice_tests` holds a span against its ticks taken singly.
- 2026-10-31: Case 1057 built (directional travel, all four forks as recommended): `Movement::Routed` for new worlds, older saves keep their semantics and hashes; `Journey::border`, `ArrivalRequest::border`, `Effect::Move`/`MovePaced` `via`; parallel routes' acts named `move:{site}-{to}-{side}`; v1 crossings carry their border and stand at the entering edge, repeats refused; the v6 habitat (`embodiment::ENTERING`, `place_entering`) lands a bordered arrival in the entering strip where a frame covers it, otherwise far (E5); Mesocosm's profile v7. Tests: `cosmograph/tests/border_arrivals_1057.rs`, `mesocosm-runtime/tests/border_arrival_1057.rs`.
- 2026-10-31: D43 to D46 keyed, the runtime triage's answers of 2026-10-28. D43 to D45 built: `held()` and the foraging fixtures unpaced; the played 879 witness at ten ticks; the offered and held traces re-recorded, their hash not compared; `physical_play` on version four; the tampered-execution seal refused at its replay. `mesocosm-runtime`'s suite passes but for the ordinary A1 foraging test, owed by D46 (`bodies` BO18).
- 2026-11-01: D47 built (`physical_swing`): the hungry starter is nudged to travel after its food step.
- 2026-11-01: D46 carried out as case 1071 (`bodies` BO18): every active form senses behind `Bodies::sensing`. A1, measured with each draw's replay set aside (it fails on main before 1071: "only a living deliberative critter or sophont is taken up"): played 20/20 and bodied 18/20 (seeds 4 and 17) with sensing and without, all 40 pairs identical in births, living and peak members; no plant's light or soil share fell below one act's need. Instrument: `reproduction::sensing`.
- 2026-11-01: case 1076 built (`schedule/mod.rs` `due`): a paced choice runs for each member whose own identity is due, so a Grouped cohort on a decision tick is evaluated member by member and splits, and the execution mode never changes who chooses. A1 seed 0, recorded in Individuals for 180 rounds, now replays in Grouped; A1's 80 draws (played and bodied, sensing on and off) replay in both modes, played 20/20 and bodied 18/20 as before. Row cost in Grouped, rows over members on A1 seed 0: 176/263 at tick 5 against 55/263 before, 269/306 at tick 10 against 205/327, and about one row per member from tick 20 under both; a 30-member world in cohorts of six, every line paced at 3, holds 29/32 rows at tick 12 against 13/32. `choice_tests` holds a cohort splitting on a decision tick and both modes alike.
- 2026-11-02: E10's first two steps built (cases 1080, 1082). The handback contract (`spatial/handback.rs`): a `Handback` names a body's end of a foreground minute, rides optionally in `AdvanceBirthSpatial` (an older save keeps its bytes and hash, `tests/fixtures/spatial_minutes_before_1080.json`), and is checked before the minute's passes against the body and terrain revisions, its tile, a whole-body stance (`check_pose`, shared with arrivals), clearance from other residents and feeding reach for contacts, moving no matter; an accepted pose is published as a draw, a refused one leaves the body as the sim had it, and both are recorded in the command's outcome. The foreground set (`Simulation::foreground`): a participant's played bodies and the near members posed in its frames, played first, then nearest a frame's centre, cut at `Limits::foreground` (16 placeholder). A step measured in release (`foreground_1080::step_cost`): about 20 to 24 us a body for column stances, the whole-body stance check and clearance, about 700 bodies to a 60 Hz frame; re-planting feet by the settling search costs about 350 us a body, about 47 to a frame. Owed: the reach a body covers in sixty world seconds (no walking speed exists in world time).
- 2026-11-02: E10's round pacing built (`cosmograph-host/src/drive/round.rs`, beside the fixed-step `Clock` it reuses; cases 1080, 1081). `Pace` is a round at `Speed` 1, 2, 4 or 8 times real time or simming on the dial; a `Pacer` turns elapsed real time into foreground steps (60 a real second, each a sixtieth of a second of world time times the speed) and minute ticks (every 60/speed real seconds), integer and independent of how the time arrives. A `Ticker` runs each boundary's tick and queued handback off the frame thread by default, in order, one at a time, or in step as a setting. Tests: each speed's steps and ticks fall due on time; a worker frame loop with 200 ms ticks never took 50 ms a frame; worker and in-step end alike. Pace stays product state, logged with the product's inputs.
- 2026-11-08: EX4 profiled and cut, outcomes unchanged (case 1174): conditions re-file only where a gate reads them, definitions read from the shared genesis, missed risk-only draws stop at the draw, agentless passes visit only world bodies; `COSMOGRAPH_PROFILE_ACTIVE=sum`; `WORLD_TIME_EXECUTION` in `tests/world_time_year.rs`.
- 2026-11-08: E7b built as first planned (case 1181): death drawn once at maturity, filed as a due event (`death.rs`, `rules/aging.rs`, `Gates::ripe`), checked in distribution beside a doubled-odds control (`tests/death_drawn.rs`).
