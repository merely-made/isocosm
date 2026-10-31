# Creature minds: learning in the crowd and out of it

**Status:** 2026-10-31: design brief from a conversation with Mark. Nothing in
it is ruled. Mark's words are quoted with their date; everything else is a
proposal, a reading of the records or a question. It is not a plan: it
changes no code, and the phases it implies belong to the plans named in §6.

**Names** are Naming 1's (cases 1036 and 1037): the sim is Cosmograph; Isocosm
(the VTT), Mesocosm and Eponym are peer products. Code paths below are under
`shared/cosmograph/src/`.

**For:** how a creature decides and learns in Cosmograph, cheaply in the
aggregate and individually where it matters; how the player's directions
become a learned, consequential relationship; how roles and sapience are
measured rather than declared; and what that gives Mesocosm, Eponym and the
VTT. Prior art: the [prior-art brief](2026-09-18_sim_prior_art_brief.md)
(Creatures, Bibites, Madingley, quality diversity); the reduction literature:
the [aggregation research](2026-09-22_aggregation_research.md).

## 1. The idea, in Mark's words

All 2026-10-31.

- "each one of these little cells are not programmed to act, but that they
  learn by being a neural network"
- "by picking individuals out of the crowd, you will have essentially primed
  and started the adaptation of a neural net and the individuation from the
  crowd"
- "the trait governing is the real atomic unit there. You don't want to go
  too deep past that"
- "organize, you know, behavior essentially by collective until things
  distinguish themselves, and then we get them individualized"
- "you direct critters in mesocosm; you name and embody a denizen in eponym;
  you control a character or a party in isocosm. all three rely on the
  sim... which should serve as all three's core, cheaply in aggregate and
  able to individuate meaningfully"
- "my instinct was to seed some behavior associated with the trait
  composition of these lil voxel creatures, though an additive entity
  formula seems quite complicated. this might be a better way to do this
  without prescription, and with delightful surprise/unexpected outcomes.
  fast, slow, and local methods, yes?"
- "the fact that emergent ecologies usually collapse... could actually be
  the Losing is Fun pressure of mesocosm"; "an ecology of one lineage is
  perhaps possible through monoculture, but may be brittle... whereas
  managing the diversity of lineages provides a robust and challenging
  ecosystem"
- "critters could recognize and differentiate player directions and like
  score/trust them by outcomes... so the game has a real consequential
  interactivity at base. i guess i pictured black&white, but with mundane
  creatures that can learn...? or like rimworld designations."
- "for direct control, you have to be able to embody the creature, which
  requires that they be sapient, so if you develop your lineage to sapience
  you get a real advantage in interacting with the world in the roguelike's
  rounds"
- "eponym is meant to be a adv.rpg game built on that, where you join or
  start settlements and learn skills, travel, and in magic worlds perhaps
  journey to divinity, in an emergent civilization of sapient creatures"
- On classifying roles and sapience by what creatures do: "agreed on
  measured classification!"

## 2. What the records already hold

Most of the idea is already ruled in other words. What is new is the
learning, the heritable drives, trust by kind of direction and measured
classification.

| Idea | Already held by | State |
| --- | --- | --- |
| Senses are the creature's own; it suggests what it would do | case 59, Directing 3 | Ruled; suggestions built (`readings::suggestions`) |
| Driving needs sapience; below it, directing | cases 60, 671, Directing 3 | Ruled; Mark restated it 2026-10-31 |
| The bond is earned by how well directions served | cases 177, 688, Mesocosm 2 | Built: one weight per participant and critter (`directing/mod.rs`, `BOND`), moved by served-ness (`directing/choice.rs`) |
| Designations grow from attention | cases 176, 214 to 216 | Ruled; standing orders are derived readings |
| A choice weighs needs, mood, nudges and bond | case 683, Directing 4 | Built: `Method::Deliberative`, a hand-written score |
| Needs gated by traits | `rules/mind.rs` `Need { traits, query, weight }` | Built; weights are world rules, not heritable |
| A small evolved policy at the choice seam | [engine ecology review](2026-08-18_engine_ecology_rulings_and_review.md) §2.6 and B1 | B1 complete 2026-08-21 in `mesocosm-core`: a quantized three-drive recurrent policy, signed byte weights, a decision trace, replay-exact. Deleted with `mesocosm-core`; nothing native replaced it |
| Crowd until noted | Processes 6, sim-execution D2, D24; `population.rs`, `band.rs` | E4a built; E4b (bands act once for all members) open; the sapient-lineage population slice waits on E4 (case 1041) |
| A lineage changes at the epoch boundary, "like the shop between rounds" | Ecology 1 | Built natively in `lineage/` |
| Memory is graded and fades at the holder's rate | Record 3, case 131 | Owed (sim S4) |
| Culture descends like a lineage | Processes 14; founding record, "cultural transmission" | Not built |
| Plasticity belongs to youth | Mesocosm overlay EB2: "Youth buys change with capability" | Not built |
| Collapse, overperformers and disease | Ecology 2, Ecology 3, Mesocosm 3 | Regions and collapse built; disturbances not |
| Promotion by novelty and quality | Record 2 (the hagiograph), case 4, MAP-Elites in the prior-art brief | Novelty gate only (EB5) |

