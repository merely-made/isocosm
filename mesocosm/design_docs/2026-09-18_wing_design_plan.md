# The wing as a simulator: design record and plan

**Date:** 2026-09-18

**Status, 2026-09-18:** design record, for Mark's reading. Nothing in it is
implemented against; the first phase is the evaluation of every active plan
in the wing against it. No lane runs until that evaluation is ruled.

**Owns:** the design of the games wing as three tiers, a simulator, a stack
and three game overlays; the rules that decide which tier a thing belongs
to; the method by which claims reach rulings and receipts reach plans; and
the order in which the wing's existing plans are re-read, kept, rewritten or
retired against this record.

**Does not own:** any tier's internal design below the level ruled here, any
product's verbs, the hagiograph's implementation (mere's eidetic family), or
naming. Names for anything founded under this record go through Mark's
naming rounds.

**Consumes:** the [founding record](2026-07-30_games_wing_founding.md),
whose laws and nouns this record sits under; the
[place-graph engine plan](2026-08-05_place_graph_engine_plan.md) §0 rulings;
the [resident views composition plan](2026-08-14_resident_views_composition_plan.md)
for "Burn proposes, the record disposes"; the
[isoscape family plan](2026-09-16_isoscape_family_plan.md) for the
generation bucket; the
[board-on-isometer plan](../../design_docs/2026-09-15_board_on_isometer_plan.md)
at the Isometry root as the worked example of a plan designed to the wrong
target.

**Why this exists.** On 2026-09-16 the board-on-isometer plan's targets were
found to be test constraints chosen by the plan's author rather than facts
about the product: a demo map, parity with the existing board, a
one-megabyte brick budget inherited unexamined, and a five-voxel tile
chosen to patch a symptom of those. Two rulings were taken on claims that
had not been checked against code. Mark's diagnosis, in his words: "You
don't know the right target. Any target is a test. Picking a flagrantly
non-representative, unreasonably limited target is just self-sabotage."
This record is the design that should have preceded that plan, taken from
Mark by questions from the top down on 2026-09-17 and 2026-09-18.

---

## 0. Rulings this record rests on

All ruled by Mark in conversation, 2026-09-17 and 2026-09-18, unless dated
otherwise. Each is quoted or paraphrased closely; the numbered rulings are
what later sections derive from.

1. **The thing is a simulator.** "A configurable, customizable, extensible
   world, history, and entity generator you can play at least three different ways.
   Isometric, dimetric, trimetric, military, cavalier, top down, first
   person, turn based, real time, these are distinctions you make on top of
   this sim."
2. **Three layers run in the background at all times:** adapting ecology in
   a world with history; society adapting and reforming according to the
   goals and abilities of its inhabitants; polities, narratives and arcs
   that will substantially change the world if left unattended. The goal
   underneath them: "define the taxonomy of an ecology, civilization, or
   campaign so you can generate each in enough mechanical depth and systemic
   quality that you can get unpredictable results."
3. **One world, one clock, unbounded time.** Each game foregrounds an
   aspect; "all the background world simming happens, the base profile
   needed for all the games, and you pull forward what is relevant to the
   current game." Mesocosm's natural period is when ecology dominates;
   Paredros needs sapient people and society, not polities; Isometry is
   bound by arcs and narratives.
4. **What is never lossy is the record of significant events.** The test is
   "unprecedented, legendary, or narratively significant, same test
   everywhere"; the promotion gates are novelty, quality and relevance. The
   hagiograph is the organ that judges.
5. **Memory is not global.** Things can be forgotten. Propagation is the
   derived form: an event's reach is a field on the place graph rather than
   per-entity simulated knowledge ("the derived form seems fair").
6. **Foregrounding is choosing** a domain, a generated ensemble, mechanics,
   controls, a perspective and a timescale. "Empires rise and fall in the
   time it takes a critter's lineage to change a trait." An ensemble is "the
   roster of relevant creatures in the relevant scope in the world" and can
   change within one game.
7. **Worlds are forkable and branchable, like a moot.** Any world state can
   found a world; continuity of game state across a branch is not promised;
   the source world keeps going. Persistence across epochs and branches is
   divinity (immortality, resurrection, avatars in a chain of heirs, becoming
   a motif of the world) or longevity by traits (elves, dwarves, sharks,
   tortoises). Creative mode may move specific things between worlds by hand.
8. **The agent kinds are four compositions.** A creature; a faction as "a
   relational entity comprised of many creatures" (a faction defined by a
   person is a party); a polity as "a political entity comprised of multiple
   factions", with state because it has a collective action methodology; a
   lineage as "a historical entity comprised of kith and kin". A settlement
   is a faction until it has a methodology, then a polity. A region is not
   an entity; it is terrain.
9. **State follows methodology.** A thing has its own state exactly when it
   has its own way of turning a situation into an action. Everything else is
   a relation, a record or a field over the things that do.
10. **Processes are one shape at every rung, choices under scarcity,** and
    that is not enough on its own: agentless processes (fields evolving) and
    rung transitions (founding, splitting, incorporation, collapse) are the
    two further shapes. The sim has verbs; a game has handles on them and
    its own verbs on top, "icing on the cake".
11. **Terrain in the sim runs from region to planetary system:** continents
    of any count or exotic structure, oceans including water worlds, a
    region alone when no whole world is needed, a world that may be inert or
    carry a core with effects or be an entity, worlds in relation to each
    other. Configurable in complexity, size, population, temperature,
    rainfall, mountainousness, material composition and world effects,
    including glyphs, suggested anatomy and constraints.
12. **Terrain always has an interior, destructible and constructible,** for
    all three games. The height-and-kind-per-cell that Isometry holds today
    is a far-rung view of that volume, not the truth.
13. **Cell size may differ,** "that's more a question of what we can do. I
    would prefer arbitrary cell sizes."
14. **A graph over the bricks drives processes,** at the rung above the
    bricks, derived from them. Above the near rung, "there could be derived
    candidate locations that are asserted when persisted."
15. **Tests are draws, not picks.** A hand-chosen instance proves nothing;
    an instance drawn from the generator under a seed nobody chose is
    evidence about the space. "Unless it were literally random. Then it
    would be useful."
16. **Stack placement:** cambium sits under isomere; rendering is isometer
    over genet and netrender; generation is isoscape with esp, dramatis and
    cleromancy beside it; persistence, branching and networking are moot,
    muniment and murm; formats hew to open standards and a wing format is
    made only when no standard holds the thing. (dramatis is re-placed under
    the trust plane in §4 after reading it; the rest stands as ruled.)
