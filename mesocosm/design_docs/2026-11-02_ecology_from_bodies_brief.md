# Ecology from bodies and selection

**Status:** 2026-11-02: design brief from a conversation with Mark, who
agreed with all seven of its ideas, the seventh most of all (§1). It is not a
plan: it changes no code, and the work it implies belongs to the plans named
in §6. Mark's words are quoted with their date.

**For:** how Cosmograph's ecology could stop being assigned (roles founded by
fiat, rates tuned per role until a web holds) and instead fall out of bodies,
physics and selection. It reads the S10 arms (sim plan Findings, 2026-11-01),
the world-time retune (cases 1077, 1078, 1103 to 1116) and the adaptation
phase (cases 1083, 1094; overlay plan EB7). Code paths are under
`shared/cosmograph/src/`.

## 1. The idea, in Mark's words

- 2026-11-01: "surely things don't live on leaves alone...? and if so, they
  wouldn't evolve to do so if there wasn't an abundance of producers anyway"
- 2026-11-02: "single bites probably shouldn't take out entire plants to
  begin with... why wouldn't grazers eat the leaves instead of the parts of
  the plant that produce leaves? or fruit, etc."
- 2026-11-02: "how should adaptation reach the sim? except through deep
  time?"
- 2026-11-02, on this brief's seven ideas: "agreed... i absolutely agree with
  1-7, 7 most of all"

## 2. Where the web stands

- **The S10 arms** (40cf7241, f346d32a, 006e3d08) found the collapse
  structural: a bite built body one for one, feeding had no hunger gate, a
  bite killed a grazer-sized producer, and each level could grow as fast as
  what it ate. Halving upkeep made it worse; rates rederived for a lossy meal
  kept the boom, because intake was sized to a fixed breeding pace.
- **Case 1078** ruled the structural fixes: feed while hungry, meals mostly
  to soil (`world:detritus`, 1091), producers many bites large, breeding by
  surplus, a biomass pyramid. Grazing works on bodied producers (1105), a
  part is exposed by its cells' position (1110), and a plant keeps a root
  store no bite reaches (1114).
- **The year bar passes** on a laid map on all five seeds (1109, 1116), but
  producers peak in months eight to nine and fall about 40 per cent by month
  twelve while grazers rise sixfold; a two-year run comes before people.
  Bodies breed on their own clocks (1104); crowding sends animals on and
  scarcity slows breeding (1111); air and water are read from terrain (1103).
- **The rates are still a table**, per role and per stage, in world time
  (`ecology/rates.rs`, 1077), tuned until the bar holds.
- **Selection is ruled but coarse in name only.** Every lineage adapts at
  the boundary by growing a copy (Ecology 1, 684); the player picks among what
  prospered and a coarse copy over years tests each (1094). Today's coarse
  copy, grouped execution plus E4's bands, is slower than the exact one at
  every population measured, 490 to 9,015 members (EB7): a copy that skips
  ticks does not exist yet.
- **Deep time** is the same sim run epoch by epoch with no hand
  (`directing/interim/deep.rs`, Record 7), and a player starts at
  habitability (Mesocosm 3). Variation is drawn and selection decides; rates
  are never tuned to earlier medians (Bodies 11).

## 3. Seven ideas

### 3.1 Every rate from body size

