# Prior art for the simulator

**Date:** 2026-09-18

*Names, 2026-09-22 (wing design record, rulings 109 and 110): the sim and the family are Isocosm, the second-person game is Eponym (formerly Paredros), the tabletop is Isocosm: VTT with Isometry as its subtitle, and Mesocosm is unchanged. Verbatim rulings, quotations and code paths keep the old words until the rename lane lands.*

**Status, 2026-09-18:** brief, for Mark's reading before W2 opens. Nothing
here is a ruling; it is the references the wing already gives, mapped to the
simulator's design questions, plus references it has not considered, each
named for the mechanism it is prior art for. Every game claim is from
general knowledge and is marked so; academic references are named by author
and year so they can be checked; nothing was verified against source today
except where the status column says.

**Owns:** the prior-art map for the
[wing design record](2026-09-18_wing_design_plan.md)'s sim tier (§3) and its
parallelism section (§4.7), and the genre question Mark raised for Mesocosm.

**Does not own:** the influence set for the games' overlays, which is the
founding record's (§1, the influence set and the integrated taste record of
2026-09-09) and each vessel brief's; rendering prior art, which the record's
§4.6 carries; or any ruling.

**Method.** A reference here is prior art for a *mechanism*, never a target.
Each row says what the reference did, which section of the sim it bears on,
and what to take or avoid. The record's rule stands: any claim that reaches a
ruling is checked first, and a claim about a game's internals is checked by
reading it or its developers' own account, not by playing it.

---

## 0. What the wing already cites, mapped to the sim

| Reference | Cited in | Bears on | What it is prior art for |
| --- | --- | --- | --- |
| Dwarf Fortress | founding record, Law B, isoscape plan | §3.4, §3.6, generation, the overlay contract | Worldgen then centuries of history before play; one world played as fortress, adventurer and legends viewer; rumours spreading by witnesses; z-levels viewed one at a time, which is the focus cut. The strongest existing proof that one sim can be played several ways |
| Rain World | ruling 9, tastes | Mesocosm's overlay; §3.5 | Creatures with their own lives and indifference to the player. Also, as generally described, a two-tier creature simulation: full physics in the visited room, an abstract simulation of creatures moving between rooms elsewhere, which is the record's background and foreground in one shipped game |
| Caves of Qud | ruling 9, tastes | §3.4, factions, generation | Generated history, sultans, cults and factions with reputations; history that the world then references in its own text |
| Kenshi | vessel briefs, tastes | Eponym's overlay; §3.2 | A world with no protagonist: factions, bases and economies that carry on; long unattended stretches |
| RimWorld | tastes | Overlay pacing; §4.7 as the counter-example | Needs and moods as utility, a storyteller pacing events, real time with pause. Its single-threaded tick is the thing the record's §4.7 exists to avoid |
| Mount & Blade | tastes | §3.2 polities, the overlay contract | Kingdoms, lords and armies moving on a live campaign map with battles as instances: two scopes of one world, and polities that change without the player |
| Rogue Legacy | founding record | Lineage | The generational loop the wing was founded on |
| XCOM 2 | tastes | Ensemble | A roster you attach to because its members' histories differ |
| Nemesis system, Shadow of Mordor | tastes | §3.4 | Named enemies who remember and return; the floor for remembrance |
| Veloren | isoscape plan | Generation, §3.5, §4.7 | Open-source Rust: chunked worldgen and RTSim, a far-tier simulation of characters and economies outside the loaded area, readable in source |
| Spore | founding record | Anti-pattern | Several games glued at their seams; the hollowing the one-substrate law exists against |
| Diablo, Spelunky, Unexplored | board discussion 2026-09-16 | Generation | Template with random fill; cyclic dungeon generation for adventures with fixed anchors |
| Tactics Ogre, FFTA, Voxatron, Katamari, Scavengers' Reign, Breath of the Wild, Balatro, Vagante, PSO, Clone Drone, CDDA, Disco Elysium, Barony, Delver, Gotcha Force, Sword Hero | tastes and rulings | The overlays | Presentation, combat, discovery, atmosphere; not the sim's |

