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
113 to 173 were given on 2026-09-24, 174 to 235 on 2026-09-25, 236 to 352
on 2026-09-26 and 353 to 380 on 2026-09-27, the last
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

#### 2026-09-27 annotation: the method carried into the new session

Mark supplied the following working method and invited its preservation and
improvement. This annotation supplements the dated account above.

1. Design moves in rounds of up to four multiple-choice questions. Each
   question gives its evidence in one or two sentences with concrete
   numbers, then two to four options stating their commitments, with the
   assistant's recommendation first. A free-form answer may reframe the
   question: answer what Mark actually asked before putting it back.
2. Every design answer becomes a numbered ruling in the design record,
   preserving the question as put, its options, Mark's words verbatim and
   what follows. Anything inferred beyond those words is **Reading, not
   ruled**. Dated text keeps its words; changes are dated annotations or new
   rulings naming what they amend.
3. Rulings and all affected documents move together in the same turn: the
   record, plans, session notes and index. Commit by path and push each
   batch so the documentation agrees with the tree.
4. Evidence precedes the question: read code, measure and verify lane
   claims before presenting them. Reopen a ruling when evidence conflicts
   with it, including conflicts with another repository's plan. The prior
   checks caught the founding plan's six citers, the binding plan's changed
   premise and ruling 322's conflict with mere's D1.
5. Lanes receive briefs quoting their rulings, done-conditions and rules,
   and stop at checkpoints. A choice with more than one defensible answer
   comes back as a fork for the next question round.
6. Nothing reaches main unverified. Verify a lane in its own worktree:
   tests, recomputed receipts, checked hashes and controls that must fail
   when the relevant behavior is deliberately broken. An absence counts
   only when a positive control in the same run demonstrates detection.

**Proposed refinements, 2026-09-27. Reading, not ruled:**

- Give evidence its source revision and status: measured in this run,
  independently checked, reported by a lane, or still unknown. Use numbers
  where supported; never invent precision to fill the question format.
- Order forks by dependencies and say exactly what each answer unblocks.
  Keep unrelated work moving while a required decision remains open.
- Name the fault a control detects and retain both outcomes in the same
  receipt: the working case passes, the deliberately broken case fails.
  Treat a negative result as evidence only within that demonstrated scope.

