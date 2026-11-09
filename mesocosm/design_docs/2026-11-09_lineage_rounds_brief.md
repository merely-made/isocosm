# Lineage rounds: classes, recipes and the edit between rounds

**Status:** 2026-11-09: put to Mark as five card rounds on 2026-11-09, opened
by his roguelike framing (§1). Its eighteen answers are cases 1198 to 1215
(§7). Doctrine refined from them the same day: Bodies 1, Bodies 6, Ecology 1
and Mesocosm 1 to 3 refined, Record 7 applied, Mesocosm 6 new. A second pass
the same day (§10) answered eight more as cases 1216 to 1222, and a third pass (§11) four more as 1223 to 1226: the playstyle's
word is *kingdom*, not *class* (1219), so read §3 to §7's "class" as
"kingdom"; the five are named plant, animal, fungus, microbe and world-class
(1218). Built: the sim's producer, consumer or decomposer reading renamed
`Trophic` (1222). The rest is unbuilt; open questions are in §8.

**For:** the mechanisms under Mesocosm's rounds, settled before any
per-class, per-role or per-form content, as Mark asked: what a class is, what
a lineage's recipe is, how a lineage changes between rounds, and how a round
starts, passes and ends. Code paths are under `shared/cosmograph/`.

## 1. The idea, in Mark's words

> for Mesocosm, defining those [mechanisms] per kingdom/role/form should be
> done after establishing decent mechanisms… look. In an rpg, you have
> classes, yes? Imagine flora, fauna, myco, micro, and macro are classes of
> entity. Imagine there are ecological roles entities can play depending on
> their kingdom (class), genotype (lineage), and phenotype (a creature's
> body/traits), from defining the environment (like the world or very large
> macro entities might).
>
> Now. This is a roguelike. The idea is that you direct critters in a densely
> populated ecology. At least each round, for now, we'll say, you can pick a
> new or the same species and location to start in each round; you unlock
> lineages. You can watch it live according to its instinct as it tries to
> survive and thrive according to its body/traits/abilities and instincts.
> Perhaps it dies; next critter of the lineage. When they all die, round's
> over and you pick again. Every round, you (and all the other lineages,
> including whomever you're playing with?) edit the lineage you played last,
> changing its traits' mapping to its body plan to generate a new template,
> which critters individuate from.

The bracketed word is his correction: "those mechanisms* (missed a word)".
He then asked for the mechanisms "as forks", and the questions in §7 are
those forks. *Flora* appears here in his words only: the founding record
(§7) keeps it platform-side, never a game word (§3).

## 2. Where it lands in what exists

Most of the framing already has a home.

- **A round is an epoch.** Record 7 ends an epoch by a timed rule, a gated
  one or a player's call. Ecology 1 already calls the boundary "the shop
  between rounds", where every lineage, played or not, folds in what
  prospered.
- **The edit is the boundary's revision.** `lineage/revise.rs` commits a
  variant of a line's development recipe drawn from what it has learned
  (752, 765). `lineage/review.rs` scores every candidate the played line
  could weigh by the boundary's grow-a-copy (684), the status quo first. A
  line's program is its committed revisions in order (`lineage/program.rs`),
  and a speciated line inherits its parent's up to the split.
- **The template is the recipe.** Bodies 6: tagmata of segment templates
  bearing kinds, with counts, variance and absence odds, plus a placement
  policy (anatomy brief §6).
- **Individuation** is each child drawing its own soma by its seed (Bodies
  6), plus the lineage policy's heritable noise at birth (mesocosm-overlay
  D2).
- **Watching on instinct** is directing: the critter chooses by its
  lineage's policy, and the player's only directive is the nudge, weighed
  by the bond (Mesocosm 2, Directing 4). Direct control waits for sapience
  (Mesocosm 1), which supersedes the project description's first-person
  pillar in play (rulings 60, 175).
- **Micro and macro bodies** exist at site grain: micro life is a surface,
  its matter on places and in hosts under its lineage's key; a terrain-body
  is a part tree big enough to carry places (Bodies 1).

What is new: classes as playstyles, the recipe as a growth grammar, a
template that moves by a success-weighted average, a player draft drawn from
meals, the systems sheet, rounds' own start and end rules, and bonding from
synergy.

## 3. Classes are playstyles

*Kingdoms since 1219; micro and macro are named microbe and world-class
(1218).*