The gaps in that set, for a simulator: nothing on society at scale, nothing on
polities that rise and fall unattended, nothing on knowledge propagation as a
model, nothing on the engineering of a background simulation beside a
foreground one except by implication, and nothing academic.

## 1. Ecology, and the wildlife question

Mark asked whether "ecology sim" is right for Mesocosm, or "wildlife sim".
The prior art splits the same way the record does. The sim tier is an
ecology: populations, trophic flow, fields, lineages, nobody's point of view.
The overlay is a wildlife game: one critter's life, first person, its senses
and its hunger. Mesocosm's CLAUDE.md already says "ecological roguelike of
lineages", which is both halves in order. "Wildlife sim" names the overlay's
seat, not the sim, and would undersell what runs when nobody is playing.

| Reference | Kind | What it did | Bears on |
| --- | --- | --- | --- |
| SimLife (1992), SimEarth | ecosystem sim | Populations, genomes and biomes at the whole-world scale, no protagonist | §3.5's aggregate tier |
| Creatures (1996) | life sim | Norns with a simulated biochemistry, genome and neural net; the deepest shipped organism model, and kleptoplasty's nearest relative in that its chemistry is real | §3.3's first shape; the body noun |
| Niche | lineage sim | A tribe of critters on hex tiles, genes inherited across generations, survival as selection; the closest shipped game to "a lineage changes a trait" | Mesocosm's overlay; §3.2 lineage |
| Ancestors: The Humankind Odyssey | lineage sim | Generations across millions of years; skills learned by one generation fixed in the next; the timescale ruling 3 gives Mesocosm | Lineage; deep time |
| Species: Artificial Life, Real Evolution; Bibites; Thrive; Keiwan's Evolution | evolution sims | Open-ended evolution of bodies and behaviour from selection alone; what "unpredictable results" looks like when it works and when it degenerates | §3.5; the generator's space |
| Ecosystem (2021) | evolution sim | Fish whose bodies and nervous systems evolve in a physical ocean; water-world ecology as the whole game | Ruling 28's water; §3.6 |
| Equilinox, Eco, The Sapling | ecosystem sims | Player-planted ecologies with population dynamics; Eco adds a society over the ecology with laws | §2 tier boundary between ecology and society |
| Tokyo Jungle, Shelter, WolfQuest, Maneater | wildlife games | An animal's life in a hostile place; hunger, predation, offspring; the overlay's genre proper | Mesocosm's overlay |
| Tierra, Avida, Polyworld, Framsticks | artificial life | Digital organisms, open-ended, the research lineage behind every evolution sim | §3.5; what "cohort" can mean |

Models to read, all checkable by author and year:

- **Lotka and Volterra** predator-prey equations, and Rosenzweig-MacArthur's
  stable version: the far-tier form of a population, two numbers and a rate.
- **Individual-based models**, Grimm and Railsback's book (2005), and the
  NetLogo wolf-sheep model: the near-tier form, and the literature on when
  the two agree, which is exactly foregrounding.
- **Dynamic Energy Budget theory**, Kooijman: one energy model for every
  organism, intake to reserve to growth, maintenance and reproduction. It is
  metabolize as a theory, and it scales.
- **Kleiber's law** and metabolic scaling theory (West, Brown, Enquist,
  1997): metabolic rate against mass to the three-quarter power. Size gates
  what you can take, as the tastes say, with a formula.
- **The Madingley model** (Harfoot et al., 2014): a general ecosystem model
  that simulates every functional group on Earth as cohorts rather than
  individuals. It is §3.5's "aggregate what isn't foregrounded" as a
  published, running system.