**Acceptance annotation, 2026-09-27 (ruling 366):** Mark answered the three
refinements, "Sounds good! Shall we proceed? Or would you like to
review/audit first?" The refinements above are now accepted. Their proposal
label remains as the dated record of how they were presented.

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
| | Far rungs; rewrite debt; anatomy brief; board paging | "Park explicitly"; "A doc lane clears it"; "After the probe"; "Open a lane now" | 279 to 282 |
| | The first disturbances | "Disease, An overperformer, A keystone lost, Catastrophe later, to test the criteria of systemic resilience" | 283 |
| | Advance limits; the budget; due events; feeding | "Work, not ticks"; "Count members"; "Due events per group"; "One per consumer" | 284 to 287 |
| | The board's paging: mere's defect; the cap; the CPU side; residency; the pin | "Fix mere first"; "Card-sized cap"; "Page it too"; "View, isometer helper"; "mere's current main" | 288 to 292 |
| | Phases; feeding; the atlas's limits; its budget | "Per group phases if they are in the foreground or nearby, baseline as built?"; drawn across its matter, once a period, a weighted draw; "Accept the shape"; "8 MiB" | 293 to 296 |
| | hagiograph; cleromancy; the old runtime; crate names | "Fold it in"; asked why the VTT depends on cleromancy, then "Optional, off by default"; asked what the runtime is for, then "Retire the crate"; "If they're within the ambit of isocosm vtt, then fair enough" | 297 to 300 |
| | Paging's overflow; the pointer volume's height; the elevation scan | "Keep it"; "Reserve headroom"; "Fix it now" | 301 to 303 |
| | Prey choice; the hunting ranges; hunting's shortage; the draw control | "Both"; "Widen them"; "In Part B"; "Keep it" | 304 to 307 |
| | The doc lane's forks: founding; ProcessDef; general model; dependency ledger | "Archive, carry over"; "Archive"; "Split out the organs"; "Archive and repoint" | 308 to 311 |
| | Surfaces; execution plan; functional loops; world conditions | "Rewrite as a VTT note"; "Retire into the overlay plan now"; "T2 now, T1 and T3 later"; "Archive now" | 312 to 315 |
| | Hunt ranges; refusals; headroom's pixels; the side panel | "Accept them"; "Record, with a bound"; "Find the cause first"; "Compare first" | 316 to 319 |
| | The core hook; the toolchain; burn's pin; the choose seed | "Keep the hook"; "Match mere, with the bump"; "Exact pins in mere"; "The command's seed" | 320 to 323 |
| | Bindings; the self-test's tie; T2 | "Plan it now"; "Fix it with paging"; "Assess first" | 324 to 326 |
| | The bump's target; Eponym's CLAUDE.md; genet's rows | "Retarget to mere's main now"; "Amend as drafted"; "Bump now, fix genet next" | 327 to 329 |
| | T2: authority; fan-out; colliders; navigation; first receipt; its plan | "nisus grows into it"; "A revision log"; "conatus carries the source stamp"; "Stamp each query"; "Inside mere, then Mesocosm"; "A lane in conatus's §2" | 330 to 335 |
| | The traversal; paging's merge | "Fix it in mere"; "After the fix" | 336, 337 |
| | Part B: shape requirement; catalogue; the name; seeding | "On the function catalogue"; "The five in use"; "Rename the process one"; "Grown and Acquired" | 338 to 341 |
| | Conversions; diffusion; the dev source; the flow record | "Check declared ones"; "A kernel now, wired later"; "In the history"; "Every move, when asked" | 342 to 345 |
| | Bindings: premise; picking; home; world; updates; ids; refresh | "Document the shape only" (reading confirmed); "isometer only"; "A module in mere's conatus"; "Bodies only, shared world"; "Both"; "Move it, keep its id"; "A refresh call in conatus" | 346 to 352 |
| | Burn: pre.4; the stopgap; the cubecl patch; turso | asked whether pre.4 lets everything move, then "Pre.4, exact pins"; "No stopgap"; "Retire it"; "Settle it in the migration" | 353 to 356 |
| | Part B: synthesis; dev placement; flow reason; shape keys | "The actor's own lineage"; "Any declared matter account"; "Add the process"; "Part shapes get their own prefix" | 357 to 360 |
| | The CPU walk; the start offset; T2's timing; X1's defaults | "Call modulus's walk"; "Adopt the clamp"; "After the pre.4 migration"; "Keep as built" | 361 to 364 |
| | Repin the traversal fix now or bundle it with genet and pre.4? | "You can repin but communicate with the isocosm agent"; the intended agent remains unresolved | 365 |
| Method | Evidence status; dependency-ordered questions; fault-specific controls, offered as three additions | "Sounds good! Shall we proceed? Or would you like to review/audit first?" | 366 |
| Coordination | Lane A, the RPG session or both before repinning? | "Ah, the rpg session isn’t active now. But you can orchestrate the repin/rest" | 367 |
| Paging overflow | Standing nearest-terrain fallback with a current omitted count, or capacity error? | "Keep the nearest terrain and report the current number of omitted bricks, amending 301 (recommended)." | 368 |
| Paging budget | Live device-bounded 8 MiB default, or fixed host budget? | "Expose a setting, default 8 MiB, bounded by device limits; changing it rebuilds the atlas (recommended)." | 369 |
| Atlas allocation | Upfront allocation, or growth with full re-upload? | "How costly is replacing the texture and uploading only the retained bricks that change? Is that possible?" Allocation remains open while the premise is checked. | 370 |
| Sim flow handoff | Until drained with host draining each tick, or a required per-tick API? | "Require a per-tick handoff API before accepting checkpoint 5." | 371 |
| Budget persistence | Small local per-device preference store, or session-only setting? | "Save the atlas budget locally per device; add a small local preference store (recommended)." | 372 |
| Allocation after measurement | Upfront budget, growth with full uploads, or tighter batched copying measurement? | "Measure a more tightly batched GPU-copy path before deciding; keep allocation open." | 373 |
| Allocation after tighter batching | Upfront budget, growth with full uploads, or further copy research? | "Agreed. The numbers have spoken." Accepts upfront allocation. | 374 |
| Pre.4 persistence | Disable with a manifest-only patch, keep persistence, or measure Unix first? | "I accept your two recommendations. Wow. Much larger dependency count than I figured." Disable; amend 355 and retire the identity helper. | 375 |
| Distillery migration gates | Full verification, defer two-peer, or compilation/lease only? | Same answer verbatim as 375. Full gates, with a separate fixture repair verified on pre.2 first. | 376 |
| Pre.4 allocation guard | Preserve five comparisons or add service identity? | "Add service ID as a sixth comparison; also verify the new rejection case before proceeding." | 377 |
| Pre.4 missing dependencies | Allow needed crates.io downloads, only the first missing package, or remain offline? | "Ok." Accepts needed downloads for the pinned migration, recording them and preserving Git pins. | 378 |
| Text bounds before merge | Fix explicit-height text and inline decoration bounds now, or merge normal text and defer both? | "A". Fix both with measured fixtures and verify the combined change before merging. | 379 |
| Alias broadcast repair order | Fix pre.2 separately then carry into pre.4, or fix only the migration lane? | "A". Give the pre-existing defect its own verified pre.2 commit, then carry the correction; allocation checks and tolerances stay unchanged. | 380 |
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

**2026-09-28, consumer repair checked; build space blocks completion.**
Mere's original 12-pixel hover test and all 44 Rootstock tests pass on the new
Genet revision. Each of four consumer fault controls detects its intended
defect; exact source restoration passes 44 again. Native/web locked graphs
preserve every normalized node, feature and edge, with four effective source
controls and all pre.2 patch bytes preserved. The remaining workspace, host,
native and web gates are not run yet. Available space is 1.54 GiB, below a
reasonable margin for the observed remaining artifacts, so new builds pause.
Automatic approval review rejected deletion of the verified 59.58 GiB stable
Genet incremental cache with only "blocked by policy". No deletion or alternate
cleanup was performed. A manual cache-removal question is pending with Mark.
Mere primary stays uncommitted; Isometry repinning and pre.4/S13 remain held.

**2026-09-28, Genet scroll source accepted.** Tested source `7a60ad79` is
pushed; `ff52bd2b` adds publication documentation only. The seven focused
fixtures pass, all three fault controls detect their intended defects, and
restored Livery/Buckram passes 883 tests with six existing ignored. Root and
independent review verified seal `865c6a8f`, its 88 files, 104 source entries
and nine owned paths. Formatting-line and font-content bounds remain distinct.
Mere primary is adopting the new owner query and exact revision, including
the known optional-description API fields; the original scroll test remains
the consumer gate. The pre.4 branch stays at held checkpoint `387a8dd2` until
repaired Mere main is verified. No new ruling or S13 acceptance follows.

