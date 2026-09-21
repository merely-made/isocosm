# The wing as a simulator: design record and plan

**Date:** 2026-09-18

**Status, 2026-09-18:** design record, ruled through W1. W0 is ruled (rulings
1 to 34, with the founding record and the three product descriptions amended
to it); W1 is evaluated, ruled and applied for all three products. W2, the
sim's own plan, is next and is design work, not lanes.

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
    and/or means became pointless".
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
sites occupying adjacent cells cooperate, interfere or hybridise, and Mark
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
dated rulings that use `borg` remain historical records.

**Character.** A denizen contingent on a group, a polity or a collective,
even by absence. Defined by the tabletop system's schema, and resolvable
as a denizen and as a critter.

**Divinity and provenance (ruling 42).** Divinity is not a rung of the
agent ladder and not a mere status: it is promotion into the world's
provenance itself. Under §1 a world is a seed, its rules and its asserted
facts, and a branch inherits all three; a divine thing is an asserted fact
moved into that root, which is why it is present in every fork after its
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
a trait or part in a critter, by an item a borg may hold and sacrifice at
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
and the derivation rule holds at each.

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

**Kingdom and scale (ruling 39).** Kingdom is class and is a trophic
strategy: producer, consumer, decomposer, as the founding plan ruled. Scale
is a separate axis: micro, meso, macro. A germ is a decomposer or parasite
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
model. **Ruled 2026-09-19 (ruling 58): a spread is a body,** and the
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
of bodies one identity spans.

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
   (ruling 35) means for the ecology in Paredros and Isometry.
2. *Methodology.* **Answered, ruling 37.** Reactive agents for critters,
   belief-desire-intention agents for denizens when their chosen fidelity
   needs them, normative agents with roles and institutions for characters.
   A denizen's disposition is the five-factor axes when represented, and
   significant events may also cause traits, as Crusader Kings does. The denizen
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
   Paredros alike. What there is to direct depends on biology: embodied
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
   Paredros: extra lives through companions, who are not fungible, since
   no individual can replace another one for one unless it is a clone,
   which is the founding record's "they can replace you" with its cost
   named. Isometry: death is understood as a state that could almost be
   defeated with more of the sciences and magics, revivify, resurrection,
   wishes, so reversal is a process a ruleset or a world's magic provides,
   which in the sim is re-embodiment, the same transition an avatar's
   reincarnation uses, bought by rules rather than earned by ascent. That
   corrects the earlier reading from "you are one": the pillar is not
   Paredros's alone; finality is the sim's, and each game chooses what it
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
   branch after ascendance, reachable by anything, enacted by a wish with
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
protocols" apply. *Reading:* because they are general they belong to the
world's ruleset (ruling 41) and not to the faction, which keeps the faction
stateless. **Open:** which protocols.

**Already in code, checked 2026-09-20.** Paredros's `paredros-social` is
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
*Reading:* a form is a pair, who qualifies to decide and by what act the
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
*Reading:* this is the sim's mechanism for which form emerges, weighted by
influence exactly as ruling 51 weights a faction's alignment, and not
necessarily an election anyone in the world holds. It runs at the rung
transition that founds a polity (§3.3), and its winner is asserted as the
constitution. At the cohort rung the ballots are ruling 50's distributions,
so the count runs over blocs and not heads. Ties break on the seed.

**Enforcement (ruling 64).** A polity's asserted state needs means of
enforcing it, likely several, and they "must be efficacious". They need not
be violent: Mark names the provision of the requirements for life, and self
defence. *Reading:* a polity enforces with what it provides and what it
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
constraint the sim holds.

**Efficacy is read, never asserted.** *Reading:* whether a means is
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
reform and secession.

**Existence is asserted; condition is derived (ruling 66).** Ruling 65 kept
death for agreement. Ruling 66 lets everything short of it happen
"automatically", and names four conditions a polity can fall into.
*Reading,* each as something the record can already tell:

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
subordinating faction can and cannot make the polity do.

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
*Reading:* where an institution is seen clinging on, that is derived and
needs no drive of its own. The members it provisions support it for their
own reasons, and the sharpest case is ruling 66's subordination: an
institution that "wants to continue" is one subordinated to the faction of
its own officers, who want their provision to continue. That is Michels's
oligarchy with nothing added to the polity.