17. **The sim is isotropy; bridging effects between the layers is
    isostasy.** Ruled 2026-09-18 after the crates.io check in §9.1; the
    remaining registries are still to be checked before either is banked
    the ledger's way, and claim needs a real publish.
18. **A game's grid is a projection.** The sim's volume is cubic; a tile
    with three, four or six sides is a region of voxels its overlay lays
    over that volume, the way a circle is in any voxel game. Ruled
    2026-09-18 ("hex as projection").
19. **The web is first-class** for the whole wing. Ruled 2026-09-18; the
    limits in §4.5 are therefore every tier's limits.
20. **One bench for everything generated and reviewed:** the sim bench for
    processes and effects including magic, the world bench, the specimen
    bench, the item bench, the effect bench, as lanes of one bench, which is
    also the dev tools. Ruled 2026-09-18.
21. **Gamepads are a genet and standards concern** to be addressed, and
    **keymapping is a capability wanted across the stack**, not only in
    isomere. Ruled 2026-09-18.
22. **Lighting, all five rows wanted:** shadows, ambient occlusion, global
    illumination, point lights with day and night, and transparency with
    water. Ruled 2026-09-18.
23. **The terrain gets a per-axis scale,** y and z as well as x, under W3
    ("add the y scale. hell, why not a z scale too"), and **paging is fixed
    one way or another** in every consumer. Ruled 2026-09-18.
24. **genet implements the full W3C animation surface,** the Web Animations
    API included; the earlier deferral was a stopgap from the stylo
    migration and never a design choice. formal-web (gterzian and Taym
    Haddadi) is the architectural reference. Ruled 2026-09-18.
25. **Skinning, textures and animation are not ruled out for the wing.**
    The rigid-parts rule is Mesocosm's voxel body model, where a part
    carries colour per voxel and animates by pose. Ruled 2026-09-18 in
    conversation ("feels rough to be so limited"); superseded the same
    day by ruling 26 on the model.
26. **glTF is the body's render, animation and interchange model,
    generated from the part tree.** "The tree is data, seems to not be
    the correct render/animation/interchange model." The sim's body stays
    a part tree; a projection generates glTF from it, and an imported
    glTF's node hierarchy is read into a tree the same way. One body kind
    at the render tier, not two. Ruled 2026-09-18.
27. **kiss3d, provided it composes with the stack,** and **the renderer
    is swappable** behind the scene contract, so a 2D game from the sim,
    or renderling and nexus far later, remain possible. Ruled 2026-09-18.
28. **Water is salva, and nondeterministic water is legitimate when a
    field directs it,** because water worlds must be possible and because
    outcomes read the field, not the particles. Ruled 2026-09-18; the
    technical reading is in §4.6.
29. **Desktop first-class; the web a supported tier with a stated floor.**
    Ruled 2026-09-18 ("agreed, I suppose"), replacing ruling 19's
    first-class web.
30. **genet's references are the other Rust and browser engines in the
    same lane,** blitz and formal-web specifically, with servo, Firefox,
    WebKit, Chrome and the webviews as the more sophisticated efforts
    with a few key architectural distinctions. Ruled 2026-09-18.

Two earlier rulings this record relies on without restating: the founding
record's five shared nouns, space, bodies, fields, time and provenance
(engine clause, narrowed 2026-08-05), and the place-graph plan's "adjacency
is derived from the landscape, never asserted" (§0, 2026-08-05), now
qualified by ruling 14.

## 1. The derivation rule

Everything below is one rule applied at every rung:

> A world is a seed, its rules, and the set of asserted facts. Everything
> else is derived on demand and may be discarded.

An entity is its seed plus the history that touched it; its default life is
recomputable. A place is derived from the rung below it, or asserted by the
generator from the rung above and then kept. History is the asserted subset
of what happened, judged by the hagiograph. Terrain at a far rung is a
surface with conditions; at the near rung it is a volume, realized under
that surface where something is foregrounded. "Burn proposes, the record
disposes" (resident views plan, 2026-08-14) is this rule stated for
inference; ruling 14 is this rule stated for places.

The rule is also the cost model. Storage is proportional to what was
asserted; processing is proportional to what is due; nothing costs anything
for merely existing. That is how hundreds of thousands of things are held
(§3.5).

## 2. The three tiers

| Tier | What it is | Who owns its verbs |
| --- | --- | --- |
| **The sim** | The base profile: what runs whether or not anyone is playing, and what may never be lost | The sim. A game reaches them as handles |
| **The stack** | What every game needs to stand on the sim: rendering, hosting, interface, generation, persistence, branching, networking, receipts, formats | Nobody; the stack has no verbs |
| **The games** | Three overlays: a domain, an ensemble rule, mechanics, controls, a perspective, a timescale | Each game, its own |

The test for which tier a thing belongs to: if it runs with nobody playing,
it is the sim; if every game needs it and it decides nothing about the
world, it is the stack; if it is a way of playing, it is a game. Today's
crate graph fails this test in several places (§4.3, §8), because the shared
parts were extracted from one product's proven path and shaped by it rather
than designed from the sim.

## 3. The sim

### 3.1 Nouns

The founding record's five nouns, each with a ladder of scale inside it.
The ladder is the same mechanism as background and foreground: the far rung
is what the near rung looks like from far enough away, with the exception
in §3.2 that some rungs are relations and some are agents with state.

| Noun | Ladder, near to far | Notes |
| --- | --- | --- |
| Bodies (agents) | creature, faction, polity, lineage | Ruling 8; lineage is provenance and orthogonal in time |
| Space (places) | cell, region, continent, world, system | Rulings 11, 12; region is terrain |
| Fields | conditions on places; effects left by processes; reach of events | Ruling 5; world state is not a kind, it is the fields a place carries |
| Time | the due-event clock; deep time before handover; branches | Rulings 3, 7 |
| Provenance | seed, deviations, asserted facts, the significant record | §1; ruling 4 |

Things and magic sit inside these: an item is a body without agency and a
relic per Law A; a glyph is magic's vocabulary and magic is a process that
bends the rules the others run under (ruling 10).

### 3.2 Agents and state

State follows methodology (ruling 9). A creature decides and acts and has
state. A faction is a relation among creatures, allegiance to a person, a
place or an idea, and its action is theirs; a party is a faction defined by
a person. A polity has a collective action methodology and so has state of
its own, over many factions. A lineage is a historical entity of kith and
kin: a record, not a decider. A settlement is a faction until it acquires a
methodology.

Consequence for the scheduler: things with methodology get due events;
relations, records and fields cost nothing on their own.

### 3.3 Processes