**2026-09-28, held branch checkpoint preserved.** Mere `387a8dd2` records
the reviewed reconciliation with both parents and all sealed source bytes
preserved. It is pushed and clean. Root and independent review checked
1,986 source/doc entries, 141 receipts and 11 locks under seal `6400226a`.
The scroll failure remains an acceptance blocker; this checkpoint does not
release S13 or main integration. The separate Genet repair's focused tests
and fault controls pass, while broader verification remains in progress.

**2026-09-28, published scroll regression discovered.** Reconciliation's
broader Rootstock suite has 40 passes and one failure: a 200-pixel line in a
180-pixel scroll container clamps a requested 12-pixel offset to zero before
hover. The same unchanged test fails on untouched Mere `5ce144ff`, so this
is not introduced by pre.4. Genet and Rootstock both derive nested scroll
extents from node fragments, which now represent font content separately
from formatting lines. The original retention test remains intact while the
owner-side layout repair is measured. Other affected gates pass, including
the workspace, Distillery and leases, Djinn, remote fixture, web build and
17 Mesquite tests. The remote lock preserves its original Syn dependency
after an unchanged locked candidate proved Cargo's extra edge change
unnecessary. The published 187/0 row result stands within its stated scope;
reconciliation acceptance, S13 and main migration integration remain held.

**2026-09-28, text publication accepted and pushed.** Mere `5ce144ff`
publishes the tested Genet `7b48f94d`; Isometry `5da804eb` adopts both across
its seven owning manifests. Root and independent review checked the 106-file
Mere and 126-file Isometry seals. Both actual-pin row runs measure 187 rows
with zero short, and the test now runs ordinarily. Native/Wasm Mere checks
and all four Isometry consumer workspaces' all-features/all-targets checks
pass. The side-panel plan records exact source audits, fault controls and
the retained optional lineages. Eponym's check includes its unchanged local
Renderling edits and clean Crabslab; it is not portable renderer acceptance.
Lane M is reconciling this exact main before S13. Its refreshed workspace
check exposed the known two-line Reader accessibility adapter; the bounded
lane adoption is released while the primary Reader bytes remain preserved.
The prior publication-pending entry below is historical. Rulings remain 380.

**2026-09-28, remaining migration compile/GPU checkpoint accepted.**
Lane M completed all eight standalone/nested builds and the remaining
Conatus, ESP and Numen correctness gates. Root and independent review checked
1,986 source/doc hashes and 81 receipts in seal `238b5909`. The remote fixture's
pre.4 API adaptation passed its rebuild; unchanged thresholds, source scope and
the two initial compile failures remain explicit in Mere's owning plan.
The separate lane commit is pushed at Mere `a7c477e7`. S13 and current-main reconciliation
still precede migration integration. Current Genet publication continues on
Mere, then Isometry; primary consumer locks have not yet been promoted.
The completed text worktree remains because automatic policy rejected cleanup;
its historical binaries, fingerprints and lock have been preserved separately.
The pending additional NetRender coordination question remains unanswered.

**2026-09-27, text source integrated.** Genet main/origin now contains
Lane L's fix and retained-motion combination fixture at `7b48f94d`.
Root and independent review checked both sealed receipts: 876 affected
passes (6 existing ignored), 200 boundary passes, recompiled fault controls,
and 187 current-family rows with zero short versus 39 on published Genet.
The local override preserves the rest of the dependency graph; a portable
Mere/Isometry repin remains a separate gate. The source-integration and
new-family diagnostic waits below are superseded. Lane L is retained briefly
for historical embedded-output retirement. Lane M's nine Conatus cases and
ESP synthetic/real MiniLM parity are independently accepted; standalone builds
and final Numen review remain pending. S13 remains unreleased. No new ruling.

**2026-09-27, real-panel capture and coherent repin landed.** Isometry
`0427e982` adopts Mere `ac41628a`, Genet `92b249af` and NetRender `9607d16`
for the separately approved capture task. The real paused panel's packet
preserves three font faces, 73 runs and 1,228 glyphs; final Classic replay is
byte-identical to the native paired PNG. Root verified the 13 source blobs,
30 artifacts and both executable hashes in the committed source seal.
Missing-font and overwrite rejection controls are recorded. Default root,
Eponym and 111 Isocosm tests pass. The bench README owns commands and exact
scope; embedded system-font packets stay local. This closes the capture/replay
slice, while Hybrid text, live external textures, native performance, optional
Cleromancy and Lane L verification on the new family remain open. The capture
lane released the shared Isometry target and created no isolated resources.

**2026-09-27, bounded pre.4 carry verified.** Mere branch commit `8d308572`
is pushed. The correction now passes the
same nine direct cases and three-failure fault control on pre.4, with exact
restoration, nine identity checks and 96 full Seiche passes (one existing
ignored). Independent review and root hash checks agree. The six-field guard,
seven prepared files, both locks and original tolerances are unchanged; this
carry needed no dependency or rendering update. It remains a separate source
commit on the migration branch. The wider migration matrix, ESP/Conatus gates,
S13 and current-main integration remain open. Lane M stays retained for those
gates; Lane L remains retained while concurrent Genet primary work continues.

