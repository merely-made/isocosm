# Trait catalogue plan

**Status: plan, 2026-09-15.** Assessment only; no code moved, nothing
committed. Founds the lane the
[glyph expression plan](2026-09-15_glyph_expression_plan.md)'s ruling 1 names,
and answers its finding R2. Consumes the
[traits brief](2026-08-29_traits_and_perception_brief.md)'s rarity ladder and
§8 questions, the [ProcessDef plan](2026-08-01_processdef_plan.md)'s PD3 pack
door, and the
[functional generation plan](2026-09-09_functional_generation_plan.md)'s
blueprints. Answers Mark's 2026-09-04 framing at the foot of the
[trophic grammar plan](2026-09-04_trophic_grammar_plan.md) §4: "make beginning
body types; start investigating a beginning set of traits."

**W1, 2026-09-18:** keep. Tier: mixed. Independently produced the
thirty-shape receipt. Evaluated against the wing design record; see
[2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md)
§2.

A trait here is what the expression plan proved it is in the data: **an
expressed site, `(PartId, ProcessRef)`** (`phenotype/mosaic.rs:68-77`). The
catalogue is the set of definitions a world admits. Nothing below coins a name.

---

## 1. What exists

### 1.1 Five native definitions, four seeded

`process/registry.rs:56-95` holds the whole shipped ruleset, in canonical
order: `mesocosm:contract` (Limb), `mesocosm:fix` (Plate), `mesocosm:intake`
(Mass), `mesocosm:secrete` (Plate, `Seeding::Acquired`), `mesocosm:sense`
(Sensor). Four are `Seeding::Geometry`; only `secrete` is acquired, so only
`secrete` is ever a choice (`process.rs:350-358`). `Process::ALL`
(`process.rs:99-105`) is the parity list.

Identity is `ProcessId { namespace, name }` with `qualified()` returning
`namespace:name` (`process.rs:304-307`) — byte-compatible with wing-glyphs'
`identifier()` (`shared/wing-glyphs/src/lib.rs:32-45`), which is why the
expression table can key on it unconverted. **A definition's rule-bearing bytes
are exactly four things** (`process.rs:391-410`): namespace, name,
`expressed_by`, `seeding`. Labels, notes, file and manifest position and the
native binding are all outside the digest, stated there in so many words.

### 1.2 Admitted definitions: the pack door

`mesocosm-phenotype` is the only door (`crates/mesocosm-phenotype/src/lib.rs:1-60`).
A pack is a manifest plus one JSON file per definition (`pack.rs:24-81`);
`role_of` admits four words and no fifth (`pack.rs:88-96`), `seeding_of` two
(`pack.rs:99-105`), `deny_unknown_fields` on both records, admission
all-or-nothing. `Registry::admit` sorts by qualified id and refuses a repeated
one (`registry.rs:119-124`); `Registry::digest` folds the *sorted* definition
digests, so file order is provably not a rule (`registry.rs:182-189`).

**So the pack door already admits an unbounded number of ids, and that is the
mechanism the catalogue uses.** What it does not admit is an unbounded number
of *rules*: see §2.3.

### 1.3 How a site is placed on a part

- `Mosaic::seed` (`mosaic.rs:135`) builds one mosaic per part from geometry and
  asks `Registry::seeds(role)` (`registry.rs:169-173`), not `admits`. The
  seeded site takes **every** cell, so a part arrives fully committed and a
  second process must take tissue off the first (`mosaic.rs:124-134`).
- `BodyPhenotype::seed` (`phenotype.rs:85-99`) is the only constructor;
  `allocations()` (`:111-117`) yields living parts only.
- `ProcessDef::admits(role)` (`process.rs:423-425`) is the site requirement;
  `BodyPhenotype::develop` is the one validator.
- Bounds: `CELL_QUANTUM = 2` (`mosaic.rs:85`), `MAX_AXIS_CELLS = 4` (`:89`),
  `MAX_CELLS = 64` (`:94`), `MAX_SITES = 8` per part (`:101`).