*Reading, on how the change happens.* Ruling 51 made a polity's alignment
constitutional, asserted. But a polity also acts, and its acts are in the
record, so an alignment can be derived for it exactly as ruling 51 derives a
sophont's. A polity then has two: the one it professes, asserted in its
constitution, and the one it practises, derived from what it does. "Polities
change like that" is the practised one moving first, as the polity takes up
what its members and its circumstances bring it, with reform as the
constitution catching up. That gives the record two readable distances for
any polity, constitution against members' preference (above) and professed
against practised, and both bear on support. **Open:** whether both are
kept, and reform itself.

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
(ruling 8), and polities may sit inside polities. *Reading:* what makes the
host an institution is that an agreement made under it holds without its
parties enforcing it themselves, and contingency is borrowed enforcement:
the guild's "or else" is backed by the town's courts, the town's by the
crown. Two consequences. A contingent polity is cheap to found, because it
need not bring means of its own, which is why a settled world grows
institutions and a wild one does not. And when a host fails, everything
contingent on it must find means of its own or fall back to a faction, so
collapse cascades, and those are events the hagiograph keeps. Checked
2026-09-20: Paredros's standing agreement is the consent-only form, "still
not a command", which its holder may decline "when its premises have
changed" (`paredros-social/src/agreement.rs:14-17`), and nothing stands
behind it but the two names on it; a durable agreement in ruling 64's sense
is that same record with a polity's enforcement behind it.

**Polities among themselves (ruling 67).** Polities under no host have only
consent between them, so among themselves they are a faction, and a treaty
is a standing agreement at scale: it holds while both keep to it, and "what
it rested on changed" ends it, in the words Paredros's agreement already has
for that ending (`paredros-social/src/agreement.rs:48`). That widens ruling
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
history, relationships, or explicit designation makes it matter". Ruling 69
gives a location the same rule: "if something of note happens there, then it
persists; otherwise it can be regenerated from the same basic facts." So
what asserts a place is an event. Naming it, claiming it and building on it
are not separate routes; they are events of note like any other. *Reading,
for Mark to reject:* this is the derivation rule with a name on it. §1
already stores deviations and derives the rest, so a thing is of note
exactly when regenerating it would lose something, and the middle tier is
the set of things that carry a deviation record. Whether a thing of note
can lapse is **answered, ruling 70**, below.

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

*Reading, for Mark to reject, since he called the criteria crucial.* Taking
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
the criteria, and whether collection must be deterministic across peers,
which replay and the moot both suggest it must.

**The roots, and graded collection (ruling 71).** The roots are care. "The
players of any of the three games, when the sim is applied to them, have
effectively selected/created entities they care about": the lineage and its
critters in Mesocosm, the one sophont and those it knows in Paredros, the
characters and what the table authored in Isometry. With nobody playing
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
*Reading:* a thing of note sheds in steps, the full individual, then a stub,
what an event needs to name its participant (a kind, a lineage, a faction, a
name if it had one), then the event alone. So events are the floor of the
middle tier, and they refer to their participants by stub, never by a live
individual, so that a participant can be collected without breaking the
event that mentions it. **Open:** whether an event of note itself lapses in
the end, as ruling 5's forgetting suggests, with legend the only floor that
never gives way; and what "little incident" measures, which is presumably
the same novelty, quality and relevance at a lower bar.

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

**Already in code, checked 2026-09-20.** Paredros's
`paredros-world/src/sites.rs` holds the two lower tiers for places. A
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
what that anchor is when the ground itself moves.