Three shapes, and no fourth found yet:

- **Choices under scarcity**, for what an agent does: inputs under scarcity,
  a choice, a cost, an outcome, a record of `(scarcity context, chosen,
  foregone, cause-link)` per Law A. One shape from a creature's metabolism
  to a polity's levy.
- **Agentless processes**, for what happens to the world: weather, decay,
  growth, erosion, fire, spread. A rule over conditions on places, running
  with nobody choosing.
- **Rung transitions**, for what happens to the agent set: a faction
  founding a polity, a lineage splitting, a settlement incorporated, a
  polity collapsing into factions. These name what dissolves and what is
  founded, and they are the events the hagiograph most often keeps.

Mesocosm's `ProcessDef` (processdef plan, 2026-08-01) is **not** the first
shape's definition as it stands, by receipt: its digest covers namespace,
name, `expressed_by` and `seeding` and nothing else
(`mesocosm-core/src/process.rs:391-403`), `expressed_by` is a subset of
four roles and `seeding` has two values (`:350-358`), so the whole
definition space holds thirty distinct rule shapes, and the trait catalogue
plan reached the same count independently. It carries no scarcity, no cost,
no foregone and no cause-link. Either it is widened to Law A's record or
the base profile takes a new definition; that is §9.7, now with evidence.
The strongest existing candidate for the new definition is Paredros's
world-conditions schema (world conditions plan, 2026-09-09): typed
conditions, operations, relations and invariants under a content-addressed
rules revision, carrying scarcity, cost and provenance, unimplemented, and
forbidden from promotion by its own stop rule, which is one ruling to
lift (found 2026-09-18 by W1).

### 3.4 The record

The hagiograph judges what is significant by novelty, quality and relevance
(ruling 4), the same test at every rung. Storage stays in journals. The
organ is live and consumed by one product, not pending and not yet
wing-wide: mesocosm-core depends on hagiograph, muniment and nisus, while
paredros-world carries none of them and its nearest thing is a
per-event-kind glyph grant table with no significance gate
(`paredros-world/src/glyphs.rs:233,357-372`; noted 2026-09-18 by W1).
mesocosm-core its `WorldRecord` is a newtype over
`hagiograph::Record` (`record.rs:17-23`), and deep time runs through
`hagiograph::{DeepTime, Handover}` (`deep_time.rs:21`). What its current
significance test lacks is the relevance gate: today it is abnormality
against the world record alone (`history.rs:40-42`). An
event's reach is a field on the place graph: seeded where it happened,
spreading along routes and carriers, decaying with time and slower with
significance, re-seeded when the hagiograph retells it (ruling 5). A
background entity knows what has reached its place. Senses matter only in
the foreground, where a realized creature or character adds what it
personally witnessed, which is its own deviation record. Forgetting is
decay; legend decays slowest.

### 3.5 Holding hundreds of thousands of things

- Aggregate what isn't foregrounded: a population is a distribution, and
  an individual is realized from it when something needs one.
- Schedule events instead of ticking entities: a priority queue of due
  events; an idle thing costs nothing.
- Store deviations, derive the rest (§1).
- Conditions live on places, not on things.

Memory is not the constraint at this scale; processing is, and the four
rules make it proportional to what is happening.

### 3.6 Terrain

The near rung is a volume with an interior, destructible and constructible,
for every game (ruling 12). The rungs above it are surfaces with conditions:
a region's relief and kinds, a continent's structure, a world's shape and
core, a system's bodies in relation (ruling 11). The heightfield and the
volume are two rungs of one thing, not two choices; where the earlier
argument about "heightfields versus bricks" belonged is here, in the ladder,
not in a renderer.

Cell size (ruling 13) is arbitrary within one rule: **power-of-two ratios
of one base unit,** chosen per chunk. Bricks stay eight cubed; a ray walks
each brick in that brick's own scale. Raymarching has no seams at
resolution changes. The five-to-two ratio caused its own alignment work;
the two alignment defects the board plan recorded, the half-voxel centre
offset and the camera's texture-sized half height, predate it and were
fixed independently (corrected 2026-09-18 by W1). With a base unit of 7.5
inches a five-foot tile is exactly one brick face.

The vertical scale is **not** free today. isometer-lens has no terrain
scale; its only scale is one isotropic float per body placement
(`isometer-lens/src/body.rs:33`). The tracer builds rays in world space and
hands them to a traversal that accepts any direction, so a y-scale on the
terrain is a small addition, but it is an addition to the family's floor,
owed under W3. Without it the power-of-two rule cannot reproduce the
shipped step: 8 px on a 16 px tile at eight voxels across wants 3.27
voxels, which no integer gives. So either the step becomes four voxels of
eight, half a tile, and the shipped look changes, or the tracer gains the
y-scale. That choice is a §9 decision beside the base unit.

Size bounds, corrected from the earlier numbers that were wrong (the
one-megabyte and 128 MiB figures were a budget constant and an arithmetic
error respectively): a mile-square world at one voxel per five-foot tile is
tens of megabytes of sparse bricks; at ten voxels per tile it is a few
hundred; both fit an integrated GPU once the brick store is sized to the
card rather than to a constant and its position index covers the loaded
window rather than the world. The web compatibility tier caps one 3D
texture at 256 a side, so a store there spans textures or lives in a
buffer.

### 3.7 The graph over the bricks

At the rung above the bricks, places are nodes derived from the volume:
rooms as connected components of air, caves, corridors, courtyards,
outdoor regions, with edges wherever a passage joins them and weights from
travel cost. Ten thousand nodes, not ten million. Processes run over it:
reachability, travel and trade, breaches and sieges, reach and plague and
temperature as diffusion, districts and factions' ground, and shape
recognition for generation and for the hagiograph's "unprecedented" test.
The graph is derived and locally re-derived from the volume's dirty regions,
so it can never disagree with the bricks. Above the near rung, where bricks
do not exist, the graph is the terrain: derived candidate locations,
asserted when persisted (ruling 14).

That is the design, not the practice. What Mesocosm does today
(corrected 2026-09-18 by W1): `Places::grown` derives links from an
integer heightfield (`mesocosm-core/src/places/relief.rs:7-13,26-32`), the
node set is a fixed three-by-three partition at any enclosure size
(`world.rs:90`), `Places::at` is a two-dimensional nearest-centre scan that
ignores height (`places.rs:170-181`), and interiors are asserted counts
(`places/grown.rs:39-45`). Volume-derived nodes are unbuilt work under W2.

### 3.8 What the sim never does