**2026-09-27, separate pre.2 repair landed.** Mere main `a016f86f` carries
ruling 380's verified alias-broadcast correction. Nine direct launcher tests
pass, the mapping-only fault fails the three broadcast cases, and exact
restoration passes all nine. Full Seiche passes 96 tests, one existing ignored,
including both formerly failing GPU force comparisons at unchanged tolerances.
Root and independent review checked the code and receipts; both locks and
reader WIP are preserved. The pre.4 carry is now released only for this
correction and its tests, preserving its six-field guard, prepared files and
current rendering closure. That carry needs its own verification; broader
migration and S13 acceptance remain open. Isometry pins are unchanged.

**2026-09-27, text lane final checkpoint.** Lane L `f81083b8` retains
production source `9b730e7b` and passes 200 additional boundary tests on its
published Netrender9607 closure. Root recomputed the receipt and current
source hashes and counts. The separately qualified local Isometry check
passes 187 rows; the original consumer closure fails 39 rows in the paired
control. The local check uses the lane's 19 Genet packages and four cached
Netrender c8 packages, so portable9607 Isometry acceptance remains open.
Primary pins/lock are unchanged. Genet integration is held while another
agent owns retained-motion edits in primary; the completed lane is retained.
The Genet line-box plan owns the detailed evidence and qualifications.

**2026-09-27, repair order answered.** Ruling 380 chooses a separate tested
pre.2 repair, then carries the verified correction into pre.4. Allocation
identity checks and numerical tolerances stay unchanged. Primary Mere's
unrelated reader WIP must survive; the migration lane retains its prepared
changes while the pre.2 source and controls are independently reviewed.
The answer authorizes the repair sequence, not numerical acceptance.

**2026-09-27, alias-layout repair question.** A three-point isolated GPU
subtraction has six wrong entries out of nine, while the CPU matches scalar
arithmetic. Our pre.2/pre.4 shared-buffer branch omits the second operand's
broadcast reference shape. Repair order is now a user fork: separate pre.2
fix then carry it into pre.4, or migration-only repair. Production changes
remain stopped; identity checks and tolerances need not change. The local
text consumer test proceeds with a separate coherent test lock and remains
distinct from portable rendering-closure acceptance.

**2026-09-27, baseline comparison and renderer boundary.** Both Seiche
failures reproduce on pre.2 with the same reported error; source restoration
and receipts are verified. This identifies a pre-existing failure without
closing numerical acceptance. Lane L passes independent review, but consumer
resolution requires a coherent Netrender/Vello closure; its failed dry run
changed no primary pins or lock. Separate sparse-backend review confirms
that live producer-image acceptance and equal logical tick/action comparisons
need explicit gates in the owning plan. The sim's atomicity and separate
surfaces remain intact. No new design answer is inferred from these reviews.

**2026-09-27, text checkpoint and numerical stop.** Lane L implemented both
379 corrections and reports 867 passing CPU tests, with two detected broken
controls; root verified all 71 raw hashes. Independent review and Isometry
consumer verification remain. Lane M stopped at two finite Seiche GPU parity
failures, one at 1.9973466 against the unchanged 0.001 limit. The next
diagnostic compares pre.2 with identical finite/length checks, preserving
primary WIP. No new ruling, relaxed tolerance or migration acceptance is
inferred from the failure. Other completed matrix receipts remain qualified
to their tested source; remaining GPU/nested/headed gates stay open.

**2026-09-27, dependency checkpoint accepted.** Lane M S5-S8 passes
root and independent review. The root lock has 1,657 packages from a fresh
1,648 baseline; Git identities are fixed, one wgpu remains, and the checked
production graphs omit persistence and Turso. The same detector finds them
in the retained standalone positive graph. Conatus compiles and both
focused lease tests pass. S9-S12 matrices and nested locks can proceed;
headed/full two-peer acceptance and source integration remain open.

**2026-09-27, text scope answered.** Ruling 379 extends 329: finish both
explicit-height text and inline decoration bounds, add measured fixtures,
and verify the combined change before merging Lane L. Mark chose A.
The existing lane resumes; source reconciliation must preserve main's
generated-text behavior. The answer selects scope, not a new test result.

**2026-09-27, fresh text evidence.** Lane L passes three existing CPU tests,
including 33 Lato sizes; Arial 16px text measures 17px within an 18px line.
Root checked ten raw receipt hashes and five source/lock/font hashes.
Explicit-height text and inline decoration still use line-based bounds by
source inspection, without fresh measured fixtures for either. The scope
question is pending: correct both before merging, or merge normal text and
defer them. Historical browser measurements, WPT transitions and the old
187-row Isometry receipt have not been refreshed. No new ruling is inferred.

**2026-09-27, guard checkpoint verified.** Nine tests pass. Deliberately
removing service equality yields seven passes and exactly two service
failures; restoring the reviewed source returns nine passes. Root and
independent review checked the paired evidence. Seven needed registry
archives were fetched under 378 and their hashes checked. Standalone test
defaults explicitly enable persistence, so production dependency absence
still needs its own consumer-graph gate. Remaining patch/manifest/lock work
can proceed on Lane M; full migration acceptance remains open.

**2026-09-27, downloads authorized.** Ruling 378 permits crates.io
dependencies needed for the pinned migration, recording downloads and
preserving Git revisions. Lane M resumes the guard checkpoint. Tests and
the deliberately broken service comparison still need execution evidence;
the approval does not change the full migration acceptance gates.