- The expression plan's trap stands: `BodyProcesses::processes` / `performs`
  (`process.rs:454-467`) answer from geometry, not allocation.

### 1.4 What the trait board can change today

The board is `mesocosm-views/src/review.rs` (PE3b), fed by `World::offers`
(`world/review.rs:270`). Each `Offer` (`:101-122`) carries a candidate
`ConditionId`, a `Score`, `price_mg` — "the development price the next
descendant of this line is charged at its birth" — a founder preview digest and
a `why_not`. Committing goes through `World::revise`, admitted only at the
lineage checkpoint (`world/revise.rs:76-79`), appending a program revision.

What it may offer is narrow. `World::candidates` (`world/adapt.rs:266-300`)
returns the status quo plus **conditions this world has already come to** that
the line does not hold, whose definition resolves and whose site some living
body could carry; nothing proposes a new one. The condition table is fixed at
**two** entries (`discovery/conditions.rs:140`): `mesocosm:endured-hunger` →
`secrete` on a Plate, 5 cells; `mesocosm:plate-eaten` → `fix` on a Plate, 4
cells, plus a lexicon word. Scoring is `net_mg = income − rent`
(`world/adapt.rs:112-114`), `beats` strictly greater (`:120-122`), and a
candidate is taken only when it beats the status quo (`:186-201`).

### 1.5 What functional generation already produces

**Correction to the brief's framing.** `shared/wing-functions` does not
generate "bearer, supply, route, gate, sense, effect". It has four node kinds,
`Source`/`Store`/`Gate`/`Effect` (`network.rs:23-41`); four site roles,
`Source`/`Store`/`Gate`/`Actuator` (`generation.rs:20-26`); three operators,
`Strengthen`/`Project`/`Store` (`operators.rs:9-14`); and routes as
`Edge { from, to, capacity }` (`network.rs:54-59`). No sense node, no bearer.

`generate` (`generation.rs:112`) produces one deterministic blueprint per
admitted `Form` (`Creature`, `Staff`, `:15-18`) over caller-supplied
`BodySite { part, role }`, bounded by `GeneratorSettings` (`:34-58`).
Mesocosm's join is `mesocosm-core/src/functions.rs:16-38`: `live_parts`
refuses a site that is not living. Two limits the plan states itself: charge
is "authored abstract charge units; this does not replace Mesocosm's conserved
matter ledger" (`2026-09-09_functional_generation_plan.md:18-19`), and the
forms "do not yet construct staff geometry or allocate magical tissue" (`:96-99`).

---

## 2. Sizing

### 2.1 The default canon's base count is illustrative, not ruled

§7.4 says "the desired default covers letters, numbers, and symbols; a world
may use only `a`, `b`, and `c`, or a mod may supply a much larger vocabulary"
(`2026-08-06_general_model_plan.md:919-921`). No number is ruled. Three honest
readings of that sentence:

| Reading | Count |
| --- | ---: |
| lowercase letters + digits | 36 |
| both cases + digits | 62 |
| printable ASCII less space (both cases, digits, 32 punctuation marks) | 94 |

Canon size is explicit configuration, not a fixed ceiling (`§7.4`, owners
paragraph; `DEFAULT_MAX_GLYPHS = 4096`, `wing-glyphs/src/lib.rs:25`). The
bench's preset canon holds **one** glyph today
(`mesocosm-genet/src/app/bench/trial/journey.rs:37-46`). **Size the catalogue
against N = 62** and state it as a parameter, not a constant.

### 2.2 The coverage arithmetic

Expression is disjunctive and many-to-many (expression plan §2.2). Two
authored parameters decide everything:

- **k** — expressing traits per glyph. k = 1 makes a single severing
  permanently unembodiable; Mark's ruling says "likely multiple". **k = 3.**
- **m** — glyphs a single trait may express. Unbounded m makes one trait a
  skeleton key and embodiment useless as a mantling gate (the expression plan's
  R2). **m ≤ 4.**

Then the catalogue size T satisfies `N·k/m ≤ T ≤ N·k`: for N = 62, k = 3,
m = 4 that is **47 ≤ T ≤ 186**.