It never renders, takes input, runs a camera or a turn order. It never knows
a hit point, a dice roll, a system's rules, a loot table, a dialogue line or
a quest. It never pathfinds an individual, checks line of sight or resolves
a single blow. It records that a wolf killed a deer and that a duke fell in
battle; the foreground game resolved both by its own rules, or the
background resolved them by outcome without playing them out. It never
decides what is fun. Mark: "sounds pretty right to me."

## 4. The stack

### 4.1 Placement

Read from each crate's own description on 2026-09-18, not from memory.

| Tier | Component | What it is | Where |
| --- | --- | --- | --- |
| Rendering | isometer | The scene: terrain tracer, body renderer, depth join, picking, glyph batch | `shared/isometer`, Isometry root |
| Rendering | netrender | Composition of rendered textures with the document's paint stream | `repos/netrender` |
| Rendering | wgpu | The device | crates.io |
| Hosting and interface | genet | The document host: window, input routing, layout. References by ruling 30: blitz and formal-web specifically; servo, Firefox, WebKit, Chrome and the webviews as the same lane with a few key architectural distinctions. Owes the full W3C animation surface (ruling 24) | `repos/genet` |
| Hosting and interface | cambium | The element layer, under isomere by ruling 16 | mere `cambium` |
| Hosting and interface | isomere | The wing GUI: sheet, panels, keymap, host assembly | `shared/isomere` |
| Receipts and driving | taproot, mesquite | The scenario driver and the scenario lane with captures, receipts and exit codes | genet, mere `cambium/mesquite` |
| Generation | isoscape | The world generator, planned and not founded | isoscape family plan |
| Generation | esp | mere's portable model-execution seam, with Burn under it | mere `intel/esp` |
| Generation | cleromancy | Deterministic and cast readings with replayable receipts: the seeded draw with a receipt that ruling 15 needs | `repos/cleromancy` |
| Trust plane | dramatis | The cast list: personae for one's own faces and keys, gaz for who one knows, gazette for resolution | mere `dramatis` |
| Trust plane, unresolved | paredros-identity | Ruled "the wing's identity crate" on 2026-08-10 and consumed by nothing outside Paredros since; `SubjectId`, body revisions, facets and the control pointer are the sim's provenance noun. Either dramatis absorbs it under W3 or the promotion is withdrawn (raised 2026-09-18 by W1) | `paredros/crates/paredros-identity` |
| Persistence | eidetic | The durable-memory family: muniment for slots, blobs and journals; chartulary for the content-addressed container graph with lineage; hagiograph for the history organ | mere `eidetic` |
| Branching and federation | moot | gemot for a moot's lifecycle and replication over p2panda; moothold for federation. The branchable-world model is this | mere `moot` |
| Networking | murm | Invitation-scoped peer conversation with signed per-author logs and a WebRTC carrier; iroh is the other carrier | mere `murm` |
| Distribution | luggage | Signed self-update over pluggable feeds | mere `system/luggage` |
| Physics and volumes | conatus | Bodies, collision, queries, fixed step; modulus for the brick atlas and traversal; nisus for revisioned voxel chunks and edits; numen for fields; seiche for force layout | mere `conatus` |

### 4.2 What replaced renderling

Renderling is retired by ruling, not yet in the tree: the presentation
plan's L7 (2026-09-11) has it exit Paredros and "nothing new is built on
renderling", but `paredros-client` still takes it unconditionally by a
machine-local path (`paredros-client/Cargo.toml:66`), the Paredros
workspace patches `spirv-std`, `craballoc` and `crabslab` solely for it,
and L7's done-condition, a client that builds with no renderling
dependency, is unmet (corrected 2026-09-18 by W1). What stands in its
place is not one thing:

- Bodies: `isometer-render`, a small flat-shaded palette-quad renderer over
  wgpu with no textures, no skinning and no engine, because parts are rigid
  by ruling.
- Terrain: `isometer-lens`'s tracer over modulus's traversal.
- The join: the tracer writes the traced surface's depth in the raster's
  clip space, so hardware depth testing settles every pixel (L3 interleave,
  which replaced the D1 join renderling proved).
- Composition: netrender stages the scene's colour view as an external
  image in the document's paint stream.

What renderling had that nothing now has is now ruled wanted (rulings 22,
26 and 27): every lighting row in §4.6, and glTF as the one body model at
the render tier, generated from the sim's part tree and read back from
imports, so skins, textures and animation channels are ordinary glTF
content rather than a second body kind. Voxel parts become glTF meshes
through the greedy quads isometer-mesh already builds. The renderer sits
behind the scene contract and is swappable. The candidate tenant is
kiss3d, whose 0.46 of 2026-08-15 sits on wgpu
30, the wing's own, with gltf, image and winit 0.30 as its dependencies,
all already in the wing; the one objection still standing unchecked is the
landscape doc's, that its constructors create their own device where
netrender composition needs the shared one. If 0.46 takes an external
device it is a tenant; if not, it is the donor for a skinned-mesh pass in
isometer-render, and the gltf crate is the loader either way.

### 4.3 Findings against the stack, 2026-09-16 to 2026-09-18

- **Two voxel chunk mechanisms.** conatus's nisus has per-cell edits and
  dirty regions; isometer-core's `Ground` has a dirty-brick queue
  (`ground.rs:388`), a revision (`:381`) and a private cell write used only
  by `carve` (`:357-379`), but no additive public write, which is why a
  grown ground never advances its revision. It is the one the renderer
  reads. Mesocosm's core uses nisus for body volumes through
  `voxel_profile` and `Ground` for terrain. The wing's don'ts forbid
  exactly this duplication. (Narrowed 2026-09-18 by W1.)
- **A revision that never moves for regrowing hosts.** isometer's
  `GroundTerrain` stamps `Ground::revision()`, which only `carve` advances
  (`isometer-core/src/ground.rs:374`) and `grow` resets to zero
  (`:210-211`), so the tracer silently skips every upload after the first
  for any host that regrows rather than carves. Found by Isometry's B4
  lane with a positive control. Paredros is the one live consumer where
  the mechanism works, because its world carves (`paredros-world/src/world.rs:195`)
  and its producer keys the rebuild on the revision
  (`producer/source.rs:275`); the fix is scoped to regrowing hosts
  (narrowed 2026-09-18 by W1).
- **No brick-level skipping.** modulus's traversal steps voxel by voxel
  through empty bricks with a loop cap of 1,024, so a tall world walks
  hundreds of air voxels per pixel and can fail to reach the ground. The
  two-level walk OpenVDB calls a hierarchical DDA is the standard fix.