Metabolic rate scales with body mass to about the three-quarter power, and
with temperature by a Boltzmann factor (Brown et al., "Toward a metabolic
theory of ecology", 2004). From that one relation follow, roughly:
per-mass metabolism as M^-1/4; lifespan and generation time as M^1/4;
population density a site supports as M^-3/4; home range about as M.

- **What exists:** bodies have mass (`lineage/body.rs` `mass_mg`); speed
  already scales by size (1085, 1112's relative Froude).
- **What it would take:** upkeep, meal size, maturity, lifespan and birth
  spacing read from a body's adult mass and the site's temperature instead of
  a role's row in `rates.rs`; a role keeps only what its body cannot say. The
  sim is integer, so the powers come from a fixed-point table or an integer
  root, the same on every peer.
- **Why:** the table the food-web lane tunes becomes a law with a few
  constants; a small fast breeder and a large slow one fall out of their
  bodies; the ten-per-cent rule becomes metabolism, not a setting.
- **Measured by:** the year bar (1077) with the table removed, and the
  biomass pyramid (1078) read rather than founded.

### 3.2 Food quality, not just quantity

Leaves are plentiful and poor, seeds and fruit rich, meat richest and rarest.
What a body extracts depends on its digestive system and the tissue it eats;
a long herbivore gut costs upkeep.

- **What exists:** tissue lives in parts with functions (Bodies 3, 5);
  systems such as `system:digestive` are realized by functions (Bodies 10);
  defensive tissue sets a bite threshold (Bodies 14); diet is read from what
  a lineage eats (case 1055).
- **What it would take:** a tissue's yield per function eaten, an extraction
  efficiency per digestive system, and toxins against detox functions.
- **Why:** specialist against generalist becomes a trade-off, plant defence
  against detox an arms race, and Mark's "things don't live on leaves alone"
  a consequence: a leaf-eater needs abundant leaves because leaves are poor.
- **Fruit** is the first thing one lineage grows for another to eat (the
  cue sent to 1078's lane), filling EB5's empty Symbiosis axis.

### 3.3 Persistence is regional

Local populations crash in nature; webs persist because patches crash out of
step and recolonize each other (Huffaker's mites, 1958; Levins'
metapopulations, 1969). S10 asked eight sites never to crash.

- **What exists:** regions derived from biomass, collapsing when a level is
  gone (Ecology 2); scatter to neighbours (1089, 1092, 1093); crowding sends
  animals on (1111).
- **What it would take:** mostly measurement. A web is healthy when its
  region keeps every level while sites blink in and out; Ecology 3's four
  measures read per region; worlds large and patchy enough to hold
  asynchrony.
- **Why:** a local collapse becomes an event in play (flee, recolonize), not
  a failed run.

### 3.4 Dormancy and seasons

The real stabilizers are buffers: seed banks outlast a bad year; animals
hibernate, estivate or migrate through a lean season.

- **What exists:** seasons in the light (case 973), seasonal seeding (1078),
  scatter, migration (1111).
- **What it would take:** a dormant stage (a seed, a cyst, a torpid body)
  paying little upkeep, eaten only by the bodies that can (granivores as a
  niche), waking on a sensed condition (1071, with one prior reading per
  sense from 1079).
- **Why:** troughs are survived rather than tuned away; the year-two crash
  of 1116 is the kind of trough a seed bank carries.

### 3.5 Disease that kills the winner

A density-dependent pathogen spreads fastest through a booming population,
the ceiling of S10 as biology ("kill the winner").

- **What exists:** disease is Ecology 3's first disturbance; an
  incorporated part keeps its donor's matter (Bodies 5); Mesocosm's
  vocabulary already holds kleptoplasty.
- **What it would take:** a lineage living in or on hosts, spreading by
  contact at a rate read from crowding, costing its host upkeep or cells.
- **Why:** booms end without a rule saying so, and parasite and symbiont
  are one mechanism with opposite signs.

### 3.6 Creatures that change the terrain

Burrows, dams, mounds and grazing lawns (ecosystem engineers: Jones, Lawton
and Shachak, 1994). Grazers fertilizing where they graze, with detritus
turned to soil (1091) and plants sensing soil (1071), would make lawns that
pull the herd back.

- **What exists:** `Effect::Terrain`; laid terrain read for air and water
  (1103); Mesocosm's voxel ground.
- **What it would take:** acts that dig, pile or dam, priced in matter and
  effort, their results outliving the maker.
- **Why:** fills EB5's empty Construction axis, and the world becomes a
  record of who lived there.

### 3.7 Don't tune the world; evolve it

Life-history traits are drawn per lineage and revised at the boundary
(Bodies 7). Make the rates' constants heritable around 3.1's law, let deep
time select them, and offer the player worlds whose webs persisted.

- **What exists:** the boundary's adaptation by grown copies (Ecology 1);
  deep time as the same sim (Record 7, `deep.rs`); prehistory handed over by
  genesis, the player arriving as heir (`wing-organs` feat rule); starting at
  habitability (Mesocosm 3).
- **What it would take:** heritable rate constants with bounds a world sets;
  worldgen running deep time until regions persist (3.3's measure), then
  offering survivors; and a step cheap enough to run it (EB7).
- **Why:** the stable web is not designed but what survived; tuning becomes
  selection, which is how the doctrine already says lineages change. Mark
  favoured this one most.

## 4. How they fit

3.1 and 3.7 are the core: body size gives selection a sensible starting point,
and selection finishes the tuning. 3.3 is a change of measure that makes 3.7
honest (a world persists by region). 3.2, 3.4 and 3.5 are the mechanisms
selection works on; 3.6 is where the world remembers it. Fruit (3.2) and
dung lawns (3.6) are the first symbioses no one authors.

## 5. Tensions

- **Selection needs a cheap step.** 3.7 runs deep time over many worlds, and
  so does 1094's adaptation copy; neither runs on today's tick-by-tick sim
  at a minute (EB7). One coarse step (case 1075's coarser rung, or E6's rate
  models) serves worldgen, the adaptation phase and the VTT's downtime.
- **The year bar and 3.3.** 1077's bar asks every role alive at month twelve
  on all five seeds; 3.3 reads it per region. A world-level bar is
  compatible; a per-site reading is not.
- **3.1 and the A1 gate.** A1 was tuned for play at the minute; deriving
  rates from size moves its numbers again (1077 already re-measures it).
- **Heritable rates and determinism.** Constants evolve as integers, drawn
  and inherited like any trait; nothing floats in the sim.
- **Mass or leg length** (the PG7 and travel session, 2026-11-02). Speed
  reads no mass: `speed::Speed::of` reads the current body's mean leg length
  (hip to sole over `isometer_core::legs::chains`), or body length where
  legless. 3.1 scales by mass. The two read one figure only by a choice, and
  they are not interchangeable: a long-legged light body differs from a
  stout one.
- **Terrain churn** (same source). 3.6's path exists: `Effect::Terrain`
  carves or deposits at the base cell (1 mm in Mesocosm), recorded as a
  command, bumping `terrain_revision`; Places re-derive the dirtied region
  (about 0.14 s for a whole 256-cell window), and every cached route and
  travel choice keys on the revision. Many diggers a tick would re-derive
  often, so edits would batch per tick or per region.
- **Dormant bodies in a round** (same source). Out of view, a body that
  does not choose stays posed at no per-tick cost. In view, a framed body
  takes one of the foreground's 16 slots (1086) whether or not it can act;
  whether the foreground skips bodies that cannot act is the foreground
  layer's question.
- **Roles as readings.** 3.1 and 3.2 push toward roles measured, not
  founded (case 1055); the web of diets' five founded roles become a
  generator's starting draw, not the web's definition.

## 6. Where the work would land

- 3.1, 3.2, 3.4: the sim plan's S10 and TU, with the food-web lane (1077,
  1078); bodies in the bodies plan.
- 3.3: the sim plan's S10 measures and Ecology 2's regions.
- 3.5: the bodies plan (incorporation) and S10.
- 3.6: the bodies plan and the presentation plan's terrain.
- 3.7: the sim plan's S5 (deep time) and the overlay plan's EB7; the coarse
  step in the execution plan (E6) or the time ladder (1075).

## 7. Open, for Mark

- Which comes first after the two-year run (1116): 3.1's size law, or 3.3's
  regional measure, which needs no new mechanism.
- Whether 3.7 evolves worlds (worldgen offers only persisting webs) or also
  keeps evolving them in play, every boundary revising rate constants.
- Which constants are heritable, and within what bounds a world sets.
- Whether 3.1 scales by mass, by the leg length speed already reads, or by
  both (§5).