The other wall is the body. An embodied set is bounded by living sites —
`MAX_SITES = 8` per part, few parts on a founder — so a body reaches roughly
5–30 concurrent traits and ~10–30 glyphs of a 62-glyph canon. That is the
desirable shape: you cannot embody the canon, so mantling is a choice under
scarcity rather than a collection counter.

**Proposed target: 60–120 definitions for a 62-base default canon**, first
tranche authored, remainder generated. 60 is the floor that gives every glyph
three expressers with no trait above four glyphs; 120 is where the board's
candidate list and the explanation path stop being readable, and above which
§2.3's saturation makes growth cosmetic. Both numbers are Mark's to move; the
ratio rule `T ∈ [N·k/m, N·k]` is the part worth keeping.

### 2.3 The saturation finding, which binds harder than the count

`ProcessDef::digest` hashes id, `expressed_by` and `seeding` and nothing else
(`process.rs:391-410`). `expressed_by` is a subset of four roles (15 non-empty
subsets) and `seeding` has two values (`process.rs:350-358`), so there are
**30 distinct rule shapes in the whole definition space**. A catalogue of 120
is therefore 120 identities over at most 30 rules; the other 90 differ only by
name, and a name is outside rule authority by the digest's own construction.

Two ways out, both Mark's:

- **Widen `ProcessDef`** with rule-bearing fields the game already has words
  for: a cell cost, a `NisKind`/`IntakePort` declaration (`process.rs:171-182`),
  a flow signature, a payload. Each new field is a digest input, so every world
  digest and every fixture moves — accepted elsewhere this month, but it is a
  world-rule change, not an authoring change.
- **Compose at the site, not in the definition.** The rarity ladder's real
  home: a trait is a *combination* of sites on one part or one body, so the
  variety comes from the mosaic (up to 8 sites per part) rather than from the
  definition. This keeps the digest untouched and is the reading §3.2 takes.

---

## 3. The catalogue's structure

### 3.1 Three kinds of entry, one identity

| Kind | Where it comes from | Identity |
| --- | --- | --- |
| **Base** | an admitted `ProcessDef`, native or pack | `namespace:name` |
| **Composed** | a declared set of two or more base traits co-expressed on one body | `namespace:name`, its own entry |
| **Functional** | a blueprint shape promoted to a trait (§3.5) | `namespace:name` |

Every entry's identity is `ProcessId::qualified()` (`process.rs:304-307`),
which is what the expression table keys on and what `identifier()` validates.
A composed trait's id is a real id, not a formula: the expression table takes
strings, so a composed entry needs no new key type, and `glyphs_of` indexes it
the same way.

### 3.2 Rarity as composition count, priced as constraint

The traits brief's ladder — 1 effect common, 2 uncommon, 3 rare, 4 legendary,
5+ epic — collides with three written rulings, recorded at
`2026-08-29_traits_and_perception_brief.md:567-600`: the founding plan bars the
loot economy by name for this exact mechanic
(`2026-07-30_mesocosm_founding_plan.md:477`), pillar 5 already names where
scarcity lives, and the stop rule is "sample constraints, not powers"
(`2026-08-06_general_model_plan.md:1301-1303`).

**Taken:** the tier as a *derived reading of composition depth* — how many base
traits an entry requires — bound to the price the board already charges
(`Offer.price_mg`, `world/review.rs:107-110`), so depth is legible through cost
rather than a printed word. **Left:** the five borrowed words, which are five
uncleared coinages. A panel says "requires three sites", not "legendary".
Naming the tiers is a naming round and is Mark's.

### 3.3 Placement rules

A catalogue entry declares, and nothing more:

- `expressed_by` — a non-empty subset of the four roles (`pack.rs:88-96`).
- `seeding` — geometry or acquired (`pack.rs:99-105`). **Generated entries are
  `Acquired` by default**: a generated definition that seeded itself would hand
  every plate in the world a free organ, which is the mistake `Seeding` exists
  to prevent (`registry.rs:86-88`).
- a cell cost within `MAX_CELLS = 64` and, for a composed entry, a site count
  within `MAX_SITES = 8`.

