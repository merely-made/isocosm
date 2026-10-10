# Directing and the interim M4: Mesocosm's first played loop on Isocosm

**Date:** 2026-10-08

**Status, 2026-10-10:** D1 to D5 are built on native Isocosm under 732's
compile gate. M3 and the interim M4 belong here (800); the after-pass still
must certify their invariant and headed done-conditions, reproduction first.

## Current state, 2026-10-10

`shared/isocosm/src/directing/` owns participants, bonds, nudges,
deliberative choices and readings. `directing/interim/` in that same
source root owns the site-grain loop, birth/death and boundary cuts,
resume and deep time. `shared/isocosm/src/lineage/` owns review,
revision and speciation. Mesocosm's contract envelopes lower in
`mesocosm/crates/mesocosm-runtime/src/runtime/translate.rs`; the host
reads native bodies and the lifted site, rather than a legacy world.

The switch and its follow-ups in Progress are implemented; the older
“D5 waits” and “no phase opened” lines are history. The remaining gate is
certification: pre-D1 saves, non-deliberative hashes, bond and zero-bond
controls, tier merging, and three epochs in both modes with replaying
headed receipts. Consumer reproduction is the prerequisite (792).
Boundary ordering still uses living-member count in this snapshot;
797 requires descending native metabolic complexity and is separate
from the already-built switch. Full in-site M4 stays in the overlay plan.

*The dated design and implementation entries below remain as written (793).*

**Status, 2026-10-08:** plan. Assessment done and its forks ruled (681 to
691); no phase opened.
**Status, 2026-10-09:** D1 to D4 built on native Isocosm under 732's
compile gate (lane `lane-directing`), each phase's done-conditions left
to the after-pass; D4 stands on three open forks (below). D5 waits. *2026-10-10:* D5 landed under the compile gate (Progress). It carries the
[Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md)'s M3 and the
interim M4 of ruling 680, and the full M4 stays there.

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
- Wounds feeding mood (227) wait on checkpoint 10; D2 reads needs and
  mood without them.
- Whether a participant entity can be shared by games beyond Mesocosm
  (the VTT's players, Eponym's sophont player).
- **2026-10-09, D4's forks, put back by the directing lane:** what
  habitability for a critter reads natively (the runner is built; the
  start is the world's deep time plus the player's epochs meanwhile); what
  a native lineage's boundary candidate is; which acts move a body between
  sites, against starting again elsewhere by `Take` (`OnCollapse`); and
  whether a region that never held a level has collapsed.

## Findings

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

- **2026-10-10, Q3 prey timing under 454 and 683:** five feeding controls
  pass, including the optional-act mismatch, with Individuals and Grouped
  scheduled saves replayed in both modes. Existing choice (six), feeding
  (ten) and scheduler (seven) controls pass. The compiled disabled-fix
  control fails on the missing meal; source restoration matches its prior
  SHA256. The restored positive control passes, and the native offline
  workspace/all-target check passes in 10.87 seconds. No rate changes
  or new reproduction sweep are included; A1 and headed certification
  remain open.
- **2026-10-10, Q3 mechanism probe:** seed 0's actual scheduled/planning
  receipts and held trajectory were traced through extinction at tick 100,
  with complete raw output retained by the sim owner's Q3 Progress.
  The temporary observer was removed. A1's new cadence was rejected on
  its complete bare arm; neither this diagnosis nor the account/806
  checkpoint closes reproduction or the native/headed certification gates.
- **2026-10-10, ruling 806:** the native boundary's lineage complexity
  aggregates distinct expressed catalogue functions and realized carried
  systems across all living members. Q3 implements this order alongside
  consumer reproduction; no certification is claimed by this decision.
- **2026-10-10, Q1:** current ownership, gates and status checked against
  the lane tree under 793 to 802; earlier dated text retained. No new
  compile, test, draw or headed receipt claimed.
- **2026-10-10, Q3 lane checkpoint (797, 806):** descending native living
  complexity replaces member count for boundary turns. The distinct varied
  union and order controls pass; anatomical scheduled and deliberative
  ceilings, meals and generic births pass the sim owner's focused checks
  and offline all-target check (its Progress). The configured reproduction
  gate stays open. Integration pending; this does not certify D5's headed
  run or the stock year/epoch.