The rounds Mark names are Mesocosm's epochs: Ecology 1 already calls the
boundary "the shop between rounds".

## 3. The shape proposed

### 3.1 Traits make the body, drives make the reward, a policy chooses

Three layers, each heritable:

- **Body traits** say what is possible: parts, functions, senses, intake
  ports. Unchanged.
- **Drives** say what satisfies. A lineage carries a drive vector that
  scales the world's needs for its members: how much hunger, safety, warmth,
  company, digging or novelty count. This is the non-prescriptive seed Mark
  asked for: a burrowing, dig-loving, dry-tolerant line drifts toward rooting
  through bedrock without anyone writing that behaviour. A drive with no body
  to serve it makes a frustrated, strange creature, which is surprise rather
  than a bug.
- **A policy** chooses. A small integer recurrent network per lineage,
  scoring the Choice processes the deliberative chooser already offers. Its
  inputs are the chooser's current terms (pressing needs scaled by drives,
  mood, live nudges and their bonds, sensed facts) plus recurrent state.

The policy sits at the seam §2.6 ruled and B1 built: it proposes, the
resolvers dispose. It never touches legality, cost, conservation, events or
replay. Opaque weights never bypass traits: a policy can choose only among
acts the body and the world admit. B1's finding carries over: a player's
nudge is arithmetically a drive bias, so its authority is bounded in
evolution's own units ("your nudge is worth as much as a strong inherited
disposition, and no more", forms-of-life brief).

### 3.2 Three timescales

| | When | What changes | Where it lives |
| --- | --- | --- | --- |
| **Local** | within a life | the individual's plastic delta on the lineage policy; its trust table (§3.4) | a fading memory (Record 3): it decays at the holder's memory rate unless renewed. Youth learns fast and pays for it (EB2) |
| **Fast** | across a few generations | culture: deltas of members who bred, taught or were imitated fold into the lineage policy | a named world mechanism with a cost, as the founding record requires ("A learned technique becomes heritable only through a named world mechanism, with its costs and transmission limits recorded"); optional per world (Schema 10) |
| **Slow** | at reproduction and the epoch boundary | drive vector, policy size and shape, mutation | Ecology 1's two doors: reproduction rerolls expression, the boundary adopts what prospered |

Folding deltas only works because every member's policy descends from the
same lineage policy: deltas from one ancestor line up, where two unrelated
networks averaged would be noise. Keeping the delta small (low rank or
sparse, signed bytes) keeps it cheap to store and to fold.

### 3.3 The crowd and the individual

The lineage policy is shared, so members with the same inputs make the same
choice: a band (E4) can run the policy once for all its members and stay
exact. A member with a live delta is distinct state, a row of its own in
`population.rs`, so it cannot fold into a band. That matches Record 2: a
thing is collected only when nothing reachable could tell it from its
regenerated self. As a delta fades, the member becomes foldable again;
while it matters, it stays an individual.

What promotes a member out of the crowd is already attention's (Directing 5):
who the player plays, what they pin or examine, the game's own care. Two
additions: the player's pick from a cohort, which starts the member's
plasticity ("primed and started the adaptation"), and novelty, read from the
measured behaviour of §3.5, which the hagiograph already judges.

