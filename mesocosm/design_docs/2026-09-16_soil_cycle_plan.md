# Soil cycle plan

**Date:** 2026-09-16

**Status, 2026-09-16:** S1 landed on main (`ca836e0`, merged from
`soil-cycle-s1-held` once green). S2, deep time measured near against far,
is next. Opened from the
[isoscape family plan](2026-09-16_isoscape_family_plan.md) ruling 20 and its
§2.6 assessment. Deep time's heir entry (D7b) waits on this plan.

**Owns:** making the enclosure's matter cycle work in the worlds the
generation door hands over: far-tier movement, how decomposers reach, sense
and eat carrion, how carrion returns matter with or without them, and how
many decomposers are founded, where, and in what form.

**Does not own:** deep time itself (the isoscape family plan), the heir
entry (D7b there), the multi-anchor network body (the playable ecology
plan's representation ruling), or any other kingdom's feeding rules except
where a step names the effect on them.

---

## 0. Rulings (Mark, 2026-09-16)

Answering §2.6 of the isoscape family plan.

1. **Fix far-tier movement first, in this lane.** A far body inside its
   target's place steps toward the target instead of leaving it, and a far
   body moves at most its dispersal budget per tick, paying per tick what a
   near body pays.
2. **Measure deep time near and far after that fix, then rule.** Isoscape
   ruling 19, everything far during deep time, stands until the numbers are
   in. Its premise was corrected in §2.6: every founder starts near.
3. **Reach, all four:** a decomposer's bite reads its body's reach; a
   scavenger closing on carrion spends its dispersal budget; founding gives
   fauna a sense organ; and decaying carrion leaves a scent in the soil that
   scavengers can follow.
4. **Supply, all four:** carrion decay proportional to mass as a declared
   world rule; more founding decomposers; decomposers founded where corpses
   fall; and sessile or creeping decomposers.
5. **Dispersal beyond creeping, for decomposers and producers alike.**
   "There should be additional dispersal methods other than creeping for
   both myco and flora." Fungi and plants spread by more than a body inching
   across the ground. (In this plan "producers" and "plants", since the bare
   word *flora* is reserved platform-side; see `mesocosm/CLAUDE.md`.)
6. **Far perception is limited to sight, as near bodies' is** (after S1
   showed whole-enclosure far perception overgrazing producers once far
   bodies could arrive).