- **2026-10-09, D1 to D4 under 732 (lane `lane-directing`).** All in
  `shared/isocosm/src/directing/` unless named.
  - **D1:** a participant is an entity of `kingdom:participant` at
    `PLACELESS`, rooted, never a target; `Relation.value` (skipped at
    nought; one relation per subject, kind and object, `schema::related`
    and `directing::hold`); the bond `directing:bond` and the played
    critter `directing:plays`; `Command::{Join, Take, Nudge}`, a nudge
    naming `Toward::{Site, Thing}` with `Aim::{Attend, Act}`, kept in
    `State.nudges` with its `Answer`; `Rules.directing`
    (`rules/directing.rs`): `Inheritance::{Fresh, Seeded, Lineage}`, Seeded
    the default, and the bond's step, span and sway.
  - **D2:** `choice.rs`: a `Method::Deliberative` critter takes one due
    Choice process, scored by the pressing needs it feeds (scaled up by low
    mood), the sway of each live nudge it answers (`sway × bond / most`) and
    its priority; the scheduler skips the rest, which `Receipt.foregone`
    lists. An answered nudge moves its bond a step up if the act left the
    critter no worse in mood and matter, down otherwise. `Founding.played`
    (`found.rs`) makes a generated lineage deliberative and gives the world
    a mind with its hunger and each trophic level's slot. Ruling 742: `Tier`
    and `TierLine` live in `tier.rs`, over site hops; legacy places
    re-exports them and keeps its `tick` adaptor.
  - **D3:** `readings/`: `suggestions` (the would-be `foregone` now,
    filtered through the survival view), `orders` (range, home, priorities,
    stances, places to avoid, from `State.nudges` and their answers),
    `regions` (grown from the lowest unclaimed site along routes while every
    slotted level fits; collapsed where a slotted level reads nought) and
    `view` (`Mode::{Survival, Creative}`: survival is the critter's site,
    visits, the sites and subjects of events it `knows`).
  - **D4:** `interim/`: `run_deep_time` (hagiograph over `Session`, an
    epoch per call), `boundary::{score, adapt}` (grow-a-copy over
    `fork_at`, the played line skipped, most numerous line first), and
    `Interim` (found, rounds, births offered with the parent kept, death
    handing the next life at its site first, boundaries, collapse with an
    `OnCollapse` setting, both modes).
  - Checks: `cargo check --workspace --all-targets --offline` green in
    `shared/isocosm`, `mesocosm`, `eponym`, the root, `isocosm-overlay`
    and `isometer`; the lane's 21 unit tests pass. Nothing else was run.
  - **2026-10-09, rulings 751 to 754 (lane `lane-directing-2`).**
    751: `readings::habitable` (the conditions the lineage's
    trait-bound processes require, and a living member); `Start { within,
    epochs }` runs the world's deep time, then epoch by epoch until a site
    is habitable (refused past `within`), then the player's epochs, and the
    first life is taken at a habitable site. 752: `Command::Revise` and
    `revise.rs`: a variant bears a learned kind the recipe lacks on one
    tagma, the lexicon unchanged, at most `Directing.variants` (4) a line;
    `revise::revisions` is the boundary's candidate source. 753: a played
    founding gives every line a hunger need and every route a
    `move:<from>-<to>` Choice process, open at its site (condition
    `site:<id>`) while mood is low; `OnCollapse::Elsewhere` stays. 754: a
    region's `held` levels are read from the founding and every body's
    place and visits, and it has collapsed only where a held level reads
    nought.
  - **For the after-pass**, beyond §3's done-conditions: a real pre-D1 v3
    save fixture (the tests show only that a plain run saves no new field);
    hashes of every non-deliberative run before and after D2 (equal by
    construction, `Deliberated::Free`, unmeasured); draws, not the fixed
    seeds the unit tests use.

