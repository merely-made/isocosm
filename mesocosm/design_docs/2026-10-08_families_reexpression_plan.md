# Re-expressing the legacy families in Isocosm

**Status, 2026-10-18:** the migration push is integrated: the families are re-expressed on native Isocosm, legacy Mesocosm and Eponym are deleted, and only `legacy/campaign` remains until V2. The Progress entries record compile-gated implementation, not certification. Open: each family's certification by conservation, replay and controls (case 791, which replaced legacy parity); V2 and E3 (cases 798, 794 and 795), which retire the campaign copy; and the plan itself, live until all seven families certify. A1 consumer reproduction passed its unchanged gate on 2026-10-17 (see the sim plan's Status). Next: each family's certification runs.

Ruling 192 had Isocosm absorb `mesocosm-core`, "piece by piece re-expressed
in its process definitions (ruling 32)", Mesocosm's core shrinking to
directing, presentation and the review. Ruling 591 amended the order: the
legacy sims moved into Isocosm first, the games pointing at it and playing
as before, and "families are re-expressed as process definitions inside
Isocosm afterwards". The move landed (`1ffb7294`); this plan is the
afterwards.

## Assessment (2026-10-08)

Surveyed read-only on main `40a31d61`; counts by `wc -l`, callers by the
items each `use isocosm::legacy::…` imports (files and imported items, not
call sites). *Inference* marks what was reasoned rather than read.

**What is there.** `shared/isocosm/src/legacy/` holds 63,277 lines:

- *mesocosm*, 45,230: `world` 15,738 (the enclosure, the sixteen-variant
  `Intent`, genesis, generation, the lineage turn, graft, consume, fixtures);
  bodies in `organism` 7,377, `axis` 3,110, `phenotype` 2,837, `development`
  1,521, `graft` 666, `growth` 330; `places` 2,773 (in-site voxel space);
  matter and processes in `process` 1,033 (the five natives), `flow` 962,
  `matter` 900, `rules` 492, `pressure` 353; lineages in `discovery` 899,
  `species` 732, `program` 632, `deep_time` 413, `score` 242; the record in
  `history` 758, `record` 337, `snapshot` 304, `chronicle` 267; effects in
  `effect_pack` 834, `effect_experiment` 468, `embodiment` 374, `functions`
  195; and `voxel_profile` 394, `cohort` 201, `rng` 88.
- *eponym*, 13,737: `world` (state, combat, technique, session, timed
  actions, anatomy, sheets, movement, navigation and more), `social`
  (epistemic, society, settlement, deeds), `identity`, and `glyphs` 949.
  Eponym's world uses Mesocosm's `places`.
- *campaign*, 4,310: the campaign world and its drafts, generator, factions,
  chronicle, packs, items, construction, overmap, store, map, collaboration.
- Beside it, 13,738 lines of legacy integration tests in `tests/` and 25
  examples, and the founding datasheets the legacy `axis` embeds (626).

**Who uses it.** Mesocosm's `mesocosm-genet` (81 files), `mesocosm-runtime`
(20), `mesocosm-phenotype` (11, pack admission), `mesocosm-views` (11);
Eponym's `eponym-client` (24), `eponym-motion` (7), `eponym-sortie` (7); the
VTT's `isometry-views` (14), `isometry-genet` (13), `isonetry` (16) and
three more, on the campaign tree only; `shared/isometer` (16) and one
wing-integration test. Many Mesocosm imports are `isometer_core` body types
re-exported at legacy paths, which re-point mechanically.

**What Isocosm already has.** Native and legacy code do not reference each
other. Bodies are closest: the native recipe, palette, development, growth
and anatomy re-express Mesocosm's, and checkpoint 6 found rent, fixing and
the grazer's meal matching Mesocosm's formulas act by act; systems
(checkpoint 9), harm (10) and territories (11) are unbuilt, and neither
Mesocosm's `World`/`Organism` nor Eponym's needs and injuries are served.
Matter and processes are partial (the flow record, ported stock and
transport, the epoch rules, the pressures). The record runs in parallel
(native `history`, `journal`, the witness and `reach`). Places have no
equivalent: native space is site-grained, and 591's own evidence was that
"Isocosm has no space inside a site". Effects, the social families, items,
factions, the overmap and storylets have no native counterpart.

