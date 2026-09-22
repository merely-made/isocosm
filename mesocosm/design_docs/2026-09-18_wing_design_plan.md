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
73. **A world has one shape, any shape.** Mark, 2026-09-21, on ruling 72's
    "any shape works": "i mean a world could be any shape, but just one. Not
    like switch between them. I mean to enable strange scenarios like a
    world resting on a critter".
74. **Nesting is one composable, expandable mechanism; world, region, area
    and battlemap are its defaults.** Asked whether the nesting of maps is
    one mechanism repeated or a fixed set of levels, Mark, 2026-09-21: "I
    think it would be cool if the mechanism was composable and expandable
    instead of the order and tiering of the nesting being predetermined. But
    yeah, those are good defaults."
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
protocols" apply. *Reading, accepted by Mark 2026-09-21 (D1):* because they are general they belong to the
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
constraint the sim holds.

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
reform and secession.

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
  with nobody choosing. Under ruling 98 these are the world entity's own
  processes, its metabolism (§3.3.1), and where the world is a critter or
  a sophont they are chosen.
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

**One definition, run two ways (ruling 75).** A process is executed for the
one in the foreground, step by step with a receipt, and applied to the many
in the background as a rate over a distribution, which is "cheaper through
aggregation and not needing to process some foreground info". The two
"should agree", and "losses should be managed between transitions in a
manner that preserves similitude". What licenses the aggregate is ruling
75's other half: "things of no note are fungible", interchangeable, so a
cohort may be counted and need not be listed, and a generated thing is "a
placeholder meant to be reified/supplanted by the player". How long a
realised thing lingers before it is funged is a host's setting, "if you have
a very capable system and want to increase the buffer of stuff before things
are funged, for a feeling of consistency", which is ruling 71's collector
given a budget.

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

Prior art for ruling 75, known. Gillespie's stochastic simulation algorithm
executes every event exactly, and his tau-leaping (2001) jumps over many
events at once by drawing their counts, with the same statistics, which is
the foreground and the background of one definition. Kevrekidis's
equation-free method is where *lifting* and *restriction* are named.
Cautions, from memory of games and unverified: the X series resolves combat
differently in and out of the player's sector and players exploit the
difference, and S.T.A.L.K.E.R.'s A-Life switches between an offline and an
online model with visible seams.

### 3.3.1 Needs, value and exchange

From ruling 94. Mark's two questions, why people act and how that aggregates
into trade, are answered here as a reading, docketed as D25, built from
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
| Critter, reactive | a need crosses a threshold and the matching ability runs: forage, hunt, flee, rest | the body's ledger (ruling 38): Mesocosm's nutrition budget in full, Paredros's `Needs { hunger, fatigue }` today (`paredros-world/src/bodies.rs:46-49`) |
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
byproduct is declared (§3.3, `OperationDef`). *Reading, docketed as D26:*

- *The link Mark asks for is the tenet, from the actor's side.* A sophont
  chooses among the acts it could take by three readings the record already
  makes: whether the act addresses a need it has (§3.3.1), whether it has
  the capability, which is ruling 50's *trust* that the process yields the
  effect, and whether its alignment approves the means and the effect,
  ruling 50's *opinion*. Paredros's willingness rule is the same three gates
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
refines ruling 47's middle form: what is sacrificed there is a technique or
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
the hagioglyph organ's plan owns.

**Materials are the roster (ruling 97).** The typology of kinds of nis is
the world's roster of lineages, and so the materials of a world are the
critters that live or lived in it. That is the nis ruling of 2026-09-02 read
from the other end: matter is typed "kingdom first (flora, fauna, myco,
micro...) and lineage under it", "with the type vocabulary world-derived
from the roster rather than an authored element table" (playable ecology
plan §6, ruling 4). No material table exists anywhere in the wing, and none
is to be written. *Reading, docketed as D28:*

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
  in Mesocosm over the ages is a material in Paredros and Isometry. That
  gives Law A's line "shapes become relics of factions" its literal
  mechanism: a shape crosses as an item bearing the lineage's nis.
- *Check against Law A,* left **open** for Mark. The founding record says
  what crosses games is "choices under scarcity, not morphology", with the
  boundary that "morphology still matters enormously within Mesocosm, where
  the simulation reads the body directly". Here morphology matters in the
  other games as material. The reading that keeps Law A is that what crosses
  is an item and its lineage's provenance, and the material properties are
  ruling 38's coarse far-rung reading of the lineage's traits, not the body;
  whether that satisfies the law is Mark's to say.
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