- **Ecopath with Ecosim** (Christensen and Walters): trophic flow networks
  with mass balance; the trophic grammar's ancestor.
- **Island biogeography** (MacArthur and Wilson) and **metapopulations**
  (Levins): what a region rung holds when places are nodes.

## 2. Society

| Reference | What it did | Bears on |
| --- | --- | --- |
| Songs of Syx | Thousands of citizens with races, needs, religion and class, a city that riots and secedes; the largest shipped society simulation per screen | §3.2 factions and polities; §3.5 |
| Dwarf Fortress, fortress mode | Personalities, relationships, needs, tantrum spirals and cults emerging from individuals; the tavern, the temple, the guild | §3.2; society "adapting and reforming according to the goals and abilities of its inhabitants" |
| The Sims | Needs as utilities, the origin of utility AI in games; relationships as scored edges | §3.3's first shape at the individual rung |
| Crusader Kings II and III | Character-driven politics: opinions, schemes, secrets, claims, councils; a polity as the sum of its characters' methods | §3.2 collective action methodology; §3.4 secrets as partial knowledge |
| King of Dragon Pass, Six Ages | The player is a clan; a ring of advisors argues; decisions are collective and mythic. The cleanest shipped picture of "a thing has state because it has a way of deciding" | Ruling 9 |
| Shadows of Doubt | A city whose citizens have routines, jobs, homes and knowledge; the game is reading evidence of what they did. A knowledge graph as the world | §3.4's reach; §3.7 |
| Watch Dogs Legion | Every citizen recruitable and playable, each with a generated life | Eponym's "they can replace you" |
| Wildermyth; Massive Chalice | Heroes who age, marry, have children and become legends or monsters across a campaign of decades or centuries | Lineage across epochs; divinity by legend |
| Prom Week, Comme il Faut (McCoy, Mateas et al., 2010) | A social physics: rules over social state that fire as moves | §3.3's first shape for social processes |
| Versu (Evans and Short, 2013) | Social practices as first-class objects that characters enter and leave; conversation as practice | The same, with dialogue |
| Talk of the Town (Ryan, 2015) | A generated town over a century: characters know things, tell each other, misremember and forget, and the errors propagate | §3.4's reach field with error; the strongest academic cite for ruling 5 |
| Sugarscape (Epstein and Axtell, 1996) | Trade, culture, conflict and wealth inequality emerging from agents on a grid with rules | The society layer as agentless plus agent processes |
| Axelrod's cultural dissemination (1997); Schelling segregation (1971) | Culture spreading by similarity; districts forming from mild preference | Factions as relations; districts on the place graph |

## 3. Polities, narratives and arcs

| Reference | What it did | Bears on |
| --- | --- | --- |
| WorldBox | A god sandbox where kingdoms, cultures, wars and plagues rise and fall with the player only nudging. The closest shipped thing to "polities that substantially change the world if left unattended" | Ruling 2's third layer |
| Mount & Blade; Bannerlord | Kingdoms at war on a live map, lords with clans and influence, fiefs changing hands without the player | Polities at campaign scale |
| Dominions | Nations, pretender gods, magic and armies in a simultaneous-turn world; divinity as a playable position | Rulings 7 and 2 |
| Shadows of Forbidden Gods | A hidden power corrupting a map of nobles and cities; polities as things to be acted on from beneath | Divinity as an agent among agents |
| Dwarf Fortress worldgen | Civilizations founding sites, warring, splitting and dying over centuries before the player arrives; legends mode as the reading of it | Deep time; rung transitions |
| Ultima Ratio Regum (Johnson) | Generated cultures with their own religions, clothing, architecture and speech; culture as the thing generated, not placed | Generation of factions and polities |
| Crusader Kings | Dynasties as the continuity across polities | Lineage as a historical entity |
| Unexplored; Dormans (2010, 2017) | Mission graphs realized as space graphs; cyclic generation with locks and keys | Variation of an authored adventure; §3.7 |
| Storylets: Fallen London, Wildermyth, Short's writing | Narrative as a pool of conditioned pieces the world state selects from | Narratives and arcs as processes |
| Ceptre (Martens, 2015) | Linear logic as a narrative and game rule language: rules consume and produce facts | A candidate shape for §3.3's rung transitions |
| Tale-Spin (Meehan, 1977), Universe (Lebowitz, 1985), Façade (Mateas and Stern, 2005) | The story generation lineage: plans, goals and a drama manager | What "narratively significant" has meant in the literature |
| Cliodynamics, Turchin (2003 onward), and the Seshat databank | Structural-demographic theory: population pressure, elite overproduction and state fiscal stress cycling into instability; Ibn Khaldun's group solidarity as the cycle's engine | A model, with data behind it, for polities rising and falling |

