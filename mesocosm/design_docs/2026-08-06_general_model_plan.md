# The General Model (2026-08-06)

**Status: research and founding, 2026-08-06.** The ecological half is a
scheduled change with gates. The fantastical half is a **proposed shape,
not an adoption**: nothing in §6-§9 is scheduled, and F-gates exist to be
argued with. Sibling to the
[place-graph engine plan](2026-08-05_place_graph_engine_plan.md), which
owns world substrate, and the
[mesocosm founding plan](2026-07-30_mesocosm_founding_plan.md), which owns
the epoch loop.

**W1, 2026-09-18:** rewrite. Tier: mixed, sim causal grammar and wing organ.
Split: E0 to E4 are sim; §7.4's hagioglyph is a wing organ no single game
owns. Rewrite is a lane under the record's W2 or W3; until it lands this
plan's done-conditions are not authoritative. Evaluated against the wing
design record; see
[2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md)
§2.

**Scoping update, 2026-09-09:** §7.1 documents multiple interacting
fantastical systems and procedural variation across worlds. §7.2 records
the subsequent open discussion of body/item grammars, magic anatomy,
operator composition, vows, curation, and SRD adjudication. These broaden
the candidate space without settling a spec or scheduling implementation.

**Implementation update, 2026-09-13:** §7.4 adopts Mark's configurable glyph
canon and journey-shaped divinity direction. Its shared kernel and Mesocosm
trial adapter are implemented and tested; playable effects and durable lives
remain open. This is a specific exception to the earlier
blanket deferral of fantastical implementation, not adoption of all §6–§9.

**Implementation status, 2026-08-07 (audit-corrected wording): E0-E4
implementation slices landed and workspace-green; acceptance gates
open.** E0 allometry, E1 anatomy-derived feeding and
predation, E2 graph dispersal, and E4 drive selection are live. E3 has a
reversible far-tier cohort projection plus conservation and promotion
receipts; the authoritative individual-to-cohort storage replacement and
the order-of-magnitude capacity benchmark remain open.

**Wing-level notice.** §5-§9 describe machinery all three vessels would
share (Mesocosm discovers, Paredros embodies, Isometry exploits). Per
`CLAUDE.md`, wing material lives once. Siblings cite this file; they do
not copy it.

---

## 0. Why one document

Two questions arrived separately and turned out to be one question.

The first was Mark's: **shouldn't the priority be to shape the world and
its conditions so that behaviour just emerges?** The second was whether
the Madingley model, and a fantastical counterpart to it, are the right
lodestars.

They join at a single observation. Madingley's spine is *biomass flowing
between pools via processes, at rates set by traits*. Strip the word
"biomass" and what remains is a **causal grammar**: sources,
transformations, storage, channels, sinks, signals, constraints. Bodies,
ecologies, and fantastical systems can all speak it.

**Corrected 2026-08-07 (review).** The first draft concluded "one model
with settings, and the ecology is its first configuration." That
generalized past the wing's own authority rules: the
[phenotype plan](2026-07-31_phenotype_plan.md) forbids sharing an
evaluator before two sovereign rule systems have independently proven the
same mechanism, and the founding record rules that what vessels share is
world identity and compatible facts, never one live world model. The
correct claim is:

> **One causal grammar, separately proven rule systems, and extraction
> after repetition.**

Mesocosm implements its ecology concretely first. One fantastical
mechanic later proves whether a bounded primitive genuinely repeats;
only then is anything extracted. Configurability survives intact (a
world's metaphysics is which carriers and rules it instantiates), but as
a family of sovereign configurations over a shared grammar, not settings
on one engine.

---

## 1. Prior-art ledger, verified 2026-08-06

Terminology first, because naming what exists tells us what does not.

| Term | Status | What it actually names |
| --- | --- | --- |
| **General Ecosystem Model (GEM)** | Established | Madingley's category. "General" = one set of ecological concepts applied to any ecosystem, terrestrial or marine, at any resolution. Madingley is described as the first process-based mechanistic GEM. |
| **Speculative evolution** | Established community term (Dougal Dixon) | The creature and ecology half, non-magical. |
| **Computational modeling of religion** | Real academic field | Agent-based modelling of belief, practice, demography. Center for Mind and Culture's Modeling Religion Project; Wildman, *Modeling Religion*. The genuine analogue on the belief side. |
| **Thaumatology** | Real word; GURPS uses it as a book title | The *study of magic systems*. The GURPS volume is explicitly a meta-book of alternative frameworks (spell-based, ceremonial, spirit-mediated, runic, freeform, material). Closest existing English word for the discipline. |
| "magical ecology" | TTRPG/worldbuilding usage, one published supplement | Content-generation advice. No formal semantics. |
| "thaumodynamics", "arcanology" | Hobbyist worldbuilding only | In-fiction disciplines invented per setting. |
| Mauss & Hubert, *A General Theory of Magic* (1902) | Real, literally titled "general theory" | Anthropological theory of magic as total social phenomenon. A theory, not a model class, and argues *against* Frazer's reduction. |

**The finding: the GEM-shaped slot is empty.** No established term names a
general, mechanistic model class for fantastical ecology or metaphysics.
Also absent, and searched for rather than skipped: any academic subfield
for procedural generation of magic systems, and any GDC talk specifically
about simulating supernatural or belief systems.

Per `CLAUDE.md`, **no name is coined here.** Recording the empty slot is
the finding. A naming round with the usual crates.io, game, studio, and
trademark checks happens separately, if ever.

### 1.1 Sources that carry weight

- **Madingley**: Purves et al. 2013; Harfoot et al. 2014, *Emergent Global
  Patterns of Ecosystem Structure and Function from a Mechanistic General
  Ecosystem Model*, PLOS Biology.
- **Effect systems**: Dwarf Fortress syndromes and spheres (wiki, verified).
- **Procedural history**: Grinblat & Bucklew, FDG 2017 PCG Workshop
  (Caves of Qud); Grinblat, "Generating Histories" in *Procedural
  Storytelling in Game Design* (2019).
- **Knowledge propagation**: James Ryan, Talk of the Town / Hennepin;
  "Simulating Character Knowledge Phenomena in Talk of the Town",
  *Game AI Pro 3* (2017). The only worked implementation found of belief
  as a *distorted propagating representation* rather than a fact.
- **Constraint design**: Sanderson's First and Second Laws (primary
  source). Second Law decomposes into three separable levers:
  *limitations* (what it cannot do), *weaknesses* (what it exposes you
  to), *costs* (what it consumes).
- **Historical schemas as structural templates**: Frazer's two laws of
  sympathetic magic; wuxing (five nodes, two directed cycles, plus a
  correspondence join table); humoral theory (a 2-axis product space);
  Paracelsian doctrine of signatures; hermetic correspondences.

### 1.2 Magic-system comparison (2026-09-09 research pass)

This pass compares mechanisms for the §7.1 discussion. Source descriptions
below are distinguished from proposed wing adaptations. Published tabletop
rules, released digital systems, fiction, and development reports establish
different kinds of evidence; none alone proves a procedural world generator.