- **A budget constant read as a limit.** `modulus::MAX_BRICKS` is 2,047,
  from three atlas layout constants sized to Paredros's one-megabyte
  residency experiment. Paging already exists (`with_capacity`,
  `retarget`) and isometer-lens wraps both (`bricks.rs:95,105`, receipted
  at `tracer_tests.rs:665`). **No production scene in the wing reaches
  it.** Paredros's paging has exactly two callers, the `v1` and `v1b`
  receipt bins behind non-default features
  (`paredros-client/src/bin/v1_residency.rs:53`, `v1b_residency.rs:48`);
  its shipped session host rebuilds the whole ground on every revision
  through `BrickMap::from_ground` (`producer/source.rs:275-280`). The
  scene board reaches the cap routinely and builds through
  `from_ground_keys` and `from_ground_filtered`
  (`crates/isometry-views/src/scene/ground.rs:244,247`). So sizing the
  store to the card is a lane in every consumer as well as isometer's.
  (Corrected twice on 2026-09-18 by W1; the first draft said no scene
  reached the cap and that Paredros ran the paging.)
- **Two scripting engines.** piccolo Lua in isometry-system and
  mesocosm-phenotype; Rhai in numen. Which is the wing's authoring language
  is a §9 question.
- **Two parley font loaders** in Paredros, recorded by M5.
- **The tracer paints a sky** with alpha 1 on a no-hit pixel, so a scene
  cannot leave its background transparent.
- **Hover does not reach a leaf.** cambium-rootstock routes no hover
  movement and publishes no cursor position; `cambium-genet-winit-host`
  does not re-export `RootView`.

### 4.4 Formats

Open standards first, a wing format only where none holds the thing
(ruling 16). The map of what exists: OpenVDB and NanoVDB for volumes, whose
eight-cubed leaves match modulus's bricks; heightmap images and GeoTIFF for
relief; GeoJSON for regions and boundaries; glTF for bodies; MagicaVoxel's
format for authoring; the Universal VTT format and Tiled's TMX for the flat
tile map any other table can read; WebGPU, the W3C spec, for the web floor
(3D textures of 2,048 a side baseline and 256 on the compatibility tier;
storage buffers of 128 MiB per binding baseline).

### 4.5 The web floor

Read from `wgpu-types` 30's limit tables on 2026-09-18. The baseline is
what WebGPU guarantees; the downlevel tier is what wgpu offers where the
adapter cannot meet the baseline.

| Limit | Baseline | Downlevel | Bears on |
| --- | --- | --- | --- |
| 2D texture, a side | 8,192 | 2,048 | Atlases, captures, the document's paint |
| 3D texture, a side | 2,048 | 256 | The brick atlas and pointer volume: a paged store spans textures or lives in a buffer on the downlevel tier |
| Buffer size | 256 MiB | 256 MiB | A brick store in a buffer; the sim's GPU-resident state if any |
| Storage buffer binding | 128 MiB | 128 MiB | Same |
| Storage buffers per stage | 8 | 4 | How many volumes and tables one shader may read |
| Compute invocations per workgroup | 256 | 256 | Burn on GPU, any compute paging |
| Compute workgroups per dimension | 65,535 | 65,535 | Same |

Below the downlevel tier is wgpu's WebGL2 fallback, which as recalled and
not re-read carries no compute and no storage buffers at all; the tracer
survives there because it reads textures, and nothing else GPU-side does.

**Posture, ruled 2026-09-18 (ruling 29):** cross-platform desktop
first-class, the web a supported tier with a stated floor per tier. The renderer's floor is the baked sprite over a heightfield, which
the wing already has, since the bake is a projection of the same voxel
bodies and the presentation plan already named it the far tier of a
hybrid; billboards are not an alternative to voxels but what voxels look
like from the cheapest seat. A traced scene is not too expensive for the
web once brick-level skipping exists and the internal resolution is low,
which the board's render scale already gives. The sim's web floor is one
thread and four gigabytes, met by scaling shard count down (§4.7), not by
a different design. The document layer is web-native regardless.

The web's other floors, none of them GPU: 32-bit wasm memory is 4 GiB
and the 64-bit form is only recently shipping; threads need cross-origin
isolation headers for shared memory, so the sim is single-threaded by
default there; there is no raw UDP, so peer transport is murm's WebRTC
carrier or a relay; durable storage is the origin-private file system,
which muniment already backs; and mere's browser policy is no JIT, so
piccolo as an interpreter is fine and Wasmtime runs ahead-of-time or on
Pulley, which the script substrate plan already prefers.

### 4.6 What renderling had, what stands, and what is missing

| Part | Today | Prior art | Prior plans |
| --- | --- | --- | --- |
| Body raster | isometer-render: flat-shaded palette quads, rigid parts, no engine | Ken Silverman's Voxlap, Voxatron, Teardown's per-object voxels | Presentation plan ruling that parts are rigid; L3 interleave |
| Terrain | isometer-lens tracer over modulus | Comanche's Voxel Space, Teardown, brickmap papers | Resident views plan D1 to V1b; family plan |
| Depth join | Tracer writes clip-space depth; hardware test settles pixels | Standard deferred-composition practice | L3 replaced renderling's D1 join |
| Composition | netrender stages the scene as an external image | | Presentation plan L3 |
| Lighting | One sun and a rim on bodies; a top-face term and fog in the grade | | Grade pass; quantiser ruling |
| Shadows | None | A shadow ray in the tracer is one more traversal per lit pixel; shadow maps for bodies | Not planned |
| Ambient occlusion | None | Per-vertex voxel occlusion as Minecraft's smooth lighting; screen-space for bodies | Not planned |
| Global illumination | None | Voxel cone tracing (Crassin 2011) uses the voxel world as its structure | Not planned |
| Point lights, day and night | None | Any deferred renderer; torches and time of day are sim fields already | Fields plane |
| Transparency and water | None; the tracer writes alpha 1 | Ruled 2026-09-18 (ruling 28): dimforge's salva (0.10, 2026-08) is the water, because water worlds must be possible. The technical reading Mark asked for: nondeterministic water is legitimate exactly when no outcome-bearing fact reads it. The wing already has that two-tier rule from the nexus ruling of 2026-08-06, which bars a nondeterministic tier from outcomes. So the field is the record: depth, current and level per place, held deterministically, which is what a creature's position, a drowning or a flood is computed from; and the particles are what the field looks like up close, driven by it and never feeding back except through the field's own coarse readings. "Voxels as a wave" is the right picture at the surface rung, where the shallow-water equations are literally a wave model, a height and a velocity per cell, cheap and deterministic; in the volume, water is a material whose fill and drain is an agentless process; and salva is the near-tier particle form of the same field. On one machine with a fixed step and one binary, salva is deterministic; across machines it is not guaranteed, which is why it stays on the ambience side of the line | §4.3 finding; rulings 22 and 28 |
| Textures on parts | None, palette only by ruling | glTF materials | Rigid-parts ruling |
| Skinning and animation | Scene bodies: per-part rigid pose from the sim. Document: genet Livery parses `@keyframes`, `animation-*` and `transition-*` with a host-driven clock, which the effect packs' beats already ride; the Web Animations API is deferred | kiss3d's skinning, glTF animation | Rigid-parts ruling; genet's CSS animations plan (2026-07-09); kiss3d ruled a donor for AOV ids only |
| glTF import | None; bodies come from voxel recipes | The gltf crate; kiss3d | Not planned |
| Effects | Effect packs; opaque spatial forms: orbit, inscription, tether, emission | | Effect pack preset plan; presentation plan |
| Particles | None | Any | Not planned |
| Labels and text in scene | DOM overlays through isomere | | isomere plan |
| Picking | Bodies and terrain hits through the scene | | I2 of the board plan |
| Post-processing | The grade: fog, quantiser | | Grade pass |