## 4. The record, memory and significance

| Reference | What it did | Bears on |
| --- | --- | --- |
| Dwarf Fortress rumours and legends | Events known by witnesses and spread by travellers; artifacts' fame; the legends viewer as the record | §3.4 |
| Nemesis system | A named enemy who remembers the encounter and returns changed | Remembrance as the floor |
| Crusader Kings secrets and hooks | Knowledge held by some characters, discoverable, tradeable | Partial knowledge as a relation |
| Talk of the Town | Knowledge that decays and mutates as it passes | The reach field's error term |
| Daley and Kendall rumour model (1964); SIR on networks | Rumour spread as an epidemic over a graph; ignorants, spreaders, stiflers | The reach field's dynamics on the place graph |
| Novelty search (Lehman and Stanley, 2011); quality diversity and MAP-Elites (Mouret and Clune, 2015) | Algorithms that keep an archive of solutions by novelty and quality across a behaviour space. Ruling 4's gates, novelty and quality, are these words in their technical sense | The hagiograph's promotion gate |
| Bayesian surprise (Itti and Baldi, 2009) | Surprise as the change a datum makes to a model | A candidate definition of "unprecedented" |
| The Sims' memories; Wildermyth's legacy | Per-character memory that changes later behaviour | Foreground deviation records |

## 5. Worlds, generation and branching

| Reference | What it did | Bears on |
| --- | --- | --- |
| Dwarf Fortress, three modes | One generated world played as a fortress, as an adventurer, or read in legends mode; retire a fortress and visit it as an adventurer. **The existence proof for ruling 1** | The overlay contract |
| Armok Vision, DFHack, Dwarf Therapist | Third-party renderers and viewers over Dwarf Fortress's running state | Ruling 27's swappable renderer; the bench |
| Cataclysm: Dark Days Ahead | The reality bubble: a fixed region around the player is simulated in full, everything outside is frozen and caught up on return; the overmap above it | The foreground boundary and catch-up, as shipped |
| Kenshi; Project Zomboid | Off-screen factions and a meta-game that runs coarsely | Far tier |
| Veloren | Rust source: chunk generation, erosion, sites, RTSim | Read it before W2's generator section |
| Minecraft | Seeded chunked worlds, biomes, structures; the derivation rule at consumer scale | §1, §3.6 |
| No Man's Sky; Elite Dangerous | Whole planets and a galaxy derived from a seed and never stored | §1 at the largest scale |
| Unexplored 2 | A persistent generated world across permadeath runs with legacy | Branching and legacy |
| Amit Patel's polygon map generation (2010) | Voronoi cells, elevation and moisture, rivers and biomes; the standard readable region generator | The region rung |
| Cordonnier et al. (2016); Genevaux et al. (2013); Mei et al. (2007) | Terrain from uplift and fluvial erosion; river networks first; fast hydraulic erosion | Continents and relief, ruling 11 |
| Diamond-square, Perlin and simplex noise | Where Mesocosm's relief already comes from | Baseline |
| Wave function collapse (Gumin, 2016); model synthesis (Merrell, 2007) | Local-constraint fill from a small vocabulary | Castles, wings and walls |
| CGA shape grammar (Müller et al., 2006) | Procedural buildings by grammar | Settlements |
| Compton's "10,000 bowls of oatmeal" (2016) | Perceptual uniqueness versus mere variation | What "unpredictable results" must mean to a player |
| Shaker, Togelius and Nelson (2016); Short and Adams (2017) | The two standard books on procedural content | Reference shelf |
| git | Branching, merging, a record of what was asserted | Ruling 7 |

