# Re-expressing the legacy families in Isocosm: what moved out on 2026-10-18

**Why this moved.** On 2026-10-18 Mark made the old numbered rulings cases, kept as precedent, with new rules numbered by him alone, and asked every active doc to lose its stacked status banners, old progress entries, stale counters and pasted copies. This file holds, verbatim, what left the live [families re-expression plan](../../2026-10-08_families_reexpression_plan.md): its stacked status banners and the Current state section that sat with them, and the Progress entries other than the five that describe the open work (the campaign, the record and social seam, the world move and the follow-ups).

## Status banners

**Status, 2026-10-10:** the migration push is integrated. Legacy
Mesocosm and Eponym are deleted; only `legacy/campaign` remains until V2.
The plan stays live until all seven families certify under 791.

## Current state, 2026-10-10

The October 8 assessment's 63,277-line legacy inventory is historical.
`shared/isocosm/src/legacy.rs` now exports only `campaign`.
Native owners under `shared/isocosm/src/` are `matter/`, `process/`,
`development.rs`, `geometry.rs`, `mosaic/`, `history/`, `map/`, `terrain/`,
`lineage/`, `effects/`, `knowing.rs` and `social/`. In-site shape, edits
and queries belong to `shared/isometer/crates/isometer-space/src/`.
Eponym's game motion, combat, admitted anatomy and control remain in
`eponym/crates/eponym-play/src/`; Mesocosm's runtime reads a native session.

Migration Progress records compile-gated implementation, not certification.
The [after-pass plan](2026-10-10_after_pass_plan.md) owns deferred runs,
consumer reproduction first. Ruling 791 replaces legacy parity with each
family's conservation, replay and controls; 672 permits obsolete tests to
retire only with their invariant's new home named. V2 and E3 are open under
798, through contracts under 794 and 795; their completion is separate.

*The dated design and receipts below remain as written (793).*

**Status, 2026-10-08:** assessment; forks ruled but 9, which goes with the bodies
family (666 to 673).
Comes after checkpoint 9 and the open bugs (wing design record, ruling 656).

## Progress entries

- **2026-10-10, Q1:** current ownership and status checked against the lane
  tree under 793; dated prose retained. Documentation verification only;
  no new test, draw or headed certification.

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
