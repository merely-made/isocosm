# The coarse step: running years nobody watches

**Status:** 2026-11-05: design brief, asked for by Mark ("Coarse-step brief")
after a reading of what the played loop stands on, revised the same day
after his correction on what the coarse tier is compared against (§2). It is
not a plan: it changes no code, and the work it implies belongs to the plans
named in §10. Its questions (§9) were answered on 2026-11-08 as cases 1163
to 1168; the answers are under §9. Mark's words are quoted with their
date.

**For:** what step lets Cosmograph run years of world time that nobody is
watching, cheaply enough for the adaptation phase, deep time and worldgen,
as a measured model of the world's normal business. Code paths are under
`shared/cosmograph/src/`. The reduction literature is the
[aggregation research](2026-09-22_aggregation_research.md); the prior art
the [prior-art brief](2026-09-18_sim_prior_art_brief.md).

## 1. Why it is a foundation

Four things the doctrine already commits to need years of unwatched world
run quickly, and none of them can run on today's step:

- **The adaptation phase** (case 1094): every lineage weighs each candidate
  change by a copy run "coarse, over years"; the player picks among what
  prospered, and every other lineage picks by its copy. A boundary is
  lineages times candidates times a year: ten lineages with four candidates
  is forty copy-years per boundary (overlay plan EB7).
- **Worldgen that selects** (ecology brief 3.7, the idea Mark favoured "most
  of all"): run deep time over many drawn worlds until their webs persist by
  region, then offer the survivors. That is centuries times draws.
- **Evolution in play** (case 1147: "Yes, slowly"): heritable rate constants
  keep drifting and being selected while the game runs, so the unwatched
  world's step must carry heritable variation, not fixed rates.
- **Deep time and downtime** (Record 7, case 1075): history is the same sim
  run with nobody watching; the VTT's downtime and every game's prehistory
  are that mode.

If no coarse step pays, "evolve it, don't tune it" cannot run, the
adaptation phase cannot weigh candidates over generations, and worlds go
back to being tuned by hand.

Mark, 2026-11-05, on what comes first: "60hz is foundational. it's pretty
crucial, especially before we wire everything, to test out the cruft that
will become tech debt later. how could i play a game that runs at 2 fps,
interface, sim, gameplay and all? the food web is important (content). the
coarse step is important (performance, content too). and the reason why
that m4 brief is ill defined is we correctly identified that all the rest
is nada without actually being able to run our tests and playtests." So the
coarse step is a speed question and a design question at once, and it sits
beside the 60 Hz work (the qualified raster retention plan in mere, the
scene composition plan here) as what makes tests and playtests runnable at
all. Keeping only what is near the view individual also shrinks the
per-frame sim.

## 2. What the coarse tier is compared against

Mark, 2026-11-05: "the sim is not gonna give the same or similar outcomes
as play. players do crazy stuff. they are not the norm. we need a baseline
of normal from the sim, not exact simulation of the individual tier at the
cohort tier... isn't that just contradictory? that's part of what makes the
aggregate, the coarsening, the world bank of entities cheap to sim too,
right? we're supposed to be modeling; the things happening in the player's
scope are not copyable by the sim, outside of them, it's normal business,
way more predictable and processable?"

The reading this brief takes, consistent with 113:

- **The coarse tier never predicts the player.** It models normal business,
  and that is why it is cheap. The noted run as individuals; the fungible
  agree only in distribution; "who" is undefined until someone looks.
- **The reference is the fine sim with nobody watching**, every member run
  individually, never play. "Exact" in the rulings and in EB7 means that
  unwatched fine run. Normal is measured from it, not declared: a
  hand-written normal would make the aggregate and the fine sim two
  different worlds, and the switch from one to the other as a region comes
  into view is a seam a player reads.
- **Normal is predictable in large numbers and noisy in small ones.** At
  tens of members per site (the S10 arms), extinctions and troughs carry
  the outcome, so the coarse tier must reproduce the spread, not only the
  mean.
- **Odd states are inputs, not forecasts.** Whatever the player leaves
  behind (a site emptied of predators, a flooded soil) is a state the
  coarse step continues from normally. Handing over from such states, not
  predicting them, is what is tested.

## 3. Where it stands

**The exact year takes about forty minutes.** Measured 2026-11-05 on the
Windows laptop, release build, `tests/world_time_year.rs` on seed 9: 240
founders on a laid 2 by 2 map, ten lineages, world-time rates, unbodied,
grouped execution. The year passed Mark's bar (every lineage alive, within
four times, no third lost) and took 2,419 s. A month cost 91 s at first and
about 280 s by months eight to twelve, as the living count rose from 240 to
about 780; 1,400 births, 716 starvations and 145 hazard deaths in the year.
A concurrent disk-cleanup job shared the machine, so the figure is
indicative, not a receipt. Against §4's working target of a quarter second
per copy-year, the exact step is about ten thousand times too slow, before
bodies, which cost more.

