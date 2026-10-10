# Re-expressing the legacy families in Isocosm

**Status, 2026-10-08:** assessment; forks ruled but 9, which goes with the bodies
family (666 to 673).
Comes after checkpoint 9 and the open bugs (wing design record, ruling 656).

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

- **2026-10-09, family 1, matter and processes (lane `lane-matter`, under
  732's compile gate; no tests, draws or certification run).** Moved into
  native Isocosm, legacy copies deleted, every caller re-pointed (the legacy
  world and siblings, mesocosm-genet, -runtime, -phenotype, -views, the legacy
  tests and examples): `process` (the five natives, the definition registry
  that pack admission fills, intake ports, reach) is `isocosm::process`, its
  seeding the function catalogue's `rules::Seeding` (Geometry is Grown; pack
  files still say `geometry`); `matter` (stock, receipts, a receipt address
  naming a member by native `Id`) is `isocosm::matter`; the legacy soil
  transport is gone for `diffusion::percolate`, generic over a column so
  ledgers and stocks share one kernel; `EpochRule` and `DeepTimeSpan` are
  native `rules::epoch`'s, the budget in `WorldRules::epoch_ticks`, placed so a
  timed world encodes and digests as before; `pressure` is native `preset`,
  which gained the worlds' names and parameters. Legacy lines in the family
  went from 3,740 to 1,257: `flow` (962) and `WorldRules` (295, the legacy
  world's own record, leaving with the world family) remain. Unit tests of the
  moved code are native tests; integration tests stay with the legacy world.
  *Reading, not ruled:* registry, stock and flow kept their behaviour exactly;
  converging the registry onto the function catalogue and the flow record onto
  native `flows` are behaviour changes, put to Mark as forks.
- **2026-10-09, family 2, bodies: the 674 seam (lane `lane-bodies`, under
  732's compile gate; no tests, draws or certification run).** Isometer's
  suites stand on their own fixtures (720): an isometer-core `fixtures`
  feature (a seeded rolling terrain, a walker body), a fixture palette in the
  mesh's tests, document-level attachment tests; the two receipts that drive
  the legacy `World` (`dc4_roster`, `render_body`) moved to mesocosm-genet's
  examples; isometer-lens, -mesh and -render no longer dev-depend on
  isocosm. `BodyDocument` parts carry `situs` and a declared `shape` (699),
  and `revive` clears a tombstone whose parent lives (719). Native bodies
  keep geometry in an optional `BodyDocument` on `Entity`, and `Part` keeps
  physiology keyed by `PartId` (674, 699), read through a new
  `geometry` module; anatomy, growth, development, births, systems, the
  stage and the probe read extents, parents, situs and tombstones through
  isometer. The registry is lowered onto the function catalogue (750):
  each native's best-fit roles and seeding read from `default_functions`,
  values and digests unchanged, and `admits` answers every role (492).
  *Reading, not ruled:* a part leaving whole (a bud, an incorporated part)
  leaves its tombstone with its situs cleared, so the recipe may grow that
  place again as before; native documents carry neutral lineage fields
  until 721 lands. Open, put back as forks: how 721's lineage vocabulary and
  699's mass reading leave isometer-core while the legacy world still stores
  them in documents; whether the legacy body modules can leave before the
  world families, which consume them; whether the registry holds all fifteen
  catalogue functions; and fork 9.
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
- **2026-10-10, family 3, the record (lane `lane-record-2`, under 732's
  compile gate; no test sweeps, draws or certification run).** Two steps
  landed.
  - *The flow record (750).* Native `Flow` gained, in native terms, each
    side's lineage and kingdom by key (`Kind`, filled from the population
    when the native sim records a move) and the move's material composition
    (`Composition` and `Conversion` now in `flows/composition`). The legacy
    world writes native `Flow`s through `legacy::mesocosm::flowing` (its
    vocabulary, leaving with that world under 755): `soil` on site
    `ENCLOSURE` (0), `substance` and `reserve` on the organism's id, the dev
    source as `Holder::Dev` made by `PlaceMatter` as native's is. The
    one-tick ledger, the commit point, `Accounts`, `Trend` and
    `WARN_AFTER_TICKS` sit beside it; `Envelope` and `RecordedEvent` moved
    into legacy `history`. Legacy `flow` (962 lines) is deleted; its readers
    in mesocosm-runtime, -views and -genet, the legacy world and its tests
    re-point. *Reading, not ruled:* legacy kingdoms map to native keys,
    producer to `kingdom:flora`, consumer to `kingdom:fauna`, decomposer to
    `kingdom:myco`, and `SpeciesId(n)` to `lineage:n`; a flow keeps holders,
    not the in-site region it happened in.
  - *`Command::Assert` (757).* A native `asserted` family. An authored
    faction is a polity whose constitution starts with no members, its
    governance and focus asserted, carrying `Authored` attributes (key,
    name, tags, claims as authored keys) while it has none (760); it takes
    a fresh id from the entity id space, polities sharing it. An authored
    fact is a note of a kind the world's rules declare, its text the djot
    and its tags in the open envelope, citing `assert:<key>` (80, 82, 85),
    written whether or not the sim advances (248). Re-asserting the same
    content changes nothing; other content under a held key is refused, and
    a refused assertion leaves no entry.
  Open, put back as forks: where a campaign's native world lives in the VTT
  and so how the campaign world becomes a reading of it; the native homes
  of authored places, characters, laws and history lines; whether legacy
  `history`, `record`, `snapshot` and the chronicles move now or with the
  world move (755), being that world's own record; and where Eponym's
  simulation record and epistemic layer go. Nothing of the campaign tree
  moved yet; the tape-drawn faction turn stays in legacy, marked to retire
  at V2 (247).
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
- **2026-10-09, family 5, lineages and the boundary (lane `lane-lineages`,
  under 732's compile gate; no tests, draws or certification run beyond the
  lane's six unit tests).** Native `isocosm::lineage` (514 lines, tests included)
  re-expresses the family over native `Lineage`, keyed by name, reusing
  directing's boundary (684) and revision (752) rather than copying them:
  `tree` is legacy `Lineages`' descent (ancestry, common ancestor, distance,
  descent, children); `speciate` is the naming act, `Command::Speciate
  { founder, name }` founding `lineage:<name>` off the founder's line, the
  record inherited whole and the founder its one member; `program` is the
  line's committed `Command::Revise` entries read from the log, a fork's
  beginning with its forebears' up to the split, with a digest; `reckon` is
  `score::readings` (growth as living matter, spread as sites, endurance as
  the oldest living age), read and never noted; `review` is PE3b's offers,
  the status quo first, every lexicon variant scored by the boundary's
  grow-a-copy, those the world would refuse kept with the reason, and
  `Review::commit` the commands an offer sends. Native discovery is the
  stage's lesson: eating a part whole teaches its kind to the eater's
  lexicon (468), for every line.
  Nothing legacy was deleted and no consumer re-pointed: `discovery`,
  `species`, `program`, `score` and the world's `adapt`, `review` and
  `revise` are fields and methods of the legacy `World` (its lineages, its
  discoveries, `adapt_round` at its boundary, `offers`, `reckon`), and
  every consumer (mesocosm-runtime's review and succession lanes, genet's
  `review.rs` and its eating, expression and grafting, mesocosm-views'
  board, mesocosm-phenotype's expression requests) reads them through a
  driver holding that `World`. They go with the world move and the switch
  (755, D5); each legacy module says so. *Reading, not ruled:* a new line's
  key is its name; readings carry feats as keys (`feat:growth`, …) beside
  hagiograph's axes, the record family deciding where they are noted;
  predation has no native reading, the native flow record being the record
  family's. Forks put to Mark: when and how the review and succession
  lanes re-point; whether the boundary and revision move from `directing`
  into `lineage`; what becomes of legacy discovery's condition table (the
  endurance route, tract grants); and whether the native revision grows to
  legacy's declared tracts and 568's folding of systems.
- **2026-10-10, family 2, bodies, second pass (lane `lane-bodies-2`, under
  732's compile gate; rulings 755 to 759).** Lineage data left isometer-core
  (721, 756): its `Part` carries an opaque `origin: Option<u64>` and no mass
  or provenance, its document no species; `centre_of_mass` takes a mass
  reading. `SpeciesId`, `Origin`, `Provenance` and `LineageBody` (a document
  with species, mass and provenance beside it per part) live in
  `isocosm::lineage`, and legacy Mesocosm and Eponym keep their bodies as
  `LineageBody` under the name `BodyDocument`. *Reading, not ruled:* the
  tag is `Option<u64>`, the donor species plus one, rather than
  wing-formats' `PartOrigin`, which would hold provenance a second time
  beside the sidecar. The founding datasheets feed native (758): one parse
  in `isocosm::datasheet`, lowered onto native kinds and recipes, which
  `bodied::default_kinds` and `roster` read, and onto legacy recipes, which
  legacy `Founding` names; native kinds take the sheets' names. Native
  re-expressions for the world moves (755): `kingdom::of` and the
  `rules::Compatibility` graft allowance. Put back as forks: mosaics (cell
  identity and intake ports against native counts) and Eponym's needs and
  wounds (counters against native accounts and checkpoint 10's wounds).
- **2026-10-10, family 6, effects (lane `lane-effects`, under 732's compile
  gate; no test sweeps, draws or certification run beyond the lane's three
  unit tests).** Native `isocosm::effects` holds the family; glyphs' rungs
  stay parked (279, 673), the code moving unchanged in behaviour.
  - `pack` is legacy `effect_pack` moved whole with its tests (the bearer
    table, `MarkRequest`, `Amount`'s curve), and the stroke `Glyph` moved
    out of `effect_experiment` into `effects`.
  - `embodiment` reads a native body: a trait is a function a living part
    expresses, keyed `function:<name>`, the function a legacy tract's
    process lowers onto (750). The rule is generic over (part, traits)
    pairs, and legacy `embodiment` is now a 66-line adapter that lowers a
    legacy phenotype through the registry to the same rule, staying with
    the world move (755) beside its tests.
  - `functions` is wing-functions membership over a native body (`refs`,
    `live_parts`, `generate_among`, `generate_for_body`); legacy
    `functions` is deleted and its one example re-points.
  - `reading::Journal` is the world-independent half of Eponym's glyph
    reading: the journey, the founding canon and the live one a published
    revision moves, and `GlyphGrantOutcome`. Eponym's legacy `glyphs` keeps
    its `GameState` half (rules, accepted kinds, evidence, the cursor) as an
    adapter over it, going with Eponym's world move (755).
  - `effect_experiment` is handed back to the game under 669, as
    `mesocosm_runtime::effect_experiment`, its tests with it.
  Callers re-pointed in mesocosm-genet (bench effects, spatial, section,
  trial) and mesocosm-runtime's glyph tests. Legacy effects lines went from
  2,820 to 1,252 (`embodiment` and Eponym's `glyphs` adapters with their
  tests). *Reading, not ruled:* the experiment went to `mesocosm-runtime`,
  the game's host-neutral runtime beside its glyph reading, rather than to
  `mesocosm-genet`, whose manifest forbids rules; native trait ids are
  catalogue function keys, so a table authored against `mesocosm:intake`
  embodies nothing on a native body until it says `function:intake`. Put
  back as forks: that trait-id namespace, and whether mesocosm-runtime's
  own glyph reading moves onto `Journal`.
- **2026-10-10, bodies and lineages, cells (lane `lane-cells`, under 732's
  compile gate; rulings 763, 765 and 766).** The boundary and the revision
  live in `lineage/`, directing re-exporting them (763). Native parts carry
  cells by identity (766): `mosaic` lays a part's lattice out from its box,
  keeps tracts (the cells each function holds) beside the counts through
  `sync`, places cells exactly through `propose`, and keeps lost cells and
  an intake port per part, the feeding mode read from active ports. The
  native revision declares tracts and folds systems (765): a `Development`
  carries both, development lays declared tracts on parts of their shape, a
  folded line's children carry its systems, and the boundary and review
  offer both beside recipe variants. *Reading, not ruled:* `sync` keeps
  cells in place where counts move but does not keep a tract connected;
  only a proposal must be. Nothing calls `propose` or reads the feeding
  mode yet; the world moves bring them.
- **2026-10-10, family 7, Eponym's world (lane `lane-eponym-world`, under
  732's compile gate; no test sweeps, draws or certification run).** Under
  755, 767 and 769 to 771 Eponym's world family moved onto native Isocosm
  and legacy Eponym (13,740 lines) is deleted, its 149 tests (65 in 14 unit
  modules, 84 in 16 integration files and their fixture) and two examples
  retired under 672.
  - *Native nouns.* `harm`: a wound takes cells from one part across its
    functions and free pool (709), keeps them in the part's lost cells,
    which the tissue ceiling and free cells now subtract (706), spills each
    account over its lowered bound to the site (708), and at the last cell
    severs by isometer's semantics, moving the tombstones' matter out and
    bumping the revision, or kills at the root (707, 710, 711, 717); a part
    is drawn by living cells where none is named (716); `Command::Wound` is
    how a game hands back a blow (669). Checkpoint 10 builds partly here;
    healing (712), hazards (704), fragments (713 to 715) and rot (718) wait
    for it. `needs`: hunger and fatigue are wants read by the mind's `Need`s
    from a body's own accounts, the matter reserve and `Energy` (767).
    `arrival`: an outsider arriving with issued matter (235, 238), its
    geometry admitted (674), and a name as a note its namer holds (36, 200);
    an authored character asserted placeless (769) arrives by its key, the
    asserted entity taking up the body with its fill and faction (760), as
    the sortie's four peers do.
    `knowing`: observations, claims, reports and corrections are notes held
    by the observer, claimant or hearer, an observation needing the observer
    to be a party or to know the event by the reach field (771).
    `social`: a deed is an act of a deed process leaving a world event,
    native events gaining the act's `object`; standing is folded from deeds;
    the willingness rule weighs crafts kept as skills and caution kept in
    the disposition; standing agreements live in `State::agreements`; a home
    is an authored place's site held by a tenancy relation naming its
    agreement (60, 63, 67, 769). Each comes through a native command.
  - *Handed back (669).* A new game crate, `eponym/crates/eponym-play`,
    drives a sapient player's subject over a native session (671): Eponym's
    world is a native founding with Eponym's pack (sophont accounts, item
    lineages, eat, rest and exertion processes, deed processes, note kinds,
    needs), played in one site isometer lifts; each accepted intent runs as
    native commands and is one tick of the sim. It keeps motion, timed
    actions, the strike resolver, techniques, sheets, admitted anatomy
    snapshots, control and the glyph reading's game half. Walking and sight
    go through isometer's in-site space (`places::steps`, new, and `ray`),
    navigation through SP5's `walk`; carves and inherited sites are facts
    the sim keeps. Population, projects, the autonomous round and the
    simulation record retired with the world (770, 771), as did facets and
    offices.
  - *Re-pointed.* `eponym-client` (its room and residency now lift Eponym's
    native world), `eponym-motion`, and `eponym-sortie`, whose society is
    the native one with S1 and S2's scene kept as its own fixture.
  *Reading, not ruled:* items are inert bodies of item lineages, carried by
  a relation, their resting place and worn part the game's (740); a whole-
  body harm out of a hundred becomes that share of living cells from one
  drawn part, and a part a strike cut is wounded through every cell; the
  game's body revision moves only with a severing (717); an arrival's matter
  is issued like the dev source's; names are notes; crafts and deed kinds
  are closed native vocabularies. Put back as forks: where handed-back game
  code lives, the deed and agreement nouns, items, and the admitted anatomy
  snapshot beside native geometry.
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