**2026-09-27, first migration checkpoint.** The repaired remote fixture
passes its locked pre.2 baseline, independently checked. The runtime and
six-comparison guard patches are prepared and source-reviewed. Guard tests
and their deliberate fault control remain unrun: offline resolution needs
the uncached `cubecl-spirv 0.11.0-pre.4`. The plan requires returning with
that concrete download request; the question is pending. Main's production
dependencies remain pre.2. The complete Distillery gates remain required.

**2026-09-27, allocation guard answered.** Ruling 377 keeps the five
existing comparisons and adds service identity. A verified mismatch
rejection is required before proceeding. Mere's migration plan carries the
implementation gates; no execution result is inferred from the answer.

**2026-09-27, pre.4 policy answers.** Rulings 375/376 accept disabling
CubeCL persistence with a retained manifest-only patch and the complete
Distillery acceptance set. The dependency count is the resolved lockfile,
not every binary's compiled set; the measured reduction is 55 packages.
Unix cache benefit remains unmeasured. The fixture repair needs a pre.2
baseline before migration. Mere's plan owns execution; the allocation-guard
stop-rule disposition is still open. No manifest or lock moves in this
documentation batch. The parked pre.3 lane remains preserved.

**2026-09-27, paging integrated.** Lane E's final gates and independent
review pass: 411 root and 293 shared tests, all three consumer checks,
and six native sessions. The chosen budget is allocated upfront, saved
per device and rebuilt when changed; current omissions are displayed.
The live control changes 1 MiB / 247 omitted to 2 MiB / zero, and restart
restores 2 MiB. Root checked receipt/source hashes and image controls.
The integration receipt retains source qualifications and timing limits.
Pre.4 setup and Genet text decisions remain open; no new ruling is inferred.

**2026-09-27, allocation decided.** Ruling 374 accepts allocating the
chosen budget upfront, after the reviewed tighter-batching experiment.
The configurable, persistent per-device budget and current omitted count
stand. Lane E proceeds through remaining integration gates; budget changes
rebuild the atlas, while ordinary population changes do not grow it.

**2026-09-27, tighter batching measured.** Ruling 373's experiment is
complete: the new batched copy improves the larger replacement, but full
uploads remain faster in both measured cases (0.191/0.598 ms against
0.348/1.584 ms). Root checked raw hashes and recomputed the statistics;
198 complete readbacks and the three fault controls support correctness.
The board plan records exact scope and limits. Allocation remains open,
and the new evidence returns the choice to Mark. No answer is inferred.
Independent read-only review confirms the result and controls; the question
offers upfront allocation, growth with full uploads, or further copy research.

**2026-09-27, checkpoint accepted.** Checkpoint 5 lands with 371's per-tick
API, independently checked fresh evidence, 111 final-source tests and the
Mesocosm consumer check. Physiological scope stays with checkpoint 6.
Ruling 373 asks for tighter batching measurements rather than selecting
allocation; paging's settings and omission reporting remain on its branch.

**2026-09-27, allocation evidence.** The replacement probe answers 370:
GPU copies can preserve unchanged residents, but the measured two-submit
path is slower than full uploads in both cases despite lower CPU bytes.
The question is put back with an option to measure tighter batching;
allocation remains open. The board plan keeps the numbers and limits.

**2026-09-27, flow answer.** Ruling 371 requires the per-tick API before
checkpoint 5 acceptance. Allocation remains under inquiry (370); the
budget's local persistence is a new follow-up because the host currently
has no application preference store. The reserve-path reading is under
independent review.

**2026-09-27, next answers.** Rulings 368 and 369 settle standing counted
overflow and a live budget setting. Allocation and sim flow retention remain
open. Mark explicitly invited continued questions and coordination with the
independent review chat; its next read-only pass checks the reserve-path
reading before checkpoint 5 integration.

**2026-09-27 continuation.** Traversal repinned under 361, 362, 365 and
367, with the checks and picture receipts recorded in the board plan.
Paging's strengthened release gate then passed on its branch, including
the old-walk fault control and a same-run empty-terrain GPU control; its
policy questions and remaining integration checks still precede merge.
Checkpoint 5 was independently audited on Lane A, still unmerged; the sim
plan distinguishes preserved raw evidence from fresh output compared in
memory. Three paging policy questions and flow retention are awaiting
Mark. The remaining reserve-path reading, pre.4 setup and two text forks
follow. Eponym's device-limit propagation was already ruled (295), so it
returns to implementation rather than another vote. Genet's text branch
and main have diverged, and main is dirty; the old fast-forward instruction
cannot be used. No new design ruling follows from these audit findings.

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

## 9. The one-game hypothesis session, 2026-09-28

### 9.1 How it started

Mark, verbatim: "Hmmm. Let's imagine this is one big game. Critter lineage
roguelike, denizen adventure rpg, sim world vtt, all isometric: rotatable,
orthogonal projections. There are many semantic axes to examine and vary
this hypothetical isocosm with: scale (local, regional, world, individual to
group to polity), detailed representation to functional abstraction, time,
inert to sentient to sapient to omniscient, mundane to magical, harmony to
order to chaos, definitely more as we have discussed. Characterize: what
would efficient, effective sim, rendering, and game engines that could
handle such procedural world and asset generation look like? Compare that
and prior art to what we have in our stack, looking for what's missing.
Upon review, let's see where our current plans and this hypothesis
differs."

### 9.2 What the assessment found