7. **The far tier runs near's rules with cheaper geometry.** Every near
   search and feeding rule (sight, the hunger gradients, approach, the
   budget, the wander) applies to far bodies; far differs only in having no
   ray casts and no brick kinematics. S1 lands once that parity holds, with
   every remaining test change named and measured. Ruled after limiting far
   sight left far consumers eating their own line: 150.8 g of 157.6 g of far
   predation on seed 7 at 200 founders, because a far body with only kin in
   sight had no kin-skipping gradient to follow.
   *Prior art, recorded 2026-09-16 from Mark's reading of Veloren:* RTSim
   reached the same rule. Its distant simulation keeps the rules and reduces
   the update rate, in plain ID-addressed tables rather than the gameplay
   ECS, with an explicit authority boundary between the two
   (https://docs.veloren.net/veloren_rtsim/). When the scale plan's cohort
   rung is taken up, that is the discipline to copy: reduce the rate, never
   the rules.
8. **Classic stabilizing mechanisms join this plan** (§3a), each assessed
   and measured before code: a consumer functional response; density limits
   above producers; dormancy, rescue and Allee effects; and disturbance,
   seasons and a sweep of starting soil richness.
9. **Re-pins for S1, ruled:** the two readings tests take newly measured
   quiet seeds against the unchanged warning threshold, and the embodied
   round test follows the line its fixture matures rather than whichever
   line commits first.


## 1. What the steps start from

Verified in §2.6 of the isoscape family plan, with `path:line`:

- **Far movement**: `graph_step` (`organism/ecology/movement.rs:566-582`)
  jumps to a neighbour place's centre and always leaves the target's place;
  `pay_travel` (`flows/returns.rs:55-73`) charges the jump to body substance.
- **Approach**: `walk_grounded` stops at `reach + GRAZE_RANGE`
  (`movement.rs:551-553`).
- **Bite**: flat `DECOMPOSE_RANGE = 6` (`movement.rs:39,103-116`).
- **Sight**: `sight_for_body(8, sensor_span, ..)` (`rates.rs:287-290`); no
  founding decomposer measured has a sense part.
- **Decay**: 1 mg every `CARRION_DECAY_TICKS = 4` ticks per corpse
  (`rates.rs:89-94`, `flows/returns.rs:76-97`).
- **Founding**: the pyramid is two thirds producers, a quarter consumers,
  the rest decomposers (`world/genesis.rs:520-545`), placed uniformly
  (`:223-226`); the door founds 24 by default (`world/generation.rs:103`).
- **Travel without machinery**: only an actuator span or the producer
  exception lets a body move (`rates.rs:381-383`).

**Instruments**: `mesocosm-core/examples/decomposer_probe.rs` (the first
epoch tick by tick, three arms: deep, near, hand) and
`matter_ledger_probe.rs` (where matter sits at each epoch).

## 2. Steps

Sequential by default: every step changes ecological outcomes, and each is
measured with both instruments before the next, so an effect can be
attributed. Steps on disjoint files may run in parallel only where the
measurement of one does not depend on the other.

| Step | Builds | Where | Model |
| --- | --- | --- | --- |
| **S1** | Far-tier movement and perception at parity with near (rulings 1, 6, 7): no bounce inside the target's place; the dispersal budget per tick, paid per voxel moved; sight as near; every near search and feeding rule, without ray casts or bricks | `organism/ecology/movement.rs`, `movement/perception.rs`, `movement/far.rs` | opus |
| **S2** | Deep time near and far, measured after S1 on seeds 7, 1 and 42 over six epochs: survival by kingdom, soil, carrion, wall time. Ruling 19 goes back to Mark with the numbers | measurement only | orchestrator |
| **S3** | The scavenger's bite reads its reach, and a scavenger closing on carrion spends its dispersal budget (ruling 3) | `movement.rs` | sonnet |
| **S4** | Carrion decay proportional to mass, a declared world rule in `WorldRules` and its digest (ruling 4); the world-state pin moves once, with its reason | `rules.rs`, `flows/returns.rs`, `ecology.rs` | sonnet |
| **S5** | Founding: a sense organ for fauna, a decomposer floor at the door, and decomposers placed where corpses will fall (rulings 3 and 4) | `world/genesis.rs`, the drawn founding | opus |
| **S6** | Soil scent: decaying carrion deposits typed matter that spreads, and scavengers follow its gradient (ruling 3) | `flows/returns.rs`, `perception.rs`, `places/soil.rs` | opus |
| **S7** | Sessile and creeping decomposers (ruling 4), and dispersal beyond creeping for decomposers and producers (ruling 5), under the reading and the methods §3 asks Mark to choose | `rates.rs`, `movement.rs`, reproduction placement | opus |
| **M1** | Consumer functional response (ruling 8): assess handling time, satiation and a low-density refuge against the tree and the measurements, then build what Mark rules | `organism/ecology.rs`, `rates.rs` | opus |
| **M2** | Density limits above producers (ruling 8): assess, then build | `organism/ecology.rs`, `rates.rs` | opus |
| **M3** | Dormancy, rescue and Allee effects (ruling 8), together with S7's propagules, since spores and seeds are both | `breeding.rs`, `organism.rs` stages | opus |
| **M4** | Disturbance and seasons from the world profiles, and a sweep of starting soil richness (ruling 8) | `pressure.rs`, `rules.rs`, genesis | opus |
| **S8** | Re-measure deep time and the matter ledger on all three seeds; hand D7b a world with a working soil cycle | measurement only | orchestrator |

**Order.** S1 (parity), S2 (near against far deep time), then M1 and M2,
which the audit ranks as the likeliest causes of overshoot and overgrazing,
then S3 to S6, then S7 with M3, then M4, then S8. Each assessment step
brings its options to Mark before code.

## 3. Decisions still Mark's

- **S7's reading.** Ruling 4 chose sessile or creeping decomposers, and
  ruling 3 improved walking ones. Proposed reading: both forms exist, set by
  the body. A decomposer drawn without locomotion machinery is sessile and
  creeps when hungry, as producers do; a limbed one walks, with S3's reach.
- **S7's dispersal methods.** Ruling 5 asks for more than creeping and
  names none. Where offspring land is set at birth in
  `organism/ecology/breeding.rs`, so propagules that land away from the
  parent (spores, seeds carried by wind, water or animals, runners) are
  reproduction placement, while creeping is body movement. S7 opens with an
  assessment of which methods the simulation can carry and brings them.
- **S4's rate.** Decay proportional to mass needs a value; S4 measures
  candidates and brings them.
- **S5's numbers.** The decomposer floor, the sense organ's form and the
  placement rule each need a value; S5 measures and brings them.
- **Ruling 19**, after S2.
- **What S1 uncovered: far bodies see everything.** Far perception returns
  true for any position in the enclosure (`movement/perception.rs:302`), so
  a far consumer targets food anywhere. While far movement was broken that
  cost nothing, because far bodies bounced instead of arriving. Once S1 lets
  them arrive, far grazing overwhelms producers (Findings). Whether to limit
  far perception, re-pin the tests, or retune consumer pressure is Mark's.

## 3a. What classic ecology says is missing (audit, 2026-09-16)

Mark asked: "Perhaps we should consider complex cellular automata sims, for
balance ideas. We missing stuff?" A Sonnet agent audited the tree read-only
against the mechanisms the classic models rely on; the orchestrator checked
the claims marked (verified). Paths under `mesocosm-core/src/`.

| Mechanism | In Mesocosm | Where | Classic model |
| --- | --- | --- | --- |
| Consumer functional response: satiation, handling time, low-density refuge, prey switching | **Partial**: a per-tick bite capped by room in the body; nothing responds to how scarce the food has become | `organism/ecology/rates.rs:220-258`, `ecology.rs:303,342-347,386-391` | Holling type II; a type III refuge stabilizes Rosenzweig-MacArthur |
| Density dependence | **Partial**: crowding throttles and thins producers only | `ecology.rs:222-236,316-330`, `rates.rs:57-68` | Verhulst logistic; Gause |
| Allee effects, minimum viable population | **Absent** (verified): reproduction reads only the individual's own stage, gestation and mass | `organism.rs:478-483` | Allee; mate-finding models |
| Dormancy and propagule banks | **Absent**: no seed, spore or cyst stage | `organism.rs` `Stage` | Cohen's bet-hedging; Chesson's storage effect |
| Immigration and rescue | **Absent**: nothing adds organisms after genesis except births | `world/genesis.rs`, `breeding.rs` | Island biogeography; the rescue effect |
| Locality of interaction | **Near present, far broken**: far perception sees the whole enclosure | `movement/perception.rs:302` | Wa-Tor; spatial rock-paper-scissors |
| Disturbance | **Absent** in the simulation; world profiles describe freezes and floods as authored prose kept off the wire | `pressure.rs:38-46` (verified), `:268` | Forest-fire model; intermediate disturbance |
| Seasons and pulses | **Absent**: producer income is a function of mass alone | `rates.rs:188-190` | Sugarscape seasons |
| Microbial loop | **Present** (verified): a background soil process turns typed matter untyped every tick, beside organisms | `places/soil/mineralization.rs:16-47`, `ecology.rs:590` | Microbial loop |
| Enrichment | **Untested**: starting soil is a fixed 100 mg per column | `world/genesis.rs:40,488` | Rosenzweig's paradox of enrichment |
| Self-regulating feedback on growth conditions | **Absent** | none found | Daisyworld |
| Adaptation | **Inert in deep time** (verified): an unplayed line weighs only inherited or discovered candidates, discovery is played-only, so an enclosure nobody plays has empty rounds | `world/adapt.rs:62-68` | Red Queen; evolutionary rescue |

**The audit's ranking of causes for today's measurements:** far perception
(being addressed now); no functional response, so grazing never eases as
food thins; no density dependence above producers, so blooms overshoot;
enrichment and disturbance unaddressed; and adaptation absent from every
headless run measured, which makes those runs a lower bound.

**Two corrections this forces on earlier text.** The isoscape plan's D7a
report said lineage 1 "takes its own adaptation turns" in deep time: it does,
and they are empty. And "micro and myco" already has its micro half in the
tree as mineralization; what is missing is the carrion-to-soil step feeding
it quickly enough, which S4 addresses.

Whether any absent mechanism joins this plan is Mark's.

## 3b. M1 and M2 assessed (2026-09-16)

An Opus agent assessed both read-only after S1, with a new instrument,
`mesocosm-core/examples/response_probe.rs`, whose positive control
reproduces the readings fixture (234 producers at tick 2,000, seed 7, 200
founders). The orchestrator verified the code claims marked and reran the
collapse. Paths under `mesocosm-core/src/`.

**The enclosure collapses on every seed at 200 founders.** Seed 7:
producers 234 at tick 2,000, **0 by 3,600**; consumers peak at 185 at 3,200,
**0 by 4,000** (verified). Seed 42 loses producers by 3,000 and consumers by
3,500; seed 1's consumers peak at 166 at 4,000 and are gone by 6,000. The
2,000-tick fixtures sit mid-overshoot.

**M1, the functional response as it stands.** A body takes **one bite or
nothing per tick** (verified, `organism/ecology.rs:341-350`): the allometric
`feeding_rate_for_body` capped by room in the body, with nothing reading
food density. Measured per consumer per tick against producers within the
8-voxel sight floor:

| Producers in sight | Grazing mg per consumer-tick | Meals per consumer-tick |
| --- | --- | --- |
| 0.23 | 3.15 | 0.15 |
| 0.71 | 6.45 | 0.34 |
| 1.75 | 10.00 | 0.65 |
| 3.12 | 9.15 | 0.60 |

So it **already saturates, a Holling type II**, with half-saturation near
half a producer in sight, and the enclosure lives on the plateau. There is
no low-density arm: at a quarter of a producer in sight, grazing is still a
founder's whole upkeep, so it never releases and producers reach zero. The
one-meal rule is never binding (meals per consumer-tick peak at 0.74).
Satiation partly exists and binds for about half the consumers at tick
2,200-2,400, but room reopens by a tick's upkeep every tick, so a sated
consumer grazes at its rent instead of stopping.

**M1 options.** Handling time (a per-meal delay on `Organism`, adds replayed
state) lowers the plateau, which the measurement says is not where the
problem is. A satiation gate (`intake_room_mg`, `organism/ledger.rs:48-51`)
is already partly there. A **low-density refuge** (food below a local
density or size is not perceived, `movement/choice.rs:49-55`, or not
admitted, `ecology.rs:341-366`; Holling type III, the refuge that
stabilizes Rosenzweig-MacArthur) is absent and is the only option that
changes the left end of the curve. The evidence favours the refuge.

**M2, density as it stands.** The crowd map counts **every living body**
per 8-voxel cell (verified, `ecology.rs:232-236`), and only producers read
it: income divided by the cell's count, floored at rent, as a request on
the soil (verified, `:316-331`). It was scoped to producers because TD6
measured that a closed matter budget bounds a stand's mass and not its
number (`2026-08-29_terrarium_dynamics_plan.md:1733-1743`); consumers were
never asked. Two findings: consumers and decomposers standing in a cell
shade its producers, contributing up to about 17% of the throttle; and the
plan's earlier text calling `Pressure::Crowding` the mechanism was wrong,
that is authored profile data nothing reads.

**Blooms overshoot without ever looking crowded.** Consumers go 29 to 185
while consumers per occupied cell stay between 1.2 and 1.9; the bloom is
occupied cells (23 to 103), not packing. Local packing appears only at tick
2,800, when producers are already falling. **Any density term on the
8-voxel cell would have been silent through the whole bloom.**

**M2 options.** Crowding on rent (`rates.rs:340-349`); on reproduction
(`organism.rs:478-483`, `breeding.rs:60-65`); a per-cell birth ceiling
(`breeding.rs:120-140`, Gause's K, Wa-Tor occupancy); or **interference**,
feeding efficiency falling with neighbours on the bite (`ecology.rs:342-346`;
Beddington-DeAngelis), which attaches to the flat per-capita grazing the
measurement shows driving the collapse. Whichever is chosen, the
neighbourhood it reads should be the bite's own (`GRAZE_RANGE + reach`, or
the 4-voxel sensory buckets), not the crowding cell. The evidence ranks M1
as the binding constraint and M2 the weaker step for this collapse.

**Decisions that are Mark's:** which M1 term, what a refuge keys on and its
threshold, whether M1 may add replayed state to `Organism`, and whether its
success is judged at 2,000 ticks or at the 4,000-tick horizon; which M2
term, what neighbourhood it reads, whether decomposers get it, whether
non-producers keep shading producers, and whether M2 waits for M1 measured
alone.

## 4. Done conditions

1. **S1.** A far body in its target's place approaches the target and does
   not leave; no far body moves more than its dispersal budget in a tick;
   travel paid equals voxels moved; in `decomposer_probe`'s deep arm no
   founding decomposer dies on a tick it hopped; far-tier cohort
   conservation still holds; every existing core and runtime test passes or
   a changed expectation is named with its reason.
2. **S2.** A table of near against far, per seed, per kingdom, with wall
   time; ruling 19 answered.
3. **S3 to S7.** Each lands with both instruments rerun and its effect
   stated against the previous step.
4. **S8.** On all three seeds, after six epochs of deep time, decomposers are
   alive, carrion holds less than a stated share of matter, and the
   generation door drafts candidates. The share is proposed with S8's
   numbers, not fixed in advance.

## Findings

- **2026-09-16.** Opened with §2.6's evidence.
- **2026-09-16, S1.** Far bodies now step straight toward their target, one
  column at a time on the ground's surface, within `dispersal_for` counted
  in three dimensions from where the tick began, never farther from the
  target and never out of the target's place once inside it; travel is paid
  per voxel moved (`movement/far.rs`, new; `graph_step` removed). The far
  wander is one voxel, as the near wander is, and a far body stops
  approaching at `reach + GRAZE_RANGE`, as a near body does. Eight new tests
  pass, including cohort conservation and determinism on generated ground;
  the six pins hold.
- **2026-09-16, S1's effect on decomposers in deep time** (24 founders):
  every founding decomposer now dies at tick 16 to 310, never on a tick it
  jumped, instead of tick 4 to 36; scavenging in the epoch rises from 18 to
  306 mg on seed 7 and from 0 to 1,165 mg on seed 42, and stays about 45 mg
  on seed 1. None is alive at epoch 6 on any seed, before or after.
- **2026-09-16, S1's effect on grazing.** Five existing tests fail, each
  because far consumers now reach food: `world/generation/tests.rs:341` and
  `:376` (the habitat trials' subject is eaten early), `tests/embodied/round.rs:174`
  (a different line wins the round), and the runtime's two readings tests,
  where seed 7 with 200 founders over 2,000 ticks goes from far grazing
  49,812 mg to 519,330 mg, far scavenging 180 mg to 121,622 mg, and
  producers 472 (917,431 mg) to **6** (17,418 mg). Far moves went from 1,665
  averaging about 42 voxels to 19,671 averaging about 1.6. The old values
  were measured against the broken movement.

## Progress

- **2026-09-16.** Plan written; S1 dispatched.
- **2026-09-16, parity (S1c).** Near and far differed in thirteen places;
  five were rules and were brought to parity (target choice and fauna
  policy, sighting memory, the hunger gradient, the wander, producer creep),
  the rest geometry and kept. Seed 7 at 200 founders: own-line far predation
  150,757 to 3,101 mg, producers at tick 2,000 47 to 234, cost per body-tick
  11.9-12.9 µs against main's 10.7-11.3. Deep-time decomposers still die in
  the first epoch. A far newborn keeps its raw scatter column rather than a
  ground stance (`breeding.rs:165-170`), noted and left.
- **2026-09-16, re-pins.** Quiet seeds measured 1-40 plus the old four:
  seed 7 peaks at 94 ticks and 99 at 116, both over the threshold; the list
  became 1, 18, 31, 36, 555 and the control arm seed 31 (neutral 0,
  stressed 166). The round test reads `round.turn(grown)`, the line the
  fixture matured. Core 773, runtime 80, pins hold.
- **2026-09-16.** Far sight limited on the held branch (uncommitted there):
  far bodies see `sight_for_body` as near bodies do, by distance without a
  ray. Two of the five failing tests pass unchanged; far consumers then ate
  their own line (150,757 of 157,626 mg far predation, seed 7, 200
  founders); the readings pair and the embodied round still fail. Rulings 7
  and 8 followed; parity dispatched.
- **2026-09-16.** M1 and M2 assessed (§3b), `response_probe.rs` added;
  their rulings wait on S2's table so Mark is asked once.
- **2026-09-16.** S1 landed on main as a merge of the held branch after
  parity and the re-pins made it green, with the full gates rerun in the
  main tree.
- **2026-09-16.** S1 built by an Opus agent, which stopped at done condition
  1 rather than edit the five failing tests. Verified by the orchestrator:
  the eight new tests pass and exactly those five fail. Main is kept green:
  S1 is committed on the local branch `soil-cycle-s1-held` and main's tree
  restored.
