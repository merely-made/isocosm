# The sim design sessions, 2026-09-16 to 2026-09-24: notes

**Date:** 2026-09-22. **Extended 2026-09-24 and 2026-09-25** with the
refinement session, §8, at Mark's word ("Record session notes").

*Names, 2026-09-22 (wing design record, rulings 109 and 110): the sim and the family are Isocosm, the second-person game is Eponym (formerly Paredros), the tabletop is Isocosm: VTT, with Isometry retired as a product word and kept only as the plain technical prefix of its crates, and Mesocosm is unchanged. Verbatim rulings, quotations and code paths keep the old words until the rename lane lands.*

**What this is.** A reference preserving the conversation in which the
wing's simulator was designed from the top down, at Mark's word ("preserve
the notes of this discussion in a secondary reference file"). It records
the method, the sequence of questions and answers, what each answer
produced, what was checked against code, what was got wrong and corrected,
what was named, and what was left open. It is not an authority: Mark's
words are quoted verbatim as numbered rulings in the
[wing design record](2026-09-18_wing_design_plan.md) §0, the readings this
record's author added are on the
[readings docket](archive_docs/2026-09-24/2026-09-21_wing_readings_docket.md), and the compiled
result is the [sim plan](2026-09-22_sim_plan.md). Where this file and those
disagree, they win.

**Who spoke.** Mark ruled; the assistant asked, checked and recorded. In
what follows "the question" is the assistant's and "the answer" is Mark's,
cited by ruling number.

---

## 1. How it started

On 2026-09-16 the board-on-isometer plan's targets were found to be test
constraints chosen by the plan's author rather than facts about the
product: a demo map, parity with the existing board, a one-megabyte brick
budget inherited unexamined, and a five-voxel tile chosen to patch a
symptom of those. Two rulings had been taken on claims never checked
against code (that the tracer assumed cubes; a 128 MiB texture bound that
was an arithmetic error).

Mark's diagnosis, in order:

- "if there's not one board for all situations, we fucked up"
- "We're thinking very small... Bush league, brother"
- "A random castle example does not prove anything" and then, "Unless it
  were literally random. Then it would be useful", which became ruling 15,
  tests are draws.
- "this is a situation requiring real reevaluation, refactoring, and likely
  reorganization of the wing"
- On the frame that shared parts are extracted from shipped games: "The
  guts here are intended to arise from combinations of game systems. It
  never made any sense that they would emerge instead of being designed...
  You don't know the right target. Any target is a test. Picking a
  flagrantly non representative, unreasonably limited target is just
  self-sabotage."

Then the method: "What if you just prompted me? Big picture to small,
continuously, until a clear picture emerges?"

## 2. The method as practised

- One question at a time, from the top down, each next question chosen from
  the last answer. No options offered for design content; options only for
  administrative forks.
- Every answer recorded verbatim as a numbered ruling in the record's §0,
  committed and pushed the same turn, by pathspec, with new files added
  first.
- Every claim under a ruling checked against code or source before it
  reached Mark, and marked checked or unchecked; findings filed with the
  plan that owns the subject, not only the record.
- The assistant's own readings flagged as readings, never folded in as
  rulings. When they piled up faster than they could be ruled, they were
  gathered on a docket for ruling in one pass, the way W1 was.
- Prior art beside every ruling: known literature named as known, game
  references from memory marked unverified, licences read from their
  texts.
- Naming through Mark's rounds: crates.io checked by API and the registries
  checked named; results in the naming ledger; nothing coined by the
  assistant.
- The record kept in Mark's words; PROJECT_DESCRIPTION and CLAUDE.md files
  edited only on his word.

## 3. The sequence

Dates are the days the answers were given. Ruling numbers refer to the
record's §0; D-numbers to the docket.

### 3.1 W0: the big picture, 2026-09-17 to 2026-09-18

| Question | Answer | Produced |
| --- | --- | --- |
| What is the thing? | A simulator: "a configurable, customizable, extensible world, history, and entity generator you can play at least three different ways" (1) | The three tiers: sim, stack, games |
| What runs in the background? | Adapting ecology, society reforming, polities and narratives and arcs; the goal is to "define the taxonomy" (2) | §3's plan |
| When does each game happen? | One world, one clock, unbounded time; each game foregrounds an aspect (3) | Foregrounding as the games' definition |
| How do you cheaply track hundreds of thousands of things? | The record of significant events is never lossy; memory is not global; propagation is derived, "the derived form seems fair" (4, 5) | The hagiograph as judge; reach as a field |
| What does a game choose? | A domain, an ensemble, mechanics, controls, a perspective, a timescale (6) | Ruling 6 |
| Are worlds one? | Forkable and branchable, like a moot; divinity persists across branches (7) | Ruling 7 |
| The taxonomy list | Creature, faction, polity, lineage; a settlement is a faction until it has a methodology; a region is terrain (8); state follows methodology (9) | §3.2 |
| Are processes one shape? | Choices under scarcity everywhere, and not enough on its own: agentless processes and rung transitions (10) | The three shapes |
| What is terrain? | Region to planetary system; an interior everywhere; arbitrary cell sizes; a graph over the bricks; candidates asserted when persisted (11 to 14) | §3.6, §3.7 |
| What else does the wing need? | The stack placement (16); renderling and nexus abandoned | §4 |
| "Proceed to design the plan" | Seven answers: the sim's name; configurable base scale and tile sides; piccolo; the limits; renderling's parts; one bench; the components (§9 of the record) | The record written |
| The name | "isotropy" for the sim, "isostasy" for bridging effects (17) | Naming ledger |
| Rendering rulings | Hex as projection (18); the web (19, then 29); one bench (20); gamepads and keymapping (21); lighting (22); per-axis scale and paging (23); W3C animation (24); glTF (26); kiss3d and a swappable renderer (27); salva (28); genet's references (30) | §4 |

### 3.2 W1: every plan evaluated, 2026-09-18

Sixty-one rows across the three products: thirty keep, twenty-two rewrite,
six retire, three surfaced. Mark: "Accept all recommendations" (31). The
process definition is founded from Eponym's world-conditions schema (32).
The founding record and Mesocosm's CLAUDE.md are amended (33), and the
three PROJECT_DESCRIPTIONs, in Mark's words ("These three games are
different ways of looking at the same simulated world..."; "Think
tactically in terms of Tactics Ogre-scale tight battlemaps..."). dramatis
absorbs eponym-identity under W3 (34). Retired plans moved to
`archive_docs/2026-09-18/` in each product with rationale.

### 3.3 W2, the creature: 2026-09-18 to 2026-09-19

| Question | Answer | Produced |
| --- | --- | --- |
| What does each game foreground? | Mesocosm the critter refined over the ages; Eponym what named entities do; Isometry characters inside polities; each layer simmed and weakly expressed elsewhere (35) | The foregrounded-rung table |
| What does a creature carry at each level? | Genotype, phenotype, kingdom; naming, relationships, disposition, memory; contingency on a group (36) | §3.2.1 and ten open questions |
| Methodology by tier? | The agent literature's tiering, reactive, BDI, normative; the Dwarf Fortress line for named figures; five-factor disposition (37) | Ruling 37 |
| What is state at each tier? | One ledger read more coarsely; "dying of scurvy in a dnd game feels bizarre"; provisioning by the collective (38) | Ruling 38 |
| Kingdom and scale? | Kingdom is class; germs micro and lion turtles macro (39); a fungus one body, a germ many (58) | Body forms |
| Rulesets? | The licence rule (40); one language at two scopes (41) | §5.1, licences read: Daggerheart out, Lancer in, ICON and CAIN to ask |
| Holding, place, memory, standing? | Containment derived, possession asserted (53); a critter lives where it can, "a human can give a dog a home" (54); one memory graded (55); standing is alignment, reputation, rank (56) | Questions 3 to 6 closed |
| Reproduction and inheritance? | Two doors: reproduction rerolls, the epoch boundary changes the lineage; Ptree as the measure (57) | Question 10 closed |
| Senses? | Play is directing, not driving; the critter's senses are its own (59); driving is Eponym with directing composed on top (60) | §9.14 |
| Death? | Final across the board, each game with its trick (61); the significant dead go to the planes, summoning is costly (62) | Question 8 closed |

### 3.4 W2, divinity and tenets: 2026-09-18

| Question | Answer | Produced |
| --- | --- | --- |
| What is divinity? | Intrinsic to the world's provenance; constructs a tier; the second tier's word in question (42) | The provenance axis |
| How is a divine thing present, and how does it end? | Omnipresent, reincarnating, or fixed; ended by prophesied conditions (43) | Presence modes |
| What decides the tier? | The quality of the journey (44); grades per glyph by the form sacrificed (46); sacrifice is destruction and the domain is the means of acquisition (47) | The divinity table |
| Two axes? | Agreed: provenance and identity (45) | Ruling 45 |
| Places, rooms, alignment? | A place ascends by presence; rooms evaluated as RimWorld does; an alignment system on tenets (48) | Tenets founded |
| The domain and its power? | Named freely; power is the frequency of the domain's process; anatomy is the other road (49); a tenet defined (50); who holds tenets (51); the referent by tier, impact against frequency (52) | The three quantities |

### 3.5 W2, factions and polities: 2026-09-20

| Question | Answer | Produced |
| --- | --- | --- |
| How does a polity decide? | Factions act by consent, peer to peer; polities add methods "part of the deal"; forms of governance ranked by alignment, "like preference-ordered rank choice voting" (63) | §3.2.2; D1 to D3 |
| What holds a polity together? | Efficacious means, not necessarily violent; a focus, not a size; contingent polities as institutions (64) | D4, D5, D9 |
| What ends one? | Without support it does nothing; it dies "only when people agree" (65); any means may stand behind a constitution and conditions change automatically (66) | The correction of the record's auto-collapse reading; D6 |
| Polities among themselves? | A faction; a treaty is a standing agreement at scale (67) | Ruling 67 |
| Where do goals come from? | From its own alignment, as individuals'; no will to continue beyond a new use (68); asserting a constitution is itself an act (81) | D7; two alignments withdrawn (79) |

Checked: `eponym-social` is the consent model at the scale of two;
`isometry-campaign`'s faction turn is a far-rung stand-in.

### 3.6 W2, keeping and places: 2026-09-20 to 2026-09-21

| Question | Answer | Produced |
| --- | --- | --- |
| What makes a location a place? | "if something of note happens there, then it persists"; denizen is the word for an entity of note; a tier between the soup and legend (69) | §3.4.1, three tiers of keeping |
| Does stuff of note fall back? | The soup is the ambient tier; return is garbage collection; "the criteria for invalidation is crucial" (70) | Roots and reachability, D11 |
| Who is the player? | The entities players care about; examination counts; "you would trash most of a raider... at a certain point just the event" (71) | Graded collection, D12 |
| Is a place's kind stored or read? | The place words defined: world map, site, region, location, wilderness, biome, environment (72); one shape per world (73); nesting composable with defaults (74) | §3.7.1; D14, D15 |
| Nesting fixed at founding? | "Shouldn't the nesting be dynamically allocated as locations are generated?" (88) | Correction |
| Scopes? | "it sounds like hardcoding an isometry paradigm into the sim" (92) | Correction; D24 |

Checked: Eponym's `sites.rs` holds the slot and the two lower tiers; the
tabletop's overmap has multi-cell sites with open kinds; `wing-impresa` is
nearly the note already.

### 3.7 W2, processes, the boundary, the docket: 2026-09-21

| Question | Answer | Produced |
| --- | --- | --- |
| One definition, two ways? | The background cheaper by aggregation; foreground and background agree; losses preserve similitude; things of no note are fungible behind a buffer (75) | Lifting and restriction, D17 |
| The sim-game boundary? | The question was about the layer underneath the sim (76); two bindings if free on mobile and web (77); a clean boundary either way (78) | §5.2 to §5.4, then held under review at Mark's word: overextended, the gate reading withdrawn |
| The docket | D8 replaced: a polity's alignment from its acts (79); D10 replaced: a note is literal, the impresa record with a djot field (80, 82, 85); "i accept the rest" (83) | Sixteen readings accepted |
| What does a thing know? | Model information spread cheaply and distributionally (84); the five points agreed (86) | §3.4's knowledge resolution |
| Secrets? | Valuable, leverage, kept by a taboo; relation, opinion, goals, perhaps personality (87) | D21 |

### 3.8 W2, founding, deep time, value, materials: 2026-09-21 to 2026-09-22

| Question | Answer | Produced |
| --- | --- | --- |
| What does a founder choose? | A basic flow of crucial details plus a seed, like RimWorld; author as much as you like; a world editor is owed (89); realignment by a generative round (90); how much history is the founder's, the timeline can be gone back into (91) | §3.9; D22, D23 |
| Deep time, same sim or a sketch? | "Same sim" (93) | Findings for the isoscape plan |
| What is value? | Scarcity, capability, material need; need from alignment, acts, predilection; "Needs, like sims?" (94) | §3.3.1; D25 |
| How do capabilities link to need? | Acting is like crafting; crafting is incorporating materials; a fey mood is a triggered need (95) | D26 |
| Abilities, skills, quality? | Abilities, skills and techniques defined; nis is a scale, not a material; the question was quality (96) | Three corrections; D27 |
| What is the typology of nis? | "whatever critters exist, across all the kingdoms, those are the things stuff can be made of??? holy shit" (97) | D28 |
| Inert matter? | The world is an entity made of stuff, the basic macro kingdom (98); its kind open, "maybe it's dead. who knows!" (99); world is a kingdom, macro the scale, meso and micro open (100) | D29; `Material::Untyped` becomes the world's nis |

### 3.9 W2, the world's lineage, magic, arcs, time: 2026-09-22

| Question | Answer | Produced |
| --- | --- | --- |
| Where does a world's magic come from? | The world has traits and a lineage; be generative; glyphs are broad; a magic system generator that feeds divinity (101); seeded, configured, static or fluid, "a bit like ideology in rimworld" (102) | §3.10; six axes; D30 |
| Is an arc the sim's or the game's? | Both; the sim knows the parts, the game makes the statement; the world "a bit like rimworld's storyteller" (103) | §3.11; D31 |
| How does time run in a shared world? | Asynchronous play merged like git, or a clock that runs when played; "A, then" (104, 105) | §3.12; D32 ruled |
| Related worlds? | Shared context without merging: branched worldlines, or neighbours in a celestial neighbourhood (105) | D33 |
| "proceed!" | The sim plan and these notes | W2 drafted |

## 4. What was got wrong, and corrected

Each was caught by Mark or by a check, and corrected in the record in
place with the correction dated.

| Claim | Correction |
| --- | --- |
| The tracer assumed cubes | It builds world-space rays; the claim was never checked |
| A 128 MiB texture bound | An arithmetic error; video memory is the bound |
| Paging was the biggest change | It existed in modulus and Eponym already |
| Renderling was current | Retired by ruling, still a dependency of `eponym-client` |
| Daggerheart's SRD is CC-BY | It is DPCGL 2.0 and excludes video games |
| A polity whose enforcement fails drops to a faction by itself | It goes inactive and dies only by agreement (65) |
| "Dormant" | Mark's word is "inactive" (66) |
| A polity has two alignments, professed and practised | One, from its acts (79, 81) |
| "Of note" is the derivation rule with a name | A literal note on the thing (80) |
| The sim is reached through mere's participant gate | Withdrawn after reading `gate.rs`, which gates chartulary graph edits |
| Lost shared-memory parallelism is the largest cost of componentisation | Unknown; shards scale across cores under either binding |
| The boundary recommendation as standing commitments | Overextended; held under review; what survives cheaply named |
| Materials are nis and scruple | They are scales; materials are the roster (96, 97) |
| Hybridising abilities is kleptoplasty | Kleptoplasty is conditional; hybridising is a technique with more than one skill (96) |
| Grinding folds into the cohort, so nothing accumulates | A skill accumulates in one sophont; only the unnoted act folds (96) |
| Nesting is set at founding | Allocated as locations are generated (88) |
| Founding supplies the default scopes | Scopes are a game's; the sim's nesting has no named tiers (92) |
| "Denizen" as a title for Eponym | Taken: a live Steam life simulator; kept as the tier word |

## 5. Naming decided or opened during the session

| Word | Disposition | Where recorded |
| --- | --- | --- |
| isotropy | the sim; crates.io free, other registries unchecked; superseded by Isocosm (110 to 112) | ruling 17, naming ledger |
| isostasy | bridging effects between layers; same status | ruling 17 |
| denizen | the individually remembered tier, superseding borg (2026-09-20, in a peer session); the platform sense became participant; killed as a product title; amended 2026-09-25 to a named entity, the tier being of note (200) | Mesocosm and Eponym CLAUDE.md; naming ledger |
| sophont | working word for a sapient critter; a different axis from notability; settled 2026-09-25 as the term of art, every sophont a denizen (200) | rulings 45, 200 |
| borg to construct | an unruled proposal | record §3.2.1 |
| site, location, region, wilderness, biome, environment | ruled in Mark's definitions | ruling 72 |
| Mesocosm's body "site" | wants an anatomical word; candidates surfaced, none coined | naming ledger |
| the thing and event words of the of-note tier | open | naming ledger |
| nis, scruple | scales of matter, not materials | ruling 96 |

## 6. What the session left open

At the session's end on 2026-09-22 Mark accepted the docket's D21 to D31
and D33 as suggested (ruling 106), keeping D19 and D20 held for W5 and W3;
confirmed and extended the change to ruling 47 (107), grading an offering
by the depth of the entity's relation to that magic in its story; had the
two `mesocosm/CLAUDE.md` amendments applied; and ruled that isotropy and
isostasy, both found as game titles, are internal names and no conflict
(108). Still open: the parked questions in the sim plan's §8, the naming
items above, the question to Massif Press, which he will ask on Discord,
and the `denizen` crate's publish. *(The publish was struck on 2026-09-25
by ruling 201.)*