At every rung the same move nests: individuals in bands, households,
settlements, cultures (Schema 2, "one ledger read more coarsely up the
levels"). Promotion and demotion conserve every account (Processes 6).

### 3.4 Trust by kind of direction

Today a bond is one weight per participant and critter. The proposal keeps
that weight and adds a small table under it: for each kind of direction in
each context (a place to range, a thing to approach, an act toward a target),
a running estimate of how following it turned out against what was
expected. `choice.rs` already measures served-ness (wellbeing before and
after an answered act); the table keeps it per kind.

What follows from that: good directions are taken faster; directions that
hurt kin bring hesitation and then refusal; trust passes on by Mesocosm 2's
world setting (fresh, seeded, whole) and by culture, so a lineage can come to
distrust one kind of order and trust others. This is Black & White's learning
creature, made legible, and RimWorld's designations made consequential: the
designation is a proposal the creature weighs, and Mesocosm 2's "only
directive is a nudge" is unchanged.

The same table serves Eponym, where a sophont's friends weigh its directives
by opinion (case 60), and Processes 13, where an act's score is need times
trust times approval.

### 3.5 Measured classification

Roles and sapience are read from what creatures do, never from weights or
declared kinds (Mark: "agreed on measured classification!").

- **Sources.** The flow record (Schema 9) names every matter move and the
  process that moved it: who ate whom, from which kingdom, by which process.
  Place use, movement and substrate come from the place graph and the
  journal. Most measurements are running totals of what the sim already
  records.
- **Graded membership.** A lineage can be 70% grazer and 30% scavenger;
  show the leading role and keep the rest.
- **Hysteresis.** A role changes only after the measure stays moved for a
  band of epochs, so one lean season does not relabel the web.
- **Unnamed roles are discoveries.** A cluster no named role covers is shown
  as new, "this lineage fills no known role": a Mesocosm goal in itself.
- **Ecology 2's trophic slot reads it.** Regions fill and collapse by trophic
  level; measured diet is the honest source of that level, where today a
  declared trait names it (`rules/directing.rs` `slots`,
  `directing/readings/regions.rs`).
- **Sapience is the same test.** A lineage reads as sapient when its members
  pass behavioural probes: imitation, following a direction never given
  before, signalling to kin, tool use. A brain-capacity trait makes passing
  possible; it is not a switch. Passing admits driving (case 671) and moves
  the lineage up Schema 2's method tiers.

One measured behaviour space serves four consumers: roles, sapience, the
hagiograph's novelty and quality gates (MAP-Elites' behaviour descriptors),
and EB3's goals, which must be "verified achievable and novel".

### 3.6 Embodiment teaches

While a player drives a sapient creature, its policy watches: the player's
choices become imitation examples for its delta. When the player leaves, it
acts a little like them; if it breeds or teaches, a little of the player
passes into the lineage by culture. In Eponym the named denizen is the
furthest case: a companion who learned from you, in a civilization descended
in part from lineages you shaped. Eponym's succession (E3.1) then has a reason
to prefer some heirs over others.

### 3.7 Collapse, diversity and losing

Ecology 3 already names the first disturbances: disease, an overperformer, a
keystone lost. For monoculture to be brittle rather than simply winning, the
pressure has to target whatever is common:

- disease and parasites that pass more easily between bodies with similar
  traits (negative frequency dependence);
- depletion: what everyone eats runs out first;
- predators whose own learning specialises on what is abundant.

Then diversity is the stable strategy, managing several lineages is the
skill, and collapse is Mesocosm's losing-is-fun pressure with instruments to
read it coming: the food web, energy flow, Ecology 3's four measures.

## 4. Cost

Analytic, not measured.

- A useful policy is about 1,000 weights: some thirty inputs, two hidden
  layers of sixteen, a score per offered process. A decision is about 1,000
  integer multiply-adds. B1 ran a smaller one inside the tick.
- Shared weights batch: one policy over a band costs one evaluation.
  Unique weights cost memory: a million members with their own 1,000 signed
  bytes is a gigabyte read every decision. Deltas keep the unique part to a
  few hundred bytes for the few who are promoted.
- Learning costs about one to two times a decision for local rules (Hebbian,
  neuromodulated); gradient methods cost about three times plus stored
  experience, too slow for a creature's life anyway.
- The bound that governs is the tick, not the network. The sim-execution
  plan's aim is 0.1 s a tick with a few hundred members, which no phase has
  reached yet; case 978 found native scheduling alone at about 0.3 s of a
  0.45 s tick. A policy evaluation per deliberative member per decision
  period is small against that, and sensing (per-pair work the traits brief
  estimated at most of the tick past 300 bodies) stays the larger cost.

## 5. Contradictions and tensions