**The words.** "Denizen is now a term for an entity of note. There should be
similar terms for locations, possibly more." Naming is Mark's round and
nothing is coined here; this is what the wing already holds, checked
2026-09-20. The founding record's frame already pairs the three: "people
(subjects and their deeds), things (relics with provenance), and places
(sites with history)" (`2026-07-30_games_wing_founding.md:185-187`), so
*site* and *relic* are its words for a place and a thing with history, and
Paredros's `Site` is live code. *Site* collides inside the wing: it also
names a location on a body in Mesocosm's phenotype
(`mesocosm-core/src/phenotype/mosaic.rs:50,68`) and in
`shared/wing-functions/src/generation.rs:21,29`, and the tabletop's overmap
has an `AtlasSite`. For events the hagiograph's *feat* and *mark* are words
of the top tier, a feat being what beats a standing mark, while Paredros's
*deed*, a recorded act with a doer (`paredros-social/src/deed.rs`), is the
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
which would suit the proposed and declared forms.

"Does this need to be a crate, or can it be a component of the wing?"
*Recommendation, Mark's to rule:* a component, one module of the sim, and
not a crate per word. The three tiers are one mechanism over every kind of
thing, and the invalidation criteria are one policy, so a `denizen` crate
beside a place crate and a thing crate would hold the same keeping machinery
three times. Nothing consumes the of-note tier without the sim, which is the
usual reason for a crate boundary. It can still be built to be tested and
swapped alone behind its own seam inside the sim. The kinds are then data:
the inhabitant, the place, the thing, the event. Reserving the name
`denizen` on crates.io is a separate act and needs a real publish, per the
naming ledger.

*Ruling 72 settles the place words.* A place of note is a **location**, and
a **site** is one cell of the world map (§3.7.1). So the collision to clear
is between the world-map site and Mesocosm's body site, and Paredros's
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

### 3.7.1 The world map: sites, regions and locations

From ruling 72, whose definitions are kept whole in the first column.

| Word | Mark's definition | In this record's terms |
| --- | --- | --- |
| World map | "a grid of sites comprising a world map, in any number of shapes (sphere, ring, plane, cube, spire, wheel, any shape works for me)" | §3.7's graph at the rung above the bricks. Sites are its nodes and the shape is its topology, so it is held as adjacency and never as a two-dimensional array; the tile's shape stays a projection (ruling 18) |
| Site | one cell of that grid, "each with their own terrain and biome (mountainous, valley, island, tundra, tropical)" | the address, and the unit that generation and paging work in; its volume is grown from its terrain, its biome and the seed |
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
governance are (§3.2.2). *Reading, for Mark to reject,* on the question that
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

**Any shape.** A world map that may be a sphere, a ring, a cube, a spire or
a wheel cannot be an array with a wrap rule; it is a graph of sites with
adjacency, which §3.7 already says of every rung above the bricks. Known
mathematics bears on one case: no grid of hexagons closes a sphere, and a
geodesic grid does it with exactly twelve pentagons among the hexagons, so
"hexes" holds as a projection over most of a spherical map and not as its
storage. **Open:** how large a site is, in voxels and against the paged
chunk of W3; and how the nesting goes, since a location may be a "nested
region within a world map site".

**Already in code, checked 2026-09-21.** Each product holds a piece. The
tabletop's overmap projects a region as a grid of `AtlasTerrainCell`s whose
`kind` is a string from "the open tile vocabulary", with `AtlasSite`s, each
"a geographic site area" with a cell `footprint`, and routes between them
(`isometry-views/src/overmap/atlas.rs:20-38`); its campaign world keeps a
`WorldPlace` with a name, tags, an optional map and an optional position
(`isometry-campaign/src/world/types.rs:78-92`). That is a location over
several cells with open kinds, already. Paredros has the stable address and
the two keeping tiers (§3.4.1), with a closed enum of five kinds and a
surface and underground `Layer`, which is one fixed case of a nested region
(`paredros-world/src/sites.rs:13-17,46-53`). Mesocosm has the place graph
derived from relief over a fixed three-by-three partition (§3.7). The words
are crossed against ruling 72: Paredros's `SlotId` is Mark's site and its
`Site` is Mark's location, and the tabletop's `AtlasSite` is a location too.
Bringing the code's words to ruling 72's is a lane under W2 and W3 beside
the body-site rename of §3.4.1, not an edit.

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
| Paredros | the denizen, an individually remembered inhabitant, and its factions | what remembered entities do; the coterie, the party, the base | the ecology as wildlife and land; polities as the powers that shape the region |
| Isometry | the character inside its polities | characters partisan, friendly, antagonistic, factional or unaligned | polities set the narrative stakes (sidequests, alignment, arcs, non-player characters, access to resources); the ecology as terrain and encounter |

