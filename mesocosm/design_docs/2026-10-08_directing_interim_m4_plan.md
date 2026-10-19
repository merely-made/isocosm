# Directing and the interim M4: Mesocosm's first played loop on Isocosm

**Date:** 2026-10-08

**Status, 2026-10-18:** D1 to D5 are built on native Isocosm and the switch has landed (`91f2e050`). Integrated since: native metabolic boundary order (case 806), pass-start prey resolution (case 454), the empty-choice retry (case 813) and seeded movement ties (case 958). All nine workspace compilers and all 51 directing controls pass, and the seed-9 pilot reaches three boundaries with one born consumer living. A1 consumer reproduction passed its unchanged 20/20 gate on 2026-10-17 (see the sim plan's Status). Open: the D1 to D5 invariant certification; and the headed three-epoch run in both modes (case 680). Next: that certification.

Rulings live in the [wing design record](2026-09-18_wing_design_plan.md);
this plan cites them by number and does not restate their reasoning.

**Ownership, 2026-10-10 (ruling 800).** This plan owns M3 and the interim
M4; the Mesocosm overlay plan keeps the profile, the contract and the full
M4, and points here. M3's done-condition, moved here from the overlay plan
in its words of 2026-09-25: "Done when a played critter acts on its own
needs, senses and mood under the player's nudges, its range, home,
priorities and stances grow from that attention, its bond moves with
outcomes and passes across generations as the world setting says, its
suggestions surface, and a seeded run replays to the same hash." D1 to D3
build it; D4 and D5 are the interim M4.

## 1. What this plan does

It builds directing (194, the overlay plan's M3) on native Isocosm and then
the interim M4 (680): a first played loop at site grain, before places, with
rounds, births, boundaries and a collapse, its receipts replaying. The
interim M4 is the switch (681): genet's runtime moves onto a native `Session`
there, legacy driving ends (671), and in-site play (carve, deposit, voxel
sight) waits for the full M4.

Rulings carried:

| Ruling | What it settles here |
|---|---|
| 177, 178 | a critter follows its player by its bond; the bond's inheritance is a world setting, seeded by default |
| 194, 679 | directing only on Isocosm; the six driving intents are the critter's own acts, `Idle` goes |
| 216, 59 | standing orders grow from attention, as desire paths |
| 225, 226, 227 | a region collapses when a trophic level is gone; regions follow biomass; mood is read, never kept |
| 671 | a critter that is not sapient is directed, never driven |
| 680, 681 | the interim M4 at site grain is the switch |
| 682 | M3 plays a native generated lineage |
| 683 | a deliberative methodology makes the critter's choice |
| 684 | the interim boundary ports grow-a-copy scoring over `Session::fork_at` |
| 685 | a suggestion is a reading over `Receipt.foregone` and `knows` |
| 686 | the runtime translates contract envelopes into native commands |
| 687, 689 | regions group sites by biomass, derived each round |
| 688 | the bond is a weighted relation to a placeless participant entity, one per player |
| 690 | standing orders are read from the logged nudges and their outcomes |
| 691 | survival mode filters through `knows` and the reach field |

## 2. What exists (survey of main at `a02b680a`)

- No player, played entity or participant anywhere in native Isocosm.
  `isocosm-overlay` (1,256 lines) is depended on by nothing.
- The scheduler (`schedule/mod.rs`, 348 lines) runs every due process for
  every ready group and skips `Method::Inert`. `Method::Deliberative` and
  `Normative` exist (`schema.rs:30-31`) and nothing uses them.
  `Receipt.foregone` (`execute.rs:136`) already lists the Choice processes
  passed over.
- `rules/mind.rs` (36 lines) has `Mind` and `Need`; mood is read from needs
  (`meaning/mod.rs`). Generated worlds set `mind: None` (`generate.rs:285`).
  The `sim:attention` account (`AccountKind::Attention`) is created and
  never used.
- `Relation` is `(subject, kind, object)` with no strength (`schema.rs:194`).
  The world is already a placeless entity of `kingdom:world`
  (`generate.rs:108`).
- `history::Command` has `Act`, `Inspect`, `Release`, `Collect`, `Learn` and
  `PlaceMatter`; no nudge. `Session` saves v3, loads v1 to v3, forks at a
  tick and merges.
- `Effect::Move` along site routes writes `visits`; no generated process
  uses it.
- `DeepTimeSpan` and `deep_time_ceiling` (`rules/epoch.rs`) are validated
  only: no native deep-time runner.
- Genet's W4 bench (`mesocosm-genet/src/app/bench/`, 8,288 lines) already
  drives a native `Session` beside the legacy world. The legacy runtime is
  wired through 81 genet files importing `isocosm::legacy` and 77
  `Intent::` uses.

## 3. Phases

Each phase is verified in its own worktree (tests pass, controls fail where
they should, replay hashes check) before it touches main.

**D1. Participants, the bond and the nudge.** A placeless participant
entity; `Relation` gains a value (read as zero from older saves); one
weighted bond relation per participant and critter (688); a logged native
`Nudge` command (686) naming a place (a site at this grain, 690) or a
thing, to attend or to act; 178's three settings in the rules, seeded by
default. Done when a seeded run with nudges replays to the same hash, a v3
save from before D1 still loads, and in the bond's controls outcomes that
serve the critter raise the weight and outcomes that harm it lower it, over
draws.

**D2. The deliberative methodology (683).** A scheduler methodology on
`Method::Deliberative` that, for a deliberative critter, chooses one
Choice process by needs, mood, nudges and bond, the rest going to
`Receipt.foregone`. Generated worlds give the played lineage a mind. Done
when an un-nudged critter acts on its own needs; a nudge moves its choice
in proportion to the bond, and with a zero bond it moves nothing (the
negative control); every other methodology's runs are unchanged (their
hashes before and after match); and choice is deterministic under the
seed.

**D3. Readings.** Suggestions (685), standing orders (690), regions (687,
689, with each trophic level's slot capacity a world rule and regions
growing along site routes) and the survival filter (691), each a pure
function of state and log. Done when each reads the same from a replayed
run as from the live one, a planted fault in each is caught by its test,
and regions merge into one as a draw's biomass falls.

**D4. The interim loop on native.** Founding from a seed, a start the
player picks (179), rounds and epochs, a birth keeping the parent by
default (183), death handing the next life from the cohort (61), the
boundary where every lineage adapts by grow-a-copy scoring over `fork_at`
(684), a regional collapse (225) played on elsewhere, survival and
creative modes. Done headless, by draws: three epochs end to end from
seeds nobody chose, receipts replaying to the same hash, controls for a
collapse that does and one that does not happen.

**D5. The switch (681).** A runtime over `Session` in genet, translating
contract envelopes (686); the section, HUD, succession and review lanes
re-pointed to it; legacy driving removed (671, 679). Done when 680's
condition holds in the headed host: three epochs end to end at site grain
on Isocosm, both modes, receipts replaying.

## 4. Open, to be put when a phase reaches it

- D4's player-picked start needs a native deep-time runner and a
  habitability reading for the critter; neither exists.
- Playing on after a regional collapse needs some generated process to use
  `Effect::Move`; which acts move a body between sites is unruled.
- Wounds feed mood (227) since 2026-10-18: played worlds carry a wound need,
  strain and relief, and rest; see the sim plan's Progress.
- Whether a participant entity can be shared by games beyond Mesocosm
  (the VTT's players, Eponym's sophont player).
- **2026-10-09, D4's forks, put back by the directing lane:** what
  habitability for a critter reads natively (the runner is built; the
  start is the world's deep time plus the player's epochs meanwhile); what
  a native lineage's boundary candidate is; which acts move a body between
  sites, against starting again elsewhere by `Take` (`OnCollapse`); and
  whether a region that never held a level has collapsed.

## Carried from the retired process plans (2026-10-18)

- Remove the test-only scratch diagnostic `directing/interim/diag.rs` (a
  consumer's income, upkeep and age, left by the reproduction lane), or move
  it to an example. (from lane queue plan §4)

## Findings

- **2026-10-16, A1 seed9 (936):** actual pass-start observation confirms
  the chosen feeding process stays committed while all resident producer
  targets depart before its pass. Replacement prey resolution returns none;
  all 64 meals block. Next tick two moves tie at zero and all 64 consumers
  take the first process ID. All 13 pass-start and 5 feeding controls pass.
  The unchanged full 162-tick extinction and bounded-window replay receipts,
  observer limitations and pending movement-policy fork are recorded once
  in [the after-pass Findings](archive_docs/2026-10-18/2026-10-10_after_pass_plan.md#findings).
  No behavior, cadence, rate or acceptance gate changes here.

- **2026-10-11, Q3 fresh bare seed zero classified:** recorded once, in [the after-pass Findings](archive_docs/2026-10-18/2026-10-10_after_pass_plan.md#findings).

- **2026-10-10, Q3 actual Choice/Plan/Act diagnostic, verified:** unchanged
  bare meal3/birth14 seed0 reproduces the rejected125-tick baseline exactly,
  including71 native births and four ordinary receipt hashes. Fresh original
  seed0 ends41 with0 births. Every tick conserves and both saved replays pass
  in both arms. At111 all37 first choices cache None/options={} before a later
  Move pass brings74 edible producers/225 matter; actual feed is Foregone.
  This is separate from68 first choices whose scored prey moved earlier in
  the same unframed Move pass (60 at93,8 at123), violating454's pass-start read.
  Candidate81..125 has79 accepted meals/170 own units at81,407 Foregone plans,
  329 NoTarget blocks and no accepted zero-share meal. Original's16 Foregone
  plans explicitly choose upkeep20 over feed10 with food already eligible.
  **Reading, not ruled:**683 chooses one process, but does not settle whether
  an empty first cache is final for the tick. Keeping the empty commitment
  and retrying only an uncommitted None at a later pass are separate defended
  policies; root owns any numbered decision. A Move frame alone is not proved
  sufficient because lazy body preservation and target-index candidates must
  agree. No policy/rate/source repair or matrix was chosen by this diagnostic.
  All173364 observer rows remain non-authoritative. Six original files restored
  exactly, two archived helpers removed;267 original input bytes match before
  and after the24.125s offline-j1 native all-target gate. Cargo released. Receipts
  use external `q3-reproduction-meal3-birth14-supply-trace` stem; A1 remains open.

- **2026-10-10, the fork resolved by existing ruling 454:** its text says
  hunters draw prey from the pass's start, while 683 chooses one process
  and never binds automatic prey for the entire tick. The bounded correction
  keeps the process choice, resolves automatic prey once at the feeding
  pass and reuses that resolution for planning and visiting. A Thing/Act
  nudge keeps its identity; live answered nudges are checked against the
  eventual target and actor's place before any bond outcome. This applies
  to targeted `Eat` processes; other targeted acts retain their existing
  behaviour. No movement priority or generator rate changes accompany it.
  The five passing native controls cover moved automatic prey,
  explicit prey and bond, stale attention answers, one process per tick and
  one resolution per pass, and a nudge naming another act, with both-mode
  scheduled replay. Feeding's explicit target candidate respects that
  optional act key; other targeted processes retain their existing scope.
  Disabling the prey resolution compiled and failed the moved-prey control:
  the consumer took zero acts rather than its one meal. The production
  source was restored byte for byte. The existing choice, feeding and
  scheduling checks pass too; this is a correctness checkpoint, and A1
  remains open.
- **2026-10-10, Q3's cached-target fork:** actual seed-0 feed planning
  receipts repeatedly refused `no target satisfies the declared scope`:
  the played actor and its chosen living producer were in different sites
  by the feeding pass. `choice.rs::deliberate` keeps the chosen process and
  target for the tick; `schedule/mod.rs` executes ascending priority, so
  generated migration at priority 0 precedes feeding at priority 10.
  The automatic members' multiple moves described below can invalidate an
  earlier eligible food target; `execute.rs` correctly rejects that later
  invalid scope. This explains a concrete intake loss in addition to rates,
  without relaxing target eligibility or blaming anatomical credit.
  *Reading, not ruled:* the source/receipt-supported options are to resolve
  automatic food targets at their chosen process's pass while preserving
  explicit target nudges, to put generated movement after feeding, or to
  keep the stale commitment as intended risk and tune around it. Q3
  recommends the first and holds the choice for the wing's ruling.
- **2026-10-10, Q3 after 797 and 806:** native boundary turns now descend
  by the union of distinct catalogue functions expressed and carried
  systems realized across living lineage members. Varied members can add
  different names. Cohort size no longer substitutes for metabolic
  complexity; extinction filtering and lineage-key ties are implementation
  readings. Native anatomical need ceilings, meal deposition and generic
  birth payment/parentage are repaired in the sim owner (its Q3 Findings).
  The declared 60-tick-epoch reproduction proof and its rejected first
  profile are tracked under after-pass A1; the historical eight-tick test
  below does not certify reproduction or the stock 525,600-tick epoch.
- **2026-10-09:** a generated ecology's consumer cannot reproduce: its
  birth needs three founded bodies of matter, it nets about one unit in
  eight ticks, and age takes it within 20 to 60. So a played consumer
  lineage ends within a lifespan, and D4's three-epoch test runs epochs of
  8 ticks. The world family's re-expression, not directing, settles this.
- **2026-10-09, after 751 to 754:** within one tick a reactive member can
  take several moves in turn, each move's pass seeing it at its new site;
  travel time is not charged. 754's "or that the world holds within reach"
  is not built: every reading of it tried brings back the first-round
  collapses 754 removed. Generated consumers still cannot reproduce; no
  ruling sets the generated ecology's rates (447 and 518 govern bodied
  reproduction by provision), so it goes back as a fork.
- **2026-10-09:** at site grain a region that never held a level reads as
  collapsed (225 says "a level gone"); with slots of two sites' worth, most
  generated founding draws show such regions on their first round. The
  tests use a region as wide as the world for the standing control.
  Open, below.
- **2026-10-09:** native `Effect::Move` has a fixed destination, so a
  generated move needs a process per route; and no native operation
  revises a lineage, so the boundary has nothing to commit
  (`interim::no_candidates`). Both are D4's forks.
- *Reading, not ruled:* the deliberative scorer's terms (needs fed × mood,
  nudge sway, priority, identity order for ties); "served" as no worse in
  mood and matter; the deep-time span the world's own plus the player's
  added epochs; the boundary's initiative by members, legacy's complexity
  having no native reading.

- **2026-10-08:** the survey's counts above, spot-checked: no manifest
  names `isocosm-overlay` but its own; `Deliberative` and `Normative` occur
  only at `schema.rs:30-31`; `generate.rs:285` sets `mind: None`;
  `sim:attention` is created at `generate.rs:122` and read nowhere.

## Progress

- **2026-10-11, journey choices settled (820 to 823):** the traveller keeps
  ageing and paying upkeep, carrying produced soil until a later site deposit.
  Paid birth shares the remaining journey; transit events have a typed journey
  location without site reach; death carries the corpse through the remaining
  duration to the destination. The canonical choices are in the
  [wing record](2026-09-18_wing_design_plan.md). Q3 is preparing native
  residency, scheduling and saved replay. Rates and 454/683/813 stay fixed;
  authentic old archives, the rejected candidate, eighty-world A1 and headed
  certification remain open. Decisions do not constitute native acceptance.

- **2026-10-11, Q3 bitten-part sharing qualified on main:** whole-prey and
  part quotas remain bounded across a pass (454, 459); zero accepted shares
  publish no flow, and partial consumption does not allocate a fractional
  whole part. The eleven-command native batch passed 48 named positives,
  two intended compiled faults, exact restoration and its all-target check.
  All nine consumer workspace compilers passed after the six-path main
  application. Complete root reviews and the preserved reader-format failure
  are recorded in [lane queue Progress](archive_docs/2026-10-18/2026-10-10_lane_queue_plan.md#progress).
  Route-duration journeys still await transit upkeep/birth choices. The
  rejected reproduction candidate, eighty-world A1 and headed certification
  remain open.

- **2026-10-11, ruling 817 and Q3 causal diagnosis:** migration departs and
  arrives after the traversed route's authored travel time; another route
  becomes available after arrival. All 117 missing feeds in the qualified
  diagnostic trace joined to selected prey moving before the feeding pass.
  Native transit state, scheduling and replay remain to be implemented and
  verified. The rejected three-site reproduction candidate remains rejected;
  the bounded bitten-part sharing proposal passed source review (d5b349e0),
  but its native gates and the eighty-world A1 matrix have not run.

- **2026-10-11, combined migration execution:** Route-duration journeys and competing-kernel updates are now applied in the combined migration. Native compiler repair precedes final-state reproduction and replay checks. The former A1 candidates and earlier Q8 statistics retain their historical evidence only; current reproduction and headed D5 remain open.

- **2026-10-17, movement ties (958):** generated played worlds now persist
  `MovementTies::Seeded`; old/absent policies retain identity order and their
  exact prior seed-9 genesis digest. Only equally best due/feasible pure moves
  draw by dynamics seed, creature, tick and tied menu. All 51 directing controls
  pass, including actual cohort movement and per-tick Individual/Grouped hash
  parity. The seed-9 pilot reaches three boundaries with one born consumer
  living and no founders; its two native replay modes match hash
  `c4a092289dd96a06`. A fresh compact played matrix is running; the 20/20 gate
  is unchanged. The after-pass plan records the pilot observer's receipt-cap
  overrun and the enforced compact retention used for the new matrix.
