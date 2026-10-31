# The wing as a simulator: design record and plan

**Date:** 2026-09-18

**Status, 2026-10-22:** the games wing's design record. The numbered entries
in section 0 are cases: the decisions Mark made from 2026-09-17 on, kept as
precedent; a new case takes the next number in answer order (DOC_POLICY,
"Decisions and doctrine", rule 1), and what is read during work is
[the doctrine](doctrine.md). On 2026-10-22 sections 1 to 11 moved to
[their archive](archive_docs/2026-10-22/2026-09-18_wing_design_plan_sections.md). On
2026-10-18 the operational and replaced cases, the stacked status banners,
and the Findings and Progress logs moved verbatim to
[the archive](../mesocosm/design_docs/archive_docs/2026-10-18/2026-09-18_wing_design_plan_retired.md);
their numbers stay unused here. [The case register](case_register.md) lists
every live case in one line with the plans that cite it.

## Current state, 2026-10-10

Cosmograph is the single sim in `shared/cosmograph`. Legacy Mesocosm and Eponym
are deleted; `shared/cosmograph/src/legacy.rs` exports only `campaign`,
held by the VTT until V2. Isometer owns in-site shape, edits and queries in
`shared/isometer/crates/isometer-space/src/`. Eponym's game-side motion,
combat, admitted anatomy and control live in `eponym/crates/eponym-play`;
Mesocosm's runtime reads a native session.

Rulings 793 to 802 supersede the earlier plan inventory and ordering:
archived owners are indexed as history; directing owns M3 and the interim
M4; the overlays own full M4, V2 and E3. V2 and E3 are open now (798),
through their contracts (794, 795), without waiting for M3 certification.
The sim plan owns the folded ecology phases S8 to S10 (802).
Isomere's combined mode host remains unbuilt; a native Mesocosm mode is
available. Eponym draws bodies through Mere's tenant; Mesocosm's adoption
and LiveBody retirement remain 796's work. These are implementation facts
and authorized targets, not new test or headed receipts.

*The dated record and receipts below remain as written (793).*

**Owns:** the design of the games wing as three tiers, a simulator, a stack
and three game overlays; the rules that decide which tier a thing belongs
to; the method by which claims reach rulings and receipts reach plans; and
the order in which the wing's existing plans are re-read, kept, rewritten or
retired against this record.

**Method annotation, 2026-09-27:** Mark restated the question, ruling and
verification method and invited improvements. The preserved method and
explicitly unruled refinements are in the [session notes,
§8.2](../mesocosm/design_docs/archive_docs/2026-10-06/2026-09-22_sim_design_session_notes.md#82-the-method-as-practised).

**Does not own:** any tier's internal design below the level ruled here, any
product's verbs, the hagiograph's implementation (mere's eidetic family), or
naming. Names for anything founded under this record go through Mark's
naming rounds.

**Consumes:** the [founding record](2026-07-30_games_wing_founding.md),
whose laws and nouns this record sits under; the
[place-graph engine plan](../mesocosm/design_docs/archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md) §0 rulings;
the [resident views composition plan](../mesocosm/design_docs/archive_docs/2026-10-28/2026-08-14_resident_views_composition_plan.md)
for "Burn proposes, the record disposes"; the
[isoscape family plan](../mesocosm/design_docs/archive_docs/2026-10-10/2026-09-16_isoscape_family_plan.md) for the
generation bucket; the
[board-on-isometer plan](archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md)
at the repository root as the worked example of a plan designed to the wrong
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

## Current architecture, 2026-09-28

**Design consolidation, rulings 404 to 407; no implementation gate opened.**
One host carries the three modes over one world save (381). The default
camera is the rotatable 2:1 dimetric view (382, 387); additional perspectives
remain possible. A mode controls how someone plays. Knowledge access controls
what they may learn. Camera and presentation detail control how permitted
information is shown. Simulation fidelity, spatial extent, social extent,
agency, retention and update frequency are separate choices with explicit
connections. Changing the camera does not itself change world facts or grant
knowledge; changing modes does not itself reincarnate a subject.

The construction model is nested assemblage of meaningful geometric and
functional primitives. Assemblies remain inspectable inside. Compatibility
rules admit combinations without enumerating every finished critter, plant,
prop, item or building. Construction supplies function and limitations;
geometry supplies measurements where a mechanic needs them (405). Tradeoffs
depend on conditions and epoch. The design does not require creatures to
learn every behavior from zero or function to emerge from unconstrained
physical optimization (407).

A source recipe and history yield a contextual embodiment. Its accepted
construction supports coordinated functional readings, geometry, collision,
animation bindings and descriptions. These derived products cite the source
revision; one body may require many runtime objects. Runtime handles and
render parts are disposable and do not define the continuing individual.
The [body contract's current refinement](../mesocosm/design_docs/archive_docs/2026-10-28/2026-07-31_wing_phenotype_contract_plan.md#current-contract-refinement-2026-09-28)
separates subject, incarnation, recipe, body revision and projection.

Capability is what current construction and circumstances support; repertoire
is the procedures an entity can attempt; proficiency is how well it performs
them. Maturation and practice can change these separately. Perception, belief,
self-belief and tenets inform choices; actual circumstances resolve effects.
The [sim plan](../mesocosm/design_docs/archive_docs/2026-10-28/2026-09-22_sim_plan.md#25-capabilities-and-knowledge) owns these
distinctions. Material and conceptual operations share composition machinery,
with mundane and magical meanings supplied by the world's rules (407).

Magic composes fundamental effects with strongly escalating energy costs,
and its scripts may read world conditions and other generators' accepted
outputs (404). Rulesets categorize those effects and supply distinctive
exemplars; generated spells can extend their catalogues. The
[functional generation plan](../mesocosm/design_docs/archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md#composition-design-2026-09-28)
owns this design, including its open cost and execution questions. Explainable
surprise and powerful combinations are wanted; fairness does not mean every
build is equally strong.

**Execution and inspection, reading rather than a new runtime ruling.**
Generate reusable structures when needed; cache derived results against their
dependencies; schedule known transitions; react to relevant changes; retain
continuous work where it serves a mechanic. World truth, changes, narrative
records, and an observer's beliefs have different purposes. A shared bench
should explain what supports an action, which circumstances changed its cost,
why it failed, and which accepted events caused its outcome, without giving
every inhabitant that diagnostic knowledge. Moirai is comparison material for
rule authoring and causal inspection, not a selected dependency or replacement
authority. Concrete source findings and remaining work live in the plans below.

## 0. Cases this record rests on

All ruled by Mark in conversation, 2026-09-17 and 2026-09-18, unless dated
otherwise. Each is quoted or paraphrased closely. They are cases: precedent
the later sections derive from, not rules; new rules are numbered by Mark.

1. **The thing is a simulator.** "A configurable, customizable, extensible
   world, history, and entity generator you can play at least three different ways.
   Isometric, dimetric, trimetric, military, cavalier, top down, first
   person, turn based, real time, these are distinctions you make on top of
   this sim."

   Links: area Stack and engine · summary The thing is one configurable, extensible world, history and entity generator playable several ways; views and perspectives are distinctions made on top of the sim.
2. **Three layers run in the background at all times:** adapting ecology in
   a world with history; society adapting and reforming according to the
   goals and abilities of its inhabitants; polities, narratives and arcs
   that will substantially change the world if left unattended. The goal
   underneath them: "define the taxonomy of an ecology, civilization, or
   campaign so you can generate each in enough mechanical depth and systemic
   quality that you can get unpredictable results."

   Links: area Processes and rules · summary Three layers run in the background always: adapting ecology, adapting society, and polities and arcs that change the world if left unattended. (amended by 978)
3. **One world, one clock, unbounded time.** Each game foregrounds an
   aspect; "all the background world simming happens, the base profile
   needed for all the games, and you pull forward what is relevant to the
   current game." Mesocosm's natural period is when ecology dominates;
   Paredros needs sapient people and society, not polities; Isometry is
   bound by arcs and narratives.

   Links: area Processes and rules · summary One world, one clock, unbounded time; each game foregrounds an aspect while all background simming runs and relevant parts are pulled forward.
4. **What is never lossy is the record of significant events.** The test is
   "unprecedented, legendary, or narratively significant, same test
   everywhere"; the promotion gates are novelty, quality and relevance. The
   hagiograph is the organ that judges.

   Links: area Record and history · summary The record of significant events (unprecedented, legendary or narratively significant) is never lossy; promotion gates are novelty, quality, relevance; the hagiograph judges.
5. **Memory is not global.** Things can be forgotten. Propagation is the
   derived form: an event's reach is a field on the place graph rather than
   per-entity simulated knowledge ("the derived form seems fair").

   Links: area Record and history · summary Memory is not global and things can be forgotten; propagation is an event's reach as a field on the place graph.
6. **Foregrounding is choosing** a domain, a generated ensemble, mechanics,
   controls, a perspective and a timescale. "Empires rise and fall in the
   time it takes a critter's lineage to change a trait." An ensemble is "the
   roster of relevant creatures in the relevant scope in the world" and can
   change within one game.

   Links: area Directing and play · summary Foregrounding is choosing a domain, ensemble, mechanics, controls, perspective and timescale; the ensemble can change within one game.
7. **Worlds are forkable and branchable, like a moot.** Any world state can
   found a world; continuity of game state across a branch is not promised;
   the source world keeps going. Persistence across epochs and branches is
   divinity (immortality, resurrection, avatars in a chain of heirs, becoming
   a motif of the world) or longevity by traits (elves, dwarves, sharks,
   tortoises). Creative mode may move specific things between worlds by hand.

   Links: area Record and history · summary Worlds are forkable and branchable; any state can found a world, the source keeps going, and persistence across branches is divinity or longevity traits.
8. **The agent kinds are four compositions.** A creature; a faction as "a
   relational entity comprised of many creatures" (a faction defined by a
   person is a party); a polity as "a political entity comprised of multiple
   factions", with state because it has a collective action methodology; a
   lineage as "a historical entity comprised of kith and kin". A settlement
   is a faction until it has a methodology, then a polity. A region is not
   an entity; it is terrain.

   *2026-10-18, Mark:* "A relation acting through members (Recommended)".
   A faction holds no state of its own and acts when its members do, as the
   sim plan reads cases 9 and 63 and the code holds it; "agent kind" here
   names a kind of grouping.

   Links: area The sim's schema and saves · summary Four compositions: creature, faction, polity, lineage; a faction is a relation acting through its members, holding no state of its own (2026-10-18); a region is terrain.
9. **State follows methodology.** A thing has its own state exactly when it
   has its own way of turning a situation into an action. Everything else is
   a relation, a record or a field over the things that do.

   Links: area The sim's schema and saves · summary A thing has its own state exactly when it has its own way of turning situations into action; all else is relation, record or field.
10. **Processes are one shape at every rung, choices under scarcity,** and
    that is not enough on its own: agentless processes (fields evolving) and
    rung transitions (founding, splitting, incorporation, collapse) are the
    two further shapes. The sim has verbs; a game has handles on them and
    its own verbs on top, "icing on the cake".

    Links: area Processes and rules · summary Processes are choices under scarcity, plus agentless field processes and rung transitions as two further shapes; games add handles and verbs on the sim's verbs.
11. **Terrain in the sim runs from region to planetary system:** continents
    of any count or exotic structure, oceans including water worlds, a
    region alone when no whole world is needed, a world that may be inert or
    carry a core with effects or be an entity, worlds in relation to each
    other. Configurable in complexity, size, population, temperature,
    rainfall, mountainousness, material composition and world effects,
    including glyphs, suggested anatomy and constraints.

    Links: area World generation · summary Terrain runs from region to planetary system, configurable in complexity, size, climate, composition and world effects including glyphs and anatomy constraints.
12. **Terrain always has an interior, destructible and constructible,** for
    all three games. The height-and-kind-per-cell that Isometry holds today
    is a far-rung view of that volume, not the truth.

    Links: area World generation · summary Terrain always has a destructible, constructible interior for all three games; Isometry's height-and-kind per cell is a far-rung view of that volume.
13. **Cell size may differ,** "that's more a question of what we can do. I
    would prefer arbitrary cell sizes."

    *2026-10-18, Mark:* "Free base unit, power-of-two cells (Recommended)".
    Any base unit per world; cells within a chunk at power-of-two multiples
    of it. Case 396's evidence had read this case as allowing power-of-two
    cells per chunk.

    Links: area World generation · summary The base unit is free per world; cells within a chunk are power-of-two multiples of it (2026-10-18, correcting 396's reading).
14. **A graph over the bricks drives processes,** at the rung above the
    bricks, derived from them. Above the near rung, "there could be derived
    candidate locations that are asserted when persisted."

    Links: area Processes and rules · summary A graph derived from the bricks, at the rung above them, drives processes; derived candidate locations are asserted when persisted.
15. **Tests are draws, not picks.** A hand-chosen instance proves nothing;
    an instance drawn from the generator under a seed nobody chose is
    evidence about the space. "Unless it were literally random. Then it
    would be useful."

    Links: area Method and documents · summary Tests are draws from the generator under an unchosen seed, not hand-picked instances, since only draws are evidence about the space.
16. **Stack placement:** cambium sits under isomere; rendering is isometer
    over genet and netrender; generation is isoscape with esp, dramatis and
    cleromancy beside it; persistence, branching and networking are moot,
    muniment and murm; formats hew to open standards and a wing format is
    made only when no standard holds the thing. (dramatis is re-placed under
    the trust plane in §4 after reading it; the rest stands as ruled.)

    Links: area Stack and engine · summary Stack placement: cambium under isomere, isometer over genet and netrender, generation in isoscape beside esp, dramatis, cleromancy; moot, muniment, murm persist; open standards first.
18. **A game's grid is a projection.** The sim's volume is cubic; a tile
    with three, four or six sides is a region of voxels its overlay lays
    over that volume, the way a circle is in any voxel game. Ruled
    2026-09-18 ("hex as projection").

    Links: area Presentation · summary A game's grid is a projection: the sim volume is cubic, and any tile shape is a voxel region an overlay lays over it. (amended by 604)
20. **One bench for everything generated and reviewed:** the sim bench for
    processes and effects including magic, the world bench, the specimen
    bench, the item bench, the effect bench, as lanes of one bench, which is
    also the dev tools. Ruled 2026-09-18.

    Links: area Stack and engine · summary One bench for everything generated and reviewed, with sim, world, specimen, item and effect lanes, which is also the dev tools.
21. **Gamepads are a genet and standards concern** to be addressed, and
    **keymapping is a capability wanted across the stack**, not only in
    isomere. Ruled 2026-09-18.

    Links: area Stack and engine · summary Gamepads are a genet and standards concern; keymapping is a capability wanted across the stack, not only in isomere.
22. **Lighting, all five rows wanted:** shadows, ambient occlusion, global
    illumination, point lights with day and night, and transparency with
    water. Ruled 2026-09-18.

    Links: area Presentation · summary Lighting wants all five rows: shadows, ambient occlusion, global illumination, point lights with day and night, transparency with water.
23. **The terrain gets a per-axis scale,** y and z as well as x, under W3
    ("add the y scale. hell, why not a z scale too"), and **paging is fixed
    one way or another** in every consumer. Ruled 2026-09-18.

    Links: area Presentation · summary Terrain gets per-axis scale (x, y, z) under W3, and paging is fixed one way or another in every consumer.
26. **glTF is the body's render, animation and interchange model,
    generated from the part tree.** "The tree is data, seems to not be
    the correct render/animation/interchange model." The sim's body stays
    a part tree; a projection generates glTF from it, and an imported
    glTF's node hierarchy is read into a tree the same way. One body kind
    at the render tier, not two. Ruled 2026-09-18.

    Links: area Bodies and anatomy · summary glTF is the body's render, animation and interchange model, generated from the part tree; the sim body stays a part tree.
27. **kiss3d, provided it composes with the stack,** and **the renderer
    is swappable** behind the scene contract, so a 2D game from the sim,
    or renderling and nexus far later, remain possible. Ruled 2026-09-18.

    Links: area Stack and engine · summary Use kiss3d provided it composes with the stack, and keep the renderer swappable behind the scene contract.
28. **Water is salva, and nondeterministic water is legitimate when a
    field directs it,** because water worlds must be possible and because
    outcomes read the field, not the particles. Ruled 2026-09-18; the
    technical reading is in §4.6.

    Links: area Processes and rules · summary Water is salva; nondeterministic water is legitimate when a field directs it, since outcomes read the field, not particles.
29. **Desktop first-class; the web a supported tier with a stated floor.**
    Ruled 2026-09-18 ("agreed, I suppose"), replacing ruling 19's
    first-class web.

    Links: area Stack and engine · summary Desktop is first-class; the web is a supported tier with a stated floor.
32. **The sim's process definition is founded from Paredros's
    world-conditions schema,** its stop rule lifted, with Law A's record
    fields added; Mesocosm's `ProcessDef` shapes become expressions in
    it. Ruled 2026-09-18.

    Links: area Processes and rules · summary The sim's process definition is founded from Paredros's world-conditions schema with its stop rule lifted and Law A record fields added.
35. **Each game foregrounds one rung of the agent ladder; the others are
    simmed and weakly expressed.** In Mark's words, 2026-09-18: "the
    primary pillar for mesocosm is the critter being refined over the ages
    according to your play preference. So in that sense, I would be more
    interested in the wildlife than the ecology. Similarly for paredros,
    what named entities do is probably most interesting compared to what
    critters and polities do. And characters, partisan, friendly,
    antagonistic, and otherwise factional or unaligned, have the most
    importance in isometry's polities, but the polities themselves set
    narrative stakes (sidequests, alignment, arcs, npcs, access to
    resources). But each of those layers can be simmed and weakly
    expressed even when they aren't the primary concern." This is the
    founding record's critter, borg, character continuity read as the
    games' domains: Mesocosm plays the critter and its lineage, Paredros
    the borg and its factions, Isometry the character inside its polities.

    Links: area Directing and play · draws on 6 · summary Each game foregrounds one rung of the agent ladder (critter, borg, character); the other rungs are simmed and weakly expressed.
36. **What a creature carries at each level of identity,** in Mark's
    words, 2026-09-18, the first W2 answer. *Critter:* "genotype
    (collection of traits in relation. We wanted to use a mosaic scene for
    this before, but I'm open to whatever) and phenotype (conditioned
    genetic expression (so depending on the circumstances, condition,
    status, and activity, the critter's body may express different
    abilities. Like when it is malnourished, maybe a carnivore becomes an
    omnivore, or a cannibal, or when in the presence of the moon, you turn
    into a werebeast). My thinking is, you have traits that might have
    adjacency effects or process conditions like jokers in Balatro, but
    the phenotype is kinda your hand to play with, except you have
    unconditional and conditional abilities. Oh, and kingdom, which is
    kinda like class." *Borg:* "when a critter becomes sapient, they also
    get the ability to name stuff, which makes it a borg. That's a
    baseline ability for borgs, along with relationships; even a
    nonsapient critter can become a borg in that regard. Borgs make me
    think: interactable, possessed of a disposition, and capable of
    remembering. But borgs also inherit the genotype and phenotypical
    expression of their lineage. They are harder to collapse into
    cohorts, being more individual." *Character:* "Characters emerge as
    borgs contingent upon the group, the polity, the collective, even if
    by absence. They're meant to be defined according to the tabletop
    system, but should also be resolvable as a critter and individual
    (borg)." Mark added: "I feel like I missed some things"; §3.2.1
    carries the answer and the gaps as open questions.

    Links: area The sim's schema and saves · summary Critter carries genotype and conditioned phenotype and kingdom; borg adds naming and relationships; character emerges contingent on group and resolves as borg and critter.
37. **Methodology by tier, and the borg line.** Ruled 2026-09-18: the
    agent literature's tiering stands as the sim's, reactive agents for
    critters, belief-desire-intention agents for borgs, normative agents
    for characters ("I cannot believe the agent literature agrees with
    the tiering... sounds good"). The borg line is drawn Dwarf Fortress's
    way, a creature becoming an individually kept, named figure when it
    does something the record keeps or is named or related, "better than
    Nemesis if you ask me": the wing does not solely care about developing
    relationships with antagonists. Disposition is the five-factor axes,
    and "a significant event causing a personality trait sounds cool
    too".

    Links: area The sim's schema and saves · draws on 36 · summary Methodology by tier: reactive critters, belief-desire-intention borgs, normative characters; a creature becomes a borg when the record keeps it; disposition is five-factor.
38. **State is one ledger, read more coarsely up the levels, and the
    coarsening is provided by the collective.** In Mark's words: "most of
    that stuff matters for critters trying to manage a nutrition budget
    and interact with the ecology. There should be some of that in
    paredros and isometry, but abstracted to less and less specific
    criteria. I think you should be able to die of not getting an
    essential nutrient in mesocosm, and you could die of starvation
    generally in paredros, and you can be debuffed for not eating in
    isometry (like according to the tabletop rules), but dying of scurvy
    in a dnd game feels bizarre, right? just not part of the rules; you
    just eat what is recognized as food, you can express the need to
    sleep in terms of exhaustion, for example. just feels like that sort
    of body concern matters less the more factions and polities there
    are; people figure out the essentials and it becomes produced,
    commoditized, etc." Ruled 2026-09-18.

    Links: area The sim's schema and saves · summary State is one ledger read more coarsely up the levels, the coarsening provided by the collective; body needs matter less as factions and polities grow.
39. **Kingdom is class, and scale is orthogonal to it.** Mark, 2026-09-18:
    beside plant, animal and fungus, "playing as germs (micro: still at
    animal scale, perhaps like a paint that can spread, inhabit a creature,
    might die or thrive from being outside, spreads depending on
    substrate, a bit like being a plant but in a creature, and decomposes
    corpses), and huge creatures (macro) existing too. I actually love the
    notion of the lion turtles from Avatar." Consequences in §3.2.1.

    Links: area Bodies and anatomy · summary Kingdom is class and scale is orthogonal to it; micro (germs) and macro (huge creatures) are playable alongside plant, animal, fungus.
40. **The licence rule for rulesets:** "As long as someone won't sue me or
    hate on me for putting the ruleset in my game, I'm happy to include
    it." Ruled 2026-09-18. Applied the same day by reading the licences
    (§5.1): Daggerheart's is a no, Lancer's is a yes with attribution,
    ICON and CAIN have no licence for implementation and are a question to
    Massif Press.

    Links: area Licensing and sources · summary Include a ruleset if nobody will sue or object: Daggerheart no, Lancer yes with attribution, ICON and CAIN need a question to Massif Press.
41. **One language at two scopes, ruled; effects from world composition
    deferred; the space scope is the extension Lancer waits on.** Mark,
    2026-09-18: "Agreed. I would also expect more general effects through
    world composition, but that seems like an additional difficulty. I
    would love to expand into a space scope and support lancer there;
    feels hard to do it without that thesis, but seems like a natural
    extension!" So world rules and game rulesets are one binding language
    at world-founding and play scopes (§9.12a closed). A third source of
    world-scope rules, effects entailed by a world's composition under
    ruling 11 (a ring's gravity, a core's magnetosphere, a thin veil
    between realms), is recorded as a later difficulty rather than part of
    the language now. The space scope, worlds in relation to each other,
    is a desired extension of the sim and Lancer is its natural ruleset;
    neither is in this round, so the next ruleset consumer for the
    tabletop stays a PbtA-shaped system (§9.12b narrowed).
    *Built out 2026-10-03 by rulings 538 to 541:* faithful editions, a
    toolkit grown from them, hybrids by named profile, and a direction
    of authority each world chooses.

    Links: area The sim's schema and saves · summary World rules and game rulesets are one binding language at founding and play scopes; composition-entailed effects deferred; space scope with Lancer is a later extension.
42. **Divinity is intrinsic provenance; constructs are a tier; the second
    tier's word is in question.** Mark, 2026-09-18: "To become divine is
    to become intrinsic to the provenance of the world; once something
    becomes divine, no matter the fork or branch, the divine should be
    there by default post its ascendance. Anything should be able to
    become divine, even a place or items; it's more like 0th tier, the
    hall of fame, even beyond the legendary for a world. It is typically
    the outcome of a wish with the world's strength (like collect the
    glyphs for the world and invoke them all, maybe more?). Constructs are
    an attractive 4th tier to me, but heh, maybe I should change 'borg' to
    mean that and instead make the 2nd tier something more... like a
    person? Or a sapient?" §3.2.1 carries the reading and the naming
    question.

    Links: area The sim's schema and saves · summary Divinity is intrinsic provenance and persists across forks and branches; anything can become divine; constructs are a tier; second tier's word in question. (amended by 45)
43. **How a divine thing is present, and how it ends.** Mark, 2026-09-18:
    "something divine could have an omnipresent instance, where there is
    one, and it's the same anywhere and everywhere. It could also
    participate in a cycle of reincarnation/recreation, destroyed only
    through prophesied conditions related to its ascent. So maybe you get
    the chain of avatars, maybe you get the one true god of lightning,
    maybe you get a worldtree that the world depends on and likewise."

    Links: area The sim's schema and saves · draws on 42 · summary A divine thing is omnipresent as one instance or cycles through reincarnation or recreation, destroyed only through prophesied conditions tied to its ascent.
44. **The quality of divinity is the quality of the journey.** Mark,
    2026-09-18: "perhaps the quality of divinity you can attain is related
    to your journey. If you fulfill the most difficult conditions for
    acquiring each glyph, you can reach tiers of godhood, like one
    immortal but vulnerable life (demigod), reincarnation on death
    (avatar) and immortality otherwise (avatar+), straight up unable to
    be killed immortality (greater divinity)."

    Links: area Organs: glyphs, impresa, hagiograph · summary Quality of divinity depends on the journey: the difficulty of acquiring each glyph yields demigod, avatar, avatar-plus, or unkillable greater divinity.
45. **Provenance and identity are two axes.** Mark, 2026-09-18: "I agree
    with the provenance and identity axes." Provenance is born of a
    lineage, made by a maker, or intrinsic to the world; identity is
    unnamed, named and sapient, or factional. Constructs are the
    made-by-a-maker kind, the divine the intrinsic kind, and neither is a
    rung of the identity ladder. For the second identity level's word he
    finds sophont and denizen both good; the choice is still his.
    **Settled by ruling 200:** sophont is the term of art for a sapient
    entity, and a denizen is a named entity; every sophont is a denizen.

    Links: area The sim's schema and saves · amends 42 · summary Provenance (lineage, maker, intrinsic) and identity (unnamed, named and sapient, factional) are two axes; constructs are made, divine is intrinsic.
46. **Grades per glyph by the form sacrificed.** Mark, 2026-09-18: "I was
    just thinking of like assigning a point tier to what form of glyph is
    sacrificed for ascension. Item vs technique/skill/ability vs embodied
    trait. So depending on if you represent, or learned, or acquired the
    gnostic knowledge of the world in the glyphs, you get more leverage
    out of the ascension, grades per glyph."

    Links: area Organs: glyphs, impresa, hagiograph · summary Glyph grade depends on the form sacrificed for ascension: item, technique or skill, or embodied trait, giving more leverage.
47. **Sacrifice is destruction, and the domain of a divinity is how its
    forms were acquired.** Mark, 2026-09-18: "Maybe destroying a thing
    counts as sacrificing it? Like you could literally carve each world
    glyph into a stick, burn all the sticks, and become a demigod of...
    carving from that, but you could also be killed just as easily. If you
    just bought all of the glyphs, you'd be a demigod of... buying, the
    dominant action for how you got the items. If you could figure out a
    way to destroy your memories of the glyphs as a sacrifice, you could
    become a god of memorizing. It's like, how did you acquire the form
    that was sacrificed/destroyed? What's the mix of forms you're
    sacrificing, because you could also mix items, skills, and your own
    body."

    Links: area Organs: glyphs, impresa, hagiograph · summary Sacrifice is destruction, and a divinity's domain is how its sacrificed forms were acquired; mixed forms mix the domain. (amended by 107)
48. **Divine places, placement, rooms and alignment.** Mark, 2026-09-18:
    "For a place, all the glyphs existing there is enough to make it a
    divine environment; a glyph being in a place has an influence on the
    place, so how durable the glyph's placement is matters, along with the
    quality of the placement (when we get to buildings and rooms, we
    should be able to evaluate the type and impressiveness of a room like
    rimworld? And an alignment system, where things can relate to the
    glyphs/symbols/factions/polities positively and/or negatively
    depending on tenets associated with effects?)"

    Links: area World generation · summary A place with all the glyphs existing there is a divine environment; placement durability and quality matter; alignment relates things to glyphs, factions and polities.
49. **The domain is named freely; the acts fuel it; power is the
    frequency of the domain's process; anatomy is the other road to long
    life.** Mark, 2026-09-18: "you can name your domain when you ascend.
    Because what your domain is remains rooted in what you did to get the
    form of glyph that was sacrificed, the name isn't what matters nor
    what determines what fuels your divinity, which are the acts. But how
    do the glyphs and their effects relate to the processes? Simple: the
    processes that lead to the effects are more aligned with the effect
    when they lead to the effect happening more. So a glyph representing
    slicing would be aligned with things that produce the slicing effect.
    Same with burning, persuading, forgetting, lightening, flying,
    protecting, healing... really any effect we can describe that is
    recognized as fundamental to the world by the world, and thus given
    its own characteristic manifestation (a glyph). You're not gonna be a
    strong god if the process your power (effect) is borne on doesn't
    happen often. So you have to be careful. But there are other options,
    like becoming a vampire, a lich, fey, eldritch... but those revolve
    around instantiating particular anatomies, magical, conditional
    (phylactery) or otherwise."

    Links: area Organs: glyphs, impresa, hagiograph · draws on 47 · summary The domain is named freely; acts fuel it; power is the frequency of its process; anatomy (vampire, lich) is the other road to long life.
50. **A tenet.** Mark, 2026-09-18: "A tenet ought to be that relation:
    that process yields that effect, and here's our opinion of that, and
    the trust people have in it for their survival/fulfillment
    (expectations: maybe people come to expect certain outcomes depending
    on how things turned out in the past, and that's a cohort opinion
    with variance possible? Idk. Feels a little complex, and we're missing
    attitude towards how things happened)".

    Links: area The sim's schema and saves · summary A tenet is the relation that a process yields an effect, with an opinion of that and the trust people place in it.
51. **Who holds tenets, and how alignment is taken at each rung.** Mark,
    2026-09-18: "Factions align with tenets, multiple, through a weighting
    of each sophont's own alignments by each sophont's respective faction
    influence. The things a sophont does can align them with one of the
    many domains, tenets, or pursuits of the world. If you do a lot of
    killing, perhaps a god of war will like you because they align with
    the glyphs of slicing, piercing, and bludgeoning, and perhaps you come
    to represent the act of killing in the manner you favor, which people
    do or don't like. Polities do align with tenets too, but
    constitutionally, not derived, in addition to the factions within
    them. Divinity blends effects and acts through their journey to
    choose a domain; some effects lend themselves to certain acts."
    **Amended by ruling 79:** a polity's alignment is derived from its acts
    too, its decisions among them.

    Links: area The sim's schema and saves · draws on 50 · summary Factions align with multiple tenets by weighting each sophont's alignments by faction influence; polities align constitutionally in addition to their factions. (amended by 79)
52. **The referent by tier, and impact against frequency.** Mark,
    2026-09-18: "choosing a referent is a privilege of the greater
    divinities and greater avatars, but even they are subject to the
    frequency of process constraint. However, a rare process may be
    mitigated if it is high impact... like if the outcome of the effect
    being applied is unprecedented, or causes something significant to
    happen."

    Links: area Organs: glyphs, impresa, hagiograph · draws on 49 · summary Choosing a referent is a privilege of greater divinities and avatars, still subject to process frequency; high-impact unprecedented effects can mitigate rarity.
53. **Holding.** Mark, 2026-09-18: "The squirrels in my backyard tell me
    critters can gather, and can hold depending on their biology (big
    cheeks, pouch, pocket space if eldritch or whatever) but have very
    limited capacity and no tool use ability. A sophont is a critter but
    smart, they use items and carry gear, so they can have an inventory
    limited by their gear instead of their biology. A mech being a borg
    with capacity just contains its pilot, who 'possesses' the mech; its
    'biology' is its construction, which need not be mechanical and inert
    or nonsentient/nonsapient. Any sophont, faction, polity can have
    possessions; a critter can collect, hide items, and use them as
    needed, but they don't really own them."

    Links: area Processes and rules · summary Critters gather and hold by biology with limited capacity; sophonts carry gear; mechs are borgs holding pilots; any sophont, faction or polity can have possessions.
54. **Place.** Mark, 2026-09-18: "a critter has habitats, habits,
    memories, but not really a home that it possesses except through
    presence and defense. Even just being named is enough to be able to
    'have' a home, because that is a feature of sapience, but that's a
    one sided assertion from the sapient to the sentient; otherwise,
    you're just living where you can, as a critter." And the sentence
    that states it: "a human can give a dog a home, but they can't make
    the dog own the home."

    Links: area Processes and rules · summary A critter has habitats, habits and memories but no home it owns; being named lets a sapient assert a home one-sidedly.
55. **One memory, graded, over one history.** Asked on 2026-09-18 whether
    a critter's memory of habit and habitat and a sophont's memory of
    events are two things or one at different ends, Mark ruled "the
    latter" (2026-09-19).

    Links: area Record and history · summary One memory, graded, over one history: critter habit and habitat and sophont event memory are the same thing at different ends.
56. **Standing is three things the record already has.** A sophont's
    alignment, derived from its acts (ruling 51); its reputation, the
    reach of its deeds among those who know of it (§3.4); and its rank,
    which a polity asserts: a title, an office, a membership. Mark,
    2026-09-19: "Yeah, sounds about right."

    Links: area The sim's schema and saves · draws on 51 · summary Standing is three things: alignment derived from acts, reputation as reach of deeds, and rank asserted by a polity.
57. **Two doors: reproduction rerolls expression, the epoch boundary
    changes the lineage; NPC lineages adapt there by default and so does
    yours.** Mark, 2026-09-19: "the question is how do npc lineages
    optimize themselves? Seems we need them to adapt at the epoch
    boundary, for clarity. And then that is the default autopilot for
    your own, too, with some optionality." On what a lineage measures
    itself against: "Something that might shed light on criteria
    organisms could measure themselves against on the metaorganism level
    (lineage): https://ptree.org/about/methods.html." And: "There should
    be a bit of a lottery with phenotypic expression, rerolled at
    reproduction, depending on the heterogeneity of your genes; lineage
    gene/trait change should be more like continental drift or switching
    jokers in balatro (you don't do it in a round, you do it in the shop,
    in between rounds...). But hey, you can always branch or fork off,
    provided you have a plan to grow your branch and compete or cooperate
    with the originating lineage."
    *Applied to systems 2026-10-05 by ruling 568:* a body's systems vary
    at birth, and the epoch boundary folds the variants that prospered into
    the lineage.

    Links: area Ecology · summary Reproduction rerolls expression and the epoch boundary changes the lineage; NPC lineages adapt at the epoch boundary by default, as does the player's.

58. **A spread is a body; a fungus is one body and a germ is many.**
    Mark, 2026-09-19: "For fungus, one body, even separate. Germs are
    many bodies. You literally burn generations spreading as germs,
    letting you revise your genes on the fly depending on
    host/environment, a unique micro advantage. And myco have longer
    life, but they are distributed, collective organisms; there is no
    individual, only the instance of the whole, like the fungelganger in
    nethermurk" (a reference of Mark's, not read here).
    *Placed 2026-10-02 by ruling 467:* a fungus's patches are parts lying on
    sites, in the part tree's model.
    *Amended 2026-10-02 by ruling 488:* a fungus is one entity whose body is
    a territory, and germs are a surface, their lineage their identity.
    *Strained 2026-10-02 by ruling 499:* a germ lineage revises its genes on
    the fly by forking strains where a host or place selects.

    Links: area Bodies and anatomy · summary A spread is a body; a fungus is one body even when separate and germs are many bodies that revise genes on the fly. (amended by 488)
59. **Play is directing, not driving; a creature's senses are its own.**
    Mark, 2026-09-19, asked about senses: "How do you perceive and react
    to the world without fauna sense organs, like most critters? And
    that's not a limitation that should be imposed on the player; playing
    blind or with no audio because it's diegetic sucks. Rather, perhaps
    literally the critter you play has its own perceptions it can suggest
    to you, which it would be acting on if it were on autopilot... but
    then that opens up, like, that play isn't direct, but rather
    directing, more like rimworld with colonists you schedule than like
    vagante or rain world with a creature you control through a hostile
    environment. I kinda fuck with that more, honestly. Hell, those same
    abilities to direct one or a cohort of critters would then be useful
    for paredros too. Perhaps we can take inspiration from pet systems,
    work schedulers, desire paths...? And provide different things to
    schedule depending on your biology? Like embodied abilities with
    procedural planning and execution to 'em, like foraging for an
    herbivore, or hunting for a predator, or scavenging...? Idk. All I
    know is, it's a terrarium, behavior I shape is more interesting than
    just making things do things outright usually."

    Links: area Directing and play · summary Play is directing, not driving; a creature suggests its own perceptions and the player schedules behavior, with players not limited by diegetic senses.
60. **Driving is Paredros; directing composes on top of it.** Mark,
    2026-09-19: "Driving is paredros. You're one sophont, with
    relationships sure, but you are one. I'm just saying you can give
    directives as that sophont to your friends and that can work a bit
    like directing mesocosm's critters, with an opinion modifier for
    command/request efficacy? So the systems compose, it seems to me."

    Links: area Directing and play · draws on 59 · summary Driving is Paredros (one sophont); directing composes on top by giving directives as that sophont, with an opinion modifier for efficacy.
61. **Death is final across the board; each game has its own trick.**
    Mark, 2026-09-19: "death is final across the board, but mesocosm's
    trick is that you have extra lives because of your cohort, paredros's
    is that you do too, but that those lives are even less fungible
    because no individual can really 1:1 replace another unless they're
    a clone. And isometry, death is understood to be a state that could
    almost be defeated if only we know more of the sciences and magics...
    revivify, resurrection, wishes..."

    Links: area Directing and play · summary Death is final across the board; each game's trick: extra lives via cohort in Mesocosm and Paredros, and Isometry treats death as nearly defeatable.
62. **The significant dead go to the planes after life, and summoning
    them back is a costly possibility.** Mark, 2026-09-19: "things that
    die that are significant should go to the planes after life, right?
    So being summoned back from there sorta makes sense as a possible,
    costly thing to do." And: "Let's keep going down. Let's plan it all."

    Links: area The sim's schema and saves · summary Significant dead go to the planes after life, and summoning them back is a possible, costly act.
63. **A faction acts by consent, peer to peer; a polity adds a constitution
    on top; forms of governance are ranked by alignment.** Mark, 2026-09-20:
    "my instinct: factions take collective action by organizing into
    teams/parties through the general methods of communication available to
    all sophonts and additionally those they each align with individually.
    so like if someone wants to do something you do, probably they work
    together. or if you really care about someone, you probably would be
    willing to join them in an act you wouldn't otherwise. so reputation and
    alignment feel useful there, for determining which action to follow, and
    factions favor peer to peer decision-making along with a few general
    protocols for cases that peer to peer agreement cannot emerge (if
    everyone has different alignments, if nobody has an actionable
    opinion...?). but polities have additional methods on top of that,
    which, like an arbitration agreement in a TOS, are part of the deal.
    whether factions or polities share decision-making protocols isn't the
    point; a faction is not oriented towards a constitution and thus lives
    and dies with consent in a way a polity kinda doesn't. this allows a
    polity to assume more hierarchal structures, more distributed
    structures, more complicated societies!" And: "i think all forms of
    governance that can be associated with acts or qualifiable parties
    should be considered. so like the way people solve their problems feeds
    their alignments which prompt their collective action methodology? maybe
    a rogue really believes in kleptocracy, but nobody else does, but they
    believe in individualism, so their support goes there next... like
    preference-ordered rank choice voting for the sim?"

    Links: area Processes and rules · summary A faction acts by consent, peer to peer; a polity adds a constitution; all governance forms are considered and ranked by alignment.
64. **A polity's state needs efficacious means of enforcement, not
    necessarily violent; a polity has a focus, not a size; polities may be
    contingent on polities, and that is building on an institution.** Mark,
    2026-09-20: "With the polity's state, there must be a means of enforcing
    that state. That need not be violence; one can imagine a polity that
    enforces its collectively decided state through the provision of the
    requirements for life, and through self defense, but whatever the means
    (likely multiple), they must be efficacious. And probably it would be
    related to the political system, the acts within. Not every polity is a
    town or a nation. A polity focused on blacksmithing would be a
    blacksmith's guild, no? And some polities may indeed be contingent upon
    others, like factions comprised a polity. If you can make a durable
    agreement that works within another polity, congrats, you've built on an
    institution".

    Links: area The sim's schema and saves · draws on 63 · summary A polity's state needs efficacious, not necessarily violent, enforcement; a polity has a focus, not a size, and may be contingent on other polities.
65. **A polity without support does nothing; it dies only when people agree
    it has.** Mark, 2026-09-20: "when a polity lacks support to do its
    operations, then it essentially does nothing. But they only die when
    people agree they do. Perhaps because their political context changed,
    or because everyone who knew about it died, or the methods, goals,
    and/or means became pointless". **Extended by ruling 136:** a polity
    dies by either route, its own dissolution or nothing holding it any
    more.

    Links: area Processes and rules · summary A polity without support does nothing and dies only when people agree it has. (amended by 136)
66. **Any means may stand behind a constitution; short of death, a polity's
    condition may change automatically.** Asked on 2026-09-20 what stands
    behind a constitution in the sim, "force, faith, money, habit, a god?",
    Mark: "Any of 'em". And, on this record's first reading of collapse
    (corrected by ruling 65): "I would be ok with a polity automatically
    becoming suppressed or inactive or superseded or subordinated to a
    faction."

    Links: area Processes and rules · draws on 65 · summary Any means may stand behind a constitution; short of death a polity may automatically become suppressed, inactive, superseded or subordinated to a faction.
67. **Polities under no host are a faction among themselves.** Put to Mark
    on 2026-09-20 as a corollary of rulings 63 and 64: "polities dealing
    with each other under no host have only consent, so among themselves
    they are a faction. A treaty is then Paredros's standing agreement at
    scale, until an empire, a church or a federation hosts them." Mark:
    "Also i agree with this."

    Links: area The sim's schema and saves · draws on 63, 64 · summary Polities under no host are a faction among themselves; a treaty is Paredros's standing agreement at scale.
68. **A polity's goals are derived from its own alignment, as an
    individual's are; it has no will to continue beyond finding a new use;
    an inactive polity can be revived.** On the reading that an inactive
    polity "can be revived by whoever brings it support, since nothing ended
    it", with the pretender, the government in exile and the restored order
    as examples, and on leaving open the charter, who must agree to a death,
    and reform and secession, Mark, 2026-09-20: "Agreed with these." Asked
    where a polity's goals come from and whether they change: "Well, why
    would a sophont become a blacksmith? Seems like there's no one clear cut
    answer besides to consider a polity sort of a collective entity with its
    own alignment/values? And then derive goals from that, like individuals
    have? If an institution outlived its use, why would it want to continue
    if it didn't find a new use? Polities change like that".

    Links: area The sim's schema and saves · draws on 64 · summary A polity's goals derive from its own alignment, as an individual's do; it has no will to continue beyond a new use; inactive polities revive.
69. **What is of note persists; the rest is regenerated. "Of note" is a tier
    between the soup and legend, for every kind of thing.** Asked what makes
    a location a place and what keeps it the same place, Mark, 2026-09-20:
    "I feel like if something of note happens there, then it persists;
    otherwise it can be regenerated from the same basic facts. Note that
    denizen is now a term for an entity of note. There should be similar
    terms for locations, possibly more..." And: "Like, a tier between
    primordial sim soup things generate from and legendary things. Just
    stuff of note".

    Links: area Record and history · summary What is of note persists and the rest is regenerated; "of note" is a tier between ambient soup and legend, for every kind of thing. (amended by 200)
70. **The soup is the ambient tier; things of note return to it by becoming
    irrelevant to the player, which is garbage collection, and the criteria
    for invalidation are crucial.** Mark, 2026-09-20, on *site* being used
    in several places: "there could be a more anatomical term for
    mesocosm's. Does this need to be a crate, or can it be a component of
    the wing?" And: "Ah, the soup, it is equivalent to the ambient tier, the
    background of information generated by an engine and localized to a
    particular address/place/scene... the use in woodshed is a great example
    of how I think about it conceptually. I see it as a characteristic of
    mere, and a wonderful sign for an app when ambiance can be created and
    used. Things can indeed return to the soup by being made irrelevant to
    the player in some manner; that's garbage collection, right? So the
    criteria for invalidation is crucial..."

    Links: area Record and history · draws on 69 · summary The soup is the ambient tier; things of note return to it by becoming irrelevant to the player (garbage collection), so invalidation criteria are crucial.
71. **The roots of collection are the entities players care about, and
    examination counts as play; collection is graded, down to just the
    event.** Asked who "the player" is that a thing must become irrelevant
    to, Mark, 2026-09-20: "The players of any of the three games, when the
    sim is applied to them, have effectively selected/created entities they
    care about. With nobody playing, you only render things with the upmost
    detail when you're examining them anyway, so it's basically the same. I
    think it's like garbage collecting the world pool in rimworld, right?
    You wouldn't trash a colonist's relative that has appeared before. But
    you would trash most of a raider that got killed with little incident;
    at a certain point just the event."

    Links: area Record and history · draws on 70 · summary Collection roots are the entities players care about; examination counts as play; collection is graded down to just the event.
72. **The place words: a world map is a grid of sites in any shape; region,
    location, wilderness, biome and environment defined.** Asked whether a
    place's kind is stored or read, Mark, 2026-09-21: "I guess i envisioned
    a grid of sites comprising a world map, in any number of shapes (sphere,
    ring, plane, cube, spire, wheel, any shape works for me), each with
    their own terrain and biome (mountainous, valley, island, tundra,
    tropical), region as an area of sites on the world map, a location as a
    known, remarkable, interesting, or potential site (it has additional
    modifiers/conditions compared to wilderness) or nested region within a
    world map site (of which there are many kinds? Like a settlement, a
    dungeon, a city, a trading outpost, a fort, a defensive gate, a highway,
    waaay more... and they can occupy multiple hexes but be considered the
    same location, kinda like civ cities? Ruins could come from sites and
    locations being reused...), wilderness as sites and areas without a
    location but from which locations could be generated according to their
    terrain, biome as the characteristic environmental pattern for a site,
    environment meaning the weather and climate conditions".

    Links: area World generation · summary A world map is a grid of sites in any shape; defines region, location, wilderness, biome and environment.
73. **A world has one shape, any shape.** Mark, 2026-09-21, on ruling 72's
    "any shape works": "i mean a world could be any shape, but just one. Not
    like switch between them. I mean to enable strange scenarios like a
    world resting on a critter".

    Links: area World generation · draws on 72 · summary A world has one shape, any shape, not switchable.
74. **Nesting is one composable, expandable mechanism; world, region, area
    and battlemap are its defaults.** Asked whether the nesting of maps is
    one mechanism repeated or a fixed set of levels, Mark, 2026-09-21: "I
    think it would be cool if the mechanism was composable and expandable
    instead of the order and tiering of the nesting being predetermined. But
    yeah, those are good defaults." **Qualified by ruling 92:** the four
    are the VTT overlay's default scopes, not levels the sim hardcodes.

    Links: area World generation · summary Nesting is one composable, expandable mechanism; world, region, area and battlemap are its defaults. (amended by 92)
75. **Background and foreground agree; the background is cheaper by
    aggregation; losses at transitions preserve similitude; things of no
    note are fungible, behind a configurable buffer.** Asked whether one
    process definition is run two ways and whether the two must agree, Mark,
    2026-09-21: "I would expect the background to be cheaper through
    aggregation and not needing to process some foreground info. I figure
    that just processing thousands of decisions/entities or orders of
    magnitude less aggregates of either would require that. I think the
    background and foreground of the sim should agree... losses should be
    managed between transitions in a manner that preserves similitude.
    Generated assets being a placeholder meant to be reified/supplanted by
    the player, things of no note are fungible, as needed for the system's
    constraints. Now, if you have a very capable system and want to increase
    the buffer of stuff before things are funged, for a feeling of
    consistency, then that should be possible too. But what do you think a
    good sim/game layer interoperation should be, for good efficiency?
    Should this be organized in the manner that wasi/wasm/wit processes are,
    like with workers and a thread pool... and have i properly understood
    your point?"

    Links: area Processes and rules · summary Background and foreground agree; the background is cheaper by aggregation; transition losses preserve similitude; things of no note are fungible behind a configurable buffer.
77. **Two bindings, and the best capabilities each platform has, if that
    costs nothing on mobile or the web.** Mark, 2026-09-21, on §5.2's fork:
    "The two bindings does seem appropriate, too, though. Honestly if there
    is no cost on mobile or the web, i could see myself preferring to use
    the best capabilities available to improve performance".

    Links: area Stack and engine · summary Two bindings, using the best capabilities each platform has if that costs nothing on mobile or the web.
78. **A clean boundary either way.** Mark, 2026-09-21: "there are perhaps
    parts that would be good to componentize and parts that might not be. So
    further optimization is possible. But the lack of a clear boundary would
    bite in the future, whereas performance will generally improve with
    newer hardware. So i favor a clean boundary either way. Would total
    componetization cause us to have ridiculous workarounds and inefficient
    methods, or does it buy architectural ergonomics that are as valuable as
    the performance the clean boundary costs?"

    Links: area Stack and engine · summary Favor a clean boundary between componentized parts either way, since lack of one would bite later and performance improves with hardware.
79. **A polity has its own alignment from its acts, as factions and
    individuals do.** On docket D8, Mark, 2026-09-21: "i think the idea is
    that you want to let the polity determine what it stands for by
    collective/hierarchal decision and actions, but also to compare that
    against the alignments of the factions, individuals, which also takes
    into account the acts of those entities. between the two of them, the
    second is really what the sim's other entity tiers provides, and the
    first is a repeat of that theme. it's more accurate to say: just like
    factions and individuals, a polity has its own alignment from its acts."

    Links: area The sim's schema and saves · amends 51 · summary A polity has its own alignment from its acts, as factions and individuals do.
80. **"Of note" is literal: a note on the thing, made by the sim as a player
    would make one.** On docket D10, Mark, 2026-09-21: "i think of it as
    like literally a note on the site/item/entity. the sim just makes notes
    in the way a player would too, generating the note from the event/thing
    that makes it 'of note.'" On D19 and D20: "idk!", so both stay held.
    **Both since ruled: D19 by ruling 154, D20 by ruling 152.**

    Links: area Record and history · draws on 69 · summary Of note is literal: a note on the thing, made by the sim as a player would make one.
81. **Asserting a constitution is itself an act, the definitive one.** On
    ruling 79, Mark, 2026-09-21: "isn't asserting a constitution of a polity
    itself an act? indeed, a definitive one."

    Links: area The sim's schema and saves · draws on 79 · summary Asserting a constitution is itself an act, the definitive one.
82. **The note is the impresa record, with a freeform text field beside the
    generated details.** Mark, 2026-09-21: "i'm alright with the impresa
    record! ah, but... maybe a lil markdown/djot/gemtxt/txt-formatted
    (sounds like a setting, i guess i'd prefer djot) field, wysiwyg for just
    freeform notes? if we can enumerate the generated details, sure, lovely,
    but something simple, accessible, and interpretable is also nice."

    Links: area Organs: glyphs, impresa, hagiograph · summary The note is the impresa record, with a freeform text field (djot preferred) beside the generated details.
84. **Information spread is to be modelled, cheaply and distributionally,
    with how a person learned a thing resolved on demand.** Asked what a
    thing knows, Mark, 2026-09-21: "didn't we want to model information
    spread? is there a way to do that cheaply and distributionally, so that
    we can resolve how a person learned a thing without mapping each step
    ahead of time necessarily? is that the best way?"

    Links: area Processes and rules · summary Information spread is to be modelled cheaply and distributionally, with how a person learned a thing resolved on demand.
85. **For the impresa: a closed subset inside an open superset.** Mark,
    2026-09-21, on where the freeform text goes: "For impresa... closed
    subset, open superset?"

    Links: area Organs: glyphs, impresa, hagiograph · draws on 82 · summary For the impresa, a closed subset inside an open superset.
86. **The five points of how knowledge resolves are agreed.** Mark,
    2026-09-21: "And agreed on the five points!"

    Links: area Processes and rules · draws on 84 · summary Agrees five points on how knowledge resolves, listed elsewhere in the record.
87. **A secret is valuable, leverage, kept by a taboo; telling it turns on
    relation, opinion and goals, and perhaps personality.** Mark,
    2026-09-21: "A secret is valuable... it is tremendous leverage, often,
    or there wouldn't be rules against its propagation. So for relation,
    opinion, and goals to factor in makes sense to me. I wonder about
    personality, too." And then: "Or perhaps not rule, but 'taboo'".

    Links: area Processes and rules · summary A secret is valuable leverage kept by a taboo; telling it turns on relation, opinion, goals and perhaps personality.
88. **Nesting is allocated as locations are generated, not fixed at
    founding.** Mark, 2026-09-21, on this record's list of what founding
    fixes: "Elaborate on the last two points. Shouldn't the nesting be
    dynamically allocated as locations are generated?"

    Links: area World generation · draws on 72 · summary Nesting is allocated dynamically as locations are generated, not fixed at founding.
89. **Founding is a short flow of crucial details and a seed; authoring is
    as deep as anyone likes, and authored content fills or displaces
    generated content; a world editor is owed.** Mark, 2026-09-21: "I feel
    like there'd probably be a basic flow where someone picks the crucial
    world details in addition to the seed. I'm mentally envisioning
    rimworld's world generation as a reference, i suppose, but with more
    than spheroid worlds." And: "I would love for people to be able to
    author as much of the world as they would like, or apply a set of
    compatible adventure packs to be integrated into the world to ensure the
    narrative hooks are intrinsically available... but that's a world(s)
    editor, which i guess we have to make? Ok. But yeah, authored content
    should fill blank content or displace generated content to the extent
    people are willing to do so."

    Links: area World generation · summary Founding is a flow of key details plus a seed; authoring may go as deep as wanted, displacing generated content; a world editor is owed.
90. **A world is realigned to another ruleset by a generative round that
    advances time.** Mark, 2026-09-21: "I would imagine that worlds can be
    converted to align with different rulesets by having them go through
    another generative round advancing time and realigning the world to a
    new ruleset (e.g. from hexploration to freeform piloting of a ship in a
    planet's orbit, or some other abstraction)..."

    Links: area World generation · summary A world is realigned to another ruleset by a generative round that advances time.
91. **How much history is the founder's choice; the generated timeline can
    be gone back into; the client is not overwhelmed and may zoom in and
    edit.** Mark, 2026-09-21: "How much history is a complexity and size
    issue. People pick how much to simulate dwarf fortress too, but even
    stopping the possibility of wiild events and catastrophes, we're also
    generating a timeline one could go back in time to prior worldstates
    with, right? Stands to reason we'd take care to not overwhelm the client
    with information, but kinda offer them the opportunity to zoom in and
    edit stuff".

    Links: area World generation · summary History depth is the founder's choice; the generated timeline can be revisited; the client is not overwhelmed and may zoom in and edit.
92. **The named scopes must not hardcode Isometry's paradigm into the sim.**
    Mark, 2026-09-21, on this record's "Founding supplies only the
    vocabulary: which kinds of step exist, and the default scopes of world,
    region, area and battlemap": "I wonder about that, 'cause it sounds like
    hardcoding an isometry paradigm into the sim. Like does mesocosm or
    paredros need those distinctions? Does each game get a different
    tiering? Or does each tier mean different things to different games?"

    Links: area Stack and engine · summary The named scopes (world, region, area, battlemap) must not hardcode Isometry's paradigm into the sim.
93. **Deep time is the same sim.** Asked whether a world's past is run by
    the same sim at a coarser grain or written by a separate history
    generator, Mark, 2026-09-21: "Same sim".

    Links: area Processes and rules · summary Deep time is the same sim, not a separate history generator.
94. **Value bears on scarcity, capability and material need; need is derived
    from alignment, acts and predilection.** Asked how things are exchanged
    and what value is, Mark, 2026-09-21: "Items, value, crafting, services,
    should bear relation to scarcity, capability, and material need. Need
    can be derived from a part of alignment, the stuff people do (acts) and
    are likely to do (predilection, personality?). But why do people take
    certain actions? Needs, like sims? And at the sim layer, how's that
    aggregate to give trade trends?"

    Links: area Processes and rules · summary Value bears on scarcity, capability and material need; need derives from alignment, acts and predilection.
95. **Taking an action is analogous to crafting; crafting is incorporating
    materials into an act; values, abilities and needs shape what is made; a
    fey mood may be a triggered need.** Mark, 2026-09-22: "How do we link
    capabilities to need and value so that people will take actions suited
    for their abilities and beliefs to address their needs? I posit that
    maybe taking those actions is analogous to a crafting process, and the
    actual crafting process should be like incorporating materials into
    that. So your values, your abilities, and your needs should shape items
    you decide to make, like other processes! That way, a rare confluence of
    circumstances and ability can be like a fey mood, and people don't just
    churn out weapons to level up and game an rng producer. Maybe even a fey
    mood can be a unique triggered need?" And: "There's also then the
    question of materials, rarities, recipes, technique, skill, hybridizing
    abilities or items, improving abilities and items, and triggered
    conditions for other sorts of acts beyond crafting".

    Links: area Processes and rules · draws on 94 · summary Taking an action is analogous to crafting; values, abilities and needs shape what is made; a fey mood may be a triggered need.
96. **Abilities, skills and techniques defined; kleptoplasty is conditional;
    nis is a scale, not a material; the question was quality, not rarity.**
    Mark, 2026-09-22: "I'm not sure kleptoplasty is as important as it once
    was (conditional, probably not everything can do it even), but i do like
    the idea of hybridizing abilities after learning enough." On materials:
    "Nis and scruples aren't a material any more than a generic 'atom' is.
    They're more like a scale, no? The more interesting question for
    materials is not 'everything is nis or collections of nis,' but rather
    the typology of kinds of nis and their interactions". On capabilities:
    "Abilities are just things an entity can do, roughly analogous to object
    and basic interactions. Skills are like professions and components of
    classes, utilitarian, practiced, experience-accumulating pursuits
    (smithing, fishing, farming, swordplay, pyromancy, etc.). A technique is
    an ability with a skill precondition. You can share techniques if both
    parties have the necessary skills." On rarity: "fair enough. 'It's rare
    because there's not a lot of it around here,' ok, but like, that's not
    was I was getting at. I was trying to get at quality. Is each sword,
    axe, and dagger the same? What sort of meaningful quality variation can
    exist between them, given the directional combat/strike quality system?
    Mount and blade provided distinctions between sorts and qualities of
    weapons... but through stats and tiers. Is that a good method, or are
    there alternatives that require less number comparison to figure out
    what is 'better'? Tiers could gate techniques, within a bucket... and if
    you can improve your sword over time, that feels fairer than 'can't
    swing that sword, it's too shitty' too..."

    Links: area The sim's schema and saves · summary Abilities, skills and techniques are defined (a technique is an ability with a skill precondition); kleptoplasty is conditional; nis is a scale, not a material.
97. **The materials of a world are its critters, across all the kingdoms.**
    Mark, 2026-09-22, on the question of the typology of kinds of nis:
    "whoa, so like, whatever critters exist, across all the kingdoms, those
    are the things stuff can be made of??? holy shit. that's way better than
    steel! it gives you a reason to learn the ecology, what stuff is,
    familiarize yourself with things to see what you can use... oh my god"

    Links: area Ecology · draws on 96 · summary The materials of a world are its critters across all kingdoms.
98. **The world is an entity made of stuff, a world-class entity, the basic
    macro kingdom; stuff can be made out of the world, even critters.**
    Mark, 2026-09-22, on inert matter: "although, presumably the world has
    to be made of stuff that isn't critter, like bedrock... but maybe we can
    resolve this in the same matter. i keep saying i want a world built on a
    critter to be possible. why don't we just make the world an entity made
    out of stuff, too? a special sort of entity, world-class. and that's the
    basic macro kingdom. then stuff can be made out of the world, even
    critters!"

    Links: area Ecology · draws on 97 · summary The world is an entity made of stuff, a world-class entity and the basic macro kingdom; stuff, even critters, can be made from the world.
99. **What kind of entity a world is stays open to the world.** Mark,
    2026-09-22: "maybe the world is inert, or sentient, or a sophont, or a
    god, or a turtle, or whatever. maybe it's flora, or fauna. maybe it's
    dead. who knows!"

    Links: area World generation · summary What kind of entity a world is (inert, sentient, sophont, god, turtle, flora, fauna, dead) stays open to the world.
100. **World is a kingdom; its scale is macro; meso and micro worlds are
     left open.** Mark, 2026-09-22: "world is a kingdom, the scale is macro.
     this leaves open meso (small moon? satellite? asteroid? ship?) and
     micro worlds :3"

     Links: area Ecology · draws on 98 · summary World is a kingdom with macro scale; meso and micro worlds are left open.
101. **The world has traits and its timeline is a lineage; the world should
     be generative; glyphs are broad; a magic system generator is wanted
     that feeds divinity without hardcoded assumptions.** Mark, 2026-09-22:
     "I was thinking that the world could have traits, and a world's
     timeline could be like a lineage, which could grow new traits (or like,
     the catalog each world's critters can draw upon, including glyphs,
     world conditions from which effects, processes, and activities are
     derived? Could that work... like how could the world be generative for
     activities, tenets, abilities, and other such stuff happening in the
     world? We could type things to provide general logical patterns of
     interaction... but what else), fork into different realms, physical
     worlds (split into/create other planets). Also, I view glyphs as a
     broad thing, i think. Believe we did some thinking on how magic could
     manifest in and through world processes and critter anatomy, allow
     rules to be bent, etc. and visually/mechanically i'd love to see
     different representations of magic t. The rules we figured out for it
     could very well map to a broader set of magical systems (unconditional,
     anatomically-gated, process or effect oriented, cyclical, idk). How do
     we design a magic system generator that can feed into the divinity
     system without hardcoding assumptions? I'm ok with changing the
     divinity system to fit"

     Links: area World generation · summary A world has traits and its timeline is a lineage; the world is generative; glyphs are broad; a magic system generator feeding divinity is wanted.
102. **A world's magic, like its other characteristics, is suggested from
     the seed, configurable, and either a static profile or fluid over time
     under conditions.** Mark, 2026-09-22: "I think it ought to be a bit
     like ideology in rimworld. People should be able to set a static
     profile or let the world change with time (with conditions, like no
     magic or magic to start or no apocalypses or banned traits). But a
     suggested set from the seed is a fine default that could then be
     configured, much like the world's other characteristics."

     Links: area World generation · draws on 101 · summary A world's magic, like other characteristics, is suggested from the seed, configurable, and either a static profile or fluid over time under conditions.
103. **The sim knows the parts of an arc and lets entities pursue
     consequential goals for real reasons; the game makes the statement; the
     world is a bit like RimWorld's storyteller.** Mark, 2026-09-22: "Might
     even think of the world as a bit like rimworld's storyteller... I think
     the sim has to be aware of the constituent parts of an arc. I think the
     sim has to enable entities to determine and pursue consequential goals
     for themselves for real reasons, and in the sim's terms. But the game
     is what takes that vocab and makes a coherent game statement out of it.
     So both, with the second being what gives the first its shape."

     Links: area Directing and play · summary The sim knows the parts of an arc and lets entities pursue consequential goals; the game makes the statement; the world is like a storyteller.
104. **Time in a shared world: asynchronous play merged like git, or a clock
     that runs only when played; Mark leans to the second and asks.** Mark,
     2026-09-22: "It would be nice if people could play together
     asynchronously, like i advance the world on my own, then you do on your
     own, then we merge like git when we when we wanna play together
     synchronously. If the conflicts aren't foregrounded material, then it
     should be fine right? Or should it be more like, whoever opens it has
     started the clock, and people can review what happened since, but time
     proceeds when the game is played. That feels more honest, predictable,
     and easy, but eh."

     Links: area Record and history · summary Shared-world time: asynchronous play merged like git, or a clock running only when played; Mark leans to the second and asks (A ruled in 105).
105. **Option A is ruled; related worlds should share a context without
     merging, as branched worldlines or as neighbours in a celestial
     neighbourhood.** Mark, 2026-09-22: "A, then. And also... it would be
     nice if we could put two worlds in a shared context without entirely
     merging them... like two worldlines with a shared history that
     branched, or for entirely different worlds, they could become a
     neighboring world in the celestial neighborhood...? That way you could
     manage a set of related worlds".

     Links: area Record and history · draws on 104 · summary Option A for shared-world time is ruled; related worlds share a context without merging, as branched worldlines or celestial neighbours.
107. **The change to ruling 47 is agreed; a sacrifice's grade is the depth
     of the entity's relation to that magic in its story; collecting all the
     magics is favoured, as a setting.** Mark, 2026-09-22: "agreed. hmm. if
     i were going to determine the significance of a thing relative to an
     entity, i'd probably ask how many things its related to in their story.
     so maybe that entity's relations to the use of the given facet/bit of
     the world's magic is how we could judge the quality of the offering in
     the divinity evaluation. that way, even if someone only used one sort
     of magic, but they used it religiously and accomplished great things
     with it, then they could still achieve greater divinity. now whether
     you just need to pass a threshold of magical potency or still need to
     collect all the magics, perhaps that can be a setting too. but i do
     favor collecting all the magics, even trinkets plus your main magic.
     links ya to the world."

     Links: area Organs: glyphs, impresa, hagiograph · amends 46, 47 · summary A sacrifice's grade is the depth of the entity's relation to that magic in its story; collecting all magics is favoured, possibly as a setting.
110. **The family and the sim are Isocosm; the tabletop is Isocosm: VTT; the
     rename is planned.** Mark, 2026-09-22: "Let's do it, isocosm, eponym,
     isocosm-vtt, etc. Let's plan the rename too". The reservations
     `eponym`, `isocosm` and `isocosm-vtt` were published to crates.io at
     0.0.1 the same day, and the rename is the [family rename
     plan](../mesocosm/design_docs/archive_docs/2026-10-10/2026-09-22_family_rename_plan.md). Where this record says
     "isotropy" it means the sim, now Isocosm; where it says "Paredros" it
     means Eponym. (The clause first written here, "the tabletop keeps
     Isometry as its subtitle", was this record's inference and not Mark's
     word; ruling 111 settles it.)

     Links: area Naming · summary The family and sim are Isocosm; the tabletop is Isocosm: VTT; reservations eponym, isocosm, isocosm-vtt published to crates.io; the rename is planned.
111. **Isometry is retired as a product word and kept as the plain technical
     prefix of the tabletop's crates.** Put to Mark on 2026-09-22 as three
     options after this record's author had inferred, without a ruling, that
     Isometry survived as the tabletop's subtitle: A, the subtitle; B,
     retired as a product word and kept as a technical one, the tabletop
     being Isocosm: VTT only and the `isometry-*` crates keeping their
     prefix as a plain description; C, retired everywhere. Mark: "b.
     although, you know, i did once wonder if there were an appropriate bird
     name for the wing". The bird is a musing, not a reopening; Isocosm
     stands.

     Links: area Naming · amends 110 · summary Isometry is retired as a product word and kept as the plain technical prefix of the tabletop's crates; the tabletop is Isocosm: VTT only.
112. **Mesocosm keeps its name; Eponym stays; the family is settled.** On
     2026-09-23 Mark reopened Mesocosm's name ("a little terrarium of
     anything kinda outstrips it") and two batches were checked (coppice,
     landrace, holobiont, ontogeny and sere the survivors, in the naming
     ledger). On 2026-09-24 his feeling settled: "isocosm for the sim,
     great. mesocosm for the critter game, great. instead of eponym.
     paracosm!" Paracosm was checked and walled (a taken engine crate, a
     registered class 9 mark, two label albums), and on the options for
     eponym: "Meh. Eponym for now." So the family stands as ruled 110 and
     111: Isocosm the sim and the family, Isocosm: VTT, Isocosm: Mesocosm,
     Isocosm: Eponym. The reservations claimed at his word on 2026-09-24,
     hagiograph, redshank and ortet, are in the ledger.

     Links: area Naming · draws on 110, 111 · summary Mesocosm keeps its name; Eponym stays; the family is settled as Isocosm with Isocosm: VTT, Mesocosm and Eponym.
113. **Similitude: the noted always run as individuals; the fungible agree
     in distribution over whatever a later process reads; what was watched
     is logged; exact agreement is a world setting, statistical the
     default.** Put to Mark on 2026-09-24, after the [aggregation
     research](../mesocosm/design_docs/2026-09-22_aggregation_research.md) found that its lossless
     runner cut evaluations 5.93 times in independent worlds and not at all
     in ecological ones: run one world twice from one seed, with a village
     of 300 going into a hard winter, watched once with every person run
     and unwatched once with the village run as a crowd; what has to come
     out the same? Mark: "not certain! recommendations?" This record's
     recommendation had five points. Anything of note never runs as a
     crowd. The fungible agree in distribution and not in identity: the
     same toll, falling the same way across everything a later process
     reads (age, role, condition, beliefs, factional blocs), within a
     stated tolerance, "who" being undefined for the fungible until someone
     looks. The test is collection's observational test run forward in
     time: nothing that reads the world afterwards, a player, a process or
     the hagiograph, can tell which way the winter ran beyond chance,
     checked on the bench over seeded draws run both ways, which rules out
     the X series's systematic difference between modes while allowing
     different samples. What was watched goes in the log, since watching
     changes which individuals exist in detail and a replay must know it,
     ruling 71's "examination counts as play" taken literally. Exact
     agreement is a world setting, the lossless runner its mode and the
     reference every approximation is tested against, and ruling 75's
     buffer is this knob, held in the world's rules and never by the host.
     Put back as three options, accept, exact by default, or distribution
     only, Mark: "Accept as recommended." This settles §3.3's "a host's
     setting" against §3.12's "never of the machine" in the second's
     favour, and gives the aggregation research its acceptance criterion.

     Links: area Processes and rules · draws on 75 · summary Similitude: noted things always run as individuals; the fungible agree in distribution; what was watched is logged; exact agreement is a world setting, statistical default.
114. **A game's ruleset is held to ruling 113's test: rulesets calibrate to
     the sim.** Put to Mark on 2026-09-24: two armies meet and a duke
     falls; at a VTT table 5e resolves the battle, and with nobody playing
     the sim resolves it as an outcome; should a game's ruleset be held to
     the same test as the sim's own two ways? Three options were put:
     rulesets calibrate, the table is canon, or a world setting. Mark:
     "Rulesets calibrate." So the sim owns the background's outcome model,
     and each ruleset, reading the ledger as its sheet (ruling 38) and
     given the same situation with no player choices, must match it in
     distribution, checked on the bench; what players choose is an input
     and never a seam. This rules the agreement half of §5.2's resolution
     handoff, held under D19, and reverses the sim plan's "owed by the
     definitions and not by any game".
     *Amended 2026-10-03 by ruling 538:* this holds in a world the sim
     leads; a world may choose instead that its ruleset leads, the sim's
     background following it.

     Links: area Processes and rules · summary A game's ruleset is held to ruling 113's test: rulesets calibrate to the sim, matching it in distribution when no player choices are made. (amended by 538)
115. **Competition: each side chooses.** Put to Mark on 2026-09-24, after
     ruling 114 left the sim owing a model of a fight and the aggregation
     research had found shared-resource competition to be a world rule
     rather than an optimization: when two want the same thing and there
     isn't enough (the last of the food, a den, a mate, a throne), what
     decides who gets it? Three options were put: contest, the
     better-placed side taking it; scramble, the shortfall shared out; or
     each side choosing. Mark: "Each side chooses." So each party's
     methodology picks contest, yield, share or trade, the sim resolves
     the choices, and whether a lineage leans to contest or scramble is a
     trait. A fight is what a contest becomes; how it resolves is the next
     question.

     Links: area Processes and rules · draws on 114 · summary Competition: each side's methodology chooses among contest, yield, share or trade, and the sim resolves the choices; lineage leaning is a trait.
116. **Most contests never reach blows.** Put to Mark on 2026-09-24,
     following ruling 115: once both sides choose to contest, what ends
     the fight? Three options were put: one side breaks, will deciding as
     the fight goes; one side is spent, the fight running mechanically; or
     most never reach blows. Mark: "Most never reach blows." So the sides
     size each other up first, by display or bluff, and most contests end
     there; only close matches escalate, and then one side breaks.
     **Refined by rulings 221 to 223:** an escalated fight builds strain
     against bearing, a break shifts the advantage without deciding, and
     the fight ends when a side yields on re-sizing or is spent.

     Links: area Processes and rules · draws on 115 · summary Most contests never reach blows: sides size each other up by display or bluff, only close matches escalate, then one side breaks. (amended by 221, 222, 223)
117. **Any belief can be wrong, through wrong or stale information, and
     entities act on what they believe.** Mark, 2026-09-24, on ruling
     116's bluff: "Deception potentially leads people to make the wrong
     judgment?" Put to him as whether things in the sim should be able to
     believe what isn't true and act on it, with three options: any belief
     can be wrong; only by deceit; only in the moment. Mark: "Any belief
     can be wrong." So a belief can be false through deceit, honest
     mistake, or a rumour that changes as it is retold, entities act on
     what they believe, and reach carries versions of an event, not only
     how strongly it arrived. And, while it was being recorded: "Allows
     for bluffing, being underestimated, but mechanic… perhaps something
     like charisma in dnd terms, posing as an action to
     intimidate/persuade/deceive? But in the sim's terms, it's 'wrong'
     information or stale information, i suppose".

     Links: area Processes and rules · draws on 116 · summary Any belief can be wrong through deceit, honest mistake or a changing rumour; entities act on what they believe; reach carries versions of an event.
118. **Whether a belief takes: what the receiver can check, who's telling,
     what it wants to hear, and how it's told, all four.** Put to Mark on
     2026-09-24, following ruling 117: when someone is shown or told
     something, what decides whether they believe it? Four options were
     put, to pick all that should count: what they can check, evidence
     outweighing what is told, so a lie lasts only where it cannot be
     checked and a stale belief falls when fresher news arrives; who's
     telling, trust in the teller by the receiver's opinion of them, their
     reputation and rank; what they want to hear, fit with the receiver's
     tenets, loyalties and hopes; and how it's told, the teller's practised
     skill to intimidate, persuade or deceive, which a ruleset reads as
     charisma. Mark picked all four. How they are weighed against each
     other is the next question.

     Links: area Processes and rules · draws on 117 · summary Whether a belief takes depends on all four: what the receiver can check, who is telling, what it wants to hear, how it is told.
119. **Both: each entity's disposition shifts a world baseline, and the
     baseline is drawn from the world entity's disposition, a neutral
     median by default.** Put to Mark on 2026-09-24, following ruling 118:
     when the four pull different ways, what sets how much each counts?
     Three options were put: their disposition; the world's rules; or
     both, a baseline set at founding and each disposition shifting it.
     Mark: "Both. World baselines can be more like aggregate baselines
     drawn from the world entity disposition? Then a neutral median
     default for the sim is the world's normal way".

     Links: area Processes and rules · draws on 118 · summary Each entity's disposition shifts a world baseline for weighing beliefs; the baseline is drawn from the world entity's disposition, a neutral median by default.
120. **The world's disposition seeds the founders only.** Put to Mark on
     2026-09-24, following ruling 119: is the world entity's disposition
     the baseline for every inhabitant's disposition, not just for how
     they weigh beliefs? Three options were put: every disposition, the
     world's pull continuing; founders only; or only belief. Mark:
     "Founders only." So the world's disposition seeds the first lineages;
     after that, dispositions are inherited and lived, and the world's
     pull fades.

     Links: area World generation · draws on 119 · summary The world's disposition seeds the founders only; afterwards dispositions are inherited and lived and the world's pull fades.
121. **The world shapes its people's dispositions through trait and
     through condition; people drift from the world's temperament, and the
     world's temperament may not be static either.** Put to Mark on
     2026-09-24, following ruling 120: ruling 119 drew the belief baseline
     from the world's disposition and ruling 120 has the world seed only
     the founders; does the belief baseline fade the same way? Two options
     were put: fades too, or stays pinned. Mark: "I think it seeds
     disposition and such things partially through trait, and partially
     through condition, right? Like is it a world of scarce or abundant
     resources, does the world have intentions, what are the world's
     cyclical processes, what is the world made of… i agree that a world's
     people can drift from their world's temperament, and a world's
     temperament is possibly not a static thing either. Should it not
     change…?" So the direct seeding is the founders' (ruling 120), the
     world's conditions keep shaping how its people turn out, and the
     baseline for weighing beliefs is the population's own, free to drift.
     Whether and how the world's temperament changes is his question, put
     back to him.

     Links: area World generation · draws on 119, 120 · summary The world shapes people's dispositions through trait and condition; people drift from the world's temperament, and that temperament may not be static.
122. **A fluid world's temperament is moved by what happens to it, by its
     own cycles, and at its epoch boundary, not by its people's
     temperament.** Ruling 121's "Should it not change…?" was answered from
     ruling 102 and §3.11: every world characteristic is a static profile
     or fluid over time under conditions, the storyteller's disposition
     included, so a founder may pin it. Put to Mark on 2026-09-24: when a
     world's temperament is fluid, what moves it? Four options were put,
     to pick all that should: what happens to it; what its people are; its
     own cycles; its epoch boundary. Mark picked what happens to it, its
     own cycles and its epoch boundary, and left out what its people are.
     So an age of war can embitter a world and an age of plenty mellow
     it, it turns with its seasons and ages, and it changes in its shop
     between rounds. Leaving out the second reads as: its people move it
     only through what they do to it, never by what they are.

     Links: area World generation · draws on 121, 102 · summary A fluid world's temperament is moved by what happens to it, its own cycles and its epoch boundary, not by its people's temperament.
123. **Harm is both vigour and wounds.** Put to Mark on 2026-09-24,
     returning to the fight after ruling 116: when a blow lands, what does
     it change in the body? Three options were put: the part it hits, a
     wound to one part of the body's tree; a pool of vigour, the reserve of
     fight left in the body, closest to hit points; or both. Mark: "Both."
     So a blow drains vigour first and wounds a part when it lands hard or
     the vigour is gone; vigour comes back with rest, and wounds heal
     slowly, or never.
     *Specified 2026-10-02 by ruling 469:* a wound loses cells from the part
     it lands on, regrown at a price where the lineage's traits allow, or
     never.

     Links: area Bodies and anatomy · summary Harm is both vigour (a reserve drained first) and wounds (to parts); vigour returns with rest and wounds heal slowly or never.
124. **The first scale target is a region; larger scales are not
     precluded.** Put to Mark on 2026-09-24, after the finding that the
     generator's declared space had parameters and no ranges and that no
     phase measured §3.5's scale: at its largest, what world should a
     seeded draw be able to run on his laptop? Three options were put: a
     region, hundreds of sites, tens of thousands of critters, hundreds of
     them named, a century of history in a few minutes; a continent,
     thousands of sites, hundreds of thousands of things, a millennium in
     minutes; or a whole world, millions of things, ages of history,
     overnight if need be. Mark: "We can start with the region, not
     precluding the larger scales of course."

     Links: area World generation · summary The first scale target is a region (hundreds of sites, tens of thousands of critters); larger scales are not precluded.
125. **Noting and promotion belong to the world; collection and merging
     belong to the record.** Put to Mark on 2026-09-24, from the review's
     finding that the sim plan's table of world transitions listed noting,
     collection, promotion to legend and merging branches beside death and
     founding a polity: are they events in the world, or work done on the
     record about it? Three options were put: record work; world events;
     or a split, noting and promotion the world's, since something became
     remarkable, and collection and merging the record's. Mark: "Split."

     Links: area Record and history · draws on 69, 70 · summary Noting and promotion belong to the world; collection and merging belong to the record.
126. **A fork founds a new world; a branch is the same world played apart
     and merged back.** Put to Mark on 2026-09-24: ruling 7 says worlds
     are "forkable and branchable"; should a fork be the new world that
     never merges back, and a branch the same world played apart and
     merged back by replay? Mark: "Yes, fork and branch." So a fork founds
     a world with its own history from that point, related by descent
     (D33), and a branch is ruling 104's play branch. D23's edit to the
     past, which may never rewrite, reads as a fork.

     Links: area Record and history · draws on 7, 104 · summary A fork founds a new world with its own history that never merges back; a branch is the same world played apart and merged back.
127. **Everything fades, at a rate the holder's memory sets; physical
     notes do not.** Put to Mark on 2026-09-24, from ruling 70's "the
     criteria for invalidation is crucial" and RimWorld's caution: does a
     note ever lapse on its own, or does a thing stay of note for as long
     as anything that matters reaches it? Three options were put: only
     when unreached; everything fades, unless renewed, legend never; or
     each kind its own rate. Mark: "Everything fades, moderated by, i
     guess in dnd terms, intelligence? Plus you can keep physical notes
     that don't fade, like a journal, notebook, diary, etc."

     Links: area Record and history · draws on 70 · summary Everything fades at a rate the holder's memory sets, unless renewed; physical notes (journal, diary) do not fade; legend never fades.
129. **A thing stays of note only while some mind remembers it or some
     bearer records it.** Put to Mark on 2026-09-24, from ruling 127:
     ruling 80 has the sim write notes "the way a player would", and a
     note now fades at the rate its holder's memory sets; when the sim
     notes a place, an item or an entity, whose memory is that note? Three
     options were put: whoever remembers it; the world's own memory; or
     kept like a player's journal, never fading. Mark: "Whoever remembers
     it." So each mind's copy fades at its own rate, a bearer's lasts with
     the bearer, and when the last copy is gone the thing returns to the
     ambient; legend never fades.

     Links: area Record and history · draws on 127, 80 · summary A thing stays of note only while some mind remembers it or some bearer records it; when the last copy goes it returns to ambient.
130. **A player keeps things of note through who they play and what they
     pin; in Eponym a player's notes are diegetic.** Put to Mark on
     2026-09-24, from ruling 129's open case: a player remembers from
     outside the world, so what keeps something of note on a player's
     account? Three options were put, to pick all that should count:
     everything they looked at, never fading; through who they play, at
     that entity's memory rate; and what they pin. Mark: "Through who they
     play, What they pin, Notes the player makes in eponym should be
     diegetic. Not so for the vtt or mesocosm. Notes external to the game
     could be embedded in the app via knot editor later."

     Links: area Record and history · draws on 129 · summary A player keeps things of note through who they play and what they pin; Eponym's notes are diegetic, the VTT's and Mesocosm's are not.
131. **A mind remembers what was new to it, mattered to it, was intense,
     or kept happening.** Put to Mark on 2026-09-24, closing §3.4.1's
     open item on what ruling 71's "little incident" measures: what makes
     a mind remember something in the first place, so it becomes of note?
     Four options were put, to pick all that should count: it was new; it
     mattered to them; it was intense; it kept happening. Mark picked all
     four. So a little incident is one that was none of these to anyone
     who saw it, and no mind keeps it.

     Links: area Record and history · draws on 71, 127 · summary A mind remembers what was new to it, mattered to it, was intense, or kept happening.
132. **An unpractised skill decays slowly, to a floor it never drops
     below.** Put to Mark on 2026-09-24, from ruling 127's "everything
     fades" and the open question of whether a skill decays: does that
     include skills, the practised crafts a sophont gets better at? Three
     options were put: skills fade too, at the mind's memory rate; skills
     keep; or slowly, to a floor. Mark: "Slowly, to a floor." So a craft
     left unpractised slips toward a floor it never loses, and doing is
     kept better than knowing.

     Links: area Processes and rules · draws on 127 · summary An unpractised skill decays slowly, to a floor it never drops below.
133. **Factions are never gated by disagreement; a polity is, where its
     ultimate authority is vested in a group.** Put to Mark on 2026-09-24,
     from §3.2.2's open "which protocols": when a faction can't agree,
     everyone wanting something different or nobody caring enough to act,
     what should a world's ruleset let happen? Three options were put, to
     pick all that should be available: nothing happens; they split; or
     their preferred way, the members falling back on the way of deciding
     their alignments rank highest, held for that one decision. Mark:
     "Their preferred way, Factions shouldn't be gated by disagreement.
     Polities' collective processes, insofar as they vest ultimate
     authority in a group rather than an individual, do gate polity acts
     on disagreements."

     Links: area Processes and rules · draws on 63 · summary Factions are never gated by disagreement; a polity is, where its ultimate authority is vested in a group.
134. **A constitution changes by the polity's own way or its amendment
     rule; force outside the constitution founds a new polity and never
     takes over the old.** Put to Mark on 2026-09-24, from §3.2.2's open
     "reform": how can a polity's constitution be changed? Three routes
     were put, to pick every one that should exist: by its own way of
     deciding; by its amendment rule; by force. Mark: "By its own way, By
     its amendment rule, For force, unless that is literally ruled by the
     constitution, circumventing a political process with violence can
     only lead to a new polity, not taking over the old one. So a faction
     wanting the full sweep of a nation's power needs to acquire it
     legitimately or claim it quickly after the dissolution of the
     previous polity."

     Links: area Processes and rules · summary A constitution changes by the polity's own way or its amendment rule; force outside it founds a new polity and never takes over the old.
135. **Secession is the seceders' own act.** Put to Mark on 2026-09-24:
     can members leave a polity and take part of it with them, a town, a
     province, a guild chapter? Three options were put: only if it
     agrees; by their own act; or only what is theirs. Mark: "By their
     own act." So the seceders act as a faction and found their own
     polity, and the old one may accept it or contest it, the contest
     running like any other (rulings 115 and 116).

     Links: area Processes and rules · draws on 115, 116 · summary Secession is the seceders' own act; they found their own polity and the old one may accept or contest.
136. **A polity dies by either route: its own dissolution, or when nothing
     holds it any more.** Put to Mark on 2026-09-24, from §3.2.2's open
     "who must agree": a polity dies only when people agree it has (ruling
     65); who has to agree? Three options were put: its own way,
     dissolution decided like reform; whoever still holds it, dying when
     no member keeps it, no mind remembers it and no bearer records it; or
     either route. Mark: "Either route."

     Links: area Processes and rules · amends 65 · summary A polity dies by either route: its own dissolution, or when nothing holds it any more.
137. **The way of deciding suggests the means of enforcing by default and
     never binds them.** Put to Mark on 2026-09-24, from §3.2.2's open
     link between a polity's deciding act and its means: rule by buying
     withholds pay, rule by divining withholds rites, rule by fighting
     fights; is that link a default or a rule? Mark: "A default." So the
     generator suggests means from the way of deciding, and a polity may
     enforce by any means it has: a theocracy that fines, a merchant
     republic with an army.

     Links: area Processes and rules · summary The way of deciding suggests the means of enforcing by default and never binds them.
138. **Deadlock is per act, not a condition.** Put to Mark on 2026-09-24,
     from §3.2.2's open "whether more conditions are wanted", with ruling
     133's deadlock as the candidate: is a deadlocked polity a fifth
     condition beside inactive, suppressed, superseded and subordinated?
     Three options were put: its own condition; inactive; or per act only.
     Mark: "Per act only." So deadlock blocks one act at a time, the
     polity stays active for everything its group does agree on, and the
     conditions stay four.

     Links: area Processes and rules · draws on 133 · summary Deadlock is per act, not a polity condition; conditions stay four.
139. **Law A is amended for materials.** Put to Mark on 2026-09-24, from
     D28's check left open for him: does materials-as-lineages satisfy Law
     A, which says morphology doesn't travel between games as foreign rule
     authority? Three options were put: it satisfies Law A, what crosses
     being an item bearing its lineage's provenance; amend Law A, naming
     materials as the exception, a lineage's traits travelling as what
     things are made of, beside "shapes become relics"; or keep them
     apart. Mark: "Amend Law A." The amendment's wording is drafted for his
     word before it touches the founding record. Shown the draft, Mark:
     "Apply both as drafted", so it stands in the founding record's Law A
     and in Mesocosm's CLAUDE.md pipeline-laws line.

     Links: area Ecology · draws on 97 · summary Law A is amended for materials: a lineage's traits may travel as what things are made of, beside shapes becoming relics.
140. **A world's materials lie on a spectrum, from a normal base that
     matches a ruleset to completely generated.** Put to Mark on
     2026-09-24: are a world's own materials, its rock, metal, water and
     air, generated per world, or drawn from a base every world shares?
     Three options were put: generated per world; a shared base; or base
     plus variants. Mark: "Base, variants, and generated. I would straight
     up love to have a completely randomized generated world, but a
     spectrum between that and a normal base world which matches a
     ruleset is probably good". Where a world sits on that spectrum reads
     as a founding setting, as its magic is (ruling 102).

     Links: area World generation · draws on 102 · summary A world's materials lie on a spectrum from a normal base matching a ruleset, through variants, to completely generated; a founding setting.
141. **A skill rises by doing, being taught, studying and breakthroughs.**
     Put to Mark on 2026-09-24, from §3.3.1's open "how a skill
     accumulates": what raises a skill? Four options were put, to pick all
     that should: doing it, harder or riskier work teaching more; being
     taught by someone more skilled; studying from bearers, books, manuals,
     a master's notes; and breakthroughs, a fey mood, a near-death, a
     masterwork. Mark picked all four.

     Links: area Processes and rules · summary A skill rises by doing, being taught, studying, and breakthroughs.
142. **Techniques are both discovered and invented.** Put to Mark on
     2026-09-24, from the open "what a technique is made of": where do new
     techniques come from? Three options were put: invented, a skilled
     practitioner combining effects it knows into one nobody had; found in
     the world's canon; or both. Mark: "Both." So the canon holds known
     techniques to be found, and masters can invent new ones beyond it,
     which spread by teaching and by bearers.

     Links: area Processes and rules · summary Techniques are both discovered in the world's canon and invented by masters, spreading by teaching and bearers.
143. **An item's improvement is limited by its maker's skill and its
     story, not its material.** Put to Mark on 2026-09-24, from the open
     "improving an item": what limits how far an item can be improved?
     Four options were put, to pick all that should: its material, the
     lineage's traits a ceiling; the maker's skill; its story, an item of
     note growing with what it has been through; or nothing. Mark picked
     the maker's skill and its story. So only a better maker takes an item
     further, an item of note grows with its deeds as well as its work,
     and its material is no ceiling.

     Links: area Processes and rules · summary An item's improvement is limited by its maker's skill and its story, not its material.
144. **An item can be a sophont: made so, awakened by its story, or
     inhabited.** Mark, 2026-09-24, while ruling 143 was being recorded:
     "We should also probably consider items that are sophont". Put to him
     as how an item comes to be a sophont, three routes to pick every one
     that should exist: made so, its maker building a mind into it, ruling
     45's construct that happens to be a sophont; awakened by its story,
     an item of note that has been through enough growing a mind of its
     own; or inhabited, a mind moving into it, a bound spirit, a soul in a
     phylactery, one of the dead summoned into a vessel. Mark picked all
     three.

     Links: area The sim's schema and saves · draws on 45 · summary An item can be a sophont: made so, awakened by its story, or inhabited.
145. **A sophont item acts through its wielder, by its own powers, and can
     take over.** Put to Mark on 2026-09-24, following ruling 144: how
     does a sophont item act in the world? Three options were put, to pick
     all that should be possible: through its wielder, directing whoever
     holds it, weighed by the wielder's opinion of it (ruling 60), the
     wielder free to refuse; by its own powers, as far as its body and
     magic allow; and taking over, a strong enough item possessing its
     wielder, the reverse of ruling 53's pilot possessing the mech. Mark
     picked all three.

     Links: area The sim's schema and saves · draws on 144, 53 · summary A sophont item acts through its wielder, by its own powers, and can take over its wielder.
146. **A sophont item is never owned.** Put to Mark on 2026-09-24: can a
     sophont item be owned? Three options were put: like any item; only by
     agreement; or never owned. Mark: "Never owned." So it can be held but
     never owned; it is a companion, and it can own things itself.

     Links: area The sim's schema and saves · draws on 144 · summary A sophont item is never owned; it can be held, is a companion, and can own things itself.
147. **A site and a paged chunk are independent.** Put to Mark on
     2026-09-24, from §3.7.1's open "how large a site is, in voxels and
     against the paged chunk": how does a site, the world map's unit,
     relate to a paged chunk, the store's unit? Two options were put:
     independent, or one site paging as one chunk. Mark: "Independent." So
     a site is the unit for generating and playing, a chunk the unit for
     paging, and a site spans as many chunks as its volume needs.

     Links: area World generation · summary A site (generating and playing unit) and a paged chunk (paging unit) are independent; a site spans as many chunks as its volume needs.
148. **When a location's ground changes or moves, its kind decides what
     it follows.** Put to Mark on 2026-09-24, from §3.7.1's open anchor of
     a place when the ground itself moves: eroded, flooded, carried off on
     a giant's back, what does the location follow? Four options were put:
     its site; its ground; its people and works; or its kind deciding.
     Mark: "Its kind decides." So a camp follows its people, a ruin its
     ground, a battlefield its site.

     Links: area World generation · draws on 72 · summary When a location's ground changes or moves, its kind decides what it follows: a camp its people, a ruin its ground, a battlefield its site.
149. **Neighbouring worlds keep their own clocks and are synced at
     crossings.** Put to Mark on 2026-09-24, from §3.12's open question:
     do neighbouring worlds share one played clock? Three options were
     put: one clock; separate clocks, a crossing converting and a
     traveller perhaps arriving in the neighbour's past or future; or
     synced at crossings. Mark: "Synced at crossings." So each world keeps
     its own clock until something crosses, and the crossing brings the
     two into step, as realignment does.

     Links: area Processes and rules · summary Neighbouring worlds keep their own clocks and are synced at crossings.
150. **A planetary system can be a host.** Put to Mark on 2026-09-24,
     from ruling 100's open item and §3.12: can a planetary system be an
     entity in its own right, a host for its worlds? Three options were
     put: yes, a host; just relations; or the founder's choice. Mark:
     "Yes, a host." So a system is an entity above the world, a sun with a
     provenance and founding ruleset of its own, hosting its worlds as a
     polity hosts others.

     Links: area World generation · draws on 100 · summary A planetary system can be a host: an entity above the world with its own provenance and founding ruleset, hosting its worlds.
151. **Owning a person is slavery, and treating people as chattel is
     itself a tenet.** Put to Mark on 2026-09-24: ruling 146 says a sophont
     item is never owned; does that hold for every sophont? Two options
     were put: every sophont, a polity's claim of people as property
     staying a claim and its enforcement and never possession; or items
     only, the rest following each world's rules. Mark: "If a person is
     owned, that's slavery. Treating people like chattel is a tenet
     itself, no? And it's clear people might differ depending on it's
     slavery by whether if they're the same kind of thing as the
     enslaved." So whether a person may be owned is a tenet (ruling 50),
     held or rejected by sophonts, factions and polities like any other,
     and how one judges a slavery turns on whether one is the same kind of
     thing as the enslaved.

     Links: area The sim's schema and saves · draws on 50, 146 · summary Whether a person may be owned is a tenet, held or rejected like any other; owning a person is slavery.
152. **Outside the world, a player directs only who they play.** Put to
     Mark on 2026-09-24, from held item D20: outside the world, who may a
     player direct, given that inside it anyone can ask anyone through the
     entity they play, weighed by opinion under ruling 60? Three options
     were put: only who they play; also what is bound to them; or that
     plus edit mode directing whatever no player is bound to. Mark: "Only
     who they play." So everything else is asked in the world, through the
     entity a player plays, and D20's provider over `mere-capability`
     implements that in W3.

     Links: area Directing and play · draws on 60 · summary Outside the world, a player directs only who they play; everything else is asked in the world through that entity.
153. **Two players may direct the same entity.** Put to Mark on
     2026-09-24, co-op being a core taste in every vessel: can two players
     direct the same entity? Three options were put: yes, both, the
     entity weighing conflicting directives by its opinion of each; one at
     a time; or no. Mark: "Yes, both."

     Links: area Directing and play · draws on 152 · summary Two players may direct the same entity, which weighs conflicting directives by its opinion of each.
154. **D19's contract is ruled.** Put to Mark on 2026-09-24: D19's
     contract, a game submitting intents, the sim returning events and a
     read-only view of each tick, and outcomes a game settles coming back
     through the handoff, follows from rulings 104, 113 and 114; rule it
     now, or keep it held for W5? Mark: "Rule it." So §5.2's points 2 to 5
     stand as the contract's shape, and W5 designs the details against
     them.

     Links: area Stack and engine · draws on 104, 113, 114 · summary D19's contract is ruled: a game submits intents, the sim returns events and a read-only per-tick view, and settled outcomes return through the handoff.
155. **What a Mesocosm player plays depends on the lineage's traits.** Put
     to Mark on 2026-09-24, from ruling 152 against ruling 59's "direct one
     or a cohort of critters": what does a Mesocosm player play? Two
     options were put: one critter, the cohort following it in the world;
     or the cohort as one entity. Mark: "Your relation to other critters in
     your lineage depends on your traits. In the most extreme cases, other
     critters in your lineage can be ecological competitors or direct
     extensions of your own critter (fungus monocreature, swarmlike
     entities like germs, etc.)"

     Links: area Mesocosm · summary What a Mesocosm player plays depends on the lineage's traits; other lineage critters may be competitors or direct extensions.
156. **A DM may take up any unclaimed entity to play, and may edit the
     world.** Put to Mark on 2026-09-24, from ruling 152: what can a DM do
     from outside the world? Three options were put: play any NPC; edit,
     not direct; or both. Mark: "Both."

     Links: area The VTT · draws on 152 · summary A DM may take up any unclaimed entity to play, and may edit the world.
157. **A thing of note is a relic, an event of note a tale, and
     Mesocosm's body site a tract.** Put to Mark on 2026-09-24 as the
     of-note naming round open since ruling 69, places of note being
     locations and entities of note denizens: what is the word for a thing
     of note, for an event of note, and, to free *site* for the world map,
     for Mesocosm's body site, one expressed process on a patch of a part's
     tissue? Options were put for each: relic, heirloom or none; tale,
     deed or none; tract, situs or keeping *site*. Mark picked relic, tale
     and tract. The body-site rename is a lane, about 104 occurrences in
     26 files by the 2026-09-20 count.

     Links: area Naming · draws on 69 · summary A thing of note is a relic, an event of note a tale, and Mesocosm's body site a tract.
158. **A mind's mood is fed by what it remembers, needs and lives
     through; personality traits are a class of traits of their own.** Put
     to Mark on 2026-09-24, opening mood and breaks: what is a mind's mood
     made of? Four options were put, to pick all that should feed it: what
     it remembers, the tales it holds by intensity and valence; what it
     needs; what it is living through; and its disposition as a baseline.
     Mark picked the first three and, in place of the fourth: "was
     figuring we could have a special class of traits, personality traits;
     inheritable in a seed form, developable if you take actions that feed
     the trait; compounding +/- modifiers to other mental/physical aspects
     as they develop... some maybe preclude aspects of play for special
     bonuses? (like brawler in rimworld) idk. i like being able to invest
     in traits with significant activity".

     Links: area The sim's schema and saves · summary A mind's mood is fed by what it remembers, needs and lives through; personality traits are an inheritable, developable class of traits of their own.
159. **Mood is read; strain is kept.** Put to Mark on 2026-09-24: is mood
     kept, or read? Three options were put: read, never kept; kept as a
     level, as vigour is; or read, with strain kept. Mark: "Read, with kept
     strain." So mood is derived on demand from its sources, while strain
     builds as a kept level whenever mood stays low, and only rest or
     relief bleeds it off.

     Links: area The sim's schema and saves · draws on 158 · summary Mood is read on demand from its sources; strain is a kept level that builds while mood stays low and eases with rest or relief.
160. **Personality traits sit on the five factors.** Put to Mark on
     2026-09-24, from ruling 158 against ruling 37: how do personality
     traits stand to the five-factor disposition? Three options were put:
     traits on the factors; traits replacing them; or the factors as
     traits. Mark: "Traits on the factors." So the five factors stay the
     continuous temperament, and personality traits are discrete features
     grown on top, their seeds drawn from the factors.

     Links: area The sim's schema and saves · draws on 158, 37 · summary Personality traits are discrete features grown on top of the five-factor continuous temperament, their seeds drawn from the factors.
161. **Past what it can bear, a mind stops heeding, acts out, or rises,
     and the break leaves a mark.** Put to Mark on 2026-09-24: what
     happens when strain passes what a mind can bear? Four options were
     put, to pick all that should: it stops heeding directives until it
     recovers; it acts out, a break drawn from its traits and situation;
     it may rise instead, a breakthrough such as the fey mood or a last
     stand; and it leaves a mark, seeding or feeding a personality trait.
     Mark picked all four.

     Links: area Processes and rules · draws on 159 · summary Past what it can bear a mind stops heeding directives, acts out, or rises; the break leaves a mark seeding or feeding a personality trait.
162. **Breaking does not spread.** Put to Mark on 2026-09-24: does
     breaking spread from one mind to others? Two options were put: it
     spreads, a panic or a riot running through a crowd; or it doesn't.
     Mark: "It doesn't." So each mind breaks alone, and a riot is many
     minds breaking for the same reasons.

     Links: area Processes and rules · draws on 161 · summary Breaking does not spread; each mind breaks alone and a riot is many minds breaking for the same reasons.
163. **Whether a break goes down or up turns on traits, the moment and
     chance.** Put to Mark on 2026-09-24: when strain breaks a mind, what
     decides whether it breaks down or rises? Three options were put, to
     pick all that should count: its traits; the moment, what is at stake
     and who depends on it; and chance, a seeded draw weighted by the rest.
     Mark picked all three.

     Links: area Processes and rules · draws on 161 · summary Whether a break goes down or up turns on traits, the moment, and seeded chance.
164. **What a mind can bear is set by its traits, its support and what it
     has been through.** Put to Mark on 2026-09-24: what sets how much
     strain a mind can bear? Three options were put, to pick all that
     should: its traits; its support, company, faith, a home and what it
     believes in; and what it has been through, strain survived hardening
     it and past breaks scarring it. Mark picked all three.

     Links: area Processes and rules · draws on 161 · summary What a mind can bear is set by its traits, its support, and what it has been through.
165. **A culture is read from its people, and named when noted.** Put to
     Mark on 2026-09-24, opening language and culture: what is a culture
     in the sim? Three options were put: read from people, with no state
     of its own; a thing of its own, kept like a polity without a
     constitution; or read, and named and remembered once minds hold it as
     a thing. Mark: "Read, named when noted."

     Links: area The sim's schema and saves · summary A culture is read from its people, with no state of its own, and named when noted.
166. **Languages divide.** Put to Mark on 2026-09-24: do sophonts speak
     languages that can divide them? Three options were put: languages
     divide, understanding needing a shared tongue learned like a skill;
     names only, languages as the generated flavour of names; or no
     languages. Mark: "Languages divide." So telling, posing and exchange
     across tongues come harder, and interpreters matter.

     Links: area The sim's schema and saves · summary Languages divide: understanding needs a shared tongue learned like a skill, and interpreters matter.
167. **Tongues and cultures descend like lineages.** Put to Mark on
     2026-09-24: do tongues and cultures descend like lineages? Three
     options were put: like lineages, splitting into dialects when their
     speakers part and drifting over deep time; fixed per people; or
     generated with no descent. Mark: "Like lineages."

     Links: area Processes and rules · summary Tongues and cultures descend like lineages, splitting into dialects when speakers part and drifting over deep time.
168. **Names come from the tongue, from tales, and from the namer.** Put to
     Mark on 2026-09-24: how are names made? Three options were put, to
     pick all that should: from the tongue, generated from a language's
     sounds and words; from tales, epithets earned by deeds and stories;
     and given by the namer, ruling 36's naming being a sapient's own act.
     Mark picked all three.

     Links: area Processes and rules · draws on 36 · summary Names come from the tongue, from tales, and from the namer.
169. **Culture spreads by contact, prestige and imposition, and drifts
     apart out of touch.** Put to Mark on 2026-09-24: how does culture
     spread between peoples? Four options were put, to pick all that
     should: contact, neighbours growing alike faster the more they share;
     prestige, copying those admired or powerful; imposition by a polity's
     means; and drift apart when out of touch. Mark picked all four.

     Links: area Processes and rules · summary Culture spreads by contact, prestige and imposition, and drifts apart out of touch.
170. **Art and ritual are crafts, and any craft or pursuit can be
     politicized or ritualized.** Offered on 2026-09-24 as a next thread,
     "whether sophonts make art and hold rites, and what those do", Mark:
     "I view art and ritual as crafts. A craft/pursuit can be
     politicized/ritualized too. Let's proceed to tech".

     Links: area The sim's schema and saves · summary Art and ritual are crafts, and any craft or pursuit can be politicized or ritualized.
171. **Technology is both a world's generated tree and what is known.** Put
     to Mark on 2026-09-24, opening technology: what is technology in the
     sim? Three options were put: what is known, read from the techniques
     and recipes minds and bearers hold, a tree emerging from their
     preconditions; an authored or generated tree peoples climb; or both,
     a world's generated tree setting what can be known and what is known
     read from minds and bearers. Mark: "Both." Put with it as a reading
     from rulings 129 and 142, unobjected: a technique no mind or bearer
     holds is lost, and can be found again.

     Links: area The sim's schema and saves · draws on 129, 142 · summary Technology is both a world's generated tree and what is known, read from minds and bearers; an unheld technique is lost and can be refound.
172. **Ages are read and named, capped by the founder, can realign the
     world, and are the epochs of cultures and society.** Put to Mark on
     2026-09-24: does a world have ages of technology? Four options were
     put, to pick all that should hold: read and named, a reading of what
     the peoples of a time commonly know; world epochs, turning at the
     world's epoch boundaries; a founder's cap on how far technology can
     go; and ages that realign the world to a new scope. Mark picked the
     first, third and fourth and, in place of the second: "Ages are the
     epochs of cultures/society."

     Links: area Record and history · draws on 171 · summary Ages are read and named, capped by the founder, can realign the world, and are the epochs of cultures and society.
173. **Invention is driven by need, contact, temperament and mastery.** Put
     to Mark on 2026-09-24: what drives invention, a master going beyond
     the canon (ruling 142)? Four options were put, to pick all that
     should: need; contact, ideas combining when peoples meet; temperament,
     curious and open minds; and mastery, deep skill making breakthroughs
     possible. Mark picked all four.

     Links: area Processes and rules · draws on 142 · summary Invention is driven by need, contact, temperament and mastery.
175. **The first Mesocosm overlay directs, as ruling 60 has it; the played
     slice's direct control is rewritten.** Put to Mark on 2026-09-25, from
     the finding that the played slice plan gives direct control of the
     organism, an intent over `World.controlled`, while ruling 60 made
     directing Mesocosm's mode the day after W1 kept that plan: for the
     first overlay's played loop, directing as ruled, directing with a
     hands-on mode for moments like a hunt, or the slice's direct control
     first? Mark: "Directing, as ruled." So the critter acts on its own
     needs and senses, which suggest to the player, and the player shapes
     it with standing orders and priorities. **The rewrite is owed now,
     and the plan retires into W5 at M3 (rulings 196 and 199).**

     Links: area Mesocosm · draws on 60 · summary The first Mesocosm overlay directs, as ruling 60 has it; the played slice's direct control is rewritten.
176. **A Mesocosm player directs with priorities, places, stances and
     nudges.** Put to Mark on 2026-09-25: what does a Mesocosm player
     direct with? Four options were put, to pick all that should be in the
     vocabulary: priorities among needs and abilities; places to range,
     avoid and make home, the desire paths of ruling 59; stances, bold or
     cautious and contest, yield or share; and nudges, one-off suggestions
     the critter weighs. Mark picked all four. **How, by rulings 214 to
     216:** the player gives the nudge by click, and the other three grow
     from the nudges' history as desire paths.

     Links: area Mesocosm · draws on 59 · summary A Mesocosm player directs with priorities, places, stances and nudges.
177. **A critter follows its player's directing by its bond.** Put to Mark
     on 2026-09-25: does your own critter always follow your directing?
     Three options were put: within its body; when its needs allow; or by
     its bond, built from how well your orders have served it, as ruling
     60 weighs directives by opinion. Mark: "By its bond."

     Links: area Mesocosm · draws on 60 · summary A critter follows its player's directing by its bond, built from how well the player's orders have served it.
178. **Whether the bond carries across generations is a setting, seeded
     by default.** Put to Mark on 2026-09-25: when your critter dies and
     you take up the next of your lineage (ruling 61), does the bond carry
     over? Three options were put: it starts fresh; it is seeded by the
     lineage, inherited in seed form as personality traits are; or it is
     the lineage's, carrying whole. Mark: "All three should be options; my
     default would be seeded".

     Links: area Mesocosm · draws on 177, 61 · summary Whether the bond carries across generations is a setting (fresh, seeded by lineage, or whole); seeded is the default.
179. **A Mesocosm player chooses when to start, from the world's
     habitability for their critter onward.** Put to Mark on 2026-09-25:
     does the first played loop start in a world with a past, deep time
     first or a bare world first? Mark: "You may choose to start at the
     start of the world's habitability for your critter, or add as much
     time as you'd like".

     Links: area Mesocosm · summary A Mesocosm player chooses when to start, from the start of the world's habitability for their critter or with as much added time as wanted.
180. **What a Mesocosm player sees is a mode: survival shows what the
     critter knows, creative shows the truth.** Put to Mark on 2026-09-25,
     from the playable ecology plan's open ruling 6: how much of the
     ecology's truth does a Mesocosm player see during live play, versus
     at the epoch review? Three options were put: what it knows; truth at
     the review; or truth anytime. Mark: "Creative vs survival mode, if you
     ask me. Both worthy and necessary, debug-wise."

     Links: area Mesocosm · summary What a Mesocosm player sees is a mode: survival shows what the critter knows, creative shows the truth.
181. **A trophic collapse may be local or global, and losing is not the
     end.** Put to Mark on 2026-09-25, from the playable ecology plan's
     open ruling 7: what makes a trophic collapse terminal in the game?
     Three options were put: producers gone; no return within a stated
     span; or a world setting. Mark: "Like rimworld, you can keep going
     after you lose, to see what happens. There's also a question of a
     trophic collapse being local or global". And, the same turn: "if a
     trophic collapse isn't everywhere, you can probably start again
     somewhere else, perhaps even with your lineage if it's still there. Or
     you know, perhaps your critter can help the world by doing something
     to regrow producers or fix the problem. Depends on their ability."
     **Refined by rulings 225, 226, 229 and 230:** a region collapses when
     a trophic level is gone there; regions are ecological and merge as
     the world's biomass falls, so a global collapse is the one region's;
     a player whose lineage ends picks another here or a new world; and a
     total collapse ends the world as a game, still watchable.

     Links: area Mesocosm · summary A trophic collapse may be local or global, and losing is not the end: play can continue or restart elsewhere. (amended by 225, 226, 229, 230)
182. **Unplayed lineages adapt, inherit and develop, weighing their
     adaptation against the rest of the web; that is what an epoch is
     about.** Put to Mark on 2026-09-25, from the playable ecology plan's
     open ruling 5: do unplayed lineages discover new developmental
     vocabulary, or only weigh what is inherited or already discovered?
     Two options were put: the same evidence rules as the player's, or
     inherited only. Mark's tap on "Inherited only" was accidental, by his
     word the same turn, and his answer is: "unplayed lineages also adapt,
     inherit, and develop. And they should weigh their adaptation against
     the rest of the troposphere. I think that's what the epoch should be
     about, aside from changes in gameplay that might randomly occur
     (individual variance?)". Asked whether "troposphere" meant the rest of
     the trophic web, the ecosystem the lineage lives in, Mark: "That is
     correct".

     Links: area Ecology · draws on 57 · summary Unplayed lineages adapt, inherit and develop, weighing their adaptation against the rest of the trophic web; that is what an epoch is about.
183. **At a birth, the player keeps the parent by default.** Put to Mark on
     2026-09-25, from the playable ecology plan's open ruling 1: when your
     critter reproduces, do you keep playing the parent or move to the
     offspring? Three options were put: choose each time; parent by
     default; or offspring by default. Mark: "Parent by default." So the
     player keeps the parent unless they take the offspring, and the
     choice can be taken back, which is the placeholder the plan already
     built.

     Links: area Mesocosm · summary At a birth the player keeps the parent by default and may take the offspring; the choice can be taken back.
184. **Creative mode shows the truth and does not edit.** Put to Mark on
     2026-09-25: does creative mode (ruling 180) also let the player change
     the world? Two options were put: see and edit; or see only, editing
     belonging to the world editor (ruling 89). Mark: "See only."

     Links: area Mesocosm · draws on 180, 89 · summary Creative mode shows the truth and does not edit; editing belongs to the world editor.
185. **Eponym's co-op is each player living their own creature.** Put to
     Mark on 2026-09-25, opening Eponym's overlay: what does co-op look
     like, given that ruling 153 lets two players direct one entity? Three
     options were put: each their own; one life, shared; or both as a
     setting. Mark: "Each their own."

     Links: area Eponym · summary Eponym's co-op is each player living their own creature.
186. **At a death in Eponym, the player chooses who to become, among
     bonded companions.** Put to Mark on 2026-09-25: when the creature you
     live dies and a companion becomes the one you play, who decides which
     companion? Three options were put: the player chooses, among
     companions with a bond to the one who died; the closest bond; or
     whoever takes up the role by the world's rules. Mark: "The player
     chooses."

     Links: area Eponym · draws on 153 · summary At a death in Eponym the player chooses who to become, among bonded companions.
187. **Survival and creative modes hold for Eponym as for Mesocosm.** Put
     to Mark on 2026-09-25: do rulings 180 and 184 hold for Eponym? Two
     options were put: the same, creative also being where tag-in lives;
     or survival only. Mark: "The same."

     Links: area Directing and play · draws on 180, 184 · summary Survival and creative modes hold for Eponym as for Mesocosm; creative is also where tag-in lives.
188. **A VTT campaign may run with the sim off.** Put to Mark on
     2026-09-25, opening the VTT's overlay: can a campaign run with the sim
     switched off, as a plain board with no world beneath? Three options
     were put: always a world; sim off allowed; or a dial of how much the
     world moves on its own. Mark: "Sim off allowed."

     Links: area The VTT · summary A VTT campaign may run with the sim off, as a plain board.
189. **An uncalibrated ruleset may play in a debug or experimental mode,
     with a warning.** Put to Mark on 2026-09-25: what happens to a system
     plugin that cannot pass ruling 114's check? Three options were put:
     refused on a world; allowed and flagged; or the DM's choice. Mark:
     "We should maybe consider allowing it debug, or experimental (with a
     warning)".
     *Bounded 2026-10-03 by ruling 538:* calibration, and so this mode,
     applies in worlds the sim leads.

     Links: area The VTT · draws on 114 · summary An uncalibrated ruleset may play in a debug or experimental mode, with a warning. (amended by 538)
190. **An adventure pack whose requirements the world lacks waits, or the
     GM forces it.** Put to Mark on 2026-09-25: an adventure pack names
     what it needs from a world, a ruin, a faction at war, a missing heir;
     what if the world doesn't have it? Three options were put: the pack
     waits, dormant until the world meets it; the DM forces it, asserting
     its content and displacing generated content (ruling 89); or the
     world bends at founding. Mark: "Wait or the gm forces it".

     Links: area The VTT · draws on 89 · summary An adventure pack whose requirements the world lacks waits dormant, or the GM forces it, displacing generated content.
191. **The VTT offers the sim's running arcs to the DM as suggested
     hooks.** Put to Mark on 2026-09-25: the sim knows the parts of the
     arcs it is running (ruling 103); does the VTT offer them to the DM?
     Three options were put: as suggested hooks, which the DM may take up,
     drop or reshape; on request; or not at all. Mark: "As suggested
     hooks."

     Links: area The VTT · draws on 103 · summary The VTT offers the sim's running arcs to the DM as suggested hooks to take up, drop or reshape.
192. **Isocosm absorbs mesocosm-core; the first overlay plays on
     Isocosm.** Put to Mark on 2026-09-25, before drafting the Mesocosm
     overlay plan, with the sizes measured: `mesocosm-core` 44,901 lines
     holding the live ecology, bodies, matter, places, deep time and
     record; `shared/isocosm` 3,208 lines, used only by the bench; how do
     they become one sim, as the rule against a stage growing its own
     engine requires? Three options were put: Isocosm absorbs,
     mesocosm-core's sim moving in piece by piece re-expressed in its
     process definitions (ruling 32), Mesocosm's own core shrinking to
     directing, presentation and the review; mesocosm-core becomes the
     sim; or a bridge now and convergence later. Mark: "Isocosm absorbs."
     *Amended 2026-10-05 by ruling 591:* the legacy sims move first and are
     re-expressed in place.

     Links: area Stack and engine · draws on 32 · summary Isocosm absorbs mesocosm-core: its sim moves in piece by piece re-expressed in process definitions, and the first overlay plays on Isocosm. (amended by 591)

193. **Moved modules are decomposed under the 600-line ceiling.** Mark,
     2026-09-25, while the Mesocosm overlay plan was being drafted: "Feel
     free to decompose them under the 600 Loc limit". So every family that
     moves into Isocosm is split as it goes, along seams the code already
     has.
     *Enforced 2026-10-05 by ruling 592:* every tracked source file, by
     script, exceptions listed with their reason.

     Links: area Method and documents · summary Moved modules are decomposed under the 600-line ceiling along seams the code already has.

197. **The overlay contract is one shared crate, `isocosm-overlay`.** Put
     to Mark on 2026-09-25, opening M1: does the contract crate live as one
     shared crate, as Mesocosm's own first, or as a shared core plus
     Mesocosm's vocabulary? Mark: "One shared crate." And its name, from
     `isocosm-contract`, `wing-overlay` or `isocosm-overlay`: Mark:
     "isocosm-overlay". So it sits in `shared/`, game-neutral, with
     Mesocosm's directives as its first game's module and Eponym and the
     VTT to add theirs.

     Links: area Stack and engine · draws on 154 · summary The overlay contract is one shared game-neutral crate, isocosm-overlay, in shared/, with each game's directives as its own module.
198. **The consistency pass is applied, and a dated line keeps its date's
     words.** Put to Mark on 2026-09-25, with the RPG systems session's
     read-only pass over this record, the sim plan and the overlay plan at
     commit 0f249ae: thirty items, each checked against the files. Its
     bookkeeping items, where a later ruling superseded an earlier one
     without a note, a number was mis-cited, a status or names line had
     gone stale, or an old word stood in present prose, each carried one
     fix: apply as proposed, or walk through them? Mark: "Apply as
     proposed." And item 10, where the rename's R1 (commit f4689b7) had put
     Isocosm into lines dated 2026-09-18, so that §9.1 said Isocosm was
     ruled that day and is a 2015 iOS puzzle game: restore isotropy, or
     keep Isocosm with a note? Mark: "Restore isotropy." So a dated line
     keeps the words of its date, as a verbatim ruling keeps its own, and a
     later name reaches it only as a note.

     Links: area Method and documents · summary The consistency pass is applied; a dated line keeps the words of its date and a later name reaches it only as a note.
200. **Of note is identified, a denizen is named, and a sophont is
     sapient.** Put to Mark on 2026-09-25, from the pass's item 9: is
     sophont settled as the second identity word, which ruling 45 left to
     him? Mark: "Denizen as the umbrella word, sophont as term of art for
     sapient entities", and then, "Denizen = named entity". Asked how that
     meets the 2026-09-20 line that a denizen implies no name: "Noted means
     identified, naming requires sapience, denizen = named, so every
     sophont is a denizen but not every denizen has to be a sophont. Of
     note then naturally transitions into named as people need to refer to
     the entity of note". So the words are three steps. An entity of note
     is one the sim identifies; it becomes a denizen when people name it,
     as they need to refer to it; a sophont is a sapient entity, and every
     sophont is a denizen. The tier is "of note", and denizen no longer
     names it. At his word ("Amend as drafted"), Mesocosm's CLAUDE.md
     denizen line is rewritten to match and a sophont line added.

     Links: area Naming · amends 69 · summary Of note means identified, denizen means named, sophont means sapient (every sophont is a denizen); denizen no longer names the of-note tier.
201. **The organ words are components, not crates, and a `denizen` crate
     needs a reason of its own.** Mark, 2026-09-24, in the RPG systems
     session, on hagioglyph, impresa, denizen, isoscape and isostasy: "Ehh,
     the organ words can be rendered as components in the repo. They
     needn't be crates." Put to him on 2026-09-25, from the pass's item 20:
     does that also drop publishing a `denizen` crate to reserve the name?
     Mark: "We need a good reason for a denizen crate. If it's just a
     component in the sim engine, nah. But if it is a way to
     transfer/import/export entity information and use it in different,
     potentially embedded contexts, that could be good". So no crate is
     published for an organ word, and a `denizen` crate is made only if it
     becomes the way entity information moves between contexts.

     Links: area Organs: glyphs, impresa, hagiograph · draws on 200 · summary Organ words are components, not crates; no crate is published for one, and a denizen crate needs a reason such as moving entity information.
202. **Speciating stays the player's act; expressing becomes the
     critter's own.** Put to Mark on 2026-09-25, from M1, where
     `mesocosm-core`'s `Speciate`, splitting the played line and naming it,
     and `Express`, spending this body's development on a discovered
     candidate now, fit neither a checkpoint nor the review: both at the
     boundary, both the player's mid-round, both the critter's own, or
     split? Mark: "Split them." So a player may split their line and name
     it, "the name is the doing", as a player's act beside directives and
     checkpoint answers; and expressing a candidate is the critter's own
     development, chosen by its methodology and steered by priorities.

     Links: area Mesocosm · summary Speciating (splitting and naming the played line) stays the player's act; expressing a candidate becomes the critter's own development.
203. **The tick stamp is a newtype over the sim's count.** Put to Mark on
     2026-09-25, the first of three forks from building M1: a newtype over
     `u64`, a plain `u64` as the sim uses, or an epoch and an offset, a
     structure the sim does not store? Mark: "Newtype over u64."

     Links: area Stack and engine · draws on 197 · summary The tick stamp is a newtype over the sim's u64 count.
204. **Subscription derives from the attention set, whose type M1
     designs.** Put to Mark on 2026-09-25, the second fork: §5.2's point 4,
     in the contract's shape since ruling 154, has one attention set per
     player drive the collector, the foreground and the event stream, but
     the set has no type. Until it has one, does the contract carry
     explicit topic keys or a structured filter, or does M1 type the
     attention set? Mark: "Type the attention set." So M1 is not done
     until the attention set has its type and subscription derives from it.

     Links: area Stack and engine · draws on 154, 197 · summary Subscription derives from the attention set, which M1 types; the contract carries no separate topic keys.
205. **A PLACES directive names a node of the place graph.** Put to Mark on
     2026-09-25, the third fork: an opaque handle to a place-graph node
     (rulings 72, 147), a site, or raw coordinates? Mark: "Place-graph
     handle." Dev intents keep raw coordinates, since they reach the grid
     directly.

     Links: area Mesocosm · draws on 72, 147 · summary A PLACES directive names a place-graph node by opaque handle; dev intents keep raw coordinates.
206. **When food is short, the sim resolves the crowd's choices in
     pairwise encounters.** Put to Mark on 2026-09-25, from the sim plan's
     S2 probe, which stopped at its first fork on finding no exact
     reference for feeding under scarcity, an eater taking the first
     eligible target in id order: pairwise encounters, a ranking at each
     site, or arrivals in turn? Mark: "Pairwise encounters." So hungry
     members meet in random pairs. A contester takes the ration from a
     yielder or a sharer, two sharers split it, and two contesters size
     each other up by body reserve, only a close match escalating, at a
     cost to both (rulings 115, 116). The margin and the cost are world
     data drawn per seed; escalation costs body reserve until vigour is in
     the ledger (ruling 123); and trade waits for something to trade.

     Links: area Processes and rules · draws on 115, 116 · summary When food is short, the sim resolves the crowd's choices in pairwise encounters: contesters take from yielders, sharers split, two contesters size up by reserve. (amended by 217)
207. **The crowd runner keeps a histogram of exact states.** Put to Mark
     on 2026-09-25, with 206: a histogram of exact states, coarsened
     states, moment closure, or super-individuals? Mark: "Exact-state
     histogram." So members are grouped by their exact state as the rules
     read it, with random count draws each round, and the pairing draw is
     the only approximation; a field is coarsened only where the rules show
     it safe, and savings are reported in evaluations.

     Links: area Processes and rules · draws on 113 · summary The crowd runner keeps a histogram of exact states, with random count draws; the pairing draw is the only approximation.
208. **What later processes read is the rules' own thresholds.** Put to
     Mark on 2026-09-25, from ruling 113's test: the fungible must agree in
     distribution over what later processes read, so what counts, counts
     and one starvation threshold, the rules' thresholds, or the full
     distribution? Mark: "The rules' thresholds." So the readings are
     every threshold the process definitions query, read mechanically from
     them, plus a distribution per field for anything an inspection shows.
     *Reading, 2026-09-26, not ruled:* the check compares readings one at a
     time, so any conjunction of thresholds one rule tests together, hungry
     and strained say, is a reading of its own.

     Links: area Processes and rules · draws on 113 · summary What later processes read is the rules' own thresholds, read mechanically from the process definitions, plus a distribution per field for anything inspection shows.
209. **The first tolerance is 0.2 per reading, certified with controls.**
     Put to Mark on 2026-09-25, with 208: a first bound of 0.2 or 0.1? Mark:
     "0.2, with controls." So each reading's bound sits in the world's
     rules, first at 0.2 (Kolmogorov-Smirnov distance) and tightened once
     the core's overheads are fixed. Agreement is tested for both
     difference and equivalence, corrected across readings, beside a
     positive control, the exact runner against itself under a dynamics
     seed kept apart from the founding seed, and a negative control, a
     crowd that averages reserves, which must fail the starvation check.
     *Amended 2026-10-02 by ruling 507:* in a body world the averaged crowd
     averages each lineage's own matter, tissue and reserve, since a
     lineage that keeps no store (506) has no reserve to average.

     Links: area Processes and rules · draws on 208 · summary The first tolerance is 0.2 per reading (Kolmogorov-Smirnov), certified with positive and negative controls and tightened later. (amended by 507)
210. **A player may pin any pointable thing.** Put to Mark on 2026-09-25,
     typing the attention set (ruling 204): what can a player pin, which
     ruling 130 keeps on the player's account? Three options were put: any
     pointable thing, entities and places, or entities only. Mark: "Any
     pointable thing." So entities, places, items, lineages, factions and
     events, anything that can have an impresa, can be pinned.

     Links: area Directing and play · draws on 204, 130 · summary A player may pin any pointable thing: entities, places, items, lineages, factions and events.
211. **An attended group runs its noted members exactly and the rest as a
     crowd.** Put to Mark on 2026-09-25, with 210: when what a player cares
     about is a group, such as Mesocosm's lineage, does every member run
     exactly, grouped without loss, or do the noted run individually and
     the fungible as a crowd, like the rest of the world? Mark: "Noted
     exact, rest crowd." So caring for a group keeps it, its noted members
     run as individuals (ruling 113), and its fungible members run as a
     crowd within the world's tolerance.

     Links: area Processes and rules · draws on 113 · summary An attended group runs its noted members exactly as individuals and its fungible members as a crowd within tolerance.
212. **Examining is what the view shows up close.** Put to Mark on
     2026-09-25, with 210: what counts as examining, which ruling 71 makes
     play and ruling 113 logs: everything in view, what the view shows at
     full detail, or what the player inspects? Mark asked which is more
     co-op friendly: "My impulse was everything, but are there arguments
     for syncing worldstate that would argue for the other two?" Answered
     from the record's sync, where intents are the log and peers replay
     (§5.2 point 2; ruling 104's merges): the grain costs no bandwidth, but
     every peer runs the union of all foregrounds, merges conflict over
     foregrounded material, everything in view would put camera moves in
     the log, and what is inspected breaks views that draw individuals. Put
     back with that and, at his asking, a recommendation, Mark: "In view, up
     close." So what the view shows at full detail, the terrarium section or
     a battlemap, is watched and runs in detail, and the watch is logged
     when that region changes; far views, an overmap or the minimap, stay
     crowd, and the camera stays presentation inside a region.

     Links: area Directing and play · draws on 71, 113 · summary Examining is what the view shows at full detail up close; that is watched, runs in detail and is logged; far views stay crowd.
213. **A player's event stream carries what is attended.** Put to Mark on
     2026-09-25, with 212: events touching anything in the attention set,
     or everything the collector keeps from it? Mark: "Depends on the co-op
     consideration. I don't know which is a simpler regime to synchronize:
     detailed but less, or everything and compressible... i lean to the
     former". Answered that under replay the stream never crosses the
     network, each peer deriving its own player's stream from the shared
     sim, so the smaller stream wins locally too. Put back, Mark: "What's
     attended." So a stream carries events touching who the player plays,
     what they pin, what they examine and the game's own care, with record
     entries reaching the played entity, and in survival mode only what the
     played critter can know (ruling 180).

     Links: area Directing and play · draws on 212, 180 · summary A player's event stream carries what is attended: who they play, what they pin and examine, and the game's own care.
214. **A click draws the critter's attention; attending is the default, and
     a right click picks another meaning.** Put to Mark on 2026-09-25, from
     the played slice plan's open question of how a player gives directives
     without a menu. Mark: "Could you click somewhere to draw attention to a
     place or thing?" Put back as what a click asks of the critter: attend
     to this, one nudge whose answer the critter decides by its needs,
     senses and mood, weighed by its bond; or a meaning set by the target, a
     place meaning go and food meaning eat. Mark: "Attend to this is
     default; pick alt meanings by right click". So a click on a place or a
     thing is a nudge (ruling 176) to attend to it, which the critter
     answers by its own lights and weighs by its bond (ruling 177), and a
     right click picks a more particular meaning. *Reading, not ruled:*
     those meanings are the critter's own acts toward the target, what its
     biology can do with it (ruling 59), and each is still a nudge the
     critter weighs.

     Links: area Mesocosm · draws on 176, 177 · summary A click draws the critter's attention as a nudge it weighs by its bond; attending is the default and a right click picks another meaning.
215. **A click never warns.** Put to Mark on 2026-09-25, with 214: can a
     click warn as well as invite, by a second gesture, or does the critter
     read the pointing through its own sense of the thing? Mark: "One
     gesture only." So no gesture warns: the critter reads what it is shown
     by its own sense of it, and avoidance comes from its experience and its
     stance.

     Links: area Mesocosm · draws on 214 · summary A click never warns; the critter reads what it is shown by its own sense, and avoidance comes from experience and stance.
216. **Standing orders grow from attention.** Put to Mark on 2026-09-25,
     with 214: where do the standing orders come from, places to range and
     make home, priorities and stances: grown from attention, set at the
     review between rounds, or grown and adjustable there? Mark: "Grown
     from attention." So they are the desire paths of ruling 59: where the
     player's attention keeps leading and the critter keeps thriving
     becomes its range and its home, and what the player keeps pointing at
     rises among its priorities. The player sets none of them directly, so
     the only directive a player sends is the nudge. *Reading, not ruled:*
     stances grow the same way, from how the encounters the player's
     attention led it into turned out, and places to avoid from what went
     badly there (215).

     Links: area Mesocosm · draws on 59, 214 · summary Standing orders (ranges, homes, priorities, stances) grow from the player's attention as desire paths; the only directive sent is the nudge.
217. **The larger reserve does not win an escalated fight; the probe's
     other readings stand.** Put to Mark on 2026-09-25, with the S2 probe's
     four readings of ruling 206: each member's choice follows its lineage's
     leaning, contest-leaners contesting and the rest sharing, and yielding
     is the smaller contester's answer to sizing up; every pair gets one
     ration before any gets two, an odd member going without; an escalated
     fight goes to the larger reserve, a tie to a seeded coin; and a fight's
     cost is capped at what a member holds, sharers taking half each. Accept
     all four, revisit the fight, or walk through each? Mark: "Revisit the
     fight." So the first, second and fourth stand as readings of ruling
     206, and the fight is reworked around strain (rulings 221 to 223).

     Links: area Processes and rules · draws on 206 · summary The larger reserve does not win an escalated fight; the probe's other readings of ruling 206 stand and the fight is reworked around strain.
218. **Each reading's bound and the competition's definition sit in the
     world's rules.** Put to Mark on 2026-09-25, from the probe's deviation:
     it kept both beside the core's `Rules`, moving them in being a schema
     change; move both in now, or wait for S1? Mark: "Move both in now."

     Links: area The sim's schema and saves · summary Each reading's bound and the competition's definition move into the world's rules now.
219. **The interpreter's meanings exist once.** Put to Mark on 2026-09-25:
     the probe's `aggregate.rs` restates the interpreter's query and effect
     meanings for counts of members, the core refusing counted shared
     writes by design; fold both into shared functions now, or during M2?
     Mark: "Fold now." So the crowd and the individual runner call the same
     code.

     Links: area Processes and rules · summary The interpreter's query and effect meanings exist once, in shared functions called by both the crowd and the individual runner.
221. **In an escalated fight, strain builds against bearing, and cost adds
     to it.** Put to Mark on 2026-09-25, reworking the fight (217), the core
     having no strain yet: each round adds strain to both sides and the
     first past its bearing breaks; or the same, with what a round costs
     each side adding to its strain? Mark: "Strain vs bearing, plus cost."
     So each round of escalation adds strain to both sides, and what the
     round costs a side, reserve spent and wounds, adds more, so the side
     losing the exchange strains faster. A side whose strain passes its
     bearing breaks. Bearing is set by traits (ruling 164), with support and
     history joining when the core has them, and strain is kept, bleeding
     off only with rest or relief (ruling 159). This is the core's first
     strain.

     Links: area Processes and rules · amends 116, 217 · summary In an escalated fight each round adds strain to both sides, plus what the round costs them; a side whose strain passes its bearing breaks.
222. **A break in a fight advantages or disadvantages; it decides
     nothing.** Put to Mark on 2026-09-25, with 221: in a fight, does a
     break down yield and a break up prevail, or does any break yield?
     Mark: "Breaking up advantages, breaking down disadvantages. No auto
     win or lose". So a break up gives its side an advantage in the rounds
     that follow and a break down a disadvantage, the direction turning on
     traits, the moment and a seeded draw (ruling 163), and neither ends the
     fight.

     Links: area Processes and rules · draws on 221, 163 · summary A break in a fight advantages or disadvantages a side in later rounds; it decides nothing automatically.
223. **A fight ends when a side yields on re-sizing or is spent, whichever
     comes first.** Put to Mark on 2026-09-25, with 222: if a break only
     shifts the advantage, what ends a fight: re-sizing, being spent, or
     whichever first? Mark: "Whichever first." So after each round both
     sides size each other up again with the new advantage, and one that
     now reads itself outmatched yields (ruling 116); or a side whose
     vigour, its reserve until vigour is in the ledger (ruling 123), runs
     out is spent.

     Links: area Processes and rules · draws on 222 · summary A fight ends when a side yields on re-sizing or is spent (vigour or reserve out), whichever comes first.
225. **A region has collapsed when one of its trophic levels is gone.** Put
     to Mark on 2026-09-25, from ruling 181's open item, what measures a
     collapse: in one region, its producers gone, any trophic level gone, or
     its living biomass below a floor? Mark: "A level gone." So a region has
     collapsed when producers, consumers or decomposers (ruling 39) are
     empty there.

     Links: area Ecology · summary A region has collapsed when one of its trophic levels (producers, consumers, decomposers) is gone there.
226. **Ecological regions follow the world's biomass; a collapse is global
     when the world has become one region.** Put to Mark on 2026-09-25, with
     225: when is a collapse global, everywhere, when no refuge is in reach,
     or past a share of regions set in the world's rules? Mark: "The number
     of regions should be linked to the total world biomass: each trophic
     level fills its slot in a region until it reads as over capacity, then
     they spill/move to a new one. If you move into a region with lots of
     creatures on your trophic level, be prepared to compete for their
     position, basically. And so when we say everywhere, really if the
     total biomass goes down enough, everywhere is one huge region.
     Geographic concerns do matter, but if we're talking about the movement
     and ecology of the critters, then i would expect that we would see such
     a collapse spread depending on the cause... i wonder what threats of
     that scale are. A pandemic? Meteor? Polities killing the world? One
     overperforming lineage? Magic shenanigans? Idk. I do know that the
     world isn't done until your lineage is done and you need to pick a new
     one, or until the world's ecology is collapsed entirely." So the
     ecological region is not ruling 72's geographic one: each trophic level
     fills its slot in a region until it reads as over capacity, then spills
     into a new region, and a newcomer to a region full of its own level
     competes for their place. As the world's biomass falls the regions
     merge, until everywhere is one region, and a collapse there (225) is
     global. A collapse spreads by its cause; what threatens at that scale,
     a pandemic, a meteor, polities killing the world, one overperforming
     lineage or magic, is open. The world's ends are rulings 229 and 230.

     Links: area Ecology · draws on 225, 181 · summary Ecological regions follow world biomass, filling by trophic level and spilling over; they merge as biomass falls, and a collapse is global when one remains.
227. **The core reads mood from needs now.** Put to Mark on 2026-09-25, as
     Lane A gives the core its first strain, from fights: should the core
     read mood from needs now, so hunger and wounds build strain too, or
     should strain come from fights only until S1 designs the mind's schema?
     Mark: "Needs now." So mood is read from a member's needs as the core has
     them, low reserve, hunger and starvation, and wounds once they exist;
     strain builds whenever that mood stays low (ruling 159), fights adding
     theirs; memory and situation, mood's other sources (ruling 158), wait
     for S1.

     *2026-10-18, Mark, on how wounds feed mood:* "The hazard joins generated
     worlds (Recommended)": case 704's agentless hazard wounds parts in
     generated Mesocosm worlds at a drawn rate. "A wound share in the core
     (Recommended)": lost and severed cells over all cells, out of a hundred,
     read by a core query Eponym also uses; played worlds get a need of
     weight -10 from a quarter lost. "1 + 2": paid healing and a new rest act
     that heals slowly without paying both answer it. "Strain and the vitals
     panel (Recommended)": played worlds gain strain and relief, and
     Mesocosm's vitals panel shows wounds and mood.

     *2026-10-18, Mark, second round, after the build showed paid healing out
     of reach (12 mg a cell against a 10 mg founder):* "Mesocosm's generated
     worlds only (Recommended)": the hazard runs there and the default body
     founding keeps rate zero. On healing: "Maybe rest heals if you can pay
     the price, and some creatures can invoke that process without rest?",
     then "Rest pays part": resting heals at a fraction of the cell price,
     with a quarter (3 mg a cell, about one meal) as the option put, and
     growth:heal lineages pay the full price without resting. "One cell a
     tick (Recommended)" and "Any lineage": every lineage can rest, one cell
     a tick. *Reading, not ruled:* how a growth:heal lineage heals without
     resting, given that every act takes the tick's one choice, comes back
     as its own question.

     *2026-10-18, Mark, third round, after rest was built:* "Accept the
     measured rate (Recommended)": A1's bar for worlds with the hazard is the
     measured rate (bodied 19 of 20 and 39 of 40 in the first runs), not 20
     of 20; nothing is tuned for it. "The starving line, 2 mg": a rest never
     leaves a body below 2 mg. "As built (Recommended)": every wounded body
     that can pay rests, reactive ones included. "A cheaper rest
     (Recommended)": growth:heal lineages rest at an eighth of the cell price
     rather than a quarter; no full-price healing outside rest is built.

     Links: area Bodies and anatomy · draws on 158, 159 · summary The core reads mood from needs now (low reserve, hunger, wounds); strain builds whenever mood stays low; memory and situation wait for S1. 2026-10-18: a hazard wounds, a core wound share feeds mood, healing or rest answers it, strain builds and vitals shows it.
228. **A right click shows a ring of acts.** Put to Mark on 2026-09-25: how
     does a right click show its meanings (ruling 214): a ring of acts around
     the cursor, a cycle in place, or the critter's glance surfacing glyphs
     on the target? Mark: "A ring of acts." So the critter's possible acts
     toward the target appear around the cursor, and a click picks one.

     Links: area Mesocosm · draws on 214 · summary A right click shows a ring of acts around the cursor, the critter's possible acts toward the target, and a click picks one.
229. **When a player's lineage ends, they pick another here or a new world,
     their call.** Put to Mark on 2026-09-25, from 226's "the world isn't
     done until your lineage is done and you need to pick a new one": pick
     another lineage in the same world, a new world, or either? Mark:
     "Either, player's call."

     Links: area Mesocosm · draws on 226 · summary When a player's lineage ends, they pick another lineage in the same world or a new world, their call.
230. **A total collapse ends the world as a game, and it stays watchable.**
     Put to Mark on 2026-09-25, from 226's "or until the world's ecology is
     collapsed entirely", ruling 181 having had no collapse end play: after a
     total collapse, is the world done but watchable, or done and closed?
     Mark: "Done, but watchable." So ruling 181's play going on after a loss
     holds, and a total collapse is the one loss that ends the game while the
     world can still be watched.

     Links: area Mesocosm · draws on 226, 181 · summary A total ecological collapse ends the world as a game, but the world stays watchable.
232. **Eponym's blows are resolved in the foreground and handed back.** Put
     to Mark on 2026-09-25, from the Eponym overlay plan's §6 decision 1:
     Eponym's strike system resolves each blow geometrically and hands the
     harm back through the handoff in the sim's terms, calibrated to the
     sim's fight (ruling 114); or the sim's own contest runs at foreground
     fidelity, Eponym supplying the actuation alone? Mark: "Foreground hands
     back." So Eponym's handoff carries vigour drained and wounds to parts,
     which must pass the sim's invariants and agree with the sim's own fight
     in distribution (rulings 114, 123, 154), as the record's §3.8 has it.

     Links: area Processes and rules · summary Eponym's blows are resolved in the foreground and handed back as vigour drained and wounds to parts, agreeing with the sim's fight in distribution.
233. **Eponym's motion and contact solver runs on the game side.** Put to
     Mark on 2026-09-25, from the same decision's second half: on the game
     side over the stack's conatus, the accepted transition for each tick
     crossing as the intent, or in the sim, fed the input frames? Mark:
     "Game side."
     *Carried out 2026-10-05 by ruling 597:* motion and contact move to
     `eponym-motion`; the sim is handed the solver.

     Links: area Eponym · summary Eponym's motion and contact solver runs on the game side over the stack's conatus.

234. **Where a first Eponym life begins in time is the player's pick, with
     society the default.** Put to Mark on 2026-09-25, from decision 2: from
     the world's habitability for the creature onward, as ruling 179 gives
     Mesocosm, only once settlements stand, or the player's pick with the
     second as the default? Mark: "Player's pick, society default."

     Links: area Eponym · summary Where a first Eponym life begins in time is the player's pick, with society the default.
235. **How a first Eponym life begins is the player's call among three.**
     Put to Mark on 2026-09-25, from decision 3: a newly generated outsider
     arriving, a birth into one of the world's lineages, a named creature
     chosen among the drawn world's denizens, or any, the player's call?
     Mark: "Any, player's call."

     Links: area Eponym · summary How a first Eponym life begins is the player's call among a new outsider, a birth into a lineage, or a named drawn denizen.
236. **Competitions are keyed by what is contested, and share a round one
     per member per tick, by need.** Put to Mark on 2026-09-26, from S2's
     checkpoint 1, which stored competitions as a map keyed by id, ruling 115
     naming many scarce things, while the probe refused a world with more than
     one: keep the map and design sharing later, design it now, or one per
     world? Mark: "Design sharing now." Asked how: one per tick by need, all a
     member needs with costs adding, or as the world's rules say? Mark: "One
     per tick, by need." So each member enters only the competition for its
     most pressing need that tick, set by its needs and priorities, and each
     competition resolves over its own entrants. **Amended by ruling 240:**
     competitions run concurrently in a tick and settle at its end.

     Links: area Processes and rules · draws on 115 · summary Competitions are keyed by what is contested, and each member enters one competition per tick, for its most pressing need. (amended by 240)
238. **At an Eponym death with no bonded companion, what follows is the
     player's call.** Put to Mark on 2026-09-26, from the Eponym overlay
     plan's decision 3: an outsider arriving, the execution plan's F8
     fallback; the player's call, as ruling 229 gives Mesocosm; or the line
     ending, watchable? Mark: "Player's call." So the player takes up
     another life in that world, any of ruling 235's three, or goes to a new
     world. *Reading, not ruled:* going to a new world is how a line ends in
     one, so the plan needs no separate answer that lets it end.

     Links: area Eponym · draws on 229, 235 · summary At an Eponym death with no bonded companion, what follows is the player's call: another life in that world or a new world.
239. **Isocosm absorbs Eponym's simulation, and driving is built only on
     Isocosm.** Put to Mark on 2026-09-26, from the plan's decision 4: with
     Isocosm absorbing `eponym-world`'s and `eponym-social`'s simulation as
     ruling 192 had it absorb `mesocosm-core`, in the plan's order (bodies,
     places, lives and rounds, standing and asks, knowledge, glyphs, then
     fights and technique), is driving built only on Isocosm, as ruling 194
     had Mesocosm's directing, or first over today's `Session`? Mark: "Only
     on Isocosm." So the absorption goes in that order, and driving has no
     prototype over `Session`.

     Links: area Eponym · summary Isocosm absorbs Eponym's simulation in the plan's order, and driving is built only on Isocosm with no prototype over Session.
240. **Competitions run concurrently in a tick and settle at its end.**
     Reopened by Mark on 2026-09-26, after ruling 236: "Although i imagine
     that might slow things down... is it better to allow for concurrent
     competitions, performance-wise?" Answered that one per tick was the
     cheaper of the options first put, and that costs carrying from one
     competition into the next was the slow one, since it splits groups of
     identical members mid-tick and forces the competitions into order; that
     competitions run at once, each against the member's state at the tick's
     start with the costs summed at its end, cost no more; and that one per
     tick costs dynamics rather than compute, a member needing food and a den
     waiting a tick for the second. Put back with that and, at his asking, a
     recommendation, Mark: "Concurrent, settled at end." So a member enters
     every competition it needs in a tick, each resolving against its state
     at the tick's start, and the costs, reserve spent and strain, are summed
     and applied at the tick's end, capped at what it holds. This amends
     236's one per tick; its map keyed by what is contested stands.

     Links: area Processes and rules · amends 236 · summary Competitions run concurrently in a tick against the tick-start state, with costs summed and applied at the tick's end, capped at what a member holds.
241. **Posing is a telling's manner.** Put to Mark on 2026-09-26, from the
     Eponym overlay plan's decision 5: posing to intimidate, persuade or
     deceive, his own idea at ruling 117 and flagged since as a reading, is a
     telling's manner, an act of its own covering wordless display and bluff
     too, or left out of E1? Mark: "A telling's manner." So posing is how a
     claim is told, the fourth thing a hearer weighs (ruling 118), carried
     on the telling and not as an intent of its own; display and bluff in a
     contest stay sizing up (ruling 116).

     Links: area Processes and rules · draws on 117, 118 · summary Posing is a telling's manner, the fourth thing a hearer weighs, carried on the telling and not as an intent of its own.
243. **With the sim on, a battlemap is projected from the generated volume,
     the DM's map an edit over it.** Put to Mark on 2026-09-26, from the VTT
     overlay plan's decision 1: the DM's own map asserted over the site's
     volume (ruling 89), a projection of the generated volume in the game's
     grid (rulings 12, 18), or both, the map an edit over what was generated?
     Mark: "Both: an edit."

     Links: area The VTT · draws on 89, 12, 18 · summary With the sim on, a battlemap is projected from the generated volume and the DM's map is an edit over it.
244. **Moves within a battlemap reach the sim.** Put to Mark on 2026-09-26,
     from the VTT plan's decision 2: only outcomes, assertions and travel,
     a token's position staying the table's, or moves within a battlemap
     too, as per-tick batches, so a character's place in the site follows
     its token? Mark: "Moves too." The contract stays coarse, batches per
     tick and never a call per token (§5.2 point 1).

     Links: area The VTT · summary Moves within a battlemap reach the sim as per-tick batches, never a call per token.
245. **At a VTT table, sharing a character and the DM playing the unclaimed
     are on by default.** Put to Mark on 2026-09-26, from the VTT overlay
     plan's decision 3: two players directing one character (ruling 153) and
     the DM playing any unclaimed one (ruling 156), on by default, or each a
     campaign setting the DM chooses? Mark: "On by default."

     Links: area The VTT · draws on 153, 156 · summary At a VTT table, sharing a character and the DM playing the unclaimed are on by default.
246. **Downtime needs each player's yes.** Put to Mark on 2026-09-26, from
     decision 4: the players' consent to downtime (rulings 63, 104, 105)
     gathered as each player's yes, a majority, or the DM's declaration with
     a veto? Mark: "Each player's yes."

     Links: area The VTT · draws on 63, 104, 105 · summary Downtime needs each player's yes.
247. **The tape-drawn faction turn retires at V2.** Put to Mark on
     2026-09-26, from decision 5: with Isocosm absorbing
     `isometry-campaign`'s world as it did Mesocosm's and Eponym's (rulings
     192, 239), does the faction turn stay as a sim-off campaign's downtime,
     or retire entirely once the sim's factions run? Mark: "Retires at V2."
     So the absorption stands in the plan's §4 order and the faction turn
     retires. *Reading, not ruled:* a sim-off campaign's downtime is then the
     table's own, as the plain tabletop plays today (ruling 188).
     *Carried out 2026-10-06 by ruling 598:* the campaign world moves, its
     `isometry-core` adapters staying with the VTT.

     Links: area The VTT · plans campaign-retirement · summary The tape-drawn faction turn retires at V2 once Isocosm absorbs the campaign world.

248. **A sim-off campaign still writes its facts as notes.** Put to Mark on
     2026-09-26, from decision 7: does a campaign played with the sim off
     still write its facts as notes, so the sim can be switched on later over
     the same history, or does switching it on found a world from that point
     (ruling 126)? Mark: "Writes notes."

     Links: area The VTT · draws on 188 · summary A sim-off campaign still writes its facts as notes, so the sim can be switched on later over the same history.
250. **An uncalibrated campaign warns when it opens and marks its
     receipts.** Put to Mark on 2026-09-26, with 249: what does the table see
     of ruling 189's debug or experimental mode, a warning when the campaign
     opens with every receipt and save marked uncalibrated, or a standing
     banner as well? Mark: "Warn at open, mark receipts."

     Links: area The VTT · draws on 189 · summary An uncalibrated campaign warns when it opens and marks every receipt and save as uncalibrated.
251. **Where an incoming part attaches is its attachment.** Put to Mark on
     2026-09-26, from the tract rename lane (ruling 224), which left alone a
     sense of *site* that is neither the world map's cell nor the tract: the
     point where an incoming part attaches to a body in growth or a graft
     (`growth::attachment()`, "landing site"). Attachment, placement, or
     tract? Mark: "Attachment."

     Links: area Naming · draws on 157 · summary Where an incoming part attaches to a body is its attachment.
252. **An organ's position in a body plan's template is its situs.** Put to
     Mark on 2026-09-26, with 251: an organ's position in a plan's template,
     a (tagma, segment) coordinate, the archetype's "organ sites": slot,
     tract, or named when the bodies family moves in M2? Mark: "Situs.
     Latin, anatomical, no conflict. Good?" Checked the same day: anatomy's
     word for where an organ lies, as in situs solitus, the organs' ordinary
     arrangement; among the anatomical candidates §3.4.1 listed when tract
     was chosen (ruling 157); used nowhere in the code; an in-sim word and
     not a crate, so the tier rule asks no registry check, though crates.io's
     `situs` is free; its other common sense, the legal situs of property,
     lies far from this domain. So *situs* it is, and with 251, *site* keeps
     only the world map's meaning.

     Links: area Naming · draws on 157, 251 · summary An organ's position in a body plan's template is its situs; site keeps only the world map's meaning.
254. **The contract's shared shapes live once, in its core.** Put to Mark on
     2026-09-26, here and in the RPG systems session: E1 and V1 left
     `WorldPoint` in three game modules, the act key in three, and `Harm`,
     `Wound`, `WoundSeverity` and `PartHandle` in two, byte-identical; lift
     them into the crate's shared core, or leave the copies? Mark: "Lift to
     the core now", and in the RPG systems session, "I don't mind lifting
     either." Done there at 269ffc5: `point.rs`, `act_key.rs`, the VTT's
     `ActionKey` renamed, and `harm.rs` are core exports, and no game module
     defines its own.

     Links: area Stack and engine · draws on 197 · summary The contract's shared shapes (WorldPoint, act key, Harm, Wound, WoundSeverity, PartHandle) live once, in the crate's core.
255. **Raw receipts live out of tree; the repository keeps their summaries,
     sources and scripts.** Put to Mark on 2026-09-26, S2's checkpoint 2
     bringing 14 MB of raw JSON against a repository pack of 9 MB: commit
     them as they are, gzipped, or keep the raw files out of tree in
     `Code/testing/isometry/`, committing the summaries and scripts? Mark:
     "Keep raw out of tree." *Reading, not ruled:* the repository keeps a
     manifest naming each raw file's out-of-tree path and hash, so a copy can
     be checked; receipts already committed stay where they are.

     Links: area Method and documents · summary Raw receipts live out of tree in Code/testing/isometry/; the repository keeps their summaries, sources and scripts.
256. **The sim's clock counts a fine unit, and each process runs on its own
     period.** Put to Mark on 2026-09-26, from an outside review's first
     finding: nothing said what a tick is in world time, so ruling 124's
     region, a century in a few minutes, could neither pass nor fail, a tick
     owing 3 s if ticks are years and 8.2 ms if days. A fine unit with a
     period per process, a day, or a world setting? Mark: "A fine unit,
     periods per process." So each process carries its period in world time,
     feeding daily, lineages yearly, polities by season; the due-event queue
     skips idle time; and ruling 124's target reads as a century of world
     time at those periods.

     Links: area Processes and rules · summary The sim's clock counts a fine unit and each process runs on its own period in world time.
257. **The clock's unit is a world setting, a minute by default.** Put to
     Mark on 2026-09-26, with 256: a second, a minute, or a world setting?
     Mark: "World setting, a minute default (a round in d&d and a few other
     systems)". *Reading, not ruled:* anything finer than a world's unit, a
     six-second round in newer tabletop systems or Eponym's real time, is the
     foreground game's to resolve, as rulings 232 and 233 and the VTT's
     handoff already have it.

     Links: area Processes and rules · draws on 256 · summary The clock's unit is a world setting, a minute by default; finer resolution is the foreground game's to resolve.
258. **The scheduler indexes processes by the traits they require now;
     queuing due events per entity is decided before M2's first receipts.**
     Put to Mark on 2026-09-26, from the review's second finding, confirmed
     in the code (`simulation.rs:251-261`) and the receipts: each due process
     was evaluated against every living group, so at 1,024 members the
     evaluations per tick grew from 5,621 with 2 lineages to 219,408 with 32,
     about 15 to 250 ms, 98% of them doing nothing, against the record's "an
     idle thing costs nothing". Fix now with results identical, or during M2?
     Mark: "Fix now, results identical." Asked which part, the trait index
     being byte-identical while per-entity queues reorder acts within a tick
     and change every result: Mark: "Index now, per-entity later."

     Links: area Processes and rules · summary The scheduler indexes processes by the traits they require now, with results identical; per-entity due-event queues are decided before M2's first receipts.
259. **The operation budget counts only the evaluations that run.** Put to
     Mark on 2026-09-26, with 258: the budget counts evaluations, so skipping
     blocked ones changes what a budget allows; count the skipped too, staying
     byte-identical everywhere, or count only what runs, re-baselining
     budgeted sessions? Mark: "Count only what runs." So budgeted sessions take
     a new baseline, and six tight-budget saves that failed to load in the
     other mode are checked again.

     Links: area Processes and rules · draws on 258 · summary The operation budget counts only the evaluations that run; budgeted sessions take a new baseline.
260. **Viability gates are staged with the absorption, family by family.**
     Put to Mark on 2026-09-26, from the review's fourth finding, that no
     phase gates on behaviour while no core the wing has run keeps an ecology
     alive: one gate before any absorption, or keep the ruled order? Mark
     asked: "Conceptually, is the ecology able to be balanced before the
     things to balance are in play? I guess the first balance is between
     producers and consumers..." Answered that today's core holds only a toy
     web, so one gate before absorption would certify the toy, and that the
     balance can be staged with the families that bring its mechanics:
     producers alone, then with consumers, then with decomposers closing the
     loop. Put back, Mark: "Staged, per family." So each of M2's families is
     done only when the level of the web it enables stays alive in most
     draws.

     Links: area Method and documents · summary Viability gates are staged with the absorption, family by family: producers, then consumers, then decomposers.
261. **The background is hybrid: derived where effects commute, a calibrated
     rate model where they do not.** Put to Mark on 2026-09-26, from the
     review's third finding, that exact grouping folds only identical members
     and the crowd's savings fell from 11 to 2.4 times once strain was
     per-member state: what produces the background, ruling 113 staying the
     check? Mark: "Hybrid." So the background is derived from the definition
     where effects commute, by lossless grouping, and where they share
     resources, targets or relationships it is a rate model per site and
     lineage, calibrated against the exact runner on the bench.

     Links: area Processes and rules · draws on 113 · summary The background is hybrid: derived where effects commute, a calibrated rate model per site and lineage where they share resources, targets or relationships. (amended by 266)
265. **The thirty shapes are expressed as data with the used mechanics, and
     the definition widens with the fields the probe proves.** Put to Mark on
     2026-09-26, with 262: the shapes as data (X1), those plus the extensions
     the five used mechanics need (X2 to X6), or wider still with the trait
     catalogue's fields? Mark asked for the third's case: "what's the
     argument for option 3? any drawbacks? otherwise, that's how I lean".
     Answered that widening makes traits differ in what they do rather than
     in name and lets packs define real processes as data, at the cost of
     every new field entering every world's digest, design surface ahead of
     the probe's evidence, and an overlap with composing tracts on a body;
     and offered widening with the fields the probe proves. Mark: "Wider,
     fields from the probe."

     Links: area Processes and rules · summary The thirty shapes are expressed as data with the used mechanics, and the process definition widens with the fields the probe proves.
266. **Rate models take over where grouping stops paying.** Put to Mark on
     2026-09-26, from a second review, which found that the certified crowd
     handled a shared resource's competition exactly in distribution: move
     ruling 261's line from "shares resources, targets or relationships"?
     Mark: "Where grouping stops paying." So the exact crowd runs wherever it
     still finds identical members, shared-resource competition included, as
     certified, and calibrated rate models take the ground where diverse
     bodies and relationships defeat grouping. This amends 261.

     Links: area Processes and rules · amends 261 · summary Rate models take over where grouping stops paying; the exact crowd runs wherever it still finds identical members, shared-resource competition included.
267. **A level of the web stays alive by four measures.** Put to Mark on
     2026-09-26, from the second review's caution that births balancing deaths
     could certify a lifeless world: ruling 260's "stays alive in most draws",
     by births balancing deaths, or by persistence, turnover, collapse and the
     web's response to an intervention? Mark: "The four measures." Each has a
     bound in the world's rules.

     Links: area Ecology · draws on 260 · summary A level of the web stays alive by four measures (persistence, turnover, collapse, response to intervention), each bounded in the world's rules.
268. **Computed amounts are bounded expression trees, with a default set
     easy to group and calibrate.** Put to Mark on 2026-09-26, from S2's
     checkpoint 3 plan, decision 5: linear in one reading, clamped, with a
     draw term; a bounded expression tree; or named native formulas? Mark:
     "2, but with a default set of expressions that are easy to group and to
     calibrate?" So a world may author a bounded expression tree, and the
     default expressions are ones the crowd groups and the bench calibrates
     easily. *Reading, not ruled:* the linear, clamped form with a draw term
     is among the defaults.

     Links: area Processes and rules · summary Computed amounts are bounded expression trees, with a default set of expressions easy to group and calibrate.
269. **Shared ground is scrambled for by what cannot decide and competed for
     by what can.** Put to Mark on 2026-09-26, from decision 6, ruling 266
     keeping the exact crowd while grouping pays: a shared site resource
     under shortage resolved by a competition, a proportional scramble, or
     one by one in identity order? Mark: "a proportional scramble if these
     creatures are not sentient, a competition if they are. like if they're
     dumb they're just gonna try to take what they can; if they've got some
     decision making capability, that resolves with a competition!"
     *Reading, not ruled:* decision-making capability is a methodology that
     chooses (ruling 37), so producers and other purely reactive members
     scramble in proportion, and members that choose compete under rulings
     115, 236 and 240. *Specified 2026-10-01 by ruling 454:* a pass reads
     the world as it began, and a short site or prey gives each taker the
     same fraction of its take, floored, the remainder staying put.

     Links: area Processes and rules · draws on 266, 115 · summary Shared ground is scrambled for proportionally by members that cannot decide and competed for by those that can.
270. **The flow record is buffered outside state.** Put to Mark on
     2026-09-26, from decision 7: each tick's matter receipts, reconciling
     every compartment, buffered outside the state hash, kept in state under
     the history cap, or no stream? Mark: "Buffered outside state."

     Links: area The sim's schema and saves · summary The flow record (per-tick matter receipts) is buffered outside the state hash.
271. **Matter a dev places comes from a dev source.** Put to Mark on
     2026-09-26, from decision 8: a dev intent placing matter breaks the
     conservation invariant; a dev source outside the conserved total,
     creative mode only, or refused? Mark: "A dev source." So it enters from
     outside the conserved total, as Mesocosm's DT3 does, and the run is
     labelled assisted.

     Links: area Processes and rules · summary Matter a dev places comes from a dev source outside the conserved total, creative mode only, and the run is labelled assisted.
272. **Pressure profiles become founding presets.** Put to Mark on
     2026-09-26, from decision 9: `mesocosm-core`'s pressure profiles, which
     nothing in the sim reads: founding presets, authoring reference, or
     retired? Mark: "Founding presets."

     Links: area World generation · summary mesocosm-core's pressure profiles become founding presets.
273. **Checkpoint 3's four readings stand.** Put to Mark on 2026-09-26: R1,
     untyped matter becomes the world's own, living matter accounts typed by
     lineage, digestion yielding the eater's matter; R2, a shape's identity is
     its set of roles; R3, a native reading of a member's own state runs in
     the crowd per exact state, so only opaque run-time scripts stay
     foreground-only; R4, until bodies move, seeding's only consumer is the
     founding generator. Mark: "Accept all four."

     Links: area Bodies and anatomy · summary Checkpoint 3's four readings stand: untyped matter becomes the world's own, living matter is lineage-typed, shape identity is its roles, native readings run in crowds.
274. **At a merge that changes lived outcomes, the player chooses.** Put to
     Mark on 2026-09-26, from both reviews: a merge by replay can change
     outcomes a player already lived through; show and choose, lived history
     holds, or the replay wins, remembered? Mark: "show and choose: keep your
     own fork, the replay adjusts around lived outcomes (merge), or the
     replay wins and the lived outcome is a tale. how about that?" Answered
     that the three are ones the sim already has, a fork (ruling 126), lived
     outcomes asserted so the replay works around them, and a tale (ruling
     157), and that two players' lived histories can collide. *Reading, not
     ruled:* when they do, each keeps their own fork by default.

     Links: area Record and history · draws on 126, 157 · summary When a merge changes lived outcomes the player chooses: keep their fork, let the replay adjust around them, or let it win, making them tales.
276. **Shapes are eight: lump, rod, sheet, point, tube, branch, shell and
     joint.** Put to Mark on 2026-09-26, from the survey of both axes that
     ruling 264 asked for: keep the four read from geometry, add joint and
     tube, or add tube, branch, shell and joint? Mark: "Add tube, branch,
     shell, joint." So the classifier reads hollowness, branching and
     enclosure as well as extents.
     *Amended 2026-10-02 by ruling 492:* shapes are read from a part's
     measurements to name it, and gate no function; no classifier reading
     hollowness, branching or enclosure had been built.
     *Read 2026-10-02 by ruling 494:* lump, rod, sheet and point from the
     box, branch and joint from the tree, tube and shell declared from the
     body's voxels.

     Links: area Bodies and anatomy · draws on 264 · summary Shapes are eight: lump, rod, sheet, point, tube, branch, shell and joint. (amended by 492)
277. **A body's functions assemble into organ systems, and new systems are
     riffed from them.** Put to Mark on 2026-09-26, with 276: which functions
     join contract, intake, sense, fix and secrete: structure and control,
     metabolism, life cycle, handling? Mark picked all four and said:
     "honestly i was thinking of the main organ systems of the body, like the
     integumentary (is that right?), respiratory, circulatory, digestive,
     neurological, reproductive, the kinds of sensation, things
     humans/fauna don't have, etc.. if we can assemble those buckets out of
     functions like the ones you propose, i'm alright with that, i guess i
     just figured take the big approach and then try figuring out parts and
     then riff new ones and mix existing ones to make new forms/systems...
     might make sense for new-looking forms of life that still are
     biologically reasonable. i don't have great specifics though." So the
     model has three levels. Organ systems: integumentary, which is right,
     the body's covering; respiratory, circulatory, digestive, nervous,
     reproductive; the senses; and systems fauna lack. Functions assemble
     them: the five, plus support, conduct, gate, store, circulate, respire,
     excrete, reproduce, grip and adhesion, each carried by the shapes it
     names. Parts are shapes expressing functions. The generator riffs new
     systems by mixing functions, for new-looking life that stays
     biologically reasonable.
     *Specified 2026-10-02 by rulings 465 and 466:* a system is a named
     network of functions in the world's rules, and each function's shapes
     and seeding are tabled.
     *Tabled 2026-10-02 by rulings 489 and 491:* ten default systems, and
     new ones riffed by substitution.

     Links: area Bodies and anatomy · draws on 276 · summary A body's functions assemble into organ systems, parts are shapes expressing functions, and new systems are riffed by mixing functions.
278. **The function vocabularies become one catalogue in the sim.** Put to
     Mark on 2026-09-26, from the survey, which found four that never meet,
     Mesocosm's processes, `wing-functions`' kinds, Eponym's grip and
     adhesion, and its contact probe's impairments: one catalogue every game
     reads, or separate for now? Mark: "One catalogue."

     Links: area Bodies and anatomy · summary The function vocabularies become one catalogue in the sim that every game reads.
283. **The first disturbances are disease, an overperformer and a keystone
     lost; catastrophe follows.** Put to Mark on 2026-09-26: which
     disturbances the sim models first, serving both as the viability gate's
     interventions (ruling 267) and, at scale, as ruling 226's world threats:
     disease, catastrophe, an overperformer, a keystone lost? Mark: "Disease,
     An overperformer, A keystone lost, Catastrophe later, to test the
     criteria of systemic resilience". So a pathogen spreading by contact and
     density, one lineage outcompeting the web, and a top consumer or key
     producer removed come first, and meteor, drought, flood and fire follow
     once the gate's criteria stand, to test the web's resilience as a whole.

     Links: area Ecology · draws on 267, 226 · summary The first disturbances modelled are disease, an overperformer and a keystone lost; catastrophe follows once the gate's criteria stand.
284. **An advance's limit guards work, not ticks.** Put to Mark on
     2026-09-26, from the scheduler index's report: at a minute's unit a
     century is 52.6 million ticks against a default advance limit of
     100,000, while the due-event queue skips idle time; guard work, raise the
     tick limit, or run many advances? Mark: "Work, not ticks." So the limit
     guards evaluations, and an advance may span any stretch of idle time.

     Links: area Processes and rules · draws on 256 · summary An advance's limit guards work (evaluations), not ticks, so an advance may span any stretch of idle time.
285. **The budget counts the members an evaluation stands for.** Put to Mark
     on 2026-09-26, with 284: individual mode evaluates a cohort member by
     member, so a history that fits a budget grouped can exceed it
     individually, as seven saves showed; count members, or let a budget
     belong to its mode? Mark: "Count members." So a budget means the same in
     both modes.

     Links: area Processes and rules · draws on 284 · summary The budget counts the members an evaluation stands for, so a budget means the same in grouped and individual modes.
286. **Due events are kept per group.** Put to Mark on 2026-09-26, with 284:
     84 to 96% of the remaining evaluations did nothing, death and age
     processes visiting every member each period to be blocked by thresholds,
     and ruling 258 had left per-entity queues for before M2's first
     receipts: due events per group, per entity, or not yet? Mark: "Due
     events per group." So a group knows when its next threshold falls due,
     an age at death or a reserve running out, and is scheduled for then;
     grouping is kept, and the changed results are certified again.

     Links: area Processes and rules · draws on 258 · summary Due events are kept per group; a group knows when its next threshold falls due and is scheduled then.
287. **Each consumer lineage has one feeding process, choosing among its
     prey.** Put to Mark on 2026-09-26, with 284: the ecology generator made
     one feeding process per pair of predator and prey lineages, so feeding
     multiplied with lineages; one per consumer, or keep the pairs? Mark:
     "One per consumer." Its choice among prey is the definition's target
     selection.

     Links: area Processes and rules · summary Each consumer lineage has one feeding process, choosing among its prey, rather than one per predator-prey pair.
289. **mere's atlas is sized to the card.** Put to Mark on 2026-09-26, with
     288: the resident cap of 2,047 bricks does not cover a 256-tile board
     with relief in the headed pane; a card-sized cap, or 2,047 for now?
     Mark: "Card-sized cap." It rides with the defect's fix.

     Links: area Stack and engine · summary mere's atlas is sized to the card, riding with the retarget defect's fix.
290. **The board's CPU side is paged too.** Put to Mark on 2026-09-26, with
     288: every overlay change regrows the whole map, seconds a click at 256;
     page it too, computing bricks from the map on demand, or atlas only?
     Mark: "Page it too."

     Links: area Stack and engine · summary The board's CPU side is paged too, computing bricks from the map on demand.
291. **Residency follows the view, in an isometer helper.** Put to Mark on
     2026-09-26, with 288: which bricks are loaded, and where the code lives?
     Mark: "View, isometer helper." So the bricks overlapping the pane plus a
     one-brick margin, a pure function of view and map; the mechanics in a
     product-neutral helper in `shared/isometer`, the board's rule in the VTT.

     Links: area Stack and engine · summary Residency follows the view (pane bricks plus one-brick margin), with mechanics in a product-neutral helper in shared/isometer and the board's rule in the VTT. (amended by 425)
293. **Per-group phases run in the foreground; the background keeps a shared
     grid.** Put to Mark on 2026-09-26, from ruling 286 as built, which left
     results identical by visiting a group only once it is ready, while each
     process kept its period grid: that, or per-group phases, each group on
     its own clock? Mark: "Per group phases if they are in the foreground or
     nearby, baseline as built?" Answered yes: members in the foreground and
     nearby run as individuals anyway (rulings 113, 212), so phases there cost
     no grouping. They come with the sim's attention sets.

     Links: area Processes and rules · draws on 286 · summary Per-group phases run in the foreground and nearby; the background keeps a shared period grid as built.
294. **A consumer eats across its prey's matter, once a period, choosing its
     prey by a weighted draw.** Put to Mark on 2026-09-26, from ruling 287's
     fork, one feeding process being unable to name one account to take
     from: what is eaten, how often, and how prey is chosen? Mark: "Drawn
     across its matter", "Once per period", "Weighted draw". So up to n of
     the chosen prey's matter is drawn proportionally from all its accounts
     and credited to the eater (X4 with R1's digestion); a consumer feeds
     once a period, the certification showing the new balance; and its prey
     is a seeded draw weighted by what each eligible prey holds.

     Links: area Processes and rules · draws on 287 · summary A consumer eats across its prey's matter once a period, choosing prey by a draw weighted by holdings.
295. **mere's atlas is sized by limits the host fills in.** Put to Mark on
     2026-09-26, from the mere lane's assessment for ruling 289: a plain-data
     `AtlasLimits`, the 3D texture dimension and an atlas byte budget, filled
     from the device's enforced limits, so `modulus` stays GPU-free; a brick
     count rounded up to whole rows; the 16 by 16 grid kept, rows growing;
     `MAX_BRICKS` kept as the default's cap of 2,047 so existing callers are
     unchanged; no limits-aware `from_keys` yet. Mark: "Accept the shape." The
     retarget fix lands beside it, with `modulus`'s tests moved out of its
     777-line `lib.rs` into a sibling file ("Keep it").

     Links: area Stack and engine · amends 289 · summary mere's atlas is sized by plain-data AtlasLimits that the host fills in from device limits; MAX_BRICKS stays the default cap.
296. **The host's atlas budget defaults to 8 MiB.** Put to Mark on
     2026-09-26, with 295, video memory being unreportable in wgpu: a
     default budget of 8 MiB, 32 MiB, or as the texture allows? Mark: "8
     MiB." So a host asks for up to 16,383 bricks by default, enough for a
     1920 by 1080 pane over the 256-tile relief board, 6,391 with its margin.

     Links: area Presentation · summary The host's atlas budget defaults to 8 MiB.
300. **The VTT's crates keep their prefix.** Put to Mark on 2026-09-26, after
     he called `isometry-genet` a stale name: rename the VTT's crates after
     the bump, during it, or keep ruling 111's `isometry-` prefix? Mark: "Eh.
     If they're within the ambit of isocosm vtt, then fair enough." So ruling
     111 stands.

     Links: area Naming · draws on 111 · summary The VTT's crates keep their isometry- prefix; ruling 111 stands.
302. **The pointer volume reserves headroom above the board's tallest
     point.** Put to Mark on 2026-09-26, with 301: the volume's height
     follows the terrain, so an edit that lifts the tallest point into a new
     brick layer rebuilds the map whole, a 1 MiB upload; accept the rare
     rebuild, or size the volume with a spare layer or two? Mark: "Reserve
     headroom." Only an edit past the headroom rebuilds. *Reading, not
     ruled:* the spare count is a setting, one layer by default until its
     memory cost comes back to Mark.

     Links: area Stack and engine · summary The pointer volume reserves headroom above the board's tallest point, one spare layer by default.
307. **The draw control stays in the check.** Put to Mark on 2026-09-26,
     with 304: Lane A added the crowd drawing prey by headcount as a control
     arm of its own accord; keep it or keep the check to the arms the
     rulings named? Mark: "Keep it." It is the control that shows whether
     the check sees prey choice at all.

     Links: area Processes and rules · summary The draw control (a crowd drawing prey by headcount) stays in the check as the arm showing whether the check sees prey choice.
317. **A refused draw is recorded and left out, and the check fails past a
     1% bound.** Put to Mark on 2026-09-26, with 316: when the crowd cannot
     group a hunt, prey running out part way through hunters in different
     states, it refuses the draw; checkpoint 4 aborted the check on one
     refusal, and 4b records it and leaves the draw out of that arm's
     comparisons, which could let a crowd that refuses often still pass;
     record with a bound, record and leave out, or abort? Mark: "Record,
     with a bound." The check fails if an arm under test refuses more than
     1% of its draws. *Reading, not ruled:* the arms under test are the
     crowds being certified or qualified, the approximate crowd included,
     and the controls record their refusals unbounded.

     Links: area Processes and rules · draws on 209 · summary A refused draw is recorded and left out of comparisons, and the check fails if an arm under test refuses more than 1% of draws.
320. **The sim core keeps an observation hook.** Put to Mark on 2026-09-26,
     from checkpoint 4b: to read prey choice in the exact runner, Lane A
     added `Simulation::watch(process)`, which keeps each accepted act of a
     process with the matter its target held as the act began until
     `take_watched()`, changes nothing the world does, and drops the watched
     acts of a refused advance; keep the hook, carry the holdings in the
     act's event, or snapshot in the probe? Mark: "Keep the hook."

     Links: area The sim's schema and saves · summary The sim core keeps its observation hook, Simulation::watch.
323. **With Cleromancy off, `>choose` seeds its draw from the command.** Put
     to Mark on 2026-09-26, with 321: the VTT's own seeded draw of ruling
     298 takes the command's seed and domain, so a command always makes the
     same choice, or the host's generation tape? Mark: "The command's seed."
     So the choice replays from the command alone, as Cleromancy's derived
     selection does.

     Links: area The VTT · summary With Cleromancy off, the VTT's >choose seeds its draw from the command's seed and domain, so the choice replays from the command alone.
330. **nisus grows into the voxel authority.** Put to Mark on 2026-09-26,
     from Lane J's assessment under ruling 326: two voxel stores exist,
     mere's nisus, with revisioned chunks, a patch that refuses stale
     revisions, dirty regions kept out of saves and the same 8³ layout, and
     isometer-core's `Ground`, which the renderer reads, with one world
     revision, a dirty queue only one reader can drain and no public
     additive write; the placement table gives revisioned voxel chunks and
     edits to nisus, and §4.3 calls the duplication a don't. nisus,
     `Ground`, or a trait deciding later? Mark: "nisus grows into it." nisus
     becomes the world store, with a chunk map, a world revision, a revision
     log and additive writes, and `Ground` becomes a thin layer over it or
     retires; the save hashes that encode `Ground`'s bytes migrate or keep
     its wire format.

     Links: area Stack and engine · summary nisus grows into the voxel authority and world store; Ground becomes a thin layer over it or retires.
331. **One edit reaches its consumers through a revision log.** Put to Mark
     on 2026-09-26, with 330: a log of changes keyed by revision at 8³-brick
     grain, each consumer reading what changed since its own last read and
     rebuilding when too far behind; per-consumer queues; or a product-side
     conductor? Mark: "A revision log."

     Links: area Stack and engine · draws on 330 · summary One edit reaches its consumers through a revision log of changes keyed by revision at 8-cubed brick grain.
332. **conatus carries the source's stamp; the barrier stays with the
     product.** Put to Mark on 2026-09-26, with 330: conatus stamps colliders
     with its own counter, which cannot be compared with the source's, and
     T2 wants a barrier holding dependent simulation until the colliders
     catch up; conatus carries the source stamp with the barrier in the
     product, products track it, or conatus holds the barrier? Mark:
     "conatus carries the source stamp." Its voxel edits take an optional
     source stamp that queries report, and the barrier lives in the product
     until a second consumer proves it.

     Links: area Stack and engine · draws on 330 · summary conatus carries the source's stamp via an optional stamp on voxel edits; the barrier stays in the product until a second consumer proves it.
333. **Navigation stays a search per query, each result stamped.** Put to
     Mark on 2026-09-26, with 330: no product caches navigation, each
     searching the current terrain per query, never stale but never saying
     which revision it read; stamp each query, a cached walkability product,
     or wait for the place graph? Mark: "Stamp each query." A cached product
     comes only when a measured query cost forces it.

     Links: area Stack and engine · draws on 330 · summary Navigation stays a search per query with each result stamped by revision; a cached product comes only when measured cost forces it.
338. **A process's shape requirement and seeding live on the function
     catalogue.** Put to Mark on 2026-09-26, from Lane A's Part B, whose
     step 1 was planned before rulings 276 to 278 put shapes and functions
     on parts under one catalogue: on the catalogue, on the process
     definition, or both? Mark: "On the function catalogue." Each function
     lists the shapes that admit it and its seeding, parts express
     functions, and a process requires a live part expressing a function,
     bound to the lowest-numbered one that qualifies. The thirty shapes of
     the old plan become every non-empty set of the eight shapes times
     seeding, 510 in all, and step 1 covers them.
     *Extended 2026-10-02 by ruling 480:* an act reads and writes the site
     of the part it binds, and a spread acts from each patch, which one
     lowest-numbered binding does not cover; the anatomy brief's next round
     settles how.
     *Settled 2026-10-02 by ruling 488:* a spread is a territory, acting at
     each place it covers, so the one binding serves a part tree. *Amended
     by ruling 492:* no shape gates a function; the catalogue's shapes are
     each function's fits.

     Links: area Bodies and anatomy · draws on 276, 277, 278 · summary A process's shape requirement and seeding live on the function catalogue; a process requires a live part expressing a function, bound to the lowest-numbered qualifier. (amended by 480, 492)
339. **The catalogue starts with the five functions in use.** Put to Mark on
     2026-09-26, with 338: the five, all fifteen of ruling 277, or those
     with the other vocabularies folded in? Mark: "The five in use."
     Contract, intake, sense, fix and secrete, rods contracting, lumps
     taking in, points sensing, sheets fixing, and sheets secreting when
     acquired; the keys stay open, the rest arriving with the anatomy brief
     (281) and the probe's evidence. *Widened 2026-10-02 by ruling 461:*
     all fifteen of 277, the other vocabularies folded in.

     Links: area Bodies and anatomy · draws on 338 · summary The function catalogue starts with the five functions in use: contract, intake, sense, fix, secrete; keys stay open. (amended by 461)
340. **The process's causal kind gives up the name `Shape`.** Put to Mark on
     2026-09-26, with 338: `isocosm::rules::Shape`, a process's causal kind
     (choice, agentless, transition), collides with ruling 276's part
     shapes; rename the process one, or qualify the part noun? Mark: "Rename
     the process one." *Reading, not ruled:* it becomes `Causation`, the
     name offered, its saved field keeping the name `shape` so old worlds
     load.

     Reading confirmed 2026-10-21 (doctrine Naming 4): Mark, "Confirm the readings".

     Links: area Naming · draws on 276, 338 · summary The process's causal kind gives up the name Shape and is renamed (offered: Causation); its saved field keeps the name shape.
341. **Seeding's two values are Grown and Acquired.** Put to Mark on
     2026-09-26, with 338: Grown and Acquired, or Mesocosm's Geometry and
     Acquired? Mark: "Grown and Acquired." *Reading, not ruled:* the names
     put with it stand, `Binding::Part`, the catalogue `functions` in the
     world's rules, and a part's `shape` and `functions`.

     Reading confirmed 2026-10-21 (doctrine Naming 4): Mark, "Confirm the readings".

     Links: area Naming · draws on 338 · summary Seeding's two values are Grown and Acquired.
342. **Declared conversions are checked.** Put to Mark on 2026-09-26, with
     338: the sim admits any mass-balanced transform while Mesocosm admits
     three, synthesis, digestion into the eater's own matter, and
     mineralization; check declared conversions, record the kind and refuse
     nothing, or refuse all else? Mark: "Check declared ones." A transform
     declaring a conversion kind is checked against the three, and the rest
     pass as before, the certified probe's grazing and drinking included.

     Links: area Processes and rules · summary Declared conversions are checked: a transform declaring a conversion kind is checked against synthesis, digestion and mineralization; the rest pass as before.
344. **The dev source's issue and the assisted label live in the history.**
     Put to Mark on 2026-09-26, with ruling 271: in the history, in the
     world's state, or both? Mark: "In the history." The command in the log
     is the record, the world hashes as the matter it holds, as Mesocosm's
     DT3 does, and receipts show the issued amount beside the conserved
     total.

     Links: area Processes and rules · draws on 271 · summary The dev source's issue and assisted label live in the history; the logged command is the record; receipts show issued amount beside the conserved total.
345. **The flow record holds every move, when asked.** Put to Mark on
     2026-09-26, with ruling 270: every move switched on when wanted, every
     move always on, or a net change per tick? Mark: "Every move, when
     asked." Each accepted act's matter moves, from, to, amount and the
     members it stands for, handed over each tick, every ledger reconciling
     from it in both runners.

     Links: area The sim's schema and saves · draws on 270 · summary The flow record holds every move, when asked, handed over each tick, with every ledger reconciling from it in both runners. (amended by 359)
347. **isometer alone answers a pointer pick.** Put to Mark on 2026-09-26,
     with 346: isometer only, conatus through the bindings as T1 was ruled,
     or both? Mark: "isometer only." The player clicks what was drawn, and
     isometer answers per part and per frame, as the VTT board already does;
     the bindings answer other queries.

     Links: area Stack and engine · summary isometer alone answers a pointer pick, per part and per frame; bindings answer other queries.
348. **The binding module, when built, lives in mere's conatus, generic over
     the key.** Put to Mark on 2026-09-26, with 346: conatus, a new crate
     under `shared/`, isometer, or `mesocosm-runtime`? Mark: "A module in
     mere's conatus." Three mere doc passages are amended with the shape,
     one of them the line placing bindings "not to Conatus".

     Links: area Stack and engine · summary The binding module, when built, lives in mere's conatus, generic over the key.
349. **It binds bodies only, on a world shared with T2's terrain collider.**
     Put to Mark on 2026-09-26, with 346: bodies on a borrowed world, bodies
     on its own world, or bodies and terrain? Mark: "Bodies only, shared
     world." Terrain arrives through T2's one path (rulings 330 to 335).

     Links: area Stack and engine · draws on 348, 330 · summary The binding module binds bodies only, on a world shared with T2's terrain collider.
350. **Accepted state arrives both whole and per key.** Put to Mark on
     2026-09-26, with 346: a whole-set reconcile and per-key set and remove,
     reconcile only, or per key only? Mark: "Both." The reconcile is the cold
     rebuild and carries the VTT's per-event accepted map; per-key updates
     carry a critter changing alone; cold and incremental must agree.

     Links: area Stack and engine · draws on 348 · summary Accepted state arrives both as a whole-set reconcile and per key; cold and incremental must agree.
351. **A moved body keeps its id.** Put to Mark on 2026-09-26, with 346: move
     it and keep its id, respawning only when the entity's shape revision
     changes, or always respawn? Mark: "Move it, keep its id."

     Links: area Stack and engine · draws on 348 · summary A moved body keeps its id; it respawns only when the entity's shape revision changes.
352. **conatus gains a query-refresh call.** Put to Mark on 2026-09-26, with
     346: conatus's queries miss a topology change until the world steps, so
     `TactileWorld` and Eponym each run a tiny step; a refresh call in
     conatus, the tiny step kept, or left to the world's owner? Mark: "A
     refresh call in conatus." It is a small mere edit, independent of the
     bindings, that both workarounds become.

     Links: area Stack and engine · summary conatus gains a query-refresh call, replacing the workaround tiny step.
357. **A declared synthesis produces the actor's own lineage's matter.** Put
     to Mark on 2026-09-27, from Part B's step 2 under ruling 342: the sim's
     core has no notion of a producer; the actor's own lineage, a flora
     kingdom, or kingdoms the rules declare? Mark: "The actor's own
     lineage." Whatever has a synthesis process is a producer, mirroring
     digestion, and no producer classification enters the core.

     Links: area Processes and rules · draws on 342 · summary A declared synthesis produces the actor's own lineage's matter; whatever has a synthesis process is a producer, with no producer classification in the core.
358. **A dev placement may fill any declared matter account.** Put to Mark
     on 2026-09-27, with 357: any declared matter account, or world matter
     only? Mark: "Any declared matter account." Mesocosm's overlay chooses
     soil.

     Links: area Processes and rules · draws on 271, 357 · summary A dev placement may fill any declared matter account; Mesocosm's overlay chooses soil.
359. **Each flow names the process that moved it.** Put to Mark on
     2026-09-27, with 357: add the act's process as the reason, or leave it
     out as ruling 345 had it? Mark: "Add the process."

     Links: area The sim's schema and saves · amends 345 · summary Each flow names the process that moved it.
360. **Part shapes take their own key prefix.** Put to Mark on 2026-09-27,
     with 357: X1 keyed part shapes as `shape:*`, the namespace the world's
     own `shape:graph` uses; separate them or share it? Mark: "Part shapes
     get their own prefix." *Reading, not ruled:* `part-shape:*`, the
     example offered.

     Links: area Bodies and anatomy · summary Part shapes take their own key prefix (part-shape:*), separate from the world's shape:graph namespace.
361. **isometer calls modulus's CPU walk instead of copying it.** Put to
     Mark on 2026-09-27, from Lane K's traversal fix under ruling 336, each
     crossing now computed as the boundary less the eye over the direction,
     which removed every fault and every headroom move on the VTT board
     frame, on the CPU and four GPU setups, for 0.02 to 0.08 ms a frame:
     isometer's GPU path includes modulus's shader, but its CPU pick path is
     a hand copy; call modulus's walk, or keep the copy? Mark: "Call
     modulus's walk." modulus makes its exact CPU mirror public and isometer
     wraps it.

     Links: area Stack and engine · summary isometer calls modulus's public exact CPU walk instead of keeping a hand copy.
362. **The traversal's start clamps into the box.** Put to Mark on
     2026-09-27, with 361: the shader's 1e-4 start offset is too small in
     f32 beyond t of about 1,000, so a first-voxel hit reports its entry face
     and the GPU and CPU disagree which, on 3,754 to 4,093 of 98,304 stress
     rays and none on the board; clamp the start voxel into the box, report
     the entry face, or leave it? Mark: "Adopt the clamp." It lands with the
     traversal fix.

     Links: area Stack and engine · draws on 361 · summary The traversal's start voxel clamps into the box and reports the entry face, landing with the traversal fix.
364. **X1's defaults are copied in explicitly.** Put to Mark on 2026-09-27,
     from Part B's step 1: the eight shapes and five functions are constants
     a world copies in, the generator not yet copying them, and the crowd
     refuses a process binding a part until the probe certifies it; keep
     that, or have the generator copy them now? Mark: "Keep as built." Were
     the defaults implicit, growing the set later (281) would change what old
     worlds mean without changing their digest.

     Links: area The sim's schema and saves · summary X1's defaults (eight shapes, five functions) are constants a world copies in explicitly, so growing the set later does not silently change old worlds.

366. **The three method refinements are accepted.** On 2026-09-27 the
     assistant offered three additions: distinguish measured evidence from
     lane reports and unknowns; order questions by what their answers
     unblock; name each control's detectable fault and retain passing and
     deliberately failing results. These were offered together, not as
     mutually exclusive options. Mark: "Sounds good! Shall we proceed? Or
     would you like to review/audit first?" The additions supplement the
     preserved method in session notes §8.2. *Reading, not ruled:* the
     assistant chose a narrow traversal audit before the already-authorized
     repin. This answer does not identify ruling 365's coordination target.

     Links: area Method and documents · summary Three method refinements accepted: separate measured evidence from lane reports, order questions by what they unblock, and name each control's detectable fault, keeping failing results.

368. **Centre-first overflow remains, with the current omitted count.**
     Asked on 2026-09-27: "1. When visible terrain exceeds the atlas budget,
     what should happen? The card-sized atlas holds 16,383 bricks; the
     measured large-board views need 2,413–2,504. Ruling 301 made centre-first
     dropping temporary, so keeping it for future overflow needs an
     amendment." Options, recommended first: (A) "Keep the nearest terrain
     and report the current number of omitted bricks, amending 301
     (recommended)." (B) "Refuse the frame with a capacity error rather than
     omit terrain." Mark selected verbatim: "Keep the nearest terrain and
     report the current number of omitted bricks, amending 301
     (recommended)." This amends 301's expiry at card sizing. The count must
     describe the current frame even when the retained keys do not change.
     *Reading, not ruled:* "nearest" retains the existing centre-first
     ordering identified in the question; it does not replace it with a
     different camera-distance rule. The review's stale-count case needs
     a regression test with unchanged keys and changing omitted counts.

     Links: area Presentation · summary When visible terrain exceeds the atlas budget, keep the nearest terrain and report the current number of omitted bricks.

369. **The host exposes a device-bounded atlas budget setting.** Asked on
     2026-09-27: "2. Should the player be able to change the terrain atlas
     budget? The host currently fixes it at 8 MiB, although the library
     accepts other budgets. A live setting also needs a rebuild that cannot
     be skipped as an unchanged frame." Options, recommended first: (A)
     "Expose a setting, default 8 MiB, bounded by device limits; changing it
     rebuilds the atlas (recommended)." (B) "Keep the host fixed at 8 MiB for
     this slice; only library callers can choose another budget." Mark
     selected verbatim: "Expose a setting, default 8 MiB, bounded by device
     limits; changing it rebuilds the atlas (recommended)." The player can
     change the budget and the next frame must reflect it. Allocation
     upfront versus growth remains a separate unanswered question.
     *Reading, not ruled:* use the existing host preference machinery and
     test one rebuild after a settings change, then an unchanged-frame skip.

     Links: area Presentation · summary The host exposes a terrain atlas budget setting, default 8 MiB, bounded by device limits, with changes rebuilding the atlas. (amended by 372)

371. **Checkpoint 5 requires a per-tick flow handoff API.** Asked on
     2026-09-27: "4. How should requested sim flow records be handed over?
     Checkpoint 5 passes 108 tests and keeps every requested move until the
     caller drains the queue, without a cap. Your ruling 345 says ‘Every
     move, when asked’; handing records over each tick was the plan's
     reading." Options, recommended first: (A) "Keep records until drained;
     live hosts drain each tick, with no silent dropping (recommended)."
     (B) "Require a per-tick handoff API before accepting checkpoint 5."
     Mark selected verbatim: "Require a per-tick handoff API before accepting
     checkpoint 5." The per-tick handoff is now required, rather than an
     assumed host convention. Recording remains opt-in under 345 and every
     requested move must be represented. Checkpoint 5's previously audited
     implementation does not satisfy this new acceptance condition yet.
     *Reading, not ruled:* the lane chooses a minimal explicit API and proves
     that successive tick results cannot accumulate or mix records, in both
     execution modes, with ledger reconciliation retained.

     Links: area The sim's schema and saves · draws on 345 · summary Checkpoint 5 requires a per-tick flow handoff API so successive tick results cannot accumulate or mix records.

372. **The atlas budget is saved locally per device.** Asked on 2026-09-27:
     "Should the atlas budget survive restarting the app? The existing
     pixel-grid preference lasts only for the session, and there is
     currently no local application-preference store. Campaign storage holds
     shared world data, so this rendering preference should stay outside
     it." Options, recommended first: (A) "Save the atlas budget locally per
     device; add a small local preference store (recommended)." (B) "Keep it
     for this session only, matching the existing pixel-grid preference."
     Mark selected verbatim: "Save the atlas budget locally per device; add
     a small local preference store (recommended)." This extends 369 with
     persistence independent of campaign authority or network replication.
     *Reading, not ruled:* persist the requested preference and apply the
     current device's enforced bounds when using it; missing or malformed
     local data falls back to the ruled default without changing campaigns.

     Links: area Presentation · amends 369 · summary The atlas budget is saved locally per device in a small local preference store, outside campaign storage; malformed data falls back to the default.

374. **Allocate the chosen terrain atlas budget upfront.** Asked on
     2026-09-27: "Which allocation policy should paging use? Independent
     review confirms the new batched copy works, but full uploads remain
     faster: 0.191/0.598 ms versus 0.348/1.584 ms across the two growth
     cases, with 30 measured samples each. Upfront allocation uses the
     chosen budget (8 MiB by default); growing by rows starts the 260-brick
     demo at 256 KiB but introduces texture replacements. These local
     timings exclude rendering and are not frame-time guarantees."
     Options, recommended first: (A) "Allocate the chosen budget upfront;
     avoid growth replacements (recommended)." (B) "Grow by rows and fully
     upload on replacement; trade occasional replacement work for lower
     initial memory." (C) "Keep allocation open and investigate reusable
     staging or another copy design before choosing." Mark answered
     verbatim: "Agreed. The numbers have spoken." This accepts the
     recommendation, closing the allocation question opened in 370 and
     held open for measurement by 373. The budget remains configurable,
     default 8 MiB and bounded by the device (369), persisted locally per
     device (372). A changed budget still rebuilds the atlas; filling its
     available capacity does not trigger growth. Copy experiments remain
     research evidence, with their stated timing and hardware limits.

     Links: area Presentation · draws on 369, 372 · summary Allocate the chosen terrain atlas budget upfront (8 MiB default, device-bounded); a changed budget rebuilds the atlas but filling capacity does not trigger growth.

381. **One host carries the three games as modes over one world save.**
     Asked on 2026-09-28, in the session on Mark's hypothesis that the wing
     is one big game ("Critter lineage roguelike, denizen adventure rpg, sim
     world vtt, all isometric: rotatable, orthogonal projections"; session
     notes §9): "Imagining this as one big game: should the wing ship as one
     executable with modes over one world save, or stay three sovereign
     products?" Evidence as put, checked that day at isometry `bfa1b36` and
     mere `5ce144ff`: "the overlay contract is game-neutral, 1,256 lines
     with a sibling module per game; all four workspaces pin mere 5ce144ff;
     the only divergent patches are the VTT's p2panda set and Eponym's
     renderling leftovers." Options, recommended first: (A) "One host,
     modes: any build carries one or all three overlays over one world save;
     each product becomes a build profile of one host, and players in
     different modes can share a trunk. Commits: amending the founding
     record's 'never a shared running world' passage (your word needed),
     retiring Eponym's renderling patches, a mode switch in isomere's host
     assembly (recommended)." (B) "One game, one product: retire the three
     product identities for one roadmap." (C) "Three sovereign products: the
     hypothesis stays a design lens, not packaging." Mark chose A, "One
     host, modes". So the three games are modes of one host over one world
     save, each product a build profile of that host, and participants in
     different modes may share one trunk (ruling 104's option A). The three
     products keep their names and descriptions. The founding record's
     "joined by a shared *history*, never by a shared running world
     instance. A vessel must never be able to require another vessel to be
     running" is superseded in substance; being wing law, its amendment
     waits on Mark's word, drafted in the session notes' §9.4. *Reading, not
     ruled:* the founding guard against coupling as obligation survives as
     "no mode may require another mode's code to run", so a one-mode build
     stays whole; the mode switch belongs to isomere's host assembly under
     W3; Eponym's renderling patches retire with §4.2's debt; W5's order
     (231) stands.

     Links: area Host and network · summary One host carries the three games as modes over one world save; each product is a build profile; different-mode players may share a trunk.

382. **The wing has one default view, isometric with quarter turns, and it
     precludes no other view.** Asked on 2026-09-28, in the same session:
     "Which projection family should the modes share?" Evidence as put:
     "SlabCamera already takes any direction plus a cutaway, so every
     orthographic preset is a vector; the VTT's lens is a 30° dimetric
     preset; isometer has no texel snapping, so free yaw over pixel art will
     shimmer; FFT and Tactics Ogre turn in quarter steps, from general
     knowledge." Options, recommended first: (A) "Ortho, quarter turns:
     every mode orthographic; yaw turns in 90° steps with an animated turn;
     free yaw only in creative mode and the bench; pitch is a per-mode
     preset. Commits: texel-snapped camera, four-yaw bake sets for the far
     tier, Eponym's first-person setting dropped (recommended)." (B) "Ortho,
     free yaw: continuous rotation in every mode." (C) "Per mode, as ruled:
     record ruling 1 stands; each overlay picks its projection, first person
     included." Mark answered verbatim: "I don't think having a default
     isometric mode with quarter turns should preclude other views, like
     free yaw isometric, or first person, or third person over the shoulder.
     But a default across the wing is a powerful thing." So the wing has one
     default view, isometric with quarter turns, and free-yaw isometric,
     first person and third person over the shoulder stay available: ruling
     1 stands with a default added, and option A's dropped first person does
     not follow. The engine owes the default a texel-snapped camera,
     animated quarter turns and yaw bake sets for baked tiers, and owes the
     other views room: `SlabCamera` is orthographic only
     (`shared/isometer/src/camera.rs:48`), so first person and over the
     shoulder need a perspective camera the scene does not have. *Reading,
     not ruled:* "isometric" is read in the game-art sense, which includes
     the VTT's 2:1 pixel lens, a 30° dimetric (`camera.rs:108`), rather than
     only true isometric at 35.26°; the default's pitch, and whether
     Mesocosm opens in the default or keeps the terrarium section as its
     opening view, go back to Mark; the tabletop `CLAUDE.md`'s "Do not treat
     camera freedom as a near-term rendering task" conflicts with a
     quarter-turn default, and its amendment waits on his word, drafted in
     the session notes' §9.4.

     Links: area Presentation · amends 1 · summary The wing has one default view, isometric with quarter turns, and it precludes no other view (free-yaw isometric, first person, over the shoulder).

383. **Harmony, order and chaos are ecological states.** Asked on
     2026-09-28: "What is harmony → order → chaos in sim terms? It appears
     nowhere in the 380 rulings, the sim plan, or the workspace." Options,
     recommended first: (A) "Cost of coordination: a derived reading, never
     stored: harmony where members' alignments agree and consent suffices;
     order where a constitution diverges from its members and working
     enforcement holds the gap (rulings 64, 65, and 79's legitimacy gap);
     chaos where neither consent nor enforcement holds (recommended)." (B)
     "A field on places, like an aether or warp level, moved by processes
     and magic." (C) "A tenet axis sophonts value and hold as tenets." Mark
     answered verbatim: "I guess i just thought of it as like ecological
     balance; whether it's unmaintained and good, maintained and good, or
     unbalanced. Just figured that described, like, ecological states. Not
     so important." So harmony is a balanced ecology that nothing
     maintains, order a balanced ecology that something maintains, and
     chaos an unbalanced one. A state is a reading and is never stored (§1),
     and no lane opens for it. *Reading, not ruled:* balance is read off
     ruling 267's four measures, persistence, turnover, collapse and the
     response to an intervention, each bounded in the world's rules;
     maintained means the balance depends on acts some agent keeps making, a
     farmer, a gardener, a lineage engineering its habitat or a polity
     provisioning its people (ruling 38), so harmony is a web held by
     agentless processes and unchoosing members alone; option A's social
     reading is not adopted.

     Links: area Ecology · draws on 267 · summary Harmony, order and chaos are ecological states read from balance: harmony is balanced and unmaintained, order balanced and maintained, chaos unbalanced; never stored.

384. **Some divine figures can peek behind the curtain: an awareness of the
     sim beyond an organism's capabilities is magic that bends the rules.**
     Asked on 2026-09-28: "Is the omniscient end of the mind axis a seat
     someone plays?" Evidence as put: "Today omniscience is a view: creative
     mode sees without editing (184), the DM edits (156), and the storyteller
     is the world's own agency (103); ascension is parked (279)." Options,
     recommended first: (A) "Play the world: a fourth mode, the participant
     plays the world entity and directs its storyteller, bounded by the
     founder's conditions (recommended)." (B) "Views only: omniscience stays
     creative mode and the DM seat; the storyteller stays the world's own
     AI." (C) "Earned in-world: only through ascension, once the divinity
     rung is unparked; the seat belongs to the divine thing." Mark answered
     verbatim: "I would think some divine figures could peek behind the
     curtain so to speak. An unnatural awareness of the sim beyond an
     organism's capabilities is a cool bending of the rules, right? Seems
     magical!" Answered in the session: yes, in this record's own terms.
     §3.10 defines bending the rules as a magic system's declared suspension
     of an invariant, and this awareness suspends two for its bearer: ruling
     5's "memory is not global", knowledge arriving only by reach, and
     ruling 59's senses being the creature's own. So the omniscient end is
     not a seat or a fourth mode. It is an ability some divine figures hold,
     and it is magic, parked with divinity and magic under ruling 279.
     *Reading, not ruled:* the bearer reads truth rather than arrivals
     within a scope, its domain's effect, its place or its referent (rulings
     47 to 52), which the hagioglyph organ's plan owns; played in survival
     mode it widens what the played figure knows and so what the view shows
     (ruling 180), with no view machinery of its own; "some" makes it a
     trait a divine figure may have, not one its tier confers; option A's
     world seat is not adopted.

     Links: area Organs: glyphs, impresa, hagiograph · draws on 5, 59, 279 · summary Some divine figures can peek behind the curtain: awareness of the sim beyond an organism's capabilities is magic bending the rules, parked with divinity.

387. **The default view's pitch is the 2:1 dimetric, 30°.** Asked on
     2026-09-28: "What pitch is the wing's default view?" Evidence as put:
     "camera.rs:108: the VTT's 2:1 lens is a 30° pitch; true isometric's
     35.26° gives 2:1.155 tiles, which loses clean 2:1 pixel stair lines."
     Options, recommended first: (A) "2:1 dimetric, 30°: the shipped lens and
     what 'isometric' means in pixel art: clean 2:1 tile edges at integer
     scale. True isometric stays available as another view (recommended)."
     (B) "True isometric, 35.26°: equal foreshortening on all three axes;
     tile edges step unevenly in pixel art." (C) "Per mode: the default
     fixes quarter turns and orthography only." Mark chose A, "2:1 dimetric,
     30°". This answers ruling 382's reading: the default view is
     `SlabCamera::dimetric_2_1` (`shared/isometer/src/camera.rs:136`) turned
     in quarter steps, and true isometric is one of the other views.

     Links: area Presentation · draws on 382 · summary The default view's pitch is the 2:1 dimetric at 30 degrees, turned in quarter steps; true isometric stays available as another view.

391. **Neighbouring sites meet through edge profiles.** Asked on 2026-09-28:
     "How do neighbouring sites meet without seams? Ruling 73 allows any
     world shape, so there are no global coordinates to sample terrain noise
     in; Mesocosm's relief is one 65×65 diamond-square field that cannot
     tile; the place-graph plan's standing ruling 7 wants a top-down
     skeleton, then bottom-up detail." Options, recommended first: (A) "Edge
     profiles: a top-down skeleton gives each site coarse facts (elevation,
     relief kind, water); each adjacency carries a boundary profile drawn
     from both sites' facts and the edge's own seed, so both sides compute
     the same border; interior detail is free within the site's own frame.
     Works on any topology (recommended)." (B) "Stitch at lift time: each
     site grows from its own seed alone, and a border band blends into
     whichever neighbours are already lifted." (C) "Global coords, flat
     first." Mark chose A, "Edge profiles". *Reading, not ruled:* a profile
     is derived and never stored, being a function of asserted facts and the
     seed (§1), and an edit on a border is an ordinary asserted voxel edit;
     the place-graph plan's 2026-08-05 ruling 6, one global coordinate space
     in which an edge is never a portal, holds within a site's frame and
     gives way between sites, where crossing an edge changes frame (D15's
     across) without ever reading as a scene transition.

     Links: area World generation · summary Neighbouring sites meet through edge profiles: a top-down skeleton gives coarse facts, and each adjacency's boundary profile is computed identically from shared facts.

392. **Isocosm owns the lift from site to volume.** Asked on 2026-09-28: "Who
     owns the lift from site to volume? Isoscape ruling 11 puts terrain
     models beside isometer's Terrain seam and the seeded pipeline in
     isoscape; ruling 192 puts places in Isocosm; isocosm depends today only
     on wing-impresa, wing-glyphs and hagiograph; Ground sits in
     isometer-core, a render-family crate, and nisus is ruled its
     successor." Options, recommended first: (A) "Isocosm lifts: Isocosm
     owns the skeleton, the edge profiles, the lift and the derived places;
     the terrain models it calls stay verb-free stack functions beside the
     Terrain seam; the sim depends on no render crate, so an adapter feeds
     the lift's plain description to Ground; isoscape keeps founding presets
     (recommended)." (B) "Isoscape lifts: isoscape owns the whole
     site-to-volume pipeline and founds now." (C) "Isocosm owns all,
     terrain models included." Mark chose A, "Isocosm lifts". *Found on
     drafting the spine plan and put back:* option A's clauses meet in one
     place, since models "beside the Terrain seam" would sit in
     isometer-core, which the no-render-crate clause keeps out of the sim's
     graph, and isometer-core renders nothing (its dependencies are
     `wing-formats`, `serde` and `postcard`). Which clause gives way is the
     spine plan's decision 1 (§A.6).

     Links: area World generation · summary Isocosm owns the lift from site to volume (skeleton, edge profiles, lift, derived places); isoscape keeps founding presets; the sim depends on no render crate. (amended by 393)

393. **The terrain models live in Isocosm.** Asked on 2026-09-28, as the
     spine plan's decision 1: "Where do the terrain models the lift calls
     live? Ruling 392's two clauses meet here: models 'beside the Terrain
     seam' would sit in isometer-core, which its 'no render crate' clause
     keeps out of the sim." Evidence as put, checked at `d80163b`:
     "isometer-core depends only on wing-formats, serde and postcard and
     renders nothing; ruling 330 says Ground 'becomes a thin layer over
     [nisus] or retires' once T2 lands." Options, recommended first: (A)
     "Models in Isocosm: relief functions, detail noise and cavity shapes
     live in Isocosm as verb-free modules; isometer-core keeps only the
     Terrain trait and Ground; the bench adapter implements Terrain over the
     lift. Amends the isoscape plan's ruling 11, and nothing in the sim has
     to unwind when Ground thins at T2 (recommended)." (B) "Isocosm uses
     isometer-core: the models sit beside the seam as isoscape ruling 11
     says; amends ruling 392's no-render-crate clause." Mark chose A,
     "Models in Isocosm". So ruling 392's no-render-crate clause holds, its
     "beside the Terrain seam" gives way, and the isoscape plan's ruling 11
     is amended: the relief and brick description move into Isocosm, not
     beside isometer's seam.

     Links: area World generation · amends 392 · summary The terrain models (relief, detail noise, cavity shapes) live in Isocosm as verb-free modules; isometer-core keeps only the Terrain trait and Ground.

394. **A site's skeleton is keyed entries in its conditions.** Asked on
     2026-09-28, decision 2: "How is a site's skeleton held? Evidence:
     Site.conditions is already an asserted BTreeMap<Key, i64> per site;
     biome is a reading over conditions (ruling 72); the wing keeps open
     sets as data, never enums." Options, recommended first: (A) "Condition
     keys: elevation, relief and water level by default, in base units,
     named by the world's rules; a world may add keys such as temperature or
     rainfall that the lift reads. No change to Site's shape
     (recommended)." (B) "Typed fields, versioned with the schema." (C) "A
     wider typed set with climate and a watershed now." Mark chose A,
     "Condition keys". So the skeleton adds no field to `Site`: the world's
     rules name the keys the lift reads, elevation, relief and water level
     by default.

     Links: area World generation · draws on 72, 391 · summary A site's skeleton is keyed entries in its conditions (elevation, relief, water level by default), named by the world's rules; Site's shape is unchanged.

396. **The lift works in chunks at power-of-two cell sizes from the start.**
     Asked on 2026-09-28, decision 4: "At what grain does the lift work in
     the first slice?" Evidence as put, arithmetic and not measured:
     "Ground::grow fills every column of one extent at one cell size; a site
     of 256 five-foot tiles at a 7.5-inch voxel is 2,048 voxels across,
     about 4.2 million columns, where Mesocosm's 129-voxel enclosure is
     16,641; ruling 13 allows power-of-two cell sizes per chunk and ruling
     147 lets a site span as many chunks as it needs." Options, recommended
     first: (A) "Chunks from the start: the lift takes a chunk key and a cell
     size from day one, so a site of any size costs only what is lifted and
     far views lift coarse; the bench lifts a window across the shared
     border (recommended)." (B) "Whole small sites at one cell size, with
     chunking arriving at T2." Mark chose A, "Chunks from the start". With
     rulings 393 to 396 the spine plan's SP0 is done.

     Links: area World generation · draws on 13, 147 · summary The lift works in power-of-two-sized chunks from the start, taking a chunk key and cell size, so a site costs only what is lifted.

397. **Each route carries its border.** Asked on 2026-09-28, in SP1's
     brief: "Where does each adjacency's frame relation live (which side a
     route leaves by and which side it enters)? Evidence: ruling 72 holds
     the world map 'as adjacency and never as a two-dimensional array';
     Route today is { to, travel, transmission }; square grids could derive
     sides from a width-by-height descriptor, but spheres and bodies (ruling
     73) cannot." Options, recommended first: (A) "On the route: each Route
     gains an optional border naming the side it leaves by and the side it
     enters, and validation checks both directions agree; skipped when
     absent, so old worlds keep their digests. The same record serves
     hexagons, pentagons and bodies later (recommended)." (B) "A grid
     descriptor: the world holds its shape with a width and height, and
     sides are derived from site positions." Mark chose A, "On the route".
     *Reading, not ruled:* a border also says whether its two sides meet
     flipped, false on planes, rings and tori and reserved for
     non-orientable shapes, skipped when false.

     Links: area The sim's schema and saves · draws on 72, 73 · summary Each route optionally carries its border naming the side it leaves by and the side it enters, validated both ways and skipped when absent.

398. **The map family is an optional part of Founding.** Asked on
     2026-09-28: "How does the map family enter founding? Evidence: Founding
     is 'saved with every bench receipt' (generate.rs); the ecology places
     groups by random(\"habitat\", group) % sites and disperses along
     routes, so it runs over any site graph." Options, recommended first:
     (A) "Optional part of Founding: Founding gains an optional map domain,
     skipped when absent so every old receipt and digest stays
     byte-identical; when present it lays the sites, their borders and
     skeletons, and the ecology runs over them unchanged (recommended)." (B)
     "Its own domain, joined to Founding later." Mark chose A, "Optional
     part of Founding". *Reading, not ruled:* with a map present, its width
     times its height is the site count, and a founding whose `sites`
     disagrees is refused rather than reconciled.

     Links: area The sim's schema and saves · summary The map family is an optional part of Founding, skipped when absent so old receipts and digests stay byte-identical.

399. **SP1's draws are scale-free.** Asked on 2026-09-28: "What space do
     SP1's seeded draws cover? Evidence: no product default exists yet
     (Isocosm's base unit default is a 1,000 µm placeholder); the VTT's
     generator authors boards to 256 tiles; Mesocosm's enclosure is 129
     voxels across." Options, recommended first: (A) "Scale-free draws:
     grids from 2×2 to 16×16 over planes, rings and tori, and site sides
     from 256 to 2,048 base units, choosing no product default; defaults
     come when a game founds on the spine (SP7) (recommended)." (B) "Fix a
     default now, a 7.5-inch base unit and 256-tile sites (2,048 voxels)."
     Mark chose A, "Scale-free draws".

     Links: area World generation · draws on 396 · summary SP1's seeded draws are scale-free (grids 2x2 to 16x16, site sides 256 to 2,048 base units), choosing no product default until a game founds.

400. **A lifted site gives its skeleton back exactly where it can.** Asked on
     2026-09-28, the first of SP2's forks: "What must a lifted site give back
     when read up? SP2's done-condition says restricting a just-lifted site
     returns its skeleton. Evidence: SP1's corners take the mean elevation of
     the sites around them, so a surface built from a site's borders alone
     averages its neighbours, not itself; water is a fill level, exact by
     construction." Options, recommended first: (A) "Exact where it can be:
     the lift is corrected so the site's mean surface over its base-grain
     columns equals its elevation exactly, water fills exactly to its level,
     and relief bounds the interior detail; restriction computes all three
     from the volume, so the check actually tests the lift (recommended)."
     (B) "Skeleton passes through, the check an identity." (C) "Within a
     tolerance the world's rules state." Mark chose A, "Exact where it can
     be". *Reading, not ruled:* exactness is kept without summing every
     column, by holding the surface as the exact interpolation of a lattice
     whose sums have closed forms (the spine plan's §A.9).

     Links: area World generation · draws on 391 · summary A lifted site gives its skeleton back exactly where it can: mean surface equals its elevation, water fills to its level, relief bounds interior detail.

401. **A site's interior is a Coons patch.** Asked on 2026-09-28: "How is a
     site's interior surface built between its four borders? Evidence: SP1
     gives each site four profiles meeting exactly at shared corners; the
     interior must meet them exactly at the edges for the border check, and
     SP2's control is detail that does not fade at the edges." Options,
     recommended first: (A) "Coons patch: transfinite interpolation from the
     four profiles, which meets every edge exactly, plus a windowed
     correction to the site's own elevation and windowed detail from its
     seed; integer, one evaluation per column (recommended)." (B) "Harmonic
     fill, solved iteratively." (C) "Distance blend, which creases along the
     diagonals." Mark chose A, "Coons patch".

     Links: area World generation · draws on 400 · summary A site's interior surface is a Coons patch: transfinite interpolation from the four border profiles plus windowed correction and detail from its seed.

402. **A lift returns columns per chunk.** Asked on 2026-09-28: "What does a
     lift return? Evidence: SP2's volumes are heightfields with materials by
     depth, no caves yet; ruling 396 wants chunks at power-of-two cell sizes;
     isometer's Ground takes a surface per column, a sea level and cavities
     through its Terrain seam; a 2,048-voxel site is about 4.2 million
     columns at the base grain." Options, recommended first: (A) "Columns per
     chunk: each lift returns a chunk window of columns (surface height and a
     material-by-depth rule) at its cell size; dense voxels are lowered from
     columns only when a store needs them (SP6), and the bench adapter feeds
     Ground directly (recommended)." (B) "Dense voxel chunks from the start."
     Mark chose A, "Columns per chunk".

     Links: area World generation · draws on 396 · summary A lift returns columns per chunk (surface height and material-by-depth rule) at its cell size; dense voxels are lowered only when a store needs them.

403. **A lift's materials are world-local ids.** Asked on 2026-09-28: "What do
     a lift's voxels name as their material? Evidence: the wing index's
     working principle, 'A voxel material is a compact id into saved
     world-local definitions'; the sim types matter by provenance (nis,
     ruling 97), the world's own kinds being its geology; Ground's materials
     are palette indices up to 63." Options, recommended first: (A)
     "World-local ids: ids into a table saved with the world, seeded with
     air, water, soil and rock as the world kingdom's own nis, which later
     grows the world's generated kinds; the bench adapter maps ids to
     Ground's palette (recommended)." (B) "Palette indices for now." Mark
     chose A, "World-local ids".

     Links: area World generation · draws on 97 · summary A lift's voxel materials are world-local ids into a table saved with the world, seeded with air, water, soil and rock as the world's nis.

404. **Magic composes fundamental effects, paid for through equivalent
     exchange, with world-dependent scripts.** Asked on 2026-09-28, after
     reviewing Moirai and the wing: "How expressive may a generated
     operation become?" The recommendation was composition over a bounded
     vocabulary, with fundamentally new operations a separate content
     extension. Mark expanded it: "As long as we can explain ‘em, i favor
     hitting the fundamentals and then letting things run wild. I want to
     be surprised! I don’t mind broken builds. I enjoy caves of qud,
     remember? But it has to be fair…" His cost principle: "Equivalent
     exchange. Any effect (burn, freeze, shock, buff, debuff, heal…) should
     be available as a cantrip and combinable. But each level of combining
     takes a magnitude more energy."

     Every admitted fundamental effect has a small cantrip expression and
     may participate in combinations. More composition demands sharply
     greater power. Mark's tenth-level, ten-effect illustration expresses
     that escalation; it does not fix a formula, a multiplier, a count of
     primitives, or a conversion into any tabletop system's spell levels.
     Powerful or unexpected builds are welcome when their behavior and
     exchange can be explained. This does not make all effects free or
     automatically known to every caster.

     Mark wants the effects applied to typologies such as Homestuck's,
     Elder Scrolls', D&D's, Pathfinder's and PbtA's, with ruleset spells as
     distinctive exemplars and generated spells extending the catalogue
     and expressing the world. These are requested comparisons and adapter
     directions, not an assertion that those systems share one taxonomy.
     CoreRPG and individual ruleset mappings remain to be designed.

     The core expands by defining an effect's script: its fundamental
     character plus fungible characteristics influenced by simulation.
     Mark's examples are fire that burns hotter near fire and costs less
     mana, ice that spreads until stopped by the sun, and stronger gravity
     in a larger world. These are possible authored world laws, not default
     laws or real-world physical claims. In his words: "basically hook
     other generators of the sim into the effect’s script calculation";
     mod authors and the generator should both be able to create such
     magic. Ball x Pit's combinations are an analogy for the composition,
     not a prescribed algorithm. Ways of using magic remain open.
     *Given a reference 2026-10-03 by ruling 522:* Ars Magica's Hermetic
     magic, Techniques composed with Forms at levels its guidelines set,
     informs the design, nothing of it copied into the sim.

     Links: area Processes and rules · summary Magic composes fundamental effects, each a combinable cantrip, each level of combining costing a magnitude more energy; effect scripts hook other generators; explainable surprises welcome.

405. **Construction declares function; geometry measures what a mechanic
     needs.** Asked in the same review: "When does geometric detail affect
     the rules?" The recommendation was: "construction declares functional
     properties; geometry supplies measurements where a mechanic needs
     them, such as clearance, reach, contact, or coverage. We should name
     those dependencies explicitly." Mark answered: "Yeah, that’s a good
     recommendation." Fine geometry need not be simulated to discover every
     capability. A declared property that depends on a measurement must
     identify it so an accepted body change can invalidate the reading.
     *Applied to bodies 2026-10-02 by ruling 462:* the sim keeps where each
     part attaches and measures reach, clearance and contact itself.
     *Applied to the catalogue 2026-10-02 by ruling 492:* each function's
     mechanic names the measurement that scales it.

     Links: area Bodies and anatomy · summary Construction declares functional properties; geometry supplies measurements where a mechanic needs them, and a declared property depending on a measurement must name it.

406. **A new life selects conditions to carry forward, with granular player
     configuration or randomization.** Asked: "How are consequences
     translated between substantially different embodiments?" Mark's
     answer, verbatim: "I would say it can influence a person in a lot of
     different ways. A detriment to the part, or the part being gone
     entirely, or even a benefit related to the incident; old losses perhaps
     led to new abilities that became characteristic… i think practically
     it’s like choosing what conditions to bring forward to the next life.
     That decision can be randomized or configured by the player. Or even
     just a blanket “keep conditions?”prompt would be fine, though i would
     prefer the more granular choices."

     History remains pointable while its expression in the next life can
     be a detriment, absence, benefit, or characteristic ability connected
     to the incident. Granular choices are preferred; randomized selection
     and a blanket keep-conditions option are supported design directions.
     Not selecting an active condition does not erase its historical event.
     This governs carryover after a new life is available; it does not
     change the existing rules for gaining reincarnation. Mapping choices,
     their costs or restrictions, and default selection remain to be designed.

     Links: area Directing and play · summary A new life selects which conditions to carry forward, granular by the player's configuration or randomized; unselected conditions keep their historical event.

407. **Construction and action distinctions from the design discussion are
     consolidated.** This records earlier answers in this conversation,
     rather than presenting them as new choices made on 2026-09-28. Mark
     chose mostly generated forms from geometric and functional primitives,
     nested composition, general compatibility enabling wild combinations,
     and construction determining capabilities and limits with stylized
     movement respecting them. He chose: "Material and conceptual operations
     share the composition system." Tradeoffs are circumstantial: "a failure
     in one epoch could be an advantage in another." He accepted the
     distinctions with "capability, repertoire, proficiency. agreed", and
     placed perception and self-perception with belief and tenets, including
     believing oneself unable to do something actually possible.

     Recipe and history provide continuity while embodiment and mechanics
     may adapt to worlds and rulesets. Incarnation is part of that model.
     The earlier objection to costly physical discovery of all function
     stands: generation assembles meaningful components; ordinary inherited
     behavior need not be learned from zero. These decisions refine the
     body and functional plans; they do not certify the current single-part
     bindings or three-operator evaluator as the complete composition system.

     Links: area Bodies and anatomy · summary Forms are mostly generated from geometric and functional primitives by nested composition; construction determines capabilities and limits; capability, repertoire and proficiency differ.

412. **Earth is matter: the volume is the world's own body.** Asked on
     2026-09-30, the spatial spine's SP4 design (Mark: "We'll design things
     until implementation is a piece of cake"): "When something carves or
     fills the volume, what happens to matter? Evidence: Mesocosm keeps
     terrain and edible soil as separate stores ('A carve changes the first
     and not the second', TD6); Isocosm's world:soil account is a site's
     edible pool; my SP2 reading wrongly made the voxel material world:soil
     the same nis as that account; ruling 12 makes terrain destructible and
     constructible." Options, recommended first: (A) "Earth is matter: the
     volume is the world's own body: carving moves each cell's mass (a
     density per material, a world rule) into whoever carved it, filling
     draws from them, so dug earth can be carried and built with and nothing
     appears or vanishes. Edible soil stays its own pool, as TD6 rules, and
     the voxel material gets its own name (recommended)." (B) "Separate,
     shape only, spoil unmodelled." (C) "Topsoil is the pool, amending TD6."
     Mark chose A, "Earth is matter". So SP2's reading that the voxel
     material and the ledger's soil are one nis is withdrawn. *Reading, not
     ruled:* the voxel material is renamed `world:earth`; air weighs
     nothing; edits by a DM, the world editor or creative mode take and
     return matter through the dev source outside the conserved total and
     label the run assisted, as ruling 271 has it for placed matter.

     Links: area World generation · draws on 12 · summary Earth is matter: the volume is the world's own body; carving moves each cell's mass to the carver, filling draws from them; soil stays separate.

413. **Edits are stored as shape operations.** Asked on 2026-09-30: "What is
     stored when the volume is edited? Evidence: Ground's own edits are
     shapes (carve at a point with a radius; cavities as rooms and a route);
     the done-condition wants stored bytes that grow with edits and not with
     sites; lifts must replay every edit a chunk touches." Options,
     recommended first: (A) "Shape operations: each edit is a shape and a
     material (carve a sphere, fill a box, dig a route), stored in order per
     site and replayed onto any chunk it overlaps; compact for big edits,
     exact for small ones (recommended)." (B) "Cell edits at the base
     grain." (C) "Shapes, compacted into cells when they grow too many."
     Mark chose A, "Shape operations".

     Links: area World generation · draws on 412 · summary Volume edits are stored as ordered shape operations per site (carve a sphere, fill a box, dig a route), replayed onto any chunk they overlap.

414. **An edit reaching across a border is one fact, read across.** Asked on
     2026-09-30: "How does an edit that reaches across a border belong to the
     sites? Evidence: SP1 gives every border a frame relation both sites
     know; SP4's done-condition says 'an edit on a border is seen from both
     sides'." Options, recommended first: (A) "One fact, read across: an edit
     is stored once, in the frame of the site it was made in, with its
     extent; any neighbour it reaches reads it through the border's frame
     relation (recommended)." (B) "Split when made, one part per site." Mark
     chose A, "One fact, read across".

     Links: area World generation · draws on 413, 391 · summary An edit reaching across a border is stored once, in its origin site's frame, and neighbours read it through the border's frame relation.

415. **A lift carries caves as columns plus exceptions.** Asked on
     2026-09-30: "How does a lift carry caves and overhangs? Evidence: SP2's
     chunk is columns (a surface top and a material-by-depth rule, ruling
     402), which cannot hold a tunnel under the surface; ruling 402 lowers
     dense voxels only when a store needs them (SP6)." Options, recommended
     first: (A) "Columns plus exceptions: a chunk keeps its columns and adds
     a sparse list of cells that differ from the column rule, so an
     untouched chunk costs what it did and an edited one pays only for its
     changes (recommended)." (B) "Dense when edited." (C) "Surface edits
     only for now." Mark chose A, "Columns plus exceptions".

     Links: area World generation · draws on 402 · summary A lift carries caves and overhangs as columns plus a sparse list of exception cells that differ from the column rule.

416. **What a carver cannot hold is heaped beside the cut.** Asked on
     2026-09-30: "When a carve yields more earth than the carver can hold,
     where does the rest go? Evidence: ruling 53 makes holding containment by
     capacity (biology, gear or construction), but Isocosm models no capacity
     yet, so this sets the rule for when it does; ruling 412 says nothing
     appears or vanishes." Options, recommended first: (A) "Heaped beside the
     cut: what the carver cannot hold is filled back as a heap next to the
     cut, an automatic fill edit, so spoil is visible terrain, conserved, and
     can be dug again. Until capacity exists, the carver holds all of it
     (recommended)." (B) "The site's stock, invisible in the volume." (C)
     "Refused whole." Mark chose A, "Heaped beside the cut".

     Links: area Processes and rules · draws on 53, 412 · summary Earth a carver cannot hold is heaped beside the cut as an automatic fill edit, conserved and diggable; until capacity exists the carver holds all.

417. **The outdoors divides into walkable patches, capped.** Asked on
     2026-09-30, the first of SP5's forks: "How is open air divided into
     places? Evidence: the air above a site is one connected component (the
     sky), so components alone cannot divide the outdoors; the record's §3.7
     lists 'outdoor regions' beside rooms, caves and corridors, aiming at ten
     thousand nodes, not ten million; Mesocosm divides its enclosure by a
     fixed three-by-three partition today." Options, recommended first: (A)
     "Walkable patches, capped: rooms, caves and tunnels are air cut off from
     the sky; the outdoors floods over walkable surface and splits wherever
     cliffs or water stop a walker, and a patch larger than a cap is cut on
     the chunk grid. Places follow how things move (recommended)." (B)
     "Drainage basins." (C) "Chunk grid." Mark chose A, "Walkable patches,
     capped". The cap's form is ruling 420's.

     Links: area The sim's schema and saves · summary Open air divides into walkable patches split by cliffs or water; a patch larger than a cap is cut on the chunk grid.

418. **Places are derived at the base grain, clearance recorded on
     edges.** Asked on 2026-09-30: "At what grain are places derived, and for
     which body? Evidence: Mesocosm's P9 and P10 showed live body geometry
     changing both passage and sight (compact and broad bodies diverge in the
     same tunnels); §5's third stop rule forbids deriving from a view's cell
     size." Options, recommended first: (A) "Base grain, clearance on edges:
     places are derived once at the base grain; every passage records its
     clearance (the widest and tallest body that fits), and a route for a
     body filters by it, so one graph serves every body (recommended)." (B)
     "A graph per body class." (C) "One declared walker." Mark chose A,
     "Base grain, clearance on edges".

     Links: area The sim's schema and saves · draws on 417 · summary Places are derived once at base grain; each passage records its clearance so one graph serves every body by filtering.

419. **A world-rule climb splits walkable patches.** Asked on 2026-09-30:
     "What steepness splits one walkable patch from the next? Evidence:
     ruling 417 splits patches 'wherever cliffs or water stop a walker', and
     ruling 418 records each passage's clearance so bodies differ per edge;
     drawn relief runs to an eighth of a site's side, and SP2's detail makes
     local steps; the base unit is scale-free." Options, recommended first:
     (A) "World-rule climb: the world's rules set a climb, default one base
     unit of rise per unit of run (45°); anything steeper splits patches, and
     each edge records its step so a body that climbs higher still passes
     (recommended)." (B) "From the roster, the least climb any living lineage
     can manage." (C) "No slope split, steepness only an edge's cost." Mark
     chose A, "World-rule climb".

     Links: area The sim's schema and saves · draws on 417, 418 · summary A world-rule climb (default one unit of rise per unit of run, 45 degrees) splits walkable patches; each edge records its step.

420. **The patch cap is a world rule with stepped presets.** Asked on
     2026-09-30: "How large may an outdoor patch grow before it is cut on the
     chunk grid? Evidence: the record's §3.7 aims at ten thousand places, not
     ten million; ruling 124's region is hundreds of sites, though only
     lifted sites have places below the site; a 2,048-unit site holds 64
     chunks at level 3 (256 units a side) and 4,096 at the base (32 units)."
     Options, recommended first: (A) "A level-3 chunk, at most 256 by 256
     base units (recommended)." (B) "A base chunk, 32 by 32." (C) "A world
     rule, no default." Mark answered verbatim: "World rule with stepped
     defaults: small (16), extra small (32), medium(64), extra medium (128),
     large (256), extra large (512), your choice (n by n)". Put back, since
     extra small came out larger than small: "Which order did you mean?"
     with the options "Names ascend" and "As written". Mark: "Names ascend,
     also, good catch. Also, would it be possible to do x by y, so 128 x 256,
     for example?" Answered in the session: yes, nothing needs squares; the
     cap is a grid of x by y cells laid over each site in its own frame,
     and powers of two only line up with chunk edges. So the presets, sides
     in base units, are extra small 16, small 32, medium 64, extra medium
     128, large 256 and extra large 512, and a founder may set their own x by
     y.

     Links: area The sim's schema and saves · draws on 417 · summary The patch cap is a world rule with stepped presets (16 to 512 base units a side) and founder-set x by y cells.

421. **A world picks the large preset, 256, by default.** Asked on
     2026-09-30: "Which preset does a world take when its founder picks none?
     Evidence: a largest drawn site (2,048 units) holds 64 patches at 256,
     256 at 128 and 1,024 at 64; the record's §3.7 aims at ten thousand
     places, not ten million, and only lifted sites have places below the
     site." Options, recommended first: (A) "Large, 256 (recommended)." (B)
     "Extra medium, 128." (C) "Medium, 64." Mark chose A, "Large, 256".

     Links: area The sim's schema and saves · draws on 420 · summary A world takes the large preset, 256 base units, by default when its founder picks none.

422. **An entity names its site until its game places it.** Asked on
     2026-09-30: "Once a site has places, what does an entity's location
     name? Evidence: Entity.place is a site id today; the sim never
     pathfinds or knows positions (§3.8); ruling 275, who owns fine position
     up close, waits on a measurement; reach, travel and plague run over
     places (§3.7)." Options, recommended first: (A) "Site until placed: an
     entity names its site; in a lifted site where its game has placed it,
     it names its patch or room, and restriction returns it to the site.
     Places are the finest location the sim holds; coordinates stay the
     foreground's (recommended)." (B) "Always the site." (C) "Coordinates up
     close." Mark chose A, "Site until placed". Ruling 275's measurement
     still decides who moves a body within a place.

     Links: area The sim's schema and saves · summary An entity names its site until its game places it, then its patch or room; places are the finest location the sim holds.

423. **The detail ladder has five rungs in one camera, and its far marks
     show modifiers.** Asked on 2026-09-30: "What rungs does the ladder
     hold, near to far? Evidence: isometer draws live voxel parts, baked
     sprites (isometer-mesh's bake), glyph marks and traced terrain; the
     VTT's overmap (sprigging's canvas) and Mesocosm's minimap (hulls on the
     HUD lane) draw places outside the scene; the record suggests
     Ptree-style distributions for cohorts, unbuilt; ruling 382 makes the
     isometric view with quarter turns the wing's default." Options,
     recommended first: (A) "Five rungs, one camera: live parts over
     base-grain terrain; baked sprites over coarse terrain; a mark per
     individual over surface tiles; aggregate marks for groups and cohorts
     over places; the world map, sites coloured by their readings with
     routes. All in the one isometric camera, the map included
     (recommended)." (B) "Three rungs and a map view: parts, sprites and
     marks in the scene; the world map stays a separate document view, as
     the overmap and minimap are today." (C) "Continuous, no rungs: each
     thing picks its representation by projected size alone, with no fixed
     set of rungs." Mark chose A and reframed its marks: "Five rungs, one
     camera, but where you’ve put glyphs and marks, i would think of them
     as modifiers and aggregate modifiers. The glyphs were really just
     effects that would be applied, and there are other effects/means of
     modifying things… unless literally every effect or modification is
     derived from a glyph, like the spell parts in elder scrolls spell
     creation… the point being, there are also conditional modifiers, like
     things like adjacency perks, that might restate or define a new
     modifier in terms of others (or generally, modifier combination…) so
     it might help to think of how we are representing conditions,
     statuses, and modifiers as applied to entities individually and in the
     aggregate".
     So one camera draws five rungs, the world map among them. The third
     and fourth rungs mark modifiers and aggregate modifiers, of which a
     glyph's effect is one kind. How conditions, statuses and modifiers are
     held on entities, singly and in aggregate, opens as its own question
     ([session notes, §10](../mesocosm/design_docs/archive_docs/2026-10-06/2026-09-22_sim_design_session_notes.md#10-the-design-session-2026-09-30)).
     *Reading, not ruled:* the question used "mark" in two senses, the
     effect experiment's drawn glyph effects (`mesocosm-core`'s
     `effect_experiment`) and a symbol standing for a thing too small to
     draw; the answer joins them, so a thing's mark at those rungs shows
     what modifies it.

     Links: area Presentation · summary The detail ladder has five rungs in one isometric camera, world map included; far marks show modifiers and aggregate modifiers, not only glyphs.

424. **A thing shows the finest rung its size warrants among those the sim
     holds.** Asked on 2026-09-30: "What decides which rung a thing shows?
     Evidence: the stop rule forbids render LOD or residency choosing the
     sim's representation; ruling 212 makes what the view shows up close an
     attention intent in the log, which is what lifts detail in the sim."
     Options, recommended first: (A) "Size, within what the sim holds: each
     thing takes the finest rung its projected size warrants among those
     the sim holds for it; zooming in examines (212), which lifts detail, so
     finer rungs arrive through the log, never by the renderer's choice
     (recommended)." (B) "Attention drives the view: rungs follow what the
     player attends rather than distance: pinned and examined things show
     fine, the rest stay coarse whatever the zoom." (C) "Sim tier alone: the
     rung is the sim's tier for the thing, individual, cohort or site,
     whatever its size on screen." Mark chose A, "Size, within what the sim
     holds". So projected size proposes and the sim's keeping bounds it: a
     cohort held as a histogram (207) shows aggregate marks however close
     the camera comes until examining lifts it, and the renderer never
     draws an individual the sim does not hold.

     Links: area Presentation · draws on 423, 212 · summary A thing shows the finest rung its projected size warrants among those the sim holds; finer rungs arrive only through the log.

425. **Terrain fits the budget in nested rings.** Asked on 2026-09-30: "How
     does terrain fit the budget across a zoom? Evidence: ruling 296's host
     budget, 8 MiB or about 16,383 bricks, while SP3's bench measured
     against GroundTerrain's default of 2,047; 291 makes residency follow
     the view; SP2 lifts any region at any power-of-two cell size; clipmaps
     are marked consumer-gated in the engine review." Options, recommended
     first: (A) "Nested rings: base grain where the view examines, each ring
     outward a power of two coarser, all inside the host's device budget;
     the bench adopts that budget like the VTT board (recommended)." (B)
     "One grain per view: the whole view takes one grain, the finest that
     fits the budget; simple, and coarse up close in wide views." (C) "Base
     grain, refuse overflow: page base grain everywhere visible and refuse
     what overflows, as SP3 does now." Mark chose A, "Nested rings". So
     ruling 291's residency becomes rings: base grain over what the view
     examines, each ring outward lifted one level coarser (SP2's chunks),
     all inside 296's 8 MiB, and the specimen bench leaves GroundTerrain's
     default of 2,047 for the host's budget. A ring's grain is residency and
     never selects the sim's representation, since every level lifts from
     the same exact surface.

     Links: area Presentation · draws on 291, 296 · summary Terrain residency uses nested rings: base grain where the view examines, each ring outward coarser, all inside the host's 8 MiB device budget.

426. **A thing changes rung seamlessly where the rungs share geometry, and
     by dither where they do not.** Asked on 2026-09-30, after the session
     notes' account of what a continuous morph would require (§10), Mark's
     own question in place of an answer to the first round's fourth: "Now
     that the morph's needs are known, how does a thing change rung?
     Evidence: sprites and live parts agree to the pixel on silhouette (3
     bodies × 4 facings) and share the bake's palette; terrain grains and a
     site's plate can slide exactly; mark to sprite needs a halved-bake
     chain (about +1/7 voxels) and unfolding needs per-part bakes (atlas ×
     part count); a group's split waits on the lift (212)." Options,
     recommended first: (A) "Seamless where shared: sprites and live parts
     meet pixel for pixel, terrain columns slide between grains, a site's
     plate rises into its relief with its mean held; marks, sprites and
     groups, which share no geometry, swap by ordered dither with
     hysteresis. Every frame exact (recommended)." (B) "Continuous morph
     throughout: adds per-part bakes, the halved-bake chain for marks, and
     members bursting from their group once the lift lands (or on approach,
     moving 212's line). Richest, with frame receipts per rung pair and
     quarter turn." (C) "Dithered everywhere: one mechanism for every rung
     pair: an ordered dither over a few frames with hysteresis, as first
     recommended." (D) "Hard pop, with hysteresis: the rung switches in one
     frame; simplest and exact, and visible as a pop." Mark chose A,
     "Seamless where shared", answered on 2026-10-01. So the step from
     sprite to live parts owes a measurement, since agreement on silhouette
     is not yet agreement on the picture: the GPU pass must land the bake's
     pixels and face shading at the threshold. Terrain slides column by
     column between the rings' levels (425), a site's plate eases into its
     relief, and marks, sprites and groups dither with hysteresis at each
     threshold. A group still dithers into members only once they are
     lifted (424); whether examining fires on approach is not ruled.
     *Reading, not ruled:* neither per-part bakes nor a chain of halved
     bakes is needed under this ruling.

     Links: area Presentation · draws on 424, 425 · summary Rung changes are seamless where rungs share geometry (sprites, live parts, terrain grains) and ordered dither with hysteresis where they do not.

427. **Every modification applies effects from the world's one vocabulary,
     and a glyph is one source among many.** Asked on 2026-09-30: "Is every
     modification built from the one effect vocabulary? Evidence:
     wing-glyphs binds each glyph to an effect id that a canon revision may
     move; 404 makes every fundamental effect a combinable cantrip; 407
     puts material and conceptual operations in one composition system;
     Elder Scrolls' spells, potions, enchantments, diseases and birthsigns
     all compose one effect list." Options, recommended first: (A) "One
     vocabulary, many sources: every modifier applies effects from the
     world's vocabulary; a glyph is one source beside traits, parts, items,
     places, conditions and relations, and a canon revision moves only what
     glyphs bind (recommended)." (B) "Effects for magic only: magic
     composes effects and pays per 404; mundane modifiers (terrain, gear,
     hunger) are plain adjustments to readings, authored apart." (C) "Every
     modifier is a glyph: every modification is a glyph in some canon,
     mundane ones included; uniform, and a canon revision can then change
     what being wet does." Mark chose A, "One vocabulary, many sources",
     answered on 2026-10-01. So the effect is the atom of modification, as
     Elder Scrolls' spell parts are, which answers 423's "unless literally
     every effect or modification is derived from a glyph": every one is
     derived from an effect. A modifier names its source, a glyph, trait,
     part, item, place, condition or relation, and a canon revision moves a
     glyph's binding without touching modifiers from other sources.
     *Reading, not ruled:* ruling 49 gives every effect the world
     recognises as fundamental "its own characteristic manifestation (a
     glyph)", so every modification's effect has a glyph whatever its
     source, and the far rungs can mark a thing's modifiers with their
     effects' glyphs. A ruleset's own arithmetic over readings, such as the
     SRD's `s_speed`, stays the ruleset's, which the sim never sees (§5).

     Links: area Processes and rules · draws on 404, 407, 423 · summary Every modification applies effects from the world's one vocabulary; a glyph is one source beside traits, parts, items, places, conditions and relations.

428. **An entity holds a condition as a record: key, magnitude, cause and
     an optional end.** Asked on 2026-09-30: "How does an entity hold a
     condition? Evidence: sites keep keyed conditions and VTT tokens keyed
     magnitudes; the handoff's Condition { subject, condition, magnitude }
     has nowhere to land, since Entity keeps none; strain is kept as an
     account that eases (159), flags as traits; 406 carries chosen
     conditions into a next life, and inheritance must be pointable."
     Options, recommended first: (A) "A condition record: key, magnitude,
     the event that caused it and an optional end; a status is a condition
     with an end (poisoned for ten turns), a lasting injury one without.
     Readings read them, the handoff lands in them, 406 chooses among them
     (recommended)." (B) "Traits and accounts: no new structure: a
     present-or-absent condition is a trait, and one with a magnitude is an
     account that eases, as strain does." (C) "Readings only: every
     condition derives from the ledger and the record (malnourished from
     matter held), never kept." Mark chose A, "A condition record",
     answered on 2026-10-01. So an entity keeps conditions with their
     causes pointable, a status is a condition with an end, the overlay's
     handoff lands in them, and 406's carryover chooses among them.
     *Reopened on 2026-10-01* before it was carried into the sim plan: the
     question's evidence left out the record's §3.5, "Conditions live on
     places, not on things", one of its four rules for holding hundreds of
     thousands of things, and ruling 123, which already keeps a body's
     wounds and vigour in its ledger, so option A's example, a lasting
     injury, had a home. Put back to Mark with both. *Settled 2026-10-01 by
     ruling 430:* the record stands beside the ledger, and §3.5's rule is
     narrowed to the environment's conditions.

     Links: area Processes and rules · summary An entity holds a condition as a record of key, magnitude, cause and optional end; a status is a condition with an end. (amended by 430)

429. **Modifiers combine in layers the world orders.** Asked on 2026-09-30:
     "How do modifiers that read other modifiers combine? Evidence:
     Balatro's jokers apply left to right, each reading the running score;
     Magic: The Gathering orders continuous effects in seven layers;
     Pathfinder takes the highest bonus of each type; 268 bounds amounts as
     expression trees; 207's crowd needs each member's reading computed
     once, deterministically." Options, recommended first: (A) "Layers in
     world order: each modifier sits in a layer the world's rules order and
     reads conditions and earlier layers only, so no cycle can form; within
     a layer each reading declares its stacking: sum, highest, lowest,
     product or replace (recommended)." (B) "The bearer's arrangement:
     modifiers apply in the order the bearer's build sets, each reading the
     running result, as Balatro's jokers do; arranging becomes play, as the
     mosaic already makes adjacency play." (C) "Dependency order: a
     modifier reads whatever it names; evaluation follows its dependencies,
     and authoring refuses a cycle." Mark chose A, "Layers in world order",
     answered on 2026-10-01. So an adjacency perk, or a modifier defined in
     terms of others, reads the layers beneath its own; every reading
     terminates and is computed once per state, as the crowd needs; the
     world's rules declare the layers and each reading's stacking; and
     computed amounts stay bounded expression trees (268). *Reading, not
     ruled:* the mosaic's adjacency (36) sits in such a layer, reading its
     neighbouring tracts.

     Links: area Processes and rules · draws on 268 · summary Modifiers combine in layers the world orders, reading only earlier layers; each reading declares stacking: sum, highest, lowest, product or replace.

430. **The environment's conditions live on places; a thing's own
     condition record sits beside its ledger.** Asked on 2026-10-01,
     putting 428 back: "Ruling 428's record meets two lines its question
     left out: the record's §3.5, "Conditions live on places, not on
     things" (one of four rules for holding hundreds of thousands of
     things, where processing, not memory, is the limit), and ruling 123,
     which keeps wounds and vigour in the body's ledger. Under 207 a crowd
     groups by what the rules read, so a record's cause need not split a
     bin, and an end can be a scheduled event (§3.5's second rule). How
     should they meet?" Options, recommended first: (A) "Narrow §3.5,
     record beside ledger: §3.5 becomes "the environment's conditions live
     on places", which things read where they stand; a thing's record keeps
     what its ledger and body do not, states with a cause and perhaps an end
     (poisoned, blessed, prone); wounds and vigour stay the ledger's (123),
     and 406 chooses among both (recommended)." (B) "One home for every
     state: as the first, but wounds move from the ledger into condition
     records, so everything a carryover chooses among lives in one place;
     vigour stays a pool." (C) "Withdraw the record: §3.5 stands as
     written; a thing's states stay ledger and traits (poison an account
     that eases, a curse a trait), their causes pointable through the
     history; the handoff maps onto them." Mark chose A, "Narrow §3.5,
     record beside ledger". So §3.5's fourth rule now reads "the
     environment's conditions live on places": rain, heat and plague
     exposure stay fields that things read where they stand, never copied
     onto each thing. 428's record holds the states a thing's ledger and
     body do not, and its example is corrected: wounds and vigour stay the
     ledger's (123), a severed part the body's. A crowd groups by what the
     rules read (207), so a record's cause splits no bin, and an end is a
     scheduled event, never a countdown. 406's carryover chooses among
     records and wounds alike.

     Links: area Processes and rules · amends 428 · summary Environmental conditions live on places; a thing's own condition record sits beside its ledger, wounds and vigour staying the ledger's.

431. **A modifier declares whether it applies to each member or to the
     group.** Asked on 2026-10-01: "How does a modifier apply to a group?
     Evidence: 207 groups a crowd by each member's exact state as the rules
     read it; the schema keeps quantity per member, a cohort's total being
     quantity times multiplicity; a polity keeps accounts of its own; ruling
     58 makes a fungus one body and germs many." Options, recommended first:
     (A) "Declared per modifier: a distributive modifier applies to each
     member and enters its state, splitting the histogram only where its
     predicate differs; a collective one belongs to the group, as a polity's
     accounts do (a swarm's cover, a herd's vigilance), and splits nothing
     (recommended)." (B) "Always to members: a group keeps no modifiers of
     its own; collective effects sit on the polity or body that holds the
     group, and the aggregate only summarises its members." (C) "The group
     stands in: while held as a crowd, members' modifiers fold into the
     group's as rates, and lifting deals them back out; cheapest, and
     inexact against 207." Mark chose A, "Declared per modifier". So every
     modifier declares itself distributive or collective. A distributive one
     enters each member's state and splits a crowd only where its predicate
     reads members differently; a collective one is the group's own, as a
     polity's accounts are. These are 423's aggregate modifiers. *Reading,
     not ruled:* a fungus, one body however far it spreads (58), takes
     collective modifiers on that body, and germs, many bodies, distributive
     ones.

     Links: area Processes and rules · draws on 207, 58 · summary Each modifier declares itself distributive (applies to each member) or collective (belongs to the group); collective splits no crowd.

432. **A far mark shows the glyphs of the effects on a thing, through a
     lens the player sets.** Asked on 2026-10-01: "What does a far mark show
     of what modifies a thing? Evidence: ruling 49 gives every fundamental
     effect its own manifestation, a glyph, and wing-glyphs keeps each
     glyph's display; knowledge access decides what a viewer may learn and
     presentation only how it is shown (the record's architecture,
     2026-09-28); 207's histogram holds each group's exact shares." Options,
     recommended first: (A) "Effect glyphs through a lens: a thing's mark
     shows the glyphs of the effects on it that the viewer may know,
     filtered by a lens the player sets, with a default per mode; a group's
     mark gives each glyph the share of members it touches, or the group's
     own value when collective (recommended)." (B) "Fixed per game: each
     game decides what its far marks show (Mesocosm hunger and lineage, the
     VTT conditions, Eponym standing), with no player lens." (C)
     "Everything known, as pips: every known modifier shows as a small pip,
     unranked and unfiltered." Mark chose A, "Effect glyphs through a lens".
     So the third and fourth rungs (423) draw the world's own glyphs: an
     individual's mark carries the glyphs of the effects modifying it, and a
     group's mark each glyph with its exact share of members (207) or the
     group's own value (431). Knowledge access bounds what may show, the
     player's lens chooses among it, and each mode supplies a default.
     *Reading, not ruled:* the word "lens" collides with the wing's lens, a
     projection (the VTT's locked isometric lens, `isometer-lens`); naming
     this filter is Mark's. *Named 2026-10-01 by ruling 434:* the player's
     filter.

     Links: area Presentation · draws on 423, 49, 207 · summary A far mark shows the glyphs of the effects on a thing, or per-glyph member shares for a group, through a player-set filter. (amended by 434)

433. **A modifier reaches only the scopes the sim holds.** Asked on
     2026-10-01: "How far does a modifier reach beyond its bearer?
     Evidence: the sim never pathfinds or checks line of sight (§3.8) and
     keeps coordinates the foreground's (422); it does hold bodies and their
     mosaics, relations, groups, places and sites; the mosaic's adjacency
     lies within one body (36)." Options, recommended first: (A) "Scopes the
     sim holds: a modifier reaches its bearer, its body's neighbouring
     tracts, its relations, its group, its place or its site; adjacency on
     a battlemap belongs to the game holding positions, in its ruleset
     (recommended)." (B) "Bearer only: a modifier touches only its bearer;
     reaching others is a process that gives them a condition." (C)
     "Through the reach field: auras spread as the reach field carries
     events, fading with distance and time." Mark chose A, "Scopes the sim
     holds". So a modifier declares its scope among bearer, body, relation,
     group, place and site, and 423's adjacency perks come in two kinds:
     between a body's tracts, the sim's (36), and between pieces on a board,
     the game's, resolved by its ruleset with positions the sim never holds
     (§3.8, 422). *Reading, not ruled:* a modifier scoped to a place or a
     site whose predicate reads only the place applies alike to every member
     there, and splits no crowd.
     *Noted 2026-10-02 with ruling 502:* Mark sees territories and surfaces
     as possibly important for auras in magical systems.

     Links: area Processes and rules · draws on 422 · summary A modifier declares its scope among bearer, body, relation, group, place and site; board adjacency belongs to the game's ruleset.

434. **The player's choice of which glyphs far marks show is a filter.**
     Asked on 2026-10-01: "What do we call the player's choice of which
     glyphs the far marks show? Evidence: "lens" already names the wing's
     projection (the VTT's locked isometric lens, isometer-lens); "filter"
     is already the creator's word for a player's narrowing choice (the
     phenotype contract plan's generation filters), and elsewhere only
     filter feeding uses it." Options, recommended first: (A) "Filter: the
     creator's existing word for a player's narrowing choice; plain, and its
     other use, filter feeding, is a different sense (recommended)." (B)
     "Lens: keep 432's word, beside the projection lens and isometer-lens."
     Mark chose A, "Filter". So 432's choice is the player's filter, each
     mode supplying a default filter, and "lens" keeps one meaning, a
     projection.

     Links: area Naming · amends 432 · summary The player's choice of which glyphs far marks show is called a filter; lens keeps one meaning, a projection.

435. **A cohort keeps a tally of its members' condition causes beside its
     state.** Asked on 2026-10-01, a fork found writing the sim plan's
     §3.6: "Where is a crowd member's condition cause kept? Evidence: a
     Population cohort shares "the complete causal state" and restrict
     merges only equal entities, so a cause inside the record would split
     cohorts, which 430 forbids; an Event is written only for a process
     marked note, one per bulk application with the group's representative
     as subject; how a thing learned is already sampled backward on demand
     (86, 117)." Options, recommended first: (A) "A tally beside the
     cohort: each cohort keeps, outside its equality, a count of members per
     cause (the act: process, tick, place); merging adds tallies, and
     lifting draws a member's cause from them by a seeded draw, after which
     the lifted member keeps its own. Exact in counts, pointable for anyone
     lifted (recommended)." (B) "Drawn back on demand: a crowd keeps no
     causes; lifting draws a plausible cause from the history, as how a
     thing learned is sampled backward (86, 117). Exact only for
     individuals." Mark chose A, "A tally beside the cohort". So a cause is
     the act that applied a condition, named by process, tick and place
     whether or not the act was noted, and a cohort's tally counts its
     members by cause outside the equality `restrict` compares. Merging adds
     tallies, and a lift draws the lifted member's cause, seeded, from them,
     so the counts stay exact and every lifted member's cause is pointable.
     *Reading, not ruled:* the members left on either side of a lift divide
     the rest of the tally by a seeded draw, since no member has an identity
     until looked at (113).

     Links: area Processes and rules · draws on 430 · summary A cohort keeps a tally of members per condition cause outside its equality; merging adds tallies and lifting draws a cause by seed.

436. **A second application of a key combines into one record by the key's
     stacking.** Asked on 2026-10-01, a fork found writing §3.6: "How does a
     second application of a key combine with the first, a spider's poison
     on a thing already poisoned? Evidence: 5e's conditions do not stack and
     the longer duration applies; Elder Scrolls stacks one effect from
     different spells but refreshes a recast of the same; Pathfinder 2e's
     valued conditions take the higher value; 429 already declares stacking
     per reading." Options, recommended first: (A) "One record, the key's
     stacking: a key declares its stacking as a reading does (429): the
     magnitudes combine by it (sum, highest, replace…) and the end becomes
     the later of the two; both causes are kept (recommended)." (B) "A
     record per application: each application keeps its own magnitude and
     end, and readings stack them when read; a thing poisoned ten times
     holds ten records, and a crowd splits by each." (C) "Refresh: a
     reapplication replaces the magnitude and the end, as many games refresh
     a debuff." Mark chose A, "One record, the key's stacking". So a thing
     holds at most one record per key: each key declares its stacking from
     429's list, a second application combines magnitudes by it, the end
     becomes the later of the two, and both causes stay counted (435).
     *Reading, not ruled:* a key declaring replace takes the new
     application's end with its magnitude, which is the refresh of option C
     available to a world that wants it.

     Links: area Processes and rules · draws on 429, 435 · summary A thing holds at most one record per key; a second application combines magnitudes by the key's stacking and the end becomes the later.

437. **A minimap or overmap is a second camera on the ladder.** Asked on
     2026-10-01, a fork found writing the presentation plan's lane L10:
     "What becomes of the VTT's overmap and Mesocosm's minimap? Evidence:
     423 put the world map in the one camera as R4, over the option of
     keeping it "a separate document view, as the overmap and minimap are
     today"; the overmap draws on sprigging's canvas and the minimap as
     hulls on the HUD lane; §9.2 counted both among the four near-to-far
     renderers." Options, recommended first: (A) "Second cameras on the
     ladder: the main camera reaches R4 by zooming out; where a game wants a
     minimap or overmap, it is a second camera on the same ladder, held at
     R4, and the separate canvas and HUD renderers retire (recommended)."
     (B) "Retire both: one camera only; zooming out is the only map." (C)
     "Keep them for now: both stay as they are until L10 lands, then come
     back as a fork." Mark chose A, "Second cameras on the ladder". So the
     four near-to-far renderers become one ladder drawn through as many
     cameras as a game wants, and the VTT's overmap on sprigging's canvas
     and Mesocosm's minimap hulls retire once a second camera held at R4
     replaces each.

     Links: area Presentation · draws on 423 · summary Minimap and overmap become second cameras on the ladder held at the world-map rung; the separate canvas and HUD renderers retire.

438. **A far mark draws a glyph's display as a pixel icon.** Asked on
     2026-10-01, a fork found writing L10: "How does a far mark draw a
     glyph's display? Evidence: wing-glyphs keeps a display as opaque
     Unicode; isometer's glyph batch draws a bounded set of punctuation
     strokes with the scene's depth, "a presentation vocabulary and nothing
     more"; the hosts are genet documents, which already lay out text; the
     look is pixel art at integer scale (387)." Options, recommended first:
     (A) "Pixel icons from the display: each glyph's display is rasterised
     once at mark size into a pixel atlas, nearest-neighbour at integer
     scale, and isometer draws marks from it with the scene's depth; a canon
     revision changes which glyph an effect shows, never the atlas
     (recommended)." (B) "Genet text over the scene: marks are document
     elements the host positions by the scene's projection, its text and CSS
     drawing each display, as the VTT's DOM board draws tiles; no depth
     shared with the scene." (C) "Strokes per glyph: the world gives each
     glyph a stroke shape from isometer's vocabulary, drawn by the existing
     batch; no text at all, and the vocabulary grows with every glyph."
     Mark chose A, "Pixel icons from the display". So each glyph's display
     is rasterised once into an atlas of pixel icons at mark size, and
     isometer draws far marks from that atlas at integer scale with the
     scene's depth. The existing glyph batch keeps drawing glyph effects'
     strokes. *Reading, not ruled:* the rasteriser is the stack's text path
     (netrender's), run once per glyph and cached, so no font draws at frame
     time.

     Links: area Presentation · draws on 432, 387 · summary Each glyph's display is rasterised once into a pixel-icon atlas; far marks draw from it at integer scale with the scene's depth.

439. **Grouping stops paying where members per state fall below a world
     rule.** Asked on 2026-10-01, opening time at scale after a remeasure of
     the 09-25 scale points on today's core (`SCALE_REMEASURE.md`): "How is
     "where grouping stops paying" (266) measured? Evidence: today at 4,096
     members, members per group fall from 10.7 to 5.7 in eight ticks; the
     crowd's savings fell from 11 to 2.4 times once strain was per-member
     state (261); a century in 300 s buys about 23 million evaluations at
     today's 13 µs each, and one daily process over 10,000 members at nine a
     group needs about 41 million." Options, recommended first: (A)
     "Members per state, a world rule: a site and lineage moves to its rate
     model when its members per distinct state fall below a bound the
     world's rules set, measured at each of its periods, and moves back when
     they rise; every switch is receipted (recommended)." (B) "When the
     budget needs it: deep time spends a work budget in evaluations,
     machine-independent (284), moving the site-lineages whose grouping
     pays least to rate models first until the century fits; amends 266's
     line from grouping to budget." (C) "Declared per process: the rules
     mark which processes run as rate models in deep time; everything else
     stays exact." Mark chose A, "Members per state, a world rule". So 266's
     line is measured per site and lineage at each of its periods, as
     members per distinct state against a bound in the world's rules; below
     it the site and lineage runs its rate model, above it the exact crowd,
     and every switch either way is receipted. The century's time is not
     guaranteed by this; the budget stays a guard on work (284).

     Links: area Processes and rules · draws on 266 · summary A site-lineage moves to its rate model when members per distinct state fall below a world-rule bound; every switch is receipted.

440. **A rate model is calibrated on the bench, once per rules digest.**
     Asked on 2026-10-01: "Where is a rate model calibrated against the
     exact runner? Evidence: 261 says on the bench; 207's crowd was
     certified on 1,000 draws for one rules revision; a world's rules are
     content-addressed (§3.1); a world's own sites, lineages and scarcity
     differ from any bench draw's." Options, recommended first: (A) "Bench,
     per rules digest: rates are fitted once per rules digest over many
     draws, certified against the exact runner as 207's crowd was, and
     shipped with the rules; a world never runs an uncalibrated model
     (recommended)." (B) "Bench, then each world: bench-fitted rates
     refined by a short exact window in each world's deep time before it
     leaps, the refinement receipted." (C) "Each world only: each world's
     deep time runs an exact window per site and lineage and fits its own
     rates." Mark chose A, "Bench, per rules digest". So a rules digest
     carries its rate models' fitted rates and their certificate, and a
     world under rules without a certified model keeps every site and
     lineage on the exact crowd, whatever 439's bound says.

     Links: area Processes and rules · draws on 261 · summary Rate models are calibrated on the bench once per rules digest and ship with the rules; uncalibrated worlds stay on the exact crowd.

442. **The all-modes build lives in a host workspace of its own.** Asked on
     2026-10-01, opening the mode host after its assessment found that no
     game runs over the sim yet: "Where does the all-modes build live?
     Evidence: the repository keeps three Cargo workspaces, the VTT's at the
     root and Mesocosm's and Eponym's below, separate "to preserve
     source/patch policy", and the root's .cargo/config.toml is inherited,
     so product patches cannot go there; vello and taffy are patched alike
     in all three, the VTT adds genet's DOM crates, ipc-channel and
     muniment, and Eponym adds rust-gpu's spirv-std and two path patches to
     Code/crates/crabslab, outside the repository, for renderling, which
     eponym-client still uses; 385 keeps "no mode may require another
     mode's code to run"." Options, recommended first: (A) "Its own host
     workspace: a host workspace holding only the mode host's binary,
     depending on the three products' crates by path under the union of
     their patches, renderling's retired first; each product's workspace
     keeps building its one-mode profile (recommended)." (B) "One workspace
     for all: the three merge into one, one lockfile and one patch table;
     simplest to build, ending the separation the root CLAUDE.md keeps."
     (C) "The root workspace hosts: the VTT's root workspace takes in the
     host binary and the other products' crates." Mark chose A, "Its own
     host workspace". So a fourth workspace holds the mode host's binary
     alone and takes the three products' crates by path; the product
     workspaces keep their separation and their one-mode builds; and
     Eponym's renderling, with its patches to paths outside the repository,
     retires before Eponym's mode can join the host's build.

     Links: area Stack and engine · draws on 381 · summary The all-modes build lives in a fourth host workspace holding only the mode host binary, depending on the products' crates by path.

443. **A window plays one mode and switches in place.** Asked on
     2026-10-01: "How does a host switch modes? Evidence: isomere's
     Assembly runs one Product in one window over cambium's winit host; 381
     lets players in different modes share a trunk across peers; the isomere
     plan's 2026-09-28 refinement, not ruled, keeps the world, subject and
     embodiment across a mode change unless a game action changes them; each
     mode brings its own controls, view defaults and attention set (212,
     213)." Options, recommended first: (A) "In place, a mode per window: a
     window plays one mode over the open world and switches in place,
     swapping controls, view and attention set without reloading the world,
     subject and embodiment kept; a second window can play another mode over
     the same world (recommended)." (B) "Modes side by side in panes: one
     window shows several modes at once, each pane with its own mode and
     attention set." (C) "One mode per launch: switching saves and
     relaunches into the other mode." Mark chose A, "In place, a mode per
     window". So a mode switch keeps the open world, the subject and its
     embodiment, and swaps the window's controls, view defaults and
     attention set; the isomere plan's refinement on what a switch keeps is
     ruled by this. Two modes over one world on one machine are two
     windows.

     Links: area Host and network · draws on 212, 213, 381 · summary A window plays one mode and switches in place, keeping world, subject and embodiment; a second window can play another mode.

444. **The world save holds the sim's history and a section per mode.**
     Asked on 2026-10-01: "What does the one world save hold? Evidence: the
     sim saves its genesis, history and checkpoints (history.rs's Saved);
     the VTT bundles maps, tilesets, sheets and system choice as a campaign
     pack; Mesocosm keeps epochs and the shop; branches and merges run over
     the sim's log (104); under 385 a build without a mode must still open
     the save." Options, recommended first: (A) "World plus a section per
     mode: one save holds the sim's history and a section per mode for what
     is that mode's alone (a campaign's maps, a lineage's shop), each
     section opaque to builds without its mode and kept intact by them
     (recommended)." (B) "The world, mode files beside it: the save is the
     sim's alone; each mode keeps its own file naming the world by digest."
     (C) "Everything as the sim's facts: mode state becomes asserted facts
     in the sim's record, branching and merging with it." Mark chose A,
     "World plus a section per mode". So one file carries the sim's history
     and, per mode, the state that is that mode's alone; a build without a
     mode carries its section through unread and unchanged, which is 385's
     guard applied to saves. *Reading, not ruled:* how a mode's section
     branches and merges with the trunk (104) is the mode's own, the sim's
     history merging as ruled.

     Links: area Record and history · draws on 104, 385 · summary The world save holds the sim's history plus an opaque section per mode, which builds without that mode carry through unchanged.

445. **What a window may know follows its seat and subject, never its
     mode.** Asked on 2026-10-01, the mode host's last open item, the
     shared presentation and knowledge contract: "What decides what a
     window may know? Evidence: Mesocosm's survival shows what the critter
     knows and creative the truth, seeing only (180, 184); a DM edits and
     may take up any unclaimed entity (156); divine figures may see behind
     the curtain as magic (384); under replay every peer runs the whole sim
     (212, 213), so access shapes play, not secrecy; a mode switch keeps the
     subject and embodiment (443); the overlay contract has an AttentionSet
     and a ViewHandle but no access." Options, recommended first: (A) "Seat
     and subject, never the mode: a window's access comes from its
     participant's seat (player, creative observer, DM) and the entity
     played, plus awareness granted in play (384); switching modes changes
     how it is shown, never what may be known, and every presentation path
     reads through it (recommended)." (B) "Each mode sets its own: survival
     or creative in Mesocosm, player or DM in the VTT, as each game decides;
     a switch may change what is known." (C) "Truth everywhere, filters
     optional: every window may see the truth, and each mode offers a
     knowledge filter the player can turn on." Mark chose A, "Seat and
     subject, never the mode". So a window's knowledge access is computed
     from three things: its participant's seat, the entity it plays, and any
     awareness granted in play, such as 384's divine sight. Readings, far
     marks (432), examiners and the event stream all read through it, and a
     mode switch changes only how the permitted is shown. *Reading, not
     ruled:* Mesocosm's survival and creative (180) read as seats, a player
     playing a critter and a creative observer seeing the truth without
     editing (184), and the VTT's DM is the seat that edits (156); the access
     type belongs in `isocosm-overlay` beside the attention set, since every
     mode shares it; and it shapes play only, every peer holding the whole
     world under replay.

     Links: area Directing and play · draws on 180, 184, 156, 384, 443 · summary What a window may know follows its participant's seat, the entity played and awareness granted in play, never its mode.

446. **A body keeps its lineage's matter as tissue and reserve.** Asked on
     2026-10-01, opening checkpoint 6 with the question the checkpoint 5
     review saved for its minimal-body probe: "Does a body keep its fuel
     apart from its tissue? Evidence: Mesocosm keeps a reserve beside each
     body's substance: a meal burns into the reserve when the body is within
     STARVED_UPKEEP_TICKS of empty and builds tissue otherwise (TD5, one rule
     for every kingdom), upkeep and rent are paid from the reserve, ceilings
     bound both at the adult mass, and what a body cannot hold spills back to
     the ground (TD6); Isocosm holds one matter account per lineage,
     digestion yielding the eater's own (273, 342); the checkpoint 5 review
     saved this for you." Options, recommended first: (A) "Two own-lineage
     accounts: each body keeps its lineage's matter as tissue and reserve: a
     meal burns into reserve near starving and builds tissue otherwise;
     upkeep and rent draw reserve first, then tissue; ceilings bound both,
     and what a body cannot hold spills to the ground (recommended)." (B)
     "One account, reserve as a reading: a body keeps one matter account,
     and the reserve is the share above a structural minimum, read by rule;
     no separate ceiling or spill." (C) "Tissue only: no reserve: upkeep and
     rent draw tissue directly, as Isocosm does now." Mark chose A, "Two
     own-lineage accounts". So Mesocosm's TD5 and TD6 physiology moves into
     Isocosm as two accounts of the body's own lineage's matter, leaving
     own-lineage digestion (273, 342) as ruled: a meal's routing between
     them is one rule for every kingdom, upkeep and rent draw the reserve
     before the tissue, ceilings bound both, and what a body cannot hold
     spills to the ground. *Reading, not ruled:* a spill is a mineralization
     (342), living matter returning to the world's; the starvation margin,
     the ceilings and the routing threshold are world rules with Mesocosm's
     values as defaults; and the probe's minimal body carries both accounts.
     *Reading corrected 2026-10-01, building step 5:* a spill is not itself a
     mineralization. Mesocosm returns rent to the ground as mineral at once
     (`complete_return`) but leaves a spill there as the spiller's typed
     matter (`deposit_stock`), which the soil's mineralization rate turns to
     mineral later (`soil::mineralize`). So the probe returns rent as soil at
     once and lands a spill on its site as the spiller's matter, where step
     4's mineralization rate, an agentless site process, returns it.
     *Located 2026-10-02 by ruling 459:* tissue in each part, the reserve in
     the parts that store.
     *Bounded and spread 2026-10-02 by rulings 463 and 464:* a store holds
     what its cells can and a body without one keeps none, and rent, growth
     and a landing meal reach the parts in proportion.

     Links: area Bodies and anatomy · draws on 273, 342 · summary A body keeps its lineage's matter as two accounts, tissue and reserve; one meal-routing rule for every kingdom, reserve drawn first, overflow spills to ground. (amended by 463, 464, 523)

447. **Reproduction is several strategies, governed by traits.** Asked on
     2026-10-01, with 446, filial cost being the other physiology the
     checkpoint 5 review left out: "What does a birth take from the parent,
     and what body does the child start with? Evidence: Mesocosm grows the
     child from the lineage's recipe at a quarter of the parent's biomass
     (OFFSPRING_COST), paid from the parent's substance, with a reserve
     endowment from its reserve as far as it goes, "a birth is a transfer,
     not a spawn" (TD6); Isocosm's Birth clones the parent's whole body and
     debits a provision from its accounts; plasticity is a life stage youth
     pays for (the epoch boundary plan)." Options, recommended first: (A) "A
     filial body, world-rule cost: the child is grown from its lineage's
     recipe at a fraction of the parent's body set by the world's rules,
     Mesocosm's quarter the default, paid from the parent's tissue, with a
     reserve endowment from its reserve as far as it goes; a birth is a
     transfer, never a spawn (recommended)." (B) "A clone with a provision:
     as Isocosm does now: the child copies the parent's body and receives a
     provision debited from the parent." (C) "A seed, grown by feeding: the
     child starts as a minimal body paid from the reserve alone and grows by
     its own feeding, the cheap many-offspring strategy." Mark answered:
     "There should probably be a few reproductive strategies governed by
     traits… so 1 and 3 and more could be options". So a lineage reproduces
     by strategies its traits govern, the filial body and the seed among
     them; which strategies form the default set, what decides a body's, and
     which further axes reproduction carries were put back the same day.
     *Reading, not ruled:* every strategy is a transfer and never a spawn, as
     TD6 and the sim's conservation of matter require.
     *Given its recipe 2026-10-02 by ruling 468:* a lineage carries a recipe
     and a placement policy, a child develops its part tree from them, and a
     maturing body grows the parts its recipe names that it lacks.
     *Amended 2026-10-03 by ruling 518:* a birth spends the provision its
     parent's reproduce cells fill, not a fraction of the parent's body,
     and comes when that provision is full.

     Links: area Bodies and anatomy · summary Reproduction is several strategies governed by lineage traits; every birth is a transfer of matter, never a spawn. (amended by 518)

448. **The default set holds four reproductive strategies.** Asked on
     2026-10-01, putting 447 back: "Which reproductive strategies does the
     default set hold? Evidence: the forms-of-life brief's reproduction axis
     (§F) has brood, the only one built, plus budding (a body at its ceiling
     routing overflow into a bud, conservation-neutral), spores
     (reproduction without locomotion, needing a bounded scatter) and
     horizontal transfer; germs are many bodies that burn generations
     spreading (58); you named the filial body and the seed. I'd take all
     four below." Options, several allowed: "Brood, a filial body: the child
     grown from the recipe at a fraction of the parent set by world rule,
     paid from its tissue, with a reserve endowment; Mesocosm's today."
     "Seed or egg: a minimal propagule paid from the reserve, growing by its
     own feeding; many offspring, most lost." "Budding and fission: a body at
     its ceiling routes its overflow into a bud, or divides, the child a
     share of the parent; how germs spread as many bodies (58)." "Spores:
     many tiny propagules scattered beyond the parent's reach, most lost,
     under a bounded scatter rule." Mark chose all four. So the default set
     is brood, seed or egg, budding and fission, and spores, each paid as
     described. *Reading, not ruled:* fractions, propagule sizes and scatter
     bounds are world rules with Mesocosm's values as defaults where it has
     them, and spores' scatter waits for the places family.
     *Joined 2026-10-02 by ruling 470:* fragmentation, a severed part living
     on as a new body where its lineage can regrow from a fragment.
     *Paid 2026-10-03 by ruling 518:* every strategy's birth spends the
     provision its parent's reproduce cells fill.
     *Specified 2026-10-03 by ruling 524:* a bud grows as a part at its
     parent's reproducing part and severs into a body of its own once full.
     *Specified 2026-10-03 by ruling 530:* how many eggs a birth lays is a
     lineage trait.

     Links: area Bodies and anatomy · draws on 447 · summary The default reproductive set holds four strategies: brood, seed or egg, budding and fission, and spores.

449. **A lineage's traits name its strategies, the body must support the one
     used, and the phenotype switches by circumstance.** Asked on
     2026-10-01, with 448: "What decides how a body reproduces? Evidence:
     ruling 36's phenotype expresses abilities by circumstance, condition,
     status and activity, and the record names an aphid growing wings and a
     locust turning gregarious as its polyphenism; 277 puts a reproductive
     organ system among the body's systems, assembled from functions; 405
     has construction declare function; a lineage's genotype changes only at
     the epoch boundary (57)." Options, recommended first: (A) "A trait the
     body must support: the lineage's traits name its strategies, the body's
     reproductive functions must express the one used, and the phenotype may
     switch among them by circumstance, as aphids switch with the season
     (36) (recommended)." (B) "One per lineage, fixed: a lineage reproduces
     one way, changed only at the epoch boundary (57)." (C) "The body alone:
     whatever reproductive functions a body expresses decide it, with no
     lineage trait." Mark chose A, "A trait the body must support". So
     which strategies a lineage may use is genotype, changing at the epoch
     boundary (57); which one a body uses is phenotype, switching by
     circumstance; and a body uses a strategy only when its reproductive
     functions express it (405, 277).

     Links: area Bodies and anatomy · draws on 447, 36 · summary A lineage's traits name its reproductive strategies; the body must support the one used; phenotype switches by circumstance.

450. **Reproduction also carries mating, semelparity or iteroparity,
     parental care and horizontal transfer.** Asked on 2026-10-01, with 448:
     "Which further axes does reproduction carry now? Evidence: Mesocosm
     breeds from one parent today; 57 rerolls expression at reproduction by
     the heterogeneity of the genes; the brief notes the engine is
     iteroparous by construction and semelparity "cheap and dramatic";
     horizontal transfer conflicts with the lexicon rule unless a symbiont's
     edge counts as eating (§F)." Options, several allowed: "Mating: two
     parents' genotypes combine, needing a partner, the reroll spreading by
     both parents' heterogeneity." "Semelparity or iteroparity: one brood
     then death, or many, as a trait." "Parental care: provisioning after
     birth, a feeding transfer from parent to young." "Horizontal transfer:
     acquiring a trait from something not eaten, a symbiont's edge counted
     as eating." Mark chose all four. So reproduction may mix two parents'
     genotypes, 57's reroll spreading by both parents' heterogeneity; a
     lineage may breed once and die or breed many times, by trait; a parent
     may provision its young after birth by a feeding transfer; and a trait
     may cross from a symbiont, whose edge counts as eating, so the lexicon
     rule stands as the brief's §F proposed.
     *Placed 2026-10-03 by ruling 521:* semelparity or iteroparity and
     parental care are built in checkpoint 8, mating and horizontal transfer
     with the boundary family.
     *Specified 2026-10-03 by rulings 526 and 527:* care feeds a hungry
     child, while the parent is not hungry, milk from the parent's
     refilling provision or its own mouthfuls from its reserve, as the
     lineage's trait says.

     Links: area Bodies and anatomy · draws on 447 · summary Reproduction also carries mating, semelparity or iteroparity, parental care and horizontal transfer, each governed by traits.

451. **An epoch ends by one of three rules, a year by default.** Asked on
     2026-10-01, opening checkpoint 6's step 4: "What ends an epoch in
     Isocosm, and how long is one by default? Evidence: Isocosm's epoch is a
     checkpoint boundary every epoch_ticks world ticks, 32 minutes in the
     generator, a test value; Mesocosm's rule has three kinds, Timed, Gated
     (named, unbuilt) and PlayerTriggered (a dev door), its default 1,000
     play ticks at ten a second, a third of a reference starter's 3,000-tick
     life; ruling 256 runs lineages yearly; 57 changes a genotype only at the
     epoch boundary." Options, recommended first: (A) "Three kinds, a year
     by default: the rule carries Timed, Gated and PlayerTriggered; Timed
     counts the world's unit and defaults to a year, the lineages' period
     (256), so the boundary where every lineage adapts (57) falls with their
     yearly process; checkpoints keep falling on boundaries (recommended)."
     (B) "Three kinds, a share of a lifespan: Timed defaults to a third of
     the world's reference lifespan, Mesocosm's own rationale, read from its
     age processes." (C) "Timed only, the founder's choice: epoch_ticks stays
     the only kind, with no default beyond what the founder sets or the
     generator draws." Mark chose A, "Three kinds, a year by default". So
     Mesocosm's three rules move into Isocosm's: Timed counts the world's
     unit, a year by default, which is 525,960 ticks at the default minute;
     Gated stays named and unbuilt; PlayerTriggered ends an epoch only on
     demand. The boundary where every lineage adapts (57) falls with the
     lineages' yearly process (256), and checkpoints fall on boundaries.

     Links: area Record and history · draws on 256 · summary An epoch ends by Timed, Gated or PlayerTriggered rule; Timed counts the world's unit and defaults to a year.

452. **Deep time's span counts epochs, as Mesocosm counts it.** Asked on
     2026-10-01, with 451: "How is deep time's span counted? Evidence:
     Mesocosm counts it in epochs (DeepTimeSpan, a world rule in the digest,
     isoscape rulings 5 and 16), the hagiograph taking DeepTime { epochs };
     ruling 124 reads the target as a century of world time; a Gated or
     PlayerTriggered epoch has no length to convert by." Options,
     recommended first: (A) "World time, epochs counted: the span is a
     duration in the world's unit, a century for 124's region; the epochs it
     holds are counted as they close, and the hagiograph is handed that
     count (recommended)." (B) "Epochs, as Mesocosm: the span counts epochs,
     its world time following the epoch rule." (C) "Both, checked: the
     founder states a duration and an epoch count, refused when they
     disagree." Mark chose B, "Epochs, as Mesocosm", over the
     recommendation. So deep time's span is a count of epochs in the world's
     rules and digest, its world time following the epoch rule: under 451's
     default year, 124's century is a span of 100 epochs. *Reading, not
     ruled:* Mesocosm's two refusals carry over with it: a span under a rule
     that never closes an epoch on its own, Gated until it is built or
     PlayerTriggered, is refused before any tick, and a run that stops
     closing epochs within one epoch more than its span is refused as
     stalled.

     Links: area Record and history · draws on 451 · summary Deep time's span counts epochs in the world's rules, as Mesocosm does; under the default year a century is 100 epochs.

454. **Shared ground is shared out from the pass's start.** Asked on
     2026-10-01, building checkpoint 6's step 5, ruling 306 having moved
     hunting's shortage to 269's scramble in Part B: "How does ruling 269's
     scramble share out ground that runs short? Evidence: today the core
     takes in identity order, lowest identities first, and the crowd refuses
     a site debit that covers only some members; 4b counted 6 hunting
     shortfalls in 1,000 draws with food and 3 with water. Mesocosm's meals
     and roots are first-come in organism order. A computed mouthful (TD9)
     makes hunters' order matter even with no shortage, so the crowd can't
     group hunters whose mouthfuls differ while meals are taken one after
     another." Options, recommended first: (A) "Shares from the pass's
     start: each pass reads the world as it stood when it began. Where a
     site or a prey holds less than all the takes on it, every taker gets
     the same fraction of its take, floored, and the remainder stays where
     it was. Hunters draw their prey from the pass's start too. Order no
     longer matters, so the crowd groups it, computed mouthfuls included.
     Ruling 287's hunts change behaviour, so 4b's two checks run again
     (recommended)." (B) "A seeded random order: each pass visits its
     reactive members in an order drawn from the dynamics seed, and whoever
     comes after the supply runs out goes without. A smaller change to the
     core, but a lottery rather than a proportional share. The crowd groups
     mixed mouthfuls only by drawing the order by count." (C) "Shares for
     site accounts only: fixing's soil is shared in proportion. Hunting
     keeps one meal at a time with fixed bites, and identity order under
     shortage. Computed mouthfuls stay with individuals, listed as waiting
     for hunting's own scramble." Mark chose A, "Shares from the pass's
     start". So each periodic pass reads the world as it stood when the pass
     began. A site account or a prey holding less than the takes on it gives
     each taker the same fraction of its take, floored, and the remainder
     stays put. Hunters draw their prey against the pass's start, so two may
     draw one prey and share it. Identity order decides no outcome of a
     pass, the crowd groups computed takes by state, and 287's hunts are
     re-baselined, 4b's two checks running again before the checkpoint
     merges. *Reading, not ruled:* the competitions of rulings 115, 236 and
     240 keep their rounds against the tick's start, being how what can
     decide competes (269).

     Links: area Processes and rules · draws on 269 · summary Shared ground is shared out from the pass's start; short supply gives each taker the same floored fraction; identity order decides nothing.

455. **A body's ceiling is read part by part.** Asked on 2026-10-01, with
     454: "How is a body's ceiling read? Evidence: Mesocosm prices each part
     at its voxels × 100/125, floored and at least 1 mg, then sums the
     parts. For a lump, two limbs and an eye from the primitive palette that
     gives 249 mg, against 251 mg if read over the whole body at once. The
     ceiling caps both accounts, prices upkeep and the mouthful, and decides
     where a meal goes." Options, recommended first: (A) "Part by part,
     exactly: one new expression node sums a formula over the living parts,
     with each part bound in turn. The per-part floor and the 1 mg minimum
     match Mesocosm to the milligram (recommended)." (B) "Whole body at
     once: use the existing voxel reading times 100/125, floored once. No
     new node, but it can differ from Mesocosm by up to about 1 mg per
     part." Mark chose A, "Part by part, exactly". So X3 gains a node
     summing an expression over a body's living parts, each bound in turn as
     the part, and the probe's ceilings match Mesocosm's to the milligram,
     the reference mass and segment entering as the expression's constants.

     Links: area Bodies and anatomy · summary A body's ceiling is read part by part, summing a formula over living parts, matching Mesocosm to the milligram.

459. **Tissue sits in parts, and the reserve in parts that store.** Asked
     on 2026-10-02, opening the anatomy brief (458): "Where does a body's
     matter sit? Evidence: Mesocosm keeps tissue part by part, each part
     with its own mass and typed mix; rent and bites take from the parts
     root first; incorporation takes a whole part; the reserve is one
     number on the organism. isometer's part carries its own mass too. The
     probe kept tissue and reserve as body accounts, and no native read
     which part held what. Ruling 277 lists 'store' among the functions."
     Options, recommended first: (A) "Tissue in parts, reserve in stores:
     each part holds its own tissue, and parts expressing 'store' hold the
     reserve (fat bodies, tubers). A bite lands on one part of the prey,
     drawn by what each holds. Incorporation takes a part whole, with its
     matter (recommended)." (B) "Tissue in parts, reserve body-wide:
     Mesocosm's model as it is: each part its own tissue, the reserve one
     account on the body. Bites and rent take from the parts in a fixed
     order." (C) "Body-wide, as the probe: two accounts on the body and
     nothing in the parts. Severing a part or incorporating one moves a
     share of the body's matter in proportion to the part's size." Mark
     chose A, "Tissue in parts, reserve in stores". So each part holds its
     own tissue and the parts that express store hold the body's reserve;
     a bite lands on one part of its prey, drawn by what each holds, and
     incorporation takes a part whole with its matter. This settles 456's
     item and locates 446's two accounts.
     *Bounded 2026-10-02 by ruling 463:* a store holds reserve up to its
     store cells times a cell's mass, and a body without one keeps none.
     *Spread by ruling 464:* rent, growth and a landing meal reach the parts
     in proportion.
     *Booked 2026-10-02 by ruling 504:* each part keeps its own ledger and
     is a holder; the body's totals are readings.

     Links: area Bodies and anatomy · draws on 446 · summary Each part holds its own tissue and parts expressing store hold the reserve; a bite lands on one part of the prey. (amended by 463)

460. **A part's cells are derived from its extents.** Asked on 2026-10-02,
     with 459: "How many cells does a part hold? Evidence: Mesocosm derives
     a part's cells from its extents: along each axis, one cell per two
     voxels of half-extent plus one, at most four per axis and 64 in all.
     Its palette plate holds nine, and each cell weighs the part's adult
     mass divided by its cells. Isocosm's part declares both numbers
     (ruling 453); the probe founded them by Mesocosm's rule." Options,
     recommended first: (A) "Derived from extents: capacity and cell mass
     are read from the part's extents by Mesocosm's rule. There's no field
     to keep and nothing to disagree with the geometry (recommended)." (B)
     "Declared per part: as ruling 453 has it: authored per part,
     independent of the part's size." (C) "Derived, declarable: Mesocosm's
     rule by default, with an author's value used wherever one is given."
     Mark chose A, "Derived from extents". So a part's capacity and cell
     mass are readings of its extents by Mesocosm's rule, and 453's two
     declared fields give way to them.
     *Wounded 2026-10-02 by ruling 469:* a part keeps the cells a wound has
     lost, so its living cells are its derived capacity less them.

     Links: area Bodies and anatomy · draws on 459 · summary A part's capacity and cell mass are derived from its extents by Mesocosm's rule; the declared fields give way.

461. **The catalogue takes all fifteen functions, the other vocabularies
     folded in.** Asked on 2026-10-02, with 459: "How far does the function
     catalogue widen in this brief? Evidence: ruling 339 started it with the
     five in use, the rest to come with this brief. Ruling 277 named
     fifteen. Ruling 278 joins four vocabularies: Mesocosm's five;
     wing-functions' charge routing (source, store, gate, actuator);
     Eponym's grip and adhesion, each declaring a reach and a load; and the
     contact probe's impairments of grip and reach. The probe's mechanics
     read only the five." Options, recommended first: (A) "All fifteen,
     others folded in: one catalogue now. Each function lists its shapes,
     its seeding and the mechanic that reads it, or none yet.
     wing-functions' kinds and Eponym's grip and adhesion map onto it
     (recommended)." (B) "Only what a mechanic reads: the five now. Each
     other function enters with the first mechanic that reads it, as each
     game's family moves into the sim." (C) "The fifteen, folding later:
     the fifteen now; the other vocabularies keep their own until their
     consumers move into the sim." Mark chose A, "All fifteen, others
     folded in". So the catalogue holds 277's fifteen, each with its
     shapes, seeding and reader, and the other three vocabularies map onto
     it.
     *Tabled 2026-10-02 by ruling 466:* each function's shapes, seeding and
     reader, with wing-functions' source and store onto store, its gate onto
     gate and its actuator onto contract.

     Links: area Bodies and anatomy · amends 339 · summary The function catalogue holds all fifteen functions, with the other vocabularies folded into it.

462. **The sim keeps where each part attaches.** Asked on 2026-10-02, with
     459: "Who measures a body's geometry for the sim? Evidence: Isocosm's
     part keeps only its parent and extents, and the sim depends on no
     isometer crate. isometer's part keeps a pivot, an attachment offset
     and a yaw; drawing reads them, and so does Mesocosm's reach (the
     farthest contracting part's distance from the root, plus its extent).
     Ruling 405: geometry measures only what a mechanic needs, and each
     such reading names what it depends on. Eponym declares reach and load
     per part instead of measuring." Options, recommended first: (A) "The
     sim keeps attachment offsets: each part records where it attaches to
     its parent, so the sim measures reach, clearance and contact itself;
     each mechanic names the measurement it reads. Pivots, yaw and volumes
     stay with isometer, for drawing (recommended)." (B) "Declared
     capabilities: parts declare what a mechanic needs, such as reach and
     load, as Eponym does. Nothing is measured in the sim." (C) "isometer
     measures: the sim asks isometer's body document for measurements,
     taking a dependency on the geometry family." Mark chose A, "The sim
     keeps attachment offsets". So each part records where it attaches to
     its parent, the sim measures reach, clearance and contact from that,
     each mechanic naming the measurement it reads (405), and pivots, yaw
     and volumes stay isometer's.
     *Brought forward 2026-10-03 by ruling 510:* the offsets are built in
     checkpoint 8, where growth and incorporation place parts by them.

     Links: area Bodies and anatomy · draws on 405 · summary Each part records where it attaches to its parent so the sim measures reach, clearance and contact; pivots and yaw stay isometer's.

463. **A store holds the reserve its cells can, and a body without one keeps
     none.** Asked on 2026-10-02, the anatomy brief's second round: "How
     much can a body's stores hold, and what of a body with none? Evidence:
     Mesocosm bounds the reserve at the body's adult mass, the same bound as
     its tissue (TD6), and every body has a reserve. Under 459 the reserve
     sits in parts that express store, and neither body the probe drew
     (fronds; a lump, limbs and an eye) expresses store. TD5 burns a meal
     into the reserve when the body is near starving, which needs somewhere
     to put it." Options, recommended first: (A) "Store cells hold it: each
     store part holds up to its store cells times one cell's mass. A body
     with no store keeps no reserve: TD5 then builds tissue and rent draws
     on tissue. Storing becomes a real strategy, fat bodies against lean
     ones (recommended)." (B) "Adult mass, held in stores: the reserve's
     bound stays the whole body's adult mass, as in Mesocosm, spread across
     its store parts. A body with no store keeps none." (C) "Every body
     keeps a reserve: stores hold up to their cells; a body with no store
     keeps a small reserve in its root, so TD5 always has somewhere to burn
     into." Mark chose A, "Store cells hold it". So a store part holds
     reserve up to its store cells times a cell's mass, and a body without
     one keeps none: TD5 then builds tissue, and rent draws tissue.
     *Founded 2026-10-02 by ruling 506:* the probe's grazers store in their
     lumps, and its producers stay lean.

     Links: area Bodies and anatomy · draws on 459 · summary A store holds reserve up to its store cells times a cell's mass; a body without a store keeps no reserve.

464. **Rent, growth and a landing meal reach a body's parts in proportion.**
     Asked on 2026-10-02, with 463: "Which parts do rent, growth and a
     landing meal reach, now that every part holds its own tissue? Evidence:
     Mesocosm takes rent and bites from its living parts in part order, root
     first, each part giving its own mix, and it grows by thickening the
     root up to the body's room. Each part has its own adult mass. Ruling
     454 made identity order decide nothing within a pass." Options,
     recommended first: (A) "In proportion: rent is drawn from the parts in
     proportion to their tissue (287's share rule). Growth and a landing
     meal fill each part in proportion to its room below its own adult mass.
     No part comes first, and none overfills (recommended)." (B) "Root
     first, as Mesocosm: rent from the root first, then the other parts in
     order; growth thickens the root up to the body's room, as Mesocosm does
     now." Mark chose A, "In proportion". So rent takes from the parts in
     proportion to their tissue, and growth and a landing meal fill each
     part in proportion to its room below its own adult mass. *Cite
     corrected 2026-10-02, on recording:* the option's "287's share rule" is
     ruling 294's proportional take, decided from 287's fork; 287 itself
     gave each consumer one feeding process.

     Links: area Bodies and anatomy · draws on 459, 294 · summary Rent draws from parts in proportion to tissue; growth and a landing meal fill each part in proportion to its room.

465. **Organ systems are named networks of functions.** Asked on 2026-10-02,
     with 463: "Are organ systems data in the sim? Evidence: ruling 277
     assembles systems from functions and has the generator riff new ones.
     Nothing reads a system yet. wing-functions already models one kind of
     system as a network: sources, stores, gates and effects on parts,
     joined by edges with capacities, which is what conduct and circulate
     would route along." Options, recommended first: (A) "Named networks of
     functions: a system is data in the world's rules: the functions it
     assembles and how the parts expressing them connect along the body's
     tree, with wing-functions' network as the model. The generator
     recombines systems, and a process may require a working one
     (recommended)." (B) "Named sets of functions: a system names its
     functions but not how they connect; it counts as present when the body
     expresses them all." (C) "The generator's grouping only: systems shape
     what the generator builds; the sim reads only parts and functions."
     Mark chose A, "Named networks of functions". So a system is data in the
     world's rules, naming the functions it assembles and how the parts
     expressing them connect along the body's tree, on wing-functions'
     network; the generator recombines systems, and a process may require a
     working one. What makes a system working is left to the brief's next
     round.
     *Settled 2026-10-02 by ruling 477:* a body's network is read from its
     tree, and a system works by degree, carrying what a process asks
     through the capacities.
     *Tabled 2026-10-02 by ruling 489:* the ten default systems, which the
     natives read from the bodies family on (490).

     Links: area Bodies and anatomy · draws on 277 · summary Organ systems are data in the world's rules: named networks of functions connected along the body's tree.

466. **The fifteen functions take the proposed shapes and seeding.** Asked
     on 2026-10-02, with 463: "Do the fifteen functions take the shapes and
     seeding proposed (see the preview), with wing-functions' source mapped
     onto store (a store that starts charged) and its actuator onto
     contract?" The preview, as put:

     | Function | Shapes | Seeding | Read by |
     | --- | --- | --- | --- |
     | contract | rod | Grown | rent, mouthful, reach |
     | intake | lump, tube | Grown | the meal |
     | sense | point | Grown | perception (waits for places) |
     | fix | sheet | Grown | a producer's income |
     | secrete | sheet | Acquired | rent, the dose |
     | support | rod, shell, joint, branch | Grown | none yet (load, clearance) |
     | conduct | tube, branch | Grown | wing-functions' edges |
     | gate | joint, tube | Grown | wing-functions' gate |
     | store | lump | Grown | the reserve (459) |
     | circulate | tube, lump | Grown | none yet |
     | respire | sheet, branch | Grown | none yet |
     | excrete | tube | Grown | none yet |
     | reproduce | lump | Grown | reproduction (447-450) |
     | grip | rod, branch, joint | Grown | Eponym's grip, contact |
     | adhesion | sheet, point | Grown | Eponym's adhesion |

     wing-functions: source -> store, store -> store, gate -> gate, actuator
     -> contract.

     Options, recommended first: (A) "The proposed table: each function
     admitted by the shapes listed, grown on them except secrete, which is
     acquired, as now. Store grows on lumps, so a lump splits its cells
     between taking in and storing (recommended)." (B) "Store acquired, not
     grown: the same table, but store is acquired, as secrete is: no body
     grows a reserve organ; a development has to give it one." (C) "Every
     shape admits every function: no shape requirements: any part can
     express any function, and the generator chooses. Simpler, but it loses
     ruling 405's tie between form and function." Mark chose A, "The
     proposed table". So each function is admitted by the shapes listed and
     grown on them, secrete alone acquired; a lump splits its cells between
     taking in and storing; and wing-functions' source and store map onto
     store, its gate onto gate and its actuator onto contract.
     *Amended 2026-10-02 by ruling 492:* the table's shapes are the fits the
     generator draws by default, not gates; any part may express any
     function, scaled by its measurements.
     *Measured 2026-10-02 by ruling 493:* each function's measurement is
     tabled, taken by its share of the cells.

     Links: area Bodies and anatomy · draws on 461 · summary The fifteen functions take the proposed shape table and seeding; secrete alone is acquired; wing-functions' kinds map onto store, gate and contract. (amended by 492)

467. **One part model serves every body form, a part attaching to a parent
     or lying on a site.** Asked on 2026-10-02, the anatomy brief's third
     round: "Do the spread and the terrain-body use the part tree's model?
     Evidence: the record's §3.2.1 gives three body forms that "share the
     ledger and the record but not the part model": a part tree (animal); a
     spread over a substrate (germ, fungus, colony); and a terrain-body
     (macro, a lion turtle whose volume carries places). Ruling 58 makes a
     fungus one body whose separate patches are its parts, and a germ many
     bodies. Isocosm's body sits on one site, and each of its parts names a
     parent." Options, recommended first: (A) "One model, placed two ways:
     every body's parts have shapes, functions, cells and tissue. A part
     either attaches to a parent (the tree) or lies on a site (a spread's
     patch), so a fungus is one body across many sites and a germ stays many
     bodies. A terrain-body is a part tree whose parts are big enough to
     carry places. One catalogue and one matter rule serve all three
     (recommended)." (B) "Part tree now, others later: rule only the part
     tree here. The spread and the terrain-body get their part model when
     the micro and macro families arrive." (C) "Three models, as §3.2.1: the
     forms share the ledger and the record, and each keeps its own part
     model." Mark chose A, "One model, placed two ways". So every body's
     parts have shapes, functions, cells and tissue, and a part attaches to
     a parent, making the tree, or lies on a site, making a spread's patch:
     a fungus is one body across many sites, a germ stays many bodies (58),
     and a terrain-body is a part tree whose parts are big enough to carry
     places. One catalogue and one matter rule serve all three, amending
     §3.2.1's "not the part model".
     *Placed 2026-10-02 by ruling 480:* a spread acts wherever a patch lies,
     its place staying its root's site.
     *Amended 2026-10-02 by ruling 488:* a spread such as mold is a
     territory and micro life a surface, so the part model serves trees and
     terrain-bodies.

     Links: area Bodies and anatomy · draws on 58 · summary One part model serves every body form: a part attaches to a parent or lies on a site. (amended by 488)

468. **A lineage carries a recipe, and its bodies also grow parts in life.**
     Asked on 2026-10-02, with 467: "Where does a body's part tree come
     from? Evidence: ruling 447 grows a filial child from its lineage's
     recipe, but Isocosm's lineage keeps no recipe, and its Birth clones the
     parent. Mesocosm's lineage carries an axial recipe (segments, tagmata,
     the situs of 252) and isometer's placement policy (facings by role,
     mirroring), and a child develops from them. In life, a Mesocosm body
     gains a part only by incorporation: what it ate lands at a
     plan-resolved attachment (251). A lineage can express only the
     appendage kinds it has eaten (its lexicon)." Options, recommended
     first: (A) "Recipe and lexicon: a lineage carries a recipe and a
     placement policy, and a child develops its part tree from them. In
     life, parts arrive only by incorporation, at a plan-resolved
     attachment. What a body incorporates teaches its lineage that kind, so
     later children can grow it: Mesocosm's kleptoplasty (recommended)." (B)
     "Recipe, no lexicon: as the first, but a recipe may name any shape.
     Incorporating lands parts but teaches the lineage nothing." (C)
     "Recipe, and growth in life: as the first, and a maturing body also
     grows the parts its recipe names that it lacks, each at a plan-resolved
     attachment, so a larva can grow its legs." Mark chose C, "Recipe, and
     growth in life". So a lineage carries a recipe and a placement policy,
     and a child develops its part tree from them; in life a part arrives by
     incorporation, landing at a plan-resolved attachment and teaching the
     lineage its kind, so later children can grow it; and a maturing body
     also grows the parts its recipe names that it lacks, each at a
     plan-resolved attachment.
     *Specified 2026-10-02 by rulings 478 and 479:* the lineage keeps
     Mesocosm's axial recipe and the placement policy, a child draws its own
     soma, and a maturing body grows toward the lineage's recipe once its
     parts are full, at PD2's price.

     Links: area Bodies and anatomy · summary A lineage carries a recipe and placement policy; children develop parts from them and maturing bodies grow recipe parts they lack.

469. **A wound loses a part's cells, regrown at a price where the lineage
     can.** Asked on 2026-10-02, with 467: "What is a wound to a part, and
     does it heal? Evidence: ruling 123 has a blow drain vigour first and
     wound a part when it lands hard or the vigour is gone; wounds heal
     slowly, or never. The record reads a wound as a loss in a part's
     structure: a cut leg slows, a lost eye blinds. Ruling 430 keeps wounds
     in the ledger, not in condition records. Mesocosm has a seam for this,
     called only by its tests: a part loses cells for good, its living cells
     being its capacity, and a function left in pieces stops. A bite (459)
     takes tissue, which regrows by feeding." Options, recommended first:
     (A) "Lost cells, regrown at a price: a wound loses cells from the part
     it lands on, so the part's capacity, adult mass and function cells
     shrink, and tissue it can no longer hold spills to the ground (446). A
     lineage whose traits allow it regrows lost cells as growth, paying each
     cell's mass; one without them never heals (recommended)." (B) "Lost
     cells, for good: Mesocosm's seam as built: a wound's cells never come
     back, and only vigour recovers." (C) "Tissue lost, structure only by
     severing: a wound takes tissue from the part, which spills to the
     ground and regrows by feeding. Only severing loses structure, so
     "never" means a lost part." Mark chose A, "Lost cells, regrown at a
     price". So a wound loses cells from the part it lands on, shrinking its
     capacity, its adult mass and its functions' cells, and the tissue it
     can no longer hold spills to the ground (446); a lineage whose traits
     allow it regrows lost cells as growth, paying each cell's mass, and one
     without them never heals.
     *Widened 2026-10-02 by ruling 485:* the same trait lets growth toward
     the recipe regrow a severed part.
     *Specified 2026-10-02 by ruling 496:* a wound takes cells in proportion
     to each function's, and a healing lineage regrows them before tissue or
     new parts.

     Links: area Bodies and anatomy · draws on 123 · summary A wound loses a part's cells; lineages whose traits allow it regrow them at a price, others never heal. (amended by 485)

470. **A severed part becomes a body of its own, alive where its lineage
     regrows from fragments.** Asked on 2026-10-02, with 467: "Where does a
     severed part's matter go? Evidence: severing takes a part and
     everything under it, never the root (isometer's sever). Mesocosm drops
     a severed part from the body's mass, so "its milligrams have left the
     conservation account". Its corpse-organ eating refuses severed parts,
     because eating one would create matter, and the dismemberment gate
     meant to put them "somewhere honest" is unbuilt. Isocosm conserves
     every milligram. Incorporation takes a part whole with its matter
     (459)." Options, recommended first: (A) "A dead body of its own: the
     severed subtree becomes a dead body on the site, keeping its tissue,
     reserve and provenance, so it can be eaten, incorporated or left to rot
     like any carcass. A shed tail is a meal (recommended)." (B) "Spilled to
     the ground: its matter lands on the site as the body's own (446's
     spill), and the site's mineralization returns it. Nothing is left to
     eat." (C) "Alive where it can be: as the first, but a severed part
     whose lineage can regrow from a fragment lives on as a new body
     (fragmentation, beside 448's budding). Others die as in the first."
     Mark chose C, "Alive where it can be". So a severed subtree becomes a
     body on the site, keeping its tissue, reserve and provenance: a new
     living body where its lineage can regrow from a fragment, fragmentation
     joining 448's budding, and otherwise a dead one, eaten, incorporated or
     left to rot like any carcass.
     *Specified 2026-10-02 by ruling 486:* every fragment of such a lineage
     lives on, and physiology decides which last.
     *Joined 2026-10-03 by ruling 524:* a bud severs from its parent as a
     fragment does, once full.

     Links: area Bodies and anatomy · summary A severed part becomes a body of its own, alive where its lineage regrows from fragments, otherwise dead.

471. **kiss3d, reshaped, is the lit body tenant.** Asked on 2026-10-02 in
     the Conatus/physics session, after Mark asked how renderling's five
     features compare with kiss3d "or other alternative prospective pieces
     of game engine that would compose into the stack", adding: "I am
     willing to reshape a good candidate into an excellent stack
     component/module/crate". Evidence: kiss3d 0.46 (wgpu 30, WGSL, BSD-3)
     covers renderling's shadow maps, light tiling (clustered forward+),
     PBR/IBL, glTF animation and skinning with morphs on web, plus SSAO, OIT,
     transmission, water waves and 2D lighting; it takes the host's device
     but keeps a thread-local `Context` singleton
     (`kiss3d-0.46.0/src/context/context.rs:12`) and wants a window.
     Renderling's edge is GPU-driven slab instancing. Options, recommended
     first: (A) "kiss3d, reshaped: an explicit context handle in place of
     the thread-local, a render-into-caller-targets entry with no window,
     the tracer's depth as a pre-pass, its shadow atlas and light buffer
     exported (recommended)." (B) Grow isometer-render with both as donors.
     (C) Re-adopt renderling. (D) Measure kiss3d and renderling first. Mark
     chose A. So ruling 27's "provided it composes" is met by reshaping
     kiss3d to the stack's device and target seams. Mere's
     `2026-08-22_conatus_engine_plan.md` holds the full evidence.
     *Annotated 2026-10-06 by ruling 606:* the reshape starts from upstream
     kiss3d, taking balaur's commits by name.

     Links: area Stack and engine · draws on 27 · summary kiss3d, reshaped with an explicit context handle and windowless render entry, is the stack's lit body renderer. (amended by 606)

472. **Lighting is a stack-owned light and environment block.** Asked the
     same day. Evidence: terrain is traced and bodies rasterised, joined by
     depth (L3), so a renderer's shadow maps and lights never reach the
     terrain unless shared. Options: (A) "Stack-owned light/environment
     block: sun and day/night from sim fields, the point-light list and the
     water field live in the scene contract; tracer and rasteriser both read
     them; the renderer exports its shadow atlas, light buffer and depth
     (recommended)." (B) The renderer owns lighting and the tracer keeps its
     own sun and fog. Mark chose A, carrying ruling 22's rows across both
     consumers.

     Links: area Stack and engine · draws on 22 · summary Lighting is a stack-owned light and environment block read by both tracer and rasteriser.
473. **WGSL/WESL for raster, CubeCL for compute.** Asked the same day.
     Evidence: all five wing render shaders are WGSL
     (`isometer-lens/src/{tracer,march,grade}.wgsl`,
     `isometer-render/src/{shader,live_body}.wgsl`), against the 2026-08-16
     "author in CubeCL, the brick renderer included" line, and CubeCL 0.10
     has no texture bindings. Options: (A) WGSL/WESL for raster, CubeCL for
     compute (recommended). (B) wgsl-rs once on wgpu 30. (C) rust-gpu 0.10
     for raster too. Mark chose A. This amends the 2026-08-16 line; rust-gpu
     stays for Nexus-derived compute only.

     Links: area Stack and engine · summary WGSL/WESL for raster shaders and CubeCL for compute; rust-gpu stays for Nexus-derived compute only.
476. **vello for documents, kiss3d 2D for lit games.** Asked the same day.
     Evidence: kiss3d's lit 2D (16 lights), radiance-cascade 2D GI and 2D
     skinning would be a second 2D renderer beside vello. Mark chose "vello
     for documents, kiss3d 2D for lit games": kiss3d's 2D only behind the
     scene contract for a lit 2D game. The alternative was vello only.

     Links: area Stack and engine · summary vello serves documents; kiss3d 2D is used only behind the scene contract for lit 2D games.

477. **A body's system network is read from its tree, and a system works by
     degree.** Asked on 2026-10-02, the anatomy brief's fourth round: "How
     is a body's system network made, and when does a system work? Evidence:
     under 465, a system is the functions it assembles and how the parts
     expressing them connect along the body's tree. wing-functions' network
     puts sources and stores (each with a capacity and a charge), gates
     (open or shut) and effects on parts, joined by edges with capacities.
     Mesocosm keeps a seeded, generated network as durable state, bound to
     the body's current parts. Eponym counts a route as open when a path
     runs from any source or store to the effect through live parts and open
     gates, within a hop limit." Options, recommended first: (A) "From the
     tree, working when routed: the world's rules give each system its
     functions and their roles. A body's network is read from its tree on
     demand: nodes on the live parts expressing those functions, edges along
     attachments, capacities from the cells of the parts between. A system
     works when each of its effects can be reached from a source or store
     through live parts and open gates (Eponym's test), so severing a part
     breaks the systems that ran through it (recommended)." (B) "From the
     tree, working by degree: as the first, but a system works to the degree
     its routes carry what a process asks through the edges' capacities, so
     a wounded conduit carries less and the process gets less." (C)
     "Generated and kept, as Mesocosm: each body keeps its network as state:
     a seeded blueprint bound at birth and re-bound after every body change.
     A system works when its routes are open, as in the first." Mark chose
     B, "From the tree, working by degree". So the world's rules give each
     system its functions and their roles; a body's network is read from its
     tree on demand, nodes on the live parts expressing those functions,
     edges along attachments and capacities from the cells of the parts
     between; and a system works to the degree its routes carry what a
     process asks, so a wounded conduit carries less and the process gets
     less.
     *Specified 2026-10-05 by rulings 560 to 562:* every cell carries, a
     conduct cell more; an intact body carries all its natives ask; and a
     process reads its system part by part.

     Links: area Bodies and anatomy · draws on 465 · summary A body's system network is read from its tree on demand, and a system works to the degree its routes carry what a process asks.

478. **The lineage keeps Mesocosm's recipe, and a body grows toward it.**
     Asked on 2026-10-02, with 477: "What does Isocosm's lineage keep as its
     recipe (468)? Evidence: Isocosm's lineage keeps its parent, revision,
     traits and kingdom, and the sim depends on no isometer or Mesocosm
     crate. Mesocosm's recipe runs head to tail as tagmata: each has a
     segment count, the appendage kind and count each segment bears, and
     their shapes. It also holds a variance, by which an individual's
     segment count may stray, and the lexicon. Each individual develops a
     soma from the recipe, with segments drawn within the variance and some
     appendages left absent: "the cheapest evidence that individuals are not
     clones". isometer's policy is the facing each role prefers, mirroring
     and tolerance." Options, recommended first: (A) "Mesocosm's recipe, in
     the sim: the lineage keeps the axial recipe (tagmata, segments,
     appendages, variance, lexicon) and the placement policy as world data.
     Each child draws its own soma from them by its seed, so siblings
     differ. A maturing body grows toward its own soma, so a limb absent at
     development stays absent (recommended)." (B) "A template tree: the
     lineage keeps one adult part tree (shapes, functions, extents,
     attachments) and its lexicon. Every child develops that tree, varying
     only in size, with no segments or tagmata." (C) "Recipe, grown toward
     the lineage's: as the first, but a maturing body grows toward the
     lineage's recipe rather than its own soma, so a limb absent at
     development grows in later." Mark chose C, "Recipe, grown toward the
     lineage's". So the lineage keeps the axial recipe (tagmata, segments,
     appendages, variance, lexicon) and the placement policy as world data;
     each child draws its own soma from them by its seed, so siblings
     differ; and a maturing body grows toward the lineage's recipe rather
     than its own soma, so a limb absent at development grows in later.
     *Bounded 2026-10-02 by ruling 485:* a severed part grows back only
     where the lineage can heal.
     *Varied 2026-10-02 by ruling 495:* a lineage trait decides whether
     segments grow toward the recipe's count, epimorphic by default.
     *Given its kinds 2026-10-03 by rulings 511 and 512:* a kind is a part
     template, and a founded lineage's recipe is drawn by its kingdom, the
     authored roster kept as presets.
     *Varied 2026-10-03 by ruling 531:* each recipe carries its absence odds
     beside its variance.

     Links: area Bodies and anatomy · draws on 468 · summary The lineage keeps Mesocosm's axial recipe and placement policy as world data; children draw their own soma; maturing bodies grow toward the lineage's recipe. (amended by 485, 495)

479. **A body grows a part it lacks once its parts are full, at PD2's
     price.** Asked on 2026-10-02, with 477: "When does a maturing body grow
     a part it lacks, and what does the part cost? Evidence: Mesocosm grows
     a filial child whole from its recipe (447); in life it adds parts only
     by incorporation and otherwise grows tissue, which 464 spreads over the
     parts in proportion to their room. A seed (448) starts minimal and
     grows by its own feeding. In isometer's plan, parts "fill in during an
     epoch, automatically" as a body eats. Mesocosm prices development by
     the cell (PD2): each cell whose expression changes costs one cell's
     mass, paid from the reserve into the ground." Options, recommended
     first: (A) "When its parts are full: growth that finds no room left in
     the body's parts grows the next part the body lacks, nearest the root
     first. Its tissue comes from that growth, and expressing its functions
     costs PD2's price, a cell's mass per cell, from the reserve into the
     ground. Growth then fills it by 464 (recommended)." (B) "By life stage:
     the recipe marks which parts come at which stage, and a body grows all
     of a stage's parts when it reaches that stage's mass, as a larva
     becomes an adult. Paid the same way." (C) "With every meal: each meal
     grows the next missing part first, as far as it reaches, before any
     tissue thickens, so a young body takes its form before its size. Paid
     the same way." Mark chose A, "When its parts are full". So growth that
     finds no room left in the body's parts grows the next part the body
     lacks, nearest the root first, its tissue from that growth and its
     functions' expression at PD2's price, a cell's mass a cell from the
     reserve into the ground; growth then fills it by 464.
     *Joined 2026-10-03 by ruling 518:* growth with the parts full also
     fills the provision; which comes first goes back to Mark. *Ordered
     2026-10-03 by ruling 520:* which comes first is a lineage trait.
     *Placed against the reserve 2026-10-03 by ruling 535:* before it or
     after it, as the lineage breeds from income or from capital.

     Links: area Bodies and anatomy · draws on 468, 464 · summary A body whose parts are full grows the next part it lacks, nearest the root first, at PD2's price of a cell's mass per cell.

480. **A spread acts wherever a patch lies.** Asked on 2026-10-02, with 477:
     "Where is a spread body present? Evidence: under 467 a fungus is one
     body whose patches lie on many sites. Isocosm ties everything to a
     body's one place: an act reads and writes its body's site (fixing draws
     that site's soil), targets are filed by place and lineage (237), and
     the crowd groups members by site. A process binds a live part
     expressing its function (338)." Options, recommended first: (A)
     "Wherever a patch lies: an act reads and writes the site of the part it
     binds, and a body can be a target wherever it has a live part, so a
     fungus fixes from each patch's soil and is grazed where its patches
     lie. The body's place, for its record and its moves, stays its root's
     site (recommended)." (B) "Its root's site only: a spread acts and is
     found only at its root's site. Patches elsewhere hold matter but act on
     nothing until the places family moves bodies between sites." Mark chose
     A, "Wherever a patch lies". So an act reads and writes the site of the
     part it binds, and a body can be a target wherever it has a live part:
     a fungus fixes from each patch's soil and is grazed where its patches
     lie, while the body's place, for its record and its moves, stays its
     root's site.
     *Amended 2026-10-02 by ruling 488:* a spread is a territory, acting at
     each place it covers; an act still reads and writes the site of the
     part it binds.

     Links: area Bodies and anatomy · draws on 467 · summary A spread body acts and can be targeted wherever a live patch lies; its place for record and moves stays its root's site. (amended by 488)

483. **The projection's diff comes from the record's receipts.** Asked the
     same day: where does the record-to-scene-contract projection learn what
     changed? Options: (A) "The record's receipts (recommended)", which the
     authoritative record already emits per tick, so the ECS never becomes a
     second source of truth. (B) hecs `ChangeTracker`, exact but scanning
     every tracked entity. (C) A dirty-flag component. Mark chose A.

     Links: area Stack and engine · summary The record-to-scene projection learns what changed from the record's per-tick receipts, not from ECS change tracking.
485. **Growth toward the recipe regrows a severed part only where the
     lineage can heal.** Asked on 2026-10-02, the anatomy brief's fifth
     round: "Does growth toward the recipe regrow a severed part? Evidence:
     469 regrows lost cells only where the lineage's traits allow, and
     others never heal (123's "slowly, or never"). 478 has a maturing body
     grow toward its lineage's recipe, so a limb absent at development grows
     in, and 479 grows a missing part once the body's parts are full.
     Severing takes a part and its subtree, which leaves a part the recipe
     names missing. A living fragment regrows a whole body (470). Mesocosm
     regrows nothing." Options, recommended first: (A) "Where the lineage
     can heal: growth toward the recipe regrows a severed part only in a
     lineage whose traits let it heal (469's trait), so one trait governs
     healing cells and regrowing parts. A lineage without it keeps its
     losses, and what development left absent still grows in (recommended)."
     (B) "Always: any body regrows a severed part once its parts are full,
     like any part it lacks. 469's trait governs only lost cells." (C)
     "Never: growth toward the recipe fills only what development left
     absent. A severed part stays lost in every lineage, and only cells
     heal." Mark chose A, "Where the lineage can heal". So growth toward the
     recipe regrows a severed part only in a lineage whose traits let it
     heal, 469's trait governing both healing cells and regrowing parts; a
     lineage without it keeps its losses, and what development left absent
     still grows in (478).

     Links: area Bodies and anatomy · draws on 469, 478 · summary Growth toward the recipe regrows a severed part only in a lineage whose traits let it heal.

486. **Every fragment of a lineage that regrows from fragments lives, and
     physiology decides which last.** Asked on 2026-10-02, with 485: "What
     does a fragment need in order to live (470)? Evidence: a severed part
     of a lineage that regrows from fragments lives on as a body, keeping
     its tissue and reserve. 479 grows it toward the recipe once its parts
     are full, and growth needs meals. A body pays rent from its reserve, or
     from tissue when it has no store (463), and starves at Mesocosm's
     margin (446). A filial child starts at a quarter of its parent (447).
     Under 58, a fungus split apart stays one body, so fragments come from
     part trees." Options, recommended first: (A) "Any fragment; physiology
     decides: every severed part of such a lineage lives on as a body. One
     with no way to feed starves by rent in time, so no new rule decides
     which fragments live (recommended)." (B) "A fragment that can feed: it
     lives only if it holds a part expressing intake or fix, so it can feed
     while it regrows. Others die at once." (C) "A fragment of size: it
     lives only if it holds at least a world-rule share of the body's mass,
     the filial child's quarter by default. Others die at once." Mark chose
     A, "Any fragment; physiology decides". So every severed part of a
     lineage that regrows from fragments lives on as a body, and one with no
     way to feed starves by rent in time; no new rule decides which
     fragments live.

     Links: area Bodies and anatomy · draws on 470 · summary Every severed part of a fragment-regrowing lineage lives on as a body; physiology decides which last, with no new rule.

488. **A mold is a territory, and micro life a surface.** Asked on
     2026-10-02, opening the anatomy brief's fifth round: "Is a body a
     placemap? Evidence: the forms-of-life brief refused internal pursuit
     because "a host is one body at one position; there is no place graph
     inside it". It named an interior place graph as the one thing that
     would need new world machinery, a second authority over space. The
     place-graph plan's rule 6 makes nesting elective and continuous in one
     global space; a place with no internal topology stays a leaf. Places
     are the finest location the sim holds (422), and reach, travel and
     plague run over them (§3.7). Ruling 39's germ can "inhabit a creature",
     and a terrain-body is at once a creature and a place (§3.2.1)."
     Options, recommended first: (A) "Every body a placemap: a body's parts
     are places nested at its location in the one place graph (rule 6),
     joined by its attachments, or for a spread by the routes between its
     patches. Other bodies can stand in those places, so a germ inhabits its
     host and a world rests on a lion turtle. Systems route along those
     edges, and a body acts once at each place it occupies: a tree once, a
     spread at each patch (recommended)." (B) "Only bodies that carry
     places: terrain-bodies and spreads are placemaps, but a part tree stays
     a leaf (rule 6). Nothing lives inside an animal, and the forms-of-life
     brief's refusal of internal pursuit stands." (C) "Not now: keep the
     part model as ruled and take the placemap reading up when the places or
     micro family needs it. 480's question of how a spread acts stays open."
     Mark answered: "Oh… i meant the other way… like think of a locale where
     mold has spread underground… would it be more helpful to think of such
     a body as more like territories than individual entities? Or surfaces
     (kinda analogue to the surfaces and effects mechanic in larian games?)?
     Same question for micro entities". Answered that both fit these forms
     better than a body with parts, and that the sim already holds most of a
     surface: a lineage's living matter on the ground as site accounts,
     where spills land; an agentless process bound to the place acting on it
     each tick, the mineralization rate; and conditions on places that
     whatever stands there reads, combining by key (430, 436). Territory
     keeps one entity, suiting a mold, 58's "only the instance of the
     whole"; surface drops it, suiting germs, 39's "paint that can spread".
     Leaned territory for mold and surface for micro. Put back: "How should
     the sim hold a spread like underground mold, and micro life? Evidence:
     58 made a fungus one body, "only the instance of the whole", and germs
     many bodies that burn generations. 39 called a germ "like a paint that
     can spread", and §3.2.1 says to play a germ is to play the lineage
     directly. The sim already lays a lineage's living matter on the ground
     as site accounts, where spills land. An agentless process bound to the
     place acts on that matter each tick (mineralization), and conditions
     live on places (430), combining by key (436). Larian's surfaces follow
     that pattern: matter on the ground that spreads, combines and affects
     whatever stands in it." Options, recommended first: (A) "Territory for
     mold, surface for micro: a mold or fungus is one entity whose body is a
     territory: its matter on each place it covers, acting there and
     spreading into neighbouring places, with no part tree. A germ lineage
     is a surface: its matter lies on places and in hosts, is spread and
     transformed by place-bound processes, and applies conditions to
     whatever stands in it. Its lineage is its only identity. A terrain-body
     stays a part tree (recommended)." (B) "Territories for both: germs are
     also entities with territories, one per colony, so a plague has an
     identity that can be met, tracked and named. Mold as in the first." (C)
     "Surfaces for both: neither is an entity: a mold is also its lineage's
     matter on places, and a played fungus directs its lineage's surface."
     (D) "Bodies, as ruled: keep 467 and 480: a spread is one body whose
     patches are parts on sites, and germs are many bodies (58)." Mark chose
     A, "Territory for mold, surface for micro". So a mold or fungus is one
     entity whose body is a territory, its matter on each place it covers,
     acting there and spreading into neighbouring places, with no part tree;
     a germ lineage is a surface, its matter lying on places and in hosts,
     spread and transformed by place-bound processes and applying conditions
     to whatever stands in it, its lineage its only identity; and a
     terrain-body stays a part tree. This amends 58's germs as many bodies,
     and 467's and 480's patches as parts, for these forms.
     *Specified 2026-10-02 by rulings 497 to 500:* both lie in places'
     ledgers under their keys and spread by overflow along routes, a germ
     lineage forks strains, and their functions scale by the matter at each
     place.
     *Placed 2026-10-02 by ruling 502:* built as the bodies family's last
     checkpoint, at site grain.

     Links: area Bodies and anatomy · amends 58, 467, 480 · summary A mold is a territory (one entity, matter on each place covered) and micro life a surface; terrain-bodies stay part trees.

489. **The default systems take the proposed table.** Asked on 2026-10-02,
     the anatomy brief's sixth round: "Do the default systems take the
     proposed table (see the preview)? Evidence: 277 names the
     integumentary, respiratory, circulatory, digestive, nervous and
     reproductive systems, the senses, and systems fauna lack. Under 465 and
     477, a system is functions in roles (sources, stores, gates, effects),
     read from the tree, and it works to the degree its routes carry what a
     process asks. Eponym's one network is a source on the root (capacity
     24, charge 24) feeding effects on two limbs. The probe's natives read
     intake (the meal), fix (income), contract (the mouthful) and secrete
     (the dose)." The preview, as put:

     | System | Sources | Stores | Gates | Effects | Read by |
     | --- | --- | --- | --- | --- | --- |
     | digestive | intake | store | gate | every living part | the meal |
     | photosynthetic | fix | store | gate | every living part | a producer's income |
     | muscular | every living part | store | gate | contract | the mouthful, reach, Eponym's strikes |
     | nervous | sense | - | gate | contract | perception, reach (wait for places) |
     | glandular | secrete | store | gate | the biting parts | the dose |
     | reproductive | every living part | store | gate | reproduce | reproduction (447 to 450) |
     | circulatory | circulate | store | gate | every living part | none yet |
     | respiratory | respire | - | gate | contract | none yet |
     | excretory | every living part | - | gate | excrete | none yet |
     | integumentary | no routes: shells expressing support | | | | none yet |

     Options, recommended first: (A) "The proposed table: ten systems, each
     with functions in roles. Routes run along attachments, their capacities
     from the cells of the parts between (477). The senses are the nervous
     system's sources. The integument routes nothing and works by the share
     of its cells still living (recommended)." (B) "Only what a mechanic
     reads: five systems now: digestive, photosynthetic, muscular, glandular
     and reproductive. The others enter with the first mechanic that reads
     them." (C) "Fewer, broader systems: four: neuromuscular (sense and
     every living part, through gates, to contract), as Eponym's one network
     is; metabolic (intake or fix, to stores and every part); reproductive;
     and the integument." Mark chose A, "The proposed table". So the default
     set holds ten systems, each its functions in roles, routes running
     along attachments with capacities from the cells of the parts between;
     the senses are the nervous system's sources; and the integument routes
     nothing and works by the share of its cells still living. Each is read
     as the table lists, and circulatory, respiratory and excretory by
     nothing yet.
     *Extended 2026-10-05 by ruling 568:* each body carries its systems,
     the world's defaults at founding, varying at birth.
     *Specified 2026-10-05 by rulings 574 and 575:* a body carries the
     defaults it realizes, and a native acts on any carried system routing
     its function, the pairings naming the defaults that route them.
     *Specified 2026-10-05 by rulings 588 and 589:* the biting parts are the
     parts bitten, and the mouthful reads muscular for the limbs' share.

     Links: area Bodies and anatomy · draws on 465, 477 · summary The default systems are ten, each its functions in roles, routes along attachments with capacities from cells; the integument works by living-cell share. (amended by 657)

491. **The generator riffs systems by substitution.** Asked on 2026-10-02,
     with 489: "How does the generator riff new systems? Evidence: in 277
     Mark asked to "riff new ones and mix existing ones to make new
     forms/systems... new-looking forms of life that still are biologically
     reasonable". A system is functions in roles (465, 477), and each
     function is admitted by its shapes (466). Under 465 the generator
     recombines systems, and a system is data in the world's rules."
     Options, recommended first: (A) "By substitution: a riffed system is a
     default one with a role's function swapped for another, or one added.
     It is kept only if a body can realize its routes: an animal whose
     respiratory source is fix, a gut that photosynthesizes. Each riff
     becomes world data with its own name (recommended)." (B) "Free
     assembly: any functions in any roles, kept if some native reads the
     result. Stranger systems, less often reasonable." (C) "Not yet: the
     default set only for now; riffing waits for the generator's own round."
     Mark chose A, "By substitution". So a riffed system is a default one
     with a role's function swapped for another, or one added, kept only if
     a body can realize its routes, such as an animal whose respiratory
     source is fix or a gut that photosynthesizes; each riff becomes world
     data with its own name.
     *Corrected 2026-10-02 by ruling 492:* 466's gate refused its own
     example, a gut that photosynthesizes, fix being admitted only on
     sheets; under 492 the gut fixes by its area.
     *Built 2026-10-02 by ruling 503:* in the bodies family's systems
     checkpoint.
     *Reframed 2026-10-05 by ruling 563:* no single riff is picked; a riff
     is a latent capability becoming an ability through variation in
     reproduction, certified as a mechanism.
     *Specified 2026-10-05 by ruling 580:* within a run, half of riffs swap
     and half add.

     Links: area Bodies and anatomy · draws on 465 · summary The generator riffs new systems by substitution, swapping or adding a role's function, kept only if a body can realize its routes. (amended by 492, 563)

492. **No shape gates a function; what a part does scales with the
     measurement its mechanic names.** Raised by Mark on 2026-10-02, the
     anatomy brief's seventh round, as the sixth was recorded: "The shapes,
     that’s the part I’m most worried about. That correspondence between
     function and shape". Asked the same day: "How should a part's shape
     relate to its function? Evidence: in the sim, a part's shape is a
     declared label, one of the eight, and 466's table gates which functions
     it may express. Nothing checks the label against the part's geometry,
     which is only a box of half-extents. The classifier 276 named, reading
     hollowness, branching and enclosure, does not exist. The only one in
     code is isometer's, which reads a box's extents into four roles (lump,
     rod, sheet, point), so tube, branch, shell and joint are labels nothing
     can verify. The gate also refuses 491's own example, a gut that
     photosynthesizes, since fix is admitted only on sheets. 405 has
     construction declare function and geometry measure what a mechanic
     needs." Options, recommended first: (A) "Measurements, not gates: any
     part can express any function. What it does scales with the measurement
     its mechanic names: fix by the area a part presents, contract by its
     length, store and intake by its volume. Form still decides how well,
     the table becomes the fits the generator draws by default, and shapes
     are read from measurements to name parts, never to gate them. A
     photosynthesizing gut fixes by its area, poorly (recommended)." (B)
     "Classified, then gated: keep 466's gate, but read each part's shape
     from its geometry, as 276 meant. The part records hollowness, branching
     and enclosure beside its extents, and the classifier names its shape,
     so a label cannot lie." (C) "Gated, with affinity: keep the eight
     declared shapes. Any shape may express any function, but at a
     world-rule fraction of the rate unless the table lists it." (D) "As
     ruled: keep 466's gate on declared labels. Riffs swap only functions
     the shapes admit, and 491's example becomes an animal whose leaves feed
     its gut." Mark chose A, "Measurements, not gates". So any part can
     express any function, and what it does scales with the measurement its
     mechanic names, fix by the area a part presents, contract by its
     length, store and intake by its volume, so form still decides how well;
     466's table becomes the fits the generator draws by default; and shapes
     are read from measurements to name parts, never to gate them. This
     amends the gate of 338 and 466 and the classifier's role in 276, and
     makes 491's example, a gut that photosynthesizes, which 466's gate
     refused when 491 was recorded, a body that can be.
     *Specified 2026-10-02 by rulings 493 and 494:* each function's
     measurement, by its share of the cells, and how the eight names are
     read.

     Links: area Bodies and anatomy · amends 276, 338, 466 · summary No shape gates a function; any part can express any function and its effect scales with the measurement its mechanic names.

493. **Each function scales with one measurement, by its share of the
     cells.** Asked on 2026-10-02, the anatomy brief's eighth round: "What
     does each function measure, and how (see the preview)? Evidence: 492
     ruled fix by area, contract by length, and store and intake by volume.
     The sim already reads two of these: Span, the longest half-extent of
     each part expressing a function, summed, which rent and the mouthful
     read; and Voxels, a part's volume, which the ceiling reads. It has no
     area reading, so fixing still reads the body's tissue to the
     three-quarter power (TD2c). A part's cells are counted per function,
     and a lump splits its cells between intake and store (466)." The
     preview, as put:

     | Function | Scales with | Read now |
     | --- | --- | --- |
     | contract | length | span: rent, the mouthful |
     | intake | volume | - |
     | sense | length | - (perception waits for places) |
     | fix | presented area | - (income reads tissue) |
     | secrete | volume | cell mass: rent, the gland |
     | support | cross-section | - |
     | conduct | cross-section | - (a route's capacity) |
     | gate | cross-section | - |
     | store | volume | - |
     | circulate | volume | - |
     | respire | presented area | - |
     | excrete | volume | - |
     | reproduce | volume | - |
     | grip | length | - |
     | adhesion | presented area | - |

     Options, recommended first: (A) "The proposed table, by cell share:
     every function names one measurement, read from each part's box: length
     (its longest extent), presented area (its largest face), cross-section
     (its smallest face) or volume (its voxels). A function takes its part's
     measurement in proportion to the cells it holds there. The natives take
     these up with the bodies family, as with their systems (490)
     (recommended)." (B) "Whole part, not by share: as the table, but a
     function reads the whole measurement of each part expressing it, as
     Span does today. A lump that takes in and stores counts its full volume
     for each." (C) "Only the four ruled: fix, contract, store and intake
     take their measurements. Each other function names its measurement when
     its mechanic arrives (405)." Mark chose A, "The proposed table, by cell
     share". So every function names one measurement read from each part's
     box, length its longest extent, presented area its largest face,
     cross-section its smallest face or volume its voxels, as the table
     lists; a function takes its part's measurement in proportion to the
     cells it holds there; and the natives take them up with the bodies
     family, as with their systems (490).
     *Applied 2026-10-02 by ruling 505:* income and the mouthful read their
     measurements alone, calibrated to the domain's median body.

     Links: area Bodies and anatomy · draws on 492 · summary Each function scales with one measurement (length, presented area, cross-section or volume), taken in proportion to its share of a part's cells.

494. **The eight names are read from the box and the tree, and hollows
     declared.** Asked on 2026-10-02, with 493: "How are tube, branch, shell
     and joint read, now that shapes only name parts (492)? Evidence:
     isometer's classifier reads a box into lump, rod, sheet and point. A
     sim part is a box, which shows no hollow (tube, shell), no fork
     (branch) and no joint. The sim's tree does show forks (a part with two
     or more children) and joints (a small part between two others).
     isometer's part points to its voxel volume, but the sim keeps no
     volumes (462). 405 lets construction declare a property that names its
     measurement." Options, recommended first: (A) "From the tree, and
     declared hollows: lump, rod, sheet and point come from the box. Branch
     and joint come from the tree. Tube and shell are declared at
     construction from the body's voxels, naming that measurement (405),
     since the sim keeps no volumes (recommended)." (B) "Four names: the sim
     names parts by the box alone: lump, rod, sheet and point. Tube, branch,
     shell and joint leave its vocabulary, amending 276." (C) "The sim keeps
     hollowness: each part records whether it is hollow and whether it
     encloses, beside its extents, so all eight are read in the sim. Branch
     and joint come from the tree." Mark chose A, "From the tree, and
     declared hollows". So lump, rod, sheet and point are read from the box,
     branch and joint from the tree, and tube and shell declared at
     construction from the body's voxels, naming that measurement (405),
     since the sim keeps no volumes.

     Links: area Bodies and anatomy · draws on 492, 276 · summary Lump, rod, sheet and point are read from the box, branch and joint from the tree, tube and shell declared at construction.

495. **Whether segments grow toward the recipe's count is a lineage trait.**
     Asked on 2026-10-02, with 493: "Does a body's segment count grow toward
     the recipe's (478)? Evidence: Mesocosm's soma draws each tagma's
     segment count within the recipe's variance, so siblings differ. 478
     grows a maturing body toward its lineage's recipe, so appendages absent
     at development grow in. In nature, millipedes add segments after
     hatching (anamorphic growth), while some centipedes hatch with all of
     theirs (epimorphic)." Options, recommended first: (A) "By lineage
     trait: whether a body adds segments toward the recipe's count as it
     matures (anamorphic) or keeps its drawn count (epimorphic) is a trait
     of the lineage, epimorphic by default. Absent appendages grow in either
     way (478) (recommended)." (B) "Segments stand as drawn: only absent
     appendages grow in. A body's segment count is its own for life." (C)
     "Segments grow too: every body grows toward the recipe's segment count:
     a short one adds segments, and a long one keeps its extras." Mark chose
     A, "By lineage trait". So whether a body adds segments toward the
     recipe's count as it matures, anamorphic, or keeps its drawn count,
     epimorphic, is a trait of its lineage, epimorphic by default; absent
     appendages grow in either way (478).

     Links: area Bodies and anatomy · draws on 478 · summary Whether segments grow toward the recipe's count is a lineage trait, epimorphic by default; absent appendages grow in either way.

496. **A wound takes cells in proportion, and healing comes first.** Asked
     on 2026-10-02, with 493: "Which cells does a wound take, and when do
     they regrow (469)? Evidence: 469 has a wound lose cells from the part
     it lands on, shrinking its capacity, adult mass and functions' cells. A
     sim part's cells are counts per function with no adjacency (Mesocosm's
     mosaic has adjacency, and a function left in pieces stops). 464 spreads
     rent and growth in proportion, with no part first. 485 regrows severed
     parts where the lineage heals, and 479 grows a missing part once the
     parts are full." Options, recommended first: (A) "In proportion, healed
     first: a wound takes cells from the part's functions in proportion to
     the cells each holds. A healing lineage's growth regrows lost cells
     before it fills tissue or grows new parts, paying each cell's mass
     (recommended)." (B) "Drawn, healed first: the cells come from one
     function, drawn by the cells each holds, so a wound lands somewhere
     specific: a bite to the gut takes intake. Regrowth as in the first."
     (C) "In proportion, healed alongside: as the first, but lost cells
     regrow in proportion alongside the tissue's filling, not before it."
     Mark chose A, "In proportion, healed first". So a wound takes cells
     from the part's functions in proportion to the cells each holds, and a
     healing lineage's growth regrows lost cells before it fills tissue or
     grows new parts, paying each cell's mass.

     Links: area Bodies and anatomy · draws on 469 · summary A wound takes cells from a part's functions in proportion; a healing lineage regrows lost cells before filling tissue or growing parts.

497. **A territory's and a surface's matter lies in each place's ledger,
     under their keys.** Asked on 2026-10-02, the anatomy brief's ninth
     round: "Where does a territory's and a surface's matter sit (488)?
     Evidence: a site's ledger already holds a lineage's living matter under
     the same keys as a body's tissue: that is where spills land, and where
     mineralization, an agentless process bound to the place, returns a
     share to soil each tick. A body's ledger can hold any key. Conditions
     live on places (430), and a thing standing there reads them." Options,
     recommended first: (A) "On the places, under their keys: both lie in
     each place's own ledger, as spills do: a territory's matter under its
     entity's key, a surface's under its lineage's. Whatever acts at a place
     (a grazer, a fire, mineralization) reads them where they lie. A host's
     ledger holds a surface's matter the same way. The territory entity
     keeps its identity, record and mind (recommended)." (B) "In the
     territory, place by place: the territory entity keeps one account for
     each place it covers, so its whole travels with its record. Surfaces as
     in the first." Mark chose A, "On the places, under their keys". So both
     lie in each place's own ledger, as spills do, a territory's matter
     under its entity's key and a surface's under its lineage's, read where
     they lie by whatever acts at the place, a grazer, a fire or
     mineralization; a host's ledger holds a surface's matter the same way;
     and the territory entity keeps its identity, record and mind.

     Links: area Bodies and anatomy · draws on 488 · summary A territory's and a surface's matter lies in each place's ledger under their keys; the territory entity keeps identity, record and mind.

498. **Territories and surfaces spread by overflow along routes.** Asked on
     2026-10-02, with 497: "How do territories and surfaces spread (488)?
     Evidence: routes join sites, each with a travel time and a transmission
     (per million). Reach already spreads an event's strength along them,
     scaled by transmission at each hop and arriving after the travel. 39's
     germ "spreads depending on substrate". 155 lets a player direct a
     fungus monocreature whole, and a methodology that chooses is 37's."
     Options, recommended first: (A) "Overflow along routes: growth at a
     place beyond what its substrate holds (the soil, a host's tissue) moves
     into neighbouring places along routes, a share by each route's
     transmission, after its travel. A territory that chooses, a player's
     mold among them, directs its overflow among its neighbours. Mold creeps
     where the ground feeds it, and a plague follows the routes
     (recommended)." (B) "Diffusion each tick: a share of the matter at each
     place moves to every neighbour each tick by transmission, whether or
     not the place is full, so a spread thins as it widens. No choosing."
     (C) "Chosen for both: a territory and a surface each run a spreading
     process whose target is drawn among their neighbours by what the
     substrate there offers, weighted like a feeding target." Mark chose A,
     "Overflow along routes". So growth at a place beyond what its substrate
     holds, the soil or a host's tissue, moves into neighbouring places
     along routes, a share by each route's transmission after its travel; a
     territory that chooses, a player's mold among them, directs its
     overflow among its neighbours; mold creeps where the ground feeds it,
     and a plague follows the routes.

     Links: area Ecology · draws on 488 · summary Territories and surfaces spread by overflow along routes: growth beyond substrate capacity moves to neighbours by route transmission after travel.

499. **A germ lineage revises its genes by forking strains.** Asked on
     2026-10-02, with 497: "What carries a germ lineage's genes as it
     revises them? Evidence: 58 has germs revise their genes on the fly by
     host and environment. §3.2.1 says that for a germ the epoch boundary
     comes every generation, and to play a germ is to play the lineage
     directly. Other lineages change their genotype only at the epoch
     boundary (57). Isocosm's lineage can fork, each lineage naming a
     parent." Options, recommended first: (A) "Strains: a germ lineage
     revises by forking strains, child lineages, where a host or place
     selects. Each strain is a surface of its own, carrying its revised
     genes, so a plague drifts and diverges as it spreads (recommended)."
     (B) "One lineage, conditioned: one genotype for the whole surface, its
     expression conditioned by host and place (36). Each generation's
     revision changes all of it." (C) "The played lineage alone: a played
     germ revises its genes on the fly, as its shop is open during the
     round. Unplayed germs revise only at the epoch boundary, like other
     lineages." Mark chose A, "Strains". So a germ lineage revises by
     forking strains, child lineages, where a host or place selects, each
     strain a surface of its own carrying its revised genes, so a plague
     drifts and diverges as it spreads.

     Links: area Bodies and anatomy · draws on 58, 488 · summary A germ lineage revises its genes by forking strains, child lineages, selected by host or place.

500. **A territory or surface expresses its functions by the matter at each
     place.** Asked on 2026-10-02, with 497: "What of the catalogue does a
     territory or a surface express without parts? Evidence: functions are
     expressed by a part's cells, each scaled by a measurement and taken by
     its share of the cells (466, 493). A territory or a surface has matter
     on places but no parts or cells (488). Mold decomposes and may fix;
     39's germ infects and decomposes corpses." Options, recommended first:
     (A) "By the matter at each place: a territory or surface expresses the
     functions its lineage's traits name, at each place where it lies,
     scaled by the matter it holds there in place of cells and measurements.
     A mold's decay at a place grows with its mass there (recommended)." (B)
     "Cells per place: the matter at each place is counted in cells of the
     lineage's functions, at a cell's mass, so the catalogue and 493's
     measurements apply unchanged, area read from the place's footprint."
     (C) "Only their own processes: they express no catalogue functions.
     Their definitions name the processes they run (decompose, infect, fix),
     which read the matter where it lies." Mark chose A, "By the matter at
     each place". So a territory or surface expresses the functions its
     lineage's traits name at each place where it lies, scaled by the matter
     it holds there in place of cells and measurements, so a mold's decay at
     a place grows with its mass there.

     Links: area Bodies and anatomy · draws on 488, 466, 493 · summary A territory or surface expresses its lineage's functions at each place, scaled by the matter it holds there.

504. **Each part keeps its own ledger and is a holder.** Asked on
     2026-10-02, opening checkpoint 7 (501): "Where does a part's matter
     live in the sim's books (459)? Evidence: an account's holder is a site,
     an entity or the dev source (31 uses in 9 files), and entity accounts
     are read 182 times in 46 files. Every flow, share, pass frame and crowd
     state is keyed by holder and account. `Binding::Part` exists, but its
     doc says a part "keeps no ledger". The hunting family's members have no
     parts at all." Options, recommended first: (A) "A ledger on each part:
     a part gains its own ledger, keyed by provenance as the scruple is, and
     becomes a holder (entity, part), so flows, shares and frames name it.
     The body's totals are readings over its parts. A body without parts
     keeps its entity ledger, so the hunting family's certified numbers must
     not move: a control (recommended)." (B) "Part keys on the body's
     ledger: the body's ledger keeps one account per part and kind (part 2's
     tissue), so holders and flows stay as they are. Parts stay ledgerless
     in the schema, and each account's name carries its part." Mark chose A,
     "A ledger on each part". So a part gains its own ledger, keyed by
     provenance as the scruple is, and becomes a holder that flows, shares
     and frames name; the body's totals are readings over its parts; and a
     body without parts keeps its entity ledger, the hunting family's
     certified numbers standing as the control.
     *Applied 2026-10-03 by ruling 544:* an incorporated part keeps its
     donor's matter under its provenance.
     *Kept 2026-10-03 by ruling 525:* the ecology family's extentless bodies
     stay, the scale control, and only worlds that ask for bodies found
     from recipes.

     Links: area Bodies and anatomy · draws on 459 · summary Each part keeps its own ledger and is a holder; the body's totals are readings; bodies without parts keep their entity ledger.

507. **The averaged crowd averages all of a lineage's own matter.** Asked on
     2026-10-02, when checkpoint 7's 1,000-draw check (master seed
     1790993524767155700) failed: "What should the averaged crowd average
     from checkpoint 7 on? Evidence: the negative control's averaged crowd
     averages each lineage's reserve (446), but under 506 producers keep
     none and starvation reads tissue below 21 mg, so it can't move
     starvation: 8.19 against 8.42, not detected. On the same 200 draws
     (seed 20261002), reserve-only averaging leaves producers identical to
     the exact runner (alive 60.65 against 60.65, starving 6.29 against
     6.28). Averaging tissue and reserve cuts their starvation to 3.25 (D
     0.335, p 0.0075), and it still catches grazers' meals." The run's other
     failure was a fault, not a fork: the exact runner weighed a prey's
     entity ledger for its meal holdings, empty since 504, reading 0 against
     the crowd's 9,830. Options, recommended first: (A) "All own matter:
     each lineage's tissue and reserve at a site, each replaced by its
     average every tick and given back by room. One rule for every lineage,
     matching the hunting family's averaged body. Piloted: starvation
     detected at D 0.335 on 200 draws (recommended)." (B) "Tissue where no
     store: a lineage that stores averages its reserve as at checkpoint 6;
     one that keeps no reserve averages its tissue. Grazers' control stays
     checkpoint 6's. Producers come out the same as option 1, which is the
     arm the pilot measured." (C) "Reserve only, any detection: keep
     checkpoint 6's averaging, and judge the control by any detected
     difference rather than a starvation one. It caught grazers' meals
     (6.36 against 7.70), but producers stay untouched, so starvation is no
     longer under test." Mark chose A, "All own matter". So from checkpoint
     7 on, the averaged crowd replaces each lineage's tissue and its reserve
     at a site by their averages every tick, each given back through the
     parts by their room, and it must still fail the starvation check
     (209). *Reading, not ruled:* the hunting family's averaged crowd, which
     averages the body each kind is sized by, is unchanged; and what an
     average cannot fit in one member's parts passes to classmates with
     room, so the control loses no matter.

     Links: area Processes and rules · amends 209 · summary From checkpoint 7 on, the averaged negative-control crowd averages each lineage's tissue and reserve together and must still fail the starvation check.

511. **A recipe's kinds are part templates.** Asked on 2026-10-03, with 510:
     "How does the sim's recipe name what a segment bears? Evidence:
     Mesocosm's tagma names an appendage kind (limb, feeler, plate, mouth,
     vane) plus selectors into a palette of four box templates per role.
     The kind implies the work: a mouth eats, a lit plate fixes. The sim's
     parts carry extents and the cells each function holds, with any
     function allowed on any shape (492). The lexicon is the set of kinds a
     lineage may express (468)." Options, recommended first: (A) "Kinds as
     part templates: a kind is a template: its box and the share of cells
     each function takes. Tagmata name a segment template and the kind each
     segment bears, with counts, and the lexicon is a set of kinds.
     Mesocosm's five appendages and its palette become the default
     templates (recommended)." (B) "Mesocosm's words as data: the appendage
     enum and palette selectors stay as they are, with a fixed table from
     each kind to its functions. The lexicon is a set of appendage kinds."
     Mark chose A, "Kinds as part templates". So a kind is a template, a
     box and the share of cells each function takes; a recipe's tagmata
     name a segment template and the kind each segment bears, with their
     counts; a lineage's lexicon is a set of kinds; and Mesocosm's five
     appendages and its palette become the default templates.

     Links: area Bodies and anatomy · draws on 468 · summary A recipe's kinds are part templates: a box and the share of cells each function takes; the lexicon is a set of kinds.

512. **A founded lineage's recipe is drawn by its kingdom, the roster kept
     as presets.** Asked on 2026-10-03, with 510: "What recipe does a
     founded world's lineage start with? Evidence: Mesocosm founds from an
     authored roster of eight by default: 3 producers, 3 consumers and 2
     decomposers, its consumers having 30 to 33 parts. A drawn founding is
     an option, chosen by kingdom. Producers get 2 to 4 stretches of 4 to 11
     segments bearing plates. Consumers get a mouth head and stretches of 1
     to 6 segments bearing limbs or feelers. Variance is 1 or 2." Options,
     recommended first: (A) "Drawn, roster as presets: the generator draws
     each lineage's recipe by its kingdom, as Mesocosm's lottery does. The
     eight authored bodies ship as founding presets, like the pressures
     (recommended)." (B) "The roster by default: as Mesocosm does now:
     lineages take the eight authored bodies, with drawing as an option."
     (C) "One minimal recipe: every lineage starts with one bare tagma of
     four segments and grows its vocabulary by incorporation." Mark chose
     A, "Drawn, roster as presets". So the generator draws each founded
     lineage's recipe by its kingdom, as Mesocosm's lottery does, and the
     eight authored bodies ship as founding presets.
     *Bounded 2026-10-03 by ruling 525:* in worlds that ask for bodies.
     *Completed 2026-10-04 by rulings 547 and 548:* each recipe reproduces
     in one cell of its root, and each lineage's life history is drawn
     within what its body allows. *And by 549 and 550:* flora and fauna
     alone draw recipes, within Mesocosm's bounds by default.

     Links: area World generation · summary The generator draws each founded lineage's recipe by its kingdom; the eight authored Mesocosm bodies ship as founding presets.

515. **A body's reproduce volume scales what a birth spends, and neither
     the budget nor the timing is hardcoded.** Asked on 2026-10-03, with
     514: "What does a body's reproduce volume scale (492, 493)? Evidence:
     Mesocosm prices a brood at a quarter of the parent's biomass (447 made
     the fraction a world rule). A body waits 480 ticks between births,
     scaled by the quarter power of its mass. Under 513 each probe body
     reproduces in one cell: about 27 to 37 voxels of a frond, 16 of a
     grazer's lump. Under 505 income and the mouthful read their
     measurements alone." Options, recommended first: (A) "How often a body
     bears: gestation is a world-rule rate per voxel of reproduce,
     calibrated so the median body waits what Mesocosm's formula gives.
     What a birth costs stays 447's world-rule fraction (recommended)." (B)
     "What a birth spends: a birth's budget is a world-rule rate per voxel
     of reproduce, calibrated to Mesocosm's quarter of the median body.
     Gestation stays Mesocosm's, by mass." (C) "A gate only: expressing
     reproduce lets a body breed. Mesocosm's fraction and gestation stand,
     and nothing reads the volume yet." Mark answered: "What a birth spends,
     but also, it's probably not wise to hardcode 1/4… nor how often.
     Sounds like a placeholder value we set". So a body's reproduce volume
     scales what a birth spends, and neither Mesocosm's quarter nor its
     gestation is taken as a fixed value; where the budget and the timing
     come from was put back the same day (518).

     Links: area Bodies and anatomy · draws on 447 · summary A body's reproduce volume scales what a birth spends, and neither the budget nor the timing is hardcoded. (amended by 518)

516. **A whole part is incorporated, as affinity decides.** Asked on
     2026-10-03, with 514: "How does a probe body incorporate (468)?
     Evidence: Mesocosm incorporates whole-body and single-part meals
     whenever the eater isn't starving, landing them at the first free box.
     Its only lexicon lesson is eating 20 mg of plate, which teaches Plate.
     Its affinity verdicts (native, adapter, refused) gate only grafts,
     while the brief puts incorporation under them. Since checkpoint 7 a
     grazer's bite lands on one part of a frond, a mouthful of about 11 mg
     against a frond's tissue of about 66 mg." Options, recommended first:
     (A) "Whole parts, by affinity: a grazer that isn't hungry, whose bite
     would take all of a part's tissue, lands the part whole at the first
     free box and teaches its lineage the kind. Affinity decides how it
     lands: native as it was, adapter expressing nothing, refused burnt as
     a meal (recommended)." (B) "Whole parts, ungated: the same landing, but
     every whole part lands as it was. Affinity is left to grafting, as in
     Mesocosm." (C) "Its own act: incorporation is a native the grazer
     chooses over grazing when not hungry. It takes a whole part, up to its
     room, gated by affinity." Mark chose A, "Whole parts, by affinity". So
     a body that isn't hungry, whose bite would take all of a part's
     tissue, lands that part whole at the first free box and teaches its
     lineage the part's kind, and Mesocosm's affinity verdict decides how
     it lands: a native part as it was, an adapter's expressing nothing,
     a refused one burnt as a meal.
     *Specified 2026-10-03 by ruling 544:* the landed part keeps its
     donor's matter.

     Links: area Bodies and anatomy · draws on 468 · summary A body that is not hungry lands a part whole at the first free box, teaching its lineage the kind; affinity decides how it lands.

518. **A birth spends the provision its reproduce cells fill.** Asked on
     2026-10-03, putting 515 back: "You chose what a birth spends, and said
     neither 1/4 nor how often should be hardcoded. Both are placeholders
     from Mesocosm's TD2 tuning: OFFSPRING_COST 4, and GESTATION_BASE 480,
     the "knob that decides boil against breathe". Where should a birth's
     budget and its timing come from? Evidence: store cells hold the
     reserve up to their cells' mass (463). Read that way, a reproduce cell
     would hold 21 to 29 mg in a frond, about 6 to 8 ticks of the median
     income (4 mg), and 12 mg in a grazer's lump. That's near Mesocosm's
     quarter of these bodies." Options, recommended first: (A) "Filled like
     a store: reproduce cells hold a provision up to their cells' mass,
     filled by growth once the parts are full, and a body bears when its
     provision is full, the birth spending it. The budget is the reproduce
     volume and the timing is what the body earns, with no fraction or
     interval; 447's fraction and gestation give way (recommended)." (B)
     "Lineage traits: the budget per voxel of reproduce and the interval
     between births are life-history traits each lineage carries. The
     generator draws them within world-rule bounds, and they're revised at
     the epoch boundary." (C) "World rules, drawn per world: a rate per
     voxel and an interval are world rules the generator draws from its
     declared space, with Mesocosm's values among them." Mark chose A,
     "Filled like a store". So a body's reproduce cells hold a provision up
     to their cells' mass, as store cells hold the reserve (463), filled by
     growth once its parts are full; the body bears when its provision is
     full, and the birth spends it. What a birth spends is the reproduce
     volume and how often it comes is what the body earns: 447's
     world-rule fraction and Mesocosm's gestation give way, and no birth
     fraction or interval is set anywhere. *Reading, not ruled:* the
     provision is a matter account of the lineage, held in its reproduce
     cells as the reserve is in its stores, and it moves by the flow
     record like any other. *Ordered 2026-10-03 by ruling 520:* whether
     growth fills the provision or a missing part first is a lineage
     trait.

     Links: area Bodies and anatomy · amends 447, 515 · summary A birth spends the provision its reproduce cells fill; the body bears when the provision is full; no birth fraction or interval is set.

519. **Hunger switches a grazer between an egg and a brood.** Asked on
     2026-10-03, checkpoint 8's third round: "Under 518 every birth spends
     the provision. What switches a grazer between laying an egg and
     brooding (514, 449)? Evidence: 514's switch read the reserve, which no
     longer pays. A brood develops the recipe's whole body from the
     provision. An egg is a minimal body that grows the rest (§6's reading).
     Water fleas brood live young while fed and lay resting eggs under
     stress. The probe already reads a grazer's hunger (TD5)." Options,
     recommended first: (A) "Hunger: a fed grazer broods and a hungry one
     lays an egg, the cheap hedge under stress, as water fleas do
     (recommended)." (B) "The provision's size: a brood when the provision
     can give every recipe part its minimum, an egg otherwise." (C) "The
     reserve still: an egg while the reserve could refill the provision, a
     brood otherwise, keeping 514's wording." Mark chose A, "Hunger". So a
     fed grazer broods and a hungry one lays an egg, the phenotype's switch
     of 449 read from the hunger TD5 already reads.
     *Specified 2026-10-04 by ruling 551:* a body whose stores are full is
     fed, TD5's horizon capped by what its stores may hold.

     Links: area Bodies and anatomy · draws on 518, 449 · summary Hunger switches a grazer between strategies: a fed grazer broods and a hungry one lays an egg.

520. **Whether growth builds a missing part or fills the provision first is
     a lineage trait.** Asked on 2026-10-03, with 519: "With its parts full,
     does growth first build a missing part (479) or fill the provision
     (518)? Evidence: both rulings take growth that finds no room left.
     Founders are founded whole, but a child from an egg starts minimal and
     lacks most of its recipe. Mesocosm matures a body by age (270 ticks for
     100 mg)." Options, recommended first: (A) "Parts first: a body grows
     toward its recipe before it provisions, so it reaches its form before
     it breeds. Maturity emerges as being whole and full, with no age
     threshold (recommended)." (B) "Provision first: a full body provisions
     before growing missing parts, so it breeds early and builds its form
     later." (C) "In proportion: growth with the parts full splits between
     the provision and the next missing part by their room." (D) "A lineage
     trait: which comes first is a lineage's life-history trait, drawn by
     the generator and revised at the epoch boundary." Mark chose D, "A
     lineage trait", against the recommendation. So whether a body with its
     parts full first grows the next missing part or first fills its
     provision is a trait of its lineage, drawn by the generator and revised
     at the epoch boundary (57). *Reading, not ruled:* a lineage that grows
     its parts first comes to breed only once whole, so its maturity
     emerges with no age threshold; one that provisions first breeds while
     still incomplete.

     Links: area Bodies and anatomy · draws on 479, 518 · summary Whether a body with full parts first grows a missing part or fills its provision is a lineage trait.

522. **Ars Magica's open text becomes a VTT ruleset and a reference for the
     wing's magic.** Raised by Mark on 2026-10-03 ("this is probably good
     shit for a ruleset", with the link
     https://github.com/OriginalMadman/Ars-Magica-Open-License), then
     asked: "Where does Ars Magica's open material go? Evidence: Atlas
     Games released the text of Ars Magica 5th Edition and its 53 books
     under CC BY-SA 4.0; the repo is a community Markdown conversion.
     Hermetic magic composes 5 Techniques with 10 Forms at levels set by
     guidelines, close to 404's fundamentals combined at rising cost. Its
     four realms' auras modify casting, as you noted for territories and
     surfaces (502). The VTT's CLAUDE.md ships only 5e SRD (CC-BY) and
     Pathfinder 2e (ORC). Share-alike: anything shipped from it is CC
     BY-SA, with Atlas's credit line." Options, recommended first: (A) "VTT
     ruleset and magic reference: the VTT gains an Ars Magica system pack
     under CC BY-SA, its CLAUDE.md content line widened by your word. Its
     magic and auras also inform the wing's design, with nothing copied
     into the sim (recommended)." (B) "A VTT ruleset only: a third system
     pack beside 5e and Pathfinder, with the wing's magic designed without
     it." (C) "Noted for later: recorded as a candidate, with no work until
     the magic family or the VTT's next system." Mark chose A, "VTT ruleset
     and magic reference". So the VTT gains an Ars Magica 5th Edition
     system pack under CC BY-SA 4.0, credited as Atlas Games' licence asks
     ("Based on the material for Ars Magica, ©1993–2024, licensed by
     Trident, Inc. d/b/a Atlas Games®, under Creative Commons
     Attribution-ShareAlike 4.0 International license 4.0"), and the VTT's
     `CLAUDE.md` content line is widened to admit it; and Hermetic magic
     and the realms' auras inform the wing's magic design (404, 502), with
     nothing copied into the sim. Mark then supplied prior art for the
     adaptation, recorded in §5.1. *Reading, not ruled:* the authoritative
     text is Atlas Games' release
     (https://www.atlas-games.com/arsmagica/openars), the community
     conversion a convenience to check against it; no pack is opened or
     scheduled by this ruling; and any lane that reads the prior art's code
     checks each project's licence first, none having been checked here.

     Links: area Licensing and sources · draws on 40, 404 · summary The VTT gains an Ars Magica 5e system pack under CC BY-SA 4.0 with Atlas credit; its magic and auras inform the wing's design.

523. **A body starves when its rent fails, not below a fixed mass.** Asked
     on 2026-10-03, checkpoint 8's fourth round, returning evidence against
     513 and 518: "Under 513 and 518 a grazer's provision is one cell, 12
     mg, and a brood or egg spends all of it. The probe starves any body
     whose tissue falls to 20 mg or less, checked every tick (Mesocosm's
     STARVATION_MG, a constant of the kind you called a placeholder). So
     every grazer child would die on its first tick. Producer buds of 21 to
     29 mg clear the line, but only just. A small grazer's rent is about 1
     mg a tick, so without the line a 12 mg child would live about 12 ticks
     unfed. What gives?" Options, recommended first: (A) "Starve when rent
     fails: a body starves when its tissue and reserve can't pay its rent,
     not below a fixed 20 mg. A small newborn lives while it can pay and
     must feed. The constant goes, like the quarter (recommended)." (B) "Two
     cells for grazers: 513 changes: grazers' recipe gives reproduce two
     cells (25 mg), clearing the line. Producers keep one, and the 20 mg
     line stays." (C) "Line scaled to the body: a body starves below a
     share of its own parts' minimum, so the line follows its size. That
     share is a world rule." Mark chose A, "Starve when rent fails". So a
     body starves when its tissue and reserve cannot pay its rent, and
     Mesocosm's fixed 20 mg line goes from the sim as its quarter did (515);
     a newborn of any size lives while it can pay, and must feed.

     Links: area Bodies and anatomy · draws on 446, 518 · summary A body starves when its tissue and reserve cannot pay its rent; the fixed 20 mg starvation line goes.

524. **A bud is a part grown off its parent.** Asked on 2026-10-03, with
     523: "What is a producer's bud under 518, which has every birth spend
     the provision (514)? Evidence: 448 described budding as "a body at its
     ceiling routes its overflow into a bud, or divides, the child a share
     of the parent". A brood draws a fresh soma from the recipe by the
     child's seed (478). Severing a subtree into a body of its own is
     already 470's fragmentation." Options, recommended first: (A) "A clone:
     a bud copies its parent's soma, the clonal strategy, while a brood
     draws a fresh one. Both spend the provision (recommended)." (B) "A part
     grown off: the bud grows as a new part at the parent's reproducing
     part, and severs into a body of its own once full, so the child is a
     share of the parent, as 448 put it." (C) "A brood by another name: a
     bud develops exactly as a brood does; the strategy only names how a
     lineage is described." Mark chose B, "A part grown off", against the
     recommendation. So a bud grows as a new part at its parent's
     reproducing part and severs into a body of its own once full, the
     child a share of its parent as 448 described it, its severing 470's.
     *Reading, not ruled:* the bud is filled as the provision is, so its
     tissue is what the provision would have held, and once severed it
     grows the rest of its lineage's recipe by 479 and 520.
     *Specified 2026-10-04 by ruling 552:* "once full" is once it holds a
     provision's worth, its parent's reproduce cells' mass.

     Links: area Bodies and anatomy · draws on 518, 470 · summary A bud grows as a part at its parent's reproducing part and severs into a body of its own once full.

525. **Only worlds that ask for bodies found their lineages from
     recipes.** Asked on 2026-10-03, with 523: "Which generated worlds
     found their lineages from recipes (512)? Evidence: Isocosm's generator
     founds the ecology family's worlds with extentless parts that carry
     traits but no matter (504 kept them as a control). Their 29 scale
     points are the wing's cost baseline, unchanged at checkpoint 7. The
     body family is founded by the probe." Options, recommended first: (A)
     "Worlds that ask for bodies: the generator draws recipes for the body
     family and any world founded with bodies. The ecology family keeps its
     extentless bodies as the scale control (recommended)." (B) "Every
     generated world: from checkpoint 8 every generated world founds recipe
     bodies. The ecology family is re-founded and its scale points measured
     again from scratch." Mark chose A, "Worlds that ask for bodies". So
     the generator draws recipes for the body family and any world founded
     with bodies, and the ecology family keeps its extentless bodies as the
     scale control.

     Links: area World generation · amends 512 · summary Only worlds that ask for bodies found lineages from recipes; the ecology family keeps extentless bodies as the scale control.

526. **Parental care feeds hungry young a parent's own mouthfuls.** Asked on
     2026-10-03, with 523: "How does parental care feed the young (450,
     built here by 521), without a fixed amount? Evidence: 450 called it
     provisioning after birth, a feeding transfer from parent to young. TD5
     already lands a meal in tissue or reserve by hunger, and under 505 a
     grazer's mouthful is read from its intake volume." Options,
     recommended first: (A) "Mouthfuls while young hunger: a parent with
     the trait gives a hungry child at its site one of its own mouthfuls a
     tick from its reserve, landed by TD5, while the parent itself isn't
     hungry. In the probe, grazers care (recommended)." (B) "The provision
     flows on: after a birth the parent routes its provision's refill to
     its young until they're whole, instead of toward its next birth." (C)
     "An endowment at birth: care is a reserve endowment the child takes at
     birth, as 447 first described it, with no feeding afterwards." Mark
     chose A, "Mouthfuls while young hunger". So a parent whose lineage
     cares gives a hungry child of its at its site one of its own mouthfuls
     a tick from its reserve, landed by TD5, while the parent itself is not
     hungry, and the probe's grazers care. *Put back the same day:* Mark
     asked "Mouthfuls, or provisions from the parent (like milk?)", and
     ruling 527 makes the food a lineage's trait.
     *Specified 2026-10-04 by ruling 554:* care reads only unweaned young,
     a young weaned the first time it is fed, its stores full; the crowd
     runs a parent and its unweaned young as individuals.

     Links: area Bodies and anatomy · draws on 450, 518 · summary A caring parent gives a hungry child one of its own mouthfuls a tick from its reserve while the parent is not hungry. (amended by 527)

527. **What a caring parent gives, milk or mouthfuls, is a lineage trait.**
     Asked on 2026-10-03, putting 526 back at Mark's question ("Mouthfuls,
     or provisions from the parent (like milk?)"): "You asked: mouthfuls,
     or provisions from the parent, like milk? What does a caring parent
     give its hungry young? Evidence: 526's mouthful is regurgitation, the
     parent's own mouthful (read from its intake volume) from its reserve,
     as birds feed nestlings. Milk would be the reproductive provision 518
     fills in the reproduce cells: after a birth it would flow to the young
     instead of toward the next birth, sized by the parent's reproduce
     volume. That would make care cost the parent births, as nursing delays
     breeding in mammals." Options, recommended first: (A) "Milk from the
     provision: while its young hunger, a caring parent's refilling
     provision goes to them rather than to its next birth, sized by its
     reproduce volume. Care trades against breeding. The probe's grazers
     give milk (recommended)." (B) "Mouthfuls, as ruled: regurgitated food:
     the parent's own mouthful from its reserve, sized by its intake
     volume. Care costs the parent's reserve, not its births." (C) "Both,
     by lineage trait: which a lineage gives, milk or mouthfuls, is its
     trait, as mammals and birds differ. The probe's grazers give one; the
     other is tested, not certified." Mark chose C, "Both, by lineage
     trait". So a caring lineage feeds its hungry young either milk, its
     refilling provision going to them rather than to its next birth and
     sized by its reproduce volume, or mouthfuls, 526's regurgitated food
     from its reserve sized by its intake volume, as its trait says; the
     probe's grazers give one, certified, and the other is tested. Which
     one the probe's grazers give goes back to Mark. *Answered 2026-10-03
     by ruling 528:* milk.

     Links: area Bodies and anatomy · amends 526 · summary What a caring parent gives its hungry young, milk or mouthfuls, is a lineage trait.

530. **How many eggs a birth lays is a lineage trait.** Asked on
     2026-10-03, with 528: "How many eggs does a birth lay (448: many
     offspring, most lost)? Evidence: under 518 an egg spends the
     provision, 12 mg in a grazer. An egg is a minimal body, the recipe's
     root, which for a grazer is its lump. Under 523 an egg lives while it
     can pay its rent, about 1 mg a tick at that size." Options, recommended
     first: (A) "One egg a birth: an egg takes the whole provision. It
     differs from a brood by its minimal body, and many offspring come from
     how often a body lays (recommended)." (B) "Clutch by lineage trait: how
     many eggs a birth splits its provision into is a lineage trait, drawn
     by the generator." (C) "As many as it can make: the provision is split
     into as many eggs as can each take one milligram per part. Most
     starve." Mark chose B, "Clutch by lineage trait", against the
     recommendation. So how many eggs a birth splits its provision into is
     a trait of the lineage, drawn by the generator and revised at the
     epoch boundary (57). *Reading, not ruled:* the provision is split among
     the clutch as a take is (464), units left over to the largest
     remainders, and the probe draws its grazers' clutch per world as it
     draws their other traits.

     Links: area Bodies and anatomy · draws on 518 · summary How many eggs a birth splits its provision into is a lineage trait, drawn by the generator and revised at the epoch boundary.

531. **A recipe carries its absence odds beside its variance.** Asked on
     2026-10-03, with 528: "How much do children vary, without Mesocosm's
     fixed odds? Evidence: Mesocosm's soma drifts each tagma's segment
     count by up to the recipe's variance (1 or 2). It leaves one segment's
     appendages absent at odds of 1 in 12 per tagma, a fixed constant ("the
     cheapest evidence that individuals are not clones"); mouths, feelers
     and canopy plates are never left absent. The probe's bodies are one
     frond, or a lump with 1 or 2 limbs and an eye." Options, recommended
     first: (A) "Odds in the recipe: each recipe carries its absence odds
     beside its variance, drawn by the generator with the recipe, with
     Mesocosm's 1 in 12 among the draws. The probe draws both per world, at
     zero in 513's control (recommended)." (B) "A world rule: the absence
     odds are one world rule, drawn per world, and each recipe keeps only
     its variance." (C) "No absences: children vary only by their segment
     counts; every appendage the recipe names develops." Mark chose A,
     "Odds in the recipe". So each recipe carries the odds that a
     segment's appendages develop absent beside its segment variance, both
     drawn by the generator with the recipe, Mesocosm's 1 in 12 among the
     draws, and the probe draws both per world, at zero in 513's control.
     *Reading, not ruled:* the kinds Mesocosm never leaves absent, the
     feeding, sensing and canopy organs, become those whose templates hold
     intake, sense or fix, so no child is born unable to feed.
     *Extended 2026-10-05 by rulings 577 to 579:* a child also takes its
     parent's varied cells, and one of its cells may vary to any grown
     function at odds its recipe draws.

     Links: area Bodies and anatomy · draws on 478 · summary Each recipe carries its appendage absence odds beside its segment variance, both drawn by the generator with the recipe.

532. **One stack helper runs the wasm constructors once, for every web
     module.** Asked on 2026-10-03 in the Burn migration session, after
     509's lane: the raw pre.4 module's `__wasm_call_ctors` calls 8,166
     `inventory` constructors expanded from Pliron's macros in five crates,
     and wasm-ld wraps every export in a call to them, 5,441 runs by the
     time the page is ready; graphshell-web's start function now calls them
     once, 1 run, the link line unchanged. "graphshell-web's start function
     calls `__wasm_call_ctors` (one run per page, link line unchanged);
     three other pre.4 web modules still run their constructors on every
     call, and Knot's and Isometry's web builds will once they take pre.4."
     Options: "one helper in a stack crate, called first from every web
     module's start, with a run-once guard and a test counting exactly one
     constructor run"; "the per-module start call as committed"; "the link
     arg plus a post-bindgen glue edit". Mark's answer, as mere's plan
     records it: "Shared stack helper". So one helper in a stack crate runs
     the constructors once, called first from every web module's start,
     with a run-once guard and a test counting exactly one run. *Reading,
     not ruled:* recommended because a second run, say from a future
     wasm-bindgen that also calls the constructors, would make each
     `inventory` node point at itself so that iterating a registry never
     ends, which the one-run test guards; and Isometry's web build calls
     the same helper when it takes pre.4.

     Links: area Stack and engine · summary One stack helper runs the wasm constructors once, called first from every web module's start, with a run-once guard and test.

535. **Whether a lineage breeds from income or from capital is its trait.**
     Asked on 2026-10-03, building checkpoint 8's step 8c: "Where do the
     provision (518) and a missing part (479) take growth, against the
     reserve? Evidence: TD5 lands a meal in tissue then reserve when fed,
     reserve then tissue when hungry, and spills the rest to the ground.
     479 and 518 both take "growth that finds no room left in the body's
     parts". Producers keep no reserve (506), so it changes only bodies
     that store. A grazer's reserve holds 12 to 48 mg, its provision 12 mg;
     under 519 a hungry grazer lays eggs." Options, recommended first: (A)
     "Before the reserve: with the parts full, growth goes to the missing
     part or the provision (in the lineage's order, 520) before the
     reserve, as 479's and 518's words say. A body breeds from income and
     stores what's left (recommended)." (B) "After the reserve: TD5 stays as
     it is, and only what it would spill goes to the missing part or the
     provision. A body stores first and breeds from surplus." (C) "A
     lineage trait: whether a lineage breeds from income or from capital is
     its trait, drawn by the generator, as nature has both." Mark chose C,
     "A lineage trait", against the recommendation. So whether growth with
     the parts full goes to the missing part or the provision before the
     reserve, breeding from income, or after it, breeding from capital, is
     a trait of the lineage, drawn by the generator and revised at the
     epoch boundary (57).

     Links: area Bodies and anatomy · draws on 520 · summary Whether a lineage's growth goes to missing part or provision before the reserve (income) or after it (capital) is a lineage trait.

536. **The constructor helper lives in cambium-genet-web-host.** Asked on
     2026-10-03 in the Burn migration session, carrying out 532: where the
     shared constructor helper lives. "Today only two pre.4 web modules
     exist: graphshell-web, and Distillery's model probe, which has no start
     function. The only mere crates in both wasm graphs are esp, eidetic and
     muniment (inference, memory, byte storage). cambium-genet-web-host is
     the existing web-boundary crate: graphshell-web uses it, but it would
     add about 159 packages to the probe's 273." Options, recommended
     first: (A) "A new zero-dependency crate at the web boundary, holding
     the helper, its run-once guard and the one-run test. Knot or Isometry
     could take it later without Cambium's stack (recommended)." (B)
     "cambium-genet-web-host. No new crate, but the probe takes ~159 more
     packages, and every future web module needs Cambium's web host." (C)
     "esp. Already in both graphs, but inference is the wrong job for it."
     Mark's answer, as the migration session relays it: "cambium-genet-web-host",
     against the recommendation. So the helper lives in
     cambium-genet-web-host: graphshell-web calls it from its start
     function, Distillery's model probe takes cambium-genet-web-host as a
     dependency to call it, and any future web module, Isometry's
     included, reaches it through Cambium's web host.
     *Pinned 2026-10-03 by ruling 537:* the probe could not share the web
     host's `wasm-bindgen`, and the modules take the newest they can.
     *Pinned exactly 2026-10-03 by ruling 545:* wasm-bindgen 0.2.129 with
     wgpu 30.0.1.

     Source: relayed by the Burn migration session ("Conatus, physics,
     seiche status") on 2026-10-03, to be recorded in mere's burn 0.22
     migration plan §13.35 on branch `burn-pre4-repin`; not yet committed
     there when this record took it, at mere `9f5a73f6`. Mere's migration
     lane owns the work.

     Links: area Stack and engine · draws on 532 · summary The shared wasm constructor helper lives in cambium-genet-web-host.

537. **The web modules take the newest wasm-bindgen they can.** Asked on
     2026-10-03 in the Burn migration session, when ruling 536 met a pin
     conflict: Distillery's model probe pins `wasm-bindgen = "=0.2.122"`
     because 0.2.123 and later turn wgpu 30's successful null
     `popErrorScope` into a GPU error, while cambium-genet-web-host and
     graphshell-web pin `=0.2.127`, and Cargo allows one 0.2.x per graph;
     graphshell-web already runs 0.2.127 on wgpu 30.0.0 with its receipts
     passing. Options: (A) "put cambium-genet-web-host's browser
     dependencies behind a default feature, so the probe takes only the
     helper with `default-features = false`"; (B) "move the probe to the
     0.2.127 family (about 159 more packages), re-proving its browser rows
     against the popErrorScope break"; (C) "the probe keeps its own local
     start function, against 536". Mark's answer, as the migration session
     relays it: "Take the newest ya can". So the probe, cambium-genet-web-host
     and graphshell-web take the newest wasm-bindgen that works for them.
     *The migration session's reading, not ruled:* the newest that works,
     shared as one exact pin by the probe, the web host and graphshell-web,
     each step back from the newest needing evidence: option B at the
     newest version. *Reading, not ruled:* Isometry's web build, reaching
     the helper through Cambium's web host (536), sits on the same
     wasm-bindgen.

     Source: the conflict and the options are in mere's burn 0.22
     migration plan §13.35, at `dd9819c4` on branch `burn-pre4-repin`; Mark's
     answer was relayed by the migration session on 2026-10-03, not yet
     recorded there. Mere's migration lane owns the work.
     *Widened 2026-10-04 by ruling 556:* every wasm module, and the newest
     wasm-bindgen as a standing posture.

     Links: area Stack and engine · amends 536 · summary The web modules take the newest wasm-bindgen they can, shared as one exact pin. (amended by 556)

538. **Each world chooses whether the sim or its ruleset leads.** Raised by
     Mark on 2026-10-03 with research into rulesets over the sim ("I guess
     the gold standard would be successfully implementing d&d, pathfinder,
     ars magica atop the sim, multiple versions, as close to the standard
     raw as possible"), then asked: "Who governs where a published ruleset
     and the sim disagree? Ruling 114 ("Rulesets calibrate"): with no
     player choices, a ruleset reading the ledger must match the sim's own
     background outcomes, which bends RAW wherever the sim models things
     differently. D&D restores all hit points on a long rest, and
     Pathfinder orders fortune and degree adjustments exactly, while the
     sim heals tissue by its body model. Faithful editions, your gold
     standard, need the opposite direction. The ledger, the record and the
     derivation rule stay inviolable either way (41)." Options, recommended
     first: (A) "The world's profile governs: a world declares its active
     mechanics profile. Where the profile specifies behaviour, foreground
     and background both obey it, the background simplifying only as far
     as outcomes survive. The sim supplies whatever the profile leaves
     unsaid, and its own model is the native profile. This reverses 114
     (recommended)." (B) "Rulesets calibrate, as ruled: 114 stands:
     editions are faithful where they agree with the sim and adapted where
     it differs, and an edition that can't pass plays in experimental mode
     (189)." (C) "Each world chooses: a world setting picks the direction:
     the sim leads and rulesets calibrate, or the ruleset leads and the
     sim's background follows it." Mark chose C, "Each world chooses",
     against the recommendation. So the direction of authority is a world
     setting: in a world the sim leads, rulesets calibrate to its
     background as 114 has them; in a world its ruleset leads, the sim's
     background follows that ruleset wherever it specifies behaviour. The
     ledger, the record and the derivation rule stay inviolable in both
     (41). *Reading, not ruled:* a ruleset-led world's background owes the
     ruleset's own procedures, simplified only as far as their outcomes
     survive, and 189's experimental mode stays for sim-led worlds.
     *Specified 2026-10-03 by rulings 542 and 543:* an edition's own state
     persists beside the body, its relations declared, and unattended its
     procedures run under declared decision policies.

     Links: area The sim's schema and saves · amends 114 · summary Each world chooses whether the sim leads and rulesets calibrate, or the ruleset leads and the sim's background follows; the ledger and record stay inviolable.

539. **Rules are built as faithful editions, a shared toolkit grown from
     them.** Asked on 2026-10-03, with 538: "What do we build for rules?
     Evidence: editions differ in sequencing, interruptions, stacking,
     recovery, advancement and downtime, and RAW leaves decisions to the
     GM. Foundry's Pathfinder 2e module builds abilities from small "rule
     elements" (selector, predicate, value, an inspector). Datasworn carries
     Ironsworn as portable JSON. A house ruleset starts once a framework
     prescribes choices: six attributes, hit points, one success ladder."
     Options, recommended first: (A) "Editions, toolkit grown: implement
     recognized editions faithfully as versioned rulesets (Pathfinder 2e,
     5e SRD versions, Ars Magica 5e), preserving the GM's decision points
     and recording their rulings. A shared toolkit (checks, costs, choices,
     modifiers, effects, durations, triggers) is extracted from what
     repeats. A house ruleset only if experiments show a need
     (recommended)." (B) "Toolkit first: design the cross-RPG kernel up
     front from §5.1's taxonomy, then write each edition on it." (C) "A
     house ruleset too: an Isocosm-native tabletop ruleset alongside the
     editions, built on the toolkit and using the sim fully." Mark chose A,
     "Editions, toolkit grown". So recognized editions are implemented
     faithfully as versioned rulesets, the GM's decision points kept and
     their rulings recorded; the shared rules toolkit is extracted from
     what the editions repeat; and a house ruleset is made only if those
     experiments show a need.

     Links: area The VTT · draws on 538 · summary Rules are built as faithful versioned editions with GM decision points kept; a shared toolkit is extracted from what repeats; house ruleset only if needed.

540. **Rulesets mix freely in content and subsystems, and meet in an
     encounter only under a named hybrid profile.** Asked on 2026-10-03,
     with 538: "How far can rulesets mix? Evidence: sharing content
     (creatures, places, histories) costs little. Combining subsystems (Ars
     Magica's seasonal lab beside Pathfinder's combat) needs explicit
     connections. A Pathfinder character fighting an Ars Magica one needs
     decisions on whose defenses apply, how damage crosses, how turns
     relate, and which exception wins: a hybrid design either way."
     Options, recommended first: (A) "Hybrids by named profile: content and
     subsystems combine freely. A cross-system encounter runs under a named
     hybrid profile that documents each crossing and where it departs from
     either edition (recommended)." (B) "Content only: worlds share content
     across rulesets, but each game runs under one ruleset." (C) "Seamless
     as the goal: the toolkit aims to make any two editions meet without a
     named hybrid." Mark chose A, "Hybrids by named profile". So content
     and subsystems combine freely across rulesets, and a cross-system
     encounter runs under a named hybrid profile that documents each
     crossing and each departure from either edition.

     Links: area The VTT · draws on 538 · summary Rulesets mix freely in content and subsystems; a cross-system encounter runs under a named hybrid profile documenting each crossing.

541. **The VTT plays faithful editions; Eponym and Mesocosm keep their own
     mechanics.** Asked on 2026-10-03, with 538: "How do the products
     relate to the rulesets? Evidence: the overlay contract (154) already
     has games submit intents and take back events. The VTT adjudicates at
     a table. Eponym's embodied control and contact resolution are
     mechanics of its own. Mesocosm's ecology, directing and epochs are
     too. The Year Zero Engine licence admits VTT modules but excludes
     video games." Options, recommended first: (A) "VTT faithful, others
     native: the VTT hosts faithful editions. Eponym and Mesocosm keep
     native mechanics, sharing the toolkit's effects and procedures, and a
     published ruleset may govern Eponym's progression, magic or costs as a
     selectable mode (recommended)." (B) "One ruleset for all: a world's
     selected ruleset governs every product that plays it, Eponym and
     Mesocosm included." (C) "VTT only: rulesets are the VTT's alone, and
     Eponym and Mesocosm stay as designed." Mark chose A, "VTT faithful,
     others native". So the VTT hosts faithful editions; Eponym and
     Mesocosm keep their native mechanics and share the toolkit's effects
     and procedures; and a published ruleset may govern Eponym's
     progression, magic or costs as a selectable mode.

     Links: area Processes and rules · draws on 154, 538 · summary The VTT hosts faithful editions; Eponym and Mesocosm keep native mechanics, sharing the toolkit's effects, with a ruleset as selectable mode.

542. **An edition's own state persists beside the body, its relations to it
     declared.** Asked on 2026-10-03, following 538: "In a world its
     ruleset leads (538), where do the edition's own facts live: hit
     points, spell slots, death-save failures, Ars Magica's seasonal
     advancement? Evidence: they can't all be regenerated from the body.
     The sim's body holds tissue in parts, and D&D hit points return on a
     rest whatever the tissue. The overlay contract (154) hands outcomes
     back through checked effects on the ledger, and the ledger, the record
     and the derivation rule stay inviolable (41)." Options, recommended
     first: (A) "Native state, related: edition state persists in its own
     right beside the body. Each fact is marked as a world fact, native
     state or a derived display, and the ruleset declares how they relate:
     what a wound to hit points does to the body, what death in either
     does to the other (recommended)." (B) "Derived from the body: edition
     state is read from the sim's body and ledger on demand (hit points
     from tissue, say), and nothing native persists." (C) "Native,
     unrelated: edition state persists apart from the body, which the
     edition ignores, so the two never constrain each other." Mark chose A,
     "Native state, related". So an edition's own state persists in its own
     right beside the body; each fact is marked a world fact, native state
     or a derived display; and the ruleset declares how they relate, what a
     wound to hit points does to the body and what death in either does to
     the other.

     Links: area The VTT · draws on 538, 41, 154 · summary An edition's own state persists beside the body, each fact marked world fact, native state or derived display, with the ruleset declaring their relations.

543. **Unattended, a ruleset-led world runs the ruleset's procedures under
     declared decision policies.** Asked on 2026-10-03, with 542: "With
     nobody playing, how does a ruleset-led world's background resolve
     what the ruleset specifies, a battle say? Evidence: Ceptre runs a
     procedure unattended once each choice is replaced by a stated
     strategy. Icepool shows a reroll's distribution depends on its policy
     (always push, or push when failure is dire). RAW leaves some decisions
     to the GM, which the VTT keeps for the table (539)." Options,
     recommended first: (A) "Procedures with policies: the ruleset's own
     procedures run, each choice a player or GM would make replaced by a
     declared decision policy for that kind of actor. They're simplified
     only as far as the bench shows the outcomes survive (recommended)."
     (B) "Sim calibrated to it: the sim's background model stays,
     calibrated to match the ruleset's outcome distribution on the bench:
     114 run in reverse." (C) "Unattended waits: what the ruleset specifies
     isn't resolved unattended; it waits for play, and the sim advances
     only what the ruleset leaves unsaid." Mark chose A, "Procedures with
     policies". So with nobody playing, a ruleset-led world's background
     runs the ruleset's own procedures, each choice a player or GM would
     make replaced by a declared decision policy for that kind of actor,
     simplified only as far as the bench shows the outcomes survive.

     Links: area The VTT · draws on 542, 539 · summary Unattended, a ruleset-led world runs the ruleset's own procedures with each player or GM choice replaced by a declared decision policy.

544. **An incorporated part keeps its donor's matter.** Asked on 2026-10-03,
     building checkpoint 8's step 8f: "When a grazer incorporates a whole
     frond (516), whose matter does the frond hold? Evidence: 504 keys a
     part's ledger by provenance, as the scruple is, and Mesocosm keeps an
     incorporated part's donor provenance (Origin::Incorporated). Rent,
     growth, starvation (523) and the provision read only a body's own
     lineage's matter. A frond of 4 to 9 cells holds about 66 mg of the
     producer's tissue." Options, recommended first: (A) "The donor's,
     kept: the frond keeps the producer's tissue under its provenance. The
     eater can't spend it on rent or grow from it, and the frond's room for
     the eater's own tissue counts what it already holds. Kleptoplasty as
     Mesocosm keeps it (recommended)." (B) "Digested in place: landing
     converts the frond's tissue to the eater's own, so the part is the
     eater's in every way but its kind." (C) "The donor's, digested over
     time: the frond keeps the donor's tissue, and rent and growth turn it
     into the eater's own over the following ticks." Mark chose A, "The
     donor's, kept". So an incorporated part keeps its donor's matter under
     its provenance; the eater can neither spend it on rent nor grow from
     it; and the part's room for the eater's own tissue counts the matter
     it already holds.

     Links: area Bodies and anatomy · draws on 504, 516 · summary An incorporated part keeps its donor's matter under its provenance; the eater can neither spend it on rent nor grow from it.

547. **A generated recipe reproduces in one cell of its root.** Asked on
     2026-10-04, building checkpoint 8's step 8g: "Where do generated
     recipes express reproduce (449)? Evidence: Mesocosm's seeding gives
     every cell of a part to its role's one process: a lump takes in, a rod
     contracts, a point senses, a sheet fixes. So no drawn or roster recipe
     would reproduce. For the probe, 513 gave reproduce one cell of the
     frond or lump. A body may use a strategy only while it expresses
     reproduce." Options, recommended first: (A) "One cell of the root: the
     generator gives reproduce one cell of each recipe's root segment
     kind, as 513 did for the probe (recommended)." (B) "An organ of its
     own: each recipe bears a reproductive lump on one segment, a kind of
     its own drawn with the recipe." (C) "Drawn among its kinds: the
     generator draws which of the recipe's kinds gives reproduce a cell."
     Mark chose A, "One cell of the root". So the generator gives
     reproduce one cell of each recipe's root segment kind, drawn and
     authored recipes alike, as 513 did for the probe. *Reading, not
     ruled:* the default templates otherwise follow Mesocosm's seeding, all
     of a part's cells to its role's process, so a lump takes in, a rod
     contracts, a point senses and a sheet fixes (511).

     Links: area World generation · draws on 512 · summary The generator gives reproduce one cell of each recipe's root segment kind, drawn and authored recipes alike.

548. **A founded lineage's life history is drawn, within what its body
     allows.** Asked on 2026-10-04, with 547: "How does the generator draw
     each founded lineage's life history? Evidence: a lineage now carries
     its strategies (449), whether it grows parts or provision first
     (520), whether it breeds from income or capital (535), its clutch
     (530), semelparity (521), its care, milk or mouthfuls (527), and
     whether it grows segments (495). A body can lay eggs only with a
     reserve, and can bud only where it reproduces. You've ruled against
     hardcoded placeholder values (515)." Options, recommended first: (A)
     "Drawn, within what bodies allow: each trait is drawn by the founding
     seed: strategies among those the lineage's body supports, clutch
     within a world-rule bound, the others either way. The probe sets its
     own (recommended)." (B) "Kingdom presets: producers bud, consumers
     brood or lay eggs, decomposers bud; the rest are drawn." (C) "One
     default history: every founded lineage takes one life history (brood,
     iteroparous, epimorphic, parts first, income, no care), varied only by
     authoring." Mark chose A, "Drawn, within what bodies allow". So the
     generator draws each founded lineage's life history by the founding
     seed: its strategies among those its body supports, its clutch within
     a world-rule bound, and each of its other traits either way; the
     probe sets its own.

     Links: area World generation · draws on 512 · summary The generator draws each founded lineage's life history by the founding seed, within what its body supports; the probe sets its own.

549. **Flora and fauna draw recipes; myco and micro wait for territories
     and surfaces.** Asked on 2026-10-04, with 547: "In a world that asks
     for bodies, which kingdoms get recipes (512, 525)? Evidence: Isocosm
     founds flora, fauna, myco and micro lineages. Mesocosm's lottery draws
     producers (rooted, bearing plates), consumers (a mouth head, limbs and
     feelers) and decomposers (crusts, detritivores). Ruling 488 made a
     mold a territory and micro life a surface, which checkpoint 11
     builds." Options, recommended first: (A) "Flora and fauna: flora draws
     as Mesocosm's producers and fauna as its consumers. Myco and micro
     stay bodiless until checkpoint 11 makes them territories and surfaces
     (488) (recommended)." (B) "Myco as decomposers too: flora as
     producers, fauna as consumers, and myco as Mesocosm's decomposers with
     bodies of their own. Micro waits for surfaces." (C) "All four: every
     kingdom draws a recipe now, micro included, and checkpoint 11
     revisits myco and micro." Mark chose A, "Flora and fauna". So in a
     world that asks for bodies, flora lineages draw recipes as
     Mesocosm's producers and fauna as its consumers, and myco and micro
     lineages stay bodiless until checkpoint 11 makes them territories and
     surfaces.

     Links: area World generation · draws on 512, 488 · summary In worlds that ask for bodies, flora and fauna draw recipes; myco and micro stay bodiless until territories and surfaces exist.

550. **The generator draws within Mesocosm's bounds where it has them.**
     Asked on 2026-10-04, with 547: "What bounds does the generator draw
     within, by default? Evidence: 548 draws a lineage's clutch within a
     world-rule bound, and 531 draws its absence odds "with Mesocosm's 1 in
     12 among the draws". Mesocosm has no clutch at all, and its variance
     is 1 or 2. Any default is a value someone sets, and a world may change
     it." Options, recommended first: (A) "Mesocosm's where it has one:
     variance 1 or 2 and absence odds 1 in 12 by default, each a range a
     world may widen. The clutch draws 1 to 4 by default, a starting range
     a world may widen (recommended)." (B) "Wide by default: defaults span
     much more than Mesocosm does (variance 0 to 3, absence never to 1 in
     4, clutch 1 to 12), so worlds start varied." (C) "No defaults: a world
     asking for bodies must state each bound, and the generator refuses
     one that doesn't." Mark chose A, "Mesocosm's where it has one". So the
     generator draws a recipe's variance from 1 to 2 and its absence odds
     at 1 in 12, and a lineage's clutch from 1 to 4, by default, each a
     range the founding states and a world may widen.

     Links: area World generation · draws on 548 · summary The generator draws within Mesocosm's bounds by default: variance 1 to 2, absence odds 1 in 12, clutch 1 to 4, each widenable.

551. **Full stores count as fed.** Asked on 2026-10-04, building checkpoint
     8's step 8h: "Under TD5, when is a probe grazer fed? Evidence: TD5
     calls a body hungry while its reserve holds fewer than 100 ticks of
     rent. Since 463 a reserve sits only in store cells, up to their mass.
     Across 300 founded worlds, no grazer of 712 can ever hold 100 ticks:
     the median store holds 36 mg against 4 mg a tick of rent (9 ticks),
     and the largest reaches 48% of the horizon. In 12 measured runs of
     400 ticks, every grazer birth was a clutch of eggs and milk never
     flowed. As built, 519's brood, 526 to 528's milk (which needs a fed
     parent) and 516's whole meals can never happen, so 514's check can't
     certify them." Options, recommended first: (A) "Full stores count as
     fed: a body is hungry while its reserve is below the lesser of TD5's
     100 ticks of rent and what its stores can hold. Mesocosm's horizon
     stands wherever a store can reach it; the probe's grazers are fed
     once their stores fill. No new number. Producers store nothing, so
     they count as fed, which changes nothing they do (recommended)." (B)
     "Horizon a world rule: TD5's 100 ticks becomes a world rule,
     Mesocosm's 100 by default. The probe draws its horizon per world
     within what its stores hold (a median 9 ticks), so the check sees both
     fed and hungry grazers." (C) "Bigger stores: the probe's grazers get a
     store that can hold 100 ticks of rent: about 400 mg, or 32 store cells
     at 12.5 mg. This changes 506's bodies." Mark chose A, "Full stores
     count as fed". So a body is hungry while its reserve is below the
     lesser of TD5's 100 ticks of rent and what its stores may hold:
     Mesocosm's horizon stands wherever a store can reach it, a body whose
     stores are full is fed, and a body that stores nothing counts as fed.

     Links: area Bodies and anatomy · draws on 463 · summary A body is hungry while its reserve is below the lesser of 100 ticks of rent and its stores' capacity; full stores count as fed.

552. **A bud severs once it holds a provision's worth.** Asked on
     2026-10-04, building checkpoint 8's step 8h: "When is a producer's bud
     full enough to sever (524)? Evidence: 524 has a bud sever 'once full',
     and its recorded reading says its tissue is 'what the provision would
     have held'. Built, it severs only at its frond's adult mass, 151 to
     453 mg, filled 21 to 63 mg a provision. Measured that way, no bud
     severed in 100 worlds at 60 ticks, nor in 20 grazer-free worlds at 300
     ticks; with five times the soil, 2 of 30 severed, at tick 88.
     Prototyped at a provision's worth, every bud severs on its first pour:
     buds are born in 8 of 100 worlds at 60 ticks, and 31 of 100 with five
     times the soil." Options, recommended first: (A) "A provision's worth:
     a bud severs once it holds what its parent's reproduce cells hold: a
     seedling that grows the rest of its frond and recipe alone (479, 520),
     as 524's reading has its tissue (recommended)." (B) "Its frond's
     ceiling: as built: a bud severs only at its kind's adult mass, filled
     over many provisions and the parent's income, so buds rarely complete
     within a run." Mark chose A, "A provision's worth". So a bud severs
     once its tissue reaches what its parent's reproduce cells hold, or its
     kind's adult mass if that is less, and the severed seedling grows the
     rest of its frond and recipe by 479 and 520.

     Links: area Bodies and anatomy · draws on 524 · summary A bud severs once its tissue reaches what its parent's reproduce cells hold, or its kind's adult mass if less.

554. **Bonds run individually until the young is weaned.** Asked on
     2026-10-04, opening checkpoint 8's step 8i: "How does the crowd run
     milk, which 526 gives a parent's own hungry child? Evidence: the crowd
     keeps counts of identical states and no identities (113: 'who' is
     undefined for the fungible; anything of note runs as an individual).
     Nursing names its target through the sim:child relation, so today the
     crowd refuses it. Over 100 exact runs at 60 ticks, milk flowed 5,500
     times. A median 48% of living grazers (quartiles 0% and 79%) were in a
     bond with a still-hungry young, and 74% with any living young. A
     member put back into a bin loses its identity, so a bond can end in
     the crowd only where the rule ends it." Options, recommended first:
     (A) "Bonds run individually: the crowd lifts a parent and its young
     out as individuals, relation kept, until the young is weaned, i.e.
     first fed with its stores full; nursing reads only unweaned young, in
     both runners. About half of grazers run member by member. It's the
     mechanism relations will need wherever the wing has them
     (recommended)." (B) "Kin care: amend 526: a fed parent nurses any
     hungry young of its lineage at its site, paired by count in pass order
     as a hunt pairs hunters with prey, in both runners. No relation is
     read and the crowd stays a crowd, but 'a child of its' goes." (C)
     "Milk out of the crowd: the crowd's check runs the probe without milk,
     and milk is certified on the exact runner alone." Mark chose A, "Bonds
     run individually". So a young is weaned the first time it is fed, its
     stores full, and nursing reads only unweaned young, in both runners;
     and the crowd runs a parent and its unweaned young as individuals,
     their relation kept, returning them to its counts once the young is
     weaned, the mechanism by which the crowd will hold any relation.

     Links: area Processes and rules · draws on 113 · summary A parent and its unweaned young run as individuals in the crowd until the young is weaned; nursing reads only unweaned young.

556. **Every wasm module moves to the newest wasm-bindgen, 0.2.129.**
     Asked on 2026-10-04 in the Burn migration session, widening 537 and
     545: the muniment OPFS probe's page-error gate control is unproven.
     Proving it needs the wasm-bindgen 0.2.126 command-line tool, which is
     not on this machine and which no ruling covers downloading, while its
     siblings all moved to the newest wasm-bindgen under 537. Options, as
     the migration session relays them: move it to 0.2.129; install the
     0.2.126 CLI; leave it unproven. Mark's answer, as the migration session
     relays it: "move it and anything else to 0.2.129. let's stay with the
     newest." So every wasm module in mere moves to wasm-bindgen 0.2.129,
     the OPFS probe and the two minimal repros among them, the repros still
     without the constructor helper (533). *Reading, not ruled:* "let's
     stay with the newest" stands for later pins too, and Isometry's web
     build, on the same pin through Cambium's web host, follows it.

     Source: the finding and the options are in mere's burn 0.22 migration
     plan, "Returned as forks", on branch `burn-pre4-repin` (its working
     copy after `045c2f60`), where the options are listed as approving the
     CLI install, leaving the gate unproven, or moving the probe to
     0.2.129; Mark's answer was relayed by the migration session on
     2026-10-04, for mere's plan to record.

     Links: area Stack and engine · amends 537 · summary Every wasm module in mere moves to wasm-bindgen 0.2.129; 'stay with the newest' stands for later pins.

560. **Every cell carries along a system's route, and conduits more.** Asked
     on 2026-10-05, opening checkpoint 9: "Which cells carry along a
     system's route (477)? Evidence: 477 takes a route's capacity from the
     cells of the parts between. The catalogue gives conduct (tubes,
     branches) wing-functions' edges as its mechanic, measured by
     cross-section. No probe part expresses conduct, gate or circulate:
     fronds are 4 to 9 cells, a lump 8, limbs 2 to 3, the eye 1." Options,
     recommended first: (A) "Every cell, conduits more: each cell of a part
     on the route carries a world rule's capacity, and a conduct cell
     carries a multiple of it. Conduits widen routes without being
     required, and the probe's bodies keep their recipes (recommended)."
     (B) "Only conduct cells: a route runs only through parts expressing
     conduct. The probe's recipes gain a conducting part between root and
     limbs, and 505's rates are calibrated a third time." (C) "By
     cross-section: each part carries by its cross-section, conduct's
     measure, whatever it expresses: fat parts carry more than thin ones,
     function aside." Mark chose A, "Every cell, conduits more". So each
     cell of a part on a route carries a capacity the world's rules set,
     a conduct cell a multiple of it, and the probe's bodies keep their
     recipes.
     *Specified 2026-10-05 by ruling 565:* a conduct cell's multiple is its
     part's cross-section over the reference segment's.

     Links: area Bodies and anatomy · draws on 477 · summary Each cell of a part on a route carries a world-rule capacity and a conduct cell a multiple of it; probe recipes unchanged.

561. **An intact body carries all its natives ask.** Asked on 2026-10-05,
     with 560: "How much does a route carry? Evidence: an intact founded
     body asks 2 to 16 mg a tick to fix (median 5) and 2 to 54 to graze
     (median 12, 90% under 30), through parts of 1 to 9 cells. The brief's
     reading sets a cell's capacity so an intact body carries all its
     natives ask. Wounds, which narrow routes, come in checkpoint 10."
     Options, recommended first: (A) "Intact carries all: a per-cell world
     rule, set so every intact founded body reads full degree. Until
     checkpoint 10's wounds, only a severing, an absence or a missing
     source lowers a degree, and checkpoint 9 certifies on cut-route
     controls and the riff (recommended)." (B) "Bodies limit themselves: a
     per-cell capacity at Mesocosm's scale, around a milligram a cell a
     tick, so thin or small bodies deliver less now. The dynamics change
     and the probe is calibrated again." Mark chose A, "Intact carries
     all". So a cell's capacity is a world rule set so every intact founded
     body reads its full degree; until wounds arrive, only a severing, an
     absence or a missing source lowers a degree, and checkpoint 9
     certifies on cut-route controls and the riff.
     *Specified 2026-10-05 by ruling 564:* each world draws the capacity,
     about 7 to 14 mg a cell a tick.

     Links: area Bodies and anatomy · draws on 560 · summary Per-cell route capacity is a world rule set so an intact body carries all its natives ask; only cuts, absences or missing sources lower degree.

562. **A process reads its system part by part.** Asked on 2026-10-05, with
     560: "How does a process read a system whose effects are every living
     part? Evidence: the digestive and photosynthetic systems feed every
     living part (489). The meal and income land on a body's parts by room
     (464). wing-functions routes first-fit, breadth-first, one target at a
     time." Options, recommended first: (A) "Part by part: each part takes
     its share of what the process lands only as far as its own route
     carries it. What can't reach a part stays where the process found it,
     the prey or the soil. The degree is what reached over what was asked
     (recommended)." (B) "One degree a body: the system's degree is its
     narrowest route's, and the whole process is scaled by it." (C)
     "Reached or not: a part beyond a cut route takes nothing and every
     reachable part takes its full share. Capacity counts only for cuts."
     Mark chose A, "Part by part". So each part takes its share of what a
     process lands only as far as its own route carries it, what cannot
     reach a part staying where the process found it, and a system's degree
     is what reached over what was asked. *Reading, not ruled:* the routing
     is wing-functions' own, which gains a query for how much a route can
     carry rather than a second router in the sim.
     *Specified 2026-10-05 by rulings 581 and 582:* a process's parts
     share a route's capacity, and a native reads the systems routing its
     function as one network.

     Links: area Bodies and anatomy · draws on 477 · summary A process reads its system part by part: each part takes its share only as far as its route carries; degree is reached over asked.

563. **No single riff is picked; riffing is to be systemic.** Asked on
     2026-10-05, checkpoint 9's second round: "Which riffed system does
     checkpoint 9 certify beside the defaults (491, 503)? Evidence: under
     561 an intact body carries all its natives ask, so a riff every probe
     body realizes changes nothing until wounds arrive. 491's own example is
     a gut that photosynthesizes, and under 492 a gut fixes by its area.
     Probe grazers have a lump that takes in, stores and reproduces, plus
     limbs and an eye; producers keep no stores." Options, recommended
     first: (A) "Gut that photosynthesizes: 491's example: the grazers'
     digestive system takes fix as a second source. Their lump gives a few
     cells to fix, so a grazer also earns a little income from its gut's
     area, read through the riff. Grazer recipes change and the calibration
     is redone (recommended)." (B) "Provision from stores: the reproductive
     system sourced from stores alone, so a grazer provisions only from its
     reserve. Producers, which store nothing, can't realize it and keep the
     default. Grazers breed from capital, echoing 535." (C) "Certify it as
     data: a riff built and checked as world data: realized or refused per
     body, its cut routes tested. A probe world carries it beside the
     defaults, inert on intact bodies until a native or a wound reads it."
     Mark answered: "The idea of the gut that photosynthesizes is about
     latent capability that can become an ability through variance via
     reproduction.  Calibrating to that one example feels odd. Shouldn’t
     we be going for the systemic approach, not picking and certifying
     individual trait/part combinations". So no trait and part combination
     is picked to calibrate or certify: a riff is a latent capability, cells
     a body already expresses, becoming an ability when variation in
     reproduction gives it a system that routes them, and the check
     certifies that mechanism wherever it arises. How systems vary and what
     the natives read of them go back to Mark as the next question. Mark
     also asked, sharing an article on computational models of thought,
     whether the work overcompensates for not letting things feed back by
     managing the dynamic; the record's answer was yes in part: 505's
     calibration to earlier medians, 553's soil widened for coverage and
     561's inert systems manage outcomes, while conservation, the runners'
     equivalence and the planted faults constrain the instrument.

     Links: area Bodies and anatomy · amends 491 · summary No single riff is picked to calibrate or certify; a riff is a latent capability becoming ability through variation, certified as a mechanism.

564. **Each world draws its per-cell capacity.** Asked on 2026-10-05, with
     563: "How tight is the per-cell capacity that lets an intact body carry
     all (561)? Evidence: an intact grazer's mouthful needs a median of 1.0
     mg per cell per tick through its parts, 2.0 at the 90th percentile and
     4.5 at the 99th. The most is 6.75, a 54 mg mouthful through an 8-cell
     lump. Only checkpoint 10's wounds will read the slack." Options,
     recommended first: (A) "Tight, about 7 mg: just above the domain's
     largest need. The fullest bodies feel their first lost cell, and most
     bodies have slack to spare (recommended)." (B) "A margin, 14 mg: twice
     the largest need. A wound has to take about half a part's cells before
     it narrows even the fullest route." (C) "Drawn per world: each world
     draws its capacity between the two, so the check covers stretched and
     slack bodies." Mark chose C, "Drawn per world". So each world draws a
     cell's capacity between about 7 mg a tick and 14, and the check covers
     stretched and slack bodies. *Reading, not ruled:* the range is a
     founding range a world may widen, 7 to 14 by default.

     Links: area Bodies and anatomy · draws on 561 · summary Each world draws its per-cell route capacity between about 7 and 14 mg a tick, a range a world may widen.

565. **A conduct part carries by its cross-section.** Asked on 2026-10-05,
     with 563: "How much more does a conduct cell carry than an ordinary
     cell (560)? Evidence: no probe or generated body has a conducting part
     yet, so the multiple is a world rule's default that nothing reads
     until one does. Conduct is measured by cross-section." Options,
     recommended first: (A) "Four times: a default of 4: a tube or branch
     carries clearly more, so conduits matter once a recipe grows one
     (recommended)." (B) "Twice: a default of 2: conduits help modestly."
     (C) "By cross-section: no fixed multiple: a conduct part carries by
     its cross-section over the reference segment's, so a wide tube carries
     more than a thin one." Mark chose C, "By cross-section". So a conduct
     cell's multiple is its part's cross-section over the reference
     segment's, a wide tube carrying more than a thin one, with no fixed
     number. *Reading, not ruled:* the reference segment's cross-section is
     its face, 25 voxels (505's evidence).

     Links: area Bodies and anatomy · draws on 560 · summary A conduct cell's capacity multiple is its part's cross-section over the reference segment's, with no fixed number.

566. **Nursing reads the young's digestive system.** Asked on 2026-10-05,
     with 563: "What does nursing read? Evidence: the table gives the meal
     to digestive, income to photosynthetic, the mouthful to muscular, the
     dose to glandular and reproduction to reproductive (489). Milk is the
     parent's provision moved to its young and digested into the young's
     tissue (527, 528). Rent, starvation, weaning and mineralization read no
     system." Options, recommended first: (A) "The young's digestive: milk
     lands in the young part by part along the young's digestive routes, as
     a meal would. The parent's provision leaves its reproduce part
     directly (recommended)." (B) "The parent's reproductive: milk leaves
     only as far as the parent's reproductive routes carry it from its
     living parts to its reproduce part, and lands in the young by room."
     (C) "Both: the parent's reproductive routes carry it out and the
     young's digestive routes take it in." Mark chose A, "The young's
     digestive". So milk lands in the young part by part along the young's
     digestive routes, as a meal would, and leaves the parent's reproduce
     part directly.

     Links: area Bodies and anatomy · draws on 527 · summary Nursing reads the young's digestive system: milk lands in the young part by part along its digestive routes, leaving the parent's reproduce part directly.

567. **The pre.4 runners build with the repo's pinned toolchain.** Asked on
     2026-10-05 in the Burn migration session, from pre.4's findings while
     proving 558: "the probe, repro and OPFS runners build from a neutral
     directory, so rustup gives them the default stable (1.97.1 here), not
     the repo's pinned 1.98.1. Every probe and repro bundle so far was built
     that way. What should the runners use?" Options, as the migration
     session relays them: (1) "The repo's pin (Recommended): each runner
     reads mere's rust-toolchain.toml and builds with it; the probe and
     repro bundles are rebuilt once and their new hashes recorded." (2)
     "Update default stable: run `rustup update stable` on this machine;
     the runners still follow whatever stable is." (3) "Leave it, record
     it: note in the burn plan that runner bundles come from the default
     stable." Mark's answer, as it relays it: "The repo's pin
     (Recommended)". So each runner reads mere's `rust-toolchain.toml` and
     builds with it, and the probe and repro bundles are rebuilt once with
     their new hashes recorded. Two findings came with it, not put to Mark:
     a bundle is byte-identical only within one target directory, since it
     embeds livery's `OUT_DIR` path and ThinLTO suffixes vary with it; and
     the OPFS probe stamps its provenance into each build, by design.

     Source: relayed by the migration session on 2026-10-05, for mere's
     burn plan §13.45 on `burn-pre4-repin`.
     *Checked 2026-10-05:* mere's burn plan records it as §13.45 on main
     (`07db35e2`), its question, options and answer as here.

     Links: area Stack and engine · summary The pre.4 probe, repro and OPFS runners build with the repo's pinned toolchain from mere's rust-toolchain.toml; bundles are rebuilt once.

568. **Systems vary at birth, and the epoch boundary folds in what
     prospered.** Asked on 2026-10-05, putting 563 back: "Where do systems
     vary (563)? Evidence: today systems are world data, one set of ten per
     world (489), and every body reads the same set. A lineage already
     varies in reproduction: each child draws its soma within its recipe's
     variance and absence odds (531). It learns the kinds its bodies take in
     (468), and its traits are revised at the epoch boundary (57), which the
     boundary family builds (521). A riff is kept only if a body can realize
     its routes (491)." Options, recommended first: (A) "At birth, per body:
     each body carries its systems, the world's defaults at founding. A
     child takes its parent's, and at odds its lineage's recipe carries,
     beside its variance and absence odds, one role's function is swapped
     for another its body expresses, kept only if the routes realize.
     Variation every generation, selection deciding (recommended)." (B) "At
     the boundary, per lineage: a lineage's systems riff by substitution at
     the epoch boundary along with its other traits (57), built with the
     boundary family. Checkpoint 9 makes systems lineage data and builds
     the riffing, with riffs drawn at founding meanwhile." (C) "Both: bodies
     vary at birth, and the epoch boundary folds the variants that
     prospered into the lineage." Mark chose C, "Both". So each body carries
     its systems, the world's defaults at founding; a child takes its
     parent's, and at odds its lineage's recipe carries one role's function
     is swapped for another its body expresses, kept only if its routes
     realize; and at the epoch boundary the variants that prospered fold
     into the lineage, 57's two doors applied to systems. *Reading, not
     ruled:* checkpoint 9 builds the systems a body carries and their
     variation at birth; the folding at the epoch boundary is built with
     the boundary family, which builds that boundary's revisions (521).
     *Specified 2026-10-05 by ruling 573:* the odds are drawn per recipe
     between 1 in 12 and 1 in 100.
     *Specified 2026-10-05 by rulings 574 and 576:* a founded body carries
     the defaults it realizes, and the functions a riff routes arise as
     cells vary at birth.
     *Specified 2026-10-05 by ruling 580:* a riff swaps or adds, at even
     odds.
     *Specified 2026-10-05 by ruling 583:* an inherited system the child
     cannot realize is carried dormant.

     Links: area Bodies and anatomy · draws on 563 · summary Each body carries its systems; a child may swap one role's function at recipe-drawn odds, and the epoch boundary folds prospering variants into the lineage.

569. **The natives act by capability.** Asked on 2026-10-05, with 568: "Do
     the natives act by capability? Evidence: the probe's natives belong to
     lineages: producers fix, grazers graze, each process keyed to a
     lineage's identity. An incorporated native frond keeps its fixing
     cells on a grazer (516), and variation can give a part cells nothing
     reads. Today those cells earn nothing, so a latent ability can never be
     selected." Options, recommended first: (A) "By capability: a native
     acts for any body whose systems route its function: a grazer whose
     system comes to route fix cells earns income. A lineage's natives are
     whatever its bodies can do (recommended)." (B) "By lineage, learned: a
     lineage gains a native when its lexicon learns a kind that expresses
     the native's function (468). Bodies of that lineage then act it." (C)
     "By lineage, as now: riffs feed only the natives a lineage already
     has. A grazer's fix cells stay latent." Mark chose A, "By capability".
     So a native acts for any body whose systems route its function, and a
     lineage's natives are whatever its bodies can do. *Reading, not
     ruled:* in the probe every lineage carries every native, each acting
     only for a body whose system routes its function, on that lineage's
     own accounts.
     *Specified 2026-10-05 by ruling 575:* a native acts on any carried
     system that routes its function.
     *Specified 2026-10-05 by ruling 590:* a body that takes in grazes any
     living body of another lineage at its site.

     Links: area Bodies and anatomy · draws on 516 · summary A native acts for any body whose carried systems route its function; a lineage's natives are whatever its bodies can do.

571. **The rates are drawn per world, not calibrated to earlier medians.**
     Asked on 2026-10-05, with 568: "When bodies change, do we keep
     calibrating rates to earlier medians? Evidence: 505 calibrated income
     and the mouthful twice to checkpoint 6's medians, now 5 and 12 mg
     because varied recipes make larger bodies. 553 widened the soil so
     buds appear within the run. With systems and riffs varying, any
     calibrated median moves again. This is the managing-outcomes habit you
     asked about." Options, recommended first: (A) "Stop; draw the rates:
     income and mouthful rates become world rules drawn per world within a
     range around today's values. Outcomes are measured, not tuned, and the
     check covers the range (recommended)." (B) "Freeze today's values:
     11/144 and 12/269 become world-rule defaults, never recalibrated.
     Outcomes are measured as they fall." (C) "Keep calibrating:
     recalibrate to checkpoint 6's medians whenever bodies change, keeping
     each checkpoint comparable with the last." Mark chose A, "Stop; draw
     the rates". So fixing's income and the mouthful take rates each world
     draws within a range around today's, outcomes are measured rather than
     tuned, and the check covers the range. *Reading, not ruled:* the range
     runs from half to twice today's rates, 11 mg per 144 faces and 12 per
     269 voxels; 505's calibration test retires, its receipts kept.

     Links: area Bodies and anatomy · summary Income and mouthful rates are world rules each world draws within half to twice today's values; outcomes are measured, not tuned to earlier medians.

573. **Each recipe draws its riff odds between 1 in 12 and 1 in 100.** Asked
     on 2026-10-05, with 572: "At what odds does a child's system vary
     (568)? Evidence: 568 puts the odds in the lineage's recipe beside its
     variance and absence odds, and the probe draws those per world (531).
     Mesocosm's absence odds are 1 in 12, its 'cheapest evidence that
     individuals are not clones'. A 60-tick probe world sees a median of 40
     births (1 to 113), so 1 in 12 gives about three riffs a world and 1 in
     100 most worlds none. A swap whose routes don't realize is not kept."
     Options, recommended first: (A) "Drawn, 1 in 12 to 100: each recipe
     draws its odds between 1 in 12 and 1 in 100, a range a world may
     widen. Some worlds riff often within a run and others rarely, and the
     check covers both (recommended)." (B) "1 in 12, as absences: the odds
     default to Mesocosm's absence odds, about three riffs a world, often
     enough that most runs see riffing." (C) "Rarer, 1 in 100: a system is
     a deeper change than a missing limb: most worlds see none within a
     run, and riffing shows over deep time." Mark chose A, "Drawn, 1 in 12
     to 100". So each recipe draws its riff odds between 1 in 12 and 1 in
     100, a range a world may widen, and the check covers worlds that riff
     often and rarely.

     Links: area World generation · draws on 568 · summary Each recipe draws its system-riff odds between 1 in 12 and 1 in 100, a range a world may widen.

574. **A body carries the systems it realizes.** Asked on 2026-10-05,
     checkpoint 9's fourth round: "Which systems does a body carry at
     founding (568)? Evidence: the ten default systems route all fifteen
     functions in some role (489). So if every body carries all ten,
     everything a body expresses is already routed and nothing is latent: a
     grazer that takes a frond whole fixes at once, by capability (569), no
     riff needed." Options, recommended first: (A) "Those it realizes: A
     body carries each default system whose roles its parts fill: a
     producer its photosynthetic and reproductive, a grazer its digestive,
     muscular, nervous and reproductive. A function it expresses outside
     them is latent until a riff routes it (recommended)." (B) "All ten:
     Every body carries the full default set. A function is latent only
     where a riff has taken it out of its system." Mark chose A, "Those it
     realizes". So a founded body carries each default system whose roles
     its parts fill, and a function it expresses outside them is latent
     until a riff routes it. *Reading, not ruled:* a system's roles are
     filled when a part expresses a function each of its source and effect
     roles names, "every living part" filled by any living part; stores and
     gates join a route where a part expresses them, but none is required,
     since no probe body expresses a gate.
     *Extended 2026-10-05 by ruling 587:* cells a development allocates in
     life bring the default system they realize.

     Links: area Bodies and anatomy · draws on 489 · summary A founded body carries each default system whose roles its parts fill; functions expressed outside them are latent until a riff routes them.

575. **A native acts on any system that routes its function.** Asked with
     574: "Which native reads a riffed system? Evidence: 489's table pairs
     each native with a system: the meal with digestive, income with
     photosynthetic, the mouthful with muscular. A riff swaps or adds a
     function in a role (491), for example fix as a digestive source."
     Options, recommended first: (A) "By function routed: A native acts on
     any carried system that routes its function: income wherever a system
     routes fix from a source, the meal wherever one routes intake. A gut
     with fix added earns income through its digestive routes
     (recommended)." (B) "By the table: Each native reads only its own
     system. A digestive system with fix as a source carries the meal from
     fix cells, eating as photosynthesis." Mark chose A, "By function
     routed". So a native acts on any system a body carries that routes its
     function, and 489's pairing of natives with systems names the defaults
     that route them, not a limit; a gut with fix added earns income
     through its digestive routes.

     Links: area Bodies and anatomy · draws on 569 · summary A native acts on any carried system that routes its function; 489's pairing names the defaults, not a limit.

576. **Latent capability arises as cells vary at birth.** Asked with 574:
     "Where does latent capability come from in the probe? Evidence: probe
     recipes give a frond fix and reproduce, a lump intake, store and
     reproduce, limbs contract and the eye sense. Children develop from the
     recipe, so within a run a body expresses only what its recipe gives,
     plus parts taken whole (in 2 to 3% of worlds). A riff can only route a
     function the body expresses." Options, recommended first: (A) "Cells
     vary at birth: A child's part may give a cell to another function the
     catalogue lets its shape express, at odds, the recipe's cell plan
     varying as its segments do. Latent capabilities arise there, and riffs
     make abilities of them (recommended)." (B) "Recipes carry latent
     cells: The generator and the probe give some kinds a cell of a
     function their lineage's systems don't route, such as a fix cell in a
     grazer's lump. Latency is founded, and riffs route it." (C) "Only
     parts taken whole: Latency comes from incorporation alone, a grazer
     keeping a frond's fixing cells. It's rare within a run." Mark chose
     A, "Cells vary at birth". So a child's part may give a cell to another
     function at birth, the recipe's cell plan varying as its segments do;
     latent capabilities arise there, and riffs make abilities of them.
     Parts taken whole remain a second source. How often a cell varies, to
     which functions (under 492 no shape gates one), whether a varied cell
     is inherited, and whether a riff swaps or adds go back to Mark as the
     next round.
     *Specified 2026-10-05 by rulings 577 to 580:* varied cells are
     inherited; one cell of a child varies at odds each recipe draws, 1 in
     12 to 1 in 100; to any grown function; and a riff swaps or adds at
     even odds.

     Links: area Bodies and anatomy · draws on 568 · summary Latent capability arises as a child's part may give a cell to another function at birth, at odds.

577. **A varied cell is inherited.** Asked on 2026-10-05, checkpoint 9's
     fifth round: "Is a varied cell inherited (576)? Evidence: a child draws
     its soma from its lineage's recipe by its own seed (531): segment
     counts drift and borne kinds go absent, but nothing passes from the
     parent's body. Systems do pass down (568). If a varied cell lasts one
     body, a riff that routes it must land in the same child: at 1 in 12 to
     1 in 100 each, that is 1 in 144 to 1 in 10,000 births, and a probe
     world sees a median of 40." Options, recommended first: (A)
     "Inherited: A child takes its parent's varied cells on top of its own
     recipe draw, as it takes its parent's systems. Latent capability
     builds up down a line, and a riff in a later generation can route it
     (recommended)." (B) "One body only: A varied cell lasts its body's
     life. Riffing routes it only when both land in one child. Within a run
     that is rare, and the epoch boundary folds in whatever prospered."
     Mark chose A, "Inherited". So a child takes its parent's varied cells
     on top of its own recipe draw, as it takes its parent's systems, and
     latent capability builds up down a line for a later riff to route.
     *Reading, not ruled:* a varied cell is kept by where it lies, its
     tagma, segment and borne part, and passes to a child whose soma
     develops that part; a segment the child's drift drops, or a borne kind
     it lacks, takes its varied cell with it.

     Links: area Bodies and anatomy · draws on 576 · summary A varied cell is inherited: a child takes its parent's varied cells on top of its own recipe draw.

578. **A child's cell varies at odds each recipe draws.** Asked with 577:
     "How often does a child's cell vary? Evidence: founded producers have
     4 to 27 cells (median 8) in 1 to 3 parts; grazers have 9 to 45
     (median 15) in 2 to 12 parts. Absences are drawn at 1 in 12 per tagma
     (531), and riffs at 1 in 12 to 1 in 100 per child, drawn per recipe
     (573). A world's median of 40 births gives about 0.4 to 3 varied
     children at per-child odds." Options, recommended first: (A) "Per
     child, drawn: One cell of one part varies, at odds each recipe draws
     between 1 in 12 and 1 in 100, as riffs are drawn. Large and small
     bodies vary alike (recommended)." (B) "Per cell, drawn: Each cell
     varies at odds drawn per recipe, between 1 in 100 and 1 in 1,000.
     Bigger bodies vary more: a 45-cell grazer about five times as often as
     an 8-cell frond." (C) "As absences, 1 in 12: A child's cell varies at
     the recipe's absence odds, about three varied children per world."
     Mark chose A, "Per child, drawn". So at odds each recipe draws between
     1 in 12 and 1 in 100, a range a world may widen, one cell of one of a
     child's parts varies. *Reading, not ruled:* the cell is drawn
     uniformly among the child's cells.
     *Specified 2026-10-05 by ruling 584:* any cell may vary, the last of a
     function that feeds, senses or fixes included.

     Links: area Bodies and anatomy · draws on 576 · summary One cell of one of a child's parts varies, at odds each recipe draws between 1 in 12 and 1 in 100.

579. **A cell may vary to any grown function.** Asked with 577: "Which
     functions can a cell vary to? Evidence: under 492 no shape gates a
     function, and 466's fits are only what the generator draws by
     default: a lump fits intake, store, circulate and reproduce; a sheet
     fix, secrete, respire and adhesion; a rod contract, support and grip;
     a point sense and adhesion. With fits only, a grazer's lump could
     never come to fix by variation (491's own example). It could only take
     a frond whole." Options, recommended first: (A) "Any, by measurement:
     Any grown function in the catalogue, drawn uniformly, so not secrete,
     which is acquired (341). What the cell does scales with the
     measurement its mechanic names (492), so a lump's fix cell fixes
     poorly, by its area (recommended)." (B) "Any, fits likelier: Any grown
     function, with the part's fits drawn at several times the odds of the
     rest. Most variation is reasonable for the shape, and the odd gut
     still comes to fix." (C) "Fits only: Only functions the part's shape
     fits. Variation stays shape-plausible. A gut comes to fix only by
     taking a frond whole." Mark chose A, "Any, by measurement". So a
     varied cell takes any grown function of the catalogue other than its
     own, drawn uniformly, and does what the measurement its mechanic names
     gives it, so a lump's fix cell fixes poorly by its area.

     Links: area Bodies and anatomy · draws on 576, 492 · summary A varied cell takes any grown catalogue function other than its own, drawn uniformly, and performs by the measurement its mechanic names.

580. **A riff swaps or adds, at even odds.** Asked with 577: "Does a riff
     swap a role's function or add one? Evidence: 491 says either, 'a
     role's function swapped for another, or one added', but 568's text
     says swapped. Under 575 a native acts on any carried system that
     routes its function. So a swap that takes intake out of a grazer's
     only digestive source ends its meal, while an add keeps it and gains
     the new one." Options, recommended first: (A) "Either, even odds: As
     491 has it: half of riffs swap and half add. A swap can cost a body an
     ability, and selection decides (recommended)." (B) "Add only: Riffs
     only add a function to a role. A body never loses what its systems
     routed, so abilities only build up." (C) "Swap only: As 568 put it: a
     riff always replaces, so every riff trades one route for another."
     Mark chose A, "Either, even odds". So half of riffs swap a role's
     function for another and half add one, a swap able to cost a body an
     ability, and selection decides. *Reading, not ruled:* the system, its
     role and the new function are drawn uniformly, the function among
     those the child's body expresses that the role does not already name.

     Links: area Bodies and anatomy · draws on 491, 568 · summary Half of riffs swap a role's function for another and half add one; a swap may cost an ability and selection decides.

581. **A process's parts share a route's capacity.** Asked on 2026-10-05,
     checkpoint 9's sixth round: "Do a body's parts share a route's
     capacity? Evidence: wing-functions resets edge capacity for every
     operation. Delivering part by part (562) as separate operations judges
     each part's route alone; delivering a process in one operation makes a
     trunk carry everything bound beyond it. 564's 7 to 14 mg a cell came
     from the shared reading: the most any part carries, 6.75, is a whole
     54 mg mouthful through an 8-cell lump. Intact bodies carry everything
     either way in checkpoint 9. The choice decides what checkpoint 10's
     wounds do." Options, recommended first: (A) "Shared: One operation per
     process: a part carries everything bound for the parts beyond it, so a
     narrowed trunk shorts every part behind it. Who loses first when it
     binds is checkpoint 10's question (recommended)." (B) "Own route: Each
     part's route is judged alone, as wing-functions works today. A
     narrowed trunk shorts only a part whose own share exceeds what it
     carries." Mark chose A, "Shared". So a process routes in one
     operation, a part carrying everything bound for the parts beyond it,
     and a narrowed trunk shorts every part behind it; who loses first when
     a trunk binds is checkpoint 10's question. *Reading, not ruled:*
     wing-functions' partial-carriage query routes a process to one sink
     that each effect part feeds by its share, so capacity is shared within
     the operation and resets between processes.

     Links: area Bodies and anatomy · draws on 562 · summary A process routes in one operation: a part carries everything bound beyond it, so a narrowed trunk shorts every part behind it.

582. **A native reads the systems routing its function as one network.**
     Asked with 581: "When several carried systems route a native's
     function, how does the native read them (575)? Evidence: every
     system's network runs over the same tree and the same capacities. They
     differ only in which parts are sources, stores, gates and effects. A
     riff adding intake to the muscular system's sources would also route
     the meal to the contracting parts. The meal's size is fixed by the
     bite, so reading each system as its own process would land it twice."
     Options, recommended first: (A) "As one network: The native reads the
     union of those systems as one network: all their sources, all their
     effects, the process landing across them (recommended)." (B) "The best
     one: The native reads whichever of those systems carries the most of
     what it asks, and the others stand idle." (C) "Default first: The
     native reads its own default system (489's pairing) where the body
     carries it, and a riffed system only where it does not." Mark chose A,
     "As one network". So a native reads the carried systems that route its
     function as one network, all their sources and all their effects, the
     process landing across them. *Reading, not ruled:* a role reading
     "every living part" names no function, so a native reads only roles
     that name its function, in the role 489's pairing gives it: fix and
     intake as sources, contract and reproduce as effects. Otherwise every
     latent cell would be routed already through a system whose sources are
     every living part, and nothing would be latent (574).

     Links: area Bodies and anatomy · draws on 575 · summary A native reads the carried systems routing its function as one network: all their sources, all their effects, the process landing across them.

583. **An inherited system the body cannot realize is carried dormant.**
     Asked with 581: "What happens to an inherited system the child's body
     doesn't realize? Evidence: absence takes a borne kind at 1 in 12 per
     tagma (531). A grazer whose only limb-bearing segment loses its limbs
     has no contracting part, so its muscular system has no effect. Varied
     cells are inherited (577), so a descendant may grow the part back.
     Founding carries only what a body realizes (574)." Options, recommended
     first: (A) "Carried dormant: The child carries it, routing nothing, and
     passes it on, so a descendant whose body realizes it again has it back.
     Latent systems build up as latent cells do (recommended)." (B)
     "Dropped: A child carries only the inherited systems its body
     realizes, as founding does. A dropped system is gone from the line
     until a riff or the epoch boundary restores it." Mark chose A, "Carried
     dormant". So a child carries every system its parent carried, one its
     body cannot realize routing nothing, and passes it on, so a descendant
     whose body realizes it again has it back.

     Links: area Bodies and anatomy · draws on 568 · summary A child carries every system its parent carried; one its body cannot realize routes nothing (dormant) and is passed on.

584. **Any cell may vary.** Asked with 581: "Can a cell vary out of the
     last cell of a function that feeds, senses or fixes? Evidence: a
     grazer's lump can hold a single intake cell (its 8 cells less store and
     reproduce), its eye has one sense cell, and a frond can hold as little
     as one fix cell. Absence spares kinds that feed, sense or fix, but only
     as a reading of 531, never ruled. Varying such a cell makes a child
     that cannot eat or fix." Options, recommended first: (A) "Any cell: Any
     cell may vary, and selection decides: a child born unable to feed
     starves. This matches 571, outcomes measured rather than managed
     (recommended)." (B) "Never the last: A cell never varies out of the
     last one its body holds of a function that feeds, senses or fixes, as
     absences spare those kinds." Mark chose A, "Any cell". So any cell may
     vary, the last of a function that feeds, senses or fixes included, and
     a child born unable to feed starves. The sparing of those kinds from
     absence, 531's reading, is not touched.

     Links: area Bodies and anatomy · draws on 578 · summary Any cell may vary, even the last of a feeding, sensing or fixing function; a child born unable to feed starves.

587. **A development brings the system its cells realize.** Asked on
     2026-10-05, checkpoint 9's seventh round: "Does a gland that develops
     in life bring its system (574)? Evidence: a quarter of producer
     cohorts are gland candidates, and the gland develops after founding
     by allocating fix cells to secrete. Founded fronds express only fix
     and reproduce. So under 574 no frond carries the glandular system, and
     the dose, a live mechanic since checkpoint 6, would stop in every
     world." Options, recommended first: (A) "Development brings it: Cells
     a development allocates during life bring the default system they
     realize, as founding does. Cells varied at birth do not; they stay
     latent until a riff routes them (recommended)." (B) "Dormant from
     founding: A lineage whose development can place a function carries
     that function's default system dormant from founding, and it wakes
     when the cells arrive." (C) "As ruled: The gland is latent like any
     other unrouted cell, and the dose stops until a riff routes secrete."
     Mark chose A, "Development brings it". So cells a development
     allocates during life bring the default system they realize, as
     founding does, while cells varied at birth stay latent until a riff
     routes them; a frond's gland brings the glandular system, and the dose
     goes on.

     Links: area Bodies and anatomy · amends 574 · summary Cells a development allocates during life bring the default system they realize; cells varied at birth stay latent until a riff routes them.

588. **The biting parts are the parts bitten.** Asked with 587: "What are
     'the biting parts' the glandular system feeds (489)? Evidence: the
     table names them as the glandular system's effects but never defines
     them. The probe's dose is a frond's defense: a grazer biting a frond
     whose gland the soil charges pays from its reserve, in the share of
     the frond its bite took. A frond has no part that bites." Options,
     recommended first: (A) "The parts bitten: The secretion reaches
     whichever part a bite lands on, and the dose is what the routes carry
     there. A frond's gland sits in the frond, so intact fronds dose as now
     (recommended)." (B) "Every living part: The secretion spreads through
     the whole body, and the dose reads the bitten part's share of it." (C)
     "The parts that bite: Venom delivered by the body's own biting parts.
     A frond has none, so the probe's dose stops until a riff routes it
     elsewhere." Mark chose A, "The parts bitten". So the glandular
     system's effects are whichever part a bite lands on, the dose being
     what its routes carry there, and intact fronds dose as now.

     Links: area Bodies and anatomy · draws on 489 · summary The glandular system's effects are whichever part a bite lands on; the dose is what its routes carry there.

589. **The muscular system carries the limbs' share of the mouthful.** Asked
     with 587: "How does the mouthful read the muscular system? Evidence:
     221 of 4,501 founded grazers (4.9%, in 35 of 300 worlds) have no
     contracting part, because absence took their limbs, which 531 allows.
     Today such a grazer bites by its intake volume alone; its limbs' span
     is what scales a bite up (TD9's build multiple). Muscular's effects
     are the contracting parts (489)." Options, recommended first: (A)
     "Limbs' share only: Muscular carries only the span's part of the
     mouthful. A limbless grazer still bites by its intake, as now
     (recommended)." (B) "Whole mouthful: Muscular carries the whole bite.
     A grazer with no contracting part cannot bite and starves; about 5%
     are born that way." Mark chose A, "Limbs' share only". So the muscular
     system carries the span's part of the mouthful, TD9's build multiple,
     to the contracting parts, and a limbless grazer bites by its intake as
     now.

     Links: area Bodies and anatomy · draws on 489 · summary The muscular system carries the span's part (build multiple) of the mouthful to contracting parts; a limbless grazer bites by intake.

590. **A body grazes other lineages.** Asked with 587: "What does a body
     that comes to take in eat (569)? Evidence: under 569 every lineage
     carries every native, so a frond may graze once a varied cell and a
     riff route intake. Today a graze targets its lineage's named prey,
     the producers, by a weighted draw (287)." Options, recommended first:
     (A) "Other lineages: A body grazes any living body of another lineage
     at its site, weighted by what each holds. Grazers eat producers as
     now, and a frond that comes to take in eats grazers (recommended)."
     (B) "Any but itself: Any living body at its site except itself, its
     own lineage included." (C) "World's food web: Each lineage keeps the
     prey the world names for it. A lineage given none cannot graze,
     whatever its systems route." Mark chose A, "Other lineages". So a body
     whose systems route intake grazes any living body of another lineage
     at its site, weighted by what each holds (287): grazers eat producers
     as now, and a frond that comes to take in eats grazers.

     Links: area Ecology · draws on 569, 287 · summary A body whose systems route intake grazes any living body of another lineage at its site, weighted by what each holds.

592. **No source file over 600 lines without a technical reason.** 2026-10-05,
     Mark, unprompted: "enforce the 600 loc limit. there is no file that
     should be bigger than that without a technical reason." So the
     ceiling (193, and the repository's `CLAUDE.md`) is checked by a script
     over every tracked source file, any exception listed with its reason,
     and the files over it today are split.
     *Scoped and enforced 2026-10-05 by rulings 593 to 595:* source only,
     a pre-commit hook, `field-bake` retired; the splits landed at
     `bb92f2e`.

     Links: area Method and documents · draws on 193 · summary No tracked source file over 600 lines without a listed technical reason; a script checks it and oversize files are split.

593. **Docs are outside the ceiling, and deduplicated.** 2026-10-05. Question:
     do markdown docs fall under the 600-line ceiling (36 over it, the wing
     record 13,725 lines)? Options: active docs too (recommended); every
     doc; source code only. Mark chose "Source code only", then "but
     deduplicate the docs." So the ceiling covers source code, and the docs
     are deduplicated: each result one home, other mentions a link.

     Links: area Method and documents · amends 592 · summary The 600-line ceiling covers source code only; docs are deduplicated, each result having one home and other mentions linking to it.

594. **A pre-commit hook enforces the ceiling.** 2026-10-05. Options: a
     pre-commit hook (recommended); by hand. Mark chose "Pre-commit hook".
     So `.githooks/pre-commit` runs `scripts/line-ceiling.ps1 -Staged` and
     this clone's `core.hooksPath` points at it. *Reading, not ruled:* the
     hook checks only what a commit stages, so branches made before the
     splits are not blocked by files they did not touch, and it says so
     and passes where no PowerShell exists.

     Links: area Method and documents · draws on 592 · summary A pre-commit hook runs the line-ceiling script on staged files to enforce the ceiling.

597. **Eponym's motion stays game-side.** 2026-10-05. Question: how does
     Eponym's world move onto the sim, given that 233 keeps motion and
     contact game-side over conatus, which wraps Rapier? Options: motion
     stays game-side (recommended); move it all, conatus included. Mark
     chose "Motion stays game-side". So `eponym-world`'s motion and contact
     (about 2,100 lines, its only conatus users) stay with the game as
     `eponym-motion`; `eponym-world`, `eponym-social` and `eponym-identity`
     move into Isocosm, and Isocosm never depends on Rapier. *Reading, not
     ruled:* the sim keeps motion's data and admission and is handed a
     solver (`MotionSolver`), none by default so motion is refused by name;
     a host hands the conatus solver, and a save with motion replays with
     the same one, keeping Eponym's rule that hosts never supply a final
     position; fixtures that need motion recorded but not solved use a
     solver that does not move.

     Links: area Eponym · draws on 233 · summary Eponym's motion and contact stay game-side as eponym-motion; other Eponym crates move into Isocosm, which never depends on Rapier.

598. **The campaign world's adapters stay with the VTT.** 2026-10-06.
     Question: how does the VTT's campaign world move onto the sim, given
     that `isometry-campaign` depends on `isometry-core`, the VTT's
     substrate, in four places (the map wrapper, token and sheet drafting,
     the overmap projection, one test)? Options: adapters stay with the
     VTT (recommended); move it all. Mark chose "Adapters stay with the
     VTT". So the campaign world moves into Isocosm as its legacy campaign
     tree, and those adapters stay in a slim VTT crate over
     `isometry-core`, the sim depending on no game crate, as with Eponym
     (597).
     *Amended 2026-10-06 by ruling 599:* the overmap projection is not an
     adapter; the overmap itself moves into the sim.

     Links: area The VTT · plans campaign-retirement · summary The campaign world moves into Isocosm as its legacy campaign tree; the map wrapper and token/sheet adapters stay in a slim VTT crate. (amended by 599, 913, 915)

599. **The overmap moves into the sim.** 2026-10-06. Question, on new
     evidence against 598: the campaign world's `travel_cost` and
     `discover_around` (sim behaviour: travel and discovery) route through
     `isometry-core`'s `Overmap`, a self-contained 262-line place graph
     with Dijkstra routing that uses only std and serde, and about eight
     VTT files also use it; where does the overmap go? Options: into the
     sim (recommended); travel stays with the VTT, as adapter methods; the
     sim uses `isometry-core` for the overmap alone. Mark chose "Into the
     sim". So `overmap.rs` moves into Isocosm with the campaign world, a
     place graph being a sim noun (ruling 72); travel and discovery stay
     sim-side; the VTT imports `Overmap` from Isocosm; `isometry-core`
     stays pure and light. The adapters left with the VTT are the map
     wrapper (`CampaignMap`) and token and sheet drafting. *Reading, not
     ruled:* the sim keeps every check lowering makes, as
     `LocalMapProposal::validate` and `DraftMap::validate`, so a draft is
     refused sim-side before any VTT document is built; the VTT's lowering
     is a pair of extension traits in the slim `isometry-campaign`.

     Links: area The VTT · amends 598 · plans campaign-retirement · summary The overmap moves into the sim as a sim noun; travel and discovery stay sim-side; the VTT imports Overmap from Isocosm.

600. **The VTT builds on the stack we have; Cleromancy leaves it.**
     2026-10-06. Question: main's root lock pins ten crates.io packages
     whose `.crate` files are gone from the local cache (imagesize, usvg,
     resvg, selectors, stylo_malloc_size_of, tikv-jemalloc-sys,
     tikv-jemallocator, fontconfig-parser, cc 1.4.6, glam 0.33.7), so the
     root workspace cannot build offline, untouched main included; how
     should the VTT side of the campaign move be verified and locked?
     Options: fetch the ten (recommended); offline, taking the resolver's
     drift to cached versions; offline, keeping main's pins by hand. Mark
     answered: "Sounds like we should just rebuild on the actual stack we
     have instead of inviting crates we don't need or want into the
     stack". Told that eight of the ten come only from Cleromancy's second
     stack (genet `5ae30cad`, mere `876320fd`), behind `isometry-genet`'s
     `cleromancy` feature (off by default since ruling 298), he added:
     "Acha, that should not be in at all. Old, stale dependency", and "The
     idea was, hey, maybe we could make seeds for worlds out of readings or
     details you provide. But eh. Marginal benefit!" So nothing is fetched
     and Cleromancy leaves the VTT: the feature, its workspace pin, its
     selection module, and the `ipc-channel` patch only its genet reached.
     `>choose` is the VTT's own seeded draw (ruling 323), the default
     already. *Reading, not ruled:* the lock takes the versions on disk
     (cc 1.6.0, find-msvc-tools 0.1.14, glam 0.33.12); the Cleromancy
     decision record moves to the VTT's archive; with Cleromancy gone the
     repo `CLAUDE.md`'s `--all-features` gate runs whole again, unchanged.

     Links: area Stack and engine · summary The VTT builds on the stack it has; Cleromancy leaves it entirely (feature, workspace pin, selection module, ipc-channel patch); nothing is fetched.

604. **Game-side physics replays bit for bit across machines.** 2026-10-06,
     from the balaur read (603). Question: Isocosm's sim is integer-only, and
     floats live game-side, in Eponym's motion over conatus, whose rapier
     3d 0.33 has neither `enhanced-determinism` nor libm; D18 (2026-09-21)
     parked one software libm as a note; balaur's recipe (rapier
     `enhanced-determinism`, glamx with `libm` and `scalar-math`, a lint
     against bare transcendentals) is diffed in CI on Linux, macOS arm64 and
     Windows, and mere measured `enhanced-determinism` free at 500 to 5,000
     bodies; Eponym saves replay motion through the solver (597). Should the
     wing require bit-identical physics replay across machines? Options:
     require it now (recommended); keep D18's note; same-machine only. Mark
     chose "Require it now". So every float path a game hands to the sim's
     physics replays bit for bit on any machine, carried out in conatus by
     the physics lane's G8 with that recipe and a digest diffed across
     Windows, macOS and Linux. This turns D18's libm note into a
     requirement. *Reading, not ruled:* until G8 lands it, a save promises
     same-machine replay only.

     Links: area Stack and engine · draws on 597 · summary Every float path a game hands to the sim's physics replays bit for bit on any machine, via the balaur-style recipe and cross-OS digest.
605. **The shards carry a shard-count equality check.** 2026-10-06, from
     the balaur read. Question: §4.7 designs the ordered merge before the
     shards, which are not built; balaur asserts one-thread and
     eight-thread runs give one digest; Isocosm's draws are keyed by seed
     and domain, so no one stream couples shards. Should the shard work
     carry an equality check? Options: make it a done-condition
     (recommended); leave it to the shard plan. Mark chose "Make it a
     done-condition". So when the shards land, one shard and N shards give
     the same state hash, at more than one thread count, as a standing test.

     Links: area Processes and rules · summary When shards land, one shard and N shards give the same state hash at more than one thread count, as a standing test.
606. **471's kiss3d reshape starts from upstream.** 2026-10-06, from the
     balaur read. Question: balaur's kiss3d is Ughuuu/kiss3d#1 (branch
     `balaur-wrapped-surface` at `4522a762`), 82 commits and 125 files over
     v0.46.0 that GitHub will not render, with useful pieces (a software
     adapter for surface-less rendering, offscreen depth always samplable,
     every shadow setting exposed, built-in shaders linked in a fixed
     order) but none of 471's four reshape items; where does the reshape
     start? Options: upstream, taking commits by name (recommended); from
     balaur's branch; wait for upstream. Mark chose "Upstream, take commits
     by name". So the reshape forks upstream kiss3d at a pinned commit and
     takes balaur's commits one at a time as it needs them, each reviewed,
     so the fork's own diff stays readable.

     Links: area Stack and engine · amends 471 · summary The kiss3d reshape forks upstream at a pinned commit and takes balaur's commits one at a time, each reviewed.

607. **Isocosm's state hash joins the family's witness, in balaur's
     shape.** 2026-10-06, from the balaur read. Question: Isocosm's
     `Simulation::state_hash` takes SHA-256 over the whole state's JSON,
     used 220 times across the tree, and answers only "unequal"; balaur
     labels each slice by stable id and names the first that differs. What
     should Isocosm do? Options: add labelled entries beside the SHA-256
     hash (recommended); leave it; replace it with a per-tick FNV digest.
     Mark asked: "Can we scope 3? Seems appealing but how would this go?"
     Told that the third means Isocosm joining the witness the rest of the
     family already uses (`isometer_core::snapshot::hash_bytes`, FNV-1a over
     postcard bytes, in Mesocosm, Eponym and the legacy trees), Mark: "Ok, 3
     sounds good to me." So Isocosm's state hash becomes the family's 64-bit
     FNV-1a witness, with labelled entries keyed by stable ids, a
     first-divergence report and a per-tick trace. *Reading, not ruled:*
     SHA-256 stays for content addresses (`crate::digest`: the genesis
     digest, branch ids) and for the draws (`crate::draw`), which this does
     not touch; FNV is an equality witness, not a cryptographic digest, so
     anything peers must trust stays on a standard cryptographic hash.

     Links: area Record and history · summary Isocosm's state hash becomes the family's 64-bit FNV-1a witness with labelled entries, first-divergence report and per-tick trace; SHA-256 stays for content addresses.

608. **Version-1 saves still load.** 2026-10-06, carrying out 607.
     Question: Isocosm's version-1 saves store the SHA-256 state hash and
     recheck it on load, and `tests/data/pre-causation-world.json` proves a
     world saved before a field rename still loads; what happens to them?
     Options: read v1, write v2 (recommended); refuse v1. Mark chose "Read
     v1, write v2". So a v1 save is verified with the SHA-256 hash it
     carries, kept only as a v1 reader, and saving again writes v2 with the
     FNV witness; the fixture's guarantee holds.

     Links: area The sim's schema and saves · summary Version-1 saves are verified with their SHA-256 hash as a v1 reader only; saving again writes v2 with the FNV witness.
609. **The labelled digest is built once, in mere.** 2026-10-06. Question:
     FNV `hash_bytes` sits in `isometer_core::snapshot`, shared by Mesocosm,
     Eponym and the legacy trees, and mere's physics lane put fork E to Mark
     (the same labelled trace and first divergence for conatus's G8); mere
     sits below Isometry, so a wing crate cannot serve conatus. Where is the
     instrument built? Options: once, in mere (recommended); beside
     `hash_bytes`; Isocosm only. Mark chose "Once, in mere". So one
     labelled-digest crate in mere serves conatus's G8, which Mark took on
     mere's side ("That, plus replay as a track", adding G10's record and
     replay), and Isocosm. *Reading, not ruled:* G8 shapes the crate;
     Isocosm's switch to the FNV witness need not wait for it, and its
     labels and first-divergence report follow when the crate exists.

     Links: area Stack and engine · draws on 604 · summary One labelled-digest crate is built once in mere, serving conatus's G8 and Isocosm.
610. **Labelled checkpoints per epoch; a per-tick trace on demand.**
     2026-10-06. Question: Isocosm advances only through ticks with due
     events (ruling 284), and its saves already record a checkpoint, tick
     and hash, at each epoch boundary, a mismatch saying only "checkpoint
     history mismatch"; balaur records a digest every tick. How often does
     Isocosm record? Options: epochs in saves, ticks on demand
     (recommended); every tick in the save. Mark chose "Epochs in saves,
     ticks on demand". So saves keep one labelled checkpoint per epoch and
     a mismatch names the first diverging entry; the bench and probe tools
     write a full per-tick trace when asked.

     Links: area Record and history · draws on 607, 284 · summary Saves keep one labelled checkpoint per epoch and a mismatch names the first diverging entry; bench and probe tools write a per-tick trace on demand.
612. **An index row is a pointer, a purpose and a status.** 2026-10-06,
     carrying out 593. Question: the wing index's rows hold 11,819 of its
     13,367 words, the design record's row alone 2,737 words of ruling
     summaries that grow with every ruling; what should a row hold?
     Options: pointer, purpose, status (recommended); the same plus a
     latest-rulings line; leave the indexes. Mark chose "Pointer, purpose,
     status". So each row is the link, one line of what the doc is for and
     a dated status; ruling summaries live only in this record.

     Links: area Method and documents · draws on 593 · summary An index row is the link, one line of purpose and a dated status; ruling summaries live only in the record.
613. **Progress logs record landings, past entries included.** 2026-10-06,
     carrying out 593. Question: this record's Progress log is 1,679 lines,
     largely one-line restatements of rulings already in §0, and plans' logs
     do the same; the standing rule keeps dated text's words, which
     conflicts with 593. What happens to them? Options: landings only,
     trimming past entries (recommended); only from now on; leave them. Mark
     chose "Landings only, trim past". So a progress entry records what was
     built or verified, with its commit, and names rulings by number without
     restating them; past entries are cut to that shape, being logs and not
     rulings, and the rulings keep their words.

     Links: area Method and documents · draws on 593 · summary A progress entry records what was built or verified with its commit, naming rulings by number; past entries are cut to that shape.
614. **Session notes are archived; briefs cite rulings by number.**
     2026-10-06, carrying out 593. Question: the sim design session notes
     (1,921 lines) tabulate each round's questions and answers, which this
     record holds as rulings, beside narrative (how it started, what was got
     wrong, what was left open), and the anatomy brief tabulates its rounds
     the same way. Options: archive the notes (recommended); keep them with
     tables cut to numbers; leave them. Mark chose "Archive the notes". So
     anything still open is extracted into a live doc, the notes move to
     `archive_docs/2026-10-06/`, and briefs keep their design text but cite
     rulings by number instead of re-tabulating rounds.

     Links: area Method and documents · draws on 593 · summary Design session notes are archived; briefs keep design text but cite rulings by number instead of re-tabulating rounds.
615. **One `DOC_POLICY.md`, and shared `CLAUDE.md` rules at the root.**
     2026-10-06, carrying out 593. Question: `DOC_POLICY.md` exists three
     times (root, Mesocosm, Eponym), one canonical core with stale local
     addenda, and the two product `CLAUDE.md` files share about 400
     ten-word runs, the two `LICENSES.md` about 250; how far does dedup go?
     Options: one `DOC_POLICY.md` only (recommended); also the `CLAUDE.md`
     files; leave the copies. Mark chose "Also the CLAUDE.md files". So the
     root keeps the one `DOC_POLICY.md`, with one addendum covering the three
     products, the product copies replaced by a link; the rules the product
     `CLAUDE.md` files share move to the root `CLAUDE.md`, the product files
     linking; the `LICENSES.md` files, per-product legal records, stay.

     Links: area Method and documents · draws on 593 · summary One root DOC_POLICY.md with a three-product addendum; rules the product CLAUDE.md files share move to the root CLAUDE.md; LICENSES.md files stay.

616. **A plan holds the spec, the record the decision.** 2026-10-06,
     carrying out 593. Question: after the dedup's first four phases the
     largest overlap left is this record against the sim plan (827 ten-word
     runs) and the anatomy brief (491), their tables and design text stating
     what rulings set; how far does dedup go? Options: spec in plans,
     decisions in the record (recommended); cut the plans' restatements; stop
     there. Mark chose "Spec in plans, decisions in record". So a plan is its
     spec's home and cites ruling numbers, this record is the decision's home,
     neither restates the other's rationale, and the overlap a spec shares
     with the ruling that set it stays.

     Links: area Method and documents · draws on 593 · summary A plan is its spec's home and cites ruling numbers; the record is the decision's home; neither restates the other's rationale.
618. **A doc's own status line is current.** 2026-10-06. Question: about
     twenty plans' own status lines are older than the status their index
     row carries; where does current status live? Options: update the doc's
     line (recommended); the index row only. Mark chose "Update the doc's
     line". So each plan's status line carries its newest dated status, and
     the index row repeats a short version of it.

     Links: area Method and documents · draws on 612 · summary Each plan's status line carries its newest dated status; the index row repeats a short version of it.
622. **scenevm is a donor of techniques only.** 2026-10-06, from Mark's
     question whether Eldiron's `scenevm` (MIT, 0.95.0) would be "of use for
     the isocosm wing", how it "compares with kiss3d as a tenant, or composes
     with the structure we're researching from balaur engine". Evidence:
     - it owns its device (`static GLOBAL_GPU: OnceLock`) and its targets,
       and is on wgpu 29.0.3 against the stack's 30;
     - it carries Eldiron's scene model and its own sun, lights and probes,
       against 472;
     - beside a raster path, it has a compute ray and path tracer;
     - it has a capped irradiance probe grid that samples along the normal.

     Read in `2026-10-06_scenevm_wing_brief.md`. Options: donor of
     techniques only (recommended); measure it as a tenant; no role. Mark
     chose "Donor of techniques only". So kiss3d stays the lit body tenant
     (471, 606), and scenevm is read, not depended on. Its techniques (the
     probe grid, progressive accumulation, its environment block's fields)
     are taken by name under MIT, with a licence row, when a lane needs them.
     Asked whether to record the read, Mark chose "Write a brief, like
     Balaur's".

     Links: area Stack and engine · draws on 471, 606 · summary scenevm is a donor of techniques only: kiss3d stays the lit body tenant; techniques are taken by name under MIT with a licence row.
623. **Rosters and rulesets move into datasheets, through a plan.**
     2026-10-06. Mark asked whether the TOML used for generated datasheets
     suits Isocosm's entity rosters and rulesets. Evidence:
     - the wing's packs are JSON (`mesocosm-pack.json`, five
       `processes/*.json`, the VTT's `isometry-pack.json` and SRD
       `data/*.json`), with behaviour in piccolo Lua;
     - its rosters are Rust code (`Founding::SpacedRoster.palette()`, the
       impresa's `SEEDED_KINDS`);
     - Livery's `properties.toml` is the stack's datasheet shape.

     Mark widened the format question to the whole stack. It is ruled in
     mere's `design_docs/2026-10-06_data_formats_brief.md` (F1 to F5): text is
     split by who writes the file (TOML for what people author or review, JSON
     for what programs write and for imports), and binary uses two formats by
     job. On the wing's own question, the options were: yes, through a plan;
     only new ones; not now. Mark chose "Yes, through a plan". So a dated plan
     names which rosters and catalogues move, their schemas in Livery's
     datasheet shape, and receipts showing worlds and state hashes unchanged
     by the move. Under F1, the wing's authored pack manifests and process
     records become TOML; the SRD data stays a JSON import.

     Links: area Method and documents · summary Rosters and rulesets move into datasheets through a dated plan; authored pack manifests and process records become TOML, SRD data stays JSON import.
624. **Packs load from TOML or JSON.** 2026-10-06, for the move of the
     pack files to TOML. Question: after the migration, should packs still
     load from JSON? Options: TOML only, with a JSON pack refused by a
     message naming the rule (recommended); accept both. Mark chose "Accept
     both". So both loaders (`mesocosm-phenotype`'s admission and
     `isometry-system`'s content packs) read either syntax, TOML preferred,
     and this repository's own packs are TOML. The pre-commit check then
     polices this repository only; a pack written elsewhere in JSON still
     loads. *Reading, not ruled:* the manifest's `abi` stays 1, since the
     schema does not change, only the syntax.

     Links: area Method and documents · draws on 623 · summary Packs load from TOML or JSON, TOML preferred; this repository's own packs are TOML; manifest abi stays 1.
625. **Every founding becomes data, instrument variants too.** 2026-10-06,
     for 623's plan. Question: which rosters move into datasheets? Options:
     the shipping rosters and kinds, with the instrument-only variants
     staying code (recommended); everything, instrument variants too; only
     the new sim's founding ruleset. Mark chose "Everything, instrument
     variants too". So every `Founding` variant (`Drawn`,
     `BrowsingConsumer`, `RosterStand`, `RosterFauna`, `Roster`,
     `BranchingRoster`, `JointedRoster`, `SpacedRoster`), with its palette
     and per-tier recipe lists, becomes a datasheet, and the enum becomes a
     list read from them, beside `SEEDED_KINDS`. Recorded worlds must
     reproduce: every founding's state hash is unchanged by the move. The
     plan is `2026-10-06_wing_datasheets_plan.md`.

     Links: area Method and documents · draws on 623 · summary Every Founding variant, instrument variants included, becomes a datasheet and the enum a list read from them; state hashes unchanged.
628. **The rationale prose moves into the datasheets.** 2026-10-06, the same
     checkpoint. Question: where does the prose go (why each body and shape
     exists, the DC findings and rulings)? Options: into the datasheets as
     TOML comments (recommended); stay in the Rust docs. Mark chose "Into
     the datasheets". So the reasons sit beside the data people edit, and
     the Rust keeps only loader documentation.

     Links: area Method and documents · draws on 593 · summary Rationale prose for bodies and shapes moves into the datasheets as TOML comments; Rust keeps only loader documentation.
633. **Saves keep fields; traces keep entities.** 2026-10-06, the state
     *(Recorded at `c0091aa8` as 622 to 625, numbers already taken by the
     datasheets rulings on main; renumbered 633 to 636 the same day.)*
     witness brief's first fork. Question: a saved checkpoint can carry its
     labelled entries per top-level `State` field (thirteen) or per entity
     (thousands in a grown world); what grain do saves keep? Options: fields
     in saves, entities in traces (recommended); entities in saves; fields
     only. Mark chose "Fields in saves, entities in traces". So a load names
     the collection that diverged, and the on-demand per-tick trace names the
     entity.

     Links: area Record and history · draws on 610 · summary Saved checkpoints carry labelled entries per top-level State field; the on-demand per-tick trace carries entities.
634. **A v2 save stores the witness as a u64.** 2026-10-06. Options: u64
     (recommended); a hex string in the existing `Key`. Mark chose "u64". So
     v2 stores the family witness's own type, and v1 saves are read through a
     separate struct carrying their string.

     Links: area The sim's schema and saves · draws on 608 · summary A v2 save stores the witness as a u64; v1 saves are read through a separate struct carrying their string.
636. **The witness hashes postcard bytes.** 2026-10-06. Options: postcard
     (recommended); JSON into FNV. Mark chose "Postcard". So Isocosm hashes
     through `isometer_core::snapshot::encode`, one encoding across the
     family.

     Links: area Record and history · draws on 607 · summary The witness hashes postcard bytes through isometer_core::snapshot::encode, one encoding across the family.
641. **A v1 save is still checked at every checkpoint.** 2026-10-07,
     reading 608 for H1. Question: 608 verifies a v1 save "with the SHA-256
     hash it carries", and today's load also checks each epoch checkpoint's
     hash; the one tracked v1 save (the pre-causation fixture, tick 7) has
     no checkpoints. Should a v1 load still check its checkpoints? Options:
     final hash and checkpoints (recommended); final hash only. Mark chose
     "Final hash and checkpoints (Recommended)". So the v1 reader takes the
     SHA-256 at each epoch boundary while replaying and compares it, and a
     v1 fixture with checkpoints, made by the pre-H1 code, proves it.

     Links: area The sim's schema and saves · draws on 608 · summary A v1 save is checked at every epoch checkpoint with the SHA-256 as well as its final hash, proved by a v1 fixture with checkpoints.
642. **The probe traces draw 0's exact run.** 2026-10-07, for H1's
     `--trace` (610). Question: `isocosm-probe` runs many drawn worlds,
     each four ways, and already reruns draw 0's exact run as a reference;
     what does its trace record? Options: draw 0's exact run, in the bench's
     `<tick> <witness>` lines (recommended); every draw's exact run; every
     arm of every draw. Mark chose "Draw 0's exact run (Recommended)". So
     `run_exact` takes an optional per-tick sink, and both tools write the
     same line format.

     Links: area Record and history · draws on 610 · summary The probe's trace records draw 0's exact run in the bench's tick-witness line format.
643. **The repo shares a build-dir, not a target-dir.** 2026-10-07,
     assessing 638. Question: cargo 1.98 has `build-dir` (stable since
     1.91), and two throwaway workspaces under one shared build-dir reused
     one serde build while each kept its own `target/` for final outputs;
     nothing in the repo reads `target/debug` by path. Which mechanism?
     Options: shared build-dir (recommended); shared target-dir. Mark chose
     "Shared build-dir (Recommended)". So intermediates are shared per
     checkout and each workspace's binaries stay where they are.

     Links: area Stack and engine · summary The repo shares a cargo build-dir, not a target-dir; intermediates are shared per checkout and binaries stay per workspace.
644. **The setting is machine-local.** 2026-10-07. Question: the root
     `.cargo/config.toml` is gitignored on purpose as the machine-local
     override file, and every workspace inherits it; where does the
     setting live? Options: the machine-local root config (recommended);
     un-ignore and commit it; `wing.ps1` passes it. Mark chose
     "Machine-local root config (Recommended)". So nothing is committed but
     the plan, and each worktree or machine opts in by the plan's recipe.

     Links: area Stack and engine · draws on 643 · summary The shared build-dir setting lives in the machine-local root cargo config; nothing is committed but the plan.
648. **Isocosm builds at opt-level 1 in every workspace.** 2026-10-08, from
     646's measurement. Question: isocosm at opt-level 1 cut its suite from
     84 to 27 minutes for 29 s more compile, and a profile set in only some
     workspaces holds isocosm twice in the shared build directory (9.3 GB);
     where is it set? Options: every workspace that builds it, committed
     (recommended); isocosm's workspace only; the machine-local config;
     leave it at 0. Mark chose "Every workspace, committed (Recommended)".
     So each workspace manifest that builds isocosm carries
     `[profile.dev.package.isocosm] opt-level = 1`; Eponym's whole dev
     profile is already 1.

     Links: area Stack and engine · summary Every workspace manifest that builds isocosm carries a committed profile.dev.package.isocosm opt-level of 1.
649. **Traces label every critter.** 2026-10-08, briefing H2 (633 said
     entities in traces). Question: population is stored as cohorts, runs
     of identical critters with consecutive ids that both execution modes
     merge alike; how do traces label critters? Options: one entry per
     critter, each cohort's digest reused for its members (recommended);
     one entry per cohort. Mark chose "One entry per critter
     (Recommended)". So a divergence names the exact critter, and a trace
     line holds as many entries as the world has critters. Notes needed no
     ruling: the list is append-only, so `note:<index>` is stable.

     Links: area Record and history · draws on 633 · summary Traces carry one entry per critter, each cohort's digest reused for its members, so a divergence names the exact critter.
651. **H2's saves move to v3.** 2026-10-08, briefing H2 on mere's
     `state-witness` (`329d60d0`). Question: a v2 checkpoint stores a tick
     and a u64; how do saves carry the per-field entries (633) at each
     epoch? Options: extend v2 with an optional field (recommended); move to
     v3. Mark chose "Move to v3". So `SAVE_VERSION` becomes 3 with entries
     required, and v2 joins v1 as a kept reader, checked by its own hashes.

     Links: area The sim's schema and saves · draws on 634, 633 · summary Save version becomes 3 with per-field entries required; v2 and v1 stay as kept readers checked by their own hashes.
652. **The state hash is the entries' digest.** 2026-10-08. Question: H1's
     hash is one FNV over the whole state's postcard bytes, while the
     crate's `Witness::digest` is balaur's fold over the entries, a
     different number for the same state; what does the state hash become?
     Options: keep H1's hash (recommended); use the entries' digest. Mark
     chose "Use the entries' digest". So `state_hash` is `Witness::digest`
     over the per-field entries, one number with one derivation, and H1's
     whole-state hash survives only to read v2 saves. *Reading, not ruled:*
     the entries include the world's seed, revision and traits beside the
     state's fields, so the digest covers what H1's covered.

     Links: area Record and history · draws on 607, 633 · summary state_hash is Witness::digest over the per-field entries; H1's whole-state hash survives only to read v2 saves.
653. **Per-tick entries go in a framed trace.** 2026-10-08. Question: how do
     the bench and probe write per-tick entries (610), a line holding one
     entry per critter (649)? Options: framed postcard under a new flag
     (recommended); JSON Lines. Mark chose "Framed postcard, new flag
     (Recommended)". So `--trace` keeps its text lines and
     `--trace-entries <file>` writes the crate's framed `Trace` (MERETRC,
     version 1), compared by `first_trace_divergence`.

     Links: area Record and history · draws on 610, 649 · summary Per-tick entries go in a framed postcard trace written by a new --trace-entries flag; --trace keeps its text lines.
657. **The nervous system carries no matter.** 2026-10-08, finishing
     checkpoint 9 (amends 489's table). Question: checkpoint 9's control for
     589 (the muscular system carries the limbs' share of the bite) failed:
     489's table gives the nervous system (sense to contract) and the
     respiratory (respire to contract) `contract` as an effect too, and 582
     reads every carried system routing a function as one network, so a
     grazer lamed in its muscular system alone still had its share carried
     by its nervous routes (10, 10, 13 and 4 mg against the same on four
     seeds; cutting every route to the contracting parts gave the limbless
     bite, 4, 4, 7 and 1). Options: fix the control, 582 standing
     (recommended); the muscular system alone carries the share, an
     exception to 582; the nervous system carries no matter, ceasing to name
     `contract` in the default table, which touches the respiratory
     system's `contract` effect too. Mark chose "Nervous carries no matter".
     So the nervous and respiratory systems name no effects in the default
     table, the muscular system is the only route to the contracting parts,
     and the control passes as written. *Annotation, 2026-10-08:* Mark,
     straight after: "The nervous system should allow the musculature,
     digestive, respiratory, and neurological systems to communicate." That
     is its purpose, replacing the reading first written here (that it would
     carry perception's signal), and 658 takes up how.

     Links: area Bodies and anatomy · amends 489 · summary The nervous and respiratory systems name no effects in the default table; the muscular system is the only route to the contracting parts.
659. **The nervous system joins systems for a process.** 2026-10-08, 658's
     design round. Question: what does its communication do, given that
     each system carries milligrams from sources through gates to effects,
     wing-functions' gates are open or shut (477), and the bite already
     reads two systems (intake for the meal, contract for the limbs'
     share)? Options: it joins systems for a process (recommended); it opens
     the other systems' gates; it carries timing. Mark chose "It joins
     systems for a process (Recommended)". So a process reading several
     systems gets their joint effect only where nervous routes connect those
     systems' parts, and each system still works alone.

     Links: area Bodies and anatomy · draws on 657 · summary The nervous system joins systems for a process: a process reading several systems gets their joint effect only where nervous routes connect their parts.
660. **Any gated system can be joined.** 2026-10-08. Question: which systems
     does it connect? Options: the four Mark named, muscular, digestive,
     respiratory and nervous (recommended); every system with gates. Mark
     chose "Every system with gates". So the joining is not limited to a
     list: any process reading two or more gated systems is joined by nerves.

     Links: area Bodies and anatomy · draws on 659 · summary Any process reading two or more gated systems is joined by nerves; the joining is not limited to a list of systems.
661. **Without nerves, systems work alone.** 2026-10-08. Question: what
     happens to a body with no nerves, or with them cut, such as the probe's
     fronds? Options: systems work alone (recommended); local reflex only;
     those systems stop. Mark chose "Systems work alone (Recommended)". So
     only the joint effect is lost: a frond is unaffected, and a nerve cut
     costs coordination, not life.

     Links: area Bodies and anatomy · draws on 659 · summary Without nerves systems work alone; only the joint effect is lost, a nerve cut costing coordination, not life.
663. **A sense reaching both systems connects them.** 2026-10-08. Question:
     what counts as the nerves connecting two systems' parts, routes running
     along attachments through live parts (477) and the senses being the
     nervous system's sources? Options: a sense reaches both (recommended);
     any chain of parts expressing a nervous function. Mark chose "A sense
     reaches both (Recommended)". So a body joins its systems through what it
     senses: a nervous route from a sense source must reach a part of each,
     and a grazer with no eye has no joint bite.

     Links: area Bodies and anatomy · draws on 659 · summary A sense reaching both systems' parts connects them; a grazer with no eye has no joint bite.
664. **The joint effect is by degree.** 2026-10-08. Question: all or nothing,
     or by degree? Options: by degree, as systems work (477) (recommended);
     all or nothing. Mark chose "By degree (Recommended)". So the joint share
     scales with the share of the second system's parts the nerves reach: one
     limb innervated of two gives about half the limbs' share.

     Links: area Bodies and anatomy · draws on 659 · summary The joint effect is by degree, scaling with the share of the second system's parts the nerves reach.
670. **In-site space is isometer's; the sim takes it through isometer.**
     2026-10-08, the families plan's second round. Question: legacy
     `places` gives Mesocosm its space inside a site and Eponym uses it,
     while native Isocosm is site-grained (591: "Isocosm has no space inside
     a site"); where does in-site space go? Options: a sim noun after the
     place graph's SP3 to SP5 (recommended); isometer's, not the sim's;
     game-side. Mark: "1 through 2. Sounds like isometer's ambit, but that
     the sim should take it through isometer." Put back as read, Mark
     confirmed: "Yes, that's it". So isometer owns in-site space, and
     Isocosm's places family consumes it through isometer rather than
     keeping a copy, re-expressed after SP3 to SP5.

     Links: area Stack and engine · amends 392, 393 · summary In-site space is isometer's; Isocosm's places family consumes it through isometer rather than keeping a copy.
671. **Non-sapient critters are never driven; Mesocosm's driving ends with
     its world family's move.** 2026-10-08. Question: directing is built
     only on Isocosm (194, the overlay plan's M3), while legacy Mesocosm
     drives its critter through a sixteen-variant `Intent`; what happens to
     direct driving when the world family moves? Options: keep driving until
     M3 (recommended); the world waits for M3; drop driving at the move.
     Mark: "Wait. Driving is ok, as long as the creature is sapient. That's
     the dividing line, right?" Answered from 60 ("Driving is paredros.
     You're one sophont…") and 175, and put back, Mark chose "Drop it at
     the move". So driving is the sapient player's mode, a critter that is
     not sapient is directed, the legacy driving does not carry over, and
     Mesocosm is not playable between its world family's move and M3.

     Links: area Directing and play · draws on 60 · summary Driving is the sapient player's mode; a non-sapient critter is directed; legacy Mesocosm driving ends with its world family's move.
673. **Parked rungs' code moves; the rungs stay parked.** 2026-10-08.
     Question: glyphs, magic and neighbouring worlds pull no code lane yet
     (279); do Eponym's glyphs and the chronicles move with their families?
     Options: move as code, rungs parked (recommended); stay legacy until
     unparked. Mark chose "Move as code; rungs stay parked (Recommended)".
     So that code re-expresses with its family, unchanged in behaviour, and
     279 still bars new design on those rungs.

     Links: area Organs: glyphs, impresa, hagiograph · draws on 279 · summary Parked rungs' code (Eponym glyphs, chronicles) re-expresses with its family unchanged in behaviour; the rungs stay parked.
674. **A body's geometry goes through isometer; its physiology stays the
     sim's.** 2026-10-08, from Mark after 670: "I would figure that's how
     the sim would relate to movement, reach, and bodies too." Question:
     which half of a body? Isocosm's native `Part` holds geometry (parent,
     shape, extents, offset, situs) and physiology (functions, cells,
     matter, the systems of checkpoints 6 to 9); isometer's `BodyDocument`
     holds geometry and identity, and isometer-core names no world or
     organism. Options: geometry through isometer (recommended); whole
     bodies through isometer; bodies stay the sim's. Mark chose "Geometry
     through isometer (Recommended)". So a body's shape is isometer's body
     document, and the sim keeps its physiology keyed by isometer's part
     ids, one geometry for every game and the lens.

     Links: area Bodies and anatomy · draws on 670 · summary A body's geometry is isometer's body document; the sim keeps physiology keyed by isometer's part ids.
675. **Movement and physical reach go through isometer; the reach field stays
     the sim's.** 2026-10-08. Question: a body's movement in a site and its
     physical reach would read isometer's in-site space, while the sim's
     reach field (`reach.rs`, sim plan §4) is when news arrives at each site,
     the record's; does that split hold? Options: space through isometer,
     news the sim's (recommended); everything spatial through isometer.
     Mark chose "Yes: space through isometer, news the sim's (Recommended)".

     Links: area Stack and engine · draws on 670 · summary Movement and physical reach read isometer's space; the reach field (when news arrives at a site) stays the sim's.
677. **The overlay's M2 is the families plan's Mesocosm share.** 2026-10-08,
     the Mesocosm overlay plan's design round. Question: M2 ("absorption by
     family") predates 591 and 666 to 676, and the families re-expression
     plan carries that work for all three games; what becomes of M2?
     Options: M2 is the families plan's Mesocosm share (recommended); keep
     M2 as its own phase. Mark chose "M2 is the families plan's Mesocosm
     share (Recommended)". So M2 points to the families plan for Mesocosm's
     families and keeps its own condition: the web each level enables stays
     alive in most draws (260), measured by persistence, turnover, collapse
     and response to intervention (267, 283).

     Links: area Mesocosm · draws on 260, 591 · summary Overlay M2 is the families plan's Mesocosm share and keeps its own condition: the web stays alive in most draws.
679. **The driving intents become the critter's own acts; `Idle` goes.**
     2026-10-08, ruling the overlay plan §3's two readings. Question: the
     old `Move`, `Metabolize`, `Consume`, `Graft`, `Deposit` and `Carve`
     were driving; `Idle` advanced a tick without acting. Options: rule both
     as read (recommended); rule the six and keep `Idle` as a player act.
     Mark chose "Rule both as read (Recommended)". So the six are the
     critter's own acts under directing, chosen by its methodology, and
     `Idle` has no role.

     Links: area Directing and play · draws on 671 · summary The six old driving intents become the critter's own acts chosen by its methodology; Idle has no role.
682. **M3 plays a native generated lineage.** 2026-10-08. Question: native
     generation (`bodied.rs`) makes lineages today, while re-expressing
     Mesocosm's founders waits on the bodies family and the families plan's
     fork 9; which critter does M3 play? Options: a native lineage
     (recommended); the founders, re-expressed. Mark chose "Native lineage
     (Recommended)". So M3 does not wait on the bodies family, and the
     founders join when it and fork 9 land.

     Links: area Mesocosm · summary M3 plays a native generated lineage; the founders join when the bodies family and fork 9 land.
683. **The critter's choice is a deliberative methodology.** 2026-10-08.
     Question: the scheduler runs every due process for each ready group, so
     nothing chooses one act over another and a nudge has nowhere to be
     weighed; where does the choice live? Options: a scheduler methodology on
     the unused `Method::Deliberative` (recommended); nudges as kept state
     that ordinary processes read through `Query`. Mark chose "Deliberative
     method (Recommended)". So a deliberative critter chooses one Choice
     process, weighing needs, mood, nudges and bond, which is 679's "chosen
     by its methodology", and `Receipt.foregone` records what it passed over.

     Links: area Directing and play · draws on 679 · summary A deliberative critter chooses one Choice process weighing needs, mood, nudges and bond; Receipt.foregone records what it passed over. (amended by 813)
684. **The interim boundary ports grow-a-copy scoring.** 2026-10-08.
     Question: the interim M4's boundaries adapt every lineage (57, 182),
     while adaptation belongs to the lineages family, fifth, after places;
     legacy `adapt_round` scores a candidate by growing a copy, and native
     `Session::fork_at` exists. Options: port grow-a-copy over `fork_at`
     (recommended); pull the lineages family ahead of places; a boundary
     without adapting. Mark chose "Port grow-a-copy (Recommended)". So
     adaptation's scoring lands natively ahead of its family, and 195 and
     668's order stands.

     Links: area Ecology · draws on 57, 182 · summary The interim boundary ports grow-a-copy adaptation scoring natively over fork_at, ahead of the lineages family; 195's order stands.
685. **A suggestion is a derived reading.** 2026-10-08, the directing
     survey's second round. Question: what is a suggestion that surfaces
     (the contract's `EventRecord` is a generic topic and a byte payload;
     `Receipt.foregone` lists the Choice processes passed over; `knows`
     records what the critter knows)? Options: a reading over `foregone`
     and `knows` (recommended); a new contract event; wait for senses. Mark
     chose "Derived reading (Recommended)". So a suggestion is what the
     critter considered and could act on, read from its receipts and what
     it knows, and nothing new is kept.

     Links: area Directing and play · draws on 683 · summary A suggestion is a derived reading over foregone and knows; nothing new is kept.
686. **The runtime translates the contract.** 2026-10-08. Question: which
     way does the contract dependency run (Isocosm does not depend on
     `isocosm-overlay`, 1,256 lines and used by nothing, and the overlay
     depends on nothing sim-internal, D18)? Options: the runtime over
     `Session` translates envelopes into native commands (recommended);
     Isocosm's `Command` takes the contract's types. Mark chose "Runtime
     translates (Recommended)". So Isocosm and the contract stay
     independent, and the nudge enters the sim as a new logged native
     command.

     Links: area Directing and play · summary The runtime over Session translates contract envelopes into native commands; Isocosm and the contract stay independent; a nudge is a new logged command.
687. **A region at site grain groups sites by biomass.** 2026-10-08.
     Question: what is a region at site grain, for the local collapse
     played on (225, 226)? Options: one site (recommended); sites grouped by
     biomass, needing a grouping rule. Mark chose "Sites grouped by
     biomass". So the interim M4 carries 226's regions at site grain: each
     trophic level fills its slot in a region of sites until over capacity,
     then spills into a new one, and the regions merge as biomass falls.
     The grouping rule is put as its own question.

     Links: area Ecology · draws on 226 · summary At site grain a region groups sites by biomass: each trophic level fills its slot until over capacity, then spills; regions merge as biomass falls.
688. **The bond is a weighted relation to a participant.** 2026-10-08.
     Question: what is the bond (177, 178)? Options: an account
     (recommended); `Tenet.trust`; a relation to a participant outside the
     world. Mark: "i can't think of how to do 1 for different players
     (coop) without doing 3, and 3 sounds fascinating". Put back: the
     participant becomes an entity with no place, as the world is one
     (98), and native `Relation` (subject, kind, object) has no strength;
     where does the strength live? Options: a weighted relation
     (recommended); a relation plus an account keyed by participant;
     accounts only, with no participant. Mark chose "Weighted relation
     (Recommended)". So a participant is a placeless entity, `Relation`
     gains a value, and each player's bond is its own weighted relation to
     the critter, moving with outcomes, which serves coop. *Reading, not
     ruled:* 178's setting seeds the weight at birth.

     Links: area Directing and play · draws on 177, 178 · summary A participant is a placeless entity and Relation gains a value; each player's bond is its own weighted relation to the critter.
689. **Regions are derived each round.** 2026-10-08, with 687. Question: how
     are 226's regions held at site grain? Options: derived each round from
     site biomass and site routes, never kept (recommended); kept and
     updated. Mark chose "Derived each round (Recommended)". So a region is
     a reading, as mood is (227): a world rule sets each trophic level's
     slot capacity, and a region grows along adjacent sites, so geography
     counts.

     Links: area Ecology · draws on 226, 687 · summary Regions are derived each round from site biomass and routes, never kept; a world rule sets each trophic level's slot capacity.
690. **Standing orders are derived from the history.** 2026-10-08.
     Question: where do 216's standing orders live? Options: derived from
     the logged nudges and their outcomes (recommended); kept on the
     entity. Mark chose "Derived from history (Recommended)". So range,
     home, priorities and stances are desire paths (59) read from the log,
     replaying exactly, and at site grain a contract `PlaceHandle` names a
     site.

     Links: area Directing and play · draws on 59, 216 · summary Standing orders are derived from logged nudges and their outcomes, replaying exactly, not kept on the entity.
691. **Survival mode filters through what the critter knows.** 2026-10-08.
     Question: how does survival mode (180, 184) filter at site grain, with
     the sense function still running nothing? Options: through `knows` and
     the reach field (recommended); defer until senses. Mark chose "Through
     knows (Recommended)". So the interim M4 ships both modes, and senses
     refine the filter when they run.

     Links: area Directing and play · draws on 180, 184 · summary Survival mode filters through what the critter knows and the reach field; the interim M4 ships both modes.
695. **parry-ground folds into conatus.** 2026-10-08. Question: the probe
     is a 481-line binary with no consumers, `GroundCollision` already
     wraps `Voxels` privately, and mesocosm-runtime's `tactile.rs` already
     feeds the ground into a conatus voxel collider; does it keep calling
     parry directly? Options: stay on parry (recommended); fold into
     conatus. Mark chose "Fold into conatus". So the probe's checks are to
     run through the stack's conatus voxel collider and the direct parry
     dependency retires (consolidate into the stack). This is a new
     objective with its own assessment: what conatus exposes for the
     probe's ray, point and manifold checks, and whether the receipt's
     figures survive the move.

     Links: area Stack and engine · summary parry-ground folds into conatus: its checks run through conatus's voxel collider and the direct parry dependency retires.
696. **The sim records edits; isometer applies them and derives places.**
     2026-10-08, the in-site space survey's round, under 670. Question: SP4
     and SP5 have no code; 413 stores edits as shape operations, "an
     asserted fact in the sim's state and history", and 422 lets an entity
     name its patch or room; who holds what? Options: the sim records,
     isometer applies (recommended); isometer owns the edit log; both in
     the sim. Mark chose "Sim records, isometer applies (Recommended)". So
     the sim keeps each edit as an asserted fact and an entity's place id,
     and isometer applies edits to its volume and derives places, passages
     and travel cost.

     Links: area Stack and engine · draws on 670, 413, 422 · summary The sim records each edit as an asserted fact and an entity's place id; isometer applies edits and derives places, passages and travel cost.
697. **nisus is the store behind isometer, now.** 2026-10-08. Question: 330
     made mere's nisus the voxel authority, and 670 gives in-site space to
     isometer, whose `Ground` stores it today; how do they fit? Options:
     isometer's API with nisus under it later, under SP6 (recommended);
     nisus as the store now; retire 330. Mark chose "Nisus as the store
     now". So 670 governs the interface and 330 the storage, and nisus goes
     in behind isometer with SP4 rather than waiting for SP6.
     *Annotation, 2026-10-08:* put back as 700 with T2 still unbuilt; it
     stands.

     Links: area Stack and engine · draws on 330, 670 · summary Nisus is the store behind isometer now, with SP4; 670 governs the interface and 330 the storage.
698. **SP4 and SP5 are built in isometer.** 2026-10-08. Question: the lift
     lives in isocosm (`src/terrain`, `src/map`, 1,040 lines) and SP3's
     adapter in genet's bench; where are SP4 and SP5 built? Options: in
     isometer, wgpu-free (recommended); in isocosm, then moved. Mark chose
     "In isometer (Recommended)". So edit application and places are an
     isometer-core module or a product-neutral isometer crate without wgpu,
     consumed by the sim directly.

     Links: area Stack and engine · draws on 670 · summary SP4 and SP5 (edit application and places) are built in isometer without wgpu and consumed by the sim directly.
699. **The seam keys by `PartId`, and situs joins the body document.**
     2026-10-08, under 674. Question: isometer's `PartId(u32)` is a vector
     index with tombstones and native ids are `u64` map keys; situs and
     shape have no isometer field; isometer's `mass_mg` would duplicate the
     sim's ledger. Options: key by `PartId`, situs and the shape name join
     `BodyDocument` (recommended); situs and shape stay sim-side; isometer
     adopts `u64` ids. Mark chose "PartId; situs joins the document
     (Recommended)". So the sim keys physiology by `PartId`, `BodyDocument`
     gains situs and the shape name as product-neutral fields, and
     `mass_mg` becomes a reading of the sim's ledger, not a second store.

     Links: area Bodies and anatomy · draws on 674 · summary The sim keys physiology by PartId; BodyDocument gains situs and shape name; mass_mg becomes a reading of the sim's ledger.
701. **The lift moves to isometer.** 2026-10-08. Question: does the lift
     (isocosm's `src/terrain` and `src/map`, 1,040 lines) move? Options: the
     lift to isometer, the world map and skeleton staying the sim's
     (recommended); it stays, with an adapter; everything moves. Mark chose
     "Lift moves to isometer (Recommended)". So site grain stays the sim's,
     and lifting a site into voxels is isometer's, beside SP4 and SP5 (698).

     Links: area Stack and engine · draws on 698 · summary Lifting a site into voxels moves to isometer beside SP4 and SP5; site grain and the world map stay the sim's.
702. **Walking, routing and sight are rewritten over SP5.** 2026-10-08.
     Question: legacy `near.rs` (369 lines) uses only `Ground` and `Aabb`;
     675 sends movement and physical reach through isometer and 597 keeps
     Eponym's conatus motion game-side. Options: into isometer as they are
     (recommended); rewritten over SP5's passages; back to the games. Mark
     chose "Rewrite over SP5". So isometer's walking, routing and sight are
     built over SP5's passages and travel cost, not raw voxels, and the
     legacy searches retire with their family. *Reading, not ruled:*
     Eponym's conatus motion stays game-side (597).

     Links: area Stack and engine · draws on 675, 597 · summary Isometer's walking, routing and sight are built over SP5's passages and travel cost, not raw voxels; legacy searches retire with their family.
703. **Soil, the tier line and the coarse partition split now.** 2026-10-08.
     Question: where do legacy soil, the tier line and the coarse `Places`
     partition go? Options: split now (recommended); decide at the family.
     Mark chose "Split now (Recommended)". So soil and its diffusion stay
     the sim's, keyed by isometer's columns, which settles 343's "whether
     columns are sites"; the tier line goes to the host or directing; and
     SP5's places supersede the coarse partition.

     Links: area Stack and engine · draws on 670 · summary Soil and its diffusion stay the sim's keyed by isometer's columns; the tier line goes to host or directing; SP5 places replace the coarse partition.
704. **Checkpoint 10's wounds come from an agentless hazard.** 2026-10-08,
     checkpoint 10's brief, first round. Question: what inflicts a wound in
     the probe before 123's fights exist (the probe's fights are between
     partless members, 504's extentless control; the bodies probe contests
     nothing; a dev command is not drawn)? Options: an agentless hazard at
     a per-world drawn rate (recommended); a bite past a threshold; fights
     joined to bodies. Mark chose "Agentless hazard (Recommended)". So a
     drawn hazard wounds a part, rate zero is the negative control, and
     123's fights reuse the same wound effect when they come.

     Links: area Bodies and anatomy · summary Checkpoint 10's wounds come from an agentless hazard at a per-world drawn rate; rate zero is the negative control.
705. **A wound is its own effect.** 2026-10-08. Question: how is a wound
     told from a bite (469: a bite takes tissue, a wound takes cells; 496
     passed on "a bite to the gut takes intake")? Options: its own `Effect`
     with its own part draw (recommended); `Eat` gains a wound flag. Mark
     chose "Its own Effect (Recommended)". So `Eat` stays tissue only, and
     in the flow record a spill goes from part to site in the part's own
     matter while a bite goes from part to eater.

     Links: area Bodies and anatomy · draws on 469, 704 · summary A wound is its own Effect with its own part draw; Eat stays tissue only.
706. **A part keeps its lost cells.** 2026-10-08. Question: capacity and
     adult mass read only from `half_extent` today, which 674 makes the
     body document's; how does a wound shrink them? Options: a lost-cells
     field that capacity and ceiling subtract (recommended); shrink
     `half_extent`. Mark chose "A lost-cells field (Recommended)". So
     physiology never writes geometry, and 674's move stays mechanical.

     Links: area Bodies and anatomy · draws on 674, 469 · summary A part keeps a lost-cells field that capacity and ceiling subtract; physiology never writes geometry.
707. **Severing follows isometer's semantics.** 2026-10-08. Question:
     native flags only the cut part (its children still count in `held`),
     isometer tombstones the whole subtree with stable ids and refuses the
     root, and a bud removes the part outright; what does severing record?
     Options: isometer's semantics (recommended); remove like a bud; the
     cut part only. Mark chose "Isometer's semantics (Recommended)". So a
     severing tombstones the whole subtree, keeps ids, never takes the
     root, and moves the tombstones' matter out, closing the gap where
     `anatomy::books` counts a tombstone's matter that `held` and `room`
     skip.

     Links: area Bodies and anatomy · draws on 674 · summary Severing tombstones the whole subtree, keeps ids, never takes the root, and moves the tombstones' matter out.
708. **A wound spills everything over its lowered bound.** 2026-10-08,
     checkpoint 10's second round. Question: 469 spills "tissue it can no
     longer hold", but the store and reproduce bounds (`anatomy::bound`)
     also derive from capacity; what spills? Options: all over its bound
     (recommended); tissue only. Mark chose "All over its bound
     (Recommended)". So tissue, reserve and provision each spill what
     exceeds their lowered bound, and no account sits above its bound.

     Links: area Bodies and anatomy · draws on 469 · summary A wound spills tissue, reserve and provision over their lowered bounds, so no account sits above its bound.
709. **Free cells count in a wound's proportion.** 2026-10-08. Question:
     do unallocated cells count in 496's proportion? Options: free cells
     count (recommended); allocated only. Mark chose "Free cells count
     (Recommended)". So a wound takes cells across every function and the
     free pool together, and free cells cushion it.

     Links: area Bodies and anatomy · draws on 496 · summary Free (unallocated) cells count in a wound's proportion and cushion it.
710. **Losing every cell severs a part, or kills at the root.**
     2026-10-08. Question: what happens when a wound takes all of a part's
     cells? Options: it severs, the root killing (recommended); a
     zero-capacity stub. Mark chose "Severs; root kills (Recommended)". So
     a non-root part with no cells left is severed under 707, and a root
     with none dies.

     Links: area Bodies and anatomy · draws on 707 · summary A non-root part with no cells left is severed; a root with none dies.
711. **Severing comes only from losing every cell.** 2026-10-08. Question:
     Eponym's strike severs past `sever_threshold` (24), and 669 hands
     strike resolution to the game; can a wound sever before all cells are
     lost? Options: only at all cells (recommended); a world-rule
     threshold; a cause that names it. Mark chose "Only at all cells
     (Recommended)". So severing has one mechanism, and a game's strikes
     reach it by wounding hard enough.

     Links: area Bodies and anatomy · draws on 710 · summary Severing comes only from losing every cell; a game's strikes reach it by wounding hard enough.
712. **Healing lands just before tissue.** 2026-10-08, checkpoint 10's
     third round. Question: 496 heals "before it fills tissue or grows new
     parts"; a hungry body lands reserve, tissue, grow, provision and a fed
     one tissue, grow, provision, reserve; where does healing sit? Options:
     just before tissue (recommended); first in both. Mark chose "Just
     before tissue (Recommended)". So a hungry body lands reserve, heal,
     tissue, grow, provision, and a fed one heal, tissue, grow, provision,
     reserve.

     Links: area Bodies and anatomy · draws on 496 · summary Healing lands just before tissue: hungry bodies land reserve, heal, tissue, grow, provision; fed ones heal first.
713. **Regrowing from a fragment is a second trait.** 2026-10-08.
     Question: 485 makes healing and regrowing parts one trait; is
     regrowing from a fragment (470) its own? Options: a second trait,
     drawn per lineage like 548's eight (recommended); one trait for both.
     Mark chose "A second trait (Recommended)". So a lineage can heal
     without its fragments living, and the anatomy brief §7's reading is
     ruled.

     Links: area Bodies and anatomy · draws on 485, 470 · summary Regrowing from a fragment is a second trait drawn per lineage, separate from healing.
714. **A living fragment re-roots as a seedling does.** 2026-10-08.
     Question: a limb's subtree holds no recipe root, so `lacking()` grows
     nothing; how does a living fragment grow? Options: re-root like a
     seedling (recommended); grow toward the root; re-develop the recipe.
     Mark chose "Re-root like a seedling (Recommended)". So the fragment's
     top part becomes part 0 at situs `[0,0,0]`, as a bud does, and it
     grows its recipe from there.

     Links: area Bodies and anatomy · draws on 470 · summary A living fragment re-roots as a seedling does: its top part becomes part 0 at situs [0,0,0] and grows its recipe from there.
715. **A living fragment is a birth, unprovisioned.** 2026-10-08. Question:
     a bud is `Born(lineage)`, spends provision (518) and counts among
     births and the living; is a fragment a birth? Options: a birth without
     provision (recommended); not a birth. Mark chose "A birth,
     unprovisioned (Recommended)". So a living fragment is `Born(lineage)`,
     counted among births and the living with a parent relation (554),
     carrying its own matter, its systems dormant where it cannot realize
     them (583).

     Links: area Bodies and anatomy · draws on 470, 554 · summary A living fragment is a birth without provision, counted among births and the living with a parent relation, systems dormant where unrealizable.
716. **A hazard's wound lands on a part drawn by cells.** 2026-10-08,
     checkpoint 10's fourth round. Question: bites draw by edible holdings
     (`anatomy::bitten`) and 705 gives the wound its own draw; which part?
     Options: by living cells (recommended); by tissue; uniform. Mark chose
     "By cells (Recommended)". So bigger parts are hit more often, structure
     for structure, apart from where the food is.

     Links: area Bodies and anatomy · draws on 705 · summary A hazard's wound lands on a part drawn by living cells, so bigger parts are hit more often.
717. **Severing bumps the body's revision; a wound does not.** 2026-10-08.
     Question: Allocate and trait marks bump `body_revision`, invalidating
     `Query::Part` bindings, and Eponym's `injure` bumps its own; do harms?
     Options: severing bumps, wounds don't (recommended); both bump. Mark
     chose "Severing bumps, wounds don't (Recommended)". So a severing
     re-resolves bindings, and a bound process keeps its wounded part and
     reads its new capacity.

     Links: area Bodies and anatomy · draws on 707 · summary Severing bumps the body's revision and re-resolves bindings; a wound does not.
718. **Checkpoint 10 builds rot; eating carrion comes later.** 2026-10-08.
     Question: 470's dead fragments are "eaten, incorporated or left to
     rot", but nothing eats or rots a dead body today; what does checkpoint
     10 build? Options: rot here, eating later (recommended); eating too;
     leave as carrion. Mark chose "Rot here, eating later (Recommended)". So
     a dead body's matter returns to the site at a drawn rate, closing the
     conservation loop, and eating carrion comes with 123's scavenging or
     the lineages family.

     Links: area Ecology · draws on 470 · summary A dead body's matter returns to the site at a drawn rate (rot) in checkpoint 10; eating carrion comes later.
719. **A regrown part revives its tombstone.** 2026-10-08. Question: under
     707 a tombstone keeps its id and situs, and `lacking()` refuses a
     situs held, severed or not; what id does a regrown part take? Options:
     a new id, the tombstone kept (recommended); revive the tombstone. Mark
     chose "Revive the tombstone". So healing's regrowth clears the severed
     flag on the part at that situs and gives it new cells, and an id names
     the part at its place in the plan across a loss. *Reading, not ruled:*
     the subtree regrows the same way, part by part from the top, and
     isometer gains the matching operation (it has `sever` and no revival)
     with the bodies family's move (676).

     Links: area Bodies and anatomy · draws on 707 · summary A regrown part revives its tombstone: healing clears the severed flag at that situs and gives it new cells.
720. **Isometer's fixtures become its own.** 2026-10-08, the in-site space
     survey's last round. Question: isometer-lens, -mesh and -render
     dev-depend on isocosm, and 16 of their files import `isocosm::legacy`
     as fixtures, which break when legacy places retires (666); what
     replaces them? Options: isometer-local fixtures (recommended); native
     isocosm draws. Mark chose "Isometer-local fixtures (Recommended)". So
     the fixtures are rebuilt from isometer's own ground and body documents,
     and isometer's tests no longer depend on the sim it serves.

     Links: area Stack and engine · summary Isometer's fixtures are rebuilt from its own ground and body documents so its tests no longer depend on the sim.
721. **Lineage vocabulary leaves isometer-core with the bodies family.**
     2026-10-08. Question: isometer-core carries Mesocosm's lineage words 39
     times (`SpeciesId`, `Origin::Incorporated{from_species}`,
     `Provenance.epoch`, `BodyPlan`); under 674 it holds one geometry for
     every game; does that vocabulary leave? Options: it leaves with the
     bodies family (recommended); it stays. Mark chose "Leaves with the
     bodies family (Recommended)". So when 674 lands, provenance and species
     ids move to the sim, keyed by `PartId` (699), isometer keeping geometry
     and an opaque origin tag, and `BodyPlan`'s shape classes stay as
     product-neutral geometry.

     Links: area Stack and engine · draws on 674, 699 · summary Lineage vocabulary leaves isometer-core with the bodies family; provenance and species ids move to the sim keyed by PartId.
724. **The folded probe stays a standalone isometry probe.** 2026-10-08.
     Question: where does it live? Options: its own workspace under
     `crates/probes`, pinned to mere by rev (recommended); `TactileWorld`'s
     tests; a receipt in mere, losing every `Ground` figure. Mark chose
     "Standalone isometry probe (Recommended)". *Reading, not ruled:* its
     name no longer fits once parry goes, and a rename is a naming round.

     Links: area Stack and engine · draws on 695 · summary The folded probe stays a standalone isometry probe in its own workspace under crates/probes, pinned to mere by rev.
735. **The kiss3d fork: v0.47.0, a mark-ik fork, a mere crate over it.**
     2026-10-09. Evidence: cargo's git cache holds dimforge/kiss3d at
     `28cdddd` (2026-08-29, past v0.46.0); v0.47.0 (2026-10-04), which
     nexus pins, is not on disk. Questions and Mark's choices: the base,
     "v0.47.0, fetched (Recommended)" over the cached commit; the home,
     "crates/kiss3d, mark-ik fork (Recommended)", pushed and pinned by rev,
     over a local-only checkout; the seam, "A mere crate over the fork
     (Recommended)", the crate owning device, targets, light block and
     depth pre-pass with the fork beneath, over games using the fork
     directly. So one fetch of upstream kiss3d is allowed, 606's
     commit-by-name rule governs what comes in from balaur, and the mere
     crate takes a plain working name until a naming round.

     Links: area Stack and engine · draws on 471, 606 · summary The kiss3d fork is v0.47.0 at crates/kiss3d (mark-ik fork, pinned by rev), with a mere crate over it owning device, targets, lights, depth pre-pass.
736. **Palette colours reach kiss3d as a texture addressed by UV.**
     2026-10-09, from the renderling inventory. Evidence: kiss3d's meshes
     carry no vertex colours (`resource/mesh3d.rs:143`), while Eponym's room
     and body meshes carry one RGBA per vertex, about one per greedy quad
     from isometer's palette. Options: a palette texture by UV
     (recommended); a vertex-colour attribute in the fork. Mark chose
     "Palette texture by UV (Recommended)". So the fork's diff stays
     narrower and the colours map onto glTF's base-colour texture (§4.2).
     *Reading, not ruled, from the same inventory:* 471's caller-target
     entry also records into the caller's encoder with no internal submits
     and returns an encode report, as netrender's opaque-tenant input
     requires; the tenant crate's API uses its own plain types, so games
     never see kiss3d's glam 0.33 beside their 0.30; rg3c's
     renderling-specific counts are rewritten against that report after
     the push (732). The inventory also found `field-bake` already retired
     (`bb92f2e8`), leaving `ambience-lease` the one probe 474 archives.

     Links: area Stack and engine · draws on 735 · summary Palette colours reach kiss3d as a texture addressed by UV, keeping the fork's diff narrow.
737. **A room is roofed air.** 2026-10-09, the places lane's forks (merged
     at `2a26c625`). Question: 417's "air cut off from the sky", read as no
     air path to the sky, floods an open-mouthed cave into the outdoor patch;
     read as solid anywhere overhead, the cave is a room; both are built.
     Options: roofed (recommended); sealed; a world rule. Mark chose "Roofed
     (Recommended)". So caves, tunnels and lintelled doorways are places
     with clearance on their edges, and `Cover::Roofed` is the default.

     Links: area Stack and engine · draws on 417 · summary A room is roofed air: caves, tunnels and lintelled doorways are places with clearance on edges, and Cover::Roofed is the default.
738. **A patch splits at narrow necks.** 2026-10-09. Question: clearance
     lives only on edges (418), so a 1-wide gap inside one large patch lets
     a 2-wide body through. Options: split where walkable width drops below
     a world-rule width (recommended); accept it; record each patch's
     narrowest width. Mark chose "Split at narrow necks (Recommended)".

     Links: area Stack and engine · draws on 418 · summary A patch splits where walkable width drops below a world-rule width.
739. **An edit's mass is credited by material.** 2026-10-09. Question: 412's
     density and matter account have no defaults and the sim names no
     carver. Options: density and account on `Material`, set at founding,
     crediting the acting entity's ledger, dev source only until set
     (recommended); one density; cells, not mass. Mark chose "Density and
     account per material (Recommended)".

     Links: area Ecology · draws on 412 · summary An edit's mass is credited by material: density and account sit on Material, set at founding, crediting the acting entity's ledger.
740. **An entity names its patch through an optional field.** 2026-10-09.
     Question: `Entity.place` is a site id, and a cohort's members share a
     record. Options: `patch: Option<PlaceId>` on `Entity`, set only for
     members split out of their cohort (recommended); a `State` map; wait on
     275. Mark chose "Optional patch on Entity (Recommended)".

     Links: area The sim's schema and saves · draws on 422 · summary Entity gains an optional patch field, set only for members split out of their cohort.
741. **Sight is a voxel ray scoped by places.** 2026-10-09. Question: no
     ruling defines sight over SP5's passages (702). Options: a voxel ray,
     the place graph choosing candidates (recommended); visible sets per
     pair of places; same or adjacent places only. Mark chose "Voxel ray,
     place-scoped (Recommended)".

     Links: area Stack and engine · draws on 702 · summary Sight is a voxel ray with the place graph choosing candidates.
742. **The tier line goes to directing.** 2026-10-09. Question: 703 sends it
     to the host or directing. Options: directing, with the family's
     re-expression (recommended); the host now. Mark chose "Directing
     (Recommended)". So it moves with the directing plan's D2.
     *Noted 2026-10-09:* the places lane coined `isometer-space` and
     `Atlas` mid-session; both are working names awaiting a naming round.

     Links: area Directing and play · draws on 703 · summary The tier line goes to directing, moving with the directing plan's D2, not to the host now.
743. **The neck width defaults to 2.** 2026-10-09, the places lane's
     second pass (merged at `af3a0fc4`). Question: 738's split has no
     default, so nothing splits until a world sets one. Options: 2, so any
     1-wide neck splits (recommended); the roster's widest body; unset.
     Mark chose "Default 2 (Recommended)".

     Links: area Stack and engine · amends 738 · summary The patch neck width defaults to 2, so any 1-wide neck splits.
744. **Cliff spans are drawn, and the lift steps its surface there.**
     2026-10-09. Question: spans derived from the lift never stop at a
     cliff, two sites meeting on one continuous profile, so only water
     stops a route, while A.2 has spans "drawn from both sites' skeletons".
     Options: draw cliff spans from both skeletons and the pair seed, the
     lift stepping its surface there (recommended); no cliffs, a steep span
     being a climb; edits make cliffs. Mark chose "Draw cliff spans
     (Recommended)". So the lift's bytes change on drawn worlds, which 732
     lets land without re-certifying the lift's receipts until after the
     push.

     Links: area World generation · draws on 391 · summary Cliff spans are drawn from both sites' skeletons and the pair seed, and the lift steps its surface there.
746 to 748. **The tenant's context slot, report and water field.**
     2026-10-09. Mark chose "Keep the scoped slot (Recommended)" for 471's
     handle, "Submissions only (Recommended)" for the frame report, and
     "Define it with its reader (Recommended)" for 472's water field.
     Recorded in full in mere's conatus engine plan (`4d8bd703`), where the
     tenant's rulings live.
749. **L7 lands on the tenant; D1 layers; the torch's shadows wait.**
     2026-10-09, merged at `f291964b` with isometry repinned to mere
     `4d8bd703`: eponym-client draws through `tenant`; renderling,
     crabslab, 729's carriage and `ambience-lease` are gone, the forks
     archived to `Code/archive/` as 474 has it. Questions: how bodies go
     over the traced terrain, kiss3d's tonemap writing the caller's target
     unblended; whether the torch casts shadows; whether eponym's
     CLAUDE.md drops its two renderling lines. Mark chose "Separate
     targets, netrender layers (Recommended)", "Off until testing
     (Recommended)" and "Yes, update them (Recommended)". So the tenant
     draws bodies onto a transparent target that netrender layers over the
     traced colour, the torch casts no shadow until the testing pass tunes
     it, and the CLAUDE.md names the tenant.

     Links: area Stack and engine · draws on 471 · summary eponym-client draws through the tenant; bodies draw onto a transparent target netrender layers over traced colour; the torch casts no shadow until testing.
751. **A site is habitable when the lineage's conditions hold and a member
     lives there.** 2026-10-09, directing D1 to D4 (merged at `d852ceb0`).
     Question: what habitability means natively for a player-picked start
     (179). Options: the first epoch boundary at which a site satisfies
     every condition the lineage's processes require and a member lives
     there (recommended); mood at least zero; habitable from founding. Mark
     chose "Conditions met, member lives (Recommended)".

     Links: area Mesocosm · draws on 179 · summary A site is habitable at the first epoch boundary where it meets every condition the lineage's processes require and a member lives there.
753. **After a regional collapse, members migrate and the player may start
     again elsewhere.** 2026-10-09. Question: `Effect::Move` takes a fixed
     destination, so migration needs a generated process per route; or the
     player takes up a member in a region still standing (181). Options:
     start again elsewhere (recommended); generated move acts; both. Mark
     chose "Both". So founding generates a move process per route, and
     `OnCollapse::Elsewhere` stays available.

     Links: area Mesocosm · draws on 181 · summary After a regional collapse, founding generates a move process per route and OnCollapse::Elsewhere stays available.
754. **A region has collapsed only in a level it once held.** 2026-10-09.
     Question: regions that never held a trophic level read as collapsed
     from the first round, while 225 says "a level gone". Options: only
     levels once held, or that the world holds within reach (recommended);
     any empty level; grow a region until it holds every level. Mark chose
     "Only levels once held (Recommended)".
     *Noted 2026-10-09, not ruled:* Mark wondered whether neural networks
     have a role. Candidates raised: a learned surrogate for the
     statistical tier, certified by the crowd's draws; a small per-lineage
     choice network whose weights are heritable traits; generation;
     sophonts through gemot's tulpa lane; denoising in the tenant. Any of
     them must infer bit-identically or log its outputs, for replay.
     Parked as research for after 732's push.

     Links: area Ecology · amends 225 · summary A region has collapsed only in a trophic level it once held; the clause about levels the world holds within reach is unbuilt.
756. **Lineage data moves into isocosm; isometer keeps an opaque tag.**
     2026-10-09. Question: about 218 provenance and 200 `mass_mg` sites
     store lineage and mass in isometer's document. Options: `SpeciesId`,
     `Origin` and `Provenance` into isocosm, legacy sidecars keyed by
     `PartId`, a per-part opaque tag in isometer (recommended); defer. Mark
     chose "Into isocosm, opaque tag (Recommended)". This carries 721 and
     699's mass reading.

     Links: area Stack and engine · draws on 721, 699 · summary Lineage and provenance data (SpeciesId, Origin, Provenance) move into isocosm as sidecars keyed by PartId; isometer keeps an opaque tag.
757. **Authored content enters by `Command::Assert`, with the record.**
     2026-10-09, from the campaign lane (merged at `b37d7e91`). Question:
     native asserts no faction, place, character, law, fact or history
     line. Options: an assertion family carrying the contract's vocabulary,
     built with the record family, the campaign world becoming a reading of
     native state (recommended); build it now; the campaign stays authored
     until V2. Mark chose "Command::Assert with the record (Recommended)".

     Links: area The sim's schema and saves · summary Authored content enters by Command::Assert with the record family; the campaign world becomes a reading of native state.
758. **The datasheets feed native; families-plan fork 9 closes.**
     2026-10-09. Question: legacy `Founding`'s eight bodies come from 1,198
     embedded TOML lines (625, 626) while `bodied::roster()` hand-codes the
     same in Rust. Options: the sheet loader lowered onto native `Recipe`
     and `Template`, `roster()` reading the sheets, legacy `Founding` a name
     into them (recommended); keep both; native code authoritative. Mark
     chose "Sheets feed native (Recommended)".

     Links: area Method and documents · draws on 625 · summary The datasheet loader lowers onto native Recipe and Template; roster() reads the sheets; legacy Founding becomes a name into them.
760. **An authored faction is a polity with no members yet.** 2026-10-09.
     Question: campaign factions are authored without members while native
     derives factions from membership (63 to 68). Options: a polity whose
     constitution starts empty, members joining as relations (recommended);
     notes until members exist; tags and claims stay the table's. Mark
     chose "Polity with no members yet (Recommended)".
     *Annotation, 2026-10-09, Mark:* "Although to author it, surely there
     need to be some attributes assigned to the polity that can fill the
     placeholder". So `Command::Assert` asserts a polity with authored
     attributes, its name, tags and claims and the constitution's
     governance and focus, standing as asserted facts while it has no
     members. *Reading, not ruled:* once members join, readings derived
     from them sit beside the authored attributes rather than replacing
     them, and where the two disagree is a later question.

     Links: area The sim's schema and saves · draws on 63 · summary An authored faction is a polity with no members yet, asserted with authored name, tags, claims, governance and focus as facts.
764. **Lexicon lessons are discovery.** 2026-10-10. Question: legacy
     discovery's condition table, with its endurance route, against native's
     lessons from eating a part whole (468). Options: native lessons are
     discovery and the table retires with the world (recommended);
     conditions as rules data; port the full model. Mark chose "Lexicon
     lessons are discovery (Recommended)".

     Links: area Ecology · draws on 468 · summary Native lessons from eating a part whole are discovery; legacy discovery's condition table retires with the world.
765. **The native revision grows to tracts and systems.** 2026-10-10.
     Options: legacy's declared tracts and 568's folding of systems at the
     boundary, built in the lineages family (recommended); recipe variants
     only. Mark chose "Grow to tracts and systems (Recommended)".

     Links: area Ecology · draws on 568, 684 · summary The native revision grows to tracts and systems at the boundary, built in the lineages family.
766. **Native parts gain cell identity.** 2026-10-10, from the bodies
     lane's second pass (merged at `5f3fe18b`). Question: legacy `Mosaic`
     gives each part a cell graph (ids, adjacency, tracts as connected cell
     sets, an intake port, a per-part stock) that feeding modes read, while
     native keeps counts per function. Options: cell identity in native,
     tracts as cell sets beside the counts and intake ports as part data
     (recommended); counts only; a presentation projection. Mark chose "Cell
     identity in native (Recommended)".

     Links: area Bodies and anatomy · summary Native parts gain cell identity: tracts as cell sets beside the counts, intake ports as part data.
767. **Eponym's needs become native accounts; its wound, checkpoint 10's
     cell loss.** 2026-10-10. Options: hunger and fatigue as native
     accounts (the matter reserve and `Energy`) read by `Need`s, the wound
     built as checkpoint 10's cell loss with Eponym's world move
     (recommended); Eponym-shaped counters in native; counters stay
     Eponym's. Mark chose "Native accounts and wounds (Recommended)".

     Links: area Eponym · draws on 239 · summary Eponym's hunger and fatigue become native accounts read by Needs; its wound is checkpoint 10's cell loss, built with Eponym's world move.
768. **A campaign's session carries its assertions.** 2026-10-10, from the
     record lane (merged at `b9e0f967`). Question: `CampaignWorld` is a field
     of isonetry's replicated snapshot, and a native `Session` needs a
     `Genesis` a campaign lacks. Options: the session carries the asserted
     entries, `CampaignWorld` folds from them and a native `Session` replays
     from them when the sim switches on (recommended); the snapshot holds a
     `Session`; wait for native founding. Mark chose "Session carries
     assertions (Recommended)", 248's "switched on later over the same
     history".

     Links: area The VTT · draws on 248 · summary A campaign's session carries its assertions; CampaignWorld folds from them and a native Session replays from them when the sim switches on.
769. **Authored places, characters, laws and history lines are native nouns
     with authored fills.** 2026-10-10. Options: each its native noun with
     authored attributes filling placeholders as 760 did (a place a `Site`,
     a character a placeless `Entity`, a law an asserted rule record, a
     history line an `Event`) (recommended); an asserted-records map; all
     as notes. Mark chose "Native nouns, authored fill (Recommended)".

     Links: area The sim's schema and saves · draws on 760 · summary Authored places, characters, laws and history lines are native nouns (Site, placeless Entity, rule record, Event) with authored attribute fills.
771. **Eponym's epistemic layer becomes notes plus knowing.** 2026-10-10.
     Options: its simulation record leaves with Eponym's world move and the
     epistemic layer becomes notes held by the observer plus the reach
     field's knowing (84, 117) (recommended); a native noun of its own. Mark
     chose "Notes plus knowing (Recommended)".

     Links: area Eponym · draws on 84, 117 · summary Eponym's epistemic layer becomes notes held by the observer plus the reach field's knowing; its simulation record leaves with its world move.
772. **A new line's name is its key.** 2026-10-10. Options: `lineage:<name>`,
     no schema change (recommended); a generated key and a name field. Mark
     chose "Its name is its key (Recommended)".

     Links: area Method and documents · summary A new line's name is its key, lineage:<name>, with no schema change.
773 to 775. **Effects keys, glyph reading and the experiment.** 2026-10-10,
     from the effects family (merged at `a8570817`). Mark chose "Native
     function keys (Recommended)": embodiment keys on `function:<name>`,
     and legacy tables naming `mesocosm:<process>` are re-authored at the
     world move (773). "At the switch (Recommended)": mesocosm-runtime's
     `GlyphReading` moves onto the native `Journal` with 762's review and
     succession (774). "mesocosm-runtime (Recommended)": `effect_experiment`
     stays in the game's host-neutral runtime under 669 (775).
776. **Eponym's items are inert entities carried by relation.** 2026-10-10,
     from Eponym's world move (merged at `6dac64b3`: legacy Eponym deleted;
     harm, needs, arrival, knowing and social native; `eponym-play` holding
     the game's half). Question: native has no item noun. Options: inert
     entities of item lineages carried by relation, position and worn part
     game-side (recommended, built); a native item noun; game-side. Mark
     chose "Inert entities, carried by relation (Recommended)".

     Links: area Eponym · draws on 239 · summary Eponym's items are inert entities of item lineages carried by relation, with position and worn part game-side.
777. **Eponym keeps its admitted anatomy snapshot.** 2026-10-10. Options:
     keep it beside the native geometry with a game revision moving only on
     severing (recommended, built); read everything from the native
     document. Mark chose "Keep the snapshot (Recommended)".

     Links: area Eponym · summary Eponym keeps its admitted anatomy snapshot beside native geometry, with a game revision moving only on severing.
778. **Crafts and deed weights are keys in the rules pack.** 2026-10-10.
     Question: Eponym's social vocabulary, built as closed native enums.
     Options: closed native enums (recommended, built); keys in the rules
     pack. Mark chose "Keys in the rules pack". So the enums become keys a
     rules pack declares, and other games can declare their own.

     Links: area Eponym · summary Crafts and deed weights are keys a rules pack declares, not closed native enums; other games can declare their own.
779. **Native `Take` admits a sophont for driving.** 2026-10-10. Question:
     who is played in Eponym, built with control game-side. Options: control
     stays game-side (recommended, built); native `Take` (directing's) also
     admits a sapient critter for driving. Mark chose "Native Take admits a
     sophont". So one native path takes up a life in both games: directing
     for a critter that is not sapient, driving for a sophont (671).

     Links: area Directing and play · draws on 671 · summary Native Take also admits a sapient critter for driving: one native path takes up a life in both games.
780 to 786. **Eponym's shape kept; the switch's follow-ups.** 2026-10-10,
     from the two world moves (Mesocosm's merged at `d3b90c6c`, the switch,
     legacy Mesocosm 40,645 lines to 2,016). Mark chose "Keep as built
     (Recommended)": deeds as events of deed processes, agreements in
     `State::agreements`, the game half in `eponym-play` (780). "Scripts
     allocate tract offers (Recommended)": the authored expression door
     proposes cells for a declared-tract offer through `Allocation::commit`
     (781). "Native dev commands (Recommended)": `ForceBirth` and `Kill` as
     dev commands labelled assisted (782). "Patch placement and a map
     layout (Recommended)": members placed by `Command::Patch` in the lifted
     site, the founding drawing a map layout (783). "Nudge gains an act key
     (Recommended)": an optional act key on `Nudge`, `PlaceMatter` naming a
     site handle (784). "One commit closes it (Recommended)": one review
     commit closes the boundary, offer 0 the leave answer (785). "Saved,
     plus Runtime::resume (Recommended)": the bare `Saved` trace, and
     `Runtime::resume` so a replay can be watched (786).
     *Reading, not ruled:* legacy Mesocosm's last 2,016 lines (the grown
     and relief terrain, bricks, near, the snapshot codec) serve only the
     parry-ground probe's fixture, so the fixture moves into the probe,
     keeping its receipt's figures, and legacy Mesocosm is deleted.
787. **Authored cells reach the world by `Command::Express`.** 2026-10-10,
     from the switch's follow-ups (merged at `3eafae18`). Options: a native
     `Command::Express { entity, part, tracts }` placing cells through
     `mosaic::propose` (recommended); a reading beside the offer; folded
     into `Revise` as counts. Mark chose "Command::Express (Recommended)".

     Links: area Bodies and anatomy · draws on 766 · summary Authored cells reach the world by native Command::Express placing cells through mosaic::propose.
790. **One after-pass plan holds every deferred done-condition.** 2026-10-10,
     from the plan review. Question: the after-pass list was scattered (732,
     761, the directing plan, the CLAUDE.md files). Options: one dated
     after-pass plan, each item with its owning plan (recommended); each plan
     keeps its own; the families plan. Mark chose "One after-pass plan
     (Recommended)".

     Links: area Method and documents · summary One dated after-pass plan holds every deferred done-condition, each item with its owning plan.
791. **Families certify against invariants, not legacy parity; amends 667.**
     2026-10-10. Question: 667's parity draws against legacy readings have
     no control now legacy is deleted. Options: each plan's invariants
     (conservation, replay, its controls) (recommended); a pinned pre-push
     worktree; parity only where sim-certified. Mark chose "Invariants, no
     legacy parity (Recommended)".

     Links: area Method and documents · summary Families certify against each plan's invariants (conservation, replay, controls), not legacy parity.
793 to 804. **The plan review's rulings.** 2026-10-10, from
     `2026-10-10_plan_review.md`, put in three rounds. Mark chose:
     - 793, "Accept it (Recommended)": the review's classification stands;
       the DONE and SUPERSEDED docs archive to `archive_docs/2026-10-10/`,
       residue extracted first, and the LIVE plans are rewritten to the
       native state.
     - 794, "Through the contract (Recommended)": Eponym's E3 and the VTT's
       V2 talk to the sim through their contracts, as Mesocosm does (154,
       197); `eponym-play`'s direct commands move behind it.
     - 795, "Contract maps to native (Recommended)": the contract keeps its
       assertion vocabulary for the table and translates to native
       `isocosm::asserted`, native gaining what it lacks.
     - 796, "Mesocosm onto the tenant (Recommended)": one body rasteriser,
       isometer-render's `LiveBody` retiring once Mesocosm moves.
     - 797, "Code follows the ruling (Recommended)": boundary turns go in
       descending metabolic complexity, defined natively; `lineage/boundary`
       stops ordering by members.
     - 798, "Now, push mode": V2 and E3 open now under 732's compile gate,
       not after M3 certifies; this amends 231's gate.
     - 799, "Two paths (Recommended)": the battlemap is a lifted
       isometer-space site with the sim on, `MapTerrain` with it off.
     - 800, "Directing owns M3, interim M4 (Recommended)": the overlay plan
       keeps the profile, the contract and the full M4.
     - 801, all four folds: protocol H2 and the overmap plan fold into V2;
       functional loops and memory into Eponym's E3; the trait catalogue
       into glyph expression; views founding retires into L10's second
       camera and dev tools into the isocosm bench (W4).
     - 802, "Sim plan owns them (Recommended)": the trophic grammar's TG3 to
       TG7, PE4's impossible-world and anti-affix conditions and the soil
       cycle's M1 to M4 become sim-plan phases; TG7's "lexicon" is renamed,
       native already having a lexicon.
     - 803, "Dramatis, as 34 has it (Recommended)": Eponym's identity moves
       to dramatis when its work resumes, staying in `eponym-play` until then.
     - 804, "Keep it for body binding (Recommended)": `TactileWorld` stays as
       body binding's Mesocosm adoption target, moving to `refresh_queries`
       when that builds.

806. **A lineage's metabolic complexity counts its collective repertoire.**
     2026-10-10, Q3's native turn-order fork under 797. Question: "Turn order
     needs one choice that ruling 797 leaves open. Functions and working
     systems belong to individual bodies, and members of a lineage can
     differ. How should we measure the whole lineage's metabolic
     complexity?" Options: distinct functions and working systems across
     all living members (recommended); the most complex living member,
     without combining capabilities split across bodies. Mark chose
     "Count distinct functions and working systems across all living members (Recommended); varied members can contribute different capabilities."
     So native boundary turns use descending counts of distinct catalogue
     functions expressed and carried systems realized across living members,
     each name counted once. Duplicating a member does not raise this count.
     *Reading, not ruled:* native living-part and system-realization predicates
     supply the count; equal counts use the stable lineage key, and member
     count remains an extinction filter.

     Links: area Ecology · draws on 797 · summary A lineage's metabolic complexity counts distinct functions and systems across all living members; native boundary turns use descending counts.
807. **Event handles are an adapter reading of native event keys.**
     2026-10-10, Q2's telling-identity fork under 794. Question: "The contracts
     lane found another identity gap: the contract names an event with a
     numeric handle, while native knowing stores its text key. The saved
     work ignored the contract's event handle. How should tellings cross
     that boundary?" Options: derive a stable handle in the adapter, share
     the mapping with VTT, and validate agreement with the native event key
     (recommended); add numeric event handles to native now; leave tellings
     explicitly unavailable until native event handles exist. Mark chose
     "Derive a stable handle from the native event key in the adapter (Recommended), share the mapping with VTT, and validate that both fields name the same event."
     So the two contracts share one mapping; a telling's handle and native
     event key must agree before the native knowing command is admitted.
     Native event keys retain their authority. *Reading, not ruled:* reverse
     lookup refuses ambiguous matches rather than choosing the first.

     Links: area The VTT · summary Event handles are an adapter reading of native event keys, one mapping shared with the VTT, validated to agree before a telling is admitted.

808. **A deed's agreement link is typed on its native event.** 2026-10-10,
     Q2's agreement/causal-event fork under 780. Question: "Native agreement
     actions currently fail because they put `agreement:0` in the event’s
     cause, which must name an existing event. Ruling 780 keeps deeds as
     events and agreements in native state but leaves their link format
     open. Where should a deed's agreement reference live?" Options:
     "Typed optional agreement ID on the event (Recommended); keeps the deed
     and its agreement link together, with causal-event validation intact.";
     "Separate native map from event key to agreement ID; keeps agreement
     links outside the event schema." Mark chose "Typed optional agreement ID on the event (Recommended); keeps the deed and its agreement link together, with causal-event validation intact."
     So native deed events carry an optional agreement id, and their causal
     event field retains its existing validation. Agreements remain in
     `State::agreements`; this link does not create another agreement owner.
     *Reading, not ruled:* the optional field defaults absent and is omitted
     from serialization when absent, preserving unassociated event bytes;
     the native social owner attaches it after an accepted deed and reads
     it when deriving deeds. Q2 verifies association and save/replay.

     Links: area Eponym · draws on 780 · summary A deed's agreement link is a typed optional agreement id on its native event; agreements stay in State::agreements.

809. **Native characters retain optional table metadata.** 2026-10-10,
     Q2's character-assertion fork under 795. Question: "The VTT character
     contract supplies a map cell and an owner, but native Character stores
     neither, so accepting it currently loses both. Ruling 795 says native
     gains missing assertion vocabulary. Where should these authored fields
     be retained?" Options: "Optional table metadata on native Character
     (Recommended); save and replicate it with the assertion, without
     granting control or moving a body."; "Campaign-owned metadata beside
     native assertions; save and replicate a separate table record.";
     "Refuse character assertions until Q9 implements their table metadata."
     Mark chose "Optional table metadata on native Character (Recommended); save and replicate it with the assertion, without granting control or moving a body."
     So native Character carries the authored map cell and owner in optional
     table metadata. The assertion, saved state and replication retain it;
     accepting these fields grants no control and moves no body. Control
     remains governed by 152 and 153. Q2 implements and verifies retention.
     *Reading, not ruled:* absence preserves the existing non-table character
     path; the contract's cell and participant handle retain their types.

     Links: area The VTT · draws on 152, 153 · summary Native Character carries the authored map cell and owner as optional table metadata, granting no control and moving no body.

810. **Healing restores a lost cell's recorded allocation.** 2026-10-10,
     Q8's healing-allocation fork under 469 and 496. Question: "When healing
     restores a wounded cell, what function should it recover? The saved
     loss keeps the cell's ID but forgets its former function; rulings 469
     and 496 require healing without settling this allocation." Options:
     "Restore its previous function or free-pool status (Recommended),
     recording that allocation alongside the loss; preserves authored
     allocation and inherited variation."; "Restore the lineage recipe's
     allocation, including inherited variation; authored changes can
     disappear through healing."; "Restore it as free, leaving later
     allocation to the body or player." Mark chose "Restore its previous function or free-pool status (Recommended), recording that allocation alongside the loss; preserves authored allocation and inherited variation."
     So a loss records the cell's previous function or free-pool status,
     and healing restores that allocation rather than recomputing the
     lineage recipe. Authored allocation and inherited variation survive.
     Healing still pays each cell's mass and precedes tissue/new parts (496,
     712); tombstone revival remains 719's native operation.
     *Reading, not ruled:* loss metadata follows its cells through severing
     and save/replay. Q8 verifies allocation, accounting and restoration;
     these consequences do not by themselves certify checkpoint 10.

     Links: area Bodies and anatomy · draws on 469, 496 · summary A loss records the cell's previous function or free-pool status, and healing restores that allocation rather than recomputing the lineage recipe.

811. **Unknown legacy losses remain while recorded losses can heal.**
     2026-10-10, Q8's legacy-loss fork after 810. Question: "Older body saves
     retain lost cell IDs but omit their former function or free-pool
     allocation, so ruling 810 cannot reconstruct those losses. How should
     healing handle them?" Options: "Load them and heal only losses with
     recorded allocations, leaving unknown losses until explicitly repaired
     (Recommended)."; "Reject damaged body saves that lack loss-allocation
     records."; "Load them, but require an authored allocation repair before
     any healing of the affected body." Mark chose "Load them and heal only losses with recorded allocations, leaving unknown losses until explicitly repaired (Recommended)."
     So legacy losses remain admissible; unknown allocation is retained as
     unknown, and healing selects losses with recorded allocations without
     stalling all healing of the affected body. Unknown losses wait for an
     explicit repair; no recipe or free-pool allocation is invented for them.
     *Reading, not ruled:* normal genesis, checkpoint, outcome and witness
     validation still applies. Q8 must distinguish old fields decoding from
     an archive actually loading and replaying; serialization/migration
     compatibility remains a verification gate, not supplied by this ruling.

     Links: area Bodies and anatomy · draws on 810 · summary Legacy losses of unknown allocation stay admissible and wait for explicit repair; healing selects only losses with recorded allocations.

812. **Healing after adapter transfer restores free-pool cells.**
     2026-10-11, Q8's transfer-allocation fork after 810. Question: "When an
     injured whole part is taken as an adapter, the adapter clears its donor
     functions. Later healing could restore a lost cell's recorded donor
     function under ruling 810. What should healing restore after that
     transfer?" Options: restore free-pool cells after the adapter clears
     allocations, retaining lost IDs (recommended); restore the recorded
     donor functions; leave the transferred losses unknown until repaired.
     Mark chose "Restore lost cells as free-pool cells after the adapter clears their function allocations (Recommended); retain the lost cell IDs."
     So clearing an adapter's function allocations also clears the function
     to be restored for its recorded losses, while retaining their cell IDs.
     Healing does not recreate donor functions. Q8 verifies normal and crowd
     transfer, paid healing and save/replay. Ruling 811 still governs unknown
     legacy allocations; these are not silently reconstructed.

     Links: area Bodies and anatomy · draws on 810 · summary Healing after an adapter transfer restores lost cells as free-pool cells, retaining the lost cell IDs.

813. **An empty choice can be reconsidered at a later pass start.**
     2026-10-11, Q3's receipt-backed timing fork under 683. Question: "Ruling
     683 allows one chosen process per tick, but leaves an empty choice
     unspecified. Q3 observed all 37 consumers choose nothing before food
     arrived at tick 111, then skip feeding despite 74 eligible producers.
     Should an empty choice commit the whole tick?" Options: reconsider at
     later pass starts until a process is selected, then keep it for the tick
     (recommended); keep the first empty choice for the whole tick. Mark chose
     "Reconsider at later pass starts until one process is selected, then keep it for the tick (Recommended)."
     So a no-process result can be reconsidered once at a later pass start;
     an actual selected process remains committed for that tick. This amends
     683's unspecified empty result without permitting a second process.
     Pass-start readings under 454 and an explicit nudge's named identity
     remain intact. Q3 qualifies the timing, one-process and replay controls
     on a fresh envelope before any reproduction pilot or rate promotion.

     Links: area Directing and play · amends 683 · summary An empty choice can be reconsidered once at a later pass start; a selected process stays committed for that tick.

814. **The VTT declares its note kinds in the world's founding rules.**
     2026-10-11, Q9's native contract-admission source audit. Question: "The
     VTT authors `reveal`, `narration`, and `history` notes, but generated
     native worlds only admit their declared `sim:*` kinds. How should VTT
     notes enter a native session?" Options: declare the table's note kinds
     in the VTT world's founding rules (recommended), preserving names and
     native validation; map them to existing native kinds while retaining
     the original kind separately; allow native assertions to introduce
     undeclared note kinds. Mark chose "Declare the table’s note kinds in the VTT world’s founding rules (Recommended); preserve their names and native validation."
     So the VTT's founding rules declare its authored note vocabulary before
     native replay; contract lowering retains each kind's name, and native
     admission continues to refuse kinds absent from those rules. This does
     not introduce a second note authority or change generic native admission.
     *Reading, not ruled:* the VTT generator/profile must assemble that
     vocabulary with the rest of its native Genesis. This ruling does not
     choose that product attachment or complete V2's hosted integration.

     Links: area The VTT · summary The VTT declares its note kinds (reveal, narration, history) in the world's founding rules; native admission still refuses undeclared kinds.

815. **Active windows contribute to one player's attention set.**
     2026-10-11, Q14's shared-session host. Question: "Ruling 204 gives each
     player one attention set; ruling 443 lets two windows show different
     modes and close-up regions. When both windows use the same participant,
     how should their attention combine?" Mark chose "Merge each active window’s
     contribution into the player’s attention set (Recommended)."
     So each active window contributes to the participant's single set.
     Independent modes and close-up regions remain local to their windows
     under 443; native attention uses their union under 204. Closing or
     replacing a window withdraws only its own contribution. Native play,
     profile care and seat access retain their existing owners.
     *Reading, not ruled:* window identities, admission commands and
     transactional host publication must implement this union and retain
     native replay; the answer does not certify the proposed host.

     Links: area Host and network · draws on 443, 204 · summary Each active window contributes to one player's single attention set, taken as their union; closing a window withdraws only its contribution.

816. **Eponym develops admitted bodies from a selected native recipe.**
     2026-10-11, Q10's original-test diagnosis. Question: "Eponym’s 25 failing
     tests reach native hunger with reserve left outside the body and no
     store cells allocated. Existing rulings make physiology native, but
     don’t specify its initial construction. What should supply an arriving
     or admitted body’s functions and part stocks?" Mark chose "Native recipe
     development: a selected native recipe supplies physiology and geometry,
     requiring a new recipe choice."
     So native development supplies both physiology and geometry from the
     selected recipe. The native admission must retain conserved stocks and
     an admitted anatomy snapshot under 777. A geometry-only embodiment and
     loose reserve do not supply functioning store cells.
     *Reading, not ruled:* the recipe choice remains open. Logged admission,
     stock placement, capacity refusal, conservation and replay must be
     verified before the original twenty-five failures can be closed.

     Links: area Eponym · summary Eponym develops admitted bodies from a selected native recipe supplying physiology and geometry, retaining conserved stocks.

817. **Migration arrives after the traversed route's travel time.**
     2026-10-11, Q3's feeding diagnosis. Question: "Q3 traced all 117 missed
     feeds to producers moving away earlier in the tick. Routes have authored
     travel times, but movement currently arrives instantly and can cross
     several routes in one tick. How should a member’s migration use that
     travel time?" Mark chose "Journey duration (Recommended): depart and
     arrive after the traversed route’s travel time, then allow another route."
     So departure begins a journey and arrival occurs after that route's
     authored duration. The member cannot begin its next route before arrival.
     *Reading, not ruled:* native transit state, arrival scheduling and saved
     replay must implement this duration. This answer neither certifies the
     rejected reproduction candidate nor changes food, reproduction rates,
     the committed choice for a tick, or the eighty-world A1 gate.

     Links: area Processes and rules · summary Migration departs and arrives after the traversed route's authored travel time; no next route before arrival.

818. **Eponym initially develops the compact six-part native recipe.**
     2026-10-11, Q10's native body admission.
     Question: "Which native recipe should initially develop Eponym’s driven sophont? The proposed compact recipe has six parts with intake, store, sense, contract, support and grip; its nominal capacities are 860mg tissue and 105mg reserve. The existing browser recipe with an explicit store tract has up to 33 parts and nominal capacities of 1324mg tissue and 387mg reserve. These are source calculations awaiting native verification; both replace the old authored seven-part starting body."
     Mark chose "Compact six-part Eponym recipe (Recommended)."
     So Native development supplies the selected intake, store, sense, contract,
     support and grip parts under 816. The former authored seven-part
     starting body is replaced by that native development.
     *Reading, not ruled:* The nominal 860mg tissue and 105mg reserve
     capacities are source calculations awaiting native measurement. The
     answer selects a recipe, not a tested admission or E3 acceptance.

     Links: area Eponym · draws on 816 · summary Eponym initially develops the compact six-part native recipe (intake, store, sense, contract, support, grip), replacing the authored seven-part body.

819. **Fresh arrival tissue fits the developed native body.**
     2026-10-11, Q10's native body admission.
     Question: "How should fresh arrival tissue be chosen when the old 48,000–56,000mg request exceeds either recipe’s capacity? This applies to newly issued starting matter; existing supplied stocks remain conserved and overflow must refuse atomically."
     Mark chose "Issue fresh tissue within the developed body’s native capacity (Recommended); retain reserve 100 and energy 100 where admitted."
     So Newly issued tissue must fit the selected development. Reserve 100 and
     energy 100 remain only where admitted. Supplied existing stocks stay
     conserved and overflow refuses atomically under 777.
     *Reading, not ruled:* Capacity measurement, stock placement and
     logged admission require atomic refusal, conservation and replay
     tests; this ruling does not alter existing supplied matter.

     Links: area Eponym · draws on 816, 777 · summary Fresh arrival tissue is issued within the developed native body's capacity; reserve 100 and energy 100 where admitted.

820. **Upkeep continues during travel and carries its produced soil.**
     2026-10-11, Q3's native journey semantics.
     Question: "During a journey, how should native upkeep run? Current upkeep consumes tissue and deposits soil at the member’s site, but a departed member is absent from both sites until arrival. Age would continue under each option."
     Mark chose "Continue upkeep and carry its produced soil until a later site deposit (Recommended)."
     So Age and upkeep continue while the member is absent from both sites.
     Soil produced by upkeep stays with the traveller until a later native
     site deposit; the last residence tag cannot receive it during transit.
     *Reading, not ruled:* The carried-soil ledger and deposit timing
     require native conservation and replay checks. This ruling changes
     neither process rates nor the committed choice within a tick.

     Links: area Processes and rules · draws on 817 · summary Age and upkeep continue during travel; soil produced by upkeep is carried until a later native site deposit.

821. **A child born during travel shares the remaining journey.**
     2026-10-11, Q3's native journey semantics.
     Question: "What should happen if a travelling member can pay for a birth before arrival? Birth currently copies the parent’s site, so transit needs an explicit placement rule."
     Mark chose "Allow birth; the child shares the parent’s remaining journey (Recommended)."
     So Native birth may occur before arrival when its ordinary payment and
     feasibility conditions hold. The child shares the parent's remaining
     journey, rather than appearing at either site immediately.
     *Reading, not ruled:* Native placement, stock payment and replay
     must preserve the shared remaining duration under 817. This answer
     does not certify the rejected reproduction candidate or A1.

     Links: area Processes and rules · draws on 817 · summary A birth during travel is allowed under ordinary conditions and the child shares the parent's remaining journey.

822. **Travel events have a typed journey location without site reach.**
     2026-10-11, Q3's native journey semantics.
     Question: "A travelling member has left both sites, but native age and birth notes require a site and currently spread their reach there. Where should an event during travel be located?"
     Mark chose "Give it a typed journey location, with no site reach until a later native telling (Recommended)."
     So An event during transit names the journey as its native location.
     It gains no site reach from a departure or destination tag; a later
     native telling establishes reach through the existing telling rules.
     *Reading, not ruled:* The typed wire layout, handle translation
     and backward reader need source review and native tests; the answer
     does not select an invented site or certify saved histories.

     Links: area Processes and rules · draws on 817 · summary An event during travel names the journey as its native location and gains no site reach until a later native telling.

823. **A corpse completes the remaining journey before site availability.**
     2026-10-11, Q3's native journey semantics.
     Question: "If a member dies during a journey, what should happen to its body and conserved matter?"
     Mark chose "Finish the remaining journey carrying the corpse, then make it available at the destination (Recommended)."
     So Death preserves the body, its conserved matter and remaining journey.
     The corpse completes that journey and becomes available at the
     destination, without an immediate deposit at either site.
     *Reading, not ruled:* Dead travellers remain absent from both site
     populations until arrival. Representation, decay and conservation
     require native verification without changing authored process rates.

     Links: area Processes and rules · draws on 817 · summary A member dying during a journey leaves a corpse that completes the journey and becomes available at the destination.

825. **The precise-contact fixture has a named native founding profile.**
     2026-10-11, Eponym's native fixture save owner.
     Question: "Eponym’s precise-contact tests need two native fixture recipes that keep their original small root shapes while adding storage. World saves currently rebuild founding from a seed and version tag, then authenticate its hash. How should those fixture recipes be saved?"
     Mark chose "Named native founding profile, selected by a saved version tag (Recommended)"
     So a saved version tag selects the declared native founding
     profile, which supplies the two fixture recipes and keeps their
     original small root shapes while adding native storage. Restore
     regenerates that founding and authenticates its hash before
     replay. Native recipes and recorded selections retain authority
     over physiology, geometry and conserved allocation.
     *Reading, not ruled:* the source proposal uses generator5 and
     GameSave8, preserves current4 and the earlier finite grammars,
     and retains Canon's actual minimum7. Native capacity, contact,
     complete replay and compiled refusal controls remain gates.

     Links: area Eponym · summary A saved version tag selects a declared native founding profile supplying the two precise-contact fixture recipes; restore regenerates and authenticates it.

827. **Spread capacity is a lineage mechanic with an explicit modifier or manipulator.**
     2026-10-11, fungal and microbial substrate capacity.
     Question: "Fungal and microbial spread can overflow into other places, but its native capacity is still unspecified. What determines how much biomass a substrate can hold?"
     Options: (1) "An explicit capacity in milligrams for each spread and substrate, configured in the world (Recommended)."; (2) "A configured ratio of the substrate’s soil or host matter."; (3) "A capacity calculated from the spread’s lineage traits."
     Mark chose "3 + 1; mechanic and modifier/manipulator"
     So the lineage-derived capacity is the mechanic, and explicit capacity
     values can modify or manipulate it at the substrate. Both belong to
     native rules and state, rather than a game-side capacity authority.
     *Reading, not ruled:* use bounded authored expressions over native
     lineage trait inputs, with declared modifier layers and stacking under
     427/429. This answer supplies neither coefficients nor one fixed
     additive, multiplicative or replacement default. World definitions
     supply those choices; missing declarations do not invent a capacity.

     Links: area Ecology · draws on 498 · summary Spread capacity is a lineage-trait mechanic, with explicit capacity values acting as modifier or manipulator at the substrate, all in native rules.

828. **Overflow is split equally before each route transmits its share.**
     2026-10-11, native spread route allocation.
     Question: "When spread exceeds capacity and several routes are available, how should the excess be allocated? Routes already have transmission rates, and untransmitted matter must stay at the source."
     Options: (1) "Divide the excess equally among eligible routes, then apply each route’s transmission rate (Recommended)."; (2) "Divide it among routes using their transmission rates as weights."; (3) "Offer the whole excess to every route, then scale the combined transfers to avoid duplicating matter."
     Mark chose "Divide the excess equally among eligible routes, then apply each route’s transmission rate (Recommended)."
     So capacity excess is divided among eligible native routes before
     their authored transmission rates apply. Untransmitted matter stays
     at the source, and departures/arrivals use the existing conserved
     native transfer queue and authored travel duration.
     *Reading, not ruled:* indivisible integer remainders remain at the
     source. Whether a zero-transmission route is eligible is asked as
     a separate fork; neither interpretation is silently selected.

     Links: area Ecology · draws on 827 · summary Overflow above capacity is divided equally among eligible routes before each route's transmission rate applies; untransmitted matter stays at source.

829. **Port feeding is declared by its process and founded with its ports.**
     2026-10-11, native intake migration scope.
     Question: "Native intake ports can restrict which living or dead matter a body eats. How broadly should that restriction apply during this migration?"
     Options: (1) "Apply it to processes that declare port feeding; migrate those processes and their founding ports together, preserving unmarked saved grammars (Recommended)."; (2) "Require a valid intake port for every feeding process and migrate all founding definitions."
     Mark chose "Apply it to processes that declare port feeding; migrate those processes and their founding ports together, preserving unmarked saved grammars (Recommended)."
     So an explicitly marked process uses the addressed living native
     intake port and its declared source selector for prospective feeding
     and actual admission. Its founding recipe/port and process migrate
     together. Unmarked recorded feeding retains its former grammar.
     *Reading, not ruled:* native descendants retain their recipe's port
     declaration; per-body port differences retain distinct admissible
     prey pools without changing the existing draw, weights or quotas.

     Links: area Processes and rules · summary Port feeding applies to processes that declare it, migrated with their founding ports; unmarked saved grammars are preserved.

830. **Native declares a terrain operation and the game resolves its geometry.**
     2026-10-11, chosen-process carve/deposit ownership.
     Question: "For a chosen process that carves or deposits terrain, who should calculate the exact cells to change? Native must still admit and record the result."
     Options: (1) "Native declares the operation; the game resolves the geometry and returns the edit to the same native Session (Recommended)."; (2) "Native calculates and applies the complete terrain edit itself."
     Mark chose "Native declares the operation; the game resolves the geometry and returns the edit to the same native Session (Recommended)."
     So the chosen native process supplies the operation, the game
     resolves its actual geometry, and the same native Session admits
     and records the resulting edit. Accepted costs, matter and history
     remain native; the game does not gain a second simulation authority.
     *Reading, not ruled:* preflight disposable candidates until every
     action-bound resolution is supplied, then publish one accepted
     command. Geometry refusal must not leave process costs committed;
     replay uses recorded resolutions and the native admission rules.

     Links: area Processes and rules · summary Native declares a terrain operation, the game resolves its geometry, and the same native Session admits and records the edit.

831. **Zero-transmission routes do not participate in the overflow split.**
     2026-10-12, the routing follow-up to828.
     Question: "Overflow is divided equally before each route’s transmission rate is applied. Should a route with zero transmission receive a share of that split?"
     Options: (1) "Exclude zero-transmission routes from the split (Recommended)"; (2) "Include them; their untransmitted share stays at the source".
     Mark chose "Exclude zero-transmission routes from the split (Recommended)"
     So automatic overflow divides excess only among eligible routes with
     positive transmission, then applies each selected route's authored rate.
     If none are eligible, the excess remains at the source. Existing native
     route validation, travel duration and conserved remainders still apply.
     This settles the explicit eligibility fork left by828.

     Links: area Ecology · amends 828 · summary Zero-transmission routes are excluded from the overflow split; if none are eligible the excess stays at the source.

832. **New ordinary feeding processes target what their declared ports admit.**
     2026-10-12, the ordinary founding follow-up to829.
     Question: "New ordinary founding can give a jaw a port that admits living consumers, while its feeding process targets producers only. How should we align that pair? Existing unmarked saves retain their behavior."
     Options: (1) "Make the process target what its port admits (Recommended)"; (2) "Keep the producer-only diet and make the port admit producers"; (3) "Keep the current pair and refuse incompatible meals".
     Mark chose "Make the process target what its port admits (Recommended)"
     So new ordinary founding derives its marked process's target and source
     selector from its actual declared intake port. A consumer-admitting jaw
     no longer retains a producer-only shortcut. Unmarked saved grammar stays
     unchanged, as829 requires; this supplies no new rate or meal quantity.
     *Reading, not ruled:* prospective eligibility and actual admission read
     offered matter provenance, including mixed-source parts, rather than
     require the current holder's lineage to equal the admitted source.

     Links: area Processes and rules · draws on 829 · summary New ordinary feeding processes derive their target and source selector from the actual declared intake port; unmarked saves unchanged.

833. **Mesocosm saves actual spatial state beside native body and terrain revisions.**
     2026-10-12, the actual geometry provider follow-up to830.
     Question: "Mesocosm has advisory display positions, but lacks authoritative in-site position, eye and terrain-target inputs. Where should the actual geometry inputs live?"
     Options: (1) "Saved Mesocosm spatial state linked to native body and terrain revisions (Recommended)"; (2) "Native state stores position and target; Mesocosm resolves the shape"; (3) "Supply explicit geometry separately for each operation".
     Mark chose "Saved Mesocosm spatial state linked to native body and terrain revisions (Recommended)"
     So Mesocosm owns and saves actual spatial inputs, bound to the corresponding
     native body and terrain revisions. Its provider supplies those actual
     inputs to830's operation door; native still admits costs, matter, placement
     and the returned edit in the same Session. Advisory display positions do
     not become the physical source of those inputs.
     *Reading, not ruled:* publish versioned game bytes and native acceptance
     together; missing or stale required physical inputs refuse rather than
     invent a position, eye, target or shape. This answer does not choose a
     movement solver, spawn coordinate, process or travel-arrival pose.

     Links: area Mesocosm · summary Mesocosm owns and saves actual spatial state, bound to native body and terrain revisions; missing or stale inputs refuse.

834. **A sim-on VTT character develops its own native body and keeps its ID.**
     2026-10-12, native body admission at the table.
     Question: "An authored VTT character currently has table metadata but no developed native body. With simulation on, should that same character develop a body, or bind to an existing body? Placement and control still require separate native admission."
     Options: (1) "Develop that character from a selected native recipe, preserving its ID (Recommended)"; (2) "Save an explicit binding to an existing native body".
     Mark chose "Develop that character from a selected native recipe, preserving its ID (Recommended)"
     So the selected recipe develops the asserted character itself; its native
     identity remains the same. Native placement and control are separately
     admitted;809's table owner and cell metadata still grant neither.
     *Reading, not ruled:* use existing `Arrival.character` plus `Develop`,
     with explicit authored lineage/recipe, stocks, seed and site. Publish
     versioned table game data and native acceptance together. The answer
     supplies no default recipe, stock quantities, ruleset calibration or
     participant/party control policy. The next ruling is835.

     Links: area The sim's schema and saves · draws on 809 · summary A sim-on VTT character develops its own native body from a selected recipe and keeps its ID; placement and control are admitted separately.

835. **Named native material recipes specify new item stocks and quantities.**
     2026-10-12, Eponym's material founding after776/801.
     Question: "For newly founded material items, what should specify their actual native stocks? Current food gets 40mg, dressing and scrap get none, while their declared weights are 250/100/900mg. Existing saves would retain their recorded holdings."
     Options: (1) "Named native material recipes specify source stocks and quantities (Recommended)"; (2) "Use the existing item-kind weights as fresh native issuance"; (3) "Require an explicit stock ledger for each newly founded item".
     Mark chose "Named native material recipes specify source stocks and quantities (Recommended)"
     So new material items use named native recipes for their source holdings
     and quantities. Existing saves retain their recorded matter; nominal
     item-kind weights do not silently replace their ledgers. This answer
     names the construction owner but supplies no recipe quantities or costs.

     Links: area Eponym · draws on 776, 801 · summary Named native material recipes specify source stocks and quantities for newly founded material items; existing saves keep recorded holdings.

836. **Each declared material has a distinct native stock in new worlds.**
     2026-10-12, material identity before fabrication.
     Question: "Soil and rock currently both credit `world:soil` when carved, which loses their material distinction. How should newly founded worlds retain material identity for later crafting?"
     Options: (1) "Give each declared material a distinct native stock (Recommended)"; (2) "Keep one matter pool with separately recorded material composition"; (3) "Let authored crafting conversions use the shared pool without retaining the distinction".
     Mark chose "Give each declared material a distinct native stock (Recommended)"
     So newly founded worlds retain separate native stocks for declared
     materials; carve, deposit and fabrication preserve that identity through
     their admitted ledgers. This does not reconstruct the composition of
     an older pooled `world:soil` balance or rewrite its saved founding.

     Links: area Ecology · summary Newly founded worlds give each declared material a distinct native stock, preserved through carve, deposit and fabrication.

837. **A native project reserves inputs and spends from that ledger as work progresses.**
     2026-10-12, interrupted material work under E3's T1/T3.
     Question: "When material work is interrupted, how should its inputs and unfinished progress remain conserved?"
     Options: (1) "Reserve inputs in a native project ledger, then spend from it as work progresses (Recommended)"; (2) "Spend directly from accessible stocks as each work step is admitted"; (3) "Spend all inputs only when the output completes".
     Mark chose "Reserve inputs in a native project ledger, then spend from it as work progresses (Recommended)"
     So admitted reservation transfers the inputs into a native project
     ledger. Work spends from that reservation; interruption preserves
     remaining inputs and unfinished progress. Matter is counted once across
     source, project, partial construction and accepted output.
     *Reading, not ruled:* authored native recipes/processes declare costs,
     progress units and output; this answer supplies no duration, yield,
     refund/cancellation policy or scripted autonomous decision.

     Links: area Processes and rules · summary A native project ledger reserves inputs and work spends from it; interruption preserves remaining inputs and progress, matter counted once.

838. **Any eligible natively authorized subject may resume a project.**
     2026-10-12, resumed work admission.
     Question: "Who should be allowed to resume an interrupted project? Craft requirements and native control still apply."
     Options: (1) "Any eligible subject with native authorization to the project (Recommended)"; (2) "Only the original maker"; (3) "Only an explicitly assigned worker".
     Mark chose "Any eligible subject with native authorization to the project (Recommended)"
     So resumption requires current craft eligibility, native project
     authorization and the ordinary control boundary. It is not restricted
     to the original maker or one assigned worker. Existing social agreement
     acceptance alone does not complete a material operation or grant an
     unrelated project's access.

     Links: area Processes and rules · draws on 837 · summary Any subject with current craft eligibility, native project authorization and ordinary control may resume a project.

839. **Native owners of reserved inputs grant project access together.**
     2026-10-12, native project grant/revoke authority under838.
     Question: "Ruling 838 allows eligible, authorized subjects to resume a project, but native agreements do not yet define who grants project access. Who should grant or revoke permission to work on a project?"
     Options: (1) "An explicitly named project authority, recorded when inputs are reserved (Recommended)"; (2) "The subject who creates the project"; (3) "The native owners of the reserved inputs, acting together".
     Mark chose "The native owners of the reserved inputs, acting together"
     So native input ownership supplies joint project access authority. A
     maker or separately authored authority does not acquire that right by
     naming itself. Native admission must establish the owners and their
     joint decision; table metadata and agreement acceptance supply none.
     *Reading, not ruled:* ownership of unowned site stocks and changes to
     the joint owner roster need their actual native semantics before grants.

     Links: area Processes and rules · draws on 838 · summary The native owners of a project's reserved inputs, acting together, grant and revoke project access.

840. **Interrupted work stays at creation by default, with carrying chosen.**
     2026-10-12, physical project placement under837/838.
     Question: "The native project ledger retains reserved matter and progress, but Eponym needs a physical location for interrupted work. Where should that work remain?"
     Options: (1) "At its creation position until explicitly moved (Recommended)"; (2) "Carried by the current worker, subject to carrying capacity"; (3) "In a site-level ledger, with an explicit position chosen when the output completes".
     Mark answered "1 default, 2 at your choice"
     So fixed creation placement is the default, and carrying is an explicit
     option with capacity checks. Interruption retains the same native
     project, stocks and progress; changing worker does not move it itself.
     *Reading, not ruled:* the player chooses carrying through an admitted
     action; this is not permission for an adapter to relocate work silently.

     Links: area Processes and rules · draws on 837, 838 · summary Interrupted work stays at its creation position by default; carrying by the current worker is an explicit choice with capacity checks.

841. **Authored step profiles tie physical movement to native processes.**
     2026-10-12, Mesocosm's actual in-site movement source.
     Question: "Mesocosm now saves actual positions, but its current physics world is advisory and cannot drive a body. How should a chosen native movement process produce an actual step?"
     Options: (1) "Authored step profiles tied to native movement processes, with geometry resolved by the game (Recommended)"; (2) "A separately selected game physics solver and movement profile".
     Mark chose "Authored step profiles tied to native movement processes, with geometry resolved by the game (Recommended)"
     So the world's authored step profile binds a native movement process;
     its chosen act drives game-resolved geometry through native admission.
     Advisory layout is not an actual step. Step values, cost, timing and
     collision parameters remain authored rather than supplied by this answer.

     Links: area Mesocosm · draws on 833 · summary Authored step profiles bind native movement processes; a chosen act drives game-resolved geometry through native admission.

842. **Destinations supply explicit authored arrival placements and windows.**
     2026-10-12, actual geometry after a native journey.
     Question: "Native journeys clear the old site placement on arrival. What should supply a body's actual destination position and spatial window?"
     Options: (1) "Explicit authored arrival placements and windows for destinations (Recommended)"; (2) "A selected game solver derives placement from authored route-entry geometry".
     Mark chose "Explicit authored arrival placements and windows for destinations (Recommended)"
     So actual destination placement and its spatial window come from
     explicit authored data and are checked against current native geometry.
     An old-site pose or a display position is not a fallback; an absent or
     refused destination binding remains unavailable. The answer supplies
     no coordinates or window dimensions.

     Links: area Mesocosm · draws on 833 · summary Destination placement and spatial window come from explicit authored arrival data checked against native geometry; no old-pose fallback.

843. **Ordinary reservation requires explicit native ownership.**
     2026-10-12, ownerless material under839.
     Question: "Native code distinguishes ownership from carrying and tenancy, but currently records no stock owners. For stocks without an asserted owner, including new items and site matter, what should reservation require?"
     Options: (1) "Require explicit native ownership before reservation (Recommended)"; (2) "Allow ownerless projects under a separately declared access policy"; (3) "Claim ownership through the admitted reservation".
     Mark answered "hm, 1, but a separate pathway would be something like a help action"
     So ordinary reservation needs an explicit native ownership assertion;
     holding, tenancy, maker provenance and the act of reservation supply
     none. A help action is a separate possible pathway, whose admission,
     inputs and permissions this answer does not define. It is not an
     inferred bypass of the ordinary ownership requirement.

     Links: area Processes and rules · draws on 839 · summary Ordinary reservation requires an explicit native ownership assertion; holding, tenancy and maker provenance supply none; a help action is a separate pathway.

844. **Polities may author any of the three collective approval strategies.**
     2026-10-12, a polity's share of joint project permission under839.
     Question: "If a polity owns some reserved inputs, how should it give its share of the joint project permission? Its native constitution names governance but does not yet evaluate collective decisions."
     Options: (1) "A native collective decision under its authored governance (Recommended)"; (2) "Unanimous approval from all current native members"; (3) "Approval by explicitly declared native representatives".
     Mark answered "all 3 are legitimate strategies"
     So native polity authorization supports an explicitly authored choice
     among governance decisions, current-member unanimity and declared
     representatives. A constitution's name does not invent its evaluation
     rules, and no strategy is silently selected for existing polities.
     *Reading, not ruled:* the applicable strategy and its required inputs
     are authored native state, checked when permission is given or revoked.

     Links: area Processes and rules · draws on 839 · summary Polities authorize via an explicitly authored choice among governance decisions, member unanimity or declared representatives; none is selected silently.

845. **Stock ownership names exact quantity lots selected for reservation.**
     2026-10-12, native property quantity boundary under839/843.
     Question: "Native ownership also needs a stock boundary: incoming matter can refill the same account after some has been reserved. Should an ownership assertion cover a specific quantity, or the account's whole changing balance?"
     Options: (1) "Exact quantity lots, selected explicitly for reservation (Recommended)"; (2) "The whole source account, including later incoming matter".
     Mark chose "Exact quantity lots, selected explicitly for reservation (Recommended)"
     So the native assertion identifies a typed stock source, account,
     exact quantity and owners. Reservation explicitly selects those lots;
     later incoming matter does not acquire an earlier quantity's owners.
     These ownership records do not issue or duplicate conserved matter.

     Links: area Processes and rules · draws on 839, 843 · summary Stock ownership names exact quantity lots selected explicitly for reservation; later incoming matter does not acquire earlier owners.

846. **Owned-stock debits select their native quantity lots explicitly.**
     2026-10-12, ordinary stock disposition after845.
     Question: "Owned stock can already leave an account through eating, transfers, transformations or terrain work. How should those actions identify which quantity lots they use? Incoming matter must not inherit an earlier lot’s owners."
     Options: (1) "Require explicit lot selection for debits of owned stock (Recommended)"; (2) "Use explicitly authored native allocation rules"; (3) "Leave ownership unresolved after an unselected debit until explicitly repaired".
     Mark chose "Require explicit lot selection for debits of owned stock (Recommended)"
     So native owned-stock debit admission identifies the selected quantity
     lots at the actual source. A later credit cannot replace their identity
     or acquire their owners. Unsupported or absent selection refuses;
     automatic allocation and unresolved tagging are not selected.
     The answer does not supply a property-transfer or consumption policy
     beyond identifying the input lots.

     Links: area Processes and rules · draws on 845 · summary Debits of owned stock must explicitly select their native quantity lots; absent selection refuses and no automatic allocation applies.

847. **Resident corpses remain visible through native awareness and close-up.**
     2026-10-12, native body visibility after death and travel.
     Question: "Native bodies and conserved matter survive death, including corpses that finish a journey. Current native views hide every dead body, so a close-up cannot show those corpses. How should visibility work?"
     Options: (1) "Show resident corpses within the same native awareness and close-up (Recommended)"; (2) "Keep them outside ordinary views, with a separate native remains view".
     Mark chose "Show resident corpses within the same native awareness and close-up (Recommended)"
     So death alone does not hide a resident native corpse from the same
     participant awareness and admitted close-up. In-transit corpses stay
     absent from both sites until native arrival. Showing remains grants no
     control or living action and does not add them to living crowd counts.
     No decay duration or replacement physical pose is supplied.

     Links: area Presentation · summary Resident corpses stay visible in the same native awareness and close-up; in-transit corpses stay absent; showing grants no control.

848. **Authored birth profiles use the parent's current physical pose.**
     2026-10-12, Mesocosm physical placement after a native birth.
     Question: "Native newborns inherit their parent’s site or remaining journey, but Mesocosm has no actual position for them. What should supply a newborn’s physical placement? Birth and its conserved matter would proceed even when placement is unavailable."
     Options: (1) "Authored birth placement profiles using the parent’s current pose (Recommended)"; (2) "Explicit placement supplied for each newborn".
     Mark chose "Authored birth placement profiles using the parent’s current pose (Recommended)"
     So authored birth profiles resolve actual native newborn placement from
     its parent's current admitted pose. Geometry, offset, window, clearance
     and placement policy remain declared inputs. Missing/refused placement
     does not delay or undo native birth or its conserved matter, and grants
     no control. A travelling parent has no site pose to guess; the native
     child still shares its remaining journey under821.

     Links: area Mesocosm · draws on 833, 821 · summary Authored birth profiles resolve a newborn's physical placement from its parent's current pose; refused placement never undoes the birth.

849. **A debit may select an exact portion of a named native lot.**
     2026-10-12, partial native stock selection under846.
     Question: "Eating, transfers and terrain work can debit less than a declared lot contains. How should explicit selection handle a partial quantity?"
     Options: (1) "Select an exact quantity from a named lot, leaving the rest at its source (Recommended)"; (2) "Require whole lots whose quantities exactly match the debit".
     Mark chose "Select an exact quantity from a named lot, leaving the rest at its source (Recommended)"
     So the caller identifies the lot and exact selected quantity. Native
     admission checks that quantity against its actual current source and
     remaining claim; the unselected portion stays there. No implicit split
     across other lots, FIFO or proportional allocation is introduced.

     Links: area Processes and rules · draws on 846 · summary A debit selects an exact quantity from a named lot, the unselected portion staying at source; no FIFO or proportional split.

850. **Each operation authors retained output claims or consumed-claim closure.**
     2026-10-12, property disposition after selected native conversion.
     Question: "Consumption and transformation can merge stocks, produce several outputs or turn material into terrain. What should happen to the selected inputs’ ownership claims?"
     Options: (1) "An authored disposition per operation declares retained output claims or closes consumed claims (Recommended)"; (2) "Always close claims on consumed or transformed inputs"; (3) "Preserve the input owners’ claims over all resulting matter".
     Mark chose "An authored disposition per operation declares retained output claims or closes consumed claims (Recommended)"
     So the operation explicitly declares which output quantities retain
     the selected owners' claims, or closes the consumed input claims.
     Native matter remains conserved independently of claim closure. Mixed
     owners, byproducts and changed stock accounts need explicit allocation;
     digestion does not infer ownership of another body's tissue. Missing
     disposition or unavailable output vocabulary refuses that owned path.
     The next unused question is851.

     Links: area Processes and rules · draws on 846 · summary Each operation declares which outputs retain the selected owners' claims or closes consumed claims; matter stays conserved regardless.

851. **Every bound table player holds downtime unless they release their own hold.**
     2026-10-12, native downtime consent after246.
     Question: "Ruling 246 requires each player’s individual consent before downtime advances native time. Which players should have to consent? Saved native table bindings include players who may currently be disconnected."
     Options: (1) "All bound native table players, including disconnected players (Recommended)"; (2) "Only bound players currently connected".
     Mark chose "1, but allow players to release their hold so others can proceed without constantly pinging them"
     So the roster includes the saved bound native players, including those
     disconnected. Each player's own release can waive their hold; connection
     state, the DM and another player cannot release it for them. The release
     duration is answered in855. Membership and native control stay separate.

     Links: area The VTT · draws on 246 · summary Every saved bound table player, disconnected included, holds downtime unless they release their own hold.

852. **A downtime proposal binds its original native tick and player roster.**
     2026-10-12, waiting native consent validity.
     Question: "While a downtime proposal awaits consent, native time or the table’s player bindings may change. What should happen to that proposal?"
     Options: (1) "Bind it to its original tick and player roster; preserve consents, but require a new proposal if either changes (Recommended)"; (2) "Apply its duration at final consent, using the current tick and player roster".
     Mark chose "Bind it to its original tick and player roster; preserve consents, but require a new proposal if either changes (Recommended)"
     So a changed tick or roster makes that proposal unavailable for advance.
     Its accepted declaration and consents remain history; a fresh proposal
     must bind the new cut. Restoring an earlier roster must not silently
     revive an invalidated proposal. Endpoint/connectivity changes alone do
     not remove the same native player from the roster.

     Links: area The VTT · draws on 851 · summary A downtime proposal binds its original tick and player roster; a change makes it unavailable and requires a new proposal.

853. **Players opt their played native bodies into or out of a party.**
     2026-10-12, native party membership for the VTT overmap.
     Question: "The VTT overmap needs a party’s actual native members. A party is a faction under ruling 8, and membership does not grant control. Who should define joining and leaving?"
     Options: (1) "DM authors explicit native membership (Recommended)"; (2) "Each player opts their played body in or out".
     Mark chose "Each player opts their played body in or out"
     So actual player intents author membership of their played native body.
     Party membership stays a native faction relation/record under8 and9;
     it grants neither Take nor collective control. Token IDs, owner labels
     and a table roster cannot stand in for native body/player identities.
     Withdrawal after a body change is answered in857.

     Links: area The VTT · draws on 8, 9 · summary Each player opts their played native body into or out of a party; membership grants no control.

854. **The overmap combines party members' disclosed knowledge in a shared view.**
     2026-10-12, native knowing and party map presentation.
     Question: "Native knowing belongs to individual bodies. Whose knowledge should supply a player’s overmap?"
     Options: (1) "Their currently played body’s native knowledge (Recommended)"; (2) "A shared view combining party members’ disclosed knowledge".
     Mark chose "A shared view combining party members’ disclosed knowledge"
     So shared-map facts derive from disclosed native knowing of actual
     party members. Sharing the view does not make each body natively know
     every contributed fact, expose undisclosed knowledge or grant control.
     Retention after leaving is answered in858.

     Links: area The VTT · draws on 853 · summary The overmap is a shared view combining party members' disclosed native knowledge; sharing grants no control or undisclosed knowledge.

855. **A self-released downtime hold covers future proposals until explicitly retaken.**
     2026-10-12, duration of851's release.
     Question: "When a player releases their downtime hold, should that cover future proposals too, or only the proposal currently waiting?"
     Options: (1) "Future proposals until they explicitly take the hold back (Recommended)"; (2) "Only the current proposal".
     Mark chose "Future proposals until they explicitly take the hold back (Recommended)"
     So release applies to the current and future proposals until that
     player explicitly takes their hold back. Accepted self-release/retake
     history survives saves. Disconnecting does not substitute for release;
     retaking does not erase earlier declarations, consents or advances.

     Links: area The VTT · draws on 851 · summary A self-released downtime hold covers current and future proposals until the player explicitly retakes it.

856. **The DM explicitly chooses which ready downtime proposal advances.**
     2026-10-12, simultaneous eligible spans after a release.
     Question: "Releasing a hold can make several downtime proposals ready at the same native tick, with different durations. Which proposal should advance time?"
     Options: (1) "The DM explicitly chooses a ready proposal (Recommended)"; (2) "The earliest accepted proposal advances automatically".
     Mark chose "The DM explicitly chooses a ready proposal (Recommended)"
     So a DM advance names the original eligible proposal. Declarations,
     yeses and hold changes can make it ready but do not silently select a
     span. Admission checks852 and an actual yes or active self-release for
     each roster player, then publishes the chosen native advance atomically.

     Links: area The VTT · draws on 851, 852 · summary The DM explicitly chooses which ready downtime proposal advances; the earliest does not advance automatically.

857. **Players may withdraw their own earlier party memberships after changing bodies.**
     2026-10-12, membership withdrawal under853.
     Question: "Party membership can outlast a player changing to another body. May that player withdraw a body they previously opted in, even when they no longer play it? New joins and disclosures would still require their currently played body."
     Options: (1) "Yes, let players withdraw their own earlier memberships (Recommended)"; (2) "Allow withdrawal only for the currently played body".
     Mark chose "Yes, let players withdraw their own earlier memberships (Recommended)"
     So the actual joining player's own prior membership authorizes its
     withdrawal after a body change. New joins and disclosures still name
     the currently played body. No automatic heir, replacement body, other
     player's removal or control grant is inferred.

     Links: area The VTT · draws on 853 · summary Players may withdraw their own earlier party memberships after changing bodies; new joins and disclosures name the current body.

858. **Disclosed map facts remain recorded after a member leaves.**
     2026-10-12, retention of854's shared knowledge.
     Question: "When a member leaves the party, should facts they already disclosed remain on the shared map?"
     Options: (1) "Keep disclosed facts in the party’s recorded map (Recommended)"; (2) "Remove that member’s contributions from the current shared view".
     Mark chose "Keep disclosed facts in the party’s recorded map (Recommended)"
     So accepted disclosures remain available in that party's recorded map
     after the contributor leaves. This retains the shared record, not an
     ongoing window into the former member's undisclosed native knowledge.
     The next unused question is859.

     Links: area The VTT · draws on 854 · summary Disclosed map facts remain in the party's recorded map after the contributing member leaves.

859. **Each player approves their currently played body's joint departure.**
     2026-10-12, collective travel after853's membership.
     Question: "Party membership grants no control over its bodies. For a joint overmap departure, who should authorize each body’s movement?"
     Options: (1) "Each player approves their currently played body; the DM applies the approved departures together (Recommended)"; (2) "Players depart individually through their native movement actions"; (3) "Players explicitly delegate party travel authority, with a way to revoke it".
     Mark chose "Each player approves their currently played body; the DM applies the approved departures together (Recommended)"
     So a joint departure carries each actual player's explicit approval
     naming their currently played native body. The DM publishes the approved
     departures together through native admission. Membership, old membership,
     owner metadata and a shared map grant no travel control. Existing native
     process choice, costs and journey admission still apply; this answer
     supplies no pace effect or shortcut to advancing native time.
     Pace question860 remains pending; the next unused question is861.

     Links: area The VTT · draws on 853 · summary Each player approves their currently played body's departure and the DM applies approved departures together through native admission.

860. **Authored native pace profiles supply duration and stock costs.**
     2026-10-12, actual pace effects after817's journey duration.
     Question: "Native journeys use the route’s authored travel time, while the VTT also requests a pace. What should give pace its actual effects?"
     Options: (1) "Authored native pace profiles specify duration and stock costs (Recommended)"; (2) "Keep the route’s native duration; pace is descriptive until its mechanics are authored".
     Mark chose "Authored native pace profiles specify duration and stock costs (Recommended)"
     So requested pace resolves to an explicitly authored native profile
     whose duration and stock costs undergo native process/journey admission.
     The legacy percentage supplies neither a default cost nor an override
     of native stock conservation, chosen processes or time consent. Values
     and available profiles remain author inputs; no recipe is guessed.
     Critter approval question861 remains pending; next unused question862.

     Links: area The VTT · draws on 817 · summary Requested pace resolves to an explicitly authored native pace profile giving duration and stock costs; no default from the legacy percentage.

861. **Directed critters receive a travel goal, bounded by their impressions and motives.**
     2026-10-12, deliberative bodies in859's joint journey.
     Question: "A played deliberative critter still chooses its own process each tick. What should an approved party journey do for that body?"
     Options: (1) "Set its native travel goal; it departs when its own choice allows, so the party can split (Recommended)"; (2) "Keep it out of joint departures; the player directs its journey separately".
     Mark answered: "oh, for mesocosm? i mean, i guess a critter under the direction of a player would travel according to their impressions of the player  as best it could in macro level travel. like faction aligned animals in rimworld. although if it has its own motives, then... hell, your choices would be limited to what it trusts you to do or what you can motivate it to do. which is sort of charming and odd, depending on what it needs and is too. 1. in eponym, not even an issue: faction aligned animals are gonna follow the faction and their in game directors, who are much more suited to directly handle creatures and such"
     So the accepted request supplies a native travel goal. Native impressions,
     trust, motivation and the critter's own motives bound its available
     following and scheduled choice; approval does not force a departure.
     The party can split. Eponym's faction-aligned animals retain their
     faction and in-world directors, without a new implicit player handle.
     This applies wherever a player actually directs a native deliberative
     animal, including the VTT case that prompted the question. No trust
     threshold, motivation strength or new control grant is invented.
     The next unused question is862.

     Links: area Directing and play · draws on 683, 859 · summary A player-directed deliberative critter gets a native travel goal from an approved journey; it departs when its own choice allows, so the party can split. (amended by 862)

862. **The existing native bond suffices for directed travel in this migration.**
     2026-10-12, reframing861's impression/motivation admission.
     Question: "Native nudges already weigh a critter’s needs and bond, but don’t yet use its impressions of the director. Where should the trust and motivation rule apply?"
     Options: (1) "Authored native rules limit which travel goals the director can offer; the critter still chooses when to follow (Recommended)"; (2) "Authored native rules weight the critter’s choice after any travel goal is offered"; (3) "Use both an offer limit and a choice weight, authored separately".
     Mark answered: "the bond would suffice, no?"
     The answer to the reframing is yes: the existing native BOND permits a
     genuine nudge, and native bond sway already contributes alongside the
     critter's own needs and motives. Use that existing bond and scheduled
     chooser for861's travel goal. No additional impression filter, trust
     threshold or motivation layer is required for this migration. Actual
     body control and bond admission remain separate, and a goal is not a
     promise that the animal departs. The next unused question is863.

     Links: area Directing and play · amends 861 · summary The existing native bond and scheduled chooser suffice for directed travel; no extra impression filter, trust threshold or motivation layer.

863. **Future produced lots are selected by explicit authored operation/output references.**
     2026-10-12, subsequent autonomous use of849's named quantity lots.
     Question: "An autonomous process can produce a new owned lot whose key did not exist at founding. How should a later process select that lot?"
     Options: (1) "An explicit authored reference names the producing native operation and output, with an exact quantity (Recommended)"; (2) "Require a separate selection of the current named lot before the later process can use it".
     Mark chose "An explicit authored reference names the producing native operation and output, with an exact quantity (Recommended)"
     So a later selector may explicitly name the producing native operation
     and its exact output, plus the selected quantity. Resolution must identify
     that actual accepted output claim, current holder and remaining amount.
     It grants no FIFO, broad descendant search, changing-account ownership
     or automatic choice of a different produced lot. Ambiguous, missing,
     spent or otherwise mismatched references refuse.

     Links: area Processes and rules · draws on 849 · summary A later process selects a future produced lot by an explicit authored reference naming the producing operation, output and exact quantity.

864. **Collect the outstanding questions and deliver them together.**
     2026-10-12, direct correction to the orchestrator's question cadence.
     Mark said: "also, when I said sequentially, i didn't mean for you to split the questions up. i actually meant for you to put them all together. so i could blitz through them and you could just focus on working. i'm saying, don't trickle them."
     So inspect independent work first, collect its known necessary forks
     and present them together for one answer pass. Do not manufacture a
     succession of isolated questions or pause ordinary migration work after
     each answer. This current instruction supersedes an interpretation of
     sequential as one question at a time. Newly discovered forks belong in
     the next collected round while independent authorized work continues.
     The next unused question is865.

     Links: area Method and documents · summary Collect outstanding questions from independent work and deliver them together in one pass; do not trickle isolated questions.

865. **A compatible dormant adventure pack activates by explicit DM acceptance.**
     2026-10-13, dormant-pack activation in V2.
     Question: "Here are the remaining VTT choices together. When a dormant adventure pack becomes compatible with the world, how should it activate?"
     Options: (1) "The DM previews it and explicitly accepts it (Recommended)"; (2) "It activates automatically at the next native clock boundary".
     Mark chose "The DM previews it and explicitly accepts it (Recommended)"
     So compatibility makes the saved dormant pack available for preview;
     the DM explicitly accepts its proposed native changes. Compatibility
     alone does not activate it or publish changes at a clock boundary.
     Native validation, preview freshness and atomic admission still apply.

     Links: area The VTT · draws on 190 · summary A compatible dormant adventure pack activates only when the DM previews and explicitly accepts it.

866. **A native act records participation in every goal it actually served.**
     2026-10-13, goal participation beside immediate event causality.
     Question: "When one native act serves several goals, which goal threads should record that participation? Its immediate causal event remains separate."
     Options: (1) "Every goal it actually served (Recommended)"; (2) "One primary goal, selected by an explicit authored rule".
     Mark chose "Every goal it actually served (Recommended)"
     So an accepted act can participate in multiple genuine goal threads.
     Each records its actual participation; no primary-goal choice discards
     the other served goals. Its immediate causal event remains separately
     validated. D31's derived arcs acquire no independent native Arc type.

     Links: area Directing and play · draws on 103 · summary A native act records participation in every goal it actually served; no primary goal discards the others.

867. **Authored polity attributes and member-derived readings retain their disagreement.**
     2026-10-13, presentation of disagreeing native polity facts.
     Question: "When authored polity attributes disagree with readings derived from its actual members, how should the VTT present them?"
     Options: (1) "Keep both and show the disagreement until an explicit native change (Recommended)"; (2) "Give the member-derived reading precedence"; (3) "Give the authored attributes precedence".
     Mark chose "Keep both and show the disagreement until an explicit native change (Recommended)"
     So the VTT retains both readings with their actual sources and shows
     their disagreement. A derived reading does not overwrite authored
     attributes, and authored attributes do not hide the member reading.
     Reconciliation requires an explicit admitted native change.

     Links: area The sim's schema and saves · draws on 760 · summary Authored polity attributes and member-derived readings are both kept and their disagreement shown until an explicit native change.

868. **Faction claims and biomes use configurable map layers.**
     2026-10-13, overlapping meanings in known native map regions.
     Question: "When a known map region has both a faction claim and a biome, how should its colouring work?"
     Options: (1) "Configurable layers showing both meanings (Recommended)"; (2) "Faction colouring takes precedence"; (3) "Biome colouring takes precedence".
     Mark chose "Configurable layers showing both meanings (Recommended)"
     So independently configurable layers retain both native meanings.
     The presentation settings grant neither knowledge nor faction control;
     admitted facts and the player's disclosed map still bound the view.

     Links: area Presentation · summary Faction claims and biomes use independently configurable map layers showing both meanings; settings grant no knowledge.

869. **Undisclosed edges of known map regions always clip sharply.**
     2026-10-13, known-region boundary presentation.
     Question: "How should undisclosed edges of known map regions appear?"
     Options: (1) "Let each view choose hard clipping or fading (Recommended)"; (2) "Always clip them sharply"; (3) "Always fade them".
     Mark chose "Always clip them sharply"
     So the native overmap sharply clips undisclosed boundaries in every
     view. A fade option is not introduced; colour layers or thumbnail
     backdrops cannot expose undisclosed map geometry beyond that boundary.

     Links: area Presentation · summary Undisclosed edges of known map regions always clip sharply in every view; no fade option.

870. **Baked map thumbnails are optional per world or view.**
     2026-10-13, native overmap backdrop presentation.
     Question: "Should the native overmap use baked map thumbnails as a backdrop?"
     Options: (1) "Optional per world or view (Recommended)"; (2) "Show native terrain without thumbnails"; (3) "Always use thumbnails".
     Mark chose "Optional per world or view (Recommended)"
     So a world or view may explicitly select a baked thumbnail backdrop.
     Native terrain and disclosed facts remain authoritative; thumbnails
     respect869's sharp clipping and do not reveal undisclosed regions.
     All six answers were delivered together under864. No question remains
     pending from this round; the next unused question is871.

     Links: area Presentation · draws on 869 · summary Baked map thumbnails are an optional backdrop per world or view and respect sharp clipping.

871. **Party map disclosures retain named known snapshots and their revisions.**
     2026-10-13, collected migration reply under864.
     Question: "How should terrain, conditions and faction claims enter the party’s recorded map?"
     Options: (1) "Explicitly disclose named terrain windows and condition/claim snapshots the body knows; retain source revisions and require a new disclosure for changes (Recommended)"; (2) "Disclose a complete snapshot of a known site"; (3) "Use separate DM-authored cartographic disclosures".
     Mark: "Explicitly disclose named terrain windows and condition/claim snapshots the body knows; retain source revisions and require a new disclosure for changes (Recommended)"
     Disclose only explicitly named windows and snapshots supported by the body's native knowledge.
     Keep authenticated source revisions; later native changes do not silently refresh the party's record.

     Links: area The VTT · draws on 854 · summary Party map disclosures name explicit terrain windows and condition or claim snapshots the body knows, with source revisions; changes need a new disclosure.

872. **Defensive tissue uses an authored native threshold expression.**
     2026-10-13, collected migration reply under864.
     Question: "Plate-shaped parts support fixation and are not necessarily armour. What determines the bite threshold for explicitly defensive tissue?"
     Options: (1) "An authored native expression over designated defensive tissue and its modifiers (Recommended)"; (2) "A fixed authored threshold for each defensive kind"; (3) "An authored geometry law for designated armour".
     Mark: "An authored native expression over designated defensive tissue and its modifiers (Recommended)"
     Defence is explicitly designated and its threshold is an authored native expression.
     Plate geometry alone establishes no armour; inputs and modifier layers remain declared.

     Links: area Bodies and anatomy · summary Defensive tissue is explicitly designated and its bite threshold is an authored native expression; plate geometry alone establishes no armour.

873. **Defence compares the final pass-granted bite on the protected part.**
     2026-10-13, collected migration reply under864.
     Question: "Several eaters can receive smaller bites than planned. Which amount should be compared with the protected part’s defensive threshold?"
     Options: (1) "The final native pass-granted bite on that protected part (Recommended)"; (2) "The planned bite before sharing matter"; (3) "Each defensive profile declares its comparator and scope".
     Mark: "The final native pass-granted bite on that protected part (Recommended)"
     Compare the actual shared-pass grant on that protected part, rather than the planned request.
     Admission and conserved matter must use that same resolved grant.

     Links: area Bodies and anatomy · draws on 872 · summary Defence compares the final pass-granted bite on the protected part, not the planned bite; admission uses the same grant.

875. **Generated-world offers require one joint actual native witness.**
     2026-10-13, collected migration reply under864.
     Question: "Before a generated world is offered, must energy capture, matter return, reproduction and counterplay work together in one actual native witness?"
     Options: (1) "One joint native witness with an authored finite horizon, followed by ecology admission (Recommended)"; (2) "Separate native witnesses from the same world, then joint ecology admission"; (3) "Verified reachability including guards, targets, bodies and costs, then ecology admission".
     Mark: "One joint native witness with an authored finite horizon, followed by ecology admission (Recommended)"
     Require one joint actual native witness over a supplied finite horizon, followed by ecology admission before offering the generated world.
     Separate demonstrations or structural reachability alone do not satisfy the selected gate; running verification remains deferred826 during migration.

     Links: area World generation · summary A generated world is offered only after one joint actual native witness over a finite horizon, then ecology admission.

876. **Location identity is learned explicitly, with membership and extent separate.**
     2026-10-13, collected migration reply under864.
     Question: "Survival knows sites but has no known Location IDs. How should a body learn the identity of a Location grouping several sites? Undisclosed boundaries still clip sharply."
     Options: (1) "Explicit native observation or telling establishes identity; member sites and extent are disclosed separately (Recommended)"; (2) "Knowing any member site establishes Location identity; extent stays hidden"; (3) "Establish identity only after all member sites are known".
     Mark: "Explicit native observation or telling establishes identity; member sites and extent are disclosed separately (Recommended)"
     Use actual native observation or telling to establish Location identity.
     Knowing a site does not infer its Location; identity does not reveal membership or extent, and unknown boundaries still clip sharply.

     Links: area Directing and play · draws on 869 · summary Location identity is learned by explicit native observation or telling; member sites and extent are disclosed separately.

877. **Companion succession uses an authored native social agreement or standing predicate.**
     2026-10-13, collected migration reply under864.
     Question: "Ruling186 requires a companion’s bond to the body that died. What native fact identifies an eligible bonded companion for Eponym succession?"
     Options: (1) "An explicitly authored native body-to-body relation, with its key and direction specified (Recommended)"; (2) "An explicitly authored native social agreement or standing predicate".
     Mark: "An explicitly authored native social agreement or standing predicate"
     Companion eligibility is an explicitly authored native social agreement or standing predicate over the deceased body and candidate.
     Do not substitute a body-to-body relation or participant-to-critter directing bond; fallback requires establishing absence under the declared predicate.

     Links: area Eponym · draws on 186 · summary Companion succession eligibility is an explicitly authored native social agreement or standing predicate over deceased body and candidate.

878. **Total collapse uses an explicitly authored native global predicate.**
     2026-10-13, collected migration reply under864.
     Question: "Ruling230 governs total world collapse, but native code only defines regional collapse. What should count as total collapse? Complementary regional losses can leave every trophic level present globally."
     Options: (1) "A historically occupied trophic level has no biomass anywhere in the world"; (2) "No living ecological biomass remains anywhere"; (3) "An explicitly authored native global collapse predicate".
     Mark: "An explicitly authored native global collapse predicate"
     Total collapse uses an explicit native global predicate; do not hardcode regional conjunction, missing trophic level or zero biomass.
     Missing declaration does not invent a threshold; the chosen consequence remains230 and owner replacement still needs its actual caller.

     Seven choices are settled;874 remains open. Next unused question879.
     Source migration continues under826; no compiler/tests/fixtures/tuning
     are authorised merely by answering these design questions.

     Links: area Ecology · amends 226 · draws on 230 · summary Total collapse uses an explicitly authored native global predicate; no hardcoded regional conjunction, missing level or zero biomass.

879. **Ecological independence requires a distinct authored mechanical effect; family is recorded.**
     Asked2026-10-13 following874's unresolved interest in both standards.
     Question: "For 874, the two standards differ: changing costs, timing or
     material flows can change mechanics while still using the same native
     operation. Requiring a new operation family is stricter. Which should
     establish that an extra ecological noun adds an independent mechanic?"
     Options: "Require a distinct authored mechanical effect; record its
     operation family too"; "Require a distinct native operation family";
     "Make the standard an authored ecology setting, recording both kinds
     of evidence". Answer: "Require a distinct authored mechanical effect;
     record its operation family too". Source evidence: current processes share
     the native Process/Effect/Amount interpreter; costs, cadence, typed
     flows and provenance can vary within one operation family. Names,
     hashes or changed random draws prove no independent mechanical effect.
     Consequence:879 resolves874. Distinct costs, timing and typed material
     flows can qualify within the same native operation family; record the
     family without imposing an additional family-difference gate. Native
     joint witness875 must precede ecological admission. Missing authored
     comparisons cannot be replaced with names, hashes or random outcomes.
     Next unused880. Tests and tuning remain deferred826.

     Links: area Ecology · draws on 874 · summary Ecological independence requires a distinct authored mechanical effect (costs, timing, typed flows may qualify); the operation family is recorded.

881. **C11's independent chooser covers autonomous territories only.**
     Asked 2026-10-14. Evidence: natively, an un-nudged deliberative
     territory scores each due Choice as needs plus sway plus priority;
     DirectOverflow feeds no need and no nudge sways it, so it takes the
     highest-priority feasible option (route valid, local excess), ties to
     the first process id. Nudges add bond-weighted sway.
     Question: "How far should the crowd's independent chooser go?"
     Options: (1) "Autonomous only (Recommended)": un-nudged deliberative
     and free territories at site grain, exact against crowd over drawn
     worlds with planted faults, nudged directing native-tested and refused
     by the crowd; (2) "Autonomous plus nudges"; (3) "Close at the kernel".
     Mark: "Autonomous only (Recommended)"
     The crowd implements its own chooser for autonomous territories and
     refuses live nudges; nudged directing keeps its native tests.

     Links: area Processes and rules · summary The crowd implements its own chooser for autonomous (un-nudged) territories and refuses live nudges; nudged directing keeps native tests.

883. **The local checkout is `Code/repos/isocosm`.**
     Asked 2026-10-14, after Mark's "feel free to rename the local
     'isometry' directory to 'isocosm'" and the rename with lane worktrees
     re-linked. Question: "What should follow the rename?" Options: (1)
     "Ledger line only (Recommended)": record it in the naming ledger, leave
     `worktrees/isometry-*` and `Code/testing/isometry` as receipts cite
     them; (2) "Rename lanes and testing too"; (3) "Nothing more".
     Mark: "Ledger line only (Recommended)"
     Lane worktree directories and the testing tree keep their names.

     Links: area Naming · draws on 110 · summary The local checkout is Code/repos/isocosm; a naming-ledger line records it and lane worktrees and the testing tree keep their isometry names.

884. **Three uniform product hosts over one embeddable sim host.**
     Asked 2026-10-14 after a runtime check: no head plays today. Default
     Mesocosm launch refuses a generated world without 875's witness, the
     last headed save (2026-09-08) no longer parses, and only the dev bench
     boots. Each product has two to four host paths; ruling 442 keeps
     per-product hosts, and M4, E4 and V4 never name the headed host.
     Question: "Which is the head?" Options: (1) "Per-product hosts
     (Recommended)", the unified host staying an inspection shell; (2)
     "Unified host", grown into the one player client; (3) "Per-product
     now, unify later".
     Mark: "if we do a unified host without having to load the capabilities
     of each of the games, then that's cool. but otherwise, we should create,
     like, three uniform product hosts and an embeddable sim host that
     handles the world founding, live keys, pointer, save import, the stuff
     all need and which can be abstracted into a core."
     *Reading, not ruled:* a Rust binary compiles in every product it can
     run, and `host/` selects products by cargo feature; runtime loading
     would need dylibs or wasm components. So the second branch applies:
     three uniform product hosts over one embeddable core that founds
     worlds, routes live keys and pointer, and imports saves. Today's halves
     are `isomere::host::Assembly` (window hooks, used by the VTT and
     Eponym) and `host/` (resume, save, stream, spatial, access).

     Links: area Host and network · summary Three uniform product hosts over one embeddable sim host handling world founding, live keys, pointer and save import.

886. **The society layer goes behind a feature and is tested in its own lane.**
     Asked 2026-10-14. Evidence: projects, institutions, custody, party,
     goals and polities are about 4.3k lines with no inline tests; projects
     is used in 14 VTT and 21 Eponym files, institutions, custody, party and
     polities by the VTT only, Mesocosm by none.
     Question: "What's its status?" Options: (1) "Freeze until a head uses it
     (Recommended)"; (2) "Test it all now"; (3) "Park behind a feature".
     Mark: "park it behind a feature and then start testing it in another of
     the consigliere's lanes; both eponym and the vtt would need it, right?"
     Answered: yes, both need it. *Reading, not ruled:* the feature is on for
     the VTT and Eponym and off for Mesocosm. Whether saves keep one shape
     across builds is an open fork, since polities, projects and
     institutions are `State` fields.

     Links: area The sim's schema and saves · summary The society layer (projects, institutions, custody, party, goals, polities) is parked behind a feature, on for VTT and Eponym, off for Mesocosm, tested separately.

889. **Generated worlds get spawn and arrival templates from the generator.**
     2026-10-14, the follow-up to 830 and 833, which chose no movement
     solver, spawn coordinate, process or arrival pose.
     Question: "A generated, unauthored world has no spawn pose and nothing driving continuous movement or arrival, so in-site carve/deposit/contact cannot happen there. Where should those inputs come from?"
     Options: (a) "Mesocosm's runtime gets a trusted movement driver that spawns bodies from a native birth/placement rule and steps them through authored step profiles"; (b) "The generator supplies spawn and arrival templates for generated worlds; movement stays template-driven"; (c) "In-site acts stay limited to authored worlds; re-scope M4's unchosen-seed run".
     Mark chose (b), confirming it directly: "the generator supplies spawn and arrival templates for generated worlds and movement stays template-driven".
     A generator lane owns it after A1's generator changes land, one
     writer at a time in `generate.rs`.

     Links: area World generation · draws on 830, 833 · summary The generator supplies spawn and arrival templates for generated worlds; movement stays template-driven.

890. **The generator gives producers a real capture path.**
     2026-10-14, capture inputs for 875's joint witness.
     Question: "The default generated producer synthesizes soil into ordinary body tissue: every generated matter account is reserve:false and nothing credits sim:energy, so only fixtures pass the 875 witness and no generated world can be offered. Who supplies capture?"
     Options: (a) "The generator gives producers a real capture path (a producer reserve account that synthesis credits, or a process crediting sim:energy)"; (b) "Packs/authored worlds supply capture inputs; generated worlds stay unofferable until one does".
     Mark chose (a), confirming it directly: "the generator gives producers a real capture path".
     Tissue is not relabelled as fuel (879). The variant is chosen on
     evidence with A1, which touches the same producer and consumer rates;
     the goal is default generated worlds passing 875's witness.

     Links: area World generation · draws on 875 · summary The generator gives producers a real capture path so default generated worlds pass 875's witness; tissue is not relabelled as fuel. (amended by 929)

891. **The generator writes each world's global collapse predicate.**
     2026-10-14, the follow-up to 878.
     Question: "Nothing authors a global collapse predicate (generate.rs sets global_collapse: None), so no world can collapse. Who authors it?"
     Options: (a) "The generator writes a declared predicate into each world's rules (visible, saved, part of the digest)"; (b) "Only packs/authored worlds carry one; generated worlds never collapse"; (c) "The player or DM authors one at founding".
     Mark chose (a), confirming it directly: "the generator writes a declared collapse predicate into each world's rules".
     The generator lane owns it with 889.

     Links: area World generation · draws on 878 · summary The generator writes a declared global collapse predicate into each world's rules, visible, saved and part of the digest.

892. **The embeddable sim host is `isocosm-host` as a library.**
     Asked 2026-10-14, after 884. Evidence: `host/` is already the package
     `isocosm-host`, a binary whose world, save and attention pieces are
     about 350 lines; Mesocosm's session runtime (`mesocosm-runtime`) and
     key-to-nudge mapping (`mesocosm-genet/src/input.rs`) hold the richer
     generic parts. Question: "Where does the embeddable sim host (884)
     live?" Options: (1) "isocosm-host as a library (Recommended)"; (2)
     "Grow mesocosm-runtime"; (3) "New crate".
     Mark: "isocosm-host as a library (Recommended)"
     `isocosm-host` becomes the core library; its inspection shell becomes
     a bin over it; generic pieces move in from the Mesocosm crates.

     Links: area Host and network · draws on 884 · summary The embeddable sim host is isocosm-host as a library; its inspection shell becomes a bin over it; generic pieces move in from Mesocosm crates.

893. **The society feature gates operations, not state.**
     Asked 2026-10-14, after 886. Question: "How far does the feature
     reach?" Options: (1) "Gate operations, keep state (Recommended)":
     commands, effects and processing behind the feature, the `State`
     fields kept and empty by default; (2) "Gate the state too".
     Mark: "Gate operations, keep state (Recommended)"
     Every build reads one save format; a featureless build carries the
     society fields empty and refuses their operations.

     Links: area The sim's schema and saves · amends 886 · summary The society feature gates operations, not state: State fields stay and are empty by default, so every build reads one save format.

896. **The inspection shell becomes a Graphshell endpoint over the core.**
     Asked 2026-10-14 from the sim host plan's fork 1: a library-and-binary
     `isocosm-host` cannot depend on products that depend on its library.
     Options: (1) "Own package under host/ (Recommended)"; (2) "Retire the
     shell"; (3) "Each product's dev lane". Mark: "own package under host/,
     but is there anything in the stack reminiscent of this, which would
     apply...? or like, what else serves a similar role?" Evidence: mere's
     Graphshell session stack (`chirograph`, `graphshell-endpoint` with local,
     stdio and network carriers, the client port `mere/ports/graphshell`)
     already serves Cleromancy, Knot, mer3ly's repo-graph, Distillery and the
     VTT's `isometry-graphshell`. Put back as "Graphshell endpoint" or "Own
     package under host/". Mark: "oh, graphshell for sure"
     The cross-product desktop shell retires in favour of a Graphshell
     endpoint over the core; product-specific panels go to product-owned
     endpoints, as the VTT's already is.

     Links: area Host and network · draws on 892 · summary The cross-product desktop inspection shell retires in favour of a Graphshell endpoint over the core; product panels go to product-owned endpoints.

898. **All three product hosts sit on isomere's `Assembly`.**
     Asked 2026-10-14, fork 3. Options: (1) "Yes, all on Assembly
     (Recommended)", moving `mesocosm-genet` off its own winit/wgpu loop;
     (2) "Mesocosm keeps its loop". Mark: "Yes, all on Assembly
     (Recommended)", then "tell me more about assembly"; given the account
     (window glue over cambium's host: viewport producers, scenario lane,
     capture, error and profile lines; Mesocosm's `SectionProducer` already
     exists), "naw, that was plenty, thank you".

     Links: area Host and network · draws on 884 · summary All three product hosts sit on isomere's Assembly, moving mesocosm-genet off its own winit and wgpu loop.

900. **Every product panel moves to an endpoint in H2, with no new isometry names.**
     Asked 2026-10-15. Options: (1) "Retire now, rebuild per product
     (Recommended)"; (2) "Move all in H2".
     Mark: "let's not make new isometry names, we're trying to obviate them
     over time, no? but i'm more tempted to do it all in h2"
     Answered: yes, the `isometry-` prefix is a plain technical name to
     retire over time. The desktop shell's product panels move to
     product-owned endpoints within H2.

     Links: area Host and network · draws on 896 · summary Every product panel moves to a product-owned endpoint within H2, and no new isometry- names are made (the prefix retires over time).

901. **The core's inspection endpoint sits in `isocosm-host` behind a feature.**
     Asked 2026-10-15. Options: (1) "isocosm-host, behind a feature
     (Recommended)", the binary becoming the stdio endpoint beside
     `--validate-save`; (2) "Always in the core".
     Mark: "isocosm-host, behind a feature (Recommended)"

     Links: area Host and network · draws on 892, 896 · summary The core's inspection endpoint sits in isocosm-host behind a feature; the binary becomes the stdio endpoint.

903. **Mesocosm's and Eponym's panels become feature-gated modules in existing crates.**
     Asked 2026-10-15. Options: (1) "Modules in existing crates
     (Recommended)": an `endpoint` module in `mesocosm-runtime` and in
     `eponym-play`; (2) "New endpoint crates".
     Mark: "Modules in existing crates (Recommended)"

     Links: area Host and network · draws on 896 · summary Mesocosm's and Eponym's panels become feature-gated endpoint modules in mesocosm-runtime and eponym-play, not new crates.

905. **A generated ecology's consumers graze.**
     Asked 2026-10-15 in plain text, from the A1 lane's draft. Evidence: of
     20 bodied gate worlds, 8 drew a limbed consumer with a jaw; a jaw eats
     only fauna (832), and the only fauna in a generated ecology is the
     consumer's own lineage, so all 8 ate themselves and were extinct by
     tick 100. The 12 that drew an intake mouth fed on producers, and 10
     qualified. Eleven rate variants never beat 1 in 8 on jaw worlds.
     Options: (1) bodied ecology consumers always draw an intake mouth and
     keep their limbs for movement, `Bodies::grazing` becoming the default,
     which changes bodied recipes and their digests for new worlds
     (recommended, by the lane and the reviewer); (2) predator mouths only
     where other fauna exists; (3) the diet left to the drawn mouth.
     Mark: "I accept the recommendations."

     Links: area World generation · draws on 832 · summary Bodied generated-ecology consumers always draw an intake mouth and keep limbs for movement; Bodies::grazing becomes the default.

906. **A lineage's lifespan is individual: a death hazard under an old-age ceiling.**
     Asked 2026-10-15 in plain text, from the A1 lane's draft. Evidence:
     with grazing and one fixed lifespan per lineage the best result is
     16/20, whole generations dying together around ticks 172 to 176; a
     hazard alone scores 13/20, founders outliving the run; hazard under the
     lineage's age as a ceiling scores 20/20 with grazers and 20/20 bare,
     the original-rate controls 0/20. Options: (1) from age 60 each
     individual dies at 2.5% a tick, never past the lineage's drawn lifespan
     of 80 to 120, a mean life near 100, through the engine's existing
     `risk` (recommended, by the lane and the reviewer); (2) a fixed
     lifespan per lineage; (3) the hazard only.
     Mark: "I accept the recommendations."
     *2026-10-16:* the hazard is declared only where the drawn ceiling is
     above maturity, so today only the A1 preset carries it (949).

     Links: area Ecology · summary A lineage's lifespan is individual: from age 60 each dies at 2.5% a tick, never past the lineage's drawn lifespan of 80 to 120. (amended by 935, 949)

907. **The candidate rates become a named world preset, not the generator's defaults.**
     Asked 2026-10-15 in plain text, then by card. Evidence: the candidate
     (birth gate 12, consumers 9; consumer meal 3; consumer birth period 14;
     provision 8; upkeep period 5, consumers 10; lifespans drawn from 80 to
     120) against today's defaults (birth gate 30, upkeep 5, lifespans 20 to
     60, meal 1, birth period 7) passes 19 of 20 draws on Mesocosm's played
     founding, one late extinction near tick 176, where A1's gate asks for
     20; the 60-tick epochs are a verification scope, not the stock
     525,600-tick epoch. Options: (1) promote with 905 and 906, accepting
     19/20 as an amendment to A1's gate (the lane's, if 19/20 is accepted);
     (2) a named world preset (the reviewer's); (3) tune further on fresh
     seeds first (the lane's otherwise). Mark: "A named preset is fine, to
     this question." Default worlds keep today's rates, and A1's gate stands
     unamended. *Reading, not ruled:* the preset's name is a plain
     technical key the lane proposes; a player-facing name would come back
     as a naming question.

     Links: area World generation · summary The candidate rates become a named world preset, not the generator's defaults; default worlds keep today's rates and A1's gate stands unamended. (amended by 935, 961)

910. **A change through any endpoint stales every endpoint's snapshot.**
     Asked 2026-10-15 in plain text. Evidence: the core, Mesocosm, Eponym
     and the VTT serve one shared world, but each endpoint kept its own
     revision and none sends notices, so a DM's act left a player's open
     snapshot current. Most acts recheck live state when they run, but a
     VTT "Propose 1 tick" from an old snapshot would be accepted against a
     newer world. Options: (1) each snapshot records the world's position
     (tick, entry count and state hash) and an intent checks it, so any
     change stales every endpoint (recommended); (2) endpoints send a
     world-changed notice and staleness stays per endpoint; (3) wait for
     the lifted reference-host capabilities (H2 step 4). Mark: "Yes"
     *Reading, not ruled:* "Yes" takes the recommendation, option 1, built
     in the shared disclosure code all four endpoints use.

     Links: area Host and network · summary Each endpoint snapshot records tick, entry count and state hash and intents check it, so a change through any endpoint stales every endpoint.

913. **`isometry-campaign` retires, its contents becoming components.**
     Given 2026-10-15 with the answer to 911, unasked. Mark: "We should
     retire isometry-campaign, which is unpublished, and make its contents
     into components." *Reading, not ruled:* a new objective that gets its
     own assessment; where each part lands (the sim, `isocosm-overlay`,
     `isocosm-vtt`, the stack) comes back as questions before anything
     moves.

     Links: area The VTT · plans campaign-retirement · summary isometry-campaign retires, its contents becoming components; where each part lands comes back as questions.

914. **The crates stay publishable in principle.**
     Given 2026-10-15 after 911 and 913, unasked. Mark: "If the rest of the
     crates are able to be published in principle, i don't mind that."
     *Reading, not ruled:* `isocosm-vtt` may depend on unpublished crates
     so long as nothing makes them unpublishable in principle: licences and
     metadata in order, and dependencies that could themselves be published.
     The components 913 makes are built the same way; publishing any of
     them stays its own decision. This reading goes back to Mark with 913's
     assessment. *Amended by 917.*

     Links: area Licensing and sources · draws on 913 · summary The crates stay publishable in principle: licences, metadata and dependencies in order; publishing any is its own decision. (amended by 917)

915. **913's "components" are owning homes.**
     Asked 2026-10-15 by card. Evidence: `isometry-campaign` is 10,764 lines
     in three parts, the campaign world and content (3,400, which 247 moves
     into the sim at V2), the VTT's game over the sim (5,500, used only by
     VTT crates), and founding, store and proposals (about 1,400). Options:
     (1) "Owning homes (Recommended)": no new crates, the world into the sim
     per 247, the game layer into `isocosm-vtt` beside its endpoint, map
     lowering into `isometry-core`, store and proposals into `isonetry`;
     (2) "Small component crates", each family its own crate; (3) "Wasm
     components", rung 3, against D18. Mark: "Owning homes (Recommended)"

     Links: area The VTT · draws on 913 · plans campaign-retirement · summary isometry-campaign's components are owning homes, no new crates: world into the sim, game layer into isocosm-vtt, map lowering into isometry-core, store into isonetry.

917. **The crates aim to be published.**
     Asked 2026-10-15 by card, to confirm 914's reading. Options: (1) "Yes,
     that reading (Recommended)": publishable in principle, nothing
     published until asked; (2) "Aim to publish": work toward publishing
     them, which also needs mere's `chirograph` and `graphshell-endpoint`
     released at the pinned revision. Mark: "Aim to publish"
     Amends 914's reading: publishing is an objective, not only a property
     to keep. *Reading, not ruled:* releases in mere are mere's to decide,
     and are asked there; the campaign retirement plan's F4 places the work.

     Links: area Licensing and sources · amends 914 · summary The crates aim to be published, not only publishable in principle.

919. **`isocosm-vtt` carries the VTT's licence, MIT OR Apache-2.0.**
     Asked 2026-10-15 by card (the retirement plan's F1). Evidence: the
     game layer moving in (6,400 lines) is MIT OR Apache-2.0, the VTT's
     licence by the root `CLAUDE.md`; `isocosm-vtt` was MPL-2.0 because its
     endpoint came from the retired shell's MPL panels (902). Options: (1)
     "MIT OR Apache-2.0 (Recommended)", its MPL endpoint relicensed by its
     author; (2) "MPL-2.0"; (3) "Per-file licences".
     Mark: "MIT OR Apache-2.0 (Recommended)"

     Links: area Licensing and sources · draws on 902 · summary isocosm-vtt carries the VTT's licence, MIT OR Apache-2.0, its MPL endpoint relicensed by its author.

920. **`isocosm-vtt`'s endpoint sits behind a feature.**
     Asked 2026-10-15 by card (F2). Options: (1) "Endpoint behind a feature
     (Recommended)", an `endpoint` feature as Mesocosm's and Eponym's have
     (903), so the session lane and the host take the game layer alone;
     (2) "Always on". Mark: "Endpoint behind a feature (Recommended)"

     Links: area Host and network · draws on 903 · summary isocosm-vtt's endpoint sits behind an endpoint feature, so the session lane and host take the game layer alone.

921. **`isonetry` holds the GM store and the proposal lifecycle until V2.**
     Asked 2026-10-15 by card (F3). Evidence: 915's summary put them in
     `isonetry`; 247 turns a proposal's branch mode into the sim's branch
     and the log into its intent log at V2. Options: (1) "isonetry until V2
     (Recommended)"; (2) "isocosm-vtt"; (3) "Stay until V2".
     Mark: "i do like isonetry as a name. did we publish it? if so, we can
     remake it into something useful. 1"
     Answered: `isonetry` was never published, and the crates.io name was
     free on 2026-10-15; it is the VTT's live session layer (about 5,000
     lines, DM-authority replication, iroh behind a feature). What to do
     with the name goes back to Mark.

     Links: area The VTT · draws on 915 · plans campaign-retirement · summary isonetry holds the GM store and the proposal lifecycle until V2.

923. **`isonetry` is remade.**
     Asked 2026-10-15 by card. Evidence: never published, the crates.io name
     free; today the VTT's live session layer, about 5,000 lines, holding
     the GM store and proposals until V2 (921), its campaign sync moving to
     Stickleback under K0 to K2. Options: (1) "Keep its job, claim in R4
     (Recommended)"; (2) "Claim the name now"; (3) "Plan a remake".
     Mark: "Plan a remake"

     Links: area Host and network · summary isonetry is remade, planned as a remake rather than kept for its job or just claimed.

924. **`isonetry` becomes the Isocosm family's session layer.**
     Asked 2026-10-15 by card. Evidence: it holds a turn-based
     host-authority replication core (protocol, sessions, iroh behind a
     feature), the VTT's campaign p2p pieces and VTT table bindings; the
     sim-host plan had no multiplayer phase. Options: (1) "Family session
     layer (Recommended)": the network seam for the sim-host core's `World`
     in all three games, one peer hosting the world and others sending
     intents and receiving the accepted log, on the stack's transport,
     Stickleback and Graphshell's network carrier, its VTT-only parts moving
     to `isocosm-vtt`; (2) "VTT adapter on Stickleback"; (3) "Retire it, bank
     the name". Mark: "Family session layer (Recommended)"
     The sim-host plan's H7.

     Links: area Host and network · draws on 923 · summary isonetry becomes the Isocosm family's session layer: host-authority replication for the core World on the stack's transport; VTT-only parts move to isocosm-vtt.

925. **Mesocosm's session lives in the core's `World`.**
     Asked 2026-10-15 by card (H3). Evidence: Mesocosm drives one saved loop
     through two doors, the owned `Runtime` with its own session and the
     borrowed door over the core's `World` that its endpoint uses. Options:
     (1) "The core's World (Recommended)": `Runtime` a thin product layer
     over the `World` and its `mesocosm` section, one door for the head and
     the endpoint, `Interim`'s founding a product step after H1's `found`;
     (2) "Runtime keeps it", the core supplying parts only.
     Mark: "The core's World (Recommended)"

     Links: area Host and network · draws on 892 · summary Mesocosm's session lives in the core's World, Runtime becoming a thin product layer: one door for the head and the endpoint.

926. **The core's driver is typed through a product trait.**
     Asked 2026-10-15 by card (H3). Options: (1) "Typed, via a product trait
     (Recommended)": the product names its envelope type and supplies
     translation, holds and its round; (2) "Opaque bytes".
     Mark: "Typed, via a product trait (Recommended)"

     Links: area Host and network · draws on 925 · summary The core's driver is typed through a product trait naming its envelope type and supplying translation, holds and round.

927. **The core owns the clock and an explicit step; products choose.**
     Asked 2026-10-15 by card (H3). Evidence: Mesocosm's `Clock` is integer
     and rational and defers capped steps; Eponym's 60 Hz motion drops its
     backlog after eight catch-up steps; the VTT has no wall clock. Options:
     (1) "Core clock + explicit step (Recommended)": each product picks
     wall-clock or explicit pacing, the VTT explicit, Eponym's motion
     adopting the clock later; (2) "Product-owned pacing".
     Mark: "Core clock + explicit step (Recommended)"

     Links: area Host and network · draws on 925 · summary The core owns the clock and an explicit step; each product picks wall-clock or explicit pacing.

932. **A generated world collapses when no organism is left (891's predicate).**
     Asked 2026-10-14 by card in the generator lane session; recorded here
     2026-10-15 from that session's verbatim report. The question as put:
     "Which condition should the generator write as each generated world's
     global collapse predicate (ruling 887, recorded in the brief as 891)? It
     is generated rules data in the existing grammar: visible, saved and part
     of the digest." Options: (1) "No organism left (Recommended)": "Collapse
     when the living Reactive/Deliberative/Normative population over all
     lineages is 0. It survives speciation and leaves out the Inert
     world-ground bodies and participant seats. It means the same as 878's
     rejected option (2), but here it is declared data rather than native
     code."; (2) "Below a founding fraction": "Collapse when the living
     population drops below a declared fraction of Founding.population, e.g.
     1/16. That is a threshold, which 878 says we must not invent unless you
     set it."; (3) "Under two founders left": "Collapse when fewer than two
     founding lineages still have living members. Children of a new lineage
     are not counted, so it can fire falsely after speciation."
     Mark: "No organism left (Recommended)", with no note.
     As built on `lane-generator-templates` (`b5d86d14`,
     `shared/isocosm/src/generate.rs`, `extinction`): one input,
     `world:living`, the living population of every lineage's Reactive,
     Deliberative and Normative members (the bare key `living` failed key
     validation), and the condition `AtLeast(Const(0), Read(Kept
     world:living))`, which is 1 exactly while that count is 0, written to
     every generated world's `rules.global_collapse`. Inert world-ground
     bodies and participant seats never count, and no lineage is named.
     *Reading, not ruled:* the question's "ruling 887" means this record's
     891, the predicate the generator declares; 887 is the Mesocosm-loop
     milestone.

     Links: area World generation · draws on 891 · summary A generated world collapses when the living Reactive, Deliberative and Normative population over all lineages is 0, declared as rules data.

935. **The A1 preset's lifespan ceiling is 100 plus a draw of 0 to 40.**
     Asked 2026-10-15 by card, from the A1 lane. Evidence: the preset as
     ruled in 906 and 907 (ceiling 80 to 120 under the hazard) draws bare
     20/20, bodied 19/20, played 18/20 on seeds 0 to 19 against the stock
     controls' 0/20; with the ceiling at 100 plus a draw (candidate 10) bare
     20/20, bodied 20/20, played 19/20, and 16/16 against 14/16 on fresh
     bodied seeds 300 to 315. Candidate 9's constant ceiling of 120 had
     scored 20/20, 20/20 and 19/20. Options: (1) 100 to 140 (recommended by
     the lane); (2) keep 80 to 120 as ruled; (3) a constant 120, amending
     906's wording. Mark: "100 to 140 (Recommended)". It changes the
     preset's numbers only (907): defaults, the hazard and the grazing
     default are untouched.

     Links: area World generation · amends 907 · summary The A1 preset's lifespan ceiling is 100 plus a draw of 0 to 40; defaults, the hazard and the grazing default are untouched.

939. **F1: the shipped pack gets the other ten definitions.** The pack ships
     five process definitions; the native registry carries fifteen, and the
     pack's note claims admission equals native, so three parity tests fail.
     Options: (1) "Ship the other ten (Recommended)"; (2) "Narrow the
     receipt" to the five bound natives; (3) "Retire under 672". Mark: "Ship
     the other ten (Recommended)"

     Links: area Method and documents · summary F1: the shipped pack ships the other ten process definitions so admission equals the native registry's fifteen.

943. **F5: worn items on a severed non-played part drop through the sim's
     handoff.** A volley severing such a part is refused whole with
     `NotPlayed`, because `release_worn` submits the drop under the player's
     participant (`contract.rs:83-86`). Options: (1) "Sim handoff
     (Recommended)": `EponymHandoff::Dropped` (`contract.rs:292`), which
     carries no participant; (2) "No participant"; (3) "Refuse such
     severing". Mark: "Sim handoff (Recommended)"

     Links: area Eponym · summary F5: worn items on a severed non-played part drop through the sim's EponymHandoff::Dropped, which carries no participant.

945. **F7: the producer builds its terrain from the resident window.** The
     native seed-7 world needs 3,062 bricks against modulus's cap of 2,047.
     Options: (1) "Resident window (Recommended)": V1b's capacity-fixed cache
     (`residency.rs`), unmeasured; (2) "Raise Mere's cap"; (3) "Smaller
     fixture world". Mark: "Resident window (Recommended)"

     Links: area Presentation · draws on 425 · summary F7: the producer builds its terrain from the resident window (V1b's capacity-fixed cache), not a raised brick cap.

949. **906's hazard acts only where the ceiling is above maturity: today, the
     A1 preset.** Asked 2026-10-16 by card from 947's review. Evidence: the
     generator declares `ecology:hazard-<i>` only where the drawn lifespan
     exceeds 60; stock lifespans are 20 to 60, so no default world carries
     it (the probe counts none), which fits 907 ("default worlds keep
     today's rates") but not 906's "the generator's". Options: (1)
     "Preset-only, say so (Recommended)", behaviour and digests unchanged;
     (2) "Preset owns it outright", `Rates::A1` declaring it explicitly;
     (3) "Every world gets it", maturity scaled to each ceiling, moving
     stock digests. Mark: "Preset-only, say so (Recommended)". This amends
     906's reading, not its words.

     Links: area Ecology · amends 906 · summary 906's hazard acts only where the drawn ceiling is above maturity, so today only the A1 preset carries it; stock digests unchanged.

953. **Every lane brief carries a receipt budget.** Mark, 2026-10-16: "The a1
     lane wasted basically half of today on 30 gb of receipts nobody will
     use." Asked by card: (1) "Cap receipts in briefs (Recommended)", each
     brief naming the receipts its done-conditions need, capping their
     total (for example 100 MB), and exploratory runs keeping summaries
     only; (2) "Global rule too"; (3) "No rule". Mark: "Cap receipts in
     briefs (Recommended)". Carried into the lane-queue plan's rules (§2).

     Links: area Method and documents · draws on 255 · summary Every lane brief names the receipts its done-conditions need and caps their total (for example 100 MB); exploratory runs keep summaries only.

954. **H4: the core has its own press type and a keymap hook.** Asked
     2026-10-16 by card, from H4's assessment. Evidence: `mesocosm-genet`
     matches winit's `Key` in `input.rs` (224 lines) and builds Mesocosm's
     envelopes there; `isocosm-host` is a window-free library; isomere's
     generic `Keymap<Press, Command>` sits on cambium. Options: (1) "Core's
     own press type (Recommended)": a plain `Press` (named keys and
     characters) and a keymap hook on the driver's `Product` trait,
     `mesocosm-genet` lowering winit's `Key` in one place and Mesocosm's
     keymap moving to `mesocosm-runtime`; (2) "isomere's Keymap", pulling
     cambium into the core; (3) "Envelopes only". Mark: "Core's own press
     type (Recommended)"

     Links: area Host and network · draws on 926 · summary H4: the core has its own plain Press type and a keymap hook on the driver's Product trait; mesocosm-genet lowers winit's Key in one place.

955. **H4: only keys that make an envelope go through the core.** Options:
     (1) "World keys only (Recommended)": E, Space, S, the checkpoint answers
     (Enter, T), the board's commit (R), a sent hint and the dev intents (X,
     F, K, G); camera, inspection, dev clock, the board cursor and the hint
     menu's navigation stay the window's; (2) "Every key via the core".
     Mark: "World keys only (Recommended)". *Reading, not ruled:* the
     continuation answers (W, N, Enter on the loop panel) go through
     `submit_loop_input`, not the envelope queue, and are outside H4's
     done-condition; they stay as they are.

     Links: area Host and network · draws on 954 · summary H4: only world keys that make an envelope go through the core; camera, inspection, dev clock and menu navigation stay the window's.

956. **H4: the core refuses a stale pick; the window still picks.** Options:
     (1) "Core checks staleness (Recommended)": the window hands the core a
     pick (body, site or nothing) stamped with the world revision of the
     frame it came from, the core refuses a stale one as the endpoints do
     (910), and the product turns it into a nudge, the geometry and identity
     checks staying in the window; (2) "Wait for H6". Mark: "Core checks
     staleness (Recommended)"

     Links: area Host and network · draws on 910 · summary H4: the window hands the core a pick stamped with the world revision and the core refuses a stale one.

957. **H4: the live-input backlog cap moves into the core's driver.** The
     window drops a key once 10 envelopes are queued (`QUEUE_CAP`). Options:
     (1) "In the core driver (Recommended)": live input through the core is
     refused at a set backlog, configurable like the step ceiling, default
     10, while a replay or script queueing directly is not capped; (2)
     "Stays in the window". Mark: "In the core driver (Recommended)"

     Links: area Host and network · draws on 954 · summary H4: the live-input backlog cap moves into the core driver, configurable, default 10; replays and scripts are not capped.
958. **Equal best movement choices use a seeded draw.** Asked 2026-10-16
     after 936's unchanged seed-9 diagnosis: all 52 producers leave before
     feeding at 15, all 64 meals block, and the next tick's two moves score 0
     and take the same first process ID. Options: (1) deterministic movement
     ties keyed by world seed, creature and tick (recommended); (2) design
     food-aware route scoring; (3) keep movement unchanged. Mark:
     "Seeded movement ties (Recommended)". The 20/20 gate stands. This changes
     movement ties, not scores, due/feasible eligibility, non-movement ties,
     food scope, selected-process commitment or rates. Implementation and
     fresh seed 9/gate evidence follow; this ruling does not predict success.

     Links: area Processes and rules · summary Equal best movement choices use a deterministic seeded draw keyed by world seed, creature and tick; scores and eligibility are unchanged.

959. **The family session replicates accepted native authority and sections.**
     H7's assessment was put to Mark on 2026-10-17 before code (923).
     Options: (1) the accepted native log plus opaque section changes
     (recommended); (2) accepted product envelopes replayed through a pinned
     product adapter. Mark: "Accepted native log plus opaque section changes
     (Recommended)". Section-only changes are accepted records; deletion is
     explicit and unknown section bytes survive. A replica does not run a
     product adapter to decide the intent again.

     Links: area Host and network · draws on 924 · summary The family session replicates the accepted native log plus opaque section changes; a replica does not rerun a product adapter to decide intents.
960. **Requests and accepted records use their existing stack seams.**
     Asked 2026-10-17 with 959: (1) admitted stream/Graphshell requests and
     replies, with Stickleback joined spaces carrying host-authored accepted
     records (recommended); (2) stream for both requests and log catch-up.
     Mark: "Stream requests; Stickleback accepted records (Recommended)".
     One host remains the sole accepting writer. The first serialized
     in-process controls precede the actual transport and product gates.

     Links: area Host and network · draws on 959 · summary Requests use admitted stream and Graphshell requests; accepted records use Stickleback joined spaces; one host stays the sole accepting writer.

961. **Mesocosm defaults to the named A1 preset; the shared generator stays Stock.**
     Asked 2026-10-17 after the three A1 candidate forms reached 20/20, since
     907 kept default rates at Stock while H6 needs a playable default launch.
     Options: (1) Mesocosm defaults to A1, generator stays Stock; (2) keep
     Stock in Mesocosm and require explicit A1 selection. Mark: "Mesocosm
     defaults to A1; generator stays Stock". This product-specific exception
     applies once the remaining receipt and physical-witness gates pass. The
     shared generator default and the unchanged A1 qualification gate remain.

     Links: area Mesocosm · draws on 907 · summary Mesocosm defaults to the named A1 preset once receipt and physical-witness gates pass; the shared generator default stays Stock.

962. **The stack supplies identity and admitted connections to family games.**
     Asked 2026-10-17 while wiring H7's shipping VTT owner: the recommended
     split leaves protected identity and admission with the existing stack;
     the game maps an authenticated peer to its native participant and seat.
     Mark: "ah, go ahead. be mindful of sys resources, and of wait time."
     This accepts the pending recommendation. Full-World replication access
     remains a separate authorization from projection or seat access. The
     product must not manufacture keys, grants or trust roots to fill a
     missing stack configuration.

     Links: area Stack and engine · draws on 960 · summary The stack supplies protected identity and admitted connections; games map a peer to participant and seat; products never manufacture keys, grants or trust roots.

963. **An unchosen Mesocosm seed may use a bounded candidate search.**
     Asked with 962 after a clock draw contained no consumer cohort. Mark's
     same "go ahead" accepts the recommended bounded search for an unchosen
     seed. Explicit seeds remain exact. Reject structurally ineligible
     candidates before the expensive witness, disclose the selected seed,
     and retain the full native witness gate for that candidate. A failed
     full witness is not a reason to silently try another world. The bound
     is configurable; this does not select a known passing seed or change
     the shared generator's Stock default.

     Links: area World generation · draws on 875 · summary An unchosen Mesocosm seed may use a bounded configurable candidate search; explicit seeds stay exact and the full native witness gate is kept.

969. **The pack-authoring fixtures are pinned LF by a narrow attributes rule.**
     Asked with 968. Options: (1) `crates/isometry-system/tests/fixtures/*.toml`
     and `crates/isometry-system/examples/packs/**` as `text eol=lf`,
     renormalized and re-checked out (recommended); (2) repo-wide
     `* text=auto eol=lf`; (3) make the test newline-agnostic in the lane's
     own code; (4) leave it to the pack-authoring lane. Mark: "Narrow
     .gitattributes rule (Recommended)". It lands as its own commit after
     the merge, verified by the same test passing on this checkout.

     Links: area Stack and engine · summary Pack-authoring TOML fixtures and example packs are pinned LF by a narrow .gitattributes rule, not repo-wide.

970. **Bodies get a fixed sun matching the tracer's; the field waits.**
     Asked 2026-10-17 with the audit's evidence: the sim exposes no sun or
     day/night field (`ecology.rs` holds only a constant per-site
     `sim:energy` account, and the presets' schedules are read by nothing);
     the tracer's sun is the literal `normalize(0.4, 0.8, 0.3)` with terrain
     light 0.38 + 0.62 * max(0, N.L) (`tracer.wgsl`, `march.wgsl`); the
     tenant lit bodies by ambient only, with `isometer::render::face_shade`
     baked into vertex colours by the quad's local axis, so a rotated body
     kept the wrong shade. Options for the slice: (a) a fixed sun, shadows
     off (recommended); (b) a sun driven from a sim field, shadows on, which
     needs the field admitted first; (c) leave ambient and wait. Mark on the
     slice: "kick b up to the orchestrator"; the orchestrating session then
     confirmed (a), with (b) staying its own assessment item. *Reading, not
     ruled:* that confirmation is the orchestrator's, not Mark's words. Under
     (a) Mark chose "Match the tracer (Recommended)", "Drop it on the tenant
     path (Recommended)" and "Master multiplier, default 1 (Recommended)".
     So the tenant carries a fixed sun toward (0.4, 0.8, 0.3), ambient 0.38
     and sun 0.62 (as kiss3d's albedo/pi diffuse, intensity 0.62 pi), shadows
     off; `geometry.rs` stops applying `face_shade` on the tenant path only
     (the VTT and Eponym `LiveBody` keep it); `--body-light` scales sun and
     ambient together, 1 matching the terrain and 0 dark. The 0.38 and 0.62
     are the tracer's constants, so terrain and bodies read under one sun
     until the admitted field replaces both, and ruling 472's block stays
     the destination. 749's torch shadows are untouched.

     The light field itself is ruled separately (971 to 974): a derived
     function in the sim over (rules, tick, site), read by the tracer and the
     tenant, with a 1,440-tick day and night as sun None. Until that lane
     lands this fixed sun stays the block's default.

     Links: area Presentation · summary Bodies on the tenant get a fixed sun toward (0.4, 0.8, 0.3), ambient 0.38, sun 0.62, shadows off, matching the tracer; --body-light scales both.

971. **The light field is a derived function in the sim.**
     Asked 2026-10-17 in the assessment of the tenant-sun round's option (b),
     which Mark kicked up to the orchestrator. Evidence: ruling 472 already
     reads the light block from sim fields; the sim's only light is the
     `sim:energy` constant, 1,000,000 credited per site per tick by
     `ecology:sunlight` and never limiting; the clock has a minute tick and
     a 525,960-tick year but no day; no plane is stored anywhere; the
     presets' energy schedules are read by nothing; the tracer's sun is a
     WGSL literal with no host input and the tenant's sun is `None`.
     Options: (1) a pure query in `shared/isocosm` over (rules, tick, site)
     returning sun direction, intensity, colour and ambient, nothing stored,
     the presets' schedules becoming parameters (recommended); (2) a stored
     per-site condition written each tick by an agentless process, like
     `world:weather`; (3) host-side from the tick alone. Mark: "Derived
     function in the sim (Recommended)". The card is in the resident views
     plan's field admission section.

     Links: area Ecology · amends 970 · summary The light field is a pure derived function in the sim over (rules, tick, site), read by the tracer and tenant, with nothing stored.

972. **Presentation and ecology both read the light block in the first slice.**
     Asked with 971. Options: (1) presentation only: the tracer gains a
     light slot replacing its literal and the tenant gets its sun, the
     ecology constant stays abundant by declaration, zero hash change
     (recommended); (2) presentation and ecology: the sunlight credit scales
     by intensity, meaningful once light limits growth, which changes every
     trajectory (pins rerecorded, the A1 20/20 matrix rerun); (3) ecology
     first. Mark: "Presentation and ecology". *Reading, not ruled:* the
     limiting amount is a tuning fork under 888 that the lane brings back
     with evidence before the credit changes; the H3 pins and the A1 matrix
     are the gate for that change.

     Links: area Ecology · draws on 971 · summary Presentation and ecology both read the light block: the sunlight credit scales by intensity, with the limiting amount a tuning fork brought back with evidence.

973. **Honest units, with night.**
     Asked with 971: the tick is a minute and the year 525,960 ticks, with no
     day constant; at the configured 10 ticks per second a 1,440-tick day is
     2.4 minutes of play and a year 14.6 hours. Options: (1) day 1,440
     ticks, season from the existing year, sun `None` below the horizon with
     the block's ambient carrying night, tidal lock and season amplitude from
     the preset (recommended); (2) a shorter play-day decoupled from the
     year; (3) arc only, no night yet. Mark: "Honest units, with night
     (Recommended)".

     Links: area Ecology · draws on 971 · summary Honest units with night: a 1,440-tick day, season from the existing year, sun None below the horizon with ambient carrying night.

975. **The core's tick cost is cut by sharing cohorts, copy on write.**
     Asked 2026-10-17 with the split of step 1 (966): on the seed-9 save 15
     ticks take 26.3 s profiled and 20.6 s unprofiled; Session clones and
     drops are 13.25 s (50%), seven clones per tick at about 62 ms to clone
     and 36 ms to drop, 95% of each in `state.population` (2,000+ cohorts,
     28.5 MB serialised); native scheduling is 7.75 s (29%); the refresh
     member checks cost 0.008 s and the Game clone 1.5 ms; one refresh per
     tick saves only about 3.9 s. Options: (1) share cohorts behind an Arc,
     copying on write, serialisation and hashes unchanged (recommended); (2)
     cut the clone count only; (3) both, in that order; (4) re-aim first.
     Mark: "Share cohorts, copy on write (Recommended)". Reading, not ruled:
     the refresh stays three per tick unless the measurement afterwards
     still points at it; 966's "once per tick" was premised on checks that
     measure as free.

     Links: area The sim's schema and saves · summary The core's tick cost is cut by sharing cohorts behind an Arc, copying on write, with serialisation and hashes unchanged.

978. **Mesocosm's default tempo follows the measured tick: 2 ticks per second.**
     Asked 2026-10-17 after 976 left the tempo open. Evidence: the configured
     10 ticks per second (TD2, 2026-08-29) against about 0.45 s per tick after
     the tick lane, native scheduling alone 0.3 s; at 10 the core's clock
     defers steps without limit, so every active callback runs the full
     8-step ceiling, about 3.6 s, and the sim falls further behind each
     round. Options: (1) the default follows measurement, 2 ticks per second,
     TD2's 10 the aspiration revisited when scheduling drops (recommended);
     (2) keep 10 and open a scheduling lane; (3) decouple the sim onto its
     own thread with held frames. Mark: "Tempo follows measurement: 2
     ticks/s (Recommended)". `mesocosm-genet`'s config default moves to 2; a
     1,440-tick day is then 12 minutes of play. The same round archived both
     merged lane sessions.

     Links: area Mesocosm · draws on 976 · summary Mesocosm's default tempo follows the measured tick: 2 ticks per second, TD2's 10 remaining the aspiration revisited when scheduling drops.

979. **The sun's parameters are an optional `Rules.sun`, absent meaning the default.**
     Asked 2026-10-17 at the light-field lane's start. Evidence: presets are
     applied at founding only as site conditions (`generate.rs`,
     `Preset::apply`), their schedule strings are prose read by nothing, and
     Rules fields skip serialising when absent so hashes stay. Options: (1)
     an optional `Rules.sun: Option<SunRules>`, presets setting it at
     founding (recommended); (2) site conditions written by the preset; (3)
     caller-passed preset, nothing stored. Mark: "Optional Rules field,
     absent = default (Recommended)". *Reading, not ruled:* the parameters
     are integers (the rules are `Eq`): tidal lock, tilt in hundredths of a
     degree, intensity percent, colour in thousandths; latitude is the site
     condition `light:latitude`, default 40 degrees.

     Links: area The sim's schema and saves · summary The sun's parameters are an optional Rules.sun: Option<SunRules>, set by presets at founding; absent means the default.

980. **A world with no sun rules gets a real day.**
     Asked with 979. Options: (1) a 1,440-tick day and a mild 23.5 degree
     season (recommended); (2) the fixed 970 sun unless a preset sets a
     schedule. Mark: "Default day: 1,440-tick day, mild season
     (Recommended)". Tidal-locked worlds take 970's fixed sun exactly.

     Links: area Ecology · draws on 979, 973 · summary A world with no sun rules gets a 1,440-tick day and a mild 23.5 degree season; tidal-locked worlds take 970's fixed sun.

982. **The light credit is 500 per site per tick at full sun.**
     Asked 2026-10-17 as 888's tuning fork, with the lane's evidence: on the
     seed-9 A1 world (offered runtime, Grouped) producers drew 605, 227, 465,
     315, 11 and 0 per site over the 10 ticks it ran before a question held
     it, peak 463 at one site in one tick, against a credit of 1,000,000 per
     tick (levels near 3,000,000). *This is a 10-tick rate, not a measured
     day.* Options: (1) 500, about the observed burst, limiting when
     producers crowd or the sun is low (recommended); (2) 100, limiting
     broadly; (3) 2,000; (4) measure the whole A1 horizon first. Mark: "500
     per site per tick (Recommended)". Receipt: `Code/testing/isometry/
     light-field-2026-10-17/light-draw.log`.

     Links: area Ecology · draws on 972, 888 · summary The light credit is 500 per site per tick at full sun.

983. **The site's light store holds a night.**
     Asked with 982: the credit is nothing at night and the old account
     capped at two ticks of credit. Options: (1) a store of about 12 hours of
     full credit, 720 x 500 = 360,000, so producers draw a reserve down in
     the dark (recommended); (2) keep the two-tick cap, producers stop at
     night; (3) a 10 percent starlight credit. Mark: "Cap holds a night's
     light (Recommended)".

     Links: area Ecology · draws on 982 · summary The site's light store holds a night: about 720 x 500 = 360,000 of full credit, so producers draw a reserve down in the dark.

984. **The credit scales through a new `Effect::Sunlight`.**
     Asked with 982 when implementation found `Effect::Transform`'s `give`
     a fixed ledger of integers. Options: (1) a new effect `Sunlight { who,
     account, full }`, resolved at staging into the transform it comes to
     (recommended); (2) computed amounts in `give` with a sun reading; (3) a
     native step before the tick. Mark: "New effect: Effect::Sunlight
     (Recommended)". Reading, not ruled: the intensity is read at world scale
     (no site latitude) at the act's tick.

     Links: area Ecology · draws on 982 · summary The credit scales through a new Effect::Sunlight { who, account, full } resolved at staging into the transform it comes to.

985. **A faction's general protocols belong to the world's ruleset.** Docketed
     as D1 in the design record's §3.2.2 and accepted by Mark on 2026-09-21;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading:
     Where peer to peer agreement cannot emerge, "a few general protocols"
     apply; because they are general they belong to the world's ruleset
     (ruling 41) and not to the faction, which keeps the faction stateless.

     Links: area Processes and rules · draws on 63, 41 · summary Where peer agreement cannot emerge, a faction's general protocols belong to the world's ruleset, keeping the faction stateless.

986. **A form of governance is who qualifies and by what act.** Docketed as D2
     in the design record's §3.2.2 and accepted by Mark on 2026-09-21; made a
     case on 2026-10-22 (Mark: "Dockets become cases"). The reading: A form is
     a pair, who qualifies to decide and by what act the decision is made.
     Qualification is a predicate over what the record already holds for an
     entity: standing in its three parts (ruling 56), possessions (ruling 53),
     body and age, lineage, provenance. Rule by the wealthy, the old, the
     strong, the ordained, the well born, the divine, everyone, or whoever the
     lot falls on is then one shape with a different predicate. The deciding
     act is a process of the first shape: voting, fighting, buying, divining,
     inheriting, taking. Both halves are vocabulary the sim has, so forms are
     data to be authored and generated, never an enum.

     Links: area Processes and rules · draws on 63 · summary A form of governance is a pair, who qualifies to decide and by what deciding act; forms are data, never an enum.

987. **The form that emerges is ranked preference over alignment.** Docketed
     as D3 in the design record's §3.2.2 and accepted by Mark on 2026-09-21;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading: A
     sophont's ranking of forms of governance falls out of its alignment, and
     support transfers down each ranking until a form holds. This is the sim's
     mechanism for which form emerges, weighted by influence exactly as ruling
     51 weights a faction's alignment, and not necessarily an election anyone
     in the world holds. It runs at the rung transition that founds a polity,
     and its winner is asserted as the constitution. At the cohort rung the
     ballots are ruling 50's distributions, so the count runs over blocs and
     not heads. Ties break on the seed.

     Links: area Processes and rules · draws on 63, 50, 51 · summary Which form of governance emerges is ranked preference derived from alignment, weighted by influence, run at founding; ties break on the seed.

988. **A polity enforces with what it provides and asserts.** Docketed as D4
     in the design record's §3.2.2 and accepted by Mark on 2026-09-21; made a
     case on 2026-10-22 (Mark: "Dockets become cases"). The reading: A polity
     enforces with what it provides and what it asserts. Ruling 38 already
     made provisioning a process of the faction and polity rungs, and what is
     provided can be withheld. Everything the record lets a polity assert it
     can also revoke: rank, office and membership (ruling 56), property
     (ruling 53), home and borders (ruling 54). Force is one more means and
     not the defining one. So the "or else" of a rule is an ordinary process
     of the first shape, with a cost to the polity that applies it, and a
     guild that bars a smith from the forge and a crown that sends soldiers
     are the same shape. The act by which a form decides suggests the means by
     which it enforces: rule by buying withholds pay, rule by divining
     withholds rites, and rule by fighting fights.

     Links: area The sim's schema and saves · draws on 64, 38, 66 · summary A polity enforces with what it provides and asserts, withholding or revoking; force is one means; the way of deciding suggests the means.

989. **Efficacy is read as a track record, never asserted.** Docketed as D5 in
     the design record's §3.2.2 and accepted by Mark on 2026-09-21; made a
     case on 2026-10-22 (Mark: "Dockets become cases"). The reading: Whether a
     means is efficacious is a track record, how often applying it produced
     compliance, which is ruling 50's trust, held about the constitution as
     about any tenet. After founding, the constitution is asserted and stays
     put while its members' derived preference keeps moving, so the distance
     between the two can be read at any time. Enforcement is what holds that
     gap open.

     Links: area The sim's schema and saves · draws on 50, 51 · summary Whether a means is efficacious is a track record, never asserted; the gap between a constitution and its members' preference is read at any time.

990. **A polity's four conditions are read from the record.** Docketed as D6
     in the design record's §3.2.2 and accepted by Mark on 2026-09-21; made a
     case on 2026-10-22 (Mark: "Dockets become cases"). The reading: Each
     condition is something the record can already tell. Inactive: it lacks
     support for its operations and does nothing. Suppressed: it has support,
     and another's enforcement stops it operating (the banned guild, the
     occupied town, the outlawed church); its members can still act as a
     faction, by consent and in secret. Superseded: another polity now binds
     the acts that were its focus, and that one's enforcement works.
     Subordinated to a faction: its methods still run and their outcome is one
     faction's will, because that faction fills the party qualified to decide
     or holds the means, the part commanding the whole.

     Links: area Processes and rules · draws on 65, 66 · summary A polity's conditions short of death (inactive, suppressed, superseded, subordinated) are each read from the record, not exclusive, reversible.

991. **No will to continue is built in.** Docketed as D7 in the design
     record's §3.2.2 and accepted by Mark on 2026-09-21; made a case on
     2026-10-22 (Mark: "Dockets become cases"). The reading: Where an
     institution is seen clinging on, that is derived and needs no drive of
     its own. The members it provisions support it for their own reasons, and
     the sharpest case is ruling 66's subordination: an institution that
     "wants to continue" is one subordinated to the faction of its own
     officers, who want their provision to continue.

     Links: area The sim's schema and saves · draws on 68, 66 · summary No will to continue is built into a polity; an institution seen clinging on is subordinated to its own officers' faction.

992. **Contingency is borrowed enforcement, and collapse cascades.** Docketed
     as D9 in the design record's §3.2.2 and accepted by Mark on 2026-09-21;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading:
     What makes the host an institution is that an agreement made under it
     holds without its parties enforcing it themselves, and contingency is
     borrowed enforcement: the guild's "or else" is backed by the town's
     courts, the town's by the crown. A contingent polity is cheap to found,
     because it need not bring means of its own, which is why a settled world
     grows institutions and a wild one does not. And when a host fails,
     everything contingent on it must find means of its own or fall back to a
     faction, so collapse cascades, and those are events the hagiograph keeps.

     Links: area The sim's schema and saves · draws on 64 · summary A contingent polity borrows its host's enforcement, so it is cheap to found, and a host's failure cascades through what is contingent on it.

993. **Collection is roots and reachability.** Docketed as D11 in the design
     record's §3.4.1 and accepted by Mark on 2026-09-21; made a case on
     2026-10-22 (Mark: "Dockets become cases"). The reading: The roots are
     what is relevant to a player: the foreground, and what the played entity
     is, holds, knows and is related to; every asserted fact a live thing
     still relies on, a claim, a membership, an agreement, a constitution;
     every pending due event; and the legendary tier whole, which is never
     collected. A thing of note is kept while something in the record reaches
     it from a root, by a relation, a possession, a memory within reach, or a
     cause-link. Unreached, it is collectible, and collecting it folds it back
     into the distribution it was realised from, so counts and masses stay
     conserved and only its deviations are lost. The test that makes this safe
     is observational: collect only what nothing reachable from a player could
     tell apart from its regenerated self.

     Links: area Record and history · draws on 70, 71 · summary Collection is roots and reachability: a thing of note is kept while something reaches it from a root; collecting folds it back, conserving counts.

994. **A thing of note sheds in steps: individual, stub, event.** Docketed as
     D12 in the design record's §3.4.1 and accepted by Mark on 2026-09-21;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading: A
     thing of note sheds in steps, the full individual, then a stub, what an
     event needs to name its participant (a kind, a lineage, a faction, a name
     if it had one), then the event alone. So events are the floor of the
     middle tier, and they refer to their participants by stub, never by a
     live individual, so that a participant can be collected without breaking
     the event that mentions it.

     Links: area Record and history · draws on 71 · summary Collection is graded: the full individual, then a stub an event names it by, then the event alone; events refer to participants by stub.

995. **The tiers of keeping are a component, not a crate per word.** Docketed
     as D13 in the design record's §3.4.1 and accepted by Mark on 2026-09-21;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading: A
     component, one module of the sim, and not a crate per word. The three
     tiers are one mechanism over every kind of thing, and the invalidation
     criteria are one policy, so a denizen crate beside a place crate and a
     thing crate would hold the same keeping machinery three times. It can
     still be built to be tested and swapped alone behind its own seam inside
     the sim. The kinds are then data: the inhabitant, the place, the thing,
     the event.

     Links: area Organs: glyphs, impresa, hagiograph · draws on 69 · summary The tiers of keeping are one component of the sim, not a crate per word; the kinds are data.

996. **A location's kind is read off its pattern.** Docketed as D14 in the
     design record's §3.7.1 and accepted by Mark on 2026-09-21; made a case on
     2026-10-22 (Mark: "Dockets become cases"). The reading: A location's kind
     reads as a biome does, the characteristic pattern of what is built there,
     who is there and what it is used for, with ruin as the clearest case.
     What is stored for a location is its extent, its modifiers and
     conditions, what happened there and who claims it; the kind is how that
     reads. A location may be only "potential", derived from terrain and not
     yet of note, so the word spans the ambient and of-note tiers and the
     keeping tier says which. A highway is a location that is also a route, a
     long thin extent over many sites, so the graph's edges can be places of
     note too.

     Links: area World generation · draws on 72, 14 · summary A location stores extent, modifiers, conditions, history and claims; its kind is read off them; a location may be potential; a highway is also a route.

997. **A nesting step says down, up, across and ratio.** Docketed as D15 in
     the design record's §3.7.1 and accepted by Mark on 2026-09-21; made a
     case on 2026-10-22 (Mark: "Dockets become cases"). The reading: What one
     step has to say, so that steps compose. Down: how the finer map is
     generated from the node above it, its terrain, biome, conditions and
     seed, so an unvisited interior costs nothing. Up: how the finer map reads
     from above, its state summarised as the node's conditions. Across: how
     adjacency crosses the boundary, so a route is one path through several
     levels. Ratio: how much finer, a power of two for volumes and free for
     graphs. The last step meets the volume, its size a world's setting with a
     default. Steps can be added at either end or in the middle without
     touching the others: a system above the world, the planes beside it, a
     dungeon's floors or a ship's decks, a body as a map.

     Links: area World generation · draws on 74, 69 · summary A nesting step says how the finer map is generated (down), reads from above (up), crosses boundaries (across), and how much finer (ratio); steps compose.

998. **What a critter-world does reaches the world as environment.** Docketed
     as D16 in the design record's §3.7.1 and accepted by Mark on 2026-09-21;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading:
     What the critter does, walking, turning, sleeping, dying, reaches the
     world as its environment. That is a reason beyond the sphere for holding
     the map as adjacency: a body's surface is no regular grid at all.

     Links: area World generation · draws on 73, 39 · summary For a world resting on a critter, what the critter does (walking, turning, sleeping, dying) reaches the world as its environment.

999. **Lifting and restriction; processes declarative enough to aggregate.**
     Docketed as D17 in the design record's §3.3 and accepted by Mark on
     2026-09-21; made a case on 2026-10-22 (Mark: "Dockets become cases"). The
     reading: Lifting takes an aggregate to individuals: sample from the
     distribution, conditioned on every asserted fact, from the seed, so the
     same place lifts the same way twice. Restriction takes individuals to an
     aggregate: conserve the accounts, counts, mass, energy, and fold what
     deviated into the distribution's parameters. Similitude is then two
     checks on seeded draws: restricting what was just lifted returns the
     aggregate, and a population run each way ends in the same statistics. A
     process must be declarative enough for its aggregate form to be derived
     from the one definition, typed inputs, costs, effects and rates; an
     opaque script can only be executed, never integrated, so piccolo authors
     and lowers to definitions, and scripted hooks run only in the foreground.

     Links: area Processes and rules · draws on 75, 32, 15 · summary Lifting samples individuals from an aggregate by seed; restriction conserves accounts back; processes are declarative so their aggregate derives, scripts foreground-only.

1000. **Two bindings as intent; the contract round-trips through bytes.**
     Docketed as D18 in the design record's §5.2, 5.4 and accepted by Mark on
     2026-09-21; made a case on 2026-10-22 (Mark: "Dockets become cases"). The
     reading: Ruling 78 alone, as a design discipline. Two bindings as an
     intent and not a duty: native first, the component binding built when the
     first rung-3 consumer exists, the contract shaped so that day needs no
     redesign. For erosion, a test and not a runtime: every contract type
     round-trips through bytes, and the contract crate may not depend on the
     sim's internals. With it the record's recommendation and verdict: keep
     the first-party cores and the sim native, the contract shaped as WIT
     forces so it can be bound as a component any day; total componentisation
     is not worth it applied totally, and the rule of thumb is call frequency
     times data volume, cold and narrow behind a component when it wants to be
     pluggable, hot or wide native.

     Links: area Stack and engine · draws on 78, 77, 76 · summary Ruling 78 as discipline: native binding first, a component binding when a rung-3 consumer exists; contract types round-trip through bytes; cores stay native.

1001. **A secret is a taboo's set of knowers, and ends by leaking.** Docketed
     as D21 in the design record's §3.4 and accepted by Mark on 2026-09-22;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading:
     What makes a secret is the taboo and not the information: a tenet holding
     a strongly negative opinion of the telling, a norm with no formal "or
     else", whose sanction is reputational, so a faction can keep secrets with
     no polity behind it. A secret is a deed whose reach is held down, and its
     value is what releasing it would do to its subject's standing among those
     who would learn it. Telling is a choice under scarcity by a knower,
     relation and opinion its trust and affinity, goals what telling buys,
     personality the thresholds. The regime is notes, not a field: an explicit
     set of knowers, each holding a note. A secret ends by leaking: once a
     telling seeds the reach field at a place it is a rumour.

     Links: area Processes and rules · draws on 87, 50, 56 · summary A secret is a taboo on telling (a norm with reputational sanction); its value is what release would do to standing; telling is a choice; it ends by leaking.

1002. **Realignment is the epoch boundary at the scale of a world.** Docketed
     as D22 in the design record's §3.9 and accepted by Mark on 2026-09-22;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading:
     Realignment is ruling 57's epoch boundary at the scale of a world, the
     shop between rounds, and it is a rung transition; a change from hex
     exploration to piloting a ship in orbit is also a change of foregrounded
     scope, so a realignment may add steps to the nesting as well as change
     the rules.

     Links: area World generation · draws on 90, 57 · summary Realignment to another ruleset is the epoch boundary at world scale, a rung transition, and may add nesting steps as well as change rules.

1003. **Going back is replay; editing the past forks.** Docketed as D23 in the
     design record's §3.9 and accepted by Mark on 2026-09-22; made a case on
     2026-10-22 (Mark: "Dockets become cases"). The reading: Going back is
     cheap because of the derivation rule: the state at any past time is the
     log replayed to then, with checkpoints at epoch boundaries bounding the
     cost. Going back and editing is asserting a fact in the past, which makes
     a fork, never a rewrite, so the timeline a founder edits is the same
     thing as the branching a moot does. "Not overwhelm" is the three tiers of
     keeping used as an interface: the timeline shows legend first, things of
     note on zooming in, and the ambient only where someone looks.

     Links: area World generation · draws on 91, 7, 126 · summary A past state is the log replayed (checkpoints bound the cost); editing the past asserts a fact there, making a fork; the timeline shows legend first.

1004. **Nesting is the sim's; scopes are a game's.** Docketed as D24 in the
     design record's §3.7.1 and accepted by Mark on 2026-09-22; made a case on
     2026-10-22 (Mark: "Dockets become cases"). The reading: Nesting is the
     world's and the sim's: a node opening into a finer map, allocated as
     locations are generated, free of scale and with no named tiers; the sim's
     only spatial nouns are the world map, the site, the region, the location
     and the volume under them. A scope is a game's: the window it frames over
     that nesting to play in, an extent and a grain with its grid projected
     over it. Each game has its own scopes over the one world, so the same
     node can be a terrarium to one game, a dungeon to another and a battlemap
     to the third. What a ruleset may still fix is grain; what founding
     supplies for nesting is only the kinds of location, as data.

     Links: area World generation · draws on 92, 72, 74 · summary Nesting (a node opening into a finer map, no named tiers) is the sim's; a scope (a window of extent and grain) is a game's; each game has its own.

1005. **Needs by tier; value is derived; trade is flow.** Docketed as D25 in
     the design record's §3.3.1 and accepted by Mark on 2026-09-22; made a
     case on 2026-10-22 (Mark: "Dockets become cases"). The reading: In brief
     (the full reading is the archived record's §3.3.1): value is derived and
     never stored, a reading over supply, capability and need at a place and a
     time, and a price is that reading at a market; money is an item whose
     only use is exchange, an asserted claim and never a primitive of the sim.
     Why things act is tiered: a critter acts when a need crosses a threshold;
     a denizen chooses a desire by belief and value; a character answers a
     role or an institution; need is read from alignment, acts and
     predilection. In aggregate a cohort's demand is a rate and a place's
     supply a stock, and exchange is flow along routes from cheap to dear, the
     diffusion the record uses for reach and plague with a price gradient;
     provisioning is a polity running that flow on purpose.

     Links: area Processes and rules · draws on 94, 37, 38 · summary Value is a reading over supply, capability and need, never stored; money is an asserted claim; acting is by tier; trade trends are flow along routes.

1006. **An act's score is need, trust and approval.** Docketed as D26 in the
     design record's §3.3.1 and accepted by Mark on 2026-09-22; made a case on
     2026-10-22 (Mark: "Dockets become cases"). The reading: A sophont chooses
     among the acts it could take by three readings: whether the act addresses
     a need it has, whether it has the capability (ruling 50's trust that the
     process yields the effect), and whether its alignment approves the means
     and the effect (ruling 50's opinion); the score of an act is need times
     trust times approval, every factor derived. What gets made is chosen by
     the same score, and an item bears its making. Nobody churns weapons to
     level up: a fungible act folds into the cohort's distribution and leaves
     no note, so grinding produces aggregate competence and never a legend. A
     fey mood is a triggered need: a rare confluence raises a need whose only
     satisfier is one act.

     Links: area Processes and rules · draws on 95, 50, 75 · summary An act is chosen by need times trust times approval; what gets made follows the same score; grinding yields aggregate competence, never a legend; a fey mood is a triggered need.

1007. **A material's properties are its lineage's traits.** Docketed as D28 in
     the design record's §3.3.1 and accepted by Mark on 2026-09-22; made a
     case on 2026-10-22 (Mark: "Dockets become cases"). The reading: A
     material's properties are its lineage's traits: hardness, weight, edge
     and wear are readings of the lineage a lot of nis came from. Interactions
     are effects: which effects a nis of that lineage produces and suffers,
     "fire against wood" being burning applied to one lineage's nis, read from
     its traits and the record, never from a table of pairs. Learning the
     ecology is the note machinery: what a thing is made of is a note on a
     lineage, held by whoever has met it, and a sophont with no note on a
     lineage cannot use it. Scarcity and provenance are literally ecological:
     a rare material is a rare critter, an extinction ends a material, and a
     sword carries its lineage.

     Links: area Ecology · draws on 97, 57, 49, 80 · summary A material's properties are its lineage's traits and its interactions are effects; what a thing is made of is a note; scarcity and provenance are ecological.

1008. **The world is an entity on the ladders the record already has.**
     Docketed as D29 in the design record's §3.3.1 and accepted by Mark on
     2026-09-22; made a case on 2026-10-22 (Mark: "Dockets become cases"). The
     reading: In brief (the full reading is the archived record's §3.3.1): the
     world is one more entity on the record's ladders, at the root, and each
     kind it may be is a position on a ruled axis: inert, a body without
     agency; sentient, a turtle, flora or fauna, a critter of the world's
     kingdom at macro scale; a sophont, with naming and memory, able to
     assert, own and hold tenets; a god, ruling 42's divinity; dead, its
     entity on the planes and its body still the ground. Agentless processes
     are the world entity's own metabolism, chosen where the world is a
     critter or a sophont; geology is anatomy, a world's kinds of nis its own
     trait catalogue; and founding chooses or draws the world's kind.

     Links: area World generation · draws on 98, 99 · summary The world's open kind (inert, sentient, sophont, god, dead) is a position on ruled axes; agentless processes are its metabolism; geology is anatomy.

1009. **The world has a lineage, and its magic is a trait on six axes.**
     Docketed as D30 in the design record's §3.10 and accepted by Mark on
     2026-09-22; made a case on 2026-10-22 (Mark: "Dockets become cases"). The
     reading: In brief (the full reading is the archived record's §3.10;
     ruling 404 and the functional generation plan now own magic's
     composition): the world has a lineage, its genotype the catalogue its
     critters draw on and its epoch boundary realignment. A world is
     generative from four sources: typing, bodies, need, and the record
     feeding back into what it recognises. A magic system is one trait of the
     world's genotype, described on six axes: source, gate, orientation, cost
     and breach, rhythm, manifestation. "Bends the rules" means a system
     declares which of the world's invariants it may suspend, under what gate,
     at what price, leaving what residue. Divinity reads a magic system
     through its effects, acquisition and frequency, so the ladder of forms is
     derived from the magic's gate axis.

     Links: area Processes and rules · draws on 101, 102, 49 · summary The world has a lineage (genotype its catalogue, realignment its boundary); a magic system is a world trait on six axes; bending the rules is declared suspension.

1010. **An arc is a thread of cause-links; the storyteller is the world's
     agency.** Docketed as D31 in the design record's §3.11 and accepted by
     Mark on 2026-09-22; made a case on 2026-10-22 (Mark: "Dockets become
     cases"). The reading: In brief (the full reading is the archived record's
     §3.11): every part of an arc is a thing the sim already has, a goal, its
     consequence by the hagiograph's test, real reasons as the act's score,
     pursuit, obstacles, stakes, turning points, an ending; an arc is a thread
     in the record, the cause-links from a goal's adoption to its ending, and
     a consequential goal leaves a note, but the sim never stores an arc. The
     game reads threads out of the record, chooses which to foreground and
     names them. The storyteller is the world entity's agency: a world given a
     disposition and goals directs its own weather, plagues and wild events,
     set by its founder, static or fluid.

     Links: area Directing and play · draws on 103, 98, 99 · summary An arc's parts are things the sim has; an arc is a derived thread, never stored; a game makes its statement; the storyteller is the world entity's agency.

1011. **Related worlds: descent and neighbourhood, neither merging.** Docketed
     as D33 in the design record's §3.12 and accepted by Mark on 2026-09-22;
     made a case on 2026-10-22 (Mark: "Dockets become cases"). The reading:
     Two relations, neither merging anything: each world keeps its own log and
     its own played clock, and what crosses between them is things with
     provenance. Descent: forked worldlines are the world kingdom's lineage
     tree, siblings sharing the log up to the fork. Neighbourhood: unrelated
     worlds placed in relation by assertion at the rung above the world, a set
     of worlds with routes between them, a crossing a rung transition recorded
     in both logs; what arrives carries its home world's nis, which is where
     exotic materials come from. Worlds under no host are a faction among
     themselves, holding by their owners' consent.

     Links: area World generation · draws on 105, 126, 11 · summary Related worlds relate by descent (forked worldlines share a prefix) or neighbourhood (worlds placed in relation, crossings recorded in both logs); nothing merges.

1012. **Quality is affordance, not a number.** Docketed as D27 in the design
      record's §3.3.1 and accepted with the docket's remaining items on
      2026-09-22 (ruling 106, Mark: "accept all as suggested"); made a case on
      2026-10-22 (Mark: "Dockets become cases"). The reading: what a weapon
      lets its wielder do, read by the strike system, and never a stat to
      compare. Four sources vary it: sort, the shape, which sets which strikes
      exist at all; material, its lineage's traits; make, the maker's skill
      and the act it came from; and condition, what has happened to it since.
      They resolve into which techniques the weapon affords, cleanly, poorly
      or not at all, so no weapon is one a skilled hand cannot swing, only one
      that affords less. Improvement is fair because it is an act, a recipe
      with the item as an input, and a blade that has done things of note may
      afford a technique of its own.

      Links: area Processes and rules · draws on 96, 95, 143 · summary An item's quality is what it affords (sort, material, make, condition read as techniques), never a stat; improvement is an act.

1013. **Pathfinder 2e is the first edition built whole at the table.** Ruled
      by Mark on 2026-10-30 (Mark: "PF2e first"), offered against the 5e SRD,
      which the adapter alone accepts today, and Ars Magica. The reading:
      legal character creation and ordinary play (equipment, actions,
      incapacitation, recorded GM rulings, save and reopen) are completed for
      Pathfinder 2e's Player Core Remaster before the other editions, which
      are specified but not built meanwhile. It keeps 249's order: the same
      edition is calibrated first where the sim leads.

      Links: area The VTT · draws on 249, 522, 541, vtt.5 · summary Pathfinder 2e is completed first for character creation and ordinary play; the other editions are specified meanwhile.

1014. **A VTT campaign's ruleset leads unless its founder picks the sim.**
      Ruled by Mark on 2026-10-30 (Mark: "Ruleset leads"), offered against
      the sim leading by default and no default. The reading: a campaign
      founded at the table defaults to its ruleset leading, the sim's
      background following wherever the edition specifies behaviour; sim-led
      is chosen at founding. 114's calibration binds only sim-led worlds, so
      V3's bench is required of an edition only when a world it plays in is
      sim-led.

      Links: area The sim's schema and saves · draws on 114, 538, 249, schema.8 · summary A VTT campaign defaults to ruleset-led; sim-led is a founding choice; calibration binds only sim-led worlds.

1015. **A town is a reading over founders.** Ruled by Mark on 2026-10-30
      (Mark: "Reading over founders"), offered against pure emergence from
      deep time and towns asserted as a place kind. The reading: the founding
      flow places households where geography and resources favour them, and
      a town is read off the dwellings, tenancies, work and institutions
      gathered there, never stored, so its history is what explains it.

      Links: area World generation · draws on 54, 63, schema.12 · summary Founding places households by geography and resources; a town is read off dwellings, tenancies, work and institutions, never stored.

1016. **Pathfinder 2e content is checked against ORC records only.** Ruled by
      Mark on 2026-10-30 (Mark: "Foundry records, ORC only"), offered against
      his own Player Core copy as the authority and both. The reading: as the
      effects README already did for spells, a lane reads the foundryvtt/pf2e
      repository's records as a check, never vendoring them, keeps only those
      whose source is a Remaster book under the ORC licence, and writes
      original summaries and executable fields. Legacy OGL records are not
      used.

      Links: area Licensing and sources · draws on 541, licensing.1 · summary Pathfinder 2e content is checked against foundryvtt/pf2e records from ORC Remaster books only, read not vendored, written as original summaries and fields.

1017. **A town's size is a ladder of words.** Ruled by Mark on 2026-10-30
      (Mark: "A size ladder"), offered against one word with size as a read
      property and leaving the word to a later round. The reading: the
      reading of 1015 takes a ladder of words by size (hamlet to city, for
      example), read off population and institutions; the words and the
      thresholds are set by a naming round, never coined in a lane.
      "Settlement" is not one of them: it already carries two senses in code.

      Links: area World generation · draws on 1015, naming.3 · summary The town reading takes a size ladder of words read off population and institutions; a naming round sets words and thresholds.

1018. **Each edition declares its campaign's shape.** Ruled by Mark on
      2026-10-30 (Mark: "Each edition declares it"), offered against one
      universal campaign that editions extend and deferring until Ars Magica.
      The reading: an edition declares its campaign structure (a party or a
      troupe, its unit of time, holdings shared like a covenant), and the
      table supplies what every shape uses: sessions, consent and the record.
      Pathfinder 2e's party, levels and encounters are one declaration, not
      the table's model.

      Links: area The VTT · draws on 522, 1013, vtt.2, vtt.5 · summary An edition declares its campaign's structure (party or troupe, time unit, shared holdings); the table supplies sessions, consent and the record.

1019. **A look is a voxel recipe; class names bind it.** Ruled by Mark on
      2026-10-30 (Mark: "Voxel source, classes bind"), settling the
      board-on-isometer plan's BD5, offered against the stylesheet theory
      (voxels baked to sprites a stylesheet names) and voxels only. The
      reading: a character's or thing's look is an isometer `Appearance`
      recipe (layers, palette, clips); PROJECT_DESCRIPTION pillar 3 holds as
      data, a pack mapping class names to layers and palettes, so a campaign
      reskins without touching the app; the creator's preview and the board
      read the same recipe.

      Links: area The VTT · draws on 1018 · summary A look is an isometer Appearance recipe; packs bind class names to layers and palettes; preview and board read one recipe.

1020. **Pathfinder 2e hit points stay the edition's; crits, dying and death
      reach the body.** Ruled by Mark on 2026-10-30 (Mark: "Crits and dying
      wound"), offered against hit points alone with death killing, every hit
      wounding in proportion, and death by the DM's ruling. The reading: hit
      points are edition state, read as vigour rather than wounds; a critical
      hit and going dying each issue a native wound, and death issues the
      native kill, in the same transaction as the edition's change; a native
      death shows on the sheet.

      Links: area The VTT · draws on 542, 1013, vtt.5 · summary PF2e HP stays edition state; critical hits and dying wound the native body and death kills it, in the same transaction.

1021. **Pathfinder 2e's first slice is two ancestries and two classes.**
      Ruled by Mark on 2026-10-30 (Mark: "2 ancestries, 2 classes"), offered
      against every ancestry with two classes and all of Player Core's first
      level. The reading: Human and Dwarf, Fighter and Rogue, about 75 ORC
      Remaster records, with no spellcasting; the slice grows once the path is
      proven.

      Links: area The VTT · draws on 1013, 1016 · summary The first PF2e slice: Human and Dwarf, Fighter and Rogue, about 75 records, no spellcasting.

1022. **A character's look is kept at the table as a class plus overrides.**
      Ruled by Mark on 2026-10-30 (Mark: "Table record, class plus
      overrides"), offered against the fully resolved recipe and the native
      binding with the sim on only. The reading: a per-token look lives in the
      table's record beside its sheet, replicated by its own event, sim on or
      off; it stores the class name with per-slot choices and palette
      overrides, so a pack's reskin reaches whatever is not overridden; the
      existing sprite names become classes with no save migration.

      Links: area The VTT · draws on 1019, vtt.7 · summary A look is kept in the table record as class plus per-slot and palette overrides, sim on or off; reskins reach what is not overridden.

1023. **People grow out of a world given deep time; a quick-start world is a
      modded, shared world.** Ruled by Mark on 2026-10-30 (Mark: "3 for sure.
      A quick start world is a modded, shared world!"), replacing 1015's
      founding flow, after his account of the ladder: creatures arising from
      the world's conditions, shaped by them and finding the leverage they
      can exact from them, becoming named, gaining friends, enemies, families
      and lineages, depending on particular areas and living in larger groups
      as their needs allow, growing "from a small hamlet into a town, then
      perhaps into a city", factions and polities developing out of them;
      each product foregrounding different layers of one sim. Offered against
      emergence only and founders as a scaffold to be replaced. The reading: in
      a world given deep time the generator places no people; lineages grow
      and spread where conditions hold them, and individuals, kin, households,
      factions and polities are read up the ladder from them, individuals
      lifted where play attends. A quick-start world is a modded, shared world
      whose founders are authored and enter by the founding door at its first
      tick. A town is read the same way in either.

      Links: area World generation · replaces 1015 · draws on naming.2, processes.6, processes.12, ecology.4, schema.5 · summary In a world given deep time people grow out of its conditions and are read up a ladder; a quick-start world is a modded, shared world with authored founders.

1024. **A faction decides by consent; a polity adds a constitution.** Ruled by
      Mark on 2026-10-30 (Mark: "Keep the split"), after he called a polity
      "really just any type of group decision-making". The reading: his phrase
      covers both rungs; groups act first as factions, and a constitution
      makes one a polity.

      Links: area The sim's schema and saves · draws on schema.5, processes.12 · summary "Any group decision-making" covers faction and polity; a constitution makes a faction a polity.

1025. **A town's tiers are hamlet, town and city.** Ruled by Mark on
      2026-10-30 (Mark: "Hamlet, town, city"), offered against settlement,
      town, city and a later naming round, settling 1017's ladder. The
      reading: three tiers read off size; their thresholds are set from
      measured draws, not guessed.

      Links: area World generation · draws on 1017, naming.3 · summary The ladder's tiers are hamlet, town and city, thresholds set from measured draws.

1026. **Pathfinder 2e is built beside the skeleton, from a saved draft, and
      stays marked experimental.** Ruled by Mark on 2026-10-30 (Mark: "All as
      recommended, keep the mark"), the rules lane's forks taken as drafted.
      The reading: Remaster attribute modifiers are built beside the skeleton's
      ability scores; the system id `pf2e-srd` stays an opaque key with a
      correct label; equipment is unmarked at the table and coins are edition
      state, as the Magic Weapon loop has them; encounter mode (initiative,
      three actions, reaction) is declared in edition state; ancestry is an
      edition fact, tied to a native lineage later; creation is a saved draft,
      one explained step at a time; legality and derivation are Rust over
      TOML, Lua kept for degrees of success; the book's words name fields
      only. Play stays marked experimental until the per-world lead choice
      exists.

      Links: area The VTT · draws on 1013, 1014, 1018, 1020, vtt.5, vtt.6 · summary PF2e is built beside the skeleton from a saved, explained draft; encounter mode is the edition's; play stays marked experimental until the lead choice exists.

1027. **A place's tier counts its residents.** Ruled by Mark on 2026-10-30
      (Mark: "Size counts residents"), offered against households with a
      standing tenancy, the region lane's other forks taken as drafted. The
      reading: hamlet, town or city is read off living residents; a dwelling
      is an asserted place linked to its map site through a `Location`; the
      tier thresholds and any scoring weights live in a datasheet; the first
      dispute read is an agreement one party ends against the other's
      standing; hooks travel the existing goal-thread path; a polity's
      governance is chosen by ranked preference over its members (Processes
      12).

      Links: area World generation · draws on 1025, processes.12, method.10 · summary A place's tier counts living residents; dwellings are places linked to map sites; thresholds in a datasheet; first dispute an agreement ended against standing.

1028. **Layers may come from `.vox` files as well as TOML.** Ruled by Mark on
      2026-10-30 (Mark: "Allow .vox too"), approving the `dot_vox` dependency,
      the appearance lane's other forks taken as drafted. The reading: layers
      are authored as TOML box fills or loaded from `.vox` files (Vengi,
      MagicaVoxel); the preview follows whichever board is active; packs bind
      item keys to layers; with the sim on the body draws as today, the look
      only sim-off for now; the words class, recipe and layer go to a naming
      round.

      Links: area The VTT · draws on 1019, 1022, bodies.2 · summary Layers come from TOML box fills or .vox files; the preview follows the active board; packs bind items to layers; sim-on bodies draw as today.

1029. **A campaign's edition is chosen at its founding.** Ruled by Mark on
      2026-10-30 (Mark: "Field in campaign founding"), offered against a DM
      action that locks once a sheet exists and a choice per character. With
      1030 he named what founding chooses: "the ruleset and other worldgen
      configs (or using preexisting worlds/applying a new ruleset to a
      preexisting world)". The reading: the edition is a field of the
      campaign's founding, beside its lead and world configuration, and a
      campaign may be founded over an existing world under a new ruleset;
      those who join, join a world under those rules.

      Links: area The VTT · draws on 1014, 1018, schema.8 · summary A campaign's edition is a founding field beside its lead and world configuration; an existing world may be founded under a new ruleset.

1030. **Roles are seats people trade; a table may have many GMs or none.**
      Ruled by Mark on 2026-10-30, answering who drafts a character (Mark:
      "1 for that reason"): "the separation between dungeon/gamemaster and
      player should not be so severe ... gms can make and play player-faction
      aligned characters along with npcs, players can make their characters
      and can take over hosting duties if needed, and the goal should be to
      allow people to trade off of those responsibilities by sharing the
      worldstate and changing authorities related to these roles. it should
      even be possible for six people to all be gm, and for eight to all be
      players relying on a prewritten campaign, with one choosing to become a
      gm when needed to resolve an issue. the goal of including scripting,
      dialogue, worldgen, the equivalent of radiant quests and nemeses and
      procedurally generated narrative cues grounded in the simmed world, is
      to make it possible to play in a sandbox fashion without a gm
      worldbuilding and cultivating the experience." The reading: the seat
      that controls a body drafts its character; GM, player and host are
      authorities over a shared world state that people hold, trade and
      share, any number at once or none; play without a GM is a goal, carried
      by the sim's generated cues.

      Links: area The VTT · draws on 153, 156, vtt.2, vtt.4, vtt.5 · summary GM, player and host are tradeable authorities over shared world state, any number or none; the body's controller drafts its character; GM-less sandbox play is a goal.

1031. **Pathfinder 2e's slice takes all forty backgrounds.** Ruled by Mark on
      2026-10-30 (Mark: "All 40"), offered against ten chosen for skill
      coverage. The reading: 1021's slice grows to every Player Core
      background and the skill feats they grant, about a hundred records.

      Links: area The VTT · draws on 1021 · summary The PF2e slice takes all forty backgrounds and their skill feats, about a hundred records.

1032. **The ladder's first readings settle residents, groups and founding
      events.** Ruled by Mark on 2026-10-30 (Mark: "All as recommended"), the
      region lane's forks after its first readings. The reading: a place's
      residents are its tenants and their kin with no home of their own on the
      same site; founding-door entries write sim events, so every product can
      read why a place exists; founders become kin only through births, the
      population model's work; a group is a polity once its constitution
      names members, otherwise a faction; a quick-start world's tenancies form
      by the founding tick's own commands; loading a quick-start world in the
      app gets an owner before the population model lands.

      Links: area World generation · draws on 1023, 1024, 1027, world.10 · summary Residents are tenants and their homeless kin; founding entries write sim events; kin come from births; a polity's constitution names members.

1033. **Looks face four ways; PF2e founds, names lores and waits for
      languages.** Ruled by Mark on 2026-10-30 (Mark: "All as recommended"),
      the appearance and rules lanes' forks after their first builds. The
      reading: each look bakes four true facings; the classes keep their
      varied looks; turn entries are labelled by the character's name; the
      founding panel picks a campaign's edition, re-founding an existing
      world under a new ruleset being a later flow; lores the book leaves to
      the player are written in as text; Player Core's nine terrain lores
      stand; languages come with the equipment increment.

      Links: area The VTT · draws on 1022, 1028, 1029, 1031 · summary Looks bake four facings and keep varied classes; turns label by name; the founding panel picks the edition; player-named lores are text.

1034. **Pathfinder 2e play: arms and armour, one-cell wounds, Stride moves
      the body.** Ruled by Mark on 2026-10-30, each as recommended. Gear
      (Mark: "All arms and armour + kit gear"): every level-0 Player Core
      weapon, armour and shield plus the gear the Fighter and Rogue kits and
      the adventurer's pack name, about 110 records, bought with 15 gp or
      taken as a class kit. Wounds (Mark: "One cell each"): a critical hit
      taken and each drop to dying wound one cell of a part the sim draws.
      Stride (Mark: "Stride carries the move"): one transaction carries the
      native cell batch, checked against Speed at the campaign's
      cells-per-square, and spends an action; where the scale is unset, a
      GM's recorded ruling stands in.

      Links: area The VTT · draws on 1020, 1021, 1026, vtt.5 · summary PF2e gear is all level-0 arms and armour plus kit gear, by gold or kit; crits and dying wound one cell; Stride carries the native move.

1035. **A look is derived in layers; items first; packs carry style until
      culture does.** Ruled by Mark on 2026-10-30 (Mark: "Derived in
      layers", "Items", "Packs now, culture later"), after he asked how
      items, character models and entities should be procedurally generated
      and varied. Offered against authored classes with seeded variation and
      looks from sim data alone. The reading: a look is derived from what a
      thing is, outward: shape from the body's part tree or the item's sort,
      palette from material, finish from make, style from a pack today and
      from culture once culture is built, wear from condition, variation from
      the seed within its lineage's bounds, and the creator's overrides last;
      the same derivation runs sim-on and sim-off, sim-off with fewer layers.
      Items are generated first.

      Links: area The VTT · draws on 1012, 1019, 1022, 1028, bodies.6, bodies.11, processes.13, schema.7 · summary A look is derived in layers (shape, material, make, style, condition, seed, overrides), sim-on or off; items first; packs carry style until culture.

1036. **Isocosm is the VTT; the sim is Cosmograph; the products are peers.**
      Ruled by Mark on 2026-10-30, after "Isometry" was found taken and
      released by another project: "i suspect that isocosm and mesocosm,
      being similar, maybe should just be peer names. could the name
      cosmograph be the name of the sim instead? then isocosm could slot in as
      the vtt's name. and no need for colons any more", then "we needn't make
      the repo `cosmograph`; the original thing was the vtt, and since that's
      now `isocosm`, let's just leave that as is", and from a round of
      alternatives (entelecheia, imago mundi, cosmogon), "Cosmograph". The
      reading: the products are Isocosm (the VTT), Mesocosm and Eponym, peers
      without a family prefix; the shared sim is Cosmograph; the repository
      stays `isocosm`. Cosmograph was chosen knowing a commercial
      graph-visualization product shares the name and Profantasy's
      Cosmographer Pro sits one word away in tabletop mapping.

      Links: area Naming · replaces naming.1 · summary The products are Isocosm (the VTT), Mesocosm and Eponym as peers; the sim is Cosmograph; the repository stays isocosm.

1037. **The tree follows the names now.** Ruled by Mark on 2026-10-30
      ("Everything now", then "isocosm-native", "Keep both", "Rename paths
      too"). The reading: the sim crates become `cosmograph`,
      `cosmograph-host` and `cosmograph-overlay` (folder `shared/cosmograph`);
      the VTT's `isometry-*` crates become `isocosm-*`; its adapter over the
      sim, `isocosm-vtt`, becomes `isocosm-native`; `isonetry` and `isomere`
      keep their names; visible strings say Isocosm; the `Code/testing/isometry`
      and `worktrees/isometry-*` paths are renamed and their citations fixed.
      `cosmograph` 0.0.1 and `isocosm` 0.0.2 were published to claim the names.

      Links: area Naming · draws on 1036 · summary Sim crates become cosmograph*, the VTT's isometry-* become isocosm-*, isocosm-vtt becomes isocosm-native; isonetry and isomere stay; paths and visible strings follow.

1038. **Cosmograph gets a standalone world-graph viewer.** Ruled by Mark on
      2026-10-31, from his wish for "a lil standalone that render[s] the world
      data and advance/reverse time in a style reminiscent of the graph
      visualization tool, but for our procedurally generated worlds, from the
      regions to the entities" (Mark: "Standalone over the endpoint", "Scrub,
      then branch", "Both"). The reading: a small native app, a Graphshell
      client of the sim host's inspection endpoint (Host 2), runs on a seed or
      a save and draws the world as one zoomable graph (regions and sites as
      clusters with route edges, opening into populations, entities of note,
      kin and relations, factions and polities, far nodes aggregates) under
      filters that narrow it by domain (geography, ecology, society,
      polities); time advances, and scrubbing back replays to a past tick
      without changing it, from which a branch can be forked (Record 4).

      Links: area Host and network · draws on 1036, host.2, record.4, processes.6, naming.3 · summary A standalone Graphshell client draws the world as one zoomable graph under domain filters; scrubbing replays the past and any tick can be branched.

1039. **Runtime and stored names follow the rename, with a version bump.**
      Ruled by Mark on 2026-10-31 (Mark: "Rename both", "Rename with a
      version bump"), offered against keeping them. The reading: the VTT's
      `ISOMETRY_*` variables become `ISOCOSM_*` and the sim's `ISOCOSM_*`
      become `COSMOGRAPH_*`, captures and log tags following, old harness
      scripts kept as history; the persisted and wire identifiers that still
      say isometry (chronicle vessel, store slots, seed domains, effect keys,
      schema ids, the session ALPN, the pack manifest file, the preferences
      directory) are renamed together under one version bump, old saves read
      through a one-time migration.

      Links: area Naming · draws on 1036, 1037, schema.9 · summary Env vars become ISOCOSM_* (VTT) and COSMOGRAPH_* (sim); persisted and wire identifiers are renamed under one version bump with a one-time migration.

1040. **Stored names migrate at load; the chronicle moves to v1; "nemeses"
      means relation-based tracking.** Ruled by Mark on 2026-10-31 (Mark: "At
      load", "Bump the chronicle to v1", "One per container"). The reading:
      renamed stored names are read new-then-old and written new, the old
      removed after the first good write, each renamed container's version
      moving once (session 24, checkpoint format 3, active effects 2); the
      shared chronicle format moves to v1 for every game, its fixtures
      regenerated. On 1030's "radiant quests and nemeses", asked whether the
      Eponym guardrail against rival-hierarchy promotion (patented to August
      2036) should limit it: "Ah, i did not mean rival-hierarchy. Just
      relation-based entity tracking. They do not own the world nemesis, even
      if they have patented the nemesis system." So nemeses are entities
      tracked by their relations to the players (remembered enmity and its
      history), and the guardrail is untouched.

      Links: area Naming · draws on 1030, 1039, schema.9 · summary Renamed stored names migrate at load, one version step per container; the chronicle moves to v1; 1030's nemeses are relation-based entity tracking, not rival-hierarchy promotion.

1041. **Quick-start worlds are founded from a picker; the population model is
      E4's.** Ruled by Mark on 2026-10-30 (Mark: "Picker + packs from disk",
      "Build on E4 when it lands"), the region lane's forks after its second
      increment. The reading: the founding panel lists the quick-start worlds
      the sim carries and those content packs offer from disk, and founds one
      in a single step; the region lane builds the sapient-lineage slice on
      the sim execution plan's E4 when it lands, not a model of its own, and
      the tier thresholds stay provisional until then.

      Links: area World generation · draws on 1023, 1032 · summary A picker founds carried or pack-folder quick-start worlds in one step; the sapient-lineage population slice waits on E4.

1042. **Pathfinder 2e ships the Adventurer's Pack, not class kits; Stride gets
      an admitted-character bench.** Ruled by Mark on 2026-10-31 (Mark: "Pack
      now, kits wait", "Build an admitted-character bench", "Generic only").
      The reading: no ORC record carries the Fighter or Rogue kit, so taking
      a class kit is refused with that reason until a source is ruled, and
      the Adventurer's Pack (checked item by item) is bought with starting
      gold; Stride moves characters admitted through native table admission,
      a bench building them, and a body without that binding is refused with
      the reason; only the generic languages ship.

      Links: area The VTT · draws on 1016, 1034 · summary PF2e class kits wait for an ORC source; the Adventurer's Pack ships; Stride moves admitted characters on a new bench; generic languages only.

1043. **Table items carry open material, make and condition keys; make names
      its grade, its act and its maker.** Ruled by Mark on 2026-10-31 (Mark:
      "Open keys on table items"; on make, "1, 3 (with the maker listed). it
      would be nice if modifiers could be applied based on 3. and even if
      we're not deriving things from deep time, we should be able to use the
      sim's machinery to make stuff that builds on a ruleset's default
      items/spells/templates"). The reading: each table item may carry
      `material`, `make` and `condition` keys with no fixed vocabulary, a pack
      mapping them to palette, finish and wear, and with the sim on they are
      filled from the item's entity; make is a grade key together with the
      act that made it and its maker, and modifiers may follow from that act;
      the sim's making machinery should be usable at the table to make new
      things on a ruleset's default items, spells and templates, without deep
      time.

      Links: area The VTT · draws on 1012, 1035, processes.13 · summary Table items carry open material, make and condition keys; make is a grade plus the act and maker, modifiers following the act; the sim's making builds on ruleset defaults.

1044. **Homebrew is the sim's making, with Distillery's models optional;
      quick-start worlds author their own ground.** Ruled by Mark on
      2026-10-31. On 1043's "homebrew addition from the distillery" (Mark:
      "Both, models optional"): new items, spells and templates are composed
      by the sim's own making machinery on a ruleset's defaults, and the
      stack's ML capabilities through Burn (Mark: "i think of that as the
      stack's ML capabilities through burn": esp, Distillery and the other
      ports being extracted from mere) may propose names, descriptions and
      variants on top when a device runs them, each proposal checked and
      built by the sim, never taken unchecked. On the head-founding gate
      (875, 885), which refused a quick-start laid over a drawn world (Mark:
      "Quick-starts author their world"): a quick-start pack declares its own
      sites and routes, so its world is authored and passes the gate.

      Links: area The VTT · draws on 1023, 1043, processes.13 · summary Homebrew is composed by the sim's making on ruleset defaults, the stack's Burn ML optional and checked; quick-start packs author their own sites and routes.

1045. **A shared mechanism moves when its second consumer is planned, not
      when it arrives.** Ruled by Mark on 2026-10-31, in Mere (house-style
      D12, refining mere's Stack 1): "second host is planned", because
      "shows up doesn't happen, you know, until it's planned. And shows up
      also doesn't happen until the extraction happens. But then the
      extraction is gated on a second consumer, right? So it's just
      circular." Planned means a dated plan in the consumer's own repository
      names the adoption with a done-condition; the move is done when the
      code has moved and its first consumer builds on it, and the second
      consumer's adoption is that consumer's receipt. Mark asked that the
      same rule reach the wing ("Also isocosm Stack 7"): 332's stamp barrier
      moves into conatus once a second consumer plans it.

      Links: area Stack and engine · amends 332 · draws on mere:stack.1 · summary A shared mechanism moves once a second consumer plans it in its own dated plan; the move closes on the first consumer, the second's adoption is its receipt.

Two earlier decisions this record relies on without restating: the founding
record's five shared nouns, space, bodies, fields, time and provenance
(engine clause, narrowed 2026-08-05), and the place-graph plan's "adjacency
is derived from the landscape, never asserted" (§0, 2026-08-05), now
qualified by ruling 14.

## 1. Sections 1 to 11, archived

On 2026-10-22 the record's narrative sections, §1 (the derivation rule) to
§11 (phases and done-conditions), moved to
[their archive](archive_docs/2026-10-22/2026-09-18_wing_design_plan_sections.md),
section numbers kept so citations still resolve. What they held is read
elsewhere now: their rules in [the doctrine](doctrine.md); the readings Mark
accepted as dockets in cases 985 to 1012 above; the wing's phases in
[the founding record's §9](2026-07-30_games_wing_founding.md#9-the-wings-phases);
and their open questions in the plans that own them.