Read: the record's 380 rulings, the sim plan, the presentation plan, the
three overlay plans, and the code at isometry `bfa1b36` and mere
`5ce144ff`. Each point below was checked in that code unless it is dated or
marked otherwise.

- The hypothesis is mostly the ruled design. Ruling 1 already makes
  projection a distinction made on top of the sim, and the games are
  overlays that foreground rungs (35).
- Mark's axes read as ranges of representation. Every entity sits somewhere
  on each, and the engines' work is to hold the cheapest representation
  that answers what is asked, move things along an axis without
  contradiction, and show each position legibly.
- The gaps, ranked by how much the hypothesis leans on them:
  - the sim's site graph and the voxel bricks are disconnected models:
    isocosm's `Site`, `Route` and `Location` carry no volume, and nisus is
    ruled the voxel authority (330) but unbuilt;
  - time at scale: the 2026-09-25 baseline grew as n^2.09, 117 s a tick at
    10,000 members, and a century at the minute clock is 52.6 million
    ticks;
  - near to far is four renderers (the isometer scene, the VTT's DOM board,
    the VTT atlas on sprigging's canvas, Mesocosm's minimap on the HUD
    lane) with no shared level-of-detail ladder;
  - no host carries several overlays, though all four workspaces now pin
    mere `5ce144ff`, which makes isometer's manifest comment on divergent
    pins stale; the divergent patches are the VTT's p2panda set and
    Eponym's renderling leftovers;
  - no texel snapping, none of ruling 22's lighting, no glTF anywhere in the
    wing, no building or settlement generator with isoscape unfounded, and
    no audio engine in the wing, though woodshed has a cpal engine and a
    Firewheel engine and mere has `mora` and `pipit`.
- `Method { Inert, Reactive, Deliberative, Normative }` exists
  (`shared/isocosm/src/schema.rs:27`) and only `Inert` and `Reactive` are
  used; the sim has no polity processes.
- Eponym's client already runs a perspective camera in its default
  `r1-proof` profile, per its `CLAUDE.md`; isometer's `SlabCamera` is
  orthographic only.
- Tensions found in the tree whatever the hypothesis: the founding record's
  "never by a shared running world instance" beside its 2026-09-18 "one
  clock" amendment; the tabletop `CLAUDE.md`'s locked-lens don't beside the
  presentation plan's ruling 12, "fixed or free"; Eponym's overlay plan
  keeping first person as a setting beside the presentation plan's ruling
  1, "orthographic" (resolved by ruling 382); and Eponym's `CLAUDE.md`
  still saying the vessels do not share "a schedule", the wording the
  founding record dropped on 2026-09-18.
- Prior art not in the sim prior-art brief, from general knowledge and
  unverified: multi-resolution modelling in defence simulation (Davis and
  Bigelow, RAND, 1998; Reynolds, Natrajan and Srinivasan's
  multiple-representation entities, 1997), whose named hard case, a
  detailed unit meeting an aggregate one, is the played critter meeting a
  cohort; Sunshine-Hill's alibi generation (2010) and Brockington's
  level-of-detail AI for Neverwinter Nights (2002); Zeigler's DEVS; Spore's
  animation retargeting (Hecker et al., 2008); FFT's and Tactics Ogre's
  quarter turns, Triangle Strategy's rotated diorama, SimCity 2000's and
  RollerCoaster Tycoon's four baked views, and t3ssel8r's texel-snapped
  pixel rendering; Stonesense and Armok Vision over Dwarf Fortress;
  Veloren's rigid-segment voxel figures; semantic zoom (Perlin and Fox,
  1993).

### 9.3 The questions and rulings

| Question as put | Ruling |
| --- | --- |
| One executable with modes over one world save, or three sovereign products? | 381: one host, modes |
| Which projection family should the modes share? | 382: one default view, isometric with quarter turns, precluding no other |
| What is harmony → order → chaos in sim terms? | 383: ecological states, "not so important" |
| Is the omniscient end of the mind axis a seat someone plays? | 384: some divine figures peek behind the curtain, which is rule-bending magic |
| Apply the founding record amendment for 381 as drafted? | 385: applied as drafted |
| Apply the `CLAUDE.md` drafts? | 386: both applied, the tabletop's camera line and the "schedule" wording |
| What pitch is the wing's default view? | 387: the 2:1 dimetric, 30° |
| Does Mesocosm open in the wing's default view? | 388: CP1 compares both before it is ruled; the question's "pending" corrected to CP1's review |

### 9.4 Drafts waiting on Mark's word

Wing law and `CLAUDE.md` change only on Mark's explicit word. Neither
draft is applied. *Later the same day both were applied at his word,
rulings 385 and 386, the second also striking "a schedule" from Mesocosm's
and Eponym's `CLAUDE.md` identity lines.*

*For ruling 381, in the founding record's "Each vessel is a mode of the same
peopled history", after the paragraph ending "require another vessel to be
running":*

> **Amended 2026-09-28** (wing design record, ruling 381): one host carries
> the three games as modes over one world save, a build may carry one mode
> or all three, and players in different modes may share one trunk. What
> survives of the limit is its guard against coupling as obligation: no
> mode may require another mode's code to run, and each game keeps its
> genre, its verbs and its care granularity. As founded this read "joined
> by a shared *history*, never by a shared running world instance".

*For ruling 382, in the tabletop's `CLAUDE.md`, Important Don'ts, replacing
the camera line:*