**The families and their order.** Mesocosm's is ruled: matter and processes,
bodies, the record, places, lineages and the boundary, then effects (195,
confirmed by 457). Eponym's (239) and the VTT's (247) are ruled in their
overlay plans, and all three predate 591. Places, bodies and the record are
shared between games. *Inference:* one merged order, shared families first:

1. Matter and processes (the ledger, conversions and the five natives;
   Mesocosm's `Registry` becoming rules-pack admission).
2. Bodies (Mesocosm's organism, phenotype, development and axis; Eponym's
   anatomy and bodies), after checkpoints 9 to 11.
3. The record (both histories, snapshots and chronicles, Eponym's
   simulation record and epistemic layer, the campaign's store and facts).
4. Places (Mesocosm's and Eponym's in-site space, the campaign's overmap
   and map), waiting on the place-graph plan's SP3 to SP5.
5. Lineages and the boundary.
6. Effects (effect packs, embodiment, functions, Eponym's glyphs).
7. Eponym's own families (239's order) and the VTT's (247's).

## Done-conditions (per family, by the overlay plan's M2)

A family is re-expressed when it runs under process definitions, conserves
its accounts, replays identically, has its tests ported or replaced by
draws, keeps every file under the 600-line ceiling, and the legacy copy no
longer owns it; where the sim plan certifies a family (as the bodies family
is), its certification passes on draws with its controls.

## Forks for Mark

1. *When a legacy copy retires.* 263: "Build, retire together" at M3; 591 and
   `legacy.rs`: each family "leaves this tree" once re-expressed.
2. *What "playing as today" holds a re-expressed family to:* replay
   identical to the legacy saves and hashes, or parity by draws with legacy
   as the control. Nothing covers legacy save formats (608 covered
   Isocosm's own v1).
3. *One merged order or three* (195, 239 and 247 predate 591).
4. *Where in-site space goes:* a sim noun, waiting on SP3 to SP5 and 343's
   columns, or isometer or the games.
5. *Code the sim's laws exclude* ("the sim never renders … resolves a
   blow"): `voxel_profile`, Eponym's sheets, techniques, timed actions,
   strike resolver and movement (597 kept motion game-side), the campaign's
   store and collaboration, `effect_experiment`, the fixture worlds (15:
   receipts become draws). Re-express, hand back to the game, or delete.
6. *Bodies before or after checkpoints 10 and 11:* Mesocosm's carcasses and
   decomposers need harm and territories.
7. *Directing:* 194 builds it only on Isocosm; whether Mesocosm keeps direct
   driving after its world family moves, or waits on M3.
8. *Parked rungs* (279): glyphs, magic and neighbouring worlds pull no code
   lane yet; whether Eponym's glyphs and the chronicles wait.
9. *The founding datasheets and the two `Founding`s* (625, 626).
10. *The legacy tests and examples:* port, replace by draws, or retire with
    their family.

## Rulings so far

- 666: a family's legacy copy leaves once its re-expression is certified
  and its callers re-point (fork 1; amends 263).
- 667: parity by draws against the legacy implementation's readings, no
  legacy save reader (fork 2).
- 668: one merged order, the shared families first in 195's order, then
  Eponym's (239) and the VTT's (247) (fork 3).
- 669: code the sim's laws exclude is handed back to its game or host, or
  deleted, when its family is reached (fork 5).
- 670: in-site space is isometer's, and the places family takes it through
  isometer, after SP3 to SP5 (fork 4).
- Fork 6 is settled by 668: bodies come after checkpoints 10 and 11.
- 671: non-sapient critters are never driven; Mesocosm's legacy driving
  ends with its world family's move, unplayable until M3 (fork 7).
- 673: glyphs and the chronicles move as code with their families; 279's
  rungs stay parked (fork 8).
- 672: legacy tests become draws or retire with their family (fork 10).
- 674 to 676: a body's geometry goes through isometer and its physiology
  stays the sim's, keyed by isometer's part ids; movement and physical
  reach go through isometer's in-site space, the reach field staying the
  sim's record; this lands with the bodies family, checkpoints 9 to 11
  certifying on the sim's geometry as built.
- 677, 678: the Mesocosm overlay's M2 is this plan's Mesocosm share, with
  the overlay's web condition (260, 267, 283); Mesocosm's world family moves
  last, its re-expression being the switch to M3's directing.
  *Annotation, 2026-10-08:* 681 amends 678: the switch is the interim M4,
  before places, and in-site play waits for the full M4.
- 696 to 703: in-site space through isometer, worked out: the sim records
  edits and place ids, isometer applies them and derives places (696); SP4
  and SP5 and the lift are built in isometer (698, 701), on nisus's store
  once mere's T2 lands (697, 700); walking, routing and sight are rewritten
  over SP5 (702); soil stays the sim's on isometer's columns (703); bodies
  key physiology by `PartId`, situs joining `BodyDocument` (699).
- 704 to 719: checkpoint 10's brief, in the sim plan; 719 has a regrown
  part revive its tombstone, so isometer gains a revival beside `sever`
  with the bodies family.
- 720, 721: isometer's fixtures become its own, and the lineage vocabulary
  leaves isometer-core with the bodies family.

Open: fork 9, the founding datasheets and the two `Founding`s, taken with
the bodies family.

## Progress

- **2026-10-10, ruling 808:** the native record/social seam keeps a deed's
  agreement association in an optional typed field on its event, with
  causal-event validation unchanged. `State::agreements` retains agreement
  authority. Q2's association and replay tests gate the implementation.
- **2026-10-10, family 3 continued, assertions and the campaign (lane
  `lane-assert`, under 732's compile gate; no test sweeps, draws or
  certification run).** Under 768 to 771:
  - *Authored nouns (769).* `Command::Assert` gains places, routes,
    characters, laws and history lines, each its native noun with an
    authored fill skipped when unset: a place a `Site` (no terrain, a fresh
    id from the entity id space), a route a `Route` on both its places'
    sites, a character a placeless `Entity` whose asserted faction takes it
    in as a `member` relation (760), a law an entry in a new `State::laws`
    record (witnessed when present), a history line an `Event`
    `line:<key>` at its place's site and about its first participant
    where those are asserted, placeless otherwise, with strength nought.
    `authored_site`, `authored_entity` and `authored_id` map authored keys
    to native ids. A newborn does not inherit a fill.
  - *The campaign carries its assertions (768).* `asserted::Asserted` folds
    a run of entries by authored key with the legacy insert-once rules (a
    route needs its places; history in the table's time order); it needs
    no world. The legacy `CampaignWorld` now carries
    `assertions: Vec<Assertion>` and their fold, saving and replicating the
    entries only; its factions, places, routes, characters, laws and
    history are readings. `WorldEvent`'s six authored variants became
    `WorldEvent::Assert`; a draft's public events are its entries in order;
    `CampaignWorld::edit` refolds a draft's entries. Deleted: legacy
    `WorldFaction`, `WorldPlace`, `WorldCharacter`, `WorldRoute`,
    `WorldLaw` and `HistoryEvent`, re-pointed in isometry-views,
    isometry-genet, isonetry, isometry-system, isometry-graphshell, the
    chronicle and the faction turn (still legacy, marked for V2, 247). The
    two packs' generators author their worlds as assertion entries.
  - *Eponym's epistemic layer (771)* is not built: it observes Eponym's
    deed log by `SubjectId`, both Eponym's world, so it waits for that
    world's move.
  *Reading, not ruled:* routes, which 769 does not name, follow places
  onto native sites; the fills' tags and claims are sets, so authored tag
  order is not kept; laws are keyed by authored key; a campaign world
  stays in `legacy::campaign` because the faction turn, overmap and
  generator still read it.
- **2026-10-09, family 7, the VTT's campaign (lane `lane-campaign`, under
  732's compile gate; no tests, draws or certification run).** Handed back
  under 669: the host-private `CampaignStore` and the proposal lifecycle
  (`CampaignProposal`, its mode and error), 360 lines, are the VTT's again in
  `crates/isometry-campaign`, where both began; their five unit tests went
  with them, as the VTT's own tests. Callers re-pointed: `isometry-genet`
  (3 files) and `isonetry` (10). Legacy campaign lines went from 4,310 to
  3,950. Nothing else moved, because nothing else has its native
  destination built yet:
  - *the record (pending):* facts and secrets as notes (80, 87, 248), the
    world's history, items bearing provenance (relics, 157), the chronicle
    (673);
  - *bodies (another lane):* characters as denizens (36, 200);
  - *places (4):* the overmap's places and routes as the native site graph
    (72, 599), the campaign maps as the DM's edit over a site's volume
    through isometer (243), with the board-on-isometer plan held;
  - *founding:* the generator and packs as the generator's declared space
    (the VTT plan's §4), native founding being changed by the directing
    lane;
  - *an assertion path native lacks:* the world's factions, places,
    characters and laws, construction, and every "In: assertions" row of
    the contract. Native `Command` asserts none of them; polities come only
    from a process's `FoundPolity` and sites only from a founding layout;
  - *V2:* the tape-drawn faction turn retires there (247) and is not moved,
    so the legacy campaign copy cannot leave in full before V2.
  *Reading, not ruled:* the proposal lifecycle went to `isometry-campaign`
  beside the store rather than to `isonetry`, its only caller, as "its game
  crate" (669). Forks put to Mark: how authored content enters native state,
  and authored factions against polities derived from members.
- **2026-10-10, the world move: Mesocosm's world family onto native, legacy
  Mesocosm deleted (lane `lane-meso-world`, under 732's compile gate; rulings
  755, 759, 762, 770, 773, 774; no tests, draws or receipts run).** Legacy
  Mesocosm went from 40,645 lines to 2,016. Commits `635bffa7` and `91f2e050`.
  - *Native gaps.* The registry carries all fifteen catalogue functions (759),
    the ten without a native process bound to none, under the `mesocosm:`
    namespace. `Interim::round_with_flows` hands the runtime each round's
    flows. The chronicle (`of`, `append`, `read`, bytes) is
    `isocosm::lineage::chronicle`. Legacy `rng` is `isocosm::rng`.
    `kingdom::of` and `rules::Compatibility` have their callers in
    mesocosm-views' vitals. `mosaic::propose` has its caller in
    mesocosm-phenotype's expression door.
  - *Re-pointed.* mesocosm-runtime, -views, -phenotype and -genet run on a
    native `Session`; the directing plan's D5 entry has the detail.
  - *Deleted.*
    - The legacy `world` with its sixteen-variant `Intent`, genesis, generation, the lineage turn, graft and consume.
    - The bodies modules: organism, phenotype, development, axis, graft, growth.
    - The legacy record (770): history, record, snapshot's world half, chronicle.
    - `flowing`, `soil`, `discovery`, `species`, `program`, `score`, `deep_time`, `cohort`, `embodiment` and `rules` (`WorldRules`).
    - 57 legacy Mesocosm integration tests and examples in `shared/isocosm`, and the runtime's, views' and phenotype's legacy-driven tests (672).
  - *Kept for Eponym's legacy world* (Eponym's world move takes them):
    - `places`, with `Scale` moved into it;
    - the `snapshot` codec;
    - the body re-exports Eponym names (`PartId`, `BodyDocument`, `SpeciesId`, `VolumeRef` and the rest).
  *Reading, not ruled:* `voxel_profile` went to mesocosm-runtime beside the
  tactile adapter (669, the overlay plan's §4 reading); the chronicle's
  legacy re-entry (`found`) and generator retired with legacy development.
- **2026-10-10, Eponym's follow-ups and legacy Mesocosm's last lines (lane
  `lane-eponym-follow`, under 732's compile gate).** Under 778 the social
  vocabulary is the rules pack's: `Rules::social` declares crafts (keys
  held as skills) and deeds by key with their weight on trust and liking
  and their phrase; the sim names only the seven deeds asks and agreements
  record, which a vocabulary must declare, and deed processes are made from
  it. Eponym's pack declares Eponym's (`eponym-play`'s `vocabulary`). Under
  779 native `Take` admits a sophont to drive beside a deliberative critter
  to direct; Eponym's player joins as a participant and takes up the
  sophont it plays at each control cut, and its session reads who is played
  from the sim. Legacy Mesocosm's last 1,981 lines served only the
  parry-ground probe: the grown enclosure, its relief and the brick terrain
  moved into the probe as its own fixture, its release run reading as
  before (136 bricks, 69,632 cells, 41,763 voxels, ray 2.5 to 4.5), and
  legacy Mesocosm is deleted. *Reading, not ruled:* an undeclared deed
  weighs nothing and is refused when recorded; Eponym's control keeps its
  log and home beside the sim's `plays`.