## 6. Divinity and long lives

| Reference | What it did | Bears on |
| --- | --- | --- |
| Dominions | A god as the player's position over a nation | Ruling 7's divinity |
| Black & White; Populous; From Dust; Reus; ActRaiser | Gods acting on a world through terrain, miracles and a creature | Divinity as handles on agentless processes |
| WorldBox | Divinity as nudging a world that runs anyway | Same |
| Dwarf Fortress gods, megabeasts and night creatures | Beings that outlast civilizations and are worshipped or feared by them | Longevity by trait; motifs |
| Wildermyth; Massive Chalice | Heroes who become legends, monsters or relics; bloodlines over centuries | The chain of heirs |
| NetHack bones files | A dead run's remains met by a later one | Already cited; branching's ancestor |

## 7. Engineering the simulation

| Reference | What it did | Bears on |
| --- | --- | --- |
| Factorio | A deterministic single-threaded update with parallel preparation and lockstep multiplayer; its developer blog is the best public account of determinism under load | §4.7's merge; replay hashes |
| Dwarf Fortress; RimWorld | Single-threaded ticks that hit the wall | The counter-examples |
| Paradox's grand strategy | Per-province parallelism with a public history of nondeterminism bugs | What to avoid |
| Minecraft's Folia | Region-threaded ticking of one world | Sharding by place |
| Bevy ECS; Unity DOTS | Systems scheduled in parallel by declared data access | In-shard parallelism |
| Rain World's abstract layer; CDDA's reality bubble; Kenshi's off-screen sim | Shipped two-tier simulation | §3.5 and ruling 6 |
| Discrete-event simulation; Time Warp (Jefferson, 1985) | Due-event queues; optimistic parallel simulation with rollback | §3.5's scheduler; a warning about what parallel DES costs |
| Gaffer on Games, "Floating Point Determinism" | Why the same code differs across machines and what to do | Rulings 15 and 28's line |

## 8. What to take into W2

- **Mesocosm's label:** the sim is an ecology; the overlay is a wildlife
  roguelike of lineages. Keep "ecological roguelike of lineages".
- **Read before the generator section:** Veloren's worldgen and RTSim
  source, since it is Rust and open; Amit Patel's map generator for the
  region rung; Cordonnier for relief.
- **Read before the record section:** Talk of the Town for reach with
  error; the quality-diversity literature for the promotion gate, since
  ruling 4's words are its words.
- **Read before the agents section:** the Madingley model and Dynamic
  Energy Budget theory for cohorts and metabolize; King of Dragon Pass for
  collective action methodology; Turchin for polities rising and falling.
- **Read before the parallelism section:** Factorio's blog and CDDA's
  reality bubble.
- **The existence proof to keep in front of every W2 question:** Dwarf
  Fortress is one generated world with a history, played three ways, with
  third-party renderers over its state. Everything the record asks for has
  shipped at least once, in one program, single-threaded, in ASCII.

## Findings

- 2026-09-18: the founding record's influence set and taste record hold
  around thirty references, all for the overlays; the sim tier had none of
  its own before this brief.
- 2026-09-18: nothing in this brief is verified against source. The
  Rain World abstract-layer claim, the CDDA reality-bubble claim and the
  Dwarf Fortress rumour claim are the three most load-bearing and should
  be checked against developer accounts before W2 cites them in a ruling.