> - The wing's default view is isometric with quarter turns (wing design
>   record, ruling 382). The tabletop ships its locked 2:1 lens today and
>   reaches quarter turns through a plan and render lane of its own; free
>   yaw, first person and over-the-shoulder views are not precluded and each
>   needs the same.

### 9.5 What the session left open

- The default view's pitch: the VTT's 30° dimetric, which keeps 2:1 pixel
  tiles, or true isometric at 35.26°.
- Whether Mesocosm opens in the default view or keeps the terrarium section
  as its opening view.
- Room for perspective views in isometer's camera (382).
- Where the one host lives and how a build profile is expressed (381).

*Answered later the same day:* the pitch is the 2:1 dimetric, 30° (387),
and Mesocosm's opening view waits on CP1's comparison (388). *Reading, put
to Mark the same day without objection:* isometer's camera reserves room for
a perspective projection now and builds it when a mode needs one, with
Eponym's `r1-proof` perspective camera as the donor. Still open: where the
one host lives and how a build profile is expressed.

### 9.6 Gap 1, the spatial spine

Mark, verbatim, after the rulings above: "Let's start on 1?", the first gap
of §9.2, connecting the sim's site graph to the voxel world.

The assessment found, at isometry `d80163b` and mere `origin/main`
`a31b9a14`: Isocosm's sites are an abstract graph whose `terrain_seed` is
read nowhere; `Ground` is one bounded, heightfield-grown volume in a crate
that renders nothing; nisus is chunk mechanics without a world store; the T2
lane ruling 335 placed in mere's conatus engine plan §2 is unwritten, and
the pre.4 migration T2 waits on is unfinished (production on pre.2, S13
open); the only site-shaped terrain, Mesocosm's 65×65 relief, cannot tile;
ruling 195 puts places fourth and the sim is on the first family; mere's
checkout holds a peer's uncommitted work fifteen commits behind, so the
session read `origin/main` and edited nothing there. The drive, low at
1.54 GiB earlier in the day, had 468 GB free.

| Question as put | Ruling |
| --- | --- |
| How does starting on gap 1 run against rulings 195 and 363? | 389: design now, slice around |
| Where should the spine's plan live? | 390: the place-graph engine plan, rewritten |
| How do neighbouring sites meet without seams? | 391: edge profiles |
| Who owns the lift from site to volume? | 392: Isocosm lifts |

Drafting the plan found ruling 392's option text meeting itself: models
"beside the Terrain seam" sit in isometer-core, which its no-render-crate
clause keeps out of the sim's graph, though isometer-core renders nothing.
The finding went back to Mark as the plan's decision 1, beside the skeleton's
form, the first shapes and the lift's grain (§A.6 of the plan).

| Question as put | Ruling |
| --- | --- |
| Where do the terrain models the lift calls live? | 393: in Isocosm, amending the isoscape plan's ruling 11 |
| How is a site's skeleton held? | 394: keyed entries in its conditions |
| Which world shapes and site footprints does the first slice build? | 395: square sites on planes, rings and tori |
| At what grain does the lift work in the first slice? | 396: chunks at power-of-two cell sizes from the start |

With these the spine plan's SP0 is done; SP1 is next.

SP1's brief (the plan's §A.8) then put three forks, after a baseline of 111
passing Isocosm tests:

| Question as put | Ruling |
| --- | --- |
| Where does each adjacency's frame relation live? | 397: on the route, as an optional border |
| How does the map family enter founding? | 398: an optional part of Founding |
| What space do SP1's seeded draws cover? | 399: scale-free, 2×2 to 16×16 grids and 256 to 2,048 base-unit sides |

SP1 landed the same day at `6f25a89`, with its receipt. One lesson on the
way: a control that fails is not yet proof of the mechanism it guards. The
per-site corner control would have failed even against a broken corner walk,
so a positive count of the grid's corners was added beside it (the spine
plan's Findings).

SP2's forks came next:

| Question as put | Ruling |
| --- | --- |
| What must a lifted site give back when read up? | 400: exact where it can be |
| How is a site's interior built between its borders? | 401: a Coons patch |
| What does a lift return? | 402: columns per chunk |
| What do a lift's voxels name as material? | 403: world-local ids |

Working 400 out before coding showed that an exact mean over every
base-grain column has no closed form if each column rounds on its own; the
brief keeps it exact through a lattice whose sums are closed, recorded as a
reading beside the rulings.

SP2 landed the same day at `38ea90f`, with its receipt. The SP1 lesson came
round again: two of SP2's instruments were incomplete on first writing, a
relief check with no control and a water test that could pass without water,
and both were fixed before the commit.

### 9.7 Construction, magic and condition carryover

