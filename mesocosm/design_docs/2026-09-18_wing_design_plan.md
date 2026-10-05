# The wing as a simulator: design record and plan

**Date:** 2026-09-18

*Names, 2026-09-22 (wing design record, rulings 109 to 111): the sim and the family are Isocosm, the second-person game is Eponym (formerly Paredros), the tabletop is Isocosm: VTT, with Isometry retired as a product word and kept only as the plain technical prefix of its crates, and Mesocosm is unchanged. The rename landed on 2026-09-24; verbatim rulings, quotations and dated lines keep the old words as history (ruling 198).*

**Status, 2026-09-18:** design record, ruled through W1. W0 is ruled (rulings
1 to 34, with the founding record and the three product descriptions amended
to it); W1 is evaluated, ruled and applied for all three products. W2, the
sim's own plan, is next and is design work, not lanes.

**Status, 2026-09-22:** rulings run to 105. W2 is drafted as the
[sim plan](2026-09-22_sim_plan.md), a schema and definitions compiled from
this record, with the [session notes](2026-09-22_sim_design_session_notes.md)
beside it; the [readings docket](archive_docs/2026-09-24/2026-09-21_wing_readings_docket.md) holds
the readings the plan depends on, and no lane is open.

**Status, 2026-09-27:** rulings run to 380, and W5 is drafted as the
[Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md), its M0 and
M1 done. The sim plan's implementation
lane is open in `shared/isocosm`; ruling 113 sets what its background must
agree with, ruling 114 holds every game's ruleset to the same test, rulings
115 and 116 make competition a choice and most contests bloodless, and
rulings 117 to 119 let any belief be wrong, say what makes one take and
how that is weighed.

**Status, 2026-09-28:** rulings run to 384. The one-game hypothesis session
ruled one host carrying the three games as modes over one world save (381),
a default view across the wing, isometric with quarter turns, precluding no
other view (382), harmony, order and chaos as ecological states (383), and
divine awareness of the sim as rule-bending magic (384). The founding
record's amendment for 381 and the tabletop `CLAUDE.md`'s for 382 are
drafted in the
[session notes' §9.4](2026-09-22_sim_design_session_notes.md#94-drafts-waiting-on-marks-word)
and wait on Mark's word. Later the same session Mark gave it: both
amendments are applied (385, 386), the default's pitch is the 2:1 dimetric,
30° (387), and Mesocosm's opening view waits on CP1 comparing its
2026-09-05 direction with the default (388). Rulings run to 388. Mark then
opened the first gap, the spatial spine: designed now and sliced around
rulings 195 and 363 (389), planned in the place-graph engine plan rewritten
as its plan (390), with sites meeting through edge profiles (391) and
Isocosm owning the lift (392). Rulings run to 392. The spine's four
decisions followed: terrain models in Isocosm (393), the skeleton as
condition keys (394), square sites first (395) and a chunked lift (396),
closing its SP0. Rulings run to 396. SP1's brief then ruled borders on
routes (397), the map family as an optional part of Founding (398) and
scale-free draws (399). Rulings run to 399. SP1 landed at `6f25a89`, and
SP2's forks followed: a lift gives its skeleton back exactly where it can
(400), the interior is a Coons patch (401), a lift returns columns per chunk
(402), and its materials are world-local ids (403). Rulings run to 403.

**Owns:** the design of the games wing as three tiers, a simulator, a stack
and three game overlays; the rules that decide which tier a thing belongs
to; the method by which claims reach rulings and receipts reach plans; and
the order in which the wing's existing plans are re-read, kept, rewritten or
retired against this record.

**Method annotation, 2026-09-27:** Mark restated the question, ruling and
verification method and invited improvements. The preserved method and
explicitly unruled refinements are in the [session notes,
§8.2](2026-09-22_sim_design_session_notes.md#82-the-method-as-practised).

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

**Status, 2026-09-28, composition refinement:** rulings run to 407. At Mark's
"Let's document!", the current architecture and the construction, magic,
learning and condition-carryover contracts below were recorded. This is a
documentation pass; SP2's landed receipt and other active work are unchanged.

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
The [body contract's current refinement](2026-07-31_wing_phenotype_contract_plan.md#current-contract-refinement-2026-09-28)
separates subject, incarnation, recipe, body revision and projection.

Capability is what current construction and circumstances support; repertoire
is the procedures an entity can attempt; proficiency is how well it performs
them. Maturation and practice can change these separately. Perception, belief,
self-belief and tenets inform choices; actual circumstances resolve effects.
The [sim plan](2026-09-22_sim_plan.md#25-capabilities-and-knowledge) owns these
distinctions. Material and conceptual operations share composition machinery,
with mundane and magical meanings supplied by the world's rules (407).

Magic composes fundamental effects with strongly escalating energy costs,
and its scripts may read world conditions and other generators' accepted
outputs (404). Rulesets categorize those effects and supply distinctive
exemplars; generated spells can extend their catalogues. The
[functional generation plan](2026-09-09_functional_generation_plan.md#composition-design-2026-09-28)
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
    the ledger's way, and claim needs a real publish. **Superseded for the
    sim's name by rulings 109 and 110 (2026-09-22): the sim is Isocosm.** Where
    this record says "isotropy" it means the sim.
18. **A game's grid is a projection.** The sim's volume is cubic; a tile
    with three, four or six sides is a region of voxels its overlay lays
    over that volume, the way a circle is in any voxel game. Ruled
    2026-09-18 ("hex as projection").
19. **The web is first-class** for the whole wing. Ruled 2026-09-18; the
    limits in §4.5 are therefore every tier's limits. **Replaced the same
    day by ruling 29: desktop first-class, the web a supported tier with a
    stated floor.**
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
31. **W1's recommendations are accepted in full,** row by row as the
    evaluations document's tables list them: 30 keep, 22 rewrite,
    6 retire, 3 surfaced (counts corrected on application). Ruled 2026-09-18. The rewrites are lanes under W2 and W3, not
    this ruling.
32. **The sim's process definition is founded from Paredros's
    world-conditions schema,** its stop rule lifted, with Law A's record
    fields added; Mesocosm's `ProcessDef` shapes become expressions in
    it. Ruled 2026-09-18.
33. **The founding record and Mesocosm's CLAUDE.md are amended** to this
    record: the schedule-and-verbs clause and the platform-first line.
    Drafts are in §9.11; the edits are Mark's to commit. Ruled
    2026-09-18.
34. **dramatis absorbs paredros-identity under W3:** its types move to
    the stack's trust plane and the sim's provenance noun, and Paredros
    consumes them back. Ruled 2026-09-18.
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
39. **Kingdom is class, and scale is orthogonal to it.** Mark, 2026-09-18:
    beside plant, animal and fungus, "playing as germs (micro: still at
    animal scale, perhaps like a paint that can spread, inhabit a creature,
    might die or thrive from being outside, spreads depending on
    substrate, a bit like being a plant but in a creature, and decomposes
    corpses), and huge creatures (macro) existing too. I actually love the
    notion of the lion turtles from Avatar." Consequences in §3.2.1.
40. **The licence rule for rulesets:** "As long as someone won't sue me or
    hate on me for putting the ruleset in my game, I'm happy to include
    it." Ruled 2026-09-18. Applied the same day by reading the licences
    (§5.1): Daggerheart's is a no, Lancer's is a yes with attribution,
    ICON and CAIN have no licence for implementation and are a question to
    Massif Press.
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
43. **How a divine thing is present, and how it ends.** Mark, 2026-09-18:
    "something divine could have an omnipresent instance, where there is
    one, and it's the same anywhere and everywhere. It could also
    participate in a cycle of reincarnation/recreation, destroyed only
    through prophesied conditions related to its ascent. So maybe you get
    the chain of avatars, maybe you get the one true god of lightning,
    maybe you get a worldtree that the world depends on and likewise."
44. **The quality of divinity is the quality of the journey.** Mark,
    2026-09-18: "perhaps the quality of divinity you can attain is related
    to your journey. If you fulfill the most difficult conditions for
    acquiring each glyph, you can reach tiers of godhood, like one
    immortal but vulnerable life (demigod), reincarnation on death
    (avatar) and immortality otherwise (avatar+), straight up unable to
    be killed immortality (greater divinity)."
45. **Provenance and identity are two axes.** Mark, 2026-09-18: "I agree
    with the provenance and identity axes." Provenance is born of a
    lineage, made by a maker, or intrinsic to the world; identity is
    unnamed, named and sapient, or factional. Constructs are the
    made-by-a-maker kind, the divine the intrinsic kind, and neither is a
    rung of the identity ladder. For the second identity level's word he
    finds sophont and denizen both good; the choice is still his.
    **Settled by ruling 200:** sophont is the term of art for a sapient
    entity, and a denizen is a named entity; every sophont is a denizen.
46. **Grades per glyph by the form sacrificed.** Mark, 2026-09-18: "I was
    just thinking of like assigning a point tier to what form of glyph is
    sacrificed for ascension. Item vs technique/skill/ability vs embodied
    trait. So depending on if you represent, or learned, or acquired the
    gnostic knowledge of the world in the glyphs, you get more leverage
    out of the ascension, grades per glyph."
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
48. **Divine places, placement, rooms and alignment.** Mark, 2026-09-18:
    "For a place, all the glyphs existing there is enough to make it a
    divine environment; a glyph being in a place has an influence on the
    place, so how durable the glyph's placement is matters, along with the
    quality of the placement (when we get to buildings and rooms, we
    should be able to evaluate the type and impressiveness of a room like
    rimworld? And an alignment system, where things can relate to the
    glyphs/symbols/factions/polities positively and/or negatively
    depending on tenets associated with effects?)"
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
50. **A tenet.** Mark, 2026-09-18: "A tenet ought to be that relation:
    that process yields that effect, and here's our opinion of that, and
    the trust people have in it for their survival/fulfillment
    (expectations: maybe people come to expect certain outcomes depending
    on how things turned out in the past, and that's a cohort opinion
    with variance possible? Idk. Feels a little complex, and we're missing
    attitude towards how things happened)".
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
52. **The referent by tier, and impact against frequency.** Mark,
    2026-09-18: "choosing a referent is a privilege of the greater
    divinities and greater avatars, but even they are subject to the
    frequency of process constraint. However, a rare process may be
    mitigated if it is high impact... like if the outcome of the effect
    being applied is unprecedented, or causes something significant to
    happen."
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
54. **Place.** Mark, 2026-09-18: "a critter has habitats, habits,
    memories, but not really a home that it possesses except through
    presence and defense. Even just being named is enough to be able to
    'have' a home, because that is a feature of sapience, but that's a
    one sided assertion from the sapient to the sentient; otherwise,
    you're just living where you can, as a critter." And the sentence
    that states it: "a human can give a dog a home, but they can't make
    the dog own the home."
55. **One memory, graded, over one history.** Asked on 2026-09-18 whether
    a critter's memory of habit and habitat and a sophont's memory of
    events are two things or one at different ends, Mark ruled "the
    latter" (2026-09-19).
56. **Standing is three things the record already has.** A sophont's
    alignment, derived from its acts (ruling 51); its reputation, the
    reach of its deeds among those who know of it (§3.4); and its rank,
    which a polity asserts: a title, an office, a membership. Mark,
    2026-09-19: "Yeah, sounds about right."
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
60. **Driving is Paredros; directing composes on top of it.** Mark,
    2026-09-19: "Driving is paredros. You're one sophont, with
    relationships sure, but you are one. I'm just saying you can give
    directives as that sophont to your friends and that can work a bit
    like directing mesocosm's critters, with an opinion modifier for
    command/request efficacy? So the systems compose, it seems to me."
61. **Death is final across the board; each game has its own trick.**
    Mark, 2026-09-19: "death is final across the board, but mesocosm's
    trick is that you have extra lives because of your cohort, paredros's
    is that you do too, but that those lives are even less fungible
    because no individual can really 1:1 replace another unless they're
    a clone. And isometry, death is understood to be a state that could
    almost be defeated if only we know more of the sciences and magics...
    revivify, resurrection, wishes..."
62. **The significant dead go to the planes after life, and summoning
    them back is a costly possibility.** Mark, 2026-09-19: "things that
    die that are significant should go to the planes after life, right?
    So being summoned back from there sorta makes sense as a possible,
    costly thing to do." And: "Let's keep going down. Let's plan it all."
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
65. **A polity without support does nothing; it dies only when people agree
    it has.** Mark, 2026-09-20: "when a polity lacks support to do its
    operations, then it essentially does nothing. But they only die when
    people agree they do. Perhaps because their political context changed,
    or because everyone who knew about it died, or the methods, goals,
    and/or means became pointless". **Extended by ruling 136:** a polity
    dies by either route, its own dissolution or nothing holding it any
    more.
66. **Any means may stand behind a constitution; short of death, a polity's
    condition may change automatically.** Asked on 2026-09-20 what stands
    behind a constitution in the sim, "force, faith, money, habit, a god?",
    Mark: "Any of 'em". And, on this record's first reading of collapse
    (corrected by ruling 65): "I would be ok with a polity automatically
    becoming suppressed or inactive or superseded or subordinated to a
    faction."
67. **Polities under no host are a faction among themselves.** Put to Mark
    on 2026-09-20 as a corollary of rulings 63 and 64: "polities dealing
    with each other under no host have only consent, so among themselves
    they are a faction. A treaty is then Paredros's standing agreement at
    scale, until an empire, a church or a federation hosts them." Mark:
    "Also i agree with this."
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
69. **What is of note persists; the rest is regenerated. "Of note" is a tier
    between the soup and legend, for every kind of thing.** Asked what makes
    a location a place and what keeps it the same place, Mark, 2026-09-20:
    "I feel like if something of note happens there, then it persists;
    otherwise it can be regenerated from the same basic facts. Note that
    denizen is now a term for an entity of note. There should be similar
    terms for locations, possibly more..." And: "Like, a tier between
    primordial sim soup things generate from and legendary things. Just
    stuff of note".
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
73. **A world has one shape, any shape.** Mark, 2026-09-21, on ruling 72's
    "any shape works": "i mean a world could be any shape, but just one. Not
    like switch between them. I mean to enable strange scenarios like a
    world resting on a critter".
74. **Nesting is one composable, expandable mechanism; world, region, area
    and battlemap are its defaults.** Asked whether the nesting of maps is
    one mechanism repeated or a fixed set of levels, Mark, 2026-09-21: "I
    think it would be cool if the mechanism was composable and expandable
    instead of the order and tiering of the nesting being predetermined. But
    yeah, those are good defaults." **Qualified by ruling 92:** the four
    are the VTT overlay's default scopes, not levels the sim hardcodes.
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
76. **The interoperation question was about the layer underneath the sim;
    and total componentisation is worth costing.** Mark, 2026-09-21, on
    §5.2's reading of his question: "Not everywhere, i was thinking like the
    layer underneath the sim, like w/rt our existing work, armillary, places
    where we instantiated that boundary for future modularity (plugins,
    expansion packs, mods, themes), and how the total stack resolves
    foreground -> background -> infrastructure." And then: "Architecturally,
    that is an interesting proposal. How bad would the total
    componetization's runtime cost be?"
77. **Two bindings, and the best capabilities each platform has, if that
    costs nothing on mobile or the web.** Mark, 2026-09-21, on §5.2's fork:
    "The two bindings does seem appropriate, too, though. Honestly if there
    is no cost on mobile or the web, i could see myself preferring to use
    the best capabilities available to improve performance".
78. **A clean boundary either way.** Mark, 2026-09-21: "there are perhaps
    parts that would be good to componentize and parts that might not be. So
    further optimization is possible. But the lack of a clear boundary would
    bite in the future, whereas performance will generally improve with
    newer hardware. So i favor a clean boundary either way. Would total
    componetization cause us to have ridiculous workarounds and inefficient
    methods, or does it buy architectural ergonomics that are as valuable as
    the performance the clean boundary costs?"
79. **A polity has its own alignment from its acts, as factions and
    individuals do.** On docket D8, Mark, 2026-09-21: "i think the idea is
    that you want to let the polity determine what it stands for by
    collective/hierarchal decision and actions, but also to compare that
    against the alignments of the factions, individuals, which also takes
    into account the acts of those entities. between the two of them, the
    second is really what the sim's other entity tiers provides, and the
    first is a repeat of that theme. it's more accurate to say: just like
    factions and individuals, a polity has its own alignment from its acts."
80. **"Of note" is literal: a note on the thing, made by the sim as a player
    would make one.** On docket D10, Mark, 2026-09-21: "i think of it as
    like literally a note on the site/item/entity. the sim just makes notes
    in the way a player would too, generating the note from the event/thing
    that makes it 'of note.'" On D19 and D20: "idk!", so both stay held.
    **Both since ruled: D19 by ruling 154, D20 by ruling 152.**
81. **Asserting a constitution is itself an act, the definitive one.** On
    ruling 79, Mark, 2026-09-21: "isn't asserting a constitution of a polity
    itself an act? indeed, a definitive one."
82. **The note is the impresa record, with a freeform text field beside the
    generated details.** Mark, 2026-09-21: "i'm alright with the impresa
    record! ah, but... maybe a lil markdown/djot/gemtxt/txt-formatted
    (sounds like a setting, i guess i'd prefer djot) field, wysiwyg for just
    freeform notes? if we can enumerate the generated details, sure, lovely,
    but something simple, accessible, and interpretable is also nice."
83. **The rest of the docket is accepted.** Mark, 2026-09-21: "also, i
    accept the rest." That is docket items D1 to D7, D9 and D11 to D18 as
    suggested; each is marked accepted where it sits in §3 and §5, and D19
    and D20 stay held.
84. **Information spread is to be modelled, cheaply and distributionally,
    with how a person learned a thing resolved on demand.** Asked what a
    thing knows, Mark, 2026-09-21: "didn't we want to model information
    spread? is there a way to do that cheaply and distributionally, so that
    we can resolve how a person learned a thing without mapping each step
    ahead of time necessarily? is that the best way?"
85. **For the impresa: a closed subset inside an open superset.** Mark,
    2026-09-21, on where the freeform text goes: "For impresa... closed
    subset, open superset?"
86. **The five points of how knowledge resolves are agreed.** Mark,
    2026-09-21: "And agreed on the five points!"
87. **A secret is valuable, leverage, kept by a taboo; telling it turns on
    relation, opinion and goals, and perhaps personality.** Mark,
    2026-09-21: "A secret is valuable... it is tremendous leverage, often,
    or there wouldn't be rules against its propagation. So for relation,
    opinion, and goals to factor in makes sense to me. I wonder about
    personality, too." And then: "Or perhaps not rule, but 'taboo'".
88. **Nesting is allocated as locations are generated, not fixed at
    founding.** Mark, 2026-09-21, on this record's list of what founding
    fixes: "Elaborate on the last two points. Shouldn't the nesting be
    dynamically allocated as locations are generated?"
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
90. **A world is realigned to another ruleset by a generative round that
    advances time.** Mark, 2026-09-21: "I would imagine that worlds can be
    converted to align with different rulesets by having them go through
    another generative round advancing time and realigning the world to a
    new ruleset (e.g. from hexploration to freeform piloting of a ship in a
    planet's orbit, or some other abstraction)..."
91. **How much history is the founder's choice; the generated timeline can
    be gone back into; the client is not overwhelmed and may zoom in and
    edit.** Mark, 2026-09-21: "How much history is a complexity and size
    issue. People pick how much to simulate dwarf fortress too, but even
    stopping the possibility of wiild events and catastrophes, we're also
    generating a timeline one could go back in time to prior worldstates
    with, right? Stands to reason we'd take care to not overwhelm the client
    with information, but kinda offer them the opportunity to zoom in and
    edit stuff".
92. **The named scopes must not hardcode Isometry's paradigm into the sim.**
    Mark, 2026-09-21, on this record's "Founding supplies only the
    vocabulary: which kinds of step exist, and the default scopes of world,
    region, area and battlemap": "I wonder about that, 'cause it sounds like
    hardcoding an isometry paradigm into the sim. Like does mesocosm or
    paredros need those distinctions? Does each game get a different
    tiering? Or does each tier mean different things to different games?"
93. **Deep time is the same sim.** Asked whether a world's past is run by
    the same sim at a coarser grain or written by a separate history
    generator, Mark, 2026-09-21: "Same sim".
94. **Value bears on scarcity, capability and material need; need is derived
    from alignment, acts and predilection.** Asked how things are exchanged
    and what value is, Mark, 2026-09-21: "Items, value, crafting, services,
    should bear relation to scarcity, capability, and material need. Need
    can be derived from a part of alignment, the stuff people do (acts) and
    are likely to do (predilection, personality?). But why do people take
    certain actions? Needs, like sims? And at the sim layer, how's that
    aggregate to give trade trends?"
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
97. **The materials of a world are its critters, across all the kingdoms.**
    Mark, 2026-09-22, on the question of the typology of kinds of nis:
    "whoa, so like, whatever critters exist, across all the kingdoms, those
    are the things stuff can be made of??? holy shit. that's way better than
    steel! it gives you a reason to learn the ecology, what stuff is,
    familiarize yourself with things to see what you can use... oh my god"
98. **The world is an entity made of stuff, a world-class entity, the basic
    macro kingdom; stuff can be made out of the world, even critters.**
    Mark, 2026-09-22, on inert matter: "although, presumably the world has
    to be made of stuff that isn't critter, like bedrock... but maybe we can
    resolve this in the same matter. i keep saying i want a world built on a
    critter to be possible. why don't we just make the world an entity made
    out of stuff, too? a special sort of entity, world-class. and that's the
    basic macro kingdom. then stuff can be made out of the world, even
    critters!"
99. **What kind of entity a world is stays open to the world.** Mark,
    2026-09-22: "maybe the world is inert, or sentient, or a sophont, or a
    god, or a turtle, or whatever. maybe it's flora, or fauna. maybe it's
    dead. who knows!"
100. **World is a kingdom; its scale is macro; meso and micro worlds are
     left open.** Mark, 2026-09-22: "world is a kingdom, the scale is macro.
     this leaves open meso (small moon? satellite? asteroid? ship?) and
     micro worlds :3"
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
102. **A world's magic, like its other characteristics, is suggested from
     the seed, configurable, and either a static profile or fluid over time
     under conditions.** Mark, 2026-09-22: "I think it ought to be a bit
     like ideology in rimworld. People should be able to set a static
     profile or let the world change with time (with conditions, like no
     magic or magic to start or no apocalypses or banned traits). But a
     suggested set from the seed is a fine default that could then be
     configured, much like the world's other characteristics."
103. **The sim knows the parts of an arc and lets entities pursue
     consequential goals for real reasons; the game makes the statement; the
     world is a bit like RimWorld's storyteller.** Mark, 2026-09-22: "Might
     even think of the world as a bit like rimworld's storyteller... I think
     the sim has to be aware of the constituent parts of an arc. I think the
     sim has to enable entities to determine and pursue consequential goals
     for themselves for real reasons, and in the sim's terms. But the game
     is what takes that vocab and makes a coherent game statement out of it.
     So both, with the second being what gives the first its shape."
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
105. **Option A is ruled; related worlds should share a context without
     merging, as branched worldlines or as neighbours in a celestial
     neighbourhood.** Mark, 2026-09-22: "A, then. And also... it would be
     nice if we could put two worlds in a shared context without entirely
     merging them... like two worldlines with a shared history that
     branched, or for entirely different worlds, they could become a
     neighboring world in the celestial neighborhood...? That way you could
     manage a set of related worlds".
106. **The docket's remaining items are accepted as suggested.** Mark,
     2026-09-22: "accept all as suggested." D21 to D31 and D33 accepted, D19
     and D20 kept held for W5 and W3. **Both since ruled: D19 by ruling
     154, D20 by ruling 152.**
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
108. **CLAUDE.md edits applied; Massif Press to be asked on Discord;
     isotropy and isostasy are not game names, so the title collisions are
     no conflict.** Mark, 2026-09-22: "you can apply the claude edits. i
     suppose i'll ask in the discord. as for isotropy and isostasy, i'm not
     trying to make those game names... so what's the conflict?"
109. **The second-person game is Eponym; Isometry names the sim and the
     family; the tabletop is Isometry: VTT.** Mark, 2026-09-22, at the end
     of a naming round (receipts in the naming ledger): "Eponym is pretty
     great! Accepted!" And: "I'm unsure isometry came after their game (when
     did we reserve the crate?)... and subtitling it would probably work to
     make it clear. Isometry: VTT is fine for the vtt, Isometry: Eponym,
     Isometry: Mesocosm, and Isometry itself can just be the sim instead of
     those two alternates. So you get isometry, isometry-vtt, mesocosm, and
     eponym." This supersedes ruling 17's name for the sim: isotropy is no
     longer the sim's name, and isostasy's status follows the same round.
     Checked the same day: the Isometry repository was bootstrapped
     2026-07-05 and the `isometry` crate reserved 2026-07-14; Dark Mode
     Games' Isometry announced its beta on 2026-07-06 after "roughly three
     months of work", so their project is the older by a few months and the
     crate is ours; neither holds a mark. The renames, Paredros to Eponym
     across crates and docs and the tabletop's package to `isometry-vtt`,
     are a lane and not an edit. Then, before the family word was banked:
     "Alternatively, isocosm!" Eponym stands; the family and sim word is
     between Isometry and Isocosm until Mark picks. **Superseded the same
     day by rulings 110 and 111.**
110. **The family and the sim are Isocosm; the tabletop is Isocosm: VTT; the
     rename is planned.** Mark, 2026-09-22: "Let's do it, isocosm, eponym,
     isocosm-vtt, etc. Let's plan the rename too". The reservations
     `eponym`, `isocosm` and `isocosm-vtt` were published to crates.io at
     0.0.1 the same day, and the rename is the [family rename
     plan](2026-09-22_family_rename_plan.md). Where this record says
     "isotropy" it means the sim, now Isocosm; where it says "Paredros" it
     means Eponym. (The clause first written here, "the tabletop keeps
     Isometry as its subtitle", was this record's inference and not Mark's
     word; ruling 111 settles it.)
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
113. **Similitude: the noted always run as individuals; the fungible agree
     in distribution over whatever a later process reads; what was watched
     is logged; exact agreement is a world setting, statistical the
     default.** Put to Mark on 2026-09-24, after the [aggregation
     research](2026-09-22_aggregation_research.md) found that its lossless
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
119. **Both: each entity's disposition shifts a world baseline, and the
     baseline is drawn from the world entity's disposition, a neutral
     median by default.** Put to Mark on 2026-09-24, following ruling 118:
     when the four pull different ways, what sets how much each counts?
     Three options were put: their disposition; the world's rules; or
     both, a baseline set at founding and each disposition shifting it.
     Mark: "Both. World baselines can be more like aggregate baselines
     drawn from the world entity disposition? Then a neutral median
     default for the sim is the world's normal way".
120. **The world's disposition seeds the founders only.** Put to Mark on
     2026-09-24, following ruling 119: is the world entity's disposition
     the baseline for every inhabitant's disposition, not just for how
     they weigh beliefs? Three options were put: every disposition, the
     world's pull continuing; founders only; or only belief. Mark:
     "Founders only." So the world's disposition seeds the first lineages;
     after that, dispositions are inherited and lived, and the world's
     pull fades.
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
125. **Noting and promotion belong to the world; collection and merging
     belong to the record.** Put to Mark on 2026-09-24, from the review's
     finding that the sim plan's table of world transitions listed noting,
     collection, promotion to legend and merging branches beside death and
     founding a polity: are they events in the world, or work done on the
     record about it? Three options were put: record work; world events;
     or a split, noting and promotion the world's, since something became
     remarkable, and collection and merging the record's. Mark: "Split."
126. **A fork founds a new world; a branch is the same world played apart
     and merged back.** Put to Mark on 2026-09-24: ruling 7 says worlds
     are "forkable and branchable"; should a fork be the new world that
     never merges back, and a branch the same world played apart and
     merged back by replay? Mark: "Yes, fork and branch." So a fork founds
     a world with its own history from that point, related by descent
     (D33), and a branch is ruling 104's play branch. D23's edit to the
     past, which may never rewrite, reads as a fork.
127. **Everything fades, at a rate the holder's memory sets; physical
     notes do not.** Put to Mark on 2026-09-24, from ruling 70's "the
     criteria for invalidation is crucial" and RimWorld's caution: does a
     note ever lapse on its own, or does a thing stay of note for as long
     as anything that matters reaches it? Three options were put: only
     when unreached; everything fades, unless renewed, legend never; or
     each kind its own rate. Mark: "Everything fades, moderated by, i
     guess in dnd terms, intelligence? Plus you can keep physical notes
     that don't fade, like a journal, notebook, diary, etc."
128. **Mesocosm's CLAUDE.md is amended for the sim's schema.** Put to Mark
     on 2026-09-24: its line "A portable profile is extracted after two
     real consumers, never declared in advance" contradicts the sim plan's
     S1. Mark: "Amend it." The line now carries the simulator's exception,
     the way the federation line does.
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
131. **A mind remembers what was new to it, mattered to it, was intense,
     or kept happening.** Put to Mark on 2026-09-24, closing §3.4.1's
     open item on what ruling 71's "little incident" measures: what makes
     a mind remember something in the first place, so it becomes of note?
     Four options were put, to pick all that should count: it was new; it
     mattered to them; it was intense; it kept happening. Mark picked all
     four. So a little incident is one that was none of these to anyone
     who saw it, and no mind keeps it.
132. **An unpractised skill decays slowly, to a floor it never drops
     below.** Put to Mark on 2026-09-24, from ruling 127's "everything
     fades" and the open question of whether a skill decays: does that
     include skills, the practised crafts a sophont gets better at? Three
     options were put: skills fade too, at the mind's memory rate; skills
     keep; or slowly, to a floor. Mark: "Slowly, to a floor." So a craft
     left unpractised slips toward a floor it never loses, and doing is
     kept better than knowing.
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
135. **Secession is the seceders' own act.** Put to Mark on 2026-09-24:
     can members leave a polity and take part of it with them, a town, a
     province, a guild chapter? Three options were put: only if it
     agrees; by their own act; or only what is theirs. Mark: "By their
     own act." So the seceders act as a faction and found their own
     polity, and the old one may accept it or contest it, the contest
     running like any other (rulings 115 and 116).
136. **A polity dies by either route: its own dissolution, or when nothing
     holds it any more.** Put to Mark on 2026-09-24, from §3.2.2's open
     "who must agree": a polity dies only when people agree it has (ruling
     65); who has to agree? Three options were put: its own way,
     dissolution decided like reform; whoever still holds it, dying when
     no member keeps it, no mind remembers it and no bearer records it; or
     either route. Mark: "Either route."
137. **The way of deciding suggests the means of enforcing by default and
     never binds them.** Put to Mark on 2026-09-24, from §3.2.2's open
     link between a polity's deciding act and its means: rule by buying
     withholds pay, rule by divining withholds rites, rule by fighting
     fights; is that link a default or a rule? Mark: "A default." So the
     generator suggests means from the way of deciding, and a polity may
     enforce by any means it has: a theocracy that fines, a merchant
     republic with an army.
138. **Deadlock is per act, not a condition.** Put to Mark on 2026-09-24,
     from §3.2.2's open "whether more conditions are wanted", with ruling
     133's deadlock as the candidate: is a deadlocked polity a fifth
     condition beside inactive, suppressed, superseded and subordinated?
     Three options were put: its own condition; inactive; or per act only.
     Mark: "Per act only." So deadlock blocks one act at a time, the
     polity stays active for everything its group does agree on, and the
     conditions stay four.
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
141. **A skill rises by doing, being taught, studying and breakthroughs.**
     Put to Mark on 2026-09-24, from §3.3.1's open "how a skill
     accumulates": what raises a skill? Four options were put, to pick all
     that should: doing it, harder or riskier work teaching more; being
     taught by someone more skilled; studying from bearers, books, manuals,
     a master's notes; and breakthroughs, a fey mood, a near-death, a
     masterwork. Mark picked all four.
142. **Techniques are both discovered and invented.** Put to Mark on
     2026-09-24, from the open "what a technique is made of": where do new
     techniques come from? Three options were put: invented, a skilled
     practitioner combining effects it knows into one nobody had; found in
     the world's canon; or both. Mark: "Both." So the canon holds known
     techniques to be found, and masters can invent new ones beyond it,
     which spread by teaching and by bearers.
143. **An item's improvement is limited by its maker's skill and its
     story, not its material.** Put to Mark on 2026-09-24, from the open
     "improving an item": what limits how far an item can be improved?
     Four options were put, to pick all that should: its material, the
     lineage's traits a ceiling; the maker's skill; its story, an item of
     note growing with what it has been through; or nothing. Mark picked
     the maker's skill and its story. So only a better maker takes an item
     further, an item of note grows with its deeds as well as its work,
     and its material is no ceiling.
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
145. **A sophont item acts through its wielder, by its own powers, and can
     take over.** Put to Mark on 2026-09-24, following ruling 144: how
     does a sophont item act in the world? Three options were put, to pick
     all that should be possible: through its wielder, directing whoever
     holds it, weighed by the wielder's opinion of it (ruling 60), the
     wielder free to refuse; by its own powers, as far as its body and
     magic allow; and taking over, a strong enough item possessing its
     wielder, the reverse of ruling 53's pilot possessing the mech. Mark
     picked all three.
146. **A sophont item is never owned.** Put to Mark on 2026-09-24: can a
     sophont item be owned? Three options were put: like any item; only by
     agreement; or never owned. Mark: "Never owned." So it can be held but
     never owned; it is a companion, and it can own things itself.
147. **A site and a paged chunk are independent.** Put to Mark on
     2026-09-24, from §3.7.1's open "how large a site is, in voxels and
     against the paged chunk": how does a site, the world map's unit,
     relate to a paged chunk, the store's unit? Two options were put:
     independent, or one site paging as one chunk. Mark: "Independent." So
     a site is the unit for generating and playing, a chunk the unit for
     paging, and a site spans as many chunks as its volume needs.
148. **When a location's ground changes or moves, its kind decides what
     it follows.** Put to Mark on 2026-09-24, from §3.7.1's open anchor of
     a place when the ground itself moves: eroded, flooded, carried off on
     a giant's back, what does the location follow? Four options were put:
     its site; its ground; its people and works; or its kind deciding.
     Mark: "Its kind decides." So a camp follows its people, a ruin its
     ground, a battlefield its site.
149. **Neighbouring worlds keep their own clocks and are synced at
     crossings.** Put to Mark on 2026-09-24, from §3.12's open question:
     do neighbouring worlds share one played clock? Three options were
     put: one clock; separate clocks, a crossing converting and a
     traveller perhaps arriving in the neighbour's past or future; or
     synced at crossings. Mark: "Synced at crossings." So each world keeps
     its own clock until something crosses, and the crossing brings the
     two into step, as realignment does.
150. **A planetary system can be a host.** Put to Mark on 2026-09-24,
     from ruling 100's open item and §3.12: can a planetary system be an
     entity in its own right, a host for its worlds? Three options were
     put: yes, a host; just relations; or the founder's choice. Mark:
     "Yes, a host." So a system is an entity above the world, a sun with a
     provenance and founding ruleset of its own, hosting its worlds as a
     polity hosts others.
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
152. **Outside the world, a player directs only who they play.** Put to
     Mark on 2026-09-24, from held item D20: outside the world, who may a
     player direct, given that inside it anyone can ask anyone through the
     entity they play, weighed by opinion under ruling 60? Three options
     were put: only who they play; also what is bound to them; or that
     plus edit mode directing whatever no player is bound to. Mark: "Only
     who they play." So everything else is asked in the world, through the
     entity a player plays, and D20's provider over `mere-capability`
     implements that in W3.
153. **Two players may direct the same entity.** Put to Mark on
     2026-09-24, co-op being a core taste in every vessel: can two players
     direct the same entity? Three options were put: yes, both, the
     entity weighing conflicting directives by its opinion of each; one at
     a time; or no. Mark: "Yes, both."
154. **D19's contract is ruled.** Put to Mark on 2026-09-24: D19's
     contract, a game submitting intents, the sim returning events and a
     read-only view of each tick, and outcomes a game settles coming back
     through the handoff, follows from rulings 104, 113 and 114; rule it
     now, or keep it held for W5? Mark: "Rule it." So §5.2's points 2 to 5
     stand as the contract's shape, and W5 designs the details against
     them.
155. **What a Mesocosm player plays depends on the lineage's traits.** Put
     to Mark on 2026-09-24, from ruling 152 against ruling 59's "direct one
     or a cohort of critters": what does a Mesocosm player play? Two
     options were put: one critter, the cohort following it in the world;
     or the cohort as one entity. Mark: "Your relation to other critters in
     your lineage depends on your traits. In the most extreme cases, other
     critters in your lineage can be ecological competitors or direct
     extensions of your own critter (fungus monocreature, swarmlike
     entities like germs, etc.)"
156. **A DM may take up any unclaimed entity to play, and may edit the
     world.** Put to Mark on 2026-09-24, from ruling 152: what can a DM do
     from outside the world? Three options were put: play any NPC; edit,
     not direct; or both. Mark: "Both."
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
159. **Mood is read; strain is kept.** Put to Mark on 2026-09-24: is mood
     kept, or read? Three options were put: read, never kept; kept as a
     level, as vigour is; or read, with strain kept. Mark: "Read, with kept
     strain." So mood is derived on demand from its sources, while strain
     builds as a kept level whenever mood stays low, and only rest or
     relief bleeds it off.
160. **Personality traits sit on the five factors.** Put to Mark on
     2026-09-24, from ruling 158 against ruling 37: how do personality
     traits stand to the five-factor disposition? Three options were put:
     traits on the factors; traits replacing them; or the factors as
     traits. Mark: "Traits on the factors." So the five factors stay the
     continuous temperament, and personality traits are discrete features
     grown on top, their seeds drawn from the factors.
161. **Past what it can bear, a mind stops heeding, acts out, or rises,
     and the break leaves a mark.** Put to Mark on 2026-09-24: what
     happens when strain passes what a mind can bear? Four options were
     put, to pick all that should: it stops heeding directives until it
     recovers; it acts out, a break drawn from its traits and situation;
     it may rise instead, a breakthrough such as the fey mood or a last
     stand; and it leaves a mark, seeding or feeding a personality trait.
     Mark picked all four.
162. **Breaking does not spread.** Put to Mark on 2026-09-24: does
     breaking spread from one mind to others? Two options were put: it
     spreads, a panic or a riot running through a crowd; or it doesn't.
     Mark: "It doesn't." So each mind breaks alone, and a riot is many
     minds breaking for the same reasons.
163. **Whether a break goes down or up turns on traits, the moment and
     chance.** Put to Mark on 2026-09-24: when strain breaks a mind, what
     decides whether it breaks down or rises? Three options were put, to
     pick all that should count: its traits; the moment, what is at stake
     and who depends on it; and chance, a seeded draw weighted by the rest.
     Mark picked all three.
164. **What a mind can bear is set by its traits, its support and what it
     has been through.** Put to Mark on 2026-09-24: what sets how much
     strain a mind can bear? Three options were put, to pick all that
     should: its traits; its support, company, faith, a home and what it
     believes in; and what it has been through, strain survived hardening
     it and past breaks scarring it. Mark picked all three.
165. **A culture is read from its people, and named when noted.** Put to
     Mark on 2026-09-24, opening language and culture: what is a culture
     in the sim? Three options were put: read from people, with no state
     of its own; a thing of its own, kept like a polity without a
     constitution; or read, and named and remembered once minds hold it as
     a thing. Mark: "Read, named when noted."
166. **Languages divide.** Put to Mark on 2026-09-24: do sophonts speak
     languages that can divide them? Three options were put: languages
     divide, understanding needing a shared tongue learned like a skill;
     names only, languages as the generated flavour of names; or no
     languages. Mark: "Languages divide." So telling, posing and exchange
     across tongues come harder, and interpreters matter.
167. **Tongues and cultures descend like lineages.** Put to Mark on
     2026-09-24: do tongues and cultures descend like lineages? Three
     options were put: like lineages, splitting into dialects when their
     speakers part and drifting over deep time; fixed per people; or
     generated with no descent. Mark: "Like lineages."
168. **Names come from the tongue, from tales, and from the namer.** Put to
     Mark on 2026-09-24: how are names made? Three options were put, to
     pick all that should: from the tongue, generated from a language's
     sounds and words; from tales, epithets earned by deeds and stories;
     and given by the namer, ruling 36's naming being a sapient's own act.
     Mark picked all three.
169. **Culture spreads by contact, prestige and imposition, and drifts
     apart out of touch.** Put to Mark on 2026-09-24: how does culture
     spread between peoples? Four options were put, to pick all that
     should: contact, neighbours growing alike faster the more they share;
     prestige, copying those admired or powerful; imposition by a polity's
     means; and drift apart when out of touch. Mark picked all four.
170. **Art and ritual are crafts, and any craft or pursuit can be
     politicized or ritualized.** Offered on 2026-09-24 as a next thread,
     "whether sophonts make art and hold rites, and what those do", Mark:
     "I view art and ritual as crafts. A craft/pursuit can be
     politicized/ritualized too. Let's proceed to tech".
171. **Technology is both a world's generated tree and what is known.** Put
     to Mark on 2026-09-24, opening technology: what is technology in the
     sim? Three options were put: what is known, read from the techniques
     and recipes minds and bearers hold, a tree emerging from their
     preconditions; an authored or generated tree peoples climb; or both,
     a world's generated tree setting what can be known and what is known
     read from minds and bearers. Mark: "Both." Put with it as a reading
     from rulings 129 and 142, unobjected: a technique no mind or bearer
     holds is lost, and can be found again.
172. **Ages are read and named, capped by the founder, can realign the
     world, and are the epochs of cultures and society.** Put to Mark on
     2026-09-24: does a world have ages of technology? Four options were
     put, to pick all that should hold: read and named, a reading of what
     the peoples of a time commonly know; world epochs, turning at the
     world's epoch boundaries; a founder's cap on how far technology can
     go; and ages that realign the world to a new scope. Mark picked the
     first, third and fourth and, in place of the second: "Ages are the
     epochs of cultures/society."
173. **Invention is driven by need, contact, temperament and mastery.** Put
     to Mark on 2026-09-24: what drives invention, a master going beyond
     the canon (ruling 142)? Four options were put, to pick all that
     should: need; contact, ideas combining when peoples meet; temperament,
     curious and open minds; and mastery, deep skill making breakthroughs
     possible. Mark picked all four.
174. **Mesocosm is the first game overlay.** Put to Mark on 2026-09-24,
     opening W5: which game goes first against the sim? Three options were
     put: Mesocosm, whose foregrounded critter and lineage the Isocosm core
     already runs and whose bench is the core's first host; Eponym; or the
     VTT. Mark: "Mesocosm."
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
176. **A Mesocosm player directs with priorities, places, stances and
     nudges.** Put to Mark on 2026-09-25: what does a Mesocosm player
     direct with? Four options were put, to pick all that should be in the
     vocabulary: priorities among needs and abilities; places to range,
     avoid and make home, the desire paths of ruling 59; stances, bold or
     cautious and contest, yield or share; and nudges, one-off suggestions
     the critter weighs. Mark picked all four. **How, by rulings 214 to
     216:** the player gives the nudge by click, and the other three grow
     from the nudges' history as desire paths.
177. **A critter follows its player's directing by its bond.** Put to Mark
     on 2026-09-25: does your own critter always follow your directing?
     Three options were put: within its body; when its needs allow; or by
     its bond, built from how well your orders have served it, as ruling
     60 weighs directives by opinion. Mark: "By its bond."
178. **Whether the bond carries across generations is a setting, seeded
     by default.** Put to Mark on 2026-09-25: when your critter dies and
     you take up the next of your lineage (ruling 61), does the bond carry
     over? Three options were put: it starts fresh; it is seeded by the
     lineage, inherited in seed form as personality traits are; or it is
     the lineage's, carrying whole. Mark: "All three should be options; my
     default would be seeded".
179. **A Mesocosm player chooses when to start, from the world's
     habitability for their critter onward.** Put to Mark on 2026-09-25:
     does the first played loop start in a world with a past, deep time
     first or a bare world first? Mark: "You may choose to start at the
     start of the world's habitability for your critter, or add as much
     time as you'd like".
180. **What a Mesocosm player sees is a mode: survival shows what the
     critter knows, creative shows the truth.** Put to Mark on 2026-09-25,
     from the playable ecology plan's open ruling 6: how much of the
     ecology's truth does a Mesocosm player see during live play, versus
     at the epoch review? Three options were put: what it knows; truth at
     the review; or truth anytime. Mark: "Creative vs survival mode, if you
     ask me. Both worthy and necessary, debug-wise."
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
183. **At a birth, the player keeps the parent by default.** Put to Mark on
     2026-09-25, from the playable ecology plan's open ruling 1: when your
     critter reproduces, do you keep playing the parent or move to the
     offspring? Three options were put: choose each time; parent by
     default; or offspring by default. Mark: "Parent by default." So the
     player keeps the parent unless they take the offspring, and the
     choice can be taken back, which is the placeholder the plan already
     built.
184. **Creative mode shows the truth and does not edit.** Put to Mark on
     2026-09-25: does creative mode (ruling 180) also let the player change
     the world? Two options were put: see and edit; or see only, editing
     belonging to the world editor (ruling 89). Mark: "See only."
185. **Eponym's co-op is each player living their own creature.** Put to
     Mark on 2026-09-25, opening Eponym's overlay: what does co-op look
     like, given that ruling 153 lets two players direct one entity? Three
     options were put: each their own; one life, shared; or both as a
     setting. Mark: "Each their own."
186. **At a death in Eponym, the player chooses who to become, among
     bonded companions.** Put to Mark on 2026-09-25: when the creature you
     live dies and a companion becomes the one you play, who decides which
     companion? Three options were put: the player chooses, among
     companions with a bond to the one who died; the closest bond; or
     whoever takes up the role by the world's rules. Mark: "The player
     chooses."
187. **Survival and creative modes hold for Eponym as for Mesocosm.** Put
     to Mark on 2026-09-25: do rulings 180 and 184 hold for Eponym? Two
     options were put: the same, creative also being where tag-in lives;
     or survival only. Mark: "The same."
188. **A VTT campaign may run with the sim off.** Put to Mark on
     2026-09-25, opening the VTT's overlay: can a campaign run with the sim
     switched off, as a plain board with no world beneath? Three options
     were put: always a world; sim off allowed; or a dial of how much the
     world moves on its own. Mark: "Sim off allowed."
189. **An uncalibrated ruleset may play in a debug or experimental mode,
     with a warning.** Put to Mark on 2026-09-25: what happens to a system
     plugin that cannot pass ruling 114's check? Three options were put:
     refused on a world; allowed and flagged; or the DM's choice. Mark:
     "We should maybe consider allowing it debug, or experimental (with a
     warning)".
     *Bounded 2026-10-03 by ruling 538:* calibration, and so this mode,
     applies in worlds the sim leads.
190. **An adventure pack whose requirements the world lacks waits, or the
     GM forces it.** Put to Mark on 2026-09-25: an adventure pack names
     what it needs from a world, a ruin, a faction at war, a missing heir;
     what if the world doesn't have it? Three options were put: the pack
     waits, dormant until the world meets it; the DM forces it, asserting
     its content and displacing generated content (ruling 89); or the
     world bends at founding. Mark: "Wait or the gm forces it".
191. **The VTT offers the sim's running arcs to the DM as suggested
     hooks.** Put to Mark on 2026-09-25: the sim knows the parts of the
     arcs it is running (ruling 103); does the VTT offer them to the DM?
     Three options were put: as suggested hooks, which the DM may take up,
     drop or reshape; on request; or not at all. Mark: "As suggested
     hooks."
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
193. **Moved modules are decomposed under the 600-line ceiling.** Mark,
     2026-09-25, while the Mesocosm overlay plan was being drafted: "Feel
     free to decompose them under the 600 Loc limit". So every family that
     moves into Isocosm is split as it goes, along seams the code already
     has.
194. **Directing is built only on Isocosm.** Put to Mark on 2026-09-25, the
     Mesocosm overlay plan's first decision: prototype directing on the
     current host over `mesocosm-core` now, or build it only on Isocosm
     once the first families have moved? Mark: "Only on Isocosm."
195. **The record moves ahead of places.** Put to Mark on 2026-09-25, the
     plan's second decision, the proposed order being matter and
     processes, bodies, places, lineages and the boundary, the record, and
     effects: keep it, or move the record ahead of places so replay and
     save are on Isocosm sooner? Mark: "Record earlier." So the order is
     matter and processes, bodies, the record, places, lineages and the
     boundary, then effects. *Confirmed from bodies onward 2026-10-02 by
     ruling 457.*
196. **The played slice plan retires into W5.** Put to Mark on 2026-09-25,
     the plan's third decision: retire the played slice plan into the
     overlay plan when M3 lands, or keep it as the host's own? Mark:
     "Retire into W5."
197. **The overlay contract is one shared crate, `isocosm-overlay`.** Put
     to Mark on 2026-09-25, opening M1: does the contract crate live as one
     shared crate, as Mesocosm's own first, or as a shared core plus
     Mesocosm's vocabulary? Mark: "One shared crate." And its name, from
     `isocosm-contract`, `wing-overlay` or `isocosm-overlay`: Mark:
     "isocosm-overlay". So it sits in `shared/`, game-neutral, with
     Mesocosm's directives as its first game's module and Eponym and the
     VTT to add theirs.
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
199. **The played slice plan's control is rewritten now, and the plan still
     retires into W5 at M3.** Put to Mark on 2026-09-25, from the pass's
     item 8: ruling 175 says the slice's direct control is rewritten, and
     ruling 196 retires the plan when M3 lands; does the plan still owe a
     rewrite before M3? Mark: "Yes, rewrite it now." So its control is
     rewritten for directing (rulings 175 to 178) ahead of M3, and at M3 the
     whole plan retires into the overlay plan.
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
203. **The tick stamp is a newtype over the sim's count.** Put to Mark on
     2026-09-25, the first of three forks from building M1: a newtype over
     `u64`, a plain `u64` as the sim uses, or an epoch and an offset, a
     structure the sim does not store? Mark: "Newtype over u64."
204. **Subscription derives from the attention set, whose type M1
     designs.** Put to Mark on 2026-09-25, the second fork: §5.2's point 4,
     in the contract's shape since ruling 154, has one attention set per
     player drive the collector, the foreground and the event stream, but
     the set has no type. Until it has one, does the contract carry
     explicit topic keys or a structured filter, or does M1 type the
     attention set? Mark: "Type the attention set." So M1 is not done
     until the attention set has its type and subscription derives from it.
205. **A PLACES directive names a node of the place graph.** Put to Mark on
     2026-09-25, the third fork: an opaque handle to a place-graph node
     (rulings 72, 147), a site, or raw coordinates? Mark: "Place-graph
     handle." Dev intents keep raw coordinates, since they reach the grid
     directly.
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
207. **The crowd runner keeps a histogram of exact states.** Put to Mark
     on 2026-09-25, with 206: a histogram of exact states, coarsened
     states, moment closure, or super-individuals? Mark: "Exact-state
     histogram." So members are grouped by their exact state as the rules
     read it, with random count draws each round, and the pairing draw is
     the only approximation; a field is coarsened only where the rules show
     it safe, and savings are reported in evaluations.
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
210. **A player may pin any pointable thing.** Put to Mark on 2026-09-25,
     typing the attention set (ruling 204): what can a player pin, which
     ruling 130 keeps on the player's account? Three options were put: any
     pointable thing, entities and places, or entities only. Mark: "Any
     pointable thing." So entities, places, items, lineages, factions and
     events, anything that can have an impresa, can be pinned.
211. **An attended group runs its noted members exactly and the rest as a
     crowd.** Put to Mark on 2026-09-25, with 210: when what a player cares
     about is a group, such as Mesocosm's lineage, does every member run
     exactly, grouped without loss, or do the noted run individually and
     the fungible as a crowd, like the rest of the world? Mark: "Noted
     exact, rest crowd." So caring for a group keeps it, its noted members
     run as individuals (ruling 113), and its fungible members run as a
     crowd within the world's tolerance.
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
215. **A click never warns.** Put to Mark on 2026-09-25, with 214: can a
     click warn as well as invite, by a second gesture, or does the critter
     read the pointing through its own sense of the thing? Mark: "One
     gesture only." So no gesture warns: the critter reads what it is shown
     by its own sense of it, and avoidance comes from its experience and its
     stance.
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
218. **Each reading's bound and the competition's definition sit in the
     world's rules.** Put to Mark on 2026-09-25, from the probe's deviation:
     it kept both beside the core's `Rules`, moving them in being a schema
     change; move both in now, or wait for S1? Mark: "Move both in now."
219. **The interpreter's meanings exist once.** Put to Mark on 2026-09-25:
     the probe's `aggregate.rs` restates the interpreter's query and effect
     meanings for counts of members, the core refusing counted shared
     writes by design; fold both into shared functions now, or during M2?
     Mark: "Fold now." So the crowd and the individual runner call the same
     code.
220. **S2 next: qualify an approximation, cut the exact runner's cost, and
     express the thirty shapes.** Put to Mark on 2026-09-25, to pick any: a
     cheaper, approximate pairing draw certified by the same check, so an
     approximation's error is measured; the staging clones and whole-world
     matter totals that make the exact runner quadratic; the thirty
     `ProcessDef` shapes in the process definition, S2's own
     done-condition; or pausing S2 for M2. Mark picked the first three.
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
222. **A break in a fight advantages or disadvantages; it decides
     nothing.** Put to Mark on 2026-09-25, with 221: in a fight, does a
     break down yield and a break up prevail, or does any break yield?
     Mark: "Breaking up advantages, breaking down disadvantages. No auto
     win or lose". So a break up gives its side an advantage in the rounds
     that follow and a break down a disadvantage, the direction turning on
     traits, the moment and a seeded draw (ruling 163), and neither ends the
     fight.
223. **A fight ends when a side yields on re-sizing or is spent, whichever
     comes first.** Put to Mark on 2026-09-25, with 222: if a break only
     shifts the advantage, what ends a fight: re-sizing, being spent, or
     whichever first? Mark: "Whichever first." So after each round both
     sides size each other up again with the new advantage, and one that
     now reads itself outmatched yields (ruling 116); or a side whose
     vigour, its reserve until vigour is in the ledger (ruling 123), runs
     out is spent.
224. **The tract rename writes the new word and still reads the old.** Put
     to Mark on 2026-09-25, opening ruling 157's rename lane beside Lane A:
     the body-site types are serialized, so what happens on disk: the code
     renamed with the wire keeping "site"; "tract" written and "site" still
     accepted on read; or a clean break, re-recording whatever carries the
     old word? Mark: "New wire, old accepted." So new data is written with
     tract, old data still loads through aliases, and hashes of newly
     written data change.
225. **A region has collapsed when one of its trophic levels is gone.** Put
     to Mark on 2026-09-25, from ruling 181's open item, what measures a
     collapse: in one region, its producers gone, any trophic level gone, or
     its living biomass below a floor? Mark: "A level gone." So a region has
     collapsed when producers, consumers or decomposers (ruling 39) are
     empty there.
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
227. **The core reads mood from needs now.** Put to Mark on 2026-09-25, as
     Lane A gives the core its first strain, from fights: should the core
     read mood from needs now, so hunger and wounds build strain too, or
     should strain come from fights only until S1 designs the mind's schema?
     Mark: "Needs now." So mood is read from a member's needs as the core has
     them, low reserve, hunger and starvation, and wounds once they exist;
     strain builds whenever that mood stays low (ruling 159), fights adding
     theirs; memory and situation, mood's other sources (ruling 158), wait
     for S1.
228. **A right click shows a ring of acts.** Put to Mark on 2026-09-25: how
     does a right click show its meanings (ruling 214): a ring of acts around
     the cursor, a cycle in place, or the critter's glance surfacing glyphs
     on the target? Mark: "A ring of acts." So the critter's possible acts
     toward the target appear around the cursor, and a click picks one.
229. **When a player's lineage ends, they pick another here or a new world,
     their call.** Put to Mark on 2026-09-25, from 226's "the world isn't
     done until your lineage is done and you need to pick a new one": pick
     another lineage in the same world, a new world, or either? Mark:
     "Either, player's call."
230. **A total collapse ends the world as a game, and it stays watchable.**
     Put to Mark on 2026-09-25, from 226's "or until the world's ecology is
     collapsed entirely", ruling 181 having had no collapse end play: after a
     total collapse, is the world done but watchable, or done and closed?
     Mark: "Done, but watchable." So ruling 181's play going on after a loss
     holds, and a total collapse is the one loss that ends the game while the
     world can still be watched.
231. **Eponym's and the VTT's overlays go side by side after Mesocosm's.**
     Put to Mark on 2026-09-25, from the §6 both new overlay plans share:
     which overlay is W5's second, Eponym, the VTT, or both side by side?
     Mark: "Side by side." So both proceed after Mesocosm's M3, each on its
     own plan.
232. **Eponym's blows are resolved in the foreground and handed back.** Put
     to Mark on 2026-09-25, from the Eponym overlay plan's §6 decision 1:
     Eponym's strike system resolves each blow geometrically and hands the
     harm back through the handoff in the sim's terms, calibrated to the
     sim's fight (ruling 114); or the sim's own contest runs at foreground
     fidelity, Eponym supplying the actuation alone? Mark: "Foreground hands
     back." So Eponym's handoff carries vigour drained and wounds to parts,
     which must pass the sim's invariants and agree with the sim's own fight
     in distribution (rulings 114, 123, 154), as the record's §3.8 has it.
233. **Eponym's motion and contact solver runs on the game side.** Put to
     Mark on 2026-09-25, from the same decision's second half: on the game
     side over the stack's conatus, the accepted transition for each tick
     crossing as the intent, or in the sim, fed the input frames? Mark:
     "Game side."
234. **Where a first Eponym life begins in time is the player's pick, with
     society the default.** Put to Mark on 2026-09-25, from decision 2: from
     the world's habitability for the creature onward, as ruling 179 gives
     Mesocosm, only once settlements stand, or the player's pick with the
     second as the default? Mark: "Player's pick, society default."
235. **How a first Eponym life begins is the player's call among three.**
     Put to Mark on 2026-09-25, from decision 3: a newly generated outsider
     arriving, a birth into one of the world's lineages, a named creature
     chosen among the drawn world's denizens, or any, the player's call?
     Mark: "Any, player's call."
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
237. **The exact runner's two remaining growing costs are cut.** Put to Mark
     on 2026-09-26: each feeding evaluation searching the whole population
     for a target, and each advance cloning the whole world; cut both, under
     the same rule as before, results byte-identical by the differential
     driver, or leave them? Mark: "Cut both."
238. **At an Eponym death with no bonded companion, what follows is the
     player's call.** Put to Mark on 2026-09-26, from the Eponym overlay
     plan's decision 3: an outsider arriving, the execution plan's F8
     fallback; the player's call, as ruling 229 gives Mesocosm; or the line
     ending, watchable? Mark: "Player's call." So the player takes up
     another life in that world, any of ruling 235's three, or goes to a new
     world. *Reading, not ruled:* going to a new world is how a line ends in
     one, so the plan needs no separate answer that lets it end.
239. **Isocosm absorbs Eponym's simulation, and driving is built only on
     Isocosm.** Put to Mark on 2026-09-26, from the plan's decision 4: with
     Isocosm absorbing `eponym-world`'s and `eponym-social`'s simulation as
     ruling 192 had it absorb `mesocosm-core`, in the plan's order (bodies,
     places, lives and rounds, standing and asks, knowledge, glyphs, then
     fights and technique), is driving built only on Isocosm, as ruling 194
     had Mesocosm's directing, or first over today's `Session`? Mark: "Only
     on Isocosm." So the absorption goes in that order, and driving has no
     prototype over `Session`.
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
241. **Posing is a telling's manner.** Put to Mark on 2026-09-26, from the
     Eponym overlay plan's decision 5: posing to intimidate, persuade or
     deceive, his own idea at ruling 117 and flagged since as a reading, is a
     telling's manner, an act of its own covering wordless display and bluff
     too, or left out of E1? Mark: "A telling's manner." So posing is how a
     claim is told, the fourth thing a hearer weighs (ruling 118), carried
     on the telling and not as an intent of its own; display and bluff in a
     contest stay sizing up (ruling 116).
242. **Eponym's E4 plays two peers.** Put to Mark on 2026-09-26, from
     decision 6: each player living their own creature being ruled (185),
     does E4's done-condition include two players over the session lane, or
     single-player first, as the founding plan parked real-time co-op
     netcode? Mark: "Two peers in E4."
243. **With the sim on, a battlemap is projected from the generated volume,
     the DM's map an edit over it.** Put to Mark on 2026-09-26, from the VTT
     overlay plan's decision 1: the DM's own map asserted over the site's
     volume (ruling 89), a projection of the generated volume in the game's
     grid (rulings 12, 18), or both, the map an edit over what was generated?
     Mark: "Both: an edit."
244. **Moves within a battlemap reach the sim.** Put to Mark on 2026-09-26,
     from the VTT plan's decision 2: only outcomes, assertions and travel,
     a token's position staying the table's, or moves within a battlemap
     too, as per-tick batches, so a character's place in the site follows
     its token? Mark: "Moves too." The contract stays coarse, batches per
     tick and never a call per token (§5.2 point 1).
245. **At a VTT table, sharing a character and the DM playing the unclaimed
     are on by default.** Put to Mark on 2026-09-26, from the VTT overlay
     plan's decision 3: two players directing one character (ruling 153) and
     the DM playing any unclaimed one (ruling 156), on by default, or each a
     campaign setting the DM chooses? Mark: "On by default."
246. **Downtime needs each player's yes.** Put to Mark on 2026-09-26, from
     decision 4: the players' consent to downtime (rulings 63, 104, 105)
     gathered as each player's yes, a majority, or the DM's declaration with
     a veto? Mark: "Each player's yes."
247. **The tape-drawn faction turn retires at V2.** Put to Mark on
     2026-09-26, from decision 5: with Isocosm absorbing
     `isometry-campaign`'s world as it did Mesocosm's and Eponym's (rulings
     192, 239), does the faction turn stay as a sim-off campaign's downtime,
     or retire entirely once the sim's factions run? Mark: "Retires at V2."
     So the absorption stands in the plan's §4 order and the faction turn
     retires. *Reading, not ruled:* a sim-off campaign's downtime is then the
     table's own, as the plain tabletop plays today (ruling 188).
248. **A sim-off campaign still writes its facts as notes.** Put to Mark on
     2026-09-26, from decision 7: does a campaign played with the sim off
     still write its facts as notes, so the sim can be switched on later over
     the same history, or does switching it on found a world from that point
     (ruling 126)? Mark: "Writes notes."
249. **The bench calibrates Pathfinder 2e first.** Put to Mark on
     2026-09-26, from the VTT overlay plan's decision 6: which ruleset the
     bench calibrates first against the sim's fight at the first tolerance
     (ruling 209), the 5e SRD content pack in the repository or Pathfinder
     2e's skeleton? Mark: "Pathfinder 2e first."
250. **An uncalibrated campaign warns when it opens and marks its
     receipts.** Put to Mark on 2026-09-26, with 249: what does the table see
     of ruling 189's debug or experimental mode, a warning when the campaign
     opens with every receipt and save marked uncalibrated, or a standing
     banner as well? Mark: "Warn at open, mark receipts."
251. **Where an incoming part attaches is its attachment.** Put to Mark on
     2026-09-26, from the tract rename lane (ruling 224), which left alone a
     sense of *site* that is neither the world map's cell nor the tract: the
     point where an incoming part attaches to a body in growth or a graft
     (`growth::attachment()`, "landing site"). Attachment, placement, or
     tract? Mark: "Attachment."
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
253. **Eponym's and the VTT's contract phases open now.** Put to Mark on
     2026-09-26, both new plans having every fork ruled and ruling 231
     having them proceed after Mesocosm's M3: open their contract phases, E1
     and V1, the `eponym/` and `vtt/` modules in `isocosm-overlay`, now, or
     hold until M3? Mark: "Open E1 and V1 now." So the RPG systems session
     builds both modules now, proving early that the contract carries three
     games; everything else in both plans still waits for M3 (231).
254. **The contract's shared shapes live once, in its core.** Put to Mark on
     2026-09-26, here and in the RPG systems session: E1 and V1 left
     `WorldPoint` in three game modules, the act key in three, and `Harm`,
     `Wound`, `WoundSeverity` and `PartHandle` in two, byte-identical; lift
     them into the crate's shared core, or leave the copies? Mark: "Lift to
     the core now", and in the RPG systems session, "I don't mind lifting
     either." Done there at 269ffc5: `point.rs`, `act_key.rs`, the VTT's
     `ActionKey` renamed, and `harm.rs` are core exports, and no game module
     defines its own.
255. **Raw receipts live out of tree; the repository keeps their summaries,
     sources and scripts.** Put to Mark on 2026-09-26, S2's checkpoint 2
     bringing 14 MB of raw JSON against a repository pack of 9 MB: commit
     them as they are, gzipped, or keep the raw files out of tree in
     `Code/testing/isometry/`, committing the summaries and scripts? Mark:
     "Keep raw out of tree." *Reading, not ruled:* the repository keeps a
     manifest naming each raw file's out-of-tree path and hash, so a copy can
     be checked; receipts already committed stay where they are.
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
257. **The clock's unit is a world setting, a minute by default.** Put to
     Mark on 2026-09-26, with 256: a second, a minute, or a world setting?
     Mark: "World setting, a minute default (a round in d&d and a few other
     systems)". *Reading, not ruled:* anything finer than a world's unit, a
     six-second round in newer tabletop systems or Eponym's real time, is the
     foreground game's to resolve, as rulings 232 and 233 and the VTT's
     handoff already have it.
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
259. **The operation budget counts only the evaluations that run.** Put to
     Mark on 2026-09-26, with 258: the budget counts evaluations, so skipping
     blocked ones changes what a budget allows; count the skipped too, staying
     byte-identical everywhere, or count only what runs, re-baselining
     budgeted sessions? Mark: "Count only what runs." So budgeted sessions take
     a new baseline, and six tight-budget saves that failed to load in the
     other mode are checked again.
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
261. **The background is hybrid: derived where effects commute, a calibrated
     rate model where they do not.** Put to Mark on 2026-09-26, from the
     review's third finding, that exact grouping folds only identical members
     and the crowd's savings fell from 11 to 2.4 times once strain was
     per-member state: what produces the background, ruling 113 staying the
     check? Mark: "Hybrid." So the background is derived from the definition
     where effects commute, by lossless grouping, and where they share
     resources, targets or relationships it is a rate model per site and
     lineage, calibrated against the exact runner on the bench.
262. **A vertical probe comes before the order from bodies onward is taken
     as fixed.** Put to Mark on 2026-09-26, from S2's checkpoint 3 plan for
     the thirty shapes: keep ruling 195's order with no probe, run the five
     used mechanics over a minimal allocated body exact against crowd with
     controls first, or move one whole body module first? Mark: "Vertical
     probe first." So the probe shows which extensions the mechanics need and
     whether the crowd still finds identical members once bodies carry cells,
     and the order from bodies on is confirmed after it. *Confirmed
     2026-10-02 by ruling 457,* after checkpoint 6.
263. **Each family is built and certified in Isocosm, and `mesocosm-core`'s
     copies retire together at M3.** Put to Mark on 2026-09-26, with 262: how
     is "`mesocosm-core` no longer owns it" met while Mesocosm's consumers
     still use the core: build and retire together, consume Isocosm's types
     per family, or move matter and processes with bodies? Mark: "Build,
     retire together." The duplication is temporary and no adapter is built.
264. **Part roles are open keys in the world's rules with a default set, and
     that set is examined before it is fixed.** Put to Mark on 2026-09-26,
     with 262: part roles from `isometer-core`'s closed four, open keys, or
     open keys with the four as the default? Mark: "i feel like we're missing
     many default part roles, but 1 sounds right. i don't mind isometer
     depending on a basic set of archetypal parts satisfying each of the many
     roles. limb, mass, sensor, plate, neurology, structure, idk. just feels
     underexamined". Told that the code's four roles are shapes, lump, rod,
     sheet and point, while a part's function comes from the processes it
     expresses, so that neurology and structure are functions, and asked
     which axis wants more: Mark: "Examine first." A survey of both axes, in
     the code and in biology, comes back to him before the default set is
     ruled.
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
266. **Rate models take over where grouping stops paying.** Put to Mark on
     2026-09-26, from a second review, which found that the certified crowd
     handled a shared resource's competition exactly in distribution: move
     ruling 261's line from "shares resources, targets or relationships"?
     Mark: "Where grouping stops paying." So the exact crowd runs wherever it
     still finds identical members, shared-resource competition included, as
     certified, and calibrated rate models take the ground where diverse
     bodies and relationships defeat grouping. This amends 261.
267. **A level of the web stays alive by four measures.** Put to Mark on
     2026-09-26, from the second review's caution that births balancing deaths
     could certify a lifeless world: ruling 260's "stays alive in most draws",
     by births balancing deaths, or by persistence, turnover, collapse and the
     web's response to an intervention? Mark: "The four measures." Each has a
     bound in the world's rules.
268. **Computed amounts are bounded expression trees, with a default set
     easy to group and calibrate.** Put to Mark on 2026-09-26, from S2's
     checkpoint 3 plan, decision 5: linear in one reading, clamped, with a
     draw term; a bounded expression tree; or named native formulas? Mark:
     "2, but with a default set of expressions that are easy to group and to
     calibrate?" So a world may author a bounded expression tree, and the
     default expressions are ones the crowd groups and the bench calibrates
     easily. *Reading, not ruled:* the linear, clamped form with a draw term
     is among the defaults.
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
270. **The flow record is buffered outside state.** Put to Mark on
     2026-09-26, from decision 7: each tick's matter receipts, reconciling
     every compartment, buffered outside the state hash, kept in state under
     the history cap, or no stream? Mark: "Buffered outside state."
271. **Matter a dev places comes from a dev source.** Put to Mark on
     2026-09-26, from decision 8: a dev intent placing matter breaks the
     conservation invariant; a dev source outside the conserved total,
     creative mode only, or refused? Mark: "A dev source." So it enters from
     outside the conserved total, as Mesocosm's DT3 does, and the run is
     labelled assisted.
272. **Pressure profiles become founding presets.** Put to Mark on
     2026-09-26, from decision 9: `mesocosm-core`'s pressure profiles, which
     nothing in the sim reads: founding presets, authoring reference, or
     retired? Mark: "Founding presets."
273. **Checkpoint 3's four readings stand.** Put to Mark on 2026-09-26: R1,
     untyped matter becomes the world's own, living matter accounts typed by
     lineage, digestion yielding the eater's matter; R2, a shape's identity is
     its set of roles; R3, a native reading of a member's own state runs in
     the crowd per exact state, so only opaque run-time scripts stay
     foreground-only; R4, until bodies move, seeding's only consumer is the
     founding generator. Mark: "Accept all four."
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
275. **Before motion is assigned, the sim resolving fine position up close is
     measured.** Put to Mark on 2026-09-26, from both reviews: who owns a
     nearby critter's movement, contact and pose? Mark: "i worry 1 would be
     too expensive for the sim. if so, 2". Answered that with a minute clock
     (257) the sim resolving it means a finer sub-clock inside the region
     examined up close, the cost he worried about, and that 257's reading
     already gives what is finer than the unit to the foreground game. Put
     back, record the foreground game's positions or measure first? Mark:
     "Measure option 1 first." *Reading, not ruled:* the measurement waits
     until the vertical probe (262) gives Isocosm minimal bodies.
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
278. **The function vocabularies become one catalogue in the sim.** Put to
     Mark on 2026-09-26, from the survey, which found four that never meet,
     Mesocosm's processes, `wing-functions`' kinds, Eponym's grip and
     adhesion, and its contact probe's impairments: one catalogue every game
     reads, or separate for now? Mark: "One catalogue."
279. **The far rungs are parked.** Put to Mark on 2026-09-26, from the first
     review: divinity, magic, language, technology and neighbouring worlds
     stay ruled but pull no code lane until a region with an ecology and a
     society runs; park them explicitly, or leave them as they are? Mark:
     "Park explicitly." No ruling is lost; the condition that unparks them is
     recorded in the sim plan.
280. **A doc lane clears the rewrite debt.** Put to Mark on 2026-09-26, from
     the first review's count, sixteen plans still marked "under rewrite" or
     "not authoritative" since W1: a doc lane, as each is touched, or archive
     the stale? Mark: "A doc lane clears it." The RPG systems session takes
     it on its own branch, rewriting each plan to the record or archiving it
     with a pointer, reviewed here.
281. **The anatomy brief follows the probe.** Put to Mark on 2026-09-26, with
     rulings 276 to 278: write the three-level model up now, ahead of S1's
     body schema, or after the vertical probe? Mark: "After the probe."
     *Item added 2026-10-01 by ruling 456:* where tissue and reserve sit in
     a body, and whether a bite lands on one part. *Opened 2026-10-02 by
     ruling 458.*
282. **A lane wires paging into the VTT's scene board now.** Put to Mark on
     2026-09-26, from the first review: the board refuses its brick map past
     `modulus::MAX_BRICKS` of 2,047, about 70 tiles square, where the
     generator authors 256, and no paging is wired into isometer's ground
     terrain; note it for V2 or open a lane? Mark: "Open a lane now." It
     works within this repository, and stops if the cap in mere's `modulus`
     would have to change.
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
284. **An advance's limit guards work, not ticks.** Put to Mark on
     2026-09-26, from the scheduler index's report: at a minute's unit a
     century is 52.6 million ticks against a default advance limit of
     100,000, while the due-event queue skips idle time; guard work, raise the
     tick limit, or run many advances? Mark: "Work, not ticks." So the limit
     guards evaluations, and an advance may span any stretch of idle time.
285. **The budget counts the members an evaluation stands for.** Put to Mark
     on 2026-09-26, with 284: individual mode evaluates a cohort member by
     member, so a history that fits a budget grouped can exceed it
     individually, as seven saves showed; count members, or let a budget
     belong to its mode? Mark: "Count members." So a budget means the same in
     both modes.
286. **Due events are kept per group.** Put to Mark on 2026-09-26, with 284:
     84 to 96% of the remaining evaluations did nothing, death and age
     processes visiting every member each period to be blocked by thresholds,
     and ruling 258 had left per-entity queues for before M2's first
     receipts: due events per group, per entity, or not yet? Mark: "Due
     events per group." So a group knows when its next threshold falls due,
     an age at death or a reserve running out, and is scheduled for then;
     grouping is kept, and the changed results are certified again.
287. **Each consumer lineage has one feeding process, choosing among its
     prey.** Put to Mark on 2026-09-26, with 284: the ecology generator made
     one feeding process per pair of predator and prey lineages, so feeding
     multiplied with lineages; one per consumer, or keep the pairs? Mark:
     "One per consumer." Its choice among prey is the definition's target
     selection.
288. **mere's retarget defect is fixed first.** Put to Mark on 2026-09-26,
     from the paging lane's assessment: after a retarget that shrinks the
     resident selection, `modulus` can leave a kept brick in a slot past the
     resident count, so its refresh panics and picks pass through visible
     ground; work around it here and fix mere next, fix mere first, or stay
     out of mere? Mark: "Fix mere first." A lane fixes it in its own worktree
     of mere, off `origin/main`, before any paging is wired here.
289. **mere's atlas is sized to the card.** Put to Mark on 2026-09-26, with
     288: the resident cap of 2,047 bricks does not cover a 256-tile board
     with relief in the headed pane; a card-sized cap, or 2,047 for now?
     Mark: "Card-sized cap." It rides with the defect's fix.
290. **The board's CPU side is paged too.** Put to Mark on 2026-09-26, with
     288: every overlay change regrows the whole map, seconds a click at 256;
     page it too, computing bricks from the map on demand, or atlas only?
     Mark: "Page it too."
291. **Residency follows the view, in an isometer helper.** Put to Mark on
     2026-09-26, with 288: which bricks are loaded, and where the code lives?
     Mark: "View, isometer helper." So the bricks overlapping the pane plus a
     one-brick margin, a pure function of view and map; the mechanics in a
     product-neutral helper in `shared/isometer`, the board's rule in the VTT.
292. **Isometry pins mere's current main.** Put to Mark on 2026-09-26, with
     288, the fix and the cap touching only `conatus`, unchanged since the pin
     of 2026-09-15: pin a patch commit on the old rev, or move the six
     manifests to mere's current main, 233 commits on? Mark: "mere's current
     main." So the pin bump adapts isometry to those commits, and follows the
     fix onto mere's main.
293. **Per-group phases run in the foreground; the background keeps a shared
     grid.** Put to Mark on 2026-09-26, from ruling 286 as built, which left
     results identical by visiting a group only once it is ready, while each
     process kept its period grid: that, or per-group phases, each group on
     its own clock? Mark: "Per group phases if they are in the foreground or
     nearby, baseline as built?" Answered yes: members in the foreground and
     nearby run as individuals anyway (rulings 113, 212), so phases there cost
     no grouping. They come with the sim's attention sets.
294. **A consumer eats across its prey's matter, once a period, choosing its
     prey by a weighted draw.** Put to Mark on 2026-09-26, from ruling 287's
     fork, one feeding process being unable to name one account to take
     from: what is eaten, how often, and how prey is chosen? Mark: "Drawn
     across its matter", "Once per period", "Weighted draw". So up to n of
     the chosen prey's matter is drawn proportionally from all its accounts
     and credited to the eater (X4 with R1's digestion); a consumer feeds
     once a period, the certification showing the new balance; and its prey
     is a seeded draw weighted by what each eligible prey holds.
295. **mere's atlas is sized by limits the host fills in.** Put to Mark on
     2026-09-26, from the mere lane's assessment for ruling 289: a plain-data
     `AtlasLimits`, the 3D texture dimension and an atlas byte budget, filled
     from the device's enforced limits, so `modulus` stays GPU-free; a brick
     count rounded up to whole rows; the 16 by 16 grid kept, rows growing;
     `MAX_BRICKS` kept as the default's cap of 2,047 so existing callers are
     unchanged; no limits-aware `from_keys` yet. Mark: "Accept the shape." The
     retarget fix lands beside it, with `modulus`'s tests moved out of its
     777-line `lib.rs` into a sibling file ("Keep it").
296. **The host's atlas budget defaults to 8 MiB.** Put to Mark on
     2026-09-26, with 295, video memory being unreportable in wgpu: a
     default budget of 8 MiB, 32 MiB, or as the texture allows? Mark: "8
     MiB." So a host asks for up to 16,383 bricks by default, enough for a
     1920 by 1080 pane over the 256-tile relief board, 6,391 with its margin.
297. **hagiograph folds into the whole-mere bump.** Put to Mark on
     2026-09-26, from the pin bump's assessment: hagiograph was pinned apart,
     at `53648d3a` in `mesocosm` and `shared/isocosm`, "until a deliberate
     whole-mere bump", which ruling 292 is; fold it in, or keep it apart?
     Mark: "Fold it in."
298. **Cleromancy is an optional feature of the VTT's host, off by
     default.** Put to Mark on 2026-09-26, from the bump's finding that the
     host depends on cleromancy unconditionally while cleromancy still pins
     the old mere and genet. Mark asked: "Isometry-genet is a stale name if
     I've ever heard one. Why do we depend on cleromancy? We probably
     shouldn't; it's meant to be extra flavor on top of the sim's
     self-sufficient processes, not a crucial dependency like mere." Answered
     that one module uses it to seal a GM's disclosed generator choice, the
     VTT keeping entropy, preview and commit. Put back, optional off by
     default or removed? Mark: "Optional, off by default." Without it the
     choice uses the VTT's own seeded draw, and cleromancy aligns in its own
     repository on its own schedule.
299. **`isometry-runtime` retires.** Put to Mark on 2026-09-26, the bump
     finding the crate pinning an old conatus: "What is that needed for?
     Also, stale name". Answered that it is the leftover of the runtime
     profile plan ruling 31 retired, a second renderer for the board, excluded
     from the workspace and referenced by nothing, whose binding table and
     event mirror were to survive one tier down as a stack adapter. Put back,
     retire it or keep it parked? Mark: "Retire the crate." Whether those two
     pieces reached the stack is checked as it goes.
300. **The VTT's crates keep their prefix.** Put to Mark on 2026-09-26, after
     he called `isometry-genet` a stale name: rename the VTT's crates after
     the bump, during it, or keep ruling 111's `isometry-` prefix? Mark: "Eh.
     If they're within the ambit of isocosm vtt, then fair enough." So ruling
     111 stands.
301. **Until the atlas is sized to the card, a frame's overflow drops the
     bricks farthest from its centre, the margin's first.** Put to Mark on
     2026-09-26, from Lane E's paging report: a frame that shows more bricks
     than the 2,047 the atlas holds, as the relief board does at the headed
     pane, loses its outer ring; keep the lane's interim rule, or keep what
     is nearest the camera? Mark: "Keep it." The dropped bricks are counted,
     and the rule lapses when the card-sized atlas lands with the pin bump
     (ruling 289).
302. **The pointer volume reserves headroom above the board's tallest
     point.** Put to Mark on 2026-09-26, with 301: the volume's height
     follows the terrain, so an edit that lifts the tallest point into a new
     brick layer rebuilds the map whole, a 1 MiB upload; accept the rare
     rebuild, or size the volume with a spare layer or two? Mark: "Reserve
     headroom." Only an edit past the headroom rebuilds. *Reading, not
     ruled:* the spare count is a setting, one layer by default until its
     memory cost comes back to Mark.
303. **The board's tallest elevation is cached, not scanned.** Put to Mark
     on 2026-09-26, with 301: Lane E found `BoardWorld::new` scanning the
     whole elevation grid on every signature check, 2.73 ms a frame at 256
     by 256; fix it now or later? Mark: "Fix it now." The cache is kept
     whenever the map changes, a lowered tallest tile included, and results
     stay identical.
304. **The check is made to see prey choice, by a reading and by the
     domain.** Put to Mark on 2026-09-26, from Lane A's checkpoint 4, which
     certified hunting exact against crowd on every reading, 34 of 34 with
     food and 44 of 44 with water, while a crowd drawing prey the wrong way,
     by headcount rather than by holdings, was caught on one reading with
     food and none with water: add a reading of prey choice, strengthen
     predation in the domain, both, or accept the gap? Mark: "Both." So a
     reading shows which prey were taken, the domain is one where the choice
     moves outcomes, and the check certifies again before Part A merges.
305. **The hunting domain's ranges widen.** Put to Mark on 2026-09-26, with
     304: Lane A set 4 to 16 hunters per site, bites of 1 to 2 and an
     appetite of 3 to 6 itself; keep them or widen them? Mark: "Widen them."
     The ranges chosen come back to him with the result.
306. **Hunting's shortage moves to ruling 269's scramble in Part B.** Put to
     Mark on 2026-09-26, with 304: when prey run out part way through a
     site's hunters the lowest identities still eat first, the one place
     identity order decides an outcome, and the crowd matches only because a
     site's hunters start as one group; switch to 269's proportional
     scramble in Part B, or before Part A merges? Mark: "In Part B." Part A
     merges with identity order and the case counted, 6 draws in 1,000 with
     food and 3 with water. *Carried out 2026-10-01 by ruling 454:* hunters
     draw from the pass's start and share a prey that cannot cover them.
307. **The draw control stays in the check.** Put to Mark on 2026-09-26,
     with 304: Lane A added the crowd drawing prey by headcount as a control
     arm of its own accord; keep it or keep the check to the arms the
     rulings named? Mark: "Keep it." It is the control that shows whether
     the check sees prey choice at all.
308. **Mesocosm's founding plan is archived once its Tone section and the
     epoch loop's turn structure are carried into the overlay plan.** Put to
     Mark on 2026-09-26, the doc lane's (ruling 280) first fork: the plan's
     world half is the sim's and its game half confirmed, while six live docs
     cite it, two as the owner of the epoch loop's turn structure and two for
     its Tone section and pillar 5, the Tone section existing nowhere else;
     archive it after carrying those over, keep it as Mesocosm's charter, or
     rewrite it to the game half? Mark: "Archive, carry over." So the
     Mesocosm overlay plan becomes Mesocosm's charter and the six citations
     repoint.
309. **The ProcessDef plan is archived.** Put to Mark on 2026-09-26, the
     second fork: ruling 32 makes its thirty shapes expressions in the sim's
     process definition (sim plan §3.1), and the overlay plan's M2 starts
     with matter and processes; archive it, rewrite it as Mesocosm's
     expressions, or keep it until M2's first family lands? Mark: "Archive."
     It points to §3.1 and M2, and the five plans citing it repoint.
310. **The general model's wing organs move to a plan of their own, and the
     rest is archived.** Put to Mark on 2026-09-26, the third fork: §1 to §6
     and E0 to E4 are the sim's, and the live part is §7.4 onward, the
     hagioglyph and impresa organs, gates G5 to G7 and the impresa finding
     of 2026-09-21; split the organs out, keep the plan as their home, or
     archive it whole? Mark: "Split out the organs." A new dated wing-organs
     plan carries §7.4, the G gates and the finding, and citations of §7
     repoint to it.
311. **The dependency ledger is archived; the order lives in the record
     alone.** Put to Mark on 2026-09-26, the fourth fork: the ledger orders
     work product-first where §11 orders it W0 to W5, sim first, and each
     overlay plan carries its own phases; archive and repoint, rewrite it
     under W0 to W5, or keep it with a header? Mark: "Archive and repoint."
     The plans citing it as ordering authority repoint to §11 and the
     overlay plans' phases.
312. **Environmental surfaces is rewritten as a short VTT note.** Put to
     Mark on 2026-09-26, the fifth fork: the plan is one product's tile
     layer, where the sim holds environment as a field on places moved by
     processes with no agent (sim plan §2.4, ruling 12); archive it with
     pointers, or rewrite it as a note on what a ruleset reads from the
     environment field on a battlemap? Mark: "Rewrite as a VTT note."
313. **Eponym's execution plan retires into the Eponym overlay plan now.**
     Put to Mark on 2026-09-26, the sixth fork: its F0 to F8 layers map one
     to one onto E2's families, while its order is product-first and its
     receipts are fixtures; keep it and retire it at E3, retire it into the
     overlay plan now, or rewrite its §4? Mark: "Retire into the overlay
     plan now." F3 to F8 map onto E2's families in a table in the overlay
     plan, and the plan is archived with a pointer. The line in
     `eponym/CLAUDE.md` that names it "the executable plan" changes, its
     wording going to Mark before the edit.
314. **Functional loops' T2 goes ahead now; T1 and T3 wait for E2's
     families.** Put to Mark on 2026-09-26, the seventh fork: when does the
     T lane run against E2's order, T2 now with T1 and T3 after, all of it
     after places, or all of it now? Mark: "T2 now, T1 and T3 later." T2,
     one edit reaching every spatial consumer, is the stack's and reads no
     place node; T1 and T3, custody, storage and projects, wait for E2's
     bodies-and-holding and lives-and-rounds families.
315. **World conditions is archived now.** Put to Mark on 2026-09-26, the
     eighth fork: ruling 32 founds the sim's process definition on it and
     the sim plan's §3.1 takes eight of its twelve parts by name, with S2
     under way; keep it until S2 lands, archive it now, or rewrite it to
     evaluator ownership? Mark: "Archive now." So §3.1 and this record cite
     the archived copy, and its Eponym-owned sections go with it.
316. **The 4b hunting domain stands, prey fat included.** Put to Mark on
     2026-09-26, from Lane A's checkpoint 4b under ruling 305: 12 to 48
     hunters per site against 128 to 256 prey, bites of 2 to 6 on prey
     bodies of 1 to 10, an appetite of 2 to 12, and a new prey fat of 0 to
     48 per prey group taken only by hunting, added because prey settled at
     nearly one weight, so a draw by holdings barely differed from one by
     headcount. With them the prey-choice reading caught the wrong-way crowd
     at a distance of 0.44, and hunting certified on 37 of 37 readings with
     food and 47 of 47 with water. Accept the ranges or send them back?
     Mark: "Accept them."
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
318. **The headroom's picture change is explained before paging merges.**
     Put to Mark on 2026-09-26, from Lane E's second round: with one spare
     layer the 256 board changes in 356 pixels, 0.011%, two 3-pixel bands
     where edge pixels land one step aside, while headroom 0 is
     byte-identical to before and the demo board is unchanged; each spare
     layer costs about 20 KB of pointer volume at the headed pane,
     re-uploaded on every pan that changes the held bricks. Find the cause
     first, merge at a default of 0, or merge at 1? Mark: "Find the cause
     first." A fix in isometer's tracer or mere's traversal comes back to
     him.
319. **The side panel's shrink at the new pins is measured against its text
     before its figures move.** Put to Mark on 2026-09-26, from Lane G's
     report: at the new pins two root tests fail because the side panel
     measures shorter, 796 to 768 px expanded, each row about a pixel
     shorter, `.side-line` rows 15 px around 16 px text with no line height
     of their own, so genet's default line height changed between the pins;
     compare the rows against their text at both pins first, or update the
     figures? Mark: "Compare first." If genet's default now falls below the
     text, it is a genet regression, brought to him before genet is touched.
320. **The sim core keeps an observation hook.** Put to Mark on 2026-09-26,
     from checkpoint 4b: to read prey choice in the exact runner, Lane A
     added `Simulation::watch(process)`, which keeps each accepted act of a
     process with the matter its target held as the act began until
     `take_watched()`, changes nothing the world does, and drops the watched
     acts of a refused advance; keep the hook, carry the holdings in the
     act's event, or snapshot in the probe? Mark: "Keep the hook."
321. **Isometry's toolchain moves to mere's, 1.98.1, with the pin bump.** Put
     to Mark on 2026-09-26, from Lane G's report: mere moved to Rust 1.98.1
     while isometry's `rust-toolchain.toml`, "pinned to match mere's", stayed
     at 1.97.1, on which everything still builds; match mere in the bump's
     merge, or stay and reword? Mark: "Match mere, with the bump."
322. **mere's conatus pins burn and cubecl exactly.** Put to Mark on
     2026-09-26, with 321: isometer's all-features build fails on a fresh
     lock, before the bump and after, because conatus asks for burn
     ^0.22.0-pre.2 and cubecl ^0.11.0-pre.2, and a caret on a pre-release
     admits pre.4, which conatus's `resident` module does not compile
     against; exact pins in mere, an exact pin in isometer-lens, or
     isometer's lock committed? Mark: "Exact pins in mere." *Reading, not
     ruled:* isometry's pins move to the mere revision that carries them.
     *Reopened the same day:* it conflicts with ruling D1 of mere's burn
     migration plan (2026-09-16), which moves mere to pre.3 as caret
     requirements and revisits tightening at stable 0.22.0. Put back,
     Mark asked whether burn 0.22.0-pre.4, published four days before,
     lets everything move to it; that is assessed before this is ruled
     again.
323. **With Cleromancy off, `>choose` seeds its draw from the command.** Put
     to Mark on 2026-09-26, with 321: the VTT's own seeded draw of ruling
     298 takes the command's seed and domain, so a command always makes the
     same choice, or the host's generation tape? Mark: "The command's seed."
     So the choice replays from the command alone, as Cleromancy's derived
     selection does.
324. **One binding adapter is planned now for Mesocosm's critters and the
     VTT's tokens.** Put to Mark on 2026-09-26, from Lane G's check under
     ruling 299: neither of `isometry-runtime`'s pieces, the binding table
     of tokens to conatus bodies and the mirror turning accepted map events
     into commands, reached the stack; isometer names no conatus body and
     conatus keeps durable bindings outside it, while Mesocosm's
     `TactileWorld` maps critters to conatus bodies itself. Plan one adapter
     now, build it now, or let them go? Mark: "Plan it now." An assessment
     takes the retired pieces, readable at f15fd43, and `TactileWorld` as
     input, and the adapter is built when he says.
325. **The overlay self-test's tie is broken by coordinate, with the paging
     merge.** Put to Mark on 2026-09-26, from Lane E's second round:
     `crates/isometry-genet/src/selftest/overlays.rs` picks its hover tile by
     the largest key over a hash map in which two tiles tie, so its path
     overlay differs between runs; fix it with the paging merge, or leave
     it? Mark: "Fix it with paging."
326. **Functional loops' T2 is assessed before it is built.** Put to Mark on
     2026-09-26, with ruling 314: T2, one edit reaching colliders,
     navigation and render through revisioned dirty regions, is stack work
     in mere's conatus and nisus (§4.3); assess first, build now, or wait
     until Eponym needs construction? Mark: "Assess first." A lane maps
     what conatus and nisus already do and what T2 needs, and comes back
     with forks before any edit to mere.
327. **The pin bump lands at mere's current main.** Put to Mark on
     2026-09-26, from the check behind ruling 322: mere's main had moved 16
     commits past the 668854c9 Lane G pinned and verified, with insigne's
     phase B and genet and netrender repinned to 0cf4f30b and c8c09f16;
     finish at 668854c9, or retarget? Mark: "Retarget to mere's main now."
     So isometry moves to mere 0418391f with genet 0cf4f30b and netrender
     c8c09f16, the toolchain to 1.98.1 with them (321), and the bump's
     checks run again there.
328. **`eponym/CLAUDE.md` names the Eponym overlay plan as the executable
     plan.** Put to Mark on 2026-09-26, after ruling 313 retired the
     execution plan: the doc lane's wording for the two lines that named
     it. Mark: "Amend as drafted."
329. **The pin bump merges now, and genet's text fragment is fixed next.**
     Put to Mark on 2026-09-26, from the side panel's comparison under
     ruling 319: at the old pins every row holds its text, and at the new
     ones rows at 11 to 13 px are a pixel shorter than their text fragment,
     because genet 6afb472a builds line boxes from rounded font metrics, as
     Chromium does, while its text fragment keeps Parley's taller extent;
     mere's newer genet does not touch that code. Bump now and fix genet
     next, fix genet before the bump, or accept it as Chromium does? Mark:
     "Bump now, fix genet next." The two figures move to the shorter panel,
     the row test is set aside with a pointer to this finding, and a genet
     lane makes the fragment take the line box's rounded metrics, landing
     with the next repin, mere's first.
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
331. **One edit reaches its consumers through a revision log.** Put to Mark
     on 2026-09-26, with 330: a log of changes keyed by revision at 8³-brick
     grain, each consumer reading what changed since its own last read and
     rebuilding when too far behind; per-consumer queues; or a product-side
     conductor? Mark: "A revision log."
332. **conatus carries the source's stamp; the barrier stays with the
     product.** Put to Mark on 2026-09-26, with 330: conatus stamps colliders
     with its own counter, which cannot be compared with the source's, and
     T2 wants a barrier holding dependent simulation until the colliders
     catch up; conatus carries the source stamp with the barrier in the
     product, products track it, or conatus holds the barrier? Mark:
     "conatus carries the source stamp." Its voxel edits take an optional
     source stamp that queries report, and the barrier lives in the product
     until a second consumer proves it.
333. **Navigation stays a search per query, each result stamped.** Put to
     Mark on 2026-09-26, with 330: no product caches navigation, each
     searching the current terrain per query, never stale but never saying
     which revision it read; stamp each query, a cached walkability product,
     or wait for the place graph? Mark: "Stamp each query." A cached product
     comes only when a measured query cost forces it.
334. **T2's first receipt is inside mere, then Mesocosm.** Put to Mark on
     2026-09-26, with 330: a conatus integration test driving the nisus
     store into a collider and a modulus refresh, then Mesocosm's carve and
     an additive write moving collider picks, routing and render slots at
     one revision; Mesocosm first; or Eponym's crossing, blocked until E2?
     Mark: "Inside mere, then Mesocosm."
335. **T2's plan is a lane in the conatus engine plan's §2.** Put to Mark on
     2026-09-26, with 330: mere's conatus engine plan §2 is complete when
     "terrain edits, body-volume edits, collision, queries, and mesh/SDF
     preparation use one chunk/revision path", T2 in general form; a lane
     there, a new plan in mere, or functional loops keeping it? Mark: "A
     lane in conatus's §2." Eponym's functional loops plan keeps only T2's
     product receipt and points there.
336. **The ray traversal's precision is fixed in mere.** Put to Mark on
     2026-09-26, from Lane E's third round under ruling 318: the 356 pixels
     come from `modulus`'s `brick_dda`, which adds each crossing step in
     32-bit floats from an eye about 1,380 units behind the board, so the
     error grows along each ray's walk and headroom only lengthens it;
     against a 64-bit traversal neither picture is exact, headroom 0 off at
     201 texels and 1 at 531. Fix it in mere, move the eye in isometer now,
     both, or leave it? Mark: "Fix it in mere." `modulus` computes each
     crossing directly or measures from the ray's entry into the box, and
     each product picks it up at its next repin, re-recording its picture
     receipts.
337. **Paging merges after the traversal fix.** Put to Mark on 2026-09-26,
     with 336: merge now at headroom 1, now at headroom 0, or after the fix?
     Mark: "After the fix."
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
339. **The catalogue starts with the five functions in use.** Put to Mark on
     2026-09-26, with 338: the five, all fifteen of ruling 277, or those
     with the other vocabularies folded in? Mark: "The five in use."
     Contract, intake, sense, fix and secrete, rods contracting, lumps
     taking in, points sensing, sheets fixing, and sheets secreting when
     acquired; the keys stay open, the rest arriving with the anatomy brief
     (281) and the probe's evidence. *Widened 2026-10-02 by ruling 461:*
     all fifteen of 277, the other vocabularies folded in.
340. **The process's causal kind gives up the name `Shape`.** Put to Mark on
     2026-09-26, with 338: `isocosm::rules::Shape`, a process's causal kind
     (choice, agentless, transition), collides with ruling 276's part
     shapes; rename the process one, or qualify the part noun? Mark: "Rename
     the process one." *Reading, not ruled:* it becomes `Causation`, the
     name offered, its saved field keeping the name `shape` so old worlds
     load.
341. **Seeding's two values are Grown and Acquired.** Put to Mark on
     2026-09-26, with 338: Grown and Acquired, or Mesocosm's Geometry and
     Acquired? Mark: "Grown and Acquired." *Reading, not ruled:* the names
     put with it stand, `Binding::Part`, the catalogue `functions` in the
     world's rules, and a part's `shape` and `functions`.
342. **Declared conversions are checked.** Put to Mark on 2026-09-26, with
     338: the sim admits any mass-balanced transform while Mesocosm admits
     three, synthesis, digestion into the eater's own matter, and
     mineralization; check declared conversions, record the kind and refuse
     nothing, or refuse all else? Mark: "Check declared ones." A transform
     declaring a conversion kind is checked against the three, and the rest
     pass as before, the certified probe's grazing and drinking included.
343. **Diffusion is ported as a kernel now and wired later.** Put to Mark on
     2026-09-26, with 338: Mesocosm spreads soil matter over a square grid of
     columns while the sim's sites form a route graph; a kernel now wired
     later, along site routes now, or left to places? Mark: "A kernel now,
     wired later." It is wired when the places family decides whether
     columns are sites.
344. **The dev source's issue and the assisted label live in the history.**
     Put to Mark on 2026-09-26, with ruling 271: in the history, in the
     world's state, or both? Mark: "In the history." The command in the log
     is the record, the world hashes as the matter it holds, as Mesocosm's
     DT3 does, and receipts show the issued amount beside the conserved
     total.
345. **The flow record holds every move, when asked.** Put to Mark on
     2026-09-26, with ruling 270: every move switched on when wanted, every
     move always on, or a net change per tick? Mark: "Every move, when
     asked." Each accepted act's matter moves, from, to, amount and the
     members it stands for, handed over each tick, every ledger reconciling
     from it in both runners.
346. **The body-binding shape is documented in conatus's docs now and built
     later; each product keeps its own table meanwhile.** Put to Mark on
     2026-09-26, from Lane I's assessment under ruling 324, which found the
     premise shifted: the board plan's live bodies are tokens drawn as
     isometer meshes, not conatus bodies, and isometer answers pointer picks
     itself, so no code queries a bound conatus body today, while tokens in
     a volume battlemap (243), critters in the terrarium and Eponym's solver
     (233) would. Design the shared mechanism anyway, document the shape
     only, or retire `TactileWorld`'s critter half? Mark: "Document the
     shape only." Asked how that sits with 348 and 349, Mark confirmed the
     reading: the common shape goes into conatus's docs now, saying where
     the module will live and how it works once built; nothing is built, and
     each product keeps its own table until he says.
347. **isometer alone answers a pointer pick.** Put to Mark on 2026-09-26,
     with 346: isometer only, conatus through the bindings as T1 was ruled,
     or both? Mark: "isometer only." The player clicks what was drawn, and
     isometer answers per part and per frame, as the VTT board already does;
     the bindings answer other queries.
348. **The binding module, when built, lives in mere's conatus, generic over
     the key.** Put to Mark on 2026-09-26, with 346: conatus, a new crate
     under `shared/`, isometer, or `mesocosm-runtime`? Mark: "A module in
     mere's conatus." Three mere doc passages are amended with the shape,
     one of them the line placing bindings "not to Conatus".
349. **It binds bodies only, on a world shared with T2's terrain collider.**
     Put to Mark on 2026-09-26, with 346: bodies on a borrowed world, bodies
     on its own world, or bodies and terrain? Mark: "Bodies only, shared
     world." Terrain arrives through T2's one path (rulings 330 to 335).
350. **Accepted state arrives both whole and per key.** Put to Mark on
     2026-09-26, with 346: a whole-set reconcile and per-key set and remove,
     reconcile only, or per key only? Mark: "Both." The reconcile is the cold
     rebuild and carries the VTT's per-event accepted map; per-key updates
     carry a critter changing alone; cold and incremental must agree.
351. **A moved body keeps its id.** Put to Mark on 2026-09-26, with 346: move
     it and keep its id, respawning only when the entity's shape revision
     changes, or always respawn? Mark: "Move it, keep its id."
352. **conatus gains a query-refresh call.** Put to Mark on 2026-09-26, with
     346: conatus's queries miss a topology change until the world steps, so
     `TactileWorld` and Eponym each run a tiny step; a refresh call in
     conatus, the tiny step kept, or left to the world's owner? Mark: "A
     refresh call in conatus." It is a small mere edit, independent of the
     bindings, that both workarounds become.
353. **Burn moves to pre.4 with exact pins, superseding the pre.3 repin and
     D1's carets.** Put to Mark on 2026-09-27, after ruling 322 was reopened
     and he asked whether pre.4 lets everything move to it. Lane H found the
     whole family published pre.4 on 2026-09-22, with no stable 0.22 yet; in
     mere only conatus breaks, about 18 mechanical lines in three files, and
     isometer-lens needs two; Distillery and its lease tests pass;
     cubek-reduce's and burn-remote's patches carry, burn-cubecl's needs a
     hand rebase and cubecl-runtime's can go; and pre.3 with carets would not
     fix isometer, a caret on pre.3 admitting pre.4. Pre.4 exact, pre.3
     exact, pre.2 exact in mere, or wait for stable? Mark: "Pre.4, exact
     pins." `=0.22.0-pre.4`, `=0.11.0-pre.4` and `=0.3.0-pre.4`; the parked
     pre.3 repin's commits guide the rebase. This answers ruling 322.
354. **No stopgap pin in isometer-lens.** Put to Mark on 2026-09-27, with
     353: pin burn at pre.2 in isometer-lens until the migration lands, or
     not? Mark: "No stopgap." isometer's all-features build stays broken on
     a fresh lock until then.
355. **mere's cubecl-runtime patch retires.** Put to Mark on 2026-09-27,
     with 353: pre.4 makes each allocation's identity public, if hidden from
     the docs; retire the patch or carry it re-pointed? Mark: "Retire it."
     burn-cubecl's guard compares the public id, and Knot's need to patch
     cubecl-runtime goes with it.
356. **turso is settled in the migration.** Put to Mark on 2026-09-27, with
     353: pre.4 brings in turso, a pre-release database, through cubecl's
     default `persistence` feature, drifting by caret in every GPU build;
     settle it in the migration, commit mere's lock again, or let it drift?
     Mark: "Settle it in the migration." The lane checks whether
     `persistence` can be turned off, which drops turso, and brings the
     answer back before anything is committed.
357. **A declared synthesis produces the actor's own lineage's matter.** Put
     to Mark on 2026-09-27, from Part B's step 2 under ruling 342: the sim's
     core has no notion of a producer; the actor's own lineage, a flora
     kingdom, or kingdoms the rules declare? Mark: "The actor's own
     lineage." Whatever has a synthesis process is a producer, mirroring
     digestion, and no producer classification enters the core.
358. **A dev placement may fill any declared matter account.** Put to Mark
     on 2026-09-27, with 357: any declared matter account, or world matter
     only? Mark: "Any declared matter account." Mesocosm's overlay chooses
     soil.
359. **Each flow names the process that moved it.** Put to Mark on
     2026-09-27, with 357: add the act's process as the reason, or leave it
     out as ruling 345 had it? Mark: "Add the process."
360. **Part shapes take their own key prefix.** Put to Mark on 2026-09-27,
     with 357: X1 keyed part shapes as `shape:*`, the namespace the world's
     own `shape:graph` uses; separate them or share it? Mark: "Part shapes
     get their own prefix." *Reading, not ruled:* `part-shape:*`, the
     example offered.
361. **isometer calls modulus's CPU walk instead of copying it.** Put to
     Mark on 2026-09-27, from Lane K's traversal fix under ruling 336, each
     crossing now computed as the boundary less the eye over the direction,
     which removed every fault and every headroom move on the VTT board
     frame, on the CPU and four GPU setups, for 0.02 to 0.08 ms a frame:
     isometer's GPU path includes modulus's shader, but its CPU pick path is
     a hand copy; call modulus's walk, or keep the copy? Mark: "Call
     modulus's walk." modulus makes its exact CPU mirror public and isometer
     wraps it.
362. **The traversal's start clamps into the box.** Put to Mark on
     2026-09-27, with 361: the shader's 1e-4 start offset is too small in
     f32 beyond t of about 1,000, so a first-voxel hit reports its entry face
     and the GPU and CPU disagree which, on 3,754 to 4,093 of 98,304 stress
     rays and none on the board; clamp the start voxel into the box, report
     the entry face, or leave it? Mark: "Adopt the clamp." It lands with the
     traversal fix.
363. **T2 is built after the pre.4 migration.** Put to Mark on 2026-09-27,
     with ruling 335: after the migration, now, or when Mesocosm needs it?
     Mark: "After the pre.4 migration." The traversal fix, the migration and
     T2 all touch conatus, so they land in turn.
364. **X1's defaults are copied in explicitly.** Put to Mark on 2026-09-27,
     from Part B's step 1: the eight shapes and five functions are constants
     a world copies in, the generator not yet copying them, and the crowd
     refuses a process binding a part until the probe certifies it; keep
     that, or have the generator copy them now? Mark: "Keep as built." Were
     the defaults implicit, growing the set later (281) would change what old
     worlds mean without changing their digest.

365. **The traversal repin proceeds with coordination.** Asked whether to
     repin isometry for the traversal fix now, leaving genet's text fix and
     burn pre.4 for later, or bundle all three, Mark answered: "You can repin
     but communicate with the isocosm agent". The 2026-09-27 handoff carried
     this answer unrecorded. The repin targets mere `7bb5bfda`; genet and
     netrender stay at their current pins. *Reading, not ruled:* the agent
     may mean Lane A (sim Part B) or the RPG session; its identity and the
     requested coordination remain unresolved before the repin proceeds.

     **2026-09-27 annotation:** the handoff preserves the question as
     "repin isometry now for the traversal fix, with genet's fix and pre.4
     in a later repin, or wait and bundle all three?" Its two alternatives
     are (1) repin traversal now and defer genet/pre.4, or (2) wait and
     bundle all three. Separate original option labels and recommendation
     order were not preserved in the handoff; they are not reconstructed.

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

367. **The new session orchestrates the repin and remaining lanes.** Asked
     which agent ruling 365 meant, with Lane A, the RPG session or both as
     the alternatives, Mark answered on 2026-09-27: "Ah, the rpg session
     isn’t active now. But you can orchestrate the repin/rest". This
     supersedes the waiting coordination step in 365: the new session takes
     over orchestration, preserving lane work and verification checkpoints.
     *Reading, not ruled:* this authorizes the traversal repin and lane
     coordination; unresolved design forks still come back to Mark.

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

370. **Atlas allocation is reopened around preserving resident GPU data.**
     Asked on 2026-09-27: "3. Should the atlas allocate its full budget
     immediately? The branch allocates all 8 MiB; the small demo's 260
     bricks fit in two 128 KiB rows. Growing later saves initial memory but
     requires replacing the texture and uploading all retained bricks
     again." Options, recommended first: (A) "Allocate the chosen budget
     upfront, keeping texture dimensions and slots stable (recommended)."
     (B) "Allocate occupied rows initially and grow up to the chosen budget,
     with replacement and full re-upload." Mark asked: "How costly is
     replacing the texture and uploading only the retained bricks that
     change? Is that possible?" This is a request to examine the premise,
     not a selection of either allocation policy. The assistant's initial
     answer: a new texture may receive unchanged residents through a GPU
     copy and only new or edited data through CPU uploads, with both
     textures alive during transfer. *Reading, not ruled:* verify the
     current layout and measure that alternative before putting the choice
     back. Allocation remains open; 368 and 369 proceed independently.

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

373. **Measure a more tightly batched GPU-copy path before choosing
     allocation.** Asked on 2026-09-27: "Given the measured replacement
     cost, which allocation path should paging take? Full uploads took
     0.198/0.589 ms in the two growth cases; the tested GPU-copy path took
     0.325/3.310 ms while sending 57–59% fewer CPU bytes. These are local
     medians, not worst-case frame guarantees." Options, recommended first:
     (A) "Allocate the chosen budget upfront; avoid growth events and keep
     the tested copy approach as research (recommended)." (B) "Grow by rows
     with full uploads for now; accept occasional replacement costs to use
     less initial memory." (C) "Measure a more tightly batched GPU-copy path
     before deciding; keep allocation open." Mark selected verbatim:
     "Measure a more tightly batched GPU-copy path before deciding; keep
     allocation open." This opens a bounded measurement, not production
     growth. *Reading, not ruled:* compare one ordered submission combining
     retained texture copying and staged changed-data copies, counting CPU
     preparation and completion, with readback and fault controls.

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

375. **Disable CubeCL persistence in pre.4; amend 355's patch retirement.**
     Asked on 2026-09-27: "1. Should pre.4 retain GPU tuning results between
     app launches?" Evidence as put: "Disabling persistence removes 55
     packages, taking the measured dependency count from 1,710 to 1,655. It
     requires retaining a small vendor patch, amending ruling 355. The
     tested Windows configuration already falls back to memory;
     persistence's benefit on Unix remains unmeasured." Options, recommended
     first: (A) "Disable persistence (recommended). Keep the manifest-only
     patch; tuning repeats when needed in each process. Retire the old
     identity helper." (B) "Keep persistence. Remove the runtime patch as
     355 intended; retain Turso and its dependencies." (C) "Measure Unix
     first. Compare cold and warm launches before choosing." Mark answered
     both questions verbatim: "I accept your two recommendations. Wow. Much
     larger dependency count than I figured." This accepts A: retain the
     vendored `cubecl-runtime` for the manifest-only removal of `persistence`
     from its defaults, while retiring its identity helper. This explicitly
     amends 355 and settles 356's persistence choice. Caches remain local to
     each process. The 1,710/1,655 figures are Lane H's lockfile package
     counts, including optional resolution, not every application's compiled
     crate count; the reduction is nine Turso crates plus 46 transitive
     packages. Unix performance has not been measured. *Reading, not ruled:*
     keep a narrow patch-removal condition when upstream makes persistence
     optional without the forcing edge. An upstream issue or PR was not an
     option in this round and is not authorized by this answer.

376. **Require the complete Distillery migration gates.** Asked on
     2026-09-27: "2. How thoroughly should we verify Distillery during the
     migration?" Evidence as put: "The full plan covers five nested
     workspaces and a two-peer run checking output, cancellation, memory
     recovery and a fresh lease. The remote fixture first needs its five
     outdated dependency patch rows aligned with the root's eight current
     rows." Options, recommended first: (A) "Require the full set
     (recommended). Repair and verify the fixture on pre.2 first, then
     repeat the complete checks on pre.4." (B) "Defer the two-peer run.
     Require compilation, lease tests and headed numerical checks; leave
     remote acceptance open." (C) "Limit this pass to compilation and
     lease tests. Leave both headed numerical and remote acceptance open."
     Mark answered both questions verbatim: "I accept your two
     recommendations. Wow. Much larger dependency count than I figured."
     This accepts A: preserve the fixture repair as a separate prerequisite,
     prove its pre.2 baseline, and require all five nested workspaces,
     compilation/lease tests, headed numerical controls and the two-peer
     lifecycle receipt after migration. Acceptance is not reduced to a
     compiling rebase. The remaining pre.4 stop-rule dispositions are not
     inferred from this answer.

377. **The pre.4 allocation guard also compares service identity.** Asked
     on 2026-09-27: "How should we carry the GPU allocation guard into
     pre.4? It currently compares five values: allocation ID, two slice
     offsets, stream and size. Pre.4 adds a service ID, but normal
     allocations still receive globally unique IDs, so a sixth comparison
     is currently redundant. The existing five must remain: different
     slices can share an allocation, and treating them as identical can
     produce the wrong calculation." Options, recommended first: (A)
     "Preserve the five comparisons; treat the new service field as context
     and proceed under the migration stop rule (recommended)." (B) "Add
     service ID as a sixth comparison; also verify the new rejection case
     before proceeding." Mark selected verbatim: "Add service ID as a
     sixth comparison; also verify the new rejection case before
     proceeding." The migrated guard preserves allocation ID, both
     offsets, stream and size and additionally requires equal service ID.
     The service mismatch rejection must be verified before proceeding;
     this disposes of the new-field stop under Mere's pre.4 plan S0-1 and
     specifies S0-2's comparison scope. *Reading, not ruled:* pair a valid
     matching-handle control with an otherwise equal handle whose service
     differs, and show that deliberately removing the service predicate
     makes the rejection check fail. Never submit the artificial mismatched
     handle to a GPU client; the control exercises the guard itself.

378. **Allow needed crates.io downloads for the pinned pre.4 migration.**
     Asked on 2026-09-27: "May Lane M download missing crates.io
     dependencies for the pinned pre.4 migration? The first concrete miss
     is cubecl-spirv 0.11.0-pre.4, required to resolve an optional
     dependency before the new guard can compile. The migration plan's
     §13.13 currently requires staying offline and returning with a bounded
     request when a package is missing." Options, recommended first:
     (A) "Allow crates.io downloads needed for this pinned migration;
     record them and keep existing Git revisions fixed (recommended)."
     (B) "Allow only cubecl-spirv 0.11.0-pre.4 now; ask again if another
     package is missing." (C) "Keep execution offline; preserve the
     prepared patches and stop before the guard build." The final direct
     question was: "May I download the crates.io dependencies needed for
     this pinned migration?" with the recommendation: "I recommend
     allowing that, recording downloads and preserving Git pins."
     Mark answered verbatim: "Ok." This accepts A and resolves S0-9 for
     the pinned migration: fetch required crates.io dependencies, record
     their versions and integrity evidence, and preserve existing Git
     revisions. It does not authorize unrelated upgrades or upstream
     communication. *Reading, not ruled:* use Cargo-managed cache fills
     for concrete misses, then return to offline/locked execution; retain
     original failures and source/hash-qualified test receipts. Ruling
     377's service rejection and deliberate fault control remain required.

379. **Finish explicit-height text and inline decoration bounds before
     merging Lane L.** Asked on 2026-09-27: "Should we fix both remaining
     text-bound issues before merging?" Evidence as put: "Three tests pass.
     At 16px Arial, text occupies 17px inside an 18px line. Source inspection
     shows explicit line heights and inline borders still use line-based
     bounds; those cases need measured fixtures." Options, recommended
     first: (A) "Fix both now (recommended). Add explicit-line-height and
     wrapped-border fixtures, correct the bounds, and verify the combined
     change before merging." (B) "Merge the normal-text fix first.
     Explicitly defer the two remaining corrections." Mark answered
     verbatim: "A". This extends ruling 329's fix scope: both remaining
     corrections and their measured fixtures belong in the combined lane
     before merge. *Reading, not ruled:* preserve current main's generated
     text and selection behavior while reconciling the lane, use the
     established font metrics for content bounds, and verify normal,
     explicit-height and decorated cases with fault-specific controls.
     Return with any metric-policy fork instead of inferring its answer.
     The three fresh tests do not certify the old browser, WPT or 187-row
     Isometry measurements; those remain separately qualified evidence.

380. **Fix the pre-existing alias broadcast defect on pre.2 first, then
     carry the verified correction into pre.4.** Asked on 2026-09-27:
     "Which repair order do you want?" Evidence as put: "We also localized
     the existing Seiche bug: our shared-buffer patch omits broadcasting
     for its second input. The small GPU fixture gets 6 of 9 values wrong."
     Options, recommended first: (A) "Fix pre.2 separately, then carry the
     verified correction into pre.4 (recommended). The existing bug gets
     its own tested commit." (B) "Fix only the migration lane. Pre.2 stays
     unchanged until the complete migration lands." The stated constraint
     was: "Allocation checks and numerical tolerances stay unchanged."
     Mark answered verbatim: "A". Repair and verify the existing pre.2
     patch in its own commit before carrying the correction into pre.4;
     the migration's remaining gates still apply. *Reading, not ruled:*
     construct alias views with the output's broadcast reference shape in
     all three affected launchers, preserving each input's own shape and
     strides, identity checks and separate output allocation. Use direct
     launcher regressions, same-shape and separate-input controls, a
     deliberately reverted layout correction, and full Seiche parity at
     its unchanged tolerances. The isolated subtraction receipt alone does
     not certify complete force calculations or migration acceptance.

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

385. **The founding record's amendment for ruling 381 is applied as
     drafted.** Asked on 2026-09-28: "Apply the founding record amendment
     for ruling 381 as drafted in the session notes §9.4? It adds a dated
     2026-09-28 amendment after 'require another vessel to be running': one
     host carries the modes over one world save, a build may carry one mode
     or all three, and what survives of the limit is 'no mode may require
     another mode's code to run', with each game keeping its genre, verbs
     and care granularity." Options, recommended first: (A) "Apply as
     drafted: the dated amendment goes into the founding record's 'Each
     vessel is a mode of the same peopled history' section; the original
     words stay as history, per the doc rules (recommended)." (B) "Hold for
     now." Mark chose A, "Apply as drafted". Applied the same day. The
     2026-08-10 words stand above the amendment as history, and ruling 381's
     reading that the guard survives as "no mode may require another mode's
     code to run" is now the founding record's text.

386. **The `CLAUDE.md` drafts are applied: the tabletop's camera line and
     the "schedule" wording.** Asked on 2026-09-28: "Apply the CLAUDE.md
     drafts? The tabletop's camera don't would become: the wing's default
     view is isometric with quarter turns (382); the tabletop ships its
     locked 2:1 lens today and reaches quarter turns through its own plan
     and render lane; free yaw, first person and over the shoulder are not
     precluded. Separately, Mesocosm's CLAUDE.md:34 and Eponym's
     CLAUDE.md:31 still say the vessels do not share 'a schedule', the
     wording ruling 33 retired." Options, recommended first: (A) "Apply
     both: tabletop camera line replaced as drafted, and 'a schedule' struck
     from Mesocosm's and Eponym's identity lines to match the founding
     record's 2026-09-18 amendment (recommended)." (B) "Camera line only."
     (C) "Hold both." Mark chose A, "Apply both". The tabletop's camera
     don't is replaced by the session notes' §9.4 draft, and "a schedule" is
     struck from both identity lines, which now agree with the founding
     record's sentence as ruling 33 amended it.

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

388. **Mesocosm's opening view is ruled on CP1's receipts, which compare its
     2026-09-05 direction with the wing default.** Asked on 2026-09-28:
     "Does Mesocosm open in the wing's default view?" Evidence as put: "its
     accepted direction of 2026-09-05 is shallow-depth up/down/left/right
     play with deliberate quarter-turn views of a cutaway terrarium; the
     clearing-and-burrow prototype CP1 that tests it is pending; the shipped
     default is an oblique section." *Corrected on recording:* CP1 has been
     implemented and native-verified since 2026-09-05, with its visual review
     pending (default creatures plan, CP1); it turns in quarter steps about
     world axes at a configurable 0 to 45 degree pitch, default 12, so what
     was pending is the review, not the prototype. Options, recommended
     first: (A) "CP1 compares both: the pending prototype runs the 2026-09-05
     direction and the wing default side by side on seeded draws;
     Mesocosm's opening view is ruled on its receipts (recommended)." (B)
     "Open in the default." (C) "Keep the section." Mark chose A, "CP1
     compares both". So CP1's review gains a second arm, the wing default
     (387's 30° pitch at the diagonal azimuth, turned in quarter steps),
     beside its 2026-09-05 arm, on seeded draws (ruling 15), and Mesocosm's
     opening view goes back to Mark on those receipts; the oblique section
     stays the shipped default until then. *Reading, not ruled:* the second
     arm needs a 45 degree azimuth setting CP1 lacks, since its turns are
     about world axes; the review weighs what the 2026-09-05 direction was
     accepted for, 2D legibility with 3D space and a visible burrow
     interior, not a preference score.

389. **The spatial spine is designed now and sliced around the recorded
     order.** Asked on 2026-09-28, when Mark chose to start on the first gap
     of the one-game assessment, connecting the sim's site graph to the
     voxel world ("Let's start on 1?"): "How does starting on gap 1 run
     against the recorded order? Ruling 195 puts places fourth in Isocosm's
     absorption (the sim is on matter and processes now), and ruling 363
     holds T2, the nisus world store, until the pre.4 migration ends
     (production is still on pre.2, S13 open)." Options, recommended first:
     (A) "Design now, slice around: plan the whole spine now, and build a
     first slice that jumps neither ruling: site skeleton, shared edge
     profiles and deterministic lifting into today's Ground through the
     Terrain seam, proven on the Isocosm bench; nisus takes the store over
     when T2 lands. mesocosm-core's places family still moves fourth
     (recommended)." (B) "Move places earlier: amend 195 so places move
     ahead of bodies and the record, and build the spine in Isocosm now; T2
     still waits on pre.4." (C) "Design only for now: no code until places'
     turn in 195's order and T2 after pre.4." Mark chose A, "Design now,
     slice around". So the spine is planned whole now, its first slice is
     new Isocosm capability that neither moves `mesocosm-core`'s places
     family ahead of ruling 195's order nor touches mere ahead of ruling
     363, and nisus takes the store over when T2 lands.

390. **The place-graph engine plan is rewritten as the spine's plan.** Asked
     on 2026-09-28: "Where should the spine's plan live? The place-graph
     engine plan owns 'the world substrate: the place graph, volumetric
     truth' but has been marked for rewrite since W1 (2026-09-18), its
     done-conditions not authoritative; the isoscape plan (2026-09-16)
     extracts the place graph into isoscape, which ruling 192 (2026-09-25)
     overtook by sending places to Isocosm." Options, recommended first: (A)
     "Rewrite place-graph plan: rewrite the place-graph engine plan to the
     record as the spine's plan, keeping its landed G0 to G4 receipts as
     history, and annotate the isoscape plan's place-graph row as overtaken
     by ruling 192 (recommended)." (B) "New spine plan: a new dated plan owns
     the spine; the place-graph plan is archived with a pointer." (C) "Fold
     into the sim plan: rows of §2.2 and a new S phase." Mark chose A,
     "Rewrite place-graph plan". The rewrite discharges W1's verdict for
     that plan (ruling 31).

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

395. **The first slice builds square sites on planes, rings and tori.**
     Asked on 2026-09-28, decision 3: "Which world shapes and site
     footprints does the first slice build?" Evidence as put: "today's
     generator draws a ring plus one random extra route per site with no
     geometry, so no current world has frames; square footprints with four
     edges tile planes, rings and tori under one frame rule; a sphere needs
     a geodesic grid of hexagons with twelve pentagons (record §3.7.1),
     whose borders need polygon interpolation." Options, recommended first:
     (A) "Squares first: square sites with four edges on planes, rings and
     tori; the edge-profile type is designed for polygons, and hexagons and
     pentagons are built second (recommended)." (B) "Polygons now, with a
     geodesic sphere among the first draws." (C) "One site first, against
     drawn border profiles." Mark chose A, "Squares first".

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

408. **SP3's capture shows water as opaque voxels.** Asked on 2026-09-28,
     after the spine's SP2 landed at `38ea90f`: "How does SP3's capture show
     water? Evidence, checked in code today: Ground stores a sea_level that
     nothing in isometer reads, so the renderer draws no water at all;
     ruling 28 (water as a field, salva for particles) and ruling 22's
     transparency row are unbuilt; the lift now yields water cells up to
     each site's level." Options, recommended first: (A) "Opaque water
     voxels: the bench adapter paints water cells as a solid palette colour,
     so the capture shows where water lies and whether it meets across the
     border; real water (transparency, the field, salva) stays ruling 28's
     lane (recommended)." (B) "Leave water out, reported in the receipt."
     (C) "Build water first, in a render lane of its own." Mark chose A,
     "Opaque water voxels". Put as the first of SP3's forks and first
     numbered 404, renumbered on recording because a parallel session had
     taken 404 to 407.

409. **SP3 captures a border window and an overview.** Asked on 2026-09-28:
     "What does SP3's capture frame? Evidence: a site at the draws' largest
     is 2,048 base units a side, about 4.2 million columns at the base grain;
     isometer's Ground grows one bounded extent column by column; SP2 lifts
     any window at any power-of-two cell size." Options, recommended first:
     (A) "Border window and overview: two captures, a window straddling the
     shared border at the base grain, where a seam would show up close, and
     both whole sites at a coarse level, where the two read as one
     landscape; each receipted with its seed and border digests
     (recommended)." (B) "Whole sites, coarse." (C) "Border window only."
     Mark chose A, "Border window and overview".

410. **Compare historical pre.2 first, then retire the patch if possible.**
     Asked on 2026-09-29 in the migration conversation: "Pre.4 passes all 21
     browser cases both with and without our patch, with zero GPU errors;
     independent review ruled out stale assets. August's pre.2 receipt failed
     shared multiplication and both LayerNorm cases. The plan makes carrying
     or retiring the patch your call. How should we proceed?" Options:
     (A) "Compare historical pre.2 in this same browser first, then decide
     whether to retire the patch (recommended)." (B) "Retire the patch based
     on this result, then run the remaining migration checks." (C) "Keep
     the patch as a precaution, record that the old failure no longer
     reproduces, and run the remaining checks." Mark answered verbatim:
     "I suppose A, then B if we can".

     The historical comparison proceeds first. *Reading, not ruled:* the
     answer authorizes retiring the `burn-cubecl` patch if the comparison
     and reviewed evidence support it, followed by the remaining migration
     checks. It does not order automatic retirement, certify untested
     launcher shapes, or accept pre.4/S13 before their remaining gates pass.
     If the comparison leaves the reason to retain the patch unresolved,
     return with that evidence rather than silently choosing option C.

     Source: the exact question/options in
     `Code/testing/mere/receipts/2026-09-29/pre4-s13/pending-question.json`
     and Mark's user message in migration chat
     `01a0e255-9be7-70f3-94d8-4125bfeab06f`, turn
     `01a0ee9b-9d2f-7160-8a46-5bc30a4014fd`. The question file's unanswered
     status describes the pre-answer checkpoint; this ruling records the
     answer. Mere's migration lane owns the comparison, patch decision
     evidence and remaining implementation gates.

411. **Diagnose retained allocations with the zero baseline preserved.**
     Asked after the remote allocator stop, answered on 2026-09-30: "The
     remote reclaim check left 10 active allocations (5,323,776 bytes) after
     400 cleanup polls; its starting baseline was zero. Migration stop rule
     4 requires bringing this back to you. Which next step?" Options:
     "A. Diagnose the retained allocations in a bounded repair lane,
     keeping the zero-baseline requirement; return any ownership or
     patch-design change as a fork (recommended)." "B. Park the pre.4
     migration with its evidence preserved and leave production on its
     current dependencies." Mark answered verbatim: "A!".

     Bounded diagnosis proceeds. The zero-active-allocation baseline stays
     required, and any ownership or patch-design change returns as a fork.
     *Reading, not ruled:* this releases the diagnosis lane, not broader
     migration acceptance, merge, promotion or the downstream repin. The
     retained allocations' cause remains a question for that investigation;
     the answer does not establish the missing numerical or recovery proof.

     Source: `allocator-stop-pending-question.json` and the separate
     `allocator-stop-answer.json` under
     `Code/testing/mere/receipts/2026-09-29/pre4-s13/post-retirement/`.
     The question's SHA256 is
     `b31b1f13b849fcadbc83c9cac50c26731807c1488084128cdd252614a0a06bb0`,
     matching the answer record. Mark's user message is in migration chat
     `01a0e255-9be7-70f3-94d8-4125bfeab06f`, turn
     `01a0f0f8-5004-7243-a6f5-0bd909dbbf3c`. The unanswered status in the
     sealed question remains historical. Mere's existing migration lane
     owns diagnosis and its evidence; no production change is claimed here.
     *Answered 2026-10-03 by ruling 508:* the repair goes in burn-remote's
     close path.

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

414. **An edit reaching across a border is one fact, read across.** Asked on
     2026-09-30: "How does an edit that reaches across a border belong to the
     sites? Evidence: SP1 gives every border a frame relation both sites
     know; SP4's done-condition says 'an edit on a border is seen from both
     sides'." Options, recommended first: (A) "One fact, read across: an edit
     is stored once, in the frame of the site it was made in, with its
     extent; any neighbour it reaches reads it through the border's frame
     relation (recommended)." (B) "Split when made, one part per site." Mark
     chose A, "One fact, read across".

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

421. **A world picks the large preset, 256, by default.** Asked on
     2026-09-30: "Which preset does a world take when its founder picks none?
     Evidence: a largest drawn site (2,048 units) holds 64 patches at 256,
     256 at 128 and 1,024 at 64; the record's §3.7 aims at ten thousand
     places, not ten million, and only lifted sites have places below the
     site." Options, recommended first: (A) "Large, 256 (recommended)." (B)
     "Extra medium, 128." (C) "Medium, 64." Mark chose A, "Large, 256".

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
     ([session notes, §10](2026-09-22_sim_design_session_notes.md#10-the-design-session-2026-09-30)).
     *Reading, not ruled:* the question used "mark" in two senses, the
     effect experiment's drawn glyph effects (`mesocosm-core`'s
     `effect_experiment`) and a symbol standing for a thing too small to
     draw; the answer joins them, so a thing's mark at those rungs shows
     what modifies it.

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

441. **A cost-only century is measured now; the century receipt waits on
     viability.** Asked on 2026-10-01: "How is the century measured before a
     drawn world lives that long? Evidence: today's history run is all dead
     by tick 65, with no births, as on 09-25; 260 stages viability per family
     with M2; S5's done-condition wants a drawn region's century receipted
     (124); the reservoir family, independent processes, ran a
     10,000-member tick in 0.42 s on 09-25." Options, recommended first: (A)
     "A cost-only draw now: a region-sized draw with deaths held off,
     labelled cost and never viability, measures a century's evaluations
     and seconds on today's core; the real receipt waits on 260's gates
     (recommended)." (B) "Wait for viability: no century number until M2's
     families keep a web alive." (C) "The reservoir family: a century on
     the family that already lives, as a lower bound on cost." Mark chose
     A, "A cost-only draw now". So a region-sized draw at ruling 256's
     periods runs a century of world time with deaths held off, and its
     receipt says on every page that it measures cost and certifies no
     viability; S5's century receipt still needs a drawn world that lives,
     which 260's staged gates decide.

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

453. **The probe's minimal body is optional fields on the core part.**
     Asked on 2026-10-01, building checkpoint 6's step 5, after its step 4
     and X3 had landed on the branch: "Where does the probe's minimal body
     live? Evidence: X2's readings and X6's allocation are core extensions
     that read a part's cells, cell mass and half-extents, which Isocosm's
     Part holds none of; my reading in the checkpoint 6 brief kept the
     minimal body in the probe's world until the anatomy brief gives the
     schema its body (281); the 09-25 crowd probe kept its bounds in its own
     world the same way; the step's done-condition wants each native run as
     definitions." Options, recommended first: (A) "Optional fields on the
     core Part: Part gains half-extents, a cell capacity, cells per function
     and cell mass, optional and hashing as before when absent; X2 and X6
     read them in meaning for both runners, the natives run as core
     definitions, and the anatomy brief reshapes the fields after the probe
     (recommended)." (B) "A body in the probe's world: the probe holds its
     own body type and runs the natives through probe-local readings; the
     core's Part and definition widen only after the anatomy brief, so the
     natives run as definitions only then." Mark chose A, "Optional fields
     on the core Part". So the core part carries half-extents, a cell
     capacity, cells per function and cell mass, each absent in parts
     without them; X2's readings and X6's allocation read them for both
     runners; and the anatomy brief (281) may reshape them once the probe
     has shown what the mechanics use. This amends the brief's reading that
     the minimal body lives in the probe's world. *Reshaped 2026-10-02 by
     rulings 459 to 462:* matter in parts, capacity and cell mass derived
     from extents, and each part's attachment kept.

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

456. **The probe's meal takes tissue, and where matter sits in a body goes
     to the anatomy brief.** Asked on 2026-10-01, with 454: "What does a meal
     take from its prey? Evidence: Mesocosm takes the mouthful from the
     prey's substance only; a body's reserve goes back to the ground when it
     dies. Isocosm's Eat takes from every matter account in proportion
     (ruling 287), which 4b's prey fat relies on." Options, recommended
     first: (A) "Tissue only, as Mesocosm: a meal names the accounts it
     takes from. The body family takes tissue only, so a bitten prey keeps
     its reserve. The hunting family keeps its proportional take of body and
     fat (recommended)." (B) "Every matter account in proportion: keep
     Isocosm's current Eat. A prey loses reserve along with its flesh, so
     being bitten also changes how hungry it is, which Mesocosm never does."
     Mark asked: "Could it depend on the part eaten, like the provenance of
     the tissue? Like, where's the reserve stored in the body?" Answered
     that Mesocosm's reserve has no place in the body, being one number on
     the organism; that each part keeps its own mass and provenance mix,
     bites and rent taking from the living parts root first and only
     incorporation taking a named part whole; that which part loses tissue
     changes nothing the five natives read; and that the probe's parts (453)
     hold no matter. Put back: "What does the probe's meal take, given that
     in Mesocosm the reserve has no place in the body and the part a bite
     lands on changes nothing the five natives read?" Options, recommended
     first: (A) "Tissue now, placement to anatomy: the probe's meal takes
     tissue only, as Mesocosm does. The anatomy brief (281) gets an item with
     your question: where tissue and reserve sit in the body, and whether a
     bite lands on one part (recommended)." (B) "Matter in parts now: each
     part holds its own tissue, plus reserve where it stores some. A bite
     lands on a drawn part of the prey and takes from that part. This widens
     ruling 453 ahead of the anatomy brief, for something none of the five
     natives reads." (C) "A storing function: the reserve lives in parts that
     express a new store function in the catalogue. A bite takes reserve
     along with tissue in the share of the prey's cells that store. The
     reserve gets a place without each part keeping its own ledger." Mark
     chose A, "Tissue now, placement to anatomy". So a meal names the
     accounts it takes from, the body family's taking tissue alone and the
     hunting family's keeping 287's proportional take, and the anatomy brief
     (281) carries Mark's question as an item of its own: where tissue and
     reserve sit in a body, and whether a bite lands on one part. *Settled
     2026-10-02 by ruling 459:* tissue in parts, the reserve in stores, a
     bite landing on one part.
     *Cite corrected 2026-10-02:* the proportional take across a prey's
     accounts is ruling 294's, decided from 287's fork; 287 gave each
     consumer one feeding process.

457. **The order from bodies onward stands.** Asked on 2026-10-02,
     reporting checkpoint 6 under ruling 262: "Ruling 195's order from
     bodies onward is bodies, the record, places, lineages and the
     boundary, then effects. The probe ran every native without places,
     but four mechanics wait for places: contract's reach, sense's
     perception, and fixing's forage radius and crowding. Nothing it found
     touches the record. Keep the order?" Options, recommended first: (A)
     "Confirm the order: bodies next, opened by the anatomy brief (281),
     then the record, places, lineages and the boundary, then effects.
     Replay and save reach Isocosm before places, as 195 wanted, and the
     waiting mechanics land with places, third (recommended)." (B) "Places
     before the record: bodies, then places, then the record. The four
     waiting mechanics, and sense with them, come sooner, so the natives
     are complete earlier; replay and save on Isocosm come later than 195
     chose." Mark chose A, "Confirm the order". So 195's order stands from
     bodies onward: bodies, opened by the anatomy brief, then the record,
     places, lineages and the boundary, and effects; contract's reach,
     sense's perception, and fixing's forage radius and crowding land with
     places.

458. **The anatomy brief comes next.** Asked on 2026-10-02, with 457:
     "What comes next?" Options, recommended first: (A) "The anatomy
     brief: ruling 281 writes it after the probe. It carries 456's item,
     where tissue and reserve sit and whether a bite lands on one part, and
     may reshape 453's part fields (recommended)." (B) "Rate models for
     bodies: grouping saves only 1.28 to 1.43x once bodies carry cells.
     Calibrate the rate models of 439 and 440 against the body family on
     the bench, ahead of the families." (C) "Conditions and modifiers (S7):
     briefed in §3.6 and S7, nothing built: condition records beside the
     ledger, layered modifiers, and a cohort's tally of causes." Mark chose
     A, "The anatomy brief". So the anatomy brief is written next, from the
     probe's evidence, carrying 456's item and free to reshape 453's
     fields.

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
473. **WGSL/WESL for raster, CubeCL for compute.** Asked the same day.
     Evidence: all five wing render shaders are WGSL
     (`isometer-lens/src/{tracer,march,grade}.wgsl`,
     `isometer-render/src/{shader,live_body}.wgsl`), against the 2026-08-16
     "author in CubeCL, the brick renderer included" line, and CubeCL 0.10
     has no texture bindings. Options: (A) WGSL/WESL for raster, CubeCL for
     compute (recommended). (B) wgsl-rs once on wgpu 30. (C) rust-gpu 0.10
     for raster too. Mark chose A. This amends the 2026-08-16 line; rust-gpu
     stays for Nexus-derived compute only.
474. **The renderling fork is archived, after L7.** Asked the same day.
     Evidence: L7's done-condition (eponym-client with no renderling
     dependency) is unmet; eponym-client and the `ambience-lease` and
     `field-bake` probes path-depend on the live `crates/renderling` and
     `crates/crabslab`; upstream craballoc 0.4 is unadopted by renderling and
     superseded by crabslab's `feat/wgsl-rs`. Mark chose "Archive the fork
     now", then, told that moving the checkouts before L7 breaks Eponym,
     "L7 first, then archive". So Eponym's Tenant and lighting move off
     renderling, the two probes are archived and the patch rows dropped, and
     then both forks move to `archive/`. Had renderling been kept, its
     residency would have been conatus/CubeCL buffers bound directly
     ("Residency via conatus/CubeCL").
475. **R2 is re-proved in isometer-render.** Asked the same day. The R2
     receipt proved CubeCL output published into a GPU-only renderling
     vertex range, one device and two allocators. Mark chose "Re-prove in
     isometer-render": isometer-render binds conatus's CubeCL buffers
     directly, testing whether the copy and the second allocator disappear.
     The alternative archived it with no successor.
476. **vello for documents, kiss3d 2D for lit games.** Asked the same day.
     Evidence: kiss3d's lit 2D (16 lights), radiance-cascade 2D GI and 2D
     skinning would be a second 2D renderer beside vello. Mark chose "vello
     for documents, kiss3d 2D for lit games": kiss3d's 2D only behind the
     scene contract for a lit 2D game. The alternative was vello only.

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

481. **hecs stays the ECS.** Asked on 2026-10-02 in the Conatus/physics
     session, after Mark said of the ECS "Not bevy, but can we compare the
     potential of the other two?" Evidence: a local bench (Ryzen 9 7940HS,
     release, 10k entities, microseconds per frame) put hecs 0.11.1 against
     shipyard 0.11.5 at iterate 9.6 vs 19.7, spawn and despawn of 1% 5.4 vs
     15, add and remove on 10% 166 vs 53, and listing changes 29 vs 20, all
     under 1.1% of a 16 ms frame; hecs is 9.5k lines with 53 reverse
     dependencies, shipyard 27.9k with 18; both are one-maintainer, MIT or
     Apache-2.0, `Send + Sync` and build for wasm. Options, recommended
     first: (A) "hecs, as ruled (recommended)." (B) shipyard, for cheap
     add/remove and built-in tracking. (C) Prototype both first. Mark chose
     A, keeping 2026-08-16's "hecs, placed".
482. **The mode host and armillary schedule; the ECS is storage.** Asked
     the same day. Evidence: armillary actors per shard, Rayon inside a
     shard and an ordered merge already give the wing its scheduling and
     determinism (§4.7). Options: (A) "Mode host and armillary; ECS is
     storage (recommended)." (B) ECS workloads inside each actor, which
     needs shipyard. Mark chose A: no second scheduler inside the first.
483. **The projection's diff comes from the record's receipts.** Asked the
     same day: where does the record-to-scene-contract projection learn what
     changed? Options: (A) "The record's receipts (recommended)", which the
     authoritative record already emits per tick, so the ECS never becomes a
     second source of truth. (B) hecs `ChangeTracker`, exact but scanning
     every tracked entity. (C) A dirty-flag component. Mark chose A.
484. **A boundary crate in Mere holds the ECS.** Asked the same day.
     Options: (A) "Boundary crate in mere (recommended)": a thin crate owning
     the `BodyId`-to-`Entity` side table (keeping the ECS disposable, never
     mirrored ids), spawn and despawn tied to receipts, a command buffer for
     off-thread writes, the scene-contract diff emitter and per-mode `World`
     construction, with systems using hecs directly inside it. (B) The same
     crate in Mesocosm until a second consumer appears. (C) No boundary
     crate. (D) A full trait over ECSes. Mark chose A, since the projection
     serves every mode.

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

487. **The brief rules its open items before the bodies family's brief.**
     Asked on 2026-10-02, with 485: "What does the anatomy brief still rule
     before S2's bodies family brief is written? Evidence: the brief holds
     16 rulings, and this round makes 20. After it, these stay open:
     variance under growth, which functions a wound takes its cells from,
     and the default systems with the processes that read them. A spread's
     connections also stay open if the placemap reading is not taken. Lanes
     stop at forks, so an open item comes back to you when the build reaches
     it." Options, recommended first: (A) "Write the family's brief next:
     carry the brief into the sim plan's §2.3 and write S2's bodies family
     brief with done-conditions. The open items go to its lane as forks,
     brought back when its build reaches them (recommended)." (B) "Rule the
     rest first: another round on the open items, then the family's brief."
     (C) "The default systems first: a round of their own for the default
     systems, content the family needs from the start, with the smaller
     items handed to the lane." Mark chose B, "Rule the rest first". So
     another round takes the open items before S2's bodies family brief is
     written. Mark added, while his answer to the round's first question was
     being taken up: "After that, default systems!" So the spread question
     comes first, then the default systems.
     *Shaped 2026-10-02 by rulings 501 to 503:* the family is checkpoints 7
     to 11, briefed in the sim plan's §9.

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

490. **The natives read their systems from the bodies family on.** Asked on
     2026-10-02, with 489: "When do the natives start reading their systems?
     Evidence: the probe's natives are certified on bodies founded before
     466, which express no store, gate or conduct: fronds that fix, and a
     grazer's intake lump, contracting rods and sensing point. Read as the
     table lists, a grazer's muscular route would run from its living parts
     to its rods along their attachments, and a wound or a severing would
     narrow or cut it. 477 lets a process require a working system."
     Options, recommended first: (A) "With the bodies family: the natives
     read the systems the table lists from S2's bodies family on. That
     family founds bodies by the recipe (478) with 466's functions and
     certifies them again. Until then nothing certified changes
     (recommended)." (B) "Now, in the probe: rewire the probe's natives to
     read their systems at once, and certify checkpoint 6's domain again on
     regrown bodies." (C) "By world rule: each native's definition names the
     system it reads, if any. The default set names none, so a world opts
     in." Mark chose A, "With the bodies family". So the natives read the
     systems the table lists from S2's bodies family on, which founds bodies
     by the recipe (478) with 466's functions and certifies them again;
     until then nothing certified changes.

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

501. **S2's bodies family is four checkpoints, in order.** Asked on
     2026-10-02, opening the bodies family's brief as 487 ordered: "How is
     S2's bodies family cut into checkpoints (see the preview)? Evidence:
     the anatomy brief holds 32 rulings. 490 has the natives read their
     systems on bodies founded by the recipe, so development comes before
     systems; wounds narrow routes (477), so harm comes after systems.
     Earlier briefs gave graft compatibility to bodies (§9, step 4) and
     reproduction's strategies to the bodies and lineages families (447 to
     450). Checkpoint 6 certified its step exact against the crowd on 1,000
     draws with three controls." The preview, as put:

     | Checkpoint | Builds | Rulings |
     | --- | --- | --- |
     | 7, matter in parts | tissue in parts, reserve in stores, spread in proportion; cells derived; measurements by cell share; names read, no gate | 459, 460, 463, 464, 492 to 494 |
     | 8, development | the lineage's recipe and policy; births by the filial body and the seed; growth toward the recipe once full; segment growth by trait; incorporation teaching the lexicon, with graft compatibility | 447 to 449, 468, 478, 479, 495 |
     | 9, systems | networks read from the tree along attachment offsets, working by degree; the ten defaults, read by the natives | 462, 465, 477, 489, 490 |
     | 10, harm and loss | wounds as lost cells, healed first; severing into bodies; regrowth where the lineage heals; fragments | 469, 470, 485, 486, 496 |

     Options, recommended first: (A) "Four, in this order: matter in parts,
     development, systems, then harm and loss, each certified exact against
     the crowd as checkpoint 6 was. Reproduction's further axes (450) wait
     for the lineages family (recommended)." (B) "Two larger ones: matter in
     parts with development as one checkpoint, and systems with harm and
     loss as the other. Fewer certifications, larger steps." (C) "One: the
     whole family as a single checkpoint, certified once at the end." Mark
     chose A, "Four, in this order". So the bodies family is checkpoint 7,
     matter in parts; 8, development; 9, systems; and 10, harm and loss,
     each with the rulings the table lists and each certified exact against
     the crowd as checkpoint 6 was; reproduction's further axes (450) wait
     for the lineages family.

502. **Territories and surfaces are built in the bodies family, last.**
     Asked on 2026-10-02, with 501: "Where are territories and surfaces
     built? Evidence: 488 makes a mold a territory and micro life a surface.
     They lie in places' ledgers under their keys (497), spread by overflow
     along routes (498), and germs fork strains (499). Sites' ledgers and
     routes exist now. Places, the finer grain of 422, come with the places
     family, after the record in 195's order." Options, recommended first:
     (A) "In the bodies family, last: a fifth checkpoint after harm and
     loss, built at site grain on the ledgers and routes that exist. Strains
     fork lineages as Isocosm already can, and places' finer grain follows
     with the places family (recommended)." (B) "With the places family:
     territories and surfaces wait until places exist, and are built there
     at their finer grain." (C) "A family of their own: mold and micro life
     get their own family, briefed after the bodies family." Mark answered:
     "1 is also possibly important for the concept of aura, if we're doing
     magical systems... so definitely in the bodies family, last". So a
     fifth checkpoint, 11, builds territories and surfaces after harm and
     loss, at site grain on the ledgers and routes that exist, strains
     forking lineages as Isocosm already can, and places' finer grain
     follows with the places family. *Reading, not ruled:* an aura may be a
     territory or a surface applying conditions on its places, within the
     place and site scopes 433 gave modifiers; whether magic uses one is the
     magic design's (404).
     *Given a reference 2026-10-03 by ruling 522:* Ars Magica's realm
     auras, which modify casting where they lie.

503. **The systems checkpoint also riffs.** Asked on 2026-10-02, with 501:
     "Where is riffing new systems by substitution (491) built? Evidence:
     the generator is S5's (§5), founding worlds and running deep time. The
     bodies family certifies on the ten default systems (489). A riff is
     kept only if a body can realize its routes, which needs systems to be
     built first." Options, recommended first: (A) "With the generator: s5's
     generator riffs systems by substitution. The bodies family builds and
     certifies only the ten defaults (recommended)." (B) "In the systems
     checkpoint: the bodies family's systems checkpoint also riffs, and
     certifies a riffed system beside the defaults." Mark chose B, "In the
     systems checkpoint". So the bodies family's systems checkpoint also
     riffs systems by substitution (491) and certifies a riffed system
     beside the ten defaults.

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

505. **Income and the mouthful read their measurements alone.** Asked on
     2026-10-02, with 504: "How do fixing's income (TD2c) and the mouthful
     (TD9) take their measurements (493)? Evidence: both read the body's
     tissue to the three-quarter power today; the mouthful also reads its
     rods' span. Mesocosm's reference segment is 125 voxels, which gives
     length 5, face 25 and volume 125. The probe's fronds are 3 to 4 voxels'
     half-extent across, so their faces are 49 to 81. The grazer's intake
     lump, at [2,2,2], is exactly 125 voxels. Its rods do nothing but
     contract, so taking the rods' span by cell share leaves it unchanged."
     Options, recommended first: (A) "Form scales Mesocosm's rates: keep
     TD2c's and TD9's rates on tissue, multiplied by the function's
     measurement over the same measurement of the reference segment. Fronds
     fix 1.96 to 3.24 times today's rate, capped by room. The grazer's lump
     is the reference size, so its meal is unchanged (recommended)." (B)
     "Form replaces mass: the rates read the measurement alone, at a
     world-rule rate per unit, calibrated so the domain's median body earns
     what it does today. Tissue no longer sets income or the mouthful." Mark
     chose B, "Form replaces mass". So TD2c's income and TD9's mouthful read
     the function's measurement alone, at a world-rule rate per unit
     calibrated so the domain's median body earns what it does today, and
     tissue no longer sets either.
     *Calibrated again 2026-10-03 by ruling 513:* with a reproduce cell in
     each probe body, so the median body still earns what it did.

506. **The probe's grazers store, and its producers stay lean.** Asked on
     2026-10-02, with 504: "How are the probe's bodies founded under 463 and
     466? Evidence: 463 lets only parts expressing store hold a reserve.
     Neither probe body expresses store, yet both keep a reserve account
     today. 466 lets a lump split its cells between taking in and storing.
     Checkpoint 7 is done only when the reserve sits in parts, so something
     must store." Options, recommended first: (A) "Fat grazers, lean
     producers: the grazer's lump splits its cells between intake and store
     by a founding draw. Producers stay single fronds with no reserve, so
     the domain tests both strategies 463 named (recommended)." (B) "Both
     lean: no store anywhere: reserves leave the domain, TD5 builds tissue
     and rent draws on tissue." (C) "Both fat: producers also gain a small
     storing lump beside the frond, a tuber, and the grazer's lump splits as
     in the first." Mark chose A, "Fat grazers, lean producers". So the
     grazer's lump splits its cells between intake and store by a founding
     draw, and producers stay single fronds with no reserve, the domain
     testing both strategies 463 named.
     *Joined 2026-10-03 by ruling 513:* each probe body gives reproduce one
     cell of its frond or its lump.

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

508. **The allocator repair goes in burn-remote's close path.** Asked on
     2026-10-03 in the Burn migration session, as ruling 411's pending
     fork: "pre.4's remote lifecycle gate holds 10 allocations (5,323,776
     bytes) after reclaim until an explicit `client.sync()`, which takes it
     to zero in 2.2 ms; where does the repair go?" Options: "the
     burn-remote close path awaits cleanup completion and propagates
     failures honestly"; "CubeCL's general completion polling changes for
     every consumer"; "park the migration". Mark's answer, as mere's plan
     records it: "burn-remote close path". So the repair goes in
     burn-remote's close path, which waits for cleanup to complete and
     reports its failures honestly. It keeps the zero-baseline gate and the
     strict numerical and recovery gates, preserves a second live lease's
     identity and tensor values, and rejects an injected synchronization
     failure. *Reading, not ruled:* as with 411, this places the repair; it
     does not accept the migration, its merge or promotion, or Isometry's
     repin, and Isometry's handoff, which takes pre.4 through mere, stays
     held.

     Source: mere's burn 0.22 migration plan, §13.31
     (`design_docs/mere_docs/implementation_strategy/2026-08-09_burn_0_22_migration_plan.md`),
     committed at `f560c3b1` on mere's branch `burn-pre4-repin`, not on
     main; the gate and A/B records it re-read are under
     `Code/testing/mere/receipts/2026-10-02/burn-pre4` (remote/, extrema/,
     web/). The migration session relayed both rulings for this record on
     2026-10-03. Mere's migration lane owns the work and its evidence.

509. **A bounded lane makes pre.4's wasm constructors run once.** Asked on
     2026-10-03 with 508: "pre.4 frames take 557 ms against pre.2's 12.1
     ms, GPU on or off, because the module re-runs its static constructors
     on every JS-to-wasm call (pliron's `inventory` registrations via
     `cubecl-core`, upstream, not our patches); native is unaffected, and
     the physics plan's P5 web defaults (N = 9, threshold 400) were ruled on
     pre.2's frame times." Options: "a bounded lane makes the constructors
     run once (a different wasm link model, or a bindgen-side fix), proves
     it with the same A/B, then re-measures P5's web crossover"; "the same
     lane plus an upstream issue"; "vendor-patch the constructor sources for
     wasm"; "park web promotion with native on pre.4". Mark's answer, as
     mere's plan records it: "Bounded fix lane". So a bounded lane makes the
     constructors run once, proves it with the same A/B and re-measures
     P5's web crossover; no upstream issue is filed; and pre.4 is not
     promoted to the web, nor the migration merged to mere's main, until the
     constructors run once and the A/B shows it.
     *Carried out 2026-10-03 by rulings 532 to 534:* one stack helper runs
     the constructors once for every web module, Distillery's probe takes
     it, and promotion waits on a GPU-on A/B taken with the machine quiet.
     *Housed 2026-10-03 by ruling 536:* the helper lives in
     cambium-genet-web-host.

     Source: mere's burn 0.22 migration plan, §13.31
     (`design_docs/mere_docs/implementation_strategy/2026-08-09_burn_0_22_migration_plan.md`),
     committed at `f560c3b1` on mere's branch `burn-pre4-repin`, not on
     main; the gate and A/B records it re-read are under
     `Code/testing/mere/receipts/2026-10-02/burn-pre4` (remote/, extrema/,
     web/). The migration session relayed both rulings for this record on
     2026-10-03. Mere's migration lane owns the work and its evidence.

510. **Attachment offsets are built in checkpoint 8.** Asked on 2026-10-03,
     opening checkpoint 8 (501): "Checkpoint 8 places grown and incorporated
     parts at a plan-resolved attachment (468, 479). Where does that
     placement come from? Evidence: Isocosm's Part keeps its parent but no
     offset, because ruling 462's offsets are scheduled for checkpoint 9.
     Mesocosm's development sets each part's pivot-to-pivot offset: a
     segment sits flush behind the last, and appendages sit in mirrored
     flank sockets. Its growth::resolve tries the plan's facings in turn and
     takes the first box that overlaps no living part, which needs those
     offsets." Options, recommended first: (A) "Offsets in checkpoint 8:
     462's offsets move forward. Development sets each part's offset as
     Mesocosm does, and growth and incorporation find a free box by
     isometer's search. Checkpoint 9 then reads the offsets for routes
     (recommended)." (B) "The tree alone, for now: a grown or incorporated
     part joins the parent its role's preferred facing picks, with no
     position or overlap check. Checkpoint 9's offsets place what already
     exists." (C) "Births only in 8: checkpoint 8 builds recipes and births.
     Growth in life and incorporation wait for checkpoint 9's offsets." Mark
     chose A, "Offsets in checkpoint 8". So 462's offsets are built in
     checkpoint 8: development sets each part's offset as Mesocosm does, a
     grown or incorporated part takes the first free box isometer's search
     finds along the placement policy's facings, and checkpoint 9 reads the
     offsets for routes.

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

513. **The probe's bodies express reproduce in one cell from founding.**
     Asked on 2026-10-03, with 510: "How do the probe's bodies support
     reproduction (449)? Evidence: a body may use a strategy only while it
     expresses reproduce. That's a lump function measured by volume, and
     it's the reproductive system's effector. The probe's producers are one
     frond of 4 to 9 cells, all fixing. Its grazers are one lump of 8 cells:
     1 to 4 store and the rest take in. One reproduce cell would take 11 to
     25% of a frond's fixing area, or 14 to 25% of a grazer's intake,
     against 505's calibrated rates." Options, recommended first: (A) "A
     cell at founding: each recipe gives reproduce one cell of the frond or
     lump, and 505's rates are calibrated again so the median body still
     earns what it did. A recipe without the cell founds checkpoint 7's
     bodies exactly, as a test's control (recommended)." (B) "Grown on
     maturing: founding stays as it is today. A maturing body re-expresses
     one cell as reproduce, at PD2's price, before it can breed." (C) "A
     part of its own: producers bear a seed pod and grazers a lump that
     reproduces, each a part of the recipe, leaving the frond and the
     feeding lump unchanged." Mark chose A, "A cell at founding". So each of
     the probe's recipes gives reproduce one cell of the frond or the lump;
     505's rates are calibrated again so the median body still earns what
     it did; and a recipe without that cell founds checkpoint 7's bodies
     exactly, a test's control.
     *Extended 2026-10-04 by ruling 547:* generated recipes reproduce in
     one cell of their root likewise.

514. **The probe's check certifies all three strategies.** Asked on
     2026-10-03, checkpoint 8's second round: "Which reproductive strategies
     does the probe's check certify? Evidence: checkpoint 8 builds brood,
     the seed or egg, and budding or fission (448; spores wait for places).
     Brood is paid from tissue with a reserve endowment; the seed or egg is
     paid from the reserve; a bud takes a body's overflow at its ceiling.
     The probe's producers keep no reserve (506), so they can bud but not
     seed. Its grazers store, so they can brood or lay eggs. Under 449 the
     phenotype switches among a lineage's strategies by circumstance."
     Options, recommended first: (A) "All three in the check: producers
     bud. Grazers lay an egg while their reserve covers one and brood
     otherwise. The certified check covers every strategy built
     (recommended)." (B) "One each: producers bud and grazers brood. The
     seed or egg is built and tested, not certified." (C) "Producers seed
     from tissue: producers set seed, paid from tissue since they keep no
     reserve, and grazers brood. Budding is built and tested, not
     certified." Mark chose A, "All three in the check". So the probe's
     producers bud and its grazers lay eggs or brood, and the certified
     check covers every strategy built. *Reopened in part the same day by
     ruling 518:* every birth now spends the provision its reproduce cells
     hold, so what switches a grazer between an egg and a brood goes back
     to Mark. *Switched 2026-10-03 by ruling 519:* by hunger, a fed grazer
     brooding and a hungry one laying an egg.

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

517. **The probe runs long enough for births.** Asked on 2026-10-03, with
     514: "How does the probe see births within its run? Evidence: Mesocosm
     matures a 100 mg body at 270 ticks and gestates 480 between births,
     both scaled by the quarter power of mass. The sim has no maturity or
     gestation yet. The probe runs 24 ticks over 1,000 draws, at 3 to 6
     seconds a draw." Options, recommended first: (A) "World-rule tempo,
     fast probe: maturity and gestation become world rules on Mesocosm's
     quarter-power curves, with its bases as defaults. The probe's world
     sets small bases so bodies bear, and children mature and bear, within
     its 24 ticks. Founders start mature at staggered gestation
     (recommended)." (B) "A longer probe: Mesocosm's tempo, with the probe
     running long enough for births, about 750 ticks. That's about 30 times
     the cost of a check." (C) "Founders due at founding: Mesocosm's tempo,
     with founders' gestation clocks drawn so some are due within 24 ticks.
     Only founders bear; their children never do." Mark chose B, "A longer
     probe". So the probe's domain is not given a tempo of its own: it runs
     as long as the world's own tempo needs for births to happen. *Reading,
     not ruled:* under 518 a birth comes when a body has filled its
     provision, tens of ticks at the probe's incomes, so the run's length
     is measured from that, not from Mesocosm's 750 ticks.

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

521. **Semelparity and parental care come with development; mating and
     horizontal transfer wait for the boundary family.** Asked on
     2026-10-03, with 519: "Where are 450's axes built? Evidence: 450 ruled
     mating, semelparity or iteroparity, parental care and horizontal
     transfer. The bodies family's brief places 447 to 449 in checkpoint 8
     and does not place 450. Mating needs two genotypes to meet and spreads
     57's reroll, which lives at the epoch boundary. Semelparity is a trait
     (one brood, then death). Parental care is a feeding transfer, and
     horizontal transfer a symbiont's edge counted as eating." Options,
     recommended first: (A) "Split by home: semelparity and parental care in
     checkpoint 8, since they're body-level. Mating and horizontal transfer
     go with the boundary family, where genotypes and the reroll live
     (recommended)." (B) "All in checkpoint 8: all four are built and
     certified with development." (C) "All with the boundary: all four wait
     for the boundary family." Mark chose A, "Split by home". So
     semelparity or iteroparity and parental care are built and certified
     in checkpoint 8, and mating and horizontal transfer are built with the
     boundary family, where genotypes and 57's reroll live.
     *Drawn in the probe 2026-10-03 by ruling 529:* a share of each
     lineage's cohorts are semelparous.

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

528. **The probe's grazers give milk.** Asked on 2026-10-03, checkpoint 8's
     fifth round: "Which care do the probe's grazers give (527)? Evidence:
     milk spends the refilling provision, one reproduce cell of 12 mg in a
     grazer, so care trades against the next birth. Mouthfuls spend the
     reserve (1 to 4 store cells of 12 mg each), sized by intake volume
     (median mouthful 11 mg), and cost no births." Options, recommended
     first: (A) "Milk: the probe's grazers nurse, exercising 518's provision
     past the birth, and care trades against breeding. Mouthfuls are
     tested, not certified (recommended)." (B) "Mouthfuls: the probe's
     grazers regurgitate from the reserve. Milk is tested, not certified."
     Mark chose A, "Milk". So the probe's grazers nurse their hungry young
     from their refilling provision, certified, and mouthfuls are tested.

529. **The probe draws semelparity per cohort.** Asked on 2026-10-03, with
     528: "Does the probe's check certify semelparity, one brood and then
     death (521)? Evidence: 514 put every strategy built into the check. The
     probe already draws traits per cohort: a quarter of producer cohorts
     can grow a gland. A semelparous body dies when its birth is done; an
     iteroparous one breeds again." Options, recommended first: (A) "Drawn
     per cohort: as the gland candidate is, a share of each lineage's
     cohorts in each world are semelparous, so the check covers both kinds
     of body (recommended)." (B) "Producers semelparous: every producer
     dies when its bud severs, as annual plants do. Grazers breed many
     times." (C) "Tested, not certified: semelparity is built and tested,
     and the probe's lineages all breed many times." Mark chose A, "Drawn
     per cohort". So a share of each probe lineage's cohorts in each world
     are semelparous, dying when their birth is done, and the check covers
     both kinds of body.

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

533. **Distillery's model probe takes the fix; the minimal repros do not.**
     Asked on 2026-10-03, with 532: "Distillery's model probe (which
     records browser timings against bounds) and two minimal repros (burn
     browser embedding; extrema, 3,648 wrapped exports) are still
     command-linked. S13(b) extrema passes either way." Options: "the probe
     takes the fix and the repros stay minimal"; "all three"; "none".
     Mark's answer, as mere's plan records it: "Probe yes, repros no". So
     Distillery's model probe calls the helper, and the burn embedding and
     extrema repros stay minimal, command-linked.

534. **Promotion waits on a GPU-on A/B taken with the machine quiet.** Asked
     on 2026-10-03, with 532: "pooled medians on a busy machine (other
     sessions up to 96% CPU): GPU off, fixed pre.4 12.2 ms against pre.2
     12.2 ms; GPU on, 21.2 ms against 15.2 ms, one vsync above, ranges
     overlapping (12.1 to 30.2 against 12.1 to 30.3), the profiles showing
     no pre.4 code; the 2,000-node settle 6.8 against 6.3 ms; P5's
     crossover still between 256 and 512 nodes." Options: "accept that
     ruling 509's condition is met for graphshell-web"; "rerun the GPU-on
     boot A/B on a quieter machine before promotion". Mark's answer, as
     mere's plan records it: "Rerun GPU-on quieter". So pre.4's promotion
     (S16) waits on a GPU-on boot A/B taken with the machine quiet. *Reading,
     not ruled:* "quiet" is shown by recording the CPU load beside each
     repetition, and the GPU-off result and P5's crossover stand.

     Source: mere's burn 0.22 migration plan, §13.34, with §13.33's
     evidence, committed at `cc91e3e8` on mere's branch `burn-pre4-repin`,
     not on main. The migration session relayed the three rulings for this
     record on 2026-10-03. Mere's migration lane owns the work.
     *Carried out 2026-10-04 by ruling 555:* the quiet is the ambient load
     bounded before each launch, outside the measured window. *Promoted
     2026-10-04 by ruling 557:* the quiet A/B found the two indistinguishable
     in frame pacing.

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

545. **The web modules take wasm-bindgen 0.2.129 with wgpu 30.0.1.** Asked
     on 2026-10-03 in the Burn migration session, carrying out 537: the
     newest wasm-bindgen, 0.2.129, still crashes with wgpu 30.0.0 once per
     GPU-on page, since from 0.2.126 its `JsOption` treats only
     `undefined` as no error while the browser answers `null`; wgpu
     30.0.1 reads it through `JsNullable` and needs 0.2.127 or later, and
     the root lock already takes 30.0.1. In one session 0.2.127 and 0.2.129
     crashed on 30.0.0 and 0.2.129 ran clean twice on 30.0.1, the
     2,000-node settle within its bounds; graphshell-web on mere's main has
     the crash today. Options: (A) "wasm-bindgen 0.2.129 with wgpu 30.0.1
     in graphshell-web, `cambium-genet-web-host` and the probe. That is one
     wgpu across the root, web and probe graphs. It removes the silent
     panic graphshell-web has on main today, and the helper and the probe
     dependency proceed as ruled (recommended)." (B) "wasm-bindgen 0.2.129
     with wgpu 30.0.0. Every GPU-on page keeps panicking once, and the
     probe's rows would fail on the worker error." (C) "all three at
     0.2.122. That is an older version, and wgpu 30.0.1 cannot take it, so
     the root lock would have to drop to 30.0.0 as well." Mark's answer, as
     the migration session relays it: "0.2.129 + wgpu 30.0.1". So
     graphshell-web, cambium-genet-web-host and Distillery's model probe
     take wasm-bindgen 0.2.129 with wgpu 30.0.1, one wgpu across the root,
     web and probe graphs, and the helper proceeds as 536 ruled. *Reading,
     not ruled:* Isometry's web build reaches the same pins through
     Cambium's web host.

     Source: the finding and the options are in mere's burn 0.22 migration
     plan §13.37, at `88897eb6` on branch `burn-pre4-repin`, its evidence
     under `Code/testing/mere/receipts/2026-10-03/pre4-bindgen`; Mark's
     answer was relayed by the migration session on 2026-10-03, for §13.38.
     *Widened 2026-10-04 by ruling 556:* the OPFS probe and the minimal
     repros take 0.2.129 too.

546. **The Distillery probe's decoder model may be downloaded as stated.**
     Asked on 2026-10-03 in the Burn migration session: "approve
     downloading HuggingFaceTB/SmolLM2-135M-Instruct at revision
     12fd25f77366fa6b3b4b768ec3050bf629380bac (Apache-2.0) from Hugging
     Face. That is config (861 B), tokenizer (2.1 MB) and weights (269 MB,
     BF16), about 271 MB in all, each checked against the SHA-256 already
     pinned in the probe's `decoder-model.json`, and stored outside the
     repo. It is for the Distillery probe's decoder row on pre.4." Options:
     "approve as stated"; "skip the decoder row for this migration".
     Mark's answer, as the migration session relays it: "Approve as
     stated". So that model, at that revision, may be downloaded from
     Hugging Face as stated, each file checked against its pinned SHA-256
     and kept outside the repository, for the probe's decoder row.

     Source: relayed by the migration session on 2026-10-03, for mere's
     burn plan §13.38 on `burn-pre4-repin`.

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

553. **The probe's soil is drawn wider.** Asked on 2026-10-04, with 552:
     "How rich in soil are the probe's worlds? Evidence: a world's matter
     is a median 94% of what its producers' bodies can hold (57% to 184%),
     before grazers take their share. Without grazers, soil falls to 5 to
     46 mg and fronds stall below their ceilings, so only soil-rich worlds
     ever provision. With provision-sized buds at 60 ticks over 100 worlds,
     sites of 200 to 2,400 mg give bud births in 8 worlds and eggs in 37;
     sites of 200 to 12,000 mg give buds in 31 and eggs in 64. Broods (95)
     and milk (71) are the same either way, and a draw costs 2.2 to 2.3 s."
     Options, recommended first: (A) "Wider soil: each site's soil drawn
     from 200 to 12,000 mg instead of 200 to 2,400, so the domain spans
     starved and surplus worlds. Measured above: buds in 31 of 100 worlds
     (recommended)." (B) "Soil scaled to bodies: each site's soil drawn as a
     share of its founders' adult mass, lean to rich (for example 50% to
     500%), whatever bodies a world draws. Not yet measured." (C) "As
     founded: soil stays 200 to 2,400 mg a site. The check certifies buds
     over the draws where they happen: about 8 in 100, roughly 80 of its
     1,000." Mark chose A, "Wider soil". So the probe draws each site's
     soil from 200 to 12,000 mg, its domain spanning starved and surplus
     worlds.

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

555. **The quiet A/B bounds the load before each launch.** Asked on
     2026-10-04 in the Burn migration session, carrying out 534: the quiet
     GPU-on A/B cannot meet its load bound on this machine. It counted 0 of
     80 repetitions, all valid; each measured window includes the measured
     Chrome's own launch and WebGPU page, and an idle minute with nothing
     of the lane running read a median of 15% and a maximum of 56%.
     Uncounted, for the record: frame p95 12.2 ms on pre.2 against 18.2 ms
     on pre.4, and time to ready 1,273 ms against 1,364 ms. Options: (A)
     bound the ambient load just before each launch, outside the measured
     window; (B) bound the load excluding the lane's own process tree; (C)
     keep the bound as stated and run on another of Mark's machines. Mark's
     answer, as the migration session relays it: "Bound load before
     launch". So only repetitions that start quiet are counted, the bound
     on the ambient load before each launch stated before the run, and
     promotion (S16) still waits on the result.

     Source: the finding and the options are in mere's burn 0.22 migration
     plan, "Quiet GPU-on A/B, ruling 534" and "Returned as forks", in the
     working copy of branch `burn-pre4-repin` (after `44500bca`), its
     evidence under `Code/testing/mere/receipts/2026-10-04/pre4-quiet-ab`;
     Mark's answer was relayed by the migration session on 2026-10-04,
     for mere's plan to record.
     *Answered 2026-10-04 by ruling 557:* 7 pre.2 and 8 pre.4 repetitions
     started quiet, indistinguishable in frame pacing, and pre.4 is
     promoted.

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

557. **pre.4 is promoted, then the handoffs follow.** Asked on 2026-10-04
     in the Burn migration session: every gate holds. The allocator is
     fixed (508); the wasm constructors run once (509, 532, 536); the web is
     on wasm-bindgen 0.2.129 with wgpu 30.0.1 (545, 556); the probe's
     embedding and decoder rows pass (546). The quiet A/B under 555 counted
     7 pre.2 and 8 pre.4 repetitions: frame p50 12.1 against 12.05 ms, p95
     12.3 against 12.25 ms, ready 1,369 against 1,389 ms. Main carries the
     pre.2 lock, and promotion swaps it for pre.4's, about 1,678 packages.
     Options, as the migration session relays them: promote, then the
     handoffs; hold. Mark's answer, as it relays it: "Promote, then
     handoffs". So the coordinator verifies and merges pre.4 into mere's
     main (S16), and the Knot and Isometry repins follow as their own
     steps, Knot first if anything breaks, per the lockstep. *Reading, not
     ruled:* Isometry's repin is a step of its own, taken from that handoff.

558. **The web workspaces commit the getrandom cfg.** Asked on 2026-10-04
     in the Burn migration session, with 559 as one multi-select question:
     every receipt bundle so far was built with `--cfg
     getrandom_backend="wasm_js"` inherited from the batch's environment, so
     a plain-shell build at the same commit makes a different bundle. Mark
     ticked "Commit the getrandom cfg". So the flag goes in the web
     workspaces' committed cargo config.

559. **The OPFS probe takes its rustfmt sweep.** Asked with 558: the
     muniment OPFS probe's runner stops at `cargo fmt --check`, because its
     sources predate the 2026-09-04 rustfmt policy, which wants any sweep as
     its own commit. Mark ticked "Sweep fmt over the OPFS probe". So the
     sweep goes in its own commit, with its hash in
     `.git-blame-ignore-revs`.

     Source for 557 to 559: the findings are in mere's burn 0.22 migration
     plan, the quiet A/B in §13.40, the OPFS probe's open fmt question at
     the end of §13.41 and the two bundles from one head in §13.42, at
     `6b6a43a0` on branch `burn-pre4-repin`; Mark's answers were relayed by
     the migration session on 2026-10-04, for mere's §13.44.

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
| Space (places) | cell, site, region, continent, world, system | Rulings 11, 12, 72; a site is one cell of the world map, a region is an area of sites and is terrain, and a location is a place of note at any extent (§3.7.1) |
| Fields | conditions on places; effects left by processes; reach of events | Ruling 5; world state is not a kind, it is the fields a place carries |
| Time | the due-event clock; deep time before handover; forks and branches | Rulings 3, 7, 126 |
| Provenance | seed, deviations, asserted facts, the significant record | §1; ruling 4 |

Things and magic sit inside these: an item is a body without agency and a
relic per Law A; a glyph is magic's vocabulary and magic is a process that
bends the rules the others run under (ruling 10). (Ruled 2026-09-24, ruling
144: an item can also be a sophont, made so by its maker, awakened by its
story, or inhabited by a mind that moves into it; then it has agency, and
its provenance stays a maker's or its own story's while its identity is a
sophont's, ruling 45's two axes. Rulings 145 and 146: it acts through its
wielder, by its own powers, and when strong enough by taking its wielder
over; and it is never owned, only held, and can own things itself. Ruling
151: for any person, owning one is slavery, and treating people as chattel
is itself a tenet, held or rejected, and judged by whether one is the same
kind of thing as the enslaved. *Reading, not ruled:* the two agree, since
the record never holds a person as plain property; where a person is
"owned", that is slavery, a claim upheld by those who hold the tenet and
enforced like any polity's means.)

### 3.2 Agents and state

State follows methodology (ruling 9). A creature decides and acts and has
state. A faction is a relation among creatures, allegiance to a person, a
place or an idea, and its action is theirs; a party is a faction defined by
a person. A polity has a collective action methodology and so has state of
its own, over many factions. A lineage is a historical entity of kith and
kin: a record, not a decider. A settlement is a faction until it acquires a
methodology, which ruling 63 sharpens to a constitution: a faction has a way
of acting together too, by consent, and §3.2.2 holds the difference.

Consequence for the scheduler: things with methodology get due events;
relations, records and fields cost nothing on their own.

### 3.2.1 The creature at three levels of identity

From ruling 36, with what the wing already holds cited beside it. This is
the first piece of W2's schema and is open where marked.

**Critter.** Kingdom, which is class: the founding plan's three lineage
strategies, producer, consumer and decomposer, and the top of the
provenance typing nis already carries. Genotype: traits in relation, with
adjacency effects and process conditions, the shape of Balatro's jokers;
the earlier form of this is the allocation mosaic ruled 2026-08-01 in the
processdef plan, an authoritative graph of capacity cells per part where
tracts (ruling 157) occupying adjacent cells cooperate, interfere or hybridise, and Mark
is open to another form. Phenotype: conditioned expression of the
genotype, the critter's hand to play, with unconditional and conditional
abilities that depend on circumstance, condition, status and activity (a
malnourished carnivore becomes an omnivore or a cannibal; the moon makes a
werebeast). Biology's names for this are phenotypic plasticity and
reaction norms, and polyphenism for the discrete cases: a locust turns
gregarious under crowding, an aphid grows wings, an ant larva becomes a
queen. The epoch boundary plan's ruling that plasticity is a life stage
that youth pays for is one conditional expression already ruled.

**Denizen.** An inhabitant the simulation remembers individually because
history, relationships, or explicit designation makes it matter. A
lineage-born denizen inherits its lineage's genotype and expression; other
provenance still has the same remembrance tier. A denizen is harder to collapse
into a cohort, being more individual. Earned notability does not require sapience, a name, a
faction, or costly foreground simulation. A denizen can therefore be a
non-sapient critter with a remembered relationship or event. This current
definition supersedes the provisional `borg` definition on 2026-09-20; the
dated rulings that use `borg` remain historical records. **Amended by ruling
200 (2026-09-25):** an entity of note is one the sim identifies, and it
becomes a denizen when people name it, as they need to refer to it. Naming
requires sapience, so every sophont is a denizen and a denizen need not be a
sophont; notability itself still needs no name. The tier is of note.

**Character.** A denizen contingent on a group, a polity or a collective,
even by absence. Defined by the tabletop system's schema, and resolvable
as a denizen and as a critter.

**Divinity and provenance (ruling 42).** Divinity is not a rung of the
agent ladder and not a mere status: it is promotion into the world's
provenance itself. Under §1 a world is a seed, its rules and its asserted
facts, and a fork or a branch inherits all three; a divine thing is an
asserted fact moved into that root, which is why it is present in every fork
and branch after its
ascendance by default, and why anything can hold it, a creature, a place
or an item. It sits beyond the hagiograph's top rank: the hagiograph
judges what is unprecedented, legendary or narratively significant, but
divinity is *enacted*, the outcome of a wish with the world's strength,
which is a rung transition of §3.3's third shape whose cost is
world-scale, such as gathering and invoking the world's whole glyph
canon. That answers question 8 for the top: what persists past a body is
the record always, a denizen's memory only if retold, a character's sheet as
an asserted fact, and a divine thing entirely, as provenance.

**Presence and ending (ruling 43).** The ascent writes two terms into the
world's rules beside the divine thing's identity: how it is present, and
how it can end. Presence has at least three modes. *Omnipresent:* one
instance, the same anywhere and everywhere, which in the sim is a
world-scope field or process rather than a body, the one true god of
lightning being a condition every place carries. *Reincarnating:* a cycle
of recreation, the chain of avatars, which is a lineage whose program
re-founds the divine thing on each death, a rung transition the world's
rules guarantee. *Fixed:* a body or a place the world depends on, a
worldtree, which is a terrain-body under ruling 39 with the world's own
processes bound to it. Ending is by prophesied conditions related to the
ascent: the only transition that can remove a divine thing is one whose
preconditions were asserted when it rose, so a prophecy is a rule in the
provenance root, checkable like any other, and "destroyed only through
prophesied conditions" is the derivation rule's guarantee that nothing
derived can erase what was asserted at the root. Prior art, unverified:
Dominions' pretender gods embodied as units and recalled after death;
Elden Ring's Erdtree as a place the world depends on; Dark Souls' First
Flame as a cycle; Pratchett's small gods, whose strength is belief, for
"a wish with the world's strength"; Cultist Simulator's ascensions as
the enactment.

**The wish, and the tiers (ruling 44, with what §7.4 of the general model
plan already rules).** The wish is the ascension the glyph organ already
defines: you must have the glyphs to ascend using them; a glyph is had, by
a trait or part in a critter, by an item a sophont may hold and sacrifice at
ascension, or by a carving on a place or thing that cannot be sacrificed;
and every glyph must have been experienced, through conditions that
trigger off the log of significant events. An individual has a journey,
the accepted order of every acquisition with its means and evidence, and
the ascension basis retains the exact grant prefix at first ascension. The
general model plan already says the same collection produces different
divinities according to how each glyph was acquired. Ruling 44 grades
that: the tier of godhood reached is a function of how difficult the
acquisition conditions fulfilled were, across the canon, and the tiers
are an ordering of ruling 43's ending terms:

| Tier | Body | Identity |
| --- | --- | --- |
| Demigod | ageless but vulnerable | one life; ends at death |
| Avatar | vulnerable | reincarnates on death |
| Avatar, greater | ageless, vulnerable | reincarnates on death |
| Greater divinity | cannot be killed | ends only by prophecy |

So the end conditions are dictated by the journey, not chosen. What the
god chooses, per §7.4, is its referent, the source and measurement period
of its power.

**How the grade is taken (ruling 46).** Each glyph sacrificed at ascension
scores by the form that bore it: an item held, the lowest, for a glyph
acquired; a technique, skill or ability, the middle, for one learned; an
embodied trait, the highest, for one represented in the body. The tier of
godhood is the aggregate of those grades across the canon. That is the
glyph expression ruling of 2026-09-15 read as a scale: a glyph is had by
a trait or part, by an item, or by a carving, and the forms now rank. Two
consequences. Mesocosm's critters, who can only embody, take the highest
grade per glyph, so the wildlife route to divinity is the strongest one,
which suits the vessel briefs' primordial-name hook. And a carving cannot
be sacrificed, so a place's ascent either counts its carvings without
consuming them, which suits a worldtree keeping its marks, or a place
ascends by another party's sacrifice on it; the hagioglyph organ decides
which. Still open: whether the presence mode of ruling 43, omnipresent,
reincarnating or fixed, is dictated by the tier, chosen by the wisher
within it, or follows the referent; Mark's answer of 2026-09-18 addressed
the grade rather than the mode, so the mode stays with the organ's plan.

**The grade, amended by ruling 107.** Ruling 46 graded a sacrificed glyph by
the form that bore it. Ruling 107 adds the second factor and makes the
ladder of forms derived, as D30 proposed: the forms a world offers are
whatever bearer forms its magic systems' gate axis admits, plus memory
always, ranked as ruling 46 ranks them; and the *quality of the offering* is
the depth of the entity's relation to that facet of the world's magic in its
story, "how many things it's related to". In the record's terms that is the
count of notes and cause-links joining the entity to uses of that facet, the
same reachability ruling 71 collects by, read from the impresa and the
journey. So one magic "used religiously" with great things accomplished by
it grades as high as a wide collection, and a trinket used once grades as a
trinket. Whether ascension needs a threshold of potency or the whole canon
is a world setting in ruling 102's flow; the favoured default is the whole
canon, "even trinkets plus your main magic", because it "links ya to the
world".

**Sacrifice and domain (ruling 47).** To sacrifice a form is to destroy
it: burn the carved sticks, spend the bought items, unmake the embodied
trait, and, if a way can be found, destroy the memory. The **tier** is the
aggregate grade of the forms destroyed (ruling 46). The **domain**, what
the divinity is *of*, is the dominant means by which those forms were
acquired, read off the journey: carve the glyphs into sticks and burn
them and you are a demigod of carving; buy them all and you are a demigod
of buying; destroy your memories of them and you are a god of memorizing.
The mix of forms may be mixed, items, skills and the body together, so a
divinity's domain is a distribution over acquisition means rather than
one word, and the journey's recorded means, ability, trait, technique,
item, bond, quest, event, or a mod-defined category per §7.4, is exactly
the evidence it reads. This resolves the general model plan's
"same collection, different divinities" into a rule: forms decide the
tier, means decide the domain. Memory is thereby a fourth bearer form
beside item, technique and trait, the one a sophont has and a critter
does not, and its sacrifice is forgetting.

**Divine places, placement and alignment (ruling 48).** A place ascends
without sacrifice: all the glyphs existing there is enough to make it a
divine environment, so a place's ascent is by presence rather than by
destruction, which answers the carving question above. A glyph placed
somewhere influences the place, and the strength of that influence is
the placement's durability and quality. That makes placement a reading
over the place rung: when buildings and rooms exist, a room has a type and
an impressiveness, evaluated as RimWorld evaluates its rooms from space,
wealth, beauty and cleanliness. And an **alignment system**: any thing
may relate positively or negatively to a glyph, a symbol, a faction or a
polity, according to **tenets** associated with effects. In the record's
terms a tenet is a rule a faction or polity holds that reads events and
effects in the record as approval or disapproval, and alignment is the
derived valence of the relation, never asserted, which is Law A's chain
from choices to values to ethos to faction with the reading rule named.
Prior art, unverified: RimWorld's Ideology, where memes yield precepts
that turn events into approval and mood; Dwarf Fortress's civilization
values and ethics; Crusader Kings III's religious tenets, whose word this
is; and Cultist Simulator's aspects as the affinities of things to
principles. Tenets are a W2 question for the faction and polity rungs:
who holds them, whether a sophont may hold its own, and how a glyph
placed in a room reads under them.

**Glyphs, effects, processes and the three quantities of a divinity
(ruling 49).** A glyph is an effect the world recognises as fundamental
and gives its own manifestation: slicing, burning, persuading, forgetting,
lightening, flying, protecting, healing. A process is aligned with an
effect to the degree it produces that effect, which is a frequency read
off the record, never asserted. A divinity then has three quantities,
all from the journey and the acts:

| Quantity | Comes from | Ruling |
| --- | --- | --- |
| Tier, how it can end | the forms sacrificed | 44, 46 |
| Domain, what it is of | the means by which those forms were acquired, blended with the effects they bore; named freely at ascension, the name deciding nothing | 47, 49 |
| Power, how strong it is | how often the process its effect is borne on happens in the world, weighted by the significance of what it causes: a rare process is mitigated when its outcomes are unprecedented or cause something significant, which is the hagiograph's own test of ruling 4 applied to power | 49, 52 |
| Referent, what is measured | fixed as the domain's effect for a demigod and an avatar; chosen, per §7.4, by a greater avatar or a greater divinity, who are still bound by frequency and impact | 52 |

"You have to be careful": a god of a rare effect is a weak god. The other
road to long life is anatomy rather than provenance: a vampire, a lich, a
fey or an eldritch thing instantiates a particular anatomy, magical or
conditional as a phylactery is, and that is ruling 7's longevity by
traits and ruling 39's body forms, not divinity.

**A tenet (ruling 50), and the gap Mark named.** A tenet is the relation
that a process yields an effect, together with the holder's opinion of
that and the trust the holder places in it for survival or fulfilment,
where trust is an expectation formed from how things turned out before,
held at the cohort rung as a distribution with variance and at the near
rung per sophont. Mark noted the definition is "missing attitude towards
how things happened". Ruling 47 already carries that dimension: the
journey records the *means* of every act as well as its effect, so a
tenet reads pairs of means and effect, not effects alone, and "killing in
the manner you favor" is a tenet's opinion of a means. The three parts
are then: a process-to-effect relation, an opinion of the effect and of
the means, and a trust with variance.

**Alignment by rung (ruling 51).** A sophont's alignment is derived from
its acts: what it does aligns it with domains, tenets and pursuits, and a
god of war aligns with slicing, piercing and bludgeoning and so with a
sophont who kills. A faction's alignment is derived as the weighting of
its members' alignments by each member's influence in the faction, which
is exactly the relational entity of ruling 8 with the weights named. A
polity's alignment is asserted, constitutional, in addition to the
factions within it, which is exactly ruling 9: a polity has state because
it decides, and its constitution is asserted state. So alignment is
derived at the sophont and faction rungs and asserted at the polity rung,
and the derivation rule holds at each. **Amended by ruling 79:** a polity's
alignment is derived from its acts too, its decisions among them; the
constitution remains an asserted fact (§3.2.2).

**The tension with §7.4, resolved by ruling 52.** Choosing a referent is
a privilege of the two greater tiers; a demigod's and an avatar's referent
is the domain's effect. Every tier is bound by the frequency of the
process its power is borne on, and frequency is weighted by impact: a
rare process is mitigated when applying its effect is unprecedented or
causes something significant, so the hagiograph's significance test
enters divine power as its second factor. §7.4's period mechanics stand
unchanged for whichever referent applies.

**Two axes (ruling 45, proposed and then agreed the same day):** Mark's
"0th tier" and "4th tier" are the two ends of one axis that is not the
identity ladder. Provenance has three kinds: born of a lineage (a critter), made by
a maker (a construct: a golem, a mech, an automaton, a raised corpse),
and intrinsic to the world (the divine). Identity has three levels:
unnamed, named and sapient, and factional. The two are orthogonal, so a
construct may be a person (an android), a divine thing may be a place,
and Lancer's mech is a construct held by a person. That keeps the ladder
at three levels and gives constructs and the divine a home without
making either a rung. Agreed by Mark as ruling 45.

**Terminology supersession (2026-09-20).** The prior naming round is closed:
**denizen** is the second simulation tier, an inhabitant individually
remembered because history, relationships, or explicit designation makes it
matter. It does not imply sapience, a name, a faction, or expensive simulation.
The platform admission sense previously carried by *denizen* is now
**participant**: an identity holding a grant with the right to petition,
including human moot peers, servitors, and scenario runners. The `denizen`
crate name and concept are reserved for the simulation tier; this does not
create that crate or move Servitor implementation. The 2026-09-18 proposal to
move `borg` to the construct axis remains an unresolved proposal, not a
ruling. Its dated wording is superseded here; historical quotations retain it.
**Amended 2026-09-25 by ruling 200:** the tier is of note, an entity the sim
identifies; a denizen is a named entity, named as people need to refer to
it, and every sophont is one.

**Kingdom and scale (ruling 39).** Kingdom is class and is a trophic
strategy: producer, consumer, decomposer, as the founding plan ruled. Scale
is a separate axis: micro, meso, macro. Ruling 100 adds *world* as a
kingdom, the root of provenance, at macro scale by default and open at meso
and micro (§3.3.1). A germ is a decomposer or parasite
at the micro scale whose body is not a part tree but a spread, a field
over a substrate or a host, that thrives or dies by what it lies on and
decomposes corpses; the founding plan's fungus, "networked, patient,
spreading through the dead", already needed that body form, and the taste
record's intelligent fungus is a colony played as an individual. A macro
creature is a body large enough to be terrain, a lion turtle: its volume
is walkable, places are derived on it, and it is at once a creature on
the agent ladder and a place on the space ladder, which ruling 11's "a
world that may be an entity" already allows at the largest scale.
Consequence for the body noun: a body is a part tree (animal), a spread
over a substrate (germ, fungus, colony), or a terrain-body (macro), and
the three body forms share the ledger and the record but not the part
model. *Amended 2026-10-02 by ruling 467:* one part model serves all
three, a part attaching to a parent or lying on a site. *Amended again by
ruling 488:* a mold is a territory and micro life a surface. **Ruled
2026-09-19 (ruling 58): a spread is a body,** and the
kingdom decides how many. A fungus is one body even when its patches are
separate: a distributed, collective organism with a long life, in which
there is no individual, only the instance of the whole, so its regions are
its parts and splitting it does not make two of it. A germ is many bodies:
a population of tiny lives turning over so fast that a played germ burns
generations while spreading, and revises its genes on the fly according
to host and environment. That is the unique micro advantage, and in the
record's terms it is the two doors of ruling 57 merging: for a germ the
epoch boundary comes every generation, the shop is open during the round,
and to play a germ is to play the lineage directly. Both kingdoms have a
genotype and a phenotype in the animal's sense; what differs is the count
of bodies one identity spans. *Ruled 2026-09-24 (ruling 155):* so what a
Mesocosm player plays follows from its traits. A player directs only who
they play (ruling 152), and a critter's relation to the rest of its lineage
depends on its traits: at one extreme its kin are its ecological
competitors, at the other they are extensions of its own critter, a fungus
monocreature or a swarm of germs, which a player directs whole.

**What the record's own structure says a creature must also carry, put to
Mark as questions rather than filled in (open, 2026-09-18):**

1. *State.* **Answered, ruling 38.** State is one ledger in the sim, the
   conserved material ledger Mesocosm keeps with injuries, age and stage
   (Dynamic Energy Budget theory's reserve, structure and maturity as the
   model shape), and each level reads it more coarsely: a critter can die
   of a missing essential nutrient, a denizen of starvation generally, a
   character is debuffed for not eating under its system's rules and
   expresses sleep as exhaustion. A character's sheet is not a second
   state; it is the system's coarse reading of the one ledger, and the
   system decides what counts as food. The coarsening is diegetic: the
   more factions and polities there are, the more the essentials are
   produced and commoditised, so the collective's processes provision the
   individual and the fine detail stops mattering. That makes provisioning
   a process of the faction and polity rungs whose output is the coarse
   reading the higher levels use, and it is what "weakly expressed"
   (ruling 35) means for the ecology in Eponym and the VTT.
2. *Methodology.* **Answered, ruling 37.** Reactive agents for critters,
   belief-desire-intention agents for denizens when their chosen fidelity
   needs them, normative agents with roles and institutions for characters.
   A denizen's disposition is the five-factor axes when represented, and
   significant events may also cause traits, as Crusader Kings does. (Ruled
   2026-09-24, ruling 120: a disposition is inherited through the lineage
   and lived; the world entity's disposition seeds only the first
   lineages, and its direct pull fades after. Ruling 121: the world keeps
   shaping its people through condition, scarcity or plenty, its
   intentions, its cycles, what it is made of, and they may drift from its
   temperament. Ruling 158: personality traits are a class of traits of
   their own, inherited in seed form and grown by acts that feed them, each
   compounding modifiers on other mental and physical aspects, some
   precluding an aspect of play for a bonus, as RimWorld's Brawler does.
   Ruling 160: they sit on the five factors, which stay the continuous
   temperament, their seeds drawn from the factors.) The denizen
   line is Dwarf Fortress's historical-figure rule: a critter becomes a denizen
   when it does something the record keeps, is named, is related, or is
   explicitly designated, not only when it becomes an antagonist.
3. *History against memory.* Partly answered by ruling 54, which gives
   critters memories beside habitats and habits, so a critter's memory is
   of places and what worked there, habit and habitat, while ruling 36's
   "capable of remembering" for a denizen is knowledge of events, which
   ruling 47 made a bearer form a sophont can sacrifice. Every level has
   a history, the deviation record §1 keeps whether or not anything can
   recall it. **Answered, ruling 55:** one memory, graded, over one
   history. Memory is a phenotype ability with a range: at the low end
   habit and habitat, places and what worked there; at the high end
   knowledge of events with error, which reach carries between sophonts
   and which can be sacrificed as a bearer form. Where a creature sits on
   that range is its phenotype's business, not its tier's, so a sophont is
   a critter whose memory reaches the top of the range along with its
   naming and tool use, and there is no second kind of memory to model.
4. *Holding.* **Answered, ruling 53.** Two relations, not one.
   *Containment* is physical and derived: a body holds what its capacity
   holds, and capacity is biology for a critter (cheeks, a pouch, an
   eldritch pocket), construction for a construct (a mech contains its
   pilot), and gear for a sophont, whose inventory is limited by what it
   carries rather than by its body, because it is a critter that is smart
   enough to use tools. Incorporation, kleptoplasty, is containment that
   becomes anatomy. *Possession* is asserted: a sophont, a faction or a
   polity can own, because each can assert, and a critter cannot; it
   collects, hides and uses, and what it holds is a fact of position and
   capacity, never a claim. So the pilot possesses the mech that contains
   it, and the mech, a construct, need not be inert or non-sapient, which
   is ruling 45's two axes at work: made, and possibly a person. Under
   the derivation rule containment is derived from the bodies and
   possession is an asserted fact in the record, which is why theft is an
   event and a squirrel's cache is not.
5. *Place.* **Answered, ruling 54.** The same split as holding. A
   critter has habitats, habits and memories, and its territory is
   presence and defence, a derived fact of where it is and what it
   fights for; it lives where it can. A home is possessed, so it is an
   asserted relation available to sapience: a sophont has a home, and if
   it can own, property, a faction has ground, a polity has borders, each
   an asserted claim. A named critter can "have" a home, but that is a
   one-sided assertion from the sophont who named it about the sentient
   thing it named, which is the first case in the record of an asserter
   asserting facts about something that cannot assert. So place has no
   relations of its own beyond position: derived presence below the
   assertion line, asserted claim above it, and naming as the act that
   lets a sophont extend claims over critters and places alike.
6. *Standing.* **Answered, ruling 56.** Three things the record already
   has: alignment derived from acts, reputation as the reach of deeds,
   rank asserted by a polity. The character level therefore adds nothing
   new to the schema: a character is a sophont with an asserted rank
   inside a polity, and "contingent on the group, even by absence" is a
   rank of none. What the tabletop system adds is its reading of that
   sophont (ruling 38), not a new kind of thing.
7. *Senses.* **Answered, ruling 59, and it answered more than senses.**
   A creature's senses are a phenotype ability and are the creature's
   own: the sim's creature perceives with what its body has, and acts on
   that when on autopilot. They are never imposed on the player as a
   limitation of the interface; playing blind because a mole is blind is
   refused. Instead the played creature surfaces its perceptions as
   suggestions, what it would act on if left alone. Which turns play from
   driving into **directing**: the player shapes the creature's behaviour,
   schedules, priorities and standing orders over its own methodology
   (ruling 37's reactive and belief-desire-intention agents), the way
   RimWorld's colonists are scheduled rather than steered, and the same
   directing serves one creature or a cohort, in Mesocosm and in
   Eponym alike. What there is to direct depends on biology: embodied
   abilities that carry their own procedural planning and execution,
   foraging for an herbivore, hunting for a predator, scavenging, and
   the player composes and prioritises them. Consequences for the
   record: the player's handles (ruling 10) are directives over
   processes the creature already runs, so the sim's agent and the
   played creature are one mechanism and the NPC autopilot of ruling 57
   is the same code with nobody directing; the overlay's controls (§5)
   are the directive vocabulary, not actuation; and a creature's
   perceptions are a reading the overlay may show, which is how a mole's
   world is playable. Prior art, unverified: RimWorld's work priorities
   and schedules and Dwarf Fortress's labours; The Sims' autonomy under
   direction; Majesty and Dungeon Keeper for indirect control, where the
   player posts wants and the creatures choose; Black & White's creature,
   taught by reward and punishment, which is behaviour shaping in
   Skinner's sense literally; Pikmin for directing a cohort; Creatures'
   Norns, who learn; pet systems from Nintendogs to Tamagotchi; desire
   paths, which are stigmergy, a field over the place graph laid down by
   traffic, so the ant trail and the desire path are one process; and
   for the abilities' planners, behaviour trees, goal-oriented action
   planning and hierarchical task networks, which are the procedural
   planning and execution Mark names.
8. *Death and what persists.* **Answered, rulings 42 and 61.** In the
   sim a body's death is final, everywhere; what persists is the record
   always, a sophont's memory only if retold, a character's sheet as an
   asserted fact, and a divine thing entirely, as provenance. Each
   overlay then has its own trick over that one death, and the trick is
   its foregrounded rung. Mesocosm: extra lives through the cohort, the
   lineage's other members, who are fungible enough that the run goes on.
   Eponym: extra lives through companions, who are not fungible, since
   no individual can replace another one for one unless it is a clone,
   which is the founding record's "they can replace you" with its cost
   named. The VTT: death is understood as a state that could almost be
   defeated with more of the sciences and magics, revivify, resurrection,
   wishes, so reversal is a process a ruleset or a world's magic provides,
   which in the sim is re-embodiment, the same transition an avatar's
   reincarnation uses, bought by rules rather than earned by ascent. That
   corrects the earlier reading from "you are one": the pillar is not
   Eponym's alone; finality is the sim's, and each game chooses what it
   does about it.

   *Where the dead are (ruling 62).* Significant things that die go to
   the planes after life: a plane is a world in relation to this one
   under ruling 11, and the hagiograph's kept dead reside there as
   entities rather than as lines in a journal, so the retold subset has a
   place and legends have an address. Re-embodiment is then summoning
   back from a plane, possible and costly, and a resurrection needs the
   dead's plane reachable and the dead willing, which is where the
   sciences and magics earn their price. Which plane receives a thing is a
   natural job for its alignment (ruling 51), the way the older tabletop
   cosmologies send a soul to the plane of its alignment as a petitioner.
   Prior art, unverified: Planescape and the D&D outer planes with their
   petitioners and the raise-dead rule that the soul must be willing;
   Hades, where the dead are the cast; Spiritfarer and Pyre for passage as
   the game.
9. *Divinity.* **Answered, ruling 42:** neither a fourth level nor a
   status but promotion into the world's provenance, present in every
   fork and branch after ascendance, reachable by anything, enacted by a wish with
   the world's strength. The chain of heirs and the avatar are how a
   divine thing is embodied in play afterwards; their design is W2's.
10. *Reproduction and inheritance.* **Answered, ruling 57.** Two doors,
    which are the two checkpoints the playable ecology plan already names.
    *Reproduction*, the individual checkpoint: the genotype passes and the
    phenotype's expression is rerolled, a lottery whose spread is the
    heterogeneity of the genes, so a homogeneous lineage breeds true and a
    mixed one surprises. *The epoch boundary*, the lineage checkpoint: the
    genotype itself changes, slowly, like continental drift, and only in
    the shop between rounds, never during a life; Balatro's jokers are
    switched in the shop, not in a hand. NPC lineages optimise themselves
    at that boundary, and that same adaptation is the default autopilot
    for the player's lineage, with optionality where play chooses. So
    play refines both, and through both doors: the critter through the
    life it lives and the expression it was dealt, the lineage through
    the boundary. Branching is a rung transition of §3.3's third shape: a
    lineage may fork, provided the branch has a plan to grow and to
    compete or cooperate with the lineage it left, which is the epoch
    boundary plan's speciation-as-an-act with its condition named.

    *What a lineage measures itself against.* Mark pointed at Ptree
    (ptree.org, methods read 2026-09-19), a reference that makes
    biological records from many sources comparable: forty-one properties
    of an organism or group, twenty-four categorical and seventeen
    numerical, among them nutrition as nine feeding categories, setting as
    seven environments, position, movement, growth form, persistence,
    organisation, organism mass distinct from offspring mass, and a
    tolerated temperature range distinct from an optimum, each defined by
    the biological question it answers and each interpretation keeping
    the evidence behind it; its front page adds habitat, climate, depth,
    pH, reproduction, longevity, body form and senses to the list. Three
    things to take. First, that vocabulary is a candidate far-rung
    representation of a lineage: a cohort is a property vector, and the
    epoch boundary moves it, which is §3.5's aggregation given real axes
    and the trait catalogue plan a species tier to sit under. Second,
    Ptree's own reading up its trees is the ladder as a working interface:
    "species records become distributions as you move up the tree", with
    medians, quartiles and whiskers, and a group's mix of feeding
    strategies shown as proportions, which is exactly how a cohort should
    read to a player and how a lineage should compare itself with
    another. Third, Ptree's discipline that an inference keeps
    its original premise and an entailment carries a basis is the record's
    own rule for derived readings, already stated for glyph affinities in
    the general model plan's §7.4, applied to biology.

### 3.2.2 Factions and polities

From rulings 63 to 68. Open where marked, and every *reading* below is
this record's, for Mark to reject.

**The line between them moves from methodology to constitution.** Ruling 8
said a settlement is a faction until it has a methodology. Ruling 63
sharpens it: a faction has a way of acting together too, but that way is its
members' consent and nothing else, so it "lives and dies with consent". A
polity adds methods that are "part of the deal", as an arbitration clause is
part of a terms of service: they bind a member who did not agree to this
decision, because the member agreed to the deal, or was born into it. Under
the derivation rule that is the same split as holding, place and alignment.
A faction's collective action is *derived* from its members' own decisions,
and the faction still has no state of its own. A polity's is *asserted*, a
constitution in the record, which is the state ruling 9 gave it. The
scheduler consequence of §3.2 stands: a faction costs nothing on its own,
since joining is a due event of the sophont who joins, and a polity's
procedures are due events of the polity.

**How a faction acts.** Someone proposes an act; others join or do not;
those who join are a party for that act, ruling 8's faction defined by a
person. A sophont joins for the two reasons Mark named. It wants the act
itself, which is its alignment read against the act's means and effect
(rulings 50 and 51). Or it cares for the proposer enough to join an act it
would not otherwise, which is its opinion of that sophont and that sophont's
reputation (ruling 56). The second is ruling 60's opinion modifier on a
directive, met from the other side: a player's directive to a companion and
one sophont's proposal to another are the same event, so directing,
companions and faction action are one mechanism. A proposal travels by the
means of communication all sophonts share and by those the proposer and the
hearer each align with, which makes reach (§3.4) the gate on who can be
asked at all. Where peer to peer agreement cannot emerge, everyone
differently aligned or nobody holding an actionable opinion, "a few general
protocols" apply. *Reading, accepted by Mark 2026-09-21 (D1):* because they are general they belong to the
world's ruleset (ruling 41) and not to the faction, which keeps the faction
stateless. **Open:** which protocols. *Ruled 2026-09-24 (ruling 133):* a
faction is never gated by disagreement. Those who join an act are its party
and act, and where the faction must decide as one, its members fall back on
their preferred way, the way of deciding their alignments rank highest,
counted as at a founding and held for that one decision. A polity is
different: where its form vests ultimate authority in a group rather than
an individual, its acts are gated on that group's disagreement, so a council
can deadlock where a monarch cannot.

**Already in code, checked 2026-09-20.** Eponym's `eponym-social` is
this model at the scale of two. Standing is folded from the deed log as
trust and affinity and carries the deeds behind it (`relation.rs:25-66`).
Willingness is three gates, can I do it, would I risk that for you, is it
more than I would bear, answered by a refusal about the work, a refusal
about the asker, or a counteroffer (`willing.rs:7-23`). A settlement is
homes offered under agreements, with residence derived from agreement state:
"Ending the agreement *is* moving out" (`settlement.rs:10-14`), which is a
faction living and dying with consent. What it lacks against ruling 63 is
the first reason to join: nothing in the crate reads the act against the
asked sophont's own alignment (no alignment, tenet or value appears in its
source), only the asker's standing and the danger. The tabletop's
`isometry-campaign` holds the opposite end: a faction turn that draws one of
five verbs per faction from an entropy tape (`faction.rs:40-57`,
`:104-110`), deciding for the faction from outside with no members in view.
Under ruling 63 that tick is a far-rung stand-in to be re-derived from
members, not the model.

**What a form of governance is.** Mark: "all forms of governance that can be
associated with acts or qualifiable parties should be considered."
*Reading, accepted by Mark 2026-09-21 (D2):* a form is a pair, who qualifies to decide and by what act the
decision is made. Qualification is a predicate over what the record already
holds for an entity: standing in its three parts (ruling 56), possessions
(ruling 53), body and age, lineage, provenance. Rule by the wealthy, the
old, the strong, the ordained, the well born, the divine, everyone, or
whoever the lot falls on is then one shape with a different predicate. The
deciding act is a process of §3.3's first shape: voting, fighting, buying,
divining, inheriting, taking. Both halves are vocabulary the sim has, so
forms are data to be authored and generated, never an enum.

**How a form is chosen: ranked preference, derived from alignment.** "The
way people solve their problems feeds their alignments which prompt their
collective action methodology." A form of governance is a means of deciding,
and ruling 50's tenets already hold opinions of means, so a sophont's
ranking of forms falls out of its alignment with no new machinery: the rogue
who solves problems by taking ranks rule by taking first and individualism
next. Support then transfers down each ranking until a form holds, Mark's
"preference-ordered rank choice voting for the sim": the rogue's first
choice has no other supporters, so their support goes to their second.
*Reading, accepted by Mark 2026-09-21 (D3):* this is the sim's mechanism for which form emerges, weighted by
influence exactly as ruling 51 weights a faction's alignment, and not
necessarily an election anyone in the world holds. It runs at the rung
transition that founds a polity (§3.3), and its winner is asserted as the
constitution. At the cohort rung the ballots are ruling 50's distributions,
so the count runs over blocs and not heads. Ties break on the seed.

**Enforcement (ruling 64).** A polity's asserted state needs means of
enforcing it, likely several, and they "must be efficacious". They need not
be violent: Mark names the provision of the requirements for life, and self
defence. *Reading, accepted by Mark 2026-09-21 (D4):* a polity enforces with what it provides and what it
asserts. Ruling 38 already made provisioning a process of the faction and
polity rungs, the collective feeding the individual, and what is provided
can be withheld. Everything the record lets a polity assert it can also
revoke: rank, office and membership (ruling 56), property (ruling 53), home
and borders (ruling 54). Force is one more means and not the defining one,
and ruling 66 closes the list by leaving it open: force, faith, money, habit,
a god, "any of 'em".
So the "or else" of a rule is an ordinary process of §3.3's first shape,
with a cost to the polity that applies it, and a guild that bars a smith
from the forge and a crown that sends soldiers are the same shape. "Related
to the political system, the acts within": the act by which a form decides
(ruling 63) suggests the means by which it enforces, so rule by buying
withholds pay, rule by divining withholds rites, and rule by fighting
fights. **Open:** whether that link is a default the generator uses or a
constraint the sim holds. *Ruled 2026-09-24 (ruling 137):* a default. The
generator suggests means from the way of deciding, and a polity may enforce
by any means it has.

**Efficacy is read, never asserted.** *Reading, accepted by Mark 2026-09-21 (D5):* whether a means is
efficacious is a track record, how often applying it produced compliance,
which is ruling 50's trust, "expectations" formed from "how things turned
out in the past", held about the constitution as about any tenet. After
founding, the constitution is asserted and stays put while its members'
derived preference keeps moving, so the distance between the two can be read
at any time; ruling 51 already keeps a polity's constitutional alignment "in
addition to the factions within" it. Enforcement is what holds that gap
open.

**Inactivity and death (ruling 65), correcting this record.** The first draft
of the paragraph above read failed enforcement as the polity falling back to
a faction by itself. Mark ruled otherwise. A polity that lacks support for
its operations "essentially does nothing", and it dies "only when people
agree" it has. So a polity has three conditions, not two: *active*, with
support enough to operate; *inactive*, ruling 66's word, asserted in the
record and doing nothing; and *dead*, by agreement. That is the derivation rule kept honest:
a constitution is an asserted fact, and an asserted fact stands until
another assertion ends it, never because a derived quantity fell. It also
names what is scarce to a polity in Mark's own word: **support**. The
scheduler consequence is free: an inactive polity has no due events and costs
nothing, as a faction does.

Mark gives three causes for the agreement. *The political context changed*,
which for a contingent polity is its host changing or falling, so ruling
64's cascade makes polities inactive and leaves their ending to those who
belong to them. *Everyone who knew about it died*, which is ruling 5 applied
to polities: memory is not global, things can be forgotten, and a polity
lives in those who know of it, so its existence has a reach like any event's
(§3.4), and agreement among nobody is agreement. *The methods, goals and/or
means became pointless*, which also says what a polity is made of: goals,
its focus (ruling 64); methods, how it decides (ruling 63); and means, how
it operates and enforces (ruling 64). §3.3's "a polity collapsing into
factions" is then the rung transition that records an agreed death, the
factions having been there all along. **Ruled, ruling 68:** an inactive polity can be
revived by whoever brings it support, since nothing ended it, which is the
pretender, the government in exile and the restored order. **Open,** and agreed
open by ruling 68: whether
a constitution borne by an item, a charter, keeps a polity from being
forgotten when its last knower dies; who must agree, and by what count; and
reform and secession. *Reading, from ruling 129, put to Mark on 2026-09-24
without objection:* the charter question is answered, since a charter is a
bearer and a thing stays of note while some bearer records it; a polity
whose last knower dies survives in its charter, and whoever finds it can
revive the polity under ruling 68. *Ruled 2026-09-24 (rulings 134 and
135), reform and secession:* a constitution changes by the polity's own way
of deciding or by its amendment rule. Force outside the constitution, unless
the constitution itself rules force legitimate, founds a new polity and never
takes over the old one, so a faction wanting a nation's full power acquires
it legitimately or claims it quickly after the old polity dissolves.
Secession is the seceders' own act, a faction founding its own polity, which
the old one may accept or contest. *Reading, not ruled:* the old polity
outlives the coup that circumvents it, as ruling 66's suppressed or
superseded, and ruling 68's government in exile is exactly the polity a
coup could not take. *Ruled 2026-09-24 (ruling 136), who must agree to a
death:* either route. A polity may dissolve itself by its own way of
deciding or its amendment rule, or it dies when nothing holds it any more,
no member keeping it, no mind remembering it and no bearer recording it.

**Existence is asserted; condition is derived (ruling 66).** Ruling 65 kept
death for agreement. Ruling 66 lets everything short of it happen
"automatically", and names four conditions a polity can fall into.
*Reading, accepted by Mark 2026-09-21 (D6),* each as something the record can already tell:

- *Inactive:* it lacks support for its operations and does nothing (ruling
  65).
- *Suppressed:* it has support, and another's enforcement stops it
  operating: the banned guild, the occupied town, the outlawed church. Its
  members can still act as a faction, by consent and in secret, which is the
  underground.
- *Superseded:* another polity now binds the acts that were its focus, and
  that one's enforcement works: the new guild, the conqueror's law, the
  successor state.
- *Subordinated to a faction:* its methods still run and their outcome is
  one faction's will, because that faction fills the party qualified to
  decide (ruling 63) or holds the means. This inverts ruling 8's composition
  for as long as it lasts, the part commanding the whole: the cabal behind
  the throne, the machine that owns the council.

So the two halves of the derivation rule sit on one entity. That a polity
exists is an asserted fact, ended only by agreement. What condition it is in
is derived, read from support, from whose enforcement prevails over its
focus, and from who fills its deciding party, and it changes with no
assertion at all. The conditions are not exclusive, since a polity can be
superseded and subordinated at once, and each is reversible while the polity
is not dead. **Open:** whether more conditions are wanted, and what a
subordinating faction can and cannot make the polity do. *Reading, from
ruling 134, put to Mark on 2026-09-24 without objection:* a subordinating
faction can make the polity do whatever its own way of deciding can do,
since it acts through that way, and can change its constitution only by
the amendment rule. *Ruled 2026-09-24 (ruling 138):* the one further
candidate, ruling 133's deadlock, is per act and not a condition, so the
conditions stay four until a new candidate is found.

Prior art for ruling 66, known. The League of Nations shows both rulings in
one life: inactive through the war, superseded by the United Nations in
1945, and dead only when its own Assembly voted to dissolve it in April
1946. Solidarity in Poland was suppressed in 1981, persisted underground,
and returned in 1989. For subordination: Michels's iron law of oligarchy
(1911), that organisations come to be run by a few whatever their
constitutions say; Stigler on regulatory capture (1971); and the
state-capture literature after Hellman, Jones and Kaufmann (2000).

Prior art for ruling 65, known. Searle's *The Construction of Social
Reality* (1995): an institutional fact exists by collective acceptance and
only while accepted, "X counts as Y in context C", which is "they only die
when people agree they do" as philosophy. In law, desuetude: English and
American statutes do not lapse by disuse and stand until repealed, while
civil-law traditions let them lapse, so both answers have been lived under
and Mark's is the first. In history: the Holy Roman Empire, long inert,
ended by a formal act in 1806; the Order of Malta, a polity recognised
without territory; titular sees, offices kept for dioceses that no longer
operate; pretenders and governments in exile. From memory of games,
unverified: Crusader Kings III's titles exist de jure whether or not anyone
holds them and are created and destroyed by explicit acts.

**A polity has a focus, not a size.** "Not every polity is a town or a
nation. A polity focused on blacksmithing would be a blacksmith's guild." So
scale is orthogonal to the polity rung, as ruling 58 made it orthogonal to
kingdom, and what a polity has instead is a focus: the acts its constitution
binds, by whom, and where. A guild binds the acts of a craft, a town the
acts within a place, a church the acts of a faith, a company the acts of a
trade. In the grammar of institutions that is the attributes, the aim and
the conditions of its rules, so jurisdiction is a predicate over acts and
needs no territory. It also says what each polity can withhold: the guild
the forge, the mark and the apprentices, the town the ground.

**A polity's goals (ruling 68).** Mark answered the question with one: "why
would a sophont become a blacksmith?" There is "no one clear cut answer", so
none is built in. A polity is "sort of a collective entity with its own
alignment/values", and its goals are derived from that, "like individuals
have". So goals are neither fixed at founding nor stored. A polity is an
agent at its own rung with a sophont's shape under ruling 37: what it knows,
what it values and what it lacks yield what it pursues, and its methods
(ruling 63) stand where a sophont's deliberation would, turning a pursuit
into an operation. What it lacks is support (ruling 65), as a sophont lacks
food. One derivation serves both rungs, which is ruling 10 again, and it
means the work of designing how a sophont's goals come from its values is
done once for both.

**No will to continue is built in.** "If an institution outlived its use,
why would it want to continue if it didn't find a new use? Polities change
like that." A polity whose use is gone finds a new one, or it becomes
pointless, and a pointless polity is one people agree is dead (ruling 65).
*Reading, accepted by Mark 2026-09-21 (D7):* where an institution is seen clinging on, that is derived and
needs no drive of its own. The members it provisions support it for their
own reasons, and the sharpest case is ruling 66's subordination: an
institution that "wants to continue" is one subordinated to the faction of
its own officers, who want their provision to continue. That is Michels's
oligarchy with nothing added to the polity.

**A polity's alignment (ruling 79), replacing this record's reading of two
alignments.** The record had read a polity as holding two, one professed in
its constitution and one practised. Mark ruled it simpler: "just like
factions and individuals, a polity has its own alignment from its acts." A
polity decides "what it stands for by collective/hierarchal decision and
actions", and deciding is acting, so its decisions, the adopting of a
constitution among them, are acts in the record like any others and its
alignment is derived from them. One alignment, one derivation, the same at
every rung, "a repeat of that theme". What is worth comparing is that
alignment "against the alignments of the factions, individuals" within it,
which the other tiers already provide; that one distance is the legitimacy
gap of the paragraphs above. This amends ruling 51's "constitutionally, not
derived" in Mark's own later words: the constitution stays an asserted fact,
and the alignment read from a polity's acts is derived. "Polities change
like that" is then a polity's acts drifting and its alignment with them.
Ruling 81 confirms the reading in Mark's words: asserting a constitution is
"itself an act... indeed, a definitive one", so a polity's first and
weightiest act is the one that founds it. **Open:** reform itself.

Prior art for ruling 68, known. Sills's *The Volunteers* (1957) named goal
succession from the March of Dimes, founded against polio, which found a new
use once the vaccine made the old one pointless; Zald and Denton (1963)
traced the YMCA from evangelism to general service. The opposite, goal
displacement, where keeping the organisation going becomes the goal, is
Merton's (1940) and Michels's, and is the drive ruling 68 declines to build
in and the reading above derives. Barnard (1938) and March and Simon (1958)
give support its accounting: an organisation persists while the inducements
it offers are worth the contributions it asks. Hannan and Freeman (1977) are
the caution, that most organisations do not adapt and are replaced instead,
so the bench should see both outcomes. In history the London livery
companies, the Blacksmiths' among them, outlived the regulation of their
crafts and found charitable and ceremonial uses, and the Order of St John
went from hospital to military order to sovereign charity.

**Contingent polities, and the institution.** "Some polities may indeed be
contingent upon others, like factions comprised a polity. If you can make a
durable agreement that works within another polity, congrats, you've built
on an institution." So composition recurses: factions comprise a polity
(ruling 8), and polities may sit inside polities. *Reading, accepted by Mark 2026-09-21 (D9):* what makes the
host an institution is that an agreement made under it holds without its
parties enforcing it themselves, and contingency is borrowed enforcement:
the guild's "or else" is backed by the town's courts, the town's by the
crown. Two consequences. A contingent polity is cheap to found, because it
need not bring means of its own, which is why a settled world grows
institutions and a wild one does not. And when a host fails, everything
contingent on it must find means of its own or fall back to a faction, so
collapse cascades, and those are events the hagiograph keeps. Checked
2026-09-20: Eponym's standing agreement is the consent-only form, "still
not a command", which its holder may decline "when its premises have
changed" (`eponym-social/src/agreement.rs:14-17`), and nothing stands
behind it but the two names on it; a durable agreement in ruling 64's sense
is that same record with a polity's enforcement behind it.

**Polities among themselves (ruling 67).** Polities under no host have only
consent between them, so among themselves they are a faction, and a treaty
is a standing agreement at scale: it holds while both keep to it, and "what
it rested on changed" ends it, in the words Eponym's agreement already has
for that ending (`eponym-social/src/agreement.rs:48`). That widens ruling
8, whose faction was "comprised of many creatures": a faction's members are
whatever can consent, sophonts or polities. Everything ruling 63 gave a
faction then applies with polities as the members. One proposes an act and
others join, for the act itself, read against their constitutional alignment
(ruling 51), or for the proposer, read as their opinion of it and its
reputation; those who join are a party for that act, which is an alliance, a
coalition or a trade league. And the way up is the same rung transition: a
faction of polities that founds a constitution with means behind it has made
a host, an empire, a church or a federation, by ruling 63's ranked
preference with polities casting the ballots. So composition is one
recursion in both directions, consent below every constitution and consent
again above it, and the sim needs no separate model of diplomacy.

Prior art for ruling 67, known. Waltz's *Theory of International Politics*
(1979) makes anarchy, the absence of any authority above states, the
defining condition of their relations, and Bull's *The Anarchical Society*
(1977) adds that they keep norms anyway, which is Ostrom's norm, a statement
with no "or else", at the scale of polities. Axelrod (1984) and Keohane
(1984) are why agreements hold there regardless: reciprocity, reputation and
the shadow of the future, which are ruling 56's reputation and ruling 50's
trust. The Hanseatic League is the case in one body: towns and merchant
guilds in league without a sovereign, deciding in a diet, and enforcing by
exclusion from the trade.

**Prior art for ruling 64,** known literature. North (1990) defines
institutions as the humanly devised constraints on interaction and makes
third-party enforcement the thing that lets strangers keep agreements, which
is Mark's institution. Greif's Maghribi traders, and Milgrom, North and
Weingast on the law merchant (1990), are enforcement without violence or a
state, by reputation and exclusion from the trade, which is the guild.
Ostrom's design principles for lasting commons (1990) include monitoring,
graduated sanctions, recognition of the right to organise by an outside
authority, and nested enterprises, the last two being the contingent polity.
Etzioni's three kinds of compliance, coercive, remunerative and normative
(1961), and Mann's four sources of social power, ideological, economic,
military and political (1986), are ready taxonomies of means. Weber's state,
a monopoly of legitimate force over a territory, is the narrow case ruling
64 declines to make the definition.

**Prior art.** Known literature first. Crawford and Ostrom's grammar of
institutions (1995) writes any institutional statement as attributes,
deontic, aim, conditions, or else: which qualified party must, may or must
not do which act under which conditions, on pain of what. It classes
statements by what they carry: a shared strategy has no deontic, a norm has
no "or else", a rule has all five. That is ruling 63's line drawn by someone
else, factions running on strategies and norms and polities on rules, and
the grammar is a candidate syntax for a constitution in ruling 41's binding
language. Ostrom's three levels of rules, operational, collective-choice and
constitutional, name what a polity adds. Hirschman's *Exit, Voice, and
Loyalty* (1970) is "lives and dies with consent": a faction's members hold
exit, and a polity raises the cost of exit and so must channel voice.
Weber's three grounds of authority, traditional, charismatic and legal, line
up with lineage, reputation and rank. For the fallback protocols the
multi-agent literature has the contract net (Smith, 1980) for allocating a
task among willing peers, and joint intentions (Cohen and Levesque) and
SharedPlans (Grosz and Kraus) for what a party commits to. Instant-runoff
counting is the transfer Mark described; it is known to be non-monotonic,
which a sim can live with. From memory of games, unverified: Crusader Kings
III's factions are coalitions a vassal joins by opinion of the liege, acting
once their combined strength passes a threshold, and its feudal contracts
are a literal deal; Victoria 3's interest groups carry ideologies that rank
laws and weigh in by clout, with legitimacy as a number; Dwarf Fortress's
entity positions attach responsibilities to appointed, elected or inherited
offices.

### 3.3 Processes

Three shapes, and no fourth found yet:

- **Choices under scarcity**, for what an agent does: inputs under scarcity,
  a choice, a cost, an outcome, a record of `(scarcity context, chosen,
  foregone, cause-link)` per Law A. One shape from a creature's metabolism
  to a polity's levy.
- **Agentless processes**, for what happens to the world: weather, decay,
  growth, erosion, fire, spread. A rule over conditions on places, running
  with nobody choosing. Under ruling 98 these are the world entity's own
  processes, its metabolism (§3.3.1), and where the world is a critter or
  a sophont they are chosen.
- **Rung transitions**, for what happens to the agent set: a faction
  founding a polity, a lineage splitting, a settlement incorporated, a
  polity collapsing into factions. These name what dissolves and what is
  founded, and they are the events the hagiograph most often keeps. (Ruled
  2026-09-24, ruling 125: noting and promotion to legend are the world's;
  collection and merging branches are work on the record, not
  transitions.)

Mesocosm's `ProcessDef` (processdef plan, 2026-08-01) is **not** the first
shape's definition as it stands, by receipt: its digest covers namespace,
name, `expressed_by` and `seeding` and nothing else
(`mesocosm-core/src/process.rs:391-403`), `expressed_by` is a subset of
four roles and `seeding` has two values (`:350-358`), so the whole
definition space holds thirty distinct rule shapes, and the trait catalogue
plan reached the same count independently. It carries no scarcity, no cost,
no foregone and no cause-link. Either it is widened to Law A's record or
the base profile takes a new definition; that is §9.7, now with evidence.
The strongest existing candidate for the new definition is Eponym's
world-conditions schema (world conditions plan, 2026-09-09): typed
conditions, operations, relations and invariants under a content-addressed
rules revision, carrying scarcity, cost and provenance, unimplemented, and
forbidden from promotion by its own stop rule, which is one ruling to
lift (found 2026-09-18 by W1).

**One definition, run two ways (ruling 75).** A process is executed for the
one in the foreground, step by step with a receipt, and applied to the many
in the background as a rate over a distribution, which is "cheaper through
aggregation and not needing to process some foreground info". The two
"should agree", and "losses should be managed between transitions in a
manner that preserves similitude". What licenses the aggregate is ruling
75's other half: "things of no note are fungible", interchangeable, so a
cohort may be counted and need not be listed, and a generated thing is "a
placeholder meant to be reified/supplanted by the player". How long a
realised thing lingers before it is funged is a world setting, "if you have
a very capable system and want to increase the buffer of stuff before things
are funged, for a feeling of consistency", which is ruling 71's collector
given a budget. (Corrected 2026-09-24 by ruling 113 from "a host's
setting": the buffer changes what is funged, so it lives in the world's
rules and replays with the log.)

*Reading, accepted by Mark 2026-09-21 (D17).* The two transitions have names in the
multiscale literature. *Lifting* takes an aggregate to individuals: sample
from the distribution, conditioned on every asserted fact, from the seed, so
the same place lifts the same way twice. *Restriction* takes individuals to
an aggregate: conserve the accounts, counts, mass, energy, which the
world-conditions schema already keeps distinct, and fold what deviated into
the distribution's parameters. Similitude is then two checks the bench can
run on seeded draws (ruling 15): restricting what was just lifted returns
the aggregate it came from, and a population run each way ends in the same
statistics. It also puts a requirement on the definition itself. A process
must be declarative enough for its aggregate form to be derived from the one
definition, typed inputs, costs, effects and rates, which the schema's
operation is ("a typed operation cannot execute arbitrary script") and an
opaque script is not: a script can only be executed, never integrated. So
ruling 32's choice is what makes ruling 75 possible, piccolo authors and
lowers to definitions, and scripted hooks run only in the foreground.

*Ruled 2026-09-24 (ruling 113).* What the two ways must agree on. Anything
of note runs individually, watched or not, so only the fungible are ever
run as a crowd, and they agree in distribution rather than identity: the
same toll, falling the same way across whatever a later process reads. The
first check above is a consistency check and no more; restriction returning
its aggregate says nothing about the evolution that follows (the
[aggregation research](2026-09-22_aggregation_research.md), reading
Kevrekidis). The second is the test, and it is collection's observational
test run forward in time: the distribution of everything a later process, a
player or the hagiograph can read is the same either way beyond chance, over
seeded draws run both ways, within the world's stated tolerance. Means and
totals are not enough, since two crowds with equal total energy can hold
different numbers of the starving, the research's counterexample. What a
player watched is an intent in the log, because watching decides which
individuals exist in detail. A world founded exact runs the lossless runner
throughout and pays for it wherever members interact.

Prior art for ruling 75, known. Gillespie's stochastic simulation algorithm
executes every event exactly, and his tau-leaping (2001) jumps over many
events at once by drawing their counts, approximately, with first-order
consistency and an approximation error (corrected 2026-09-24 from "with the
same statistics", after the aggregation research's reading of Rathinam,
Petzold, Cao and Gillespie), which is the foreground and the background of
one definition. Kevrekidis's
equation-free method is where *lifting* and *restriction* are named.
Cautions, from memory of games and unverified: the X series resolves combat
differently in and out of the player's sector and players exploit the
difference, and S.T.A.L.K.E.R.'s A-Life switches between an offline and an
online model with visible seams.

**Ecological states (ruling 383, 2026-09-28).** Harmony, order and chaos
name the state of an ecology: balanced with nothing maintaining it, balanced
because something maintains it, and unbalanced. Each is a reading over the
web, never stored, and no lane is open for it.

### 3.3.1 Needs, value and exchange

From ruling 94. Mark's two questions, why people act and how that aggregates
into trade, are answered here as a reading, docketed as D25, accepted 2026-09-22, built from
rulings already made.

**Value (ruling 94).** Items, value, crafting and services "bear relation to
scarcity, capability, and material need". In the record's terms those are
the three inputs of §3.3's first shape: scarcity is the supply at a place,
capability is who can make or do the thing (a phenotype ability, a craft, a
technique of ruling 47), and need is demand. So value is derived and never
stored, a reading over supply, capability and need at a place and a time,
and a price is that reading at a market, one place where exchange processes
concentrate. Money is then an item whose only use is to be exchanged, which
makes it an asserted claim of the kind ruling 64 lets a polity stand behind,
or ruling 67's consent among those who take it, and never a primitive of the
sim.

**Why people act: needs, but at three tiers.** "Needs, like sims?" Yes at
the bottom and not only that. Ruling 37 already tiers the methodology, and
each tier answers the question its own way:

| Tier | Why it acts | What it wants |
| --- | --- | --- |
| Critter, reactive | a need crosses a threshold and the matching ability runs: forage, hunt, flee, rest | the body's ledger (ruling 38): Mesocosm's nutrition budget in full, Eponym's `Needs { hunger, fatigue }` today (`eponym-world/src/bodies.rs:46-49`) |
| Denizen, belief-desire-intention | a desire is chosen among several by what it believes and what it values, and pursued as an intention through several acts | the ledger's needs, plus wants derived from alignment (ruling 51): comfort, standing, safety, company, the things its tenets approve |
| Character, normative | a role or an institution asks it: an office, an oath, a contract, a levy | the above, plus obligations asserted by its polity (ruling 56's rank) |

Mark's derivation of need holds at all three: need is read from "a part of
alignment" (what a sophont values), "the stuff people do (acts)" (its
record, so a smith needs iron because it smiths) and "are likely to do
(predilection, personality?)", where predilection is ruling 37's disposition
setting the thresholds, as it does for telling secrets (D21). This is the
Sims' model made honest: the Sims has one tier of needs with a utility curve
each; the wing has needs at the bottom and values and obligations above, all
three derived from state the record keeps.

**How it aggregates into trade trends.** Ruling 75's aggregate form, applied
to exchange. A cohort at a place is a distribution over needs and
capabilities, so its demand for a good is a rate, and the place's supply is
a stock moved by production, consumption and carriage. Exchange in the
aggregate is then flow along the graph's routes (§3.7) from where a thing is
cheap to where it is dear, at a rate set by the price difference less the
cost of travel, which is the same diffusion the record already uses for
reach, temperature and plague, with a gradient in price instead of
concentration. A trade trend is that flow read over time. Foreground
exchanges are the lifted case: one sophont, one price, a receipt in the
record; and their restriction back is a stock changing hands. Provisioning
(ruling 38) is a polity running this flow on purpose toward its members, and
a shortage is a stock at zero with need still rising, which is the event the
hagiograph may keep.

**Acting and crafting (ruling 95).** Mark posits that taking an action "is
analogous to a crafting process", and that crafting proper is "incorporating
materials into that". The wing's one verb already says this: Mesocosm's
*metabolize*, "world into self, self into world". An act is the verb;
crafting is the verb pointed outward, matter incorporated into an act and an
item left behind, as kleptoplasty is the verb pointed inward. So crafting is
not a fourth process shape. It is the first shape with matter among its
inputs, and the schema's operation already has the slot: its commitments
include "item, matter, energy, time", its transforms are typed, and a
byproduct is declared (§3.3, `OperationDef`). *Reading, docketed as D26, accepted 2026-09-22:*

- *The link Mark asks for is the tenet, from the actor's side.* A sophont
  chooses among the acts it could take by three readings the record already
  makes: whether the act addresses a need it has (§3.3.1), whether it has
  the capability, which is ruling 50's *trust* that the process yields the
  effect, and whether its alignment approves the means and the effect,
  ruling 50's *opinion*. Eponym's willingness rule is the same three gates
  asked of oneself: can I, will it work, do I want to (`willing.rs:7-23`).
  So people take actions suited to their abilities and beliefs because the
  score of an act is need times trust times approval, and every factor is
  derived.
- *What gets made is shaped the same way.* An item is the outcome of an act,
  so it is chosen by the same score: a smith who needs to eat and values
  generosity forges the plough the village lacks (ruling 94's scarcity) and
  not the hundredth sword. And an item bears its making: nis is matter typed
  by provenance, and a made thing keeps the maker, the materials and the act
  in its record, as ruling 53's construct keeps its maker.
- *Nobody churns weapons to level up,* for a reason the tiers of keeping
  already give. A fungible act is folded into the cohort's distribution
  (ruling 75), so a hundred identical swords move the smiths' competence at
  the world's rate and leave no note; only an act of note leaves a note on
  the maker (ruling 80, impresa `experience`) and on the thing. The journey
  of ruling 47 records means, not counts. Grinding therefore produces
  aggregate competence and never a legend, and a legend comes only from a
  rare confluence.
- *A fey mood is a triggered need.* Yes, and it is the same machinery with a
  rare trigger: a confluence of circumstances (a glyph placed nearby, ruling
  48; an unprecedented event reaching the sophont, §3.4; a material of note
  in hand) raises a need whose only satisfier is one act, in the
  world-conditions schema's terms a precondition met rarely and a commitment
  of attention that will not release; the act's outcome passes the
  hagiograph's test on novelty alone. That is Dwarf Fortress's strange mood
  with its cause named, and it also gives Mark's "wild events" of ruling 91
  a personal scale.

*Mark's open list, placed, and corrected by ruling 96.* Three of this
record's placements were wrong. **Materials:** nis and scruple are not
materials, "any more than a generic 'atom' is. They're more like a scale":
nis is matter typed by provenance and scruple a part's measured mix, the two
rungs matter is read at, and the material question is "the typology of kinds
of nis and their interactions". Checked 2026-09-22, that typology is three
kinds today, `NisKind::{Producer, Consumer, Decomposer}`
(`mesocosm-core/src/process.rs:172-176`), the trophic strategies, and
nothing about how kinds interact beyond who may eat what. **Open:** the
typology of kinds and their interactions, which is the sim's materials
question. **Capabilities:** an *ability* is a thing an entity can do,
"roughly analogous to object and basic interactions"; a *skill* is a
profession or a component of a class, "utilitarian, practiced,
experience-accumulating": smithing, fishing, farming, swordplay, pyromancy;
a *technique* is "an ability with a skill precondition", and "you can share
techniques if both parties have the necessary skills". So a skill is the
thing that accumulates, which answers half of the open question below:
repetition of a skill does accumulate in one sophont, and what folds into
the cohort under ruling 75 is the unnoted act, not the practice. That also
refines the middle form of rulings 46 and 47: what is sacrificed there is a technique or
a skill, and a technique is what is taught, since sharing one needs the
skill on both sides. **Hybridising:** kleptoplasty is conditional, "probably
not everything can do it", and not the mechanism; hybridising abilities is
what a sophont does "after learning enough", which under the definitions
above is a technique whose precondition is more than one skill. **Rarity**
stands as scarcity, but the question Mark asked was *quality*, taken up
below. A recipe is a process definition with matter among its commitments,
data under ruling 41; a triggered condition beyond crafting is a
precondition of any operation, so a vow, a grudge, a pilgrimage or a duel is
a triggered need of the fey mood's kind. **Open:** how a skill accumulates
and whether it decays; improving an item; what a technique is made of, which
the hagioglyph organ's plan owns. *Ruled 2026-09-24 (ruling 132), the
decay:* an unpractised skill decays slowly toward a floor it never drops
below, so doing is kept better than knowing, which fades at the mind's
memory rate under ruling 127. Prior art, known: procedural memory outlasts
declarative memory, and skill-retention studies find that unpractised skill
decays with time. From memory of games, unverified: RimWorld decays only
high skills when they go unpractised. *Ruled 2026-09-24 (rulings 141 to
143), the rest:* a skill rises by doing, harder or riskier work teaching
more, by being taught, by studying from bearers, and by breakthroughs, a fey
mood among them. Techniques are both found in the canon and invented beyond
it by masters, spreading by teaching and by bearers; what a technique is
made of beyond that stays with the hagioglyph organ's plan. An item is
improved only as far as its maker's skill and its story allow, and its
material is no ceiling. *Reading, not ruled:* an invented technique that
spreads far or becomes legend is a candidate for the canon's next revision,
which is how the canon grows. *Ruled 2026-09-24 (ruling 170):* art and
ritual are crafts, and any craft or pursuit can be politicized or
ritualized. *Reading, not ruled:* a pursuit is politicized when a polity
takes it as its focus (ruling 64's guild), and ritualized when a tenet
attaches an obligation or a taboo to its performance; a work of art that
depicts a tale is a bearer of that tale, and keeps it of note under ruling
129.

**Technology (rulings 171 and 172).** A world's generated tree sets what can
be known, and what is known is read from the techniques and recipes that
minds and bearers hold; a technique no mind or bearer holds is lost, and can
be found again (a reading from rulings 129 and 142, put to Mark without
objection). An age of technology is an epoch of a culture or society, read
from what its peoples commonly know and named when noted, as a culture is;
the founder may cap how far technology can go, as a world's magic is set
(ruling 102); and an age that reaches a new scale can realign the world to
a new scope (rulings 41 and 90), as spaceflight would open the space scope.
Invention, a master going beyond the canon, is driven by need, by contact
between peoples, by temperament and by mastery (ruling 173).
Prior art, known: Civilization's authored tech tree as the climbed case,
and the idea of recipe preconditions as an emergent tree, which is what
crafting games' recipe graphs already are.

**Materials are the roster (ruling 97).** The typology of kinds of nis is
the world's roster of lineages, and so the materials of a world are the
critters that live or lived in it. That is the nis ruling of 2026-09-02 read
from the other end: matter is typed "kingdom first (flora, fauna, myco,
micro...) and lineage under it", "with the type vocabulary world-derived
from the roster rather than an authored element table" (playable ecology
plan §6, ruling 4). No material table exists anywhere in the wing, and none
is to be written. *Reading, docketed as D28, accepted 2026-09-22:*

- *A material's properties are its lineage's traits.* Ruling 57 already
  reads a lineage as a property vector, Ptree's vocabulary of mass, growth
  form, persistence, tolerated temperature and the rest, and the trait
  catalogue is where such properties live. So hardness, weight, edge and
  wear are readings of the lineage that a lot of nis came from, the way the
  strike system reads geometry, and D27's material axis of quality is the
  lineage.
- *Interactions are effects.* Ruling 49 makes an effect a glyph and a
  process aligned with an effect by how often it yields it. A material's
  interactions are then which effects a nis of that lineage produces and
  suffers: wood burns, shell turns a cut, a mycelium holds water. "Fire
  against wood" is burning applied to nis of one lineage, with the outcome
  read from that lineage's traits and the record, never from a table of
  pairs.
- *Learning the ecology is the note machinery.* What a thing is made of is
  knowledge: a note (ruling 80, impresa `discover`) on a lineage, held by
  whoever has met it, and a recipe or a technique (ruling 96) names lineages
  or kinds. A sophont with no note on a lineage cannot use it, and denizens
  learn as players do, so a foreign material is one the town's smiths have
  no note on. That is Mark's "reason to learn the ecology" as mechanism: to
  "familiarize yourself with things to see what you can use".
- *Scarcity and provenance are now literally ecological.* Ruling 94's
  scarcity of a material is the abundance of a critter at a place; a rare
  material is a rare critter, and an extinction ends a material. Nis carries
  where it came from, so a sword carries its lineage, and a lineage refined
  in Mesocosm over the ages is a material in Eponym and the VTT. That
  gives Law A's line "shapes become relics of factions" its literal
  mechanism: a shape crosses as an item bearing the lineage's nis.
- *Check against Law A,* left **open** for Mark. The founding record says
  what crosses games is "choices under scarcity, not morphology", with the
  boundary that "morphology still matters enormously within Mesocosm, where
  the simulation reads the body directly". Here morphology matters in the
  other games as material. The reading that keeps Law A is that what crosses
  is an item and its lineage's provenance, and the material properties are
  ruling 38's coarse far-rung reading of the lineage's traits, not the body;
  whether that satisfies the law is Mark's to say. *Ruled 2026-09-24
  (ruling 139):* amend Law A. Materials are named as the exception, a
  lineage's traits travelling as what things are made of, beside "shapes
  become relics"; applied at Mark's word to the founding record's Law A and
  Mesocosm's CLAUDE.md the same day.
- *Inert matter,* **answered by ruling 98.** The world is itself an entity
  made of stuff, "a special sort of entity, world-class", and "the basic
  macro kingdom". So bedrock, iron, clay, water and air are nis whose
  provenance is the world, and there is no untyped floor: the nis ruling's
  "matter fully returned to soil has lost its nis and is untyped stock" is
  amended to "has returned to the world's nis", a finding filed with the
  playable ecology plan, which owns it, against `Material::Untyped`
  (`mesocosm-core/src/matter/stock.rs:20-25`). Everything then has a
  provenance and the root of every provenance is the world, which is what
  ruling 42 already says of divinity, "intrinsic to the provenance of the
  world", and what ruling 43's worldtree, "with the world's own processes
  bound to it", assumed.

**The world as an entity (rulings 98 and 99).** *Reading, docketed as D29, accepted 2026-09-22.*
The world is one more entity on the ladders the record already has, at the
root. Its kind is open to the world: "maybe the world is inert, or sentient,
or a sophont, or a god, or a turtle, or whatever. maybe it's flora, or
fauna. maybe it's dead." Each of those is a position on a ruled axis, so
nothing new is needed to say it:

| "The world is..." | In the record's terms |
| --- | --- |
| inert | a body without agency, as an item is (§3.1); no methodology, so no state of its own (ruling 9) |
| sentient, a turtle, flora or fauna | a critter of the world's kingdom at the macro scale, the terrain-body of ruling 39, and ruling 73's world resting on a critter is then the world entity itself |
| a sophont | the same, with naming and memory; it can assert, own and hold tenets, and be petitioned |
| a god | ruling 42's divinity, and the natural case, since the world is already the provenance root every ascent is promoted into |
| dead | ruling 61's finality with ruling 62's residence: a dead world is one whose entity is on the planes, its body still the ground |

What that buys, each from a ruling already made. *Agentless processes have a
body.* §3.3's second shape, weather, decay, erosion, fire, is the world
entity's own metabolism, Mesocosm's one verb at the root: "world into self,
self into world" is now literal at both ends, and when the world is a
critter or a sophont its acts are those same processes chosen. *Geology is
anatomy.* A world's stuff is its parts, so the body pipeline's parts and
scruples describe a crust, a vein of ore, an aquifer, and a world's kinds of
nis are its own trait catalogue, which is the material typology ruling 97
asked for at the root. (Ruled 2026-09-24, ruling 140: that catalogue lies
on a spectrum, from a normal base world that matches a ruleset, through a
base with variants, to a completely generated one.) *Constructs close the loop.* "Stuff can be made out
of the world, even critters": a golem or an automaton is ruling 45's
construct with the world's nis as its matter, and every producer already
makes its own nis out of the world's. *Divinity and the planes line up.* A
god is intrinsic to the world's provenance because the world is the entity
whose provenance that is, and a plane, a world in relation (ruling 11), is
another such entity. *Founding chooses or draws it.* What kind of entity the
world is belongs to the short flow of ruling 89 beside seed, shape and
ruleset, or to the seed if left blank, which is the strange-scenario knob
Mark keeps asking for, "a world built on a critter", given a home.

**World is a kingdom, and scale stays orthogonal (ruling 100).** Ruling 39's
two axes hold: *world* joins flora, fauna, myco and micro as a kingdom, and
a world's scale is macro, "the basic" case, with meso and micro worlds left
open: "small moon? satellite? asteroid? ship?" *Reading, added to D29, accepted 2026-09-22.* The
lion turtle question dissolves: kingdom being class and scale orthogonal, a
lion turtle is macro fauna, and a world resting on a critter is a
world-kingdom entity resting on macro fauna, unless a founder makes the
critter the world itself (ruling 99). The three scales of the world kingdom
are ecology's own words, microcosm, mesocosm and macrocosm, and Mesocosm's
enclosure is by its own description "larger than a microcosm, smaller than
the world": the game's terrarium is a meso world, and the germ's host of
ruling 39 is a micro one. A ship as a meso world is ruling 45's construct of
the world's kingdom, with its own nis, its own fields of air and warmth, its
own nesting into decks (ruling 74), and the vessel of ruling 90's orbit
example and ruling 41's space scope; it rests on or within a macro world as
a critter does. **Open:** whether a planetary system (ruling 11) is an
entity of the world kingdom at a scale above macro, which ruling 39's three
scales do not name, and so whether a sun has a provenance.

Prior art for rulings 98 and 99, known. Norse creation makes the world from
the body of the giant Ymir, his flesh the earth, his bones the mountains,
his blood the sea, and the Chinese Pangu and the Babylonian Tiamat are the
same myth, which is "stuff can be made out of the world" with the world a
dead critter. Discworld's Great A'Tuin is the world on a turtle, and
Lovelock's Gaia hypothesis is the world as one self-regulating organism, the
sentient case argued in earnest. From memory of games, unverified: Noita
gives every pixel a material with interactions, living and inert in one
system, and Dwarf Fortress's stone layers carry material properties as its
creatures' tissues do.

Prior art for ruling 97. Known: Monster Hunter is this loop as a shipped
game, study a creature, hunt it, and craft armour and weapons from its
parts, each set named for the creature, so learning the ecology is the
progression; and for most of human history the toolkit was organisms, bone,
antler, sinew, hide, gut, horn, shell, wood and silk, a bow being yew, horn
and sinew, with stone and metal the late exceptions. From memory of games,
unverified: Dwarf Fortress gives each creature its own bone, shell, leather
and silk beside stone and metal; Valheim's materials come from each biome's
creatures; Kenshi's from its fauna.

**Quality (ruling 96).** "Is each sword, axe, and dagger the same?" Mark
asks what meaningful variation there can be "given the directional
combat/strike quality system", whether Mount & Blade's stats and tiers are a
good method or there is one with "less number comparison", and notes that
tiers could gate techniques within a bucket and that improving a sword over
time is fairer than "can't swing that sword". *Recommendation, docketed as
D27, Mark's to rule.* Quality as **affordance**, not as a number: what a
weapon lets its wielder do, read by the strike system, and never a stat to
compare.

- *What already varies.* Checked 2026-09-22: Eponym resolves a strike
  geometrically, a swept volume against a body's part bounds with line of
  sight through the ground, and the hit's `quality` is the overlap
  (`eponym-world/src/combat.rs:53-60,218-232`, `combat/precision.rs`);
  harm is a base plus that quality. So the system already reads a strike's
  quality off geometry, and a weapon's shape, reach and sweep are already
  inputs it could vary on with no numbers added. Body and part vary too.
- *Four sources of variation, all already in the record.* Sort: the shape,
  which sets which strikes exist at all, thrust, cut, chop, hook, reach.
  Material: ruling 96's typology of kinds, which sets what a blade does
  against what, edge against flesh, weight against armour, and how it wears.
  Make: the maker's skill and the act it came from (ruling 95), so a
  well-made blade has its balance and edge where its sort wants them and a
  poor one does not. Condition: what has happened to it since, wear,
  notches, a reforging, notes of ruling 80.
- *Read as affordance.* Those four resolve into which techniques the weapon
  affords, which is Mark's "tiers could gate techniques, within a bucket"
  made the whole model: a sword of a given sort and make lets a swordsman of
  a given skill perform some techniques cleanly, some poorly, and some not
  at all, with the geometry doing the work. "Better" is then answered the
  way a player answers it in a game with movesets rather than stats: this
  blade takes the drawing cut, that one will not hold an edge, and one
  glance at a short list of affordances says so. No comparison of numbers,
  and no weapon a skilled hand cannot swing, only ones that afford less.
- *Improvement is fair because it is an act.* Improving a sword over time is
  a recipe with the item as an input (ruling 95), sharpening, rebalancing,
  reforging with a better material, so an item improves by the same process
  shape as anything else, and a sword that has done things of note carries
  notes that may afford a technique of its own, which is how a named blade
  comes to be. That is the fairer path Mark wants: no gate, a journey, for
  the sword as for its wielder.

Prior art for the quality question. Known: Mount & Blade is Mark's
reference, sorts and qualities as stats and tiers, the method to improve on.
From memory of games, unverified: Dark Souls answers "better" by movesets
per weapon class and upgrades the one weapon you carry, which is affordance
plus improvement; Elden Ring lets techniques be attached to a weapon;
Brogue's whole progression is enchanting one chosen item, the strongest case
that improving what you have feels fairer than replacing it; Monster Hunter
shows sharpness as a gauge that wears and is honed, condition without a
number; Dwarf Fortress crosses a quality grade with material properties, so
a masterwork copper sword and a plain steel one differ in kind and not on
one axis; Kingdom Come: Deliverance wears and sharpens weapons at a
grindstone.
Checked 2026-09-22: Eponym's `Work` is a craft, a grade and a danger, over
a closed `Craft` of five (`offer.rs:9-12`, `companion.rs:24-30`), which is
capability as a level and not yet as a journey; wing-glyphs' `Acquisition`
keeps glyph, provenance, tick, life and canon revision (`journey.rs:51-63`),
the shape a technique's record would take.

Prior art for ruling 95, known. Dwarf Fortress's strange mood is the
reference Mark names: a rare state that seizes a dwarf, demands particular
materials, and yields an artifact or a breakdown, with the artifact kept
forever, which is a triggered need, a recipe with preconditions, and the
hagiograph in one mechanic (its details from memory, unverified). Ingold's
*Making* (2013) argues that making is a correspondence between maker,
material and act rather than the imposing of a form, which is crafting as
incorporation. Utility-based AI and GOAP choose acts by scored needs against
capabilities, the same score as the tenet from the actor's side. From memory
of games, unverified: Skyrim's smithing is the churn Mark rejects, a hundred
iron daggers levelling a skill; Kenshi and Mount & Blade level by use with
diminishing returns, a middle case.

**Already in code, checked 2026-09-21.** Both products hold the bottom tier.
Mesocosm keeps a conserved material ledger and reads starvation from it
(ruling 38); Eponym keeps `Needs` on the body and `party_resources` as
named stores per owner in the tabletop
(`isometry-campaign/src/world/types.rs:60-64`), with "the system decides
what a store is worth". No product derives a price or moves goods along
routes, and the world-conditions schema keeps matter, energy, charge,
attention, time and social obligation as distinct accounts that "an
operation cannot exchange without an authored transform", so an exchange is
an authored operation over those accounts and never an implicit conversion.

Prior art for ruling 94, known. Maslow's hierarchy is the Sims' needs and
the popular reference; the more exact one is Max-Neef's matrix of nine needs
against four satisfiers, where a need is universal and its satisfiers are
cultural, which is ruling 38's "the system decides what counts as food"
generalised. For the tiers, BDI is Bratman's and the Sims' needs are
Maslow's; for the aggregate, gravity models of trade (Tinbergen, 1962) make
flow proportional to the two sizes over the distance, and Ricardo's
comparative advantage is why capability differences make trade at all. From
memory of games, unverified: Dwarf Fortress derives value from material and
quality with a civilisation's preferences, and its caravans are trade as
carriage; the X series and EVE Online run supply and demand per station with
prices read off stock; Victoria 3 pools goods per market with prices moving
on the balance of buy and sell orders; RimWorld's price per item is a base
times a trader's markup and a settlement's biome.

**Competition (ruling 115).** When two want the same thing and there isn't
enough, the last of the food, a den, a mate, a throne, each side chooses:
its methodology picks contest, yield, share or trade, and the sim resolves
what was chosen. Trade is exchange (ruling 94); a share splits the thing; a
yield hands it over; a contest is a fight, whose resolution is the sim's own
outcome model under ruling 114 and is not yet designed. For a critter the
choice is reactive, so whether a lineage leans to contest or scramble is a
trait that moves at the epoch boundary (ruling 57); a denizen or sophont
scores it like any act, by need, trust and approval (D26), with its
disposition and its opinion of the other. This is the competition semantics
the [aggregation research](2026-09-22_aggregation_research.md) found missing
for shared resources, and it makes the mix a population statistic: a crowd's
share of contests follows from how its members lean. Prior art, known:
Nicholson's scramble and contest competition (1954), under which scramble
tends to boom and crash and contest to hold a population steady; Maynard
Smith and Price's hawk and dove (1973), where the mix of fighters and
yielders is itself what evolves.

**Contests (ruling 116).** Most contests never reach blows: the sides size
each other up first, by display or bluff, and most end there; only close
matches escalate, and then one side breaks. *Reading, not ruled:* what a
side sizes up is what it can perceive of the other, its body, gear, numbers
and display, and what it knows of it, reputation arriving through the reach
field as ruling 56's standing, so a thing with a fearsome name wins most of
its contests before they start; a bluff is a display beyond what stands
behind it, and it holds until a close match calls it. Prior art, known:
resource-holding potential (Parker, 1974); red deer stags roaring, then
walking in parallel, and fighting only when closely matched (Clutton-Brock
and colleagues, 1979); the sequential assessment model (Enquist and Leimar,
1983).

**Harm (ruling 123).** When a contest does reach blows, a blow drains vigour
first and wounds a part of the body's tree when it lands hard or the vigour
is gone; vigour comes back with rest, and wounds heal slowly, or never. Both
are written to the one ledger of ruling 38. *Reading, not ruled:* vigour is
the fight's short draw on the body, read against the fatigue and reserve
the ledger already keeps, and a wound is a loss in a part's structure, so a
cut leg slows and a lost eye blinds by what the part tree no longer
affords; a side that runs out of vigour is a side that breaks (ruling 116);
and a ruleset's hit points calibrate against vigour more than wounds, since
5e's own text treats hit points as durability, will and luck rather than
flesh. Prior art, known: vitality and wound points (the d20 Star Wars
roleplaying game, 2000, later a D&D variant); RimWorld and Dwarf Fortress,
which wound parts and keep no pool.

**Mood and strain (rulings 158 and 159).** A mind's mood is read, never
stored: it comes from what the mind remembers, the tales it holds weighed by
how intense and how good or bad they were and fading as they fade (ruling
127), from what it needs, and from what it is living through, its vigour,
wounds, danger, company and place. Strain is kept: it builds on the ledger
while mood stays low, as vigour drains in a fight, and only rest or relief
bleeds it off. Its personality traits (ruling 158) modify both. *Ruled
2026-09-24 (rulings 161 and 162):* past what a mind can bear, it stops
heeding directives and follows its own needs until it recovers, acts out in
a break drawn from its traits and situation, or rises instead, a
breakthrough such as the fey mood or a last stand; and a break leaves a
mark, seeding or feeding a personality trait. Breaking does not spread: each
mind breaks alone, so a riot is many minds breaking for the same reasons.
*Ruled 2026-09-24 (rulings 163 and 164):* whether a break goes down or up
turns on the mind's traits, the moment, what is at stake and who depends on
it, and a seeded draw weighted by both; and what a mind can bear is set by
its traits, its support, company, faith, a home and what it believes in,
and what it has been through, strain survived hardening it and past breaks
scarring it. Prior art,
known: RimWorld's mood as a sum of timed thoughts with break thresholds and
inspirations; Dwarf Fortress's accumulated stress, its tantrums, and its
strange moods, the fey mood of ruling 95; Darkest Dungeon's stress meter,
ending in an affliction or a virtue.

### 3.4 The record

The hagiograph judges what is significant by novelty, quality and relevance
(ruling 4), the same test at every rung. Storage stays in journals. The
organ is live and consumed by one product, not pending and not yet
wing-wide: mesocosm-core depends on hagiograph, muniment and nisus, while
eponym-world carries none of them and its nearest thing is a
per-event-kind glyph grant table with no significance gate
(`eponym-world/src/glyphs.rs:233,357-372`; noted 2026-09-18 by W1).
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

**How knowledge resolves (ruling 84; the five points agreed, ruling 86).**
Mark asks for information spread modelled "cheaply and distributionally",
with "how a person learned a thing" resolved "without mapping each step
ahead of time", and whether that is the best way. It is, for the background,
and the record already holds every part of it.

1. *The field stores arrivals, not histories.* For an event that spreads,
   each place it reaches keeps three numbers: when it arrived, from which
   neighbouring place or carrier kind, and at what strength. Reach at any
   later time is that strength under ruling 5's decay, computed and never
   stored, and a field that has decayed everywhere is collected like
   anything else (ruling 70). A legendary event keeps a floor of being known
   everywhere and needs no per-place entries.

2. *Whether someone knows is a seeded draw.* For a background entity the
   chance is the reach at the places it has been while it was there, shaped
   by its exposure and by how much the event matters to it, its relations
   and alignment. The draw is seeded by the entity and the event, so the
   same question gets the same answer every time it is asked. A traveller
   knows more than the town because the places it has been are part of the
   draw.

3. *How they learned it is sampled backward, on demand.* The arrival entries
   are a tree back to where the event happened. Asked how someone knows, the
   sim walks it, "by the trade road from the coast, from a carrier at the
   market", and lifts (D17) a teller out of the carrier cohort only if
   someone needs to meet them. Nothing is mapped ahead of time, and the
   answer agrees with what a forward simulation would have produced.

4. *Once resolved for something cared about, it is a note.* The answer
   becomes an impresa record on the knower, its `discover` kind citing the
   event (ruling 82), so it cannot change under a player's eyes. That is
   ruling 75's pattern again: fields for the fungible many, notes for the
   few.

5. *Who knows more than their place:* carriers, who have been elsewhere;
   bearers, since a book or a charter is an item bearing notes and a library
   is a location full of them; and rank, which opens a polity's records. All
   three are things the sim already has. The one case a field serves badly
   is a secret, known to three people and no distribution at all; that is a
   set of notes spreading along relations and not places, taken up under
   ruling 87 below.

*Ruled 2026-09-24 (ruling 117): a belief can be wrong.* The five points
resolve whether someone knows; ruling 117 adds that what they know may be
false, and that they act on it. In Mark's terms it is "'wrong' information
or stale information". Wrong information arrives false, by deceit, honest
mistake, or a retelling that changed it, so the field carries versions of an
event and an arrival entry names the version that arrived. Stale information
arrived true and has been overtaken since: every belief is dated by its
arrival and the world may have moved on, which is the field's own delay and
needs nothing new stored. *Reading, not ruled:* posing, Mark's "posing as an
action to intimidate/persuade/deceive", is an act of the first shape whose
outcome is a belief in whoever receives it; to intimidate is to pose
strength, to persuade is to move a belief or an opinion, and to deceive is
to plant a false one; ruling 116's bluff is posing inside a contest; and a
ruleset's charisma is its reading of the actor's capacity for posing,
calibrated under ruling 114.

*Ruled 2026-09-24 (ruling 118): what makes a belief take.* Four things
count, and all four were ruled in: what the receiver can check, since
evidence outweighs what is told, so a lie lasts only where it cannot be
checked and a stale belief falls when fresher news arrives; who is telling,
the receiver's trust in the teller by its opinion of them, their reputation
and their rank; what it wants to hear, the claim's fit with its tenets,
loyalties and hopes; and how it is told, the teller's practised skill in
ruling 96's sense, which a ruleset reads as charisma. How the four are
weighed when they pull apart is ruling 119. Prior art, known: source
credibility (Hovland and Weiss, 1951); motivated reasoning (Kunda, 1990);
Bayesian updating as the evidence term alone.

*Ruled 2026-09-24 (ruling 119): how the four are weighed.* Each entity's
disposition shifts a world baseline, and the baseline is an aggregate one
drawn from the world entity's disposition (rulings 98 and 103): a world's
temperament is its believers' normal way. The sim's default is a neutral
median, "the world's normal way" when nothing sets it otherwise, which is
presumably what a world without a disposition of its own takes, since
ruling 103 gives one only to an agentive world. *Refined by rulings 120 and
121:* the baseline is the population's own aggregate, seeded from the
world's disposition in the first lineages and moved after by what the
world's conditions make of its people, so it drifts.

**Culture and language (rulings 165 and 166).** A culture is read from its
people: what a population shares in tenets, taboos, customs, ways of deciding
and words, read off its members as a faction's alignment is, with no state of
its own. Once minds hold it as a thing it is named and remembered, as a place
becomes a location (ruling 129). Languages divide: understanding needs a
shared tongue, learned like a skill (ruling 96), so telling, posing and
exchange across tongues come harder, and interpreters matter. *Reading, not
ruled:* a tale that crosses a tongue is retold, so translation is one more
way a belief goes wrong under ruling 117. Prior art, known: Axelrod's
dissemination of culture (1997), where neighbours who already share traits
converge further; Dwarf Fortress's generated languages and Ultima Ratio
Regum's generated cultures. *Ruled 2026-09-24 (rulings 167 to 169):* tongues
and cultures descend like lineages, splitting into dialects when their
speakers part and drifting over deep time, so languages have family trees
as critters do; names come from a tongue's sounds and words, from tales as
epithets, and from whoever names a thing; and culture spreads by contact,
faster between peoples who already share much, by prestige, and by
imposition, and drifts apart where peoples are out of touch. Prior art for
descent, known: historical linguistics' family trees, and phylogenetic
methods now borrowed from biology to build them.

**Secrets (ruling 87).** "A secret is valuable... it is tremendous leverage,
often, or there wouldn't be rules against its propagation", the word then
corrected to "not rule, but 'taboo'"; and whether one is told turns on
"relation, opinion, and goals", with personality wondered about. *Reading,
docketed as D21, accepted 2026-09-22:*

- *What makes a secret* is the taboo and not the information. In the
  record's terms a taboo is a tenet (ruling 50) holding a strongly negative
  opinion of an act, here the telling; in the accepted classes of §3.2.2 it
  is a norm, a "must not" with no formal "or else", whose sanction is
  reputational. So a faction can keep secrets with no polity behind it, and
  a polity may add a rule on top.
- *Why it is leverage.* Reputation is the reach of one's deeds among those
  who know of them (ruling 56). A secret is a deed whose reach is being held
  down, and its value is what releasing it would do to its subject's
  standing among those who would learn it, which their tenets already say.
- *Telling is a choice under scarcity by a knower,* and the wing has the
  rule for it: Eponym's willingness gates, would I risk that for you, is
  it more than I would bear (§3.2.2), with the taboo's sanction as the
  danger and the hearer as the asker. Relation and opinion are its trust and
  affinity; goals are what telling buys. Personality is not a fourth factor
  but the thresholds: that rule already reads `bearable(affinity, caution)`
  as "3 + affinity / 2 - caution" (`eponym-social/src/willing.rs:40-42`),
  caution selling what affection buys, and ruling 37's five factors supply
  such terms.
- *The regime is notes, not a field.* A secret is an explicit set of
  knowers, each holding a note. Eponym holds this regime already, checked
  2026-09-21: an "append-only, observer-scoped record of claims about
  accepted deeds" whose evidence is "a direct sighting or an addressed
  transmission" (`eponym-social/src/epistemic.rs:7-11,37-43`).
- *A secret ends by leaking.* The moment a telling seeds the reach field at
  a place it is a rumour, and points 1 to 3 take over. That is ruling 75's
  transition from the few to the many, run once.

Prior art for ruling 87, known. Simmel's "The Sociology of Secrecy and of
Secret Societies" (1906) treats the secret as a social form that binds those
who share it. Arrow's information paradox (1962) is why secrets trade badly:
a buyer cannot value one without learning it. Dunbar (2004) reads gossip as
how groups police their members, which is ruling 64's enforcement by
reputation seen from the other side: rumour is the channel by which a norm
with no "or else" is enforced, and a secret is the move against it. From
memory of games, unverified: Crusader Kings III keeps each secret as a set
of named knowers and turns an exposed or threatened one into a hook, which
is leverage as a mechanic; Caves of Qud trades secrets as goods. From memory
of personality research, unverified: a tendency to gossip goes with
extraversion and against agreeableness and honesty.

Prior art, known. Kingman's coalescent (1982) is the proof that this is
sound and not a shortcut: population genetics samples the genealogy of only
the individuals it cares about, backward in time, and the result is
statistically identical to simulating the whole population forward. The same
trick answers a lifted denizen's ancestry. The Daley-Kendall rumour model
(1964) and metapopulation epidemic models are the forward half, compartments
per place with mixing inside and carriers between, which is a field on a
place graph; epidemiology's transmission trees are the arrival entries.
Hybrid stochastic simulation, exact for small counts and leaping for large,
is the two regimes. From memory of games, unverified: Dwarf Fortress tracks
rumours per historical figure with who heard what from whom, the explicit
and costly form, and Crusader Kings III keeps each secret as a set of named
knowers, which is the regime for secrets.

### 3.4.1 Three tiers of keeping

From ruling 69. The middle tier is Mark's "tier between primordial sim soup
things generate from and legendary things. Just stuff of note", and it holds
for every kind of thing the sim has:

| Tier | What is stored | What puts a thing there | Rulings |
| --- | --- | --- | --- |
| Ambient, the soup | nothing of its own: the seed, the rules and the basic facts it regenerates from; populations as distributions, locations as derived nodes | nothing; it is the default of §1 and §3.5, and what a thing of note returns to | 69, 70 |
| Of note | the thing itself, individually: its asserted facts and its deviation record | something of note happened to it or there, a relationship holds it, or someone designated it | 69 |
| Legendary | the record of it, never lossy, retold and memorialised; the significant dead on the planes | the hagiograph's test, unprecedented, legendary or narratively significant, by novelty, quality and relevance | 4, 62 |

The middle gate is lower than the hagiograph's and is not the hagiograph's.
For an inhabitant the middle tier is the **denizen**, by the terminology
supersession of 2026-09-20, an inhabitant remembered individually "because
history, relationships, or explicit designation makes it matter". (Amended
by ruling 200: the tier is of note, and an inhabitant of note becomes a
denizen once people name it.) Ruling 69
gives a location the same rule: "if something of note happens there, then it
persists; otherwise it can be regenerated from the same basic facts." So
what asserts a place is an event. Naming it, claiming it and building on it
are not separate routes; they are events of note like any other. This
record then read "of note" as the derivation rule renamed, a thing being of
note when it carries a deviation. **Replaced by ruling 80:** it is "like
literally a note on the site/item/entity. the sim just makes notes in the
way a player would too, generating the note from the event/thing that makes
it 'of note.'" So the unit of the middle tier is a note: it sits on a thing,
it is generated from the event that occasioned it, and the sim and a player
write the same kind. A thing is of note while it bears one. Whether a thing
of note can lapse is **answered, ruling 70**, below.

**Already in code, checked 2026-09-21.** The wing has an organ of almost
exactly this shape. `shared/wing-impresa` holds "what a pointable thing has
come to be associated with, and what has lapsed, over time", for "a glyph, a
critter, a borg, a character, a faction, a place, an item and a lot of
matter" alike (`src/lib.rs:4-7`). Its one record type names a subject and an
object, a kind from an open set seeded with claim, discover, experience,
embody, invoke and defeat, the tick, a stance of `Associate` or `Lapse`, and
a `cause` that is "the evidence string of the accepted event this record
cites", and "every field is what an accepted event already knew, so writing
one is transcription and never inference" (`src/impresa.rs:35-57`,
`src/kinds.rs:16-23`). That is a note generated from the event that makes a
thing of note, with lapsing built in. **Answered, ruling 82:** the note is
the impresa record, so the middle tier is the set of things bearing a live
impresa and collection is lapsing; and beside "the generated details", where
they can be enumerated, Mark wants a freeform text field, "something
simple, accessible, and interpretable", its format a setting with djot
preferred, edited what-you-see. Checked 2026-09-21: djot is already the
stack's document format and the editor exists. `knot-document` is a
"Djot-first local document authority and reusable Knot surface" and
knot-editor is a "files-in-place, local-first Djot editor" that also reads
Scrolltext and Gemtext, so both the preference and the setting are met by
what the stack has, and §9.8 already names knot-editor for editable text.
*Finding for the organ's owner,* filed in the general model plan: the
record type is closed today (`deny_unknown_fields`, schema version 1) and
holds to "transcription and never inference", which authored free text is
not, so whether the text is an optional field on the record or a sibling
beside it is the owner's decision. **Ruling 85 proposes the answer:**
"closed subset, open superset". Checked against the crate, it fits its own
doctrine: `wing-impresa` is "the record type and its validation only" and "a
product owns storage" (`src/lib.rs:9-11`). So the closed record stays
exactly as it is, the validated core of every note, and a note is an open
envelope around it, held by whoever stores it, carrying the freeform djot
text and whatever else a product adds. Authored text never enters the
kernel, "transcription and never inference" holds, and no schema bump is
owed. A note a player jots with no event behind it still has a closed core:
the jotting is the accepted event, under a kind the world adds to its kind
set, which the crate already lets a world do.

**The ambient tier (ruling 70).** The soup "is equivalent to the ambient
tier, the background of information generated by an engine and localized to
a particular address/place/scene", which Mark sees "as a characteristic of
mere, and a wonderful sign for an app when ambiance can be created and
used". Checked 2026-09-20: the phrase "ambient tier" is in neither
repository's documents, so it is recorded here as Mark's; the idea is in
both. Woodshed's musical projections plan states an **ambient context
boundary**: "available catalog material, currently displayed material,
inspected focus, and authored Card occurrences are different states"
(`woodshed/design_docs/2026-09-04_musical_projections_plan.md:316-317`).
Those four are this record's ladder under other names: what the engine can
generate at this address, what is realised in the scene, what is
foregrounded, and what is asserted. Mere's doc index summarises its physics
scenes plan's living backdrop as "ambient, cheap, default-intangible"
(`mere/design_docs/DOC_README.md:331`), a sim painted behind what can be
touched, and its meaningful physics signals plan has an ambient pulse that
reads live runtime state (`:233`).

**Returning to the soup is garbage collection (ruling 70).** "Things can
indeed return to the soup by being made irrelevant to the player in some
manner; that's garbage collection, right? So the criteria for invalidation
is crucial." So of note is not permanent. Only legend is, by ruling 4.
Woodshed's same passage already carries an eviction policy worth reading as
prior art from inside the stack: at capacity it offers "to replace quiet,
unpinned background", it must "protect authored, focused, and pinned
material", it must "preserve coordinates when material returns", and it must
not "silently evict" (`:322-331`). In the sim's terms: the ambient is
evictable, the asserted and the foregrounded are protected, and a thing that
comes back is regenerated at the same address from the same basic facts.

*Reading, accepted by Mark 2026-09-21 (D11).* Taking
"garbage collection" at its word gives the criteria a shape: roots and
reachability. The roots are what is relevant to a player: the foreground,
and what the played entity is, holds, knows and is related to; every
asserted fact that a live thing still relies on, a claim, a membership, an
agreement, a constitution; every pending due event; and the legendary tier
whole, which is never collected. A thing of note is kept while something in
the record reaches it from a root, by a relation, a possession, a memory
within reach (§3.4), or a cause-link. Unreached, it is collectible, and
collecting it folds it back into the distribution it was realised from, so
counts and masses stay conserved and only its deviations are lost. The test
that makes this safe is observational: collect only what nothing reachable
from a player could tell apart from its regenerated self. Who "the player"
is was **answered by ruling 71**, below. **Open, and crucial:** the rest of
the criteria. Whether collection must be deterministic across peers is
answered by §3.12's recommendation, D32, if accepted: it must, because a
merge replays.

**The roots, and graded collection (ruling 71).** The roots are care. "The
players of any of the three games, when the sim is applied to them, have
effectively selected/created entities they care about": the lineage and its
critters in Mesocosm, the one sophont and those it knows in Eponym, the
characters and what the table authored in the VTT. With nobody playing
nothing changes, because "you only render things with the upmost detail when
you're examining them anyway": examination is the root when nobody plays, so
the bench's observer, a deep-time run's handover and a player are one thing
to the collector, whoever is attending. Several players in one world are
then several sets of roots, and a thing is kept while any of them reaches
it.

Mark's two cases set the shape. "You wouldn't trash a colonist's relative
that has appeared before": kept by a relation to something cared about
**and** by having appeared. Both halves matter. A relative who has never
appeared carries no deviation and is regenerated on demand from the relation
itself, which is the observational test of the reading above at work. "You
would trash most of a raider that got killed with little incident; at a
certain point just the event": collection is graded, never all or nothing.
*Reading, accepted by Mark 2026-09-21 (D12):* a thing of note sheds in steps, the full individual, then a stub,
what an event needs to name its participant (a kind, a lineage, a faction, a
name if it had one), then the event alone. So events are the floor of the
middle tier, and they refer to their participants by stub, never by a live
individual, so that a participant can be collected without breaking the
event that mentions it. **Open:** whether an event of note itself lapses in
the end, as ruling 5's forgetting suggests, with legend the only floor that
never gives way; and what "little incident" measures, which is presumably
the same novelty, quality and relevance at a lower bar. *Ruled 2026-09-24
(ruling 127), the first half:* everything fades unless renewed, at a rate
the holder's memory sets, "in dnd terms, intelligence", ruling 55's graded
memory; a note kept on a bearer, a journal, a notebook, a diary, does not
fade and lasts as long as the bearer does; legend never fades. "Little
incident" stays open. *Ruled 2026-09-24 (ruling 129):* the note's holders
are whoever remembers the thing and whatever records it, each mind's copy
fading at its own rate; when the last is gone, the thing returns to the
ambient. So the middle tier is the world's own: of note means remembered in
the world or written down in it. *Reading, not ruled:* this makes the
observational test safe by construction for the world's minds, since what
no mind or bearer holds, nothing in the world can tell from its regenerated
self; a relation still keeps a thing's stub while it names it; and players
are the case left open, since a player remembers outside the world.
*Ruled 2026-09-24 (ruling 130): players.* A player keeps a thing of note
through the entity they play, whose memory fades at its own rate, and
through what they pin. In Eponym a player's notes are diegetic, the
sophont's own, a bearer in the world that can be found, read, lost or
stolen like any other; in the VTT and Mesocosm they sit outside the world.
Notes external to a game may later live in the app through knot-editor
(§9.8). *Ruled 2026-09-24 (ruling 131): what a mind remembers.* What was
new to it, what mattered to it, what was intense, and what kept happening;
a "little incident" is none of these to anyone who saw it, so no mind keeps
it and it never becomes of note. *Reading, not ruled:* these are ruling 4's
novelty, relevance and quality at a mind's own bar, with repetition added,
and repetition is also what renews a fading note under ruling 127. Prior
art, known: the von Restorff effect for novelty, the self-reference effect
for relevance, flashbulb memory for intensity, and Ebbinghaus's forgetting
curve with spaced repetition for the rest.

Prior art for ruling 71, checked 2026-09-20 from web search summaries of the
Steam Workshop pages for Better GC (id 2982026860) and RuntimeGC (id
962732083), the decompiled source itself not read. RimWorld's `WorldPawnGC`
keeps world pawns that are important, a faction leader, a kidnapped or
quest-reserved pawn, a caravan member, a pawn whose corpse is on a map, and
keeps any pawn with a relation to another or an appearance in a log or a
used tale; the rest are discarded, and kept pawns that need no simulation
are *mothballed*, held without ticking, which is this record's idle thing
that costs nothing. Its known failure is the caution for the criteria:
because any relation pins a pawn for good, long saves are reported to
accumulate some two thousand uncollectible pawns, and mods exist only to
collect harder. Mark's grading, most of the raider and then just the event,
is the fix RimWorld lacks. From memory, unverified: RimWorld's tales keep a
snapshot of each pawn they mention, so a tale survives its pawn's discard,
and its tale kinds are volatile, expirable and permanent, which would be the
three tiers again under other names.

Prior art for ruling 70. Known: tracing collection is exactly roots and
reachability, and the generational hypothesis, that most objects die young,
predicts that most things of note lapse soon and few are ever promoted, with
legend as the tenured generation. For the mechanism rather than the
criteria, forest-rs's `invalidation` crate (surveyed 2026-07-21, not
re-read) is channel-based dirty tracking with deterministic ordered drains.
From memory of games, unverified: RimWorld collects world pawns that no
colonist relation, quest or faction role still references; Crusader Kings
III prunes the dead who have no dynastic or historical bearing; No Man's Sky
regenerates worlds from the seed and keeps only a bounded set of player
edits, the oldest lapsing first; and Dwarf Fortress, which never culls its
historical figures, is the caution about what permanence costs.

**Already in code, checked 2026-09-20.** Eponym's
`eponym-world/src/sites.rs` holds the two lower tiers for places. A
`SlotId` is "a structural address. Its occupant may change without changing
its containment or routes" (`:19-25`). A `Site` sits in a slot with a kind,
wilds, settlement, ruin, encounter or dungeon, and a source that is either
`Generated` or `Inherited(HistoryFactId)` (`:46-67`): regenerable, or kept
and pointing at the fact that keeps it. That also answers "what keeps it the
same place" as the code stands: the slot. A razed castle is the same slot
with the kind ruin, and "changing a site's meaning cannot move it
accidentally" (`:69-70`). What the slot rests on is mesocosm-core's
`PlaceId`, a fixed partition today (§3.7), so the address survives because
the geometry is never re-derived. Under volume-derived nodes a place of note
needs an anchor that re-derivation maps onto and cannot erase. **Open:**
what that anchor is when the ground itself moves. *Ruled 2026-09-24
(ruling 148):* the location's kind decides: a camp follows its people, a
ruin its ground, a battlefield its site.

**The words.** "Denizen is now a term for an entity of note. There should be
similar terms for locations, possibly more." Naming is Mark's round and
nothing is coined here; this is what the wing already holds, checked
2026-09-20. The founding record's frame already pairs the three: "people
(subjects and their deeds), things (relics with provenance), and places
(sites with history)" (`2026-07-30_games_wing_founding.md:185-187`), so
*site* and *relic* are its words for a place and a thing with history, and
Eponym's `Site` is live code. *Site* collides inside the wing: it also
names a location on a body in Mesocosm's phenotype
(`mesocosm-core/src/phenotype/mosaic.rs:50,68`) and in
`shared/wing-functions/src/generation.rs:21,29`, and the tabletop's overmap
has an `AtlasSite`. (Ruled 2026-09-24, ruling 157: a thing of note is a
*relic*, the founding record's own word; an event of note is a *tale*, and
tales fade where legends do not; and Mesocosm's body site becomes a
*tract*, freeing *site* for the world map, the rename a lane. Eponym's
*deed* stays its record of an act.) For events the hagiograph's *feat* and *mark* are words
of the top tier, a feat being what beats a standing mark, while Eponym's
*deed*, a recorded act with a doer (`eponym-social/src/deed.rs`), is the
nearest thing the wing has to an act of note. On crates.io, exact names, the
only registry checked: taken are site, landmark, relic, locus, haunt, locale
and keepsake; free are stead, feat and heirloom. The kinds that may want a
middle-tier word are the inhabitant, which has one, the place, the thing,
the event, and perhaps the group and the lineage.

*On ruling 70's two naming points.* "There could be a more anatomical term
for mesocosm's": Mesocosm's `Site` is "one expressed process: what it is,
and which tissue it occupies" (`mosaic.rs:66-77`), a patch of cells on a
part, and the same file's prose already calls these organs ("ordinary
grazing does not shuffle organs", `:34`). Renaming it frees *site* for
places. The vocabulary is 104 occurrences in 26 source files across Mesocosm
and `shared/wing-functions`, and it is serialised, so the rename is a lane
with format and golden-trace consequences, not an edit. Anatomical words
that fit the sense, none checked or coined: *organ*, which the wing also
spends on its engine organs; *locus*, genetics' word for a position;
*situs*, anatomy's Latin for where an organ lies; and *anlage* or
*primordium*, embryology's words for the tissue an organ develops from,
which would suit the proposed and declared forms. *Ruled since:* tract for
the expressed process (ruling 157), attachment for where an incoming part
joins (251), and situs for an organ's position in a plan's template (252).

"Does this need to be a crate, or can it be a component of the wing?"
*Recommendation, accepted by Mark 2026-09-21 (D13):* a component, one module of the sim, and
not a crate per word. The three tiers are one mechanism over every kind of
thing, and the invalidation criteria are one policy, so a `denizen` crate
beside a place crate and a thing crate would hold the same keeping machinery
three times. Nothing consumes the of-note tier without the sim, which is the
usual reason for a crate boundary. It can still be built to be tested and
swapped alone behind its own seam inside the sim. The kinds are then data:
the inhabitant, the place, the thing, the event. No crate is published
for the name: the organ words are components, and a `denizen` crate is made
only if it becomes the way entity information moves between contexts
(ruling 201).

*Ruling 72 settles the place words.* A place of note is a **location**, and
a **site** is one cell of the world map (§3.7.1). So the collision to clear
is between the world-map site and Mesocosm's body site, now the tract
(ruling 157), and Eponym's
`Site` and the tabletop's `AtlasSite` both name what ruling 72 calls a
location.

### 3.5 Holding hundreds of thousands of things

- Aggregate what isn't foregrounded: a population is a distribution, and
  an individual is realized from it when something needs one.
- Schedule events instead of ticking entities: a priority queue of due
  events; an idle thing costs nothing.
- Store deviations, derive the rest (§1).
- Conditions live on places, not on things.

Memory is not the constraint at this scale; processing is, and the four
rules make it proportional to what is happening.

*Amended 2026-10-01 by ruling 430:* the fourth rule reads "the environment's
conditions live on places". Things read rain, heat and plague exposure where
they stand, and none is copied onto each thing. A thing keeps its own
condition records (428) only for states its ledger and body do not hold;
their ends are scheduled events, as the second rule wants, and a crowd groups
by what the rules read, so a record's cause splits no bin (207).

*Ruled 2026-09-24 (ruling 124): the first target is a region.* At its
largest, a seeded draw runs a region on Mark's laptop: hundreds of sites,
tens of thousands of critters, hundreds of them named, a century of history
in a few minutes. The continent this section's title claims, and a whole
world, are later targets and not precluded.

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

**The spine (rulings 389 to 392, 2026-09-28).** What connects the world
map's sites to the voxels under them is planned whole in the
[place-graph engine plan](2026-08-05_place_graph_engine_plan.md), rewritten
as the spine's plan (390). A top-down skeleton gives each site coarse facts,
each adjacency carries an edge profile both sides compute alike (391), and
Isocosm lifts a site's volume from those facts and its seed (392). The first
slice builds that lift without moving `mesocosm-core`'s places family ahead
of ruling 195 or touching mere ahead of ruling 363 (389).

### 3.7.1 The world map: sites, regions and locations

From ruling 72, whose definitions are kept whole in the first column.

| Word | Mark's definition | In this record's terms |
| --- | --- | --- |
| World map | "a grid of sites comprising a world map, in any number of shapes (sphere, ring, plane, cube, spire, wheel, any shape works for me)" | §3.7's graph at the rung above the bricks. Sites are its nodes and the shape is its topology, so it is held as adjacency and never as a two-dimensional array; the tile's shape stays a projection (ruling 18) |
| Site | one cell of that grid, "each with their own terrain and biome (mountainous, valley, island, tundra, tropical)" | the address, and the unit that generation and play work in, paging being the chunk's (ruling 147); its volume is grown from its terrain, its biome and the seed |
| Region | "an area of sites on the world map" | a set of sites; terrain, not an entity (ruling 8) |
| Location | "a known, remarkable, interesting, or potential site (it has additional modifiers/conditions compared to wilderness) or nested region within a world map site"; "they can occupy multiple hexes but be considered the same location, kinda like civ cities" | the place of note of ruling 69, and the word for it: an extent with identity, from part of one site to many sites, carrying modifiers and conditions that wilderness lacks |
| Wilderness | "sites and areas without a location but from which locations could be generated according to their terrain" | the ambient tier for places (ruling 70), holding ruling 14's derived candidate locations |
| Biome | "the characteristic environmental pattern for a site" | a reading, a pattern over conditions, not a stored fact |
| Environment | "the weather and climate conditions" | fields on places (§3.1), moved by agentless processes (§3.3) |
| Ruin | "Ruins could come from sites and locations being reused" | a reading over history: what an earlier location left under or beside a later one |

**What follows.** *Site* joins §3.1's ladder between the cell and the
region. A location's kinds are many and open, "a settlement, a dungeon, a
city, a trading outpost, a fort, a defensive gate, a highway, waaay more",
so they are data to be authored and generated and never an enum, as forms of
governance are (§3.2.2). *Reading, accepted by Mark 2026-09-21 (D14),* on the question that
was asked: Mark calls a biome "the characteristic environmental pattern",
which makes it something read off a site's conditions, and a location's kind
reads the same way, the characteristic pattern of what is built there, who
is there and what it is used for, with ruin as the clearest case since it
"could come from sites and locations being reused". What is stored for a
location is then its extent, its modifiers and conditions, what happened
there and who claims it; the kind is how that reads. A location may be only
"potential", which is ruling 14's candidate, derived from terrain and not
yet of note, so the word spans the ambient and of-note tiers and the keeping
tier says which. A highway is a location that is also a route, a long thin
extent over many sites, so the graph's edges can be places of note too.

**One shape per world (ruling 73).** "A world could be any shape, but just
one. Not like switch between them." So the shape is a fact of the world's
founding, set once with the seed and the world-founding ruleset (ruling 41),
and the graph of sites never changes topology in play. The purpose is
"strange scenarios like a world resting on a critter", which the record can
already say: a macro creature is a terrain-body (ruling 39, and ruling 43's
worldtree), so a world resting on a critter is a world map whose graph of
sites is laid over a body. *Reading, accepted by Mark 2026-09-21 (D16):* what
the critter does, walking, turning, sleeping, dying, reaches the world as
its environment. That is a reason
beyond the sphere for holding the map as adjacency: a body's surface is no
regular grid at all.

**Any shape.** A world map that may be a sphere, a ring, a cube, a spire or
a wheel cannot be an array with a wrap rule; it is a graph of sites with
adjacency, which §3.7 already says of every rung above the bricks. Known
mathematics bears on one case: no grid of hexagons closes a sphere, and a
geodesic grid does it with exactly twelve pentagons among the hexagons, so
"hexes" holds as a projection over most of a spherical map and not as its
storage. **Open:** how large a site is, in voxels and against the paged
chunk of W3. *Ruled 2026-09-24 (ruling 147):* independent. A site is the
unit for generating and playing and a chunk the unit for paging, so a site
spans as many chunks as its volume needs, its size in voxels following from
the world's base unit and settings. How the nesting goes is **answered, ruling 74**, below.

**Nesting is one composable mechanism (ruling 74).** "Composable and
expandable instead of the order and tiering of the nesting being
predetermined", with the VTT's scopes, world, region, area and battlemap,
as "good defaults". So there is one step, a node of a map opening into a
finer map, and the kinds of step a world has are data from its founding
(ruling 41), never code, while the steps themselves are allocated as
locations are generated (amended by ruling 88, §3.9). That is the wing's most load-bearing rule applied to maps,
"do not let a stage grow its own engine" (Mesocosm's CLAUDE.md, the
anti-Spore insurance): no level has rules of its own, only the one step
repeated. It is also §3.1's ladder made honest, since "the far rung is what
the near rung looks like from far enough away" describes a step and never
said how many there are.

**Nesting is the sim's; scopes are a game's (ruling 92), correcting this
record.** Mark is right that the record had put a VTT paradigm into
the sim. World, region, area and battlemap are the VTT's second pillar,
"Maps at every scope, sculpted... Same world, different scopes"
(`design_docs/PROJECT_DESCRIPTION.md:26-29`), and the pillar's own word is
*scope*. The record conflated two things. *Answer to ruling 92's questions,
docketed as D24, accepted 2026-09-22:*

- *Nesting* is the world's and the sim's: a node opening into a finer map,
  allocated as locations are generated (ruling 88), free of scale and with
  no named tiers at all. The only spatial nouns the sim has are Mark's own
  from ruling 72: the world map, the site, the region, the location, and the
  volume under them.
- *A scope* is a game's: the window it frames over that nesting to play in,
  an extent and a grain with the game's grid projected over it (ruling 18).
  It sits with perspective in ruling 6's list of what foregrounding chooses,
  so it belongs to the overlay and W5, not to founding.
- *Each game has its own scopes over the one world.* The VTT has its four.
  Mesocosm has one, the enclosure, "larger than a microcosm, smaller than
  the world", which "names the scale of the enclosure"
  (`mesocosm/design_docs/PROJECT_DESCRIPTION.md:17-19`), and perhaps the
  world map at a lineage's grain when deep time is watched. Eponym has a
  continuous one, "settlements, dungeons, ruins, underground regions, and
  the open world" walked without a cut
  (`eponym/design_docs/PROJECT_DESCRIPTION.md:11-12`). The same node can
  be a terrarium to one game, a dungeon to another and a battlemap to the
  third, which is the pillar's sentence taken literally.
- So: neither of the other games needs those distinctions; each game gets
  its own scopes; and the tiers do not mean different things to different
  games, because the sim has no tiers to mean anything. Ruling 74's "good
  defaults" then reads as the defaults of the VTT's overlay. What a ruleset
  may still fix is grain, its tile as a power of two of the base unit
  (§9.2), which is ruling 41's play scope and not the world's. What founding
  supplies for nesting is only the kinds of location, as data, each saying
  what it opens into.

*Reading, accepted by Mark 2026-09-21 (D15):* what one step has to say, so that steps
compose. *Down*, how the finer map is generated from the node above it: its
terrain, biome, conditions and seed, the "same basic facts" of ruling 69, so
an unvisited interior costs nothing. *Up*, how the finer map reads from
above: its state summarised as the node's conditions, which is the
derivation rule's far rung. *Across*, how adjacency crosses the boundary, so
that leaving a nested map by its east edge arrives in the neighbouring
node's map and a route is one path through several levels. *Ratio*, how much
finer, which for volumes is ruling 13's power of two and for graphs is free.
The last step is the one that meets the volume, where a node's map is bricks
and no longer sites; by ruling 74 its size is a world's setting with a
default, not a constant. Expandable means steps can be added at either end
or in the middle without touching the others: a system above the world for
the space scope (ruling 41's Lancer), the planes beside it (ruling 62), a
dungeon's floors or a ship's decks inside a location, and a body as a map,
which is both the world resting on a critter (ruling 73) and the germ's
world inside its host (ruling 39).

**Already in code, checked 2026-09-21.** Each product holds a piece. The
tabletop's overmap projects a region as a grid of `AtlasTerrainCell`s whose
`kind` is a string from "the open tile vocabulary", with `AtlasSite`s, each
"a geographic site area" with a cell `footprint`, and routes between them
(`isometry-views/src/overmap/atlas.rs:20-38`); its campaign world keeps a
`WorldPlace` with a name, tags, an optional map and an optional position
(`isometry-campaign/src/world/types.rs:78-92`). That is a location over
several cells with open kinds, already. Eponym has the stable address and
the two keeping tiers (§3.4.1), with a closed enum of five kinds and a
surface and underground `Layer`, which is one fixed case of a nested region
(`eponym-world/src/sites.rs:13-17,46-53`). Mesocosm has the place graph
derived from relief over a fixed three-by-three partition (§3.7). The words
are crossed against ruling 72: Eponym's `SlotId` is Mark's site and its
`Site` is Mark's location, and the tabletop's `AtlasSite` is a location too.
Bringing the code's words to ruling 72's is a lane under W2 and W3 beside
the body-site rename of §3.4.1, not an edit. Against ruling 74 the tabletop
nests in two fixed steps: the overmap is "the party's pointcrawl", a graph of
places and routes drawn "above the tactical maps"
(`isometry-views/src/overmap.rs:1-10`), a region projects to the atlas grid,
and a `WorldPlace` may name the `map` it opens into. The steps are real and
their order is code, which is what ruling 74 asks to become data.

Prior art for ruling 74, known. OGC's IndoorGML standard is nearest: space
as cells with a graph of nodes and relations over them, and a multi-layered
space model in which separate layers of cells are joined by inter-layer
connections, so nesting and crossing are data in an open format.
Hierarchical pathfinding (HPA*, Botea, Mueller and Schaeffer, 2004) builds a
graph over clusters of a finer graph with entrances as edges, to any number
of levels, which is the *across* and *up* of a step as an algorithm. Pixar's
USD, now an open standard, composes nested scenes by reference and loads a
payload only on demand, which is composition and paging in one mechanism.
Harel's statecharts (1987) are the formal case for nesting that composes.
The tabletop practice is a hexcrawl holding a pointcrawl holding a dungeon's
rooms, stacked as the campaign needs. The cautions are the fixed stacks:
Spore's stages, each with its own engine, and, from memory and unverified,
Dwarf Fortress's world, region and embark, and Caves of Qud's world,
parasang and zone.

Prior art for ruling 72. Known: the tabletop hexcrawl, from the *Outdoor
Survival* map of original D&D to the West Marches, is this model played by
hand, a hex as the site, a keyed hex as a location, and unkeyed wilderness
generating encounters and places from tables by terrain type; Whittaker's
biome diagram (1975) reads a biome off temperature and precipitation, which
is a biome as a pattern over environment; Civilization's cities spreading
over several hexes are Mark's own reference. From memory of games,
unverified: Dwarf Fortress's world is a grid of region tiles each with a
biome, and its *sites*, its word for what ruling 72 calls locations, span
one or more tiles with a local map nested inside; Caves of Qud nests zones
inside world-map tiles and keeps only the zones a player has touched.

### 3.8 What the sim never does

It never renders, takes input, runs a camera or a turn order. It never knows
a hit point, a dice roll, a system's rules, a loot table, a dialogue line or
a quest. It never pathfinds an individual, checks line of sight or resolves
a single blow. It records that a wolf killed a deer and that a duke fell in
battle; the foreground game resolved both by its own rules, or the
background resolved them by outcome without playing them out. It never
decides what is fun. Mark: "sounds pretty right to me."

*Ruled 2026-09-24 (ruling 114):* the two resolutions agree. The
background's outcome model is the sim's, and a game's ruleset resolving the
same situation with no player choices matches it in distribution, ruling
113's test, checked on the bench through the ruleset's reading of the
ledger as its sheet. What players choose is an input, never a seam. So the
sim owes a background outcome model for every kind of outcome a ruleset
resolves, a fight among them.

### 3.9 Founding a world

From rulings 88 to 91. This is the generator rung of W2, in outline.

**What founding fixes, and the two points Mark asked about.** Founding is "a
basic flow where someone picks the crucial world details in addition to the
seed", with RimWorld's world generation as the reference "but with more than
spheroid worlds" (ruling 89). What cannot be left to later is short: the
seed; the world's one shape (ruling 73); the world-founding ruleset (ruling
41); and the base unit. *The base unit* is §9.2's ruling: "the sim's length
unit is a real length; a world sets its base voxel as a real length and a
ruleset sets its tile as a power of two of it." It is fixed at founding
because every volume in the world is a power-of-two multiple of it (ruling
13), so changing it would re-cut every brick; under ruling 90 even that
becomes possible, as a realignment and never as an edit. *The nesting* this
record listed wrongly. It wrote that "a world's stack of levels is data set
at its founding", and ruling 88 corrects it: nesting is "dynamically
allocated as locations are generated". What founding supplies is only the
kinds of location, as data, each saying what it opens into; this record
first added "the default scopes of ruling 74" here, which ruling 92 caught
as the VTT's paradigm, and scopes are a game's and not the world's
(§3.7.1). The instances are allocated when a location is generated or first realised,
by what the location is, a dungeon opening into floors and a ship into
decks, and an unvisited one allocates nothing (accepted reading D15). §3.7.1
is amended to match.

**Every world trait, the same way (ruling 102).** What the short flow shows
is the seed's suggestion for each trait of the world's genotype, magic among
them, and each is configurable, marked static or fluid, and bounded by
conditions (§3.10).

**Authoring (ruling 89).** "Authored content should fill blank content or
displace generated content to the extent people are willing to do so." That
is the founding record's third pipeline law, player history displaces
procedural content and never gates it, applied to the founder. Under the
derivation rule it needs no mechanism of its own: an authored thing is an
asserted fact, and the generator derives only what nobody asserted. A
"compatible adventure pack" is a rung 1 or 2 pack (§5.3) whose content
asserts locations, factions and hooks where the world meets its
requirements. Checked 2026-09-21, the tabletop already has the shape: a
storylet carries `StoryletRequirements`, faction tags, hidden facts and
world laws that must hold, and role slots to be filled from the world
(`isometry-campaign/src/world/types.rs:151-165`), and its faction turn is
proposed and then previewed, edited and committed by whoever holds edit mode
(`faction.rs:12-13`), which is generated content offered for displacement.
"That's a world(s) editor, which i guess we have to make? Ok." **Open:**
where it lives. The tabletop is already a map editor and a campaign world
with proposal and commit, and W4's one bench "is also the dev tools";
whether the world editor is one of those, both, or a third thing is Mark's.

**Realignment (ruling 90).** A world is converted to another ruleset "by
having them go through another generative round advancing time and
realigning the world to a new ruleset". The world-conditions schema already
demands exactly this of any change of rules: "A rules revision cannot
reinterpret a past accepted event. Conversion is a new, explicit event with
both old and new revisions recorded"
(`eponym/design_docs/archive_docs/2026-09-26/2026-09-09_world_conditions_plan.md`, core invariant
4). Advancing time is what makes the seams honest: whatever the new ruleset
cannot express is accounted for by the years that passed, as ruling 38 made
the coarsening of needs diegetic. *Reading, docketed as D22, accepted 2026-09-22:* realignment is
ruling 57's epoch boundary at the scale of a world, the shop between rounds,
and it is a rung transition of §3.3; Mark's example, from hex exploration to
piloting a ship in orbit, is also a change of foregrounded scope under
ruling 74, so a realignment may add steps to the nesting as well as change
the rules.

**History (ruling 91).** How much past a world is given is the founder's
choice, "a complexity and size issue", as in Dwarf Fortress. The generated
timeline is one "one could go back in time to prior worldstates with", and
the client is not to be overwhelmed but offered "the opportunity to zoom in
and edit stuff". *Reading, docketed as D23, accepted 2026-09-22:* going back is cheap because of
the derivation rule. A world is its seed, its rules and its asserted facts,
and the sim is deterministic, so the state at any past time is that log
replayed to then, with checkpoints at epoch boundaries bounding the cost and
the oldest coarsened first under ruling 70's buffer. Going back *and
editing* is asserting a fact in the past, which makes a fork (ruling 7;
named by ruling 126, since a branch merges back and this may not), never a
rewrite, so the timeline a founder edits is the same thing as the
branching a moot does. "Not overwhelm" is the three tiers of keeping used as
an interface: the timeline shows legend first, things of note on zooming in,
and the ambient only where someone looks, lifted on demand.

**Deep time (ruling 93): "Same sim".** A world's past is the sim run forward
from a bare world for the span its founder chose (ruling 91), and no
separate history generator exists. What follows from rulings already made,
and adds nothing: deep time is the sim with nobody attending, so by ruling
71 nothing is foregrounded and by ruling 75 everything runs in the
aggregate, which is what makes a long past affordable; the wild events and
catastrophes of ruling 91 come from the world's own rules and need no
authoring; and every past state is reachable by replay (D23). Mesocosm does
this today, checked 2026-09-21: its deep time is "Mesocosm's own simulation,
one idle tick at a time, with no hand, no checkpoint and no answer to any
question", so that "a generated past is made of the same stuff as a played
one (Law C)" (`mesocosm-core/src/deep_time.rs:7-14`). Two things in that
sentence do not scale to the wing and are filed with the isoscape family
plan, which owns deep time: it ticks the near rung one idle tick at a time,
where a world of many sites over centuries needs ruling 75's aggregate form,
and it keeps no checkpoint, which going back into a timeline would want.

Prior art for rulings 89 to 91. Known: the Forgotten Realms realigned one
world to new rulesets three times by exactly Mark's method, an in-world
upheaval and a jump in the timeline, the Time of Troubles for the second
edition, the Spellplague with nearly a century skipped for the fourth, and
the Second Sundering for the fifth. Paradox's converters carry one world
from Crusader Kings to Europa Universalis and onward, a realignment between
rulesets at a date where one game's span ends and the next begins. From
memory of games, unverified: RimWorld's founding asks for a seed, coverage,
rainfall, temperature, population and factions; Dwarf Fortress lets the
founder choose the length of history among world size, civilisations, sites
and savagery, and its legends mode is a timeline to browse and not to edit.

### 3.10 The world's lineage and the magic generator

**2026-09-28 refinement:** ruling 404 makes fundamental effects composable,
gives them cantrip expressions and escalating exchange costs, and admits
world-dependent scripts authored by modders or generated by the sim. The
[functional generation plan](2026-09-09_functional_generation_plan.md#composition-design-2026-09-28)
owns the current model and open questions; the six axes below remain useful
dimensions. Ruling 407 adds the construction and learning distinctions.

From ruling 101. Every clause of Mark's first sentence is already held by a
ruling, and the magic generator is a reading, docketed as D30, accepted 2026-09-22.

**The world has a lineage.** Under rulings 98 to 100 the world is an entity
of a kingdom, so ruling 57 applies to it whole. A lineage is provenance
orthogonal in time, with two doors: reproduction rerolls expression, and the
epoch boundary changes the genotype. A world's genotype is its traits, which
is exactly "the catalog each world's critters can draw upon": the
world-founding ruleset of ruling 41, made of the glyph canon, where each
glyph is bound to an effect and a variant is a base with modifiers under a
revision (`wing-glyphs/src/canon.rs:35-66`); the world's kinds of nis, its
geology (ruling 97); and the conditions, operations, relations and
invariants of the schema (ruling 32). Its expression is the roster and the
history that unfold from it under the seed. Its epoch boundary is ruling
90's realignment, the shop between rounds where a world "could grow new
traits" (D22). To "fork into different realms" is ruling 7's forking (ruling 126) and
ruling 11's planes, worlds in relation; to "split into/create other planets"
is §3.3's lineage-splitting transition at the world rung, and under ruling
100 the child may be meso, a moon calved from a planet.

**How the world is generative.** Mark's "we could type things to provide
general logical patterns of interaction... but what else". Four sources, all
in the record. *Typing* is the schema's four kinds, and generation composes
operations from the canon's effects, the world's kinds of nis and its
conditions under its invariants; the general model plan's operator
vocabulary, strengthen, transform, instantiate, project, puppet and the
rest, is a composition algebra proposed for exactly this (§7.4, "operator
composition"). *Bodies* generate abilities, since an ability is what anatomy
affords (ruling 96) and a body requirement "resolves to a currently live,
revision-scoped part address". *Need* generates activities, since an act is
what addresses a need under scarcity (rulings 94 and 95), so a world with a
need grows the activity that meets it. *The record* is the "what else": by
ruling 49 a process is aligned to an effect by how often it yields it, the
hagiograph promotes, and the hagioglyph revises the canon, so what a world's
history does feeds back into what it recognises; a tenet (ruling 50) is a
process-to-effect pair with an opinion, so a world generates its tenets from
the pairs its history exhibits and the alignments of those who watched. The
catalogue is the generator's first half and history is its second.

**A magic system is a world trait, described on six axes.** "Glyphs are a
broad thing", and the rules already figured for magic "could very well map
to a broader set of magical systems". A generated magic system is one trait
of the world's genotype, drawn at founding or grown at a boundary, and it is
described by axes that are each a dimension the record already has:

| Axis | Values, Mark's and the record's | Where it already lives |
| --- | --- | --- |
| Source, where the power comes from | the world's own fields (a storm, a heat gradient, a seam of its nis), the caster's anatomy, a bearer (an item, a technique, a trait), a god, the record's frequency | the world-conditions plan's `accumulate`, from "a declared emitter or environmental condition... or an organism's organ"; rulings 42, 47, 49 |
| Gate, who may | unconditional; anatomically gated, a part; acquired, a glyph on a journey; skill gated, a technique (ruling 96); social, a tenet or a taboo (ruling 87); placed, a divine environment (ruling 48) | wing-glyphs' `ProvenanceKind::{Ability, Trait, Technique, Item, Bond, Quest, Event, Custom}` is this list in code (`journey.rs:12-22`) |
| Orientation | process oriented, bending how a process runs, its rate, its preconditions, its interruption; or effect oriented, yielding a glyph's effect outright | ruling 10 against ruling 49; an `OperationDef`'s preconditions against its transforms |
| Cost and breach | which account pays, matter, energy, charge, attention, time or social obligation; and which invariant, if any, the system may suspend, at what price and leaving what residue | the schema's core invariant 2 and its declared byproducts and `RiskOutcome` |
| Rhythm | cyclical, by a phase, a season or a referent period; continuous; once | §7.4's periods; ruling 52 |
| Manifestation | how it shows and is sensed | ruling 49's "characteristic manifestation"; wing-glyphs' `ExpressionTable`; the world-conditions plan's "sensing is its own operation" |

"Bends the rules" gets its definition from the cost axis: a magic system
declares which of the world's invariants it may suspend, under what gate, at
what price, leaving what residue. Ordinary magic suspends none; it is an
operation with an unusual effect, and fire from a glyph is the same burning
as fire from a torch, arrived at otherwise. Rule-bending magic suspends one,
and the breach is an event the record keeps under ruling 4's "unprecedented"
test, which is why it is rare and why divinity feeds on it (ruling 52, a
rare process mitigated by impact). Different representations follow from the
manifestation axis and the canon's expression tables: two worlds' magic can
look and play differently while the sim runs one mechanism.

**Where a world's magic comes from (ruling 102).** All three, in Mark's
order: "a suggested set from the seed is a fine default", "that could then
be configured", and thereafter "a static profile or let the world change
with time (with conditions...)", "a bit like ideology in rimworld". And not
for magic alone: "much like the world's other characteristics". So every
trait of the world's genotype (§3.10) is founded the same way, drawn from
the seed, shown in ruling 89's short flow, overridden where the founder
likes, and carrying two settings: *static* or *fluid*, that is whether
ruling 90's realignment may change it at an epoch boundary; and
*conditions*, which are founder assertions over the generator that bind the
fluid case. Mark's examples place themselves: "no magic" and "magic to
start" fix the presence and the timing of a trait; "banned traits" remove
entries from the catalogue the roster draws on; "no apocalypses" bars a
class of ruling 91's wild events, which are rung transitions and agentless
processes of §3.3. Under the derivation rule a condition is an asserted fact
about the world's genotype, and a fluid world's traits then drift within
what was asserted, as a lineage does inside its constraints. That is ruling
89's "authored content fills blank content or displaces generated content"
applied to the world's own traits, with the seed as the generated content.
Prior art, from memory and unverified: RimWorld's Ideology lets a player
author a belief system in full or take a generated one, and choose it fixed
or *fluid*, developing through play, which is the pair of settings here.

**Feeding divinity without hardcoding.** Divinity reads a magic system
through three things it already reads: effects, as glyphs in the canon;
acquisition, as the journey with its means (ruling 47); and frequency in the
record (rulings 49 and 52). A generated system that declares its effects as
glyphs, its gate as journey provenance and runs as processes feeds divinity
unchanged. What changes, as Mark allows: the fixed ladder of forms of rulings 46 and 47,
item under technique under embodied trait with memory a fourth, becomes the
gate axis of the world's magic, so the tiers of godhood a world affords are
the forms its magic lets a sophont bear and sacrifice. A world whose magic
is unconditional offers nothing to sacrifice and so no ascent by sacrifice,
only ruling 43's enactment; an anatomically gated one offers the highest
form first. The ladder is derived from the world's magic and no longer
fixed, and ruling 44's "quality of the journey" stands as the measure.

**Awareness beyond the body (ruling 384, 2026-09-28).** Some divine figures
can peek behind the curtain. An awareness of the sim beyond what an
organism's body affords suspends ruling 5's locality of knowledge and ruling
59's senses for its bearer, so it is rule-bending by this section's
definition. It is an ability some divine figures have, not a seat or a
mode, and it is parked with divinity and magic (ruling 279).

Prior art for ruling 101, known. Ars Magica's hermetic grammar composes
every effect from five techniques and ten forms, the typed-composition
answer to "what else" as a shipped rules system. GURPS Thaumatology is a
magic-system generator in print, building systems from choices about source,
cost, gate and rhythm. Sanderson's laws hold that a magic's limits and
costs, not its powers, make it interesting, which is the cost axis as
doctrine. From memory of games, unverified: Dwarf Fortress generates each
world's secrets and spheres, and later versions its magic, per world; Noita
composes spells as programs on a wand, process-oriented magic as play; Mage:
the Ascension's spheres are an effect-oriented canon.

### 3.11 Arcs and the storyteller

From ruling 103: "both, with the second being what gives the first its
shape". *Reading, docketed as D31, accepted 2026-09-22.*

**The sim's half: the parts.** Every constituent part of an arc is a thing
the sim already has, so the sim is "aware of the constituent parts" without
any arc type of its own:

| Part of an arc | In the sim's terms | Ruling |
| --- | --- | --- |
| A goal | a need, a desire chosen under beliefs and values, or an obligation, by tier; a polity's from its alignment | §3.3.1, 68 |
| Consequential | achieving it would be an event of note or of legend, by the hagiograph's test | 4, 69 |
| Real reasons | the score of the act, need times trust times approval, and nothing else | D26 |
| Pursuit | choices under scarcity, each with its foregone and its cause-link | Law A, 10 |
| Obstacles | scarcity, other agents' goals, enforcement, taboo | 94, 63, 64, 87 |
| Stakes | what is foregone, and standing at risk | Law A, 56 |
| Turning points | events of note and rung transitions | 69, §3.3 |
| Ending | the goal met, abandoned, or its holder dead or dissolved | 61, 65 |

An arc is then a thread in the record: the cause-links from a goal's
adoption to its ending, and a *consequential* goal is one whose adoption
leaves a note (ruling 80; wing-glyphs already carries `Quest` among its
journey provenances, `journey.rs:18`), so it survives collection while it is
pursued. The sim stores intentions as agent state (ruling 9) and notes on
the consequential ones; it never stores an arc, which is derived, as
significance is.

**The game's half: the statement.** "The game is what takes that vocab and
makes a coherent game statement out of it." An overlay reads threads out of
the record, chooses which to foreground, and names them, which is what §3.8
already reserves to it: the sim "never decides what is fun". Each product
has its reading. The tabletop's storylets carry requirements over factions,
hidden facts and world laws, with role slots filled from the world (§3.9),
and ruling 3 binds the VTT to arcs and narratives. Eponym owes its
"legibility surface" as day-one work (its CLAUDE.md). Mesocosm's statement
is the epoch review. The vocabulary is the same for all three; the statement
differs.

**The storyteller is the world entity's agency.** "Might even think of the
world as a bit like rimworld's storyteller." Under rulings 98 and 99 the
world is an entity whose kind is open, and an inert world's processes fall
from rates while a sentient, sophont or divine world *chooses* its agentless
processes (§3.3). So a storyteller is a world given a disposition (ruling
37's five factors) and goals (ruling 68), directing its own weather, plagues
and wild events toward what it wants, and a founder sets it in ruling 102's
flow, static or fluid, with conditions such as "no apocalypses" bounding it.
RimWorld's storytellers are then three dispositions of a world; the wing's
are drawn, configured, and may be gods.

*Ruled 2026-09-24 (rulings 120 to 122).* The storyteller's disposition
seeds its first lineages directly and shapes its people after only through
condition (rulings 120 and 121). When fluid, the disposition itself is moved
by what happens to the world, by its own cycles and at its epoch boundary,
and not by its people's temperament (ruling 122): an age of war can
embitter a world and an age of plenty mellow it, and its people move it only
by what they do to it.

Prior art for ruling 103, known. RimWorld's storyteller and Left 4 Dead's AI
Director are the pacing case, an agent scheduling incidents against the
state of play, which here becomes the world's own agency. Dwarf Fortress's
legends mode is the reading case, arcs found after the fact in an event log.
The story-sifting literature, Ryan's work on curating simulated storyworlds
and Kreminski's Felt, makes the game's half a query: patterns over a
simulation's event log that pick out threads worth telling, which is exactly
an overlay reading arcs from the record. Bremond's elementary sequence, a
possibility, its actualisation and its outcome, is the arc in three parts,
the same three the table above opens and closes with. Failbetter's storylets
are the tabletop's own word.

### 3.12 Time in a shared world

From ruling 104, a question put back to Mark with this record's
recommendation, docketed as D32.

**What the record already settles.** Worlds are forkable and branchable,
like a moot (ruling 7), and a world is its seed, its rules and its asserted
facts, with everything else derived (§1). So two players who advance a world
separately hold two branches, and a merge is a merge of two logs of intents,
never of two world states. That is why Mark's "if the conflicts aren't
foregrounded material, then it should be fine" is nearly right and can be
made exact: the ambient tier is regenerated from the merged facts and can
never conflict (rulings 69 and 70); only asserted facts and notes can, and a
conflict is exactly an intent that, replayed into the merged history, an
invariant refuses, the world-conditions schema's `Refused` (§3.3). Merging
is then the tabletop's proposal and commit at the scale of a branch, and it
respects the wing's standing rule against speculative CRDTs: intents are
additive, the sim is the materialiser, and refusal is the conflict rule
(Mesocosm's CLAUDE.md). Synchronous play stays what the VTT already has,
one authority and an ordered event log.

**What no merge can do.** Two branches that advanced different spans do not
merge. A branch that ran a century and one that ran a day cannot be unioned
in time order without the day player's intents landing a century in the
past, and the day player's sophont is dead in the other branch by ruling 61.
Bringing them together is ruling 90's realignment, a generative round
advancing the short branch, and that is a founding-scale act with consent,
not a merge. So the honest rule is not a simplification of the git rule; it
is what the git rule needs underneath it, since a shared trunk gives every
merge a common span.

| Option | What it is | What it costs |
| --- | --- | --- |
| A. Trunk and branches | a shared world's clock proceeds only when played, by whoever is playing; anyone may play a branch offline and later propose it back; a merge is a replay of the union of intents in time order, and a refusal on replay is a conflict for whoever holds edit mode | replay must be deterministic across peers; refusals need an interface |
| B. Merge only | every player advances a private branch and worlds meet only by merging | branches that advanced different spans cannot merge, only realign, so players at different timescales never meet |
| C. Trunk only | one clock, advanced by whoever plays, no offline branches | no asynchronous play; a century player and a day player cannot share a world at all |

*Recommendation:* **A.** It keeps Mark's "honest, predictable, and easy" as
the trunk's rule and his asynchronous play as branches, and each half is
machinery already ruled or built. "Whoever opens it has started the clock"
is ruling 71's examination counting as play; "people can review what
happened since" is D23's timeline read through the three tiers of keeping,
legend first. Advancing the trunk past what another player's foreground can
bear, a century over a sophont's life, is a proposal needing that player's
consent (ruling 63, the players a faction) or a branch. Whether a shared
world may also run unattended at a rate, a moot's server as the storyteller
(§3.11), is one more founder setting in ruling 102's flow, off by default.
Two things follow. Collection must be deterministic across peers, which
closes §3.4.1's open item: a merge that replays must reproduce, so what is
funged and when is a function of the seed and the log and never of the
machine. And branches are cheap because the ambient is free: a branch stores
only its intents and the notes they made.

**Related worlds (ruling 105).** Option A is ruled, and Mark asks for a
shared context over worlds that are not merged: "two worldlines with a
shared history that branched", or "entirely different worlds" made
neighbours "in the celestial neighborhood", so that "you could manage a set
of related worlds". *Reading, docketed as D33, accepted 2026-09-22.* Two relations, both already
in the record, and neither merges anything: each world keeps its own log and
its own played clock, and what crosses between them is things with
provenance.

- *Descent.* Forked worldlines (ruling 126's fork) are the world kingdom's
  lineage tree (ruling 101): siblings share a common prefix of the log up
  to the fork, so the legend from before it is one legend cited by both, and the
  histories after it diverge. Mesocosm's terminology already names this:
  *fili*, "lineage across worlds (forks, campaign descent, cross-moot
  grafts)", which is the record of descent between worlds and nothing else.
- *Neighbourhood.* Unrelated worlds are placed in relation by assertion at
  the rung above the world, ruling 11's planetary system and the planes as
  "worlds in relation", with ruling 74's nesting composing upward, "a system
  above the world for the space scope". A neighbourhood is then a set of
  worlds with routes between them, a ship's passage or a portal to a plane,
  and a crossing is a rung transition recorded in both logs: a departure
  with provenance in one and an arrival in the other, which is what ruling
  7's creative mode does by hand and what ruling 62's summoning does for the
  dead. What arrives carries its home world's nis, so under ruling 97 a
  thing from a neighbouring world is made of a lineage this world never had,
  which is where exotic materials come from.
- *Managing the set.* Worlds under no host are, by ruling 67, a faction
  among themselves: the relation holds by the consent of their owners and
  each keeps its own clock. That is the moot's shape, and its federation
  tiers are already the stack's way to hold a set of related worlds; ruling
  100's open item, whether a system is an entity of the world kingdom at a
  scale above macro, is then the question of whether a neighbourhood may be
  given a host, a sun with a provenance and a founding ruleset of its own,
  and stays open. **Open:** whether worlds in one neighbourhood share one
  played clock, which a traveller's continuous time seems to require, or
  convert between clocks at each crossing. *Ruled 2026-09-24 (rulings 149
  and 150):* each world keeps its own clock and a crossing brings the two
  into step, as realignment does; and a system can be a host, a sun with
  a provenance and founding ruleset of its own, hosting its worlds as a
  polity hosts others.

Prior art for ruling 105, known: git's forks and remotes, Mark's own figure,
are descent and neighbourhood exactly, shared history without merging and
unrelated repositories placed in relation; Planescape's planes joined by
portals with Sigil as the hub, and Spelljammer's crystal spheres crossed by
ship, are the two kinds of neighbourhood in one tabletop tradition; Outer
Wilds is a neighbourhood on one clock, a whole system as a single playable
place; Kerbal Space Program is one a ship crosses. From memory, unverified:
Magic: The Gathering's planeswalking is a multiverse of worlds in relation
with no shared history.

Prior art for ruling 104. Known: git is Mark's own reference, and the stack
already carries weave, an entity-level merge driver for code, which is the
same idea, merge by the thing and not by the line; play-by-mail and
Civilization's play-by-cloud are the honest rule, time proceeding when
played; Animal Crossing is the other rule, the world moving while the player
is away; mere's deterministic replay brief (2026-07-31) records that
state-as-fold and capture-and-replay have been the stack's doctrine since
2026-07-02, which is what makes merge-as-replay possible at all.

## 4. The stack

### 4.1 Placement

Read from each crate's own description on 2026-09-18, not from memory.

| Tier | Component | What it is | Where |
| --- | --- | --- | --- |
| Rendering | isometer | The scene: terrain tracer, body renderer, depth join, picking, glyph batch | `shared/isometer`, the repository root |
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
| Trust plane | eponym-identity | Ruled "the wing's identity crate" on 2026-08-10 and consumed by nothing outside Eponym since; `SubjectId`, body revisions, facets and the control pointer are the sim's provenance noun. Dramatis absorbs it under W3 (ruling 34; raised 2026-09-18 by W1) | `eponym/crates/eponym-identity` |
| Persistence | eidetic | The durable-memory family: muniment for slots, blobs and journals; chartulary for the content-addressed container graph with lineage; hagiograph for the history organ | mere `eidetic` |
| Branching and federation | moot | gemot for a moot's lifecycle and replication over p2panda; moothold for federation. The branchable-world model is this | mere `moot` |
| Networking | murm | Invitation-scoped peer conversation with signed per-author logs and a WebRTC carrier; iroh is the other carrier | mere `murm` |
| Distribution | luggage | Signed self-update over pluggable feeds | mere `system/luggage` |
| Physics and volumes | conatus | Bodies, collision, queries, fixed step; modulus for the brick atlas and traversal; nisus for revisioned voxel chunks and edits; numen for fields; seiche for force layout | mere `conatus` |

### 4.2 What replaced renderling

Renderling is retired by ruling, not yet in the tree: the presentation
plan's L7 (2026-09-11) has it exit Eponym and "nothing new is built on
renderling", but `eponym-client` still takes it unconditionally by a
machine-local path (`eponym-client/Cargo.toml:66`), the Eponym
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
behind the scene contract and is swappable. The tenant is kiss3d,
**checked 2026-09-18 from its published 0.46.0 source:** its context is
initialised from an existing instance, device, queue, adapter and surface
format (`src/context/context.rs:48`), and it renders to offscreen colour
and depth buffers a caller can hand on
(`src/resource/framebuffer_manager.rs:10-88`), which is the tenant shape
netrender composition needs. Its context is a thread-local singleton, one
per thread, which fits the kernel owning the GPU on one thread. It also
carries a hardware ray tracer behind wgpu's experimental ray-query
feature (`src/renderer/raytracer/`, `src/window/wgpu_canvas.rs:49-63`),
which bears on rulings 22's shadows and global illumination. Earlier
draft text follows for the record of what was checked and when: kiss3d, whose 0.46 of 2026-08-15 sits on wgpu
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
  *(Corrected 2026-09-26 by the T2 assessment of ruling 326:
  `voxel_profile` mirrors terrain `Ground` into nisus chunks, its only use
  in the repository (`mesocosm-core/src/voxel_profile.rs`); no body volume
  touches nisus. Rulings 330 to 335 settle the duplication: nisus grows
  into the authority.)*
- **A revision that never moves for regrowing hosts.** isometer's
  `GroundTerrain` stamps `Ground::revision()`, which only `carve` advances
  (`isometer-core/src/ground.rs:374`) and `grow` resets to zero
  (`:210-211`), so the tracer silently skips every upload after the first
  for any host that regrows rather than carves. Found by Isometry's B4
  lane with a positive control. Eponym is the one live consumer where
  the mechanism works, because its world carves (`eponym-world/src/world.rs:195`)
  and its producer keys the rebuild on the revision
  (`producer/source.rs:275`); the fix is scoped to regrowing hosts
  (narrowed 2026-09-18 by W1).
- **No brick-level skipping.** modulus's traversal steps voxel by voxel
  through empty bricks with a loop cap of 1,024, so a tall world walks
  hundreds of air voxels per pixel and can fail to reach the ground. The
  two-level walk OpenVDB calls a hierarchical DDA is the standard fix.
- **A budget constant read as a limit.** `modulus::MAX_BRICKS` is 2,047,
  from three atlas layout constants sized to Eponym's one-megabyte
  residency experiment. Paging already exists (`with_capacity`,
  `retarget`) and isometer-lens wraps both (`bricks.rs:95,105`, receipted
  at `tracer_tests.rs:665`). **No production scene in the wing reaches
  it.** Eponym's paging has exactly two callers, the `v1` and `v1b`
  receipt bins behind non-default features
  (`eponym-client/src/bin/v1_residency.rs:53`, `v1b_residency.rs:48`);
  its shipped session host rebuilds the whole ground on every revision
  through `BrickMap::from_ground` (`producer/source.rs:275-280`). The
  scene board reaches the cap routinely and builds through
  `from_ground_keys` and `from_ground_filtered`
  (`crates/isometry-views/src/scene/ground.rs:244,247`). So sizing the
  store to the card is a lane in every consumer as well as isometer's.
  (Corrected twice on 2026-09-18 by W1; the first draft said no scene
  reached the cap and that Eponym ran the paging.)
- **Two scripting engines.** piccolo Lua in isometry-system and
  mesocosm-phenotype; Rhai in numen. Which is the wing's authoring language
  is a §9 question.
- **Two parley font loaders** in Eponym, recorded by M5.
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
verbs it reaches as handles (ruling 10).

The domain is a rung of the agent ladder, refined by play, with every other
rung simmed in the background and weakly expressed in the foreground
(ruling 35). The founding record's current continuity of critter, denizen and
character (§1, "The continuity: critter, borg, character," superseded for
terminology on 2026-09-20) is the same
creature at three levels of identity, and the three games are those three
levels:

| Game | Foregrounded rung | Refined by play | Weakly expressed |
| --- | --- | --- | --- |
| Mesocosm | the critter and its lineage | the critter, over the ages, according to play preference | the ecology as weather, prey and competitors; society and polities as distant pressures that can still influence events |
| Eponym | the denizen, a named inhabitant (ruling 200), and its factions | what remembered entities do; the coterie, the party, the base | the ecology as wildlife and land; polities as the powers that shape the region |
| The VTT | the character inside its polities | characters partisan, friendly, antagonistic, factional or unaligned | polities set the narrative stakes (sidequests, alignment, arcs, non-player characters, access to resources); the ecology as terrain and encounter |

"Weakly expressed" is a requirement on the sim, not on the game: every rung
must be able to run at background fidelity and surface as effects,
encounters, pressures and stakes in a game that does not play it. What that
requires of each rung is a W2 question.

**One host, modes (ruling 381, 2026-09-28).** The three games are modes of
one host over one world save, each product a build profile of it, and
players in different modes may share one trunk under ruling 104. **A default
view (ruling 382).** The wing has one default view, isometric with quarter
turns, which precludes no other: free-yaw isometric, first person and third
person over the shoulder stay available, so perspective remains a mode's
choice as ruling 1 has it, now with a shared default. Its pitch is the 2:1
dimetric, 30° (ruling 387), and whether Mesocosm opens in it is ruled on
CP1's comparison (ruling 388).

### 5.1 Rulesets over the sim

Mark's question of 2026-09-18: "how do you make one sim be useful to both
a srd, pf2e, daggerheart, PbtA, GURPS, etc. ruleset? maybe we taxonomize
each, consider them each a language we have to make bindings to the sim
for?" And his addendum: "perhaps it's worthwhile to think of some world
effects as like rulesets for mesocosm and/or paredros?" This section is
the assessment; the rulings it needs are in §9.12.

**The seam that exists.** The VTT's system plugin (`crates/isometry-system`,
read 2026-09-18) is already a binding of the shape Mark describes: a system
is a schema of fields, derived values as Lua functions of the sheet, and
actions as a dice expression plus a Lua bonus; an adjudicated action names
a target and asks the script how well it landed as a **degree of success**,
-1 to 2, scales its effect by degree, and resolves into typed deltas,
conditions, forced movement, beats and defeats that the substrate obeys
without knowing what a hit point is. A Pathfinder 2e skeleton is the second
ruleset and "the reason the action spec has to generalize": degrees of
success came from it. So the two-consumer test has been run once, inside
the d20 family, and the seam held.

**What varies across the rulesets Mark named,** from general knowledge of
each system (unverified against their texts today):

| Concern | 5e SRD | Pathfinder 2e | Daggerheart | Powered by the Apocalypse | GURPS |
| --- | --- | --- | --- | --- | --- |
| Resolution | d20 plus modifier against a number | the same with four degrees | two d12, Hope and Fear, against a number, the higher die a twist | 2d6 plus stat in three bands, 6 or less, 7 to 9, 10 or more | 3d6 roll under a skill, margin of success |
| Turn structure | action, bonus, reaction, movement per turn, initiative | three actions per turn, initiative | no initiative; a spotlight passes, Fear buys the GM's turn | conversational, no turns; the GM moves when the fiction demands | one-second turns with manoeuvres |
| State reading | hit points | hit points | hit points, stress, hope | harm as clocks | hit points, fatigue, injury |
| Sheet | class, species, feats, spell slots | ancestry, class, feats, three-action abilities | class, domain cards, experiences | a playbook of moves | points bought into skills and advantages |
| Progression | levels | levels | levels | marked experience | points |
| The GM's side | rulings | rulings | GM moves bought with Fear | GM moves on a miss or a lull, principles | rulings |

**What is common,** which is the binding language's vocabulary:

- A **check** is a situation plus a stat turned into an **outcome band with
  a margin and a twist**: fail, partial, success, critical, plus how far
  and whether a complication rides along. PbtA's three bands, Pathfinder's
  four degrees, Daggerheart's success-with-Fear and GURPS's margin are all
  readings of that one shape, and the existing degree ladder is most of it.
  The sim records the band, never the dice.
- A **reading** turns the one ledger into what the system shows and rules
  on: hit points, stress, harm clocks, exhaustion (ruling 38).
- A **sheet** is a bundle of **grants**: abilities, modifiers, resources
  and recognitions (what counts as food, a weapon, a spell), which is the
  shape wing-glyphs already uses for glyph grants.
- **Turn structure** is the overlay's timescale (ruling 6), not the sim's;
  a ruleset chooses initiative, three actions, a spotlight or no turns.
- **GM moves** are rules that prompt the world on a miss or a lull; under
  the sim they are processes the ruleset registers, which is the same
  shape as the hagiograph retelling and the third process shape founding
  something.

**A ruleset is therefore a language with bindings,** as Mark said: a pack
of schema plus piccolo Lua over a bindings interface whose vocabulary is
the list above. The sim never sees the ruleset; it exposes situations, the
ledger and the record, and accepts outcomes as asserted facts and effects
on the ledger.

**World effects are rulesets at world scope.** Mark's addendum, and the
wing already does it in two places: Mesocosm's process definitions ship as
a data-only pack, are lowered to a ruleset, and a world records that
ruleset by digest (processdef plan, PD3); Eponym's world-conditions
schema carries a content-addressed rules revision (ruling 32). Dwarf
Fortress's raws are the shipped precedent: the definitions of creatures,
materials and reactions are data, mods change them, and a world locks its
raws at generation. So there are two scopes of one language: world rules,
bound when a world is founded and inherited by every game played in it
(magic and glyphs, suggested anatomy, constraints, what the moon does),
and game rulesets, bound when a game is played over it. Both are packs,
both lower to rulesets, both are recorded by digest. This answers W2's
first question: what may a ruleset add or forbid is exactly what a
world-scope pack may, and neither may override the ledger, the record or
the derivation rule.

**Prior art for one engine over many rulesets.** Fantasy Grounds is the
closest: a CoreRPG base ruleset with 5e, Pathfinder, GURPS and dozens of
others layered on it, written in Lua. Foundry VTT is a system-agnostic core
of documents and a data model with hundreds of community systems as data
plus automation. Roll20 is sheets as templates with scripted workers.
Owlcat's engine has shipped both Pathfinder 1e and the Warhammer 40,000
roleplaying rules from one data-driven core. Among video games otherwise,
an engine binds one system, Infinity Engine to AD&D, Neverwinter Nights to
3e, Solasta and Baldur's Gate 3 to 5e, Temple of Elemental Evil to 3.5.
Outside games, Ludii runs hundreds of board games from one description
language of shared atoms, the Stanford Game Description Language does the
same for general game playing, and rules-as-code projects, Catala and
OpenFisca, bind many jurisdictions' law to one engine, which is the same
problem with higher stakes. All from general knowledge, unverified.

**Licensing, checked 2026-09-18 against the licence texts,** under
ruling 40's rule that a ruleset is in if nobody will sue or hate on the
wing for it.

- **5e SRD, CC BY 4.0; Pathfinder 2e, ORC.** In, as the tabletop's
  CLAUDE.md already permits.
- **Daggerheart: out under its licence.** The Darrington Press Community
  Gaming License 2.0 (26 August 2026, read from the PDF) grants Public
  Game Content only in "Permitted Formats", which it defines as print and
  digital print, streaming, podcasts, and virtual tabletops it has
  whitelisted for non-commercial use only, and states that the term
  "excludes, without limitation, film, television, video games, and any
  other audiovisual medium not expressly permitted"; "video games" also
  appear in its list of Prohibited Content. A game of the wing is a video
  game, so Daggerheart's mechanics cannot ship in it without a separate
  licence from Darrington Press. The SRD can still be read as prior art
  for the binding language, which the taxonomy above does.
- **Lancer: in, with attribution.** Massif Press's Lancer Third Party
  License, read from massifpress.com: "You may use the mechanics of Core
  Lancer, Lancer: Battlegroup, or any other Lancer product as the base for
  your system, setting, or game", for sale or free, with no art or text,
  no logos, no claim of affiliation, the stated acknowledgement and the
  copyright line in the product, and no content directing hate at
  protected groups. COMP/CON, Massif's own character tool, is GPL-3 and
  reads Lancer's data from an open repository, so software implementation
  is the established practice. Lancer brings to the binding language what
  the d20 family lacks: accuracy and difficulty as d6 pools modifying a
  d20, pilot triggers resolved in bands at 10 and 17, popcorn initiative
  alternating sides, heat, structure and stress as three readings of one
  body, sizes occupying tiles, and a mech as a body the pilot occupies,
  which is a body inside a body and the sim's holding relation at its
  strongest.
- **ICON and CAIN: unlicensed for implementation; ask.** ICON's page
  grants only "download or distribute it as long as you don't modify the
  credits page", for a free playtest of a future project; CAIN's page
  states no licence at all. Mechanics as such are not copyrightable, but
  ruling 40's second clause, being hated on, is exactly the risk of
  implementing a designer's playtest without asking. Tom Bloom's
  community is one Mark listens in on, so the ask is cheap and the
  answer may be a licence like Lancer's. ICON brings a tactical and a
  narrative layer usable separately; CAIN brings d6 dice pools counting
  successes against a pressure track, a third resolution family.
- **Powered by the Apocalypse** is a design framework; its moves and
  bands are mechanics, not text, and the wing writes its own. **GURPS** is
  not open. "POTA" in Mark's list is read as PbtA.

**Where to start.** Not by binding five systems. The vocabulary above is
the binding language; the existing seam covers the d20 family; the next
consumer should be the one most unlike it, a PbtA-shaped system with
bands, moves, no turns and GM moves, because it stresses turn structure
and the GM's side, which the degree ladder never touched. Daggerheart
adds the twist die and the Fear economy and is the other candidate,
licence permitting. Two consumers of different families is the wing's own
rule, and it has been met for one family only. What each game's profile contains
is that game's design and not this record's. Two questions belong to the
overlay tier: who resolves an event when a DM and the sim both could, and
where a foreground game's rules stop and the sim's begin. Both are inside a
branch, so they do not touch the base profile, and rulings 114, 154, 156,
188 and 189 answer them.

*2026-10-03, ruling 522:* Ars Magica 5th Edition, whose text Atlas Games
released under CC BY-SA 4.0, becomes a VTT ruleset, and its Hermetic magic
and realm auras a reference for the wing's magic. It is a family unlike
the d20 pair: stress and simple dice with botches, magic composed of
Techniques and Forms, and seasonal advancement for a covenant. Prior art
Mark supplied for adapting it, each still to be read and its licence
checked before any code of it is studied:

| Work | What it offers |
| --- | --- |
| *Ars Magica: Years of Conquest*, Black Chicken Studios, 2012 ([announcement](https://forum.atlas-games.com/t/ars-magica-video-game/7312), [campaign](https://www.kicktraq.com/projects/blackchickenstudios/ars-magica-video-game/)) | A licensed generational simulation RPG proposal whose Kickstarter failed: design precedent for communities, accumulated knowledge and history, not evidence the approach worked |
| Ars Magica for Foundry VTT ([project](https://github.com/Xzotl42/arm5e), [listing](https://foundryvtt.com/packages/arm5e)) | Open fifth-edition tabletop play with a content compendium: how the rules become digital records and player tools |
| Metacreator ([fifth edition](https://atlas-games.com/news/post?s=great-news-metacreator-for-arm5), [features](https://atlas-games.com/forumARCHIVE/threads/000128.html), [retirement](https://forum.atlas-games.com/t/metacreator-distribution/171200)) | Character and covenant management with study, training, spell research, enchanting, aging and kept history: the seasonal side; sales ended in 2022 |
| [Ars Magica Character Generator](https://github.com/garin1000/Ars-Magica-Character-Generator) | A pure Rust rules library behind a Tauri and Svelte interface, mechanics in JSON with stable identifiers, choices saved with the ruleset's identity and version; characters implemented, covenants not; maturity unassessed |
| [Fate of Ars Magica](https://github.com/ArtturiLaitakari/FateofArsMagica) | A small fan conversion to Fate Core and Accelerated: which distinctions survive a change of resolution system |
| [Ars Magica 2](https://www.curseforge.com/minecraft/mc-mods/ars-magica-2), [Mana and Artifice](https://www.curseforge.com/minecraft/mc-mods/mana-and-artifice), [Ars Nouveau](https://www.curseforge.com/minecraft/mc-mods/ars-nouveau) | Minecraft spellcrafting from shapes, components, modifiers and rituals, Mana and Artifice also with modular constructs: the practical limits and interface of composed spells, not faithful to the tabletop rules |

Mark's suggested reading: Foundry's module and the Rust generator for how
rules are encoded, Metacreator for seasonal history, and Mana and Artifice
and Ars Nouveau for composed spells.

*2026-10-03, rulings 538 to 541:* Mark's gold standard is faithful
editions, D&D, Pathfinder and Ars Magica in their versions, as close to RAW
as can be, with rulesets mixable over a coherent abstraction. Each world
chooses whether the sim or its ruleset leads (538); rules are built as
faithful versioned editions, keeping the GM's decision points, with a
toolkit extracted from what repeats and a house ruleset only if needed
(539); content and subsystems mix freely, and an encounter across systems
runs under a named hybrid profile (540); and the VTT plays editions while
Eponym and Mesocosm keep their native mechanics, sharing the toolkit (541).
Further prior art Mark supplied, each still to be read and its licence
checked before any code of it is studied:

| Work | What it offers |
| --- | --- |
| [Datasworn](https://github.com/rsek/datasworn), with [Iron Vault](https://ironvault.quest/other-features/rulesets-and-homebrew.html) | Ironsworn and Starforged as JSON under a language-independent schema with generated types, Rust among them; content packages a separate tool imports and compiles homebrew into: author once, use across tools. Schema MIT, content licensed apart ([publisher's terms](https://tomkinpress.com/pages/licensing)) |
| [Foundry's Pathfinder 2e rule elements](https://github.com/foundryvtt/pf2e/wiki/Quickstart-guide-for-rule-elements) | Abilities as small operations, each a selector, predicates, values from character or item data, and an inspector showing what a roll read: inspectable conditional effects |
| [Inform](https://github.com/ganelson/inform) ([actions](https://ganelson.github.io/inform/WorldModelKit/actns.html)) | A world model processing actions through rulebooks of reach, visibility, carrying and actor: legible rule authoring and its exceptions |
| [Icepool](https://github.com/HighDiceRoller/icepool) | Exact probabilities of dice mechanics, pools, keeps, rerolls and opposed rolls, for comparing a mapping's distributions on a bench; a reroll needs its decision policy |
| [Versu](https://versu.com/about/how-versu-works/) ([content](https://emshort.blog/2013/02/22/versu-content-structure/), [conversation](https://emshort.blog/2013/02/26/versu-conversation-implementation/)) | Social practices, roles and reactions, characters able to break a norm and others answering it: oaths, rank, hospitality and covenant politics |
| [Ceptre](https://www.cs.cmu.edu/~cmartens/ceptre.pdf) | Resource-transforming rules in stages, a player's choice replaceable by a stated strategy: which parts of a procedure become simulation once policies are supplied |
| [BRP](https://brp.chaosium.com/brp-downloads/) | A generic ruleset with an ORC reference document, for adaptation experiments |
| [GURPS Lite](https://www.sjgames.com/gurps/lite/3e/gurpslite.pdf) | Physical detail at configurable depth, as a design reference |
| [Year Zero Engine](https://freeleaguepublishing.com/wp-content/uploads/2023/11/YZE-Standard-Reference-Document.pdf) | Risk, pushed rolls, stress and resource pressure; its [licence](https://freeleaguepublishing.com/wp-content/uploads/2026/03/Year-Zero-Engine-License-Agreement-version-1.1.pdf) admits VTT modules and excludes video games |
| [Ironsworn](https://tomkinpress.com/collections/ironsworn) | Solo and cooperative play, with Datasworn's digital path |

*Reading, not ruled:* portable content, executable effects and decision
policies are separate investigations, and the test of a mapping is that
two rules keep the same consequences, costs, permissions and choices, not
only an outcome's label.

### 5.2 The boundary between the sim and a game (ruled in part)

**Review, 2026-09-21, at Mark's word: "This ruling feels particularly
contentious, so let's review what it would cost and if we've overextended
ourselves a bit in the brainstorming."** §5.2 to §5.4 are held. Only ruling
78, a clean boundary, is ruled in them. *(Since then, rulings 114 and 154
ruled §5.2's points 2 to 5 on 2026-09-24, and points 1, 6 and 7 stand as D18
left them; see point 5's note.)* Sized against what exists:

| Proposed commitment | What exists today | What it would cost |
| --- | --- | --- |
| The overlay contract as its own crate of value types | no sim crate and no overlay trait; the wing's only `Product` trait is isomere's GUI host (`shared/isomere/src/host.rs:135`) | little, if the contract is shaped that way when it is first written, which is W5's work and not W2's |
| A component binding kept green in CI | no CI in this repository; Wasmtime in none of the wing's three lockfiles; mere's binding for two narrow worlds is 3,371 lines of Rust and 193 of WIT | thousands of lines, a heavy dependency mere's own plan calls "an unmade dependency decision", a CI system, and rework on every contract change during the phases when the contract changes most |
| The sim reached through mere's gate | a gate for chartulary graph edits | withdrawn above |
| A W2 probe of the component constant | nothing | a lane with its own toolchain, useful only if componentisation is live |
| One software `libm` everywhere | unknown | nothing until peers verify each other's replays |

*Finding.* Yes, overextended, in three ways. A question about the layer
under the sim was answered and then turned into standing commitments, a
second binding and a CI duty, for a contract that does not exist over a sim
that has no schema; W2's done-condition is that schema, its process
definitions, the record and reach, and a generator, and the overlay is W5.
One reading was asserted from a summary and did not survive reading the
code. And the record has been growing faster than it can be ruled: 1,275
lines since ruling 62, with fifteen flagged readings, three recommendations
and twelve open markers waiting on Mark, some of them carrying later ones.

*What survives cheaply, accepted by Mark 2026-09-21 (D18).* Ruling 78 alone, as a design
discipline. Two bindings as an intent and not a duty: native first, the
component binding built when the first rung 3 consumer exists, the contract
shaped so that day needs no redesign. For erosion, a test and not a runtime:
every contract type round-trips through bytes, which replay and the network
need anyway and which proves no pointer crosses, and the contract crate may
not depend on the sim's internals. The probe and `libm` are notes for the
day they matter. All of it belongs to W5 and W3, not to W2.

Mark asked, in ruling 75, what good interoperation between the sim and the
game layer should be for efficiency, and whether it should be organised as
WASI, wasm and WIT processes are, with workers and a thread pool. This is
this record's recommendation and is Mark's to rule. Ruling 76 then said the
question was about the layer underneath the sim, which §5.3 answers; what
follows here is the boundary above it and stands as written.

1. **Grain before mechanism.** The cost of a boundary is how often it is
   crossed and how much is copied each time, so the contract is coarse:
   batches per tick, never a call per entity.

2. **In: intents.** A game never writes sim state. It submits intents
   stamped for a tick: directives, and where the overlay opens it, the
   actuation of one body (§9.14). Intents are the sim's only
   nondeterministic input, so they are also the replay log and the network
   protocol, which both products already practise: the tabletop's
   DM-authority ordered event log and Eponym's fixed input trace with
   save, reload and replay.

3. **Out: events and views.** Events are the sim's receipts and record
   entries, streamed by subscription. Views are a read-only snapshot of tick
   N, read by the renderer and the interface while the sim computes N+1, so
   neither waits on the other and nothing is locked; isometer's Scene is
   already that seam for rendering (§4.7).

4. **One attention set per player.** The roots of ruling 71, what a player
   cares about, are also what is simulated at full detail and what is
   subscribed to. One set drives the collector, the foreground and the event
   stream.

5. **The resolution handoff.** §3.8 already says the foreground game
   resolves a blow by its own rules or the background resolves it by
   outcome. So when an operation touches something foregrounded the sim asks
   the overlay to resolve it and takes back an outcome in the sim's own
   terms, which must pass the sim's invariants, the accounts conserved;
   otherwise the sim resolves it by rate. That the two agree is ruling 75,
   owed by the ruleset and checked on the bench. (Ruled 2026-09-24 for this
   point by ruling 114, and with points 2 to 4 as the contract's shape by
   ruling 154. Point 2's directives are narrowed by rulings 152 and 153: a
   player directs only the entity they play, and two players may direct
   the same one; a DM may take up any unclaimed entity to play and may
   edit the world (ruling 156). Points 1, 6 and 7 stay as D18 left them.)

6. **WIT as the discipline, with two bindings.** Shape the contract the way
   WIT forces: values copied, opaque handles minted by the host, no pointer
   into sim memory, capabilities by grant. Mere's script substrate already
   practises it: "Values are copied across the boundary; no language-native
   object graph crosses it", and an ungranted capability is unlinked and
   unreachable (`mere/crates/script/wit/world.wit`, checked 2026-09-21).
   Bind that one contract twice: natively, as the trait of §9.1, for the
   three first-party cores in one address space; and as a component world on
   Wasmtime for mods and third-party overlays. The same discipline is what
   lets the boundary cross a Worker on the web, a peer on the network and a
   replay file unchanged.

7. **Not WIT between the sim and its own cores, and not the component as the
   unit of the sim's parallelism.** Every call across a component boundary
   copies its values, which suits a batch of events per tick and would ruin
   a query per entity. And the component model's concurrency, checked
   2026-09-21, is native async: WASI 0.3.0 shipped on 2026-06-11 with `async
   func`, `stream` and `future`, first implemented as final in Wasmtime 46,
   with threads and zero-copy named as later work. So parallelism stays as
   §4.7 has it, shards of the place graph as armillary actors, Rayon inside
   a shard, an ordered merge between them. Components fit that as many
   single-threaded instances, one per shard, fed their shard's inputs in
   order. The answer to the question as asked is yes to the shape, shared
   nothing between shards and across the boundary with shared memory only
   inside a shard, and no to wasm as the literal mechanism between the sim
   and the first-party games. Wasm's determinism, no clock and no thread
   unless granted, is an asset for the mods that do run in it.

### 5.3 How the stack resolves: foreground, background, infrastructure

From ruling 76. Read 2026-09-21 in mere: armillary's README and founding
proposal, the script substrate's WIT package and hosts, the participant gate
and packs plan, the capability crate and servitor.

| Layer | What it is | Where it runs | How it talks |
| --- | --- | --- | --- |
| Foreground, a game | an overlay (ruling 6); it resolves what is foregrounded by its ruleset | input, interface and rendering on armillary's single-threaded kernel, which alone owns the window and the GPU; game logic beside it | intents down, events and views up (§5.2) |
| Background, the sim | the world running with nobody playing | shards of the place graph, each an armillary actor on the pool, Rayon inside a shard (§4.7) | an ordered merge between shards; receipts to the record |
| Infrastructure, the stack | armillary's kernel, actors, pool and generation stamps; eidetic's journals and hagiograph; the script substrate; the gate and the capability algebra; isometer and netrender; moot and murm | each where mere already puts it | `Send` messages between actors; petitions through the gate |

**Armillary.** "A single-threaded host kernel owns the canonical state (the
document model, the GPU device, the window); actors run off-thread and talk
to it only by `Send` message" (`mere/crates/armillary/README.md`). A marker
type makes moving the kernel to another thread a compile error, the pool
keeps the thread count at peak concurrent actors, and generation stamps let
the kernel drop work that returns stale. Its consumers today are canvas,
seiche, crawl, esp and fetch. Its founding proposal says nothing on
determinism or an ordered merge, so §4.7's merge is the sim's to add, not
armillary's to supply.

**Where the boundary has already been instantiated.** The script substrate
is one WIT package, `mere:script@0.1.0`, with two worlds: `app-core` imports
`log`, `caps` and `actions`, and `document-core` imports `log`, `caps`,
`net` and `document-host`; each exports activate, an event handler and
deactivate (`mere/crates/script/wit/world.wit`). A host grants capabilities
per instance and "unimported means unreachable, enforced at instantiation".
A script instance is not an actor of its own: it is "a `!Send` subsystem
built inside the content actor's existing `spawn_on` run closure" (document
script substrate plan, archived 2026-07-03). The extension ladder is the
participant gate and packs plan's **power ladder**
(`2026-07-17_participant_gate_packs_plan.md`, §3), where a *pack* is a
bundle of rungs 1 and 2 and a *mod* is rung 3 and past:

| Rung | Form, in mere's words | Runtime | Web | What the wing puts there |
| --- | --- | --- | --- | --- |
| 1 | "Action macro / scenario data" | none | yes | themes (mere's theme files are serde data with no code), and every open set this record has called data and never an enum: kinds of location, forms of governance, trait catalogues, glyph canons |
| 2 | "piccolo/rhai script" | a script engine | yes | rulesets and world-founding packs, authored in piccolo and lowered to definitions by digest (ruling 41), so the sim evaluates them natively |
| 3 | "wasm component" | "wasmtime (native), jco later" | "later" | new code from outside: a resolver, a generator, a director |
| 4 | "native crate" | compiled in | not distributable | the three first-party cores |

The line that matters most is under that table in mere: "Every rung emits
the same proposals through the same gate and surfaces in the same palette."
The rungs differ in runtime and never in what they are allowed to do, which
the gate decides. And mere already says of the tabletop: "Isometry campaign
packs ride these same rails (same envelope, isometry's own world inside)."

*Reading, withdrawn in part on 2026-09-21.* This record first read the sim's
intent boundary and mere's participant gate as one shape and said the sim
"should be reached through that gate". That was written from a summary of
`servitor/src/lib.rs` without reading the gate. Read since
(`mere/crates/servitor/src/gate.rs:7-29`): a petition is "a batch of
`EditSpec`s against its nested graph", checked for scope over node ids and
facets, and committed by chartulary's `commit_batch`. It is a gate for edits
to a participant's chartulary graph at a person's pace. The sim's state is
not a chartulary graph, and Mesocosm's CLAUDE.md forbids describing world
nouns as chartulary-typed, so the gate itself does not fit and the claim is
withdrawn. What may carry over is narrower: `mere-capability` is described
as the "shared capability algebra for Mere authority providers", so who may
direct which entity could be a provider over its `Power`, `Scope` and
`Facet` order. That is a question for W3 beside ruling 34, not a finding.

### 5.4 What total componentisation would cost

From ruling 76's second question. In the power ladder's terms, total
componentisation is moving the three first-party cores, and perhaps the sim,
from rung 4 to rung 3. **None of this is measured in this stack.** Mere's
own script substrate plan lists "the per-interaction serialization tax" as
"high it is real; unquantified" and says "measure before assuming the
boundary is free". The sizes below are from outside sources and known
mechanics, and are orders of magnitude, not receipts.

| Cost | Size | Basis |
| --- | --- | --- |
| Compute inside a component | roughly 1.2 to 2.5 times native on compute-bound code | web sources read 2026-09-21: one 2026 survey of runtimes puts Wasmtime at 2.41 times native on its suite, others claim 75 to 85 per cent of native; Cranelift is not LLVM and SIMD stops at 128 bits |
| Lost shared-memory parallelism | structural; its size is unknown (first written here as "the largest", withdrawn under ruling 78: shared-nothing shards scale with cores, so what is lost is convenience and halo copies, whose size depends on how well the place graph shards) | a component is single-threaded with its own memory, and WASI 0.3.0 adds async, not threads; Rayon inside a shard goes, and parallelism comes only from an instance per core with halos copied each tick |
| Crossing the boundary | small if batched, ruinous if chatty | a call is tens of nanoseconds before its payload, and lists and strings are copied and allocated on the far side; ten thousand events a tick is well under a millisecond, a hundred thousand calls a tick is most of a frame |
| Views for rendering | paid per change, never per frame | a snapshot cannot be borrowed across the boundary, so terrain and bodies cross as diffs of what is dirty, which the design wants anyway |
| The web | the slowest path for everything | Wasmtime does not run in a browser; mere names jco, "later", and says packs "degrade to rungs 1-2 content there"; a componentised sim would cross through JavaScript glue on every call |

So with a coarse contract the copies are not what hurts. What hurts is
paying one and a half to two times on exactly the part of the program whose
throughput matters, deep time and the background, and giving up Rayon inside
a shard, in exchange for sandboxing code the wing wrote itself. The
sandbox's value is for code it did not write, which is rung 3 already.

**On the web and on mobile (ruling 77).** Mark leans to the two bindings,
"if there is no cost on mobile or the web", preferring "the best
capabilities available to improve performance". The condition holds for
first-party code and fails only for rung 3, checked 2026-09-21:

| Platform | Native binding: the cores and the sim | Component binding: rung 3 mods |
| --- | --- | --- |
| Desktop | direct calls in one address space; actors and Rayon | Wasmtime with Cranelift, ahead of time by preference |
| Web | the same direct calls: the app, its cores and the sim are one wasm module in the browser, so the first-party boundary is free there too. What the web costs is the browser's wasm speed, and Rayon running serially without the nightly, cross-origin-isolated build (§4.7); both are paid whatever the binding | none until jco: mere says "jco later", and packs "degrade to rungs 1-2 content there" |
| iOS | direct calls at native speed | no compiling on the device, since iOS gives third-party apps no writable-executable memory; Wasmtime falls back to its Pulley interpreter, of which its own documentation says "a performance penalty should be expected" (web search summaries, 2026-09-21) |
| Android | direct calls at native speed | not checked |
| Mobile or web as a lens | mere's platform adapter has them as thin clients over p2p, "scores out, scene diffs back, intents through the participant gate", which is §5.2's contract crossing a network with no sim on the device | the host's business, not the lens's |

So two bindings cost first-party code nothing anywhere, because the native
binding is Rust calling Rust, inside the browser's module as much as on a
desktop. What the small platforms cost falls on rung 3 alone, and rungs 1
and 2, data and piccolo, run everywhere. It is also the strongest argument
against total componentisation: on iOS it would put the whole sim under an
interpreter, and on the web behind glue. "The best capabilities available"
is then a tier per platform under one contract, which is how ruling 29's
floor for the web already reads: actors and Rayon on a desktop, one Worker
on the web, an interpreter only for outside code where a platform forbids
compiling it. Mobile is not yet a ruled target of the wing.

*Recommendation, carried by D18 as accepted 2026-09-21.* Keep the first-party cores and the sim at
rung 4, and keep §5.2's discipline so that the choice stays late-binding: a
contract shaped as WIT forces can be bound as a component on any day without
redesign, so the architectural value of the proposal is kept and its runtime
cost is not paid until something needs the sandbox. And take mere's advice
before believing any number here, with one small probe under W2: the same
two workloads, a field pass over places and a due-event queue, built
natively and as a Wasmtime component, run on seeded draws, reporting compute
ratio, cost per crossing and cost per kilobyte.

**Would it force ridiculous workarounds (ruling 78)?** Mark rules for "a
clean boundary either way", because "the lack of a clear boundary would bite
in the future, whereas performance will generally improve with newer
hardware", and asks whether total componentisation buys ergonomics worth
what it costs. First a distinction the question folds together: the clean
boundary and the component are two things. The boundary is §5.2's
discipline, and natively it costs almost nothing, a snapshot where a borrow
would do and values where a reference would do, both of which replay and
threads want anyway. The component is one binding of that boundary, and it
is the binding that costs.

| What total componentisation would force | How bad |
| --- | --- |
| Boundaries drawn by who owns memory, not by concept. Each component has its own memory, so the terrain volume and the big field arrays live in one component and everything else gets copies, diffs or chatty accessors. The result is a few large components, not many small ones | the real design distortion |
| All parallelism as instances with halos exchanged each tick, domain decomposition where native code writes a parallel iterator | well understood and more code; it also forces the ordered merge §4.7 wants designed first, which is a gain |
| Flat interfaces. WIT has no generics, no closures and no recursive values (mere's own WIT notes "WIT value types are not recursive" and flattens its tree to parent ids), so distributions, process definitions and graphs cross as records and handles with conversion code on both sides | friction, not distortion |
| Two toolchains, slower builds, and debugging and profiling across a guest boundary | friction, improving |
| The small platforms: an interpreter for everything on iOS, glue on every crossing on the web (above) | the worst of it |
| A constant factor on compute everywhere | real, and the part newer hardware does absorb |

| What it would buy | Obtainable without it? |
| --- | --- |
| A boundary that cannot be eroded, since no pointer can cross | yes: the contract as its own crate of value types, the sim's internals private behind it, and the component binding kept building and passing the same conformance suite in CI, so the second binding is the boundary's standing proof |
| Floating point that is bit-identical across peers' machines, which replay hashes and the moot need | yes with discipline: one software `libm` everywhere and no platform intrinsics |
| Sandboxing, capability grants, crash isolation, hot reload, any language | only for what is actually a component, which is why rung 3 exists |
| Location transparency: a shard that can run on a thread, a Worker, a subprocess or a peer | yes: armillary's actors already exchange `Send` values, and the contract's values serialise |

*Verdict, carried by D18 as accepted 2026-09-21.* Not ridiculous, and not worth it applied totally.
Nothing on the first list is absurd; each is a known pattern. But the
distortion is real, the small platforms are the worst of it, and nearly
everything on the second list can be had while shipping native, provided the
component binding is kept alive as a tested second binding and never allowed
to rot. One caution on the premise: newer hardware mostly adds cores, not
single-thread speed, so it absorbs a constant factor and rewards whatever
scales across cores, which shared-nothing shards do under either binding.
"Parts that would be good to componentize and parts that might not be" then
has a rule of thumb, call frequency times data volume. Cold and narrow goes
behind a component when it wants to be pluggable: generators, a ruleset's
foreground resolver, a director, format adapters, the hagiograph's
retelling. Hot or wide stays native: field passes, the due-event loop, graph
derivation, the record's commit path, anything feeding the tracer. The W2
probe of this section measures the constant, so the line can be moved later
on a receipt.

### 5.5 The first overlay: Mesocosm (rulings 174 to 184, 192 and 193)

The [Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md) compiles
this section, with rulings 192 and 193 on `mesocosm-core`'s absorption into
Isocosm, into W5's phases.

W5 is Mesocosm's (ruling 174). Its domain is the critter and its lineage
(ruling 35), what it plays follows the lineage's traits (ruling 155), and
its played loop directs, as ruling 60 made its mode (ruling 175): the
critter acts on its own needs and senses, which suggest to the player, and
the player shapes it. The player directs with priorities among needs and
abilities, with places to range, avoid and make home, with stances such as
bold or cautious and whether to contest, yield or share, and with nudges,
one-off suggestions the critter weighs like any other (ruling 176). The
critter follows by its bond, built from how well the player's orders have
served it, ruling 60's opinion weighting turned on the player (ruling 177).
The played slice's direct control is rewritten to match, now (ruling
199), and the plan retires into W5 when M3 lands (ruling 196); its terrarium
section and receipts stand. Whether the bond carries across generations is
a setting with three choices, starting fresh, seeded by the lineage, or the
lineage's own, seeded by default (ruling 178). The player chooses when to
start: at the start of the world's habitability for their critter, or with
as much deep time added as they like (ruling 179).

What the player sees is a mode (ruling 180): survival shows what the played
critter knows, creative shows the truth, and both are needed, debugging
among the reasons; creative mode sees and does not edit, which is the world
editor's (ruling 184). At a birth the player keeps the parent by default,
and may take the offspring instead (ruling 183). A trophic collapse may be local or global, and losing is
not the end (ruling 181): play can go on to see what happens; after a
local collapse the player can start again elsewhere, even with their
lineage if it survives, or their critter can help the world recover,
regrowing producers or fixing the cause as far as its abilities allow. At
the epoch boundary every lineage, played or not, adapts, inherits and
develops, weighing its adaptation against the rest of the trophic web
(ruling 182); that is what an epoch is about, beside the individual
variance of expression rerolled at reproduction (ruling 57).

### 5.6 Eponym's overlay (rulings 185 to 187)

The [Eponym overlay plan](../../eponym/design_docs/2026-09-25_eponym_overlay_plan.md) compiles this section, with rulings 60, 130
and 152 to 156, into W5's phases. Eponym's own plans already hold most of its profile: one named creature,
driven, until death (ruling 60); companions configured in advance and never
commanded, peers who may refuse; succession, "Death, and a companion
becomes the played character"; and no real-time puppeteering of a party.
Three things were ruled against today's rulings. Co-op is each player
living their own named creature in the same world, peers to each other as
to anyone (ruling 185), so ruling 153's shared directing is not Eponym's
co-op shape. At a death, the player chooses who to become, among
companions with a bond to the one who died (ruling 186). Survival and
creative modes hold as in Mesocosm: survival shows what the creature one
lives knows, its diegetic notes included, and creative shows the truth and
is where tag-in lives (ruling 187).

### 5.7 The VTT's overlay (rulings 188 to 191)

The [VTT overlay plan](../../design_docs/2026-09-25_vtt_overlay_plan.md) compiles this section, with rulings 114, 154
and 189, into W5's phases. The VTT's product description holds its profile: a substrate for tabletop
systems over the Isocosm sim, systems as plugins, the substrate tracking
geometry and turns and never a hit point. A campaign may switch the sim off
and play as a plain tabletop (ruling 188). A system plugin that cannot pass
ruling 114's calibration may still play in a debug or experimental mode,
with a warning (ruling 189). *Readings put to Mark on 2026-09-25 without
objection:* a campaign's time passes as the table plays, downtime the DM
declares running the sim forward as deep time does, with the players'
consent, and between sessions only if the founder turned the unattended
rate on (rulings 104 and 105); players see what their characters know and
the DM sees the truth, survival and creative split by role. *Reading, not
ruled:* with the sim off there is no background to agree with, so any
ruleset plays there and calibration does not apply.

An adventure pack whose requirements the world lacks waits, dormant until
the world meets them, or the GM forces it, asserting its content over what
was generated (ruling 190). The VTT offers the arcs the sim is running to
the DM as suggested hooks, to take up, drop or reshape (ruling 191).
*Readings put to Mark on 2026-09-25 without objection:* a campaign starts
from a drawn world or from the DM's own maps and packs, authored content
filling or displacing generated content (ruling 89), or with the sim off
(ruling 188); player characters are contingent on groups "even if by
absence" (ruling 36), members or outsiders like anyone, where they start
being the DM's setup; and the DM's prep is world editing, the world
editor's (rulings 89, 156 and 184).

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
| `MAX_BRICKS` is three layout constants; `with_capacity` and `retarget` exist and Eponym uses them | Read in modulus `lib.rs` and Eponym `residency.rs`, 2026-09-17 | Checked |
| Renderling is retired; isometer-render depends on wgpu only | Read in L7 and `isometer-render/Cargo.toml`, 2026-09-18. W1 found the check too narrow: `eponym-client` still depends on renderling unconditionally and L7 is unmet | Checked for isometer-render; wrong as a wing claim, corrected in §4.2 |
| nisus has per-cell edits and dirty regions and `Ground` does not use it | Read in nisus `lib.rs` and isometer-core manifest, 2026-09-18 | Checked |
| WebGPU compatibility tier caps 3D textures at 256 a side | From the wgpu limits tables, 2026-09-17 | Unchecked against the spec text |
| RimWorld derives regions and rooms from cells and runs temperature and pathing over them | Prior-art memory | Unchecked |
| petgraph carries connected components, Dijkstra, A*, k-shortest paths, dominators, min cut and subgraph isomorphism | Prior-art memory; version in chartulary not read | Unchecked |
| Basic Fantasy RPG's line is CC BY-SA since 2023 | Prior-art memory | Unchecked |
| Storage and processing scale as §3.5 claims at hundreds of thousands of entities | Argument from the derivation rule, not measured | Unchecked; the first measured target is a region (ruling 124), a done-condition of the sim plan's S5 |

## 8. The contradictions of 2026-09-16, and what resolves each

| Contradiction | Resolved by |
| --- | --- |
| Shared organs extracted from one product's path and shaped by it; the second consumer's plan titled "what isometer offers" | §2's tier test and §6: the stack is designed from the sim's nouns, never from a product's data |
| Two grains, a six-inch voxel world and a five-foot sprite grid, sharing one scene | §3.6: one base unit, power-of-two ratios per chunk; a tile is a brick face |
| Two theories of appearance in the VTT, stylesheet and voxel-sourced, with the colour table now generating the CSS | Open: a game-overlay question (§5), to be ruled in the VTT's own plan |
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
   `paredros-core` (after ruling 109, `eponym-world` becomes
   `eponym-core`; Mark, 2026-09-24: "Yes"), and the three product cores are defined in one standard
   way that plugs into the sim; that standard is the game-overlay contract
   of §5, a trait each core implements against the sim, in the spirit of
   mesquite's `Product`. **Ruled 2026-09-18: isotropy** for the sim, and
   **isostasy** for whatever bridges effects between the layers. Games,
   studios and marks remain to be checked before banking; claim by publish.
   **Checked 2026-09-22 by web search:** isotropy is a 2015 iOS puzzle game
   (Dmitriy Prikhodko) and Isotropic Games a placeholder studio site;
   isostasy is a live Steam title, an NES-style Metroidvania by Gravel
   Studios. Mark, ruling 108: neither word is to be a game name, so there is
   no conflict; both are banked as internal names for the sim and the
   bridging organ and never as product titles. Trademark registers were not
   searched. Claim by publish stands. **Superseded on 2026-09-22 by
   rulings 109 to 112:** the sim and the family are Isocosm. These dated
   lines keep isotropy, restored by ruling 198.
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
   record (ruling 28). **Closed 2026-09-18:** kiss3d 0.46 takes an
   external device and renders offscreen (§4.2), so it is a tenant, and
   ruling 27's composition test is met on paper; the running proof is
   W3's.
6. **Which game first.** Answered: the sim gets a bench, and ruled
   further the same day: **one bench** with lanes for the sim (processes
   and effects including magic), the world, specimens, items and effects,
   which is also the dev tools. W4 is that bench; the first game overlay
   follows it.
7. **`ProcessDef` as the base profile's process definition.** Answered,
   ruling 32.
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
   **Closed 2026-09-18, marked so 2026-09-22:** Mark restated pillar 2
   ("Maps at every scope, sculpted... Same world, different scopes") and the
   file notes that the earlier 15 by 15 to 30 by 30 "was a view, not a
   limit"; this item stayed marked open by error until the to-do review.
10. **The vertical scale and the shipped step** (from §3.6). **Ruled
    2026-09-18: a per-axis scale, y and z as well as x, added to the
    tracer under W3** (ruling 23), so the shipped step is reproduced
    exactly and the base unit need not be cubic. Paging is fixed in every
    consumer under the same ruling.
14. **Directing against driving:** **ruled, ruling 60.** Directing is
    Mesocosm's mode over its critter and its cohort. Driving is
    Eponym's, because you are one sophont, and the taste record's
    skill-based action stands; directing composes on top of it as the
    directives that sophont gives its companions, whose efficacy is
    modified by their opinion of it. That is Eponym's first pillar,
    peers you address rather than units you command, given its
    mechanism: a directive to a peer is a request, and ruling 51's
    alignment and ruling 37's disposition set how far it is honoured.
    Isometry's turns are their own mode. The overlay contract therefore
    carries both a directive vocabulary and, where the overlay opens it,
    actuation of one body.
13. **The referent of divine power:** **ruled, ruling 52.** Fixed as the
    domain's effect for the lower tiers, chosen by the greater tiers, all
    bound by frequency weighted by impact.
12. **Rulesets over the sim** (§5.1): (a) **ruled, ruling 41:** world
    effects and game rulesets are one language at two scopes,
    world-founding and play, both packs lowered to rulesets recorded by
    digest; effects entailed by world composition are a later difficulty.
    (b) **narrowed by ruling 41:** the next ruleset consumer after the d20
    family is a PbtA-shaped system; Lancer waits on the space scope, a
    desired extension of the sim that is not in this round. (c) **closed
    by ruling 40 and the licence text:** Daggerheart is out, Lancer is in
    with attribution, ICON and CAIN are a question to Massif Press.
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
    verbs. **Ruled 2026-09-18 (ruling 33): amend both.** Drafts for
    Mark's reading and commit, since the founding record is wing law and
    the CLAUDE.md is his:

    *Founding record §1, replacing "they do not share a genre, a schedule,
    or their verbs":* "They share a world substrate, a lineage model, a
    trust plane, one clock and the simulator's own verbs, which each game
    reaches as handles; they do not share a genre, and each owns the verbs
    it lays on top. (Amended 2026-09-18; the wing design record, rulings 3
    and 10.)"

    *Founding record §6, replacing "the platform is extracted from shipped
    games, never built platform-first":* "The simulator and the stack are
    designed from the games' systems in combination and tested by seeded
    draws from the generator; a game is an overlay designed against them.
    Nothing is declared representative by being built small first.
    (Amended 2026-09-18; the wing design record, §2 and §6.)"

    *`mesocosm/CLAUDE.md`, Important Don'ts, the federation line:* keep
    "Do not build the federation platform first" as written, since it is
    about federation and not the sim, and add after it: "The simulator is
    the exception by ruling of 2026-09-18: it is designed from the games'
    systems in combination, not extracted from a shipped game. See the
    wing design record." The `PROJECT_DESCRIPTION.md` restatements remain
    Mark's own words.
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
- **W1, every active plan evaluated.** Landed 2026-09-18: 61 rows, 30 keep,
  22 rewrite, 6 retire, 3 surfaced; applied to every plan and index. Done when each active plan in the
  three products' indexes carries one line against this record, keep,
  rewrite or retire, with the tier it belongs to and the assumption it
  rests on that this record confirms or contradicts. The board-on-isometer
  plan is first, since it is the worked example. Retired plans move to
  `archive_docs/<date>/` with rationale, per policy; nothing is deleted.
- **W2, the sim's design.** Done when the base profile has its own plan
  under this record, with the taxonomy of §3 as a schema, the three process
  shapes as definitions, the record and reach field specified, and a
  generator whose declared space is the source of every receipt. Drafted
  2026-09-22 as the [sim plan](2026-09-22_sim_plan.md): met on paper, the
  docket items its §8 lists being ruled or moved (ruling 106; D19 and D20
  by rulings 154 and 152).
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
  receipts drawn from the generator. **Ruled 2026-09-24 (ruling 174):
  Mesocosm goes first,** and its played loop directs, as ruling 60 has it
  (ruling 175, 2026-09-25). Planned as the
  [Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md), phases M0
  to M4, M0 and M1 done on 2026-09-25 and M2 to M4 proposed. Eponym's and
  the VTT's overlays were planned on 2026-09-25 as the
  [Eponym overlay plan](../../eponym/design_docs/2026-09-25_eponym_overlay_plan.md), E0 to E4,
  and the [VTT overlay plan](../../design_docs/2026-09-25_vtt_overlay_plan.md), V0 to
  V4, proposed and not opened; they go side by side after Mesocosm's M3
  (ruling 231).

No code lane ran before W1 was ruled; the sim's lane opened after it, on
2026-09-22.

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

- 2026-10-05: checkpoint 9 opened; its first round ruled (560 to 562):
  every cell carries along a route and conduits more, an intact body
  carries all its natives ask, and a process reads its system part by
  part. Annotation on 477. Carried into the sim plan's checkpoint 9 brief,
  the anatomy brief's §1, the session notes' §10 and the index.
- 2026-10-04: rulings 557 to 559 recorded at the Burn migration session's
  request: pre.4 is promoted into mere's main and the Knot and Isometry
  repins follow as handoffs (557); the web workspaces commit the getrandom
  cfg (558); the OPFS probe takes its rustfmt sweep (559). Annotations on
  534 and 555. Carried into the session notes' §9.9 and the index.
  Isometry's repin onto pre.4 is a handoff to come.
- 2026-10-04: checkpoint 8, development, merged at `e3297e6` under rulings 510
  to 531, 535, 544 and 547 to 554, certified with its controls; the
  readings taken while building it are in the sim plan's Findings for Mark
  to reject.
- 2026-10-04: ruling 556 recorded at the Burn migration session's
  request, widening 537 and 545: every wasm module in mere moves to
  wasm-bindgen 0.2.129, the OPFS probe and the minimal repros among them,
  rather than installing the 0.2.126 CLI for one probe's gate control.
  Annotations on 537 and 545. Carried into the session notes' §9.9 and the
  index.
- 2026-10-04: ruling 555 recorded at the Burn migration session's
  request, carrying out 534: the quiet GPU-on A/B bounds the ambient load
  before each launch, after 0 of 80 repetitions met a bound that counted
  the measured page's own launch. Annotation on 534. Carried into the
  session notes' §9.9 and the index.
- 2026-10-04: ruling 554 recorded, opening checkpoint 8's step 8i: a
  young is weaned the first time it is fed, nursing reads only unweaned
  young, and the crowd runs a parent and its unweaned young as
  individuals, relation kept, the mechanism by which it will hold any
  relation. Annotation on 526. Carried into the sim plan's checkpoint 8
  brief, the session notes' §10 and the index.
- 2026-10-04: rulings 552 and 553 recorded, building checkpoint 8's step
  8h: a bud severs once it holds a provision's worth (552), and the
  probe's soil is drawn from 200 to 12,000 mg a site (553), after
  measuring that whole-frond buds never severed within a run and that
  most worlds held less matter than their producers' bodies. Annotation
  on 524. Carried into the sim plan's checkpoint 8 brief, the session
  notes' §10 and the index.
- 2026-10-04: ruling 551 recorded, building checkpoint 8's step 8h: full
  stores count as fed, TD5's horizon capped by what a body's stores may
  hold, after measuring that no probe grazer's store could reach 100 ticks
  of rent. Annotation on 519. Carried into the sim plan's checkpoint 8
  brief, the session notes' §10 and the index.
- 2026-10-04: rulings 549 and 550 recorded, building checkpoint 8's step
  8g: flora and fauna draw recipes while myco and micro wait for
  territories and surfaces (549), and the generator draws within
  Mesocosm's bounds where it has them, the clutch 1 to 4 (550).
  Annotation on 512. Carried into the sim plan's checkpoint 8 brief, the
  session notes' §10 and the index.
- 2026-10-04: rulings 547 and 548 recorded, building checkpoint 8's step
  8g: a generated recipe reproduces in one cell of its root (547), and a
  founded lineage's life history is drawn within what its body allows
  (548). Annotations on 512 and 513. Carried into the sim plan's
  checkpoint 8 brief, the session notes' §10 and the index.
- 2026-10-03: rulings 545 and 546 recorded at the Burn migration session's
  request: the web modules take wasm-bindgen 0.2.129 with wgpu 30.0.1,
  carrying out 537 (545), and the Distillery probe's decoder model may be
  downloaded as stated (546). Annotation on 536. Carried into the session
  notes' §9.9 and the index.
- 2026-10-03: ruling 544 recorded, building checkpoint 8's step 8f: an
  incorporated part keeps its donor's matter, unspendable by the eater
  and counted against the part's room. Annotations on 504 and 516.
  Carried into the sim plan's checkpoint 8 brief, the session notes' §10
  and the index.
- 2026-10-03: rulings 542 and 543 recorded, following 538: an edition's
  own state persists beside the body, its relations to it declared (542),
  and unattended a ruleset-led world runs the ruleset's procedures under
  declared decision policies (543). Annotation on 538. Carried into the
  sim plan's §3.5, the VTT's overlay plan, the session notes' §10 and the
  index.
- 2026-10-03: rulings 538 to 541 recorded, rulesets over the sim, raised
  by Mark with research and its prior art: each world chooses whether the
  sim or its ruleset leads (538, against the recommendation, amending
  114); faithful versioned editions with a toolkit grown from them (539);
  hybrids by named profile (540); the VTT faithful, Eponym and Mesocosm
  native (541). Annotations on 41, 114 and 189; the prior art in §5.1.
  Carried into the sim plan's §3.5, the VTT's overlay plan, the session
  notes' §10 and both indexes.
- 2026-10-03: ruling 537 recorded at the Burn migration session's request:
  the web modules take the newest wasm-bindgen they can (Mark: "Take the
  newest ya can"), after 536's helper met the model probe's older pin.
  Annotation on 536. Carried into the session notes' §9.9 and the index.
- 2026-10-03: ruling 536 recorded at the Burn migration session's request,
  Mark having ruled it there against the recommendation: 532's constructor
  helper lives in cambium-genet-web-host rather than a new crate.
  Annotation on 509. Carried into the session notes' §9.9 and the index.
- 2026-10-03: ruling 535 recorded, building checkpoint 8's step 8c: whether
  a lineage breeds from income or from capital, growth with its parts full
  going to its missing parts and provision before or after its reserve, is
  its trait (against the recommendation). Annotation on 479. Carried into
  the sim plan's checkpoint 8 brief, the session notes' §10 and the index.
- 2026-10-03: rulings 532 to 534 recorded at the Burn migration session's
  request, Mark having ruled them there: one stack helper runs pre.4's wasm
  constructors once for every web module (532), Distillery's model probe
  takes it and the minimal repros do not (533), and promotion waits on a
  GPU-on A/B taken with the machine quiet (534). Annotation on 509.
  Carried into the session notes' §9.9 and the index.
- 2026-10-03: rulings 528 to 531 recorded, checkpoint 8's fifth round: the
  probe's grazers give milk (528); the probe draws semelparity per cohort
  (529); how many eggs a birth lays is a lineage trait (530, against the
  recommendation); and a recipe carries its absence odds beside its
  variance (531). Annotations on 448, 478, 521 and 527. Carried into the
  sim plan's checkpoint 8 brief, the anatomy brief, the session notes'
  §10 and the index.
- 2026-10-03: rulings 523 to 527 recorded, checkpoint 8's fourth round: a
  body starves when its rent fails, Mesocosm's 20 mg line going (523,
  evidence returned against 513 and 518); a bud is a part grown off its
  parent (524, against the recommendation); only worlds that ask for
  bodies found from recipes (525); parental care feeds hungry young the
  parent's own mouthfuls (526), and, put back at Mark's question, milk or
  mouthfuls by lineage trait (527). Annotations on 448, 450, 470, 504 and
  512.
  Carried into the sim plan's checkpoint 8 brief, the anatomy brief, the
  session notes' §10 and the index.
- 2026-10-03: rulings 519 to 522 recorded: hunger switches a grazer
  between an egg and a brood (519); growth's order between a missing part
  and the provision is a lineage trait (520, against the recommendation);
  semelparity and parental care come with development, mating and
  horizontal transfer with the boundary family (521); and Ars Magica's open
  text becomes a VTT ruleset and a reference for the wing's magic (522,
  raised by Mark), its prior art in §5.1. Annotations on 404, 450, 479,
  502, 514 and 518. Carried into the sim plan's checkpoint 8 brief, the
  anatomy brief, the session notes' §10, the index and the VTT's
  `CLAUDE.md`, whose content line is widened at Mark's word.
- 2026-10-03: rulings 514 to 518 recorded, checkpoint 8's second round:
  the check certifies all three strategies (514); reproduce volume scales
  what a birth spends, nothing hardcoded (515, Mark's reframe); a whole
  part is incorporated as affinity decides (516); the probe runs long
  enough for births (517); and a birth spends the provision its reproduce
  cells fill, with no fraction or interval (518, put back from 515).
  Annotations on 447, 448 and 479. Carried into the sim plan's checkpoint
  8 brief, the anatomy brief, the session notes' §10 and the index.
- 2026-10-03: rulings 510 to 513 recorded, checkpoint 8's first round:
  attachment offsets built in checkpoint 8 (510), a recipe's kinds as part
  templates (511), founded recipes drawn by kingdom with the roster as
  presets (512), and the probe's bodies expressing reproduce in one cell
  from founding (513). Annotations on 462, 478, 505 and 506. Carried into
  the sim plan's checkpoint 8 and 9 briefs, the anatomy brief, the session
  notes' §10 and the index.
- 2026-10-03: rulings 508 and 509 recorded at the Burn migration session's
  request, Mark having ruled them there: the allocator repair goes in
  burn-remote's close path (508), answering 411's fork, and a bounded lane
  makes pre.4's wasm constructors run once before pre.4 reaches the web
  (509). Annotation on 411. Carried into the session notes' §9.9 and the
  index.
- 2026-10-03: checkpoint 7, matter in parts, merged at `c51bb0a` under rulings
  504 to 507, certified with its controls; the readings taken while
  building it are in the sim plan's Findings for Mark to reject.
- 2026-10-02: ruling 507 recorded, while certifying checkpoint 7: the
  averaged crowd averages all of a lineage's own matter, tissue and
  reserve, producers keeping no reserve under 506. Annotation on 209.
  Carried into the sim plan's checkpoint 7 brief, the session notes' §10
  and the index.
- 2026-10-02: rulings 504 to 506 recorded, checkpoint 7's forks: each part
  keeps its own ledger and is a holder (504); income and the mouthful read
  their measurements alone, calibrated to the median body (505), against the
  recommendation; and the probe's grazers store while its producers stay
  lean (506). Annotations on 459, 463 and 493. Carried into the sim plan's
  checkpoint 7 brief, the anatomy brief, the session notes' §10 and the
  index.
- 2026-10-02: rulings 501 to 503 recorded, the bodies family's shape: four
  checkpoints in order, matter in parts, development, systems, harm and loss
  (501); territories and surfaces built last, Mark noting their likely use
  for auras (502); and the systems checkpoint riffing (503). Annotations on
  433, 487, 488 and 491. The family is briefed in the sim plan's §9 as
  checkpoints 7 to 11, and §2.3 gains an anatomy row.
- 2026-10-02: rulings 497 to 500 recorded, the anatomy brief's ninth round,
  which rules it through: territories and surfaces lie in each place's
  ledger under their keys (497) and spread by overflow along routes (498), a
  germ lineage forks strains (499), and functions scale by the matter at
  each place (500). Annotations on 58 and 488. Carried into the brief, the
  sim plan, the session notes' §10 and the index.
- 2026-10-02: rulings 493 to 496 recorded, the anatomy brief's eighth round:
  each function's measurement, by its share of the cells (493); the eight
  names read from the box and the tree, tube and shell declared (494);
  segment growth a lineage trait, epimorphic by default (495); and a wound
  taking cells in proportion, healed first (496). Annotations on 276, 466,
  469, 478 and 492. Carried into the brief, the sim plan, the session notes'
  §10 and the index.
- 2026-10-02: ruling 492 recorded, the anatomy brief's seventh round, raised
  by Mark: no shape gates a function, and what a part does scales with the
  measurement its mechanic names. Annotations on 276, 338 (with 488's
  settling of its spread question), 405, 466 and 491. Carried into the
  brief, the sim plan, the session notes' §10 and the index.
- 2026-10-02: rulings 489 to 491 recorded, the anatomy brief's sixth round:
  the default systems take the proposed table of ten (489), the natives read
  them from the bodies family on (490), and the generator riffs systems by
  substitution (491). Annotations on 277 and 465. Carried into the brief,
  the sim plan, the session notes' §10 and the index.
- 2026-10-02: ruling 488 recorded, the fifth round's first question after
  Mark's reframe of it: a mold is a territory and micro life a surface,
  amending 58, 467 and 480 for these forms; annotations on 58, 467, 480 and
  §3.2.1's body forms. Carried into the brief, the sim plan, the session
  notes' §10 and the index.
- 2026-10-02: rulings 485 to 487 recorded, the anatomy brief's fifth round:
  a severed part grows back only where the lineage can heal (485), every
  fragment of such a lineage lives and physiology decides which last (486),
  and the brief rules its open items before the bodies family's brief, the
  spread question and then the default systems (487). The round's first
  question, whether a body is a placemap, came back reframed, whether a
  spread and micro life are territories or surfaces, and is put back.
  Annotations on 469, 470 and 478. Carried into the brief, the sim plan, the
  session notes' §10 and the index.
- 2026-10-02: rulings 481 to 484 recorded from the Conatus/physics session,
  the ECS left open at 476: hecs stays (481); the mode host and armillary
  schedule, the ECS is storage (482); the projection's diff comes from the
  record's receipts (483); a boundary crate in Mere holds the ECS (484).
- 2026-10-02: rulings 477 to 480 recorded, the anatomy brief's fourth round:
  a body's system network read from its tree, a system working by degree
  (477); the lineage keeping Mesocosm's recipe, a maturing body growing
  toward it rather than its own soma (478); a missing part grown once the
  body's parts are full, at PD2's price (479); and a spread acting wherever
  a patch lies (480). Mark chose against the recommendation on 477 and 478.
  Annotations on 338, 465, 467 and 468. Carried into the brief, the sim
  plan, the session notes' §10 and the index.
- 2026-10-02: rulings 471 to 476 recorded from the Conatus/physics session
  (Mark asked them carried here): kiss3d, reshaped, as the lit body tenant
  (471); a stack-owned light and environment block (472); WGSL/WESL for
  raster and CubeCL for compute, amending the 2026-08-16 line (473); the
  renderling fork archived after L7 (474); R2 re-proved in isometer-render
  (475); vello for documents and kiss3d 2D for lit games (476). Open: the
  ECS, hecs against shipyard, compared and coming back to Mark.
- 2026-10-02: rulings 467 to 470 recorded, the anatomy brief's third round:
  one part model for every body form, a part attaching to a parent or lying
  on a site (467); a lineage's recipe, from which a child develops and a
  maturing body grows the parts it lacks, incorporation teaching the lineage
  what it eats (468); a wound loses cells, regrown at a price where the
  lineage can (469); and a severed part becomes a body of its own, alive
  where its lineage regrows from fragments (470). Mark chose against the
  recommendation on 468 and 470. Annotations on §3.2.1's body forms, 58,
  123, 447, 448 and 460. Carried into the brief, the sim plan, the session
  notes' §10 and the index.
- 2026-10-02: rulings 463 to 466 recorded, the anatomy brief's second round:
  a store holds what its cells can and a body without one keeps no reserve
  (463), rent, growth and a landing meal reach parts in proportion (464),
  organ systems are named networks of functions (465), and the fifteen
  functions' shapes and seeding (466). Annotations on 277, 446, 459 and 461,
  and cite corrections on 456 and 464: the proportional take is 294's, not
  287's. Carried into the brief, the sim plan, the session notes' §10 and
  the index.
- 2026-10-02: rulings 459 to 462 recorded, the anatomy brief's first
  round: tissue in parts and the reserve in stores (459), cells derived
  from extents (460), all fifteen functions with the other vocabularies
  folded in (461), and attachment offsets kept by the sim (462).
  Annotations on 339, 405, 446, 453 and 456; the brief opened as
  `2026-10-02_anatomy_brief.md`, and carried into the sim plan's §2.3,
  the session notes' §10 and the index.
- 2026-10-02: rulings 457 and 458 recorded from checkpoint 6's report: the
  order from bodies onward stands (195, 262), and the anatomy brief comes
  next (281). Annotations on 195, 262 and 281; carried into the sim plan's
  status, the session notes' §10 and the index.
- 2026-10-01: checkpoint 6 merged at `b4345cc`, built under rulings 446
  and 451 to 456 and verified on its branch: the five natives run as
  definitions over a minimal allocated body, the crowd is certified for
  part-bound processes on 1,000 draws, and 4b's hunting checks pass again
  under 454's shares. The sim plan's Findings holds the extension list and
  the density, about 1.15 living members a state; the order from bodies
  onward (195, 262) goes back to Mark.
- 2026-10-01: rulings 454 to 456 recorded while building checkpoint 6's
  step 5, from assessing the body family against Mesocosm's tick: shared
  ground shared out from the pass's start (454, carrying out 269 and 306), a
  body's ceiling read part by part (455), and the probe's meal taking tissue,
  with where matter sits in a body handed to the anatomy brief (456, after
  Mark's free-text question was answered and put back). 446's reading
  corrected: a spill is not itself a mineralization. Annotations on 269,
  281 and 306; carried into the sim plan's checkpoint 6 brief and status,
  and the session notes' §10. The kernel (`2ec21eb` to `93e58f6`) is on the
  branch, not yet on main.
- 2026-10-01: ruling 453 recorded while building checkpoint 6's step 5: the
  probe's minimal body is optional fields on the core part, amending the
  brief's reading that it would live in the probe's world. Step 4 and X3
  are built and verified on the `checkpoint-6` branch, not yet on main.
  Carried into the sim plan's checkpoint 6 brief, the session notes' §10
  and the index.
- 2026-10-01: rulings 451 and 452 recorded as checkpoint 6 opened: an
  epoch ends by one of Mesocosm's three rules, a year by default (451), and
  deep time's span counts epochs (452, over the recommendation of world
  time). Carried into the sim plan's checkpoint 6 brief, the session notes'
  §10 and the index.
- 2026-10-01: rulings 446 to 450 recorded, checkpoint 6's design: a body's
  tissue and reserve (446), and reproduction as several strategies governed
  by traits (447), the default set brood, seed or egg, budding and fission
  and spores (448), chosen by a trait the body must support and switched by
  circumstance (449), with mating, semelparity or iteroparity, parental care
  and horizontal transfer (450). Lane A's checkpoint 3 plan, found only in
  that lane's session, has its steps 4 and 5 restated in the sim plan's §9
  as checkpoint 6's brief. Carried into the sim plan's §2.3, §3.4, §9,
  status and Progress, the forms-of-life brief's §F, the session notes'
  §10 and the index.
- 2026-10-01: ruling 445 recorded, the mode host's knowledge contract: a
  window's access follows its seat and subject, never its mode. The mode
  host has no open design item; it waits on Mesocosm's M3 and on Eponym's
  renderling retiring. Carried into the isomere plan, the session notes'
  §10 and the index.
- 2026-10-01: rulings 442 to 444 recorded, the mode host's first round,
  after its assessment found no game running on the sim: the all-modes build
  in a host workspace of its own (442), a window playing one mode and
  switching in place (443), and the world save holding the sim's history and
  a section per mode (444). Carried into the isomere plan's status, §2.4 and
  Progress, the session notes' §10 and the index.
- 2026-10-01: rulings 439 to 441 recorded, time at scale's first round,
  after the 09-25 scale points were remeasured on today's core (n^1.02,
  33 ms a tick at 4,096 members; the history run still dies by tick 65):
  grouping stops paying below a world rule's members per state (439), rate
  models are certified once per rules digest on the bench (440), and a
  cost-only century draw is measured before viability (441). Carried into
  the sim plan's §5.3, S5, Findings and Progress, the session notes' §10
  and the index.
- 2026-10-01: rulings 435 to 438 recorded, the four forks the briefs'
  writing found: a cohort's tally of condition causes beside its state
  (435), one record per key combined by the key's stacking (436), minimaps
  and overmaps as second cameras on the ladder (437), and far marks drawn as
  pixel icons from each glyph's display (438). Carried into the sim plan's
  §3.6 and S7, the presentation plan's L10, the session notes' §10 and the
  index.
- 2026-10-01: ruling 434 recorded, 432's choice named the player's filter.
  At Mark's word the two briefs were written: the detail ladder as the
  presentation plan's lane L10, and conditions and modifiers as the sim
  plan's §3.6 with phase S7. The writing found four forks, recorded in the
  session notes' §10 and put to Mark.
- 2026-10-01: rulings 430 to 433 recorded, one zoom's third round: 428
  settled against §3.5 and ruling 123, §3.5's fourth rule narrowed to the
  environment's conditions with the record beside the ledger (430);
  modifiers declared distributive or collective (431); far marks drawing
  the effects' glyphs through a lens the player sets, the word's collision
  with the wing's projection lens left to Mark (432); and a modifier's
  reach limited to the scopes the sim holds (433). §3.5 carries a dated
  amendment. Carried into the sim plan's §2.3, §2.4 and §2.5, the session
  notes' §10, the presentation plan's open decisions and the index.
- 2026-10-01: rulings 426 to 429 recorded, one zoom's second round: the
  transition seamless where rungs share geometry and dithered where they do
  not (426), every modification an effect from the world's one vocabulary
  with a glyph one source among many (427), an entity's condition record
  (428), and modifiers combined in layers the world orders (429). 428 was
  reopened the same day, before reaching the sim plan, on the record's §3.5
  ("Conditions live on places, not on things") and ruling 123's wounds in
  the ledger, which its question's evidence left out; it is put back to
  Mark. Carried into the session notes' §10, the sim plan's §2.4 and §2.5,
  the presentation plan's open decisions and the index.
- 2026-09-30: rulings 423 to 425 recorded, the first round of one zoom, the
  wing's detail ladder: five rungs in one camera with the far marks read as
  modifiers (423), the finest rung a thing's size warrants among those the
  sim holds (424), and terrain in nested rings inside the host's budget
  (425). The transition between rungs came back as a question, answered in
  the session notes' §10 with what a continuous morph would require, and
  423 opened how conditions, statuses and modifiers sit on entities, singly
  and in aggregate. Carried into the session notes' §10 and the
  presentation plan's open decisions.
- 2026-09-30: rulings 412 to 422 recorded, the spatial spine's SP4 and SP5
  designed: earth is matter (412), edits as shape operations (413), one fact
  read across borders (414), columns plus exceptions (415), overflow heaped
  beside the cut (416), walkable patches (417) at the base grain with
  clearance on edges (418), a world-rule climb (419), the patch cap's
  presets (420) and default (421), and an entity naming its site until its
  game places it (422). Carried into the spine plan's §A.11 and §A.12, the
  session notes' §10 and the index.
- 2026-09-30: **ruling 411's diagnosis is verified; repair choice pending.**
  Control and diagnostic runs with the same executable/model inputs both
  retained 10 allocations / 5,323,776 active bytes and exited 1 with empty
  stdout. Four further samples without explicit synchronization were
  unchanged. `client.sync().await` returned `Ok(())` in 1.9222 ms; active
  counters became zero, and subsequent cleanup cleared the 41,943,040
  reserved bytes. Source bytes and modification time were restored. Nine
  corrupted-evidence controls were rejected. The evidence supports a
  completion/polling issue after teardown submissions; it does not identify
  the exact retained allocations or prove a production repair. Strict
  numerical error, fresh-session recovery and preservation of another live
  lease remain unverified. Evidence and review qualifications:
  `Code/testing/mere/receipts/2026-09-29/pre4-s13/allocator-diagnosis/independent-diagnostic-outcome.json`.
  The adjacent `repair-fork-pending-question.json` asks whether to repair
  the existing `burn-remote` close path, change general CubeCL polling, or
  park the migration. No repair is selected and no new ruling is recorded;
  production repair, migration acceptance and integration remain held.
- 2026-09-30: ruling 411 records Mark's "A!": bounded allocator diagnosis
  proceeds with the zero-active baseline retained. Ownership or patch-design
  changes return as forks. This answers the diagnosis/park question below;
  broader migration acceptance, merge and downstream repin remain held.
- 2026-09-29: **pre.4 stopped at remote allocator reclamation.** After
  ruling 410's comparison and paired launcher checks, the migration lane
  retired the four `burn-cubecl` selectors locally at Mere `124fc42b`.
  The subsequent sealed remote lifecycle run exited 1 without timing out:
  its zero-active baseline ended with 10 active allocations / 5,323,776
  active bytes. Reserved bytes were 41,943,040, not measured physical VRAM.
  Source, binary and model inputs were unchanged. JSON stdout was empty;
  the native-control stage marker establishes neither the strict prior
  numerical ceiling nor fresh-session recovery. Cause remains unknown.
  Stop rule 4 holds further browser GPU checks, candidate edits, merge and
  downstream repin pending Mark's answer to bounded diagnosis versus
  parking pre.4. No new ruling or migration acceptance is recorded here.
  Evidence: `Code/testing/mere/receipts/2026-09-29/pre4-s13/post-retirement/root-remote-allocator-stop.json`.
  Isometry's accepted Mere `32edc2ad` / Genet `7a60ad79` pins are unchanged.
- 2026-09-29: ruling 410 records Mark's "I suppose A, then B if we can":
  compare historical unpatched pre.2 in the current browser first, then
  retire the patch if supported. The conditional-retirement interpretation
  is labelled a reading. The migration lane owns execution; neither this
  answer nor the unexpected unpatched pass accepts the remaining S13 gates.
- 2026-09-29: the scroll repair is published through Mere `32edc2ad` and
  adopted by the seven Isometry manifests with Genet `7a60ad79`. The ordinary
  row test holds all 187 rows with its original allowance; all four workspace
  checks and 27 focused spine/lift tests pass. Eight locked graphs and 22
  deliberate source/path faults preserve the inherited optional lineages and
  qualified local renderer context. The side-panel plan owns the detailed
  receipt. Mere's later semantic main `99e44853` is being reconciled into
  pre.4 after a reviewed local pre-build checkpoint `0ea65fb6`; that checkpoint
  is not migration acceptance, and S13 remains held. Earlier disk/scroll holds
  below are historical. No new design ruling follows.
- 2026-09-28: rulings 408 and 409 recorded, SP3's capture for the spatial
  spine: water as opaque voxels (408), a border window and an overview
  (409). SP3's handoff is the spine plan's §A.10.
- 2026-09-28: at Mark's "Let's document!", recorded rulings 404 to 407 and
  the current architecture; refined composition, geometry dependence,
  capability/repertoire/proficiency, contextual embodiment and selected
  condition carryover in their existing plans. Documentation only; the
  spatial spine's SP2 work and all existing implementation gates stand.

- 2026-09-28: rulings 400 to 403 recorded, SP2's forks: the skeleton given
  back exactly where it can be (400), a Coons patch interior (401), columns
  per chunk (402), world-local material ids (403). Carried into the spine
  plan's §A.9, the session notes' §9.6 and the index.
- 2026-09-28: rulings 397 to 399 recorded, SP1's brief: borders on routes
  (397), the map family as an optional part of Founding (398), scale-free
  draws (399). Carried into the spine plan's §A.8, the session notes' §9.6
  and the index.
- 2026-09-28: rulings 393 to 396 recorded, the spine plan's four decisions:
  terrain models in Isocosm, amending the isoscape plan's ruling 11 (393);
  the skeleton as condition keys (394); square sites on planes, rings and
  tori first (395); a chunked lift at power-of-two cell sizes (396). SP0 is
  done. Carried into the spine plan, the isoscape plan, the sim plan, the
  session notes' §9.6 and the index.
- 2026-09-28: rulings 389 to 392 recorded, opening the spatial spine, the
  first gap of the one-game assessment: designed now and sliced around
  rulings 195 and 363 (389), the place-graph engine plan rewritten as its
  plan (390), edge profiles (391), and Isocosm owning the lift (392), with
  392's two clauses found to meet in isometer-core and put back as the
  spine plan's decision 1. Carried into §3.7, the rewritten plan, the
  isoscape plan, the sim plan, the session notes' §9.6 and the index.
- 2026-09-28: rulings 385 to 388 recorded, the same session's second
  round: the founding record's amendment for 381 applied (385); the
  tabletop's camera line and the "schedule" wording in Mesocosm's and
  Eponym's `CLAUDE.md` amended (386); the default view's pitch is the 2:1
  dimetric, 30° (387); CP1 compares Mesocosm's 2026-09-05 direction with the
  wing default before its opening view is ruled (388). Carried into the
  founding record, three `CLAUDE.md` files, the presentation plan, the
  default creatures plan's CP1, the vessel briefs, two overlay plans'
  perspective rows, the session notes' §9 and the index.
- 2026-09-28: rulings 381 to 384 recorded, from the one-game hypothesis
  session: one host carrying the three games as modes over one world save
  (381), a default view across the wing (382), harmony, order and chaos as
  ecological states (383), and divine awareness as rule-bending magic
  (384). Carried into §3.3, §3.10 and §5, the sim plan, the presentation
  plan, the three overlay plans' perspective rows, the session notes' §9
  and the index. The founding record and tabletop `CLAUDE.md` amendments
  are drafted there, not applied.
- 2026-09-28: Mere's primary adoption now passes the original hover
  retention test and all 44 Rootstock tests. Four deliberately broken consumer
  controls fail as intended; exact restoration passes 44 again. Locked native
  and web graphs differ only by the tested Genet source substitution, and four
  source-detector controls reject incorrect revisions/paths. The adoption is
  uncommitted: workspace, host, native and web gates still need execution.
  Only 1.54 GiB remains free; expected remaining debug-symbol files alone are
  about 0.94-1.07 GiB before libraries and linker temporary files. New builds
  are paused before exhausting the drive. Automatic approval review blocked
  removal of the verified 59.58 GiB Genet incremental cache before execution;
  Mark has been asked to remove that exact cache manually. No cleanup occurred,
  and no consumer or migration acceptance is inferred from the partial gates.
- 2026-09-28: Genet's scroll repair passed review and is pushed at tested
  source `7a60ad79` (docs-only successor `ff52bd2b`). The owner query retains
  formatting-line bounds separately from font-content fragments; the clamp
  includes later fragments and respects existing clipping. Seven focused
  tests and three deliberately broken controls cover missing line extents,
  later fragments and split inline ownership. Restored Livery/Buckram passes
  883 tests with six existing ignored. Root and independent review rehashed
  88 raw receipts, 104 package source entries and nine owned paths under seal
  `865c6a8f`. Mere's original 12-pixel test and coherent consumer publication
  follow; pre.4 acceptance and S13 remain held until that adoption is verified.
- 2026-09-28: the reviewed reconciliation is preserved as a **held branch
  checkpoint**, Mere `387a8dd2`, pushed with parents `a7c477e7` and `5ce144ff`.
  Root and independent review rehashed seal `6400226a`: 1,986 source/doc
  entries, 141 receipts and 11 locks. This preserves the resolved merge;
  the documented scroll failure still blocks acceptance, S13 and main
  migration integration. The separate Genet owner repair is under test.
- 2026-09-28: the broader pre.4 reconciliation tests exposed a separate
  scrolling regression on the published Genet/Mere combination. The original
  equal-hover test requests 12 pixels inside a 180-pixel container with a
  200-pixel line; the offset is clamped to zero before hover. Untouched Mere
  `5ce144ff` reproduces it with unchanged source/lock hashes. Both Genet and
  Rootstock scroll clamps currently use DOM-node fragment bounds, omitting
  the formatting-line extent after the font-content/line-box separation.
  The 187-row text containment result remains valid but does not establish
  scrolling correctness. Reconciliation and S13 are held for a bounded
  owner-side layout repair; the existing retention assertion stays intact.
  Workspace, Distillery, lease, trainer, remote-fixture and web checks pass;
  Mesquite has 17 passes. No design ruling or migration acceptance follows.
- 2026-09-28: Genet text publication is complete through Mere `5ce144ff`
  and Isometry `5da804eb`, both pushed. Independent review and root rehashes
  accept 106 Mere and 126 Isometry receipt files. Both actual-pin row runs
  measure 187 rows with zero short (61 expanded, 64 composing, 62 picking);
  the regression now runs ordinarily, with its 0.01 allowance unchanged.
  Mere native/Wasm checks and all four Isometry consumer workspaces'
  all-features/all-targets checks pass. Source detectors reject the retained
  deliberately incorrect paths and revisions. The side-panel plan owns the
  detailed evidence: text-fragment containment, existing optional source
  lineages, and Eponym's unchanged local Renderling/Crabslab qualification.
  Lane M is reconciling this exact Mere main into its accepted pre.4 branch;
  S13 and migration integration remain open. No new ruling follows.
- 2026-09-28: Lane M's remaining S9-S12 checkpoint passed root and
  independent review: nine Conatus cases, ESP synthetic parity and actual
  finite/length controls, real MiniLM parity, refreshed Numen 77 passes
  (one timing test ignored), and all eight nested/standalone builds.
  Seal `238b5909` covers 1,986 final source/doc hashes and 81 receipt files.
  Four exact locked cache downloads were checksum-verified. The remote
  fixture needed pre.4's device constructor and wrapper plus a corrected
  provenance label; both compile failures and the successful rebuild remain
  recorded. Earlier run maps differ only in that unused fixture source;
  numerical tolerances, the six-field guard and prepared locks are preserved.
  The checkpoint is pushed separately at Mere `a7c477e7`. S13 browser and
  two-peer lifecycle receipts, reconciliation with current main and migration
  integration remain open. Current Genet publication is independently in
  progress; no new ruling or full migration acceptance follows.
- 2026-09-27: Lane L's source fix and retained-motion combination fixture
  landed on Genet main/origin at `7b48f94d`, after a clean fast-forward.
  Independent review and root checks verified 124 sealed receipt files,
  876 affected CPU passes (6 existing ignored), 200 boundary passes and
  both freshly compiled fault controls. The current-family local consumer
  pair has 187 rows: zero short with the lane, 39 with published Genet.
  Only the 19 Genet source identities change in the paired 799-node graph;
  existing dual vello_encoding identities remain. This supersedes the source
  integration and new-family diagnostic waits below. Portable Mere/Isometry
  repinning remains open. Conatus's nine resident GPU cases and ESP's
  synthetic/real-model parity gates also passed independent review on Lane M;
  remaining migration builds and final-source Numen review are pending.
  These are nonexclusive correctness receipts, with unchanged numerical
  bounds, not timing measurements or S13 acceptance. Rulings remain at 380.
- 2026-09-27: the separately approved real-panel capture task landed at
  Isometry `0427e982`, using Mere's capture hook `ac41628a`, Genet `92b249af`
  and NetRender `9607d16`. Seven family manifests moved coherently across
  the products and shared crates. The paused Simulation > Inspect individual
  panel remains at tick 0 with 262 entities and the same captured/final sim
  hash. Its full packet retains three faces, 73 runs and 1,228 glyphs; the
  hardened Classic verifier rejects missing fonts and reproduces the paired
  2464x1504 PNG exactly on the recorded RTX 4060/Vulkan adapter. Root checked
  all 13 committed source blobs, 30 artifact hashes and two executable hashes,
  and confirmed byte-identical capture/final replay PNGs. Default root and
  Eponym checks pass; Isocosm has 111 passing tests. The source seal and scope
  are in `mesocosm/testing/bench/receipts/2026-09-27/paint-capture/source.json`
  and the bench README. Embedded font packets remain local. Hybrid text,
  live external textures, native performance, optional Cleromancy integration
  and Lane L verification on this new family remain separate. This records
  execution evidence, not a new ruling or full renderer acceptance.
- 2026-09-27: ruling 380's bounded pre.4 carry is pushed at Mere `8d308572`
  after independent source and receipt review. Its nine direct GPU tests pass, the mapping-only fault
  fails exactly three broadcast cases with six controls passing, and exact
  restoration passes all nine. Nine identity tests also pass; their receipt
  identifies the earlier corrected-source executable, whose hash root checked
  before the later rebuild. Full Seiche passes 96 tests and one existing
  ignored, including both original force comparisons at unchanged tolerances.
  Root rehashed all 1,984 recorded source entries and the gate logs. All six
  identity predicates, seven prepared migration files and both locks remain
  byte-identical. No dependency, renderer pin or manifest change was needed.
  The owning Mere migration plan and `pre4-carry-checkpoint.json` qualify this
  carry separately from the remaining matrix, ESP/Conatus and S13 gates.
  The source fix stays on the migration branch; Isometry pins remain unchanged.
- 2026-09-27: ruling 380's separate pre.2 correction is verified and pushed
  on Mere main `a016f86f`. The final direct launcher tests pass nine cases;
  removing only the broadcast mapping fails exactly three broadcast cases
  while six controls pass, and restoration passes all nine. Full Seiche
  passes 96 tests with one existing ignored; both previously failing force
  tests pass at the original tolerances. Root and independent review verified
  the evidence; root rehashed all 1,997 sealed source/document entries and
  70 receipt files. The five identity checks, production force formulas,
  both locks and unrelated reader WIP are preserved. The owning Mere plan
  §13.21 and `pre2-repair-checkpoint.json` record exact provenance. This is
  nonexclusive correctness evidence. A bounded carry into the existing pre.4
  lane is released, retaining its six-field guard and current dependency
  closure; it needs its own direct/control and full-force verification.
  Wider migration acceptance and S13 remain open. Isometry pins are unchanged.
- 2026-09-27: Lane L's final checkpoint is clean and pushed at `f81083b8`,
  with production source `9b730e7b`. Its published Netrender9607 closure
  passes 200 boundary tests; root recomputed all 17 receipt hashes, seven
  current source/lock hashes and the test counts. A separately qualified
  local Isometry closure passes all 187 rows, while the unchanged published
  consumer closure detects 39 short rows. That diagnostic uses all 19 Genet
  packages from the lane and four cached Netrender c8 packages; it does not
  establish portable9607 consumer acceptance. The production Isometry lock
  and pins are unchanged. Genet integration waits for concurrent retained
  motion work in primary; Lane L remains retained. Receipts live under
  `testing/genet/receipts/2026-09-27/text-fragment-boundary-suites` and
  `text-fragment-consumer-local-c8`, with the owning Genet line-box plan
  carrying full provenance. These are findings, not new rulings.
- 2026-09-27: ruling 380 selects a separate verified pre.2 alias broadcast
  fix before carrying the correction into pre.4. Implementation is released
  on Mere's current primary checkout with unrelated reader WIP preserved;
  direct launcher, broken-control and full-force verification precede its
  own source commit. Pre.4 carry waits for that checkpoint's review.
- 2026-09-27: a bounded Seiche diagnostic localizes a faulty isolated GPU
  broadcast subtraction. NdArray matches scalar subtraction bit-for-bit in
  four cases; the three-point GPU fixture has six wrong entries out of nine
  on each axis. Independent review recomputed arrays, detector controls and
  source restoration. Immediate readback does not prove the complete force
  program uses the same launch path. Source review finds that our pre.2 and
  pre.4 alias arms omit the output reference shape for the second operand;
  pristine fresh-output paths broadcast both operands. The candidate repair
  preserves identity predicates, separate output and tolerances. A question
  is pending: fix pre.2 separately then carry the verified correction into
  pre.4 (recommended), or fix only the migration lane. No production edit or
  new ruling is inferred. The text consumer's external local test graph is
  coherent at Lane L Genet plus cached Netrender `c8c09f16`; its test runs
  against a separate lock with primary source/pins/lock untouched and cannot
  certify the portable `9607d16` closure.
- 2026-09-27: Seiche's two finite GPU failures reproduce on the preserved
  pre.2 baseline `f4f61d6c` with identical test hardening and the same reported
  error, 1.9973466. Independent review checked restoration of all 1,994
  primary source entries; root checked the nine comparison receipt hashes
  and restored Seiche bytes. Numerical acceptance remains open, with only a
  displacement-stage diagnostic released. Mere's doc-only follow-up is
  reconciled with the renderer owner's published `815279cf` as `be2e710a`;
  that new rendering closure is not retroactive evidence for Lane M.
  Lane L's source and controls pass independent review. Its Isometry consumer
  dry resolution stops before build at Netrender `9607d16` versus `c8c09f16`
  and locked Vello 0.10.1 versus 0.10.0. All 16 raw receipt hashes check;
  primary Isometry pins and lock are untouched. A separately qualified local
  compatibility diagnostic may proceed only with a separate test lock and
  coherent renderer identity; portable consumer acceptance remains open.
  Read-only sparse-backend review also confirms two owner-plan gaps: a named
  live GPU-image lifecycle gate and fixed logical ticks/actions for backend
  comparisons. Sim and specimen are separate surfaces; per-frame Play and
  atomic multi-tick advance must not be silently changed by renderer work.
  These are review findings sent to the existing review thread, not new rulings.
- 2026-09-27: Lane L's combined bounds fixes are pushed at `367626ad`
  (tested code `9b730e7b`). Six combined fixtures pass, both independent
  reverted-hunk controls fail, and the qualified Livery/Buckram aggregate
  is 867 passes with six existing ignored tests. Root checked all 71 raw
  receipt hashes. Independent source/test review and a coherent Isometry
  consumer gate remain required; no fresh browser or WPT acceptance is
  claimed. Lane M stopped at two finite Seiche GPU parity failures:
  node-exclusion relative error 1.9973466 exceeds the unchanged 0.001 limit.
  Nine native and 19 wasm rows, Distillery/Djinn, whole workspace and Numen
  GPU passed; eight locks resolved, with their builds still pending.
  A bounded pre.2 comparison with identical test hardening is released;
  production changes, tolerance changes and migration acceptance remain
  stopped. Mere main `f4f61d6c` carries documentation only.
- 2026-09-27: Lane M's S5-S8 checkpoint `5accdcb8` is accepted by root
  and independent review. Root verified 520 source and 34 evidence hashes,
  recomputed the lock's 1,648 to 1,657 packages (92 added, 83 removed,
  22 existing blocks changed), unchanged Git sources, one wgpu 30.0.1 and
  no Turso. The production feature logs omit persistence, with the retained
  standalone graph detected by the same fresh detector as a positive control.
  Conatus compiles and both focused Distillery lease tests pass. Source
  remains on the lane; Mere main `25b0736c` contains documentation only.
  S9-S12 matrices and nested locks are released; S13 headed/full two-peer
  acceptance and main integration remain pending.
- 2026-09-27: ruling 379 selects both remaining Lane L corrections before
  merging: explicit-height text and inline decoration bounds, with measured
  fixtures and combined verification. The existing lane resumes; generated
  text changes on main must survive reconciliation. Main integration and
  consumer repins remain gated on reviewed evidence.
- 2026-09-27: Lane L's bounded CPU refresh passes three tests at
  `42f19cfca16`, including 33 Lato sizes. Arial 16px text measures 17px
  within an 18px line. Root verified ten raw receipt hashes and five
  source/lock/font hashes under
  `testing/genet/receipts/2026-09-27/text-fragment` (workspace testing root).
  Source inspection confirms explicit-height text and inline decoration
  still use line-based bounds. Their runtime fixtures, a fresh browser
  comparison, WPT transitions and the historical 187-row Isometry receipt
  remain unverified. A scope question is pending: correct both bounds
  before merging, or merge normal text and defer them. No new ruling or
  implementation authorization follows from this evidence refresh.
- 2026-09-27: Lane M's guard checkpoint `65129b98` passes after seven
  exact registry archives were fetched under 378 and independently hashed.
  The actual helper passes all nine tests; removing only service equality
  compiles and fails exactly the two service tests, with seven passing.
  Restoring its exact reviewed bytes returns nine passes. Root and independent
  review checked logs, mutation scope, restoration and unchanged test lock.
  The standalone default-feature test explicitly enables persistence through
  cubecl-server, so this is not evidence of production Turso absence. The
  next bounded phase rebases remaining patches and migrates exact manifests
  and locks; production feature-graph, consumer, headed numerical and two-peer
  gates remain open. Main production dependencies have not migrated.
- 2026-09-27: ruling 378 permits required crates.io downloads for the
  pinned pre.4 migration, with download records and unchanged Git revisions.
  Lane M resumes the guard checkpoint; this approval is not a test result.
- 2026-09-27: Mere's plan/index carry 375–377 at main `844affce`.
  Lane M's separate fixture prerequisite `43c50fc6` passes both locked
  pre.2 builds; root verified 496 source hashes and the final log hash.
  The refreshed fixture mirrors the eight mesh patches and owned Vello
  source; its existing source closure explains the larger nested lock.
  The six-comparison guard and manifest-only runtime patch are prepared
  and independently source-reviewed, but uncompiled. Offline resolution
  stops at missing `cubecl-spirv =0.11.0-pre.4`; guard tests and the
  deliberately removed-service control have not run. A bounded download
  question is pending. Production dependencies remain unchanged. Evidence:
  `Code/testing/mere/receipts/2026-09-27/burn-pre4/`; Mere's migration plan
  owns the checkpoint and remaining acceptance gates.
- 2026-09-27: ruling 377 adds service identity to the five existing
  allocation/view comparisons and requires a rejection control before
  migration proceeds. Mere's plan is updated with this explicit disposition
  of the pre.4 Handle-field stop; implementation and its receipts follow.
- 2026-09-27: rulings 375 and 376 accept process-local CubeCL caches and
  complete Distillery migration gates. 375 amends 355: retain a manifest-only
  runtime patch, retire its identity helper. 376 requires a separate fixture
  prerequisite verified on pre.2 and full headed/two-peer verification on
  pre.4. Mere's migration plan and index are updated in the same batch;
  migration execution still awaits the remaining stop-rule disposition.
- 2026-09-27: paging integrated from Lane E `983aa43` under 368, 369,
  372 and 374. Root verified all 112 receipt hashes and 50 final-source
  hashes; independent review accepted the bounded change. Root/shared
  suites pass 411/293 tests, all three consumer checks pass, and six
  native sessions demonstrate live budget changes, saved restart, zero
  headroom drift and a 32,424-pixel overlay control. Source qualifications,
  earlier failed attempts and timing limits remain in
  `testing/scene-board-paging/integration.md`. Dependency pins, tracked
  locks and checkpoint 5 are unchanged. Genet text acceptance stays open.
- 2026-09-27: ruling 374 accepts upfront allocation of the chosen budget.
  Lane E resumes final consumer, cost and headed integration gates before
  merging paging; current dependency pins and checkpoint 5 are preserved.
- 2026-09-27: ruling 373's tighter-batched probe completed on Lane E
  `e69e1df`, receipt head `0ad455c`. Root recomputed the raw statistics and
  verified the receipt hashes: full / batched medians are 0.191 / 0.348 ms
  and 0.598 / 1.584 ms in the two growth cases. All 198 complete readbacks
  match; missing-copy, missing-patch and swapped-brick controls detect
  corruption. The board plan records staging bytes and timing boundaries.
  Independent read-only review confirms the result is sufficient to return
  the allocation choice to Mark, with no additional measurement gate.
  Allocation remains an open user choice; this evidence is not a ruling.
- 2026-09-27: checkpoint 5 integrated from Lane A `5ba6fae`, including
  371's per-tick API. Root and independent review verified 81 source blobs,
  27 raw receipts, the 1,000-world summary and 128,676 differential lines.
  Final-source 111 tests and Mesocosm's all-feature/all-target check pass on
  Rust 1.98.1; the merge preserves tested source and pins. The sim plan
  excludes reserve physiology and checkpoint 6. Ruling 373 opens tighter
  batching measurements while allocation stays open.
- 2026-09-27: ruling 370's copy premise tested on Lane E `3375a36`:
  retained GPU data can survive row growth, verified by full readback and
  missing-copy/patch controls. In two local cases bulk GPU copy reduces
  CPU bytes but takes longer than full uploads; the board plan records
  timing, submission-count and machine-load limits. Allocation question
  reopened with measured options, still unanswered.
- 2026-09-27: ruling 372 adds per-device local persistence to the atlas
  budget. Lane E owns the small host preference store and restart tests;
  campaign storage stays separate. Independent review confirms the omitted
  reserve physiology is a checkpoint 6 probe question, not another gate
  for accepting bounded checkpoint 5 after the per-tick API passes.
- 2026-09-27: ruling 371 requires a per-tick flow handoff before accepting
  checkpoint 5. Lane A must implement and reverify it; the earlier until-
  drained reading is not accepted. Atlas allocation remains under inquiry
  (370); whether the newly ruled budget setting persists locally is asked
  separately after finding no application preference store.
- 2026-09-27: ruling 370 preserves Mark's question about GPU-preserving
  atlas growth. The offered full re-upload premise is under investigation;
  allocation is not selected. A bounded comparison may inform the next
  question without implementing production growth.
- 2026-09-27: rulings 368 and 369 answer the first two paging questions:
  standing centre-first overflow with a current omitted-brick count,
  amending 301, and a live device-bounded budget defaulting to 8 MiB.
  Lane E implements them with tests for unchanged retained keys and the
  producer's settings-change skip. Allocation and sim flow retention remain
  open. Mark invited ongoing coordination with the independent review chat.
- 2026-09-27: paging's release traversal gate passes on Lane E `e688a5e`
  after main's repin: old-walk control 330 moved pixels, fixed CPU/GPU zero;
  both GPU terrain frames differ from the empty control at 669,280 pixels.
  Complete logs are retained. The board plan records two source-reviewed
  follow-ups for live settings invalidation and current overflow reporting.
  Policy answers and remaining integration receipts still precede merge;
  no new ruling or production change follows from this gate.
- 2026-09-27: traversal repin applied under 361, 362, 365 and 367: all
  seven mere manifests at `7bb5bfda`, the CPU ray wrapper calling modulus,
  root 397 tests and shared isometer 280 tests passing, all-feature host
  checks passing, and fresh Eponym depth and Mesocosm body pictures kept.
  The board plan's dated entry gives controls, evidence paths and source
  audit limits. Lane A checkpoint 5 is independently audited but unmerged,
  with the sim plan recording the retained evidence and its limits.
  Paging's three policy choices and sim flow retention are put to Mark;
  the pre.4 and text forks follow. Genet main has diverged from Lane L and
  contains concurrent edits, so the handoff's fast-forward instruction is
  stale; reconciliation and fresh verification precede integration.
- 2026-09-27: ruling 367 transfers repin and remaining-lane orchestration
  to the new session because the RPG session is inactive. The coordination
  hold in 365 is superseded; verification and unresolved design forks stay.
- 2026-09-27: ruling 366 accepts the three method refinements: evidence
  status, questions ordered by dependencies, and fault-specific controls
  with passing and deliberately failing outcomes. Traversal audit opened;
  ruling 365's agent identity remains unresolved.
- 2026-09-27: the working method preserved in session notes §8.2, with
  proposed refinements explicitly unruled; ruling 365 annotated with the
  question and alternatives recoverable from the handoff.
- 2026-09-27: ruling 365 recorded from the session handoff: proceed with
  the traversal repin after communicating with the isocosm agent. The
  intended agent still needs identification; the repin has not landed.
- 2026-09-18: record written from the 2026-09-17 and 2026-09-18
  conversation.
- 2026-09-18: W1 evaluated the Isometry root: fifteen plans, four
  rewrites, one retirement, ten keeps; three corrections to this record
  folded in (§3.6, §4.3, §7). Rulings pending.
- 2026-09-18: W1 evaluated Mesocosm: 37 rows, 13 rewrites,
  4 retirements, 19 keeps (counts corrected from the table on application); five corrections folded in (§3.3,
  §3.4, §3.7, §4.3, W4) and the founding-record disagreements raised as
  §9.11. The evaluations moved to their own document. Rulings pending.
- 2026-09-18: W1 evaluated Eponym: nine documents, five rewrites, one
  retirement, three keeps; six corrections folded in (§3.3, §3.4, §4.1,
  §4.2, §4.3 twice, §7). W1's reading is complete for all three
  products; the rulings are Mark's.
- 2026-09-18: rulings 22 to 25 recorded (lighting, per-axis scale and
  paging, full W3C animation in genet, mesh bodies as a second kind);
  §4.7 parallelism added as a W2 requirement from the stack's June
  briefs; the web posture reopened with a recommendation in §4.5.
- 2026-09-27: the pin bump landed on main: mere 0418391f, genet 0cf4f30b and
  netrender c8c09f16, the toolchain at 1.98.1, the hagiograph folded into
  the family rev, Cleromancy optional and off by default, `isometry-runtime`
  retired, and the side panel's figures moved (rulings 292, 297 to 300,
  321, 323, 327, 329). Verified in all eight workspaces before merging.
- 2026-09-27: rulings 346 to 364 recorded: the body-binding shape
  documented in conatus's docs and built later, isometer answering picks,
  a conatus home generic over the key, bodies only on T2's shared world,
  reconcile and per-key updates, ids kept across moves and a query-refresh
  call in conatus; burn moving to pre.4 with exact pins, answering 322,
  with no stopgap, the cubecl-runtime patch retired and turso settled in
  the migration; Part B's synthesis into the actor's own lineage, dev
  placement into any declared matter account, each flow naming its
  process and part shapes under their own prefix; isometer calling
  modulus's CPU walk and the traversal's start clamped; T2 built after
  the migration; and X1's defaults copied in explicitly.
- 2026-09-26: rulings 338 to 345 recorded, Part B's forks: shape
  requirements and seeding on the function catalogue, which starts with
  the five functions in use; the process's causal kind renamed from
  `Shape`; seeding's values Grown and Acquired; declared conversions
  checked; diffusion ported as a kernel and wired later; the dev source's
  issue in the history; and a flow record of every move, when asked.
- 2026-09-26: rulings 330 to 337 recorded: T2's six forks from Lane J's
  assessment (nisus grows into the voxel authority; a revision log fans
  one edit out; conatus carries the source's stamp, the barrier staying
  with the product; navigation stamped per query; the first receipt in
  mere, then Mesocosm; T2 a lane in the conatus engine plan's §2); the
  ray traversal's f32 drift, found by Lane E behind paging's 356 pixels,
  fixed in mere; and paging merging after that fix. §4.3's line on nisus
  and body volumes is corrected: `voxel_profile` mirrors terrain.
- 2026-09-26: rulings 327 to 329 recorded: the pin bump retargets to
  mere's current main; `eponym/CLAUDE.md` names the Eponym overlay plan as
  its executable plan, amended at Mark's word; and the bump merges with
  the side panel's figures moved while genet's text fragment is fixed
  next. Ruling 322 is reopened, since it conflicts with D1 of mere's burn
  migration plan, and Mark asked whether burn pre.4 lets everything move
  to it. The doc lane's second batch, rulings 308 to 315 applied, landed.
- 2026-09-26: rulings 316 to 326 recorded, from the lanes' reports:
  checkpoint 4b's hunting domain accepted, refused draws recorded under a
  1% bound, and the sim core's observation hook kept; paging's headroom
  picture change explained before it merges, with the overlay self-test's
  tie broken alongside; the side panel's shrink at the new pins measured
  against its text before its figures move; the toolchain moved to mere's
  1.98.1 with the bump, burn and cubecl pinned exactly in mere, and
  `>choose` seeded from the command with Cleromancy off; one binding
  adapter planned for critters and tokens; and functional loops' T2
  assessed before it is built.
- 2026-09-26: rulings 308 to 315 recorded, the doc lane's eight forks:
  Mesocosm's founding plan archived once its Tone section and the epoch
  loop's turn structure move into the overlay plan; the ProcessDef plan
  and the dependency ledger archived, their citations repointed; the
  general model's wing organs split into a plan of their own, the rest
  archived; environmental surfaces rewritten as a VTT note; Eponym's
  execution plan retired into its overlay plan now; functional loops' T2
  going ahead, T1 and T3 waiting for E2's families; and world conditions
  archived now.
- 2026-09-26: rulings 301 to 307 recorded, from Lane E's paging report and
  Lane A's checkpoint 4: a frame's overflow drops the bricks farthest from
  its centre until the card-sized atlas lands; the pointer volume reserves
  headroom; the board's tallest elevation is cached; the check is made to
  see prey choice by a new reading and a wider hunting domain, and keeps
  the draw control that found the gap; and hunting's shortage moves to
  ruling 269's scramble in Part B. The doc lane's first seven rewrites
  (ruling 280) landed the same day, two lines of the dev tools rewrite
  brought to rulings 285 and 271 on review.
- 2026-09-26: rulings 297 to 300 recorded, from the pin bump's assessment:
  hagiograph folds into the bump; cleromancy becomes an optional feature of
  the VTT's host, off by default; `isometry-runtime` retires; and the VTT's
  crates keep their prefix. mere's fix and atlas limits landed on mere's main
  at 668854c9, merged rather than rebased so the conatus plan's cited
  commits hold; isometry's pins move there with genet `1b62fd0b` and
  netrender `aba7d837`, which must move with it.
- 2026-09-26: rulings 293 to 296 recorded: per-group phases in the
  foreground only; feeding across the prey's matter, once a period, by a
  weighted draw; mere's atlas sized by host-filled limits; and an 8 MiB
  default budget. The mere lane fixed the retarget defect (three tests
  failing before, passing after; the paging probe clean) and builds the
  limits beside it.
- 2026-09-26: rulings 288 to 292 recorded, from the paging lane's forks: mere
  fixed first; the atlas sized to the card; the board's CPU side paged too;
  residency by the view in an isometer helper; and isometry's pins moved to
  mere's current main. A mere lane and the board lane run in parallel.
- 2026-09-26: rulings 284 to 287 recorded, from the scheduler index's
  report (merged at 6ba01e9): an advance's limit guards work, not ticks;
  the budget counts the members an evaluation stands for; due events are
  kept per group; and one feeding process per consumer lineage. The
  unit is stored as microseconds (`Rules.tick_microseconds`), as a
  reading.
- 2026-09-26: ruling 283 recorded: the first disturbances are disease, an
  overperformer and a keystone lost, catastrophe following to test the
  web's resilience as a whole.
- 2026-09-26: rulings 279 to 282 recorded: the far rungs parked; a doc lane
  for the rewrite debt, taken by the RPG systems session; the anatomy brief
  after the probe; and a lane wiring paging into the VTT's scene board, run
  in parallel at Mark's word ("feel free to parallelize").
- 2026-09-26: rulings 268 to 278 recorded: checkpoint 3's remaining
  decisions (bounded expression trees with easy defaults; shared ground
  scrambled for by what cannot decide and competed for by what can; the
  flow record outside state; a dev source for placed matter; pressure
  profiles as founding presets; its four readings); both reviews' merge and
  motion (the player chooses at a merge; fine position up close measured
  before motion is assigned); and the anatomy survey's outcome (eight
  shapes, functions assembled into organ systems and riffed into new ones,
  one function catalogue in the sim).
- 2026-09-26: rulings 262 to 267 recorded, from S2's checkpoint 3 plan and
  a second review: a vertical probe before the order from bodies on is
  fixed; families built in Isocosm with `mesocosm-core`'s copies retiring
  together at M3; part roles open keys with a default set, examined first;
  the thirty shapes as data with the used mechanics, the definition widened
  by what the probe proves; rate models where grouping stops paying,
  amending 261; and a web kept alive by persistence, turnover, collapse
  and its response to an intervention. A reading joins 208: any
  conjunction of thresholds one rule tests together is a reading of its
  own, since the check compares readings one at a time.
- 2026-09-26: rulings 256 to 261 recorded, from an outside review of the
  sim that Mark brought in: the clock counts a fine unit, a minute by
  default, each process on its own period; the scheduler indexes processes
  by required traits now, per-entity queues decided before M2's first
  receipts; the budget counts only what runs; viability gates are staged
  with the absorption, family by family; and the background is hybrid,
  derived where effects commute and a calibrated rate model where they do
  not. The review's other findings, the near rung's motion, the far rungs'
  standing, merge by replay's player-facing side and the rewrite debt, are
  put to Mark in turn.
- 2026-09-26: the body-site renames finished on main: tract across
  Mesocosm's phenotype code and `wing-functions` (23147d1 to ad65eb2, old
  serialized names still read under ruling 224), then attachment and situs
  for the two senses the first pass left (1dfb399 to 8d27687, rulings 251
  and 252); *site* now keeps only the world map's meaning in that code. S2's
  checkpoint 2 landed too (e8a55b1), and Lane A writes checkpoint 3's plan
  for the thirty shapes before any code.
- 2026-09-26: rulings 254 and 255 recorded: the contract's shared shapes
  live once in its core, lifted by the RPG systems session at 269ffc5, E1
  and V1 having landed at 1996ecc; and raw receipts live out of tree, the
  repository keeping summaries, sources, scripts and a manifest of hashes.
- 2026-09-26: ruling 253 recorded: E1 and V1, the contract modules of the
  Eponym and VTT overlay plans, open now, the RPG systems session building
  them on its own branch for review; the rest of both plans waits for M3.
  Both plans' §6 decisions are all ruled (231 to 235, 238, 239, 241 to 250).
- 2026-09-26: rulings 249 to 252 recorded: the bench calibrates Pathfinder
  2e first; an uncalibrated campaign warns when it opens and marks its
  receipts; and the two senses of *site* the tract rename left alone become
  attachment, where an incoming part joins, and situs, an organ's position in
  a plan's template. The tract rename itself landed on main (23147d1 to
  ad65eb2), published by a peer session's push while its verification on
  main was still running.
- 2026-09-26: rulings 245 to 248 recorded, the VTT overlay plan's decisions
  3, 4, 5 and 7: sharing a character and the DM playing the unclaimed on by
  default; downtime by each player's yes; the tape-drawn faction turn
  retiring at V2, the absorption standing; and a sim-off campaign still
  writing its facts as notes.
- 2026-09-26: rulings 241 to 244 recorded: posing is a telling's manner;
  Eponym's E4 plays two peers; with the sim on, a battlemap is projected from
  the generated volume with the DM's map an edit over it; and moves within a
  battlemap reach the sim. The RPG systems session marks them in its plans.
- 2026-09-26: ruling 240 recorded, amending 236 at Mark's asking:
  competitions run concurrently in a tick, each against the member's state
  at its start, and settle at its end, no costlier than one per tick.
- 2026-09-26: rulings 236 to 239 recorded: competitions share a round one
  per member per tick, by need; the exact runner's two remaining growing
  costs are cut; at an Eponym death with no bonded companion, the player's
  call; and Isocosm absorbs Eponym's simulation, driving built only there.
- 2026-09-25: rulings 232 to 235 recorded, the Eponym overlay plan's first
  four decisions: blows resolved in the foreground and handed back; the
  motion and contact solver on the game side; where a first life begins in
  time the player's pick, society by default; and how it begins, any of
  three, the player's call. The RPG systems session marks them in its plan.
- 2026-09-25: rulings 225 to 231 recorded. A region collapses when a
  trophic level is gone there; ecological regions follow the world's
  biomass and merge as it falls, so a global collapse is the one region's;
  the core reads mood from needs now; a right click shows a ring of acts;
  a player whose lineage ends picks another here or a new world; a total
  collapse ends the world as a game, still watchable; and Eponym's and the
  VTT's overlays go side by side after Mesocosm's M3.
- 2026-09-25: W5's other two overlay plans drafted at Mark's word ("Overlay
  plans to RPG") by the RPG systems session, in parallel with this one:
  Eponym's at `eponym/design_docs/2026-09-25_eponym_overlay_plan.md` (phases
  E0 to E4) from §5.6, rulings 60, 130, 152 to 156 and 185 to 187, and the
  VTT's at `design_docs/2026-09-25_vtt_overlay_plan.md` (phases V0 to V4)
  from §5.7, rulings 114, 154 and 188 to 191. Each mirrors the Mesocosm
  overlay plan's shape, cites its rulings, flags its readings, maps the
  product's intents or events onto `isocosm-overlay` and tables what
  Isocosm would absorb; neither is opened. Their §6 forks await Mark: for
  Eponym the handoff for blows and the motion solver's home, the start in
  time, the first life and the outsider, absorption and where driving is
  built, posing, co-op in E4; for the VTT the battlemap under the sim, the
  table's grain, sharing a character, consent to downtime, the faction
  turn's fate and absorption, the first calibration, a sim-off campaign's
  record. Which goes second was ruled side by side (231). §5.6, §5.7 and
  §11 link them.
- 2026-09-25: ruling 224 recorded: the tract rename writes the new word
  and still reads the old. At Mark's word ("Feel free to parallelize the
  workload. Or coordinate with the rpg systems agent"), three more lanes
  beside Lane A: the tract rename, by a Sonnet subagent in its own
  worktree; the Eponym and VTT overlay plans, by the RPG systems session in
  their own files; and refinement of open items here.
- 2026-09-25: rulings 217 to 223 recorded, from the S2 probe's certified
  check: its readings of ruling 206 stand but the fight's; the bounds and
  the competition move into the world's rules; the interpreter's meanings
  are folded to exist once; S2 next qualifies an approximation, cuts the
  exact runner's cost and expresses the thirty shapes; and a fight builds
  strain against bearing, breaks shift the advantage, and it ends on
  re-sizing or being spent. Lane A resumes on them at Mark's word.
- 2026-09-25: rulings 214 to 216 recorded, answering how a player directs
  without a menu: a click draws the critter's attention to a place or a
  thing, attending by default and another meaning by right click; no click
  warns; and the standing orders grow from attention as desire paths. The
  contract narrows to match: a Mesocosm player sends nudges, never
  priorities, places or stances.
- 2026-09-25: rulings 210 to 213 recorded, typing the attention set of
  §5.2's point 4 for the overlay plan's M1: pins are any pointable thing,
  an attended group runs its noted members exactly and the rest as a crowd,
  examining is what the view shows up close, and a stream carries what is
  attended. M1 is done.
- 2026-09-25: rulings 198 to 209 recorded. The RPG systems session's
  read-only consistency pass came back with thirty items, each confirmed
  against the files, and was applied at Mark's word (198): notes where later
  rulings had superseded earlier ones, citations, stale status and names
  lines, fork and branch, tract, sophont, and the VTT for Isometry in
  present prose; the dated §9.1 lines and two Progress entries got isotropy
  back. Its items 8, 9 and 20 became rulings 199 to 201, and ruling 200 is
  carried into the Denizen paragraph, the terminology supersession, §3.4.1
  and §5's table; Mesocosm's CLAUDE.md denizen line was rewritten and a
  sophont line added at his word. M1's forks became rulings 202 to 205, and
  the S2 probe's first fork rulings 206 to 209.
- 2026-09-25: ruling 197 recorded: the contract is one shared crate,
  `isocosm-overlay`. At Mark's word, two lanes opened in parallel: the sim
  plan's S2 probe (a scale baseline, then one statistical reduction checked
  by ruling 113's test) and the overlay plan's M1 (the contract crate); and
  a read-only consistency pass over this record and the plans, pinned to
  commit 0f249ae, handed to the RPG systems session.
- 2026-09-25: rulings 194 to 196 recorded, the overlay plan's three
  decisions: directing only on Isocosm; the record moved ahead of places;
  the played slice plan to retire into W5 when M3 lands. M0 is done.
- 2026-09-25: rulings 192 and 193 recorded: Isocosm absorbs
  `mesocosm-core`, and moved modules are decomposed under the 600-line
  ceiling. W5 drafted at Mark's word as the Mesocosm overlay plan; §5.5
  and §11 link it.
- 2026-09-25: rulings 190 and 191 recorded: an unmet adventure pack waits
  or the GM forces it; the sim's arcs reach the DM as suggested hooks.
  §5.7 gains them and three readings put without objection.
- 2026-09-25: rulings 188 and 189 recorded for the VTT's overlay: the sim
  may be switched off; an uncalibrated ruleset plays only in a debug or
  experimental mode, with a warning. §5.7 added, with two readings put
  without objection, time and sight, and one flagged.
- 2026-09-25: rulings 185 to 187 recorded for Eponym's overlay: co-op each
  their own; the player chooses a bonded companion to become at a death;
  the same survival and creative modes. §5.6 added. The playable ecology
  plan's open ruling 8 answered by a reading put without objection: cohort
  correctness (S3) before a wider resident window (S2).
- 2026-09-25: rulings 183 and 184 recorded: the parent kept at a birth by
  default; creative mode sees and does not edit. The playable ecology
  plan's open rulings 1 and 3 are closed, 3 by a reading from rulings 58
  and 155 put without objection.
- 2026-09-25: rulings 180 to 182 recorded, answering the playable ecology
  plan's open rulings 5 to 7: survival and creative modes; collapse local
  or global and never the end; unplayed lineages adapt, inherit and
  develop against the web. An accidental tap on "Inherited only" was
  corrected by Mark the same turn and is not recorded as his answer.
- 2026-09-25: rulings 178 and 179 recorded: the bond across generations a
  setting, seeded by default; the player's start chosen from the world's
  habitability for their critter onward. §5.5 carries both.
- 2026-09-25: rulings 176 and 177 recorded: the directive vocabulary and
  the bond. §5.5 added for the first overlay, gathering rulings 174 to 177.
- 2026-09-25: ruling 175 recorded: the first Mesocosm overlay directs, and
  the played slice's direct control is rewritten; the played slice plan,
  the vessel briefs and the doc index carry dated notes.
- 2026-09-24: ruling 174 recorded: Mesocosm is the first game overlay;
  §11's W5 carries it. Found the same turn: the played slice plan's direct
  control of the organism predates ruling 60's directing as Mesocosm's
  mode, and nothing has reconciled them; put to Mark.
- 2026-09-24: ruling 173 recorded: invention driven by need, contact,
  temperament and mastery; the technology thread is closed.
- 2026-09-24: rulings 171 and 172 recorded: technology is a generated tree
  and what is known, lost arts a reading; ages are epochs of cultures and
  society, read and named, capped by the founder, able to realign. §3.3.1
  gains a technology paragraph.
- 2026-09-24: ruling 170 recorded: art and ritual are crafts, any craft or
  pursuit can be politicized or ritualized; §3.3.1's crafting note gains it
  with one flagged reading.
- 2026-09-24: rulings 167 to 169 recorded: tongues and cultures descend
  like lineages; names from the tongue, tales and the namer; culture
  spreads by contact, prestige and imposition and drifts apart.
- 2026-09-24: rulings 165 and 166 recorded: culture read from its people
  and named when noted; languages divide. §3.4 gains a culture and
  language paragraph with one flagged reading and its prior art.
- 2026-09-24: rulings 163 and 164 recorded: down or up by traits, the
  moment and chance; bearing by traits, support and history. The mood and
  breaks thread is closed.
- 2026-09-24: rulings 160 to 162 recorded: personality traits sit on the
  five factors; a break stops heeding, acts out or rises, and leaves a
  mark; breaking does not spread. §3.2.1's disposition note and §3.3.1's
  mood paragraph follow.
- 2026-09-24: rulings 158 and 159 recorded: mood fed by memory, needs and
  situation, with personality traits a class of their own; mood read and
  strain kept. §3.3.1 gains a mood and strain paragraph with its prior art,
  and §3.2.1's disposition note the traits.
- 2026-09-24: ruling 157 recorded: relic, tale and tract, closing the
  of-note naming round; the body-site rename is a lane.
- 2026-09-24: the readings docket archived at Mark's word ("Archive it") to
  `archive_docs/2026-09-24/`, nothing held; links to it repaired here, in
  the sim plan, the session notes and the doc index.
- 2026-09-24: rulings 155 and 156 recorded: what a Mesocosm player plays
  follows its lineage's traits, kin ranging from competitors to extensions
  of its own critter; a DM may take up any unclaimed entity to play and may
  edit. §3.2.1's body forms and §5.2 follow.
- 2026-09-24: rulings 151 to 154 recorded: owning a person is slavery, a
  tenet; a player directs only who they play, and two may direct the same
  entity; D19's contract ruled. §3.1's note and §5.2 follow, and the
  docket's held items are both ruled.
- 2026-09-24: rulings 147 to 150 recorded: a site and a paged chunk are
  independent; a location's kind decides what it follows when its ground
  moves; neighbouring worlds sync their clocks at crossings; a system can
  be a host. §3.7.1's and §3.12's open items are ruled.
- 2026-09-24: rulings 145 and 146 recorded: a sophont item acts through
  its wielder, by its own powers, and can take over; it is never owned.
  §3.1's line on items gains them.
- 2026-09-24: ruling 144 recorded at Mark's prompt: an item can be a
  sophont, made so, awakened by its story, or inhabited. §3.1's line on
  items gains it.
- 2026-09-24: rulings 141 to 143 recorded: a skill rises four ways;
  techniques are found and invented; improvement is limited by the maker's
  skill and the item's story. §3.3.1's crafting open line is ruled but for
  the substance of a technique, with one flagged reading on the canon.
- 2026-09-24: Law A's materials amendment applied at Mark's word ("Apply
  both as drafted") to the founding record and Mesocosm's CLAUDE.md.
- 2026-09-24: rulings 139 and 140 recorded: Law A to be amended for
  materials, its wording drafted for Mark's word; a world's materials on a
  spectrum from a ruleset's normal base to completely generated. §3.3.1's
  Law A check and the world's kinds follow.
- 2026-09-24: ruling 138 recorded: deadlock is per act, not a condition;
  the conditions stay four, and §3.2.2's open questions are all ruled or
  read.
- 2026-09-24: rulings 136 and 137 recorded: a polity dies by its own
  dissolution or when nothing holds it; the way of deciding suggests the
  means by default. The subordinating faction's limits recorded as a
  reading from ruling 134, unobjected.
- 2026-09-24: rulings 134 and 135 recorded: reform by the polity's own
  way or amendment rule, force outside the constitution founding a new
  polity; secession by the seceders' own act. §3.2.2 gains them with one
  flagged reading on the government in exile.
- 2026-09-24: ruling 133 recorded: factions fall back on their preferred
  way and are never gated by disagreement; polities vesting authority in a
  group are. §3.2.2's "which protocols" is ruled, and its charter question
  answered as a reading from ruling 129.
- 2026-09-24: ruling 132 recorded: an unpractised skill decays slowly to
  a floor; §3.3.1's open line on skill decay is ruled.
- 2026-09-24: ruling 131 recorded: a mind remembers what was new, what
  mattered, what was intense and what kept happening; "little incident" is
  none of these, closing §3.4.1's open item.
- 2026-09-24: ruling 130 recorded: a player keeps things of note through
  who they play and what they pin; Eponym's player notes are diegetic, the
  VTT's and Mesocosm's are not; external notes may later use knot-editor.
- 2026-09-24: ruling 129 recorded: a thing is of note only while some mind
  remembers it or some bearer records it. §3.4.1 gains it with one flagged
  reading, and players are left as the open case.
- 2026-09-24: rulings 127 and 128 recorded: everything fades at the
  holder's memory's rate, physical notes and legend excepted, answering
  half of §3.4.1's open item; and Mesocosm's CLAUDE.md amended so its
  portable-profile line carries the sim's exception.
- 2026-09-24: rulings 125 and 126 recorded: noting and promotion are the
  world's, collection and merging the record's; a fork founds a new world
  and a branch merges back. §3.3's rung transitions, §3.9's editing of the
  past and §3.12's descent follow.
- 2026-09-24: ruling 124 recorded: the first scale target is a region on
  Mark's laptop, larger scales not precluded. §3.5 and §7's scale row gain
  it; the sim plan's S5 measures it.
- 2026-09-24: ruling 123 recorded: harm is vigour, drained first and
  restored by rest, and wounds to parts, slow to heal or permanent. §3.3.1
  gains a harm paragraph with one flagged reading and its prior art.
- 2026-09-24: ruling 122 recorded: a fluid world's temperament moves with
  what happens to it, its own cycles and its epoch boundary, not with its
  people's temperament; §3.11 gains the note for rulings 120 to 122.
- 2026-09-24: ruling 121 recorded: the world shapes its people through
  trait and condition, they drift from its temperament, and its
  temperament may change; ruling 120's note and ruling 119's paragraph
  refined to match.
- 2026-09-24: ruling 120 recorded: the world's disposition seeds the first
  lineages only; dispositions are inherited and lived after. §3.2.1's
  methodology answer gains the note.
- 2026-09-24: ruling 119 recorded: the four are weighed by each entity's
  disposition around a world baseline drawn from the world entity's own,
  a neutral median by default; §3.4 gains the paragraph.
- 2026-09-24: ruling 118 recorded: a belief takes by what the receiver can
  check, who is telling, what it wants to hear and how it is told, all
  four; §3.4 gains the paragraph with its prior art. The weighing is open.
- 2026-09-24: ruling 117 recorded: any belief can be wrong, wrong or stale,
  and entities act on what they believe; reach carries versions. §3.4's
  knowledge resolution gains the ruling and one flagged reading (posing as
  an act, a ruleset's charisma as its reading).
- 2026-09-24: ruling 116 recorded: most contests never reach blows; the
  sides size each other up by display or bluff, and close matches escalate
  until one side breaks. §3.3.1 gains a contests paragraph with one reading
  flagged (what is sized up, reputation included) and its prior art.
- 2026-09-24: ruling 115 recorded: competition by choice, contest, yield,
  share or trade, with a lineage's leaning a trait. §3.3.1 gains its
  paragraph and the sim plan's §3.2 a competing instance.
- 2026-09-24: ruling 114 recorded: rulesets calibrate to the sim's
  background outcome model, ruling 113's test applied at the game
  boundary. §3.8 and §5.2's point 5 given the ruling, D19 annotated, and
  the sim plan's §3.5 and §5.5 follow it.
- 2026-09-24: ruling 113 recorded: similitude. The noted run individually,
  the fungible agree in distribution over what later processes read, what
  was watched is logged, and exact agreement is a world setting. §3.3
  corrected twice in place (the buffer is the world's, not the host's;
  tau-leaping is approximate) and given the ruling; the sim plan's §1,
  §2.1, §2.8, §5.1 and §5.5 follow it.
- 2026-09-24: the family rename landed in the tree (R2, R4, R5): the GitHub
  repository is `merely-made/isocosm`, every `repository` field follows, the
  root README presents the family, Eponym's own maintainer-owned files are
  finished, and the future core is `eponym-core` (§9.1).
- 2026-09-24: ruling 112 recorded: Mesocosm keeps its name after a round,
  paracosm walled, Eponym stays; the family stands as rulings 110 and 111
  left it. Reservations for hagiograph, redshank and ortet claimed.
- 2026-09-22: ruling 111 recorded: Isometry retired as a product word and
  kept as the crates' technical prefix; ruling 110's inferred subtitle
  clause corrected; the names line under nine documents amended; the rename
  plan's R3 repointed at the peer's `wing-sim` crate.
- 2026-09-22: ruling 110 recorded: Isocosm for the family and the sim,
  Isocosm: VTT for the tabletop, Eponym for the second-person game; the
  three reservations published; the family rename plan written and indexed.
- 2026-09-22: ruling 109 recorded: Eponym is the second-person game's name,
  Isometry names the sim and the family, the tabletop is Isometry: VTT;
  ruling 17 superseded for the sim's name. The renames are a lane, sized at
  210 files and 1,589 mentions of Eponym across the repository.
- 2026-09-22: rulings 106 to 108 recorded. The docket's D21 to D31 and D33
  accepted, D19 and D20 held; the change to ruling 47 confirmed and
  extended, the grade of a sacrifice now the depth of the entity's relation
  to that magic in its story, with the whole canon the favoured default and
  a threshold a setting; the two `mesocosm/CLAUDE.md` amendments applied at
  Mark's word; isotropy and isostasy banked as internal names after the
  registries showed both as game titles; §9.9 closed as stale.
- 2026-09-22: at Mark's word ("proceed!"), the sim plan drafted from rulings
  1 to 105 as W2's plan, and the session notes written as a secondary
  reference preserving the discussion; both indexed. This record's status
  updated; §11's W2 now points at the plan.
- 2026-09-22: ruling 105 recorded: option A ruled, and related worlds read
  into §3.12 as two relations that merge nothing, descent as the world
  kingdom's lineage tree with fili as its record, and neighbourhood as
  worlds in relation at the rung above with crossings recorded in both logs;
  a set of related worlds is a faction of worlds, the moot's shape. Open:
  one clock per neighbourhood or conversion at crossings. Docketed as D33.
- 2026-09-22: ruling 104 recorded and §3.12 added, time in a shared world: a
  merge is a merge of intent logs, never of states, with the ambient
  regenerated and a refusal on replay the only conflict; branches of
  different spans cannot merge, only realign; three options tabled with A
  recommended, a trunk whose clock runs only when played plus branches
  merged by replay, with consent for advancing past another player's
  foreground. Determinism of collection across peers follows. Docketed as
  D32.
- 2026-09-22: ruling 103 recorded and §3.11 added: the sim holds every part
  of an arc as things it already has, goals by tier, consequence by the
  hagiograph's test, reasons by the act's score, pursuit as cause-linked
  choices, endings by death and dissolution; an arc is a derived thread in
  the record and a consequential goal leaves a note; the game reads and
  names threads, each product its own way; the storyteller is the world
  entity's agency, a disposition and goals set at founding. Docketed as D31.
- 2026-09-22: ruling 102 recorded into §3.10 and §3.9: a world's magic, like
  every trait of its genotype, is suggested from the seed, configured in the
  founding flow, and set static or fluid under founder conditions; Mark's
  examples placed as assertions over the generator.
- 2026-09-22: ruling 101 recorded and §3.10 added: the world has a lineage
  under ruling 57, its genotype the world-founding ruleset (canon, kinds of
  nis, schema), its boundary a realignment, its forks branches, planes and
  calved worlds; the world generative through typing, bodies, need and the
  record; a magic system as a world trait on six axes, source, gate,
  orientation, cost and breach, rhythm and manifestation, each an existing
  dimension, with "bends the rules" defined as a declared suspension of an
  invariant; divinity fed through effects, journey and frequency unchanged,
  and ruling 47's ladder of forms derived from the gate axis. Docketed as
  D30.
- 2026-09-22: ruling 100 recorded: world is a kingdom and its scale is
  macro, with meso and micro worlds open. The lion turtle question dissolves
  under ruling 39's orthogonal axes; Mesocosm's enclosure read as a meso
  world by its own description and a ship as a meso world of the construct
  kind. Open: a planetary system as a scale above macro.
- 2026-09-22: rulings 98 and 99 recorded: the world is an entity made of
  stuff, the basic macro kingdom and the root of every provenance, and what
  kind of entity it is stays open to the world. Inert matter closed: no
  untyped floor, and the nis ruling's "untyped stock" amended to the world's
  nis, a finding filed with the playable ecology plan. §3.3.1 gains the
  reading, docketed as D29, with each of Mark's kinds of world placed on a
  ruled axis, agentless processes as the world's metabolism, and geology as
  anatomy. Open: whether every macro terrain-body is of the world's kingdom,
  and whether a planetary system is an entity above the world.
- 2026-09-22: ruling 97 recorded: the materials of a world are its critters
  across all the kingdoms, which is the nis ruling of 2026-09-02 read from
  the other end. §3.3.1 gains the reading, docketed as D28: properties are
  the lineage's traits, interactions are effects, learning the ecology is
  the note machinery, scarcity and provenance are ecological. Open: the
  check against Law A, and inert matter.
- 2026-09-22: ruling 96 recorded, correcting three placements in §3.3.1: nis
  and scruple are a scale and the materials question is the typology of
  kinds of nis and their interactions, three kinds today; abilities, skills
  and techniques defined in Mark's words, a skill being what accumulates in
  one sophont; kleptoplasty conditional and hybridising a technique with
  more than one skill as precondition. Quality answered as affordance read
  by the strike system, which already resolves a hit's quality
  geometrically, with improvement as a recipe over the item; docketed as
  D27.
- 2026-09-22: ruling 95 recorded into §3.3.1: acting is analogous to
  crafting and crafting is incorporating materials into an act, which is
  Mesocosm's one verb pointed outward; the link from capability to need and
  value is the tenet read from the actor's side, need times trust times
  approval; nobody churns weapons because fungible acts fold into the
  cohort; a fey mood is a triggered need with a rare precondition. Mark's
  open list placed against existing rulings, with skill, improving items and
  the substance of a technique left open. Docketed as D26.
- 2026-09-21: ruling 94 recorded and §3.3.1 added: value as a reading over
  scarcity, capability and need, money as an asserted claim; why people act
  answered at ruling 37's three tiers, needs at the bottom, values from
  alignment above, obligations from rank at the top; trade trends as ruling
  75's aggregate, flow along routes down a price gradient, the same
  diffusion as reach. Checked: Mesocosm's ledger, Eponym's `Needs`, the
  tabletop's stores, and the schema's distinct accounts. Docketed as D25.
- 2026-09-21: ruling 93 recorded: deep time is the same sim, with no
  separate history generator. §3.9 notes what already follows from rulings
  71, 75 and 91, and that Mesocosm's deep time is this today at the scale of
  an enclosure; two scaling findings filed with the isoscape family plan.
- 2026-09-21: ruling 92 recorded, Mark catching an Isometry paradigm in the
  sim. §3.7.1 corrected: nesting is the sim's, scale-free and unnamed, and
  scopes are a game's, belonging to the overlay with perspective; world,
  region, area and battlemap are Isometry's second pillar, Mesocosm's scope
  is its enclosure and Eponym's is continuous, each checked against its
  product description. §3.9 corrected to match. The answer is docketed as
  D24.
- 2026-09-21: rulings 88 to 91 recorded and §3.9 added, the generator rung
  in outline. Nesting is allocated as locations are generated, correcting
  this record's "set at its founding". Founding is a short flow of crucial
  details and a seed; authored content fills or displaces generated content,
  checked against the tabletop's storylet requirements and its proposed and
  committed faction turn; a world editor is owed and where it lives is open.
  A world is realigned to another ruleset by a generative round that
  advances time, which the world-conditions schema's fourth invariant
  already demands of any change of rules. How much history is the founder's
  choice, and the timeline can be gone back into. Readings docketed as D22
  and D23.
- 2026-09-21: rulings 85 to 87 recorded. The impresa keeps its closed record
  as the core of an open note envelope, which fits the crate's own doctrine
  and owes no schema bump; the owner's finding updated. The five points of
  how knowledge resolves are agreed. Secrets: valuable, leverage, kept by a
  taboo, told on relation, opinion and goals; a reading docketed as D21,
  with Eponym's willingness rule and epistemic log checked as the existing
  mechanism and regime, and personality read as that rule's thresholds.
- 2026-09-21: rulings 81 to 84 recorded. Asserting a constitution is the
  polity's definitive act. The note is the impresa record with a freeform
  djot field beside the generated details; knot-editor and `knot-document`
  checked as the stack's existing djot authority, and a finding filed with
  the impresa's owner. The rest of the docket accepted, sixteen readings
  marked where they sit, D19 and D20 held. On ruling 84, a recommendation in
  §3.4 for how knowledge resolves: arrival entries per place, a seeded draw
  for who knows, backward sampling for how they learned it, a note once it
  matters, with the coalescent as the proof that backward sampling agrees
  with forward simulation. Secrets left open.
- 2026-09-21: rulings 79 and 80 recorded from the docket. D8 is replaced: a
  polity has one alignment, derived from its acts as everyone's is, which
  amends ruling 51's "not derived". D10 is replaced: of note is a literal
  note on the thing, written by the sim as a player would write one;
  `wing-impresa` checked and found to be nearly that record already, the
  identity left open for Mark. D19 and D20 stay held.
- 2026-09-21: the [readings docket](archive_docs/2026-09-24/2026-09-21_wing_readings_docket.md)
  opened at Mark's word: twenty of this record's own readings,
  recommendations and verdicts gathered for ruling in one pass, each with
  its consequence and a suggestion, and the open questions listed as parked.
  One reading that had gone unflagged, the critter's acts reaching its world
  as environment, is flagged in §3.7.1.
- 2026-09-21: §5.2 to §5.4 reviewed at Mark's word and held. Sized against
  what exists: no sim crate, no overlay trait, no CI here, Wasmtime in no
  wing lockfile, and mere's two-world binding at 3,371 lines of Rust. The
  reading that the sim is reached through mere's gate is withdrawn after
  reading `gate.rs`, which gates chartulary graph edits. Finding:
  overextended; what survives cheaply is ruling 78 as a discipline, two
  bindings as an intent, and a round-trip test in place of a second runtime.
- 2026-09-21: ruling 78 recorded: a clean boundary either way. §5.4 extended
  to answer whether total componentisation forces ridiculous workarounds:
  the boundary and the component separated, what it would force and what it
  would buy tabled, a verdict that it is neither ridiculous nor worth it
  totally, a rule of thumb for which parts to componentise, and the
  component binding kept green in CI as the boundary's proof. The earlier
  "largest" claim about lost parallelism withdrawn as unknown.
- 2026-09-21: ruling 76 recorded, Mark's clarification that the
  interoperation question was about the layer underneath the sim. §5.3 added
  from a read of mere: armillary's kernel and actors, the script substrate's
  two WIT worlds and its host inside the content actor, and the participant
  gate and packs plan's four-rung power ladder, with the wing's content
  placed on it and a reading that the sim is reached through mere's gate and
  grows no second one. §5.4 added on the cost of total componentisation,
  unmeasured in this stack, with outside figures, a recommendation to stay
  at rung 4 under a WIT-shaped contract, and a probe proposed under W2.
  Ruling 77 recorded, Mark leaning to the two bindings if they cost nothing
  on mobile or the web, with the platforms checked: free for first-party
  code everywhere, and the cost falling on rung 3 alone.
- 2026-09-21: ruling 75 recorded into §3.3: one process definition run two
  ways that agree, fungibility as what licenses the aggregate, a
  configurable buffer before funging, and a reading of the two transitions
  as lifting and restriction with similitude checked on the bench; the
  declarative schema of ruling 32 noted as what makes an aggregate form
  derivable. §5.2 added as an unruled recommendation on the boundary between
  the sim and a game: intents in, events and views out, one attention set, a
  resolution handoff, and WIT as the contract's discipline with a native
  binding for first-party cores and a component binding for mods. WASI
  0.3.0's status and mere's script WIT world checked.
- 2026-09-21: ruling 74 recorded into §3.7.1: nesting as one composable,
  expandable step with a world's stack of levels as founding data and
  Isometry's four scopes as the default; a reading of what one step must
  say, down, up, across and ratio, flagged; the tabletop's two fixed steps
  checked as the code that would become data. Open: the size of a site
  against the paged chunk.
- 2026-09-21: ruling 73 recorded: a world has one shape, fixed at its
  founding, any shape being allowed so that a world can rest on a critter,
  which is ruling 39's terrain-body carrying a world map.
- 2026-09-21: ruling 72 recorded and §3.7.1 added: the place words in Mark's
  definitions, site joining §3.1's ladder, location as the word for a place
  of note at any extent, wilderness as the ambient tier for places, biome
  and ruin as readings. Checked against the tabletop's overmap, which
  already has multi-cell sites with open kinds, Eponym's `sites.rs` and
  Mesocosm's place graph; the code's words are crossed against the ruling's
  and the renames are a lane. Open: the size of a site and how the nesting
  goes.
- 2026-09-20: ruling 71 recorded into §3.4.1: the roots of collection are
  the entities players care about, examination standing in for play when
  nobody plays; collection is graded from the individual to a stub to the
  event alone. RimWorld's `WorldPawnGC` checked by web search: important and
  related pawns kept, the unsimulated mothballed, and relations pinning
  pawns for good as its known failure. Open: whether events of note lapse,
  and what "little incident" measures.
- 2026-09-20: ruling 70 recorded into §3.4.1: the soup as the ambient tier,
  checked against woodshed's ambient context boundary and mere's living
  backdrop; return to the soup as garbage collection, with a roots and
  reachability reading flagged and the criteria marked open and crucial; an
  anatomical rename of Mesocosm's body `Site` sized at 104 occurrences in 26
  files; and a recommendation that the of-note tier be a component of the
  sim and not a crate per word.
- 2026-09-20: ruling 69 recorded and §3.4.1 added: three tiers of keeping,
  the soup, things of note and legend, for every kind of thing, with a
  location persisting when something of note happens there. Checked against
  Eponym's `sites.rs`, whose slot and `Generated` or `Inherited` source
  are the two lower tiers already, and against the founding record's "relics
  with provenance" and "sites with history". Open: lapsing back into the
  soup, the anchor of a place when the ground moves, and the middle-tier
  words, which are Mark's naming round.
- 2026-09-20: ruling 68 recorded: a polity's goals derived from its own
  alignment as an individual's are, no built-in will to continue, goal
  change as finding a new use, and revival of an inactive polity ruled.
  Readings flagged: clinging on as subordination to the faction of a
  polity's own officers, and professed against practised alignment. The
  faction and polity rung's big picture is closed; its open details stay
  listed in §3.2.2 for the W2 sim plan.
- 2026-09-20: ruling 67 recorded: polities under no host are a faction among
  themselves and a treaty is a standing agreement at scale, which widens
  ruling 8's faction to any members that can consent and makes a federation
  the same rung transition as a founding. No separate model of diplomacy.
- 2026-09-20: ruling 66 recorded: any means may stand behind a constitution,
  and a polity may automatically become suppressed, inactive, superseded or
  subordinated to a faction. §3.2.2 now holds existence as asserted and
  condition as derived, and takes Mark's "inactive" in place of this
  record's "dormant". Open: further conditions, and the limits of a
  subordinating faction.
- 2026-09-20: ruling 65 recorded, correcting this record's reading of
  collapse: a polity without support goes dormant and does nothing, and dies
  only by agreement, for a changed context, for being forgotten, or for
  having become pointless. A polity is goals, methods and means, and what is
  scarce to it is support. Open: charters as bearers, who must agree, reform
  and secession.
- 2026-09-20: ruling 64 recorded into §3.2.2: enforcement as what a polity
  provides and asserts and can therefore withhold and revoke, efficacy read
  as ruling 50's trust, collapse as what the record says once enforcement
  fails, a polity's focus as a predicate over acts with scale orthogonal,
  and contingent polities as borrowed enforcement with cascading failure.
  Checked against Eponym's standing agreement, the consent-only form.
  Open: whether the link from deciding act to means is a default or a
  constraint, reform and secession, and whether collapse is a threshold or a
  slide.
- 2026-09-20: ruling 63 recorded and §3.2.2 added: a faction acts by consent
  and is derived, a polity adds an asserted constitution, a form of
  governance is a qualifying predicate and a deciding act, and forms are
  chosen by influence-weighted ranked preference derived from alignment.
  Checked against `eponym-social`, which is the consent model at the scale
  of two without alignment, and `isometry-campaign`'s faction turn, which is
  a far-rung stand-in. Open: the fallback protocols, and what holds and ends
  a polity.
- 2026-09-19: ruling 62 recorded, the significant dead residing on the
  planes after life and summoning as costly re-embodiment; W2 continues
  down into the faction and polity rung at Mark's word.
- 2026-09-19: ruling 61 recorded, death final in the sim with each
  game's trick over it, Mesocosm's cohort, Eponym's non-fungible
  companions, Isometry's reversal by rules; the earlier reading of the
  pillar corrected.
- 2026-09-19: ruling 60 recorded, driving as Eponym's mode with
  directing composed on top through opinion-modified directives to
  companions; §9.14 closed, and the last of §3.2.1's ten questions with
  it.
- 2026-09-19: ruling 59 recorded, play as directing rather than driving,
  senses as the creature's own and surfaced as suggestions, the played
  creature and the NPC autopilot as one mechanism, with prior art; one
  tension with the taste record's skill-based action raised as §9.14.
- 2026-09-19: ruling 58 recorded, spreads as bodies, a fungus one and a
  germ many, with the two doors merging at micro scale.
- 2026-09-19: ruling 57 recorded, the two doors of inheritance, NPC
  lineages adapting at the epoch boundary as the player's autopilot,
  branching with a plan, and Ptree's property vocabulary read as the
  far-rung representation of a lineage.
- 2026-09-19: ruling 56 recorded, standing as alignment, reputation and
  rank; the character level adds no new kind to the schema.
- 2026-09-19: ruling 55 recorded, one memory graded over one history;
  §3.2.1's third question closed.
- 2026-09-18: ruling 54 recorded, place as the same split as holding,
  presence and defence derived below the assertion line and home,
  property, ground and borders asserted above it, with naming as the act
  that extends a sophont's claims over critters and places.
- 2026-09-18: ruling 53 recorded, holding as two relations, containment
  derived from capacity and possession asserted by those who can assert.
- 2026-09-18: ruling 52 recorded, the referent as a privilege of the
  greater tiers and power as frequency weighted by significance; §9.13
  closed.
- 2026-09-18: rulings 49 to 51 recorded: the three quantities of a
  divinity, tier from forms, domain from means and effects, power from
  process frequency; a tenet as a process-to-effect relation with opinion
  and trust, its missing attitude to means supplied by ruling 47;
  alignment derived at the sophont and faction rungs and constitutional
  at the polity rung. One tension with §7.4's chosen referent raised.
- 2026-09-18: rulings 47 and 48 recorded: sacrifice as destruction with
  forms deciding the tier and acquisition means deciding the domain,
  memory as a fourth bearer form; divine places by presence, placement as
  a reading with durability and quality, rooms evaluated as RimWorld
  does, and alignment by tenets as a derived valence, with prior art.
- 2026-09-18: ruling 46 recorded, grades per glyph by the form sacrificed,
  item, learned, embodied, aggregated to the tier of godhood; presence
  mode left to the organ's plan.
- 2026-09-18: ruling 45 recorded, the provenance and identity axes
  agreed; the second tier's word narrowed to sophont or denizen.
- 2026-09-20: terminology supersession recorded: `denizen` is the individually
  remembered simulation tier; the platform admission term is `participant`.
  The proposed `borg`-to-construct reassignment remains open.
- 2026-09-18: ruling 44 recorded, the tiers of godhood as the quality of
  the journey, read against the general model plan's §7.4 and the glyph
  expression ruling of 2026-09-15.
- 2026-09-18: ruling 43 recorded, a divine thing's presence modes and
  prophesied ending, read as terms the ascent writes into the world's
  rules.
- 2026-09-18: ruling 42 recorded, divinity as intrinsic provenance and
  constructs as a tier, with the provenance-axis reading proposed and the
  second tier's word put to Mark's naming round.
- 2026-09-18: ruling 41 recorded: one language at two scopes, effects
  from world composition deferred, the space scope as the extension
  Lancer waits on.
- 2026-09-18: rulings 39 and 40 recorded: kingdom as class with scale
  orthogonal, germs as spreads and macro creatures as terrain-bodies; and
  the licence rule, applied by reading the Daggerheart, Lancer, ICON and
  CAIN licence texts.
- 2026-09-18: §5.1 written, rulesets over the sim: the existing system
  plugin seam read as the binding that exists, the taxonomy of what
  varies across five rulesets and what is common, world effects as
  rulesets at world scope per Mark's addendum, prior art, and the
  Daggerheart licence checked. Three rulings put to Mark as §9.12.
- 2026-09-18: rulings 37 and 38 recorded: methodology by tier as the
  agent literature's, the borg line as Dwarf Fortress's historical
  figures, disposition as five factors plus event-caused traits; and
  state as one ledger read more coarsely up the levels, the coarsening
  provisioned by the collective. §3.2.1's first two questions closed.
- 2026-09-18: ruling 36 recorded, the creature at three levels of
  identity in Mark's words, with §3.2.1 carrying the answer, the wing's
  existing vocabulary beside it and ten open questions.
- 2026-09-18: ruling 35 recorded, each game's foregrounded rung in Mark's
  words, mapped to the founding record's critter, borg, character
  continuity in §5. The prior-art brief for the sim written the same day.
- 2026-09-18: W1 applied to all three products (200bec7, 37aa580 and the
  Eponym commit): every plan carries its line, six retirements archived,
  indexes consistent. The founding record and the three descriptions
  amended in Mark's words (d64e710). W1 closed.
- 2026-09-18: rulings 31 to 34 recorded: W1 accepted in full, the
  process definition founded from the world-conditions schema, the
  founding record and CLAUDE.md amendments drafted in §9.11 for Mark's
  commit, and eponym-identity to dramatis under W3. kiss3d checked from
  source and found to compose. W1's application begins, one product at a
  time.
- 2026-09-18: rulings 26 to 30 recorded: glTF generated from the tree as
  the one render model; kiss3d if it composes, renderer swappable; salva
  with the field as the record; desktop first-class and the web a tier
  with a floor; genet's reference set. Rayon on the web re-checked and
  still nightly-only.
- 2026-09-18: Mark answered §9: isotropy and isostasy named, hex as
  projection, web first-class, one bench, gamepads to genet, keymapping
  across the stack. Open: lighting parts, `ProcessDef`, localization.