| Prior art and source | What the source establishes | Proposed lesson for the wing |
| --- | --- | --- |
| [Ars Magica, publisher overview](https://atlas-games.com/product_tables/AG0215) and [rules briefing](https://www.atlas-games.com/pdf_storage/ArMcheatsheet.pdf) | Techniques and Forms combine actions and subjects; the older briefing explains spontaneous casting with effect guidelines and storyguide judgment | A language for expressing intent and discovering combinations. Explicitly implement supported meanings; the tabletop referee's interpretation is not a computational mechanism. The briefing is historical, not a current-edition RAW receipt |
| [RuneQuest magic rules](https://rqwiki.chaosium.com/rules/magic.html) | Spirit magic draws on personal magic points and uses a focus; Rune magic invokes a deity through cult access, sacrifice, and Rune points | Coexisting traditions can have different sources, relationships, replenishment and obligations, even where outcomes overlap. Culture can determine practical access without merely renaming spells |
| [Noita, developer description and FAQ](https://noitagame.com/) | Crafted spells affect simulated materials, including simplified reactions, electricity and thermodynamics | Magic participates in terrain and material consequences. One effect can provide attack, excavation, escape or an environmental hazard. This establishes world interaction, not generation of different physical laws per run |
| [Hex Casting, versioned project manual](https://hexcasting.hexxy.media/v/0.10.3/1.0/en_us/) | Pattern sequences manipulate a stack of information and invoke effects | Powerful reference for explicit spell composition and inspecting intermediate results. A programming interface is one optional practice; it need not be the player's universal means of casting |
| [Book of Hours, developer overview](https://weatherfactory.biz/book-of-hours/) and [released crafting changes](https://weatherfactory.biz/coseley-release/) | Occult books, learning, visitors and writing histories structure play; release notes document crafting helpers and persistent memories | Knowledge and history can be useful magical objects. Investigate the full recipe semantics separately before claiming a general correspondence engine; these pages establish the narrower implemented features |
| [Mistborn, author's account](https://faq.brandonsanderson.com/knowledge-base/how-did-you-come-up-with-the-magic-system-2/) and [interaction example](https://www.brandonsanderson.com/blogs/blog/annotation-mistborn-2-chapter-twelve) | Separately conceived systems were brought together; the author identifies interactions between pushing/pulling and changing one's weight | A few shared physical variables can connect distinct systems deeply. This is literary design evidence, not a playable balance receipt |
| [GURPS Thaumatology, publisher preview](https://dtrpg-public-files.s3.us-east-2.amazonaws.com/custom_previews/12199/224852.pdf) | A toolkit covering alternative frameworks, including Path/Book and symbol magic | Useful comparison vocabulary for world profiles. Framework breadth does not establish that all variants share one evaluator or compose automatically |
| [Dwarf Fortress, Bay 12 development log](https://bay12games.com/dwarves/index.html), 2026-03-31 entry | Development report describes generated magical forces, earlier civilizations with different access, and varied magical laboratories | Closest direct procedural-world research direction here: connect cosmology, history, practice and machinery. Treat as development evidence, not a released full myth-and-magic acceptance result |

The earlier empty-field finding above records an August search outcome, not
proof that relevant work does not exist. Bay 12's concrete development reports
deserve follow-up beyond the older myth-generation transcript. This pass does
not settle the existing Qud, Dominions, or full Book of Hours recipe debts.

For the next discussion, use three comparisons: **grammar versus physical
interaction** (Ars Magica/Hex Casting/Noita), **different sources and social
access** (RuneQuest), and **interlocking systems across history**
(Mistborn/Dwarf Fortress). These can coexist. A practical design question is
whether two ways to produce heat also differ in their supply, skill, equipment,
social commitments, and counterplay.

### 1.3 The cautionary source

Raph Koster's own account of Ultima Online: the simulated ecology was cut
in beta for **performance and maintainer comprehension**, after being
rewritten by an engineer who did not understand it. What died was the
behavioural half. The static resource data survived and still feeds
crafting today.

Two lessons, both binding on this document:

> **Simulated layers die to performance and to comprehension before they
> die to players.** And **the data half can survive when the dynamics half
> is throttled.**

Which argues for inspectable state (fields, syndromes, currencies) over
opaque dynamics (behaviour engines), and for building every layer so its
state stays legible and attributable even if its dynamics are cut.

---

## 2. Madingley, mechanically

- **Cohorts, not individuals.** A cohort is organisms sharing functional
  group and continuous traits within a cell, carrying three state
  variables: abundance, body mass per individual, reproductive mass. When
  cohort count exceeds a threshold, the nearest pair in trait space merges
  and biomass is conserved across the merge.
- **Autotrophs are stocks, not cohorts.** Deliberately: "individual" is
  ill-defined for a plant, and marine turnover outruns the timestep.
- **Six heterotroph processes per timestep, plus primary production**:
  metabolism, feeding, growth, reproduction, mortality (predation,
  starvation, background, senescence), dispersal (diffusion,
  starvation-driven, currents). **The order in which cohorts act is
  randomised each timestep** (verified against the paper 2026-08-07; an
  earlier draft mis-stated this as process-order randomisation).
- **Categorical traits select qualitative mechanisms** (feeding mode;
  endo/ectotherm); **continuous traits, chiefly body mass, modulate
  rates** alongside environment and functional group, through allometric
  relations, across **10 ug to 150,000 kg** (fourteen orders of
  magnitude; an earlier draft wrote 10 mg).
- **The payoff**: biomass pyramids, trophic structure along productivity
  gradients, latitudinal carnivore ratios, body-mass/density relations,
  all **emergent and unfitted**, from individual-level processes alone.

That last line is the published receipt for "shape the conditions and let
behaviour emerge."

---

## 3. Where Mesocosm already stands

Checked against the live core and workspace seams, 2026-08-07.

| Madingley process | Mesocosm |
| --- | --- |
| Metabolism | Present. `pay_upkeep`, budget first then body. |
| Autotrophy | Present. Producers fix, crowding shades income. |
| Herbivory | Present. Consumers graze producers in range. |
| Decomposition | Present, and explicit rather than folded into stocks. |
| Growth | Present. `gain_mass`, juvenile stage. |
| Reproduction | Present. Gestation, offspring costs a share of parent mass. |
| Mortality | Present. Starvation, senescence, predation-as-carrion. |
| **Predation** | Present. Feeding mode is read from body symmetry and contractile anatomy; live prey records `MealKind::Predation`. |
| **Dispersal** | Present in the scheduled slice. Near bodies move by integer steps; far bodies move through the place graph or diffuse when exhausted. |

Mesocosm additionally has something Madingley does not: **`Signal`**, an
advertised claim that can be false. Choosing a meal is already a judgment
rather than a lookup.

At founding, the ecological work was **two missing processes and one changed
principle**, not a rewrite. The scheduled slice now supplies those process
seams; the remaining work is receipt depth, especially population-scale far
state.

### 3.1 The changed principle

Madingley's real lesson is not the process list. It is *categorical
minimal, continuous drives rates*. The scheduled slice now expresses
maturity, lifespan, gestation, income, feeding, decay, upkeep, and dispersal
through integer mass-derived functions. `Kingdom` remains a compatibility
reading, but is derived from the body's symmetry; feeding mode also reads
contractile anatomy.

### 3.2 The concrete bug

`upkeep_mg = UPKEEP_MG + biomass_mg / UPKEEP_SHARE` is **linear in mass**.
Real metabolism scales as roughly mass^0.75 (Kleiber). Linear upkeep
over-taxes large bodies at exactly the rate that makes large bodies
unviable, so the world carries a **size ceiling nobody chose**. This is
worth fixing on its own merits, independently of everything else here.

### 3.3 What emergence buys, concretely

Point the existing pieces at each other and roles stop being categories:

- **Feeding is a satisfied process, not an anatomy reading.** The first
  draft said "a body that can take living prey *is* a predator," which
  regressed behind the ProcessDef ruling that capability is read from
  **allocation, anatomy, channels, cost, and environment**. A mouth-shaped
  part establishes nothing by itself: it may lack contraction, digestion,
  control, throughput, or a suitable medium, and two identical plates may
  express armour and light capture. The correct sentence: **ecology
  queries satisfied feeding processes; trophic role is a derived summary
  of realized activity.** The landed E1 derives from anatomy as an
  interim; the PD1b-backed form is the standing target. `Kingdom`'s own
  doc says a lineage may combine roles while each organism stores one
  variant; producer, consumer, and decomposer want to become
  independently realizable strategies, with `Signal` remaining the
  advertised claim.
- **Life history derives from mass.** Maturity, lifespan, gestation,
  dispersal range, feeding rate as power laws. Large becomes slow,
  long-lived, wide-ranging, hungry but efficient, with nothing authored.
- **The `Hunter` dissolves.** Pursuit is what a fast, large-mouthed,
  starving thing does about a reachable meal. The authored `places::Hunter`
  path is retired; only the old epoch pressure fixture still uses "hunter"
  as a test label.

### 3.4 Cohorts are the far tier's natural unit

The place-graph plan's two-tier simulation wants exactly Madingley's
representation: individuals near the focus, **cohorts** at graph distance,
with trait-space merging as demotion and splitting as promotion. The current
slice forms deterministic far-tier cohorts and conserves count, biomass,
energy, and age sums; the individual roster still remains authoritative until
the capacity experiment admits replacing it.

---

## 4. State, as a design questionnaire

**Reframed 2026-08-07 (review).** The first draft called these axes a
currency type and biomass "conserved." Both were wrong: Mesocosm's
simulated biomass is **sourced** (production) and **sunk** (upkeep,
return); a deeper matter model could conserve constituents across
reservoirs, but the stock as simulated does not. And one "currency" type
covering stocks, fields, curses, beliefs, and temperature would be a
universal property bag wearing a nicer name.

The eight axes survive as what they actually are: **a questionnaire every
proposed piece of world state must answer.**

| Axis | Values |
| --- | --- |
| **Conservation** | conserved; sourced (enters from outside); sunk (leaves); both |
| **Transfer cost** | full (giver loses what receiver gains); partial (conversion loss); none (copying) |
| **Locus** | entity; place; edge (kinship, provenance, contact); global |
| **Autonomous change** | persists; decays; regenerates; oscillates |
| **Rivalry** | rival; non-rival |
| **Transmutability** | which other currencies, at what loss |
| **Flow constraint** | which edges permit flow: trophic, contact, sight, descent, provenance |
| **Observability** | visible; inferable; hidden |

Worked points:

- **Biomass**: conserved, full-cost transfer, entity-located, decaying via
  upkeep, rival, weakly transmutable, flows along trophic and contact
  edges, partly observable through size. *The ecology is this point.*
- **A curse**: non-rival, no-cost transfer, edge-located along provenance
  or descent, persistent, hidden until it fires.
- **Ambient power**: sourced, place-located, regenerating, rival,
  observable, flows by proximity.

**Not every combination is coherent**, which is DF's spheres lesson
applied here: any generator over this space needs an **exclusion
relation**, the same way a deity may not hold precluded spheres.

### 4.1 Typed state carriers (2026-08-07)

Answers to the questionnaire sort into a small set of carrier types, and
they stay typed rather than unifying:

| Carrier | What it is | Worked example |
| --- | --- | --- |
| **quantity** | conserved or sourced stock on an entity | biomass, energy |
| **field** | place-indexed intensity | ambient power, temperature |
| **condition** | attached state with triggers and duration | a curse, a disease |
| **relation mark** | state on a provenance, descent, contact, or trust edge | contagion, a debt |
| **claim** | observer-relative information that may be false | belief, `Signal` |

An effect application may target any carrier, but **effects propose typed
state changes**; they do not write into a universal currency.

**Observability is a relation, not an axis.** A carrier may hold an
*emission profile* (a field that glows, venom that smells, `Signal`'s
advertised claim); whether anyone observes it depends on the observer's
senses, instruments, position, and history. The eighth axis of the
questionnaire asks about emission; observation lives in the epistemic
loop (S8).

---

## 5. One effect system, four channels

Dwarf Fortress's syndromes are the strongest single mechanism found:
alcohol, snake venom, plague, vampirism, mummy curses, and werebeast
infection are *the same object*, applied through typed channels (contact,
ingested, inhaled, injected), carrying effect sets. Even lycanthropy's
supernatural part is a **trigger predicate over world state**, not a
special case.

The rule this implies, and it is the don't-duplicate ruling again:

> **Do not build a magic-effect type.** Build one effect-application
> envelope with typed channels, and let disease, venom, weather,
> blessing, and curse all emit it. **Effects propose typed state
> changes** against the S4.1 carriers; a world with more carriers than
> biomass gets a richer effect space for free, without a universal
> property bag.

---

## 6. Derivation rules (proposed)

How generated bodies and places acquire fantastical properties without
anyone authoring them. All three have historical schemas behind them and,
usefully, existing hooks here.

- **Signatures** (Paracelsus): form indicates function. A rule for reading
  properties *off* a generated morphology. Hook: the axial generator.
- **Similarity** (Frazer's first law): like produces like. Formally, a
  distance metric over trait vectors. Hook: recipes and somas are already
  trait vectors.
- **Contagion** (Frazer's second law): things once in contact continue to
  act at a distance. Formally, an edge in a provenance graph. Hook:
  **every incorporated part already carries `Provenance`.**

The alignment supplies useful inputs for sympathetic magic; link semantics,
costs, transmission, severance, and discovery still require implementation.

### 6.1 Kleptoplasty past biology

The wing's acquisition metaphor already reaches: kleptoplasty is keeping
working machinery from what you eat. Extending it past biological traits
needs **no new intent**; metabolize stays the one verb.

It needs **criteria**, which is Sanderson's Second Law made mechanical:

- a part structurally capable of hosting the property (capability, not
  inventory);
- a source in a state that releases it;
- finite capacity, so acquisition trades against acquisition.

This is `ProcessDef` and phenotype work, not new machinery.

### 6.2 The skeleton: identity, facts, derivation, transition, projection (2026-08-07; revised in review)

The entity-model question resolves into **five** layers. The prompt that
named the derivation layer was
[PolyCSS](https://github.com/LayoutitStudio/polycss), a CSS 3D engine
rendering VOX/glTF meshes as real DOM elements, each individually
addressable and styled by rules. Its world-lane technique does not
transfer (per-polygon DOM at simulation scale is cardinality death). Its
architecture names one layer; the review caught that four layers describe
a *read pipeline*, while a simulation additionally needs **transition**,
which the wing already owns and the first draft failed to list.

| Layer | What it is | Already standing |
| --- | --- | --- |
| **Identity** | Stable ids | `OrganismId`, `PartId`, `PlaceId`; chartulary Container platform-side |
| **Facts** | Facets as plain serialized state: mass, traits, carriers. Ordered, hashed, replayable | the core |
| **Derivation** | Rules computing rates, conditions, and affordances from facts | grade blocks; E0 allometry; §6 rules; V2's dependency digests as the invalidation |
| **Transition** | Intent and process resolution: time, choices, conflicts, costs, refusal, causal records. Events yield new facts | `Intent`/`Outcome`, `act.rs`, the ecology step, `History`, replay hashes |
| **Projection** | Per-vessel lenses reading computed values, every emitted element carrying source identity | genet-probe doctrine; `BodyLensProjection` sidecars |

PolyCSS needs no transition layer because CSS describes presentation.
Mesocosm does; it is where the game lives.

The correspondence that makes "derivation" a *styling* layer, closing a
ruling made earlier ("the soul question is a styling matter"):

- signatures and similarity are **selectors over trait vectors**;
- the cascade's inputs, corrected 2026-08-07 (kingdom was circular once
  kingdom became derived): **world law, environment, developmental
  program, phenotype allocation and anatomy, current condition**;
- allometric rates, derived feeding modes, and fantastical properties are
  **computed values**;
- the grade is literally the stylesheet's visual half;
- and each derived property **declares its combinator**: replace, add,
  multiply, clamp, require, or prohibit. A universal winning-declaration
  rule is too weak for biological flow.

Prior art the stack already owns: **livery**, whose enumerable
TOML-property-database discipline is the tamed version of this. The
binding constraint comes from §1.2 and from CSS's own failure mode
(specificity wars): the cascade stays small, strictly ordered, and
attributable. "Why is this critter fast" must answer with a rule chain.

On ECS, reworded after review (the first draft made a category mistake
by opposing them): **ECS is a storage and iteration technique; this
skeleton is a semantic model. ECS is not the domain ontology, and storage
may become data-oriented without changing authority.** Nothing here rules
a data-oriented layout in or out.

Two adjacencies recorded while here: for **Isometry**, PolyCSS is
near-literal prior art (Foundry-class scenes as identity-bearing DOM
elements styled by rules, with a VOX import path rhyming with the bake
pipeline). And the **sprite-stacking deferral has expired**: it was
parked pending a pulled-back camera, which Mesocosm now has (place-graph
plan §0.4). *Correction 2026-08-07: an earlier line called PolyCSS
"structurally sprite stacking"; it is not. PolyCSS meshes VOX into
visible polygon faces placed as DOM leaves; sprite stacking layers
parallel image slices. The reopening stands on the camera ruling alone.*

---

## 7. Composition grammar (proposed)

Ars Magica's formulation is the canonical generative magic grammar: 5
Techniques x 10 Forms, plus requisites, plus per-combination magnitude
guidelines that let any point be *priced* without being authored.

The adaptation, sharper than adopting it whole:

> **Fix the Technique axis; generate the Form axis from the world's own
> ontology.** Create, perceive, transform, control, destroy are near
> universal verbs over anything. Forms should come from what the world
> actually contains: its materials, its kingdoms, its currencies.

Technique x Form is then a matrix, and per the precluded-pairs rule **a
given world fills only some cells**. Which cells exist is that world's
magical character. A closed-form cost function over the parameter vector
(Morrowind's spellmaker is the worked example; Angband's power budget is
the learned-distribution variant) can price candidates. Balance still needs
interaction sampling and playtesting; pricing alone cannot establish it.

---

### 7.1 Procedural systems across worlds (2026-09-09 scoping)

**Status: discussion scope, documentation only.** Mark wants several deep,
interacting fantastical systems, with different realizations across worlds.
The earlier impossible-ecology direction remains a useful starting family;
it does not exclude deliberate magic, ritual, or spellcraft from this session.
The charge-and-sympathetic-link example in
`paredros/design_docs/2026-09-09_world_conditions_plan.md` is one candidate,
not the universal model or a selected implementation gate.

The earlier claims that sympathetic magic is nearly free, that a cost formula
ensures balance, and that an effect vector cheaply supplies discovery are too
strong. Existing provenance is a useful input, not a remote-action mechanic.
Costs need interaction testing; discovery needs evidence, experiments, and
people who can retain and communicate what they learn.

#### What generation changes

Separate five choices so a world can vary one without accidentally changing
the others:

| Choice | Generated or authored variation | Consequence |
| --- | --- | --- |
| Causal rules | What can change what, through which relationships | A severed part carries influence in one world; only a voluntarily given part does in another |
| Limits | Capacity, range, delays, costs, failure, exceptions | Distance, elapsed time, or broken consent can interrupt the same route |
| Embodiment | Which bodies, materials, places, seasons, and tools realize a rule | The route might require a living organ, constructed resonator, or seasonal mineral |
| Practice and knowledge | Learned methods, institutions, claims, disagreements, access | A public craft, guarded lineage practice, and misunderstood natural event can arise from the same law |
| Presentation | Names, shapes, sounds, gestures, visible traces | Distinct aesthetics can preserve the same mechanics |

Presentation variation is worthwhile. It is insufficient by itself to deliver
structurally different worlds. Conversely, every world need not invent new
physics: recognizable laws help players transfer knowledge, with local
exceptions discovered through play. Expose world-creation choices for families,
degree of causal variation, rarity, discoverability, and simulation budget.
Exact ruleset profiles pin their procedures; procedural adaptation is a
separate, honestly labelled mode.

#### Candidate families to discuss together

These are design alternatives and combinations, not an adopted taxonomy.

| Family | Persistent state and means of action | Limits and counterplay |
| --- | --- | --- |
| Material flows and fields | Accumulate, store, conduct, transform, and release a quantity | Supply, leakage, capacity, overload, insulation, competing uses |
| Sympathy and correspondence | Establish or exploit a typed relation between particular subjects | Provenance or matching criteria, link cost, range, severance, interference |
| Living transformations | Cultivate organs, exchange tissue, induce developmental processes | Viability, host compatibility, upkeep, recovery, ecological supply |
| Names, memory, and vows | Explicit rules read an identity, remembered fact, or recorded commitment | Who can establish it, what counts as fulfilment, forgetting, release, disputed knowledge |
| Agents and bargains | An independently acting being grants an effect or changes its behaviour | Consent, interests, ability to perform, obligation, refusal, retaliation |
| Place and time anomalies | A locality changes routes, rates, recurrence, or available transformations | Boundaries, entry and exit, recurrence conditions, detectable traces |

Surgery and ordinary grafting stay useful without fantastical permission.
Supernatural transformation adds a particular law, not a global exemption
from anatomy. Extra-arm combat similarly comes from usable anatomy: additional
punches have their own bindings, timing, contacts, costs, and recovery while
sharing balance and attention. The functional-loop and world-conditions plans
in Paredros own those action details.

#### How deep systems combine

Begin with authored causal mechanisms and generate their selection,
parameters, embodiments, and a sparse set of explicit couplings. A coupling
declares which output one system supplies as another's input, with units or
typed facts, delay, cost, authority, and failure behaviour. Shared words such
as "energy" or "memory" do not establish compatibility. A more expressive
rule-synthesis grammar is a later option, once these mechanisms show which
combinations are useful and explainable.

For example, a world might grow organs that store storm charge. A voluntarily
given fragment establishes a sympathetic route, and a maintained vow permits
that route to carry charge. This joins living transformation, a material
account, a relation, and a commitment. The recipient's overload limit still
applies. Breaking the vow closes the route; it does not delete stored charge.
A second world may support the same charge machinery through constructed
resonators and vibration, with neither vow nor biological donor. Those worlds
share one mechanism but require different bodies, buildings, practices, and
social arrangements.

Not every family needs to combine with every other. Generate a few legible
connections, including antagonisms and tradeoffs. An ecosystem, settlement,
or character should be able to exploit a rule in several ways: tools, shelter,
travel, care, conflict, and livelihood. Spell lists can emerge as learned
techniques over these systems rather than exhaust their uses.

#### Generation and persistence obligations

A proposed world must supply witnesses that its selected mechanisms can
actually occur: reachable resources and conditions, feasible carriers,
affordable actions, and observable consequences. Validate missing inputs,
unaccounted creation, inaccessible prerequisites, runaway feedback, and
dominant combinations. Time-stepped feedback is allowed; bound work per step
and make instability an explicit possible world behaviour. These checks
establish coherence, not guaranteed fun or balance.

Persist the instantiated law revision and generated bindings, not just a seed
that a future generator may interpret differently. Discovery remains distinct
from truth: claims can be incomplete or wrong, and characters' interests can
change how they interpret an event. Hagiograph retains significant history;
ordinary personal recall also needs its own bounded retention policy.
If a law consumes memory, define exactly which authoritative record it reads.
Cache eviction or archival compaction must not accidentally cast a spell,
erase a vow, release a debt, or revive a dead subject.

Long-lived worlds need restorable checkpoints, a bounded recent event tail,
retained consequential evidence, and measured growth with population and
exploration. Persistent changes can grow reasonably; endlessly duplicated
replay records need not. Paredros's memory-and-remembrance plan owns its
proposed save/recall retention work; its small save-growth receipt is not a
multi-year world capacity result.

#### Research lanes, then implementation lanes

1. **Mechanisms and prior art:** compare working causal families, player
   discovery, and interactions from primary sources. Revisit the verification
   debts below before treating older research summaries as established facts.
   Done when examples distinguish implemented mechanics from announced ideas
   and identify a reusable mechanism rather than a setting to imitate.
2. **World composition:** describe contrasting worlds using overlapping
   families, including a concrete coupling, ordinary uses, failure and
   counterplay. Done when differences can be explained through causes,
   embodiment, practice, and presentation independently.
3. **Authority and growth:** map each candidate state to the existing world,
   body, terrain, knowledge, and save owners. Done when a change has one owner,
   discoverable consequences, and a retention policy preserving active causes.

Only then promote bounded Luna/Terra implementation lanes by owner. A first
functional loop can establish generation, action, consequence, discovery,
and restoration without requiring a curated scene. The useful proof is that
systems interact consistently and remain playable; scene production is not
the acceptance gate. Open discussion: which families deserve the first mix,
how radically laws vary, and how much ordinary inhabitants already know.

### 7.2 Body plans, magic anatomy, operators, and items (2026-09-09 discussion)

**Status: open discussion, not a spec. Nothing in this section is settled.**
This records Mark's proposed synthesis following the prior-art discussion.
The phrase "settled framing" in the submitted notes describes their working
aura framing; the explicit open-discussion status governs this record. The
following proposals do not silently replace existing implemented semantics.

#### Proposal: body generation and curation

Body plans are samples from a clade-rooted grammar: symmetry axis,
segmentation rule, segment count, regional contents, and attachment vocabulary.
A flat catalogue of 100-200 plans is the wrong organizing structure. Curated
defaults are worked examples and modder documentation, tentatively about 20
per category, chosen for different grammatical cases rather than zoological
completeness. Curation displaces generation; the generator fills open choices
and admitted results persist rather than being rerolled each run.

Separate three axes: form grammar, organ systems (including magic anatomy),
and scale. The proposed scale examples are germ, bug, house cat, horse,
elephant, island turtle, and planetoid. Fauna/flora/myco, micro/macro,
chimerism, psionics and divinity must not be sibling categories. Chimerism is
structural incorporation made heritable: the seam must reconcile local axes,
segment counts, and attachments. Psionic capability can have a physical
substrate that is findable, excisable, susceptible to disease, and valuable.
Dorohedoro's devil core is a submitted inspiration, not a verified mechanics
source. Divine descent need not imply an organ or a deity supplying power.

Generate before completing a catalogue. Inspect roughly 50 outputs for
creature legibility and noise, then curate deliberate gaps. Repeat for items.
The current axial recipe already supplies a starting grammar; this proposal
does not mean discarding it and beginning from zero. See the phenotype plan's
"The axial generator" and ProcessDef plan's developmental anatomy join.

**Live check:** `crates/mesocosm-core/src/world/generation/body_plan.rs`
already has Axial and Branched selectors. Branched arranges axial tagmata in
a tree; it is not yet an indeterminate growth or fungal reconnection grammar.
`src/body.rs` supplies parts with stable identity, mass, geometry and local
attachment frames. These are concrete shared-substrate candidates, not an
existing item generator.

**Review:** curate both unreachable cases and representative generated
examples that teach the grammar. Spread the first inspection across declared
dimensions; 50 similar axial samples cannot validate branching or chimerism.
Replacing a default before realization is distinct from revising an existing
inhabited world. Later curation must target an explicit revision or an open
generation choice rather than overwrite persisted consequences silently.

#### Proposal: aura location and flow

External aura, internal aura, core, and generation suggest strip, penetrate,
excise, and interrupt as counterplay. These are useful encounters, not a
mandatory four-part taxonomy. Candidate embodiments include distributed
nodes, multiple cores, conducting/protective dermis (including fantastical
fire, ice, or wind behaviour), clothing, tools, resonance, and entered bodily
states with readable tells and interruption opportunities.

Sample site, distribution, boundary and gating, with three initial binaries:
concentrated/distributed, interior/exterior, continuous/gated. Ordinary
critters are proposed as low-magnitude draws rather than a separate system.

**Review: the eight combinations are not eight complete topologies.** They
describe distribution and activity; connectivity, direction, source, storage,
and transport still matter. Core, dermis and state machine are overlapping
descriptions, so there are not literally four empty cells left by those names.
Possible examples, not proposed classes:

| Distribution | Location | Activity | Example |
| --- | --- | --- | --- |
| Concentrated | Interior | Continuous | An organ maintaining an internal field |
| Concentrated | Interior | Gated | A gland discharging during a particular breath |
| Concentrated | Exterior | Continuous | A persistent orbiting focus |
| Concentrated | Exterior | Gated | A temporary focus formed beyond a horn |
| Distributed | Interior | Continuous | Conducting tissue throughout a body |
| Distributed | Interior | Gated | Nodes synchronized only in a trance |
| Distributed | Exterior | Continuous | An enveloping mantle or living garment |
| Distributed | Exterior | Gated | A skin-wide discharge during a defensive state |

Generation, storage, distribution and gating can reside in different parts.
Removing a source might stop recharge while leaving a stored field intact.
Penetrating the boundary need not disable the whole network. Local laws also
need to distinguish "available at negligible magnitude" from "absent or
impossible here"; the former must not silently make aura universal.

#### Proposal: operator composition

Generation supplies substances and bearers; operators describe possible
changes. These ten are the submitted vocabulary, not a closed final algebra:

| Operator | Proposed role |
| --- | --- |
| Strengthen | Charge or improve an existing capacity |
| Transform | Change aura into other forms, including elemental effects |
| Instantiate | Create a thing with specified qualities |
| Project | Apply an operation at range |
| Puppet | Control operation or behaviour rather than intrinsic properties |
| Subtract | Drain, nullify, or produce absence; an ecological counter-role |
| Bind | Establish links, shared damage, collective sensing or hive relations |
| Store | Persist aura in an object or place: relics, curses, haunted ground |
| Second-order | Modify the cost or availability of another operator |
| Raise | Establish a persistent developing subject whose form records training and which can outlive its maker |

Examples: Transform + Project yields an elemental projectile; Instantiate +
Puppet yields a remote construct; Store + Bind yields a relic linking holders.
Abilities are sampled chains, with curated examples still admitted. Nen
supplies the inspiration for the first five and aptitude falloff; the
transferable proposal is distance-dependent costs, not mandatory adoption of
its category list. Native aptitude may be a ring position or a point in a
generated graph. Nen beasts and PSO mags inspire Raise. Training conditions
may be authored, generated, or derived from play history. FMA's exchange and
contact with the totality, JJK's vows and cursed energy, and Destiny's granted
immortality and embodied planets are research leads, not verified claims in
this document.

**Review: distinguish roles within the vocabulary.** Transform changes state;
Project modifies a route; Store changes persistence; Raise introduces a
subject's lifecycle; second-order effects modify admitted operations. They can
compose without having identical input/output shapes. Each composition needs
typed targets, resources, duration, output, and declared failure. Establish
whether Store preserves fuel, a procedure, a binding, or all three. Observation
and sensing need an explicit account too: reading aura cannot remain a free
UI privilege outside the operators or ordinary senses.

Potential restrictions for second-order generation: explicit writable
parameters, modification-depth bounds, per-step execution budgets, and no
self-disabling validator or implicit resource creation. A creature unable to
use magic can be valid. A world whose required lifecycles or starting actions
are impossible is a different failure. Test feasibility of intended roles,
not a universal requirement that everyone retain magical powers.

#### Proposal: vows, alignment, and persistent identity

Divine alignment changes aura and unalignment reverts it; aura can expose
present or past affiliation. Breaking a vow carries oathbreaker costs unless
released by the patron, with a release/breach condition potentially granting
unique power. The submitted model proposes one vow operator with satisfaction
and breach clauses differentiated by patron disposition.

**Review:** keep creation, fulfilment, breach, release, and amendment as
distinct events within that one contract. Patron disposition can select a
consequence without turning release into an accidental breach. A patron's
judgment needs a declared evaluation time, rather than retroactively changing
an old event. Present alignment can revert while a historical trace persists;
whether that trace is physically detectable, merely recorded, or erased is a
world rule. A fulfilled condition grants power only through a declared source
and effect. Sentient patron decisions need not be reducible to a numeric sign.

#### Proposal: items use the same expressive foundations

Items have form grammar (hilt, blade, haft, socket and attachment sites),
material systems supplied by their world, and scale from needle to siege
engine to a god's discarded tooth. Local ore and fabrication make material
provenance an ecological fact. Curated items displace generated ones.

Store is proposed as the enchantment mechanism: an operator chain with a gate
persists in a form. Store-capable creatures consequently supply smithing and
cursing roles. Raise extends to developing objects, such as weapons changed
by use. Provenance records maker, materials and deeds; Hagiograph makes
legendary craft consequential rather than assigning a rarity tier.

The user's lean is one grammar rather than two, with golems and familiars as
boundary cases. **Review alternative:** one composable part/attachment
language, with multiple construction and growth productions and explicit
facets for agency, metabolism, repair, reproduction and development. A
body/object boolean cannot express a living weapon, an inert corpse, or an
autonomous constructed body cleanly. This preserves a common representation
without forcing a sword through axial animal development. Shared syntax
does not make all material or operator combinations admissible in every world.

Enchantment may also be a continuing external Bind or an intrinsically active
material. If Store becomes the umbrella, it should preserve these distinct
dependencies. Generating a stored chain still needs a feasible carrier, fuel,
capacity, gating, and effects; an arbitrary sampled chain is not yet a usable
item. Power decay belongs to the process governing that stored state;
forgetting belongs to knowledge retention. One must not happen implicitly
because the other is compacted. If remembrance fuels an item, explicitly
couple those accounts.

Length of history is not significance: repeated trivial use must not mint a
legend. Prefer consequential retained evidence and declared developmental
triggers. Raise needs stable identity and development; biological reproduction
or genealogical lineage can remain optional for an individual raised object.

#### Proposal: SRD adjudication over geometric anatomy

Target the D&D SRD with all its playable ancestries available by default.
Underneath, retain geometric targeting, targeted growth, optional physical
collision, and to-hit that can be independent of actual intersection. Organ
hits arise from called shots or incidental hitbox location. The accepted
tradeoff in the discussion is two descriptions of an event for swappable
rules and adjustable weight on player execution. Defaults remain open.

**Source check and scope:** the [official SRD page](https://www.dndbeyond.com/srd)
lists content exclusions; "every ancestry" must name a pinned SRD roster,
not all D&D publications. The [5.2.1 rules](https://media.dndbeyond.com/compendium-images/srd/5.2/SRD_CC_v5.2.1.pdf)
provide attacks, AC, damage, critical hits, and abstract hit points, not a
general geometric strike-quality or organ-injury rule. Called shots and
incidental organ consequences require an explicitly labelled extension.
The adapter version is not chosen by this note.

Two descriptions should yield one committed result. Preserve targeting and
contact evidence separately from the rules verdict; declare how a
non-intersecting successful attack selects an anatomical site. Exact SRD play
must not acquire extra organ disabilities or death thresholds from the
geometry layer. A hybrid can deliberately do so. Similarly, base SRD item
stats need an explicit equipment-profile mapping, not an inferred formula
from material and shape alone.

#### Questions retained for discussion

- Default weight of geometry versus SRD adjudication, and the anatomy policy
  when to-hit succeeds without an intersection.
- Whether roughly 20 examples is useful outside fauna; what grammar coverage
  the first 50 inspected outputs actually exercise.
- Graft seam reconciliation across local axes, attachments and resource flow;
  how a somatic graft becomes a heritable developmental rule.
  A candidate seam preserves each subtree's local axes, supplies a boundary
  attachment frame, and explicitly reconnects compatible functional ports.
  It need not force both bodies into one global symmetry or segment count.
- Whether plant and fungal development need branching, fusion, modular and
  indeterminate growth productions alongside segmentation.
- Continuous dimensions versus mechanical scale thresholds and named bands.
- Aura distribution/location/gating examples above, plus connectivity and
  the possibility of absent aura.
- Ring versus generated aptitude graph, including directed distance and how
  learned abilities behave when an aptitude changes.
- Bounds on second-order operations and feasibility checks for intended roles.
- Authored, sampled, or history-derived Raise conditions and resulting agency.
- One grammar or several productions sharing materials, operators, identity
  and part attachment; the user's current lean remains one grammar.
- Stored-aura decay, historical retention, and any explicit coupling between
  remembrance and continued power.

### 7.3 Missing generators (2026-09-09 discussion inventory)

**Status: proposed decomposition, not eight approved subsystems.** These are
missing responsibilities relative to §7.2; some extend existing generators.
The user broadly agreed with the previous review, without resolving every
open question or adopting a final architecture.

Live reads for this inventory: `mesocosm-core/src/world/generation.rs`
already samples constrained bodies and habitat, with admission/rejection
records, and `axis` supplies recipes, archetypes, branch layouts and appendage
chains. `process.rs` and `process/registry.rs` implement a small concrete
process vocabulary and its allocation admission. In Paredros, `src/items.rs`
places Food, Dressing and Scrap at world sites; it is not an item-form
constructor. `src/technique.rs` explains ArrestFall with body/equipment
implementations; it is not a generated technique grammar. These are scoped
code findings, not a whole-wing absence proof.

**Cross-wing correction, same discussion:** Mark explicitly reaffirmed that
Mesocosm and Isometry are available for reuse and development. Isometry already
has a bounded generator runner (`isometry-system::GeneratorRuntime`), typed
proposals for items, NPCs, maps, world facts, storylets and campaigns
(`isometry-campaign/src/generator.rs`), and host preview/commit wiring
(`isometry-genet/src/generators.rs`). Its campaign items also carry
system-interpreted material, enchantment, curse and origin modifiers.
The inventory below describes additional semantic depth; it is not a request
to rebuild that hosting, proposal, or item machinery inside Paredros.
Isometry's `WorldLaw` stores pack vocabulary; its presence alone does not
implement the proposed anatomy/operator semantics.

| Generator responsibility | Produces | Missing breadth |
| --- | --- | --- |
| Form and growth structure | Part arrangements, attachment sites, and rules for changing them | Extend axial/branched recipes with indeterminate branching, fusion, object assembly and chimeric boundaries; account for continuous dimensions and scale thresholds |
| Substances and sources | World-admitted material properties, carriers, transformations, and where supplies occur | Connect useful properties to available sources and processes rather than independently randomizing ore, organ and enchantment labels; typed matter storage is only part of this |
| World laws and relationships | Admitted causal families, couplings, compatibility, exceptions, costs and ranges | Instantiate a coherent local rule set, including whether aura exists; initially compose supported mechanisms rather than synthesize arbitrary new execution semantics |
| Functional anatomy | Allocation and connections between sources, stores, conductors, gates, actuators and senses | Generate a working internal network with capacity, bottlenecks, redundancy and failure; include functional adapters at grafts and item attachments |
| Techniques and aptitude | Supported operator compositions, their body/tool bindings, commitment and execution conditions, and aptitude costs | Generate feasible actions for actual bearers; include sensing, counterplay and interrupted outcomes. Cost-distance topology is an input here, not necessarily another subsystem |
| Development and training | Maturation, repair, acquisition, Raise conditions, and changes to growing items or companions | Generate future developmental possibilities tied to available experiences; actual growth follows recorded play, not fabricated accomplishments |
| Practices and institutions | Craft traditions, teaching, ritual procedures, bargains, prohibitions and beliefs about magic | Produce different ways people access and understand real mechanisms; accepted contracts and patron decisions belong to the social runtime |
| Evidence and historical expression | Discoverable traces, demonstrations, texts, scars, ruins, memorial forms and interpretations | Project admitted causes and significant history into things people can encounter; legends and rumors can disagree with truth, with that difference represented explicitly |

Items and creatures can consume the same responsibilities in different
combinations. An object needs a form and materials; enchantment may add an
operation, store or external relation. A raised object additionally needs
development and possibly agency. There need not be a separate enchantment
table or an independently generated legendary rarity value.

Not every generated rule needs a unique visual language. Generate readable
presentation from function: source/gate activity supplies tells, routes supply
traces, and failures leave evidence. World style and cultural interpretation
can vary those signals while preserving a learnable relationship to causes.

**Required machinery, not further generators:** execution, typed composition
validation, capacity/reachability checks, replay/persistence, and curation
precedence. They decide whether candidates work and preserve accepted results.
Seed independence, rule versions and explicit revision targets prevent later
curation or generator changes from rewriting an inhabited world silently.

Suggested first investigation joins form, functional anatomy and technique
generation under a small authored rule set. Sample a bearer with a supply,
route, gate, sense and effect; then inspect what happens when a relevant part
is cut, replaced or attached to an item. This tests structural breadth and
functional consequences before attempting generated cosmologies. Sample both
critter and constructed forms early; a large catalogue can follow.

### 7.4 Glyph canon, the journey, and divinity (2026-09-13)

**Status: G1/G2 implemented and tested; G3/G4 open, 2026-09-13; G5 landed
in Paredros 2026-09-14; G6 and G7 ruled 2026-09-15, not started.** On
2026-09-15 Mark ruled that **hagioglyph** names this whole organ, the journey
to divinity, and that **impresa** names the association organ (G7). Mark
explicitly authorized a module usable by Mesocosm, Paredros, and Isometry.
Sharing this bounded machinery proceeds now; sharing the products' entire
rule evaluators is not required. The journey to divinity itself becomes the
new god's motif and constraint. This section owns progression and world laws;
the [presentation plan](2026-09-11_orthographic_voxel_presentation_plan.md#glyph-effects-and-interaction-experiment-2026-09-13)
owns the glyph experiments and their rendering evidence.

#### World vocabulary and effect identity

A world admits a finite, versioned canon. Each base glyph has a stable ID,
display text, and a unique effect correspondence. The desired default covers
letters, numbers, and symbols; a world may use only `a`, `b`, and `c`, or a mod
may supply a much larger vocabulary. Arbitrary Unicode text is admitted as
display data. A character code, rendered shape, or font's glyph index is not
the gameplay ID. Explicit aliases and accent/variant definitions resolve to
a base and declared modifiers. Variants do not silently increase the set
needed for collection; a world wishing to require one makes it a base entry.
Font availability and legibility remain presentation validation, especially
for combining marks and visually similar signs.

The canonical correspondence is authored. A seeded shuffle permutes it while
preserving the one-to-one mapping, then the resulting definition is saved.
Reopening never reruns a newer generator over an inhabited world's meanings.
Effect IDs alone are not implemented effects: a product must admit a behavior,
cost, target rule, receiver interaction, and visible explanation for each.
The current three punctuation renderings are not an alphabet-sized spellbook.

Acquiring every base glyph in that world's canon qualifies the individual to
ascend or request a wish. An empty canon grants neither by collection. Other
routes, such as a named item or an extraordinary deed, are independent,
authored predicates; they need not invent missing glyphs. Whether a wish
consumes a collection, can repeat, or excludes later ascension remains a
world rule. A qualification receipt does not execute an unrestricted wish.
Canon changes during play require explicit migration of completion and
correspondence. They must not quietly revoke a god or make an old collection
complete under different meanings.

**G5, hagioglyph (Mark, 2026-09-14).** A glyph may vary over time by world
criteria: a canon revision the world publishes, caused by a period settling,
a hagiograph promotion admitted with its condition receipt, or an authored
epoch, and recorded in accepted history rather than rewriting it. Every
acquisition and grant keeps the revision it was accepted under; completion
and the ascension basis are judged against the journey's founding canon;
only the live effect of an owned glyph follows the current revision, so the
same glyph acquired at different times can carry different costs. The kernel
gains the revision stamp and a correspondence diff between two revisions;
Paredros is the first consumer, under its remembrance plan's stage F3b5.

**Hagioglyph names the organ (Mark, 2026-09-15).** Until this ruling the
word named only G5's time-varying glyph. It now names the whole of this
section's machinery: the canon, the journey, ascension, the chosen referent
and its periods, and the revisions above. This mirrors **hagiograph**, which
names the memorial organ rather than one memorial. A glyph under G5 is "a
glyph under revision" or "the live effect"; F3b5 in Paredros and R7 in the
effect pack preset plan are relabeled accordingly. The dependency stays
one-way: a hagiograph promotion is one cause of a canon revision, so the
divinity organ consumes the memorial organ and never the reverse. Crate
naming follows the ledger's tier rule; `shared/wing-glyphs` keeps its plain
name as the kernel under the organ.

**G6, chains and standing (Mark, 2026-09-15).** Effects compose into
chains, and the invoker's standing with a glyph is a per-individual,
per-glyph pair of ledgers, affinity and resistance, moved only by accepted
events under stated decay policies.

- A chain is a typed admission, the way the trophic grammar admits intake:
  links must be compatible by behaviour kind and receiver class, which the
  glyph experiment already requires before marks form strings, and chain
  length is a configured bound. Products adjudicate; the kernel owns the
  chain grammar and its receipts.
- A link's success reads affinity minus resistance and chain length, drawn
  from the seeded integer stream so replay holds. Success falls with chain
  length; a failure is an accepted event, never a silent roll.
- A failure raises the invoker's resistance to that glyph, capped, with a
  decay policy stated per world, so no run of ill luck locks a glyph for
  ever. Resistance is symmetric standing: it also applies against that
  glyph arriving from outside, so a failure becomes a defense.
- Affinity rises through the journey (provenance, successful invocations)
  and raises magnitude, including the self-targeted side effects an effect
  pack declares. High affinity is therefore dangerous when the side effects
  are unmanaged. Decay is evaluated lazily from the last event tick, never
  per tick, so it is deterministic and costs nothing at rest. Caps, decay
  and side-effect routing are explicit world policy, per the period rule
  above: no hidden bonuses.

**G7, impresa, the association organ (Mark, 2026-09-15).** "A god's
associations are their impresa, but other things could have theirs too."
Anything pointable, a glyph, a critter, a borg, a character, a faction, a
place, an item, a lot of nis, bears an impresa: the record of what it has
come to be associated with, and what has lapsed, over time. Plural imprese.
The expression table above is one static association, glyph to trait; the
impresa generalizes it across subjects and history.

- **One record type.** Subject identity, object identity, a kind, the canon
  revision, the cause receipt, the tick, and whether the record associates
  or lapses. Every record cites an accepted event; nothing is inferred from
  a name. Disassociation is a record, never a deletion.
- **Three readings of one record.** The accepted record is fact. An
  observer's held associations are belief, kept in the memory and belief
  layer Paredros already has, free to diverge from fact; reputation is
  belief. The hagiograph's promotions of associations that met a condition
  are legend. No reading is a second authority.
- **Kinds are a world-configured closed set**, seeded from claim, discover,
  experience, embody, invoke, defeat, and extended by packs the way the
  canon is; directional, subject to object. A lapse by decay is a record
  emitted at settlement, like a period.
- **Consequential records never compact.** A record cited by a later
  accepted event, by a god's ascension basis or by a faction's stance is
  retained; the rest fall under the retention budgets and compaction the
  owners paragraph already anticipates.
- **Cross-vessel, imprese travel as records**, which satisfies
  choices-not-morphology and pointable inheritance; each vessel maps kinds
  to its own meaning.
- **Home.** A plain `wing-` crate beside `wing-glyphs` owns the record type
  and its validation only; products own storage in their accepted history
  and derive per-subject and per-glyph readings as projections. Extraction
  into mere's linked-data edges, whose evidence-sense vocabulary landed on
  2026-09-15, waits for two consumers, and the kinds are designed so that
  projection stays possible. Prior art to read before the kinds are ruled:
  Dwarf Fortress legends mode and Caves of Qud's sultan histories. The
  Nemesis boundary stands: remembered grudges are fine, rival promotion
  hierarchies are not.

**Done when (G6):** a two-link chain on compatible declarations admits and
an incompatible one refuses by name; success, failure and the resulting
resistance change replay identically from a restored save; resistance
decays to its floor on a stated policy and blocks an incoming effect of the
same glyph in the meantime; a high-affinity invocation reports its declared
side effect on the invoker. **Done when (G7):** two vessels append imprese
for one identity through the shared record type without sharing a world;
a lapse and a re-association on the same pair are both pointable with their
causes; the belief reading of a faction diverges from fact after a witnessed
event and reconciles on a recorded one; a consequential record survives a
compaction that removes an unreferenced one.

**G5 amended, hagioglyph is the organ (Mark, 2026-09-15).** The paragraph
above names one mechanism, a glyph whose live meaning follows a published
canon revision. That mechanism is now called **canon revision**, after the
kernel's `canon_revision` stamp and `Canon::correspondence_diff`, and no
longer owns the word. **Hagioglyph** is the wing's glyph organ as a whole:
the canon and its revisions, expression through traits, the journey,
experience, standing (affinity and resistance, to be ruled) and ascension,
with `wing-glyphs` as its kernel and each product's glyph reading as a
consumer. The dependency arrow is unchanged: hagiograph promotes, a canon
revision follows, the hagioglyph reads it. Paredros's stage F3b5, its
document-host glossary line and the `paredros-world` glyph reading's
module comment carry the narrow sense and relabel to canon revision on
the Paredros side.

#### An individual has a journey, not just an inventory

Record original acquisition order, incarnation, simulation tick, and accepted
source evidence, including the means: ability, trait, technique, item, bond,
quest, event, or a mod-defined category. Losing an item is distinct from
forgetting that one learned through it. Reacquisition never rewrites the
first acquisition. Critter, borg, and character are all eligible; the record
belongs to the continuing individual, independent of taxonomy and controller.
Accepted append order is authoritative; source tick metadata never reorders
the journey across different clocks. `ascension_basis()` retains the exact
grant prefix at first ascension, even when later lives add further evidence.

Ascension enables reincarnation of that individual. It preserves the journey
while starting a new body's experience progression. Configured experience
thresholds restore inherent access in original acquisition order. Provenance
remains available at each unlock so a bond-earned effect can return as a
bond-dependent ability, while a practiced technique can require performance.
The later bag may expand without rearranging earlier acquisitions.

The same collection can therefore produce different divinities. As an
illustrative rule, a rescuer who learned through sheltering others might
recover protection through maintained bonds; a survivor who acquired that
glyph through enduring fire might recover it through exposure and endurance.
These are authored possibilities, not judgments inferred from event names.
Any generated affinity, cost, obligation, exception, or restriction must cite
the acquisition evidence and rule revision supporting it. Grouping acquisition
categories is an inspectable motif summary; it is not yet that rule evaluator.
Show the likely constraints before commitment, and retain the player's means
of understanding, fulfilling, or deliberately challenging them.

#### Chosen divine power and periods

Collection determines available vocabulary. Divine power depends on the
chosen referent: a condition, event, entity kind, or another admitted metric.
The god chooses its source and measurement period before the period begins,
then chooses again at its end. The definition includes world/reference scope,
predicate revision, unit, aggregation, and the simulation interval `[start,end)`.
A name such as "popular" is insufficient: event occurrences, distinct actors,
and entity-ticks are different quantities. Products supply accepted observations;
the shared ledger never samples a renderer, wall clock, or arbitrary biography.

For the first kernel, the explicit policy is accumulation followed by settlement
at the end. Every simulation tick contributes one aggregate, including zero.
Missing ticks cannot be mistaken for zero; duplicate or out-of-order input is
rejected. Choice stays fixed, arithmetic is checked, and settlement is one-time.
A long period delays access and accumulates more of a continuing source. A
short period offers earlier access and an earlier chance to change dependence.
With identical observations, dividing a period does not multiply its total.
Additional interest, decay, averaging, caps, stockpiles, spending, and ongoing
access during a period need explicit world policies rather than hidden bonuses.

Journey and referent are distinct but connected. A world's journey rules may
restrict eligible referents, alter their costs, or impose obligations; a chosen
referent does not erase the history that made the god. Reference definitions
and active commitments must survive death, replay, and save restoration under
the product's chosen reincarnation rules. The first accumulator proves period
mechanics, not persistent divine economy or a universal predicate language.

#### Owners and implementation gates

`shared/wing-glyphs` owns canon validation, deterministic correspondence,
acquisition history, ordered unlocks, explicit progression transitions, and
fixed-period accumulation. It has no graphics or product dependencies.
Cloned canons share immutable indexed storage, with logarithmic identity
lookups. Individual journeys store their acquisitions and references without
copying the entire vocabulary. Exported snapshots embed the full canon for
standalone restoration; a future world save can deduplicate that carriage.
Canon size and journey retention budgets are explicit configuration, not a
fixed alphabet ceiling. Large mod canons can raise those budgets and the JSON
ingress limit. Every successful JSON export must fit its corresponding import
budget. Rejected grants leave evidence intact and report exhaustion; silently
evicting an original acquisition would change the future god. Longer-lived
worlds will need compaction preserving those consequential records.
`wing-functions` keeps body-bound resource routing and operator receipts;
`wing-formats` keeps primitive interchange framing. Neither needs to become
a magic scheduler. Glyph display and strings consume admitted effect facts;
their color, animation, and generated layout cannot award an acquisition.

Mesocosm's optional disposable trial maps positive accepted movement, feeding,
and carving records through configured grant rules. Actual history ordinal,
tick, and organism identify the evidence. Trial reset reconstructs the same
journey. This is an integration experiment, not a new field in durable `World`
or a playable reincarnation feature. Isometry's system-resolution boundary
owns adjudication; Paredros's continuing subject identity is distinct from a
body revision and from succession into another existing life.

1. **G1, shared kernel, implemented:** validate small/empty/Unicode canons, variants and
   seeded bijections; preserve first acquisition, source, and order; prove
   non-vacuous completion, reincarnation unlocks, fixed-period totals, rejection
   without mutation, and validated journey restoration. The period accumulator
   has inspection serialization only; durable period restoration is G4.
   No effect execution claim.
2. **G2, real consumer, implemented:** optional Mesocosm rules consume accepted world events;
   refused actions cannot grant the action's glyph; baseline/reset/replay and
   repeated reads reproduce the same records. No player-succession shortcut.
3. **G3, usable vocabulary:** admit a default effect pack and two contrasting
   journey rules through product adjudication. Explain their costs and motifs,
   execute receiver interactions, and show distinguishable spatial results.
4. **G4, durable lives and divinity:** products admit world/canon/journey save
   ownership, alternate ascension and wish policies, same-individual rebirth,
   active source commitments, and spending. A second product exercises the
   shared API with its own identity and law rather than importing Mesocosm's
   ecology. Mods and world migrations have explicit rejection and repair paths.

#### First implementation receipt

The new MPL-2.0 `shared/wing-glyphs` crate passed **16 tests** at ac38887 (28 as of 2026-09-15, after the pack and expression tables), including a
literal seeded-shuffle vector, opaque Unicode and explicit variants, a
5,000-entry canon with shared storage, first-acquisition preservation,
divine-only rebirth, immutable ascension evidence, snapshot transition replay,
and five fixed-period tests. The period tests include zero support, ordering,
overflow, one-time settlement and additive short/long periods. Snapshot replay
checks consistency, not the authenticity of a caller's evidence.

Mesocosm's final release runtime suite passes **44 tests**, including five new
consumer tests. They exercise real accepted carving, movement and feeding,
zero-removal/rejected actions, unchanged core history/hash, exact reset/replay,
and actual control transfer followed by another body's carve. The collection
does not follow that control transfer.

The [public-API example](../crates/mesocosm-runtime/examples/glyph_journey.rs)
produces [this JSON receipt](../testing/glyphs/journey.json). An unmodified
seed-0 founder removes 15 voxels at `[0,12,0]`, earning the one configured base
glyph. Its four-tick metric observations are `15,0,0,0`; settlement is 15.
The copied shared journey qualifies for both routes, ascends, reincarnates,
retains its original source, and restores inherent access at 10 experience,
with none at 9. Experience is explicitly supplied to this isolated probe.
The core world does not reincarnate, award that experience, execute the
referenced effect, or mint divine power. Baseline hash is `765c055b377dfb62`;
the ordinary four-action world ends at `ea76e63d1fb9fa13`.

Validation commands were `cargo test --offline` from `shared/wing-glyphs`, and
`cargo test --release --frozen -p mesocosm-runtime --lib -j3` plus
`cargo run --release --frozen -p mesocosm-runtime --example glyph_journey -j3`
from `mesocosm`. The product lockfile is locally generated under its existing
ignore policy; the standalone shared crate's lockfile is tracked. These are
CPU correctness receipts. This slice changes no native view and supplies no
new visual or performance acceptance claim.

## 8. Discovery as the delivery vehicle (proposed)

If the laws differ per world, **learning them is the content**. NetHack
shuffles appearance-to-identity per game and deliberately supplies more
appearances than items, so elimination stays imperfect. Morrowind's
alchemy hides most ingredient effects and combines by intersection.
Ultima Ratio Regum makes religious identity *inferable from observable
behaviour* rather than readable from a panel.

A hidden effect vector is not yet discovery (review, 2026-08-07). The
loop that makes it one:

```text
world truth
  -> exposure or signal (the carrier's emission profile)
  -> observation (observer's senses, instruments, position)
  -> remembered claim or hypothesis
  -> experiment or consequential choice
  -> confirmation, revision, or deception
```

`Signal` is the landed seed: advertised appearance already disagrees with
actual venom or trophic behaviour, so the claim/truth split exists. F3
builds the rest of the loop on that split.

**Open question the plan must eventually answer: who owns a discovery?**
The current animula, the biological lineage, the world record, shared
players, or a combination. Without a locus, every world's generated laws
are rediscovered from nothing each run and no culture of knowledge
accumulates. `tulpa` (the retold subset) is the wing's existing vocabulary
for exactly this kind of memory. **(2026-09-02 note: this organ is now
called hagiograph; "tulpa" has been renamed to gemot's federated
adapter-training lane; see repo `CLAUDE.md`.)**

This distributes across the wing without any vessel converting anything:
**Mesocosm discovers, Paredros embodies** (transformations, pacts, curses
as syndromes), **Isometry exploits** (auras and fields as tactical
terrain).

The cheapest first version is worth building before anything ambitious: a
small fixed-width effect vector per organism, combination by intersection.
It turns the existing procedural ecology into a magic-materials economy at
nearly no cost.

---

## 9. Fields (proposed)

The most implementable "magic as simulated quantity" model found is,
unexpectedly, Genshin's elemental gauge theory: application in gauge
units, an aura tax on creation, linear decay inversely proportional to
base duration, consumption on reaction, per-source internal cooldowns. It
is deterministic and integer-friendly, which matters here more than it
matters there. Divinity: Original Sin 2 is the same idea expressed
spatially, as persistent surface state with combination rules.

For a place graph over voxel ground the natural form is a **small vector
of channels per place**, with units, decay rate, and reactions, read by
spawn rules, terrain, and the ecology's mortality terms. Vintage Story's
temporal stability and Black & White's belief-generated influence radius
are the same shape at world scale.

Breath of the Wild supplies the law that keeps such a table from
exploding: elements act on materials, materials do not act on materials.
The asymmetry is what makes a small rule set multiply rather than square.

---

## 10. Gates

### Ecological, scheduled

**E0. Allometry, and the size ceiling.** `[implemented 2026-08-07]` Replace flat rate constants with
mass-derived rates; fix linear upkeep to a ^0.75-shaped law in integer
arithmetic.
**Done when:** a world sustains bodies across at least three orders of
magnitude of mass; maturity, lifespan, gestation, and feeding rate all
vary with mass; existing ecology receipts are re-greened rather than
deleted; no rate constant remains that should have been a function of mass.

**E1. Predation, and feeding mode from anatomy.** `[implemented 2026-08-07]` Consumers may take
living consumers. `Kingdom` becomes a derived reading of anatomy rather
than a genesis assignment.
**Done when:** a lineage whose bodies acquire the relevant parts begins
taking live prey with nothing in the code naming it a predator; trophic
levels are countable in a run; and the reckoning can distinguish predation
from scavenging (it already can, by event order).

**E2. Dispersal.** `[implemented 2026-08-07, epoch receipt open]` Movement as a far-tier process: diffusion, plus
starvation-driven migration.
**Done when:** populations track productivity across the place graph over
an epoch, and a locally exhausted place is left rather than starved in.

**E3. Cohorts.** `[conservation slice implemented 2026-08-07, capacity gate open]` Far-tier state becomes cohorts with trait-space merging;
near tier stays individual. Promotion and demotion are cohort split and
merge.
**Done when:** biomass is conserved across every merge and split; far-tier
outcomes stay within existing receipts; and the population the far tier
can carry rises by an order of magnitude.

Current receipt: deterministic cohort formation, exact scalar split/merge
conservation, and promotion/demotion counts. Persistent far-tier storage and
the 10x capacity measurement are not yet claimed.

**E4. Drives replace the `Hunter`.** `[implemented 2026-08-07]` Behaviour selection scores reachable
affordances against need and body. The FSM demotes to a probe (see
place-graph plan G3) and leaves the tree.
**Done when:** the chase receipt still passes with no type named `Hunter`
in the path; a slow armoured starving body does something *different* from
a fast large-mouthed one under identical conditions; and a predator that
picks badly starves.

### Acceptance gates, open (review, 2026-08-07; renamed by audit the same day)

**The accurate status is: implementation slices landed; acceptance
gates open.** The first version of this section called these "follow-up
receipts," which the audit correctly flagged as hiding open gates. E1
still uses interim anatomy categories pending PD1b; E2 lacks its epoch
receipt; E3 retains the individual roster as authority and lacks the
capacity proof; E4 selects feeding targets, not yet hunt, migrate,
avoid, graze, and rest through one selector. Each gate below is OPEN
until its condition holds:

- **E0**: allometry becomes a configurable *baseline* modified by active
  tissue/process allocation, metabolic mode, and environment; and every
  derived rate must produce a **derivation trace** on demand, not only
  the right number.
- **E1**: the PD1b-backed form (satisfied feeding processes; trophic
  labels as projections of realized activity) replaces the interim
  anatomy reading; `Kingdom` decomposes into independently realizable
  strategies.
- **E2**: matter, population, and causal movement are **recorded across
  place boundaries**, so migration is attributable, not just simulated.
- **E3**: cohorts inherit the phenotype plan's contract, not Madingley's
  identity-free one: lineage and count; mass and energy; developmental
  distribution; causal seed; **exact refusal to aggregate named, played,
  injured, or chronicled subjects**; deterministic materialization
  without rerolling. Before claiming scale: define **sufficient
  statistics per process** (age structure, reproductive reserves,
  process-expression distribution, lineage contribution, carriers,
  seeds), then a **paired near/far equivalence scenario**. Biomass
  conservation alone is not the receipt.
- **E4**: the done-condition strengthens from "nothing named `Hunter`"
  to **"the same decision machinery produces hunting, migration,
  avoidance, grazing, and rest from different bodies and needs."**

### Fantastical, proposed only

**Not scheduled. No adoption decision. Listed so the shape can be argued
with.**

These historical F-gates remain proposals. The authorized G1–G4 glyph lane
in §7.4 is a separate current implementation sequence; its canon and journey
do not wait on the earlier F0 prohibition on shared types or registries.

Reordered 2026-08-07 (review): **the proof precedes the abstraction.**
The first draft's F0 was a registry, which is declaring the portable
profile in advance; the sequence now follows the evaluator rule.

- **F0. One fantastical vertical slice.** One unusual carrier state, one
  cost, one application route, one discoverable consequence, implemented
  concretely inside Mesocosm's own rules. No registry, no shared type.
  **Direction ruled 2026-08-07 (Mark): impossible ecology, not
  spellcasting.** Candidates in the founding record's wording: an
  organism that metabolizes remembered events; migration following
  kinship rather than distance; a predator that consumes names or
  affinities; a body incorporating architectural material; a place that
  develops organs. Each is a perturbation of continuity, which is the
  wing's question wearing the fantastical layer.
- **F1. Effect envelope, extracted.** Only if the F0 slice and an
  ecological effect (venom is the standing candidate) genuinely repeat
  the same application shape does the §5 envelope get extracted.
- **F2. Derivation.** §6: signatures, similarity, contagion, over existing
  trait vectors and provenance edges. Kleptoplasty criteria.
- **F3. Discovery.** §8's epistemic loop on `Signal`'s claim/truth split,
  cheapest version first, with the ownership locus decided.
- **F4. Fields.** §9, per place, integer.
- **F5. Grammar.** §7, only after F0-F2 exist to generate Forms *from*.
  The fixed Technique axis is a **world profile, not engine law**; and a
  closed-form cost function prices effects but cannot establish balance
  without scenario sampling.
- **Registry, if ever**: follows the second working state family, never
  precedes it.

---

## 11. Stop rules

- **No second engine.** The fantastical layer is carriers, effects, and
  derivation rules over the existing machinery. If it grows its own
  simulation loop, it has become the thing the anti-Spore law forbids.
- **No shared evaluator before two sovereign proofs** (phenotype plan's
  rule, which the first draft violated in spirit). Common grammar,
  sovereign rule systems, extraction after repetition. **2026-09-13 exception:**
  Mark explicitly authorized the bounded shared glyph canon, journey, and
  period machinery in §7.4 before a second consumer. Product world-effect
  adjudication remains local; the old registry sequencing rule does not block
  this slice.
- **State legible, dynamics throttleable** (§1.2). Every layer must leave
  inspectable, attributable state behind if its dynamics get cut.
- **No name coined without a naming round.** §1's empty slot is a finding,
  not an invitation.
- **Do not model a plan on an unshipped system.** Dwarf Fortress's full
  procedural magic does not ship; spheres, secrets, syndromes, and
  primordial remnants do. Cite what exists.
- **Sample constraints, not powers.** Sanderson's Second Law: a generator
  over powers produces noise, a generator over limitations, weaknesses,
  and costs produces character.
- **Generated rules must be discoverable** or generation has bought
  nothing a content pack would not have.
- Do not let effects, fields, or currencies become world truth in place of
  the integer authority. They are state *in* the world, not a second
  authority over it.

---

## Findings

- **2026-09-21, from the wing design record's rulings 80 and 82:** the sim's
  middle tier of keeping, "of note", is ruled to be a literal note on a
  thing, and the note is ruled to be the impresa record, so a thing is of
  note while it bears a live impresa and collection is lapsing. Mark also
  wants a freeform text field beside the generated details, format a
  setting, djot preferred, edited what-you-see; knot-editor and
  `knot-document` are the stack's djot authority. `wing-impresa`'s `Record`
  is closed today (`deny_unknown_fields`, schema version 1) and holds to
  "transcription and never inference", which authored text is not. Owed
  here: whether the text is an optional field on the record or a sibling
  record beside it, and the version bump either way. **Updated the same
  day, ruling 85:** Mark proposes "closed subset, open superset". The
  closed `Record` stays as it is and a note is an open envelope around it in
  the storing product, so nothing is owed in this crate beyond, at most, a
  world adding a kind for a player's own jotting.

- **2026-09-13, glyph ownership inventory:** `shared/wing-functions` owns
  resource routing; `shared/wing-formats` owns primitive framed records.
  Neither supplies acquisition history or divine progression. Mesocosm's
  `runtime::trial` exposes exact accepted history after normal core stepping.
  `isometry-system/src/sys/system_actions.rs::resolve_action` is the tabletop
  rule-resolution seam. Paredros's `SubjectId` and `BodyRevisionId` distinguish
  person from body, while ordinary succession changes the subject. These
  findings support the separate `wing-glyphs` module and an optional trial
  adapter; they do not establish shared persistent souls or rule execution.

- **2026-08-06:** the GEM-shaped slot for a general model class of
  fantastical ecology is **empty** (§1), verified rather than assumed.
  Adjacent named things exist: GEM, speculative evolution, computational
  modeling of religion, thaumatology.
- **2026-08-06:** Mesocosm's ecology already implements six of Madingley's
  eight process slots (§3). The gap is predation and dispersal.
- **2026-08-06:** `upkeep_mg` is linear in body mass where allometry says
  ^0.75, imposing an unchosen size ceiling (§3.2).
- **2026-08-06:** the wing already carries hooks for all three classical
  derivation rules: trait vectors for similarity, morphology for
  signatures, and `Provenance` edges for contagion (§6).
- **2026-08-07:** the entity skeleton is four layers, not an ECS:
  identity, facts, derivation-as-cascade, projection-with-identity
  (§6.2). PolyCSS supplied the naming prompt; livery supplies the tamed
  prior art; sprite stacking's pulled-back-camera deferral has expired.
- **2026-08-07, review pass (accepted nearly whole):** the plan's two
  load-bearing corrections are (1) "one model with settings" replaced by
  **one causal grammar, sovereign evaluators, extraction after
  repetition** (§0), and (2) the skeleton gaining its **transition**
  layer (§6.2), without which it described a simulator schema and not a
  game. Also accepted: typed state carriers over a currency type (§4.1),
  effects *propose* typed changes (§5), feeding as satisfied processes
  with trophic role as a summary of realized activity (§3.3), cascade
  inputs de-circularized with declared combinators (§6.2), the cohort
  lineage contract and sufficient statistics (gates), the epistemic loop
  and the discovery-ownership question (§8), and per-gate follow-up
  receipts. Verified against the paper: Madingley spans **10 µg**, and
  randomizes **cohort order**, both corrected. One nuance retained
  rather than conceded: carriers keep an *emission profile*;
  observability is the relation between that profile and an observer,
  which is `Signal`'s existing split.

### Verification debts

Carried honestly from the research sweep, to be settled before anything
in §6-§9 is built on the details:

- The Caves of Qud state-machine-plus-replacement-grammar description and
  the "rationalise cause and effect after the fact" framing come from
  search summaries and paper metadata; the FDG 2017 PDF itself was not
  extracted. Re-verify directly.
- **Dominions' province-level belief propagation was not verified** and
  looks like one of the stronger missing examples. Worth a dedicated look.
- Spore's part-capability property claim is forum-sourced only.
- Sanderson's Third Law was not read at the primary source.
- Bay 12 dev pages were only partially captured verbatim.
- Not investigated for budget: Populous, Dungeon Crawl Stone Soup's
  schools, Niche, Cataclysm: DDA's Magiclysm spell schema, Potion Craft's
  navigable 2D effect space.

## Progress

- **2026-09-13, glyph G1/G2:** adopted §7.4 and implemented `wing-glyphs`,
  with a configurable accepted-event Mesocosm trial adapter and public-API
  example. Sixteen kernel and 44 final runtime tests pass. Review added shared
  indexed canon storage, configurable budgets, a divine-only rebirth gate,
  transition-validated restoration and immutable founding evidence. G3/G4
  retain default effect content, generated journey restrictions, durable
  reincarnation, wish execution, power spending and second-product admission.

- **2026-09-09, body/operator discussion:** §7.2 preserves the submitted
  proposal and open questions, with review notes distinguished from it.
  Verified current Axial/Branched generation and part attachment seams;
  checked the SRD roster and adjudication boundary against official sources.
  No implementation or default-weighting decision.

- **2026-09-09:** §7.1 records the renewed procedural-system scoping:
  distinct causal families, explicit couplings, world variation through
  laws/embodiment/practice/presentation, discovery and retention obligations,
  and research lanes preceding owner-specific implementation. Corrected
  earlier cheapness and automatic-balance claims. Documentation only.

- **2026-08-06:** founded. Ecological half scheduled as E0-E4; fantastical
  half proposed as F0-F5 pending a ruling. Prior-art ledger and
  verification debts recorded. Research sweep covered ~20 games, the
  design-theory literature, and historical schemas; the Ultima Online
  postmortem supplied the binding caution.
- **2026-08-07:** §6.2 added: the four-layer skeleton and the
  styling-as-derivation correspondence, from the PolyCSS reading session.
- **2026-08-07:** E0-E4 implementation slice landed. Integer allometry,
  body-derived feeding and predation, graph dispersal, far-tier cohort
  conservation, and anatomy-driven drives are covered by core receipts;
  `places::Hunter` was retired. The full offline workspace test suite is
  green. E3's authoritative far-tier storage and 10x capacity gate remain
  open by design.
- **2026-08-07:** review pass folded in: grammar-not-model, transition
  layer, typed carriers, satisfied-process feeding, cohort contract,
  epistemic loop, F-gates reordered proof-first, two Madingley facts
  re-verified at the source.
- **2026-09-02, terminology note (doc only):** Mark authorized renaming the
  memorial organ to **hagiograph** and reassigning **tulpa** to gemot's
  federated adapter-training lane. A dated note was added at this doc's
  tulpa mention rather than rewriting the historical text; see repo
  `CLAUDE.md`. No code changed.