**The world as an entity (rulings 98 and 99).** *Reading, docketed as D29.*
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
asked for at the root. *Constructs close the loop.* "Stuff can be made out
of the world, even critters": a golem or an automaton is ruling 45's
construct with the world's nis as its matter, and every producer already
makes its own nis out of the world's. *Divinity and the planes line up.* A
god is intrinsic to the world's provenance because the world is the entity
whose provenance that is, and a plane, a world in relation (ruling 11), is
another such entity. *Founding chooses or draws it.* What kind of entity the
world is belongs to the short flow of ruling 89 beside seed, shape and
ruleset, or to the seed if left blank, which is the strange-scenario knob
Mark keeps asking for, "a world built on a critter", given a home.

**Open.** Ruling 39 called the world's scale *macro* and ruling 98 calls the
world "the basic macro kingdom", so whether every macro terrain-body, the
lion turtle among them, is of the world's kingdom or may be macro fauna on
its own is Mark's to say; and whether a planetary system (ruling 11) is an
entity above the world in the same way.

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

- *What already varies.* Checked 2026-09-22: Paredros resolves a strike
  geometrically, a swept volume against a body's part bounds with line of
  sight through the ground, and the hit's `quality` is the overlap
  (`paredros-world/src/combat.rs:53-60,218-232`, `combat/precision.rs`);
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
Checked 2026-09-22: Paredros's `Work` is a craft, a grade and a danger, over
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
(ruling 38); Paredros keeps `Needs` on the body and `party_resources` as
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

**Secrets (ruling 87).** "A secret is valuable... it is tremendous leverage,
often, or there wouldn't be rules against its propagation", the word then
corrected to "not rule, but 'taboo'"; and whether one is told turns on
"relation, opinion, and goals", with personality wondered about. *Reading,
docketed as D21:*

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
  rule for it: Paredros's willingness gates, would I risk that for you, is
  it more than I would bear (§3.2.2), with the taboo's sanction as the
  danger and the hearer as the asker. Relation and opinion are its trust and
  affinity; goals are what telling buys. Personality is not a fourth factor
  but the thresholds: that rule already reads `bearable(affinity, caution)`
  as "3 + affinity / 2 - caution" (`paredros-social/src/willing.rs:40-42`),
  caution selling what affection buys, and ruling 37's five factors supply
  such terms.
- *The regime is notes, not a field.* A secret is an explicit set of
  knowers, each holding a note. Paredros holds this regime already, checked
  2026-09-21: an "append-only, observer-scoped record of claims about
  accepted deeds" whose evidence is "a direct sighting or an addressed
  transmission" (`paredros-social/src/epistemic.rs:7-11,37-43`).
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
history, relationships, or explicit designation makes it matter". Ruling 69
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
*Reading, accepted by Mark 2026-09-21 (D12):* a thing of note sheds in steps, the full individual, then a stub,
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
*Recommendation, accepted by Mark 2026-09-21 (D13):* a component, one module of the sim, and
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
chunk of W3. How the nesting goes is **answered, ruling 74**, below.

**Nesting is one composable mechanism (ruling 74).** "Composable and
expandable instead of the order and tiering of the nesting being
predetermined", with Isometry's scopes, world, region, area and battlemap,
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
record.** Mark is right that the record had put an Isometry paradigm into
the sim. World, region, area and battlemap are Isometry's second pillar,
"Maps at every scope, sculpted... Same world, different scopes"
(`design_docs/PROJECT_DESCRIPTION.md:26-29`), and the pillar's own word is
*scope*. The record conflated two things. *Answer to ruling 92's questions,
docketed as D24:*

- *Nesting* is the world's and the sim's: a node opening into a finer map,
  allocated as locations are generated (ruling 88), free of scale and with
  no named tiers at all. The only spatial nouns the sim has are Mark's own
  from ruling 72: the world map, the site, the region, the location, and the
  volume under them.
- *A scope* is a game's: the window it frames over that nesting to play in,
  an extent and a grain with the game's grid projected over it (ruling 18).
  It sits with perspective in ruling 6's list of what foregrounding chooses,
  so it belongs to the overlay and W5, not to founding.