## 7. Lessons for the next session

- Design to the target, never to a fixture; a hand-picked example proves
  nothing and a seeded draw does.
- Ask one question and record the answer whole; do not propose design
  content.
- Check before claiming, and say what was checked.
- A reading is a reading until ruled; when readings outrun rulings, docket
  them.
- A question about one layer is not licence to commit to another; the
  review of 2026-09-21 is the case to remember.
- Findings go to the owner's plan the same day.

## 8. The refinement session, 2026-09-22 to 2026-09-25

Mark asked for a verdict on the plan and these notes ("we get a good design
out of these days of q&a brainstorming?"). After a detour into the family
rename, which he closed ("forget about the rename. the sim design and the
wing design is more important"), he asked to be questioned, "ask me
questions, refine refine", then "put the question to me directly". Rulings
113 to 173 were given on 2026-09-24, 174 to 235 on 2026-09-25 and 236 to 278
on 2026-09-26, the last
of them answering the consistency pass and the two lanes' forks, typing
the attention set, and settling the input by click.

### 8.1 How it started

The review found the backbone sound: the derivation rule, state following
methodology, the tiers of keeping, materials as the roster, and time as a
played trunk. It found the hard mechanism thin. Nothing owed agreement for
outcomes a game resolves; the cohort was not in the schema; the claim that
an aggregate form is derivable rested on one table row; collection
determinism had two holes, the funging buffer called a host's setting and
what players examined never logged; shape three mixed world events with
record work; "branch" meant two things; and no phase measured the scale the
record claims. A peer session's aggregation research, landed meanwhile,
confirmed the cohort and derivability findings from primary sources and
built only exact grouping: 5.93 times fewer evaluations in independent
worlds, none in ecological ones.

### 8.2 The method as practised

- Each question put directly as a choice, with the consequences named in
  each option, one to four at a time. Mark picked, wrote his own answer, or
  asked for a recommendation.
- Asked for a recommendation once (ruling 113), the assistant gave one as a
  reading with its consequences, and it became a ruling only at his word.
- Answers derivable from rulings already made went to Mark as readings he
  could object to, not as questions: the charter (from 129) and a
  subordinating faction's limits (from 134).
- A message Mark sent while a ruling was being recorded was quoted in the
  ruling it shaped (117, 144).
- Each ruling was recorded in the record's §0 and where it sits, carried
  into the sim plan, and committed and pushed by pathspec the same turn, at
  Mark's word ("Commit and push").
- Wing law and CLAUDE.md changed only on his explicit word, the Law A
  amendment shown as a draft first.

### 8.3 The sequence

| Thread | Question | Answer | Ruling |
| --- | --- | --- | --- |
| Sim and games | One world run twice, a village watched once and run as a crowd once: what must come out the same? | "not certain! recommendations?", then the recommendation accepted: the noted run individually, the fungible agree in distribution over what later processes read, what was watched is logged, exact is a world setting | 113 |
| | Is a game's ruleset held to the same test? | "Rulesets calibrate" | 114 |
| Conflict | When two want one scarce thing, what decides? | "Each side chooses" | 115 |
| | What ends a fight? | "Most never reach blows" | 116 |
| | What does a blow change? | "Both", vigour and wounds | 123 |
| Belief | Can things believe what isn't true? | "Any belief can be wrong", wrong or stale | 117 |
| | What decides whether a belief takes? | all four: evidence, trust, fit, skill | 118 |
| | How are they weighed? | "Both", a world baseline shifted by disposition | 119 |
| | Does the world's disposition set every inhabitant's? | "Founders only" | 120 |
| | Does the belief baseline fade too? | through trait and condition; people drift; "Should it not change…?" | 121 |
| | What moves a fluid world's temperament? | events, its cycles, its epoch boundary | 122 |
| Scale and structure | The largest world a draw runs on the laptop? | "start with the region" | 124 |
| | Noting, collection, promotion, merging: world or record? | "Split" | 125 |
| | Fork and branch? | "Yes, fork and branch" | 126 |
| Keeping | Does a note lapse? | "Everything fades", by intelligence; physical notes don't | 127 |
| | Whose memory is a note on a place? | "Whoever remembers it" | 129 |
| | What keeps a thing of note for a player? | who they play and what they pin; Eponym's notes diegetic | 130 |
| | What makes a mind remember? | all four: new, relevant, intense, repeated | 131 |
| Skills and items | Does a skill fade? | "Slowly, to a floor" | 132 |
| | What raises a skill; where do techniques come from; what limits improvement? | all four; both; the maker's skill and the item's story | 141 to 143 |
| | Items that are sophont (Mark's prompt) | made so, awakened, inhabited; they act three ways; never owned | 144 to 146 |
| Politics | When a faction can't agree? | its preferred way; factions never gated, group-vested polities are | 133 |
| | Reform; secession; death; means; deadlock | lawful routes, force founding a new polity; the seceders' own act; either route; a default; per act | 134 to 138 |
| Materials | Does materials-as-lineages satisfy Law A? | "Amend Law A", applied as drafted | 139 |
| | A world's own materials? | a spectrum from a ruleset's base to fully generated | 140 |
| Space and worlds | Site and chunk; a location's anchor; clocks; systems | independent; by its kind; synced at crossings; a host | 147 to 150 |
| Held items | Is every sophont unowned? | owning a person is slavery, a tenet judged by kinship | 151 |
| | Who may a player direct? Two at once? D19? | only who they play; "Yes, both"; "Rule it" | 152 to 154 |
| | What a Mesocosm player plays; what a DM may do | by the lineage's traits; "Both" | 155, 156 |
| Naming | The thing, event and body-site words | relic, tale, tract | 157 |
| Mood and breaks | What is mood made of; kept or read? | memory, needs and situation, with personality traits proposed by Mark; "Read, with kept strain" | 158, 159 |
| | Traits and the five factors; what a break does; does it spread? | "Traits on the factors"; all four; "It doesn't" | 160 to 162 |
| | Down or up; what sets bearing | traits, the moment, chance; traits, support, history | 163, 164 |
| Culture | What is a culture; do languages divide? | "Read, named when noted"; "Languages divide" | 165, 166 |
| | Descent, names, spread | "Like lineages"; tongue, tales, namer; contact, prestige, imposition, drift | 167 to 169 |
| | Art and ritual (offered as a thread) | "I view art and ritual as crafts" | 170 |
| Technology | What is it; ages; invention | "Both"; read and named, capped, able to realign, "the epochs of cultures/society"; need, contact, temperament, mastery | 171 to 173 |
| W5, Mesocosm | Which game first; control; directives; obedience | "Mesocosm"; "Directing, as ruled"; all four; "By its bond" | 174 to 177 |
| | The bond across generations; the start | three options, seeded by default; from habitability onward | 178, 179 |
| | Truth; collapse; unplayed lineages | creative and survival; local or global, never the end; they adapt, inherit and develop against the web | 180 to 182 |
| | At a birth; creative mode | "Parent by default"; "See only" | 183, 184 |
| | One sim; the ceiling; the plan's decisions | "Isocosm absorbs"; "decompose them under the 600 Loc limit"; only on Isocosm, record earlier, retire the slice | 192 to 196 |
| | The contract crate | "One shared crate"; "isocosm-overlay" | 197 |
| The pass and the lanes | The pass's bookkeeping; its dated lines; the slice plan | "Apply as proposed"; "Restore isotropy"; "Yes, rewrite it now" | 198, 199 |
| | The identity words; the organ crates | "Denizen as the umbrella word, sophont as term of art for sapient entities", "Denizen = named entity", then noted as identified and named as denizen; a denizen crate only for a good reason | 200, 201 |
| | Speciate and Express; the tick; subscription; places | "Split them"; "Newtype over u64"; "Type the attention set"; "Place-graph handle" | 202 to 205 |
| | Feeding under scarcity; the crowd; the readings; the tolerance | "Pairwise encounters"; "Exact-state histogram"; "The rules' thresholds"; "0.2, with controls" | 206 to 209 |
| | The attention set: pins, groups, examining, the stream | "Any pointable thing"; "Noted exact, rest crowd"; asked which is more co-op friendly, then "In view, up close"; leaning to "detailed but less", then "What's attended" | 210 to 213 |
| | The input: the click; warning; standing orders | "Could you click somewhere to draw attention to a place or thing?", then "Attend to this is default; pick alt meanings by right click"; "One gesture only"; "Grown from attention" | 214 to 216 |
| | The probe's readings; into `Rules`; the duplication; S2 next | "Revisit the fight"; "Move both in now"; "Fold now"; an approximation, the cost, the thirty shapes | 217 to 220 |
| | The fight: strain; a break's direction; the end | "Strain vs bearing, plus cost"; "Breaking up advantages, breaking down disadvantages. No auto win or lose"; "Whichever first" | 221 to 223 |
| | The tract rename's wire | "New wire, old accepted" | 224 |
| | Collapse; regions; mood; the right click | "A level gone"; regions linked to the world's biomass, spilling and merging; "Needs now"; "A ring of acts" | 225 to 228 |
| | The world's ends; W5's second | "Either, player's call"; "Done, but watchable"; "Side by side" | 229 to 231 |
| W5, Eponym's plan | Blows; the solver; the start; the first life | "Foreground hands back"; "Game side"; "Player's pick, society default"; "Any, player's call" | 232 to 235 |
| | Competitions; the two costs; no companion; absorption | "Design sharing now", then "One per tick, by need"; "Cut both"; "Player's call"; "Only on Isocosm" | 236 to 239 |
| | Sharing, reopened for performance | "is it better to allow for concurrent competitions, performance-wise?", then "Concurrent, settled at end" | 240 |
| W5, the other two plans | Posing; co-op in E4; the battlemap; the table's grain | "A telling's manner"; "Two peers in E4"; "Both: an edit"; "Moves too" | 241 to 244 |
| | Sharing at the table; downtime; the faction turn; a sim-off record | "On by default"; "Each player's yes"; "Retires at V2"; "Writes notes" | 245 to 248 |
| | The first calibration; the debug table; two senses of site | "Pathfinder 2e first"; "Warn at open, mark receipts"; "Attachment"; "Situs. Latin, anatomical, no conflict. Good?" | 249 to 252 |
| | E1 and V1 | "Open E1 and V1 now" | 253 |
| | Duplicated shapes; raw receipts | "Lift to the core now"; "Keep raw out of tree" | 254, 255 |
| The outside review | The tick; its unit; the scheduler; the budget; viability; the background | "A fine unit, periods per process"; "World setting, a minute default"; "Index now, per-entity later"; "Count only what runs"; asked whether an ecology can balance before its parts exist, then "Staged, per family"; "Hybrid" | 256 to 261 |
| | The probe; the handover; part roles; the thirty shapes; the hybrid's line; alive | "Vertical probe first"; "Build, retire together"; "Examine first"; asked for option 3's case, then "Wider, fields from the probe"; "Where grouping stops paying"; "The four measures" | 262 to 267 |
| | Amounts; shared ground; the flow record; dev matter; pressures; readings | "2, but with a default set of expressions that are easy to group and to calibrate?"; scramble if they cannot decide, compete if they can; "Buffered outside state"; "A dev source"; "Founding presets"; "Accept all four" | 268 to 273 |
| | Merge; motion; shapes; functions; one catalogue | the player chooses among fork, merge and tale; "Measure option 1 first"; "Add tube, branch, shell, joint"; organ systems riffed from functions; "One catalogue" | 274 to 278 |
| W5, Eponym | Co-op; succession; modes | "Each their own"; "The player chooses"; "The same" | 185 to 187 |
| W5, the VTT | Sim off; uncalibrated rulesets; packs; hooks | "Sim off allowed"; debug or experimental with a warning; "Wait or the gm forces it"; "As suggested hooks" | 188 to 191 |

Administrative: ruling 128 amended Mesocosm's CLAUDE.md portable-profile
line; the readings docket was archived to `archive_docs/2026-09-24/` once
nothing was held; relic, tale and tract joined Mesocosm's terminology; Law
A was amended for materials at Mark's word; and the
[Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md) was drafted as
W5, its M0 done the same day. Then, at Mark's word, work went parallel:
the overlay plan's M1 and an S2 probe of the sim plan opened as lanes, and a
read-only consistency pass over the record and the plans went to the RPG
systems session. Its thirty items were each confirmed against the files and
applied at Mark's word (198), and Mesocosm's CLAUDE.md denizen line was
rewritten, with a sophont line added, at his word (200).

### 8.4 What was got wrong, and corrected

| Claim | Correction |
| --- | --- |
| The funging buffer is "a host's setting" (record §3.3) | It is the world's, in its rules and its log (113) |
| Tau-leaping gives "the same statistics" (record §3.3) | Approximately, with first-order consistency and an error (the aggregation research) |
| Similitude is "owed by the definitions and not by any game" (sim plan §5.5) | Rulesets owe it too (114) |
| Collection, noting, promotion and merging are world transitions (sim plan §3.4) | Noting and promotion are the world's; collection and merging the record's (125) |
| "Branch" for both a new world and a play branch | Fork and branch (126) |
| An item is "a body without agency" | Unless it is a sophont (144) |
| The review's "the CLAUDE.md amendment is unlanded" | Overtaken the same day, when ruling 108 applied it |
| A progress line naming the next question | Changed to "open" when Mark's message redirected the thread |
| A commit staging a moved file's old path | Staged nothing; redone with the new path |
| An accidental tap, "Inherited only", on unplayed lineages | Corrected by Mark the same turn; his correction is ruling 182, and the tap is not recorded as his answer |
| The played slice plan's direct control, kept by W1 the day before ruling 60 | Superseded by ruling 175, with dated notes in the plan, the vessel briefs and the index |
| The rename's R1 put Isocosm into lines dated 2026-09-18, so the record said Isocosm was ruled that day and is a 2015 iOS puzzle game | Isotropy restored in the dated lines, here and in the record and the sim plan (198) |
| `isocosm-overlay`'s README claimed a 0.0.1 name reservation | None exists on crates.io; the line was corrected on merge |

### 8.5 What the session left open

The substance of a technique, which the hagioglyph organ's plan owns; the
tract rename in Mesocosm's phenotype code, a lane of about 104
occurrences; D20's provider over `mere-capability`, W3's to build; the
overlay plan's M2 to M4, proposed and not opened, M1 being done; the S2 probe, being built under rulings 206
to 209; the threats that could collapse a world, which Mark left open
(226); and the §6 forks of Eponym's and the VTT's overlay plans, drafted
the same day by the RPG systems session.

### 8.6 Lessons for the next session

- Put the question directly, as a choice whose options name their
  consequences. The options are the answer's space, not a proposal, and
  Mark's own words beat them.
- Give a recommendation only when asked, as a reading, and rule it only at
  his word.
- Send a derivable answer to Mark as a reading he can object to, not as
  another question.
- A message sent mid-turn is the answer's context: quote it in the ruling.
- Peers commit into the same files: check the tree before each commit,
  stage by pathspec, and never name a moved file's old path.
- A tapped answer can be an accident. When Mark corrects one, the
  correction is the ruling, and the record says the tap was withdrawn.
- Read the plans a new thread consumes before asking: the played slice's
  direct control was a contradiction already in the tree, found only when
  W5 opened.