Mark: "i meant to divide different types of playstyles with the classes,
that's all. it's not scientifically... accurate or anything" (1207). Five
classes: three lifestyles at the middle scale (plant, animal, fungus) and two
scales whose physics plays the same verbs differently.

| Scale | What changes | What the player plays |
|---|---|---|
| Micro | Viscosity beats inertia (low Reynolds number: stop swimming and you stop); diffusion and gradients steer; generations are fast; genes move sideways between lineages. | A population as a surface (Bodies 1), steered along gradients. |
| Middle | Inertia, individual bodies, pursuit and flight. | A critter, today's Mesocosm. |
| Macro | The square-cube law binds; the body carries places and accretes its own ecology; its time runs slow. | A habitat, played by shaping conditions for what lives on and in it: the forms-of-life brief's inhabitation (§5), influence rather than action. |

A lineage keeps its lifestyle in its recipe at any scale: a micro line can be
photosynthetic or predatory. Class bounds which operators and parts a recipe
may use; role is read from what members did, never chosen (1201). A lineage
crosses scales by a major transition at a boundary: multicellularity or
aggregation up from micro, accreting its own ecology up to macro,
parasitism or symbiosis down (1209).

The class names are working labels. They wait for a naming round (Naming
3), and *flora* is not available.

## 4. The recipe as a growth grammar

A recipe is a developmental program of growth rules; an ingredient adds a
rule or tunes one at a step of growth (1202). Every class draws its rules
from one set of operators, which it weights and bounds; subclasses are
presets (1206). The set below is a reading until built:

- **Repeat:** grow another copy of a unit (segment, stem node, zooid, cell).
- **Branch:** fork a growing tip.
- **Differentiate:** a unit's identity follows its position (Hox genes,
  heterocysts, a colony's castes).
- **Symmetry:** bilateral, radial or none.
- **Fuse:** join with another part or partner (hyphal anastomosis, lichen).
- **Switch:** change stage on a condition (metamorphosis, fruiting,
  dormancy).
- **Accrete:** lay down non-living structure (shell, wood, reef).
- **Allocate:** divide resources among parts (root to shoot).

When a rule fires is itself editable. Shifting it (heterochrony, of which
neoteny is one case) is a cheap edit with large effects. Bodies come out
self-similar wherever rules recur, the naturalistic, fractal look Mark asked
for.

Developmental systems the presets can draw on, by class:

| Class | Real systems (candidate subclasses) | Character of its grammar |
|---|---|---|
| Animal | Segmented along an axis (arthropod, annelid, vertebrate: today's tagmata recipe); radial (cnidarian, five-fold echinoderm); colonial (coral, bryozoan, siphonophore); metamorphic (larva to adult). | Repeat a segment along an axis, differentiate it by position, branch appendages. |
| Plant | Trees grown from their tips (Hallé and Oldeman's 23 architecture models); clonal spreaders on runners; rosettes; mosses. | Indeterminate tip growth repeating node, leaf and bud; leaves at the golden angle; allocation shifting with the environment. L-systems were made for this. |
| Fungus | Mycelial networks; budding yeasts; fruiting bodies on a trigger; lichen, fused with a photosynthetic partner. | A growing graph: tips extend toward resources, branch and fuse; a condition switches it to fruiting. |
| Micro | Single cells (shape, flagella, fission); biofilms with self-organising patterns; aggregators (Dictyostelium's slug and fruiting body, Physarum's network); filaments with specialised cells. | Cells with rules for adhesion, signalling and differentiation; horizontal transfer moves ingredients between unrelated lines. |
| Macro | Builders that accrete (reef, stromatolite, peat); giant clones (Pando's aspen, honey fungus); bodies that become habitat (a whale fall). | Accretion and niche construction: growth lays down structure that reshapes the terrain. |

Today's recipe becomes the segmented animal preset; nothing built is
discarded.

## 5. The edit between rounds

Mark: "making monster hunter equipment, but a bit more like making
monsters… like armored core, but you're generating from a recipe not hand
assembling" (1200). The edit has three layers:

1. **Drift, every lineage's.** At the boundary the template moves to the
   average of its members' recipes, each weighted by reproductive success,
   as an evolution strategy (CMA-ES and its kin) updates its mean (1208).
   The forward copy (684, the overlay plan's EB7) then weighs candidates
   from there against the web. Mark called this the "baseline aggregate
   drift representation". The played critter counts like any member, so
   beating the aggregate pulls the mean harder: his "averaging the
   generated creature against the whole to spread their genes or playing
   them to try to get a better result than the aggregate averaging".
2. **The player's draft.** Remaps offered from what the line incorporated
   that round, picked within an upkeep budget (1200).
3. **Creative mode.** Free editing within the class's bounds, for
   development (1200).

Today the boundary picks the best-scoring candidate by the copy (684);
averaging the members first is new.

**What a meal yields** (1203). Class decides what can be extracted: an
animal tissues and organs, a plant minerals from decay, a fungus compounds
and enzymes, a microbe genes by horizontal transfer. Kinship sets the yield:
close kin is taken easily, distant prey is potent, rare and risky (a
chimera's cost). Familiarity gates the novel: an ingredient near what the
recipe already is comes easily, and a new one only by exposure across
generations. Ecology 1's discovery, the lessons of eating a part whole, is
how a yield enters the draft.

**Conflicting edits** adopt or branch (1205): each edit proposes a child
revision; matching edits merge, conflicting ones fork the lineage, so a
disagreement between players, or between a player and the drift, ends in
speciation. The drift carries a branch nobody claims. This rules the
concurrency the overlay plan's EB6 left open.

**The systems sheet** (1215), in Mark's words "the armored core/ftl
abstraction … to describe the resultant creature in a systems oriented and
statistical manner w/rt its needs/abilities/the environmental ranges that it
could exist in". It is derived from the grown body, never summed from the
recipe:

- **Needs:** upkeep by body size (Kleiber's law; ecology brief 3.1),
  particular nutrients, water, light, oxygen.
- **Abilities:** speed, reach, force, armour, senses, fecundity, dispersal.
- **Tolerances:** performance curves over temperature, moisture, light and
  salinity. This is the fundamental niche, Hutchinson's hypervolume; play
  finds the realised niche.

At the edit it shows as a diff ("+reach, −cold range, +upkeep"), with
upkeep and mass as caps, as energy and load cap an Armored Core frame. Each
ingredient previews its expected contribution before the player commits.

## 6. A round: start, succession, end, discovery and bond

- **Start** (1211): a known lineage, at a place where it has members or one
  dispersal step beyond; past the range a start seeds a new one, at its
  risk. A line's first round may still start at the world's habitability
  for it (Mesocosm 3).
- **In the round** (1198): the critter lives by its instinct, its lineage's
  policy and its body; the player nudges.
- **Succession** (1204): at a death the player continues as the critter's
  offspring, else its nearest kin in the round. Having young is how a player
  keeps playing a line.
- **End** (1199, 1212, 1213): players agree to end once a span of world time
  agreed beforehand has passed; or a condition, picked by the players from
  offers verified achievable and novel (EB3's stop rule), is met; or, for a
  player, no offspring or kin is left in the round to play. That extinction
  is local, in Mark's words: "if there is no offspring or kin to play left".
  The edit lands on whatever survives anywhere (members elsewhere, dormant
  propagules, the nearest living fork); with nothing left it is lost, so a
  line truly extinct stays extinct.
- **Discovery** (1210): a lineage becomes known when a player outpaces it
  in biomass, incorporates its members, forks it from their own, or meets
  it. Any known lineage can be watched. Mark: "you can watch any creature
  you discover, but are they gonna listen to you? You have to be able to
  bond with 'em to direct 'em, right?"
- **Bond** (1214): a lineage's regard for a player rises when the player's
  lineages benefit it (pollinating, dispersing, cleaning or feeding it).
  Regard sets each new critter's starting bond, and nudges that serve the
  critter grow it within a life (Mesocosm 2). Regard is keyed to the
  player's persona, the participant entity of Directing 4, by a persona
  seed, and heritable within a bound, so a wild line can be domesticated by
  effort over generations (mesocosm-overlay D3's candidate).

## 7. Answered

Put to Mark as five card rounds on 2026-11-09, then four more in the second
pass (§10). Each answer is a case in the design record, in his words.
Operational answers (when to write this brief, how far to record it, whether
to commit) are not cases.

| # | Question | Case | Answer | Doctrine |
|---|---|---|---|---|
| 1 | What the player does in a life | 1198 | The nudge | Applies Mesocosm 2; Mesocosm 1 refined |
| 2 | What ends a round | 1199 | Extinction, players' agreement after an agreed span, or a condition | Applies Record 7; Mesocosm 3 refined |
| 3 | What limits the edit | 1200 | The copy's drift as baseline, a draft as input, free edit as creative mode; generated from a recipe | Ecology 1 refined; Mesocosm 6 new |
| 4 | How class and role attach | 1201 | Class bounds, role emerges | Bodies 1 refined |
| 5 | What a recipe is | 1202 | A growth grammar, ingredients contributing step by step | Bodies 6 refined |
| 6 | What a meal yields | 1203 | Class digests, kinship tunes; and familiarity | Ecology 1 refined |
| 7 | Who a player continues as | 1204 | Offspring, else kin | Mesocosm 1 refined |
| 8 | How conflicting edits resolve | 1205 | Adopt or branch | Ecology 1 refined |
| 9 | How the grammar spans classes | 1206 | Shared operators, class presets | Bodies 6 refined |
| 10 | What macro is | 1207 | Classes are playstyles; micro and macro are scales that play differently | Bodies 1 refined |
| 11 | How the template updates | 1208 | Success-weighted average | Ecology 1 refined |
| 12 | Whether a line crosses scales | 1209 | By major transitions | Bodies 1 refined |
| 13 | How lineages are unlocked | 1210 | All four; watching needs discovery, directing a bond | Mesocosm 3 refined |
| 14 | Where a round starts | 1211 | Where it lives, or one hop | Mesocosm 3 refined |
| 15 | Where conditions come from | 1212 | Verified offers | Applies Record 7; Mesocosm 3 refined |
| 16 | What an extinct line's edit does | 1213 | Local extinction; lands on survivors | Mesocosm 3 refined |
| 17 | How a bond is earned | 1214 | Synergy plus served nudges | Mesocosm 2 refined |
| 18 | What the sheet is built from | 1215 | Derived, with previews | Mesocosm 6 new |
| 19 | How scales are paced | 1216 | A microbe needs a medium; world-class is the aggregate view between rounds | Bodies 1, Mesocosm 1 refined |
| 20 | A player with nothing to play, in company | 1217 | Hop to the nearest living member; with none, wait out the round, offering to forfeit or end it | Mesocosm 1 and 3 refined |
| 21 | World-class view: rule or candidate | 1216 | Rule it | Mesocosm 1 refined |
| 22 | The five names | 1218 | Plant, animal, fungus, microbe, world-class | Bodies 1 refined |
| 23 | 'Class' has two senses | 1219 | "I was just using class for the rpg metaphor" | Naming 3 refined |
| 24 | The playstyle's word | 1219 | Kingdom, retire the enum | Bodies 1 and 6, Ecology 1, Mesocosm 6, Naming 3 refined |
| 25 | Candidate kingdoms | 1220, 1221 | All seven; a composite is two kingdoms in one lineage; players may share a lineage | Mesocosm 1 and 6 refined |
| 26 | The reading's new name | 1222 | Trophic | Naming 3 refined; built |
| 27 | What a microbe player directs | 1223 | Switches + plasmid draft | Mesocosm 1 refined |
| 28 | How microbe play is paced | 1224 | Event-paced, watchable live | Mesocosm 1 refined |
| 29 | How strategies relate to a strain | 1225 | One strain, switchable | Mesocosm 1 refined |
| 30 | A microbe inside another player's critter | 1226 | Open play, regard decides | Mesocosm 1 refined |
| 31 | What keys a player's persona | 1227 | Their dramatis identity, minted if absent | None; Directing 4 unchanged |

Question 16 came back because the framing's "When they all die … edit the
lineage you played last" read against the project description's "your line
continues or it does not" and Mesocosm 3. Mark's answer resolved it: the
extinction is local to the round.

## 8. Open

- **The microbe's numbers.** Which switches and plasmids exist, what each
  costs to carry, and what the coarse step owes crowd-grain strains in hosts.
- **Regard's bound.** How far regard is heritable; the persona itself is the
  player's dramatis identity (1227).
- **The operators.** Whether eight is the set; how a crossing unlocks
  another class's operators; what self-similar growth costs at site grain.
- **The sheet.** Which tolerance axes the world exposes, and whether a
  preview is a static derivation or a short forward copy.
- **Drift and draft together.** Whether the player's picks enter the copy as
  candidates beside the averaged template, or are applied after it.
- **Candidate kingdoms** (1220): which come first, and what each needs of
  the sim (a colony's castes, a host as a parasite's world, a replicator
  with no body, sophonts for an idea).
- **The asserted keys** stay `kingdom:flora` and its siblings (1218); the
  founding record (§7) keeps *flora* off the game's words, which the
  display terms now honour, but the key still carries it.
- **The project description** says a player unlocks other lineages "as you
  outpace them"; outpacing is now one of four routes to discovery, and
  directing needs a bond. The description is the maintainer's; this is
  surfaced, not edited.

## 9. Where the work would land

- `shared/cosmograph/src/lineage/`: the success-weighted average before the
  forward copy (`boundary`, `review.rs`), adopt-or-branch concurrency
  (`revise.rs`, `speciate.rs`), landing an edit on survivors.
- The bodies plan and Bodies 6: the grammar, its operators and the class
  presets, the tagmata recipe kept as the segmented animal preset.
- The overlay plan: rounds (EB3's conditions, EB6's adopt-or-branch, EB7's
  copy as drift), succession and the draft; Mesocosm's host for the sheet
  and its previews.
- Directing: regard as a lineage-level relation to the participant entity,
  beside the bond and trust per kind of direction (1054).

A plan follows when code changes (DOC_POLICY §8); this brief changes no code.

## 10. The second pass

Asked the same day, after the first commit, from the open questions above.

- **Scales** (1216): a microbe lives in a medium, a host or a substrate. A
  world-class lineage is played from the aggregate view between rounds,
  in Mark's words "diagetic directing, the management of world cycles":
  managing the world's time flow and cycles, and a stream of holistic and
  specific events the sim's conditions raise. It sits beside World 4's
  world as a generative entity and storyteller.
- **A player out** (1217): with no offspring or kin near, the player hops to
  the nearest living member of their line, however far. With none anywhere
  they wait out the round, and may offer to forfeit it or end it where it
  can be ended; the others agree or play on. This amends 1213: a round no
  longer ends for a player while their line lives anywhere.
- **Names** (1218, 1219): *kingdom* is the playstyle's word; the five are
  plant, animal, fungus, microbe and world-class. Taken words were ruled
  out first: clade, species and ancestral line mean lineage; grade, form,
  habit, guild and realm are spoken for elsewhere in the wing.
- **Candidate kingdoms** (1220): colony (a hive whose castes the player
  allocates; a solitary-to-social transition), parasite or symbiont (the
  host as world and vehicle), composite (two kingdoms coexisting in one
  lineage, "kinda like ball x pit partners"), replicator (no body, only a
  recipe hijacking others' growth), idea (spreading through sophonts'
  minds, a bridge to Eponym), spirit (acting on a place through conditions
  and belief, toward Organs 1's divinity) and construct (made, reproducing
  by fabrication).
- **Sharing** (1221): players may share a lineage or each play their own.
  Sharing one, they plant its critters where they choose, add to its budget,
  and add their edits to its gene pool, to be played or folded in;
  individuating the critter a player plays draws on that budget, so a
  shared line can be rich in edits and poor in bodies.
- **Trophic** (1222), built: `shared/cosmograph/src/trophic.rs` (was
  `kingdom.rs`) holds `Trophic` and `trophic::of`; isometer-render's
  `trophic_colour` (was `kingdom_colour`) tints by guise; Mesocosm's vitals
  and port words read `trophic:` terms, and the pack's kingdom terms display
  the five names. Checked: the Mesocosm, cosmograph, isometer and Eponym
  workspaces build with all targets; clippy is clean of errors; the
  mesocosm-views, -phenotype, -genet and -runtime library tests and
  cosmograph's trophic and port tests pass.

## 11. Microbe play

- **What you direct** (1223): a standing stance of gene switches (virulence,
  motility, biofilm, spores, enzymes, nutrients leaked to the host), which
  the population carries out, as real strains regulate genes and flip them
  at quorum; and a draft of one-shot plasmid abilities picked up by contact
  with other strains, costly to carry and lost without selection, spent to
  break into a host or survive a purge.
- **Pacing** (1224): the strain runs at crowd grain on the world clock.
  Choices fall due at host and patch events: entering a host, quorum, the
  host eating, mating or dying, immunity rising, contact with another
  strain. Between them the player can watch the spread live. No one waits.
- **Strategies** (1225): any strain may decompose, infect, aid a host,
  join it or disperse on wind and water. The virulence–transmission
  tradeoff is the pathogen's core choice: kill fast and strand yourself, or
  keep the host walking. Staying integrated long enough is the
  endosymbiont's transition into a composite with the host.
- **Beside an animal player** (1226): open play. A microbe player can live
  in another player's critter as gut mutualist or pathogen; mutualism earns
  the host lineage's regard (1214), and the animal's nudges (avoid carrion,
  eat bitter plants) are the counterplay.