1. **Determinism.** Replay from the seed and the log on every peer (Record 5,
   the sim plan's constraints) rules out floating-point inference and
   learning in the authority path. Weights, deltas, learning rules and
   recurrent state are integers, as B1's were; floats may only propose,
   requantized before authority (§2.6, "Burn proposes, the record disposes";
   case 1044's "each proposal checked and built by the sim, never taken
   unchecked").
2. **One definition for the crowd.** Processes 6: "A process is declarative
   enough for its aggregate to derive from its one definition, so scripted
   hooks run only in the foreground." A learned policy is not declarative.
   Either the population model runs the shared policy over band
   representatives (exact, E4b-shaped) or a rate distilled from it (E6),
   certified against the individual runner like any rate.
3. **Legibility.** Case 59's suggestions and Directing 4 say a suggestion is
   derived from the chooser's own record of what it weighed; Eponym's pillar
   4 says "Legible or it isn't real." A learned policy needs B1's decision
   trace: the inputs, every option's score and the chosen act.
4. **Choice changes come back as questions.** The sim-execution constraints:
   "Anything that changes how a choice is made or kept (683, 813) ... comes
   back to Mark as a question, not an optimisation." Putting a policy in the
   chooser is such a change.
5. **Party control.** Mark's "you control a character or a party in isocosm"
   sits beside VTT 3 ("A party is opt-in and grants no control") and case
   152 ("Outside the world, a player directs only who they play"). It reads
   as one player playing several bodies, which VTT 2 allows; a question if
   it means more.
6. **Mesocosm's person.** Mesocosm's project description says "You are this
   body ... experienced directly", and the founding record's table gives it
   the first person; Mark now says "you direct critters in mesocosm", as
   cases 59 and 175 ruled. The description is maintainer-owned; surfaced,
   not edited.

## 6. Where the work would land

Cues for the plans that own each part, not phases of this brief.

- `sim` S4: deltas and trust as graded, fading memory (Record 3).
- `sim-execution` E4b and E6: the shared policy over bands, or its rate.
- `mesocosm-overlay` M3.2 and M3.3: the policy at the deliberative seam; the
  trust table under the bond; EB2's plasticity as the local learning rate;
  EB3's goals and EB5's significance over the measured behaviour space.
- `sim` S8 to S10: diet measured from the flow record feeding the trophic
  slot; disturbances that press on the common.
- `eponym-overlay` E3.1 and E3.2: imitation while driven; succession
  preferring the companions who learned from you; directives weighed by
  trust.
- `wing-organs`: the hagiograph reading novelty and quality from the same
  behaviour space.
- `sim-host` H2 (inspection is endpoints): a roles-and-trust reading per
  lineage on the Graphshell endpoint.

## 7. Questions for Mark

1. **Where the policy sits.** Does it replace the deliberative scorer's
   hand-written terms, or take them as inputs (proposed), or bias them the
   way B1's biases did?
2. **Who has one.** Every deliberative lineage, or fauna first as in B1,
   with flora growing by a developmental grammar instead (§2.6)?
3. **The cultural mechanism.** Which named mechanism folds deltas into the
   lineage (kin nearby imitating, parent to young, teaching), and what does
   it cost?
4. **The crowd.** Does the population model run the shared policy over band
   representatives, or a rate distilled from it?
5. **Trust's grain.** Per kind of direction, per place, per target lineage,
   or a mix?
6. **Sapience's probes.** Which behaviours count, and does sapience belong to
   a lineage, an individual or both?
7. **Imitation's reach.** Does a driven creature learn from the player only
   in Eponym, or in Mesocosm too once its lineage is sapient?

## 8. Latent synergies

Inside the wing:

- **B1 is the first step, already proven.** Its integer recurrent policy,
  decision trace and replay test are in git history from 2026-08-21; it was
  deleted with `mesocosm-core`, not refuted.
- **EB2 and local learning are one mechanism.** Youth's plasticity is the
  learning rate; "Do not make plasticity free" is the cost.
- **Record 3 and the delta are one memory.** "One memory, graded, over one
  history, from a critter's habit to a sophont's events": a fading delta is
  the habit end of it.
- **One behaviour space, four readers** (§3.5).
- **The deferred contested flow** ("niche overlap can cause a lineage to
  fail, distinct resource paths can coexist", `mesocosm-overlay`) is the
  diversity pressure of §3.7 stated as a done-condition.
- **Mesocosm's unlocks.** The project description's "you unlock the ability
  to play as other lineages as you outpace them" gains a bond to earn: a new
  lineage starts with no trust in you.
- **The VTT.** Lineage temperaments, trust and measured roles are hooks the
  sim offers and the DM decides on (VTT 4).
- **The world-graph viewer (case 1038).** Its ecology filter is where
  measured roles, drives and trust show: lineages as nodes coloured by
  measured role, bonds and trust as edges, and scrubbing back shows a role
  or a culture forming. Branching a past tick tests "what if this lineage
  had trusted the player less" without touching the trunk.
- **The sapient-lineage slice (case 1041).** The region lane's population
  slice for sapient lineages builds on E4; sapience read by §3.5's probes
  is what admits a lineage to it.

## 9. Across the merely-made repos

From a read-only survey of the sibling repos, 2026-10-31, its key claims
checked against source. Nothing outside this repository covers measured
roles, sapience, imitation of a player or cohorts; those are Cosmograph's
own. What exists elsewhere is the delta, adoption, trust and memory
machinery, and a family-wide stance on models.

| Repo | What bears on minds | Code or plan |
| --- | --- | --- |
| esp | LoRA deltas over an immutable base, a deterministic finite-difference trainer and a Burn-autodiff trainer, "bytes in, bytes out"; seeded sampling replays | Code (`crates/esp/src/infer/decoder/{lora,train,train_autodiff}.rs`), llama attention and next-token only. The pattern transfers, the code does not: a lineage policy is integer and tiny |
| gemot | FLoRA: social receipts for federated LoRA rounds, stacking heterogeneous-rank deltas with a signed rank budget and replay independent of arrival order; adoption with a frozen electorate and rollback | Code (`crates/gemot/src/moot/flora.rs`). It carries references and receipts; tensors run elsewhere. The nearest design to §3.2's cultural fold and to EB6's open "who must adopt a shared continuation?" |
| gemot `mien` | A per-viewer standing ledger from followed-through commitments; event-sourced, integer-only, deterministic, lapsing without heartbeats | Code (`crates/mien/src/ledger.rs`). The shape of §3.4's trust table (event-sourced, integer, fading), though nothing in it learns |
| eidetic | `hagiograph` (already used here) judges feats against a high-water mark; `impresa` keeps what a pointable thing has come to be associated with and what has lapsed; adapter manifests and eval tallies | Code. The hagiograph is the promotion signal of §3.3; impresa fits culture and legend |
| mere, conatus | Resident spatial state beside a Burn lane; numen fields with Burn lowering; the Burn utilization brief's remote lane over iroh and murm | Code and brief. Isometer-lens's `resident_ground` example already proves "a Burn pass proposes, Ground accepts the integer consequence, the delta replays without Burn" |
| knot-editor | "a script or a model may propose, never assert; a reading is labelled with what produced it" | Plan rule; the family's stance, the same as §2.6 and case 1044 |

Constraints from the same read:

- Burn stays opt-in. esp: "the default ESP dependency graph remains
  model/runtime free"; Burn types are never exposed from another crate.
  The sim's default build carries no ML dependency; a policy is plain
  integer code.
- Burn pins differ: esp at `=0.22.1`, isometer-lens at `=0.22.0`. Any
  proposer that uses both moves them together first.
- Mere's data-oriented doctrine: "replay from empty must reproduce the
  tables". Learned weights are derived from the log, never stored as
  unexplained state.
- Case 1045: a mechanism moves into a shared crate once a second consumer
  plans it. Cosmograph's integer fold stays its own until then; gemot's
  FLoRA is a reference design, not a dependency.

## 10. Sidequests

Cues, each small and each visible.

- **A roles reading.** Measured diet per lineage from the flow record, on the
  inspection endpoint, beside its declared kingdom and mode. Shows at once
  whether the declared and measured webs differ.
- **B1, native.** The policy at `Method::Deliberative`'s seam behind a world
  rule, absent meaning the hand-written score (Schema 10).
- **Prior art to read before §3.2 is built.** Creatures' Norn brain (drive
  reduction as reward, the closest shipped drives-as-traits); Black & White's
  creature learning; Bibites' evolved brains; Kooijman's energy budgets for
  what a drive costs.
- **An offline proposer.** At world generation or epoch review (§2.6), a
  Burn pass through esp's trainer pattern searches lineage policies and
  hands back quantized candidates the boundary adopts or refuses, behind a
  feature, as case 1044 lets models propose homebrew.