### 4.7 Parallelism and processes

Read from mere's substrate parallelism composition brief (2026-06-21) and
cross-platform parallelism strategy (2026-06-19) on 2026-09-18. The stack
has already decided the levers:

- **armillary** is the actor kernel: a non-sendable host kernel that
  alone owns the GPU, and sendable actors, thread-per-actor or pooled,
  one per unit of work.
- **Rayon** is the in-unit data-parallel lever on native. On the web it
  becomes a Worker pool only under a nightly, atomics, shared-memory build
  behind cross-origin isolation headers, which the brief hard-gates as a
  PWA-only path; without that it runs serially. Re-checked 2026-09-18 at
  Mark's question: still true. The shim's README says WebAssembly thread
  support "is still only available in nightly", was last tested against
  nightly-2025-11-15 with `-Zbuild-std`, and rustc's own target page for
  wasm32-unknown-unknown still routes atomics through `-Zbuild-std`. The
  browser path has not moved; the runtime-side threads target for
  Wasmtime is a separate matter and not the browser's.
- **The Scene is the serialization seam** between a worker and the main
  thread; the encoder exists and its per-frame cost was measured small.
- **Wasmtime** is the mod and extension runtime, ahead-of-time by
  preference, with OS subprocesses for hostile content (script substrate
  plan, completed 2026-07-03).