The one validator stays the only way in. Nothing in the catalogue may place a
site; it may only be a thing a proposal cites.

### 3.4 Authored versus generated, under the pipeline laws

- **Authored**: the five natives; the first tranche of base entries; every
  composed entry's *declaration* of which bases it requires; the expression
  table itself (ruling 2).
- **Generated**: further base entries over the declared axes, and composed
  entries sampled from the base set.

Law A: what a catalogue entry exports across games is its id and the board's
`(chosen, foregone, price)` record, never a body. Law B: the catalogue keeps a
small loud set — the authored tranche — with generated entries as the quiet
remainder. Law C: authored and generated entries are admitted through the
identical `Registry::admit` path with no flag distinguishing them, which is
already true because the digest excludes provenance.

### 3.5 Promoting a functional blueprint to a trait

Not free, and the join does not exist. A blueprint binds
`BodySite { part, role: SiteRole }` (`generation.rs:20-31`) where `SiteRole` is
`Source`/`Store`/`Gate`/`Actuator`; a trait needs `Role` plus a `ProcessRef`.
So promotion requires an authored table from `SiteRole` to
`(Role, ProcessId)` — authored, because guessing it from part order is exactly
what `functions.rs:25-27` refuses — and a ruling that abstract charge stays
outside the matter ledger (`2026-09-09_functional_generation_plan.md:18-19`).
Until both exist, functional entries are **out of the first tranche** and are
step 6 of §6.

### 3.6 Ownership

`shared/wing-glyphs` owns the expression table and nothing about bodies;
`mesocosm-core` owns `Registry`, `ProcessDef`, resolution and the embodied
reading; `mesocosm-phenotype` owns admission from disk and, new here,
**generation of candidate definitions**, because the core is deterministic,
integer-only and free of I/O. The catalogue ships as pack files under
`packs/mesocosm/processes/`.

---

## 4. Acquisition over epochs

Mark's ruling 3: a lineage takes traits across epochs on the trait board, so a
critter cut down to nothing is a dead end only within its epoch.

**The route that exists**: evidence → `discovery::Condition` → `Candidate`
(`discovery.rs:220-243`) → `World::candidates` → `Offer` → `World::revise` →
program revision → descendants born expressing it, priced at `price_mg`.

**The two gaps**, both real:

1. **The condition table is fixed at two entries** (`conditions.rs:140`). A
   catalogue of 60 needs conditions of the same order, or 58 entries are
   unreachable. The table's own comment says PE4's generated conditions arrive
   "through the same shape, admitted rather than coined"
   (`conditions.rs:135-139`) — that is the door, and it is unbuilt.
2. **The board cannot choose an inert trait.** `Score::net_mg` is income minus
   rent (`adapt.rs:112-114`); a pack-minted definition has `native: None`
   (`process.rs:374`), and nothing outside `Registry::of_native`
   (`registry.rs:158-162`) and its tests reads that field, so no ecology rule
   consumes it. Taking cells for such a site lowers income and raises nothing,
   so it is strictly dominated by the status quo and `beats` refuses it forever.
   Every generated catalogue entry is in this position. Three candidate
   answers, all Mark's: give non-native definitions a declared ecological
   consequence (a port, a payload, an upkeep discount); let the board score
   embodiment as well as matter, which makes glyph coverage a second currency
   and needs a ruling against pillar 5; or admit catalogue traits only through
   the *played* line's review, where the player chooses and the scorer only
   informs. **This plan takes none of them.**

**The manner.** A trait granted on the board and then expressed on a body is
the `embodiment` manner in the experience record, under
`ProvenanceKind::Trait` — a variant that has existed since G1
(`wing-glyphs/src/journey.rs:12-21`) and has never been written. Per Mark's
experience ruling, the manner is recorded with the trigger and is what shapes
the divinity later; the board's own `Event::Revised` and the body's
`Event::Expressed { organism, part, cost_mg }` (`history.rs:151-155`) are the
two accepted facts that back it. Nothing new is logged.

---