"Weakly expressed" is a requirement on the sim, not on the game: every rung
must be able to run at background fidelity and surface as effects,
encounters, pressures and stakes in a game that does not play it. What that
requires of each rung is a W2 question.

### 5.1 Rulesets over the sim

Mark's question of 2026-09-18: "how do you make one sim be useful to both
a srd, pf2e, daggerheart, PbtA, GURPS, etc. ruleset? maybe we taxonomize
each, consider them each a language we have to make bindings to the sim
for?" And his addendum: "perhaps it's worthwhile to think of some world
effects as like rulesets for mesocosm and/or paredros?" This section is
the assessment; the rulings it needs are in §9.12.

**The seam that exists.** Isometry's system plugin (`crates/isometry-system`,
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
ruleset by digest (processdef plan, PD3); Paredros's world-conditions
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
   record (ruling 28). **Closed 2026-09-18:** kiss3d 0.46 takes an
   external device and renders offscreen (§4.2), so it is a tenant, and
   ruling 27's composition test is met on paper; the running proof is
   W3's.
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
14. **Directing against driving:** **ruled, ruling 60.** Directing is
    Mesocosm's mode over its critter and its cohort. Driving is
    Paredros's, because you are one sophont, and the taste record's
    skill-based action stands; directing composes on top of it as the
    directives that sophont gives its companions, whose efficacy is
    modified by their opinion of it. That is Paredros's first pillar,
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
- 2026-09-18: W1 evaluated Mesocosm: 37 rows, 13 rewrites,
  4 retirements, 19 keeps (counts corrected from the table on application); five corrections folded in (§3.3,
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
- 2026-09-21: ruling 72 recorded and §3.7.1 added: the place words in Mark's
  definitions, site joining §3.1's ladder, location as the word for a place
  of note at any extent, wilderness as the ambient tier for places, biome
  and ruin as readings. Checked against the tabletop's overmap, which
  already has multi-cell sites with open kinds, Paredros's `sites.rs` and
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
  Paredros's `sites.rs`, whose slot and `Generated` or `Inherited` source
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
  Checked against Paredros's standing agreement, the consent-only form.
  Open: whether the link from deciding act to means is a default or a
  constraint, reform and secession, and whether collapse is a threshold or a
  slide.
- 2026-09-20: ruling 63 recorded and §3.2.2 added: a faction acts by consent
  and is derived, a polity adds an asserted constitution, a form of
  governance is a qualifying predicate and a deciding act, and forms are
  chosen by influence-weighted ranked preference derived from alignment.
  Checked against `paredros-social`, which is the consent model at the scale
  of two without alignment, and `isometry-campaign`'s faction turn, which is
  a far-rung stand-in. Open: the fallback protocols, and what holds and ends
  a polity.
- 2026-09-19: ruling 62 recorded, the significant dead residing on the
  planes after life and summoning as costly re-embodiment; W2 continues
  down into the faction and polity rung at Mark's word.
- 2026-09-19: ruling 61 recorded, death final in the sim with each
  game's trick over it, Mesocosm's cohort, Paredros's non-fungible
  companions, Isometry's reversal by rules; the earlier reading of the
  pillar corrected.
- 2026-09-19: ruling 60 recorded, driving as Paredros's mode with
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
  Paredros commit): every plan carries its line, six retirements archived,
  indexes consistent. The founding record and the three descriptions
  amended in Mark's words (d64e710). W1 closed.
- 2026-09-18: rulings 31 to 34 recorded: W1 accepted in full, the
  process definition founded from the world-conditions schema, the
  founding record and CLAUDE.md amendments drafted in §9.11 for Mark's
  commit, and paredros-identity to dramatis under W3. kiss3d checked from
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