What that means for the sim, and what preparing ahead of time looks like
(Mark, 2026-09-18: "I would not like to find myself unable to leverage
modern hardware effectively like RimWorld"):

- **The sim is sharded by the place graph.** Each shard is an armillary
  actor with its own due-event queue over its places; agentless field
  passes run data-parallel over places with Rayon inside a shard.
- **Determinism is the constraint,** the one RimWorld never met and
  Factorio did: fixed shard assignment and an ordered merge of cross-shard
  effects per tick, or the replay hashes break. The merge is designed
  before the shards are.
- **On the web** the sim runs in one Worker, co-located with its script
  host as the brief already decided, and scales down by shard count.

This is a W2 requirement.

## 5. The games

A game is an overlay: a domain, an ensemble rule, mechanics, controls, a
perspective and a timescale (ruling 6). Its verbs are its own; the sim's
verbs it reaches as handles (ruling 10). What each game's profile contains
is that game's design and not this record's. Two questions belong to the
overlay tier and are left open here: who resolves an event when a DM and
the sim both could, and where a foreground game's rules stop and the sim's
begin. Both are inside a branch, so they do not touch the base profile.

## 6. Method

- **Targets are draws.** A receipt is a seeded draw from the generator's
  declared space, and failures are found, not chosen (ruling 15). A fixture
  is allowed as a regression pin, never as the done-condition.
- **Claims are checked before rulings.** No claim reaches Mark as a fact
  unless it was read in code or measured, and the check is recorded beside
  the claim. §7 applies this to this record.
- **Numbers carry their build and their source.** Debug and release
  differ by an order of magnitude on this stack.
- **Findings go to the owner's plan,** not only the finder's.
- **Facts several plans depend on live once** and are cited: the mere pin,
  the brick cap, test counts, the web floor.
- **Done-conditions come from the product's scale and content,** stated in
  a plan's first section, before anything about what a crate offers.

## 7. Assumptions in this record, checked or not

| Claim | Status | How |
| --- | --- | --- |
| The tracer builds each ray in world space and hands it to a traversal that accepts any direction, so a per-brick or per-axis scale is a multiply on entry, not a rewrite | Read in `tracer.wgsl` and `brick_dda.wgsl`, 2026-09-17; not run. W1 confirmed no such scale exists today (`body.rs:33` is the only scale, isotropic, per body) | Design claim; the addition is owed under W3 |
| modulus steps voxel by voxel with a 1,024 loop cap and no brick-level skip | Read in `brick_dda.wgsl`, 2026-09-17 | Checked |
| `MAX_BRICKS` is three layout constants; `with_capacity` and `retarget` exist and Paredros uses them | Read in modulus `lib.rs` and Paredros `residency.rs`, 2026-09-17 | Checked |
| Renderling is retired; isometer-render depends on wgpu only | Read in L7 and `isometer-render/Cargo.toml`, 2026-09-18. W1 found the check too narrow: `paredros-client` still depends on renderling unconditionally and L7 is unmet | Checked for isometer-render; wrong as a wing claim, corrected in §4.2 |
| nisus has per-cell edits and dirty regions and `Ground` does not use it | Read in nisus `lib.rs` and isometer-core manifest, 2026-09-18 | Checked |
| WebGPU compatibility tier caps 3D textures at 256 a side | From the wgpu limits tables, 2026-09-17 | Unchecked against the spec text |
| RimWorld derives regions and rooms from cells and runs temperature and pathing over them | Prior-art memory | Unchecked |
| petgraph carries connected components, Dijkstra, A*, k-shortest paths, dominators, min cut and subgraph isomorphism | Prior-art memory; version in chartulary not read | Unchecked |
| Basic Fantasy RPG's line is CC BY-SA since 2023 | Prior-art memory | Unchecked |
| Storage and processing scale as §3.5 claims at hundreds of thousands of entities | Argument from the derivation rule, not measured | Unchecked |

## 8. The contradictions of 2026-09-16, and what resolves each

| Contradiction | Resolved by |
| --- | --- |
| Shared organs extracted from one product's path and shaped by it; the second consumer's plan titled "what isometer offers" | §2's tier test and §6: the stack is designed from the sim's nouns, never from a product's data |
| Two grains, a six-inch voxel world and a five-foot sprite grid, sharing one scene | §3.6: one base unit, power-of-two ratios per chunk; a tile is a brick face |
| Two theories of appearance in Isometry, stylesheet and voxel-sourced, with the colour table now generating the CSS | Open: a game-overlay question (§5), to be ruled in Isometry's own plan |
| Substrate everywhere, play nowhere; receipts about picking called "playable" | §6: done-conditions from content and scale; the games tier is where play is designed |
| The DOM board and the scene board as two boards | §3.6 and §3.7: the map's height-and-kind is a far-rung view; one renderer over the sim's terrain ladder |
| A brick cap treated as a wall; a warning built for a limit that should not exist | §4.3: a budget constant; the store is sized to the card and paged |
| Rulings taken on unchecked claims (cubic voxels; 128 MiB) | §6 and §7 |

## 9. Decisions for Mark

Mark answered on 2026-09-18; each item records the answer and what is
still open under it.

1. **The sim's home and name.** Answered: not isoscape, which is
   generation alone. Mark offered **isosim**. Candidates checked on
   crates.io the same day: isosim free, isostasy free, isogloss free,
   isoform free, isotropy free, isohyet and isopleth free, isochron taken
   (a cron engine). Only crates.io was checked; games, studios and marks
   were not, so none is banked yet. Registers: isosim is plain and says
   what it is; isostasy is the equilibrium the crust seeks under load, the
   register of three layers adapting; isoform is one gene expressed as many
   forms, the register of one sim played many ways; isotropy is the same in
   every direction. Also ruled in shape: `paredros-world` becomes
   `paredros-core`, and the three product cores are defined in one standard
   way that plugs into the sim; that standard is the game-overlay contract
   of §5, a trait each core implements against the sim, in the spirit of
   mesquite's `Product`. **Ruled 2026-09-18: isotropy** for the sim, and
   **isostasy** for whatever bridges effects between the layers. Games,
   studios and marks remain to be checked before banking; claim by publish.
2. **The base unit and tile geometry.** Answered: the base scale is
   configurable for other rulesets, since five feet is one game's number;
   and a tile may have a configurable number of sides. Ruled here in
   consequence: the sim's length unit is a real length; a world sets its
   base voxel as a real length and a ruleset sets its tile as a power of
   two of it. The regular tilings of the plane are exactly three,
   triangles, squares and hexagons, and games use squares (including the
   isometric diamond, which is a rotated square, and staggered squares) and
   hexagons in two orientations; triangles rarely. Semi-regular tilings
   exist and no game the wing cares about uses them. In three dimensions
   the cube is the only regular space-filler and hexagonal prisms also
   fill space. **Open, with a recommendation:** the sim's volume stays
   cubic and a game's grid, whatever its sides, is a projection its overlay
   lays over that volume, so a hexagonal tile is a region of voxels the
   way a circle is in any voxel game. That keeps ruling 1 literal, the grid
   is a distinction on top. The alternative, a hexagonal-prism brick in
   modulus, is a second brick shape in the family floor and is not
   recommended. **Ruled 2026-09-18: hex as projection.**
3. **Scripting.** Answered: piccolo. Nothing Rhai does that Lua cannot at
   the level of capability; numen's Rhai front end exists to lower field
   expressions to Burn from an AST, which piccolo does not expose. Ruled
   in consequence: piccolo is the wing's one authoring language; numen's
   Rhai stays an internal front end until fields are wing-authored, at
   which point numen gains a Lua front end rather than the wing gaining
   Rhai. mere's own topology brief already names piccolo as the
   modding-Lua option.
4. **The web floor.** The limits are in §4.5. Ruled first-class on
   2026-09-18, reopened the same day, and **ruled again as ruling 29:**
   desktop first-class, the web a tier with a stated floor whose renderer
   floor is the existing bake.
5. **Renderling's parts.** §4.6 is the inventory. Mark noted that genet's
   animation work is done; it is the document half of the animation row,
   CSS animations and transitions in Livery, and scene bodies animate by
   per-part pose from the sim, so nothing renderling had for animation is
   missing. **Ruled 2026-09-18:** all five lighting rows (ruling 22);
   glTF as the one body model at the render tier, generated from the
   tree (ruling 26); kiss3d provided it composes, behind a swappable
   scene contract (ruling 27); salva as the water with the field as the
   record (ruling 28). **Open:** whether kiss3d 0.46 takes an external
   device, which decides tenant versus donor, and is the composition test
   ruling 27 names.
6. **Which game first.** Answered: the sim gets a bench, and ruled
   further the same day: **one bench** with lanes for the sim (processes
   and effects including magic), the world, specimens, items and effects,
   which is also the dev tools. W4 is that bench; the first game overlay
   follows it.
7. **`ProcessDef` as the base profile's process definition.** Not yet
   answered.
9. **`PROJECT_DESCRIPTION.md` at the Isometry root contradicts the
   record and the repo,** found by W1 and maintainer-owned, so surfaced
   rather than edited: pillar 4 says rules are Rhai scripts where §9.3 and
   the repo's own CLAUDE.md say piccolo; pillar 5 says players
   "eventually" join from a browser where ruling 19 makes the web
   first-class; and pillar 2 states the product's board scale as roughly
   15 by 15 to 30 by 30, against a live generator edge of 256 and against
   Mark's stated scope of 2026-09-16 (a castle with tunnels and levels, a
   region, a mile-square world). §6 wants done-conditions from the
   product's stated scale, and the product's stated scale is stale.
   **Open:** Mark restates pillar 2, or the record's scale is the one he
   stated in conversation.
10. **The vertical scale and the shipped step** (from §3.6). **Ruled
    2026-09-18: a per-axis scale, y and z as well as x, added to the
    tracer under W3** (ruling 23), so the shipped step is reproduced
    exactly and the base unit need not be cubic. Paging is fixed in every
    consumer under the same ruling.