## 5. Interaction with the expression table

Ruling 2 closed the shuffle question: **authored, with a pinned per-world
shuffle**. Consequences for the catalogue:

- The authored `ExpressionSpec` (expression plan §2.1) ships; a world may
  permute it by seed exactly as `Canon::shuffled(seed, revision)` does
  (`wing-glyphs/src/canon.rs:172-198`), and **the result is saved into the
  world's definition and never rerun** on an inhabited world.
- `covers(&Canon)` must also be checked against the catalogue: every trait id
  in the table must resolve in the world's `Registry`, or a world admits a
  table naming traits it cannot express. That check belongs in `mesocosm-core`,
  which is the only side holding the registry.
- **Migration**, under §7.4's rule that a change must not "make an old
  collection complete under different meanings": a revision may **add** trait
  ids to a glyph's expresser list freely; **removing** a glyph's last expresser
  is refused at `new`; completion is judged against the journey's founding
  revision while embodiment follows the current one (`journey.rs:231-236`,
  `:249-260`), so a catalogue revision changes what a body embodies today and
  can never retroactively complete a journey.
- Crate split unchanged: wing-glyphs the table, `mesocosm-core` the resolution
  to `ProcessRef` and the reading over `BodyPhenotype`.

---

## 6. Move order

Smallest first; each green and revertible. Steps 1 and 2 give the expression
slice a catalogue larger than five without waiting for any generator.

1. **An authored base tranche, as a pack.** 12 to 20 new `ProcessDef`s in
   `packs/mesocosm/processes/`, all `Acquired`, over the four roles, no native
   binding. Pure data; no core change. *Gate:* `mesocosm-phenotype`'s admission
   suite; `Registry::admit` refuses a duplicate id; the ruleset digest moves
   once and is re-pinned; the native parity receipt in `process/tests.rs` is
   untouched because the native table is.
2. **A catalogue reading, in core.** The `Registry`-side helpers `covers` needs:
   every catalogue id resolves, every entry's `expressed_by` is non-empty, the
   distinct-rule count is reported. *Gate:* core's suite plus
   `tests/move_pins.rs` — no `World` touched, so no pin may move. **This is the
   step that lets the expression slice's `covers` pass against a canon larger
   than one glyph.**
3. **Composed entries, declared.** The set of base ids an entry requires, the
   derived depth reading, and the body predicate "expresses all required bases
   on living attached parts". *Gate:* two of three required bases does not bear
   it; the reading is `&`-only and byte-identical twice over.
4. **The condition door widens.** Enough admitted conditions that the authored
   tranche is reachable on the board, through `conditions()`'s shape rather
   than coined. *Gate:* `World::candidates` offers a catalogue trait; a commit
   through `World::revise` lands a revision; matter conserved to the milligram;
   discovery digests pinned.
5. **Generation, in `mesocosm-phenotype`.** A seeded sampler over the declared
   axes producing candidate `ProcessDef`s and composed declarations, on its own
   salted stream (the traits brief's three draw constraints,
   `2026-08-29_traits_and_perception_brief.md:281-292`), with an inspection
   example and a receipt in the shape `inspect_functions` uses. *Gate:* 50
   seeds; every accepted candidate serializes without regeneration; rejections
   named; one seed byte-identical twice.
6. **Functional promotion.** §3.5's authored `SiteRole` → `(Role, ProcessId)`
   table, gated on Mark's ruling about abstract charge. *Gate:* a promoted
   entry admits, expresses, and is refused on a severed site.

Workspace gates throughout: `cargo check --workspace --all-features
--all-targets` on the Mesocosm workspace, `cargo fmt` clean, and
`structure-cli` and `population` named as excluded with their HEAD failure
hashes, as the expression plan requires.

---

## 7. Risks and open decisions for Mark

**R1. The definition space saturates at 30 rules** (§2.3). A catalogue of 60 to
120 is mostly names over identical rules unless `ProcessDef` gains rule-bearing
fields, which moves every world digest. **Decide before step 1**: widen the
definition, or accept that variety comes from composition at the site.