**Today's coarse copy is slower than the exact one** (EB7,
`lineage/boundary/coarse_tests.rs`, 2026-11-02): grouped execution plus
E4's bands took 6.8, 36 and 107 s against 4.8, 18 and 35 s exact, at
horizons of 20, 180 and 600 ticks. That readout ran the old per-tick A1
rates on a web of diets, which dies within a few hundred ticks; its 14 of
14 agreements at 180 and 600 ticks are candidates tying over a dead web,
not evidence the coarse copy picks well.

**Grouping does not compress bodied worlds.** The exact-state crowd (207)
saved 5.93 times the evaluations in independent worlds and nothing in
ecological ones, where feeding falls back to individuals (aggregation
research). Played worlds hold about 1.2 members per exact body state (D24),
so a histogram of exact states barely folds them, and ruling 439's
members-per-state switch would put nearly every bodied site and lineage
below any useful bound.

**The population model was ruled as rates and built as stepped
representatives.** D24 ruled counts by age band and condition, with births,
deaths, meals, upkeep and moves "as rates". D27 then ran each band as a
representative body with a count "through the scheduler", and that is what
`band.rs` builds: far members fold into a representative plus a pool (D28),
and the representative is still stepped every tick by the same processes.
E4b, bands acting once for all their members with effects scaled by count,
is open, and folding costs more than it saves at 1.2 members per state.
Nothing yet advances a band by a rate over a span.

**No rate model exists.** E6 (rulings 261, 266, 439, 440) is the ruled
shape: fitted once per rules digest on the bench, certified against the
exact runner as 207's crowd was, shipped with the rules; a world under
rules without a certified model stays exact.

**The coarser rung's precondition is met.** Case 1075 ruled the time ladder
as "configurable spans now and aggregation later", aggregated rung by rung
"once E7/E8 land". E7a (strain settled when read) and E8 (decision periods
per lineage, meals scaled to the period) are built and exact; E7b was built
simpler than planned, the per-tick hazard keeping its draw but making no act
when it misses. The advance limit already guards work, not ticks (284), and
idle time is skipped.

**The rates are becoming a law.** Case 1148 ruled "mass for rates, leg for
speed": rates scale with mass to the three-quarter power (ecology brief
3.1, queued after the seed bank by 1146). Once rates are a function of body
mass and heritable constants, a population's expected births, deaths and
upkeep over a span can be computed from its members' masses rather than
fitted per rules digest. Feeding and competition are the exception: who
eats whom, how often, and how crowding bites come from bodies meeting in
space, which no allometric law supplies.

## 4. What "quickly enough" has to mean

EB7's done-condition asks for "a world year quickly enough for the phase".
The phase budget is not stated anywhere. As a working figure: if a boundary
may take ten seconds of a player's time and holds forty copy-years, each
copy-year must cost about a quarter of a second. Worldgen selection over,
say, a hundred draws of a century is ten thousand world-years, which at a
quarter of a second is about forty minutes on one core, before parallel
copies. These figures are this brief's, not rulings (question 2).