11. **The founding record disagrees with this record in three places,**
    found by W1, and both are wing-level, so which yields is Mark's. (a)
    The founding record's §1 says the vessels "do not share a genre, a
    schedule, or their verbs"; rulings 3 and 10 give them one clock and a
    sim that has verbs the games reach as handles. The 2026-08-05 engine
    clause narrowed sharing for organs only and left schedule and verbs
    untouched. (b) Its §6 says "the platform is extracted from shipped
    games, never built platform-first", repeated in `mesocosm/CLAUDE.md`,
    against §11's W2 to W4 preceding W5; Mark's diagnosis of 2026-09-17
    already rejected that frame for the sim ("the guts here are intended
    to arise from combinations of game systems. It never made any sense
    that they would emerge instead of being designed"), so the amendment
    is recording a ruling already made. (c) Mesocosm's
    `PROJECT_DESCRIPTION.md` files "the world's biota speciating on its
    own whether or not anyone is playing" under Speculative, which is §2's
    definition of the sim, and says the vessels share no schedule or
    verbs. Amending the founding record and the CLAUDE.md line is Mark's
    edit; this record does not claim precedence over them until he does.
8. **Components.** Answered in part: audio from woodshed (cpal, hound,
   symphonia and midir are already in the family); text and fonts from
   genet's text stack (parley for layout under the standards review's S9
   contract, illume for lexing and highlighting, knot-editor for document
   authority over editable text); a pack and modding format from the
   stack's script substrate (wasm components on Wasmtime, AOT preferred,
   ruled in mere's document script substrate plan, completed 2026-07-03)
   beside piccolo for rules; a save format from eidetic. **Open:**
   localization, for which the stack has nothing and ICU4X with Fluent is
   the standards-shaped candidate; an input abstraction across hosts,
   where Mark ruled that gamepads are a genet and standards concern to be
   addressed (the W3C Gamepad API is the standard, and nothing in the
   stack's docs mentions it yet) and that keymapping is a capability wanted
   across the stack, so isomere's keymap moves down a tier; and the
   concretization of each choice above in its owner's plan.

## 10. W1 evaluations

The evaluations of every active plan against this record live in
[2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md),
one section per product, so this record stays the design and that document
carries the rulings as they are made.

## 11. Phases and done-conditions

- **W0, this record ruled.** Done when Mark has read it, the §9 decisions
  are taken or explicitly deferred, and the record is committed with its
  index row.
- **W1, every active plan evaluated.** Done when each active plan in the
  three products' indexes carries one line against this record, keep,
  rewrite or retire, with the tier it belongs to and the assumption it
  rests on that this record confirms or contradicts. The board-on-isometer
  plan is first, since it is the worked example. Retired plans move to
  `archive_docs/<date>/` with rationale, per policy; nothing is deleted.
- **W2, the sim's design.** Done when the base profile has its own plan
  under this record, with the taxonomy of §3 as a schema, the three process
  shapes as definitions, the record and reach field specified, and a
  generator whose declared space is the source of every receipt.
- **W3, the stack re-derived.** Done when each stack component's plan
  states what it takes from the sim's nouns and nothing from any product's
  data, and the §4.3 findings are lanes in their owners' plans.
- **W4, the bench.** Done when one bench, which is also the dev tools,
  runs the sim headless under a seed with receipts and has lanes for
  processes and effects including magic, the world, specimens, items and
  effects, so that a draw from the generator's declared space can be run,
  replayed and reviewed in any of them without any game. The specimen
  bench is the first lane to lift: today it is a lane of Mesocosm's product
  host (`mesocosm-genet/src/app/bench/`), not headless and not game-free,
  so it does not yet satisfy this condition (corrected 2026-09-18 by W1).
- **W5, the first game overlay.** Done when a game has a profile designed
  to §5 as a core implementing the overlay contract, and a played loop with
  receipts drawn from the generator.

No code lane runs before W1 is ruled.

## Findings

- 2026-09-17: the tracer's orthographic branch begins on the section's
  wall and passes a world-space ray to `brick_dda`, which clips against the
  pointer volume and steps by voxel; nothing in either assumes a cube of
  any particular size in world units. The claim that shorter voxels needed
  a rewrite was wrong.
- 2026-09-17: the 128 MiB "texture bound" quoted on 2026-09-17 was 512
  cubed presented beside "2,048 a side"; 2,048 cubed at one byte is 8 GiB.
  Video memory is the bound.
- 2026-09-18: dramatis is identity (personae, gaz, gazette), not
  generation; re-placed under the trust plane.
- 2026-09-18: nisus exists in conatus with the single-voxel write and dirty
  regions that isometer's `Ground` lacks.
- 2026-09-18, from W1 on the tabletop: the scene board reaches the brick
  cap and never calls isometer-lens's paging wrap; the two recorded
  alignment defects predate the subdivision; isometer-lens has no terrain
  scale and the power-of-two rule cannot reproduce the shipped step
  without one; `GroundTerrain::revision()` still returns a grown ground's
  zero (`shared/isometer/src/scene/terrain.rs:52`,
  `isometer-core/src/ground.rs:211`) and only `carve` advances it, so the
  stale-terrain bug is unfixed upstream and Isometry carries a local
  workaround; PROJECT_DESCRIPTION's pillar 2 scale of 15 by 15 to 30 by 30
  is the product's stated scale and every size the board argument turned
  on was a crate constant.

## Progress

- 2026-09-18: record written from the 2026-09-17 and 2026-09-18
  conversation.
- 2026-09-18: W1 evaluated the Isometry root: fifteen plans, four
  rewrites, one retirement, ten keeps; three corrections to this record
  folded in (§3.6, §4.3, §7). Rulings pending.
- 2026-09-18: W1 evaluated Mesocosm: thirty-three plans, twelve rewrites,
  four retirements, seventeen keeps; five corrections folded in (§3.3,
  §3.4, §3.7, §4.3, W4) and the founding-record disagreements raised as
  §9.11. The evaluations moved to their own document. Rulings pending.
- 2026-09-18: W1 evaluated Paredros: nine documents, five rewrites, one
  retirement, three keeps; six corrections folded in (§3.3, §3.4, §4.1,
  §4.2, §4.3 twice, §7). W1's reading is complete for all three
  products; the rulings are Mark's.
- 2026-09-18: rulings 22 to 25 recorded (lighting, per-axis scale and
  paging, full W3C animation in genet, mesh bodies as a second kind);
  §4.7 parallelism added as a W2 requirement from the stack's June
  briefs; the web posture reopened with a recommendation in §4.5.
- 2026-09-18: rulings 26 to 30 recorded: glTF generated from the tree as
  the one render model; kiss3d if it composes, renderer swappable; salva
  with the field as the record; desktop first-class and the web a tier
  with a floor; genet's reference set. Rayon on the web re-checked and
  still nightly-only.
- 2026-09-18: Mark answered §9: isotropy and isostasy named, hex as
  projection, web first-class, one bench, gamepads to genet, keymapping
  across the stack. Open: lighting parts, `ProcessDef`, localization.