**R2. The trait board cannot currently take a catalogue trait** (§4, gap 2).
The scorer is income minus rent and an inert definition only costs. This blocks
ruling 3's "a lineage acquires traits over epochs" for everything outside the
five natives. **Needs a ruling before step 4.**

**R3. Rarity tiers, or the repo's own idiom?** Both steelmen are already
written out at `2026-08-29_traits_and_perception_brief.md:709-760`, and Mark
has not ruled. §3.2 proposes depth-as-price and no printed tier words;
adopting the five words is a naming round.

**R4. The catalogue's count is not ruled because the canon's is not.** §2.1
gives 36 / 62 / 94 and this plan sizes on 62. If Mark wants Mesocosm's own
default canon deliberately small — which §7.4 permits explicitly — the
catalogue floor drops proportionally and steps 5 and 6 may never be needed.

**R5. Lineage distance is still `None` for every founder pair**
(`species.rs:224-226`; traits brief §8 question 1), so the proximity half of
Mark's cost formula cannot be written. Out of scope, and named so nobody prices
a catalogue trait on it by accident.

**R6. A rarity-weighted draw is a proposer that knows which traits are good.**
The blind-proposer ruling's old home (`epoch/adapt.rs`) died with the trait
array on 2026-09-04, but the live proposer `World::candidates`
(`world/adapt.rs:266`) still proposes nothing. A weighted generator must sit on
the *authoring* side of the pack door, never inside `candidates`.

---

## 8. Done conditions

1. The authored tranche admits: `mesocosm-phenotype` lowers it to a
   `Registry` whose `len()` is at least 17, with no duplicate qualified id, and
   the ruleset digest is stable across two admissions in different file order.
2. Every base of a canon of at least 26 glyphs has at least three expressing
   trait ids, every trait id expresses at most four bases, and every id in the
   table resolves in the world's registry — asserted, not inspected.
3. The distinct-rule count is reported by a test and asserted against the
   catalogue size, so R1's saturation is visible in CI rather than discovered
   later.
4. A composed entry's predicate is exact: a body expressing all required bases
   on living attached parts bears it; removing any one by severing removes it;
   the reading is `&`-only and byte-identical run twice.
5. A catalogue trait reaches the board: it appears in `World::offers` with a
   `price_mg`, commits through `World::revise`, and a descendant is born
   expressing it, with matter conserved to the milligram across the birth.
6. Migration: a revision that adds an expresser is admitted; one that removes a
   glyph's last expresser is refused at `new`; a journey founded under revision
   1 judges completion against revision 1 while its body embodies under
   revision 2, pinned by a test before a second revision exists.
7. The generation receipt: 50 seeds, every accepted candidate serializing
   without regeneration, rejections named, one seed reproduced byte-identically,
   under `testing/traits/<date>/`.
8. The six move pins, the 24 wing-glyphs tests and the 49 runtime tests stay
   green at every step, with `structure-cli` and `population` named as excluded
   with their HEAD failure hashes.

---

## Findings (2026-09-15)

Measured in the tree today.

- **Counts:** five native definitions, four seeded, one acquired
  (`process/registry.rs:56-95`); two discovery conditions
  (`discovery/conditions.rs:140`); one glyph in the bench canon
  (`bench/trial/journey.rs:37-46`).
- **The definition space holds 30 distinct rules**: 15 non-empty role subsets
  × 2 seedings, from `ProcessDef::digest`'s four inputs (`process.rs:391-410`)
  and the four roles `role_of` admits (`mesocosm-phenotype/src/pack.rs:88-96`).
- **`ProcessDef::native` is read in exactly one non-test place**,
  `Registry::of_native` (`registry.rs:158-162`), so a pack-minted definition
  has no ecological consequence today.
- **The brief's six-name functional list does not match the crate**
  (`network.rs:23-41`, `generation.rs:20-26`, `operators.rs:9-14`).
- **`Offer.price_mg` is already the board's cost** (`world/review.rs:107-110`),
  so acquisition needs no new currency.

## Progress

- **2026-09-15.** Plan written. No code moved, nothing committed.