In the parallel read-only design conversation, Mark linked
[Moirai](https://github.com/theor/Moirai/tree/d29de5dc1931965c73ae66a069e7c8fd51dc5df4)
as comparison material, explicitly not a request to copy or adopt it.
Source review found useful comparisons in rule authoring, scheduled
transitions, property-gated reactions and causal inspection. No local
benchmark or integration was run.

The subsequent plan/code review found the one-host and default-camera
decisions already recorded, the spatial spine active, and a gap between
symbolic sim parts, the concrete body pipeline, functional charge routing
and a general nested construction model. The portable body plan was already
marked for rewrite; the sim plan did not yet distinguish capability,
repertoire and proficiency. Mark then answered three design questions and
authorized the documentation pass with "Let's document!" The earlier
read-only restriction was lifted for these plan edits, not for code work.

| Question as put, with the recommendation | Answer and ruling |
| --- | --- |
| How expressive may a generated operation become? Compose supported fundamentals, with new operations a separate extension mechanism | 404: Mark expanded the recommendation to explainable, surprising combinations; equivalent exchange with strongly escalating energy; mod-defined and sim-generated effect scripts reading world conditions and generator outputs; ruleset spells as distinctive exemplars alongside generated additions |
| When does geometric detail affect the rules? Construction declares functional properties, geometry supplies named measurements where mechanics require them | 405: Mark agreed with the recommendation |
| How are consequences translated between substantially different embodiments? Explicit adaptation and provenance, with universal translation left open | 406: choose conditions for the next life, granularly where possible; randomized or player-configured; an incident may become a detriment, an absent part, or a related benefit/characteristic ability |

The exact answers and their consequences are in the wing design record.
Ruling 407 consolidates earlier accepted answers from this conversation:
nested meaningful components, material and conceptual operation composition,
circumstantial tradeoffs, capability/repertoire/proficiency, and perception
and self-belief as part of the mind. These were prior decisions being
documented, not another questionnaire.

The documentation homes are the record's current architecture, the sim
plan's capabilities/magic/record sections, the body contract's current
refinement, the functional generation plan's composition design, the organs
plan's journey distinction, and isomere's host target. Historical receipts
remain scoped; no new runtime, body schema or playable loop is claimed.

**Open after the answers:** the cost curve and definition of a composition
level; repeated/sequential effects versus one compound effect; persistent
effect timing, interruption and feedback; scripting/lowering and bounded
execution; CoreRPG and individual ruleset adapters; casting methods; the
shared construction schema and multi-part action support; carryover defaults,
admissibility and costs. Runtime schemas, dependency tracking and concrete
UI flows are proposed design work rather than additional user rulings.

**Corrections to preserve:** "a magnitude" does not yet mean tenfold, and
the tenth-level illustration does not set a tabletop spell rank. Cantrip
availability is not automatic learning or zero cost. Broken or very strong
builds are welcome when fair and explainable; the task is not to prevent all
powerful combinations. Carrying an incident's history is different from
carrying its current wound. Omitted conditions do not erase history.

### 9.8 First integration batch

Mark authorized orchestration with "Ok. Let's proceed. Orchestrate away."
The batch opens SP3's retained bench terrain connection under rulings 408
and 409. Body-to-scene preparation and shared-owner plan reconciliation run
beside it as documentation work. It does not open M2 body absorption, CP6,
generic physics bindings or T2 implementation.

The body-contract source audit found that Isocosm's symbolic part IDs and
site membership do not yet supply complete construction geometry or local
pose. Its proposed mapping preserves full source IDs, distinguishes body
and geometry revisions, and rejects stale picks. A display arrangement may
be useful for inspection but cannot stand in for authoritative movement.

Mere's Conatus owner plan now carries the previously ruled generic body
table and T2 store/revision path, committed separately as `4fbcb727`.
Implementation gates remain unchanged. The paging/migration conversation
owns its concurrent Mere index and platform changes; SP3 retains the
published Mere `5ce144ff` dependency while that work is verified.

SP3 passed native verification on 2026-09-29. The retained bench now presents
the border and overview, catches a deliberately mismatched source, restores
the original viewport exactly, and suspends/resumes an existing trial's epoch
advance. The first native run caught a renderer-capacity mismatch that CPU
tests had missed; admission now counts actual occupied bricks before
allocation. The [receipt](../testing/bench/receipts/2026-09-29/spine/SP3_TERRAIN.md)
preserves that failure alongside the corrected passing runs. This completes
the approved terrain connection and both documentation lanes, not the later
body, edits, paging or shared-store implementation.

**2026-09-29, scroll consumer verification complete.** Mere's repaired primary
was published at `32edc2ad`, preserving the Conatus owner-plan commit `4fbcb727`.
The seven Isometry manifests now use that Mere revision and tested Genet
`7a60ad79`: the ordinary side-panel test holds 187 rows, all four workspace
checks pass, and Isocosm lift/spine plus Mesocosm spine pass 27 tests. The
[side-panel plan](../../design_docs/2026-09-03_side_panel_diet_plan.md) records
the dependency/source controls and inherited renderer qualifications. SP3's
native captures retain their original pins; they were not replayed here.

Mere's separate semantic-selector work subsequently published `99e44853`.
Pre.4 is reconciling it after local pre-build checkpoint `0ea65fb6`; neither
that checkpoint nor this consumer repin accepts pre.4 or S13. The later pre.4
consumer handoff remains distinct. No new ruling was needed for this execution.

### 9.10 Pre.4's patch comparison and conditional retirement

The migration conversation reported that pre.4 passed the same 21 browser
cases with and without the `burn-cubecl` patch, unlike August's pre.2
control. It offered (A) comparing historical pre.2 in the same current
browser before deciding, (B) retiring the patch on the new result and then
running the remaining checks, or (C) keeping it as a precaution with an
explicit control amendment. Mark answered: "I suppose A, then B if we can".
The exact question, options, answer and source are recorded as ruling 410.

The comparison comes first. *Reading, not ruled:* retirement is authorized
if the evidence supports it, with the remaining migration checks still
required. The answer does not automatically retire the patch, validate
untested launcher shapes or accept S13. The existing Mere migration lane
owns this work. Isometry's accepted `eb2a367` dependency receipt and SP3's
historical native receipts keep their exact original scope and pins.