## 5. Four families of coarse step

### A. A fitted rate model per rules digest (E6 as ruled)

Per site and lineage, a stochastic model of births, deaths, meals, upkeep
and moves, fitted on the bench over many draws and certified against the
exact runner; the world leaps spans with it.

- **For:** already ruled (261, 266, 439, 440); certification has a shape;
  fast once fitted.
- **Against:** a model fitted per rules digest goes stale as heritable
  constants drift (1147), since a lineage's rates are its traits, not its
  rules; every novel trait combination is outside the fit.

### B. The coarser rung on the same definitions (case 1075)

Run the same process definitions with a longer span per step, rate-type
effects scaled by the span as E8 already scales meals by the decision
period.

- **For:** one definition, no fitted model, nothing to go stale; the
  doctrine's own unification ("one clock, detail by attention").
- **Against:** bodied processes are minute-grained (walking, routes, bites,
  footing), so a day-long step must stand in for a day of encounters, which
  is a rate model under another name for exactly the processes that cost
  the most (EX4: the shared-ground feed and produce passes dominate).

### C. The population model, leaping (D24 as ruled, with 3.1's law)

Outside the near tier a site and lineage is what D24 said: counts by stage
and condition with its matter, advanced over a span by rates read from the
members' masses and heritable constants through 3.1's law. Only the
interaction terms (feeding, crowding, predation encounter) are calibrated,
from unwatched fine runs, per web shape rather than per rules digest. This
is the shape of the Madingley model (Harfoot et al., 2014): cohorts by
functional group with allometric rates on a monthly step. §6 sets it out.

- **For:** heritable drift is free; the interaction terms are few and
  testable; it finishes what D24 ruled and D27 built only half of.
- **Against:** the interaction terms are where the S10 arms found the web's
  behaviour lives (functional response, overshoot, troughs at tens of
  members), so their calibration carries the risk; a cohort's spatial
  structure within a site is gone.

### D. A learned surrogate per rules digest

A with a non-parametric fit: the same staleness under drift and the same
certification burden, with less to read when it fails. Not recommended.

## 6. How C works

**The coarse state, per site and lineage** (D24): counts by life stage and
condition (fed, hungry, wounded); matter per account with D28's pool, so
totals are conserved; the lineage's heritable constants as a small
distribution, not one value, so drift (1147) survives a fold; and the
site's own stocks (soil, detritus, the light store). Noted members (roots,
relations, anything pointed at) never fold.

**The step.** Over a span of a day, a week or a month, each class's expected
births, deaths, maturations, moves along routes and upkeep are worked out
from 3.1's individual rates summed over the class's trait spread. Actual
counts are drawn as binomial or Poisson numbers from the keyed integer RNG,
so the step replays. A class under about ten members is stepped event by
event instead of leapt, since extinctions and troughs live at small counts
and leaping misjudges them. Every flow moves matter between ledgers, with
rounding residue in the pool.

**Measured normal.** Only the interaction terms are fitted: how fast
grazers find food at a given density, how crowding slows breeding, how
often predators meet prey. Fine runs with nobody watching record, every
span, each site and lineage's coarse state and what happened next
(`Simulation::watch`, ruling 320, is the hook). A small functional-response
family (type II or III grazing with a crowding term, integer coefficients)
is fitted per web shape, or a lookup by coarse-state bin. The fit runs on
the bench on whichever machine is idle, never in play. The coarse rates are
then what the fine world did in that state.

**Handing over.** Fold, when attention leaves: members summarise into
classes, losing nothing because the pool keeps the totals. Lift, when
attention arrives: individuals are drawn from the class counts, the lineage
recipe and the trait spread, placed through Places, and the lift is logged
(113). The step is defined from any state, so a state the player left
continues from where it is.

**The checks**, with no player in any of them:

- **Same start, both ways.** From seeded worlds and planted odd starting
  states (predators removed, grazers doubled, a site emptied, soil
  flooded), many fine and many coarse replicates run over a horizon, and
  the spread of what later processes read is compared: counts per site and
  lineage, age and condition mix, extinction odds and timing, stocks, peaks
  and troughs. `cosmograph-probe`'s difference and equivalence tests (Holm
  corrected, with 317's 1% refusal bound) are the machinery.
- **The seam.** Fold, run coarse, lift, run fine, against fine throughout.
  A burst of deaths just after a lift means the lift draws bad bodies.
- **EB7 on top, relaxed.** The coarse copy's pick scores within a tolerance
  of the exact copy's best, rather than naming the same candidate: on near
  ties the same name is noise.

**What it costs.** A coarse step is a few hundred integer operations per
site and lineage; a year at monthly spans for ten lineages on four sites is
about 480 site-lineage steps, which should be milliseconds. That is an
estimate, not a measurement. The fine runs are the expensive part, paid
once per web shape on the bench.

**The order.**

1. A recorder of coarse snapshots and the events that followed, over fine
   runs.
2. Coarse state, fold, lift and the step with demography only (births,
   deaths, ageing, upkeep), tested first on worlds with no feeding
   interaction, where it should match almost exactly.
3. The fitted feeding and crowding terms, tested on the web.
4. The odd-state and seam suite.
5. Wiring into EB7's copies and deep time.

Steps 1 and 2 can start now. Fitting step 3 waits for the food-web lane's
two-year bar (1146), or it calibrates to a web that is about to change.

## 7. Moving through the world