- **2026-10-10, D5 under 732 (lane `lane-meso-world`).** The switch landed
  at `91f2e050` under the compile gate; 680's headed condition is the
  after-pass's.
  - *The runtime* (`mesocosm-runtime`):
    - `Runtime` drives `Interim` one round a tick from a generated world with bodies and a played lineage.
    - It translates contract envelopes into native commands (686):
      - a nudge is `Command::Nudge` (an act key is dropped at site grain);
      - the player's act is `Speciate`;
      - a birth answer keeps the parent or takes the child, and a death answer takes up an heir;
      - a review answer commits `lineage::Review`'s offer, where offer 0, the status quo, closes the turn;
      - `EndEpoch` advances to the boundary, and `PlaceMatter` places `world:soil` at the played site;
      - `ForceBirth` and `Kill` are refused, since native has no command for them.
    - Checkpoints are read from the interim's happenings (762), and the review is `lineage::Review`.
    - The glyph reading runs on `effects::Journal` (774), reading carves, moves and intake from the log and flows.
    - The flow windows read native flows. A run's record is its session save, replayed by `Session::load`.
  - *The views* read the native world:
    - vitals (holdings, `kingdom::of`, feeding mode, the graft allowance);
    - the board over `Offer` and `Reading`;
    - the minimap over sites, laid on a ring;
    - the follow and part tiles.
  - *The host* (`mesocosm-genet`):
    - The section draws the played critter's site lifted through `Simulation::volume` (isometer-space), with the bodies of that site's members at presentation spots.
    - Input directs: E attends, Q acts, S speciates, and the checkpoint and board keys answer.
    - Legacy driving is removed (671, 679), along with the legacy creator, the specimen bench (the native sim panel stays), hand-driven eating, grafting and expression, the fixture scenes and the receipt examples.
    - Genet went from 32,278 lines to 8,147.
  - *For the after-pass:*
    - 680's headed condition: three epochs, both modes, receipts replaying;
    - a founding with bodies and a played lineage actually starting;
    - consumer reproduction (761);
    - the retired tests' behaviour as draws (672);
    - body scale against ground cells;
    - watching a replay in the host, which needs a `Runtime` resumed from a save.

- **2026-10-10, the switch's follow-ups, rulings 781 to 784 and 786
  (lane `lane-meso-follow`, under 732's compile gate; nothing run).**
  Landed at `03f87b98`.
  - *Dev commands (782).* `Command::ForceBirth` runs the parent's birth
    process, its account needs issued by the dev source. `Command::Kill`
    kills where the member stands. Both label the run assisted.
  - *Act keys (784).* A nudge carries an optional act key, a process key
    that the choice honours. The contract's `PlaceMatter` names a site
    handle.
  - *Placement (783).* `map::position` gives a site's grid cell, and the
    runtime's founding lays a 3 by 2 grid. The runtime gives each unplaced
    cohort at the played site a patch of the lifted window by
    `Command::Patch`. Genet draws a body at its patch, and the minimap draws
    each site at its grid cell.
  - *Resume (786).* `Interim::resume` and `Runtime::resume` take a save up
    where it stands, and genet's `--watch` plays on from it.
  - *The authored door (781).* At a boundary, the runtime runs the pack's
    scripts for every declared-tract offer on a copy of the played body,
    through `Allocation::commit`, and the board shows the result beside the
    row.
  - *Reading, not ruled:*
    - the authored cells are shown and never sent, since native has no
      command that places cells exactly;
    - placement gives one patch per cohort, round-robin over the window's
      patches;
    - a resumed run takes the save's first participant and creative mode.

- **2026-10-10, ruling 787 (lane `lane-express`, the after-pass open,
  789).**
  - `Command::Express { entity, part, tracts }` places a part's cells
    through `mosaic::propose`, all or nothing. Each function must be in
    the catalogue and the part must be living. It moves the body revision.
  - Committing a review offer sends the first holding script's placement
    after the offer's own commands.
  - Tests:
    - isocosm's mosaic, directing, lineage, process and kingdom unit tests:
      50 of 50, the two new `Express` tests among them;
    - mesocosm-runtime: 21 of 21.
  - Two stale pins of the five-native registry were updated to 759's
    fifteen, read from the catalogue. The tactile fixture's capsule is
    raised above the isometer fixture's relief.

- **2026-10-08:** plan drafted from the directing survey; rulings 681 to
  691 taken in three rounds.