- *Each game has its own scopes over the one world.* Isometry has its four.
  Mesocosm has one, the enclosure, "larger than a microcosm, smaller than
  the world", which "names the scale of the enclosure"
  (`mesocosm/design_docs/PROJECT_DESCRIPTION.md:17-19`), and perhaps the
  world map at a lineage's grain when deep time is watched. Paredros has a
  continuous one, "settlements, dungeons, ruins, underground regions, and
  the open world" walked without a cut
  (`paredros/design_docs/PROJECT_DESCRIPTION.md:11-12`). The same node can
  be a terrarium to one game, a dungeon to another and a battlemap to the
  third, which is the pillar's sentence taken literally.
- So: neither of the other games needs those distinctions; each game gets
  its own scopes; and the tiers do not mean different things to different
  games, because the sim has no tiers to mean anything. Ruling 74's "good
  defaults" then reads as the defaults of Isometry's overlay. What a ruleset
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
several cells with open kinds, already. Paredros has the stable address and
the two keeping tiers (§3.4.1), with a closed enum of five kinds and a
surface and underground `Layer`, which is one fixed case of a nested region
(`paredros-world/src/sites.rs:13-17,46-53`). Mesocosm has the place graph
derived from relief over a fixed three-by-three partition (§3.7). The words
are crossed against ruling 72: Paredros's `SlotId` is Mark's site and its
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
as Isometry's paradigm, and scopes are a game's and not the world's
(§3.7.1). The instances are allocated when a location is generated or first realised,
by what the location is, a dungeon opening into floors and a ship into
decks, and an unvisited one allocates nothing (accepted reading D15). §3.7.1
is amended to match.

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
(`paredros/design_docs/2026-09-09_world_conditions_plan.md`, core invariant
4). Advancing time is what makes the seams honest: whatever the new ruleset
cannot express is accounted for by the years that passed, as ruling 38 made
the coarsening of needs diegetic. *Reading, docketed as D22:* realignment is
ruling 57's epoch boundary at the scale of a world, the shop between rounds,
and it is a rung transition of §3.3; Mark's example, from hex exploration to
piloting a ship in orbit, is also a change of foregrounded scope under
ruling 74, so a realignment may add steps to the nesting as well as change
the rules.

**History (ruling 91).** How much past a world is given is the founder's
choice, "a complexity and size issue", as in Dwarf Fortress. The generated
timeline is one "one could go back in time to prior worldstates with", and
the client is not to be overwhelmed but offered "the opportunity to zoom in
and edit stuff". *Reading, docketed as D23:* going back is cheap because of
the derivation rule. A world is its seed, its rules and its asserted facts,
and the sim is deterministic, so the state at any past time is that log
replayed to then, with checkpoints at epoch boundaries bounding the cost and
the oldest coarsened first under ruling 70's buffer. Going back *and
editing* is asserting a fact in the past, which makes a branch (ruling 7),
never a rewrite, so the timeline a founder edits is the same thing as the
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

### 5.2 The boundary between the sim and a game (recommendation, unruled)

**Review, 2026-09-21, at Mark's word: "This ruling feels particularly
contentious, so let's review what it would cost and if we've overextended
ourselves a bit in the brainstorming."** §5.2 to §5.4 are held. Only ruling
78, a clean boundary, is ruled in them. Sized against what exists:

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
   DM-authority ordered event log and Paredros's fixed input trace with
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
   owed by the ruleset and checked on the bench.

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
is then a tier per platform under one contract, which is how ruling 19's
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
  diffusion as reach. Checked: Mesocosm's ledger, Paredros's `Needs`, the
  tabletop's stores, and the schema's distinct accounts. Docketed as D25.
- 2026-09-21: ruling 93 recorded: deep time is the same sim, with no
  separate history generator. §3.9 notes what already follows from rulings
  71, 75 and 91, and that Mesocosm's deep time is this today at the scale of
  an enclosure; two scaling findings filed with the isoscape family plan.
- 2026-09-21: ruling 92 recorded, Mark catching an Isometry paradigm in the
  sim. §3.7.1 corrected: nesting is the sim's, scale-free and unnamed, and
  scopes are a game's, belonging to the overlay with perspective; world,
  region, area and battlemap are Isometry's second pillar, Mesocosm's scope
  is its enclosure and Paredros's is continuous, each checked against its
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
  with Paredros's willingness rule and epistemic log checked as the existing
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
- 2026-09-21: the [readings docket](2026-09-21_wing_readings_docket.md)
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