Mark, 2026-11-05: "we've asked before, how large a local area we could
render. Be interesting to be able to migrate freely through areas,
seamlessly or map by map per world tile like (some "navigate by walking to
the map edge" mods of) rimworld, eh. Seems important, eponym-wise, and would
be delightful, mesocosm-wise."

Travel is where the coarse tier meets the player most often: every area
walked into is lifted from it, and every area left behind folds back. A
coarse tier that hands over cleanly is what makes a migration arrive
somewhere alive, with the herds, troughs and carrion the area's normal
business left, rather than somewhere freshly founded.

What the records already hold:

- **The site is the playing unit, the chunk the paging unit** (147); a site
  of 2,048 units spans 64 level-3 chunks. Outdoor patches are cut by a
  world-rule cap with presets from 16 to 512 units a side, or a founder's
  own x by y (420).
- **Arrivals enter by the border they crossed** (1057), keyed by the
  route's border, parallel routes distinct; Mesocosm's runtime carries it
  (`mesocosm-runtime/tests/border_arrival_1057.rs`).
- **The region** (place graph SP8, ruling 124): a drawn region of hundreds
  of sites "lifted where attention is, with memory bounded by the loaded
  window and not by the world". Open.
- **Distant terrain at coarser rings** (presentation L10), with rung changes
  seamless where rungs share geometry (426); a passage narrower than a
  coarse cell can vanish at a far ring (place graph PG4, open).
- **Near is the rendered area plus the roots** (D25), so what is rendered
  is what runs individually: the render window and the sim's near tier are
  one size.

What is not held: **no measured answer to how large a window can be
rendered and simulated inside 16.7 ms.** It is set by terrain bricks and
residency (PG5), bodies near-live and far-cached (1161), Places derivation
(about 0.14 s to re-derive a whole 256-cell window after an edit; ecology
brief §5) and cold route search, which the execution plan already finds too
slow "for fluid play" (EX2). It is a 60 Hz measurement, and the window size
it yields bounds both what a player sees and how many bodies run
individually.

Two ways to travel, not exclusive:

- **Map by map.** Walk to a site's edge, cross by its border (1057), arrive
  in the next site lifted from the coarse tier. The crossing hides the lift,
  the site stays the unit the sim and the store already use, and memory is
  one site plus its neighbours' coarse state. Most of it exists.
- **Seamless.** SP8's region streams sites in ahead of the walker and folds
  them behind, coarse rings carrying the horizon. It needs lifts fast
  enough to finish before a walker reaches the edge, PG4's passages kept at
  coarse rings, and PG5's residency.

The order this brief would suggest: map by map first, since it is the same
lift, fold and border with the latency hidden; seamless as a world setting
once the window measurement and lift times allow it (question 5).

## 8. Tensions

- **Heritable rates against per-digest fits.** Rulings 440 and 1147 pull
  apart: a fit per rules digest assumes the rates are the rules', and
  evolution in play makes them the lineages'. C fits only the interaction
  terms and reads the rest from the members.
- **Space.** Being at one site does not mean equal access to its food
  (aggregation research). A month-long step carries moves between sites
  (journeys, 1111's crowded walkers leaving) as rates, and a region's
  persistence (ecology brief 3.3) is read across sites.
- **Determinism.** Every count is one keyed integer draw, not a replay of
  the minute draws it replaces; nothing floats.
- **The near/far line.** Whatever runs far hands back to the near tier
  without a seam a player can read (D26's band in ticks, 113's log of what
  was watched).

## 9. Questions for Mark

1. **Which family to try first.** Recommended: C, with B as the control it
   is measured against.
2. **What "quickly enough" is.** How long may a boundary take, and how much
   worldgen time is acceptable for selecting worlds? Recommended: a quarter
   second per copy-year as the bar to design against (§4).
3. **What certifies the coarse tier.** Recommended: §6's checks, same start
   both ways over planted odd states, the seam, and EB7's pick within a
   tolerance of the exact best; the reference always the unwatched fine sim.
4. **Whether to start before 3.1 and the two-year bar.** Recommended: steps
   1 and 2 now on today's world-time rates; the fit (step 3) after the
   two-year bar, and the rate source moved to 3.1's law when it lands.
5. **How travel works (§7).** Recommended: map by map first, crossing by
   the border (1057) with the lift hidden in the crossing; seamless as a
   world setting once the render-window measurement and lift times allow
   it.
6. **The render window as a measurement.** Recommended: measure the
   largest window that renders and simulates inside 16.7 ms on the matched
   fixture the 60 Hz work uses, and make it the bound for the near tier.

**Answered 2026-11-08** (cases in the design record):

- **1163:** one sim, pared back by context, with a cohort form after the
  field's models. Not C as written: the cohort form is the far form the one
  sim's processes run in, demography from 3.1's law, encounters from
  mass-scaled functional responses derived from the processes' parameters
  (Madingley's way, set once, not fitted per web); a term is fitted per web
  shape only if the checks fail. B is the control arm. Mark: "a useful
  prototype. we will be able to determine if this is a good model, or if we
  should check out others!"
- **1164:** "Measure first". §4's quarter second stays a working figure.
- **1165:** §6's checks, as recommended; EB7's done-condition relaxed.
- **1166:** "Wait for 3.1 and the bar". §6's steps 1 and 2 do not start
  now; the food web's S10 work is the critical path.
- **1167:** map by map first, as recommended.
- **1168:** the window is measured and bounds the near tier, as recommended.

## 10. Where the work would land

- The step itself: the execution plan's E4 (finishing D24 as rates, E4b)
  and E6 (the calibration of interaction terms, amending 440's per-digest
  fit if Mark so rules), with case 1075's rung as the control arm.
- Its consumers: the overlay plan's EB7 (the adaptation copy, its
  done-condition relaxed if question 3 is so answered), the sim plan's S5
  (deep time, worldgen) and TU, and the ecology brief's 3.7.
- The re-run of the coarse readout on the world-time ecology: EB7's
  Findings.
- Travel: the place graph plan's SP8 (the region), PG4 (passages at coarse
  rings) and PG5 (residency); the presentation plan's L10 (rings); the
  window measurement beside the scene composition plan's 60 Hz work;
  Eponym's E3.4 (real-time zoom) as the first product consumer.
